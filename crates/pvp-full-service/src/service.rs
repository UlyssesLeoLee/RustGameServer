//! pvp-full-service 业务实现
//!
//! 1 套代码 + shared_platform::PvpConfig 6-12 变体数据驱动 (per 9/4 MD §4 反例)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use shared_platform::data_driven::{PvpConfig, PvpMode, PvpModeConfig};

use crate::entity::{PvpMatch, PlayerPvpState};
use crate::error::{Error, Result};

type PlayerStates = HashMap<(Uuid, PvpMode), PlayerPvpState>;
type Matches = HashMap<Uuid, PvpMatch>;

pub struct PvpFullServiceImpl {
    config: PvpConfig,
    states: Arc<RwLock<PlayerStates>>,
    matches: Arc<RwLock<Matches>>,
}

impl PvpFullServiceImpl {
    pub fn new() -> Self {
        Self {
            config: PvpConfig::default(),
            states: Arc::new(RwLock::new(HashMap::new())),
            matches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn variant_count(&self) -> usize {
        self.config.variant_count()
    }

    pub fn parse_mode(&self, s: &str) -> Result<PvpMode> {
        PvpMode::from_str(s).ok_or_else(|| Error::UnknownMode(s.to_string()))
    }

    pub fn get_mode_config(&self, mode: PvpMode) -> Option<&PvpModeConfig> {
        self.config.get(mode)
    }

    pub async fn get_pvp_info(&self, mode_str: &str, player_id: Uuid) -> Result<(String, u32, u32, bool)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?;
        let states = self.states.read().await;
        let daily_used = states.get(&(player_id, mode)).map(|s| s.daily_used).unwrap_or(0);
        Ok((cfg.display_name.clone(), cfg.daily_limit, daily_used, cfg.cross_server_enabled))
    }

    pub async fn match_player(&self, mode_str: &str, player_id: Uuid, _score: i32) -> Result<PvpMatch> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?.clone();
        let mut states = self.states.write().await;
        let state = states.entry((player_id, mode)).or_insert_with(|| PlayerPvpState::new(player_id, mode));
        if state.daily_used >= cfg.daily_limit {
            return Err(Error::DailyLimitReached(mode_str.into()));
        }
        let opponent = Uuid::new_v4();
        let m = PvpMatch::new(mode, player_id, opponent);
        self.matches.write().await.insert(m.match_id, m.clone());
        Ok(m)
    }

    pub async fn report_result(&self, mode_str: &str, match_id: Uuid, player_id: Uuid, won: bool) -> Result<(i32, bool)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?.clone();
        let mut matches = self.matches.write().await;
        let m = matches.get_mut(&match_id).ok_or_else(|| Error::InvalidRequest("match not found".into()))?;
        if m.finished {
            return Err(Error::InvalidRequest("match already finished".into()));
        }
        m.finish(player_id);
        drop(matches);
        let mut states = self.states.write().await;
        let state = states.entry((player_id, mode)).or_insert_with(|| PlayerPvpState::new(player_id, mode));
        let before = state.score;
        state.apply_result(won, &cfg);
        let promoted = state.score > before;
        Ok((state.score, promoted))
    }

    pub async fn get_season_info(&self, mode_str: &str) -> Result<(String, i64)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?;
        if !cfg.uses_season {
            return Err(Error::InvalidRequest(format!("mode {} does not use season", mode_str)));
        }
        Ok((format!("{}_S01", mode.as_str()), chrono::Utc::now().timestamp_millis() + 30 * 86400 * 1000))
    }

    pub async fn claim_reward(&self, mode_str: &str, player_id: Uuid, _rank: u32) -> Result<Vec<u32>> {
        let mode = self.parse_mode(mode_str)?;
        let states = self.states.read().await;
        let s = states.get(&(player_id, mode)).ok_or_else(|| Error::InvalidRequest("no state".into()))?;
        Ok(vec![s.score as u32])
    }

    pub async fn get_leaderboard(&self, mode_str: &str, top_n: u32) -> Result<Vec<(Uuid, i32)>> {
        let mode = self.parse_mode(mode_str)?;
        let states = self.states.read().await;
        let mut v: Vec<(Uuid, i32)> = states.iter()
            .filter_map(|((pid, m), s)| if *m == mode { Some((*pid, s.score)) } else { None })
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(v.into_iter().take(top_n as usize).collect())
    }
}

