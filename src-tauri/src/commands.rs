//! Команды, которые вызывает UI через `invoke`. Ошибки уходят строкой.

use minus27_core::{
    state::{Settings, SyncInfo},
    weight::{self, Entry},
};
use serde::Serialize;
use tauri::State;

use crate::{
    storage::{self, Db, Op},
    sync::Remote,
};

#[derive(Serialize)]
pub struct Snapshot {
    settings: Settings,
    entries: Vec<Entry>,
}

#[tauri::command]
pub fn load(db: State<'_, Db>) -> Snapshot {
    db.read(|d| Snapshot { settings: d.settings.clone(), entries: d.entries.clone() })
}

#[tauri::command]
pub fn save_settings(db: State<'_, Db>, settings: Settings) -> Result<(), String> {
    db.write(|d| d.settings = settings).map_err(|e| e.to_string())
}

/// Сохраняет запись (новую или поверх старой за ту же дату) и возвращает все записи.
#[tauri::command]
pub fn save_entry(db: State<'_, Db>, entry: Entry) -> Result<Vec<Entry>, String> {
    weight::check(&entry, None)?;
    let t = storage::now_ms();
    db.write(|d| {
        d.mark(&entry.date, Op::Up, t);
        weight::upsert(&mut d.entries, Entry { updated_ms: t, ..entry });
        d.entries.clone()
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_entry(db: State<'_, Db>, date: String) -> Result<Vec<Entry>, String> {
    let t = storage::now_ms();
    db.write(|d| {
        if weight::remove(&mut d.entries, &date) {
            d.mark(&date, Op::Del, t);
        }
        d.entries.clone()
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_info(db: State<'_, Db>, remote: State<'_, Remote>) -> SyncInfo {
    remote.info(&db)
}

#[tauri::command]
pub async fn sign_in(
    db: State<'_, Db>,
    remote: State<'_, Remote>,
    email: String,
    password: String,
) -> Result<SyncInfo, String> {
    remote.sign_in(&db, &email, &password).await
}

#[tauri::command]
pub fn sign_out(db: State<'_, Db>, remote: State<'_, Remote>) -> Result<SyncInfo, String> {
    remote.sign_out(&db)
}

#[tauri::command]
pub async fn sync_now(db: State<'_, Db>, remote: State<'_, Remote>) -> Result<Vec<Entry>, String> {
    remote.sync(&db).await
}
