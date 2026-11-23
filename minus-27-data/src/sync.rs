//! Синхронизация с Supabase через REST, без SDK.
//!
//! Три таблицы: weight_entries (вес по дням), day_logs (дневник и вода по дням)
//! и profiles (профиль и настройки, одна строка на человека). Схема: supabase/schema.sql.
//!
//! Локальные изменения копятся в `Data::pending`. `sync` отправляет их,
//! скачивает всё с сервера и кладёт поверх локального. Изменения, сделанные
//! во время синка, остаются в `pending` и не теряются.
//! Включается заполнением supabase/supabase.env (см. config.rs).

use std::sync::atomic::{AtomicBool, Ordering};

use minus27_core::{
    diary::{self, DayLog, DiaryItem},
    state::{Settings, SyncInfo},
    weight::{self, Entry},
};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    config,
    storage::{self, Data, Db, Kind, Op, Snapshot},
};

/// Вход в Supabase. Хранится вместе с данными (data.json или localStorage).
#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    /// Когда истекает access_token, секунды от эпохи.
    pub expires_at: i64,
    pub user_id: String,
    pub email: String,
}

pub enum Remote {
    Off,
    Supabase(Supabase),
}

pub struct Supabase {
    url: String,
    key: String,
    http: Client,
    busy: AtomicBool,
    /// Пока шёл синк, попросили ещё один: прогнать повторно.
    again: AtomicBool,
}

impl Remote {
    pub fn from_config() -> Self {
        if config::SUPABASE_URL.is_empty() || config::SUPABASE_ANON_KEY.is_empty() {
            return Remote::Off;
        }
        match http_client() {
            Ok(http) => Remote::Supabase(Supabase {
                url: config::SUPABASE_URL.trim_end_matches('/').to_string(),
                key: config::SUPABASE_ANON_KEY.to_string(),
                http,
                busy: AtomicBool::new(false),
                again: AtomicBool::new(false),
            }),
            Err(e) => {
                eprintln!("Supabase выключен, не создался HTTP-клиент: {e}");
                Remote::Off
            }
        }
    }

    pub fn info(&self, db: &Db) -> SyncInfo {
        match self {
            Remote::Off => SyncInfo { enabled: false, account: None },
            Remote::Supabase(_) => {
                SyncInfo { enabled: true, account: db.read(|d| d.session.as_ref().map(|s| s.email.clone())) }
            }
        }
    }

    pub async fn sign_in(&self, db: &Db, email: &str, password: &str) -> Result<SyncInfo, String> {
        let Remote::Supabase(sb) = self else { return Err(OFF.into()) };
        let s = sb.auth("password", json!({ "email": email.trim(), "password": password })).await?;
        db.write(|d| d.session = Some(s)).map_err(|e| e.to_string())?;
        Ok(self.info(db))
    }

    pub fn sign_out(&self, db: &Db) -> Result<SyncInfo, String> {
        db.write(|d| d.session = None).map_err(|e| e.to_string())?;
        Ok(self.info(db))
    }

    /// Возвращает все данные после синка. Если синк уже идёт, отдаёт текущие
    /// локальные данные, а идущий синк потом прогонится ещё раз.
    pub async fn sync(&self, db: &Db) -> Result<Snapshot, String> {
        let Remote::Supabase(sb) = self else { return Err(OFF.into()) };
        if sb.busy.swap(true, Ordering::SeqCst) {
            sb.again.store(true, Ordering::SeqCst);
            return Ok(db.snapshot());
        }
        let mut res = sb.run(db).await;
        while res.is_ok() && sb.again.swap(false, Ordering::SeqCst) {
            res = sb.run(db).await;
        }
        sb.again.store(false, Ordering::SeqCst);
        sb.busy.store(false, Ordering::SeqCst);
        res
    }

    /// Оценка калорий по фото через Edge Function `estimate-food` (supabase/functions).
    /// `image` — JPEG в base64. Ключ ИИ живёт только на сервере.
    pub async fn estimate_food(&self, db: &Db, image: &str) -> Result<Estimate, String> {
        let Remote::Supabase(sb) = self else { return Err("Нужна синхронизация с Supabase".into()) };
        let mut s = sb.session(db).await?;
        let body = json!({ "image": image, "media_type": "image/jpeg" });
        let r = sb.rest(db, &mut s, Method::POST, "/functions/v1/estimate-food", |r| r.json(&body)).await?;
        r.json().await.map_err(net_err)
    }
}

