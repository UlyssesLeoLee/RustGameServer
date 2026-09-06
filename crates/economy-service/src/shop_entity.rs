//! economy-service 商店 + 抽卡 + 限时 + 充值 实体定义
//!
//! v3 增量 (per 闪烁之光借鉴路线图 2026-09-05 Phase 2, economy + 商城 90 RPC).
//! 数据驱动反例 (per 9/4 MD §4): 9 个 holiday_* 活动 → 1 套 ActivityTemplate + 配置

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =====================================================================
// 商店类 (20 RPC)
// =====================================================================

/// 商店商品条目 (per 9/4 MD proto_134)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopItemEntity {
    pub item_id: String,
    pub sku: String,
    pub name: String,
    pub price_amount: i64,
    pub price_currency: i32, // 1=Gold, 2=Diamond, 3=Token
    pub stock: i32,          // -1 = unlimited
    pub vip_level_required: i32,
    pub level_required: i32,
    pub limit_per_player: i32,
    pub tag: String,
}

/// 商店购买记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopRecord {
    pub record_id: Uuid,
    pub player_id: String,
    pub shop_id: i32,
    pub item_id: String,
    pub quantity: i32,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub bought_at: DateTime<Utc>,
}

/// 商店刷新状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopRefreshState {
    pub player_id: String,
    pub shop_id: i32,
    pub refreshed_at: DateTime<Utc>,
    pub refresh_count: i32,
    pub next_refresh_at: DateTime<Utc>,
}

/// 神秘商店
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MysteryShop {
    pub mystery_shop_id: i32,
    pub unlock_level: i32,
    pub refresh_cost: i64,
    pub max_refresh: i32,
}

/// 神秘商店玩家状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MysteryShopState {
    pub player_id: String,
    pub mystery_shop_id: i32,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub refresh_count: i32,
    pub refreshed_at: DateTime<Utc>,
    pub current_items: Vec<ShopItemEntity>,
}

/// 兑换商店
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExchangeShop {
    pub exchange_id: i32,
    pub cost_currency: i32, // points type
    pub items: Vec<ShopItemEntity>,
}

/// 玩家积分
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerPoints {
    pub player_id: String,
    pub point_type: i32, // 1=竞技,2=公会,3=成就
    pub balance: i64,
}

/// 神格许愿池
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WishPool {
    pub pool_id: i32,
    pub name: String,
    pub items: Vec<ShopItemEntity>,
    pub free_count: i32,
    pub next_free_at: DateTime<Utc>,
}

/// 玩家许愿状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WishState {
    pub player_id: String,
    pub pool_id: i32,
    pub free_used: i32,
    pub last_free_at: DateTime<Utc>,
}

/// 礼包码
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiftCode {
    pub code: String,
    pub server_id: i32,
    pub reward_template: String,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub max_uses: i32,
    pub current_uses: i32,
}

/// 礼包码使用记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiftCodeRedemption {
    pub code: String,
    pub player_id: String,
    pub server_id: i32,
    pub redeemed_at: DateTime<Utc>,
}

/// 战利品表条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LootEntry {
    pub item_id: String,
    pub weight: i32,
    pub rarity: i32, // 1=common, 2=uncommon, 3=rare, 4=epic, 5=legendary
}

/// 战利品表
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LootTable {
    pub loot_table_id: i32,
    pub entries: Vec<LootEntry>,
}

/// 战利品批次
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LootBatch {
    pub batch_id: Uuid,
    pub player_id: String,
    pub loot_table_id: i32,
    pub rolled_items: Vec<String>,
    pub claimed: bool,
    pub rolled_at: DateTime<Utc>,
}

// =====================================================================
// 充值类 (15 RPC)
// =====================================================================

/// 充值档位
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RechargeTierEntity {
    pub tier_id: i32,
    pub sku: String,
    pub amount_cents: i64,
    pub currency_given: i32,
    pub currency_amount: i64,
    pub bonus_pct: i32,
    pub first_bonus_pct: i32,
    pub tag: String,
}

