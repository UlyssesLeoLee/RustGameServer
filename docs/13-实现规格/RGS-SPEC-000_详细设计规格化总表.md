# 详细设计到实现规格总表

**RGS-SPEC-000**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-000 |
| 版本 | 0.2 |
| 状态 | 规格包草案，待详细设计评审通过后进入实现 |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 适用范围 | 当前仓库全部 36 份 RGS-DTL 详细设计 |
| 规范真源 | 对应 RGS-DTL 文档；本规格不得与其字段、状态机、错误码、接口和安全约束冲突 |
| 实现边界 | Rust 1.98 stable（用户目标、GA 前不可核验）、Actix Web 4.14.1、PostgreSQL 18.6；实际环境须通过 Gate 核验 |

## 修订历史

| 版本 | 修订日 | 修订者 | 内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 首版：36 份 DTL 与同号 SPEC 一对一映射。 |
| 0.2 | 2026-08-21 | 架构师 | 绑定 RGS-IMPL-001；将 workspace、contracts、错误、Saga、测试与部署工程约定集中引用，避免 SPEC 间平行解释。 |

## 1. 规格化规则

每份 RGS-SPEC-DTL-xxx 是对应 RGS-DTL-xxx 的实现规格，不是重新设计。实现人员必须按以下顺序使用：

1. 先阅读对应 RGS-DTL 的定位、非目标、接口/数据模型、状态机、错误处理和测试章节。
2. 将本规格的实现模块、配置、部署、观测和验收项登记到对应源码/manifest/CI。
3. 若本规格与 DTL 冲突，停止实现，提交 DTL 变更评审；不得在代码中自行解释。
4. 若 DTL 仍有 TBD、待审批或未核验依赖，本规格保持 blocked，不得标记为 Done。
5. 实现目录、依赖方向、CI、错误/序列化、Saga、测试与部署必须遵循 [RGS-IMPL-001](RGS-IMPL-001_实施约定与工程边界.md)；该文件不替代源 DTL。

## 2. 统一实现契约

### 2.1 Cargo 与代码边界

- Cargo workspace 使用显式 members、resolver = 3；根 `Cargo.lock` 入仓。领域逻辑位于 `crates/rgs-{domain}`，部署二进制位于 `services/rgs-{domain}-service`，contracts 按域生成。
- 禁止泛化 `rgs-common`；服务只依赖自己声明的 domain contract、按域 contract、数据库/缓存/事件 adapter 和 observability façade。
- 不允许跨域直接读写其他域数据库；跨域使用既定 API、Outbox/event 或 workflow。
- 每个 Atomic App/Plugin 必须有 app_id、plugin_id、version、manifest、health、lifecycle、rollback 和 owner。
- 观测能力先于业务埋点：crates/observability-contract → crates/observability → redaction/runtime → testkit → service middleware。

### 2.2 API、数据与错误

- API 请求保留 request_id、operator_id、trace_id；写操作遵守 approval_ref、expected_version、幂等键和 fencing/OCC 约束。
- 字段名、枚举、状态机转移、错误码、SQL 列名和 proto 编号以源 DTL 为准；错误同时有稳定符号码和域号段数字码，transport 使用 gRPC canonical status/HTTP problem details。
- 所有外部输入必须经过认证、授权、限流、校验、幂等、脱敏、埋点和审计的既定管道。
- retry 必须有上限、退避、超时、幂等和 dead-letter/人工介入出口；禁止无限重试。
- migration、配置、manifest 和 event schema 必须可回滚或满足 expand-contract；migration 只由 DB owner 执行，禁止跨 DB FK。

### 2.3 可观测性

- 业务代码只使用统一 observability façade，不直接调用裸 OTel、tracing、log、Grafana 或存储后端。
- metrics 使用有限标签；player_id、session_id、scene_id、room_id、match_id、request_id、trace_id、event_id、workflow_id 进入日志/trace/event，不进入 metric label。
- RT tick、QUIC datagram、entity 循环不得同步 telemetry I/O、逐项 span 或逐项日志；使用窗口聚合和异常快照。
- 结构化日志按 RGS-BAS-004/RGS-DTL-004 脱敏；普通日志与 OPERATION_AUDIT 分离。
- 应用 telemetry 走 OTLP；基础设施和 exporter 使用 Prometheus scrape；日志使用 sanitized stdout/agent；游戏状态事件保留业务关联键。

### 2.4 K3s/Kubernetes 与发布

- 使用 Kubernetes-compatible Deployment、StatefulSet、DaemonSet、Service、ConfigMap、Secret、RBAC、NetworkPolicy 和 health probe。
- K3s 特性不得未经 GATE-02 证据直接假定；StorageClass、IngressClass、CNI、ServiceMonitor/PodMonitor 能力必须实测。
- Collector、应用、Atomic App/Plugin、dashboard、alert 和配置必须可独立灰度、回滚和审计。
- readiness、liveness、degraded、draining、failed 语义必须版本化且与 dashboard 一致。

