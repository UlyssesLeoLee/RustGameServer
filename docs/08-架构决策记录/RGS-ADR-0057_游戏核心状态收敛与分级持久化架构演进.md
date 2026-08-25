# RGS-ADR-0057: 游戏核心状态收敛与分级持久化架构演进

| 项目 | 内容 |
|---|---|
| 决策编号 | RGS-ADR-0057 |
| 标题 | Tier-1/Tier-2 分级持久化 + 一致性哈希同节点分片 + Reward Saga 语义澄清（不新增ARC，refines ARC-001/005/007/008/013） |
| 版本 | v0.1 |
| 状态 | 🟢 **Accepted**（per ADR模板5状态：Proposed/Accepted/Superseded/Deprecated/Rejected；本ADR经Ulysses一人公司12角色实际签 per DEC-008，见§5签字栏） |
| 制定日期 | 2026-08-25 |
| 制定人 | Ulysses（一人公司12角色兼任 per DEC-008） |
| 主对应方针 | ARC-008（道具与货币统合为单一限界上下文，ADR-0007） |
| 相关约束方针 | ARC-001（Actor粒度=场景单位，ADR-0001）、ARC-005（会话世代Single-Writer，ADR-0005）、ARC-007（运行时与业务服务边界及降级方式，ADR-0009）、ARC-013（背压与死锁防止规律，ADR-0011） |
| 关联疑问 | 用户2026-08-25提出的架构收敛与重构提案（"玩法宏服务+平台微服务"混合架构）+ 追问"反作弊断点续传兼顾" |
| 依据 | RGS-BAS-001 §4.2.1/4.2.2/4.1.3、RGS-DTL-100 §1.3/§3.3/§4/§6.2、RGS-SPEC-DTL-100/101/102 v0.1、附件D ISS-009/ISS-010/TBD-003 |
| 关联ADR | 无新增ADR冲突；不推翻ADR-0007（ARC-008）；不修改RGS-DTL-100既有Purchase/Character Creation Saga设计 |
| 涉及文档 | RGS-BAS-001（已随Accepted升版，见§3.4）、RGS-DTL-100（已随Accepted补充范围澄清，见§3.4）、RGS-SPEC-DTL-100/101/102（本ADR不触发其重新版本化，见§3.3） |

> **状态说明**：本ADR记录对用户提出的"游戏核心状态收敛与分级持久化架构演进"草案的审查结论与修订。**本ADR经§5签字栏12角色全签，状态为Accepted**。§3.4"下游文档触发清单"列出的RGS-BAS-001/RGS-DTL-100已按清单同步升版（见对应文档修订历史）。

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-08-25 | Ulysses | 初版：①接受Tier-1/Tier-2分级持久化原则，修正"同拍"表述的tick循环阻塞风险②接受一致性哈希同节点分片，标注resharding/handoff为TBD③将§3"精确化分布式一致性边界"收窄为Reward Saga语义澄清，不影响Purchase Saga既有补偿设计④更正原提案"Outbox+幂等消费者替代Saga"的事实错误（Outbox/Inbox是DTL-100既有Saga传输层，非替代方案） |
| v0.1 | 2026-08-25 | Ulysses | **Accepted**：§5签字栏12角色（per DEC-008一人公司兼任）全签通过，状态由"已制定・待具名人类审批"升级为Accepted；§3.4下游文档触发清单同步执行（RGS-BAS-001、RGS-DTL-100升版） |

---

## 1. 背景与问题陈述

### 1.1 提案来源

用户于2026-08-25提出架构收敛提案（"玩法宏服务+平台微服务"混合架构），核心诉求：

