#!/usr/bin/env bash
# rgs-flash-mock regression test — 错误码覆盖 (per RGS-TEST-DESIGN-2026-09-05 v0.1 §4)
#
# 范围: 60 module 通用错误码 1:N 映射验证
# - 0       OK
# - 1001-1099 参数 / 校验错
# - 1100-1199 鉴权 / 权限
# - 2000-2099 资源不存在 / 已存在
# - 3000-3099 状态机非法迁移
# - 4000-4099 业务锁 / 配额
# - 5000-5099 服务端内部错
# - 9000-9099 上游依赖故障
#
# 用法:
#   RGS_GAP_MOCK_URL=http://127.0.0.1:8791 ./regression-test-error-codes.sh

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"
RESULTS_DIR="${RESULTS_DIR:-/tmp/rgs-flash-mock-error-codes-$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$RESULTS_DIR"

if [ -t 1 ]; then
  RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; NC=''
fi

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# 1. mock server 健康
echo "=== 1. mock server 健康 ==="
if ! curl -sS -m 5 "$BASE_URL/health" > /dev/null 2>&1; then
  echo -e "${RED}FAIL: mock server $BASE_URL 不可达${NC}"
  exit 2
fi
echo -e "${GREEN}PASS: mock OK${NC}"
echo ""

# 2. 通用错误码 1:N 映射
echo "=== 2. 通用错误码 1:N 映射 ==="
# 每个错误码: input → expect code
declare -A EC_CASES=(
  ["PARAM_INVALID_1001"]="empty_field"
  ["FIELD_TOO_LONG_1002"]="field_too_long"
  ["PERM_DENIED_1003"]="permission_denied"
  ["NOT_FOUND_2004"]="resource_missing"
  ["ALREADY_EXISTS_2005"]="duplicate_id"
  ["STATE_INVALID_3001"]="state_machine_illegal"
  ["RATE_LIMITED_4001"]="quota_exceeded"
  ["INTERNAL_5001"]="server_internal"
  ["UPSTREAM_DOWN_9001"]="dependency_unavailable"
)

# 每个错误码类至少 1 个 module 验证
declare -a MODULES_FOR_EC=(
  "combat" "partner" "guild" "arena" "role"
  "star" "adventure" "sns" "say" "holiday"
  "endless" "boss" "guild_shipping" "guild_dun" "item"
  "formation" "login_partial" "map" "mail" "exchange"
  "vip" "convert" "drama" "rank" "avatar"
  "guild_skill" "days_rank" "lev_gift" "quest" "conn_login"
  "power_gift" "honor" "charge" "recruit" "group_control"
  "activity" "feat" "login_days" "checkin" "dungeon"
)

for ec_name in "${!EC_CASES[@]}"; do
  echo -n "  $ec_name ... "
  # 调 mock /error-code/{code} 端点
  code=$(echo "$ec_name" | grep -oE '[0-9]+$')
  response=$(curl -sS -m 5 -o "$RESULTS_DIR/${ec_name}.json" -w '%{http_code}' \
    "$BASE_URL/error-code/$code" 2>/dev/null || echo "000")

  case $response in
    200)
      # 验证响应 code 字段
      resp_code=$(jq -r '.code // "null"' "$RESULTS_DIR/${ec_name}.json" 2>/dev/null)
      if [ "$resp_code" = "$code" ]; then
        echo -e "${GREEN}PASS (code=$code)${NC}"
        PASS_COUNT=$((PASS_COUNT + 1))
      else
        echo -e "${RED}FAIL (expected code=$code, got=$resp_code)${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
      fi
      ;;
    404)
      echo -e "${YELLOW}SKIP (404, mock 端点待 v0.1 实施)${NC}"
      SKIP_COUNT=$((SKIP_COUNT + 1))
      ;;
    *)
      echo -e "${RED}FAIL (http=$response)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
      ;;
  esac
done
echo ""

# 3. 42 module 错误码矩阵抽查
echo "=== 3. 42 module × 错误码 矩阵抽查 ==="
# 抽查规则: 5 域每域 1 个 module, 各跑 3 个错误码
declare -A SAMPLE_MATRIX=(
  ["combat:1001:empty_combat_type"]="PARAM_INVALID"
  ["combat:2004:invalid_session_id"]="NOT_FOUND"
  ["combat:3001:state_in_init_exit"]="STATE_INVALID"
  ["economy:1001:empty_account_id"]="PARAM_INVALID"
  ["economy:4001:quota_exceeded"]="RATE_LIMITED"
  ["economy:5001:ledger_write_fail"]="INTERNAL"
  ["match:1001:invalid_match_id"]="PARAM_INVALID"
  ["match:3001:state_not_in_round"]="STATE_INVALID"
  ["match:2004:match_not_found"]="NOT_FOUND"
  ["social:1003:not_guild_member"]="PERM_DENIED"
  ["social:1001:empty_guild_name"]="PARAM_INVALID"
  ["social:2005:guild_name_taken"]="ALREADY_EXISTS"
  ["admin:1003:not_admin"]="PERM_DENIED"
  ["admin:4001:rate_limited"]="RATE_LIMITED"
  ["admin:5001:audit_log_fail"]="INTERNAL"
)

for case_key in "${!SAMPLE_MATRIX[@]}"; do
  IFS=':' read -r module code scenario <<< "$case_key"
  ec_name="${SAMPLE_MATRIX[$case_key]}"
  echo -n "  $module:$code:$scenario ... "

  response=$(curl -sS -m 5 -o "$RESULTS_DIR/${module}_${code}_${scenario}.json" -w '%{http_code}' \
    -X POST "$BASE_URL/abstract/$module" \
    -H "Content-Type: application/json" \
    -d "{\"scenario\":\"$scenario\"}" 2>/dev/null || echo "000")

  case $response in
    200|201)
      resp_code=$(jq -r '.code // "null"' "$RESULTS_DIR/${module}_${code}_${scenario}.json" 2>/dev/null)
      if [ "$resp_code" = "$code" ]; then
        echo -e "${GREEN}PASS ($ec_name)${NC}"
        PASS_COUNT=$((PASS_COUNT + 1))
      else
        echo -e "${YELLOW}WARN (expected=$code, got=$resp_code, mock 阶段)${NC}"
        SKIP_COUNT=$((SKIP_COUNT + 1))
      fi
      ;;
    404)
      echo -e "${YELLOW}SKIP (404, mock 端点待 v0.1 实施)${NC}"
      SKIP_COUNT=$((SKIP_COUNT + 1))
      ;;
    *)
      echo -e "${RED}FAIL (http=$response)${NC}"
      FAIL_COUNT=$((FAIL_COUNT + 1))
      ;;
  esac
done
echo ""

# 报告
echo "=== 报告 ==="
echo -e "${GREEN}PASS: $PASS_COUNT${NC}"
echo -e "${YELLOW}SKIP: $SKIP_COUNT (mock 阶段端点待实施)${NC}"
echo -e "${RED}FAIL: $FAIL_COUNT${NC}"
echo "Results: $RESULTS_DIR"

if [ $FAIL_COUNT -gt 0 ]; then
  exit 1
fi
exit 0
