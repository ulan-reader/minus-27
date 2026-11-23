mod commands;
mod reminders;

use minus27_data::{Db, Remote, storage::JsonFile};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("data.json");
            app.manage(Db::open(Box::new(JsonFile::new(path)))?);
            app.manage(Remote::from_config());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load,
            commands::save_settings,
            commands::save_entry,
            commands::delete_entry,
            commands::save_day,
            commands::import_data,
            commands::estimate_food,
            commands::set_reminders,
            commands::sync_info,
            commands::sign_in,
            commands::sign_out,
            commands::sync_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
