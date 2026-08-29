//! card-service 域 entity 定义
//!
//! 桶 10 (card catalog) 实装 (per RGS-DTL-038 §4.4 + §7.1 + FR-003/FR-006/BR-003):
//! - **Card**: 卡牌主数据 (catalog, 静态 / 慢变)
//! - **CardSeries**: 卡包 / 系列 (含 drop_table / price / status)
//! - **CardInstance**: 玩家收藏的卡牌实例 (动态, 玩家强属性)
//! - **DropTable**: 抽卡概率表 (per DEC-038-06 强制公开)
//! - **DropEntry**: 抽卡概率条目 (按 rarity 出卡)
//!
//! 业务约束：
//! - Card / CardSeries 由运营配置 (write-once, 慢变, 缓存友好)
//! - CardInstance 由抽卡 / 交易 / GM 补偿产生 (动态, 玩家强属性)
//! - DropTable 每次调整 version++，与历史 snapshot 保留一致 (per SR-001)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::Result;

// ============================================================================
// 枚举 (与 proto v1 common.v1 CardRarity / CardType / Status / CurrencyType 对应)
// ============================================================================

/// 卡牌稀有度 (与 common.proto CardRarity 数值一一对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CardRarity {
    /// 0 = 未指定
    Unspecified = 0,
    /// 1 = N 普通
    Common = 1,
    /// 2 = R 罕见
    Uncommon = 2,
    /// 3 = SR 稀有
    Rare = 3,
    /// 4 = SSR 史诗
    Epic = 4,
    /// 5 = UR 传说
    Legendary = 5,
}

impl CardRarity {
    /// proto int32 -> enum (Unknown -> Unspecified)
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => CardRarity::Common,
            2 => CardRarity::Uncommon,
            3 => CardRarity::Rare,
            4 => CardRarity::Epic,
            5 => CardRarity::Legendary,
            _ => CardRarity::Unspecified,
        }
    }

    /// enum -> proto int32
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 业务展示名
    pub fn display_name(&self) -> &'static str {
        match self {
            CardRarity::Unspecified => "Unknown",
            CardRarity::Common => "Common",
            CardRarity::Uncommon => "Uncommon",
            CardRarity::Rare => "Rare",
            CardRarity::Epic => "Epic",
            CardRarity::Legendary => "Legendary",
        }
    }
}

/// 卡牌类型 (与 common.proto CardType 数值一一对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    /// 0 = 未指定
    Unspecified = 0,
    /// 1 = 生物
    Creature = 1,
    /// 2 = 法术
    Spell = 2,
    /// 3 = 装备
    Equipment = 3,
    /// 4 = 地
    Land = 4,
    /// 5 = 陷阱
    Trap = 5,
    /// 6 = 英雄
    Hero = 6,
}

impl CardType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => CardType::Creature,
            2 => CardType::Spell,
            3 => CardType::Equipment,
            4 => CardType::Land,
            5 => CardType::Trap,
            6 => CardType::Hero,
            _ => CardType::Unspecified,
        }
    }

    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// 卡包 / 系列状态 (与 common.proto Status 数值一一对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CardSeriesStatus {
    /// 0 = 未指定
    Unspecified = 0,
    /// 1 = 活跃 (可抽)
    Ok = 1,
    /// 2 = 待发布
    Pending = 2,
    /// 3 = 失败 (运营禁用)
    Failed = 3,
    /// 4 = 取消
    Cancelled = 4,
}

impl CardSeriesStatus {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => CardSeriesStatus::Ok,
            2 => CardSeriesStatus::Pending,
            3 => CardSeriesStatus::Failed,
            4 => CardSeriesStatus::Cancelled,
            _ => CardSeriesStatus::Unspecified,
        }
    }

    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 是否可抽卡（per DEC-038-06 强制：仅 Ok 状态可抽）
    pub fn is_packable(&self) -> bool {
        matches!(self, CardSeriesStatus::Ok)
    }
}

impl Default for CardSeriesStatus {
    fn default() -> Self {
        CardSeriesStatus::Ok
    }
}

/// 货币类型 (与 common.proto CurrencyType 数值一一对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CurrencyType {
    /// 0 = 未指定
    Unspecified = 0,
    /// 1 = 软通
    Soft = 1,
    /// 2 = 硬通
    Hard = 2,
    /// 3 = 卡牌价值
    CardValue = 3,
}

impl CurrencyType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => CurrencyType::Soft,
            2 => CurrencyType::Hard,
            3 => CurrencyType::CardValue,
            _ => CurrencyType::Unspecified,
        }
    }

    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// 卡牌实例来源 (与 card.proto CardInstance.Source 数值一一对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CardInstanceSource {
    /// 0 = 未指定
    Unspecified = 0,
    /// 1 = 开包
    Pack = 1,
    /// 2 = 任务奖励
    Reward = 2,
    /// 3 = 交易
    Trade = 3,
    /// 4 = GM 补偿
    GmGrant = 4,
    /// 5 = 活动
    Event = 5,
}

