# 附件A 术语表・缩略语一览

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-002 |
| 版本 | 1.1 |
| 父文档 | RGS-REQ-001 需求定义书 |
| 制定日 | 2026-08-15 |
| 最终更新日 | 2026-08-17 |

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 1.0 | 2026-08-15 | 架构师 | 初版制定 |
| 1.1 | 2026-08-17 | 架构师 | **处置ISS-076**：新增§5A"领域扩展术语"，回补RGS-REQ-014（NEURO）起13个域文档正文中声明、但从未回收至本附件的域内术语，恢复本附件"唯一基准"的治理承诺 |

---

## 1. 本附件的定位

本附件是本项目所用术语的**唯一基准**。设计书、代码注释、试验规格书、问题工单中的用词**必须**遵从本附件。表述出现分歧时，以本附件为准。

新术语的追加**必须**通过本附件的修订进行。未登录于本附件的自造术语**不得**在正式文档中使用。

本文档体系遵循日本 IPA（独立行政法人情报处理推进机构）的《要件定義書》标准框架，故部分框架层级术语（如「非機能要求グレード」「共通フレーム」）标注日文原名，以便对照该标准原文；除此之外的正文一律使用中文。

---

## 2. 核心架构术语

| 中文（采用表述） | 日文原名 | English | 定义 | 关联需求 |
|---|---|---|---|---|
| 场景Actor | シーンアクター | Scene Actor | 单一场景状态的唯一写入者。1场景 = 1个Tokio task。本系统中Actor的**唯一粒度** | ARC-001、FR-RT-001 |
| ECS | ECS | Entity Component System | 场景内实体的数据布局与处理方式。实体为ID，状态为组件，处理为system | ARC-001、FR-RT-002 |
| 实体 | 実体（エンティティ） | Entity | 场景内可模拟的对象（玩家、NPC、投射物）。**玩家不是独立Actor，而是实体** | ARC-001 |
| 邮箱 | メールボックス | Mailbox | Actor的输入队列。本系统中**必须为有界** | CON-008、ARC-013 |
| 监督 | 監督（スーパービジョン） | Supervision | Actor异常终止时的隔离与再启动机制 | FR-RT-010 |
| 排空 | ドレイン | Drain | 节点退出前停止接受新连接，安全转移并确定既存状态的处理 | FR-GW-009、FR-RT-012 |
| 限界上下文 | 限界コンテキスト | Bounded Context | 拥有独立数据与用语体系的业务边界。本系统数据所有权的划分单位 | ARC-008、DR-004 |
| 数据所有权 | データ所有権 | Database Ownership | 一个限界上下文排他性地拥有自身数据的原则 | DR-004、DR-005 |

## 3. 实时同步术语

| 中文（采用表述） | 日文原名 | English | 定义 | 关联需求 |
|---|---|---|---|---|
| 周期／帧 | ティック | Tick | 固定周期的模拟单位。本系统为20Hz（50ms） | NFR-PE-001 |
| 兴趣区域 | 関心領域（AOI） | Area of Interest | 应向各玩家配送的实体集合，以均等网格计算 | FR-SY-001、ARC-002 |
| 基线 | ベースライン | Baseline | 客户端已确认的状态，是差分计算的基准点 | FR-SY-004 |
| 差分快照 | 差分スナップショット | Delta Snapshot | 仅包含与基线之差的状态更新 | FR-SY-003 |
| 客户端预测 | クライアント予測 | Client-side Prediction | 不等待服务器应答、将输入立即在本地应用 | ARC-002 |
| 服务器和解 | サーバー和解 | Server Reconciliation | 检测权威状态与预测结果的差异并重新应用 | ARC-002、FR-SY-008 |
| 延迟补偿 | ラグ補償 | Lag Compensation | 判定时将目标对象回溯至历史位置 | FR-RT-007 |
| 量化 | 量子化 | Quantization | 将浮点值转换为固定精度整数以节省带宽 | FR-SY-006、ARC-004 |
| 位打包 | ビットパッキング | Bit Packing | 不按字节对齐、紧密编码各字段 | FR-SY-006、ARC-004 |
| 不可靠通道 | 不可靠経路（Datagram） | Unreliable Channel | 不进行重传的发送通道，用于高频状态 | IF-001-1、ARC-003 |
| 可靠通道 | 可靠経路（Stream） | Reliable Channel | 具备到达保证的发送通道，用于不可丢失的事件 | IF-001-2、ARC-003 |
| 带宽预算 | 帯域予算 | Bandwidth Budget | 单玩家下行带宽上限，是优先级控制的约束条件 | NFR-PE-006、FR-SY-005 |

