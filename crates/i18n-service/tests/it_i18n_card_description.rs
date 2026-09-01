//! i18n-service 跨模块 IT 场景 (per PT-WORKER-BRIEFING §2)
//!
//! 3 IT 覆盖 (跨模块 = i18n + card description 契约, 模拟 card-service 调用 i18n):
//! 1. test_card_description_i18n_chain (card 描述 key 跨 locale 解析)
//! 2. test_card_description_partial_fallback (部分 locale 缺失, fallback chain)
//! 3. test_card_description_gettexts_batch_initialization (客户端初始化批量拉取)

use std::sync::Arc;
use std::time::Duration;
use i18n_service::entity::{I18nText, Locale};
use i18n_service::repository::{I18nRepository, InMemoryI18nRepository};
use i18n_service::service::{GetTextResult, I18nService, I18nServiceImpl};

/// 模拟 card-service 提供的 description key 命名: "card.{card_id}.description"
/// 与 i18n-service 约定的 key 格式一致
async fn make_service_with_card_descriptions() -> I18nServiceImpl {
    let repo: Arc<dyn I18nRepository> = Arc::new(
        InMemoryI18nRepository::new().with_texts(vec![
            I18nText::new("card.fire_dragon.description".into(), Locale::ZhCn, "火龙".into()),
            I18nText::new("card.fire_dragon.description".into(), Locale::EnUs, "A dragon of fire".into()),
            I18nText::new("card.fire_dragon.description".into(), Locale::JaJp, "火の竜".into()),
            I18nText::new("card.ice_dragon.description".into(), Locale::EnUs, "An ice dragon".into()),
        ]),
    );
    I18nServiceImpl::new(repo)
}

#[tokio::test]
async fn test_card_description_i18n_chain() {
    let svc = make_service_with_card_descriptions().await;
    // 4 locale 都能拉到 (zh_cn / en_us / ja_jp / ko_kr — ko_kr 缺, fallback 到 en_us)
    for (locale, expected_text, expect_fallback) in [
        (Locale::ZhCn, "火龙", false),
        (Locale::EnUs, "A dragon of fire", false),
        (Locale::JaJp, "火の竜", false),
        (Locale::KoKr, "A dragon of fire", true), // ko_kr 缺 → fallback en_us
    ] {
        let r: GetTextResult = svc.get_text("card.fire_dragon.description", locale).await.unwrap();
        assert_eq!(r.text, expected_text, "locale={:?}", locale);
        assert_eq!(r.fallback_used, expect_fallback, "locale={:?}", locale);
    }
}

#[tokio::test]
async fn test_card_description_partial_fallback() {
    let svc = make_service_with_card_descriptions().await;
    // ice_dragon 只有 en_us, ja_jp 应 fallback 到 en_us
    let r = svc.get_text("card.ice_dragon.description", Locale::JaJp).await.unwrap();
    assert_eq!(r.text, "An ice dragon");
    assert!(r.fallback_used);
    // 缓存命中后第二次立即返
    let r2 = svc.get_text("card.ice_dragon.description", Locale::JaJp).await.unwrap();
    assert_eq!(r2.text, "An ice dragon");
}

#[tokio::test]
async fn test_card_description_gettexts_batch_initialization() {
    let svc = make_service_with_card_descriptions().await;
    // 客户端初始化: 1 RTT 拉 3 个 key (zh_cn 全部命中)
    let keys = vec![
        "card.fire_dragon.description".to_string(),
        "card.ice_dragon.description".to_string(),
        "card.unknown.description".to_string(), // 完全缺
    ];
    let r = svc.get_texts(keys, Locale::ZhCn).await.unwrap();
    assert_eq!(r.entries.len(), 3);
    assert!(r.any_fallback); // unknown 触发
    assert_eq!(r.entries[0].text, "火龙");
    assert!(!r.entries[0].fallback_used);
    // ice_dragon 没有 zh_cn, 应 fallback 到 en_us
    assert!(r.entries[1].fallback_used);
    assert_eq!(r.entries[1].text, "An ice dragon");
    assert_eq!(r.entries[2].text, ""); // unknown 完全缺
    assert!(r.entries[2].fallback_used);
}

#[tokio::test]
async fn test_cache_layer_short_ttl_invalidates_after_expiry() {
    // 缓存层验证 (复用 i18n service 的 TtlCache, 短 TTL)
    let repo: Arc<dyn I18nRepository> = Arc::new(
        InMemoryI18nRepository::new().with_texts(vec![
            I18nText::new("k1".into(), Locale::EnUs, "v1".into()),
        ]),
    );
    let svc = I18nServiceImpl::with_cache_ttl(repo.clone(), Duration::from_millis(50));
    let r1 = svc.get_text("k1", Locale::EnUs).await.unwrap();
    assert_eq!(r1.text, "v1");
    std::thread::sleep(Duration::from_millis(80));
    repo.upsert_text(&I18nText::new("k1".into(), Locale::EnUs, "v2".into())).await.unwrap();
    let r2 = svc.get_text("k1", Locale::EnUs).await.unwrap();
    assert_eq!(r2.text, "v2", "缓存过期后应重读 DB");
}
