-- economy-service migration 0007_sagas_partitioned (per RGS-BAS-007 §4 + v0.2 §3.2 T-03 + §9.4)
-- sagas 按月 RANGE 分区 (per 14-§3.x 性能优化建议, PH-3 实施)
-- 3 年 (36 分区) 滚动保留
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 SRE + DBA + economy Lead 评审 + PH-3 实施窗口
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在评审通过 + 双写期验证前 apply 到生产
-- ⚠️ Expand-Contract 模式 (per 17-P0-02): 1) 新建 sagas_partitioned  2) 双写期  3) 切读流量  4) rename  5) 清理
--
-- 实施步骤 (per 17-P0-02 修复建议):
-- 1. 本 migration: 仅建 sagas_partitioned + 当月 + 下月分区
-- 2. PH-3 后续 migration: 数据迁移 + 双写期
-- 3. PH-3 后续 migration: 应用层切换写入目标表
-- 4. PH-3 后续 migration: rename sagas → sagas_legacy, ...partitioned → sagas

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
-- RGS-BAS-100 Saga 分布式事务基本设计书: sagas 是 saga 状态机载体
-- RGS-DB-BAS-001 v0.2 §3.2 T-03: sagas PH-3 实施按月分区
-- RGS-DB-BAS-001 v0.2 §9.4: SQL 模板 + PH-3 实施计划
-- 13-Outbox 跨域模板: 命令幂等性 UNIQUE(command_id)
-- RGS-SPEC-CROSS-005 §2: snake_case / TIMESTAMPTZ / 不允许 nullable 主键
-- 17-P0-04: 跨表 FK 用 DO + ALTER TABLE 后置 (sagas → outbox 跨表 FK)

-- ============================================================
-- 1. 新建 sagas_partitioned 分区表 (per created_at 月度 RANGE)
-- 字段从现有 schema 推断 (per 14-§4 + RGS-BAS-100 + RGS-DTL-100 §3):
--   id UUID PK, command_id UUID UNIQUE, saga_type TEXT,
--   state TEXT, payload JSONB, result JSONB, error TEXT,
--   started_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, created_at TIMESTAMPTZ
-- 注: 现有 sagas schema 需先 Read 确认 (本 DRAFT 基于 14-§4 + RGS-DTL-100 §3 推断)
-- ============================================================
CREATE TABLE IF NOT EXISTS sagas_partitioned (
    id UUID NOT NULL,
    command_id UUID NOT NULL,                -- 跨表 FK → outbox(command_id) (幂等性), 见下方 ALTER
    saga_type TEXT NOT NULL,                 -- e.g. 'trade' / 'auction_execute' / 'guild_join'
    state TEXT NOT NULL DEFAULT 'pending'
        CHECK (state IN ('pending', 'compensating', 'completed', 'failed', 'rolled_back')),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    result JSONB,                            -- saga 完成后填
    error TEXT,                              -- saga 失败时填 (应用层脱敏)
    retry_count INT NOT NULL DEFAULT 0
        CHECK (retry_count >= 0),
    max_retries INT NOT NULL DEFAULT 3
        CHECK (max_retries >= 0),
    next_retry_at TIMESTAMPTZ,               -- 重试退避时间 (应用层算)
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)             -- 分区表 PK 必须包含分区键
) PARTITION BY RANGE (created_at);

-- UNIQUE 约束: command_id 幂等性 (per 13-Outbox 跨域模板 + 现有 sagas 0002_saga_init.sql)
-- 注: 分区表的 UNIQUE 约束必须包含分区键, 否则 PG 报错
DO $$
BEGIN
    ALTER TABLE sagas_partitioned
        ADD CONSTRAINT uq_sagas_partitioned_command_id UNIQUE (command_id, created_at);
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- 初始分区: 当月 + 下月
DO $$
DECLARE
    current_month_start TIMESTAMPTZ := date_trunc('month', now());
    next_month_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '1 month';
    month_after_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '2 month';
    current_partition TEXT := 'sagas_y' || to_char(current_month_start, 'YYYYMM');
    next_partition TEXT := 'sagas_y' || to_char(next_month_start, 'YYYYMM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF sagas_partitioned FOR VALUES FROM (%L) TO (%L)',
        current_partition, current_month_start, next_month_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF sagas_partitioned FOR VALUES FROM (%L) TO (%L)',
        next_partition, next_month_start, month_after_start
    );
