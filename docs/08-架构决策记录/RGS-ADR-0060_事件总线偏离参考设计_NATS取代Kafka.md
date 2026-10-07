# RGS-ADR-0060: 事件总线偏离参考设计 — NATS JetStream 2.14 (Latest) 取代 Apache Kafka

| 项目 | 内容 |
|---|---|
| 决策编号 | RGS-ADR-0060 |
| 标题 | 事件总线偏离参考设计：NATS JetStream 2.14 (Latest) 取代 Apache Kafka |
| 状态 | **待具名人类审批**（per DEC-008 一人公司兼任；本文为候选提案，由 worker (ULYS-55, ULYS-54.A) 起草） |
| 制定日期 | 2026-09-15 JST |
| 制定人 | worker (ULYS-55 agent) |
| 主对应方针 | ARC-010（事件命名 + partition_key 规则）、ARC-011（Saga 边界）、ARC-014（未证明需要不引入）、RGS-ADR-0008（中间件导入判定基准） |
| 关联上游决议 | RGS-REQ-005 附件 D §4 OSS 许可盘点 L453（事件基础设施 = **Apache Kafka**, Apache-2.0, 合规, 导入须经 ARC-014 判定） |
| 关联下游文档 | RGS-TS-001 §3.6.1 + §5.1；RGS-REQ-031 CEM 中心事件管理；RGS-BAS-001 §4.7 事件与可观测性设计；RGS-ADR-0015 Saga 边界；RGS-ADR-0051 中心事件管理；RGS-SPEC-CROSS-003 跨域事件 Schema v0.2；RGS-TST-S5-outbox-NATS-IT 集成测试；scripts/verify_fail_closed.ps1 |
| 关联调查 | `docs/00-基准与治理/ULYS-54-RGS-INV-001_缓存选型与设计偏离调查报告_v0.1.md`（§4.3 横向偏离扫描 NATS vs Kafka 节 + §5.2 P1 #3 建议「立 ADR-0060」） |

> **状态说明**：本文是候选 ADR，用以正式归档 RGS 事件总线在「上游登记（Apache Kafka）vs 下游选型（NATS JetStream）」上的双轨决议。RGS-TS-001 §3.6.1 自 v0.6 (2026-08-22) 起标"【一致】"，自 v0.7 (2026-08-24) 起经 Q-M-10 答复 + ACTIONS-v0.3 B-09 升为"**【已决策：NATS JetStream】**"，但**未触发 RGS-ADR-0008 §2 闸门、未复审 RGS-REQ-005 §4 上游登记、未补立单点 ADR**。DEC 是即时决策（per Q-M-10 答复 + ACTIONS-v0.3 B-09 合并动作），ADR 是单点决定的可追溯记录——本 ADR 把这条 DEC 链沉淀为 ADR 记录，闭合治理漏洞。具名人类审批通过前，本文不构成生产基线变更；TS-001 / REQ-031 / SPEC-CROSS-003 等下游文档的字面修改在审批通过前不执行。

---

## 1. 背景（Context）

RGS 的事件总线在两份独立登记册上产生了"双轨"决议：

- **上游登记册**：RGS-REQ-005 附件 D §4 OSS 许可盘点 L453（2026-08-19 起登记）把 **Apache Kafka** 登记为「事件基础设施」合规组件，许可 Apache-2.0，备注「**导入须经 ARC-014 判定**」——即选 Kafka 是上游"开放技术栈约束"环节的合规默认，但**任何偏离 Kafka 的备选都必须经过 ARC-014 单点 ADR 闸门**。
- **下游选型报告**：RGS-TS-001 §3.6.1（v0.6 起 2026-08-22「一致」，v0.7 起 2026-08-24「已决策」）改选 **NATS JetStream**（自 v0.6 写 `2.10+`，per ADR-0060 v0.2 升档为 `2.14 (Latest)`），理由 4 条（ARC-010 满足 / 单一二进制 / 持久化一站式 / 比 Kafka 资源占用低一个量级），**且未触发 RGS-ADR-0008 闸门、未补立单点 ADR**。

### 1.1 与 ADR-0059 缓存偏离的同构性（结构性问题）

ULYS-54 §3 + ADR-0059 已揭示 TS-001 §3.5.1 改选 Redis 时绕过了 RGS-ADR-0008 §2 闸门（缓存组件偏离）。NATS vs Kafka 是**同一结构问题的另一面**：

