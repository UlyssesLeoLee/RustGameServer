# ADR-0015 §5 协同决策章节补 (Outbox 跨域协调) — 候选

> **状态**: 🔵 v0.1 candidate (per ULYS-103 acceptance #4 + ADR-0061 §6 P2-#5 §4)
>
> **依据**: ADR-0015 §5 关联段 + ADR-0061 §4 后果 §5 关联 (per ADR-0061 v0.3 已添加 ADR-0015 协同决策项)
>
> **ULYS-103 worker**: agent c557dae5-42e4-4d60-bf27-36eeddbf67bb (2026-09-19 JST)
>
> **重要声明**: 本文件是**候选草案**，**不直接修改** ADR-0015 主文档（ADR-0015 头表「状态」为 Accepted，修订需 Ulysses 一人公司 12 角色 per DEC-008 显式授权；ULYS-103 worker 作为 agent 不代签已 Accepted ADR）。本候选为 ADR-0015 v0.x → v0.x+1 升版做草案准备。

---

## §1. 候选章节内容（插入位置：ADR-0015 §5 关联 段尾）

> ### 5.x Outbox 跨域协调（per ADR-0061 §4 后果）
>
> RGS 6 域通过**自研 Outbox 模式 + NATS JetStream**实现跨域事件传播（per ADR-0061 §2 决定 1）。本节明确 Outbox 路径在 Saga 边界（ADR-0015 §2）下的协调规则：
>
> **1. Outbox 与 Saga Reserve/Commit 步骤的关系**：
> - Saga Reserve 步骤（per RGS-DTL-100 §4 + RGS-SPEC-CROSS-005 事务性消息）: 业务写 DB + 写 outbox 表**必须同一事务**（per ADR-0061 §2 决定 2）。outbox 表的 `saga_id` 列关联 Saga 上下文。
> - Saga Commit 步骤: 跨域补偿事件通过 outbox 发布，consumer 端按 `command_id` 幂等（per ADR-0061 §2 决定 3）。
>
> **2. Outbox 4 状态机与 Saga 状态机正交**:
> - Outbox: Pending → InFlight → Sent / Failed（per `crates/shared-platform/src/outbox.rs`）。
> - Saga: Pending → Reserved → Committed / Compensated（per ADR-0015 §2 决定 1）。
> - 两个状态机独立运行: Saga 步骤状态由 saga 业务表维护; Outbox 行状态由 relay 维护。两者通过 `saga_id` 关联（不强制，但用于追溯）。
>
> **3. 单一调解者原则**（per ADR-0015 §2 决定 2）:
> - Saga 单一调解者 = Saga Orchestrator（per `crates/shared-platform/src/saga_orchestrator.rs`）。
> - **Outbox relay 不参与 Saga 协调决策**: relay 只负责 publish 跨域事件，不知道 Saga 状态机。
> - Saga Orchestrator 通过「业务层 Saga 步骤 + outbox 写」组合实现调解, 不引入 Outbox relay 作为调解者（避免 §3.2 Choreography 反模式）。
>
> **4. DLQ 与 Saga 补偿的关系**（per ADR-0061 §2 决定 4）:
> - Outbox Failed 状态（per P2-#3 DLQ 草案 §2.2）触发 DLQ, **不直接触发 Saga 补偿**。
> - Saga 补偿由 Saga Orchestrator 在 Saga 业务步骤失败时主动发起, 不依赖 Outbox 失败信号。
> - 两者解耦: DLQ 是事件层 SRE 处置路径, Saga 补偿是业务层事务恢复路径。

---

## §2. 候选修订历史条目（per §7 修订历史）

| 版本 | 日期 | 修订者 | 内容 |
|---|---|---|---|
| 0.x+1 | 待 D-Boy 显式授权 | Ulysses 一人公司 (per DEC-008) | §5 关联 新增 "5.x Outbox 跨域协调" 章节 (per ADR-0061 §4 后果 + ULYS-103 worker v0.1 candidate) |
| 0.x | 既有 | 既有 | ADR-0015 既有 §5 关联段不变 |

---

## §3. 升版路径

**触发条件**（per ADR-0015 升版流程 + DEC-008）:
1. ADR-0061 头表「状态」为「已批准 (Accepted)」（per ADR-0061 v0.4+ 修订 + multica issue ULYS-89 v0.5 闭环）。
2. D-Boy 显式授权 agent 推进 ADR-0015 §5 补 Outbox 跨域协调（per ULYS-55 / ADR-0060 precedent comment「可以merge」go-ahead 模式）。
3. ULYS-103 worker 升 commit + 把 §1 候选内容合并到 ADR-0015 §5 + §2 修订历史条目合并到 ADR-0015 §7 + 头表最新版本字段 v0.x → v0.x+1 + 头表最新修订日期字段。

**当前状态（2026-09-19 JST）**:
- ADR-0061 在多 worktree 视角下状态不一致:
  - multica 侧（per ULYS-89 评论 `01a0b8d9-...`）: 已 Approved（per D-Boy「帮我完成剩余」comment `01a0b8cc-...` go-ahead）
  - 本 worktree 物理文件: 仍为「待具名人类审批」（per `docs/08-架构决策记录/RGS-ADR-0061_CDC_Outbox偏离参考设计_自研Outbox取代DebeziumCDC.md` 头表）
- ULYS-103 worker **不代签 ADR-0015 修订**, 仅预备 §1 候选内容 + §2 修订历史条目草稿。升版动作等 ADR-0061 头表物理同步到 Accepted 后, 由 Ulysses 一人公司 12 角色 (per DEC-008) 显式授权下执行。

---

## §4. 关联文档

- **ADR-0015** 主文档 (`docs/08-架构决策记录/RGS-ADR-0015_工作流（Saga）适用边界与单一调解者原则.md`) — **不替换**。
- **ADR-0061** CDC/Outbox 偏离 (`docs/08-架构决策记录/RGS-ADR-0061_CDC_Outbox偏离参考设计_自研Outbox取代DebeziumCDC.md`) — §4 后果 §5 关联 已添加 ADR-0015 协同决策项。
- **RGS-DTL-100 §4 Outbox + Inbox Pattern** (per ADR-0061 v0.3 章节引用修订) — Saga Reserve/Commit 与 Outbox 同事务要求。
- **RGS-SPEC-CROSS-005** 事务性消息规范 (实际位于 RGS-DTL-100 §4, 见 SPEC-CROSS-005 路径修正注记)。
- **crates/shared-platform/src/saga_orchestrator.rs** — Saga 单一调解者 source of truth。
- **crates/shared-platform/src/outbox.rs** — Outbox 4 状态机 source of truth。
- **07_PoisonEvent_DLQ草案.md** — DLQ 入口与处置流程 (per §1 候选章节第 4 项 DLQ 与 Saga 补偿的关系)。

---

## §5. 修订历史

| 版本 | 日期 | 修订者 | 内容 |
|---|---|---|---|
| 0.1 candidate | 2026-09-19 JST | ULYS-103 worker (agent c557dae5) | v0.1 候选：ADR-0015 §5 关联 新增 "5.x Outbox 跨域协调" 章节草案（4 项要点）；升版路径 + 当前状态注记；修订历史条目草稿 |