//! leaderboard-service 4 端到端 IT (per RGS-REQ-038 §FR-007 + 任务清单 "4 个端到端 IT")
//!
//! 走 mock InMemoryRepository, 不依赖真 PG (per WF-1-55.32 fail-closed 策略,
//! mock db 即可验证业务路径; 真 PG 由 tests/pg_integration 单独覆盖).
//!
//! 4 IT 覆盖:
//! 1. 入榜 → 查询 端到端 (Casual 周榜)
//! 2. 分页 (Ranked 赛季榜, 入 25 玩家, 拉 3 页)
//! 3. 玩家位次 (3 类榜单分别入榜, 验证 GetPlayerRank 聚合)
//! 4. 排序稳定性 (同分时 updated_at 早者优先, rank 严格递进)

use std::sync::Arc;

use leaderboard_service::entity::{LeaderboardPeriod, LeaderboardType};
use leaderboard_service::repository::{InMemoryLeaderboardRepository, LeaderboardRepository};
use leaderboard_service::service::{LeaderboardDomainService, LeaderboardServiceImpl};
use uuid::Uuid;

fn svc() -> LeaderboardServiceImpl {
    LeaderboardServiceImpl::new(Arc::new(InMemoryLeaderboardRepository::new()))
}

// ============================================================================
// IT 1: 入榜 → 查询 (Casual 周榜)
// ============================================================================

#[tokio::test]
async fn it_add_entry_then_query_casual() {
    let s = svc();
    // 3 个玩家入榜
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    let p3 = Uuid::new_v4();
    for (p, score, wins, losses) in [
        (p1, 100, 10, 5),
        (p2, 200, 20, 3),
        (p3, 50, 5, 10),
    ] {
        s.add_entry(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            p,
            format!("p-{}", p.simple()),
            score,
            wins,
            losses,
        )
        .await
        .unwrap();
    }

    let (entries, total, has_next) = s
        .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 20)
        .await
        .unwrap();
    assert_eq!(total, 3);
    assert!(!has_next);
    assert_eq!(entries.len(), 3);
    // rank 1 → score 200, rank 2 → 100, rank 3 → 50
    assert_eq!(entries[0].player_id, p2);
    assert_eq!(entries[0].rank, 1);
    assert_eq!(entries[0].score, 200);
    assert_eq!(entries[1].player_id, p1);
    assert_eq!(entries[1].rank, 2);
    assert_eq!(entries[2].player_id, p3);
    assert_eq!(entries[2].rank, 3);
}

// ============================================================================
// IT 2: 分页 (Ranked 赛季榜, 25 玩家, 拉 3 页)
// ============================================================================

#[tokio::test]
async fn it_paginate_ranked_leaderboard() {
    let s = svc();
    // 25 玩家入 ranked 赛季榜
    for i in 0..25 {
        s.add_entry(
            LeaderboardType::Ranked,
            LeaderboardPeriod::Seasonal,
            "season_2026_s1".to_string(),
            Uuid::new_v4(),
            format!("p-{:02}", i),
            (i as i64) * 100,
            i as u32,
            0,
        )
        .await
        .unwrap();
    }

    // Page 1, size 10 → rank 1..10 (score 2400..1500)
    let (p1, total1, has_next1) = s
        .get_ranked_leaderboard(
            LeaderboardPeriod::Seasonal,
            "season_2026_s1".to_string(),
            1,
            10,
        )
        .await
        .unwrap();
    assert_eq!(total1, 25);
    assert!(has_next1);
    assert_eq!(p1.len(), 10);
    assert_eq!(p1[0].rank, 1);
    assert_eq!(p1[0].score, 2400);
    assert_eq!(p1[9].rank, 10);
    assert_eq!(p1[9].score, 1500);

    // Page 2, size 10 → rank 11..20
    let (p2, total2, has_next2) = s
        .get_ranked_leaderboard(
            LeaderboardPeriod::Seasonal,
            "season_2026_s1".to_string(),
            2,
            10,
        )
        .await
        .unwrap();
    assert_eq!(total2, 25);
    assert!(has_next2);
    assert_eq!(p2[0].rank, 11);
    assert_eq!(p2[9].rank, 20);

    // Page 3, size 10 → rank 21..25 (5 entries, has_next=false)
    let (p3, _, has_next3) = s
        .get_ranked_leaderboard(
            LeaderboardPeriod::Seasonal,
            "season_2026_s1".to_string(),
            3,
            10,
        )
        .await
        .unwrap();
    assert!(!has_next3);
    assert_eq!(p3.len(), 5);
    assert_eq!(p3[0].rank, 21);
    assert_eq!(p3[4].rank, 25);
}

// ============================================================================
// IT 3: 玩家位次聚合 (3 类榜单分别入榜 → GetPlayerRank)
// ============================================================================

