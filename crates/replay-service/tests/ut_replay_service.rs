//! replay-service 域 Service 单元测试 (per RGS-REQ-038 §FR-008 + 任务清单 "4 RPC × 2 = 8 UT")
//!
//! 8 UT 覆盖 (happy path + validation, 4 RPC 各 2 个):
//! 1.  save_replay_happy_path_with_default_ttl       (SaveReplay happy path)
//! 2.  save_replay_validates_player_a_not_empty      (SaveReplay validation)
//! 3.  get_replay_returns_full_data                  (GetReplay happy path)
//! 4.  get_replay_returns_not_found_for_missing      (GetReplay not found)
//! 5.  list_replays_filters_by_player_a_with_pagination (ListReplays happy path)
//! 6.  list_replays_excludes_expired_by_default      (ListReplays filter)
//! 7.  stream_replay_chunks_split_correctly          (StreamReplay happy path)
//! 8.  stream_replay_rejects_nonexistent             (StreamReplay not found)
//!
//! 走 mock InMemoryReplayRepository + InMemoryBackend, 不依赖真 PG / 文件系统
//! (per WF-1-55.32 fail-closed 策略, mock 即可验证业务路径)

use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use uuid::Uuid;

use replay_service::entity::{ReplayFilter, ReplayMode};
use replay_service::error::Error;
use replay_service::repository::{InMemoryReplayRepository, PageRequest, ReplayRepository};
use replay_service::service::{ReplayDomainService, ReplayServiceImpl};
use replay_service::storage::{InMemoryBackend, StorageBackend};

fn make_svc() -> (ReplayServiceImpl, Arc<InMemoryReplayRepository>, Arc<InMemoryBackend>) {
    let repo: Arc<InMemoryReplayRepository> = Arc::new(InMemoryReplayRepository::new());
    let storage: Arc<InMemoryBackend> = Arc::new(InMemoryBackend::new());
    let svc = ReplayServiceImpl::new(
        repo.clone() as Arc<dyn ReplayRepository>,
        storage.clone() as Arc<dyn StorageBackend>,
    );
    (svc, repo, storage)
}

// ============================================================================
// UT 1: SaveReplay happy path + default TTL
// ============================================================================

#[tokio::test]
async fn save_replay_happy_path_with_default_ttl() {
    let (svc, _repo, _storage) = make_svc();
    let match_id = Uuid::new_v4();
    let meta = svc
        .save_replay(
            match_id,
            "player-a-uuid".to_string(),
            Some("player-b-uuid".to_string()),
            ReplayMode::Ranked,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            600,
            0, // default TTL
            None,
        )
        .await
        .unwrap();
    assert_eq!(meta.match_id, match_id);
    assert_eq!(meta.player_a, "player-a-uuid");
    assert_eq!(meta.player_b, Some("player-b-uuid".to_string()));
    assert_eq!(meta.mode, ReplayMode::Ranked);
    assert_eq!(meta.object_size, 10);
    assert_eq!(meta.duration_secs, 600);
    // Ranked TTL = 90 天
    let diff = (meta.expires_at - meta.created_at).num_seconds();
    assert_eq!(diff, 90 * 24 * 60 * 60);
    assert!(!meta.is_expired());
    assert!(meta.object_key.starts_with("replays/"));
    assert!(meta.object_key.ends_with(".dat"));
}

// ============================================================================
// UT 2: SaveReplay validation
// ============================================================================

#[tokio::test]
async fn save_replay_validates_player_a_not_empty() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc
        .save_replay(
            Uuid::new_v4(),
            String::new(),
            None,
            ReplayMode::Casual,
            vec![1, 2, 3],
            60,
            0,
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}

