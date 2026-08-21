# 游戏服务器可观测性基本设计书

**RGS-GOBS-003**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GOBS-003 |
| 版本 | 0.1 |
| 状态 | 草案，待技术评审、平台评审和容量评审 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 上位需求 | RGS-GOBS-002 |
| 扩展对象 | RGS-BAS-001 §4.8、RGS-BAS-004、RGS-DTL-004 |
| 设计性质 | 逻辑基本设计；不代表已安装 Grafana/Prometheus/Collector |

## 1. 设计原则与边界

本书不另造一套与游戏服务器脱节的监控系统，而是在 GW、RT、业务域、数据库、缓存、Outbox/event、workflow、K3s/Kubernetes 和基础设施的既有边界上增加可观测性能力。

设计原则：

1. 从玩家体验反向定位：Player Experience → Runtime → Service → Middleware → Platform → Infrastructure。
2. 用统一 façade 隔离后端；业务代码不直接依赖 exporter、Grafana、Prometheus client 或日志存储。
3. 在边界和聚合点埋点；不在 20Hz tick、QUIC datagram、实体循环中做同步 I/O。
4. OTel、Prometheus scrape、结构化日志和游戏事件各司其职，不强行统一为一种传输。
5. 所有队列、batch、retry、buffer 有界；telemetry 失败优先降级，不阻塞游戏核心。
6. 指标标签低基数；玩家/房间/场景等个体值进入日志、trace 属性或事件，不进入 time series label。
7. Atomic App/Plugin 从注册、版本、激活、停用、健康到回滚均使用稳定身份；热插拔不等于在 tick 线程动态卸载任意代码。

本书建立在 RGS-DTL-004 的 SDK trait、脱敏算法和 JSON schema 之上，不复制其细节；本书补充后端无关的拓扑、运行时、游戏域仪表、Collector、SLO、资源、K3s 适配、发布和验证。

## 2. 逻辑可观测性架构

~~~mermaid
flowchart TB
    subgraph APP["Atomic App / Service process"]
        CODE["业务代码\nGW / RT / PL / EC / MT / GD / AD / EV / WF"]
        FACADE["crates/observability\n统一 façade + redaction + bounded queue"]
        CODE -->|仅使用统一 API| FACADE
    end

    FACADE -->|OTLP traces| CG[Collector Gateway]
    FACADE -->|OTLP application metrics| CG
    FACADE -->|JSON stdout / sanitized events| LOGAGENT[Node log agent]
    EXPORTER[Node / K8s / PG / Cache exporters] -->|Prometheus scrape| PROM[Prometheus-compatible TSDB]
    COLAGENT[Collector Agent / DaemonSet] -->|host/pod/collector metrics| PROM
    CG -->|metrics route| PROM
    CG -->|traces route| TRACE[Trace store adapter]
    LOGAGENT -->|logs route| LOG[Log store adapter]
    PROM --> G[Grafana + Alerting]
    TRACE --> G
    LOG --> G
    GAME[Game events\nmatch/session/degradation/audit] -->|correlation keys| LOG
    G -->|trace_id / request_id / event_id| TRACE
    G -->|trace_id / player_id / match_id| LOG
    G -->|SLO / recording rules| PROM
~~~

### 2.1 信号选择

| 数据 | 默认路径 | 原因 | 不采用的做法 |
|---|---|---|---|
| 应用 trace | façade → OTLP → Collector Gateway → trace store | 标准传播、批量、采样、跨服务 | 业务代码直连 trace store |
| 应用指标 | façade → OTLP → Collector；Collector 输出 Prometheus-compatible | 应用与服务资源 metadata 一致 | 每个业务模块各自选 metric client |
| 基础设施/平台指标 | exporter → Prometheus scrape | scrape 对节点、K8s、DB、Collector 自监控自然 | 为所有 exporter 重新包一层 OTel |
| 结构化日志 | sanitized JSON stdout → node agent/Collector → log store | K8s stdout 生命周期清晰、失败不阻塞 | 在 tick 内同步发日志网络请求 |
| 游戏运营/状态事件 | façade → sanitized event log 或已有 event role | 需要 player/match/room/event_id 关联与审计 | 用高基数 metric 代替事件 |
| Grafana | 统一查询/联查界面 | 玩家→指标→trace→日志→事件的入口 | 把 dashboard 逻辑写入业务代码 |

