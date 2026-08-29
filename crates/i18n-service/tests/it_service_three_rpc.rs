//! i18n-service 桶 14 IT (per RGS-DTL-038 §4.1 + DEC-038-05 + W35 桶 14 补完)
//!
//! ## 范围 (3 IT)
//! 1. get_text_single_key  (单 key RPC, 正常路径)
//! 2. get_texts_batch (批量 RPC, 部分 fallback)
//! 3. list_languages (4 语言列表)
//!
//! 关联: 8/29 15:30 失职落档 → W35 补完, 31 测试全过目标 (6 UT + 3 IT)
//!      8/29 16:38 JST 9 DEC A 拍板 (DEC-038-05)

use i18n_service::entity::{I18nText, Locale};
use i18n_service::repository::{I18nRepository, InMemoryI18nRepository};
use i18n_service::service::{GetTextResult, GetTextsResult, I18nService, I18nServiceImpl};
use std::sync::Arc;

/// 构造带 seed 数据的 InMemory service (per IT 通用 fixture)
async fn make_service_with_seeds() -> I18nServiceImpl {
    let repo = Arc::new(
        InMemoryI18nRepository::new().with_texts(vec![
            I18nText::new("card.shanghai.welcome".to_string(), Locale::ZhCn, "欢迎来到上海".to_string()),
            I18nText::new("card.shanghai.welcome".to_string(), Locale::EnUs, "Welcome to Shanghai".to_string()),
            I18nText::new("card.shanghai.welcome".to_string(), Locale::JaJp, "上海へようこそ".to_string()),
            I18nText::new("card.peking.info".to_string(), Locale::ZhCn, "北京信息".to_string()),
            I18nText::new("card.peking.info".to_string(), Locale::EnUs, "Peking Info".to_string()),
        ]),
    );
    I18nServiceImpl::new(repo as Arc<dyn I18nRepository>)
}

/// IT-1: GetText 单 key 正常路径
#[tokio::test]
async fn it_get_text_single_key_returns_correct_text() {
    let svc = make_service_with_seeds().await;

    // 正常 zh_cn 命中
    let r: GetTextResult = svc
        .get_text("card.shanghai.welcome", Locale::ZhCn)
        .await
        .unwrap();
    assert_eq!(r.text, "欢迎来到上海");
    assert_eq!(r.locale, Locale::ZhCn);
    assert!(!r.fallback_used);

    // 切换 locale 命中
    let r2 = svc
        .get_text("card.shanghai.welcome", Locale::JaJp)
        .await
        .unwrap();
    assert_eq!(r2.text, "上海へようこそ");
    assert_eq!(r2.locale, Locale::JaJp);
    assert!(!r2.fallback_used);
}

/// IT-2: GetTexts 批量 (per RGS-DTL-038 §4.1 性能优化, 1 RTT 拉多 key)
#[tokio::test]
async fn it_get_texts_batch_returns_all_keys() {
    let svc = make_service_with_seeds().await;

    let r: GetTextsResult = svc
        .get_texts(
            vec![
                "card.shanghai.welcome".to_string(),
                "card.peking.info".to_string(),
            ],
            Locale::EnUs,
        )
        .await
        .unwrap();

    assert_eq!(r.entries.len(), 2);
    assert!(!r.any_fallback);
    assert_eq!(r.entries[0].text, "Welcome to Shanghai");
    assert_eq!(r.entries[1].text, "Peking Info");

    // 切换到 ja_jp, 两个 key 都缺, 应 fallback 到 en_us
    let r2 = svc
        .get_texts(
            vec![
                "card.shanghai.welcome".to_string(),
                "card.peking.info".to_string(),
            ],
            Locale::JaJp,
        )
        .await
        .unwrap();

    assert!(r2.any_fallback);
    // shanghai.welcome 在 ja_jp 有定义, 不会 fallback
    assert!(!r2.entries[0].fallback_used);
    assert_eq!(r2.entries[0].text, "上海へようこそ");
    // peking.info 在 ja_jp 缺, 应 fallback 到 en_us
    assert!(r2.entries[1].fallback_used);
    assert_eq!(r2.entries[1].text, "Peking Info");
}

/// IT-3: ListLanguages 列出 4 语言, zh_cn 是 default
#[tokio::test]
async fn it_list_languages_returns_four_locales() {
    let svc = make_service_with_seeds().await;

    let langs = svc.list_languages().await.unwrap();
    assert_eq!(langs.len(), 4);

    let locales: Vec<Locale> = langs.iter().map(|l| l.locale).collect();
    assert!(locales.contains(&Locale::ZhCn));
    assert!(locales.contains(&Locale::EnUs));
    assert!(locales.contains(&Locale::JaJp));
    assert!(locales.contains(&Locale::KoKr));

    // zh_cn 应是默认
    let zh = langs.iter().find(|l| l.locale == Locale::ZhCn).unwrap();
    assert!(zh.is_default);
    assert!(zh.enabled);
    assert_eq!(zh.display_name, "简体中文");
}
