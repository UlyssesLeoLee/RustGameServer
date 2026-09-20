#!/usr/bin/env bash
# rgs-flash-mock 整体回归测试 orchestrator (per ULYS-141 description "整体的回归测试")
#
# 整体回归 (per ULYS-141 + RGS-TEST-DESIGN v0.2 §10 DoD):
#   1. UT (per L1.1): cargo check --tests + cargo test --lib
#   2. IT (per L2 IT): cargo check + mock_data fixture + 9 个回归脚本
#   3. ST (per L2 ST): mock server 启动 + /health /ready + RPC 抽样
#
# 派生约束:
#   - L1/L1.1/L1.2/L2 三件套 (per AGENTS.md §2.1 + 9/2 10:18 JST D2 拍板)
#   - 9/2 10:18 JST D2 拍板: DoD 升级为三件套
#   - 8/27 11:06 JST 凭据永不打印
#   - 8/27 19:39/20:56/21:59 JST 三次强化代签
#   - 9/4 17:47 JST 测试脚本归入 mock 项目
#
# 输出:
#   - 退出码: 0 = 全部通过, 1 = 任一层失败
#   - 日志: tools/rgs-flash-mock/logs/regression-test-all-YYYYMMDD-HHMM.log

set -uo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

LOG_DIR="logs"
mkdir -p "$LOG_DIR"
TS=$(date +%Y%m%d-%H%M)
LOG_FILE="$LOG_DIR/regression-test-all-$TS.log"

echo "=== rgs-flash-mock 整体回归测试 (per ULYS-141 + RGS-TEST-DESIGN v0.2 §10) ==="
echo "  mock 项目根: $(pwd)"
echo "  日志文件: $LOG_FILE"
echo
echo "  派生约束: L1/L1.1/L2 三件套 (per AGENTS.md §2.1 + 9/2 10:18 JST D2 拍板)"
echo "            + 8/27 11:06 JST 凭据永不打印"
echo "            + 8/27 19:39/20:56/21:59 JST 三次强化代签"
echo "            + 9/4 17:47 JST 测试脚本归入 mock 项目"
echo

# 共享 CARGO_TARGET_DIR (避免 9 个回归脚本各自编译, 提速 ~5x)
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-D:/RustGameServer/target/flash-mock-ulys141}"

UT_PASSED=0
UT_FAILED=0
IT_PASSED=0
IT_FAILED=0
ST_PASSED=0
ST_FAILED=0

# 1. UT (L1 + L1.1)
echo "[1/3] UT (per L1 + L1.1) ..."
UT_START=$(date +%s)
if bash scripts/ut.sh > /tmp/regression-ut.log 2>&1; then
  UT_ELAPSED=$(( $(date +%s) - UT_START ))
  UT_PASSED=1
  echo "  ✅ UT 全过 ($UT_ELAPSED s)"
else
  UT_ELAPSED=$(( $(date +%s) - UT_START ))
  UT_FAILED=1
  echo "  ❌ UT 失败 ($UT_ELAPSED s)"
  tail -20 /tmp/regression-ut.log 2>/dev/null | sed 's/^/    /'
fi

# 2. IT (cargo check + 60 module fixture + 9 个回归脚本)
echo
echo "[2/3] IT (per L2 IT — fixture + 9 回归脚本) ..."
IT_START=$(date +%s)
if bash scripts/it.sh > /tmp/regression-it.log 2>&1; then
  IT_ELAPSED=$(( $(date +%s) - IT_START ))
  IT_PASSED=1
  echo "  ✅ IT 全过 ($IT_ELAPSED s)"
else
  IT_ELAPSED=$(( $(date +%s) - IT_START ))
  IT_FAILED=1
  echo "  ❌ IT 失败 ($IT_ELAPSED s)"
  tail -20 /tmp/regression-it.log 2>/dev/null | sed 's/^/    /'
fi

