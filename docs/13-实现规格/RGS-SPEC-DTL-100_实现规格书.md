# RGS-DTL-100 实现规格书

**RGS-SPEC-DTL-100**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-100 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口（见 §A.3），待 RGS-DTL-100 具名 DD Review |
| 源详细设计 | RGS-DTL-100（本 DTL 今日未升版，SPEC v0.2 为前瞻性草案，见 §A.1） |
| 实现范围 | `saga-runtime` crate（或等效 Saga Runtime 服务）+ 各参与服务（economy/inventory/shop/mail/character/rank）的 outbox/inbox 表 + `cluster_ops_db` 内 9 张 Saga Store 表 |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、PostgreSQL（`cluster_ops_db` + 各服务本地 DB）、`async-nats`（NATS JetStream，per RGS-DTL-100 §6.2/RGS-REQ-038 FR-NET-011 现状确认） |
| 规格真源 | 源 DTL 的 Saga Store Schema（§7）、Outbox/Inbox 表结构（§4）、补偿时序（§1.3/§2/§3.3）、Reservation 状态机（§5） |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-25 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses(架构师兼 per DEC-008) | Ulysses(一人公司 12 角色兼任,per DEC-008) | 初版。落地 RGS-DTL-100 v0.2 当前内容(3 类 Saga 时序 + Outbox/Inbox + Reservation + Saga Store 9 表 + 补偿时序)转可执行实现清单;§1~§8 完整起草。 | §1~§8(全新增) |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 对齐源 DTL-100 当前版本(`0.2`) + 头表 0.2 + 新增 §A v0.2 对齐说明;**不引入新设计**;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2) | §A(新增) |

---

## 1. 使用规则

本规格把 RGS-DTL-100 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-100 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 Saga Store 9 张表（§7）、Outbox/Inbox 表（各参与服务本地 DB，§4）、Reservation 两阶段状态机（Reserve→Commit/Release，§5）、逆序补偿逻辑（§1.3/§2）、Reward Saga 不可逆事件的 Manual Intervention Queue（§3.3）；不得让 Saga 主流程走同步 RPC 链（§6.1"避免级联超时"硬约束）；不得对已完成的不可逆事件（如 MatchFinished）发出"CancelReward"类撤销事件（§3.3 硬约束）。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| Saga Store | `cluster_ops_db`：`saga_definition`/`saga_instance`/`saga_step`/`saga_event`/`saga_command`/`saga_compensation`/`saga_snapshot`/`saga_failure`/`saga_audit`（9 表） | 字段/索引/FK 与 DTL §7 逐条一致 |
| Saga Runtime | Saga Runtime 服务（crate 名待定，沿用既有 workspace 命名约定）：编排 Purchase/Character Creation/Reward 三类 Saga 定义 + 通用状态机（PENDING/RUNNING/WAITING/RETRYING/COMPENSATING/COMPLETED/FAILED/COMPENSATED） | `fence_token` 单调递增防过期 Leader（DTL §7 saga_instance 注释） |
| Outbox | 各参与服务（economy/inventory/shop/mail/character/rank-service）本地 DB 内 `outbox` 表 + Outbox Worker | 表结构与 DTL §4.1 一致；本地事务内 `domain_update` + `outbox INSERT` 一次 COMMIT |
| Inbox | 各消费端（saga-runtime 及各参与服务）本地 DB 内 `inbox` 表 | `event_id` PRIMARY KEY 去重；表结构与 DTL §4.2 一致 |
| 事件总线 | NATS JetStream（`async-nats`，与 RGS-DTL-100§6.2 现状一致，参见 RGS-REQ-038 FR-NET-011 对该现状与附件D§4.1候选不一致的记录，本规格不裁决） | Subject 命名遵循 §6.2（`SAGA.*`/`EVENT.{domain}.{action}`/`COMMAND.{service}.{action}`） |
| CI | fmt、clippy、test、deny、schema、secret、high-cardinality checks | 负例必须阻断合并 |

## 3. 实现契约

