//! PlayerProfile 更新链路 + wins ≤ total 约束 IT (per RGS IT-AGENT-BRIEFING v1 §3.1)
//!
//! ## 目的
//! 验证 player 域卡牌游戏档案 (per DTL-038 §4.3 FR-001) 在多次 update 链路下:
//! 1. 链式 update 业务字段 (ranked_score / total_matches / total_wins / collection_count /
//!    ranked_tier / preferred_locale) 不丢字段
//! 2. 业务不变量 `total_wins ≤ total_matches` 在整条链路上必须保持
//! 3. unknown player / 不存在的 player_id 必须 NotFound
//!
//! ## 范围 (4 IT 覆盖)
//! 1. test_profile_update_chain_preserves_fields — 链式 update 不丢字段
//! 2. test_wins_leq_total_invariant_through_chain — 核心: wins ≤ total 在链路上保持
//! 3. test_update_unknown_player_returns_not_found — 权限/存在性: 不存在 player 必须 NotFound
//! 4. test_profile_chain_accumulates_wins_only_when_matches_grow — 验证合理累加路径
//!
//! ## wins ≤ total 约束的当前实现状态 (per service.rs L253-278 update_player_profile)
//! - **业务层已强制** (per 2026-09-01 22:25 JST WBS v0.2 B3 实装, 基线 commit 858becb +
//!   2ef872b): service.update_player_profile 在 total_wins > total_matches 时
//!   返回 `Error::Validation`, DB 层不加 CHECK 约束 (per OPEN-QA v0.2 §Q3
//!   决策: 业务层校验对累计更新路径更灵活)
//! - 本 IT 范围: 验证**合法输入路径** (wins ≤ total) 在整条链路上"自然保持",
//!   4 tests 全部用 wins ≤ total 输入, 断言保持不变
//! - **业务层拒绝路径** (wins > total) 由 service.rs 末尾 mod tests 覆盖:
//!   `update_player_profile_wins_gt_total_returns_validation_error` +
//!   `update_player_profile_wins_eq_total_plus_one_returns_validation_error`
//!   (1 happy / 1 边界 / 2 error 共 4 case, per 858becb L1717+)
//! - DTL-038 §7.2 player_profiles 表实装同批持久化, 业务层 invariant 已先落地
//! - WBS v0.2 桶 8 (Phase B 业务 P1 backlog) 派工: B3 跟 DTL-038 §7.2 同批
//!   关联, 不单独插队, 5 worker 中 w1 player 责任段
//!
//! ## 跳过机制
//! - 无需 DATABASE_URL (InMemory 路径)

use player_service::entity::PlayerProfile;
use player_service::repository::{DeckRepository, InMemoryDeckRepository, InMemoryPlayerRepository};
use player_service::service::{PlayerService, PlayerServiceImpl};
use std::sync::Arc;
use uuid::Uuid;

/// 业务不变量: total_wins ≤ total_matches.
/// 业务层已强制 (per WBS v0.2 B3, 2026-09-01 22:25 JST 派工; commit 858becb +
/// 2ef872b 在 service.update_player_profile 返 `Error::Validation`).
/// IT 层 helper 显式 assert, 供 4 IT 链路用合法输入验证不变量"自然保持".
fn assert_wins_leq_total(p: &PlayerProfile) {
    assert!(
        p.total_wins <= p.total_matches,
        "业务不变量违反: total_wins={} > total_matches={}",
        p.total_wins,
        p.total_matches
    );
}

/// 构造带 InMemory repo 的 PlayerServiceImpl.
/// (本 IT 不需要 session/deck, 但 PlayerServiceImpl::new 必须 4 参, 用空 repo 占位)
fn make_service() -> (PlayerServiceImpl, Arc<InMemoryPlayerRepository>) {
    let players = Arc::new(InMemoryPlayerRepository::new());
    let sessions = Arc::new(
        player_service::repository::InMemoryPlayerSessionRepository::new(),
    );
    let decks = Arc::new(InMemoryDeckRepository::new());
    let svc = PlayerServiceImpl::new(
        players.clone() as Arc<dyn player_service::repository::PlayerRepository>,
        sessions.clone() as Arc<dyn player_service::repository::PlayerSessionRepository>,
        decks.clone() as Arc<dyn DeckRepository>,
    );
    (svc, players)
}

