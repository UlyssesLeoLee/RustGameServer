//! i18n-service 域 Service 业务实装(per RGS-DTL-038 §4.1 + DEC-038-05)
//!
//! ## 范围(per RGS-PLAN-WBS-token-bucket-v0.3 §2.2 桶 14 补完)
//! - 3 RPC 完整业务:GetText / GetTexts / ListLanguages
//! - Redis 缓存占位: in-memory BTreeMap + 5 分钟 TTL(per DEC-038-05)
//!   Redis 实际集成待 W36+ (per DTL-038 §6 跨域 saga 时一并引入)
//! - gRPC 桥接到 tonic server
//!
//! ## 业务规则
//! - GetText: 单 key + locale 拉取,缺该 locale 时 fall back 到 fallback_locale (en_us),
//!           再缺则返 fallback_used=true + default_locale 文本, 仍缺则返 KeyNotFound
//! - GetTexts: 批量拉取, 内部循环 GetText 逻辑, 收集 any_fallback 标志
//! - ListLanguages: 列出 enabled=true 的语言, is_default=true 标 1 个
//!
//! ## 关联
//! - 8/29 15:30 失职落档 i18n-service skeleton → 本次补完(commit 01f4be5 升级)
//! - 8/29 16:38 JST 9 DEC A 拍板 (DEC-038-05) — Redis 缓存 + DB + i18n-service 独立
//! - W36+ 待办: Redis 实际集成 + 跨域 saga(per DTL-038 §6.2 + §6.3)

use crate::entity::{LanguageInfo, Locale};
use crate::repository::I18nRepository;
use crate::Result;

use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[async_trait]
pub trait I18nService: Send + Sync {
    async fn get_text(&self, key: &str, locale: Locale) -> Result<GetTextResult>;
    async fn get_texts(&self, keys: Vec<String>, locale: Locale) -> Result<GetTextsResult>;
    async fn list_languages(&self) -> Result<Vec<LanguageInfo>>;
}

/// GetText RPC 业务结果(per RGS-DTL-038 §4.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTextResult {
    pub key: String,
    pub locale: Locale,
    pub text: String,
    pub fallback_used: bool,
}

/// GetTexts RPC 业务结果(批量)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTextsResult {
    pub entries: Vec<GetTextResult>,
    pub locale: Locale,
    pub any_fallback: bool,
}

/// Redis 缓存占位: 内存 BTreeMap + 5 分钟 TTL(per DEC-038-05)
///
/// 设计: 用 `BTreeMap<(String, &'static str), (String, Instant)>`,
/// `Instant` 记录入缓存时刻, 每次读时先检查是否过期。
///
/// W36+ 替换为 `redis-rs` 真实 client + SETEX 5 分钟。
#[derive(Clone)]
pub struct TtlCache {
    inner: Arc<Mutex<BTreeMap<(String, String), (String, Instant)>>>,
    ttl: Duration,
}

impl TtlCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BTreeMap::new())),
            ttl,
        }
    }

    /// 默认 5 分钟 TTL (per DEC-038-05)
    pub fn default_5min() -> Self {
        Self::new(Duration::from_secs(300))
    }

    /// 取缓存; 命中且未过期返 Some(text), 否则 None
    pub fn get(&self, key: &str, locale: Locale) -> Option<String> {
        let g = self.inner.lock().unwrap();
        let k = (key.to_string(), locale.as_str().to_string());
        g.get(&k).and_then(|(text, inserted_at)| {
            if inserted_at.elapsed() < self.ttl {
                Some(text.clone())
            } else {
                None
            }
        })
    }

    /// 写入缓存
    pub fn put(&self, key: &str, locale: Locale, text: &str) {
        let mut g = self.inner.lock().unwrap();
        g.insert(
            (key.to_string(), locale.as_str().to_string()),
            (text.to_string(), Instant::now()),
        );
    }

    /// 清空缓存(测试用)
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.inner.lock().unwrap().clear();
    }

    /// 缓存大小(测试用)
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }
}

pub struct I18nServiceImpl {
    repo: Arc<dyn I18nRepository>,
    cache: TtlCache,
}

impl I18nServiceImpl {
    pub fn new(repo: Arc<dyn I18nRepository>) -> Self {
        Self {
            repo,
            cache: TtlCache::default_5min(),
        }
    }

    /// 测试用 builder: 注入自定义 TTL
    #[allow(dead_code)]
    pub fn with_cache_ttl(repo: Arc<dyn I18nRepository>, ttl: Duration) -> Self {
        Self {
            repo,
            cache: TtlCache::new(ttl),
        }
    }

