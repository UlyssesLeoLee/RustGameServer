//! battle-service 域 entity 定义 (per 9/4 MD §2 + W5 简报 + 闪烁之光借鉴)
//!
//! 域核心 entity (7 域独立, per ARC-008 5 独立 DB → 7 域扩展):
//! - **BattlePhase**: 战斗阶段状态机 (Init → Prepare → RoundStart → Action → RoundEnd → End)
//! - **BattleOutcome**: 战斗结果 (Victory / Defeat / Draw / Surrender)
//! - **BattleMode**: 战斗模式 (PVE / PVP / BOSS / ENDLESS / ESCORT / EXPEDITION / GUILD_WAR / CROSS_SERVER)
//! - **PvpMode**: 6 个 PVP 变体配置 (ranked/casual/cross-server/...)  数据驱动反例
//! - **RoomType**: 房间类型 (Raid / Dungeon / Mine / Boss / Custom)
//! - **RoomBuff**: 房间 BUFF (id/name/duration/effect_json)
//! - **CompanionSlot**: 伙伴槽位 (active/hired)
//! - **EscortQuality**: 护送品质 (Common/Rare/Epic/Legendary)
//! - **MineResource**: 矿脉资源 (iron/silver/gold/crystal)
//! - **HolidayActivity**: 节日活动 (bid:93031/... 数据驱动反例)
//! - **HolidayReward**: 节日奖励
//! - **PvpRanking**: PVP 排名条目

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// 战斗状态机 (per proto_200 20000-20063 战斗生命周期)
// ============================================================================

/// 战斗阶段 (与 proto 20000 战斗初始化数据 → 20006 战斗结果 状态机对应)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BattlePhase {
    /// 未开始
    Unspecified = 0,
    /// 初始化 (proto 20000)
    Init = 1,
    /// 准备 (proto 20001)
    Prepare = 2,
    /// 回合开始 (proto 20002)
    RoundStart = 3,
    /// 行动中 (proto 20004 技能 / 20005 客户端播放完成)
    Action = 4,
    /// 回合结束
    RoundEnd = 5,
    /// 战斗结束 (proto 20006 / 20008 / 20033)
    End = 6,
    /// 退出
    Exited = 7,
}

impl BattlePhase {
    /// 业务校验：合法状态转移 (per 战斗引擎)
    pub fn can_transition_to(self, next: BattlePhase) -> bool {
        use BattlePhase::*;
        match (self, next) {
            (Unspecified, _) => false,
            (Exited, _) => false,
            (End, Exited) => true,
            (End, _) => false,
            (Init, Prepare | End | Exited) => true,
            (Prepare, RoundStart | Action | End | Exited) => true,
            (RoundStart, Action | RoundEnd | End | Exited) => true,
            (Action, RoundEnd | Action | End | Exited) => true,
            (RoundEnd, RoundStart | Action | End | Exited) => true,
            _ => false,
        }
    }
}

impl Default for BattlePhase {
    fn default() -> Self {
        BattlePhase::Unspecified
    }
}

/// 战斗结果
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BattleOutcome {
    Unspecified = 0,
    Victory = 1,
    Defeat = 2,
    Draw = 3,
    Surrender = 4,
}

impl Default for BattleOutcome {
    fn default() -> Self {
        BattleOutcome::Unspecified
    }
}

/// 战斗模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BattleMode {
    Unspecified = 0,
    Pve = 1,
    Pvp = 2,
    Boss = 3,
    Endless = 4,
    Escort = 5,
    Expedition = 6,
    GuildWar = 7,
    CrossServer = 8,
    Room = 9,
}

impl Default for BattleMode {
    fn default() -> Self {
        BattleMode::Unspecified
    }
}

// ============================================================================
// PVP 数据驱动 (per 9/4 MD §4 反例: 6 个 PVP 变体 1 套 + 配置)
// ============================================================================

/// PVP 变体模式 (6 个变体共享同一份 PvPService, 仅配置不同)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PvpMode {
    /// 排位赛 (ranked ladder)
    Ranked = 1,
    /// 休闲赛
    Casual = 2,
    /// 跨服赛 (per proto_243.erl)
    CrossServer = 3,
    /// 冠军赛 (per proto_202 20250+ 赛程)
    Championship = 4,
    /// 英雄殿 (per proto_243 24311 大神风采)
    HeroHall = 5,
    /// 友谊赛 / 切磋 (per proto_200 20014 切磋)
    Friendly = 6,
}