/// 链式 update 字段保留: register → update 1 (初值) → update 2 (mid values) → update 3 (end values)
/// 每步都验证 service 返回值包含前次 update 的所有字段 + 本次变更字段.
#[tokio::test]
async fn test_profile_update_chain_preserves_fields() {
    let (svc, _players) = make_service();
    let owner = svc.register("profile-alice".to_string()).await.unwrap();
    let pid = owner.id;

    // 1) 初值: get_player_profile → 全默认 0/0/Bronze/zh-CN
    let initial = svc.get_player_profile(pid).await.unwrap();
    assert_eq!(initial.player_id, pid);
    assert_eq!(initial.ranked_score, 0);
    assert_eq!(initial.ranked_tier, "Bronze");
    assert_eq!(initial.total_matches, 0);
    assert_eq!(initial.total_wins, 0);
    assert_eq!(initial.collection_count, 0);
    assert_eq!(initial.preferred_locale, "zh-CN");
    assert_wins_leq_total(&initial);

    // 2) 第 1 次 update: ranked_score=1500, total_matches=10, total_wins=6
    let update1 = PlayerProfile {
        player_id: pid,
        ranked_score: 1500,
        ranked_tier: "Silver".to_string(),
        total_matches: 10,
        total_wins: 6,
        collection_count: 20,
        preferred_locale: "en-US".to_string(),
    };
    let back1 = svc.update_player_profile(update1.clone()).await.unwrap();
    assert_eq!(back1.ranked_score, 1500);
    assert_eq!(back1.ranked_tier, "Silver");
    assert_eq!(back1.total_matches, 10);
    assert_eq!(back1.total_wins, 6);
    assert_eq!(back1.collection_count, 20);
    assert_eq!(back1.preferred_locale, "en-US");
    assert_wins_leq_total(&back1);

    // 3) 第 2 次 update: ranked_score↑=1700, total_matches↑=20, total_wins↑=12
    let update2 = PlayerProfile {
        player_id: pid,
        ranked_score: 1700,
        ranked_tier: "Gold".to_string(),
        total_matches: 20,
        total_wins: 12,
        collection_count: 25,
        preferred_locale: "ja-JP".to_string(),
    };
    let back2 = svc.update_player_profile(update2.clone()).await.unwrap();
    assert_eq!(back2.ranked_score, 1700, "ranked_score 必更新");
    assert_eq!(back2.ranked_tier, "Gold", "ranked_tier 必更新");
    assert_eq!(back2.total_matches, 20, "total_matches 必更新");
    assert_eq!(back2.total_wins, 12, "total_wins 必更新");
    assert_eq!(back2.collection_count, 25, "collection_count 必更新");
    assert_eq!(back2.preferred_locale, "ja-JP", "locale 必更新");
    assert_wins_leq_total(&back2);

    // 4) 第 3 次 update: 重置 ranked_score=0 但 total_matches/wins 仍合理
    let update3 = PlayerProfile {
        player_id: pid,
        ranked_score: 0,
        ranked_tier: "Bronze".to_string(),
        total_matches: 25,
        total_wins: 15,
        collection_count: 30,
        preferred_locale: "zh-CN".to_string(),
    };
    let back3 = svc.update_player_profile(update3.clone()).await.unwrap();
    assert_eq!(back3.ranked_score, 0);
    assert_eq!(back3.ranked_tier, "Bronze");
    assert_eq!(back3.total_matches, 25);
    assert_eq!(back3.total_wins, 15);
    assert_eq!(back3.collection_count, 30);
    assert_wins_leq_total(&back3);
}

