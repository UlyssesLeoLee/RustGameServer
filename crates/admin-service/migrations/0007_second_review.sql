-- admin-service migration 0007_second_review (per RGS-INC-001 v0.3 §X.5 + §X.2 RequireSecondReview)
-- second_review 表: COC 复核工作流数据载体 (RequireSecondReview 决策的暂存层)
-- 库: admin_db (per ARC-008 5 独立 DB 原则)
-- 范围: second_review (1 张新 Transaction 表 DDL, 含 PRIMARY KEY + 索引)
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 admin 域 Lead 拍板 (per RGS-INC-001 v0.3 §X.8)
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在 admin Lead 拍板前 apply 到生产
-- ⚠️ 拍板后: 移除本注释 + apply + 同步 RGS-INC-001 v0.3 §X.5 状态

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_admin:rgs_admin@localhost:5544/admin_db
-- 若只改本文件 schema、未跑 sqlx prepare, CI 会以 "no cached query for ..." 阻断合并

-- ============================================================
-- 硬约束 (per RGS-INC-001 v0.3 §X.5 + §X.6 + RGS-SPEC-CROSS-005)
-- ============================================================
-- §X.5 状态机: pending → approved (SuperAdmin 通过) / pending → rejected (SuperAdmin 拒绝)
-- §X.5 批准后: gm-backend 异步执行原 GM 操作 (重放 original_request), 写 audit_log
-- §X.5 SLA: 默认 24h 复核窗口, 超时自动 reject (per batch 域 cron 每日 03:00 UTC 扫表)
-- §X.6 第 3 条: WASM 决策不可落库 — WASM 不直接写 second_review, 仅返 decision; 真正写库由 Rust 走现有路径
-- §X.6 第 7 条: 决策可重放 — params_hash + module_version + module_hash + decision 4 字段全落 second_review
-- RGS-SPEC-CROSS-005 §2: snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键
-- RGS-SPEC-CROSS-005 §2: 跨 DB 禁用外键, 应用层校验 (本表不引用跨 DB 表, 安全)
-- 17-P0-04 + 13-§3.3: 跨表 FK 用 DO + ALTER TABLE 后置; CHECK 约束用 DO + EXCEPTION 幂等块
-- RGS-BAS-007: Transaction 表 (事件流水, append-only, 永久保留 per NFR-SE-010 双层审计)
-- RGS-INC-001 v0.3 §X.9 已知缺口: second_review SQL migration 缺失, 本文件补齐

