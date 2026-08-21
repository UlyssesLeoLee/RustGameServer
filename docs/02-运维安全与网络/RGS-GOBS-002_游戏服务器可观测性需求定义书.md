# 游戏服务器可观测性需求定义书

**RGS-GOBS-002**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GOBS-002 |
| 版本 | 0.1 |
| 状态 | 草案，待技术评审、容量评审和责任人签字 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 上位依据 | RGS-REQ-001、RGS-REQ-008、RGS-BAS-001、RGS-BAS-004、RGS-DTL-004 |
| 适用对象 | GW、RT、SY、PL、EC、MT、GD、AD、EV、WF、ClusterOps、Atomic App/Plugin |
| ID 规则 | GOBS-REQ/NFR/MET/LOG/TRC/ALT/SEC/OPS-xxx |
| 实现状态 | 需求已定义；当前工作区无实现文件，未授权直接上线 |

## 1. 定位与目标

本书把“新增 Grafana”改写为“基于现有游戏服务器架构的全面可观测性增强”。目标不是让每个模块都接入同一种后端，而是让 SRE 可以从玩家体验问题向下追踪到游戏运行时、服务、依赖、集群和基础设施。

优先级固定为：

1. 玩家体验：登录、连接、入场、匹配、战斗、状态同步、结算、重连。
2. 游戏运行时：20Hz tick、Actor、mailbox、AOI、网络发送和过载降级。
3. 服务与中间件：请求、gRPC、PostgreSQL、缓存、Outbox、event、workflow。
4. 平台与基础设施：K3s/Kubernetes、节点、网络、存储、Collector。

本书不把现有设计文档中的组件名当成已部署事实；现状证据以 RGS-GOBS-001 为准。

## 2. 范围与非范围

### 2.1 范围

- 统一 observability façade、Resource metadata、日志/指标/trace 初始化。
- GW、RT、业务服务、数据库、缓存、Outbox/event/workflow 的边界埋点。
- 游戏 tick、网络和 session 生命周期的低侵入聚合指标。
- 结构化日志、脱敏、采样、强制捕获和跨信号关联。
- Grafana 仪表盘、SLO、burn-rate 告警、故障演练和 runbook。
- K3s 适配、Collector/存储资源预算、背压、丢弃和保留策略。
- Atomic App/Plugin 的注册、版本、激活/停用、健康和回滚可观测性。

### 2.2 非范围

- 在尚未通过 GOBS-GATE-01～06 前编写业务源码或安装平台。
- 替换 RGS-DTL-004 已定义的 SDK trait、字段脱敏算法和 JSON schema。
- 直接决定未经过 ARC-014/容量评审的 Kafka、日志平台、trace 平台或独立消息中间件。
- 把面向策划的数据分析、留存、付费转化等产品分析指标混入故障诊断主链路。
- 在游戏 tick 内同步访问数据库、事件总线、Collector 或日志存储。

## 3. 约束与验收基准

| 既有约束 | 本书要求 |
|---|---|
| NFR-PE-001～003 | tick 20Hz/50ms、p99 处理 <25ms、调度误差 p99 <10ms；观测数据不能改变这些目标 |
| NFR-PE-004～012 | 输入 ACK、同步、重连、业务 gRPC、DB、Outbox、登录体验必须可以从指标/trace/日志验证 |
| NFR-PE-013～019 | 100k CCU、节点/场景容量、峰值 3x 和 15 分钟扩容必须有容量和过载信号 |
| CON-007 | tick 路径不得同步调用 DB/message/workflow；观测 SDK 也不得破坏该约束 |
| CON-008 | 所有 telemetry queue/mailbox bounded；导出失败只可降级/丢弃/采样，不可无限堆积 |
| NFR-OP-001～004 | trace 全链路、关联键完整、指标覆盖目标、结构化日志和 PII 脱敏 |
| NFR-OP-005/NFR-OP-008/NFR-OP-010 | SLO/早期预警、15 分钟定位、常态 SRE ≤2 |
| NFR-MI-005 | event、workflow、cache 和 telemetry backend 必须由 abstraction 隔离，允许替换 |
| RGS-BAS-004 既有保留基线 | 原始指标 15 天、聚合指标 400 天、行为日志 400 天、审计日志 3 年；变更需经数据分类/安全/成本评审 |
| RGS-BAS-023 / RGS-DTL-031 | 请求处理顺序、request/operator/approval/version/trace 关联以及 PFAU/ACK/OCC/fencing/CEM/DLQ 观测不得被业务旁路 |

