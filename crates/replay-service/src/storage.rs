//! replay-service 对象存储抽象 (per RGS-DTL-038 §3 DEC-038-03 推荐 A)
//!
//! ## 域职责
//! 元数据存 PostgreSQL, 回放数据存对象存储 (cluster-ops S3-兼容).
//! 本地用 `LocalFsBackend` 模拟, 生产可替换为 `S3Backend` (TODO W36+).
//!
//! ## 设计原则
//! - `StorageBackend` trait: put / get / delete / list / exists
//! - `LocalFsBackend`: 写到本地目录, mock cluster-ops 对象存储
//! - `InMemoryBackend`: 内存 map, 测用
//! - 跨平台: Windows + WSL 都能跑 (用 `std::path::Path`)
//!
//! ## 替换模式
//! 生产部署时, 把 `LocalFsBackend` 替换为 S3 / MinIO / GCS 客户端
//! (仅 trait 实现不同, 业务层零改动).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use bytes::Bytes;

use crate::Result;

// ============================================================================
// StorageBackend trait
// ============================================================================

/// 对象存储抽象 (per DEC-038-03 cluster-ops 对象存储)
///
/// 业务语义:
/// - key: 业务主键 (e.g. "replays/2026/08/{uuid}.dat")
/// - value: 二进制 bytes (回放数据: move log / board snapshots)
/// - put: 幂等 (同 key 覆盖)
/// - get / exists: key 不存在返 None / false
/// - list: 按 prefix 过滤 (返回所有匹配 key + 0 size, 用于 cron 清理)
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 写入对象 (幂等, 覆盖)
    async fn put(&self, key: &str, data: Bytes) -> Result<()>;

    /// 读取对象 (None = 不存在)
    async fn get(&self, key: &str) -> Result<Option<Bytes>>;

    /// 删除对象 (true = 删除成功, false = key 不存在)
    async fn delete(&self, key: &str) -> Result<bool>;

    /// 检查 key 是否存在
    async fn exists(&self, key: &str) -> Result<bool>;

    /// 按 prefix 列出 keys (用于 cron 清理过期对象)
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// 获取对象大小 (None = 不存在, 用于快速元数据查询)
    async fn size(&self, key: &str) -> Result<Option<u64>>;
}

// ============================================================================
// LocalFsBackend (生产 + 集成测试用, mock cluster-ops 对象存储)
// ============================================================================

/// 本地文件系统后端 (mock cluster-ops 对象存储)
///
/// 业务语义:
/// - root_dir: 根目录 (e.g. `/var/lib/rgs/replays/` 或测试用 `tempdir`)
/// - key "replays/2026/08/foo.dat" -> 文件 `<root_dir>/replays/2026/08/foo.dat`
/// - 自动创建父目录 (mkdir -p)
pub struct LocalFsBackend {
    root_dir: PathBuf,
}

impl LocalFsBackend {
    /// 工厂: 新建 LocalFsBackend (不创建目录, 第一次 put 时才创建)
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    /// 解析 key -> 物理路径
    fn resolve(&self, key: &str) -> Result<PathBuf> {
        // 防御: 禁止 .. (路径穿越)
        if key.contains("..") {
            return Err(crate::Error::Validation(format!(
                "storage key contains '..': {}",
                key
            )));
        }
        Ok(self.root_dir.join(key))
    }
}

#[async_trait]
impl StorageBackend for LocalFsBackend {
    async fn put(&self, key: &str, data: Bytes) -> Result<()> {
        let path = self.resolve(key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                crate::Error::Storage(format!(
                    "mkdir failed for {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
        tokio::fs::write(&path, &data)
            .await
            .map_err(|e| crate::Error::Storage(format!("write failed for {}: {}", key, e)))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let path = self.resolve(key)?;
        match tokio::fs::read(&path).await {
            Ok(data) => Ok(Some(Bytes::from(data))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(crate::Error::Storage(format!(
                "read failed for {}: {}",
                key, e
            ))),
        }
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let path = self.resolve(key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(crate::Error::Storage(format!(
                "delete failed for {}: {}",
                key, e
            ))),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.resolve(key)?;
        Ok(Path::new(&path).is_file())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let base = self.resolve(prefix)?;
        if !base.exists() {
            return Ok(Vec::new());
        }
        let mut keys = Vec::new();
        let mut stack = vec![base];
        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| {
                crate::Error::Storage(format!(
                    "read_dir failed for {}: {}",
                    dir.display(),
                    e
                ))
            })?;
            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                crate::Error::Storage(format!("next_entry failed: {}", e))
            })? {
                let entry_path = entry.path();
                let file_type = entry.file_type().await.map_err(|e| {
                    crate::Error::Storage(format!("file_type failed: {}", e))
                })?;
                if file_type.is_dir() {
                    stack.push(entry_path);
                } else if file_type.is_file() {
                    // 计算相对 key (strip root_dir)
                    if let Ok(rel) = entry_path.strip_prefix(&self.root_dir) {
                        let key = rel.to_string_lossy().replace('\\', "/");
                        keys.push(key);
                    }
                }
            }
        }
        // 按 key 排序, 便于测试断言
        keys.sort();
        Ok(keys)
    }

    async fn size(&self, key: &str) -> Result<Option<u64>> {
        let path = self.resolve(key)?;
        match tokio::fs::metadata(&path).await {
            Ok(m) => Ok(Some(m.len())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(crate::Error::Storage(format!(
                "metadata failed for {}: {}",
                key, e
            ))),
        }
    }
}

// ============================================================================
// InMemoryBackend (单元测试用)
// ============================================================================

/// 内存后端 (单元测试用, 不跨进程)
pub struct InMemoryBackend {
    inner: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for InMemoryBackend {
    async fn put(&self, key: &str, data: Bytes) -> Result<()> {
        if key.contains("..") {
            return Err(crate::Error::Validation(format!(
                "storage key contains '..': {}",
                key
            )));
        }
        self.inner
            .lock()
            .unwrap()
            .insert(key.to_string(), data.to_vec());
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .get(key)
            .map(|v| Bytes::from(v.clone())))
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.inner.lock().unwrap().contains_key(key))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys: Vec<String> = self
            .inner
            .lock()
            .unwrap()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        Ok(keys)
    }

    async fn size(&self, key: &str) -> Result<Option<u64>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .get(key)
            .map(|v| v.len() as u64))
    }
}

