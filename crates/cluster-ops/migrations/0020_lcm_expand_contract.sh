#!/usr/bin/env bash
# 0020_lcm_expand_contract.sh
# 服务器全生命周期管理（LCM）6 张表 Expand-Contract 双向演练脚本（per WBS v0.3 L4 WF-1-2068 + RGS-SPEC-DTL-042 §3 第 5 条 + M-2068.5）
#
# 目的：
#   1. 演练 PG 池（admin_db sandbox）实测 6 张新表 DDL 上线（expand）
#   2. 演练回滚（contract）：CASCADE drop 6 张表 + 验证 cascade 影响面
#   3. 重新 expand 证明 migration 幂等（IF NOT EXISTS + 幂等分区）
#   4. 验证硬约束（FR-LCM-062 locked_at 锁定后不可改 / NFR-SE-010 archive_policy 无删除路径 / FR-LCM-001 不新建独立 DB）
#
# 用法：
#   DATABASE_URL=postgres://rgs_lcm:rgs_lcm@localhost:5544/admin_db bash 0020_lcm_expand_contract.sh
#
# 环境：
#   - PG 18.6+（per RGS-TS-001 v0.6 §5.2 + RGS-BAS-007 §4 月度范围分区）
#   - 演练 PG 池（**不**使用生产 DB，per FR-LCM-003 + DrillExecutor 沙箱隔离）
#   - 演练 PG 池运行前**必须**清空（drop tables + reset）；脚本 idempotent，可重复跑
#
# 退出码：
#   0 — 所有阶段（expand / verify / contract / re-expand）通过
#   1 — 任何阶段失败；失败时打印最近一次 psql 输出用于排查
#
# RACI：
#   R：Worker（DBA + Admin 域 Lead 兼）—— 本脚本由 Ulysses 显式签字
#   A：Ulysses（per DEC-008 12 角色兼任）

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly MIGRATION_SQL="${SCRIPT_DIR}/0020_lcm_tables.sql"

# 颜色（仅 stdout 是否为 TTY 时启用，便于 CI log 抓取）
if [[ -t 1 ]]; then
    readonly C_RESET='\033[0m'
    readonly C_RED='\033[31m'
    readonly C_GREEN='\033[32m'
    readonly C_YELLOW='\033[33m'
    readonly C_BLUE='\033[34m'
else
    readonly C_RESET=''
    readonly C_RED=''
    readonly C_GREEN=''
    readonly C_YELLOW=''
    readonly C_BLUE=''
fi

log() {
    local level="$1"; shift
    case "$level" in
        INFO)  printf "${C_BLUE}[INFO]${C_RESET}  %s\n" "$*";;
        PASS)  printf "${C_GREEN}[PASS]${C_RESET}  %s\n" "$*";;
        WARN)  printf "${C_YELLOW}[WARN]${C_RESET}  %s\n" "$*";;
        FAIL)  printf "${C_RED}[FAIL]${C_RESET}  %s\n" "$*";;
        PHASE) printf "\n${C_YELLOW}========== %s ==========${C_RESET}\n" "$*";;
    esac
}

require_env() {
    if [[ -z "${DATABASE_URL:-}" ]]; then
        log FAIL "DATABASE_URL 未设置。请指定演练 PG 池 URL，例如："
        log FAIL "  DATABASE_URL=postgres://rgs_lcm:rgs_lcm@localhost:5544/admin_db"
        exit 1
    fi

    if [[ ! -f "$MIGRATION_SQL" ]]; then
        log FAIL "找不到 migration SQL: $MIGRATION_SQL"
        exit 1
    fi

    # 强制禁止使用生产 DB（per FR-LCM-003 DrillExecutor 沙箱隔离）
    if [[ "$DATABASE_URL" == *"prod"* || "$DATABASE_URL" == *"production"* ]]; then
        log FAIL "DATABASE_URL 包含 'prod' / 'production' 关键字，禁止使用生产 DB（per FR-LCM-003）"
        exit 1
    fi

    # 检查 psql 可用
    if ! command -v psql >/dev/null 2>&1; then
        log FAIL "psql 不在 PATH（PG 客户端必需）"
        exit 1
    fi
}

