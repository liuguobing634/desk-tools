use crate::storage::app_data_dir_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::AppHandle;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    group_id: Option<String>,
    id: String,
    title: String,
    file_name: String,
    updated_at: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDocument {
    id: String,
    title: String,
    file_name: String,
    content: String,
    updated_at: u64,
    group_id: Option<String>,
}

fn notes_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_dir_path(app, "notes")
}

fn note_file_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    Ok(notes_dir(app)?.join(format!("{id}.md")))
}

fn extract_title(content: &str, fallback: &str) -> String {
    content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let title = trimmed.trim_start_matches('#').trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }

            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }

            None
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn file_updated_at(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("读取笔记元数据失败: {error}"))?;
    let modified = metadata
        .modified()
        .map_err(|error| format!("读取笔记修改时间失败: {error}"))?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("解析笔记修改时间失败: {error}"))?;

    Ok(duration.as_secs())
}

fn note_id_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
}

fn note_document_from_path(path: &Path) -> Result<NoteDocument, String> {
    let id = note_id_from_path(path).ok_or_else(|| "无效的笔记文件名".to_string())?;
    let content = fs::read_to_string(path).map_err(|error| format!("读取笔记失败: {error}"))?;
    let title = extract_title(&content, "未命名笔记");
    let updated_at = file_updated_at(path)?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "无法解析笔记文件名".to_string())?
        .to_string();

    // 解析分组信息
    let group_id = parse_group_id_from_content(&content);

    Ok(NoteDocument {
        id,
        title,
        file_name,
        content,
        updated_at,
        group_id,
    })
}

fn parse_group_id_from_content(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("<!-- group: ") && trimmed.ends_with(" -->") {
                let group_id = trimmed
                    .trim_start_matches("<!-- group: ")
                    .trim_end_matches(" -->")
                    .trim();
                if !group_id.is_empty() {
                    return Some(group_id.to_string());
                }
            }
            None
        })
}

fn strip_group_meta(content: &str) -> String {
    let lines = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("<!-- group: ") && trimmed.ends_with(" -->"))
        })
        .collect::<Vec<_>>();

    let mut normalized = lines.join("\n");
    if content.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn with_group_meta(content: &str, group_id: Option<&str>) -> String {
    let mut normalized = strip_group_meta(content).trim_end().to_string();

    if let Some(group_id) = group_id.filter(|value| !value.trim().is_empty()) {
        if !normalized.is_empty() {
            normalized.push_str("\n\n");
        }
        normalized.push_str(&format!("<!-- group: {} -->", group_id));
        normalized.push('\n');
    } else if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}

#[tauri::command]
pub fn list_notes(app: AppHandle) -> Result<Vec<NoteSummary>, String> {
    let dir = notes_dir(&app)?;
    let mut notes = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|error| format!("读取笔记目录失败: {error}"))? {
        let entry = entry.map_err(|error| format!("读取笔记目录项失败: {error}"))?;
        let path = entry.path();

        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let document = note_document_from_path(&path)?;
        notes.push(NoteSummary {
            group_id: document.group_id,
            id: document.id,
            title: document.title,
            file_name: document.file_name,
            updated_at: document.updated_at,
        });
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| a.title.cmp(&b.title)));

    Ok(notes)
}

#[tauri::command]
pub fn load_note(app: AppHandle, id: String) -> Result<NoteDocument, String> {
    let path = note_file_path(&app, &id)?;

    if !path.exists() {
        return Err("笔记不存在".to_string());
    }

    note_document_from_path(&path)
}

#[tauri::command]
pub fn create_note(app: AppHandle, title: String, group_id: Option<String>) -> Result<NoteDocument, String> {
    let note_id = format!("note-{}", uuid::Uuid::new_v4().simple());
    let safe_title = if title.trim().is_empty() {
        "未命名笔记"
    } else {
        title.trim()
    };
    let content = format!("# {safe_title}\n\n");
    let content = with_group_meta(&content, group_id.as_deref());
    let path = note_file_path(&app, &note_id)?;

    fs::write(&path, content).map_err(|error| format!("创建笔记失败: {error}"))?;

    note_document_from_path(&path)
}

#[tauri::command]
pub fn save_note(app: AppHandle, id: String, content: String) -> Result<NoteDocument, String> {
    let path = note_file_path(&app, &id)?;
    let current_group_id = if path.exists() {
        note_document_from_path(&path)?.group_id
    } else {
        None
    };
    let content = with_group_meta(&content, current_group_id.as_deref());

    fs::write(&path, content).map_err(|error| format!("保存笔记失败: {error}"))?;

    note_document_from_path(&path)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, id: String) -> Result<(), String> {
    let path = note_file_path(&app, &id)?;

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path).map_err(|error| format!("删除笔记失败: {error}"))
}

