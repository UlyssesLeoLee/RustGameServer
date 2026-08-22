//! economy-service 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-015 §3 经济域数据模型 + Saga Q-003）
//! - Account：账户（balance + version OCC + currency）
//! - TransactionLedger：账目（saga_id + command_id + idempotency_key 三件套）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 货币类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Currency {
    /// 金币
    Gold,
    /// 钻石
    Diamond,
    /// 代币
    Token,
}

/// 账户状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// 正常
    Active,
    /// 冻结
    Frozen,
    /// 关闭
    Closed,
}

/// 玩家账户（root entity，per RGS-DTL-015 §3.1）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    /// 账户 ID
    pub id: Uuid,
    /// 所属玩家 ID
    pub player_id: Uuid,
    /// 货币类型
    pub currency: Currency,
    /// 余额（最小单位：分 / 钻 / 代币）
    pub balance: i64,
    /// OCC 乐观锁版本号
    pub version: i64,
    /// 状态
    pub status: AccountStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// 工厂：新建账户（balance=0 / version=0 / Active）
    pub fn new(player_id: Uuid, currency: Currency) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            player_id,
            currency,
            balance: 0,
            version: 0,
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// 存款（不更新 version，调用方需走 OCC）
    pub fn credit(&mut self, amount: i64) {
        self.balance += amount;
        self.updated_at = Utc::now();
    }

    /// 取款（返回 false = 余额不足；调用方需走 OCC）
    pub fn try_debit(&mut self, amount: i64) -> bool {
        if self.balance < amount {
            return false;
        }
        self.balance -= amount;
        self.updated_at = Utc::now();
        true
    }
}

/// 账目类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    /// 充值
    Deposit,
    /// 消费
    Spend,
    /// 转账
    Transfer,
    /// 退款
    Refund,
    /// 补偿（Saga）
    Compensation,
}

/// 账目状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// 待处理
    Pending,
    /// 已确认
    Confirmed,
    /// 已撤销（Saga 补偿）
    Reversed,
    /// 失败
    Failed,
}

/// 账目记录（per RGS-DTL-100 Saga 幂等三件套 + DTL-015 §3.2 账目模型）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionLedger {
    /// 交易 ID
    pub id: Uuid,
    /// 账户 ID
    pub account_id: Uuid,
    /// 业务幂等键（command_id / saga_id / idempotency_key 三件套合一）
    pub idempotency_key: String,
    /// 关联 Saga ID（None = 非 Saga 触发的交易）
    pub saga_id: Option<Uuid>,
    /// 原始 command_id（per RGS-DTL-100 §6 幂等性）
    pub command_id: Option<Uuid>,
    /// 金额（正 = 收入 / 负 = 支出）
    pub amount: i64,
    /// 货币
    pub currency: Currency,
    /// 类型
    pub kind: TransactionKind,
    /// 状态
    pub status: TransactionStatus,
    /// 备注
    pub memo: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl TransactionLedger {
    /// 工厂：新建账目（默认 Pending）
    pub fn new(
        account_id: Uuid,
        amount: i64,
        currency: Currency,
        kind: TransactionKind,
        idempotency_key: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            idempotency_key,
            saga_id: None,
            command_id: None,
            amount,
            currency,
            kind,
            status: TransactionStatus::Pending,
            memo: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_new_defaults() {
        let account = Account::new(Uuid::new_v4(), Currency::Gold);
        assert_eq!(account.balance, 0);
        assert_eq!(account.version, 0);
        assert_eq!(account.status, AccountStatus::Active);
    }

    #[test]
    fn account_credit_debit() {
        let mut account = Account::new(Uuid::new_v4(), Currency::Gold);
        account.credit(100);
        assert_eq!(account.balance, 100);
        assert!(account.try_debit(30));
        assert_eq!(account.balance, 70);
        assert!(!account.try_debit(100));
        assert_eq!(account.balance, 70);
    }

    #[test]
    fn ledger_idempotency_key_required() {
        let entry = TransactionLedger::new(
            Uuid::new_v4(),
            100,
            Currency::Diamond,
            TransactionKind::Deposit,
            "test-key-001".to_string(),
        );
        assert_eq!(entry.idempotency_key, "test-key-001");
        assert_eq!(entry.status, TransactionStatus::Pending);
    }
}
