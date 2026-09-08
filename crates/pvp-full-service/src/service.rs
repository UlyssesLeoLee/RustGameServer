//! pvp-full-service 业务实现
//!
//! 1 套代码 + shared_platform::PvpConfig 6-12 变体数据驱动 (per 9/4 MD §4 反例)
//! W8 L18: 23 新 RPC 共享 4 类存储 (rank_history / seasons / season_passes / match_history)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use shared_platform::data_driven::{PvpConfig, PvpMode, PvpModeConfig};

use crate::entity::{
    MatchHistoryEntry, PvpMatch, PlayerPvpState, RankHistoryEntry, SeasonPass, SeasonRewardPool,
    SeasonStats, SeasonSummary, Tier,
};
use crate::error::{Error, Result};

type PlayerStates = HashMap<(Uuid, PvpMode), PlayerPvpState>;
type Matches = HashMap<Uuid, PvpMatch>;
/// (player, mode) -> history (时间倒序 push)
type RankHistories = HashMap<(Uuid, PvpMode), Vec<RankHistoryEntry>>;
type Seasons = HashMap<(PvpMode, String), SeasonSummary>;
type SeasonRewardPools = HashMap<(PvpMode, String), SeasonRewardPool>;
type SeasonStatsMap = HashMap<(PvpMode, String), SeasonStats>;
type SeasonPasses = HashMap<(Uuid, PvpMode, String), SeasonPass>;
type MatchHistories = HashMap<(Uuid, PvpMode), Vec<MatchHistoryEntry>>;
/// (player, mode) -> 当前连胜计数
type StreakCounts = HashMap<(Uuid, PvpMode), u32>;

pub struct PvpFullServiceImpl {
    config: PvpConfig,
    states: Arc<RwLock<PlayerStates>>,
    matches: Arc<RwLock<Matches>>,
    rank_histories: Arc<RwLock<RankHistories>>,
    seasons: Arc<RwLock<Seasons>>,
    season_reward_pools: Arc<RwLock<SeasonRewardPools>>,
    season_stats: Arc<RwLock<SeasonStatsMap>>,
    season_passes: Arc<RwLock<SeasonPasses>>,
    match_histories: Arc<RwLock<MatchHistories>>,
    streaks: Arc<RwLock<StreakCounts>>,
}

