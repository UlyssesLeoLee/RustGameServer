# 实施约定与工程边界（Implementation Conventions）

**RustGameServer — 实施前工程约定**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-001 |
| 版本 | 0.1 |
| 状态 | **技术基线已收敛；待具名 Gate 批准；不构成编码、迁移或部署授权** |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 适用范围 | 53 开发环境构筑及其后的全部 Rust workspace、服务、契约、迁移、CI 与部署制品 |
| 规范真源 | RGS-REQ / RGS-BAS / RGS-DTL / RGS-SPEC；本文件只固化工程实现约定，不重定义业务语义 |
| 关联 | RGS-TS-001 v0.4、RGS-QA-001 v0.9、RGS-PLAN-001 v0.6、RGS-DTL-031 v0.2、RGS-SPEC-000、RGS-IMPL-001 v0.1、RGS-REV-003、RGS-REV-004/005/006、RGS-ENV-001、RGS-HANDOFF-001 v0.1 |
| 资源约束 | **DEC-005（5 域独立 Lead）**：player / economy / match / social / admin 各自配独立 Lead 签字栏；架构师不兼任 player 域 Lead；SRE 不兼任 admin 域 Lead。**5 域独立 Lead 必然突破 NFR-OP-010**（2 SRE ≤ 20 人·天/周），由 PM + SRE Lead 重算编制，详见 RGS-QA-001 v0.8 §9.4 |

> 本文件把实施前 Q-101～Q-405 的技术答案固定为一套可审查约定。`待具名批准`是 Gate 证据状态，**不是技术问题仍未作答**；任何例外必须更新本文件、对应 ADR/DTL/SPEC，并经同级 Gate 批准。

## 修订历史

| 版本 | 修订日 | 修订者 | 内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 首版。收敛 workspace、crate、proto、迁移、错误、Saga、测试、运行时、密钥、部署与可观测性约定。 |

## 1. 适用与优先级

1. 需求、ADR、BAS、DTL 与同号 SPEC 是业务和架构语义真源；本文件不得覆盖其字段、状态机、DB 所有权或非目标。
2. 本文件解决的是实现一致性问题：目录、依赖方向、错误边界、测试、CI 和部署做法。
3. 未完成 `G-CODE-01`～`G-CODE-07` 时，只可修订文档、验证环境和测试设计；不得创建业务 Rust 代码、SQL migration、Helm/Kubernetes 生产制品。
4. 任何新 crate、proto、migration、Feature 或部署单元必须登记其源 DTL、SPEC、owner、验收项与回滚路径。

## 2. Phase 0：workspace、边界与工具链

### 2.1 Q-101～106：目录、crate、协议与迁移

| Q | 已固定约定 | 强制规则 |
|---|---|---|
| Q-101 | 仓库根为 virtual Cargo workspace | 根 `Cargo.toml` 仅含 `[workspace]`；显式 `members`，`resolver = "3"`；`proto/`、`deploy/`、`docs/` 不是 Cargo member。 |
| Q-102 | 不建立泛化 `rgs-common` | 禁止把 logging/config/error/time 汇入万能 crate。跨服务共用能力必须职责单一、最小依赖且有 owner；业务错误留在领域。 |
| Q-103 | 每域以业务库和部署二进制分离 | `crates/rgs-{domain}` 是领域逻辑库；`services/rgs-{domain}-service` 是独立可部署 bin。禁止把部署、DB 连接或 HTTP handler 放入领域库。 |
| Q-104 | ClusterOps 是 admin 限界上下文内的独立控制面服务 | 使用 `crates/rgs-cluster-ops` 与 `services/rgs-cluster-ops-service`；它不替代五域 App。COC UI 仅经 AdminService 写入，ClusterOps 不编排业务 Saga。 |
| Q-105 | proto 按域与版本隔离 | 路径为 `proto/rgs/{player,economy,match,social,admin,cluster_ops}/v1/*.proto`。生成结果按域进入 `rgs-contracts-{domain}`；`common.proto` 只保留真正跨域的最小类型。 |
| Q-106 | migration 按 DB owner 隔离 | 路径为 `crates/rgs-{domain}/migrations/YYYYMMDDHHMMSS_description.sql`；仅可改本域 DB、不得有跨 DB FK。`admin_db` 由 admin 的单一 migration runner 执行，ClusterOps 不另建 DB 或并行迁移器。 |

首版目录固定如下；任何新增共享 crate 都必须证明其至少被两个已冻结边界使用，且不引入反向依赖：

