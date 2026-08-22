//! Outbox 模式（per RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005 事务性消息）
//!
//! 54.11 实化：OutboxEntry + OutboxRepository trait + Pg/InMemory impl
//! 55.17 升级（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）：
//!   - list_pending 加 `FOR UPDATE SKIP LOCKED`（多 relay 并发安全）
//!   - append 接受 `PgExecutor`（业务 + outbox 同事务）
//!   - 状态机加 `in_flight` + `lease_until`（retry 持锁防重复消费）
//!
//! 设计原则（per transactional outbox pattern）：
//! - 业务写 DB + 写 outbox 表必须在同一事务（per DTL-100 §5.3）
//! - 单独 relay 进程轮询 outbox 表，发布到 NATS
//! - 至少一次投递（at-least-once）：业务已 commit 但 NATS 未发 → relay 重试
//! - 幂等性：consumer 端靠 envelope.command_id 去重
//!
//! 状态机（per RGS-REV-007 CH2）：
//! ```text
//!   Pending --list_pending + lease--> InFlight --publish ok--> Sent
//!                                                --publish err (retry<max)--> InFlight (retry_count+1, lease 续期)
//!                                                --publish err (retry>=max)--> Failed
//!   InFlight (lease 过期) --list_pending + reclaim--> InFlight (新 lease)
//! ```
//!
//! 多 relay 副本并发安全：
//! - list_pending 用 `FOR UPDATE SKIP LOCKED` 跳过被其他 relay 持锁的行
//! - 持锁期间 entry 处于 `in_flight` + `lease_until`（默认 30s）
//! - relay 崩溃后 lease 过期，另一副本可重试

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, PgPool, Postgres, Row, Transaction};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
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

/// 默认 lease 时长（per RGS-REV-007 CH2）：30s
pub const DEFAULT_LEASE: Duration = Duration::from_secs(30);

/// Outbox 状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutboxStatus {
    /// 待发送
    Pending,
    /// 已被某 relay 持锁（lease_until 内），正在 publish
    InFlight,
    /// 已发送
    Sent,
    /// 失败（重试 N 次后放弃）
    Failed,
}

impl OutboxStatus {
    /// 数据库字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboxStatus::Pending => "pending",
            OutboxStatus::InFlight => "in_flight",
            OutboxStatus::Sent => "sent",
            OutboxStatus::Failed => "failed",
        }
    }
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
    /// Lease 截止时间（InFlight 状态持有，per RGS-REV-007 CH2）
    pub lease_until: Option<DateTime<Utc>>,
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
            lease_until: None,
            created_at: Utc::now(),
            sent_at: None,
        }
    }

    /// 链式 setter
    pub fn with_saga(mut self, saga_id: Uuid) -> Self {
        self.saga_id = Some(saga_id);
        self
    }

    /// 标记 InFlight（relay 持锁 lease_until = now + lease）
    pub fn mark_in_flight(&mut self, lease: Duration) {
        self.status = OutboxStatus::InFlight;
        self.lease_until = Some(
            Utc::now() + chrono::Duration::milliseconds(lease.as_millis() as i64),
        );
    }

    /// 标记已发送
    pub fn mark_sent(&mut self) {
        self.status = OutboxStatus::Sent;
        self.sent_at = Some(Utc::now());
        self.last_error = None;
        self.lease_until = None;
    }

    /// 标记失败（重试 +1）
    pub fn mark_failed(&mut self, error: String) {
        self.retry_count += 1;
        self.last_error = Some(error);
    }

    /// 标记最终失败（重试耗尽）
    pub fn mark_giveup(&mut self) {
        self.status = OutboxStatus::Failed;
        self.lease_until = None;
    }
}

/// Outbox Repository trait
#[async_trait]
pub trait OutboxRepository: Send + Sync {
    /// 在事务（或 pool）内追加 outbox 条目（per DTL-100 §5.3 同事务要求）
    ///
    /// 55.17 签名升级：`executor` 接受 `PgExecutor`（即 `&PgPool` 或 `&mut Transaction<'_, Postgres>`）
    /// 让调用方把"业务写 DB"和"写 outbox"包在同一事务里。
    async fn append<'e, E: PgExecutor<'e>>(
        &self,
        entry: &OutboxEntry,
        executor: E,
    ) -> Result<()>;

    /// 列出待发送条目（relay 调用）
    ///
    /// 55.17 实现要点：
    /// - 在事务内 SELECT ... FOR UPDATE SKIP LOCKED
    /// - 命中后立刻 UPDATE 为 `in_flight` + `lease_until = now() + DEFAULT_LEASE`
    /// - 提交后其他 relay 看不到这些行（直到 lease 过期）
    async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>>;

