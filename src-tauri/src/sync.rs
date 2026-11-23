//! Синхронизация с Supabase через REST, без SDK.
//!
//! Схема: локальные изменения копятся в `Data::pending`. `sync` отправляет их
//! (upsert и delete), скачивает все записи с сервера и кладёт их поверх локальных.
//! Изменения, сделанные во время синка, остаются в `pending` и не теряются.
//! Включается заполнением config.rs.

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use minus27_core::{
    state::SyncInfo,
    weight::{self, Entry},
};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    config,
    storage::{self, Db, Op},
};

/// Вход в Supabase. Хранится в data.json, в UI не уходит.
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
        // Корни из webpki-root-certs, а не системные: системный верификатор
        // на Android требует отдельной инициализации через JNI.
        let certs = webpki_root_certs::TLS_SERVER_ROOT_CERTS
            .iter()
            .filter_map(|c| reqwest::Certificate::from_der(c.as_ref()).ok());
        match Client::builder().timeout(Duration::from_secs(20)).tls_certs_only(certs).build() {
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

    /// Возвращает записи после синка. Если синк уже идёт, отдаёт текущие
    /// локальные записи, а идущий синк потом прогонится ещё раз.
    pub async fn sync(&self, db: &Db) -> Result<Vec<Entry>, String> {
        let Remote::Supabase(sb) = self else { return Err(OFF.into()) };
        if sb.busy.swap(true, Ordering::SeqCst) {
            sb.again.store(true, Ordering::SeqCst);
            return Ok(db.read(|d| d.entries.clone()));
        }
        let mut res = sb.run(db).await;
        while res.is_ok() && sb.again.swap(false, Ordering::SeqCst) {
            res = sb.run(db).await;
        }
        sb.again.store(false, Ordering::SeqCst);
        sb.busy.store(false, Ordering::SeqCst);
        res
    }
}

const OFF: &str = "Синхронизация выключена";
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
    async fn run(&self, db: &Db) -> Result<Vec<Entry>, String> {
        let mut s = self.session(db).await?;

        // Первый синк под этим аккаунтом: отправить все локальные записи.
        let now = storage::now_ms();
        db.write(|d| {
            if d.linked.as_deref() != Some(s.user_id.as_str()) {
                let dates: Vec<String> = d.entries.iter().map(|e| e.date.clone()).collect();
                for date in dates {
                    d.mark(&date, Op::Up, now);
                }
                d.linked = Some(s.user_id.clone());
            }
        })
        .map_err(io_err)?;

        let (pending, entries) = db.read(|d| (d.pending.clone(), d.entries.clone()));
        let ups: Vec<Row> = pending
            .iter()
            .filter(|p| p.op == Op::Up)
            .filter_map(|p| entries.iter().find(|e| e.date == p.date))
            .map(|e| Row {
                user_id: Some(s.user_id.clone()),
                entry_date: e.date.clone(),
                weight_kg: e.weight,
                waist_cm: e.waist,
                steps: e.steps,
                updated_at: Some(iso_ms(if e.updated_ms > 0 { e.updated_ms } else { now })),
            })
            .collect();
        let dels: Vec<&str> = pending.iter().filter(|p| p.op == Op::Del).map(|p| p.date.as_str()).collect();

        if !ups.is_empty() {
            let body = serde_json::to_value(&ups).map_err(|e| e.to_string())?;
            self.rest(db, &mut s, Method::POST, "/rest/v1/weight_entries?on_conflict=user_id,entry_date", |r| {
                r.header("Prefer", "resolution=merge-duplicates,return=minimal").json(&body)
            })
            .await?;
        }
        if !dels.is_empty() {
            // Даты вида 2026-11-23, экранировать нечего. Чужие строки отсекает RLS.
            let path = format!("/rest/v1/weight_entries?entry_date=in.({})", dels.join(","));
            self.rest(db, &mut s, Method::DELETE, &path, |r| r.header("Prefer", "return=minimal")).await?;
        }
        // Убираем только отправленное: если запись поменяли во время синка, её t другой.
        db.write(|d| d.pending.retain(|p| !pending.iter().any(|q| q.date == p.date && q.t == p.t)))
            .map_err(io_err)?;

        let mut remote = Vec::new();
        loop {
            let path = format!(
                "/rest/v1/weight_entries?select=entry_date,weight_kg,waist_cm,steps,updated_at&order=entry_date.asc&limit={PAGE}&offset={}",
                remote.len()
            );
            let part: Vec<Row> =
                self.rest(db, &mut s, Method::GET, &path, |r| r).await?.json().await.map_err(net_err)?;
            let n = part.len();
            remote.extend(part);
            if n < PAGE {
                break;
            }
        }

        db.write(|d| {
            let mut next: Vec<Entry> = Vec::with_capacity(remote.len());
            for r in &remote {
                weight::upsert(
                    &mut next,
                    Entry {
                        date: r.entry_date.clone(),
                        weight: r.weight_kg,
                        waist: r.waist_cm,
                        steps: r.steps,
                        updated_ms: r.updated_at.as_deref().and_then(parse_ms).unwrap_or(0),
                    },
                );
            }
            // То, что ещё не ушло на сервер, важнее серверной версии.
            for p in &d.pending {
                match p.op {
                    Op::Up => {
                        if let Some(e) = d.entries.iter().find(|e| e.date == p.date) {
                            weight::upsert(&mut next, e.clone());
                        }
                    }
                    Op::Del => {
                        weight::remove(&mut next, &p.date);
                    }
                }
            }
            d.entries = next;
            d.entries.clone()
        })
        .map_err(io_err)
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
            if !r.status().is_success() {
                return Err(error_text(r).await);
            }
            return Ok(r);
        }
        Err("Сессия истекла, войди заново".into())
    }
}

const NO_NET: &str = "Нет сети";

fn net_err(e: reqwest::Error) -> String {
    if e.is_connect() || e.is_timeout() { format!("{NO_NET}, изменения отправятся при следующем синке") } else { e.to_string() }
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

    fn iso_parse_check() -> i64 {
        (weight::day_num("2026-11-23").unwrap() as i64 * 86_400 + 10 * 3600 + 15 * 60) * 1000
    }
}
