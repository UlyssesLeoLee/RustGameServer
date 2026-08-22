-- player-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-018 §3）
-- 5 域玩家域 player_db schema 初始
-- 54.4 占位：仅最小 schema；54.5-54.7 业务实施时按 DTL-018 详细 entity 扩展

CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_players_name ON players (name);
CREATE INDEX IF NOT EXISTS idx_players_level ON players (level);

-- 54.4 接受：占位最小 schema；active-active 跨服身份、好友关系等
-- 详细字段待 54.6 entity 实施
