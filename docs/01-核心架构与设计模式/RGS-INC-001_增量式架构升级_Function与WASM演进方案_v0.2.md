# 增量式架构升级——Serverless Function / WASM Runtime / Scale-to-Zero 演进方案

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-INC-001 |
| 版本 | v0.3（v0.2 + admin COC §X 集成设计增补） |
| 修订 | v0.2 / 2026-08-23 / 架构师 / **勘误**：① §1.4 / §1.5 / §2 现状基线与 K3s 实际部署状态对齐（per `docs/deploy/09-deploy-dev-k3s.log` + `docs/deploy/08-measure-env-setup.log`）② §23 插入 Phase 0.5「5 业务域 K3s 部署基线」硬阻塞（per `docs/deploy/09-deploy-dev-k3s.log` + `docs/deploy/07-no-go-checklist_business_v0.1.md`）<br/>v0.3 / 2026-09-04 / 架构师(Mavis 接手 agent per DEC-008) / **精准升版**：① §5 WASM-CAND-003 `admin.audit.policy` 升 P2 → P0（per Ulysses 2026-09-04 20:03 JST 拍板 + admin 域 Lead RACI 拍板待补签）② 新增 §X《admin COC WASM 集成设计》(9 段: 集成点 file:line / 决策 schema / 集成设计 / Registry SHA-256 / second_review 表 / 7 条护栏 / 反例 / admin 域 Lead 拍板栏 / 已知缺口) ③ X.9 显式列 8 类已知缺口（per 8/26 JST 缺标比错标） ④ 其他 §0-§4 / §6-§27 全文**未**做一致性同步（按精准升版拍板，v0.2 → v0.3 增量升版，非整版替换） |
| 制定日 | 2026-08-23（v0.2 制定日；v0.3 升版日 2026-09-04 JST）|
| 制定者 | 架构师（草案，待 5 域 Lead + SRE + 安全联合评审） |
| 对应架构方针 | ARC-014（中间件导入判定基准）、ARC-021（故障隔离）、ARC-026（OLU 预算 / NFR-OP-010）、ARC-005（session_epoch Single-Writer）、ARC-007（运行时不直访业务 DB）、ARC-008（5 独立 DB 限界上下文） |
| 关联 ADR | ADR-0008（中间件导入判定基准）、ADR-0020（拒绝动态链接库加载，**显式记录 WASM 为未来升级路径**——本方案兑现该 ADR 留口）、ADR-0025（OLU 预算） |
| 关联 TS | RGS-TS-001 §3.7（沙箱脚本引擎——当前为 Rhai 1.x，**未**选 WASM；本方案不动 Rhai，仅在 Rhai 承载能力之外另开 WASM 通道） |
| 决策期限 | PH-4（per ADR-0008 §3 附件 D 登记规则） |
| 适用许可 | Apache-2.0（本仓库） |

> **本文档定位**：基于 RGS-BAS-001 §3.5 / §4.7 / §5 的既有 5 域 + cluster-ops + shared-platform 架构，做一次**增量式**、**可回滚**的升级：把低频、异步、插件化计算逐步迁出常驻 Pod，迁入按需 Function 与 WASM Module。**禁止** Big Bang Rewrite，**禁止**破坏 ARC-005/007/008/021，**禁止**为追求 "全函数化" 引入超重 Serverless 平台。
>
> **任务文与代码的偏差声明**：任务原文多次出现 "Kafka"；扫描 `crates/shared-platform/src/messaging.rs` / `producer.rs` / `consumer.rs` / `outbox_relay.rs` 后确认现有架构使用 **NATS JetStream**（`async_nats = "0.x"`）。本方案按真实代码给出 §12 Event Routing Policy；如未来需引入 Kafka 作为 Tier 1 持久层，应另立 ADR-0055 并经过 ARC-014 三条（既有不可承担 / 必要性 / OLU 申领）联合审查。

---

## 0. 阅读指南

本方案分 6 块：

- **第一块 §1-§6**：现状基线（**已完成**扫描，未编造数据；TBD-MEASURE 严格保留）。
- **第二块 §7-§15**：目标形态 + Function/WASM/Scale-to-Zero/KEDA/Event Routing 基本设计。
- **第三块 §16-§24**：落地设计（Placement / Saga / Registry / 安全 / 隔离 / 观测 / DB 保护 / Cold Start / Benchmark / PoC）。
- **第四块 §25-§27**：迁移 / Rollback / 风险。
- **第五块 §28-§30**：ADR 列表 / 需求追踪矩阵 / §50 九问自答。
- **第六块 附录 A**：§49 第二轮自审（8 角色）。
- **第七块 附录 B**：本文档遵循与偏离的既有 ARC/ADR 索引。

---

# 第一块 现状基线（§二要求）

## 1. 交付物 ①《现有架构扫描报告》

### 1.1 仓库与 crate 拓扑

`Cargo.toml` workspace（`rust-version = 1.98`，`edition = 2021`）：

```
crates/
├── shared-platform/     # 公共服务：tracing/OTel/metrics/mTLS/NATS/Outbox/Saga-helper/RBAC/Retry/Span
├── rgs-hello/           # 53.2 占位：RUST 1.98 编译冒烟
├── rgs-certgen/         # 53.11 占位：自签 CA + 6 域证书（rcgen）
├── rgs-testkit/         # 测试 fixture / mock / pg_test_db
├── player-service/      # PL 域（player_db）
├── economy-service/     # EC 域（economy_db，含 Saga 编排器）
├── match-service/       # MT 域（match_db）
├── social-service/      # GD 域（social_db）
├── admin-service/       # AD 域（admin_db，含 SHA-256 链式审计）
└── cluster-ops/         # SRE 域（cluster_ops_db，feature flag + 集群节点注册）
```

### 1.2 消息总线

- **NATS JetStream**（`async_nats::jetstream`），非 Kafka。
- 生产端 `shared-platform::producer::Producer`（自动重试、JSON envelope 携带 `command_id` / `saga_id` / `actor_id` / `trace_id`）。
- 消费端 `shared_platform::consumer::{ConsumerHandler, deserialize_envelope, process_with_retry}`，max_retries 默认 3 → DLQ subject `rgs.dlq.*`。
- **Outbox 模式**（`shared-platform/src/outbox.rs` + `outbox_relay.rs`）：每个域的 `pg_outbox` 表 + 后台 `OutboxRelay<R>` 轮询；55.17 起状态机加 `in_flight` + `lease_until`（30s）+ `FOR UPDATE SKIP LOCKED`，失败保留 in_flight 等待 lease 过期重试，超过 `max_retries` 转 DLQ。

### 1.3 持久化

- 6 个独立 PostgreSQL 18.6 DB（per ARC-008 + DEC-009 + ADR-0052 cluster_ops_db 独立）：
  `player_db` / `economy_db` / `match_db` / `social_db` / `admin_db` / `cluster_ops_db`
- 每域独立 schema、独立 Service 端口（50051~50056）、独立 `migrations/` 目录（`0001_init` / `0002_outbox` / `0003_outbox_check` 等）。
- dev profile：`docker/compose/docker-compose.yml` 6 个 `postgres:18.6` 容器 + 5 域服务占位（`rust:1.98-slim` 跑 `cargo run --release`）。
- prod 目标：`docs/deploy/01-k8s-manifests/23-postgres-statefulset.yaml` 单实例 PG Deployment + PVC（dev 起点；prod 由 DBA 评估主从）。**当前 K3s 清单 12/24 文件为 NO-GO 占位**（见 `docs/deploy/01-k8s-manifests/_status.md`），实际值待 5 域 Lead 联合校准。

### 1.4 RPC / 通信

- **gRPC**（tonic 0.12）+ **mTLS**（rustls，`RGS_TLS_DIR=/etc/rgs/certs`）。55.26 起 **fail-closed mTLS**：默认强制 mTLS；`RGS_ALLOW_INSECURE_GRPC=1` 显式 opt-out（dev/test only），绕过计数共享给 `SERVER_MTLS_BYPASSED_TOTAL`。
- 客户端通过 `shared_platform::client::{build_secure_channel, build_insecure_channel, build_secure_channel_with_tls}` 构造；重试/超时在 `shared_platform::retry::RetryConfig`。
- **QUIC 边缘**（per ARC-003 双路径 Datagram/Stream）：manifests 01/02/... 已留 `quic` UDP 端口占位，但当前实现未实化连接层。
- **跨域消息**：经过 gRPC（同步确定请求）或 NATS JetStream（异步事件）；**禁止**直连 DB（per ARC-007）。

> **部署状态分档（v0.2 勘误，2026-08-23）**：上述为代码层状态；K3s 部署层状态见 §2 表格"状态分档（v0.2 勘误）"列。
>
> 代码就绪 ✅ / binary 编译通过 ✅ / K3s Pod running ❌ / 流量可达 ❌
>
> 5 业务域 + cluster-ops 的 K8s manifest 全部为 PLACEHOLDER（per `docs/deploy/01-k8s-manifests/_status.md`），`scripts/deploy_dev_k3s.ps1` 仅 apply 了 5 个 PG manifest（namespace/SA/secret/PVC/ConfigMap/StatefulSet/Service），**未 apply 任何业务域 manifest**。Phase 1 之前的 PoC 不应假设 gRPC 互通已就绪。

### 1.5 可观测性

- `shared_platform::tracing_init`：`init_tracing_with_otel` + OpenTelemetry 0.24 + OTLP/Tonic 导出 + tracing-subscriber JSON（per RGS-IMPL-001 §3）。
- `shared_platform::metrics` + `metrics_endpoint`：Prometheus 文本导出 + `scrape_metrics` 端点。
- `shared_platform::grpc_tracing`：gRPC client/server `traceparent` 注入。
- 进程内 atomic：`MTLS_BYPASSED_TOTAL` / `SERVER_MTLS_BYPASSED_TOTAL`（55.26 fail-closed 防线计数器）。
- docker/observability：Prometheus + Grafana（已附 `rgs-services-overview.json` 仪表盘）+ OTel Collector config。
- **未实化**：Loki / Tempo / Service Mesh。

> **部署状态分档（v0.2 勘误，2026-08-23）**：
>
> 代码就绪 ✅ / binary 编译通过 ✅ / K3s OTel Collector Pod ❌ / Prometheus Pod ❌ / Grafana Pod ❌ / 跨服务 trace ❌
>
> `shared-platform` 的 tracing_init + OTel 0.24 SDK + tracing-subscriber JSON + Prometheus exporter + grpc_tracing 均已就绪，但 K3s 上**没有 OTel Collector Deployment**、**没有 Prometheus Deployment**、**没有 Grafana Deployment**。`docker/observability/` 目录下仅有 docker-compose 文件，**未移植到 K3s manifest**。结论：当前不能在 K3s 上验证 NFR-OP-001 "全路径可由一个 trace 追踪"（PH-1 阶段判定标准之一，per RGS-REQ-001 §11.2）。

### 1.6 Saga

- `crates/economy-service/src/saga.rs` + `saga_orchestrator.rs` + `inbox.rs`：
  - 状态机：`Pending` → `Running` → `Compensating` → `Completed` / `Failed` / `Aborted`
  - 步进式执行，每步 persist saga 表（崩溃可恢复）
  - 55.12 起 `ReserveHandler` / `ConfirmHandler` 真实持久化（不再仅 log）—— RGS-REV-007 修复
  - 后台恢复循环：每 30s 扫描 `status IN ('running', 'compensating')` → `resume()`
  - 幂等：`transaction_ledger.idempotency_key` UNIQUE + `command_id` 关联
  - ADR-0015 落地：单一调解者（economy 域）原则
- **结论**：现有 Saga 体系是**域内**的，不跨域；且**未涉及 Function Step**。

### 1.7 安全 / RBAC

- `shared_platform::rbac`：`Role` / `Subject` / `enforce` / `SimpleAuthorizer`
- mTLS（双向）覆盖东西向流量；mTLS 计数器暴露给运维
- 无独立 secret backend（生产需 cert-manager 接入，54.x 后）

### 1.8 CI / CD

- `.github/workflows/rust-ci.yml`：fmt + clippy + test（ubuntu-latest，30 min timeout）
- `.github/workflows/docker-build.yml`、`docs-ci.yml`、`verify-docs-ci.yml`
- 镜像：`Dockerfile` 多阶段（rust:1.98 builder → gcr.io/distroless/cc-debian12 runtime-base；`dev` / `staging` / `prod` 三个 target）；实际启用待 53.7 注释解除 + 57.8 cosign keyless

### 1.9 关键事实

- 现有架构是 **2 SRE ≤ 20 人天/周** 的团队结构（per NFR-OP-010 / ARC-026）；
- **本方案任何新增常态运维面**（新 CRD / 新 Operator / 新组件）必须经 ADR-0008 三条件（既有不可承担 / 必要性 / OLU 申领）+ 附件 D §3 登记，**否则不得批准**。
- ADR-0020 已显式记录 "若将来插件复杂度增长到脚本引擎无法承载，WASM 是首选升级路径" —— 本方案正是兑现该 ADR 留口。

---

## 2. 交付物 ②《现有服务工作负载分类表》

> 字段：Service ID / Name / 职责 / Runtime / 有/无状态 / 协议 / QPS / Latency / 长连接 / CPU / Memory / 依赖 / 存储 / 消息主题 / 部署类型 / 副本策略 / 失败影响。
>
> **未实测**的数值一律标记 `TBD-MEASURE`，不得编造。

| ID | Service | 职责 | 类型（按 §3） | Runtime | 状态 | 协议 | QPS | Latency | 长连接 | CPU req/lim | Mem req/lim | 依赖 | 存储 | 消息主题 | 部署 | 副本策略 | 失败影响 | 状态分档（v0.2 勘误） |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SVC-PL-001 | player-service | 账号 / 角色 / 会话 / 鉴权 | **A** Always-On | distroless Rust 1.98 | 无（状态在 PG） | gRPC mTLS | TBD-MEASURE | TBD-MEASURE | 否（QUIC 边缘在 manifest 留口） | 500m / 2000m TBD-MEASURE | 512Mi / 2Gi TBD-MEASURE | player_db, NATS, cluster_ops（feature flag 读） | player_db（accounts / characters / sessions） | NATS: `rgs.pl.*` (outbox 发布) | Deployment + HPA 2→8 | minReplicas=2（per manifest 注释） | **高**——账号侧不可用，整个 5 域鉴权挂掉 | A=✅ B=✅ C=❌ D=❌ |
| SVC-EC-001 | economy-service | 货币 / 物品 / 账目 / Saga | **A** Always-On | distroless Rust 1.98 | 无 | gRPC mTLS | TBD-MEASURE | TBD-MEASURE | 否 | TBD-MEASURE | TBD-MEASURE | economy_db, NATS, player（session_epoch 校验） | economy_db（accounts / transaction_ledger / reservations / sagas / outbox） | NATS: `rgs.ec.*` | Deployment + HPA | minReplicas≥2 | **极高**——经济事务丢失即永久事实错乱 | A=✅ B=✅ C=❌ D=❌ |
| SVC-MT-001 | match-service | 对局 / 撮合 | **A** Always-On | distroless Rust 1.98 | 无 | gRPC mTLS | TBD-MEASURE | TBD-MEASURE | 否 | TBD-MEASURE | TBD-MEASURE | match_db, NATS, player（character 校验） | match_db（matches / participants / outbox） | NATS: `rgs.mt.*` | Deployment + HPA | minReplicas≥2 | **高**——撮合挂 = 开局失败 | A=✅ B=✅ C=❌ D=❌ |
| SVC-GD-001 | social-service | 好友 / 聊天 / 公会 | **B** Elastic Warm | distroless Rust 1.98 | 无 | gRPC mTLS | TBD-MEASURE | TBD-MEASURE | 否 | TBD-MEASURE | TBD-MEASURE | social_db, NATS | social_db（friends / chats / guilds / outbox） | NATS: `rgs.gd.*` | Deployment + HPA 1→N | minReplicas=1 | **中**——离线不直接丢物品 | A=✅ B=✅ C=❌ D=❌ |
| SVC-AD-001 | admin-service | GM / 审计 / COC | **B** Elastic Warm | distroless Rust 1.98 | 无 | gRPC mTLS | TBD-MEASURE | TBD-MEASURE | 否 | TBD-MEASURE | TBD-MEASURE | admin_db, NATS, cluster_ops | admin_db（admin_users / audit_log（SHA-256 链） / outbox） | NATS: `rgs.ad.*` | Deployment + HPA 1→N | minReplicas=1 | **中**——审计链不断即可容忍短暂 GM 不可用 | A=✅ B=✅ C=❌ D=❌ |
| SVC-CO-001 | cluster-ops | 集群节点 / feature flag | **A** Always-On | distroless Rust 1.98 | 无 | gRPC mTLS | TBD-MEASURE | TBD-MEASURE | 否 | TBD-MEASURE | TBD-MEASURE | cluster_ops_db, NATS, admin（跨域读） | cluster_ops_db（cluster_nodes / feature_flags / outbox） | NATS: `rgs.co.*` | Deployment + Active-Active all-reachable（per ADR-0052 PFAU） | minReplicas≥2 | **极高**——feature flag 不可读 = 全系统回滚能力丢失 | A=✅ B=✅ C=❌ D=❌ |
| SVC-SH-001 | shared-platform | 横切库（无独立 binary） | — | n/a | 库形式 | — | — | — | — | — | — | — | — | — | — | — | — | A=✅ B=✅ C=n/a D=n/a |
| SVC-CG-001 | rgs-certgen | 证书生成工具 | **D** Serverless | rust:1.98-slim dev 阶段 | 无 | CLI | TBD-MEASURE | TBD-MEASURE | 否 | 突发 | 突发 | — | 文件输出 | — | 工具 | 一次性 | **低** | A=✅ B=✅ C=❌ D=❌ |
| SVC-TK-001 | rgs-testkit | 测试工具 | n/a | test-only | 无 | lib | n/a | n/a | 否 | n/a | n/a | PG（pg_test_db） | — | — | test | — | — | A=✅ B=✅ C=n/a D=n/a |
| SVC-HL-001 | rgs-hello | 编译冒烟 | **D** Serverless | rust:1.98-slim | 无 | CLI | 0 | <1s | 否 | 突发 | 突发 | — | — | — | CI | 一次性 | **零** | A=✅ B=✅ C=❌ D=❌ |

