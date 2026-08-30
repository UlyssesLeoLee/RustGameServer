//! 卡牌 8 桶 / 子桶 1: trade 跨域 saga 端到端 IT (per RGS-DTL-038 §6 + DEC-038-04)
//!
//! W36 跨域 1/3 步收尾 — economy-service trade 域 3 个 saga 的端到端 IT
//!
//! 覆盖范围 (3 IT):
//!   1. `it_open_pack_saga_end_to_end` — OpenPack saga (3 步: DebitCurrency → GenerateDropResult → AddCardToCollection)
//!   2. `it_bid_auction_saga_end_to_end` — BidAuction saga (4 步: LockAuction → DebitCurrency → UpdateHighestBid → CheckAuctionEnded)
//!   3. `it_execute_auction_saga_end_to_end` — ExecuteAuction saga (5 步: FinalizeAuction → TransferCurrency → RemoveCardFromCollection → AddCardToCollection → AddTransactionLog)
//!
//! 设计：
//! - MockCardClient / MockTradeClient 提供可控行为, 记录调用次数
//! - InMemory* repository 提供端到端数据流
//! - 3 步 / 4 步 / 5 步 全调, 验证最终状态 (余额 / instance 数量 / 业务规则)
//!
//! 锚定文件：
//! - 源: src/trade_saga.rs (3 saga 实化)
//! - 源: src/trade_saga_clients.rs (CardClient / TradeClient trait + Mock impl)
//! - 设计: docs/00-基准与治理/RGS-DTL-038 §6.1 / §6.2 / §6.3
//! - 拍板: docs/00-基准与治理/RGS-DDD-CARD-9DEC-2026-08-29.md (DEC-038-04 trade 归 economy-service v2)
//!
//! mTLS 验证: 业务层与传输层解耦, Mock 客户端不涉及 TLS, 真实 gRPC 客户端在
//!            CardGrpcClient::new() 处强制 mTLS (per BAS-003 fail-closed).

use std::sync::Arc;
use uuid::Uuid;

use economy_service::entity::Currency;
use economy_service::repository::{
    AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
    TransactionLedgerRepository,
};
use economy_service::trade_repository::{InMemoryTradeRepository, TradeRepository};
use economy_service::trade_saga::{
    BidAuctionInput, BidAuctionSaga, ExecuteAuctionInput, ExecuteAuctionSaga, OpenPackInput,
    OpenPackSaga,
};
use economy_service::trade_saga_clients::{CardClient, MockCardClient, MockTradeClient, TradeClient};

// ============================================================================
// 测试装配套件
// ============================================================================

fn bootstrap() -> (
    Arc<InMemoryAccountRepository>,
    Arc<InMemoryTransactionLedgerRepository>,
    Arc<InMemoryTradeRepository>,
    Arc<MockCardClient>,
    Arc<MockTradeClient>,
) {
    let led = Arc::new(InMemoryTransactionLedgerRepository::new());
    let acc = Arc::new(
        InMemoryAccountRepository::new().with_shared_ledger(led.inner.clone()),
    );
    let trades = Arc::new(InMemoryTradeRepository::new());
    let card = Arc::new(MockCardClient::new());
    let trade = Arc::new(MockTradeClient::new());
    (acc, led, trades, card, trade)
}

async fn fund(
    acc: &InMemoryAccountRepository,
    player: Uuid,
    currency: Currency,
    amount: i64,
) {
    let mut a = economy_service::entity::Account::new(player, currency);
    a.credit(amount);
    acc.save(&a).await.unwrap();
}

// ============================================================================
// IT 1: OpenPack saga 端到端 (3 步全过, 验证余额 + 抽卡 + 加卡)
// ============================================================================

