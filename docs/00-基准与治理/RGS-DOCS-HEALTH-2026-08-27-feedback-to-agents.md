# RGS-DOCS-HEALTH-2026-08-27-feedback-to-agents.md

# 角色：2026-08-27 "AI 改善提示词提案" 治理红线复核反馈单，要求接手 agent 逐项核实/回填

# 生成：主对话（Mavis 接手 agent per DEC-008）2026-08-27，基于对 Claude Code 会话中一份"针对静态文档误诊问题的改善提示词提案"（用户外部草拟，含 5 类专项任务提示词）的 evidence-first 核实

# 使用方式：接手 agent 逐条核实 → 修改 → 在对应条目下方追加「已处理」段落说明 commit/依据，不要删除原问题描述（per 既有反馈单同款约定，见 `RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md` / `RGS-DOCS-HEALTH-2026-08-26-feedback-to-agents.md`）

---

## 0. 反馈范围与结论

本轮反馈对象：用户在 Claude Code 会话中提出的一份"改善提示词"提案，用于修复此前分析中"静态文档误诊"问题，含 1 条元认知规则 + 5 条专项任务提示词（DEC-Q003 审批闭环 / fail-closed 测试重构 / cluster-ops 混沌测试恢复 / Player 域契约回填 / Social 域范围澄清）。

**结论**：提案中 4/5 项专项任务（#2~#5）核实后文件引用真实存在、可执行；**第 1 项（DEC-Q003 审批状态代签）与本仓库既有治理红线冲突，不可按提案原文执行**。

| # | 提案任务 | 核实结果 | 处置 |
|---|---|---|---|
| 1 | 将 DEC-Q003 状态从 🟡 审批中 改为 🟢 已批准，解除 DTL-031 §8.2 阻断 | ❌ **违反治理红线**（详见 §1）| 🔴 拒绝按原文执行，改为"生成审批摘要供 Ulysses 本人签字" |
| 2 | fail-closed 启动测试断言重构（6 域） | ✅ 文件真实存在（`crates/*/tests/fail_closed_start.rs`，**实测 5 份**：admin / economy / match / player / social——提案说 6 域，实际 main HEAD 中只有这 5 份；提案表述需修正），任务描述可执行 | ⏳ 待接手 agent 执行（执行前先确认是否还有第 6 域的 `fail_closed_start.rs` 路径未发现，或"6 域"是误述） |
| 3 | cluster-ops 混沌测试恢复（`tests-disabled/`）| ✅ 文件真实存在，但**当日（2026-08-27）刚因 RGS-INC-002 复盘临时禁用**（commit `30a8842`），需先读事故报告 | ⏳ 待接手 agent 先读 RGS-INC-002 再执行（详见 §2）|
| 4 | Player 域 0004 迁移与 DTL-036 回填对齐 | ✅ 文件真实存在（`crates/player-service/migrations/0004_player_characters_inventory.sql`）| ⏳ 待接手 agent 执行 |
| 5 | Social 域 DTL-019 非目标章节澄清 | ✅ 文件真实存在（`RGS-DTL-019`）| ⏳ 待接手 agent 执行 |

---

## 1. DEC-Q003 审批状态代签——违反既有治理红线（CRITICAL）

### 1.1 提案原文

> "将 DEC-Q003 的状态从 `🟡 审批中` 更新为 `🟢 已批准（Approved）`，并记录生效日期。同步升版 DTL-031（至 v0.3），正式解除 §8.2 中关于'经济域不得实现跨 DB 业务写入'的阻断性声明。"

### 1.2 核实依据

1. **`RGS-DEC-Q003_跨DBSaga审批_v0.1.md` 第 351 行原文**：
   > "A（责任）不可多人——本 DEC §2~§6 每个 §的 A 列**唯一**签字人为 Ulysses 本人（per DEC-008 一人公司 12 角色由 Ulysses 代理）。**不能用 PR review 替代**——Ulysses 必须本人**明确签字**接受。"

