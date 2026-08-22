-- player-service migration 0002_outbox（per RGS-REV-007 CH1+AH1 / DEC-015 P1 / WF-1-55.17）
-- 事务性消息 outbox 表（per RGS-DTL-100 §5.3 + RGS-SPEC-CROSS-005）
-- 状态机：pending → in_flight → sent / failed
-- 多 relay 副本并发：FOR UPDATE SKIP LOCKED 防重复消费 + lease_until 防死锁

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
