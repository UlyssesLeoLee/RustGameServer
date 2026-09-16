# RGS-ADR-0061: CDC / Outbox 偏离参考设计 — 自研 Outbox 取代 Debezium CDC

| 项目 | 内容 |
|---|---|
| 决策编号 | RGS-ADR-0061 |
| 标题 | CDC / Outbox 偏离参考设计：自研 Outbox 取代 Debezium CDC（4 状态机 outbox_worker） |
| 状态 | **待具名人类审批**（per DEC-008 一人公司兼任；本文为候选提案，由 worker (ULYS-56, ULYS-54.B) 起草） |
| 制定日期 | 2026-09-15 JST |
| 最新修订 | 2026-09-16 JST（v0.2 — §6 后续工作项 P1+P2 共 9 项候选草案已起草, 存放于 `docs/00-基准与治理/ULYS-56-follow-up-drafts/`） |
| 制定人 | worker (ULYS-56 agent) |
| 主对应方针 | ARC-014（未证明需要不引入）、RGS-ADR-0008（中间件导入判定基准）、RGS-ADR-0015（工作流 Saga 适用边界与单一调解者） |
| 关联上游决议 | RGS-REQ-005 附件 D §4 OSS 许可盘点（**Debezium 主项目未登记**——参考设计默认路径，未被 RGS 实际采用） |
| 关联下游文档 | RGS-REQ-100 §7 + BR-111；RGS-BAS-001 §4.7 事件与可观测性设计 + §5.8 通用表结构；RGS-DTL-100 §5.3 事务性消息；RGS-REV-007 CH1+CH2+AH1 Outbox 升级；RGS-SPEC-CROSS-005 事务性消息；`crates/shared-platform/src/outbox.rs`；`crates/shared-platform/src/outbox_relay.rs`；6 份 `0[0-3]X_outbox.sql` migration |
| 关联调查 | `docs/00-基准与治理/ULYS-54-RGS-INV-001_缓存选型与设计偏离调查报告_v0.1.md`（§4.4 横向偏离扫描「自研 Outbox vs Debezium CDC」节 + §5.2 P1 #4 建议「立 ADR-0061」） |

> **状态说明**：本文是候选 ADR，用以正式归档 RGS CDC 路径「参考设计：`PostgreSQL WAL → Debezium → Kafka`」vs「RGS 实际：`事务内强制 Outbox INSERT → 自研 4 状态机 outbox_worker 轮询 → NATS JetStream`」的双轨偏离。偏离已通过 **ADR-0015 Saga 边界 + RGS-REQ-100 §7 BR-111** 隐含「不引入 Debezium」路径形成事实决议，但**没有单点 ADR 显式记录**——这是与 ADR-0059 缓存偏离、ADR-0060 事件总线偏离**同构的治理漏洞**。具名人类审批通过前，本文不构成生产基线变更；REQ-005 附件 D §3/§4、BAS-001 §4.7、DTL-100 §5.3 等下游文档的字面修改在审批通过前不执行。

---

## 1. 背景（Context）

RGS 的 CDC / Outbox 路径在「参考设计 v2」与「RGS 实际」之间存在结构性偏离，参考设计 §16-18 明确写 `PostgreSQL WAL → Debezium → Kafka`（Outbox 模式 + Debezium CDC → Kafka），但 RGS 实际走自研 Outbox 路径——`crates/shared-platform/src/outbox.rs`（基于 sqlx 的 **4 状态机** outbox_worker：Pending → InFlight → Sent / Failed），由 `crates/shared-platform/src/outbox_relay.rs` 轮询并发布到 NATS JetStream，**全程无 Debezium 引入**（无 K8s manifest、无 ADR、Cargo 依赖零命中）。

偏离已通过 ADR-0015 Saga 边界 + RGS-REQ-100 §7 BR-111 形成事实决议，但**单点 ADR 缺失**。这是参考设计强调 Debezium 的核心能力——「捕获所有 DB 变更」——与 RGS 自研路径的「事务内强制 outbox 写入」规避方案的等价性论证缺失，跨域合规审计时无法定位单点 ADR。

### 1.1 与 ADR-0059 / ADR-0060 偏离的同构性（结构性问题）

ULYS-54 §3 + ADR-0059 已揭示 TS-001 §3.5.1 改选 Redis 时绕过 RGS-ADR-0008 §2 闸门；ULYS-54 §4.3 + ADR-0060 已揭示 TS-001 §3.6.1 改选 NATS 时同样绕过闸门。CDC 偏离是**同一结构问题的第三面**：