impl PvpMode {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(PvpMode::Ranked),
            2 => Some(PvpMode::Casual),
            3 => Some(PvpMode::CrossServer),
            4 => Some(PvpMode::Championship),
            5 => Some(PvpMode::HeroHall),
            6 => Some(PvpMode::Friendly),
            _ => None,
        }
    }
    pub fn as_code(&self) -> i32 {
        *self as i32
    }
    /// 每种模式每日挑战次数上限 (per 业务规则)
    pub fn daily_challenge_limit(&self) -> u32 {
        match self {
            PvpMode::Ranked => 10,
            PvpMode::Casual => 20,
            PvpMode::CrossServer => 5,
            PvpMode::Championship => 3,
            PvpMode::HeroHall => 1,
            PvpMode::Friendly => 999, // 切磋无次数限制
        }
    }
    /// 是否启用排位分
    pub fn uses_rank_score(&self) -> bool {
        matches!(self, PvpMode::Ranked | PvpMode::CrossServer)
    }
}

/// PVP 排名条目 (per proto_202 20220-20223 + proto_243 24308-24309)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PvpRanking {
    pub rank: u32,
    pub player_id: String,
    pub player_name: String,
    pub rank_score: u32,
    pub pvp_mode: PvpMode,
    pub server_id: u32,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// 房间 / 矿脉 (per proto_206 房间战 + 矿脉)
// ============================================================================

/// 房间类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RoomType {
    Unspecified = 0,
    /// 副本房间
    Instance = 1,
    /// 矿脉房间
    Mine = 2,
    /// 神装副本
    Holy = 3,
    /// BOSS 房
    Boss = 4,
    /// 自定义
    Custom = 5,
}

impl Default for RoomType {
    fn default() -> Self {
        RoomType::Unspecified
    }
}

/// 房间 BUFF (per proto_206 20601 BUFF信息)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomBuff {
    pub buff_id: u32,
    pub name: String,
    pub duration_turns: u32,
    pub effect_json: String,
}

/// 矿脉资源 (per proto_206 20640-20660 矿脉基础协议)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MineResource {
    Unspecified = 0,
    Iron = 1,
    Silver = 2,
    Gold = 3,
    Crystal = 4,
}

impl MineResource {
    pub fn base_yield(&self) -> u32 {
        match self {
            MineResource::Iron => 100,
            MineResource::Silver => 50,
            MineResource::Gold => 20,
            MineResource::Crystal => 5,
            _ => 0,
        }
    }
}

/// 伙伴槽位 (per proto_206 20604-20605 + proto_239 + proto_244 派出/雇佣)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanionSlot {
    pub slot_id: u32,
    pub companion_id: String,
    pub companion_name: String,
    pub level: u32,
    pub stars: u32,
    pub element: String,
    pub source: CompanionSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CompanionSource {
    Unspecified = 0,
    /// 玩家自己的伙伴
    SelfOwned = 1,
    /// 雇佣他人 (per proto_239 23909 雇佣伙伴)
    Hired = 2,
    /// 支援 (per proto_244 24405-24406 支援)
    Supported = 3,
}

impl Default for CompanionSource {
    fn default() -> Self {
        CompanionSource::Unspecified
    }
}

// ============================================================================
// 护送 + 节日 (数据驱动反例)
// ============================================================================

/// 护送品质 (per proto_240 24001 刷新护送品质)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EscortQuality {
    Unspecified = 0,
    Common = 1,
    Uncommon = 2,
    Rare = 3,
    Epic = 4,
    Legendary = 5,
}

impl EscortQuality {
    pub fn reward_multiplier(&self) -> f64 {
        match self {
            EscortQuality::Unspecified => 1.0,
            EscortQuality::Common => 1.0,
            EscortQuality::Uncommon => 1.5,
            EscortQuality::Rare => 2.0,
            EscortQuality::Epic => 3.0,
            EscortQuality::Legendary => 5.0,
        }
    }
    pub fn refresh_cost(&self) -> u32 {
        match self {
            EscortQuality::Unspecified => 0,
            EscortQuality::Common => 0,
            EscortQuality::Uncommon => 10,
            EscortQuality::Rare => 50,
            EscortQuality::Epic => 200,
            EscortQuality::Legendary => 1000,
        }
    }
}