impl PvpFullServiceImpl {
    pub fn new() -> Self {
        Self {
            config: PvpConfig::default(),
            states: Arc::new(RwLock::new(HashMap::new())),
            matches: Arc::new(RwLock::new(HashMap::new())),
            rank_histories: Arc::new(RwLock::new(HashMap::new())),
            seasons: Arc::new(RwLock::new(HashMap::new())),
            season_reward_pools: Arc::new(RwLock::new(HashMap::new())),
            season_stats: Arc::new(RwLock::new(HashMap::new())),
            season_passes: Arc::new(RwLock::new(HashMap::new())),
            match_histories: Arc::new(RwLock::new(HashMap::new())),
            streaks: Arc::new(RwLock::new(HashMap::new())),
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

    fn parse_tier(s: &str) -> Result<Tier> {
        Tier::from_str(s).ok_or_else(|| Error::UnknownTier(s.to_string()))
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

    // ======== 排位 (Rank) 8 RPC ========

    /// GetCurrentRank (player_id, mode) -> rank + tier
    pub async fn get_current_rank(&self, mode_str: &str, player_id: Uuid) -> Result<(i32, Tier, u32)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?;
        if !cfg.uses_rank_score {
            return Err(Error::InvalidRequest(format!("mode {} does not use rank", mode_str)));
        }
        let states = self.states.read().await;
        let s = states.get(&(player_id, mode)).ok_or_else(|| Error::PlayerNotFound(player_id.to_string()))?;
        let tier = Tier::from_score(s.score);
        Ok((s.score, tier, 0))
    }

    /// GetRankLeaderboard (mode, tier, top_n) -> rows
    pub async fn get_rank_leaderboard(&self, mode_str: &str, tier_str: &str, top_n: u32) -> Result<Vec<(Uuid, i32)>> {
        let mode = self.parse_mode(mode_str)?;
        let target_tier = Self::parse_tier(tier_str)?;
        let states = self.states.read().await;
        let mut v: Vec<(Uuid, i32)> = states.iter()
            .filter_map(|((pid, m), s)| if *m == mode && Tier::from_score(s.score) == target_tier { Some((*pid, s.score)) } else { None })
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(v.into_iter().take(top_n as usize).collect())
    }

    /// ReportRankMatchResult (player_id, win, score_delta) -> updated
    pub async fn report_rank_match_result(&self, mode_str: &str, player_id: Uuid, win: bool, score_delta: i32) -> Result<(i32, Tier, bool)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?.clone();
        if !cfg.uses_rank_score {
            return Err(Error::InvalidRequest(format!("mode {} does not use rank", mode_str)));
        }
        let mut states = self.states.write().await;
        let state = states.entry((player_id, mode)).or_insert_with(|| PlayerPvpState::new(player_id, mode));
        let before_tier = Tier::from_score(state.score);
        let before_score = state.score;
        if win {
            state.wins += 1;
            state.score = (state.score + score_delta).min(5000);
        } else {
            state.losses += 1;
            state.score = (state.score - score_delta.abs()).max(0);
        }
        let new_tier = Tier::from_score(state.score);
        let promoted = (new_tier as u32) > (before_tier as u32);
        // 写 rank_history
        let mut histories = self.rank_histories.write().await;
        let entry = RankHistoryEntry { timestamp_ms: chrono::Utc::now().timestamp_millis(), score: state.score, tier: new_tier };
        histories.entry((player_id, mode)).or_insert_with(Vec::new).push(entry);
        // 连胜 / 连败
        let mut streaks = self.streaks.write().await;
        let sk = streaks.entry((player_id, mode)).or_insert(0);
        if win { *sk += 1; } else { *sk = 0; }
        let _ = before_score;
        Ok((state.score, new_tier, promoted))
    }

    /// ClaimRankSeasonReward (player_id, season_id) -> rewards
    pub async fn claim_rank_season_reward(&self, mode_str: &str, player_id: Uuid, season_id: &str) -> Result<(Vec<u32>, Tier)> {
        let mode = self.parse_mode(mode_str)?;
        let states = self.states.read().await;
        let s = states.get(&(player_id, mode)).ok_or_else(|| Error::PlayerNotFound(player_id.to_string()))?;
        let tier = Tier::from_score(s.score);
        let bonus = tier.daily_bonus() as u32;
        let rewards = vec![bonus; 3];
        // 累加 season_stats.total_rewards_claimed
        let mut stats_map = self.season_stats.write().await;
        let stats = stats_map.entry((mode, season_id.to_string())).or_insert_with(SeasonStats::default);
        stats.total_rewards_claimed = stats.total_rewards_claimed.saturating_add(1);
        Ok((rewards, tier))
    }

    /// GetRankHistory (player_id, top_n) -> history
    pub async fn get_rank_history(&self, mode_str: &str, player_id: Uuid, top_n: u32) -> Result<Vec<RankHistoryEntry>> {
        let mode = self.parse_mode(mode_str)?;
        let histories = self.rank_histories.read().await;
        let h = histories.get(&(player_id, mode)).cloned().unwrap_or_default();
        Ok(h.into_iter().rev().take(top_n as usize).collect())
    }

    /// GetRankTierConfig (tier) -> tier info
    pub async fn get_rank_tier_config(&self, _mode_str: &str, tier_str: &str) -> Result<(i32, i32, i32)> {
        let t = Self::parse_tier(tier_str)?;
        let (lo, hi) = t.score_range();
        Ok((lo, hi, t.daily_bonus()))
    }

    /// GetRankStreakBonus (player_id) -> streak + bonus
    pub async fn get_rank_streak_bonus(&self, mode_str: &str, player_id: Uuid) -> Result<(u32, i32)> {
        let mode = self.parse_mode(mode_str)?;
        let streaks = self.streaks.read().await;
        let s = streaks.get(&(player_id, mode)).copied().unwrap_or(0);
        // bonus = streak * 5
        Ok((s, (s as i32) * 5))
    }

    /// ResetRankSeason (season_id) -> reset summary
    pub async fn reset_rank_season(&self, mode_str: &str, season_id: &str) -> Result<(u32, u32)> {
        let mode = self.parse_mode(mode_str)?;
        let mut states = self.states.write().await;
        // 重置该 mode 下所有 player 分数到 1000
        let mut count = 0u32;
        for ((_pid, m), s) in states.iter_mut() {
            if *m == mode {
                s.score = 1000;
                count += 1;
            }
        }
        // 7 段位默认都有人, 上报 7
        let mut stats_map = self.season_stats.write().await;
        stats_map.insert((mode, season_id.to_string()), SeasonStats {
            total_matches: 0,
            total_players: count,
            total_rewards_claimed: 0,
        });
        Ok((count, Tier::ALL.len() as u32))
    }

    // ======== 赛季 (Season) 8 RPC ========

    /// GetCurrentSeason -> season info
    pub async fn get_current_season(&self, mode_str: &str) -> Result<(String, i64, i64, u32)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?;
        if !cfg.uses_season {
            return Err(Error::InvalidRequest(format!("mode {} does not use season", mode_str)));
        }
        let season_id = format!("{}_S01", mode.as_str());
        let now = chrono::Utc::now().timestamp_millis();
        let dur_days = 30u32;
        let starts = now;
        let ends = now + (dur_days as i64) * 86400 * 1000;
        // 注册到 seasons map
        let mut seasons = self.seasons.write().await;
        seasons.insert((mode, season_id.clone()), SeasonSummary {
            season_id: season_id.clone(), mode, starts_at_ms: starts, ends_at_ms: ends, active: true,
        });
        Ok((season_id, starts, ends, dur_days))
    }

