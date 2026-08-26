//! cluster-ops · realm_lifecycle · Saga 幂等性（per M-2067.4）
//!
//! 硬约束（per RGS-SPEC-DTL-042 §5 幂等一致性）：
//! - `request_id` 唯一
//! - `realm_lifecycle_run` 表 `(request_id, operator_id)` 唯一索引
//! - Saga 步骤重试时返回 `AlreadyApplied` 幂等结果
//!
//! 设计：
//! - `IdempotencyKey`：复合键 (request_id, operator_id)
//! - `IdempotencyRecord`：阶段 / Saga ID / 结果状态 / 时间戳
//! - `IdempotencyStore` trait + InMemoryIdempotencyStore 实现
//! - SagaOrchestrator 在 dispatch 入口查询；命中 → 返 `Error::AlreadyApplied`
//! - 真实 DB 持久化属 WF-1-2068 (migrations/0020_lcm_tables.sql)

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::realm_lifecycle::error::Result;
use crate::realm_lifecycle::saga::orchestrator::LcmPhase;

/// 幂等键：(request_id, operator_id) 复合（per RGS-SPEC-DTL-042 §5）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdempotencyKey {
    pub request_id: Uuid,
    pub operator_id: Uuid,
}

impl IdempotencyKey {
    pub fn new(request_id: Uuid, operator_id: Uuid) -> Self {
        Self {
            request_id,
            operator_id,
        }
    }

    /// 规范字符串形式（per economy saga.idempotency_key）
    pub fn canonical(&self) -> String {
        format!("lcm:{}:{}", self.request_id, self.operator_id)
    }
}

/// 幂等记录
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    /// 复合键
    pub key: IdempotencyKey,
    /// 阶段
    pub phase: LcmPhase,
    /// Saga ID
    pub saga_id: Uuid,
    /// 结果状态（"completed" / "failed" / "in_progress"）
    pub outcome: String,
    /// 记录时间
    pub recorded_at: DateTime<Utc>,
}

impl IdempotencyRecord {
    pub fn new(key: IdempotencyKey, phase: LcmPhase, saga_id: Uuid, outcome: impl Into<String>) -> Self {
        Self {
            key,
            phase,
            saga_id,
            outcome: outcome.into(),
            recorded_at: Utc::now(),
        }
    }
}

/// 幂等性存储 trait
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// 查询 (request_id, operator_id) 命中记录
    async fn lookup(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>>;
    /// 写记录（不重复；命中则覆盖 outcome）
    async fn record(&self, record: IdempotencyRecord) -> Result<()>;
}

/// 内存版 IdempotencyStore（per WF-1-2067 范围；DB 版本属 WF-1-2068）
pub struct InMemoryIdempotencyStore {
    inner: Mutex<HashMap<IdempotencyKey, IdempotencyRecord>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// 当前记录数（测试用）
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// 是否空（测试用）
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn lookup(&self, key: &IdempotencyKey) -> Result<Option<IdempotencyRecord>> {
        Ok(self.inner.lock().unwrap().get(key).cloned())
    }

    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        self.inner
            .lock()
            .unwrap()
            .insert(record.key, record);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_canonical_format() {
        let req = Uuid::nil();
        let op = Uuid::nil();
        let key = IdempotencyKey::new(req, op);
        let canonical = key.canonical();
        assert!(canonical.starts_with("lcm:"));
        assert!(canonical.contains(&req.to_string()));
    }

    #[tokio::test]
    async fn in_memory_store_lookup_miss() {
        let store = InMemoryIdempotencyStore::new();
        let key = IdempotencyKey::new(Uuid::new_v4(), Uuid::new_v4());
        let result = store.lookup(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn in_memory_store_record_then_lookup_hit() {
        let store = InMemoryIdempotencyStore::new();
        let req = Uuid::new_v4();
        let op = Uuid::new_v4();
        let key = IdempotencyKey::new(req, op);
        let saga_id = Uuid::new_v4();
        let record = IdempotencyRecord::new(key, LcmPhase::NewRealm, saga_id, "completed");
        store.record(record.clone()).await.unwrap();
        let result = store.lookup(&key).await.unwrap().unwrap();
        assert_eq!(result.key, key);
        assert_eq!(result.phase, LcmPhase::NewRealm);
        assert_eq!(result.saga_id, saga_id);
        assert_eq!(result.outcome, "completed");
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn in_memory_store_overwrite_outcome() {
        let store = InMemoryIdempotencyStore::new();
        let key = IdempotencyKey::new(Uuid::new_v4(), Uuid::new_v4());
        let saga_id = Uuid::new_v4();
        store
            .record(IdempotencyRecord::new(
                key,
                LcmPhase::NewRealm,
                saga_id,
                "in_progress",
            ))
            .await
            .unwrap();
        store
            .record(IdempotencyRecord::new(
                key,
                LcmPhase::NewRealm,
                saga_id,
                "completed",
            ))
            .await
            .unwrap();
        let result = store.lookup(&key).await.unwrap().unwrap();
        assert_eq!(result.outcome, "completed");
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn in_memory_store_distinct_keys() {
        let store = InMemoryIdempotencyStore::new();
        let k1 = IdempotencyKey::new(Uuid::new_v4(), Uuid::new_v4());
        let k2 = IdempotencyKey::new(Uuid::new_v4(), Uuid::new_v4());
        store
            .record(IdempotencyRecord::new(k1, LcmPhase::NewRealm, Uuid::new_v4(), "x"))
            .await
            .unwrap();
        store
            .record(IdempotencyRecord::new(k2, LcmPhase::Scale, Uuid::new_v4(), "y"))
            .await
            .unwrap();
        assert_eq!(store.len(), 2);
        assert!(store.lookup(&k1).await.unwrap().is_some());
        assert!(store.lookup(&k2).await.unwrap().is_some());
    }
}