impl CardInstanceSource {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => CardInstanceSource::Pack,
            2 => CardInstanceSource::Reward,
            3 => CardInstanceSource::Trade,
            4 => CardInstanceSource::GmGrant,
            5 => CardInstanceSource::Event,
            _ => CardInstanceSource::Unspecified,
        }
    }

    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

// ============================================================================
// Card (catalog, 静态)
// ============================================================================

/// 卡牌属性 (与 proto CardStats 对应: attack / health / mana / custom map)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CardStats {
    pub attack: u32,
    pub health: u32,
    pub mana: u32,
    /// 扩展属性 (key -> value)
    pub custom: HashMap<String, i32>,
}

/// 卡牌 (catalog 静态, per DTL-038 §7.1 cards 表)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
    /// 卡牌 master ID (业务主键, e.g. "card_001")
    pub card_id: String,
    /// 所属卡包 / 系列
    pub series_id: String,
    /// 卡牌名 (默认 zh-CN)
    pub name_default: String,
    /// 名称多语言 (locale -> text), 桶 14 i18n-service 实装前可空
    pub name_i18n: HashMap<String, String>,
    /// 卡牌类型
    pub card_type: CardType,
    /// 稀有度
    pub rarity: CardRarity,
    /// 基础费用
    pub base_cost: u32,
    /// 描述多语言
    pub description_i18n: HashMap<String, String>,
    /// 效果引用 (业务层 game-logic 解析, per DTL-038 §9.1 P2 TODO)
    pub effect_ref: String,
    /// 卡牌属性
    pub stats: CardStats,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Card {
    /// 工厂：新建卡牌 (id / series / name / type / rarity 必填, 其它 default)
    pub fn new(
        card_id: String,
        series_id: String,
        name_default: String,
        card_type: CardType,
        rarity: CardRarity,
    ) -> Self {
        let now = Utc::now();
        Self {
            card_id,
            series_id,
            name_default,
            name_i18n: HashMap::new(),
            card_type,
            rarity,
            base_cost: 0,
            description_i18n: HashMap::new(),
            effect_ref: String::new(),
            stats: CardStats::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

// ============================================================================
// DropTable / DropEntry (抽卡概率, per DEC-038-06 强制公开)
// ============================================================================

/// 抽卡概率条目 (按 rarity)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropEntry {
    pub rarity: CardRarity,
    /// 出几张 (per pack_size, 通常 1)
    pub count: u32,
    /// 概率 (0.0 - 1.0)
    pub probability: f64,
    /// 单卡 ID (可选, 用于保底 / 定向 UP)
    pub card_id: Option<String>,
}

/// 抽卡概率表 (per DTL-038 §4.4 + §6.1 + DEC-038-06)
///
/// 业务规则：
/// - 每次调整 version++，与历史 snapshot 保留一致 (per SR-001)
/// - 同一系列下，所有 DropEntry.probability 之和应 ≤ 1.0
///   (允许剩余概率 = 抽不到, 用于保底补偿)
/// - OpenPackResponse 必须返回 snapshot (per DEC-038-06 强制公开)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropTable {
    /// 每次调整递增 (单调递增)
    pub version: u32,
    /// 快照时间
    pub snapshot_at: DateTime<Utc>,
    /// 概率条目列表
    pub entries: Vec<DropEntry>,
}

impl DropTable {
    /// 工厂：新建概率表 (version=1, 当前时间)
    pub fn new(entries: Vec<DropEntry>) -> Self {
        Self {
            version: 1,
            snapshot_at: Utc::now(),
            entries,
        }
    }

    /// 业务校验：所有 probability 之和 ≤ 1.0
    pub fn validate(&self) -> Result<()> {
        let sum: f64 = self.entries.iter().map(|e| e.probability).sum();
        if sum > 1.0 + f64::EPSILON {
            return Err(crate::Error::Validation(format!(
                "drop_table probability sum {} > 1.0",
                sum
            )));
        }
        if self.entries.is_empty() {
            return Err(crate::Error::Validation(
                "drop_table must have at least one entry".to_string(),
            ));
        }
        Ok(())
    }

    /// 按概率抽样 (业务层抽卡算法, per DTL-038 §6.1)
    ///
    /// 业务规则：0.0 <= rand < sum, 落入 entry.probability 累计区间
    /// 不在 entry 范围内的随机数 -> 视为 "未抽中" (返 None, 用于业务层补抽 / 保底)
    pub fn sample(&self) -> Option<&DropEntry> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        // 业务简化版：使用确定性 hash 作为随机源 (便于 IT 复现)
        // 生产环境应替换为 rand crate (per DTL-038 §6.1 业务层, 当前桶 10 占位)
        let mut hasher = DefaultHasher::new();
        self.version.hash(&mut hasher);
        Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
        let r = (hasher.finish() as f64) / (u64::MAX as f64);
        let mut acc = 0.0_f64;
        for e in &self.entries {
            acc += e.probability;
            if r < acc {
                return Some(e);
            }
        }
        None
    }
}

