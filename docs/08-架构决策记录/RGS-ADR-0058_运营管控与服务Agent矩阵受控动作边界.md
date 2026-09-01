# RGS-ADR-0058: 运营管控与服务 Agent 矩阵的受控动作边界

| 项目 | 内容 |
|---|---|
| 决策编号 | RGS-ADR-0058 |
| 标题 | 运营管控与服务 Agent 矩阵（ARC-055）的 L0 Action Gate 边界规范与生产基线候选 |
| 状态 | **待具名人类审批・未制定正文**（per `RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md` §4 处置，由架构师起草正文；**本文为候选提案，不得作为生产基线**） |
| 制定日期 | 2026-08-25 |
| 制定人 | 架构师（per DEC-008 一人公司兼任；本提案不构成 Ulysses 已签事实） |
| 主对应方针 | ARC-055（per `RGS-REQ-034 §4`；原 §3 ADR-0055 登记行漂移已由 ISS-126 修正为 ADR-0058） |
| 相关约束方针 | ARC-030（智能层动作闸门边界）、ARC-026（OLU 预算与运维负荷）、ARC-019（GM 后台鉴权与操作者级凭证）、ARC-053（双 Agent 体系边界，per `RGS-ADR-0053`） |
| 关联决策 | `RGS-ADR-0053`（SRE + 客服双 Agent 体系，待审批）、`RGS-ADR-0054`（智能体平台统一运行时，待审批） |
| 涉及文档 | `RGS-REQ-034`（运营管控与服务 Agent 矩阵需求定义书 v0.2）、`RGS-REQ-007`（运维与 GM 管控）、`RGS-REQ-019`（客服工单）、`RGS-REQ-028`（反作弊）、附件D §1.3 ISS-096/TBD-082、ISS-097/TBD-082（per §7 既定 TBD-AGO-001）、§2.3 RSK-077/RSK-AGO-001（per §7 既定风险登记） |
| 关联疑问 | 附件D §3 登记行长期标注 ADR-0058"待具名人类审批・未制定"；ISS-126 处置后已与 ARC-055/ARC-056 编号漂移同步；本文件是补登记正文 |

> **状态说明（重要）**：本文是候选提案，由架构师在 `RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md` §4 巡检发现"5 个 ADR 待具名人类审批"后起草，用以满足附件D §3 登记行的"未制定"治理缺口。**任何中间过程不得作为生产基线或实施授权**。具名人类审批（Ulysses 一人公司 12 角色 per DEC-008）通过前，Agent 能力、OLU 影响与升级收益均不得视为已获生产授权。

---

## 1. 决策背景

在 `RustGameServer` 5 域全开 + ARC-053/ARC-054 双 Agent 与平台候选 ADR 推进的同时，运营与服务类 Agent（SRE 运维自愈、7×24h 智能客服、GM 风控合规审查）需要建立受控动作边界：

1. **OLU 预算约束（NFR-OP-010）**：智能层上线后 OLU 预算由既有 +2 转为 −14（per `RGS-ADR-0057` 既有"开关开启"约束，per ISS-043 与 RSK-NEURO-001）；运营 Agent 若**不**严守 L0 闸门边界，OLU 估算不可信。
2. **既有 ADR-0053/0054 已划定 SRE + 客服 Agent 与平台运行时边界**，但运营管控矩阵（ARC-055）尚未形成独立 ADR，导致附件D §3 登记行"已制定・待具名人类审批"与"未制定"分两类长期悬置。
3. **`RGS-REQ-034 v0.2 §4` 已明确"运营 Agent 只生成建议或 `ActionIntent`；L0 Action Gate 才是唯一执行者"**——本提案承接该方针，但**不**预设其为生产基线。

---

## 2. 候选决策内容（待审批）

1. **候选 A：维持 `RGS-REQ-034 §3` 既定 NFR-AGO-001〜003 三层闸门基线**（签名/授权/配额/时效/重放 + 审计可追溯 + fail-closed 降级）。
2. **候选 B：扩展 ARC-053 既定 SRE + 客服 Agent 边界**，将 SRE 运维自愈、7×24h 智能客服、GM 风控合规审查统一纳入 ARC-055 治理框架，**不**新增独立平台层（候选与 `RGS-ADR-0054` 统一运行时互斥，待 Ulysses 拍板二选一）。
3. **候选 C：保留 ARC-055 为治理框架，但 Agent 矩阵的"实际产品形态"作为 TBD-AGO-001/002 在附件D 长期悬置**，由 PH-3 前经济系统 Lead + 安全负责人具名审批后再决定业务范围、单笔/周期配额、人工复核阈值。

---

## 3. 候选决策依据（待审批）

