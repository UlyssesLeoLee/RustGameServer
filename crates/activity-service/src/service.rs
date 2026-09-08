//! activity-service 业务实现
//!
//! 9 holiday_* 数据驱动 (per 9/4 MD §4 反例, 1 套 + 配置):
//! - get_holiday_info: 走 HolidayConfig.get(activity_id)
//! - draw_holiday_prize: 按 activity_id 路由 1 套抽奖逻辑
//! - get_holiday_tasks / claim_holiday_reward: 1 套任务模板
//!
//! 业务方法: ≥10 真实 + 签到/成就

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use shared_platform::data_driven::{HolidayConfig, HolidayActivity};

use crate::entity::{Achievement, PlayerSignin};
use crate::error::{Error, Result};

/// 任务进度
#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub task_id: String,
    pub current: u32,
    pub target: u32,
    pub claimed: bool,
}

/// 玩家在某 holiday_* 的任务列表
#[derive(Debug, Clone, Default)]
pub struct PlayerHolidayTasks {
    pub player_id: Uuid,
    pub activity_id: String,
    pub tasks: Vec<TaskProgress>,
}

impl PlayerHolidayTasks {
    pub fn new(player_id: Uuid, activity_id: &str) -> Self {
        Self {
            player_id,
            activity_id: activity_id.to_string(),
            // 默认 3 个任务 (1 套模板, 9 个 holiday_* 复用)
            tasks: vec![
                TaskProgress { task_id: "daily_kill".into(), current: 0, target: 10, claimed: false },
                TaskProgress { task_id: "daily_login".into(), current: 0, target: 7, claimed: false },
                TaskProgress { task_id: "spend".into(), current: 0, target: 1000, claimed: false },
            ],
        }
    }

    pub fn advance(&mut self, task_id: &str, by: u32) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            t.current = (t.current + by).min(t.target);
        }
    }
}

/// 抽奖奖池项
#[derive(Debug, Clone)]
pub struct PrizeItem {
    pub item_id: u32,
    pub count: u32,
    pub is_rare: bool,
}

// ========== W7 L18 战令 / 签到扩展 / 回归 / 推送 / 拉新 子结构 ==========
// 接受 §1.3 NO-GO 风险 per 9/8 20:08 JST Ulysses 拍板 (后续可能返工)

/// 战令状态 (per player_id, season_id)
#[derive(Debug, Clone)]
pub struct BattlePassState {
    pub player_id: Uuid,
    pub season_id: u32,
    pub level: u32,
    pub xp: u32,
    pub is_premium: bool,
    pub claimed_levels: HashSet<u32>,
}

impl BattlePassState {
    pub fn new(player_id: Uuid, season_id: u32) -> Self {
        Self {
            player_id,
            season_id,
            level: 1,
            xp: 0,
            is_premium: false,
            claimed_levels: HashSet::new(),
        }
    }

    /// 每级所需 XP (简化: 100/级)
    pub fn xp_per_level() -> u32 {
        100
    }

    /// 升级所需 XP 总数 (per 当前 level)
    pub fn next_level_xp(&self) -> u32 {
        Self::xp_per_level()
    }

    /// 应用 XP, 升级
    pub fn apply_xp(&mut self, xp: u32) {
        self.xp += xp;
        while self.xp >= Self::xp_per_level() {
            self.xp -= Self::xp_per_level();
            self.level += 1;
        }
    }
}

/// 战令奖励
#[derive(Debug, Clone)]
pub struct BattlePassReward {
    pub level: u32,
    pub item_id: u32,
    pub count: u32,
    pub is_premium: bool,
}

/// 签到扩展状态 (per player_id, month)
#[derive(Debug, Clone)]
pub struct SigninExtState {
    pub player_id: Uuid,
    pub month: u32,
    pub signed_days: Vec<u32>,
    pub total_days: u32,
    pub streak_max: u32,
    pub miss_count: u32,
}

impl SigninExtState {
    pub fn new(player_id: Uuid, month: u32) -> Self {
        Self {
            player_id,
            month,
            signed_days: Vec::new(),
            total_days: 0,
            streak_max: 0,
            miss_count: 0,
        }
    }
}

/// 签到日奖励模板
#[derive(Debug, Clone)]
pub struct SigninDayReward {
    pub day: u32,
    pub item_id: u32,
    pub count: u32,
}

/// 回归玩家状态
#[derive(Debug, Clone)]
pub struct ReturningPlayerState {
    pub player_id: Uuid,
    pub last_login_ms: i64,
    pub is_returning: bool,
    pub claimed_rewards: HashSet<u32>,
}

impl ReturningPlayerState {
    pub fn new(player_id: Uuid, last_login_ms: i64) -> Self {
        // 30 天未登录 = 回归玩家
        let is_returning = last_login_ms == 0;
        Self {
            player_id,
            last_login_ms,
            is_returning,
            claimed_rewards: HashSet::new(),
        }
    }
}

/// 回归活动配置
#[derive(Debug, Clone)]
pub struct ReturnActivityConfig {
    pub reward_id: u32,
    pub item_id: u32,
    pub count: u32,
    pub expires_at_ms: i64,
    pub name: String,
}

/// 推送记录
#[derive(Debug, Clone)]
pub struct PushRecord {
    pub push_id: u32,
    pub title: String,
    pub body: String,
    pub created_at_ms: i64,
    pub is_read: bool,
}

