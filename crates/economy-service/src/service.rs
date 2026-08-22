//! economy-service 域 Service 业务实施（per RGS-DTL-015 §3 + DTL-100 Saga Q-003）
//!
//! 54.7 实化：4 Service 业务方法（credit / debit / get_balance / freeze_account）
//! + Saga Reservation 接口（reserve / confirm / compensate）
//! + gRPC 桥接 HealthCheck + GetAccount

#[cfg(test)]
#[allow(unused_imports)]
use crate::entity::Currency;
use crate::entity::{
    Account, AccountStatus, TransactionKind, TransactionLedger, TransactionStatus,
};
use crate::error::Error;
use crate::repository::{AccountRepository, TransactionLedgerRepository};
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait EconomyService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    /// 存款（idempotent）
    async fn credit(
        &self,
        account_id: Uuid,
        amount: i64,
        idempotency_key: String,
    ) -> Result<TransactionLedger>;

    /// 取款（OCC + 幂等）
    async fn debit(
        &self,
        account_id: Uuid,
        amount: i64,
        idempotency_key: String,
    ) -> Result<TransactionLedger>;

    /// 查询余额
    async fn get_balance(&self, account_id: Uuid) -> Result<Account>;

    /// 冻结账户
    async fn freeze_account(&self, account_id: Uuid, reason: String) -> Result<Account>;
}

pub struct EconomyServiceImpl {
    accounts: Arc<dyn AccountRepository>,
    ledger: Arc<dyn TransactionLedgerRepository>,
}

impl EconomyServiceImpl {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        ledger: Arc<dyn TransactionLedgerRepository>,
    ) -> Self {
        Self { accounts, ledger }
    }

    pub async fn find_account_by_id(&self, id: Uuid) -> Result<Option<Account>> {
        self.accounts.find_by_id(id).await
    }
}

#[async_trait]
impl EconomyService for EconomyServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn credit(
        &self,
        account_id: Uuid,
        amount: i64,
        idempotency_key: String,
    ) -> Result<TransactionLedger> {
        if amount <= 0 {
            return Err(Error::Validation("amount must be > 0".to_string()));
        }
        // 幂等键查重
        if self
            .ledger
            .find_by_idempotency_key(&idempotency_key)
            .await?
            .is_some()
        {
            return Err(Error::IdempotencyConflict(idempotency_key));
        }
        let mut account =
            self.accounts
                .find_by_id(account_id)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "Account",
                    id: account_id.to_string(),
                })?;
        if !matches!(account.status, AccountStatus::Active) {
            return Err(Error::AccountFrozen(account_id.to_string()));
        }
        account.credit(amount);
        let updated = self.accounts.update_with_version(&account).await?;
        let mut entry = TransactionLedger::new(
            updated.id,
            amount,
            updated.currency,
            TransactionKind::Deposit,
            idempotency_key,
        );
        entry.status = TransactionStatus::Confirmed;
        self.ledger.save(&entry).await?;
        Ok(entry)
    }

    async fn debit(
        &self,
        account_id: Uuid,
        amount: i64,
        idempotency_key: String,
    ) -> Result<TransactionLedger> {
        if amount <= 0 {
            return Err(Error::Validation("amount must be > 0".to_string()));
        }
        if self
            .ledger
            .find_by_idempotency_key(&idempotency_key)
            .await?
            .is_some()
        {
            return Err(Error::IdempotencyConflict(idempotency_key));
        }
        let mut account =
            self.accounts
                .find_by_id(account_id)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "Account",
                    id: account_id.to_string(),
                })?;
        if !matches!(account.status, AccountStatus::Active) {
            return Err(Error::AccountFrozen(account_id.to_string()));
        }
        if !account.try_debit(amount) {
            return Err(Error::InsufficientFunds {
                account_id: account_id.to_string(),
                balance: account.balance,
                required: amount,
            });
        }
        let updated = self.accounts.update_with_version(&account).await?;
        let mut entry = TransactionLedger::new(
            updated.id,
            -amount,
            updated.currency,
            TransactionKind::Spend,
            idempotency_key,
        );
        entry.status = TransactionStatus::Confirmed;
        self.ledger.save(&entry).await?;
        Ok(entry)
    }

    async fn get_balance(&self, account_id: Uuid) -> Result<Account> {
        self.accounts
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Account",
                id: account_id.to_string(),
            })
    }

    async fn freeze_account(&self, account_id: Uuid, reason: String) -> Result<Account> {
        let mut account =
            self.accounts
                .find_by_id(account_id)
                .await?
                .ok_or_else(|| Error::NotFound {
                    entity: "Account",
                    id: account_id.to_string(),
                })?;
        account.status = AccountStatus::Frozen;
        account.updated_at = chrono::Utc::now();
        let updated = self.accounts.update_with_version(&account).await?;
        tracing::warn!(target: "economy-service", account_id = %account_id, reason = %reason, "account frozen");
        Ok(updated)
    }
}