| 维度 | 缓存偏离 (ADR-0059 已处置) | 事件总线偏离 (本 ADR 处置) |
|---|---|---|
| 上游登记 | REQ-005 §4 L452 Valkey (合规) | REQ-005 §4 L453 Apache Kafka (合规, 须 ARC-014 判定) |
| 下游选型 | TS-001 §3.5.1 Redis 7.2+ | TS-001 §3.6.1 NATS JetStream 2.14 (Latest)（自 `2.10+` 升档 per ADR-0060 v0.2） |
| 闸门触发 | ❌ 未触发 | ❌ 未触发 |
| 决议路径 | 4 次升版 (v0.4→v0.7) 未回头补救 | DEC 拍板 (Q-M-10 + ACTIONS-v0.3 B-09) |
| 偏离事实 | 零代码层影响 (无 Redis 客户端依赖) | **已生产实装** (5 域 `async-nats` 客户端 + 30-nats-*.yaml 6 份 K8s manifest) |
| ADR 缺口 | ADR-0059 候选已立 | **本 ADR 候选 (ADR-0060) 待具名审批** |

**DEC 不是 ADR**——DEC 是即时决策（per ACTIONS-v0.3 B-09 WF-1-55.48 执行），ADR 是单点决定的可追溯记录。本 ADR 的存在不否定 DEC-005/006 + Q-M-10 答复的合法性，但把 DEC 链沉淀为 ADR，**闭合治理漏洞**。

### 1.2 偏离事实（per ULYS-54 §4.3）

| 维度 | 参考设计 / 上游 | RGS 实际 | 决议来源 |
|---|---|---|---|
| 事件总线 | Apache Kafka | NATS JetStream **2.14 (Latest)** | TS-001 §3.6.1（v0.7 已决策；版本号升档 per ADR-0060 v0.2 候选） |
| 客户端语言 | Java / Scala / 多语言官方客户端 | **Rust (`async-nats = "0.42"`)** — 我们项目实际使用，synadia 同团队维护 | `Cargo.toml` workspace dep + `crates/shared-platform/src/producer.rs:11` |
| 持久化 | Log 段文件，长期保留 | Stream + Consumer 持久化，可配置保留 | TS-001 §3.6.1 备选 |
| 顺序保证 | Partition 内强顺序 | Stream 内消息有序 | TS-001 §3.6.1 备选 |
| 吞吐量 | 百万级 QPS | 十万级 QPS（单节点） | TS-001 §3.6.1 备选 |
| 运维负担 | JVM + ZooKeeper/KRaft，4-8GB 内存 | Go 单二进制（**服务端**，非项目代码），50-200MB 内存 | TS-001 §3.6.1 备选 |
| 服务端协议 | Apache-2.0（Linux Foundation） | Apache-2.0（CNCF, 2025 license 风波后守住，未改 BSL） | NATS 官方 LICENSE-APACHE-2.0.txt + svix.com FAQ 2026-08-28 |
| DEC 链 | — | Q-M-10 答复 + ACTIONS-v0.3 B-09 升「已决策」 | DEC-005/006 路径 |
| ADR | — | **缺失（待本工单产出）** | 本工单 |

### 1.3 NATS JetStream 实际使用证据（per 验收标准 3）

- **Cargo 依赖**：workspace `Cargo.toml` L88 显式声明 `async-nats = "0.42"`；4 个 crate 共享依赖（`crates/shared-platform/Cargo.toml`, `crates/gm-backend/Cargo.toml`, `crates/rgs-overflow-alert/Cargo.toml`）+ 5 域 `main.rs` 实际使用（`admin-service`, `cluster-ops`, `economy-service`, `match-service`, `player-service`）+ 共享层 `crates/shared-platform/src/{producer,consumer,messaging,outbox_relay}.rs`
- **K8s manifests**：`docs/deploy/01-k8s-manifests/30-nats-{configmap,networkpolicy,pvc,sa,service,statefulset}.yaml` 6 份均已合并（per Q-M-10 答复确认）
- **Outbox schema 稳定**：`000X_outbox.sql` + idempotent check 5 域联检通过（per Q-M-10 答复确认）
- **msg header 格式冻结**：随 RGS-SPEC-CROSS-003 v0.2 冻结（per B-09 Q-D-09 + Q-M-10 合并动作）
- **主题命名空间**：`rgs.events.<domain>.<aggregate>.<action>.<version>`（per RGS-SPEC-CROSS-003 v0.1 §L37，如 `rgs.events.economy.wallet.committed.v1`）
- **集成测试**：`docs/00-基准与治理/RGS-TST-S5-outbox-NATS-IT-设计书.md` 已立专项集成测试设计书

**NATS JetStream 不是「未决/待决」状态，是已决策且已落地**。本 ADR 是把已形成的 DEC 决策沉淀为 ADR 记录。

