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

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{Player, PlayerSession, PlayerStatus};
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
}
