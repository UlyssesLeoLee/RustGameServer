//! match-service 域 v2 Repository (per RGS-DTL-038 v0.1 §4.2 + §5 + §7.1)
//!
//! 卡牌游戏 session/turn 抽象的 3 个 Repository trait + Pg + InMemory 实现:
//! - `GameSessionRepository`: game_sessions 表
//! - `MoveRepository`: moves 表
//! - `MatchmakingTicketRepository`: matchmaking_tickets 表 (EnqueueMatchmaking 队列)
//!
//! v1 repository (Match / MatchParticipant) 保留, 不破坏现有 5 域 matchmaker.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity_v2::{
    Board, GameMode, GameSession, MatchmakingTicket, Move, MoveType, SessionPlayer, SessionStatus,
};
use crate::Result;

// ============================================================================
// GameSessionRepository
// ============================================================================

#[async_trait]
pub trait GameSessionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GameSession>>;
    async fn find_by_room_code(&self, room_code: &str) -> Result<Option<GameSession>>;
    async fn save(&self, entity: &GameSession) -> Result<GameSession>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    async fn list_by_status(&self, status: SessionStatus, limit: i64) -> Result<Vec<GameSession>>;
    async fn list_by_player(&self, player_id: &str, limit: i64) -> Result<Vec<GameSession>>;
}

// ============================================================================
// MoveRepository
// ============================================================================

#[async_trait]
pub trait MoveRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Move>>;
    async fn list_by_match(&self, match_id: Uuid) -> Result<Vec<Move>>;
    async fn list_by_match_turn(&self, match_id: Uuid, turn_index: u32) -> Result<Vec<Move>>;
    async fn save(&self, entity: &Move) -> Result<Move>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// MatchmakingTicketRepository
// ============================================================================

#[async_trait]
pub trait MatchmakingTicketRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MatchmakingTicket>>;
    async fn list_by_player(&self, player_id: &str) -> Result<Vec<MatchmakingTicket>>;
    async fn list_by_status(&self, status: i32) -> Result<Vec<MatchmakingTicket>>;
    async fn find_matchable(
        &self,
        mode: GameMode,
        rank_score: u32,
    ) -> Result<Vec<MatchmakingTicket>>;
    async fn save(&self, entity: &MatchmakingTicket) -> Result<MatchmakingTicket>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
}

// ============================================================================
// PgGameSessionRepository
// ============================================================================

pub struct PgGameSessionRepository {
    pool: PgPool,
}

impl PgGameSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_game_session(row: sqlx::postgres::PgRow) -> Result<GameSession> {
    let mode_i: i16 = row.get("mode");
    let status_i: i16 = row.get("status");
    let players_json: serde_json::Value = row.get("players");
    let board_json: serde_json::Value = row.get("board_snapshot");
    let pending_moves_json: serde_json::Value = row.get("pending_moves");

    let mode = match mode_i {
        1 => GameMode::Ranked,
        2 => GameMode::Casual,
        3 => GameMode::Room,
        4 => GameMode::PveAi,
        _ => GameMode::Unspecified,
    };
    let status = match status_i {
        1 => SessionStatus::Creating,
        2 => SessionStatus::Waiting,
        3 => SessionStatus::Starting,
        4 => SessionStatus::Running,
        6 => SessionStatus::Paused,
        7 => SessionStatus::Ending,
        8 => SessionStatus::Ended,
        9 => SessionStatus::Canceled,
        _ => SessionStatus::Creating,
    };

    let players: Vec<SessionPlayer> = serde_json::from_value(players_json).map_err(|e| {
        crate::Error::Internal(anyhow::anyhow!("failed to deserialize players: {}", e))
    })?;
    let board: Board = serde_json::from_value(board_json).map_err(|e| {
        crate::Error::Internal(anyhow::anyhow!("failed to deserialize board: {}", e))
    })?;
    let pending_moves: Vec<Move> = serde_json::from_value(pending_moves_json).map_err(|e| {
        crate::Error::Internal(anyhow::anyhow!("failed to deserialize pending_moves: {}", e))
    })?;