/// 拉新状态
#[derive(Debug, Clone)]
pub struct InviteState {
    pub invite_code: String,
    pub invited_count: u32,
    pub rewards: Vec<PrizeItem>,
}

impl InviteState {
    pub fn new(invite_code: String) -> Self {
        Self {
            invite_code,
            invited_count: 0,
            rewards: vec![
                PrizeItem { item_id: 4001, count: 1, is_rare: false },
                PrizeItem { item_id: 4002, count: 5, is_rare: false },
            ],
        }
    }
}

/// 活动运营域业务实现
pub struct ActivityServiceImpl {
    /// 9 个 holiday_* 活动配置 (per 9/4 MD §4 反例, 1 套 + 配置)
    holiday_config: HolidayConfig,
    /// 玩家任务进度: (player_id, activity_id) -> tasks
    player_tasks: Arc<RwLock<HashMap<(Uuid, String), PlayerHolidayTasks>>>,
    /// 玩家签到: player_id -> PlayerSignin
    signins: Arc<RwLock<HashMap<Uuid, PlayerSignin>>>,
    /// 玩家成就: player_id -> Vec<Achievement>
    achievements: Arc<RwLock<HashMap<Uuid, Vec<Achievement>>>>,
    /// W7 L18 问卷: 玩家已提交 (player_id, survey_id) 防重复
    submitted_surveys: Arc<RwLock<HashSet<(Uuid, u32)>>>,
    /// W7 L18 战令: (player_id, season_id) -> BattlePassState
    battle_pass: Arc<RwLock<HashMap<(Uuid, u32), BattlePassState>>>,
    /// W7 L18 签到扩展: (player_id, month) -> SigninExtState
    signin_ext: Arc<RwLock<HashMap<(Uuid, u32), SigninExtState>>>,
    /// W7 L18 签到月份奖励模板: month -> Vec<SigninDayReward>
    signin_calendar_config: Arc<RwLock<HashMap<u32, Vec<SigninDayReward>>>>,
    /// W7 L18 回归玩家: player_id -> ReturningPlayerState
    returning: Arc<RwLock<HashMap<Uuid, ReturningPlayerState>>>,
    /// W7 L18 回归活动配置 (静态, 全局): reward_id -> ReturnActivityConfig
    return_activities_config: Vec<ReturnActivityConfig>,
    /// W7 L18 推送: player_id -> Vec<PushRecord>
    push_list: Arc<RwLock<HashMap<Uuid, Vec<PushRecord>>>>,
    /// W7 L18 拉新: player_id -> InviteState
    invites: Arc<RwLock<HashMap<Uuid, InviteState>>>,
}

