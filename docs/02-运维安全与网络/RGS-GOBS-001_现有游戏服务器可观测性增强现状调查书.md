# 现有游戏服务器可观测性增强现状调查书

**RGS-GOBS-001**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GOBS-001 |
| 版本 | 0.1 |
| 状态 | 草案，待技术评审与责任人签字 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 调查范围 | Rust 游戏服务器设计文档、运维说明、技术选型、工作流与当前工作区 |
| 上位依据 | RGS-REQ-001、RGS-REQ-008、RGS-BAS-001、RGS-BAS-004、RGS-DTL-004、RGS-OPS-001、RGS-TS-001 |
| 结论性质 | 现状证据调查；不等同于已部署证明，不授权直接进入实现 |

## 1. 调查结论

本次调查确认了一个必须在入口处写明的事实：

> 当前工作区有完整的架构与运维设计基线，但未发现可执行的 Rust workspace、服务源码、Cargo manifest、数据库迁移、Docker/Helm/Kubernetes/K3s 清单或实际部署记录。因此下文凡标记为“设计基线”的内容，均不能表述为“当前线上已经存在”。

可确认的逻辑目标链路为：

玩家体验 → 游戏运行时 → 网关/业务服务 → 数据库/缓存/事件基础设施 → 集群平台 → 基础设施。

可确认的可观测性目标为：

Grafana → 指标 → trace → 结构化日志 → 游戏运营事件，

并通过统一关联键 trace_id、span_id、request_id、player_id、event_id、workflow_id、match_id 进行跨层检索。

当前不能确认的事项包括：

- 是否已经有 K3s 集群、节点池、Ingress、Service、Deployment、StatefulSet、ConfigMap 或 Secret。
- 是否实际运行 PostgreSQL 18.4、Redis/Valkey、NATS JetStream 或任何事件总线。
- 是否存在 Gateway、Runtime、Player、Economy、Match、Social、Admin 等可执行服务。
- OTel Collector、Prometheus、Grafana、日志存储、trace 存储是否已部署。
- Rust、Actix Web、数据库客户端和 exporter 的实际版本。
- 任何吞吐、延迟、CPU、内存、网络、采样率、存储保留期或告警触发数据。

因此，本书的出口结论是：

1. 可以据现有设计文档继续完成可观测性需求与基本设计。
2. 不可以据本书宣称“监控系统已接入”或开始大规模埋点实现。
3. 进入 53 开发环境构筑前，必须完成 §10 的证据补齐门禁。

## 2. 证据分级与调查方法

### 2.1 证据分级

| 等级 | 含义 | 本次状态 |
|---|---|---|
| E0 | 工作区实际文件、命令输出、运行记录直接证明 | 只证明“缺少实现文件”；未证明运行时不存在 |
| E1 | 受控设计文档中的目标架构或规范 | 大量存在 |
| E2 | 集群/主机/数据库/Collector 的实际运行证据 | 未提供，未发现 |
| E3 | 负载测试、故障演练、生产指标证明 | 未提供 |

### 2.2 调查动作

本次调查对工作区执行了以下只读检查：

- 检查 Git 工作树，保留其他 session 的修改，不做 reset/checkout。
- 枚举 Cargo.toml、Cargo.lock、*.rs、*.yaml、*.yml、*.proto、*.sql、Dockerfile、Helm Chart 和 .codegraph。
- 检查 docs/ 中的需求、基本设计、详细设计、技术选型、部署说明和工作流。
- 用现有文档的 FR/NFR/IF/ARC/PH 编号追踪网关、运行时、数据、事件与可观测性边界。
- 使用 OpenTelemetry Rust 官方文档确认 Resource、批量导出、传播器和热路径指标的设计依据；使用 Tokio Console 官方文档确认其只能作为受控调试工具。

## 3. 当前工作区实际状况（E0）