// ============================================================================
// CardSeries (卡包 / 系列)
// ============================================================================

/// 货币 (与 proto Currency 对应: type + amount)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Currency {
    pub currency_type: CurrencyType,
    pub amount: i64,
}

/// 卡包 / 系列 (per DTL-038 §7.1 card_series 表)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardSeries {
    /// 系列 ID (业务主键, e.g. "series_001")
    pub series_id: String,
    /// 系列名 (默认 zh-CN)
    pub name_default: String,
    /// 名称多语言
    pub name_i18n: HashMap<String, String>,
    /// 一包几张
    pub pack_size: u32,
    /// 抽卡概率表 (per DEC-038-06 强制公开)
    pub drop_table: DropTable,
    /// 价格
    pub price: Currency,
    /// 发布时间
    pub released_at: DateTime<Utc>,
    /// 状态
    pub status: CardSeriesStatus,
}

impl CardSeries {
    /// 工厂：新建系列 (默认 pack_size=5, status=Ok)
    pub fn new(series_id: String, name_default: String, pack_size: u32) -> Self {
        Self {
            series_id,
            name_default,
            name_i18n: HashMap::new(),
            pack_size,
            drop_table: DropTable::new(Vec::new()),
            price: Currency {
                currency_type: CurrencyType::Soft,
                amount: 0,
            },
            released_at: Utc::now(),
            status: CardSeriesStatus::Ok,
        }
    }

    /// 业务校验：可抽卡 (per DEC-038-06)
    pub fn ensure_packable(&self) -> Result<()> {
        if !self.status.is_packable() {
            return Err(crate::Error::Conflict(format!(
                "card_series {} status {:?} not packable",
                self.series_id, self.status
            )));
        }
        if self.pack_size == 0 {
            return Err(crate::Error::Validation(format!(
                "card_series {} pack_size = 0",
                self.series_id
            )));
        }
        self.drop_table.validate()?;
        Ok(())
    }
}

// ============================================================================
// CardInstance (玩家收藏, 动态)
// ============================================================================

/// 卡牌实例 (玩家收藏, per DTL-038 §7.1 card_instances 表)
///
/// 业务含义：玩家真正拥有的卡牌 (不同于 Card 主数据)
/// - instance_id: UUID, 全局唯一
/// - card_id: 静态 card.id (跨域引用)
/// - owner_id: 玩家 ID
/// - source: 来源 (开包 / 任务 / 交易 / GM / 活动)
/// - level: 等级 (1-N, 升级系统, 业务层)
/// - attrs: 个性化属性 (强化 / 精炼)
/// - tradable: 可交易
/// - locked: 锁定中
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardInstance {
    pub instance_id: Uuid,
    pub card_id: String,
    pub owner_id: Uuid,
    pub acquired_at: DateTime<Utc>,
    pub source: CardInstanceSource,
    pub level: u32,
    pub attrs: HashMap<String, i32>,
    pub tradable: bool,
    pub locked: bool,
}

impl CardInstance {
    /// 工厂：新建卡牌实例 (默认 level=1, tradable=true, locked=false)
    pub fn new(card_id: String, owner_id: Uuid, source: CardInstanceSource) -> Self {
        Self {
            instance_id: Uuid::new_v4(),
            card_id,
            owner_id,
            acquired_at: Utc::now(),
            source,
            level: 1,
            attrs: HashMap::new(),
            tradable: true,
            locked: false,
        }
    }

