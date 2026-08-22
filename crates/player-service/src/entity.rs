//! player-service 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-018 §3 玩家域数据模型）
//! - Player：账号档案（昵称、等级、vip、状态、最近登录）
//! - PlayerSession：会话（device / ip / heartbeat / expires）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 玩家账号状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    /// 正常
    Active,
    /// 封禁
    Banned,
    /// 停用
    Disabled,
    /// 待激活
    Pending,
}

/// 玩家账号（root entity，per RGS-DTL-018 §3.1）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    /// 玩家 ID（业务主键）
    pub id: Uuid,
    /// 昵称（唯一）
    pub name: String,
    /// 等级（默认 1）
    pub level: i32,
    /// VIP 等级（0 = 非 VIP）
    pub vip_level: i32,
    /// 账号状态
    pub status: PlayerStatus,
    /// 最近登录时间（None = 从未登录）
    pub last_login_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Player {
    /// 工厂：新建玩家（默认 Active / Lv1 / VIP0）
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            level: 1,
            vip_level: 0,
            status: PlayerStatus::Active,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 玩家会话（per RGS-DTL-018 §3.2 active-active 跨服身份）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerSession {
    /// 会话 ID
    pub id: Uuid,
    /// 所属玩家 ID
    pub player_id: Uuid,
    /// 设备 ID
    pub device_id: String,
    /// 登录 IP
    pub ip: String,
    /// 登录时间
    pub login_at: DateTime<Utc>,
    /// 最近心跳时间
    pub last_heartbeat_at: DateTime<Utc>,
    /// 会话过期时间
    pub expires_at: DateTime<Utc>,
}

impl PlayerSession {
    /// 工厂：新建会话（默认 24h 过期）
    pub fn new(player_id: Uuid, device_id: String, ip: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            player_id,
            device_id,
            ip,
            login_at: now,
            last_heartbeat_at: now,
            expires_at: now + chrono::Duration::hours(24),
        }
    }

    /// 心跳刷新（更新 last_heartbeat_at + 滑动 expires_at）
    pub fn heartbeat(&mut self) {
        let now = Utc::now();
        self.last_heartbeat_at = now;
        self.expires_at = now + chrono::Duration::hours(24);
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_new_defaults() {
        let p = Player::new("alice".to_string());
        assert_eq!(p.name, "alice");
        assert_eq!(p.level, 1);
        assert_eq!(p.vip_level, 0);
        assert_eq!(p.status, PlayerStatus::Active);
        assert!(p.last_login_at.is_none());
    }

    #[test]
    fn player_session_heartbeat_slides_expiry() {
        let player_id = Uuid::new_v4();
        let mut s = PlayerSession::new(player_id, "dev-1".to_string(), "127.0.0.1".to_string());
        let old_expiry = s.expires_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        s.heartbeat();
        assert!(s.expires_at > old_expiry);
        assert!(!s.is_expired());
    }
}
