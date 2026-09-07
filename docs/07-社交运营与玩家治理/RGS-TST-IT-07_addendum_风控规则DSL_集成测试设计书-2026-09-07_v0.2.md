# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 07 社交运营与玩家治理 — 风控规则 DSL（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-07-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-028-ADD1 + RGS-DTL-025（v0.3，DSL 增补） |
| 升版基线 | v0.1 (2026-08-19) → v0.2 (2026-09-07, 8 维度增量) |
| V模型层级 | TL-2 集成 / TL-3 契约 |
| 制定日 | 2026-08-19 |
| 升版日 | 2026-09-07 |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定 |
| **0.2** | 2026-09-07 | 架构师（Mavis 接手代签 per DEC-008） | 8 维度增量升版：① 8 域扩展（per 9/6 闪烁之光兼容）② admin-coc §X 引用（per 9/5 ae9702d/6c2a786/ab127e4/3695f3b）③ 风控 + coc_policy 决策树（per gm_handlers.rs L79-129）④ 派生约束守护 L15-L23（per 9/6 6c6839e cutover） |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 升版（v0.2 自审） | 架构师（Mavis 接手代签 per DEC-008） | 2026-09-07 | 8 维度增量；B3 二审待 |
| 评审（架构） |  |  | 待 Ulysses 二审 |

---

## 0. v0.1 → v0.2 升版范围（8 维度增量，主题特定子集）

| # | 维度 | v0.1 现状 (8/19) | v0.2 增量 (9/7) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §2 R001~R014 仅覆盖 social 域 DSL 推送 | §2 增 R015~R018：5 NEW 域（scene/battle/network/account/sub8）+ batch 域 DSL 推送 + coc_policy 联动 | a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673 |
| 2 | **admin-coc §X 引用** | §3 追溯性未引用 admin-coc | §3 增 admin-coc §X 追溯行：coc_policy 决策树 + gm_handlers.rs L79-129 联动 | ae9702d/6c2a786/ab127e4/3695f3b |
| 3 | **风控 + coc_policy 决策树** | §2 缺 coc_policy 决策树 | §2 增 R019~R021：coc_policy 决策树 3 场景（per gm_handlers.rs L79-129）+ DSL + coc 联动 | gm_handlers.rs L79-129 / RGS-DDD-2026-09-05-PHASE-B v0.2 |
| 4 | **派生约束守护 L15-L23** | §4 通过判定未含派生约束 | §4 增 §派生约束守护段，列出 L15-L23 落地状态 | 6c6839e / add4238 |

**已知缺口（per 缺标比错标安全, per 8/26 JST）**:
- 5 NEW 域（scene/battle/network/account/sub8）真实 DSL 推送 + coc 联调需 ST Phase C 验证
- coc_policy 决策树 3 场景覆盖（per gm_handlers.rs L79-129）需 admin 域 Lead 二审
- DSL 灰度 5s 切流 + NATS 延迟/乱序 fence_epoch 拒旧版 待 PH-7 验证

---

## 1. 目的

验证 DSL 引擎与仓库推送 + 灰度路由的集成。v0.2 新增 8 域 DSL 推送 + admin-coc §X 引用 + coc_policy 决策树 3 场景联动。

## 2. 测试用例