## 4. 功能需求

| ID | 需求 | 验收要点 |
|---|---|---|
| GOBS-REQ-001 | 系统必须为每个进程建立统一 Resource metadata：service.name、service.version、deployment.environment.name、instance、region、zone，并附带 K8s workload 元数据（可用时）。 | 任意信号可按 service/version/environment/instance 过滤；缺失平台字段不能导致进程不可用 |
| GOBS-REQ-002 | 业务代码必须只依赖统一 observability façade，不得直接依赖 Grafana、Prometheus、OTel exporter 或日志后端。 | CI/静态检查能拒绝越过 façade 的调用；替换后端不改业务 API |
| GOBS-REQ-003 | GW→RT→业务服务→DB/Outbox→event/consumer→workflow 的分布式调用必须保留可检索的 trace context 和关联键。 | 以登录、匹配、结算各一条样例完成跨信号检索 |
| GOBS-REQ-004 | 系统必须记录 session、match/room、plugin/App、drain、降级、重试、背压和恢复等状态边界事件。 | 日志/事件可回答“何时、哪个边界、因何变化、影响多少玩家” |
| GOBS-REQ-005 | RT 必须提供 tick 的窗口聚合指标和异常诊断快照，而不是逐 tick/逐实体 trace 或日志。 | 负载下 tick 仍满足 NFR-PE-002；slow tick 能定位到阶段和队列 |
| GOBS-REQ-006 | 系统必须观测 QUIC connection/stream/datagram、HTTP/gRPC、session reconnect、ACK、丢弃、限流和 load shedding。 | 网络问题可与玩家体验和服务依赖错误关联 |
| GOBS-REQ-007 | 系统必须提供面向玩家体验、运行时、服务依赖、平台资源和 observability 自身的 Grafana 仪表盘及 drill-down。 | 15 分钟内从体验 SLO 进入 trace、日志和事件 |
| GOBS-REQ-008 | 系统必须为关键 SLO、错误预算、burn rate、依赖故障和观测管道自身故障提供告警与 runbook 链接。 | 故障演练可触发告警，并能确认恢复和抑制重复告警 |

## 5. 非功能需求

| ID | 需求 | 目标/门槛 |
|---|---|---|
| GOBS-NFR-001 | 高热路径观测开销必须受控。 | 以同一负载的 telemetry on/off 对比，tick p99、输入 ACK p99、吞吐和 CPU 变化必须在 PH-7 由容量评审批准的预算内；未测前不得给出伪精确百分比 |
| GOBS-NFR-002 | tick、packet、entity 路径必须无同步 telemetry I/O。 | 代码评审和压测证明无 DB/message/exporter/blocking logger 调用 |
| GOBS-NFR-003 | 所有 telemetry 队列、batch、retry、buffer 必须有上限和丢弃策略。 | 能看到 queue depth、dropped、retry exhausted；无 unbounded channel |
| GOBS-NFR-004 | 指标标签必须有限、可枚举、可预算。 | 禁止 player_id、session_id、scene_id、room_id、match_id、request_id、trace_id、event_id 作为 metric label |
| GOBS-NFR-005 | 采样必须可配置、可审计，错误、GM/high-risk、降级、拒绝和背压事件可强制捕获。 | 正常流量采样率由 PH-4/PH-7 负载数据确定；未定前只作为环境配置，不宣称最优值 |
| GOBS-NFR-006 | 信号丢失必须可见，且观测故障不能使游戏核心不可用。 | Collector/导出故障时业务仍可运行；丢弃、降级、采样切换有指标和日志 |
| GOBS-NFR-007 | 存储保留、索引和容量必须按查询目标、事件量、采样率和保留期计算。 | 公式、容量压测和成本/OLU 评审通过后锁定保留配置 |
| GOBS-NFR-008 | 日常运维必须适合 SRE ≤2。 | dashboard、告警、runbook、备份恢复和升级步骤可由两人以内常态维护 |

## 6. 指标需求

### 6.1 指标目录