    Ok(GameSession {
        match_id: row.get("match_id"),
        mode,
        status,
        players,
        host_id: row.get("host_id"),
        room_code: row.get("room_code"),
        room_password_hash: row.get("room_password_hash"),
        max_players: row.get::<i32, _>("max_players") as u32,
        min_players: row.get::<i32, _>("min_players") as u32,
        turn_index: row.get::<i32, _>("turn_index") as u32,
        current_player_id: row.get("current_player_id"),
        next_turn_deadline_ms: row.get("next_turn_deadline_ms"),
        board,
        board_snapshot_ref: row.get("board_snapshot_ref"),
        winner_id: row.get("winner_id"),
        end_reason: row.get("end_reason"),
        ai_difficulty: row.get::<i32, _>("ai_difficulty") as u32,
        timeout_count: row.get::<i32, _>("timeout_count") as u32,
        pending_moves,
        started_at: row.get("started_at"),
        ended_at: row.get("ended_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

#[async_trait]
impl GameSessionRepository for PgGameSessionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GameSession>> {
        let row = sqlx::query(
            "SELECT match_id, mode, status, players, host_id, room_code, room_password_hash, \
             max_players, min_players, turn_index, current_player_id, next_turn_deadline_ms, \
             board_snapshot, board_snapshot_ref, winner_id, end_reason, ai_difficulty, \
             timeout_count, pending_moves, started_at, ended_at, created_at, updated_at \
             FROM game_sessions WHERE match_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_game_session(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_room_code(&self, room_code: &str) -> Result<Option<GameSession>> {
        let row = sqlx::query(
            "SELECT match_id, mode, status, players, host_id, room_code, room_password_hash, \
             max_players, min_players, turn_index, current_player_id, next_turn_deadline_ms, \
             board_snapshot, board_snapshot_ref, winner_id, end_reason, ai_difficulty, \
             timeout_count, pending_moves, started_at, ended_at, created_at, updated_at \
             FROM game_sessions WHERE room_code = $1",
        )
        .bind(room_code)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_game_session(r)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, entity: &GameSession) -> Result<GameSession> {
        let mode_i = entity.mode as i16;
        let status_i = entity.status as i16;
        let players_json = serde_json::to_value(&entity.players).map_err(|e| {
            crate::Error::Internal(anyhow::anyhow!("failed to serialize players: {}", e))
        })?;
        let board_json = serde_json::to_value(&entity.board).map_err(|e| {
            crate::Error::Internal(anyhow::anyhow!("failed to serialize board: {}", e))
        })?;
        let pending_moves_json = serde_json::to_value(&entity.pending_moves).map_err(|e| {
            crate::Error::Internal(anyhow::anyhow!("failed to serialize pending_moves: {}", e))
        })?;

        sqlx::query(
            "INSERT INTO game_sessions \
             (match_id, mode, status, players, host_id, room_code, room_password_hash, \
              max_players, min_players, turn_index, current_player_id, next_turn_deadline_ms, \
              board_snapshot, board_snapshot_ref, winner_id, end_reason, ai_difficulty, \
              timeout_count, pending_moves, started_at, ended_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23) \
             ON CONFLICT (match_id) DO UPDATE SET \
                mode = EXCLUDED.mode, status = EXCLUDED.status, players = EXCLUDED.players, \
                host_id = EXCLUDED.host_id, room_code = EXCLUDED.room_code, room_password_hash = EXCLUDED.room_password_hash, \
                max_players = EXCLUDED.max_players, min_players = EXCLUDED.min_players, \
                turn_index = EXCLUDED.turn_index, current_player_id = EXCLUDED.current_player_id, \
                next_turn_deadline_ms = EXCLUDED.next_turn_deadline_ms, board_snapshot = EXCLUDED.board_snapshot, \
                board_snapshot_ref = EXCLUDED.board_snapshot_ref, winner_id = EXCLUDED.winner_id, \
                end_reason = EXCLUDED.end_reason, ai_difficulty = EXCLUDED.ai_difficulty, \
                timeout_count = EXCLUDED.timeout_count, pending_moves = EXCLUDED.pending_moves, \
                ended_at = EXCLUDED.ended_at, updated_at = EXCLUDED.updated_at",
        )
        .bind(entity.match_id)
        .bind(mode_i)
        .bind(status_i)
        .bind(players_json)
        .bind(&entity.host_id)
        .bind(&entity.room_code)
        .bind(&entity.room_password_hash)
        .bind(entity.max_players as i32)
        .bind(entity.min_players as i32)
        .bind(entity.turn_index as i32)
        .bind(&entity.current_player_id)
        .bind(entity.next_turn_deadline_ms)
        .bind(board_json)
        .bind(&entity.board_snapshot_ref)
        .bind(&entity.winner_id)
        .bind(&entity.end_reason)
        .bind(entity.ai_difficulty as i32)
        .bind(entity.timeout_count as i32)
        .bind(pending_moves_json)
        .bind(entity.started_at)
        .bind(entity.ended_at)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM game_sessions WHERE match_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_by_status(&self, status: SessionStatus, limit: i64) -> Result<Vec<GameSession>> {
        let status_i = status as i16;
        let rows = sqlx::query(
            "SELECT match_id, mode, status, players, host_id, room_code, room_password_hash, \
             max_players, min_players, turn_index, current_player_id, next_turn_deadline_ms, \
             board_snapshot, board_snapshot_ref, winner_id, end_reason, ai_difficulty, \
             timeout_count, pending_moves, started_at, ended_at, created_at, updated_at \
             FROM game_sessions WHERE status = $1 ORDER BY created_at LIMIT $2",
        )
        .bind(status_i)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_game_session).collect()
    }

    async fn list_by_player(&self, player_id: &str, limit: i64) -> Result<Vec<GameSession>> {
        // JSONB @> containment: players contains object with this player_id
        let query_player = serde_json::json!([{ "player_id": player_id }]);
        let rows = sqlx::query(
            "SELECT match_id, mode, status, players, host_id, room_code, room_password_hash, \
             max_players, min_players, turn_index, current_player_id, next_turn_deadline_ms, \
             board_snapshot, board_snapshot_ref, winner_id, end_reason, ai_difficulty, \
             timeout_count, pending_moves, started_at, ended_at, created_at, updated_at \
             FROM game_sessions WHERE players @> $1::jsonb ORDER BY created_at DESC LIMIT $2",
        )
        .bind(query_player)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_game_session).collect()
    }
}

// ============================================================================
// PgMoveRepository
// ============================================================================

pub struct PgMoveRepository {
    pool: PgPool,
}

impl PgMoveRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_move(row: sqlx::postgres::PgRow) -> Result<Move> {
    let move_type_i: i16 = row.get("move_type");
    let move_type = match move_type_i {
        1 => MoveType::PlayCard,
        2 => MoveType::Attack,
        3 => MoveType::EndTurn,
        4 => MoveType::Surrender,
        5 => MoveType::UseAbility,
        _ => MoveType::Unspecified,
    };
    Ok(Move {
        move_id: row.get("move_id"),
        match_id: row.get("match_id"),
        player_id: row.get("player_id"),
        turn_index: row.get::<i32, _>("turn_index") as u32,
        move_type,
        payload_json: row.get("payload_json"),
        result_json: row.get("result_json"),
        accepted: row.get("accepted"),
        reject_reason: row.get("reject_reason"),
        occurred_at: row.get("occurred_at"),
    })
}

#[async_trait]
impl MoveRepository for PgMoveRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Move>> {
        let row = sqlx::query(
            "SELECT move_id, match_id, player_id, turn_index, move_type, payload_json, result_json, \
             accepted, reject_reason, occurred_at FROM moves WHERE move_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_move(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_match(&self, match_id: Uuid) -> Result<Vec<Move>> {
        let rows = sqlx::query(
            "SELECT move_id, match_id, player_id, turn_index, move_type, payload_json, result_json, \
             accepted, reject_reason, occurred_at FROM moves WHERE match_id = $1 ORDER BY occurred_at",
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_move).collect()
    }

    async fn list_by_match_turn(&self, match_id: Uuid, turn_index: u32) -> Result<Vec<Move>> {
        let rows = sqlx::query(
            "SELECT move_id, match_id, player_id, turn_index, move_type, payload_json, result_json, \
             accepted, reject_reason, occurred_at FROM moves WHERE match_id = $1 AND turn_index = $2 ORDER BY occurred_at",
        )
        .bind(match_id)
        .bind(turn_index as i32)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_move).collect()
    }

    async fn save(&self, entity: &Move) -> Result<Move> {
        let move_type_i = entity.move_type as i16;
        sqlx::query(
            "INSERT INTO moves \
             (move_id, match_id, player_id, turn_index, move_type, payload_json, result_json, \
              accepted, reject_reason, occurred_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (move_id) DO UPDATE SET \
                result_json = EXCLUDED.result_json, accepted = EXCLUDED.accepted, \
                reject_reason = EXCLUDED.reject_reason",
        )
        .bind(entity.move_id)
        .bind(entity.match_id)
        .bind(&entity.player_id)
        .bind(entity.turn_index as i32)
        .bind(move_type_i)
        .bind(&entity.payload_json)
        .bind(&entity.result_json)
        .bind(entity.accepted)
        .bind(&entity.reject_reason)
        .bind(entity.occurred_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM moves WHERE move_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// PgMatchmakingTicketRepository
// ============================================================================

pub struct PgMatchmakingTicketRepository {
    pool: PgPool,
}

impl PgMatchmakingTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_ticket(row: sqlx::postgres::PgRow) -> Result<MatchmakingTicket> {
    let mode_i: i16 = row.get("mode");
    let mode = match mode_i {
        1 => GameMode::Ranked,
        2 => GameMode::Casual,
        3 => GameMode::Room,
        4 => GameMode::PveAi,
        _ => GameMode::Unspecified,
    };
    Ok(MatchmakingTicket {
        ticket_id: row.get("ticket_id"),
        player_id: row.get("player_id"),
        mode,
        rank_score_min: row.get::<i32, _>("rank_score_min") as u32,
        rank_score_max: row.get::<i32, _>("rank_score_max") as u32,
        deck_card_id: row.get("deck_ref_card_id"),
        deck_instance_id: row.get("deck_ref_inst_id"),
        status: row.get::<i16, _>("status") as i32,
        match_id: row.get("match_id"),
        created_at: row.get("created_at"),
        matched_at: row.get("matched_at"),
        cancelled_at: row.get("cancelled_at"),
        expires_at: row.get("expires_at"),
    })
}

#[async_trait]
impl MatchmakingTicketRepository for PgMatchmakingTicketRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MatchmakingTicket>> {
        let row = sqlx::query(
            "SELECT ticket_id, player_id, mode, rank_score_min, rank_score_max, deck_ref_card_id, \
             deck_ref_inst_id, status, match_id, created_at, matched_at, cancelled_at, expires_at \
             FROM matchmaking_tickets WHERE ticket_id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(row_to_ticket(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_player(&self, player_id: &str) -> Result<Vec<MatchmakingTicket>> {
        let rows = sqlx::query(
            "SELECT ticket_id, player_id, mode, rank_score_min, rank_score_max, deck_ref_card_id, \
             deck_ref_inst_id, status, match_id, created_at, matched_at, cancelled_at, expires_at \
             FROM matchmaking_tickets WHERE player_id = $1 ORDER BY created_at DESC",
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_ticket).collect()
    }

    async fn list_by_status(&self, status: i32) -> Result<Vec<MatchmakingTicket>> {
        let rows = sqlx::query(
            "SELECT ticket_id, player_id, mode, rank_score_min, rank_score_max, deck_ref_card_id, \
             deck_ref_inst_id, status, match_id, created_at, matched_at, cancelled_at, expires_at \
             FROM matchmaking_tickets WHERE status = $1 ORDER BY created_at ASC",
        )
        .bind(status as i16)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_ticket).collect()
    }

    async fn find_matchable(
        &self,
        mode: GameMode,
        rank_score: u32,
    ) -> Result<Vec<MatchmakingTicket>> {
        let mode_i = mode as i16;
        let rows = sqlx::query(
            "SELECT ticket_id, player_id, mode, rank_score_min, rank_score_max, deck_ref_card_id, \
             deck_ref_inst_id, status, match_id, created_at, matched_at, cancelled_at, expires_at \
             FROM matchmaking_tickets \
             WHERE status = 1 AND mode = $1 \
               AND rank_score_min <= $2 AND rank_score_max >= $2 \
               AND expires_at > now() \
             ORDER BY created_at ASC LIMIT 10",
        )
        .bind(mode_i)
        .bind(rank_score as i32)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_ticket).collect()
    }

    async fn save(&self, entity: &MatchmakingTicket) -> Result<MatchmakingTicket> {
        let mode_i = entity.mode as i16;
        sqlx::query(
            "INSERT INTO matchmaking_tickets \
             (ticket_id, player_id, mode, rank_score_min, rank_score_max, deck_ref_card_id, \
              deck_ref_inst_id, status, match_id, created_at, matched_at, cancelled_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (ticket_id) DO UPDATE SET \
                status = EXCLUDED.status, match_id = EXCLUDED.match_id, \
                matched_at = EXCLUDED.matched_at, cancelled_at = EXCLUDED.cancelled_at",
        )
        .bind(entity.ticket_id)
        .bind(&entity.player_id)
        .bind(mode_i)
        .bind(entity.rank_score_min as i32)
        .bind(entity.rank_score_max as i32)
        .bind(&entity.deck_card_id)
        .bind(&entity.deck_instance_id)
        .bind(entity.status as i16)
        .bind(entity.match_id)
        .bind(entity.created_at)
        .bind(entity.matched_at)
        .bind(entity.cancelled_at)
        .bind(entity.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM matchmaking_tickets WHERE ticket_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// InMemoryGameSessionRepository
// ============================================================================

pub struct InMemoryGameSessionRepository {
    inner: Mutex<HashMap<Uuid, GameSession>>,
}

impl InMemoryGameSessionRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryGameSessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameSessionRepository for InMemoryGameSessionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GameSession>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_room_code(&self, room_code: &str) -> Result<Option<GameSession>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|s| s.room_code.as_deref() == Some(room_code))
            .cloned())
    }
    async fn save(&self, entity: &GameSession) -> Result<GameSession> {
        self.inner
            .lock()
            .unwrap()
            .insert(entity.match_id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
    async fn list_by_status(&self, status: SessionStatus, limit: i64) -> Result<Vec<GameSession>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.status == status)
            .take(limit as usize)
            .cloned()
            .collect())
    }
    async fn list_by_player(&self, player_id: &str, limit: i64) -> Result<Vec<GameSession>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|s| s.players.iter().any(|p| p.player_id == player_id))
            .take(limit as usize)
            .cloned()
            .collect())
    }
}