## 4. 数据一致性术语

| 中文（采用表述） | 日文原名 | English | 定义 | 关联需求 |
|---|---|---|---|---|
| 实时状态 | リアルタイム状態 | Realtime State | 场景Actor在内存中拥有权威、可容忍丢失的状态 | DR-001 |
| 永久事实 | 永続的事実 | Durable Fact | PostgreSQL拥有权威、不可容忍丢失的数据 | DR-002 |
| 乐观并发控制 | 楽観的並行制御（OCC） | Optimistic Concurrency Control | 以`version`一致为更新条件，不一致视为冲突的方式 | DR-007〜009 |
| 事务性发件箱 | トランザクショナル Outbox | Transactional Outbox | 与业务更新处于同一事务内、将事件写入表的方式 | DR-011〜015 |
| 双写 | 二重書き込み | Dual Write | 先提交数据库再向外部基础设施发布的错误方式。**禁止** | DR-012 |
| 变更数据捕获 | CDC | Change Data Capture | 从数据库日志（WAL）捕获变更并向外部传播的方式 | FR-EV-002 |
| 幂等 | 冪等 | Idempotent | 同一请求无论执行多少次，副作用只发生一次的性质 | ARC-009 |
| 至少一次 | At-Least-Once | At-Least-Once | 允许重复投递的投递保证，是本系统的默认设置 | ARC-009 |
| 实效一次 | Effectively Once | Effectively Once | 由幂等、唯一约束、版本号、状态机组合实现的结果性唯一性 | ARC-009 |
| 防护令牌 | フェンシングトークン | Fencing Token | 通过单调递增标识符排除旧写入者的机制 | ARC-005 |
| 会话世代 | セッションエポック | Session Epoch | 本系统中防护令牌的具体实现，每次登录递增 | FR-PL-003、ARC-005 |
| 分区键 | パーティションキー | Partition Key | 保证同一聚合的事件落入同一分区的键 | ARC-010 |
| 顺序边界 | 順序境界 | Ordering Boundary | 顺序保证成立的范围，不假设全局顺序 | ARC-010 |
| 补偿 | 補償 | Compensation | 对已成功的操作进行业务上的撤销处理 | FR-WF-003、ARC-011 |
| 死信队列 | DLQ | Dead Letter Queue | 永久性失败消息的隔离目的地 | FR-EV-005 |

## 5. 运维・质量术语

| 中文（采用表述） | 日文原名 | English | 定义 | 关联需求 |
|---|---|---|---|---|
| 背压 | 背圧 | Backpressure | 根据下游处理能力抑制上游速率的机制 | ARC-013 |
| 负载卸除 | ロードシェディング | Load Shedding | 过载时主动拒绝部分请求 | ARC-013、NFR-PE-018 |
| 熔断器 | サーキットブレーカ | Circuit Breaker | 连续失败时切断调用、等待恢复的机制 | ARC-013 |
| 降级运行 | 縮退運転 | Degraded Operation | 停止部分功能、维持核心功能的运行方式 | NFR-AV-009 |
| 检查点 | チェックポイント | Checkpoint | 实时状态的周期性保存 | FR-RT-009、NFR-AV-006 |
| 目标恢复时间 | 目標復旧時間（RTO） | Recovery Time Objective | 从故障发生到恢复的允许时长 | NFR-AV-003、004 |
| 目标恢复点 | 目標復旧地点（RPO） | Recovery Point Objective | 允许的数据丢失时间跨度 | NFR-AV-005、006 |
| 同时在线数 | 同時接続数（CCU） | Concurrent Users | 同时连接服务器的玩家数量 | NFR-PE-013〜015 |
| 非功能需求等级 | 非機能要求グレード | Non-Functional Requirements Grade | IPA制定的非功能需求分类与等级体系 | 第9章、附件B |
| 架构决策记录 | アーキテクチャ決定記録（ADR） | Architecture Decision Record | 记录方案决策及其依据与被否决方案的文档 | ARC-014、AI-007 |
| 可追溯性矩阵 | トレーサビリティマトリクス | Traceability Matrix | 需求与设计・实现・试验对应关系表 | 附件C |
| 扩展-收缩 | Expand-Contract | Expand-Contract | 保持兼容性的两阶段数据库结构迁移手法 | NFR-MI-002 |

