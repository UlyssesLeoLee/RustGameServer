//! Outbox 模式（per RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005 事务性消息）
//!
//! 54.11 实化：OutboxEntry + OutboxRepository trait + Pg/InMemory impl
//!
//! 设计原则（per transactional outbox pattern）：
//! - 业务写 DB + 写 outbox 表必须在同一事务
//! - 单独 relay 进程轮询 outbox 表，发布到 NATS
//! - 至少一次投递（at-least-once）：业务已 commit 但 NATS 未发 → relay 重试
//! - 幂等性：consumer 端靠 envelope.command_id 去重
//!
//! 状态机：Pending → Sent / Failed
//!
//! 注：shared-platform 提供通用 OutboxRepository trait；
//!     各域 crate 需在自己 migrations 里建 outbox 表（per 域 schema）

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

/// Outbox 错误
#[derive(Debug, Error)]
pub enum OutboxError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Outbox Result 类型
pub type Result<T> = std::result::Result<T, OutboxError>;

/// Outbox 状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    /// 待发送
    Pending,
    /// 已发送
    Sent,
    /// 失败（重试 N 次后放弃）
    Failed,
}

/// Outbox 条目（per DTL-100 §5.3 事务性消息）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutboxEntry {
    /// Outbox ID
    pub id: Uuid,
    /// 目标 subject
    pub subject: String,
    /// payload (JSON)
    pub payload: String,
    /// 业务 command_id（consumer 幂等性 key）
    pub command_id: Uuid,
    /// 业务 saga_id
    pub saga_id: Option<Uuid>,
    /// 状态
    pub status: OutboxStatus,
    /// 重试次数
    pub retry_count: u32,
    /// 上次错误（Failed 时记录）
    pub last_error: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 发送时间
    pub sent_at: Option<DateTime<Utc>>,
}

impl OutboxEntry {
    /// 工厂：新建 Pending 条目
    pub fn new(subject: String, payload: String, command_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject,
            payload,
            command_id,
            saga_id: None,
            status: OutboxStatus::Pending,
            retry_count: 0,
            last_error: None,
            created_at: Utc::now(),
            sent_at: None,
        }
    }

    /// 链式 setter
    pub fn with_saga(mut self, saga_id: Uuid) -> Self {
        self.saga_id = Some(saga_id);
        self
    }

    /// 标记已发送
    pub fn mark_sent(&mut self) {
        self.status = OutboxStatus::Sent;
        self.sent_at = Some(Utc::now());
        self.last_error = None;
    }

    /// 标记失败（重试 +1）
    pub fn mark_failed(&mut self, error: String) {
        self.retry_count += 1;
        self.last_error = Some(error);
    }

    /// 标记最终失败（重试耗尽）
    pub fn mark_giveup(&mut self) {
        self.status = OutboxStatus::Failed;
    }
}

/// Outbox Repository trait
#[async_trait]
pub trait OutboxRepository: Send + Sync {
    /// 在事务内追加（per DTL-100 §5.3 同事务要求）
    /// 注：调用方需保证 `tx` 跟业务写 DB 在同一事务
    async fn append(&self, entry: &OutboxEntry) -> Result<()>;
    /// 列出待发送条目（relay 调用）
    async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>>;
    /// 标记已发送
    async fn mark_sent(&self, id: Uuid) -> Result<()>;
    /// 标记失败（retry_count+1）
    async fn mark_failed(&self, id: Uuid, error: String) -> Result<()>;
    /// 标记最终失败
    async fn mark_giveup(&self, id: Uuid) -> Result<()>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgOutboxRepository {
    pool: PgPool,
}

impl PgOutboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn status_to_str(s: OutboxStatus) -> &'static str {
    match s {
        OutboxStatus::Pending => "pending",
        OutboxStatus::Sent => "sent",
        OutboxStatus::Failed => "failed",
    }
}

fn parse_status(s: &str) -> OutboxStatus {
    match s {
        "sent" => OutboxStatus::Sent,
        "failed" => OutboxStatus::Failed,
        _ => OutboxStatus::Pending,
    }
}

fn row_to_entry(row: sqlx::postgres::PgRow) -> OutboxEntry {
    let status_str: String = row.get("status");
    OutboxEntry {
        id: row.get("id"),
        subject: row.get("subject"),
        payload: row.get("payload"),
        command_id: row.get("command_id"),
        saga_id: row.get("saga_id"),
        status: parse_status(&status_str),
        retry_count: row.get::<i32, _>("retry_count") as u32,
        last_error: row.get("last_error"),
        created_at: row.get("created_at"),
        sent_at: row.get("sent_at"),
    }
}