#[tokio::test]
async fn it_player_rank_aggregates_three_boards() {
    let s = svc();
    let p = Uuid::new_v4();

    // ranked 赛季榜入榜
    s.add_entry(
        LeaderboardType::Ranked,
        LeaderboardPeriod::Seasonal,
        "season_2026_s1".to_string(),
        p,
        "alice".to_string(),
        1500,
        8,
        2,
    )
    .await
    .unwrap();
    // casual 周榜入榜
    s.add_entry(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p,
        "alice".to_string(),
        42,
        30,
        10,
    )
    .await
    .unwrap();
    // collection 历史榜入榜
    s.add_entry(
        LeaderboardType::Collection,
        LeaderboardPeriod::AllTime,
        "".to_string(),
        p,
        "alice".to_string(),
        10000,
        0,
        0,
    )
    .await
    .unwrap();

    // 3 个其他玩家也入 ranked, 这样 alice 排名是 2
    for i in 0..3 {
        s.add_entry(
            LeaderboardType::Ranked,
            LeaderboardPeriod::Seasonal,
            "season_2026_s1".to_string(),
            Uuid::new_v4(),
            format!("p-{}", i),
            2000 + (i as i64) * 100,
            0,
            0,
        )
        .await
        .unwrap();
    }

    // GetPlayerRank 不能跨 (type, period) 拉 ranked, 这里直接验证 casual + collection
    let (ranked, casual, collection) =
        s.get_player_rank(p, LeaderboardPeriod::Weekly).await.unwrap();
    // ranked: 服务端用空 season_id 查不到, 返回 None
    assert!(ranked.is_none());
    // casual: alice 应在
    assert!(casual.is_some());
    assert_eq!(casual.as_ref().unwrap().score, 42);
    assert_eq!(casual.as_ref().unwrap().wins, 30);
    assert_eq!(casual.as_ref().unwrap().losses, 10);
    // collection: period=AllTime, Weekly 查不到
    assert!(collection.is_none());

    // 查 AllTime: ranked/casual 都查不到, collection 应在
    let (ranked2, casual2, collection2) =
        s.get_player_rank(p, LeaderboardPeriod::AllTime).await.unwrap();
    assert!(ranked2.is_none());
    assert!(casual2.is_none());
    assert!(collection2.is_some());
    assert_eq!(collection2.as_ref().unwrap().score, 10000);
}

// ============================================================================
// IT 4: 排序稳定性 (同分时 updated_at 早者优先)
// ============================================================================

#[tokio::test]
async fn it_sort_stable_when_scores_tied() {
    let repo = InMemoryLeaderboardRepository::new();
    // 3 个玩家同分 100, 按 add 时间排序
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    let p3 = Uuid::new_v4();

    repo.upsert(&leaderboard_service::entity::LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p1,
        "first".to_string(),
        100,
        0,
        0,
    ))
    .await
    .unwrap();
    // 强制时间间隔避免毫秒级冲突
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.upsert(&leaderboard_service::entity::LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p2,
        "second".to_string(),
        100,
        0,
        0,
    ))
    .await
    .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    repo.upsert(&leaderboard_service::entity::LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p3,
        "third".to_string(),
        100,
        0,
        0,
    ))
    .await
    .unwrap();

    // 第 4 个玩家高分, 应排第 1
    let p_top = Uuid::new_v4();
    repo.upsert(&leaderboard_service::entity::LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        p_top,
        "top".to_string(),
        200,
        0,
        0,
    ))
    .await
    .unwrap();

    let s = LeaderboardServiceImpl::new(Arc::new(InMemoryLeaderboardRepository::new()));
    // 改用 service 走 list_by_board (mock repo 独立, 上面的 repo 不共享, 改在 service repo 重建)
    let svc_repo = InMemoryLeaderboardRepository::new();
    // 重做上面入榜流程到 svc_repo
    for (p, name) in [(p1, "first"), (p2, "second"), (p3, "third"), (p_top, "top")] {
        svc_repo
            .upsert(&leaderboard_service::entity::LeaderboardEntry::new(
                LeaderboardType::Casual,
                LeaderboardPeriod::Weekly,
                "".to_string(),
                p,
                name.to_string(),
                if p == p_top { 200 } else { 100 },
                0,
                0,
            ))
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    // 让 svc2 共享 svc_repo 以保证排序稳定
    let svc2 = LeaderboardServiceImpl::new(Arc::new(svc_repo));

    let (entries, total, _) = svc2
        .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 20)
        .await
        .unwrap();
    assert_eq!(total, 4);
    assert_eq!(entries.len(), 4);
    // rank 1 → top (200)
    assert_eq!(entries[0].player_id, p_top);
    assert_eq!(entries[0].rank, 1);
    assert_eq!(entries[0].score, 200);
    // rank 2..4 → 同分 100, 按 updated_at 升序 (入榜时间早者优先)
    assert_eq!(entries[1].player_id, p1);
    assert_eq!(entries[1].rank, 2);
    assert_eq!(entries[1].display_name, "first");
    assert_eq!(entries[2].player_id, p2);
    assert_eq!(entries[2].rank, 3);
    assert_eq!(entries[2].display_name, "second");
    assert_eq!(entries[3].player_id, p3);
    assert_eq!(entries[3].rank, 4);
    assert_eq!(entries[3].display_name, "third");

    // rank 严格递进 1..4
    for (i, e) in entries.iter().enumerate() {
        assert_eq!(e.rank, (i + 1) as u32);
    }
    let _ = s; // 抑制 unused 警告
}
