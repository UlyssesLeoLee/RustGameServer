-- cluster-ops migration 0021_lcm_step_execution (per RGS-DB-BAS-001 v0.2 §6.6.2 + 2026-09-01 21:16 JST 缺口 review 拍板)
-- 服务器全生命周期管理 (rgs-realm-lifecycle 子模块, 归 rgs-cluster-ops crate, per ARC-051 扩展)
-- 1 张新 Work 表 DDL, 全部在既有 admin_db (per FR-LCM-001, 与 0020_lcm_tables.sql 同库)
-- 范围: lcm_step_execution (LCM 单 step 级别执行记录)
-- 引用: RGS-DB-BAS-001 v0.2 §6.6.2 schema 草案 + 14-分区策略 §7 Work 表 cleanup SOP
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 admin Lead 拍板 4 项问题 (§6.6.2 末) + PH-2 评审
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在 admin Lead 拍板前 apply 到生产
-- ⚠️ 拍板后: 移除本注释 + apply + 同步 RGS-DB-BAS-001 v0.2 §6.6.2 → v0.3 移表到 §3.3 Work 表

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 本 migration 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_lcm:rgs_lcm@localhost:5544/admin_db
-- 若只改本文件 schema、未跑 sqlx prepare, CI 会以 "no cached query for ..." 阻断合并

-- ============================================================
-- 硬约束 (per RGS-BAS-007 + RGS-DB-BAS-001 v0.2)
-- ============================================================
-- FR-LCM-001: 1 张新表全部在 admin_db; 不新建独立数据库
-- FR-LCM-002: 阶段/step 变更全流程留痕既有 admin_db.audit_log (per RGS-REV-007 hash 链防篡改), 新表不绕过也不复制
-- RGS-BAS-007 §2: snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键
-- RGS-BAS-007 §4: Work 表不分区, 走 cleanup job (per 14-§7, 24h TTL)
-- RGS-BAS-007 §1.5 + RGS-DB-BAS-001 v0.2 §6.1: 跨 DB 禁用外键, 应用层校验
-- RGS-SPEC-CROSS-005 §2: 跨 DB 禁用外键, 应用层校验
-- 17-P0-04 + 13-§3.3: 跨表 FK 用 DO + ALTER TABLE 后置; CHECK 约束用 DO + EXCEPTION 幂等块
-- RGS-DB-BAS-001 v0.2 §6.6.2: 归 Work 表 (业务执行中临时存在 + 完成后 24h 清理)

-- ============================================================
-- 表 1/1: lcm_step_execution (per RGS-DB-BAS-001 v0.2 §6.6.2)
-- LCM run 内单 step 执行记录 (per run 多 step: provision / configure / smoke_test / route53_update / load_balance_update / health_check ...)
-- expires_at 24h TTL, cleanup job per 14-§7.2
-- ============================================================

-- 1. CREATE TABLE 主表 (无 FK, 避免 forward ref 问题, per 17-P0-04)
CREATE TABLE IF NOT EXISTS lcm_step_execution (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL,                    -- 跨表 FK → realm_lifecycle_run, 见下方 ALTER
    step_seq INT NOT NULL,                   -- 步骤序号 (在 phase 内, 1..N)
    step_name TEXT NOT NULL,                 -- e.g. 'provision' / 'configure' / 'smoke_test' / 'route53_update' / 'load_balance_update' / 'health_check'
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'succeeded', 'failed', 'skipped')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    attempt_count INT NOT NULL DEFAULT 0
        CHECK (attempt_count >= 0),
    last_error TEXT,                         -- 失败时填, 应用层脱敏
    step_metadata JSONB,                     -- step-specific data (e.g. provision 资源规格 / configure 配置 diff)
    expires_at TIMESTAMPTZ NOT NULL,         -- 24h TTL (应用层算 created_at + INTERVAL '24 hours')
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 幂等性: 同一 run 内 step_seq 唯一
    CONSTRAINT uq_lcm_step_run_seq UNIQUE (run_id, step_seq)
);

-- 2. 索引 (cleanup 加速 + 业务查询)
CREATE INDEX IF NOT EXISTS idx_lcm_step_run_id
    ON lcm_step_execution (run_id);
CREATE INDEX IF NOT EXISTS idx_lcm_step_status_started
    ON lcm_step_execution (status, started_at DESC) WHERE status IN ('pending', 'in_progress');
CREATE INDEX IF NOT EXISTS idx_lcm_step_expires_at
    ON lcm_step_execution (expires_at) WHERE status IN ('pending', 'in_progress', 'succeeded', 'failed', 'skipped');

-- 3. 跨表 FK: run_id → realm_lifecycle_run.id (用 DO + ALTER TABLE 后置, per 17-P0-04)
-- realm_lifecycle_run 在 0020_lcm_tables.sql 创建, 跨 migration 文件 ALTER 是安全的
DO $$
BEGIN
    ALTER TABLE lcm_step_execution
        ADD CONSTRAINT fk_lcm_step_run_id
        FOREIGN KEY (run_id) REFERENCES realm_lifecycle_run(id) ON DELETE CASCADE;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- ============================================================
-- 已知缺口 (待 admin Lead 拍板 4 项问题, per RGS-DB-BAS-001 v0.2 §6.6.2)
-- ============================================================
-- Q1 (admin Lead): 是否实装 lcm_step_execution Work 表?
--     - 拍板"是" → 移除 DRAFT 注释 + apply + 同步 RGS-DB-BAS-001 v0.2 §6.6.2 → v0.3 移表
--     - 拍板"否" → 删除本 migration 文件 + 修订 RGS-DB-BAS-001 v0.2 §6.6.2 为"已确认不实装"
-- Q2 (admin Lead): 保留期 24h vs 7d vs 30d?
--     - 当前草案: 24h (最严, step 执行历史通常 1 小时内完成)
--     - 候选: 7d (留 GM 调查缓冲) / 30d (与 audit_log 一致, 但 audit_log 已独立保留)
-- Q3 (admin Lead): 跨 step 状态共享用 step_metadata JSONB 是否合理?
--     - 当前草案: step_metadata JSONB 自由 schema
--     - 候选: 强 schema (per step_name 拆 6 张 step_*_detail 表) / 完全无状态共享
-- Q4 (admin Lead): 与 admin_backend gRPC 接口集成路径?
--     - 当前草案: admin_backend 加 GetLCMStepExecution / ListLCMStepExecutionsByRun RPC
--     - 候选: 仅 DB 直接读 (gm_backend → admin_db) / 通过 saga 异步通知

-- ============================================================
-- 其他已知缺口 (技术层)
-- ============================================================
-- 1. step_name 枚举值当前是 free text, 建议 PH-2 评审加 CHECK 限定
-- 2. step_metadata JSONB 无 JSON schema 约束, 应用层负责
-- 3. last_error TEXT 大小无上限, 应用层负责 (建议 ≤ 8KB)
-- 4. cleanup job 本身在 cluster-ops/src/jobs/cleanup_lcm.rs 待实装 (per 14-§7.2)
-- 5. 跨 run 的 step_name 一致性靠应用层保证, DB 层不强制