-- ============================================================
-- 表 1/1: second_review (per RGS-INC-001 v0.3 §X.5)
-- COC 复核工作流数据载体; review_id UUID PK
-- status 枚举: pending (待复核) / approved (通过) / rejected (拒绝/超时)
-- coc_* 字段: WASM module 决策时的元信息 (per §X.6 第 7 条 决策可重放)
-- original_request JSONB: gm_handlers 入口 request 完整 payload (用于批准后重放)
-- reviewer_* 字段: SuperAdmin 复核者信息 (pending 时为 NULL)
-- trace_id: OTel trace_id (per §X.6 第 6 条 全链路 trace 透传)
-- ============================================================
CREATE TABLE IF NOT EXISTS second_review (
    review_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id          UUID NOT NULL,                                       -- gm_handlers 入口的 request_id
    actor_id            UUID NOT NULL,                                       -- 操作者 admin_id
    action              TEXT NOT NULL,                                       -- "player.ban" / "economy.grant" / ...
    target_id           TEXT NOT NULL,                                       -- 目标 account_id / ...
    coc_decision        TEXT NOT NULL,                                       -- "RequireSecondReview"
    coc_reason          TEXT NOT NULL,                                       -- WASM 决策理由
    coc_module_version  TEXT NOT NULL,                                       -- 决策时加载的 module version
    coc_module_hash     TEXT NOT NULL,                                       -- 决策时加载的 module sha256
    coc_params_hash     TEXT NOT NULL,                                       -- CocPolicyInput 序列化 SHA-256
    original_request    JSONB NOT NULL,                                      -- gm_handlers 入口 request 完整 payload
    status              TEXT NOT NULL DEFAULT 'pending',                     -- pending / approved / rejected
        CHECK (status IN ('pending', 'approved', 'rejected')),
    reviewer_id         UUID,                                                -- 复核者 admin_id (pending 时 NULL)
    reviewed_at         TIMESTAMPTZ,                                         -- 复核完成时间 (pending 时 NULL)
    review_comment      TEXT,                                                -- 复核者备注
    trace_id            TEXT NOT NULL,                                       -- OTel trace_id
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 索引: (status, created_at) 复合索引, 用于 batch 域 cron 扫表超时 (per §X.5 SLA 24h)
-- 也是 pending 列表查询的常用路径 (gm-backend SuperAdmin 复核界面)
CREATE INDEX IF NOT EXISTS idx_second_review_status_created
    ON second_review (status, created_at)
    WHERE status = 'pending';

-- 索引: 按 request_id 查 (同一 GM 操作的复核记录去重 / 关联)
-- 不在 §X.5 原始 schema 中, 但 gm-backend 批准后重放原 GM 操作 (per §X.5 第 3 步) 需要 by-request-id 查
-- 加在 DO + EXCEPTION 幂等块内, 防止重复创建
DO $$
BEGIN
    CREATE INDEX idx_second_review_request_id
        ON second_review (request_id);
EXCEPTION
    WHEN duplicate_table THEN NULL;
    WHEN duplicate_object THEN NULL;
END $$;

-- 索引: 按 trace_id 查 (OTel trace 关联 / 调试 / 跨域 saga 跟踪)
-- 不在 §X.5 原始 schema 中, 但 per §X.6 第 6 条 全链路 trace 透传, trace_id 是常用排障字段
DO $$
BEGIN
    CREATE INDEX idx_second_review_trace_id
        ON second_review (trace_id)
        WHERE trace_id IS NOT NULL;
EXCEPTION
    WHEN duplicate_table THEN NULL;
    WHEN duplicate_object THEN NULL;
END $$;

-- ============================================================
-- 已知缺口 (待 admin 域 Lead 拍板, per RGS-INC-001 v0.3 §X.8 + §X.9)
-- ============================================================
-- Q1 (admin Lead): status 枚举是否需要 'expired' (超时自动 reject 后的终态)?
--     - 当前草案: 超时 reject 与主动 reject 共用 'rejected', 区分靠 reviewed_at NULL/最近 + trace
--     - 候选: 拆 'rejected' 为 'rejected' (主动) / 'expired' (超时) 两个终态
-- Q2 (admin Lead): original_request JSONB 是否需要 schema 约束?
--     - 当前草案: 自由 JSONB, 应用层负责 (per RGS-SPEC-CROSS-005 §3 跨域 JSON 规范)
--     - 候选: 加 JSON schema (action + target + params 三段固定结构)
-- Q3 (admin Lead): 是否需要 reviewer_ip / user_agent 字段 (审计加强)?
--     - 当前草案: 不在 §X.5 原始 schema 内
--     - 候选: 加 INET + TEXT 字段, 与 audit_log 一致
-- Q4 (admin Lead): coc_decision 是否需要 CHECK 限定枚举 (目前仅 "RequireSecondReview" 1 值)?
--     - 当前草案: TEXT 自由, 未来扩展 (e.g. "Deny" / "Allow" 直接走 audit_log) 灵活
--     - 候选: 加 CHECK (coc_decision IN ('RequireSecondReview', 'Deny', 'Allow'))
-- Q5 (admin Lead): SLA 24h 阈值是否需字段化 (e.g. expires_at TIMESTAMPTZ)?
--     - 当前草案: 24h hardcode 在 batch 域 cron 扫描逻辑
--     - 候选: 加 expires_at = created_at + INTERVAL '24 hours', cron 扫 expires_at < now()

-- ============================================================
-- 其他已知缺口 (技术层)
-- ============================================================
-- 1. PostgreSQL 13+ 内置 gen_random_uuid() (pgcrypto 仅 PG < 13 需要)
-- 2. actor_id / reviewer_id 是同 DB 软引用 (admin_db.admin_users.id), 单 DB 内应用层校验
-- 3. request_id 是应用层 UUID, 不强制唯一 (同一 request 可发多次复核请求, 罕见)
-- 4. original_request JSONB 大小无上限, 应用层负责 (建议 ≤ 64KB, per RGS-SPEC-CROSS-005 §3)
-- 5. coc_params_hash 不加唯一约束 (允许同 params 多次复核, 取决于 policy 决策)
-- 6. status='pending' 状态无 TTL, 依赖 batch 域 cron 每日 03:00 UTC 扫表 (per §X.5 SLA)
-- 7. 终态 'approved'/'rejected' 永久保留 (per NFR-SE-010 双层审计, 不允许 DELETE/TRUNCATE)
