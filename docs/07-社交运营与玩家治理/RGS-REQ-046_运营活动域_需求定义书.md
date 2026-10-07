# 需求定义书（要件定義書 / Requirements Definition Document）

**运营活动域（Operate Domain）需求定义**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-046 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-REQ-001 v1.5 §4 业务需求 + §5 功能需求；RGS-REQ-013 v1.4 横切关注点 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联下游 | RGS-BAS-046 / RGS-DTL-053 / operate/v1/operate.proto (1 service, 7 RPC) |
| 状态 | 草案, 待评审 |
| 关联 crate | `crates/operate-service/` (operate_db, 384 LOC) |
| ARC | ARC-054（运营活动 + 复用 ARC-021 插件体系） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计识别出 operate-service 为代码先行扩展域，crate 已实装 384 LOC、1 service、7 RPC，但缺正式 REQ/BAS/DTL 三层文档）。本文档为此扩展域补全的需求层 | 全部 |

> 本文档以代码先行（code-first）方式补全需求定义：对 `crates/operate-service/` 既已实装的 1 个 gRPC service、7 个 RPC 进行反向需求抽象。

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 运营活动域与 holiday_*/activity-service 既有的职责边界 |
| 评审（DBA） | | | operate_db 物理划分 |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 背景与范围
2. 术语约定
3. 业务需求
4. 功能需求
5. 非功能需求
6. 架构设计方针 ARC-054
7. 验收标准
8. 风险与未决事项
9. 关联文档

---

# 1. 背景与范围

## 1.1 背景

`crates/operate-service/` 是 RGS 7 域扩展下独立拆分的运营活动微服务。其设计**承接** battle-service HolidayActivityService 与 activity-service 既有的活动相关能力，但承担**运营活动专用**功能：开服活动 / 限时礼包 / 节日活动总入口 / 活动日历 / 活动进度推送等。

本域覆盖 1 个 gRPC service（OperateService）、7 个 RPC（含 1 HealthCheck）。其中 5 RPC 为真实业务逻辑（活动日历 / 进度推送 / 总入口），2 RPC 为 Unimplemented stub。

## 1.2 范围边界

**In-Scope**：

- 活动日历（按时间聚合所有活动）
- 活动总入口（聚合 holiday_* / 开服活动 / 限时礼包）
- 活动进度推送（玩家订阅的活动进度变化）
- 活动跨域协调（如跨域任务联动）

**Out-of-Scope**：

- 具体活动战斗逻辑（属 battle HolidayActivityService 既有）
- 具体活动配置（属 ARC-021 插件 + activity-service 既有）
- 卡牌 / 道具具体规则（属策划层）

## 1.3 与既有基础设施的关系

| 既有机制 | 本域使用方式 |
|---|---|
| ARC-021 插件热插拔 | 活动配置**严格**作为插件承载 |
| ARC-008 5 独立 DB → 7 域扩展 | 落位独立 `operate_db`（仅活动日历/订阅） |
| RGS-REQ-013 FR-GOV-001 | 活动奖励经 EC 单点 |
| battle HolidayActivityService | 具体活动战斗**严格**复用既有 |
| activity-service 既有 | 活动规则**严格**复用既有 |

---

# 2. 术语约定

| 术语 | 英文 | 含义 |
|---|---|
| 活动日历 | ActivityCalendar | 按时间聚合所有活动（holiday_* / 开服 / 限时 / 跨域） |
| 活动总入口 | ActivityHub | 玩家进入活动的统一入口 |
| 活动订阅 | ActivitySubscription | 玩家订阅的活动进度推送 |
| 活动进度 | ActivityProgress | 玩家在某活动的完成度 |

---

# 3. 业务需求 (BR)

## BR-OPR-001 活动日历

- 按时间聚合所有活动（holiday_* / 开服 / 限时 / 跨域）
- 玩家可订阅日历通知
- 活动开始 / 结束自动通知

**验收**：日历可查询 + 通知可订阅 + 通知推送。

## BR-OPR-002 活动总入口

- 玩家进入"运营活动"面板
- 面板展示当前可参与的活动列表（含跨域活动）
- 玩家点击进入具体活动（委托给 battle / activity-service 既有）

**验收**：总入口可访问 + 委托路径完整。

## BR-OPR-003 活动进度推送

- 玩家订阅某活动的进度变化
- 服务端主动推送（per ActivityConfig.notify_enabled）

**验收**：订阅可 CRUD + 推送可达。

## BR-OPR-004 活动跨域协调

