//! Доступ к данным. Два режима с одинаковым API:
//! - в приложении (Tauri) — команды из src-tauri/src/commands.rs через `invoke`;
//! - на сайте — тот же minus27-data прямо в WASM, данные в localStorage.

use std::rc::Rc;

use minus27_core::{
    diary::DayLog,
    state::{Reminder, Settings, SyncInfo},
    weight::Entry,
};
use minus27_data::{Db, Remote, Snapshot, sync::Estimate, web::LocalStore};
use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Запущены ли внутри Tauri (там есть `window.__TAURI__`, см. withGlobalTauri).
fn tauri() -> bool {
    js_sys::Reflect::get(&js_sys::global(), &"__TAURI__".into()).is_ok_and(|v| !v.is_undefined())
}

async fn call<A: Serialize, R: DeserializeOwned>(cmd: &str, args: &A) -> Result<R, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let out = invoke(cmd, args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("Команда {cmd} не выполнилась")))?;
    serde_wasm_bindgen::from_value(out).map_err(|e| e.to_string())
}

/// Данные сайта: создаются при первом обращении.
struct Web {
    db: Db,
    remote: Remote,
}

thread_local! {
    static WEB: Rc<Web> = Rc::new(Web {
        db: Db::open(Box::new(LocalStore)).expect("LocalStore не возвращает ошибок при чтении"),
        remote: Remote::from_config(),
    });
}

fn web() -> Rc<Web> {
    WEB.with(Rc::clone)
}

#[derive(Serialize)]
struct NoArgs {}

#[derive(Serialize)]
struct SettingsArg<'a> {
    settings: &'a Settings,
}

#[derive(Serialize)]
struct EntryArg<'a> {
    entry: &'a Entry,
}

#[derive(Serialize)]
struct DateArg<'a> {
    date: &'a str,
}

#[derive(Serialize)]
struct DayArg<'a> {
    day: &'a DayLog,
}

#[derive(Serialize)]
struct DataArg<'a> {
    data: &'a Snapshot,
}

#[derive(Serialize)]
struct ImageArg<'a> {
    image: &'a str,
}

#[derive(Serialize)]
struct RemindersArg<'a> {
    list: &'a [Reminder],
}

#[derive(Serialize)]
struct SignInArgs<'a> {
    email: &'a str,
    password: &'a str,
}

pub async fn load() -> Result<Snapshot, String> {
    if tauri() { call("load", &NoArgs {}).await } else { Ok(web().db.snapshot()) }
}

/// Возвращает сохранённые настройки (с новой отметкой времени для синка).
pub async fn save_settings(s: &Settings) -> Result<Settings, String> {
    if tauri() {
        call("save_settings", &SettingsArg { settings: s }).await
    } else {
        web().db.save_settings(s.clone())
    }
}

pub async fn save_entry(e: &Entry) -> Result<Vec<Entry>, String> {
    if tauri() { call("save_entry", &EntryArg { entry: e }).await } else { web().db.save_entry(e.clone()) }
}

pub async fn delete_entry(date: &str) -> Result<Vec<Entry>, String> {
    if tauri() { call("delete_entry", &DateArg { date }).await } else { web().db.delete_entry(date) }
}

pub async fn save_day(d: &DayLog) -> Result<Vec<DayLog>, String> {
    if tauri() { call("save_day", &DayArg { day: d }).await } else { web().db.save_day(d.clone()) }
}

pub async fn import_data(data: &Snapshot) -> Result<Snapshot, String> {
    if tauri() { call("import_data", &DataArg { data }).await } else { web().db.import(data.clone()) }
}

pub async fn estimate_food(image: &str) -> Result<Estimate, String> {
    if tauri() {
        call("estimate_food", &ImageArg { image }).await
    } else {
        let w = web();
        w.remote.estimate_food(&w.db, image).await
    }
}

/// Поставить напоминания. `false` — не поддерживается (сайт, компьютер) или нет разрешения.
pub async fn set_reminders(list: &[Reminder]) -> Result<bool, String> {
    if tauri() { call("set_reminders", &RemindersArg { list }).await } else { Ok(false) }
}

/// Внутри приложения (а не на сайте): там есть напоминания.
pub fn is_app() -> bool {
    tauri()
}

pub async fn sync_info() -> Result<SyncInfo, String> {
    if tauri() {
        call("sync_info", &NoArgs {}).await
    } else {
        let w = web();
        Ok(w.remote.info(&w.db))
    }
}

pub async fn sign_in(email: &str, password: &str) -> Result<SyncInfo, String> {
    if tauri() {
        call("sign_in", &SignInArgs { email, password }).await
    } else {
        let w = web();
        w.remote.sign_in(&w.db, email, password).await
    }
}

pub async fn sign_out() -> Result<SyncInfo, String> {
    if tauri() {
        call("sign_out", &NoArgs {}).await
    } else {
        let w = web();
        w.remote.sign_out(&w.db)
    }
}

pub async fn sync_now() -> Result<Snapshot, String> {
    if tauri() {
        call("sync_now", &NoArgs {}).await
    } else {
        let w = web();
        w.remote.sync(&w.db).await
    }
}