| ID | 指标/指标族 | 最小标签集合 | 用途 |
|---|---|---|---|
| GOBS-MET-001 | rgs_request_duration_ms、rgs_request_total、rgs_request_errors_total | service、operation_class、transport、result、error_type | 服务黄金信号 |
| GOBS-MET-002 | rgs_scene_tick_duration_ms、rgs_scene_tick_overrun_total、rgs_scene_tick_lag_ms | service、runtime_pool、scene_bucket、phase | tick 处理、调度与阶段定位 |
| GOBS-MET-003 | rgs_scene_entity_count、rgs_scene_mailbox_depth、rgs_actor_count | runtime_pool、scene_bucket、capacity_bucket | Scene/Actor 容量和队列 |
| GOBS-MET-004 | rgs_session_transitions_total、rgs_connection_active、rgs_reconnect_duration_ms | service、transport、state、result | session 生命周期 |
| GOBS-MET-005 | rgs_quic_rtt_ms、rgs_quic_bytes_total、rgs_quic_packets_dropped_total、rgs_input_ack_duration_ms | service、transport、direction、result、reason | 网络与玩家交互 |
| GOBS-MET-006 | rgs_match_duration_ms、rgs_room_active、rgs_match_result_total | service、mode、result、region_bucket | 匹配/房间/结算 |
| GOBS-MET-007 | rgs_dependency_duration_ms、rgs_dependency_errors_total、rgs_db_pool_saturation_ratio | service、dependency、operation_class、result、db_role | gRPC/PG/cache 依赖 |
| GOBS-MET-008 | rgs_outbox_lag_seconds、rgs_event_consumer_delay_seconds、rgs_workflow_retry_total | service、event_role、consumer_group、result、retry_class | 异步事实传播 |
| GOBS-MET-009 | rgs_observability_queue_depth、rgs_observability_export_dropped_total、rgs_observability_export_duration_ms、rgs_plugin_activation_total | service、signal、exporter_role、result、plugin_name、version | 观测管道与 Atomic App/Plugin |
| GOBS-MET-010 | rgs_pfau_state_duration_ms、rgs_clusterops_ack_duration_ms、rgs_clusterops_ack_missing_total、rgs_clusterops_occ_conflict_total、rgs_clusterops_fencing_conflict_total、rgs_cem_delay_seconds、rgs_dlq_depth | service、operation_class、result、state、reason、pool、queue | ClusterOps/PFAU/CEM 控制面；个体 operation_id 只进日志/trace |

### 6.2 标签与聚合规则

允许的标签只来自固定集合：service、service_version、environment、region、zone、runtime_pool、operation_class、transport、direction、protocol、dependency、db_role、pool、queue、result、status、error_type、reason、phase、signal、exporter_role、consumer_group、mode、capacity_bucket、scene_bucket、plugin_name、plugin_version。

player_id、character_id、session_id、scene_id、room_id、match_id、guild_id、request_id、trace_id、span_id、event_id、workflow_id 只能进入结构化日志、trace 属性或事件 payload，并须按 RGS-BAS-004/DTL-004 脱敏；不得作为 metric label。

所有 histogram bucket、scene_bucket、capacity_bucket、error_type 和 operation_class 必须在代码审查中列出有限枚举，并由 CI 检查非法 label。

## 7. 日志需求

| ID | 需求 | 约束 |
|---|---|---|
| GOBS-LOG-001 | 日志必须为结构化 JSON，统一包含 timestamp、level、service.name、service.version、environment、trace_id、span_id、message。 | 与 RGS-BAS-004 schema 一致；普通文本仅允许启动失败的最小 fallback |
| GOBS-LOG-002 | 业务关联字段必须按需附加。 | player_id、character_id、request_id、event_id、workflow_id、scene_id、match_id、operator_id 仅在有上下文时写入 |
| GOBS-LOG-003 | 日志必须在统一 façade 处脱敏。 | password、token、credential、payment secret 不得记录；IP mask；email/phone hash；禁止 bypass |
| GOBS-LOG-004 | session、match/room、tick slow、背压、重试、降级、plugin/App 生命周期必须产生可检索事件日志。 | 每类事件有 event_type、result、reason、duration、impact_bucket |
| GOBS-LOG-005 | tick 诊断日志必须是异常快照或固定窗口摘要。 | 不逐 tick、逐 packet、逐 entity 记录；异常快照包含 entity count、mailbox、阶段分布和最近窗口 |
| GOBS-LOG-006 | 审计日志与普通诊断日志必须分离，并遵守既有保留基线。 | operator/audit 使用 RGS-BAS-003/OPERATION_AUDIT 规则；原始指标 15 天、聚合指标 400 天、行为日志 400 天、审计日志 3 年，变更需审批 |

