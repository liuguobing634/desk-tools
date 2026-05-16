use crate::storage::ActiveStorage;
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

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

// ── helpers ────────────────────────────────────────────────

fn note_key(id: &str) -> String {
    format!("notes/{id}.md")
}

fn group_key(id: &str) -> String {
    format!("groups/{id}.json")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn build_note_document(key: &str, content: &str, updated_at: u64) -> Result<NoteDocument, String> {
    let id = key
        .strip_prefix("notes/")
        .and_then(|s| s.strip_suffix(".md"))
        .ok_or_else(|| "无效的笔记键".to_string())?
        .to_string();
    let title = extract_title(content, "未命名笔记");
    let group_id = parse_group_id_from_content(content);
    let file_name = key
        .split('/')
        .last()
        .unwrap_or(key)
        .to_string();

    Ok(NoteDocument {
        id,
        title,
        file_name,
        content: content.to_string(),
        updated_at,
        group_id,
    })
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

// ── note commands ──────────────────────────────────────────

#[tauri::command]
pub fn list_notes(storage: tauri::State<'_, ActiveStorage>) -> Result<Vec<NoteSummary>, String> {
    let objects = storage.list_objects("notes/")?;
    let mut notes = Vec::new();

    for obj in &objects {
        if !obj.key.ends_with(".md") {
            continue;
        }

        let content_bytes = storage.get_object(&obj.key)?;
        let content =
            String::from_utf8(content_bytes).map_err(|error| format!("解析笔记内容失败: {error}"))?;
        let title = extract_title(&content, "未命名笔记");
        let group_id = parse_group_id_from_content(&content);
        let id = obj
            .key
            .strip_prefix("notes/")
            .and_then(|s| s.strip_suffix(".md"))
            .unwrap_or(&obj.key)
            .to_string();
        let file_name = obj.key.split('/').last().unwrap_or(&obj.key).to_string();

        notes.push(NoteSummary {
            group_id,
            id,
            title,
            file_name,
            updated_at: obj.updated_at,
        });
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| a.title.cmp(&b.title)));

    Ok(notes)
}

#[tauri::command]
pub fn load_note(
    storage: tauri::State<'_, ActiveStorage>,
    id: String,
) -> Result<NoteDocument, String> {
    let key = note_key(&id);
    log::debug!("load_note: id={id}, key={key}");

    if !storage.object_exists(&key)? {
        return Err("笔记不存在".to_string());
    }

    let info = storage.get_object_info(&key)?;
    let content_bytes = storage.get_object(&key)?;
    let content =
        String::from_utf8(content_bytes).map_err(|error| format!("解析笔记内容失败: {error}"))?;

    log::debug!("load_note: loaded key={key}, size={}", content.len());
    build_note_document(&key, &content, info.updated_at)
}

#[tauri::command]
pub fn create_note(
    storage: tauri::State<'_, ActiveStorage>,
    title: String,
    group_id: Option<String>,
) -> Result<NoteDocument, String> {
    let note_id = format!("note-{}", uuid::Uuid::new_v4().simple());
    let safe_title = if title.trim().is_empty() {
        "未命名笔记"
    } else {
        title.trim()
    };
    let content = format!("# {safe_title}\n\n");
    let content = with_group_meta(&content, group_id.as_deref());
    let key = note_key(&note_id);

    storage.put_object(&key, content.as_bytes())?;

    build_note_document(&key, &content, now_secs())
}

#[tauri::command]
pub fn save_note(
    storage: tauri::State<'_, ActiveStorage>,
    id: String,
    content: String,
) -> Result<NoteDocument, String> {
    let key = note_key(&id);
    let current_group_id = if storage.object_exists(&key)? {
        let existing_bytes = storage.get_object(&key)?;
        let existing_content = String::from_utf8(existing_bytes)
            .map_err(|error| format!("解析现有笔记失败: {error}"))?;
        parse_group_id_from_content(&existing_content)
    } else {
        None
    };
    let content = with_group_meta(&content, current_group_id.as_deref());

    storage.put_object(&key, content.as_bytes())?;

    build_note_document(&key, &content, now_secs())
}

#[tauri::command]
pub fn delete_note(
    storage: tauri::State<'_, ActiveStorage>,
    id: String,
) -> Result<(), String> {
    let key = note_key(&id);
    storage.delete_object(&key)
}

// ── group commands ─────────────────────────────────────────

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteGroup {
    id: String,
    name: String,
    color: Option<String>,
    created_at: u64,
    updated_at: u64,
}

#[tauri::command]
pub fn list_note_groups(
    storage: tauri::State<'_, ActiveStorage>,
) -> Result<Vec<NoteGroup>, String> {
    let objects = storage.list_objects("groups/")?;
    let mut groups = Vec::new();

    for obj in &objects {
        if !obj.key.ends_with(".json") {
            continue;
        }

        let content_bytes = storage.get_object(&obj.key)?;
        let group: NoteGroup = serde_json::from_slice(&content_bytes)
            .map_err(|error| format!("解析分组数据失败: {error}"))?;

        groups.push(group);
    }

    groups.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.created_at.cmp(&b.created_at)));

    Ok(groups)
}