| 维度 | 缓存偏离（ADR-0059 已处置） | 事件总线偏离（ADR-0060 已立候选） | CDC 偏离（本 ADR 处置） |
|---|---|---|---|
| 上游登记 | REQ-005 §4 L452 Valkey（合规） | REQ-005 §4 L453 Apache Kafka（合规，须 ARC-014 判定） | 参考设计 §16-18：PostgreSQL WAL → Debezium → Kafka（**未在 REQ-005 §4 显式登记**） |
| RGS 实际 | TS-001 §3.5.1 Redis 7.2+ | TS-001 §3.6.1 NATS JetStream 2.10+ | `crates/shared-platform/src/outbox.rs` + `outbox_relay.rs` 自研 4 状态机 |
| 闸门触发 | ❌ 未触发 | ❌ 未触发 | ❌ 未触发 |
| 决议路径 | 4 次升版（v0.4→v0.7）未回头补救 | DEC 拍板（Q-M-10 + ACTIONS-v0.3 B-09） | **隐含路径**：ADR-0015 Saga 边界 + REQ-100 §7 BR-111 |
| 偏离事实 | 零代码层影响（无 Redis 客户端依赖） | 已生产实装（5 域 `async-nats` + 6 份 K8s manifest） | **已生产实装**（6 份 `outbox` 表 + outbox_worker + outbox_relay + 6 个 `main.rs` 启动 outbox relay） |
| ADR 缺口 | ADR-0059 候选已立 | ADR-0060 候选已立 | **本 ADR 候选（ADR-0061）待具名审批** |

**结构同构 = 同一治理漏洞的三次重复**：单点改选 / 选型 / 引入都未触发 RGS-ADR-0008 §2 闸门，靠的是不同形式的「隐含路径」绕过。本 ADR 把 CDC 偏离正式归档，闭合治理漏洞的第三面。

### 1.2 偏离事实（per ULYS-54 §4.4）

| 维度 | 参考设计 / 上游 | RGS 实际 | 决议来源 |
|---|---|---|---|
| CDC 组件 | Debezium（PostgreSQL WAL → Kafka） | **无 Debezium** | 隐含路径 |
| Outbox 模式 | 事务内 INSERT Outbox + Debezium 捕获 | 事务内 INSERT Outbox + 自研 outbox_worker 轮询 | `crates/shared-platform/src/outbox.rs` |
| 事件捕获范围 | 所有 DB 变更（Debezium WAL 捕获） | 仅显式 INSERT Outbox 表的变更 | 自研路径性质 |
| 状态机 | Debezium 内置（基于 WAL offset） | **4 状态机** Pending / InFlight / Sent / Failed + `lease_until`（per RGS-REV-007 CH2） | `outbox.rs` L52-75 |
| 多 relay 并发 | Debezium Connect 单实例 OR cluster | `FOR UPDATE SKIP LOCKED` + 30s lease + reclaim（多 relay 副本并发安全） | `outbox.rs` L24-26 + L271 |
| 持久层 | PostgreSQL + Kafka | PostgreSQL + NATS JetStream（per ADR-0060 候选） | TS-001 §3.6.1 |
| 决策链 | — | ADR-0015 Saga 边界 + REQ-100 §7 BR-111「禁止 Redis Enterprise / 云专有 / 商业 SaaS / 闭源事务协调器」+「评估纯开源替代方案」 | 隐含路径 |
| ADR | — | **缺失（待本工单产出）** | 本工单 |

### 1.3 自研 Outbox 路径已实装证据（per 验收标准 2）

**1.3.1 代码层（crates/shared-platform + 6 个域）**：

- `crates/shared-platform/src/outbox.rs`：OutboxEntry + OutboxRepository trait + Pg/InMemory impl，4 状态机 OutboxStatus（Pending/InFlight/Sent/Failed），lease 默认 30s（per RGS-REV-007 CH2）
- `crates/shared-platform/src/outbox_relay.rs`：OutboxRelay + RelayConfig，轮询发布到 NATS
- **6 个服务的 `main.rs` 已启动 outbox relay 后台轮询**：`crates/admin-service/src/main.rs`、`crates/cluster-ops/src/main.rs`、`crates/economy-service/src/main.rs`、`crates/match-service/src/main.rs`、`crates/player-service/src/main.rs`、`crates/social-service/src/main.rs`（前 5 域 + cluster-ops）
- `crates/economy-service/tests/integration_outbox.rs`：集成测试覆盖 outbox 写入 + 发布路径

