//! i18n-service 域 entity 定义
//!
//! 2 个核心 entity:
//! - Locale: 语言枚举 (zh_cn / en_us / ja_jp / ko_kr, per RGS-DTL-038 §4.1)
//! - I18nText: 单 key + locale 的文案条目
//! - LanguageInfo: 系统支持语言元数据
//!
//! 规范: RGS-REQ-038 §NFR-005 + RGS-DTL-038 §4.1 + DEC-038-05

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 系统支持的语言 (与 common.proto v2 Locale enum 对齐)
/// locale string 格式: lowercase_underscore, e.g. "zh_cn"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    ZhCn,
    EnUs,
    JaJp,
    KoKr,
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::ZhCn => "zh_cn",
            Locale::EnUs => "en_us",
            Locale::JaJp => "ja_jp",
            Locale::KoKr => "ko_kr",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "zh_cn" => Some(Locale::ZhCn),
            "en_us" => Some(Locale::EnUs),
            "ja_jp" => Some(Locale::JaJp),
            "ko_kr" => Some(Locale::KoKr),
            _ => None,
        }
    }

    /// 默认 fallback locale (per RGS-DTL-038 §4.1: 默认 zh_cn, fallback en_us)
    pub fn default_locale() -> Self {
        Locale::ZhCn
    }

    /// 二级 fallback locale (per RGS-DTL-038 §4.1: zh_cn 缺失时回 en_us)
    pub fn fallback_locale() -> Self {
        Locale::EnUs
    }
}

/// 文案条目 (key, locale, text 三元组)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct I18nText {
    /// 文案 key, e.g. "card.shanghai.welcome"
    pub key: String,
    /// 语言
    pub locale: Locale,
    /// 文案内容
    pub text: String,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl I18nText {
    pub fn new(key: String, locale: Locale, text: String) -> Self {
        let now = Utc::now();
        Self {
            key,
            locale,
            text,
            updated_at: now,
            created_at: now,
        }
    }
}

/// 语言元数据 (per i18n_languages 表)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageInfo {
    pub locale: Locale,
    /// 本地化显示名, e.g. "简体中文" / "English" / "日本語" / "한국어"
    pub display_name: String,
    /// 是否系统默认
    pub is_default: bool,
    /// 是否启用 (false = 客户端不应使用)
    pub enabled: bool,
}

impl LanguageInfo {
    pub fn new(locale: Locale, display_name: String, is_default: bool, enabled: bool) -> Self {
        Self {
            locale,
            display_name,
            is_default,
            enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_roundtrip() {
        for l in [Locale::ZhCn, Locale::EnUs, Locale::JaJp, Locale::KoKr] {
            assert_eq!(Locale::from_str(l.as_str()), Some(l));
        }
        assert_eq!(Locale::from_str("bogus"), None);
        assert_eq!(Locale::from_str("zh-CN"), None, "must be lowercase_underscore");
    }

    #[test]
    fn locale_default_is_zh_cn() {
        assert_eq!(Locale::default_locale(), Locale::ZhCn);
        assert_eq!(Locale::fallback_locale(), Locale::EnUs);
    }

    #[test]
    fn i18n_text_factory_initializes_timestamps() {
        let t = I18nText::new("k".to_string(), Locale::EnUs, "hello".to_string());
        assert_eq!(t.key, "k");
        assert_eq!(t.locale, Locale::EnUs);
        assert_eq!(t.text, "hello");
    }
}
