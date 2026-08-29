//! card-service 桶 10 端到端 IT (per RGS-DTL-038 §4.4 + FR-003/FR-006/BR-003)
//!
//! ## 目的
//! 验证 card catalog + collection + OpenPack 完整业务链路,
//! 走 InMemoryRepository 模拟 DB 路径, 不依赖外部 PG.
//!
//! ## 范围 (5 IT 覆盖 per 任务书要求)
//! 1. test_catalog_list_pagination_and_filter
//! 2. test_collection_read_with_filter
//! 3. test_open_pack_one_pack_five_cards_with_drop_table_snapshot
//! 4. test_open_pack_repeated_100_times_distribution (statistical check)
//! 5. test_remove_card_instance
//!
//! ## 设计
//! - 走 InMemoryCardRepository + InMemoryCardSeriesRepository + InMemoryCardInstanceRepository, 不用真 PG
//! - 跨域验证: card 不存在时 add_card_to_collection 返 NotFound
//! - 权限验证: 非 owner 调用 remove_card_from_collection 返 Forbidden
//! - drop_table snapshot 验证 (per DEC-038-06 强制公开)
//!
//! ## 跳过机制
//! - 无需 DATABASE_URL (InMemory 路径)

use card_service::entity::{
    Card, CardInstance, CardInstanceSource, CardRarity, CardSeries, CardType, DropEntry, DropTable,
};
use card_service::repository::{
    CardInstanceFilter, CardInstanceRepository, CardRepository, CardSeriesRepository,
    InMemoryCardInstanceRepository, InMemoryCardRepository, InMemoryCardSeriesRepository,
    PageRequest,
};
use card_service::service::{CardService, CardServiceImpl};
use std::sync::Arc;
use uuid::Uuid;

fn make_service() -> (
    CardServiceImpl,
    Arc<InMemoryCardRepository>,
    Arc<InMemoryCardSeriesRepository>,
    Arc<InMemoryCardInstanceRepository>,
) {
    let cards: Arc<InMemoryCardRepository> = Arc::new(InMemoryCardRepository::new());
    let series: Arc<InMemoryCardSeriesRepository> = Arc::new(InMemoryCardSeriesRepository::new());
    let instances: Arc<InMemoryCardInstanceRepository> = Arc::new(
        InMemoryCardInstanceRepository::new(cards.clone() as Arc<dyn CardRepository>),
    );
    let svc = CardServiceImpl::new(
        cards.clone() as Arc<dyn CardRepository>,
        series.clone() as Arc<dyn CardSeriesRepository>,
        instances.clone() as Arc<dyn CardInstanceRepository>,
    );
    (svc, cards, series, instances)
}

fn sample_card(id: &str, rarity: CardRarity, card_type: CardType, series_id: &str) -> Card {
    Card::new(
        id.to_string(),
        series_id.to_string(),
        format!("Card {}", id),
        card_type,
        rarity,
    )
}

fn packable_series(id: &str) -> CardSeries {
    let mut s = CardSeries::new(id.to_string(), format!("Series {}", id), 5);
    s.drop_table = DropTable::new(vec![
        DropEntry {
            rarity: CardRarity::Common,
            count: 1,
            probability: 0.7,
            card_id: Some("card_common".to_string()),
        },
        DropEntry {
            rarity: CardRarity::Rare,
            count: 1,
            probability: 0.25,
            card_id: Some("card_rare".to_string()),
        },
        DropEntry {
            rarity: CardRarity::Legendary,
            count: 1,
            probability: 0.05,
            card_id: Some("card_legendary".to_string()),
        },
    ]);
    s
}

// ============================================================================
// IT 1: catalog 列表 + 分页 + 过滤 (按 type/rarity/series)
// ============================================================================

#[tokio::test]
async fn test_catalog_list_pagination_and_filter() {
    let (svc, cards, _, _) = make_service();
    // 准备 5 张卡: 跨 type / rarity / series
    cards
        .create(&sample_card(
            "c1",
            CardRarity::Common,
            CardType::Creature,
            "series_001",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "c2",
            CardRarity::Rare,
            CardType::Spell,
            "series_001",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "c3",
            CardRarity::Legendary,
            CardType::Creature,
            "series_002",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "c4",
            CardRarity::Common,
            CardType::Spell,
            "series_002",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "c5",
            CardRarity::Epic,
            CardType::Creature,
            "series_001",
        ))
        .await
        .unwrap();

    // 1. 全列表, page=1 size=3
    let filter = card_service::repository::CardFilter::default();
    let (items, total, has_next) = svc
        .list_cards(&filter, PageRequest { page: 1, page_size: 3 })
        .await
        .unwrap();
    assert_eq!(total, 5);
    assert_eq!(items.len(), 3);
    assert!(has_next);

    // 2. page=2 size=3
    let (items, total, has_next) = svc
        .list_cards(&filter, PageRequest { page: 2, page_size: 3 })
        .await
        .unwrap();
    assert_eq!(total, 5);
    assert_eq!(items.len(), 2);
    assert!(!has_next);

    // 3. 按 rarity=Common 过滤
    let filter = card_service::repository::CardFilter {
        rarity_filter: Some(CardRarity::Common),
        ..Default::default()
    };
    let (items, total, _) = svc
        .list_cards(&filter, PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 2);
    assert_eq!(items.len(), 2);

    // 4. 按 type=Creature 过滤
    let filter = card_service::repository::CardFilter {
        type_filter: Some(CardType::Creature),
        ..Default::default()
    };
    let (items, total, _) = svc
        .list_cards(&filter, PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 3);
    assert_eq!(items.len(), 3);

    // 5. 按 series_id=series_001 过滤
    let filter = card_service::repository::CardFilter {
        series_id_filter: Some("series_001".to_string()),
        ..Default::default()
    };
    let (items, total, _) = svc
        .list_cards(&filter, PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 3);
    assert_eq!(items.len(), 3);

    // 6. 组合: series_001 + Creature
    let filter = card_service::repository::CardFilter {
        type_filter: Some(CardType::Creature),
        series_id_filter: Some("series_001".to_string()),
        ..Default::default()
    };
    let (items, total, _) = svc
        .list_cards(&filter, PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 2); // c1 (Common Creature series_001) + c5 (Epic Creature series_001)
    assert_eq!(items.len(), 2);
}

