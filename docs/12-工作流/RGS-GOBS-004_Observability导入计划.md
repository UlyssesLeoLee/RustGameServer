# Observability 导入计划

**RGS-GOBS-004**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GOBS-004 |
| 版本 | 0.2 |
| 状态 | 草案，待 RGS-GOBS-001～003 评审与入口签字 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 适用流程 | 日本式 SI 九阶段：基本设计 → 详细设计 → 实装 → 测试 → 移行/发布 → 运维 |
| 先决条件 | RGS-ADR-0052、RGS-DTL-031、RGS-PLAN-001、五域 DTL、RGS-TS-001、RGS-IMPL-001 与既有 Gate |
| 现实边界 | 当前工作区没有 Rust workspace、服务源码、K3s manifest 或实际 telemetry backend |

## 修订历史

| 版本 | 修订日 | 修订内容 |
|---|---|---|
| 0.1 | 2026-08-21 | 初版：以现有游戏服务器架构为对象的可观测性增强导入计划。 |
| 0.2 | 2026-08-21 | 对齐 RGS-IMPL-001、Rust 1.98 stable 的 GA/CI Gate、`rgs-testkit` 命名及 façade 边界。 |

## 1. 计划目标

本计划把可观测性作为现有游戏服务器架构的横切能力导入，而不是孤立安装 Grafana。完成后应能：

玩家体验 → Runtime → Service → Middleware → K3s → Infrastructure

逐层定位，并从 Grafana 的指标进入 trace、结构化日志和游戏事件；同时确保 RT tick、QUIC datagram、Atomic App/Plugin 的原子升级和热插拔不被观测系统破坏。

本计划只在设计与审批完成后进入实现。当前不执行安装、迁移、部署或业务埋点。

## 2. 入口、出口与总体门禁

### 2.1 入口门禁

| 门 | 条件 | 证据 |
|---|---|---|
| GOBS-GATE-01 | 实际 Cargo workspace、crate/service、CI、Rust 1.98 stable（GA 后）和 Actix Web 4.14.1 版本已取得 | 仓库、CI、toolchain 输出；不得以 beta/nightly 或旧版本替代 |
| GOBS-GATE-02 | K3s/K8s 节点、网络、Ingress、存储、RBAC、NetworkPolicy 能力已核验 | 集群只读诊断 |
| GOBS-GATE-03 | PostgreSQL 18.4、cache、event role、workflow 拓扑已核验 | 版本、连接、拓扑和备份证据 |
| GOBS-GATE-04 | GOBS-001～004、RGS-BAS-004、RGS-DTL-004 无冲突且完成签字 | 评审记录 |
| GOBS-GATE-05 | tick/login/match/settlement/CCU 基线和 OLU/容量预算已获得 | PH-0 基线报告 |
| GOBS-GATE-06 | 统一 façade、Atomic App/Plugin manifest contract、测试骨架先于业务埋点 | 设计评审与负例测试 |

### 2.2 共同出口条件

- 任何生产路径都有低基数 metrics、结构化日志、必要 trace 边界和关联键。
- tick/packet/entity 路径无同步 telemetry I/O、无 per-item span/log。
- Collector、存储、配置和 plugin 版本可独立灰度、回滚和自监控。
- 玩家体验 SLO、错误预算、burn-rate、runbook 和故障演练闭环通过。
- 文档 registry、追踪矩阵、CI 检查、部署清单和审批记录一致。

## 3. 分阶段导入路线

### PH-0：证据对齐、基线和决策门

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-000 |
| 主要动作 | 真实仓库/集群/中间件 inventory；清理“设计对象=现状”的误读；确认 K3s、PG 18.4、cache、event role、OTel/Prom/Grafana 目标；统一 metric registry、health contract、retention 和 OLU 统计口径 |
| 交付物 | E0/E1/E2 证据表、服务/依赖清单、玩家体验基线、容量/OLU 初算、保留基线确认、指标/健康契约决策记录 |
| 依赖 | RGS-GOBS-001、RGS-TS-001、RGS-PLAN-001 |
| 入口 | GOBS-GATE-01～03 |
| 出口 | 目标版本、平台能力、event role、物理后端选择范围和未知项均有 owner；未知项不得被写成已完成 |
| 回滚 | 只修改调查/决策文档，不触及运行环境 |

