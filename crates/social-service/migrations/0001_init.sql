-- social-service migration 0001_init（per WBS v0.3 §2A.5 WF-1-54.4 + DTL-026 §3）
-- 5 域社交域 social_db schema 初始
-- 54.6 实化：guilds + guild_members

CREATE TABLE IF NOT EXISTS guilds (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    leader_id UUID NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    member_count INTEGER NOT NULL DEFAULT 1,
    experience BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_guilds_leader_id ON guilds (leader_id);
CREATE INDEX IF NOT EXISTS idx_guilds_level ON guilds (level);

-- 公会成员（per DTL-026 §3.2）
CREATE TABLE IF NOT EXISTS guild_members (
    id UUID PRIMARY KEY,
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    player_id UUID NOT NULL,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('leader', 'officer', 'member')),
    contribution BIGINT NOT NULL DEFAULT 0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (guild_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_members_guild_id ON guild_members (guild_id);
CREATE INDEX IF NOT EXISTS idx_members_player_id ON guild_members (player_id);
