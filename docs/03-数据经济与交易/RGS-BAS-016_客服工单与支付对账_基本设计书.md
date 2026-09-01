# 基本设计书（基本設計書 / Basic Design Document）

**客服工单与支付对账 Customer Support Ticketing & Payment Reconciliation**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-016 |
| 版本 | 0.4 |
| 父文档 | RGS-REQ-019 需求定义书（ARC-033） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-09-01 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-019§8 ARC-033展开为工单组件设计、对账批处理时序、数据模型 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①补充工单状态机迁移条件表与去重字段（FR-SUP-002、FR-SUP-007）②补充SLA分级数值基准表（FR-SUP-003）③补充`SupportTicket`/`PaymentOrder`索引与唯一性约束④补充对账批处理异常分支（服务商侧数据延迟/不可用）与"比对条件写反"防护（RSK-SUP-002） | FR-SUP-002、FR-SUP-003、FR-SUP-007、RSK-SUP-002 |
| 0.3 | 2026-08-16 | 架构师 | — | **补齐跨文档字段清单同步**（RGS-BAS-010 PAT-CR-004处置）：`PaymentOrder`表补入RGS-BAS-020§2.5此前单向追加、未同步回本表的`payment_channel`/`platform_type`/`platform_environment`/`refund_status`四字段，本表重申为该逻辑表的唯一权威字段清单 | FR-PLT-003〜005 |
| 0.4 | 2026-09-01 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部 36 个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§2.1（组件划分架构边界观察）／§2.2（SupportTicket 数据模型 schema 事件）／§2.3（工单状态机迁移 + 玩家-客服对话内容脱敏）／§2.4（SLA 分级基准预警 + 升级通知）／§3.1（PaymentOrder 数据模型 schema 事件 + 跨文档字段同步）／§3.2（对账批处理全链路 + 双重布尔校验 + 幂等键 + 资产结算补偿）／§3.3（对账异常分支 + RSK-SUP-002 防护 + 服务商侧不可用告警）／§4.1（上线前检查清单执行）／§4.2（代码评审检查清单执行）共 9 个"本功能日志设计"小节全部新增；每节均含 5 列详尽版（字段名／触发条件／频率估算／采样策略／脱敏与成本），显式区分 `info!`／`warn!`／`error!`（release 必出，编译期常驻，per BAS-004 v0.3 §6.2 强制全采样白名单）与 `debug!`／`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only，release build 完全剔除零运行时开销）两类事件；字段名前缀 `cs.*`（区别于 BAS-002 `mnt.*` ／ BAS-003 `gm.*` ／ BAS-004 `log.*` ／ BAS-005 `plugin.*` ／ BAS-009 `gov.*`），命名严格 snake_case 与 BAS-004 v0.3 §4.6.1／§4.6.2 保持拼写一致（FR-LOG-013）；**客服工单域特殊考虑**（合规审计 + 隐私双重约束）—— ①工单创建／分配／处理／关闭 → release 必出 + 强制全采样（FR-SUP-001〜005 强约束）；②玩家-客服对话内容 → 脱敏（邮箱／手机哈希化 per BAS-004 §5.1，对话原文 dump 仅 debug-only）；③支付对账（订单／退款／差异）→ release 必出 + 强制全采样（NFR-EC 合规）；④**支付凭证／卡号 → 禁止记录**（per BAS-004 §5.1 + §4.4 release 必出宏清单，SDK 黑名单拦截）；⑤工单 SLA 警告／超时 → `warn!` 强制全采样（per BAS-009 治理事件必出模式）；⑥对账双重布尔校验（RSK-SUP-002 防护）→ release 必出 + 强制全采样 + 快照可追溯；⑦对账服务商侧不可用 → 触发告警通道（per RGS-BAS-003 §6）；§4.1 上线前检查清单新增 log 章节上线检查项（log_chapter_present + release_required_grep_passed + debug_only_compliant + release_required_macro_no_cfg 共 4 项 CI 验证事件）；§4.2 代码评审检查清单新增 log 章节代码评审检查项（admin_bypass / provider_txn_id_uniqueness / dual_boolean_direction / dedup_prompt_not_block / conversation_pii_log / payment_credential_log_attempt 共 6 项静态扫描事件）；§5 追溯性新增 AC-SUP-006（debug-only 宏 release 完全剔除）与 AC-SUP-007（每功能 BAS 文档须含本功能 log 设计章节），与 BAS-001 v1.5 §4.8.3.4（commit 32d9eb6）／ BAS-003 v0.3 §13（commit 75a001c）／ BAS-004 v0.3 §12（commit 47e26b0+0ee6262）／ BAS-005 v0.3 §11（commit 20b84a1）／ BAS-009 v0.7 §7（commit 9a628cf）形成统一规范 | §2.1、§2.2、§2.3、§2.4、§3.1、§3.2、§3.3、§4.1、§4.2、§5 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 对账批处理与支付服务商回调接口的兼容性 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [客服工单组件设计](#2-客服工单组件设计)
3. [支付对账数据模型与时序](#3-支付对账数据模型与时序)
4. [标准化检查清单](#4-标准化检查清单)
5. [追溯性](#5-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-019定义的ARC-033，全部组件依附既有AD（GM后台）/EC（经济）限界上下文运行，不新建独立限界上下文。

---

# 2. 客服工单组件设计

## 2.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `TicketService` | AD | 工单创建/状态机/SLA计时，玩家侧与GM后台侧共用同一数据模型 |
| `TicketEscalationNotifier` | AD | SLA超时检测与升级提醒，复用RGS-BAS-003§6告警推送通道 |
| （处置执行） | AD既有`AdminService` | 工单处理决定的**唯一**执行入口，`TicketService`本身不直接修改账号状态 |

### 2.1 本功能日志设计

本节覆盖**客服工单域整体架构的边界观察点**——客服工单组件本身不直接产生业务事件（业务事件归 §2.2～§3.3 各功能段），但 `TicketService` 启动／关闭、`TicketEscalationNotifier` 心跳、与 `AdminService` 审计通道建立等架构层诊断事件是 SRE 在 Prometheus／Grafana 上追踪"工单能力是否可用"与"对账链路是否存活"的必要输入。**架构层诊断事件属治理信号** → release 必出。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.component.ticket_service.boot_completed` | `TicketService` 启动完成，注册表监听器／DB 连接池已就绪 | 每节点启动 1 次 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `node_id` ／ `bounded_context`（`AD`）；约 220B／条 × 启动频次 = 极低 |
| `cs.component.ticket_service.boot_failed` | 启动失败（DB 连接失败／订阅通道未就绪） | 极少（部署事故） | release 必出（100% 强制全采样，`error!` 级别） | 含 `node_id` ／ `error` ／ `trace_id`；约 300B／条 |
| `cs.component.escalation_notifier.boot_completed` | `TicketEscalationNotifier` 启动完成，SLA 扫描定时器就绪 | 每节点启动 1 次 | release 必出（100% 强制全采样） | 含 `node_id` ／ `tick_interval_seconds`；约 240B／条 |
| `cs.component.escalation_notifier.tick_heartbeat` | SLA 扫描定时器心跳（典型 60s 一次，复用 RGS-BAS-003 §6 告警推送通道） | 极低（1／分钟／节点） | release 必出（100% 强制全采样） | 含 `tick_id` ／ `scanned_ticket_count` ／ `approaching_sla_count`；约 260B／条 |
| `cs.component.admin_audit_link_ready` | 与 `AdminService` 审计通道建立（处置执行的**唯一**入口，§2.1） | 启动 1 次 | release 必出（100% 强制全采样） | 含 `link_id` ／ `channel_kind`（gRPC stream／poll）；约 220B／条 |
| `cs.component.admin_audit_link_dropped` | 与 `AdminService` 审计通道断开（影响"绕过 AdminService"检测） | 极少 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `link_id` ／ `disconnect_reason` ／ `last_heartbeat_at`；约 280B／条 |
| `cs.component.ticket_service.shutdown_completed` | `TicketService` 优雅关闭，工单上下文已保存（无未提交状态） | 每节点关闭 1 次 | release 必出（100% 强制全采样） | 含 `node_id` ／ `pending_ticket_count` ／ `shutdown_kind`（SIGTERM／HPA scale-in）；约 260B／条 |
| `cs.component.debug.boundary_dag_dump` | 跨组件依赖图 dump（`TicketService` ↔ `EscalationNotifier` ↔ `AdminService`） | 启动 1 次 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-3KB／条（release 剔除，零运行时开销） |
| `cs.component.debug.bridge_invocation_latency` | 组件间桥接调用耗时（微秒级，如 `TicketService` → `AdminService`） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 200B／条（release 剔除） |
| `cs.component.debug.escalation_tick_simulation` | SLA 升级判定的时间推进模拟 dump（用于测试 SLA 边界） | 极低（CI 测试） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `cs.component.debug.boundary_dag_dump` 在多节点集群下可能 3KB+ —— release build 完全剔除，避免 `RUST_LOG=debug` 误开时撑爆生产日志通道
- `cs.component.escalation_notifier.tick_heartbeat` 是**生产事件**（per BAS-004 §4.4 release 必出宏清单"业务关键事件"）—— release 必出 + 强制全采样，便于 SRE 按 `node_id` 维度聚合 SLA 扫描存活率
- `cs.component.admin_audit_link_dropped` 是**安全事件**（`AdminService` 是处置执行唯一入口，通道断开即失去"绕过检测"）—— release 必出 + `warn!` 强制全采样，不挂 `#[cfg]`