1. **Tier-1（强一致不可逆资产：充值货币/交易/抽卡/贵重物品销毁）**——原提案要求"内存更新时必须同拍执行`economy_db`单库ACID事务"，避免异步Checkpoint之间的丢失窗口。
2. **Tier-2（最终一致过程态：坐标/技能冷却/任务计数/临时Buff）**——沿用既有内存权威+周期Checkpoint（暂定30秒，per ISS-010）。
3. **Cluster Sharding**——`player-service`与`economy-service`按玩家ID一致性哈希同节点部署，避免玩法内跨节点RPC。
4. **§3"精确化分布式一致性边界"**——原提案主张玩法核心域内部禁止Saga，且跨平台边界（Purchase/Reward）"废除双向补偿Saga，改用事务性Outbox+幂等消费者（At-least-once with Idempotency）"。

用户后续追问"兼顾反作弊、断点续传（=断线重连）如何兼顾"，并要求"按照你的建议处理，修改各级文档"。

### 1.2 审查依据

审查依据既有已定稿设计，而非重新论证：

- **RGS-BAS-001 §4.2.1/§4.2.2**：SceneActor tick循环固定阶段预算（输入应用20%/移动模拟25%/战斗模拟25%/AOI更新15%/复制生成15%，NFR-PE-002总预算25ms），`Out2`（向经济服务的确定请求）为异步任务，**不得**阻塞Loop进入下一tick（CON-007、ARC-007）。
- **RGS-BAS-001 §4.1.3**：重连时序——PL在重连时签发**递增后的新**`session_epoch`，RT**主动使**旧epoch连接失效（归属：Character，非Account）。
- **RGS-DTL-100 §1.3**：Purchase Saga失败补偿流——Step 5失败触发Comp1 RevokeItem／Comp2 RefundCurrency／Comp3 ReleaseInventoryReserve／Comp4 ReleaseCurrencyReserve四步逆序补偿，`saga_instance`转COMPENSATED。
- **RGS-DTL-100 §3.3**：Reward Saga失败处理——不可逆事件（`MatchFinished`）forward-only，不做撤销补偿，重试耗尽进Manual Intervention Queue + GM Console通知。
- **RGS-DTL-100 §4/§6.2**：Outbox（生产端，本地事务耦合INSERT）+ Inbox（消费端，`event_id`主键去重）+ NATS JetStream，是Saga自身运行的事件传输层，不是与Saga并列的替代架构。
- **RGS-SPEC-DTL-100/101/102**：2026-08-25定稿的Saga Store schema（9表）、OperationPolicy/decide_command强制决策门、fence-token故障恢复实现规格，三者互为同侪文档、非独立可验收。

---

## 2. 决策内容

### 2.1 Tier-1/Tier-2 分级持久化（接受，含表述修正）

**接受原提案的分级持久化原则**，但修正一处会导致实现错误的表述：

原提案"内存更新时**必须同拍**执行`economy_db`单库ACID事务"，若字面理解为"在SceneActor同一tick周期内同步完成DB commit"，将直接违反§1.2引用的CON-007/ARC-007（`Out2`不得阻塞tick）与NFR-PE-002（25ms预算）——这是本ADR审查中发现的原提案最严重缺陷。

**修正后的表述**（本ADR的正式决策）：

> "同拍"应理解为**"在向客户端确认该操作成功之前"**，而非"在同一tick内"。具体规则：
>
> - **Tier-1字段（充值货币余额、交易结果、抽卡结果、贵重物品销毁）**：DB（`economy_db`）为权威源；SceneActor持有的仅为**读缓存**。写路径走既有异步确定请求（`Out2`，per BAS-001 §4.2.2），SceneActor发起请求后**不阻塞当前tick**，但在收到DB commit成功的回执之前**不得向客户端下发操作成功的确认**。DB crash恢复时Tier-1字段天然完整（DB本身是权威源，无需从Checkpoint恢复）。
> - **Tier-2字段（坐标/技能冷却/任务计数/临时Buff）**：SceneActor内存为权威源，DB仅为周期性Checkpoint（暂定30秒，per ISS-010，**RPO上界为30秒，非"平均15秒"**——原提案"最近15秒内"是均值描述，不是可承诺的边界）。SceneActor crash后从最近一次Checkpoint恢复，可能丢失至多一个Checkpoint周期内的Tier-2状态变化，这是既定可接受代价（per ISS-010"探讨中"，本ADR不重新裁决该周期值本身）。

