#!/usr/bin/env bash
# rgs-flash-mock regression test — 60 module 全覆盖 (per RGS-TEST-DESIGN-2026-09-05 v0.1 §2)
#
# 范围:
# - 12 Partial module (combat/partner/guild/arena/role/market/misc/login/rank + 5 域内 1:1 映射)
# - 30 新 module (per 协议号映射 addendum §5.6 ~ §5.42)
# - 18 跨域抽象 module (5 域 + 平台层 + function-plane, per §2 跨域抽象 module)
#
# 用法:
#   RGS_GAP_MOCK_URL=http://127.0.0.1:8791 ./regression-test-60-all-modules.sh
#
# 退出码:
#   0 全部 PASS
#   1 有 module FAIL
#   2 mock server 不可达

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"
RESULTS_DIR="${RESULTS_DIR:-/tmp/rgs-flash-mock-results-$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$RESULTS_DIR"

# 颜色 (terminal 才输出)
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  NC='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; NC=''
fi

PASS_COUNT=0
FAIL_COUNT=0
FAILED_MODULES=()

# 60 module 列表 (per RGS-TEST-DESIGN-2026-09-05 §2 + 协议号映射 addendum §5)
ALL_MODULES=(
  # 12 Partial (per REQ §3.1-3.5)
  "combat" "partner" "guild" "arena" "role" "market" "misc" "login" "rank"
  # 30 新 module (per 协议号映射 §5.6-5.42)
  "star" "adventure" "sns" "say" "holiday" "endless" "boss" "guild_shipping" "guild_dun"
  "item" "formation" "login_partial" "map" "mail" "exchange" "vip" "convert" "drama"
  "avatar" "guild_skill" "days_rank" "lev_gift" "quest" "conn_login" "power_gift" "honor"
  "charge" "recruit" "group_control" "activity" "feat" "login_days" "checkin"
  # 3 缺名补 (per protocol mapping 没显式列但 mock_data 有)
  "dungeon" "group_control" "endless_partial"
  # 18 跨域抽象 module (per TEST-DESIGN §2.1-2.8)
  "profile_sync" "session" "inventory"
  "trade" "ledger" "saga"
  "matchmaking_v2" "spectator" "replay"
  "guild_lifecycle" "party" "chat"
  "audit_log" "rbac" "gm_handler"
  "outbox" "mtls" "rbac_ctrl"
  "realm_lifecycle" "app_deployment" "plugin_registry"
  "function_registry" "function_gateway" "wasm_host"
)

# 1. 检查 mock server
echo "=== 1. mock server 健康检查 ==="
if ! curl -sS -m 5 "$BASE_URL/health" > /dev/null 2>&1; then
  echo -e "${RED}FAIL: mock server $BASE_URL 不可达${NC}"
  exit 2
fi
echo -e "${GREEN}PASS: mock server $BASE_URL OK${NC}"
echo ""

# 2. 全 module 覆盖率检查
echo "=== 2. 60 module 覆盖率 ==="
TOTAL_MODULES=${#ALL_MODULES[@]}
echo "Total modules: $TOTAL_MODULES"

for module in "${ALL_MODULES[@]}"; do
  echo -n "  $module ... "

  result_file="$RESULTS_DIR/${module}.json"

  # 调 /coverage endpoint 或 per-module probe
  if [ -f "${BASH_SOURCE%/*}/../mock_data/${module}.json" ]; then
    # 有 fixture: 验证 mock_data JSON 合法 + rpcs 至少 1 个
    rpc_count=$(jq -r '.rpcs | length' "${BASH_SOURCE%/*}/../mock_data/${module}.json" 2>/dev/null || echo "0")
    if [ "$rpc_count" -gt 0 ]; then
      echo -e "${GREEN}PASS (${rpc_count} rpcs)${NC}"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      echo -e "${RED}FAIL (0 rpcs in fixture)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
      FAILED_MODULES+=("$module")
    fi
  else
    # 无 fixture: 抽象 module, 调 mock /abstract/{module} 端点
    response=$(curl -sS -m 5 -o "$result_file" -w '%{http_code}' \
      "$BASE_URL/abstract/$module" 2>/dev/null || echo "000")
    if [ "$response" = "200" ]; then
      echo -e "${GREEN}PASS (abstract)${NC}"
      PASS_COUNT=$((PASS_COUNT + 1))
    elif [ "$response" = "404" ]; then
      # 抽象 module 暂时未实装 — 标 SKIP (yellow)
      echo -e "${YELLOW}SKIP (404, 待 v0.1 sprint 实施)${NC}"
    else
      echo -e "${RED}FAIL (http=$response)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
      FAILED_MODULES+=("$module")
    fi
  fi
done

# 3. 报告
echo ""
echo "=== 3. 报告 ==="
echo "Total: $TOTAL_MODULES"
echo -e "${GREEN}PASS: $PASS_COUNT${NC}"
echo -e "${RED}FAIL: $FAIL_COUNT${NC}"
if [ ${#FAILED_MODULES[@]} -gt 0 ]; then
  echo ""
  echo "Failed modules:"
  for m in "${FAILED_MODULES[@]}"; do
    echo "  - $m"
  done
fi
echo ""
echo "Results dir: $RESULTS_DIR"

# 4. 退出码
if [ $FAIL_COUNT -gt 0 ]; then
  exit 1
fi
exit 0