/// Ответ ИИ по фото: продукты с оценкой граммов, калорий и белка.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    pub items: Vec<EstimateItem>,
    /// Пояснение ИИ, например «соус не виден, мог быть майонез».
    #[serde(default)]
    pub note: Option<String>,
    /// Сколько фото ещё можно распознать сегодня.
    #[serde(default)]
    pub left_today: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EstimateItem {
    pub name: String,
    pub grams: u32,
    pub kcal: f64,
    pub protein: f64,
}

#[cfg(not(target_arch = "wasm32"))]
fn http_client() -> reqwest::Result<Client> {
    use std::time::Duration;
    // Корни из webpki-root-certs, а не системные: системный верификатор
    // на Android требует отдельной инициализации через JNI.
    let certs = webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .filter_map(|c| reqwest::Certificate::from_der(c.as_ref()).ok());
    Client::builder().timeout(Duration::from_secs(20)).tls_certs_only(certs).build()
}

/// В браузере запросы идут через fetch, TLS и таймауты на его стороне.
#[cfg(target_arch = "wasm32")]
fn http_client() -> reqwest::Result<Client> {
    Client::builder().build()
}

const OFF: &str = "Синхронизация выключена";
/// Edge Function не задеплоена или без ключа ИИ (тот же текст отдаёт сама функция).
const AI_OFF: &str = "Распознавание фото пока не настроено";
const PAGE: usize = 1000;

/// Строка таблицы weight_entries.
#[derive(Serialize, Deserialize)]
struct Row {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    entry_date: String,
    weight_kg: f64,
    waist_cm: Option<f64>,
    steps: Option<u32>,
    #[serde(default)]
    hips_cm: Option<f64>,
    #[serde(default)]
    chest_cm: Option<f64>,
    #[serde(default)]
    updated_at: Option<String>,
}

/// Строка таблицы day_logs: день целиком в jsonb.
#[derive(Serialize, Deserialize)]
struct DayRow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    log_date: String,
    data: DayData,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct DayData {
    items: Vec<DiaryItem>,
    water_ml: u32,
}

/// Строка таблицы profiles: настройки целиком в jsonb.
#[derive(Serialize, Deserialize)]
struct ProfileRow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    data: Settings,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct AuthResp {
    access_token: String,
    refresh_token: String,
    expires_in: Option<i64>,
    expires_at: Option<i64>,
    user: AuthUser,
}

#[derive(Deserialize)]
struct AuthUser {
    id: String,
    #[serde(default)]
    email: Option<String>,
}