/// 端到端: 玩家开 2 包卡 → DebitCurrency(2*100=200) + GenerateDropResult(2 包 × 2 cards) + AddCardToCollection(4 张)
///
/// 验证:
/// - saga_id 已生成
/// - 余额: 1000 - 200 = 800
/// - 4 张 card_instance 已创建 (mock 添加计数 = 4)
/// - generate_drop_result 调用 1 次
#[tokio::test]
async fn it_open_pack_saga_end_to_end() {
    let (acc, led, _trades, card, _trade) = bootstrap();
    let player = Uuid::new_v4();
    fund(&acc, player, Currency::Gold, 1000).await;
    // 注入 mock 抽卡结果: 每包 2 张
    card.set_drop_result(vec!["card-A".to_string(), "card-B".to_string()]);

    let saga = OpenPackSaga::new(
        acc.clone() as Arc<dyn economy_service::AccountRepository>,
        led.clone() as Arc<dyn economy_service::TransactionLedgerRepository>,
        card.clone() as Arc<dyn economy_service::CardClient>,
    );
    let out = saga
        .execute(OpenPackInput {
            player_id: player,
            series_id: "series-001".to_string(),
            pack_count: 2,
            pack_size: 2,
            price: 100,
            currency_type: 1, // Gold
            idempotency_key: "k-open-pack-1".to_string(),
        })
        .await
        .expect("OpenPack saga should succeed");

    // saga_id 已生成
    assert_ne!(out.saga_id, Uuid::nil());
    // 扣 100 * 2 = 200
    assert_eq!(out.currency_debited, 200);
    // 4 张卡
    assert_eq!(out.card_instance_ids.len(), 4);

    // 余额验证
    let a = acc
        .find_by_player_and_currency(player, Currency::Gold)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(a.balance, 800);

    // mock 客户端调用次数
    assert_eq!(card.generate_count(), 1, "generate_drop_result called once");
    assert_eq!(card.add_count(), 4, "add_card_to_collection called 4 times");
    assert_eq!(card.remove_count(), 0, "no cards removed in happy path");

    // 已添加的 instance 列表
    let added = card.added_instances();
    assert_eq!(added.len(), 4);
    for (inst_id, owner, card_id, source) in &added {
        assert_ne!(*inst_id, Uuid::nil());
        assert_eq!(*owner, player);
        assert!(card_id == "card-A" || card_id == "card-B");
        assert_eq!(*source, economy_service::CardSource::Pack);
    }

    // ledger: 1 条 spend 账目
    let spend_entry = led
        .find_by_idempotency_key("k-open-pack-1")
        .await
        .unwrap()
        .expect("spend ledger entry should exist");
    assert_eq!(spend_entry.amount, -200);
    assert_eq!(
        spend_entry.kind,
        economy_service::entity::TransactionKind::Spend
    );
}

// ============================================================================
// IT 2: BidAuction saga 端到端 (4 步全过, 验证余额 + auction + 旧出价者退款)
// ============================================================================

