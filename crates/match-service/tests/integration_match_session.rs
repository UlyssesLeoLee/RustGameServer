//! match-service v2 卡牌游戏 session/turn IT (per RGS-DTL-038 §4.2 + §5)
//!
//! 5 个 IT (per 任务要求):
//! 1. 入队 → 撮合 → session 创建
//! 2. 玩家出牌 → board 更新
//! 3. 投降判负
//! 4. 断线 → 暂停 → 重连
//! 5. turn 超时自动判负
//!
//! 桶 9 补完: 通过 MatchServiceImpl (持有 MatchmakerServiceV2) + grpc_service 端到端验证
//! 不依赖真实 DB, 使用 InMemory 仓库

use std::sync::Arc;

use match_service::matchmaker_v2::MatchmakerServiceV2;
use match_service::repository::InMemoryMatchParticipantRepository;
use match_service::repository::InMemoryMatchRepository;
use match_service::repository_v2::{
    InMemoryGameSessionRepository, InMemoryMatchmakingTicketRepository, InMemoryMoveRepository,
};
use match_service::service::grpc_service::MatchGrpcService;
use match_service::service::MatchServiceImpl;

use match_service::entity_v2::{GameMode as GameModeV2, SessionPlayer, SessionStatus};
use match_service::matchmaker_v2::EnqueueResult;
use match_service::proto::v1 as match_proto;

use match_service::common::v1 as common_proto;
use match_service::proto::v1::match_service_server::MatchService;
use tonic::Request;

fn make_player_proto(id: &str) -> common_proto::PlayerId {
    common_proto::PlayerId {
        player_id: Some(common_proto::EntityId {
            id: id.to_string(),
        }),
        display_name: format!("P-{}", id),
        rank_score: 1500,
        level: 10,
    }
}

fn make_service() -> (Arc<MatchServiceImpl>, Arc<MatchmakerServiceV2>) {
    let v2 = Arc::new(MatchmakerServiceV2::new(
        Arc::new(InMemoryGameSessionRepository::new()),
        Arc::new(InMemoryMoveRepository::new()),
        Arc::new(InMemoryMatchmakingTicketRepository::new()),
    ));
    let svc = Arc::new(MatchServiceImpl::with_matchmaker_v2(
        Arc::new(InMemoryMatchRepository::new()),
        Arc::new(InMemoryMatchParticipantRepository::new()),
        v2.clone(),
    ));
    (svc, v2)
}

