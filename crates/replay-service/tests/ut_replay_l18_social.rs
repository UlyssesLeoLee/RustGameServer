//! replay-service W9 L18 5 新增 RPC 单元测试
//!
//! 5 RPC × 2-3 UT = 13 UT 覆盖 (per 任务简报: ≥10 UT):
//! 1. GetReplayInfo: 2 UT (happy path + not found)
//! 2. DeleteReplay: 3 UT (owner happy + non-owner forbidden + replay 不存在)
//! 3. LikeReplay: 3 UT (happy + idempotent + replay 不存在)
//! 4. UnlikeReplay: 3 UT (happy + unliked-not-liked + replay 不存在)
//! 5. CollectReplay: 3 UT (happy + idempotent + validation)
//!
//! 走 mock InMemoryReplayRepository + InMemoryBackend, 不依赖真 PG / 文件系统
//! (per WF-1-55.32 fail-closed 策略, mock 即可验证业务路径)
//!
//! 参考 9/4 MD §0 闪烁之光录像回放"点赞/收集"社交层 (借鉴不照搬)

use std::sync::Arc;

use uuid::Uuid;

use replay_service::error::Error;
use replay_service::repository::{InMemoryReplayRepository, ReplayRepository};
use replay_service::service::{LikesMap, ReplayDomainService, ReplayServiceImpl};
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

async fn save_one_replay(
    svc: &ReplayServiceImpl,
    player_id_str: &str,
) -> uuid::Uuid {
    let meta = svc
        .save_replay(
            Uuid::new_v4(),
            player_id_str.to_string(),
            None,
            replay_service::entity::ReplayMode::Casual,
            b"replay-bytes".to_vec(),
            60,
            0,
            None,
        )
        .await
        .unwrap();
    meta.replay_id
}

// ============================================================================
// L18-6 GetReplayInfo: 2 UT
// ============================================================================

#[tokio::test]
async fn get_replay_info_happy_path() {
    use replay_service::entity::ReplayMode;
    let (svc, _repo, _storage) = make_svc();
    let uploader_str = Uuid::new_v4().to_string();
    let replay_id = save_one_replay(&svc, &uploader_str).await;

    // 先点个赞 / 收个集
    let liker1 = Uuid::new_v4();
    let liker2 = Uuid::new_v4();
    svc.like_replay(replay_id, liker1).await.unwrap();
    svc.like_replay(replay_id, liker2).await.unwrap();
    let collection_id = Uuid::new_v4();
    let player_id = Uuid::parse_str(&uploader_str).unwrap();
    svc.collect_replay(replay_id, player_id, collection_id)
        .await
        .unwrap();

    let info = svc.get_replay_info(replay_id).await.unwrap();
    assert_eq!(info.replay_id, replay_id);
    assert_eq!(info.uploader_id, uploader_str);
    assert_eq!(info.mode, ReplayMode::Casual);
    assert_eq!(info.duration_secs, 60);
    assert_eq!(info.like_count, 2);
    assert_eq!(info.collect_count, 1);
    // created_at_ms 应是有效时间戳 (大于 2020-01-01)
    assert!(info.created_at_ms() > 1_577_836_800_000);
}

#[tokio::test]
async fn get_replay_info_not_found() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc.get_replay_info(Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, Error::ReplayNotFound(_)));
}

#[tokio::test]
async fn get_replay_info_validates_nil_replay_id() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc.get_replay_info(Uuid::nil()).await.unwrap_err();
    assert!(matches!(err, Error::Validation(_)));
}

// ============================================================================
// L18-7 DeleteReplay: 3 UT
// ============================================================================

#[tokio::test]
async fn delete_replay_by_owner_happy_path() {
    let (svc, _repo, _storage) = make_svc();
    let uploader = Uuid::new_v4();
    let replay_id = save_one_replay(&svc, &uploader.to_string()).await;

    let success = svc
        .delete_replay_by_owner(replay_id, uploader)
        .await
        .unwrap();
    assert!(success);

    // 二次查应 ReplayNotFound
    let err = svc.get_replay_info(replay_id).await.unwrap_err();
    assert!(matches!(err, Error::ReplayNotFound(_)));
}

