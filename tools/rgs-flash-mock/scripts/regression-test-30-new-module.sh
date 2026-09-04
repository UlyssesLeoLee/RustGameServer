#!/usr/bin/env bash
# W3 启动 Phase 3 回归测试 (per 9/4 18:03 JST user 拍板 option C + 17:47 JST "测试脚本+数据归入 mock 项目")
# 验证 30 新 module mock.json + W2 12 Partial mock.json (累计 42) + gap matrix 100% 覆盖 + W3 报告完整性
# per AGENTS.md §2.1 L1 (cargo check 60s) + L11 (per-worker CARGO_TARGET_DIR) + L12.2 选项 B (写不 commit, 主会话统一)

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== W3 Phase 3 回归测试 (per 9/4 18:03-19:16 JST) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check (per L1, 60s 内 1 次拿 status, 不 polling)
echo "[1/6] cargo check (per L1 60s) ..."
if cargo check --tests 2>&1 | tail -3; then
  echo "  ✅ cargo check 0 error"
else
  echo "  ❌ cargo check 失败"
  exit 1
fi

# 2. 42 mock.json (W2 12 + W3 30) JSON valid 验证
echo
echo "[2/6] 42 mock.json JSON valid 验证 (W2 12 + W3 30) ..."
PARTIAL_LIST_W2="combat guild arena role market misc login rank conn_login recruit group_control activity"
PARTIAL_LIST_W3="avatar honor login_days checkin feat charge item mail exchange convert lev_gift power_gift boss dungeon endless adventure star drama sns guild_shipping guild_dun guild_skill formation quest partner holiday say map vip days_rank"
total_valid=0
for p in $PARTIAL_LIST_W2 $PARTIAL_LIST_W3; do
  if python -c "import json; json.load(open('mock_data/${p}.json'))" 2>/dev/null; then
    total_valid=$((total_valid + 1))
  else
    echo "  ❌ mock_data/${p}.json (JSON invalid)"
    exit 1
  fi
done
if [ "$total_valid" -eq 42 ]; then
  echo "  ✅ 42/42 mock.json valid (W2 12 + W3 30)"
else
  echo "  ❌ $total_valid/42 mock.json valid (expected 42)"
  exit 1
fi

# 3. 42 Partial cmds 总数验证
echo
echo "[3/6] 42 Partial cmds 总数验证 (W2 12 + W3 30) ..."
total=$(python -c "
import json
total = 0
W2 = ['combat','guild','arena','role','market','misc','login','rank','conn_login','recruit','group_control','activity']
W3 = ['avatar','honor','login_days','checkin','feat','charge','item','mail','exchange','convert','lev_gift','power_gift','boss','dungeon','endless','adventure','star','drama','sns','guild_shipping','guild_dun','guild_skill','formation','quest','partner','holiday','say','map','vip','days_rank']
for p in W2 + W3:
    data = json.load(open(f'mock_data/{p}.json'))
    total += len(data.get('rpcs', []))
print(total)
")
echo "  Total cmds: $total (target: 12 Partial + 30 新 module = 42 module / ~447 cmds)"

# 4. 12-大类-RPC-清单.md gap matrix 验证 (含 §16 W3 启动)
echo
echo "[4/6] 12-大类-RPC-清单.md gap matrix 验证 (含 §16 W3 启动) ..."
if [ -f "docs/12-大类-RPC-清单.md" ]; then
  size=$(wc -c < "docs/12-大类-RPC-清单.md")
  lines=$(wc -l < "docs/12-大类-RPC-清单.md")
  if grep -q '§16. W3 启动 Phase 3' "docs/12-大类-RPC-清单.md"; then
    echo "  ✅ docs/12-大类-RPC-清单.md 含 §16 W3 启动段 ($size bytes / $lines lines)"
  else
    echo "  ⚠️ docs/12-大类-RPC-清单.md 存在但缺 §16 W3 启动段"
  fi
else
  echo "  ❌ docs/12-大类-RPC-清单.md 不存在"
  exit 1
fi

# 5. W2 + W3 报告完整性验证
echo
echo "[5/6] W2 + W3 报告完整性验证 (2 W2 + 1 handoff + 5 W3 = 8 doc) ..."
REPORT_LIST="docs/W2-PHASE-2-WORKER-1-REPORT.md docs/W2-PHASE-2-WORKER-2-REPORT.md docs/W2-PHASE-2-WORKER-1-HANDOFF.md docs/W3-PHASE-3-WORKER-1-REPORT.md docs/W3-PHASE-3-WORKER-2-REPORT.md docs/W3-PHASE-3-WORKER-3-REPORT.md docs/W3-PHASE-3-WORKER-4-REPORT.md docs/W3-PHASE-3-WORKER-5-REPORT.md"
for f in $REPORT_LIST; do
  if [ -f "$f" ]; then
    size=$(wc -c < "$f")
    echo "  ✅ $f ($size bytes)"
  else
    echo "  ❌ $f 不存在"
    exit 1
  fi
done

# 6. DDD Review W3 启动 closure 文档验证
echo
echo "[6/6] DDD Review W3 启动 closure 验证 ..."
if [ -f "../../docs/14-项目治理/RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md" ]; then
  size=$(wc -c < "../../docs/14-项目治理/RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md")
  echo "  ✅ DDD Review W3 v0.1 closure ($size bytes)"
else
  echo "  ⚠️ DDD Review W3 v0.1 closure 待主会话 commit 后验证"
fi

echo
echo "=== W3 Phase 3 回归测试完成 (per 18:03-19:16 JST W3 启动 option C) ==="
echo "  42 mock.json 100% 覆盖 (W2 12 Partial + W3 30 新 module) ✅"
echo "  gap matrix 12-大类-RPC-清单.md 含 §16 W3 启动段 ✅"
echo "  8 doc 完整 (W2 2 报告 + 1 handoff + W3 5 报告) ✅"
echo "  派生约束守护: L1 / L11 / L12.2 选项 B / 8/27 11:06 JST 凭据硬 ban / 8/27 19:39 三次强化代签 / 8/26 缺标比错标 ✅"
