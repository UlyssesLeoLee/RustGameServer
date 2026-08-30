//! economy-service trade 跨域 saga 客户端 (per RGS-DTL-038 §6 + DEC-038-04)
//!
//! W36 跨域 1/3 步收尾: economy-service trade 域的卡牌 / 私下交易客户端抽象.
//! 真实 gRPC 客户端 (CardGrpcClient) 通过 tonic Channel + mTLS (per BAS-003),
//! 测试用 MockCardClient / MockTradeClient 提供可控行为.
//!
//! 设计原则 (per BAS-003 mTLS fail-closed):
//! - 所有客户端 trait 抽象, 业务层不直接依赖 tonic
//! - gRPC 真实客户端强制 mTLS, 测试客户端绕过 (但通过 trait 隔离)
//! - 错误统一映射为 economy-service::Error
//!
//! 设计原则 (per 5 域 gRPC 一致性):
//! - CardClient / TradeClient 都返回 `Result<T>` (即 economy-service::Result<T>)
//! - 失败模式: tonic::Status → Error::Transport, 业务校验失败 → Error::Validation 等
//!
//! 卡牌 8 桶 / 子桶 1: trade 域 (per RGS-DTL-038 §4.4 + DEC-038-04 + 9 DEC 全 A 拍板)

use crate::error::Error;
use crate::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ============================================================================
// CardClient trait —— economy-service 调 card-service 的 gRPC 抽象
// ============================================================================

/// 卡牌服务客户端 trait (per RGS-DTL-038 §6.1 OpenPack saga + §6.3 ExecuteAuction saga)
///
/// 业务方法覆盖 trade 跨域 saga 所需:
/// - `generate_drop_result`: OpenPack step 2 (生成抽卡结果)
/// - `add_card_to_collection`: OpenPack step 3 + ExecuteAuction step 4
/// - `remove_card_from_collection`: ExecuteAuction step 3
#[async_trait]
pub trait CardClient: Send + Sync {
    /// §6.1 OpenPack step 2: 按 drop_table 抽 N 张
    ///
    /// 业务实现位于 card-service::CardServiceImpl::generate_drop_result
    /// (per RGS-DTL-038 §6.1, 业务层不依赖随机源, 调用方传入 drop_table)
    ///
    /// 返回抽到的 card_id 列表 (per DropEntry.count 累加, per RGS-DTL-038 §6.1)
    async fn generate_drop_result(
        &self,
        series_id: &str,
        pack_count: u32,
        pack_size: u32,
    ) -> Result<Vec<String>>;

    /// §6.1 OpenPack step 3 + §6.3 ExecuteAuction step 4: 添加卡牌到玩家收藏
    ///
    /// 业务: 给 owner_id 添加 card_id 的卡牌实例, source 用于审计
    /// 返回新创建的 instance_id (UUID)
    async fn add_card_to_collection(
        &self,
        owner_id: Uuid,
        card_id: &str,
        source: CardSource,
        saga_id: Uuid,
    ) -> Result<Uuid>;

    /// §6.3 ExecuteAuction step 3: 从玩家收藏移除卡牌实例
    ///
    /// 业务: 卖家卡牌实例从 collection 转移到 winner, source=Trade
    /// 返回是否成功移除
    async fn remove_card_from_collection(
        &self,
        instance_id: Uuid,
        owner_id: Uuid,
        reason: &str,
        saga_id: Uuid,
    ) -> Result<bool>;
}

/// 卡牌来源 (per card.proto CardInstance.Source, 卡牌添加用)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSource {
    /// 开包
    Pack,
    /// 任务奖励
    Reward,
    /// 交易
    Trade,
    /// GM 补偿
    GmGrant,
    /// 活动
    Event,
}

impl CardSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardSource::Pack => "pack",
            CardSource::Reward => "reward",
            CardSource::Trade => "trade",
            CardSource::GmGrant => "gm_grant",
            CardSource::Event => "event",
        }
    }
}

// ============================================================================
// TradeClient trait —— economy-service 调 trade-service (内部 / 跨域模拟)
// ============================================================================

