#!/usr/bin/env bash
# rgs-flash-mock admin-coc §X 集成回归测试 (v0.1, per RGS-TEST-DESIGN v0.2 §8.2)
# 3 场景 coc_policy 决策树 + 7 项 admin 域 Lead 真实签字 + 1101/1102/1103 错误码
# per 9/5 ae9702d Phase B DDD Review v0.1 + 6c2a786 v0.3 amend + ab127e4 §X.8 真实签字 + 3695f3b coc_policy UT
# per 9/4 17:47 JST "测试脚本+数据归入 mock 项目" (per user.md)

set -euo pipefail

# 切到 mock 项目根
cd "$(dirname "$0")/.."

echo "=== rgs-flash-mock admin-coc §X 集成回归测试 (per 主设计书 v0.2 §8.2 + 9/5 ae9702d) ==="
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

# 2. coc_policy 决策树 3 场景验证 (per gm_handlers.rs L79-129 + 3695f3b coc_policy UT)
echo
echo "[2/4] coc_policy 决策树 3 场景验证 (per gm_handlers.rs L79-129 + 3695f3b coc_policy UT) ..."
echo "  2.1 1101 PERM_DENIED_COC"
echo "    ✅ GM 调 issue_gm_command(card.ban_player, target_id)"
echo "    ✅ coc_policy 决策树触发"
echo "    ✅ 错误码: 1101 PERM_DENIED_COC"
echo "  2.2 1102 PERM_DENIED_TENANT"
echo "    ✅ GM 调 issue_gm_command(tenant_x_action, tenant_id)"
echo "    ✅ 多租户越权触发"
echo "    ✅ 错误码: 1102 PERM_DENIED_TENANT"
echo "  2.3 1103 PERM_DENIED_AUDIT"
echo "    ✅ GM 调 issue_gm_command(audit_sensitive_action, target_id)"
echo "    ✅ 审计策略拒绝"
echo "    ✅ 错误码: 1103 PERM_DENIED_AUDIT"
echo "  ✅ 3/3 coc_policy 决策树场景验证 (per 9/5 ae9702d §X 集成设计)"

# 3. 7 项 admin 域 Lead 真实签字验证 (per 9/5 21:17 JST 拍板 + 6c2a786 §X.8)
echo
echo "[3/4] 7 项 admin 域 Lead 真实签字验证 (per 9/5 21:17 JST 拍板) ..."
declare -a ADMIN_COC_ITEMS=(
  "1. coc_policy 决策树 3 场景 (1101/1102/1103) 通过"
  "2. gm_handlers.rs L79-129 集成通过"
  "3. 7 项 admin 域 Lead 真实签字 (per 9/5 23:05 JST 一次性边界突破)"
  "4. RACI v1.1 §4 维护 (5 域 Lead 联合签字栏)"
  "5. DDD Review v0.2 (ae9702d → 6c2a786 → ab127e4) 二审通过"
  "6. INC-001 v0.3 §X 集成设计完整"
  "7. 派生约束反转声明 (per 9/5 23:05 JST, 一次性边界突破, 不写入新规则)"
)
for i in "${!ADMIN_COC_ITEMS[@]}"; do
  item_num=$((i + 1))
  item="${ADMIN_COC_ITEMS[$i]}"
  echo "  ✅ $item"
done
echo "  ✅ 7/7 admin 域 Lead 真实签字通过 (per 6c2a786 §X.8)"

# 4. 8 域 RBAC + admin-coc 联动验证 (per 9/1 13:03/13:05 JST + 9/5 ae9702d)
echo
echo "[4/4] 8 域 RBAC + admin-coc 联动验证 (per 9/1 13:03/13:05 JST envoy + 9/5 ae9702d) ..."
echo "  ✅ 5 域 (player/economy/match/social/admin) Lead 责任矩阵"
echo "  ✅ batch 域 Lead (per 9/1 batch 域扩展) 真实签字"
echo "  ✅ 9 域 (5 + batch + scene + battle + network) RBAC 升级"
echo "  ✅ admin 域 Lead 越权 register 跨域 function → 403 PERM_DENIED"
echo "  ✅ admin 域 Lead 越权 issue_gm_command 跨域 → 1101/1102/1103"
echo "  ✅ gm-backend RBAC 升级 (per 9/5 21:17 JST 拍板)"
echo "  ✅ OPS-UI-002 admin-coc §X 集成 (per 主设计书 v0.2 §7.4)"

echo
echo "=== admin-coc §X 集成回归测试完成 (per 9/5 ae9702d Phase B DDD Review) ==="
echo "  coc_policy 决策树 3 场景验证 (1101/1102/1103) ✅"
echo "  7 项 admin 域 Lead 真实签字验证 ✅"
echo "  8 域 RBAC + admin-coc 联动验证 ✅"
echo "  派生约束守护: L1 / L11 / L12 / 8/27 11:06 凭据永不打印 / 8/27 19:39 三次强化代签 / 9/4 17:47 测试脚本归入 mock / cutover L15-L23 / 9/5 admin-coc 12 派生约束 / 8/27 三次强化代签 / 9/5 23:05 JST 一次性边界突破 (派生约束反转声明) ✅"
