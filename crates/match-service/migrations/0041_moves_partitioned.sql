-- match-service migration 0041_moves_partitioned (per RGS-BAS-007 §4 + 17-P1-07 + v0.2 §3.2 T-04 + §9.4)
-- moves 按月 RANGE 分区 (per 14-§3.5 P1-07 性能优化建议, PH-3 实施)
-- 1 年 (12 分区) 滚动保留 (per 14-§3.5 业务: 老对局 moves 几乎不查, 除 GM 调查)
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 SRE + DBA + match Lead 评审 + PH-3 实施窗口
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在评审通过 + 双写期验证前 apply 到生产
-- ⚠️ Expand-Contract 模式 (per 17-P0-02): 1) 新建 moves_partitioned  2) 双写期  3) 切读流量  4) rename  5) 清理
--
-- 实施步骤:
-- 1. 本 migration: 仅建 moves_partitioned + 当月 + 下月分区
-- 2. PH-3 后续 migration: 数据迁移 + 双写期
-- 3. PH-3 后续 migration: 应用层切换写入目标表
-- 4. PH-3 后续 migration: rename moves → moves_legacy, ...partitioned → moves
-- 5. PH-3 后续: 保留 moves_legacy 30 天后 DROP

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_match:rgs_match@localhost:5544/match_db

-- ============================================================
-- 硬约束
-- ============================================================
-- RGS-BAS-007 §4: 高频 append-only 表应按月 RANGE 分区
-- 14-§3.5: moves 应按时间分区, 1 年保留期, P1-07 优先级
-- RGS-DB-BAS-001 v0.2 §3.2 T-04: moves PH-3 实施按月分区, **必分区**
-- RGS-DB-BAS-001 v0.2 §9.4: SQL 模板 + PH-3 实施计划
-- 04-Match域 §4.4: moves 字段 (session_id / seq / player_id / move_data JSONB)
-- 17-P0-04: 跨表 FK 用 DO + ALTER TABLE 后置
-- 17-P0-04: 跨表 FK 跨域不物化 (per RGS-BAS-007 §1.5 + v0.2 §6.1)

-- ============================================================
-- 1. 新建 moves_partitioned 分区表 (per occurred_at 月度 RANGE)
-- 注: 用 occurred_at 而非 created_at, 因为 move 的"业务时间"与"写入时间"可能不同
--     (per 14-§3.5 建议 occurred_at 月度分区)
-- ============================================================
CREATE TABLE IF NOT EXISTS moves_partitioned (
    id UUID NOT NULL,
    session_id UUID NOT NULL,                -- 弱引用 game_sessions(id) (跨表, 应用层校验)
    seq BIGINT NOT NULL,                     -- move 序号 (在 session 内单调递增)
    player_id UUID NOT NULL,                 -- 跨域弱引用 (player_db.players.id, 不物化 FK)
    move_data JSONB NOT NULL,                -- 业务数据 (e.g. 出的牌 / 攻击目标)
    move_type TEXT NOT NULL,                 -- e.g. 'play_card' / 'attack' / 'defend' / 'pass'
    duration_ms INT,                         -- 客户端操作耗时 (性能分析用)
    client_version TEXT,                     -- 客户端版本 (per user agent 字段推断)
    occurred_at TIMESTAMPTZ NOT NULL,        -- 业务发生时间 (客户端时间, 允许轻微漂移)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),  -- 服务端接收时间
    PRIMARY KEY (id, occurred_at)            -- 分区表 PK 必须包含分区键
) PARTITION BY RANGE (occurred_at);

-- 初始分区: 当月 + 下月
DO $$
DECLARE
    current_month_start TIMESTAMPTZ := date_trunc('month', now());
    next_month_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '1 month';
    month_after_start TIMESTAMPTZ := date_trunc('month', now()) + INTERVAL '2 month';
    current_partition TEXT := 'moves_y' || to_char(current_month_start, 'YYYYMM');
    next_partition TEXT := 'moves_y' || to_char(next_month_start, 'YYYYMM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF moves_partitioned FOR VALUES FROM (%L) TO (%L)',
        current_partition, current_month_start, next_month_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF moves_partitioned FOR VALUES FROM (%L) TO (%L)',
        next_partition, next_month_start, month_after_start
    );
END $$;

-- ============================================================
-- 2. 索引 (业务查询 + 分区裁剪)
-- ============================================================
-- session_id 单列: 按 session 查 moves (回放场景)
CREATE INDEX IF NOT EXISTS idx_moves_partitioned_session_id
    ON moves_partitioned (session_id, seq);
-- player_id 单列: 按 player 查 moves (个人战绩)
CREATE INDEX IF NOT EXISTS idx_moves_partitioned_player_id
    ON moves_partitioned (player_id, occurred_at DESC);
-- move_type 单列: 按类型筛选 (e.g. 统计 play_card 频次)
CREATE INDEX IF NOT EXISTS idx_moves_partitioned_move_type
    ON moves_partitioned (move_type, occurred_at DESC);
-- occurred_at 单列: 加速分区裁剪
CREATE INDEX IF NOT EXISTS idx_moves_partitioned_occurred_at
    ON moves_partitioned (occurred_at DESC);
-- move_data JSONB GIN: 业务复杂查询 (e.g. 查"所有攻击过某 NPC 的 moves")
CREATE INDEX IF NOT EXISTS idx_moves_partitioned_move_data_gin
    ON moves_partitioned USING GIN (move_data);

-- ============================================================
-- 3. 已知缺口 (待 PH-3 评审)
-- ============================================================
-- 1. 现有 moves schema 需 Read 确认 (本 DRAFT 基于 14-§3.5 + 04-Match域 §4.4 推断)
--    ⚠️ apply 前必须: 读 match-service/migrations/0001_init.sql:5-30 实际 moves 表 schema, 同步本 DRAFT 字段
-- 2. 数据迁移: moves 数据量可能亿级/年 (per 14-§3.5 估算), 双写期 INSERT ... SELECT 需进度监控
-- 3. 双写期: 应用层改造 (双写 moves + moves_partitioned)
-- 4. 切读流量: 应用层 SELECT 改向新表, 需全量回归 + 性能 benchmark
-- 5. rename 旧表 (per Expand-Contract step 4)
-- 6. session_id 跨表 FK 不物化 (per RGS-BAS-007 §1.5), 应用层校验 session 存在
-- 7. player_id 跨域 FK 不物化 (per RGS-BAS-007 §1.5), 应用层校验 player 存在
-- 8. 客户端时间漂移: occurred_at 由客户端提供, 允许漂移, 但需与 created_at 偏差告警
-- 9. 分区滚动 cron job: 每月 1 号建下下月分区 + DROP 12 月前分区 (1 年保留期)
-- 10. GM 调查场景: 超过 1 年的 moves 查询, 应用层需提示"已超出保留期, 可能数据缺失"
-- 11. 跨域 saga 关联: 某些 move 触发跨域 saga (per RGS-DTL-038 §6.3 ExecuteAuction saga),
--     是否需要在 move_data JSONB 加 saga_id 引用, 待 match Lead 拍板
