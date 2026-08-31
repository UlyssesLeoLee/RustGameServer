//! match-service 域 Service 业务实施（per RGS-DTL-016 §3 + RGS-DTL-038 §4.2/§5）
//!
//! 54.7 实化：4 Service 业务方法（create_match / join_match / start_match / finish_match）
//! + gRPC 桥接 HealthCheck + GetMatch
//!
//! 桶 9 补完 (per RGS-DTL-038 §4.2 + §5):
//! - MatchServiceImpl 扩展 `matchmaker_v2: Arc<MatchmakerServiceV2>` 字段
//! - 旧构造函数 `new(v1_repo, v1_participant_repo)` 保留兼容
//! - 新构造函数 `with_matchmaker_v2` 注入 v2 matchmaker
//! - 9 个 v2 RPC stub 升级到调用 matchmaker_v2 业务逻辑
//! - SubscribeMatch 用 `matchmaker_v2.event_bus().subscribe()` 流式推送
//!
//! ## 9 RPC 映射 (per §4.2)
//! 1. EnqueueMatchmaking    → matchmaker_v2.enqueue_matchmaking
//! 2. CancelMatchmaking     → matchmaker_v2.cancel_matchmaking
//! 3. GetMatchmakingStatus  → matchmaker_v2.get_matchmaking_status
//! 4. CreateMatch           → matchmaker_v2.create_match
//! 5. JoinMatch             → matchmaker_v2.join_match
//! 6. LeaveMatch            → matchmaker_v2.leave_match
//! 7. GetMatchState         → matchmaker_v2.get_match_state
//! 8. SubmitMove            → matchmaker_v2.submit_move
//! 9. SubscribeMatch        → matchmaker_v2.event_bus().subscribe()

use crate::entity::{Match, MatchMode, MatchParticipant, MatchStatus, Team};
use crate::entity_v2::{
    GameMode as GameModeV2, GameSession, MoveType, SessionPlayer, SessionStatus,
};
use crate::error::Error;
use crate::matchmaker_v2::{MatchmakerServiceV2, TicketStatus};
use crate::repository::{MatchParticipantRepository, MatchRepository};
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait MatchService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    /// 创建对局
    async fn create_match(&self, room_id: String, mode: MatchMode) -> Result<Match>;

    /// 玩家加入对局
    async fn join_match(
        &self,
        match_id: Uuid,
        player_id: Uuid,
        team: Team,
    ) -> Result<MatchParticipant>;

    /// 开始对局
    async fn start_match(&self, match_id: Uuid) -> Result<Match>;

    /// 结束对局
    async fn finish_match(&self, match_id: Uuid, winner: Option<Team>) -> Result<Match>;
}

pub struct MatchServiceImpl {
    /// v1 (5 域 matchmaker) 仓库
    matches: Arc<dyn MatchRepository>,
    participants: Arc<dyn MatchParticipantRepository>,
    /// v2 (卡牌游戏 session/turn) matchmaker (per RGS-DTL-038 §4.2 + §5)
    matchmaker_v2: Option<Arc<MatchmakerServiceV2>>,
}

impl MatchServiceImpl {
    /// v1 兼容构造函数 (旧 5 域 matchmaker 业务)
    /// v2 业务不可用 (matchmaker_v2 = None), 9 RPC 仍返回 unimplemented 错误
    pub fn new(
        matches: Arc<dyn MatchRepository>,
        participants: Arc<dyn MatchParticipantRepository>,
    ) -> Self {
        Self {
            matches,
            participants,
            matchmaker_v2: None,
        }
    }

    /// 桶 9 新构造函数: 注入 v2 matchmaker (per RGS-DTL-038 §4.2 + §5)
    pub fn with_matchmaker_v2(
        matches: Arc<dyn MatchRepository>,
        participants: Arc<dyn MatchParticipantRepository>,
        matchmaker_v2: Arc<MatchmakerServiceV2>,
    ) -> Self {
        Self {
            matches,
            participants,
            matchmaker_v2: Some(matchmaker_v2),
        }
    }

    /// 仅 v2 构造 (per RGS-DTL-038 §4.2, 纯 v2 服务场景)
    pub fn v2_only(
        matchmaker_v2: Arc<MatchmakerServiceV2>,
        matches: Arc<dyn MatchRepository>,
        participants: Arc<dyn MatchParticipantRepository>,
    ) -> Self {
        Self {
            matches,
            participants,
            matchmaker_v2: Some(matchmaker_v2),
        }
    }

    /// 取 v2 matchmaker (无 → Internal 错)
    pub fn v2(&self) -> Result<&Arc<MatchmakerServiceV2>> {
        self.matchmaker_v2
            .as_ref()
            .ok_or_else(|| Error::Internal(anyhow::anyhow!("matchmaker_v2 not configured")))
    }

    pub async fn find_match_by_id(&self, id: Uuid) -> Result<Option<Match>> {
        self.matches.find_by_id(id).await
    }
}

