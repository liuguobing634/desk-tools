mod commands;
mod settings;
mod storage;

use crate::settings::AppSettings;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("无法获取应用数据目录: {error}"))?;

            let settings_path = app_data_dir.join("settings.json");
            let settings = AppSettings::load_or_default(&settings_path);
            let storage =
                crate::settings::create_storage(&settings, &app_data_dir)?;

            app.manage(storage);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::todo::load_todos,
            commands::todo::save_todos,
            commands::note::list_notes,
            commands::note::load_note,
            commands::note::create_note,
            commands::note::save_note,
            commands::note::delete_note,
            commands::note::list_note_groups,
            commands::note::create_note_group,
            commands::note::update_note_group,
            commands::note::delete_note_group,
            commands::note::move_note_to_group,
            commands::note::import_image,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
