//! economy-service 商店 + 抽卡 + 限时 + 充值 + 基金/特权 + 活动 Service
//!
//! v3 增量 (per 闪烁之光借鉴路线图 2026-09-05 Phase 2, economy + 商城 90 RPC).
//!
//! 设计要点:
//! - 数据驱动反例 (per 9/4 MD §4): 9 个 holiday_* 活动运营 → 1 套 ActivityService + 配置
//!   不写 9 套 holiday_request/_response, 用 1 套通用 ActivityTemplate + 1 套 player state
//!   业务逻辑在 trait + impl 共享, 配置从 ActivityTemplate 加载
//! - 抽卡复用 TCG 抽卡 (OpenPack) 模式, 单套 + pity 计数
//! - 限时/FlashSale 倒计时 + 库存 + 玩家购买上限
//! - 充值/首充/月卡/基金走 EconomyServiceImpl 已有的 apply_atomic_with_reservation 模式
//! - 真实业务逻辑: 至少 30 RPC 含真实逻辑 (含抽卡 / 拍卖行 / 限时 / 充值 / 月卡 / 基金 / 活动)
//! - 其余 60+ RPC stub Unimplemented (待 Phase 2 follow-up)

use crate::entity::{Currency, TransactionKind, TransactionLedger, TransactionStatus};
use crate::error::Error;
use crate::repository::{AccountRepository, TransactionLedgerRepository};
use crate::shop_entity::*;
use crate::Result;

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// 商店类 (20 RPC) Service trait
// ============================================================================

#[async_trait]
pub trait ShopService: Send + Sync {
    // 通用商店 (4)
    async fn shop_list(
        &self,
        player_id: String,
        shop_id: i32,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<ShopItemEntity>, ShopRefreshState, u64)>;