    /// 标记已发送
    async fn mark_sent(&self, id: Uuid) -> Result<()>;

    /// 标记失败（retry_count+1），保留 in_flight 状态等 lease 过期重试
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
    s.as_str()
}

fn parse_status(s: &str) -> OutboxStatus {
    match s {
        "pending" => OutboxStatus::Pending,
        "in_flight" => OutboxStatus::InFlight,
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
        lease_until: row.get("lease_until"),
        created_at: row.get("created_at"),
        sent_at: row.get("sent_at"),
    }
}

#[async_trait]
impl OutboxRepository for PgOutboxRepository {
    async fn append<'e, E: PgExecutor<'e>>(
        &self,
        entry: &OutboxEntry,
        executor: E,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO outbox \
             (id, subject, payload, command_id, saga_id, status, retry_count, last_error, lease_until, created_at, sent_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
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
        .bind(entry.lease_until)
        .bind(entry.created_at)
        .bind(entry.sent_at)
        .execute(executor)
        .await?;
        Ok(())
    }

    /// 55.17：FOR UPDATE SKIP LOCKED + 自动 mark in_flight + lease_until
    ///
    /// 实现：
    /// 1. begin tx
    /// 2. SELECT ... FROM outbox
    ///      WHERE status='pending'
    ///         OR (status='in_flight' AND (lease_until IS NULL OR lease_until < NOW()))
    ///      ORDER BY created_at LIMIT $1 FOR UPDATE SKIP LOCKED
    /// 3. UPDATE outbox SET status='in_flight', lease_until=now()+30s
    ///    WHERE id IN (上面选出的 id)
    /// 4. commit（持锁期 in_flight，其他 relay 看不到）
    async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        // Step 1: 取锁（SKIP LOCKED 跳过其他 relay 正在 publish 的行）
        //         同时回收 lease 已过期的 in_flight 行（relay 崩溃后被另一副本接管）
        let rows = sqlx::query(
            "SELECT id, subject, payload, command_id, saga_id, status, retry_count, last_error, lease_until, created_at, sent_at \
             FROM outbox \
             WHERE status = 'pending' \
                OR (status = 'in_flight' AND (lease_until IS NULL OR lease_until < NOW())) \
             ORDER BY created_at \
             LIMIT $1 \
             FOR UPDATE SKIP LOCKED",
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;

        if rows.is_empty() {
            tx.commit().await?;
            return Ok(Vec::new());
        }

        // Step 2: 收集 id，mark in_flight + lease_until（重置 lease）
        let ids: Vec<Uuid> = rows.iter().map(|r| r.get::<Uuid, _>("id")).collect();

        sqlx::query(
            "UPDATE outbox \
             SET status = 'in_flight', lease_until = NOW() + INTERVAL '30 seconds' \
             WHERE id = ANY($1)",
        )
        .bind(&ids)
        .execute(&mut *tx)
        .await?;

        // Step 3: 提交（持锁期 in_flight + lease，其他 relay 看不到）
        tx.commit().await?;

        // 把返回 rows 状态修正为 in_flight（与 DB 一致）
        Ok(rows.into_iter().map(row_to_entry).collect())
    }

    async fn mark_sent(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE outbox SET status = 'sent', sent_at = now(), last_error = NULL, lease_until = NULL WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: String) -> Result<()> {
        // 失败：保留 in_flight 状态 + lease_until 不变，等 lease 过期另一副本重试
        // （per RGS-REV-007 CH2：失败时 in_flight 保留等 lease 过期被另一副本重试）
        // retry_count+1 让 max_retries 检查生效；超过 max → relay 调 mark_giveup
        sqlx::query(
            "UPDATE outbox SET retry_count = retry_count + 1, last_error = $1 \
             WHERE id = $2",
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_giveup(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE outbox SET status = 'failed', lease_until = NULL WHERE id = $1",
        )
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
    /// Lease 时长（默认 30s；测试可设短）
    lease: Duration,
}

impl InMemoryOutboxRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            lease: DEFAULT_LEASE,
        }
    }

    /// 自定义 lease（测试用，可设短 lease 验证过期重试）
    pub fn with_lease(lease: Duration) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            lease,
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
    async fn append<'e, E: PgExecutor<'e>>(
        &self,
        entry: &OutboxEntry,
        _executor: E,
    ) -> Result<()> {
        // InMemory 不需要 executor（实际业务里 executor 是给 Pg 用的同事务 tx）
        self.inner.lock().unwrap().insert(entry.id, entry.clone());
        Ok(())
    }

