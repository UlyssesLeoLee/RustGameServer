//! player-service 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-018 §3 玩家域数据模型）
//! - Player：账号档案（昵称、等级、vip、状态、最近登录）
//! - PlayerSession：会话（device / ip / heartbeat / expires）
//!
//! 桶 11 卡牌游戏（per DTL-038 §4.3 + §7.1 + FR-001/FR-002）：3 个 v2 entity
//! - PlayerProfile：卡牌游戏玩家档案（ranked_score / tier / total_matches / ...）
//! - Deck：卡组（owner / name / mode / slots / share_code / ...）
//! - DeckSlot：卡组单卡槽（card_id + count）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 玩家账号状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    /// 正常
    Active,
    /// 封禁
    Banned,
    /// 停用
    Disabled,
    /// 待激活
    Pending,
}

/// 玩家账号（root entity，per RGS-DTL-018 §3.1）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    /// 玩家 ID（业务主键）
    pub id: Uuid,
    /// 昵称（唯一）
    pub name: String,
    /// 等级（默认 1）
    pub level: i32,
    /// VIP 等级（0 = 非 VIP）
    pub vip_level: i32,
    /// 账号状态
    pub status: PlayerStatus,
    /// 最近登录时间（None = 从未登录）
    pub last_login_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Player {
    /// 工厂：新建玩家（默认 Active / Lv1 / VIP0）
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            level: 1,
            vip_level: 0,
            status: PlayerStatus::Active,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 玩家会话（per RGS-DTL-018 §3.2 active-active 跨服身份）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerSession {
    /// 会话 ID
    pub id: Uuid,
    /// 所属玩家 ID
    pub player_id: Uuid,
    /// 设备 ID
    pub device_id: String,
    /// 登录 IP
    pub ip: String,
    /// 登录时间
    pub login_at: DateTime<Utc>,
    /// 最近心跳时间
    pub last_heartbeat_at: DateTime<Utc>,
    /// 会话过期时间
    pub expires_at: DateTime<Utc>,
}

impl PlayerSession {
    /// 工厂：新建会话（默认 24h 过期）
    pub fn new(player_id: Uuid, device_id: String, ip: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            player_id,
            device_id,
            ip,
            login_at: now,
            last_heartbeat_at: now,
            expires_at: now + chrono::Duration::hours(24),
        }
    }

    /// 心跳刷新（更新 last_heartbeat_at + 滑动 expires_at）
    pub fn heartbeat(&mut self) {
        let now = Utc::now();
        self.last_heartbeat_at = now;
        self.expires_at = now + chrono::Duration::hours(24);
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

// ============================================================================
// v2 卡牌游戏 entity (per DTL-038 §4.3 + §7.1 + FR-001/FR-002)
// 桶 11 增量, 由 player-service 承载 deck 业务 (per DEC-038-01)
// ============================================================================

/// 卡组状态 (per DTL-038 §7.1 decks.status)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeckStatus {
    /// 草稿（编辑中, 未激活）
    Draft,
    /// 激活（可用于对战）
    Active,
    /// 归档（用户隐藏, 不参与列表默认展示）
    Archived,
}

impl DeckStatus {
    /// 转 SQL 存储字符串 (per migration v038_decks.sql CHECK 约束)
    pub fn as_str(&self) -> &'static str {
        match self {
            DeckStatus::Draft => "draft",
            DeckStatus::Active => "active",
            DeckStatus::Archived => "archived",
        }
    }

    /// 从 SQL 字符串解析
    pub fn parse(s: &str) -> Self {
        match s {
            "active" => DeckStatus::Active,
            "archived" => DeckStatus::Archived,
            _ => DeckStatus::Draft,
        }
    }
}

impl Default for DeckStatus {
    fn default() -> Self {
        DeckStatus::Draft
    }
}

/// 卡组游戏模式（per DTL-038 §7.1 decks.mode）
///
/// 数值与 proto v2 Deck.mode 对应（int32 占位，TODO(common.proto v2) 迁 GameMode 枚举）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    /// 1 = 天梯
    Ranked = 1,
    /// 2 = 休闲
    Casual = 2,
    /// 3 = 房间
    Room = 3,
    /// 4 = AI
    Ai = 4,
}

impl GameMode {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            1 => Some(GameMode::Ranked),
            2 => Some(GameMode::Casual),
            3 => Some(GameMode::Room),
            4 => Some(GameMode::Ai),
            _ => None,
        }
    }
}

/// 卡组卡槽（per DTL-038 §4.3 DeckSlot）
///
/// 业务层约束：count ∈ [1, 3]（per DTL-038 §4.3 DeckSlot.count 注释）
/// 卡组合法性约束（30-60 张, 同卡 ≤ 2 张, per DTL-038 规则引擎占位）由 service.validate_deck_slots 校验
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeckSlot {
    /// 卡牌 master ID（跨域引用, card-service 域；本 DDL 不物化 FK）
    pub card_id: String,
    /// 同卡数量 (1-3, 业务层校验)
    pub count: u32,
}

impl DeckSlot {
    /// 工厂：新建卡槽
    pub fn new(card_id: String, count: u32) -> Self {
        Self { card_id, count }
    }
}