```text
Cargo.toml                         # virtual workspace / resolver = "3"
Cargo.lock                         # 唯一、入仓
rust-toolchain.toml                # 仅在可用 stable 版本被 CI 核验后写入
proto/rgs/{domain}/v1/*.proto
crates/rgs-{player,economy,match,social,admin}/
crates/rgs-cluster-ops/
crates/rgs-contracts-{domain}/
crates/rgs-testkit/
services/rgs-{player,economy,match,social,admin}-service/
services/rgs-cluster-ops-service/
deploy/{charts,cluster-manifest}/
```

新 Atomic App 的脚手架固定使用 `cargo-generate` 模板：模板创建上述领域库、service bin、域版本化 proto、迁移目录、测试夹具和 Helm values；共享 Helm 能力以 library chart 提供。内部 CLI 不是并列候选，仅当模板无法表达重复的受控操作时，才可经 ADR 新增。

### 2.2 Q-107～108：版本、锁定与 CI

| Q | 已固定约定 | Gate / 例外 |
|---|---|---|
| Q-107 | CI 必跑 `cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --workspace --locked`、`cargo deny check`、`cargo audit`、`cargo llvm-cov --workspace`、proto/schema 检查、migration 前进/回退演练、Helm lint/render/dry-run。 | 不全局开启 `clippy::pedantic`；只逐条纳入经 review 证明有价值的 pedantic lint。覆盖率从第一条可运行切片起度量，达到 QA 阈值后才作为合并 Gate。 |
| Q-108 | 目标为 **Rust 1.98 stable**，Edition 2024；应用 workspace 的唯一根 `Cargo.lock` 必须入仓，CI 用 `--locked`。 | 截至本文制定日，官方发布页可核验的 stable 为 1.97.1；Rust 1.98 在正式 GA、可安装并通过全量 CI 前只能是目标，不得伪造为已验证环境。若 1.98 尚不可用，`G-CODE-06` 保持 Open。 |

`Cargo.lock` 不是按 member crate 分别处理：本仓库部署的是一个 workspace 中的应用集合，故只有根锁文件，必须随依赖变更评审并提交。

## 3. Phase 1：跨服务工程约定

| Q | 已固定约定 | 强制规则 |
|---|---|---|
| Q-201 | crate 内使用 `thiserror`；`anyhow` 仅限 bin 组合根、运维任务与测试边界。 | 公共 domain/API 不得暴露 `anyhow::Error`；五类错误为业务、系统、外部依赖、并发与安全。 |
| Q-202 | gRPC/event 用 protobuf + `prost`；HTTP/Admin 用 `serde_json`。 | 暂不采用 `postcard`：没有经 DTL 冻结的内部 IPC 协议前不得增加第三种序列化格式。 |
| Q-203 | 采用 `tracing`、`tracing-subscriber`、`tracing-opentelemetry`、`opentelemetry-otlp`。 | 只在服务启动/关闭边界初始化 provider、resource、propagator、exporter 与 flush/shutdown；业务代码只调用统一 observability façade。 |
| Q-204 | 错误同时有稳定符号码与域号段数字码，如 `PLAYER_NOT_FOUND (1001)`。 | gRPC 使用 canonical status；RGS 详细码放 error detail，HTTP 映射为 problem details。不得以字符串 message 作为机器判断依据。 |
| Q-205 | 跨 DB 一致性采用“单 DB 事务 + 同事务 Outbox + 唯一 Saga 调解者 + inbox/request_id 去重 + 显式补偿”。 | 禁止 2PC/XA；高频货币/道具留在 economy 单库事务；每个 Saga 必须有持久状态、补偿、超时、人工升级和至少三个真实场景测试。此答案须由架构、DBA 与经济 Lead 具名批准后关闭 `G-CODE-04`。 |
| Q-206 | trait 仅用于外部边界；单元测试使用 fake/mockall；集成测试使用 Testcontainers 运行真实 PostgreSQL 与 NATS。 | 禁止为每个内部函数抽象 trait；`rgs-testkit` 只提供 fixture、fake、契约夹具和故障注入工具。 |
| Q-207 | 根 `Cargo.lock` 入仓。 | CI 的 build/test/audit 必须使用 `--locked`；依赖更新只经单独评审 PR。 |

## 4. Phase 2：运行时与安全约定