/// trade 客户端 trait —— economy-service 内部 (saga 跨域编排用)
///
/// 业务覆盖 trade 跨域 saga 所需:
/// - `lock_auction`: BidAuction step 1 (锁定拍卖, 防并发)
/// - `finalize_auction`: ExecuteAuction step 1 (标记 sold, 锁定数据)
/// - `transfer_currency`: ExecuteAuction step 2 (winner → seller)
#[async_trait]
pub trait TradeClient: Send + Sync {
    /// §6.2 BidAuction step 1: 锁定拍卖 (防并发出价)
    ///
    /// 业务: 把 auction.status 临时标记为 Locked, 返回旧 highest_bid / highest_bidder
    /// 失败: auction 不存在 / 已 sold / 已 cancelled
    async fn lock_auction(
        &self,
        auction_id: Uuid,
        saga_id: Uuid,
    ) -> Result<AuctionLockState>;

    /// §6.3 ExecuteAuction step 1: 终结拍卖 (finalize, 标 sold)
    ///
    /// 业务: 标 auction.status = Sold, 设置 winner_id / final_price / closed_at
    /// 失败: auction 不存在 / 已 finalize
    async fn finalize_auction(
        &self,
        auction_id: Uuid,
        winner_id: Uuid,
        final_price: i64,
        saga_id: Uuid,
    ) -> Result<()>;

    /// §6.3 ExecuteAuction step 2: 货币转账 (winner → seller, 扣手续费)
    ///
    /// 业务: winner 扣 final_price * (10000 - tax_bps) / 10000, seller 加相同金额
    ///       tax_bps 由调用方传入 (per AUCTION_FEE_BPS = 500 = 5%)
    /// 失败: 余额不足 / 账户冻结
    async fn transfer_currency(
        &self,
        from_player: Uuid,
        to_player: Uuid,
        amount: i64,
        currency_type: i32,
        saga_id: Uuid,
    ) -> Result<()>;

    /// §6.3 ExecuteAuction step 5: 写交易流水
    async fn add_transaction_log(
        &self,
        player_id: Uuid,
        amount: i64,
        currency_type: i32,
        saga_id: Uuid,
        memo: &str,
    ) -> Result<()>;
}

/// 拍卖锁定状态 (lock_auction 返回值)
#[derive(Debug, Clone)]
pub struct AuctionLockState {
    pub auction_id: Uuid,
    pub seller_id: Uuid,
    pub card_id: String,
    pub card_instance_id: Uuid,
    pub currency_type: i32,
    pub old_highest_bid: i64,
    pub old_highest_bidder: Uuid,
    pub new_bid_amount: i64,
    pub new_bidder_id: Uuid,
}

// ============================================================================
// MockCardClient —— 测试用 (per 现有 InMemory* 模式, 卡牌 8 桶子桶 1 测用)
// ============================================================================

/// 测试用 CardClient —— 内存模拟, 可注入失败行为
///
/// 行为:
/// - generate_drop_result: 固定返回 [mock_card_id] * N (由测试 setup)
/// - add_card_to_collection: 记录 instance_id, 返回 UUID
/// - remove_card_from_collection: 记录删除, 返回 true
///
/// 字段:
/// - drop_result: 注入的抽卡结果 (per 测试需求)
/// - add_count / remove_count: 调用计数器 (断言用)
/// - fail_next: 设置 true 后, 下一次调用返回 Validation 错误
#[derive(Default, Clone)]
pub struct MockCardClient {
    inner: Arc<Mutex<MockCardClientState>>,
}

#[derive(Default)]
struct MockCardClientState {
    /// 注入的抽卡结果 (per OpenPack step 2 mock)
    pub drop_result: Vec<String>,
    /// 注入的 add_card_to_collection 返回的 instance_id (None = auto-gen)
    pub next_add_instance_id: Option<Uuid>,
    /// 调用计数 (断言用)
    pub add_count: u32,
    pub remove_count: u32,
    pub generate_count: u32,
    /// 已添加的 instance_id 列表 (供 execute 验证)
    pub added_instances: Vec<(Uuid, Uuid, String, CardSource)>, // (instance_id, owner_id, card_id, source)
    /// 已移除的 instance_id 列表
    pub removed_instances: Vec<(Uuid, Uuid)>, // (instance_id, owner_id)
    /// 失败注入: 下一次操作返回的 Err
    pub fail_next: Option<String>,
}

