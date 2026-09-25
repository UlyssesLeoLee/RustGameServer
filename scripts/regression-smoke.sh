#!/usr/bin/env bash
# regression-smoke.sh — mock 项目 UT/IT/ST 全流程回归入口
#
# 用途(per Ulysses 2026-09-20 14:00 JST 'mock 项目 UT IT ST 完善脚本并整体回归测试' 指令,
#      落点 ULYS-141):跨 mock 资产 5 域 + cluster-ops + gm-backend + 工具集
#      + ST k3s e2e, 一次性回归, 出 PASS/FAIL/SKIP 汇总 + manifest.json + summary.json。
#
# 设计:
#   L1 (UT)  — 7 域 example + rgs-testkit self_test + rgs-certgen 黑盒 + gm-backend 黑盒
#   L2 (IT)  — 5 域 + cluster-ops + gm-backend integration_* 测试
#   L3 (ST)  — k3s e2e-smoke 12 端口 + ST scripts(st-01..16) 抽样
#   汇总    — PASS / FAIL / SKIP + per-step elapsed + manifest.json
#
# 用法:
#   bash scripts/regression-smoke.sh [--batch-id ID] [--evidence-dir DIR]
#                                    [--skip-examples] [--skip-it] [--skip-st]
#                                    [--skip-e2e-smoke] [--st-only]
#                                    [--verbose] [--json] [--help]
#
# 落点:docs/00-基准与治理/.test-evidence/regression/{batch_id}/
#       - manifest.json     (git head / rust / host / artifacts)
#       - summary.json      (per-step pass/fail/elapsed_ms + grand total)
#       - regression.log    (tee of all stdout)
#       - step-*.log        (per-step cargo/e2e 完整输出)
#       - regression.md     (人类可读总结)
#
# 关联:docs/00-基准与治理/mock-registry.md §5
#       docs/00-基准与治理/.test-evidence/README.md
#       scripts/test-evidence.ps1 (PowerShell 版,WSL e2e 入口)
#
# 强约束(per AGENTS.md §1.2 环境变量安全):
#   - 禁止打印 env 值,只可 invoke。
#   - 跨工具链决策前先查 workspace 依赖 + 文档段(per AGENTS.md §2.2 L3)。
#
# 退出码:
#   0 = 全部 PASS 或 仅 SKIP
#   1 = 任何 FAIL
#   2 = 调用错误(参数/依赖缺失)

set -euo pipefail

# ---------- argparse ----------
BATCH_ID=""
EVIDENCE_DIR=""
SKIP_EXAMPLES=0
SKIP_IT=0
SKIP_ST=0
SKIP_E2E_SMOKE=0
ST_ONLY=0
VERBOSE=0
JSON_OUT=0

usage() {
    sed -n '4,40p' "$0"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --batch-id)        BATCH_ID="$2"; shift 2 ;;
        --evidence-dir)    EVIDENCE_DIR="$2"; shift 2 ;;
        --skip-examples)   SKIP_EXAMPLES=1; shift ;;
        --skip-it)         SKIP_IT=1; shift ;;
        --skip-st)         SKIP_ST=1; shift ;;
        --skip-e2e-smoke)  SKIP_E2E_SMOKE=1; shift ;;
        --st-only)         ST_ONLY=1; shift ;;
        --verbose|-v)      VERBOSE=1; shift ;;
        --json)            JSON_OUT=1; shift ;;
        --help|-h)         usage ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done

if [[ "$ST_ONLY" == "1" ]]; then
    SKIP_EXAMPLES=1
    SKIP_IT=1
fi

# ---------- paths & env ----------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

if [[ -z "$BATCH_ID" ]]; then
    BATCH_ID="regression-$(date -u +%Y%m%dT%H%M%SZ)"
fi
if [[ -z "$EVIDENCE_DIR" ]]; then
    EVIDENCE_DIR="$REPO_ROOT/docs/00-基准与治理/.test-evidence/regression/$BATCH_ID"
