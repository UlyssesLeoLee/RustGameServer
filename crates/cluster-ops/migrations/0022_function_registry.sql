-- cluster-ops migration 0022_function_registry (per RGS-INC-001 v0.3 §X.4 + ADR-0020 §3.1 + §15)
-- Function Registry 表: WASM module 注册 + 版本管理 + SHA-256 校验
-- 库: cluster_ops_db (per ARC-008 + ADR-0052 cluster_ops 独立)
-- 范围: function_registry (1 张新 Master 表 DDL, 含 PRIMARY KEY + 索引)
--
-- ⚠️ MIGRATION_STATUS: DRAFT — 待 admin 域 Lead 拍板 (per RGS-INC-001 v0.3 §X.8)
-- ⚠️ 本文件已 commit 到 git (DRAFT 状态), 但**不**在 admin Lead 拍板前 apply 到生产
-- ⚠️ 拍板后: 移除本注释 + apply + 同步 RGS-INC-001 v0.3 §X.4 状态

-- ============================================================
-- sqlx prepare 检查 (per RGS-IMPL-005 BUILD 规范)
-- ============================================================
-- 上线前**必须**在本地 PG 演练环境跑:
--   cargo sqlx prepare --workspace -- --all-targets
-- 然后把生成的 .sqlx/ 目录 commit 进 git, 供 CI 在 SQLX_OFFLINE=true 下编译
-- 演练 PG 池环境变量: DATABASE_URL=postgres://rgs_lcm:rgs_lcm@localhost:5544/cluster_ops_db
-- 若只改本文件 schema、未跑 sqlx prepare, CI 会以 "no cached query for ..." 阻断合并

-- ============================================================
-- 硬约束 (per RGS-INC-001 v0.3 §X.4 + §X.6 + RGS-SPEC-CROSS-005)
-- ============================================================
-- §X.4 第 4 条: WasmHost.load() 校验 module.hash == function_registry.module_sha256, 不一致直接 fail-closed
-- §X.6 第 4 条: Registry SHA-256 校验 (per ADR-0020 §3.1 + §15) — 每次 WasmHost.call 前校验
-- §X.6 第 5 条: 版本热升级 + 即时回滚 — 保留 ≥ 2 历史 module 版本, SuperAdmin 触发回滚只换 module 不重启 svc
-- RGS-SPEC-CROSS-005 §2: snake_case 字段名 / TIMESTAMPTZ 时间戳 / 不允许 nullable 主键
-- RGS-SPEC-CROSS-005 §2: 跨 DB 禁用外键, 应用层校验 (本表不引用跨 DB 表, 安全)
-- 17-P0-04 + 13-§3.3: 跨表 FK 用 DO + ALTER TABLE 后置; CHECK 约束用 DO + EXCEPTION 幂等块
-- RGS-BAS-007: Master 表 (SCD 策略, slowly changing, 不分区不清理)
-- RGS-INC-001 v0.3 §X.9 已知缺口: function_registry SQL migration 缺失, 本文件补齐

-- ============================================================
-- 表 1/1: function_registry (per RGS-INC-001 v0.3 §X.4)
-- WASM module 注册表; PRIMARY KEY (function_id, version) 复合主键
-- function_id 命名空间: e.g. "admin.coc.policy" / "economy.grant_validator" / ...
-- version 语义: "v1" / "v2" / ... (字符串, 允许 semver-like 灵活扩展)
-- status 枚举: "active" (生产可加载) / "rollback" (回滚中) / "disabled" (禁用)
-- prev_version: 回滚目标版本, 仅 status='rollback' 时有值
-- uploaded_by: SuperAdmin 的 admin_id (UUID, 跨 DB 软引用, 应用层校验)
-- module_sha256: SHA-256 of .wasm bytes (16 进 64 字符)
-- ============================================================
CREATE TABLE IF NOT EXISTS function_registry (
    function_id     TEXT NOT NULL,                       -- e.g. "admin.coc.policy"
    version         TEXT NOT NULL,                       -- "v1" / "v2" / ...
    module_sha256   TEXT NOT NULL,                       -- SHA-256 of .wasm bytes (64 hex)
    status          TEXT NOT NULL                        -- "active" / "rollback" / "disabled"
        CHECK (status IN ('active', 'rollback', 'disabled')),
    prev_version    TEXT,                                -- 回滚目标版本 (status='rollback' 时有值)
    uploaded_by     UUID NOT NULL,                       -- SuperAdmin admin_id (跨 DB 软引用, 应用层校验)
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (function_id, version)
);

-- 索引: 按 status 查 (e.g. "查所有 active module" 用于 WasmHost pool 加载)
-- per RGS-INC-001 v0.3 §X.4 原始 schema + §X.6 第 5 条 (status 过滤是热路径)
CREATE INDEX IF NOT EXISTS idx_function_registry_status
    ON function_registry (status)
    WHERE status IN ('active', 'rollback');

-- 索引: 按 function_id 查 (查某 function 的所有历史版本, 用于回滚选版)
-- 不在 §X.4 原始 schema 中, 但回滚流程 (RGS-INC-001 v0.3 §X.4 第 4 步) 需要 list-all-versions
-- 加在 DO + EXCEPTION 幂等块内, 防止重复创建
DO $$
BEGIN
    CREATE INDEX idx_function_registry_function_id
        ON function_registry (function_id);
EXCEPTION
    WHEN duplicate_table THEN NULL;
    WHEN duplicate_object THEN NULL;
END $$;

-- ============================================================
-- 已知缺口 (待 admin 域 Lead 拍板, per RGS-INC-001 v0.3 §X.8 + §X.9)
-- ============================================================
-- Q1 (admin Lead): status='rollback' 时 prev_version 是否 NOT NULL?
--     - 当前草案: prev_version TEXT nullable, 应用层校验 (rollback 必填)
--     - 候选: 加 CHECK ((status = 'rollback' AND prev_version IS NOT NULL) OR (status != 'rollback'))
-- Q2 (admin Lead): module_sha256 是否需要长度 CHECK (64 hex 字符)?
--     - 当前草案: 仅 TEXT, 应用层负责格式校验
--     - 候选: 加 CHECK (length(module_sha256) = 64 AND module_sha256 ~ '^[0-9a-f]+$')
-- Q3 (admin Lead): 是否需要 deprecation_note / rollback_reason 字段?
--     - 当前草案: 不在 §X.4 原始 schema 内
--     - 候选: 加 TEXT 字段记录回滚原因
-- Q4 (admin Lead): 是否需要 "热加载耗时" 字段 (e.g. last_loaded_at / load_count)?
--     - 当前草案: 不在 §X.4 原始 schema 内
--     - 候选: 加 TIMESTAMPTZ + INT 监控字段

-- ============================================================
-- 其他已知缺口 (技术层)
-- ============================================================
-- 1. uploaded_by 是跨 DB 软引用 (admin_db.admin_users.id), 跨 DB 不加 FK
-- 2. 同一 (function_id, version) 仅允许 1 行 (PRIMARY KEY 强制), 重传需新建版本
-- 3. module_sha256 唯一性不强制 (理论上不同 function 哈希可能碰撞, 但概率极低)
-- 4. status='rollback' 持久期无 TTL, 需应用层定期清理 (per RGS-INC-001 v0.3 §X.6 第 5 条)
-- 5. 复合主键 (function_id, version) 不支持 ON CONFLICT 单列 upsert, 需完整 2 列 PK