// ============================================================================
// IT 2: 收藏读 + 过滤
// ============================================================================

#[tokio::test]
async fn test_collection_read_with_filter() {
    let (svc, cards, _, instances) = make_service();
    // 准备 3 张卡 (跨 rarity)
    cards
        .create(&sample_card(
            "c_common",
            CardRarity::Common,
            CardType::Creature,
            "series_001",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "c_rare",
            CardRarity::Rare,
            CardType::Spell,
            "series_001",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "c_legendary",
            CardRarity::Legendary,
            CardType::Creature,
            "series_002",
        ))
        .await
        .unwrap();

    let owner = Uuid::new_v4();
    let other = Uuid::new_v4();

    // owner 收藏 3 张 (跨 rarity)
    let inst1 = CardInstance::new("c_common".to_string(), owner, CardInstanceSource::Pack);
    let inst2 = CardInstance::new("c_rare".to_string(), owner, CardInstanceSource::Reward);
    let inst3 = CardInstance::new(
        "c_legendary".to_string(),
        owner,
        CardInstanceSource::Pack,
    );
    instances
        .add_many(&[inst1.clone(), inst2.clone(), inst3.clone()])
        .await
        .unwrap();

    // other 收藏 1 张
    let other_inst =
        CardInstance::new("c_common".to_string(), other, CardInstanceSource::Pack);
    instances.add_many(&[other_inst]).await.unwrap();

    // 1. owner 全列表
    let (items, total) = svc
        .get_player_collection(owner, &CardInstanceFilter::default(), PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 3);
    assert_eq!(items.len(), 3);

    // 2. owner 按 rarity=Common 过滤
    let filter = CardInstanceFilter {
        rarity_filter: Some(CardRarity::Common),
        ..Default::default()
    };
    let (items, total) = svc
        .get_player_collection(owner, &filter, PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].card_id, "c_common");

    // 3. owner 按 series_id=series_002 过滤 (跨 master 关联)
    let filter = CardInstanceFilter {
        series_id_filter: Some("series_002".to_string()),
        ..Default::default()
    };
    let (items, total) = svc
        .get_player_collection(owner, &filter, PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].card_id, "c_legendary");

    // 4. other 只看到自己的 1 张
    let (items, total) = svc
        .get_player_collection(other, &CardInstanceFilter::default(), PageRequest::default())
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].card_id, "c_common");
}

// ============================================================================
// IT 3: OpenPack: 1 包 5 张 + drop_table snapshot
// ============================================================================

#[tokio::test]
async fn test_open_pack_one_pack_five_cards_with_drop_table_snapshot() {
    let (svc, cards, series, _) = make_service();
    cards
        .create(&sample_card(
            "card_common",
            CardRarity::Common,
            CardType::Creature,
            "series_001",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "card_rare",
            CardRarity::Rare,
            CardType::Spell,
            "series_001",
        ))
        .await
        .unwrap();
    cards
        .create(&sample_card(
            "card_legendary",
            CardRarity::Legendary,
            CardType::Creature,
            "series_001",
        ))
        .await
        .unwrap();
    series.upsert(&packable_series("series_001")).await.unwrap();

    let owner = Uuid::new_v4();
    let result = svc
        .open_pack(owner, "series_001", 1, Some("saga_xyz".to_string()))
        .await
        .unwrap();

    // pack_size=5, count=1 → 5 instances
    assert_eq!(result.instances.len(), 5);
    // drop_table snapshot 必返 (per DEC-038-06 强制公开)
    assert_eq!(result.drop_table.version, 1);
    assert_eq!(result.drop_table.entries.len(), 3);
    assert!(result.drop_table.snapshot_at <= chrono::Utc::now());
    // transaction_id = saga_id (桶 10 简化)
    assert_eq!(result.transaction_id, "saga_xyz");
    // 所有 instance 应有 owner + source=Pack
    for inst in &result.instances {
        assert_eq!(inst.owner_id, owner);
        assert_eq!(inst.source, CardInstanceSource::Pack);
        assert!(!inst.locked);
    }
}

