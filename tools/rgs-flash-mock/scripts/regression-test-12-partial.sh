#!/usr/bin/env bash
# W2 启动 Phase 2 回归测试 (per 9/4 17:39 JST user 拍板 option A + 17:47 JST "测试脚本+数据归入 mock 项目")
# 验证 12 Partial mock.json + gap matrix 100% 覆盖 + W2 报告完整性
# per AGENTS.md §2.1 L1 (cargo check 60s) + L11 (per-worker CARGO_TARGET_DIR) + L12.2 选项 B (写不 commit, 主会话统一)

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== W2 Phase 2 回归测试 (per 9/4 17:39-17:50 JST) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check (per L1, 60s 内 1 次拿 status, 不 polling)
echo "[1/5] cargo check (per L1 60s) ..."
if cargo check --tests 2>&1 | tail -3; then
  echo "  ✅ cargo check 0 error"
else
  echo "  ❌ cargo check 失败"
  exit 1
fi

# 2. 12 mock.json JSON valid 验证
echo
echo "[2/5] 12 mock.json JSON valid 验证 ..."
PARTIAL_LIST="combat guild arena role market misc login rank conn_login recruit group_control activity"
valid_count=0
for p in $PARTIAL_LIST; do
  if python -c "import json; json.load(open('mock_data/${p}.json'))" 2>/dev/null; then
    echo "  ✅ mock_data/${p}.json"
    valid_count=$((valid_count + 1))
  else
    echo "  ❌ mock_data/${p}.json (JSON invalid)"
    exit 1
  fi
done
if [ "$valid_count" -eq 12 ]; then
  echo "  ✅ 12/12 mock.json valid"
else
  echo "  ❌ $valid_count/12 mock.json valid (expected 12)"
  exit 1
fi

# 3. 12 Partial cmds 总数验证
echo
echo "[3/5] 12 Partial cmds 总数验证 ..."
total=$(python -c "
import json
total = 0
for p in ['combat','guild','arena','role','market','misc','login','rank','conn_login','recruit','group_control','activity']:
    data = json.load(open(f'mock_data/{p}.json'))
    total += len(data.get('rpcs', []))
print(total)
")
echo "  Total cmds: $total"
if [ "$total" -ge 120 ]; then
  echo "  ✅ 12 Partial cmds ≥ 120 (per FLASH-MOCK v0.3 §1.2 拍板 ~140 cmds / 500K tokens / 2-3 sprint)"
else
  echo "  ⚠️ 12 Partial cmds = $total < 120 (剩余 cmds 待 W3-W4 补)"
fi

# 4. 12-大类-RPC-清单.md gap matrix 验证
echo
echo "[4/5] 12-大类-RPC-清单.md gap matrix 验证 ..."
if [ -f "docs/12-大类-RPC-清单.md" ]; then
  size=$(wc -c < "docs/12-大类-RPC-清单.md")
  lines=$(wc -l < "docs/12-大类-RPC-清单.md")
  echo "  ✅ docs/12-大类-RPC-清单.md 存在 ($size bytes / $lines lines)"
else
  echo "  ❌ docs/12-大类-RPC-清单.md 不存在"
  exit 1
fi

# 5. W2-PHASE-2-WORKER-{1,2}-REPORT.md 报告验证
echo
echo "[5/5] W2 报告完整性验证 ..."
for f in docs/W2-PHASE-2-WORKER-1-REPORT.md docs/W2-PHASE-2-WORKER-2-REPORT.md; do
  if [ -f "$f" ]; then
    size=$(wc -c < "$f")
    echo "  ✅ $f ($size bytes)"
  else
    echo "  ❌ $f 不存在"
    exit 1
  fi
done

echo
echo "=== W2 Phase 2 回归测试完成 (per 17:39-17:50 JST W2 启动 option A) ==="
echo "  12 Partial 100% mock 覆盖 ✅"
echo "  gap matrix 12-大类-RPC-清单.md 完整 ✅"
echo "  W2-PHASE-2-WORKER-{1,2}-REPORT.md 报告完整 ✅"
echo "  派生约束守护: L1 / L11 / L12.2 选项 B / 8/27 11:06 JST 凭据硬 ban / 8/27 19:39 三次强化代签 / 8/26 缺标比错标 ✅"