## 5A. 领域扩展术语（本次新增，回补NEURO起13个域，处置ISS-076）

> **背景**：本附件§1已声明为"术语表述分歧防止的唯一基准"，但RGS-REQ-014（NEURO域）起的13个域文档各自在其正文§2"术语约定"声明了域内新增术语，从未被回收合并至本附件——本表首次系统性回补，此后新域的术语约定须同批次同步至此（同附件A原有"新术语的追加必须通过本附件的修订进行"的既定规则）。

| 域 | 中文（采用表述） | English | 定义 | 关联需求 |
|---|---|---|---|---|
| NEURO | 确定性分级 | Determinism Tier | 按"同一输入是否必产生同一输出"对系统各层的分级，L0（绝对确定性）〜L4（非确定性） | RGS-REQ-014§4.3.1 |
| NEURO | 确定性闸门 | Determinism Gate | 数据/控制从低确定性层流向高确定性层时必须穿过的三重校验结构（枚举白名单+值域校验+人工审批） | ARC-030 |
| NEURO | 建议 | Recommendation | 智能层唯一产出形式：不具备执行力，须经既有GM控制平面人工审批 | FR-NEURO-022 |
| NEURO | 全局开关 | Global Kill Switch | 智能层整体启停的唯一收口点，默认关闭，仅AdminService可写 | FR-NEURO-049 |
| GSM | 派生排行视图 | Derived Ranking View | 排行榜对外查询的数据形态，由权威数据源派生，允许短暂滞后但不构成独立权威 | ARC-031 |
| GSM | 举报 | Report | 玩家对另一玩家可疑/违规行为提交的结构化反馈，检测信号来源之一 | FR-GSM-030 |
| GSM | 赛季 | Season | 有明确起止时间的运营周期，段位/排行榜在边界处按既定规则重置或继承 | RGS-REQ-017§2 |
| TRD | 挂单 | Offer | 交易发起方声明"以A换B"的意向，尚未被对方确认 | RGS-REQ-018§2 |
| TRD | 原子成立 | — | 交易双方转移在同一事务边界内同时成立或同时不成立 | ARC-032 |
| SUP | 工单 | Ticket | 玩家发起的、针对自身账号问题的申诉记录 | RGS-REQ-019§2 |
| SUP | 掉单 | — | 玩家已完成支付但系统内部订单未收到成功回调，导致权益未发放 | ARC-033 |
| INF | 容灾 | Disaster Recovery（DR） | 区域级故障时的业务连续性保证机制 | ARC-034 |
| INF | 数据分析管线 | Analytics Pipeline | 面向产品/运营决策的用户行为数据处理链路，区别于面向运维故障定位的可观测性 | ARC-035 |
| IDN | 身份提供方 | Identity Provider（IdP） | 第三方登录服务，验证用户真实身份并签发身份令牌 | ARC-036 |
| IDN | 账号联合 | Account Federation | 一个游戏账号绑定多个第三方身份，任一身份均可登录同一账号 | RGS-REQ-021§2 |
| OPT | 推送 | Push Notification | 玩家未在线时经系统级通道（APNs/FCM）送达的提醒消息，区别于站内邮件 | ARC-037 |
| PLT | 收据 | Receipt | 平台为一笔内购交易签发的、可被服务器验证真伪的凭证 | ARC-038 |
| PLT | 逻辑服 | Realm/Server | 玩家数据相互隔离的独立游戏世界实例 | RGS-REQ-023§2 |
| PLT | 合服 | — | 将两个以上逻辑服的玩家数据合并至同一逻辑服的运维操作 | RGS-REQ-023§7 |
| CAP | 容量级别 | Capacity Tier | 按并发规模划分的架构演进阶段（T0〜T3） | ARC-040 |
| CAP | 横向分片 | Horizontal Shard | 相互独立、各自完整的服务器部署单元，分片间玩家数据不互通 | RGS-REQ-025§2 |
| CAP | 弹性预留 | Elastic Reservation | 常态负载之上预先保留的容量余量，吸收突发流量在自动扩容生效前的窗口期 | FR-CAP-020 |
| PPL | 前处理／后处理 | Pre/Post-processing | 业务逻辑执行前/后的统一处理阶段（鉴权限流校验等/序列化脱敏审计等） | ARC-041 |
| PPL | 旁路 | Bypass | 请求处理绕过标准管道某阶段直接到达业务逻辑或直接返回响应的路径，原则禁止 | FR-PPL系 |
| DEP | 集群清单 | Cluster Manifest | 声明式文件，列出目标集群应含的全部Atomic App及目标版本/环境参数 | ARC-042 |
| DEP | 依赖图 | Dependency Graph | App之间先后依赖关系的有向无环图（DAG），决定编排执行顺序 | RGS-REQ-027§2 |
| ANT | 检测信号 | Detection Signal | 单次产生的、可疑但不足以单独定案的异常观测 | ARC-043 |
| ANT | 案件 | Case | 针对某玩家的多个检测信号（或举报）聚合后形成的待审核治理对象 | FR-ANT-010 |