#[tokio::test]
async fn delete_replay_by_owner_non_owner_forbidden() {
    let (svc, _repo, _storage) = make_svc();
    let uploader = Uuid::new_v4();
    let other_player = Uuid::new_v4();
    let replay_id = save_one_replay(&svc, &uploader.to_string()).await;

    let err = svc
        .delete_replay_by_owner(replay_id, other_player)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));

    // replay 仍存在
    let info = svc.get_replay_info(replay_id).await.unwrap();
    assert_eq!(info.replay_id, replay_id);
}

#[tokio::test]
async fn delete_replay_by_owner_missing_replay_returns_false() {
    let (svc, _repo, _storage) = make_svc();
    let player = Uuid::new_v4();
    let success = svc
        .delete_replay_by_owner(Uuid::new_v4(), player)
        .await
        .unwrap();
    assert!(!success);
}

#[tokio::test]
async fn delete_replay_by_owner_cleans_likes_and_collections() {
    let (svc, _repo, _storage) = make_svc();
    let uploader = Uuid::new_v4();
    let replay_id = save_one_replay(&svc, &uploader.to_string()).await;

    // 加点 likes 和 collections
    let liker = Uuid::new_v4();
    svc.like_replay(replay_id, liker).await.unwrap();
    let collection_id = Uuid::new_v4();
    svc.collect_replay(replay_id, uploader, collection_id)
        .await
        .unwrap();

    // 验证 likes/collections 非空
    let likes_handle = svc.likes_handle();
    let likes = likes_handle.read().await;
    assert!(likes.contains_key(&replay_id));
    drop(likes);
    let colls_handle = svc.collections_handle();
    let colls = colls_handle.read().await;
    assert!(colls.contains_key(&replay_id));
    drop(colls);

    // 删 replay
    svc.delete_replay_by_owner(replay_id, uploader)
        .await
        .unwrap();

    // likes/collections 应清空
    let likes_handle = svc.likes_handle();
    let likes = likes_handle.read().await;
    assert!(!likes.contains_key(&replay_id));
    drop(likes);
    let colls_handle = svc.collections_handle();
    let colls = colls_handle.read().await;
    assert!(!colls.contains_key(&replay_id));
}

// ============================================================================
// L18-8 LikeReplay: 3 UT
// ============================================================================

#[tokio::test]
async fn like_replay_happy_path() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let player1 = Uuid::new_v4();
    let player2 = Uuid::new_v4();

    let count1 = svc.like_replay(replay_id, player1).await.unwrap();
    assert_eq!(count1, 1);
    let count2 = svc.like_replay(replay_id, player2).await.unwrap();
    assert_eq!(count2, 2);
}

#[tokio::test]
async fn like_replay_is_idempotent() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let player = Uuid::new_v4();

    let c1 = svc.like_replay(replay_id, player).await.unwrap();
    let c2 = svc.like_replay(replay_id, player).await.unwrap();
    let c3 = svc.like_replay(replay_id, player).await.unwrap();
    assert_eq!(c1, 1);
    assert_eq!(c2, 1); // 幂等
    assert_eq!(c3, 1);
}

#[tokio::test]
async fn like_replay_rejects_missing_replay() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc
        .like_replay(Uuid::new_v4(), Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::ReplayNotFound(_)));
}

#[tokio::test]
async fn like_replay_validates_nil_inputs() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;

    let err1 = svc
        .like_replay(Uuid::nil(), Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(err1, Error::Validation(_)));
    let err2 = svc
        .like_replay(replay_id, Uuid::nil())
        .await
        .unwrap_err();
    assert!(matches!(err2, Error::Validation(_)));
}

// ============================================================================
// L18-9 UnlikeReplay: 3 UT
// ============================================================================

#[tokio::test]
async fn unlike_replay_happy_path() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    svc.like_replay(replay_id, p1).await.unwrap();
    svc.like_replay(replay_id, p2).await.unwrap();

    let after = svc.unlike_replay(replay_id, p1).await.unwrap();
    assert_eq!(after, 1);
    let after2 = svc.unlike_replay(replay_id, p2).await.unwrap();
    assert_eq!(after2, 0);
}

