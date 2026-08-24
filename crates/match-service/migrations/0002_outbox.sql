-- match-service migration 0002_outbox（per RGS-REV-007 CH1+AH1 / DEC-015 P1 / WF-1-55.17）
-- 事务性消息 outbox 表（per RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005）
-- 状态机：pending → in_flight → sent / failed
-- 多 relay 副本并发：FOR UPDATE SKIP LOCKED 防重复消费 + lease_until 防死锁
--
-- ⚠️ 已知反 pattern（per RGS-REV-009 CR-2 / WF-1-55.28）: 本文件内嵌的
-- `CONSTRAINT chk_outbox_status CHECK (...)` 在 `CREATE TABLE IF NOT EXISTS`
-- 块**内最后一行**, 在 fresh DB 部署有效, 但**已部署环境 13dec2d 的 CREATE
-- 块被 sqlx 静默跳过 → CHECK 约束永不生效**.
-- **修复已下推到独立 migration**: `0003_outbox_check_idempotent.sql` 用
-- `DO $$ ... EXCEPTION WHEN duplicate_object THEN NULL; END $$;` 幂等块,
-- 独立加 CHECK 约束, 兼容 fresh DB + 已部署两种环境.
-- **本文件**不修改（保持历史 migration 不可变语义）, 修复完全在
-- 0003_outbox_check_idempotent.sql 承担.

CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    subject VARCHAR(256) NOT NULL,
    payload JSONB NOT NULL,
    command_id UUID NOT NULL,
    saga_id UUID,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    lease_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox (created_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_outbox_in_flight ON outbox (lease_until) WHERE status = 'in_flight';
CREATE INDEX IF NOT EXISTS idx_outbox_command_id ON outbox (command_id);