## 8. Trace 需求

| ID | 需求 | 验收要点 |
|---|---|---|
| GOBS-TRC-001 | 服务间使用 W3C Trace Context；GW→RT→业务→DB/Outbox→event/workflow 保持关联。 | 端到端样例可查询完整路径 |
| GOBS-TRC-002 | span 名称使用低基数的 context.verb 形式。 | 不包含 player_id、room_id、动态 URL、SQL 参数或实体 ID |
| GOBS-TRC-003 | 强制 span 边界为请求、gRPC client/server、DB transaction、Outbox publish/consume、workflow transition、关键外部调用。 | 一次关键请求有足够边界但无业务语句噪声 |
| GOBS-TRC-004 | 高频 tick、packet、entity 更新不得创建 span。 | 使用 MET/LOG 聚合和异常快照替代 |
| GOBS-TRC-005 | 错误、超时、拒绝、降级、GM/high-risk 操作和背压事件必须可强制采样。 | 采样切换有配置版本和审计字段 |
| GOBS-TRC-006 | 异步事件和 workflow 使用 trace link，而不是错误地伪造同一个同步 span。 | event header 保存 trace_id；workflow_id 与 trace_id 双向可检索 |

## 9. 告警与仪表盘需求

| ID | 需求 | 最小交付 |
|---|---|---|
| GOBS-ALT-001 | 玩家体验 Overview 必须展示登录、入场、ACK、重连、匹配、结算和 disconnect/error budget。 | Grafana 00 Player Experience |
| GOBS-ALT-002 | Runtime Overview 必须展示 tick p50/p95/p99/max、overrun、lag、entity、mailbox、Actor 和降级。 | Grafana 10 Runtime |
| GOBS-ALT-003 | Service/Dependency Overview 必须展示请求黄金信号、gRPC、PG、cache、Outbox/event/workflow。 | Grafana 20/30 Dependencies |
| GOBS-ALT-004 | Platform/Observability Overview 必须展示 Pod、节点、网络、Collector queue/drop、存储和告警管道。 | Grafana 40/50 Platform |
| GOBS-ALT-005 | 告警必须按 SLO burn rate、持续时间、影响范围和依赖证据设计。 | 禁止仅凭 CPU 单指标触发玩家故障告警 |
| GOBS-ALT-006 | 每个生产告警必须链接 runbook、关联 dashboard、最近 trace/log 查询和回滚/降级动作。 | 演练中可从 alert 进入处理闭环 |

## 10. 安全需求

| ID | 需求 |
|---|---|
| GOBS-SEC-001 | telemetry 出站必须使用最小权限、TLS/mTLS 或受控网络策略；Collector 接收端不可暴露公网。 |
| GOBS-SEC-002 | trace/log 属性必须执行 PII、credential、payment 和 token 脱敏；禁止以“调试模式”绕过。 |
| GOBS-SEC-003 | player_id 明文只允许在访问受控的日志/trace 范围；查询界面必须有 RBAC 和审计。 |
| GOBS-SEC-004 | operator_id、operation_id、plugin activation/deactivation 等运维事件必须进入审计链路。 |
| GOBS-SEC-005 | retention、导出、备份和删除必须遵守数据分类、最小保留和恢复验证策略。 |

## 11. 运维需求

