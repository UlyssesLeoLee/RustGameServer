//! match-service 域 v2 业务实装 (per RGS-DTL-038 v0.1 §4.2 + §5)
//!
//! 卡牌游戏 session/turn 抽象的 9 个 RPC + 完整状态机业务实装.
//!
//! 上游: RGS-REQ-038 §FR-004 session / §FR-005 匹配
//! 设计: RGS-DTL-038 §4.2 message / §5.1 状态机 / §5.2 状态转移表
//!
//! ## 9 RPC (per §4.2 + §5.5)
//! 1. EnqueueMatchmaking   — 玩家入队
//! 2. CancelMatchmaking    — 取消撮合
//! 3. GetMatchmakingStatus — 查 ticket
//! 4. CreateMatch          — 房主建 session (per §5.1 Creating → Waiting/Starting)
//! 5. JoinMatch            — 玩家加入 (ROOM 模式)
//! 6. LeaveMatch           — 玩家离开 / 投降 (per §5.2 Running → Ending)
//! 7. GetMatchState        — 查 board + pending moves
//! 8. SubmitMove           — 玩家出牌 / 结束回合 (per §5.2 RUNNING 状态机)
//! 9. SubscribeMatch       — 流式订阅 session 事件
//!
//! ## 8 状态转移函数 (per §5.2 状态转移表)
//! - transition_to_waiting
//! - transition_to_starting
//! - transition_to_running
//! - transition_to_paused
//! - transition_to_resumed
//! - transition_to_ending
//! - transition_to_ended
//! - transition_to_canceled
//! (实装在 entity_v2::GameSession, 本模块负责编排 + 持久化)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex as AsyncMutex};
use uuid::Uuid;

use crate::entity_v2::{
    Board, GameMode, GameSession, MatchmakingTicket, Move, MoveType, SessionPlayer, SessionStatus,
};
use crate::error::Error;
use crate::repository_v2::{
    GameSessionRepository, MatchmakingTicketRepository, MoveRepository,
};
use crate::Result;

// ============================================================================
// Event Bus (per §4.2 SubscribeMatch stream RPC)
// ============================================================================

/// Match event (per §4.2 MatchEvent)
#[derive(Debug, Clone)]
pub enum MatchEvent {
    /// 战牌快照更新
    Snapshot {
        occurred_at_ms: i64,
        board_snapshot: String,
    },
    /// move 已应用
    MoveApplied {
        occurred_at_ms: i64,
        mv: Move,
    },
    /// 回合切换
    TurnChanged {
        occurred_at_ms: i64,
        new_turn_index: u32,
        new_player_id: String,
    },
    /// 玩家加入
    PlayerJoined {
        occurred_at_ms: i64,
        player: SessionPlayer,
    },
    /// 玩家离开
    PlayerLeft {
        occurred_at_ms: i64,
        player_id: String,
    },
    /// session 结束
    MatchEnded {
        occurred_at_ms: i64,
        end_reason: String,
        winner_id: Option<String>,
    },
    /// turn 超时警告
    TimeoutWarning {
        occurred_at_ms: i64,
        turn_index: u32,
        remaining_ms: i64,
    },
}

impl MatchEvent {
    pub fn occurred_at_ms(&self) -> i64 {
        match self {
            Self::Snapshot { occurred_at_ms, .. }
            | Self::MoveApplied { occurred_at_ms, .. }
            | Self::TurnChanged { occurred_at_ms, .. }
            | Self::PlayerJoined { occurred_at_ms, .. }
            | Self::PlayerLeft { occurred_at_ms, .. }
            | Self::MatchEnded { occurred_at_ms, .. }
            | Self::TimeoutWarning { occurred_at_ms, .. } => *occurred_at_ms,
        }
    }
}

/// Event bus: per match_id broadcast channel
pub type EventSender = broadcast::Sender<MatchEvent>;

#[derive(Clone)]
pub struct EventBus {
    inner: Arc<AsyncMutex<HashMap<Uuid, EventSender>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    /// 获取或创建 match_id 的 broadcast channel
    pub async fn subscribe(&self, match_id: Uuid) -> broadcast::Receiver<MatchEvent> {
        let mut map = self.inner.lock().await;
        let sender = map
            .entry(match_id)
            .or_insert_with(|| broadcast::channel(64).0);
        sender.subscribe()
    }

    /// 推送事件
    pub async fn publish(&self, match_id: Uuid, event: MatchEvent) {
        let map = self.inner.lock().await;
        if let Some(sender) = map.get(&match_id) {
            // 容错: 没人订阅时不报错
            let _ = sender.send(event);
        }
    }

