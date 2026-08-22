-- match-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-016 §3）
-- 5 域匹配域 match_db schema 初始
-- 54.6 实化：matches + match_participants

CREATE TABLE IF NOT EXISTS matches (
    id UUID PRIMARY KEY,
    room_id TEXT NOT NULL UNIQUE,
    mode TEXT NOT NULL CHECK (mode IN ('1v1', '2v2', '5v5', 'battle_royale')),
    status TEXT NOT NULL DEFAULT 'waiting'
        CHECK (status IN ('waiting', 'in_progress', 'finished', 'cancelled')),
    winner_team TEXT CHECK (winner_team IS NULL OR winner_team IN ('blue', 'red', 'none')),
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_matches_status ON matches (status);
CREATE INDEX IF NOT EXISTS idx_matches_scheduled_at ON matches (scheduled_at);

-- 对局参与者（per DTL-016 §3.2）
CREATE TABLE IF NOT EXISTS match_participants (
    id UUID PRIMARY KEY,
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    player_id UUID NOT NULL,
    team TEXT NOT NULL CHECK (team IN ('blue', 'red', 'none')),
    score INTEGER NOT NULL DEFAULT 0,
    kills INTEGER NOT NULL DEFAULT 0,
    deaths INTEGER NOT NULL DEFAULT 0,
    assists INTEGER NOT NULL DEFAULT 0,
    is_mvp BOOLEAN NOT NULL DEFAULT false,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (match_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_participants_match_id ON match_participants (match_id);
CREATE INDEX IF NOT EXISTS idx_participants_player_id ON match_participants (player_id);