    /// GetText 核心业务: cache → repo → fallback 链
    async fn resolve_text(&self, key: &str, locale: Locale) -> Result<GetTextResult> {
        // 1. 缓存命中
        if let Some(text) = self.cache.get(key, locale) {
            return Ok(GetTextResult {
                key: key.to_string(),
                locale,
                text,
                fallback_used: false,
            });
        }

        // 2. DB 拉取(指定 locale)
        if let Some(t) = self.repo.find_text(key, locale).await? {
            self.cache.put(key, locale, &t.text);
            return Ok(GetTextResult {
                key: key.to_string(),
                locale,
                text: t.text,
                fallback_used: false,
            });
        }

        // 3. Fallback: 尝试 fallback_locale (en_us per DTL-038 §4.1)
        let fb = Locale::fallback_locale();
        if locale != fb {
            if let Some(t) = self.repo.find_text(key, fb).await? {
                self.cache.put(key, fb, &t.text);
                return Ok(GetTextResult {
                    key: key.to_string(),
                    locale: fb,
                    text: t.text,
                    fallback_used: true,
                });
            }
        }

        // 4. 二级 fallback: 尝试 default_locale (zh_cn)
        let def = Locale::default_locale();
        if locale != def && fb != def {
            if let Some(t) = self.repo.find_text(key, def).await? {
                self.cache.put(key, def, &t.text);
                return Ok(GetTextResult {
                    key: key.to_string(),
                    locale: def,
                    text: t.text,
                    fallback_used: true,
                });
            }
        }

        // 5. 全缺: KeyNotFound(per DTL-038 §4.1 缺文案上报客户端用 key 占位)
        Err(crate::Error::KeyNotFound(key.to_string()))
    }
}

#[async_trait]
impl I18nService for I18nServiceImpl {
    async fn get_text(&self, key: &str, locale: Locale) -> Result<GetTextResult> {
        if key.trim().is_empty() {
            return Err(crate::Error::Validation("key must not be empty".to_string()));
        }
        self.resolve_text(key, locale).await
    }

