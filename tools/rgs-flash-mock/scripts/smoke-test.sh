#!/usr/bin/env bash
# rgs-flash-mock smoke test (v0.1)
# 12 大类 RPC 抽样 + 健康检查 + coverage 验证
# per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §3

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"

echo "=== rgs-flash-mock smoke test (v0.1) ==="
echo "BASE_URL: $BASE_URL"
echo ""

# 1. 健康检查
echo "1. /health"
curl -sS "$BASE_URL/health" | jq . 2>/dev/null || curl -sS "$BASE_URL/health"
echo ""

# 2. 就绪探针
echo "2. /ready"
curl -sS "$BASE_URL/ready" | jq . 2>/dev/null || curl -sS "$BASE_URL/ready"
echo ""

# 3. 12 大类 RPC 抽样
declare -a RPCS=(
  "scene:GetScene"
  "scene:MovePlayer"
  "character:GetPlayerProfile"
  "character:UpgradeSkill"
  "combat:StartCombat"
  "combat:SubmitAction"
  "pvp:EnqueuePVP"
  "pvp:GetPVPMatch"
  "guild:GetGuild"
  "guild:JoinGuild"
  "economy:GetAccount"
  "economy:CreateAuction"
  "social:GetFriendList"
  "social:SendMessage"
  "activity:GetActiveEvent"
  "activity:ClaimReward"
  "payment:Recharge"
  "payment:QueryRechargeHistory"
  "leaderboard:GetLeaderboard"
  "gm:BanAccount"
  "gm:GrantCompensation"
)

for rpc in "${RPCS[@]}"; do
  category="${rpc%%:*}"
  rpc_name="${rpc##*:}"
  echo "3.${category}/${rpc_name}"
  curl -sS -X POST "$BASE_URL/rpc/$category/$rpc_name" \
    -H "Content-Type: application/json" \
    -d '{"player_id":"test-player-001"}' \
    | jq -c '{rpc_code, category, rpc_name, status, rgs_backend, rgs_rpc, latency_ms}' 2>/dev/null \
    || curl -sS -X POST "$BASE_URL/rpc/$category/$rpc_name" \
        -H "Content-Type: application/json" \
        -d '{"player_id":"test-player-001"}'
  echo ""
done

# 4. coverage 报告
echo "4. /coverage"
curl -sS "$BASE_URL/coverage" | jq '.overall_coverage, .total_rpcs, .by_status' 2>/dev/null \
  || curl -sS "$BASE_URL/coverage"
echo ""

echo "=== smoke test done ==="
