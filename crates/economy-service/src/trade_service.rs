//! economy-service trade 域 Service 业务实施 (per RGS-DTL-038 §4.4 + §6.2/§6.3 + DEC-038-04)
//!
//! 卡牌 8 桶 / 子桶 1: trade 域 5 RPC 业务实施.
//!
//! 业务范围:
//!   - CreateAuction: 卖家创建公开拍卖
//!   - BidAuction: 玩家出价 + 旧最高出价者补偿 + 拍卖到期自动成交 (per §6.2 Trade saga 4 步 + §6.3 ExecuteAuction saga)
//!   - CancelAuction: 卖家撤单 + 退还所有出价者
//!   - ListAuction: 公开拍卖列表 (active / closed / all filter)
//!   - GetTradeHistory: 玩家交易历史 (卖家 + 出价者 双视角)
//!   - ExecuteTrade: 私下交易 (TODO 跨域 saga, W36+ 接入; 当前占位 propose/cancel)
//!
//! 跨域 saga: 公开拍卖成交 (ExecuteAuction §6.3) 涉及 card-service 协同 (卡牌实例转移),
//!           私下交易 (ExecuteTrade) 涉及 card-service + economy-service 双向货币/卡牌转移.
//!           当前用 TODO 注释标记 W36+ 接入点, 业务实现保持单域内可运行 (mock 跨域).

use crate::entity::{Currency, TransactionKind, TransactionStatus};
use crate::error::Error;
use crate::repository::{AccountRepository, TransactionLedgerRepository};
use crate::trade_entity::{Auction, AuctionFilter, AuctionStatus, PrivateTrade, PrivateTradeStatus};
use crate::trade_repository::TradeRepository;
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// 默认拍卖时长 24h (per RGS-REQ-038 §FR-009)
pub const DEFAULT_AUCTION_DURATION_SECS: i64 = 86400;
/// 平台手续费 5% (per RGS-REQ-038 §FR-009 auction 撮合, future: per 平台规则)
pub const AUCTION_FEE_BPS: i64 = 500; // basis points, 5%

/// TradeService trait —— 5 RPC 业务接口
#[async_trait]
pub trait TradeService: Send + Sync {
    /// 卖家创建公开拍卖
    async fn create_auction(
        &self,
        seller_id: String,
        card_id: String,
        card_instance_id: String,
        min_price: i64,
        currency_type: i32,
        ends_at_unix: i64,
    ) -> Result<Auction>;

    /// 玩家出价（含旧最高出价者补偿 + 自动成交判定）
    async fn bid_auction(
        &self,
        auction_id: Uuid,
        bidder_id: String,
        amount: i64,
        idempotency_key: String,
    ) -> Result<BidResult>;

    /// 卖家撤单（含退还当前最高出价者）
    async fn cancel_auction(
        &self,
        auction_id: Uuid,
        seller_id: String,
    ) -> Result<CancelResult>;

    /// 公开拍卖列表
    async fn list_auctions(
        &self,
        filter: AuctionFilter,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)>;

    /// 玩家交易历史
    async fn get_trade_history(
        &self,
        player_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)>;
}

/// 出价业务结果
#[derive(Debug, Clone)]
pub struct BidResult {
    pub auction: Auction,
    pub bid_id: Uuid,
    pub is_highest: bool,
    pub auction_ended: bool,
    pub refunded_to: String,
    pub refund_amount: i64,
}

/// 撤单业务结果
#[derive(Debug, Clone)]
pub struct CancelResult {
    pub auction: Auction,
    pub refunded: i64,
    pub refunded_to: String,
}

/// TradeService 业务实现
pub struct TradeServiceImpl {
    trades: Arc<dyn TradeRepository>,
    /// 跨域 mock: 当前用 economy 内的 account 完成货币扣减 / 退还
    /// W36+ 接入 saga orchestrator 后, 此处改为调用 saga 步骤 (per §6.2 + §6.3)
    accounts: Arc<dyn AccountRepository>,
    ledger: Arc<dyn TransactionLedgerRepository>,
}

