//! social-service 域 entity 定义
//!
//! 54.6 实化：2 个核心 entity（per RGS-DTL-026 §3 社交域数据模型）
//! - Guild：公会
//! - GuildMember：公会成员

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 公会成员角色
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuildRole {
    /// 会长
    Leader,
    /// 副会长
    Officer,
    /// 普通成员
    Member,
}

/// 公会（root entity，per RGS-DTL-026 §3.1）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guild {
    /// 公会 ID
    pub id: Uuid,
    /// 公会名（唯一）
    pub name: String,
    /// 公会描述
    pub description: String,
    /// 会长 ID
    pub leader_id: Uuid,
    /// 等级
    pub level: i32,
    /// 成员数
    pub member_count: i32,
    /// 经验值
    pub experience: i64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Guild {
    /// 工厂：新建公会
    pub fn new(name: String, description: String, leader_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            leader_id,
            level: 1,
            member_count: 1,
            experience: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// 加经验（自动升级检查）
    pub fn add_experience(&mut self, exp: i64) {
        self.experience += exp;
        // 简单升级曲线：100 * level^2 经验升一级
        while self.experience >= (100 * (self.level as i64).pow(2)) {
            self.experience -= 100 * (self.level as i64).pow(2);
            self.level += 1;
        }
        self.updated_at = Utc::now();
    }
}

/// 公会成员（per RGS-DTL-026 §3.2）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildMember {
    /// 成员记录 ID
    pub id: Uuid,
    /// 公会 ID
    pub guild_id: Uuid,
    /// 玩家 ID
    pub player_id: Uuid,
    /// 角色
    pub role: GuildRole,
    /// 贡献值
    pub contribution: i64,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
}

impl GuildMember {
    /// 工厂：新建成员（默认 Member）
    pub fn new(guild_id: Uuid, player_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            guild_id,
            player_id,
            role: GuildRole::Member,
            contribution: 0,
            joined_at: Utc::now(),
        }
    }

    /// 提升为副会长
    pub fn promote_to_officer(&mut self) {
        self.role = GuildRole::Officer;
    }

    /// 降为普通成员
    pub fn demote_to_member(&mut self) {
        self.role = GuildRole::Member;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guild_add_experience_levels_up() {
        let mut g = Guild::new("g1".to_string(), "d".to_string(), Uuid::new_v4());
        assert_eq!(g.level, 1);
        g.add_experience(100); // Lv1→Lv2 需 100 exp
        assert_eq!(g.level, 2);
        assert_eq!(g.experience, 0);
    }

    #[test]
    fn guild_member_promote_demote() {
        let mut m = GuildMember::new(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(m.role, GuildRole::Member);
        m.promote_to_officer();
        assert_eq!(m.role, GuildRole::Officer);
        m.demote_to_member();
        assert_eq!(m.role, GuildRole::Member);
    }
}
