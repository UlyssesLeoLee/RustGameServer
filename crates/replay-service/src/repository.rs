//! replay-service 域 Repository
//!
//! 桶 13 (replay) 实化 (per RGS-DTL-038 §3 DEC-038-03 + §7.1 #7):
//! - ReplayRepository (replays 表, 元数据)
//! - 双实现: PgRepository (sqlx, 生产) + InMemoryRepository (单测)
//!
//! 设计原则:
//! - 元数据访问走此模块, 数据访问走 `storage::StorageBackend`
//! - 元数据与对象存储分离 (per DEC-038-03, 防存储双写复杂)
//! - 跨域引用: match_id / player_a / player_b 不物化 FK (per ARC-008)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{ReplayFilter, ReplayMeta, ReplayMode};
use crate::Result;

// ============================================================================
// 分页 (per common.proto PageRequest)
// ============================================================================

/// 分页请求 (per common.proto PageRequest)
#[derive(Debug, Clone, Copy)]
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

/// 分页响应 (per common.proto PageResponse)
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

// ============================================================================
// Trait 定义
// ============================================================================

/// Replay Repository (元数据访问)
#[async_trait]
pub trait ReplayRepository: Send + Sync {
    /// 按 replay_id 查询
    async fn find_by_id(&self, replay_id: Uuid) -> Result<Option<ReplayMeta>>;

    /// 插入新回放元数据
    async fn insert(&self, meta: &ReplayMeta) -> Result<ReplayMeta>;

    /// 删除元数据 (物理删除, 用于过期清理; 实际删数据在 storage 层)
    async fn delete(&self, replay_id: Uuid) -> Result<bool>;

    /// 按过滤 + 分页列出
    async fn list(
        &self,
        filter: &ReplayFilter,
        page_req: PageRequest,
    ) -> Result<Page<ReplayMeta>>;

    /// 清理已过期元数据 (返回删除数量, 用于 cron job)
    /// 业务约束: 仅删除 expires_at < now() 的元数据
    async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64>;
}

// ============================================================================
// PgRepository (sqlx 实现, 生产用)
// ============================================================================

pub struct PgReplayRepository {
    pool: PgPool,
}

impl PgReplayRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_meta(row: sqlx::postgres::PgRow) -> Result<ReplayMeta> {
    let mode_int: i16 = row.get("mode");
    let player_b: Option<String> = row.get("player_b");
    let object_size: Option<i64> = row.get("object_size");
    let duration_secs: Option<i32> = row.get("duration_secs");
    Ok(ReplayMeta {
        replay_id: row.get("replay_id"),
        match_id: row.get("match_id"),
        player_a: row.get("player_a"),
        player_b,
        mode: ReplayMode::from_i32(i32::from(mode_int)),
        object_key: row.get("object_key"),
        object_size: object_size.unwrap_or(0),
        duration_secs: duration_secs.map(|v| v as u32).unwrap_or(0),
        created_at: row.get("created_at"),
        expires_at: row.get("expires_at"),
    })
}

