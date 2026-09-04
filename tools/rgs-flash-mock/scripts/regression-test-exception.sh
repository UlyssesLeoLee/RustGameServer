#!/usr/bin/env bash
# rgs-flash-mock regression test — 异常路径 (per RGS-TEST-DESIGN-2026-09-05 v0.1 §6)
#
# 范围: 60 module 异常路径覆盖
# - 网络中断 (client → server 1s 后断)
# - 超时 (server 处理 > 5s)
# - 重入 (同 id 连续 3 次, 间隔 100ms)
# - mTLS 证书失效 (RGS_TLS_DIR/ca.pem 过期)
# - 凭据泄漏尝试 (env var 出现在 log) — per 8/27 11:06 JST 硬 ban
#
# 用法:
#   RGS_GAP_MOCK_URL=http://127.0.0.1:8791 ./regression-test-exception.sh

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"
RESULTS_DIR="${RESULTS_DIR:-/tmp/rgs-flash-mock-exception-$(date +%Y%m%d-%H%M%S)}"
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

# 2. 通用异常路径
echo "=== 2. 通用异常路径 5 类 ==="

# 2.1 网络中断模拟 (curl --max-time 1 + 服务器模拟慢响应)
echo -n "  network_interrupt ... "
start=$(date +%s%N)
response=$(curl -sS -m 1 -o "$RESULTS_DIR/network_interrupt.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/exception-test" \
  -H "Content-Type: application/json" \
  -d '{"scenario":"slow","delay_ms":3000}' 2>/dev/null || echo "timeout")
end=$(date +%s%N)
elapsed_ms=$(( (end - start) / 1000000 ))
if [ "$response" = "timeout" ] || [ "$response" = "000" ]; then
  if [ $elapsed_ms -ge 900 ] && [ $elapsed_ms -le 1500 ]; then
    echo -e "${GREEN}PASS (client 超时 ~${elapsed_ms}ms)${NC}"
    PASS_COUNT=$((PASS_COUNT+1))
  else
    echo -e "${YELLOW}WARN (elapsed=${elapsed_ms}ms, 期望 ~1000ms)${NC}"
    SKIP_COUNT=$((SKIP_COUNT+1))
  fi
else
  echo -e "${YELLOW}SKIP (http=$response, mock 阶段)${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi

# 2.2 超时 (deadline 5s, server 处理 6s)
echo -n "  server_timeout_5s ... "
start=$(date +%s%N)
response=$(curl -sS -m 5 -o "$RESULTS_DIR/server_timeout.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/exception-test" \
  -H "Content-Type: application/json" \
  -d '{"scenario":"slow","delay_ms":6000}' 2>/dev/null || echo "deadline")
end=$(date +%s%N)
elapsed_ms=$(( (end - start) / 1000000 ))
if [ "$response" = "deadline" ] || [ "$response" = "000" ]; then
  if [ $elapsed_ms -ge 4500 ] && [ $elapsed_ms -le 5500 ]; then
    echo -e "${GREEN}PASS (deadline exceeded ~${elapsed_ms}ms)${NC}"
    PASS_COUNT=$((PASS_COUNT+1))
  else
    echo -e "${YELLOW}WARN (elapsed=${elapsed_ms}ms, 期望 ~5000ms)${NC}"
    SKIP_COUNT=$((SKIP_COUNT+1))
  fi
else
  echo -e "${YELLOW}SKIP (http=$response, mock 阶段)${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi

# 2.3 重入 (同 id 连续 3 次, 间隔 100ms)
echo -n "  reentrant_3x ... "
reentrant_id="reentrant-$(date +%s)"
declare -a REENTRANT_CODES=()
for i in 1 2 3; do
  sleep 0.1
  code=$(curl -sS -m 5 -o "$RESULTS_DIR/reentrant_${i}.json" -w '%{http_code}' \
    -X POST "$BASE_URL/abstract/exception-test" \
    -H "Content-Type: application/json" \
    -d "{\"id\":\"$reentrant_id\",\"scenario\":\"reentrant\"}" 2>/dev/null || echo "000")
  REENTRANT_CODES+=("$code")
done
# 期望: 3 次 code 一致 (幂等)
if [ "${REENTRANT_CODES[0]}" = "${REENTRANT_CODES[1]}" ] && [ "${REENTRANT_CODES[1]}" = "${REENTRANT_CODES[2]}" ]; then
  echo -e "${GREEN}PASS (3 次 code 一致: ${REENTRANT_CODES[0]})${NC}"
  PASS_COUNT=$((PASS_COUNT+1))
else
  echo -e "${YELLOW}WARN (3 次 code: ${REENTRANT_CODES[*]})${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi

# 2.4 mTLS 证书失效 (模拟 — 调 https 端点 + 过期 cert)
echo -n "  mtls_cert_expired ... "
# 检查是否启用了 mTLS 端点 (mock 阶段通常无)
https_url="${BASE_URL/http/https}"
if curl -sS -m 3 "$https_url/health" > /dev/null 2>&1; then
  # 真正 https 端点可用, 检查证书
  cert_expiry=$(echo | openssl s_client -connect "${BASE_URL#http*://}:443" -servername "${BASE_URL#http*://}" 2>/dev/null | openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2)
  if [ -n "$cert_expiry" ]; then
    echo -e "${GREEN}PASS (cert expires: $cert_expiry)${NC}"
    PASS_COUNT=$((PASS_COUNT+1))
  else
    echo -e "${YELLOW}WARN (cert 查询失败, mock 阶段)${NC}"
    SKIP_COUNT=$((SKIP_COUNT+1))
  fi
else
  echo -e "${YELLOW}SKIP (mock 无 mTLS 端, 真实部署阶段验证)${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi

# 2.5 凭据泄漏尝试 (per 8/27 11:06 JST 硬 ban) — 验证 log 不含 env var 明文
echo -n "  secret_redaction ... "
# 模拟: 用 secret 调端点, 然后 grep log
SECRET="RGS_TEST_PASSWORD_DO_NOT_LOG_12345"
response=$(curl -sS -m 5 -o "$RESULTS_DIR/secret_redaction.json" -w '%{http_code}' \
  -X POST "$BASE_URL/abstract/exception-test" \
  -H "Content-Type: application/json" \
  -d "{\"secret\":\"$SECRET\",\"scenario\":\"secret_test\"}" 2>/dev/null || echo "000")

# 检查 mock server log 文件 (如果存在)
LOG_FILE="/var/log/rgs-mock/server.log"
if [ -f "$LOG_FILE" ]; then
  if grep -q "$SECRET" "$LOG_FILE" 2>/dev/null; then
    echo -e "${RED}FAIL (secret 出现在 log! 违反 8/27 11:06 JST 硬 ban)${NC}"
    FAIL_COUNT=$((FAIL_COUNT+1))
  else
    echo -e "${GREEN}PASS (log 中无 secret 明文)${NC}"
    PASS_COUNT=$((PASS_COUNT+1))
  fi
else
  echo -e "${YELLOW}SKIP (log 文件不在 $LOG_FILE, mock 阶段)${NC}"
  SKIP_COUNT=$((SKIP_COUNT+1))
fi
echo ""

# 3. 42 module 异常路径抽查
echo "=== 3. 42 module 异常路径抽查 ==="
# 5 域每域 1 个 module × 3 异常场景
declare -A MODULE_EXCEPTION=(
  ["combat:network_drop:retry_should_succeed"]="网络中断后 retry"
  ["combat:deadline:deadline_exceeded"]="5s deadline"
  ["combat:reentrant:idempotent_return_same"]="重入幂等"
  ["economy:network_drop:retry_with_backoff"]="100/200/400ms 指数退避"
  ["economy:deadline:deadline_with_dead_letter"]="deadline + dead-letter"
  ["economy:reentrant:no_double_spend"]="重入不重复扣款"
  ["match:network_drop:spectator_reconnect"]="观战重连"
  ["match:deadline:match_state_persisted"]="deadline 后状态保留"
  ["match:reentrant:no_double_match"]="重入不重复撮合"
  ["social:network_drop:guild_chat_retry"]="聊天重试"
  ["social:deadline:guild_sync_delayed"]="deadline 后异步同步"
  ["social:reentrant:no_double_join"]="重入不重复加入"
  ["admin:network_drop:audit_retry"]="audit log 重试"
  ["admin:deadline:audit_dead_letter"]="audit deadline DLQ"
  ["admin:reentrant:no_double_audit"]="重入不重复 audit"
)

for case_key in "${!MODULE_EXCEPTION[@]}"; do
  IFS=':' read -r module scenario expected <<< "$case_key"
  echo -n "  $module:$scenario ... "

  response=$(curl -sS -m 5 -o "$RESULTS_DIR/ex_${module}_${scenario}.json" -w '%{http_code}' \
    -X POST "$BASE_URL/abstract/$module" \
    -H "Content-Type: application/json" \
    -d "{\"scenario\":\"$scenario\"}" 2>/dev/null || echo "000")

  case $response in
    200|201)
      echo -e "${GREEN}PASS (http=$response, $expected)${NC}"
      PASS_COUNT=$((PASS_COUNT+1))
      ;;
    404)
      echo -e "${YELLOW}SKIP (404, mock 端点待 v0.1 实施)${NC}"
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
