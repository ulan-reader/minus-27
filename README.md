# Минус 27 🔥

Личное приложение для похудения: **97 → 70 кг**. Android-приложение и сайт (на iPhone ставится на экран «Домой»).
План питания на 7 дней, список покупок и трекер веса с аналитикой.
Всё на Rust: и логика, и интерфейс.

## Возможности

- 🍽️ **План** — меню на неделю в двух режимах: «Оригинал» (~2050 ккал, ~161 г белка) и «Минус» (~1816 ккал, ~155 г белка). Калории и белок по каждому приёму пищи и дню.
- 🛒 **Покупки** — список продуктов на выбранные дни, сгруппированный по категориям, с отметками «куплено» и копированием в буфер.
- ⚖️ **Вес** — записи веса, талии и шагов; скользящее среднее за 7 дней, недельные блоки, темп, вердикт («темп хороший», «медленно две недели подряд»…), прогноз даты цели и график.
- ☁️ **Синхронизация** — необязательная, через Supabase. Без неё всё хранится на устройстве и работает офлайн.

## Стек

| Слой | Технологии |
|---|---|
| Интерфейс | [Leptos 0.8](https://leptos.dev) (CSR → WASM), собирается [Trunk](https://trunkrs.dev) |
| Оболочка | [Tauri 2](https://tauri.app) (Android, также Windows); на сайте — браузер (PWA) |
| Логика | Крейт `minus27-core`, чистый Rust, из зависимостей только `serde` |
| Данные | Крейт `minus27-data`: JSON-файл в приложении, localStorage на сайте |
| Синк | Supabase (Auth + REST) через `reqwest`, один код для приложения и сайта |

## Структура

```
minus-27/
├─ minus-27-core/        🧮 логика: продукты, план, аналитика веса (+ тесты)
├─ minus-27-data/        💾 хранение и синк с Supabase (приложение и сайт)
├─ src/                  🖥️ UI на Leptos
│  ├─ app.rs             состояние и вкладки
│  ├─ api.rs             данные: команды Tauri или minus27-data в браузере
│  └─ tabs/              plan.rs · shop.rs · weight.rs
├─ src-tauri/            ⚙️ Android-приложение (команды для UI)
├─ web/                  🌐 манифест, иконки и service worker сайта
├─ supabase/
│  ├─ schema.sql         таблица и RLS
│  └─ supabase.env       ключи Supabase (не в git)
└─ styles.css
```

UI сам считает план и аналитику через `minus27-core`. За данными он идёт в `api.rs`: в приложении это команды Tauri, на сайте тот же `minus27-data` прямо в браузере.

## Быстрый старт

### Что нужно

- Rust (stable) и цели:
  ```bash
  rustup target add wasm32-unknown-unknown aarch64-linux-android x86_64-linux-android
  ```
- `cargo install trunk --locked` и `cargo install tauri-cli --version "^2" --locked`
- Android Studio с SDK и NDK (для эмулятора — образ Google APIs)

### Запуск

```bash
cargo test -p minus27-core      # тесты логики
cargo tauri dev                 # на компьютере
cargo tauri android dev         # в эмуляторе или на телефоне по USB
trunk serve --open              # сайт на http://localhost:1420
```

## Синхронизация (необязательно)

1. Создай проект в [Supabase](https://supabase.com) и выполни [`supabase/schema.sql`](supabase/schema.sql) в SQL Editor.
2. Создай пользователя: **Authentication → Users → Add user** (с Auto Confirm).
3. Отключи регистрацию для остальных: **Authentication → Sign In / Providers → Allow new users to sign up** — выкл.
4. Скопируй `supabase/supabase.env.example` в `supabase/supabase.env` и впиши URL и **publishable** ключ.

Если `supabase.env` нет, синк просто выключен. Секретный ключ (`sb_secret_…`) в приложение не нужен, вставлять его нельзя.

## Сборка APK

1. Один раз создай ключ подписи (вне репозитория):
   ```bash
   keytool -genkey -v -keystore ~/upload-keystore.jks -storetype JKS -keyalg RSA -keysize 2048 -validity 10000 -alias upload -dname "CN=minus-27"
   ```
2. Создай `src-tauri/gen/android/keystore.properties` (не в git):
   ```properties
   password=пароль_от_ключа
   keyAlias=upload
   storeFile=C:\\Users\\<ты>\\upload-keystore.jks
   ```
3. Собери:
   ```bash
   cargo tauri android build --apk --target aarch64
   ```
   APK: `src-tauri/gen/android/app/build/outputs/apk/universal/release/`.

Ключ и пароль не теряй: без них новую версию не поставить поверх старой.

## Сайт (iPhone и любой браузер)

Сайт собирается из того же кода и публикуется на GitHub Pages при каждом пуше в `master` ([.github/workflows/pages.yml](.github/workflows/pages.yml)).

1. **Settings → Pages → Source: GitHub Actions**.
2. **Settings → Secrets and variables → Actions** — секреты `SUPABASE_URL` и `SUPABASE_ANON_KEY` (publishable).
3. После пуша сайт будет по адресу `https://<логин>.github.io/minus-27/`.

На iPhone: открыть сайт в Safari → «Поделиться» → **«На экран Домой»**. Приложение запускается во весь экран и открывается без сети. Данные хранятся в браузере и синхронизируются через Supabase с Android.

## Как это устроено

- **Даты** считаются как номер дня от 1970-01-01, без часовых поясов.
- **Вердикт** сравнивает недели, где есть минимум 3 взвешивания; первые 1–2 недели в основном уходит вода.
- **Синк**: локальные изменения копятся в очереди и при синке отправляются (upsert/delete), затем с сервера скачиваются все записи. Изменения, сделанные во время синка, не теряются.
- **TLS на Android**: `reqwest` использует встроенные корневые сертификаты (`webpki-root-certs`), потому что системный верификатор на Android требует инициализации через JNI.

---

Цифры калорий приблизительные, по таблицам калорийности. Это не медицинская рекомендация.
