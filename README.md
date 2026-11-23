<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="112" alt="Минус 27">
</p>

<h1 align="center">Минус 27</h1>

<p align="center">
  Приложение для похудения без голодовок: план питания под твою норму, дневник,<br>
  калории по фото и прогресс веса. Android и сайт для любого телефона.<br>
  Всё на Rust — и логика, и интерфейс.
</p>

<p align="center">
  <a href="../../releases/latest">Скачать APK</a> ·
  <a href="https://ulan-reader.github.io/minus-27/">Открыть сайт</a> ·
  <a href="LICENSE">MIT</a>
</p>

---

## Возможности

- **Своя норма.** Анкета при первом запуске: пол, возраст, рост, вес, цель, активность и темп. Калории, белок и вода считаются по формуле Миффлина — Сан Жеора и пересчитываются по среднему весу за неделю.
- **План на 7 дней.** Готовое меню подгоняется под норму. Любой продукт можно заменить (граммы пересчитаются по калориям), изменить вес, убрать или добавить новый.
- **Сегодня.** Кольца калорий, белка и воды, отметка приёмов пищи из плана одним нажатием, быстрый ввод веса, вода по стакану.
- **Калории по фото.** Фото тарелки → ИИ определяет блюда, граммы, калории и белок → проверяешь и добавляешь в дневник.
- **Покупки.** Список на выбранные дни по категориям, с отметками и копированием.
- **Прогресс.** Среднее за 7 дней, недельные блоки, темп, вердикт («темп хороший», «медленно две недели подряд»), прогноз даты цели, график, замеры талии, бёдер и груди.
- **Свои продукты** с КБЖУ с упаковки.
- **Напоминания** на Android: взвеситься, поесть, выпить воды.
- **Синхронизация** между устройствами через Supabase. Без неё всё работает офлайн на устройстве.
- **Экспорт и импорт** всех данных в JSON, вес в CSV.

## Установка

**Android.** Скачай `minus-27-vX.Y.Z.apk` со страницы [Releases](../../releases/latest), открой файл на телефоне и разреши установку.

