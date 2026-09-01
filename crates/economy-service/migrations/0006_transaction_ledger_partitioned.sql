-- economy-service migration 0006_transaction_ledger_partitioned (per RGS-BAS-007 §4 + 17-P0-02 模板 + v0.2 §3.2 T-02 + §9.4)
-- transaction_ledger 按月 RANGE 分区 (per 14-§3.x 性能优化建议, PH-3 实施)
-- 3 年 (36 分区) 滚动保留 (per 业务需求, 与 audit_log 一致)
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 SRE + DBA + economy Lead 评审 + PH-3 实施窗口
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在评审通过 + 双写期验证前 apply 到生产
-- ⚠️ Expand-Contract 模式: 1) 新建 transaction_ledger_partitioned  2) 双写期  3) 切读流量  4) rename 旧表  5) 清理
--
-- 实施步骤 (per 17-P0-02 修复建议):
-- 1. 本 migration: 仅建 transaction_ledger_partitioned + 当月 + 下月分区
-- 2. PH-3 后续 migration: 数据迁移 (双写期 + 后台 batch copy)
-- 3. PH-3 后续 migration: 应用层切换写入目标表
-- 4. PH-3 后续 migration: rename transaction_ledger → transaction_ledger_legacy, ...partitioned → transaction_ledger
-- 5. PH-3 后续: 保留 transaction_ledger_legacy 30 天后 DROP

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_economy:rgs_economy@localhost:5544/economy_db

-- ============================================================
-- 硬约束
-- ============================================================
-- RGS-BAS-007 §4: 高频 append-only 表应按月 RANGE 分区
-- RGS-DB-BAS-001 v0.2 §3.2 T-02: transaction_ledger PH-3 实施按月分区
-- RGS-DB-BAS-001 v0.2 §9.4: SQL 模板 + PH-3 实施计划
-- RGS-SPEC-CROSS-005 §2: snake_case 字段名 / TIMESTAMPTZ 时间戳
-- 17-P0-04: 跨表 FK 用 DO + ALTER TABLE 后置 (本表无跨表 FK, 但 OCC 模式需保留)
-- OCC 模式: balance_after + saga_id + version 列必须保留 (per RGS-DB-BAS-001 v0.2 §6.2)

-- ============================================================
-- 1. 新建 transaction_ledger_partitioned 分区表 (per created_at 月度 RANGE)
-- 字段从现有 schema 推断 (per 14-§4 + RGS-DTL-100 §6):
--   id UUID PK, account_id UUID, amount BIGINT, tx_type TEXT,
--   balance_after BIGINT, saga_id UUID, created_at TIMESTAMPTZ DEFAULT now()
-- 注: 现有 transaction_ledger schema 需先 Read 确认 (本 DRAFT 基于 14-§4 + RGS-DTL-100 §6 推断)
-- ============================================================
CREATE TABLE IF NOT EXISTS transaction_ledger_partitioned (
    id UUID NOT NULL,
    account_id UUID NOT NULL,                -- 弱引用 accounts(id) (同库内可不物化)
    amount BIGINT NOT NULL,                  -- 正=入账 / 负=出账
    tx_type TEXT NOT NULL,                   -- e.g. 'deposit' / 'withdraw' / 'saga_credit' / 'saga_debit' / 'transfer'
    balance_after BIGINT NOT NULL,           -- OCC 校验: balance_after = balance_prev + amount
    saga_id UUID,                            -- 跨 saga 关联 (saga 启动时填)
    related_ledger_id UUID,                  -- 转账场景: 关联对端 ledger 行
    metadata JSONB,                          -- tx-specific data (e.g. reason / source)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)             -- 分区表 PK 必须包含分区键
) PARTITION BY RANGE (created_at);

-- 初始分区: 当月 + 下月
DO $$
DECLARE
    current_month_start TIMESTAMPTZ := date_trunc('month', now());
    next_month_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '1 month';
    month_after_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '2 month';
    current_partition TEXT := 'transaction_ledger_y' || to_char(current_month_start, 'YYYYMM');
    next_partition TEXT := 'transaction_ledger_y' || to_char(next_month_start, 'YYYYMM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF transaction_ledger_partitioned FOR VALUES FROM (%L) TO (%L)',
        current_partition, current_month_start, next_month_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF transaction_ledger_partitioned FOR VALUES FROM (%L) TO (%L)',
        next_partition, next_month_start, month_after_start
    );
END $$;

-- ============================================================
-- 2. 索引 (业务查询 + 分区裁剪优化)
-- ============================================================
-- account_id 单列: 按账户查流水 (常见)
CREATE INDEX IF NOT EXISTS idx_ledger_partitioned_account_id
    ON transaction_ledger_partitioned (account_id, created_at DESC);
-- saga_id 单列: 按 saga 查流水 (saga 演练 / 故障排查)
CREATE INDEX IF NOT EXISTS idx_ledger_partitioned_saga_id
    ON transaction_ledger_partitioned (saga_id) WHERE saga_id IS NOT NULL;
-- tx_type 单列: 按类型筛选 (e.g. 统计 deposit 总量)
CREATE INDEX IF NOT EXISTS idx_ledger_partitioned_tx_type
    ON transaction_ledger_partitioned (tx_type, created_at DESC);
-- created_at 单列: 加速分区裁剪 (PG 自动用, 显式建更明确)
CREATE INDEX IF NOT EXISTS idx_ledger_partitioned_created_at
    ON transaction_ledger_partitioned (created_at DESC);

-- ============================================================
-- 3. 已知缺口 (待 PH-3 评审)
-- ============================================================
-- 1. 现有 transaction_ledger schema 需 Read 确认 (本 DRAFT 基于 14-§4 + RGS-DTL-100 §6 推断)
--    ⚠️ apply 前必须: 读 0001_init.sql:42-60 实际 schema, 同步本 DRAFT 字段
-- 2. 数据迁移: 现有数据可能亿级 (per 14-§3 估算), 双写期 INSERT ... SELECT 需进度监控
-- 3. 双写期: 应用层改造 (双写 transaction_ledger + transaction_ledger_partitioned)
-- 4. 切读流量: 应用层 SELECT 改向新表, 需全量回归 + 历史数据一致性校验
-- 5. rename 旧表 (per Expand-Contract step 4)
-- 6. 应用层 OCC 模式 (WHERE version = ?) 是否需新增 version 列? (本 DRAFT 未加, 待确认)
-- 7. balance_after CHECK 约束: balance_after + related_balance_after = 0 (转账场景), 应用层保证
-- 8. 分区滚动 cron job: 每月 1 号建下下月分区 + DROP 36 月前分区
-- 9. 现有 saga_id 与 sagas 表的跨表关联, 是否需要在分区表也保留 (per FR-LCM 风格)
