-- player-service m4 cleanup + redo (per 9/1 09:55 JST)
-- 真因: m4 forward ref FK 失败, 修复后 owner 不是 player_user (我用 postgres 建的表)
-- 修复: DROP 表 + DELETE m4 行 + ALTER table owner 用 player_user 重跑

-- 删表 (cascade FK)
DROP TABLE IF EXISTS player_characters CASCADE;
DROP TABLE IF EXISTS player_inventory CASCADE;

-- 删 m4 记录 (让 sqlx 重新跑 m4)
DELETE FROM _sqlx_migrations WHERE version = 4;

-- 重新跑 m4 (player_user owner)
-- 表 1
CREATE TABLE player_characters (
    id                  UUID PRIMARY KEY,
    player_id           UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    char_class          TEXT NOT NULL CHECK (char_class IN ('warrior', 'mage', 'archer', 'assassin', 'support')),
    level               INTEGER NOT NULL DEFAULT 1 CHECK (level >= 1 AND level <= 999),
    hp                  INTEGER NOT NULL DEFAULT 100 CHECK (hp >= 0),
    atk                 INTEGER NOT NULL DEFAULT 10 CHECK (atk >= 0),
    def                 INTEGER NOT NULL DEFAULT 5 CHECK (def >= 0),
    crit_rate           REAL NOT NULL DEFAULT 0.05 CHECK (crit_rate >= 0.0 AND crit_rate <= 1.0),
    stats               JSONB NOT NULL DEFAULT '{}'::jsonb,
    primary_weapon_id   UUID,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE player_characters OWNER TO player_user;

-- 表 2
CREATE TABLE player_inventory (
    id                  UUID PRIMARY KEY,
    player_id           UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    item_id             UUID NOT NULL,
    quantity            INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    slot                INTEGER NOT NULL CHECK (slot >= 0 AND slot < 200),
    acquired_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata            JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_pi_player_slot UNIQUE (player_id, slot)
);
ALTER TABLE player_inventory OWNER TO player_user;

-- 索引
CREATE INDEX idx_pc_player_id ON player_characters (player_id);
CREATE INDEX idx_pc_class_level ON player_characters (char_class, level);
CREATE INDEX idx_pc_weapon ON player_characters (primary_weapon_id);
CREATE INDEX idx_pc_stats_gin ON player_characters USING GIN (stats);
CREATE INDEX idx_pi_player_id ON player_inventory (player_id);
CREATE INDEX idx_pi_item_id ON player_inventory (item_id);
CREATE INDEX idx_pi_acquired_at ON player_inventory (acquired_at);
CREATE INDEX idx_pi_metadata_gin ON player_inventory USING GIN (metadata);
ALTER INDEX idx_pc_player_id OWNER TO player_user;
ALTER INDEX idx_pc_class_level OWNER TO player_user;
ALTER INDEX idx_pc_weapon OWNER TO player_user;
ALTER INDEX idx_pc_stats_gin OWNER TO player_user;
ALTER INDEX idx_pi_player_id OWNER TO player_user;
ALTER INDEX idx_pi_item_id OWNER TO player_user;
ALTER INDEX idx_pi_acquired_at OWNER TO player_user;
ALTER INDEX idx_pi_metadata_gin OWNER TO player_user;

-- FK constraints (separately, after both tables exist)
ALTER TABLE player_characters ADD CONSTRAINT fk_pc_player FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE;
ALTER TABLE player_characters ADD CONSTRAINT fk_pc_weapon FOREIGN KEY (primary_weapon_id) REFERENCES player_inventory(id) ON DELETE SET NULL;
