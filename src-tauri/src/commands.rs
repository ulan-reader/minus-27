//! Команды, которые вызывает UI через `invoke`. Ошибки уходят строкой.
//! Сама логика в minus27-data: её же использует сайт.

use minus27_core::{
    diary::DayLog,
    state::{Reminder, Settings, SyncInfo},
    weight::Entry,
};
use minus27_data::{Db, Remote, Snapshot, sync::Estimate};
use tauri::{AppHandle, State};

use crate::reminders;

#[tauri::command]
pub fn load(db: State<'_, Db>) -> Snapshot {
    db.snapshot()
}

#[tauri::command]
pub fn save_settings(db: State<'_, Db>, settings: Settings) -> Result<Settings, String> {
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
pub fn save_day(db: State<'_, Db>, day: DayLog) -> Result<Vec<DayLog>, String> {
    db.save_day(day)
}

#[tauri::command]
pub fn import_data(db: State<'_, Db>, data: Snapshot) -> Result<Snapshot, String> {
    db.import(data)
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
pub async fn sync_now(db: State<'_, Db>, remote: State<'_, Remote>) -> Result<Snapshot, String> {
    remote.sync(&db).await
}

#[tauri::command]
pub async fn estimate_food(db: State<'_, Db>, remote: State<'_, Remote>, image: String) -> Result<Estimate, String> {
    remote.estimate_food(&db, &image).await
}

/// Поставить напоминания. `false` — уведомления не разрешены или платформа их не умеет.
#[tauri::command]
pub fn set_reminders(app: AppHandle, list: Vec<Reminder>) -> Result<bool, String> {
    reminders::apply(&app, &list)
}