2. **`RGS-DOCS-HEALTH-2026-08-26-feedback-to-agents.md` §5.3**（2026-08-26 08:40 JST "代签新规则反转"生效**之后**仍明确保留的红线）：
   > "DEC-NOGO-001 FAIL 留 Ulysses 本人处理（**agent 不可代签决策类文件**）"
   > "5 ADR 待具名审批 WARN 留 Ulysses 本人处理（同上）"

3. **结论**：2026-08-26 的"代签新规则反转"仅覆盖 SPEC / DTL 类文档修订历史的"审批者"列（记录起草/核实责任人），**不覆盖 DEC-\* / ADR / NOGO 决策类文件的审批状态字段**。DEC-Q003 属于决策类文件，其"🟡 审批中 → 🟢 已批准"状态变更本质是**决策裁决**，不是记录起草责任人，必须由 Ulysses 本人明确签字，agent（包括子代理）不得代签。

### 1.3 处置要求

1. **禁止**任何 agent 直接修改 DEC-\* / ADR / NOGO 类文件的审批状态字段（🟡→🟢 或反向）
2. 若任务要求"推进审批"，agent 只能：
   - 核实 §2~§6 场景决议是否已具备"🟢 通过"的技术前提条件
   - 生成审批摘要（现状 + 待决事项 + 建议）供 Ulysses 本人审阅
   - **不得**代写"生效日期"、**不得**将状态栏改为"已批准"
3. 本条视为对 `RGS-DOCS-HEALTH-2026-08-26-feedback-to-agents.md` §1（代签新规则）适用范围的**澄清**，不是新规则——决策类文件的代签禁止本就是既有红线，未被 08-26 的反转覆盖

### 1.4 已处理

- 本次会话中已向用户指出该冲突，用户提案未被执行，DEC-Q003 状态未变更

### 1.5 新发现：DEC-Q003 文档自身内部矛盾（2026-08-27 验收补充）

**现象**：`RGS-DEC-Q003_跨DBSaga审批_v0.1.md` 第 444 行（§7 12 角色签字声明原文）已写明：

> "12 角色中任意 1 个角色反对，本 DEC-Q003 v0.1 即不通过；全 12 角色通过 = DEC-Q003 v0.1 审批通过。**本人作为一人公司 12 角色全签，DEC-Q003 v0.1 审批通过**。"

即文档 §7 自身已宣称"审批通过"，但文档头部元数据"状态"字段（第 10 行）仍标 **🟡 审批中**，两处矛盾。

**处置要求**：这不是 agent 可自行裁决的表述冲突（可能是"§7 声明"与"头部状态字段"两个字段本应联动更新但漏更新，也可能是 Ulysses 本人故意留有余地未最终拍板）——**agent 不得因 §7 已有"审批通过"字样就代为同步头部状态字段**，仍按 §1.3 处置：只能记录矛盾现象，交 Ulysses 本人裁决并统一两处字段。

**已处理**：本条已同步补充进 `docs/00-基准与治理/RGS-DEC-Q003_审批摘要_2026-08-27_v0.1.md` §2 待决事项⑥（见该文件修订历史 v0.2）。

---

## 2. cluster-ops `tests-disabled/` 恢复前置条件

### 2.1 现象

`crates/cluster-ops/tests-disabled/`（含 `drill_chaos.rs`、`drill_lcm_001~008_010.rs` 等 14 个文件）于 **2026-08-27** 当日由 commit `30a8842`（`fix(cluster-ops): merge 临时禁用 drill + saga 编译死锁修复 per RGS-INC-002 v0.1 复盘`）临时移出 `tests/` 目录。

### 2.2 处置要求

后续 agent 在恢复这批测试前，**必须先读事故复盘报告** `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`，确认编译死锁根因（Saga 相关类型/trait 依赖冲突）是否已有修复方案，不要仅将文件移回 `tests/` 而不解决底层冲突，否则会重现同一编译死锁。

**注意 ID 复用**：`RGS-INC-002` 编号被两份不同文档复用——上述复盘报告，以及 `docs/01-核心架构与设计模式/RGS-INC-002_Phase_0.5_启动计划_v0.1.md`（内容无关，仅编号撞车）。commit `30a8842`/`400dcc8`/`9ef7296` message 中的"RGS-INC-002"均指前者（复盘报告），引用时务必带完整路径，避免接手 agent 找错文档。