| 检查对象 | 结果 | 解释 |
|---|---|---|
| Rust workspace / Cargo.toml | 未发现 | RGS-PLAN-001 中的目标布局是规划，不是当前文件 |
| Rust 源码 / *.rs | 未发现 | 无法做真实 middleware、tick、DB pool 或 exporter 调查 |
| K3s/Kubernetes/Helm manifest | 未发现 | 无法确认 Deployment、StatefulSet、Service、NetworkPolicy 或 ServiceMonitor |
| Dockerfile / compose | 未发现 | RGS-OPS-001 的命令是操作说明，不是执行证据 |
| SQL migration / proto | 未发现 | 不能确认 PG schema、Outbox schema 或 gRPC 契约的实际版本 |
| 实际指标、日志、trace、告警 | 未提供 | 不能计算基线、容量或采样率 |
| 设计文档 | 存在 | 可作为 E1 逻辑基线，需与实现逐项对账 |

特别注意：RGS-OPS-001 中出现了 Cargo.toml、crates/、services/、k8s/、rgs-monitoring 等路径和对象，但它们在当前工作区没有对应实现文件。因此必须把运维说明中的“应执行命令”与“已执行结果”分开。

## 4. 现有设计架构基线（E1）

下面的图只表示 RGS 设计文档的目标关系；实线表示逻辑调用方向，虚线表示设计中提到但当前工作区未验证的部署对象。

~~~mermaid
flowchart LR
    P[玩家客户端] -->|QUIC / Datagram / Stream| GW[GW Gateway]
    O[运维与 GM] -->|HTTPS / RBAC| API[API Gateway\nPH-6]
    GW -->|validated input| RT[RT Realtime Runtime\nScene Actor / 20Hz tick]
    API --> PL[PL Player]
    API --> EC[EC Economy]
    API --> MT[MT Match]
    API --> GD[GD Social]
    API --> AD[AD Admin]
    RT --> SY[SY Sync / AOI]
    RT --> PL
    RT --> EC
    PL --> PG[(PostgreSQL\npermanent facts)]
    EC --> PG
    MT --> PG
    GD --> PG
    AD --> PG
    GW --> CACHE[(Cache / session / rate limit)]
    RT --> CACHE
    PG --> OUTBOX[Outbox / dispatcher]
    OUTBOX --> BUS[Event infrastructure\nproduct not proven]
    BUS --> WF[Workflow]
    GW -. OTLP .-> OTC[OTel Collector]
    RT -. OTLP .-> OTC
    PL -. OTLP .-> OTC
    EC -. OTLP .-> OTC
    MT -. OTLP .-> OTC
    GD -. OTLP .-> OTC
    AD -. OTLP .-> OTC
    OTC -.-> MS[(Metric store)]
    OTC -.-> LS[(Log store)]
    OTC -.-> TS[(Trace store)]
    MS -.-> G[Grafana / Alerting]
    LS -.-> G
    TS -.-> G
~~~

### 4.1 领域职责与权威性

| 域/组件 | 设计职责 | 运行时事实 | 可观测性重点 |
|---|---|---|---|
| GW | 连接终止、鉴权、session、路由、限流、过载与 drain | E1 only | 连接生命周期、握手、重连、RTT、拒绝、load shedding |
| RT | Scene Actor 单写者、20Hz tick、输入校验、战斗、场景转移 | E1 only | tick 时延/超时/lag、Actor 数、mailbox、实体数、广播 |
| SY | AOI 与派生同步状态 | E1 only | 同步频率、发送队列、丢弃/降级、状态版本 |
| PL/EC/MT/GD/AD | 永久事实与管理审计 | E1 only | gRPC、DB pool、事务、错误、审计事件 |
| Outbox/EV | 事实传播与异步消费 | 设计默认 Outbox；产品未证实 | outbox lag、重试、幂等、consumer delay |
| WF | 长事务/补偿工作流 | E1 only | workflow state、重试、等待时长、死信 |
| OB | trace/metrics/log 收集与查询 | E1 logical only | Collector 健康、丢弃、背压、查询可用性 |

### 4.2 关键流程的观测边界

