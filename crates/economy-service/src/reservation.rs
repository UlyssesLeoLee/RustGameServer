//! economy-service Reservation 模式（per RGS-DTL-100 §3.2）
//!
//! 54.8 实化：Reservation entity + Repository trait + Pg/InMemory impl
//!
//! 设计：Reservation 不直接修改 balance，只 mark 占用；confirm 才真扣，compensate 释放。
//! 防止 Saga 半完成时 balance 被锁死。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::Currency;
use crate::Result;

/// Reservation 状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReservationStatus {
    /// 已预留（待 confirm / compensate）
    Reserved,
    /// 已确认（实际扣款）
    Confirmed,
    /// 已补偿（释放）
    Compensated,
    /// 已过期
    Expired,
}

/// 资金预留（per RGS-DTL-100 §3.2 关键能力）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reservation {
    /// Reservation ID
    pub id: Uuid,
    /// 关联 Saga ID
    pub saga_id: Uuid,
    /// 关联账户 ID
    pub account_id: Uuid,
    /// 金额
    pub amount: i64,
    /// 货币
    pub currency: Currency,
    /// 状态
    pub status: ReservationStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间（默认 5 分钟）
    pub expires_at: DateTime<Utc>,
}

impl Reservation {
    /// 工厂：新建 reservation（默认 5 分钟过期）
    pub fn new(saga_id: Uuid, account_id: Uuid, amount: i64, currency: Currency) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            saga_id,
            account_id,
            amount,
            currency,
            status: ReservationStatus::Reserved,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
        }
    }

    /// 确认
    pub fn confirm(&mut self) {
        self.status = ReservationStatus::Confirmed;
    }

    /// 补偿（释放）
    pub fn compensate(&mut self) {
        self.status = ReservationStatus::Compensated;
    }

    /// 标记过期
    pub fn mark_expired(&mut self) {
        self.status = ReservationStatus::Expired;
    }

    /// 是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Reservation Repository trait
#[async_trait]
pub trait ReservationRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Reservation>>;
    async fn list_by_saga(&self, saga_id: Uuid) -> Result<Vec<Reservation>>;
    async fn save(&self, entity: &Reservation) -> Result<Reservation>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgReservationRepository {
    pool: PgPool,
}

impl PgReservationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn currency_to_str(c: Currency) -> &'static str {
    match c {
        Currency::Gold => "gold",
        Currency::Diamond => "diamond",
        Currency::Token => "token",
    }
}

fn parse_currency(s: &str) -> Currency {
    match s {
        "diamond" => Currency::Diamond,
        "token" => Currency::Token,
        _ => Currency::Gold,
    }
}

fn status_to_str(s: ReservationStatus) -> &'static str {
    match s {
        ReservationStatus::Reserved => "reserved",
        ReservationStatus::Confirmed => "confirmed",
        ReservationStatus::Compensated => "compensated",
        ReservationStatus::Expired => "expired",
    }
}

fn parse_status(s: &str) -> ReservationStatus {
    match s {
        "confirmed" => ReservationStatus::Confirmed,
        "compensated" => ReservationStatus::Compensated,
        "expired" => ReservationStatus::Expired,
        _ => ReservationStatus::Reserved,
    }
}

fn row_to_reservation(row: sqlx::postgres::PgRow) -> Reservation {
    let currency_str: String = row.get("currency");
    let status_str: String = row.get("status");
    Reservation {
        id: row.get("id"),
        saga_id: row.get("saga_id"),
        account_id: row.get("account_id"),
        amount: row.get("amount"),
        currency: parse_currency(&currency_str),
        status: parse_status(&status_str),
        created_at: row.get("created_at"),
        expires_at: row.get("expires_at"),
    }
}

#[async_trait]
impl ReservationRepository for PgReservationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Reservation>> {
        let row = sqlx::query(
            "SELECT id, saga_id, account_id, amount, currency, status, created_at, expires_at \
             FROM reservations WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_reservation))
    }

    async fn list_by_saga(&self, saga_id: Uuid) -> Result<Vec<Reservation>> {
        let rows = sqlx::query(
            "SELECT id, saga_id, account_id, amount, currency, status, created_at, expires_at \
             FROM reservations WHERE saga_id = $1 ORDER BY created_at",
        )
        .bind(saga_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_reservation).collect())
    }

    async fn save(&self, entity: &Reservation) -> Result<Reservation> {
        sqlx::query(
            "INSERT INTO reservations \
             (id, saga_id, account_id, amount, currency, status, created_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                status = EXCLUDED.status",
        )
        .bind(entity.id)
        .bind(entity.saga_id)
        .bind(entity.account_id)
        .bind(entity.amount)
        .bind(currency_to_str(entity.currency))
        .bind(status_to_str(entity.status))
        .bind(entity.created_at)
        .bind(entity.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryReservationRepository {
    inner: Mutex<HashMap<Uuid, Reservation>>,
}

impl InMemoryReservationRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryReservationRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReservationRepository for InMemoryReservationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Reservation>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_saga(&self, saga_id: Uuid) -> Result<Vec<Reservation>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.saga_id == saga_id)
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &Reservation) -> Result<Reservation> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_default_expiry() {
        let r = Reservation::new(Uuid::new_v4(), Uuid::new_v4(), 100, Currency::Gold);
        assert_eq!(r.amount, 100);
        assert_eq!(r.status, ReservationStatus::Reserved);
        assert!(!r.is_expired());
        assert!(r.expires_at > r.created_at);
    }

    #[tokio::test]
    async fn in_memory_reservation_lifecycle() {
        let repo = InMemoryReservationRepository::new();
        let saga_id = Uuid::new_v4();
        let r = Reservation::new(saga_id, Uuid::new_v4(), 50, Currency::Diamond);
        let id = r.id;
        repo.save(&r).await.unwrap();
        let mut loaded = repo.find_by_id(id).await.unwrap().unwrap();
        loaded.confirm();
        repo.save(&loaded).await.unwrap();
        let after = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(after.status, ReservationStatus::Confirmed);
    }

    #[tokio::test]
    async fn list_by_saga() {
        let repo = InMemoryReservationRepository::new();
        let saga_id = Uuid::new_v4();
        repo.save(&Reservation::new(
            saga_id,
            Uuid::new_v4(),
            10,
            Currency::Gold,
        ))
        .await
        .unwrap();
        repo.save(&Reservation::new(
            saga_id,
            Uuid::new_v4(),
            20,
            Currency::Gold,
        ))
        .await
        .unwrap();
        let list = repo.list_by_saga(saga_id).await.unwrap();
        assert_eq!(list.len(), 2);
    }
}
