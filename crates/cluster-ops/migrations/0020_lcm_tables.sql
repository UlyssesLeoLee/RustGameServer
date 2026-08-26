-- cluster-ops migration 0020_lcm_tables（per WBS v0.3 L4 WF-1-2068 + RGS-DTL-042 §3 + RGS-SPEC-DTL-042 §3 第 5 条）
-- 服务器全生命周期管理（rgs-realm-lifecycle 子模块，归 rgs-cluster-ops crate，per ARC-051 扩展）
-- 6 张新表 DDL，全部在既有 admin_db（per ARC-008 5 独立 DB 原则 + FR-LCM-001），**不**新建独立数据库
-- 范围：M-2068.1 ~ M-2068.4 (6 表 + 索引 + 月度分区)
--
-- ============================================================
-- sqlx prepare 检查（per M-2068.6 + RGS-IMPL-005 BUILD 规范）
-- ============================================================
-- 本 migration 上线前**必须**在本地 PG 演练环境跑：
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git，供 CI 在 SQLX_OFFLINE=true 下编译。
-- 演练 PG 池环境变量：DATABASE_URL=postgres://rgs_lcm:rgs_lcm@localhost:5544/admin_db
-- 若只改本文件 schema、未跑 sqlx prepare，CI 会以 "no cached query for ..." 阻断合并。
--
-- ============================================================
-- 硬约束（per RGS-SPEC-DTL-042 §3 第 5 条 + RGS-IMPL-002 §3 PG 编码规范）
-- ============================================================
-- FR-LCM-001：6 张表全部在 admin_db；不新建独立数据库
-- FR-LCM-002：阶段变更全流程留痕既有 admin_db.audit_log（per RGS-REV-007 hash 链防篡改），新表不绕过也不复制
-- FR-LCM-062：merge_conflict_rule_set_v2 在 locked_at 锁定后不允许运行时修改（DDL 配 locked_at TIMESTAMPTZ + 应用层校验）
-- FR-LCM-081：归档**不**删除数据（archive_policy 表**不**含 DELETE / TRUNCATE 路径，per NFR-SE-010 双层审计）
-- RGS-SPEC-CROSS-005 §2：snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键 / 跨 DB 禁用外键
-- RGS-BAS-007 §4：realm_lifecycle_run 按 created_at 月度范围分区（与既有 admin_db.audit_log 同构，3 年 N+2 滚动保留）
-- NFR-SE-010：GDPR 删除通路 admin_db.audit_log 双层审计，6 张新表**不**复制该通路
-- ============================================================

-- ============================================================
-- 表 1/6: realm_lifecycle_run（per M-2068.1）
-- 主运行记录；按 created_at 月度范围分区；与既有 admin_db.audit_log 同构
-- ============================================================
CREATE TABLE IF NOT EXISTS realm_lifecycle_run (
    id UUID PRIMARY KEY,
    feature_subtype TEXT NOT NULL
        CHECK (feature_subtype IN ('new_realm', 'scale', 'split', 'merge', 'merge_rollback', 'retire', 'archive')),
    realm_id UUID NOT NULL,
    operator_id UUID NOT NULL,
    request_id UUID NOT NULL,
    approval_ref TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed', 'failed', 'rolled_back')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    trace_id TEXT,
    -- 幂等性：同 request_id + operator_id 唯一（per RGS-SPEC-DTL-042 §5 幂等一致性 + 同 RGS-DTL-031 §3.1）
    CONSTRAINT uq_lifecycle_run_request_operator UNIQUE (request_id, operator_id)
) PARTITION BY RANGE (created_at);

-- 月度分区（per RGS-BAS-007 §4 既定分区策略：3 年 36 个分区滚动保留）
-- 初始创建当月 + 下月分区（生产环境由分区滚动定时任务维护）
DO $$
DECLARE
    current_month_start TIMESTAMPTZ := date_trunc('month', now());
    next_month_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '1 month';
    month_after_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '2 month';
    current_partition TEXT := 'realm_lifecycle_run_y' || to_char(current_month_start, 'YYYYMM');
    next_partition TEXT := 'realm_lifecycle_run_y' || to_char(next_month_start, 'YYYYMM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF realm_lifecycle_run FOR VALUES FROM (%L) TO (%L)',
        current_partition, current_month_start, next_month_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF realm_lifecycle_run FOR VALUES FROM (%L) TO (%L)',
        next_partition, next_month_start, month_after_start
    );
END $$;