## 2.2 数据模型（逻辑字段）

`SupportTicket`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `ticket_id` | uuid | 唯一标识 |
| `player_id` | 玩家ID | 提交者 |
| `category` | enum(`ban_appeal`／`item_anomaly`／`payment_issue`／`other`) | 问题类型 |
| `state` | enum(`待受理`／`处理中`／`待玩家补充信息`／`已解决`／`已驳回`) | 状态机 |
| `sla_deadline` | timestamp | 依`category`分级计算 |
| `resolution_summary` | string，可选 | 关闭时的处理结论摘要（FR-SUP-005） |
| `admin_action_ref` | 可选，引用`AdminService`操作记录ID | 若处理涉及账号状态变更，关联对应的执行记录，**不**在本表直接存储执行结果 |
| `dedup_key` | `player_id`+`category`+滚动时间窗口哈希 | FR-SUP-007去重字段，同构复用RGS-REQ-017 FR-GSM-033的去重思想 |
| `created_at` | timestamp | 提交时间，参与SLA计时起点 |

索引/约束：`ticket_id`为主键；`(player_id, state)`复合索引支撑FR-SUP-006"玩家查询自己的工单列表"；`(state, sla_deadline)`复合索引支撑`TicketEscalationNotifier`定时扫描临近/超过SLA的工单；`(dedup_key)`唯一索引（非强制拒绝，命中时提示"检测到相似工单"供玩家选择合并或继续新建，区别于RGS-BAS-014举报去重的强制不计数——工单去重是**提示**而非**拒绝**，因为申诉场景玩家可能确实有多个独立诉求）。

### 2.2 本功能日志设计

本节覆盖**`SupportTicket` 数据模型 schema 事件**的观察点——`SupportTicket` 表本身是"逻辑字段定义"（无运行时事件），但 DDL 部署／索引创建／`dedup_key` 哈希计算／跨文档字段同步四类治理事件产生 release 必出事件。**`dedup_key` 唯一索引命中为提示而非拒绝**（per §2.2 设计要点，区别于举报去重）—— 命中时 release 必出 + 强制全采样，便于客服合并处理；`player_id` 在所有事件中均**不**记录明文（哈希化 per BAS-004 v0.3 §5.1）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.schema.support_ticket.ddl_applied` | `SupportTicket` 表 DDL 部署（首次部署或迁移） | 极低（迁移级） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `version` ／ `migration_id` ／ `affected_table`（`SupportTicket`）；约 240B／条 |
| `cs.schema.support_ticket.index_created` | 复合索引 `(player_id, state)` ／ `(state, sla_deadline)` ／ 唯一索引 `(dedup_key)` 任一项创建 | 极低（迁移级） | release 必出（100% 强制全采样） | 含 `index_name` ／ `index_kind`（composite／unique）；约 220B／条 |
| `cs.schema.support_ticket.dedup_key_computed` | `dedup_key = player_id + category + 滚动时间窗口哈希` 计算完成（FR-SUP-007 落地） | 每次工单提交 | release 必出（100% 强制全采样，FR-SUP-007 关键事件） | `player_id` 不明文（哈希化 per §5.1）／ `category` ／ `window_kind`；约 250B／条 |
| `cs.schema.support_ticket.dedup_key_collision.prompted` | `(dedup_key)` 唯一索引命中，**提示**玩家（不强制拒绝，§2.2 索引约束） | 偶发 | release 必出（100% 强制全采样） | 含 `existing_ticket_id` ／ `new_player_id_hash` ／ `category` ／ `player_decision`（merge／continue_new）；约 280B／条 |
| `cs.schema.support_ticket.uniqueness_violation.detected` | `(dedup_key)` 唯一索引真冲突（非"提示"语义，误改代码时触发） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 级别） | 含 `conflicting_ticket_id` ／ `new_player_id_hash` ／ `category` ／ `expected_behavior` ／ `actual_behavior`；约 320B／条 |
| `cs.schema.support_ticket.field_added` | 既有 `SupportTicket` 表新增字段（schema 演进） | 极低（迁移级） | release 必出（100% 强制全采样） | 含 `field_name` ／ `field_type` ／ `migration_id`；约 240B／条 |
| `cs.schema.support_ticket.field_deprecated` | 既有字段标记 deprecated（保留读权限，禁写） | 极低（迁移级） | release 必出（100% 强制全采样） | 含 `field_name` ／ `deprecation_phase` ／ `removal_target_version`；约 260B／条 |
| `cs.schema.support_ticket.debug.ddl_dump` | `SupportTicket` 完整 DDL dump（含全部约束／索引） | 极低（CI 验证） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-5KB／条（release 剔除） |
| `cs.schema.support_ticket.debug.index_plan_dump` | 索引使用情况 EXPLAIN dump（用于 dedup_key 命中性能排查） | 极低（SRE 排查） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 剔除） |
| `cs.schema.support_ticket.debug.dedup_key_window_collision_analysis` | 滚动时间窗口内 `dedup_key` 冲突详细分析（含来源玩家哈希／分类／窗口） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `cs.schema.support_ticket.uniqueness_violation.detected` 是**阻断级**信号（`dedup_key` 命中应"提示"而非"拒绝"，触发即代码缺陷）—— release 必出 + `error!` 强制全采样，不挂 `#[cfg]`
- `cs.schema.support_ticket.dedup_key_collision.prompted` 是**正常业务路径**（FR-SUP-007 设计要求）—— release 必出 + 强制全采样，便于 SRE 按 `category` 维度聚合"提示-合并"率
- `cs.schema.support_ticket.dedup_key_computed` 中 `player_id` 必须哈希化（per BAS-004 §5.1）—— 严禁明文 `player_id` 入日志
- `cs.schema.support_ticket.debug.ddl_dump` 在大型表下可能 5KB+ —— release 完全剔除

### 2.3 状态机迁移条件（FR-SUP-002落地）

| 迁移 | 触发条件 | 拒绝条件 |
|---|---|---|
| `待受理 → 处理中` | 客服/GM认领工单 | 工单已关闭（`已解决`/`已驳回`） |
| `处理中 → 待玩家补充信息` | 客服标记需要更多信息 | — |
| `待玩家补充信息 → 处理中` | 玩家补充回复 | 超过配置的静默期（默认7天）自动转`已驳回`，防止工单无限期悬挂 |
| `处理中 → 已解决` | 客服记录`resolution_summary`并关闭 | `resolution_summary`为空（FR-SUP-005强制关闭时必须留痕） |
| `处理中 → 已驳回` | 客服判定不成立并记录理由 | 同上 |

### 2.3 本功能日志设计

