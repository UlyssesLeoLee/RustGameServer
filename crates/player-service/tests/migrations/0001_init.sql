-- player-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-018 §3）
-- 5 域玩家域 player_db schema 初始
-- 54.6 实化：players + player_sessions 表

CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    level INTEGER NOT NULL DEFAULT 1,
    vip_level INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'banned', 'disabled', 'pending')),
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_players_name ON players (name);
CREATE INDEX IF NOT EXISTS idx_players_level ON players (level);
CREATE INDEX IF NOT EXISTS idx_players_status ON players (status);

-- 玩家会话（per DTL-018 §3.2 active-active 跨服身份）
CREATE TABLE IF NOT EXISTS player_sessions (
    id UUID PRIMARY KEY,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    ip TEXT NOT NULL,
    login_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_player_sessions_player_id ON player_sessions (player_id);
CREATE INDEX IF NOT EXISTS idx_player_sessions_expires_at ON player_sessions (expires_at);
