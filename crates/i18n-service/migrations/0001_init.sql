-- i18n-service migration 0001_init (per RGS-REQ-038 §NFR-005 + RGS-DTL-038 §4.1 + DEC-038-05)
-- 卡牌游戏多语言文案 i18n_db schema 初始
--
-- 表设计:
-- - i18n_texts: 文案表 (key, locale, text 三元组唯一)
-- - i18n_languages: 系统支持语言清单 + 默认 locale
-- - 高频读 (GetText / GetTexts) 走 idx_i18n_key_locale 索引
-- - Redis 缓存 5 分钟 TTL (per DEC-038-05 推荐 A)

CREATE TABLE IF NOT EXISTS i18n_texts (
    key TEXT NOT NULL,
    locale TEXT NOT NULL CHECK (locale IN ('zh_cn', 'en_us', 'ja_jp', 'ko_kr')),
    text TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (key, locale)
);

-- 高频读路径索引
CREATE INDEX IF NOT EXISTS idx_i18n_key ON i18n_texts (key);

CREATE TABLE IF NOT EXISTS i18n_languages (
    locale TEXT PRIMARY KEY CHECK (locale IN ('zh_cn', 'en_us', 'ja_jp', 'ko_kr')),
    display_name TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 唯一性: 至多 1 个默认 locale (per RGS-DTL-038 §4.1 默认 zh_cn)
CREATE UNIQUE INDEX IF NOT EXISTS idx_i18n_default_locale
    ON i18n_languages (is_default) WHERE is_default = TRUE;
