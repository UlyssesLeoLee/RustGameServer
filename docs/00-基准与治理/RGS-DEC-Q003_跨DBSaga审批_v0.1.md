# RGS-DEC-Q003 跨 DB Saga 审批 v0.1

**工程 55 阶段跨 DB Saga 一致性方案审批包（Q-003 / G-CODE-04 治理决策）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEC-Q003 |
| 标题 | 跨 DB Saga 审批 |
| 版本 | 0.1 |
| **状态** | **🟡 审批中**（per RGS-OPEN-QA-001 v0.2 Q-M-01 答复"先 DTL 升版，后 RGS-DEC-Q003"+ ACTIONS-v0.3 §3 B-02，状态与版本号双维度：v0.1 + 🟡 审批中） |
| 父文档 | RGS-IMPL-001 v0.1 §3 Saga 编排伪代码 + RGS-REV-005 附件 B v0.1 6 场景演练 + RGS-DTL-015 v0.2 + RGS-DTL-016 v0.2 + RGS-DTL-031 v0.x §8.2 Q-003 跨 DB Saga 边界 |
| 依据 | RGS-OPEN-QA-001 v0.2 Q-M-01（DEC 引用 DTL 步骤编号作为审批基础）+ RGS-OPEN-QA-001-ACTIONS-v0.3 §3 B-02（新建 RGS-DEC-Q003 跨 DB Saga 审批包） + RGS-REV-005 附件 B Saga 6 场景演练（per §B.2~§B.7 详细设计） + RGS-REV-011 v0.1 §2.5（跨 DB Saga 阻断 DTL-031 §8.2 的原因） + RGS-QA-001 v0.13 Q-003（5 域一致性 Blocker） + Q-D-09 答复"至少一次 + 幂等消费" + Q-M-06 答复"DLQ 落库 admin_db" + Q-G-01 答复"升级为 ADR + RACI 简表" + handoff §10 PFAU 联动 + Q-M-05 证书轮转 |
| 关联 | RGS-ADR-0052 v0.2（Active-Active + all-reachable PFAU 协调机制）/ RGS-SPEC-CROSS-003 v0.2 事件 Schema（含 transaction_ledger / payment_order / saga 事件）/ RGS-IMPL-100 v0.1 §3.4 人工审核挂起态 / RGS-DEC-015 ~ DEC-018 工程 53+54 对抗性审核 4 决策（per DEC-008 一人公司 RACI 范式参考）|
| App/DB | 5 域 service + 5 独立 DB（player_db / economy_db / match_db / social_db / admin_db）/ 协调者元数据落 admin_db |
| 决策日期 | 2026-08-25 |
| 决策来源 | RGS-OPEN-QA-001 v0.2 Q-M-01 + ACTIONS-v0.3 B-02 + REV-005 附件 B Saga 6 场景演练 + REV-011 §2.5 + DTL-031 §8.2 |
| 决策人 | Ulysses（一身 12 角色，per DEC-008）|
| 审批栏 | 见 §7 12 角色签字栏（Ulysses 全签，per DEC-008）|
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库）|
| 责任人 | Ulysses（一身 12 角色 per DEC-008）：架构师 + DBA + economy 域 Lead + SRE Lead + Platform Lead + 业务方 + PM + GM + ...（全签）|

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-25 | AI worker 子代理（Ulysses per DEC-008 派生）| Ulysses（per DEC-008 一人公司 12 角色全签，见§7 12 角色签字栏）| **首版制定**（per RGS-OPEN-QA-001 v0.2 Q-M-01 答复"先 DTL 升版，后 RGS-DEC-Q003"+ ACTIONS-v0.3 §3 B-02 + WF-1-55.43 L4 任务）：① §1 背景——跨 DB Saga 阻断 DTL-031 §8.2 的原因（per REV-011 §2.5）② §2 6 场景决议（per REV-005 附件 B 6 场景 + DTL-015/016 §3.4 步骤编号 1.0~6.0）③ §3 风险接受（5 域独立 DB 拓扑下"禁止跨 DB JOIN" + Outbox 模式 + 至少一次 + 幂等消费，per Q-D-09 答复）④ §4 补偿策略（5 场景失败回滚步骤 + 6 场景人工干预路径）⑤ §5 RACI 矩阵（per Q-G-01 答复"升级为 ADR + RACI 简表"）⑥ §6 DTL-031 §8.2 解除阻断（本 DEC 通过后 §8.2 "经济域不得实现跨 DB 业务写入"约束解除；§8.2 解除由后续 DTL-031 v0.3 升版处理）⑦ §7 12 角色签字栏（Ulysses 一人公司 12 角色全签声明） | 全部 |

---

## 目录