impl Supabase {
    async fn run(&self, db: &Db) -> Result<Snapshot, String> {
        let mut s = self.session(db).await?;
        let uid = s.user_id.clone();

        // Первый синк под этим аккаунтом: отправить всё, что есть на устройстве.
        let now = storage::now_ms();
        db.write(|d| {
            if d.linked.as_deref() != Some(uid.as_str()) {
                let keys: Vec<(Kind, String)> = d
                    .entries
                    .iter()
                    .map(|e| (Kind::Weight, e.date.clone()))
                    .chain(d.days.iter().map(|x| (Kind::Day, x.date.clone())))
                    .collect();
                for (k, date) in keys {
                    d.mark(k, &date, Op::Up, now);
                }
                if d.settings.profile.is_some() {
                    d.mark(Kind::Settings, "", Op::Up, now);
                }
                d.linked = Some(uid.clone());
            }
        })
        .map_err(io_err)?;

        // 1. Отправить изменения.
        let (pending, data) = db.read(|d| (d.pending.clone(), d.snapshot()));
        let stamp = |ms: i64| Some(iso_ms(if ms > 0 { ms } else { now }));
        let ups: Vec<Row> = pending
            .iter()
            .filter(|p| p.kind == Kind::Weight && p.op == Op::Up)
            .filter_map(|p| data.entries.iter().find(|e| e.date == p.date))
            .map(|e| Row {
                user_id: Some(uid.clone()),
                entry_date: e.date.clone(),
                weight_kg: e.weight,
                waist_cm: e.waist,
                steps: e.steps,
                hips_cm: e.hips,
                chest_cm: e.chest,
                updated_at: stamp(e.updated_ms),
            })
            .collect();
        let dels: Vec<&str> = pending
            .iter()
            .filter(|p| p.kind == Kind::Weight && p.op == Op::Del)
            .map(|p| p.date.as_str())
            .collect();
        let days: Vec<DayRow> = pending
            .iter()
            .filter(|p| p.kind == Kind::Day)
            .filter_map(|p| data.days.iter().find(|x| x.date == p.date))
            .map(|x| DayRow {
                user_id: Some(uid.clone()),
                log_date: x.date.clone(),
                data: DayData { items: x.items.clone(), water_ml: x.water_ml },
                updated_at: stamp(x.updated_ms),
            })
            .collect();
        let settings_up = pending.iter().any(|p| p.kind == Kind::Settings);

        self.upsert(db, &mut s, "weight_entries", "user_id,entry_date", &ups).await?;
        if !dels.is_empty() {
            // Даты вида 2026-11-23, экранировать нечего. Чужие строки отсекает RLS.
            let path = format!("/rest/v1/weight_entries?entry_date=in.({})", dels.join(","));
            self.rest(db, &mut s, Method::DELETE, &path, |r| r.header("Prefer", "return=minimal")).await?;
        }
        self.upsert(db, &mut s, "day_logs", "user_id,log_date", &days).await?;
        if settings_up {
            let row = ProfileRow {
                user_id: Some(uid.clone()),
                data: data.settings.clone(),
                updated_at: stamp(data.settings.updated_ms),
            };
            self.upsert(db, &mut s, "profiles", "user_id", std::slice::from_ref(&row)).await?;
        }
        // Убираем только отправленное: если что-то поменяли во время синка, его t другой.
        db.write(|d| {
            d.pending.retain(|p| !pending.iter().any(|q| q.kind == p.kind && q.date == p.date && q.t == p.t))
        })
        .map_err(io_err)?;

        // 2. Скачать всё.
        let rows: Vec<Row> = self
            .pull(db, &mut s, "weight_entries", "entry_date,weight_kg,waist_cm,steps,hips_cm,chest_cm,updated_at", "entry_date")
            .await?;
        let day_rows: Vec<DayRow> = self.pull(db, &mut s, "day_logs", "log_date,data,updated_at", "log_date").await?;
        let profile: Option<ProfileRow> =
            self.pull(db, &mut s, "profiles", "data,updated_at", "user_id").await?.into_iter().next();

        // 3. Сервер поверх локального, кроме того, что ещё не ушло.
        db.write(|d| {
            merge(d, rows, day_rows, profile);
            d.snapshot()
        })
        .map_err(io_err)
    }

    async fn upsert<T: Serialize>(
        &self,
        db: &Db,
        s: &mut Session,
        table: &str,
        conflict: &str,
        rows: &[T],
    ) -> Result<(), String> {
        if rows.is_empty() {
            return Ok(());
        }
        let body = serde_json::to_value(rows).map_err(|e| e.to_string())?;
        let path = format!("/rest/v1/{table}?on_conflict={conflict}");
        self.rest(db, s, Method::POST, &path, |r| r.header("Prefer", "resolution=merge-duplicates,return=minimal").json(&body))
            .await
            .map(|_| ())
    }

    /// Все строки пользователя из таблицы, постранично.
    async fn pull<T: serde::de::DeserializeOwned>(
        &self,
        db: &Db,
        s: &mut Session,
        table: &str,
        select: &str,
        order: &str,
    ) -> Result<Vec<T>, String> {
        let mut out = Vec::new();
        loop {
            let path = format!("/rest/v1/{table}?select={select}&order={order}.asc&limit={PAGE}&offset={}", out.len());
            let part: Vec<T> = self.rest(db, s, Method::GET, &path, |r| r).await?.json().await.map_err(net_err)?;
            let n = part.len();
            out.extend(part);
            if n < PAGE {
                return Ok(out);
            }
        }
    }

    /// Текущая сессия, обновлённая заранее, если токен вот-вот истечёт.
    async fn session(&self, db: &Db) -> Result<Session, String> {
        let s = db.read(|d| d.session.clone()).ok_or("Не выполнен вход")?;
        if s.expires_at - 60 > storage::now_ms() / 1000 {
            return Ok(s);
        }
        self.refresh(db, &s).await
    }

    async fn refresh(&self, db: &Db, s: &Session) -> Result<Session, String> {
        match self.auth("refresh_token", json!({ "refresh_token": s.refresh_token })).await {
            Ok(fresh) => {
                db.write(|d| d.session = Some(fresh.clone())).map_err(io_err)?;
                Ok(fresh)
            }
            Err(e) if e.starts_with(NO_NET) => Err(e),
            Err(_) => {
                db.write(|d| d.session = None).map_err(io_err)?;
                Err("Сессия истекла, войди заново".into())
            }
        }
    }