### PH-1：统一契约、crate 骨架和 CI

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-010 |
| 主要动作 | 在 RGS-IMPL-001 定义的虚拟 workspace 内创建 `crates/rgs-observability-contract`、`crates/rgs-observability`、redaction、runtime、`crates/rgs-testkit`；建立 CI、rustfmt/clippy/cargo-deny、非法 label/secret 负例 |
| 交付物 | init_observability、Correlation、finite attributes、bounded queue、shutdown、redaction/schema、fake sink |
| 依赖 | PH-0；RGS-DTL-004 |
| 出口 | 空服务/示例 App 可通过统一 façade 启动、记录、导出、降级和关闭；业务 crate 无 backend import |
| 回滚 | crate 版本回退；不回滚业务数据库或玩家数据 |

### PH-2：Collector、平台和观测自监控

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-020 |
| 主要动作 | 部署 K8s-compatible Collector Agent/Gateway、统一 health contract、memory limiter、batch/retry/drop、Prometheus scrape 和平台 exporter；完成 K3s 适配 |
| 交付物 | Collector config、resource/NetworkPolicy/RBAC、ServiceMonitor-compatible 资源、health probe schema、dashboard 50、pipeline runbook |
| 依赖 | PH-0、PH-1、GOBS-GATE-02 |
| 出口 | received/exported/dropped/queue/retry/health 可见；Collector 故障不阻塞最小业务样例 |
| 回滚 | 删除/回滚 Collector workload 和配置，应用回到最小 stderr/stdout fallback |

### PH-3：GW、RT、session、网络和玩家体验

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-030 |
| 主要动作 | 在 Gateway middleware、连接/session 状态机、QUIC 连接/Stream/Datagram 边界、RT tick supervisor、sync/AOI、drain/load shedding 接入 façade |
| 交付物 | dashboard 00/10；login-to-scene、ACK、reconnect、disconnect、tick、mailbox、entity、overrun 指标；slow tick 诊断快照 |
| 依赖 | PH-1/2；RGS-DTL-031；Player/Match 领域 DTL |
| 出口 | 达到 RGS-REQ-001 NFR-PE-001～007 的观测验收；tick 无 per-tick span/log 和同步导出 |
| 回滚 | 关闭低优先级 metrics/trace，保留错误和强制捕获；可按 App/Plugin 版本回退 |

### PH-4：业务服务、数据库、缓存和异步链路

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-040 |
| 主要动作 | PL/EC/MT/GD/AD 的 HTTP/gRPC、repository/DB pool/transaction、cache、Outbox、event consumer、workflow transition 接入 |
| 交付物 | dashboard 20/30；request/DB/cache/outbox/event/workflow 指标；W3C trace 和异步 trace link；业务事件 |
| 依赖 | 五域 DTL、RGS-REQ-001 ARC-014、PG 18.4/cache/event 实际证据 |
| 出口 | 登录、匹配、结算和故障路径可通过 trace_id/event_id/workflow_id 联查；不假定 Kafka 产品 |
| 回滚 | 按服务/插件版本灰度回退；Outbox/业务数据不因 telemetry 回滚而改变 |

### PH-5：日志安全、SLO、告警和 runbook

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-050 |
| 主要动作 | 固化 JSON schema、PII/secret redaction、审计分流和既有保留基线；建立唯一 metric registry/alias 规则；实现 00/10/20/30/40/50 dashboard、recording rules、burn-rate、alert routing 和 runbook |
| 交付物 | GOBS-REQ-002 §7～§11 全部验收样例；告警 owner/severity/query/rollback/degrade action；保留和 alias 变更审批 |
| 依赖 | PH-2～4；安全评审；RGS-BAS-003/004 |
| 出口 | 任意故障从 dashboard 进入 trace/log/event/runbook；redaction 和高基数负例通过 |
| 回滚 | 告警配置版本回滚；日志级别/采样策略版本回滚，不回滚审计记录 |

### PH-6：负载、故障、混沌和容量

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-060 |
| 主要动作 | telemetry on/off 对比；100/1k/10k/目标 CCU；tick、登录、匹配、结算；runtime kill、PG stop、cache loss、network partition、event restart、drain；验证 retention、metric registry 和 health contract |
| 交付物 | 资源/带宽/存储/series/span/log 量、采样率、保留期、queue/drop、扩容/回滚阈值；FT-001～FT-006 报告 |
| 依赖 | PH-3～5；RGS-TS-001；RGS-TST-UT/IT/ST |
| 出口 | RGS-REQ-001 NFR-PE、NFR-OP、CON-007/008 通过；观察资源预算不侵蚀游戏目标 |
| 回滚 | 关闭非必要 signal、降低采样、停止 trace/log 高成本路线；保持业务 metrics 和 error capture |