// 分组相关命令
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteGroup {
    id: String,
    name: String,
    color: Option<String>,
    created_at: u64,
    updated_at: u64,
}

fn groups_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_dir_path(app, "groups")
}

fn images_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app_data_dir_path(app, "images")
}

fn group_file_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    Ok(groups_dir(app)?.join(format!("{id}.json")))
}

#[tauri::command]
pub fn list_note_groups(app: AppHandle) -> Result<Vec<NoteGroup>, String> {
    let dir = groups_dir(&app)?;
    
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|error| format!("创建分组目录失败: {error}"))?;
        return Ok(Vec::new());
    }

    let mut groups = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|error| format!("读取分组目录失败: {error}"))? {
        let entry = entry.map_err(|error| format!("读取分组目录项失败: {error}"))?;
        let path = entry.path();

        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|error| format!("读取分组文件失败: {error}"))?;
        let group: NoteGroup = serde_json::from_str(&content)
            .map_err(|error| format!("解析分组数据失败: {error}"))?;

        groups.push(group);
    }

    groups.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.created_at.cmp(&b.created_at)));

    Ok(groups)
}

#[tauri::command]
pub fn create_note_group(app: AppHandle, name: String, color: Option<String>) -> Result<NoteGroup, String> {
    let group_id = format!("group-{}", uuid::Uuid::new_v4().simple());
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("获取当前时间失败: {error}"))?
        .as_secs();

    let group = NoteGroup {
        id: group_id.clone(),
        name: name.trim().to_string(),
        color,
        created_at: now,
        updated_at: now,
    };

    let path = group_file_path(&app, &group_id)?;
    let content = serde_json::to_string_pretty(&group)
        .map_err(|error| format!("序列化分组数据失败: {error}"))?;

    fs::write(&path, content).map_err(|error| format!("创建分组失败: {error}"))?;

    Ok(group)
}

#[tauri::command]
pub fn update_note_group(
    app: AppHandle,
    id: String,
    name: String,
    color: Option<String>,
) -> Result<NoteGroup, String> {
    let path = group_file_path(&app, &id)?;

    if !path.exists() {
        return Err("分组不存在".to_string());
    }

    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("获取当前时间失败: {error}"))?
        .as_secs();

    let mut group: NoteGroup = serde_json::from_str(
        &fs::read_to_string(&path).map_err(|error| format!("读取分组数据失败: {error}"))?
    ).map_err(|error| format!("解析分组数据失败: {error}"))?;

    group.name = name.trim().to_string();
    group.color = color;
    group.updated_at = now;

    let content = serde_json::to_string_pretty(&group)
        .map_err(|error| format!("序列化分组数据失败: {error}"))?;

    fs::write(&path, content).map_err(|error| format!("更新分组失败: {error}"))?;

    Ok(group)
}

#[tauri::command]
pub fn delete_note_group(app: AppHandle, id: String) -> Result<(), String> {
    let path = group_file_path(&app, &id)?;

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path).map_err(|error| format!("删除分组失败: {error}"))
}

#[tauri::command]
pub fn move_note_to_group(
    app: AppHandle,
    note_id: String,
    group_id: Option<String>,
) -> Result<NoteDocument, String> {
    let path = note_file_path(&app, &note_id)?;

    if !path.exists() {
        return Err("笔记不存在".to_string());
    }

    // 读取当前笔记内容
    let mut document = note_document_from_path(&path)?;
    
    // 更新分组ID
    document.group_id = group_id.clone();

    // 重新写入文件，更新分组元数据
    let content = with_group_meta(&document.content, group_id.as_deref());

    // 保存更新后的内容
    fs::write(&path, content).map_err(|error| format!("移动笔记失败: {error}"))?;

    // 重新读取更新后的文档
    note_document_from_path(&path)
}

#[tauri::command]
pub fn import_image(app: AppHandle) -> Result<Option<String>, String> {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"])
        .pick_file()
    {
        let images_dir = images_dir(&app)?;
        if !images_dir.exists() {
            fs::create_dir_all(&images_dir).map_err(|error| format!("创建图片目录失败: {error}"))?;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        let unique_name = format!("{}.{}", uuid::Uuid::new_v4().simple(), extension);
        let dest_path = images_dir.join(&unique_name);

        fs::copy(&path, &dest_path).map_err(|error| format!("复制图片失败: {error}"))?;

        let dest_path_str = dest_path
            .to_str()
            .ok_or_else(|| "图片路径包含无效字符".to_string())?
            .to_string();

        Ok(Some(dest_path_str))
    } else {
        Ok(None)
    }
}
