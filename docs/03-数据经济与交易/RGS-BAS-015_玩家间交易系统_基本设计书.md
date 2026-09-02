# 基本设计书（基本設計書 / Basic Design Document）

**玩家间交易系统 Player-to-Player Trading System**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-015 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-018 需求定义书（ARC-032） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-09-01 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-018§8 ARC-032展开为交易Saga组件设计、数据模型、防欺诈字段级设计 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①新增`TradeVisibilityGuard`组件落地FR-TRD-006交易目标可见性限制②补充`TradeOffer`索引/唯一性约束与`TradeAuditLog`字段级设计（FR-TRD-015〜018）③补充并发调包/双花的乐观锁校验机制与Saga补偿自身失败的升级分支（RSK-TRD-002） | FR-TRD-006、FR-TRD-015〜018、RSK-TRD-002 |
| 0.3 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部 36 个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§2.1（状态机迁移）/§2.2（组件划分与资产冻结解冻）/§2.3（TradeVisibilityGuard目标可见性校验，**注：原`### 2.3`升至`## 2.3`以与§2.1/§2.2保持H2层级一致**）/§3.1（数据模型迁移与归档）/§4.1（Saga原子成立/Saga补偿/RSK-TRD-002升级分支/乐观锁/反作弊联动）/§5.1（上线前检查清单执行跟踪）/§5.2（代码评审检查清单执行跟踪）共 7 个"本功能日志设计"小节全部新增；每节均含 5 列详尽版（字段名/触发条件/频率估算/采样策略/脱敏与成本），显式区分 `info!`/`warn!`/`error!`（release 必出，编译期常驻，per BAS-004 v0.3 §6.2 强制全采样白名单）与 `debug!`/`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only，release build 完全剔除零运行时开销）两类事件；字段名前缀统一为 `trade.*`（区别于 BAS-002 `mnt.*` / BAS-003 `gm.*` / BAS-005 `plugin.*` / BAS-007 `db.*` / BAS-009 `gov.*`），命名严格 snake_case 与 BAS-004 v0.3 §4.6.1/§4.6.2 保持拼写一致（FR-LOG-013）；**交易域特殊强制**：①交易发起/接受/拒绝/取消/超时/完成/补偿/升级 8 类生命周期事件 → `info!` 强制全采样（资产不可逆，全链路审计）②双账户资产变更（atomic_transfer）→ `info!` 强制全采样 + `trace_id` 关联（财务一致性证据）③RSK-TRD-002 升级分支（`CompensationFailed` 状态迁移）→ `error!` 强制全采样（与 NFR-AV-005 可用性 + 财务数据完整性挂钩，P0 告警链路立即捕获）④反作弊联动（高频/大额/异常时序）→ `error!` 强制全采样（与既有反作弊链路联锁）⑤撮合引擎/价格波动 → debug-only（性能敏感，release 剔除）；§5.1/§5.2 检查清单新增 log 章节上线检查项；§6 追溯性新增 AC-TRD-006（debug-only 宏 release 完全剔除）与 AC-TRD-007（每功能 BAS 文档须含本功能 log 章节），与 BAS-001 v1.5 §4.8.3.4（commit 32d9eb6）/ BAS-007 v0.3（commit e711d09）/ BAS-009 v0.7（commit 9a628cf）/ BAS-004 v0.3（commit 47e26b0+0ee6262）形成统一规范 | §2.1、§2.2、§2.3、§3.1、§4.1、§5.1、§5.2、§6 |
| 0.4 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实「処理フロー」段四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板, RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1): 新增 §2.4 処理フロー（处理流程 / Processing Flow）段, 含主流程图 (mermaid sequenceDiagram, 8 actor: Initiator / Target / TradeOfferService / TradeVisibilityGuard / TradeOfferStateMachine / DB / EC 域 (FR-EC-003) / TradeSettlementSaga / GM 人工核账队列) + 異常分支表 (8 行, 覆盖可见性校验失败 / 资产冻结失败 / 状态并发变化 / 乐观锁失效 / EC Saga 步骤失败 / 事务提交失败 / 补偿本身失败 RSK-TRD-002 / 幂等命中) + 决策点矩阵 (6 行, 覆盖可见性范围评估 / 接受时状态 / 乐观锁校验 / Saga 失败补偿 / 反作弊联动 / 补偿失败升级) + 验证点清单 (8 行, 与 §2.1 / §2.2 / §2.3 / §4.1 既定 4 节呼应); trace_id 贯穿全链路 (per BAS-004 v0.3 §4.4); 事务边界与 Saga 跨域标注 (per BAS-100 v0.1, TradeSettlementSaga 4 步资产转移同事务 + EC 跨域走 Saga); 与既有 §2.1 状态机 / §2.3 可见性校验 / §4 交易成立时序 互为详细化引用, §2.4 为全景流程 + 异常分支 + 决策点 + 验证点汇总 | §2.4、§6 |

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
   - 2.4 [処理フロー（处理流程 / Processing Flow）](#24-処理フロー处理流程--processing-flow)
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

### 2.1 本功能日志设计

本节覆盖**交易状态机迁移（含 5 种迁移路径 + 拒绝路径）**的运行时观察点——交易域核心可观察面，**资产不可逆**特性要求全部生命周期事件 release 必出 + 100% 强制全采样（与 NFR-EC-001 资产安全 + FR-TRD-015 审计强需求挂钩），SRE 可按 `trade_id` 维度回放单笔交易完整时间线。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.state.draft_to_offered.committed` | 发起方提交挂单且资产冻结成功（`Draft → Offered`） | 1-50/s（峰值 100/s 短促） | release 必出（100% 强制全采样，**交易域强制全采样白名单**——per BAS-004 v0.3 §6.2） | 含`trade_id`／`initiator_id`／`target_id`／`frozen_assets_hash`／`expire_at`；约 350B/条 × 50/s = 17.5KB/s 稳态 |
| `trade.state.draft_to_offered.rejected` | 资产不足/已被占用导致 `Draft → Offered` 拒绝 | 偶发 | release 必出（100% 强制全采样） | 含`trade_id`／`initiator_id`／`reject_reason`（insufficient_frozen/occupied）；约 280B/条 |
| `trade.state.offered_to_accepted.committed` | 目标玩家显式接受（`Offered → Accepted`） | 1-50/s | release 必出（100% 强制全采样，**交易域强制**） | 含`trade_id`／`initiator_id`／`target_id`／`accept_at`／`snapshot_version`；约 300B/条 |
| `trade.state.offered_to_accepted.rejected` | 挂单已过期/已撤销导致 `Offered → Accepted` 拒绝 | 偶发 | release 必出（100% 强制全采样） | 含`trade_id`／`current_state`（expired/cancelled）／`reject_reason`；约 250B/条 |
| `trade.state.offered_to_cancelled.by_initiator` | 发起方主动撤销（`Offered → Cancelled`） | 1-20/s | release 必出（100% 强制全采样，**交易域强制**） | 含`trade_id`／`initiator_id`／`cancel_at`／`thawed_assets_count`；约 280B/条 |
| `trade.state.offered_to_expired.auto` | 定时任务扫描发现 `Offered` 且 `expire_at` 已过（`Offered → Expired`，系统自动迁移） | 1-10/分钟 | release 必出（100% 强制全采样，**交易域强制**） | 含`trade_id`／`expire_at`／`scan_batch_id`／`thawed_assets_count`；约 250B/条 |
| `trade.state.accepted_to_settled.committed` | 原子成立流程完成（`Accepted → Settled`，详§4.1） | 1-50/s | release 必出（100% 强制全采样，**交易域强制** + `trace_id` 关联 §4.1） | 含`trade_id`／`settle_at`／`trace_id`／`fee_deducted`；约 320B/条 |
| `trade.state.transition.idempotent_replay` | 重复提交同一"接受"操作，命中 `trade_id+state` 幂等键直接返回既有结果（FR-TRD-012） | 偶发 | release 必出（100% 强制全采样） | 含`trade_id`／`replay_state`／`original_settle_at`；约 200B/条 |
| `trade.state.debug.full_state_dump` | 状态机完整状态 dump（含全部 `initiator_items` / `target_items` / `snapshot_version` 详情） | 1-50/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-8KB/条（release 剔除，**含双方资产详情严禁进生产**） |
| `trade.state.debug.transition_graph_snapshot` | 状态机迁移图快照（用于离线回放与状态机正确性分析） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.state.debug.full_state_dump` **含双方全部资产详情**（物品 ID / 数量 / 货币额）—— release build **必须**完全剔除，**严禁**进生产日志通道（即使 RUST_LOG=debug 误开）
- 8 类 release 必出事件均为 `info!` 级别（release 常驻，per §4.8.3.2 二维矩阵 `info!` 行），SRE 需在 Loki/Grafana 按 `trade_id` 维度做单笔交易时间线回放——本节是交易域**可审计性**的运行时事实依据

---

## 2.2 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `TradeOfferService` | EC | 挂单创建/撤销/超时管理，资产冻结/解冻 |
| `TradeSettlementSaga` | EC（复用ARC-011既定Saga边界） | `Accepted`后的原子成立编排：双方资产转移+补偿逻辑 |
| `TradeAuditLog` | EC | 复用RGS-BAS-003§7审计设计存储结构 |
| `TradeVisibilityGuard` | EC | 挂单创建前校验目标玩家是否在允许范围内（好友/同队伍，具体范围随TBD-TRD-001评审结果配置），拒绝范围外的挂单创建请求（FR-TRD-006） |

### 2.2 本功能日志设计

本节覆盖**4 个核心组件（`TradeOfferService` / `TradeSettlementSaga` / `TradeAuditLog` / `TradeVisibilityGuard`）的资产冻结/解冻/审计写/可见性校验**的运行时观察点——**资产不可逆**特性要求资产侧事件 release 必出 + 100% 强制全采样（与 NFR-EC-001 资产安全 + FR-TRD-015 审计强需求挂钩），与 §2.1 状态机事件是同一序列的不同字段。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.offer_service.freeze_assets.committed` | `TradeOfferService` 完成发起方资产冻结（`Draft → Offered` 路径必经） | 1-50/s | release 必出（100% 强制全采样，**交易域强制全采样白名单**——per BAS-004 v0.3 §6.2） | 含`trade_id`／`initiator_id`／`frozen_assets_hash`／`freeze_at`；约 320B/条 × 50/s = 16KB/s 稳态 |
| `trade.offer_service.freeze_assets.failed` | 资产冻结失败（DB 死锁/连接池耗尽/资金不足） | 偶发 | release 必出（100% 强制全采样） | 含`trade_id`／`initiator_id`／`error`／`trace_id`；约 300B/条 |
| `trade.offer_service.thaw_assets.committed` | 资产解冻成功（`Cancelled`/`Expired`/`Compensated` 路径必经） | 1-30/s | release 必出（100% 强制全采样，**交易域强制**） | 含`trade_id`／`player_id`／`thaw_reason`（cancelled/expired/compensated）/`thawed_assets_count`；约 300B/条 |
| `trade.saga.settlement.started` | `TradeSettlementSaga` 收到 `Accepted → Settled` 编排请求 | 1-50/s | release 必出（100% 强制全采样，**交易域强制** + `trace_id` 关联） | 含`trade_id`／`snapshot_version`／`started_at`／`trace_id`；约 280B/条 |
| `trade.saga.settlement.completed` | Saga 全部 4 步资产转移完成（`Accepted → Settled` 成功） | 1-50/s | release 必出（100% 强制全采样，**交易域强制** + `trace_id` 关联） | 含`trade_id`／`completed_at`／`duration_ms`／`trace_id`；约 280B/条 |
| `trade.saga.compensation.executed` | Saga 补偿路径触发（任一步失败回滚已执行步骤） | 偶发 | release 必出（100% 强制全采样，**交易域强制** + `trace_id` 关联） | 含`trade_id`／`failed_step`／`rolled_back_steps`／`trace_id`；约 350B/条 |
| `trade.audit_log.write.committed` | `TradeAuditLog` 行写入成功（按 FR-TRD-015 7 种 `event_type` 全部出） | 1-200/s（每次状态迁移 + 补偿 + 升级 1 行） | release 必出（100% 强制全采样，**交易域强制**——审计写不允许采样降级） | 含`trade_id`／`event_type`／`actor_id`／`occurred_at`；约 250B/条 × 200/s = 50KB/s 稳态 |
| `trade.audit_log.write.failed` | `TradeAuditLog` 写入失败（DB 约束冲突/连接断） | 极少 | release 必出（100% 强制全采样 + `error!`） | 含`trade_id`／`event_type`／`error`／`trace_id`；约 300B/条；**审计写失败 P0 告警**（同 BAS-003 §7 关键设计纪律） |
| `trade.visibility.check.passed` | `TradeVisibilityGuard` 校验通过（`trade_visibility_scope` 范围内） | 1-50/s | release 必出（100% 强制全采样，**交易域强制**——FR-TRD-006 落地证据） | 含`trade_id`／`initiator_id`／`target_id`／`scope_evaluated`；约 280B/条 |
| `trade.visibility.check.failed` | `TradeVisibilityGuard` 校验拒绝（不在范围内） | 偶发 | release 必出（100% 强制全采样 + `warn!`，**交易域强制**） | 含`trade_id`／`initiator_id`／`target_id`／`scope_evaluated`／`reject_reason`；约 300B/条 |
| `trade.offer_service.debug.freeze_assets_full_dump` | 冻结资产完整详情（物品 ID/数量/货币额/装备绑定状态） | 1-50/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-10KB/条（release 剔除，**含双方资产详情严禁进生产**） |
| `trade.saga.debug.compensation_step_dump` | Saga 补偿各步骤的 partial state（含已写入的 partial asset 行） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |
| `trade.audit_log.debug.event_payload` | `TradeAuditLog` 行的 `snapshot_at_event` 完整 jsonb payload | 1-200/s | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-2KB/条（release 剔除） |
| `trade.visibility.debug.scope_config_dump` | `trade_visibility_scope` 配置项完整 dump（含 `friend_only`/`party_only`/`friend_or_party` 三态） | 偶发（配置变更时） | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-1KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.offer_service.debug.freeze_assets_full_dump` 在大额交易下可能 10KB+ —— release build 完全剔除，避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `trade.audit_log.write.failed` 是**审计写失败 P0 告警**（同 BAS-003 §7 关键设计纪律）—— release 必出 + 强制全采样，便于 P0 告警链路立即捕获并触发资产一致性回查
- 4 个组件的 release 必出事件均为 `info!` 级别（per §4.8.3.2 二维矩阵 `info!` 行常驻），便于 SRE 按 `trade_id` 维度做跨组件时间线拼接

---

## 2.3 交易目标可见性校验（FR-TRD-006落地）

`TradeVisibilityGuard`在`TradeOfferService`创建挂单前同步调用，校验规则以配置项`trade_visibility_scope`表达（枚举：`friend_only`／`party_only`／`friend_or_party`，具体取值随TBD-TRD-001与策划评审结果确定，评审前默认`friend_or_party`）。校验失败时`Draft → Offered`迁移直接拒绝，不产生资产冻结副作用（不消耗FR-TRD-002冻结路径）。该校验为**同步阻塞**校验（区别于任务系统FR-GSM-015异步进度更新），因为挂单创建本身即为低频操作，无需异步化。

> **层级修正说明（v0.3）**：原`### 2.3`提升为`## 2.3`以与§2.1/§2.2保持H2层级一致；本节日志设计小节编号保持`### 2.3`形式。

