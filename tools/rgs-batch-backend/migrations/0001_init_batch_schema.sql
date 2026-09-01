-- 0001_init_batch_schema.sql
-- rgs-batch PG schema 初始化 (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-5 + DETAILED-DESIGN §2.1, 2026-09-02 02:05 JST Mavis 接手代签)
--
-- 3 schema: batch_master / batch_transaction / batch_work (per BAS-001 v0.2 §3 横展开三分类)
-- 16 张表 (per BATCH REQ §4 数据模型):
--   Master 5: task_def, task_template, data_source, worker_pool, schedule
--   Transaction 8: task_execution, sub_task, audit_event, dlq_event, log_event, data_migration, saga_instance, message_outbox
--   Work 3: task_progress, task_buffer, audit_session
--
-- 约束:
--   - 按 BAS-001 v0.2 §3.2 Transaction append-only
--   - 按 BAS-001 v0.2 §3.3 Work session-bound + cleanup SOP
--   - 按 BAS-001 v0.2 §6.4 audit_event T-01 永久保留 (NFR-29)

SET client_min_messages = WARNING;

-- ========== batch_master (Master 5 张) ==========

CREATE TABLE IF NOT EXISTS batch_master.task_def (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    task_type       VARCHAR(32) NOT NULL,  -- gm_grant / log_process / data_migration / log_query / custom
    payload_schema  JSONB NOT NULL,
    owner_domain    VARCHAR(32) NOT NULL,  -- player / economy / match / social / admin / batch
    version         INTEGER NOT NULL DEFAULT 1,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(name, version)
);

CREATE INDEX IF NOT EXISTS idx_task_def_owner ON batch_master.task_def(owner_domain);
CREATE INDEX IF NOT EXISTS idx_task_def_active ON batch_master.task_def(is_active);

CREATE TABLE IF NOT EXISTS batch_master.task_template (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL UNIQUE,
    description     TEXT,
    task_def_id     UUID NOT NULL REFERENCES batch_master.task_def(id),
    template_body   JSONB NOT NULL,
    variables       JSONB,
    version         INTEGER NOT NULL DEFAULT 1,  -- GAP-7 任务模板版本化
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_template_def ON batch_master.task_template(task_def_id);

CREATE TABLE IF NOT EXISTS batch_master.data_source (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL UNIQUE,
    source_type     VARCHAR(32) NOT NULL,  -- postgres / sqlite / s3 / file / http
    conn_str_ref    VARCHAR(128) NOT NULL,  -- env var 引用, 不存实际凭据 (per 8/27 11:06 JST 硬 ban + NFR-30)
    credentials_ref VARCHAR(128),  -- env var 引用
    description     TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS batch_master.worker_pool (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL UNIQUE,
    pool_size       INTEGER NOT NULL DEFAULT 5,
    rpm_limit       INTEGER NOT NULL DEFAULT 1000,  -- 每分钟请求数
    target_domains  TEXT[] NOT NULL,  -- player/economy/match/social/admin/batch
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS batch_master.schedule (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL UNIQUE,
    task_def_id     UUID NOT NULL REFERENCES batch_master.task_def(id),
    cron_expr       VARCHAR(64),  -- cron 表达式 (3 种触发模式之一)
    interval_ms     BIGINT,  -- interval 模式 (毫秒)
    once_at         TIMESTAMPTZ,  -- oneshot 模式
    trigger_mode    VARCHAR(16) NOT NULL,  -- cron / interval / oneshot
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (trigger_mode IN ('cron', 'interval', 'oneshot'))
);

CREATE INDEX IF NOT EXISTS idx_schedule_active ON batch_master.schedule(is_active);