**1.3.2 数据库 schema（6 份 migration）**：

- `crates/admin-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`
- `crates/cluster-ops/migrations/0002_outbox.sql` + `0003_outbox_check_idempotent.sql`
- `crates/economy-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`
- `crates/match-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`
- `crates/player-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`
- `crates/social-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`
- 所有表结构遵循 `RGS-BAS-001 §5.8` 通用表结构范式（最低限度列集合），仅 `aggregate_type` 取值不同

**1.3.3 已捕获事件族范围（5 域 + cluster-ops，per 验收标准 2）**：

| 服务 | outbox 表 | aggregate_type 取值 |
|---|---|---|
| admin-service | ✅ | admin.* |
| economy-service | ✅ | economy.*（per `RGS-BAS-001 §5.4.1 ECONOMY_OUTBOX`） |
| match-service | ✅ | match.* |
| player-service | ✅ | player.* |
| social-service | ✅ | social.* |
| cluster-ops | ✅ | cluster_ops.* |

**`shared_platform` 是库 crate，不是服务**——它定义 `OutboxEntry` / `OutboxRepository` / `OutboxRelay` 类型供 6 个域 `main.rs` 引入，本身无独立 `outbox` 表。ULYS-54 §4.4 描述的「5 域 + cluster_ops + shared_platform」应理解为「**5 域 + cluster_ops**」共 **6 个服务**各自持有一份 `outbox` 表 + 引用 `shared_platform` 库。

**1.3.4 跨域事件 Schema 与主题命名**：

- `RGS-SPEC-CROSS-003 v0.2`：主题命名空间 `rgs.events.<domain>.<aggregate>.<action>.<version>`（per §L37，如 `rgs.events.economy.wallet.committed.v1`）
- `RGS-SPEC-CROSS-005`：事务性消息规范（per `outbox.rs` 文件头 L1）
- `RGS-DTL-100 §5.3`：事务性消息详细设计
- `RGS-REV-007 CH1+CH2+AH1`：Outbox 升级记录（FOR UPDATE SKIP LOCKED + lease + InFlight 状态机）

**1.3.5 零 Debezium 证据**（per 验收标准 5 + ULYS-54 §4.4 复现依据）：

- `Cargo.toml` workspace + 32 个 crate `Cargo.toml`：**无 `debezium-client` / `debezium-rs` / 任何 Debezium 相关依赖**（grep 零命中）
- `Cargo.lock`：**无 Debezium 相关 crate 间接依赖**（grep 零命中）
- `docs/deploy/01-k8s-manifests/` + `docs/deploy/02-helm-charts/`：**无 Debezium Connect / Debezium Server Deployment / Service / ConfigMap**（grep 零命中）
- 文档层 ADR 索引：**RGS-ADR-0015（唯一相关 ADR）是 Saga 边界，未涉及 Debezium 选型**（grep Debezium 零命中）

**Debezium 不是「未决/待决」状态，是从一开始就没被引入且被「事务内强制 outbox 写入」规避方案替代**。本 ADR 是把这条隐含路径沉淀为 ADR 记录。

### 1.4 Debezium 合规性评估（per 验收标准 5）

为完备 §3 备选方案否决论证，对 Debezium 本身合规性做严格区分：

| Debezium 形态 | 许可 | 商业属性 | BR-111 合规 | 备注 |
|---|---|---|---|---|
| **Debezium 主项目**（debezium/debezium GitHub） | **Apache-2.0** | 开源 | ✅ 完全合规 | 不属「闭源 / SaaS / 商业」 |
| Debezium Connectors for Confluent Platform | 商业闭源（Confluent 专有） | 闭源 / 商业 | ❌ 不合规 | 属「商业 SaaS」 |
| Debezium Server（独立运行时，非 Confluent） | Apache-2.0 | 开源 | ✅ 完全合规 | 独立部署，无 Confluent 依赖 |
| Debezium + Kafka Connect（Apache Kafka 子项目） | Apache-2.0 | 开源 | ✅ 完全合规 | Kafka Connect 框架本身合规 |

**结论**：Debezium 主项目 = Apache-2.0，**不属 BR-111「禁止 Redis Enterprise / 云专有 / 商业 SaaS / 闭源事务协调器」任何一类**。BR-111「纯开源约束」与 Debezium 不冲突——Debezium 完全合规。RGS 以 BR-111 为理由「隐含拒绝 Debezium」是 BR-111 的**过度延伸**（per ULYS-54 §4.4 影响范围段）。