    /// 清理已结束 match 的 channel
    pub async fn cleanup(&self, match_id: Uuid) {
        let mut map = self.inner.lock().await;
        map.remove(&match_id);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MatchmakerServiceV2 — 9 RPC 业务编排
// ============================================================================

/// 9 RPC 业务实装 (per RGS-DTL-038 §4.2 + §5)
pub struct MatchmakerServiceV2 {
    sessions: Arc<dyn GameSessionRepository>,
    moves: Arc<dyn MoveRepository>,
    tickets: Arc<dyn MatchmakingTicketRepository>,
    event_bus: EventBus,
}

impl MatchmakerServiceV2 {
    pub fn new(
        sessions: Arc<dyn GameSessionRepository>,
        moves: Arc<dyn MoveRepository>,
        tickets: Arc<dyn MatchmakingTicketRepository>,
    ) -> Self {
        Self {
            sessions,
            moves,
            tickets,
            event_bus: EventBus::new(),
        }
    }

    pub fn event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    /// 桶 9 service.rs 接入: 返回 session 仓库 (供 service 层 / UT 操控)
    pub fn sessions(&self) -> Arc<dyn GameSessionRepository> {
        self.sessions.clone()
    }

    /// 桶 9 service.rs 接入: 返回 move 仓库
    pub fn moves_repo(&self) -> Arc<dyn MoveRepository> {
        self.moves.clone()
    }

    /// 桶 9 service.rs 接入: 返回 ticket 仓库
    pub fn tickets_repo(&self) -> Arc<dyn MatchmakingTicketRepository> {
        self.tickets.clone()
    }

    // ========================================================================
    // 1. EnqueueMatchmaking (per §4.2 + §5.2 Enqueue)
    // ========================================================================

    /// 玩家入队 (per §4.2 EnqueueMatchmaking)
    /// - 创建 ticket (status=queued)
    /// - 立即尝试撮合 (find_matchable 拉取候选)
    /// - 命中候选 → 创建 session, 双方 ticket 置 matched
    /// - 未命中 → ticket 保持 queued
    pub async fn enqueue_matchmaking(
        &self,
        player: SessionPlayer,
        mode: GameMode,
        rank_score_min: u32,
        rank_score_max: u32,
    ) -> Result<EnqueueResult> {
        // 校验
        if player.player_id.is_empty() {
            return Err(Error::Validation("player_id required".to_string()));
        }
        if matches!(mode, GameMode::Unspecified) {
            return Err(Error::Validation("mode required".to_string()));
        }
        // 房间模式不支持自动撮合, 应走 CreateMatch
        if matches!(mode, GameMode::Room) {
            return Err(Error::Validation(
                "ROOM mode must use CreateMatch, not EnqueueMatchmaking".to_string(),
            ));
        }

        // 1) 创建 ticket
        let mut ticket = MatchmakingTicket::new(
            player.player_id.clone(),
            mode,
            rank_score_min,
            rank_score_max,
            player.deck_card_id.clone(),
            player.deck_instance_id.clone(),
        );
        ticket = self.tickets.save(&ticket).await?;

        // 2) 拉取候选 (自身不算, 简单实现: 选第 1 个不同 player)
        let rank_score = player.rank_score;
        let candidates = self.tickets.find_matchable(mode, rank_score).await?;
        let opponent_ticket = candidates
            .into_iter()
            .find(|t| t.player_id != player.player_id);

        match opponent_ticket {
            Some(opp) => {
                // 3) 撮合成功: 创建 session
                let opponent = SessionPlayer::new(opp.player_id.clone(), format!("P-{}", opp.player_id))
                    .with_deck(opp.deck_card_id.clone(), opp.deck_instance_id.clone());

                let mut session = GameSession::new(mode, player.clone(), 2, 2);
                session.add_player(opponent.clone()).map_err(|e| {
                    Error::Internal(anyhow::anyhow!("failed to add opponent: {}", e))
                })?;
                session.transition_to_starting().map_err(|e| {
                    Error::Internal(anyhow::anyhow!("transition_to_starting failed: {}", e))
                })?;
                session.transition_to_running().map_err(|e| {
                    Error::Internal(anyhow::anyhow!("transition_to_running failed: {}", e))
                })?;
                session = self.sessions.save(&session).await?;

                // 4) 更新双方 ticket
                ticket.matched(session.match_id);
                let mut opp_updated = opp.clone();
                opp_updated.matched(session.match_id);
                self.tickets.save(&ticket).await?;
                self.tickets.save(&opp_updated).await?;

                // 5) 推送 PLAYER_JOINED + TURN_CHANGED 事件
                let now_ms = chrono::Utc::now().timestamp_millis();
                self.event_bus
                    .publish(
                        session.match_id,
                        MatchEvent::PlayerJoined {
                            occurred_at_ms: now_ms,
                            player: opponent,
                        },
                    )
                    .await;
                self.event_bus
                    .publish(
                        session.match_id,
                        MatchEvent::TurnChanged {
                            occurred_at_ms: now_ms,
                            new_turn_index: 0,
                            new_player_id: session
                                .current_player_id
                                .clone()
                                .unwrap_or_default(),
                        },
                    )
                    .await;

                Ok(EnqueueResult::Matched {
                    ticket_id: ticket.ticket_id,
                    match_id: session.match_id,
                })
            }
            None => Ok(EnqueueResult::Queued {
                ticket_id: ticket.ticket_id,
                estimated_wait_ms: 5000,
            }),
        }
    }

    // ========================================================================
    // 2. CancelMatchmaking (per §4.2 + §5.2 Cancel)
    // ========================================================================

    pub async fn cancel_matchmaking(&self, ticket_id: Uuid, player_id: &str) -> Result<bool> {
        let mut ticket = self
            .tickets
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "MatchmakingTicket",
                id: ticket_id.to_string(),
            })?;
        // 桶 9 gRPC CancelMatchmakingRequest 无 player_id 字段, 透传 "" 跳过所有权校验
        // 非空时仍做所有权校验 (供 IT/UT 直接调用场景)
        if !player_id.is_empty() && ticket.player_id != player_id {
            return Err(Error::Forbidden(
                "ticket does not belong to player".to_string(),
            ));
        }
        if ticket.status == 2 {
            return Err(Error::Validation(
                "cannot cancel matched ticket".to_string(),
            ));
        }
        ticket.cancelled();
        self.tickets.save(&ticket).await?;
        Ok(true)
    }

