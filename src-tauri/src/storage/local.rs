use super::{ObjectInfo, StorageBackend};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn full_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    fn ensure_parent(&self, key: &str) -> Result<(), String> {
        if let Some(parent) = self.full_path(key).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {e}"))?;
        }
        Ok(())
    }

    fn file_updated_at(path: &Path) -> Result<u64, String> {
        let metadata =
            fs::metadata(path).map_err(|error| format!("读取元数据失败: {error}"))?;
        let modified = metadata
            .modified()
            .map_err(|error| format!("读取修改时间失败: {error}"))?;
        let duration = modified
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("解析修改时间失败: {error}"))?;
        Ok(duration.as_secs())
    }
}

impl StorageBackend for LocalStorage {
    fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectInfo>, String> {
        let dir = self.full_path(prefix);
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("创建目录失败: {e}"))?;
            return Ok(Vec::new());
        }

        let mut objects = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {e}"))? {
            let entry = entry.map_err(|e| format!("读取目录项失败: {e}"))?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let key = if prefix.is_empty() {
                file_name.to_string()
            } else {
                format!("{}{}", prefix, file_name)
            };
            let updated_at = Self::file_updated_at(&path).unwrap_or(0);
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            objects.push(ObjectInfo {
                key,
                updated_at,
                size,
            });
        }
        Ok(objects)
    }

    fn get_object(&self, key: &str) -> Result<Vec<u8>, String> {
        let path = self.full_path(key);
        fs::read(&path).map_err(|e| format!("读取对象失败: {e}"))
    }

    fn put_object(&self, key: &str, data: &[u8]) -> Result<(), String> {
        self.ensure_parent(key)?;
        let path = self.full_path(key);
        fs::write(&path, data).map_err(|e| format!("写入对象失败: {e}"))
    }

    fn delete_object(&self, key: &str) -> Result<(), String> {
        let path = self.full_path(key);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("删除对象失败: {e}"))?;
        }
        Ok(())
    }

    fn object_exists(&self, key: &str) -> Result<bool, String> {
        Ok(self.full_path(key).exists())
    }

    fn get_object_info(&self, key: &str) -> Result<ObjectInfo, String> {
        let path = self.full_path(key);
        if !path.exists() {
            return Err("对象不存在".to_string());
        }
        let updated_at = Self::file_updated_at(&path)?;
        let size = path
            .metadata()
            .map(|m| m.len())
            .map_err(|e| format!("读取文件大小失败: {e}"))?;
        Ok(ObjectInfo {
            key: key.to_string(),
            updated_at,
            size,
        })
    }

    fn resolve_url(&self, key: &str) -> String {
        self.full_path(key)
            .to_str()
            .unwrap_or("")
            .to_string()
    }
}