impl TradeServiceImpl {
    pub fn new(
        trades: Arc<dyn TradeRepository>,
        accounts: Arc<dyn AccountRepository>,
        ledger: Arc<dyn TransactionLedgerRepository>,
    ) -> Self {
        Self {
            trades,
            accounts,
            ledger,
        }
    }

    /// 货币类型转换: common.proto CurrencyType (1/2/3) → economy Currency
    fn parse_currency(currency_type: i32) -> Result<Currency> {
        match currency_type {
            1 => Ok(Currency::Gold),     // soft = gold
            2 => Ok(Currency::Diamond),  // hard = diamond
            3 => Ok(Currency::Token),    // card_value = token
            _ => Err(Error::Validation(format!(
                "unknown currency_type: {}",
                currency_type
            ))),
        }
    }

    /// 内部 helper: 退货币（per §6.2 step 2 失败补偿 / §6.3 step 2 失败补偿 / 撤单退款）
    async fn refund_currency(
        &self,
        player_id: &str,
        amount: i64,
        currency: Currency,
        idempotency_key: &str,
        memo: &str,
    ) -> Result<()> {
        if amount <= 0 {
            return Ok(());
        }
        // 查找玩家对应货币的账户
        let player_uuid = Uuid::parse_str(player_id)
            .map_err(|_| Error::Validation(format!("invalid player uuid: {}", player_id)))?;
        let account = self
            .accounts
            .find_by_player_and_currency(player_uuid, currency)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Account",
                id: format!("{}-{:?}", player_id, currency),
            })?;
        let mut updated = account.clone();
        updated.credit(amount);
        let mut entry = TransactionLedger::new(
            updated.id,
            amount,
            currency,
            TransactionKind::Refund,
            idempotency_key.to_string(),
        );
        entry.status = TransactionStatus::Confirmed;
        entry.memo = Some(memo.to_string());
        self.accounts.apply_atomic(&updated, &entry).await?;
        Ok(())
    }
}

#[async_trait]
impl TradeService for TradeServiceImpl {
    async fn create_auction(
        &self,
        seller_id: String,
        card_id: String,
        card_instance_id: String,
        min_price: i64,
        currency_type: i32,
        ends_at_unix: i64,
    ) -> Result<Auction> {
        // 输入校验
        if seller_id.is_empty() {
            return Err(Error::Validation("seller_id required".to_string()));
        }
        if card_id.is_empty() || card_instance_id.is_empty() {
            return Err(Error::Validation(
                "card_id and card_instance_id required".to_string(),
            ));
        }
        if min_price < 0 {
            return Err(Error::Validation("min_price must be >= 0".to_string()));
        }
        Self::parse_currency(currency_type)?;

        // 计算截止时间
        let ends_at = if ends_at_unix > 0 {
            chrono::DateTime::<chrono::Utc>::from_timestamp(ends_at_unix, 0)
                .ok_or_else(|| Error::Validation("invalid ends_at_unix".to_string()))?
        } else {
            chrono::Utc::now() + chrono::Duration::seconds(DEFAULT_AUCTION_DURATION_SECS)
        };
        if ends_at <= chrono::Utc::now() {
            return Err(Error::Validation("ends_at must be in future".to_string()));
        }

        let auction = Auction::new(
            seller_id.clone(),
            card_id,
            card_instance_id,
            min_price,
            currency_type,
            (ends_at - chrono::Utc::now()).num_seconds(),
        );
        let saved = self.trades.save_auction(&auction).await?;

        // TODO(W36+): 卡牌实例 escrowed 化 (跨域 saga step 1: card-service.MarkCardLocked)
        // 公开拍卖创建时, 卖家卡牌实例应被锁定不可用于其他交易.
        // 当前单域实现不处理, 留作跨域 saga 接入点.

        tracing::info!(
            target: "economy-service.trade",
            auction_id = %saved.auction_id,
            seller_id = %seller_id,
            "auction created"
        );
        Ok(saved)
    }

