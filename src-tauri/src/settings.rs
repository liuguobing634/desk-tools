use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::storage::local::LocalStorage;
use crate::storage::minio::MinioStorage;
use crate::storage::ActiveStorage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StorageMode {
    Local,
    Minio,
}

impl Default for StorageMode {
    fn default() -> Self {
        StorageMode::Local
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinioConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub storage_mode: StorageMode,
    pub minio: Option<MinioConfig>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            storage_mode: StorageMode::Local,
            minio: None,
        }
    }
}

impl AppSettings {
    pub fn load_or_default(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化设置失败: {e}"))?;
        std::fs::write(path, content)
            .map_err(|e| format!("保存设置文件失败: {e}"))
    }
}

/// Create an ActiveStorage instance based on the given settings.
/// `base_path` is the local app data directory (used for Local storage mode).
pub fn create_storage(settings: &AppSettings, base_path: &Path) -> Result<ActiveStorage, String> {
    match settings.storage_mode {
        StorageMode::Local => Ok(ActiveStorage::Local(LocalStorage::new(base_path))),
        StorageMode::Minio => {
            let config = settings
                .minio
                .as_ref()
                .ok_or_else(|| "MinIO 存储模式已启用但缺少配置信息".to_string())?;
            let storage = MinioStorage::new(config)?;
            Ok(ActiveStorage::Minio(storage))
        }
    }
}
