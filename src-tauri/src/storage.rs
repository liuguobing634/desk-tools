use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn app_data_file_path(app: &AppHandle, file_name: &str) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录: {error}"))?;

    fs::create_dir_all(&app_dir).map_err(|error| format!("无法创建应用数据目录: {error}"))?;

    Ok(app_dir.join(file_name))
}

pub fn app_data_dir_path(app: &AppHandle, dir_name: &str) -> Result<PathBuf, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录: {error}"))?;

    let target_dir = app_dir.join(dir_name);

    fs::create_dir_all(&target_dir).map_err(|error| format!("无法创建应用数据目录: {error}"))?;

    Ok(target_dir)
}
