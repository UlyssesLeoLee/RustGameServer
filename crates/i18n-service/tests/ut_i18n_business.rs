//! i18n-service 域业务函数单元测试 (per PT-WORKER-BRIEFING §2 必做)
//!
//! 5 UT 覆盖:
//! 1. locale_str_enum_roundtrip (lowercase_underscore 格式)
//! 2. locale_default_fallback_chain (default=zh_cn, fallback=en_us)
//! 3. i18n_text_factory_initializes_timestamps
//! 4. ttl_cache_basic_put_get
//! 5. ttl_cache_expires_after_ttl (短 TTL 验证)

use std::sync::Arc;
use std::time::Duration;
use i18n_service::entity::{I18nText, Locale};
use i18n_service::repository::{I18nRepository, InMemoryI18nRepository};
use i18n_service::service::TtlCache;

#[test]
fn locale_str_enum_roundtrip() {
    for l in [Locale::ZhCn, Locale::EnUs, Locale::JaJp, Locale::KoKr] {
        assert_eq!(Locale::from_str(l.as_str()), Some(l));
    }
    assert_eq!(Locale::from_str("bogus"), None);
    assert_eq!(Locale::from_str("zh-CN"), None); // 必须 lowercase_underscore
}

#[test]
fn locale_default_fallback_chain() {
    assert_eq!(Locale::default_locale(), Locale::ZhCn);
    assert_eq!(Locale::fallback_locale(), Locale::EnUs);
    // 防 DEC-038-05 拍板改变后, 同步修改
}

#[test]
fn i18n_text_factory_initializes_timestamps() {
    let t = I18nText::new("k".to_string(), Locale::EnUs, "hello".to_string());
    assert_eq!(t.key, "k");
    assert_eq!(t.locale, Locale::EnUs);
    assert_eq!(t.text, "hello");
    assert_eq!(t.created_at, t.updated_at);
}

#[test]
fn ttl_cache_basic_put_get() {
    let cache = TtlCache::new(Duration::from_secs(60));
    assert!(cache.get("k1", Locale::EnUs).is_none());
    cache.put("k1", Locale::EnUs, "hello");
    assert_eq!(cache.get("k1", Locale::EnUs), Some("hello".to_string()));
    // 不同 locale 互不影响
    assert!(cache.get("k1", Locale::ZhCn).is_none());
}

#[test]
fn ttl_cache_expires_after_ttl() {
    let cache = TtlCache::new(Duration::from_millis(50));
    cache.put("k1", Locale::EnUs, "v1");
    assert_eq!(cache.get("k1", Locale::EnUs), Some("v1".to_string()));
    std::thread::sleep(Duration::from_millis(80));
    // 过期后应返 None
    assert!(cache.get("k1", Locale::EnUs).is_none());
}

#[tokio::test]
async fn in_memory_upsert_then_find_text() {
    let repo: Arc<dyn I18nRepository> = Arc::new(InMemoryI18nRepository::new());
    let t = I18nText::new("k1".to_string(), Locale::EnUs, "hello".to_string());
    repo.upsert_text(&t).await.unwrap();
    let got = repo.find_text("k1", Locale::EnUs).await.unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().text, "hello");
}
