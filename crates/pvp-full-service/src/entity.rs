//! pvp-full-service 域 entity
//!
//! 复用 shared_platform::PvpMode (6-12 变体) + PvpConfig
//! W8 L18: 加 RankEntry / SeasonEntry / SeasonPass / MatchEntry 4 类存储 (23 RPC 共享)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use shared_platform::data_driven::{PvpConfig, PvpMode, PvpModeConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PvpMatch {
    pub match_id: Uuid,
    pub mode: PvpMode,
    pub player_a: Uuid,
    pub player_b: Uuid,
    pub started_at_ms: i64,
    pub finished: bool,
    pub winner: Option<Uuid>,
}

impl PvpMatch {
    pub fn new(mode: PvpMode, player_a: Uuid, player_b: Uuid) -> Self {
        Self {
            match_id: Uuid::new_v4(),
            mode,
            player_a,
            player_b,
            started_at_ms: chrono::Utc::now().timestamp_millis(),
            finished: false,
            winner: None,
        }
    }

    pub fn finish(&mut self, winner: Uuid) {
        self.finished = true;
        self.winner = Some(winner);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerPvpState {
    pub player_id: Uuid,
    pub mode: PvpMode,
    pub score: i32,
    pub daily_used: u32,
    pub wins: u32,
    pub losses: u32,
}

impl PlayerPvpState {
    pub fn new(player_id: Uuid, mode: PvpMode) -> Self {
        Self { player_id, mode, score: 1000, daily_used: 0, wins: 0, losses: 0 }
    }

    pub fn apply_result(&mut self, won: bool, mode_cfg: &PvpModeConfig) {
        if won {
            self.wins += 1;
            if mode_cfg.uses_rank_score {
                self.score = (self.score + 25).min(5000);
            }
        } else {
            self.losses += 1;
            if mode_cfg.uses_rank_score {
                self.score = (self.score - 20).max(0);
            }
        }
        self.daily_used = self.daily_used.saturating_add(1);
    }
}

/// 段位枚举 (1 套代码 + 段位阈值表, 不为每 tier 复制代码)
/// Bronze / Silver / Gold / Platinum / Diamond / Master / Grandmaster
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Master,
    Grandmaster,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Bronze => "bronze",
            Tier::Silver => "silver",
            Tier::Gold => "gold",
            Tier::Platinum => "platinum",
            Tier::Diamond => "diamond",
            Tier::Master => "master",
            Tier::Grandmaster => "grandmaster",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bronze" => Some(Tier::Bronze),
            "silver" => Some(Tier::Silver),
            "gold" => Some(Tier::Gold),
            "platinum" => Some(Tier::Platinum),
            "diamond" => Some(Tier::Diamond),
            "master" => Some(Tier::Master),
            "grandmaster" => Some(Tier::Grandmaster),
            _ => None,
        }
    }
    pub const ALL: [Tier; 7] = [
        Tier::Bronze, Tier::Silver, Tier::Gold, Tier::Platinum,
        Tier::Diamond, Tier::Master, Tier::Grandmaster,
    ];

    /// 根据分数定位 tier (1 套规则, 7 段位共用)
    pub fn from_score(score: i32) -> Tier {
        match score {
            s if s >= 4500 => Tier::Grandmaster,
            s if s >= 4000 => Tier::Master,
            s if s >= 3500 => Tier::Diamond,
            s if s >= 3000 => Tier::Platinum,
            s if s >= 2500 => Tier::Gold,
            s if s >= 2000 => Tier::Silver,
            _ => Tier::Bronze,
        }
    }

    /// 段位最小/最大分数 + 每日加成
    pub fn score_range(&self) -> (i32, i32) {
        match self {
            Tier::Bronze => (0, 1999),
            Tier::Silver => (2000, 2499),
            Tier::Gold => (2500, 2999),
            Tier::Platinum => (3000, 3499),
            Tier::Diamond => (3500, 3999),
            Tier::Master => (4000, 4499),
            Tier::Grandmaster => (4500, 5000),
        }
    }
    pub fn daily_bonus(&self) -> i32 {
        match self {
            Tier::Bronze => 5,
            Tier::Silver => 10,
            Tier::Gold => 15,
            Tier::Platinum => 20,
            Tier::Diamond => 30,
            Tier::Master => 40,
            Tier::Grandmaster => 50,
        }
    }
}

/// 排位历史 1 条
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankHistoryEntry {
    pub timestamp_ms: i64,
    pub score: i32,
    pub tier: Tier,
}

/// 赛季概要
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeasonSummary {
    pub season_id: String,
    pub mode: PvpMode,
    pub starts_at_ms: i64,
    pub ends_at_ms: i64,
    pub active: bool,
}

