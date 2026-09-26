# 需求定义书（要件定義書 / Requirements Definition Document）

**战斗域（Battle Domain）需求定义**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-040 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-REQ-001 v1.5 §4 业务需求 + §5 功能需求；RGS-REQ-013 v1.4 横切关注点 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联下游 | RGS-BAS-040 / RGS-DTL-047 / battle/v1/battle.proto (12 service, 250 RPC 含 12 HealthCheck) |
| 状态 | 草案, 待评审 |
| 关联 crate | `crates/battle-service/` (battle_db) |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 issue thread 9/4 MD §2 审计识别出 battle-service 为代码先行扩展域，crate 已实装 2,875 LOC、12 service、241 RPC，但缺正式 REQ/BAS/DTL 三层文档）。本文档为此扩展域补全的需求层 | 全部 |

> 本文档以代码先行（code-first）方式补全需求定义：对 `crates/battle-service/` 既已实装的 12 个 gRPC service、241 个 RPC 进行反向需求抽象，仅描述既有能力之"为何存在 / 服务于何种业务目的"，不引入新功能。

---

## 目录

1. [背景与范围](#1-背景与范围)
2. [术语约定](#2-术语约定)
3. [业务需求](#3-业务需求)
4. [功能需求：战斗引擎](#4-功能需求战斗引擎)
5. [功能需求：PVP 变体群](#5-功能需求pvp-变体群)
6. [功能需求：PVE 与副本](#6-功能需求pve-与副本)
7. [功能需求：房间与矿战](#7-功能需求房间与矿战)
8. [功能需求：无尽塔 / 护送 / 公会战](#8-功能需求无尽塔-护送-公会战)
9. [功能需求：跨服 PVP](#9-功能需求跨服-pvp)
10. [功能需求：远征 / 节日活动](#10-功能需求远征-节日活动)
11. [非功能需求](#11-非功能需求)
12. [架构设计方针 ARC-046](#12-架构设计方针-arc-046)
13. [验收标准](#13-验收标准)
14. [风险与未决事项](#14-风险与未决事项)
15. [关联文档](#15-关联文档)

---

# 1. 背景与范围

## 1.1 背景

`crates/battle-service/` 是 RGS（RustGameServer）7 域扩展（per 路线图 §3 W5 + 9/4 MD §2 + 8/21 JST 5 域 → 9/1 JST batch 域 → 9/5 JST battle 域）下独立拆分的战斗微服务。其设计**借鉴**《[游戏A]》（AFK Arena 类）放置卡牌战斗的多变体形态，但**反对**[游戏A]的"一活动一模块"反例（per 9/4 MD §4）。

本域覆盖 12 个 gRPC service、241 个 RPC（含 12 个 HealthCheck）；其中 30 RPC 为真实业务逻辑（核心战斗生命周期 / 数据驱动框架 / 业务校验），220 RPC 为 Unimplemented stub（Phase 3 业务实装）。

| Service | proto | RPC 数 | 业务实装策略 |
|---|---|---|---|
| BattleEngineService | proto_200 | 31 | 真实逻辑：核心战斗状态机 |
| PvPService | proto_202 | 30 | 真实逻辑：6 个 PVP 变体复用 1 套数据驱动框架 |
| BossService | proto_205 | 15 | 真实逻辑：PVE BOSS |
| RoomService | proto_206 | 46 | 真实逻辑：房间战 + 矿战 |
| InstanceService | proto_207 | 6 | 真实逻辑：副本 |
| EndlessTowerService | proto_239 | 12 | stub |
| EscortService | proto_240 | 17 | stub |
| HolyEquipService | proto_241 | 23 | stub |
| GuildWarService | proto_242 | 17 | stub |
| CrossServerService | proto_243 | 19 | stub，复用 PvPService 数据驱动 |
| ExpeditionService | proto_244 | 15 | stub |
| HolidayActivityService | proto_248 | 18 | stub：1 服务覆盖 9 个 holiday_* 变体（数据驱动） |

## 1.2 范围边界

**In-Scope**：

- 战斗状态机（回合制 + 即时制）
- PVP / PVE / 房间战 / 矿战 / 副本 / 跨服 PVP / 公会战 / 远征 / 节日活动
- 数据驱动 PVP 框架（1 个 PvPService + PvPConfig 覆盖 6 个变体）
- 数据驱动节日活动框架（1 个 HolidayActivityService + ActivityConfig 覆盖 9 个变体）
- 战斗生命周期：开始 / 暂停 / 恢复 / 结束 / 重连

**Out-of-Scope**：

- 卡牌数值平衡、抽卡概率（属策划层，per X-002）
- 客户端 UI / 美术 / 音频（per X-001）
- 战斗客户端预测算法（属 ARC-002 既有同步层职责）
- 战斗录像的回放与分享（属 replay-extra-service 范畴）
- 战斗结算后的段位 / 积分变化（属 RGS-REQ-029 匹配系统与 RGS-REQ-017 排行榜既定职责）

## 1.3 与既有基础设施的关系（避免重复建设）

| 既有机制 | 本域使用方式 |
|---|---|
| ARC-001 Actor 粒度 = 场景单位 | 一场战斗 = 一个 BattleActor，复用 ARC-001 状态机 |
| ARC-002 状态同步 + 客户端预测 | 战斗内同步走既有差分快照；本域不另建同步算法 |
| ARC-008 5 独立 DB → 7 域扩展 | 本域落位独立 `battle_db`（不与 player / social / economy 混库） |
| ARC-011 Saga 单调解者 | 战斗结算产生道具 / 货币发放**必须**经 EC 既有确定请求路径（per RGS-REQ-013 FR-GOV-001） |
| ARC-021 插件热插拔 | 节日活动 / 远征等高度可变的玩法作为插件承载，**不**新建独立 service |
| RGS-BAS-013 §2.2 大厅作为特殊场景 | 战斗场景从大厅转入时**必须**走 FR-RT-008 既有场景间转移机制 |
| RGS-BAS-026 匹配系统 | PVP 玩家入队**必须**经 MT（match）既有匹配流程，**不**在本域另建匹配 |

---

# 2. 术语约定

| 术语 | 英文 | 含义 |
|---|---|---|
| 战斗 | Battle | 一场完整对局，由 BattleActor 管理状态 |
| 战斗阶段 | BattlePhase | 枚举：Init / Preparation / Combat / Settlement / End |
| 战斗模式 | BattleMode | 枚举：PVE / PVP / Room / Mine / Guild / Expedition / Holiday / Boss / Instance / EndlessTower / Escort / CrossServer |
| 战斗结果 | BattleOutcome | 枚举：Win / Lose / Draw / Aborted / Timeout |
| PVP 模式 | PvpMode | 6 个变体：Ranked / Casual / CrossServer / Arena / Tournament / Custom |
| 房间类型 | RoomType | 枚举：Normal / Boss / Mine / Escort |
| 房间 buff | RoomBuff | 房间内所有玩家共享的属性加成 |
| 矿战资源 | MineResource | 矿战可产出的资源类型（金币 / 钻石 / 稀有材料） |
| 护送品质 | EscortQuality | 枚举：Common / Rare / Epic / Legendary |
| 节日活动 | HolidayActivity | 由 ActivityConfig 数据驱动的 holiday_* 变体（bid:93031/...） |
| 战斗数据驱动配置 | PvPConfig / ActivityConfig | 6 个 PVP 变体与 9 个 holiday_* 变体共用配置 |
| 战斗同伴槽位 | CompanionSlot | 玩家携带的出战伙伴的槽位（含 CompanionSource 来源） |

---

# 3. 业务需求

| ID | 需求 | 优先级 |
|---|---|---|
| BR-BAT-001 | 玩家须能在多种战斗形态（PVE / PVP / 房间 / 矿战 / 公会战 / 跨服 PVP 等）中获得一致的核心战斗体验（起手 → 回合 → 结算） | 高 |
| BR-BAT-002 | 策划须能在不部署代码的前提下，通过配置驱动快速新增 / 调整 PVP 变体与节日活动（per 9/4 MD §4 反"一活动一模块"反例） | 高 |
| BR-BAT-003 | 战斗结算产生的道具 / 货币发放**必须**与既有经济系统保持一致（per RGS-REQ-013 FR-GOV-001 永久事实强制路由） | 高 |
| BR-BAT-004 | 玩家在战斗中断线 / 重连 / 跨设备时，战斗状态**必须**可恢复且不产生"两次奖励"或"奖励丢失" | 高 |
| BR-BAT-005 | 战斗场景的进入 / 退出**必须**经既有 FR-RT-008 场景间转移机制，不允许旁路 | 高 |

---

# 4. 功能需求：战斗引擎（BattleEngineService, 31 RPC）

| ID | 需求 |
|---|---|
| FR-BAT-001 | `StartBattle` **必须**初始化 BattleActor、加载参战双方数据（玩家 / NPC / Boss），转 `Init → Preparation` 阶段 |
| FR-BAT-002 | `SubmitAction` **必须**接收客户端提交的动作（出牌 / 攻击 / 释放技能），经 ARC-002 服务器权威校验后写回 BattleActor 状态 |
| FR-BAT-003 | `Tick` **必须**驱动战斗状态机按回合推进，**不得**依赖客户端 tick 频率（服务器权威） |
| FR-BAT-004 | `PauseBattle` / `ResumeBattle` **必须**支持玩家主动暂停（限断线 / 切场景场景，超时自动恢复） |
| FR-BAT-005 | `SettleBattle` **必须**在 `Combat → Settlement` 阶段计算奖励，**必须**经 EC 既有确定请求路径发放（per FR-GOV-001），**不得**在 battle_db 直接写 economy_db |
| FR-BAT-006 | `QueryBattleState` **必须**返回 BattleActor 当前状态（参战者 / 阶段 / 双方 HP / buff / 累计回合），供客户端重连后同步 |
| FR-BAT-007 | `AbortBattle` **必须**支持 GM / 玩家 / 系统中止，结算按既定的"中止"分支处理（per BattleOutcome::Aborted） |

---

# 5. 功能需求：PVP 变体群（PvPService, 30 RPC）

**反例原则**：[游戏A] 6 个 PVP 变体不重复 6 套代码（per 9/4 MD §4），1 个 PvPService + PvPConfig 覆盖全部 6 个变体。

| ID | 需求 |
|---|---|
| FR-BAT-010 | `EnqueuePvp` **必须**依据 `PvpConfig.mode` 字段（ranked / casual / cross-server / arena / tournament / custom）路由至对应匹配池，**必须**复用 RGS-BAS-026 既有匹配流程，**不得**在本域另建匹配 |
| FR-BAT-011 | `GetPvpConfig` **必须**返回当前生效的 PvPConfig（含变体名 / 队伍规模 / 段位限制 / 奖励规则） |
| FR-BAT-012 | `SubmitPvpResult` **必须**复用 BattleEngineService 的结算接口，**不得**为 PVP 变体重写结算逻辑 |
| FR-BAT-013 | `ListPvpVariants` **必须**返回当前所有启用 PVP 变体的元数据（供大厅入口展示，per RGS-REQ-016 FR-LBY-004） |

---

# 6. 功能需求：PVE 与副本（BossService / InstanceService, 21 RPC）

| ID | 需求 |
|---|---|
| FR-BAT-020 | BossService `StartBossBattle` **必须**加载 Boss 配置（Boss ID / 难度 / 队伍等级限制），**必须**校验玩家是否持有相应 Boss 入口券（per RGS-REQ-013 FR-EC-003 既有路径） |
| FR-BAT-021 | BossService `DamageBoss` **必须**累计伤害并广播给所有参战者（复用 ARC-002 差分快照机制） |
| FR-BAT-022 | InstanceService `CreateInstance` **必须**为每个队伍分配独立副本实例（key = `instance:{instance_id}`），实例**必须**有 TTL（超时自动销毁） |
| FR-BAT-023 | InstanceService `JoinInstance` / `LeaveInstance` **必须**校验参战者所属队伍一致性，**不得**允许跨队伍加入 |

---

# 7. 功能需求：房间与矿战（RoomService, 46 RPC）

| ID | 需求 |
|---|---|
| FR-BAT-030 | RoomService `CreateRoom` **必须**依据 `RoomType` 字段（Normal / Boss / Mine / Escort）初始化对应房间，**必须**校验创建者权限 |
| FR-BAT-031 | RoomService `JoinRoom` / `LeaveRoom` **必须**校验房间容量上限，房间满员**必须**拒绝（per NFR-SE-001 服务器权威） |
| FR-BAT-032 | RoomService `ApplyRoomBuff` **必须**使 buff 对房间内所有玩家生效，**不得**为每个玩家单独应用（房间 buff 的语义即共享） |
| FR-BAT-033 | MineService `ClaimMineResource` **必须**在矿战结束（或周期结算）后发放 `MineResource`，发放路径**必须**经 EC 既有确定请求路径 |

---

# 8. 功能需求：无尽塔 / 护送 / 公会战（EndlessTowerService / EscortService / GuildWarService, 46 RPC）

| ID | 需求 |
|---|---|
| FR-BAT-040 | EndlessTowerService `StartTower` **必须**加载无尽塔配置（层数 / 每层 NPC / 通关奖励递增规则） |
| FR-BAT-041 | EscortService `StartEscort` **必须**校验护送品质（Common / Rare / Epic / Legendary）与玩家持有任务券的匹配 |
| FR-BAT-042 | GuildWarService `DeclareWar` **必须**校验公会间关系与宣战冷却期，**必须**经 GD 既有公会服务（per RGS-REQ-016 FR-LBY 既有）确认公会存在 |
| FR-BAT-043 | GuildWarService `SubmitWarResult` **必须**写入公会战历史，奖励发放**必须**经 EC 既有确定请求路径 |

---

# 9. 功能需求：跨服 PVP（CrossServerService, 19 RPC）

| ID | 需求 |
|---|---|
| FR-BAT-050 | CrossServerService **必须**复用 PvPService 的数据驱动模式（per 9/4 MD §4），**不**新建独立 service（与既有 PvPService 共享 PvPConfig.mode=cross_server 分支） |
| FR-BAT-051 | 跨服匹配的边界**必须**符合 RGS-BAS-026 §3.4 既定的"跨分片匹配池边界"判定，**不得**隐式扩大匹配范围 |
| FR-BAT-052 | 跨服 PVP 的延迟**应当**满足 NFR-OP-005 既有目标值（per 路线图 §4.3），实际指标由 DTL-047 §5 算法详细设计给出 |

---

# 10. 功能需求：远征 / 节日活动（ExpeditionService / HolidayActivityService, 33 RPC）

| ID | 需求 |
|---|---|
| FR-BAT-060 | ExpeditionService `StartExpedition` **必须**复用 BattleEngineService 核心战斗逻辑，**不**为远征单独实现战斗状态机 |
| FR-BAT-061 | HolidayActivityService **必须**覆盖 9 个 holiday_* 变体（bid:93031/...），由 ActivityConfig 数据驱动（per 9/4 MD §4 反例原则），**不**为每个变体新建独立 service |
| FR-BAT-062 | 节日活动的配置变更**必须**经 RGS-REQ-009 插件体系 ARC-021 既有通道上线，**不得**要求重新部署 battle-service |
| FR-BAT-063 | 节日活动的奖励发放**必须**判定为"经济类插件"（影响货币 / 道具数值），由 EC 单点执行（per RGS-REQ-013 FR-GOV-030） |

---

# 11. 非功能需求

| ID | 类别 | 内容 | 目标值 / 判定基准 |
|---|---|---|---|
| NFR-BAT-001 | 性能 | 单场战斗状态切换延迟 | P99 < 50ms（per ARC-002 既有同步机制目标值，本域不新增独立目标） |
| NFR-BAT-002 | 一致性 | 战斗结算发放**必须**与既有道具 / 货币操作同等一致性保证（per AC-009 总量差分为 0） | 故障注入覆盖（详见 §13 AC-BAT-004） |
| NFR-BAT-003 | 安全 | `SubmitAction` **必须**服务器权威校验（per NFR-SE-001），客户端**不得**绕过服务器直接修改 BattleActor |
| NFR-BAT-004 | 可恢复 | 玩家断线后**必须**能在 N 分钟内重连并恢复 BattleActor 状态，**不得**产生重复奖励 |
| NFR-BAT-005 | 容量 | 单 BattleActor **应当**支持最多 N 人同时参战（N 由具体战斗模式决定，PVE 通常 1-5 人，公会战最多 50 人，配置可调） | per RGS-REQ-025 ARC-040 既有容量分级 |
| NFR-BAT-006 | 隐私 | 跨服 PVP **不得**泄露对方玩家精确位置 / 个人信息（per NFR-SE-005 既有） | 字段级设计见 DTL-047 |

---

# 12. 架构设计方针 ARC-046

本节为 RGS-REQ-001 第 10 章架构方针体系的延续，编号紧接 ARC-045。**决定事项**，变更须经 ADR 与审批。

## 12.1 ARC-046：战斗域数据驱动 + 反"一活动一模块"原则

| 项目 | 内容 |
|---|---|
| **决定** | 6 个 PVP 变体不重复 6 套代码，由 1 个 `PvPService` + `PvPConfig`（ranked / casual / cross-server / arena / tournament / custom）覆盖；9 个 holiday_* 活动不重复 9 套代码，由 1 个 `HolidayActivityService` + `ActivityConfig`（bid:93031/...）覆盖；其他类似多变体形态（无尽塔 / 远征 / 节日活动）同理 |
| **理由** | per 9/4 MD §4 + 路线图 §0.3 [游戏A]"一活动一模块"反例明确识别：每新建一个变体即新建一套代码，会导致战斗域在 1 年内产生 100+ service 的不可维护状态。**这是本域最重要的反例原则** |
| **数据驱动配置的承载机制** | ①PVP 变体 / 节日活动等高度可配置项**应当**作为 RGS-REQ-009 插件体系下的"特性开关 + 配置数据"承载（per ARC-021）；②**不**使用沙箱脚本（战斗逻辑属强实时性，沙箱性能不可接受）；③**不**新建独立 service |
| **新增变体的标准流程** | ①策划 / 数值提交 PvPConfig / ActivityConfig 数据 ②经 ARC-021 既有插件注册通道上线 ③无须代码改动，无须重启服务 |
| **否决方案** | "为每个变体新建独立 service / crate"——直接违反 ARC-018（功能挂载评审） + 9/4 MD §4 反例原则，已否决 |
| **适用范围** | 本域所有多变体形态的 service（PvP / HolidayActivity / EndlessTower / Expedition / Escort 等） |

---

# 13. 验收标准

| ID | 验收标准 |
|---|---|
| AC-BAT-001 | 玩家从大厅进入战斗场景、提交动作、战斗结算、回到大厅的完整路径可稳定运行（per AC-LBY-001 既有验收方法扩展） |
| AC-BAT-002 | 6 个 PVP 变体**均**经同一 `PvPService` 接口实现，**无**独立的变体 service / crate（per ARC-046 反例原则） |
| AC-BAT-003 | 9 个 holiday_* 活动**均**经同一 `HolidayActivityService` + ActivityConfig 实现，**无**独立的活动 service / crate |
| AC-BAT-004 | 战斗结算的故障注入试验（战斗结算成功但 EC 发放失败 / 反之）均由既有 Saga 补偿正确处理，无终态不一致（per AC-009 同款判定） |
| AC-BAT-005 | 玩家断线后 5 分钟内重连，BattleActor 状态可恢复，**不**产生重复奖励（per NFR-BAT-004） |
| AC-BAT-006 | 跨服 PVP 的玩家数据**不**经本域中转泄露至其他域（per NFR-SE-005） |

---

# 14. 风险与未决事项

| ID | 内容 | 处理阶段 |
|---|---|---|
| TBD-BAT-001 | 跨服 PVP 的具体延迟目标值（NFR-BAT-002 当前引用既有值，需在 DTL-047 §5 算法详细设计中给出实测建议） | DTL-047 §5 |
| TBD-BAT-002 | 断线重连的最大允许时长（NFR-BAT-004 当前为"分钟"量级，需策划与运营确认） | PH-2〜PH-6 |
| TBD-BAT-003 | 公会战最大参战人数（NFR-BAT-005 当前为"配置可调"，需策划确认上限） | PH-2 |
| RSK-BAT-001 | 若 Phase 3 实装阶段绕过 ARC-046 数据驱动原则、为每个变体新建 service，会重演[游戏A]"一活动一模块"反例。缓解：代码评审须核对多变体形态的实现是否复用同一 service + Config | 持续跟踪 |
| RSK-BAT-002 | 战斗场景退出 / 中止的幂等性须 DTL-047 详细设计阶段重点验证（防止断线重连 + 战斗结算 + 中止三路并发产生双发奖励） | DTL-047 §5 |

---

# 15. 关联文档

| 文档编号 | 文档名 | 与本文档的关系 |
|---|---|---|
| RGS-REQ-001 | 需求定义书 | 本文档展开其 BR-005（战斗）/ FR-BAT-001〜063 等 |
| RGS-REQ-013 | 体系治理与横切关注点 | 战斗结算永久事实必须经 EC（FR-GOV-001） |
| RGS-REQ-009 | 插件热插拔与生命周期管理 | 节日活动作为插件承载（ARC-021） |
| RGS-REQ-016 | 大厅、社交通信与运营活动 | 战斗场景从大厅转入走既有机制（FR-LBY-001） |
| RGS-REQ-017 | 排行榜、任务成就与玩家治理 | 战斗产生的段位 / 积分走既有排行榜 |
| RGS-REQ-029 | 匹配系统 | PVP 玩家入队经既有 MT 匹配流程 |
| RGS-BAS-001 §4.3 | 同步・AOI | 战斗内同步复用差分快照 |
| RGS-BAS-005 | 插件热插拔 | 节日活动作为插件上线 |
| RGS-BAS-013 | 大厅、社交通信与运营活动 | 战斗场景从大厅转入时序 |
| RGS-BAS-026 | 匹配系统 | 跨服匹配池边界判定 |
| RGS-DTL-001 §3.2 | OCC + 幂等 | 战斗结算事务边界 |
| RGS-BAS-040 | 战斗域 基本设计书（本文档的下游） | 本文档需求展开 |
| RGS-DTL-047 | 战斗域 详细设计书（本文档的下游） | 本文档物理 / 实现级设计 |

---

> **文档编号说明**：本文档为 RGS-REQ-040，与其配套基本设计书编号为 RGS-BAS-040（按既有跨类别编号独立规则——类别内序号连续，跨类别不要求对齐，但战斗域作为独立范畴，约定 REQ/BAS/DTL 三者编号保持连续：B40/B40/D47，DTL-047 取自 040+7 = 047 是因为 DTL-040 已被 Admin 域占用）。
