//! leaderboard-service 域 entity 定义
//!
//! 1 个核心 entity: LeaderboardEntry
//! - (leaderboard_type, period, season_id) 维度上的玩家条目
//! - 排名 rank 通过 score DESC 排序后实时计算
//!
//! 规范: RGS-REQ-038 §FR-007 + RGS-DTL-038 §3

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 榜单类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardType {
    /// 天梯榜
    Ranked,
    /// 休闲榜
    Casual,
    /// 集换价值榜
    Collection,
}

impl LeaderboardType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeaderboardType::Ranked => "ranked",
            LeaderboardType::Casual => "casual",
            LeaderboardType::Collection => "collection",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ranked" => Some(LeaderboardType::Ranked),
            "casual" => Some(LeaderboardType::Casual),
            "collection" => Some(LeaderboardType::Collection),
            _ => None,
        }
    }
}

/// 榜单周期
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardPeriod {
    /// 周榜
    Weekly,
    /// 月榜
    Monthly,
    /// 赛季榜
    Seasonal,
    /// 历史榜
    AllTime,
}

impl LeaderboardPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeaderboardPeriod::Weekly => "weekly",
            LeaderboardPeriod::Monthly => "monthly",
            LeaderboardPeriod::Seasonal => "seasonal",
            LeaderboardPeriod::AllTime => "all_time",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "weekly" => Some(LeaderboardPeriod::Weekly),
            "monthly" => Some(LeaderboardPeriod::Monthly),
            "seasonal" => Some(LeaderboardPeriod::Seasonal),
            "all_time" => Some(LeaderboardPeriod::AllTime),
            _ => None,
        }
    }
}

/// 榜单条目 (per RGS-DTL-038 §3.1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardEntry {
    /// 记录 ID
    pub id: Uuid,
    /// 榜单类型
    pub leaderboard_type: LeaderboardType,
    /// 周期
    pub period: LeaderboardPeriod,
    /// 赛季 ID (ranked 必填, 其他可为 "")
    pub season_id: String,
    /// 玩家 ID
    pub player_id: Uuid,
    /// 玩家展示名 (per DTL-038 §2 PII 最小化)
    pub display_name: String,
    /// 分数 (MMR / 胜场 / 集换价值, 取决于榜单类型)
    pub score: i64,
    /// 胜场
    pub wins: u32,
    /// 负场
    pub losses: u32,
    /// 排名 1-based; 0 = 尚未入榜
    pub rank: u32,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl LeaderboardEntry {
    /// 工厂: 新建条目 (rank=0, 后续通过 service::recompute_rank 写入)
    pub fn new(
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: String,
        player_id: Uuid,
        display_name: String,
        score: i64,
        wins: u32,
        losses: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            leaderboard_type,
            period,
            season_id,
            player_id,
            display_name,
            score,
            wins,
            losses,
            rank: 0,
            updated_at: now,
            created_at: now,
        }
    }

    /// 应用新分数 + 胜负场 (内部 AddEntry 用)
    pub fn apply_score(&mut self, score: i64, wins: u32, losses: u32) {
        self.score = score;
        self.wins = wins;
        self.losses = losses;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaderboard_type_roundtrip() {
        for t in [
            LeaderboardType::Ranked,
            LeaderboardType::Casual,
            LeaderboardType::Collection,
        ] {
            assert_eq!(LeaderboardType::from_str(t.as_str()), Some(t));
        }
        assert_eq!(LeaderboardType::from_str("bogus"), None);
    }

    #[test]
    fn leaderboard_period_roundtrip() {
        for p in [
            LeaderboardPeriod::Weekly,
            LeaderboardPeriod::Monthly,
            LeaderboardPeriod::Seasonal,
            LeaderboardPeriod::AllTime,
        ] {
            assert_eq!(LeaderboardPeriod::from_str(p.as_str()), Some(p));
        }
        assert_eq!(LeaderboardPeriod::from_str("bogus"), None);
    }

    #[test]
    fn entry_factory_initializes_zero_rank() {
        let pid = Uuid::new_v4();
        let e = LeaderboardEntry::new(
            LeaderboardType::Ranked,
            LeaderboardPeriod::Seasonal,
            "s1".to_string(),
            pid,
            "alice".to_string(),
            1500,
            10,
            5,
        );
        assert_eq!(e.score, 1500);
        assert_eq!(e.wins, 10);
        assert_eq!(e.losses, 5);
        assert_eq!(e.rank, 0);
        assert_eq!(e.player_id, pid);
    }

    #[test]
    fn entry_apply_score_updates_timestamp() {
        let mut e = LeaderboardEntry::new(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            Uuid::new_v4(),
            "bob".to_string(),
            0,
            0,
            0,
        );
        let before = e.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(5));
        e.apply_score(7, 5, 2);
        assert_eq!(e.score, 7);
        assert_eq!(e.wins, 5);
        assert_eq!(e.losses, 2);
        assert!(e.updated_at > before);
    }
}