impl MockCardClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 mock 抽卡结果
    pub fn set_drop_result(&self, card_ids: Vec<String>) {
        self.inner.lock().unwrap().drop_result = card_ids;
    }

    /// 调用计数
    pub fn add_count(&self) -> u32 {
        self.inner.lock().unwrap().add_count
    }
    pub fn remove_count(&self) -> u32 {
        self.inner.lock().unwrap().remove_count
    }
    pub fn generate_count(&self) -> u32 {
        self.inner.lock().unwrap().generate_count
    }

    /// 已添加的 instance 列表 (供断言)
    pub fn added_instances(&self) -> Vec<(Uuid, Uuid, String, CardSource)> {
        self.inner.lock().unwrap().added_instances.clone()
    }

    /// 已移除的 instance 列表 (供断言)
    pub fn removed_instances(&self) -> Vec<(Uuid, Uuid)> {
        self.inner.lock().unwrap().removed_instances.clone()
    }

    /// 注入下一次失败
    pub fn fail_next(&self, reason: &str) {
        self.inner.lock().unwrap().fail_next = Some(reason.to_string());
    }
}

#[async_trait]
impl CardClient for MockCardClient {
    async fn generate_drop_result(
        &self,
        _series_id: &str,
        pack_count: u32,
        _pack_size: u32,
    ) -> Result<Vec<String>> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        st.generate_count += 1;
        // 按 pack_count 倍数扩展 (per OpenPack 业务: pack_size * pack_count)
        let mut result = Vec::new();
        for _ in 0..pack_count {
            result.extend(st.drop_result.iter().cloned());
        }
        Ok(result)
    }

    async fn add_card_to_collection(
        &self,
        owner_id: Uuid,
        card_id: &str,
        source: CardSource,
        _saga_id: Uuid,
    ) -> Result<Uuid> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        let instance_id = st
            .next_add_instance_id
            .take()
            .unwrap_or_else(Uuid::new_v4);
        st.add_count += 1;
        st.added_instances
            .push((instance_id, owner_id, card_id.to_string(), source));
        Ok(instance_id)
    }

    async fn remove_card_from_collection(
        &self,
        instance_id: Uuid,
        owner_id: Uuid,
        _reason: &str,
        _saga_id: Uuid,
    ) -> Result<bool> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        st.remove_count += 1;
        st.removed_instances.push((instance_id, owner_id));
        Ok(true)
    }
}

// ============================================================================
// MockTradeClient —— 测试用
// ============================================================================

/// 测试用 TradeClient —— 调用 self.trade_service 业务方法
///
/// 行为:
/// - lock_auction: 调 trade_service.bid_auction 之前的 lock 模拟 (status = Active 即可)
/// - finalize_auction: 标 auction.status = Sold
/// - transfer_currency: 调 trade_service 内部 account 逻辑
///
/// 设计: 通过 Arc<dyn ...> 注入 trade_service / accounts / ledger 等依赖
#[derive(Clone)]
pub struct MockTradeClient {
    inner: Arc<Mutex<MockTradeClientState>>,
    pub trade_service: Option<Arc<crate::trade_service::TradeServiceImpl>>,
}

#[derive(Default)]
struct MockTradeClientState {
    pub lock_count: u32,
    pub finalize_count: u32,
    pub transfer_count: u32,
    pub log_count: u32,
    pub locked_auctions: Vec<Uuid>,
    pub finalized_auctions: Vec<(Uuid, Uuid, i64)>, // (auction_id, winner_id, final_price)
    pub transferred: Vec<(Uuid, Uuid, i64)>, // (from, to, amount)
    pub logged: Vec<(Uuid, i64)>, // (player_id, amount)
    pub fail_next: Option<String>,
}

impl MockTradeClient {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(MockTradeClientState::default())),
            trade_service: None,
        }
    }

    pub fn with_trade_service(
        mut self,
        svc: Arc<crate::trade_service::TradeServiceImpl>,
    ) -> Self {
        self.trade_service = Some(svc);
        self
    }

    pub fn lock_count(&self) -> u32 {
        self.inner.lock().unwrap().lock_count
    }
    pub fn finalize_count(&self) -> u32 {
        self.inner.lock().unwrap().finalize_count
    }
    pub fn transfer_count(&self) -> u32 {
        self.inner.lock().unwrap().transfer_count
    }
    pub fn log_count(&self) -> u32 {
        self.inner.lock().unwrap().log_count
    }

    pub fn fail_next(&self, reason: &str) {
        self.inner.lock().unwrap().fail_next = Some(reason.to_string());
    }

    pub fn locked_auctions(&self) -> Vec<Uuid> {
        self.inner.lock().unwrap().locked_auctions.clone()
    }

    pub fn finalized_auctions(&self) -> Vec<(Uuid, Uuid, i64)> {
        self.inner.lock().unwrap().finalized_auctions.clone()
    }
}