但这不意味着 ADR-0061 必须撤销自研 Outbox 决议——见 §3 备选方案逐条否决论证，核心否决理由是「Debezium 引入新运维实体的 OLU vs 自研路径已实装的既有 OLU」+「事务内强制 outbox 写入是充分设计」。

### 1.5 ARC-014 / RGS-ADR-0008 闸门触发必要性（per 验收标准 1）

RGS-ADR-0008 §2 闸门条件 ①「既有组件无法承担该职责」要求论证——Debezium 是否承担 RGS 实际承担的「事件传播」职责？本 ADR §3 备选方案逐条回应。核心论证：**RGS 事件传播职责由自研 Outbox + outbox_relay + NATS JetStream 三件套承担**（per `crates/shared-platform/src/{outbox.rs,outbox_relay.rs}` + ADR-0060），**Debezium 的核心能力「捕获所有 DB 变更」在 RGS 不被业务依赖**（per ULYS-54 §4.4 评估 + §1.3.3 已捕获事件族范围清单）。

---

## 2. 决定（Decision）

**维持自研 Outbox 路径（4 状态机 outbox_worker + 6 域 `outbox` 表 + `shared_platform` 抽象层），正式归档偏离参考设计的事实，把「事务内强制 outbox 写入」约束正式化，并触发附件 D §3 登记行同步。** 具体内容：

1. **CDC / Outbox 路径产品**：自研 4 状态机 outbox_worker（Pending / InFlight / Sent / Failed + `lease_until` 30s + `FOR UPDATE SKIP LOCKED` 多 relay 并发安全），由 `crates/shared-platform/src/outbox.rs` 定义、`outbox_relay.rs` 调度、6 域 `main.rs` 启动后台轮询、6 份 `outbox` 表承载。**不引入 Debezium**（无 K8s manifest、无 ADR、无 Cargo 依赖）。
2. **「事务内强制 outbox 写入」约束正式化**：所有业务变更必须在同一 SQL 事务内 INSERT 到本服务的 `outbox` 表（per `outbox.rs` L10-11「业务写 DB + 写 outbox 表必须在同一事务（per DTL-100 §5.3）」+ `outbox.rs` L162-195 `append` 接受 `PgExecutor` 让调用方把「业务写 DB」和「写 outbox」包在同一事务里）。**不在事务内的 DB 变更不会被传播**——这是显式的设计约束，不是 bug。
3. **协议层 / 抽象层**：保持 `crates/shared-platform::producer` / `consumer` / `messaging` / `outbox` / `outbox_relay` 抽象层，业务域代码不直接接触 NATS SDK，**Debezium SDK 也不渗入业务域**（Debezium 当前零引入）。NFR-MI-005「可替换空间」由抽象层提供——未来若需引入 Debezium，仅需替换共享层实现。
4. **下游级联（本 ADR 审批通过后执行）**：
   - **RGS-REQ-005 附件 D §3 登记行**：新增「ADR-0061 CDC / Outbox 偏离参考设计: 自研 Outbox 4 状态机取代 Debezium CDC (待具名人类审批)」登记行
   - **RGS-REQ-005 附件 D §4 OSS 许可盘点**：补注「per ADR-0061，CDC 路径偏离参考设计 Debezium 至自研 Outbox 4 状态机（crates/shared-platform）」「Debezium 主项目 Apache-2.0 合规（per ADR-0061 §1.4）但未引入」
   - **RGS-REQ-100 §7 BR-111 备注栏**：补注「per ADR-0061，BR-111『纯开源约束』与 Debezium Apache-2.0 不冲突；Debezium 拒绝理由是 OLU 与设计替代性论证，非 BR-111 合规」
   - **RGS-BAS-001 §4.7 事件与可观测性设计**：补注「自研 Outbox 4 状态机取代 Debezium CDC（per ADR-0061）；§4.7.1 分发器流程不变（按 outbox 表轮询）」
   - **RGS-BAS-001 §5.8 并发控制与 Outbox 通用表结构范式**：补注「per ADR-0061，6 域 `outbox` 表结构由 `crates/shared-platform/src/outbox.rs` 4 状态机驱动」
   - **RGS-DTL-100 §5.3 事务性消息**：补注「per ADR-0061，`PgExecutor` 接受 + `FOR UPDATE SKIP LOCKED` + 30s lease 是规避 Debezium「捕获所有 DB 变更」能力的工程方案」
   - **RGS-ADR-0015 §决策链**：补注「隐含 Outbox 路径 per ADR-0061」
   - **ULYS-54 INV-001 v0.2+ §4.4**：标注「ADR-0061 候选已立，待审批」（per ULYS-54 处置决议同步；ULYS-56 不修改 INV-001，§6 后续工作项 P3 列入）
   - **代码层**：维持现状（`crates/shared-platform` 4 状态机 + 6 域 `outbox` 表 + 6 个 `main.rs` 启动 outbox relay + `crates/economy-service/tests/integration_outbox.rs`），不引入 Debezium