    async fn bid_auction(
        &self,
        auction_id: Uuid,
        bidder_id: String,
        amount: i64,
        idempotency_key: String,
    ) -> Result<BidResult> {
        // 输入校验
        if bidder_id.is_empty() {
            return Err(Error::Validation("bidder_id required".to_string()));
        }
        if amount <= 0 {
            return Err(Error::Validation("amount must be > 0".to_string()));
        }

        // 1. 加载拍卖
        let mut auction = self
            .trades
            .find_auction_by_id(auction_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Auction",
                id: auction_id.to_string(),
            })?;

        // 2. 业务规则校验（活跃 + 卖家不可自买 + 金额合法）
        auction
            .is_valid_bid(amount, &bidder_id)
            .map_err(|e| Error::Validation(e.to_string()))?;

        // 3. 旧最高出价者记录（用于补偿退款）
        let old_highest_bid = auction.highest_bid;
        let old_highest_bidder = auction.highest_bidder.clone();

        // 4. 幂等检查
        if self
            .ledger
            .find_by_idempotency_key(&idempotency_key)
            .await?
            .is_some()
        {
            return Err(Error::IdempotencyConflict(idempotency_key));
        }

        // 5. 扣减新出价者货币 + 写账目 (per §6.2 step 2: DebitCurrency)
        //    跨域 saga 化后, 此处调用 saga reserve 步骤, 当前单域用 apply_atomic 模拟
        let currency = Self::parse_currency(auction.currency_type)?;
        let bidder_uuid = Uuid::parse_str(&bidder_id)
            .map_err(|_| Error::Validation(format!("invalid bidder uuid: {}", bidder_id)))?;
        let bidder_account = self
            .accounts
            .find_by_player_and_currency(bidder_uuid, currency)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Account",
                id: format!("{}-{:?}", bidder_id, currency),
            })?;
        let mut debited = bidder_account.clone();
        if !debited.try_debit(amount) {
            return Err(Error::InsufficientFunds {
                account_id: bidder_account.id.to_string(),
                balance: bidder_account.balance,
                required: amount,
            });
        }
        let mut entry = TransactionLedger::new(
            debited.id,
            -amount,
            currency,
            TransactionKind::Spend,
            idempotency_key.clone(),
        );
        entry.status = TransactionStatus::Confirmed;
        entry.memo = Some(format!("bid on auction {}", auction_id));
        self.accounts.apply_atomic(&debited, &entry).await?;

        // 6. 更新 auction 最高出价 (per §6.2 step 3: UpdateHighestBid)
        let new_highest_bid = amount;
        let mut refunded_to = String::new();
        let mut refund_amount = 0i64;
        auction.highest_bid = new_highest_bid;
        auction.highest_bidder = bidder_id.clone();

        // 7. 退还旧最高出价者 (per §6.2 step 2 补偿 / §6.3 step 2 失败补偿)
        if !old_highest_bidder.is_empty() && old_highest_bid > 0 {
            let refund_key = format!("refund-{}-{}", auction_id, old_highest_bidder);
            self.refund_currency(
                &old_highest_bidder,
                old_highest_bid,
                currency,
                &refund_key,
                &format!("outbid on auction {}", auction_id),
            )
            .await?;
            refunded_to = old_highest_bidder;
            refund_amount = old_highest_bid;
        }

        // 8. 检查拍卖是否到期 (per §6.2 step 4: CheckAuctionEnded)
        let auction_ended = auction.is_expired();
        if auction_ended {
            auction.status = AuctionStatus::Sold;
            auction.winner_id = Some(bidder_id.clone());
            auction.final_price = amount;
            auction.closed_at = Some(chrono::Utc::now());
            // TODO(W36+): 触发 §6.3 ExecuteAuction saga
            //   1. trade-service.FinalizeAuction (此处已完成)
            //   2. economy-service.TransferCurrency (卖家收款)
            //   3. card-service.RemoveCardFromCollection
            //   4. card-service.AddCardToCollection
            //   5. economy-service.AddTransactionLog
            // 当前单域仅做 FinalizeAuction, 跨域步骤留 TODO.
        }

        // 9. 持久化 auction
        let saved = self.trades.update_auction(&auction).await?;

        tracing::info!(
            target: "economy-service.trade",
            auction_id = %auction_id,
            bidder_id = %bidder_id,
            amount = amount,
            is_highest = true,
            auction_ended = auction_ended,
            "bid placed"
        );

        Ok(BidResult {
            auction: saved,
            bid_id: entry.id,
            is_highest: true,
            auction_ended,
            refunded_to,
            refund_amount,
        })
    }

    async fn cancel_auction(
        &self,
        auction_id: Uuid,
        seller_id: String,
    ) -> Result<CancelResult> {
        // 1. 加载拍卖
        let mut auction = self
            .trades
            .find_auction_by_id(auction_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Auction",
                id: auction_id.to_string(),
            })?;

        // 2. 仅卖家可撤
        if auction.seller_id != seller_id {
            return Err(Error::Forbidden(format!(
                "only seller can cancel auction {}",
                auction_id
            )));
        }
        // 3. 仅 Active 可撤
        if auction.status != AuctionStatus::Active {
            return Err(Error::Conflict(format!(
                "auction {} not active (status={:?})",
                auction_id, auction.status
            )));
        }

        // 4. 退还当前最高出价者
        let refunded_to = auction.highest_bidder.clone();
        let refund_amount = auction.highest_bid;
        if !refunded_to.is_empty() && refund_amount > 0 {
            let currency = Self::parse_currency(auction.currency_type)?;
            let refund_key = format!("cancel-refund-{}-{}", auction_id, refunded_to);
            self.refund_currency(
                &refunded_to,
                refund_amount,
                currency,
                &refund_key,
                &format!("auction {} cancelled", auction_id),
            )
            .await?;
        }

        // 5. 更新状态
        auction.status = AuctionStatus::Cancelled;
        auction.closed_at = Some(chrono::Utc::now());
        let saved = self.trades.update_auction(&auction).await?;

        // TODO(W36+): 卖家卡牌解锁 (card-service.MarkCardUnlocked)

        tracing::info!(
            target: "economy-service.trade",
            auction_id = %auction_id,
            seller_id = %seller_id,
            refunded = refund_amount,
            "auction cancelled"
        );

        Ok(CancelResult {
            auction: saved,
            refunded: refund_amount,
            refunded_to,
        })
    }

    async fn list_auctions(
        &self,
        filter: AuctionFilter,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)> {
        let page = if page == 0 { 1 } else { page };
        let page_size = if page_size == 0 {
            20
        } else if page_size > 100 {
            100
        } else {
            page_size
        };
        self.trades.list_auctions(filter, page, page_size).await
    }

    async fn get_trade_history(
        &self,
        player_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Auction>, u64)> {
        if player_id.is_empty() {
            return Err(Error::Validation("player_id required".to_string()));
        }
        let page = if page == 0 { 1 } else { page };
        let page_size = if page_size == 0 {
            20
        } else if page_size > 100 {
            100
        } else {
            page_size
        };
        self.trades
            .list_auctions_by_player(&player_id, page, page_size)
            .await
    }
}

