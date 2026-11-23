mod commands;
mod config;
mod storage;
mod sync;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("data.json");
            app.manage(storage::Db::open(Box::new(storage::JsonFile::new(path)))?);
            app.manage(sync::Remote::from_config());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load,
            commands::save_settings,
            commands::save_entry,
            commands::delete_entry,
            commands::sync_info,
            commands::sign_in,
            commands::sign_out,
            commands::sync_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
