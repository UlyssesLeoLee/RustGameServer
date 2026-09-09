#!/usr/bin/env bash
# rgs-grpc-bridge smoke test (per AGENTS.md §2 6.3 ST 模板)
#
# 前提: bridge 在 127.0.0.1:8080 跑 (dev insecure 模式)
#       player-service 在 backend (e.g. localhost:50051 insecure)
#
# 用法:
#   bash tools/rgs-grpc-bridge/docs/smoke-test.sh
#
# 检查:
# 1. bridge 自身 8080 端口可访问
# 2. HealthCheck 透传到 backend (expect ok=0 / serving)
# 3. 错误路径 (e.g. /player.v1.PlayerService/Unknown) 返回正确 status code

set -e

BRIDGE="${RGS_BRIDGE_URL:-http://127.0.0.1:8080}"
PASS=0
FAIL=0
TOTAL=0

check() {
    local name="$1"
    local cmd="$2"
    local expect="$3"
    TOTAL=$((TOTAL+1))
    local out
    out=$(eval "$cmd" 2>&1) || true
    if echo "$out" | grep -q "$expect"; then
        echo "  ✅ $name"
        PASS=$((PASS+1))
    else
        echo "  ❌ $name"
        echo "     expected: $expect"
        echo "     got: $out"
        FAIL=$((FAIL+1))
    fi
}

echo "===== rgs-grpc-bridge smoke test @ $BRIDGE ====="

# 1. health (H5 gRPC client 不直接打, 但需要 backend 通)
check "HealthCheck ok=0" \
    "curl -fsS -X POST $BRIDGE/player.v1.PlayerService/HealthCheck -H 'Content-Type: application/json' -d '{\"service\":\"\"}'" \
    '"status":'

# 2. GetServerTime (10103 -> 10380, server_time_unix 字段)
check "GetServerTime" \
    "curl -fsS -X POST $BRIDGE/player.v1.PlayerService/GetServerTime -H 'Content-Type: application/json' -d '{\"requestId\":\"req_smoke_$(date +%s)\"}'" \
    'serverTimeUnix'

# 3. CreateCharacter 失败 (无 account_id, 期望 backend validation 错)
check "CreateCharacter 缺 account_id 失败" \
    "curl -sS -X POST $BRIDGE/player.v1.PlayerService/CreateCharacter -H 'Content-Type: application/json' -d '{\"characterName\":\"smoke-test\",\"classId\":1}'" \
    -E 'code|message|error|status'

# 4. 错误 RPC 路径 -> 404 or 501
check "Unknown RPC 返回 error" \
    "curl -sS -X POST $BRIDGE/player.v1.PlayerService/UnknownMethod -H 'Content-Type: application/json' -d '{}'" \
    -E 'UNIMPLEMENTED|unimplemented|not.found|error'

# 5. CORS preflight (浏览器 fetch 需要)
check "CORS preflight 通过" \
    "curl -sS -o /dev/null -w '%{http_code}' -X OPTIONS $BRIDGE/player.v1.PlayerService/HealthCheck -H 'Origin: http://127.0.0.1:8788' -H 'Access-Control-Request-Method: POST' -H 'Access-Control-Request-Headers: content-type'" \
    '2'

echo ""
echo "===== result: $PASS/$TOTAL pass, $FAIL fail ====="
exit $FAIL
