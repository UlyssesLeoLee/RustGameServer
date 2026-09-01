//! leaderboard-service 跨模块集成场景 (per 9/1 PT-WORKER 派工 §3 IT)
//!
//! 3 跨场景：
//! 1. alice 同时入 3 类榜单 → get_player_rank 跨 (type, period) 聚合返回
//! 2. ranked 赛季榜入 N → get_ranked_leaderboard 跨页 → 排序稳定 + 无重复
//! 3. upsert 改变分数 → 跨期查询 rank 同步变化（rank_changed 信号）

use std::sync::Arc;

use leaderboard_service::entity::{LeaderboardPeriod, LeaderboardType};
use leaderboard_service::repository::InMemoryLeaderboardRepository;
use leaderboard_service::service::{LeaderboardDomainService, LeaderboardServiceImpl};
use uuid::Uuid;

fn svc() -> LeaderboardServiceImpl {
    LeaderboardServiceImpl::new(Arc::new(InMemoryLeaderboardRepository::new()))
}

#[tokio::test]
async fn it_alice_aggregated_across_three_boards() {
    let s = svc();
    let alice = Uuid::new_v4();
    // alice 入 ranked 赛季榜
    s.add_entry(
        LeaderboardType::Ranked,
        LeaderboardPeriod::Seasonal,
        "season_2026_s1".to_string(),
        alice,
        "alice".to_string(),
        1500,
        8,
        2,
    )
    .await
    .unwrap();
    // alice 入 casual 周榜
    s.add_entry(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        alice,
        "alice".to_string(),
        42,
        30,
        10,
    )
    .await
    .unwrap();
    // alice 入 collection 历史榜
    s.add_entry(
        LeaderboardType::Collection,
        LeaderboardPeriod::AllTime,
        "".to_string(),
        alice,
        "alice".to_string(),
        10000,
        0,
        0,
    )
    .await
    .unwrap();
    // get_player_rank(weekly) → ranked 查不到 (无 season), casual 在, collection 周期不匹配
    let (ranked_w, casual_w, coll_w) =
        s.get_player_rank(alice, LeaderboardPeriod::Weekly).await.unwrap();
    assert!(ranked_w.is_none());
    assert!(casual_w.is_some());
    assert!(coll_w.is_none());
    // get_player_rank(alltime) → ranked 查不到, casual 周期不匹配, collection 在
    let (ranked_a, casual_a, coll_a) =
        s.get_player_rank(alice, LeaderboardPeriod::AllTime).await.unwrap();
    assert!(ranked_a.is_none());
    assert!(casual_a.is_none());
    assert!(coll_a.is_some());
    assert_eq!(coll_a.unwrap().score, 10000);
}

#[tokio::test]
async fn it_ranked_seasonal_pagination_no_overlap() {
    let s = svc();
    for i in 0..30 {
        s.add_entry(
            LeaderboardType::Ranked,
            LeaderboardPeriod::Seasonal,
            "s2026".to_string(),
            Uuid::new_v4(),
            format!("p{:02}", i),
            (i as i64) * 100,
            0,
            0,
        )
        .await
        .unwrap();
    }
    // 拉 3 页, 每页 10, 验证无重复
    let mut seen = std::collections::HashSet::new();
    for page in 1..=3 {
        let (entries, total, has_next) = s
            .get_ranked_leaderboard(
                LeaderboardPeriod::Seasonal,
                "s2026".to_string(),
                page,
                10,
            )
            .await
            .unwrap();
        assert_eq!(total, 30);
        if page < 3 {
            assert!(has_next);
        } else {
            assert!(!has_next);
        }
        for e in &entries {
            assert!(seen.insert(e.player_id), "duplicate player_id across pages");
        }
    }
    assert_eq!(seen.len(), 30);
}

#[tokio::test]
async fn it_upsert_score_change_signals_rank_change() {
    let s = svc();
    let bob = Uuid::new_v4();
    let carol = Uuid::new_v4();
    // bob 1500 入榜 rank=1
    s.add_entry(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        "".to_string(),
        bob,
        "bob".to_string(),
        1500,
        0,
        0,
    )
    .await
    .unwrap();
    // carol 1000 入榜 rank=2
    let (carol_e, carol_changed) = s
        .add_entry(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            carol,
            "carol".to_string(),
            1000,
            0,
            0,
        )
        .await
        .unwrap();
    assert_eq!(carol_e.rank, 2);
    assert!(carol_changed);
    // carol 提升到 2000 → rank_changed=true, rank=1; bob 降为 2
    let (carol_e2, carol_changed2) = s
        .add_entry(
            LeaderboardType::Casual,
            LeaderboardPeriod::Weekly,
            "".to_string(),
            carol,
            "carol".to_string(),
            2000,
            0,
            0,
        )
        .await
        .unwrap();
    assert_eq!(carol_e2.rank, 1);
    assert!(carol_changed2, "score 提升 rank 必变");
    // 拉榜验证 bob 降为 rank=2
    let (page, _, _) = s
        .get_casual_leaderboard(LeaderboardPeriod::Weekly, 1, 10)
        .await
        .unwrap();
    assert_eq!(page[0].player_id, carol);
    assert_eq!(page[1].player_id, bob);
    assert_eq!(page[1].rank, 2);
}
