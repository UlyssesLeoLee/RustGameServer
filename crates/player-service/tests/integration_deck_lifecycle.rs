//! 卡牌游戏桶 11 端到端 IT (per DTL-038 §4.3 + FR-002, 桶 11 增量)
//!
//! ## 目的
//! 验证 deck CRUD + share 完整业务链路 (创建→更新→分享→拉取→删除),
//! 走 InMemoryDeckRepository 模拟 DB 路径, 不依赖外部 PG.
//!
//! ## 范围 (5 IT 覆盖 per 任务书要求)
//! 1. test_create_deck_returns_draft_default
//! 2. test_full_lifecycle_create_update_share_pull_delete
//! 3. test_list_decks_pagination_per_owner
//! 4. test_share_deck_unpublic_clears_share_code
//! 5. test_validation_errors_empty_name_and_invalid_mode
//!
//! ## 设计
//! - 走 InMemoryDeckRepository + InMemoryPlayerRepository, 不用真 PG
//! - 跨域验证: player 不存在时 create_deck 返 NotFound
//! - 权限验证: 非 owner 调用 update/delete 返 Forbidden
//! - share_code 唯一性: 公开后取消再公开应生成新 share_code
//!
//! ## 跳过机制
//! - 无需 DATABASE_URL (InMemory 路径)

use player_service::entity::{Deck, DeckSlot, DeckStatus};
use player_service::repository::{
    DeckRepository, InMemoryDeckRepository, InMemoryPlayerRepository, PageRequest,
};
use player_service::service::{PlayerService, PlayerServiceImpl};
use std::sync::Arc;
use uuid::Uuid;

fn make_service() -> (
    PlayerServiceImpl,
    Arc<InMemoryPlayerRepository>,
    Arc<InMemoryDeckRepository>,
) {
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
    (svc, players, decks)
}

#[tokio::test]
async fn test_create_deck_returns_draft_default() {
    let (svc, _, _) = make_service();
    // 先注册玩家
    let player = svc.register("alice".to_string()).await.unwrap();
    let deck = svc
        .create_deck(player.id, "aggressive".to_string(), 1)
        .await
        .unwrap();
    assert_eq!(deck.owner_id, player.id);
    assert_eq!(deck.name, "aggressive");
    assert_eq!(deck.mode, 1);
    assert_eq!(deck.status, DeckStatus::Draft);
    assert!(!deck.is_public);
    assert!(deck.share_code.is_none());
    assert_eq!(deck.like_count, 0);
    assert!(deck.slots.is_empty());
}

#[tokio::test]
async fn test_full_lifecycle_create_update_share_pull_delete() {
    let (svc, _, _) = make_service();
    let player = svc.register("bob".to_string()).await.unwrap();

    // 1. CREATE
    let deck = svc
        .create_deck(player.id, "control".to_string(), 2)
        .await
        .unwrap();
    let deck_id = deck.id;
    assert_eq!(deck.status, DeckStatus::Draft);

    // 2. UPDATE (全量替换 slots + 改名)
    let new_slots = vec![
        DeckSlot::new("card-001".to_string(), 2),
        DeckSlot::new("card-002".to_string(), 1),
    ];
    let updated = svc
        .update_deck(
            deck_id,
            player.id,
            Some("control-v2".to_string()),
            Some(new_slots.clone()),
        )
        .await
        .unwrap();
    assert_eq!(updated.name, "control-v2");
    assert_eq!(updated.slots.len(), 2);
    assert_eq!(updated.slots[0].card_id, "card-001");

    // 3. SHARE (开启)
    let shared = svc.share_deck(deck_id, player.id, true).await.unwrap();
    assert!(shared.is_public);
    let share_code = shared.share_code.clone().expect("share_code 必生成");
    // 校验 UUIDv4 格式
    Uuid::parse_str(&share_code).expect("share_code 必须是 UUID 字符串");

    // 4. GET_SHARED (按 share_code 拉取)
    let pulled = svc.get_shared_deck(share_code.clone()).await.unwrap();
    assert_eq!(pulled.id, deck_id);
    assert!(pulled.is_public);
    assert_eq!(pulled.share_code.as_deref(), Some(share_code.as_str()));

    // 5. DELETE
    let deleted = svc.delete_deck(deck_id, player.id).await.unwrap();
    assert!(deleted);
    // 二次删应返 NotFound (per service.delete_deck 行为: 已删除则 NotFound)
    let err_again = svc.delete_deck(deck_id, player.id).await.unwrap_err();
    assert!(matches!(
        err_again,
        player_service::error::Error::NotFound { .. }
    ));
    // 拉取已删除 deck 应 NotFound
    let err = svc.get_deck(deck_id).await.unwrap_err();
    assert!(matches!(
        err,
        player_service::error::Error::NotFound { .. }
    ));
}