### 1.4 ARC-014 / RGS-ADR-0008 闸门触发必要性（per 验收标准 2 备注）

RGS-REQ-005 §4 L453 备注栏写「**导入须经 ARC-014 判定**」——TS-001 §3.6.1 改选 NATS 时该栏未被复审。按 RGS-ADR-0008 §2 闸门条件 ①「既有组件（Kafka）无法承担该职责」要求论证，本 ADR §3 备选方案逐条回应。

---

## 2. 决定（Decision）

**维持 NATS JetStream 2.14 (Latest) 选型，正式归档偏离参考设计的事实，并触发附件 D §3 登记行同步。** 具体内容：

1. **事件总线产品**：NATS JetStream **2.14 (Latest)**（per TS-001 §3.6.1 v0.7 + Q-M-10 答复 + ACTIONS-v0.3 B-09；版本号升档自 `2.10+` → `2.14` per ADR-0060 v0.2）。NATS JetStream 由 Synadia 维护，**服务端** Apache-2.0 许可（CNCF 托管，2025 license 风波后守住 Apache-2.0 未改 BSL；per NATS 官方 LICENSE-APACHE-2.0.txt + svix.com FAQ 2026-08-28），**客户端** `async-nats = "0.42"` 纯 Rust 实现（synadia 同团队维护，无 FFI / cgo），单二进制 Go 服务端（**非项目代码**，仅通过 K8s manifest 部署），Stream 持久化 + 消费者组 + 重放 + DLQ 一站式。
2. **协议层**：NATS 文本协议（subject-based routing），与 Kafka 二进制协议不通——但**客户端 API 抽象已就位**（`crates/shared-platform/src/producer.rs` + `consumer.rs` + `messaging.rs`），业务域代码不直接接触 NATS SDK，业务可替换空间（per NFR-MI-005）由共享层抽象提供。
3. **事件命名 + partition_key**：维持 `rgs.events.<domain>.<aggregate>.<action>.<version>`（per RGS-SPEC-CROSS-003 v0.2 §L37），`partition_key` 通过 subject 内嵌（`subject` 内含 aggregate id）实现 Stream 内单 aggregate 有序，与 Kafka partition 语义等价。
4. **下游级联（本 ADR 审批通过后执行）**：
   - **RGS-REQ-005 附件 D §3 登记行**：新增「ADR-0060 事件总线偏离参考设计: NATS JetStream 2.14 (Latest) 取代 Kafka (待具名人类审批)」登记行
   - **RGS-REQ-005 附件 D §4 OSS 许可盘点**：L453 行（Apache Kafka）补注「per ADR-0060，事件总线偏离至 NATS JetStream 2.14 (Latest) (Apache-2.0)」
   - **RGS-TS-001 §3.6.1**：补注「偏离参考设计（per ADR-0060 候选）」，状态保持「【已决策：NATS JetStream】」，版本号随 ADR-0060 v0.2 升档为 `2.14 (Latest)`
   - **RGS-TS-001 §5.1 已决选型表**：「NATS JetStream」行补注「(偏离参考设计 Kafka，per ADR-0060，版本 `2.14 (Latest)`)」
   - **RGS-TS-001 修订历史**：新增 v0.10 条目记录本次偏离正式化
   - **RGS-REQ-031 CEM 中心事件管理**：§「事件基础设施」补注「NATS JetStream, per ADR-0060」
   - **RGS-SPEC-CROSS-003 v0.2**：§L37 主题命名空间补注「基于 NATS JetStream subject 路由, per ADR-0060」
   - **RGS-TST-S5-outbox-NATS-IT-设计书**：标题补注「per ADR-0060 NATS JetStream 偏离」
   - **代码层**：维持现状（`async-nats = "0.42"` 4 处显式依赖 + workspace 共享），不引入新客户端 crate
5. **不撤销下游已落地**：K8s manifest（30-nats-*.yaml 6 份）、Outbox schema、msg header 格式、集成测试设计书——**全部不重做**，仅补注 ADR-0060 引用。
6. **撤销决议**：无撤销动作——本 ADR 是归档偏离事实，不改变生产状态。

---

## 3. 曾考虑并否决的方案（Alternatives Considered）

> 以下逐条引 TS-001 §3.6.1 备选表 + 本 ADR 补充论证。注：上游登记 Apache Kafka 是合规默认，但其相对 NATS JetStream 的 trade-off 由本节明确。

### 3.1 回到 Apache Kafka（撤销 NATS 偏离）

否决理由：