/// 赛季奖励池
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeasonRewardPool {
    pub total_items: u32,
    pub rare_items: u32,
    pub total_gold: u32,
}

/// 赛季通行证
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeasonPass {
    pub player_id: Uuid,
    pub season_id: String,
    pub current_level: u32,
    pub current_xp: u32,
    pub total_xp: u32,
    pub premium: bool,
}

impl SeasonPass {
    pub fn new(player_id: Uuid, season_id: &str) -> Self {
        Self {
            player_id,
            season_id: season_id.to_string(),
            current_level: 1,
            current_xp: 0,
            total_xp: 0,
            premium: false,
        }
    }

    /// 累积 XP, 触发升档
    pub fn add_xp(&mut self, xp: u32) -> bool {
        self.total_xp = self.total_xp.saturating_add(xp);
        self.current_xp = self.current_xp.saturating_add(xp);
        let mut level_up = false;
        while self.current_xp >= 100 {
            self.current_xp -= 100;
            self.current_level = self.current_level.saturating_add(1);
            level_up = true;
        }
        level_up
    }
}

/// 匹配历史 1 条
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchHistoryEntry {
    pub match_id: Uuid,
    pub timestamp_ms: i64,
    pub win: bool,
    pub score_delta: i32,
}

/// 赛季战绩统计
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeasonStats {
    pub total_matches: u32,
    pub total_players: u32,
    pub total_rewards_claimed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pvp_mode_all_count() {
        assert_eq!(PvpMode::ALL.len(), 9);
    }

    #[test]
    fn match_factory_unfinished() {
        let m = PvpMatch::new(PvpMode::Ranked, Uuid::new_v4(), Uuid::new_v4());
        assert!(!m.finished);
        assert!(m.winner.is_none());
    }

    #[test]
    fn match_finish_sets_winner() {
        let mut m = PvpMatch::new(PvpMode::Ranked, Uuid::new_v4(), Uuid::new_v4());
        let w = m.player_a;
        m.finish(w);
        assert!(m.finished);
        assert_eq!(m.winner, Some(w));
    }

    #[test]
    fn player_state_win_increases_score() {
        let mut s = PlayerPvpState::new(Uuid::new_v4(), PvpMode::Ranked);
        let cfg = PvpConfig::default().get(PvpMode::Ranked).unwrap().clone();
        s.apply_result(true, &cfg);
        assert_eq!(s.wins, 1);
        assert!(s.score > 1000);
    }

    #[test]
    fn player_state_loss_casual_no_score_change() {
        let mut s = PlayerPvpState::new(Uuid::new_v4(), PvpMode::Casual);
        let cfg = PvpConfig::default().get(PvpMode::Casual).unwrap().clone();
        s.apply_result(false, &cfg);
        assert_eq!(s.losses, 1);
        assert_eq!(s.score, 1000); // 不影响分数
    }

    #[test]
    fn tier_from_score_7_buckets() {
        assert_eq!(Tier::from_score(0), Tier::Bronze);
        assert_eq!(Tier::from_score(1999), Tier::Bronze);
        assert_eq!(Tier::from_score(2000), Tier::Silver);
        assert_eq!(Tier::from_score(2500), Tier::Gold);
        assert_eq!(Tier::from_score(3000), Tier::Platinum);
        assert_eq!(Tier::from_score(3500), Tier::Diamond);
        assert_eq!(Tier::from_score(4000), Tier::Master);
        assert_eq!(Tier::from_score(4500), Tier::Grandmaster);
        assert_eq!(Tier::from_score(5000), Tier::Grandmaster);
    }

    #[test]
    fn tier_from_str_roundtrip() {
        for t in Tier::ALL.iter() {
            assert_eq!(Tier::from_str(t.as_str()), Some(*t));
        }
        assert_eq!(Tier::from_str("nope"), None);
    }

    #[test]
    fn season_pass_xp_level_up() {
        let mut sp = SeasonPass::new(Uuid::new_v4(), "S01");
        assert!(!sp.add_xp(50));
        assert_eq!(sp.current_level, 1);
        assert!(sp.add_xp(60));
        assert_eq!(sp.current_level, 2);
        assert_eq!(sp.current_xp, 10);
    }

    #[test]
    fn tier_score_range_increasing() {
        let mut prev_max = -1;
        for t in Tier::ALL.iter() {
            let (lo, hi) = t.score_range();
            assert!(lo > prev_max, "tier {:?} lo={} should be > prev_max={}", t, lo, prev_max);
            assert!(hi >= lo);
            prev_max = hi;
        }
    }
}