### 2.3 本功能日志设计

本节覆盖**`TradeVisibilityGuard` 目标可见性校验（含 3 态配置项 `trade_visibility_scope` 评估 + 范围外拒绝）**的运行时观察点——本节是 FR-TRD-006 落地的运行时事实依据，且**配置项变更属治理事件**（影响交易成立资格边界），需 release 必出。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.visibility.check.passed` | `TradeVisibilityGuard` 校验通过（`trade_visibility_scope` 范围内） | 1-50/s | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2，**FR-TRD-006 落地证据**） | 含`trade_id`／`initiator_id`／`target_id`／`scope_evaluated`；约 280B/条 |
| `trade.visibility.check.failed` | `TradeVisibilityGuard` 校验拒绝（目标玩家不在 `trade_visibility_scope` 范围内） | 偶发 | release 必出（100% 强制全采样 + `warn!`，per BAS-004 v0.3 §6.2） | 含`trade_id`／`initiator_id`／`target_id`／`scope_evaluated`／`reject_reason`（out_of_scope_*/friend_list_miss/party_miss）；约 300B/条 |
| `trade.visibility.scope.config_changed` | `trade_visibility_scope` 配置项变更（3 态之一切换：friend_only/party_only/friend_or_party） | 极低（季度评审触发） | release 必出（100% 强制全采样，**治理事件**——per BAS-004 v0.3 §6.2） | 含`old_value`／`new_value`／`changed_by`／`effective_at`；约 250B/条 |
| `trade.visibility.scope.evaluated_per_call` | 每次 `TradeVisibilityGuard` 调用时的 3 态配置项当前值 dump | 1-50/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除）——高频读取避免撑爆生产日志通道 | 约 200B/条（release 剔除） |
| `trade.visibility.debug.friend_party_list_dump` | 校验时双方好友列表 / 队伍成员列表完整 dump（用于离线复审"为何 out_of_scope"） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-3KB/条（release 剔除，**含玩家关系数据严禁进生产**） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.visibility.debug.friend_party_list_dump` 在大型好友列表/队伍下可能 3KB+ —— release build 完全剔除，避免 RUST_LOG=debug 误开时泄漏玩家社交关系
- `trade.visibility.scope.config_changed` 是**治理事件**（与 TBD-TRD-001 评审结果绑定）—— release 必出 + §6.2 强制全采样，便于 SRE/Gov 团队按 `changed_by` 维度追溯历次范围变更

