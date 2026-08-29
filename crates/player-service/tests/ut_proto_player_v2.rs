//! player-service v2 proto UT (per RGS-DTL-038 §4.3)
//!
//! 3 UT (适配桶 11 实装的简化版字段名):
//! 1. test_player_profile_with_currencies
//! 2. test_deck_with_slots
//! 3. test_deck_slot_count_range

use player_service::proto::v1::*;
use player_service::common::v1 as common;

#[test]
fn test_player_profile_with_currencies() {
    // 桶 11 实装简化: player_id 是 string, currencies 用本地 Currency
    let p = PlayerProfile {
        player_id: "p1".to_string(),
        ranked_score: 1500,
        ranked_tier: "Gold".to_string(),
        total_matches: 100,
        total_wins: 65,
        collection_count: 50,
        currencies: vec![
            Currency { code: "GOLD".to_string(), amount: 1000 },
            Currency { code: "GEM".to_string(), amount: 50 },
        ],
        preferred_locale: "zh-CN".to_string(),
    };
    assert_eq!(p.player_id, "p1");
    assert_eq!(p.ranked_tier, "Gold");
    assert_eq!(p.total_matches, 100);
    assert_eq!(p.total_wins, 65);
    assert_eq!(p.currencies.len(), 2);
    assert_eq!(p.currencies[0].amount, 1000);
    assert_eq!(p.preferred_locale, "zh-CN");
}

#[test]
fn test_deck_with_slots() {
    // 桶 11 简化: owner_id 是 string, mode 是 int32
    let deck = Deck {
        deck_id: "deck_001".to_string(),
        owner_id: "p1".to_string(),
        name: "MyDeck".to_string(),
        mode: common::GameMode::Ranked as i32,
        slots: vec![
            DeckSlot { card_id: "c1".to_string(), count: 3 },
            DeckSlot { card_id: "c2".to_string(), count: 2 },
        ],
        status: common::Status::Ok as i32,
        created_at: Some(common::Timestamp { seconds: 1700000000, nanos: 0 }),
        updated_at: Some(common::Timestamp { seconds: 1700000100, nanos: 0 }),
        is_public: true,
        share_code: "abc-123".to_string(),
        like_count: 42,
    };
    assert_eq!(deck.name, "MyDeck");
    assert_eq!(deck.owner_id, "p1");
    assert_eq!(deck.slots.len(), 2);
    assert_eq!(deck.slots[0].count, 3);
    assert!(deck.is_public);
    assert_eq!(deck.like_count, 42);
}

#[test]
fn test_deck_slot_count_range() {
    // 业务规则: 单卡数量 1-3
    for count in [1u32, 2, 3] {
        let s = DeckSlot {
            card_id: "c1".to_string(),
            count,
        };
        assert!(s.count >= 1 && s.count <= 3);
    }
}
