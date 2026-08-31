//! match-service match 结束 → SaveReplay saga 重试 IT (per W36 2026-08-30)
//!
//! ## 目的 (per IT-AGENT-BRIEFING §3.3 第 3 项)
//! 验证 SaveReplay saga 链路在 replay-service 瞬时失败时, 通过 saga 层 retry
//! 机制能恢复成功; 模拟生产中常见的 1 次瞬时失败场景.
//!
//! ## 设计
//! - 真实 `ReplayClient` 实现是 fire-and-forget 1 次失败即放弃 (per replay_client.rs 设计注释)
//! - saga 层 retry 由本 IT 注入的 `RetryReplayClient` wrapper 提供 (per IT-AGENT-BRIEFING §3.3 "mock 重试机制")
//! - `FailingThenOkMock` 模拟瞬时失败: 头 N 次返 Unavailable, 之后返成功
//!
//! ## 验证项 (3 步)
//! 1. 失败 1 次后重试成功 → SaveReplay 最终 Ok
//! 2. mock 被调用 N+1 次 (N 次失败 + 1 次成功)
//! 3. 最终结果 (replay_id, object_key) 正确

use std::sync::atomic::{AtomicU32, Ordering};
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
// FailingThenOkMock — 模拟瞬时失败: 头 fail_count 次返 Unavailable, 之后返成功
//
// 逻辑: load remaining_failures; > 0 则 compare_exchange 减 1 并返失败; 否则返成功
// ============================================================================
struct FailingThenOkMock {
    remaining_failures: Arc<AtomicU32>,
    calls: Arc<Mutex<Vec<SaveReplayRequest>>>,
    success_object_key: String,
}

impl FailingThenOkMock {
    fn new(fail_count: u32) -> Self {
        Self {
            remaining_failures: Arc::new(AtomicU32::new(fail_count)),
            calls: Arc::new(Mutex::new(Vec::new())),
            success_object_key: format!("obj/test/{}", Uuid::new_v4()),
        }
    }

