## 事实纠正：NATS JetStream **没有**结束官方支持

你大概率把 **NATS Streaming (STAN)** 和 **NATS JetStream** 混了 — 这是两个不同的东西，命运也不同。

---

### 1. 谁真的结束支持了

- **NATS Streaming (STAN)**：2021-04 宣布 deprecate，**2023-06 EOL**，repo 已 archive（[github.com/nats-io/nats-streaming-server](https://github.com/nats-io/nats-streaming-server)）
- STAN 是 NATS 的**前身**，独立的 server binary，已于 2023 退场

### 2. NATS JetStream 当前状态（2026-09 此刻）

来源：[docs.nats.io/release-notes/](https://docs.nats.io/release-notes/) + [synadia.com/legal/support](https://www.synadia.com/legal/support)

| 版本 | 发布日 | 状态 |
|---|---|---|
| 2.14 | 2026-04-30 | **Latest** |
| 2.13 | — | Skipped（从未发布）|
| 2.12 | 2025-09-22 | Maintained（**剩 ~6 天**，2026-09-22 到期）|
| 2.11 | 2025-03-19 | **End of life**（final v2.11.17，2026-04-27）|
| 2.15 | 2026-09（预期）| 即将发布 |

Synadia 支持策略：
- 每个 minor 版本支持 **1 年**
- 始终支持最近 **2 个 minor 版本**
- 当前受支持 = 2.12（剩 ~6 天）+ 2.14；2.15 出来后替换 2.12

**结论**：JetStream 活跃维护中，没有 EOL 信号。RGS TS-001 §3.6.1 写的 `NATS JetStream 2.10+` 偏旧，应升档 `2.14 (Latest)` — 顺带把 ADR-0060 §2 决定行版本号同步。

### 3. 2025 商标事件 ≠ EOL

2025 Synadia 想把 `nats-server` 改 BSL 重许可，CNCF 公开反对，最后协议：**nats-server 留 Apache 2.0 继续在 CNCF 下**。server 没换 license，没停维护。这是治理纠纷不是退场。详见 [svix.com: Is NATS Deprecated?](https://www.svix.com/resources/faq/is-nats-deprecated/)（2026-08-28 更新）

---

### 4. 那你的真问题是什么？请点一个

#### 选项 A：**撤销担忧，维持 ADR-0060 现状**
- JetStream 没 EOL，TS-001 §3.6.1 选型理由全部成立
- 我会做的两件小事：
  1. ADR-0060 §1.2 加一行「客户端语言 = Rust (`async-nats` 0.42）」回应你上轮提的调性问题
  2. ADR-0060 §2 + TS-001 §3.6.1 版本号从 `2.10+` 升档到 `2.14 (Latest)`
- §3 备选表（Kafka / RabbitMQ / Redis Streams / Pulsar 逐条否决）不动

#### 选项 B：**切回 Kafka（参考设计原意）**
- 撤 ADR-0060 候选（保持「待具名审批」不批），改立 ADR-0061「事件总线切回 Kafka」
- 选型链：Kafka 3.x + KRaft（去 ZooKeeper）+ `rdkafka` crate（C 绑定 librdkafka，需要 C 工具链）/ 或纯 Rust `kafka` crate（功能弱）
- **代价**：JVM 服务端 4–8GB 内存 / 镜像重 / 与 5 域 main.rs 的 `async-nats` 调用面全替换 / 共享层 `producer/consumer/outbox_relay` 三模块重写 / `shared-platform/Cargo.toml` 改依赖
- **收益**：回归上游参考设计 §2 + §11 原文 / 吞吐上限百万级 QPS / 选型文档链闭合（ADR-0061 §1 显式记录「参考设计本来就写 Kafka，前一版 ADR-0060 是偏离备案，本次是回归」）
- INV-001 v0.3 §4.3 同步联动：原「NATS 偏离归档」段改「NATS 偏离归档已撤，事件总线回归 Kafka」

#### 选项 C：**换别的（请指明候选）**
- RabbitMQ（`lapin` crate，纯 Rust，AMQP 0.9.1）
- Redis Streams（`redis` crate，纯 Rust；与已选 Redis 缓存栈同源，少一个组件）
- Apache Pulsar（`pulsar-client`，Rust 绑定有 C++ 依赖）

---

**当前 ADR-0060 状态**：候选待具名审批（per DEC-008 一人公司兼任）。**在你明确点 A / B / C 之前我不会自己改 §2 决定**。

如果 A，我直接动手改两处小版本号 + 加一行「客户端语言」；
如果 B，我重新打开 Stage 2 起草 ADR-0061（不在本工单范围内，会建议挂新工单 ULYS-55.B）；
如果 C，告诉我候选，我给你备选表 §3 的逐条否决/接受分析。

等你回。