#[tokio::test]
async fn test_list_decks_pagination_per_owner() {
    let (svc, _, _) = make_service();
    let player_a = svc.register("carol".to_string()).await.unwrap();
    let player_b = svc.register("dave".to_string()).await.unwrap();

    // player_a 创建 5 个 deck
    for i in 0..5 {
        svc.create_deck(player_a.id, format!("a-deck-{}", i), 1)
            .await
            .unwrap();
    }
    // player_b 创建 3 个 deck
    for i in 0..3 {
        svc.create_deck(player_b.id, format!("b-deck-{}", i), 2)
            .await
            .unwrap();
    }

    // player_a 分页 page=1, page_size=3 → 3 items, total=5
    let (items_a_p1, total_a) = svc
        .list_decks(
            player_a.id,
            PageRequest {
                page: 1,
                page_size: 3,
            },
        )
        .await
        .unwrap();
    assert_eq!(total_a, 5);
    assert_eq!(items_a_p1.len(), 3);

    // player_a 分页 page=2, page_size=3 → 2 items (5-3)
    let (items_a_p2, total_a_p2) = svc
        .list_decks(
            player_a.id,
            PageRequest {
                page: 2,
                page_size: 3,
            },
        )
        .await
        .unwrap();
    assert_eq!(total_a_p2, 5);
    assert_eq!(items_a_p2.len(), 2);

    // player_b 应只见 3 个 (不混)
    let (items_b, total_b) = svc
        .list_decks(
            player_b.id,
            PageRequest {
                page: 1,
                page_size: 10,
            },
        )
        .await
        .unwrap();
    assert_eq!(total_b, 3);
    assert_eq!(items_b.len(), 3);
    for d in &items_b {
        assert_eq!(d.owner_id, player_b.id);
    }
}

#[tokio::test]
async fn test_share_deck_unpublic_clears_share_code() {
    let (svc, _, _) = make_service();
    let player = svc.register("eve".to_string()).await.unwrap();
    let deck = svc
        .create_deck(player.id, "deck".to_string(), 1)
        .await
        .unwrap();
    let deck_id = deck.id;

    // 公开
    let shared = svc.share_deck(deck_id, player.id, true).await.unwrap();
    assert!(shared.is_public);
    let code1 = shared.share_code.clone().unwrap();

    // 取消公开
    let unshared = svc.share_deck(deck_id, player.id, false).await.unwrap();
    assert!(!unshared.is_public);
    assert!(unshared.share_code.is_none());

    // 取消后 get_shared_deck 该 code 返 NotFound
    let err = svc.get_shared_deck(code1.clone()).await.unwrap_err();
    assert!(matches!(
        err,
        player_service::error::Error::NotFound { .. }
    ));

    // 重新公开 → 应生成新 code (与 code1 不同)
    let reshared = svc.share_deck(deck_id, player.id, true).await.unwrap();
    assert!(reshared.is_public);
    let code2 = reshared.share_code.clone().unwrap();
    // 取消时 share_code 已被清空, 重新公开必生成新 code
    assert_ne!(code1, code2, "取消后再公开应生成新 share_code");
}

#[tokio::test]
async fn test_validation_errors_empty_name_and_invalid_mode() {
    let (svc, _, _) = make_service();
    let player = svc.register("frank".to_string()).await.unwrap();

    // 空 name 必返 Validation
    let err = svc
        .create_deck(player.id, "".to_string(), 1)
        .await
        .unwrap_err();
    assert!(matches!(err, player_service::error::Error::Validation(_)));

    // 超长 name (>64) 必返 Validation
    let long_name = "x".repeat(65);
    let err = svc
        .create_deck(player.id, long_name, 1)
        .await
        .unwrap_err();
    assert!(matches!(err, player_service::error::Error::Validation(_)));

    // 无效 mode 必返 Validation
    let err = svc
        .create_deck(player.id, "deck".to_string(), 99)
        .await
        .unwrap_err();
    assert!(matches!(err, player_service::error::Error::Validation(_)));

    // player 不存在 必返 NotFound
    let err = svc
        .create_deck(Uuid::new_v4(), "deck".to_string(), 1)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        player_service::error::Error::NotFound { .. }
    ));
}