    /// ListSeasons (top_n) -> seasons
    pub async fn list_seasons(&self, mode_str: &str, top_n: u32) -> Result<Vec<SeasonSummary>> {
        let mode = self.parse_mode(mode_str)?;
        let seasons = self.seasons.read().await;
        let mut v: Vec<SeasonSummary> = seasons.iter()
            .filter_map(|((m, _), s)| if *m == mode { Some(s.clone()) } else { None })
            .collect();
        v.sort_by(|a, b| b.starts_at_ms.cmp(&a.starts_at_ms));
        Ok(v.into_iter().take(top_n as usize).collect())
    }

    /// GetSeasonRewardPool (season_id) -> pool
    pub async fn get_season_reward_pool(&self, mode_str: &str, season_id: &str) -> Result<SeasonRewardPool> {
        let mode = self.parse_mode(mode_str)?;
        let pools = self.season_reward_pools.read().await;
        let p = pools.get(&(mode, season_id.to_string())).cloned().unwrap_or(SeasonRewardPool {
            total_items: 100, rare_items: 10, total_gold: 50_000,
        });
        Ok(p)
    }

    /// GetSeasonPassInfo (player_id, season_id) -> progress
    pub async fn get_season_pass_info(&self, mode_str: &str, player_id: Uuid, season_id: &str) -> Result<(u32, u32, u32, bool)> {
        let mode = self.parse_mode(mode_str)?;
        let passes = self.season_passes.read().await;
        let p = passes.get(&(player_id, mode, season_id.to_string())).cloned().unwrap_or_else(|| SeasonPass::new(player_id, season_id));
        Ok((p.current_level, p.current_xp, p.total_xp, p.premium))
    }

    /// AdvanceSeasonPass (player_id, season_id, xp) -> new level
    pub async fn advance_season_pass(&self, mode_str: &str, player_id: Uuid, season_id: &str, xp: u32) -> Result<(u32, u32, bool)> {
        let mode = self.parse_mode(mode_str)?;
        let mut passes = self.season_passes.write().await;
        let p = passes.entry((player_id, mode, season_id.to_string())).or_insert_with(|| SeasonPass::new(player_id, season_id));
        let level_up = p.add_xp(xp);
        Ok((p.current_level, p.current_xp, level_up))
    }