本节覆盖**工单状态机迁移 + 玩家-客服对话内容**的观察点——**合规审计 + 隐私双重约束**场景。状态机迁移（创建／分配／处理／关闭）→ release 必出 + 强制全采样（FR-SUP-001〜005 强约束），玩家-客服对话内容 → 脱敏（邮箱／手机哈希化 per BAS-004 v0.3 §5.1），**支付凭证／卡号 → 禁止记录**（per BAS-004 §5.1 黑名单，SDK 层拦截丢弃）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.ticket.lifecycle.submitted` | 玩家提交工单（新 ticket 创建，FR-SUP-001） | 偶发（玩家驱动） | release 必出（100% 强制全采样，FR-SUP-001 关键事件） | 含 `ticket_id` ／ `player_id`（明文允许 per §5.1）／ `category` ／ `submitted_at`；约 220B／条 |
| `cs.ticket.lifecycle.claimed` | 客服／GM 认领工单（`待受理` → `处理中`，FR-SUP-002） | 偶发 | release 必出（100% 强制全采样，FR-SUP-002 关键事件） | 含 `ticket_id` ／ `agent_id` ／ `claimed_at`；约 240B／条 |
| `cs.ticket.lifecycle.player_response_requested` | `处理中` → `待玩家补充信息` | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `agent_id` ／ `silent_period_days`；约 220B／条 |
| `cs.ticket.lifecycle.player_response_received` | 玩家补充回复（`待玩家补充信息` → `处理中`） | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `player_id` ／ `response_at`；约 220B／条 |
| `cs.ticket.lifecycle.resolved` | `处理中` → `已解决`（`resolution_summary` 留痕，FR-SUP-005） | 偶发 | release 必出（100% 强制全采样，FR-SUP-005 关键事件） | 含 `ticket_id` ／ `agent_id` ／ `resolution_summary_length`（限 200 字，已脱敏）；约 380B／条 |
| `cs.ticket.lifecycle.rejected` | `处理中` → `已驳回` | 偶发 | release 必出（100% 强制全采样，FR-SUP-005 关键事件） | 含 `ticket_id` ／ `agent_id` ／ `rejection_reason`；约 300B／条 |
| `cs.ticket.lifecycle.auto_rejected.timeout` | 超过静默期（默认 7 天）自动转 `已驳回`（防无限期悬挂） | 偶发 | release 必出（100% 强制全采样，治理事件必出） | 含 `ticket_id` ／ `silent_period_days` ／ `auto_rejected_at` ／ `last_response_at`；约 280B／条 |
| `cs.ticket.transition.rejected.invalid` | 非法迁移（如工单已关闭时尝试认领，§2.3 状态机迁移表拒绝条件） | 配置错／攻击 | release 必出（100% 强制全采样，`error!` 级别） | 含 `ticket_id` ／ `attempted_transition` ／ `current_state` ／ `rejection_reason`；约 320B／条 |
| `cs.ticket.transition.rejected.empty_resolution` | `已解决` 迁移时 `resolution_summary` 为空（FR-SUP-005 强制留痕，§2.3 拒绝条件） | 配置错 | release 必出（100% 强制全采样，`error!` 级别） | 含 `ticket_id` ／ `agent_id` ／ `attempted_transition`；约 240B／条 |
| `cs.ticket.conversation.message_logged` | 玩家-客服对话内容落库（FR-SUP-006） | 偶发 | release 必出（100% 强制全采样，FR-SUP-006 合规审计需要） | **不**记录原文；脱敏后：哈希化邮箱／手机 + 消息长度 + 消息分类 + 附件标记；约 300B／条 |
| `cs.ticket.conversation.redaction_applied` | 邮箱／手机号哈希化脱敏触发（per BAS-004 v0.3 §5.1） | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `redaction_kind`（email／phone）／ `redacted_count`；约 220B／条 |
| `cs.ticket.conversation.payment_credential_blocked` | 玩家-客服对话中含支付凭证／卡号，SDK 黑名单拦截丢弃（per BAS-004 v0.3 §5.1） | 极少（误发／攻击） | release 必出（100% 强制全采样，`warn!` 级别） | 含 `ticket_id` ／ `redaction_kind`（card_number／cvv／credential_token）；**无明文**；约 240B／条 |
| `cs.ticket.conversation.attachment_scanned` | 玩家-客服对话附件（图片／PDF）含敏感内容扫描 | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `attachment_id` ／ `attachment_kind` ／ `scan_result`（clean／flagged）；约 280B／条 |
| `cs.ticket.conversation.admin_action_ref_logged` | `admin_action_ref` 字段写入（关联 `AdminService` 操作记录，§2.2 字段定义） | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `admin_action_id` ／ `action_kind`；约 220B／条 |
| `cs.ticket.debug.conversation_payload_dump` | 玩家-客服对话原文 dump（PII 重度，**严禁**进 release） | 极低（审计／法务取证） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB／条（release 完全剔除，零运行时开销） |
| `cs.ticket.debug.state_machine_transition_full_trace` | 状态机迁移全链路 trace（含每步拒绝原因） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除） |
| `cs.ticket.debug.dedup_key_collision_detail` | `dedup_key` 冲突的来源玩家／时间窗口／分类详情 | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 300B／条（release 剔除） |
| `cs.ticket.debug.conversation_pii_scan_match_dump` | PII 扫描命中的原文片段 dump（仅 debug build 留存用于规则迭代） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 完全剔除，避免 PII 泄漏） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §5.1 双重约束）：
- `cs.ticket.conversation.message_logged` 是**合规审计关键事件**（FR-SUP-006 强制留痕）—— release 必出 + 强制全采样，不挂 `#[cfg]`
- `cs.ticket.conversation.payment_credential_blocked` 是**安全事件**（支付凭证拦截）—— release 必出 + `warn!` 强制全采样，**绝不**记录明文卡号／CVV／token
- `cs.ticket.transition.rejected.empty_resolution` 是**FR-SUP-005 强制留痕**保障—— release 必出 + `error!` 强制全采样
- `cs.ticket.debug.conversation_payload_dump` **PII 重度**—— release build 完全剔除，避免 `RUST_LOG=debug` 误开时泄漏玩家对话内容
- `cs.ticket.debug.conversation_pii_scan_match_dump` 同样 PII 重度—— release 完全剔除
- 治理事件清单（强制 release 必出）：`lifecycle.submitted` ／ `lifecycle.claimed` ／ `lifecycle.resolved` ／ `lifecycle.rejected` ／ `lifecycle.auto_rejected.timeout` ／ `conversation.message_logged` ／ `conversation.payment_credential_blocked` ／ `transition.rejected.*` 共 8 个治理／合规信号必须 production 可见

### 2.4 SLA分级基准（FR-SUP-003，TBD-SUP-001评审前的默认建议值）

| `category` | 首次响应SLA（默认建议值，最终以TBD-SUP-001评审结果为准） | 升级提醒触发点 |
|---|---|---|
| `payment_issue` | p95 < 4小时 | 超过SLA的80%时长即触发`TicketEscalationNotifier`提前预警 |
| `ban_appeal` | p95 < 24小时 | 同上 |
| `item_anomaly` | p95 < 24小时 | 同上 |
| `other` | p95 < 48小时 | 同上 |

### 2.4 本功能日志设计

本节覆盖**SLA 分级基准 + 升级提醒**的观察点——SLA 基准本身是文档产物（无运行时），但**SLA 警告／超时** → `warn!` 强制全采样（per BAS-009 治理事件必出模式 + BAS-004 v0.3 §4.4 release 必出宏清单"异常但已处理"行），**升级通知 / 实际 p95 违反基准** → release 必出，便于 SRE 按 `category` 维度追踪"哪些类目长期不达标"。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.sla.warning.approaching` | 任意工单超过 SLA 80% 时长（§2.4 升级提醒触发点，提前预警） | 偶发（SLA 接近） | release 必出（100% 强制全采样，`warn!` 强制全采样，per BAS-004 v0.3 §4.4 异常但已处理） | 含 `ticket_id` ／ `category` ／ `sla_deadline` ／ `progress_ratio`（0.8+）／ `remaining_seconds`；约 280B／条 |
| `cs.sla.warning.breached` | SLA 超过 100% 触发升级（4h／24h／24h／48h per category，§2.4 表） | 偶发（SLA 超时） | release 必出（100% 强制全采样，`warn!` 强制全采样） | 含 `ticket_id` ／ `category` ／ `expected_p95_seconds` ／ `actual_age_seconds` ／ `breach_kind`（payment_issue／ban_appeal／item_anomaly／other）；约 340B／条 |
| `cs.sla.escalation.notified` | 升级通知已发（走 RGS-BAS-003 §6 告警推送通道 + §6.3 复用） | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `notification_channel`（Webhook／email／IM）／ `notified_role`（senior_agent／supervisor）；约 260B／条 |
| `cs.sla.escalation.escalated_to_senior` | 升级到资深客服／主管（RGS-BAS-003 §6.3 复用） | 偶发 | release 必出（100% 强制全采样） | 含 `ticket_id` ／ `escalated_to` ／ `escalation_reason` ／ `original_agent_id`；约 280B／条 |
| `cs.sla.audit.grade_violation` | 实际响应时间超过对应 `category` 的 p95 目标（4h／24h／24h／48h），周级统计检出 | 偶发（周级） | release 必出（100% 强制全采样，`warn!` 级别） | 含 `category` ／ `target_p95_seconds` ／ `actual_p95_seconds` ／ `window_kind`（daily／weekly）；约 320B／条 |
| `cs.sla.audit.breach_rate_exceeded` | 某 `category` 的 SLA 违反率超过阈值（典型 5%，具体值 TBD-SUP-001 评审） | 偶发（周级） | release 必出（100% 强制全采样，`warn!` 级别，触发运营告警） | 含 `category` ／ `breach_rate` ／ `threshold` ／ `window_kind`；约 240B／条 |
| `cs.sla.baseline.review_registered` | SLA 分级基准评审结果登记（TBD-SUP-001 决议） | 极低（决议级） | release 必出（100% 强制全采样） | 含 `category` ／ `old_p95_seconds` ／ `new_p95_seconds` ／ `tbd_ref`（TBD-SUP-001）／ `decider_id`；约 320B／条 |
| `cs.sla.silent_period.exceeded` | 玩家超过静默期（默认 7 天，§2.3 状态机迁移表）未补充信息，触发自动 `已驳回` | 偶发 | release 必出（100% 强制全采样，治理事件必出） | 含 `ticket_id` ／ `category` ／ `silent_period_days` ／ `last_player_response_at`；约 280B／条 |
| `cs.sla.debug.p95_calculation_breakdown` | p95 响应时间计算的逐项明细（bucket 分布／样本数） | 极低（周级统计） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB／条（release 剔除） |
| `cs.sla.debug.ticket_age_distribution_dump` | 全部未关闭工单的年龄分布 dump（按 `category` 分组） | 极低（SRE 排查） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB／条（release 剔除） |
| `cs.sla.debug.category_baseline_change_history` | SLA 分级基准变更历史（TBD-SUP-001 评审前后对照） | 极低（决议级） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 300B／条（release 剔除） |
| `cs.sla.debug.escalation_chain_simulation` | 升级链推演 dump（哪条工单会按什么路径升级） | 极低（CI 测试） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §6.2 强制全采样白名单）：
- `cs.sla.warning.approaching` ／ `warning.breached` 是**异常但已处理**事件（per BAS-004 v0.3 §4.4 释放必出宏清单对应行）—— release 必出 + `warn!` 强制全采样，不挂 `#[cfg]`
- `cs.sla.audit.grade_violation` ／ `breach_rate_exceeded` 是**SRE 运营关注信号**—— release 必出 + `warn!` 强制全采样，便于周报聚合
- `cs.sla.baseline.review_registered` 是**重大治理事件**（TBD-SUP-001 决议）—— release 必出 + 强制全采样，便于 SLA 历年变更审计
- `cs.sla.debug.ticket_age_distribution_dump` 大型项目下可能 5KB+ —— release 完全剔除
- 治理事件清单（强制 release 必出）：`sla.warning.approaching` ／ `sla.warning.breached` ／ `sla.escalation.notified` ／ `sla.escalation.escalated_to_senior` ／ `sla.audit.grade_violation` ／ `sla.audit.breach_rate_exceeded` ／ `sla.baseline.review_registered` ／ `sla.silent_period.exceeded` 共 8 个 SLA 治理信号必须 production 可见

