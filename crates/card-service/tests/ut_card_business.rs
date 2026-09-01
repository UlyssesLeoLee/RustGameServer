//! card-service 域业务函数单元测试 (per PT-WORKER-BRIEFING §2 必做)
//!
//! 5 UT 覆盖:
//! 1. card_rarity_proto_roundtrip (proto int32 <-> enum 双向)
//! 2. card_series_status_packable (Ok 可抽 / 其他不可)
//! 3. drop_table_validate_ok_and_sum_le_one (业务规则: probability 之和 ≤ 1.0)
//! 4. card_instance_ensure_removable_locked (locked → Conflict)
//! 5. card_stats_custom_attrs (自定义属性 map)

use card_service::entity::{
    Card, CardInstance, CardInstanceSource, CardRarity, CardSeries, CardSeriesStatus, CardStats,
    CardType, DropEntry, DropTable,
};
use uuid::Uuid;

#[test]
fn card_rarity_proto_roundtrip() {
    for r in [
        CardRarity::Common,
        CardRarity::Uncommon,
        CardRarity::Rare,
        CardRarity::Epic,
        CardRarity::Legendary,
    ] {
        assert_eq!(CardRarity::from_i32(r.as_i32()), r);
    }
    assert_eq!(CardRarity::from_i32(99), CardRarity::Unspecified);
}

#[test]
fn card_series_status_packable() {
    assert!(CardSeriesStatus::Ok.is_packable());
    assert!(!CardSeriesStatus::Pending.is_packable());
    assert!(!CardSeriesStatus::Failed.is_packable());
    assert!(!CardSeriesStatus::Cancelled.is_packable());
}

#[test]
fn drop_table_validate_ok_and_sum_le_one() {
    let dt = DropTable::new(vec![
        DropEntry { rarity: CardRarity::Common, count: 4, probability: 0.7, card_id: None },
        DropEntry { rarity: CardRarity::Rare, count: 1, probability: 0.2, card_id: None },
        DropEntry { rarity: CardRarity::Legendary, count: 1, probability: 0.05, card_id: None },
    ]);
    assert!(dt.validate().is_ok());
    // 0.7 + 0.2 + 0.05 = 0.95 ≤ 1.0
    let sum: f64 = dt.entries.iter().map(|e| e.probability).sum();
    assert!(sum <= 1.0);
}

#[test]
fn card_instance_ensure_removable_locked() {
    let mut i = CardInstance::new("c1".to_string(), Uuid::new_v4(), CardInstanceSource::Pack);
    assert!(i.ensure_removable().is_ok());
    i.locked = true;
    assert!(i.ensure_removable().is_err());
}

#[test]
fn card_stats_custom_attrs() {
    let mut s = CardStats::default();
    s.custom.insert("taunt".to_string(), 1);
    s.custom.insert("shield".to_string(), 3);
    let c = Card::new("c".into(), "s".into(), "x".into(), CardType::Creature, CardRarity::Common);
    assert_eq!(c.stats.attack, 0);
    assert_eq!(s.custom.get("taunt"), Some(&1));
}

#[test]
fn card_series_new_default_status_is_ok() {
    let s = CardSeries::new("s1".into(), "Starter".into(), 5);
    assert_eq!(s.series_id, "s1");
    assert_eq!(s.pack_size, 5);
    assert_eq!(s.status, CardSeriesStatus::Ok);
    assert!(s.drop_table.entries.is_empty());
}
