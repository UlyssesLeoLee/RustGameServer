# 附件A 术语表・缩略语一览

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-002 |
| 版本 | 1.0 |
| 父文档 | RGS-REQ-001 需求定义书 |
| 制定日 | 2026-08-15 |

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 1.0 | 2026-08-15 | 架构师 | 初版制定 |

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