---

# 3. 支付对账数据模型与时序

## 3.1 数据模型

`PaymentOrder`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `order_id` | uuid | 内部订单ID |
| `provider_txn_id` | string，可选 | 支付服务商侧交易ID，对账关联键 |
| `state` | enum(`待支付`／`已支付`／`已发货`／`发货失败`／`待补偿`／`已补偿`) | 状态机 |
| `amount` | decimal | 金额 |
| `updated_at` | timestamp | 最近状态变更时间，供对账任务判断"待支付"是否超过合理时长（异常分支参考） |
| `payment_channel` | enum(`platform_iap`／`direct_gateway`) | 区分平台内购与本文档既定直连支付，决定对账/退款处理走哪条子流程（RGS-REQ-023 FR-PLT-005跨文档扩展字段，本表为权威定义处，2026-08-16纳入） |
| `platform_type` | enum，可选（仅`payment_channel=platform_iap`时非空） | `app_store`／`google_play`（同上扩展，RGS-BAS-020§2.5） |
| `platform_environment` | enum(`sandbox`／`production`)，可选（仅平台内购适用） | 沙盒/生产环境标记，须与收据校验时平台返回的环境一致（同上扩展，RGS-BAS-020§2.5 FR-PLT-004） |
| `refund_status` | enum(`none`／`refunded`／`clawback_pending`／`clawback_done`)，默认`none` | 退款处理状态（同上扩展，RGS-BAS-020§2.5 FR-PLT-003） |

> **跨文档字段扩展声明**：`payment_channel`〜`refund_status`四字段由RGS-REQ-023/BAS-020（平台内购合规）在本表基础上追加，**本表是`PaymentOrder`的唯一权威字段清单**——任何文档若需扩展本表结构，**必须**同步在此处登记（同RGS-BAS-010§7.1新增检查项"跨限界上下文表结构扩展须同步更新原表文档"），不得仅在扩展方文档单向记录导致字段清单分散、失去单一真相来源。

索引/约束：`order_id`为主键；`(provider_txn_id)`唯一索引（允许NULL，`待支付`阶段可能尚无服务商侧交易ID）——该唯一索引是幂等键（NFR-SUP-004）与对账关联键的双重保证，比对逻辑**必须**以此索引做`UPSERT`/条件更新而非应用层先查后写，避免RSK-SUP-002"比对条件写反"类缺陷绕过数据库层面的唯一性保护；`(state, updated_at)`复合索引支撑异常分支扫描长时间停留在非终态的订单；`(platform_type, provider_txn_id)`复合唯一索引（RGS-BAS-020§2.5扩展）确保跨平台交易标识不产生误关联。

### 3.1 本功能日志设计

本节覆盖**`PaymentOrder` 数据模型 schema 事件**的观察点——`PaymentOrder` 表本身是"逻辑字段定义"（无运行时），但 DDL 部署／索引创建／幂等键注册／**跨文档字段同步**（per §3.1 跨文档字段扩展声明）四类治理事件产生 release 必出事件。**跨文档字段同步是阻断级**（RGS-BAS-010 §7.1 双向同步检查）—— release 必出 + 强制全采样 + 阻断级告警。**幂等键唯一性违反**（NFR-SUP-004）→ release 必出 + `error!` 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.payment.schema.payment_order.ddl_applied` | `PaymentOrder` 表 DDL 部署（首次部署或迁移） | 极低（迁移级） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `version` ／ `migration_id` ／ `affected_table`（`PaymentOrder`）；约 240B／条 |
| `cs.payment.schema.payment_order.index_created` | 唯一索引 `(provider_txn_id)` ／ 复合索引 `(state, updated_at)` ／ 复合唯一索引 `(platform_type, provider_txn_id)` 任一项创建（§3.1 索引约束） | 极低（迁移级） | release 必出（100% 强制全采样） | 含 `index_name` ／ `index_kind`（unique／composite）／ `nullability`；约 260B／条 |
| `cs.payment.schema.idempotency_key_registered` | `provider_txn_id` 幂等键注册（NFR-SUP-004 双重保证：幂等键 + 对账关联键） | 极低 | release 必出（100% 强制全采样） | 含 `provider_txn_id_hash`（明文按 §5.1 脱敏规则处理）／ `idempotency_role`；约 220B／条 |
| `cs.payment.schema.cross_table_field_sync_applied` | `payment_channel` ／ `platform_type` ／ `platform_environment` ／ `refund_status` 四字段跨文档同步（§3.1 跨文档字段扩展声明 + RGS-BAS-010 §7.1） | 极低（决议级） | release 必出（100% 强制全采样，治理事件必出） | 含 `field_name` ／ `source_bas`（BAS-020）／ `target_bas`（BAS-016）／ `sync_kind`（add／update／deprecate）；约 320B／条 |
| `cs.payment.schema.cross_table_field_sync_failed` | 跨文档字段同步未在两表同时落地（仅在扩展方文档单向记录，违反 RGS-BAS-010 §7.1 单一真相来源原则） | 极少（CI 检出） | release 必出（100% 强制全采样，`error!` 级别，触发阻断告警） | 含 `field_name` ／ `source_bas` ／ `target_bas` ／ `expected_in_target` ／ `actual_in_target`；约 360B／条 |
| `cs.payment.schema.uniqueness_violation.provider_txn_id` | `(provider_txn_id)` 唯一索引真冲突（NFR-SUP-004 幂等键保护，§3.1 索引约束） | 极少（重放攻击） | release 必出（100% 强制全采样，`error!` 级别） | 含 `provider_txn_id_hash` ／ `existing_order_id` ／ `new_request_id`；约 280B／条 |
| `cs.payment.schema.uniqueness_violation.platform_collision` | `(platform_type, provider_txn_id)` 复合唯一索引冲突（跨平台误关联，§3.1 索引约束） | 极少（重放／误关联） | release 必出（100% 强制全采样，`error!` 级别） | 含 `platform_type` ／ `provider_txn_id_hash` ／ `existing_order_id` ／ `conflicting_platform`；约 320B／条 |
| `cs.payment.schema.field_added` | 既有 `PaymentOrder` 表新增字段（schema 演进） | 极低（迁移级） | release 必出（100% 强制全采样） | 含 `field_name` ／ `field_type` ／ `migration_id` ／ `cross_table_sync_required`（布尔）；约 300B／条 |
| `cs.payment.schema.field_deprecated` | 既有字段标记 deprecated（保留读权限，禁写） | 极低（迁移级） | release 必出（100% 强制全采样） | 含 `field_name` ／ `deprecation_phase` ／ `removal_target_version`；约 260B／条 |
| `cs.payment.schema.debug.ddl_dump` | `PaymentOrder` 完整 DDL dump（含全部约束／索引） | 极低（CI 验证） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-5KB／条（release 剔除） |
| `cs.payment.schema.debug.idempotency_key_window_dump` | 幂等键时间窗口内重复 key 列表 dump（用于 NFR-SUP-004 排查） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-2KB／条（release 剔除） |
| `cs.payment.schema.debug.cross_table_field_diff` | 跨文档字段同步前后两表字段清单 diff（BAS-016 vs BAS-020） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + RGS-BAS-010 §7.1）：
- `cs.payment.schema.cross_table_field_sync_failed` 是**阻断级**信号（违反单一真相来源原则）—— release 必出 + `error!` 强制全采样，触发 CI 阻断告警
- `cs.payment.schema.uniqueness_violation.*` 是**重放攻击防护信号**（NFR-SUP-004 幂等保证）—— release 必出 + `error!` 强制全采样
- `cs.payment.schema.cross_table_field_sync_applied` 是**重大治理事件**（跨 BAS 同步）—— release 必出 + 强制全采样，便于审计回溯每次字段扩展
- `cs.payment.schema.debug.ddl_dump` 在大型表下可能 5KB+ —— release 完全剔除
- 治理事件清单（强制 release 必出）：`ddl_applied` ／ `index_created` ／ `idempotency_key_registered` ／ `cross_table_field_sync_applied` ／ `cross_table_field_sync_failed` ／ `uniqueness_violation.*` ／ `field_added` ／ `field_deprecated` 共 9 个 schema 治理信号必须 production 可见

## 3.2 对账批处理时序

```
定时任务触发（周期见NFR-SUP-002）
  → ReconciliationJob拉取支付服务商侧对账文件/API（时间窗口内的交易记录）
  → 与内部PaymentOrder按provider_txn_id关联比对
  → 发现"服务商侧已支付但内部订单未达已发货"的记录 → 标记为待补偿
  → 待补偿记录逐条校验金额是否超过TBD-SUP-002阈值：
      未超阈值 → 复用FR-EC-003确定请求路径自动发放 → 状态迁移为已补偿
      超阈值 → 生成SupportTicket（category=payment_issue）转人工复核，不自动发放
  → 全部对账动作记录审计日志（复用RGS-BAS-003§7）