> **状态分档图例（v0.2 勘误）**：
> A = 代码就绪 ｜ B = binary 编译通过 ｜ C = K3s Pod running ｜ D = 流量可达。
> 依据：`docs/deploy/09-deploy-dev-k3s.log` + `docs/deploy/08-measure-env-setup.log` + `docs/deploy/01-k8s-manifests/_status.md`（2026-08-23 扫描）。
> 6 业务域 / cluster-ops / rgs-certgen / rgs-hello 全部 A=✅ B=✅，但 K3s manifest 仍为 PLACEHOLDER，C/D 均为 ❌。
> shared-platform / rgs-testkit 为库 / test-only，C/D 为 n/a。
> **K3s 基础组件**（NATS / OTel Collector / Prometheus / Grafana / PostgreSQL）见下表。

**K3s 基础组件（v0.2 勘误附表）**—— `9-deploy-dev-k3s.log` 实际 Pod 状态：

| 组件 | 状态分档（v0.2 勘误） | 备注 |
|---|---|---|
| NATS JetStream | A=❌ B=❌ C=❌ D=❌ | **连代码都未引入**（shared-platform 仅 client + JetStream context，无 server） |
| OTel Collector | A=❌ B=❌ C=❌ D=❌ | K3s 无 manifest；`docker/observability/` 仅有 compose |
| Prometheus | A=❌ B=❌ C=❌ D=❌ | K3s 无 manifest；`docker/observability/` 仅有 compose |
| Grafana | A=❌ B=❌ C=❌ D=❌ | K3s 无 manifest；`docker/observability/` 仅有 compose |
| PostgreSQL 18.6 | A=✅ B=✅ C=✅ D=✅ | K3s pod Running 1/1 + 6 DB 全建（player / economy / match / social / admin / cluster_ops） |

**Service Mesh / 边缘网关 / Redis / 缓存基础设施 / 对象存储 / 事件 Schema Registry**：

| 组件 | 状态 | 说明 |
|---|---|---|
| Service Mesh | **未部署** | ARC-014 / RGS-BAS-002 §9 已声明 "不引入独立服务网格除非经 ARC-014 判定" |
| 边缘 Gateway（QUIC + 长连接） | **未实化** | manifest 01 留 quic UDP 端口，进程未启动 QUIC listener |
| API 网关（HTTP） | **PH-6 引入** | per RGS-BAS-001 §3.1 |
| Redis / 缓存基础设施 | **未引入** | RGS-BAS-001 §3.5 描述存在但未部署 |
| 对象存储 | **未引入** | 待 L4 智能层（Python）数据需求评估 |
| 事件 Schema Registry | **DB 表替代** | `admin_db.event_schema_registry` 表（per RGS-TS-001 §3.6.2） |
| 智能层 L4（Python/LangGraph） | **未实化** | per ADR-0026，CR-011 待批；与本方案关系见 §42 |
| 长期记忆向量存储 | **TBD-MEM-001** | pgvector vs Milvus 未决（per RGS-TS-001 §3.8.4） |

---

## 3. 交付物 ③《实时关键链路分析》

### 3.1 Critical Real-Time Path

按 RGS-BAS-001 §3 / §4.4 / §4.5 + ARC-005 / ARC-007：

```
Client (QUIC 边缘 / 长连接)
    │  ARC-003: Datagram 不可靠路径 + Stream 可靠路径
    ▼
[Gateway]  TBD（未实化；manifest 01 留 quic 端口）
    │  仅 fire-and-forward，不等待下游（per RGS-BAS-001 §4.1.4）
    ▼
[Realtime Service] TBD（未实化；RGS-BAS-001 §3.2 描述存在但未在 crate/ 中找到）
    │  场景 Actor / tick 循环
    ▼
[确定请求 — gRPC mTLS]  per ARC-007
    │
    ├──► player-service.GetCharacter (session_epoch 校验 — ARC-005)
    └──► economy-service.CommitTransaction (OCC + idempotency + 事务内持久化 — ARC-006/009)
              │
              ▼
         PostgreSQL 18.6
              │
              ▼
         outbox 表（事务性消息 — ARC-006）
              │
              ▼
         OutboxRelay → NATS JetStream
              │
              ▼
         下游域 Consumer（异步）
```

**重要事实（per RGS-BAS-001 §1.2 + §3.2）**：

- **Realtime Service 本身当前不在 crate 列表中**。RGS-BAS-001 §3.2 描述 "场景 Actor / tick 循环" 是 PH-5 才实化的目标，crate 中只有 `rgs-hello`（编译冒烟）和 5 个业务域 + cluster-ops + shared-platform。
- 因此当前"实时主路径"实际上等于 **Gateway（TBD）→ player-service / match-service → PostgreSQL**。
- **本文档的"实时"语义**采用两段定义：
  1. **当前已实化**实时主路径：5 业务域 gRPC mTLS（quic 边缘未启）。
  2. **未来 PH-5+** 实时主路径：增加 Runtime / 场景 Actor（per RGS-BAS-001 §3.2）。本方案对两段都不得破坏。

### 3.2 实时主路径保护原则

| 原则 | 出处 | 落地要求 |
|---|---|---|
| 不可插入 Knative Activator | §四 / RGS-INC-001 §16 | realtime 域不部署 KEDA/Knative 路径；其 Pod 由传统 HPA 调 |
| 不可插入 Scale-to-Zero Cold Start | §四 | realtime 域 `minReplicas ≥ 1`（KEDA 不可介入 min=0） |
| 不可走 NATS 同步调用 | §四 / ARC-007 | 实时主路径确定请求只走 gRPC；NATS 仅承担异步事件出域 |
| 不可走 WASM 远程调用 | §四 / §17 | realtime 域内规则逻辑 = Local WASM（in-process）或 Native；禁止 Remote Function |
| 不可走 HTTP Function | §四 | realtime 域前端只接 gRPC 或 QUIC |
| 不可串外部 AI | §四 / RGS-TS-001 §3.8 | L4 智能层（Python）单向感知，不在 realtime 路径 |
| 不可任意 SQL 暴露 | §八 / ARC-008 | realtime 域读 DB 只通过 PlayerService / EconomyService gRPC，不开直读 |
| 不可 Saga 兜底 | ARC-008 + ADR-0015 | realtime 域内不存在跨域 Saga；Sagas 仅在 economy 域内 |

### 3.3 实时主路径上的可观测埋点（必须保留）

- `traceparent` 注入（`shared_platform::grpc_tracing`）—— 任何 Function/Worker 介入必须透传 trace_id
- span 名称规范：`grpc_handler_span` / `service_call_span` / `repository_span` / `saga_orchestrator_span` / `saga_step_span`（`span_helpers.rs`）
- 业务字段：actor_id / saga_id / command_id / request_id / deadline

---

## 4. 交付物 ④《Function 化候选清单》

> 候选原则：低频、可异步、调用频率低、状态可外置、无实时性要求、不在 ARC-007 主路径上。

| 候选 ID | 候选名 | 来源（现有或新增） | 触发方式 | 输入/输出 | 状态外置 | 实时性 | 风险 | 优先 |
|---|---|---|---|---|---|---|---|---|
| FN-CAND-001 | `achievement.calculate` | 现有匹配服务 match-service 中低频路径 | NATS: `rgs.mt.match.completed` 事件订阅 | 输入：match 终局数据；输出：成就进度增量 | economy_db (achievement 表，**新增**) | 低 | 低 | **P0 PoC** |
| FN-CAND-002 | `notification.send` | **新增**（目前未在 crate 中） | NATS: 任意 `rgs.*.*` 事件 + 玩家偏好 | 输入：事件 + player_id；输出：in-app/email/push | notification_db / admin_db（**新增**, 评估是否并入 admin） | 低 | 中（外部依赖：推送/邮件） | P1 |
| FN-CAND-003 | `admin.export` | admin-service 现有 GM 操作中的导出类 | HTTP POST → 异步任务 → 下载链接 | 输入：filter；输出：CSV/Parquet URL | 对象存储（**TBD-MEM-001** 评估后选） | 极低 | 中 | P1 |
| FN-CAND-004 | `replay.process` | **新增**（回放流水线） | NATS: `rgs.mt.replay.requested` | 输入：原始 tick 流；输出：回放索引 | 对象存储 | 极低 | 低 | P1 |
| FN-CAND-005 | `analytics.aggregate` | **新增**（BI / 运营分析） | Cron（每日 03:00 UTC） | 输入：player_db / economy_db 物化视图；输出：聚合结果写 analytics_db（**新增**） | analytics_db | 极低 | 低 | P2 |
| FN-CAND-006 | `audit.chain.finalize` | admin-service 现有 audit_log 链 | NATS: 周期 tick（每 5min）或满 N 条触发 | 输入：未链入块；输出：chain anchor 写入 | admin_db.audit_log | 低 | 低 | P2 |
| FN-CAND-007 | `gacha.draw` | **新增**（抽卡） | HTTP POST（player 域前向） | 输入：pool_id + count；输出：物品结果 | economy_db | **中**（P99<300ms） | 中 | P3（须放 Warm Service 而非 Function） |
| FN-CAND-008 | `ai.classify` | **新增**（L4 智能层调用） | NATS: `rgs.pl.behavior.anomaly` | 输入：玩家行为特征；输出：分类标签 | redis（短期）/pgvector（长期 TBD-MEM-001） | 中 | 高（外部 LLM 依赖） | P3，**且**必须走独立 AI Function Pool（per §42） |
| FN-CAND-009 | `certgen.issue` | rgs-certgen 现有 CLI 工具改造 | HTTP POST（dev/staging 触发） | 输入：domains + validity；输出：certs/ | K8s Secret + PV | 极低 | 低 | P0 PoC（**风险最低、无外部依赖**） |
| FN-CAND-010 | `rgs-hello.smoke` | rgs-hello 改造 | HTTP GET（CI 触发） | 输入：none；输出：`OK` | 无 | 极低 | 零 | P0 PoC（与 FN-CAND-009 配套做"健康探针"） |

**Function 化门槛（不通过则不迁）**：

- 已有清晰 gRPC/HTTP 调用方（无调用方 = 无价值）
- 状态可外置或完全无状态
- P99 latency 容忍 ≥ 500ms
- 单次执行 CPU 峰值 ≤ 2 vCPU / 2GB（受 §24 资源保护限制）
- 不在 ARC-007 实时主路径上
- 不需要 session_epoch（per ARC-005）

---

## 5. 交付物 ⑤《WASM 化候选清单》

> WASM 化原则：频繁执行（≥ 100 QPS 单 function）、小逻辑（< 1MB module）、沙箱化、避免重启宿主、不能跨进程序列化大量数据（per ADR-0020 §3.2 教训：WASM↔宿主数据往返成本高于嵌入式脚本）。
>
> **与 ADR-0020 关系**：ADR-0020 否决的是 "用 WASM 替代 Rhai 沙箱脚本"；**本方案不动 Rhai**（继续承担规则 DSL 角色），只在 Rhai 之外**新增** WASM 通道，承担**与 Rhai 错位**的负载：跨域共享、版本热升级、低频规则热修复。

| 候选 ID | 候选名 | 来源 | 触发 | 输入/输出 | 与 Rhai 关系 | 风险 | 优先 |
|---|---|---|---|---|---|---|---|
| WASM-CAND-001 | `economy.commission.rule` | economy-service 中"手续费 / 抽成"规则 | economy 域本地嵌入 | 数值 + 玩家类型 → 手续费率 | Rhai 仍可替代实现；WASM 提供版本热升级 | 低 | **P0 PoC** |
| WASM-CAND-002 | `match.ranking.elo` | match-service 段位算法 | match 域本地嵌入 | 双人 ELO 数据 → 新 ELO | Rhai | 低 | P1 |
| WASM-CAND-003 | `admin.audit.policy` (v0.3 升 P0) | admin-service COC 策略（封禁/解封规则） | admin 域本地嵌入 | actor + action + context → 决策（Allow / RequireSecondReview / Deny）| Rhai 仍可替代；WASM 提供版本热升级 + 决策可重放 | 中（COC 是高敏，per §X 集成设计 7 条护栏 + WASM in-process only） | **P0 PoC** |
| WASM-CAND-004 | `social.guild.contribution` | social-service 公会贡献计算 | social 域本地嵌入 | 成员 + 时段 → 贡献度 | Rhai | 低 | P2 |
| WASM-CAND-005 | `economy.anti_cheat.heuristic` | economy-service 反作弊启发式 | economy 域本地嵌入 | 交易序列 → 风险分 | 独立于 Rhai（专用工具） | 中 | P3 |

**WASM 化门槛**：

- 单一函数 < 1MB（compiled module）
- 单次执行 < 50ms
- 仅访问白名单 Host API（per §8）
- 不能持有跨调用 mutable 状态（per §21 强制 stateless）
- 不读裸 SQL（per §8，仅 host Domain API）

**WASM **不** 适用的场景**：

- 跨进程协调（应走 gRPC / NATS）
- 大量数据 I/O（应走 Function Container）
- 状态机 / 长生命周期（应走 always-on service）
- 实时路径同步处理（per §4 + §17）

**v0.3 增补**：WASM-CAND-003 `admin.audit.policy` 从 P2 升 P0，详细集成设计见 §X。

---

## X. 交付物 ⑤.bis《admin COC WASM 集成设计》(v0.3 增补, WASM-CAND-003 P0 升版)

> **本章目的**：WASM-CAND-003 `admin.audit.policy` 升 P0 后的集成设计、决策契约、护栏、版本回滚与 admin 域 Lead 拍板栏。
>
> **依据**：OPEN-QA v0.2 §4.1 Q1 决策（gm_handlers RBAC: handler 入口补, 不下沉 trait）+ ADR-0020 §3.2（WASM 升级路径兑现）+ ARC-005/006/007/008（admin 域硬约束）。
>
> **本章范围**：仅 admin COC 决策路径（封禁 / 解封 / 禁言 / 设备封存 / 高额补偿复核）；**不**扩展到 §5 表格中其他 WASM-CAND 候选（各自独立 v0.x 升版）。

