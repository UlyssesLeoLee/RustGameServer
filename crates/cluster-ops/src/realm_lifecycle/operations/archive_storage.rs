//! In-Memory `ArchiveObjectStorage` 测试实现（per RGS-IMPL-PLAN-LCM-001 §3.7 "降级策略"）
//!
//! 2026-09-05 P1-3 拆分：从 archive.rs 抽出。
//! 模拟 N+2 副本：每次 `put_object` 生成 N+2 个副本记录
//!
//! **生产实现**：`S3ArchiveStorage`（写入 S3 + lifecycle policy to Glacier；
//! 由 WBS L4 #2074 的 SRE 接力任务实现）

use super::archive::{
    ArchiveObjectStorage, LcmResult, PutObjectResult, ReplicaInfo, StorageRedundancy,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

/// In-Memory `ArchiveObjectStorage`（仅供单元 / 集成测试用）
///
/// 模拟 N+2 副本：每次 `put_object` 生成 N+2 个副本记录
pub struct InMemoryArchiveStorage {
    inner: std::sync::Mutex<std::collections::HashMap<(String, String), Vec<ReplicaInfo>>>,
    /// 模拟副本数（默认 3 = N+2）
    pub simulated_replica_count: u8,
}

impl Default for InMemoryArchiveStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryArchiveStorage {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
            simulated_replica_count: StorageRedundancy::NPlus2.required_replica_count(),
        }
    }

    pub fn with_replica_count(mut self, n: u8) -> Self {
        self.simulated_replica_count = n;
        self
    }
}

#[async_trait]
impl ArchiveObjectStorage for InMemoryArchiveStorage {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        bytes: &[u8],
    ) -> LcmResult<PutObjectResult> {
        let replicas: Vec<ReplicaInfo> = (0..self.simulated_replica_count)
            .map(|i| ReplicaInfo {
                replica_id: format!("replica-{i}"),
                availability_zone: format!("az-{}", i % 3),
                storage_class: "STANDARD_IA".to_string(),
                size_bytes: bytes.len() as u64,
                created_at: Utc::now(),
            })
            .collect();
        let count = replicas.len() as u8;
        self.inner
            .lock()
            .expect("lock")
            .insert((bucket.to_string(), key.to_string()), replicas);
        Ok(PutObjectResult {
            bucket: bucket.to_string(),
            key: key.to_string(),
            size_bytes: bytes.len() as u64,
            etag: format!("etag-{}", Uuid::new_v4()),
            replica_count: count,
        })
    }

    async fn list_replicas(&self, bucket: &str, key: &str) -> LcmResult<Vec<ReplicaInfo>> {
        let g = self.inner.lock().expect("lock");
        Ok(g.get(&(bucket.to_string(), key.to_string()))
            .cloned()
            .unwrap_or_default())
    }
}