// ============================================================================
// InMemoryMoveRepository
// ============================================================================

pub struct InMemoryMoveRepository {
    inner: Mutex<HashMap<Uuid, Move>>,
}

impl InMemoryMoveRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryMoveRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MoveRepository for InMemoryMoveRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Move>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_match(&self, match_id: Uuid) -> Result<Vec<Move>> {
        let mut v: Vec<Move> = self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.match_id == match_id)
            .cloned()
            .collect();
        v.sort_by_key(|m| m.occurred_at);
        Ok(v)
    }
    async fn list_by_match_turn(&self, match_id: Uuid, turn_index: u32) -> Result<Vec<Move>> {
        let mut v: Vec<Move> = self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.match_id == match_id && m.turn_index == turn_index)
            .cloned()
            .collect();
        v.sort_by_key(|m| m.occurred_at);
        Ok(v)
    }
    async fn save(&self, entity: &Move) -> Result<Move> {
        self.inner.lock().unwrap().insert(entity.move_id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
}

// ============================================================================
// InMemoryMatchmakingTicketRepository
// ============================================================================

pub struct InMemoryMatchmakingTicketRepository {
    inner: Mutex<HashMap<Uuid, MatchmakingTicket>>,
}

impl InMemoryMatchmakingTicketRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryMatchmakingTicketRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MatchmakingTicketRepository for InMemoryMatchmakingTicketRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<MatchmakingTicket>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn list_by_player(&self, player_id: &str) -> Result<Vec<MatchmakingTicket>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|t| t.player_id == player_id)
            .cloned()
            .collect())
    }
    async fn list_by_status(&self, status: i32) -> Result<Vec<MatchmakingTicket>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect())
    }
    async fn find_matchable(
        &self,
        mode: GameMode,
        rank_score: u32,
    ) -> Result<Vec<MatchmakingTicket>> {
        let now: DateTime<Utc> = Utc::now();
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|t| {
                t.status == 1
                    && t.mode == mode
                    && t.rank_score_min <= rank_score
                    && t.rank_score_max >= rank_score
                    && t.expires_at > now
            })
            .take(10)
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &MatchmakingTicket) -> Result<MatchmakingTicket> {
        self.inner
            .lock()
            .unwrap()
            .insert(entity.ticket_id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player(id: &str) -> SessionPlayer {
        SessionPlayer::new(id.to_string(), format!("P-{}", id))
    }

    fn make_session(num_players: usize) -> GameSession {
        let host = make_player("p1");
        let mut s = GameSession::new(GameMode::Ranked, host, 2, 2);
        for i in 2..=num_players {
            s.add_player(make_player(&format!("p{}", i))).unwrap();
        }
        s
    }

    #[tokio::test]
    async fn in_memory_game_session_lifecycle() {
        let repo = InMemoryGameSessionRepository::new();
        let mut s = make_session(2);
        let id = s.match_id;
        s.transition_to_starting().unwrap();
        s.transition_to_running().unwrap();
        repo.save(&s).await.unwrap();

        let loaded = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(loaded.status, SessionStatus::Running);
        assert_eq!(loaded.players.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_game_session_find_by_room_code() {
        let repo = InMemoryGameSessionRepository::new();
        let mut s = GameSession::new(GameMode::Room, make_player("p1"), 4, 2);
        s.room_code = Some("ROOM42".to_string());
        s.transition_to_waiting().unwrap();
        repo.save(&s).await.unwrap();

        let found = repo.find_by_room_code("ROOM42").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().room_code, Some("ROOM42".to_string()));
    }

    #[tokio::test]
    async fn in_memory_game_session_list_by_status() {
        let repo = InMemoryGameSessionRepository::new();
        let s1 = make_session(2);
        let mut s2 = make_session(2);
        s2.transition_to_starting().unwrap();
        s2.transition_to_running().unwrap();
        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();

        let running = repo.list_by_status(SessionStatus::Running, 10).await.unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].match_id, s2.match_id);
    }

    #[tokio::test]
    async fn in_memory_game_session_list_by_player() {
        let repo = InMemoryGameSessionRepository::new();
        let s1 = make_session(2);
        let s2 = make_session(2);
        repo.save(&s1).await.unwrap();
        repo.save(&s2).await.unwrap();

        let p1_sessions = repo.list_by_player("p1", 10).await.unwrap();
        assert_eq!(p1_sessions.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_game_session_delete() {
        let repo = InMemoryGameSessionRepository::new();
        let s = make_session(2);
        let id = s.match_id;
        repo.save(&s).await.unwrap();
        let deleted = repo.delete_by_id(id).await.unwrap();
        assert!(deleted);
        let found = repo.find_by_id(id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn in_memory_move_save_and_list() {
        let repo = InMemoryMoveRepository::new();
        let match_id = Uuid::new_v4();
        let m1 = Move::new(match_id, "p1".to_string(), 0, MoveType::PlayCard, "{}".to_string());
        let m2 = Move::new(match_id, "p1".to_string(), 0, MoveType::EndTurn, "{}".to_string());
        repo.save(&m1).await.unwrap();
        repo.save(&m2).await.unwrap();

        let list = repo.list_by_match(match_id).await.unwrap();
        assert_eq!(list.len(), 2);
        let turn0 = repo.list_by_match_turn(match_id, 0).await.unwrap();
        assert_eq!(turn0.len(), 2);
        let turn1 = repo.list_by_match_turn(match_id, 1).await.unwrap();
        assert_eq!(turn1.len(), 0);
    }

    #[tokio::test]
    async fn in_memory_ticket_find_matchable() {
        let repo = InMemoryMatchmakingTicketRepository::new();
        let t1 = MatchmakingTicket::new("p1".to_string(), GameMode::Ranked, 1000, 2000, None, None);
        let t2 = MatchmakingTicket::new("p2".to_string(), GameMode::Ranked, 1500, 2500, None, None);
        let t3 = MatchmakingTicket::new("p3".to_string(), GameMode::Casual, 0, 0, None, None);
        repo.save(&t1).await.unwrap();
        repo.save(&t2).await.unwrap();
        repo.save(&t3).await.unwrap();

        // 1700 应匹配 t1+t2 (rank 范围内), 不匹配 t3 (mode 不同)
        let matched = repo.find_matchable(GameMode::Ranked, 1700).await.unwrap();
        assert_eq!(matched.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_ticket_lifecycle() {
        let repo = InMemoryMatchmakingTicketRepository::new();
        let mut t = MatchmakingTicket::new("p1".to_string(), GameMode::Casual, 0, 0, None, None);
        let tid = t.ticket_id;
        repo.save(&t).await.unwrap();
        t.matched(Uuid::new_v4());
        repo.save(&t).await.unwrap();
        let loaded = repo.find_by_id(tid).await.unwrap().unwrap();
        assert_eq!(loaded.status, 2);
    }
}