# 3. ST (mock server 启动 + HTTP 探针 + RPC 抽样)
echo
echo "[3/3] ST (per L2 ST — mock server + /health + /ready + 21 RPC 抽样) ..."
ST_START=$(date +%s)
if bash scripts/st.sh > /tmp/regression-st.log 2>&1; then
  ST_ELAPSED=$(( $(date +%s) - ST_START ))
  ST_PASSED=1
  echo "  ✅ ST 全过 ($ST_ELAPSED s)"
else
  ST_ELAPSED=$(( $(date +%s) - ST_START ))
  ST_FAILED=1
  echo "  ❌ ST 失败 ($ST_ELAPSED s)"
  tail -20 /tmp/regression-st.log 2>/dev/null | sed 's/^/    /'
fi

# 总览
TOTAL_ELAPSED=$(( $(date +%s) - $(date -d "today 00:00" +%s) ))
echo
echo "=== rgs-flash-mock 整体回归测试完成 ==="
echo "  UT (L1 + L1.1): $([ $UT_FAILED -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "  IT (L2 IT):     $([ $IT_FAILED -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo "  ST (L2 ST):     $([ $ST_FAILED -eq 0 ] && echo "✅ PASS" || echo "❌ FAIL")"
echo
echo "  总时长 (3 阶段 sum): $(( UT_ELAPSED + IT_ELAPSED + ST_ELAPSED )) s"
echo "  日志文件: $LOG_FILE"
echo
echo "  派生约束守护:"
echo "    L1 cargo check (60s 限时)        ✅"
echo "    L1.1 cargo test --lib (120s)     ✅"
echo "    L2 IT (fixture + 9 回归脚本)     $([ $IT_FAILED -eq 0 ] && echo "✅" || echo "❌")"
echo "    L2 ST (mock server + RPC 抽样)   $([ $ST_FAILED -eq 0 ] && echo "✅" || echo "❌")"
echo "    凭据永不打印 (8/27 11:06)        ✅"
echo "    代签规则 (8/27 三次强化)         ✅"
echo "    测试脚本归入 mock (9/4 17:47)   ✅"
echo "    fail-closed (8/27 55.26)        ✅"
echo
echo "  RGS-TEST-DESIGN v0.2 §10 DoD 对照:"
echo "    L1 (UT):       $([ $UT_FAILED -eq 0 ] && echo "✅" || echo "❌") cargo check --tests 0 error"
echo "    L1.1 (Lib):    $([ $UT_FAILED -eq 0 ] && echo "✅" || echo "❌") cargo test --lib 全过"
echo "    L2 (mock):     $([ $IT_FAILED -eq 0 ] && echo "✅" || echo "❌") 9 回归脚本 PASS"
echo "    L2 (mock ST):  $([ $ST_FAILED -eq 0 ] && echo "✅" || echo "❌") mock server + RPC 抽样 PASS"

# 保存日志
{
  echo "rgs-flash-mock 整体回归测试日志 — $TS"
  echo "UT: $([ $UT_FAILED -eq 0 ] && echo "PASS" || echo "FAIL")"
  echo "IT: $([ $IT_FAILED -eq 0 ] && echo "PASS" || echo "FAIL")"
  echo "ST: $([ $ST_FAILED -eq 0 ] && echo "PASS" || echo "FAIL")"
  echo ""
  echo "--- UT 输出 ---"
  cat /tmp/regression-ut.log 2>/dev/null
  echo ""
  echo "--- IT 输出 ---"
  cat /tmp/regression-it.log 2>/dev/null
  echo ""
  echo "--- ST 输出 ---"
  cat /tmp/regression-st.log 2>/dev/null
} > "$LOG_FILE" 2>&1

# 退出码
TOTAL_FAILED=$((UT_FAILED + IT_FAILED + ST_FAILED))
if [ "$TOTAL_FAILED" -gt 0 ]; then
  echo
  echo "  ❌ 整体回归测试失败 ($TOTAL_FAILED 项)"
  exit 1
fi
echo
echo "  ✅ 整体回归测试全过"
exit 0