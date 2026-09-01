-- player-service migration 4 fix (per 9/1 09:53 JST 部署恢复)
-- 真因: m4 line 93 在 player_characters 内 CREATE TABLE 时引用 player_inventory (line 114 才建)
-- 修复: 拆 CREATE TABLE 跟 FK constraint, 顺序: 建表 → 建表 → ALTER 加 FK
-- 注意: 这是 m4 的 sqlx 副作用, m4 文件本身也需要后续修, 这里仅 workaround

-- 表 1: player_characters (no FK to player_inventory)
CREATE TABLE IF NOT EXISTS player_characters (
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

-- 表 2: player_inventory
CREATE TABLE IF NOT EXISTS player_inventory (
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

-- 索引 (per m4 原始)
CREATE INDEX IF NOT EXISTS idx_pc_player_id ON player_characters (player_id);
CREATE INDEX IF NOT EXISTS idx_pc_class_level ON player_characters (char_class, level);
CREATE INDEX IF NOT EXISTS idx_pc_weapon ON player_characters (primary_weapon_id);
CREATE INDEX IF NOT EXISTS idx_pc_stats_gin ON player_characters USING GIN (stats);
CREATE INDEX IF NOT EXISTS idx_pi_player_id ON player_inventory (player_id);
CREATE INDEX IF NOT EXISTS idx_pi_item_id ON player_inventory (item_id);
CREATE INDEX IF NOT EXISTS idx_pi_acquired_at ON player_inventory (acquired_at);
CREATE INDEX IF NOT EXISTS idx_pi_metadata_gin ON player_inventory USING GIN (metadata);

-- FK constraints (separately, after both tables exist)
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_pc_player') THEN
    ALTER TABLE player_characters ADD CONSTRAINT fk_pc_player FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_pc_weapon') THEN
    ALTER TABLE player_characters ADD CONSTRAINT fk_pc_weapon FOREIGN KEY (primary_weapon_id) REFERENCES player_inventory(id) ON DELETE SET NULL;
  END IF;
END $$;

-- 记录 m4 已跑 (避免 pod 重启再跑)
INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
VALUES (4, 'player_characters_inventory', now(), true, 0, 0)
ON CONFLICT (version) DO NOTHING;
