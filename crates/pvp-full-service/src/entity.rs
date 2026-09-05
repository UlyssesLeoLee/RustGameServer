//! pvp-full-service 域 entity
//!
//! 复用 shared_platform::PvpMode (6-12 变体) + PvpConfig

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
}