#[tokio::test]
async fn save_replay_validates_match_id_not_nil() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc
        .save_replay(
            Uuid::nil(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            vec![1, 2, 3],
            60,
            0,
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}

#[tokio::test]
async fn save_replay_validates_mode_must_be_specified() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc
        .save_replay(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Unspecified,
            vec![1, 2, 3],
            60,
            0,
            None,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}

// ============================================================================
// UT 3: GetReplay happy path
// ============================================================================

#[tokio::test]
async fn get_replay_returns_full_data() {
    let (svc, _repo, storage) = make_svc();
    // 先存一个
    let meta = svc
        .save_replay(
            Uuid::new_v4(),
            "p-a".to_string(),
            None,
            ReplayMode::Casual,
            b"replay data bytes".to_vec(),
            30,
            0,
            None,
        )
        .await
        .unwrap();
    // 再拉
    let replay = svc.get_replay(meta.replay_id).await.unwrap();
    assert_eq!(replay.meta.replay_id, meta.replay_id);
    assert_eq!(replay.data, b"replay data bytes".to_vec());
    // 同时验证 storage 里确实有数据
    assert!(storage.exists(&meta.object_key).await.unwrap());
}

// ============================================================================
// UT 4: GetReplay not found
// ============================================================================

#[tokio::test]
async fn get_replay_returns_not_found_for_missing() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc.get_replay(Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, Error::ReplayNotFound(_)));
}

// ============================================================================
// UT 5: ListReplays happy path (过滤 + 分页)
// ============================================================================

#[tokio::test]
async fn list_replays_filters_by_player_a_with_pagination() {
    let (svc, _repo, _storage) = make_svc();
    // 准备 25 个回放: 15 玩家 A, 10 玩家 B
    for i in 0..15 {
        svc.save_replay(
            Uuid::new_v4(),
            "player-A".to_string(),
            None,
            ReplayMode::Ranked,
            format!("data-a-{}", i).into_bytes(),
            60,
            0,
            None,
        )
        .await
        .unwrap();
    }
    for i in 0..10 {
        svc.save_replay(
            Uuid::new_v4(),
            "player-B".to_string(),
            None,
            ReplayMode::Casual,
            format!("data-b-{}", i).into_bytes(),
            60,
            0,
            None,
        )
        .await
        .unwrap();
    }

    // Page 1 size 10 → 玩家 A 应有 15 个
    let filter = ReplayFilter {
        player_a_filter: Some("player-A".to_string()),
        ..Default::default()
    };
    let (page1, total1, has_next1) = svc
        .list_replays(
            &filter,
            PageRequest { page: 1, page_size: 10 },
        )
        .await
        .unwrap();
    assert_eq!(total1, 15);
    assert_eq!(page1.len(), 10);
    assert!(has_next1);

    // Page 2 size 10 → 剩 5 个, has_next=false
    let (page2, total2, has_next2) = svc
        .list_replays(
            &filter,
            PageRequest { page: 2, page_size: 10 },
        )
        .await
        .unwrap();
    assert_eq!(total2, 15);
    assert_eq!(page2.len(), 5);
    assert!(!has_next2);

    // 全部都是玩家 A
    assert!(page1.iter().all(|m| m.player_a == "player-A"));
    assert!(page2.iter().all(|m| m.player_a == "player-A"));
}

// ============================================================================
// UT 6: ListReplays excludes expired by default
// ============================================================================

#[tokio::test]
async fn list_replays_excludes_expired_by_default() {
    let (svc, repo, _storage) = make_svc();
    // 插入 1 个正常 (Casual, 7d)
    svc.save_replay(
        Uuid::new_v4(),
        "p-active".to_string(),
        None,
        ReplayMode::Casual,
        b"active".to_vec(),
        60,
        0,
        None,
    )
    .await
    .unwrap();
    // 直接在 repo 里插 1 个已过期 (Casual mode, 但手动改 expires_at 到过去)
    use replay_service::entity::ReplayMeta;
    let mut m = ReplayMeta::new(
        Uuid::new_v4(),
        "p-expired".to_string(),
        None,
        ReplayMode::Casual,
        "k".to_string(),
    );
    m.expires_at = chrono::Utc::now() - chrono::Duration::seconds(1);
    repo.insert(&m).await.unwrap();

    // 默认 include_expired=false → 应只返 1 个 (active)
    let filter = ReplayFilter::default();
    let (items, total, _) = svc
        .list_replays(&filter, PageRequest { page: 1, page_size: 20 })
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].player_a, "p-active");

    // include_expired=true → 应返 2 个
    let filter_all = ReplayFilter {
        include_expired: true,
        ..Default::default()
    };
    let (_items2, total2, _) = svc
        .list_replays(&filter_all, PageRequest { page: 1, page_size: 20 })
        .await
        .unwrap();
    assert_eq!(total2, 2);
}