```

> 幂等键：以`provider_txn_id`为幂等键，同一笔服务商交易记录在重复对账批次中不重复触发补偿（NFR-SUP-004）。

### 3.2 本功能日志设计

本节覆盖**对账批处理全链路**的观察点——**支付对账（订单／退款／差异）→ release 必出 + 强制全采样**（NFR-EC 合规审计需要完整链路），**资产结算补偿**（FR-EC-003 复用）属生产关键事件 → release 必出，**双重布尔校验**（RSK-SUP-002 防护，§3.3 异常分支）→ release 必出 + 强制全采样 + 快照可追溯。**支付凭证／卡号 → 禁止记录**（per BAS-004 v0.3 §5.1，SDK 层拦截，对账批处理**不**处理支付凭证本身）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.recon.batch.started` | 定时任务触发对账批处理（周期见 NFR-SUP-002） | 极低（1／小时或 N 小时） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2，NFR-EC 合规） | 含 `batch_id` ／ `window_start` ／ `window_end` ／ `trigger_kind`（cron／manual）；约 260B／条 |
| `cs.recon.batch.completed` | 对账批处理正常完成 | 极低 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `matched_count` ／ `mismatch_count` ／ `compensation_count` ／ `latency_ms`；约 320B／条 |
| `cs.recon.batch.failed` | 批处理异常失败（非服务商侧数据问题） | 偶发 | release 必出（100% 强制全采样，`error!` 强制全采样） | 含 `batch_id` ／ `error` ／ `trace_id` ／ `failed_step`；约 340B／条 |
| `cs.recon.provider_file.fetched` | 拉取支付服务商侧对账文件／API 成功 | 极低 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `provider` ／ `record_count` ／ `fetch_latency_ms` ／ `file_hash`；约 300B／条 |
| `cs.recon.compare.matched` | 服务商记录与内部 `PaymentOrder` 一致（`provider_txn_id` 关联成功） | 极低 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `order_id` ／ `provider_txn_id_hash`；约 220B／条 |
| `cs.recon.compare.mismatch.found` | 发现 "服务商侧已支付但内部订单未达已发货"（待补偿候选，§3.2 时序第 3 步） | 偶发 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `order_id` ／ `provider_txn_id_hash` ／ `mismatch_kind`（state_mismatch／amount_mismatch／missing_local）；约 360B／条 |
| `cs.recon.dual_boolean_check.passed` | "待补偿" 判定前双重布尔校验通过（per §3.3 RSK-SUP-002 防护：① `provider_side_paid=true` ② `internal_state NOT IN [已发货, 已补偿]`） | 偶发 | release 必出（100% 强制全采样，RSK-SUP-002 防护可追溯） | 含 `batch_id` ／ `order_id` ／ `condition_1_result` ／ `condition_2_result` ／ `snapshot_id` ／ `evaluated_at`；约 420B／条 |
| `cs.recon.dual_boolean_check.failed` | 双重布尔校验任一不满足，**不**进入"待补偿"流程（RSK-SUP-002 缓解，§3.3 末段） | 极少 | release 必出（100% 强制全采样，`warn!` 强制全采样） | 含 `batch_id` ／ `order_id` ／ `failed_condition`（1=provider_not_paid／2=internal_already_finalized） ／ `snapshot_id`；约 360B／条 |
| `cs.recon.compensation.threshold_evaluated` | 待补偿记录逐条校验金额是否超过 TBD-SUP-002 阈值（§3.2 时序第 4 步） | 偶发 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `order_id` ／ `amount` ／ `threshold` ／ `decision`（auto／support_ticket）；约 280B／条 |
| `cs.recon.compensation.auto_issued` | 复用 FR-EC-003 确定请求路径自动发放（金额未超 TBD-SUP-002 阈值，§3.2 时序第 4 步"未超阈值"分支） | 偶发 | release 必出（100% 强制全采样，资产结算关键事件） | 含 `order_id` ／ `compensation_amount` ／ `compensation_currency` ／ `request_id` ／ `compensation_kind`（道具／货币）；约 300B／条 |
| `cs.recon.compensation.support_ticket_created` | 超阈值转人工复核，创建 `SupportTicket`（`category=payment_issue`，§3.2 时序第 4 步"超阈值"分支） | 偶发 | release 必出（100% 强制全采样） | 含 `order_id` ／ `support_ticket_id` ／ `threshold` ／ `actual_amount` ／ `reviewer_assigned_at`；约 320B／条 |
| `cs.recon.idempotency.duplicate_batch_skipped` | 同 `provider_txn_id` 在重复批次中**不**重复触发补偿（NFR-SUP-004 幂等键保证，§3.2 末段幂等键声明） | 偶发 | release 必出（100% 强制全采样，幂等键保护） | 含 `batch_id` ／ `provider_txn_id_hash` ／ `previous_batch_id` ／ `previous_compensation_id`；约 320B／条 |
| `cs.recon.audit.action_recorded` | 全部对账动作写入审计（§3.2 时序第 5 步，per RGS-BAS-003 §7 审计设计复用） | 极低 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `action_kind` ／ `actor_id` ／ `audit_id`；约 240B／条 |
| `cs.recon.refund.processed` | 退款处理状态变更（`refund_status` 字段从 `none` → `refunded`／`clawback_pending`／`clawback_done`，§3.1 跨文档字段扩展） | 偶发 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `order_id` ／ `old_refund_status` ／ `new_refund_status` ／ `refund_amount` ／ `provider_txn_id_hash`；约 320B／条 |
| `cs.recon.platform_type.disambiguation` | 跨平台交易标识消歧（`platform_type` + `provider_txn_id` 复合唯一索引，§3.1 索引约束） | 偶发 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `provider_txn_id_hash` ／ `platform_type` ／ `disambiguated_order_id`；约 280B／条 |
| `cs.recon.debug.provider_file_raw_dump` | 服务商侧对账文件原始内容 dump（**PII 重度**，仅审计／法务取证用） | 极低（审计／法务取证） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 10-100KB／条（release 完全剔除，零运行时开销） |
| `cs.recon.debug.compare_snapshot_full` | 双重布尔校验的逐条原始快照（含未匹配记录全量） | 偶发（SRE 排查） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 5-20KB／条（release 剔除） |
| `cs.recon.debug.compensation_decision_tree` | 补偿决策树 dump（阈值比较／分支命中） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 剔除） |
| `cs.recon.debug.provider_file_headers_dump` | 服务商侧对账文件 HTTP 响应 headers dump | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-2KB／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §6.2 强制全采样白名单）：
- `cs.recon.dual_boolean_check.passed` 是**RSK-SUP-002 防护可追溯点**（§3.3 末段要求"各自记录比对依据快照"）—— release 必出 + 强制全采样，`snapshot_id` 关联 SRE 排查时拉取 debug-only 原始快照
- `cs.recon.compensation.auto_issued` 是**资产结算关键事件**（每次发放均产生）—— release 必出 + 强制全采样，便于 SRE 按 `order_id` 维度聚合
- `cs.recon.idempotency.duplicate_batch_skipped` 是**幂等键保护信号**（NFR-SUP-004）—— release 必出 + 强制全采样，便于排查重放攻击
- `cs.recon.debug.provider_file_raw_dump` **PII 重度**（可能含玩家支付凭证片段）—— release 完全剔除，避免 PII 泄漏
- `cs.recon.refund.processed` ／ `platform_type.disambiguation` 是**跨文档字段扩展产物**（§3.1 跨文档字段扩展声明四字段）—— release 必出，便于审计 BAS-016 ↔ BAS-020 字段同步落地
- 治理事件清单（强制 release 必出）：`batch.*` ／ `provider_file.fetched` ／ `compare.*` ／ `dual_boolean_check.*` ／ `compensation.*` ／ `idempotency.*` ／ `audit.action_recorded` ／ `refund.*` ／ `platform_type.*` 共 13 个对账／合规／资产结算信号必须 production 可见

### 3.3 异常分支

```
支付服务商侧对账文件/API本次拉取失败或数据不完整（服务商侧临时故障/延迟）
  → ReconciliationJob本轮跳过，记录告警（复用RGS-BAS-003§6），不将"未取到数据"误判为"服务商侧无交易"
  → 下一周期正常拉取时自动补齐窗口（对账窗口须与上次成功窗口重叠，避免因单次失败产生的比对空档遗漏掉单）

比对条件疑似写反（RSK-SUP-002缓解）：ReconciliationJob在判定"待补偿"前须同时满足①provider_txn_id在服务商侧记录中状态为"支付成功"②本地PaymentOrder.state不在(已发货、已补偿)集合内，两个条件均需显式布尔校验并各自记录比对依据快照，代码评审须逐行核对条件方向未写反（§4.2检查项）
```

### 3.3 本功能日志设计

