use super::{ObjectInfo, StorageBackend};
use crate::settings::MinioConfig;
use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;

pub struct MinioStorage {
    bucket: Box<Bucket>,
    endpoint: String,
    bucket_name: String,
}

impl MinioStorage {
    pub fn new(config: &MinioConfig) -> Result<Self, String> {
        let bucket = Bucket::new(
            &config.bucket,
            Region::Custom {
                region: "us-east-1".to_owned(),
                endpoint: config.endpoint.clone(),
            },
            Credentials::new(
                Some(&config.access_key),
                Some(&config.secret_key),
                None,
                None,
                None,
            )
            .map_err(|e| format!("创建 MinIO Credentials 失败: {e}"))?,
        )
        .map_err(|e| format!("创建 MinIO Bucket 失败: {e}"))?
        .with_path_style();

        Ok(Self {
            bucket,
            endpoint: config.endpoint.trim_end_matches('/').to_string(),
            bucket_name: config.bucket.clone(),
        })
    }

    fn parse_last_modified(s: &str) -> u64 {
        // Parse ISO 8601 date string like "2024-01-01T00:00:00.000Z"
        if let Some(cleaned) = s
            .strip_suffix('Z')
            .or_else(|| s.strip_suffix("+0000"))
        {
            if cleaned.len() >= 19 {
                let date_part = &cleaned[..19]; // "2024-01-01T00:00:00"
                if let Some(dt) = chrono_lite::parse(date_part) {
                    return dt;
                }
            }
        }
        // Fallback: try to parse as simple timestamp
        s.parse::<u64>().unwrap_or(0)
    }
}

/// Minimal date parsing for ISO 8601 "YYYY-MM-DDTHH:MM:SS" format
mod chrono_lite {
    pub fn parse(s: &str) -> Option<u64> {
        // s = "2024-01-01T00:00:00"
        let parts: Vec<&str> = s.split('T').collect();
        if parts.len() != 2 {
            return None;
        }
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        let time_parts: Vec<&str> = parts[1].split(':').collect();
        if date_parts.len() != 3 || time_parts.len() != 3 {
            return None;
        }
        let year: i32 = date_parts[0].parse().ok()?;
        let month: u32 = date_parts[1].parse().ok()?;
        let day: u32 = date_parts[2].parse().ok()?;
        let hour: u32 = time_parts[0].parse().ok()?;
        let min: u32 = time_parts[1].parse().ok()?;
        let sec: u32 = time_parts[2].parse().ok()?;

        // Algorithm from https://en.wikipedia.org/wiki/Unix_time
        let mut days = 0i64;
        for y in 1970..year {
            days += if is_leap(y) { 366 } else { 365 };
        }
        let month_days = if is_leap(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        for m in 0..(month as usize).saturating_sub(1) {
            days += month_days[m] as i64;
        }
        days += (day as i64) - 1;
        let secs = days * 86400 + hour as i64 * 3600 + min as i64 * 60 + sec as i64;
        Some(secs as u64)
    }

    fn is_leap(y: i32) -> bool {
        (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
    }
}

impl StorageBackend for MinioStorage {
    fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectInfo>, String> {
        let results = self
            .bucket
            .list(prefix.to_string(), Some("/".to_string()))
            .map_err(|e| format!("MinIO list 失败: {e}"))?;

        let mut objects = Vec::new();
        for result in results {
            for item in result.contents {
                objects.push(ObjectInfo {
                    key: item.key,
                    updated_at: Self::parse_last_modified(&item.last_modified),
                    size: item.size,
                });
            }

            // Also include common prefixes (subdirectories) contents
            if let Some(prefixes) = result.common_prefixes {
                for prefix_item in prefixes {
                    let sub_objects = self.list_objects(&prefix_item.prefix)?;
                    objects.extend(sub_objects);
                }
            }
        }

        Ok(objects)
    }

    fn get_object(&self, key: &str) -> Result<Vec<u8>, String> {
        let data = self
            .bucket
            .get_object(key)
            .map_err(|e| format!("MinIO get_object 失败: {e}"))?;

        if data.status_code() != 200 {
            return Err(format!("MinIO get_object 返回状态码: {}", data.status_code()));
        }

        Ok(data.to_vec())
    }

    fn put_object(&self, key: &str, data: &[u8]) -> Result<(), String> {
        let response = self
            .bucket
            .put_object(key, data)
            .map_err(|e| format!("MinIO put_object 失败: {e}"))?;

        if response.status_code() != 200 {
            return Err(format!(
                "MinIO put_object 返回状态码: {}",
                response.status_code()
            ));
        }

        Ok(())
    }

    fn delete_object(&self, key: &str) -> Result<(), String> {
        let response = self
            .bucket
            .delete_object(key)
            .map_err(|e| format!("MinIO delete_object 失败: {e}"))?;

        if response.status_code() != 204 && response.status_code() != 200 {
            return Err(format!(
                "MinIO delete_object 返回状态码: {}",
                response.status_code()
            ));
        }

        Ok(())
    }

    fn object_exists(&self, key: &str) -> Result<bool, String> {
        match self.bucket.head_object(key) {
            Ok((_, code)) => Ok(code == 200),
            Err(_) => Ok(false),
        }
    }

    fn get_object_info(&self, key: &str) -> Result<ObjectInfo, String> {
        let (head, code) = self
            .bucket
            .head_object(key)
            .map_err(|e| format!("MinIO head_object 失败: {e}"))?;

        if code != 200 {
            return Err("对象不存在".to_string());
        }

        Ok(ObjectInfo {
            key: key.to_string(),
            updated_at: head
                .last_modified
                .as_deref()
                .map(|s| Self::parse_last_modified(s))
                .unwrap_or(0),
            size: head.content_length.unwrap_or(0) as u64,
        })
    }

    fn resolve_url(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket_name, key)
    }
}