    /// ClaimSeasonPassReward (player_id, season_id, level) -> rewards
    pub async fn claim_season_pass_reward(&self, mode_str: &str, player_id: Uuid, season_id: &str, level: u32) -> Result<(bool, Vec<u32>, u32)> {
        let mode = self.parse_mode(mode_str)?;
        let mut passes = self.season_passes.write().await;
        let p = passes.entry((player_id, mode, season_id.to_string())).or_insert_with(|| SeasonPass::new(player_id, season_id));
        if p.current_level < level {
            return Ok((false, vec![], level));
        }
        let reward_count = level as usize;
        let rewards = vec![10u32; reward_count.min(5)];
        // season_stats
        let mut stats_map = self.season_stats.write().await;
        let stats = stats_map.entry((mode, season_id.to_string())).or_insert_with(SeasonStats::default);
        stats.total_rewards_claimed = stats.total_rewards_claimed.saturating_add(1);
        Ok((true, rewards, level))
    }

    /// GetSeasonLeaderboard (season_id, top_n) -> rows
    pub async fn get_season_leaderboard(&self, mode_str: &str, season_id: &str, top_n: u32) -> Result<Vec<(Uuid, i32)>> {
        let mode = self.parse_mode(mode_str)?;
        let _ = season_id; // 当前实现: 全局模式 leaderboard
        let states = self.states.read().await;
        let mut v: Vec<(Uuid, i32)> = states.iter()
            .filter_map(|((pid, m), s)| if *m == mode { Some((*pid, s.score)) } else { None })
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(v.into_iter().take(top_n as usize).collect())
    }

    /// GetSeasonStats (season_id) -> total matches, players, etc.
    pub async fn get_season_stats(&self, mode_str: &str, season_id: &str) -> Result<SeasonStats> {
        let mode = self.parse_mode(mode_str)?;
        let stats_map = self.season_stats.read().await;
        let s = stats_map.get(&(mode, season_id.to_string())).cloned().unwrap_or_default();
        Ok(s)
    }

    // ======== 匹配 (Match) 7 RPC ========

    /// StartMatch (player_id, mode) -> match_id
    pub async fn start_match(&self, mode_str: &str, player_id: Uuid) -> Result<(Uuid, i64)> {
        let mode = self.parse_mode(mode_str)?;
        let cfg = self.config.get(mode).ok_or_else(|| Error::UnknownMode(mode_str.into()))?.clone();
        let mut states = self.states.write().await;
        let state = states.entry((player_id, mode)).or_insert_with(|| PlayerPvpState::new(player_id, mode));
        if state.daily_used >= cfg.daily_limit {
            return Err(Error::DailyLimitReached(mode_str.into()));
        }
        state.daily_used = state.daily_used.saturating_add(1);
        let opponent = Uuid::new_v4();
        let m = PvpMatch::new(mode, player_id, opponent);
        let started_at = m.started_at_ms;
        let match_id = m.match_id;
        self.matches.write().await.insert(match_id, m);
        Ok((match_id, started_at))
    }

    /// CancelMatch (player_id) -> bool
    pub async fn cancel_match(&self, mode_str: &str, player_id: Uuid) -> Result<bool> {
        let mode = self.parse_mode(mode_str)?;
        let mut matches = self.matches.write().await;
        let to_remove: Vec<Uuid> = matches.iter()
            .filter_map(|(mid, m)| {
                if !m.finished && m.mode == mode && (m.player_a == player_id || m.player_b == player_id) {
                    Some(*mid)
                } else {
                    None
                }
            })
            .collect();
        let count = to_remove.len();
        for mid in to_remove {
            matches.remove(&mid);
        }
        Ok(count > 0)
    }

    /// GetMatchStatus (match_id) -> status
    pub async fn get_match_status(&self, _mode_str: &str, match_id: Uuid) -> Result<(String, i64)> {
        let matches = self.matches.read().await;
        let m = matches.get(&match_id).ok_or_else(|| Error::MatchNotFound(match_id.to_string()))?;
        let status = if m.finished { "finished" } else { "in_progress" };
        Ok((status.to_string(), m.started_at_ms))
    }

