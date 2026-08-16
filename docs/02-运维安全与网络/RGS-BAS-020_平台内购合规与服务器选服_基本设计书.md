# 基本设计书（基本設計書 / Basic Design Document）

**平台内购合规与服务器选服 Platform IAP Compliance & Realm Selection**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-020 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-023 需求定义书（ARC-038） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-08-16 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-023§9 ARC-038展开为收据校验组件设计与时序、选服路由设计、合服演练与执行流程 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①补充平台官方验证接口不可用时的待重试队列设计（RSK-PLT-001）②补充`PaymentOrder`平台内购扩展字段与沙盒/生产环境隔离（FR-PLT-004、FR-PLT-005）③补充合服冲突解决规则的配置表字段级设计（FR-PLT-021） | FR-PLT-004、FR-PLT-005、FR-PLT-021、RSK-PLT-001 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 合服演练模式与正式执行的数据一致性保证是否充分 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [平台收据校验组件设计](#2-平台收据校验组件设计)
3. [选服路由设计](#3-选服路由设计)
4. [合服/分服执行流程](#4-合服分服执行流程)
5. [标准化检查清单](#5-标准化检查清单)
6. [追溯性](#6-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-023定义的ARC-038，全部组件依附既有PL/AD限界上下文运行，不新建独立限界上下文。

---

# 2. 平台收据校验组件设计

## 2.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `ReceiptVerifier` | PL/EC | 向App Store/Google Play官方接口校验收据，每个平台一个适配子模块 |
| `RefundNotificationHandler` | PL/EC | 接收平台异步退款通知，触发权益追回流程 |

## 2.2 收据校验时序

```
客户端完成平台内购，取得收据
  → 提交收据至服务器
  → ReceiptVerifier依平台类型选择适配子模块，向平台官方接口验证
  → 验证失败（签名无效/环境不匹配）→ 拒绝，记录审计日志（FR-PLT-004，含失败原因分类：invalid_signature／already_used／sandbox_prod_mismatch）
  → 验证接口不可用（超时/5xx，区别于"验证失败"的明确拒绝）→ 不判定为欺诈，投递至待重试队列（见§2.4，RSK-PLT-001）
  → 验证成功 → 取得平台侧唯一交易标识
      → 以交易标识为幂等键查询既有PaymentOrder（复用RGS-BAS-016§3.1数据模型）
          已存在 → 直接返回既有结果，不重复处理
          不存在 → 写入PaymentOrder + 复用FR-EC-003确定请求路径发放权益
```

## 2.4 平台校验接口不可用的待重试队列（RSK-PLT-001落地）

`PendingReceiptVerification`（依附既有PL/EC上下文数据库，不新建独立库）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `pending_id` | uuid | 唯一标识 |
| `raw_receipt` | 加密存储的原始收据 | 待重试的收据内容 |
| `platform_type` | enum(`app_store`／`google_play`） | 决定重试时使用哪个适配子模块 |
| `retry_count` | int | 已重试次数 |
| `next_retry_at` | timestamp | 下次重试时间（指数退避，复用ARC-009标准消费者重试参数量级） |
| `status` | enum(`pending`／`resolved`／`abandoned`) | `abandoned`为超过最大重试次数后的终态，转人工（生成RGS-BAS-016 SupportTicket，category=payment_issue） |

`ReceiptVerifier`定时任务扫描`status=pending AND next_retry_at<=now()`的记录重新发起验证，成功后进入§2.2正常发放路径并将本记录标记`resolved`；超过最大重试次数（详细设计确定阈值）标记`abandoned`并转人工复核，**不得**因平台接口持续不可用而无限期悬挂玩家的合法收据。

## 2.5 `PaymentOrder`平台内购扩展字段（FR-PLT-004、FR-PLT-005）

复用RGS-BAS-016§3.1既定`PaymentOrder`结构，新增以下字段以承载平台内购特有信息（不新建独立表，遵循FR-PLT-005"共享同一套数据模型"）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `payment_channel` | enum(`platform_iap`／`direct_gateway`) | 区分平台内购与RGS-REQ-019既有直连支付，决定对账/退款处理走哪条子流程 |
| `platform_type` | enum，可选（仅`payment_channel=platform_iap`时非空） | `app_store`／`google_play` |
| `platform_environment` | enum(`sandbox`／`production`) | 沙盒/生产环境标记，**必须**与收据校验时平台返回的环境一致，环境不匹配须拒绝（FR-PLT-004"环境不匹配"分支），防止沙盒测试收据被用于生产环境权益发放 |
| `refund_status` | enum(`none`／`refunded`／`clawback_pending`／`clawback_done`) | 退款处理状态（FR-PLT-003），初始为`none` |

索引：`(platform_type, provider_txn_id)`复合唯一索引（`provider_txn_id`复用RGS-BAS-016既定字段承载平台交易标识），确保跨平台交易标识不产生误关联。

## 2.3 退款处理时序

```
平台异步推送退款/撤销通知（App Store Server Notifications / Google Play RTDN）
  → RefundNotificationHandler接收并校验通知来源真实性（平台签名验证）
  → 关联至对应PaymentOrder（依交易标识）
  → 触发权益追回流程：依TBD-PLT-001确定的追回方式（扣除等价物/标记负债/不追回）
  → 追回结果留痕（复用RGS-BAS-003§7审计设计）
```

---

# 3. 选服路由设计

## 3.1 组件划分

| 组件 | 职责 |
|---|---|
| `RealmDirectoryService` | 维护逻辑服列表与状态（正常/爆满/维护中），依附AD限界上下文，状态由GM后台配置驱动 |
| `RealmRouter` | 鉴权成功后、进入大厅前的路由决策，依附PL限界上下文 |

## 3.2 选服时序

```
鉴权成功（复用既有FR-GW-002）
  → RealmRouter查询账号是否已有"主服"记录
      有 → 直接路由至主服，跳过选服界面
      无（首次登录）→ 客户端展示RealmDirectoryService提供的服务器列表（含状态）
          玩家选择 → 记录为主服 → 路由至该服
  → 路由完成后进入既有大厅流程（RGS-REQ-016/BAS-013）
```

---

# 4. 合服/分服执行流程

## 4.1 冲突解决规则配置表（FR-PLT-021落地）

`MergeConflictRuleSet`（配置表，与具体某次合服作业关联，非全局默认值，因不同批次合服的运营诉求可能不同）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `merge_job_id` | uuid | 关联具体一次合服作业 |
| `character_name_conflict_rule` | enum(`auto_rename_with_suffix`／`require_manual_rename_on_login`) | 同名角色处理策略 |
| `unique_item_conflict_rule` | enum(`stack_additively`／`keep_both_as_separate`／`keep_earliest_and_compensate`) | 重复唯一性道具处理策略（如限定称号类不可叠加道具的处理） |
| `currency_conflict_rule` | enum(`sum`) | 货币类冲突固定为累加（货币无"唯一性冲突"概念，仅需求和） |
| `approved_by` | 运营/架构师签署 | FR-PLT-021"须与运营团队评审确定"的评审记录关联 |

`MergeConflictRuleSet`须在§4.2步骤1完成评审并锁定后，方可进入步骤2演练环境执行；演练/正式执行均读取同一份已锁定配置，**不得**在正式执行时临时调整规则（避免"执行人员临时决定"，FR-PLT-021明确禁止的情形）。

## 4.2 复用ARC-018挂载/退场检查清单的合服适配

| 步骤 | 内容 | 对应ARC-018既定步骤 |
|---|---|---|
| 1. 冲突规则评审 | 运营+架构师评审同名角色/重复道具的处理规则，配置化落地（FR-PLT-021） | 挂载前评审 |
| 2. 演练环境执行 | 在演练环境以生产数据快照执行完整合并流程，核对资产总量前后一致 | 挂载前验证 |
| 3. 演练结果评审 | 演练无异常方可排期正式执行；有异常须回到步骤1修正规则 | 挂载判定 |
| 4. 维护窗口正式执行 | 被合并服进入维护模式（复用既有维护模式传播机制）→ 执行数据合并 → 校验完成 | 正式挂载 |
| 5. 被合并服退场 | 数据合并确认无误后，被合并服按ARC-018既定退场流程下线 | 退场 |

---

# 5. 标准化检查清单

## 5.1 上线前检查清单

- [ ] 伪造收据拒绝测试通过
- [ ] 收据幂等测试通过（重复提交不重复发放）
- [ ] 退款通知处理测试通过，权益追回逻辑正确
- [ ] 选服路由验证：首次登录展示服务器列表，后续登录默认路由主服
- [ ] 合服演练流程至少完整执行一次并通过资产一致性校验（若适用多服架构）
- [ ] 平台验证接口不可用的待重试队列（§2.4）已验证：接口恢复后待重试收据自动完成发放，超限转人工
- [ ] `PaymentOrder.platform_environment`沙盒/生产不一致校验已验证拒绝跨环境收据
- [ ] `MergeConflictRuleSet`已在合服作业前完成评审锁定，演练与正式执行读取同一份配置
- [ ] 注：`PendingReceiptVerification`定时重试任务为新增常态运维面，OLU运维负荷未核算，见ISS-065

## 5.2 代码评审检查清单

- [ ] 收据校验路径未出现仅信任客户端声明、跳过平台官方验证的分支
- [ ] 合服执行代码未跳过步骤2演练直接进入步骤4正式执行

---

# 6. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-038、FR-PLT-001〜005 | §2、§2.4（待重试队列）、§2.5（PaymentOrder扩展字段） |
| FR-PLT-010〜013 | §3 |
| FR-PLT-020〜023 | §4、§4.1（冲突解决规则配置） |
| NFR-PLT-001〜004 | §2、§4 |
| AC-PLT-001〜004 | §5.1 |
| TBD-PLT-001〜002、RSK-PLT-001〜002 | §5.1、§2.4（RSK-PLT-001） |

---

> 本文档与RGS-REQ-023（平台内购合规与服务器选服 需求定义书）配套使用。