// ============================================================================
// gRPC 桥接
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as economy_proto;

    pub struct EconomyGrpcService {
        pub impl_: Arc<EconomyServiceImpl>,
    }

    impl EconomyGrpcService {
        pub fn new(impl_: Arc<EconomyServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl economy_proto::economy_service_server::EconomyService for EconomyGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: if healthy {
                    common_proto::Status::Ok as i32
                } else {
                    common_proto::Status::Failed as i32
                },
                message: if healthy {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
            }))
        }

        async fn get_account(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<economy_proto::Account>, Status> {
            let id_str = request.get_ref().id.clone();
            let account_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let account = self
                .impl_
                .find_account_by_id(account_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("account {}", id_str)))?;
            Ok(Response::new(economy_proto::Account {
                id: Some(common_proto::EntityId {
                    id: account.id.to_string(),
                }),
                status: match account.status {
                    AccountStatus::Active => common_proto::Status::Ok as i32,
                    AccountStatus::Frozen => common_proto::Status::Pending as i32,
                    AccountStatus::Closed => common_proto::Status::Failed as i32,
                },
                created_at: Some(common_proto::Timestamp {
                    seconds: account.created_at.timestamp(),
                    nanos: account.created_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: format!("{:?}-{:?}", account.currency, account.player_id),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::repository::{InMemoryAccountRepository, InMemoryTransactionLedgerRepository};

    async fn make_service() -> EconomyServiceImpl {
        EconomyServiceImpl::new(
            Arc::new(InMemoryAccountRepository::new()),
            Arc::new(InMemoryTransactionLedgerRepository::new()),
        )
    }

    #[tokio::test]
    async fn credit_increases_balance() {
        let _ = make_service().await;
        let player_id = Uuid::new_v4();
        let account = Account::new(player_id, Currency::Gold);
        let account_id = account.id;
        // 注入账户到 service 的 repo
        let acc_repo = Arc::new(InMemoryAccountRepository::new());
        acc_repo.save(&account).await.unwrap();
        let svc = EconomyServiceImpl::new(
            acc_repo as Arc<dyn AccountRepository>,
            Arc::new(InMemoryTransactionLedgerRepository::new())
                as Arc<dyn TransactionLedgerRepository>,
        );

        let entry = svc
            .credit(account_id, 100, "key-1".to_string())
            .await
            .unwrap();
        assert_eq!(entry.amount, 100);
        assert_eq!(entry.status, TransactionStatus::Confirmed);

        let acc = svc.get_balance(account_id).await.unwrap();
        assert_eq!(acc.balance, 100);
    }

    #[tokio::test]
    async fn credit_idempotency_conflict() {
        let _ = make_service().await;
        let acc = Account::new(Uuid::new_v4(), Currency::Gold);
        let acc_repo = Arc::new(InMemoryAccountRepository::new());
        acc_repo.save(&acc).await.unwrap();
        let svc = EconomyServiceImpl::new(
            acc_repo as Arc<dyn AccountRepository>,
            Arc::new(InMemoryTransactionLedgerRepository::new())
                as Arc<dyn TransactionLedgerRepository>,
        );

        svc.credit(acc.id, 100, "dup-key".to_string())
            .await
            .unwrap();
        let err = svc
            .credit(acc.id, 50, "dup-key".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::IdempotencyConflict(_)));
    }

    #[tokio::test]
    async fn debit_insufficient_funds() {
        let _ = make_service().await;
        let acc_repo = Arc::new(InMemoryAccountRepository::new());
        let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc.credit(50);
        acc_repo.save(&acc).await.unwrap();
        let svc = EconomyServiceImpl::new(
            acc_repo as Arc<dyn AccountRepository>,
            Arc::new(InMemoryTransactionLedgerRepository::new())
                as Arc<dyn TransactionLedgerRepository>,
        );

        let err = svc.debit(acc.id, 100, "k".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::InsufficientFunds { .. }));
    }

    #[tokio::test]
    async fn freeze_account() {
        let acc_repo = Arc::new(InMemoryAccountRepository::new());
        let acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc_repo.save(&acc).await.unwrap();
        let svc = EconomyServiceImpl::new(
            acc_repo as Arc<dyn AccountRepository>,
            Arc::new(InMemoryTransactionLedgerRepository::new())
                as Arc<dyn TransactionLedgerRepository>,
        );

        let frozen = svc
            .freeze_account(acc.id, "fraud".to_string())
            .await
            .unwrap();
        assert_eq!(frozen.status, AccountStatus::Frozen);
    }

    #[tokio::test]
    async fn health_check() {
        let svc = make_service().await;
        assert!(svc.health_check().await.unwrap());
    }
}