| 用例 ID | 试验级别 | 测试目的 | シナリオ | テストデータ |
|---|---|---|---|---|
| TST-IT-07-R001 | [TL-2] | 仓库推送 → 5s 全节点生效（AC-ANT-101） | — | — |
| TST-IT-07-R002 | [TL-2] | 灰度：canary 推送，stable 节点不加载 | — | — |
| TST-IT-07-R003 | [TL-2] | 回滚：5s 全节点回退（AC-ANT-103） | — | — |
| TST-IT-07-R004 | [TL-2] | 行为采集 → DSL 决策 → 案件形成 | — | — |
| TST-IT-07-R005 | [TL-2] | 规则命中次数埋点 | — | — |
| TST-IT-07-R006 | [TL-2] | 沙箱与 Rhai 引擎集成 | — | — |
| TST-IT-07-R007 | [TL-3] | DSL 仓库 HTTP API 契约 | — | — |
| TST-IT-07-R008 | [TL-2] | 规则版本单调递增 | — | — |
| TST-IT-07-R009 | [TL-2] | 跨服务规则调用（FR-ANT-006 集成） | — | — |
| TST-IT-07-R010 | [TL-2] | ARC-043-5 与 CDN channel 共用 | — | — |
| TST-IT-07-R011 | [TL-3] | DTL-025 v0.3 §6.2：签名 manifest、固定对象版本和 hash 校验后才可加载 | — | — |
| TST-IT-07-R012 | [TL-2] | DTL-025 v0.3 §6.4：NATS 延迟/乱序时以清单和 `fence_epoch` 拒绝旧版本 | — | — |
| TST-IT-07-R013 | [TL-7] | DTL-025 v0.3 §6.5：节点与规则清单分区后租约失效即 fail-closed，恢复验签后才执行 | — | — |
| TST-IT-07-R014 | [TL-2] | DTL-025 v0.3 §6.6：回滚的目标版本、审批、节点确认和未确认节点审计完整 | — | — |
| **TST-IT-07-R015** | **[TL-2]** | **8 域扩展 DSL 推送 (per 9/6 闪烁之光兼容)** | **scene 域 DSL 推送** | scene DSL 5 规则 |
| **TST-IT-07-R016** | **[TL-2]** | **8 域扩展 DSL 推送 (per 9/6)** | **battle 域 DSL 推送** | battle DSL 5 规则 |
| **TST-IT-07-R017** | **[TL-2]** | **8 域扩展 DSL 推送 (per 9/6)** | **network + account + sub8 DSL 推送** | network/account/sub8 DSL 15 规则 |
| **TST-IT-07-R018** | **[TL-2]** | **batch 域 DSL 推送 (per 9/1 batch v0.1)** | **batch 域 DSL 推送 + coc 联动** | batch DSL 5 规则 |
| **TST-IT-07-R019** | **[TL-2]** | **coc_policy 决策树 场景 1 (per gm_handlers.rs L79-129)** | **低风险：自动放行** | 1 场景 coc_policy |
| **TST-IT-07-R020** | **[TL-2]** | **coc_policy 决策树 场景 2 (per gm_handlers.rs L79-129)** | **中风险：人工审核** | 1 场景 coc_policy |
| **TST-IT-07-R021** | **[TL-2]** | **coc_policy 决策树 场景 3 (per gm_handlers.rs L79-129)** | **高风险：自动拒绝 + 审计** | 1 场景 coc_policy |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-ANT-005~012 | TST-IT-07-R001~R009 |
| ARC-043-5 | TST-IT-07-R010 |
| AC-ANT-101/103 | TST-IT-07-R001/R003 |
| DTL-025 v0.3 §6.2~§6.6 | TST-IT-07-R011~R014 |
| 9/6 8 域扩展 | TST-IT-07-R015~R017 |
| 9/1 batch v0.1 | TST-IT-07-R018 |
| 9/5 admin-coc Phase B | TST-IT-07-R019~R021 |

## 4. 通过判定

- 全部 PASS（含 v0.1 14 条 + v0.2 7 条 = 21 条）
- 推送/回滚 ≤ 5s
- 灰度隔离 100%
- 8 域 DSL 推送全节点生效
- coc_policy 决策树 3 场景覆盖
- 5 NEW 域（scene/battle/network/account/sub8）+ batch 域 DSL 灰度 100% 隔离

## 5. 派生约束守护

| 派生约束 | 状态 | 引用 |
|---|---|---|
| L1 (cargo check 60s) | ✅ N/A (doc) | AGENTS.md §2.1 |
| L11 (cargo build dir lock) | ✅ N/A (doc) | AGENTS.md §2.2 |
| L12 (临时 log 不入 commit) | ✅ (git status 0 untracked) | AGENTS.md §2.2 |
| L13 (8 维度 git show --stat 实证) | ✅ (本节 §0 引用全部 commit 实证) | AGENTS.md §2.2 |
| L15 (native binary 跨工具链) | ⏳ (待 ST Phase C) | 6c6839e |
| L19 (mTLS 业务级 = saga 触达) | ✅ (DSL 推送走 mTLS 通道) | 6c6839e |
| L21 (跨工具链 gRPC Code 解析) | ✅ (coc_policy gRPC Code 解析) | 6c6839e |
| L23 (4 层自动探针) | ✅ (DSL 推送 4 层探针) | 6c6839e |
| B3 (DDD Review 二审) | ⏳ (Mavis 自审 1 次停手, Ulysses 二审待) | AGENTS.md §3.x |

---

> 与 RGS-TST-IT-07 共存（v0.2 升版）。