5. **不引入新客户端 crate**：维持现状（workspace `Cargo.toml` + 32 个 crate `Cargo.toml` 全无 Debezium 依赖），未来实装 Debezium 时按 NFR-MI-005「可替换」原则引入 `debezium-rs` 或自研抽象层。
6. **撤销决议**：无撤销动作——本 ADR 是归档偏离事实 + 正式化「事务内强制 outbox 写入」约束，不改变生产状态。

---

## 3. 曾考虑并否决的方案（Alternatives Considered）

> 以下逐条回应 RGS-ADR-0008 §2 闸门条件 ①「既有组件（自研 Outbox）无法承担该职责」的潜在反验 + 偏离参考设计的备选。注：参考设计 §16-18 默认路径是 Debezium，但其相对自研 Outbox 的 trade-off 由本节明确。

### 3.1 引入 Debezium CDC（Debezium Connect + Apache Kafka Connect）

否决理由：

- **违背「事务内强制 outbox 写入」约束的等价性论证失败**：Debezium 通过 PostgreSQL WAL 捕获**所有** DB 变更（包括非业务触发的事务、运维 SQL、手工 UPDATE 等），RGS 当前架构以「Outbox 只能捕获显式写入 outbox 表的事件，不在事务内的 DB 变更不会被传播」为设计约束（per `outbox.rs` L10-11 + ULYS-54 §4.4 评估）。若引入 Debezium 捕获所有 DB 变更，**会破坏「显式事务内写 Outbox」的等价性论证**——业务层不再能保证「事件 = 业务事务的同事务产物」。
- **新运维实体的 OLU 估算**：Debezium Connect 需要 JVM + Kafka Connect 框架 + PostgreSQL WAL slot 配置 + Debezium PostgreSQL Connector plugin 部署 + 监控（Connector 健康 + WAL lag + DLQ）。RGS 当前 6 域 outbox + outbox_relay 是单语言（Rust）+ 单二进制（NATS JetStream），OLU 已纳入 NFR-OP-010 预算；引入 Debezium 增加 JVM 资源开销 + ZooKeeper/KRaft 复杂度（若 Kafka Connect 集群模式）。
- **「捕获所有 DB 变更」能力在 RGS 不被业务依赖**（per §1.3.3 已捕获事件族清单）：6 域 outbox 表覆盖全部 5 域 + cluster_ops 的业务事件传播需求；非 outbox 写入的 DB 变更（运维 SQL、手工 UPDATE）按设计就**不应**被传播——Debezium 反而是过度设计。
- **违背 ARC-014「未证明需要不引入」**：当前无业务用例需要「捕获所有 DB 变更」；若有新用例，应在 ADR 中独立论证。

### 3.2 引入 Debezium + 自研 Outbox 双轨

否决理由：

- **同一职责两套实现违反 ARC-014「同一类目只选一个」**：Debezium 捕获 WAL + 自研 Outbox 捕获显式 INSERT 是两条独立事件传播路径，业务层无法保证「事件 = 业务事务的同事务产物」。
- **OLU 倍增**：维护两套事件传播基础设施（Debezium Connect 集群 + outbox_worker 集群）vs 当前仅维护 outbox_worker 集群。
- **复杂度无收益**：RGS 当前业务事件族清单（5 域 + cluster_ops）已全部由 outbox 覆盖，Debezium 增量覆盖的「非 outbox DB 变更」**不应被传播**——双轨是引入复杂度而无收益。

### 3.3 引入 Apache Kafka Connect（含 Debezium Connector）+ Kafka 取代 NATS

否决理由：