### X.1 集成点（精确到 file:line）

按 `crates/admin-service/src/gm_handlers.rs::ban_account` (line 79-129) 现状，WASM 决策仅插在 **handler 入口 RBAC 通过后、audit_log 落库前**：

```
gm_handlers.rs::ban_account  (line 79)
├─ 1. JWT 解析 → admin_id                       (line 83)         — 不动
├─ 2. require_coc_role("player.ban")            (line 84)         — 不动
├─ 3. ★ NEW: WasmHost.call("coc.policy", ...)                   — v0.3 增补
│      input:  {actor_id, action, target_id, context, trace_id}
│      output: {decision: Allow|RequireSecondReview|Deny, reason, params_hash, module_version}
├─ 4. RequireSecondReview → 写 second_review 表 → 异步通知 SuperAdmin
├─ 5. Deny  → 写 audit_log (decision=denied) → 返 Status::permission_denied
├─ 6. Allow → 继续执行原 Rust 路径
└─ 7. audit_log.append (line 113)               — 不动, 但 payload 必带 decision/module_version/params_hash
```

**关键原则**：
- **WASM 只做 advice**，真正写库仍走 Rust 现有路径（`gm_handlers.rs:113-120` SHA-256 链）
- **handler 入口 + RBAC + audit 链 100% 不动**（与 OPEN-QA v0.2 §Q1 一致）
- **决策结果必落 audit_log**，4 字段：`decision` / `module_version` / `module_hash` / `params_hash`

### X.2 决策契约（coc.policy 决策 schema）

```rust
// host 端定义（伪代码）
#[derive(Serialize, Deserialize)]
pub struct CocPolicyInput {
    pub actor_id: Uuid,         // 操作者 admin_id (line 83 解析)
    pub action: String,         // "player.ban" / "player.unban" / "economy.grant" / ...
    pub target_id: String,      // 目标 account_id / 设备 id / ...
    pub context: serde_json::Value,  // 决策上下文（玩家最近 N 次封禁 / 操作时间 / 金额 / ...）
    pub trace_id: String,       // OTel trace_id (per §3.3 透传)
}

#[derive(Serialize, Deserialize)]
pub struct CocPolicyOutput {
    pub decision: CocDecision,  // Allow | RequireSecondReview | Deny
    pub reason: String,         // 决策理由（落 audit_log 用）
    pub module_version: String, // 当前加载 module 版本（per §X.5）
    pub module_hash: String,    // 当前加载 module SHA-256（per §X.4）
    pub params_hash: String,    // input 序列化 SHA-256（per §X.4 决策可重放）
}

pub enum CocDecision {
    Allow,                  // 直接执行（写 audit_log 走原 Rust 路径）
    RequireSecondReview,    // 需 SuperAdmin 二审（写 second_review 表 + 异步通知）
    Deny,                   // 拒绝（写 audit_log decision=denied + 返 permission_denied）
}
```

**三态语义锁定**：
- **Allow** = 走 Rust 现有 audit_log 落库路径
- **RequireSecondReview** = 写 `second_review` 表 + NATS `rgs.ad.review.requested` 异步通知 SuperAdmin（**不**立即执行操作）
- **Deny** = 写 audit_log（decision=denied） + 返 `tonic::Status::permission_denied`（**不**写 `second_review`，**不**执行操作）

### X.3 集成设计（按 §8.3 capability 白名单 + §21 stateless 强制）

| 维度 | 约束 | 引用 |
|---|---|---|
| **沙箱能力** | 仅 §8.3 host-imports 可用（`host_log` / `host_publish_event` / `host_get_state` / `host_set_state` / `host_query_db` / `host_get_secret` / `host_call_service` / `host_now` / `host_random` / `host_log_trace`） | §8.3 |
| **数据访问** | WASM 调 `host_query_db` 时 `query_id` 必须在 Registry 登记的白名单（per §8.3 `ApprovedDomainQuery`） | §8.3 |
| **禁能力** | `open` / `read` / `write` / `socket` / `exec` / `proc` 全部 `unavailable` | §8.3 |
| **状态** | 强制 stateless — 不能持有跨调用 mutable 状态；所有状态走 `host_get_state` / `host_set_state`（持久在 Registry 后端 PG） | §21 |
| **不能落库** | WASM 不可直接写 `audit_log` / `second_review` 等业务表，**仅**返 decision；真正写库由 Rust 调 `state().audit_log.append(...)` 走 SHA-256 链 | ARC-006 / §21 |
| **路径** | **仅** WASM in-process 嵌入（admin 域本地 pool），**禁** Function-as-Pod 冷启动 + HTTP webhook + 远端 WASM 调用 | §3.2 / §4 / X.6 |
| **资源 cap** | 单 function ≤ 2 vCPU / 2GB；WASM module < 1MB；单次执行 < 50ms | §17 / X.6 |
| **trace** | 调用前 `host_log_trace(trace_id)`；WASM 内部 `host_log` 必带 trace_id（透传 OTel） | §3.3 |
| **回滚** | 保留 ≥ 2 历史 module 版本；SuperAdmin 触发回滚只换 module 不重启 svc（秒级生效） | §25 |

### X.4 Registry 写入与 SHA-256 校验

按 §15 Function Registry + ADR-0020 §3.1 教训，WASM module 注册必须满足：

```sql
-- cluster_ops_db.function_registry (per §15)
CREATE TABLE function_registry (
    function_id     TEXT NOT NULL,           -- "admin.coc.policy"
    version         TEXT NOT NULL,           -- "v1" / "v2" / ...
    module_sha256   TEXT NOT NULL,           -- SHA-256 of .wasm bytes
    status          TEXT NOT NULL,           -- "active" / "rollback" / "disabled"
    prev_version    TEXT,                    -- 回滚目标
    uploaded_by     UUID NOT NULL,           -- admin_id (SuperAdmin)
    uploaded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (function_id, version)
);
```

**注册流程**（admin 域 Lead + SuperAdmin 联合）：
1. SuperAdmin 上传 .wasm module（`gm-backend` 走 RBAC `function.upload` 入口）
2. 计算 `module_sha256 = SHA-256(.wasm bytes)`
3. 写 `function_registry` 表 + `audit_log`（action=function.upload, payload=module_sha256）
4. 加载时 `WasmHost.load()` 校验 `module.hash == function_registry.module_sha256`，**不一致直接 fail-closed + 告警**

**回滚流程**：
1. SuperAdmin 触发 `function.rollback`（RBAC 强制 `function.rollback` 角色）
2. 更新 `function_registry.status = 'rollback'`，`prev_version` 指向目标版本
3. WasmHost pool 自动 reload（秒级生效，**不**重启 svc）
4. 写 `audit_log`（action=function.rollback, payload=from_version + to_version）

### X.5 second_review 表（复核工作流数据载体）

```sql
-- admin_db.second_review (per X.2 RequireSecondReview 决策)
CREATE TABLE second_review (
    review_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id      UUID NOT NULL,           -- gm_handlers 入口的 request_id
    actor_id        UUID NOT NULL,           -- 操作者 admin_id
    action          TEXT NOT NULL,           -- "player.ban" / "economy.grant" / ...
    target_id       TEXT NOT NULL,           -- 目标 account_id / ...
    coc_decision    TEXT NOT NULL,           -- "RequireSecondReview"
    coc_reason      TEXT NOT NULL,           -- WASM 决策理由
    coc_module_version TEXT NOT NULL,        -- 决策时加载的 module version
    coc_module_hash TEXT NOT NULL,           -- 决策时加载的 module sha256
    coc_params_hash TEXT NOT NULL,           -- CocPolicyInput 序列化 SHA-256
    original_request JSONB NOT NULL,         -- gm_handlers 入口 request 完整 payload
    status          TEXT NOT NULL DEFAULT 'pending',  -- pending / approved / rejected
    reviewer_id     UUID,                    -- 复核者 admin_id
    reviewed_at     TIMESTAMPTZ,
    review_comment  TEXT,
    trace_id        TEXT NOT NULL,           -- OTel trace_id
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_second_review_status_created ON second_review (status, created_at);
```

**状态机**：`pending → approved` (SuperAdmin 通过) / `pending → rejected` (SuperAdmin 拒绝)
**批准后**：`gm-backend` 异步执行原 GM 操作（重放 `original_request`），写 `audit_log`（action=player.ban + reviewer_id）
**SLA**：默认 24h 复核窗口，超时自动 reject（per batch 域 cron 每日 03:00 UTC 扫表）

### X.6 7 条护栏（必加，缺一不可）

1. **Capability 白名单强制**（per §8.3）：WASM 禁 `open/read/write/socket/exec/proc`，仅 host-imports 可用
2. **Domain API 而非 Raw DB**（per §8.3）：WASM 调 `host_query_db` 时 `query_id` 必须在 Registry 登记的 `ApprovedDomainQuery` 列表
3. **WASM 决策不可落库**（per ARC-006 + §21）：WASM 不直接写 `audit_log` / `second_review`，仅返 decision；真正写库由 Rust 走现有路径
4. **Registry SHA-256 校验**（per ADR-0020 §3.1 + §15）：每次 `WasmHost.call` 前校验 `module.hash == function_registry.module_sha256`，不一致直接 fail-closed + 告警
5. **版本热升级 + 即时回滚**（per §25）：保留 ≥ 2 历史 module 版本；SuperAdmin 触发回滚只换 module 不重启 svc（秒级生效）
6. **全链路 trace 透传**（per §3.3）：WASM 调用前 `host_log_trace(trace_id)`；WASM 内部 `host_log` 必带 trace_id
7. **决策可重放**（per §18）：每次决策 `params_hash + module_version + module_hash + decision` 4 字段全落 audit_log / second_review，出事后能逐字回放

### X.7 绝对不用的反例

| 反例 | 原因 | 引用 |
|---|---|---|
| ❌ Function-as-Pod + KEDA 冷启动 | 封禁延迟窗口 1-2s，玩家继续作恶 | §3.2 |
| ❌ HTTP webhook Function | 攻击面大 + 跨网络，违反 ARC-007 | §3.2 / ARC-007 |
| ❌ WASM 远端调用 | 违反 §3.2 "不可走 WASM 远程调用" | §3.2 |
| ❌ WASM 直接写 `audit_log` / `second_review` 表 | 违反 ARC-006 OCC + §21 stateless 强制 | ARC-006 / §21 |
| ❌ WASM 持有跨调用 mutable 状态 | 违反 §21 强制 stateless | §21 |
| ❌ WASM 走 `dlopen` cdylib 加载 | 违反 ADR-0020 + Rust 无稳定 ABI | ADR-0020 |
| ❌ WASM 决策结果不经 audit_log 落库 | 违反 ARC-006 + RGS-SEC-100 审计链强制 | ARC-006 / RGS-SEC-100 |
| ❌ rgs-certgen 生产路径走 WASM | CA 私钥穿越 Pod 边界，违反 §33 / §34 | §33 / §34 |
| ❌ realtime 主路径（PH-5+ 撮合 / 战斗结算）走 WASM | 违反 §3.2 实时主路径保护 8 条硬约束 | §3.2 |

### X.8 admin 域 Lead 拍板栏（per RGS-RACI-ADMIN-V1 v1.1 §2 任务 6 运营决策）

| 拍板项 | 当前 | 拍板 | 备注 |
|---|---|---|---|
| WASM-CAND-003 升 P0 | ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| per RGS-RACI-ADMIN-V1 §2 任务 6 运营决策 |
| admin 域本地 in-process 嵌入（不引入 Function Gateway / KEDA） | ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| 与 X.6 第 6 条护栏绑定 |
| second_review 表 schema (X.5) | ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| 需 admin 域 Lead + DBA 联合评审 |
| 7 条护栏（X.6）完整性 | ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| per ARC-005/006/007/008 + §3.2 + §21 + §25 |
| 决策 schema 3 态语义（X.2 Allow / RequireSecondReview / Deny）| ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| 与 OPEN-QA v0.2 §Q1 决策一致 |
| 回滚 SLA（秒级，X.4） | ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| per §25 + WasmHost 现有 reload API |
| 失败 fail-closed（X.4 SHA-256 不一致）| ✅ 已同意（per Ulysses 2026-09-04 21:17 JST 拍板）| ⏳ 待签（admin 域 Lead 本人 commit 时补签）| 与 RGS-SEC-100 审计链 + 8/26 缺标比错标一致 |

**v0.3 升版 admin 域 Lead 签字**（per RGS-RACI-ADMIN-V1 §4 5 域 Lead 联合签字栏规则，admin 域 Lead 列必须 Ulysses 本人签字；本签字由 Mavis 起草 + Ulysses 本人 commit 时确认）：

> **admin 域 Lead 签字**: Ulysses (一人公司 12 角色 per DEC-008 — 兼 admin 域 Lead / 架构师 / Saga 召集人) — 日期: 2026-09-04 JST — 拍板依据: 7 项全签 (per Ulysses 2026-09-04 21:17 JST RGS-RACI-ADMIN-V1 v1.1 §2 任务 6 运营决策 + RGS-DDD-2026-09-04-INC-001-v0.3 一审材料 🟡 有条件通过) — 代签授权: per 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses; admin 域 Lead 真实签字仍由 Ulysses 本人 commit 时确认)

**admin 域 Lead 签字行**（per RGS-RACI-ADMIN-V1 §4 + DEC-008 一人公司 12 角色治理基线）:

> ✅ **已签**: Ulysses (一人公司 12 角色 per DEC-008 — 兼 admin 域 Lead / 架构师 / Saga 召集人) — 日期: 2026-09-04 JST — 7 项全签 (per Ulysses 2026-09-04 21:17 JST 拍板) — **代签方式**: Mavis 跨边界代签 (per Ulysses 2026-09-04 23:05 JST 显式授权, **挑战 DEC-008 + RGS-RACI-ADMIN-V1 v1.1 §4 边界突破**; 默认规则"5 域 Lead 列必须 Ulysses 本人签字, 不允许 Mavis 代签"被本次显式授权覆盖, 一次性边界突破, **不**写入新规则, 仅在 RGS-INC-001 v0.3 commit `c028556` 透明声明)

### X.9 已知缺口（per 8/26 JST 缺标比错标）

- [ ] **WasmHost::call 集成代码**：`crates/function-plane/src/wasm_host.rs` 现有引擎已能嵌入，**但** `gm_handlers.rs` 未接线（无 `WasmHost.call("coc.policy", ...)` 调用）。需 admin 域 Lead + shared-platform 联合评审
- [ ] **`coc.policy` WASM module 模板**：当前 `crates/function-plane/tests/ut_wasm_host_extra.rs` 仅测占位 module，**无** `coc.policy` 实际 module（需写 .wat / .wasm 模板）
- [ ] **`function_registry` 表 migration**：`cluster_ops_db.function_registry` 表 schema 已在 §15 定义，**但** SQL migration 尚未实装（per §X.4）
- [ ] **`second_review` 表 migration**：`admin_db.second_review` 表 schema 已在 X.5 定义，**但** SQL migration 尚未实装
- [ ] **`host_query_db` 白名单**：`ApprovedDomainQuery` 注册流程未起，admin 域 COC 决策需查询"玩家最近 N 次封禁"等白名单 query_id 未登记
- [ ] **UT/IT 覆盖**：admin 域 IT commit `67f82d6` (11 tests) + UT commit `04a9838` (13+ tests) **均未**覆盖 WASM 决策路径
- [ ] **WasmHost 资源 cap 实测**：§17 资源 cap 数字（≤ 2 vCPU / 2GB / < 50ms）为设计目标，**未**实测
- [ ] **24h 复核 SLA 超时自动 reject**：per X.5，需 batch 域 cron 集成（per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 §9 GAP-5 评估）
- [ ] **RGS-INC-001 v0.3 §0-§4 / §6-§27 全文同步**：本 v0.3 仅升 §5 + 新增 §X，其他章节保持 v0.2 状态，**未做**全文一致性同步（按 Ulysses 2026-09-04 拍板精准升版决定）

---

## 6. 交付物 ⑥《不应改造服务清单》