## 6. 缩略语一览

| 缩略语 | 全称 | 备注 |
|---|---|---|
| ADR | Architecture Decision Record | 架构决策记录 |
| AOI | Area of Interest | 兴趣区域 |
| CCU | Concurrent Users | 同时在线数 |
| CDC | Change Data Capture | 变更数据捕获 |
| DLQ | Dead Letter Queue | 死信队列 |
| ECS | Entity Component System | 实体组件系统 |
| MTU | Maximum Transmission Unit | 最大传输单元 |
| OCC | Optimistic Concurrency Control | 乐观并发控制 |
| OSI | Open Source Initiative | 开源许可认证机构（CON-001） |
| OTLP | OpenTelemetry Protocol | 可观测性数据传输协议 |
| QUIC | — | RFC 9000。基于UDP的加密多路复用传输协议 |
| RACI | Responsible, Accountable, Consulted, Informed | 责任分工表 |
| RBAC | Role-Based Access Control | 基于角色的访问控制 |
| RPO | Recovery Point Objective | 目标恢复点 |
| RTO | Recovery Time Objective | 目标恢复时间 |
| RTT | Round-Trip Time | 往返延迟时间 |
| SLO | Service Level Objective | 服务水平目标 |
| WAL | Write-Ahead Log | 预写日志 |

---

## 7. 表述规则（防止用词分歧）

| 采用表述 | 不得使用的表述 | 理由 |
|---|---|---|
| 场景Actor | 场景服务器、场景节点、区域Actor | 明确表达职责单位的唯一性 |
| 实体（Entity） | 对象、单位、角色实例 | 与ECS术语体系保持一致 |
| 永久事实 | 主数据、永久数据、DB数据 | 与数据分类（DR-002）保持一致 |
| 实时状态 | 易失状态、内存状态、临时状态 | 与数据分类（DR-001）保持一致 |
| 确定请求 | 保存请求、持久化请求、提交请求 | 明确表达ARC-006所定义的边界 |
| 会话世代 | 会话代数、登录ID、化身编号 | 与ARC-005的实现命名保持一致 |
| 事件基础设施 | Kafka、MQ、消息队列 | 避免将特定产品名混入需求文档（NFR-MI-005） |
| 工作流基础设施 | Temporal、Saga引擎 | 同上 |
| 缓存基础设施 | Valkey、Redis | 同上。但在已确定采用产品的设计书中可使用产品名 |

> **原则**：需求定义书使用**角色名**，不使用产品名。产品名在方案设计书及ADR中确定。此举是为了保障NFR-MI-005（可替换性）。