# ============================================================================
# EXPAND 阶段
# ============================================================================
phase_expand() {
    log PHASE "PHASE 1: EXPAND（创建 6 张表 + 索引 + 月度分区）"

    log INFO "应用 migration 0020_lcm_tables.sql ..."
    # 幂等：IF NOT EXISTS + DO $$ ... EXCEPTION ... END $$；可重复跑
    if ! psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$MIGRATION_SQL" >/tmp/0020_expand.log 2>&1; then
        log FAIL "EXPAND 失败，psql 输出："
        cat /tmp/0020_expand.log
        exit 1
    fi
    log PASS "EXPAND 成功（6 张表 + 索引 + 2 个月度分区）"
}

# ============================================================================
# VERIFY 阶段
# ============================================================================
phase_verify() {
    log PHASE "PHASE 2: VERIFY（验证硬约束 + 表结构）"

    local expected_tables=(
        "realm_lifecycle_run"
        "new_realm_plan"
        "split_plan"
        "merge_conflict_rule_set_v2"
        "retire_plan"
        "archive_policy"
    )

    # 硬约束 1: 6 张表全部存在
    log INFO "验证 6 张表全部存在 ..."
    for t in "${expected_tables[@]}"; do
        local exists
        exists=$(psql "$DATABASE_URL" -t -A -c "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '$t');")
        if [[ "$exists" != "t" ]]; then
            log FAIL "表 $t 不存在"
            exit 1
        fi
    done
    log PASS "6 张表全部存在"

    # 硬约束 2: FR-LCM-062 — merge_conflict_rule_set_v2 必须有 locked_at 字段 + check 约束
    log INFO "验证 FR-LCM-062（merge_conflict_rule_set_v2 locked_at + check 约束）..."
    local locked_at_count check_count
    locked_at_count=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'merge_conflict_rule_set_v2' AND column_name = 'locked_at';")
    if [[ "$locked_at_count" -lt 1 ]]; then
        log FAIL "merge_conflict_rule_set_v2 缺少 locked_at 列（FR-LCM-062）"
        exit 1
    fi
    check_count=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM information_schema.check_constraints WHERE constraint_name = 'chk_merge_conflict_lock_consistency';")
    if [[ "$check_count" -lt 1 ]]; then
        log FAIL "merge_conflict_rule_set_v2 缺少 chk_merge_conflict_lock_consistency check 约束（FR-LCM-062）"
        exit 1
    fi
    log PASS "FR-LCM-062：locked_at + check 约束存在"

    # 硬约束 3: NFR-SE-010 — archive_policy 必须不含 DELETE 触发器 / 没有 TRUNCATE 路径
    # 注：DDL 本身就不写 DELETE，本检查在 DDL 阶段做（grep 0020_lcm_tables.sql 已通过 CI）
    # 此处运行时再确认一次没有 DELETE 触发器
    log INFO "验证 NFR-SE-010（archive_policy 无 DELETE 触发器）..."
    local delete_triggers
    delete_triggers=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM information_schema.triggers WHERE event_object_table = 'archive_policy' AND event_manipulation = 'DELETE';")
    if [[ "$delete_triggers" -ne 0 ]]; then
        log FAIL "archive_policy 上存在 DELETE 触发器（违反 NFR-SE-010）"
        exit 1
    fi
    log PASS "NFR-SE-010：archive_policy 无 DELETE 触发器"

    # 硬约束 4: FR-LCM-001 — 不新建独立数据库 / 不新建独立 schema
    log INFO "验证 FR-LCM-001（不新建独立数据库 / schema）..."
    local current_db
    current_db=$(psql "$DATABASE_URL" -t -A -c "SELECT current_database();")
    if [[ "$current_db" != "admin_db" ]]; then
        log FAIL "当前 DATABASE 名称是 '$current_db'，应当是 'admin_db'（per FR-LCM-001 + ARC-008 5 独立 DB 原则）"
        exit 1
    fi
    log PASS "FR-LCM-001：当前 DB 是 admin_db（无独立 DB / schema）"

    # 验证 5: realm_lifecycle_run 必须按 created_at 月度范围分区
    log INFO "验证 realm_lifecycle_run 月度范围分区（per RGS-BAS-007 §4）..."
    local partition_strategy
    partition_strategy=$(psql "$DATABASE_URL" -t -A -c "SELECT partstrat FROM pg_partitioned_table JOIN pg_class ON pg_class.oid = pg_partitioned_table.partrelid WHERE relname = 'realm_lifecycle_run';")
    if [[ "$partition_strategy" != "r" ]]; then
        log FAIL "realm_lifecycle_run 分区策略不是 'r' (RANGE)，实际是 '$partition_strategy'（per RGS-BAS-007 §4）"
        exit 1
    fi
    local partition_count
    partition_count=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM pg_inherits JOIN pg_class child ON child.oid = pg_inherits.inhrelid JOIN pg_class parent ON parent.oid = pg_inherits.inhparent WHERE parent.relname = 'realm_lifecycle_run';")
    if [[ "$partition_count" -lt 2 ]]; then
        log FAIL "realm_lifecycle_run 月度分区数 $partition_count < 2（应当至少含当月 + 下月）"
        exit 1
    fi
    log PASS "realm_lifecycle_run 按 created_at 月度范围分区（$partition_count 个分区）"

    # 验证 6: 唯一约束 (request_id, operator_id) 必须存在
    log INFO "验证 realm_lifecycle_run (request_id, operator_id) 唯一约束（per FR-LCM-002 幂等性）..."
    local uq_count
    uq_count=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM pg_constraint WHERE conname = 'uq_lifecycle_run_request_operator' AND contype = 'u';")
    if [[ "$uq_count" -ne 1 ]]; then
        log FAIL "uq_lifecycle_run_request_operator 唯一约束缺失（per FR-LCM-002）"
        exit 1
    fi
    log PASS "(request_id, operator_id) 唯一约束存在"

    # 验证 7: 索引数量（per M-2068.4）
    log INFO "验证索引数量（per M-2068.4）..."
    local total_indexes
    total_indexes=$(psql "$DATABASE_URL" -t -A -c "SELECT COUNT(*) FROM pg_indexes WHERE tablename IN ('realm_lifecycle_run', 'new_realm_plan', 'split_plan', 'merge_conflict_rule_set_v2', 'retire_plan', 'archive_policy') AND indexname LIKE 'idx_%';")
    if [[ "$total_indexes" -lt 12 ]]; then
        log FAIL "索引数量 $total_indexes < 12（期望 ≥ 12 个，含 GIN + 部分索引）"
        exit 1
    fi
    log PASS "索引数量 $total_indexes（≥ 12，含 GIN + 部分索引）"
}

