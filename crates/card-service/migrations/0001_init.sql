-- card-service migration 0001_init (per WBS v0.5 桶 10 + DTL-038 §7.1 #1-3)
-- 卡牌游戏新域 card_db schema 初始
-- 桶 10 实化: cards / card_series / card_instances 3 张表
-- 其它 5 张表 (decks / game_sessions / moves / replays / auctions) 归其它域, 不在本域

-- 1. cards (catalog, 静态) — per DTL-038 §7.1 #1
CREATE TABLE IF NOT EXISTS cards (
    card_id          TEXT PRIMARY KEY,
    series_id        TEXT NOT NULL,
    name_default     TEXT NOT NULL,
    name_i18n        JSONB NOT NULL DEFAULT '{}'::jsonb,
    type             SMALLINT NOT NULL,
    rarity           SMALLINT NOT NULL,
    base_cost        INT NOT NULL DEFAULT 0,
    description_i18n JSONB NOT NULL DEFAULT '{}'::jsonb,
    effect_ref       TEXT NOT NULL DEFAULT '',
    stats            JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_cards_series ON cards(series_id);
CREATE INDEX IF NOT EXISTS idx_cards_rarity ON cards(rarity);
CREATE INDEX IF NOT EXISTS idx_cards_type ON cards(type);

-- 2. card_series (卡包 / 系列) — per DTL-038 §7.1 #2
CREATE TABLE IF NOT EXISTS card_series (
    series_id    TEXT PRIMARY KEY,
    name_default TEXT NOT NULL,
    name_i18n    JSONB NOT NULL DEFAULT '{}'::jsonb,
    pack_size    INT NOT NULL CHECK (pack_size > 0),
    drop_table   JSONB NOT NULL DEFAULT '{"version":1,"snapshot_at":"","entries":[]}'::jsonb,
    price_type   SMALLINT NOT NULL DEFAULT 1,
    price_amount BIGINT NOT NULL DEFAULT 0,
    released_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    status       SMALLINT NOT NULL DEFAULT 1 -- 1=活跃 2=待发布 3=失败 4=取消
);
CREATE INDEX IF NOT EXISTS idx_card_series_status ON card_series(status);

-- 3. card_instances (玩家收藏, 动态) — per DTL-038 §7.1 #3
CREATE TABLE IF NOT EXISTS card_instances (
    instance_id  UUID PRIMARY KEY,
    card_id      TEXT NOT NULL,  -- 跨域引用 cards.card_id, 不物化 FK (跨 DB)
    owner_id     TEXT NOT NULL,  -- 跨域引用 players.id (UUID 字符串), 不物化 FK
    acquired_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    source       SMALLINT NOT NULL,
    level        INT NOT NULL DEFAULT 1,
    attrs        JSONB NOT NULL DEFAULT '{}'::jsonb,
    tradable     BOOLEAN NOT NULL DEFAULT TRUE,
    locked       BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX IF NOT EXISTS idx_card_instances_owner ON card_instances(owner_id);
CREATE INDEX IF NOT EXISTS idx_card_instances_card ON card_instances(card_id);
CREATE INDEX IF NOT EXISTS idx_card_instances_owner_acquired ON card_instances(owner_id, acquired_at DESC);