/// 充值订单
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RechargeOrder {
    pub order_id: Uuid,
    pub player_id: String,
    pub tier_id: i32,
    pub channel: i32,
    pub amount_cents: i64,
    pub currency_amount: i64,
    pub status: String, // "pending" / "paid" / "delivered" / "failed" / "refunded"
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

/// 月卡
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonthlyCard {
    pub monthly_card_id: i32,
    pub name: String,
    pub cost_cents: i64,
    pub duration_days: i32,
    pub daily_currency: i64,
    pub daily_items: Vec<String>,
}

/// 月卡玩家状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonthlyCardState {
    pub player_id: String,
    pub monthly_card_id: i32,
    pub activated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub days_claimed: i32,
    pub total_days: i32,
}

/// 首充记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirstRechargeRecord {
    pub player_id: String,
    pub claimed_tiers: Vec<i32>,
    pub recharged_tier_ids: Vec<i32>,
    pub total_spent_cents: i64,
}

/// 战力档
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PowerRank {
    pub rank_id: i32,
    pub required_power: i32,
    pub reward_items: Vec<ShopItemEntity>,
}

/// 玩家战力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerPower {
    pub player_id: String,
    pub current_power: i64,
    pub current_rank: i32,
}

/// 成长基金
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrowthFund {
    pub fund_id: i32,
    pub name: String,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub max_level: i32,
    pub tiers: Vec<FundTierEntity>,
    pub expires_at: DateTime<Utc>,
}

/// 基金档位
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FundTierEntity {
    pub level: i32,
    pub reward_amount: i64,
    pub reward_currency: i32,
}

/// 玩家基金状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FundPlayerState {
    pub player_id: String,
    pub fund_id: i32,
    pub owned: bool,
    pub activated_at: Option<DateTime<Utc>>,
    pub claimed_levels: Vec<i32>,
    pub current_xp: i32,
}

// =====================================================================
// 抽卡类 (15 RPC)
// =====================================================================

/// 抽卡池
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonPoolEntity {
    pub pool_id: i32,
    pub name: String,
    pub pool_type: i32, // 1=常驻, 2=限时, 3=联动, 4=新手
    pub cost_currency: i32,
    pub single_cost: i64,
    pub ten_cost: i64,
    pub pity_4star: i32,
    pub pity_5star: i32,
    pub featured_item_id: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub available_items: Vec<String>,
}

/// 抽卡条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonEntry {
    pub item_id: String,
    pub weight: i32,
    pub rarity: i32, // 3, 4, 5
    pub is_featured: bool,
}

/// 玩家抽卡状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonPlayerState {
    pub player_id: String,
    pub pool_id: i32,
    pub pity_count: i32,
    pub four_star_count: i32,
    pub five_star_count: i32,
    pub total_pulls: i32,
    pub free_remaining: i32,
    pub last_free_at: DateTime<Utc>,
    pub featured_guarantee_used: bool,
}

/// 抽卡结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonResultEntity {
    pub result_id: Uuid,
    pub player_id: String,
    pub pool_id: i32,
    pub drawn_items: Vec<String>,
    pub rarity_3: i32,
    pub rarity_4: i32,
    pub rarity_5: i32,
    pub new_pity_count: i32,
    pub hit_pity: bool,
    pub is_featured: bool,
    pub rolled_at: DateTime<Utc>,
}

/// 抽卡卡池
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonBox {
    pub box_id: i32,
    pub name: String,
    pub unlock_required: i32,
    pub items: Vec<ShopItemEntity>,
}

/// 玩家卡池进度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonBoxProgress {
    pub player_id: String,
    pub box_id: i32,
    pub unlock_progress: i32,
    pub unlocked: bool,
}

/// 分享奖励状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummonShareState {
    pub player_id: String,
    pub pool_id: i32,
    pub share_count: i32,
    pub last_share_target: i32,
}

// =====================================================================
// 限时/FlashSale (10 RPC)
// =====================================================================

/// 限时折扣条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlashSaleItemEntity {
    pub flash_sale_id: i32,
    pub item_id: String,
    pub name: String,
    pub price_amount: i64,
    pub original_price: i64,
    pub price_currency: i32,
    pub stock: i32,
    pub sold: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub discount_pct: i32,
    pub tag: String,
}