fi
mkdir -p "$EVIDENCE_DIR"
REGRESSION_LOG="$EVIDENCE_DIR/regression.log"

# tee 全程 stdout/stderr 到 regression.log(同时仍 echo 到控制台)
# 实现:tee 在子进程内同时写 fd 1 (终端) 和 log 文件,
#      通过 process substitution 把脚本 stdout 转发给 tee。
exec > >(tee -a "$REGRESSION_LOG") 2>&1

# ---------- colors ----------
if [[ -t 1 ]] && [[ "$JSON_OUT" != "1" ]]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; CYAN=''; NC=''
fi

# ---------- counters & state ----------
PASS=0
FAIL=0
SKIP=0
declare -a STEP_NAMES=()
declare -a STEP_LAYERS=()
declare -a STEP_RESULTS=()   # PASS | FAIL | SKIP
declare -a STEP_ELAPSED_MS=()
declare -a STEP_DETAILS=()   # cargo test pass/fail 数 或 detail 文字
declare -a STEP_LOGS=()      # 相对 evidence_dir 的 log 路径

step_started_ms() { date +%s%3N; }

step_header() {
    local layer="$1" name="$2"
    echo ""
    echo -e "${CYAN}==== [${layer}] ${name} ====${NC}"
}

# cargo test output → 抽 (passed, failed, total); 失败输出原样
# 兼容格式:
#   "test result: ok. 12 passed; 0 failed; ..."
#   "test result: FAILED. 10 passed; 2 failed; ..."
parse_cargo_test() {
    local log="$1"
    awk '
        /test result: ok\./ {
            for (i=1; i<=NF; i++) {
                if ($i == "passed;") { passed += $(i-1) }
                if ($i == "failed;") { failed += $(i-1) }
            }
        }
        /test result: FAILED\./ {
            for (i=1; i<=NF; i++) {
                if ($i == "passed;") { passed += $(i-1) }
                if ($i == "failed;") { failed += $(i-1) }
            }
        }
        END {
            if (passed+failed == 0) print "no-result"
            else printf "%d passed, %d failed", passed, failed
        }
    ' "$log"
}

# 跑一个 step, 入参 layer / name / cmd-array / extra-summary
# 返回 0=PASS, 1=FAIL, 2=SKIP
run_step() {
    local layer="$1" name="$2"
    shift 2

    local log_rel="step-${name}.log"
    local log_abs="$EVIDENCE_DIR/$log_rel"

    step_header "$layer" "$name"

    if [[ "$1" == "__SKIP__" ]]; then
        local reason="$2"
        echo -e "${YELLOW}  [SKIP]${NC} $name ($reason)"
        SKIP=$((SKIP + 1))
        STEP_NAMES+=("$name"); STEP_LAYERS+=("$layer")
        STEP_RESULTS+=("SKIP"); STEP_ELAPSED_MS+=(0); STEP_DETAILS+=("$reason")
        STEP_LOGS+=("")
        return 2
    fi

    local start_ms
    start_ms=$(step_started_ms)

    set +e
    "$@" > "$log_abs" 2>&1
    local rc=$?
    set -e

    local elapsed_ms=$(( $(step_started_ms) - start_ms ))
    local summary
    summary=$(parse_cargo_test "$log_abs")
    if [[ "$summary" == "no-result" ]]; then
        # 不是 cargo test(可能是 e2e-smoke 或 ps1 包装) → 直接看 rc
        if [[ $rc -eq 0 ]]; then
            summary="exit_ok"
        else
            summary="exit_fail"
        fi
    fi

    if [[ $rc -eq 0 ]]; then
        echo -e "${GREEN}  [PASS]${NC} $name (${summary}, ${elapsed_ms}ms)"
        PASS=$((PASS + 1))
        STEP_NAMES+=("$name"); STEP_LAYERS+=("$layer")
        STEP_RESULTS+=("PASS"); STEP_ELAPSED_MS+=("$elapsed_ms"); STEP_DETAILS+=("$summary")
        STEP_LOGS+=("$log_rel")
        return 0
    else
        echo -e "${RED}  [FAIL]${NC} $name (rc=$rc, ${summary}, ${elapsed_ms}ms)"
        FAIL=$((FAIL + 1))
        STEP_NAMES+=("$name"); STEP_LAYERS+=("$layer")
        STEP_RESULTS+=("FAIL"); STEP_ELAPSED_MS+=("$elapsed_ms"); STEP_DETAILS+=("$summary")
        STEP_LOGS+=("$log_rel")
        if [[ "$VERBOSE" == "1" ]]; then
            tail -15 "$log_abs" | sed 's/^/    /'
        else
            tail -5 "$log_abs" | sed 's/^/    /'
        fi
        return 1
    fi
}