---



## 2.4 処理フロー（处理流程 / Processing Flow）

> 落实 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板)
> 详细时序见 §4 交易成立时序, 本段为全景流程 + 异常分支 + 决策点 + 验证点汇总

### 2.4.1 主流程图 (mermaid sequenceDiagram)

```mermaid
sequenceDiagram
    autonumber
    actor Initiator as 发起方玩家
    actor Target as 目标方玩家
    participant TOS as TradeOfferService
    participant TVG as TradeVisibilityGuard
    participant TOSM as TradeOfferStateMachine
    participant DB as player_db (TradeOffer/TradeAuditLog)
    participant ECN as EC 域 (FR-EC-003 路径)
    participant TSS as TradeSettlementSaga
    participant GMQ as GM 人工核账队列

    Note over Initiator,GMQ: trace_id 贯穿全链路, per BAS-004 v0.3 §4.4
    Note over Initiator,GMQ: 事务边界: 资产原子转移同事务; EC 跨域走 Saga, per BAS-100 v0.1
    Note over Initiator,GMQ: FR-TRD-006 可见性校验为同步阻塞, 与异步任务进度更新区分

    rect rgb(240, 248, 255)
        Note over Initiator,DB: 主路径 1: 挂单创建与可见性校验 (Draft → Offered)
        Initiator->>TOS: 发起挂单 (initiator_items)
        TOS->>TVG: 同步校验目标可见性 (FR-TRD-006)
        alt 范围外 (out_of_scope_*)
            TVG-->>TOS: 拒绝 (无冻结副作用)
            TOS-->>Initiator: 拒绝"目标不可见"
        else 范围内
            TOS->>TOSM: Draft → Offered 状态迁移
            TOSM->>DB: 冻结 initiator_items (BEGIN)
            TOSM->>DB: 插入 TradeOffer row (snapshot_version=1)
            TOSM->>DB: 追加 TradeAuditLog (event_type=created)
            TOSM->>DB: COMMIT
            TOSM-->>TOS: 挂单成功 (trade_id, expire_at)
            TOS-->>Initiator: 返回 trade_id
            TOS->>Target: 通知挂单 (复用 ARC-010 事件基础设施)
        end
    end

    rect rgb(255, 250, 240)
        Note over Target,DB: 主路径 2: 接受与过期取消 (Offered → Accepted / Cancelled / Expired)
        Target->>TOS: 接受挂单 (trade_id)
        TOS->>TOSM: 校验当前 state == Offered
        alt 已不在 Offered (并发)
            TOSM-->>TOS: 拒绝"挂单状态已变"
            TOS-->>Target: 拒绝
        else Offered
            TOSM->>DB: snapshot_version 锁定 (FR-TRD-014 乐观锁基线)
            TOSM->>TOSM: Offered → Accepted 状态迁移
            TOSM->>DB: 追加 TradeAuditLog (event_type=accepted)
            TOSM->>DB: COMMIT
            TOSM-->>TOS: 已接受
            TOS-->>Target: 已接受
            TOS-->>Initiator: 通知已接受
        end
    end

    rect rgb(240, 255, 240)
        Note over Target,GMQ: 主路径 3: 交易成立与原子结算 (Accepted → Settled, 含 Saga 补偿)
        Target->>TOS: 触发结算 (trade_id, snapshot_version)
        TOS->>TSS: 启动 TradeSettlementSaga
        TSS->>DB: 乐观锁校验 snapshot_version (FR-TRD-014)
        alt 快照失效 (并发操作使其失效)
            TSS-->>TOS: 拒绝"快照已失效" (FR-TRD-014)
            TOS-->>Target: 拒绝 + 提示重新挂单
        else 快照有效
            TSS->>ECN: 调用 FR-EC-003 路径 (Saga 步骤 1: deduct_initiator)
            ECN-->>TSS: success
            TSS->>ECN: 步骤 2: deduct_target
            ECN-->>TSS: success
            TSS->>ECN: 步骤 3: grant_initiator
            ECN-->>TSS: success
            TSS->>ECN: 步骤 4: grant_target
            ECN-->>TSS: success
            TSS->>DB: BEGIN 原子事务
            TSS->>DB: Accepted → Settled 状态迁移
            TSS->>DB: 追加 TradeAuditLog (event_type=settled)
            TSS->>DB: COMMIT
            TSS-->>TOS: 结算成功
            TOS-->>Target: 核销成功
            TOS-->>Initiator: 通知结算成功
        end
    end

    rect rgb(255, 240, 240)
        Note over Target,GMQ: 主路径 4: 补偿与升级 (Saga 失败 / RSK-TRD-002 最坏分支)
        TSS->>ECN: 任一步失败
        ECN-->>TSS: 失败 (步骤 N)
        TSS->>ECN: 反向补偿 N-1...1 (回滚资产)
        alt 补偿成功
            TSS->>DB: 状态保持 Accepted (供重试)
            TSS->>DB: 追加 TradeAuditLog (event_type=compensated)
            TSS-->>TOS: 提示"服务暂不可用请重试"
        else 补偿失败 (RSK-TRD-002 最坏分支)
            TSS->>DB: 强制迁移至 CompensationFailed 中间态
            TSS->>DB: 追加 TradeAuditLog (event_type=escalated)
            TSS->>GMQ: 推入人工核账队列
            TSS-->>TOS: 提示"已转人工核实"
        end
    end

    Note over Initiator,GMQ: 异常通路 (DLQ + 重试): EC 域不可达 -> ARC-009 消费者标准模式 (重试 3 次 指数退避 100/200/400ms) -> DLQ 报警
```