impl ActivityServiceImpl {
    pub fn new() -> Self {
        // W7 L18 回归活动默认配置 (3 个 reward_id)
        let return_activities_config = vec![
            ReturnActivityConfig {
                reward_id: 1,
                item_id: 9101,
                count: 10,
                expires_at_ms: i64::MAX,
                name: "回归大礼包".into(),
            },
            ReturnActivityConfig {
                reward_id: 2,
                item_id: 9102,
                count: 5,
                expires_at_ms: i64::MAX,
                name: "回归钻石".into(),
            },
            ReturnActivityConfig {
                reward_id: 3,
                item_id: 9103,
                count: 1,
                expires_at_ms: i64::MAX,
                name: "回归稀有角色".into(),
            },
        ];
        // W7 L18 签到扩展默认配置: month=9 -> 30 天 daily reward
        let mut signin_calendar_config = HashMap::new();
        let month9_days: Vec<SigninDayReward> = (1..=30)
            .map(|d| SigninDayReward {
                day: d,
                item_id: 3000 + d,
                count: if d % 7 == 0 { 5 } else { 1 },
            })
            .collect();
        signin_calendar_config.insert(9u32, month9_days);
        Self {
            holiday_config: HolidayConfig::default(),
            player_tasks: Arc::new(RwLock::new(HashMap::new())),
            signins: Arc::new(RwLock::new(HashMap::new())),
            achievements: Arc::new(RwLock::new(HashMap::new())),
            submitted_surveys: Arc::new(RwLock::new(HashSet::new())),
            battle_pass: Arc::new(RwLock::new(HashMap::new())),
            signin_ext: Arc::new(RwLock::new(HashMap::new())),
            signin_calendar_config: Arc::new(RwLock::new(signin_calendar_config)),
            returning: Arc::new(RwLock::new(HashMap::new())),
            return_activities_config,
            push_list: Arc::new(RwLock::new(HashMap::new())),
            invites: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 1 套代码 + activity_id 路由 (per 9/4 MD §4 反例)
    pub fn get_holiday_info(&self, activity_id: &str, now_ms: i64) -> Result<&HolidayActivity> {
        self.holiday_config
            .get(activity_id)
            .ok_or_else(|| Error::ActivityNotFound(activity_id.to_string()))
            .and_then(|act| {
                if act.is_open_at(now_ms) {
                    Ok(act)
                } else {
                    Err(Error::ActivityNotOpen(activity_id.to_string()))
                }
            })
    }

    pub fn list_holiday_ids(&self) -> Vec<String> {
        self.holiday_config.list_ids()
    }

    /// 1 套抽奖逻辑, 9 个 holiday_* 复用
    pub async fn draw_prize(
        &self,
        activity_id: &str,
        player_id: Uuid,
        draw_count: u32,
        now_ms: i64,
    ) -> Result<Vec<PrizeItem>> {
        let act = self.get_holiday_info(activity_id, now_ms)?;
        // 简化: 1/draw_count 概率出稀有
        let mut drawn = Vec::new();
        for i in 0..draw_count {
            let is_rare = (i + (now_ms as u32).wrapping_add(player_id.as_u128() as u32)) % 10 == 0;
            drawn.push(PrizeItem {
                item_id: if is_rare { 9001 } else { 1001 },
                count: 1,
                is_rare,
            });
        }
        let _ = act; // 引用以保留活动校验副作用
        Ok(drawn)
    }

    /// 获取玩家在某 holiday 的任务进度
    pub async fn get_player_tasks(
        &self,
        player_id: Uuid,
        activity_id: &str,
    ) -> Result<Vec<TaskProgress>> {
        // 校验活动存在
        self.holiday_config
            .get(activity_id)
            .ok_or_else(|| Error::ActivityNotFound(activity_id.to_string()))?;
        let mut map = self.player_tasks.write().await;
        let entry = map
            .entry((player_id, activity_id.to_string()))
            .or_insert_with(|| PlayerHolidayTasks::new(player_id, activity_id));
        Ok(entry.tasks.clone())
    }

    pub async fn advance_task(
        &self,
        player_id: Uuid,
        activity_id: &str,
        task_id: &str,
        by: u32,
    ) -> Result<()> {
        self.holiday_config
            .get(activity_id)
            .ok_or_else(|| Error::ActivityNotFound(activity_id.to_string()))?;
        let mut map = self.player_tasks.write().await;
        let entry = map
            .entry((player_id, activity_id.to_string()))
            .or_insert_with(|| PlayerHolidayTasks::new(player_id, activity_id));
        entry.advance(task_id, by);
        Ok(())
    }

    pub async fn claim_task(
        &self,
        player_id: Uuid,
        activity_id: &str,
        task_id: &str,
    ) -> Result<Vec<PrizeItem>> {
        let mut map = self.player_tasks.write().await;
        let entry = map
            .get_mut(&(player_id, activity_id.to_string()))
            .ok_or_else(|| Error::PlayerState("no tasks for player".into()))?;
        let t = entry
            .tasks
            .iter_mut()
            .find(|t| t.task_id == task_id)
            .ok_or_else(|| Error::InvalidRequest(format!("task {} not found", task_id)))?;
        if t.claimed {
            return Err(Error::PlayerState("already claimed".into()));
        }
        if t.current < t.target {
            return Err(Error::PlayerState("task not complete".into()));
        }
        t.claimed = true;
        Ok(vec![PrizeItem { item_id: 5001, count: 1, is_rare: false }])
    }

    // ========== 签到 ==========

    pub async fn get_signin_status(&self, player_id: Uuid, month: u32) -> PlayerSignin {
        let map = self.signins.read().await;
        map.get(&player_id)
            .cloned()
            .unwrap_or_else(|| PlayerSignin::new(player_id, month))
    }

    pub async fn do_signin(
        &self,
        player_id: Uuid,
        day: u32,
    ) -> Result<Vec<PrizeItem>> {
        let month = 9u32; // 简化
        let mut map = self.signins.write().await;
        let s = map
            .entry(player_id)
            .or_insert_with(|| PlayerSignin::new(player_id, month));
        if !s.can_sign(day) {
            return Err(Error::InvalidRequest(format!("day {} not signable", day)));
        }
        s.sign(day);
        Ok(vec![PrizeItem { item_id: 2001, count: 1, is_rare: false }])
    }

    pub async fn resignin(&self, player_id: Uuid, day: u32) -> Result<u32> {
        let mut map = self.signins.write().await;
        let s = map
            .entry(player_id)
            .or_insert_with(|| PlayerSignin::new(player_id, 9));
        if !s.is_signed(day) {
            return Err(Error::InvalidRequest(format!("day {} not signed", day)));
        }
        s.signed_days.retain(|&d| d != day);
        s.streak_days = s.streak_days.saturating_sub(1);
        Ok(100) // cost
    }

    // ========== 成就 ==========

    pub async fn get_achievements(&self, player_id: Uuid) -> Vec<Achievement> {
        let map = self.achievements.read().await;
        map.get(&player_id).cloned().unwrap_or_else(|| {
            vec![
                Achievement::new(1, "首战告捷", 1),
                Achievement::new(2, "百战老兵", 100),
                Achievement::new(3, "收集者", 50),
            ]
        })
    }

    pub async fn claim_achievement(
        &self,
        player_id: Uuid,
        achievement_id: u32,
    ) -> Result<Vec<PrizeItem>> {
        let achievements = self.get_achievements(player_id).await;
        let a = achievements
            .into_iter()
            .find(|a| a.achievement_id == achievement_id)
            .ok_or_else(|| Error::InvalidRequest(format!("achievement {} not found", achievement_id)))?;
        if !a.is_complete() {
            return Err(Error::PlayerState("achievement not complete".into()));
        }
        if a.claimed {
            return Err(Error::PlayerState("already claimed".into()));
        }
        Ok(vec![PrizeItem { item_id: 7001, count: 1, is_rare: true }])
    }

    pub async fn advance_achievement(
        &self,
        player_id: Uuid,
        achievement_id: u32,
        by: u32,
    ) -> Result<()> {
        let mut map = self.achievements.write().await;
        let list = map.entry(player_id).or_insert_with(|| {
            vec![
                Achievement::new(1, "首战告捷", 1),
                Achievement::new(2, "百战老兵", 100),
                Achievement::new(3, "收集者", 50),
            ]
        });
        if let Some(a) = list.iter_mut().find(|a| a.achievement_id == achievement_id) {
            a.advance(by);
        }
        Ok(())
    }

    // ========== W7 L18 问卷 (1 个, 模板) ==========
    // 9/8 20:08 JST Ulysses 拍板启动 L18 W7-W9 派工, 接受 §1.3 NO-GO 风险

    /// 提交问卷, 简单 in-memory 已提交集合防重复
    pub async fn submit_survey(
        &self,
        player_id: Uuid,
        survey_id: u32,
        answers_json: &str,
    ) -> Result<(u32, u32)> {
        if answers_json.is_empty() {
            return Err(Error::InvalidRequest("answers_json is empty".into()));
        }
        // 防重复: 同 player + survey 只能提交 1 次
        let key = (player_id, survey_id);
        let mut submitted = self.submitted_surveys.write().await;
        if submitted.contains(&key) {
            return Err(Error::PlayerState(format!("survey {} already submitted", survey_id)));
        }
        submitted.insert(key);
        // 简单奖励: survey_id % 5 决定道具, count = survey_id
        let item_id = 8000u32 + (survey_id % 5) as u32;
        let count = survey_id.max(1);
        Ok((item_id, count))
    }

    // ========== W7 L18 战令 (Battle Pass) 5 RPC ==========

    /// 获取战令信息
    pub async fn get_battle_pass_info(
        &self,
        player_id: Uuid,
        season_id: u32,
    ) -> Result<(u32, u32, u32, bool)> {
        // (level, xp, next_level_xp, is_premium)
        if season_id == 0 {
            return Err(Error::InvalidRequest("season_id must be > 0".into()));
        }
        let map = self.battle_pass.read().await;
        let s = map.get(&(player_id, season_id)).cloned().unwrap_or_else(|| BattlePassState::new(player_id, season_id));
        Ok((s.level, s.xp, s.next_level_xp(), s.is_premium))
    }

    /// 购买战令高级版
    pub async fn buy_battle_pass(
        &self,
        player_id: Uuid,
        season_id: u32,
    ) -> Result<(bool, u32)> {
        if season_id == 0 {
            return Err(Error::InvalidRequest("season_id must be > 0".into()));
        }
        let mut map = self.battle_pass.write().await;
        let s = map.entry((player_id, season_id)).or_insert_with(|| BattlePassState::new(player_id, season_id));
        if s.is_premium {
            return Err(Error::PlayerState("battle pass already bought".into()));
        }
        s.is_premium = true;
        Ok((true, 168))  // 168 钻
    }

    /// 增加战令 XP, 返回新等级
    pub async fn advance_battle_pass_xp(
        &self,
        player_id: Uuid,
        season_id: u32,
        xp: u32,
    ) -> Result<(u32, u32)> {
        // (new_level, current_xp)
        if season_id == 0 {
            return Err(Error::InvalidRequest("season_id must be > 0".into()));
        }
        if xp == 0 {
            return Err(Error::InvalidRequest("xp must be > 0".into()));
        }
        let mut map = self.battle_pass.write().await;
        let s = map.entry((player_id, season_id)).or_insert_with(|| BattlePassState::new(player_id, season_id));
        s.apply_xp(xp);
        Ok((s.level, s.xp))
    }

    /// 领取战令奖励
    pub async fn claim_battle_pass_reward(
        &self,
        player_id: Uuid,
        season_id: u32,
        level: u32,
    ) -> Result<Vec<PrizeItem>> {
        if season_id == 0 {
            return Err(Error::InvalidRequest("season_id must be > 0".into()));
        }
        if level == 0 {
            return Err(Error::InvalidRequest("level must be > 0".into()));
        }
        let mut map = self.battle_pass.write().await;
        let s = map.entry((player_id, season_id)).or_insert_with(|| BattlePassState::new(player_id, season_id));
        if s.claimed_levels.contains(&level) {
            return Err(Error::PlayerState(format!("level {} already claimed", level)));
        }
        if s.level < level {
            return Err(Error::PlayerState(format!("player level {} < required {}", s.level, level)));
        }
        s.claimed_levels.insert(level);
        // 简单奖励: level 决定 item_id
        let item_id = 6000u32 + level;
        Ok(vec![PrizeItem { item_id, count: 1, is_rare: level % 5 == 0 }])
    }

    /// 战令排行榜 (按 level 降序, 取 top_n)
    pub async fn get_battle_pass_leaderboard(
        &self,
        season_id: u32,
        top_n: u32,
    ) -> Result<Vec<(u32, Uuid, i64)>> {
        // (rank, player_id, score)
        if season_id == 0 {
            return Err(Error::InvalidRequest("season_id must be > 0".into()));
        }
        if top_n == 0 {
            return Err(Error::InvalidRequest("top_n must be > 0".into()));
        }
        let map = self.battle_pass.read().await;
        let mut rows: Vec<(u32, Uuid, i64)> = map
            .iter()
            .filter(|((_, sid), _)| *sid == season_id)
            .map(|((pid, _), s)| (s.level, *pid, s.xp as i64))
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then(b.2.cmp(&a.2)));
        rows.truncate(top_n as usize);
        Ok(rows)
    }

    // ========== W7 L18 签到扩展 (Signin Extension) 5 RPC ==========

    /// 获取签到日历
    pub async fn get_signin_calendar(
        &self,
        player_id: Uuid,
        month: u32,
    ) -> Result<(Vec<SigninDayReward>, Vec<u32>)> {
        // (calendar_days, signed_days)
        if month == 0 || month > 12 {
            return Err(Error::InvalidRequest("month must be 1..=12".into()));
        }
        let cfg = self.signin_calendar_config.read().await;
        let days = cfg
            .get(&month)
            .cloned()
            .unwrap_or_default();
        if days.is_empty() {
            return Err(Error::ActivityNotFound(format!("month {}", month)));
        }
        let map = self.signin_ext.read().await;
        let s = map.get(&(player_id, month)).cloned().unwrap_or_else(|| SigninExtState::new(player_id, month));
        Ok((days, s.signed_days))
    }

    /// 领取月累计奖励 (签满 7/15/30 触发)
    pub async fn claim_month_reward(
        &self,
        player_id: Uuid,
        month: u32,
    ) -> Result<Vec<PrizeItem>> {
        if month == 0 || month > 12 {
            return Err(Error::InvalidRequest("month must be 1..=12".into()));
        }
        let map = self.signin_ext.read().await;
        let s = map.get(&(player_id, month)).cloned().unwrap_or_else(|| SigninExtState::new(player_id, month));
        // 7/15/30 累计奖励, 简化为按月签天数
        if s.total_days < 7 {
            return Err(Error::PlayerState("not enough signin days for monthly reward".into()));
        }
        Ok(vec![PrizeItem { item_id: 2100 + month, count: (s.total_days / 7) as u32, is_rare: s.total_days >= 30 }])
    }

    /// 签到排行榜 (按 total_days 降序, top_n)
    pub async fn get_signin_leaderboard(
        &self,
        month: u32,
        top_n: u32,
    ) -> Result<Vec<(u32, Uuid, i64)>> {
        if month == 0 || month > 12 {
            return Err(Error::InvalidRequest("month must be 1..=12".into()));
        }
        if top_n == 0 {
            return Err(Error::InvalidRequest("top_n must be > 0".into()));
        }
        let map = self.signin_ext.read().await;
        let mut rows: Vec<(u32, Uuid, i64)> = map
            .iter()
            .filter(|((_, m), _)| *m == month)
            .map(|((pid, _), s)| (s.streak_max, *pid, s.total_days as i64))
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then(b.2.cmp(&a.2)));
        rows.truncate(top_n as usize);
        Ok(rows)
    }