# ---------- banner ----------
echo "=== regression-smoke.sh ==="
echo "  batch_id    = $BATCH_ID"
echo "  evidence    = $EVIDENCE_DIR"
echo "  date        = $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "  host        = $(hostname 2>/dev/null || echo unknown)"
echo "  rustc       = $(rustc --version 2>/dev/null || echo missing)"
echo "  cargo       = $(cargo --version 2>/dev/null || echo missing)"
echo "  repo        = $REPO_ROOT"
echo "  flags       = skip-examples=$SKIP_EXAMPLES skip-it=$SKIP_IT skip-st=$SKIP_ST skip-e2e-smoke=$SKIP_E2E_SMOKE st-only=$ST_ONLY"

# ---------- git context ----------
# 注:git 是 native Windows 程序, MSYS path translation 在 -C 参数上不生效,
#    必须用 forward-slash native 路径 (C:/Users/...) 而非 (/c/Users/...)。
#    同时所有 git 调用都包 || echo unknown 防止 set -e 误杀。
#    转换 MSYS 路径 → native Windows 路径(/c/X → C:/X,/d/X → D:/X 等)
to_native_path() {
    local p="$1"
    case "$p" in
        /[a-zA-Z]/*) echo "$(echo "$p" | sed 's|^\/\([a-zA-Z]\)|\1:|')" ;;
        *)           echo "$p" ;;
    esac
}
GIT_NATIVE_PATH=$(to_native_path "$REPO_ROOT")
GIT_HEAD=$(git -C "$GIT_NATIVE_PATH" rev-parse HEAD 2>/dev/null || echo unknown)
GIT_BRANCH=$(git -C "$GIT_NATIVE_PATH" rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)
# status 的退出码非 0 不算错(可能 dirty/branch detached),用 || echo 0 转 0 再统计
GIT_DIRTY=$(git -C "$GIT_NATIVE_PATH" status --porcelain 2>/dev/null | wc -l | tr -d ' ' || echo 0)
echo "  git head    = $GIT_HEAD"
echo "  git branch  = $GIT_BRANCH"
echo "  git dirty   = $GIT_DIRTY (lines)"

# 检查 k3s cluster 是否可达(供 ST 步骤决策)
# 注:每个子命令都包 set +e/-e 防止 WSL startup noise / sudo 提示等触发 set -e
K3S_REACHABLE=0
set +e
if command -v wsl >/dev/null 2>&1; then
    wsl -e bash -c 'echo wsl-ok' >/dev/null 2>&1
    if [ $? -eq 0 ]; then
        wsl -e bash -c 'export KUBECONFIG=/etc/rancher/k3s/k3s.yaml; sudo -n /usr/local/bin/k3s kubectl get nodes --no-headers' >/dev/null 2>&1
        if [ $? -eq 0 ]; then
            K3S_REACHABLE=1
        fi
    fi
fi
set -e
if [ "$K3S_REACHABLE" = "1" ]; then
    echo "  k3s        = reachable"
else
    echo "  k3s        = NOT-reachable"
fi

# ============================================================================
# L1 — UT (unit / black-box / example)
# ============================================================================
if [[ $SKIP_EXAMPLES == 0 ]]; then
    echo ""
    echo -e "${CYAN}############ L1 — UT (单元 / 黑盒 / example) ############${NC}"

    # 1.1 7 域 example(mock 资产演示)
    for ex in domain_player_demo domain_economy_demo domain_match_demo \
              domain_social_demo domain_admin_demo domain_cluster_ops_demo \
              domain_gm_backend_demo; do
        run_step "UT-EX" "example-${ex}" \
            bash -c "cargo run --example $ex -p rgs-testkit --quiet" \
            || true
    done

    # 1.2 rgs-testkit self_test(mock 入口 API)
    run_step "UT-EX" "rgs-testkit-self_test" \
        bash -c "cargo test -p rgs-testkit --quiet --test self_test" \
        || true

    # 1.3 rgs-certgen 黑盒(UT-09 工具集)
    run_step "UT-09" "rgs-certgen-ut_blackbox" \
        bash -c "cargo test -p rgs-certgen --quiet --test ut_blackbox" \
        || true

    # 1.4 gm-backend 黑盒(UT-08)
    run_step "UT-08" "gm-backend-ut" \
        bash -c "cargo test -p gm-backend --quiet --tests" \
        || true
fi

# ============================================================================
# L2 — IT (integration)
# ============================================================================
if [[ $SKIP_IT == 0 ]]; then
    echo ""
    echo -e "${CYAN}############ L2 — IT (集成测试) ############${NC}"

    # 5 域 + cluster-ops 集成测试(IT-01..IT-06 per RGS-TST-IT-00..06)
    for crate in player-service economy-service match-service social-service \
                 admin-service cluster-ops; do
        run_step "IT" "integration-${crate}" \
            bash -c "cargo test -p $crate --quiet --tests 2>&1 | tail -50" \
            || true
    done
fi

# ============================================================================
# L3 — ST (system / e2e)
# ============================================================================
if [[ $SKIP_ST == 0 ]]; then
    echo ""
    echo -e "${CYAN}############ L3 — ST (系统 / k3s e2e) ############${NC}"

    # 3.1 e2e-smoke(12 端口 + gm-backend /healthz) — 需 k3s 集群跑着
    if [[ $SKIP_E2E_SMOKE == 1 ]]; then
        run_step "ST-E2E" "e2e-smoke" "__SKIP__" "用户指定 --skip-e2e-smoke"
    elif [[ $K3S_REACHABLE == 0 ]]; then
        run_step "ST-E2E" "e2e-smoke" "__SKIP__" "k3s 不可达 (wsl/kubectl/sudo 均不可用)"
    elif [[ ! -f "$SCRIPT_DIR/e2e-smoke.sh" ]]; then
        run_step "ST-E2E" "e2e-smoke" "__SKIP__" "e2e-smoke.sh 不存在"
    else
        # 在 wsl 侧跑 wsl-side driver,通过 wsl 调用
        run_step "ST-E2E" "e2e-smoke" \
            bash -c "cd /mnt/d/RustGameServer && bash scripts/e2e-smoke.sh 2>&1 | tail -50" \
            || true
    fi

    # 3.2 ST 脚本抽样 — st-01..st-10 是 5 域 cross-domain,无 mTLS 强依赖,作为快测入口
    # 不在此跑 st-11..st-16(需要 mTLS 证书 + grpcurl,慢且依赖前置);它们由 scripts/st/ 单独触发
    for st in st-01-player-grpc-port-and-gm-backend \
              st-03-economy-grpc-port-and-outbox \
              st-05-match-grpc-port-and-replay \
              st-07-social-grpc-port-and-guild \
              st-09-admin-grpc-port-and-audit; do
        if [[ -f "$SCRIPT_DIR/st/${st}.ps1" ]]; then
            # st-NN 脚本要 WSL + pwsh + k3s 可达;否则 skip
            if [[ $K3S_REACHABLE == 0 ]]; then
                run_step "ST-SCRIPT" "${st}" "__SKIP__" "k3s 不可达"
            else
                run_step "ST-SCRIPT" "${st}" \
                    bash -c "cd /mnt/d/RustGameServer && pwsh -NoProfile -NonInteractive -File scripts/st/${st}.ps1 -EvidenceDir /mnt/d/RustGameServer/docs/00-基准与治理/.test-evidence/regression/$BATCH_ID/st 2>&1 | tail -30" \
                    || true
            fi
        else
            run_step "ST-SCRIPT" "${st}" "__SKIP__" "scripts/st/${st}.ps1 不存在"
        fi
    done
fi

# ============================================================================
# 汇总
# ============================================================================
echo ""
echo -e "${CYAN}=== 汇总 ===${NC}"
echo -e "  ${GREEN}PASS${NC} = $PASS"
echo -e "  ${RED}FAIL${NC} = $FAIL"
echo -e "  ${YELLOW}SKIP${NC} = $SKIP"
echo ""
echo "  evidence    = $EVIDENCE_DIR"
echo "  log         = $REGRESSION_LOG"

# ---------- 写 manifest.json ----------
MANIFEST="$EVIDENCE_DIR/manifest.json"
{
    echo "{"
    echo "  \"batch_id\": \"$BATCH_ID\","
    echo "  \"git_head\": \"$GIT_HEAD\","
    echo "  \"git_branch\": \"$GIT_BRANCH\","
    echo "  \"git_dirty_lines\": $GIT_DIRTY,"
    echo "  \"host\": \"$(hostname 2>/dev/null || echo unknown)\","
    echo "  \"rustc\": \"$(rustc --version 2>/dev/null || echo missing)\","
    echo "  \"cargo\": \"$(cargo --version 2>/dev/null || echo missing)\","
    echo "  \"k3s_reachable\": $K3S_REACHABLE,"
    echo "  \"flags\": {"
    echo "    \"skip_examples\": $SKIP_EXAMPLES,"
    echo "    \"skip_it\": $SKIP_IT,"
    echo "    \"skip_st\": $SKIP_ST,"
    echo "    \"skip_e2e_smoke\": $SKIP_E2E_SMOKE,"
    echo "    \"st_only\": $ST_ONLY"
    echo "  },"
    echo "  \"totals\": { \"pass\": $PASS, \"fail\": $FAIL, \"skip\": $SKIP },"
    echo "  \"steps\": ["
    mfirst=1
    for i in "${!STEP_NAMES[@]}"; do
        if [[ $mfirst -eq 1 ]]; then mfirst=0; else echo ","; fi
        printf '    {"name":"%s","layer":"%s","result":"%s","elapsed_ms":%s,"detail":"%s","log":"%s"}' \
            "${STEP_NAMES[$i]}" "${STEP_LAYERS[$i]}" "${STEP_RESULTS[$i]}" \
            "${STEP_ELAPSED_MS[$i]}" "${STEP_DETAILS[$i]}" "${STEP_LOGS[$i]}"
    done
    echo ""
    echo "  ]"
    echo "}"
} > "$MANIFEST"

# ---------- 写 summary.json ----------
SUMMARY="$EVIDENCE_DIR/summary.json"
{
    echo "{"
    echo "  \"batch_id\": \"$BATCH_ID\","
    echo "  \"totals\": { \"pass\": $PASS, \"fail\": $FAIL, \"skip\": $SKIP },"
    echo "  \"by_layer\": {"
    layers=("UT-EX" "UT-09" "UT-08" "IT" "ST-E2E" "ST-SCRIPT")
    sfirst=1
    for ly in "${layers[@]}"; do
        lp=0; lf=0; ls=0
        for j in "${!STEP_LAYERS[@]}"; do
            if [[ "${STEP_LAYERS[$j]}" == "$ly" ]]; then
                case "${STEP_RESULTS[$j]}" in
                    PASS) lp=$((lp + 1)) ;;
                    FAIL) lf=$((lf + 1)) ;;
                    SKIP) ls=$((ls + 1)) ;;
                esac
            fi
        done
        if [[ $sfirst -eq 1 ]]; then sfirst=0; else echo ","; fi
        printf '    "%s": {"pass":%d,"fail":%d,"skip":%d}' "$ly" "$lp" "$lf" "$ls"
    done
    echo ""
    echo "  },"
    echo "  \"verdict\": \"$([ $FAIL -eq 0 ] && echo PASS || echo FAIL)\""
    echo "}"
} > "$SUMMARY"

# ---------- 写 regression.md(人类可读) ----------
MD="$EVIDENCE_DIR/regression.md"
{
    echo "# Regression Smoke — $BATCH_ID"
    echo ""
    echo "## Context"
    echo ""
    echo "| field | value |"
    echo "|---|---|"
    echo "| batch_id | \`$BATCH_ID\` |"
    echo "| date | $(date -u +%Y-%m-%dT%H:%M:%SZ) |"
    echo "| git head | \`$GIT_HEAD\` |"
    echo "| git branch | \`$GIT_BRANCH\` |"
    echo "| git dirty | $GIT_DIRTY lines |"
    echo "| host | $(hostname 2>/dev/null || echo unknown) |"
    echo "| rustc | $(rustc --version 2>/dev/null || echo missing) |"
    echo "| cargo | $(cargo --version 2>/dev/null || echo missing) |"
    echo "| k3s | $([[ $K3S_REACHABLE == 1 ]] && echo reachable || echo NOT-reachable) |"
    echo ""
    echo "## Totals"
    echo ""
    echo "| metric | count |"
    echo "|---|---|"
    echo "| **PASS** | $PASS |"
    echo "| **FAIL** | $FAIL |"
    echo "| **SKIP** | $SKIP |"
    echo ""
    echo "## Per-Layer"
    echo ""
    echo "| layer | PASS | FAIL | SKIP |"
    echo "|---|---|---|---|"
    for ly in "${layers[@]}"; do
        lp=0; lf=0; ls=0
        for j in "${!STEP_LAYERS[@]}"; do
            if [[ "${STEP_LAYERS[$j]}" == "$ly" ]]; then
                case "${STEP_RESULTS[$j]}" in
                    PASS) lp=$((lp + 1)) ;;
                    FAIL) lf=$((lf + 1)) ;;
                    SKIP) ls=$((ls + 1)) ;;
                esac
            fi
        done
        echo "| $ly | $lp | $lf | $ls |"
    done
    echo ""
    echo "## Per-Step"
    echo ""
    echo "| # | layer | step | result | elapsed | detail | log |"
    echo "|---|---|---|---|---|---|---|"
    n=1
    for i in "${!STEP_NAMES[@]}"; do
        log_link="${STEP_LOGS[$i]:-(none)}"
        echo "| $n | ${STEP_LAYERS[$i]} | \`${STEP_NAMES[$i]}\` | ${STEP_RESULTS[$i]} | ${STEP_ELAPSED_MS[$i]}ms | ${STEP_DETAILS[$i]} | $log_link |"
        n=$((n + 1))
    done
    echo ""
    if [[ $FAIL -eq 0 ]]; then
        echo "## Verdict"
        echo ""
        echo "全 PASS(或仅 SKIP),无 fail。"
    else
        echo "## Verdict"
        echo ""
        echo "**$FAIL** 条 FAIL,查 \`step-*.log\`。"
    fi
} > "$MD"

# ---------- 最终输出 ----------
if [[ $FAIL -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}✅ regression smoke: PASS (PASS=$PASS FAIL=$FAIL SKIP=$SKIP)${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ regression smoke: FAIL ($FAIL 条 FAIL,PASS=$PASS SKIP=$SKIP)${NC}"
    echo "  查 $EVIDENCE_DIR/step-*.log 与 regression.md"
    exit 1
fi