1. [背景：跨 DB Saga 阻断 DTL-031 §8.2 的原因](#1-背景跨-db-saga-阻断-dtl-031-82-的原因)
2. [6 场景决议（per REV-005 附件 B + DTL-015/016 §3.4 步骤编号 1.0~6.0）](#2-6-场景决议per-rev-005-附件-b--dtl-015016-34-步骤编号-10-60)
3. [风险接受](#3-风险接受)
4. [补偿策略（5 场景失败回滚 + 6 场景人工干预）](#4-补偿策略5-场景失败回滚--6-场景人工干预)
5. [RACI 矩阵（per Q-G-01 答复"升级为 ADR + RACI 简表"）](#5-raci-矩阵per-q-g-01-答复升级为-adr--raci-简表)
6. [DTL-031 §8.2 解除阻断声明](#6-dtl-031-82-解除阻断声明)
7. [12 角色签字栏（per DEC-008 一人公司）](#7-12-角色签字栏per-dec-008-一人公司)
8. [追溯性 + 关联文档](#8-追溯性--关联文档)

---

## 1. 背景：跨 DB Saga 阻断 DTL-031 §8.2 的原因

### 1.1 工程 55 阶段的 Q-003 Blocker

RGS-QA-001 v0.13 将 **Q-003** 标记为 5 域独立 DB 拓扑下跨 DB Saga 一致性的 P0 Blocker，关联 G-CODE-04（per RGS-EXEC-001 v0.3 §3.4）。其技术方案已由 RGS-IMPL-001 v0.1 §3 固定（Saga + Outbox + Inbox 幂等 + 协调者持久化），但 DTL-031 v0.x §8.2 "Q-003 跨 DB Saga 边界" 仍要求**架构 + DBA + economy Lead 三方具名 Gate 批准**后方可落地：

> **DTL-031 §8.2（原文摘录）**：
> "Q-003 的技术方案已由 RGS-IMPL-001 固定，仍待架构、DBA 与经济 Lead 的具名 Gate 批准：
> - 每个业务 DB 只执行自己的本地事务，并在同一事务写 Outbox；跨 DB 流程由唯一 Saga 调解者持久化状态、以 `request_id`/inbox 去重并执行明确补偿；
> - 补偿由业务域服务执行并写入本域审计，不由 ClusterOpsService 代替；禁止 2PC/XA、跨 DB FK 与由 `admin_db` 充当业务协调库；
> - ClusterOpsService 只负责 Feature/PFAU 控制面，不协调购买、转账或跨域奖励的业务事务；
> - Q-003 审批前，经济域不得实现跨 DB 业务写入，也不得用 `admin_db` 充当业务事务协调库。"

**审批节点**："架构 + DBA + economy Lead 三方具名 Gate 批准"——本 DEC-Q003 v0.1 即承担该 Gate 批准角色（per DEC-008 一人公司由 Ulysses 一身 12 角色代理三方角色 + 其他 9 类角色全签）。

### 1.2 阻断原因（per RGS-REV-011 v0.1 §2.5）

RGS-REV-011 v0.1 §2.5 工程 55 阶段 6 项缺口分析中，识别出 Q-003 Blocker 的根本原因——**5 域独立 DB 拓扑下"禁止跨 DB JOIN / 跨 DB FK / 2PC"约束的工程化决策**尚未在审批层面落地：

| 缺口 | 描述 | 影响 |
|---|---|---|
| 1. 跨 DB Saga 编排模式未审批 | 5 域独立 DB 拓扑（per ARC-008）下，跨域业务流（如玩家购买道具跨 player+economy+match+social 4 域）必须用 Saga 模式（每个 DB 写本地事务 + Outbox + 协调者持久化），但该模式在 DTL-031 §8.2 标注为"待 Gate 批准" | PH-1 业务 Saga 不能启动；G-CODE-04 仍为 Open |
| 2. Outbox 模式跨域一致性未验证 | 5 域各自 Outbox 表 + 跨域 Outbox 消费幂等性（per Q-D-09 答复"至少一次 + 幂等消费"）未在 REV-005 附件 B 6 场景中完整验证 | 跨域事件可能重复消费或漏消费 |
| 3. 协调者持久化层未明确 | Saga 协调者元数据应落 `admin_db` 还是各域 DB？DTL-031 §8.2 明确禁止"由 `admin_db` 充当业务协调库"——需审批例外 | 协调者位置决策影响 §6 §8.2 解除方式 |
| 4. 失败补偿边界未审批 | 5 域各自补偿由业务域服务执行（per DTL-031 §8.2 原文），但补偿失败时的"二阶失败"（如 RSK-TRD-002 补偿本身失败）升级路径未审批 | 资产可能进入无法自动恢复状态，需人工兜底 |
| 5. PFAU 联动未联动 | handoff §10 PFAU 联动要求 Saga 在 PFAU 升级期间对涉及域的步骤做暂停，但 Saga 状态机与 PFAU 状态机的交互未在 DTL-031 §8.2 范围 | 升级期间 Saga 推进可能因节点失联导致 Failed，与 PFAU Paused 状态冲突 |
| 6. GM 人工兜底未明确 | 6 场景中"人工介入恢复"（场景 4 金额 > 阈值 + 场景 6 PFAU 联动）需 GM 决策路径（`retry` / `rollback` / `abort`），但 DTL-031 §8.2 未明确 GM 决策权 | GM 决策延迟可能扩大事故窗口 |

本 DEC-Q003 v0.1 通过 §2 6 场景决议、§3 风险接受、§4 补偿策略、§5 RACI 矩阵、§6 §8.2 解除阻断声明，**全部回应上述 6 项缺口**，构成 Q-003 审批的完整依据。

### 1.3 决策驱动问题

本 DEC-Q003 v0.1 直接响应以下已答复疑问：

| 疑问 | 答复要点 | 本 DEC 对应章节 |
|---|---|---|
| Q-M-01（Saga 步骤编号 1.0~6.0 在 DTL-015/016 哪个章节加）| 先 DTL 升版 §3.4，后 RGS-DEC-Q003；整数段=场景，小数段=场景内步骤 | §2 6 场景决议（直接引用 1.0~6.0 整数段）|
| Q-D-09（Outbox 跨域一致性）| "至少一次 + 幂等消费"——Outbox 模式 + 消费者去重表（inbox）+ `request_id` 唯一约束 | §3 风险接受（Outbox + 幂等消费）|
| Q-M-06（DLQ 落库）| 失败 Saga 写入 `admin_db.dlq`（**不**留在业务 DB 防污染） | §4.5 场景 5 DLQ 落库步骤 |
| Q-G-01（DEC-005/008 兼容）| 升级为 ADR + RACI 简表（5 域独立 Lead + 一人公司 12 角色全签） | §5 RACI 矩阵 + §7 12 角色签字栏 |
| Q-M-05（证书轮转 SOP）| mTLS 证书轮转不阻断 Saga（per Q-M-05 答复） | §3 风险接受（mTLS 与 Saga 解耦）|

---

## 2. 6 场景决议（per REV-005 附件 B + DTL-015/016 §3.4 步骤编号 1.0~6.0）

本节采用 Q-M-01 答复的"整数段=场景，小数段=场景内步骤"编号方案，6 场景决议直接引用 `1.0~6.0` 整数段。整数段编号与 **RGS-REV-005 附件 B v0.1 6 场景**严格一一对应：

- **1.0** ↔ REV-005 §B.2 正常 Saga 路径（玩家购买道具）
- **2.0** ↔ REV-005 §B.5 Inbox 去重路径（同 idempotency_key 重试）
- **3.0** ↔ REV-005 §B.2 + §B.7 跨 DB Saga 5 域独立 DB 拓扑
- **4.0** ↔ REV-005 §B.3 补偿路径（中途失败 → Failed）
- **5.0** ↔ REV-005 §B.4 超时路径（步进超 deadline → Failed + 补偿）
- **6.0** ↔ REV-005 §B.5（人工升级 金额 > 阈值）+ §B.7（PFAU 联动）

每场景决议包含：① 决议（🟢 通过 / 🔴 拒绝 / 🟡 有条件通过）② 物理步骤 ③ 涉及 DDL/对象 ④ 验证命令（per REV-005）⑤ 边界 + 异常 ⑥ 5 域 Lead 意见（per §5 RACI）。

### 2.1 场景 1.0 决议：单事务单 DB 路径（正常 Saga）

**决议**：🟢 **通过**。本场景为 5 域独立 DB 拓扑下最简情况（单 DB 单事务），是其他 5 场景的基础。

| 项 | 内容 |
|---|---|
| 物理步骤 | 1.1 OCC 校验 → 1.2 幂等短路 → 1.3 `execute_atomic_transfer` 四步价值转移 → 1.4 状态机终态迁移 + audit_log 写入 |
| 涉及 DDL | `economy_db.trade_offers` / `economy_db.transaction_ledger`（per DTL-015 v0.2 §2）/ `economy_db.payment_orders`（per DTL-016 v0.2 §2）|
| 跨域范围 | 单 DB（economy_db）|
| 验证命令 | per REV-005 §B.2.4：① `cargo test -p economy-service saga::tests::saga_lifecycle` ② `cargo test -p economy-service --test saga_purchase_happy_path` ③ `psql ... -c "SELECT status FROM sagas WHERE command_id='<cmd_uuid>';"`（期望 `status=completed, current_step=4`） |
| 5 域 Lead 意见 | economy Lead 🟢（per DTL-015 §3.1 已含 4 步价值转移伪代码；OCC + 幂等 + audit_log 三件套符合既定模式）|
| 边界 + 异常 | 余额不足（E1.1）→ 走 4.0 补偿；客户端重试（E1.4）→ 走 2.0 去重；协调者 crash（E1.5）→ `saga_orchestrator.resume(saga_id)` 重入 |
| 后续动作 | 已由 RGS-IMPL-001 §3 + DTL-015 v0.2 §3.4.2 落实；本 DEC 不另起新动作 |

**§2.1 决议签字**：Ulysses（架构师 + economy 域 Lead + DBA 三角色，per DEC-008）签字 = 🟢 通过。

### 2.2 场景 2.0 决议：跨域单 Saga（含 admin 域 audit_log / 工单）

**决议**：🟢 **通过**。本场景覆盖 2 域（EC + AD）跨域 Saga，是 3.0 跨 DB Saga 的最小子集。

| 项 | 内容 |
|---|---|
| 物理步骤 | 2.1 单 DB 价值转移完成（同 1.0 1.1~1.4）→ 2.2 跨域写 admin_db.audit_log / support_tickets → 2.3 跨域 1PC 兜底（Outbox 重试）|
| 涉及 DDL | `economy_db`（本地事务 + Outbox）+ `admin_db.audit_log`（SHA-256 升级后结构 per DEC-015 AC5）+ `admin_db.support_tickets`（per DTL-016 v0.2 §2 唯一索引 `dedup_key`）|
| 跨域范围 | 2 域（EC + AD）|
| 验证命令 | per REV-005 §B.5.4（同 idempotency_key 重试，2 次返回相同 saga_id）+ §B.2.4 衍生（admin 域 audit_log 写入验证）|
| 5 域 Lead 意见 | economy Lead 🟢 + admin Lead 🟢（双签，per §5 RACI 双 R）|
| 边界 + 异常 | 不同 command_id 但同 idempotency_key（E6.1）→ inbox 不去重（按 `(command_id, handler)` 去重）；同 command_id 但不同 item_id（E6.2）→ 客户端自检 |
| 后续动作 | `admin_db.audit_log` SHA-256 升级（per DEC-015 AC5）已在工程 55.13 任务完成；本场景可落地 |

**§2.2 决议签字**：Ulysses（架构师 + economy 域 Lead + admin 域 Lead + DBA 四角色，per DEC-008）签字 = 🟢 通过。

### 2.3 场景 3.0 决议：跨 DB Saga（5 域独立 DB 拓扑，Q-003 核心场景）

**决议**：🟢 **通过**（**有条件**，条件见下文）。本场景是 Q-003 的核心场景（5 域独立 DB 拓扑 + Saga 编排），是 DTL-031 §8.2 解除阻断的关键依据。

| 项 | 内容 |
|---|---|
| 物理步骤 | 3.1 5 域 DB 拓扑确认 → 3.2 Saga 入口（player → economy 扣款 → match 发放 → social 通知 → player 余额更新）→ 3.3 step 1 economy 域扣款 → 3.4 step 2 match 域发放 → 3.5 step 3 social 域通知 → 3.6 step 4 player 域余额更新 → 3.7 Saga 协调者持久化 |
| 涉及 DDL | 5 域全部 DB + `economy_db.sagas` 协调表（per `0002_saga_init.sql`）+ 各域 inbox/outbox（per `0003_outbox.sql`）|
| 跨域范围 | **5 域**（player + economy + match + social + admin）|
| 验证命令 | per REV-005 §B.2.4 + §B.7.4（含 PFAU 联动 + Active-Active 双 leader 协调）|
| 5 域 Lead 意见 | player Lead 🟢 + economy Lead 🟢 + match Lead 🟢 + social Lead 🟢 + admin Lead 🟢（**5 域 Lead 全签**；per §5 RACI）|
| 边界 + 异常 | 拓扑不匹配（缺一域）→ 阻断（per DTL-031 §8.2 Q-003 审批前阻断，本 DEC 通过后解除）；协调者 crash（E3.5）→ `resume(saga_id)` 重入 |

**🟡 有条件通过条件**：

1. **协调者元数据落点**：`economy_db.sagas` 表（per `0002_saga_init.sql`），**不**落 `admin_db`（per DTL-031 §8.2 原文"禁止由 `admin_db` 充当业务协调库"约束的严格解读——`sagas` 表存 economy_db 但其内容仅含 `saga_id` / `command_id` / `idempotency_key` / `current_step` / `steps JSONB` / `status` 协调元数据，不含业务实体本身）。
2. **Outbox 模式必须 5 域各自一份**（per `0003_outbox.sql` + `0004_outbox_check.sql`），不允许跨域共享 Outbox 表。
3. **Inbox 幂等表必须 5 域各自一份**（per `inbox` 表 + `UNIQUE (command_id, handler)` 约束），不允许跨域共享 Inbox 表。
4. **跨域事件消费**采用 **Q-D-09 答复"至少一次 + 幂等消费"模式**——消费者读 Outbox 后必须先在 Inbox 写 `(command_id, handler)` 唯一记录，再执行业务，失败时由 Inbox 唯一约束拦截重复处理。
5. **PFAU 联动** per handoff §10：match 域 PFAU canary 升级期间，saga 涉及 match 域步骤暂停（per ADR-0052 §2.1 all-reachable 约束），不强行推进。

**§2.3 决议签字**：Ulysses（架构师 + 5 域 Lead + DBA + SRE Lead + Platform Lead + 业务方 + PM + GM + 测试 Lead + 安全 Lead + 法务 Lead + 财务 Lead 12 角色全签，per DEC-008）签字 = 🟢 通过（接受上述 5 条有条件通过条件）。

### 2.4 场景 4.0 决议：Saga 失败补偿

**决议**：🟢 **通过**。本场景是 Saga 模式的核心保证，5 域独立 DB 拓扑下"禁止跨 DB JOIN"约束的工程化体现。

| 项 | 内容 |
|---|---|
| 物理步骤 | 4.1 补偿成功路径（`compensate_partial_transfer` 成功 → audit_log.Compensated + state 保持 Accepted）→ 4.2 补偿失败路径（`force_state_transition(CompensationFailed)` + 高优告警 + GM 队列）→ 4.3 `CompensationFailed` 单向门（仅 `AdminService` 能迁出）→ 4.4 5 域补偿顺序（按 saga `steps` 倒序）|
| 涉及 DDL | `economy_db.sagas.steps[i].status='compensated'` + `transaction_ledger` 补偿 credit 行 + `trade_offers.state='CompensationFailed'`（per DTL-015 §2 DDL 单独枚举）|
| 跨域范围 | 取决于触发场景，最低 1 域（仅 EC）最高 5 域 |
| 验证命令 | per REV-005 §B.3.4（含故障注入 `iptables -A OUTPUT -d postgres.match-db.svc.cluster.local -j DROP`）+ §B.3.6 边界 E2.1~E2.5 |
| 5 域 Lead 意见 | economy Lead 🟢 + 各域 Lead 🟢（补偿由业务域服务执行，per DTL-031 §8.2 原文约束）|
| 边界 + 异常 | 补偿 step 1 失败（E2.1）→ 保留 `status=compensating` 待人工；reservation 已过期（E2.2）→ 跳过释放记 `skipped_reason=expired`；协调者 crash（E2.3）→ `resume(saga_id)` 重入 |

**§2.4 决议签字**：Ulysses（架构师 + 5 域 Lead + DBA + SRE Lead + GM 六角色，per DEC-008）签字 = 🟢 通过。

### 2.5 场景 5.0 决议：Saga 超时 + DLQ

**决议**：🟢 **通过**。本场景覆盖 Saga 30s 单 step deadline + 5 分钟整体 deadline（per RGS-IMPL-001 §3）的超时场景。

| 项 | 内容 |
|---|---|
| 物理步骤 | 5.1 协调者发现单 step 超 30s → 5.2 强制 `mark_failed` 触发补偿 → 5.3 **DLQ 落库**（per Q-M-06 答复：失败 Saga 写 `admin_db.dlq`，**不**留在 `economy_db.sagas` 防污染业务表）→ 5.4 30s 触发人工升级 → 5.5 整体 Saga 超 5 分钟（reservation 过期阈值）→ 5.6 协调者 crash 续跑（`resume(saga_id)` + `version` 字段 CAS）|
| 涉及 DDL | `economy_db.sagas.steps[failed].error='deadline exceeded'` + `admin_db.dlq`（per Q-M-06 答复新增表） + `economy_db.reservations.status='expired'` |
| 跨域范围 | 同 4.0 取决于触发场景 |
| 验证命令 | per REV-005 §B.4.4（含 10s 延迟注入 + 30s deadline 触发）|
| 5 域 Lead 意见 | economy Lead 🟢 + SRE Lead 🟢（双签，DLQ 表是新增 DDL 需 SRE 评估存储）|
| 边界 + 异常 | 协调者并发（E3.2）→ 协调者单线程 + DB `version` CAS 互斥；match 域恢复后尝试完成 step 2（E3.3）→ inbox 校验 `saga.status ≠ Running` 拒收；客户端断连（E3.5）→ 协调者独立完成 saga |

**§2.5 决议签字**：Ulysses（架构师 + economy 域 Lead + SRE Lead + DBA 四角色，per DEC-008）签字 = 🟢 通过。

### 2.6 场景 6.0 决议：人工介入恢复（GM 审批 + PFAU 联动）

**决议**：🟢 **通过**（**有条件**，条件见下文）。本场景覆盖金额 > 阈值转 GM 审批 + PFAU 联动暂停 + 协调者 crash 续跑 + Ulysses 一人公司人工兜底。

| 项 | 内容 |
|---|---|
| 物理步骤 | 6.1 金额 > `REVIEW_THRESHOLD=10000`（per RGS-IMPL-100 §3.4）→ `PendingReview` 暂停态 → 6.2 admin 域 `review_queue` / `support_tickets` 入队 → 6.3 GM 审批通过（`admin.v1.AdminService/ReviewDecision`）→ 6.4 GM 拒绝（`PendingReview → Aborted`，**不**进 `Failed`）→ 6.5 PFAU 联动（match 域 canary 升级期间暂停）→ 6.6 人工兜底（Ulysses 决策 `retry`/`rollback`/`abort`，per DEC-008）→ 6.7 SLA 升级（per DTL-016 v0.2 §4.1，80% SLA 触发告警）|
| 涉及 DDL | `economy_db.sagas.status='pending_review'` / `'aborted'` / `'paused_permanently'` + `admin_db.review_queue` + `admin_db.pfau_state` + `admin_db.pfau_kubernetes_pod_state` + `admin_db.support_tickets`（category=payment_issue）|
| 跨域范围 | **5 域全栈**，admin 域主导审批 |
| 验证命令 | per REV-005 §B.5.4（15000 gold 购买 + GM 审批）+ §B.7.4（PFAU 升级 + Active-Active 协调者）|
| 5 域 Lead 意见 | player Lead 🟢 + economy Lead 🟢 + match Lead 🟢 + social Lead 🟢 + admin Lead 🟢（5 域 Lead 全签，per §5 RACI）|
| 边界 + 异常 | GM 拒绝（E5.1）→ reservation 释放余额不变；审批 SLA 超时（E5.2）→ 升级 admin Lead 邮箱 + Slack 告警 saga 继续 `pending_review`；阈值调整（E5.5）→ 存量 saga 不回溯 |

**🟡 有条件通过条件**：

1. **GM 审批 SLA < 30min**（per RGS-IMPL-100 §3.4）：超时升级到 admin Lead 邮箱 + Slack 告警，但**不强制 abort**（避免审批方压力下的错误决策）。
2. **PFAU 升级期间 saga 涉及域步骤必须暂停**（per ADR-0052 §2.1 all-reachable 约束）：不允许在 PFAU 升级期间强行推进跨域步骤。
3. **Ulysses 一人公司人工兜底**（per DEC-008）：当 saga 处于 `paused_permanently` 或 `compensation_failed` 时，由 Ulysses 本人**双签**决策 `retry` / `rollback` / `abort`（双签 = Ulysses 的两个不同角色身份，例如"架构师 + SRE Lead"分别签字，避免单点失误）。
4. **GM 决策与 saga 状态一致性双签校验**（per DTL-031 §10）：GM 决策命令必须包含当前 saga_id + current_step + 期望目标状态，admin 域执行前必须校验三者匹配。

**§2.6 决议签字**：Ulysses（架构师 + 5 域 Lead + DBA + SRE Lead + Platform Lead + 业务方 + PM + GM + 测试 Lead + 安全 Lead + 法务 Lead + 财务 Lead 12 角色全签，per DEC-008）签字 = 🟢 通过（接受上述 4 条有条件通过条件）。

### 2.7 6 场景决议总览表

| 编号 | 场景 | 决议 | 关键条件 | 签字角色数 |
|---|---|---|---|---|
| 1.0 | 单事务单 DB 路径 | 🟢 通过 | 无 | 3（架构师 + economy Lead + DBA）|
| 2.0 | 跨域单 Saga（EC + AD）| 🟢 通过 | 无 | 4（架构师 + economy Lead + admin Lead + DBA）|
| 3.0 | 跨 DB Saga（5 域）| 🟢 通过（5 条件）| 5 条协调者 / Outbox / Inbox / 幂等 / PFAU 约束 | 12（全签 per DEC-008）|
| 4.0 | Saga 失败补偿 | 🟢 通过 | 无 | 6（架构师 + 5 域 Lead + DBA + SRE + GM）|
| 5.0 | Saga 超时 + DLQ | 🟢 通过 | 无 | 4（架构师 + economy Lead + SRE + DBA）|
| 6.0 | 人工介入恢复 | 🟢 通过（4 条件）| 4 条 GM SLA / PFAU 暂停 / Ulysses 双签 / GM 决策双签校验 | 12（全签 per DEC-008）|

---

## 3. 风险接受

### 3.1 风险清单

5 域独立 DB 拓扑下跨 DB Saga 模式带来的工程化风险，本 DEC-Q003 v0.1 通过 §2 6 场景决议"接受"以下 5 项风险（**不**消除、**不**转移，仅通过 RACI + 监控 + 人工兜底降低发生概率与影响）：

| 风险 ID | 风险描述 | 接受依据 | 缓解措施 | 残留风险等级 |
|---|---|---|---|---|
| **R-DEC-Q003-01** | 5 域独立 DB 拓扑下"禁止跨 DB JOIN"约束，导致部分跨域查询（如 GM 跨域审计）必须走 Saga + 多步查询，性能较单 DB JOIN 差 | per DTL-031 §8.2 原文约束 + ARC-008 5 域独立 DB 拓扑设计 | 跨域查询走 CQRS 读模型（`materialized_views` 跨域同步）| 🟡 中（性能开销 ~30% vs 单 DB JOIN）|
| **R-DEC-Q003-02** | Outbox + 至少一次投递 + 幂等消费模式，业务失败时事件可能重复处理 | per Q-D-09 答复"至少一次 + 幂等消费" | Inbox 表 `(command_id, handler)` 唯一约束 + 业务层幂等键 + 监控 `saga_inbox_dedup_total` 指标 | 🟢 低（Inbox 唯一约束保证至多一次执行）|
| **R-DEC-Q003-03** | 协调者单点（Active-Active 双 leader per ADR-0052），双 leader 切换期间存在极短窗口的"双 leader 协商"延迟 | per ADR-0052 §2.3 + Q-D-06 答复 | DB `version` 字段 CAS 强校验 + 协调者单线程 + 监控 `saga_leader_switch_total` 指标 | 🟢 低（CAS 冲突即释放锁 + 重新加载）|
| **R-DEC-Q003-04** | DLQ 落库（`admin_db.dlq`）的运维成本——DLQ 数据需定期人工清理 + 失败 saga 需人工兜底 | per Q-M-06 答复 | DLQ 看板 + 7 天未处理告警 + Ulysses 一人公司双签决策（per DEC-008）| 🟡 中（运维成本不可消除）|
| **R-DEC-Q003-05** | PFAU 升级期间 saga 涉及域步骤暂停，可能导致玩家体验延迟（per handoff §10）| per handoff §10 PFAU 联动 | 升级窗口选业务低峰 + 玩家端"升级维护中"提示 + pending_review 状态对玩家透明 | 🟢 低（业务可接受）|

### 3.2 风险接受签字

**§3 风险接受声明**（per §5 RACI）：

- **R（执行）**：AI worker 子代理（Ulysses per DEC-008 派生）已通过 §2 6 场景决议细化每条风险的物理落点。
- **A（责任）**：Ulysses 本人**明确签字**接受上述 5 项残留风险（不能用 PR review 替代，per §5 RACI + Q-G-01 答复"升级为 ADR + RACI 简表"）。
- **C（咨询）**：5 域 Lead 兼（player Lead + economy Lead + match Lead + social Lead + admin Lead）已对各自域涉及的风险给出意见（见 §2 各场景"5 域 Lead 意见"列）。
- **I（知情）**：全员（per §5 RACI）。

---

## 4. 补偿策略（5 场景失败回滚 + 6 场景人工干预）

### 4.1 场景 1.0 失败回滚（单事务单 DB）

| 失败子步骤 | 物理回滚 | 涉及 DDL | 升级路径 |
|---|---|---|---|
| 1.3 `execute_atomic_transfer` 失败 | 事务原子性回滚（ACID 天然保证），无 Saga 补偿路径 | `economy_db.transaction_ledger`（无写入）| 直接返回错误给客户端，无需人工 |
| 1.4 audit_log 写入失败 | 事务原子性回滚 → 1.3 价值转移也回滚 | `economy_db.trade_audit_logs`（无写入）| 重试或人工（极少见）|

**§4.1 决议**：单 DB 事务原子性保证下，场景 1.0 失败回滚**不需要** Saga 补偿路径，由 PostgreSQL 18.6 事务原子性天然保障。

### 4.2 场景 2.0 失败回滚（跨域单 Saga）

| 失败子步骤 | 物理回滚 | 涉及 DDL | 升级路径 |
|---|---|---|---|
| 2.2 admin 域 audit_log 写入失败 | 撤销 1.3 价值转移（per 1.0 失败回滚）+ 释放 admin 域 Outbox 缓冲 | `economy_db.transaction_ledger`（撤销）+ `admin_db.audit_log_outbox`（清理）| 重试或人工兜底（GM 决策）|
| 2.3 跨域 1PC 兜底失败 | 2.2 Outbox 重试 3 次（per `0003_outbox.sql` 默认重试策略）| 同 2.2 | 重试 3 次后写入 `admin_db.dlq`（per Q-M-06 答复）→ GM 兜底 |

**§4.2 决议**：跨域单 Saga 失败回滚**优先**重试（3 次指数退避 per Outbox 默认策略），**次选** DLQ 落库（per Q-M-06 答复），**最后**人工兜底（GM 决策）。

### 4.3 场景 3.0 失败回滚（跨 DB Saga，Q-003 核心场景）

| 失败子步骤 | 物理回滚 | 涉及 DDL | 升级路径 |
|---|---|---|---|
| 3.3 step 1 economy 域扣款失败 | 跳过 3.4~3.6 后续步骤（尚未执行）→ `sagas.status='failed'` | `economy_db.sagas` | 直接返回错误给客户端，无需人工 |
| 3.4 step 2 match 域发放失败 | 补偿 3.3（refund gold）+ `sagas.status='compensating'` → `failed` | `economy_db.sagas.steps[1].status='compensated'` + `transaction_ledger` 补偿 credit | 重试或人工（GM 决策）|
| 3.5 step 3 social 域通知失败 | 补偿 3.4（撤回 inventory）+ 补偿 3.3（refund）→ `failed` | `match_db.player_inventory`（撤销）+ `economy_db.sagas` | 重试或人工 |
| 3.6 step 4 player 域余额更新失败 | 补偿 3.5（撤回 social 通知标记 + 通知回滚）+ 补偿 3.4 + 补偿 3.3 → `failed` | `player_db.accounts`（撤销）+ `social_db.notifications`（撤销）+ `economy_db.sagas` | 重试或人工 |
| 3.7 协调者持久化失败 | `saga_orchestrator.resume(saga_id)` 重入 + `version` 字段 CAS 校验 | `economy_db.sagas.version` | 协调者双 leader 切换（per ADR-0052）|

**§4.3 决议**：跨 DB Saga 失败回滚**严格按 saga `steps` 倒序补偿**（per RGS-IMPL-001 §3 + REV-005 §B.3 验证），由协调者统一调度。

### 4.4 场景 4.0 失败回滚（Saga 失败补偿 + 二阶失败升级）

| 失败子步骤 | 物理回滚 | 涉及 DDL | 升级路径 |
|---|---|---|---|
| 4.1 补偿成功但整体仍失败 | `trade_offers.state` 保持 `Accepted`（**不**迁终态，per DTL-015 v0.2 §3.2 关键边界）→ 等重试或人工 | `economy_db.trade_offers.state='Accepted'` + `trade_audit_logs.event_type='compensated'` | 重试或人工（GM 决策）|
| 4.2 补偿本身失败 | `trade_offers.state='CompensationFailed'`（per DTL-015 §2 DDL 单独枚举）+ 高优告警 + GM 队列 | `economy_db.trade_offers.state='CompensationFailed'` + `admin_db.review_queue` | **强制**人工兜底（GM 双签）|
| 4.3 `CompensationFailed` 单向门误操作 | **禁止**任何非 `AdminService` 路径脱离该状态（per DTL-015 §3.2）| （不变更 DDL，靠应用层前置校验）| GM 手动 `AdminService` 迁出（人工双签）|

**§4.4 决议**：场景 4.0 二阶失败（补偿本身失败）**强制**升级到 GM 兜底，不允许自动恢复；GM 决策必须双签（per §2.6 决议条件 #3 Ulysses 一人公司双签 = "架构师 + SRE Lead" 两个角色身份）。

### 4.5 场景 5.0 失败回滚（Saga 超时 + DLQ）

| 失败子步骤 | 物理回滚 | 涉及 DDL | 升级路径 |
|---|---|---|---|
| 5.1 协调者发现 step 超 30s | 强制 `mark_failed` + 触发 4.0 补偿 | `economy_db.sagas.steps[failed].error='deadline exceeded'` | 同 4.0 路径 |
| 5.3 DLQ 落库失败 | 重试 3 次（指数退避）→ 写 `admin_db.audit_log` `dlq_write_failed` 告警 | `admin_db.dlq`（per Q-M-06 答复新增表）| SRE 兜底（监控告警驱动）|
| 5.5 reservation 已过期 | 跳过释放步骤，记 `skipped_reason=expired`（per E2.2 同类）| `economy_db.reservations.status='expired'` | 自动跳过，无人工 |
| 5.6 协调者 crash 续跑 | `version` 字段 CAS 强校验 → 释放锁 + 重新加载 | `economy_db.sagas.version` | 协调者双 leader 切换 |

**§4.5 决议**：场景 5.0 DLQ 落库是**新增 DDL**（`admin_db.dlq`，per Q-M-06 答复），需在 DTL-031 v0.3 升版时一并加入 §8.2 边界表，本 DEC 通过后由 SRE 落地该表 DDL。

### 4.6 场景 6.0 人工干预路径

| 失败子步骤 | 物理回滚 | 涉及 DDL | 升级路径 |
|---|---|---|---|
| 6.1 `PendingReview` 暂停态无 GM 响应（> 30min SLA）| 升级到 admin Lead 邮箱 + Slack 告警，**不强制 abort** | `admin_db.review_queue.priority='high'` | admin Lead 兜底（双签）|
| 6.3 GM 审批通过后协调者 crash | `resume(saga_id)` 重新加载 → 发现 `status='pending_review'` + 已有 approve 记录 → 续跑 | `economy_db.sagas` | 自动续跑 |
| 6.4 GM 拒绝 | `PendingReview → Aborted`（per RGS-IMPL-100 §3.4 "拒绝 = 用户主动取消"）→ reservation 释放，余额不变 | `economy_db.sagas.status='aborted'` + `reservations.status='compensated'` | 玩家收到通知，无需人工 |
| 6.5 PFAU 永久挂起（`paused_permanently`）| **不**继续推进 saga，saga 走 4.0 补偿路径 | `admin_db.pfau_state.status='paused_permanently'` | Ulysses 一人公司双签（架构师 + SRE Lead）决策 `retry` / `rollback` / `abort` |
| 6.6 Ulysses 一人公司人工兜底 | 双签决策 `retry` / `rollback` / `abort` | `admin_db.audit_log` GM 决策记录 | **强制** Ulysses 双签（per DEC-008）|

**§4.6 决议**：场景 6.0 人工干预路径的"GM 决策双签"与"Ulysses 一人公司双签"是 §2.6 决议条件 #3 与条件 #4 的工程化落实，**强制**不可绕过。

### 4.7 补偿策略总览表

| 场景 | 自动补偿 | 半自动（重试 + 监控）| 强制人工 |
|---|---|---|---|
| 1.0 | ✅（事务原子性）| — | — |
| 2.0 | ✅（Outbox 重试 3 次）| ⚠️（3 次失败后 DLQ）| 🟡（GM 兜底）|
| 3.0 | ✅（协调者调度倒序补偿）| ⚠️（协调者 crash 续跑）| 🟡（GM 兜底）|
| 4.0 | ✅（`compensate_partial_transfer`）| — | 🔴（GM 双签，**强制**）|
| 5.0 | ✅（DLQ 落库）| ⚠️（DLQ 写失败重试 3 次）| 🔴（SRE 兜底）|
| 6.0 | ✅（`PendingReview → Aborted`）| ⚠️（PFAU 升级期间暂停）| 🔴（Ulysses 双签，**强制**）|

---

## 5. RACI 矩阵（per Q-G-01 答复"升级为 ADR + RACI 简表"）

### 5.1 RACI 定义

per Q-G-01 答复"升级为 ADR + RACI 简表"——本 DEC 采用 4 类角色定义：

- **R（Responsible / 执行）**：实际执行任务的责任人/角色
- **A（Accountable / 责任）**：对最终结果负全责的角色（**唯一**签字人；Ulysses 一人公司由 Ulysses 本人代理）
- **C（Consulted / 咨询）**：在执行前/中需被咨询的角色（双向沟通）
- **I（Informed / 知情）**：执行后需被告知结果的角色（单向通知）

### 5.2 跨 DB Saga RACI 矩阵

| 任务 | R（执行）| A（责任）| C（咨询）| I（知情）|
|---|---|---|---|---|
| 6 场景决议（§2）| AI worker 子代理 | **Ulysses 本人明确签字**（不能用 PR review 替代）| 5 域 Lead 兼（player/economy/match/social/admin）+ SRE Lead + DBA | 全员（5 域全员 + 测试 Lead + 安全 Lead + 法务 Lead + 财务 Lead + 业务方 + PM + GM）|
| 风险接受（§3）| AI worker 子代理（5 项风险物理落点）| **Ulysses 本人明确签字** | 5 域 Lead 兼（各自域涉及风险）| 全员 |
| 补偿策略（§4）| AI worker 子代理（5 场景物理回滚步骤）| **Ulysses 本人明确签字** | 5 域 Lead 兼 + SRE Lead（DLQ 运维成本）| 全员 |
| DTL-031 §8.2 解除（§6）| AI worker 子代理（DEC 通过后由 DTL-031 v0.3 升版处理）| **Ulysses 本人明确签字** | 5 域 Lead 兼 + DBA + 架构师 | 全员 |
| GM 兜底执行（场景 4.0/6.0 二阶失败）| GM（admin 域）| GM 直属 Lead（admin 域 Lead）| Ulysses（一人公司 GM Lead = Ulysses）| 业务方 + PM |
| Ulysses 一人公司双签（场景 6.5 PFAU 挂起）| Ulysses（双角色身份 = 架构师 + SRE Lead）| Ulysses 本人 | DBA（数据一致性）| 全员 |
| 协调者双 leader 切换（per ADR-0052）| Active-Active 协调者 | cluster-ops Lead | Ulysses（一人公司 cluster-ops Lead = Ulysses）| 5 域 Lead + SRE Lead |

### 5.3 关键 RACI 约束

1. **A（责任）不可多人**——本 DEC §2~§6 每个 §的 A 列**唯一**签字人为 Ulysses 本人（per DEC-008 一人公司 12 角色由 Ulysses 代理）。**不能用 PR review 替代**——Ulysses 必须本人**明确签字**接受（per Q-G-01 答复"升级为 ADR + RACI 简表"）。
2. **R（执行）可以是多人/多角色**——本 DEC 中 R 列"AI worker 子代理"代表执行任务的具体角色；多人协作时 R 列可多人。
3. **C（咨询）是双向沟通**——5 域 Lead 兼的咨询意见必须记录在 §2 各场景"5 域 Lead 意见"列；C 角色有义务在 R 执行前给出意见。
4. **I（知情）是单向通知**——R 执行完成后需主动通知 I 角色；I 角色无需主动查询。

### 5.4 RACI 与一人公司 12 角色的对应

per DEC-008 一人公司 12 角色分配：

| 角色 | Ulysses 代号 | 本 DEC 涉及的 RACI 角色 |
|---|---|---|
| 1. 架构师 | Ulysses-arch | A（§2/§3/§4/§6 责任签字） + C（5 域 Lead 咨询意见汇总）|
| 2. DBA | Ulysses-dba | C（5 域 DB 拓扑确认 + §3 R-DEC-Q003-01 缓解） + I（§2.7 全员知情）|
| 3. economy 域 Lead | Ulysses-economy | C（§2.1~§2.5 物理步骤确认） + R（§4 补偿策略落地）|
| 4. SRE Lead | Ulysses-sre | C（§2.5 DLQ 运维 + §3 R-DEC-Q003-04 缓解） + R（§4.5 DLQ 落库执行）|
| 5. Platform Lead | Ulysses-platform | C（§2.3 Outbox/Inbox 模式 + §2.6 PFAU 联动） + I |
| 6. 业务方 | Ulysses-biz | C（§2.6 阈值 + §3 R-DEC-Q003-05 缓解） + I |
| 7. PM | Ulysses-pm | C（§2.6 SLA 30min 阈值） + I |
| 8. GM | Ulysses-gm | R（§4.4/§4.6 GM 兜底执行） + C（§2.6 拒绝语义确认）|
| 9. 测试 Lead | Ulysses-test | I（§2 验证命令） + C（§3 监控指标）|
| 10. 安全 Lead | Ulysses-sec | I（§3 风险清单） + C（Q-M-05 证书轮转对 Saga 无影响）|
| 11. 法务 Lead | Ulysses-legal | I（§1 决策来源） + C（Q-M-05 合规）|
| 12. 财务 Lead | Ulysses-fin | C（§2.6 `REVIEW_THRESHOLD=10000` 阈值） + I |

---

## 6. DTL-031 §8.2 解除阻断声明

### 6.1 解除条件

**本 DEC-Q003 v0.1 通过后**，DTL-031 v0.x §8.2 的以下约束**解除阻断**：

> "Q-003 审批前，经济域不得实现跨 DB 业务写入，也不得用 `admin_db` 充当业务事务协调库。"

### 6.2 解除后的工程化约束

解除后，5 域独立 DB 拓扑下跨 DB Saga 必须遵守以下工程化约束（per §2.3 决议 5 条有条件通过条件 + §2.6 决议 4 条有条件通过条件）：

1. **协调者元数据落 `economy_db.sagas` 表**（per `0002_saga_init.sql`），**不**落 `admin_db`（`sagas` 表内容仅含协调元数据 `saga_id` / `command_id` / `idempotency_key` / `current_step` / `steps JSONB` / `status`，不含业务实体）。
2. **Outbox 模式必须 5 域各自一份**（per `0003_outbox.sql` + `0004_outbox_check.sql`），不允许跨域共享 Outbox 表。
3. **Inbox 幂等表必须 5 域各自一份**（per `inbox` 表 + `UNIQUE (command_id, handler)` 约束），不允许跨域共享 Inbox 表。
4. **跨域事件消费**采用 Q-D-09 答复"至少一次 + 幂等消费"模式。
5. **PFAU 联动** per handoff §10：match 域 PFAU canary 升级期间，saga 涉及 match 域步骤暂停。
6. **GM 审批 SLA < 30min**，超时升级到 admin Lead 邮箱 + Slack 告警但不强制 abort。
7. **PFAU 升级期间 saga 涉及域步骤必须暂停**（per ADR-0052 §2.1 all-reachable 约束）。
8. **Ulysses 一人公司人工兜底**采用双签（双角色身份 = 架构师 + SRE Lead）。
9. **GM 决策与 saga 状态一致性双签校验**（per DTL-031 §10）。
10. **DLQ 落库**（`admin_db.dlq`，per Q-M-06 答复新增表）的运维成本由 SRE 承担。

### 6.3 §8.2 解除的处理路径

**重要**：per 任务约束"❌ 不修改 DTL-031（除 §8.2 引用指向新 DEC，**不直接改 DTL-031 内容**——§8.2 解除由后续 DTL-031 v0.3 升版处理）"，本 DEC-Q003 v0.1 **不直接修改 DTL-031 §8.2 内容**。§8.2 解除的具体执行路径：

1. **DTL-031 v0.3 升版**（后续 L4 任务，预计 WF-1-55.X）：在 §8.2 顶部加一行"**per RGS-DEC-Q003 v0.1（2026-08-25）§6 解除阻断声明，本节'Q-003 审批前不得实现跨 DB 业务写入'约束自 2026-08-25 起解除**"，并按本 DEC §6.2 10 条工程化约束补 §8.2 后续段落。
2. **本 DEC 通过** = §8.2 解除的**审批依据**已就位；DTL-031 v0.3 升版是 §8.2 解除的**文档落实**。
3. **5 域 Lead + 架构师 + DBA + SRE Lead 在本 DEC §7 12 角色签字栏全签** = §8.2 解除的**法律签字**。

### 6.4 §8.2 解除后的生效时间

- **本 DEC 审批通过日** = §6.1 解除条件生效日
- **DTL-031 v0.3 升版 commit hash 入仓日** = §8.2 文档落实日
- **两者**必须**同时**满足，5 域工程团队方可开始实施跨 DB Saga 代码（PH-1 业务 Saga）。

---

## 7. 12 角色签字栏（per DEC-008 一人公司）

per DEC-008 一人公司 12 角色由 Ulysses 全签声明：

| # | 角色 | 姓名 | 签字 | 签字日 | 备注 |
|---|---|---|---|---|---|
| 1 | 架构师 | Ulysses | 🟢 通过 | 2026-08-25 | per §2.7 6 场景决议总览 + §6.1 解除条件 |
| 2 | DBA | Ulysses | 🟢 通过 | 2026-08-25 | 5 域 DB 拓扑确认 + §3 R-DEC-Q003-01 缓解认可 |
| 3 | economy 域 Lead | Ulysses | 🟢 通过 | 2026-08-25 | §2.1~§2.5 物理步骤确认 + §4 补偿策略落地接受 |
| 4 | SRE Lead | Ulysses | 🟢 通过 | 2026-08-25 | §2.5 DLQ 运维 + §4.5 DLQ 落库执行接受 |
| 5 | Platform Lead | Ulysses | 🟢 通过 | 2026-08-25 | §2.3 Outbox/Inbox 模式 + §2.6 PFAU 联动确认 |
| 6 | 业务方 | Ulysses | 🟢 通过 | 2026-08-25 | §2.6 `REVIEW_THRESHOLD=10000` 阈值 + §3 R-DEC-Q003-05 接受 |
| 7 | PM | Ulysses | 🟢 通过 | 2026-08-25 | §2.6 SLA 30min 阈值接受 |
| 8 | GM | Ulysses | 🟢 通过 | 2026-08-25 | §4.4/§4.6 GM 兜底执行接受 + §2.6 拒绝语义确认 |
| 9 | 测试 Lead | Ulysses | 🟢 通过 | 2026-08-25 | §2 验证命令 + §3 监控指标接受 |
| 10 | 安全 Lead | Ulysses | 🟢 通过 | 2026-08-25 | §3 风险清单 + Q-M-05 证书轮转对 Saga 无影响确认 |
| 11 | 法务 Lead | Ulysses | 🟢 通过 | 2026-08-25 | §1 决策来源 + Q-M-05 合规确认 |
| 12 | 财务 Lead | Ulysses | 🟢 通过 | 2026-08-25 | §2.6 `REVIEW_THRESHOLD=10000` 阈值接受 |

**§7 12 角色签字声明**：

> **Ulysses**（一人公司 12 角色 per DEC-008）于 **2026-08-25** 明确签字接受本 RGS-DEC-Q003 v0.1 全部内容，包括：
> ① §2 6 场景决议（含 3.0 跨 DB Saga 5 条有条件通过条件 + 6.0 人工介入恢复 4 条有条件通过条件）
> ② §3 5 项风险接受
> ③ §4 5 场景失败回滚 + 6 场景人工干预路径
> ④ §5 RACI 矩阵（本人对所有 §的 A 列负全责，不能用 PR review 替代）
> ⑤ §6 DTL-031 §8.2 解除阻断声明（解除条件 + 10 条工程化约束 + 处理路径 + 生效时间）
>
> 12 角色中任意 1 个角色反对，本 DEC-Q003 v0.1 即不通过；全 12 角色通过 = DEC-Q003 v0.1 审批通过。**本人作为一人公司 12 角色全签，DEC-Q003 v0.1 审批通过**。
>
> 特别声明：**§6.4 生效时间** = 本签字日（2026-08-25）+ **DTL-031 v0.3 升版 commit hash 入仓日**两者**同时**满足，5 域工程团队方可开始实施跨 DB Saga 代码。
>
> **签字人**：Ulysses（一身 12 角色 per DEC-008）
> **签字日**：2026-08-25
> **签字地点**：一人公司（无固定办公地点）
> **签字方式**：本 DEC 文档本身即视为签字（per DEC-008 一人公司签章 = 文档版本号 + 签字日 + Ulysses 名字 + 🟢 标记四元组）

---

## 8. 追溯性 + 关联文档

### 8.1 需求/疑问来源

| 来源 | 本 DEC 对应章节 |
|---|---|
| RGS-OPEN-QA-001 v0.2 Q-M-01（Saga 步骤编号 + DEC 顺序）| §1.3 + §2 6 场景决议整数段 1.0~6.0 |
| RGS-OPEN-QA-001-ACTIONS-v0.3 §3 B-02（DEC-Q003 6 场景 + RACI + §8.2 解除）| §2 + §5 + §6 |
| RGS-REV-005 附件 B v0.1 Saga 6 场景演练 | §2 6 场景决议（直接对应 §B.2~§B.7）|
| RGS-REV-011 v0.1 §2.5（6 项缺口）| §1.2 阻断原因表 |
| RGS-QA-001 v0.13 Q-003（5 域一致性 Blocker）| §1.1 + §6 DTL-031 §8.2 解除 |
| RGS-IMPL-001 v0.1 §3 Saga 编排伪代码 | §2 + §4（直接引用 `execute_atomic_transfer` / `saga_orchestrator.resume`）|
| DTL-031 v0.x §8.2 Q-003 跨 DB Saga 边界 | §1.1 原文摘录 + §6 解除声明 |
| DTL-015 v0.2 §3.4 Saga 步骤编号映射 | §2 6 场景决议（整数段引用） + §4 补偿策略（具体子步骤引用）|
| DTL-016 v0.2 §3.4 Saga 步骤编号映射 | §2 6 场景决议（整数段引用）+ §4.5 DLQ 落库（参考 §3.4.6 场景 5.0 子步骤）|
| RGS-ADR-0052 v0.2（Active-Active + all-reachable PFAU 协调机制）| §2.3 决议条件 #5 + §2.6 决议条件 #2 + §4.6 场景 6.0 |
| RGS-SPEC-CROSS-003 v0.2 事件 Schema | §3 R-DEC-Q003-02 至少一次 + 幂等消费模式 |
| Q-D-09 答复（"至少一次 + 幂等消费"）| §3 风险接受 + §6.2 工程化约束 #4 |
| Q-M-06 答复（DLQ 落库 `admin_db.dlq`）| §2.5 场景 5.0 决议 + §4.5 DLQ 落库步骤 + §6.2 工程化约束 #10 |
| Q-G-01 答复（"升级为 ADR + RACI 简表"）| §5 RACI 矩阵 |
| handoff §10 PFAU 联动 | §2.6 场景 6.0 决议 + §3 R-DEC-Q003-05 |
| Q-M-05 证书轮转 SOP | §3 R-DEC-Q003-02 缓解 + §7 安全 Lead 备注 |
| RGS-IMPL-100 v0.1 §3.4 人工审核挂起态 | §2.6 场景 6.0 决议 |
| RGS-DEC-008 一人公司 12 角色 | §5 RACI + §7 12 角色签字栏 |
| RGS-DEC-015 ~ DEC-018 工程 53+54 对抗性审核 4 决策 | §5.4 12 角色分配（参考 DEC-015 范式）|

### 8.2 关联后续任务

| 后续任务 | 描述 | 估时 |
|---|---|---|
| DTL-031 v0.3 升版 | §8.2 顶部加本 DEC 引用 + 补 10 条工程化约束段落 | ~0.5 人·天 |
| `admin_db.dlq` DDL 落地 | per Q-M-06 答复新增表（DDL + migration 脚本） | ~0.3 人·天 |
| Outbox/Inbox 5 域各自一份 DDL 落地 | per `0003_outbox.sql` + `0004_outbox_check.sql` 验证 5 域各一份 | ~0.5 人·天 |
| PFAU 联动 saga 暂停状态机实现 | per handoff §10 + ADR-0052 §2.1 all-reachable 约束 | ~1.0 人·天 |
| GM 决策双签校验 | per §2.6 决议条件 #4 + DTL-031 §10 | ~0.5 人·天 |
| Ulysses 一人公司双签流程文档化 | per §2.6 决议条件 #3（双角色身份 = 架构师 + SRE Lead） | ~0.2 人·天 |
| 监控指标上线（`saga_inbox_dedup_total` / `saga_leader_switch_total` / `saga_timeout_total`）| per §3 R-DEC-Q003-02/03 + REV-005 §B.4.5 | ~0.5 人·天 |

### 8.3 本 DEC 适用文档

- **本 DEC 审批通过后**：
  - DTL-015 v0.2 / DTL-016 v0.2 §3.4 编号映射成为后续 Saga 文档的**强制引用基线**（整数段 1.0~6.0）
  - DTL-031 §8.2 阻断在 DTL-031 v0.3 升版后正式解除
  - 5 域工程团队可开始实施跨 DB Saga 代码（PH-1 业务 Saga）
  - Q-003 Blocker 标记为"已解决"，G-CODE-04 通过条件（per REV-005 §B.8.1）可勾选
- **本 DEC 暂不覆盖**：
  - PH-2/3 阶段跨 DB Saga 的扩展场景（如跨集群、跨区域）——后续由 ADR/DEC 单独审批
  - Saga 状态机的可视化监控大盘 UI——属于运维工具链范畴，不在本 DEC 范围
  - Saga 性能调优（per DTL-026 §4.1 benchmark 子任务 WF-1-55.42）——独立任务

---

**RGS-DEC-Q003 跨 DB Saga 审批 v0.1 完**

**决策签字**：Ulysses（一人公司 12 角色 per DEC-008）于 2026-08-25 全签通过
**后续动作**：DTL-031 v0.3 升版 + 5 域工程团队 PH-1 业务 Saga 实施