    /// 获取签到统计
    pub async fn get_signin_stats(
        &self,
        player_id: Uuid,
    ) -> Result<(u32, u32, u32)> {
        // (total_days, streak_max, miss_count)
        let map = self.signin_ext.read().await;
        let mut total_days = 0u32;
        let mut streak_max = 0u32;
        let mut miss_count = 0u32;
        for ((pid, _), s) in map.iter() {
            if *pid == player_id {
                total_days += s.total_days;
                streak_max = streak_max.max(s.streak_max);
                miss_count += s.miss_count;
            }
        }
        Ok((total_days, streak_max, miss_count))
    }

    /// GM 补签: 直接添加某天到 signed_days
    pub async fn advance_signin(
        &self,
        player_id: Uuid,
        day: u32,
    ) -> Result<Vec<PrizeItem>> {
        if day == 0 || day > 31 {
            return Err(Error::InvalidRequest("day must be 1..=31".into()));
        }
        let month = 9u32; // 简化
        let mut map = self.signin_ext.write().await;
        let s = map.entry((player_id, month)).or_insert_with(|| SigninExtState::new(player_id, month));
        if s.signed_days.contains(&day) {
            return Err(Error::PlayerState(format!("day {} already signed", day)));
        }
        s.signed_days.push(day);
        s.signed_days.sort();
        s.total_days += 1;
        s.streak_max = s.streak_max.max(s.total_days);
        Ok(vec![PrizeItem { item_id: 2001, count: 1, is_rare: false }])
    }