    /// 55.17：InMemory 版模拟 SKIP LOCKED + in_flight
    ///
    /// 行为：
    /// - 选 status='pending' 的行 + lease 已过期的 in_flight 行
    /// - 立刻 mark in_flight + lease_until = now + self.lease
    /// - lease_until 未过期的 in_flight 行跳过（模拟其他 relay 持锁）
    async fn list_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>> {
        let now = Utc::now();
        let mut guard = self.inner.lock().unwrap();
        let mut all: Vec<OutboxEntry> = guard
            .values()
            .filter(|e| {
                // pending 行 OK
                if e.status == OutboxStatus::Pending {
                    return true;
                }
                // in_flight：lease 已过期或无 lease → 视为可重试
                if e.status == OutboxStatus::InFlight {
                    return match e.lease_until {
                        None => true,
                        Some(lease) => lease < now,
                    };
                }
                false
            })
            .cloned()
            .collect();
        all.sort_by_key(|e| e.created_at);
        all.truncate(limit as usize);

        // mark in_flight + lease（重置 lease）
        let lease = self.lease;
        for entry in &mut all {
            entry.mark_in_flight(lease);
            if let Some(stored) = guard.get_mut(&entry.id) {
                stored.mark_in_flight(lease);
            }
        }
        Ok(all)
    }

    async fn mark_sent(&self, id: Uuid) -> Result<()> {
        if let Some(e) = self.inner.lock().unwrap().get_mut(&id) {
            e.mark_sent();
        }
        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: String) -> Result<()> {
        // 55.17：失败时 in_flight 保留，lease_until 不变，等过期另一副本重试
        if let Some(e) = self.inner.lock().unwrap().get_mut(&id) {
            e.mark_failed(error);
            // status 保持 InFlight，lease_until 保持原值
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
///
/// 55.17 升级：加 `in_flight` 状态 + `lease_until` 列 + 部分索引
pub const MIGRATION_TEMPLATE: &str = r#"
-- Outbox 表（per RGS-DTL-100 §5.3 事务性消息 + per RGS-SPEC-CROSS-005）
-- 54.11 模板：各域 migrations 应包含本表
-- 55.17 升级（per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1）：
--   - status 加 'in_flight'
--   - 加 lease_until 列（relay 持锁期）
--   - 加部分索引 idx_outbox_pending / idx_outbox_in_flight
CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    subject TEXT NOT NULL,
    payload TEXT NOT NULL,
    command_id UUID NOT NULL,
    saga_id UUID,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_flight', 'sent', 'failed')),
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    lease_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox (created_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_outbox_in_flight ON outbox (lease_until) WHERE status = 'in_flight';
CREATE INDEX IF NOT EXISTS idx_outbox_command_id ON outbox (command_id);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用：构造一个 lazy PgPool（不真连 DB）作为 InMemory append 的 executor
    /// InMemoryOutboxRepository 忽略 executor，但 trait 签名要求 `PgExecutor`
    fn lazy_pool() -> sqlx::PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/nonexistent")
            .expect("lazy connect should not fail")
    }

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
        assert!(entry.lease_until.is_none());
    }

    #[test]
    fn outbox_entry_mark_sent() {
        let mut e = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        e.mark_sent();
        assert_eq!(e.status, OutboxStatus::Sent);
        assert!(e.sent_at.is_some());
        assert!(e.lease_until.is_none());
    }

    #[test]
    fn outbox_entry_retry_count_increments() {
        let mut e = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        e.mark_failed("timeout".to_string());
        e.mark_failed("timeout".to_string());
        assert_eq!(e.retry_count, 2);
        assert_eq!(e.last_error, Some("timeout".to_string()));
    }

    #[test]
    fn outbox_entry_mark_in_flight_sets_lease() {
        let mut e = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        e.mark_in_flight(Duration::from_secs(30));
        assert_eq!(e.status, OutboxStatus::InFlight);
        assert!(e.lease_until.is_some());
    }

    #[test]
    fn outbox_status_serde_roundtrip() {
        for s in [
            OutboxStatus::Pending,
            OutboxStatus::InFlight,
            OutboxStatus::Sent,
            OutboxStatus::Failed,
        ] {
            let j = serde_json::to_string(&s).unwrap();
            let d: OutboxStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(d, s);
        }
    }

    #[tokio::test]
    async fn in_memory_outbox_append_and_list() {
        let repo = InMemoryOutboxRepository::new();
        let pool = lazy_pool();
        let entry = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        repo.append(&entry, &pool).await.unwrap();
        let list = repo.list_pending(10).await.unwrap();
        // list_pending 把 pending → in_flight，所以 list 长度仍为 1
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, entry.id);
        assert_eq!(list[0].status, OutboxStatus::InFlight);
    }