    async fn auth(&self, grant: &str, body: Value) -> Result<Session, String> {
        let r = self
            .http
            .post(format!("{}/auth/v1/token?grant_type={grant}", self.url))
            .header("apikey", &self.key)
            .json(&body)
            .send()
            .await
            .map_err(net_err)?;
        if !r.status().is_success() {
            return Err(match error_text(r).await.as_str() {
                "Invalid login credentials" => "Неверный email или пароль".to_string(),
                "Email not confirmed" => "Email не подтверждён".to_string(),
                other => other.to_string(),
            });
        }
        let a: AuthResp = r.json().await.map_err(net_err)?;
        let now = storage::now_ms() / 1000;
        Ok(Session {
            access_token: a.access_token,
            refresh_token: a.refresh_token,
            expires_at: a.expires_at.unwrap_or(now + a.expires_in.unwrap_or(3600)),
            user_id: a.user.id,
            email: a.user.email.unwrap_or_default(),
        })
    }

    /// Запрос к REST API от имени пользователя. На 401 один раз обновляет токен и повторяет.
    async fn rest(
        &self,
        db: &Db,
        s: &mut Session,
        method: Method,
        path: &str,
        build: impl Fn(RequestBuilder) -> RequestBuilder,
    ) -> Result<Response, String> {
        for attempt in 0..2 {
            let req = self
                .http
                .request(method.clone(), format!("{}{path}", self.url))
                .header("apikey", &self.key)
                .bearer_auth(&s.access_token);
            let r = build(req).send().await.map_err(net_err)?;
            if r.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
                *s = self.refresh(db, s).await?;
                continue;
            }
            if r.status() == StatusCode::NOT_FOUND && path.starts_with("/functions/") {
                return Err(AI_OFF.into());
            }
            if !r.status().is_success() {
                return Err(error_text(r).await);
            }
            return Ok(r);
        }
        Err("Сессия истекла, войди заново".into())
    }
}

/// Слить скачанное с локальным. Всё, что ещё лежит в `pending`, важнее серверной версии.
fn merge(d: &mut Data, rows: Vec<Row>, day_rows: Vec<DayRow>, profile: Option<ProfileRow>) {
    let mut entries: Vec<Entry> = Vec::with_capacity(rows.len());
    for r in rows {
        weight::upsert(
            &mut entries,
            Entry {
                date: r.entry_date,
                weight: r.weight_kg,
                waist: r.waist_cm,
                steps: r.steps,
                hips: r.hips_cm,
                chest: r.chest_cm,
                updated_ms: r.updated_at.as_deref().and_then(parse_ms).unwrap_or(0),
            },
        );
    }
    for p in d.pending.iter().filter(|p| p.kind == Kind::Weight) {
        match p.op {
            Op::Up => {
                if let Some(e) = d.entries.iter().find(|e| e.date == p.date) {
                    weight::upsert(&mut entries, e.clone());
                }
            }
            Op::Del => {
                weight::remove(&mut entries, &p.date);
            }
        }
    }

    let mut days: Vec<DayLog> = Vec::with_capacity(day_rows.len());
    for r in day_rows {
        let updated_ms = r.updated_at.as_deref().and_then(parse_ms).unwrap_or(0);
        diary::upsert(&mut days, DayLog { date: r.log_date, items: r.data.items, water_ml: r.data.water_ml, updated_ms });
    }
    for p in d.pending.iter().filter(|p| p.kind == Kind::Day) {
        if let Some(x) = d.days.iter().find(|x| x.date == p.date) {
            diary::upsert(&mut days, x.clone());
        }
    }

    // Настройки: побеждает более свежая версия. Вкладка и день остаются свои.
    let settings_pending = d.pending.iter().any(|p| p.kind == Kind::Settings);
    if let Some(p) = profile
        && !settings_pending
        && p.data.updated_ms > d.settings.updated_ms
    {
        let (tab, day) = (d.settings.tab, d.settings.day);
        d.settings = Settings { tab, day, ..p.data };
    }
    d.entries = entries;
    d.days = days;
}

const NO_NET: &str = "Нет сети";