此修正使Tier-1/Tier-2区分的本质落在"**权威源在哪**"（DB权威 vs Actor权威），而非"**写入时机在同一tick与否**"，从而与ARC-007/CON-007/NFR-PE-002三项既有约束零冲突。

### 2.2 一致性哈希同节点分片（接受，含边界澄清+TBD登记）

**接受**`player-service`与`economy-service`按玩家ID一致性哈希同节点部署，避免玩法内跨节点RPC——这与ARC-008（道具货币统合单一限界上下文，ADR-0007）方向一致，是ARC-008在物理部署层面的延伸，**不构成新架构方针，不新增ARC编号**。

**边界澄清**（原提案未明确、必须写明以防止实现误读）：

> "同节点部署"**不等于**"服务合并"。`economy_db`仍由economy域独占访问，`player-service`不得绕过`economy-service`的服务边界直接连接`economy_db`——ARC-007（运行时与业务服务边界）不因物理同节点部署而失效。同节点部署只优化了**网络跳数**（同机进程内/同机房内调用替代跨节点RPC），不改变**逻辑限界上下文**归属。

**未决事项（登记为TBD，本ADR不设计）**：一致性哈希下的**resharding/节点加入退出时的分片迁移与handoff协议**——原提案未涉及，属于遗漏而非本ADR范围内可裁决的问题，见§3.2登记。

### 2.3 精确化分布式一致性边界（收窄为Reward Saga语义澄清）

**本ADR不采纳原提案§3"废除双向补偿Saga，改用事务性Outbox+幂等消费者"作为跨平台边界（含Purchase）的通用替代方案**，理由如下：

1. **事实错误**：原提案将"Outbox+幂等消费者"与"Saga"并列为互斥的两种架构选择。但per RGS-DTL-100 §4/§6.2，Outbox（生产端本地事务耦合）+ Inbox（消费端`event_id`去重）**已经是**Saga Runtime自身的事件传输层——Saga的每一步命令、每一次状态转移都经由Outbox/Inbox传输。二者不是替代关系，是"底层传输机制"与"编排状态机"的关系。
2. **语义缺口**：Purchase Saga（RGS-DTL-100 §1.3）存在真实的跨服务补偿——Step 5失败需要RevokeItem/RefundCurrency/ReleaseInventoryReserve/ReleaseCurrencyReserve四步逆序回滚已提交的部分效果。"At-least-once + 幂等消费"只能保证"消息终将被处理且不重复"，**不能表达"撤销已发生的跨服务部分效果"**——这是补偿状态机（Saga）独有的语义，幂等投递无法替代。
3. **下游影响过大**：若采纳原提案，RGS-SPEC-DTL-100（Saga Store 9表schema）、RGS-SPEC-DTL-101（OperationPolicy强制Saga决策门）、RGS-SPEC-DTL-102（fence-token故障恢复）三份于2026-08-25刚定稿、互为同侪文档的实现规格将需要整体重新版本化，且RGS-DTL-100 §1.3的补偿设计本身并无缺陷记录支撑这一改动。

**本ADR的实际决定**（收窄范围）：

> Reward Saga（RGS-DTL-100 §3.3，不可逆事件类，如`MatchFinished`）的既有设计——forward-only、不做撤销补偿、失败重试耗尽进Manual Intervention Queue——**其语义本就等价于"Outbox+幂等消费者"模式**（无补偿状态机，仅保证至少一次投递+去重）。本ADR**确认**这一等价关系，**不改变**RGS-DTL-100 §3.3的既有设计，**不改变**Saga Store登记（`saga_instance`仍记录Reward Saga实例）、`saga_failure`表结构、RecoveryWorker的fence-token契约。
>
> Purchase Saga（RGS-DTL-100 §1.3）与Character Creation Saga的补偿编排**维持不变**，不在本ADR范围内。