#[async_trait]
impl ReplayRepository for PgReplayRepository {
    async fn find_by_id(&self, replay_id: Uuid) -> Result<Option<ReplayMeta>> {
        let row = sqlx::query(
            "SELECT replay_id, match_id, player_a, player_b, mode, object_key, object_size, \
                    duration_secs, created_at, expires_at \
             FROM replays WHERE replay_id = $1",
        )
        .bind(replay_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_meta(r)?)),
            None => Ok(None),
        }
    }

    async fn insert(&self, meta: &ReplayMeta) -> Result<ReplayMeta> {
        let mode_num: i16 = meta.mode.as_i32() as i16;
        sqlx::query(
            "INSERT INTO replays \
                (replay_id, match_id, player_a, player_b, mode, object_key, object_size, \
                 duration_secs, created_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(meta.replay_id)
        .bind(meta.match_id)
        .bind(&meta.player_a)
        .bind(&meta.player_b)
        .bind(mode_num)
        .bind(&meta.object_key)
        .bind(meta.object_size)
        .bind(meta.duration_secs as i32)
        .bind(meta.created_at)
        .bind(meta.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(meta.clone())
    }

    async fn delete(&self, replay_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM replays WHERE replay_id = $1")
            .bind(replay_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        filter: &ReplayFilter,
        page_req: PageRequest,
    ) -> Result<Page<ReplayMeta>> {
        let offset = ((page_req.page.saturating_sub(1)) * page_req.page_size) as i64;
        let limit = page_req.page_size as i64;

        // 动态 WHERE 构造 (filter 4 选 N)
        let mut where_clauses: Vec<String> = Vec::new();
        if let Some(ref p) = filter.player_a_filter {
            where_clauses.push(format!("player_a = '{}'", p.replace('\'', "''")));
        }
        if let Some(ref p) = filter.player_b_filter {
            where_clauses.push(format!("player_b = '{}'", p.replace('\'', "''")));
        }
        if let Some(m) = filter.mode_filter {
            where_clauses.push(format!("mode = {}", m.as_i32()));
        }
        if !filter.include_expired {
            where_clauses.push("expires_at > now()".to_string());
        }
        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM replays {}", where_sql);
        let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(&self.pool).await?;

        let list_sql = format!(
            "SELECT replay_id, match_id, player_a, player_b, mode, object_key, object_size, \
                    duration_secs, created_at, expires_at \
             FROM replays {} ORDER BY created_at DESC OFFSET {} LIMIT {}",
            where_sql, offset, limit
        );
        let rows = sqlx::query(&list_sql).fetch_all(&self.pool).await?;
        let items: Vec<ReplayMeta> = rows
            .into_iter()
            .map(row_to_meta)
            .collect::<Result<Vec<_>>>()?;

        let has_next = (offset + items.len() as i64) < total;
        Ok(Page {
            items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query("DELETE FROM replays WHERE expires_at < $1")
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

// ============================================================================
// InMemoryRepository (测用)
// ============================================================================

pub struct InMemoryReplayRepository {
    inner: Mutex<HashMap<Uuid, ReplayMeta>>,
}

impl InMemoryReplayRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryReplayRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReplayRepository for InMemoryReplayRepository {
    async fn find_by_id(&self, replay_id: Uuid) -> Result<Option<ReplayMeta>> {
        Ok(self.inner.lock().unwrap().get(&replay_id).cloned())
    }

    async fn insert(&self, meta: &ReplayMeta) -> Result<ReplayMeta> {
        self.inner
            .lock()
            .unwrap()
            .insert(meta.replay_id, meta.clone());
        Ok(meta.clone())
    }

    async fn delete(&self, replay_id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&replay_id).is_some())
    }

    async fn list(
        &self,
        filter: &ReplayFilter,
        page_req: PageRequest,
    ) -> Result<Page<ReplayMeta>> {
        let guard = self.inner.lock().unwrap();
        let mut items: Vec<ReplayMeta> = guard
            .values()
            .filter(|m| {
                let mut ok = true;
                if let Some(ref p) = filter.player_a_filter {
                    ok = ok && &m.player_a == p;
                }
                if let Some(ref p) = filter.player_b_filter {
                    ok = ok && m.player_b.as_ref() == Some(p);
                }
                if let Some(mode) = filter.mode_filter {
                    ok = ok && m.mode == mode;
                }
                if !filter.include_expired {
                    ok = ok && m.expires_at > Utc::now();
                }
                ok
            })
            .cloned()
            .collect();
        // 按 created_at DESC 排序
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = items.len() as i64;
        let start = offset_for(page_req.page, page_req.page_size);
        let end = std::cmp::min(start + page_req.page_size as usize, items.len());
        let page_items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };
        let has_next = (start + page_items.len()) < items.len();
        Ok(Page {
            items: page_items,
            total,
            page: page_req.page,
            page_size: page_req.page_size,
            has_next,
        })
    }

    async fn delete_expired(&self, now: DateTime<Utc>) -> Result<u64> {
        let mut guard = self.inner.lock().unwrap();
        let before = guard.len();
        guard.retain(|_, m| m.expires_at >= now);
        Ok((before - guard.len()) as u64)
    }
}

fn offset_for(page: u32, page_size: u32) -> usize {
    ((page.saturating_sub(1)) * page_size) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_meta(player_a: &str, mode: ReplayMode) -> ReplayMeta {
        ReplayMeta::new(
            Uuid::new_v4(),
            player_a.to_string(),
            None,
            mode,
            format!("replays/rp-{}.dat", Uuid::new_v4()),
        )
    }

    #[tokio::test]
    async fn in_memory_insert_and_find() {
        let repo = InMemoryReplayRepository::new();
        let m = sample_meta("p-a", ReplayMode::Ranked);
        let id = m.replay_id;
        repo.insert(&m).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().player_a, "p-a");
    }

    #[tokio::test]
    async fn in_memory_delete() {
        let repo = InMemoryReplayRepository::new();
        let m = sample_meta("p-a", ReplayMode::Casual);
        let id = m.replay_id;
        repo.insert(&m).await.unwrap();
        assert!(repo.delete(id).await.unwrap());
        assert!(!repo.delete(id).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn in_memory_list_filter_by_player_a() {
        let repo = InMemoryReplayRepository::new();
        repo.insert(&sample_meta("p-a", ReplayMode::Ranked)).await.unwrap();
        repo.insert(&sample_meta("p-a", ReplayMode::Casual)).await.unwrap();
        repo.insert(&sample_meta("p-b", ReplayMode::Ranked)).await.unwrap();
        let filter = ReplayFilter {
            player_a_filter: Some("p-a".to_string()),
            ..Default::default()
        };
        let page = repo
            .list(&filter, PageRequest { page: 1, page_size: 20 })
            .await
            .unwrap();
        assert_eq!(page.total, 2);
        assert_eq!(page.items.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_list_filter_by_mode() {
        let repo = InMemoryReplayRepository::new();
        repo.insert(&sample_meta("p-a", ReplayMode::Ranked)).await.unwrap();
        repo.insert(&sample_meta("p-b", ReplayMode::Casual)).await.unwrap();
        let filter = ReplayFilter {
            mode_filter: Some(ReplayMode::Ranked),
            ..Default::default()
        };
        let page = repo
            .list(&filter, PageRequest { page: 1, page_size: 20 })
            .await
            .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].mode, ReplayMode::Ranked);
    }

    #[tokio::test]
    async fn in_memory_list_pagination() {
        let repo = InMemoryReplayRepository::new();
        for _ in 0..25 {
            repo.insert(&sample_meta("p", ReplayMode::Casual)).await.unwrap();
        }
        let page1 = repo
            .list(
                &ReplayFilter::default(),
                PageRequest { page: 1, page_size: 10 },
            )
            .await
            .unwrap();
        assert_eq!(page1.total, 25);
        assert_eq!(page1.items.len(), 10);
        assert!(page1.has_next);
        let page3 = repo
            .list(
                &ReplayFilter::default(),
                PageRequest { page: 3, page_size: 10 },
            )
            .await
            .unwrap();
        assert_eq!(page3.items.len(), 5);
        assert!(!page3.has_next);
    }

    #[tokio::test]
    async fn in_memory_delete_expired() {
        let repo = InMemoryReplayRepository::new();
        let mut m1 = sample_meta("p-a", ReplayMode::Casual);
        m1.expires_at = Utc::now() - chrono::Duration::seconds(1); // 已过期
        let m2 = sample_meta("p-b", ReplayMode::Casual);
        repo.insert(&m1).await.unwrap();
        repo.insert(&m2).await.unwrap();
        let removed = repo.delete_expired(Utc::now()).await.unwrap();
        assert_eq!(removed, 1);
        assert!(repo.find_by_id(m1.replay_id).await.unwrap().is_none());
        assert!(repo.find_by_id(m2.replay_id).await.unwrap().is_some());
    }
}