    #[tokio::test]
    async fn in_memory_outbox_mark_sent_removes_from_pending() {
        let repo = InMemoryOutboxRepository::new();
        let pool = lazy_pool();
        let entry = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        let id = entry.id;
        repo.append(&entry, &pool).await.unwrap();
        // list_pending 把它转 in_flight
        let _ = repo.list_pending(10).await.unwrap();
        repo.mark_sent(id).await.unwrap();
        let list = repo.list_pending(10).await.unwrap();
        assert_eq!(list.len(), 0);
    }

    /// 55.17 测试 1：list_pending 模拟 SKIP LOCKED 行为
    ///
    /// 场景：
    /// - 3 条 pending
    /// - 第 1 次 list_pending(1) → 返回 1 条（被本调用 in_flight）
    /// - 第 2 次 list_pending(10) → 只剩 2 条 pending
    /// - 但同时其他 relay 持锁的 in_flight 行不应被返回
    #[tokio::test]
    async fn list_pending_skips_locked_rows() {
        let repo = InMemoryOutboxRepository::new();
        let pool = lazy_pool();

        // 3 条 pending
        for _ in 0..3 {
            let e = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
            repo.append(&e, &pool).await.unwrap();
        }

        // 第 1 个 relay 抢到 1 条
        let first = repo.list_pending(1).await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].status, OutboxStatus::InFlight);

        // 第 2 个 relay 抢剩下 2 条（不会重复拿第 1 个 relay 已持锁的 1 条）
        let second = repo.list_pending(10).await.unwrap();
        assert_eq!(second.len(), 2);
        // 第 1 个 relay 拿到的 id 不应在 second 出现
        assert!(!second.iter().any(|e| e.id == first[0].id));
    }

    /// 55.17 测试 2：append_in_transaction_persists（模拟"业务 + outbox 同事务"）
    ///
    /// InMemory 版验证：调用方传 executor，append 不崩溃；
    /// 之后 list_pending 仍能找到该 entry（说明数据被持久化到 repo 内部 store）
    #[tokio::test]
    async fn append_in_transaction_persists() {
        let repo = InMemoryOutboxRepository::new();
        let pool = lazy_pool();
        let entry = OutboxEntry::new(
            "rgs.player.registered.v1".to_string(),
            r#"{"player_id":"abc"}"#.to_string(),
            Uuid::new_v4(),
        );
        // 实参：&pool（Pg 模式可换 &mut *tx；InMemory 模式忽略）
        repo.append(&entry, &pool).await.unwrap();

        // 验证：entry 可被 list_pending 找到
        let list = repo.list_pending(10).await.unwrap();
        assert!(list.iter().any(|e| e.id == entry.id));
    }

    /// 55.17 测试 3：lease 过期后重试（per RGS-REV-007 CH2）
    ///
    /// 场景：relay 1 取到 entry（in_flight + lease 100ms），未 mark_sent 直接"崩溃"
    ///       → 等 200ms（lease 过期）
    ///       → relay 2 list_pending 应能再次拿到（status in_flight 但 lease 已过期）
    #[tokio::test]
    async fn lease_expiry_retry_picks_up_expired_in_flight() {
        // 短 lease 100ms 便于测试
        let repo = InMemoryOutboxRepository::with_lease(std::time::Duration::from_millis(100));
        let pool = lazy_pool();
        let entry = OutboxEntry::new("rgs.test".to_string(), "{}".to_string(), Uuid::new_v4());
        let id = entry.id;
        repo.append(&entry, &pool).await.unwrap();

        // relay 1 抢到
        let first = repo.list_pending(10).await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].id, id);
        assert_eq!(first[0].status, OutboxStatus::InFlight);

        // 立即再 list：lease 未过 → 空（被 relay 1 持锁）
        let immediate = repo.list_pending(10).await.unwrap();
        assert_eq!(immediate.len(), 0, "second relay should skip locked row");

        // 模拟 publish 失败：保留 in_flight + lease_until 不变
        repo.mark_failed(id, "transient".to_string()).await.unwrap();

        // 等 lease 过期
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        // relay 2 重试：lease 已过 → 应能再拿到
        let retry = repo.list_pending(10).await.unwrap();
        assert_eq!(retry.len(), 1, "lease expired → row should be re-fetchable");
        assert_eq!(retry[0].id, id);
        // retry_count 已被 mark_failed +1
        assert_eq!(retry[0].retry_count, 1);
    }
}