// ============================================================================
// UT 7: StreamReplay chunks split correctly
// ============================================================================

#[tokio::test]
async fn stream_replay_chunks_split_correctly() {
    let (svc, _repo, _storage) = make_svc();
    // 准备一个 4096 字节的回放 (4 KiB)
    let data: Vec<u8> = (0..4096u32).map(|i| (i % 256) as u8).collect();
    let meta = svc
        .save_replay(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            data.clone(),
            60,
            0,
            None,
        )
        .await
        .unwrap();
    // 用 1024 byte chunk (刚好不触发 clamp)
    let mut stream = svc
        .stream_replay(meta.replay_id, 1024, 0)
        .await
        .unwrap();
    let mut collected: Vec<u8> = Vec::new();
    let mut last_chunk_idx: u32 = 0;
    let mut chunk_count: u32 = 0;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.unwrap();
        collected.extend_from_slice(&chunk.payload);
        chunk_count += 1;
        last_chunk_idx = chunk.chunk_index;
        if chunk.is_last {
            break;
        }
    }
    assert_eq!(collected, data);
    // chunk_size=1024, data=4096 → 4 chunks (1024+1024+1024+1024), chunk_index 0..3
    assert_eq!(chunk_count, 4);
    assert_eq!(last_chunk_idx, 3);
}

// ============================================================================
// UT 8: StreamReplay not found
// ============================================================================

#[tokio::test]
async fn stream_replay_rejects_nonexistent() {
    let (svc, _repo, _storage) = make_svc();
    let result = svc.stream_replay(Uuid::new_v4(), 64 * 1024, 0).await;
    match result {
        Err(Error::ReplayNotFound(_)) => {} // expected
        Err(other) => panic!("expected ReplayNotFound, got {:?}", other),
        Ok(_) => panic!("expected Err, got Ok"),
    }
}

// ============================================================================
// 额外: cleanup_expired + delete_replay
// ============================================================================

#[tokio::test]
async fn delete_replay_removes_meta_and_object() {
    let (svc, repo, storage) = make_svc();
    let meta = svc
        .save_replay(
            Uuid::new_v4(),
            "p".to_string(),
            None,
            ReplayMode::Casual,
            b"x".to_vec(),
            60,
            0,
            None,
        )
        .await
        .unwrap();
    assert!(storage.exists(&meta.object_key).await.unwrap());
    assert!(svc.delete_replay(meta.replay_id).await.unwrap());
    assert!(!storage.exists(&meta.object_key).await.unwrap());
    assert!(repo.find_by_id(meta.replay_id).await.unwrap().is_none());
}

#[tokio::test]
async fn cleanup_expired_removes_both_meta_and_objects() {
    let (svc, repo, storage) = make_svc();
    use replay_service::entity::ReplayMeta;
    // 1 个正常
    svc.save_replay(
        Uuid::new_v4(),
        "p-active".to_string(),
        None,
        ReplayMode::Casual,
        b"active".to_vec(),
        60,
        0,
        None,
    )
    .await
    .unwrap();
    // 1 个已过期
    let mut m = ReplayMeta::new(
        Uuid::new_v4(),
        "p-expired".to_string(),
        None,
        ReplayMode::Casual,
        "replays/2026/08/expired.dat".to_string(),
    );
    m.expires_at = chrono::Utc::now() - chrono::Duration::seconds(1);
    repo.insert(&m).await.unwrap();
    storage
        .put(&m.object_key, Bytes::from_static(b"expired-bytes"))
        .await
        .unwrap();
    let (removed, keys) = svc.cleanup_expired().await.unwrap();
    assert_eq!(removed, 1);
    assert_eq!(keys.len(), 1);
    assert!(keys[0].contains("expired"));
    assert!(!storage.exists(&m.object_key).await.unwrap());
    assert!(repo.find_by_id(m.replay_id).await.unwrap().is_none());
}
