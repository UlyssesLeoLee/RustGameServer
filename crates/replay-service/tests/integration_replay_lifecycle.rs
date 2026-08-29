//! replay-service 端到端 IT (per RGS-REQ-038 §FR-008 + 任务清单 "4 个 IT")
//!
//! 4 IT 覆盖:
//! 1. test_save_replay_then_get_replay_round_trip         (Save → Get 完整往返)
//! 2. test_list_replays_by_player_with_pagination         (按 player_a 过滤 + 分页)
//! 3. test_stream_replay_yields_full_payload_via_chunks   (流式读取, 拼回原数据)
//! 4. test_expired_replays_are_cleaned_up                 (过期清理)
//!
//! 走 LocalFsBackend (mock cluster-ops 对象存储, 跨平台 Windows + WSL)
//! + InMemoryReplayRepository, 不依赖真 PG (per WF-1-55.32 fail-closed 策略).

use std::sync::Arc;

use bytes::Bytes;
use chrono::Utc;
use futures::StreamExt;
use uuid::Uuid;

use replay_service::entity::{ReplayFilter, ReplayMeta, ReplayMode};
use replay_service::repository::{InMemoryReplayRepository, PageRequest, ReplayRepository};
use replay_service::service::{ReplayDomainService, ReplayServiceImpl};
use replay_service::storage::{LocalFsBackend, StorageBackend};

fn make_svc() -> (
    ReplayServiceImpl,
    Arc<InMemoryReplayRepository>,
    Arc<LocalFsBackend>,
    tempfile::TempDir,
) {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let storage = Arc::new(LocalFsBackend::new(tmp.path()));
    let repo: Arc<InMemoryReplayRepository> = Arc::new(InMemoryReplayRepository::new());
    let svc = ReplayServiceImpl::new(
        repo.clone() as Arc<dyn ReplayRepository>,
        storage.clone() as Arc<dyn StorageBackend>,
    );
    (svc, repo, storage, tmp)
}

// ============================================================================
// IT 1: SaveReplay → GetReplay round-trip
// ============================================================================

#[tokio::test]
async fn test_save_replay_then_get_replay_round_trip() {
    let (svc, _repo, storage, _tmp) = make_svc();

    let match_id = Uuid::new_v4();
    let payload = b"complete replay data: move log + board snapshots + final state";
    let meta = svc
        .save_replay(
            match_id,
            "player-a-uuid".to_string(),
            Some("player-b-uuid".to_string()),
            ReplayMode::Ranked,
            payload.to_vec(),
            600,
            0,
            Some("saga-rank-001".to_string()),
        )
        .await
        .expect("save_replay");

    // 元数据正确
    assert_eq!(meta.match_id, match_id);
    assert_eq!(meta.object_size, payload.len() as i64);
    assert_eq!(meta.duration_secs, 600);
    assert_eq!(meta.mode, ReplayMode::Ranked);

    // 对象确实在 LocalFs 落盘
    assert!(storage.exists(&meta.object_key).await.unwrap());

    // GetReplay 拉回来, 数据完整
    let replay = svc.get_replay(meta.replay_id).await.expect("get_replay");
    assert_eq!(replay.meta.replay_id, meta.replay_id);
    assert_eq!(replay.meta.object_key, meta.object_key);
    assert_eq!(replay.data, payload);

    // 二次 GetReplay 仍然成功 (幂等)
    let replay2 = svc.get_replay(meta.replay_id).await.expect("get_replay 2");
    assert_eq!(replay2.data, payload);
}

// ============================================================================
// IT 2: ListReplays 按 player_a 过滤 + 分页
// ============================================================================

#[tokio::test]
async fn test_list_replays_by_player_with_pagination() {
    let (svc, _repo, _storage, _tmp) = make_svc();

    // 准备 3 个玩家 × 各 10 个不同 mode 的回放 = 30 个
    for i in 0..30 {
        let (player, mode) = match i % 3 {
            0 => ("alice", ReplayMode::Ranked),
            1 => ("bob", ReplayMode::Casual),
            _ => ("charlie", ReplayMode::Room),
        };
        svc.save_replay(
            Uuid::new_v4(),
            player.to_string(),
            None,
            mode,
            format!("data-{}", i).into_bytes(),
            60 + i as u32,
            0,
            None,
        )
        .await
        .expect("save");
    }

    // 过滤 player_a=alice (10 个), page_size=4 → 3 页: 4 + 4 + 2
    let filter = ReplayFilter {
        player_a_filter: Some("alice".to_string()),
        ..Default::default()
    };
    let (p1, total1, has_next1) = svc
        .list_replays(&filter, PageRequest { page: 1, page_size: 4 })
        .await
        .unwrap();
    assert_eq!(total1, 10);
    assert_eq!(p1.len(), 4);
    assert!(has_next1);
    assert!(p1.iter().all(|m| m.player_a == "alice"));

    let (p2, total2, has_next2) = svc
        .list_replays(&filter, PageRequest { page: 2, page_size: 4 })
        .await
        .unwrap();
    assert_eq!(total2, 10);
    assert_eq!(p2.len(), 4);
    assert!(has_next2);

    let (p3, total3, has_next3) = svc
        .list_replays(&filter, PageRequest { page: 3, page_size: 4 })
        .await
        .unwrap();
    assert_eq!(total3, 10);
    assert_eq!(p3.len(), 2);
    assert!(!has_next3);

    // 过滤 mode + player (alice + Ranked → 仅有 1/3 是 alice × Ranked, 10 个)
    let filter_combined = ReplayFilter {
        player_a_filter: Some("alice".to_string()),
        mode_filter: Some(ReplayMode::Ranked),
        ..Default::default()
    };
    let (items, total, _) = svc
        .list_replays(
            &filter_combined,
            PageRequest { page: 1, page_size: 20 },
        )
        .await
        .unwrap();
    assert_eq!(total, 10);
    assert!(items.iter().all(|m| m.player_a == "alice" && m.mode == ReplayMode::Ranked));
}

