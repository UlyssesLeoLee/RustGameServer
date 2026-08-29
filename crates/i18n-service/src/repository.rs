//! i18n-service 域 Repository
//!
//! trait + PgRepository (sqlx impl) + InMemoryRepository (测用)
//! 规范: RGS-REQ-038 §NFR-005 + RGS-DTL-038 §4.1 + DEC-038-05
//!
//! 设计要点:
//! - 文案 (key, locale) 唯一, 走 idx_i18n_key 索引
//! - 高频读 (GetText / GetTexts) 优先用 InMemory 缓存, 缓存 5 分钟 TTL
//! - 缓存 miss 时 fall through 到 PgRepository

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::entity::{I18nText, LanguageInfo, Locale};
use crate::Result;

/// i18n 域 Repository trait
#[async_trait]
pub trait I18nRepository: Send + Sync {
    /// 按 (key, locale) 拉取单条文案
    async fn find_text(&self, key: &str, locale: Locale) -> Result<Option<I18nText>>;

    /// 按 key 拉取所有 locale 的文案(给 GetTexts 批量 + fallback 用)
    /// 返回 HashMap<locale_str, I18nText>, 缺该 locale 时不包含 entry
    async fn find_texts_by_key(&self, key: &str) -> Result<HashMap<String, I18nText>>;

    /// 列出系统支持的所有语言(给 ListLanguages RPC)
    async fn list_languages(&self) -> Result<Vec<LanguageInfo>>;