## 3. 统一规格模板

每份子规格固定包含：

| 章节 | 必须落地的内容 |
|---|---|
| 1. 来源与范围 | 对应 DTL、上位需求、非目标、未决门 |
| 2. 实现单元 | crates、services、proto、migration、deploy、CI 文件 |
| 3. 接口与数据 | API、事件、状态机、表、配置、版本兼容 |
| 4. 运行与容错 | timeout、retry、idempotency、backpressure、shutdown、rollback |
| 5. 观测与安全 | metrics、logs、trace、audit、redaction、RBAC、NetworkPolicy |
| 6. 测试 | UT/IT/ST/load/chaos/security/rollback 证据 |
| 7. Definition of Done | 编译、静态检查、测试、部署 dry-run、追踪矩阵和审批 |

## 4. 全部详细设计映射

| DTL | SPEC | 实现主边界 | 状态 |
|---|---|---|---|
| RGS-DTL-001 | [RGS-SPEC-DTL-001](RGS-SPEC-DTL-001_实现规格书.md) | 核心服务、runtime、DB/API contract | 待 DTL/DD 签字 |
| RGS-DTL-002 | [RGS-SPEC-DTL-002](RGS-SPEC-DTL-002_实现规格书.md) | 功能挂载、脚手架、CI | 待 DTL/DD 签字 |
| RGS-DTL-003 | [RGS-SPEC-DTL-003](RGS-SPEC-DTL-003_实现规格书.md) | Ops/GM 控制面、审计、Webhook | 待 DTL/DD 签字 |
| RGS-DTL-004 | [RGS-SPEC-DTL-004](RGS-SPEC-DTL-004_实现规格书.md) | observability façade、redaction、SDK | 待 DTL/DD 签字 |
| RGS-DTL-005 | [RGS-SPEC-DTL-005](RGS-SPEC-DTL-005_实现规格书.md) | Plugin registry、热插拔、生命周期 | 待 DTL/DD 签字 |
| RGS-DTL-006 | [RGS-SPEC-DTL-006](RGS-SPEC-DTL-006_实现规格书.md) | NetworkPolicy、TLS、DDoS/WAF adapter | 待 DTL/DD 签字 |
| RGS-DTL-007 | [RGS-SPEC-DTL-007](RGS-SPEC-DTL-007_实现规格书.md) | DB schema、migration、存储过程规范 | 待 DTL/DD 签字 |
| RGS-DTL-008 | [RGS-SPEC-DTL-008](RGS-SPEC-DTL-008_实现规格书.md) | Client/SDK、同步与批处理接口 | 待 DTL/DD 签字 |
| RGS-DTL-009 | [RGS-SPEC-DTL-009](RGS-SPEC-DTL-009_实现规格书.md) | 治理、配置、变更和合规 | 待 DTL/DD 签字 |
| RGS-DTL-011 | [RGS-SPEC-DTL-011](RGS-SPEC-DTL-011_实现规格书.md) | Agent/service integration | 待 DTL/DD 签字 |
| RGS-DTL-012 | [RGS-SPEC-DTL-012](RGS-SPEC-DTL-012_实现规格书.md) | 测试基础设施、自动化验证 | 待 DTL/DD 签字 |
| RGS-DTL-013 | [RGS-SPEC-DTL-013](RGS-SPEC-DTL-013_实现规格书.md) | 社交/运营服务 | 待 DTL/DD 签字 |
| RGS-DTL-014 | [RGS-SPEC-DTL-014](RGS-SPEC-DTL-014_实现规格书.md) | 社交运营、活动与玩家治理 | 待 DTL/DD 签字 |
| RGS-DTL-015 | [RGS-SPEC-DTL-015](RGS-SPEC-DTL-015_实现规格书.md) | 数据经济/交易服务 | 待 DTL/DD 签字 |
| RGS-DTL-016 | [RGS-SPEC-DTL-016](RGS-SPEC-DTL-016_实现规格书.md) | 数据经济/交易服务 | 待 DTL/DD 签字 |
| RGS-DTL-017 | [RGS-SPEC-DTL-017](RGS-SPEC-DTL-017_实现规格书.md) | 数据分析管线、读取模型 | 待 DTL/DD 签字 |
| RGS-DTL-018 | [RGS-SPEC-DTL-018](RGS-SPEC-DTL-018_实现规格书.md) | 身份、第三方登录、合规 | 待 DTL/DD 签字 |
| RGS-DTL-019 | [RGS-SPEC-DTL-019](RGS-SPEC-DTL-019_实现规格书.md) | 玩家治理、策略与封禁 | 待 DTL/DD 签字 |
| RGS-DTL-020 | [RGS-SPEC-DTL-020](RGS-SPEC-DTL-020_实现规格书.md) | 平台内购、选服、合规 | 待 DTL/DD 签字 |
| RGS-DTL-021 | [RGS-SPEC-DTL-021](RGS-SPEC-DTL-021_实现规格书.md) | 网络拓扑、容灾、数据管线 | 待 DTL/DD 签字 |
| RGS-DTL-022 | [RGS-SPEC-DTL-022](RGS-SPEC-DTL-022_实现规格书.md) | 核心算法、容量与 runtime | 待 DTL/DD 签字 |
| RGS-DTL-023 | [RGS-SPEC-DTL-023](RGS-SPEC-DTL-023_实现规格书.md) | 请求处理管道、前后处理 | 待 DTL/DD 签字 |
| RGS-DTL-024 | [RGS-SPEC-DTL-024](RGS-SPEC-DTL-024_实现规格书.md) | App 集群、部署脚本、DAG | 待 DTL/DD 签字 |
| RGS-DTL-025 | [RGS-SPEC-DTL-025](RGS-SPEC-DTL-025_实现规格书.md) | 社交通信、运营活动 | 待 DTL/DD 签字 |
| RGS-DTL-026 | [RGS-SPEC-DTL-026](RGS-SPEC-DTL-026_实现规格书.md) | 社交/玩家运营服务 | 待 DTL/DD 签字 |
| RGS-DTL-027 | [RGS-SPEC-DTL-027](RGS-SPEC-DTL-027_实现规格书.md) | Client SDK、引擎适配层 | 待 DTL/DD 签字 |
| RGS-DTL-031 | [RGS-SPEC-DTL-031](RGS-SPEC-DTL-031_实现规格书.md) | ClusterOps、CEM、PFAU、GitOps | 待 Q-025/DD 签字 |
| RGS-DTL-032 | [RGS-SPEC-DTL-032](RGS-SPEC-DTL-032_实现规格书.md) | Agent runtime | 待 DTL/DD 签字 |
| RGS-DTL-033 | [RGS-SPEC-DTL-033](RGS-SPEC-DTL-033_实现规格书.md) | Agent runtime | 待 DTL/DD 签字 |
| RGS-DTL-034 | [RGS-SPEC-DTL-034](RGS-SPEC-DTL-034_实现规格书.md) | Agent runtime | 待 DTL/DD 签字 |
| RGS-DTL-035 | [RGS-SPEC-DTL-035](RGS-SPEC-DTL-035_实现规格书.md) | Agent runtime | 待 DTL/DD 签字 |
| RGS-DTL-036 | [RGS-SPEC-DTL-036](RGS-SPEC-DTL-036_实现规格书.md) | Player App、player_db、plugin host | 待 DTL/DD 签字 |
| RGS-DTL-037 | [RGS-SPEC-DTL-037](RGS-SPEC-DTL-037_实现规格书.md) | Economy App、economy_db | 待 DTL/DD 签字 |
| RGS-DTL-038 | [RGS-SPEC-DTL-038](RGS-SPEC-DTL-038_实现规格书.md) | Match App、match_db | 待 DTL/DD 签字 |
| RGS-DTL-039 | [RGS-SPEC-DTL-039](RGS-SPEC-DTL-039_实现规格书.md) | Social App、social_db | 待 DTL/DD 签字 |
| RGS-DTL-040 | [RGS-SPEC-DTL-040](RGS-SPEC-DTL-040_实现规格书.md) | Admin App、admin_db、ClusterOps | 待 DTL/DD 签字 |

## 5. 统一 Definition of Done

一个 SPEC 只有同时满足以下条件才可标记 Done：

- 对应 DTL 的审批状态允许实现；所有 TBD/待人类决策项有已批准处置。
- Cargo workspace 可构建；`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --workspace --locked`、`cargo deny check`、`cargo audit`、`cargo llvm-cov --workspace` 和 schema/proto 检查通过。
- migration、manifest、ConfigMap/Secret、NetworkPolicy、health probe 和 rollback dry-run 通过。
- API/event/error/ID/DB 字段与 DTL 逐项一致；无跨域数据库旁路。
- metrics/logs/traces/审计满足 RGS-GOBS-002/003；高基数和脱敏负例通过。
- UT/IT/ST/load/security/chaos/rollback 证据已归档，并回填 RGS-REQ-004 追踪矩阵。
- 变更有 owner、版本、审批、dashboard、alert、runbook 和恢复路径。

**结论：本索引使 36 份 DTL 都有明确 SPEC 交付物；工程约定的唯一索引为 [RGS-IMPL-001](RGS-IMPL-001_实施约定与工程边界.md)，实现仍受既有 Gate 和各 DTL 审批状态约束。**
