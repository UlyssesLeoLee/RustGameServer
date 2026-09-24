-- admin-service migration 0005_lcm_step_execution (per WBS v0.2 桶 8 B8 / BAS-001 v0.3 §6.6.2)
--
-- 拍板: lcm_step_execution 归 **Work** 表 (per BAS-001 v0.3 §6.6.2 admin Lead 拍板 opt1, 2026-09-01 22:25 JST)
-- 保留期: 24h (completed step 在 24h 内 cleanup, per brief B8 "决策 保留期 24h vs 7d vs 30d" → 24h)
-- 范围: admin_db (per FR-LCM-001 LCM 6 表共置 admin_db + BAS-001 v0.3 §4.5 + §6.6.1)
-- 关联 run: realm_lifecycle_run (per BAS-001 v0.3 §6.6.1 Transaction T-01)
--
-- ⚠️ ULysS-95 修复 (2026-09-19, per RGS-BAS-007 §1.5 + RGS-SPEC-CROSS-005 §2 + ULYS-95 拍板):
--   去掉 `run_id REFERENCES realm_lifecycle_run(id) ON DELETE CASCADE` 跨 crate DDL 约束.
--   原因: `realm_lifecycle_run` 由 `crates/cluster-ops/migrations/0020_lcm_tables.sql` 建表,
--         但 admin-service migration runner (`sqlx::migrate!("./migrations")`) 只能 apply 本 crate
--         内的 migrations; sqlx::test 每个 test 新建空 DB 后跑全套 migrations 时, cluster-ops 的
--         0020 不在 apply 序列 → FK 目标不存在 → 必失败 (代码 42P01).
--   决策: 遵循 BAS-001 v0.3 §3.3 + RGS-SPEC-CROSS-005 §2 原则 —— **跨 module / 跨 crate 边界
--         不在 DDL 层加约束, 由应用层校验**. `run_id` 保留为无约束 UUID NOT NULL, 应用层
--         (PgLcmStepExecutionRepository PH-2 待实装) 在 INSERT 前 SELECT 校验 run 存在,
--         ON DELETE CASCADE 由应用层删除监听或 cleanup cron 配合处理.
--
-- 业务语义: LCM run (realm_lifecycle_run) 1 条 = 1 个 phase, 但 phase 内部多 step.
-- 例: new_realm phase 包含 provision / configure / smoke_test / route53_update /
--     load_balance_update / health_check 等 step. 本表 = step 级别实时执行记录.
--
-- 字段设计 (per BAS-001 v0.3 §6.6.2 候选表):
--   id          UUID PK
--   run_id      UUID NOT NULL (应用层保证指向 admin_db.realm_lifecycle_run.id; 跨 crate 边界不物化 FK)
--   step_seq    INT NOT NULL (phase 内步骤序号, 1-based)
--   step_name   TEXT NOT NULL (e.g. 'provision' / 'configure' / 'smoke_test')
--   status      TEXT NOT NULL DEFAULT 'pending'
--                CHECK (status IN ('pending', 'in_progress', 'succeeded', 'failed', 'skipped'))
--   started_at       TIMESTAMPTZ
--   completed_at     TIMESTAMPTZ
--   attempt_count    INT NOT NULL DEFAULT 0 (per-step retry 计数, 区别于 run-level retry)
--   last_error       TEXT
--   step_metadata    JSONB (跨 step 状态共享; per brief B8 "跨 step 状态共享用 step_metadata JSONB 是否合理" → 采纳)
--   expires_at  TIMESTAMPTZ NOT NULL (cleanup cron 在此时间后删除; 默认 = created_at + 24h)
--   created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
--   UNIQUE (run_id, step_seq)
--
-- 索引 (per BAS-001 v0.2 §6.6.2):
--   idx_lcm_step_run_id         ON lcm_step_execution (run_id)         — by run 查询 step 列表
--   idx_lcm_step_expires_at     ON lcm_step_execution (expires_at)
--                                 WHERE status IN ('pending', 'in_progress')  — cleanup cron partial index
--   idx_lcm_step_status         ON lcm_step_execution (status, started_at DESC) — 状态聚合查询

CREATE TABLE IF NOT EXISTS lcm_step_execution (
    id UUID PRIMARY KEY,
    -- run_id 跨 crate 引用 admin_db.realm_lifecycle_run.id, 由应用层校验 (per ULYS-95 修复 + RGS-BAS-007 §1.5)
    run_id UUID NOT NULL,
    step_seq INT NOT NULL,
    step_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'succeeded', 'failed', 'skipped')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    attempt_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    step_metadata JSONB,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (run_id, step_seq)
);

CREATE INDEX IF NOT EXISTS idx_lcm_step_run_id
    ON lcm_step_execution (run_id);

CREATE INDEX IF NOT EXISTS idx_lcm_step_expires_at
    ON lcm_step_execution (expires_at)
    WHERE status IN ('pending', 'in_progress');

CREATE INDEX IF NOT EXISTS idx_lcm_step_status
    ON lcm_step_execution (status, started_at DESC);

-- 注: cleanup cron 业务逻辑 (per BAS-001 v0.2 §6.3 14-§7 cleanup SOP + BAS-007 §4)
--   DELETE FROM lcm_step_execution
--   WHERE expires_at < NOW()
--     AND status IN ('succeeded', 'failed', 'skipped');
-- 实施位置: admin-service 启动时 spawn 定时任务 (PH-2 待实装).
