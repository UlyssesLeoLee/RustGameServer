-- =============================================================================
-- RGS outbox 表 migration 模板 (v1, 2026-09-19 JST)
-- =============================================================================
--
-- 用途: 新域 onboarding 时, 复制本文件到 crates/<new-domain>/migrations/0001_outbox.sql
--       并按需修改 (通常无需修改, 直接复制即可).
--
-- 依据:
-- - RGS-DTL-100 §4 Outbox + Inbox Pattern (per ADR-0061 v0.3 章节引用修订,
--   原 §5.3 已改为别名)
-- - RGS-SPEC-CROSS-005 事务性消息
-- - crates/shared-platform/src/outbox.rs::MIGRATION_TEMPLATE (54.11 模板 +
--   55.17 升级: in_flight 状态 + lease_until 列)
-- - ADR-0061 §2 决定 2: 「事务内强制 outbox 写入」约束
-- - ULYS-103 acceptance #2: 新域 onboarding 7 步 checklist 第 2 步产物
--
-- 7 步 checklist:
-- 1. 新域 RGS-REQ-NNN 需求定义书
-- 2. **新域 outbox 表 migration 创建** ← 本文件即产物
-- 3. 新域 main.rs 集成 outbox relay (per crates/<example>/src/main.rs)
-- 4. 新域事件族清单登记 (per `09a_跨域事件族清单_v1.1_可验证部分.md` §C 模板)
-- 5. 新域集成测试 (per crates/economy-service/tests/integration_outbox_atomicity.rs)
-- 6. 新域监控指标接入 (per `06_Outbox监控指标草案.md`)
-- 7. DLQ 接入 (per `07_PoisonEvent_DLQ草案.md` §2.2)
--
-- 已知反模式 (per RGS-REV-009 CR-2 / WF-1-55.28):
--   本文件内嵌的 `CONSTRAINT chk_outbox_status CHECK (...)` 在
--   `CREATE TABLE IF NOT EXISTS` 块**内最后一行**, 在 fresh DB 部署有效,
--   但**已部署环境的 CREATE 块被 sqlx 静默跳过 → CHECK 约束永不生效**.
--   **修复**: 创建本文件后,**立即追加** `0002_outbox_check_idempotent.sql`
--   (复制 crates/<example>/migrations/0004_outbox_check_idempotent.sql),
--   用 `DO $$ ... EXCEPTION WHEN duplicate_object THEN NULL; END $$;` 幂等块,
--   独立加 CHECK 约束, 兼容 fresh DB + 已部署两种环境.
-- =============================================================================

-- Outbox 表（per RGS-DTL-100 §4 事务性消息 + per RGS-SPEC-CROSS-005）
-- 54.11 模板: 各域 migrations 应包含本表
-- 55.17 升级 (per RGS-REV-007 CH1+CH2+AH1 / DEC-015 P1):
--   - status 加 'in_flight'
--   - 加 lease_until 列 (relay 持锁期)
--   - 加部分索引 idx_outbox_pending / idx_outbox_in_flight
CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    subject VARCHAR(256) NOT NULL,
    payload JSONB NOT NULL,
    command_id UUID NOT NULL,
    saga_id UUID,
    status VARCHAR(16) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_flight', 'sent', 'failed')),
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    lease_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))
);

-- 部分索引: 加速 list_pending 的 status='pending' 扫描
CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox (created_at) WHERE status = 'pending';

-- 部分索引: 加速 lease 过期回收 (per RGS-REV-007 CH2)
CREATE INDEX IF NOT EXISTS idx_outbox_in_flight ON outbox (lease_until) WHERE status = 'in_flight';

-- 幂等性 key 索引 (consumer 端去重 per envelope.command_id)
CREATE INDEX IF NOT EXISTS idx_outbox_command_id ON outbox (command_id);

-- 关联文档: 见文件头注释 §「依据」段
-- 模板版本: v1 (2026-09-19 JST)
-- 来源: crates/shared-platform/src/outbox.rs::MIGRATION_TEMPLATE