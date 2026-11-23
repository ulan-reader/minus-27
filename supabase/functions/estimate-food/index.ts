// Оценка калорий и белка по фото еды.
//
// Приложение шлёт JPEG (base64) с токеном пользователя. Функция:
// 1) списывает одно распознавание из дневного лимита (public.ai_take, supabase/schema.sql);
// 2) отправляет фото в Claude и получает строгий JSON по схеме;
// 3) возвращает { items, note, left_today }.
//
// Ключ Anthropic хранится только в секретах Supabase, в приложение он не попадает.
// Деплой и секреты: см. README, раздел «ИИ: калории по фото».

import Anthropic from "npm:@anthropic-ai/sdk";

const MODEL = Deno.env.get("MINUS27_AI_MODEL") ?? "claude-opus-5-5";
const EFFORT = (Deno.env.get("MINUS27_AI_EFFORT") ?? "low") as "low" | "medium" | "high";
const DAILY_LIMIT = Number(Deno.env.get("MINUS27_AI_DAILY_LIMIT") ?? "10");
/** ~1,5 МБ base64 ≈ 1 МБ JPEG. Приложение само ужимает фото до 1024 px. */
const MAX_IMAGE_CHARS = 1_500_000;

const SUPABASE_URL = Deno.env.get("SUPABASE_URL")!;
const SUPABASE_ANON_KEY = Deno.env.get("SUPABASE_ANON_KEY")!;

// Ключ ИИ необязателен: без него функция отвечает «не настроено», а не падает на старте.
const API_KEY = Deno.env.get("ANTHROPIC_API_KEY");
const client = API_KEY ? new Anthropic({ apiKey: API_KEY }) : null;

const cors = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, apikey, content-type",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...cors, "Content-Type": "application/json" } });
}

const SCHEMA = {
  type: "object",
  properties: {
    items: {
      type: "array",
      items: {
        type: "object",
        properties: {
          name: { type: "string", description: "Название по-русски, коротко: «Гречка отварная»" },
          grams: { type: "integer", description: "Оценка веса порции в граммах" },
          kcal: { type: "number", description: "Калории этой порции" },
          protein: { type: "number", description: "Белок этой порции, г" },
        },
        required: ["name", "grams", "kcal", "protein"],
        additionalProperties: false,
      },
    },
    note: { type: "string", description: "Коротко о неуверенности: скрытое масло, соус и т.п. Пусто, если нечего сказать." },
  },
  required: ["items", "note"],
  additionalProperties: false,
} as const;

const PROMPT = `На фото еда. Определи каждое блюдо или продукт на тарелке и оцени вес порции в граммах,
калории и белок именно этой порции (не на 100 г). Ориентируйся на размер посуды и столовых приборов.
Учитывай вероятное масло при жарке. Если на фото нет еды, верни пустой список и объясни в note.
Названия пиши по-русски.`;

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return new Response(null, { headers: cors });
  if (req.method !== "POST") return json({ message: "Только POST" }, 405);
  if (!client) return json({ message: "Распознавание фото пока не настроено" }, 503);

  const auth = req.headers.get("Authorization") ?? "";
  let body: { image?: string; media_type?: string };
  try {
    body = await req.json();
  } catch {
    return json({ message: "Неверный запрос" }, 400);
  }
  const image = body.image ?? "";
  if (!image || image.length > MAX_IMAGE_CHARS) return json({ message: "Фото не пришло или слишком большое" }, 400);
  const mediaType = body.media_type === "image/png" ? "image/png" : "image/jpeg";

  // Лимит: функция в базе работает от имени пользователя (его токен), чужой счётчик не тронуть.
  const take = await fetch(`${SUPABASE_URL}/rest/v1/rpc/ai_take`, {
    method: "POST",
    headers: { apikey: SUPABASE_ANON_KEY, Authorization: auth, "Content-Type": "application/json" },
    body: JSON.stringify({ p_limit: DAILY_LIMIT }),
  });
  if (!take.ok) return json({ message: "Войди в аккаунт, чтобы распознавать фото" }, 401);
  const left: number = await take.json();
  if (left < 0) return json({ message: `Лимит ${DAILY_LIMIT} фото в день исчерпан, попробуй завтра` }, 429);

  try {
    const response = await client.beta.messages.create({
      model: MODEL,
      max_tokens: 4000,
      betas: ["server-side-fallback-2026-07-01"],
      fallbacks: "default",
      output_config: { effort: EFFORT, format: { type: "json_schema", schema: SCHEMA } },
      messages: [
        {
          role: "user",
          content: [
            { type: "image", source: { type: "base64", media_type: mediaType, data: image } },
            { type: "text", text: PROMPT },
          ],
        },
      ],
    });

    if (response.stop_reason === "refusal") return json({ message: "ИИ не смог обработать это фото" }, 422);
    if (response.stop_reason === "max_tokens") return json({ message: "Слишком сложное фото, попробуй крупнее" }, 422);

    const text = response.content.flatMap((b) => (b.type === "text" ? [b.text] : [])).join("");
    const parsed = JSON.parse(text) as { items: { name: string; grams: number; kcal: number; protein: number }[]; note: string };
    const items = parsed.items.map((i) => ({
      name: i.name,
      grams: Math.max(0, Math.round(i.grams)),
      kcal: Math.max(0, Math.round(i.kcal)),
      protein: Math.max(0, Math.round(i.protein * 10) / 10),
    }));
    return json({ items, note: parsed.note || null, left_today: left });
  } catch (error) {
    if (error instanceof Anthropic.RateLimitError) return json({ message: "ИИ перегружен, попробуй через минуту" }, 503);
    if (error instanceof Anthropic.AuthenticationError) {
      console.error("Неверный ANTHROPIC_API_KEY");
      return json({ message: "ИИ не настроен" }, 500);
    }
    if (error instanceof Anthropic.APIError) {
      console.error(`Claude API ${error.status}: ${error.message}`);
      return json({ message: "Ошибка ИИ, попробуй ещё раз" }, 502);
    }
    console.error(error);
    return json({ message: "Не удалось разобрать ответ ИИ" }, 500);
  }
});