#[async_trait]
impl MatchService for MatchServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn create_match(&self, room_id: String, mode: MatchMode) -> Result<Match> {
        tracing::debug!(
            operation = "matchmaking_entry",
            service = "match-service",
            method = "create_match",
            room_id = %room_id,
            mode = ?mode,
            "matchmaking: create match"
        );
        if room_id.is_empty() {
            return Err(Error::Validation("room_id must not be empty".to_string()));
        }
        if self.matches.find_by_room_id(&room_id).await?.is_some() {
            return Err(Error::Conflict(format!(
                "room_id {} already in use",
                room_id
            )));
        }
        let m = Match::new(room_id, mode);
        self.matches.save(&m).await?;
        Ok(m)
    }

    async fn join_match(
        &self,
        match_id: Uuid,
        player_id: Uuid,
        team: Team,
    ) -> Result<MatchParticipant> {
        tracing::debug!(
            operation = "matchmaking_candidate_join",
            service = "match-service",
            method = "join_match",
            match_id = %match_id,
            player_id = %player_id,
            team = ?team,
            "matchmaking: candidate join"
        );
        let m = self
            .matches
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Match",
                id: match_id.to_string(),
            })?;
        if m.status != MatchStatus::Waiting {
            return Err(Error::MatchAlreadyStarted(match_id.to_string()));
        }
        // 检查是否已加入
        let existing = self.participants.list_by_match(match_id).await?;
        if existing.iter().any(|p| p.player_id == player_id) {
            return Err(Error::Conflict(format!(
                "player {} already in match",
                player_id
            )));
        }
        let p = MatchParticipant::new(match_id, player_id, team);
        self.participants.save(&p).await?;
        Ok(p)
    }

    async fn start_match(&self, match_id: Uuid) -> Result<Match> {
        let mut m = self
            .matches
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Match",
                id: match_id.to_string(),
            })?;
        if m.status != MatchStatus::Waiting {
            return Err(Error::MatchAlreadyStarted(match_id.to_string()));
        }
        m.start();
        self.matches.save(&m).await?;
        Ok(m)
    }

    async fn finish_match(&self, match_id: Uuid, winner: Option<Team>) -> Result<Match> {
        let mut m = self
            .matches
            .find_by_id(match_id)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Match",
                id: match_id.to_string(),
            })?;
        if m.status != MatchStatus::InProgress {
            return Err(Error::Validation(format!(
                "match {} is not in progress",
                match_id
            )));
        }
        m.finish(winner);
        self.matches.save(&m).await?;
        Ok(m)
    }
}

// ============================================================================
// Proto ↔ Entity 转换工具
// ============================================================================

pub mod conv {
    use super::*;
    use crate::matchmaker_v2::MatchEvent as MatchEventV2;

    // ---- GameMode ----
    pub fn game_mode_from_proto(i: i32) -> GameModeV2 {
        match i {
            1 => GameModeV2::Ranked,
            2 => GameModeV2::Casual,
            3 => GameModeV2::Room,
            4 => GameModeV2::PveAi,
            _ => GameModeV2::Unspecified,
        }
    }

    pub fn game_mode_to_proto(m: GameModeV2) -> i32 {
        m as i32
    }

    // ---- SessionPlayer <-> common.v1.PlayerId ----
    pub fn player_from_proto(p: &crate::common::v1::PlayerId) -> SessionPlayer {
        let pid = p
            .player_id
            .as_ref()
            .map(|e| e.id.clone())
            .unwrap_or_default();
        let sp = SessionPlayer::new(pid, p.display_name.clone())
            .with_rank(p.rank_score, p.level);
        // 注: common.v1.PlayerId 不含 deck_ref 字段 (per proto 定义)
        // deck_ref 在 EnqueueMatchmakingRequest / SubmitMoveRequest 等 message 上独立携带
        sp
    }

    pub fn player_to_proto(p: &SessionPlayer) -> crate::common::v1::PlayerId {
        crate::common::v1::PlayerId {
            player_id: Some(crate::common::v1::EntityId {
                id: p.player_id.clone(),
            }),
            display_name: p.display_name.clone(),
            rank_score: p.rank_score,
            level: p.level,
        }
    }