物理 trace/log 存储以 adapter 表示；具体产品、索引、保留期和部署方式必须在 ARC-014、GOBS-OPS-002、容量和安全评审后锁定。Prometheus/Grafana 是 RGS-TS-001 的目标基线，不等于当前工作区已有实例。

### 2.2 Collector 拓扑

默认采用两层但允许按规模裁剪：

- Agent：每节点一个 DaemonSet，收集 stdout、节点/容器基础信号和 Collector 自身指标；不为每个 Pod 建 sidecar，避免观测资源随 Pod 数线性放大。
- Gateway：Deployment，接收应用 OTLP，执行 batch、memory limiter、属性过滤、采样、路由、重试和导出。
- 小规模开发环境可省略 Gateway，应用直接发送到 Agent/单实例 Collector；必须保留同样的 façade 契约和背压语义。
- 生产是否使用 tail sampling、是否分区部署 Gateway、是否使用独立 log/trace store，必须由流量、可靠性和 OLU 评审决定。

所有 Collector 队列必须有 max queue、最大 batch、发送超时、重试上限、memory limiter、drop counter 和健康探针。

## 3. 统一 observability façade

### 3.1 计划中的 crate 边界

当前仓库未发现这些目录；以下是进入 53 后的目标布局，不是现状声明：

