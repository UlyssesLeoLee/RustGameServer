-- admin-service migration 0006_audit_log_partitioned (per RGS-BAS-007 §4 + 17-P0-02 + v0.2 §3.2 T-05)
-- audit_log 按月 RANGE 分区（per RGS-REQ-001 §NFR-SE-010 双层审计 3 年保留期）
-- 36 个分区滚动保留（3 年 = 36 月）
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 SRE + DBA + admin Lead 评审 + PH-2 实施窗口
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在评审通过 + 双写期验证前 apply 到生产
-- ⚠️ Expand-Contract 模式: 1) 新建 audit_log_partitioned  2) 双写期  3) 切读流量  4) rename 旧表  5) 清理
--
-- 实施步骤 (per 17-P0-02 修复建议):
-- 1. 本 migration: 仅建 audit_log_partitioned + 当月 + 下月分区
-- 2. PH-2 后续 migration: 数据迁移 (双写期 + 后台 batch copy)
-- 3. PH-2 后续 migration: 应用层切换写入目标表 (从 audit_log → audit_log_partitioned)
-- 4. PH-2 后续 migration: rename audit_log → audit_log_legacy, audit_log_partitioned → audit_log
-- 5. PH-2 后续: 保留 audit_log_legacy 30 天后 DROP
-- 6. PH-2 后续: 实施分区滚动 cron job (per 14-§5 + RGS-BAS-007 §4)

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_admin:rgs_admin@localhost:5544/admin_db
-- 若只改本文件 schema、未跑 sqlx prepare, CI 会以 "no cached query for ..." 阻断合并

-- ============================================================
-- 硬约束
-- ============================================================
-- RGS-BAS-007 §4: audit_log 应按月 RANGE 分区, 3 年 (36 分区) 滚动保留
-- RGS-REQ-001 §NFR-SE-010: 双层审计, 3 年保留
-- RGS-SEC-100 §7: hash 链防篡改 (per hash + prev_hash UNIQUE), 禁 UPDATE/DELETE 触发器
-- RGS-SPEC-CROSS-005 §2: snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键
-- 17-P0-04 + 13-§3.3: 跨表 FK 用 DO + ALTER TABLE 后置; CHECK 约束用 DO + EXCEPTION 幂等块
-- 17-P0-02: Expand-Contract 模式, 本 migration 仅建分区表, 数据迁移在后续 migration

-- ============================================================
-- 1. 新建 audit_log_partitioned 分区表 (per created_at 月度 RANGE)
-- 字段与 audit_log 完全一致 (LIKE INCLUDING ALL), 仅 created_at 触发分区
-- 不在 CREATE 时加触发器 (避免 forward ref), 在下方 ALTER TABLE 加
-- ============================================================
CREATE TABLE IF NOT EXISTS audit_log_partitioned (LIKE audit_log INCLUDING ALL)
    PARTITION BY RANGE (created_at);

-- 初始分区: 当月 + 下月 (per 14-§2.1 模式, per 0020_lcm_tables.sql:51-67)
DO $$
DECLARE
    current_month_start TIMESTAMPTZ := date_trunc('month', now());
    next_month_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '1 month';
    month_after_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '2 month';
    current_partition TEXT := 'audit_log_y' || to_char(current_month_start, 'YYYYMM');
    next_partition TEXT := 'audit_log_y' || to_char(next_month_start, 'YYYYMM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF audit_log_partitioned FOR VALUES FROM (%L) TO (%L)',
        current_partition, current_month_start, next_month_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF audit_log_partitioned FOR VALUES FROM (%L) TO (%L)',
        next_partition, next_month_start, month_after_start
    );
END $$;

-- ============================================================
-- 2. 禁 UPDATE/DELETE 触发器 (per RGS-SEC-100 §7)
-- 用 DO + ALTER TABLE 后置 (per 17-P0-04, 避免 CREATE 内 forward ref)
-- 触发器函数从 audit_log 复用 (audit_log_no_modify), 无需重建
-- ============================================================
DO $$
BEGIN
    -- audit_log_no_modify 函数在 0001_init.sql:36-40 已建, 直接复用
    -- 仅 DROP + CREATE 触发器到新分区表
    EXECUTE 'DROP TRIGGER IF EXISTS audit_log_no_update ON audit_log_partitioned';
    EXECUTE 'CREATE TRIGGER audit_log_no_update
        BEFORE UPDATE ON audit_log_partitioned
        FOR EACH ROW EXECUTE FUNCTION audit_log_no_modify()';

    EXECUTE 'DROP TRIGGER IF EXISTS audit_log_no_delete ON audit_log_partitioned';
    EXECUTE 'CREATE TRIGGER audit_log_no_delete
        BEFORE DELETE ON audit_log_partitioned
        FOR EACH ROW EXECUTE FUNCTION audit_log_no_modify()';
END $$;

-- ============================================================
-- 3. 索引 (从 audit_log 复用模式: actor_id / action / created_at)
-- 必加 created_at 索引, 加速分区裁剪 (partition pruning)
-- 必加 created_at + actor_id 复合索引, 加速按 actor + 时间范围查询
-- ============================================================
-- 注意: LIKE INCLUDING ALL 已包含原 audit_log 的索引定义, 但分区表的索引需要显式创建
-- (LIKE INCLUDING ALL 不复制索引, 仅复制列 + 约束)
CREATE INDEX IF NOT EXISTS idx_audit_partitioned_actor_id
    ON audit_log_partitioned (actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_partitioned_action
    ON audit_log_partitioned (action);
CREATE INDEX IF NOT EXISTS idx_audit_partitioned_created_at
    ON audit_log_partitioned (created_at DESC);
-- 复合索引: 加速按 actor + 时间范围查询 (常见审计查询)
CREATE INDEX IF NOT EXISTS idx_audit_partitioned_actor_created_at
    ON audit_log_partitioned (actor_id, created_at DESC);

-- ============================================================
-- 4. 已知缺口 (待 PH-2 评审)
-- ============================================================
-- 1. 数据迁移 (历史数据 0001_init.sql:23-32 已有, 可能百万~亿行): 后续 migration 用 INSERT ... SELECT + 进度监控
-- 2. 双写期: 应用层改造 (双写 audit_log + audit_log_partitioned), 需 RGS-IMPL-005 §3 编码规范升版
-- 3. 切读流量: 应用层 SELECT FROM audit_log → SELECT FROM audit_log_partitioned, 需全量回归测试
-- 4. rename: ALTER TABLE audit_log RENAME TO audit_log_legacy + audit_log_partitioned RENAME TO audit_log
--    ⚠️ rename 后触发器函数 audit_log_no_modify 仍指向原表, 需 DROP + 重建触发器
-- 5. audit_log_legacy 保留 30 天后 DROP (合规要求 NFR-SE-010 双层审计 30 天缓冲)
-- 6. 分区滚动 cron job: 每月 1 号建下下月分区 + DROP 36 月前分区 (per 14-§5 + RGS-BAS-007 §4)
-- 7. 现有审计通路验证: 双写期后抽样对比 audit_log_legacy 与 audit_log_partitioned 行数 + hash 链一致性
-- 8. 应用层 SELECT FOR UPDATE 锁 latest 行 (per 0002_audit.sql:7 注释) 是否需要适配分区表
