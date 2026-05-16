use crate::settings::{AppSettings, MinioConfig};
use crate::storage::local::LocalStorage;
use crate::storage::minio::MinioStorage;
use crate::storage::StorageBackend;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub uploaded: u32,
    pub downloaded: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

#[tauri::command]
pub fn sync_notes(app: AppHandle) -> Result<SyncResult, String> {
    let app_data_dir: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?;

    let settings_path = app_data_dir.join("settings.json");
    let settings = AppSettings::load_or_default(&settings_path);

    let config: MinioConfig = settings
        .minio
        .ok_or_else(|| "尚未配置 MinIO 连接信息，请先在设置中填写 MinIO 配置。".to_string())?;

    let local = LocalStorage::new(&app_data_dir);
    let remote = MinioStorage::new(&config)?;

    let mut result = SyncResult::default();
    let prefixes = ["notes/", "groups/"];

    for prefix in &prefixes {
        sync_prefix(&local, &remote, prefix, &mut result);
    }

    Ok(result)
}

fn sync_prefix(
    local: &LocalStorage,
    remote: &MinioStorage,
    prefix: &str,
    result: &mut SyncResult,
) {
    let local_objects = match local.list_objects(prefix) {
        Ok(objs) => objs,
        Err(e) => {
            result.errors.push(format!("读取本地 {prefix} 失败: {e}"));
            return;
        }
    };

    let remote_objects = match remote.list_objects(prefix) {
        Ok(objs) => objs,
        Err(e) => {
            result.errors.push(format!("读取远程 {prefix} 失败: {e}"));
            return;
        }
    };

    let mut local_map: std::collections::BTreeMap<&str, u64> = std::collections::BTreeMap::new();
    for obj in &local_objects {
        local_map.insert(&obj.key, obj.updated_at);
    }

    let mut remote_map: std::collections::BTreeMap<&str, u64> = std::collections::BTreeMap::new();
    for obj in &remote_objects {
        remote_map.insert(&obj.key, obj.updated_at);
    }

    let mut all_keys: Vec<&str> = local_map
        .keys()
        .chain(remote_map.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    all_keys.sort();

    for key in &all_keys {
        match (local_map.get(key), remote_map.get(key)) {
            (Some(&local_ts), Some(&remote_ts)) => {
                if local_ts > remote_ts {
                    match push_local_to_remote(local, remote, key) {
                        Ok(()) => result.uploaded += 1,
                        Err(e) => result.errors.push(format!("上传 {key}: {e}")),
                    }
                } else if remote_ts > local_ts {
                    match pull_remote_to_local(remote, local, key) {
                        Ok(()) => result.downloaded += 1,
                        Err(e) => result.errors.push(format!("下载 {key}: {e}")),
                    }
                } else {
                    result.skipped += 1;
                }
            }
            (Some(_), None) => {
                match push_local_to_remote(local, remote, key) {
                    Ok(()) => result.uploaded += 1,
                    Err(e) => result.errors.push(format!("上传 {key}: {e}")),
                }
            }
            (None, Some(_)) => {
                match pull_remote_to_local(remote, local, key) {
                    Ok(()) => result.downloaded += 1,
                    Err(e) => result.errors.push(format!("下载 {key}: {e}")),
                }
            }
            (None, None) => unreachable!(),
        }
    }
}

fn push_local_to_remote(
    local: &LocalStorage,
    remote: &MinioStorage,
    key: &str,
) -> Result<(), String> {
    let data = local.get_object(key)?;
    remote.put_object(key, &data)
}

fn pull_remote_to_local(
    remote: &MinioStorage,
    local: &LocalStorage,
    key: &str,
) -> Result<(), String> {
    let data = remote.get_object(key)?;
    local.put_object(key, &data)
}