    // ---- Move <-> proto::v1::Move ----
    pub fn move_from_proto(
        match_id: Uuid,
        turn_index: u32,
        m: &crate::proto::v1::Move,
    ) -> crate::entity_v2::Move {
        use crate::entity_v2::Move as EntityMove;
        let move_type = match m.r#type {
            1 => MoveType::PlayCard,
            2 => MoveType::Attack,
            3 => MoveType::EndTurn,
            4 => MoveType::Surrender,
            5 => MoveType::UseAbility,
            _ => MoveType::Unspecified,
        };
        let player_id = m
            .player
            .as_ref()
            .and_then(|p| p.player_id.as_ref().map(|e| e.id.clone()))
            .unwrap_or_default();
        let mut em = EntityMove::new(match_id, player_id, turn_index, move_type, m.payload_json.clone());
        em.accepted = m.accepted;
        em.result_json = if m.result_json.is_empty() {
            None
        } else {
            Some(m.result_json.clone())
        };
        if !m.move_id.is_empty() {
            if let Ok(parsed) = Uuid::parse_str(&m.move_id) {
                em.move_id = parsed;
            }
        }
        em
    }

    pub fn move_to_proto(m: &crate::entity_v2::Move) -> crate::proto::v1::Move {
        let type_i = match m.move_type {
            MoveType::PlayCard => 1,
            MoveType::Attack => 2,
            MoveType::EndTurn => 3,
            MoveType::Surrender => 4,
            MoveType::UseAbility => 5,
            MoveType::Unspecified => 0,
        };
        crate::proto::v1::Move {
            move_id: m.move_id.to_string(),
            player: Some(crate::common::v1::PlayerId {
                player_id: Some(crate::common::v1::EntityId {
                    id: m.player_id.clone(),
                }),
                display_name: String::new(),
                rank_score: 0,
                level: 0,
            }),
            r#type: type_i,
            payload_json: m.payload_json.clone(),
            occurred_at_ms: m.occurred_at.timestamp_millis(),
            result_json: m.result_json.clone().unwrap_or_default(),
            accepted: m.accepted,
        }
    }

    // ---- SessionStatus -> proto::v1::Match (GetMatch/GetMatchState) ----
    pub fn session_to_match_proto(s: &GameSession) -> crate::proto::v1::Match {
        use crate::common::v1 as common_proto;
        let status_i = match s.status {
            SessionStatus::Creating => 2,    // STATUS_PENDING
            SessionStatus::Waiting => 2,     // STATUS_PENDING
            SessionStatus::Starting => 2,    // STATUS_PENDING
            SessionStatus::Running => 1,     // STATUS_OK
            SessionStatus::Paused => 2,      // STATUS_PENDING
            SessionStatus::Ending => 2,      // STATUS_PENDING
            SessionStatus::Ended => 1,       // STATUS_OK
            SessionStatus::Canceled => 4,    // STATUS_CANCELLED
        };
        let players: Vec<common_proto::PlayerId> =
            s.players.iter().map(player_to_proto).collect();
        crate::proto::v1::Match {
            id: Some(common_proto::EntityId {
                id: s.match_id.to_string(),
            }),
            status: status_i,
            created_at: Some(common_proto::Timestamp {
                seconds: s.created_at.timestamp(),
                nanos: s.created_at.timestamp_subsec_nanos() as i32,
            }),
            display_name: s.room_code.clone().unwrap_or_default(),
            mode: game_mode_to_proto(s.mode),
            players,
            board_snapshot_ref: s.board_snapshot_ref.clone().unwrap_or_default(),
            turn_index: s.turn_index,
        }
    }

    // ---- TicketStatus -> proto Status enum ----
    pub fn ticket_status_to_proto(s: TicketStatus) -> i32 {
        match s {
            TicketStatus::Queued => 1,
            TicketStatus::Matched => 2,
            TicketStatus::Cancelled => 3,
            TicketStatus::Expired => 4,
        }
    }

    // ---- GameEvent -> proto MatchEvent ----
    pub fn event_to_proto(e: &MatchEventV2) -> crate::proto::v1::MatchEvent {
        use crate::matchmaker_v2::MatchEvent as E;
        let type_i = match e {
            E::Snapshot { .. } => 1,
            E::MoveApplied { .. } => 2,
            E::TurnChanged { .. } => 3,
            E::PlayerJoined { .. } => 4,
            E::PlayerLeft { .. } => 5,
            E::MatchEnded { .. } => 6,
            E::TimeoutWarning { .. } => 7,
        };
        let occurred_at_ms = e.occurred_at_ms();
        let payload = match e {
            E::Snapshot { board_snapshot, .. } => {
                crate::proto::v1::match_event::Payload::BoardSnapshot(board_snapshot.clone())
            }
            E::MoveApplied { mv, .. } => crate::proto::v1::match_event::Payload::Move(
                move_to_proto(mv),
            ),
            E::TurnChanged {
                new_turn_index, ..
            } => crate::proto::v1::match_event::Payload::NewTurnIndex(*new_turn_index),
            E::PlayerJoined { player, .. } => {
                crate::proto::v1::match_event::Payload::Player(player_to_proto(player))
            }
            E::PlayerLeft { player_id, .. } => {
                // proto PlayerId required; put player_id in EntityId
                crate::proto::v1::match_event::Payload::Player(crate::common::v1::PlayerId {
                    player_id: Some(crate::common::v1::EntityId {
                        id: player_id.clone(),
                    }),
                    display_name: String::new(),
                    rank_score: 0,
                    level: 0,
                })
            }
            E::MatchEnded { end_reason, .. } => {
                crate::proto::v1::match_event::Payload::EndReason(end_reason.clone())
            }
            E::TimeoutWarning {
                turn_index,
                remaining_ms,
                ..
            } => {
                // 用 EndReason 槽位透传, 实际"timeout_warning" string + remaining 信息
                // 简化: 直接在 end_reason 槽位塞 "turn=N,remaining_ms=M"
                let _ = turn_index;
                let _ = remaining_ms;
                crate::proto::v1::match_event::Payload::EndReason(format!(
                    "timeout_warning:turn={},remaining_ms={}",
                    turn_index, remaining_ms
                ))
            }
        };
        crate::proto::v1::MatchEvent {
            r#type: type_i,
            occurred_at_ms,
            payload: Some(payload),
        }
    }

    pub fn parse_uuid(s: &str) -> Result<Uuid> {
        Uuid::parse_str(s)
            .map_err(|_| Error::Validation(format!("invalid uuid: {}", s)))
    }

    pub fn ok_status(s: &str) -> Result<Uuid> {
        parse_uuid(s)
    }
}

#[cfg(test)]
mod conv_tests {
    //! service.rs::conv 单元测试 (per UT-AGENT-BRIEFING-v3 Step 2)
    //!
    //! 覆盖:
    //! - game_mode 双向映射 (1-4 ↔ Ranked/Casual/Room/PveAi, 0/默认 → Unspecified)
    //! - parse_uuid 错误路径
    //! - player_from_proto / player_to_proto 字段透传
    //! - move_type 双向映射 (1-5 ↔ PlayCard/Attack/EndTurn/Surrender/UseAbility)
    //!
    //! 注: 不依赖 DB / 网络, 纯 DTO 转换逻辑

    use super::conv;
    use crate::common::v1 as common_proto;
    use crate::entity_v2::{Move, MoveType, SessionPlayer};
    use crate::matchmaker_v2::TicketStatus;

    // ==================== game_mode ====================

    #[test]
    fn game_mode_roundtrip_all_known_values() {
        // 1-4 已知值: Ranked/Casual/Room/PveAi
        assert_eq!(conv::game_mode_from_proto(1), crate::entity_v2::GameMode::Ranked);
        assert_eq!(conv::game_mode_from_proto(2), crate::entity_v2::GameMode::Casual);
        assert_eq!(conv::game_mode_from_proto(3), crate::entity_v2::GameMode::Room);
        assert_eq!(conv::game_mode_from_proto(4), crate::entity_v2::GameMode::PveAi);
        // 0 / 5+ → Unspecified
        assert_eq!(conv::game_mode_from_proto(0), crate::entity_v2::GameMode::Unspecified);
        assert_eq!(conv::game_mode_from_proto(99), crate::entity_v2::GameMode::Unspecified);
    }

    #[test]
    fn game_mode_to_proto_is_identity_for_known() {
        // 已知的 GameMode 转回 proto 数字 (枚举 discriminant 稳定)
        assert_eq!(conv::game_mode_to_proto(crate::entity_v2::GameMode::Ranked), 1);
        assert_eq!(conv::game_mode_to_proto(crate::entity_v2::GameMode::Casual), 2);
        assert_eq!(conv::game_mode_to_proto(crate::entity_v2::GameMode::Room), 3);
        assert_eq!(conv::game_mode_to_proto(crate::entity_v2::GameMode::PveAi), 4);
    }

