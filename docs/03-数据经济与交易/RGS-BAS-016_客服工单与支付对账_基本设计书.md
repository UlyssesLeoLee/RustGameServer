# 基本设计书（基本設計書 / Basic Design Document）

**客服工单与支付对账 Customer Support Ticketing & Payment Reconciliation**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-016 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-019 需求定义书（ARC-033） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-08-16 |
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

### 2.3 状态机迁移条件（FR-SUP-002落地）

| 迁移 | 触发条件 | 拒绝条件 |
|---|---|---|
| `待受理 → 处理中` | 客服/GM认领工单 | 工单已关闭（`已解决`/`已驳回`） |
| `处理中 → 待玩家补充信息` | 客服标记需要更多信息 | — |
| `待玩家补充信息 → 处理中` | 玩家补充回复 | 超过配置的静默期（默认7天）自动转`已驳回`，防止工单无限期悬挂 |
| `处理中 → 已解决` | 客服记录`resolution_summary`并关闭 | `resolution_summary`为空（FR-SUP-005强制关闭时必须留痕） |
| `处理中 → 已驳回` | 客服判定不成立并记录理由 | 同上 |

### 2.4 SLA分级基准（FR-SUP-003，TBD-SUP-001评审前的默认建议值）

| `category` | 首次响应SLA（默认建议值，最终以TBD-SUP-001评审结果为准） | 升级提醒触发点 |
|---|---|---|
| `payment_issue` | p95 < 4小时 | 超过SLA的80%时长即触发`TicketEscalationNotifier`提前预警 |
| `ban_appeal` | p95 < 24小时 | 同上 |
| `item_anomaly` | p95 < 24小时 | 同上 |
| `other` | p95 < 48小时 | 同上 |

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

### 3.3 异常分支

```
支付服务商侧对账文件/API本次拉取失败或数据不完整（服务商侧临时故障/延迟）
  → ReconciliationJob本轮跳过，记录告警（复用RGS-BAS-003§6），不将"未取到数据"误判为"服务商侧无交易"
  → 下一周期正常拉取时自动补齐窗口（对账窗口须与上次成功窗口重叠，避免因单次失败产生的比对空档遗漏掉单）

比对条件疑似写反（RSK-SUP-002缓解）：ReconciliationJob在判定"待补偿"前须同时满足①provider_txn_id在服务商侧记录中状态为"支付成功"②本地PaymentOrder.state不在(已发货、已补偿)集合内，两个条件均需显式布尔校验并各自记录比对依据快照，代码评审须逐行核对条件方向未写反（§4.2检查项）
```

---

# 4. 标准化检查清单

## 4.1 上线前检查清单

- [ ] 工单状态机非法迁移拒绝验证通过
- [ ] SLA分级（TBD-SUP-001）已与客服/运营团队评审确定
- [ ] 对账批处理故障注入试验：模拟服务商侧记录延迟到达，验证不产生误判
- [ ] 自动补偿金额阈值（TBD-SUP-002）已与财务团队评审确定
- [ ] 超阈值补偿转人工复核路径验证通过

## 4.2 代码评审检查清单

- [ ] 工单处理动作未出现绕过`AdminService`的直接账号状态修改
- [ ] 对账比对逻辑的关联键（`provider_txn_id`）唯一性校验存在
- [ ] 对账"待补偿"判定条件（§3.3）双重布尔校验方向已逐行核对，未写反
- [ ] `SupportTicket.dedup_key`命中为提示而非拒绝，未阻止玩家提交合理的新工单

---

# 5. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-033、FR-SUP-001〜007 | §2、§2.3（状态机迁移）、§2.4（SLA分级） |
| FR-SUP-010〜015 | §3、§3.3（异常分支） |
| NFR-SUP-001〜004 | §3.2 |
| AC-SUP-001〜004 | §4.1 |
| TBD-SUP-001〜002、RSK-SUP-001〜002 | §4.1 |

---

> 本文档与RGS-REQ-019（客服工单与支付对账 需求定义书）配套使用。
