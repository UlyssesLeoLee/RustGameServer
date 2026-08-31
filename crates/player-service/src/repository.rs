//! player-service 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository 测试用
//! 规范：RGS-DTL-018 §3 玩家域数据访问层
//!
//! 设计原则：
//! - trait 抽象数据访问，不绑定具体实现
//! - PgRepository：生产用 sqlx PgPool（非宏版 query，运行时绑定）
//! - InMemoryRepository：单测用，验证 trait 行为一致性
//! - list_paginated：分页查询（per common.proto PageRequest/PageResponse）
//!
//! 桶 11 增量 (per DTL-038 §4.3 + §7.1 + FR-002 + DEC-038-01)：
//! - DeckRepository trait + PgDeckRepository sqlx impl + InMemoryDeckRepository 测试用
//! - 7 方法: create_deck / get_deck / update_deck / delete_deck / list_decks / share_deck / get_shared_deck

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{Deck, DeckSlot, DeckStatus, Player, PlayerSession, PlayerStatus};
use crate::Result;

/// 分页请求（per common.proto PageRequest）
#[derive(Debug, Clone)]
pub struct PageRequest {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// 分页响应（per common.proto PageResponse）
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

/// Player Repository trait
#[async_trait]
pub trait PlayerRepository: Send + Sync {
    /// 按 id 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Player>>;
    /// 按昵称查询（unique index）
    async fn find_by_name(&self, name: &str) -> Result<Option<Player>>;
    /// 保存（insert / update）
    async fn save(&self, entity: &Player) -> Result<Player>;
    /// 按 id 删除
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 分页列出所有玩家
    async fn list_paginated(&self, req: PageRequest) -> Result<Page<Player>>;
}

/// PlayerSession Repository trait
#[async_trait]
pub trait PlayerSessionRepository: Send + Sync {
    /// 按 id 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PlayerSession>>;
    /// 按 player_id 列出所有活跃会话
    async fn list_by_player(&self, player_id: Uuid) -> Result<Vec<PlayerSession>>;
    /// 保存
    async fn save(&self, entity: &PlayerSession) -> Result<PlayerSession>;
    /// 按 id 删除
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 清理过期会话
    async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64>;
}

/// Deck Repository trait（per DTL-038 §4.3 + §7.1 + FR-002，桶 11 增量）
#[async_trait]
pub trait DeckRepository: Send + Sync {
    /// 创建卡组（id 由 entity 提供；返回持久化后的 entity）
    async fn create(&self, entity: &Deck) -> Result<Deck>;
    /// 按 id 查询
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Deck>>;
    /// 全量更新（name / mode / slots / status / is_public / share_code / like_count / updated_at）
    async fn update(&self, entity: &Deck) -> Result<Deck>;
    /// 按 id 删除
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 按 owner_id 分页查询
    async fn list_by_owner(
        &self,
        owner_id: Uuid,
        req: PageRequest,
    ) -> Result<Page<Deck>>;
    /// 按 share_code 查询（用于 GetSharedDeck；要求 is_public=true）
    async fn find_by_share_code(&self, share_code: &str) -> Result<Option<Deck>>;
}

// ============================================================================
// PgRepository（sqlx 实现，生产用）
// ============================================================================

/// sqlx PgPool 实现
pub struct PgPlayerRepository {
    pool: PgPool,
}

impl PgPlayerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_player(row: sqlx::postgres::PgRow) -> Player {
    let status_str: String = row.get("status");
    Player {
        id: row.get("id"),
        name: row.get("name"),
        level: row.get("level"),
        vip_level: row.get("vip_level"),
        status: parse_status(&status_str),
        last_login_at: row.get("last_login_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl PlayerRepository for PgPlayerRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Player>> {
        let row = sqlx::query(
            "SELECT id, name, level, vip_level, status, last_login_at, created_at, updated_at \
             FROM players WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_player))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Player>> {
        let row = sqlx::query(
            "SELECT id, name, level, vip_level, status, last_login_at, created_at, updated_at \
             FROM players WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_player))
    }

    async fn save(&self, entity: &Player) -> Result<Player> {
        let status_str = status_to_str(entity.status);
        sqlx::query(
            "INSERT INTO players (id, name, level, vip_level, status, last_login_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                name = EXCLUDED.name, \
                level = EXCLUDED.level, \
                vip_level = EXCLUDED.vip_level, \
                status = EXCLUDED.status, \
                last_login_at = EXCLUDED.last_login_at, \
                updated_at = EXCLUDED.updated_at",
        )
        .bind(entity.id)
        .bind(&entity.name)
        .bind(entity.level)
        .bind(entity.vip_level)
        .bind(status_str)
        .bind(entity.last_login_at)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM players WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_paginated(&self, req: PageRequest) -> Result<Page<Player>> {
        let offset = ((req.page.saturating_sub(1)) * req.page_size) as i64;
        let limit = req.page_size as i64;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM players")
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query(
            "SELECT id, name, level, vip_level, status, last_login_at, created_at, updated_at \
             FROM players ORDER BY created_at DESC OFFSET $1 LIMIT $2",
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let items = rows.into_iter().map(row_to_player).collect();
        Ok(Page {
            items,
            total,
            page: req.page,
            page_size: req.page_size,
        })
    }
}

pub struct PgPlayerSessionRepository {
    pool: PgPool,
}

impl PgPlayerSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_session(row: sqlx::postgres::PgRow) -> PlayerSession {
    PlayerSession {
        id: row.get("id"),
        player_id: row.get("player_id"),
        device_id: row.get("device_id"),
        ip: row.get("ip"),
        login_at: row.get("login_at"),
        last_heartbeat_at: row.get("last_heartbeat_at"),
        expires_at: row.get("expires_at"),
    }
}

#[async_trait]
impl PlayerSessionRepository for PgPlayerSessionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PlayerSession>> {
        let row = sqlx::query(
            "SELECT id, player_id, device_id, ip, login_at, last_heartbeat_at, expires_at \
             FROM player_sessions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_session))
    }

    async fn list_by_player(&self, player_id: Uuid) -> Result<Vec<PlayerSession>> {
        let rows = sqlx::query(
            "SELECT id, player_id, device_id, ip, login_at, last_heartbeat_at, expires_at \
             FROM player_sessions WHERE player_id = $1 ORDER BY login_at DESC",
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_session).collect())
    }

    async fn save(&self, entity: &PlayerSession) -> Result<PlayerSession> {
        sqlx::query(
            "INSERT INTO player_sessions \
             (id, player_id, device_id, ip, login_at, last_heartbeat_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (id) DO UPDATE SET \
                last_heartbeat_at = EXCLUDED.last_heartbeat_at, \
                expires_at = EXCLUDED.expires_at",
        )
        .bind(entity.id)
        .bind(entity.player_id)
        .bind(&entity.device_id)
        .bind(&entity.ip)
        .bind(entity.login_at)
        .bind(entity.last_heartbeat_at)
        .bind(entity.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM player_sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query("DELETE FROM player_sessions WHERE expires_at < $1")
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

// ============================================================================
// InMemoryRepository（单测用，验证 trait 行为）
// ============================================================================

pub struct InMemoryPlayerRepository {
    inner: Mutex<HashMap<Uuid, Player>>,
}

impl InMemoryPlayerRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPlayerRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlayerRepository for InMemoryPlayerRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Player>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_name(&self, name: &str) -> Result<Option<Player>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|p| p.name == name)
            .cloned())
    }
    async fn save(&self, entity: &Player) -> Result<Player> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
    async fn list_paginated(&self, req: PageRequest) -> Result<Page<Player>> {
        let guard = self.inner.lock().unwrap();
        let mut all: Vec<Player> = guard.values().cloned().collect();
        all.sort_by_key(|p| std::cmp::Reverse(p.created_at));
        let total = all.len() as i64;
        let offset = ((req.page.saturating_sub(1)) * req.page_size) as usize;
        let items = all
            .into_iter()
            .skip(offset)
            .take(req.page_size as usize)
            .collect();
        Ok(Page {
            items,
            total,
            page: req.page,
            page_size: req.page_size,
        })
    }
}

pub struct InMemoryPlayerSessionRepository {
    inner: Mutex<HashMap<Uuid, PlayerSession>>,
}

impl InMemoryPlayerSessionRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPlayerSessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlayerSessionRepository for InMemoryPlayerSessionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PlayerSession>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_player(&self, player_id: Uuid) -> Result<Vec<PlayerSession>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.player_id == player_id)
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &PlayerSession) -> Result<PlayerSession> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
    async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64> {
        let mut guard = self.inner.lock().unwrap();
        let before = guard.len();
        guard.retain(|_, s| s.expires_at >= now);
        Ok((before - guard.len()) as u64)
    }
}

// ============================================================================
// Deck Repository impls (per DTL-038 §4.3 + §7.1, 桶 11 增量)
// ============================================================================

// ============================================================================
// PgDeckRepository (sqlx 实现, 生产用)
// ============================================================================

/// sqlx PgPool 实现
pub struct PgDeckRepository {
    pool: PgPool,
}

impl PgDeckRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// slots 列 (JSONB) 序列化: Vec<DeckSlot> → serde_json::Value
fn slots_to_jsonb(slots: &[DeckSlot]) -> Result<serde_json::Value> {
    serde_json::to_value(slots).map_err(|e| {
        crate::Error::Internal(anyhow::anyhow!("serialize slots to JSONB failed: {}", e))
    })
}

/// slots 列 (JSONB) 反序列化: serde_json::Value → Vec<DeckSlot>
fn jsonb_to_slots(value: serde_json::Value) -> Result<Vec<DeckSlot>> {
    serde_json::from_value(value).map_err(|e| {
        crate::Error::Internal(anyhow::anyhow!("deserialize slots from JSONB failed: {}", e))
    })
}

fn row_to_deck(row: sqlx::postgres::PgRow) -> Result<Deck> {
    let status_i16: i16 = row.get("status");
    let status_num = i32::from(status_i16);
    let status = match status_num {
        2 => DeckStatus::Active,
        3 => DeckStatus::Archived,
        _ => DeckStatus::Draft,
    };
    let mode_i16: i16 = row.get("mode");
    let mode = i32::from(mode_i16);
    let slots_jsonb: serde_json::Value = row.get("slots");
    let slots = jsonb_to_slots(slots_jsonb)?;
    let like_count_i32: i32 = row.get("like_count");
    let like_count = like_count_i32.max(0) as u32;

    Ok(Deck {
        id: row.get("deck_id"),
        owner_id: row.get("owner_id"),
        name: row.get("name"),
        mode,
        slots,
        status,
        is_public: row.get("is_public"),
        share_code: row.get("share_code"),
        like_count,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

#[async_trait]
impl DeckRepository for PgDeckRepository {
    async fn create(&self, entity: &Deck) -> Result<Deck> {
        let slots_jsonb = slots_to_jsonb(&entity.slots)?;
        let status_num: i16 = match entity.status {
            DeckStatus::Draft => 1,
            DeckStatus::Active => 2,
            DeckStatus::Archived => 3,
        };
        let mode_num: i16 = entity.mode as i16;
        let like_count_i32: i32 = entity.like_count as i32;

        sqlx::query(
            "INSERT INTO decks \
             (deck_id, owner_id, name, mode, slots, status, is_public, share_code, like_count, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(entity.id)
        .bind(entity.owner_id)
        .bind(&entity.name)
        .bind(mode_num)
        .bind(slots_jsonb)
        .bind(status_num)
        .bind(entity.is_public)
        .bind(entity.share_code.as_deref())
        .bind(like_count_i32)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Deck>> {
        let row = sqlx::query(
            "SELECT deck_id, owner_id, name, mode, slots, status, is_public, share_code, like_count, created_at, updated_at \
             FROM decks WHERE deck_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_deck(r)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, entity: &Deck) -> Result<Deck> {
        let slots_jsonb = slots_to_jsonb(&entity.slots)?;
        let status_num: i16 = match entity.status {
            DeckStatus::Draft => 1,
            DeckStatus::Active => 2,
            DeckStatus::Archived => 3,
        };
        let mode_num: i16 = entity.mode as i16;
        let like_count_i32: i32 = entity.like_count as i32;

        sqlx::query(
            "UPDATE decks SET \
                owner_id = $2, \
                name = $3, \
                mode = $4, \
                slots = $5, \
                status = $6, \
                is_public = $7, \
                share_code = $8, \
                like_count = $9, \
                updated_at = $10 \
             WHERE deck_id = $1",
        )
        .bind(entity.id)
        .bind(entity.owner_id)
        .bind(&entity.name)
        .bind(mode_num)
        .bind(slots_jsonb)
        .bind(status_num)
        .bind(entity.is_public)
        .bind(entity.share_code.as_deref())
        .bind(like_count_i32)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM decks WHERE deck_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_by_owner(
        &self,
        owner_id: Uuid,
        req: PageRequest,
    ) -> Result<Page<Deck>> {
        let offset = ((req.page.saturating_sub(1)) * req.page_size) as i64;
        let limit = req.page_size as i64;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM decks WHERE owner_id = $1")
            .bind(owner_id)
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query(
            "SELECT deck_id, owner_id, name, mode, slots, status, is_public, share_code, like_count, created_at, updated_at \
             FROM decks WHERE owner_id = $1 ORDER BY updated_at DESC OFFSET $2 LIMIT $3",
        )
        .bind(owner_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            items.push(row_to_deck(r)?);
        }
        Ok(Page {
            items,
            total,
            page: req.page,
            page_size: req.page_size,
        })
    }

    async fn find_by_share_code(&self, share_code: &str) -> Result<Option<Deck>> {
        let row = sqlx::query(
            "SELECT deck_id, owner_id, name, mode, slots, status, is_public, share_code, like_count, created_at, updated_at \
             FROM decks WHERE share_code = $1 AND is_public = TRUE",
        )
        .bind(share_code)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_deck(r)?)),
            None => Ok(None),
        }
    }
}

// ============================================================================
// InMemoryDeckRepository (单测用, 验证 trait 行为)
// ============================================================================

pub struct InMemoryDeckRepository {
    inner: Mutex<HashMap<Uuid, Deck>>,
}

impl InMemoryDeckRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryDeckRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeckRepository for InMemoryDeckRepository {
    async fn create(&self, entity: &Deck) -> Result<Deck> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Deck>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }

    async fn update(&self, entity: &Deck) -> Result<Deck> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }

    async fn list_by_owner(
        &self,
        owner_id: Uuid,
        req: PageRequest,
    ) -> Result<Page<Deck>> {
        let guard = self.inner.lock().unwrap();
        let mut all: Vec<Deck> = guard
            .values()
            .filter(|d| d.owner_id == owner_id)
            .cloned()
            .collect();
        all.sort_by_key(|d| std::cmp::Reverse(d.updated_at));
        let total = all.len() as i64;
        let offset = ((req.page.saturating_sub(1)) * req.page_size) as usize;
        let items = all
            .into_iter()
            .skip(offset)
            .take(req.page_size as usize)
            .collect();
        Ok(Page {
            items,
            total,
            page: req.page,
            page_size: req.page_size,
        })
    }

    async fn find_by_share_code(&self, share_code: &str) -> Result<Option<Deck>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|d| d.is_public && d.share_code.as_deref() == Some(share_code))
            .cloned())
    }
}