#[tauri::command]
pub fn create_note_group(
    storage: tauri::State<'_, ActiveStorage>,
    name: String,
    color: Option<String>,
) -> Result<NoteGroup, String> {
    let group_id = format!("group-{}", uuid::Uuid::new_v4().simple());
    let now = now_secs();

    let group = NoteGroup {
        id: group_id.clone(),
        name: name.trim().to_string(),
        color,
        created_at: now,
        updated_at: now,
    };

    let key = group_key(&group_id);
    let content = serde_json::to_string_pretty(&group)
        .map_err(|error| format!("序列化分组数据失败: {error}"))?;

    storage.put_object(&key, content.as_bytes())?;

    Ok(group)
}

#[tauri::command]
pub fn update_note_group(
    storage: tauri::State<'_, ActiveStorage>,
    id: String,
    name: String,
    color: Option<String>,
) -> Result<NoteGroup, String> {
    let key = group_key(&id);

    if !storage.object_exists(&key)? {
        return Err("分组不存在".to_string());
    }

    let content_bytes = storage.get_object(&key)?;
    let mut group: NoteGroup = serde_json::from_slice(&content_bytes)
        .map_err(|error| format!("解析分组数据失败: {error}"))?;

    group.name = name.trim().to_string();
    group.color = color;
    group.updated_at = now_secs();

    let content = serde_json::to_string_pretty(&group)
        .map_err(|error| format!("序列化分组数据失败: {error}"))?;

    storage.put_object(&key, content.as_bytes())?;

    Ok(group)
}

#[tauri::command]
pub fn delete_note_group(
    storage: tauri::State<'_, ActiveStorage>,
    id: String,
) -> Result<(), String> {
    let key = group_key(&id);
    storage.delete_object(&key)
}

#[tauri::command]
pub fn move_note_to_group(
    storage: tauri::State<'_, ActiveStorage>,
    note_id: String,
    group_id: Option<String>,
) -> Result<NoteDocument, String> {
    let key = note_key(&note_id);

    if !storage.object_exists(&key)? {
        return Err("笔记不存在".to_string());
    }

    let content_bytes = storage.get_object(&key)?;
    let content =
        String::from_utf8(content_bytes).map_err(|error| format!("解析笔记内容失败: {error}"))?;

    let new_content = with_group_meta(&content, group_id.as_deref());
    storage.put_object(&key, new_content.as_bytes())?;

    build_note_document(&key, &new_content, now_secs())
}

// ── image command ──────────────────────────────────────────

#[tauri::command]
pub fn import_image(storage: tauri::State<'_, ActiveStorage>) -> Result<Option<String>, String> {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"])
        .pick_file()
    {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        let unique_name = format!("{}.{}", uuid::Uuid::new_v4().simple(), extension);
        let key = format!("images/{unique_name}");

        let data = std::fs::read(&path).map_err(|error| format!("读取图片文件失败: {error}"))?;
        storage.put_object(&key, &data)?;

        Ok(Some(storage.resolve_url(&key)))
    } else {
        Ok(None)
    }
}