// ============================================================================
// IT 3: StreamReplay 流式读取 (chunked output 拼回原数据)
// ============================================================================

#[tokio::test]
async fn test_stream_replay_yields_full_payload_via_chunks() {
    let (svc, _repo, _storage, _tmp) = make_svc();

    // 准备一个 ~10 KiB 的回放
    let payload: Vec<u8> = (0..10_000u32).map(|i| (i % 256) as u8).collect();
    let meta = svc
        .save_replay(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            payload.clone(),
            60,
            0,
            None,
        )
        .await
        .expect("save");

    // 显式 chunk_size=1024 (10 个 chunk) — 注: 内部还会 clamp 到 MIN..MAX
    let mut stream = svc
        .stream_replay(meta.replay_id, 1024, 0)
        .await
        .expect("stream");

    let mut collected: Vec<u8> = Vec::with_capacity(payload.len());
    let mut chunks: Vec<u32> = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.expect("chunk");
        chunks.push(chunk.chunk_index);
        // 验证 offset 单调递增, payload 拼接正确
        let expected_start = chunk.offset as usize;
        let expected_end = expected_start + chunk.payload.len();
        assert_eq!(&chunk.payload, &payload[expected_start..expected_end]);
        collected.extend_from_slice(&chunk.payload);
        if chunk.is_last {
            assert_eq!(expected_end, payload.len());
            break;
        }
    }
    assert_eq!(collected, payload);
    // chunk_size=1024, data=10000 → ceil(10000/1024) = 10 chunks
    assert_eq!(chunks.len(), 10);
    // chunk_index 单调
    for (i, idx) in chunks.iter().enumerate() {
        assert_eq!(*idx, i as u32);
    }

    // 断点续传: start_offset=5120 (5*1024), 应从 offset 5120 开始
    let mut stream2 = svc
        .stream_replay(meta.replay_id, 1024, 5120)
        .await
        .expect("stream 2");
    let mut tail: Vec<u8> = Vec::new();
    while let Some(c) = stream2.next().await {
        let chunk = c.expect("chunk 2");
        tail.extend_from_slice(&chunk.payload);
        if chunk.is_last {
            break;
        }
    }
    // tail 长度 = 10000 - 5120 = 4880
    assert_eq!(tail.len(), 4880);
    assert_eq!(tail, &payload[5120..]);
}

// ============================================================================
// IT 4: 过期 replay 清理 (mock 时间)
// ============================================================================

#[tokio::test]
async fn test_expired_replays_are_cleaned_up() {
    let (svc, repo, storage, _tmp) = make_svc();

    // 1 个正常 (Casual, 7d)
    let active = svc
        .save_replay(
            Uuid::new_v4(),
            "p-active".to_string(),
            None,
            ReplayMode::Casual,
            b"active-bytes".to_vec(),
            60,
            0,
            None,
        )
        .await
        .expect("save active");

    // 3 个已过期 (直接改 expires_at 到过去, 模拟时间流逝)
    let mut expired_keys = Vec::new();
    for i in 0..3 {
        let mut m = ReplayMeta::new(
            Uuid::new_v4(),
            format!("p-exp-{}", i),
            None,
            ReplayMode::Casual,
            format!("replays/2026/08/exp-{}.dat", i),
        );
        m.expires_at = Utc::now() - chrono::Duration::seconds(60 + i as i64);
        repo.insert(&m).await.expect("insert expired");
        storage
            .put(&m.object_key, Bytes::from(format!("exp-bytes-{}", i)))
            .await
            .expect("put expired");
        expired_keys.push(m.object_key.clone());
    }

    // 清理前: 4 个元数据 + 4 个对象
    assert_eq!(
        svc.list_replays(
            &ReplayFilter { include_expired: true, ..Default::default() },
            PageRequest { page: 1, page_size: 20 }
        )
        .await
        .unwrap()
        .1,
        4
    );
    for k in &expired_keys {
        assert!(storage.exists(k).await.unwrap());
    }

    // 跑清理
    let (removed, keys) = svc.cleanup_expired().await.expect("cleanup");
    assert_eq!(removed, 3);
    assert_eq!(keys.len(), 3);

    // 清理后: 1 个元数据 + 1 个对象
    let (items, total, _) = svc
        .list_replays(
            &ReplayFilter { include_expired: true, ..Default::default() },
            PageRequest { page: 1, page_size: 20 },
        )
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].replay_id, active.replay_id);

    for k in &expired_keys {
        assert!(!storage.exists(k).await.unwrap(), "expired object should be deleted: {}", k);
    }
    // active 仍存在
    assert!(storage.exists(&active.object_key).await.unwrap());

    // 二次清理 (无过期) → 0
    let (removed2, keys2) = svc.cleanup_expired().await.unwrap();
    assert_eq!(removed2, 0);
    assert_eq!(keys2.len(), 0);
}
