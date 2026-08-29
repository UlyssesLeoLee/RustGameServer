//! card-service v1 proto UT (per RGS-DTL-038 §4.4)
//!
//! 5 UT (简化版, 验证核心 message/enum 不依赖嵌套 enum):
//! 1. test_card_basic_fields
//! 2. test_card_stats
//! 3. test_drop_table_version
//! 4. test_card_series_pack_size
//! 5. test_open_pack_response_transaction_id

use card_service::proto::v1::*;
use card_service::common::v1 as common;

#[test]
fn test_card_basic_fields() {
    let c = Card {
        card_id: "card_001".to_string(),
        name: Some(common::I18nString {
            default_text: "Fire Dragon".to_string(),
            translations: vec![],
        }),
        r#type: common::CardType::Creature as i32,
        rarity: common::CardRarity::Legendary as i32,
        series_id: "series_001".to_string(),
        base_cost: 7,
        description: None,
        effect_ref: "damage_5".to_string(),
        stats: None,
    };
    assert_eq!(c.card_id, "card_001");
    assert_eq!(c.rarity, common::CardRarity::Legendary as i32);
    assert_eq!(c.base_cost, 7);
    assert_eq!(c.r#type, common::CardType::Creature as i32);
}

#[test]
fn test_card_stats() {
    let stats = CardStats {
        attack: 8,
        health: 6,
        mana: 0,
        custom: Default::default(),
    };
    assert_eq!(stats.attack, 8);
    assert_eq!(stats.health, 6);
    assert_eq!(stats.mana, 0);
}

#[test]
fn test_drop_table_version() {
    let dt = DropTable {
        version: 1,
        snapshot_at: Some(common::Timestamp { seconds: 1700000000, nanos: 0 }),
        entries: vec![],
    };
    assert_eq!(dt.version, 1);
    // 业务规则: 每次调整递增
    let dt2 = DropTable {
        version: 2,
        snapshot_at: None,
        entries: vec![],
    };
    assert!(dt2.version > dt.version);
}

#[test]
fn test_card_series_pack_size() {
    let series = CardSeries {
        series_id: "series_001".to_string(),
        name: Some(common::I18nString {
            default_text: "Starter Set".to_string(),
            translations: vec![],
        }),
        pack_size: 5,
        drop_table: None,
        price: None,
        released_at: None,
        status: common::Status::Ok as i32,
    };
    assert_eq!(series.pack_size, 5);
    assert_eq!(series.series_id, "series_001");
    assert_eq!(series.status, common::Status::Ok as i32);
}

#[test]
fn test_open_pack_response_transaction_id() {
    let resp = OpenPackResponse {
        instances: vec![],
        drop_table: None,
        transaction_id: "tx_001".to_string(),
    };
    assert_eq!(resp.transaction_id, "tx_001");
    // 业务层: 每次 OpenPack 返 drop_table 快照 (per DEC-038-06 强制公开)
    // 此 UT 只验证 transaction_id, drop_table 字段保留
}
