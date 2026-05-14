pub mod local;
pub mod minio;

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

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub updated_at: u64,
    pub size: u64,
}

/// Abstract storage backend for notes, groups, and images.
pub trait StorageBackend: Send + Sync + 'static {
    fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectInfo>, String>;
    fn get_object(&self, key: &str) -> Result<Vec<u8>, String>;
    fn put_object(&self, key: &str, data: &[u8]) -> Result<(), String>;
    fn delete_object(&self, key: &str) -> Result<(), String>;
    fn object_exists(&self, key: &str) -> Result<bool, String>;
    fn get_object_info(&self, key: &str) -> Result<ObjectInfo, String>;
    fn resolve_url(&self, key: &str) -> String;
}

/// Enum that delegates to the active storage backend.
pub enum ActiveStorage {
    Local(local::LocalStorage),
    Minio(minio::MinioStorage),
}

impl ActiveStorage {
    pub fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectInfo>, String> {
        match self {
            ActiveStorage::Local(s) => s.list_objects(prefix),
            ActiveStorage::Minio(s) => s.list_objects(prefix),
        }
    }

    pub fn get_object(&self, key: &str) -> Result<Vec<u8>, String> {
        match self {
            ActiveStorage::Local(s) => s.get_object(key),
            ActiveStorage::Minio(s) => s.get_object(key),
        }
    }

    pub fn put_object(&self, key: &str, data: &[u8]) -> Result<(), String> {
        match self {
            ActiveStorage::Local(s) => s.put_object(key, data),
            ActiveStorage::Minio(s) => s.put_object(key, data),
        }
    }

    pub fn delete_object(&self, key: &str) -> Result<(), String> {
        match self {
            ActiveStorage::Local(s) => s.delete_object(key),
            ActiveStorage::Minio(s) => s.delete_object(key),
        }
    }

    pub fn object_exists(&self, key: &str) -> Result<bool, String> {
        match self {
            ActiveStorage::Local(s) => s.object_exists(key),
            ActiveStorage::Minio(s) => s.object_exists(key),
        }
    }

    pub fn get_object_info(&self, key: &str) -> Result<ObjectInfo, String> {
        match self {
            ActiveStorage::Local(s) => s.get_object_info(key),
            ActiveStorage::Minio(s) => s.get_object_info(key),
        }
    }

    pub fn resolve_url(&self, key: &str) -> String {
        match self {
            ActiveStorage::Local(s) => s.resolve_url(key),
            ActiveStorage::Minio(s) => s.resolve_url(key),
        }
    }
}