**iPhone и любой браузер.** Открой [сайт](https://ulan-reader.github.io/minus-27/) в Safari → «Поделиться» → **«На экран Домой»**. Запускается во весь экран и открывается без сети.

Вход нужен только для синхронизации и распознавания фото. Регистрации в приложении нет: аккаунты создаёт владелец своего Supabase-проекта.

## Стек

| Часть | Технологии |
|---|---|
| Интерфейс | [Leptos 0.8](https://leptos.dev) → WASM, сборка [Trunk](https://trunkrs.dev) |
| Приложение | [Tauri 2](https://tauri.app) (Android), плагин уведомлений |
| Логика | `minus27-core`: продукты, план, нормы, дневник, аналитика веса (чистый Rust, с тестами) |
| Данные | `minus27-data`: JSON-файл в приложении, localStorage на сайте, синк с Supabase через `reqwest` |
| Сервер | Supabase: Auth, Postgres с RLS, Edge Function для ИИ |
| ИИ | Claude (Anthropic) через Edge Function, ключ хранится только на сервере |

```
minus-27/
├─ minus-27-core/        логика и тесты
├─ minus-27-data/        хранение, синк, запрос к ИИ
├─ src/                  интерфейс: app.rs, screens/, components.rs, icons.rs
├─ src-tauri/            Android-приложение: команды, напоминания
├─ web/                  манифест PWA, service worker, выбор фото
├─ supabase/
│  ├─ schema.sql                    таблицы, RLS, лимит ИИ
│  ├─ functions/estimate-food/      Edge Function «калории по фото»
│  └─ supabase.env.example          образец ключей
└─ .github/workflows/    сайт на GitHub Pages и APK в Releases
```

## Своя копия

### 1. Supabase

1. Создай проект на [supabase.com](https://supabase.com).
2. **SQL Editor** → вставь [`supabase/schema.sql`](supabase/schema.sql) → **Run**. Файл можно выполнять повторно при обновлениях.
3. **Authentication → Users → Add user** (с Auto Confirm) — для себя и тех, кого пускаешь.
4. **Authentication → Sign In / Providers** → выключи **Allow new users to sign up**.
5. Скопируй `supabase/supabase.env.example` в `supabase/supabase.env` и впиши URL и **publishable**-ключ. Секретный ключ в приложение не нужен.

Без `supabase.env` приложение работает только локально.

### 2. ИИ: калории по фото (необязательно)

Всё приложение работает и без этого шага: план, дневник, вода, вес, покупки, синхронизация, напоминания.
Без ключа не будет только кнопки «Фото еды» — она покажет «Распознавание фото пока не настроено».
Включить можно в любой момент, пересобирать приложение для этого не нужно.

Нужен ключ [Anthropic API](https://console.anthropic.com) и [Supabase CLI](https://supabase.com/docs/guides/cli).

```bash
supabase login
supabase link --project-ref <id-проекта>
supabase secrets set ANTHROPIC_API_KEY=sk-ant-...
supabase functions deploy estimate-food
```

Необязательные секреты функции:

| Секрет | По умолчанию | Что делает |
|---|---|---|
| `MINUS27_AI_DAILY_LIMIT` | `10` | Фото в день на человека |
| `MINUS27_AI_MODEL` | `claude-opus-5-5` | Модель. `claude-haiku-5-5` в разы дешевле, но менее точна |
| `MINUS27_AI_EFFORT` | `low` | Глубина анализа: `low`, `medium`, `high` |

Ориентир стоимости одного фото на модели по умолчанию — около $0,02 (фото ужимается до 1024 px). Лимит в день хранится в таблице `ai_usage`, обойти его из приложения нельзя.

### 3. Запуск

Один раз поставить инструменты:

```bash
rustup target add wasm32-unknown-unknown aarch64-linux-android x86_64-linux-android
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

Дальше из корня проекта:

| Что | Команда |
|---|---|
| Тесты логики и синка | `cargo test -p minus27-core -p minus27-data` |
| Сайт в браузере (http://localhost:1420) | `trunk serve --open` |
| Приложение на компьютере | `cargo tauri dev` |
| Android в эмуляторе или на телефоне по USB | `cargo tauri android dev` |
| APK для установки | `cargo tauri android build --apk --target aarch64` |

Не запускай `android dev` и `android build` одновременно: оба пишут в `dist/`, и сборка упадёт на заблокированном `dist/.trunk-lock`.

### 4. Публикация

**Сайт** собирается и выкладывается на GitHub Pages при каждом пуше в `master` ([pages.yml](.github/workflows/pages.yml)). Нужно: **Settings → Pages → Source: GitHub Actions** и секреты `SUPABASE_URL`, `SUPABASE_ANON_KEY`.

**APK** собирается и прикладывается к релизу при пуше тега ([release.yml](.github/workflows/release.yml)):

```bash
git tag v1.0.0
git push origin v1.0.0
```

Дополнительные секреты: `ANDROID_KEYSTORE_BASE64` (файл `upload-keystore.jks` в base64) и `ANDROID_KEYSTORE_PASSWORD`. Получить base64 в PowerShell:

```bash
[Convert]::ToBase64String([IO.File]::ReadAllBytes("$env:USERPROFILE\upload-keystore.jks")) | Set-Clipboard
```

Для локальной сборки APK (команда в разделе «Запуск») создай `src-tauri/gen/android/keystore.properties` (не в git):

```properties
password=пароль_от_ключа
keyAlias=upload
storeFile=C:\\Users\\<ты>\\upload-keystore.jks
```

Готовый файл: `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`.

Ключ подписи и пароль храни вне компьютера: без них обновление не встанет поверх установленного приложения.

## Как это устроено

- **Норма:** основной обмен × активность − дефицит под темп (не больше 25% расхода и не ниже 1500/1200 ккал). Белок 1,6 г на кг веса, вода 30 мл на кг.
- **План:** встроенное меню масштабируется под норму (порции с шагом 5 г, яйца целыми штуками). Первая правка сохраняет план как свой.
- **Вердикт** сравнивает недели, где есть минимум 3 взвешивания; первые 1–2 недели в основном уходит вода.
- **Синк:** изменения копятся в очереди и при синке отправляются, затем с сервера скачивается всё. Неотправленное важнее серверного, настройки — побеждает более свежая версия.
- **TLS на Android:** `reqwest` берёт встроенные корневые сертификаты (`webpki-root-certs`): системный верификатор на Android требует инициализации через JNI.

## Лицензия

[MIT](LICENSE). Калории и нормы приблизительные и не заменяют консультацию врача. При заболеваниях, беременности и для подростков подбирай питание со специалистом.
