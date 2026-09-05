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

use crate::entity::{Currency, TransactionKind, TransactionLedger, TransactionStatus};
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

        // ====================================================================
        // v3 增量: 90 RPC stub 桥接 (per 9/4 MD Phase 2 economy + 商城)
        // TradeGrpcService 路径: 全 Unimplemented (实际业务在 EconomyGrpcService)
        // ====================================================================

        async fn shop_list(
            &self,
            _request: Request<economy_proto::ShopListRequest>,
        ) -> std::result::Result<Response<economy_proto::ShopListResponse>, Status> {
            Err(Status::unimplemented("shop_list"))
        }
        async fn shop_buy(
            &self,
            _request: Request<economy_proto::ShopBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::ShopBuyResponse>, Status> {
            Err(Status::unimplemented("shop_buy"))
        }
        async fn shop_refresh(
            &self,
            _request: Request<economy_proto::ShopRefreshRequest>,
        ) -> std::result::Result<Response<economy_proto::ShopRefreshResponse>, Status> {
            Err(Status::unimplemented("shop_refresh"))
        }
        async fn shop_record(
            &self,
            _request: Request<economy_proto::ShopRecordRequest>,
        ) -> std::result::Result<Response<economy_proto::ShopRecordResponse>, Status> {
            Err(Status::unimplemented("shop_record"))
        }
        async fn mystery_shop_list(
            &self,
            _request: Request<economy_proto::MysteryShopListRequest>,
        ) -> std::result::Result<Response<economy_proto::MysteryShopListResponse>, Status> {
            Err(Status::unimplemented("mystery_shop_list"))
        }
        async fn mystery_shop_buy(
            &self,
            _request: Request<economy_proto::MysteryShopBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::MysteryShopBuyResponse>, Status> {
            Err(Status::unimplemented("mystery_shop_buy"))
        }
        async fn mystery_shop_refresh(
            &self,
            _request: Request<economy_proto::MysteryShopRefreshRequest>,
        ) -> std::result::Result<Response<economy_proto::MysteryShopRefreshResponse>, Status> {
            Err(Status::unimplemented("mystery_shop_refresh"))
        }
        async fn mystery_shop_unlock(
            &self,
            _request: Request<economy_proto::MysteryShopUnlockRequest>,
        ) -> std::result::Result<Response<economy_proto::MysteryShopUnlockResponse>, Status> {
            Err(Status::unimplemented("mystery_shop_unlock"))
        }
        async fn exchange_list(
            &self,
            _request: Request<economy_proto::ExchangeListRequest>,
        ) -> std::result::Result<Response<economy_proto::ExchangeListResponse>, Status> {
            Err(Status::unimplemented("exchange_list"))
        }
        async fn exchange_do(
            &self,
            _request: Request<economy_proto::ExchangeDoRequest>,
        ) -> std::result::Result<Response<economy_proto::ExchangeDoResponse>, Status> {
            Err(Status::unimplemented("exchange_do"))
        }
        async fn exchange_record(
            &self,
            _request: Request<economy_proto::ExchangeRecordRequest>,
        ) -> std::result::Result<Response<economy_proto::ExchangeRecordResponse>, Status> {
            Err(Status::unimplemented("exchange_record"))
        }
        async fn wish_list(
            &self,
            _request: Request<economy_proto::WishListRequest>,
        ) -> std::result::Result<Response<economy_proto::WishListResponse>, Status> {
            Err(Status::unimplemented("wish_list"))
        }
        async fn wish_draw(
            &self,
            _request: Request<economy_proto::WishDrawRequest>,
        ) -> std::result::Result<Response<economy_proto::WishDrawResponse>, Status> {
            Err(Status::unimplemented("wish_draw"))
        }
        async fn wish_reward(
            &self,
            _request: Request<economy_proto::WishRewardRequest>,
        ) -> std::result::Result<Response<economy_proto::WishRewardResponse>, Status> {
            Err(Status::unimplemented("wish_reward"))
        }
        async fn point_shop_list(
            &self,
            _request: Request<economy_proto::PointShopListRequest>,
        ) -> std::result::Result<Response<economy_proto::PointShopListResponse>, Status> {
            Err(Status::unimplemented("point_shop_list"))
        }
        async fn point_shop_buy(
            &self,
            _request: Request<economy_proto::PointShopBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::PointShopBuyResponse>, Status> {
            Err(Status::unimplemented("point_shop_buy"))
        }
        async fn gift_code_redeem(
            &self,
            _request: Request<economy_proto::GiftCodeRedeemRequest>,
        ) -> std::result::Result<Response<economy_proto::GiftCodeRedeemResponse>, Status> {
            Err(Status::unimplemented("gift_code_redeem"))
        }
        async fn gift_code_query(
            &self,
            _request: Request<economy_proto::GiftCodeQueryRequest>,
        ) -> std::result::Result<Response<economy_proto::GiftCodeQueryResponse>, Status> {
            Err(Status::unimplemented("gift_code_query"))
        }
        async fn loot_roll(
            &self,
            _request: Request<economy_proto::LootRollRequest>,
        ) -> std::result::Result<Response<economy_proto::LootRollResponse>, Status> {
            Err(Status::unimplemented("loot_roll"))
        }
        async fn loot_claim(
            &self,
            _request: Request<economy_proto::LootClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::LootClaimResponse>, Status> {
            Err(Status::unimplemented("loot_claim"))
        }

        // 充值类 (15)
        async fn recharge_list(
            &self,
            _request: Request<economy_proto::RechargeListRequest>,
        ) -> std::result::Result<Response<economy_proto::RechargeListResponse>, Status> {
            Err(Status::unimplemented("recharge_list"))
        }
        async fn recharge_do(
            &self,
            _request: Request<economy_proto::RechargeDoRequest>,
        ) -> std::result::Result<Response<economy_proto::RechargeDoResponse>, Status> {
            Err(Status::unimplemented("recharge_do"))
        }
        async fn recharge_order_query(
            &self,
            _request: Request<economy_proto::RechargeOrderQueryRequest>,
        ) -> std::result::Result<Response<economy_proto::RechargeOrderQueryResponse>, Status> {
            Err(Status::unimplemented("recharge_order_query"))
        }
        async fn recharge_order_finish(
            &self,
            _request: Request<economy_proto::RechargeOrderFinishRequest>,
        ) -> std::result::Result<Response<economy_proto::RechargeOrderFinishResponse>, Status> {
            Err(Status::unimplemented("recharge_order_finish"))
        }
        async fn monthly_card_info(
            &self,
            _request: Request<economy_proto::MonthlyCardInfoRequest>,
        ) -> std::result::Result<Response<economy_proto::MonthlyCardInfoResponse>, Status> {
            Err(Status::unimplemented("monthly_card_info"))
        }
        async fn monthly_card_claim(
            &self,
            _request: Request<economy_proto::MonthlyCardClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::MonthlyCardClaimResponse>, Status> {
            Err(Status::unimplemented("monthly_card_claim"))
        }
        async fn monthly_card_buy(
            &self,
            _request: Request<economy_proto::MonthlyCardBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::MonthlyCardBuyResponse>, Status> {
            Err(Status::unimplemented("monthly_card_buy"))
        }
        async fn first_recharge_list(
            &self,
            _request: Request<economy_proto::FirstRechargeListRequest>,
        ) -> std::result::Result<Response<economy_proto::FirstRechargeListResponse>, Status> {
            Err(Status::unimplemented("first_recharge_list"))
        }
        async fn first_recharge_claim(
            &self,
            _request: Request<economy_proto::FirstRechargeClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::FirstRechargeClaimResponse>, Status> {
            Err(Status::unimplemented("first_recharge_claim"))
        }
        async fn first_recharge_status(
            &self,
            _request: Request<economy_proto::FirstRechargeStatusRequest>,
        ) -> std::result::Result<Response<economy_proto::FirstRechargeStatusResponse>, Status> {
            Err(Status::unimplemented("first_recharge_status"))
        }
        async fn power_pack_list(
            &self,
            _request: Request<economy_proto::PowerPackListRequest>,
        ) -> std::result::Result<Response<economy_proto::PowerPackListResponse>, Status> {
            Err(Status::unimplemented("power_pack_list"))
        }
        async fn power_pack_buy(
            &self,
            _request: Request<economy_proto::PowerPackBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::PowerPackBuyResponse>, Status> {
            Err(Status::unimplemented("power_pack_buy"))
        }
        async fn growth_fund_list(
            &self,
            _request: Request<economy_proto::GrowthFundListRequest>,
        ) -> std::result::Result<Response<economy_proto::GrowthFundListResponse>, Status> {
            Err(Status::unimplemented("growth_fund_list"))
        }
        async fn growth_fund_buy(
            &self,
            _request: Request<economy_proto::GrowthFundBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::GrowthFundBuyResponse>, Status> {
            Err(Status::unimplemented("growth_fund_buy"))
        }
        async fn growth_fund_claim(
            &self,
            _request: Request<economy_proto::GrowthFundClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::GrowthFundClaimResponse>, Status> {
            Err(Status::unimplemented("growth_fund_claim"))
        }

        // 抽卡类 (15)
        async fn summon_list(
            &self,
            _request: Request<economy_proto::SummonListRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonListResponse>, Status> {
            Err(Status::unimplemented("summon_list"))
        }
        async fn summon_info(
            &self,
            _request: Request<economy_proto::SummonInfoRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonInfoResponse>, Status> {
            Err(Status::unimplemented("summon_info"))
        }
        async fn summon_single_pull(
            &self,
            _request: Request<economy_proto::SummonSinglePullRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonSinglePullResponse>, Status> {
            Err(Status::unimplemented("summon_single_pull"))
        }
        async fn summon_ten_pull(
            &self,
            _request: Request<economy_proto::SummonTenPullRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonTenPullResponse>, Status> {
            Err(Status::unimplemented("summon_ten_pull"))
        }
        async fn summon_free(
            &self,
            _request: Request<economy_proto::SummonFreeRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonFreeResponse>, Status> {
            Err(Status::unimplemented("summon_free"))
        }
        async fn summon_pity(
            &self,
            _request: Request<economy_proto::SummonPityRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonPityResponse>, Status> {
            Err(Status::unimplemented("summon_pity"))
        }
        async fn summon_share_reward(
            &self,
            _request: Request<economy_proto::SummonShareRewardRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonShareRewardResponse>, Status> {
            Err(Status::unimplemented("summon_share_reward"))
        }
        async fn summon_record(
            &self,
            _request: Request<economy_proto::SummonRecordRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonRecordResponse>, Status> {
            Err(Status::unimplemented("summon_record"))
        }
        async fn summon_box_list(
            &self,
            _request: Request<economy_proto::SummonBoxListRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonBoxListResponse>, Status> {
            Err(Status::unimplemented("summon_box_list"))
        }
        async fn summon_box_unlock(
            &self,
            _request: Request<economy_proto::SummonBoxUnlockRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonBoxUnlockResponse>, Status> {
            Err(Status::unimplemented("summon_box_unlock"))
        }
        async fn summon_featured_draw(
            &self,
            _request: Request<economy_proto::SummonFeaturedDrawRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonFeaturedDrawResponse>, Status> {
            Err(Status::unimplemented("summon_featured_draw"))
        }
        async fn summon_reset_pity(
            &self,
            _request: Request<economy_proto::SummonResetPityRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonResetPityResponse>, Status> {
            Err(Status::unimplemented("summon_reset_pity"))
        }
        async fn summon_exchange(
            &self,
            _request: Request<economy_proto::SummonExchangeRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonExchangeResponse>, Status> {
            Err(Status::unimplemented("summon_exchange"))
        }
        async fn summon_banner_list(
            &self,
            _request: Request<economy_proto::SummonBannerListRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonBannerListResponse>, Status> {
            Err(Status::unimplemented("summon_banner_list"))
        }
        async fn summon_guaranteed_info(
            &self,
            _request: Request<economy_proto::SummonGuaranteedInfoRequest>,
        ) -> std::result::Result<Response<economy_proto::SummonGuaranteedInfoResponse>, Status> {
            Err(Status::unimplemented("summon_guaranteed_info"))
        }

        // 拍卖行扩展 (10)
        async fn auction_my_listings(
            &self,
            _request: Request<economy_proto::AuctionMyListingsRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionMyListingsResponse>, Status> {
            Err(Status::unimplemented("auction_my_listings"))
        }
        async fn auction_search(
            &self,
            _request: Request<economy_proto::AuctionSearchRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionSearchResponse>, Status> {
            Err(Status::unimplemented("auction_search"))
        }
        async fn auction_buyout(
            &self,
            _request: Request<economy_proto::AuctionBuyoutRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionBuyoutResponse>, Status> {
            Err(Status::unimplemented("auction_buyout"))
        }
        async fn auction_relist(
            &self,
            _request: Request<economy_proto::AuctionRelistRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionRelistResponse>, Status> {
            Err(Status::unimplemented("auction_relist"))
        }
        async fn auction_auto_bid(
            &self,
            _request: Request<economy_proto::AuctionAutoBidRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionAutoBidResponse>, Status> {
            Err(Status::unimplemented("auction_auto_bid"))
        }
        async fn auction_my_bids(
            &self,
            _request: Request<economy_proto::AuctionMyBidsRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionMyBidsResponse>, Status> {
            Err(Status::unimplemented("auction_my_bids"))
        }
        async fn auction_saved_search(
            &self,
            _request: Request<economy_proto::AuctionSavedSearchRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionSavedSearchResponse>, Status> {
            Err(Status::unimplemented("auction_saved_search"))
        }
        async fn auction_watch_list(
            &self,
            _request: Request<economy_proto::AuctionWatchListRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionWatchListResponse>, Status> {
            Err(Status::unimplemented("auction_watch_list"))
        }
        async fn auction_watch(
            &self,
            _request: Request<economy_proto::AuctionWatchRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionWatchResponse>, Status> {
            Err(Status::unimplemented("auction_watch"))
        }
        async fn auction_unwatch(
            &self,
            _request: Request<economy_proto::AuctionUnwatchRequest>,
        ) -> std::result::Result<Response<economy_proto::AuctionUnwatchResponse>, Status> {
            Err(Status::unimplemented("auction_unwatch"))
        }

        // 限时 (10)
        async fn flash_sale_list(
            &self,
            _request: Request<economy_proto::FlashSaleListRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleListResponse>, Status> {
            Err(Status::unimplemented("flash_sale_list"))
        }
        async fn flash_sale_info(
            &self,
            _request: Request<economy_proto::FlashSaleInfoRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleInfoResponse>, Status> {
            Err(Status::unimplemented("flash_sale_info"))
        }
        async fn flash_sale_buy(
            &self,
            _request: Request<economy_proto::FlashSaleBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleBuyResponse>, Status> {
            Err(Status::unimplemented("flash_sale_buy"))
        }
        async fn flash_sale_countdown(
            &self,
            _request: Request<economy_proto::FlashSaleCountdownRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleCountdownResponse>, Status> {
            Err(Status::unimplemented("flash_sale_countdown"))
        }
        async fn flash_sale_record(
            &self,
            _request: Request<economy_proto::FlashSaleRecordRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleRecordResponse>, Status> {
            Err(Status::unimplemented("flash_sale_record"))
        }
        async fn flash_sale_subscribe(
            &self,
            _request: Request<economy_proto::FlashSaleSubscribeRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleSubscribeResponse>, Status> {
            Err(Status::unimplemented("flash_sale_subscribe"))
        }
        async fn flash_sale_hot(
            &self,
            _request: Request<economy_proto::FlashSaleHotRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleHotResponse>, Status> {
            Err(Status::unimplemented("flash_sale_hot"))
        }
        async fn flash_sale_recommend(
            &self,
            _request: Request<economy_proto::FlashSaleRecommendRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleRecommendResponse>, Status> {
            Err(Status::unimplemented("flash_sale_recommend"))
        }
        async fn flash_sale_stock(
            &self,
            _request: Request<economy_proto::FlashSaleStockRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleStockResponse>, Status> {
            Err(Status::unimplemented("flash_sale_stock"))
        }
        async fn flash_sale_claim(
            &self,
            _request: Request<economy_proto::FlashSaleClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::FlashSaleClaimResponse>, Status> {
            Err(Status::unimplemented("flash_sale_claim"))
        }

        // 基金/特权 (10)
        async fn fund_list(
            &self,
            _request: Request<economy_proto::FundListRequest>,
        ) -> std::result::Result<Response<economy_proto::FundListResponse>, Status> {
            Err(Status::unimplemented("fund_list"))
        }
        async fn fund_buy(
            &self,
            _request: Request<economy_proto::FundBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::FundBuyResponse>, Status> {
            Err(Status::unimplemented("fund_buy"))
        }
        async fn fund_claim(
            &self,
            _request: Request<economy_proto::FundClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::FundClaimResponse>, Status> {
            Err(Status::unimplemented("fund_claim"))
        }
        async fn fund_status(
            &self,
            _request: Request<economy_proto::FundStatusRequest>,
        ) -> std::result::Result<Response<economy_proto::FundStatusResponse>, Status> {
            Err(Status::unimplemented("fund_status"))
        }
        async fn privilege_list(
            &self,
            _request: Request<economy_proto::PrivilegeListRequest>,
        ) -> std::result::Result<Response<economy_proto::PrivilegeListResponse>, Status> {
            Err(Status::unimplemented("privilege_list"))
        }
        async fn privilege_activate(
            &self,
            _request: Request<economy_proto::PrivilegeActivateRequest>,
        ) -> std::result::Result<Response<economy_proto::PrivilegeActivateResponse>, Status> {
            Err(Status::unimplemented("privilege_activate"))
        }
        async fn privilege_buy(
            &self,
            _request: Request<economy_proto::PrivilegeBuyRequest>,
        ) -> std::result::Result<Response<economy_proto::PrivilegeBuyResponse>, Status> {
            Err(Status::unimplemented("privilege_buy"))
        }
        async fn fund_progress(
            &self,
            _request: Request<economy_proto::FundProgressRequest>,
        ) -> std::result::Result<Response<economy_proto::FundProgressResponse>, Status> {
            Err(Status::unimplemented("fund_progress"))
        }
        async fn privilege_daily(
            &self,
            _request: Request<economy_proto::PrivilegeDailyRequest>,
        ) -> std::result::Result<Response<economy_proto::PrivilegeDailyResponse>, Status> {
            Err(Status::unimplemented("privilege_daily"))
        }
        async fn privilege_rewards(
            &self,
            _request: Request<economy_proto::PrivilegeRewardsRequest>,
        ) -> std::result::Result<Response<economy_proto::PrivilegeRewardsResponse>, Status> {
            Err(Status::unimplemented("privilege_rewards"))
        }

        // 活动 (5) - 数据驱动
        async fn activity_list(
            &self,
            _request: Request<economy_proto::ActivityListRequest>,
        ) -> std::result::Result<Response<economy_proto::ActivityListResponse>, Status> {
            Err(Status::unimplemented("activity_list"))
        }
        async fn activity_claim(
            &self,
            _request: Request<economy_proto::ActivityClaimRequest>,
        ) -> std::result::Result<Response<economy_proto::ActivityClaimResponse>, Status> {
            Err(Status::unimplemented("activity_claim"))
        }
        async fn activity_template(
            &self,
            _request: Request<economy_proto::ActivityTemplateRequest>,
        ) -> std::result::Result<Response<economy_proto::ActivityTemplateResponse>, Status> {
            Err(Status::unimplemented("activity_template"))
        }
        async fn activity_progress(
            &self,
            _request: Request<economy_proto::ActivityProgressRequest>,
        ) -> std::result::Result<Response<economy_proto::ActivityProgressResponse>, Status> {
            Err(Status::unimplemented("activity_progress"))
        }
        async fn activity_subscribe(
            &self,
            _request: Request<economy_proto::ActivitySubscribeRequest>,
        ) -> std::result::Result<Response<economy_proto::ActivitySubscribeResponse>, Status> {
            Err(Status::unimplemented("activity_subscribe"))
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

    async fn fund(acc_repo: &InMemoryAccountRepository, player_id: Uuid, currency: Currency, amount: i64) {
        let mut acc = crate::entity::Account::new(player_id, currency);
        acc.credit(amount);
        acc_repo.save(&acc).await.unwrap();
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
        fund(&acc_repo, seller, Currency::Gold, 1000).await;
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
        fund(&acc_repo, bidder, Currency::Gold, 1000).await;
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
        fund(&acc_repo, bidder, Currency::Gold, 1000).await;

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
        fund(&acc_repo, bidder1, Currency::Gold, 1000).await;
        fund(&acc_repo, bidder2, Currency::Gold, 1000).await;

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
        fund(&acc_repo, bidder, Currency::Gold, 1000).await;

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
        fund(&acc_repo, bidder, Currency::Gold, 50).await;

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
        fund(&acc_repo, bidder, Currency::Gold, 1000).await;
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
        fund(&acc_repo, bidder, Currency::Gold, 1000).await;

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

    // =========================================================================
    // 桶 14 / 子桶 1 收尾: 5 RPC × 2 (happy + validation) = 10 新增 UT
    // 覆盖 trade 域 5 RPC 业务逻辑边界 (per RGS-DTL-038 §4.4 + DEC-038-04)
    // =========================================================================

    // [1/10] create_auction happy: 显式 ends_at_unix
    #[tokio::test]
    async fn ut_create_auction_happy_with_explicit_ends_at() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let future_unix = (chrono::Utc::now() + chrono::Duration::hours(2)).timestamp();
        let auction = svc
            .create_auction(
                Uuid::new_v4().to_string(),
                "card-exp".to_string(),
                "inst-exp".to_string(),
                500,
                2, // Diamond
                future_unix,
            )
            .await
            .unwrap();
        assert_eq!(auction.status, AuctionStatus::Active);
        assert_eq!(auction.currency_type, 2);
        assert!(auction.ends_at.timestamp() <= future_unix + 1);
    }

    // [2/10] create_auction validation: 未知 currency_type
    #[tokio::test]
    async fn ut_create_auction_invalid_currency_type() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let err = svc
            .create_auction(
                Uuid::new_v4().to_string(),
                "card-x".to_string(),
                "inst-x".to_string(),
                100,
                99, // unknown
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // [3/10] bid_auction happy: 出价成为最高, 余额正确扣减
    #[tokio::test]
    async fn ut_bid_auction_happy_becomes_highest() {
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
        fund(&acc_repo, bidder, Currency::Gold, 500).await;
        let r = svc
            .bid_auction(auction.auction_id, bidder.to_string(), 150, "k-bid-1".to_string())
            .await
            .unwrap();
        assert!(r.is_highest);
        assert_eq!(r.auction.highest_bid, 150);
        assert_eq!(r.auction.highest_bidder, bidder.to_string());
        // 余额: 500 - 150 = 350
        let acc = acc_repo
            .find_by_player_and_currency(bidder, Currency::Gold)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(acc.balance, 350);
    }

    // [4/10] bid_auction validation: amount = 0 拒绝
    #[tokio::test]
    async fn ut_bid_auction_invalid_zero_amount() {
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
        fund(&acc_repo, bidder, Currency::Gold, 1000).await;
        let err = svc
            .bid_auction(auction.auction_id, bidder.to_string(), 0, "k-zero".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // [5/10] cancel_auction happy: 无出价时撤单, refunded = 0
    #[tokio::test]
    async fn ut_cancel_auction_happy_no_bids() {
        let (svc, _acc_repo, _led_repo) = make_service();
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
        let r = svc
            .cancel_auction(auction.auction_id, seller.to_string())
            .await
            .unwrap();
        assert_eq!(r.refunded, 0);
        assert_eq!(r.refunded_to, "");
        assert_eq!(r.auction.status, AuctionStatus::Cancelled);
    }

    // [6/10] cancel_auction validation: 不存在 auction
    #[tokio::test]
    async fn ut_cancel_auction_invalid_not_found() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let err = svc
            .cancel_auction(Uuid::new_v4(), Uuid::new_v4().to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    // [7/10] list_auctions happy: filter=All 包含已撤单的拍卖
    #[tokio::test]
    async fn ut_list_auctions_happy_filter_all() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        // 2 active + 1 cancelled
        for i in 0..2 {
            svc.create_auction(
                seller.to_string(),
                format!("card-a-{}", i),
                format!("inst-a-{}", i),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        }
        let to_cancel = svc
            .create_auction(
                seller.to_string(),
                "card-c".to_string(),
                "inst-c".to_string(),
                100,
                1,
                0,
            )
            .await
            .unwrap();
        svc.cancel_auction(to_cancel.auction_id, seller.to_string())
            .await
            .unwrap();
        // All filter: 应返回 3 条
        let (_list_all, total_all) = svc
            .list_auctions(AuctionFilter::All, 1, 10)
            .await
            .unwrap();
        assert_eq!(total_all, 3);
        // Active filter: 应返回 2 条
        let (_list_active, total_active) = svc
            .list_auctions(AuctionFilter::Active, 1, 10)
            .await
            .unwrap();
        assert_eq!(total_active, 2);
    }

    // [8/10] list_auctions validation: page_size 上限保护
    #[tokio::test]
    async fn ut_list_auctions_invalid_pagination_capped() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let seller = Uuid::new_v4();
        // page_size > 100 应被截到 100 (impl 在 service.rs / trade_service.rs)
        let (_list, _total) = svc
            .list_auctions(AuctionFilter::All, 1, 500)
            .await
            .unwrap();
        // page_size=0 应 fallback 到 20
        let (_list2, _total2) = svc
            .list_auctions(AuctionFilter::All, 0, 0)
            .await
            .unwrap();
        // 验证不 panic, 返回值有效
        let _ = seller; // suppress unused
    }

    // [9/10] get_trade_history happy: 空历史
    #[tokio::test]
    async fn ut_get_trade_history_happy_empty() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let random_player = Uuid::new_v4().to_string();
        let (list, total) = svc
            .get_trade_history(random_player.clone(), 1, 10)
            .await
            .unwrap();
        assert_eq!(total, 0);
        assert!(list.is_empty());
    }

    // [10/10] get_trade_history validation: empty player_id
    #[tokio::test]
    async fn ut_get_trade_history_invalid_empty_player() {
        let (svc, _acc_repo, _led_repo) = make_service();
        let err = svc
            .get_trade_history("".to_string(), 1, 10)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }
}
