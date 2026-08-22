//! economy-service 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository 测试用
//! 规范：RGS-DTL-015 §3 经济域数据访问层 + RGS-DTL-100 Saga 幂等性

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{
    Account, AccountStatus, Currency, TransactionKind, TransactionLedger, TransactionStatus,
};
use crate::Result;

/// Account Repository trait
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>>;
    async fn find_by_player_and_currency(
        &self,
        player_id: Uuid,
        currency: Currency,
    ) -> Result<Option<Account>>;
    /// OCC 乐观锁更新（version 必须匹配）
    async fn update_with_version(&self, account: &Account) -> Result<Account>;
    async fn save(&self, entity: &Account) -> Result<Account>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

/// TransactionLedger Repository trait
#[async_trait]
pub trait TransactionLedgerRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TransactionLedger>>;
    /// 幂等键查重（per RGS-DTL-100 §6 幂等性）
    async fn find_by_idempotency_key(&self, key: &str) -> Result<Option<TransactionLedger>>;
    /// 按 saga_id 列所有账目
    async fn list_by_saga(&self, saga_id: Uuid) -> Result<Vec<TransactionLedger>>;
    async fn save(&self, entity: &TransactionLedger) -> Result<TransactionLedger>;
}

// ============================================================================
// PgRepository（sqlx 实现）
// ============================================================================

pub struct PgAccountRepository {
    pool: PgPool,
}

impl PgAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_account(row: sqlx::postgres::PgRow) -> Account {
    let currency_str: String = row.get("currency");
    let status_str: String = row.get("status");
    Account {
        id: row.get("id"),
        player_id: row.get("player_id"),
        currency: parse_currency(&currency_str),
        balance: row.get("balance"),
        version: row.get("version"),
        status: parse_account_status(&status_str),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl AccountRepository for PgAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>> {
        let row = sqlx::query(
            "SELECT id, player_id, currency, balance, version, status, created_at, updated_at \
             FROM accounts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_account))
    }

    async fn find_by_player_and_currency(
        &self,
        player_id: Uuid,
        currency: Currency,
    ) -> Result<Option<Account>> {
        let row = sqlx::query(
            "SELECT id, player_id, currency, balance, version, status, created_at, updated_at \
             FROM accounts WHERE player_id = $1 AND currency = $2",
        )
        .bind(player_id)
        .bind(currency_to_str(currency))
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_account))
    }

    async fn update_with_version(&self, account: &Account) -> Result<Account> {
        // OCC：version 匹配才更新 + 自动 version+1
        let result = sqlx::query(
            "UPDATE accounts SET balance = $1, version = version + 1, status = $2, updated_at = $3 \
             WHERE id = $4 AND version = $5",
        )
        .bind(account.balance)
        .bind(account_status_to_str(account.status))
        .bind(account.updated_at)
        .bind(account.id)
        .bind(account.version)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(crate::Error::Validation(format!(
                "OCC conflict: account {} version {} stale",
                account.id, account.version
            )));
        }
        Ok(Account {
            version: account.version + 1,
            ..account.clone()
        })
    }

    async fn save(&self, entity: &Account) -> Result<Account> {
        sqlx::query(
            "INSERT INTO accounts (id, player_id, currency, balance, version, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                balance = EXCLUDED.balance, version = EXCLUDED.version, \
                status = EXCLUDED.status, updated_at = EXCLUDED.updated_at",
        )
        .bind(entity.id)
        .bind(entity.player_id)
        .bind(currency_to_str(entity.currency))
        .bind(entity.balance)
        .bind(entity.version)
        .bind(account_status_to_str(entity.status))
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM accounts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

pub struct PgTransactionLedgerRepository {
    pool: PgPool,
}

impl PgTransactionLedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_ledger(row: sqlx::postgres::PgRow) -> TransactionLedger {
    let currency_str: String = row.get("currency");
    let kind_str: String = row.get("kind");
    let status_str: String = row.get("status");
    TransactionLedger {
        id: row.get("id"),
        account_id: row.get("account_id"),
        idempotency_key: row.get("idempotency_key"),
        saga_id: row.get("saga_id"),
        command_id: row.get("command_id"),
        amount: row.get("amount"),
        currency: parse_currency(&currency_str),
        kind: parse_transaction_kind(&kind_str),
        status: parse_transaction_status(&status_str),
        memo: row.get("memo"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl TransactionLedgerRepository for PgTransactionLedgerRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TransactionLedger>> {
        let row = sqlx::query(
            "SELECT id, account_id, idempotency_key, saga_id, command_id, amount, currency, kind, status, memo, created_at \
             FROM transaction_ledger WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_ledger))
    }

    async fn find_by_idempotency_key(&self, key: &str) -> Result<Option<TransactionLedger>> {
        let row = sqlx::query(
            "SELECT id, account_id, idempotency_key, saga_id, command_id, amount, currency, kind, status, memo, created_at \
             FROM transaction_ledger WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_ledger))
    }

    async fn list_by_saga(&self, saga_id: Uuid) -> Result<Vec<TransactionLedger>> {
        let rows = sqlx::query(
            "SELECT id, account_id, idempotency_key, saga_id, command_id, amount, currency, kind, status, memo, created_at \
             FROM transaction_ledger WHERE saga_id = $1 ORDER BY created_at",
        )
        .bind(saga_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_ledger).collect())
    }

    async fn save(&self, entity: &TransactionLedger) -> Result<TransactionLedger> {
        sqlx::query(
            "INSERT INTO transaction_ledger \
             (id, account_id, idempotency_key, saga_id, command_id, amount, currency, kind, status, memo, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (id) DO UPDATE SET \
                status = EXCLUDED.status, memo = EXCLUDED.memo",
        )
        .bind(entity.id)
        .bind(entity.account_id)
        .bind(&entity.idempotency_key)
        .bind(entity.saga_id)
        .bind(entity.command_id)
        .bind(entity.amount)
        .bind(currency_to_str(entity.currency))
        .bind(transaction_kind_to_str(entity.kind))
        .bind(transaction_status_to_str(entity.status))
        .bind(&entity.memo)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryAccountRepository {
    inner: Mutex<HashMap<Uuid, Account>>,
}

impl InMemoryAccountRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_player_and_currency(
        &self,
        player_id: Uuid,
        currency: Currency,
    ) -> Result<Option<Account>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|a| a.player_id == player_id && a.currency == currency)
            .cloned())
    }
    async fn update_with_version(&self, account: &Account) -> Result<Account> {
        let mut guard = self.inner.lock().unwrap();
        match guard.get(&account.id) {
            Some(existing) if existing.version == account.version => {
                let updated = Account {
                    version: account.version + 1,
                    ..account.clone()
                };
                guard.insert(account.id, updated.clone());
                Ok(updated)
            }
            Some(_) => Err(crate::Error::Validation(format!(
                "OCC conflict: account {} version {} stale",
                account.id, account.version
            ))),
            None => Ok(account.clone()),
        }
    }
    async fn save(&self, entity: &Account) -> Result<Account> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
}

pub struct InMemoryTransactionLedgerRepository {
    inner: Mutex<HashMap<Uuid, TransactionLedger>>,
}

impl InMemoryTransactionLedgerRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryTransactionLedgerRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransactionLedgerRepository for InMemoryTransactionLedgerRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<TransactionLedger>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_idempotency_key(&self, key: &str) -> Result<Option<TransactionLedger>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|t| t.idempotency_key == key)
            .cloned())
    }
    async fn list_by_saga(&self, saga_id: Uuid) -> Result<Vec<TransactionLedger>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|t| t.saga_id == Some(saga_id))
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &TransactionLedger) -> Result<TransactionLedger> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
}