### 2.4.2 異常分支表

| 异常点 | 触发条件 | 处理动作 | 用户感知 | 补偿动作 |
|---|---|---|---|---|
| 目标可见性校验失败 | TradeVisibilityGuard 评估 target 不在 	rade_visibility_scope 范围 | Draft → Offered 迁移直接拒绝 (无冻结副作用, per FR-TRD-006) | 提示"目标不可见" | 无 (无副作用) |
| 资产冻结失败 | initiator 资产已被其他途径占用 / 余额不足 | Draft → Offered 迁移拒绝 | 提示"资产已被占用" | 无 (无副作用) |
| 接受时状态非 Offered | 并发操作导致 state 已变 (Cancelled / Expired / Accepted) | 拒绝"挂单状态已变" | 提示"挂单已变化" | 无 (无副作用) |
| 乐观锁校验失败 | snapshot_version 不匹配 (FR-TRD-014 触发, 双花/调包防护) | 拒绝进入原子事务 | 提示"快照已失效，请重新挂单" | 无 (无副作用) |
| EC 域步骤失败 | FR-EC-003 路径任一步返回失败 (步骤 1-4) | Saga 反向补偿 N-1...1 | 提示"服务暂不可用" | 资产恢复至冻结前状态, 状态保持 Accepted 供重试 |
| 事务提交失败 | DB 写失败 (网络/约束冲突/死锁) | 整体回滚 | 提示"结算失败请重试" | 客户端重试 (幂等键 	rade_id+state, per FR-TRD-012) |
| 补偿本身失败 (RSK-TRD-002) | Saga 补偿路径任一步也失败 (回滚时资产写入失败) | 强制迁移至 CompensationFailed 中间态 | 提示"已转人工核实" | GM 人工核账队列介入 (per FR-TRD-016), 期间禁止相关资产被其他操作占用 |
| 重复提交幂等命中 | 	rade_id+state 已为 Settled 重复提交 | 直接返回既有 Settled 结果 (FR-TRD-012) | 重复提交无副作用 | 无 |

