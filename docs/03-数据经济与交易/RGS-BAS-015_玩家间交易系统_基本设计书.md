# 基本设计书（基本設計書 / Basic Design Document）

**玩家间交易系统 Player-to-Player Trading System**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-015 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-018 需求定义书（ARC-032） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-08-16 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-018§8 ARC-032展开为交易Saga组件设计、数据模型、防欺诈字段级设计 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①新增`TradeVisibilityGuard`组件落地FR-TRD-006交易目标可见性限制②补充`TradeOffer`索引/唯一性约束与`TradeAuditLog`字段级设计（FR-TRD-015〜018）③补充并发调包/双花的乐观锁校验机制与Saga补偿自身失败的升级分支（RSK-TRD-002） | FR-TRD-006、FR-TRD-015〜018、RSK-TRD-002 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | Saga补偿路径是否覆盖全部故障时点 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [交易状态机与组件设计](#2-交易状态机与组件设计)
3. [数据模型](#3-数据模型)
4. [交易成立时序](#4-交易成立时序)
5. [标准化检查清单](#5-标准化检查清单)
6. [追溯性](#6-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-018定义的ARC-032，遵循ARC-018挂载原则——全部组件依附既有EC限界上下文运行，不新建独立限界上下文、数据库或部署单元。

---

# 2. 交易状态机与组件设计

## 2.1 状态机（ST-004落地）

```
Draft → Offered → Accepted → Settled
                 ↘ Cancelled
Offered → Expired（超时自动迁移，不需人工触发）
```

| 迁移 | 触发条件 | 拒绝条件 |
|---|---|---|
| `Draft → Offered` | 发起方提交挂单，己方资产冻结成功 | 资产不足/已被其他操作占用 |
| `Offered → Accepted` | 目标玩家显式接受 | 挂单已过期/已撤销 |
| `Offered → Cancelled` | 发起方主动撤销 | 已进入`Accepted`（不可逆，FR-TRD-013） |
| `Offered → Expired` | 达到`expire_at`且无人操作 | — |
| `Accepted → Settled` | 原子成立流程完成（§4） | 任一方资产快照已失效（FR-TRD-014） |

## 2.2 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `TradeOfferService` | EC | 挂单创建/撤销/超时管理，资产冻结/解冻 |
| `TradeSettlementSaga` | EC（复用ARC-011既定Saga边界） | `Accepted`后的原子成立编排：双方资产转移+补偿逻辑 |
| `TradeAuditLog` | EC | 复用RGS-BAS-003§7审计设计存储结构 |
| `TradeVisibilityGuard` | EC | 挂单创建前校验目标玩家是否在允许范围内（好友/同队伍，具体范围随TBD-TRD-001评审结果配置），拒绝范围外的挂单创建请求（FR-TRD-006） |

### 2.3 交易目标可见性校验（FR-TRD-006落地）

`TradeVisibilityGuard`在`TradeOfferService`创建挂单前同步调用，校验规则以配置项`trade_visibility_scope`表达（枚举：`friend_only`／`party_only`／`friend_or_party`，具体取值随TBD-TRD-001与策划评审结果确定，评审前默认`friend_or_party`）。校验失败时`Draft → Offered`迁移直接拒绝，不产生资产冻结副作用（不消耗FR-TRD-002冻结路径）。该校验为**同步阻塞**校验（区别于任务系统FR-GSM-015异步进度更新），因为挂单创建本身即为低频操作，无需异步化。

---

# 3. 数据模型

`TradeOffer`（逻辑字段）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `trade_id` | uuid | 唯一标识 |
| `initiator_id` / `target_id` | 玩家ID | 发起方/目标方 |
| `initiator_items` / `target_items` | 引用既有物品/货币规格 | 双方声明的交换内容 |
| `snapshot_version` | int | `Accepted`时刻锁定的快照版本号，用于FR-TRD-014防调包校验 |
| `state` | enum（同§2.1状态机） | 当前状态 |
| `expire_at` | timestamp | 过期时间 |
| `fee_rate` | decimal，可选 | 手续费率（TBD-TRD-002待定，默认0） |

索引/约束：`trade_id`为主键；`(initiator_id, state)`与`(target_id, state)`复合索引支撑"我发起的/我收到的挂单列表"查询；`(state, expire_at)`复合索引支撑§4定时任务批量扫描`Offered`且已超期的记录以驱动自动解冻（FR-TRD-003）。`snapshot_version`每次挂单内容变更（若在`Draft`阶段允许编辑）递增，作为乐观锁版本号（见§4并发控制）。

`TradeAuditLog`（逻辑字段，FR-TRD-015/016）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `log_id` | uuid | 唯一标识 |
| `trade_id` | uuid | 关联`TradeOffer`，非外键强约束（归档后原表可能已清理，同RGS-BAS-007§4归档标准） |
| `event_type` | enum(`created`／`accepted`／`cancelled`／`expired`／`settled`／`compensated`／`escalated`) | 记录交易生命周期全部关键事件，含FR-TRD-015要求的"已成立、已撤销、已过期"及本节新增的补偿/升级事件 |
| `actor_id` | 玩家ID，可空 | 触发该事件的操作者（系统自动触发如`expired`时为空） |
| `snapshot_at_event` | jsonb | 事件发生时的双方资产快照，供GM查证调包争议 |
| `occurred_at` | timestamp | 事件时间 |

索引：`(trade_id, occurred_at)`复合索引支撑单笔交易的完整时间线回放；`(actor_id, occurred_at)`复合索引支撑FR-TRD-018"按玩家ID检索交易历史"的GM后台查询（复用RGS-BAS-003§3.4只读查询模式，直接以此索引服务，不新增专属查询工具）。分区归档按`occurred_at`月度分区，复用RGS-BAS-007§4标准（FR-TRD-016、NFR-TRD-004默认1年保留期）。

---

# 4. 交易成立时序

```
双方均已Accept
  → TradeSettlementSaga读取trade_id对应的snapshot_version
  → 校验双方资产状态未发生快照外变更（若已变更，拒绝并转入人工/自动补偿流程）
  → 复用FR-EC-003确定请求路径，在同一事务边界内完成：
      甲方扣除己方物品 + 乙方扣除己方物品 + 甲方获得乙方物品 + 乙方获得甲方物品（+ 手续费扣除，若启用）
  → 全部成功 → 状态迁移至Settled，记录TradeAuditLog（event_type=settled）
  → 任一步失败 → Saga补偿：回滚已执行步骤，资产恢复至冻结前状态，状态保持Accepted供重试或转人工处理，记录TradeAuditLog（event_type=compensated）
  → 补偿本身失败（如回滚时资产写入也失败，RSK-TRD-002最坏情形）→ 交易状态强制迁移至专用中间态`CompensationFailed`（不复用既有ST-004枚举值，避免与正常终态混淆），触发高优先级告警（复用RGS-BAS-003§6），记录TradeAuditLog（event_type=escalated），转入GM人工核账队列，禁止该笔trade_id相关资产在人工核实前被其他操作占用
```

> 并发控制（RSK-TRD-002双花/调包防护）：`TradeSettlementSaga`在执行资产转移前，以`snapshot_version`做乐观锁校验（`UPDATE ... WHERE trade_id=? AND snapshot_version=?`），校验失败（快照已被并发操作使其失效，如资产在确认窗口内被其他途径变更）直接拒绝进入原子事务，返回错误令客户端重新发起挂单，**不得**静默使用旧快照继续结算（FR-TRD-014）。

> 幂等键：以`trade_id`+当前`state`为幂等键，重复提交同一笔"接受"操作在`state`已为`Settled`时直接返回既有结果，不重复执行（FR-TRD-012）。

---

# 5. 标准化检查清单

## 5.1 上线前检查清单

- [ ] 交易原子成立故障注入试验通过（中断后重启，无单方受损）
- [ ] 挂单超时自动解冻验证通过
- [ ] 幂等性验证：重复提交接受操作不产生重复扣款/发货
- [ ] 交易目标可见性范围（TBD-TRD-001）已与策划评审确定
- [ ] GM后台交易历史查询接口可用
- [ ] `TradeVisibilityGuard`范围配置已按TBD-TRD-001评审结果上线（FR-TRD-006）
- [ ] `CompensationFailed`升级告警与GM人工核账队列已联调验证（RSK-TRD-002最坏分支）
- [ ] 注：`TradeVisibilityGuard`同步校验与人工核账队列为新增常态运维面，OLU运维负荷未核算，见ISS-065

## 5.2 代码评审检查清单

- [ ] 交易价值转移路径未绕过FR-EC-003确定请求路径
- [ ] `Accepted`状态迁移后未出现允许反悔的代码路径

---

# 6. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-032、FR-TRD-001〜006 | §2、§2.3（FR-TRD-006可见性校验） |
| FR-TRD-010〜018 | §3、§4 |
| RSK-TRD-002 | §4（乐观锁并发控制、CompensationFailed升级分支） |
| NFR-TRD-001〜004 | §4 |
| AC-TRD-001〜004 | §5.1 |
| TBD-TRD-001〜002、RSK-TRD-001〜002 | §5.1 |

---

> 本文档与RGS-REQ-018（玩家间交易系统 需求定义书）配套使用。