// ============================================================================
// helpers
// ============================================================================

fn status_to_str(s: PlayerStatus) -> &'static str {
    match s {
        PlayerStatus::Active => "active",
        PlayerStatus::Banned => "banned",
        PlayerStatus::Disabled => "disabled",
        PlayerStatus::Pending => "pending",
    }
}

fn parse_status(s: &str) -> PlayerStatus {
    match s {
        "banned" => PlayerStatus::Banned,
        "disabled" => PlayerStatus::Disabled,
        "pending" => PlayerStatus::Pending,
        _ => PlayerStatus::Active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_player_save_and_find() {
        let repo = InMemoryPlayerRepository::new();
        let p = Player::new("bob".to_string());
        let id = p.id;
        repo.save(&p).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.name, "bob");
        assert_eq!(found.level, 1);
    }

    #[tokio::test]
    async fn in_memory_player_find_by_name() {
        let repo = InMemoryPlayerRepository::new();
        let p = Player::new("carol".to_string());
        repo.save(&p).await.unwrap();
        let found = repo.find_by_name("carol").await.unwrap().unwrap();
        assert_eq!(found.id, p.id);
    }

    #[tokio::test]
    async fn in_memory_player_delete() {
        let repo = InMemoryPlayerRepository::new();
        let p = Player::new("dave".to_string());
        let id = p.id;
        repo.save(&p).await.unwrap();
        assert!(repo.delete_by_id(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_player_list_paginated() {
        let repo = InMemoryPlayerRepository::new();
        for i in 0..5 {
            repo.save(&Player::new(format!("p{}", i))).await.unwrap();
        }
        let page = repo
            .list_paginated(PageRequest {
                page: 1,
                page_size: 3,
            })
            .await
            .unwrap();
        assert_eq!(page.total, 5);
        assert_eq!(page.items.len(), 3);
    }

    #[tokio::test]
    async fn in_memory_session_delete_expired() {
        let repo = InMemoryPlayerSessionRepository::new();
        let player_id = Uuid::new_v4();
        let mut s = PlayerSession::new(player_id, "d".to_string(), "1.1.1.1".to_string());
        s.expires_at = Utc::now() - chrono::Duration::hours(1);
        repo.save(&s).await.unwrap();
        let removed = repo.delete_expired(Utc::now()).await.unwrap();
        assert_eq!(removed, 1);
    }

    // ----- v2 Deck repository UT (per DTL-038 §4.3, 桶 11 增量) -----

    #[tokio::test]
    async fn in_memory_deck_create_and_find() {
        let repo = InMemoryDeckRepository::new();
        let owner = Uuid::new_v4();
        let d = Deck::new(owner, "aggressive".to_string(), 1);
        let id = d.id;
        repo.create(&d).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found.id, d.id);
        assert_eq!(found.owner_id, owner);
        assert_eq!(found.name, "aggressive");
        assert_eq!(found.mode, 1);
        assert_eq!(found.status, DeckStatus::Draft);
        assert!(!found.is_public);
        assert!(found.share_code.is_none());
    }

    #[tokio::test]
    async fn in_memory_deck_update_slots() {
        let repo = InMemoryDeckRepository::new();
        let owner = Uuid::new_v4();
        let mut d = Deck::new(owner, "control".to_string(), 2);
        d.slots.push(DeckSlot::new("card-1".to_string(), 2));
        d.slots.push(DeckSlot::new("card-2".to_string(), 3));
        repo.create(&d).await.unwrap();

        // 全量替换 slots
        d.slots.clear();
        d.slots.push(DeckSlot::new("card-9".to_string(), 1));
        d.updated_at = Utc::now();
        repo.update(&d).await.unwrap();

        let found = repo.find_by_id(d.id).await.unwrap().unwrap();
        assert_eq!(found.slots.len(), 1);
        assert_eq!(found.slots[0].card_id, "card-9");
        assert_eq!(found.slots[0].count, 1);
    }

    #[tokio::test]
    async fn in_memory_deck_delete_by_id() {
        let repo = InMemoryDeckRepository::new();
        let d = Deck::new(Uuid::new_v4(), "deck".to_string(), 1);
        let id = d.id;
        repo.create(&d).await.unwrap();
        assert!(repo.delete_by_id(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
        // 二次删返回 false
        assert!(!repo.delete_by_id(id).await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_deck_list_by_owner_paginated() {
        let repo = InMemoryDeckRepository::new();
        let owner_a = Uuid::new_v4();
        let owner_b = Uuid::new_v4();
        for i in 0..5 {
            repo.create(&Deck::new(owner_a, format!("a-{}", i), 1))
                .await
                .unwrap();
        }
        for i in 0..3 {
            repo.create(&Deck::new(owner_b, format!("b-{}", i), 2))
                .await
                .unwrap();
        }
        let page = repo
            .list_by_owner(
                owner_a,
                PageRequest {
                    page: 1,
                    page_size: 3,
                },
            )
            .await
            .unwrap();
        assert_eq!(page.total, 5);
        assert_eq!(page.items.len(), 3);
        // owner_b 单独查
        let page_b = repo
            .list_by_owner(
                owner_b,
                PageRequest {
                    page: 1,
                    page_size: 10,
                },
            )
            .await
            .unwrap();
        assert_eq!(page_b.total, 3);
    }

    #[tokio::test]
    async fn in_memory_deck_find_by_share_code_only_public() {
        let repo = InMemoryDeckRepository::new();
        let owner = Uuid::new_v4();
        // 私有 deck 不应被 share_code 查
        let mut private = Deck::new(owner, "private".to_string(), 1);
        private.share_code = Some("secret-code".to_string());
        private.is_public = false;
        repo.create(&private).await.unwrap();
        assert!(repo.find_by_share_code("secret-code").await.unwrap().is_none());

        // 公开 deck 可查
        let mut public = Deck::new(owner, "public".to_string(), 1);
        public.share_code = Some("public-code".to_string());
        public.is_public = true;
        repo.create(&public).await.unwrap();
        let found = repo.find_by_share_code("public-code").await.unwrap().unwrap();
        assert_eq!(found.id, public.id);
        assert!(found.is_public);
    }

    #[tokio::test]
    async fn in_memory_deck_update_share_state() {
        // 验证 update 可变更 is_public + share_code
        let repo = InMemoryDeckRepository::new();
        let mut d = Deck::new(Uuid::new_v4(), "deck".to_string(), 1);
        repo.create(&d).await.unwrap();

        // 开启分享
        d.is_public = true;
        d.share_code = Some("share-1".to_string());
        d.updated_at = Utc::now();
        repo.update(&d).await.unwrap();
        let found = repo.find_by_id(d.id).await.unwrap().unwrap();
        assert!(found.is_public);
        assert_eq!(found.share_code.as_deref(), Some("share-1"));

        // 取消分享
        d.is_public = false;
        d.share_code = None;
        d.updated_at = Utc::now();
        repo.update(&d).await.unwrap();
        let found = repo.find_by_id(d.id).await.unwrap().unwrap();
        assert!(!found.is_public);
        assert!(found.share_code.is_none());
    }

    // ====== v3 增量 (RGS UT 桶 11 / 玩家域, per UT-AGENT-BRIEFING §2 Step 2) ======

    // ----- InMemoryPlayerRepository 边界 -----

    #[tokio::test]
    async fn in_memory_player_delete_nonexistent_returns_false() {
        let repo = InMemoryPlayerRepository::new();
        let id = Uuid::new_v4();
        assert!(!repo.delete_by_id(id).await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_player_find_by_id_nonexistent_returns_none() {
        let repo = InMemoryPlayerRepository::new();
        let id = Uuid::new_v4();
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_player_find_by_name_nonexistent_returns_none() {
        let repo = InMemoryPlayerRepository::new();
        assert!(repo.find_by_name("ghost").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_player_list_paginated_empty_repo() {
        let repo = InMemoryPlayerRepository::new();
        let page = repo
            .list_paginated(PageRequest::default())
            .await
            .unwrap();
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
        assert_eq!(page.page, 1);
        assert_eq!(page.page_size, 20);
    }

    #[tokio::test]
    async fn in_memory_player_list_paginated_offset() {
        let repo = InMemoryPlayerRepository::new();
        for i in 0..6 {
            repo.save(&Player::new(format!("p-{:02}", i))).await.unwrap();
        }
        // page 1 size 2 → 2 项
        let p1 = repo.list_paginated(PageRequest { page: 1, page_size: 2 }).await.unwrap();
        assert_eq!(p1.total, 6);
        assert_eq!(p1.items.len(), 2);
        // page 2 size 2 → 2 项 (offset 2)
        let p2 = repo.list_paginated(PageRequest { page: 2, page_size: 2 }).await.unwrap();
        assert_eq!(p2.total, 6);
        assert_eq!(p2.items.len(), 2);
        // page 4 size 2 → 0 项 (offset 6, 越界)
        let p4 = repo.list_paginated(PageRequest { page: 4, page_size: 2 }).await.unwrap();
        assert_eq!(p4.total, 6);
        assert!(p4.items.is_empty());
    }

    #[tokio::test]
    async fn in_memory_player_save_overwrites_existing() {
        let repo = InMemoryPlayerRepository::new();
        let mut p = Player::new("alice".to_string());
        repo.save(&p).await.unwrap();
        // 同一 id 再 save, 应覆盖
        p.level = 99;
        p.vip_level = 5;
        repo.save(&p).await.unwrap();
        let found = repo.find_by_id(p.id).await.unwrap().unwrap();
        assert_eq!(found.level, 99);
        assert_eq!(found.vip_level, 5);
        // repo 仍只 1 项
        let all = repo.list_paginated(PageRequest { page: 1, page_size: 100 }).await.unwrap();
        assert_eq!(all.total, 1);
    }

    // ----- InMemoryPlayerSessionRepository 边界 -----

    #[tokio::test]
    async fn in_memory_session_find_by_id_nonexistent_returns_none() {
        let repo = InMemoryPlayerSessionRepository::new();
        assert!(repo.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_session_delete_nonexistent_returns_false() {
        let repo = InMemoryPlayerSessionRepository::new();
        assert!(!repo.delete_by_id(Uuid::new_v4()).await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_session_list_by_player_filters_other_players() {
        let repo = InMemoryPlayerSessionRepository::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let s1 = PlayerSession::new(p1, "d1".to_string(), "1.1.1.1".to_string());
        let s2 = PlayerSession::new(p2, "d2".to_string(), "2.2.2.2".to_string());
        let s3 = PlayerSession::new(p1, "d3".to_string(), "1.1.1.2".to_string());
        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();
        repo.save(&s3).await.unwrap();
        let p1_sessions = repo.list_by_player(p1).await.unwrap();
        assert_eq!(p1_sessions.len(), 2);
        let p2_sessions = repo.list_by_player(p2).await.unwrap();
        assert_eq!(p2_sessions.len(), 1);
    }

    #[tokio::test]
    async fn in_memory_session_delete_expired_keeps_valid_ones() {
        let repo = InMemoryPlayerSessionRepository::new();
        let pid = Uuid::new_v4();
        // 1 个过期, 1 个未过期
        let mut expired = PlayerSession::new(pid, "d-old".to_string(), "1.1.1.1".to_string());
        expired.expires_at = Utc::now() - chrono::Duration::hours(1);
        let valid = PlayerSession::new(pid, "d-new".to_string(), "1.1.1.2".to_string());
        // valid.expires_at 默认 now + 24h, 当前未过期
        repo.save(&expired).await.unwrap();
        repo.save(&valid).await.unwrap();
        let removed = repo.delete_expired(Utc::now()).await.unwrap();
        assert_eq!(removed, 1);
        // valid 仍存
        assert!(repo.find_by_id(valid.id).await.unwrap().is_some());
        // expired 已删
        assert!(repo.find_by_id(expired.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_session_save_overwrites_existing() {
        let repo = InMemoryPlayerSessionRepository::new();
        let pid = Uuid::new_v4();
        let mut s = PlayerSession::new(pid, "dev".to_string(), "1.1.1.1".to_string());
        repo.save(&s).await.unwrap();
        s.heartbeat();
        repo.save(&s).await.unwrap();
        let back = repo.find_by_id(s.id).await.unwrap().unwrap();
        // 第二次 save 后 last_heartbeat_at 已被刷新 (≥ 首次)
        assert!(back.last_heartbeat_at >= s.login_at);
    }

    // ----- InMemoryDeckRepository 边界 -----

    #[tokio::test]
    async fn in_memory_deck_find_by_id_nonexistent_returns_none() {
        let repo = InMemoryDeckRepository::new();
        assert!(repo.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_deck_delete_nonexistent_returns_false() {
        let repo = InMemoryDeckRepository::new();
        assert!(!repo.delete_by_id(Uuid::new_v4()).await.unwrap());
    }

    #[tokio::test]
    async fn in_memory_deck_list_by_owner_empty() {
        let repo = InMemoryDeckRepository::new();
        let page = repo
            .list_by_owner(Uuid::new_v4(), PageRequest::default())
            .await
            .unwrap();
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());
    }

    #[tokio::test]
    async fn in_memory_deck_list_by_owner_offset() {
        let repo = InMemoryDeckRepository::new();
        let owner = Uuid::new_v4();
        for i in 0..6 {
            repo.create(&Deck::new(owner, format!("d-{:02}", i), 1)).await.unwrap();
        }
        // page 1 size 2 → 2 项
        let p1 = repo.list_by_owner(owner, PageRequest { page: 1, page_size: 2 }).await.unwrap();
        assert_eq!(p1.total, 6);
        assert_eq!(p1.items.len(), 2);
        // page 3 size 2 → 2 项 (offset 4)
        let p3 = repo.list_by_owner(owner, PageRequest { page: 3, page_size: 2 }).await.unwrap();
        assert_eq!(p3.total, 6);
        assert_eq!(p3.items.len(), 2);
        // page 4 size 2 → 0 项 (offset 6, 越界)
        let p4 = repo.list_by_owner(owner, PageRequest { page: 4, page_size: 2 }).await.unwrap();
        assert_eq!(p4.total, 6);
        assert!(p4.items.is_empty());
    }

    #[tokio::test]
    async fn in_memory_deck_create_overwrites_existing() {
        let repo = InMemoryDeckRepository::new();
        let mut d = Deck::new(Uuid::new_v4(), "deck".to_string(), 1);
        repo.create(&d).await.unwrap();
        d.name = "renamed".to_string();
        d.is_public = true;
        d.share_code = Some("share-x".to_string());
        repo.create(&d).await.unwrap();
        let back = repo.find_by_id(d.id).await.unwrap().unwrap();
        assert_eq!(back.name, "renamed");
        assert!(back.is_public);
        assert_eq!(back.share_code.as_deref(), Some("share-x"));
    }

    #[tokio::test]
    async fn in_memory_deck_find_by_share_code_skips_unpublic_even_with_code() {
        let repo = InMemoryDeckRepository::new();
        let mut d = Deck::new(Uuid::new_v4(), "private".to_string(), 1);
        d.is_public = false; // 显式私有
        d.share_code = Some("leaked".to_string());
        repo.create(&d).await.unwrap();
        // 私有 deck 不应被 find_by_share_code 命中
        assert!(repo.find_by_share_code("leaked").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_deck_find_by_share_code_empty_string() {
        let repo = InMemoryDeckRepository::new();
        // 空 share_code 不命中任何记录
        assert!(repo.find_by_share_code("").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_deck_list_by_owner_returns_only_target_owner() {
        let repo = InMemoryDeckRepository::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        for i in 0..3 {
            repo.create(&Deck::new(a, format!("a-{}", i), 1)).await.unwrap();
        }
        for i in 0..5 {
            repo.create(&Deck::new(b, format!("b-{}", i), 2)).await.unwrap();
        }
        // owner_a 应只看到 3 个
        let pa = repo.list_by_owner(a, PageRequest { page: 1, page_size: 100 }).await.unwrap();
        assert_eq!(pa.total, 3);
        for d in &pa.items {
            assert_eq!(d.owner_id, a);
        }
        // owner_b 应只看到 5 个
        let pb = repo.list_by_owner(b, PageRequest { page: 1, page_size: 100 }).await.unwrap();
        assert_eq!(pb.total, 5);
        for d in &pb.items {
            assert_eq!(d.owner_id, b);
        }
    }
}