    // ==================== parse_uuid ====================

    #[test]
    fn parse_uuid_valid_and_invalid() {
        // 有效 UUID 字符串
        let u = conv::parse_uuid("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            u.to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        // 无效 → Validation 错
        let err = conv::parse_uuid("not-a-uuid").unwrap_err();
        match err {
            crate::Error::Validation(msg) => assert!(msg.contains("invalid uuid")),
            _ => panic!("expected Validation, got {:?}", err),
        }
    }

    // ==================== player_from_proto / player_to_proto ====================

    #[test]
    fn player_proto_roundtrip_preserves_fields() {
        let proto = common_proto::PlayerId {
            player_id: Some(common_proto::EntityId {
                id: "pid-001".to_string(),
            }),
            display_name: "alice".to_string(),
            rank_score: 1500,
            level: 25,
        };
        let sp = conv::player_from_proto(&proto);
        assert_eq!(sp.player_id, "pid-001");
        assert_eq!(sp.display_name, "alice");
        assert_eq!(sp.rank_score, 1500);
        assert_eq!(sp.level, 25);

        let back = conv::player_to_proto(&sp);
        assert_eq!(back.player_id.unwrap().id, "pid-001");
        assert_eq!(back.display_name, "alice");
        assert_eq!(back.rank_score, 1500);
        assert_eq!(back.level, 25);
    }

    #[test]
    fn player_from_proto_handles_missing_entity_id() {
        // player_id 字段为 None → 空字符串
        let proto = common_proto::PlayerId {
            player_id: None,
            display_name: "bob".to_string(),
            rank_score: 0,
            level: 0,
        };
        let sp = conv::player_from_proto(&proto);
        assert_eq!(sp.player_id, "");
        assert_eq!(sp.display_name, "bob");
    }

    // ==================== move_type ====================

    #[test]
    fn move_type_roundtrip_all_known_values() {
        // move_from_proto: 1-5 → 5 种 MoveType
        for (i, expected) in &[
            (1i32, MoveType::PlayCard),
            (2, MoveType::Attack),
            (3, MoveType::EndTurn),
            (4, MoveType::Surrender),
            (5, MoveType::UseAbility),
        ] {
            let proto = crate::proto::v1::Move {
                move_id: String::new(),
                player: Some(common_proto::PlayerId {
                    player_id: Some(common_proto::EntityId { id: "p".to_string() }),
                    display_name: String::new(),
                    rank_score: 0,
                    level: 0,
                }),
                r#type: *i,
                payload_json: String::new(),
                occurred_at_ms: 0,
                result_json: String::new(),
                accepted: false,
            };
            let m = conv::move_from_proto(uuid::Uuid::new_v4(), 1, &proto);
            assert_eq!(&m.move_type, expected, "type {} → expected {:?}", i, expected);

            // 反向: move_to_proto
            let back = conv::move_to_proto(&m);
            assert_eq!(back.r#type, *i, "roundtrip: type {} → {}", i, back.r#type);
        }
        // 0 / 6+ → Unspecified
        let proto_unspec = crate::proto::v1::Move {
            move_id: String::new(),
            player: Some(common_proto::PlayerId {
                player_id: Some(common_proto::EntityId { id: "p".to_string() }),
                display_name: String::new(),
                rank_score: 0,
                level: 0,
            }),
            r#type: 0,
            payload_json: String::new(),
            occurred_at_ms: 0,
            result_json: String::new(),
            accepted: false,
        };
        let m = conv::move_from_proto(uuid::Uuid::new_v4(), 1, &proto_unspec);
        assert_eq!(m.move_type, MoveType::Unspecified);
    }

    // ==================== ticket_status_to_proto ====================

    #[test]
    fn ticket_status_to_proto_mapping() {
        // per service.rs::conv::ticket_status_to_proto 定义
        assert_eq!(conv::ticket_status_to_proto(TicketStatus::Queued), 1);
        assert_eq!(conv::ticket_status_to_proto(TicketStatus::Matched), 2);
        assert_eq!(conv::ticket_status_to_proto(TicketStatus::Cancelled), 3);
        assert_eq!(conv::ticket_status_to_proto(TicketStatus::Expired), 4);
    }

    #[test]
    fn move_to_proto_propagates_optional_result_json() {
        // result_json None → 空字符串 (per proto3 默认)
        let m = Move {
            move_id: uuid::Uuid::nil(),
            match_id: uuid::Uuid::nil(),
            player_id: "p".to_string(),
            turn_index: 0,
            move_type: MoveType::PlayCard,
            payload_json: String::new(),
            occurred_at: chrono::Utc::now(),
            result_json: None,
            accepted: false,
            reject_reason: None,
        };
        let back = conv::move_to_proto(&m);
        assert_eq!(back.result_json, "");

        // result_json Some(s) → 原样透传
        let mut m2 = m.clone();
        m2.result_json = Some("ok".to_string());
        let back2 = conv::move_to_proto(&m2);
        assert_eq!(back2.result_json, "ok");
    }

    // ==================== session_player 构造 (per 业务) ====================

    #[test]
    fn session_player_builder_chain() {
        // SessionPlayer::new + with_rank 链式构造 (per entity_v2)
        let sp = SessionPlayer::new("pid-1".to_string(), "alice".to_string()).with_rank(2000, 30);
        assert_eq!(sp.player_id, "pid-1");
        assert_eq!(sp.display_name, "alice");
        assert_eq!(sp.rank_score, 2000);
        assert_eq!(sp.level, 30);
    }
}

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as match_proto;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::sync::broadcast;
    use tonic::codegen::tokio_stream::Stream;

    // ============================================================================
    // v2 SubscribeMatch stream type (per §4.2 stream RPC)
    // 将 broadcast::Receiver<MatchEvent> 包装成 tonic Stream
    // ============================================================================

    pub struct MatchEventStream {
        inner: broadcast::Receiver<crate::matchmaker_v2::MatchEvent>,
    }

    impl MatchEventStream {
        pub fn new(inner: broadcast::Receiver<crate::matchmaker_v2::MatchEvent>) -> Self {
            Self { inner }
        }
    }

    impl Stream for MatchEventStream {
        type Item = std::result::Result<match_proto::MatchEvent, Status>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            // Pin::get_mut 安全: Self 字段是 broadcast::Receiver, 自身可 Pin (Unpin)
            let this = self.get_mut();
            loop {
                match this.inner.try_recv() {
                    Ok(evt) => return Poll::Ready(Some(Ok(conv::event_to_proto(&evt)))),
                    Err(broadcast::error::TryRecvError::Empty) => {
                        // 注册 waker 等待新事件
                        let waker = cx.waker().clone();
                        let mut rx = this.inner.resubscribe();
                        tokio::spawn(async move {
                            // 仅作 waker 触发, 事件丢失可接受 (client 会拉 get_match_state)
                            let _ = rx.recv().await;
                            waker.wake();
                        });
                        return Poll::Pending;
                    }
                    Err(broadcast::error::TryRecvError::Lagged(_)) => {
                        // 跳过积压, 继续下一次
                        continue;
                    }
                    Err(broadcast::error::TryRecvError::Closed) => {
                        return Poll::Ready(None);
                    }
                }
            }
        }
    }