### 2.4.3 决策点矩阵

| 决策点 | 条件 | 主分支 | 备选分支 | 触发后果 |
|---|---|---|---|---|
| 可见性范围评估 | 	rade_visibility_scope 配置 (friend_only/party_only/friend_or_party) + 目标玩家关系 | 范围内 → 允许挂单 | 范围外 → 拒绝 (无冻结) | 用户感知: 进入挂单流程 / 拒绝"目标不可见" |
| 接受时状态校验 | 挂单 state == Offered | 继续 (锁定 snapshot_version) | state ≠ Offered → 拒绝 | 用户感知: 进入结算流程 / 拒绝"挂单已变化" |
| 乐观锁校验 | UPDATE ... WHERE trade_id=? AND snapshot_version=? 受影响行数 | = 1 → 进入原子事务 | = 0 → 拒绝 (FR-TRD-014, 快照失效) | 用户感知: 进入结算 / 拒绝"快照已失效" |
| Saga 失败补偿策略 | EC 步骤 N 失败 | 反向补偿 N-1...1, 状态保持 Accepted | 部分补偿 + DLQ 人工介入 | 用户感知: 资产自动回退 (per ARC-009) |
| 反作弊联动触发 | 反作弊规则检测 (高频/大额/异常时序, RSK-TRD-002 联动) | 拒绝该笔挂单 + 写 	rade.anti_fraud.* | 仅告警 (低风险) | 用户感知: 拒绝 (反作弊), 反作弊团队可按 player_id 维度做行为画像 |
| 补偿失败升级 (RSK-TRD-002) | Saga 补偿本身失败 | 强制迁移至 CompensationFailed + 推入 GM 人工核账队列 | 不升级 (错误, 资产不一致) | 用户感知: 提示"已转人工", 资产冻结至人工核实完成 |

### 2.4.4 验证点清单

| 验证时机 | 验证内容 | 通过标准 | 失败处理 |
|---|---|---|---|
| 可见性校验 | TradeVisibilityGuard 评估结果 | target 在 	rade_visibility_scope 范围内 | 拒绝挂单 (无冻结副作用), 记录 	rade.visibility.check.failed |
| 资产冻结 (Draft → Offered) | initiator 资产可冻结 (未被占用, 余额/物品充足) | 冻结成功 | 拒绝迁移, 记录 	rade.state.draft_to_offered.rejected |
| 接受时状态 | 挂单 state == Offered | 严格相等 | 拒绝"挂单已变化", 记录 	rade.state.offered_to_accepted.rejected |
| 乐观锁校验 (FR-TRD-014) | UPDATE ... WHERE trade_id=? AND snapshot_version=? 受影响行数 | = 1 (无并发失效) | 拒绝进入原子事务, 记录 	rade.settlement.snapshot.stale_rejected (反作弊联动) |
| Saga 步骤完成 | EC 4 步全部成功 (deduct_initiator/deduct_target/grant_initiator/grant_target) | 4/4 成功 | 反向补偿已执行步骤, 记录 	rade.settlement.atomic_transfer.failed |
| 事务提交 (Accepted → Settled) | TradeOffer + TradeAuditLog 同事务写入 | tx_id COMMIT 成功 | 整体回滚, 记录 	rade.settlement.transaction_rolled_back |
| 补偿执行 | Saga 反向补偿 N-1...1 全部成功 | 资产恢复至冻结前状态 | 升级至 CompensationFailed + GM 人工核账队列, 记录 	rade.settlement.compensation_failed.escalated (P0 告警) |
| 幂等性 (FR-TRD-012) | 	rade_id+state 重复提交命中 | state 已是 Settled 时直接返回既有结果 | 不重复执行, 记录 	rade.settlement.idempotent_replay.detected |

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

## 3.1 本功能日志设计

