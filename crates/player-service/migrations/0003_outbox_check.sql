-- player-service migration 0003_outbox_check（per RGS-REV-009 CR-2 / WF-1-55.28）
-- 修复 13dec2d 写在 `CREATE TABLE IF NOT EXISTS outbox (...)` 块内 CHECK 静默失效问题：
-- 55.17 commit `53a8d37` 已创建 outbox 表，13dec2d 的 CREATE 块被 sqlx 静默跳过，
-- CHECK 约束在已部署环境永不生效。
-- 本 migration 用 `DO $$ ... EXCEPTION WHEN duplicate_object THEN NULL; END $$;`
-- 幂等块独立加 CHECK 约束，兼容 fresh DB（约束不存在→添加）和已部署（约束已存在→no-op）两种环境。
-- 状态机：pending / in_flight / sent / failed（与 13dec2d CHECK 字面量完全一致）

DO $$ BEGIN
    ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status
        CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'));
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;