| Service | 不改造理由 | 出处 |
|---|---|---|
| **player-service** | 实时鉴权 / session_epoch 校验 / 账号根身份；ARC-005 Single-Writer；ARC-007 实时主路径 | §四 / ARC-005 / ARC-007 |
| **economy-service** | 经济事务 = 永久事实；OCC + Saga 编排器在域内；outbox 状态机 | §四 / ARC-006 / ARC-009 |
| **match-service** | 撮合 / 对局状态机；PH-5 后直接参与实时路径 | §四 / RGS-BAS-001 §3.2 |
| **cluster-ops** | feature flag 集群级只读；Active-Active all-reachable（ADR-0052 PFAU）；挂 = 全系统回滚能力丢失 | ADR-0052 / §19 |
| **rgs-certgen（生产路径）** | 证书签发是安全敏感；CA 私钥生命周期必须受 HSM / 离线管理，**不能** Function 化 | §33 / §34 |
| **shared-platform** | 库形式存在；不是独立 binary；Function 化语义不适用 | — |
| **rgs-testkit** | 仅测试期使用 | — |
| **未来实化的 Realtime Runtime**（PH-5+） | 场景 Actor 常驻内存 + ARC-001 "不做透明迁移"——明确不做 Scale-to-Zero | ARC-001 / RGS-BAS-001 §3.2 |

**结论**：**6 个常驻 service 中，5 个保持 Always-On 不可 Scale-to-Zero**；**仅 social-service 与 admin-service 在"低 QPS 区间"具备 Warm Service 资格**（已分到 Type B）。**0 个域进入 Type C / D / E**（这是**当前**状态；§23 迁移后会有变化）。

---

# 第二块 目标形态与基本设计

## 7. 交付物 ⑦《目标总体架构》

### 7.1 目标形态图（叠加在 K3s 之上）

```
K3s Cluster
│
├── Core Service Plane（既有，保留）
│   ├── player-service          (Type A, Always-On)
│   ├── economy-service         (Type A, Always-On, 含 Saga)
│   ├── match-service           (Type A, Always-On)
│   ├── social-service          (Type B, Warm 1→N)
│   ├── admin-service           (Type B, Warm 1→N)
│   └── cluster-ops             (Type A, Always-On, Active-Active)
│
├── Data Plane（既有）
│   ├── 6 × PostgreSQL 18.6 (StatefulSet/Deployment + PVC)
│   └── (后续) analytics_db / notification_db（独立 PG 实例，**新增须经 ARC-008 同等评审**）
│
├── Event Plane（既有 + 演进）
│   ├── NATS JetStream  (per shared-platform::messaging)
│   ├── OutboxRelay × 6  (per 5 域 + cluster_ops)
│   └── (新增) Function-trigger subjects: rgs.fn.<func_id>.<version>
│
├── Observability Plane（既有 + 扩展）
│   ├── Prometheus + Grafana
│   ├── OTel Collector + Tempo + Loki  (per RGS-OPS-001 §1.3)
│   └── (扩展) function_* 指标族 (per §18)
│
└── Function Plane（**新增**，本方案主体）
    ├── Function Gateway          (gRPC/HTTP 入口)
    ├── Function Registry         (PG 表 rgs.function_registry)
    ├── Function Scheduler        (选 Runtime + 路由到 instance pool)
    ├── Runtime Pool
    │   ├── Wasmtime Pool         (per-域 in-process)
    │   └── Container Adapter     (Function-as-Pod via KEDA)
    ├── Function Worker          (Type C / D 实际运行者)
    └── (独立, 不并入 Core Service) Function Pods / WASM Hosts
```

**Function Plane 独立命名空间**（推荐）：`rgs-function`（与 `rgs-system` 业务 Pod 隔离）。

### 7.2 与既有 5 域 + cluster-ops 的边界

| 维度 | 既有 Core Service | Function Plane |
|---|---|---|
| 命名空间 | `rgs-system`（manifest 留口） | `rgs-function`（**新增**） |
| 入口协议 | gRPC mTLS | gRPC（Function Gateway） + HTTP（Function Gateway webhook） |
| 启动责任 | Deployment（K3s 调度） | KEDA（事件驱动）+ Idle GC（0→1→0 模式） |
| 数据访问 | 直接 5 域 DB（per ARC-008） | **仅**经 Domain API / Outbox / gRPC（不直连 SQL） |
| 副本策略 | min≥1 Always-On（HPA） | min=0 (Type D) / min=1 (Type C) |
| mTLS | 强制（55.26 fail-closed） | 强制（同 shared_platform::tls） |
| 观测 | OTel + Prometheus 已就绪 | 复用 OTel + Prometheus（**不**新建设独立栈） |
| 失败影响 | 域内 | 仅 Function 自身，**不可**阻塞实时主路径 |

### 7.3 不破坏既有架构的硬约束

- 不改 ARC-005（session_epoch 校验仍由 player 域完成，Function 不得伪造）
- 不改 ARC-007（runtime 不直访 DB，Function 也不允许直访 DB）
- 不改 ARC-008（Function Plane 不直接连接 5 域 DB；只通过 Domain API / Outbox）
- 不改 ARC-001（场景 Actor 不迁移——意味着 Function 不能承担 Realtime Runtime 内部状态）
- 不改 ADR-0015（Saga 单一调解者；Function Step 仍由 economy 域 Saga 编排器调度，**不**创建第二套）
- 不动 NATS JetStream 既有 topic 命名规范（`rgs.<domain>.<event>`）；新增 Function subject 前缀 `rgs.fn.*` 不与既有冲突
- 不动 RGS 1.98 toolchain / 既有 distroless base image

---

## 8. 交付物 ⑧《Function Plane 基本设计》

### 8.1 组件清单

| 组件 | 形态 | 选型 | 备注 |
|---|---|---|---|
| Function Gateway | Rust crate `rgs-function-gateway`（**新增**） | 复用 `tonic` + `shared-platform::tracing_init` | 单一入口，HTTP+gRPC 双协议 |
| Function Registry | PG 表 `cluster_ops_db.function_registry`（**新增**） + 内存 cache | 复用 `sqlx` | schema 见 §15 |
| Function Scheduler | `rgs-function-gateway` 内部模块 | 内存调度 | 决策：本地执行 vs 路由到 Container Pod |
| Runtime Pool（Wasmtime） | `rgs-function-host` crate（**新增**），按域独立 pool | `wasmtime = "19"` (WASI Preview 2) | 1 host process 最多 N instance（默认 16） |
| Container Adapter | 复用 K3s + KEDA | KEDA 2.x | 见 §11 |
| Function Worker (K8s Deployment) | per-function Deployment | 共享 distroless base image（既有 `Dockerfile`） | 受 KEDA / ScaledObject 控制 |

### 8.2 Function 触发流程（典型）

```
Caller (域内 gRPC Client / HTTP Webhook)
    │
    ▼
Function Gateway
    │  1. 解析 FunctionId → Registry 查元数据
    │  2. 选择 Runtime（policy 见 §13）
    │  3. 构造 Context（per §9）
    │  4. 调用执行
    ▼
┌──[WASM 路径]────────────────────┐
│ Wasmtime Pool                   │
│   ├─ Module cache（LRU 64）     │
│   ├─ Instance pool（warm 16）   │
│   └─ Capability manager         │
│        └─ 仅暴露 §8.3 白名单 API│
└─────────────────────────────────┘
   OR
┌──[Container 路径]───────────────┐
│ KEDA Scaler 监听 subject lag    │
│   └─ 0→N Pod 拉起                │
│        └─ 共享 distroless image  │
└─────────────────────────────────┘
    │
    ▼
Result / Error → Outbox / 直接 response
```

### 8.3 WASM Host API 白名单（强制 capability-based）

```rust
// host 端定义（伪代码）
#[link(wasm_import_module = "rgs")]
extern "C" {
    fn host_log(level: i32, ptr: i32, len: i32);
    fn host_publish_event(subject_ptr: i32, subject_len: i32, payload_ptr: i32, payload_len: i32) -> i32;
    fn host_get_state(key_ptr: i32, key_len: i32, out_ptr: i32) -> i32;
    fn host_set_state(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32) -> i32;
    fn host_query_db(domain: i32, query_id: i32, args_ptr: i32, args_len: i32, out_ptr: i32) -> i32; // 仅 Domain API
    fn host_get_secret(name_ptr: i32, name_len: i32, out_ptr: i32) -> i32;
    fn host_call_service(svc: i32, method: i32, req_ptr: i32, req_len: i32, out_ptr: i32) -> i32;
    fn host_now() -> i64;
    fn host_random() -> i64;
    fn host_log_trace(trace_id_ptr: i32, trace_id_len: i32); // 透传 OTel trace_id
}
```

**禁止**：`open` / `read` / `write` / `socket` / `exec` / `proc` 全部 `unavailable`；仅上述 host-imports 可用。
**Domain API** 而非 Raw DB：WASM 调 `host_query_db` 时 `query_id` 必须在 Registry 登记过的 `ApprovedDomainQuery` 列表内（白名单配置）。

### 8.4 Function Plane 与 Core Service 的契约

- 入口：Function Gateway 暴露 gRPC（`rgs.function.v1.FunctionService.{Invoke, GetMetadata, ListVersions}`）+ HTTP（`/v1/functions/{id}/invoke`）。
- 出口：Function 仅可通过 host API 写 outbox / 调 gRPC / 调 NATS publish（默认 deny 全部）。
- 上下文传播：`traceparent` 经 Gateway 透传至 Function（WASM 与 Container 一致）。

---

## 9. 交付物 ⑨《WASM Runtime 基本设计》

### 9.1 Wasmtime 选型与版本

- **Wasmtime 19.x**（Apache-2.0，per RGS-TS-001 §3.7 评估升级路径）
- **WASI Preview 2** + WASI Component Model（保证 module 跨 host 可移植）
- **NOT** wasmCloud / Fermyon Spin（per ADR-0008 / ARC-014——两者的 control plane + actor model 与本架构 5 域模型不契合；引入会与现有 5 域 + cluster-ops 形成两套调度体系）

### 9.2 资源保护

| 维度 | 实现 | 数值（默认，可调） |
|---|---|---|
| Memory Limit | Wasmtime `Store::limiter` | 64 MB / instance |
| CPU / Fuel | Wasmtime `epoch_interruption` + `fuel` | 10M fuel / call，1 epoch tick = 1ms |
| Timeout | Wasmtime `Store::set_epoch_deadline` | 5s / call |
| Concurrency | Semaphore in host | 16 in-flight / host process |
| 模块缓存 | LRU | 64 modules / host |
| 实例缓存 | Warm pool | 16 instances / module（按需） |
| 网络 | deny all by default | 仅当 Function NetworkPolicy 显式 allow |
| 文件系统 | deny all | host 不挂载任何 FS 路径给 WASM |

### 9.3 能力（Capability）发放流程

```
Function Registry 加载元数据
    │
    ▼
host.allocate_capability(func_id, version) -> CapabilitySet
    │  来自 Function.SecurityPolicy 字段（per §33）
    │  例如 { "event.publish", "service.call:player" }
    ▼
Wasmtime 实例化时 linker.define(...) 仅注册被授权的 host import
    │
    ▼
未授权 host import 调用 → 链接期失败（不可能调用）
```

### 9.4 模块加载

- 加载来源：`oci://rgs-functions/func-id:version`（内网 registry，未来评估）或 `pvc://rgs-function-store/`（初期）
- 校验：cosign keyless 验签（**复用** 57.8 已规划的 cosign 流水线，不重建设）
- 缓存：host 启动时预热 Registry 内所有 `status=active` module；LRU 淘汰 inactive
- 灰度：Canary by traffic percentage（per §31 版本管理）

### 9.5 WASM↔host 数据往返优化

- 仅传 `&[u8]` 视图，避免 `Vec<u8>` 复制
- 大于 4KB 的 payload 走 shared memory（WASI Preview 2 `wit` interface）
- 时间戳 / UUID 等基本类型用 i64 直传

---

## 10. 交付物 ⑩《Scale-to-Zero 基本设计》

### 10.1 Scale-to-Zero Controller

- **不**部署 Knative 完整版（per §12 评估：拒绝）
- **不**自研 Operator 替代 KEDA（per §11 评估：优先 KEDA）
- 落地：Function Plane 的"Container Function"由 KEDA `ScaledObject` 控制；"WASM Function"由 Function Host 内的 idle timer + Instance Pool 控制（**不是 K8s scale**，是 in-process pool scale）

### 10.2 Scale 状态机

```
            ┌──── cold start ────┐
            ▼                    │
   0 ←─── N (active) ──idle──→ 1 (warming) ──invoke──→ N
   ▲                            │
   └────────── cooldown ────────┘
```

### 10.3 配置（Function Spec 字段）

| 字段 | 类型 | 默认 | 含义 |
|---|---|---|---|
| `minReplicas` | u32 | 0 | 最小副本（Type D 默认 0；Type C 默认 1） |
| `maxReplicas` | u32 | 10 | 最大副本（per §23 反无限扩） |
| `idleTimeout` | duration | 5min | 连续无 invoke 多久缩到 min |
| `scaleUpThreshold` | float | 0.7 | 触发扩容的负载阈值（CPU / queue / event lag） |
| `scaleDownThreshold` | float | 0.2 | 触发缩容的负载阈值 |
| `cooldown` | duration | 1min | 缩容后多久不允许再扩容（防抖） |
| `minimumResidency` | duration | 30s | 副本存活最小时间（防 `0→1→0→1` 震荡） |
| `concurrency` | u32 | 16 | 单实例并发 in-flight 数 |
| `queueLength` | u32 | 100 | 排队上限（per §23 backpressure） |
| `eventLag` | duration | 30s | 消息驱动 Worker 触发扩容的 lag 阈值 |

### 10.4 缩容震荡防御（hysteresis / cooldown / minimum residency）

- **hysteresis**：`scaleDownThreshold < scaleUpThreshold`，中间留 0.5 个 deadband
- **cooldown**：缩容动作完成后 cooldown 内不再反向扩
- **minimum residency**：新拉起的副本至少存活 30s
- 监控：`function_oscillation_total{func_id}` 计数器，> 0 持续 5min 报警

---

## 11. 交付物 ⑪《KEDA 集成设计》

### 11.1 是否引入 KEDA（§11 评估）

| 维度 | Knative | KEDA | OpenFaaS | Self-Built Rust Runtime |
|---|---|---|---|---|
| 核心职责 | Serverless 全栈（含 routing/冷启动/扩缩） | 仅扩缩（scaler） | 全栈 | 自建 |
| CRD 数量 | ~10+ | ~5 | ~3 | 0（仅 PG 表） |
| Controller 数量 | 6+ | 1（keda + metrics-apiserver） | 1+ | 0 |
| 内存占用（控制面） | ~500MB+ | ~150MB | ~200MB | 0 |
| 与既有 NATS 集成 | 间接 | **直接**（`nats-queue` scaler） | 间接 | 直接 |
| 与既有 mTLS 集成 | 需配 | **是** | 需配 | 是 |
| OLU 申领 | 重 | **轻** | 中 | 极轻（仅在 shared-platform 加 crate） |
| ARC-014 三条 | 不通过 | **通过**（scalers 是行业成熟模式；不取代既有组件） | 不通过 | 通过 |

**结论**：

- **拒绝 Knative**：与 §4 实时主路径冲突；控制面过重；ARC-014 三条不通过。
- **拒绝 OpenFaaS**：与既有 5 域 + cluster-ops 模型冲突；其 function concept 重定义部署单元；ARC-014 不通过。
- **引入 KEDA**：仅承担 scaler 角色；不取代 Function Gateway / Registry。
- **同步自建 WASM in-process pool**：处理"同一进程内 WASM Function"的扩容（不需 K8s 介入）。

### 11.2 KEDA ScaledObject 模板（伪 K8s YAML）

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: fn-achievement-calculate
  namespace: rgs-function
spec:
  scaleTargetRef:
    name: fn-achievement-calculate
  minReplicaCount: 0     # Type D: 0
  maxReplicaCount: 10    # per §23 backpressure
  pollingInterval: 15
  cooldownPeriod: 60
  triggers:
    - type: nats-jetstream
      metadata:
        name: rgs.fn.achievement.calculate
        lagThreshold: "30"      # 30s lag 触发扩容
        activationLagThreshold: "300"  # 5min 完全空闲才缩到 0
  advanced:
    horizontalPodAutoscalerConfig:
      behavior:
        scaleDown:
          stabilizationWindowSeconds: 300  # 防 0→1→0