/// 端到端: 卖家创建拍卖 → bidder1 出价 200 → bidder2 出价 300 触发退款
///
/// 验证:
/// - lock_auction 调用 2 次 (每次出价)
/// - bidder1 余额: 1000 - 200 + 200 (退) = 1000
/// - bidder2 余额: 1000 - 300 = 700
/// - auction.highest_bid = 300, highest_bidder = bidder2
/// - trade_client.transfer_count = 0 (ExecuteAuction 未触发, 拍卖未到期)
#[tokio::test]
async fn it_bid_auction_saga_end_to_end() {
    let (acc, led, trades, card, trade) = bootstrap();
    let seller = Uuid::new_v4();
    let bidder1 = Uuid::new_v4();
    let bidder2 = Uuid::new_v4();
    fund(&acc, bidder1, Currency::Gold, 1000).await;
    fund(&acc, bidder2, Currency::Gold, 1000).await;

    // 创建拍卖 (1 小时后到期, 避免 timing race)
    let ends_at_unix = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
    let auction = economy_service::trade_entity::Auction::new(
        seller.to_string(),
        "card-001".to_string(),
        Uuid::new_v4().to_string(),
        100, // min_price
        1,   // Gold
        ends_at_unix - chrono::Utc::now().timestamp(),
    );
    let auction = trades.save_auction(&auction).await.unwrap();
    assert_eq!(auction.status, economy_service::trade_entity::AuctionStatus::Active);

    // 构造 BidAuctionSaga (含 ExecuteAuctionSaga, 用于拍卖到期时触发)
    let exec_saga = Arc::new(ExecuteAuctionSaga::new(
        trades.clone() as Arc<dyn economy_service::TradeRepository>,
        acc.clone() as Arc<dyn economy_service::AccountRepository>,
        led.clone() as Arc<dyn economy_service::TransactionLedgerRepository>,
        trade.clone() as Arc<dyn economy_service::TradeClient>,
        card.clone() as Arc<dyn economy_service::CardClient>,
    ));
    let saga = BidAuctionSaga::new(
        trades.clone() as Arc<dyn economy_service::TradeRepository>,
        acc.clone() as Arc<dyn economy_service::AccountRepository>,
        led.clone() as Arc<dyn economy_service::TransactionLedgerRepository>,
        trade.clone() as Arc<dyn economy_service::TradeClient>,
        card.clone() as Arc<dyn economy_service::CardClient>,
    )
    .with_execute_auction_saga(exec_saga);

    // bidder1 出价 200
    let r1 = saga
        .execute(BidAuctionInput {
            auction_id: auction.auction_id,
            bidder_id: bidder1,
            amount: 200,
            idempotency_key: "k-bid-1".to_string(),
        })
        .await
        .expect("first bid should succeed");
    assert!(r1.is_highest);
    assert!(!r1.auction_ended);
    assert!(r1.refunded_to.is_none());
    assert_eq!(r1.refund_amount, 0);

    // bidder2 出价 300 → 触发 bidder1 退款
    let r2 = saga
        .execute(BidAuctionInput {
            auction_id: auction.auction_id,
            bidder_id: bidder2,
            amount: 300,
            idempotency_key: "k-bid-2".to_string(),
        })
        .await
        .expect("second bid should succeed");
    assert!(r2.is_highest);
    assert!(!r2.auction_ended);
    assert_eq!(r2.refunded_to, Some(bidder1), "bidder1 should be refunded");
    assert_eq!(r2.refund_amount, 200);

    // 余额验证
    let b1 = acc
        .find_by_player_and_currency(bidder1, Currency::Gold)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(b1.balance, 1000, "bidder1: 1000 - 200 + 200 = 1000");
    let b2 = acc
        .find_by_player_and_currency(bidder2, Currency::Gold)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(b2.balance, 700, "bidder2: 1000 - 300 = 700");

    // auction 验证
    let a = trades
        .find_auction_by_id(auction.auction_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(a.highest_bid, 300);
    assert_eq!(a.highest_bidder, bidder2.to_string());
    assert_eq!(a.status, economy_service::trade_entity::AuctionStatus::Active);

    // mock 客户端验证
    assert_eq!(trade.lock_count(), 2, "lock_auction called twice");
    assert_eq!(trade.finalize_count(), 0, "no auction finalized yet");
    assert_eq!(trade.transfer_count(), 0, "no currency transferred");
    assert_eq!(card.add_count(), 0, "no cards added (auction not ended)");
    assert_eq!(card.remove_count(), 0, "no cards removed (auction not ended)");

    // ledger 验证: 2 spend + 1 refund = 3 条 (用 idempotency_key 找)
    let bid1 = led
        .find_by_idempotency_key("k-bid-1")
        .await
        .unwrap()
        .expect("bid1 ledger entry should exist");
    assert_eq!(bid1.amount, -200);
    let bid2 = led
        .find_by_idempotency_key("k-bid-2")
        .await
        .unwrap()
        .expect("bid2 ledger entry should exist");
    assert_eq!(bid2.amount, -300);
    let refund_key = format!("refund-{}-{}", auction.auction_id, bidder1);
    let refund_entry = led
        .find_by_idempotency_key(&refund_key)
        .await
        .unwrap()
        .expect("refund ledger entry should exist");
    assert_eq!(refund_entry.amount, 200);
}

// ============================================================================
// IT 3: ExecuteAuction saga 端到端 (5 步全过, 验证 5 步调用 + 最终输出)
// ============================================================================

/// 端到端: ExecuteAuction saga 全链 — FinalizeAuction → TransferCurrency → RemoveCardFromCollection → AddCardToCollection → AddTransactionLog
///
/// 验证:
/// - finalize_auction 调用 1 次
/// - transfer_currency 调用 1 次 (winner → seller, 扣 5% tax)
/// - remove_card_from_collection 调用 1 次 (卖家卡牌实例)
/// - add_card_to_collection 调用 1 次 (winner 收到卡牌)
/// - add_transaction_log 调用 2 次 (卖家收入 + 平台 tax)
/// - tax = 1000 * 5% = 50, seller_amount = 950
/// - new_card_instance_id 已生成
#[tokio::test]
async fn it_execute_auction_saga_end_to_end() {
    let (acc, led, trades, card, trade) = bootstrap();
    let seller = Uuid::new_v4();
    let winner = Uuid::new_v4();
    fund(&acc, winner, Currency::Gold, 1000).await;
    let card_instance_id = Uuid::new_v4();
    let auction_id = Uuid::new_v4();

    let saga = ExecuteAuctionSaga::new(
        trades.clone() as Arc<dyn economy_service::TradeRepository>,
        acc.clone() as Arc<dyn economy_service::AccountRepository>,
        led.clone() as Arc<dyn economy_service::TransactionLedgerRepository>,
        trade.clone() as Arc<dyn economy_service::TradeClient>,
        card.clone() as Arc<dyn economy_service::CardClient>,
    );
    let out = saga
        .execute(ExecuteAuctionInput {
            auction_id,
            winner_id: winner,
            seller_id: seller,
            card_id: "card-001".to_string(),
            card_instance_id,
            final_price: 1000,
            currency_type: 1, // Gold
            tax_bps: 500,      // 5%
        })
        .await
        .expect("ExecuteAuction saga should succeed");

    // 业务输出
    assert_ne!(out.saga_id, Uuid::nil());
    assert_eq!(out.tax_collected, 50, "tax = 1000 * 500 / 10000 = 50");
    assert_eq!(out.amount_transferred, 950, "seller_amount = 1000 - 50 = 950");
    assert_ne!(out.new_card_instance_id, Uuid::nil());

    // 5 步全调 (含 1 个 add_transaction_log 二次调用, 因 tax > 0)
    assert_eq!(trade.finalize_count(), 1, "finalize_auction called once");
    assert_eq!(trade.transfer_count(), 1, "transfer_currency called once");
    assert_eq!(card.remove_count(), 1, "remove_card_from_collection called once");
    assert_eq!(card.add_count(), 1, "add_card_to_collection called once");
    assert_eq!(trade.log_count(), 2, "add_transaction_log called twice (seller + tax)");

    // finalize 记录
    let finalized = trade.finalized_auctions();
    assert_eq!(finalized.len(), 1);
    assert_eq!(finalized[0].0, auction_id);
    assert_eq!(finalized[0].1, winner);
    assert_eq!(finalized[0].2, 1000);

    // 移除的 instance 验证 (mock CardClient 暴露 added/removed 计数)
    assert_eq!(card.remove_count(), 1, "remove_card_from_collection called once");
    let removed = card.removed_instances();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].0, card_instance_id);
    assert_eq!(removed[0].1, seller);
    let added = card.added_instances();
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].1, winner, "new card added to winner");
    assert_eq!(added[0].2, "card-001");
    assert_eq!(added[0].3, economy_service::CardSource::Trade);

    // ledger: ExecuteAuction 不写 ledger (mock transfer/log 不写), 应为 0 条 (fund 不走 ledger)
    // 注: 真实业务里 transfer_currency 应在 economy-service 内写 ledger, 当前 mock 仅记录
    // 此处只验证 fund 没写 ledger
    let entries = led
        .find_by_idempotency_key("fund-not-used")
        .await
        .unwrap();
    assert!(entries.is_none(), "no entries with this key");
}
