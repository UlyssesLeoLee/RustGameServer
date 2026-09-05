//! activity-service 域 entity 定义
//!
//! 核心 entity:
//! - HolidayActivity 复用 shared_platform (per 9/4 MD §4 反例, 1 套 + 9 配置)
//! - PlayerSignin 玩家签到记录
//! - Achievement 玩家成就

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use shared_platform::data_driven::{HolidayActivity, HolidayConfig, PvpMode, PvpConfig};

/// 玩家签到状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerSignin {
    pub player_id: Uuid,
    pub month: u32,
    pub signed_days: Vec<u32>,
    pub streak_days: u32,
    pub last_signin_at: DateTime<Utc>,
}

impl PlayerSignin {
    pub fn new(player_id: Uuid, month: u32) -> Self {
        Self {
            player_id,
            month,
            signed_days: Vec::new(),
            streak_days: 0,
            last_signin_at: Utc::now(),
        }
    }

    pub fn is_signed(&self, day: u32) -> bool {
        self.signed_days.contains(&day)
    }

    pub fn can_sign(&self, day: u32) -> bool {
        day >= 1 && day <= 31 && !self.is_signed(day)
    }

    pub fn sign(&mut self, day: u32) {
        if self.can_sign(day) {
            self.signed_days.push(day);
            self.signed_days.sort();
            self.streak_days += 1;
            self.last_signin_at = Utc::now();
        }
    }
}

/// 成就进度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Achievement {
    pub achievement_id: u32,
    pub name: String,
    pub current: u32,
    pub target: u32,
    pub claimed: bool,
}

impl Achievement {
    pub fn new(achievement_id: u32, name: &str, target: u32) -> Self {
        Self {
            achievement_id,
            name: name.to_string(),
            current: 0,
            target,
            claimed: false,
        }
    }

    pub fn progress_pct(&self) -> u32 {
        if self.target == 0 {
            return 100;
        }
        (self.current.min(self.target) * 100) / self.target
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.target
    }

    pub fn advance(&mut self, by: u32) {
        self.current = (self.current + by).min(self.target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signin_can_sign_first_day() {
        let mut s = PlayerSignin::new(Uuid::new_v4(), 9);
        assert!(s.can_sign(1));
        s.sign(1);
        assert!(s.is_signed(1));
        assert!(!s.can_sign(1));
    }

    #[test]
    fn signin_invalid_day_rejected() {
        let s = PlayerSignin::new(Uuid::new_v4(), 9);
        assert!(!s.can_sign(0));
        assert!(!s.can_sign(32));
    }

    #[test]
    fn signin_streak_increments() {
        let mut s = PlayerSignin::new(Uuid::new_v4(), 9);
        s.sign(1);
        s.sign(2);
        s.sign(3);
        assert_eq!(s.streak_days, 3);
        assert_eq!(s.signed_days, vec![1, 2, 3]);
    }

    #[test]
    fn achievement_progress_pct_zero_target() {
        let a = Achievement::new(1, "test", 0);
        assert_eq!(a.progress_pct(), 100);
    }

    #[test]
    fn achievement_progress_pct_partial() {
        let mut a = Achievement::new(1, "test", 100);
        a.advance(25);
        assert_eq!(a.progress_pct(), 25);
    }

    #[test]
    fn achievement_is_complete() {
        let mut a = Achievement::new(1, "test", 5);
        a.advance(5);
        assert!(a.is_complete());
    }

    #[test]
    fn achievement_advance_caps_at_target() {
        let mut a = Achievement::new(1, "test", 10);
        a.advance(100);
        assert_eq!(a.current, 10);
    }

    #[test]
    fn holiday_config_loaded_from_shared_platform() {
        // 9 个 holiday_* 来自 shared_platform (per 9/4 MD §4 反例)
        let cfg = HolidayConfig::default();
        assert_eq!(cfg.variant_count(), 9);
    }
}