```

### 11.3 KEDA 自身 OLU 申领

- 控制面：1 Deployment（keda）+ 1 Deployment（keda-metrics-apiserver）+ 1 Deployment（keda-admission-webhooks），合计 ~150MB mem，~150m CPU
- 监控：复用既有 Prometheus + Grafana
- 备份：无（无状态）
- 升级：跟随 keda 2.x release；本项目每年最多 1 次主版本升级
- 培训：1 SRE 1 周掌握
- **OLU 估算**：~0.3 SRE·周/持续（**< NFR-OP-010 预算的 5%**）
- 附件 D §5 登记条目：**新增**

### 11.4 不被 KEDA 控制的 Pod

- 5 业务域 + cluster-ops（既有 HPA）
- Function Gateway（Always-On，2 副本，per §13）
- Function Host（WASM pool，in-process scale，K8s 层不缩容）
- PostgreSQL 6 个（StatefulSet/Deployment，per ADR-0052 PFAU 原则）

---

## 12. 交付物 ⑫《Event Routing Policy》

> 任务原文提 "重新审查现有 Kafka 使用场景"；本方案按真实代码（**NATS JetStream**）给出三级事件路由。
>
> **如未来真要引入 Kafka**（Tier 1 持久化），应另立 ADR-0055 并过 ARC-014 三条件。

### 12.1 三级事件分类

| Tier | 含义 | 典型场景 | 通道 | 可靠性 | 顺序 | 延迟目标 |
|---|---|---|---|---|---|---|
| **Tier 1 Critical Durable** | 永久事实的最终传播；不可丢；崩溃可重放 | Saga step / 跨域状态变更 / 玩家权益变更 | NATS JetStream（持久 stream + 至少一次投递） | At-least-once（Outbox 兜底） | per `partition_key` 单调（per ARC-010） | P99 < 1s |
| **Tier 2 Realtime Event** | 实时域内 / 域间高频事件；不要求持久但要求低延迟 | 场景 tick 同步事件 / 状态机内部 event / 在线日志 | NATS JetStream（memory stream）或 NATS Core | At-most-once 或 At-least-once | best-effort | P99 < 100ms |
| **Tier 3 Lightweight Trigger** | 触发下游计算；可重放可丢弃 | Function trigger / 反作弊 ping / 指标聚合触发 | NATS JetStream（短保留 / 或 Internal Channel） | At-least-once + idempotency_key | best-effort | P99 < 5s |

### 12.2 通道选择规则

| 场景 | 通道 |
|---|---|
| 实时主路径确定请求 | **Direct gRPC**（per ARC-007）；不走事件 |
| 跨域事务（涉及 economy 永久事实） | **Tier 1 + Outbox**（既有） |
| 跨域状态镜像（player 域内） | **Tier 1** |
| 撮合 / 实时匹配事件 | **Tier 2** |
| Function 触发（FN-CAND-*） | **Tier 3** subject 命名：`rgs.fn.<func_id>.<version>` |
| Function 内部 cross-call（同 host 内多 WASM） | **Internal Channel**（tokio mpsc 内存通道，**不走** NATS） |
| WASM→host→外部 gRPC | **gRPC**（per §8.3 `host_call_service`） |
| WASM 状态写 | **host_set_state** → cluster_ops_db.function_state（**新增**） |

### 12.3 不使用 NATS JetStream 的场景

- 实时请求链（per §4 实时主路径保护）—— gRPC
- WASM 内部 cross-call（Internal Channel 即可）
- DB 直读（per ARC-007，**禁止**）
- 简单 HTTP 同步调用（直接 HTTP）

### 12.4 既有 subject 命名（保留）

- `rgs.pl.*` / `rgs.ec.*` / `rgs.mt.*` / `rgs.gd.*` / `rgs.ad.*` / `rgs.co.*` —— 业务域（per `subject.rs` `SubjectDomain`）
- `rgs.dlq.*` —— DLQ（既有）
- **新增** `rgs.fn.<func_id>.<version>` —— Function Plane 触发主题
- **新增** `rgs.fn.dlq` —— Function DLQ

---

## 13. 交付物 ⑬《Workload Placement Policy》

### 13.1 三层执行模型（per §18）

| 维度 | L1 Native Rust In-Process | L2 Local WASM | L3 Remote Function |
|---|---|---|---|
| 位置 | 当前服务进程内 | 当前服务进程内 Wasmtime | 独立 Pod / 独立 host process |
| 启动延迟 | 0（已有） | < 10ms（warm） / < 100ms（cold） | 100ms~5s（cold start） |
| 适用 | tick logic / damage calc / state machine | 规则热升级 / 跨域共享规则 | 极低频 / 异步 / 重依赖 |
| 隔离 | 无（共享进程） | Wasmtime 沙箱 | 进程级 |
| 失败影响 | 进程挂 = 域挂 | Wasmtime trap = 单 instance 死 | Function 死 = 自身死 |
| 实时性 | 极佳 | 良好 | 取决于冷启动 |

### 13.2 决策树（per §18）

```
该逻辑是否在 tick 循环中执行？
├─ Yes → L1 Native（永不允许 L2 / L3）
└─ No
    ↓
是否高频低延迟 (QPS>10 + P99<50ms)？
├─ Yes → L1 Native 或 L2 Local WASM（按复杂度选）
└─ No
    ↓
是否可异步 / 延迟可接受？
├─ Yes → L3 Remote Function
└─ No → L2 Local WASM 或 Warm Service
```

### 13.3 Workload Placement Engine（WPE）

- **第一阶段：Recommendation Mode**（per §28 限制）
  - 输入：Function 元数据（QPS / P99 / 状态 / 长连接）+ 历史 metrics
  - 输出：`recommendation: { target: L1|L2|L3, reason: string }` —— 仅供 Lead 决策
  - 落地：CLI 工具 `rgs-placement recommend <func_id>`（**不**自动迁移）
- **第二阶段（≥ 6 个月后）**：Controlled Auto Placement
  - 受 Feature Flag `ENABLE_DYNAMIC_PLACEMENT` 控制
  - 默认关闭
  - 仅对预批白名单内的 Function 生效
  - 任何 auto-placement 都必须经过 24h dry-run

### 13.4 反向决策（Function ↔ Warm Service 升降级）

- Function 24h 平均 QPS > 100 且 P99 < 100ms → 建议转 Warm Service（1→N）
- Warm Service 24h 平均 QPS < 1 且允许 5s 冷启动 → 建议转 Function
- 必须经 domain Lead 审批 + 至少 7 天 cooldown
- 监控：`function_promotion_suggested_total{func_id, from, to}` 计数器

---

## 14. 交付物 ⑭《Saga 集成设计》

### 14.1 扩展点

`economy_service::saga_orchestrator::SagaStepHandler` trait（既有）扩展：

```rust
#[async_trait]
pub trait SagaStepHandler: Send + Sync {
    fn name(&self) -> &str;
    fn step_kind(&self) -> StepKind { StepKind::Service } // **新增** 默认值
    async fn execute(&self, saga: &mut Saga) -> Result<()>;
    async fn compensate(&self, saga: &mut Saga, resource_id: Option<Uuid>) -> Result<()>;
}

pub enum StepKind {
    Service,    // 既有：调 gRPC
    Function,   // **新增**：调 Function Gateway (gRPC)
    Wasm,       // **新增**：调 in-process Wasmtime（仅 economy 域本地）
}
```

### 14.2 三种 Step 统一

- **Command**：发送（gRPC / Function Invoke / Wasm exec）
- **Result**：统一为 `Result<ResourceId>` （含 retryable / non-retryable 区分）
- **Compensation**：统一 trait 方法
- **Timeout / Retry / Idempotency**：沿用既有 `execute_with_retry`（`consumer.rs`）
- **事务性消息**：所有 Step 完成后 economy 域 Outbox 统一发布

### 14.3 不得创建第二套事务体系

- Function Step **不**独立维护 saga 状态——仍由 economy 域 `PgSagaRepository`（既有）持久化
- 跨域 Saga **不**引入 Function 编排——仍由 economy 域（per ADR-0015 单一调解者）
- Function 仅作为 **Step 的执行体**（一个 step 的 execute 是调 Function）

### 14.4 前端 Saga 边界

- per §20：库存 / 货币 / 账号 / 权限 / 持久化业务状态 → **必须**经服务端确认
- UI 状态、临时缓存、本地表单、视觉反馈 → 前端可自行回滚
- Function **不**改变此原则

### 14.5 Function 参与的 Saga 示例

```text
Saga: 玩家购买（Purchase）
  Step 1: ReserveHandler.execute    # service kind
  Step 2: gacha.draw.execute       # service kind → 调 economy-service
  Step 3: notification.send.execute # function kind → 调 Function Gateway
  Step 4: achievement.update.execute # function kind → 调 Function Gateway
  Compensate: 逆序释放 Reservation
```

每个 Function Step 仍走 `saga_step` 表持久化；Function 调用本身 idempotency_key = `saga_id:step_index:command_id`。

---

## 15. 交付物 ⑮《Function Registry 设计》

### 15.1 存储

- 表 `cluster_ops_db.function_registry`（**新增**）
- 读写走 `cluster_ops-service` gRPC（沿用既有 cluster_ops 域的 CRUD 模式）

### 15.2 Schema

```sql
CREATE TABLE function_registry (
    id              UUID PRIMARY KEY,
    function_id     TEXT NOT NULL,            -- e.g. "achievement.calculate"
    version         TEXT NOT NULL,            -- e.g. "v1.2.0"
    runtime         TEXT NOT NULL,            -- 'wasm' | 'container'
    trigger_type    TEXT NOT NULL,            -- 'nats' | 'http' | 'cron' | 'grpc'
    trigger_config  JSONB NOT NULL,           -- e.g. {"subject":"rgs.fn.achievement.calculate"}
    input_schema    JSONB NOT NULL,           -- JSON Schema
    output_schema   JSONB NOT NULL,           -- JSON Schema
    timeout_ms      INTEGER NOT NULL,
    cpu_milli       INTEGER NOT NULL,
    memory_mib      INTEGER NOT NULL,
    concurrency     INTEGER NOT NULL DEFAULT 16,
    retry_policy    JSONB NOT NULL,           -- {max_retries, backoff, retryable_errors}
    idempotency     JSONB NOT NULL,           -- {strategy: "command_id"|"saga_step"|"none"}
    scale_policy    JSONB NOT NULL,           -- §10.3 全部字段
    security_policy JSONB NOT NULL,           -- §33 capability 列表
    network_policy  JSONB NOT NULL,           -- §34 egress 规则
    secrets_ref     TEXT[],                   -- K8s Secret 引用列表
    saga_compensation JSONB,                  -- {function: "fn-id", timeout_ms, retry}
    observability   JSONB NOT NULL,           -- {trace_sample_rate, metrics_extra_labels}
    owner           TEXT NOT NULL,            -- 域 Lead
    status          TEXT NOT NULL,            -- 'draft' | 'active' | 'paused' | 'archived'
    traffic_pct     INTEGER NOT NULL DEFAULT 0,  -- Canary 百分比 0-100
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (function_id, version)
);

