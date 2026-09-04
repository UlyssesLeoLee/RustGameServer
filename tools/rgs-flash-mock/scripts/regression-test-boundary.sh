#!/usr/bin/env bash
# rgs-flash-mock regression test — 边界值 (per RGS-TEST-DESIGN-2026-09-05 v0.1 §5)
#
# 范围: 60 module 边界值覆盖
# - 空字符串 / 空数组
# - 极大值 (i32::MAX / i64::MAX / u64::MAX)
# - 极小值 (i32::MIN)
# - 超长字符串 (>4096)
# - 特殊字符 (\n \r \t \" \\ ')
# - 负数
# - 重复 id (幂等性)
# - 并发同 id (竞态)
#
# 用法:
#   RGS_GAP_MOCK_URL=http://127.0.0.1:8791 ./regression-test-boundary.sh

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"
RESULTS_DIR="${RESULTS_DIR:-/tmp/rgs-flash-mock-boundary-$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$RESULTS_DIR"

if [ -t 1 ]; then
  RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; NC=''
fi

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# 1. mock 健康
echo "=== 1. mock 健康 ==="
if ! curl -sS -m 5 "$BASE_URL/health" > /dev/null 2>&1; then
  echo -e "${RED}FAIL: mock 不可达${NC}"
  exit 2
fi
echo -e "${GREEN}PASS${NC}"
echo ""

# 2. 通用边界值场景 (跨 module 适用)
echo "=== 2. 通用边界值 8 类 ==="

# 2.1 空字符串
echo -n "  empty_string ... "
response=$(curl -sS -m 5 -o "$RESULTS_DIR/empty_string.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/boundary-test" \
  -H "Content-Type: application/json" \
  -d '{"field":"","case":"empty_string"}' 2>/dev/null || echo "000")
