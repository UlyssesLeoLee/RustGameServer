//! guild-service 域 entity
//!
//! 核心: Guild + GuildMember + GuildApplication

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 角色枚举 (0=member, 1=elder, 2=vice_leader, 3=leader)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuildRole {
    Member = 0,
    Elder = 1,
    ViceLeader = 2,
    Leader = 3,
}

impl GuildRole {
    pub fn can_kick(&self) -> bool {
        matches!(self, GuildRole::Leader | GuildRole::ViceLeader)
    }
    pub fn can_promote(&self) -> bool {
        matches!(self, GuildRole::Leader)
    }
    pub fn can_disband(&self) -> bool {
        matches!(self, GuildRole::Leader)
    }
    pub fn from_i32(v: i32) -> Self {
        match v {
            3 => GuildRole::Leader,
            2 => GuildRole::ViceLeader,
            1 => GuildRole::Elder,
            _ => GuildRole::Member,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guild {
    pub guild_id: Uuid,
    pub name: String,
    pub leader_id: Uuid,
    pub notice: String,
    pub level: u32,
    pub capacity: u32,
    pub created_at: DateTime<Utc>,
}

impl Guild {
    pub fn new(name: &str, leader_id: Uuid, capacity: u32) -> Self {
        Self {
            guild_id: Uuid::new_v4(),
            name: name.to_string(),
            leader_id,
            notice: String::new(),
            level: 1,
            capacity,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildMember {
    pub player_id: Uuid,
    pub display_name: String,
    pub role: GuildRole,
    pub contribution: u32,
    pub joined_at: DateTime<Utc>,
}

impl GuildMember {
    pub fn new(player_id: Uuid, display_name: &str, role: GuildRole) -> Self {
        Self {
            player_id,
            display_name: display_name.to_string(),
            role,
            contribution: 0,
            joined_at: Utc::now(),
        }
    }

    pub fn add_contribution(&mut self, by: u32) {
        self.contribution = self.contribution.saturating_add(by);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildApplication {
    pub guild_id: Uuid,
    pub applicant_id: Uuid,
    pub applied_at: DateTime<Utc>,
    pub status: ApplicationStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guild_role_can_kick() {
        assert!(GuildRole::Leader.can_kick());
        assert!(GuildRole::ViceLeader.can_kick());
        assert!(!GuildRole::Elder.can_kick());
        assert!(!GuildRole::Member.can_kick());
    }

    #[test]
    fn guild_role_can_promote_only_leader() {
        assert!(GuildRole::Leader.can_promote());
        assert!(!GuildRole::ViceLeader.can_promote());
    }

    #[test]
    fn guild_role_from_i32() {
        assert_eq!(GuildRole::from_i32(3), GuildRole::Leader);
        assert_eq!(GuildRole::from_i32(0), GuildRole::Member);
        assert_eq!(GuildRole::from_i32(99), GuildRole::Member);
    }

    #[test]
    fn guild_factory_default_level() {
        let g = Guild::new("test", Uuid::new_v4(), 50);
        assert_eq!(g.level, 1);
        assert_eq!(g.capacity, 50);
    }

    #[test]
    fn member_contribution_saturates() {
        let mut m = GuildMember::new(Uuid::new_v4(), "alice", GuildRole::Member);
        m.add_contribution(100);
        m.add_contribution(u32::MAX);
        assert_eq!(m.contribution, u32::MAX);
    }
}