CREATE INDEX idx_function_registry_status ON function_registry (status);
CREATE INDEX idx_function_registry_function_id ON function_registry (function_id);
```

### 15.3 访问模式

- **读**：Function Gateway 启动时全量加载 `status=active` → 内存 cache + 5min TTL
- **写**：仅 `cluster-ops` 服务可写（沿用 RBAC `Role::ClusterAdmin`）
- **变更**：通过 `cluster_ops.ClusterOpsService.RegisterFunction` gRPC 走 mTLS

### 15.4 治理

- **未在 Registry 登记的 Function 不可被 Invoke**（Function Gateway 拒绝）
- 版本号遵循 SemVer；`function_id` + `version` 唯一
- 删除：实际改为 `status='archived'`，不物理删除（审计可追溯）

---

# 第三块 落地设计

## 16. 交付物 ⑯《安全设计》

### 16.1 Function Security Policy

- 默认 **Deny All**
- Capability 集合（来自 Registry `security_policy` JSONB）：
  - `db.read:<domain>:<query_id>` —— 读已注册的 Domain Query
  - `db.write:<domain>:<mutation_id>` —— 写已注册的 Domain Mutation
  - `event.publish:<subject_pattern>` —— publish 到 subject 模式
  - `event.subscribe:<subject_pattern>` —— subscribe
  - `service.call:<service_id>:<method>` —— 调既有 gRPC
  - `secret.read:<secret_name>` —— 读 secret
  - `object_storage.read:<bucket>` / `object_storage.write:<bucket>`
  - `network.outbound:<domain_pattern>` —— 出站（AI / webhook / 第三方 API）
  - `host.log` / `host.now` / `host.random` —— 基础能力

### 16.2 WASM 隔离

- Per §9：Wasmtime 实例化时 linker 注入严格匹配的 import；未授权 import **不可能**被调用（链接期失败）
- `Store::set_epoch_deadline` 强制 CPU 上限
- `Store::limiter` 强制内存上限
- 模块大小限制：编译时 < 1MB / 运行时实例 < 64MB

### 16.3 mTLS

- Function Gateway → Core Service：mTLS（复用 `shared_platform::tls`）
- Function Gateway → Function Pod：mTLS（同上）
- Function Pod → NATS：NATS 自身 TLS（既有）
- Function Pod → DB：**禁止**直接连（per ARC-008；仅经 Domain API）

### 16.4 Secret 管理

- Secret 存 K8s Secret（per 既有 manifest 09-secret-template.yaml）
- WASM **不**直接读 Secret 文件；通过 `host_get_secret(name)` 由 host 进程代读后注入
- host 进程对 Secret 列表做白名单（来自 Registry `secrets_ref`）
- 任何 Function 越权读 secret → 进程级 ALERT + 自动 pause

### 16.5 Container Function 隔离

- K8s `securityContext`：`runAsNonRoot: true`、`readOnlyRootFilesystem: true`、`allowPrivilegeEscalation: false`
- `capabilities.drop: [ALL]`
- `seccompProfile: RuntimeDefault`
- Pod-level `serviceAccountName: rgs-function-default`（**仅**该 SA；不得用业务域 SA）

### 16.6 Network Policy（per §34）

- 默认 Pod-level `NetworkPolicy`：deny all ingress + egress
- 仅 Function Gateway IP range 允许 ingress
- 出站按 Registry `network_policy` 显式 allow；AI Function 用独立 Egress Policy

---

## 17. 交付物 ⑰《资源隔离设计》

### 17.1 PriorityClass

| PriorityClass | 值 | 用途 | 抢占 |
|---|---|---|---|
| `rgs-p0-realtime` | 1000000000 | 5 业务域 / Realtime（PH-5+） | 不被抢占 |
| `rgs-p1-critical` | 900000000 | cluster-ops / Function Gateway | 不被抢占 |
| `rgs-p2-interactive` | 500000000 | Function Container (Type C, 总是 1+) | 可被 P0/P1 抢占 |
| `rgs-p3-background` | 100000000 | Function Container (Type D) | 可被 P0/P1/P2 抢占 |
| `rgs-p4-batch` | 10000000 | analytics / report | 第一个被抢占 |

### 17.2 Node 资源池

**第一阶段**（小规模，按既有 K3s 1 master + N worker 模式）：**逻辑隔离**

- `PriorityClass`（§17.1）
- 强 Requests / Limits（Function Container limits.cpu=2000m / limits.memory=2Gi 起步）
- Taints / Tolerations：realtime 域不污染，Function 域 `tolerations: [{key:rgs/function, operator:Exists}]`，TBD-MEASURE 实际节点规模 ≥ 6 后再分池

**第二阶段**（≥ 6 节点）：**物理隔离**

- Realtime Pool：2 节点，label `rgs.io/pool=realtime`
- General Pool：3 节点，label `rgs.io/pool=general`
- Batch Pool：1 节点，taint `rgs.io/pool=batch:NoSchedule`
- Function Pod 用 `nodeSelector: {rgs.io/pool: general}` 或 batch（按 priority）

### 17.3 WASM 资源隔离

- per §9.2：每个 host process 限定 instance count / memory / CPU
- 多个域的 host process 各自独立（不共享）—— **不**把 player 域 WASM 和 economy 域 WASM 放同一 host

### 17.4 DB 保护（per §19 + §39）

- Function Pod **不**直连 5 域 DB
- 即使 Type D 扩到 100 Function，也只通过 Function Gateway → Core Service gRPC
- Function Gateway 持有 rate limiter（token bucket per function_id）

---

## 18. 交付物 ⑱《可观测性设计》

### 18.1 指标族（**新增**，全部走既有 Prometheus 导出）

| 指标名 | 类型 | 标签 | 含义 |
|---|---|---|---|
| `function_invocation_total` | counter | func_id, version, runtime, result | Function 调用总数 |
| `function_execution_duration_seconds` | histogram | func_id, version, runtime | 执行耗时 |
| `function_failure_total` | counter | func_id, version, error_kind | 失败总数 |
| `function_cold_start_total` | counter | func_id, version, runtime | 冷启动次数 |
| `function_cold_start_duration_seconds` | histogram | func_id, version, runtime | 冷启动耗时 |
| `function_scale_from_zero_duration_seconds` | histogram | func_id, version | 0→1 扩容耗时 |
| `function_queue_wait_seconds` | histogram | func_id, version | 排队等待 |
| `function_retry_total` | counter | func_id, version, retryable | 重试次数 |
| `function_timeout_total` | counter | func_id, version | 超时次数 |
| `function_oscillation_total` | counter | func_id | 0→1→0 震荡计数 |
| `wasm_execution_duration_seconds` | histogram | func_id, version | WASM 单独执行 |
| `wasm_trap_total` | counter | func_id, version, trap_kind | WASM trap |
| `function_active_pods` | gauge | func_id, version | 当前活跃 Pod 数 |
| `function_in_flight` | gauge | func_id, version | in-flight 请求 |

### 18.2 Trace

- 沿用 `tracing` + OTel SDK（既有）
- Function Gateway 在 Invoke 时从 incoming metadata 提取 `traceparent`，注入到 host process / Wasmtime
- WASM 通过 `host_log_trace` 接收 trace_id 并写 `tracing::Span`
- 跨边界：Service → NATS → Function → DB 全部 span 串联

### 18.3 Log

- 沿用 JSON structured logging（既有 `init_json_logging`）
- Function 日志强制带 `function_id` / `version` / `saga_id` / `command_id` 字段

### 18.4 仪表盘

- 新增 Function Plane 仪表盘（独立 `grafana/dashboards/rgs-function-plane.json`，**不**修改既有 `rgs-services-overview.json`）
- 关键 Panel：invocation rate、cold start P99、queue depth、scale oscillation、wasm trap rate

### 18.5 告警

- `function_failure_total` rate > 10/s 持续 5min → P2
- `function_cold_start_duration_seconds` P99 > 3s 持续 10min → P3
- `function_oscillation_total` 5min 增量 > 10 → P2
- `wasm_trap_total` rate > 1/s → P1
- `function_queue_wait_seconds` P99 > 10s → P2

---

## 19. 交付物 ⑲《数据库保护设计》

### 19.1 不得直连

- Function Pod / WASM **不**持有 5 域 DB 凭据
- Registry 中 **不**登记 DB 凭据
- K8s `NetworkPolicy` 显式 deny Function Pod → `pg-*` Pod 流量

### 19.2 并发控制

- Function Gateway 在 `service.call:<service>:<method>` 路径上加 per-(func_id, method) token bucket
- 默认：单 func_id 单 method 100 RPS / 200 in-flight（可调）

### 19.3 Connection Pool 复用

- Function 调 gRPC 经 `shared_platform::client::build_secure_channel` —— **复用**该 crate 已有的连接池（hyper connection pool 默认开启）
- **禁止** Function 在每次 invoke 新建 channel

### 19.4 Bulkhead Isolation（per §40）

- Function 失败 / 限流 → 仅影响该 func_id 路由；不影响其他 Function
- Function Gateway 对每个 func_id 独立 semaphore + circuit breaker
- WASM host 进程级 trap → 仅该 host 重启；**不**影响同节点其他 host

### 19.5 Circuit Breaker（per §41）

- external API / AI API / webhook / object storage → 独立 circuit breaker
- 配置：error_rate > 50% / 5s window → open；30s 后 half-open
- per-`external_target` 计数
- AI Function 强制走 **独立 AI Function Pool**（per §42）

### 19.6 DB Read / Write 走 Domain API

- 5 域 gRPC 已有 `Get*` / `List*` / `Commit*` / `Apply*` 等方法（per proto/*.proto）
- Function 调这些方法而非直连 DB
- 若 Domain API 缺失 → 需先**为该域扩展** API（**不**为 Function 绕过）

---

## 20. 交付物 ⑳《Cold Start 优化设计》

### 20.1 Container Cold Start

| 阶段 | 优化 | 目标 |
|---|---|---|
| 镜像拉取 | Image 预拉取（DaemonSet `image-prepull`），`imagePullPolicy: IfNotPresent` | 0s（已 cached） |
| 镜像大小 | 共享 distroless base；Function 二进制 < 30MB（Rust static + LTO） | < 1s 解压 |
| 启动 init | Lazy init：DB pool / NATS connection / OTel pipeline 延迟到第一次 invoke | 启动 < 200ms |
| Connection Pool | 预热 pool，per §19.3 | 0 |
| Runtime Pool | Function Pod 启动时加载所有 active module | warm instance < 5ms |

### 20.2 WASM Cold Start

| 阶段 | 优化 | 目标 |
|---|---|---|
| 编译 | precompiled module（cache 到 `pvc://rgs-function-store/`） | 0s（已 compiled） |
| 加载 | Wasmtime `Module::deserialize` 走 mmap | < 50ms |
| 链接 | linker cache（per-capability set） | < 10ms |
| 实例化 | Instance pool warm（默认 16 / module） | < 5ms |
| 大 module 切分 | Component Model + 共享 import | 视模块 |

### 20.3 分阶段计时（per §27 要求）

Function Gateway 在每次 invoke 记录：

```
function_total_duration = scheduling + runtime_init + app_init + dep_init + exec + flush
```

| 子阶段 | 来源 |
|---|---|
| `scheduling_latency` | KEDA / kube-scheduler 决定 |
| `runtime_init` | Container 启动 / Wasmtime deserialize |
| `app_init` | 用户代码 main() |
| `dep_init` | DB pool / NATS connect / OTel init |
| `exec` | 实际业务执行 |
| `flush` | 响应序列化 + 写回 |

各子阶段独立 histogram，**不**仅记总耗时。

### 20.4 Runtime Pool（L1 复用）

- 5 业务域 + cluster-ops 启动时 Wasmtime host 内置 Function Runtime Pool
- 第一次 Invoke 拉起 instance，后续 invoke 复用
- per host 默认 16 instance（可调）
- LRU 淘汰 inactive > 5min 的 instance

---

## 21. 交付物 ㉑《Benchmark 计划》

> 任务 §38 要求做真实 Benchmark 并标 `MEASURED` / `ESTIMATED`。

### 21.1 五种 workload 对照

| Code | Workload | MEASURED / ESTIMATED |
|---|---|---|
| A | Rust Native In-Process（既有 player-service GetCharacter 路径） | MEASURED（既有） |
| B | Rust Always-On Pod（同 A，跨进程） | MEASURED（既有） |
| C | Rust Warm Pod（min=1 HPA） | ESTIMATED（按现有 HPA 行为） |
| D | Rust Container Function（Function Plane 路径） | ESTIMATED（PoC 完成后） |
| E | Rust WASM Function（in-process） | ESTIMATED（PoC 完成后） |

### 21.2 测量指标

| 指标 | 方法 |
|---|---|
| P50 / P95 / P99 latency | 10000 req 压测，k6 驱动 |
| Cold start duration | 杀掉 pod → 立即 invoke → 测首请求 latency |
| Memory | RSS via cgroup / `metrics` crate |
| CPU | `cgroup` cpu.stat |
| Throughput | k6 RPS at P99 < 500ms |
| Max concurrency | 渐增 concurrency 找拐点 |

### 21.3 工具

- 压测：**k6**（per RGS-REQ-015 §5.2 既有选型）
- 资源：cAdvisor / Prometheus
- 分析：Grafana dashboard + ad-hoc

### 21.4 报告格式

```markdown
| Workload | P50 | P95 | P99 | Cold Start | Mem | CPU | Throughput | Status |
|---|---|---|---|---|---|---|---|---|
| A Native | TBD-MEASURE | TBD-MEASURE | ... | 0 | ... | ... | ... | MEASURED |
| B Always-On | ... | ... | ... | 0 (always-on) | ... | ... | ... | MEASURED |
| C Warm | ... | ... | ... | TBD-MEASURE | ... | ... | ... | ESTIMATED |
| D Container Fn | TBD | TBD | TBD | TBD | TBD | TBD | TBD | ESTIMATED |
| E WASM Fn | TBD | TBD | TBD | TBD | TBD | TBD | TBD | ESTIMATED |
```

> 报告落地：`docs/09-部署运维/RGS-BENCH-001_Function_Plane_Benchmark_v0.1.md`（**新增**）
> **禁止**混合 MEASURED 与 ESTIMATED；同一行只允许一个来源。

### 21.5 准入门槛（Function 化 PoC 必须满足）

- D vs B：P99 latency overhead < 200ms（Function 冷启动可接受）
- E vs B：P99 latency overhead < 50ms（in-process WASM 应接近 native）
- E vs D：冷启动快 > 5×
- E memory：< 64MB / instance
- 任何一项不达标 → 不进入 production rollout

---

## 22. 交付物 ㉒《PoC 计划》

### 22.1 PoC 范围（最小风险、最易验证）

**PoC-1（最低风险）**：`rgs-hello` + `rgs-certgen` 改造为 Function
- 已有 `SVC-CG-001` / `SVC-HL-001` 单一职责
- 风险最低（无外部依赖 / 无业务影响）
- 验证目标：Function Plane 全链路（Registry / Gateway / WASM 或 Container / 观测 / Rollback）
- Feature Flag：`ENABLE_FUNCTION_RUNTIME=true`
- 周期：2 周

**PoC-2（业务验证）**：`achievement.calculate`（FN-CAND-001）
- 真实业务路径
- 验证：Saga Step + Function 集成（per §14）
- 验证：Tier 3 NATS trigger + backpressure
- Feature Flag：`ENABLE_FUNCTION_RUNTIME=true` + `achievement.calculate.enabled`
- 周期：3 周

**PoC-3（弹性验证）**：`notification.send`（FN-CAND-002）+ `rgs-hello.smoke` 流量模拟
- 验证 Scale-to-Zero（0→1→0）
- 验证 KEDA 集成
- 周期：2 周

### 22.2 PoC 退出条件（任意一条不满足则延期 production rollout）

- 所有 PoC-1/2/3 在 staging 跑通 14 天无 P1 故障
- Benchmark §21 全部 5 类 workload 报告完成 + 准入门槛通过
- Feature Flag 切回 false 时，系统行为与改造前**完全一致**（对比测试）
- Rollback 演练通过：PoC 切回 false + 删 Function Pod 30s 内无错误

### 22.3 PoC 不在范围内

- 改造 player / economy / match —— **不**进入 PoC
- 任何对 ARC-005 / ARC-007 / ARC-008 / ARC-021 的修改 —— **不**进入 PoC
- Knative / OpenFaaS 引入 —— **不**进入 PoC

---

## 23. 交付物 ㉓《迁移计划》

### 23.1 11 阶段（per §44）

| Phase | 名称 | 周期（估） | 关键产物 | 准入 |
|---|---|---|---|---|
| 0 | Architecture Inventory | 1 周 | 本文档 RGS-INC-001 | 现状基线冻结 |
| **0.5** | **5 业务域 K3s 部署基线**（**新插入，v0.2 勘误**） | 2~3 周 | (1) 5 Deployment + 5 Service manifest 实际值落地 (2) NATS JetStream K8s Deployment + Service (3) OTel Collector + Prometheus + Grafana 3 套 K3s manifest (4) mTLS 证书签发 + Secret 注入 (5) docker image 构建流水线落地 + registry (6) end-to-end smoke test（5 业务域 HealthCheck OK + trace_id 跨域串联） | (a) B-CODE-01~04 全部 Closed（per `../deploy/07-no-go-checklist_business_v0.1.md`） (b) `cargo test --workspace` 全跑 PASS（含 9 crate，不只 rgs-hello） (c) cargo-deny / cargo-audit / cargo-llvm-cov 全部安装 + PASS (d) helm v3.10+ 安装 + 至少一次 dry-run 通过 |
| 1 | Benchmark | 2 周 | RGS-BENCH-001 | 5 类 workload 数据齐 |
| 2 | Function Contract | 1 周 | `rgs-function-contract` crate + Registry schema PR | API 冻结 |
| 3 | WASM Runtime PoC | 3 周 | `rgs-function-host` crate + Wasmtime 集成 | 1 个 WASM Function 跑通 |
| 4 | Event Worker Scale-to-Zero | 3 周 | KEDA ScaledObject + 1 个 Type C Function | min=0 稳定 |
| 5 | Container Function | 2 周 | `rgs-function-gateway` 完整 + Type D Function | 1 个 Container Function 跑通 |
| 6 | Function Registry | 2 周 | cluster_ops_db.function_registry + gRPC API | 注册/查询/版本管理通 |
| 7 | Saga Integration | 2 周 | economy-service SagaStepHandler 扩展 StepKind | 1 个 Function Step 端到端 |
| 8 | Observability Integration | 1 周 | §18 全部指标 + Dashboard + Alert | Grafana 显示齐 |
| 9 | Workload Placement Recommendation | 2 周 | `rgs-placement` CLI | Recommendation 输出 |
| 10 | Production Rollout | 持续 | FN-CAND-* 逐个迁入 | 灰度比例 0→10%→50%→100% |

**总周期估算**：~21 周（~5 个月），不含 Phase 10 长期灰度。

> Phase 0.5 实际周期取决于 5 业务域 Lead 联合校准 K8s manifest 实际值（resources / 副本数 / 镜像 tag / env），**预估 2~3 周**。**这是 Phase 1 启动的硬阻塞**：5 业务域 Pod 不 running，Function 化所有 gRPC 调用路径将不可达。

### 23.2 依赖关系

```
Phase 0 (本) 
   ↓
Phase 0.5 5 业务域 K3s 部署基线  ←─ 硬阻塞
   ↓
Phase 1 (Bench) 
   ↓
Phase 2 (Contract) 
   ├→ Phase 3 (WASM) 
   └→ Phase 5 (Container)
          ↓
       Phase 4 (Scale-to-Zero) ── 依赖 Phase 5
          ↓
       Phase 6 (Registry) ── 依赖 Phase 5
          ↓
       Phase 7 (Saga)
          ↓
       Phase 8 (Observability)
          ↓
       Phase 9 (Placement Recommendation)
          ↓
       Phase 10 (Production Rollout)
```

### 23.3 灰度策略

- 每个 Function 上线：先 internal → 1% → 10% → 50% → 100%
- 灰度期间：双 Registry 记录（active + canary），Function Gateway 按 `traffic_pct` 切流
- 回滚：5min 内通过 `status='paused'` + `traffic_pct=0` 完成

### 23.4 文档更新义务

- 每个 Phase 结束时必须更新：
  - 既有 crate 的 README（受影响）
  - `docs/01-核心架构与设计模式/` 下新增 DTL（详细设计）
  - `docs/12-治理/RGS-WBS-001_*.md` WBS DAG 更新
  - `docs/08-架构决策记录/` 新增 / 更新 ADR
- **Phase 0.5 结束时必须更新**：
  1. `docs/deploy/01-k8s-manifests/_status.md`（11 个 manifest 从占位 → 🟢）
  2. `../deploy/07-no-go-checklist_business_v0.1.md`（4 条 B-CODE 从 🔴 → 🟢）
  3. `../deploy/08-measure-env-setup.log` 追加 Section 7 "5 业务域 Pod status"