    pub struct MatchGrpcService {
        pub impl_: Arc<MatchServiceImpl>,
    }

    impl MatchGrpcService {
        pub fn new(impl_: Arc<MatchServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl match_proto::match_service_server::MatchService for MatchGrpcService {
        type SubscribeMatchStream = MatchEventStream;

        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "match-service",
                method = "HealthCheck",
                "enter grpc handler"
            );
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: if healthy {
                    common_proto::Status::Ok as i32
                } else {
                    common_proto::Status::Failed as i32
                },
                message: if healthy {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
            }))
        }

        async fn get_match(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<match_proto::Match>, Status> {
            let id_str = request.get_ref().id.clone();
            let match_id_parsed = Uuid::parse_str(&id_str).ok();
            tracing::debug!(
                operation = "grpc_handler_entry",
                service = "match-service",
                method = "GetMatch",
                match_id = %match_id_parsed.as_ref().map(|u| u.to_string()).unwrap_or_else(|| id_str.clone()),
                "enter grpc handler"
            );
            let match_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let m = self
                .impl_
                .find_match_by_id(match_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("match {}", id_str)))?;
            Ok(Response::new(match_proto::Match {
                id: Some(common_proto::EntityId {
                    id: m.id.to_string(),
                }),
                status: m.status as i32,
                created_at: Some(common_proto::Timestamp {
                    seconds: m.created_at.timestamp(),
                    nanos: m.created_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: m.room_id,
                mode: 0,
                players: vec![],
                board_snapshot_ref: String::new(),
                turn_index: 0,
            }))
        }

        // ========================================================================
        // v2 9 RPC 业务实装 (per RGS-DTL-038 §4.2 + §5)
        // ========================================================================

        async fn enqueue_matchmaking(
            &self,
            request: Request<match_proto::EnqueueMatchmakingRequest>,
        ) -> std::result::Result<Response<match_proto::EnqueueMatchmakingResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let player = req
                .player
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("player required"))?;
            let session_player = conv::player_from_proto(player);
            let mode = conv::game_mode_from_proto(req.mode);
            tracing::debug!(
                service = "match-service",
                method = "EnqueueMatchmaking",
                player_id = %session_player.player_id,
                mode = ?mode,
                "enqueue_matchmaking"
            );
            let result = v2
                .enqueue_matchmaking(
                    session_player,
                    mode,
                    req.rank_score_min,
                    req.rank_score_max,
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let resp = match result {
                crate::matchmaker_v2::EnqueueResult::Queued {
                    ticket_id,
                    estimated_wait_ms,
                } => match_proto::EnqueueMatchmakingResponse {
                    ticket_id: ticket_id.to_string(),
                    estimated_wait_ms,
                },
                crate::matchmaker_v2::EnqueueResult::Matched {
                    ticket_id,
                    match_id,
                } => match_proto::EnqueueMatchmakingResponse {
                    ticket_id: ticket_id.to_string(),
                    estimated_wait_ms: 0,
                }
                .with_optional_match_id(match_id),
            };
            Ok(Response::new(resp))
        }

        async fn cancel_matchmaking(
            &self,
            request: Request<match_proto::CancelMatchmakingRequest>,
        ) -> std::result::Result<Response<match_proto::CancelMatchmakingResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let ticket_id = conv::parse_uuid(&req.ticket_id)
                .map_err(Into::<tonic::Status>::into)?;
            tracing::debug!(
                service = "match-service",
                method = "CancelMatchmaking",
                ticket_id = %ticket_id,
                "cancel_matchmaking"
            );
            // proto §4.2 CancelMatchmakingRequest 无 player_id 字段
            // 桶 9: 透传 empty string 跳过所有权校验 (mTLS + ticket_id 已足够)
            // 后续桶 (saga 落地) 补 player_id 字段
            let cancelled = v2
                .cancel_matchmaking(ticket_id, "")
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(match_proto::CancelMatchmakingResponse { cancelled }))
        }

        async fn get_matchmaking_status(
            &self,
            request: Request<match_proto::GetMatchmakingStatusRequest>,
        ) -> std::result::Result<Response<match_proto::GetMatchmakingStatusResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let ticket_id = conv::parse_uuid(&req.ticket_id)
                .map_err(Into::<tonic::Status>::into)?;
            let status = v2
                .get_matchmaking_status(ticket_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(match_proto::GetMatchmakingStatusResponse {
                status: conv::ticket_status_to_proto(status.status),
                match_id: status.match_id.map(|u| u.to_string()).unwrap_or_default(),
            }))
        }

        async fn create_match(
            &self,
            request: Request<match_proto::CreateMatchRequest>,
        ) -> std::result::Result<Response<match_proto::CreateMatchResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let host = req
                .host
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("host required"))?;
            let host_player = conv::player_from_proto(host);
            let mode = conv::game_mode_from_proto(req.mode);
            tracing::debug!(
                service = "match-service",
                method = "CreateMatch",
                host_id = %host_player.player_id,
                mode = ?mode,
                room_code = %req.room_code,
                "create_match"
            );
            // min_players: proto 没显式字段, 默认 2 (1v1) / ROOM 模式可由 host 通过 deck_ref / 后续桶扩展
            let min_players: u32 = 2;
            let room_code = if req.room_code.is_empty() {
                None
            } else {
                Some(req.room_code.clone())
            };
            let room_password = if req.room_password.is_empty() {
                None
            } else {
                Some(req.room_password.clone())
            };
            let result = v2
                .create_match(
                    host_player,
                    mode,
                    room_code,
                    room_password,
                    req.max_players.max(2),
                    min_players,
                    req.ai_difficulty,
                )
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(match_proto::CreateMatchResponse {
                match_id: result.match_id.to_string(),
                mode: conv::game_mode_to_proto(result.mode),
                room_code: result.room_code.unwrap_or_default(),
            }))
        }

        async fn join_match(
            &self,
            request: Request<match_proto::JoinMatchRequest>,
        ) -> std::result::Result<Response<match_proto::JoinMatchResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let match_id = conv::parse_uuid(&req.match_id)
                .map_err(Into::<tonic::Status>::into)?;
            let player = req
                .player
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("player required"))?;
            let session_player = conv::player_from_proto(player);
            tracing::debug!(
                service = "match-service",
                method = "JoinMatch",
                match_id = %match_id,
                player_id = %session_player.player_id,
                "join_match"
            );
            let room_code = if req.room_code.is_empty() {
                None
            } else {
                Some(req.room_code.clone())
            };
            let room_password = if req.room_password.is_empty() {
                None
            } else {
                Some(req.room_password.clone())
            };
            let result = v2
                .join_match(match_id, session_player, room_code, room_password)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(match_proto::JoinMatchResponse {
                joined: result.joined,
                turn_index: result.turn_index,
            }))
        }

        async fn leave_match(
            &self,
            request: Request<match_proto::LeaveMatchRequest>,
        ) -> std::result::Result<Response<match_proto::LeaveMatchResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let match_id = conv::parse_uuid(&req.match_id)
                .map_err(Into::<tonic::Status>::into)?;
            let player = req
                .player
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("player required"))?;
            let player_id = player
                .player_id
                .as_ref()
                .map(|e| e.id.clone())
                .unwrap_or_default();
            tracing::debug!(
                service = "match-service",
                method = "LeaveMatch",
                match_id = %match_id,
                player_id = %player_id,
                surrender = req.surrender,
                "leave_match"
            );
            let result = v2
                .leave_match(match_id, &player_id, req.surrender)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(match_proto::LeaveMatchResponse {
                left: result.left,
                match_result: result.match_result,
            }))
        }

        async fn get_match_state(
            &self,
            request: Request<match_proto::GetMatchStateRequest>,
        ) -> std::result::Result<Response<match_proto::GetMatchStateResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let match_id = conv::parse_uuid(&req.match_id)
                .map_err(Into::<tonic::Status>::into)?;
            let player = req
                .player
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("player required"))?;
            let session_player = conv::player_from_proto(player);
            let state = v2
                .get_match_state(match_id, &session_player)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let pending_moves: Vec<match_proto::Move> = state
                .session
                .pending_moves
                .iter()
                .map(conv::move_to_proto)
                .collect();
            Ok(Response::new(match_proto::GetMatchStateResponse {
                r#match: Some(conv::session_to_match_proto(&state.session)),
                board_snapshot: state.board_snapshot,
                pending_moves,
                next_turn_deadline_ms: state.next_turn_deadline_ms.unwrap_or(0),
            }))
        }

        async fn submit_move(
            &self,
            request: Request<match_proto::SubmitMoveRequest>,
        ) -> std::result::Result<Response<match_proto::SubmitMoveResponse>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let match_id = conv::parse_uuid(&req.match_id)
                .map_err(Into::<tonic::Status>::into)?;
            let player = req
                .player
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("player required"))?;
            let session_player = conv::player_from_proto(player);
            let move_proto = req
                .r#move
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("move required"))?;
            let entity_move = conv::move_from_proto(match_id, req.turn_index, move_proto);
            tracing::debug!(
                service = "match-service",
                method = "SubmitMove",
                match_id = %match_id,
                player_id = %session_player.player_id,
                turn_index = req.turn_index,
                "submit_move"
            );
            let result = v2
                .submit_move(match_id, &session_player, req.turn_index, entity_move)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(match_proto::SubmitMoveResponse {
                accepted: result.accepted,
                new_turn_index: result.new_turn_index,
                new_board_snapshot_ref: result.new_board_snapshot_ref.unwrap_or_default(),
                reject_reason: result.reject_reason.unwrap_or_default(),
            }))
        }

        async fn subscribe_match(
            &self,
            request: Request<match_proto::SubscribeMatchRequest>,
        ) -> std::result::Result<Response<Self::SubscribeMatchStream>, Status> {
            let v2 = self.impl_.v2().map_err(Into::<tonic::Status>::into)?;
            let req = request.into_inner();
            let match_id = conv::parse_uuid(&req.match_id)
                .map_err(Into::<tonic::Status>::into)?;
            let player = req
                .player
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("player required"))?;
            let session_player = conv::player_from_proto(player);
            let receiver = v2
                .subscribe_match(match_id, &session_player, req.full_snapshot_first)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(MatchEventStream::new(receiver)))
        }
    }
}