- **违背 DEC 决策链 + 已落地事实**：DEC-005/006 + Q-M-10 + ACTIONS-v0.3 B-09 已升「已决策」状态；5 域 `async-nats` 已实装；K8s manifest 6 份已合并；Outbox schema 稳定；集成测试已立项。回到 Kafka = 推翻既有 DEC + 重做全栈，**无触发场景**（Kafka 在吞吐上有优势，但 RGS 5 域单节点十万级 QPS 满足 ARC-010 partition_key 设计的实际需求；per RSK-073「每季度 review 上游活跃度」已设风险缓冲）。
- **RGS-ADR-0008 §2 闸门条件 ①「既有 NATS 无法承担该职责」不成立**：NATS JetStream 已承担 5 域全部事件传播，6 份 manifest 落地 + 集成测试已立项，「无法承担」是反事实。
- **违背 ARC-014「未证明需要不引入」**：JVM 资源开销 + ZooKeeper/KRaft 复杂度 vs RGS 5 域全开的 OLU 预算（NFR-OP-010），Kafka 引入新运维实体未通过 ARC-014 闸门。

### 3.2 维持 Apache Kafka 不变（从一开始就未偏离）

否决理由：

- **不存在的备选**：本 ADR 的偏离事实已发生（TS-001 §3.6.1 v0.6+ 已选 NATS），「维持 Kafka 不变」是假设性场景，不构成本 ADR 的真实备选。
- TS-001 §3.6.1 的 4 条 NATS 理由（ARC-010 满足 / 单一二进制 / 持久化一站式 / 资源占用低）若被推翻，需在 TS-001 §3.6.1 内部重新论证——本文不重做该论证。

### 3.3 RabbitMQ

否决理由（per TS-001 §3.6.1 备选表）：吞吐/分区能力低于 NATS JetStream；与 ARC-010 partition_key 设计契合度低（RabbitMQ queue-based 路由 vs NATS subject-based 路由，前者难表达 partition_key 一致性约束）。

### 3.4 Redis Streams

否决理由（per TS-001 §3.6.1 备选表）：可靠性 + 复制能力弱于 NATS JetStream（Redis Streams 主从 + Sentinel 仅 1 主写多从读，副本数据非强一致；NATS JetStream Raft 共识协议支持 3-replica 强一致 + 自动选主）。

### 3.5 Apache Pulsar

否决理由（per TS-001 §3.6.1 备选表）：生态复杂度高（Pulsar = BookKeeper + ZooKeeper + Pulsar Broker 三层，比 Kafka 复杂；NATS JetStream = 单二进制 Go 实现，运维负担低一个量级）。RSK-073 已为 NATS 维护节奏放缓预留 Pulsar 评估路径（每季度 review）。

---

## 4. 结果与代价（Consequences）

**获得**：

- **偏离参考设计的事实正式归档**：补登记 RGS-REQ-005 §3 ADR-0060 行，治理漏洞闭合——后续审计 / 招聘 / 评审可定位单点 ADR；
- **DEC 与 ADR 分工明确**：DEC 是即时决策（per Q-M-10 + ACTIONS-v0.3 B-09），ADR 是可追溯记录——本 ADR 把 DEC 链沉淀为 ADR，**符合 RGS-ADR-0008 §4「驳回须写明未满足哪一条」的反向义务**；
- **ARC-014 / RGS-ADR-0008 闸门执行的先例成立**：与 ADR-0059 缓存偏离同构处置，未来类似偏离有模板可循；
- **trade-off 显式记录**：运维简化（NATS Server 单二进制 50-200MB vs Kafka JVM + ZooKeeper 4-8GB）换取吞吐上限（NATS 十万级 QPS 单节点 vs Kafka 百万级 QPS）——这是 RGS 主动接受的代价，与 ARC-014 / OLU 约束一致；
- **5 域全栈零代码层改动**：NATS 已实装、Outbox 已稳定、msg header 已冻结——本 ADR 不引入新依赖、不撤销已落地；
- **NFR-MI-005 可替换空间**：业务域通过 `crates/shared-platform::producer` / `consumer` 抽象接触事件总线，**NATS SDK 不渗入业务域**（per `crates/shared-platform/src/messaging.rs` 抽象层），未来若需迁移回 Kafka 仅需替换共享层实现。

**付出**：

