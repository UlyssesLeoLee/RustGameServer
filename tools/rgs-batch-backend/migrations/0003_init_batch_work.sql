-- 0003_init_batch_work.sql
-- rgs-batch PG schema batch_work 初始化 (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-5, 2026-09-02 02:10 JST Mavis 接手代签)
--
-- Work 3 张 (per BAS-001 v0.2 §3.3 session-bound + cleanup SOP)
-- 1. task_progress W-01: 任务进度 (WebSocket 流式, GAP-2)
-- 2. task_buffer W-02: 任务缓冲 (临时数据)
-- 3. audit_session W-03: 审计会话 (per BAS-001 §6.6)
--
-- 约束:
--   - 按 BAS-001 v0.2 §3.3 session-bound, 完成后 cleanup
--   - 按 §6.4 cleanup job 24h TTL (PH-2 加 GIN 索引)

SET client_min_messages = WARNING;

CREATE TABLE IF NOT EXISTS batch_work.task_progress (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_execution_id UUID NOT NULL REFERENCES batch_transaction.task_execution(id) ON DELETE CASCADE,
    session_id      UUID NOT NULL,  -- websocket session
    progress_pct    INTEGER NOT NULL DEFAULT 0 CHECK (progress_pct BETWEEN 0 AND 100),
    current_step    VARCHAR(128),
    message         TEXT,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 24h TTL (per BAS-001 §6.4)
);

CREATE INDEX IF NOT EXISTS idx_progress_session ON batch_work.task_progress(session_id);
CREATE INDEX IF NOT EXISTS idx_progress_execution ON batch_work.task_progress(task_execution_id);

CREATE TABLE IF NOT EXISTS batch_work.task_buffer (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_execution_id UUID NOT NULL REFERENCES batch_transaction.task_execution(id) ON DELETE CASCADE,
    buffer_type     VARCHAR(32) NOT NULL,  -- request / response / intermediate
    payload         JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 24h TTL (per BAS-001 §6.4)
);

CREATE INDEX IF NOT EXISTS idx_buffer_execution ON batch_work.task_buffer(task_execution_id);

CREATE TABLE IF NOT EXISTS batch_work.audit_session (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL UNIQUE,  -- 用户会话 ID
    actor_id        UUID NOT NULL,
    actor_role      VARCHAR(32) NOT NULL,
    ip_address      INET,
    user_agent      TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,  -- session-bound, 过期后清理 (per BAS-001 §3.3)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_session_actor ON batch_work.audit_session(actor_id);
CREATE INDEX IF NOT EXISTS idx_session_expires ON batch_work.audit_session(expires_at);