    /// 业务校验：可删除 (未 locked)
    pub fn ensure_removable(&self) -> Result<()> {
        if self.locked {
            return Err(crate::Error::Conflict(format!(
                "card_instance {} is locked",
                self.instance_id
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_new_defaults() {
        let c = Card::new(
            "card_001".to_string(),
            "series_001".to_string(),
            "Fire Dragon".to_string(),
            CardType::Creature,
            CardRarity::Legendary,
        );
        assert_eq!(c.card_id, "card_001");
        assert_eq!(c.series_id, "series_001");
        assert_eq!(c.name_default, "Fire Dragon");
        assert_eq!(c.card_type, CardType::Creature);
        assert_eq!(c.rarity, CardRarity::Legendary);
        assert_eq!(c.base_cost, 0);
        assert!(c.effect_ref.is_empty());
        assert_eq!(c.stats.attack, 0);
        assert_eq!(c.stats.health, 0);
        assert_eq!(c.stats.mana, 0);
        assert!(c.name_i18n.is_empty());
    }

    #[test]
    fn card_rarity_roundtrip() {
        for r in [
            CardRarity::Common,
            CardRarity::Uncommon,
            CardRarity::Rare,
            CardRarity::Epic,
            CardRarity::Legendary,
        ] {
            assert_eq!(CardRarity::from_i32(r.as_i32()), r);
        }
        assert_eq!(CardRarity::from_i32(99), CardRarity::Unspecified);
    }

    #[test]
    fn card_type_roundtrip() {
        for t in [
            CardType::Creature,
            CardType::Spell,
            CardType::Equipment,
            CardType::Land,
            CardType::Trap,
            CardType::Hero,
        ] {
            assert_eq!(CardType::from_i32(t.as_i32()), t);
        }
        assert_eq!(CardType::from_i32(99), CardType::Unspecified);
    }

    #[test]
    fn card_series_status_packable() {
        assert!(CardSeriesStatus::Ok.is_packable());
        assert!(!CardSeriesStatus::Pending.is_packable());
        assert!(!CardSeriesStatus::Failed.is_packable());
        assert!(!CardSeriesStatus::Cancelled.is_packable());
    }

    #[test]
    fn card_instance_source_roundtrip() {
        for s in [
            CardInstanceSource::Pack,
            CardInstanceSource::Reward,
            CardInstanceSource::Trade,
            CardInstanceSource::GmGrant,
            CardInstanceSource::Event,
        ] {
            assert_eq!(CardInstanceSource::from_i32(s.as_i32()), s);
        }
    }

    #[test]
    fn card_instance_new_defaults() {
        let owner = Uuid::new_v4();
        let i = CardInstance::new("card_001".to_string(), owner, CardInstanceSource::Pack);
        assert_eq!(i.card_id, "card_001");
        assert_eq!(i.owner_id, owner);
        assert_eq!(i.source, CardInstanceSource::Pack);
        assert_eq!(i.level, 1);
        assert!(i.tradable);
        assert!(!i.locked);
        assert!(i.attrs.is_empty());
    }

    #[test]
    fn card_instance_ensure_removable() {
        let mut i = CardInstance::new("c".to_string(), Uuid::new_v4(), CardInstanceSource::Pack);
        assert!(i.ensure_removable().is_ok());
        i.locked = true;
        assert!(i.ensure_removable().is_err());
    }

    #[test]
    fn drop_table_validate_empty() {
        let dt = DropTable::new(Vec::new());
        assert!(dt.validate().is_err());
    }

    #[test]
    fn drop_table_validate_sum_exceeds_one() {
        let dt = DropTable::new(vec![
            DropEntry {
                rarity: CardRarity::Common,
                count: 1,
                probability: 0.7,
                card_id: None,
            },
            DropEntry {
                rarity: CardRarity::Rare,
                count: 1,
                probability: 0.5,
                card_id: None,
            },
        ]);
        assert!(dt.validate().is_err());
    }

    #[test]
    fn drop_table_validate_ok() {
        let dt = DropTable::new(vec![
            DropEntry {
                rarity: CardRarity::Common,
                count: 4,
                probability: 0.7,
                card_id: None,
            },
            DropEntry {
                rarity: CardRarity::Rare,
                count: 1,
                probability: 0.2,
                card_id: None,
            },
            DropEntry {
                rarity: CardRarity::Legendary,
                count: 1,
                probability: 0.05,
                card_id: None,
            },
        ]);
        assert!(dt.validate().is_ok());
    }

    #[test]
    fn card_series_new_defaults() {
        let s = CardSeries::new("series_001".to_string(), "Starter".to_string(), 5);
        assert_eq!(s.series_id, "series_001");
        assert_eq!(s.pack_size, 5);
        assert_eq!(s.status, CardSeriesStatus::Ok);
        assert!(s.drop_table.entries.is_empty());
    }

    #[test]
    fn card_series_ensure_packable_failed_status() {
        let mut s = CardSeries::new("series_x".to_string(), "x".to_string(), 5);
        s.status = CardSeriesStatus::Cancelled;
        assert!(s.ensure_packable().is_err());
    }

    #[test]
    fn card_series_ensure_packable_zero_pack_size() {
        let mut s = CardSeries::new("series_x".to_string(), "x".to_string(), 0);
        s.drop_table = DropTable::new(vec![DropEntry {
            rarity: CardRarity::Common,
            count: 1,
            probability: 1.0,
            card_id: None,
        }]);
        assert!(s.ensure_packable().is_err());
    }
}