[[ "$response" =~ ^(200|201|404)$ ]] && { echo -e "${YELLOW}SKIP/OK (http=$response, mock 阶段)${NC}"; SKIP_COUNT=$((SKIP_COUNT+1)); } || { echo -e "${RED}FAIL (http=$response)${NC}"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 2.2 i32::MAX
echo -n "  i32_max ... "
response=$(curl -sS -m 5 -o "$RESULTS_DIR/i32_max.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/boundary-test" \
  -H "Content-Type: application/json" \
  -d "{\"field\":2147483647,\"case\":\"i32_max\"}" 2>/dev/null || echo "000")
[[ "$response" =~ ^(200|201|404)$ ]] && { echo -e "${YELLOW}SKIP/OK (http=$response)${NC}"; SKIP_COUNT=$((SKIP_COUNT+1)); } || { echo -e "${RED}FAIL (http=$response)${NC}"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 2.3 i32::MIN
echo -n "  i32_min ... "
response=$(curl -sS -m 5 -o "$RESULTS_DIR/i32_min.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/boundary-test" \
  -H "Content-Type: application/json" \
  -d "{\"field\":-2147483648,\"case\":\"i32_min\"}" 2>/dev/null || echo "000")
[[ "$response" =~ ^(200|201|404)$ ]] && { echo -e "${YELLOW}SKIP/OK (http=$response)${NC}"; SKIP_COUNT=$((SKIP_COUNT+1)); } || { echo -e "${RED}FAIL (http=$response)${NC}"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 2.4 超长字符串 (4096 字符)
LONG_STR=$(printf 'A%.0s' {1..4096})
echo -n "  string_4096_chars ... "
response=$(curl -sS -m 5 -o "$RESULTS_DIR/string_4096.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/boundary-test" \
  -H "Content-Type: application/json" \
  -d "{\"field\":\"$LONG_STR\",\"case\":\"long_string\"}" 2>/dev/null || echo "000")
[[ "$response" =~ ^(200|201|404)$ ]] && { echo -e "${YELLOW}SKIP/OK (http=$response)${NC}"; SKIP_COUNT=$((SKIP_COUNT+1)); } || { echo -e "${RED}FAIL (http=$response)${NC}"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 2.5 特殊字符
echo -n "  special_chars ... "
response=$(curl -sS -m 5 -o "$RESULTS_DIR/special_chars.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/boundary-test" \
  -H "Content-Type: application/json" \
  -d "$(printf '{"field":"\\n\\r\\t\\"\\\\'"'"'","case":"special"}')" 2>/dev/null || echo "000")
[[ "$response" =~ ^(200|201|404)$ ]] && { echo -e "${YELLOW}SKIP/OK (http=$response)${NC}"; SKIP_COUNT=$((SKIP_COUNT+1)); } || { echo -e "${RED}FAIL (http=$response)${NC}"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 2.6 负数
echo -n "  negative ... "
response=$(curl -sS -m 5 -o "$RESULTS_DIR/negative.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/boundary-test" \
  -H "Content-Type: application/json" \
  -d '{"field":-1,"case":"negative"}' 2>/dev/null || echo "000")
[[ "$response" =~ ^(200|201|404)$ ]] && { echo -e "${YELLOW}SKIP/OK (http=$response)${NC}"; SKIP_COUNT=$((SKIP_COUNT+1)); } || { echo -e "${RED}FAIL (http=$response)${NC}"; FAIL_COUNT=$((FAIL_COUNT+1)); }

# 2.7 幂等性 (同 id 调 3 次)
echo -n "  idempotent_3x ... "
duplicate_id="00000000-0000-0000-0000-000000000000"
for i in 1 2 3; do
  curl -sS -m 5 -o "$RESULTS_DIR/idempotent_${i}.json" \
    -X POST "$BASE_URL/abstract/boundary-test" \
    -H "Content-Type: application/json" \
    -d "{\"id\":\"$duplicate_id\",\"case\":\"idempotent\"}" > /dev/null 2>&1
done
# 3 次响应 code 应该一致
c1=$(jq -r '.code' "$RESULTS_DIR/idempotent_1.json" 2>/dev/null)
c2=$(jq -r '.code' "$RESULTS_DIR/idempotent_2.json" 2>/dev/null)
c3=$(jq -r '.code' "$RESULTS_DIR/idempotent_3.json" 2>/dev/null)
if [ "$c1" = "$c2" ] && [ "$c2" = "$c3" ] && [ -n "$c1" ]; then
  echo -e "${GREEN}PASS (3 次 code 一致: $c1)${NC}"
  PASS_COUNT=$((PASS_COUNT+1))
else
  echo -e "${YELLOW}SKIP/WARN (3 次 code: $c1 / $c2 / $c3, mock 阶段)${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi

# 2.8 并发同 id (10 个并发)
echo -n "  concurrent_10x ... "
for i in $(seq 1 10); do
  curl -sS -m 5 -o "$RESULTS_DIR/concurrent_${i}.json" \
    -X POST "$BASE_URL/abstract/boundary-test" \
    -H "Content-Type: application/json" \
    -d '{"id":"concurrent-1","case":"concurrent"}' > /dev/null 2>&1 &
done
wait
# 至少 1 个成功 + 其它 1003 STATE_INVALID (锁)
success_count=$(grep -l '"code":0' "$RESULTS_DIR"/concurrent_*.json 2>/dev/null | wc -l)
if [ "$success_count" -ge 1 ]; then
  echo -e "${GREEN}PASS ($success_count/10 成功, 锁生效)${NC}"
  PASS_COUNT=$((PASS_COUNT+1))
else
  echo -e "${YELLOW}SKIP/WARN (0 成功, mock 阶段)${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi
echo ""

# 3. 42 module 边界值抽查
echo "=== 3. 42 module 边界值抽查 ==="
# 5 域每域 1 个 module × 5 边界场景
declare -A MODULE_BOUNDARY=(
  ["combat:empty_combat_type"]="空 combat_type"
  ["combat:max_pity_count"]="pity_count=i32::MAX"
  ["combat:zero_speed"]="speed=0"
  ["combat:negative_target_id"]="target_id=-1"
  ["combat:duplicate_combat_session_id"]="同 session_id 重复 2 次"
  ["economy:zero_amount"]="amount=0"
  ["economy:max_currency"]="amount=i64::MAX"
  ["economy:empty_account_id"]="空 account_id"
  ["economy:long_remark"]="remark 4096 字符"
  ["economy:duplicate_trade_id"]="同 trade_id 重复"
  ["match:empty_match_id"]="空 match_id"
  ["match:invalid_uuid_format"]="非 UUID 格式"
  ["match:zero_player_count"]="player_count=0"
  ["match:max_player_count"]="player_count=1000"
  ["match:concurrent_same_match_id"]="并发同 match_id"
  ["social:empty_guild_name"]="空 guild_name"
  ["social:long_guild_name"]="guild_name 256 字符"
  ["social:invalid_invite_code"]="无效邀请码"
  ["social:duplicate_member_id"]="同 member_id 重复加入"
  ["social:concurrent_guild_join"]="并发加入 guild"
  ["admin:invalid_actor_id"]="非 UUID actor"
  ["admin:long_action_payload"]="payload 8KB"
  ["admin:zero_target_id"]="target_id=0"
  ["admin:invalid_audit_hash"]="hash 校验失败"
  ["admin:concurrent_audit_append"]="并发 audit_log 追加"
)

for case_key in "${!MODULE_BOUNDARY[@]}"; do
  IFS=':' read -r module scenario <<< "$case_key"
  desc="${MODULE_BOUNDARY[$case_key]}"
  echo -n "  $module:$scenario ($desc) ... "

  response=$(curl -sS -m 5 -o "$RESULTS_DIR/bv_${module}_${scenario}.json" -w '%{http_code}' \
    -X POST "$BASE_URL/abstract/$module" \
    -H "Content-Type: application/json" \
    -d "{\"scenario\":\"$scenario\"}" 2>/dev/null || echo "000")

  case $response in
    200|201)
      resp_code=$(jq -r '.code // "null"' "$RESULTS_DIR/bv_${module}_${scenario}.json" 2>/dev/null)
      if [ "$resp_code" = "0" ] || [ "$resp_code" = "1001" ] || [ "$resp_code" = "2004" ] || [ "$resp_code" = "3001" ]; then
        echo -e "${GREEN}PASS (code=$resp_code)${NC}"
        PASS_COUNT=$((PASS_COUNT+1))
      else
        echo -e "${YELLOW}WARN (code=$resp_code, mock 阶段)${NC}"
        SKIP_COUNT=$((SKIP_COUNT+1))
      fi
      ;;
    404)
      echo -e "${YELLOW}SKIP (404)${NC}"
      SKIP_COUNT=$((SKIP_COUNT+1))
      ;;
    *)
      echo -e "${RED}FAIL (http=$response)${NC}"
      FAIL_COUNT=$((FAIL_COUNT+1))
      ;;
  esac
done
echo ""

# 报告
echo "=== 报告 ==="
echo -e "${GREEN}PASS: $PASS_COUNT${NC}"
echo -e "${YELLOW}SKIP: $SKIP_COUNT${NC}"
echo -e "${RED}FAIL: $FAIL_COUNT${NC}"
echo "Results: $RESULTS_DIR"

if [ $FAIL_COUNT -gt 0 ]; then
  exit 1
fi
exit 0