本节覆盖**对账异常分支**的观察点——**支付对账（订单／退款／差异）→ release 必出 + 强制全采样**（NFR-EC 合规）。**服务商侧不可用** → 触发告警（per RGS-BAS-003 §6 告警推送通道，NFR-OP-005 24×365）。**RSK-SUP-002 "比对条件写反"** → release 必出 + `error!` 强制全采样（阻断级）。**对账窗口重叠**（防单次失败产生比对空档）→ release 必出 + 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.recon.provider.fetch_failed` | 服务商侧对账文件／API 拉取失败（临时故障／延迟，§3.3 异常分支首段） | 偶发 | release 必出（100% 强制全采样，`warn!` 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `provider` ／ `error_kind`（timeout／4xx／5xx／connection_reset）／ `retry_count` ／ `next_retry_at`；约 320B／条 |
| `cs.recon.provider.fetch_skipped` | 本轮跳过（**不**将"未取到数据"误判为"服务商侧无交易"，§3.3 异常分支首段） | 偶发 | release 必出（100% 强制全采样，`warn!` 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `skip_reason` ／ `next_retry_at` ／ `skipped_window`；约 280B／条 |
| `cs.recon.provider.window_overlap_extended` | 对账窗口与上次成功窗口重叠（避免单次失败产生比对空档遗漏掉单，§3.3 异常分支末段） | 偶发 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `current_window_start` ／ `previous_successful_window_end` ／ `overlap_seconds` ／ `extension_reason`；约 320B／条 |
| `cs.recon.provider.unavailable_extended` | 服务商侧持续不可用，达到告警阈值（典型 15 分钟，NFR-OP-005 24×365 触发 P1 告警） | 极少 | release 必出（100% 强制全采样，`error!` 强制全采样，触发告警通道 RGS-BAS-003 §6） | 含 `provider` ／ `unavailable_duration_seconds` ／ `consecutive_failure_count` ／ `last_successful_batch_id` ／ `alert_id`；约 360B／条 |
| `cs.recon.provider.recovered` | 服务商侧从不可用恢复，下一批对账正常完成 | 极少 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `provider` ／ `unavailable_duration_seconds` ／ `recovery_batch_id` ／ `missed_window_count`；约 300B／条 |
| `cs.recon.compare.reversed_condition.detected` | 比对条件疑似写反（RSK-SUP-002，"待补偿" 判定条件方向错误，§3.3 末段防护目标） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `batch_id` ／ `detected_inverted_condition`（1=provider_not_paid_check_reversed／2=internal_state_check_reversed） ／ `affected_record_count` ／ `snapshot_id`；约 380B／条 |
| `cs.recon.compare.inconsistency.escalated_to_human` | 双重布尔校验通过后，人工发现仍异常，转人工复核（双道防线 §3.2 时序第 4 步"超阈值"分支路径） | 极少 | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `order_id` ／ `manual_reviewer_id` ／ `inconsistency_kind` ／ `escalation_reason`；约 300B／条 |
| `cs.recon.compare.missing_local_record.escalated` | 服务商侧已支付，但本地 `PaymentOrder` 完全缺失（可能是支付前订单创建失败或数据库回滚） | 极少 | release 必出（100% 强制全采样，`error!` 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `provider_txn_id_hash` ／ `provider_side_amount` ／ `provider_side_paid_at` ／ `escalation_ticket_id`；约 340B／条 |
| `cs.recon.compare.duplicate_provider_record` | 服务商侧对账文件中同一 `provider_txn_id` 出现多次（NFR-SUP-004 幂等键应保证但仍需检出） | 极少 | release 必出（100% 强制全采样，`warn!` 级别） | 含 `batch_id` ／ `provider_txn_id_hash` ／ `duplicate_count` ／ `first_seen_at` ／ `last_seen_at`；约 300B／条 |
| `cs.recon.compare.amount_mismatch.detected` | 服务商侧金额与本地 `PaymentOrder.amount` 不一致（超出合理误差） | 极少 | release 必出（100% 强制全采样，`error!` 强制全采样，NFR-EC 合规） | 含 `batch_id` ／ `order_id` ／ `provider_amount` ／ `local_amount` ／ `mismatch_ratio` ／ `support_ticket_id`；约 320B／条 |
| `cs.recon.debug.fetch_failure_evidence` | 拉取失败时的服务商侧原始响应 dump（headers／cookies／SSL 状态） | 偶发（SRE 排查） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 5-20KB／条（release 完全剔除，**不**记录敏感 headers） |
| `cs.recon.debug.unmatched_provider_records_full` | 服务商侧有但本地无的完整记录 dump（含明文 `provider_txn_id`） | 偶发（SRE 排查） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 10-50KB／条（release 完全剔除） |
| `cs.recon.debug.reversed_condition_trace` | 比对条件写反的代码路径 trace（含执行栈） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 剔除） |
| `cs.recon.debug.window_overlap_visualization` | 对账窗口重叠区间的可视化 dump（mermaid gantt） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-2KB／条（release 剔除） |
| `cs.recon.debug.provider_unavailable_timeline` | 服务商侧不可用时间线 dump（含每次重试的精确时间） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + RGS-BAS-003 §6 告警联动）：
- `cs.recon.provider.unavailable_extended` 是**P1 告警触发信号**（NFR-OP-005 24×365）—— release 必出 + `error!` 强制全采样，**不**挂 `#[cfg]`
- `cs.recon.compare.reversed_condition.detected` 是**RSK-SUP-002 阻断级信号**（比对条件写反即资金损失风险）—— release 必出 + `error!` 强制全采样 + 触发应急响应通道
- `cs.recon.compare.missing_local_record.escalated` 是**资产安全事件**（玩家已付款但无订单）—— release 必出 + `error!` 强制全采样，自动生成 `SupportTicket`（`category=payment_issue`）转人工
- `cs.recon.compare.amount_mismatch.detected` 是**财务合规事件**—— release 必出 + `error!` 强制全采样，财务团队介入
- `cs.recon.debug.fetch_failure_evidence` **可能含敏感 headers**（cookie／auth）—— release 完全剔除，避免密钥泄漏
- `cs.recon.debug.unmatched_provider_records_full` **可能含 PII 关联字段**—— release 完全剔除
- 治理事件清单（强制 release 必出）：`provider.fetch_failed` ／ `provider.fetch_skipped` ／ `provider.window_overlap_extended` ／ `provider.unavailable_extended` ／ `provider.recovered` ／ `compare.reversed_condition.detected` ／ `compare.inconsistency.escalated_to_human` ／ `compare.missing_local_record.escalated` ／ `compare.duplicate_provider_record` ／ `compare.amount_mismatch.detected` 共 10 个异常／阻断级信号必须 production 可见

---

# 4. 标准化检查清单

## 4.1 上线前检查清单

- [ ] 工单状态机非法迁移拒绝验证通过
- [ ] SLA分级（TBD-SUP-001）已与客服/运营团队评审确定
- [ ] 对账批处理故障注入试验：模拟服务商侧记录延迟到达，验证不产生误判
- [ ] 自动补偿金额阈值（TBD-SUP-002）已与财务团队评审确定
- [ ] 超阈值补偿转人工复核路径验证通过
- [ ] **每功能 BAS 文档均含"本功能 log 设计"章节**（per BAS-001 v1.5 §4.8.3 + BAS-004 v0.3 §4.4 release 必出宏清单与各功能 §X.Y 对应），且 log 章节内明确区分 debug-only（`#[cfg(debug_assertions)]` 守护的 `debug!`／`trace!`）与 release 必出（`info!`／`warn!`／`error!`）两类事件
- [ ] **release 必出事件清单（§2.1〜§4.2 全部 9 个本功能 log 设计章节）** 逐项可在治理脚本 `scripts/check-docs-consistency.sh` 中 grep 验证（对应事件名 `cs.*`），未遗漏治理关键事件（工单创建／分配／处理／关闭 / 支付对账 / SLA 警告 / 双重布尔校验 / 跨文档字段同步 / 支付凭证拦截）
- [ ] **debug-only 宏未守护 `info!`／`warn!`／`error!`**（per BAS-004 v0.3 §4.3 规则 #1 + §4.4 反例），CI 静态扫描（per BAS-004 v0.3 §11.1 新服务埋点接入检查清单）通过
- [ ] **客服工单域特殊合规检查**：玩家-客服对话脱敏逻辑完整（per BAS-004 v0.3 §5.1，邮箱／手机哈希化）、支付凭证／卡号字段在全部 log 设计中**不**出现明文（per BAS-004 v0.3 §5.1 + §4.4 release 必出宏清单）、NFR-EC 合规审计需要的全部对账事件（`cs.recon.*`）release 必出 + 强制全采样

### 4.1 本功能日志设计