| 流程 | 设计路径 | 必须关联的字段 | 不应做的事情 |
|---|---|---|---|
| 登录到入场 | Client → GW → Auth/PL → RT → Scene | trace_id、request_id、player_id、scene_id | 不把 player_id 当 metric label |
| session 生命周期 | connect → authenticate → bind → active → reconnect/drain | session_id（日志/trace）与生命周期事件 | 不为每个 session 建时间序列 |
| match/room | Match → room allocation → RT bind → result/persist | match_id、room_id、event_id | 不在 tick 内为每个 room 建 span |
| 游戏 tick | 20Hz，50ms 周期，Actor 单写 | scene_id 只进日志/诊断；指标用有限 bucket | 不在每 tick 或每实体写日志/trace |
| 状态同步 | QUIC Datagram 高频；可靠事件走 Stream | trace short prefix 只在连接/事件边界 | 不把完整 trace_id 重复写入每个 datagram |
| 持久化 | 业务服务 → PG，Outbox 记录事实 | trace_id、event_id、workflow_id | 不让 tick 同步调用 DB |
| 异步事件 | Outbox → event infrastructure → consumer → workflow | event header + trace link | 不假定 Kafka 已存在 |
| Admin/ClusterOps | 运维 API → RBAC → operation/audit | operator_id、operation_id、event_id | 不把审计日志与普通业务日志混为一类 |

## 5. 已有可观测性规范盘点

| 文档 | 已定义内容 | 证据 | 缺口 |
|---|---|---|---|
| RGS-REQ-001 §10 | ARC-017；全链路 trace、指标覆盖、结构化日志、SLO、15 分钟定位、SRE ≤2 | E1 | 无实现和运行证据 |
| RGS-BAS-001 §3/§4.8 | 服务拓扑、OTLP、trace propagation、指标/日志/trace 存储逻辑 | E1 | 物理部署和资源预算未证实 |
| RGS-REQ-008 | 统一埋点、日志、采样、脱敏、CI 检查要求 | E1 | 与本次游戏 runtime/业务体验指标仍需展开 |
| RGS-BAS-004 | 指标命名、字段、脱敏、高基数约束、span 命名、tick 无 span | E1 | 不选择物理后端；样本率待负载试验 |
| RGS-DTL-004 | 统一 instrumentation SDK 的 trait、脱敏、JSON schema、错误强制采样 | E1 | 不实现 exporter/Collector/存储 |
| RGS-BAS-031 | ClusterOps/CEM 设计引用 OTel Collector 与 Prometheus | E1 | 仍无真实组件清单或探针数据 |
| RGS-OPS-001 | OTel Collector、Prometheus、Grafana、日志/监控部署操作示例 | E1 | 示例路径与当前工作区不一致；K3s 未确认 |
| RGS-TS-001 §3.9 | OTel SDK/Collector、Prometheus、Grafana 目标技术选型 | E1 | 目标基线待审批/待安装验证 |
| RGS-REQ-015 | k6 负载结果应进入既有可观测性 Dashboard | E1 | 无 k6 脚本、结果和 Dashboard 实例 |
| RGS-DTL-031 §运行指标 | PFAU/ACK/OCC/fencing/CEM/DLQ 等 ClusterOps 指标 | E1 | 指标名、标签、阈值、保留和归档未闭合 |

## 6. 平台与中间件判断

### 6.1 K3s 与 Kubernetes

设计文档使用 Kubernetes 对象和通用 K8s 术语；RGS-OPS-001 的部分命令还假定 kubeadm/Helm。当前没有 K3s kubeconfig、节点信息或 manifest。故本项目的基本设计采用 Kubernetes-compatible 资源模型，并设置 K3s 适配门：

- 交付前必须提供 kubectl version、节点、命名空间、StorageClass、IngressClass、CNI、ServiceMonitor/PodMonitor 能力证据。
- 未完成适配门前，不得写“已部署在 K3s”。
- 若 K3s 仅作为轻量开发环境，生产目标仍需单独确认；不可把两者的资源、网络和存储行为混为一谈。

### 6.2 PostgreSQL、缓存与事件基础设施

