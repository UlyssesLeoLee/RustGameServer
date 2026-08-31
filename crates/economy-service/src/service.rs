//! economy-service 域 Service 业务实施（per RGS-DTL-015 §3 + DTL-100 Saga Q-003）
//!
//! 54.7 实化：4 Service 业务方法（credit / debit / get_balance / freeze_account）
//! + Saga Reservation 接口（reserve / confirm / compensate）
//! + gRPC 桥接 HealthCheck + GetAccount
//!
//! 55.12 实化（per RGS-REV-007 AC4 / DEC-015 P1）：
//! + `apply_atomic_with_reservation` 内部 helper —— 给 SagaOrchestrator 的
//!   ReserveHandler/ConfirmHandler 用，集中封装「持久化 reservation + 原子账户更新 +
//!   写账目」三步语义。

use crate::entity::{
    Account, AccountStatus, Currency, TransactionKind, TransactionLedger, TransactionStatus,
};
use crate::error::Error;
use crate::repository::{AccountRepository, TransactionLedgerRepository};
use crate::reservation::{Reservation, ReservationRepository};
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait EconomyService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    /// 存款（idempotent）
    ///
    /// **Deprecated**: 新代码应走 saga 路径,通过 `EconomyServiceImpl::apply_atomic_with_reservation`
    /// helper 完成,该 helper 实现完整的 reservation cleanup 防止 dangling reservation 堆积。
    /// 详见 RGS-REV-009 V3 M-1。
    #[deprecated(
        note = "考虑用 apply_atomic_with_reservation 走 saga 路径, 该 helper 实现完整 reservation cleanup 防止 dangling reservation. 详情见 RGS-REV-009 V3 M-1."
    )]
    async fn credit(
        &self,
        account_id: Uuid,
        amount: i64,
        idempotency_key: String,
    ) -> Result<TransactionLedger>;

    /// 取款（OCC + 幂等）
    ///
    /// **Deprecated**: 新代码应走 saga 路径,通过 `EconomyServiceImpl::apply_atomic_with_reservation`
    /// helper 完成,该 helper 实现完整的 reservation cleanup 防止 dangling reservation 堆积。
    /// 详见 RGS-REV-009 V3 M-1。
    #[deprecated(
        note = "考虑用 apply_atomic_with_reservation 走 saga 路径, 该 helper 实现完整 reservation cleanup 防止 dangling reservation. 详情见 RGS-REV-009 V3 M-1."
    )]
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

    /// Saga step handler 内部 helper（per RGS-REV-007 AC4 / DEC-015 P1 / RGS-REV-008 CC-4）：
    /// 一次性完成「持久化 reservation + 扣减余额 + 原子账户 OCC 更新 + 写账目」四步。
    ///
    /// 业务语义：handler 调它做 ReserveHandler.execute / ConfirmHandler.execute 的核心动作。
    /// 失败语义：调用方负责根据返回的 Error 决定是否触发补偿（confirm 失败 → reserve 补偿）。
    ///
    /// 顺序保证：
    /// 1. 先 reservation.save —— 失败时不进入下一步
    /// 2. 再 try_debit —— 余额不足时返回 InsufficientFunds（不写账目），
    ///    并**清理已持久化的 reservation**（per RGS-REV-008 CC-4 / verify-C 修复）
    /// 3. 再 apply_atomic —— OCC + 账目原子更新（per AC3 修复）；
    ///    失败时**清理已持久化的 reservation**（per RGS-REV-008 CC-4 / verify-C 修复），
    ///    避免 dangling reservation 堆积
    /// 4. happy path：返 (updated_account, saved_entry, saved_reservation) 三元组
    ///
    /// 返回 (更新后 Account, 写后 Ledger, 持久化后 Reservation)。
    #[allow(clippy::too_many_arguments)]
    pub async fn apply_atomic_with_reservation(
        &self,
        account: &Account,
        amount: i64,
        currency: Currency,
        kind: TransactionKind,
        saga_id: Uuid,
        command_id: Uuid,
        idempotency_key: String,
        reservations: &Arc<dyn ReservationRepository>,
    ) -> Result<(Account, TransactionLedger, Reservation)> {
        if amount <= 0 {
            return Err(Error::Validation("amount must be > 0".to_string()));
        }
        // 1. 构造并持久化 reservation
        let reservation = Reservation::new(saga_id, account.id, amount, currency);
        let saved_reservation = reservations.save(&reservation).await?;

        // 2. 扣减余额（OCC 前本地校验）
        let mut debited = account.clone();
        if !debited.try_debit(amount) {
            // 补偿：清理 dangling reservation（per RGS-REV-008 CC-4 / verify-C 修复）
            // try_debit 失败时本地扣减未发生，但 reservation 已落库。
            // 必须 delete_by_id 防止表堆积 + 避免后续 compensate 误触发 +amount 退款。
            if let Err(cleanup_err) = reservations.delete_by_id(saved_reservation.id).await {
                tracing::warn!(
                    target: "economy-service",
                    reservation_id = %saved_reservation.id,
                    saga_id = %saga_id,
                    account_id = %account.id,
                    "failed to cleanup dangling reservation after insufficient funds: {}",
                    cleanup_err
                );
            }
            return Err(Error::InsufficientFunds {
                account_id: account.id.to_string(),
                balance: account.balance,
                required: amount,
            });
        }

        // 3. 原子 OCC 更新账户 + 写账目
        let mut entry =
            TransactionLedger::new(account.id, -amount, currency, kind, idempotency_key);
        entry.saga_id = Some(saga_id);
        entry.command_id = Some(command_id);
        entry.status = TransactionStatus::Confirmed;
        // 分离 Ok/Err：失败时也清理 reservation（per RGS-REV-008 CC-4 / verify-C 修复）
        // 防止 apply_atomic OCC 冲突 / DB 异常路径下 dangling reservation 堆积，
        // 避免后续 reserve 补偿误触发（per RGS-REV-008 CC-4 资金幻影风险的核心 patch）。
        let apply_result = self.accounts.apply_atomic(&debited, &entry).await;
        let (updated_account, saved_entry) = match apply_result {
            Ok(v) => v,
            Err(e) => {
                if let Err(cleanup_err) = reservations.delete_by_id(saved_reservation.id).await {
                    tracing::warn!(
                        target: "economy-service",
                        reservation_id = %saved_reservation.id,
                        saga_id = %saga_id,
                        account_id = %account.id,
                        "failed to cleanup dangling reservation after apply_atomic failure: {}",
                        cleanup_err
                    );
                }
                return Err(e);
            }
        };

        Ok((updated_account, saved_entry, saved_reservation))
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
        // 原子事务：OCC 更新账户 + 写入账目（per RGS-REV-007 AC3 修复）
        let mut entry = TransactionLedger::new(
            account.id,
            amount,
            account.currency,
            TransactionKind::Deposit,
            idempotency_key,
        );
        entry.status = TransactionStatus::Confirmed;
        let (_, saved_entry) = self.accounts.apply_atomic(&account, &entry).await?;
        Ok(saved_entry)
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
        // 原子事务：OCC 更新账户 + 写入账目（per RGS-REV-007 AC3 修复）
        let mut entry = TransactionLedger::new(
            account.id,
            -amount,
            account.currency,
            TransactionKind::Spend,
            idempotency_key,
        );
        entry.status = TransactionStatus::Confirmed;
        let (_, saved_entry) = self.accounts.apply_atomic(&account, &entry).await?;
        Ok(saved_entry)
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
    use crate::trade_entity::AuctionFilter;
    use crate::trade_service::{TradeService, TradeServiceImpl};

    pub struct EconomyGrpcService {
        pub impl_: Arc<EconomyServiceImpl>,
        pub trade: Arc<TradeServiceImpl>,
    }

    impl EconomyGrpcService {
        pub fn new(impl_: Arc<EconomyServiceImpl>, trade: Arc<TradeServiceImpl>) -> Self {
            Self { impl_, trade }
        }
    }

    /// Auction → proto.Auction 转换
    fn auction_to_proto(a: &crate::trade_entity::Auction) -> economy_proto::Auction {
        economy_proto::Auction {
            auction_id: a.auction_id.to_string(),
            seller_id: a.seller_id.clone(),
            card_id: a.card_id.clone(),
            card_instance_id: a.card_instance_id.clone(),
            min_price: a.min_price,
            currency_type: a.currency_type,
            highest_bid: a.highest_bid,
            highest_bidder: a.highest_bidder.clone(),
            status: a.status.as_i32(),
            started_at: Some(common_proto::Timestamp {
                seconds: a.started_at.timestamp(),
                nanos: a.started_at.timestamp_subsec_nanos() as i32,
            }),
            ends_at: Some(common_proto::Timestamp {
                seconds: a.ends_at.timestamp(),
                nanos: a.ends_at.timestamp_subsec_nanos() as i32,
            }),
            closed_at: a.closed_at.map(|t| common_proto::Timestamp {
                seconds: t.timestamp(),
                nanos: t.timestamp_subsec_nanos() as i32,
            }),
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

        // ========== trade 域 5 RPC (per RGS-DTL-038 §4.4 + DEC-038-04) ==========
        // 委托给 self.trade (TradeServiceImpl) 业务实现
        async fn create_auction(
            &self,
            request: Request<economy_proto::CreateAuctionRequest>,
        ) -> std::result::Result<Response<economy_proto::CreateAuctionResponse>, Status> {
            let req = request.into_inner();
            let auction = self
                .trade
                .create_auction(
                    req.seller_id,
                    req.card_id,
                    req.card_instance_id,
                    req.min_price,
                    req.currency_type,
                    req.ends_at_unix,
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(economy_proto::CreateAuctionResponse {
                auction_id: auction.auction_id.to_string(),
                started_at: Some(common_proto::Timestamp {
                    seconds: auction.started_at.timestamp(),
                    nanos: auction.started_at.timestamp_subsec_nanos() as i32,
                }),
                ends_at: Some(common_proto::Timestamp {
                    seconds: auction.ends_at.timestamp(),
                    nanos: auction.ends_at.timestamp_subsec_nanos() as i32,
                }),
            }))
        }

        async fn bid_auction(
            &self,
            request: Request<economy_proto::BidAuctionRequest>,
        ) -> std::result::Result<Response<economy_proto::BidAuctionResponse>, Status> {
            let req = request.into_inner();
            let auction_id = Uuid::parse_str(&req.auction_id).map_err(|_| {
                Status::invalid_argument(format!("invalid auction_id: {}", req.auction_id))
            })?;
            let result = self
                .trade
                .bid_auction(auction_id, req.bidder_id, req.amount, req.idempotency_key)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(economy_proto::BidAuctionResponse {
                bid_id: result.bid_id.to_string(),
                is_highest: result.is_highest,
                current_highest: result.auction.highest_bid,
                auction_ended: result.auction_ended,
                refunded_to: result.refunded_to,
                refund_amount: result.refund_amount,
            }))
        }

        async fn cancel_auction(
            &self,
            request: Request<economy_proto::CancelAuctionRequest>,
        ) -> std::result::Result<Response<economy_proto::CancelAuctionResponse>, Status> {
            let req = request.into_inner();
            let auction_id = Uuid::parse_str(&req.auction_id).map_err(|_| {
                Status::invalid_argument(format!("invalid auction_id: {}", req.auction_id))
            })?;
            let result = self
                .trade
                .cancel_auction(auction_id, req.seller_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(economy_proto::CancelAuctionResponse {
                cancelled: true,
                refunded: result.refunded,
                refunded_to: result.refunded_to,
            }))
        }

        async fn list_auction(
            &self,
            request: Request<economy_proto::ListAuctionRequest>,
        ) -> std::result::Result<Response<economy_proto::ListAuctionResponse>, Status> {
            let req = request.into_inner();
            let filter = AuctionFilter::from_i32(req.filter);
            let page_req = req.page.unwrap_or_default();
            let (list, total) = self
                .trade
                .list_auctions(filter, page_req.page, page_req.page_size)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let page_size = if page_req.page_size == 0 { 20 } else { page_req.page_size };
            let has_next = (page_req.page as u64) * (page_size as u64) < total;
            Ok(Response::new(economy_proto::ListAuctionResponse {
                auctions: list.iter().map(auction_to_proto).collect(),
                page: Some(common_proto::PageResponse {
                    total: total as u32,
                    has_next,
                    next_cursor: String::new(),
                }),
            }))
        }

        async fn get_trade_history(
            &self,
            request: Request<economy_proto::GetTradeHistoryRequest>,
        ) -> std::result::Result<Response<economy_proto::GetTradeHistoryResponse>, Status> {
            let req = request.into_inner();
            let page_req = req.page.unwrap_or_default();
            let (list, total) = self
                .trade
                .get_trade_history(req.player_id, page_req.page, page_req.page_size)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let page_size = if page_req.page_size == 0 { 20 } else { page_req.page_size };
            let has_next = (page_req.page as u64) * (page_size as u64) < total;
            Ok(Response::new(economy_proto::GetTradeHistoryResponse {
                trades: list.iter().map(auction_to_proto).collect(),
                page: Some(common_proto::PageResponse {
                    total: total as u32,
                    has_next,
                    next_cursor: String::new(),
                }),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    // 内部测试直接调用 deprecated 的 EconomyService::credit/debit 是合理的
    // (这些 trait method 仍被现有测试覆盖, migration 到 saga 路径由后续 task 处理)
    #![allow(deprecated)]

    #[allow(unused_imports)]
    use super::*;
    use crate::repository::{InMemoryAccountRepository, InMemoryTransactionLedgerRepository};
    use crate::reservation::{InMemoryReservationRepository, ReservationRepository};

    /// 构造带共享 ledger 的 service（per RGS-REV-007 AC3 修复）
    /// 共享 ledger HashMap 让 apply_atomic 可原子写两侧
    fn make_service_paired() -> (
        EconomyServiceImpl,
        Arc<InMemoryAccountRepository>,
        Arc<InMemoryTransactionLedgerRepository>,
    ) {
        let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
        let acc_repo =
            Arc::new(InMemoryAccountRepository::new().with_shared_ledger(led_repo.inner.clone()));
        let svc = EconomyServiceImpl::new(
            acc_repo.clone() as Arc<dyn AccountRepository>,
            led_repo.clone() as Arc<dyn TransactionLedgerRepository>,
        );
        (svc, acc_repo, led_repo)
    }

    #[tokio::test]
    async fn credit_increases_balance() {
        let (svc, acc_repo, _led_repo) = make_service_paired();
        let player_id = Uuid::new_v4();
        let account = Account::new(player_id, Currency::Gold);
        let account_id = account.id;
        acc_repo.save(&account).await.unwrap();

        let entry = svc
            .credit(account_id, 100, "key-1".to_string())
            .await
            .unwrap();
        assert_eq!(entry.amount, 100);
        assert_eq!(entry.status, TransactionStatus::Confirmed);

        let acc = svc.get_balance(account_id).await.unwrap();
        assert_eq!(acc.balance, 100);
        // per RGS-REV-007 AC3: ledger 同步写入（apply_atomic 原子性）
        assert_eq!(
            acc_repo
                .inner
                .lock()
                .unwrap()
                .get(&account_id)
                .unwrap()
                .balance,
            100
        );
    }

    #[tokio::test]
    async fn credit_idempotency_conflict() {
        let (svc, acc_repo, _led_repo) = make_service_paired();
        let acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc_repo.save(&acc).await.unwrap();

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
        let (svc, acc_repo, _led_repo) = make_service_paired();
        let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc.credit(50);
        acc_repo.save(&acc).await.unwrap();

        let err = svc.debit(acc.id, 100, "k".to_string()).await.unwrap_err();
        assert!(matches!(err, Error::InsufficientFunds { .. }));
    }

    #[tokio::test]
    async fn debit_atomic_balance_and_ledger() {
        // 验证 RGS-REV-007 AC3 修复：debit 成功时 balance 和 ledger 同步更新
        let (svc, acc_repo, led_repo) = make_service_paired();
        let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc.credit(100);
        acc_repo.save(&acc).await.unwrap();

        let entry = svc.debit(acc.id, 30, "spend-1".to_string()).await.unwrap();
        assert_eq!(entry.amount, -30);
        assert_eq!(entry.status, TransactionStatus::Confirmed);

        let balance = svc.get_balance(acc.id).await.unwrap().balance;
        assert_eq!(balance, 70, "balance should be 100 - 30 = 70");

        // ledger 同步写入（apply_atomic 原子性）
        let led_count = led_repo.inner.lock().unwrap().len();
        assert_eq!(led_count, 1, "ledger should have exactly 1 entry");
    }

    #[tokio::test]
    async fn freeze_account() {
        let (svc, acc_repo, _led_repo) = make_service_paired();
        let acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc_repo.save(&acc).await.unwrap();

        let frozen = svc
            .freeze_account(acc.id, "fraud".to_string())
            .await
            .unwrap();
        assert_eq!(frozen.status, AccountStatus::Frozen);
    }

    #[tokio::test]
    async fn health_check() {
        let (svc, _acc_repo, _led_repo) = make_service_paired();
        assert!(svc.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn apply_atomic_with_reservation_persists_all_three() {
        // 验证 55.12 新 helper：reservations + accounts.apply_atomic 一并完成
        let (svc, acc_repo, led_repo) = make_service_paired();
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

        let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc.credit(500);
        let account_id = acc.id;
        acc_repo.save(&acc).await.unwrap();

        let saga_id = Uuid::new_v4();
        let cmd_id = Uuid::new_v4();
        let (updated, entry, reservation) = svc
            .apply_atomic_with_reservation(
                &acc,
                100,
                Currency::Gold,
                TransactionKind::Transfer,
                saga_id,
                cmd_id,
                "k-saga-1".to_string(),
                &res_repo_dyn,
            )
            .await
            .unwrap();

        // 1. account: balance = 500 - 100 = 400, version + 1
        assert_eq!(updated.balance, 400);
        assert_eq!(updated.version, acc.version + 1);

        // 2. ledger: 1 entry, -100, saga_id 关联
        assert_eq!(entry.amount, -100);
        assert_eq!(entry.saga_id, Some(saga_id));
        assert_eq!(entry.command_id, Some(cmd_id));
        assert_eq!(entry.status, TransactionStatus::Confirmed);
        assert_eq!(led_repo.inner.lock().unwrap().len(), 1);

        // 3. reservation: 已持久化，saga_id + account_id 正确
        let from_repo = res_repo
            .find_by_id(reservation.id)
            .await
            .unwrap()
            .expect("reservation persisted");
        assert_eq!(from_repo.saga_id, saga_id);
        assert_eq!(from_repo.account_id, account_id);
        assert_eq!(from_repo.amount, 100);
        assert_eq!(from_repo.currency, Currency::Gold);

        // 4. account 也确实被 OCC 更新了
        let reloaded = acc_repo.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(reloaded.balance, 400);
    }

    #[tokio::test]
    async fn apply_atomic_with_reservation_rejects_non_positive_amount() {
        let (svc, acc_repo, _led_repo) = make_service_paired();
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();
        let acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc_repo.save(&acc).await.unwrap();

        let err = svc
            .apply_atomic_with_reservation(
                &acc,
                0,
                Currency::Gold,
                TransactionKind::Transfer,
                Uuid::new_v4(),
                Uuid::new_v4(),
                "k-zero".to_string(),
                &res_repo_dyn,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn apply_atomic_with_reservation_insufficient_funds_cleans_reservation() {
        // 验证 RGS-REV-008 CC-4 / verify-C 修复：
        // try_debit 失败时, 已持久化的 reservation 必须被 delete_by_id 清理,
        // 避免 dangling reservation 堆积 + 误触发后续 compensate 退款。
        let (svc, acc_repo, led_repo) = make_service_paired();
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

        let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc.credit(50); // 余额 50
        let account_id = acc.id;
        acc_repo.save(&acc).await.unwrap();

        let saga_id = Uuid::new_v4();
        let cmd_id = Uuid::new_v4();

        // 预写入 1 个 dummy reservation 用不同 saga_id, 用于验证 cleanup 不影响其它记录
        let dummy = Reservation::new(Uuid::new_v4(), account_id, 1, Currency::Gold);
        let dummy_id = dummy.id;
        res_repo.save(&dummy).await.unwrap();

        // 触发 try_debit 失败路径：扣 100 > 余额 50
        let err = svc
            .apply_atomic_with_reservation(
                &acc,
                100,
                Currency::Gold,
                TransactionKind::Transfer,
                saga_id,
                cmd_id,
                "k-isf".to_string(),
                &res_repo_dyn,
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::InsufficientFunds { .. }),
            "expected InsufficientFunds, got {:?}",
            err
        );

        // 关键断言：apply_atomic_with_reservation 内部产生的 saved_reservation
        // 已被 cleanup 清理。list_by_saga(saga_id) 应返回 0 条。
        let for_saga = res_repo.list_by_saga(saga_id).await.unwrap();
        assert_eq!(
            for_saga.len(),
            0,
            "dangling reservation must be cleaned up after insufficient funds; got {:?}",
            for_saga
        );

        // dummy 未受影响
        let still_dummy = res_repo.find_by_id(dummy_id).await.unwrap();
        assert!(
            still_dummy.is_some(),
            "dummy reservation should be untouched"
        );

        // 账户余额未变（未被扣）
        let reloaded = acc_repo.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(reloaded.balance, 50);

        // ledger 无任何条目
        assert_eq!(led_repo.inner.lock().unwrap().len(), 0);

        // account version 未变 (无 apply_atomic 调用)
        assert_eq!(reloaded.version, acc.version);
    }

    #[tokio::test]
    async fn apply_atomic_with_reservation_occ_conflict_cleans_reservation() {
        // 验证 RGS-REV-008 CC-4 / verify-C 修复：
        // apply_atomic OCC 失败时, 已持久化的 reservation 必须被 delete_by_id 清理,
        // 防止 dangling reservation 堆积 + 后续 compensate 误触发凭空 +amount 退款。
        let (svc, acc_repo, led_repo) = make_service_paired();
        let res_repo = Arc::new(InMemoryReservationRepository::new());
        let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

        let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
        acc.credit(500); // 余额足够, 走 try_debit 成功路径
        let account_id = acc.id;
        let original_version = acc.version;
        acc_repo.save(&acc).await.unwrap();

        // 模拟 OCC 冲突：直接修改 acc_repo 里的 account.version, 让传入的 acc (旧 version) 触发冲突。
        {
            let mut guard = acc_repo.inner.lock().unwrap();
            let stored = guard.get_mut(&account_id).expect("account saved");
            stored.version = original_version + 99; // bump version -> 传入的 acc 旧 version 必冲突
        }

        // 预写入 1 个 dummy reservation 用不同 saga_id, 用于验证 cleanup 不影响其它记录
        let dummy = Reservation::new(Uuid::new_v4(), account_id, 1, Currency::Gold);
        let dummy_id = dummy.id;
        res_repo.save(&dummy).await.unwrap();

        let saga_id = Uuid::new_v4();
        let cmd_id = Uuid::new_v4();

        // 触发 apply_atomic 失败路径 (OCC 冲突)
        let err = svc
            .apply_atomic_with_reservation(
                &acc,
                100,
                Currency::Gold,
                TransactionKind::Transfer,
                saga_id,
                cmd_id,
                "k-occ".to_string(),
                &res_repo_dyn,
            )
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::Validation(ref msg) if msg.contains("OCC conflict")),
            "expected OCC conflict Validation error, got {:?}",
            err
        );

        // 关键断言：dangling reservation 已被清理。list_by_saga(saga_id) 应返回 0 条。
        let for_saga = res_repo.list_by_saga(saga_id).await.unwrap();
        assert_eq!(
            for_saga.len(),
            0,
            "dangling reservation must be cleaned up after apply_atomic OCC failure; got {:?}",
            for_saga
        );

        // dummy 未受影响
        let still_dummy = res_repo.find_by_id(dummy_id).await.unwrap();
        assert!(
            still_dummy.is_some(),
            "dummy reservation should be untouched"
        );

        // ledger 无任何条目（apply_atomic 失败, ledger INSERT 未发生）
        assert_eq!(led_repo.inner.lock().unwrap().len(), 0);

        // 账户余额未变（apply_atomic 失败回滚）
        let reloaded = acc_repo.find_by_id(account_id).await.unwrap().unwrap();
        assert_eq!(reloaded.balance, 500);
        // version 是我们之前 bump 后的值（apply_atomic 失败未影响）
        assert_eq!(reloaded.version, original_version + 99);
    }

    // ========================================================================
    // Proptest (RGS-UT 2026-08-31 JST) — apply_atomic_with_reservation 守恒
    // ========================================================================

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        /// apply_atomic_with_reservation happy path 余额守恒:
        /// 任意 (initial, amount) 组合下, 余额足够时, helper 成功后
        /// 账户余额 = initial - amount, reservation 1 条, ledger 1 条.
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(256))]

            #[test]
            fn apply_atomic_with_reservation_conservation(
                initial in 100i64..100_000,
                amount in 1i64..5_000,
            ) {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let amount = amount.min(initial);
                    let (svc, acc_repo, led_repo) = make_service_paired();
                    let res_repo = Arc::new(InMemoryReservationRepository::new());
                    let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

                    let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
                    acc.credit(initial);
                    let account_id = acc.id;
                    acc_repo.save(&acc).await.unwrap();

                    let saga_id = Uuid::new_v4();
                    let cmd_id = Uuid::new_v4();
                    let key = format!("k-prop-{}-{}", initial, amount);
                    let (updated, entry, reservation) = svc
                        .apply_atomic_with_reservation(
                            &acc,
                            amount,
                            Currency::Gold,
                            TransactionKind::Transfer,
                            saga_id,
                            cmd_id,
                            key,
                            &res_repo_dyn,
                        )
                        .await
                        .unwrap();

                    prop_assert_eq!(updated.balance, initial - amount);
                    prop_assert_eq!(entry.amount, -amount);
                    prop_assert_eq!(reservation.amount, amount);
                    prop_assert_eq!(led_repo.inner.lock().unwrap().len(), 1);
                    let reloaded = acc_repo.find_by_id(account_id).await.unwrap().unwrap();
                    prop_assert_eq!(reloaded.balance, initial - amount);
                    Ok(())
                });
            }
        }

        /// apply_atomic_with_reservation 余额不足时不变量:
        /// 任意 (initial, amount > initial) 组合下, helper 返 InsufficientFunds,
        /// 账户余额不变, reservation 已清理 (无 dangling).
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(256))]

            #[test]
            fn apply_atomic_with_reservation_insufficient_no_dangle(
                initial in 0i64..10_000,
                extra in 1i64..50_000,
            ) {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let (svc, acc_repo, _led_repo) = make_service_paired();
                    let res_repo = Arc::new(InMemoryReservationRepository::new());
                    let res_repo_dyn: Arc<dyn ReservationRepository> = res_repo.clone();

                    let mut acc = Account::new(Uuid::new_v4(), Currency::Gold);
                    acc.credit(initial);
                    let account_id = acc.id;
                    acc_repo.save(&acc).await.unwrap();

                    let amount = initial + extra; // 必超余额
                    let saga_id = Uuid::new_v4();
                    let cmd_id = Uuid::new_v4();
                    let key = format!("k-prop-isf-{}-{}", initial, amount);
                    let err = svc
                        .apply_atomic_with_reservation(
                            &acc,
                            amount,
                            Currency::Gold,
                            TransactionKind::Transfer,
                            saga_id,
                            cmd_id,
                            key,
                            &res_repo_dyn,
                        )
                        .await
                        .unwrap_err();
                    prop_assert!(matches!(err, Error::InsufficientFunds { .. }), "expected InsufficientFunds");
                    // 关键: 无 dangling reservation
                    let for_saga = res_repo.list_by_saga(saga_id).await.unwrap();
                    prop_assert_eq!(for_saga.len(), 0,
                        "no dangling reservation should be left after InsufficientFunds");
                    // 余额不变
                    let reloaded = acc_repo.find_by_id(account_id).await.unwrap().unwrap();
                    prop_assert_eq!(reloaded.balance, initial);
                    Ok(())
                });
            }
        }
    }
}
