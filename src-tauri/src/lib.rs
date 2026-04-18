mod commands;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