| 对象 | 设计资料中的信息 | 当前可确认度 | 本调查结论 |
|---|---|---|---|
| PostgreSQL | RGS-TS-001 目标为 18.4；旧 OPS 文档仍有 16.x 描述 | E1 | 18.4 是目标基线，需在环境中核验 |
| Rust/Actix Web | RGS-TS-001（2026-08-21）目标为 Rust stable 1.97.1 / Actix Web 4.14.1 | E1 | 目标基线不是本机实测；CI/toolchain 核验后才可用于实现 |
| Cache | 需求使用 generic cache；TS-001 目标 Redis 7.2+ Cluster | E1 | 不能断言实际运行 Redis/Valkey |
| Event | RGS-REQ-001 ARC-014 默认 Outbox polling；TS-001 目标 NATS JetStream | E1 | 不能断言 Kafka；先用角色抽象与证据门 |
| Workflow | service state machine + retry 为默认 | E1 | 需要与实际消费者、补偿路径对账 |

ARC-014 的中间件引入门禁仍然有效：不得因为“可观测性”直接追加 Kafka、独立消息总线、日志平台或多套 Collector；必须先证明消费者数量、吞吐、重放、延迟、DB 负荷和 SRE OLU。

## 7. 六层可观测性缺口

| 层 | 需要回答的问题 | 当前已有设计 | 现状缺口 |
|---|---|---|---|
| 基础设施 | 节点/磁盘/网络是否成为瓶颈 | K8s/数据池/观测池逻辑 | 无节点指标、容量与告警证据 |
| 平台 | Pod、调度、HPA、Ingress、NetworkPolicy 是否正常 | Deployment/StatefulSet/HPA 设计 | 无 K3s/K8s 对象和事件 |
| 中间件 | PG、cache、Outbox、event、workflow 是否拖慢业务 | DB/缓存/事件角色定义 | 无 pool、lag、retry、backlog 实测 |
| 后端服务 | 请求错误、延迟、依赖失败在哪里 | RGS-BAS-004 指标/日志/trace 规范 | 无服务代码、middleware、trace 验证 |
| 游戏运行时 | tick、Actor、mailbox、同步、重连是否健康 | RGS-REQ-001 PE NFR | 无 Runtime 实现与负载基线 |
| 业务/玩家体验 | 登录、入场、匹配、战斗、结算是否受损 | FR/NFR 和业务域 DTL | 无玩家体验事件和 SLO 实测 |

## 8. 最佳埋点位置与侵入边界

优先在“边界”和“聚合点”埋点，而不是把埋点散落到每个业务语句：

1. GW 的连接、鉴权、路由、限流、drain middleware。
2. gRPC/HTTP server-client middleware，统一创建 server/client span。
3. repository/DB pool、事务、Outbox 写入边界。
4. event producer/consumer、workflow transition、重试与死信边界。
5. session 状态机、match/room 状态机、插件/App 激活和停用边界。
6. RT tick supervisor 与每秒聚合器；对 slow tick 生成诊断快照。
7. QUIC 连接/Stream/Datagram 统计与序列号/丢弃边界。
8. Collector、Prometheus、日志/trace 存储的自身运行指标。

明确禁止：

- tick 内同步 export、同步网络 I/O、同步 DB/消息调用。
- 每 tick、每 packet、每 entity 建 span 或写普通日志。
- 将 player_id、session_id、scene_id、room_id、match_id、request_id、trace_id 作为指标标签。
- 在代码中直接依赖某个 Grafana/Prometheus/OTel exporter，破坏原子 App 的可替换边界。

## 9. 当前风险清单