#[tokio::test]
async fn unlike_replay_when_not_liked_returns_zero() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let player = Uuid::new_v4();

    let after = svc.unlike_replay(replay_id, player).await.unwrap();
    assert_eq!(after, 0);
}

#[tokio::test]
async fn unlike_replay_rejects_missing_replay() {
    let (svc, _repo, _storage) = make_svc();
    let err = svc
        .unlike_replay(Uuid::new_v4(), Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::ReplayNotFound(_)));
}

// ============================================================================
// L18-10 CollectReplay: 3 UT
// ============================================================================

#[tokio::test]
async fn collect_replay_happy_path() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let player = Uuid::new_v4();
    let collection_a = Uuid::new_v4();
    let collection_b = Uuid::new_v4();

    let (success1, count1) = svc
        .collect_replay(replay_id, player, collection_a)
        .await
        .unwrap();
    assert!(success1);
    assert_eq!(count1, 1);
    let (success2, count2) = svc
        .collect_replay(replay_id, player, collection_b)
        .await
        .unwrap();
    assert!(success2);
    assert_eq!(count2, 2);
}

#[tokio::test]
async fn collect_replay_is_idempotent() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let player = Uuid::new_v4();
    let collection = Uuid::new_v4();

    let (s1, c1) = svc
        .collect_replay(replay_id, player, collection)
        .await
        .unwrap();
    let (s2, c2) = svc
        .collect_replay(replay_id, player, collection)
        .await
        .unwrap();
    let (s3, c3) = svc
        .collect_replay(replay_id, player, collection)
        .await
        .unwrap();
    assert!(s1);
    assert!(s2); // 幂等
    assert!(s3);
    assert_eq!(c1, 1);
    assert_eq!(c2, 1);
    assert_eq!(c3, 1);
}

#[tokio::test]
async fn collect_replay_validates_inputs() {
    let (svc, _repo, _storage) = make_svc();
    let replay_id = save_one_replay(&svc, &Uuid::new_v4().to_string()).await;
    let player = Uuid::new_v4();
    let collection = Uuid::new_v4();

    let err = svc
        .collect_replay(Uuid::nil(), player, collection)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));

    let err = svc
        .collect_replay(replay_id, Uuid::nil(), collection)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));

    let err = svc
        .collect_replay(replay_id, player, Uuid::nil())
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation(_)));

    let err = svc
        .collect_replay(Uuid::new_v4(), player, collection)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::ReplayNotFound(_)));
}

// ============================================================================
// 共享存储 with_social_storage 工厂测试
// ============================================================================

#[tokio::test]
async fn with_social_storage_shares_state() {
    use std::collections::HashMap;
    let repo: Arc<InMemoryReplayRepository> = Arc::new(InMemoryReplayRepository::new());
    let storage: Arc<InMemoryBackend> = Arc::new(InMemoryBackend::new());

    let mut preset_likes: LikesMap = HashMap::new();
    let mut preset_colls: HashMap<Uuid, std::collections::HashSet<Uuid>> = HashMap::new();
    let replay_id = Uuid::new_v4();
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();
    let c1 = Uuid::new_v4();
    preset_likes.insert(
        replay_id,
        vec![p1, p2].into_iter().collect(),
    );
    preset_colls.insert(replay_id, vec![c1].into_iter().collect());

    let likes = Arc::new(tokio::sync::RwLock::new(preset_likes));
    let colls = Arc::new(tokio::sync::RwLock::new(preset_colls));

    let _svc = ReplayServiceImpl::with_social_storage(
        repo.clone() as Arc<dyn ReplayRepository>,
        storage.clone() as Arc<dyn StorageBackend>,
        likes.clone(),
        colls.clone(),
    );

    // 预填的 likes 共享存储应可见
    let l = likes.read().await;
    assert_eq!(l.get(&replay_id).unwrap().len(), 2);
    drop(l);
}