impl Default for PvpFullServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_count_is_9() {
        let svc = PvpFullServiceImpl::new();
        assert_eq!(svc.variant_count(), 9);
    }

    #[test]
    fn parse_mode_roundtrip() {
        let svc = PvpFullServiceImpl::new();
        for m in PvpMode::ALL.iter() {
            assert_eq!(svc.parse_mode(m.as_str()).unwrap(), *m);
        }
    }

    #[test]
    fn parse_mode_unknown_fails() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.parse_mode("bogus");
        assert!(matches!(r, Err(Error::UnknownMode(_))));
    }

    #[tokio::test]
    async fn get_pvp_info_ranked() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (name, limit, used, cross) = svc.get_pvp_info("ranked", p).await.unwrap();
        assert_eq!(name, "排位赛");
        assert_eq!(limit, 10);
        assert_eq!(used, 0);
        assert!(!cross);
    }

    #[tokio::test]
    async fn match_player_unknown_mode_fails() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.match_player("bogus", Uuid::new_v4(), 1000).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn match_player_ranked() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.match_player("ranked", p, 1500).await.unwrap();
        assert_eq!(m.mode, PvpMode::Ranked);
        assert!(!m.finished);
    }

    #[tokio::test]
    async fn report_result_win_increases_score() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.match_player("ranked", p, 1500).await.unwrap();
        let (score, _) = svc.report_result("ranked", m.match_id, p, true).await.unwrap();
        assert!(score > 1000);
    }

    #[tokio::test]
    async fn report_result_loss_decreases_score() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let m = svc.match_player("ranked", p, 1500).await.unwrap();
        let (score, _) = svc.report_result("ranked", m.match_id, p, false).await.unwrap();
        assert!(score < 1000);
    }

    #[tokio::test]
    async fn report_unknown_match_fails() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.report_result("ranked", Uuid::new_v4(), Uuid::new_v4(), true).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn daily_limit_ranked() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        // 排位赛 daily_limit = 10
        for _ in 0..10 {
            let m = svc.match_player("ranked", p, 1000).await.unwrap();
            svc.report_result("ranked", m.match_id, p, true).await.unwrap();
        }
        let r = svc.match_player("ranked", p, 1000).await;
        assert!(matches!(r, Err(Error::DailyLimitReached(_))));
    }

    #[tokio::test]
    async fn friendly_no_daily_limit_quick() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        for _ in 0..3 {
            let m = svc.match_player("friendly", p, 1000).await.unwrap();
            svc.report_result("friendly", m.match_id, p, true).await.unwrap();
        }
    }

    #[tokio::test]
    async fn season_info_ranked() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.get_season_info("ranked").await.unwrap();
        assert!(!r.0.is_empty());
    }

    #[tokio::test]
    async fn season_info_friendly_rejected() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.get_season_info("friendly").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn leaderboard_sorted() {
        let svc = PvpFullServiceImpl::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let m1 = svc.match_player("ranked", p1, 1000).await.unwrap();
        svc.report_result("ranked", m1.match_id, p1, true).await.unwrap();
        let m2 = svc.match_player("ranked", p2, 1000).await.unwrap();
        svc.report_result("ranked", m2.match_id, p2, false).await.unwrap();
        let lb = svc.get_leaderboard("ranked", 5).await.unwrap();
        assert_eq!(lb[0].0, p1);
        assert_eq!(lb[1].0, p2);
    }
}
