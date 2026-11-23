//! Команды, которые вызывает UI через `invoke`. Ошибки уходят строкой.
//! Сама логика в minus27-data: её же использует сайт.

use minus27_core::{
    state::{Settings, SyncInfo},
    weight::Entry,
};
use minus27_data::{Db, Remote, Snapshot};
use tauri::State;

#[tauri::command]
pub fn load(db: State<'_, Db>) -> Snapshot {
    db.snapshot()
}

#[tauri::command]
pub fn save_settings(db: State<'_, Db>, settings: Settings) -> Result<(), String> {
    db.save_settings(settings)
}

#[tauri::command]
pub fn save_entry(db: State<'_, Db>, entry: Entry) -> Result<Vec<Entry>, String> {
    db.save_entry(entry)
}

#[tauri::command]
pub fn delete_entry(db: State<'_, Db>, date: String) -> Result<Vec<Entry>, String> {
    db.delete_entry(&date)
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
