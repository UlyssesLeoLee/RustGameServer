//! social-service 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository
//! 规范：RGS-DTL-026 §3 社交域数据访问层

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{Guild, GuildMember, GuildRole};
use crate::Result;

/// Guild Repository trait
#[async_trait]
pub trait GuildRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Guild>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Guild>>;
    async fn save(&self, entity: &Guild) -> Result<Guild>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    async fn list_by_leader(&self, leader_id: Uuid) -> Result<Vec<Guild>>;
}

/// GuildMember Repository trait
#[async_trait]
pub trait GuildMemberRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GuildMember>>;
    async fn list_by_guild(&self, guild_id: Uuid) -> Result<Vec<GuildMember>>;
    async fn find_by_player(&self, player_id: Uuid) -> Result<Vec<GuildMember>>;
    async fn save(&self, entity: &GuildMember) -> Result<GuildMember>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgGuildRepository {
    pool: PgPool,
}

impl PgGuildRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_guild(row: sqlx::postgres::PgRow) -> Guild {
    Guild {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        leader_id: row.get("leader_id"),
        level: row.get("level"),
        member_count: row.get("member_count"),
        experience: row.get("experience"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl GuildRepository for PgGuildRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Guild>> {
        let row = sqlx::query(
            "SELECT id, name, description, leader_id, level, member_count, experience, created_at, updated_at \
             FROM guilds WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_guild))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Guild>> {
        let row = sqlx::query(
            "SELECT id, name, description, leader_id, level, member_count, experience, created_at, updated_at \
             FROM guilds WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_guild))
    }

    async fn save(&self, entity: &Guild) -> Result<Guild> {
        sqlx::query(
            "INSERT INTO guilds \
             (id, name, description, leader_id, level, member_count, experience, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (id) DO UPDATE SET \
                name = EXCLUDED.name, description = EXCLUDED.description, \
                leader_id = EXCLUDED.leader_id, level = EXCLUDED.level, \
                member_count = EXCLUDED.member_count, experience = EXCLUDED.experience, \
                updated_at = EXCLUDED.updated_at",
        )
        .bind(entity.id)
        .bind(&entity.name)
        .bind(&entity.description)
        .bind(entity.leader_id)
        .bind(entity.level)
        .bind(entity.member_count)
        .bind(entity.experience)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM guilds WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_by_leader(&self, leader_id: Uuid) -> Result<Vec<Guild>> {
        let rows = sqlx::query(
            "SELECT id, name, description, leader_id, level, member_count, experience, created_at, updated_at \
             FROM guilds WHERE leader_id = $1 ORDER BY created_at DESC",
        )
        .bind(leader_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_guild).collect())
    }
}

pub struct PgGuildMemberRepository {
    pool: PgPool,
}

impl PgGuildMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_member(row: sqlx::postgres::PgRow) -> GuildMember {
    let role_str: String = row.get("role");
    GuildMember {
        id: row.get("id"),
        guild_id: row.get("guild_id"),
        player_id: row.get("player_id"),
        role: parse_role(&role_str),
        contribution: row.get("contribution"),
        joined_at: row.get("joined_at"),
    }
}

#[async_trait]
impl GuildMemberRepository for PgGuildMemberRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GuildMember>> {
        let row = sqlx::query(
            "SELECT id, guild_id, player_id, role, contribution, joined_at \
             FROM guild_members WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_member))
    }

    async fn list_by_guild(&self, guild_id: Uuid) -> Result<Vec<GuildMember>> {
        let rows = sqlx::query(
            "SELECT id, guild_id, player_id, role, contribution, joined_at \
             FROM guild_members WHERE guild_id = $1 ORDER BY joined_at",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_member).collect())
    }

    async fn find_by_player(&self, player_id: Uuid) -> Result<Vec<GuildMember>> {
        let rows = sqlx::query(
            "SELECT id, guild_id, player_id, role, contribution, joined_at \
             FROM guild_members WHERE player_id = $1",
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_member).collect())
    }

    async fn save(&self, entity: &GuildMember) -> Result<GuildMember> {
        sqlx::query(
            "INSERT INTO guild_members (id, guild_id, player_id, role, contribution, joined_at) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (id) DO UPDATE SET \
                role = EXCLUDED.role, contribution = EXCLUDED.contribution",
        )
        .bind(entity.id)
        .bind(entity.guild_id)
        .bind(entity.player_id)
        .bind(role_to_str(entity.role))
        .bind(entity.contribution)
        .bind(entity.joined_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM guild_members WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryGuildRepository {
    inner: Mutex<HashMap<Uuid, Guild>>,
}

impl InMemoryGuildRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryGuildRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GuildRepository for InMemoryGuildRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Guild>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_name(&self, name: &str) -> Result<Option<Guild>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|g| g.name == name)
            .cloned())
    }
    async fn save(&self, entity: &Guild) -> Result<Guild> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
    async fn list_by_leader(&self, leader_id: Uuid) -> Result<Vec<Guild>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|g| g.leader_id == leader_id)
            .cloned()
            .collect())
    }
}

pub struct InMemoryGuildMemberRepository {
    inner: Mutex<HashMap<Uuid, GuildMember>>,
}

impl InMemoryGuildMemberRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryGuildMemberRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GuildMemberRepository for InMemoryGuildMemberRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GuildMember>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_guild(&self, guild_id: Uuid) -> Result<Vec<GuildMember>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.guild_id == guild_id)
            .cloned()
            .collect())
    }
    async fn find_by_player(&self, player_id: Uuid) -> Result<Vec<GuildMember>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.player_id == player_id)
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &GuildMember) -> Result<GuildMember> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
}