本节覆盖**`TradeOffer` / `TradeAuditLog` 两张核心表的 DDL 生命周期（建表/索引/分区/迁移/归档）**的运行时观察点——`TradeAuditLog` 是**不可变审计表**（per FR-TRD-015），其分区归档操作须 release 必出 + 强制全采样（与 NFR-TRD-004 默认 1 年保留期 + FR-TRD-016 强制审计不可丢挂钩）。`TradeOffer` 表的乐观锁 OCC 冲突事件须 release 必出（与 RSK-TRD-002 双花/调包防护挂钩）。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.ddl.trade_offer_table_created` | `TradeOffer` 表 DDL 在生产首次执行（CI 迁移） | 极低（一次性 + 重大 schema 变更） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含`table_name`／`column_count`／`index_count`／`migration_id`；约 350B/条 |
| `trade.ddl.trade_audit_log_table_created` | `TradeAuditLog` 表 DDL 在生产首次执行（不可变审计表，per FR-TRD-015） | 极低（一次性） | release 必出（100% 强制全采样，**审计表生命周期不可丢**） | 含`table_name`／`partition_strategy`（monthly_by_occurred_at）／`retention_days`（365=NFR-TRD-004）；约 380B/条 |
| `trade.ddl.index_created` | `(initiator_id, state)` / `(target_id, state)` / `(state, expire_at)` / `(trade_id, occurred_at)` / `(actor_id, occurred_at)` 5 个复合索引创建成功 | 极低（一次性） | release 必出（100% 强制全采样） | 含`index_name`／`column_names`／`migration_id`；约 300B/条 |
| `trade.ddl.partition_detached` | `TradeAuditLog` 月度分区按 NFR-TRD-004 1 年保留期自动 detach 归档 | 1/月 | release 必出（100% 强制全采样，**审计归档不可丢**） | 含`partition_name`／`detach_at`／`row_count_archived`；约 280B/条 |
| `trade.ddl.migration_applied` | 任何 `trade_offer*` / `trade_audit_log*` 的 schema 迁移在生产 apply 成功 | 1-5/季度 | release 必出（100% 强制全采样） | 含`migration_id`／`migration_name`／`applied_at`／`applied_by`；约 300B/条 |
| `trade.ddl.migration_failed` | 任何 `trade_offer*` / `trade_audit_log*` 的 schema 迁移失败（约束冲突/锁等待超时） | 极少 | release 必出（100% 强制全采样 + `error!`） | 含`migration_id`／`error`／`trace_id`；约 350B/条 |
| `trade.offer.occ_conflict_detected` | `TradeOffer` 行并发更新时 `snapshot_version` 不匹配（RSK-TRD-002 双花/调包防护触发） | 偶发 | release 必出（100% 强制全采样 + `warn!`，**反作弊联动**） | 含`trade_id`／`expected_version`／`actual_version`／`concurrent_actor_id`；约 320B/条 |
| `trade.audit_log.archive_purge_blocked` | 自动归档任务尝试 purge `TradeAuditLog` 已 detach 分区被阻断（FR-TRD-016 不可丢原则） | 极少 | release 必出（100% 强制全采样 + `error!`） | 含`partition_name`／`blocker_rule`（retention_under_1y/external_hold）／`trace_id`；约 300B/条 |
| `trade.ddl.debug.migration_sql_redacted_dump` | 迁移 SQL 完整 dump（**敏感字段已脱敏**——仅含 DDL 不含数据） | 1-5/季度 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-2KB/条（release 剔除） |
| `trade.ddl.debug.partition_health_dump` | `TradeAuditLog` 各分区的健康度 dump（行数/索引大小/压缩率） | 1/月 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 3-10KB/条（release 剔除） |
| `trade.ddl.debug.explain_plan_dump` | 高频 OCC 冲突查询的执行计划 dump（用于索引复核） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.ddl.debug.partition_health_dump` 在大型分区下可能 10KB+ —— release build 完全剔除，避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `trade.audit_log.archive_purge_blocked` 是**审计完整性事件**（per FR-TRD-016）—— `error!` 级别 + release 必出 + §6.2 强制全采样，便于 DBA 团队按 `blocker_rule` 维度追溯历次阻断
- `trade.offer.occ_conflict_detected` 触发即代表双花/调包防护生效——release 必出 + `warn!`（**非** `error!`，属正常防护动作），便于反作弊系统按 `concurrent_actor_id` 维度做高频模式识别

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

## 4.1 本功能日志设计