| crate/模块 | 职责 | 禁止依赖 |
|---|---|---|
| crates/observability-contract | Resource、Correlation、MetricKey、Event、SamplingPolicy、Shutdown 接口 | 具体 exporter、业务域 |
| crates/observability | init、tracing span、metrics、events、logger façade | Grafana、业务 DB、业务消息 |
| crates/observability-redaction | 字段分类、mask/hash/drop、schema 校验 | 业务存储 |
| crates/observability-runtime | tick/window aggregator、bounded buffer、drop accounting | blocking I/O |
| services/* | GW/RT/业务使用 façade | 直接调用 OTel/log backend |
| crates/testkit | fake sink、redaction/property、trace fixture、load hooks | 生产凭证/真实数据 |

### 3.2 初始化契约

逻辑初始化顺序：

1. 读取版本化配置与环境变量，生成 service.name/version/environment。
2. 合并部署资源属性：instance、pod、node、namespace、region、zone、app/plugin/version。
3. 初始化 redaction/schema validator。
4. 设置 W3C Trace Context propagator。
5. 创建 metrics instruments、tracer provider、bounded batch processor 和 sanitized logger。
6. 启动健康与 drop accounting。
7. 注册 graceful shutdown：停止接收新工作，flush 至 deadline，超时计数后退出。

逻辑 API：

~~~text
init_observability(resource, config) -> ObservabilityHandle
request_span(handle, operation_class, parent_context) -> SpanGuard
record_metric(handle, instrument, finite_attributes, value)
emit_event(handle, event_type, correlation, sanitized_fields)
set_sampling_policy(handle, versioned_policy)
health_snapshot(handle) -> TelemetryHealth
shutdown_observability(handle, deadline)
~~~

所有 API 的实现必须保证：

- 初始化失败可选择“最小 stderr fallback + 业务继续/进程拒绝启动”两类明确策略，不能静默。
- 业务路径不等待 exporter；超时、queue full、导出失败记录 drop reason。
- 不允许业务代码自定义任意 label 名称或未经 schema 的字段。
- plugin/App 的 app_name、plugin_name、version 属于资源/有限属性；不能把动态用户输入放入 label。

### 3.3 Atomic App/Plugin 热插拔边界

可热切换的是版本化配置和受控 instrumentation provider；不可在正在执行的 Scene Actor/tick 上动态卸载包含任意状态的代码。一次 App/Plugin 生命周期至少产生：

registered → validated → activated → healthy → draining → deactivated → rolled_back/failed。

每个状态事件带 app_id、plugin_id、version、instance、operation_id、trace_id（若有）、result、reason 和 impact_bucket。指标只使用 plugin_name/version 这类有限维度，实例和操作详情进入日志/trace。

这样既保持原子 App 可独立升级/回滚，也避免“观测插件热拔插”引入不可控的 Rust 动态库 ABI、tick 竞争或指标 namespace 漂移。

## 4. 埋点位置设计

### 4.1 请求和跨服务边界

| 边界 | 建议埋点 | 主要关联 |
|---|---|---|
| Gateway ingress | connection accept、TLS/QUIC handshake、auth、route、limit、drain | trace_id、request_id、session_id、result |
| HTTP/gRPC server | server span、status、duration、payload size bucket | service、operation_class、error_type |
| HTTP/gRPC client | client span、peer service、timeout/retry | dependency、retry_class |
| Runtime ingress | input validation/order/duplicate/reject、ACK | session_id/player_id 在日志，transport/result 在指标 |
| Session state machine | state transition、heartbeat、reconnect、bind/unbind | session event + connection active |
| Match/room | allocation、queue、assignment、start/end、result | match/room 进日志/trace |
| Repository/DB | pool acquire、transaction、query class、commit/rollback | db_role、operation_class、duration |
| Outbox/event | insert、publish、consume、ack、retry、dead letter | event_id/workflow_id 在日志/trace link |
| Workflow | transition、wait、compensation、terminal state | workflow_id + trace link |
| Admin/ClusterOps | command、RBAC、approval、audit、rollback | operator_id、operation_id |

### 4.2 游戏 tick 与同步

RT 采用固定窗口聚合器：

- 50ms tick 由 runtime 记录轻量计数器/本地时间；每 1 秒或固定窗口输出 duration、lag、overrun、phase、entity、mailbox 的聚合值。
- 指标按 p50/p95/p99/max 和有限 bucket 输出；scene_id、Actor id、player_id 只进入异常诊断快照。
- slow tick 阈值必须从 NFR-PE-002/003 和负载分布推导；slow tick 记录最近窗口的 phase distribution、entity_count、mailbox_depth、drop/degrade reason。
- 同一个 tick 内不创建 child span；若需要追踪一次战斗/场景转移，使用请求边界或事件边界的 span/link。
- QUIC Datagram 仅记录连接级、协议级、方向级计数和分桶 RTT/丢弃；完整 trace_id 不重复写入每个 datagram。

## 5. 指标基本设计

### 5.1 指标族

| 族 | 代表指标 | 默认标签 |
|---|---|---|
| Request | rgs_request_duration_ms、rgs_request_total、rgs_request_errors_total | service、operation_class、transport、result、error_type |
| Runtime | rgs_scene_tick_duration_ms、rgs_scene_tick_lag_ms、rgs_scene_tick_overrun_total | runtime_pool、scene_bucket、phase |
| Actor | rgs_actor_count、rgs_scene_entity_count、rgs_scene_mailbox_depth | runtime_pool、capacity_bucket、queue |
| Player | rgs_session_transitions_total、rgs_connection_active、rgs_reconnect_duration_ms | transport、state、result |
| Network | rgs_quic_rtt_ms、rgs_quic_bytes_total、rgs_quic_packets_dropped_total、rgs_input_ack_duration_ms | protocol、direction、result、reason |
| Game | rgs_match_duration_ms、rgs_room_active、rgs_match_result_total | mode、result、region_bucket |
| Dependency | rgs_dependency_duration_ms、rgs_dependency_errors_total、rgs_db_pool_saturation_ratio | dependency、operation_class、db_role、result |
| Async | rgs_outbox_lag_seconds、rgs_event_consumer_delay_seconds、rgs_workflow_retry_total | event_role、consumer_group、retry_class |
| Telemetry | rgs_observability_queue_depth、rgs_observability_export_dropped_total | signal、exporter_role、result |
| Plugin | rgs_plugin_activation_total、rgs_plugin_active | plugin_name、plugin_version、state、result |
| ClusterOps | rgs_pfau_state_duration_ms、rgs_clusterops_ack_duration_ms、rgs_clusterops_ack_missing_total、rgs_clusterops_occ_conflict_total、rgs_clusterops_fencing_conflict_total、rgs_cem_delay_seconds、rgs_dlq_depth | operation_class、result、state、reason、pool、queue |

### 5.2 高基数与查询替代

禁止把 player_id、session_id、scene_id、room_id、match_id、request_id、trace_id、span_id、event_id、workflow_id 放入指标标签。

需要个体分析时：

1. 从 dashboard 的有限 bucket 和 SLO 时间窗口确定异常范围。
2. 通过 trace_id/request_id/match_id/player_id 进入日志或 trace 查询。
3. 通过 event_id/workflow_id 进入 Outbox/event/workflow 记录。
4. 对长期产品分析另建经批准的数据分析管线，不混用故障诊断指标。

### 5.3 唯一指标注册表

RGS-BAS-004 的指标名是规范基线；RGS-OPS-001 中出现的其他示例名不得直接用于 dashboard 或告警，除非登记为显式 alias，并注明 owner、转换关系、兼容期和测试。CI 必须阻断未登记的指标名，避免同一含义出现 rgs_scene_tick_duration_ms 与 rgs_tick_duration_p99_ms 两套口径。

ClusterOps/PFAU/CEM/DLQ 指标沿用 RGS-DTL-031 的控制面语义；operation_id、request_id、operator_id 和事件详情只进日志/trace，不能进入 label。

## 6. 日志与事件基本设计

日志输出为单行 JSON，至少包含：

timestamp、level、service.name、service.version、deployment.environment.name、instance、trace_id、span_id、request_id（若有）、message、event_type（若为事件）、result、reason。

业务字段按上下文添加：player_id、character_id、scene_id、match_id、room_id、guild_id、event_id、workflow_id、operator_id。写入前由 redaction layer 根据字段分类处理，禁止各业务模块私自关闭脱敏。

事件类别最少包括：

- session.connected/authenticated/bound/reconnected/disconnected；
- match.queued/assigned/started/completed/failed；
- room.created/closed/full；
- runtime.tick_slow/degraded/drain；
- dependency.timeout/retry/circuit_open；
- outbox.published/consumer_delayed/dead_letter；
- plugin.registered/activated/healthy/draining/deactivated/rolled_back；
- admin.command/audit/rollback。

普通日志、诊断事件和 OPERATION_AUDIT 分离存储或分离访问策略；审计不能被低等级日志采样掉。既有保留基线为：原始指标 15 天、聚合指标 400 天、行为日志 400 天、审计日志 3 年；改变这些值必须经过数据分类、安全、成本和恢复评审。

## 7. Trace 基本设计

### 7.1 传播

- Client→GW：连接建立时可携带 short trace prefix 或请求上下文；高频 datagram 不重复携带完整 trace_id。
- GW→RT/services：内部 HTTP/gRPC 使用 W3C Trace Context。
- services→PG：DB span 与请求 trace 关联；Outbox 行包含可检索 trace_id/event_id。
- Outbox→event→consumer：事件 header 带 trace_id 或 trace link；consumer 新建处理 span。
- event→workflow：workflow_id 与 trace_id 双向索引，避免把异步长等待误画成一个未结束的同步 span。

### 7.2 Span 边界、命名与采样

span 名称使用 context.verb，例如 gateway.authenticate、rt.scene_transfer、ec.commit_transaction、outbox.publish、workflow.compensate。

正常成功路径可采样；错误、超时、拒绝、降级、GM/high-risk、背压、采样策略变更和插件回滚强制捕获。采样率由 PH-4/PH-7 负载和存储预算决定；PH-1～PH-3 若流量很低可暂时 100%，但必须以环境配置和成本观测为依据。

## 8. Tokio runtime 与 profiling

生产只输出轻量 runtime metrics：worker busy/idle、runtime lag、task poll duration bucket、blocking pool saturation、queue depth、spawn/drop、panic/restart。指标仍通过 façade 管理，不让每个 Tokio task 产生高基数 span。

Tokio Console 需要 tokio_unstable、Tokio tracing feature 和 console subscriber 初始化，适合开发/预发布的短时诊断；它不是生产默认监控后端。生产如需启用必须：

- 单独的构建 profile 和 NetworkPolicy；
- 明确采集期限、权限、资源上限和关闭方式；
- 与玩家体验压测对比，证明不会改变 tick/ACK 目标；
- 禁止在生产长期以调试配置运行。

## 9. 依赖和平台观测

### 9.1 PostgreSQL 18.6

PostgreSQL 18.6 是 RGS-TS-001 的目标版本，须在环境门核验。应用侧至少观测 pool acquired/idle/wait、transaction duration、commit/rollback、query class duration、timeout、deadlock、replica lag（如有）。数据库侧使用已批准的 exporter/scrape，避免在每条 SQL 日志记录参数和个人数据。

### 9.2 Cache

按 cache role 观测 hit/miss、latency、connection、eviction、memory、cluster health、timeout/retry；key、player_id 和 session_id 不进入指标 label。cache 丢失时以重建、降级和恢复事件关联玩家影响。

### 9.3 Outbox/Event/Workflow

需求层使用 generic event infrastructure，ARC-014 默认 Outbox polling；RGS-TS-001 的 NATS JetStream 是目标选型。设计只依赖 event_role、consumer_group、partition/ordering role、lag、retry 和 dead-letter 语义，不把 Kafka 当作事实。

### 9.4 K3s/Kubernetes

使用 Kubernetes-compatible Deployment/StatefulSet/DaemonSet/Service/ServiceMonitor-compatible 对象，K3s 适配通过 GOBS-GATE-02 验证。平台侧收集节点、Pod、容器重启、调度、HPA、Ingress、网络策略、磁盘/存储和 API server 健康；不能依赖未确认的 K3s 专有插件。

每个 Atomic App 必须实现版本化 health contract，至少区分 live、ready、degraded、draining、failed；readiness 不能因为 trace store 暂时不可用而把游戏核心误标为不可用。ClusterOps/控制面可以另行定义其 ACK/PFAU 健康，但字段和状态转换必须可被 façade、平台 probe 和 dashboard 同时识别。

## 10. Collector、背压与容错

| 阶段 | 策略 | 可见性 |
|---|---|---|
| façade queue 满 | 丢弃正常低优先级 telemetry；保留 error/forced capture 配额 | queue_depth、dropped_total、drop_reason |
| exporter 暂时失败 | bounded retry + exponential backoff + timeout | export_errors、retry_total、last_success |
| gateway memory 高 | memory limiter、降低 batch、切换采样、拒绝低优先级 | memory、sampling_version、degrade event |
| trace store 不可用 | metrics/logs 仍独立流转；trace 进入受限缓冲后丢弃 | trace_drop、pipeline health |
| log store 不可用 | stdout 保留到平台生命周期；不在业务线程等待 | agent_drop、disk/backpressure |
| shutdown/drain | 停止新工作，按 deadline flush | shutdown_duration、flush_timeout |

观测系统不是游戏业务的强同步依赖。只有当需求明确要求“拒绝启动”时才把初始化失败作为启动门；运行中 exporter 失败不得触发玩家请求同步阻塞。

## 11. SLO、仪表盘与告警

### 11.1 仪表盘层级

| 编号 | Dashboard | 内容 |
|---|---|---|
| 00 | Player Experience | login-to-scene、ACK、reconnect、disconnect、match、settlement、error budget |
| 10 | Runtime | tick、lag、overrun、Actor、entity、mailbox、sync、degrade、drain |
| 20 | Services | GW/API/PL/EC/MT/GD/AD 请求黄金信号、版本与实例 |
| 30 | Dependencies | PG、cache、Outbox、event、workflow、外部服务 |
| 40 | Platform | K3s/K8s workload、节点、网络、存储、Ingress、HPA |
| 50 | Observability | Collector received/exported/dropped、queue、memory、storage、sampling |

### 11.2 告警原则

告警使用 SLO burn rate、持续时间、影响范围和依赖证据；禁止仅用 CPU 作为玩家故障告警。初始规则只定义形态，阈值由基线/负载试验填写：

- Player login/scene-entry SLO burn；
- input ACK 或 disconnect/reconnect burn；
- tick p99 + overrun + affected runtime pool；
- Outbox/event delay + consumer errors；
- PG pool saturation/timeout/deadlock；
- cache timeout/cluster degradation；
- Collector queue/drop/export failure；
- plugin version unhealthy/rollback；
- K3s workload crashloop/unschedulable/storage pressure。

每条告警包含 owner、severity、dashboard、runbook、query window、suppression/maintenance、rollback/degrade action 和恢复条件。

## 12. 安全与数据治理

- OTLP、scrape、日志和 trace 管道只在受控网络内开放；Collector 接收端不得公开暴露。
- 使用 TLS/mTLS、K3s/K8s NetworkPolicy、RBAC、Secret 最小权限。
- redaction 在 façade 之前/入口处完成；禁止在 dashboard 或存储端才依赖脱敏。
- password、token、credential、payment secret 永不记录；IP mask；email/phone hash。
- player_id 明文只能在受控日志/trace 查询中出现，metric 禁止；查询、导出和管理员操作进入审计。
- retention 分为 telemetry health、SLO metrics、normal traces、forced traces、diagnostic logs、audit；具体期限由数据分类和法律/运营规则评审。

## 13. 容量、资源与成本模型

在无实际代码和流量基线时不虚构固定 CPU/RAM 数值。按以下公式在 PH-0/PH-7 测得：

- 指标样本量 = series_count × scrape_or_export_frequency × retention。
- trace 写入量 = request_rate × average_spans_per_request × sample_ratio × average_span_bytes。
- 日志写入量 = event_rate × average_log_bytes × retention。
- Collector memory = queue_items × average_item_bytes × signal_factor + batch/retry overhead。
- 网络带宽 = metrics + trace + logs + game-event payload + retry overhead。

资源预算必须分别给游戏进程、Collector Agent、Collector Gateway、Prometheus-compatible store、trace/log store，并提供 headroom、故障时的 backpressure、扩容和回滚阈值。观测资源预算不能以牺牲 RT tick 和玩家体验为代价。OLU 统计必须统一时间口径（人·天/周或 OLU/月）后再与 SRE ≤2 约束比较，不能混用两个分母。

## 14. 实现文件与影响分类

当前无实现文件。通过门禁后，预计新增/修改对象如下：

| 影响 | 对象 | 说明 |
|---|---|---|
| 高 | crates/observability* | 统一 contract、facade、redaction、runtime aggregator |
| 高 | services/gateway、services/runtime | 边界 middleware、session、network、tick window |
| 中 | services/player/economy/match/social/admin | 请求、DB、Outbox、业务事件 |
| 中 | crates/rgs-testkit、CI | 负例、cardinality、redaction、trace fixture、deny checks |
| 高 | deploy/helm 或 k3s manifests | Collector、ServiceMonitor-compatible、NetworkPolicy、RBAC、resource |
| 中 | dashboards/alerts/runbooks | Grafana、SLO、burn-rate、故障操作 |
| 低至中 | 文档/registry | traceability、版本、审批和变更记录 |

所有工作区路径是计划对象；创建前须通过 GOBS-GATE-01。

## 15. 设计验收矩阵

| 设计检查 | 通过条件 |
|---|---|
| façade boundary | 业务模块无直接 backend import；plugin/App 只使用稳定 contract |
| hot path | tick、packet、entity 无同步 I/O、无 per-item span/log |
| cardinality | 非法 label 的负例 CI 失败；有限枚举有清单 |
| propagation | login/match/settlement/event/workflow 样例可用 trace_id/event_id 联查 |
| failure isolation | Collector、trace store、log store 故障不阻塞游戏核心 |
| K3s adaptation | 节点、storage、network、RBAC、workload 和 health 通过适配测试 |
| capacity | telemetry on/off 与故障模式下 PE NFR、CPU/RAM/network、queue/drop 有数据 |
| security | secret/PII/非法标签负例通过；审计可检索 |
| rollback | Collector/config/plugin/backend 能独立灰度和回滚 |
| health contract | live/ready/degraded/draining/failed 的语义、probe 与 dashboard 一致 |
| metric registry | 指标名、alias、owner、labels、bucket、兼容期可自动检查 |

## 16. 依赖与官方参考

本书使用的库行为依据：

- OpenTelemetry Rust：Resource metadata、W3C TraceContext、BatchSpanProcessor、force_flush/shutdown 和绑定 instrument 的官方文档。
- Tokio Console：官方文档对 tokio_unstable、tracing feature 和 console subscriber 的要求。
- Actix Web、Rust stable、PostgreSQL 18.6 的版本目标以 RGS-TS-001 为准；版本是否真正落地必须由 GOBS-GATE-01/GATE-03 核验。

## 17. Revision 1 → Revision 2 自审

Revision 1 发现：

1. 只有 OTel 视角会漏掉 Prometheus scrape、stdout 日志、游戏事件和 K3s 自监控。
2. 只有后端链路会漏掉 tick、session、QUIC、match/room 和玩家体验。
3. 只谈 dashboard 会遗漏背压、资源、retention、Security、rollback 和运维负荷。
4. 直接写 K3s/Kafka/Loki/Tempo 会把未证实选项写成事实。

Revision 2 修正：

- 使用混合采集：应用 OTLP、基础设施 scrape、结构化 stdout、游戏事件。
- 明确 façade/crate 边界和 Atomic App/Plugin 生命周期，从构造初期就保证可插拔。
- 增加 tick/网络/业务/依赖/Collector 的具体埋点点位和禁止项。
- 将阈值、样本率、存储和资源全部绑定到基线/容量评审。
- 将 K3s、物理后端、事件产品保留为经过门禁的适配/选择，不假定已部署。

**基本设计结论：逻辑拓扑和接入边界已完成；物理安装、代码实现、阈值和版本锁定仍待门禁与评审。**