// ============================================================================
// helper: 生成对象 key (per 业务规则)
// ============================================================================

/// 生成对象存储 key (per ReplayMeta)
/// 格式: `replays/{YYYY}/{MM}/rp-{replay_id}.dat`
/// 注: created_at 决定目录分桶, 便于运维清理 (按月归档)
pub fn build_object_key(replay_id: &uuid::Uuid, created_at: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "replays/{:04}/{:02}/rp-{}.dat",
        created_at.format("%Y").to_string().parse::<u32>().unwrap_or(1970),
        created_at.format("%m").to_string().parse::<u32>().unwrap_or(1),
        replay_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn build_object_key_format() {
        let id = uuid::Uuid::nil();
        let ts = chrono::Utc.with_ymd_and_hms(2026, 8, 29, 10, 0, 0).unwrap();
        let key = build_object_key(&id, ts);
        assert!(key.starts_with("replays/2026/08/rp-"));
        assert!(key.ends_with(".dat"));
    }

    #[test]
    fn local_fs_resolve_rejects_path_traversal() {
        let backend = LocalFsBackend::new("/tmp/replays");
        assert!(backend.resolve("../etc/passwd").is_err());
        assert!(backend.resolve("foo/../bar").is_err());
        assert!(backend.resolve("safe/path").is_ok());
    }

    #[tokio::test]
    async fn in_memory_put_get_roundtrip() {
        let backend = InMemoryBackend::new();
        backend
            .put("test/key1", Bytes::from_static(b"hello world"))
            .await
            .unwrap();
        let data = backend.get("test/key1").await.unwrap();
        assert_eq!(data, Some(Bytes::from_static(b"hello world")));
    }

    #[tokio::test]
    async fn in_memory_get_missing_key() {
        let backend = InMemoryBackend::new();
        let data = backend.get("nope").await.unwrap();
        assert_eq!(data, None);
    }

    #[tokio::test]
    async fn in_memory_delete_roundtrip() {
        let backend = InMemoryBackend::new();
        backend
            .put("k1", Bytes::from_static(b"data"))
            .await
            .unwrap();
        assert!(backend.delete("k1").await.unwrap());
        assert!(!backend.delete("k1").await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_list_with_prefix() {
        let backend = InMemoryBackend::new();
        backend
            .put("replays/2026/a", Bytes::from_static(b"a"))
            .await
            .unwrap();
        backend
            .put("replays/2026/b", Bytes::from_static(b"b"))
            .await
            .unwrap();
        backend
            .put("other/x", Bytes::from_static(b"x"))
            .await
            .unwrap();
        let keys = backend.list("replays/").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"replays/2026/a".to_string()));
        assert!(keys.contains(&"replays/2026/b".to_string()));
    }

    #[tokio::test]
    async fn in_memory_size() {
        let backend = InMemoryBackend::new();
        backend
            .put("k", Bytes::from_static(b"12345"))
            .await
            .unwrap();
        assert_eq!(backend.size("k").await.unwrap(), Some(5));
        assert_eq!(backend.size("nope").await.unwrap(), None);
    }

    #[tokio::test]
    async fn in_memory_put_rejects_path_traversal() {
        let backend = InMemoryBackend::new();
        assert!(backend
            .put("../etc/passwd", Bytes::from_static(b"x"))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn local_fs_put_get_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = LocalFsBackend::new(tmp.path());
        backend
            .put("replays/2026/08/rp-1.dat", Bytes::from_static(b"payload"))
            .await
            .unwrap();
        let data = backend.get("replays/2026/08/rp-1.dat").await.unwrap();
        assert_eq!(data, Some(Bytes::from_static(b"payload")));
        assert_eq!(
            backend
                .size("replays/2026/08/rp-1.dat")
                .await
                .unwrap(),
            Some(7)
        );
        assert!(backend.exists("replays/2026/08/rp-1.dat").await.unwrap());
    }

    #[tokio::test]
    async fn local_fs_delete_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = LocalFsBackend::new(tmp.path());
        backend
            .put("k", Bytes::from_static(b"v"))
            .await
            .unwrap();
        assert!(backend.delete("k").await.unwrap());
        assert!(!backend.delete("k").await.unwrap());
        assert!(!backend.exists("k").await.unwrap());
    }

    #[tokio::test]
    async fn local_fs_list_with_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        let backend = LocalFsBackend::new(tmp.path());
        backend
            .put("replays/2026/a.dat", Bytes::from_static(b"a"))
            .await
            .unwrap();
        backend
            .put("replays/2026/b.dat", Bytes::from_static(b"b"))
            .await
            .unwrap();
        backend
            .put("other/x.dat", Bytes::from_static(b"x"))
            .await
            .unwrap();
        let keys = backend.list("replays/").await.unwrap();
        assert_eq!(keys.len(), 2);
    }
}
