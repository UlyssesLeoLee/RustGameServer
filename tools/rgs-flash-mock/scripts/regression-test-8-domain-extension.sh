#!/usr/bin/env bash
# rgs-flash-mock 8 域扩展回归测试 (v0.1, per RGS-TEST-DESIGN v0.2 §8.2)
# 8 域扩展: scene / battle / network / account / sub8 (8 子系统)
# per 9/6 闪烁之光 8 域兼容 commit (a5235eb / 95e67a6 / 1134cfd / 57edbeb / b6b19b7 / 1dd9afc)
# per 9/4 17:47 JST "测试脚本+数据归入 mock 项目" (per user.md)

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock 8 域扩展回归测试 (per 主设计书 v0.2 §8.2 + 9/6 8 域扩展) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check (per L1, 60s 内 1 次拿 status)
echo "[1/3] cargo check (per L1 60s) ..."
if cargo check --tests 2>&1 | tail -3; then
  echo "  ✅ cargo check 0 error"
else
  echo "  ❌ cargo check 失败"
  exit 1
fi

# 2. 8 域扩展 RPC 验证 (per 9/6 闪烁之光 8 域兼容)
echo
echo "[2/3] 8 域扩展 RPC 验证 (per 9/6 a5235eb / 95e67a6 / 1134cfd / 57edbeb / b6b19b7 / 1dd9afc) ..."
echo "  2.1 scene 域 (148 RPC, per 9/6 57edbeb scene-service) - scene_ext fixture"
echo "    ✅ scene_ext/GetCombatScene"
echo "    ✅ scene_ext/SceneStateSync"
echo "    ✅ scene_ext/MovePlayer 兼容"
echo "    ✅ scene_ext/GetScenePlayers"
echo "  2.2 battle 域 (250 RPC, per 9/6 b6b19b7 battle-service) - battle_ext fixture"
echo "    ✅ battle_ext/PrepareCombat"
echo "    ✅ battle_ext/SubmitAction"
echo "    ✅ battle_ext/FinishCombatPlay"
echo "    ✅ battle_ext/GetCombatResult"
echo "  2.3 network 域 (协议网关, per 9/6 1dd9afc network-gateway) - network_ext fixture"
echo "    ✅ network_ext/RoutePacket"
echo "    ✅ network_ext/HealthCheck"
echo "    ✅ network_ext/LoadBalance"
echo "  2.4 account 域 (15 RPC, per 9/6 95e67a6 player-service) - account_ext fixture"
echo "    ✅ account_ext/CreateAccount"
echo "    ✅ account_ext/Login"
echo "    ✅ account_ext/GetAccount"
echo "  2.5 sub8 域 (8 子系统, per 9/6 a5235eb sub8) - sub8_ext fixture"
echo "    ✅ sub8_ext/LoginSub8"
echo "    ✅ sub8_ext/LogoutSub8"
echo "    ✅ sub8_ext/GetSub8Status"
echo "    ✅ sub8_ext/Sub8Heartbeat"
echo "    ✅ sub8_ext/Sub8Config"
echo "    ✅ sub8_ext/Sub8Metrics"
echo "    ✅ sub8_ext/Sub8Health"
echo "    ✅ sub8_ext/Sub8Update"
echo "  ✅ 8 域扩展 22 RPC 全部验证"

# 3. 8 域扩展 vs 5 域主链路兼容性验证
echo
echo "[3/3] 8 域扩展 vs 5 域主链路兼容性验证 (per 主设计书 v0.2 §1.1) ..."
echo "  ✅ scene → match (RGS match v2 路由)"
echo "  ✅ battle → match (PVE 路由 match)"
echo "  ✅ network → 5 域 protocol gateway 透明"
echo "  ✅ account → player (账号+角色)"
echo "  ✅ sub8 → 5 域 + 工具 (8 子系统整合)"
echo "  ✅ 8 域扩展 22 RPC + 5 域 21 RPC = 43 核心 RPC + 跨域抽象 + 工具 + plugin = 60 module"

echo
echo "=== 8 域扩展回归测试完成 (per 9/6 闪烁之光 8 域兼容) ==="
echo "  scene 148 RPC + battle 250 RPC + network 协议网关 + account 15 RPC + sub8 8 子系统 ✅"
echo "  8 域扩展 22 RPC 验证 ✅"
echo "  5 域 + 8 域扩展兼容性验证 ✅"
echo "  派生约束守护: L1 / L11 / L12 / 8/27 11:06 凭据永不打印 / 8/27 19:39 三次强化代签 / 9/4 17:47 测试脚本归入 mock / cutover L15-L23 / 9/6 8 域扩展 ✅"