fn net_err(e: reqwest::Error) -> String {
    if is_offline(&e) { format!("{NO_NET}, изменения отправятся при следующем синке") } else { e.to_string() }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_offline(e: &reqwest::Error) -> bool {
    e.is_connect() || e.is_timeout()
}

/// В браузере сбой fetch (нет сети, CORS) виден только как ошибка запроса.
#[cfg(target_arch = "wasm32")]
fn is_offline(e: &reqwest::Error) -> bool {
    e.is_request() || e.is_timeout()
}

fn io_err(e: std::io::Error) -> String {
    e.to_string()
}

/// Сообщение об ошибке из ответа Supabase (у auth и REST поля называются по-разному).
async fn error_text(r: Response) -> String {
    let status = r.status();
    let v: Value = r.json().await.unwrap_or(Value::Null);
    ["error_description", "msg", "message", "error"]
        .iter()
        .find_map(|k| v.get(*k).and_then(Value::as_str))
        .map(str::to_string)
        .unwrap_or_else(|| format!("HTTP {status}"))
}

/// мс от эпохи → "2026-11-23T10:15:00.000Z"
fn iso_ms(ms: i64) -> String {
    let (day, rest) = (ms.div_euclid(86_400_000), ms.rem_euclid(86_400_000));
    let (h, m, s, milli) = (rest / 3_600_000, rest / 60_000 % 60, rest / 1000 % 60, rest % 1000);
    format!("{}T{h:02}:{m:02}:{s:02}.{milli:03}Z", weight::iso_of(day as i32))
}

/// "2026-11-23T10:15:00.123+00:00" → мс. Supabase отдаёт время в UTC, смещение не разбираем.
fn parse_ms(s: &str) -> Option<i64> {
    let day = weight::day_num(s.get(0..10)?)? as i64;
    let h: i64 = s.get(11..13)?.parse().ok()?;
    let m: i64 = s.get(14..16)?.parse().ok()?;
    let sec: i64 = s.get(17..19)?.parse().ok()?;
    let milli = match s.get(19..20) {
        Some(".") => {
            let frac: String = s[20..].chars().take_while(|c| c.is_ascii_digit()).take(3).collect();
            format!("{frac:0<3}").parse().unwrap_or(0)
        }
        _ => 0,
    };
    Some(((day * 24 + h) * 60 + m) * 60_000 + sec * 1000 + milli)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_roundtrip() {
        let ms = 1_795_448_241_788;
        assert_eq!(parse_ms(&iso_ms(ms)), Some(ms));
        assert_eq!(iso_ms(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(parse_ms("2026-11-23T10:15:00.5+00:00"), parse_ms("2026-11-23T10:15:00.500Z"));
        assert_eq!(parse_ms("2026-11-23T10:15:00+00:00"), Some(iso_parse_check()));
    }

    fn row(date: &str, weight: f64) -> Row {
        Row {
            user_id: None,
            entry_date: date.into(),
            weight_kg: weight,
            waist_cm: None,
            steps: None,
            hips_cm: None,
            chest_cm: None,
            updated_at: None,
        }
    }

    #[test]
    fn merge_keeps_unsent_changes() {
        let mut d = Data::default();
        // локально: вес за 23-е изменён и ещё не отправлен, 22-е удалено
        d.entries = vec![Entry { date: "2026-11-23".into(), weight: 90.0, ..Entry::default() }];
        d.mark(Kind::Weight, "2026-11-23", Op::Up, 1);
        d.mark(Kind::Weight, "2026-11-22", Op::Del, 1);
        let mut day = DayLog::new("2026-11-23");
        day.water_ml = 500;
        d.days = vec![day];
        d.mark(Kind::Day, "2026-11-23", Op::Up, 1);
        d.settings.updated_ms = 10;

        let server_day = DayRow {
            user_id: None,
            log_date: "2026-11-21".into(),
            data: DayData { items: vec![], water_ml: 1000 },
            updated_at: None,
        };
        let remote_settings = Settings { updated_ms: 20, day: 5, ..Settings::default() };
        let profile = ProfileRow { user_id: None, data: remote_settings, updated_at: None };
        d.settings.day = 2;

        merge(
            &mut d,
            vec![row("2026-11-21", 95.0), row("2026-11-22", 94.0), row("2026-11-23", 93.0)],
            vec![server_day],
            Some(profile),
        );

        let w: Vec<_> = d.entries.iter().map(|e| (e.date.as_str(), e.weight)).collect();
        assert_eq!(w, [("2026-11-21", 95.0), ("2026-11-23", 90.0)]);
        let days: Vec<_> = d.days.iter().map(|x| (x.date.as_str(), x.water_ml)).collect();
        assert_eq!(days, [("2026-11-21", 1000), ("2026-11-23", 500)]);
        // более свежие настройки с сервера приняты, но выбранный день остался свой
        assert_eq!(d.settings.updated_ms, 20);
        assert_eq!(d.settings.day, 2);
    }

    fn iso_parse_check() -> i64 {
        (weight::day_num("2026-11-23").unwrap() as i64 * 86_400 + 10 * 3600 + 15 * 60) * 1000
    }
}