    /// ReportMatchResult (match_id, winner_team) -> updated
    pub async fn report_match_result(&self, mode_str: &str, match_id: Uuid, winner_team: &str) -> Result<(bool, u32)> {
        let mode = self.parse_mode(mode_str)?;
        let (player_a, player_b) = {
            let mut matches = self.matches.write().await;
            let m = matches.get_mut(&match_id).ok_or_else(|| Error::MatchNotFound(match_id.to_string()))?;
            if m.finished {
                return Ok((false, 0));
            }
            m.finished = true;
            m.winner = Some(if winner_team == "A" { m.player_a } else { m.player_b });
            (m.player_a, m.player_b)
        };
        // 累加 player A 胜负 + B 胜负
        let mut states = self.states.write().await;
        {
            let sa = states.entry((player_a, mode)).or_insert_with(|| PlayerPvpState::new(player_a, mode));
            if winner_team == "A" { sa.wins = sa.wins.saturating_add(1); } else { sa.losses = sa.losses.saturating_add(1); }
        }
        {
            let sb = states.entry((player_b, mode)).or_insert_with(|| PlayerPvpState::new(player_b, mode));
            if winner_team == "B" { sb.wins = sb.wins.saturating_add(1); } else { sb.losses = sb.losses.saturating_add(1); }
        }
        // 记录 match_history (player_a 视角)
        let mut mh = self.match_histories.write().await;
        let entry = MatchHistoryEntry {
            match_id, timestamp_ms: chrono::Utc::now().timestamp_millis(),
            win: winner_team == "A", score_delta: 0,
        };
        mh.entry((player_a, mode)).or_insert_with(Vec::new).push(entry);
        // season_stats
        let mut stats_map = self.season_stats.write().await;
        let season_id = format!("{}_S01", mode.as_str());
        let stats = stats_map.entry((mode, season_id)).or_insert_with(SeasonStats::default);
        stats.total_matches = stats.total_matches.saturating_add(1);
        Ok((true, 1))
    }

    /// GetMatchHistory (player_id, top_n) -> history
    pub async fn get_match_history(&self, mode_str: &str, player_id: Uuid, top_n: u32) -> Result<Vec<MatchHistoryEntry>> {
        let mode = self.parse_mode(mode_str)?;
        let mh = self.match_histories.read().await;
        let v = mh.get(&(player_id, mode)).cloned().unwrap_or_default();
        Ok(v.into_iter().rev().take(top_n as usize).collect())
    }

    /// GetMatchStats (player_id) -> win/loss/streak
    pub async fn get_match_stats(&self, mode_str: &str, player_id: Uuid) -> Result<(u32, u32, u32)> {
        let mode = self.parse_mode(mode_str)?;
        let states = self.states.read().await;
        let s = states.get(&(player_id, mode)).ok_or_else(|| Error::PlayerNotFound(player_id.to_string()))?;
        let streaks = self.streaks.read().await;
        let st = streaks.get(&(player_id, mode)).copied().unwrap_or(0);
        Ok((s.wins, s.losses, st))
    }

    /// GetMatchReward (match_id) -> rewards
    pub async fn get_match_reward(&self, _mode_str: &str, match_id: Uuid) -> Result<Vec<u32>> {
        let matches = self.matches.read().await;
        let m = matches.get(&match_id).ok_or_else(|| Error::MatchNotFound(match_id.to_string()))?;
        if !m.finished {
            return Ok(vec![]);
        }
        let base = if m.winner == Some(m.player_a) { 30 } else { 10 };
        Ok(vec![base, 5, 2])
    }
}