/// 限时玩家限制
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlashSalePlayerLimit {
    pub player_id: String,
    pub flash_sale_id: i32,
    pub bought_count: i32,
}

/// 限时玩家订阅
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlashSaleSubscription {
    pub subscribe_id: Uuid,
    pub player_id: String,
    pub flash_sale_id: i32,
    pub notify_before_secs: i32,
}

/// 限时玩家购买记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlashSaleRecordEntity {
    pub record_id: Uuid,
    pub player_id: String,
    pub flash_sale_id: i32,
    pub item_id: String,
    pub quantity: i32,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub bought_at: DateTime<Utc>,
}

// =====================================================================
// 基金/特权 (10 RPC)
// =====================================================================

/// 特权条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivilegeItemEntity {
    pub privilege_id: i32,
    pub name: String,
    pub cost_amount: i64,
    pub cost_currency: i32,
    pub duration_days: i32,
    pub perks: Vec<String>,
    pub daily_rewards: Vec<PrivilegeDailyReward>,
}

/// 特权每日奖励
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivilegeDailyReward {
    pub day: i32,
    pub reward_amount: i64,
    pub reward_currency: i32,
}

/// 特权玩家状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivilegePlayerState {
    pub player_id: String,
    pub privilege_id: i32,
    pub activated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub claimed_days: Vec<i32>,
    pub last_claim_at: DateTime<Utc>,
}

// =====================================================================
// 活动 (5 RPC) - 数据驱动模板
// =====================================================================

/// 活动类型 (per 9/4 MD §4 反例: 不写 9 套 holiday_* 重复)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Unspecified,
    Holiday,
    Signin,
    Achievement,
    Battlepass,
    Return,
    Invite,
    LevelReward,
    Daily,
    Weekly,
}

impl ActivityType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Holiday,
            2 => Self::Signin,
            3 => Self::Achievement,
            4 => Self::Battlepass,
            5 => Self::Return,
            6 => Self::Invite,
            7 => Self::LevelReward,
            8 => Self::Daily,
            9 => Self::Weekly,
            _ => Self::Unspecified,
        }
    }
    pub fn as_i32(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::Holiday => 1,
            Self::Signin => 2,
            Self::Achievement => 3,
            Self::Battlepass => 4,
            Self::Return => 5,
            Self::Invite => 6,
            Self::LevelReward => 7,
            Self::Daily => 8,
            Self::Weekly => 9,
        }
    }
}

/// 活动模板 (从配置文件加载, 数据驱动核心)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityTemplateEntity {
    pub activity_id: i32,
    pub name: String,
    pub activity_type: ActivityType,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub max_progress: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub template_json: String, // 活动配置 JSON: reward tiers, conditions, etc.
    pub enabled: bool,
}

/// 活动玩家状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityPlayerState {
    pub player_id: String,
    pub activity_id: i32,
    pub progress: i32,
    pub claimed_tiers: Vec<i32>,
    pub subscribed: bool,
    pub notify_channel: i32,
}

impl ActivityPlayerState {
    pub fn new(player_id: String, activity_id: i32) -> Self {
        Self {
            player_id,
            activity_id,
            progress: 0,
            claimed_tiers: Vec::new(),
            subscribed: false,
            notify_channel: 0,
        }
    }
}

/// 活动奖励 tier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityRewardTier {
    pub tier: i32,
    pub progress_required: i32,
    pub reward_amount: i64,
    pub reward_currency: i32,
    pub reward_items: Vec<ShopItemEntity>,
}

// =====================================================================
// 拍卖行扩展 (10 RPC)
// =====================================================================

/// 自动出价代理
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuctionAutoBid {
    pub player_id: String,
    pub auction_id: Uuid,
    pub max_amount: i64,
    pub registered_at: DateTime<Utc>,
}

/// 玩家关注
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuctionWatch {
    pub player_id: String,
    pub auction_id: Uuid,
    pub watched_at: DateTime<Utc>,
}