- **违背 ADR-0060 候选（事件总线偏离参考设计 NATS vs Kafka）**：RGS 已决策 NATS JetStream 2.10+（per TS-001 §3.6.1 v0.7 + Q-M-10 答复 + ACTIONS-v0.3 B-09）；回到 Kafka = 推翻 ADR-0060 + DEC-005/006，**与本 ADR 并行撤销代价过大**。
- **违背 ARC-014 / OLU**：JVM + ZooKeeper/KRaft 复杂度 vs 当前 5 域 NATS 单二进制 OLU（per ADR-0060 §3.1）。

### 3.4 自研 PostgreSQL WAL 解析器

否决理由：

- **违背 ARC-014「未证明需要不引入」**：PostgreSQL WAL 解析需要维护 `pgoutput` / `test_decoding` 插件适配 + LSN 跟踪 + checkpoint 恢复 + 崩溃一致性保障——OLU 与 Debezium 持平，但生态成熟度远低于 Debezium（Debezium 有 Red Hat / Decodable 商业支持）。
- **RGS 已有等价方案**：自研 Outbox 已经实现「事务内强制写 + relay 轮询」等价于 WAL 解析的事务一致性 + 事件传播，无业务用例驱动自研 WAL 解析器。
- **违背 BR-112 可观测性**：WAL 解析器的监控指标（LSN 滞后 / checkpoint 位置 / replication slot 大小）需要新建可观测性体系；outbox 已用 NATS JetStream 指标 + tracing 覆盖。

### 3.5 维持现状不正式化（即不立 ADR-0061）

否决理由：

- **重复 ADR-0059 / ADR-0060 已揭示的治理漏洞**：ULYS-54 调查报告 §3-§4 已识别「单点改选未触发 ADR 闸门」的结构性问题；本 ADR 是该结构问题的第三面（CDC 维度），不正式化 = 治理漏洞未闭合。
- **跨域合规审计时无法定位单点 ADR**：未来审计 / 招聘 / 评审问「为什么 RGS 不引入 Debezium」时，无单点 ADR 可引——只能拼接 ADR-0015 Saga 边界 + REQ-100 §7 BR-111 隐含推理，论证链不完整。
- **违背 RGS-ADR-0008 §4「驳回须写明未满足哪一条」的反向义务**：当前隐含路径「BR-111『纯开源约束』 → Debezium 拒绝」实际上是 BR-111 的过度延伸（Debezium 本身合规），未在 ADR 中明确写明「Debezium 不引入」的真实理由（OLU + 设计等价性）。

---

## 4. 结果与代价（Consequences）

**获得**：

- **偏离参考设计的事实正式归档**：补登记 RGS-REQ-005 §3 ADR-0061 行，治理漏洞闭合（与 ADR-0059 / ADR-0060 同构处置，三面治理漏洞全部关闭）；
- **DEC 与 ADR 分工明确**：隐含路径（ADR-0015 + REQ-100 §7 BR-111）→ ADR（ADR-0061）沉淀为可追溯记录——**符合 RGS-ADR-0008 §4「驳回须写明未满足哪一条」的反向义务**；
- **「事务内强制 outbox 写入」约束正式化**：从 `outbox.rs` 文件头注释 + `DTL-100 §5.3` 升为 ADR 级正式约束，未来业务实现 / 代码审查 / 新成员 onboarding 有单点可引；
- **BR-111「纯开源约束」边界澄清**：Debezium 主项目 Apache-2.0 合规，但 RGS 不引入 = OLU + 设计替代性论证（非 BR-111 合规），避免 BR-111 的过度延伸；
- **ARC-014 / RGS-ADR-0008 闸门执行的先例成立**：与 ADR-0059 / ADR-0060 同构处置，未来类似偏离（自研 vs 引入开源中间件）有模板可循；
- **6 域全栈零代码层改动**：自研 Outbox 已实装（6 份 `outbox` 表 + outbox_worker + outbox_relay + 6 个 `main.rs` 启动 + 集成测试），本 ADR 不引入新依赖、不撤销已落地；
- **NFR-MI-005 可替换空间**：业务域通过 `crates/shared-platform::outbox` 抽象接触 CDC 路径，**Debezium SDK 不渗入业务域**（Debezium 当前零引入），未来若需迁移仅需替换共享层实现；
- **与 NFR-PE-011 性能约束兼容**：自研 outbox_worker 4 状态机 + 30s lease + `FOR UPDATE SKIP LOCKED` 已通过 `RGS-TST-S5-outbox-NATS-IT-设计书` 集成测试 + `RGS-CAP-001` 容量基线论证。

**付出**：