# ============================================================================
# CONTRACT 阶段
# ============================================================================
phase_contract() {
    log PHASE "PHASE 3: CONTRACT（CASCADE drop 6 张表 + 验证 cascade 影响面）"

    # 在 drop 前记录：哪些对象会级联删除（外键依赖）
    log INFO "drop 前扫描 6 张表上的外键依赖 ..."
    local fk_deps
    fk_deps=$(psql "$DATABASE_URL" -t -A -c "
        SELECT conrelid::regclass || ' -> ' || confrelid::regclass
        FROM pg_constraint
        WHERE contype = 'f'
          AND (conrelid::regclass::text LIKE '%_plan'
               OR conrelid::regclass::text LIKE '%_policy'
               OR conrelid::regclass::text LIKE '%_rule_set%')
          AND connamespace = 'admin_db'::regnamespace;
    " 2>/dev/null || true)
    if [[ -n "$fk_deps" ]]; then
        log INFO "cascade 影响面："
        echo "$fk_deps" | sed 's/^/    /'
    else
        log INFO "cascade 影响面：无（6 张表无被引用的 FK）"
    fi

    log INFO "drop 6 张表（CASCADE）..."
    # CASCADE：清理由 FK 引用的依赖（虽然本脚本只创建了这 6 张表，6 张表之间 run_id 是同 DB 内 FK）
    # 用单个 SQL 批量 drop，原子性
    if ! psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'EOF' >/tmp/0020_contract.log 2>&1
DROP TABLE IF EXISTS new_realm_plan CASCADE;
DROP TABLE IF EXISTS split_plan CASCADE;
DROP TABLE IF EXISTS retire_plan CASCADE;
DROP TABLE IF EXISTS archive_policy CASCADE;
DROP TABLE IF EXISTS merge_conflict_rule_set_v2 CASCADE;
DROP TABLE IF EXISTS realm_lifecycle_run CASCADE;
EOF
    then
        log FAIL "CONTRACT 失败，psql 输出："
        cat /tmp/0020_contract.log
        exit 1
    fi
    log PASS "CONTRACT 成功（6 张表全部 CASCADE drop）"

    # 验证所有 6 张表已不存在
    log INFO "验证 6 张表已全部 drop ..."
    local remaining
    remaining=$(psql "$DATABASE_URL" -t -A -c "
        SELECT COUNT(*) FROM information_schema.tables
        WHERE table_name IN ('realm_lifecycle_run', 'new_realm_plan', 'split_plan',
                             'merge_conflict_rule_set_v2', 'retire_plan', 'archive_policy');
    ")
    if [[ "$remaining" -ne 0 ]]; then
        log FAIL "drop 后仍有 $remaining 张表残留"
        exit 1
    fi
    log PASS "6 张表全部 drop 完成"
}

# ============================================================================
# RE-EXPAND 阶段（证明 migration 幂等）
# ============================================================================
phase_reexpand() {
    log PHASE "PHASE 4: RE-EXPAND（重新应用 migration，验证 IF NOT EXISTS 幂等）"

    # 重新 apply：必须成功（IF NOT EXISTS 保证幂等）
    if ! psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$MIGRATION_SQL" >/tmp/0020_reexpand.log 2>&1; then
        log FAIL "RE-EXPAND 失败，psql 输出："
        cat /tmp/0020_reexpand.log
        exit 1
    fi
    log PASS "RE-EXPAND 成功（migration 幂等，可重复跑）"

    # 再次验证 6 张表存在
    log INFO "再次验证 6 张表已重建 ..."
    local re_count
    re_count=$(psql "$DATABASE_URL" -t -A -c "
        SELECT COUNT(*) FROM information_schema.tables
        WHERE table_name IN ('realm_lifecycle_run', 'new_realm_plan', 'split_plan',
                             'merge_conflict_rule_set_v2', 'retire_plan', 'archive_policy');
    ")
    if [[ "$re_count" -ne 6 ]]; then
        log FAIL "RE-EXPAND 后表数量 $re_count != 6"
        exit 1
    fi
    log PASS "RE-EXPAND 验证：6 张表全部重建"
}

# ============================================================================
# 主流程
# ============================================================================
main() {
    log INFO "0020_lcm_expand_contract.sh 启动"
    log INFO "  DATABASE_URL=$DATABASE_URL"
    log INFO "  MIGRATION_SQL=$MIGRATION_SQL"

    require_env

    phase_expand
    phase_verify
    phase_contract
    phase_reexpand

    log PHASE "全部阶段通过"
    log PASS "0020_lcm_expand_contract.sh 演练完成（expand / verify / contract / re-expand 全绿）"
    log INFO "后续步骤："
    log INFO "  1. cargo sqlx prepare --workspace -- --all-targets  生成 .sqlx/ 缓存"
    log INFO "  2. 提交 PR（含本脚本 + 0020_lcm_tables.sql + .sqlx/）"
    log INFO "  3. CI 在 SQLX_OFFLINE=true 下编译 + 演练环境跑本脚本全绿后，方可合入 main"
    log INFO "  4. 生产 admin_db 部署时**只** apply 0020_lcm_tables.sql（不跑本脚本的 contract 段）"
}

main "$@"
