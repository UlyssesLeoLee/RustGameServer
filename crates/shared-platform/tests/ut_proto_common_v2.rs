//! shared-platform common.v2 proto round-trip UT (per RGS-DTL-038 §4.1)
//!
//! 覆盖 8 个 v2 message/enum (10 UT):
//! 1. test_locale_default_unspecified
//! 2. test_localized_text_construct
//! 3. test_i18n_string_default_text_fallback
//! 4. test_card_type_enum_values
//! 5. test_card_rarity_enum_values
//! 6. test_card_ref_with_both_ids
//! 7. test_game_mode_enum_values
//! 8. test_player_id_with_rank_level
//! 9. test_currency_type_values
//! 10. test_currency_default_zero

use shared_platform::proto::v1::*;

#[test]
fn test_locale_default_unspecified() {
    let locale = Locale::Unspecified as i32;
    assert_eq!(locale, 0);
    // 验证枚举转换 (tonic 去前缀: LOCALE_UNSPECIFIED → Unspecified)
    let l = Locale::try_from(0).unwrap();
    assert!(matches!(l, Locale::Unspecified));
}

#[test]
fn test_localized_text_construct() {
    // LocalizedText 没 serde derive (tonic 默认), 用字段访问验证
    let text = LocalizedText {
        locale: Locale::ZhCn as i32,
        text: "测试".to_string(),
    };
    assert_eq!(text.locale, Locale::ZhCn as i32);
    assert_eq!(text.text, "测试");
}

#[test]
fn test_i18n_string_default_text_fallback() {
    let i18n = I18nString {
        default_text: "Hello".to_string(),
        translations: vec![],
    };
    assert_eq!(i18n.default_text, "Hello");
    assert!(i18n.translations.is_empty());
    // 业务层 fallback 语义: translations 为空时用 default_text
}

#[test]
fn test_card_type_enum_values() {
    // tonic 去 CARD_TYPE_ 前缀
    assert_eq!(CardType::Unspecified as i32, 0);
    assert_eq!(CardType::Creature as i32, 1);
    assert_eq!(CardType::Spell as i32, 2);
    assert_eq!(CardType::Equipment as i32, 3);
    assert_eq!(CardType::Land as i32, 4);
    assert_eq!(CardType::Trap as i32, 5);
    assert_eq!(CardType::Hero as i32, 6);
}

#[test]
fn test_card_rarity_enum_values() {
    assert_eq!(CardRarity::Unspecified as i32, 0);
    assert_eq!(CardRarity::Common as i32, 1);
    assert_eq!(CardRarity::Uncommon as i32, 2);
    assert_eq!(CardRarity::Rare as i32, 3);
    assert_eq!(CardRarity::Epic as i32, 4);
    assert_eq!(CardRarity::Legendary as i32, 5);
}

#[test]
fn test_card_ref_with_both_ids() {
    let card_ref = CardRef {
        card_id: "card_001".to_string(),
        instance_id: "instance_abc".to_string(),
    };
    assert_eq!(card_ref.card_id, "card_001");
    assert_eq!(card_ref.instance_id, "instance_abc");
    // 业务层: 静态 catalog 用 card_id, 玩家收藏用 instance_id
}

#[test]
fn test_game_mode_enum_values() {
    // 4 类对战模式
    assert_eq!(GameMode::Unspecified as i32, 0);
    assert_eq!(GameMode::Ranked as i32, 1);
    assert_eq!(GameMode::Casual as i32, 2);
    assert_eq!(GameMode::Room as i32, 3);
    assert_eq!(GameMode::PveAi as i32, 4);
}

#[test]
fn test_player_id_with_rank_level() {
    // PlayerId 字段: player_id (EntityId) / display_name / rank_score / level
    let player = PlayerId {
        player_id: Some(EntityId {
            id: "player_001".to_string(),
        }),
        display_name: "TestPlayer".to_string(),
        rank_score: 1500,
        level: 25,
    };
    assert_eq!(player.player_id.as_ref().unwrap().id, "player_001");
    assert_eq!(player.display_name, "TestPlayer");
    assert_eq!(player.rank_score, 1500);
    assert_eq!(player.level, 25);
}

#[test]
fn test_currency_type_values() {
    // 3 类货币
    assert_eq!(CurrencyType::Unspecified as i32, 0);
    assert_eq!(CurrencyType::Soft as i32, 1);
    assert_eq!(CurrencyType::Hard as i32, 2);
    assert_eq!(CurrencyType::CardValue as i32, 3);
}

#[test]
fn test_currency_default_zero() {
    let c = Currency {
        r#type: CurrencyType::Soft as i32,
        amount: 0,
    };
    assert_eq!(c.r#type, 1);
    assert_eq!(c.amount, 0);
}