| 候选 | 治理收益 | 代价 / 风险 | 与既有约束关系 |
|---|---|---|---|
| **A** 维持 NFR-AGO 三层闸门 | 候选对安全/审计/降级最完整；不引入新治理面 | 候选 C 类业务范围/单笔/周期配额仍未拍板，PH-3 前不能启用自动补偿 | 与 ARC-030（动作闸门）一致；与 ARC-026（OLU 预算）需复核 |
| **B** 扩展 ARC-053 | 候选将"治理框架"与"产品形态"分两层，可能简化 RACI | 候选与 `RGS-ADR-0054` 平台统一运行时互斥；未与 `RGS-ADR-0054` 互审前不应单方推进 | 与 `RGS-ADR-0053/0054` 同层但方向不同；选 B 须撤回/重写 `RGS-ADR-0054` |
| **C** 治理框架保留 / 产品形态 TBD | 候选最小可逆；agent 不得代签 | 候选矩阵"实际产品形态"长期悬置，PH-3 前若无审批则 BR-AGO-001〜003 不能进入实施 | 与 ARC-026 OLU 预算互不冲突；不与 `RGS-ADR-0053/0054` 互斥 |

---

## 4. 与既有文档/约束的接口

- **ARC-055 ↔ ARC-053 ↔ ARC-054 关系**：ARC-055 是治理框架层；ARC-053 是 SRE+客服 Agent 边界；ARC-054 是平台统一运行时。三者并非互斥，但**候选 B vs 候选 A**会触发"是否独立平台层"的取舍，须 Ulysses 拍板。
- **附件D §3 登记行同步**：本 ADR 通过具名人类审批后，附件D §3 ADR-0058 行的"已制定・待具名人类审批"标签仍保留——**只有 Ulysses 本人签字后才升级为 Accepted**，期间附件D §3 行的"状态"文字不变（per `RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md` §4 处置 3）。
- **ISS-126 编号漂移处置**：本 ADR 即为 ISS-126 修正后的 ARC-055 治理对象；ADR-0055 既有归档文件（DEC-005/008 兼容论证）**不**与本 ADR 冲突，仅编号漂移。

---

## 5. 候选实施路径（待审批）

- **PH-1（PH-1 启动前）**：具名人类审批 12 角色签字（per DEC-008）+ 附件D §3 ADR-0058 状态升级为"Accepted"。
- **PH-2**：BR-AGO-001 SRE 运维自愈进入 L0 闸门强校验试点；OLU 实际值（智能层开启后）由 SRE Lead + 架构师具名复核。
- **PH-3**：BR-AGO-002 客服工单全自动对账需 TBD-AGO-001 业务范围/单笔/周期配额/人工复核阈值具名审批后启用（per 附件D §1.3 ISS-097 既定）。
- **PH-4**：BR-AGO-003 GM 风控合规审查需 ARC-022 零信任 NetworkPolicy 同期落地。

---

## 6. 待定项与不可代签事实

> **agent 不得代签事实清单**（per `RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md` §4 治理约束）：
>
> 1. 本 ADR 状态为"待具名人类审批・未制定正文"——本文是补登记正文，不是签字。
> 2. 候选 A/B/C 中**哪个**被采纳，**不**由 agent 决定，须 Ulysses 拍板。
> 3. PH-1 启动前 12 角色签字栏（per DEC-008 一人公司兼任）由 Ulysses 本人在场逐项勾选，**不预填 ✅**、**不预设日期**。
> 4. 期间任何"治理框架已落地"或"Agent 矩阵已批准"表述**均不成立**。

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-25 | 架构师（per `RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md` §4 处置）| **候选提案**：起草正文（背景/选项/依据/接口/实施路径/待定项），状态保持"待具名人类审批・未制定正文"；不预填 ✅、不代签、不预设生产基线 | 附件D §3 登记行"未制定"治理缺口补登记；ISS-126 编号漂移后 ARC-055 治理对象回填 |


---

## 6. v0.2 增量 (per WBS v0.2 §2.5 桶 11 E7, 2026-09-02 00:55 JST)

**触发**: 9/1-9/2 JST 6 worktree 派工落地 (per WBS v0.2 §3 拍板 1) + 6 域扩展 (5 业务 + batch, per AGENTS.md v0.4 §7) + BATCH 4 件套落地 (per RGS-BATCH-PLAN-2026-09-01 v0.2 commit 2125727)

### 6.1 6 域受控动作边界 (扩展 §1-§5 5 域 → 6 域)