| ID | 需求 | 出口 |
|---|---|---|
| GOBS-OPS-001 | 提供可复用的 crates/observability façade 与初始化契约。 | 业务 crate 不依赖后端 exporter |
| GOBS-OPS-002 | Collector/存储/Exporter 的部署必须有资源、背压、升级、回滚和健康检查。 | K3s 适配门通过 |
| GOBS-OPS-003 | 观测系统必须自监控并暴露 received/exported/dropped/queue/retry/health 指标，并定义统一 health/readiness/liveness/degraded 语义。 | 观测管道和每个 Atomic App 的健康故障本身可告警；响应 schema 版本化 |
| GOBS-OPS-004 | 负载、故障、网络分区、PG primary stop、cache loss、event restart 和 drain 演练必须验证关联链路。 | 对应 RGS-REQ-001 FT-001～FT-006 |
| GOBS-OPS-005 | 版本升级必须经过灰度、兼容、备份、回退和数据恢复验证。 | telemetry backend 与游戏核心可独立回滚 |
| GOBS-OPS-006 | Atomic App/Plugin 必须拥有稳定的 app/plugin/version/instance 标识，激活、停用、健康和回滚可观测。 | 热插拔不会造成指标/trace namespace 漂移 |
| GOBS-OPS-007 | 采样、日志级别、降级和告警抑制的变更必须有配置版本、操作者和审计记录。 | 可回放变更影响并恢复上一版本 |

## 12. 验收场景

| 验收编号 | 场景 | 通过条件 |
|---|---|---|
| AC-GOBS-001 | 登录→入场 | 一条 trace 贯通 GW、PL、PG/缓存、RT；日志可用 request_id/player_id 查询 |
| AC-GOBS-002 | 匹配→房间→tick→结算 | match/room 事件、RT 指标、结算 DB/Outbox 和 workflow 可联查 |
| AC-GOBS-003 | slow tick 与 mailbox 堵塞 | 不产生 per-tick span；生成聚合指标和一条诊断快照；告警链接 runbook |
| AC-GOBS-004 | QUIC 丢包/重连/load shedding | 网络指标、session 事件、玩家体验 SLO 和降级原因一致 |
| AC-GOBS-005 | PG/cache/event 任一依赖故障 | 依赖错误、重试/背压、影响范围和恢复过程可检索 |
| AC-GOBS-006 | Collector/exporter 故障 | 游戏核心继续运行；drop/queue/health 告警可见；恢复后不无限补发 |
| AC-GOBS-007 | plugin/App 激活、停用、回滚 | app/plugin/version 稳定；生命周期事件、指标和 trace 仍可按版本过滤 |
| AC-GOBS-008 | PII/secret 负例 | 日志/trace/metric 不出现 password/token/credential/payment secret 或非法高基数标签 |

## 13. 完整追踪矩阵

矩阵中的 ID 范围表示该范围内每个 ID 均按同一行的映射逐项验收；不代表省略需求。代码模块和 K3s 组件均标为“计划对象”，因为当前工作区尚无对应文件。