// ============================================================================
// helpers
// ============================================================================

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

fn account_status_to_str(s: AccountStatus) -> &'static str {
    match s {
        AccountStatus::Active => "active",
        AccountStatus::Frozen => "frozen",
        AccountStatus::Closed => "closed",
    }
}

fn parse_account_status(s: &str) -> AccountStatus {
    match s {
        "frozen" => AccountStatus::Frozen,
        "closed" => AccountStatus::Closed,
        _ => AccountStatus::Active,
    }
}

fn transaction_kind_to_str(k: TransactionKind) -> &'static str {
    match k {
        TransactionKind::Deposit => "deposit",
        TransactionKind::Spend => "spend",
        TransactionKind::Transfer => "transfer",
        TransactionKind::Refund => "refund",
        TransactionKind::Compensation => "compensation",
    }
}

fn parse_transaction_kind(s: &str) -> TransactionKind {
    match s {
        "deposit" => TransactionKind::Deposit,
        "spend" => TransactionKind::Spend,
        "transfer" => TransactionKind::Transfer,
        "refund" => TransactionKind::Refund,
        _ => TransactionKind::Compensation,
    }
}

fn transaction_status_to_str(s: TransactionStatus) -> &'static str {
    match s {
        TransactionStatus::Pending => "pending",
        TransactionStatus::Confirmed => "confirmed",
        TransactionStatus::Reversed => "reversed",
        TransactionStatus::Failed => "failed",
    }
}

fn parse_transaction_status(s: &str) -> TransactionStatus {
    match s {
        "confirmed" => TransactionStatus::Confirmed,
        "reversed" => TransactionStatus::Reversed,
        "failed" => TransactionStatus::Failed,
        _ => TransactionStatus::Pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_account_occ() {
        let repo = InMemoryAccountRepository::new();
        let player_id = Uuid::new_v4();
        let mut account = Account::new(player_id, Currency::Gold);
        account.credit(100);
        repo.save(&account).await.unwrap();

        let mut a2 = repo.find_by_id(account.id).await.unwrap().unwrap();
        a2.credit(50);
        let updated = repo.update_with_version(&a2).await.unwrap();
        assert_eq!(updated.balance, 150);
        assert_eq!(updated.version, 1);

        // version mismatch → 失败
        let stale = Account {
            version: 0,
            ..updated.clone()
        };
        assert!(repo.update_with_version(&stale).await.is_err());
    }

    #[tokio::test]
    async fn in_memory_ledger_idempotency() {
        let repo = InMemoryTransactionLedgerRepository::new();
        let key = "saga-x-cmd-y-001".to_string();
        let entry = TransactionLedger::new(
            Uuid::new_v4(),
            100,
            Currency::Diamond,
            TransactionKind::Deposit,
            key.clone(),
        );
        repo.save(&entry).await.unwrap();
        let found = repo.find_by_idempotency_key(&key).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().amount, 100);
    }
}