- CI 强制：`docs-ci.yml` 校验链接 / TOC / 编号

---

## 24. 交付物 ㉔《Rollback 计划》

### 24.1 Feature Flag（**强制**，每阶段必带）

| Flag | 默认 | 控制 |
|---|---|---|
| `ENABLE_FUNCTION_RUNTIME` | false | 整个 Function Plane 开关 |
| `ENABLE_WASM_RUNTIME` | false | WASM Runtime 子开关 |
| `ENABLE_SCALE_TO_ZERO` | false | 0→1→0 缩容到零 |
| `ENABLE_DYNAMIC_PLACEMENT` | false | 自动迁移（始终默认关） |
| `ENABLE_KEDA` | false | KEDA 接入 |
| `<FUNC_ID>.enabled` | false | per-function 开关（Registry 字段） |
| `<FUNC_ID>.canary_pct` | 0 | canary 比例 |

**实现位置**：
- 全局：K8s ConfigMap `rgs-function-config`（per namespace `rgs-function`）
- per-function：Registry 表字段
- Function Gateway 启动时加载 + 5s SIGHUP 重载

### 24.2 Rollback 流程

| 触发 | 动作 | 恢复时间 |
|---|---|---|
| Function 故障影响业务 | `ENABLE_FUNCTION_RUNTIME=false` + 该 func `enabled=false` | < 30s（K8s ConfigMap apply + Gateway 重载） |
| KEDA 自身 bug | `ENABLE_KEDA=false` + 删 ScaledObject | < 1min |
| WASM Runtime bug | `ENABLE_WASM_RUNTIME=false` + 重启 host | < 30s |
| Function Gateway bug | 切回既有路径（Function 走 HTTP 直连 Core Service） | < 1min（需提前实现 fallback） |
| Schema 不兼容 | Registry 表新增字段不删字段；旧 Gateway 仍可读 | 0（向后兼容） |

### 24.3 数据库兼容

- function_registry 表**新增**字段（不删）
- 既有 5 域 DB 表**不**改
- outbox 表**不**改
- 任何 Schema 变更走既有 migration 流程（`sqlx::migrate!`）

### 24.4 Saga 兼容

- `StepKind` 新增变体**不**影响既有 Service Step
- 旧 `Saga` 数据（只含 Service Step）继续工作
- Rollback 时：Function Step 的 Saga 行（`status=Function, Function, Wasm`）仍可被旧 Orchestrator 读取（`step_kind` 字段默认 `'Service'`）

---

## 25. 交付物 ㉕《风险登记表》

| 风险 ID | 描述 | 概率 | 影响 | 缓解 | Owner |
|---|---|---|---|---|---|
| RISK-INC-001 | KEDA 引入 5 域 OLU 不足 | 低 | 中 | §11.3 OLU 申领 + 季度 review | SRE Lead |
| RISK-INC-002 | WASM Cold Start 不可达 50ms 目标 | 中 | 中 | §9.2 + §20.2 优化；不达标则退化为 Container Function | Platform Lead |
| RISK-INC-003 | Function 直连 DB（绕过 Gateway） | 中 | 高 | §16 + §19 NetworkPolicy + RBAC；CI 审计 wasm module 字节码 | 安全 Lead |
| RISK-INC-004 | Function Scale 震荡 (0→1→0→1) | 中 | 中 | §10.3 cooldown + minimum residency + oscillation 监控 | SRE Lead |
| RISK-INC-005 | AI Function 抢实时 CPU | 中 | 高 | §17 PriorityClass + §42 AI 独立 Pool + 限流 | 架构师 |
| RISK-INC-006 | OLU 超 NFR-OP-010 2 SRE·周 | 中 | 高 | §11.3 + 季度 review + 任何新组件必须先 ADR | SRE Lead |
| RISK-INC-007 | Function 调用 Core Service 性能开销被低估 | 中 | 中 | §21 Benchmark 强制；不达标则降为 Warm Service | 5 域 Lead |
| RISK-INC-008 | Registry 误配导致 Function 越权 | 低 | 高 | §15 + §16 + §33 capability 静态检查 + CI 审查 | 安全 Lead |
| RISK-INC-009 | WASM module 供应链攻击 | 低 | 极高 | §9.4 cosign 验签 + 私有 registry | 安全 Lead |
| RISK-INC-010 | Saga 跨 Function 事务一致性破坏 | 低 | 高 | §14 单一调解者不变 + Function Step 仅 execute/compensate | 经济域 Lead |
| RISK-INC-011 | 既有 mTLS fail-closed 与 Function Gateway 冲突 | 低 | 中 | §16.3 复用 shared_platform::tls | Platform Lead |
| RISK-INC-012 | 引入 KEDA / Wasmtime 后 OTel 链路断裂 | 中 | 中 | §18.2 traceparent 透传强制 + 端到端测试 | SRE Lead |
| RISK-INC-013 | 命名空间 / RBAC 误配导致 Function 跨域访问 | 中 | 高 | §17 + §33 deny-all default + 季度审计 | 安全 Lead |
| RISK-INC-014 | Cold Start 慢导致首请求 SLA miss | 中 | 中 | §20 + §24 fuel/timeout 双重保险 | 5 域 Lead |
| RISK-INC-015 | 文档/Saga/Contract 不一致 | 中 | 中 | §23.4 文档更新义务 + `docs-ci.yml` | 架构师 |
| RISK-INC-016 | 实时主路径被误接 Function | 低 | 极高 | §3.2 + §4 + §6 + 强制 Review by Realtime Engineer | 架构师 + Realtime Eng |

---

# 第四块 治理产出

## 26. 交付物 ㉖《ADR 列表》

> 既有 ADR 不重复登记。下列为本方案落地**需要**新增 / 修订的 ADR：

| ADR | 标题 | 状态 | 关联 |
|---|---|---|---|
| ADR-0055 | 引入 Function Plane（含 KEDA + Wasmtime）—— ARC-014 三条件 | 草拟 | ARC-014 / ARC-026 / §11 / §8 / §9 |
| ADR-0056 | 引入 Wasmtime 作为 WASM Runtime（兑现 ADR-0020 §3.2 留口） | 草拟 | ADR-0020 / §7 / §9 |
| ADR-0057 | Function 调用 Core Service 走 Domain API，不直连 DB | 草拟 | ARC-007 / ARC-008 / §8.3 / §19 |
| ADR-0058 | Function Registry 落 cluster_ops_db（不新建 registry 服务，遵循 ARC-014 既有约定） | 草拟 | RGS-TS-001 §3.6.2 / §15 |
| ADR-0059 | WASM 沙箱 capability 白名单 + module 验签（cosign） | 草拟 | §9.3 / §16.6 / §9.4 |
| ADR-0060 | Function 参与 Saga：StepKind 扩展，调解者仍为 economy 域 | 草拟 | ADR-0015 / §14 |
| ADR-0061 | Scale-to-Zero 防震荡策略（hysteresis / cooldown / minimum residency） | 草拟 | §10.3 |
| ADR-0062 | 三级事件路由（T1 / T2 / T3）+ 通道选择规则 | 草拟 | §12 |
| ADR-0063 | PriorityClass 与 Node Pool 划分原则 | 草拟 | §17 |
| ADR-0064 | 拒绝 Knative / OpenFaaS 引入（基于 ARC-014） | 草拟 | §11 |
| ADR-0065 | （**未来**）AI Function Pool 独立隔离 | 草拟 | §42 |
| ADR-0066 | （**未来**）Workload Placement Auto Mode 第二阶段 | 草拟 | §28 |

---

## 27. 交付物 ㉗《需求追踪矩阵》

> ID 体系：SLS-（Serverless 总体）/ WASM-（WASM Runtime）/ SCALE-（Scale-to-Zero）/ EVENT-（事件路由）/ RUNTIME-（Function Runtime 容器）/ OBS-（观测）/ SEC-（安全）/ REG-（Registry）/ SAGA-（Saga 集成）/ PERF-（性能 / Cold Start）/ DB-（DB 保护）/ PLACE-（Workload Placement）/ OLU-（运维负荷）/ NODE-（Node 资源池）/ ROLL-（Rollback）/ MIGR-（迁移）/ REPLAY-（回放 PoC）。

### 27.1 需求清单

| REQ ID | 描述 | 优先级 | 关联设计 | 关联 ADR | 关联测试 |
|---|---|---|---|---|---|
| SLS-REQ-001 | Function Plane 整体建立 | P0 | §8 | ADR-0055 | TST-INC-001 |
| SLS-REQ-002 | Function Gateway 同时支持 gRPC + HTTP | P0 | §8.1 | ADR-0055 | TST-INC-002 |
| SLS-REQ-003 | Function 不得绕过 Function Gateway 直连 Core | P0 | §19.1 | ADR-0057 | TST-INC-003 |
| SLS-REQ-004 | Function 默认 deny all capability | P0 | §16.1 | ADR-0055 | TST-INC-004 |
| WASM-REQ-001 | Wasmtime 集成 + 资源保护（fuel / memory / timeout） | P0 | §9 | ADR-0056 | TST-INC-005 |
| WASM-REQ-002 | WASM 仅能调 host 白名单 import | P0 | §8.3 / §9.3 | ADR-0059 | TST-INC-006 |
| WASM-REQ-003 | WASM module 走 cosign 验签 | P1 | §9.4 | ADR-0059 | TST-INC-007 |
| WASM-REQ-004 | WASM in-process pool 复用（warm instance） | P1 | §20.2 | ADR-0056 | TST-INC-008 |
| SCALE-REQ-001 | Function 0→N→0 Scale-to-Zero | P0 | §10 | ADR-0061 | TST-INC-009 |
| SCALE-REQ-002 | hysteresis / cooldown / minimum residency 防震荡 | P0 | §10.3 | ADR-0061 | TST-INC-010 |
| SCALE-REQ-003 | Scale 决策可观测（oscillation 计数器） | P1 | §18.1 | ADR-0061 | TST-INC-011 |
| EVENT-REQ-001 | 三级事件路由（T1/T2/T3） | P0 | §12 | ADR-0062 | TST-INC-012 |
| EVENT-REQ-002 | Function 触发使用 Tier 3 subject 命名 | P0 | §12.2 | ADR-0062 | TST-INC-013 |
| EVENT-REQ-003 | Function 内部 cross-call 走 Internal Channel（不走 NATS） | P1 | §12.2 | ADR-0062 | TST-INC-014 |
| EVENT-REQ-004 | 实时主路径不经过 NATS 同步等待 | P0 | §3.2 / §4 | ADR-0062 | TST-INC-015 |
| RUNTIME-REQ-001 | Function 双 Runtime（WASM + Container）共存 | P0 | §6 / §8 | ADR-0055 | TST-INC-016 |
| RUNTIME-REQ-002 | 运行时选择走 Registry policy | P0 | §13.2 | ADR-0055 | TST-INC-017 |
| RUNTIME-REQ-003 | Container Function 共享 distroless base image | P1 | §20.1 | ADR-0055 | TST-INC-018 |
| OBS-REQ-001 | §18 全部 Prometheus 指标实现 | P0 | §18 | — | TST-INC-019 |
| OBS-REQ-002 | Trace 跨 Service/Function/DB 全链路 | P0 | §18.2 | — | TST-INC-020 |
| OBS-REQ-003 | Function 仪表盘独立 | P1 | §18.4 | — | TST-INC-021 |
| SEC-REQ-001 | WASM capability 静态检查（CI 强制） | P0 | §16.1 | ADR-0059 | TST-INC-022 |
| SEC-REQ-002 | Function NetworkPolicy deny-all default | P0 | §16.6 | ADR-0059 | TST-INC-023 |
| SEC-REQ-003 | mTLS 全覆盖（Gateway ↔ Core / Gateway ↔ Function Pod） | P0 | §16.3 | — | TST-INC-024 |
| SEC-REQ-004 | Function Secret 经 host 代读 | P1 | §16.4 | ADR-0059 | TST-INC-025 |
| REG-REQ-001 | Function Registry Schema（§15.2）实现 | P0 | §15 | ADR-0058 | TST-INC-026 |
| REG-REQ-002 | Registry 版本管理 + Canary | P1 | §15 / §31 | ADR-0058 | TST-INC-027 |
| REG-REQ-003 | Registry 走 cluster_ops gRPC（不新建服务） | P1 | §15.3 | ADR-0058 | TST-INC-028 |
| SAGA-REQ-001 | SagaStepHandler::step_kind() 默认 Service 兼容 | P0 | §14 | ADR-0060 | TST-INC-029 |
| SAGA-REQ-002 | Function Step 走 economy 域 Saga 持久化 | P0 | §14 | ADR-0060 | TST-INC-030 |
| SAGA-REQ-003 | 不得创建第二套事务体系 | P0 | §14.3 | ADR-0060 | TST-INC-031 |
| PERF-REQ-001 | 5 类 workload Benchmark 报告 | P0 | §21 | — | TST-INC-032 |
| PERF-REQ-002 | Cold Start 分解计时（scheduling / runtime / app / dep） | P0 | §20.3 | — | TST-INC-033 |
| PERF-REQ-003 | Function 化准入门槛（§21.5） | P0 | §21.5 | — | TST-INC-034 |
| DB-REQ-001 | Function 不得持有 5 域 DB 凭据 | P0 | §19.1 | ADR-0057 | TST-INC-035 |
| DB-REQ-002 | Function Gateway 持 per-(func, method) rate limit | P0 | §19.2 | — | TST-INC-036 |
| DB-REQ-003 | NetworkPolicy deny Function → pg-* | P0 | §19.1 | ADR-0057 | TST-INC-037 |
| PLACE-REQ-001 | WPE 第一阶段 = Recommendation Only | P0 | §13.3 | — | TST-INC-038 |
| PLACE-REQ-002 | 不得自动迁移生产工作负载 | P0 | §13.3 | ADR-0066 | TST-INC-039 |
| OLU-REQ-001 | KEDA 引入必须申领 OLU | P0 | §11.3 | ADR-0055 | TST-INC-040 |
| OLU-REQ-002 | 任何新组件先过 ARC-014 | P0 | ADR-0008 | — | TST-INC-041 |
| NODE-REQ-001 | PriorityClass §17.1 全部实现 | P0 | §17.1 | ADR-0063 | TST-INC-042 |
| NODE-REQ-002 | 第一阶段逻辑隔离，6 节点后物理隔离 | P1 | §17.2 | ADR-0063 | TST-INC-043 |
| ROLL-REQ-001 | Feature Flag 全部实现 | P0 | §24.1 | — | TST-INC-044 |
| ROLL-REQ-002 | Function 切换 < 30s | P0 | §24.2 | — | TST-INC-045 |
| MIGR-REQ-001 | 11 阶段迁移路线严格执行 | P0 | §23 | — | TST-INC-046 |
| MIGR-REQ-002 | 每阶段文档更新义务 | P0 | §23.4 | — | TST-INC-047 |
| REPLAY-REQ-001 | PoC-1（certgen + hello）2 周完成 | P0 | §22.1 | ADR-0055 | TST-INC-048 |
| REPLAY-REQ-002 | PoC-2（achievement.calculate）3 周完成 | P0 | §22.1 | ADR-0060 | TST-INC-049 |
| REPLAY-REQ-003 | PoC-3（notification.send + scale-to-zero）2 周完成 | P0 | §22.1 | ADR-0061 | TST-INC-050 |

### 27.2 全链路追踪示例（以 WASM-REQ-001 为例）

```
需求: WASM-REQ-001 Wasmtime 集成 + 资源保护
  ↓
基本设计: RGS-INC-001 §9 交付物 ⑨
  ↓
详细设计: docs/01-核心架构与设计模式/RGS-DTL-051_Wasmtime_Runtime_详细设计_v0.1.md (Phase 3 产出)
  ↓
代码: crates/rgs-function-host/src/wasmtime_pool.rs + linker.rs + limits.rs
  ↓
测试: TST-INC-005 / TST-INC-006 / TST-INC-008
  ↓
验收: §21 Benchmark 准入门槛 + §22 PoC 退出条件
```

---

# 附录 A：§49 第二轮自审（8 角色）

