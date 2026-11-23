//! Обёртки над командами из src-tauri/src/commands.rs.

use minus27_core::{
    state::{Settings, SyncInfo},
    weight::Entry,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

async fn call<A: Serialize, R: DeserializeOwned>(cmd: &str, args: &A) -> Result<R, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let out = invoke(cmd, args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("Команда {cmd} не выполнилась")))?;
    serde_wasm_bindgen::from_value(out).map_err(|e| e.to_string())
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
struct SignInArgs<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
pub struct Snapshot {
    pub settings: Settings,
    pub entries: Vec<Entry>,
}

pub async fn load() -> Result<Snapshot, String> {
    call("load", &NoArgs {}).await
}

pub async fn save_settings(s: &Settings) -> Result<(), String> {
    call("save_settings", &SettingsArg { settings: s }).await
}

pub async fn save_entry(e: &Entry) -> Result<Vec<Entry>, String> {
    call("save_entry", &EntryArg { entry: e }).await
}

pub async fn delete_entry(date: &str) -> Result<Vec<Entry>, String> {
    call("delete_entry", &DateArg { date }).await
}

pub async fn sync_info() -> Result<SyncInfo, String> {
    call("sync_info", &NoArgs {}).await
}

pub async fn sign_in(email: &str, password: &str) -> Result<SyncInfo, String> {
    call("sign_in", &SignInArgs { email, password }).await
}

pub async fn sign_out() -> Result<SyncInfo, String> {
    call("sign_out", &NoArgs {}).await
}

pub async fn sync_now() -> Result<Vec<Entry>, String> {
    call("sync_now", &NoArgs {}).await
}