本节覆盖**上线前检查清单执行**的观察点——"清单 5 项功能 + 4 项 log 章节上线检查项"逐项验证过程产生 release 必出事件（per BAS-005 v0.3 §10.2 + BAS-009 v0.7 §6.1 检查清单自身 log 设计模式）。**对账故障注入试验**（§4.1 检查项 3）的"是否产生误判"判定 → release 必出 + 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.prelaunch.checklist.started` | 上线前检查清单逐项验证（CI 入参） | 极低（上线前） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `release_version` ／ `ci_run_id` ／ `started_at`；约 220B／条 |
| `cs.prelaunch.checklist.passed` | 全部上线前检查项通过（5 项功能 + 4 项 log 章节上线检查项） | 极低（上线前） | release 必出（100% 强制全采样，治理事件必出） | 含 `release_version` ／ `verifier_id` ／ `pass_timestamp` ／ `check_count`；约 260B／条 |
| `cs.prelaunch.checklist.item_failed` | 9 项中任一项未通过 | 偶发 | release 必出（100% 强制全采样，治理事件必出） | 含 `release_version` ／ `failed_item`（state_machine_validation／sla_grade_reviewed／recon_failure_injection／auto_compensation_threshold／super_threshold_handoff／log_chapter_present／release_required_grep_passed／debug_only_compliant／release_required_macro_no_cfg） ／ `reason`；约 380B／条 |
| `cs.prelaunch.recon.fault_injection_passed` | 对账故障注入试验通过（§4.1 检查项 3，模拟服务商侧记录延迟到达，验证不产生误判） | 极低（上线前） | release 必出（100% 强制全采样，NFR-EC 合规） | 含 `release_version` ／ `fault_scenario`（provider_side_delay／provider_side_partial／provider_side_unavailable） ／ `injected_delay_seconds` ／ `misjudgment_count`；约 320B／条 |
| `cs.prelaunch.recon.fault_injection_failed` | 对账故障注入试验产生误判（违反 §3.3 "不将未取到数据误判为无交易"） | 极少 | release 必出（100% 强制全采样，`error!` 级别，**阻断级**信号） | 含 `release_version` ／ `fault_scenario` ／ `misjudgment_kind`（false_positive_compensation／false_negative_compensation） ／ `affected_order_count`；约 360B／条 |
| `cs.prelaunch.sla.grade_reviewed` | SLA 分级（TBD-SUP-001）已与客服／运营团队评审 | 极低（上线前） | release 必出（100% 强制全采样） | 含 `release_version` ／ `reviewer_id` ／ `tbd_ref`（TBD-SUP-001） ／ `decided_p95_seconds_per_category`；约 360B／条 |
| `cs.prelaunch.compensation.threshold_reviewed` | 自动补偿金额阈值（TBD-SUP-002）已与财务团队评审 | 极低（上线前） | release 必出（100% 强制全采样） | 含 `release_version` ／ `reviewer_id` ／ `threshold_value` ／ `currency`；约 240B／条 |
| `cs.prelaunch.state_machine.validation_passed` | 工单状态机非法迁移拒绝验证通过（§4.1 检查项 1） | 极低（上线前） | release 必出（100% 强制全采样） | 含 `release_version` ／ `test_case_count` ／ `rejected_transition_count` ／ `state_machine_version`；约 280B／条 |
| `cs.prelaunch.compensation.handoff_validated` | 超阈值补偿转人工复核路径验证通过（§4.1 检查项 5） | 极低（上线前） | release 必出（100% 强制全采样） | 含 `release_version` ／ `path_kind`（auto_to_support_ticket／support_ticket_to_human_review） ／ `test_scenario_count`；约 280B／条 |
| `cs.prelaunch.log_chapter.presence_verified` | "本功能日志设计" 章节在 BAS-016 全部 ## L2 段存在性验证（per BAS-005 v0.3 §10.2 第 1 项 CI 验证事件） | 极低（CI 验证） | release 必出（100% 强制全采样） | 含 `bas_id`（RGS-BAS-016） ／ `l2_section_count` ／ `log_section_count` ／ `coverage_ratio`；约 300B／条 |
| `cs.prelaunch.log_chapter.release_required_grep_passed` | release 必出事件清单（`cs.*` 治理事件）grep 验证通过（per BAS-005 v0.3 §10.2 第 2 项 CI 验证事件） | 极低（CI 验证） | release 必出（100% 强制全采样） | 含 `bas_id` ／ `expected_event_count` ／ `matched_event_count` ／ `missing_events`；约 320B／条 |
| `cs.prelaunch.log_chapter.debug_only_compliant` | debug-only 事件严格遵守 BAS-004 v0.3 §4.3 四条铁律（per BAS-005 v0.3 §10.2 第 3 项 CI 验证事件） | 极低（CI 验证） | release 必出（100% 强制全采样） | 含 `bas_id` ／ `checked_file_count` ／ `violation_count` ／ `violations`；约 300B／条 |
| `cs.prelaunch.log_chapter.release_macro_no_cfg` | release build 中**不**存在 `info!`／`warn!`／`error!` 被 `#[cfg(debug_assertions)]` 守护的代码点（per BAS-005 v0.3 §10.2 第 4 项 CI 验证事件） | 极低（CI 验证） | release 必出（100% 强制全采样） | 含 `bas_id` ／ `grep_pattern` ／ `violation_count` ／ `violations`；约 280B／条 |
| `cs.prelaunch.debug.full_checklist_dump` | 9 项检查清单的逐项核对结果（含 pass/fail 矩阵） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-2KB／条（release 剔除） |
| `cs.prelaunch.debug.fault_injection_timeline` | 故障注入试验的时间线 dump（注入→对账→判定） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除） |
| `cs.prelaunch.debug.log_chapter_coverage_diff` | BAS-016 全部 ## L2 段的 log 章节覆盖 diff（哪些段未覆盖） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-2KB／条（release 剔除） |
| `cs.prelaunch.debug.grep_pattern_dump` | CI 静态扫描使用的 grep 模式 dump（含 BAS-004 §4.4 释放必出宏清单） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + BAS-005 v0.3 §10.2 + BAS-009 v0.7 §6.1 模式）：
- `cs.prelaunch.recon.fault_injection_failed` 是**阻断级信号**（上线阻断）—— release 必出 + `error!` 强制全采样，不挂 `#[cfg]`
- `cs.prelaunch.checklist.passed` 是**上线门禁通过信号**—— release 必出 + 强制全采样，便于运维审计上线历史
- `cs.prelaunch.log_chapter.*` 是**log 章节自身上线检查**（per BAS-005 v0.3 §10.2 + BAS-009 v0.7 §6.1 自检模式）—— release 必出 + 强制全采样，构成"log 章节自描述 self-check"闭环
- `cs.prelaunch.debug.full_checklist_dump` 涉及 9 项逐项核对矩阵—— release 完全剔除
- 治理事件清单（强制 release 必出）：`checklist.*` ／ `recon.fault_injection_*` ／ `sla.grade_reviewed` ／ `compensation.threshold_reviewed` ／ `state_machine.validation_passed` ／ `compensation.handoff_validated` ／ `log_chapter.*` 共 11 个上线门禁信号必须 production 可见

## 4.2 代码评审检查清单

- [ ] 工单处理动作未出现绕过`AdminService`的直接账号状态修改
- [ ] 对账比对逻辑的关联键（`provider_txn_id`）唯一性校验存在
- [ ] 对账"待补偿"判定条件（§3.3）双重布尔校验方向已逐行核对，未写反
- [ ] `SupportTicket.dedup_key`命中为提示而非拒绝，未阻止玩家提交合理的新工单
- [ ] **debug-only 事件严格遵守 RGS-BAS-004 v0.3 §4.3 四条铁律**（宏直接守护、避免 `if cfg!` 外层、参数 O(1)、关联 ID 预先 `let` 绑定）
- [ ] **release build 中不存在 `info!`／`warn!`／`error!` 被 `#[cfg(debug_assertions)]` 守护的代码点**（grep 验证）
- [ ] **客服工单域 PII 静态扫描**：玩家-客服对话中明文 PII（邮箱／手机号）记录尝试 → PR 合并阻断（per §2.3 `cs.ticket.conversation.*` + BAS-004 v0.3 §5.1）
- [ ] **支付凭证静态扫描**：支付凭证／卡号字段记录尝试 → PR 合并阻断（per §2.3 `cs.ticket.conversation.payment_credential_blocked` + BAS-004 v0.3 §5.1 + §4.4 release 必出宏清单）
- [ ] **跨文档字段同步静态检查**：BAS-016 与 BAS-020 双方 `PaymentOrder` 字段清单完全一致（per RGS-BAS-010 §7.1 + §3.1 跨文档字段扩展声明 + `cs.payment.schema.cross_table_field_sync_applied`）
- [ ] **NFR-EC 合规审计对账事件覆盖**：NFR-EC 合规审计需要的全部对账事件（`cs.recon.*`）在代码中**逐项可检索到对应调用点**（grep 验证），未遗漏 `cs.recon.dual_boolean_check.passed` ／ `cs.recon.dual_boolean_check.failed` ／ `cs.recon.compare.reversed_condition.detected` ／ `cs.recon.provider.unavailable_extended` 等关键事件

### 4.2 本功能日志设计