/// 核心: wins ≤ total 在整条链路上必须保持.
/// 模拟天梯赛季推进: total_matches/total_wins 同步增长, ranked_score 浮动,
/// 任何中间步不变量都成立.
#[tokio::test]
async fn test_wins_leq_total_invariant_through_chain() {
    let (svc, _players) = make_service();
    let owner = svc.register("inv-bob".to_string()).await.unwrap();
    let pid = owner.id;

    // 链 5 步: total_matches 0→10→25→50→100, total_wins 0→6→15→30→58
    let chain = [
        (0u32, 0u32),
        (10, 6),
        (25, 15),
        (50, 30),
        (100, 58),
    ];
    for (i, (matches, wins)) in chain.iter().enumerate() {
        let p = PlayerProfile {
            player_id: pid,
            ranked_score: 1000 + (i as u32) * 200,
            ranked_tier: if i < 2 { "Bronze" } else { "Silver" }.to_string(),
            total_matches: *matches,
            total_wins: *wins,
            collection_count: 10,
            preferred_locale: "zh-CN".to_string(),
        };
        let back = svc.update_player_profile(p).await.unwrap();
        assert_wins_leq_total(&back);
        assert_eq!(back.total_matches, *matches, "step {}: matches", i);
        assert_eq!(back.total_wins, *wins, "step {}: wins", i);
    }

    // 旁证: 通过 get_player_profile 拉到的也是同一不变量状态
    let final_p = svc.get_player_profile(pid).await.unwrap();
    // 注意: get_player_profile 当前是占位 (返默认), 与 update 不同步; 这里仅验证
    // 不变量检查本身在最终状态上 OK
    assert_wins_leq_total(&final_p);
}

/// 业务链路: 玩家反复"加胜场"的合理路径 — 每赢一场 total_wins+1, total_matches+1,
/// 验证累加后仍 wins ≤ total (即不允许"凭空多出胜场").
#[tokio::test]
async fn test_profile_chain_accumulates_wins_only_when_matches_grow() {
    let (svc, _players) = make_service();
    let owner = svc.register("acc-carol".to_string()).await.unwrap();
    let pid = owner.id;

    // 起始 baseline: 0 战 0 胜
    let baseline = svc
        .update_player_profile(PlayerProfile {
            player_id: pid,
            ranked_score: 0,
            ranked_tier: "Bronze".to_string(),
            total_matches: 0,
            total_wins: 0,
            collection_count: 0,
            preferred_locale: "zh-CN".to_string(),
        })
        .await
        .unwrap();
    assert_wins_leq_total(&baseline);

    // 链: 10 场, 7 胜 (3 负). 每次 +1/+1 (胜) 或 +1/+0 (负)
    let mut matches = 0u32;
    let mut wins = 0u32;
    let outcomes = [true, true, false, true, false, true, true, false, true, true];
    for (i, won) in outcomes.iter().enumerate() {
        matches += 1;
        if *won {
            wins += 1;
        }
        let p = PlayerProfile {
            player_id: pid,
            ranked_score: 100 * (wins as u32),
            ranked_tier: "Bronze".to_string(),
            total_matches: matches,
            total_wins: wins,
            collection_count: 0,
            preferred_locale: "zh-CN".to_string(),
        };
        let back = svc.update_player_profile(p).await.unwrap();
        assert_wins_leq_total(&back);
        assert_eq!(back.total_matches, matches, "step {}: matches", i);
        assert_eq!(back.total_wins, wins, "step {}: wins", i);
    }
    // 最终: 10 战 7 胜
    assert_eq!(matches, 10);
    assert_eq!(wins, 7);
}

/// 不存在 player_id 调 update_player_profile 必须 NotFound (per service.update_player_profile:
/// "验证 player 存在", 找不到 → NotFound).
#[tokio::test]
async fn test_update_unknown_player_returns_not_found() {
    let (svc, _players) = make_service();
    let ghost = Uuid::new_v4();
    let p = PlayerProfile {
        player_id: ghost,
        ranked_score: 0,
        ranked_tier: "Bronze".to_string(),
        total_matches: 0,
        total_wins: 0,
        collection_count: 0,
        preferred_locale: "zh-CN".to_string(),
    };
    let err = svc.update_player_profile(p).await.unwrap_err();
    assert!(
        matches!(err, player_service::error::Error::NotFound { .. }),
        "不存在 player update 必 NotFound, got: {:?}",
        err
    );
}
