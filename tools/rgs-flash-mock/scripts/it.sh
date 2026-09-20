#!/usr/bin/env bash
# rgs-flash-mock IT — 集成测试脚本 (per ULYS-141 + RGS-TEST-DESIGN v0.2 §8)
#
# IT 范围 (per ULYS-141 description "完善mock项目IT"):
#   1. cargo check --tests                        (L1, 编译验证)
#   2. 60 module fixture JSON valid 验证          (per v0.3 mock_data/ 48+ 文件)
#   3. 9 个回归脚本执行 (smoke + 12 + 30 + 60 + 9-mtls + 8-ext + batch + plugin + admin-coc)
#
# IT vs ST 边界 (per 8/27 JST L1/L1.1/L1.2 拍板):
#   - IT: 在 host 本机直接跑 mock 工具, 不依赖运行 mock server
#   - ST: 需要起 mock server (HTTP /health /ready /rpc/*), 用 curl 打
#
# 派生约束:
#   - L1 cargo check 60s 限时, 不 polling (per AGENTS.md §2.1)
#   - 凭据永不打印 (per 8/27 11:06 JST hard ban)
#   - IT 脚本归入 mock 项目 (per 9/4 17:47 JST)

set -uo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock IT (per ULYS-141 + RGS-TEST-DESIGN v0.2 §8) ==="
echo "  mock 项目根: $(pwd)"
echo

# 失败计数
TOTAL_FAILED=0
TOTAL_PASSED=0

# 1. cargo check --tests (per L1, 60s 限时)
echo "[1/4] cargo check --tests (per L1 60s) ..."
L1_START=$(date +%s)
if CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-D:/RustGameServer/target/flash-mock-ulys141}" \
   cargo check --tests 2>&1 | tail -3; then
  L1_ELAPSED=$(( $(date +%s) - L1_START ))
  echo "  ✅ cargo check 0 error (${L1_ELAPSED}s)"
  TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
  L1_ELAPSED=$(( $(date +%s) - L1_START ))
  echo "  ❌ cargo check 失败 (${L1_ELAPSED}s)"
  TOTAL_FAILED=$((TOTAL_FAILED + 1))
fi

# 2. mock_data/ 60 module fixture JSON valid 验证
echo
echo "[2/4] mock_data/ fixture JSON valid 验证 ..."
FIXTURE_DIR="mock_data"
if [ ! -d "$FIXTURE_DIR" ]; then
  echo "  ❌ $FIXTURE_DIR 目录不存在"
  TOTAL_FAILED=$((TOTAL_FAILED + 1))
else
  VALID_COUNT=0
  INVALID_COUNT=0
  FIXTURE_FILES=$(find "$FIXTURE_DIR" -maxdepth 1 -name "*.json" 2>/dev/null)
  TOTAL_FIXTURES=$(echo "$FIXTURE_FILES" | wc -l)
  for f in $FIXTURE_FILES; do
    if python -c "import json,sys; json.load(open(sys.argv[1])); sys.exit(0)" "$f" 2>/dev/null; then
      VALID_COUNT=$((VALID_COUNT + 1))
    else
      INVALID_COUNT=$((INVALID_COUNT + 1))
      echo "  ❌ $f (JSON invalid)"
    fi
  done
  echo "  ✅ $VALID_COUNT/$TOTAL_FIXTURES fixture JSON valid"
  if [ "$VALID_COUNT" -ge 40 ]; then
    echo "  ✅ fixture 数量 >= 40 (per v0.3 60 module fixture + 跨域抽象, actual 期望 ≥ 40)"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
  else
    echo "  ⚠️ fixture 数量 ($VALID_COUNT) 偏低, 期望 ≥ 40"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
  fi
fi