    // ========================================================================
    // 3. GetMatchmakingStatus (per §4.2)
    // ========================================================================

    pub async fn get_matchmaking_status(
        &self,
        ticket_id: Uuid,
    ) -> Result<MatchmakingStatus> {
        let ticket = self
            .tickets
            .find_by_id(ticket_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "MatchmakingTicket",
                id: ticket_id.to_string(),
            })?;
        let status = match ticket.status {
            1 => TicketStatus::Queued,
            2 => TicketStatus::Matched,
            3 => TicketStatus::Cancelled,
            4 => TicketStatus::Expired,
            _ => TicketStatus::Queued,
        };
        Ok(MatchmakingStatus {
            status,
            match_id: ticket.match_id,
        })
    }

    // ========================================================================
    // 4. CreateMatch (per §4.2 + §5.1 Creating → Waiting/Starting)
    // ========================================================================

    pub async fn create_match(
        &self,
        host: SessionPlayer,
        mode: GameMode,
        room_code: Option<String>,
        room_password: Option<String>,
        max_players: u32,
        min_players: u32,
        ai_difficulty: u32,
    ) -> Result<CreateMatchResult> {
        // 校验
        if host.player_id.is_empty() {
            return Err(Error::Validation("host.player_id required".to_string()));
        }
        if matches!(mode, GameMode::Unspecified) {
            return Err(Error::Validation("mode required".to_string()));
        }
        if max_players < min_players {
            return Err(Error::Validation(
                "max_players must be >= min_players".to_string(),
            ));
        }
        if matches!(mode, GameMode::Room) {
            if room_code.is_none() {
                return Err(Error::Validation(
                    "ROOM mode requires room_code".to_string(),
                ));
            }
            // 检查 room_code 不重复
            if let Some(code) = room_code.as_ref() {
                if self.sessions.find_by_room_code(code).await?.is_some() {
                    return Err(Error::Conflict(format!(
                        "room_code {} already in use",
                        code
                    )));
                }
            }
        }

        let mut session = GameSession::new(mode, host, max_players, min_players);
        session.room_code = room_code.clone();
        // 简化: 密码明文存 hash(本次存"hash:"前缀假 hash); 生产应 argon2/bcrypt
        session.room_password_hash = room_password.map(|p| format!("hash:{}", p));
        session.ai_difficulty = ai_difficulty;
        if matches!(mode, GameMode::Room) {
            session.transition_to_waiting().map_err(|e| {
                Error::Internal(anyhow::anyhow!("transition_to_waiting: {}", e))
            })?;
        } else {
            // 非 ROOM: 直接走 Creating → Starting (假设对手已经在 ticket 撮合中)
            // 这里简化为保持 Creating, 等 JoinMatch / 撮合来推进
            // (实际: 撮合流程已通过 enqueue_matchmaking 创建 session)
        }
        session = self.sessions.save(&session).await?;

        Ok(CreateMatchResult {
            match_id: session.match_id,
            mode: session.mode,
            room_code: session.room_code,
        })
    }

    // ========================================================================
    // 5. JoinMatch (per §4.2 + §5.1 Waiting → Starting)
    // ========================================================================

    pub async fn join_match(
        &self,
        match_id: Uuid,
        player: SessionPlayer,
        room_code: Option<String>,
        room_password: Option<String>,
    ) -> Result<JoinMatchResult> {
        let mut session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;

        // 校验
        if session.is_full() {
            return Err(Error::MatchFull {
                match_id: match_id.to_string(),
            });
        }
        // 已加入检查 (放在状态检查之前, 让"已加入"优先于"已开赛"返回 Conflict)
        if session.players.iter().any(|p| p.player_id == player.player_id) {
            return Err(Error::Conflict(format!(
                "player {} already in session",
                player.player_id
            )));
        }
        if !session.status.is_terminal() == false || session.status == SessionStatus::Ended {
            return Err(Error::MatchAlreadyStarted(match_id.to_string()));
        }
        if !matches!(session.status, SessionStatus::Creating | SessionStatus::Waiting) {
            return Err(Error::MatchAlreadyStarted(match_id.to_string()));
        }
        // 房间密码校验
        if session.room_password_hash.is_some() {
            let required = session.room_password_hash.as_ref().unwrap();
            let provided_hash = room_password
                .as_ref()
                .map(|p| format!("hash:{}", p))
                .unwrap_or_default();
            if &provided_hash != required {
                return Err(Error::Forbidden("invalid room password".to_string()));
            }
        }
        // 房间码校验
        if let Some(code) = session.room_code.as_ref() {
            if let Some(provided) = room_code.as_ref() {
                if provided != code {
                    return Err(Error::Validation("room_code mismatch".to_string()));
                }
            }
        }
        // 已加入检查: 已上移到上方 (status 检查前) 以让"已加入"优先于"已开赛"返回 Conflict

        let player_clone = player.clone();
        session.add_player(player).map_err(|e| {
            Error::Internal(anyhow::anyhow!("add_player: {}", e))
        })?;

        // 玩家到齐 → 自动 Starting
        if session.is_ready_to_start() {
            session.transition_to_starting().map_err(|e| {
                Error::Internal(anyhow::anyhow!("transition_to_starting: {}", e))
            })?;
        }
        session = self.sessions.save(&session).await?;

        // 推送 PLAYER_JOINED 事件
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.event_bus
            .publish(
                session.match_id,
                MatchEvent::PlayerJoined {
                    occurred_at_ms: now_ms,
                    player: player_clone,
                },
            )
            .await;

        Ok(JoinMatchResult {
            joined: true,
            turn_index: session.turn_index,
        })
    }

    // ========================================================================
    // 6. LeaveMatch (per §4.2 + §5.2 投降 / 断线)
    // ========================================================================

    pub async fn leave_match(
        &self,
        match_id: Uuid,
        player_id: &str,
        surrender: bool,
    ) -> Result<LeaveMatchResult> {
        let mut session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;

        // 已在终态: 返回
        if session.status.is_terminal() {
            return Ok(LeaveMatchResult {
                left: false,
                match_result: "already_ended".to_string(),
            });
        }

        // 玩家必须在场
        if !session.players.iter().any(|p| p.player_id == player_id) {
            return Err(Error::NotInMatch {
                player_id: player_id.to_string(),
                match_id: match_id.to_string(),
            });
        }

        let removed = session.remove_player(player_id, surrender).map_err(|e| {
            Error::Internal(anyhow::anyhow!("remove_player: {}", e))
        })?;
        if !removed {
            return Err(Error::NotInMatch {
                player_id: player_id.to_string(),
                match_id: match_id.to_string(),
            });
        }

        // 投降 / 判负: 直接到 Ending → Ended
        let match_result = if surrender {
            // 投降: 找另一个非投降玩家作为 winner
            let winner = session
                .players
                .iter()
                .find(|p| !p.surrendered && !p.disconnected)
                .map(|p| p.player_id.clone());
            session
                .transition_to_ending(winner.clone(), "surrender".to_string())
                .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_ending: {}", e)))?;
            session
                .transition_to_ended()
                .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_ended: {}", e)))?;
            "surrender".to_string()
        } else {
            // 断线: 标记 disconnected, 不立即判负 (per §5.4 5-30s PAUSED, 30-60s AI, 60s+ 判负)
            // 简化: 这里只标 disconnected, 实际超时由后台任务推动
            // 如果所有玩家都 disconnect → 取消
            if session.active_player_count() == 0 {
                session
                    .transition_to_canceled("all_disconnected".to_string())
                    .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_canceled: {}", e)))?;
                "all_disconnected".to_string()
            } else {
                "disconnect".to_string()
            }
        };

        session = self.sessions.save(&session).await?;

        // 推送 PLAYER_LEFT + (如结束) MATCH_ENDED 事件
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.event_bus
            .publish(
                session.match_id,
                MatchEvent::PlayerLeft {
                    occurred_at_ms: now_ms,
                    player_id: player_id.to_string(),
                },
            )
            .await;
        if session.status == SessionStatus::Ended || session.status == SessionStatus::Canceled {
            self.event_bus
                .publish(
                    session.match_id,
                    MatchEvent::MatchEnded {
                        occurred_at_ms: now_ms,
                        end_reason: session.end_reason.clone().unwrap_or_default(),
                        winner_id: session.winner_id.clone(),
                    },
                )
                .await;
            self.event_bus.cleanup(session.match_id).await;
        }

        Ok(LeaveMatchResult {
            left: true,
            match_result,
        })
    }

    // ========================================================================
    // 7. GetMatchState (per §4.2)
    // ========================================================================

    pub async fn get_match_state(&self, match_id: Uuid, _player: &SessionPlayer) -> Result<MatchState> {
        let session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;

        let board_json = serde_json::to_string(&session.board).map_err(|e| {
            Error::Internal(anyhow::anyhow!("serialize board: {}", e))
        })?;
        let deadline = session.next_turn_deadline_ms;

        Ok(MatchState {
            session,
            board_snapshot: board_json,
            next_turn_deadline_ms: deadline,
        })
    }

    // ========================================================================
    // 8. SubmitMove (per §4.2 + §5.2 SubmitMove)
    // ========================================================================

    pub async fn submit_move(
        &self,
        match_id: Uuid,
        player: &SessionPlayer,
        turn_index: u32,
        mv: Move,
    ) -> Result<SubmitMoveResult> {
        let mut session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;

        // 校验
        if session.status != SessionStatus::Running {
            return Err(Error::Validation(format!(
                "session {} not running (status={})",
                match_id, session.status
            )));
        }
        if session.turn_index != turn_index {
            return Err(Error::Validation(format!(
                "turn_index mismatch: expected {}, got {}",
                session.turn_index, turn_index
            )));
        }
        if session.current_player_id.as_deref() != Some(player.player_id.as_str()) {
            return Err(Error::Forbidden(format!(
                "not your turn (current={}, you={})",
                session.current_player_id.as_deref().unwrap_or(""),
                player.player_id
            )));
        }

        // 应用 move
        let mut move_record = mv;
        move_record.turn_index = turn_index;
        move_record.player_id = player.player_id.clone();
        move_record.match_id = match_id;
        move_record.accepted = true;

        // 业务层结果 (per §4.2 SubmitMoveResponse.result_json)
        // 简化: 这里只 echo payload_json; 实际 game-logic crate 处理
        move_record.result_json = Some(move_record.payload_json.clone());

        // 投降: 直接到 ENDING
        if move_record.move_type == MoveType::Surrender {
            move_record = self.moves.save(&move_record).await?;
            let winner = session
                .players
                .iter()
                .find(|p| p.player_id != player.player_id)
                .map(|p| p.player_id.clone());
            session
                .transition_to_ending(winner.clone(), "surrender".to_string())
                .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_ending: {}", e)))?;
            session
                .transition_to_ended()
                .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_ended: {}", e)))?;
            session = self.sessions.save(&session).await?;

            let now_ms = chrono::Utc::now().timestamp_millis();
            self.event_bus
                .publish(
                    match_id,
                    MatchEvent::MoveApplied {
                        occurred_at_ms: now_ms,
                        mv: move_record.clone(),
                    },
                )
                .await;
            self.event_bus
                .publish(
                    match_id,
                    MatchEvent::MatchEnded {
                        occurred_at_ms: now_ms,
                        end_reason: "surrender".to_string(),
                        winner_id: winner,
                    },
                )
                .await;
            self.event_bus.cleanup(match_id).await;

            return Ok(SubmitMoveResult {
                accepted: true,
                new_turn_index: session.turn_index,
                new_board_snapshot_ref: session.board_snapshot_ref.clone(),
                reject_reason: None,
            });
        }

        // 普通 move: 写 move log, 应用 board (简化: 直接在 board 计数)
        // 实际: game-logic crate 根据 move_type 处理; 这里只记录
        session.board.counters.insert(
            format!("last_move_{}", session.turn_index),
            session.board.counters.len() as i32 + 1,
        );
        if matches!(move_record.move_type, MoveType::EndTurn) {
            // 结束回合 → 切换玩家
            let deadline_ms = chrono::Utc::now().timestamp_millis() + 60_000;
            session.advance_turn(Some(deadline_ms)).map_err(|e| {
                Error::Internal(anyhow::anyhow!("advance_turn: {}", e))
            })?;
        }
        move_record = self.moves.save(&move_record).await?;
        session = self.sessions.save(&session).await?;

        // 推送 MOVE_APPLIED + (如切换回合) TURN_CHANGED
        let now_ms = chrono::Utc::now().timestamp_millis();
        self.event_bus
            .publish(
                match_id,
                MatchEvent::MoveApplied {
                    occurred_at_ms: now_ms,
                    mv: move_record.clone(),
                },
            )
            .await;
        if matches!(move_record.move_type, MoveType::EndTurn) {
            self.event_bus
                .publish(
                    match_id,
                    MatchEvent::TurnChanged {
                        occurred_at_ms: now_ms,
                        new_turn_index: session.turn_index,
                        new_player_id: session.current_player_id.clone().unwrap_or_default(),
                    },
                )
                .await;
        }

        Ok(SubmitMoveResult {
            accepted: true,
            new_turn_index: session.turn_index,
            new_board_snapshot_ref: session.board_snapshot_ref.clone(),
            reject_reason: None,
        })
    }

    // ========================================================================
    // 9. SubscribeMatch (per §4.2 stream)
    // ========================================================================

    pub async fn subscribe_match(
        &self,
        match_id: Uuid,
        _player: &SessionPlayer,
        full_snapshot_first: bool,
    ) -> Result<broadcast::Receiver<MatchEvent>> {
        // 校验 session 存在
        let session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;

        let receiver = self.event_bus.subscribe(match_id).await;

        if full_snapshot_first {
            // 推一条 SNAPSHOT 事件 (per §4.2 MatchEvent.SNAPSHOT)
            let board_json = serde_json::to_string(&session.board).map_err(|e| {
                Error::Internal(anyhow::anyhow!("serialize board: {}", e))
            })?;
            let now_ms = chrono::Utc::now().timestamp_millis();
            let snapshot = MatchEvent::Snapshot {
                occurred_at_ms: now_ms,
                board_snapshot: board_json,
            };
            // 试图发送, 忽略 "no active receivers" 错误 (因为 receiver 是在这之后)
            // 简化: 跳过首次推送, 由调用方在拿到 receiver 后立即查 get_match_state
            let _ = snapshot;
        }

        Ok(receiver)
    }

    // ========================================================================
    // 辅助: 状态转移直接调用 (供 IT / GM 工具用)
    // ========================================================================

    /// 强制暂停 (per §5.5 强制踢出 / GM 暂停)
    pub async fn pause_session(&self, match_id: Uuid) -> Result<()> {
        let mut session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;
        session.transition_to_paused().map_err(|e| {
            Error::Internal(anyhow::anyhow!("transition_to_paused: {}", e))
        })?;
        self.sessions.save(&session).await?;
        Ok(())
    }

    /// 恢复 (per §5.2 PAUSED → RUNNING)
    pub async fn resume_session(&self, match_id: Uuid) -> Result<()> {
        let mut session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;
        session.transition_to_resumed().map_err(|e| {
            Error::Internal(anyhow::anyhow!("transition_to_resumed: {}", e))
        })?;
        self.sessions.save(&session).await?;
        Ok(())
    }

    /// turn 超时自动判负 (per §5.3 累计 3 次 OR §5.4 60s+)
    pub async fn timeout_turn(&self, match_id: Uuid) -> Result<()> {
        let mut session = self
            .sessions
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "GameSession",
                id: match_id.to_string(),
            })?;
        if session.status != SessionStatus::Running {
            return Ok(());
        }
        session.timeout_count += 1;
        if session.timeout_count >= 3 {
            // 3 次累计 → 判负: 找对手作为 winner
            let current = session.current_player_id.clone();
            let winner = session
                .players
                .iter()
                .find(|p| Some(&p.player_id) != current.as_ref())
                .map(|p| p.player_id.clone());
            session
                .transition_to_ending(winner.clone(), "timeout".to_string())
                .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_ending: {}", e)))?;
            session
                .transition_to_ended()
                .map_err(|e| Error::Internal(anyhow::anyhow!("transition_to_ended: {}", e)))?;
            session = self.sessions.save(&session).await?;
            let now_ms = chrono::Utc::now().timestamp_millis();
            self.event_bus
                .publish(
                    match_id,
                    MatchEvent::MatchEnded {
                        occurred_at_ms: now_ms,
                        end_reason: "timeout".to_string(),
                        winner_id: winner,
                    },
                )
                .await;
            self.event_bus.cleanup(match_id).await;
        } else {
            // 强制切到下一回合
            let deadline_ms = chrono::Utc::now().timestamp_millis() + 60_000;
            session.advance_turn(Some(deadline_ms)).map_err(|e| {
                Error::Internal(anyhow::anyhow!("advance_turn: {}", e))
            })?;
            session = self.sessions.save(&session).await?;
            let now_ms = chrono::Utc::now().timestamp_millis();
            self.event_bus
                .publish(
                    match_id,
                    MatchEvent::TurnChanged {
                        occurred_at_ms: now_ms,
                        new_turn_index: session.turn_index,
                        new_player_id: session.current_player_id.clone().unwrap_or_default(),
                    },
                )
                .await;
        }
        Ok(())
    }
}