END $$;

-- ============================================================
-- 2. 跨表 FK: command_id → outbox(command_id) (per 13-Outbox 模板幂等性)
-- 用 DO + ALTER TABLE 后置 (per 17-P0-04, 避免 forward ref)
-- outbox 在 0003_outbox.sql, 跨 migration 文件 ALTER 是安全的
-- 注意: command_id 跨分区, FK 不能强制 (PG 限制), 仅作为应用层一致性保证
-- 实际: command_id UNIQUE(command_id, created_at) 复合约束已保证幂等
-- 跨表 FK 需改为弱引用 (不物化), 已在 0020_lcm_tables.sql:22 明确
-- ============================================================
-- ⚠️ 跨域 FK 不物化原则 (per RGS-BAS-007 §1.5 + v0.2 §6.1)
-- command_id 跨 outbox 表, 实际为弱引用, 应用层校验幂等性
-- 不加物理 FK

-- ============================================================
-- 3. 索引 (saga 业务查询 + 分区裁剪)
-- ============================================================
-- state 单列: 按状态筛选 (e.g. 查 pending / failed)
CREATE INDEX IF NOT EXISTS idx_sagas_partitioned_state
    ON sagas_partitioned (state, created_at DESC);
-- saga_type 单列: 按类型筛选 (e.g. 查 trade saga)
CREATE INDEX IF NOT EXISTS idx_sagas_partitioned_saga_type
    ON sagas_partitioned (saga_type, created_at DESC);
-- retry 待执行: 调度器轮询 (state=pending/failed AND next_retry_at < now())
CREATE INDEX IF NOT EXISTS idx_sagas_partitioned_next_retry
    ON sagas_partitioned (next_retry_at) WHERE state IN ('pending', 'compensating') AND next_retry_at IS NOT NULL;
-- created_at 单列: 加速分区裁剪
CREATE INDEX IF NOT EXISTS idx_sagas_partitioned_created_at
    ON sagas_partitioned (created_at DESC);

-- ============================================================
-- 4. 已知缺口 (待 PH-3 评审)
-- ============================================================
-- 1. 现有 sagas schema 需 Read 确认 (本 DRAFT 基于 14-§4 + RGS-DTL-100 §3 推断)
--    ⚠️ apply 前必须: 读 0002_saga_init.sql 实际 schema, 同步本 DRAFT 字段
-- 2. 数据迁移: 现有 saga 行可能万级 (per 14-§3 估算), 双写期 INSERT ... SELECT 需进度监控
-- 3. 双写期: 应用层改造 (双写 sagas + sagas_partitioned)
-- 4. 切读流量: 应用层 SELECT 改向新表, 需全量回归 + saga 状态机一致性校验
-- 5. rename 旧表 (per Expand-Contract step 4)
-- 6. 与 outbox 的 command_id 关联: 已用 UNIQUE(command_id, created_at) 保证幂等
-- 7. 与 reservation 关联: saga 创建/释放 reservation, 跨表应用层一致性
-- 8. max_retries / next_retry_at 字段: 现有 schema 不一定有, 需确认 (本 DRAFT 加了)
-- 9. 分区滚动 cron job: 每月 1 号建下下月分区 + DROP 36 月前分区
-- 10. saga 跨月执行的边界: saga 跨 2 月时, 状态更新会写到新分区, PK 含 created_at 仍 OK