/// 节日活动 (per 9/4 MD §4 反例: 9 个 holiday_* 抽象为统一接口 + 数据驱动配置)
///
/// 1 个 HolidayActivity 实例 = 1 份活动配置, 通过 activity_id 路由不同活动
/// (e.g. bid:93031 元宵冒险1 / bid:93032 元宵冒险2 / bid:93033 元宵冒险3)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HolidayActivity {
    pub activity_id: String,
    pub activity_name: String,
    pub opens_at_ms: i64,
    pub closes_at_ms: i64,
    /// 业务配置 JSON (per 9/4 MD §4 数据驱动, 不为每个活动重写 1 套)
    /// 例如: {"task_type":"daily_kill","target":10,"reward_id":12345}
    pub config_json: String,
}

impl HolidayActivity {
    /// 业务校验：当前是否在活动窗口内
    pub fn is_in_window(&self, now_ms: i64) -> bool {
        now_ms >= self.opens_at_ms && now_ms < self.closes_at_ms
    }
}

/// 节日活动奖励条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HolidayReward {
    pub activity_id: String,
    pub item_id: i32,
    pub count: i32,
    pub description: String,
    pub claimed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_phase_default_is_unspecified() {
        assert_eq!(BattlePhase::default(), BattlePhase::Unspecified);
    }

    #[test]
    fn battle_phase_can_transition_init_to_prepare() {
        assert!(BattlePhase::Init.can_transition_to(BattlePhase::Prepare));
    }

    #[test]
    fn battle_phase_cannot_skip_prepare() {
        assert!(!BattlePhase::Init.can_transition_to(BattlePhase::Action));
    }

    #[test]
    fn battle_phase_end_only_to_exited() {
        assert!(BattlePhase::End.can_transition_to(BattlePhase::Exited));
        assert!(!BattlePhase::End.can_transition_to(BattlePhase::Action));
    }

    #[test]
    fn battle_phase_round_end_to_round_start_allowed() {
        assert!(BattlePhase::RoundEnd.can_transition_to(BattlePhase::RoundStart));
    }

    #[test]
    fn battle_phase_action_to_action_self_loop() {
        // 多 action 顺序执行: 同一 phase 内的 action 累积
        assert!(BattlePhase::Action.can_transition_to(BattlePhase::Action));
    }

    #[test]
    fn battle_phase_unspecified_cannot_transition() {
        assert!(!BattlePhase::Unspecified.can_transition_to(BattlePhase::Init));
    }

    #[test]
    fn battle_phase_exited_is_terminal() {
        for next in [
            BattlePhase::Init,
            BattlePhase::Prepare,
            BattlePhase::RoundStart,
            BattlePhase::Action,
            BattlePhase::End,
        ] {
            assert!(!BattlePhase::Exited.can_transition_to(next));
        }
    }

    #[test]
    fn pvp_mode_roundtrip() {
        for m in [
            PvpMode::Ranked,
            PvpMode::Casual,
            PvpMode::CrossServer,
            PvpMode::Championship,
            PvpMode::HeroHall,
            PvpMode::Friendly,
        ] {
            assert_eq!(PvpMode::from_code(m.as_code()), Some(m));
        }
    }

    #[test]
    fn pvp_mode_from_invalid_code_is_none() {
        assert_eq!(PvpMode::from_code(99), None);
    }

    #[test]
    fn pvp_mode_daily_limit() {
        assert_eq!(PvpMode::Ranked.daily_challenge_limit(), 10);
        assert_eq!(PvpMode::Friendly.daily_challenge_limit(), 999);
    }

    #[test]
    fn pvp_mode_uses_rank_score() {
        assert!(PvpMode::Ranked.uses_rank_score());
        assert!(PvpMode::CrossServer.uses_rank_score());
        assert!(!PvpMode::Casual.uses_rank_score());
    }

    #[test]
    fn escort_quality_reward_multiplier() {
        assert_eq!(EscortQuality::Common.reward_multiplier(), 1.0);
        assert_eq!(EscortQuality::Legendary.reward_multiplier(), 5.0);
    }

    #[test]
    fn escort_quality_refresh_cost() {
        assert_eq!(EscortQuality::Common.refresh_cost(), 0);
        assert_eq!(EscortQuality::Legendary.refresh_cost(), 1000);
    }

    #[test]
    fn mine_resource_base_yield() {
        assert_eq!(MineResource::Iron.base_yield(), 100);
        assert_eq!(MineResource::Crystal.base_yield(), 5);
    }

    #[test]
    fn holiday_activity_in_window() {
        let act = HolidayActivity {
            activity_id: "93031".to_string(),
            activity_name: "元宵冒险1".to_string(),
            opens_at_ms: 1000,
            closes_at_ms: 2000,
            config_json: "{}".to_string(),
        };
        assert!(act.is_in_window(1500));
        assert!(!act.is_in_window(500));
        assert!(!act.is_in_window(2500));
    }
}
