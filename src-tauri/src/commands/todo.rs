use crate::storage::app_data_file_path;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    id: String,
    text: String,
    done: bool,
    created_at: u64,
}

#[tauri::command]
pub fn load_todos(app: AppHandle) -> Result<Vec<TodoItem>, String> {
    let file_path = app_data_file_path(&app, "todos.json")?;

    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|error| format!("读取待办数据失败: {error}"))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&content).map_err(|error| format!("解析待办数据失败: {error}"))
}

#[tauri::command]
pub fn save_todos(app: AppHandle, todos: Vec<TodoItem>) -> Result<(), String> {
    let file_path = app_data_file_path(&app, "todos.json")?;
    let content = serde_json::to_string_pretty(&todos)
        .map_err(|error| format!("序列化待办数据失败: {error}"))?;

    fs::write(&file_path, content).map_err(|error| format!("保存待办数据失败: {error}"))
}
