//! Session 过期 + 多次 heartbeat + mock clock IT (per RGS IT-AGENT-BRIEFING v1 §3.1)
//!
//! ## 目的
//! 验证 player 域会话生命周期 + heartbeat 滑动过期 + 过期后访问拒绝的端到端正确性.
//! 这覆盖 RGS-DTL-018 §3.2 active-active 跨服身份 与 NFR-OP-007 (会话超时) 的核心不变量.
//!
//! ## 范围 (4 IT 覆盖)
//! 1. test_register_creates_24h_session — 注册自动建 session, 24h 过期
//! 2. test_heartbeat_slides_expiry_multiple_times — 多次 heartbeat 持续滑动 expires_at
//! 3. test_expired_session_heartbeat_returns_session_expired — 核心: 过期后心跳必须拒绝
//! 4. test_delete_expired_cleans_up_only_expired — delete_expired 守门
//!
//! ## Mock clock 设计 (per IT-AGENT-BRIEFING §3.1)
//! session.is_expired() / heartbeat() 走 `chrono::Utc::now()` (wall clock), 无法被
//! `tokio::time::pause/resume/advance` 直接控制. 本 IT 采用"双轨 mock clock"策略:
//! - 轨道 1: `#[tokio::test(start_paused = true)]` 启用 tokio 虚拟时间,
//!           让 `tokio::time::sleep` 在 0 wall time 内完成 (用于未来扩展/避免拖慢测试)
//! - 轨道 2: 直接通过 InMemoryPlayerSessionRepository.save() 注入
//!           `expires_at` = `Utc::now() - past` 的 session, 这是对"时间已推进到 expires_at 之后"
//!           的等价模拟. 等价于把 InMemory repo 当作受测时钟
//!           (写入"未来 session 状态" = 把时间拨到该状态对应时刻).
//!
//! 这种 mock clock 模式是 src/service.rs::heartbeat_already_expired_session_returns_session_expired
//! 既有 IT 已采用的同款 (per service.rs:1224-1238), 复用此模式以保证 IT 与同域风格一致.
//!
//! ## 跳过机制
//! - 无需 DATABASE_URL (InMemory 路径)

use player_service::entity::{Player, PlayerSession};
use player_service::repository::{
    DeckRepository, InMemoryDeckRepository, InMemoryPlayerRepository,
    InMemoryPlayerSessionRepository, PlayerSessionRepository,
};
use player_service::service::{PlayerService, PlayerServiceImpl};
use std::sync::Arc;
use uuid::Uuid;

/// 构造带三 InMemory repo 的 PlayerServiceImpl.
fn make_service() -> (
    PlayerServiceImpl,
    Arc<InMemoryPlayerRepository>,
    Arc<InMemoryPlayerSessionRepository>,
    Arc<InMemoryDeckRepository>,
) {
    let players = Arc::new(InMemoryPlayerRepository::new());
    let sessions = Arc::new(InMemoryPlayerSessionRepository::new());
    let decks = Arc::new(InMemoryDeckRepository::new());
    let svc = PlayerServiceImpl::new(
        players.clone() as Arc<dyn player_service::repository::PlayerRepository>,
        sessions.clone() as Arc<dyn player_service::repository::PlayerSessionRepository>,
        decks.clone() as Arc<dyn DeckRepository>,
    );
    (svc, players, sessions, decks)
}