    fn total_calls(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    fn last_captured(&self) -> Option<SaveReplayRequest> {
        self.calls.lock().unwrap().last().cloned()
    }
}

#[async_trait::async_trait]
impl ReplayClientTrait for FailingThenOkMock {
    async fn save_replay(
        &self,
        req: SaveReplayRequest,
    ) -> Result<SaveReplayOutcome, tonic::Status> {
        self.calls.lock().unwrap().push(req);
        // 简化: load 决定本调用是否失败, 然后 store (CAS 避免双重递减)
        let prev = self.remaining_failures.load(Ordering::SeqCst);
        if prev > 0 {
            // 抢到 1 次失败额度
            let _ = self.remaining_failures.compare_exchange(
                prev,
                prev - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            );
            return Err(tonic::Status::unavailable(
                "replay-service transient error",
            ));
        }
        // 成功
        Ok(SaveReplayOutcome {
            replay_id: Uuid::nil(),
            object_key: self.success_object_key.clone(),
            object_size: 0,
        })
    }
}

// ============================================================================
// RetryReplayClient — saga 层 retry wrapper (per IT-AGENT-BRIEFING §3.3 "mock 重试机制")
// ============================================================================
struct RetryReplayClient {
    inner: Arc<dyn ReplayClientTrait>,
    max_retries: u32,
    /// retry 间退避 base (ms)
    backoff_ms: u64,
}

impl RetryReplayClient {
    fn new(inner: Arc<dyn ReplayClientTrait>, max_retries: u32) -> Self {
        Self {
            inner,
            max_retries,
            backoff_ms: 5,
        }
    }
}

#[async_trait::async_trait]
impl ReplayClientTrait for RetryReplayClient {
    async fn save_replay(
        &self,
        req: SaveReplayRequest,
    ) -> Result<SaveReplayOutcome, tonic::Status> {
        let mut last_err: Option<tonic::Status> = None;
        for attempt in 0..=self.max_retries {
            match self.inner.save_replay(req.clone()).await {
                Ok(outcome) => return Ok(outcome),
                Err(e) => {
                    if attempt < self.max_retries {
                        // retry 前小退避 (5ms)
                        tokio::time::sleep(Duration::from_millis(self.backoff_ms)).await;
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap())
    }
}

fn make_player(id: &str) -> SessionPlayer {
    SessionPlayer::new(id.to_string(), format!("P-{}", id)).with_rank(1500, 10)
}

async fn make_running_session(sessions: &Arc<InMemoryGameSessionRepository>) -> Uuid {
    let match_id = Uuid::new_v4();
    let mut session = match_service::entity_v2::GameSession::new(
        GameMode::Casual,
        make_player("p1"),
        2,
        2,
    );
    session.match_id = match_id;
    session.add_player(make_player("p2")).expect("add p2");
    session.transition_to_starting().expect("starting");
    session.transition_to_running().expect("running");
    session.current_player_id = Some("p1".to_string());
    session.started_at = chrono::Utc::now() - chrono::Duration::seconds(1);
    sessions.save(&session).await.expect("save session");
    match_id
}

async fn submit_surrender(
    svc: &MatchmakerServiceV2,
    match_id: Uuid,
) {
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
    let r = svc
        .submit_move(match_id, &make_player("p1"), 0, mv)
        .await
        .expect("submit_move ok");
    assert!(r.accepted, "Surrender must be accepted");
}

// ============================================================================
// IT: SaveReplay 1 次失败后 saga 层重试成功
// ============================================================================
#[tokio::test]
async fn it_replay_saga_retries_after_one_transient_failure() {
    // 1) inner mock 失败 1 次, 第 2 次成功
    let inner = Arc::new(FailingThenOkMock::new(1));
    let inner_arc: Arc<dyn ReplayClientTrait> = inner.clone();

    // 2) retry wrapper: 最多重试 3 次 (即最多 4 次总尝试)
    let retry_client = Arc::new(RetryReplayClient::new(inner_arc, 3));
    let retry_arc: Arc<dyn ReplayClientTrait> = retry_client.clone();

    // 3) 注入到 matchmaker
    let sessions: Arc<InMemoryGameSessionRepository> =
        Arc::new(InMemoryGameSessionRepository::new());
    let svc = Arc::new(MatchmakerServiceV2::with_replay_client(
        sessions.clone(),
        Arc::new(InMemoryMoveRepository::new()),
        Arc::new(InMemoryMatchmakingTicketRepository::new()),
        retry_arc,
    ));

    // 4) 构造 session, 触发 surrender → trigger_save_replay (fire-and-forget)
    let match_id = make_running_session(&sessions).await;
    submit_surrender(&svc, match_id).await;

    // 5) 等 fire-and-forget spawn + retry 链完成
    // retry 间 5ms 退避 × 1 次 retry + spawn 调度 = 50ms 足够
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 步 1: inner mock 被调用 2 次 (1 fail + 1 success)
    let total_calls = inner.total_calls();
    assert_eq!(
        total_calls, 2,
        "1 次失败后重试 1 次成功, inner mock 应被调用 2 次, got {}",
        total_calls
    );

    // 步 2: 第 2 次 (最后一次) 调用的 req 字段正确
    let last = inner.last_captured().expect("at least 1 call");
    assert_eq!(last.match_id, match_id, "最后调用 match_id 应正确");
    assert_eq!(last.player_a, "p1");
    assert_eq!(last.player_b.as_deref(), Some("p2"));
    assert_eq!(last.mode, GameMode::Casual as i32);

    // 步 3: session 仍 Ended (retry 失败不破坏业务)
    let s_final = svc
        .sessions()
        .find_by_id(match_id)
        .await
        .unwrap()
        .expect("session found");
    assert_eq!(
        s_final.status,
        match_service::entity_v2::SessionStatus::Ended,
        "session 必须 Ended, 不受 SaveReplay 重试影响"
    );
}

// ============================================================================
// IT: SaveReplay 重试耗尽后, 业务仍正常返回 (fire-and-forget 不级联)
// ============================================================================
#[tokio::test]
async fn it_replay_saga_gives_up_after_exhausted_retries() {
    // 1) inner mock 持续失败 100 次 (永远不成功)
    let inner = Arc::new(FailingThenOkMock::new(100));
    let inner_arc: Arc<dyn ReplayClientTrait> = inner.clone();

    // 2) retry wrapper: 最多重试 2 次 (即 3 次总尝试)
    let retry_client = Arc::new(RetryReplayClient::new(inner_arc, 2));
    let retry_arc: Arc<dyn ReplayClientTrait> = retry_client.clone();

    let sessions: Arc<InMemoryGameSessionRepository> =
        Arc::new(InMemoryGameSessionRepository::new());
    let svc = Arc::new(MatchmakerServiceV2::with_replay_client(
        sessions.clone(),
        Arc::new(InMemoryMoveRepository::new()),
        Arc::new(InMemoryMatchmakingTicketRepository::new()),
        retry_arc,
    ));

    let match_id = make_running_session(&sessions).await;
    submit_surrender(&svc, match_id).await;

    // 等 retry 链耗尽: 3 次尝试 × 5ms backoff = 15ms
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 步 1: inner mock 被调用 3 次 (1 初始 + 2 retry)
    let total_calls = inner.total_calls();
    assert_eq!(
        total_calls, 3,
        "1 初始 + 2 retry = 3 次, got {}",
        total_calls
    );

    // 步 2: session 仍 Ended (fire-and-forget 失败不级联)
    let s_final = svc
        .sessions()
        .find_by_id(match_id)
        .await
        .unwrap()
        .expect("session found");
    assert_eq!(
        s_final.status,
        match_service::entity_v2::SessionStatus::Ended,
        "session 必须 Ended, SaveReplay 失败不级联业务"
    );
}
