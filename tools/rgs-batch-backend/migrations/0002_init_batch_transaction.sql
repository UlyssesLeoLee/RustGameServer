-- 0002_init_batch_transaction.sql
-- rgs-batch PG schema batch_transaction 初始化 (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-5, 2026-09-02 02:10 JST Mavis 接手代签)
--
-- Transaction 8 张 (per BAS-001 v0.2 §3.2 append-only)
-- 1. task_execution T-01: 任务执行实例
-- 2. sub_task T-02: 子任务
-- 3. audit_event T-03: 审计事件 (永久保留, per NFR-29 + BAS-001 §6.4)
-- 4. dlq_event T-04: DLQ 死信
-- 5. log_event T-05: 日志事件
-- 6. data_migration T-06: 数据迁移
-- 7. saga_instance T-07: saga 实例
-- 8. message_outbox T-08: 消息 outbox
--
-- 约束:
--   - 按 BAS-001 v0.2 §3.2 INSERT-only (除 DROP PARTITION)
--   - 按 §6.5 分区滚动 (PH-3 实施, 本版本预 schema)
--   - audit_event T-03 永久保留 (NFR-29)
--   - dlq_event T-04 + log_event T-05 按 cleanup SOP 保留 90 天

SET client_min_messages = WARNING;

CREATE TABLE IF NOT EXISTS batch_transaction.task_execution (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_def_id     UUID NOT NULL REFERENCES batch_master.task_def(id),
    schedule_id     UUID REFERENCES batch_master.schedule(id),
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',  -- pending/in_progress/succeeded/failed/skipped
    priority        INTEGER NOT NULL DEFAULT 0,  -- GAP-4 任务优先级
    payload         JSONB NOT NULL,
    started_at      TIMESTAMPTZ,
    finished_at     TIMESTAMPTZ,
    duration_ms     BIGINT,
    error_message   TEXT,
    trace_id        VARCHAR(64),  -- OTel trace ID
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (status IN ('pending', 'in_progress', 'succeeded', 'failed', 'skipped'))
);

CREATE INDEX IF NOT EXISTS idx_task_execution_status ON batch_transaction.task_execution(status);
CREATE INDEX IF NOT EXISTS idx_task_execution_priority ON batch_transaction.task_execution(priority DESC, created_at);
CREATE INDEX IF NOT EXISTS idx_task_execution_def ON batch_transaction.task_execution(task_def_id);
CREATE INDEX IF NOT EXISTS idx_task_execution_created ON batch_transaction.task_execution(created_at);

CREATE TABLE IF NOT EXISTS batch_transaction.sub_task (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id       UUID NOT NULL REFERENCES batch_transaction.task_execution(id) ON DELETE CASCADE,
    target_domain   VARCHAR(32) NOT NULL,  -- player/economy/match/social/admin
    target_endpoint VARCHAR(128) NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',
    attempt         INTEGER NOT NULL DEFAULT 0,
    max_attempts    INTEGER NOT NULL DEFAULT 3,
    response        JSONB,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at     TIMESTAMPTZ,
    CHECK (status IN ('pending', 'in_progress', 'succeeded', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_sub_task_parent ON batch_transaction.sub_task(parent_id);
CREATE INDEX IF NOT EXISTS idx_sub_task_status ON batch_transaction.sub_task(status);

CREATE TABLE IF NOT EXISTS batch_transaction.audit_event (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id        UUID NOT NULL,  -- 操作人
    actor_role      VARCHAR(32) NOT NULL,  -- 5 域 Lead / batch Lead / 架构师 / SRE
    action          VARCHAR(64) NOT NULL,  -- create/update/delete/execute/cancel/retry
    target_type     VARCHAR(32) NOT NULL,  -- task_def / task_execution / data_source / worker_pool
    target_id       UUID,
    parameters_hash VARCHAR(64) NOT NULL,  -- sha256(参数), 不存明文凭据 (per NFR-30 + 8/27 11:06 JST 硬 ban)
    result          VARCHAR(16) NOT NULL,  -- success/failure
    trace_id        VARCHAR(64),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 永久保留 (per NFR-29 + BAS-001 §6.4)
);

CREATE INDEX IF NOT EXISTS idx_audit_actor ON batch_transaction.audit_event(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_target ON batch_transaction.audit_event(target_type, target_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON batch_transaction.audit_event(created_at);

CREATE TABLE IF NOT EXISTS batch_transaction.dlq_event (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sub_task_id     UUID NOT NULL REFERENCES batch_transaction.sub_task(id),
    failure_reason  TEXT NOT NULL,
    attempts        INTEGER NOT NULL,
    last_error      TEXT,
    payload         JSONB NOT NULL,
    resolved        BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at     TIMESTAMPTZ,
    resolved_by     UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 90 天保留 (per BAS-001 §6.4)
);

CREATE INDEX IF NOT EXISTS idx_dlq_unresolved ON batch_transaction.dlq_event(resolved, created_at);

CREATE TABLE IF NOT EXISTS batch_transaction.log_event (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_execution_id UUID REFERENCES batch_transaction.task_execution(id),
    log_level       VARCHAR(16) NOT NULL,  -- debug/info/warn/error
    source          VARCHAR(64) NOT NULL,  -- console / backend / 5 域
    message         TEXT NOT NULL,
    context         JSONB,
    trace_id        VARCHAR(64),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 90 天保留 (per BAS-001 §6.4)
);

CREATE INDEX IF NOT EXISTS idx_log_execution ON batch_transaction.log_event(task_execution_id);
CREATE INDEX IF NOT EXISTS idx_log_created ON batch_transaction.log_event(created_at);
CREATE INDEX IF NOT EXISTS idx_log_level ON batch_transaction.log_event(log_level);

CREATE TABLE IF NOT EXISTS batch_transaction.data_migration (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(128) NOT NULL,
    data_source_id  UUID NOT NULL REFERENCES batch_master.data_source(id),
    target_table    VARCHAR(128) NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',
    rows_total      BIGINT,
    rows_migrated   BIGINT NOT NULL DEFAULT 0,
    before_snapshot JSONB,  -- rollback 用 (per F-24)
    rollback_sql    TEXT,  -- rollback SQL (per F-24 + GAP-8)
    started_at      TIMESTAMPTZ,
    finished_at     TIMESTAMPTZ,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'rolled_back'))
);

CREATE INDEX IF NOT EXISTS idx_migration_status ON batch_transaction.data_migration(status);

CREATE TABLE IF NOT EXISTS batch_transaction.saga_instance (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    saga_type       VARCHAR(64) NOT NULL,
    status          VARCHAR(16) NOT NULL DEFAULT 'started',
    payload         JSONB NOT NULL,
    steps           JSONB NOT NULL,
    current_step    INTEGER NOT NULL DEFAULT 0,
    error_message   TEXT,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at     TIMESTAMPTZ,
    CHECK (status IN ('started', 'compensating', 'succeeded', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_saga_status ON batch_transaction.saga_instance(status);

CREATE TABLE IF NOT EXISTS batch_transaction.message_outbox (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type  VARCHAR(64) NOT NULL,  -- task_execution / sub_task / saga
    aggregate_id    UUID NOT NULL,
    event_type      VARCHAR(64) NOT NULL,
    payload         JSONB NOT NULL,
    published       BOOLEAN NOT NULL DEFAULT FALSE,
    published_at    TIMESTAMPTZ,
    retry_count     INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_outbox_unpublished ON batch_transaction.message_outbox(published, created_at);