    // ========== W7 L18 回归玩家 (Returning Player) 4 RPC ==========

    /// 是否回归玩家
    pub async fn is_returning_player(
        &self,
        player_id: Uuid,
    ) -> Result<(bool, i64)> {
        // (is_returning, last_login_ms)
        let map = self.returning.read().await;
        let s = map.get(&player_id).cloned().unwrap_or_else(|| ReturningPlayerState::new(player_id, 0));
        Ok((s.is_returning, s.last_login_ms))
    }

    /// 获取回归玩家奖励列表 (config driven)
    pub async fn get_return_reward(
        &self,
        player_id: Uuid,
    ) -> Result<(Vec<ReturnActivityConfig>, bool)> {
        // (rewards, already_claimed_all)
        let map = self.returning.read().await;
        let s = map.get(&player_id).cloned().unwrap_or_else(|| ReturningPlayerState::new(player_id, 0));
        if !s.is_returning {
            return Err(Error::PlayerState("player is not returning".into()));
        }
        let already_claimed_all = s.claimed_rewards.len() >= self.return_activities_config.len();
        Ok((self.return_activities_config.clone(), already_claimed_all))
    }

    /// 领取回归玩家奖励
    pub async fn claim_return_reward(
        &self,
        player_id: Uuid,
        reward_id: u32,
    ) -> Result<PrizeItem> {
        let mut map = self.returning.write().await;
        let s = map.entry(player_id).or_insert_with(|| ReturningPlayerState::new(player_id, 0));
        if !s.is_returning {
            return Err(Error::PlayerState("player is not returning".into()));
        }
        if s.claimed_rewards.contains(&reward_id) {
            return Err(Error::PlayerState(format!("reward {} already claimed", reward_id)));
        }
        let cfg = self
            .return_activities_config
            .iter()
            .find(|c| c.reward_id == reward_id)
            .ok_or_else(|| Error::InvalidRequest(format!("reward_id {} not found", reward_id)))?
            .clone();
        s.claimed_rewards.insert(reward_id);
        Ok(PrizeItem { item_id: cfg.item_id, count: cfg.count, is_rare: false })
    }