// ============================================================================
// helpers
// ============================================================================

fn role_to_str(r: GuildRole) -> &'static str {
    match r {
        GuildRole::Leader => "leader",
        GuildRole::Officer => "officer",
        GuildRole::Member => "member",
    }
}

fn parse_role(s: &str) -> GuildRole {
    match s {
        "leader" => GuildRole::Leader,
        "officer" => GuildRole::Officer,
        _ => GuildRole::Member,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_guild_crud() {
        let repo = InMemoryGuildRepository::new();
        let g = Guild::new("g1".to_string(), "d".to_string(), Uuid::new_v4());
        let id = g.id;
        repo.save(&g).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.name, "g1");
        assert_eq!(found.level, 1);
    }

    #[tokio::test]
    async fn in_memory_member_find_by_player() {
        let repo = InMemoryGuildMemberRepository::new();
        let player_id = Uuid::new_v4();
        repo.save(&GuildMember::new(Uuid::new_v4(), player_id))
            .await
            .unwrap();
        let list = repo.find_by_player(player_id).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn in_memory_guild_delete_by_id() {
        let repo = InMemoryGuildRepository::new();
        let g = Guild::new("g".to_string(), "".to_string(), Uuid::new_v4());
        let id = g.id;
        repo.save(&g).await.unwrap();
        assert!(repo.delete_by_id(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
        // 二次删除返回 false
        assert!(!repo.delete_by_id(id).await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_guild_find_by_name() {
        let repo = InMemoryGuildRepository::new();
        repo.save(&Guild::new("alpha".to_string(), "".to_string(), Uuid::new_v4()))
            .await
            .unwrap();
        let found = repo.find_by_name("alpha").await.unwrap();
        assert!(found.is_some());
        assert!(repo.find_by_name("nonexistent").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_guild_list_by_leader() {
        let repo = InMemoryGuildRepository::new();
        let leader = Uuid::new_v4();
        repo.save(&Guild::new("A".to_string(), "".to_string(), leader))
            .await
            .unwrap();
        repo.save(&Guild::new("B".to_string(), "".to_string(), leader))
            .await
            .unwrap();
        repo.save(&Guild::new("C".to_string(), "".to_string(), Uuid::new_v4()))
            .await
            .unwrap();
        let by_leader = repo.list_by_leader(leader).await.unwrap();
        assert_eq!(by_leader.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_member_list_by_guild() {
        let repo = InMemoryGuildMemberRepository::new();
        let gid = Uuid::new_v4();
        repo.save(&GuildMember::new(gid, Uuid::new_v4()))
            .await
            .unwrap();
        repo.save(&GuildMember::new(gid, Uuid::new_v4()))
            .await
            .unwrap();
        repo.save(&GuildMember::new(Uuid::new_v4(), Uuid::new_v4()))
            .await
            .unwrap();
        let list = repo.list_by_guild(gid).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_member_delete_by_id() {
        let repo = InMemoryGuildMemberRepository::new();
        let m = GuildMember::new(Uuid::new_v4(), Uuid::new_v4());
        let id = m.id;
        repo.save(&m).await.unwrap();
        assert!(repo.delete_by_id(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Guild save 后 find_by_id 必能拿回原 entity
        #[test]
        fn guild_in_memory_save_find_roundtrip(
            name in "[A-Za-z0-9]{1,32}",
            desc in ".*",
            leader_bytes in any::<[u8; 16]>(),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let repo = InMemoryGuildRepository::new();
                let leader = Uuid::from_bytes(leader_bytes);
                let g = Guild::new(name.clone(), desc.clone(), leader);
                let id = g.id;
                repo.save(&g).await.unwrap();
                let back = repo.find_by_id(id).await.unwrap().unwrap();
                prop_assert_eq!(back.name, name);
                prop_assert_eq!(back.description, desc);
                prop_assert_eq!(back.leader_id, leader);
                prop_assert_eq!(back.level, g.level);
                prop_assert_eq!(back.member_count, g.member_count);
                prop_assert_eq!(back.experience, g.experience);
                Ok(())
            });
        }

        /// Guild save 覆盖原 entity (同 id 二次 save 后 find_by_id 返回新值)
        #[test]
        fn guild_in_memory_save_overwrites(
            name1 in "[A-Za-z0-9]{1,16}",
            name2 in "[A-Za-z0-9]{1,16}",
            leader_bytes in any::<[u8; 16]>(),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let repo = InMemoryGuildRepository::new();
                let leader = Uuid::from_bytes(leader_bytes);
                let mut g = Guild::new(name1, "".to_string(), leader);
                let id = g.id;
                repo.save(&g).await.unwrap();
                g.name = name2.clone();
                repo.save(&g).await.unwrap();
                let back = repo.find_by_id(id).await.unwrap().unwrap();
                prop_assert_eq!(back.name, name2);
                Ok(())
            });
        }

        /// GuildMember 按 player_id 查找不变式:save 一个 player, 必查到 1 个
        #[test]
        fn guild_member_find_by_player_count(
            guild_bytes in any::<[u8; 16]>(),
            player_bytes in any::<[u8; 16]>(),
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let repo = InMemoryGuildMemberRepository::new();
                let gid = Uuid::from_bytes(guild_bytes);
                let pid = Uuid::from_bytes(player_bytes);
                repo.save(&GuildMember::new(gid, pid)).await.unwrap();
                let list = repo.find_by_player(pid).await.unwrap();
                prop_assert_eq!(list.len(), 1);
                prop_assert_eq!(list[0].player_id, pid);
                Ok(())
            });
        }
    }
}