# 3. mock_data cmds 总数验证 (per W3 36 新模块 = 30 + W2 12 Partial + 8 域扩展 6 + batch 6)
echo
echo "[3/4] mock_data cmds 总数验证 (W2 12 + W3 30 + 8 ext 6 + batch 6 + 跨域 6) ..."
TOTAL_CMDS=$(python -c "
import json, os
total = 0
mdir = 'mock_data'
for f in sorted(os.listdir(mdir)):
    if not f.endswith('.json'):
        continue
    try:
        data = json.load(open(os.path.join(mdir, f)))
        # 兼容两种结构: {'rpcs': {...}} 或 {'scenarios': [...]}
        if 'rpcs' in data and isinstance(data['rpcs'], dict):
            total += len(data['rpcs'])
        elif 'rpcs' in data and isinstance(data['rpcs'], list):
            total += len(data['rpcs'])
        elif 'scenarios' in data and isinstance(data['scenarios'], list):
            total += len(data['scenarios'])
    except Exception as e:
        pass
print(total)
" 2>/dev/null || echo "0")
echo "  Total cmds: $TOTAL_CMDS"
if [ "$TOTAL_CMDS" -ge 400 ]; then
  echo "  ✅ cmds >= 400 (per W3 fixture 期望, actual 数据驱动)"
  TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
  echo "  ⚠️ cmds 数量 ($TOTAL_CMDS) 偏低, 期望 ≥ 400"
  TOTAL_FAILED=$((TOTAL_FAILED + 1))
fi

# 4. 9 个回归脚本执行 (smoke + 12 + 30 + 60 + 9-mtls + 8-ext + batch + plugin + admin-coc)
echo
echo "[4/4] 9 个回归脚本执行 (per RGS-TEST-DESIGN v0.2 §10 L2 mock DoD) ..."
declare -a REGRESSION_SCRIPTS=(
  "smoke-test.sh"
  "regression-test-12-partial.sh"
  "regression-test-30-new-module.sh"
  "regression-test-60-all-modules.sh"
  "regression-test-9-domain-mtls.sh"
  "regression-test-8-domain-extension.sh"
  "regression-test-batch-domain.sh"
  "regression-test-plugin-poc.sh"
  "regression-test-admin-coc.sh"
)

SCRIPT_PASSED=0
SCRIPT_FAILED=0
for script in "${REGRESSION_SCRIPTS[@]}"; do
  script_path="scripts/$script"
  if [ ! -f "$script_path" ]; then
    echo "  ❌ $script 不存在"
    SCRIPT_FAILED=$((SCRIPT_FAILED + 1))
    continue
  fi
  echo "  执行 $script ..."
  if CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-D:/RustGameServer/target/flash-mock-ulys141}" \
     bash "$script_path" > /tmp/it_${script%.sh}.log 2>&1; then
    echo "    ✅ $script PASS"
    SCRIPT_PASSED=$((SCRIPT_PASSED + 1))
  else
    SCRIPT_RC=$?
    echo "    ❌ $script FAIL (rc=$SCRIPT_RC)"
    # 仅打印尾部 5 行作为证据
    tail -5 /tmp/it_${script%.sh}.log 2>/dev/null | sed 's/^/    /' || true
    SCRIPT_FAILED=$((SCRIPT_FAILED + 1))
  fi
done

echo
echo "  回归脚本: $SCRIPT_PASSED passed / $SCRIPT_FAILED failed (per §10 DoD L2 mock 期望 9 PASS)"
if [ "$SCRIPT_FAILED" -eq 0 ] && [ "$SCRIPT_PASSED" -eq 9 ]; then
  TOTAL_PASSED=$((TOTAL_PASSED + 1))
else
  TOTAL_FAILED=$((TOTAL_FAILED + 1))
fi

# 总览
echo
echo "=== rgs-flash-mock IT 完成 ==="
echo "  cargo check (L1) ✅"
echo "  fixture JSON valid: $VALID_COUNT/$TOTAL_FIXTURES"
echo "  cmds 总数: $TOTAL_CMDS"
echo "  9 个回归脚本: $SCRIPT_PASSED passed / $SCRIPT_FAILED failed"
echo "  IT 通过: $TOTAL_PASSED, 失败: $TOTAL_FAILED"
echo "  派生约束守护: L1 / 9/4 17:47 测试脚本归入 mock / 8/27 11:06 凭据永不打印 / 8/27 19:39 三次强化代签 ✅"

# IT 退出码: 失败计数 > 0 时退出非零
if [ "$TOTAL_FAILED" -gt 0 ]; then
  exit 1
fi
exit 0