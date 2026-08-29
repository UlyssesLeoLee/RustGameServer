//! leaderboard-service 4 RPC 单元测试 (per RGS-REQ-038 §FR-007 + 任务清单 "4 个 RPC 各 2 UT")
//!
//! 8 UT 覆盖:
//! 1.  get_ranked_leaderboard_requires_season_id
//! 2.  get_casual_leaderboard_returns_empty_initially
//! 3.  add_entry_then_get_casual_leaderboard
//! 4.  get_player_rank_returns_three_entries
//! 5.  upsert_rank_changes_when_score_overtakes
//! 6.  pagination_respects_limit_and_offset
//! 7.  ranked_validation_empty_season_rejected
//! 8.  add_entry_rejects_empty_display_name

use std::sync::Arc;

use leaderboard_service::entity::{LeaderboardPeriod, LeaderboardType};
use leaderboard_service::error::Error;
use leaderboard_service::repository::{InMemoryLeaderboardRepository, LeaderboardRepository};
use leaderboard_service::service::{LeaderboardDomainService, LeaderboardServiceImpl};
use uuid::Uuid;

fn svc() -> LeaderboardServiceImpl {
    LeaderboardServiceImpl::new(Arc::new(InMemoryLeaderboardRepository::new()))
}

// ============================================================================
// UT 1: GetRankedLeaderboard — ranked 榜要求 season_id 非空
// ============================================================================

#[tokio::test]
async fn get_ranked_leaderboard_requires_season_id() {
    let s = svc();
    let err = s
        .get_ranked_leaderboard(LeaderboardPeriod::Seasonal, "".to_string(), 1, 10)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::InvalidLeaderboardSpec(_)));
}

// ============================================================================
// UT 2: GetCasualLeaderboard — 空榜返回空 entries
// ============================================================================

#[tokio::test]
async fn get_casual_leaderboard_returns_empty_initially() {
    let s = svc();
    let (entries, total, has_next) = s
        .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 20)
        .await
        .unwrap();
    assert!(entries.is_empty());
    assert_eq!(total, 0);
    assert!(!has_next);
}

// ============================================================================
// UT 3: AddEntry + GetCasualLeaderboard — 入榜后能查询到
// ============================================================================

#[tokio::test]
async fn add_entry_then_get_casual_leaderboard() {
    let s = svc();
    let p = Uuid::new_v4();
    let (entry, rank_changed) = s
        .add_entry(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            p,
            "alice".to_string(),
            100,
            10,
            5,
        )
        .await
        .unwrap();
    assert_eq!(entry.score, 100);
    assert_eq!(entry.wins, 10);
    assert_eq!(entry.losses, 5);
    assert_eq!(entry.rank, 1);
    assert!(rank_changed, "首次入榜 rank_changed 必为 true");

    let (entries, total, _) = s
        .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 20)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display_name, "alice");
    assert_eq!(entries[0].player_id, p);
}

// ============================================================================
// UT 4: GetPlayerRank — 玩家在 3 类榜单的位置
// ============================================================================

#[tokio::test]
async fn get_player_rank_returns_three_entries() {
    let s = svc();
    let p = Uuid::new_v4();
    s.add_entry(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p,
        "bob".to_string(),
        5,
        3,
        1,
    )
    .await
    .unwrap();
    s.add_entry(
        LeaderboardType::Collection,
        LeaderboardPeriod::AllTime,
        "".to_string(),
        p,
        "bob".to_string(),
        500,
        0,
        0,
    )
    .await
    .unwrap();

    // ranked 没入 (需要 season_id), casual/collection 已入
    let (ranked, casual, collection) =
        s.get_player_rank(p, LeaderboardPeriod::Weekly).await.unwrap();
    assert!(ranked.is_none());
    assert!(casual.is_some());
    assert_eq!(casual.as_ref().unwrap().score, 5);
    // collection 的 period 是 AllTime, Weekly 查不到
    assert!(collection.is_none());

    // AllTime 查: ranked 缺 season 没入, casual 是 Weekly 没入, collection 应在
    let (ranked2, casual2, collection2) =
        s.get_player_rank(p, LeaderboardPeriod::AllTime).await.unwrap();
    assert!(ranked2.is_none());
    assert!(casual2.is_none());
    assert!(collection2.is_some());
    assert_eq!(collection2.unwrap().score, 500);
}

