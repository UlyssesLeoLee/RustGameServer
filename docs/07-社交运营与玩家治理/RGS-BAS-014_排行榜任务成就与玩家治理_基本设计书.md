# 基本设计书（基本設計書 / Basic Design Document）

**排行榜、任务成就与玩家治理 Leaderboard, Quest/Achievement & Player Governance**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-014 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-017 需求定义书（ARC-031） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-08-16 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-017§11 ARC-031展开为：派生排行视图的组件设计与更新时序、任务/成就的配置化触发引擎设计、邮件系统的数据模型、举报/黑名单的字段级设计、赛季重置的时序图 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①新增`RankingDimensionConfig`维度可配置表（FR-GSM-001）②补充`RankingViewUpdater`消费失败/死信分支与视图重建路径（FR-GSM-003、NFR-GSM-002）③新增举报者信誉度字段与降权机制设计（FR-GSM-033、RSK-GSM-002）④补充`MailMessage`/`PlayerReport`/`PlayerBlocklist`索引与唯一性约束（复用RGS-BAS-007标准） | FR-GSM-001、FR-GSM-003、FR-GSM-033、NFR-GSM-002、RSK-GSM-002 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 派生视图更新时序是否与既有ARC-009事件基础设施的幂等保证一致 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [排行榜：派生视图组件设计](#2-排行榜派生视图组件设计)
3. [任务与成就：配置化触发引擎设计](#3-任务与成就配置化触发引擎设计)
4. [邮件系统：数据模型](#4-邮件系统数据模型)
5. [举报与黑名单：字段级设计](#5-举报与黑名单字段级设计)
6. [赛季与段位：重置时序](#6-赛季与段位重置时序)
7. [标准化检查清单](#7-标准化检查清单)
8. [追溯性](#8-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-017定义的ARC-031（派生排行视图的一致性边界）及其配套的四个功能模块，遵循ARC-018挂载原则——本文档定义的全部组件均**依附**既有限界上下文（EC／GD／MT／AD）运行，**不新建**独立限界上下文、独立数据库或独立部署单元。

命名约定：本文档中的字段级设计以"逻辑字段"表述，物理DDL遵循RGS-BAS-007既定的数据库设计标准（命名规范、索引/分区标准）执行，不在本文档重复定义。

---

# 2. 排行榜：派生视图组件设计

## 2.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `RankingSource` | EC／GD（权威数据所在上下文） | 权威分数变更时发布`RankingScoreChanged`事件（复用ARC-010事件基础设施），**不直接**写入排行视图 |
| `RankingViewUpdater` | 缓存基础设施（ARC-012既有缓存边界的具体化） | 订阅`RankingScoreChanged`，对增量变更做局部重排序写入派生视图 |
| `RankingQueryService` | 依附GD/RT既有API网关路由 | 对外提供分页查询与"附近排名"查询（FR-GSM-004），**只读**派生视图，不触达权威表 |
| `RankingAuthoritativeFallback` | EC／GD（权威数据所在上下文） | 仅在赛季结算（FR-GSM-006）时点被调用，从权威表直接计算最终名次 |

## 2.2 派生视图的数据结构（TBD-GSM-001待评审前的默认方案）

复用ARC-012既定缓存基础设施的有序集合能力（如有序集合类型的键值存储）作为默认方案，键为`ranking:{维度}:{赛季ID}`，成员为玩家ID，分值为对应维度分数。选型最终确定见TBD-GSM-001（ISS-046）。

### 2.2.1 排行维度可配置扩展（FR-GSM-001字段级设计）

新增维度不得硬编码，须通过配置表`RankingDimensionConfig`声明（复用ARC-016数值表热更新分发机制，不修改`RankingViewUpdater`代码路径）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `dimension_id` | string | 唯一标识，如`player_level`／`season_score`／`guild_prestige`（PH-6+） |
| `source_context` | enum(`EC`／`GD`／`MT`) | 权威分数来源的限界上下文，决定`RankingSource`订阅哪个事件流 |
| `source_event` | string | 触发分数变更的事件类型（如`PlayerLevelUp`、`SeasonScoreSettled`），须为ARC-010既定事件流中已存在的事件 |
| `score_field_path` | string | 从事件载荷中提取分数值的字段路径 |
| `season_scoped` | bool | 是否随赛季重置（`season_score`为`true`，`player_level`为`false`） |
| `enabled` | bool | 是否对外暴露查询，用于灰度上线新维度而不影响既有维度 |

新增一种排行维度仅需新增一行`RankingDimensionConfig`配置并声明其`source_event`订阅，`RankingSource`/`RankingViewUpdater`按配置驱动，不需为新维度编写专属分支代码，满足FR-GSM-001"应当可配置扩展"要求。

## 2.3 更新时序（增量式，FR-GSM-003）

```
权威数据变更（如玩家升级/赛季积分结算）
  → RankingSource发布RankingScoreChanged{player_id, dimension, new_score}（ARC-010事件基础设施，至少一次投递）
  → RankingViewUpdater消费事件（幂等：以player_id+dimension为幂等键，重复投递不产生重复排序副作用）
  → 对派生视图做单条成员分值更新（局部操作，不全量重算）
  → 更新完成后记录本次更新时间戳，供NFR-GSM-002滞后监控读取
```

> 消费失败重试与死信处理复用ARC-009既定的事件消费者标准模式，不新增专属基础设施。

### 2.3.1 异常分支

```
RankingViewUpdater消费RankingScoreChanged失败（如缓存基础设施瞬时不可用）
  → 按ARC-009标准重试策略重试N次
  → 仍失败 → 投递至既有死信队列（复用ARC-009死信处理），记录告警（RGS-BAS-003§6）
  → 死信事件不阻塞后续事件消费（幂等键保证乱序到达也不产生错误覆盖：仅当new_score对应的事件时间戳晚于视图当前记录的最后更新时间时才写入，防止死信重放导致的乱序覆盖新数据）
  → 运维/告警响应后，可触发"视图重建"（从权威表按`RankingDimensionConfig.source_event`对应的权威数据全量重算一次目标维度的派生视图，作为NFR-GSM-002滞后超限或死信事件堆积后的兜底恢复手段，重建期间该维度查询**应当**降级提示"数据更新中"而非报错）
```

## 2.4 一致性边界的落地规则（ARC-031核心约束）

| 场景 | 是否可用派生视图 | 依据 |
|---|---|---|
| 常态排行榜展示（榜单/附近排名查询） | **可以**，允许NFR-GSM-002定义的滞后 | FR-GSM-002 |
| 赛季结算的名次判定（决定奖励发放） | **不可以**，必须回落权威数据源 | FR-GSM-006、FR-GSM-043 |
| GM后台查询某玩家当前分数 | **不可以**，直接查权威表（低频操作，无性能顾虑） | ARC-031决定 |

## 2.5 滞后监控

`RankingViewUpdater`每次更新记录`last_update_lag_ms`（事件产生时间与视图更新完成时间之差），接入RGS-BAS-004既有黄金指标体系，超过NFR-GSM-002阈值告警（复用RGS-BAS-003§6告警推送通道）。**须与派生视图功能同批上线，不得后补**（RSK-031缓解措施）。

---

# 3. 任务与成就：配置化触发引擎设计

## 3.1 配置表结构（逻辑字段，复用ARC-016热更新分发）

`QuestDefinition`（任务/成就共享同一配置表结构，通过`category`字段区分）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `quest_id` | string | 唯一标识 |
| `category` | enum(`quest`／`achievement`) | 任务或成就 |
| `trigger_condition` | 声明式表达式（如`event=ItemGranted AND item_type=monster_kill AND count>=100`） | 触发条件，**不得**要求编写专属订阅代码（FR-GSM-010） |
| `reset_policy` | enum(`never`／`season`／`period_days:N`) | 是否随赛季/周期重置（FR-GSM-014，成就默认`never`，任务默认`season`） |
| `reward_spec` | 引用既有物品/货币发放规格（复用FR-EC-003确定请求路径的入参结构） | 领奖时通过`RewardGrantService`发放 |

## 3.2 触发引擎组件

| 组件 | 职责 |
|---|---|
| `QuestConditionSubscriber` | 订阅ARC-010既定事件流（`ItemGranted`、对局结算等），按`trigger_condition`表达式匹配，**异步**更新任务进度（FR-GSM-015，不阻塞事件产生方） |
| `QuestProgressStore` | 持久化玩家任务进度，依附既有EC/GD上下文数据库，不新建独立库 |
| `QuestStateMachine` | 状态机：`可领取→已领取→进行中→已完成→已领奖`，非法迁移拒绝（FR-GSM-012，复用RGS-REQ-001第8章状态机纪律） |
| `QuestRewardGranter` | 复用FR-EC-003确定请求路径发放奖励，**不新设旁路**（FR-GSM-013） |

## 3.3 新增触发条件类型的扩展方式（NFR-GSM-004验证点）

新增一种触发条件类型仅需：①在`trigger_condition`表达式语法中新增操作符/字段（若有必要）②新增对应事件的订阅声明（配置项，非代码）。**不得**要求修改已有任务的代码路径，AC-GSM-003对此验证。

---

# 4. 邮件系统：数据模型

## 4.1 逻辑数据模型

`MailMessage`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `mail_id` | uuid | 唯一标识 |
| `recipient_id` | 玩家ID | 收件人 |
| `mail_type` | enum(`system`／`business`) | 系统邮件（运营批量发放，复用FR-AD-002来源）／业务邮件（如交易失败退还），共享同一模型（FR-GSM-021） |
| `subject` / `body` | string | 标题/正文 |
| `attachments` | 引用既有物品/货币发放规格 | 领取时复用FR-EC-003确定请求路径（FR-GSM-022） |
| `read_status` | enum(`unread`／`read`) | 已读/未读 |
| `claim_status` | enum(`unclaimed`／`claimed`) | 附件是否已领取 |
| `expire_at` | timestamp | 保留期截止（默认90天，NFR-GSM-006，可配置） |

索引：`(recipient_id, read_status, expire_at)`复合索引支撑玩家收件箱列表的高频查询（按收件人过滤未读/未过期）；`(expire_at)`单列索引支撑§4.3按月度分区的到期清理批处理扫描。`mail_id`为主键，物理DDL细节遵循RGS-BAS-007命名规范。

## 4.2 批量发送

`MailBatchSender`接受目标条件（全服/玩家列表/满足特定条件的群体），**异步**逐条生成`MailMessage`（FR-GSM-024，复用RGS-BAS-003控制平面既有异步工单处理模式，不阻塞GM操作的即时响应）。

## 4.3 保留期清理

复用RGS-BAS-007§4既定的分区归档标准：`MailMessage`表按`expire_at`月度分区，到期分区归档/清理。清理前T-3天对未领取邮件触发到期提醒（复用既有通知/告警机制，FR-GSM-023）。

---

# 5. 举报与黑名单：字段级设计

## 5.1 举报

`PlayerReport`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `report_id` | uuid | 唯一标识 |
| `reporter_id` / `target_id` | 玩家ID | 举报者/被举报者 |
| `report_type` | enum(`cheating`／`harassment`／`inappropriate_name`／`other`) | FR-GSM-030 |
| `context_ref` | 可选，对局ID/聊天记录ID | 上下文引用 |
| `dedup_key` | `reporter_id`+`target_id`+滚动时间窗口的哈希 | 用于FR-GSM-033去重统计，防止重复举报虚增信号强度 |
| `signal_weight` | decimal，默认1.0 | 该条举报计入信号强度的权重，受`ReporterReputation.weight_multiplier`折算（见下） |
| `created_at` | timestamp | 举报提交时间，参与滚动时间窗口计算与RSK-GSM-002持续追踪 |

索引：`(target_id, created_at)`复合索引支撑GM后台按被举报者聚合查询（RGS-BAS-003§3.4只读查询模式）；`(dedup_key)`唯一索引直接在数据库层面阻止同一滚动窗口内的重复计数写入，不依赖应用层去重逻辑单独兜底（FR-GSM-033）。

审计留痕复用RGS-BAS-003§7审计设计（存储结构与保留期一致），GM后台查询复用RGS-BAS-003§3.4只读查询模式。**处置路径**：举报仅产生信号，`PlayerReport`记录**不直接**触发任何`AdminService`调用；处罚必须由GM人工或RGS-REQ-014智能层（若启用，经ARC-030确定性闸门）显式调用既有`BanAccount`/`MuteChat`（FR-GSM-032）。

### 5.1.1 举报者信誉度（RSK-GSM-002缓解机制字段级设计）

`ReporterReputation`（依附既有GD/AD上下文数据库，不新建独立库）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `reporter_id` | 玩家ID | 主键，一个玩家一条信誉度记录 |
| `substantiated_count` | int | 经GM/闸门判定为"举报属实并触发处罚"的历史次数 |
| `unsubstantiated_count` | int | 经GM判定为"举报不实/恶意"的历史次数 |
| `weight_multiplier` | decimal，范围[0.1, 1.5] | 该举报者未来举报计入信号强度的折算系数，按`substantiated_count`/`unsubstantiated_count`比例周期性重算（详细算法留待详细设计，本表仅定义数据结构与边界） |
| `updated_at` | timestamp | 最近一次信誉度重算时间 |

信誉度更新时机：GM在`AdminService`对举报作出处置决定（处罚或标记不实）时，**异步**触发`ReporterReputation`重算（复用ARC-009事件消费幂等机制，不阻塞GM操作），**不得**影响FR-GSM-032"举报本身不自动触发处罚"的既定原则——信誉度仅调节未来举报的信号权重，不构成对被举报者的处罚依据。

## 5.2 黑名单

`PlayerBlocklist`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `owner_id` | 玩家ID | 黑名单所有者 |
| `blocked_id` | 玩家ID | 被拉黑者 |
| `created_at` | timestamp | 生效时间（即时生效） |

主键/唯一性约束：`(owner_id, blocked_id)`复合唯一索引，防止同一玩家对同一目标重复拉黑产生冗余行；`(owner_id)`单列索引支撑"查询自己的黑名单列表"（NFR-GSM-005唯一允许的查询路径）；**不得**在`blocked_id`上建立可被反查`owner_id`集合的索引/接口，避免NFR-GSM-005"不向第三方暴露谁拉黑了谁"被索引可用性间接绕过。

查询边界（NFR-GSM-005）：仅`owner_id`本人可查询自己的`PlayerBlocklist`，**不对**`blocked_id`（含被拉黑者本人）暴露该记录的存在性。

生效点：既有FR-LBY-011私聊路由在建立会话前须查询`PlayerBlocklist`（若`blocked_id`=发起方且`owner_id`=接收方，拒绝路由），组队邀请路径同理接入。**不影响**同一公开频道的可见性——黑名单不等同于隐身（FR-GSM-035）。

---

# 6. 赛季与段位：重置时序

## 6.1 段位状态机

复用RGS-REQ-001第8章状态机纪律，段位迁移（如晋升/降级）**必须**经由既定规则计算，非法迁移（跳级晋升）拒绝（FR-GSM-041）。

## 6.2 赛季边界原子切换时序（复用ARC-016 tick边界原子切换思想）

```
赛季边界时刻T到达
  → 赛季切换协调者（依附既有调度基础设施，不新建独立组件）触发"赛季结算"流程
  → RankingAuthoritativeFallback对权威数据源计算最终名次（不使用派生视图滞后快照，FR-GSM-006/FR-GSM-043）
  → 按TBD-GSM-002确定的继承规则（清零/按比例保留/软重置区间）计算新赛季初始段位/积分
  → 幂等写入新赛季初始状态（重复触发不产生重复奖励，NFR-GSM-003）
  → 赛季奖励通过既有FR-EC-003确定请求路径发放（对应邮件系统或直接背包发放）
  → 原子提交：新赛季状态与旧赛季结算记录在同一事务边界内落地，不产生"部分玩家已按新赛季结算、部分仍按旧赛季"的中间态（FR-GSM-040）
```

## 6.3 切换时正在进行中的对局

赛季边界T到达前已开始、T之后结束的对局，其结算**归属规则**（按旧赛季结算，或不计入任何赛季）须在`QuestDefinition`/赛季配置中显式声明，不得产生未定义行为（FR-GSM-044）。默认规则：以对局**开始时间**所属赛季结算。

---

# 7. 标准化检查清单

## 7.1 上线前检查清单

- [ ] 排行榜滞后监控（§2.5）已与派生视图功能同批上线
- [ ] 派生视图数据结构选型已完成评审（TBD-GSM-001/ISS-046决议）
- [ ] 赛季结算路径的故障注入试验（中断后重触发）验证幂等，无重复/遗漏奖励
- [ ] 任务奖励、邮件附件领取路径均验证复用FR-EC-003，无独立发放旁路
- [ ] 黑名单查询边界验证：非`owner_id`无法查得黑名单内容
- [ ] 举报路径验证：单次举报不触发`AdminService`自动调用
- [ ] 赛季继承规则（TBD-GSM-002）已与策划评审确定并写入配置
- [ ] 举报处理SLA（TBD-GSM-003）已与运营团队评审确定
- [ ] 新增排行维度已通过`RankingDimensionConfig`配置验证，未修改`RankingViewUpdater`代码路径（FR-GSM-001）
- [ ] 派生视图死信/重建路径（§2.3.1）已具备可操作的运维手册，重建期间查询降级提示已实现
- [ ] `PlayerReport.dedup_key`唯一索引已在DDL中落地，未仅依赖应用层去重（FR-GSM-033）
- [ ] `PlayerBlocklist(owner_id, blocked_id)`唯一约束已落地，`blocked_id`侧无可反查`owner_id`的索引/接口（NFR-GSM-005）
- [ ] 注：`RankingViewUpdater`死信处理、`ReporterReputation`异步重算为本批新增的常态运维面，OLU运维负荷未核算，见ISS-065

## 7.2 代码评审检查清单

- [ ] 排行榜查询路径未出现对权威表的实时全表排序查询
- [ ] 新增任务触发条件类型未修改已有任务代码路径
- [ ] 邮件系统未出现与FR-AD-002批量补偿重复的独立发放逻辑
- [ ] 赛季切换流程未出现跨越边界的非原子写入

---

# 8. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-031、FR-GSM-001〜006 | §2、§2.2.1（FR-GSM-001维度配置）、§2.3.1（死信/重建异常分支） |
| FR-GSM-010〜015 | §3 |
| FR-GSM-020〜024 | §4 |
| FR-GSM-030〜035 | §5、§5.1.1（RSK-GSM-002信誉度机制） |
| FR-GSM-040〜044 | §6 |
| NFR-GSM-001〜006 | §2.5、§3.3、§6.2 |
| AC-GSM-001〜005 | §7.1 |
| TBD-GSM-001〜003 | §2.2、§6.2、§7.1 |
| RSK-GSM-001〜002 | §2.5、§7.1 |

---

> 本文档与RGS-REQ-017（排行榜、任务成就与玩家治理 需求定义书）配套使用。