本节覆盖**`TradeSettlementSaga` 原子成立流程（snapshot_version 乐观锁 + 双账户资产原子转移 + Saga 补偿 + RSK-TRD-002 升级分支）**的运行时观察点——本节是交易域**最关键**的可观察面，与 NFR-EC-001 资产安全 + RSK-TRD-002 双花/调包防护 + NFR-AV-005 可用性 + 财务数据完整性 多重强需求挂钩。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.settlement.snapshot.version_locked` | `Accepted` 时刻锁定 `snapshot_version`（防调包基线建立） | 1-50/s | release 必出（100% 强制全采样，**交易域强制全采样白名单**——per BAS-004 v0.3 §6.2） | 含`trade_id`／`snapshot_version`／`initiator_assets_hash`／`target_assets_hash`／`locked_at`；约 350B/条 × 50/s = 17.5KB/s 稳态 |
| `trade.settlement.snapshot.stale_rejected` | 乐观锁校验失败（FR-TRD-014：快照已被并发操作使其失效，拒绝进入原子事务） | 偶发 | release 必出（100% 强制全采样 + `warn!`，**反作弊联动**——双花/调包防护生效证据） | 含`trade_id`／`expected_version`／`actual_version`／`stale_field`（initiator_assets/target_assets）/`trace_id`；约 320B/条 |
| `trade.settlement.atomic_transfer.completed` | 双方 4 步资产转移 + 手续费（若启用）原子事务全部成功（`Accepted → Settled`） | 1-50/s | release 必出（100% 强制全采样，**交易域强制** + `trace_id` 关联——财务一致性证据） | 含`trade_id`／`initiator_id`／`target_id`／`settle_at`／`duration_ms`／`fee_deducted`／`trace_id`；约 380B/条 |
| `trade.settlement.atomic_transfer.failed` | 原子事务任一步失败（如账户余额不足/物品已不在背包/DB 死锁） | 偶发 | release 必出（100% 强制全采样 + `error!`） | 含`trade_id`／`failed_step`（deduct_initiator/deduct_target/grant_initiator/grant_target/fee）/`error`／`trace_id`；约 380B/条 |
| `trade.settlement.compensation.executed` | Saga 补偿路径触发（按§4 全部已执行步骤回滚，资产恢复至冻结前状态） | 偶发 | release 必出（100% 强制全采样，**交易域强制** + `trace_id` 关联——资产可恢复性证据） | 含`trade_id`／`rolled_back_steps`／`comp_at`／`trace_id`；约 350B/条 |
| `trade.settlement.compensation_failed.escalated` | **RSK-TRD-002 最坏分支触发**（补偿本身失败，如回滚时资产写入也失败），状态强制迁移至 `CompensationFailed` 中间态 | 极少（金融事故级） | release 必出（100% 强制全采样 + `error!` + **P0 告警**，**交易域强制**） | 含`trade_id`／`initiator_id`／`target_id`／`comp_failure_step`／`escalated_at`／`trace_id`；约 420B/条；P0 告警链路立即触发 GM 人工核账队列（FR-TRD-016 + §5.1） |
| `trade.settlement.fee.deducted` | TBD-TRD-002 手续费率非 0 时从双方账户扣除交易费（`fee_rate` 默认 0） | 1-50/s（若启用） | release 必出（100% 强制全采样，**财务入账证据**） | 含`trade_id`／`fee_payer`（initiator/target/split）/`fee_amount`／`fee_currency`；约 280B/条 |
| `trade.settlement.idempotent_replay.detected` | 重复提交"接受"操作命中 `trade_id+state` 幂等键（FR-TRD-012，返回既有 `Settled` 结果不重复执行） | 偶发 | release 必出（100% 强制全采样） | 含`trade_id`／`replay_state`／`original_settle_at`；约 220B/条 |
| `trade.anti_fraud.high_frequency_detected` | 同一玩家短窗口（如 1 分钟）内发起挂单数超过阈值 | 极少（反作弊联动） | release 必出（100% 强制全采样 + `error!`，**反作弊联动**——与既有反作弊链路联锁） | 含`player_id`／`trade_count`／`window_seconds`／`threshold`／`trade_ids`；约 350B/条 |
| `trade.anti_fraud.large_value_alert` | 单笔挂单总价值超过反作弊大额阈值 | 极少（反作弊联动） | release 必出（100% 强制全采样 + `error!`，**反作弊联动**） | 含`trade_id`／`initiator_id`／`target_id`／`total_value`／`threshold`；约 320B/条 |
| `trade.anti_fraud.abnormal_sequence_detected` | 异常时序模式（如同一对玩家短时间内反复挂单-撤销）触发反作弊规则 | 极少（反作弊联动） | release 必出（100% 强制全采样 + `error!`，**反作弊联动**） | 含`player_pair`／`sequence_pattern`／`occurrence_count`；约 300B/条 |
| `trade.settlement.debug.full_saga_state_dump` | Saga 全部 4 步中间状态 + 补偿路径的完整 dump | 1-50/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 3-15KB/条（release 剔除，**含双方全部资产中间状态严禁进生产**） |
| `trade.settlement.debug.compensation_partial_assets` | Saga 补偿过程中已写入的 partial asset 行 dump（含具体物品 ID/数量） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |
| `trade.matching_engine.debug.price_curve_dump` | 撮合引擎价格曲线 / 供需比 dump（**性能敏感**——撮合 tick 高频，**仅** debug-only 守护） | 100-1000/s（撮合 tick 高频） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.2-1KB/条（release 剔除，**严禁进生产日志通道**——撑爆风险） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.settlement.debug.full_saga_state_dump` 在大额交易下可能 15KB+ —— release build 完全剔除，避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `trade.matching_engine.debug.price_curve_dump` 是**撮合 tick 高频**事件（100-1000/s）—— release 完全剔除是性能硬性要求，**严禁**进生产日志通道
- `trade.settlement.compensation_failed.escalated` 是**金融事故级** P0 告警—— `error!` 级别 + release 必出 + §6.2 强制全采样 + GM 人工核账队列联动，便于事故响应团队按 `trade_id` 维度做资产追回
- 3 类反作弊联动事件（`trade.anti_fraud.*`）触发即代表反作弊规则触发—— release 必出 + `error!` + §6.2 强制全采样，便于反作弊系统按 `player_id` 维度做行为画像

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
- [ ] **每功能章节（§2.1／§2.2／§2.3／§3.1／§4.1）均含"本功能日志设计"子节**，且明确区分 `info!`/`warn!`/`error!`（release 必出，§6.2 强制全采样白名单）与 `debug!`/`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only，release build 完全剔除零运行时开销）两类事件（per AC-TRD-006 / AC-TRD-007）
- [ ] **release 必出事件 grep 验证通过**：CI 静态扫描全部 `trade.*` 命名空间下非 `.debug.` 前缀的事件均存在于源代码（per BAS-004 v0.3 §11.1）
- [ ] **debug-only 四铁律合规验证通过**：CI 静态检查全部 `trade.*.debug.*` 宏均被 `#[cfg(debug_assertions)]` 守护（per BAS-004 v0.3 §4.4 + §9 第 5 项）
- [ ] **release 必出宏未被 cfg 守护验证通过**：CI 静态检查 `info!`/`warn!`/`error!` 宏（`trade.*` 命名空间下非 `.debug.` 前缀）未被 `#[cfg]` 守护（与 BAS-004 v0.3 §4.4 反例对照）

### 5.1 本功能日志设计

本节覆盖**上线前 7 项检查项的执行跟踪**的观察点——上线前检查清单的逐项执行/失败/全部通过是**上线放行**的运行时事实依据，每项检查的执行结果 release 必出便于 SRE/Audit 团队按 `run_id` 维度追溯历次上线流程。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.checklist.pre_launch.run_started` | 上线前检查清单执行启动（按 `run_id` 聚合） | 1/上线 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含`run_id`／`started_at`／`started_by`；约 250B/条 |
| `trade.checklist.pre_launch.item.passed` | 上线前检查清单任一项执行通过（7 项之一：故障注入/超时解冻/幂等/可见性范围/GM后台接口/VisibilityGuard上线/CompensationFailed联调） | 1/上线 × 7 项 | release 必出（100% 强制全采样） | 含`run_id`／`item_kind`（atomic_settlement_injection/expire_thaw/idem/visibility_scope/gm_query/guard_config/comp_failed_linkage）/`checked_by`；约 320B/条 |
| `trade.checklist.pre_launch.item.failed` | 上线前检查清单任一项执行失败（如故障注入出现单方受损） | 1/上线 × 偶发 | release 必出（100% 强制全采样 + `error!`，**上线阻断**） | 含`run_id`／`item_kind`／`failure_reason`／`trace_id`；约 380B/条 |
| `trade.checklist.pre_launch.all_completed` | 上线前 7 项检查全部通过（上线放行信号） | 1/上线 | release 必出（100% 强制全采样，**上线放行**信号——per BAS-004 v0.3 §6.2） | 含`run_id`／`completed_at`／`all_passed`（true/false）／`total_items`；约 250B/条 |
| `trade.checklist.pre_launch.evidence_attachment` | 上线前检查的证据材料附件引用（测试报告/联调纪要等） | 1/上线 × 7 项 | release 必出（100% 强制全采样） | 含`run_id`／`item_kind`／`evidence_url`（内部对象存储签名 URL）/`uploaded_by`；约 300B/条 |
| `trade.checklist.debug.fault_injection_log` | 故障注入试验的完整日志 dump（用于离线复审"无单方受损"） | 1/上线 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 5-50KB/条（release 剔除，依赖注入场景数量） |
| `trade.checklist.debug.compensation_failed_linkage_dump` | `CompensationFailed` 联调验证的完整事件序列 dump | 1/上线 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-5KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.checklist.debug.fault_injection_log` 在多场景注入下可能 50KB+ —— release build 完全剔除，避免 RUST_LOG=debug 误开时撑爆生产日志通道
- `trade.checklist.pre_launch.item.failed` 是**上线阻断级**事件（per §5.1 全部项须通过才能放行）—— `error!` 级别 + release 必出 + §6.2 强制全采样，便于 release manager 按 `item_kind` 维度快速定位失败项
- 7 类检查项的 `passed` 事件均为 `info!` 级别（per §4.8.3.2 二维矩阵 `info!` 行常驻），便于上线审计团队按 `run_id` 维度回放历次上线流程

