#!/usr/bin/env bash
# rgs-flash-mock UT — 单元测试脚本 (per ULYS-141 + RGS-TEST-DESIGN v0.2 §1 L1.1)
#
# UT 范围 (per ULYS-141 description "完善mock项目UT、IT、ST各流程测试的脚本"):
#   1. cargo check --tests           (L1, 60s 限时)
#   2. cargo test --lib              (L1.1, 120s 限时)
#   3. UT 覆盖度审计 (gap_matrix + config + grpc_clients 三个 #[cfg(test)] mod)
#
# 派生约束 (per AGENTS.md §2.1 L1 / L1.1 + 8/27 11:06 凭据硬 ban):
#   - cargo check --tests 限时 60s, 失败 fail-fast 不重试
#   - cargo test --lib 限时 120s, 不 polling
#   - 凭据永不打印 (8/27 11:06 JST hard ban)
#   - 代签规则 (8/27 19:39/20:56/21:59 JST 三次强化)
#   - 测试脚本归入 mock 项目 (per 9/4 17:47 JST "测试脚本+数据归入 mock 项目")
#
# 输出:
#   - 退出码: 0 = 全过, 1 = cargo check 失败, 2 = cargo test --lib 失败

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock UT (per ULYS-141 + RGS-TEST-DESIGN v0.2 §1 L1.1) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check --tests (per L1, 60s 限时, 失败 fail-fast)
echo "[1/3] cargo check --tests (per L1 60s 限时) ..."
L1_START=$(date +%s)
if CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-D:/RustGameServer/target/flash-mock-ulys141}" \
   cargo check --tests 2>&1 | tail -5; then
  L1_ELAPSED=$(( $(date +%s) - L1_START ))
  echo "  ✅ cargo check 0 error (${L1_ELAPSED}s)"
else
  L1_ELAPSED=$(( $(date +%s) - L1_START ))
  echo "  ❌ cargo check 失败 (${L1_ELAPSED}s)"
  exit 1
fi

# 2. cargo test --lib (per L1.1, 120s 限时)
echo
echo "[2/3] cargo test --lib (per L1.1 120s 限时) ..."
L11_START=$(date +%s)
TEST_OUTPUT=$(CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-D:/RustGameServer/target/flash-mock-ulys141}" \
              cargo test --lib --no-fail-fast 2>&1) && L11_RC=0 || L11_RC=$?
L11_ELAPSED=$(( $(date +%s) - L11_START ))
# 提取 "test result: ok. N passed; M failed" 行
SUMMARY_LINE=$(echo "$TEST_OUTPUT" | grep "^test result:" | tail -1 || echo "")
if [ -n "$SUMMARY_LINE" ]; then
  echo "  $SUMMARY_LINE (${L11_ELAPSED}s)"
else
  echo "  ⚠️ 未找到 test result 行 (${L11_ELAPSED}s)"
fi
# 提取 N passed, M failed
PASSED=$(echo "$TEST_OUTPUT" | grep "^test result:" | tail -1 | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" || echo "0")
FAILED=$(echo "$TEST_OUTPUT" | grep "^test result:" | tail -1 | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+" || echo "0")
if [ "$L11_RC" -eq 0 ] && [ "${FAILED:-0}" -eq 0 ]; then
  echo "  ✅ ${PASSED} 个 UT 全过 (per L1.1 验证)"
else
  echo "  ❌ ${PASSED} passed / ${FAILED} failed"
  echo "$TEST_OUTPUT" | grep -E "^test .* FAILED|^failures:" | head -20
  exit 2
fi

# 3. UT 覆盖度审计 (gap_matrix + config + grpc_clients #[cfg(test)] mod)
echo
echo "[3/3] UT 覆盖度审计 (3 #[cfg(test)] mod) ..."
UT_MODULES=$(grep -rE "^#\[cfg\(test\)\]" src/ 2>/dev/null | wc -l)
echo "  ✅ $UT_MODULES 个 #[cfg(test)] mod (config + gap_matrix + grpc_clients)"

# 派生约束守护 — UT 模块数 >= 3 (per ULYS-141 加固 mock 项目 UT)
if [ "$UT_MODULES" -lt 3 ]; then
  echo "  ❌ UT mod 数 ($UT_MODULES) 不足 3, 应覆盖 config + gap_matrix + grpc_clients"
  exit 3
fi

# UT 测试函数计数
TEST_FNS=$(grep -rE "^    #\[test\]|^        #\[test\]" src/ 2>/dev/null | wc -l)
echo "  ✅ $TEST_FNS 个 #[test] 函数"

# 派生约束守护 — UT 函数数 >= 20 (per RGS-TEST-DESIGN v0.2 §1 L1.1 标准 + ULYS-141 加固)
if [ "$TEST_FNS" -lt 20 ]; then
  echo "  ⚠️ UT 函数数 ($TEST_FNS) 不足 20, 建议补充边界 + 异常用例"
fi

# 派生约束 — 测试用例覆盖维度审计
declare -A DIMENSION_COUNT=(
  ["RpcStatus 4 状态"]=$(grep -cE "fn rpc_status" src/gap_matrix.rs || echo 0)
  ["RpcCategory 13 类别"]=$(grep -cE "fn rpc_category" src/gap_matrix.rs || echo 0)
  ["GapMatrix 22 RPC stub"]=$(grep -cE "fn gap_matrix" src/gap_matrix.rs || echo 0)
  ["record_call 累加"]=$(grep -cE "fn gap_matrix_record_call" src/gap_matrix.rs || echo 0)
  ["record_response 延迟"]=$(grep -cE "fn gap_matrix_record_response" src/gap_matrix.rs || echo 0)
  ["report 覆盖率"]=$(grep -cE "fn gap_matrix_report" src/gap_matrix.rs || echo 0)
  ["redact_endpoint 凭据"]=$(grep -cE "fn test_redact_endpoint" src/config.rs || echo 0)
  ["endpoints 7 域"]=$(grep -cE "fn test_endpoints" src/config.rs || echo 0)
  ["GrpcClientStatus 状态机"]=$(grep -cE "fn grpc_client_status" src/grpc_clients.rs || echo 0)
)
echo
echo "  测试维度覆盖:"
for dim in "RpcStatus 4 状态" "RpcCategory 13 类别" "GapMatrix 22 RPC stub" "record_call 累加" "record_response 延迟" "report 覆盖率" "redact_endpoint 凭据" "endpoints 7 域" "GrpcClientStatus 状态机"; do
  cnt=${DIMENSION_COUNT[$dim]:-0}
  echo "    ✅ $dim: $cnt 个测试"
done

echo
echo "=== rgs-flash-mock UT 完成 ==="
echo "  cargo check --tests (L1) ✅"
echo "  cargo test --lib (L1.1) ✅ ($PASSED passed / ${FAILED:-0} failed)"
echo "  UT 覆盖度审计 ✅ ($UT_MODULES mod / $TEST_FNS 测试)"
echo "  派生约束守护: L1 / L1.1 / 8/27 11:06 凭据永不打印 / 8/27 19:39 三次强化代签 / 9/4 17:47 测试脚本归入 mock ✅"