// ============================================================================
// 内部 trait 扩展
// ============================================================================

trait EnqueueRespExt {
    fn with_optional_match_id(self, _match_id: Uuid) -> Self;
}

impl EnqueueRespExt for crate::proto::v1::EnqueueMatchmakingResponse {
    fn with_optional_match_id(self, _match_id: Uuid) -> Self {
        // proto 字段只有 ticket_id + estimated_wait_ms, match_id 暂不带
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InMemoryMatchParticipantRepository, InMemoryMatchRepository};
    use crate::repository_v2::{
        InMemoryGameSessionRepository, InMemoryMatchmakingTicketRepository, InMemoryMoveRepository,
    };

    fn svc() -> MatchServiceImpl {
        MatchServiceImpl::new(
            Arc::new(InMemoryMatchRepository::new()),
            Arc::new(InMemoryMatchParticipantRepository::new()),
        )
    }

    fn svc_v2() -> (MatchServiceImpl, Arc<MatchmakerServiceV2>) {
        let v2 = Arc::new(MatchmakerServiceV2::new(
            Arc::new(InMemoryGameSessionRepository::new()),
            Arc::new(InMemoryMoveRepository::new()),
            Arc::new(InMemoryMatchmakingTicketRepository::new()),
        ));
        let s = MatchServiceImpl::with_matchmaker_v2(
            Arc::new(InMemoryMatchRepository::new()),
            Arc::new(InMemoryMatchParticipantRepository::new()),
            v2.clone(),
        );
        (s, v2)
    }