// ============================================================================
// UT 5: Upsert — 新高分玩家超越老玩家, rank 变化
// ============================================================================

#[tokio::test]
async fn upsert_rank_changes_when_score_overtakes() {
    let repo = InMemoryLeaderboardRepository::new();
    // p1 先入榜, score=100
    let p1 = Uuid::new_v4();
    let e1 = leaderboard_service::entity::LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p1,
        "p1".to_string(),
        100,
        0,
        0,
    );
    let (out1, _) = repo.upsert(&e1).await.unwrap();
    assert_eq!(out1.rank, 1);

    // p2 入榜, score=200, 应排第 1; p1 降为 rank 2
    let p2 = Uuid::new_v4();
    let e2 = leaderboard_service::entity::LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p2,
        "p2".to_string(),
        200,
        0,
        0,
    );
    let (out2, _) = repo.upsert(&e2).await.unwrap();
    assert_eq!(out2.rank, 1);

    let p1_rank = repo
        .rank_of(p1, LeaderboardType::Casual, LeaderboardPeriod::Weekly, "")
        .await
        .unwrap();
    assert_eq!(p1_rank, Some(2));
}

// ============================================================================
// UT 6: 分页 — limit + offset 正确
// ============================================================================

#[tokio::test]
async fn pagination_respects_limit_and_offset() {
    let s = svc();
    // 入 5 个玩家, score 0..50
    let mut pids = Vec::new();
    for i in 0..5 {
        let p = Uuid::new_v4();
        pids.push(p);
        s.add_entry(
            LeaderboardType::Collection,
            LeaderboardPeriod::AllTime,
            "".to_string(),
            p,
            format!("p{}", i),
            (i as i64) * 10,
            0,
            0,
        )
        .await
        .unwrap();
    }
    // Page 1, size 2 → score 40, 30 (rank 1, 2)
    let (p1, total1, has_next1) = s
        .get_collection_leaderboard(LeaderboardPeriod::AllTime, 1, 2)
        .await
        .unwrap();
    assert_eq!(total1, 5);
    assert!(has_next1);
    assert_eq!(p1[0].score, 40);
    assert_eq!(p1[0].rank, 1);
    assert_eq!(p1[1].score, 30);
    assert_eq!(p1[1].rank, 2);

    // Page 3, size 2 → score 0 (rank 5)
    let (p3, total3, has_next3) = s
        .get_collection_leaderboard(LeaderboardPeriod::AllTime, 3, 2)
        .await
        .unwrap();
    assert_eq!(total3, 5);
    assert!(!has_next3);
    assert_eq!(p3[0].score, 0);
    assert_eq!(p3[0].rank, 5);
}

// ============================================================================
// UT 7: 非法分页参数 → InvalidPage
// ============================================================================

#[tokio::test]
async fn ranked_validation_empty_season_rejected() {
    let s = svc();
    // season_id 为空
    let err1 = s
        .get_ranked_leaderboard(LeaderboardPeriod::AllTime, "".to_string(), 1, 10)
        .await
        .unwrap_err();
    assert!(matches!(err1, Error::InvalidLeaderboardSpec(_)));

    // page=0
    let err2 = s
        .get_ranked_leaderboard(LeaderboardPeriod::AllTime, "s1".to_string(), 0, 10)
        .await
        .unwrap_err();
    assert!(matches!(err2, Error::InvalidPage(_)));
}

// ============================================================================
// UT 8: AddEntry 拒绝空 display_name
// ============================================================================

#[tokio::test]
async fn add_entry_rejects_empty_display_name() {
    let s = svc();
    let err = s
        .add_entry(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            Uuid::new_v4(),
            "".to_string(),
            100,
            0,
            0,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}
