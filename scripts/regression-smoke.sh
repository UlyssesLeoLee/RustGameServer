#!/usr/bin/env bash
# regression-smoke.sh
# 用途:跨域回归 smoke 入口(per Ulysses 2026-08-28 08:40 JST 'mock 和脚本存在 mock 项目用于回归测试' 指令)
# 设计:
#   1. 跑 7 域 example 确认 mock 资产未坏
#   2. 跑 rgs-testkit self_test 确认 mock 入口未坏
#   3. 跑 rgs-certgen 17 黑盒 (UT-09) 确认工具集未坏
#   4. 跑 gm-backend 19 黑盒 (UT-08) 确认 GM 后台未坏
#   5. 跑 5 域 + cluster-ops 集成测试确认 8 域未坏
#   6. (可选) e2e-smoke 12 端口
# 落点:e2e 输出 stdout,可 tee 到 docs/00-基准与治理/.test-evidence/regression/{date}/regression.log
# 关联:docs/00-基准与治理/mock-registry.md §5

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

function step() {
    echo ""
    echo -e "${YELLOW}==== $1 ====${NC}"
}

function run_step() {
    local name=$1
    local cmd=$2
    if eval "$cmd" > /tmp/regression-$name.log 2>&1; then
        echo -e "${GREEN}  [PASS]${NC} $name"
        PASS=$((PASS+1))
    else
        echo -e "${RED}  [FAIL]${NC} $name (log: /tmp/regression-$name.log)"
        FAIL=$((FAIL+1))
        tail -5 /tmp/regression-$name.log | sed 's/^/    /'
    fi
}

function skip_step() {
    local name=$1
    local reason=$2
    echo -e "${YELLOW}  [SKIP]${NC} $name ($reason)"
    SKIP=$((SKIP+1))
}

echo "=== regression-smoke.sh ==="
echo "  date = $(date +%Y-%m-%dT%H:%M:%S%z)"
echo "  host = $(hostname)"
echo "  rust = $(rustc --version)"
echo "  repo = $REPO_ROOT"

# 1. 7 域 example
step "1. 7 域 example (mock 资产演示)"
for ex in domain_player_demo domain_economy_demo domain_match_demo domain_social_demo domain_admin_demo domain_cluster_ops_demo domain_gm_backend_demo; do
    run_step "example-$ex" "cargo run --example $ex -p rgs-testkit --quiet"
done

# 2. rgs-testkit self_test
step "2. rgs-testkit self_test (mock 入口)"
run_step "rgs-testkit-self_test" "cargo test -p rgs-testkit --quiet --test self_test"

# 3. rgs-certgen 17 黑盒
step "3. rgs-certgen 17 黑盒 (UT-09)"
run_step "rgs-certgen-ut" "cargo test -p rgs-certgen --quiet --test ut_blackbox"

# 4. gm-backend 19 黑盒
step "4. gm-backend 19 黑盒 (UT-08)"
run_step "gm-backend-ut" "cargo test -p gm-backend --quiet"

# 5. 5 域 + cluster-ops 集成测试
step "5. 5 域 + cluster-ops 集成测试"
for crate in player-service economy-service match-service social-service admin-service cluster-ops; do
    run_step "integration-$crate" "cargo test -p $crate --quiet --tests 2>&1 | grep -q 'test result: ok'"
done

# 6. (可选) e2e-smoke — 需要 k3s 集群跑着
step "6. e2e-smoke (可选,需要 k3s 集群跑着)"
if [ -f "$SCRIPT_DIR/e2e-smoke.sh" ]; then
    run_step "e2e-smoke" "bash $SCRIPT_DIR/e2e-smoke.sh 2>&1 | tail -3 | grep -q 'ALL PASS'"
else
    skip_step "e2e-smoke" "e2e-smoke.sh 不存在"
fi

# 汇总
echo ""
echo "=== 汇总 ==="
echo -e "  ${GREEN}PASS${NC} = $PASS"
echo -e "  ${RED}FAIL${NC} = $FAIL"
echo -e "  ${YELLOW}SKIP${NC} = $SKIP"
echo ""
if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ regression smoke 全 PASS${NC}"
    exit 0
else
    echo -e "${RED}❌ regression smoke 有 $FAIL 条 FAIL,查 /tmp/regression-*.log${NC}"
    exit 1
fi