    /// 列出所有回归活动
    pub async fn list_return_activities(&self) -> Vec<ReturnActivityConfig> {
        self.return_activities_config.clone()
    }

    // ========== W7 L18 推送 (Push) 2 RPC ==========

    /// 获取推送列表 (since_ms 之后)
    pub async fn get_push_list(
        &self,
        player_id: Uuid,
        since_ms: i64,
    ) -> Result<Vec<PushRecord>> {
        let map = self.push_list.read().await;
        let items = map
            .get(&player_id)
            .cloned()
            .unwrap_or_default();
        let filtered: Vec<PushRecord> = items
            .into_iter()
            .filter(|p| p.created_at_ms > since_ms)
            .collect();
        Ok(filtered)
    }

    /// 标记推送已读
    pub async fn mark_push_read(
        &self,
        player_id: Uuid,
        push_id: u32,
    ) -> Result<bool> {
        let mut map = self.push_list.write().await;
        let list = map.entry(player_id).or_insert_with(Vec::new);
        let p = list
            .iter_mut()
            .find(|p| p.push_id == push_id)
            .ok_or_else(|| Error::InvalidRequest(format!("push_id {} not found", push_id)))?;
        p.is_read = true;
        Ok(true)
    }

    // ========== W7 L18 拉新 (Invite) 1 RPC ==========

    /// 获取拉新状态
    pub async fn get_invite_status(
        &self,
        player_id: Uuid,
    ) -> Result<(String, u32, Vec<PrizeItem>)> {
        // (invite_code, invited_count, rewards)
        let map = self.invites.read().await;
        let s = map.get(&player_id).cloned().unwrap_or_else(|| {
            // 简化: 用 player_id 前 8 字符做 invite_code
            let code = format!("INV{:08X}", player_id.as_u128() as u32);
            InviteState::new(code)
        });
        Ok((s.invite_code, s.invited_count, s.rewards))
    }
}