/// 卡组（per DTL-038 §4.3 Deck + §7.1 decks 表）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deck {
    /// 卡组 ID (UUID, 主键)
    pub id: Uuid,
    /// 所属玩家 ID
    pub owner_id: Uuid,
    /// 卡组名
    pub name: String,
    /// 模式 (1=Ranked 2=Casual 3=Room 4=AI)
    pub mode: i32,
    /// 卡槽列表
    pub slots: Vec<DeckSlot>,
    /// 状态
    pub status: DeckStatus,
    /// 是否公开
    pub is_public: bool,
    /// 公开分享码（UUIDv4 string, 仅 is_public=true 时非 None）
    pub share_code: Option<String>,
    /// 点赞数
    pub like_count: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Deck {
    /// 工厂：新建卡组（默认 Draft / 私有 / 0 点赞）
    pub fn new(owner_id: Uuid, name: String, mode: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            owner_id,
            name,
            mode,
            slots: Vec::new(),
            status: DeckStatus::Draft,
            is_public: false,
            share_code: None,
            like_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// 总卡数 = Σ slots.count
    pub fn total_card_count(&self) -> u32 {
        self.slots.iter().map(|s| s.count).sum()
    }

    /// 卡组不重复卡数 = |slots|
    pub fn unique_card_count(&self) -> usize {
        self.slots.len()
    }
}

/// 玩家卡牌游戏档案（per DTL-038 §4.3 PlayerProfile + FR-001）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerProfile {
    /// 玩家 ID（与 players.id 一致）
    pub player_id: Uuid,
    /// 天梯积分
    pub ranked_score: u32,
    /// 段位（Bronze / Silver / Gold / ...）
    pub ranked_tier: String,
    /// 总对战数
    pub total_matches: u32,
    /// 总胜场
    pub total_wins: u32,
    /// 收藏数（卡牌收藏, 跨域引用 card-service 域, 本地存 count）
    pub collection_count: u32,
    /// 首选语言 (BCP-47: zh-CN / en-US / ja-JP / ...)
    pub preferred_locale: String,
}

impl PlayerProfile {
    /// 工厂：新建档案（默认 Bronze / 0 / 0 / 0 / 0 / zh-CN）
    pub fn new(player_id: Uuid) -> Self {
        Self {
            player_id,
            ranked_score: 0,
            ranked_tier: "Bronze".to_string(),
            total_matches: 0,
            total_wins: 0,
            collection_count: 0,
            preferred_locale: "zh-CN".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_new_defaults() {
        let p = Player::new("alice".to_string());
        assert_eq!(p.name, "alice");
        assert_eq!(p.level, 1);
        assert_eq!(p.vip_level, 0);
        assert_eq!(p.status, PlayerStatus::Active);
        assert!(p.last_login_at.is_none());
    }

    #[test]
    fn player_session_heartbeat_slides_expiry() {
        let player_id = Uuid::new_v4();
        let mut s = PlayerSession::new(player_id, "dev-1".to_string(), "127.0.0.1".to_string());
        let old_expiry = s.expires_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        s.heartbeat();
        assert!(s.expires_at > old_expiry);
        assert!(!s.is_expired());
    }

    // ----- v2 卡牌游戏 entity UT (per DTL-038 §4.3 + §7.1, 桶 11 增量) -----

    #[test]
    fn deck_new_defaults() {
        let owner = Uuid::new_v4();
        let d = Deck::new(owner, "MyDeck".to_string(), 1);
        assert_eq!(d.owner_id, owner);
        assert_eq!(d.name, "MyDeck");
        assert_eq!(d.mode, 1);
        assert_eq!(d.status, DeckStatus::Draft);
        assert!(!d.is_public);
        assert!(d.share_code.is_none());
        assert_eq!(d.like_count, 0);
        assert!(d.slots.is_empty());
        assert_eq!(d.total_card_count(), 0);
        assert_eq!(d.unique_card_count(), 0);
    }

    #[test]
    fn deck_total_card_count_sums_slots() {
        let mut d = Deck::new(Uuid::new_v4(), "aggro".to_string(), 1);
        d.slots.push(DeckSlot::new("card-1".to_string(), 2));
        d.slots.push(DeckSlot::new("card-2".to_string(), 3));
        d.slots.push(DeckSlot::new("card-3".to_string(), 1));
        assert_eq!(d.total_card_count(), 6);
        assert_eq!(d.unique_card_count(), 3);
    }

    #[test]
    fn deck_status_roundtrip() {
        for s in [DeckStatus::Draft, DeckStatus::Active, DeckStatus::Archived] {
            assert_eq!(DeckStatus::parse(s.as_str()), s);
        }
        // 未知字符串降级到 Draft
        assert_eq!(DeckStatus::parse("bogus"), DeckStatus::Draft);
    }

    #[test]
    fn game_mode_roundtrip() {
        for m in [GameMode::Ranked, GameMode::Casual, GameMode::Room, GameMode::Ai] {
            assert_eq!(GameMode::from_i32(m.as_i32()), Some(m));
        }
        assert_eq!(GameMode::from_i32(99), None);
        assert_eq!(GameMode::from_i32(0), None);
    }

    #[test]
    fn player_profile_new_defaults() {
        let pid = Uuid::new_v4();
        let p = PlayerProfile::new(pid);
        assert_eq!(p.player_id, pid);
        assert_eq!(p.ranked_score, 0);
        assert_eq!(p.ranked_tier, "Bronze");
        assert_eq!(p.total_matches, 0);
        assert_eq!(p.total_wins, 0);
        assert_eq!(p.collection_count, 0);
        assert_eq!(p.preferred_locale, "zh-CN");
    }
}