## 5.2 代码评审检查清单

- [ ] 交易价值转移路径未绕过FR-EC-003确定请求路径
- [ ] `Accepted`状态迁移后未出现允许反悔的代码路径
- [ ] **新增交易域 log 事件命名合规**：所有 `trade.*` 字段名严格 snake_case + 与 BAS-004 v0.3 §4.6.1/§4.6.2 拼写一致（FR-LOG-013），release 必出事件与 debug-only 事件路径前缀分离
- [ ] **新增交易域 log 事件调用合规**：未手写 `#[cfg(debug_assertions)] debug!(...)`（per BAS-004 v0.3 §4.4 规则 #1 与 §8 脚手架约束），必须使用脚手架预生成的 debug-only 模板片段
- [ ] **log 章节上线检查项已纳入 CI**（per §5.1 末 3 项 + BAS-004 v0.3 §11.1 新服务埋点接入检查清单 + §11.2 既有服务改造检查清单）

### 5.2 本功能日志设计

本节覆盖**代码评审 2 项核心检查的执行跟踪**的观察点——代码评审是**变更上线**的运行时事实依据（与既有 CI 流水线 + review tool 集成），每条规则的命中/未命中 release 必出便于审计团队按 `commit_sha` 维度追溯历次代码评审结果。沿用 BAS-001 v1.5 §4.8.3 模板 + BAS-004 v0.3 §4.3 字段规范 + §4.4 debug-only 守护 + §5.1 脱敏 + §6.2 强制全采样。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `trade.review.code.run_started` | 代码评审检查清单执行启动（按 `commit_sha` 关联） | 1/PR × 1-N commit | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含`review_run_id`／`commit_sha`／`started_at`／`reviewer_id`；约 280B/条 |
| `trade.review.code.item.passed` | 代码评审任一项通过（2 项之一：FR-EC-003 路径无绕过/Accepted 后无反悔路径） | 1/PR × 2 项 | release 必出（100% 强制全采样） | 含`review_run_id`／`commit_sha`／`item_kind`（fr_ec_003_no_bypass/accepted_no_revoke）/`reviewer_id`；约 320B/条 |
| `trade.review.code.item.flagged` | 代码评审任一项命中违规（**变更阻断**） | 偶发 | release 必出（100% 强制全采样 + `error!`，**变更阻断**信号） | 含`review_run_id`／`commit_sha`／`item_kind`／`offending_file`／`offending_line`／`reviewer_id`；约 380B/条 |
| `trade.review.code.all_completed` | 代码评审 2 项检查全部通过（变更放行信号） | 1/PR × commit | release 必出（100% 强制全采样，**变更放行**信号——per BAS-004 v0.3 §6.2） | 含`review_run_id`／`commit_sha`／`all_passed`（true/false）；约 250B/条 |
| `trade.review.code.commit_linked` | 代码评审结果与 `commit_sha` 关联入库（供后续追溯） | 1/PR × commit | release 必出（100% 强制全采样） | 含`commit_sha`／`trade_related_files_count`／`linked_at`；约 220B/条 |
| `trade.review.debug.diff_against_baseline` | 本次 commit 与基线版本的逐行 diff（仅含 `crates/*` 中交易域相关文件，**敏感字段已脱敏**——仅含逻辑代码不含密钥/凭证） | 1/PR | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB/条（release 剔除） |
| `trade.review.debug.violation_code_snippet` | 违规位置的代码片段（用于复审"为何命中反模式"） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 0.5-3KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `trade.review.debug.diff_against_baseline` 在大型 diff 下可能 10KB+ —— release build 完全剔除，避免 RUST_LOG=debug 误开时泄漏未发布代码
- `trade.review.code.item.flagged` 是**变更阻断级**事件（per §5.2 全部项须通过才能合并）—— `error!` 级别 + release 必出 + §6.2 强制全采样，便于 release manager 按 `offending_file` 维度快速定位违规位置
- 2 类检查项的 `passed` 事件均为 `info!` 级别（per §4.8.3.2 二维矩阵 `info!` 行常驻），便于审计团队按 `commit_sha` 维度回放历次代码评审

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
| **AC-TRD-006** | §2.1、§2.2、§2.3、§3.1、§4.1、§5.1、§5.2（debug-only 宏 release 完全剔除——跨 7 个"本功能日志设计"小节 + §5.1/§5.2 log 章节上线检查项多点验证，与 BAS-001 v1.5 §4.8.3.4 / BAS-007 v0.3 / BAS-009 v0.7 / BAS-004 v0.3 §12 形成统一规范） |
| **AC-TRD-007** | §2.1、§2.2、§2.3、§3.1、§4.1、§5.1、§5.2（每功能 BAS 文档须含本功能 log 设计章节，跨 7 个新小节 + §5.1/§5.2 多点验证） |
| **AC-TRD-008（処理フロー四要素）** | §2.4（mermaid sequenceDiagram 8 actor + 異常分支表 8 行 + 决策点矩阵 6 行 + 验证点清单 8 行，与 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 §3 必含四要素一致；trace_id 贯穿全链路 per BAS-004 v0.3 §4.4；事务边界 + Saga 跨域标注 per BAS-100 v0.1；与 BAS-019 v0.4 §1.1 范式对齐） |

---

> 本文档与RGS-REQ-018（玩家间交易系统 需求定义书）配套使用。