- Reservation 两阶段模型（DTL §5）**必须**遵循：Reserve 仅增加 `reserved`，不扣 `available`；Commit 时才 `available -= amount, reserved -= amount`；Release 只减 `reserved`。任何实现不得跳过 Reserve 阶段直接扣 `available`。
- 所有 Command **必须**携带 `idempotency_key`（格式 `{saga_id}:{step_index}`，DTL §7 `saga_command.idempotency_key UNIQUE`），参与服务收到重复 `idempotency_key` 必须走 inbox dedup 返回已缓存结果，不得重复执行业务效果。
- 补偿**必须**逆序执行（从失败步骤反向），已成功的步骤按 §1.3/§2 补偿表逐步回滚；已 COMMIT 的步骤不得跳过补偿直接标记 Saga 失败。
- Reward Saga（§3）失败重试耗尽后**必须**进入 `saga_failure` 表（`requires_manual=true`）并通知 GM Console，**不得**对已发生的不可逆事实（`MatchFinished`）尝试撤销；补偿只能走 Corrective Event（如手工补发货币），不得发 `CancelReward` 类事件。
- Saga 主流程步骤间**必须**走异步事件（NATS JetStream）或 Saga Runtime 主动下发 Command，**不得**让 Saga 步骤之间形成同步 RPC 链式调用（DTL §6.1，避免级联超时）。
- Outbox `event_id`（UUID v7）即 `dedup_id`，发布到 NATS JetStream 时必须携带，Outbox Worker crash 重启后必须能从 PENDING 状态继续，不得重复发布已 PUBLISHED 的事件。
- `saga_instance.fence_token` 单调递增，Saga Runtime 多副本部署时必须用于防止过期 Leader 继续操作同一 Saga 实例（详细恢复语义见 RGS-DTL-102，本规格不重复定义）。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 指标（`saga_*`）：Saga 状态机转移计数（按 `saga_type`/`from_state`/`to_state`）、步骤失败计数（按 `participant`/`action`）、补偿触发计数、Manual Intervention Queue 深度、Outbox/Inbox 积压（`status='PENDING'` 行数按服务分组）。
- 指标标签：仅 `saga_type`/`state`/`participant`/`action`/`error_type` 等低基数标签；`player_id`/`account_id`/`saga_id` **不**作为 metric label（高基数，per 既有 façade 约束）。
- 关键请求必须能用 `saga_id` + `trace_id`（RGS-SPEC-CROSS-006）反查 Saga Store 全部步骤、补偿记录、事件日志。
- `saga_audit` 表（高风险操作审计）与普通结构化日志分离，归入 OPERATION_AUDIT。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 幂等一致性 | `idempotency_key` UNIQUE 约束 + inbox `event_id` PK 双层去重；参与服务收到重复 Command 不得产生副作用 |
| 故障 | Outbox Worker crash 重启从 PENDING 继续；Saga Runtime 多副本切主时 `fence_token` 阻止过期 Leader 写入（详见 DTL-102） |
| 不可逆事件处理 | Reward Saga 失败重试耗尽 **必须** 走 Manual Intervention Queue，**不得**自动尝试撤销 MatchFinished 事实（§3.3 硬约束，代码 review 需验证无 `CancelReward` 类事件定义） |
| 数据治理 | `saga_instance.payload`/`saga_step.input`/`output` JSONB 内容需符合各域自身的 PII 边界（本规格不重复定义各域字段脱敏规则，引用各域 DTL） |
| 发布 | `saga_definition` 版本化（`saga_type` + `version` UNIQUE），Saga 定义变更走新版本行，不得就地修改已发布的 `definition_json`（避免运行中 Saga 实例定义漂移） |

## 6. 测试规格

- UT：覆盖 Reservation 两阶段状态机（Reserve/Commit/Release/强制释放）全部转移路径 + Outbox/Inbox 幂等去重逻辑 + `idempotency_key` UNIQUE 冲突处理。
- IT：覆盖 Purchase Saga 全链路（Happy Path + Step 2/4/5 各失败点触发的逆序补偿）+ Character Creation Saga 全链路（Happy Path + 各失败点补偿表 §2.2 对应回滚）+ Reward Saga（Happy Path + 失败重试耗尽进入 Manual Intervention Queue）。
- ST：Saga Runtime 多副本部署下的 Leader 切换与 `fence_token` 防护（与 RGS-DTL-102 联合验证）；Outbox Worker crash-restart 后不重复发布。
- Chaos：参与服务响应超时/拒绝/部分成功等 5 类故障注入下的补偿正确性；NATS JetStream 消息重复投递下的 inbox dedup 正确性。
- Security：grep 验证代码库不存在 `CancelReward` 或等价的比赛结果撤销事件类型定义。

测试必须回填 RGS-REQ-004 追踪矩阵（Saga/COC 相关 AC 项，若既有矩阵未覆盖需先在附件C登记后回填）和 DTL-100 §1〜§7 的验收项；不能只证明"服务启动"。

## 7. Definition of Done

- RGS-DTL-100 的 3 类 Saga 时序（Purchase/Character Creation/Reward）与 Outbox/Inbox/Reservation/Saga Store Schema 全部与实现逐项对账。
- Cargo fmt、clippy、test、deny、schema、secret、high-cardinality 检查通过。
- Saga Store 9 表 migration 落地，字段/索引/FK 与 DTL §7 完全一致。
- Reward Saga 的 Manual Intervention Queue + GM Console 通知链路集成测试通过。
- 与 RGS-DTL-101（OperationPolicy/AuthorityBoundary）、RGS-DTL-102（故障恢复）的接口契约联合验证通过（三份 DTL 为同侪文档，非独立可验收）。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