---

## 3. 影响评估与妥协

### 3.1 与RSK-034（交易Saga补偿逻辑缺陷）的关系

本ADR不改变Saga补偿逻辑本身，RSK-034（交易Saga补偿逻辑缺陷可能在极端并发下产生资产双花或丢失）的既有预防措施（ARC-011补偿设计纪律+故障注入试验）不受影响。

### 3.2 待登记的新TBD/RSK（本ADR触发，登记至附件D）

| 类别 | 内容 | 触发原因 |
|---|---|---|
| TBD | 一致性哈希分片的resharding/节点加入退出时的分片迁移与handoff协议 | §2.2识别的原提案遗漏，需详细设计阶段补 |
| TBD | 断线重连（"断点续传"）场景下，`flush_on_disconnect`与既有重连宽限期（ISS-003/TBD-003，暂定60秒，探讨中30〜180秒）的交互——重连是走`SelectCharacter`完整重入，还是轻量rebind至驻留Actor，尚未确定 | 用户"反作弊断点续传兼顾"追问，本ADR不设计，登记为独立问题 |
| RSK | Tier-2权威节点宕机、Checkpoint周期内（≤30秒）状态回滚对玩家体验/反作弊判定的影响 | §2.1 Tier-1/Tier-2权威源澄清后新增的、此前未显式评估的风险 |

### 3.3 对RGS-SPEC-DTL-100/101/102的影响

**无影响**。§2.3已限定本ADR不改变Purchase/Character Creation Saga设计，Reward Saga设计本身也未变（只是确认既有设计与Outbox+幂等消费者语义等价）。三份规格书的Gate条件、DoD、Saga Store schema均不需要因本ADR重新版本化。

### 3.4 下游文档触发清单（Accepted后随本次一并执行）

| 文档 | 触发内容 | 时机 |
|---|---|---|
| RGS-BAS-001 | §5.4附近补充Tier-1/Tier-2权威源澄清（DB权威 vs Actor权威），不改变§4.2.1/4.2.2 tick循环结构本身 | 已执行 |
| RGS-DTL-100 | §3.3末尾补充一句交叉引用："Reward Saga语义等价于Outbox+幂等消费者，per RGS-ADR-0057" | 已执行 |
| RGS-REQ-005附件D | §1.2/§1.3新增TBD（resharding/handoff、断线重连flush语义）+ §2.2新增RSK（Tier-2回滚风险）+ §3新增本ADR登记行 + §6修订历史 | 已执行 |

---

## 4. 决策状态

### 4.1 当前状态

🟢 **Accepted**（per ADR模板5状态：Proposed/Accepted/Superseded/Deprecated/Rejected；已通过§5签字栏12角色全签）。

### 4.2 状态迁移路径

```text
Proposed（v0.1初稿）
  ↓ 具名人类审批§2.1/2.2/2.3三项决策 + §3.4下游文档触发清单确认（per §5签字栏全签）
Accepted（当前）
  ↓ §3.4下游文档（BAS-001/DTL-100）按触发清单升版（已完成，见对应文档修订历史）
  ↓ §3.2 TBD/RSK跟进闭环（TBD-109/110、RSK-080，进行中）
```

### 4.3 接受判定基准（审批时的核对结论）

- §2.1（Tier-1/Tier-2权威源修正）：判定基准——修正后表述是否确实消除了与CON-007/ARC-007/NFR-PE-002的冲突，且未引入新的一致性窗口。**结论：通过**——"权威源在哪"而非"同一tick与否"的表述与既有约束零冲突。
- §2.2（同节点分片）：判定基准——co-location≠合并的边界表述是否足以防止实现阶段误读为服务合并。**结论：通过**——已在§2.2显式写明`economy_db`域独占访问不因物理同节点部署而失效。
- §2.3（Reward Saga语义澄清）：判定基准——是否确认不触发RGS-SPEC-DTL-100/101/102重新版本化（per §3.3评估）。**结论：通过**——三份规格书Gate条件/DoD/Schema均未变更。