| 编号 | 风险 | 影响 | 处置 |
|---|---|---|---|
| GOBS-RSK-001 | 设计对象未映射到源码/manifest | 可能把不存在的组件当成已存在 | PH-0 建立实际 inventory 和 owner |
| GOBS-RSK-002 | K3s 未证实 | Collector、存储、Ingress 方案可能不可部署 | K3s 适配门；失败则保留 K8s-compatible |
| GOBS-RSK-003 | PG 18.4 与旧 OPS 16.x 冲突 | 环境和迁移验证误用版本 | 以 TS-001 目标为候选，环境核验后锁定 |
| GOBS-RSK-004 | Outbox/NATS/Kafka 语义混淆 | 引入不必要中间件或错误指标 | 先使用 event role；按 ARC-014 选型 |
| GOBS-RSK-005 | 过度埋点污染 tick | 玩家体验和容量目标受损 | 采用窗口聚合、有限标签、错误强制采样 |
| GOBS-RSK-006 | 物理后端未选定 | dashboard/retention/成本无法落地 | 在导入计划 PH-0/PH-1 设决策门 |
| GOBS-RSK-007 | 其他 session 正在修改文档 | 追踪关系可能短暂不一致 | 只引用已存在文档编号；最终运行一致性脚本 |
| GOBS-RSK-008 | BAS-004 与 OPS-001 出现不同指标名 | 告警和 dashboard 可能查询错误序列 | 建立唯一 metric registry；别名必须显式登记并测试 |
| GOBS-RSK-009 | OLU 使用人·天/周与 OLU/月两种口径 | 无法证明 SRE ≤2 约束 | PH-0 统一口径并重算 |
| GOBS-RSK-010 | 健康检查示例缺少统一响应语义 | K3s readiness/liveness 可能误判 | 在 façade/平台契约中定义 health states 和版本 |

## 10. 开发入口门禁

进入 53 开发环境构筑前，至少需要以下证据：

| 门 | 必需证据 | 出口 |
|---|---|---|
| GOBS-GATE-01 | 实际 Cargo workspace、crate/service 清单、Rust toolchain 输出 | 代码边界可追踪 |
| GOBS-GATE-02 | K3s/K8s 集群、节点、Ingress、存储、网络策略证据 | 平台边界可追踪 |
| GOBS-GATE-03 | PostgreSQL 18.4、缓存、event role 的实际版本/拓扑 | 中间件边界可追踪 |
| GOBS-GATE-04 | crates/observability API、Collector 接收/导出契约、日志 schema 评审通过 | 统一抽象可实现 |
| GOBS-GATE-05 | tick/登录/匹配/结算基线与容量预算 | 采样和指标阈值可计算 |
| GOBS-GATE-06 | RGS-GOBS-002/003/004 技术评审签字；与 RGS-BAS-004、RGS-DTL-004 无冲突 | 可进入实现 |

## 11. 后续调查输入

若要把 E1 设计升级为 E2/E3 现状，下一轮必须提供：

- 仓库源码或至少一个可构建的 Cargo workspace。
- CI 日志、Rust toolchain、Actix Web、sqlx/数据库客户端版本。
- K3s 集群只读诊断：节点、namespace、workload、service、ingress、storage、network policy。
- PostgreSQL 18.4、缓存、Outbox/event、workflow 的连接和拓扑（凭证脱敏）。
- 现有日志样例、指标列表、trace 样例、告警规则和一条故障演练记录。
- 100/1k/10k/目标 CCU 下 tick、连接、登录、匹配、结算的基线。
- 现有保留基线（原始指标 15 天、聚合指标 400 天、行为日志 400 天、审计日志 3 年）及其数据分类审批；若要改变，必须给出成本、合规和恢复理由。
- ClusterOps/CEM/PFAU 指标注册表、健康检查响应契约和 OLU 统一统计口径。

## 12. 调查自审结论

本书 Revision 1 的主要问题是容易把文档中的目标对象读成当前部署对象。Revision 2 的修正为：

- 增加 E0/E1/E2/E3 证据分级，明确当前工作区缺少实现文件。
- 不再假定 K3s、Kafka、Redis、OTel Collector 或 Grafana 已部署。
- 将 Rust/Actix/PostgreSQL 18.4 标为目标基线，要求环境核验后才锁定。
- 把 tick、packet、entity 的高频路径设为明确的侵入禁区。
- 把物理后端、K3s 适配和容量预算设置为导入门，而不是在调查阶段预先宣称完成。

**本书结论：E1 设计基线成立，E2/E3 现状未成立；实现门禁未通过。**
