//! match-service match session → SaveReplay saga IT (per W36 2026-08-30)
//!
//! ## 目的 (per IT-AGENT-BRIEFING §3.3 第 2 项)
//! 验证 match session 终态 (投降/超时判负) 触发 SaveReplay saga 时,
//! 业务层构造的 `SaveReplayRequest` 字段 (match_id, player_a/b, mode, data, duration_secs) 准确.
//! 用 MockReplayClient 捕获请求, 不连真实 replay-service gRPC.
//!
//! ## 触发路径
//! 1. `submit_move(MoveType::Surrender)` → session Ending → Ended → trigger_save_replay
//!
//! ## 验证项 (3 步)
//! 1. mock 收到 1 次 save_replay 调用
//! 2. match_id == session.match_id
//! 3. player_a == host, player_b == Some(opponent), mode == Casual, data 非空
//!
//! ## 设计
//! - 复用 InMemory 仓库 (per IT-AGENT-BRIEFING §1 InMemory mock 风格)
//! - MockReplayClient 完整实现 ReplayClientTrait, 捕获所有请求
//! - fire-and-forget 用 `tokio::time::sleep` 等 spawn 任务完成

use std::sync::{Arc, Mutex};
use std::time::Duration;

use match_service::entity_v2::{GameMode, Move, MoveType, SessionPlayer};
use match_service::matchmaker_v2::MatchmakerServiceV2;
use match_service::replay_client::{
    ReplayClientTrait, SaveReplayOutcome, SaveReplayRequest,
};
use match_service::repository_v2::{
    GameSessionRepository, InMemoryGameSessionRepository, InMemoryMatchmakingTicketRepository,
    InMemoryMoveRepository,
};
use uuid::Uuid;

// ============================================================================
// MockReplayClient — 捕获所有 save_replay 调用供 IT 验证
// ============================================================================
struct MockReplayClient {
    calls: Arc<Mutex<Vec<SaveReplayRequest>>>,
}

impl MockReplayClient {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn captured(&self) -> Vec<SaveReplayRequest> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl ReplayClientTrait for MockReplayClient {
    async fn save_replay(
        &self,
        req: SaveReplayRequest,
    ) -> Result<SaveReplayOutcome, tonic::Status> {
        self.calls.lock().unwrap().push(req);
        Ok(SaveReplayOutcome {
            replay_id: Uuid::nil(),
            object_key: format!("obj/test/{}", Uuid::new_v4()),
            object_size: 0,
        })
    }
}

fn make_player(id: &str, elo: u32) -> SessionPlayer {
    SessionPlayer::new(id.to_string(), format!("P-{}", id)).with_rank(elo, 10)
}

fn make_service_with_replay(
    mock: Arc<MockReplayClient>,
) -> (Arc<MatchmakerServiceV2>, Arc<InMemoryGameSessionRepository>) {
    let sessions: Arc<InMemoryGameSessionRepository> = Arc::new(InMemoryGameSessionRepository::new());
    let svc = Arc::new(MatchmakerServiceV2::with_replay_client(
        sessions.clone(),
        Arc::new(InMemoryMoveRepository::new()),
        Arc::new(InMemoryMatchmakingTicketRepository::new()),
        mock,
    ));
    (svc, sessions)
}

// ============================================================================
// IT: match session → Surrender → SaveReplay saga 触发 + 字段准确
// ============================================================================
#[tokio::test]
async fn it_match_session_to_replay_saga_sends_correct_request() {
    let mock = Arc::new(MockReplayClient::new());
    let (svc, sessions) = make_service_with_replay(mock.clone());

    // 1) 构造 Running session: Casual, p1 (host) vs p2
    let match_id = Uuid::new_v4();
    let mut session = match_service::entity_v2::GameSession::new(
        GameMode::Casual,
        make_player("p1", 1500),
        2,
        2,
    );
    session.match_id = match_id;
    session
        .add_player(make_player("p2", 1500))
        .expect("add p2");
    session.transition_to_starting().expect("starting");
    session.transition_to_running().expect("running");
    session.current_player_id = Some("p1".to_string());
    // started_at 设为 1s 前, 便于验证 duration_secs > 0
    session.started_at = chrono::Utc::now() - chrono::Duration::seconds(1);
    sessions.save(&session).await.expect("save session");

    // 2) p1 提交 Surrender → session → Ended → trigger_save_replay (fire-and-forget)
    let mv = Move {
        move_id: Uuid::new_v4(),
        match_id,
        player_id: "p1".to_string(),
        turn_index: 0,
        move_type: MoveType::Surrender,
        payload_json: "{}".to_string(),
        result_json: None,
        accepted: false,
        reject_reason: None,
        occurred_at: chrono::Utc::now(),
    };
    let submit_result = svc
        .submit_move(match_id, &make_player("p1", 1500), 0, mv)
        .await
        .expect("submit_move ok");
    assert!(submit_result.accepted, "Surrender must be accepted");

    // 3) 验证 session 已 Ended
    let s_final = svc
        .sessions()
        .find_by_id(match_id)
        .await
        .unwrap()
        .expect("session found");
    assert_eq!(
        s_final.status,
        match_service::entity_v2::SessionStatus::Ended,
        "session must be Ended after surrender"
    );

    // 4) 等 fire-and-forget spawn 完成 (mock sync mutex, 50ms 足够)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 步 1: mock 收到 1 次 save_replay
    let calls = mock.captured();
    assert_eq!(
        calls.len(),
        1,
        "SaveReplay saga 应被触发 1 次, got {}",
        calls.len()
    );

    // 步 2: match_id 正确
    let req = &calls[0];
    assert_eq!(
        req.match_id, match_id,
        "SaveReplayRequest.match_id 应等于 session.match_id"
    );

    // 步 3: player_a, player_b, mode, data 字段准确
    assert_eq!(
        req.player_a, "p1",
        "player_a 应为 host (p1), got {}",
        req.player_a
    );
    assert_eq!(
        req.player_b.as_deref(),
        Some("p2"),
        "player_b 应为 opponent (p2), got {:?}",
        req.player_b
    );
    assert_eq!(
        req.mode,
        GameMode::Casual as i32,
        "mode 应为 Casual (2), got {}",
        req.mode
    );
    assert!(
        !req.data.is_empty(),
        "data 字段不应为空 (board JSON 序列化)"
    );
    // data 解析后应含 match_id + board + players + end_reason
    let data_str = std::str::from_utf8(&req.data).expect("data 是 utf8");
    assert!(
        data_str.contains(&match_id.to_string()),
        "data 应含 match_id, got: {}",
        data_str
    );
    assert!(
        data_str.contains("p1") && data_str.contains("p2"),
        "data 应含 players, got: {}",
        data_str
    );
    assert!(
        data_str.contains("surrender"),
        "data 应含 end_reason=surrender, got: {}",
        data_str
    );

    // 步 4: duration_secs > 0 (started_at 设 1s 前)
    assert!(
        req.duration_secs >= 1,
        "duration_secs 应 >= 1, got {}",
        req.duration_secs
    );

    // 步 5: saga_id (单次调用模式, 应 None)
    assert!(
        req.saga_id.is_none(),
        "W36 单次调用 saga_id 应 None, got {:?}",
        req.saga_id
    );
}