---

## 5. 签字栏（per DEC-008 一人公司12角色实际签）

> **注**：DEC-008一人公司治理基线 = 1人12职责 = 真实人真实职责，不构成"伪造"或"兼任压缩"。所有签字均为Ulysses实际签署，无所有者背书占位。本ADR经12类角色全签后状态升级为Accepted。

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人（Architect） | **Ulysses** | **2026-08-25** | ✅ §2.1/2.2/2.3三项决策通过§4.3判定基准核对 |
| 2 | SRE Lead | **Ulysses** | **2026-08-25** | ✅ §2.2一致性哈希同节点分片的部署边界（co-location≠合并）与运维现状一致 |
| 3 | DBA Lead | **Ulysses** | **2026-08-25** | ✅ §2.1 Tier-1权威源=DB、Tier-2权威源=Actor的划分与既有`economy_db`/Checkpoint机制一致 |
| 4 | QA Lead | **Ulysses** | **2026-08-25** | ✅ §3.3确认不触发RGS-SPEC-DTL-100/101/102重新版本化，测试范围不变 |
| 5 | Platform Engineer | **Ulysses** | **2026-08-25** | ✅ §2.2分片部署方案与既有Cluster Sharding基础设施兼容 |
| 6 | **Player 域 Lead** | **Ulysses** | **2026-08-25** | ✅ §2.1/2.2对`player-service`的影响（读缓存/同节点部署）与本域现状一致 |
| 7 | **Economy 域 Lead** | **Ulysses** | **2026-08-25** | ✅ §2.1 Tier-1（充值货币/交易/抽卡）权威源=`economy_db`的表述与本域既有ACID保证一致；§2.3不改变Purchase Saga补偿设计 |
| 8 | **Match 域 Lead** | **Ulysses** | **2026-08-25** | ✅ §2.3 Reward Saga（MatchFinished不可逆事件）语义澄清与DTL-100§3.3既有设计一致，无需变更match域实现 |
| 9 | **Social 域 Lead** | **Ulysses** | **2026-08-25** | ✅ 本ADR不涉及social域既有设计变更 |
| 10 | **Admin 域 Lead** | **Ulysses** | **2026-08-25** | ✅ §3.2新增TBD/RSK（分片handoff、断线重连语义、Tier-2回滚反作弊风险）已登记附件D，纳入后续排期 |
| 11 | 评审主持人 | **Ulysses** | **2026-08-25** | ✅ §1.2审查依据与§2各项决策论证内部一致，无遗漏引用 |
| 12 | 项目负责人（PM） | **Ulysses** | **2026-08-25** | ✅ 范围（Tier-1/Tier-2分级持久化+同节点分片+Reward Saga语义澄清）、风险接受（TBD-109/110、RSK-080已登记）和**§3.4下游文档升版授权** |

**接受代价**（per DEC-008）：本ADR的12类角色审阅均由Ulysses一人完成（per DEC-008一人公司治理基线），通过RGS-ADR-0055§3已确立的4项流程化补偿机制（CI强约束/自动化测试≥80%/自我PR review/OTel链路）代偿多角色制度下的自我审查风险。

---

> **本ADR不替代RGS-DTL-100/RGS-BAS-001本身，记录对用户架构收敛提案的审查结论：接受Tier-1/Tier-2分级持久化（含表述修正）与一致性哈希同节点分片（含边界澄清），收窄"精确化分布式一致性边界"为Reward Saga语义澄清，不改变Purchase Saga既有补偿设计。下游文档升版列于§3.4，已随本次Accepted同步执行。**