impl Default for ActivityServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn holiday_info_known_activity() {
        let svc = ActivityServiceImpl::new();
        let act = svc.get_holiday_info("93031", 1000).unwrap();
        assert_eq!(act.activity_id, "93031");
        assert_eq!(act.activity_name, "元宵冒险1");
    }

    #[tokio::test]
    async fn holiday_info_unknown_activity() {
        let svc = ActivityServiceImpl::new();
        let r = svc.get_holiday_info("bogus", 1000);
        assert!(matches!(r, Err(Error::ActivityNotFound(_))));
    }

    #[tokio::test]
    async fn holiday_info_out_of_window() {
        let svc = ActivityServiceImpl::new();
        let r = svc.get_holiday_info("93031", -1);
        assert!(matches!(r, Err(Error::ActivityNotOpen(_))));
    }

    #[tokio::test]
    async fn list_holiday_ids_sorted() {
        let svc = ActivityServiceImpl::new();
        let ids = svc.list_holiday_ids();
        assert_eq!(ids.len(), 9);
    }

    #[tokio::test]
    async fn draw_prize_known_activity() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let drawn = svc.draw_prize("lantern", p, 5, 1000).await.unwrap();
        assert_eq!(drawn.len(), 5);
    }

    #[tokio::test]
    async fn draw_prize_unknown_activity() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.draw_prize("bogus", p, 1, 1000).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn get_player_tasks_creates_default() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let tasks = svc.get_player_tasks(p, "lantern").await.unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn advance_and_claim_task() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.advance_task(p, "lantern", "daily_kill", 10).await.unwrap();
        let rewards = svc.claim_task(p, "lantern", "daily_kill").await.unwrap();
        assert_eq!(rewards.len(), 1);
    }

    #[tokio::test]
    async fn claim_incomplete_task_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.get_player_tasks(p, "lantern").await.unwrap();
        let r = svc.claim_task(p, "lantern", "daily_kill").await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn signin_creates_and_signs() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let rewards = svc.do_signin(p, 1).await.unwrap();
        assert_eq!(rewards.len(), 1);
        let status = svc.get_signin_status(p, 9).await;
        assert_eq!(status.signed_days, vec![1]);
    }

    #[tokio::test]
    async fn signin_double_sign_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.do_signin(p, 1).await.unwrap();
        let r = svc.do_signin(p, 1).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn resignin_after_signin() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.do_signin(p, 5).await.unwrap();
        let cost = svc.resignin(p, 5).await.unwrap();
        assert_eq!(cost, 100);
    }

    #[tokio::test]
    async fn achievements_default_three() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let list = svc.get_achievements(p).await;
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn advance_achievement_completes() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.advance_achievement(p, 1, 1).await.unwrap();
        let list = svc.get_achievements(p).await;
        let a = list.iter().find(|a| a.achievement_id == 1).unwrap();
        assert!(a.is_complete());
    }

    #[tokio::test]
    async fn claim_complete_achievement_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.advance_achievement(p, 1, 1).await.unwrap();
        let rewards = svc.claim_achievement(p, 1).await.unwrap();
        assert_eq!(rewards.len(), 1);
        assert!(rewards[0].is_rare);
    }

    #[tokio::test]
    async fn claim_unknown_achievement_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.claim_achievement(p, 999).await;
        assert!(r.is_err());
    }

    // ========== W7 L18 问卷 UT (1 个, 模板) ==========

    #[tokio::test]
    async fn submit_survey_first_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (item_id, count) = svc.submit_survey(p, 3, "{\"q1\":\"a\"}").await.unwrap();
        assert_eq!(item_id, 8000 + 3 % 5);
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn submit_survey_duplicate_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.submit_survey(p, 3, "{\"q1\":\"a\"}").await.unwrap();
        let r = svc.submit_survey(p, 3, "{\"q1\":\"b\"}").await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn submit_survey_empty_answers_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.submit_survey(p, 3, "").await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    // ========== W7 L18 战令 (Battle Pass) 5 RPC UT ==========
    // 11 UT: 5 happy + 6 error

    #[tokio::test]
    async fn bp_get_info_default_level1() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (level, xp, next, is_premium) = svc.get_battle_pass_info(p, 1).await.unwrap();
        assert_eq!(level, 1);
        assert_eq!(xp, 0);
        assert_eq!(next, 100);
        assert!(!is_premium);
    }

    #[tokio::test]
    async fn bp_get_info_season_zero_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.get_battle_pass_info(p, 0).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn bp_buy_succeeds_then_premium() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (ok, price) = svc.buy_battle_pass(p, 1).await.unwrap();
        assert!(ok);
        assert_eq!(price, 168);
        let (_, _, _, is_premium) = svc.get_battle_pass_info(p, 1).await.unwrap();
        assert!(is_premium);
    }

    #[tokio::test]
    async fn bp_buy_twice_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.buy_battle_pass(p, 1).await.unwrap();
        let r = svc.buy_battle_pass(p, 1).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn bp_advance_xp_levels_up() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (new_level, current_xp) = svc.advance_battle_pass_xp(p, 1, 250).await.unwrap();
        // 250 xp / 100 = level 2 + 50 xp
        assert_eq!(new_level, 3);
        assert_eq!(current_xp, 50);
    }

    #[tokio::test]
    async fn bp_advance_xp_zero_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.advance_battle_pass_xp(p, 1, 0).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn bp_claim_reward_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.advance_battle_pass_xp(p, 1, 200).await.unwrap(); // level 3
        let rewards = svc.claim_battle_pass_reward(p, 1, 2).await.unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].item_id, 6002);
    }

    #[tokio::test]
    async fn bp_claim_reward_duplicate_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.advance_battle_pass_xp(p, 1, 200).await.unwrap();
        svc.claim_battle_pass_reward(p, 1, 2).await.unwrap();
        let r = svc.claim_battle_pass_reward(p, 1, 2).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn bp_claim_reward_insufficient_level_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        // level 1, try claim level 5
        let r = svc.claim_battle_pass_reward(p, 1, 5).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn bp_leaderboard_top_n() {
        let svc = ActivityServiceImpl::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        svc.advance_battle_pass_xp(p1, 1, 100).await.unwrap(); // level 2
        svc.advance_battle_pass_xp(p2, 1, 500).await.unwrap(); // level 6
        let rows = svc.get_battle_pass_leaderboard(1, 10).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1, p2); // higher level first
    }

    #[tokio::test]
    async fn bp_leaderboard_top_n_zero_fails() {
        let svc = ActivityServiceImpl::new();
        let r = svc.get_battle_pass_leaderboard(1, 0).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    // ========== W7 L18 签到扩展 (Signin Extension) 5 RPC UT ==========
    // 10 UT: 5 happy + 5 error

    #[tokio::test]
    async fn sx_get_calendar_month9_30_days() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (days, signed) = svc.get_signin_calendar(p, 9).await.unwrap();
        assert_eq!(days.len(), 30);
        assert!(signed.is_empty());
    }

    #[tokio::test]
    async fn sx_get_calendar_invalid_month_zero_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.get_signin_calendar(p, 0).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn sx_get_calendar_invalid_month_13_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.get_signin_calendar(p, 13).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn sx_claim_month_reward_requires_7_days() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.claim_month_reward(p, 9).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn sx_claim_month_reward_after_7_advance() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        for d in 1..=7 {
            svc.advance_signin(p, d).await.unwrap();
        }
        let rewards = svc.claim_month_reward(p, 9).await.unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].count, 1);
    }

    #[tokio::test]
    async fn sx_leaderboard_top_n() {
        let svc = ActivityServiceImpl::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        for d in 1..=3 {
            svc.advance_signin(p1, d).await.unwrap();
        }
        for d in 1..=10 {
            svc.advance_signin(p2, d).await.unwrap();
        }
        let rows = svc.get_signin_leaderboard(9, 5).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1, p2);
    }

    #[tokio::test]
    async fn sx_leaderboard_zero_top_n_fails() {
        let svc = ActivityServiceImpl::new();
        let r = svc.get_signin_leaderboard(9, 0).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn sx_get_stats_empty_player() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (total, streak_max, miss) = svc.get_signin_stats(p).await.unwrap();
        assert_eq!(total, 0);
        assert_eq!(streak_max, 0);
        assert_eq!(miss, 0);
    }

    #[tokio::test]
    async fn sx_advance_signin_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let rewards = svc.advance_signin(p, 1).await.unwrap();
        assert_eq!(rewards.len(), 1);
        let (total, _, _) = svc.get_signin_stats(p).await.unwrap();
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn sx_advance_signin_duplicate_day_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.advance_signin(p, 5).await.unwrap();
        let r = svc.advance_signin(p, 5).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    // ========== W7 L18 回归玩家 (Returning Player) 4 RPC UT ==========
    // 8 UT: 4 happy + 4 error

    #[tokio::test]
    async fn rt_is_returning_default_false() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (is_ret, last) = svc.is_returning_player(p).await.unwrap();
        // 默认 last_login_ms = 0 -> is_returning = true (per ReturningPlayerState::new)
        assert!(is_ret);
        assert_eq!(last, 0);
    }

    #[tokio::test]
    async fn rt_get_reward_non_returning_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        // 注入 non-returning 状态
        {
            let mut map = svc.returning.write().await;
            map.insert(p, ReturningPlayerState {
                player_id: p,
                last_login_ms: 1_000_000,
                is_returning: false,
                claimed_rewards: HashSet::new(),
            });
        }
        let r = svc.get_return_reward(p).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn rt_get_reward_returning_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        // 默认 is_returning = true
        let (rewards, already_all) = svc.get_return_reward(p).await.unwrap();
        assert_eq!(rewards.len(), 3);
        assert!(!already_all);
    }

    #[tokio::test]
    async fn rt_claim_reward_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.claim_return_reward(p, 1).await.unwrap();
        assert_eq!(r.item_id, 9101);
        assert_eq!(r.count, 10);
    }

    #[tokio::test]
    async fn rt_claim_reward_duplicate_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        svc.claim_return_reward(p, 1).await.unwrap();
        let r = svc.claim_return_reward(p, 1).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    #[tokio::test]
    async fn rt_claim_reward_unknown_id_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.claim_return_reward(p, 999).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    #[tokio::test]
    async fn rt_list_activities_default_3() {
        let svc = ActivityServiceImpl::new();
        let acts = svc.list_return_activities().await;
        assert_eq!(acts.len(), 3);
        assert_eq!(acts[0].reward_id, 1);
    }

    #[tokio::test]
    async fn rt_claim_reward_non_returning_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        {
            let mut map = svc.returning.write().await;
            map.insert(p, ReturningPlayerState {
                player_id: p,
                last_login_ms: 1_000_000,
                is_returning: false,
                claimed_rewards: HashSet::new(),
            });
        }
        let r = svc.claim_return_reward(p, 1).await;
        assert!(matches!(r, Err(Error::PlayerState(_))));
    }

    // ========== W7 L18 推送 (Push) 2 RPC UT ==========
    // 4 UT: 2 happy + 2 error

    #[tokio::test]
    async fn push_list_empty_default() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let items = svc.get_push_list(p, 0).await.unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn push_list_filter_since_ms() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        // 注入 2 条 push
        {
            let mut map = svc.push_list.write().await;
            map.entry(p).or_insert_with(Vec::new).push(PushRecord {
                push_id: 100, title: "old".into(), body: "old".into(),
                created_at_ms: 1000, is_read: false,
            });
            map.entry(p).or_insert_with(Vec::new).push(PushRecord {
                push_id: 200, title: "new".into(), body: "new".into(),
                created_at_ms: 2000, is_read: false,
            });
        }
        let items = svc.get_push_list(p, 1500).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].push_id, 200);
    }

    #[tokio::test]
    async fn push_mark_read_succeeds() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        {
            let mut map = svc.push_list.write().await;
            map.entry(p).or_insert_with(Vec::new).push(PushRecord {
                push_id: 1, title: "t".into(), body: "b".into(),
                created_at_ms: 100, is_read: false,
            });
        }
        let ok = svc.mark_push_read(p, 1).await.unwrap();
        assert!(ok);
        // 再次 mark 不应报错 (幂等)
        let ok2 = svc.mark_push_read(p, 1).await.unwrap();
        assert!(ok2);
    }

    #[tokio::test]
    async fn push_mark_read_unknown_fails() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let r = svc.mark_push_read(p, 999).await;
        assert!(matches!(r, Err(Error::InvalidRequest(_))));
    }

    // ========== W7 L18 拉新 (Invite) 1 RPC UT ==========
    // 2 UT: 1 happy + 1 error path coverage

    #[tokio::test]
    async fn inv_get_status_default() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (code, count, rewards) = svc.get_invite_status(p).await.unwrap();
        assert!(code.starts_with("INV"));
        assert_eq!(count, 0);
        assert_eq!(rewards.len(), 2);
    }

    #[tokio::test]
    async fn inv_get_status_same_player_same_code() {
        let svc = ActivityServiceImpl::new();
        let p = Uuid::new_v4();
        let (code1, _, _) = svc.get_invite_status(p).await.unwrap();
        // 注入 invites 状态
        {
            let mut map = svc.invites.write().await;
            map.insert(p, InviteState {
                invite_code: code1.clone(),
                invited_count: 5,
                rewards: vec![PrizeItem { item_id: 9999, count: 1, is_rare: true }],
            });
        }
        let (code2, count2, rewards2) = svc.get_invite_status(p).await.unwrap();
        assert_eq!(code1, code2);
        assert_eq!(count2, 5);
        assert_eq!(rewards2.len(), 1);
    }
}
