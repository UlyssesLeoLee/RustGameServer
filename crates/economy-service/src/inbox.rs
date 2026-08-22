//! economy-service Inbox 模式（per RGS-DTL-100 §6 幂等性）
//!
//! 54.8 实化：InboxEntry entity + Repository trait + Pg/InMemory impl
//!
//! 设计：处理 command 前先 check inbox（command_id + handler），已处理直接返回原结果。
//! 防止消息重投 / 客户端重试导致重复扣款。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::Result;

/// Inbox 处理结果
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InboxStatus {
    /// 已处理
    Processed,
    /// 处理失败（可重试）
    Failed,
}

/// Inbox 实体（per RGS-DTL-100 §6 幂等性）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InboxEntry {
    /// Inbox ID
    pub id: Uuid,
    /// command_id（业务幂等键）
    pub command_id: Uuid,
    /// handler 名称（如 "saga.transfer"）
    pub handler: String,
    /// 处理结果 JSON
    pub result: String,
    /// 状态
    pub status: InboxStatus,
    /// 处理时间
    pub processed_at: DateTime<Utc>,
}

impl InboxEntry {
    /// 工厂：新建已处理 inbox
    pub fn new(command_id: Uuid, handler: String, result: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            command_id,
            handler,
            result,
            status: InboxStatus::Processed,
            processed_at: Utc::now(),
        }
    }
}

/// Inbox Repository trait
#[async_trait]
pub trait InboxRepository: Send + Sync {
    /// 按 (command_id, handler) 查（幂等性 check）
    async fn find_by_command(&self, command_id: Uuid, handler: &str) -> Result<Option<InboxEntry>>;
    /// 追加处理结果
    async fn append(&self, entry: &InboxEntry) -> Result<InboxEntry>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgInboxRepository {
    pool: PgPool,
}

impl PgInboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn status_to_str(s: InboxStatus) -> &'static str {
    match s {
        InboxStatus::Processed => "processed",
        InboxStatus::Failed => "failed",
    }
}

fn parse_status(s: &str) -> InboxStatus {
    match s {
        "failed" => InboxStatus::Failed,
        _ => InboxStatus::Processed,
    }
}

fn row_to_inbox(row: sqlx::postgres::PgRow) -> InboxEntry {
    let status_str: String = row.get("status");
    InboxEntry {
        id: row.get("id"),
        command_id: row.get("command_id"),
        handler: row.get("handler"),
        result: row.get("result"),
        status: parse_status(&status_str),
        processed_at: row.get("processed_at"),
    }
}

#[async_trait]
impl InboxRepository for PgInboxRepository {
    async fn find_by_command(&self, command_id: Uuid, handler: &str) -> Result<Option<InboxEntry>> {
        let row = sqlx::query(
            "SELECT id, command_id, handler, result, status, processed_at \
             FROM inbox WHERE command_id = $1 AND handler = $2",
        )
        .bind(command_id)
        .bind(handler)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_inbox))
    }

    async fn append(&self, entry: &InboxEntry) -> Result<InboxEntry> {
        sqlx::query(
            "INSERT INTO inbox (id, command_id, handler, result, status, processed_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (command_id, handler) DO NOTHING",
        )
        .bind(entry.id)
        .bind(entry.command_id)
        .bind(&entry.handler)
        .bind(&entry.result)
        .bind(status_to_str(entry.status))
        .bind(entry.processed_at)
        .execute(&self.pool)
        .await?;
        Ok(entry.clone())
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryInboxRepository {
    inner: Mutex<HashMap<(Uuid, String), InboxEntry>>,
}

impl InMemoryInboxRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryInboxRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InboxRepository for InMemoryInboxRepository {
    async fn find_by_command(&self, command_id: Uuid, handler: &str) -> Result<Option<InboxEntry>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .get(&(command_id, handler.to_string()))
            .cloned())
    }
    async fn append(&self, entry: &InboxEntry) -> Result<InboxEntry> {
        self.inner
            .lock()
            .unwrap()
            .insert((entry.command_id, entry.handler.clone()), entry.clone());
        Ok(entry.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_inbox_idempotency() {
        let repo = InMemoryInboxRepository::new();
        let cmd_id = Uuid::new_v4();
        let entry = InboxEntry::new(
            cmd_id,
            "saga.transfer".to_string(),
            r#"{"ok":true}"#.to_string(),
        );
        repo.append(&entry).await.unwrap();

        // 第二次处理同 command_id
        let found = repo.find_by_command(cmd_id, "saga.transfer").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().result, r#"{"ok":true}"#);
    }

    #[tokio::test]
    async fn in_memory_inbox_different_handlers_independent() {
        let repo = InMemoryInboxRepository::new();
        let cmd_id = Uuid::new_v4();
        let e1 = InboxEntry::new(cmd_id, "handler-a".to_string(), "r1".to_string());
        let e2 = InboxEntry::new(cmd_id, "handler-b".to_string(), "r2".to_string());
        repo.append(&e1).await.unwrap();
        repo.append(&e2).await.unwrap();

        assert!(repo
            .find_by_command(cmd_id, "handler-a")
            .await
            .unwrap()
            .is_some());
        assert!(repo
            .find_by_command(cmd_id, "handler-b")
            .await
            .unwrap()
            .is_some());
    }
}
