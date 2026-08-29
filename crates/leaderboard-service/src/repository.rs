//! leaderboard-service 域 Repository
//!
//! trait + PgRepository (sqlx impl) + InMemoryRepository (测用)
//! 规范: RGS-DTL-038 §3 榜单域数据访问层
//!
//! 设计要点:
//! - rank 不在 leaderboard_entries 表中作为权威列 (避免 UPDATE 链风暴)
//! - rank 通过 (type, period, season_id) partition + score DESC 排序实时计算
//! - 高频读走 idx_lb_type_period_season_score 索引 (per migration 0001_init.sql)

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{LeaderboardEntry, LeaderboardPeriod, LeaderboardType};
use crate::Result;

/// 榜单域 Repository trait
#[async_trait]
pub trait LeaderboardRepository: Send + Sync {
    /// 按 (type, period, season_id) + score DESC + limit/offset 拉取榜单
    /// 返回: (entries, total_count)
    async fn list_by_board(
        &self,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LeaderboardEntry>, i64)>;

    /// 按 player_id 查该玩家在 (type, period) 下的条目
    async fn find_by_player(
        &self,
        player_id: Uuid,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
    ) -> Result<Option<LeaderboardEntry>>;

    /// 按 (type, period, season_id) partition 内 player 的 score DESC 排名 (1-based)
    /// 返回 None 表示该玩家未入榜
    async fn rank_of(
        &self,
        player_id: Uuid,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
    ) -> Result<Option<u32>>;

    /// Upsert: 同 (type, period, season_id, player_id) 唯一, 存在则更新 score/wins/losses/display_name/updated_at
    /// 返回: (更新后条目, rank 是否发生变化)
    async fn upsert(&self, entity: &LeaderboardEntry) -> Result<(LeaderboardEntry, bool)>;

    /// 删除 (内部清理用, 默认不暴露给 client)
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgLeaderboardRepository {
    pool: PgPool,
}

impl PgLeaderboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_entry(row: sqlx::postgres::PgRow) -> LeaderboardEntry {
    let type_str: String = row.get("leaderboard_type");
    let period_str: String = row.get("period");
    LeaderboardEntry {
        id: row.get("id"),
        leaderboard_type: LeaderboardType::from_str(&type_str).unwrap_or(LeaderboardType::Casual),
        period: LeaderboardPeriod::from_str(&period_str).unwrap_or(LeaderboardPeriod::AllTime),
        season_id: row.get("season_id"),
        player_id: row.get("player_id"),
        display_name: row.get("display_name"),
        score: row.get("score"),
        wins: row.get::<i32, _>("wins") as u32,
        losses: row.get::<i32, _>("losses") as u32,
        rank: row.get::<i32, _>("rank") as u32,
        updated_at: row.get("updated_at"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl LeaderboardRepository for PgLeaderboardRepository {
    async fn list_by_board(
        &self,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LeaderboardEntry>, i64)> {
        // total
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM leaderboard_entries \
             WHERE leaderboard_type = $1 AND period = $2 AND season_id = $3",
        )
        .bind(leaderboard_type.as_str())
        .bind(period.as_str())
        .bind(season_id)
        .fetch_one(&self.pool)
        .await?;

        // 走 idx_lb_type_period_season_score 索引, score DESC + LIMIT/OFFSET 分页
        let rows = sqlx::query(
            "SELECT id, leaderboard_type, period, season_id, player_id, display_name, \
                    score, wins, losses, 0 AS rank, updated_at, created_at \
             FROM leaderboard_entries \
             WHERE leaderboard_type = $1 AND period = $2 AND season_id = $3 \
             ORDER BY score DESC, updated_at ASC \
             LIMIT $4 OFFSET $5",
        )
        .bind(leaderboard_type.as_str())
        .bind(period.as_str())
        .bind(season_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut entries: Vec<LeaderboardEntry> = rows.into_iter().map(row_to_entry).collect();
        // rank 1-based, 实时计算
        for (i, e) in entries.iter_mut().enumerate() {
            e.rank = (offset as u32) + (i as u32) + 1;
        }
        Ok((entries, total))
    }

    async fn find_by_player(
        &self,
        player_id: Uuid,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
    ) -> Result<Option<LeaderboardEntry>> {
        let row = sqlx::query(
            "SELECT id, leaderboard_type, period, season_id, player_id, display_name, \
                    score, wins, losses, 0 AS rank, updated_at, created_at \
             FROM leaderboard_entries \
             WHERE player_id = $1 AND leaderboard_type = $2 AND period = $3 AND season_id = $4",
        )
        .bind(player_id)
        .bind(leaderboard_type.as_str())
        .bind(period.as_str())
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_entry))
    }

    async fn rank_of(
        &self,
        player_id: Uuid,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
    ) -> Result<Option<u32>> {
        // 通过 subquery 计算 "比我分高的人数 + 1" 即为我的 rank
        // (同分时 updated_at 早者优先, 与 list_by_board 排序一致)
        let rank: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) + 1 FROM leaderboard_entries t1 \
             WHERE t1.leaderboard_type = $1 AND t1.period = $2 AND t1.season_id = $3 \
               AND (t1.score > (SELECT score FROM leaderboard_entries t2 \
                                WHERE t2.player_id = $4 AND t2.leaderboard_type = $1 \
                                  AND t2.period = $2 AND t2.season_id = $3) \
                    OR (t1.score = (SELECT score FROM leaderboard_entries t2 \
                                    WHERE t2.player_id = $4 AND t2.leaderboard_type = $1 \
                                      AND t2.period = $2 AND t2.season_id = $3) \
                        AND t1.updated_at < (SELECT updated_at FROM leaderboard_entries t2 \
                                             WHERE t2.player_id = $4 AND t2.leaderboard_type = $1 \
                                               AND t2.period = $2 AND t2.season_id = $3)))",
        )
        .bind(leaderboard_type.as_str())
        .bind(period.as_str())
        .bind(season_id)
        .bind(player_id)
        .fetch_optional(&self.pool)
        .await?;

