use crate::settings::AppSettings;
use crate::storage::app_data_file_path;
use tauri::AppHandle;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = app_data_file_path(&app, "settings.json")?;
    Ok(AppSettings::load_or_default(&path))
}

#[tauri::command]
pub fn update_settings(app: AppHandle, settings: AppSettings) -> Result<String, String> {
    let path = app_data_file_path(&app, "settings.json")?;
    settings.save(&path)?;
    Ok("设置已保存。请重启应用以应用新的存储模式。".to_string())
}