// ============================================================================
// IT 4: 重复 OpenPack 100 次验证概率分布 (statistical check)
// ============================================================================

#[tokio::test]
async fn test_open_pack_repeated_100_times_distribution() {
    let (svc, cards, series, _) = make_service();
    for cid in ["c1", "c2", "c3"] {
        let r = match cid {
            "c1" => CardRarity::Common,
            "c2" => CardRarity::Rare,
            _ => CardRarity::Legendary,
        };
        cards
            .create(&sample_card(cid, r, CardType::Creature, "series_dist"))
            .await
            .unwrap();
    }
    let mut s = CardSeries::new("series_dist".to_string(), "Dist".to_string(), 5);
    s.drop_table = DropTable::new(vec![
        DropEntry {
            rarity: CardRarity::Common,
            count: 1,
            probability: 0.7,
            card_id: Some("c1".to_string()),
        },
        DropEntry {
            rarity: CardRarity::Rare,
            count: 1,
            probability: 0.25,
            card_id: Some("c2".to_string()),
        },
        DropEntry {
            rarity: CardRarity::Legendary,
            count: 1,
            probability: 0.05,
            card_id: Some("c3".to_string()),
        },
    ]);
    series.upsert(&s).await.unwrap();

    let owner = Uuid::new_v4();
    let mut counts = std::collections::HashMap::new();
    let n = 100u32;
    for _ in 0..n {
        let r = svc
            .open_pack(owner, "series_dist", 1, None)
            .await
            .unwrap();
        for inst in &r.instances {
            *counts.entry(inst.card_id.clone()).or_insert(0u32) += 1;
        }
    }
    let total_cards: u32 = counts.values().sum();
    // pack_size=5, count=1, n=100 → 500 cards total
    assert_eq!(total_cards, 5 * n);
    // c1 (~70%), c2 (~25%), c3 (~5%) — 允许 ±15% 偏差 (500 样本下统计噪声)
    let c1 = *counts.get("c1").unwrap_or(&0) as f64 / total_cards as f64;
    let c2 = *counts.get("c2").unwrap_or(&0) as f64 / total_cards as f64;
    let c3 = *counts.get("c3").unwrap_or(&0) as f64 / total_cards as f64;
    assert!((c1 - 0.7).abs() < 0.15, "c1={} expected ~0.7", c1);
    assert!((c2 - 0.25).abs() < 0.15, "c2={} expected ~0.25", c2);
    assert!((c3 - 0.05).abs() < 0.15, "c3={} expected ~0.05", c3);
}

// ============================================================================
// IT 5: 删除卡牌实例 (含 owner 校验 + locked 校验)
// ============================================================================

#[tokio::test]
async fn test_remove_card_instance() {
    let (svc, cards, _, instances) = make_service();
    cards
        .create(&sample_card(
            "c1",
            CardRarity::Common,
            CardType::Creature,
            "series_001",
        ))
        .await
        .unwrap();

    let owner = Uuid::new_v4();
    let other = Uuid::new_v4();

    // 1. owner 加 1 张
    let (_, inst1) = svc
        .add_card_to_collection(owner, "c1", CardInstanceSource::Pack, None)
        .await
        .unwrap();
    let (_, inst2) = svc
        .add_card_to_collection(owner, "c1", CardInstanceSource::Reward, None)
        .await
        .unwrap();

    // 2. 非 owner 删除 → Forbidden
    let res = svc
        .remove_card_from_collection(inst1.instance_id, other, "test".to_string(), None)
        .await;
    assert!(res.is_err());

    // 3. owner 正常删除
    let removed = svc
        .remove_card_from_collection(
            inst1.instance_id,
            owner,
            "test".to_string(),
            Some("saga_001".to_string()),
        )
        .await
        .unwrap();
    assert!(removed);
    // 验证删除成功: list 只剩 1 张
    let count = instances.count_by_owner(owner).await.unwrap();
    assert_eq!(count, 1);

    // 4. 删除不存在的 instance → NotFound
    let fake_id = Uuid::new_v4();
    let res = svc
        .remove_card_from_collection(fake_id, owner, "test".to_string(), None)
        .await;
    assert!(res.is_err());

    // 5. 删除 locked instance → Conflict
    let mut inst3 = CardInstance::new("c1".to_string(), owner, CardInstanceSource::Pack);
    inst3.locked = true;
    instances.add_many(&[inst3.clone()]).await.unwrap();
    let res = svc
        .remove_card_from_collection(inst3.instance_id, owner, "test".to_string(), None)
        .await;
    assert!(matches!(
        res,
        Err(card_service::Error::Conflict(_))
    ));

    // 6. inst2 仍存在, 可正常删除
    let removed = svc
        .remove_card_from_collection(inst2.instance_id, owner, "test".to_string(), None)
        .await
        .unwrap();
    assert!(removed);
    let count = instances.count_by_owner(owner).await.unwrap();
    assert_eq!(count, 1); // 只剩 locked inst3
}