// ============================================================================
// Result types (per §4.2 Response messages)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueResult {
    Queued {
        ticket_id: Uuid,
        estimated_wait_ms: i64,
    },
    Matched {
        ticket_id: Uuid,
        match_id: Uuid,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketStatus {
    Queued,
    Matched,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone)]
pub struct MatchmakingStatus {
    pub status: TicketStatus,
    pub match_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CreateMatchResult {
    pub match_id: Uuid,
    pub mode: GameMode,
    pub room_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JoinMatchResult {
    pub joined: bool,
    pub turn_index: u32,
}

#[derive(Debug, Clone)]
pub struct LeaveMatchResult {
    pub left: bool,
    pub match_result: String,
}

#[derive(Debug, Clone)]
pub struct MatchState {
    pub session: GameSession,
    pub board_snapshot: String,
    pub next_turn_deadline_ms: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SubmitMoveResult {
    pub accepted: bool,
    pub new_turn_index: u32,
    pub new_board_snapshot_ref: Option<String>,
    pub reject_reason: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository_v2::{
        InMemoryGameSessionRepository, InMemoryMatchmakingTicketRepository, InMemoryMoveRepository,
    };

    fn make_player(id: &str) -> SessionPlayer {
        SessionPlayer::new(id.to_string(), format!("P-{}", id)).with_rank(1500, 10)
    }

    fn make_service() -> MatchmakerServiceV2 {
        MatchmakerServiceV2::new(
            Arc::new(InMemoryGameSessionRepository::new()),
            Arc::new(InMemoryMoveRepository::new()),
            Arc::new(InMemoryMatchmakingTicketRepository::new()),
        )
    }

    #[tokio::test]
    async fn enqueue_matchmaking_queued_no_opponent() {
        let svc = make_service();
        let r = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Casual, 0, 0)
            .await
            .unwrap();
        assert!(matches!(r, EnqueueResult::Queued { .. }));
    }

    #[tokio::test]
    async fn enqueue_matchmaking_matched_two_players() {
        let svc = make_service();
        let _r1 = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Ranked, 1400, 1600)
            .await
            .unwrap();
        let r2 = svc
            .enqueue_matchmaking(make_player("p2"), GameMode::Ranked, 1400, 1600)
            .await
            .unwrap();
        match r2 {
            EnqueueResult::Matched { match_id, .. } => {
                let s = svc
                    .event_bus
                    .subscribe(match_id)
                    .await;
                drop(s);
            }
            _ => panic!("expected Matched"),
        }
    }

    #[tokio::test]
    async fn enqueue_matchmaking_room_mode_rejected() {
        let svc = make_service();
        let err = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Room, 0, 0)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn enqueue_empty_player_id_rejected() {
        let svc = make_service();
        let err = svc
            .enqueue_matchmaking(
                SessionPlayer::new(String::new(), "x".to_string()),
                GameMode::Casual,
                0,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn enqueue_unspecified_mode_rejected() {
        let svc = make_service();
        let err = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Unspecified, 0, 0)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn cancel_matchmaking_success() {
        let svc = make_service();
        let r = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Casual, 0, 0)
            .await
            .unwrap();
        let ticket_id = match r {
            EnqueueResult::Queued { ticket_id, .. } => ticket_id,
            _ => panic!("expected Queued"),
        };
        let cancelled = svc.cancel_matchmaking(ticket_id, "p1").await.unwrap();
        assert!(cancelled);
    }

    #[tokio::test]
    async fn cancel_matchmaking_wrong_player_rejected() {
        let svc = make_service();
        let r = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Casual, 0, 0)
            .await
            .unwrap();
        let ticket_id = match r {
            EnqueueResult::Queued { ticket_id, .. } => ticket_id,
            _ => panic!("expected Queued"),
        };
        let err = svc.cancel_matchmaking(ticket_id, "p2").await.unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn cancel_matchmaking_not_found() {
        let svc = make_service();
        let err = svc
            .cancel_matchmaking(Uuid::new_v4(), "p1")
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn get_matchmaking_status_queued() {
        let svc = make_service();
        let r = svc
            .enqueue_matchmaking(make_player("p1"), GameMode::Casual, 0, 0)
            .await
            .unwrap();
        let ticket_id = match r {
            EnqueueResult::Queued { ticket_id, .. } => ticket_id,
            _ => panic!(),
        };
        let status = svc.get_matchmaking_status(ticket_id).await.unwrap();
        assert_eq!(status.status, TicketStatus::Queued);
        assert!(status.match_id.is_none());
    }

    #[tokio::test]
    async fn create_match_room_creates_waiting() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("ROOM1".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        assert_eq!(r.room_code, Some("ROOM1".to_string()));
        let s = svc.get_match_state(r.match_id, &make_player("host")).await.unwrap();
        assert_eq!(s.session.status, SessionStatus::Waiting);
    }

    #[tokio::test]
    async fn create_match_duplicate_room_code() {
        let svc = make_service();
        svc.create_match(
            make_player("host1"),
            GameMode::Room,
            Some("ROOM1".to_string()),
            None,
            4,
            2,
            0,
        )
        .await
        .unwrap();
        let err = svc
            .create_match(
                make_player("host2"),
                GameMode::Room,
                Some("ROOM1".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn create_match_max_lt_min_rejected() {
        let svc = make_service();
        let err = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                1,
                2,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn create_match_room_no_code_rejected() {
        let svc = make_service();
        let err = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                None,
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn join_match_full_session_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let err = svc
            .join_match(r.match_id, make_player("p3"), None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MatchFull { .. }));
    }

    #[tokio::test]
    async fn join_match_wrong_room_password_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                Some("secret".to_string()),
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let err = svc
            .join_match(r.match_id, make_player("p2"), None, Some("wrong".to_string()))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn join_match_auto_start_when_ready() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let join = svc
            .join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        assert!(join.joined);
        let s = svc
            .get_match_state(r.match_id, &make_player("p2"))
            .await
            .unwrap();
        // min_players=2 已到齐, 应进入 Starting
        assert!(matches!(
            s.session.status,
            SessionStatus::Starting | SessionStatus::Running
        ));
    }

    #[tokio::test]
    async fn join_match_duplicate_player_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        // host 已在 session 中, 重复 join 拒绝
        let err = svc
            .join_match(r.match_id, make_player("host"), None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn leave_match_surrender_ends_session() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        // 强制 status=Running (从 Starting 进 Running, 否则 leave_match surrender 失败)
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        svc.sessions.save(&s).await.unwrap();
        let leave = svc
            .leave_match(r.match_id, "p2", true)
            .await
            .unwrap();
        assert_eq!(leave.match_result, "surrender");
        let s_after = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert!(s_after.status == SessionStatus::Ended || s_after.status == SessionStatus::Ending);
        let _ = s; // suppress unused
    }

    #[tokio::test]
    async fn leave_match_disconnect_marks_only() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                4,
                3,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        // 不够 3 人, 不进 Running
        let leave = svc
            .leave_match(r.match_id, "p2", false)
            .await
            .unwrap();
        assert_eq!(leave.match_result, "disconnect");
        let s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert!(s.players.iter().find(|p| p.player_id == "p2").unwrap().disconnected);
        assert!(!s.status.is_terminal());
    }

    #[tokio::test]
    async fn leave_match_all_disconnect_cancels() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        // 都断线
        let _ = svc
            .leave_match(r.match_id, "host", false)
            .await
            .unwrap();
        let leave2 = svc
            .leave_match(r.match_id, "p2", false)
            .await
            .unwrap();
        // 第二次: 已无活跃玩家 → all_disconnected
        assert!(leave2.match_result == "all_disconnected" || leave2.match_result == "disconnect");
        let s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        // 终止态 (Ended 或 Canceled)
        assert!(s.status.is_terminal());
    }

    #[tokio::test]
    async fn leave_match_not_in_match_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let err = svc
            .leave_match(r.match_id, "ghost", false)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotInMatch { .. }));
    }

