#!/usr/bin/env bash
# rgs-flash-mock 跨域事件链回归 (v0.1)
#
# 闭环 README.md 已知缺口:
#   #1 gap_matrix 22 RPC stub 缺 E2E 故事链
#   #2 saga_abstract 3 module 仅 mock, 缺 e2e cross-domain event verification
#
# 验证载体: tests/saga_story_chain.rs (cargo integration test) +
#           src/saga_verify.rs (故事链 / 事件链模型 + 单元测试)
#
# 场景 (5 域链): player → economy → match → social → admin
#   201 GetPlayerProfile → 602 CreateAuction → 301 StartCombat
#   → 702 SendMessage → 1101 BanAccount
#   每个 rpc_code 都取自 src/gap_matrix.rs 的 22 个真实 stub, 期望状态对齐 stub 初值
#
# 断言 (可证伪, 不是 echo 式假绿):
#   - 默认故事链全绿
#   - 注入状态偏差 → 必须报出**具体失败步** (match.start / rpc 301)
#   - gap_matrix 无此 code → 必须报出 "gap_matrix 无 rpc_code"
#   - 悬空事件 (产生但无人消费) → 必须被检出
#   - 顺序错乱 (消费了更后面才产生的事件) → 必须被检出
#   - 域归属不符 (消费方域 != 事件目标域) → 必须被检出
#
# 退出码: 0 = 全部通过 / 非 0 = 有断言失败 (含 cargo test 失败)
#
# 环境:
#   - CARGO_TARGET_DIR 默认 E:/DevCache/cargo/mock-saga (per AGENTS.md L11/L30 per-worker E 盘隔离)
#   - RGS_MOCK_SKIP_CARGO_CHECK=1 可跳过前置 cargo check (默认仍会跑)
#   - RGS_MOCK_SKIP_CARGO_TEST=1 可跳过 cargo test (仅做静态检查)

set -uo pipefail

cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock 跨域事件链回归 (per README 已知缺口 #1 / #2) ==="
echo "  mock 项目根: $(pwd)"
echo

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-E:/DevCache/cargo/mock-saga}"

# 1. 编译门禁 (per AGENTS.md §2.1 L1)
if [ "${RGS_MOCK_SKIP_CARGO_CHECK:-0}" = "1" ]; then
  echo "[1/2] cargo check --tests (已按 RGS_MOCK_SKIP_CARGO_CHECK=1 跳过)"
else
  echo "[1/2] cargo check --tests (per L1) ..."
  CHECK_LOG=$(mktemp 2>/dev/null || echo "./.saga-check.log")
  if cargo check --tests --locked >"$CHECK_LOG" 2>&1; then
    tail -2 "$CHECK_LOG" | sed 's/^/    /'
    echo "  ✅ cargo check 0 error"
    rm -f "$CHECK_LOG"
  else
    RC=$?
    tail -20 "$CHECK_LOG" | sed 's/^/    /'
    echo "  ❌ cargo check 失败 (rc=$RC)"
    rm -f "$CHECK_LOG"
    exit 1
  fi
fi
echo

# 2. 故事链 / 事件链断言
if [ "${RGS_MOCK_SKIP_CARGO_TEST:-0}" = "1" ]; then
  echo "[2/2] cargo test --test saga_story_chain (已按 RGS_MOCK_SKIP_CARGO_TEST=1 跳过)"
  exit 0
fi

echo "[2/2] cargo test --test saga_story_chain --locked ..."
TEST_LOG=$(mktemp 2>/dev/null || echo "./.saga-test.log")
if cargo test --test saga_story_chain --locked >"$TEST_LOG" 2>&1; then
  grep -E "^test |^test result:" "$TEST_LOG" | sed 's/^/  /'
  echo "  ✅ 跨域故事链 + 事件链断言全过"
  rm -f "$TEST_LOG"
  exit 0
else
  RC=$?
  grep -E "^test |^test result:|^---- .* stdout|panicked at" "$TEST_LOG" | head -30 | sed 's/^/  /'
  echo "  ❌ 跨域故事链 / 事件链断言失败 (rc=$RC)"
  rm -f "$TEST_LOG"
  exit 1
fi
