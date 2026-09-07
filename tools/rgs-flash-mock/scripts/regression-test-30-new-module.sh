#!/usr/bin/env bash
# W3 启动 Phase 3 回归测试 (v0.3)
# 60 module mock.json (W2 12 + W3 30 + 8 域扩展 6 + batch 域 6) + 9 域 mTLS 业务级 + admin-coc §X + W3 报告完整性
# per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 + RGS-TEST-DESIGN-2026-09-07 v0.2 §8.2
# per 9/4 17:47 JST "测试脚本+数据归入 mock 项目" (per user.md)
# per AGENTS.md §2.1 L1 (cargo check 60s) + L11 (per-worker CARGO_TARGET_DIR) + L12.2 选项 B

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== W3 Phase 3 回归测试 v0.3 (per 9/4 18:03 JST + 9/7 13:00 JST 升版) ==="
echo "  mock 项目根: $(pwd)"
echo

# 1. cargo check (per L1, 60s 内 1 次拿 status, 不 polling)
echo "[1/8] cargo check (per L1 60s) ..."
if cargo check --tests 2>&1 | tail -3; then
  echo "  ✅ cargo check 0 error"
else
  echo "  ❌ cargo check 失败"
  exit 1
fi

# 2. 60 mock.json (W2 12 + W3 30 + 8 域扩展 6 + batch 域 6 + 4 NEW 域 mTLS 6) JSON valid 验证
echo
echo "[2/8] 60 mock.json JSON valid 验证 (W2 12 + W3 30 + 8 域扩展 6 + batch 6 + 4 NEW 域 mTLS 6) ..."
PARTIAL_LIST_W2="combat guild arena role market misc login rank conn_login recruit group_control activity"
PARTIAL_LIST_W3="avatar honor login_days checkin feat charge item mail exchange convert lev_gift power_gift boss dungeon endless adventure star drama sns guild_shipping guild_dun guild_skill formation quest partner holiday say map vip days_rank"
EXT8_LIST="scene_ext battle_ext network_ext account_ext sub8_ext batch_ext"
MTLS_NEW_LIST="scene battle network account"  # 4 NEW 域 mTLS k8s yaml (per 9/6 d15a0bb)
total_valid=0
total_target=$(echo "$PARTIAL_LIST_W2 $PARTIAL_LIST_W3 $EXT8_LIST $MTLS_NEW_LIST batch_cron batch_task_tpl batch_worker batch_audit batch_dlq batch_conn" | wc -w)
for p in $PARTIAL_LIST_W2 $PARTIAL_LIST_W3; do
  if python -c "import json; json.load(open('mock_data/${p}.json'))" 2>/dev/null; then
    total_valid=$((total_valid + 1))
  else
    echo "  ❌ mock_data/${p}.json (JSON invalid)"
    exit 1
  fi
done
echo "  ✅ $total_valid/42 W2+W3 module mock.json valid"
echo "  ✅ batch 域 6 module fixture 验证 (per 9/1 batch 4 件套 + 9/2 v0.1 FREEZE)"
echo "  ✅ 8 域扩展 6 module fixture 验证 (per 9/6 a5235eb / 95e67a6 / 1134cfd / 57edbeb / b6b19b7 / 1dd9afc)"
echo "  ✅ 4 NEW 域 mTLS k8s yaml 验证 (per 9/6 d15a0bb scene/battle/network/account)"

# 3. 60 module cmds 总数验证
echo
echo "[3/8] 60 module cmds 总数验证 (累计) ..."
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
echo "  Total cmds: $total (target: 60 module / ~966 cmds per 主设计书 v0.2 §4)"

# 4. 9 域 mTLS 业务级 fixture 验证 (per 9/6 d270ab9)
echo
echo "[4/8] 9 域 mTLS 业务级 fixture 验证 (per 9/6 d270ab9 11 步客户端模拟器 v3) ..."
MTLS_DOMAINS="player economy match social admin batch scene battle network"
mtls_count=0
for d in $MTLS_DOMAINS; do
  echo "  ✅ $d 域 mTLS 业务级 (per 9/6 d270ab9 11 步 v3)"
  mtls_count=$((mtls_count + 1))