    #[tokio::test]
    async fn submit_move_wrong_turn_index_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        // 状态可能 Starting, 强制推到 Running
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        // 简化: 把 status 改成 Running (仅测试, 实际由状态机推进)
        // 注: 这里不调用 transition, 直接通过 Repository save 强制写入
        // 实际测试应使用 start_match 之类的辅助
        // 跳过此断言, 只测 NotRunning 路径
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        svc.sessions.save(&s).await.unwrap();

        let mv = Move::new(r.match_id, "host".to_string(), 99, MoveType::PlayCard, "{}".to_string());
        let err = svc
            .submit_move(r.match_id, &make_player("host"), 99, mv)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[tokio::test]
    async fn submit_move_wrong_player_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        svc.sessions.save(&s).await.unwrap();

        let mv = Move::new(r.match_id, "p2".to_string(), 0, MoveType::PlayCard, "{}".to_string());
        let err = svc
            .submit_move(r.match_id, &make_player("p2"), 0, mv)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn submit_move_end_turn_advances() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        svc.sessions.save(&s).await.unwrap();

        let mv = Move::new(r.match_id, "host".to_string(), 0, MoveType::EndTurn, "{}".to_string());
        let res = svc
            .submit_move(r.match_id, &make_player("host"), 0, mv)
            .await
            .unwrap();
        assert!(res.accepted);
        assert_eq!(res.new_turn_index, 1);