        // 先查玩家是否存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM leaderboard_entries \
             WHERE player_id = $1 AND leaderboard_type = $2 AND period = $3 AND season_id = $4)",
        )
        .bind(player_id)
        .bind(leaderboard_type.as_str())
        .bind(period.as_str())
        .bind(season_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            return Ok(None);
        }
        Ok(rank.map(|r| r as u32))
    }

    async fn upsert(&self, entity: &LeaderboardEntry) -> Result<(LeaderboardEntry, bool)> {
        // 计算 upsert 前的 rank (用于判断 rank_changed)
        let old_rank = self
            .rank_of(entity.player_id, entity.leaderboard_type, entity.period, &entity.season_id)
            .await?;

        sqlx::query(
            "INSERT INTO leaderboard_entries \
                (id, leaderboard_type, period, season_id, player_id, display_name, \
                 score, wins, losses, rank, updated_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, $10, $11) \
             ON CONFLICT (leaderboard_type, period, season_id, player_id) DO UPDATE SET \
                score = EXCLUDED.score, wins = EXCLUDED.wins, losses = EXCLUDED.losses, \
                display_name = EXCLUDED.display_name, updated_at = EXCLUDED.updated_at",
        )
        .bind(entity.id)
        .bind(entity.leaderboard_type.as_str())
        .bind(entity.period.as_str())
        .bind(&entity.season_id)
        .bind(entity.player_id)
        .bind(&entity.display_name)
        .bind(entity.score)
        .bind(entity.wins as i32)
        .bind(entity.losses as i32)
        .bind(entity.updated_at)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;

        let new_rank = self
            .rank_of(entity.player_id, entity.leaderboard_type, entity.period, &entity.season_id)
            .await?;

        let rank_changed = match (old_rank, new_rank) {
            (Some(o), Some(n)) => o != n,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };

        // 读回最新 row 拿真实 id (upsert 触发 ON CONFLICT 时, id 可能被保留为新生成的)
        let mut updated = entity.clone();
        if let Some(r) = new_rank {
            updated.rank = r;
        }
        Ok((updated, rank_changed))
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM leaderboard_entries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// InMemoryRepository (测用)
// ============================================================================

pub struct InMemoryLeaderboardRepository {
    inner: Mutex<HashMap<Uuid, LeaderboardEntry>>,
}

impl InMemoryLeaderboardRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryLeaderboardRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LeaderboardRepository for InMemoryLeaderboardRepository {
    async fn list_by_board(
        &self,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LeaderboardEntry>, i64)> {
        let guard = self.inner.lock().unwrap();
        let mut matching: Vec<LeaderboardEntry> = guard
            .values()
            .filter(|e| {
                e.leaderboard_type == leaderboard_type
                    && e.period == period
                    && e.season_id == season_id
            })
            .cloned()
            .collect();
        // 按 score DESC, 同分 updated_at 早者优先
        matching.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.updated_at.cmp(&b.updated_at))
        });
        let total = matching.len() as i64;
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, matching.len());
        let mut page: Vec<LeaderboardEntry> = if start < matching.len() {
            matching[start..end].to_vec()
        } else {
            Vec::new()
        };
        for (i, e) in page.iter_mut().enumerate() {
            e.rank = (start as u32) + (i as u32) + 1;
        }
        Ok((page, total))
    }

    async fn find_by_player(
        &self,
        player_id: Uuid,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
    ) -> Result<Option<LeaderboardEntry>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|e| {
                e.player_id == player_id
                    && e.leaderboard_type == leaderboard_type
                    && e.period == period
                    && e.season_id == season_id
            })
            .cloned())
    }

    async fn rank_of(
        &self,
        player_id: Uuid,
        leaderboard_type: LeaderboardType,
        period: LeaderboardPeriod,
        season_id: &str,
    ) -> Result<Option<u32>> {
        let guard = self.inner.lock().unwrap();
        let target = match guard.values().find(|e| {
            e.player_id == player_id
                && e.leaderboard_type == leaderboard_type
                && e.period == period
                && e.season_id == season_id
        }) {
            Some(e) => e,
            None => return Ok(None),
        };
        let higher = guard
            .values()
            .filter(|e| {
                e.leaderboard_type == leaderboard_type
                    && e.period == period
                    && e.season_id == season_id
                    && (e.score > target.score
                        || (e.score == target.score && e.updated_at < target.updated_at))
            })
            .count();
        Ok(Some((higher + 1) as u32))
    }

    async fn upsert(&self, entity: &LeaderboardEntry) -> Result<(LeaderboardEntry, bool)> {
        let old_rank = self
            .rank_of(entity.player_id, entity.leaderboard_type, entity.period, &entity.season_id)
            .await?;
        // 临界区: 移除旧 + 插入新 (原子, 不跨 await)
        {
            let mut guard = self.inner.lock().unwrap();
            // 移除 (type, period, season_id, player_id) 已存在的条目
            guard.retain(|_, e| {
                !(e.leaderboard_type == entity.leaderboard_type
                    && e.period == entity.period
                    && e.season_id == entity.season_id
                    && e.player_id == entity.player_id)
            });
            guard.insert(entity.id, entity.clone());
        }
        let new_rank = self
            .rank_of(entity.player_id, entity.leaderboard_type, entity.period, &entity.season_id)
            .await?;
        let rank_changed = match (old_rank, new_rank) {
            (Some(o), Some(n)) => o != n,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };
        let mut updated = entity.clone();
        if let Some(r) = new_rank {
            updated.rank = r;
        }
        Ok((updated, rank_changed))
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        t: LeaderboardType,
        p: LeaderboardPeriod,
        season: &str,
        score: i64,
    ) -> LeaderboardEntry {
        LeaderboardEntry::new(
            t,
            p,
            season.to_string(),
            Uuid::new_v4(),
            "p".to_string(),
            score,
            0,
            0,
        )
    }

    #[tokio::test]
    async fn in_memory_upsert_and_rank() {
        let repo = InMemoryLeaderboardRepository::new();
        let e1 = entry(LeaderboardType::Ranked, LeaderboardPeriod::AllTime, "", 100);
        let pid1 = e1.player_id;
        repo.upsert(&e1).await.unwrap();

        let rank = repo
            .rank_of(
                pid1,
                LeaderboardType::Ranked,
                LeaderboardPeriod::AllTime,
                "",
            )
            .await
            .unwrap();
        assert_eq!(rank, Some(1));
    }

    #[tokio::test]
    async fn in_memory_list_sorted_by_score_desc() {
        let repo = InMemoryLeaderboardRepository::new();
        let e1 = entry(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 50);
        let e2 = entry(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 200);
        let e3 = entry(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 100);
        repo.upsert(&e1).await.unwrap();
        repo.upsert(&e2).await.unwrap();
        repo.upsert(&e3).await.unwrap();

        let (page, total) = repo
            .list_by_board(
                LeaderboardType::Casual,
                LeaderboardPeriod::Weekly,
                "",
                10,
                0,
            )
            .await
            .unwrap();
        assert_eq!(total, 3);
        assert_eq!(page.len(), 3);
        assert_eq!(page[0].score, 200);
        assert_eq!(page[0].rank, 1);
        assert_eq!(page[1].score, 100);
        assert_eq!(page[1].rank, 2);
        assert_eq!(page[2].score, 50);
        assert_eq!(page[2].rank, 3);
    }

    #[tokio::test]
    async fn in_memory_pagination() {
        let repo = InMemoryLeaderboardRepository::new();
        for s in 0..5 {
            let e = entry(LeaderboardType::Collection, LeaderboardPeriod::AllTime, "", s * 10);
            repo.upsert(&e).await.unwrap();
        }
        let (page1, total) = repo
            .list_by_board(
                LeaderboardType::Collection,
                LeaderboardPeriod::AllTime,
                "",
                2,
                0,
            )
            .await
            .unwrap();
        assert_eq!(total, 5);
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].rank, 1);
        assert_eq!(page1[0].score, 40);
        assert_eq!(page1[1].rank, 2);
        assert_eq!(page1[1].score, 30);

        let (page2, _) = repo
            .list_by_board(
                LeaderboardType::Collection,
                LeaderboardPeriod::AllTime,
                "",
                2,
                2,
            )
            .await
            .unwrap();
        assert_eq!(page2[0].rank, 3);
        assert_eq!(page2[0].score, 20);
    }
}
