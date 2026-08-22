//! match-service 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-016 §3 匹配域数据模型）
//! - Match（rust 关键字规避：r#Match）：对局
//! - MatchParticipant：参与者
//!
//! 注意：`match` 是 Rust 关键字，所有 entity / trait / struct 内部使用时
//! 必须用 `r#match` raw identifier。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 对局模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    /// 1v1
    OneVsOne,
    /// 2v2
    TwoVsTwo,
    /// 5v5
    FiveVsFive,
    /// 大乱斗
    BattleRoyale,
}

/// 对局状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    /// 等待开始
    Waiting,
    /// 进行中
    InProgress,
    /// 已结束
    Finished,
    /// 已取消
    Cancelled,
}

/// 队伍
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Team {
    Blue,
    Red,
    None,
}

/// 对局（root entity，per RGS-DTL-016 §3.1）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Match {
    /// 对局 ID
    pub id: Uuid,
    /// 房间 ID
    pub room_id: String,
    /// 模式
    pub mode: MatchMode,
    /// 状态
    pub status: MatchStatus,
    /// 胜方（None = 尚未结束或平局）
    pub winner_team: Option<Team>,
    /// 计划开始时间
    pub scheduled_at: DateTime<Utc>,
    /// 实际开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 实际结束时间
    pub ended_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Match {
    /// 工厂：新建对局（默认 Waiting / scheduled = now）
    pub fn new(room_id: String, mode: MatchMode) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            room_id,
            mode,
            status: MatchStatus::Waiting,
            winner_team: None,
            scheduled_at: now,
            started_at: None,
            ended_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 开始对局
    pub fn start(&mut self) {
        self.status = MatchStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 结束对局
    pub fn finish(&mut self, winner: Option<Team>) {
        self.status = MatchStatus::Finished;
        self.winner_team = winner;
        self.ended_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

/// 对局参与者（per RGS-DTL-016 §3.2）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchParticipant {
    /// 参与记录 ID
    pub id: Uuid,
    /// 对局 ID
    pub match_id: Uuid,
    /// 玩家 ID
    pub player_id: Uuid,
    /// 队伍
    pub team: Team,
    /// 分数
    pub score: i32,
    /// 击杀
    pub kills: i32,
    /// 死亡
    pub deaths: i32,
    /// 助攻
    pub assists: i32,
    /// 是否 MVP
    pub is_mvp: bool,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
}

impl MatchParticipant {
    /// 工厂：新建参与者
    pub fn new(match_id: Uuid, player_id: Uuid, team: Team) -> Self {
        Self {
            id: Uuid::new_v4(),
            match_id,
            player_id,
            team,
            score: 0,
            kills: 0,
            deaths: 0,
            assists: 0,
            is_mvp: false,
            joined_at: Utc::now(),
        }
    }

    /// KDA 比
    pub fn kda_ratio(&self) -> f64 {
        if self.deaths == 0 {
            return (self.kills + self.assists) as f64;
        }
        (self.kills + self.assists) as f64 / self.deaths as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_lifecycle() {
        let mut m = Match::new("room-1".to_string(), MatchMode::TwoVsTwo);
        assert_eq!(m.status, MatchStatus::Waiting);
        m.start();
        assert_eq!(m.status, MatchStatus::InProgress);
        assert!(m.started_at.is_some());
        m.finish(Some(Team::Blue));
        assert_eq!(m.status, MatchStatus::Finished);
        assert_eq!(m.winner_team, Some(Team::Blue));
        assert!(m.ended_at.is_some());
    }

    #[test]
    fn participant_kda() {
        let p = MatchParticipant {
            id: Uuid::new_v4(),
            match_id: Uuid::new_v4(),
            player_id: Uuid::new_v4(),
            team: Team::Blue,
            score: 100,
            kills: 10,
            deaths: 0,
            assists: 5,
            is_mvp: true,
            joined_at: Utc::now(),
        };
        // deaths=0 → returns kills+assists = 15
        assert_eq!(p.kda_ratio(), 15.0);
    }
}