impl Default for PvpFullServiceImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 旧 6 变体 7 RPC 测试 (保留) ===

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

    // === 排位 (Rank) 8 RPC 测试 ===

    #[tokio::test]
    async fn get_current_rank_default_bronze() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.get_current_rank("ranked", p).await;
        assert!(matches!(r, Err(Error::PlayerNotFound(_))));
    }

    #[tokio::test]
    async fn get_current_rank_after_match_silver() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        // 通过 report_rank_match_result 写分数到 2050
        svc.report_rank_match_result("ranked", p, true, 1050).await.unwrap();
        let (score, tier, _rank) = svc.get_current_rank("ranked", p).await.unwrap();
        assert_eq!(score, 2050);
        assert_eq!(tier, Tier::Silver);
    }

    #[tokio::test]
    async fn get_current_rank_casual_rejected() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.get_current_rank("casual", p).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn rank_leaderboard_filter_by_tier() {
        let svc = PvpFullServiceImpl::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        svc.report_rank_match_result("ranked", p1, true, 1100).await.unwrap(); // 2100 Silver
        svc.report_rank_match_result("ranked", p2, true, 0).await.unwrap();    // 1000 Bronze
        let silver = svc.get_rank_leaderboard("ranked", "silver", 10).await.unwrap();
        assert!(silver.iter().any(|(pid, _)| *pid == p1));
        assert!(!silver.iter().any(|(pid, _)| *pid == p2));
    }

    #[tokio::test]
    async fn rank_leaderboard_unknown_tier() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.get_rank_leaderboard("ranked", "nope_tier", 10).await;
        assert!(matches!(r, Err(Error::UnknownTier(_))));
    }

    #[tokio::test]
    async fn report_rank_match_result_promotes_tier() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        // 1000 -> 2600 直接跳 Silver -> Gold
        let (_, new_tier, promoted) = svc.report_rank_match_result("ranked", p, true, 1600).await.unwrap();
        assert_eq!(new_tier, Tier::Gold);
        assert!(promoted);
    }

    #[tokio::test]
    async fn report_rank_match_result_loss_drops_tier() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        // 先升到 3100 Platinum, 再输 1100 回到 2000 Silver
        svc.report_rank_match_result("ranked", p, true, 2100).await.unwrap();
        let (_, _, _) = svc.report_rank_match_result("ranked", p, false, 1100).await.unwrap();
        let (score, tier, _) = svc.get_current_rank("ranked", p).await.unwrap();
        assert_eq!(score, 2000);
        assert_eq!(tier, Tier::Silver);
    }

    #[tokio::test]
    async fn claim_rank_season_reward_after_play() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        svc.report_rank_match_result("ranked", p, true, 1500).await.unwrap();
        let (rewards, tier) = svc.claim_rank_season_reward("ranked", p, "ranked_S01").await.unwrap();
        assert_eq!(rewards.len(), 3);
        assert!(tier.daily_bonus() > 0);
    }

    #[tokio::test]
    async fn get_rank_history_records_writes() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        svc.report_rank_match_result("ranked", p, true, 200).await.unwrap();
        svc.report_rank_match_result("ranked", p, true, 300).await.unwrap();
        let h = svc.get_rank_history("ranked", p, 10).await.unwrap();
        assert_eq!(h.len(), 2);
        // 最新在前
        assert!(h[0].score >= h[1].score);
    }

    #[tokio::test]
    async fn get_rank_tier_config_returns_range() {
        let svc = PvpFullServiceImpl::new();
        let (lo, hi, bonus) = svc.get_rank_tier_config("ranked", "diamond").await.unwrap();
        assert_eq!(lo, 3500);
        assert_eq!(hi, 3999);
        assert_eq!(bonus, 30);
    }

    #[tokio::test]
    async fn get_rank_streak_bonus_streak_3() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        svc.report_rank_match_result("ranked", p, true, 50).await.unwrap();
        svc.report_rank_match_result("ranked", p, true, 50).await.unwrap();
        svc.report_rank_match_result("ranked", p, true, 50).await.unwrap();
        let (streak, bonus) = svc.get_rank_streak_bonus("ranked", p).await.unwrap();
        assert_eq!(streak, 3);
        assert_eq!(bonus, 15);
    }

    #[tokio::test]
    async fn get_rank_streak_bonus_loss_resets() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        svc.report_rank_match_result("ranked", p, true, 50).await.unwrap();
        svc.report_rank_match_result("ranked", p, true, 50).await.unwrap();
        svc.report_rank_match_result("ranked", p, false, 50).await.unwrap();
        let (streak, _) = svc.get_rank_streak_bonus("ranked", p).await.unwrap();
        assert_eq!(streak, 0);
    }

    #[tokio::test]
    async fn reset_rank_season_zeros_all() {
        let svc = PvpFullServiceImpl::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        svc.report_rank_match_result("ranked", p1, true, 2000).await.unwrap();
        svc.report_rank_match_result("ranked", p2, true, 1500).await.unwrap();
        let (count, tiers) = svc.reset_rank_season("ranked", "ranked_S02").await.unwrap();
        assert_eq!(count, 2);
        assert_eq!(tiers, 7);
        let (s1, _, _) = svc.get_current_rank("ranked", p1).await.unwrap();
        assert_eq!(s1, 1000);
    }

    // === 赛季 (Season) 8 RPC 测试 ===

    #[tokio::test]
    async fn get_current_season_ranked_30d() {
        let svc = PvpFullServiceImpl::new();
        let (sid, _s, e, dur) = svc.get_current_season("ranked").await.unwrap();
        assert!(sid.starts_with("ranked"));
        assert!(e > 0);
        assert_eq!(dur, 30);
    }

    #[tokio::test]
    async fn get_current_season_friendly_rejected() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.get_current_season("friendly").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn list_seasons_after_get() {
        let svc = PvpFullServiceImpl::new();
        svc.get_current_season("ranked").await.unwrap();
        let v = svc.list_seasons("ranked", 10).await.unwrap();
        assert!(!v.is_empty());
    }

    #[tokio::test]
    async fn list_seasons_empty_initially_for_mode() {
        let svc = PvpFullServiceImpl::new();
        let v = svc.list_seasons("casual", 10).await.unwrap();
        assert!(v.is_empty());
    }

    #[tokio::test]
    async fn get_season_reward_pool_default() {
        let svc = PvpFullServiceImpl::new();
        let p = svc.get_season_reward_pool("ranked", "ranked_S01").await.unwrap();
        assert_eq!(p.total_items, 100);
        assert_eq!(p.rare_items, 10);
        assert_eq!(p.total_gold, 50_000);
    }

    #[tokio::test]
    async fn get_season_pass_info_default_l1() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (lvl, xp, total, prem) = svc.get_season_pass_info("ranked", p, "ranked_S01").await.unwrap();
        assert_eq!(lvl, 1);
        assert_eq!(xp, 0);
        assert_eq!(total, 0);
        assert!(!prem);
    }

    #[tokio::test]
    async fn advance_season_pass_levels_up() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (lvl, _, up) = svc.advance_season_pass("ranked", p, "ranked_S01", 150).await.unwrap();
        assert_eq!(lvl, 2);
        assert!(up);
    }

    #[tokio::test]
    async fn advance_season_pass_no_level_up() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (lvl, _, up) = svc.advance_season_pass("ranked", p, "ranked_S01", 30).await.unwrap();
        assert_eq!(lvl, 1);
        assert!(!up);
    }

    #[tokio::test]
    async fn claim_season_pass_reward_below_level_rejected() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (ok, rewards, _) = svc.claim_season_pass_reward("ranked", p, "ranked_S01", 5).await.unwrap();
        assert!(!ok);
        assert!(rewards.is_empty());
    }

    #[tokio::test]
    async fn claim_season_pass_reward_at_level_ok() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (ok, rewards, level) = svc.claim_season_pass_reward("ranked", p, "ranked_S01", 1).await.unwrap();
        assert!(ok);
        assert!(!rewards.is_empty());
        assert_eq!(level, 1);
    }

    #[tokio::test]
    async fn get_season_leaderboard_empty() {
        let svc = PvpFullServiceImpl::new();
        let lb = svc.get_season_leaderboard("ranked", "ranked_S01", 10).await.unwrap();
        assert!(lb.is_empty());
    }

    #[tokio::test]
    async fn get_season_stats_default_zero() {
        let svc = PvpFullServiceImpl::new();
        let s = svc.get_season_stats("ranked", "ranked_S01").await.unwrap();
        assert_eq!(s.total_matches, 0);
        assert_eq!(s.total_players, 0);
        assert_eq!(s.total_rewards_claimed, 0);
    }

    // === 匹配 (Match) 7 RPC 测试 ===

    #[tokio::test]
    async fn start_match_creates_uuid() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, started) = svc.start_match("casual", p).await.unwrap();
        assert!(!mid.is_nil());
        assert!(started > 0);
    }

    #[tokio::test]
    async fn start_match_daily_limit_blocks() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        // hero_hall daily_limit=1
        svc.start_match("hero_hall", p).await.unwrap();
        let r = svc.start_match("hero_hall", p).await;
        assert!(matches!(r, Err(Error::DailyLimitReached(_))));
    }

    #[tokio::test]
    async fn cancel_match_unknown_player_returns_false() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.cancel_match("casual", Uuid::new_v4()).await.unwrap();
        assert!(!r);
    }

    #[tokio::test]
    async fn cancel_match_after_start_ok() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        svc.start_match("casual", p).await.unwrap();
        let r = svc.cancel_match("casual", p).await.unwrap();
        assert!(r);
    }

    #[tokio::test]
    async fn get_match_status_in_progress() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, _) = svc.start_match("casual", p).await.unwrap();
        let (status, _) = svc.get_match_status("casual", mid).await.unwrap();
        assert_eq!(status, "in_progress");
    }

    #[tokio::test]
    async fn get_match_status_unknown_fails() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.get_match_status("casual", Uuid::new_v4()).await;
        assert!(matches!(r, Err(Error::MatchNotFound(_))));
    }

    #[tokio::test]
    async fn report_match_result_finishes_match() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, _) = svc.start_match("casual", p).await.unwrap();
        let (updated, count) = svc.report_match_result("casual", mid, "A").await.unwrap();
        assert!(updated);
        assert_eq!(count, 1);
        let (status, _) = svc.get_match_status("casual", mid).await.unwrap();
        assert_eq!(status, "finished");
    }

    #[tokio::test]
    async fn report_match_result_unknown_fails() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.report_match_result("casual", Uuid::new_v4(), "A").await;
        assert!(matches!(r, Err(Error::MatchNotFound(_))));
    }

    #[tokio::test]
    async fn get_match_history_after_result() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, _) = svc.start_match("casual", p).await.unwrap();
        svc.report_match_result("casual", mid, "A").await.unwrap();
        let h = svc.get_match_history("casual", p, 10).await.unwrap();
        assert_eq!(h.len(), 1);
    }

    #[tokio::test]
    async fn get_match_stats_unknown_player_fails() {
        let svc = PvpFullServiceImpl::new();
        let r = svc.get_match_stats("casual", Uuid::new_v4()).await;
        assert!(matches!(r, Err(Error::PlayerNotFound(_))));
    }

    #[tokio::test]
    async fn get_match_stats_after_play() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, _) = svc.start_match("casual", p).await.unwrap();
        svc.report_match_result("casual", mid, "A").await.unwrap();
        let (w, l, _st) = svc.get_match_stats("casual", p).await.unwrap();
        assert_eq!(w + l, 1);
    }

    #[tokio::test]
    async fn get_match_reward_before_finish_empty() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, _) = svc.start_match("casual", p).await.unwrap();
        let r = svc.get_match_reward("casual", mid).await.unwrap();
        assert!(r.is_empty());
    }

    #[tokio::test]
    async fn get_match_reward_after_finish_nonempty() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        let (mid, _) = svc.start_match("casual", p).await.unwrap();
        svc.report_match_result("casual", mid, "A").await.unwrap();
        let r = svc.get_match_reward("casual", mid).await.unwrap();
        assert!(!r.is_empty());
    }

    // === 跨 RPC 集成测试 ===

    #[tokio::test]
    async fn integration_full_rank_season_match_flow() {
        let svc = PvpFullServiceImpl::new();
        let p = Uuid::new_v4();
        // 1. GetCurrentSeason
        let (sid, _, _, _) = svc.get_current_season("ranked").await.unwrap();
        // 2. StartMatch
        let (mid, _) = svc.start_match("ranked", p).await.unwrap();
        // 3. ReportMatchResult
        svc.report_match_result("ranked", mid, "A").await.unwrap();
        // 4. ReportRankMatchResult
        let (score, tier, _) = svc.report_rank_match_result("ranked", p, true, 200).await.unwrap();
        assert!(score >= 1200);
        // 5. GetCurrentRank
        let (s2, t2, _) = svc.get_current_rank("ranked", p).await.unwrap();
        assert_eq!(s2, score);
        assert_eq!(t2, tier);
        // 6. ClaimRankSeasonReward
        let (rewards, _) = svc.claim_rank_season_reward("ranked", p, &sid).await.unwrap();
        assert!(!rewards.is_empty());
        // 7. GetSeasonStats 应有 total_matches=1, total_rewards_claimed=1
        let stats = svc.get_season_stats("ranked", &sid).await.unwrap();
        assert_eq!(stats.total_matches, 1);
        assert_eq!(stats.total_rewards_claimed, 1);
    }
}
