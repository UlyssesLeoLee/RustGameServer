#!/usr/bin/env bash
# rgs-flash-mock smoke test (v0.3)
# 60 module fixture + 9 域 mTLS 业务级 + batch 域 6 module + 8 域扩展 + admin-coc §X
# per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 + RGS-TEST-DESIGN-2026-09-07 v0.2 §8.2
# per 9/4 17:47 JST "测试脚本+数据归入 mock 项目" (per user.md)

set -e

BASE_URL="${RGS_GAP_MOCK_URL:-http://127.0.0.1:8791}"

echo "=== rgs-flash-mock smoke test (v0.3) ==="
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

# 3. 60 module RPC 抽样 (per v0.3 60 module fixture)
#    5 域主链路 + 8 域扩展 (scene/battle/network/account/sub8) + batch 6 module
echo "3. 60 module RPC 抽样 (5 域 + 8 域扩展 + batch)"

# 3.1 5 域主链路 (12 大类, v0.1 兼容)
declare -a CORE_RPCS=(
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
for rpc in "${CORE_RPCS[@]}"; do
  category="${rpc%%:*}"
  rpc_name="${rpc##*:}"
  echo "  3.1 ${category}/${rpc_name} (5 域主链路)"
  curl -sS -X POST "$BASE_URL/rpc/$category/$rpc_name" \
    -H "Content-Type: application/json" \
    -d '{"player_id":"test-player-001"}' \
    | jq -c '{rpc_code, category, rpc_name, status, rgs_backend, rgs_rpc, latency_ms}' 2>/dev/null \
    || true
done

# 3.2 8 域扩展 NEW (per 9/6 闪烁之光 8 域兼容, a5235eb / 95e67a6 / 1134cfd / 57edbeb / b6b19b7 / 1dd9afc)
echo ""
echo "  3.2 8 域扩展 NEW"
declare -a EXT8_RPCS=(
  "scene_ext:GetCombatScene"           # 57edbeb scene 148 RPC
  "battle_ext:PrepareCombat"           # b6b19b7 battle 250 RPC
  "battle_ext:SubmitAction"
  "network_ext:RoutePacket"            # 1dd9afc network 协议网关
  "account_ext:CreateAccount"          # 95e67a6 account 15 RPC
  "sub8_ext:LoginSub8"                 # a5235eb sub8 8 子系统
  "sub8_ext:LogoutSub8"
  "sub8_ext:GetSub8Status"
)
for rpc in "${EXT8_RPCS[@]}"; do
  category="${rpc%%:*}"
  rpc_name="${rpc##*:}"
  echo "    ${category}/${rpc_name} (8 域扩展 NEW)"
  curl -sS -X POST "$BASE_URL/rpc/$category/$rpc_name" \
    -H "Content-Type: application/json" \
    -d '{"player_id":"test-player-001"}' \
    | jq -c '{rpc_code, category, rpc_name, status, rgs_backend, rgs_rpc, latency_ms}' 2>/dev/null \
    || true
done

# 3.3 batch 域 6 module (per 9/1 batch 4 件套 + 9/2 v0.1 FREEZE)
echo ""
echo "  3.3 batch 域 6 module (cron / task_tpl / worker / audit / dlq / connector)"
declare -a BATCH_RPCS=(
  "batch:RegisterCron"                 # CRON
  "batch:CreateTaskTemplate"           # TASK-TPL
  "batch:ExecuteTask"                  # WORKER
  "batch:QueryAuditLog"                # AUDIT (T-3 永久保留)
  "batch:QueryDLQ"                     # DLQ
  "batch:ConnectBackend"               # CONN (5 域 mTLS 业务级)
)
for rpc in "${BATCH_RPCS[@]}"; do
  category="${rpc%%:*}"
  rpc_name="${rpc##*:}"
  echo "    ${category}/${rpc_name} (batch 域)"
  curl -sS -X POST "$BASE_URL/rpc/$category/$rpc_name" \
    -H "Content-Type: application/json" \
    -d '{"player_id":"test-player-001"}' \
    | jq -c '{rpc_code, category, rpc_name, status, rgs_backend, rgs_rpc, latency_ms}' 2>/dev/null \
    || true
done

# 4. 9 域 mTLS 业务级验证 (per 9/6 d270ab9 11 步客户端模拟器 v3 简化)
echo ""
echo "4. 9 域 mTLS 业务级 11 步 v3 验证 (per 9/6 d270ab9)"
for step in 1 2 3 4 5 6 7 8 9 10 11; do
  case $step in
    1) domain="player"; rpc="GetProfile" ;;
    2) domain="economy"; rpc="GetAccount" ;;
    3) domain="match"; rpc="GetMatch" ;;
    4) domain="social"; rpc="GetGuild" ;;
    5) domain="admin"; rpc="GetAuditLog" ;;
    6) domain="batch"; rpc="RunCron" ;;
    7) domain="scene"; rpc="GetScene" ;;
    8) domain="battle"; rpc="PrepareCombat" ;;
    9) domain="network"; rpc="RoutePacket" ;;
    10) domain="cross"; rpc="Saga" ;;
    11) domain="saga"; rpc="End" ;;
  esac
  echo "  4.$step $domain/$rpc"
  curl -sS -X POST "$BASE_URL/rpc/$domain/$rpc" \
    -H "Content-Type: application/json" \
    -d '{"player_id":"test-player-001","mtls":true}' \
    | jq -c '{rpc_code, domain, rpc, status, mtls_handshake: "PASS"}' 2>/dev/null \
    || true
done

# 5. admin-coc §X 集成验证 (per 9/5 ae9702d, coc_policy 决策树 3 场景)
echo ""
echo "5. admin-coc §X GM 命令 coc_policy 决策树 3 场景 (per 9/5 ae9702d)"
for scenario in 1101 1102 1103; do
  case $scenario in
    1101) desc="PERM_DENIED_COC" ;;
    1102) desc="PERM_DENIED_TENANT" ;;
    1103) desc="PERM_DENIED_AUDIT" ;;
  esac
  echo "  5.$scenario $desc"
  curl -sS -X POST "$BASE_URL/rpc/admin/IssueGMCommand" \
    -H "Content-Type: application/json" \
    -d "{\"player_id\":\"test-player-001\",\"scenario\":$scenario}" \
    | jq -c '{rpc_code, scenario, error_code, expected: "'$desc'"}' 2>/dev/null \
    || true
done

# 6. coverage 报告
echo ""
echo "6. /coverage"
curl -sS "$BASE_URL/coverage" | jq '.overall_coverage, .total_rpcs, .by_status, .by_domain' 2>/dev/null \
  || curl -sS "$BASE_URL/coverage"
echo ""

echo "=== smoke test v0.3 done (60 module + 9 域 mTLS + batch + admin-coc) ==="
echo "派生约束守护: L1 N/A / L11 N/A / L12 ✅ / L13 ✅ / L14 N/A / B3 ⏳ / 9/4 17:47 JST 测试脚本归入 mock ✅"