    async fn get_texts(&self, keys: Vec<String>, locale: Locale) -> Result<GetTextsResult> {
        if keys.is_empty() {
            return Err(crate::Error::Validation("keys must not be empty".to_string()));
        }
        // 批量内循环 resolve_text, 单 key 失败时跳过(返回空 text + fallback_used=true)
        // 业务选择: 批量接口不强失败(per DTL-038 §4.1 客户端初始化常用)
        let mut entries = Vec::with_capacity(keys.len());
        let mut any_fallback = false;
        for key in keys {
            match self.resolve_text(&key, locale).await {
                Ok(r) => {
                    if r.fallback_used {
                        any_fallback = true;
                    }
                    entries.push(r);
                }
                Err(crate::Error::KeyNotFound(_)) => {
                    // 缺文案: 插入空 + fallback 标记
                    entries.push(GetTextResult {
                        key: key.clone(),
                        locale,
                        text: String::new(),
                        fallback_used: true,
                    });
                    any_fallback = true;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(GetTextsResult {
            entries,
            locale,
            any_fallback,
        })
    }

    async fn list_languages(&self) -> Result<Vec<LanguageInfo>> {
        self.repo.list_languages().await
    }
}

// ============================================================================
// gRPC 桥接
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as i18n_proto;

    pub struct I18nGrpcService {
        pub impl_: Arc<I18nServiceImpl>,
    }

    impl I18nGrpcService {
        pub fn new(impl_: Arc<I18nServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    fn locale_from_proto(v: i32) -> Result<Locale> {
        match v {
            1 => Ok(Locale::ZhCn),
            2 => Ok(Locale::EnUs),
            3 => Ok(Locale::JaJp),
            4 => Ok(Locale::KoKr),
            _ => Err(crate::Error::Validation(format!(
                "unsupported locale: {}",
                v
            ))),
        }
    }

    fn locale_to_proto(l: Locale) -> i32 {
        match l {
            Locale::ZhCn => common_proto::Locale::ZhCn as i32,
            Locale::EnUs => common_proto::Locale::EnUs as i32,
            Locale::JaJp => common_proto::Locale::JaJp as i32,
            Locale::KoKr => common_proto::Locale::KoKr as i32,
        }
    }

    #[tonic::async_trait]
    impl i18n_proto::i18n_service_server::I18nService for I18nGrpcService {
        async fn health_check(
            &self,
            _request: tonic::Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<tonic::Response<common_proto::HealthCheckResponse>, tonic::Status>
        {
            Ok(tonic::Response::new(common_proto::HealthCheckResponse {
                status: common_proto::Status::Ok as i32,
                message: "ok".to_string(),
            }))
        }

        async fn get_text(
            &self,
            request: tonic::Request<i18n_proto::GetTextRequest>,
        ) -> std::result::Result<tonic::Response<i18n_proto::GetTextResponse>, tonic::Status>
        {
            let req = request.into_inner();
            let key = req.key.clone();
            let locale = locale_from_proto(req.locale)
                .map_err(Into::<tonic::Status>::into)?;
            let result = self
                .impl_
                .get_text(&key, locale)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(tonic::Response::new(i18n_proto::GetTextResponse {
                key: result.key,
                locale: locale_to_proto(result.locale),
                text: result.text,
                fallback_used: result.fallback_used,
            }))
        }

        async fn get_texts(
            &self,
            request: tonic::Request<i18n_proto::GetTextsRequest>,
        ) -> std::result::Result<tonic::Response<i18n_proto::GetTextsResponse>, tonic::Status>
        {
            let req = request.into_inner();
            let keys = req.keys.clone();
            let locale = locale_from_proto(req.locale)
                .map_err(Into::<tonic::Status>::into)?;
            let result = self
                .impl_
                .get_texts(keys, locale)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let entries: Vec<i18n_proto::I18nEntry> = result
                .entries
                .into_iter()
                .map(|e| i18n_proto::I18nEntry {
                    key: e.key,
                    text: e.text,
                    fallback_used: e.fallback_used,
                })
                .collect();
            Ok(tonic::Response::new(i18n_proto::GetTextsResponse {
                entries,
                locale: locale_to_proto(result.locale),
                any_fallback: result.any_fallback,
            }))
        }

        async fn list_languages(
            &self,
            _request: tonic::Request<i18n_proto::ListLanguagesRequest>,
        ) -> std::result::Result<tonic::Response<i18n_proto::ListLanguagesResponse>, tonic::Status>
        {
            let langs = self
                .impl_
                .list_languages()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let languages: Vec<i18n_proto::LanguageInfo> = langs
                .into_iter()
                .map(|l| i18n_proto::LanguageInfo {
                    locale: locale_to_proto(l.locale),
                    display_name: l.display_name,
                    is_default: l.is_default,
                })
                .collect();
            Ok(tonic::Response::new(i18n_proto::ListLanguagesResponse {
                languages,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::I18nText;
    use crate::repository::InMemoryI18nRepository;

    fn make_service() -> I18nServiceImpl {
        let repo = Arc::new(
            InMemoryI18nRepository::new().with_texts(vec![
                I18nText::new("k1".to_string(), Locale::ZhCn, "你好".to_string()),
                I18nText::new("k1".to_string(), Locale::EnUs, "hello".to_string()),
                I18nText::new("k2".to_string(), Locale::EnUs, "world".to_string()),
            ]),
        );
        I18nServiceImpl::new(repo)
    }

    /// 1. GetText 命中 exact locale
    #[tokio::test]
    async fn get_text_exact_locale_match() {
        let svc = make_service();
        let r = svc.get_text("k1", Locale::EnUs).await.unwrap();
        assert_eq!(r.text, "hello");
        assert_eq!(r.locale, Locale::EnUs);
        assert!(!r.fallback_used);
    }

    /// 2. GetText 缺该 locale 时 fallback
    #[tokio::test]
    async fn get_text_fallback_when_locale_missing() {
        let svc = make_service();
        // k2 没有 ja_jp, 应该 fallback 到 en_us (per DTL-038 §4.1)
        let r = svc.get_text("k2", Locale::JaJp).await.unwrap();
        assert_eq!(r.text, "world");
        assert_eq!(r.locale, Locale::EnUs);
        assert!(r.fallback_used);
    }

    /// 3. GetText 全缺时 KeyNotFound
    #[tokio::test]
    async fn get_text_key_not_found() {
        let svc = make_service();
        let err = svc.get_text("nonexistent", Locale::EnUs).await.unwrap_err();
        assert!(matches!(err, crate::Error::KeyNotFound(_)));
    }

    /// 4. GetTexts 批量, 部分 fallback
    #[tokio::test]
    async fn get_texts_batch_with_mixed_fallback() {
        let svc = make_service();
        let r = svc
            .get_texts(
                vec!["k1".to_string(), "k2".to_string(), "missing".to_string()],
                Locale::JaJp,
            )
            .await
            .unwrap();
        assert_eq!(r.entries.len(), 3);
        assert!(r.any_fallback);
        // k1 在 ja_jp 缺, fallback 到 en_us -> "hello"
        let k1 = &r.entries[0];
        assert_eq!(k1.text, "hello");
        assert!(k1.fallback_used);
        // k2 同上
        let k2 = &r.entries[1];
        assert_eq!(k2.text, "world");
        assert!(k2.fallback_used);
        // missing 完全缺, 占位
        let m = &r.entries[2];
        assert_eq!(m.text, "");
        assert!(m.fallback_used);
    }

    /// 5. GetTexts 空 keys 列表 → Validation 错误
    #[tokio::test]
    async fn get_texts_empty_keys_validation_error() {
        let svc = make_service();
        let err = svc.get_texts(vec![], Locale::EnUs).await.unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    /// 6. ListLanguages 返 4 语言, zh_cn 是 default
    #[tokio::test]
    async fn list_languages_default_seeded() {
        let svc = make_service();
        let langs = svc.list_languages().await.unwrap();
        assert_eq!(langs.len(), 4);
        let zh = langs.iter().find(|l| l.locale == Locale::ZhCn).unwrap();
        assert!(zh.is_default);
        assert!(zh.enabled);
    }

    /// 7. 缓存命中 (测试缓存层)
    #[tokio::test]
    async fn cache_hit_avoids_repo_call() {
        let repo = Arc::new(InMemoryI18nRepository::new().with_texts(vec![
            I18nText::new("cached_k".to_string(), Locale::EnUs, "cached_text".to_string()),
        ]));
        let svc = I18nServiceImpl::new(repo.clone() as Arc<dyn I18nRepository>);

        // 第一次: DB 拉取, 应缓存
        let r1 = svc.get_text("cached_k", Locale::EnUs).await.unwrap();
        assert_eq!(r1.text, "cached_text");
        assert_eq!(svc.cache.len(), 1);

        // 第二次: 缓存命中, 即便 DB 删了也能返
        repo.upsert_text(&I18nText::new(
            "cached_k".to_string(),
            Locale::EnUs,
            "changed_in_db".to_string(),
        ))
        .await
        .unwrap();
        let r2 = svc.get_text("cached_k", Locale::EnUs).await.unwrap();
        assert_eq!(r2.text, "cached_text", "应命中缓存, 不走 DB");
    }

    /// 8. 缓存过期(用极短 TTL 验证)
    #[tokio::test]
    async fn cache_ttl_expiry() {
        let repo = Arc::new(InMemoryI18nRepository::new().with_texts(vec![
            I18nText::new("ttl_k".to_string(), Locale::EnUs, "v1".to_string()),
        ]));
        let svc = I18nServiceImpl::with_cache_ttl(
            repo.clone() as Arc<dyn I18nRepository>,
            Duration::from_millis(50),
        );

        // 第一次写入缓存
        let r1 = svc.get_text("ttl_k", Locale::EnUs).await.unwrap();
        assert_eq!(r1.text, "v1");

        // 立即读: 缓存命中
        let r2 = svc.get_text("ttl_k", Locale::EnUs).await.unwrap();
        assert_eq!(r2.text, "v1");

        // 等过期
        tokio::time::sleep(Duration::from_millis(80)).await;

        // 更新 DB
        repo.upsert_text(&I18nText::new(
            "ttl_k".to_string(),
            Locale::EnUs,
            "v2".to_string(),
        ))
        .await
        .unwrap();

        // 再读: 缓存过期, 走 DB
        let r3 = svc.get_text("ttl_k", Locale::EnUs).await.unwrap();
        assert_eq!(r3.text, "v2", "缓存过期后应重读 DB");
    }

    /// 9. GetText 空 key 校验
    #[tokio::test]
    async fn get_text_empty_key_validation() {
        let svc = make_service();
        let err = svc.get_text("", Locale::EnUs).await.unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    /// 10. Locale::default_locale + fallback_locale 一致性
    #[test]
    fn locale_default_fallback_consistency() {
        assert_eq!(Locale::default_locale(), Locale::ZhCn);
        assert_eq!(Locale::fallback_locale(), Locale::EnUs);
        // 防 DEC-038-05 拍板改变后, 同步修改
    }
}