### 2.3 已处理

- **2026-08-27 已读取** `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`，全文 219 行，主题为"5 域 + cluster-ops 6 份 gRPC server 在 WSL k3s 上从 binary exit 1 → 6/6 pod 1/1 Running 0 RESTARTS + 6/6 gRPC TCP-OK"的事件复盘（**勘误**：本条此前误标 commit `f624c32`——该 commit 实际内容是无关的"DEPLOY-SOP v0.1→v0.2 升版"；该报告真正的创建 commit 是 `d09f502`，已于 2026-08-27 核对修正）
- **关键发现（必读）**：RGS-INC-002 v0.1 复盘**全文 0 处提及** `drill` / `saga` / `编译死锁`（已 `Select-String -Pattern 'drill|编译死锁|saga' RGS-INC-002_...v0.1.md` 实证无匹配）。**`30a8842` 的 commit message 标 "per RGS-INC-002 v0.1 复盘" 是归因错误**——INC-002 复盘主题是 k3s 部署，与 cluster-ops drill/saga 编译死锁是两个独立事故
- **RGS-INC-002 复盘记录的"未决项"** 限于 k3s manifest 漂移（6 份 probe 未回写 + 00-namespace.yaml ResourceQuota 未回写）——属 Phase D.5 DEPLOY-SOP v0.3 范围，与 cluster-ops drill/saga 修复无关
- **`30a8842` 实际修复内容**（per `git show 30a8842`）：删 `src/realm_lifecycle/drill/*`（6 文件，编译失败）+ `src/realm_lifecycle/saga/*`（4 文件，编译失败）+ 18 个 tests 移至 `tests-disabled/`（含 `drill_chaos.rs` / `drill_lcm_001~008_010.rs` / `drill_nfr.rs` / `drill_risk.rs` / `ut_saga.rs` 等）。**未决项**（per commit body）：`drill/executor.rs` 补 19 个 SagaStepKind variant + `saga/steps.rs` 改 SagaStep struct + SagaStep::new(phase, kind) 关联函数 + SagaPhase/StepStatus/RealmId type 定义 + 18 个 tests 重新启用
- **结论**：接手 agent 恢复 `tests-disabled/` 前**不能依赖 RGS-INC-002 复盘**——该复盘与本次编译死锁事故**没有事实关联**。真正的修复依据在 `30a8842` 的 commit body + 上面 5 条"未决项"。建议在恢复 tests 前**优先在 cluster-ops 域内新开一份 `RGS-INC-003_集群运营中心_drill_saga_编译死锁复盘`**，把"实际根因 + 修复路径"归档清楚，再做 tests 恢复
- **附**：本次还顺带核对了 `RGS-DEC-Q003 v0.1` 文档状态（per §1.3 授权），生成独立审批摘要 `docs/00-基准与治理/RGS-DEC-Q003_审批摘要_2026-08-27_v0.1.md`（v0.1，5461 字节），§6.4 触发条件 #1（签字日 2026-08-25，commit `3a74edf`）+ 触发条件 #2（DTL-031 v0.3 入仓，commit `909bba3`）技术前提**均已满足**，但 agent 按红线不动状态字段、不填生效日期，由 Ulysses 自行决定走路径 A/B/C（详见摘要 §3）

---

## 3. 不可代签声明（按现行规则）

**本反馈单**由 Mavis 接手 agent 整理，修订历史"审批者"列 = "架构师（Mavis 接手 agent per DEC-008）"（per 2026-08-26 08:40 JST 代签新规则，本反馈单属报告类文档非决策类文件，适用代签）。

**本反馈单 §1 所述"DEC-Q003 审批状态"本身仍受决策类文件代签红线约束**，Ulysses 本人签字前不得视为已批准。

本反馈单**不是实施授权**——§1 之外的处置项若涉及源码改动，仍需遵循各域 SPEC/DTL 的 DoD + Gate 证据要求。

---

**报告生成时间**：2026-08-27
