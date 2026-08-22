//! match-service 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository
//! 规范：RGS-DTL-016 §3 匹配域数据访问层
//!
//! 注意：`Match` 是 Rust 关键字，repository 内部用 `r#Match` raw identifier。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{Match, MatchMode, MatchParticipant, MatchStatus, Team};
use crate::Result;

/// Match Repository trait
#[async_trait]
pub trait MatchRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Match>>;
    async fn find_by_room_id(&self, room_id: &str) -> Result<Option<Match>>;
    async fn save(&self, entity: &Match) -> Result<Match>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 按状态分页查询
    async fn list_by_status(&self, status: MatchStatus, limit: i64) -> Result<Vec<Match>>;
}

/// MatchParticipant Repository trait
#[async_trait]
pub trait MatchParticipantRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MatchParticipant>>;
    async fn list_by_match(&self, match_id: Uuid) -> Result<Vec<MatchParticipant>>;
    async fn save(&self, entity: &MatchParticipant) -> Result<MatchParticipant>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgMatchRepository {
    pool: PgPool,
}

impl PgMatchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_match(row: sqlx::postgres::PgRow) -> Match {
    let mode_str: String = row.get("mode");
    let status_str: String = row.get("status");
    let winner_str: Option<String> = row.get("winner_team");
    Match {
        id: row.get("id"),
        room_id: row.get("room_id"),
        mode: parse_mode(&mode_str),
        status: parse_status(&status_str),
        winner_team: winner_str.as_deref().and_then(parse_team),
        scheduled_at: row.get("scheduled_at"),
        started_at: row.get("started_at"),
        ended_at: row.get("ended_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl MatchRepository for PgMatchRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Match>> {
        let row = sqlx::query(
            "SELECT id, room_id, mode, status, winner_team, scheduled_at, started_at, ended_at, created_at, updated_at \
             FROM matches WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_match))
    }

    async fn find_by_room_id(&self, room_id: &str) -> Result<Option<Match>> {
        let row = sqlx::query(
            "SELECT id, room_id, mode, status, winner_team, scheduled_at, started_at, ended_at, created_at, updated_at \
             FROM matches WHERE room_id = $1",
        )
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_match))
    }

    async fn save(&self, entity: &Match) -> Result<Match> {
        sqlx::query(
            "INSERT INTO matches \
             (id, room_id, mode, status, winner_team, scheduled_at, started_at, ended_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET \
                status = EXCLUDED.status, winner_team = EXCLUDED.winner_team, \
                started_at = EXCLUDED.started_at, ended_at = EXCLUDED.ended_at, \
                updated_at = EXCLUDED.updated_at",
        )
        .bind(entity.id)
        .bind(&entity.room_id)
        .bind(mode_to_str(entity.mode))
        .bind(match_status_to_str(entity.status))
        .bind(entity.winner_team.map(team_to_str))
        .bind(entity.scheduled_at)
        .bind(entity.started_at)
        .bind(entity.ended_at)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM matches WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_by_status(&self, status: MatchStatus, limit: i64) -> Result<Vec<Match>> {
        let rows = sqlx::query(
            "SELECT id, room_id, mode, status, winner_team, scheduled_at, started_at, ended_at, created_at, updated_at \
             FROM matches WHERE status = $1 ORDER BY scheduled_at LIMIT $2",
        )
        .bind(match_status_to_str(status))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_match).collect())
    }
}

pub struct PgMatchParticipantRepository {
    pool: PgPool,
}

impl PgMatchParticipantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_participant(row: sqlx::postgres::PgRow) -> MatchParticipant {
    let team_str: String = row.get("team");
    MatchParticipant {
        id: row.get("id"),
        match_id: row.get("match_id"),
        player_id: row.get("player_id"),
        team: parse_team(&team_str).unwrap_or(Team::None),
        score: row.get("score"),
        kills: row.get("kills"),
        deaths: row.get("deaths"),
        assists: row.get("assists"),
        is_mvp: row.get("is_mvp"),
        joined_at: row.get("joined_at"),
    }
}

#[async_trait]
impl MatchParticipantRepository for PgMatchParticipantRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MatchParticipant>> {
        let row = sqlx::query(
            "SELECT id, match_id, player_id, team, score, kills, deaths, assists, is_mvp, joined_at \
             FROM match_participants WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_participant))
    }