// ============================================================================
// gRPC 桥接
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as economy_proto;

    pub struct TradeGrpcService {
        pub impl_: Arc<TradeServiceImpl>,
    }

    impl TradeGrpcService {
        pub fn new(impl_: Arc<TradeServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    fn auction_to_proto(a: &Auction) -> economy_proto::Auction {
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
    impl economy_proto::economy_service_server::EconomyService for TradeGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: common_proto::Status::Ok as i32,
                message: "ok".to_string(),
            }))
        }

        async fn get_account(
            &self,
            _request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<economy_proto::Account>, Status> {
            Err(Status::unimplemented(
                "get_account handled by EconomyGrpcService",
            ))
        }

        async fn create_auction(
            &self,
            request: Request<economy_proto::CreateAuctionRequest>,
        ) -> std::result::Result<Response<economy_proto::CreateAuctionResponse>, Status> {
            let req = request.into_inner();
            let auction = self
                .impl_
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
            let auction_id = Uuid::parse_str(&req.auction_id)
                .map_err(|_| Status::invalid_argument(format!("invalid auction_id: {}", req.auction_id)))?;
            let result = self
                .impl_
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
            let auction_id = Uuid::parse_str(&req.auction_id)
                .map_err(|_| Status::invalid_argument(format!("invalid auction_id: {}", req.auction_id)))?;
            let result = self
                .impl_
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
                .impl_
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
                .impl_
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

// ============================================================================
// 私下交易 (ExecuteTrade) 占位 (per RGS-DTL-038 §6.3 + DEC-038-04)
// 范围: W36+ 接入跨域 saga (card-service 协同), 当前实现 propose + cancel
// ============================================================================

/// ExecuteTrade —— 玩家间私下交易占位
///
/// 业务语义: 玩家 A 向玩家 B 提议私下交易, 双方 accept 后:
///   1. A 货币 (proposer_currency_amount) 转给 B
///   2. A 卡牌实例 (proposer_card_instance_id) 转给 B
///   3. B 货币 (counterparty_currency_amount) 转给 A
///   4. B 卡牌实例 (counterparty_card_instance_id) 转给 A
///
/// 跨域 saga (per §6.3 模式): 4 步链式执行, 任意失败触发反向补偿.
/// W36+ 接入点: step 2/4 调 card-service gRPC; 当前单域仅做 propose/cancel 状态管理.
pub struct ExecuteTradeServiceImpl {
    trades: Arc<dyn TradeRepository>,
}

impl ExecuteTradeServiceImpl {
    pub fn new(trades: Arc<dyn TradeRepository>) -> Self {
        Self { trades }
    }

    /// 提议私下交易
    pub async fn propose(
        &self,
        proposer_id: String,
        counterparty_id: String,
        proposer_currency_amount: i64,
        proposer_currency_type: Option<i32>,
        proposer_card_instance_id: Option<String>,
        counterparty_currency_amount: i64,
        counterparty_currency_type: Option<i32>,
        counterparty_card_instance_id: Option<String>,
    ) -> Result<PrivateTrade> {
        if proposer_id == counterparty_id {
            return Err(Error::Validation(
                "proposer and counterparty must differ".to_string(),
            ));
        }
        let trade = PrivateTrade::new(
            proposer_id,
            counterparty_id,
            proposer_currency_amount,
            proposer_currency_type,
            proposer_card_instance_id,
            counterparty_currency_amount,
            counterparty_currency_type,
            counterparty_card_instance_id,
        );
        self.trades.save_private_trade(&trade).await
    }

    /// 取消私下交易
    pub async fn cancel(&self, trade_id: Uuid, requester_id: String) -> Result<PrivateTrade> {
        let mut trade = self
            .trades
            .find_private_trade_by_id(trade_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "PrivateTrade",
                id: trade_id.to_string(),
            })?;
        if trade.proposer_id != requester_id && trade.counterparty_id != requester_id {
            return Err(Error::Forbidden("only trade parties can cancel".to_string()));
        }
        if trade.status != PrivateTradeStatus::Proposed {
            return Err(Error::Conflict(format!(
                "trade {} not in Proposed state",
                trade_id
            )));
        }
        trade.status = PrivateTradeStatus::Cancelled;
        trade.closed_at = Some(chrono::Utc::now());
        self.trades.update_private_trade(&trade).await
    }

    /// 接受私下交易 (W36+ TODO: 触发跨域 saga ExecuteTrade)
    pub async fn accept(&self, trade_id: Uuid, requester_id: String) -> Result<PrivateTrade> {
        let mut trade = self
            .trades
            .find_private_trade_by_id(trade_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "PrivateTrade",
                id: trade_id.to_string(),
            })?;
        if trade.counterparty_id != requester_id {
            return Err(Error::Forbidden(
                "only counterparty can accept".to_string(),
            ));
        }
        if trade.status != PrivateTradeStatus::Proposed {
            return Err(Error::Conflict(format!(
                "trade {} not in Proposed state",
                trade_id
            )));
        }
        trade.status = PrivateTradeStatus::Accepted;
        // TODO(W36+): 触发跨域 saga ExecuteTrade (per §6.3 pattern)
        //   step 1: economy-service.TransferCurrency(proposer -> counterparty, proposer_currency_amount)
        //   step 2: card-service.RemoveCardFromCollection(proposer, proposer_card_instance_id)
        //   step 3: economy-service.TransferCurrency(counterparty -> proposer, counterparty_currency_amount)
        //   step 4: card-service.RemoveCardFromCollection(counterparty, counterparty_card_instance_id)
        //   step 5: card-service.AddCardToCollection(proposer, counterparty_card_instance_id)
        //   step 6: card-service.AddCardToCollection(counterparty, proposer_card_instance_id)
        //   step 7: economy-service.AddTransactionLog
        // 当前单域实现仅标记 Accepted, 不执行实际转移.
        self.trades.update_private_trade(&trade).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InMemoryAccountRepository, InMemoryTransactionLedgerRepository};
    use crate::trade_repository::InMemoryTradeRepository;
    use chrono::Utc;
    use std::sync::Arc;

    /// 构造共享 ledger 的测试服务
    fn make_service() -> (
        TradeServiceImpl,
        Arc<InMemoryAccountRepository>,
        Arc<InMemoryTransactionLedgerRepository>,
    ) {
        let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
        let acc_repo = Arc::new(
            InMemoryAccountRepository::new().with_shared_ledger(led_repo.inner.clone()),
        );
        let trade_repo = Arc::new(InMemoryTradeRepository::new());
        let svc = TradeServiceImpl::new(
            trade_repo as Arc<dyn TradeRepository>,
            acc_repo.clone() as Arc<dyn AccountRepository>,
            led_repo.clone() as Arc<dyn TransactionLedgerRepository>,
        );
        (svc, acc_repo, led_repo)
    }

    fn fund(acc_repo: &InMemoryAccountRepository, player_id: Uuid, currency: Currency, amount: i64) {
        let mut acc = crate::entity::Account::new(player_id, currency);
        acc.credit(amount);
        futures::executor::block_on(async {
            acc_repo.save(&acc).await.unwrap();
        });
    }

    #[tokio::test]
    async fn create_auction_happy_path() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let auction = svc
            .create_auction(
                Uuid::new_v4().to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        assert_eq!(auction.status, AuctionStatus::Active);
        assert_eq!(auction.min_price, 100);
        assert!(auction.ends_at > Utc::now());
    }

    #[tokio::test]
    async fn create_auction_invalid_inputs() {
        let (svc, _acc_repo, _led_repo) = make_service();
        // empty seller
        let err = svc
            .create_auction(
                "".to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
        // negative min_price
        let err = svc
            .create_auction(
                "seller-1".to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                -1,
                1,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn bid_auction_seller_cannot_bid() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, seller, Currency::Gold, 1000);
        let err = svc
            .bid_auction(
                auction.auction_id,
                seller.to_string(),
                200,
                "k-bid-1".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn bid_auction_below_min() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, bidder, Currency::Gold, 1000);
        let err = svc
            .bid_auction(
                auction.auction_id,
                bidder.to_string(),
                50,
                "k-bid-1".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn bid_auction_happy_path() {
        let (svc, acc_repo, led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, bidder, Currency::Gold, 1000);

        let result = svc
            .bid_auction(
                auction.auction_id,
                bidder.to_string(),
                200,
                "k-bid-1".to_string(),
            )
            .await
            .unwrap();
        assert!(result.is_highest);
        assert_eq!(result.auction.highest_bid, 200);
        assert_eq!(result.auction.highest_bidder, bidder.to_string());
        assert!(!result.auction_ended);

        // 余额应减少
        let bidder_acc = acc_repo
            .find_by_player_and_currency(bidder, Currency::Gold)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(bidder_acc.balance, 800);
        // ledger 写入 1 条
        assert_eq!(led_repo.inner.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn bid_auction_outbid_refund() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder1 = Uuid::new_v4();
        let bidder2 = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, bidder1, Currency::Gold, 1000);
        fund(&acc_repo, bidder2, Currency::Gold, 1000);

        // bidder1 出价 200
        let r1 = svc
            .bid_auction(
                auction.auction_id,
                bidder1.to_string(),
                200,
                "k-bid-1".to_string(),
            )
            .await
            .unwrap();
        assert!(r1.is_highest);
        assert!(r1.refunded_to.is_empty());

        // bidder2 出价 300, 应触发 bidder1 退款
        let r2 = svc
            .bid_auction(
                auction.auction_id,
                bidder2.to_string(),
                300,
                "k-bid-2".to_string(),
            )
            .await
            .unwrap();
        assert!(r2.is_highest);
        assert_eq!(r2.refunded_to, bidder1.to_string());
        assert_eq!(r2.refund_amount, 200);

        // bidder1 余额: 1000 - 200 + 200 (退款) = 1000
        let b1 = acc_repo
            .find_by_player_and_currency(bidder1, Currency::Gold)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(b1.balance, 1000);
        // bidder2 余额: 1000 - 300 = 700
        let b2 = acc_repo
            .find_by_player_and_currency(bidder2, Currency::Gold)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(b2.balance, 700);
    }

    #[tokio::test]
    async fn bid_auction_idempotency() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, bidder, Currency::Gold, 1000);

        svc.bid_auction(
            auction.auction_id,
            bidder.to_string(),
            200,
            "k-dup".to_string(),
        )
        .await
        .unwrap();
        let err = svc
            .bid_auction(
                auction.auction_id,
                bidder.to_string(),
                300,
                "k-dup".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::IdempotencyConflict(_)));
    }

    #[tokio::test]
    async fn bid_auction_insufficient_funds() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, bidder, Currency::Gold, 50);

        let err = svc
            .bid_auction(
                auction.auction_id,
                bidder.to_string(),
                200,
                "k-bid".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InsufficientFunds { .. }));
    }

    #[tokio::test]
    async fn cancel_auction_happy_path() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        fund(&acc_repo, bidder, Currency::Gold, 1000);
        svc.bid_auction(
            auction.auction_id,
            bidder.to_string(),
            200,
            "k-bid".to_string(),
        )
        .await
        .unwrap();

        let result = svc
            .cancel_auction(auction.auction_id, seller.to_string())
            .await
            .unwrap();
        assert_eq!(result.refunded, 200);
        assert_eq!(result.refunded_to, bidder.to_string());
        assert_eq!(result.auction.status, AuctionStatus::Cancelled);

        // bidder 余额: 1000 - 200 + 200 = 1000
        let b = acc_repo
            .find_by_player_and_currency(bidder, Currency::Gold)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(b.balance, 1000);
    }

    #[tokio::test]
    async fn cancel_auction_only_seller() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let other = Uuid::new_v4();
        let auction = svc
            .create_auction(
                seller.to_string(),
                "card-001".to_string(),
                "inst-001".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();

        let err = svc
            .cancel_auction(auction.auction_id, other.to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn list_and_history() {
        let (svc, acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        let bidder = Uuid::new_v4();
        fund(&acc_repo, bidder, Currency::Gold, 1000);

        // 3 拍卖
        for i in 0..3 {
            svc.create_auction(
                seller.to_string(),
                format!("card-{}", i),
                format!("inst-{}", i),
                100 + i as i64 * 10,
                1,
                0,
            )
            .await
            .unwrap();
        }
        let (list, total) = svc
            .list_auctions(AuctionFilter::Active, 1, 10)
            .await
            .unwrap();
        assert_eq!(total, 3);
        assert_eq!(list.len(), 3);

        // 历史
        let (_hist, _total) = svc
            .get_trade_history(seller.to_string(), 1, 10)
            .await
            .unwrap();
    }
}