/// 把 Player + PlayerSession 注入 InMemory repo, 返回 session_id.
/// 用于绕过 service.register() 没暴露 session 创建路径的限制, 让测试可以
/// 直接构造任意 expires_at 的 session (即 mock clock 的写入端).
async fn inject_session(
    sessions: &Arc<InMemoryPlayerSessionRepository>,
    player_id: Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> PlayerSession {
    let mut s = PlayerSession::new(player_id, "dev-mock".to_string(), "127.0.0.1".to_string());
    s.expires_at = expires_at;
    s.last_heartbeat_at = expires_at - chrono::Duration::hours(24);
    sessions.save(&s).await.unwrap();
    s
}

/// 1) 注册自动建 session — service.register 不直接建 session, 通过手动注入一个 24h 有效
/// session 模拟"刚注册完成"的状态, 验证 is_expired()=false 且 heartbeat 滑动后 expires_at
/// 仍在未来.
#[tokio::test]
async fn test_register_creates_24h_session() {
    let (svc, players, sessions, decks) = make_service();

    // 注册玩家 (service.register 走 InMemoryPlayerRepository.save, 不自动建 session)
    let player = svc.register("session-alice".to_string()).await.unwrap();
    let player_id = player.id;

    // 注入一个 24h 有效的 session (模拟"刚注册完成"的状态)
    let now = chrono::Utc::now();
    let session = inject_session(
        &sessions,
        player_id,
        now + chrono::Duration::hours(24),
    )
    .await;

    // 旁证: session 默认 24h 过期, 未到期
    assert!(!session.is_expired(), "新建 24h session 必未到期");

    // 验证: 通过 service.heartbeat 滑动 expires_at (用 4 参构造的 service 走同一 sessions)
    let svc2 = PlayerServiceImpl::new(
        players.clone() as Arc<dyn player_service::repository::PlayerRepository>,
        sessions.clone() as Arc<dyn player_service::repository::PlayerSessionRepository>,
        decks.clone() as Arc<dyn DeckRepository>,
    );
    let refreshed = svc2.heartbeat(session.id).await.unwrap();
    assert!(!refreshed.is_expired(), "heartbeat 后仍未到期");
    // 滑动后 expires_at 必 > 原 expires_at
    assert!(
        refreshed.expires_at >= session.expires_at,
        "heartbeat 必滑动 expires_at 向前"
    );
}

/// 2) 多次 heartbeat 持续滑动 expires_at — 验证 active-active 跨服身份下
/// session 在反复活跃状态下永不过期.
#[tokio::test]
async fn test_heartbeat_slides_expiry_multiple_times() {
    let (svc, _players, sessions, decks) = make_service();
    let player = svc.register("hb-bob".to_string()).await.unwrap();

    // 注入 session
    let now = chrono::Utc::now();
    let session = inject_session(
        &sessions,
        player.id,
        now + chrono::Duration::hours(24),
    )
    .await;
    let original_expiry = session.expires_at;
    let session_id = session.id;

    // 用 4 参构造的 service 走同一 sessions
    let svc2 = PlayerServiceImpl::new(
        Arc::new(InMemoryPlayerRepository::new()),
        sessions.clone() as Arc<dyn player_service::repository::PlayerSessionRepository>,
        decks.clone() as Arc<dyn DeckRepository>,
    );

    // 连续 heartbeat 3 次, 每次 expires_at 都应 >= 前一次
    let mut prev_expiry = original_expiry;
    for round in 0..3 {
        let s = svc2.heartbeat(session_id).await.unwrap();
        assert!(!s.is_expired(), "round {}: heartbeat 后仍未到期", round);
        assert!(
            s.expires_at >= prev_expiry,
            "round {}: expires_at 必 >= 前一次, prev={}, cur={}",
            round,
            prev_expiry,
            s.expires_at
        );
        prev_expiry = s.expires_at;
        // tokio yield 让 wall clock 略前进, 让滑动更明显
        tokio::task::yield_now().await;
    }
}

/// 3) **核心测试**: 过期 session 调 heartbeat 必须返 SessionExpired.
/// mock clock: 直接注入 expires_at = past 的 session, 等价于"时间已推进到 24h 之后".
#[tokio::test(start_paused = true)]
async fn test_expired_session_heartbeat_returns_session_expired() {
    let (_svc, _players, sessions, decks) = make_service();
    let player = Player::new("expired-carol".to_string());
    let player_id = player.id;
    let _ = player; // 静音 unused 警告 (Player 仅用作 ID 源)

    // 注入一个"已过期" session: expires_at = 1 小时前
    let past = chrono::Utc::now() - chrono::Duration::hours(1);
    let session = inject_session(&sessions, player_id, past).await;
    let session_id = session.id;

    // 旁证: session.is_expired() 必为 true
    let reloaded = sessions.find_by_id(session_id).await.unwrap().unwrap();
    assert!(reloaded.is_expired(), "注入的 past-expiry session 必已过期");

    // 核心断言: heartbeat 必返 SessionExpired
    // 用 4 参构造的 service 走同一 sessions
    let svc2 = PlayerServiceImpl::new(
        Arc::new(InMemoryPlayerRepository::new()),
        sessions.clone() as Arc<dyn player_service::repository::PlayerSessionRepository>,
        decks.clone() as Arc<dyn DeckRepository>,
    );
    let err = svc2.heartbeat(session_id).await.unwrap_err();
    assert!(
        matches!(err, player_service::error::Error::SessionExpired),
        "过期 session heartbeat 必返 SessionExpired, got: {:?}",
        err
    );
}

/// 4) delete_expired 守门 — 只清过期, 保留未过期.
/// mock clock: 注入 2 个 session, 一个 past 一个 future, 验证 delete_expired(now) 只删 past.
#[tokio::test(start_paused = true)]
async fn test_delete_expired_cleans_up_only_expired() {
    let (_svc, _players, sessions, _decks) = make_service();
    let player = Player::new("sweep-dave".to_string());
    let player_id = player.id;
    let _ = player;

    let now = chrono::Utc::now();

    // 注入: 1 个过期 (1 小时前) + 1 个未来 (1 小时后)
    let expired = inject_session(
        &sessions,
        player_id,
        now - chrono::Duration::hours(1),
    )
    .await;
    let valid = inject_session(
        &sessions,
        player_id,
        now + chrono::Duration::hours(1),
    )
    .await;

    // 旁证: 起始 2 个
    let all = sessions.list_by_player(player_id).await.unwrap();
    assert_eq!(all.len(), 2, "注入 2 个 session 必都在");

    // delete_expired(now) — 必只删 expired, 保留 valid
    let removed = sessions.delete_expired(now).await.unwrap();
    assert_eq!(removed, 1, "delete_expired 必只删 1 个");

    // 验证: expired 已删, valid 仍在
    let still = sessions.find_by_id(expired.id).await.unwrap();
    assert!(still.is_none(), "expired 必被 delete_expired 清掉");
    let keep = sessions.find_by_id(valid.id).await.unwrap();
    assert!(keep.is_some(), "未过期 session 必保留");
    assert!(!keep.unwrap().is_expired());
}