    /// 插入 / 更新文案 (Upsert 语义, 主要给 admin 工具用, 客户端不直接调)
    async fn upsert_text(&self, text: &I18nText) -> Result<()>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgI18nRepository {
    pool: PgPool,
}

impl PgI18nRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_text(row: sqlx::postgres::PgRow) -> I18nText {
    let locale_str: String = row.get("locale");
    I18nText {
        key: row.get("key"),
        locale: Locale::from_str(&locale_str).unwrap_or(Locale::EnUs),
        text: row.get("text"),
        updated_at: row.get("updated_at"),
        created_at: row.get("created_at"),
    }
}

fn row_to_language(row: sqlx::postgres::PgRow) -> LanguageInfo {
    let locale_str: String = row.get("locale");
    LanguageInfo {
        locale: Locale::from_str(&locale_str).unwrap_or(Locale::EnUs),
        display_name: row.get("display_name"),
        is_default: row.get("is_default"),
        enabled: row.get("enabled"),
    }
}

#[async_trait]
impl I18nRepository for PgI18nRepository {
    async fn find_text(&self, key: &str, locale: Locale) -> Result<Option<I18nText>> {
        let row = sqlx::query(
            "SELECT key, locale, text, updated_at, created_at \
             FROM i18n_texts WHERE key = $1 AND locale = $2",
        )
        .bind(key)
        .bind(locale.as_str())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_text))
    }

    async fn find_texts_by_key(&self, key: &str) -> Result<HashMap<String, I18nText>> {
        let rows = sqlx::query(
            "SELECT key, locale, text, updated_at, created_at \
             FROM i18n_texts WHERE key = $1",
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let t = row_to_text(row);
            map.insert(t.locale.as_str().to_string(), t);
        }
        Ok(map)
    }

    async fn list_languages(&self) -> Result<Vec<LanguageInfo>> {
        let rows = sqlx::query(
            "SELECT locale, display_name, is_default, enabled \
             FROM i18n_languages WHERE enabled = TRUE ORDER BY locale",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_language).collect())
    }

    async fn upsert_text(&self, text: &I18nText) -> Result<()> {
        sqlx::query(
            "INSERT INTO i18n_texts (key, locale, text, updated_at, created_at) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (key, locale) DO UPDATE SET \
                text = EXCLUDED.text, updated_at = EXCLUDED.updated_at",
        )
        .bind(&text.key)
        .bind(text.locale.as_str())
        .bind(&text.text)
        .bind(text.updated_at)
        .bind(text.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ============================================================================
// InMemoryRepository (测用)
// ============================================================================

pub struct InMemoryI18nRepository {
    /// (key, locale) -> I18nText
    texts: Mutex<HashMap<String, I18nText>>,
    /// locale -> LanguageInfo
    languages: Mutex<HashMap<String, LanguageInfo>>,
}

impl InMemoryI18nRepository {
    pub fn new() -> Self {
        let mut languages = HashMap::new();
        // 默认填 4 个语言, 跟 migrations 0001 默认一致
        languages.insert(
            "zh_cn".to_string(),
            LanguageInfo::new(Locale::ZhCn, "简体中文".to_string(), true, true),
        );
        languages.insert(
            "en_us".to_string(),
            LanguageInfo::new(Locale::EnUs, "English".to_string(), false, true),
        );
        languages.insert(
            "ja_jp".to_string(),
            LanguageInfo::new(Locale::JaJp, "日本語".to_string(), false, true),
        );
        languages.insert(
            "ko_kr".to_string(),
            LanguageInfo::new(Locale::KoKr, "한국어".to_string(), false, true),
        );
        Self {
            texts: Mutex::new(HashMap::new()),
            languages: Mutex::new(languages),
        }
    }

    /// 测试用: 批量预填文案
    pub fn with_texts(self, texts: Vec<I18nText>) -> Self {
        let mut g = self.texts.lock().unwrap();
        for t in texts {
            g.insert(format!("{}::{}", t.key, t.locale.as_str()), t);
        }
        drop(g);
        self
    }
}

impl Default for InMemoryI18nRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl I18nRepository for InMemoryI18nRepository {
    async fn find_text(&self, key: &str, locale: Locale) -> Result<Option<I18nText>> {
        let g = self.texts.lock().unwrap();
        Ok(g.get(&format!("{}::{}", key, locale.as_str())).cloned())
    }

    async fn find_texts_by_key(&self, key: &str) -> Result<HashMap<String, I18nText>> {
        let g = self.texts.lock().unwrap();
        let mut out = HashMap::new();
        for (k, v) in g.iter() {
            if let Some((k_key, k_locale)) = k.split_once("::") {
                if k_key == key {
                    out.insert(k_locale.to_string(), v.clone());
                }
            }
        }
        Ok(out)
    }

    async fn list_languages(&self) -> Result<Vec<LanguageInfo>> {
        let g = self.languages.lock().unwrap();
        let mut v: Vec<LanguageInfo> = g
            .values()
            .filter(|l| l.enabled)
            .cloned()
            .collect();
        v.sort_by(|a, b| a.locale.as_str().cmp(b.locale.as_str()));
        Ok(v)
    }

    async fn upsert_text(&self, text: &I18nText) -> Result<()> {
        let mut g = self.texts.lock().unwrap();
        g.insert(format!("{}::{}", text.key, text.locale.as_str()), text.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_upsert_and_find_text() {
        let repo = InMemoryI18nRepository::new();
        let t = I18nText::new("k1".to_string(), Locale::EnUs, "hello".to_string());
        repo.upsert_text(&t).await.unwrap();

        let got = repo.find_text("k1", Locale::EnUs).await.unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().text, "hello");
    }

    #[tokio::test]
    async fn in_memory_find_text_locale_miss_returns_none() {
        let repo = InMemoryI18nRepository::new();
        let got = repo.find_text("nonexistent", Locale::EnUs).await.unwrap();
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn in_memory_find_texts_by_key_returns_all_locales() {
        let repo = InMemoryI18nRepository::new()
            .with_texts(vec![
                I18nText::new("k2".to_string(), Locale::EnUs, "hello".to_string()),
                I18nText::new("k2".to_string(), Locale::ZhCn, "你好".to_string()),
            ]);
        let m = repo.find_texts_by_key("k2").await.unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m.get("en_us").unwrap().text, "hello");
        assert_eq!(m.get("zh_cn").unwrap().text, "你好");
    }

    #[tokio::test]
    async fn in_memory_list_languages_default_seeded() {
        let repo = InMemoryI18nRepository::new();
        let langs = repo.list_languages().await.unwrap();
        assert_eq!(langs.len(), 4);
        let zh = langs.iter().find(|l| l.locale == Locale::ZhCn).unwrap();
        assert!(zh.is_default);
    }
}