/// 玩家出价记录 (active)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuctionBid {
    pub bid_id: Uuid,
    pub auction_id: Uuid,
    pub bidder_id: String,
    pub amount: i64,
    pub placed_at: DateTime<Utc>,
    pub is_active: bool,
}

/// 保存的搜索
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedSearch {
    pub saved_search_id: Uuid,
    pub player_id: String,
    pub keyword: String,
    pub rarity: i32,
    pub min_price: i32,
    pub max_price: i32,
    pub notify_channel: i32,
}

// =====================================================================
// InMemory 仓库 (测试用, 复用 InMemory*Repository 模式)
// =====================================================================

/// 商店 + 抽卡 + 限时 + 充值 + 基金 + 特权 + 活动 统一 in-memory 仓库
/// 模式参考 trade_service.rs / trade_entity.rs / trade_repository.rs
#[derive(Debug, Default)]
pub struct InMemoryEconomyV3Repository {
    pub shop_items: HashMap<(i32, String), ShopItemEntity>,  // (shop_id, item_id) → item
    pub shop_records: Vec<ShopRecord>,
    pub shop_refresh_states: HashMap<(String, i32), ShopRefreshState>, // (player, shop)

    pub mystery_shops: HashMap<i32, MysteryShop>,
    pub mystery_states: HashMap<(String, i32), MysteryShopState>,

    pub exchange_shops: HashMap<i32, ExchangeShop>,
    pub player_points: HashMap<(String, i32), PlayerPoints>,

    pub wish_pools: HashMap<i32, WishPool>,
    pub wish_states: HashMap<(String, i32), WishState>,

    pub gift_codes: HashMap<(String, i32), GiftCode>, // (code, server_id)
    pub gift_redemptions: Vec<GiftCodeRedemption>,

    pub loot_tables: HashMap<i32, LootTable>,
    pub loot_batches: HashMap<Uuid, LootBatch>,

    pub recharge_tiers: HashMap<i32, RechargeTierEntity>,
    pub recharge_orders: HashMap<Uuid, RechargeOrder>,

    pub monthly_cards: HashMap<i32, MonthlyCard>,
    pub monthly_card_states: HashMap<(String, i32), MonthlyCardState>,

    pub first_recharge: HashMap<String, FirstRechargeRecord>,

    pub power_ranks: HashMap<i32, PowerRank>,
    pub player_powers: HashMap<String, PlayerPower>,

    pub growth_funds: HashMap<i32, GrowthFund>,
    pub fund_states: HashMap<(String, i32), FundPlayerState>,

    pub summon_pools: HashMap<i32, SummonPoolEntity>,
    pub summon_entries: HashMap<i32, Vec<SummonEntry>>, // pool_id → entries
    pub summon_player_states: HashMap<(String, i32), SummonPlayerState>,
    pub summon_results: Vec<SummonResultEntity>,

    pub summon_boxes: HashMap<i32, SummonBox>,
    pub summon_box_progress: HashMap<(String, i32), SummonBoxProgress>,
    pub summon_share_states: HashMap<(String, i32), SummonShareState>,

    pub flash_sale_items: HashMap<i32, FlashSaleItemEntity>,
    pub flash_sale_player_limits: HashMap<(String, i32), FlashSalePlayerLimit>,
    pub flash_sale_subscriptions: HashMap<Uuid, FlashSaleSubscription>,
    pub flash_sale_records: Vec<FlashSaleRecordEntity>,

    pub privilege_items: HashMap<i32, PrivilegeItemEntity>,
    pub privilege_player_states: HashMap<(String, i32), PrivilegePlayerState>,

    pub activity_templates: HashMap<i32, ActivityTemplateEntity>,
    pub activity_player_states: HashMap<(String, i32), ActivityPlayerState>,
    pub activity_reward_tiers: HashMap<i32, Vec<ActivityRewardTier>>, // activity_id → tiers

    pub auction_auto_bids: Vec<AuctionAutoBid>,
    pub auction_watches: Vec<AuctionWatch>,
    pub auction_bids: Vec<AuctionBid>,
    pub saved_searches: HashMap<Uuid, SavedSearch>,
}

impl InMemoryEconomyV3Repository {
    pub fn new() -> Self {
        Self::default()
    }
}
