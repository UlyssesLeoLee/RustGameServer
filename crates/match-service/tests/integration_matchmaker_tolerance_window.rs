//! match-service matchmaker 容差窗口 IT (per RGS-DTL-026 §4.1)
//!
//! ## 目的 (per IT-AGENT-BRIEFING §3.3 第 1 项)
//! 验证 matchmaker 容差函数 grace 期内不扩容 + 过期后线性扩容 + 上限 cap 的行为契约,
//! 并把容差派生到 v2 matchmaker 的 rank_score 范围, 演示
//! "5 玩家 Elo 差 100 → grace 期内不配对 / 过期后自动配对" 的端到端语义.
//!
//! ## mock clock 设计
//! v1 `tolerance()` 是纯函数, `waiting_seconds` 即为 clock 时间; 不需要真实 time 注入.
//! v2 matchmaker 通过 `rank_score_min/max` 间接表达容差窗口, 测试时按
//! `tolerance(waiting, params)` 算出允许范围, 再调 `enqueue_matchmaking`.
//!
//! ## 测试
//! 1. `tolerance_grace_period_holds_initial` (3 步) — grace 期内 (t=0..5) tolerance=initial
//! 2. `tolerance_after_grace_widens_linearly` (3 步) — grace 过后 (t=6..30) 线性扩
//! 3. `tolerance_caps_at_max` (3 步) — t 超过 175 时 cap 在 max
//! 4. `it_five_players_elo_diff_100_match_within_tolerance_window` (4 步) —
//!    5 玩家 Elo 差 100 + 派生容差 range → t=0 全 queued, t=30 配对, t=200 (cap) 仍配对

use std::sync::Arc;

use match_service::entity_v2::{GameMode, SessionPlayer};
use match_service::matchmaker::{tolerance as tolerance_v1, ToleranceParams};
use match_service::matchmaker_v2::{EnqueueResult, MatchmakerServiceV2};
use match_service::repository_v2::{
    InMemoryGameSessionRepository, InMemoryMatchmakingTicketRepository, InMemoryMoveRepository,
};

// ============================================================================
// 默认参数对齐 per RGS-DTL-026 §4.1 提案值 (PH-5 实测前占位)
// ============================================================================
const INITIAL: f64 = 50.0;
const WIDEN_RATE: f64 = 2.0;
const MAX: f64 = 400.0;
const GRACE: u32 = 5;

/// 5 玩家 Elo 差 100 (per IT-AGENT-BRIEFING §3.3)
const PLAYER_ELOS: [u32; 5] = [1500, 1600, 1700, 1800, 1900];

fn make_service() -> MatchmakerServiceV2 {
    MatchmakerServiceV2::new(
        Arc::new(InMemoryGameSessionRepository::new()),
        Arc::new(InMemoryMoveRepository::new()),
        Arc::new(InMemoryMatchmakingTicketRepository::new()),
    )
}

fn make_player(id: &str, elo: u32) -> SessionPlayer {
    SessionPlayer::new(id.to_string(), format!("P-{}", id)).with_rank(elo, 10)
}

/// 用 v1 `tolerance()` 派生 rank_score_min/max
/// 模拟 "等待 waiting_secs 秒" 后, matchmaker 看到的容差窗口
fn elo_range(elo: u32, waiting_secs: u32, params: &ToleranceParams) -> (u32, u32) {
    let t = tolerance_v1(waiting_secs, params);
    let min = (elo as f64 - t).max(0.0) as u32;
    let max = (elo as f64 + t).min(u32::MAX as f64) as u32;
    (min, max)
}

// ============================================================================
// IT 1: grace 期内 tolerance = initial (3 步)
// ============================================================================
#[test]
fn tolerance_grace_period_holds_initial() {
    // per RGS-BAS-026 §4.1: grace period 内不扩容, 保持 initial_tolerance
    let p = ToleranceParams::default();
    assert_eq!(p.initial_tolerance, INITIAL, "initial 应为 50.0");
    assert_eq!(p.grace_period_secs, GRACE, "grace 应为 5s");

    // 步 1: t=0 边界 (刚好 grace 期内)
    assert_eq!(
        tolerance_v1(0, &p),
        INITIAL,
        "t=0 应等于 initial_tolerance"
    );

    // 步 2: t=grace/2 (中间)
    assert_eq!(
        tolerance_v1(GRACE / 2, &p),
        INITIAL,
        "t=2 应等于 initial_tolerance"
    );

    // 步 3: t=grace (边界, 仍 grace 期内)
    assert_eq!(
        tolerance_v1(GRACE, &p),
        INITIAL,
        "t=5 应等于 initial_tolerance (边界包含)"
    );
}

// ============================================================================
// IT 2: grace 过后线性扩 (3 步)
// ============================================================================
#[test]
fn tolerance_after_grace_widens_linearly() {
    let p = ToleranceParams::default();
    // grace=5, widen=2/s, max=400

    // 步 1: t=6 (grace 刚过 1s) → initial + rate * 1 = 52
    assert_eq!(
        tolerance_v1(6, &p),
        INITIAL + WIDEN_RATE * 1.0,
        "t=6 应为 50 + 2*1 = 52"
    );

    // 步 2: t=10 (grace 过后 5s) → 50 + 2*5 = 60
    assert_eq!(
        tolerance_v1(10, &p),
        INITIAL + WIDEN_RATE * 5.0,
        "t=10 应为 50 + 2*5 = 60"
    );

    // 步 3: t=30 (grace 过后 25s) → 50 + 2*25 = 100
    assert_eq!(
        tolerance_v1(30, &p),
        100.0,
        "t=30 应为 50 + 2*25 = 100 (恰好覆盖 Elo 差 100)"
    );
}