-- realm_lifecycle_run 索引（per M-2068.4 + RGS-SPEC-CROSS-005 §2.2 命名规范）
CREATE INDEX IF NOT EXISTS idx_lifecycle_run_status_created_at
    ON realm_lifecycle_run (status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_lifecycle_run_realm_id
    ON realm_lifecycle_run (realm_id);
CREATE INDEX IF NOT EXISTS idx_lifecycle_run_feature_subtype
    ON realm_lifecycle_run (feature_subtype, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_lifecycle_run_trace_id
    ON realm_lifecycle_run (trace_id) WHERE trace_id IS NOT NULL;

-- ============================================================
-- 表 2/6: new_realm_plan（per M-2068.2）
-- 新建 realm 计划；同 DB 内 FK 到 realm_lifecycle_run
-- ============================================================
CREATE TABLE IF NOT EXISTS new_realm_plan (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL
        REFERENCES realm_lifecycle_run(id) ON DELETE RESTRICT,
    target_region TEXT NOT NULL,
    target_player_count INT NOT NULL CHECK (target_player_count > 0),
    target_tps INT NOT NULL CHECK (target_tps > 0),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'validated', 'executing', 'done', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_new_realm_plan_run_id ON new_realm_plan (run_id);
CREATE INDEX IF NOT EXISTS idx_new_realm_plan_status ON new_realm_plan (status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_new_realm_plan_target_region ON new_realm_plan (target_region);

-- ============================================================
-- 表 3/6: split_plan（per M-2068.2）
-- 分服计划；source_realm_id 至少分裂为 2 个目标 realm
-- ============================================================
CREATE TABLE IF NOT EXISTS split_plan (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL
        REFERENCES realm_lifecycle_run(id) ON DELETE RESTRICT,
    source_realm_id UUID NOT NULL,
    target_realm_count INT NOT NULL CHECK (target_realm_count >= 2),
    split_strategy TEXT NOT NULL
        CHECK (split_strategy IN ('hash', 'range', 'manual')),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'validated', 'executing', 'done', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_split_plan_run_id ON split_plan (run_id);
CREATE INDEX IF NOT EXISTS idx_split_plan_source_realm_id ON split_plan (source_realm_id);
CREATE INDEX IF NOT EXISTS idx_split_plan_status ON split_plan (status, created_at DESC);

-- ============================================================
-- 表 4/6: merge_conflict_rule_set_v2（per M-2068.2 + FR-LCM-062）
-- 合服冲突规则集 v2；locked_at 锁定后**不**允许运行时修改（应用层校验）
-- ============================================================
CREATE TABLE IF NOT EXISTS merge_conflict_rule_set_v2 (
    id UUID PRIMARY KEY,
    rule_set_version INT NOT NULL CHECK (rule_set_version > 0),
    rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- locked_at 非空时禁止 UPDATE/DELETE（应用层校验 + sqlx save 实现）
    locked_at TIMESTAMPTZ,
    locked_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 同一版本号只能有一份（per FR-LCM-002 + 同 RGS-BAS-007 §2 命名规范）
    CONSTRAINT uq_merge_conflict_rule_set_version UNIQUE (rule_set_version),
    -- 锁定一致性：locked_at 与 locked_by 同步；锁定后禁止再修改
    CONSTRAINT chk_merge_conflict_lock_consistency
        CHECK ((locked_at IS NULL AND locked_by IS NULL) OR (locked_at IS NOT NULL AND locked_by IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS idx_merge_conflict_rule_set_version
    ON merge_conflict_rule_set_v2 (rule_set_version DESC);
CREATE INDEX IF NOT EXISTS idx_merge_conflict_rule_set_locked
    ON merge_conflict_rule_set_v2 (locked_at) WHERE locked_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_merge_conflict_rule_set_rules_gin
    ON merge_conflict_rule_set_v2 USING GIN (rules);

-- ============================================================
-- 表 5/6: retire_plan（per M-2068.3）
-- 退场计划；query_channel_rbac 配置退场后查询通道的允许角色（per FR-LCM-007）
-- 默认 ["cs_agent", "sre", "legal"]，可通过 UPDATE 修改
-- ============================================================
CREATE TABLE IF NOT EXISTS retire_plan (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL
        REFERENCES realm_lifecycle_run(id) ON DELETE RESTRICT,
    target_realm_id UUID NOT NULL,
    archive_threshold_days INT NOT NULL
        CHECK (archive_threshold_days BETWEEN 30 AND 90),
    query_channel_rbac JSONB NOT NULL DEFAULT '["cs_agent", "sre", "legal"]'::jsonb,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'validated', 'executing', 'done', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_retire_plan_run_id ON retire_plan (run_id);
CREATE INDEX IF NOT EXISTS idx_retire_plan_target_realm_id ON retire_plan (target_realm_id);
CREATE INDEX IF NOT EXISTS idx_retire_plan_status ON retire_plan (status, created_at DESC);

-- ============================================================
-- 表 6/6: archive_policy（per M-2068.3 + FR-LCM-081 + NFR-SE-010）
-- 归档策略：仅迁移存储位置，**不**删除数据；N+2 冗余（per RSK-LCM-005 缓解）
-- 3 年热 + 10 年冷（per RGS-SPEC-DTL-042 §8 Gate 证据）
-- 注：本表**不**含 DELETE / TRUNCATE 路径（per NFR-SE-010 双层审计，删除走 admin_db.audit_log）
-- ============================================================
CREATE TABLE IF NOT EXISTS archive_policy (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL UNIQUE,
    hot_storage_tier TEXT NOT NULL
        CHECK (hot_storage_tier IN ('ssd', 'nvme', 'hdd')),
    cold_storage_tier TEXT NOT NULL
        CHECK (cold_storage_tier IN ('object_storage', 'tape', 'glacier')),
    hot_retention_years INT NOT NULL CHECK (hot_retention_years >= 3),
    cold_retention_years INT NOT NULL CHECK (cold_retention_years >= 10),
    n_plus_2_redundancy BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_archive_policy_realm_id ON archive_policy (realm_id);
CREATE INDEX IF NOT EXISTS idx_archive_policy_n_plus_2 ON archive_policy (n_plus_2_redundancy)
    WHERE n_plus_2_redundancy = TRUE;