### PH-7：灰度、发布与正式运行

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-070 |
| 主要动作 | 先非核心 Atomic App，再 Gateway，再 Runtime，再核心经济/结算；每批次比较玩家体验、资源、drop、SLO |
| 交付物 | canary checklist、change/audit、版本兼容矩阵、rollback rehearsal、值班 handoff |
| 依赖 | PH-6；RGS-BAS-009/ADR-0052/DTL-031 的批准状态 |
| 出口 | 按 App/Plugin version 过滤、激活、drain、回滚可见；生产告警无未认领高等级规则 |
| 回滚 | 单 App/Plugin/Collector/dashboard/config 独立回滚；不得用全量回滚掩盖局部缺陷 |

### PH-8：持续治理

| 项目 | 内容 |
|---|---|
| 工作包 | GOBS-WBS-080 |
| 主要动作 | 每次新 App/Plugin/服务必须登记 telemetry contract、dashboard、alert、runbook、owner、容量和安全字段；季度重算基数和保留 |
| 交付物 | telemetry registry、schema version、compatibility test、月度 SLO/成本/OLU 报告 |
| 依赖 | PH-7 |
| 出口 | 新组件未接入 façade/traceability/CI/health 时无法合并或部署 |
| 回滚 | 版本化 registry 和配置，可回到上一兼容 schema |

## 4. 实装顺序与原子 App/Plugin 构造规则

必须先建立以下不变顺序：

1. contracts → observability façade → redaction/schema → bounded runtime aggregator。
2. App/Plugin manifest → stable app/plugin/version identity → lifecycle/health events。
3. CI cardinality/secret/raw backend checks → fake sink tests → one minimal service。
4. Gateway/Runtime boundaries → domain services → middleware → dashboards/alerts。
5. load/chaos/capacity → canary → production。

禁止以下反向顺序：

- 先让各业务模块直接调用 OTel/Prometheus，再事后补抽象。
- 先创建 Grafana dashboard，再按 dashboard 反推不存在的指标。
- 先引入 Kafka/独立日志平台/动态插件加载，再寻找使用理由。
- 先在 RT tick 中加调试 span，再用采样试图消除性能损失。

Atomic App/Plugin 的最小 manifest telemetry contract：

| 字段 | 要求 |
|---|---|
| app_id/plugin_id/version | 稳定、版本化、有限维度 |
| service.name | 与运行时 Resource 一致 |
| owned operations/events | 列出 operation_class、event_type、依赖角色 |
| metrics | 只登记有限标签、bucket 和 owner |
| traces | 列出入口/出口 span 与 async link |
| logs | schema、敏感字段分类、redaction profile |
| health | readiness、liveness、degraded、draining、rollback |
| dashboard/alert/runbook | 每个生产告警均有 owner 和处理入口 |
| compatibility | schema、collector、backend、config 的兼容范围 |

## 5. 资源、容量和 OLU 计算门

不在计划阶段编造固定资源数字。PH-0 建立公式，PH-6 用数据锁定：

- series = 有限标签组合 × service/version/region/role 组合。
- trace bytes/day = requests/day × spans/request × sample ratio × bytes/span。
- log bytes/day = events/day × bytes/event。
- queue memory = queue items × average item bytes × signal factor。
- network = metrics + traces + logs + retries + replication。
- storage = daily ingest × retention × replication × index factor。

每一项需要：

- 游戏进程 telemetry overhead；
- Collector Agent/Gateway CPU、RAM、网络；
- metric/trace/log storage IOPS、容量、保留和备份；
- 故障/重试/峰值 3x 的 headroom；
- SRE 运维 OLU 与恢复时间；
- 关闭高成本信号、降采样、回退和扩容动作。

未完成上述核算时，GOBS-NFR-001、GOBS-NFR-007、GOBS-NFR-008 只能处于“设计约束”，不能标记“已验收”。

## 6. 测试与证据计划

| 层 | 测试内容 | 证据 |
|---|---|---|
| UT | redaction、schema、非法 label、finite enum、bounded queue、sampling、shutdown | RGS-TST-UT-02 + AC-GOBS-008 |
| IT | Collector OTLP、Prom scrape、stdout agent、PG/cache/event fake、trace propagation、drop/backpressure | RGS-TST-IT-02 + AC-GOBS-001/005/006 |
| ST | K3s/K8s workload、NetworkPolicy、RBAC、Ingress、dashboard/alert、fault inject | RGS-TST-ST-02 + GOBS-GATE-02 |
| Load | CCU、tick、ACK、reconnect、match、settlement、telemetry on/off | NFR-PE-001～019；PH-6 report |
| Chaos | runtime kill、PG primary stop、cache total loss、network partition、event restart、drain | FT-001～FT-006 |
| Security | secret/PII leakage、trace query RBAC、OTLP ingress、audit | GOBS-SEC-001～005 |
| Rollback | app/plugin、Collector、config、dashboard/schema、backend adapter independently | PH-7 rollback rehearsal |