| Q | 已固定约定 | 强制规则 |
|---|---|---|
| Q-301 | Tokio 1.x multi-thread runtime。 | Actix Web 仅用于 HTTP ingress；tonic 用于内部 gRPC。 |
| Q-302 | 初始使用系统 allocator。 | 禁止默认全局启用 mimalloc；仅在目标平台的压测证明尾延迟或碎片收益后，以单独 ADR/feature 引入。 |
| Q-303 | `reqwest` 仅供外部或管理面出站 HTTP；服务间通信用 tonic/event。 | 不得以 HTTP client 绕过 gRPC/event contract 或直连其他域 DB。 |
| Q-304 | Figment 仅在服务启动边界合成 TOML 与环境覆盖。 | 必须反序列化为类型化配置；domain 不直接读取环境变量。 |
| Q-305 | `secrecy` 保护进程内密钥；Kubernetes Secret 只作交付载体。 | 只读挂载、最小 RBAC、静态加密、轮换与脱敏为必需；禁止日志记录和长期环境变量暴露。 |
| Q-306 | 不引入统一 ULID。 | 延续既定 UUID、BIGINT 与 `player_seq` 映射；新增 ID 类型必须由对应 DTL/SPEC 批准，`request_id` 继续遵循既定 UUID 幂等语义。 |

## 5. Phase 3：部署与运维约定

| Q | 已固定约定 | 强制规则 |
|---|---|---|
| Q-401 | 运行时镜像使用 `distroless/cc-debian12:nonroot`。 | 镜像必须按 digest 固定、以非 root 运行，并保留受控 debug 镜像流程。 |
| Q-402 | 发布制品使用不可变 Git SHA/digest 与 OCI revision/version labels。 | `git describe --dirty` 仅可标记本地开发镜像；dirty tree 不得生成发布制品。 |
| Q-403 | Helm 按独立可部署服务发布，提供共享 library chart。 | 禁止五域 mega chart；每个 release 有独立 values、NetworkPolicy、迁移 Job 与回滚路径。 |
| Q-404 | 无状态服务使用 Argo Rollouts canary。 | PFAU 仍由 ClusterOps 状态机控制；应用 canary 与 Feature/PFAU 不能互相替代。 |
| Q-405 | Prometheus、Grafana、Loki、Tempo 作为观测后端，OTel Collector 为统一出口。 | 应用不得直接依赖 Grafana/Loki；遵守 RGS-DTL-004 与 GOBS 的脱敏、高基数和审计约束。 |

## 6. 尚需具名 Gate 的事项（不是技术未决项）

| Gate | 已有技术答案 | 尚缺证据 / 签署 |
|---|---|---|
| G-CODE-02 / Q-025 | DTL-031 的控制面边界、Cargo 位置、PFAU 状态与安全约束已设计。 | 架构、平台、SRE、DBA 的字段级 DD Review 与项目负责人签署。 |
| G-CODE-03 | all-reachable、K8s 健康事实源、Active-Active + OCC/fencing 已载入 ADR-0052。 | 目标拓扑核验、故障注入计划与风险接受。 |
| G-CODE-04 / Q-003 | 本文 §3 的 Saga/Outbox/补偿约定。 | 架构、DBA、经济 Lead 对真实场景、补偿 SLO 与升级路径具名批准。 |
| G-CODE-05 | 五域 App/DB/插件宿主和 contracts 目录已定义。 | 五域 Lead 的依赖矩阵与 DD Review 签署。 |
| G-CODE-06 | Rust 1.98 stable、Actix Web 4.14.1、PostgreSQL 18.4、CI 命令已定义。 | Rust 1.98 GA 可用性、真实工具链输出、workspace/CI bootstrap 与 PostgreSQL 18.4 演练。 |
| G-CODE-07 | `rgs-testkit` 的职责、测试分层和 CI 要求已定义。 | OLU 重算、QA/SRE 签署与首条测试证据。 |

## 7. 追溯性

| 来源 | 本文件落点 |
|---|---|
| RGS-REQ-006 / RGS-BAS-002 / RGS-DTL-002 | §2 的 workspace、App、迁移、Helm 与挂载边界 |
| RGS-REQ-009 / RGS-BAS-005 / RGS-DTL-005 | §1、§5 的插件宿主、隔离与动态库禁止 |
| RGS-ADR-0015 / RGS-DTL-001 / RGS-DTL-015 | §3 Q-205 的单调解者 Saga、Outbox 与补偿 |
| RGS-ADR-0052 / RGS-DTL-031 | §2 Q-104、§5 Q-404、§6 的 ClusterOps/PFAU 边界 |
| RGS-DTL-004 / RGS-GOBS-003 | §3 Q-203、§5 Q-405 的 observability façade 与 OTel 出口 |
| RGS-PLAN-001 / RGS-SPEC-000 | §1、§6 的 Gate 与 DTL→SPEC→实现证据链 |