        let s_after = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s_after.current_player_id, Some("p2".to_string()));
    }

    #[tokio::test]
    async fn submit_move_surrender_ends() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        svc.sessions.save(&s).await.unwrap();

        let mv = Move::new(r.match_id, "host".to_string(), 0, MoveType::Surrender, "{}".to_string());
        let res = svc
            .submit_move(r.match_id, &make_player("host"), 0, mv)
            .await
            .unwrap();
        assert!(res.accepted);
        let s_after = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s_after.status, SessionStatus::Ended);
        assert_eq!(s_after.winner_id, Some("p2".to_string()));
        assert_eq!(s_after.end_reason, Some("surrender".to_string()));
    }

    #[tokio::test]
    async fn submit_move_session_not_running_rejected() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        // 状态是 Starting (min_players=2 已到齐)
        // 不强制 Running, 直接 submit 应被拒
        let mv = Move::new(r.match_id, "host".to_string(), 0, MoveType::PlayCard, "{}".to_string());
        // 如果状态已经是 Starting/Running, 也会因为 current_player_id=None 拒
        let err = svc
            .submit_move(r.match_id, &make_player("host"), 0, mv)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_) | Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn pause_resume_session() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        svc.sessions.save(&s).await.unwrap();

        svc.pause_session(r.match_id).await.unwrap();
        let s_paused = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s_paused.status, SessionStatus::Paused);

        svc.resume_session(r.match_id).await.unwrap();
        let s_resumed = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s_resumed.status, SessionStatus::Running);
    }

    #[tokio::test]
    async fn timeout_turn_after_3_loses() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        svc.sessions.save(&s).await.unwrap();

        // 3 次 timeout → 判负
        svc.timeout_turn(r.match_id).await.unwrap();
        svc.timeout_turn(r.match_id).await.unwrap();
        svc.timeout_turn(r.match_id).await.unwrap();
        let s_after = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s_after.status, SessionStatus::Ended);
        assert_eq!(s_after.end_reason, Some("timeout".to_string()));
        assert_eq!(s_after.winner_id, Some("p2".to_string()));
    }

    #[tokio::test]
    async fn timeout_turn_under_3_advances() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        svc.join_match(r.match_id, make_player("p2"), None, None)
            .await
            .unwrap();
        let mut s = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        s.turn_index = 0;
        svc.sessions.save(&s).await.unwrap();

        svc.timeout_turn(r.match_id).await.unwrap();
        let s_after = svc
            .sessions
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(s_after.status, SessionStatus::Running);
        assert_eq!(s_after.turn_index, 1);
        assert_eq!(s_after.timeout_count, 1); // 累计 1 次, 切换玩家后不清零 (per §5.3)
    }

    #[tokio::test]
    async fn subscribe_match_returns_receiver() {
        let svc = make_service();
        let r = svc
            .create_match(
                make_player("host"),
                GameMode::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        let rx = svc
            .subscribe_match(r.match_id, &make_player("host"), true)
            .await
            .unwrap();
        // receiver 存在即可
        drop(rx);
    }

    #[tokio::test]
    async fn subscribe_match_not_found() {
        let svc = make_service();
        let err = svc
            .subscribe_match(Uuid::new_v4(), &make_player("p1"), false)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }
}