## 7. 风险与处置

| 风险 | 触发 | 处置 | 回滚 |
|---|---|---|---|
| 现实代码与设计不一致 | GOBS-GATE-01 发现缺 crate/边界 | 停止批量埋点，更新现状和 DTL | 回到 PH-0 |
| K3s 能力不足 | 无 storage/ServiceMonitor/Ingress/RBAC | 采用兼容资源或修正平台适配，不宣称已部署 | 回到 PH-2 决策门 |
| telemetry 影响 tick | tick p99/CPU/queue 超门槛 | 降采样、关普通日志、保留错误和聚合指标 | per-App/Plugin 版本回退 |
| cardinality 爆炸 | series 或查询成本超预算 | label validator 阻断；改为日志/trace/event | 回退 schema version |
| Collector 背压 | queue/drop/retry 告警 | 扩容/分流/降采样/保留错误 | 回到最小 pipeline |
| event 产品假定错误 | 实际不是 NATS/Outbox | 保留 event role adapter，按 ARC-014 重评 | 不修改业务 contract |
| 其他 session 改动冲突 | 文档/registry checker 失败 | 以最新批准文档合并，保留用户修改 | 不做 destructive merge |

## 8. 追踪与变更管理

所有阶段交付物必须能从 GOBS-REQ-002 的追踪矩阵反查：

Requirement → Architecture → Code Module → K3s Component → Metric/Log/Trace → Dashboard → Alert → Test。

新增 App/Plugin 时至少新增：

- 一个 manifest telemetry contract；
- 一个 façade integration test；
- 一个 health/lifecycle event；
- 一个 dashboard 查询或复用声明；
- 一个 alert/runbook owner 声明；
- 一条容量/基数/安全负例。

文档变更必须同步 RGS-REQ-005 附件 D、document-registry 和 cross-reference checker；不在未审批的旧版本上直接实现。

## 9. Revision 1 → Revision 2 最终自审

### 9.1 Revision 1 问题

1. 只按“部署 Grafana”排期，遗漏真实架构调查和六层观测。
2. 先写业务埋点，后补统一 crate，无法保证原子 App/Plugin 初始构造正确。
3. 将 K3s、Kafka、Redis、OTel/Prom/Grafana 目标写成事实。
4. 缺少 tick 性能、cardinality、backpressure、retention、rollback 和故障演练的出口。

### 9.2 Revision 2 修正

- PH-0 强制证据/版本/平台/中间件对齐，工作区没有实现文件时不越过事实边界。
- PH-1 把 contract、facade、redaction、bounded runtime 和 CI 放在所有业务埋点前。
- PH-2～PH-5 按信号职责引入 OTLP、Prom scrape、stdout、game events，而非单一后端。
- PH-3 明确 tick/QUIC/session 低侵入边界；PH-6 用负载和故障数据锁定预算。
- PH-7 以 Atomic App/Plugin 独立灰度、健康、drain、回滚为发布单元。
- 全程以 GOBS-REQ-002 追踪矩阵和既有 SI 九阶段 Gate 作为出口条件。
- 把现有指标命名冲突、原始/聚合/行为/审计保留基线、健康检查语义和 OLU 统计口径纳入正式门禁。

## 10. 最终审核结论

| 审核项 | 结论 |
|---|---|
| 现有架构理解 | E1 设计基线已整理；E2/E3 实际运行证据缺失 |
| 需求完整性 | 已覆盖玩家体验、runtime、service、middleware、平台、基础设施、业务事件、安全和运维 |
| 原子化解耦 | façade、contract、manifest、plugin lifecycle 和 backend adapter 先于业务埋点 |
| 热插拔正确性 | 版本化配置/受控 provider 可切换；不在 tick 上动态卸载任意代码 |
| Rust/Actix/PG | 目标版本以 RGS-TS-001 为准；Rust workspace、Actix、PostgreSQL 18.4 仍需环境核验 |
| K3s | 使用 K8s-compatible 设计；K3s 适配门未通过前不得宣称已部署 |
| 实现入口 | GOBS-GATE-01～06 未全部通过前，不进入大规模编程和安装 |

**计划结论：四份 GOBS 文档完成后可进行评审；评审通过、证据补齐和 Gate 全部满足后，按 PH-1→PH-7 开始实现。**