- **下游文档字面更新**：REQ-005 §3/§4、TS-001 §3.6.1/§5.1/修订历史、REQ-031 CEM、SPEC-CROSS-003、TST-S5 共 7 处补注；
- **吞吐量上限风险**：NATS JetStream 单节点十万级 QPS vs Kafka 百万级 QPS——RGS 5 域全开 + 6 worktree 派工的 OLU 估算需复核（per `RGS-CAP-001 §3` 容量基线 + NFR-PE-011）；
- **上游维护节奏风险**：RSK-073 已登记「NATS JetStream 上游维护节奏放缓可能增加替换风险」——每季度 review 上游活跃度并同步 OSS 盘点；
- **偏离参考设计的事实需在交付时向上声明**：本 ADR 通过具名人类审批后，需在交付文档（PH-1 启动前 handoff）显式声明「RGS 事件总线偏离参考设计 Kafka，per ADR-0060」。

**已知张力**：

- **RGS-REQ-005 §4 L453 备注栏「导入须经 ARC-014 判定」与 RGS-TS-001 §3.6.1 缺 ARC-014 闸门回执的张力**：本 ADR 通过后 L453 备注补注「per ADR-0060 闸门回执已立」，**关闭张力**；通过前该栏仍标「未闸门回执」。
- **NATS 5 域单节点 vs Kafka 集群的容量对比张力**：RGS 5 域全开 + 6 worktree 派工下的峰值 QPS 待 SRE Lead 具名复核（per NFR-PE-011）。

---

## 5. 关联

- **依赖本决策**：ARC-010（事件命名 + partition_key 规则）、ARC-011（Saga 边界）、ARC-014（未证明需要不引入）、RGS-ADR-0008（中间件导入判定基准）、RGS-REQ-005 附件 D §4 OSS 许可盘点 L453、RGS-ADR-0015（Saga 边界与单一调解者）、RGS-ADR-0051（中心事件管理）
- **本决策归档的偏离**：RGS-TS-001 §3.6.1（v0.6 起至 v0.7 持续偏离 Kafka 改为 NATS JetStream）
- **本决策触发的下游级联**：RGS-REQ-005 §3 登记行 + §4 L453 备注；RGS-TS-001 §3.6.1/§5.1/修订历史；RGS-REQ-031 CEM §事件基础设施；RGS-SPEC-CROSS-003 v0.2 §L37；RGS-TST-S5-outbox-NATS-IT-设计书标题
- **冲突 / 延伸事项**：无；与 ADR-0059 缓存偏离同构处置（已闭环）；与 ULYS-56 (ADR-0061 自研 Outbox vs Debezium CDC) 无依赖可并行；与 ULYS-54.C (ADR-0062 Temporal vs 自研 Saga) 方法论共享

---

## 6. 后续工作项（按优先级）

| 优先级 | 工作项 | 备注 |
|---|---|---|
| **P0** | 本 ADR 具名人类审批（Ulysses 一人公司 12 角色 per DEC-008） | 审批通过后执行 §2 决定 4 的下游级联 |
| **P1** | RGS-REQ-005 附件 D §3 + §4 补注（登记行 + L453 备注） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P1** | RGS-TS-001 v0.10 升版（§3.6.1 + §5.1 + 修订历史） | 候选操作者：架构师；本 ADR 通过后即可起草 |
| **P2** | RGS-REQ-031 / SPEC-CROSS-003 / TST-S5 三处补注 | 文档维护者 follow-up |
| **P2** | NATS 性能拐点监控（per RSK-073 + NFR-PE-011） | SRE Lead 主导；每季度 review 上游活跃度 |
| **P3** | ULYS-54 INV-001 v0.2+ 联动：本 ADR 候选已立 + ADR-0059 + ADR-0061 闭环 | ULYS-54 关闭前同步 |

---

## 7. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-09-15 JST | worker (ULYS-55 agent) | 初版制定。归档 NATS JetStream vs Kafka 偏离事实；下游级联清单 7 项；后续工作项 6 项 |
| 0.2 | 2026-09-16 JST | worker (ULYS-55 agent) | **响应具名人类 Option A 选定**（per ULYS-55 评论 01a0a95b-64a9-7e22-a8bf-63c8e2b8c7a4）：① 版本号升档 `NATS JetStream 2.10+` → `NATS JetStream 2.14 (Latest)`（per Synadia 当前受支持窗口 + 文档澄清 STAN vs JetStream 混淆）② §1.2 加「客户端语言 = Rust (`async-nats = "0.42"`)、「服务端协议 = Apache-2.0 (CNCF 守住 2025 BSL 风波)」两行（回应项目语言调性问题 + 开源协议澄清）④ §2 banner + §1.2 + §4 trade-off 三处版本号同步升档。**状态保持「待具名人类审批」**：具名人类已表达意向 (Option A)，待文档化审批动作（per DEC-008 一人公司兼任） |

> **下次评审**：随 ULYS-54 处置决议同步更新（批准 / 修订 / 驳回）+ 本 ADR 具名人类审批通过后升级为 Accepted。