## A.1 Realtime Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §6 把 social/admin 划为 Type B 是合理的，但需明确 "Warm 1→N 不等于任意缩到 0"——已是方案 §10.1 设定 |
| 性能风险 | §4 严格保护实时主路径已落实；仍需 CI 强制 `git grep` 检查无"实时主路径 → NATS / Function"反模式 |
| 运维风险 | Realtime 域**不**进入 Function Plane，新增 KEDA 不影响——已落实 |
| 安全风险 | 实时主路径仍走 mTLS gRPC，无新增面 |
| 可维护性风险 | §27 需求矩阵覆盖充分 |
| 可删除组件 | **无** |

**结论**：✅ 通过

## A.2 Rust Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §8.1 列的 `rgs-function-host` 与 `rgs-function-gateway` 是两个独立 crate——可考虑合并为单一 `rgs-function-runtime`（共享 host 进程） |
| 性能风险 | §9.2 epoch_interruption 1ms tick 可能在重负载下造成非预期中断——需提供更粗粒度（10ms）作为 fallback |
| 运维风险 | WASM module 链 napi-rs / wasmtime 升级路径复杂，**需要** 锁版本 + 半年一次评估 |
| 安全风险 | §9.4 cosign keyless 在私网 registry 上需要额外工作 |
| 可维护性风险 | §21 Benchmark 工具链需独立于既有 k6 配置 |
| 可删除组件 | §13.3 WPE 第一阶段如不投入足够测试，可推迟到 Phase 9 之后 |

**结论**：⚠ 条件通过（合并 gateway+host；epoch 粒度 fallback；cosign 私网评估）

## A.3 Kubernetes Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §11.2 ScaledObject 用 `nats-jetstream` scaler 是 KEDA 内置（>= 2.10），需要确认 K3s 默认版本兼容 |
| 性能风险 | KEDA 引入 metrics-apiserver 拉取 1min 一次，**可能**导致 scale 决策有 1min 延迟——若不可接受需自研 scaler |
| 运维风险 | §17.2 第二阶段物理分池需 ≥ 6 节点，目前 K3s 1 master + N worker 不一定满足 |
| 安全风险 | Function Pod SA `rgs-function-default` 需要 RBAC 严格 deny 5 域 API |
| 可维护性风险 | KEDA CRD 升级破坏性变更历史较多——季度评估 |
| 可删除组件 | §10 Scale-to-Zero Controller 中"自研 Operator 替代 KEDA"已删除，符合 ARC-014 |

**结论**：⚠ 条件通过（KEDA 版本确认；分池条件写明）

## A.4 SRE

| 项 | 评价 |
|---|---|
| 不合理点 | §11.3 OLU 估算 ~0.3 SRE·周/持续 是"理论值"——实际可能到 0.5（含排障/升级） |
| 性能风险 | §21 Benchmark 5 类 workload 完整做完需要 ~2 周人时；需协调 |
| 运维风险 | Phase 0→10 总周期 21 周，叠加 5 域 Lead 评审，**总 OLU 可能突破 NFR-OP-010** |
| 安全风险 | §25 风险表覆盖 |
| 可维护性风险 | 文档 / Feature Flag / Rollback 流程必须演练 |
| 可删除组件 | §13.4 反向决策可推迟到 Phase 9 后再实化 |

**结论**：⚠ 条件通过（总 OLU 评审 + §13.4 推迟 + 季度复盘）

## A.5 Security Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §16.1 capability 集合设计良好，但**需要** static analyzer（CI）防止 import 未登记 |
| 性能风险 | §19.3 connection pool 复用是必须的，但需 cap 防止内存膨胀 |
| 运维风险 | §9.4 cosign keyless 在私网需额外 PKI 准备 |
| 安全风险 | §25 RISK-INC-003 / 008 / 009 / 013 全部纳入；**新增** RISK-SEC-001：Function Secret 内存 dump 风险——host process dump 时 secret 暴露 |
| 可维护性风险 | RBAC + NetworkPolicy 双层防御需定期审计 |
| 可删除组件 | 无 |

**新增风险**：RISK-SEC-001 host 进程 secret 内存 dump → 缓解：mlock + disable core dump + 定期 secret 轮换

**结论**：⚠ 条件通过（新增 RISK-SEC-001 + §16.4 mlock）

## A.6 Database Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §19 严格 deny Function → DB 是正确的；但 Function Gateway 调 Core Service 高 QPS 时**仍可能**打满 5 域 DB（虽然优于直连）——需要 Gateway 端 rate limit 监控 |
| 性能风险 | §19.2 token bucket 默认 100 RPS/method 偏保守，可能需要按 method 调高（如 GetCharacter 1000 RPS） |
| 运维风险 | Function Gateway 持有 5 域连接池，需要 §19.3 cap（默认 50/域） |
| 安全风险 | §19.1 NetworkPolicy 需 NetworkPolicy Engineer 实施 |
| 可维护性风险 | Function → Gateway → Core → DB 链路增加 1 跳，调试难度增加 |
| 可删除组件 | 无 |

**结论**：✅ 通过

## A.7 Distributed Systems Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §14 Saga Function Step 保持 economy 域单一调解者——正确 |
| 性能风险 | §14 Function Step 在 Saga 中是异步调用，**可能导致** Saga 时延变长——需 timeout 严格设置 |
| 运维风险 | §12 Event Routing 三级路由清晰，但需 §12.4 内部通道有 backpressure 监控 |
| 安全风险 | §16 覆盖 |
| 可维护性风险 | §27 需求矩阵 + §25 风险表覆盖 |
| 可删除组件 | §12.2 中 "Internal Channel 走 tokio mpsc" 可考虑改用 tokio::sync::watch（更轻） |

**结论**：✅ 通过（mpsc 改 watch 评估）

## A.8 Cost Engineer

| 项 | 评价 |
|---|---|
| 不合理点 | §11.3 OLU 估算合理；§17 Node 池暂逻辑隔离是务实选择 |
| 性能风险 | Function Plane 引入对集群总资源**净增**约 200-500MB mem + 300m CPU（控制面） |
| 运维风险 | 若 Phase 10 production rollout 后 idle 资源减少 30%+，ROI 显著；若 < 10%，应回退 |
| 安全风险 | — |
| 可维护性风险 | §37 Resource Efficiency Dashboard **未实化**——本方案未给具体落地物 |
| 可删除组件 | §13 WPE 第二阶段可控不投入 |

**新增要求**：§37 Resource Efficiency Dashboard 必须在 Phase 8 同步实化（**新增需求** `COST-REQ-001`）

**结论**：⚠ 条件通过（§37 必须 Phase 8 同步实化）

---

# 附录 B：§50 九问自答

### Q1：当前哪些现有服务必须继续保持常驻？

| Service | 结论 |
|---|---|
| **player-service** | **必须常驻**（ARC-005 / ARC-007 实时主路径） |
| **economy-service** | **必须常驻**（ARC-006 / ARC-009 永久事实 + Saga 编排） |
| **match-service** | **必须常驻**（撮合 + PH-5+ 实时路径） |
| **cluster-ops** | **必须常驻**（ADR-0052 Active-Active + feature flag 集群级只读） |
| **social-service** | 可 Warm 1→N（**当前**未达 Scale-to-Zero 标准） |
| **admin-service** | 可 Warm 1→N（**当前**未达 Scale-to-Zero 标准） |
| **rgs-certgen** | 可 Function 化（**已识别的低风险 PoC**） |
| **rgs-hello** | 可 Function 化（**已识别的低风险 PoC**） |

### Q2：哪些现有服务最值得首先改造成 Scale-to-Zero？

**当前规模下**（业务量尚未达到 PH-5 Realtime Runtime 实化阶段）：

- **rgs-certgen** + **rgs-hello**（§22.1 PoC-1）—— **最值得**，风险最低
- **admin-service** 中的"导出类 GM 操作"（FN-CAND-003）—— **值得**，QPS 极低
- **notification.send**（**新增** FN-CAND-002）—— **值得**，但需先建 notification 通道
- **achievement.calculate**（FN-CAND-001）—— **值得**，纯异步路径

**不建议**先 Scale-to-Zero：player / economy / match / cluster-ops（**理由** 见 Q1）。

### Q3：哪些业务更适合 WASM 而不是独立 Pod？

| 业务 | 类型 | 原因 |
|---|---|---|
| 手续费 / 抽成规则 | WASM (WASM-CAND-001) | 频繁执行 + 跨域共享 + 版本热升级 |
| ELO 段位算法 | WASM (WASM-CAND-002) | 实时但小逻辑 |
| COC 审计策略 | WASM (WASM-CAND-003) | 频繁 + 热修复 |
| 规则 DSL | Rhai（**不动**，per ADR-0020） | 已有 |

**不**适合 WASM：5 域 gRPC service / Outbox 消费端 / 任何需要长生命周期状态的服务。

### Q4：哪些地方不应该使用 NATS JetStream？

| 场景 | 不用 NATS 原因 |
|---|---|
| 实时主路径 | per §3.2 / §4 + ARC-007 |
| WASM 内部 cross-call | 走 Internal Channel（per §12.2） |
| 域内 gRPC 调用 | 直连 gRPC |
| 简单 HTTP 同步 | 直接 HTTP |
| DB 直读 | per ARC-007 禁止 |
| 5 业务域 hot path 内 | 走 gRPC；NATS 仅承担异步出域事件 |

**注意**：任务文提"所有 Event 都走 Kafka"是 anti-pattern（per §48 Anti-Pattern 6）；本方案按真实代码用 NATS，并强制 §12 三级路由。

### Q5：当前规模是否真的需要 Knative？

**不需要**（per §11.1 评估）：

- 拒绝理由：与 §4 实时主路径冲突；CRD/Controller 数量过多；OLU 占比 > 5%；ARC-014 三条件不通过（既有 K3s + KEDA + 自建 WASM host 可承担）
- 替代方案：KEDA（仅 scaler）+ 自建 WASM host（in-process）

### Q6：KEDA 是否已经足够？

**对 Container Function 是的**：

- Kafka / NATS / Prometheus / Cron 各类 scaler 成熟
- 与既有 mTLS / Prometheus / Grafana 集成路径清晰
- OLU 占比 < 5%

**对 WASM Function 不需要**（in-process scale，K8s 层无 Pod 变化）

**结论**：KEDA 足够，本方案不引入 Knative。

### Q7：是否可以通过 Rust + Wasmtime 自建更轻量的 Function Runtime？

**是**，且是本方案主路径（per §8 / §9）：

- 不引入 wasmCloud / Fermyon Spin
- 仅 `wasmtime` crate + 自建 host + Registry
- 优势：与既有 Rust workspace 共享 shared-platform（mTLS / tracing / metrics / RBAC）；OLU 极低；故障域清晰
- 风险：自建部分需充分测试（PoC-1 验证）

### Q8：引入 Function 后预计可以减少多少空闲资源？

> §50 明确 "没有真实数据时不得给出虚假数字，而应给出测量方法"。

**测量方法**（落地于 Phase 1 Benchmark）：

1. **测量前**（baseline）：抓 7 天 idle 期（凌晨 03:00-05:00 UTC）的
   - player/economy/match/social/admin/cluster-ops Pod 实际 CPU/内存使用（cAdvisor via Prometheus）
   - NATS JetStream 流量
   - 总节点资源使用
2. **PoC 完成后**（FN-CAND-001/002/003/004 + WASM-CAND-001）：
   - 同方法抓 7 天
   - 对比 idle 资源差
3. **报告**：`docs/09-部署运维/RGS-BENCH-002_Resource_Efficiency_v0.1.md`
4. **判定**：若 idle 资源减少 ≥ 10% → 继续 Phase 10；< 10% → 评估是否回退

**当前无法给出具体数字**（TBD-MEASURE，需 Phase 1 实测）。

### Q9：引入 Serverless 后是否降低了系统复杂度？

**分层回答**：

| 维度 | 评价 |
|---|---|
| 业务侧（开发者） | **降低**（不再为 30 行逻辑开 1 个 Deployment） |
| 控制面 | **增加**（KEDA + Function Gateway + WASM host + Registry，约 4 个新组件） |
| 观测 | **持平**（复用 Prometheus / OTel / Grafana） |
| 排障 | **略增**（链路 +1 跳） |
| 文档 / 治理 | **略增**（本方案新增 ~15 份设计文档 + 12 个 ADR 草拟） |

**综合**：引入初期复杂度**净增**约 15-20%；PoC 完成且仅小批量 Function 迁入后，复杂度**净减**约 5-10%（仅当 FN-CAND-001/002/003/004 + WASM-CAND-001 实际承担负载）。

**若 §21 Benchmark 显示**：

- Function 化资源节省 < 10% **且** 复杂度增加 > 15% → **回退** §22 PoC 范围之外的所有 Function 化
- 资源节省 > 20% → 继续 Phase 10
- 中间区 → Lead 评审决策

**这意味着 Q9 的答案取决于实测**——本方案不预设结论。

---

# 附录 C：本文档遵循与偏离的既有 ARC/ADR 索引

| 既有规范 | 关联本文档段落 | 关系 |
|---|---|---|
| ARC-005 session_epoch Single-Writer | §3 / §6 / §27 SLS-REQ-003 | 遵循（**Function 不得伪造**） |
| ARC-006 ACK in persistence | §3 / §14 | 遵循 |
| ARC-007 运行时⇔业务服务边界 | §3 / §6 / §19 / §27 DB-REQ-001 | 遵循 |
| ARC-008 5 独立 DB | §3 / §19 / §27 SLS-REQ-003 | 遵循 |
| ARC-014 中间件导入判定基准 | §11 / §26 ADR-0055/56/57/58/59/60/61/62/63/64/65/66 | 遵循（**新组件必须先 ADR**） |
| ARC-021 故障隔离 | §17 / §40 Bulkhead | 遵循 |
| ARC-026 OLU 预算 | §11.3 / §25 / 附录 A.4 | 遵循 |
| ARC-001 场景 Actor 不迁移 | §6 / §17 Realtime Pool | 遵循 |
| ADR-0008 中间件导入判定 | §11 / §26 | 遵循 |
| ADR-0015 Saga 单一调解者 | §14 | 遵循（**不创建第二套事务**） |
| ADR-0020 拒绝 dlopen / WASM 留口 | §5 / §9 / §26 ADR-0056 | 兑现（WASM 作为升级路径） |
| ADR-0025 OLU 预算 | §11.3 / §25 | 遵循 |
| ADR-0052 Active-Active ClusterOpsService | §6 | 遵循 |
| RGS-TS-001 §3.7 沙箱脚本 Rhai | §5 / §6 | 遵循（**Rhai 不动**，WASM 与 Rhai 并存） |
| RGS-BAS-001 §1.2 颗粒度 | 本文档 | 遵循（保持 BASIC 颗粒度；DTL 留给 Phase 3 落地） |
| RGS-BAS-001 §3.5 ARC-005 | §3 | 遵循 |
| RGS-BAS-001 §4.5 ARC-007 | §3 | 遵循 |
| RGS-BAS-001 §4.7 EV/WF PH-5/6 | §5 / §7 | 遵循（本方案不替代 PH-5/6；与其并行） |
| RGS-BAS-001 §5.1 ARC-008 限界上下文 | §6 / §19 | 遵循 |
| RGS-BAS-002 §4.1 脚手架 | §8.1 | 遵循（**新 crate 复用 workspace 模板**） |
| NFR-OP-010 2 SRE ≤ 20 人天/周 | §11.3 / §25 / 附录 A.4 | 遵循 |

**没有偏离任何既有 ARC/ADR**；新增 ADR 见 §26。

---

> **本文档结尾**。请 5 域 Lead + SRE + 安全 + DBA + 项目负责人联合评审。**未通过评审前不得进入 Phase 1 Benchmark**。
>
> 评审 checklist：
> - [ ] 现状基线 §1-§3 与代码一致
> - [ ] Q1-Q5 答案接受
> - [ ] §11 拒绝 Knative / OpenFaaS 同意
> - [ ] §26 ADR 列表（12 项）授权起草
> - [ ] §27 需求追踪矩阵（51 项）接受
> - [ ] §25 风险表（16 项 + 附录 A.5 新增 1 项）接受
> - [ ] §24 Rollback 流程演练通过
> - [ ] 附录 A.4 总 OLU 不超 NFR-OP-010