// ============================================================================
// IT 3: cap 在 max (3 步)
// ============================================================================
#[test]
fn tolerance_caps_at_max() {
    let p = ToleranceParams::default();
    // grace=5, widen=2/s, max=400
    // t=180 → 50 + 2*175 = 400 (刚好到 max)
    // t=200 → 50 + 2*195 = 440, 但 cap at 400
    // t=10000 → 也 cap at 400

    // 步 1: t=180 (刚好到 max)
    assert_eq!(
        tolerance_v1(180, &p),
        MAX,
        "t=180 应为 max=400 (刚到饱和点)"
    );

    // 步 2: t=200 (已超 20s, 应 cap)
    assert_eq!(
        tolerance_v1(200, &p),
        MAX,
        "t=200 应 cap 在 max=400"
    );

    // 步 3: t=10000 (远超, 应 cap)
    assert_eq!(
        tolerance_v1(10000, &p),
        MAX,
        "t=10000 应 cap 在 max=400 (saturated)"
    );
}

// ============================================================================
// IT 4: 5 玩家 Elo 差 100, 端到端 matchmaker 行为 (4 步)
// ============================================================================
#[tokio::test]
async fn it_five_players_elo_diff_100_match_within_tolerance_window() {
    let svc = make_service();
    let params = ToleranceParams::default();
    let mode = GameMode::Ranked;

    // 步 1: 5 玩家在 grace 期内 (t=0) 入队, 应全 queued
    // tolerance(0) = 50, 派生 range = elo ± 50, 邻玩家 Elo 差 100 > 50, 不配对
    for (i, elo) in PLAYER_ELOS.iter().enumerate() {
        let player = make_player(&format!("p{}", i + 1), *elo);
        let (min, max) = elo_range(*elo, 0, &params);
        let r = svc
            .enqueue_matchmaking(player, mode, min, max)
            .await
            .expect("enqueue ok");
        assert!(
            matches!(r, EnqueueResult::Queued { .. }),
            "p{} (elo={}) t=0 应 queued (tolerance 50 < 100), got {:?}",
            i + 1,
            elo,
            r
        );
    }

    // 步 2: 5 玩家清空 (新建 service 模拟 "新一局"), 等待 t=30s 后入队, 至少 p1-p2 配对
    // tolerance(30) = 100, 派生 range = elo ± 100, 邻玩家 Elo 差 100 正好到边界
    let svc2 = make_service();
    let mut matched_count = 0u32;
    let mut queued_count = 0u32;
    for (i, elo) in PLAYER_ELOS.iter().enumerate() {
        let player = make_player(&format!("q{}", i + 1), *elo);
        let (min, max) = elo_range(*elo, 30, &params);
        let r = svc2
            .enqueue_matchmaking(player, mode, min, max)
            .await
            .expect("enqueue ok");
        match r {
            EnqueueResult::Matched { .. } => matched_count += 1,
            EnqueueResult::Queued { .. } => queued_count += 1,
        }
    }
    // 至少 p1-p2 (elo 1500/1600) 配对; 其余根据队列时序:
    // q1 入队 → queued (无对手)
    // q2 入队 → 1500 在 [1500, 1700], match q1+q2 (matched=1)
    // q3 入队 → q1/q2 已 matched, queued
    // q4 入队 → 候选: q3 的 [1600, 1800] 含 1800, match q3+q4 (matched=2)
    // q5 入队 → q1/q2/q3/q4 状态, queued
    assert!(
        matched_count >= 1,
        "t=30 时至少 1 对应配对 (p1-p2), got matched={} queued={}",
        matched_count,
        queued_count
    );

    // 步 3: 同样 5 玩家, 等待 t=200s (cap 已饱和), 配对数不应少于 t=30
    let svc3 = make_service();
    let mut matched_at_cap = 0u32;
    for (i, elo) in PLAYER_ELOS.iter().enumerate() {
        let player = make_player(&format!("c{}", i + 1), *elo);
        let (min, max) = elo_range(*elo, 200, &params);
        let r = svc3
            .enqueue_matchmaking(player, mode, min, max)
            .await
            .expect("enqueue ok");
        if matches!(r, EnqueueResult::Matched { .. }) {
            matched_at_cap += 1;
        }
    }
    // cap 不应影响配对数 (t=200 和 t=30 都是 2 对匹配: c1-c2 + c3-c4)
    assert_eq!(
        matched_at_cap, 2,
        "t=200 cap 状态下应配对 2 对 (c1-c2 + c3-c4), got {}",
        matched_at_cap
    );

    // 步 4: 单调不减约束 (per RGS-BAS-026 §4.1) — 跨 grace 边界不能减少
    let mut prev = tolerance_v1(0, &params);
    for t in 0..200 {
        let cur = tolerance_v1(t, &params);
        assert!(
            cur >= prev,
            "tolerance 不单调: t={} cur={} < prev={}",
            t,
            cur,
            prev
        );
        prev = cur;
    }
}
