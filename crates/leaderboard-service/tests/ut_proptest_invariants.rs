//! leaderboard-service proptest 块 (per 9/1 PT-WORKER 派工)
//!
//! Invariant 覆盖：
//! 1. 随机 (score, updated_at) 序列 upsert 后，list_by_board 必须 score DESC + rank 严格递进
//! 2. upsert 同 player_id N 次，最终只有 1 条记录（去重 + 唯一约束）
//! 3. delete_by_id 删后 list 长度严格 -1

use leaderboard_service::entity::{LeaderboardEntry, LeaderboardPeriod, LeaderboardType};
use leaderboard_service::repository::{InMemoryLeaderboardRepository, LeaderboardRepository};
use proptest::prelude::*;
use uuid::Uuid;

fn make_entry(season: &str, score: i64) -> LeaderboardEntry {
    LeaderboardEntry::new(
        LeaderboardType::Casual,
        LeaderboardPeriod::Weekly,
        season.to_string(),
        Uuid::new_v4(),
        "p".to_string(),
        score,
        0,
        0,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Invariant 1: 随机 score 序列入榜后, list_by_board 必须按 score DESC, rank 严格 1..=N 递进
    #[test]
    fn list_is_sorted_by_score_desc(
        scores in proptest::collection::vec(any::<i64>().no_shrink(), 1..30)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repo = InMemoryLeaderboardRepository::new();
            for s in &scores {
                let _ = repo.upsert(&make_entry("", *s)).await.unwrap();
            }
            let (page, total) = repo
                .list_by_board(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 1000, 0)
                .await
                .unwrap();
            prop_assert_eq!(total as usize, scores.len());
            prop_assert_eq!(page.len(), scores.len());
            // rank 必须严格 1..=N
            for (i, e) in page.iter().enumerate() {
                prop_assert_eq!(e.rank, (i + 1) as u32);
            }
            // 分数必须 non-increasing
            for w in page.windows(2) {
                prop_assert!(w[0].score >= w[1].score);
            }
            Ok(())
        })?;
    }

    // Note: proptest 宏内闭包被发送到 thread, lifetimes 必须 'static, 故用 fn-level closure 写

    /// Invariant 2: upsert 同 player_id N 次, 该玩家在 partition 内只有 1 条记录
    #[test]
    fn upsert_same_player_id_is_idempotent(
        initial in any::<i64>(),
        updates in proptest::collection::vec(any::<i64>().no_shrink(), 1..10)
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repo = InMemoryLeaderboardRepository::new();
            // 初始入榜
            let first = make_entry("", initial);
            let pid = first.player_id;
            repo.upsert(&first).await.unwrap();
            // 重复 upsert 同一 player
            for s in &updates {
                let e = LeaderboardEntry::new(
                    LeaderboardType::Casual,
                    LeaderboardPeriod::Weekly,
                    String::new(),
                    pid,
                    "p".to_string(),
                    *s,
                    0,
                    0,
                );
                let _ = repo.upsert(&e).await.unwrap();
            }
            let (page, total) = repo
                .list_by_board(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 1000, 0)
                .await
                .unwrap();
            prop_assert_eq!(total, 1, "upsert should collapse to single record");
            prop_assert_eq!(page.len(), 1);
            // 终态分数 = updates 末位
            prop_assert_eq!(page[0].score, *updates.last().unwrap());
            Ok(())
        })?;
    }

    /// Invariant 3: delete_by_id 后 list 长度严格 -1
    #[test]
    fn delete_reduces_count_by_one(
        n in 2usize..10
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repo = InMemoryLeaderboardRepository::new();
            let mut ids = Vec::new();
            for i in 0..n {
                let e = make_entry("", i as i64);
                ids.push(e.id);
                repo.upsert(&e).await.unwrap();
            }
            let (_before, total_before) = repo
                .list_by_board(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 1000, 0)
                .await
                .unwrap();
            prop_assert_eq!(total_before as usize, n);
            let victim = ids[0];
            let removed = repo.delete_by_id(victim).await.unwrap();
            prop_assert!(removed);
            let (after, total_after) = repo
                .list_by_board(LeaderboardType::Casual, LeaderboardPeriod::Weekly, "", 1000, 0)
                .await
                .unwrap();
            prop_assert_eq!(total_after as usize, n - 1);
            prop_assert_eq!(after.len(), n - 1);
            prop_assert!(!after.iter().any(|e| e.id == victim));
            Ok(())
        })?;
    }
}