    async fn list_by_match(&self, match_id: Uuid) -> Result<Vec<MatchParticipant>> {
        let rows = sqlx::query(
            "SELECT id, match_id, player_id, team, score, kills, deaths, assists, is_mvp, joined_at \
             FROM match_participants WHERE match_id = $1 ORDER BY joined_at",
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_participant).collect())
    }

    async fn save(&self, entity: &MatchParticipant) -> Result<MatchParticipant> {
        sqlx::query(
            "INSERT INTO match_participants \
             (id, match_id, player_id, team, score, kills, deaths, assists, is_mvp, joined_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET \
                score = EXCLUDED.score, kills = EXCLUDED.kills, \
                deaths = EXCLUDED.deaths, assists = EXCLUDED.assists, \
                is_mvp = EXCLUDED.is_mvp",
        )
        .bind(entity.id)
        .bind(entity.match_id)
        .bind(entity.player_id)
        .bind(team_to_str(entity.team))
        .bind(entity.score)
        .bind(entity.kills)
        .bind(entity.deaths)
        .bind(entity.assists)
        .bind(entity.is_mvp)
        .bind(entity.joined_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM match_participants WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryMatchRepository {
    inner: Mutex<HashMap<Uuid, Match>>,
}

impl InMemoryMatchRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryMatchRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MatchRepository for InMemoryMatchRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Match>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_room_id(&self, room_id: &str) -> Result<Option<Match>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|m| m.room_id == room_id)
            .cloned())
    }
    async fn save(&self, entity: &Match) -> Result<Match> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
    async fn list_by_status(&self, status: MatchStatus, limit: i64) -> Result<Vec<Match>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.status == status)
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

pub struct InMemoryMatchParticipantRepository {
    inner: Mutex<HashMap<Uuid, MatchParticipant>>,
}

impl InMemoryMatchParticipantRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryMatchParticipantRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MatchParticipantRepository for InMemoryMatchParticipantRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MatchParticipant>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_match(&self, match_id: Uuid) -> Result<Vec<MatchParticipant>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|p| p.match_id == match_id)
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &MatchParticipant) -> Result<MatchParticipant> {
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

fn mode_to_str(m: MatchMode) -> &'static str {
    match m {
        MatchMode::OneVsOne => "1v1",
        MatchMode::TwoVsTwo => "2v2",
        MatchMode::FiveVsFive => "5v5",
        MatchMode::BattleRoyale => "battle_royale",
    }
}

fn parse_mode(s: &str) -> MatchMode {
    match s {
        "1v1" => MatchMode::OneVsOne,
        "2v2" => MatchMode::TwoVsTwo,
        "5v5" => MatchMode::FiveVsFive,
        _ => MatchMode::BattleRoyale,
    }
}

fn match_status_to_str(s: MatchStatus) -> &'static str {
    match s {
        MatchStatus::Waiting => "waiting",
        MatchStatus::InProgress => "in_progress",
        MatchStatus::Finished => "finished",
        MatchStatus::Cancelled => "cancelled",
    }
}

fn parse_status(s: &str) -> MatchStatus {
    match s {
        "waiting" => MatchStatus::Waiting,
        "in_progress" => MatchStatus::InProgress,
        "finished" => MatchStatus::Finished,
        _ => MatchStatus::Cancelled,
    }
}

fn team_to_str(t: Team) -> &'static str {
    match t {
        Team::Blue => "blue",
        Team::Red => "red",
        Team::None => "none",
    }
}

fn parse_team(s: &str) -> Option<Team> {
    match s {
        "blue" => Some(Team::Blue),
        "red" => Some(Team::Red),
        _ => Some(Team::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_match_lifecycle() {
        let repo = InMemoryMatchRepository::new();
        let mut m = Match::new("r1".to_string(), MatchMode::FiveVsFive);
        let id = m.id;
        m.start();
        repo.save(&m).await.unwrap();

        let loaded = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(loaded.status, MatchStatus::InProgress);
    }

    #[tokio::test]
    async fn in_memory_match_list_by_status() {
        let repo = InMemoryMatchRepository::new();
        let mut m1 = Match::new("r1".to_string(), MatchMode::TwoVsTwo);
        let mut m2 = Match::new("r2".to_string(), MatchMode::TwoVsTwo);
        m1.start();
        m2.start();
        m2.finish(Some(Team::Blue));
        repo.save(&m1).await.unwrap();
        repo.save(&m2).await.unwrap();

        let waiting = repo
            .list_by_status(MatchStatus::InProgress, 10)
            .await
            .unwrap();
        assert_eq!(waiting.len(), 1);
    }

    #[tokio::test]
    async fn in_memory_participant_list_by_match() {
        let repo = InMemoryMatchParticipantRepository::new();
        let match_id = Uuid::new_v4();
        for _ in 0..3 {
            repo.save(&MatchParticipant::new(match_id, Uuid::new_v4(), Team::Blue))
                .await
                .unwrap();
        }
        let list = repo.list_by_match(match_id).await.unwrap();
        assert_eq!(list.len(), 3);
    }
}
