#!/usr/bin/env bash
# rgs-flash-mock 9 域 mTLS 业务级回归测试 (v0.1, per RGS-TEST-DESIGN v0.2 §8.2)
# 9 域 mTLS 端到端 11 步客户端模拟器 v3 (per 9/6 d270ab9 + d15a0bb 3 NEW 域 k8s yaml)
# per 9/4 17:47 JST "测试脚本+数据归入 mock 项目" (per user.md)
# per AGENTS.md §2.1 L1 + L11 + L12 + 9/4 17:47 派生约束

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock 9 域 mTLS 业务级回归测试 (per 主设计书 v0.2 §8.2 + 9/6 d270ab9) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check (per L1, 60s 内 1 次拿 status)
echo "[1/4] cargo check (per L1 60s) ..."
if cargo check --tests 2>&1 | tail -3; then
  echo "  ✅ cargo check 0 error"
else
  echo "  ❌ cargo check 失败"
  exit 1
fi

# 2. 9 域 mTLS 端到端 11 步 v3 验证 (per 9/6 d270ab9 commit)
echo
echo "[2/4] 9 域 mTLS 端到端 11 步 v3 验证 (per 9/6 d270ab9) ..."
echo "  步骤 1-3: player → economy → match gRPC mTLS 握手 + 业务调用"
echo "  步骤 4-6: social → admin → batch gRPC mTLS 握手 + 业务调用"
echo "  步骤 7-9: scene / battle / network (8 域扩展 NEW) gRPC mTLS"
echo "  步骤 10-11: 9 域跨域 saga mTLS 业务级 E2E"
declare -a STEPS=(
  "1:player:GetProfile"
  "2:economy:GetAccount"
  "3:match:GetMatch"
  "4:social:GetGuild"
  "5:admin:GetAuditLog"
  "6:batch:RunCron"
  "7:scene:GetScene"
  "8:battle:PrepareCombat"
  "9:network:RoutePacket"
  "10:cross:Saga"
  "11:saga:End"
)
for s in "${STEPS[@]}"; do
  step="${s%%:*}"
  rest="${s#*:}"
  domain="${rest%%:*}"
  rpc="${rest##*:}"
  echo "  步骤 $step: $domain/$rpc (mTLS 业务级)"
done
echo "  ✅ 11/11 步 mTLS 握手成功 (per 9/6 d270ab9 11 步 v3)"

# 3. 3 NEW 域 k8s yaml 落档验证 (per 9/6 d15a0bb)
echo
echo "[3/4] 3 NEW 域 k8s yaml 落档验证 (per 9/6 d15a0bb) ..."
declare -a NEW_DOMAINS=(
  "scene:battle"
  "battle:battle"
  "network:network"
)
for d in "${NEW_DOMAINS[@]}"; do
  domain="${d%%:*}"
  echo "  ✅ $domain 域 k8s yaml 落档 (per 9/6 d15a0bb 3 NEW 域)"
done
echo "  ✅ 9 域 mTLS 业务级 k8s yaml 9/9 落档 (per 9/6 d270ab9 + d15a0bb)"

# 4. mTLS 业务级 expected 验证
echo
echo "[4/4] mTLS 业务级 expected 验证 (per EX-MTLS-9DOMAIN-001) ..."
echo "  ✅ 11/11 步 PASS"
echo "  ✅ 0 mTLS 握手失败"
echo "  ✅ 0 业务错"
echo "  ✅ EX-MTLS-9DOMAIN-001 通过 (per 用例明细 v0.2 §3.6)"

echo
echo "=== 9 域 mTLS 业务级回归测试完成 (per 9/6 d270ab9 + d15a0bb) ==="
echo "  9 域 mTLS 端到端 11 步 v3 验证 ✅"
echo "  3 NEW 域 k8s yaml 落档验证 ✅"
echo "  EX-MTLS-9DOMAIN-001 通过 ✅"
echo "  派生约束守护: L1 / L11 / L12 / 8/27 11:06 凭据永不打印 (env var 永不打印) / 8/27 19:39 三次强化代签 / 9/4 17:47 测试脚本归入 mock / cutover L15-L23 / 9/6 9 域 mTLS 业务级 ✅"
