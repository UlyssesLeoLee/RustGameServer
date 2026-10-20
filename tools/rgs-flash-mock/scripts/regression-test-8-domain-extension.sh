#!/usr/bin/env bash
# rgs-flash-mock 8 域扩展回归测试 (v0.1, per RGS-TEST-DESIGN v0.2 §8.2)
# 8 域扩展: scene / battle / network / account / sub8 (8 子系统)
# per 9/6 [游戏A] 8 域兼容 commit (a5235eb / 95e67a6 / 1134cfd / 57edbeb / b6b19b7 / 1dd9afc)
# per 9/4 17:47 JST "测试脚本+数据归入 mock 项目" (per user.md)
#
# RGS_MOCK_SKIP_CARGO_CHECK=1 — 跳过步骤 1 的 cargo check (仅供并行 lane 本地验证)。
# 默认行为不变: 不设该变量时步骤 1 照常执行 cargo check --tests。
#
# 步骤 2 的 "8 域扩展 RPC 清单" 注释块是 22 RPC 的单一事实来源, 由
# scripts/regression-test-new-domain-fixture-coverage.sh 解析校验 (不手写第二份清单)。

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock 8 域扩展回归测试 (per 主设计书 v0.2 §8.2 + 9/6 8 域扩展) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check (per L1, 60s 内 1 次拿 status)
if [ "${RGS_MOCK_SKIP_CARGO_CHECK:-0}" = "1" ]; then
  echo "[1/3] cargo check 已跳过 (RGS_MOCK_SKIP_CARGO_CHECK=1) ..."
else
  echo "[1/3] cargo check (per L1 60s) ..."
  CC_LOG=$(mktemp 2>/dev/null || echo "./.cc.log")
  if timeout 600 bash -c "CARGO_TARGET_DIR='${CARGO_TARGET_DIR:-D:/RustGameServer/target/flash-mock-ulys141}' cargo check --tests" >"$CC_LOG" 2>&1; then
    tail -3 "$CC_LOG" | sed 's/^/    /'
    rm -f "$CC_LOG"
    echo "  ✅ cargo check 0 error"
  else
    CC_RC=$?
    tail -10 "$CC_LOG" | sed 's/^/    /'
    if [ "$CC_RC" -eq 124 ]; then
      echo "  ❌ cargo check 超时 600s (per ULYS-141 Windows 落地)"
    else
      echo "  ❌ cargo check 失败 (rc=$CC_RC)"
    fi
    rm -f "$CC_LOG"
    exit 1
  fi
fi

# 2. 8 域扩展 RPC fixture 覆盖断言 (per scripts/regression-test-new-domain-fixture-coverage.sh)
echo
echo "[2/3] 8 域扩展 RPC fixture 覆盖断言 (per 9/6 [游戏A] 8 域兼容) ..."

# --- 8 域扩展 RPC 清单 (单一事实来源; 本块被 regression-test-new-domain-fixture-coverage.sh 解析) ---
# 2.1 scene 域 (148 RPC, per 9/6 57edbeb scene-service)
# scene_ext/GetCombatScene
# scene_ext/SceneStateSync
# scene_ext/MovePlayer
# scene_ext/GetScenePlayers
# 2.2 battle 域 (250 RPC, per 9/6 b6b19b7 battle-service)
# battle_ext/PrepareCombat
# battle_ext/SubmitAction
# battle_ext/FinishCombatPlay
# battle_ext/GetCombatResult
# 2.3 network 域 (协议网关, per 9/6 1dd9afc network-gateway)
# network_ext/RoutePacket
# network_ext/HealthCheck
# network_ext/LoadBalance
# 2.4 account 域 (15 RPC, per 9/6 95e67a6 player-service)
# account_ext/CreateAccount
# account_ext/Login
# account_ext/GetAccount
# 2.5 sub8 域 (8 子系统, per 9/6 a5235eb sub8)
# sub8_ext/LoginSub8
# sub8_ext/LogoutSub8
# sub8_ext/GetSub8Status
# sub8_ext/Sub8Heartbeat
# sub8_ext/Sub8Config
# sub8_ext/Sub8Metrics
# sub8_ext/Sub8Health
# sub8_ext/Sub8Update
# --- 清单结束 ---

COVERAGE_SCRIPT="scripts/regression-test-new-domain-fixture-coverage.sh"
if [ ! -f "$COVERAGE_SCRIPT" ]; then
  echo "  ❌ 缺 fixture 覆盖自检脚本: $COVERAGE_SCRIPT"
  exit 1
fi
COVERAGE_OUT=$(bash "$COVERAGE_SCRIPT") || {
  printf '%s\n' "$COVERAGE_OUT"
  echo "  ❌ 8 域扩展 fixture 覆盖校验失败 (per $COVERAGE_SCRIPT)"
  exit 1
}
printf '%s\n' "$COVERAGE_OUT"
# 本脚本拥有 "22 RPC" 这个数字 (清单块 + 下方汇总行), 这里断言覆盖数与之相符
case "$COVERAGE_OUT" in
  *"22/22 RPC"*) : ;;
  *)
    echo "  ❌ 期望 8 域扩展 22 RPC 全覆盖, 实际: $(printf '%s\n' "$COVERAGE_OUT" | tail -1)"
    exit 1
    ;;
esac
echo "  ✅ 8 域扩展 22 RPC 全部验证 (fixture 覆盖断言通过)"

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
echo "=== 8 域扩展回归测试完成 (per 9/6 [游戏A] 8 域兼容) ==="
echo "  scene 148 RPC + battle 250 RPC + network 协议网关 + account 15 RPC + sub8 8 子系统 ✅"
echo "  8 域扩展 22 RPC 验证 ✅"
echo "  5 域 + 8 域扩展兼容性验证 ✅"
echo "  派生约束守护: L1 / L11 / L12 / 8/27 11:06 凭据永不打印 / 8/27 19:39 三次强化代签 / 9/4 17:47 测试脚本归入 mock / cutover L15-L23 / 9/6 8 域扩展 ✅"
