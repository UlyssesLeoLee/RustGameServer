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

    #[test]
    fn guild_factory_initial_state() {
        let leader = Uuid::new_v4();
        let g = Guild::new("Test".to_string(), "desc".to_string(), leader);
        assert_eq!(g.name, "Test");
        assert_eq!(g.leader_id, leader);
        assert_eq!(g.level, 1);
        assert_eq!(g.member_count, 1);
        assert_eq!(g.experience, 0);
        assert_eq!(g.created_at, g.updated_at);
    }

    #[test]
    fn guild_role_serde_roundtrip() {
        for r in [GuildRole::Leader, GuildRole::Officer, GuildRole::Member] {
            let json = serde_json::to_string(&r).unwrap();
            let back: GuildRole = serde_json::from_str(&json).unwrap();
            assert_eq!(r, back);
        }
    }

    #[test]
    fn guild_multi_level_up() {
        let mut g = Guild::new("X".to_string(), "".to_string(), Uuid::new_v4());
        // Lv1→2: 100, Lv2→3: 400, Lv3→4: 900 → 累计 1400 升到 Lv4
        g.add_experience(1400);
        assert_eq!(g.level, 4);
    }

    #[test]
    fn guild_member_initial_role_is_member() {
        let m = GuildMember::new(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(m.role, GuildRole::Member);
        assert_eq!(m.contribution, 0);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// UUID 字符串往返不变式:任何 16 字节序列都应能正确 roundtrip
        #[test]
        fn guild_id_string_roundtrip(bytes in any::<[u8; 16]>()) {
            let id = Uuid::from_bytes(bytes);
            let s = id.to_string();
            let parsed = Uuid::parse_str(&s).unwrap();
            prop_assert_eq!(id, parsed);
        }

        /// 新建 Guild 的初始不变式:level=1, member_count=1, experience=0
        /// 且 updated_at >= created_at (按时间推进单调性)
        #[test]
        fn guild_new_invariants(
            name in "[A-Za-z0-9 ]{1,32}",
            desc in ".*",
            leader_bytes in any::<[u8; 16]>(),
        ) {
            let leader = Uuid::from_bytes(leader_bytes);
            let g = Guild::new(name.clone(), desc.clone(), leader);
            prop_assert_eq!(g.level, 1);
            prop_assert_eq!(g.member_count, 1);
            prop_assert_eq!(g.experience, 0);
            prop_assert_eq!(g.name, name);
            prop_assert_eq!(g.description, desc);
            prop_assert_eq!(g.leader_id, leader);
            prop_assert_eq!(g.created_at, g.updated_at);
        }

        /// add_experience 非负时:level 单调不减
        #[test]
        fn guild_level_monotonic_nondecreasing(exp in 0i64..10_000) {
            let mut g = Guild::new("G".to_string(), "".to_string(), Uuid::new_v4());
            let before = g.level;
            g.add_experience(exp);
            prop_assert!(g.level >= before, "level must not decrease: before={} after={}", before, g.level);
        }

        /// GuildMember 新建不变式:role=Member, contribution=0
        #[test]
        fn guild_member_new_invariants(
            guild_bytes in any::<[u8; 16]>(),
            player_bytes in any::<[u8; 16]>(),
        ) {
            let gid = Uuid::from_bytes(guild_bytes);
            let pid = Uuid::from_bytes(player_bytes);
            let m = GuildMember::new(gid, pid);
            prop_assert_eq!(m.role, GuildRole::Member);
            prop_assert_eq!(m.contribution, 0);
            prop_assert_eq!(m.guild_id, gid);
            prop_assert_eq!(m.player_id, pid);
        }

        /// promote_to_officer 终态:role 必为 Officer
        #[test]
        fn promote_to_officer_idempotent_on_role(
            guild_bytes in any::<[u8; 16]>(),
            player_bytes in any::<[u8; 16]>(),
        ) {
            let mut m = GuildMember::new(
                Uuid::from_bytes(guild_bytes),
                Uuid::from_bytes(player_bytes),
            );
            m.promote_to_officer();
            prop_assert_eq!(m.role, GuildRole::Officer);
            // 重复 promote 不应改变状态
            m.promote_to_officer();
            prop_assert_eq!(m.role, GuildRole::Officer);
        }
    }
}
