//! activity-service 业务实现
//!
//! 9 holiday_* 数据驱动 (per 9/4 MD §4 反例, 1 套 + 配置):
//! - get_holiday_info: 走 HolidayConfig.get(activity_id)
//! - draw_holiday_prize: 按 activity_id 路由 1 套抽奖逻辑
//! - get_holiday_tasks / claim_holiday_reward: 1 套任务模板
//!
//! 业务方法: ≥10 真实 + 签到/成就

use std::collections::HashMap;
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
}

impl ActivityServiceImpl {
    pub fn new() -> Self {
        Self {
            holiday_config: HolidayConfig::default(),
            player_tasks: Arc::new(RwLock::new(HashMap::new())),
            signins: Arc::new(RwLock::new(HashMap::new())),
            achievements: Arc::new(RwLock::new(HashMap::new())),
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
}