本节覆盖**代码评审检查清单执行**的观察点——PR 触发代码评审检查清单（4 项功能 + 6 项 log 章节代码评审检查项）逐项验证。**所有代码缺陷类信号**（admin_bypass ／ provider_txn_id_uniqueness 缺失 ／ dual_boolean 写反 ／ dedup_key 行为错 ／ 对话明文 PII 记录 ／ 支付凭证记录尝试）→ release 必出 + `error!` 强制全采样。**6 项 log 章节代码评审检查项**（per BAS-005 v0.3 §10.2 + BAS-004 v0.3 §11.1 新服务埋点接入检查清单）→ release 必出 + 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `cs.review.checklist.started` | PR 触发代码评审检查清单（10 项） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `pr_id` ／ `changed_file_count` ／ `started_at`；约 240B／条 |
| `cs.review.checklist.passed` | 4 项功能 + 6 项 log 章节代码评审检查全部通过 | 偶发 | release 必出（100% 强制全采样，治理事件必出） | 含 `pr_id` ／ `reviewer_id` ／ `pass_timestamp`；约 240B／条 |
| `cs.review.checklist.item_failed` | 10 项中任一项未通过 | 偶发 | release 必出（100% 强制全采样，治理事件必出） | 含 `pr_id` ／ `failed_item`（admin_bypass_check／provider_txn_id_uniqueness／dual_boolean_direction／dedup_prompt_not_block／conversation_pii_log／payment_credential_log_attempt／log_chapter_present／release_required_grep_passed／debug_only_compliant／release_required_macro_no_cfg） ／ `reason`；约 400B／条 |
| `cs.review.admin_bypass.detected` | 工单处理动作试图绕过 `AdminService` 直接修改账号状态（§4.2 检查项 1，ARC-019 核心验证项） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `attempted_path` ／ `target_account_id_hash` ／ `affected_file` ／ `affected_line_range`；约 340B／条 |
| `cs.review.provider_txn_id.uniqueness_missing` | 对账比对逻辑的关联键（`provider_txn_id`）唯一性校验缺失（§4.2 检查项 2，NFR-SUP-004 保障） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `affected_file` ／ `affected_line_range` ／ `expected_constraint` ／ `actual_constraint`；约 340B／条 |
| `cs.review.dual_boolean.reversed_detected` | 对账 "待补偿" 判定条件方向写反（§4.2 检查项 3，§3.3 RSK-SUP-002 防护） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `inverted_condition`（1=provider_not_paid_check_reversed／2=internal_state_check_reversed） ／ `affected_file` ／ `affected_line_range` ／ `affected_logic_summary`；约 380B／条 |
| `cs.review.dedup_key.behavior_wrong` | `SupportTicket.dedup_key` 命中改为拒绝（应保持提示，§4.2 检查项 4） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `expected_behavior`（prompt） ／ `actual_behavior`（reject） ／ `affected_file`；约 320B／条 |
| `cs.review.conversation.pii_log_detected` | 玩家-客服对话明文 PII 被记录（违反 BAS-004 v0.3 §5.1，per §2.3 客服工单域隐私约束） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `pii_kind`（email／phone） ／ `affected_file` ／ `affected_line_range` ／ `redaction_required`；约 320B／条 |
| `cs.review.payment_credential.log_attempt_detected` | 支付凭证／卡号字段尝试记录（违反 BAS-004 v0.3 §5.1，per §2.3 客服工单域支付凭证禁止） | 极少（代码缺陷） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `field_name` ／ `affected_file` ／ `affected_line_range` ／ `pii_kind`（card_number／cvv／credential_token）；约 340B／条 |
| `cs.review.cross_table_sync_check_failed` | 跨文档字段同步检查失败（per RGS-BAS-010 §7.1 + §3.1 跨文档字段扩展声明，BAS-016 ↔ BAS-020 字段不一致） | 极少（CI 检出） | release 必出（100% 强制全采样，`error!` 强制全采样，**阻断级**信号） | 含 `pr_id` ／ `field_name` ／ `source_bas`（BAS-020） ／ `target_bas`（BAS-016） ／ `expected_in_target` ／ `actual_in_target`；约 360B／条 |
| `cs.review.log_chapter.presence_verified` | "本功能日志设计" 章节在 BAS-016 全部 ## L2 段存在性验证（per §4.1 模式 + BAS-005 v0.3 §10.2） | 偶发（CI 验证） | release 必出（100% 强制全采样） | 含 `pr_id` ／ `bas_id` ／ `l2_section_count` ／ `log_section_count` ／ `coverage_ratio`；约 320B／条 |
| `cs.review.log_chapter.release_required_grep_passed` | release 必出事件清单（`cs.*` 治理事件）grep 验证通过（per §4.1 模式） | 偶发（CI 验证） | release 必出（100% 强制全采样） | 含 `pr_id` ／ `bas_id` ／ `expected_event_count` ／ `matched_event_count` ／ `missing_events`；约 340B／条 |
| `cs.review.log_chapter.debug_only_compliant` | debug-only 事件严格遵守 BAS-004 v0.3 §4.3 四条铁律（per §4.1 模式） | 偶发（CI 验证） | release 必出（100% 强制全采样） | 含 `pr_id` ／ `bas_id` ／ `checked_file_count` ／ `violation_count` ／ `violations`；约 320B／条 |
| `cs.review.log_chapter.release_macro_no_cfg` | release build 中**不**存在 `info!`／`warn!`／`error!` 被 `#[cfg(debug_assertions)]` 守护的代码点（per §4.1 模式） | 偶发（CI 验证） | release 必出（100% 强制全采样） | 含 `pr_id` ／ `bas_id` ／ `grep_pattern` ／ `violation_count` ／ `violations`；约 300B／条 |
| `cs.review.conversation.pii_redaction_compliant` | 玩家-客服对话脱敏逻辑完整（per BAS-004 v0.3 §5.1 + §2.3 客服工单域特殊考虑） | 偶发（CI 验证） | release 必出（100% 强制全采样） | 含 `pr_id` ／ `bas_id` ／ `checked_path_count` ／ `redaction_path_count` ／ `missing_redaction_paths`；约 340B／条 |
| `cs.review.debug.full_review_checklist_dump` | 10 项代码评审检查的逐项核对结果 | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-2KB／条（release 剔除） |
| `cs.review.debug.pr_diff_with_findings` | PR diff 全文 + 检查发现标记（含代码上下文） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 5-50KB／条（release 完全剔除，避免泄漏代码片段） |
| `cs.review.debug.code_path_static_analysis` | 静态分析输出（哪条控制流可能绕过 `AdminService`） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB／条（release 剔除） |
| `cs.review.debug.pii_pattern_match_dump` | PII 模式匹配 dump（哪些代码位置匹配 `email`／`phone`／`card` 正则） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B／条（release 剔除，**不**记录明文 PII） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4 + §5.1 + §6.2 + BAS-005 v0.3 §10.2 + BAS-009 v0.7 §6.1 模式）：
- `cs.review.admin_bypass.detected` ／ `provider_txn_id.uniqueness_missing` ／ `dual_boolean.reversed_detected` ／ `dedup_key.behavior_wrong` ／ `conversation.pii_log_detected` ／ `payment_credential.log_attempt_detected` ／ `cross_table_sync_check_failed` 全部是**阻断级**信号（PR 合并阻断）—— release 必出 + `error!` 强制全采样，不挂 `#[cfg]`
- `cs.review.conversation.pii_log_detected` ／ `payment_credential.log_attempt_detected` 是**合规审计关键信号**（per §2.3 客服工单域特殊考虑 + BAS-004 v0.3 §5.1）—— release 必出 + `error!` 强制全采样，便于 SRE 合规审计回溯
- `cs.review.log_chapter.*` 是**log 章节自身代码评审检查**（per §4.1 自检模式 + BAS-005 v0.3 §10.2）—— release 必出 + 强制全采样，构成"log 章节 self-check"闭环
- `cs.review.debug.pr_diff_with_findings` 在大型 PR 下可能 50KB+ —— release 完全剔除，避免 RUST_LOG=debug 误开时泄漏代码片段
- `cs.review.debug.pii_pattern_match_dump` **可能含明文 PII 模式片段**—— release 完全剔除
- 治理事件清单（强制 release 必出）：`checklist.*` ／ `admin_bypass.detected` ／ `provider_txn_id.uniqueness_missing` ／ `dual_boolean.reversed_detected` ／ `dedup_key.behavior_wrong` ／ `conversation.pii_log_detected` ／ `payment_credential.log_attempt_detected` ／ `cross_table_sync_check_failed` ／ `log_chapter.*` ／ `conversation.pii_redaction_compliant` 共 16 个代码评审／合规／阻断级信号必须 production 可见

---

# 5. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-033、FR-SUP-001〜007 | §2、§2.3（状态机迁移）、§2.4（SLA分级） |
| FR-SUP-010〜015 | §3、§3.3（异常分支） |
| NFR-SUP-001〜004 | §3.2 |
| AC-SUP-001〜004 | §4.1 |
| TBD-SUP-001〜002、RSK-SUP-001〜002 | §4.1 |
| FR-PLT-003〜005（跨文档字段扩展） | §3.1（跨文档字段扩展声明）、§3.1 本功能日志设计（`cs.payment.schema.cross_table_field_sync_*`） |
| NFR-EC（NFR-EC 合规审计） | §3.2、§3.3、§3.2 本功能日志设计（`cs.recon.batch.*`／`cs.recon.compare.*`／`cs.recon.compensation.*`／`cs.recon.idempotency.*`）、§3.3 本功能日志设计（`cs.recon.provider.*`／`cs.recon.compare.inconsistency.*`） |
| FR-LOG-010/011/012/013/020/021/040/041（埋点与日志规范） | §2.1〜§4.2 全部 9 个本功能 log 设计章节、§4.1 上线前检查清单 log 章节上线检查项、§4.2 代码评审检查清单 log 章节代码评审检查项 |
| **AC-SUP-006（debug-only 宏在 release build 完全由 `#[cfg(debug_assertions)]` 剔除，二进制中无相关调用）** | §2.1〜§4.2 全部 9 个本功能 log 设计章节中所有 `cs.*.debug.*` 字段 + RGS-BAS-004 v0.3 §4.4 编译期×运行时二维矩阵 + §4.3 四条铁律 + §11.1 新服务埋点接入检查清单 | §2.1、§2.2、§2.3、§2.4、§3.1、§3.2、§3.3、§4.1、§4.2 |
| **AC-SUP-007（每功能 BAS 文档须含本功能 log 设计章节，区分 debug-only / release 必出）** | §2.1／§2.2／§2.3／§2.4／§3.1／§3.2／§3.3／§4.1／§4.2 共 9 个"本功能日志设计"小节 + §4.1 检查项第 6 条（每功能 log 章节存在性）+ §4.1 检查项第 7 条（release 必出事件 grep 验证）+ §4.1 检查项第 8 条（debug-only 四铁律合规）+ §4.1 检查项第 9 条（客服工单域特殊合规检查）+ §4.2 检查项第 5 条（debug-only 四铁律合规）+ §4.2 检查项第 6 条（release 必出宏未被 `#[cfg]` 守护）+ §4.2 检查项第 7 条（客服工单域 PII 静态扫描）+ §4.2 检查项第 8 条（支付凭证静态扫描）+ §4.2 检查项第 9 条（跨文档字段同步静态检查）+ §4.2 检查项第 10 条（NFR-EC 合规审计对账事件覆盖） | §2.1、§2.2、§2.3、§2.4、§3.1、§3.2、§3.3、§4.1、§4.2 |

---

> 本文档与RGS-REQ-019（客服工单与支付对账 需求定义书）配套使用。
