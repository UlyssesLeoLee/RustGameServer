//! 卡组分享完整生命周期 IT (per RGS IT-AGENT-BRIEFING v1 §3.1 / DTL-038 §4.3 FR-002)
//!
//! ## 目的
//! 验证 deck share 5 步业务链路的端到端正确性:
//!   owner 创 deck → share → 另一 player 通过 share_code 拉取 → owner 取消 share → 再次拉 fail.
//! 这是 deck 业务"公开 → 拉取 → 撤回"完整闭环的最高规格 IT 覆盖, 对应 DTL-038 §4.3 FR-002
//! "卡组分享拉取"的业务定义.
//!
//! ## 范围 (4 IT 覆盖 5 步主链 + 边界)
//! 1. test_full_share_lifecycle_5_steps — 主链: create → share → 另一 player 拉 → unshare → 拉 fail
//! 2. test_share_cycles_generate_distinct_codes — 同一 deck 反复 share 应生成新 share_code
//! 3. test_share_deck_by_non_owner_fails — 权限: 非 owner share 必须 Forbidden
//! 4. test_get_shared_deck_with_unknown_code_fails — 拉不存在的 code 必须 NotFound
//!
//! ## 设计
//! - 走 InMemoryDeckRepository + InMemoryPlayerRepository, 不用真 PG
//! - 跨玩家模拟: 同一 service 实例 (Arc<InMemory*Repository>) 注入两个玩家账号
//! - share_code UUIDv4 格式校验 (per service.share_deck 行为)
//!
//! ## 跳过机制
//! - 无需 DATABASE_URL (InMemory 路径)

use player_service::entity::DeckStatus;
use player_service::repository::{
    DeckRepository, InMemoryDeckRepository, InMemoryPlayerRepository,
    InMemoryPlayerSessionRepository,
};
use player_service::service::{PlayerService, PlayerServiceImpl};
use std::sync::Arc;
use uuid::Uuid;

/// 构造带三 InMemory repo 的 PlayerServiceImpl (与 service.rs::make_service 同样式)
fn make_service() -> (
    PlayerServiceImpl,
    Arc<InMemoryPlayerRepository>,
    Arc<InMemoryPlayerSessionRepository>,
    Arc<InMemoryDeckRepository>,
) {
    let players = Arc::new(InMemoryPlayerRepository::new());
    let sessions = Arc::new(InMemoryPlayerSessionRepository::new());
    let decks = Arc::new(InMemoryDeckRepository::new());
    let svc = PlayerServiceImpl::new(
        players.clone() as Arc<dyn player_service::repository::PlayerRepository>,
        sessions.clone() as Arc<dyn player_service::repository::PlayerSessionRepository>,
        decks.clone() as Arc<dyn DeckRepository>,
    );
    (svc, players, sessions, decks)
}

/// 主链: 5 步完整生命周期
/// 1) owner 创 deck
/// 2) owner share (public=true) — 生成 share_code
/// 3) 另一 player 通过 share_code 拉取 (跨玩家访问)
/// 4) owner 取消 share (public=false) — 清空 share_code
/// 5) 拉取应 fail (NotFound)
#[tokio::test]
async fn test_full_share_lifecycle_5_steps() {
    let (svc, _players, _sessions, _decks) = make_service();

    // 1) owner 创 deck
    let owner = svc.register("owner-alice".to_string()).await.unwrap();
    let _viewer = svc.register("viewer-bob".to_string()).await.unwrap();
    let deck = svc
        .create_deck(owner.id, "aggro-rush".to_string(), 1)
        .await
        .unwrap();
    let deck_id = deck.id;
    assert_eq!(deck.status, DeckStatus::Draft);
    assert!(!deck.is_public, "新建 deck 默认不公开");
    assert!(deck.share_code.is_none(), "新建 deck 应无 share_code");

    // 2) owner share (开启)
    let shared = svc.share_deck(deck_id, owner.id, true).await.unwrap();
    assert!(shared.is_public, "share 后 is_public=true");
    let share_code = shared
        .share_code
        .clone()
        .expect("share 后 share_code 必生成");
    // share_code 必须是 UUIDv4 字符串
    Uuid::parse_str(&share_code).expect("share_code 必须是 UUID 字符串");

    // 3) 另一 player (viewer) 通过 share_code 拉取 — 跨玩家访问必须成功
    let pulled_by_viewer = svc.get_shared_deck(share_code.clone()).await.unwrap();
    assert_eq!(pulled_by_viewer.id, deck_id, "viewer 拉到的 deck id 必须匹配");
    assert_eq!(pulled_by_viewer.owner_id, owner.id, "deck owner 必须保留");
    assert!(pulled_by_viewer.is_public);
    assert_eq!(
        pulled_by_viewer.share_code.as_deref(),
        Some(share_code.as_str()),
        "viewer 拉到的 share_code 必须一致"
    );

    // 4) owner 取消 share (public=false) — 清空 share_code
    let unshared = svc.share_deck(deck_id, owner.id, false).await.unwrap();
    assert!(!unshared.is_public, "取消后 is_public=false");
    assert!(unshared.share_code.is_none(), "取消后 share_code 必清空");

    // 5) 再次拉取应 fail (原 share_code 已废弃, NotFound)
    let err = svc.get_shared_deck(share_code.clone()).await.unwrap_err();
    assert!(
        matches!(err, player_service::error::Error::NotFound { .. }),
        "取消 share 后原 code 拉取必须 NotFound, got: {:?}",
        err
    );

    // 旁证: deck 仍存在 (取消 share 不删 deck), 拉取 by id 仍 OK
    let still_exists = svc.get_deck(deck_id).await.unwrap();
    assert_eq!(still_exists.id, deck_id);
    assert!(!still_exists.is_public, "取消后 deck 仍存在但私有");
}