#[async_trait]
impl TradeClient for MockTradeClient {
    async fn lock_auction(
        &self,
        auction_id: Uuid,
        _saga_id: Uuid,
    ) -> Result<AuctionLockState> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        st.lock_count += 1;
        st.locked_auctions.push(auction_id);
        // mock 锁定状态: 调用方 (saga) 应已加载 auction 上下文, 此处仅返回 placeholder
        // 业务实现里 saga 步骤会从 saga context 读 auction 信息
        Ok(AuctionLockState {
            auction_id,
            seller_id: Uuid::nil(),
            card_id: String::new(),
            card_instance_id: Uuid::nil(),
            currency_type: 1,
            old_highest_bid: 0,
            old_highest_bidder: Uuid::nil(),
            new_bid_amount: 0,
            new_bidder_id: Uuid::nil(),
        })
    }

    async fn finalize_auction(
        &self,
        auction_id: Uuid,
        winner_id: Uuid,
        final_price: i64,
        _saga_id: Uuid,
    ) -> Result<()> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        st.finalize_count += 1;
        st.finalized_auctions.push((auction_id, winner_id, final_price));
        // 真实业务: 调 trade_service 标记 sold (mock 仅记录)
        if let Some(svc) = &self.trade_service {
            // 业务层 finalize: trade_service 直接写 auction.status = Sold
            // 此处 mock 仅记录, 业务流假设 saga 已在外层更新 auction
            let _ = svc;
        }
        Ok(())
    }

    async fn transfer_currency(
        &self,
        from_player: Uuid,
        to_player: Uuid,
        amount: i64,
        _currency_type: i32,
        _saga_id: Uuid,
    ) -> Result<()> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        st.transfer_count += 1;
        st.transferred.push((from_player, to_player, amount));
        Ok(())
    }

    async fn add_transaction_log(
        &self,
        player_id: Uuid,
        amount: i64,
        _currency_type: i32,
        _saga_id: Uuid,
        _memo: &str,
    ) -> Result<()> {
        let mut st = self.inner.lock().unwrap();
        if let Some(reason) = st.fail_next.take() {
            return Err(Error::Validation(reason));
        }
        st.log_count += 1;
        st.logged.push((player_id, amount));
        Ok(())
    }
}

// ============================================================================
// CardGrpcClient —— 真实 gRPC 客户端 (mTLS, per BAS-003)
// ============================================================================

/// 真实 gRPC CardClient —— per RGS-DTL-038 §6.1/§6.3, 通过 tonic Channel 调 card-service
///
/// mTLS 强制 (per BAS-003 fail-closed):
/// - 必须传 ca_cert_path + client_cert_path + client_key_path
/// - 任何 TLS 配置失败 → Err (不允许静默 fallback 明文)
///
/// 设计:
/// - 用 tonic::transport::Channel, 连接 card-service 地址
/// - 包装 card.v1::card_service_client::CardServiceClient
/// - 调用方法映射: AddCardToCollection / RemoveCardFromCollection
/// - generate_drop_result: 通过 OpenPack RPC 实现 (saga step 2 整批下发, 业务层做概率)
#[derive(Clone)]
pub struct CardGrpcClient {
    /// tonic Channel (mTLS 强制)
    _channel: Arc<tonic::transport::Channel>,
    /// domain (SAN)
    _domain: String,
}