#[async_trait]
impl OutboxRepository for PgOutboxRepository {
    async fn append(&self, entry: &OutboxEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO outbox \
             (id, subject, payload, command_id, saga_id, status, retry_count, last_error, created_at, sent_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(entry.id)
        .bind(&entry.subject)
        .bind(&entry.payload)
        .bind(entry.command_id)
        .bind(entry.saga_id)
        .bind(status_to_str(entry.status))
        .bind(entry.retry_count as i32)
        .bind(&entry.last_error)
        .bind(entry.created_at)
        .bind(entry.sent_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
        let rows = sqlx::query(
            "SELECT id, subject, payload, command_id, saga_id, status, retry_count, last_error, created_at, sent_at \
             FROM outbox WHERE status = 'pending' ORDER BY created_at LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_entry).collect())
    }

    async fn mark_sent(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE outbox SET status = 'sent', sent_at = now(), last_error = NULL WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: String) -> Result<()> {
        sqlx::query(
            "UPDATE outbox SET retry_count = retry_count + 1, last_error = $1 WHERE id = $2",
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_giveup(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE outbox SET status = 'failed' WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ============================================================================
// InMemoryRepository（测用）
// ============================================================================

pub struct InMemoryOutboxRepository {
    inner: Mutex<HashMap<Uuid, OutboxEntry>>,
}

impl InMemoryOutboxRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryOutboxRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OutboxRepository for InMemoryOutboxRepository {
    async fn append(&self, entry: &OutboxEntry) -> Result<()> {
        self.inner.lock().unwrap().insert(entry.id, entry.clone());
        Ok(())
    }

    async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
        let mut all: Vec<OutboxEntry> = self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|e| e.status == OutboxStatus::Pending)
            .cloned()
            .collect();
        all.sort_by_key(|e| e.created_at);
        all.truncate(limit as usize);
        Ok(all)
    }

    async fn mark_sent(&self, id: Uuid) -> Result<()> {
        if let Some(e) = self.inner.lock().unwrap().get_mut(&id) {
            e.mark_sent();
        }
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: String) -> Result<()> {
        if let Some(e) = self.inner.lock().unwrap().get_mut(&id) {
            e.mark_failed(error);
        }
        Ok(())
    }

    async fn mark_giveup(&self, id: Uuid) -> Result<()> {
        if let Some(e) = self.inner.lock().unwrap().get_mut(&id) {
            e.mark_giveup();
        }
        Ok(())
    }
}

/// 通用 outbox 表 migration 模板（per 域复制使用）
pub const MIGRATION_TEMPLATE: &str = r#"
-- Outbox 表（per RGS-DTL-100 §5.3 事务性消息 + per RGS-SPEC-CROSS-005）
-- 54.11 模板：各域 migrations 应包含本表（或类似结构）
CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    subject TEXT NOT NULL,
    payload TEXT NOT NULL,
    command_id UUID NOT NULL,
    saga_id UUID,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'sent', 'failed')),
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox (status);
CREATE INDEX IF NOT EXISTS idx_outbox_created_at ON outbox (created_at);
CREATE INDEX IF NOT EXISTS idx_outbox_command_id ON outbox (command_id);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbox_entry_new_defaults() {
        let cmd_id = Uuid::new_v4();
        let entry = OutboxEntry::new(
            "rgs.player.registered.v1".to_string(),
            "{}".to_string(),
            cmd_id,
        );
        assert_eq!(entry.status, OutboxStatus::Pending);
        assert_eq!(entry.retry_count, 0);
        assert!(entry.last_error.is_none());
    }

    #[test]
    fn outbox_entry_mark_sent() {
        let mut e = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        e.mark_sent();
        assert_eq!(e.status, OutboxStatus::Sent);
        assert!(e.sent_at.is_some());
    }

    #[test]
    fn outbox_entry_retry_count_increments() {
        let mut e = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        e.mark_failed("timeout".to_string());
        e.mark_failed("timeout".to_string());
        assert_eq!(e.retry_count, 2);
        assert_eq!(e.last_error, Some("timeout".to_string()));
    }

    #[tokio::test]
    async fn in_memory_outbox_append_and_list() {
        let repo = InMemoryOutboxRepository::new();
        let entry = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        repo.append(&entry).await.unwrap();
        let list = repo.list_pending(10).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, entry.id);
    }

    #[tokio::test]
    async fn in_memory_outbox_mark_sent_removes_from_pending() {
        let repo = InMemoryOutboxRepository::new();
        let entry = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        let id = entry.id;
        repo.append(&entry).await.unwrap();
        repo.mark_sent(id).await.unwrap();
        let list = repo.list_pending(10).await.unwrap();
        assert_eq!(list.len(), 0);
    }
}