- **下游文档字面更新**：REQ-005 §3/§4、REQ-100 §7 BR-111 备注、BAS-001 §4.7/§5.8、DTL-100 §5.3、ADR-0015 决策链、INV-001 v0.2+ §4.4 共 6 处补注；
- **失去 Debezium「捕获所有 DB 变更」能力的 trade-off**（per ULYS-54 §4.4 评估）：非事务内 DB 变更不被传播——这是显式设计约束，由 §2 决定 2「事务内强制 outbox 写入」承接；
- **强制事务内写入的工程负担**：业务实现者必须理解「事务内 INSERT outbox」是事件传播的强制路径——CR（Code Review）闸门须检查 `outbox.append` 与业务 DML 在同一 `Transaction` / `PgExecutor` 内；
- **BR-111 边界澄清可能引发下游 ADR 复审**：BR-111「禁止 Redis Enterprise / 云专有 / 商业 SaaS / 闭源事务协调器」与 Debezium Apache-2.0 不冲突的结论，可能引发其他 ADR（如 ADR-0044 客户端资源分发自托管）的边界复审——属合理治理循环；
- **偏离参考设计的事实需在交付时向上声明**：本 ADR 通过具名人类审批后，需在交付文档（PH-1 启动前 handoff）显式声明「RGS CDC 路径偏离参考设计 Debezium，per ADR-0061；自研 Outbox 4 状态机」。

**已知张力**：

- **RGS-REQ-100 §7 BR-111 与 Debezium Apache-2.0 的张力**：BR-111 备注栏「如 Redis 功能确实需要，评估纯开源替代方案（KeyDB / Dragonfly / 自研 PostgreSQL-based 缓存）」当前未提及 Debezium。本 ADR 通过后 BR-111 备注补注「per ADR-0061，Debezium 本身合规但未引入，理由是 OLU + 设计替代性」，**关闭张力**；通过前该栏仍标「BR-111 边界未澄清 Debezium」。
- **与 ADR-0060 事件总线选型的路径耦合**（per 验收标准 3）：若 ADR-0060 通过 NATS JetStream（当前候选），Debezium → NATS 集成无现成 connector（Debezium 原生仅支持 Kafka Connect / Pulsar / Kinesis sink）；Debezium → NATS 需自研 sink connector，OLU 进一步增加。**本 ADR 通过后该路径耦合消解**——RGS CDC 路径 = 自研 Outbox + NATS JetStream（per ADR-0060 + ADR-0061 协同），无需 Debezium → NATS connector。
- **「捕获所有 DB 变更」能力未来用例的张力**：若未来出现「非 outbox DB 变更需被传播」的业务用例（如审计 / 反作弊 / 数据分析实时管道），需立新 ADR 重新评估 Debezium 引入。当前用例清单（5 域 + cluster_ops）已覆盖，**该张力属未来风险，非当前阻塞**。

---

## 5. 关联

- **依赖本决策**：ARC-014（未证明需要不引入）、RGS-ADR-0008（中间件导入判定基准）、RGS-ADR-0015（工作流 Saga 适用边界与单一调解者）、RGS-REQ-100 §7 BR-111（纯开源约束）、RGS-DTL-100 §5.3 事务性消息、RGS-REV-007 CH1+CH2+AH1（Outbox 升级）、RGS-SPEC-CROSS-005 事务性消息
- **本决策归档的偏离**：参考设计 §16-18「PostgreSQL WAL → Debezium → Kafka」vs RGS 实际「事务内强制 Outbox INSERT → 自研 4 状态机 outbox_worker 轮询 → NATS JetStream」
- **本决策触发的下游级联**：RGS-REQ-005 §3 登记行 + §4 OSS 许可盘点补注 + Debezium Apache-2.0 合规备注；RGS-REQ-100 §7 BR-111 备注栏补注；RGS-BAS-001 §4.7/§5.8 补注；RGS-DTL-100 §5.3 补注；RGS-ADR-0015 决策链补注；ULYS-54 INV-001 v0.2+ §4.4 标注
- **协同决策**：ADR-0060 事件总线偏离（NATS JetStream vs Kafka，候选待具名审批）——RGS CDC 路径 = 自研 Outbox + NATS JetStream，本 ADR 与 ADR-0060 协同形成完整事件传播栈
- **冲突 / 延伸事项**：无；与 ADR-0059 缓存偏离、ADR-0060 事件总线偏离同构处置（三面治理漏洞全部闭合）；与 ULYS-55 (ADR-0060 NATS vs Kafka) 无依赖可并行；与 ULYS-54 INV-001 v0.2+ §5.2 P1 #5 (Temporal vs 自研 Saga) 属 DEC 拍板范围暂不拆子任务