/// 同一 deck 反复 share → unshare → share 应生成不同 share_code
/// (per service.share_deck: make_public=true 且 share_code=None 时才新生成, 重新 share 前已 None)
#[tokio::test]
async fn test_share_cycles_generate_distinct_codes() {
    let (svc, _, _, _) = make_service();
    let owner = svc.register("cycler".to_string()).await.unwrap();
    let deck = svc
        .create_deck(owner.id, "cycle-deck".to_string(), 2)
        .await
        .unwrap();
    let deck_id = deck.id;

    // 第 1 轮: share
    let s1 = svc.share_deck(deck_id, owner.id, true).await.unwrap();
    let code1 = s1.share_code.clone().unwrap();
    // 第 1 轮: unshare
    svc.share_deck(deck_id, owner.id, false).await.unwrap();
    // 第 2 轮: 重新 share
    let s2 = svc.share_deck(deck_id, owner.id, true).await.unwrap();
    let code2 = s2.share_code.clone().unwrap();
    // 反复 share 应生成不同 code (取消时清空, 重新 share 必新建)
    assert_ne!(code1, code2, "再次 share 应生成新 share_code");
    // 老 code 在第 1 轮 unshare 后已失效
    let err_old = svc.get_shared_deck(code1).await.unwrap_err();
    assert!(matches!(
        err_old,
        player_service::error::Error::NotFound { .. }
    ));
}

/// 权限边界: 非 owner 调 share_deck 必须 Forbidden
#[tokio::test]
async fn test_share_deck_by_non_owner_fails() {
    let (svc, _, _, _) = make_service();
    let owner = svc.register("real-owner".to_string()).await.unwrap();
    let attacker = svc.register("attacker".to_string()).await.unwrap();
    let deck = svc
        .create_deck(owner.id, "private".to_string(), 1)
        .await
        .unwrap();
    let deck_id = deck.id;

    // attacker 试图 share owner 的 deck
    let err = svc
        .share_deck(deck_id, attacker.id, true)
        .await
        .unwrap_err();
    assert!(
        matches!(err, player_service::error::Error::Forbidden(_)),
        "非 owner share 必须 Forbidden, got: {:?}",
        err
    );

    // 旁证: attacker 试图 unshare 也必须 Forbidden
    let err2 = svc
        .share_deck(deck_id, attacker.id, false)
        .await
        .unwrap_err();
    assert!(
        matches!(err2, player_service::error::Error::Forbidden(_)),
        "非 owner unshare 必须 Forbidden, got: {:?}",
        err2
    );

    // 旁证: deck 状态未被 attacker 改动
    let still = svc.get_deck(deck_id).await.unwrap();
    assert!(!still.is_public, "attacker 攻击后 deck 仍私有");
    assert!(still.share_code.is_none());
}

/// 拉取不存在的 share_code 必须 NotFound
#[tokio::test]
async fn test_get_shared_deck_with_unknown_code_fails() {
    let (svc, _, _, _) = make_service();
    // 完全不存在的 UUID 字符串
    let fake_code = Uuid::new_v4().to_string();
    let err = svc.get_shared_deck(fake_code).await.unwrap_err();
    assert!(
        matches!(err, player_service::error::Error::NotFound { .. }),
        "不存在 code 必须 NotFound, got: {:?}",
        err
    );

    // 空字符串也必须 NotFound 或 Validation
    let err_empty = svc.get_shared_deck("".to_string()).await.unwrap_err();
    assert!(
        matches!(
            err_empty,
            player_service::error::Error::NotFound { .. }
                | player_service::error::Error::Validation(_)
        ),
        "空 code 必须 NotFound 或 Validation, got: {:?}",
        err_empty
    );
}