- 活动可能跨多个域（战斗 + 任务 + 收藏）
- operate-service 仅承担"跨域协调元数据"，具体战斗/任务逻辑由各自域承担

**验收**：跨域活动元数据可查询。

---

# 4. 功能需求 (FR)

## 4.1 活动日历

| ID | 需求 |
|---|---|
| FR-OPR-001 | `GetCalendar` **必须**支持按时间范围 + 类型过滤 |
| FR-OPR-002 | 活动开始 / 结束**应当**自动推送通知（per Subscription） |

## 4.2 活动总入口

| ID | 需求 |
|---|---|
| FR-OPR-010 | `GetActivityHub` **必须**返回当前可参与的活动列表 |
| FR-OPR-011 | `EnterActivity` **必须**委托给对应域（battle / activity-service） |

## 4.3 活动订阅

| ID | 需求 |
|---|---|
| FR-OPR-020 | `SubscribeActivity` **必须**支持 CRUD |
| FR-OPR-021 | 推送**应当**走 RGS-REQ-008 埋点 + 推送通道既有 |

## 4.4 跨域协调

| ID | 需求 |
|---|---|
| FR-OPR-030 | `GetCrossDomainActivity` **必须**返回跨域活动元数据 |
| FR-OPR-031 | 跨域活动**不得**在本域内实现战斗/任务逻辑 |

---

# 5. 非功能需求 (NFR)

| ID | 类别 | 内容 | 目标值 / 判定基准 |
|---|---|---|---|
| NFR-OPR-001 | 性能 | 日历查询 P99 延迟 | < 100ms（per NFR-OP-005 既有） |
| NFR-OPR-002 | 一致性 | 活动奖励**必须**经 EC 单点 | per FR-GOV-001 |
| NFR-OPR-003 | 可观测 | 活动进入 / 完成**必须**埋点 | per RGS-REQ-008 既有 |

---

# 6. 架构设计方针 ARC-054

## 6.1 ARC-054：运营活动 + 复用既有活动域原则

| 项目 | 内容 |
|---|---|
| **决定** | operate-service 仅承担"日历 / 总入口 / 订阅 / 跨域协调"4 类元数据/聚合功能；具体活动**严格**复用 battle HolidayActivityService + activity-service 既有 |
| **理由** | 活动相关逻辑分散在 battle / activity / lbx / 任务等多个域，operate 仅承担"聚合 + 通知 + 跨域元数据"的横向协调职责，避免与各域重复 |
| **承载机制** | 活动日历 / 订阅规则作为 ARC-021 既有插件通道承载 |
| **否决方案** | "在 operate 内自建具体活动战斗逻辑 / 自建活动规则"——直接违反既有职责，已否决 |
| **适用范围** | 本域所有运营活动聚合 / 协调功能 |

---

# 7. 验收标准

| ID | 验收标准 |
|---|---|
| AC-OPR-001 | 日历可查询 + 通知推送可达 |
| AC-OPR-002 | 总入口可访问 + 委托路径完整 |
| AC-OPR-003 | 订阅可 CRUD + 推送走既有通道 |
| AC-OPR-004 | 跨域活动元数据可查询，**不**在本域实现具体战斗/任务 |

---

# 8. 风险与未决事项

| ID | 内容 | 处理阶段 |
|---|---|---|
| RSK-OPR-001 | 跨域活动的元数据一致性须 DTL-053 详细设计阶段重点验证 | DTL-053 §3 |

---

# 9. 关联文档

| 文档编号 | 文档名 | 与本文档的关系 |
|---|---|---|
| RGS-REQ-001 | 需求定义书 | 本文档展开其 §4 / §5 |
| RGS-REQ-013 | 体系治理与横切关注点 | 活动奖励经 EC（FR-GOV-001） |
| RGS-REQ-009 | 插件热插拔 | 活动配置作为插件承载（ARC-021） |
| RGS-REQ-016 | 大厅、社交通信与运营活动 | 活动入口从大厅进入 |
| RGS-REQ-040 | 战斗域 | holiday_* 活动由 battle HolidayActivityService 提供 |
| activity-service 既有 | 活动规则 | 本域严格复用 |
| RGS-BAS-046 | 运营活动域 基本设计书（本文档的下游） | 本文档需求展开 |
| RGS-DTL-053 | 运营活动域 详细设计书（本文档的下游） | 本文档物理 / 实现级设计 |

> **文档编号说明**：本文档为 RGS-REQ-046，配套基本设计书 RGS-BAS-046，详细设计书 RGS-DTL-053。