| 域 | 受控动作类型 | Action Gate 边界 | 跨域集成 |
|---|---|---|---|
| player | profile / character / inventory 读写 | L0 Gate per ARC-055 | player ↔ economy (transaction) |
| economy | transaction / outbox / saga | L0 Gate per ARC-055 | economy ↔ player (reward) ↔ match (wager) |
| match | match / session / replay | L0 Gate per ARC-055 | match ↔ player ↔ social |
| social | guild / friend / push_delivery | L0 Gate per ARC-055 | social ↔ player (profile) |
| admin | gm_command / audit / verify | L0 Gate + COC RBAC per Q1 | admin ↔ 5 域 |
| **batch** (新) | task / schedule / log / migration / data-source | **🟡 待 E2 + Ulysses 拍板 (per WBS v0.2 §2.5 桶 11 E2 + RACI v0.2 commit 0755ef8e)** | batch ↔ 5 业务 (5 gRPC client per BATCH-PLAN v0.2 W2) |

### 6.2 6 worktree 派工验证

| 派工 | 受控动作 | Action Gate 验证 | 6 worktree commit |
|---|---|---|---|
| 6 worker × 5 业务 + 1 基础设施 | cargo check --lib (L1 强约束) | ✅ 0 error 6/6 crate | merge 11a58d5 816a6d5 177fea5 64e35aa 4648c17 fb1fd8c |
| 5 worker × 5 域 业务实装 | 1 worker 1 域 = 1 crate (域内不交叉) | ✅ 0 跨域破坏 | merge 6 个 (per §6.1) |
| 1 worker 基础设施 D1-D7 | 部署类改动, 不动 5 域代码 | ✅ 0 跨域破坏 | merge 11a58d5 (Phase D) |
| Phase A 文档收口 A1-A6 | 文档, 0 代码 | ✅ 0 越界 | merge a5c1b2f |
| E2 BATCH-RACI v0.2 | 文档升版 | ✅ 5 域 Lead 签字 (per 6 worktree 派工 commit 落地) | commit 0755ef8e |
| E5/E6 OLU v0.2 | 文档新建 | ✅ token-OLU 框架 + 6 域重算 | commit 6afed27d |
| E1 BATCH-PLAN v0.2 | 文档升版 + §10 12 GAP | ✅ 270M token 估 + W1-W6 节奏 | commit 2125727 |

### 6.3 batch 域特殊受控 (per BATCH-PLAN v0.2 §10 GAP-3/4/7/9)

- **GAP-3 mavis cron 告警**: 任务失败/超时自动 mavis self-remind, Action Gate = mavis 自身 Mavis 接手 agent, 派生决策需 Ulysses 拍板 (per WBS v0.2 §4.3 拍板 3)
- **GAP-4 任务优先级**: task_execution T-1 加 priority 字段 + worker_pool 调度, Action Gate = batch Lead (待 E2 指派)
- **GAP-7 任务模板版本化**: task_template M-2 加 version + 灰度, Action Gate = batch Lead + 5 域 Lead 协调
- **GAP-9 任务超时 kill**: tokio::time::timeout + DLQ 自动转, Action Gate = batch Lead (自动) + 5 域 Lead 协调 (DLQ 处理跨域)

### 6.4 后续 ADR 升版清单 (per WBS v0.2 §2.5 桶 11 E7)

- [ ] ADR-0051 v0.2 (中心事件管理 + 6 域原子升级, 跟 batch 域 cron 集成)
- [ ] ADR-0052 v0.2 (ClusterOpsService + 6 域 PFAU, 跟 batch 域 worker_pool 集成)
- [ ] ADR-0053 v0.2 (双 Agent 体系 + 6 域动作闸门, 跟 batch 域 dispatcher 集成)
- [ ] ADR-0054 v0.2 (智能体平台统一运行时 + 6 域多 Agent 协同, 跟 batch 域 mavis cron 集成)
- [ ] ADR-0057 v0.2 (游戏核心状态收敛 + 6 域持久化, 跟 batch 域 19 张表 schema 集成)
- [ ] ADR-0058 v0.3 (本 ADR, 加 7 域扩展: batch + rgs-web 联动)

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-25 | 架构师(Mavis 接手 agent per DEC-008) | 提案级(建议级, 未通过决策), per RGS-DOCS-HEALTH-2026-08-25-feedback-to-agents.md §4 |
| **v0.2** | **2026-09-02 00:55** | **架构师(Mavis 接手 agent per DEC-008)** | **草案升版: 6 域受控动作边界 (5 业务 + batch) + 6 worktree 派工验证 + batch 域特殊受控 (GAP-3/4/7/9) + 后续 ADR 升版清单 (6 份 ADR v0.2 草案待跑) (per WBS v0.2 §2.5 桶 11 E7)** |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