进入实现前必须取得：① 源 DTL RGS-DTL-100 的具名 DD Review；② RGS-DTL-101/RGS-DTL-102 同侪文档已定稿（三者共同构成 Saga 子系统完整设计，不得单独进入实现）；③ `async-nats` NATS JetStream 依赖版本核验（沿用既有 `crates/shared-platform/src/producer.rs` 基线，不新增候选）；④ Saga Runtime 多副本部署环境下的 `fence_token` 防护实测。**本规格不覆盖**：RGS-REQ-038 附带记录的"附件D§4.1 登记候选 Apache Kafka 与代码实际 NATS JetStream 不一致"问题的裁决——该差异由既有事件基础设施选型登记流程处理，非本 Saga 实现规格的 Gate 条件。

---

## A. v0.2 对齐说明（2026-08-26，基于源 DTL 今日状态）

> **本节定位**：本 SPEC v0.2 是 17 份"未升版 DTL"的前瞻性 v0.2 草案（per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2）。**不引入新设计**——仅落实/复核源 DTL 当前版本 + 父 BAS 既有内容；正文本 §1~§8 不重写，新增内容仅本节。

### A.1 源 DTL 今日升版增量（前瞻性视角）

- **源 DTL**：RGS-DTL-100
- **源 DTL 今日状态**：`0.2`（`2026-08-25`）
- **源 DTL 升版路径**：**今日未升版**（`git log --since="2026-08-26 00:00" --until="2026-08-26 23:59" -- docs/**/RGS-DTL-100_*.md` 无 commit）
- **源 DTL 升版类型**：**前瞻性草案**（非"今日升版沉淀"）
- **核心要点**：源 DTL 当前为 v0.2（2026-08-25 升版），升版内容为反映 RGS-ADR-0057（Accepted）§2.3：§3.3 末尾补充交叉引用，确认 Reward Saga 既有设计语义等价于 Outbox+幂等消费者；不改变本节设计本身，不改变 Purchase/Character Creation Saga 补偿编排，不触发 RGS-SPEC-DTL-100/101/102 重新版本化（per RGS-ADR-0057§3.3）。本 SPEC v0.2 草拟时源 DTL 未追加 §A 类的"已知缺口/待办"清单。

### A.2 对本 SPEC 的影响（实现侧）

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.2 同步（v0.1 起草时源 DTL 已为 v0.2） | 与源 DTL `0.2` 同步（范围不变，仅元数据对齐） |
| 源 DTL 真源 | RGS-DTL-100 v0.2 | RGS-DTL-100 `0.2`（具体修订见 §A.1） |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review（本 SPEC v0.2 不阻塞） |
| §8 Gate 证据 | 待 ①源 DTL DD Review ② RGS-DTL-101/102 同侪定稿 ③ `async-nats` 版本核验 ④ `fence_token` 防护实测 | 同 v0.1（本前瞻性草案不新增 Gate） |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全（per DTL-036 v1.4.1 hotfix 复盘 §修式）。本节列出来源 DTL 升版自身声明的待办 / 缺口，本 SPEC 不预设处置方案，待 DDD Review 阶段配套决策。

- 源 DTL 升版内容仅为 前瞻性草案（本 DTL 今日未升版）时，本节无新缺口继承。
- 若源 DTL 升版伴随 §3 已知缺口清单（如 RGS-DTL-036 v1.4.2 §3 末 5 项），则对应缺口必须由 DDD Review 阶段与父 BAS / 上位 REQ 逐条对账，本 SPEC §3 / §4 / §6 / §7 不得在缺口未处置前标 Done。
- 本 SPEC v0.2 自审 0 项发现，**等 DDD Review 阶段再行检查**。

### A.4 引用链与证据

- 源 DTL 修订历史条目：见 RGS-DTL-100 §修订历史表（本 DTL 今日未升版，引用最新一次历史升版 v0.2 2026-08-25 反映 RGS-ADR-0057 §2.3）
- 父 BAS 升版条目：见对应父 RGS-BAS-100 §修订历史表（本 DTL 对应父 BAS，本日是否升版需自审）
- 同期 SPEC 调整总报告：[RGS-SPEC-000 详细设计规格化总表 v0.3](../RGS-SPEC-000_详细设计规格化总表.md) + RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2_v0.1.md（17 份前瞻性 SPEC v0.2 同批）
- **代签已允许**（per 2026-08-26 08:40 JST 偏好反转）：本节"审批者"列 = 真实责任署名 "架构师(Ulysses（一人公司 12 角色 per DEC-008）)"，**不**再受"审批者 = —"硬约束（原占位状态见 git 历史）

> **本 v0.2 调整严格遵循**：① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 代签已允许（新规则） ⑤ 缺标比错标更安全（per DTL-036 hotfix 复盘修式）。