    fn make_player(id: &str) -> crate::common::v1::PlayerId {
        crate::common::v1::PlayerId {
            player_id: Some(crate::common::v1::EntityId {
                id: id.to_string(),
            }),
            display_name: format!("P-{}", id),
            rank_score: 1500,
            level: 10,
        }
    }

    // ===== v1 (原 6 UT 保留兼容) =====

    #[tokio::test]
    async fn create_and_get_match() {
        let s = svc();
        let m = s
            .create_match("r1".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        assert_eq!(m.status, MatchStatus::Waiting);
        let loaded = s.find_match_by_id(m.id).await.unwrap().unwrap();
        assert_eq!(loaded.room_id, "r1");
    }

    #[tokio::test]
    async fn create_duplicate_room_fails() {
        let s = svc();
        s.create_match("r1".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        let err = s
            .create_match("r1".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn join_then_start_then_finish() {
        let s = svc();
        let m = s
            .create_match("r2".to_string(), MatchMode::FiveVsFive)
            .await
            .unwrap();
        let p_id = Uuid::new_v4();
        s.join_match(m.id, p_id, Team::Blue).await.unwrap();

        let started = s.start_match(m.id).await.unwrap();
        assert_eq!(started.status, MatchStatus::InProgress);

        let finished = s.finish_match(m.id, Some(Team::Blue)).await.unwrap();
        assert_eq!(finished.status, MatchStatus::Finished);
        assert_eq!(finished.winner_team, Some(Team::Blue));
    }

    #[tokio::test]
    async fn start_already_started_fails() {
        let s = svc();
        let m = s
            .create_match("r3".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        s.start_match(m.id).await.unwrap();
        let err = s.start_match(m.id).await.unwrap_err();
        assert!(matches!(err, Error::MatchAlreadyStarted(_)));
    }

    #[tokio::test]
    async fn join_after_start_fails() {
        let s = svc();
        let m = s
            .create_match("r4".to_string(), MatchMode::TwoVsTwo)
            .await
            .unwrap();
        s.start_match(m.id).await.unwrap();
        let err = s
            .join_match(m.id, Uuid::new_v4(), Team::Blue)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MatchAlreadyStarted(_)));
    }

    #[tokio::test]
    async fn health_check() {
        let s = svc();
        assert!(s.health_check().await.unwrap());
    }

    // ===== v2 (9 RPC × 2 UT = 18 UT, per RGS-DTL-038 §4.2) =====

    // ---- 1. EnqueueMatchmaking ----
    #[tokio::test]
    async fn enqueue_matchmaking_happy_queued() {
        let (s, v2) = svc_v2();
        let player = make_player("p1");
        let mode = GameModeV2::Casual as i32;
        let r = v2
            .enqueue_matchmaking(
                conv::player_from_proto(&player),
                conv::game_mode_from_proto(mode),
                0,
                0,
            )
            .await
            .unwrap();
        // 验证 service 层 v2 可用
        let _ = s.v2().unwrap();
        assert!(matches!(
            r,
            crate::matchmaker_v2::EnqueueResult::Queued { .. }
        ));
    }

    #[tokio::test]
    async fn enqueue_matchmaking_validation_rejects_room_mode() {
        let (s, v2) = svc_v2();
        let player = make_player("p1");
        // ROOM 模式应被 EnqueueMatchmaking 拒
        let err = v2
            .enqueue_matchmaking(
                conv::player_from_proto(&player),
                conv::game_mode_from_proto(GameModeV2::Room as i32),
                0,
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
        let _ = s;
    }

    // ---- 2. CancelMatchmaking ----
    #[tokio::test]
    async fn cancel_matchmaking_happy() {
        let (_s, v2) = svc_v2();
        let player = make_player("p1");
        let r = v2
            .enqueue_matchmaking(
                conv::player_from_proto(&player),
                GameModeV2::Casual,
                0,
                0,
            )
            .await
            .unwrap();
        let ticket_id = match r {
            crate::matchmaker_v2::EnqueueResult::Queued { ticket_id, .. } => ticket_id,
            _ => panic!("expected Queued"),
        };
        let cancelled = v2.cancel_matchmaking(ticket_id, "p1").await.unwrap();
        assert!(cancelled);
    }

    #[tokio::test]
    async fn cancel_matchmaking_validation_wrong_player() {
        let (_s, v2) = svc_v2();
        let player = make_player("p1");
        let r = v2
            .enqueue_matchmaking(
                conv::player_from_proto(&player),
                GameModeV2::Casual,
                0,
                0,
            )
            .await
            .unwrap();
        let ticket_id = match r {
            crate::matchmaker_v2::EnqueueResult::Queued { ticket_id, .. } => ticket_id,
            _ => panic!(),
        };
        let err = v2.cancel_matchmaking(ticket_id, "p2").await.unwrap_err();
        assert!(matches!(err, Error::Forbidden(_)));
    }

    // ---- 3. GetMatchmakingStatus ----
    #[tokio::test]
    async fn get_matchmaking_status_happy_queued() {
        let (_s, v2) = svc_v2();
        let player = make_player("p1");
        let r = v2
            .enqueue_matchmaking(
                conv::player_from_proto(&player),
                GameModeV2::Casual,
                0,
                0,
            )
            .await
            .unwrap();
        let ticket_id = match r {
            crate::matchmaker_v2::EnqueueResult::Queued { ticket_id, .. } => ticket_id,
            _ => panic!(),
        };
        let status = v2.get_matchmaking_status(ticket_id).await.unwrap();
        assert_eq!(status.status, TicketStatus::Queued);
    }

    #[tokio::test]
    async fn get_matchmaking_status_validation_not_found() {
        let (_s, v2) = svc_v2();
        let err = v2
            .get_matchmaking_status(Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    // ---- 4. CreateMatch ----
    #[tokio::test]
    async fn create_match_happy_room() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("ROOM1".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        assert_eq!(r.room_code, Some("ROOM1".to_string()));
    }

    #[tokio::test]
    async fn create_match_validation_max_lt_min() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let err = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                1, // max
                2, // min
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // ---- 5. JoinMatch ----
    #[tokio::test]
    async fn join_match_happy_auto_start() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let p2 = make_player("p2");
        let j = v2
            .join_match(r.match_id, conv::player_from_proto(&p2), None, None)
            .await
            .unwrap();
        assert!(j.joined);
    }

    #[tokio::test]
    async fn join_match_validation_full() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                2, // max
                2, // min
                0,
            )
            .await
            .unwrap();
        let p2 = make_player("p2");
        v2.join_match(r.match_id, conv::player_from_proto(&p2), None, None)
            .await
            .unwrap();
        let p3 = make_player("p3");
        let err = v2
            .join_match(r.match_id, conv::player_from_proto(&p3), None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::MatchFull { .. }));
    }

    // ---- 6. LeaveMatch ----
    #[tokio::test]
    async fn leave_match_happy_surrender() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let p2 = make_player("p2");
        v2.join_match(r.match_id, conv::player_from_proto(&p2), None, None)
            .await
            .unwrap();
        // 强制 status=Running (否则 leave_match 的 transition_to_ending 要求 Running/Paused)
        let mut s = v2
            .sessions()
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        v2.sessions().save(&s).await.unwrap();
        let l = v2.leave_match(r.match_id, "p2", true).await.unwrap();
        assert_eq!(l.match_result, "surrender");
    }

    #[tokio::test]
    async fn leave_match_validation_not_in_match() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let err = v2
            .leave_match(r.match_id, "ghost", false)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotInMatch { .. }));
    }

    // ---- 7. GetMatchState ----
    #[tokio::test]
    async fn get_match_state_happy() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let state = v2
            .get_match_state(r.match_id, &conv::player_from_proto(&host))
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&state.board_snapshot).unwrap();
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn get_match_state_validation_not_found() {
        let (_s, v2) = svc_v2();
        let err = v2
            .get_match_state(Uuid::new_v4(), &conv::player_from_proto(&make_player("p1")))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }

    // ---- 8. SubmitMove ----
    #[tokio::test]
    async fn submit_move_happy_end_turn_advances() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        let p2 = make_player("p2");
        v2.join_match(r.match_id, conv::player_from_proto(&p2), None, None)
            .await
            .unwrap();
        // 强制 status=Running
        let mut s = v2
            .sessions()
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        v2.sessions().save(&s).await.unwrap();
        let m = crate::entity_v2::Move::new(
            r.match_id,
            "host".to_string(),
            0,
            MoveType::EndTurn,
            "{}".to_string(),
        );
        let res = v2
            .submit_move(r.match_id, &conv::player_from_proto(&host), 0, m)
            .await
            .unwrap();
        assert!(res.accepted);
        assert_eq!(res.new_turn_index, 1);
    }

    #[tokio::test]
    async fn submit_move_validation_wrong_turn_index() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                2,
                2,
                0,
            )
            .await
            .unwrap();
        let p2 = make_player("p2");
        v2.join_match(r.match_id, conv::player_from_proto(&p2), None, None)
            .await
            .unwrap();
        let mut s = v2
            .sessions()
            .find_by_id(r.match_id)
            .await
            .unwrap()
            .unwrap();
        s.status = SessionStatus::Running;
        s.current_player_id = Some("host".to_string());
        v2.sessions().save(&s).await.unwrap();
        let m = crate::entity_v2::Move::new(
            r.match_id,
            "host".to_string(),
            99, // wrong
            MoveType::PlayCard,
            "{}".to_string(),
        );
        let err = v2
            .submit_move(r.match_id, &conv::player_from_proto(&host), 99, m)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    // ---- 9. SubscribeMatch ----
    #[tokio::test]
    async fn subscribe_match_happy_returns_receiver() {
        let (_s, v2) = svc_v2();
        let host = make_player("host");
        let r = v2
            .create_match(
                conv::player_from_proto(&host),
                GameModeV2::Room,
                Some("R".to_string()),
                None,
                4,
                2,
                0,
            )
            .await
            .unwrap();
        let rx = v2
            .subscribe_match(r.match_id, &conv::player_from_proto(&host), false)
            .await
            .unwrap();
        // 接收方存在即可 (后续测试可 publish 后验证)
        drop(rx);
    }

    #[tokio::test]
    async fn subscribe_match_validation_not_found() {
        let (_s, v2) = svc_v2();
        let err = v2
            .subscribe_match(
                Uuid::new_v4(),
                &conv::player_from_proto(&make_player("p1")),
                false,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound { .. }));
    }
}
