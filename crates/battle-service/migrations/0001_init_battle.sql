-- battle-service init migration (per ARC-008 5 独立 DB → 7 域扩展)
-- 占位: 后续桶 10 业务实装接 PgRepository 时补 16 张表 schema (DB 三分类, per 9/1 18:30 JST)

-- 战斗 master (静态 / 慢变)
CREATE TABLE IF NOT EXISTS battle_master (
    battle_id TEXT PRIMARY KEY,
    mode TEXT NOT NULL,
    config_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 战斗 transaction (事件流水, append-only)
CREATE TABLE IF NOT EXISTS battle_transaction (
    event_id UUID PRIMARY KEY,
    battle_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 战斗 work (流程中临时, session-bound)
CREATE TABLE IF NOT EXISTS battle_work (
    session_id UUID PRIMARY KEY,
    player_id TEXT NOT NULL,
    state_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_battle_work_player ON battle_work(player_id);
CREATE INDEX IF NOT EXISTS idx_battle_work_expires ON battle_work(expires_at);
CREATE INDEX IF NOT EXISTS idx_battle_transaction_battle ON battle_transaction(battle_id);