// ============================================================================
// IT 1: 入队 → 撮合 → session 创建
// ============================================================================
#[tokio::test]
async fn it_enqueue_then_match_creates_session() {
    let (svc, _v2) = make_service();
    let grpc = MatchGrpcService::new(svc.clone());

    // p1 入队
    let p1 = make_player_proto("p1");
    let r1 = grpc
        .enqueue_matchmaking(Request::new(match_proto::EnqueueMatchmakingRequest {
            request_id: "req-1".to_string(),
            player: Some(p1.clone()),
            mode: GameModeV2::Casual as i32,
            rank_score_min: 0,
            rank_score_max: 3000,
            deck_ref: None,
        }))
        .await
        .unwrap()
        .into_inner();
    let ticket1 = r1.ticket_id;
    assert!(!ticket1.is_empty(), "ticket1 should be non-empty");

    // p2 入队 → 撮合 → session 创建
    let p2 = make_player_proto("p2");
    let r2 = grpc
        .enqueue_matchmaking(Request::new(match_proto::EnqueueMatchmakingRequest {
            request_id: "req-2".to_string(),
            player: Some(p2.clone()),
            mode: GameModeV2::Casual as i32,
            rank_score_min: 0,
            rank_score_max: 3000,
            deck_ref: None,
        }))
        .await
        .unwrap()
        .into_inner();

    // p2 应立即 matched
    let status = grpc
        .get_matchmaking_status(Request::new(match_proto::GetMatchmakingStatusRequest {
            ticket_id: r2.ticket_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    // Casual 撮合: p2 应该被撮合, status=Matched
    assert_eq!(
        status.status,
        match_proto::get_matchmaking_status_response::Status::Matched as i32
    );
}

// ============================================================================
// IT 2: 玩家出牌 → board 更新
// ============================================================================
#[tokio::test]
async fn it_submit_move_updates_board() {
    let (svc, v2) = make_service();
    let grpc = MatchGrpcService::new(svc.clone());

    // host 建 ROOM 会话
    let host = make_player_proto("host");
    let create_resp = grpc
        .create_match(Request::new(match_proto::CreateMatchRequest {
            request_id: "req-create".to_string(),
            mode: GameModeV2::Room as i32,
            host: Some(host.clone()),
            deck_ref: None,
            room_code: "ROOM1".to_string(),
            room_password: String::new(),
            max_players: 2,
            ai_difficulty: 0,
        }))
        .await
        .unwrap()
        .into_inner();
    let match_id_str = create_resp.match_id;

    // p2 加入
    let p2 = make_player_proto("p2");
    let _join = grpc
        .join_match(Request::new(match_proto::JoinMatchRequest {
            request_id: "req-join".to_string(),
            match_id: match_id_str.clone(),
            player: Some(p2),
            deck_ref: None,
            room_code: "ROOM1".to_string(),
            room_password: String::new(),
        }))
        .await
        .unwrap()
        .into_inner();

    // 强制 status=Running + current_player_id=host (session 进入 Running 后才能 submit_move)
    let match_id = uuid::Uuid::parse_str(&match_id_str).unwrap();
    let mut s = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    s.status = SessionStatus::Running;
    s.current_player_id = Some("host".to_string());
    v2.sessions().save(&s).await.unwrap();

    // 初始 board snapshot
    let state_before = grpc
        .get_match_state(Request::new(match_proto::GetMatchStateRequest {
            request_id: "req-state-before".to_string(),
            match_id: match_id_str.clone(),
            player: Some(host.clone()),
        }))
        .await
        .unwrap()
        .into_inner();

    // host 出牌
    let mv = match_proto::Move {
        move_id: String::new(),
        player: Some(host.clone()),
        r#type: match_proto::r#move::MoveType::PlayCard as i32,
        payload_json: r#"{"card_id":"C-001"}"#.to_string(),
        occurred_at_ms: 0,
        result_json: String::new(),
        accepted: true,
    };
    let resp = grpc
        .submit_move(Request::new(match_proto::SubmitMoveRequest {
            request_id: "req-submit".to_string(),
            match_id: match_id_str.clone(),
            player: Some(host.clone()),
            turn_index: 0,
            r#move: Some(mv),
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(resp.accepted, "move should be accepted");

    // 出牌后 board snapshot 应该变化 (board.counters 增加了 "last_move_0")
    let state_after = grpc
        .get_match_state(Request::new(match_proto::GetMatchStateRequest {
            request_id: "req-state-after".to_string(),
            match_id: match_id_str.clone(),
            player: Some(host.clone()),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_ne!(
        state_before.board_snapshot, state_after.board_snapshot,
        "board snapshot should change after submit_move"
    );
}

// ============================================================================
// IT 3: 投降判负
// ============================================================================
#[tokio::test]
async fn it_surrender_ends_session() {
    let (svc, v2) = make_service();
    let grpc = MatchGrpcService::new(svc.clone());

    // 建 ROOM 2 人会话
    let host = make_player_proto("host");
    let create_resp = grpc
        .create_match(Request::new(match_proto::CreateMatchRequest {
            request_id: "req-create".to_string(),
            mode: GameModeV2::Room as i32,
            host: Some(host.clone()),
            deck_ref: None,
            room_code: "SURR".to_string(),
            room_password: String::new(),
            max_players: 2,
            ai_difficulty: 0,
        }))
        .await
        .unwrap()
        .into_inner();
    let match_id_str = create_resp.match_id;
    let match_id = uuid::Uuid::parse_str(&match_id_str).unwrap();

    let p2 = make_player_proto("p2");
    grpc.join_match(Request::new(match_proto::JoinMatchRequest {
        request_id: "req-join".to_string(),
        match_id: match_id_str.clone(),
        player: Some(p2),
        deck_ref: None,
        room_code: "SURR".to_string(),
        room_password: String::new(),
    }))
    .await
    .unwrap();

    // 强制 Running
    let mut s = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    s.status = SessionStatus::Running;
    s.current_player_id = Some("host".to_string());
    v2.sessions().save(&s).await.unwrap();

    // p2 投降
    let p2 = make_player_proto("p2");
    let leave_resp = grpc
        .leave_match(Request::new(match_proto::LeaveMatchRequest {
            request_id: "req-surrender".to_string(),
            match_id: match_id_str.clone(),
            player: Some(p2),
            surrender: true,
        }))
        .await
        .unwrap()
        .into_inner();

    assert!(leave_resp.left, "p2 should have left");
    assert_eq!(
        leave_resp.match_result, "surrender",
        "result should be surrender"
    );

    // session 应处于终态
    let s_after = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    assert!(s_after.status.is_terminal(), "session should be terminal");
    assert_eq!(s_after.end_reason, Some("surrender".to_string()));
    assert_eq!(s_after.winner_id, Some("host".to_string()));
}

// ============================================================================
// IT 4: 断线 → 暂停 → 重连 (用 leave_match(false) 标 disconnected, 然后 pause/resume 验证状态机)
// ============================================================================
#[tokio::test]
async fn it_disconnect_pause_resume() {
    let (svc, v2) = make_service();
    let grpc = MatchGrpcService::new(svc.clone());

    let host = make_player_proto("host");
    let create_resp = grpc
        .create_match(Request::new(match_proto::CreateMatchRequest {
            request_id: "req-create".to_string(),
            mode: GameModeV2::Room as i32,
            host: Some(host.clone()),
            deck_ref: None,
            room_code: "DISC".to_string(),
            room_password: String::new(),
            max_players: 2,
            ai_difficulty: 0,
        }))
        .await
        .unwrap()
        .into_inner();
    let match_id_str = create_resp.match_id;
    let match_id = uuid::Uuid::parse_str(&match_id_str).unwrap();

    let p2 = make_player_proto("p2");
    grpc.join_match(Request::new(match_proto::JoinMatchRequest {
        request_id: "req-join".to_string(),
        match_id: match_id_str.clone(),
        player: Some(p2.clone()),
        deck_ref: None,
        room_code: "DISC".to_string(),
        room_password: String::new(),
    }))
    .await
    .unwrap();

    // 强制 Running
    let mut s = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    s.status = SessionStatus::Running;
    s.current_player_id = Some("host".to_string());
    v2.sessions().save(&s).await.unwrap();

    // p2 断线 (surrender=false, 标 disconnected)
    let leave_resp = grpc
        .leave_match(Request::new(match_proto::LeaveMatchRequest {
            request_id: "req-disconnect".to_string(),
            match_id: match_id_str.clone(),
            player: Some(p2),
            surrender: false,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        leave_resp.match_result, "disconnect",
        "p2 should be marked as disconnect"
    );

    // 验证: session 不应进入终态 (因为 host 还在)
    let s_after_dc = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    assert!(
        !s_after_dc.status.is_terminal(),
        "session should NOT be terminal after partial disconnect"
    );
    let p2_player = s_after_dc
        .players
        .iter()
        .find(|p| p.player_id == "p2")
        .unwrap();
    assert!(p2_player.disconnected, "p2 should be marked disconnected");

    // GM 暂停
    v2.pause_session(match_id).await.unwrap();
    let s_paused = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    assert_eq!(s_paused.status, SessionStatus::Paused);

    // 重连 (模拟 GM 恢复)
    v2.resume_session(match_id).await.unwrap();
    let s_resumed = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    assert_eq!(s_resumed.status, SessionStatus::Running);
}

// ============================================================================
// IT 5: turn 超时自动判负 (3 次累计)
// ============================================================================
#[tokio::test]
async fn it_turn_timeout_auto_lose() {
    let (svc, v2) = make_service();
    let grpc = MatchGrpcService::new(svc.clone());

    let host = make_player_proto("host");
    let create_resp = grpc
        .create_match(Request::new(match_proto::CreateMatchRequest {
            request_id: "req-create".to_string(),
            mode: GameModeV2::Room as i32,
            host: Some(host.clone()),
            deck_ref: None,
            room_code: "TIMEOUT".to_string(),
            room_password: String::new(),
            max_players: 2,
            ai_difficulty: 0,
        }))
        .await
        .unwrap()
        .into_inner();
    let match_id_str = create_resp.match_id;
    let match_id = uuid::Uuid::parse_str(&match_id_str).unwrap();

    let p2 = make_player_proto("p2");
    grpc.join_match(Request::new(match_proto::JoinMatchRequest {
        request_id: "req-join".to_string(),
        match_id: match_id_str.clone(),
        player: Some(p2),
        deck_ref: None,
        room_code: "TIMEOUT".to_string(),
        room_password: String::new(),
    }))
    .await
    .unwrap();

    // 强制 Running + current_player_id=host (host 是超时方)
    let mut s = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    s.status = SessionStatus::Running;
    s.current_player_id = Some("host".to_string());
    s.timeout_count = 0;
    v2.sessions().save(&s).await.unwrap();

    // 3 次 timeout → host 判负, p2 是 winner
    v2.timeout_turn(match_id).await.unwrap();
    v2.timeout_turn(match_id).await.unwrap();
    v2.timeout_turn(match_id).await.unwrap();

    let s_after = v2.sessions().find_by_id(match_id).await.unwrap().unwrap();
    assert_eq!(s_after.status, SessionStatus::Ended);
    assert_eq!(s_after.end_reason, Some("timeout".to_string()));
    assert_eq!(s_after.winner_id, Some("p2".to_string()));
}

// ============================================================================
// 辅助: 不通过 gRPC 的入队 (供测试 helper 使用)
// ============================================================================
#[allow(dead_code)]
async fn enqueue_helper(
    v2: &MatchmakerServiceV2,
    player: common_proto::PlayerId,
    mode: GameModeV2,
) -> EnqueueResult {
    let sp = SessionPlayer::new(
        player.player_id.unwrap().id,
        player.display_name,
    )
    .with_rank(player.rank_score, player.level);
    v2.enqueue_matchmaking(sp, mode, 0, 0).await.unwrap()
}