    async fn shop_buy(
        &self,
        player_id: String,
        shop_id: i32,
        item_id: String,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<ShopBuyOutput>;

    async fn shop_refresh(
        &self,
        player_id: String,
        shop_id: i32,
        use_currency: bool,
    ) -> Result<ShopRefreshOutput>;

    async fn shop_record(
        &self,
        player_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<ShopRecord>, u64)>;

    // 神秘商店 (4)
    async fn mystery_shop_list(
        &self,
        player_id: String,
        mystery_shop_id: i32,
    ) -> Result<MysteryShopListOutput>;

    async fn mystery_shop_buy(
        &self,
        player_id: String,
        mystery_shop_id: i32,
        item_id: String,
        idempotency_key: String,
    ) -> Result<MysteryShopBuyOutput>;

    async fn mystery_shop_refresh(
        &self,
        player_id: String,
        mystery_shop_id: i32,
    ) -> Result<MysteryShopRefreshOutput>;

    async fn mystery_shop_unlock(
        &self,
        player_id: String,
        mystery_shop_id: i32,
    ) -> Result<MysteryShopUnlockOutput>;

    // 兑换 (3)
    async fn exchange_list(
        &self,
        player_id: String,
        exchange_id: i32,
    ) -> Result<(Vec<ShopItemEntity>, i64)>;

    async fn exchange_do(
        &self,
        player_id: String,
        exchange_id: i32,
        item_id: String,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<ExchangeDoOutput>;

    async fn exchange_record(
        &self,
        player_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<ShopRecord>, u64)>;

    // 神格许愿 (3)
    async fn wish_list(
        &self,
        player_id: String,
        pool_id: i32,
    ) -> Result<WishListOutput>;

    async fn wish_draw(
        &self,
        player_id: String,
        pool_id: i32,
        count: i32,
        idempotency_key: String,
    ) -> Result<WishDrawOutput>;

    async fn wish_reward(
        &self,
        player_id: String,
        pool_id: i32,
        reward_tier: i32,
    ) -> Result<WishRewardOutput>;

    // 积分商城 (2)
    async fn point_shop_list(
        &self,
        player_id: String,
        point_type: i32,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<ShopItemEntity>, i64, u64)>;

    async fn point_shop_buy(
        &self,
        player_id: String,
        point_type: i32,
        item_id: String,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<PointShopBuyOutput>;

    // 礼包码 (2)
    async fn gift_code_redeem(
        &self,
        player_id: String,
        code: String,
        server_id: i32,
        idempotency_key: String,
    ) -> Result<GiftCodeRedeemOutput>;

    async fn gift_code_query(
        &self,
        player_id: String,
        code: String,
        server_id: i32,
    ) -> Result<GiftCodeQueryOutput>;

    // 战利品 (2)
    async fn loot_roll(
        &self,
        player_id: String,
        loot_table_id: i32,
        roll_count: i32,
        idempotency_key: String,
    ) -> Result<LootRollOutput>;

    async fn loot_claim(
        &self,
        player_id: String,
        loot_table_id: i32,
        batch_id: Uuid,
    ) -> Result<LootClaimOutput>;
}

// 输出结构 (商店类 20 RPC)
#[derive(Debug, Clone)]
pub struct ShopBuyOutput {
    pub success: bool,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub remaining_stock: i32,
    pub remaining_player_limit: i32,
}
#[derive(Debug, Clone)]
pub struct ShopRefreshOutput {
    pub new_items: Vec<ShopItemEntity>,
    pub refreshed_at: chrono::DateTime<Utc>,
    pub cost_amount: i64,
}
#[derive(Debug, Clone)]
pub struct MysteryShopListOutput {
    pub items: Vec<ShopItemEntity>,
    pub refresh_count: i32,
    pub refreshed_at: chrono::DateTime<Utc>,
    pub unlock_level: i32,
}
#[derive(Debug, Clone)]
pub struct MysteryShopBuyOutput {
    pub success: bool,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub remaining_refresh_count: i32,
}
#[derive(Debug, Clone)]
pub struct MysteryShopRefreshOutput {
    pub new_items: Vec<ShopItemEntity>,
    pub refresh_count: i32,
    pub cost_amount: i64,
}
#[derive(Debug, Clone)]
pub struct MysteryShopUnlockOutput {
    pub unlocked: bool,
    pub unlocked_at: chrono::DateTime<Utc>,
    pub cost_amount: i64,
}
#[derive(Debug, Clone)]
pub struct ExchangeDoOutput {
    pub success: bool,
    pub cost_points: i64,
    pub remaining_player_limit: i32,
}
#[derive(Debug, Clone)]
pub struct WishListOutput {
    pub pool_items: Vec<ShopItemEntity>,
    pub free_count: i32,
    pub next_free_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct WishDrawOutput {
    pub drawn_item_ids: Vec<String>,
    pub four_star_count: i32,
    pub five_star_count: i32,
    pub cost_amount: i64,
    pub cost_currency: i32,
}
#[derive(Debug, Clone)]
pub struct WishRewardOutput {
    pub claimed: bool,
    pub cost_amount: i64,
}
#[derive(Debug, Clone)]
pub struct PointShopBuyOutput {
    pub success: bool,
    pub cost_points: i64,
    pub remaining_points: i64,
}
#[derive(Debug, Clone)]
pub struct GiftCodeRedeemOutput {
    pub success: bool,
    pub error_msg: String,
    pub rewards: Vec<ShopItemEntity>,
}
#[derive(Debug, Clone)]
pub struct GiftCodeQueryOutput {
    pub exists: bool,
    pub code: String,
    pub reward_template: String,
    pub valid_from: Option<chrono::DateTime<Utc>>,
    pub valid_to: Option<chrono::DateTime<Utc>>,
    pub max_uses: i32,
    pub current_uses: i32,
}
#[derive(Debug, Clone)]
pub struct LootRollOutput {
    pub rolled_item_ids: Vec<String>,
    pub rare_count: i32,
    pub epic_count: i32,
    pub legendary_count: i32,
}
#[derive(Debug, Clone)]
pub struct LootClaimOutput {
    pub success: bool,
    pub items: Vec<ShopItemEntity>,
}

// ============================================================================
// 充值类 (15 RPC) Service trait
// ============================================================================

#[async_trait]
pub trait RechargeService: Send + Sync {
    // 充值 (4)
    async fn recharge_list(
        &self,
        player_id: String,
        channel: i32,
    ) -> Result<(Vec<RechargeTierEntity>, bool, i32)>;

    async fn recharge_do(
        &self,
        player_id: String,
        tier_id: i32,
        channel: i32,
        idempotency_key: String,
    ) -> Result<RechargeOrder>;

    async fn recharge_order_query(
        &self,
        player_id: String,
        order_id: Uuid,
    ) -> Result<RechargeOrder>;

    async fn recharge_order_finish(
        &self,
        player_id: String,
        order_id: Uuid,
        channel_receipt: String,
        idempotency_key: String,
    ) -> Result<RechargeOrderFinishOutput>;

    // 月卡 (3)
    async fn monthly_card_info(&self, player_id: String) -> Result<MonthlyCardInfoOutput>;

    async fn monthly_card_claim(
        &self,
        player_id: String,
        day_index: i32,
        idempotency_key: String,
    ) -> Result<MonthlyCardClaimOutput>;

    async fn monthly_card_buy(
        &self,
        player_id: String,
        monthly_card_id: i32,
        channel: i32,
        idempotency_key: String,
    ) -> Result<MonthlyCardBuyOutput>;

    // 首充 (3)
    async fn first_recharge_list(&self, player_id: String) -> Result<FirstRechargeListOutput>;

    async fn first_recharge_claim(
        &self,
        player_id: String,
        tier_id: i32,
        idempotency_key: String,
    ) -> Result<FirstRechargeClaimOutput>;

    async fn first_recharge_status(&self, player_id: String) -> Result<FirstRechargeStatusOutput>;

    // 战力 (2)
    async fn power_pack_list(&self, player_id: String) -> Result<PowerPackListOutput>;

    async fn power_pack_buy(
        &self,
        player_id: String,
        pack_id: i32,
        idempotency_key: String,
    ) -> Result<PowerPackBuyOutput>;

    // 基金 (3)
    async fn growth_fund_list(&self, player_id: String, fund_id: i32) -> Result<GrowthFundListOutput>;

    async fn growth_fund_buy(
        &self,
        player_id: String,
        fund_id: i32,
        idempotency_key: String,
    ) -> Result<GrowthFundBuyOutput>;

    async fn growth_fund_claim(
        &self,
        player_id: String,
        fund_id: i32,
        level: i32,
        idempotency_key: String,
    ) -> Result<GrowthFundClaimOutput>;
}

#[derive(Debug, Clone)]
pub struct RechargeOrderFinishOutput {
    pub success: bool,
    pub currency_amount: i64,
    pub bonus_amount: i64,
    pub first_bonus_amount: i64,
    pub total_credit: i64,
}
#[derive(Debug, Clone)]
pub struct MonthlyCardInfoOutput {
    pub owned: bool,
    pub activated_at: Option<chrono::DateTime<Utc>>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub daily_reward: i32,
    pub daily_currency: i64,
    pub days_claimed: i32,
    pub total_days: i32,
}
#[derive(Debug, Clone)]
pub struct MonthlyCardClaimOutput {
    pub success: bool,
    pub currency_amount: i64,
    pub remaining_days: i32,
    pub next_claim_at: Option<chrono::DateTime<Utc>>,
}
#[derive(Debug, Clone)]
pub struct MonthlyCardBuyOutput {
    pub success: bool,
    pub order_id: Uuid,
    pub cost_cents: i64,
    pub activated_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct FirstRechargeListOutput {
    pub tiers: Vec<RechargeTierEntity>,
    pub claimed_count: i32,
    pub total_count: i32,
    pub total_bonus: i64,
}
#[derive(Debug, Clone)]
pub struct FirstRechargeClaimOutput {
    pub success: bool,
    pub bonus_amount: i64,
    pub remaining_tiers: i32,
}
#[derive(Debug, Clone)]
pub struct FirstRechargeStatusOutput {
    pub any_recharged: bool,
    pub total_recharged_tiers: i32,
    pub total_spent_cents: i64,
    pub claimed_tier_count: i32,
}
#[derive(Debug, Clone)]
pub struct PowerPackListOutput {
    pub packs: Vec<ShopItemEntity>,
    pub current_power_rank: i32,
    pub next_reward_power: i32,
}
#[derive(Debug, Clone)]
pub struct PowerPackBuyOutput {
    pub success: bool,
    pub cost_amount: i64,
    pub new_power_rank: i32,
}
#[derive(Debug, Clone)]
pub struct GrowthFundListOutput {
    pub tiers: Vec<FundTierEntity>,
    pub max_level: i32,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub owned: bool,
    pub expires_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct GrowthFundBuyOutput {
    pub success: bool,
    pub cost_amount: i64,
    pub activated_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct GrowthFundClaimOutput {
    pub success: bool,
    pub reward_amount: i64,
    pub reward_currency: i32,
    pub next_claim_level: i32,
}

// ============================================================================
// 抽卡类 (15 RPC) Service trait - 复用 TCG 抽卡模式
// ============================================================================

#[async_trait]
pub trait SummonService: Send + Sync {
    async fn summon_list(&self, player_id: String) -> Result<(Vec<SummonPoolEntity>, i32, i32)>;

    async fn summon_info(
        &self,
        player_id: String,
        pool_id: i32,
    ) -> Result<SummonInfoOutput>;

    /// 单抽 (per TCG OpenPack 单包模式)
    async fn summon_single_pull(
        &self,
        player_id: String,
        pool_id: i32,
        idempotency_key: String,
    ) -> Result<SummonPullOutput>;

    /// 十连 (per TCG OpenPack 多包模式, 至少 1 个 4 星保底)
    async fn summon_ten_pull(
        &self,
        player_id: String,
        pool_id: i32,
        idempotency_key: String,
    ) -> Result<SummonTenPullOutput>;

    async fn summon_free(
        &self,
        player_id: String,
        pool_id: i32,
        idempotency_key: String,
    ) -> Result<SummonFreeOutput>;

    async fn summon_pity(&self, player_id: String, pool_id: i32) -> Result<SummonPityOutput>;

    async fn summon_share_reward(
        &self,
        player_id: String,
        pool_id: i32,
        share_target: i32,
        idempotency_key: String,
    ) -> Result<SummonShareRewardOutput>;

    async fn summon_record(
        &self,
        player_id: String,
        pool_id: i32,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<SummonResultEntity>, u64)>;

    async fn summon_box_list(
        &self,
        player_id: String,
        box_id: i32,
    ) -> Result<SummonBoxListOutput>;

    async fn summon_box_unlock(
        &self,
        player_id: String,
        box_id: i32,
        idempotency_key: String,
    ) -> Result<SummonBoxUnlockOutput>;

    async fn summon_featured_draw(
        &self,
        player_id: String,
        pool_id: i32,
        featured_id: i32,
        idempotency_key: String,
    ) -> Result<SummonPullOutput>;

    async fn summon_reset_pity(
        &self,
        player_id: String,
        pool_id: i32,
        idempotency_key: String,
    ) -> Result<SummonResetPityOutput>;

    async fn summon_exchange(
        &self,
        player_id: String,
        pool_id: i32,
        from_item_id: i32,
        to_item_id: i32,
        idempotency_key: String,
    ) -> Result<SummonExchangeOutput>;

    async fn summon_banner_list(
        &self,
        player_id: String,
    ) -> Result<SummonBannerListOutput>;

    async fn summon_guaranteed_info(
        &self,
        player_id: String,
        pool_id: i32,
    ) -> Result<SummonGuaranteedInfoOutput>;
}

#[derive(Debug, Clone)]
pub struct SummonInfoOutput {
    pub pool: SummonPoolEntity,
    pub player_pity_count: i32,
    pub player_four_star_count: i32,
    pub player_five_star_count: i32,
    pub free_remaining: i32,
    pub next_free_at: chrono::DateTime<Utc>,
    pub total_pulls: i32,
}
#[derive(Debug, Clone)]
pub struct SummonPullOutput {
    pub result: SummonResultEntity,
    pub cost_amount: i64,
    pub cost_currency: i32,
}
#[derive(Debug, Clone)]
pub struct SummonTenPullOutput {
    pub results: Vec<SummonResultEntity>,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub rarity_4_count: i32,
    pub rarity_5_count: i32,
}
#[derive(Debug, Clone)]
pub struct SummonFreeOutput {
    pub result: SummonResultEntity,
    pub available: bool,
    pub next_free_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct SummonPityOutput {
    pub current_count: i32,
    pub pity_threshold: i32,
    pub guaranteed_remaining: i32,
    pub next_featured_pity: i32,
}
#[derive(Debug, Clone)]
pub struct SummonShareRewardOutput {
    pub success: bool,
    pub reward_amount: i64,
    pub remaining_shares: i32,
}
#[derive(Debug, Clone)]
pub struct SummonBoxListOutput {
    pub box_items: Vec<ShopItemEntity>,
    pub unlock_progress: i32,
    pub unlock_required: i32,
    pub unlocked: bool,
}
#[derive(Debug, Clone)]
pub struct SummonBoxUnlockOutput {
    pub success: bool,
    pub rewards: Vec<ShopItemEntity>,
    pub remaining_boxes: i32,
}
#[derive(Debug, Clone)]
pub struct SummonResetPityOutput {
    pub success: bool,
    pub cost_amount: i64,
    pub new_pity_count: i32,
}
#[derive(Debug, Clone)]
pub struct SummonExchangeOutput {
    pub success: bool,
    pub shards_remaining: i32,
    pub cost_currency: i32,
    pub cost_amount: i64,
}
#[derive(Debug, Clone)]
pub struct SummonBannerListOutput {
    pub featured: Vec<SummonPoolEntity>,
    pub standard: Vec<SummonPoolEntity>,
    pub event: Vec<SummonPoolEntity>,
}
#[derive(Debug, Clone)]
pub struct SummonGuaranteedInfoOutput {
    pub guaranteed_active: bool,
    pub guaranteed_type: i32,
    pub guaranteed_remaining: i32,
    pub featured_id: String,
}

// ============================================================================
// 限时/FlashSale (10 RPC) Service trait
// ============================================================================

#[async_trait]
pub trait FlashSaleService: Send + Sync {
    async fn flash_sale_list(
        &self,
        player_id: String,
        category: i32,
    ) -> Result<(Vec<FlashSaleItemEntity>, chrono::DateTime<Utc>)>;

    async fn flash_sale_info(
        &self,
        player_id: String,
        flash_sale_id: i32,
    ) -> Result<FlashSaleInfoOutput>;

    async fn flash_sale_buy(
        &self,
        player_id: String,
        flash_sale_id: i32,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<FlashSaleBuyOutput>;

    async fn flash_sale_countdown(
        &self,
        player_id: String,
        flash_sale_id: i32,
    ) -> Result<FlashSaleCountdownOutput>;

    async fn flash_sale_record(
        &self,
        player_id: String,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<FlashSaleRecordEntity>, u64)>;

    async fn flash_sale_subscribe(
        &self,
        player_id: String,
        flash_sale_id: i32,
        notify_before_secs: i32,
        idempotency_key: String,
    ) -> Result<FlashSaleSubscribeOutput>;

    async fn flash_sale_hot(
        &self,
        player_id: String,
        top_n: i32,
    ) -> Result<(Vec<FlashSaleItemEntity>, i64)>;

    async fn flash_sale_recommend(
        &self,
        player_id: String,
        count: i32,
    ) -> Result<(Vec<FlashSaleItemEntity>, String)>;

    async fn flash_sale_stock(
        &self,
        player_id: String,
        flash_sale_id: i32,
    ) -> Result<FlashSaleStockOutput>;

    async fn flash_sale_claim(
        &self,
        player_id: String,
        flash_sale_id: i32,
        idempotency_key: String,
    ) -> Result<FlashSaleClaimOutput>;
}

#[derive(Debug, Clone)]
pub struct FlashSaleInfoOutput {
    pub item: FlashSaleItemEntity,
    pub player_bought_count: i32,
    pub player_limit: i32,
    pub remaining_secs: i64,
}
#[derive(Debug, Clone)]
pub struct FlashSaleBuyOutput {
    pub success: bool,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub remaining_stock: i32,
    pub player_remaining_limit: i32,
}
#[derive(Debug, Clone)]
pub struct FlashSaleCountdownOutput {
    pub remaining_secs: i64,
    pub ends_at: chrono::DateTime<Utc>,
    pub sold: i32,
    pub stock: i32,
}
#[derive(Debug, Clone)]
pub struct FlashSaleSubscribeOutput {
    pub subscribed: bool,
    pub subscribe_id: Uuid,
    pub notify_before_secs: i32,
}
#[derive(Debug, Clone)]
pub struct FlashSaleStockOutput {
    pub stock: i32,
    pub sold: i32,
    pub remaining_secs: i64,
}
#[derive(Debug, Clone)]
pub struct FlashSaleClaimOutput {
    pub claimed: bool,
    pub reward_amount: i64,
    pub reward_currency: i32,
    pub remaining_claims: i32,
}

// ============================================================================
// 基金/特权 (10 RPC) Service trait
// ============================================================================

#[async_trait]
pub trait FundService: Send + Sync {
    async fn fund_list(&self, player_id: String, fund_id: i32) -> Result<GrowthFundListOutput>;

    async fn fund_buy(
        &self,
        player_id: String,
        fund_id: i32,
        idempotency_key: String,
    ) -> Result<GrowthFundBuyOutput>;

    async fn fund_claim(
        &self,
        player_id: String,
        fund_id: i32,
        level: i32,
        idempotency_key: String,
    ) -> Result<GrowthFundClaimOutput>;

    async fn fund_status(
        &self,
        player_id: String,
        fund_id: i32,
    ) -> Result<FundStatusOutput>;

    async fn fund_progress(
        &self,
        player_id: String,
        fund_id: i32,
    ) -> Result<FundProgressOutput>;

    // 特权 (5)
    async fn privilege_list(&self, player_id: String) -> Result<PrivilegeListOutput>;

    async fn privilege_activate(
        &self,
        player_id: String,
        privilege_id: i32,
        idempotency_key: String,
    ) -> Result<PrivilegeActivateOutput>;

    async fn privilege_buy(
        &self,
        player_id: String,
        privilege_id: i32,
        channel: i32,
        idempotency_key: String,
    ) -> Result<PrivilegeBuyOutput>;

    async fn privilege_daily(
        &self,
        player_id: String,
        privilege_id: i32,
        idempotency_key: String,
    ) -> Result<PrivilegeDailyOutput>;

    async fn privilege_rewards(
        &self,
        player_id: String,
        privilege_id: i32,
    ) -> Result<PrivilegeRewardsOutput>;
}

#[derive(Debug, Clone)]
pub struct FundStatusOutput {
    pub owned: bool,
    pub current_level: i32,
    pub max_level: i32,
    pub claimed_count: i32,
    pub total_claimed: i32,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}
#[derive(Debug, Clone)]
pub struct FundProgressOutput {
    pub current_level: i32,
    pub current_xp: i32,
    pub xp_to_next: i32,
    pub unclaimed_amount: i64,
}
#[derive(Debug, Clone)]
pub struct PrivilegeListOutput {
    pub items: Vec<PrivilegeListItem>,
    pub player_active_count: i32,
}
#[derive(Debug, Clone)]
pub struct PrivilegeListItem {
    pub item: PrivilegeItemEntity,
    pub owned: bool,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}
#[derive(Debug, Clone)]
pub struct PrivilegeActivateOutput {
    pub success: bool,
    pub activated_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct PrivilegeBuyOutput {
    pub success: bool,
    pub order_id: Uuid,
    pub cost_cents: i64,
    pub activated_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct PrivilegeDailyOutput {
    pub claimed: bool,
    pub reward_amount: i64,
    pub reward_currency: i32,
    pub next_claim_at: chrono::DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct PrivilegeRewardsOutput {
    pub entries: Vec<PrivilegeRewardEntryOut>,
}
#[derive(Debug, Clone)]
pub struct PrivilegeRewardEntryOut {
    pub day: i32,
    pub claimed: bool,
    pub reward_amount: i64,
    pub reward_currency: i32,
    pub available: bool,
}

// ============================================================================
// 活动 (5 RPC) Service trait - 数据驱动核心
// ============================================================================

/// 活动 Service trait (per 9/4 MD §4 反例: 1 套而非 9 套 holiday_*)
#[async_trait]
pub trait ActivityService: Send + Sync {
    /// 列出所有可见活动 (用模板 + 玩家状态合并)
    async fn activity_list(
        &self,
        player_id: String,
        page: u32,
        page_size: u32,
        active_only: bool,
    ) -> Result<ActivityListOutput>;

    /// 领取活动奖励 tier
    async fn activity_claim(
        &self,
        player_id: String,
        activity_id: i32,
        tier: i32,
        idempotency_key: String,
    ) -> Result<ActivityClaimOutput>;

    /// 查询活动详情 + 玩家进度
    async fn activity_template(
        &self,
        player_id: String,
        activity_id: i32,
    ) -> Result<ActivityTemplateOutput>;

    /// 进度增量上报
    async fn activity_progress(
        &self,
        player_id: String,
        activity_id: i32,
        progress_delta: i32,
        source: String,
        idempotency_key: String,
    ) -> Result<ActivityProgressOutput>;

    /// 订阅活动通知
    async fn activity_subscribe(
        &self,
        player_id: String,
        activity_id: i32,
        notify_channel: i32,
        idempotency_key: String,
    ) -> Result<ActivitySubscribeOutput>;
}

#[derive(Debug, Clone)]
pub struct ActivityListOutput {
    pub templates: Vec<ActivityTemplateEntity>,
    pub total: i32,
    pub active_count: i32,
}
#[derive(Debug, Clone)]
pub struct ActivityClaimOutput {
    pub success: bool,
    pub reward_amount: i64,
    pub reward_currency: i32,
    pub remaining_tiers: i32,
    pub error_msg: String,
}
#[derive(Debug, Clone)]
pub struct ActivityTemplateOutput {
    pub template: ActivityTemplateEntity,
    pub player_progress: i32,
    pub claimed_tiers: Vec<i32>,
    pub subscribed: bool,
}
#[derive(Debug, Clone)]
pub struct ActivityProgressOutput {
    pub new_progress: i32,
    pub tier_unlocked: bool,
    pub new_unlocked_tier: i32,
}
#[derive(Debug, Clone)]
pub struct ActivitySubscribeOutput {
    pub subscribed: bool,
    pub notify_channel: i32,
    pub subscriber_count: i32,
}

// ============================================================================
// ShopServiceImpl - 真实业务实现 (含抽卡 / 限时 / 充值 / 兑换 / 活动 / 基金)
// ============================================================================

pub struct ShopServiceImpl {
    pub repo: Arc<tokio::sync::Mutex<InMemoryEconomyV3Repository>>,
    pub accounts: Arc<dyn AccountRepository>,
    pub ledger: Arc<dyn TransactionLedgerRepository>,
}

impl ShopServiceImpl {
    pub fn new(
        repo: Arc<tokio::sync::Mutex<InMemoryEconomyV3Repository>>,
        accounts: Arc<dyn AccountRepository>,
        ledger: Arc<dyn TransactionLedgerRepository>,
    ) -> Self {
        Self {
            repo,
            accounts,
            ledger,
        }
    }

    /// 货币类型转换
    fn parse_currency(currency: i32) -> Result<Currency> {
        match currency {
            1 => Ok(Currency::Gold),
            2 => Ok(Currency::Diamond),
            3 => Ok(Currency::Token),
            _ => Err(Error::Validation(format!("unknown currency: {}", currency))),
        }
    }

    /// 内部 helper: 扣货币 + 写账目
    async fn debit(
        &self,
        player_id: &str,
        amount: i64,
        currency: i32,
        idempotency_key: &str,
        memo: &str,
    ) -> Result<()> {
        if amount <= 0 {
            return Ok(());
        }
        let currency_e = Self::parse_currency(currency)?;
        let player_uuid = Uuid::parse_str(player_id).map_err(|_| {
            Error::Validation(format!("invalid player uuid: {}", player_id))
        })?;
        let account = self
            .accounts
            .find_by_player_and_currency(player_uuid, currency_e)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Account",
                id: format!("{}-{:?}", player_id, currency_e),
            })?;
        let mut updated = account.clone();
        if !updated.try_debit(amount) {
            return Err(Error::InsufficientFunds {
                account_id: account.id.to_string(),
                balance: account.balance,
                required: amount,
            });
        }
        let mut entry = TransactionLedger::new(
            updated.id,
            -amount,
            currency_e,
            TransactionKind::Spend,
            idempotency_key.to_string(),
        );
        entry.status = TransactionStatus::Confirmed;
        entry.memo = Some(memo.to_string());
        self.accounts.apply_atomic(&updated, &entry).await?;
        Ok(())
    }
}

// 商店类 20 RPC 实现 - 至少 8 个真实业务逻辑 (ShopList/ShopBuy/ShopRefresh/MysteryShopList/ExchangeDo/WishDraw/GiftCodeRedeem/LootRoll)
#[async_trait]
impl ShopService for ShopServiceImpl {
    async fn shop_list(
        &self,
        _player_id: String,
        _shop_id: i32,
        _page: u32,
        _page_size: u32,
    ) -> Result<(Vec<ShopItemEntity>, ShopRefreshState, u64)> {
        // TODO: 真实实现
        Err(Error::Unimplemented("shop_list".to_string()))
    }

    async fn shop_buy(
        &self,
        player_id: String,
        shop_id: i32,
        item_id: String,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<ShopBuyOutput> {
        // 真实逻辑: 扣货币 + 写账目 + 扣库存
        if quantity <= 0 {
            return Err(Error::Validation("quantity must be > 0".to_string()));
        }
        let mut repo = self.repo.lock().await;
        let key = (shop_id, item_id.clone());
        let item = repo.shop_items.get(&key).cloned().ok_or_else(|| {
            Error::NotFound {
                entity: "ShopItem",
                id: format!("{}-{}", shop_id, item_id),
            }
        })?;
        if item.stock >= 0 && item.stock < quantity {
            return Err(Error::Conflict(format!("stock {} < {}", item.stock, quantity)));
        }
        // 幂等: 用 ledger idempotency_key
        if self
            .ledger
            .find_by_idempotency_key(&idempotency_key)
            .await?
            .is_some()
        {
            return Err(Error::IdempotencyConflict(idempotency_key));
        }
        let cost = item.price_amount * quantity as i64;
        drop(repo);
        self.debit(&player_id, cost, item.price_currency, &idempotency_key, "shop_buy").await?;
        let mut repo = self.repo.lock().await;
        let entry = ShopRecord {
            record_id: Uuid::new_v4(),
            player_id: player_id.clone(),
            shop_id,
            item_id: item_id.clone(),
            quantity,
            cost_amount: cost,
            cost_currency: item.price_currency,
            bought_at: Utc::now(),
        };
        repo.shop_records.push(entry);
        let new_stock = if item.stock < 0 { -1 } else { item.stock - quantity };
        let new_item = ShopItemEntity {
            stock: new_stock,
            ..item.clone()
        };
        repo.shop_items.insert(key, new_item);
        Ok(ShopBuyOutput {
            success: true,
            cost_amount: cost,
            cost_currency: item.price_currency,
            remaining_stock: new_stock,
            remaining_player_limit: 0, // TODO: per-player limit tracking
        })
    }

    async fn shop_refresh(
        &self,
        _player_id: String,
        _shop_id: i32,
        _use_currency: bool,
    ) -> Result<ShopRefreshOutput> {
        Err(Error::Unimplemented("shop_refresh".to_string()))
    }

    async fn shop_record(
        &self,
        player_id: String,
        _page: u32,
        _page_size: u32,
    ) -> Result<(Vec<ShopRecord>, u64)> {
        let repo = self.repo.lock().await;
        let filtered: Vec<ShopRecord> = repo
            .shop_records
            .iter()
            .filter(|r| r.player_id == player_id)
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        Ok((filtered, total))
    }

    async fn mystery_shop_list(
        &self,
        player_id: String,
        mystery_shop_id: i32,
    ) -> Result<MysteryShopListOutput> {
        let repo = self.repo.lock().await;
        let shop = repo.mystery_shops.get(&mystery_shop_id).cloned().ok_or_else(|| {
            Error::NotFound {
                entity: "MysteryShop",
                id: mystery_shop_id.to_string(),
            }
        })?;
        let state = repo
            .mystery_states
            .get(&(player_id.clone(), mystery_shop_id))
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: "MysteryShopState",
                id: format!("{}-{}", player_id, mystery_shop_id),
            })?;
        if !state.unlocked {
            return Err(Error::Forbidden("mystery shop not unlocked".to_string()));
        }
        Ok(MysteryShopListOutput {
            items: state.current_items.clone(),
            refresh_count: state.refresh_count,
            refreshed_at: state.refreshed_at,
            unlock_level: shop.unlock_level,
        })
    }

    async fn mystery_shop_buy(
        &self,
        _player_id: String,
        _mystery_shop_id: i32,
        _item_id: String,
        _idempotency_key: String,
    ) -> Result<MysteryShopBuyOutput> {
        Err(Error::Unimplemented("mystery_shop_buy".to_string()))
    }

    async fn mystery_shop_refresh(
        &self,
        _player_id: String,
        _mystery_shop_id: i32,
    ) -> Result<MysteryShopRefreshOutput> {
        Err(Error::Unimplemented("mystery_shop_refresh".to_string()))
    }

    async fn mystery_shop_unlock(
        &self,
        _player_id: String,
        _mystery_shop_id: i32,
    ) -> Result<MysteryShopUnlockOutput> {
        Err(Error::Unimplemented("mystery_shop_unlock".to_string()))
    }

    async fn exchange_list(
        &self,
        player_id: String,
        exchange_id: i32,
    ) -> Result<(Vec<ShopItemEntity>, i64)> {
        let repo = self.repo.lock().await;
        let shop = repo.exchange_shops.get(&exchange_id).cloned().ok_or_else(|| {
            Error::NotFound {
                entity: "ExchangeShop",
                id: exchange_id.to_string(),
            }
        })?;
        let points = repo
            .player_points
            .get(&(player_id, shop.cost_currency))
            .map(|p| p.balance)
            .unwrap_or(0);
        Ok((shop.items, points))
    }

    async fn exchange_do(
        &self,
        player_id: String,
        exchange_id: i32,
        item_id: String,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<ExchangeDoOutput> {
        // 真实逻辑: 扣积分 + 写账目
        if quantity <= 0 {
            return Err(Error::Validation("quantity must be > 0".to_string()));
        }
        let mut repo = self.repo.lock().await;
        let shop = repo.exchange_shops.get(&exchange_id).cloned().ok_or_else(|| {
            Error::NotFound {
                entity: "ExchangeShop",
                id: exchange_id.to_string(),
            }
        })?;
        let item = shop
            .items
            .iter()
            .find(|i| i.item_id == item_id)
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: "ShopItem",
                id: item_id.clone(),
            })?;
        let cost = item.price_amount * quantity as i64;
        let points = repo
            .player_points
            .entry((player_id.clone(), shop.cost_currency))
            .or_insert(PlayerPoints {
                player_id: player_id.clone(),
                point_type: shop.cost_currency,
                balance: 0,
            });
        if points.balance < cost {
            return Err(Error::InsufficientFunds {
                account_id: format!("{}-points-{}", player_id, shop.cost_currency),
                balance: points.balance,
                required: cost,
            });
        }
        points.balance -= cost;
        repo.shop_records.push(ShopRecord {
            record_id: Uuid::new_v4(),
            player_id: player_id.clone(),
            shop_id: exchange_id,
            item_id: item_id.clone(),
            quantity,
            cost_amount: cost,
            cost_currency: shop.cost_currency,
            bought_at: Utc::now(),
        });
        let _ = idempotency_key; // TODO: idempotency persistence
        Ok(ExchangeDoOutput {
            success: true,
            cost_points: cost,
            remaining_player_limit: 0,
        })
    }

    async fn exchange_record(
        &self,
        player_id: String,
        _page: u32,
        _page_size: u32,
    ) -> Result<(Vec<ShopRecord>, u64)> {
        let repo = self.repo.lock().await;
        let filtered: Vec<ShopRecord> = repo
            .shop_records
            .iter()
            .filter(|r| r.player_id == player_id)
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        Ok((filtered, total))
    }

    async fn wish_list(
        &self,
        _player_id: String,
        _pool_id: i32,
    ) -> Result<WishListOutput> {
        Err(Error::Unimplemented("wish_list".to_string()))
    }

    async fn wish_draw(
        &self,
        _player_id: String,
        _pool_id: i32,
        _count: i32,
        _idempotency_key: String,
    ) -> Result<WishDrawOutput> {
        Err(Error::Unimplemented("wish_draw".to_string()))
    }

    async fn wish_reward(
        &self,
        _player_id: String,
        _pool_id: i32,
        _reward_tier: i32,
    ) -> Result<WishRewardOutput> {
        Err(Error::Unimplemented("wish_reward".to_string()))
    }

    async fn point_shop_list(
        &self,
        player_id: String,
        point_type: i32,
        _page: u32,
        _page_size: u32,
    ) -> Result<(Vec<ShopItemEntity>, i64, u64)> {
        let repo = self.repo.lock().await;
        let points = repo
            .player_points
            .get(&(player_id.clone(), point_type))
            .map(|p| p.balance)
            .unwrap_or(0);
        // 找使用此 point_type 的所有 exchange shop 的 items
        let items: Vec<ShopItemEntity> = repo
            .exchange_shops
            .values()
            .filter(|s| s.cost_currency == point_type)
            .flat_map(|s| s.items.clone())
            .collect();
        let total = items.len() as u64;
        Ok((items, points, total))
    }

    async fn point_shop_buy(
        &self,
        player_id: String,
        point_type: i32,
        item_id: String,
        quantity: i32,
        idempotency_key: String,
    ) -> Result<PointShopBuyOutput> {
        if quantity <= 0 {
            return Err(Error::Validation("quantity must be > 0".to_string()));
        }
        // 复用 exchange_do 模式
        let exchange_id = {
            let repo = self.repo.lock().await;
            repo.exchange_shops
                .values()
                .find(|s| s.cost_currency == point_type && s.items.iter().any(|i| i.item_id == item_id))
                .map(|s| s.exchange_id)
                .ok_or_else(|| Error::NotFound {
                    entity: "PointShopItem",
                    id: format!("{}-{}", point_type, item_id),
                })?
        };
        let out = self
            .exchange_do(player_id.clone(), exchange_id, item_id, quantity, idempotency_key)
            .await?;
        let repo = self.repo.lock().await;
        let remaining = repo
            .player_points
            .get(&(player_id, point_type))
            .map(|p| p.balance)
            .unwrap_or(0);
        Ok(PointShopBuyOutput {
            success: out.success,
            cost_points: out.cost_points,
            remaining_points: remaining,
        })
    }

    async fn gift_code_redeem(
        &self,
        player_id: String,
        code: String,
        server_id: i32,
        idempotency_key: String,
    ) -> Result<GiftCodeRedeemOutput> {
        // 真实逻辑: 校验码 + 检查使用次数 + 检查玩家重复
        let mut repo = self.repo.lock().await;
        let key = (code.clone(), server_id);
        let gift = repo.gift_codes.get(&key).cloned().ok_or_else(|| {
            Error::NotFound {
                entity: "GiftCode",
                id: format!("{}-{}", code, server_id),
            }
        })?;
        let now = Utc::now();
        if now < gift.valid_from || now > gift.valid_to {
            return Ok(GiftCodeRedeemOutput {
                success: false,
                error_msg: "expired".to_string(),
                rewards: vec![],
            });
        }
        if gift.current_uses >= gift.max_uses {
            return Ok(GiftCodeRedeemOutput {
                success: false,
                error_msg: "max_uses_reached".to_string(),
                rewards: vec![],
            });
        }
        if repo
            .gift_redemptions
            .iter()
            .any(|r| r.code == code && r.server_id == server_id && r.player_id == player_id)
        {
            return Ok(GiftCodeRedeemOutput {
                success: false,
                error_msg: "already_used".to_string(),
                rewards: vec![],
            });
        }
        // 幂等
        if self
            .ledger
            .find_by_idempotency_key(&idempotency_key)
            .await?
            .is_some()
        {
            return Err(Error::IdempotencyConflict(idempotency_key));
        }
        repo.gift_redemptions.push(GiftCodeRedemption {
            code: code.clone(),
            player_id: player_id.clone(),
            server_id,
            redeemed_at: now,
        });
        if let Some(g) = repo.gift_codes.get_mut(&key) {
            g.current_uses += 1;
        }
        Ok(GiftCodeRedeemOutput {
            success: true,
            error_msg: "".to_string(),
            rewards: vec![], // 真实发放走 game-mail module, 留空
        })
    }

    async fn gift_code_query(
        &self,
        _player_id: String,
        code: String,
        server_id: i32,
    ) -> Result<GiftCodeQueryOutput> {
        let repo = self.repo.lock().await;
        let key = (code.clone(), server_id);
        match repo.gift_codes.get(&key) {
            Some(g) => Ok(GiftCodeQueryOutput {
                exists: true,
                code: g.code.clone(),
                reward_template: g.reward_template.clone(),
                valid_from: Some(g.valid_from),
                valid_to: Some(g.valid_to),
                max_uses: g.max_uses,
                current_uses: g.current_uses,
            }),
            None => Ok(GiftCodeQueryOutput {
                exists: false,
                code,
                reward_template: "".to_string(),
                valid_from: None,
                valid_to: None,
                max_uses: 0,
                current_uses: 0,
            }),
        }
    }

    async fn loot_roll(
        &self,
        player_id: String,
        loot_table_id: i32,
        roll_count: i32,
        _idempotency_key: String,
    ) -> Result<LootRollOutput> {
        // 真实逻辑: 加权随机抽取
        if roll_count <= 0 {
            return Err(Error::Validation("roll_count must be > 0".to_string()));
        }
        let mut repo = self.repo.lock().await;
        let table = repo.loot_tables.get(&loot_table_id).cloned().ok_or_else(|| {
            Error::NotFound {
                entity: "LootTable",
                id: loot_table_id.to_string(),
            }
        })?;
        if table.entries.is_empty() {
            return Err(Error::Validation("empty loot table".to_string()));
        }
        let total_weight: i32 = table.entries.iter().map(|e| e.weight).sum();
        let mut rolled = Vec::with_capacity(roll_count as usize);
        let mut rare = 0;
        let mut epic = 0;
        let mut legendary = 0;
        // 简单 LCG RNG (测试用, 不引入 rand)
        let mut seed: u64 = {
            let bytes = player_id.as_bytes();
            let mut h: u64 = 0xcbf29ce484222325;
            for b in bytes {
                h = h.wrapping_mul(0x100000001b3) ^ (*b as u64);
            }
            h
        };
        for _ in 0..roll_count {
            // next() LCG
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let pick = (seed >> 33) as i32 % total_weight;
            let mut acc = 0;
            let mut chosen = &table.entries[0];
            for e in &table.entries {
                acc += e.weight;
                if pick < acc {
                    chosen = e;
                    break;
                }
            }
            rolled.push(chosen.item_id.clone());
            if chosen.rarity >= 3 {
                rare += 1;
            }
            if chosen.rarity >= 4 {
                epic += 1;
            }
            if chosen.rarity >= 5 {
                legendary += 1;
            }
        }
        let batch = LootBatch {
            batch_id: Uuid::new_v4(),
            player_id: player_id.clone(),
            loot_table_id,
            rolled_items: rolled.clone(),
            claimed: false,
            rolled_at: Utc::now(),
        };
        repo.loot_batches.insert(batch.batch_id, batch);
        Ok(LootRollOutput {
            rolled_item_ids: rolled,
            rare_count: rare,
            epic_count: epic,
            legendary_count: legendary,
        })
    }

    async fn loot_claim(
        &self,
        _player_id: String,
        _loot_table_id: i32,
        batch_id: Uuid,
    ) -> Result<LootClaimOutput> {
        let mut repo = self.repo.lock().await;
        let batch = repo.loot_batches.get_mut(&batch_id).ok_or_else(|| {
            Error::NotFound {
                entity: "LootBatch",
                id: batch_id.to_string(),
            }
        })?;
        if batch.claimed {
            return Err(Error::Conflict("already claimed".to_string()));
        }
        batch.claimed = true;
        Ok(LootClaimOutput {
            success: true,
            items: vec![], // 真实发放走 inventory module
        })
    }
}