impl CardGrpcClient {
    /// 构造 mTLS gRPC 客户端
    ///
    /// 输入: card-service 地址 (e.g. "https://card-service:50051")
    ///       + ClientTlsConfigInput (ca_cert + client_cert + client_key)
    ///
    /// 失败: 任何 TLS 加载失败 → Err (不静默)
    pub fn new(
        endpoint: String,
        domain: String,
        tls_input: shared_platform::tls::ClientTlsConfigInput,
    ) -> Result<Self> {
        // mTLS 强制 (per BAS-003)
        let client_tls_config = shared_platform::tls::load_client_tls(&tls_input)
            .map_err(|e| Error::Unavailable(format!("mTLS load failed: {}", e)))?;
        // 构造 channel (lazy connect) — 用 domain 传入 endpoint, mTLS 注入
        let endpoint = tonic::transport::Endpoint::from_shared(endpoint)
            .map_err(|e| Error::Validation(format!("invalid endpoint: {}", e)))?
            .tls_config(client_tls_config)
            .map_err(|e| Error::Unavailable(format!("mTLS config failed: {}", e)))?
            .connect_lazy();
        Ok(Self {
            _channel: Arc::new(endpoint),
            _domain: domain,
        })
    }

    /// insecure 客户端 (仅 dev/test, fail-closed bypass)
    pub fn new_insecure(endpoint: String) -> Result<Self> {
        let channel = tonic::transport::Endpoint::from_shared(endpoint)
            .map_err(|e| Error::Validation(format!("invalid endpoint: {}", e)))?
            .connect_lazy();
        Ok(Self {
            _channel: Arc::new(channel),
            _domain: String::new(),
        })
    }
}

#[async_trait]
impl CardClient for CardGrpcClient {
    async fn generate_drop_result(
        &self,
        _series_id: &str,
        _pack_count: u32,
        _pack_size: u32,
    ) -> Result<Vec<String>> {
        // 真实实现: 调 card-service.OpenPack (含 drop_table)
        // 此处只编译, 业务层在 OpenPack saga step 2 调
        // 实际实现需 include card.v1 proto + CardServiceClient
        // 当前为占位, 业务 IT 用 MockCardClient 即可
        Err(Error::Unavailable(
            "CardGrpcClient::generate_drop_result: not yet wired (use MockCardClient for IT)"
                .to_string(),
        ))
    }

    async fn add_card_to_collection(
        &self,
        _owner_id: Uuid,
        _card_id: &str,
        _source: CardSource,
        _saga_id: Uuid,
    ) -> Result<Uuid> {
        Err(Error::Unavailable(
            "CardGrpcClient::add_card_to_collection: not yet wired (use MockCardClient for IT)"
                .to_string(),
        ))
    }

    async fn remove_card_from_collection(
        &self,
        _instance_id: Uuid,
        _owner_id: Uuid,
        _reason: &str,
        _saga_id: Uuid,
    ) -> Result<bool> {
        Err(Error::Unavailable(
            "CardGrpcClient::remove_card_from_collection: not yet wired (use MockCardClient for IT)"
                .to_string(),
        ))
    }
}

// ============================================================================
// Tests —— clients 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_card_generate_returns_drop_result() {
        let c = MockCardClient::new();
        c.set_drop_result(vec!["card-A".to_string(), "card-B".to_string()]);
        let r = c.generate_drop_result("series-1", 2, 3).await.unwrap();
        // pack_count=2 × 2 cards = 4
        assert_eq!(r.len(), 4);
        assert_eq!(c.generate_count(), 1);
    }

    #[tokio::test]
    async fn mock_card_add_records_call() {
        let c = MockCardClient::new();
        let owner = Uuid::new_v4();
        let inst = c
            .add_card_to_collection(owner, "card-1", CardSource::Pack, Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(c.add_count(), 1);
        let added = c.added_instances();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].0, inst);
        assert_eq!(added[0].1, owner);
    }

    #[tokio::test]
    async fn mock_card_fail_next_propagates() {
        let c = MockCardClient::new();
        c.fail_next("simulated card-service outage");
        let err = c
            .add_card_to_collection(Uuid::new_v4(), "card-1", CardSource::Pack, Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
        // 第二次调用应成功 (fail_next 仅一次)
        let _ = c
            .add_card_to_collection(Uuid::new_v4(), "card-1", CardSource::Pack, Uuid::new_v4())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn mock_trade_lock_records() {
        let t = MockTradeClient::new();
        let auction_id = Uuid::new_v4();
        let state = t.lock_auction(auction_id, Uuid::new_v4()).await.unwrap();
        assert_eq!(state.auction_id, auction_id);
        assert_eq!(t.lock_count(), 1);
    }

    #[tokio::test]
    async fn mock_trade_transfer_records() {
        let t = MockTradeClient::new();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        t.transfer_currency(from, to, 500, 1, Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(t.transfer_count(), 1);
    }
}