---

## 6. 后续工作项（按优先级）

| 优先级 | 工作项 | 备注 |
|---|---|---|
| **P0** | 本 ADR 具名人类审批（Ulysses 一人公司 12 角色 per DEC-008） | 审批通过后执行 §2 决定 4 的下游级联 |
| **P1** | RGS-REQ-005 附件 D §3 + §4 补注（登记行 + Debezium Apache-2.0 合规备注） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P1** | RGS-REQ-100 §7 BR-111 备注栏补注（澄清 Debezium 不属 BR-111 禁止类目） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P1** | RGS-BAS-001 §4.7 + §5.8 补注（4 状态机 + 6 域 outbox 表结构） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P1** | RGS-DTL-100 §5.3 补注（FOR UPDATE SKIP LOCKED + 30s lease 工程方案） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P2** | RGS-ADR-0015 决策链补注（隐含 Outbox 路径 per ADR-0061） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P2** | Outbox 监控指标补全（per `outbox.rs` + NATS JetStream 指标） | SRE Lead 主导；outbox_pending 计数 / outbox_in_flight lease 滞后 / outbox_failed 累积 / relay 轮询周期 |
| **P2** | Poison event 处理流程（Failed 状态 4 状态机的 DLQ 路径） | 当前 Failed 仅 `last_error` 字段记录；需明确 DLQ 表 / 重试策略 / 告警阈值 |
| **P2** | Outbox schema evolution 流程（per `0004_outbox_check_idempotent.sql` 模式） | 当前靠逐次 `0XXX_outbox*.sql` migration；需明确 schema 变更的回滚 / 兼容性策略 |
| **P2** | 跨域事件族清单补全（5 域 + cluster-ops 当前已覆盖；未来新域 onboarding 需补登记） | 域 owner + 架构师联合维护 |
| **P3** | ULYS-54 INV-001 v0.2+ §4.4 联动：本 ADR 候选已立 + ADR-0059 + ADR-0060 + ADR-0061 三面治理漏洞全部闭合 | ULYS-54 关闭前同步；ULYS-56 不修改 INV-001，由 ULYS-54 协调者执行 |
| **P3** | 交付文档（PH-1 启动前 handoff）显式声明「RGS CDC 路径偏离参考设计 Debezium，per ADR-0061；自研 Outbox 4 状态机」 | 候选操作者：架构师；本 ADR 通过后即可起草 |

---

## 7. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-09-15 JST | worker (ULYS-56 agent) | 初版制定。归档自研 Outbox 4 状态机 vs Debezium CDC 偏离事实；Debezium Apache-2.0 合规性评估（§1.4）；「事务内强制 outbox 写入」约束正式化（§2 决定 2）；6 域 outbox 实装证据清单（§1.3）；下游级联清单 6 项；后续工作项 11 项；与 ADR-0059 / ADR-0060 同构处置（三面治理漏洞闭合） |
| 0.2 | 2026-09-16 JST | worker (ULYS-56 agent, per user 指示 "完成后续") | **§6 后续工作项 P1+P2 共 9 项候选草案已起草**：P1 (4 项) REQ-005/REQ-100/BAS-001/DTL-100 补注候选 + P2 (5 项) ADR-0015 决策链补注 + Outbox 监控指标 + Poison event DLQ + Schema evolution + 跨域事件族清单，存放于 `docs/00-基准与治理/ULYS-56-follow-up-drafts/`。**注**：ADR-0061 多次引用 "RGS-DTL-100 §5.3 事务性消息"，但 RGS-DTL-100 实际章节为 §4 (L403) "Outbox + Inbox Pattern"——已记录于候选草案 `04_DTL-100_§5.3_补注候选.md` §2，建议 v0.3+ 修订 ADR-0061 章节引用。**范围外**：P0 (具名人类审批, per DEC-008) + P3-#1 (INV-001 v0.2+ §4.4 联动, ULYS-54 协调者执行) + P3-#2 (PH-1 启动前 handoff 声明) 不由 ULYS-56 worker 执行。 |

> **下次评审**：随 ULYS-54 处置决议同步更新（批准 / 修订 / 驳回）+ 本 ADR 具名人类审批通过后升级为 Accepted。