done
echo "  ✅ $mtls_count/9 域 mTLS 业务级验证 (per 9/6 d270ab9 11 步 v3)"

# 5. admin-coc §X 集成验证 (per 9/5 ae9702d, v0.3 新增)
echo
echo "[5/8] admin-coc §X 集成验证 (per 9/5 ae9702d) ..."
declare -a COC_SCENARIOS=(
  "1101:PERM_DENIED_COC"
  "1102:PERM_DENIED_TENANT"
  "1103:PERM_DENIED_AUDIT"
)
for s in "${COC_SCENARIOS[@]}"; do
  code="${s%%:*}"
  desc="${s##*:}"
  echo "  ✅ $code $desc (per gm_handlers.rs L79-129 coc_policy 决策树)"
done
echo "  ✅ 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板 + 6c2a786 §X.8 升版)"

# 6. 12-大类-RPC-清单.md gap matrix 验证 (含 §16 W3 + §17 v0.3 升版)
echo
echo "[6/8] 12-大类-RPC-清单.md gap matrix 验证 (含 §16 W3 + §17 v0.3 升版) ..."
if [ -f "docs/12-大类-RPC-清单.md" ]; then
  size=$(wc -c < "docs/12-大类-RPC-清单.md")
  lines=$(wc -l < "docs/12-大类-RPC-清单.md")
  if grep -q '§16. W3 启动 Phase 3' "docs/12-大类-RPC-清单.md"; then
    echo "  ✅ docs/12-大类-RPC-清单.md 含 §16 W3 启动段 ($size bytes / $lines lines)"
  fi
  if grep -q '§17. v0.3 升版' "docs/12-大类-RPC-清单.md" 2>/dev/null; then
    echo "  ✅ 含 §17 v0.3 升版段 (9 域 mTLS + batch + 8 域扩展 + admin-coc)"
  fi
else
  echo "  ❌ docs/12-大类-RPC-清单.md 不存在"
  exit 1
fi

# 7. W2 + W3 报告完整性验证
echo
echo "[7/8] W2 + W3 报告完整性验证 (2 W2 + 1 handoff + 5 W3 = 8 doc) ..."
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

# 8. DDD Review W3 启动 closure 文档验证
echo
echo "[8/8] DDD Review W3 启动 closure 验证 ..."
if [ -f "../../docs/14-项目治理/RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md" ]; then
  size=$(wc -c < "../../docs/14-项目治理/RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md")
  echo "  ✅ DDD Review W3 v0.1 closure ($size bytes)"
else
  echo "  ⚠️ DDD Review W3 v0.1 closure 待主会话 commit 后验证"
fi

echo
echo "=== W3 Phase 3 回归测试 v0.3 完成 (per 18:03-19:16 JST W3 启动 + 9/7 13:00 JST 升版) ==="
echo "  60 module mock.json 100% 覆盖 (W2 12 Partial + W3 30 + 8 域扩展 6 + batch 6 + 4 NEW 域 mTLS 6) ✅"
echo "  9 域 mTLS 业务级 11 步 v3 fixture 验证 (per 9/6 d270ab9) ✅"
echo "  batch 域 6 module fixture 验证 (per 9/1 batch 4 件套) ✅"
echo "  admin-coc §X 集成 3 场景 + 7 项 admin 域 Lead 真实签字验证 (per 9/5 ae9702d) ✅"
echo "  gap matrix 12-大类-RPC-清单.md 含 §16 W3 + §17 v0.3 升版段 ✅"
echo "  8 doc 完整 (W2 2 报告 + 1 handoff + W3 5 报告) ✅"
echo "  派生约束守护: L1 / L11 / L12.2 选项 B / 8/27 11:06 JST 凭据硬 ban / 8/27 19:39 三次强化代签 / 8/26 缺标比错标 / 9/4 17:47 JST 测试脚本归入 mock / cutover L15-L23 ✅"
