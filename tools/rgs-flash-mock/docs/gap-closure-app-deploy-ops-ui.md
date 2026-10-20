# app 独立更新 + ops UI 用例闭环 (GAP-001)

> 缺口来源: `tools/rgs-flash-mock/README.md:108` (app 独立更新 缺 4 用例) + `tools/rgs-flash-mock/README.md:109` (ops UI 缺 6 用例)
> 用例真源: `docs/15-IPA-完全对齐438cmds/RGS-TEST-CASES-2026-09-07_v0.2.md` §3.3 (L317-325) / §3.4 (L327-338)
> 判定条件: `docs/15-IPA-完全对齐438cmds/RGS-TEST-DESIGN-2026-09-07_v0.2.md` §7.3 (L290-312) / §7.4 (L315-359)
> 归属: `mock_data/app_deploy.json` + `mock_data/ops_ui.json` + `scripts/regression-test-app-deploy-ops-ui.sh`
> 代签: Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化)

## 1. 补了什么

| scenario | 用例标题 | 判定条件来源 |
|---|---|---|
| APP-DEPLOY-001 | player-service v0.2 灰度发布 10% → 100% | 主设计书 §7.3 L293-305 原文 |
| APP-DEPLOY-002 | battle-service v0.2 灰度发布 (8 域扩展 NEW) | 主设计书 §7.3 L307-312 原文 |
| APP-DEPLOY-003 | economy-service 跨域回滚 | TEST-CASES §3.3 L323 标题级, 判定字段本 lane 细化 |
| APP-DEPLOY-004 | k3s rolling update 不中断 function-plane | TEST-CASES §3.3 L324 标题级, 判定字段本 lane 细化 |
| OPS-UI-001 | gm-backend /ops/functions/register RBAC 拦截 | 主设计书 §7.4 L318-326 原文 |
| OPS-UI-002 | admin-coc §X GM 命令 coc_policy 决策树 | 主设计书 §7.4 L328-336 原文 |
| OPS-UI-003 | ops UI env var 永不打印 | 主设计书 §7.4 L338-343 原文 |
| OPS-UI-004 | ops UI REDACTED filter 触发 | 主设计书 §7.4 L345-350 原文 |
| OPS-UI-005 | 各域 Lead 越权 register 跨域 function → 403 | 主设计书 §7.4 L352-359 原文 |
| OPS-UI-006 | rgs-batch-backend /ops/batch/cron RBAC 拦截 | TEST-CASES §3.4 L336-337 标题级, 判定字段本 lane 细化 |

每个 scenario 内 `expected.expected_source` 字段自标该行判定条件是「设计书原文」还是「本 lane 细化」。

## 2. 原文 vs 细化的边界 (缺标比错标安全)

- **原文**: 001/002 (app)、OPS-UI-001~005 的域归属、步骤要点、判定数值 (Pod 数 4 / 灰度 1→2→4 / latency p99 < 100ms / error rate < 0.5% / 3 场景错误码 1101 PERM_DENIED_COC、1102 PERM_DENIED_TENANT、1103 PERM_DENIED_AUDIT / 7 项决策全通过 / 跨域 register 全 403)。
- **本 lane 细化**: APP-DEPLOY-003 / APP-DEPLOY-004 / OPS-UI-006 —— 这 3 个用例在 TEST-CASES 里只有标题, 主设计书 §7.3/§7.4 未列, 因此其判定字段由本 lane 拟定, **待评审确认**。
- README 缺口行写的「per 主设计书 v0.2 §7.4」: §7.3 对应 app 独立更新、§7.4 对应 ops 运维 UI (L290 / L315), 用例本体定义在 TEST-CASES 文档 §3.3/§3.4。

## 3. 怎么验证

```bash
cd tools/rgs-flash-mock
bash scripts/regression-test-app-deploy-ops-ui.sh
```

脚本是**真实断言** (不是 echo 式假绿): 校验 10 个 scenario 齐全且 id 全局唯一、每个 scenario 有 name/expected/assertions、逐条执行 fixture 内嵌 `assertions` (field 路径取值比对 equals)、对 OPS-UI-003/004 额外扫描明文 env/secret 值。

实测输出: 断言总数 56 / scenario 数 10 (期望 10) / 退出码 0。

## 4. 可证伪实测 (条件不满足必须红)

| 破坏方式 | 期望 | 实测 |
|---|---|---|
| `app_deploy.json` 的 `expected.pods_total` 4 → 99 | exit 1 | exit 1 |
| `ops_ui.json` 的 `DB_PASSWORD` 值改成明文 | exit 1 | exit 1 |
| 恢复原值 | exit 0 | exit 0 |

## 5. 剩余缺口 (已知, 本 lane 不覆盖)

- APP-DEPLOY-002/004 需要真实 k3s 集群 + 4 Pod 灰度; 本地无集群, fixture 只固化判定口径。
- OPS-UI-001/005/006 的 403 需要真实 gm-backend RBAC; 本地不可验证。
- README.md:108 / README.md:109 的缺口行需在主会话合并后统一更新 (本 lane 不改 README)。