| Requirement | Architecture | Code Module（计划） | K3s Component（计划） | Metric / Log / Trace | Dashboard | Alert | Test |
|---|---|---|---|---|---|---|---|
| GOBS-REQ-001～003 | §3/§14 Resource + propagation | crates/observability；gw middleware；service middleware | OTLP receiver；workload labels | resource attrs；W3C trace；structured log | 00/20 | trace gap | AC-001/002；TST-ST-02-024 |
| GOBS-REQ-004～006 | §14 lifecycle/runtime/network | session；match；runtime；quic adapter | GW/RT workload；Service | lifecycle log；tick/net/session metrics | 00/10 | reconnect/load-shed | AC-003/004 |
| GOBS-REQ-007～008 | §16 dashboards/SLO | dashboard-as-code；alert rules | Grafana/Alertmanager-compatible | SLO metrics；alert annotation | 00/10/20/40/50 | burn rate/dependency | AC-005/006 |
| GOBS-NFR-001～004 | §14/§15 hot path/cardinality | bounded aggregator；label validator | pod resource/NetworkPolicy | queue/drop；finite labels | 10/50 | telemetry overload | AC-003/008；TST-UT-02 |
| GOBS-NFR-005～008 | §15 sampling/capacity | sampling config；retention config | Collector gateway/storage | sample/drop/capacity | 50 | storage/collector | AC-006；PH-7 load |
| GOBS-MET-001～003 | §14.2 runtime/service | runtime metrics；request middleware | GW/RT/ServiceMonitor-compatible | request/tick/Actor metrics | 10/20 | tick/request SLO | AC-001/003 |
| GOBS-MET-004～006 | §14.3 player/network/game | session/quic/match adapters | Gateway/Runtime | session/quic/match metrics + events | 00/10 | reconnect/ACK/match | AC-002/004 |
| GOBS-MET-007～010 | §14.4 dependency/telemetry/ClusterOps | db/cache/outbox/collector/clusterops adapters | PG/cache/event/Collector/ClusterOps | dependency/lag/drop/plugin/PFAU/CEM/DLQ metrics | 20/30/40/50 | dependency/lag/drop/control-plane | AC-005/006/007 |
| GOBS-LOG-001～003 | §14.5 logging/security | facade + redaction | stdout collector/agent | JSON log/schema/PII | 50 + log drilldown | redaction failure | AC-008 |
| GOBS-LOG-004～006 | §14.5 events/audit | lifecycle/event/audit adapters | log pipeline/storage | lifecycle/slow-tick/audit log | 00/10/40 | event loss/audit | AC-003/007/008 |
| GOBS-TRC-001～003 | §14.1 tracing | HTTP/gRPC/DB/event middleware | Collector gateway/trace store | W3C spans/links | 00/20 | trace completeness | AC-001/002/005 |
| GOBS-TRC-004～006 | §14.2/14.6 sampling | tick aggregator/sampler | Collector tail-sampling-compatible | no hot span; error force capture | 10/50 | sample/drop | AC-003/006 |
| GOBS-ALT-001～003 | §16 dashboards | dashboards and recording rules | Grafana/Prometheus-compatible | player/runtime/dependency | 00/10/20/30 | SLO burn rate | AC-001～005 |
| GOBS-ALT-004～006 | §16/§17 operations | alert rules/runbook links | node/kube/Collector exporters | platform/observability health | 40/50 | pipeline/platform | AC-006 |
| GOBS-SEC-001～005 | §14.5/§18 security | redaction/RBAC/audit | NetworkPolicy/Secret/RBAC | redaction/security audit | 40/50 | policy violation | AC-008 |
| GOBS-OPS-001～003 | §13/§14 abstraction/health | crates/observability；collector config；health contract | Collector/Exporter/health/Service probe | received/exported/dropped/health state | 40/50 | observability/App health | AC-006/007 |
| GOBS-OPS-004～005 | §17/§19 rollout | testkit/load/chaos/runbook | fault injection/rollback | fault correlation/recovery | all | incident/recovery | FT-001～FT-006 |
| GOBS-OPS-006～007 | §13.3/§18 plugin governance | app registry/plugin lifecycle/config audit | App Deployment/ConfigMap/RBAC | plugin lifecycle/config audit | 00/40/50 | bad version/change | AC-007/008 |

## 14. 统一抽象的需求边界

基本设计必须提供以下逻辑 API 能力，但不在本书绑定具体 exporter：

- init_observability(resource, config)；
- request_span(context, operation_class)；
- record_metric(instrument, finite_attributes, value)；
- emit_event(event_type, correlation, sanitized_fields)；
- set_sampling_policy(versioned_policy)；
- shutdown_observability(deadline)。

API 只能生成有限标签、脱敏日志和有界 buffer；业务服务不得持有 backend client。Atomic App/Plugin 通过稳定的 app/plugin/version 资源属性接入，不得以动态库卸载破坏正在执行的 tick 或 span。

## 15. 需求审查与 Revision 2

Revision 1 自审发现：

1. 如果只写 Grafana/Prometheus，无法解释 RT tick 和玩家体验。
2. 如果把所有 ID 放到 metrics，必然导致 cardinality 爆炸。
3. 如果把 OTel exporter 直接散落到业务代码，Atomic App/Plugin 无法替换。
4. 如果把当前设计对象称为“已部署”，会绕过 53 的入口门禁。

Revision 2 修正：

- 按八类 GOBS ID 建立可验收要求和追踪矩阵。
- 明确 tick/packet/entity 无同步 I/O、无 per-tick/per-packet span。
- 将 player/session/scene/match 等高基数值移至日志、trace 属性或事件。
- 保留 OTel、Prometheus scrape、结构化日志和游戏事件的分工，而非强制单一路径。
- 把 K3s、物理后端、采样率、资源与保留期设置为门禁/容量评审事项。
- 将 plugin/App 生命周期纳入可观测性，保证热插拔构成从最初接入就可追踪。

**需求结论：本书可作为 RGS-GOBS-003 基本设计输入；实现入口仍受 RGS-GOBS-001 §10 和既有 DD Gate 约束。**
