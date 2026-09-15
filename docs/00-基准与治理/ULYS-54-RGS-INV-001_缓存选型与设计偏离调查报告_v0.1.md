# 缓存选型与设计偏离调查报告 (ULYS-54)

**项目**: RustGameServer (RGS)
**调查范围**: 从技术选型文档 → 设计 → 实际部署全过程
**调查日**: 2026-09-15 JST
**调查者**: worker (ULYS-54 agent)
**任务来源**: ULYS-54「调查缓存使用 redis 而非 Valkey 的原因 + 横向展开找出与本份设计的出入点」

---

## 1. 调查方法

| 层级 | 检索范围 |
|---|---|
| 选型层 | `docs/10-技术选型/RGS-TS-001_主要技术选型报告.md` §3.5 + §5.1/5.2 |
| 治理层 | `docs/00-基准与治理/RGS-REQ-005_附件D_问题风险管理表.md` §4 OSS 许可盘点 |
| 需求层 | `docs/00-基准与治理/RGS-REQ-001_需求定义书.md` §6.1 数据分类 + §9 NFR + §10 ARC |
| 设计层 | `docs/01-核心架构与设计模式/RGS-BAS-001_基本设计书.md`、`RGS-DTL-006_详细设计书.md`、`RGS-ADR-0008_设立中间件导入判定基准.md`、`RGS-ADR-0052_Active-Active_ClusterOpsService…` |
| 部署层 | `docs/09-部署运维/RGS-OPS-001_保姆级部署说明.md` §1.3 + §2.3 + §2.5 |
| 业务层 | `docs/00-基准与治理/requirements/RGS-REQ-100_Saga事务系统需求定义书_v0.1.md` BR-111/BR-112 + §7 |
| 代码层 | `Cargo.toml` workspace + 全部 `crates/*/Cargo.toml` + 实际 `redis-rs`/`valkey-rs` 引用 |
| 适配层 | `docs/15-IPA-完全对齐438cmds/RGS-DDD-2026-09-04_v0.2.md`、`RGS-BDD-v0.2-addendum-frontend适配层.md` |

使用 `search_files` (content + files_only + count) 对 `redis|valkey|Redis|Valkey|REDIS|VALKEY` 做全文检索，结果 70 个文件命中 147 处。在此基础上做了语义分层（决定层 / 引用层 / 占位层 / 误用层）。

---

## 2. 核心结论 (TL;DR)

**Redis vs Valkey 偏离是表象，根因是「选型报告擅自覆盖了需求登记册的决议」。** 偏离链贯穿 5 个文档层、1 个部署层、5 个业务层：

1. **RGS-REQ-005 附件D §4**（OSS 许可盘点）已把 **Valkey** 登记为「缓存基础设施」合规组件，备注「Redis 2024年许可变更后的分支」——即选 Valkey 是因为 Redis 上游许可变更。
2. **RGS-TS-001 §3.5.1**（技术选型报告 v0.7，2026-08-24）改选 **Redis 7.2+**，理由「License 切换为 RSAL/SSPL 后仍可自托管」——直接抵消了 REQ-005 的选型，且**未触发 RGS-ADR-0008 闸门**。
3. 此后所有下游文档（BAS-001、DTL-006、OPS-001、ADR-0052、REQ-100、CAP-001）全部跟随 Redis，再无人复审上游决议。
4. **代码层至今零 Redis 客户端依赖**——所有 `crates/*/Cargo.toml` 均无 `redis`/`fred`/`valkey` crate，实际集成从未发生；偏离纯粹是「纸面偏离」。
5. 同一调查还发现 6 处与「参考设计 v2」的结构性偏离：消息总线 NATS JetStream（参考设计：Kafka）、CDC Debezium 缺失、Transactional Outbox 自研轮询替代、Workflow 自研 Saga Runtime（参考设计：Temporal）、API 网关 OpenResty 兼任（参考设计：独立 Envoy）、智能层 L4 仍保留（参考设计：未提）。

详细见 §3–§5。

---

## 3. Redis vs Valkey 偏离根因追溯

### 3.1 时间线（按登记册版本倒推）

| 日期 | 文档 | 关键行 | 决议 |
|---|---|---|---|
| 2026-08-15 | RGS-REQ-001 §6.1 DR-003 + NFR-MI-005 | `缓存基础设施` (角色名, 不用产品名) | 缓存 = DR-003, 可容忍丢失, 必须可重建; 抽象边界可替换 |
| 2026-08-19 | RGS-REQ-002 §A 术语表 L169 | `缓存基础设施 \| Valkey、Redis` + 例外「已确定采用产品的设计书中可使用产品名」 | 候选名单: Valkey + Redis |
| 2026-08-19 | **RGS-REQ-005 附件D §4 OSS 许可盘点** L452 | `缓存基础设施 \| **Valkey** \| BSD 3-Clause \| 合规 \| Redis 2024年许可变更后的分支` | **合规组件已选定 Valkey**, 理由即 Redis 上游许可变更 |
| 2026-08-20 | RGS-REQ-100 §7 + BR-111 | 「禁止 Redis Enterprise / 云专有 / 商业 SaaS / 闭源事务协调器」「如 Redis 功能确实需要, 评估纯开源替代方案 (KeyDB / Dragonfly / 自研 PostgreSQL-based 缓存)」 | 自上而下的「不用 Redis Enterprise / 评估开源替代」约束, Valkey 显然在「开源替代」内 |
| 2026-08-21 | RGS-ADR-0008 中间件导入判定基准 §3 | 引入任何新中间件前须证明 ①既有组件无法承担 ②职责确属必要 ③完成 OLU 估算 | 闸门成立, 但**未对 Valkey→Redis 的回退做 ADR** |
| **2026-08-21** | **RGS-TS-001 §3.5.1 缓存** (v0.4/v0.5/v0.6/v0.7 均为同口径) | `【已决】Redis 7.2+ Cluster` + 理由「License 切换为 RSAL/SSPL 后仍可自托管」 | **改选 Redis, 否决 REQ-005 已登记的 Valkey** |
| 2026-08-22 | RGS-DTL-006 §4 多层速率限制 | INCR+EXPIRE Lua 脚本走 Redis key | 算法按 Redis 写, 无可替换抽象 |
| 2026-08-24 | RGS-TS-001 v0.7 §3.6.1 | 事件总线决策升「已决策:NATS JetStream」 | 事件总线升档同时, 缓存未做同步复审 |
| 2026-08-25 | RGS-BAS-001 §9.1.1 + RGS-DTL-006 同步升版 | 错误日志字段 `infra.kind` 列出 `postgres/nats/redis` | Redis 渗入日志 schema 命名 |
| 2026-08-25 | RGS-ADR-0052 §5.2.2 分布式锁 | `cluster-ops distributed lock (Redis Redlock 或 NATS KV lock), 5s lease` | 强依赖 Redis Redlock, 无 fallback |
| 2026-08-29 | RGS-CAP-001 §3.3 | `Redis Redlock 单实例约 50k QPS` | Redis Redlock 进入容量基线 |
| 2026-08-29 | `crates/i18n-service/src/service.rs` | Redis 缓存占位 = `Arc<Mutex<BTreeMap>>` + 5 分钟 TTL; 注释 `W36+ 替换为 redis-rs 真实 client + SETEX 5 分钟` | 占位而非实装, 但已锁定「真实 client = redis-rs」 |
| 2026-08-29 | RGS-DDD-2026-09-04_v0.2 §3 + §5 | leaderboard 走 Redis sorted set; arena ListRankings 走 Redis P99 < 10ms; 6 个域写 `redis cache + sqlx 兜底` | Redis 作为多个域的热路径 |
| 2026-09-04 | RGS-BDD-v0.2-addendum-frontend适配层 §1.2 + §7.2 | `不引入 Kafka / Redis: 跟 闪烁之光 现状一致`; `session 持久化: 适配层本地 LRU cache (moka) + Redis 备份 (v0.3+ 评估, per BDD v0.1 §1.3 不引入 Kafka/Redis 一致性)` | **此处出现内部矛盾**: 一面说「不引入 Redis」, 一面又把 Redis 列为 v0.3+ 的备份存储 |
| 2026-09-XX | RGS-OPS-001 §1.3 + §2.3 + §2.5 | `Redis 7.2+` + `image: redis:7-alpine` + `REDIS_URL=redis://...` | 部署层固化 Redis, 镜像版本钉到 `redis:7-alpine` |

### 3.2 偏离的「双轨」证据

REQ-005 选 Valkey、TS-001 选 Redis, 但两个文档都没互引对方的决议。两份都标「已决 / 合规」。这就是双轨——同一决议在两份独立登记册上各自表述。

### 3.3 TS-001 §3.5.1 给出的 3 个理由, 逐条反验

| TS-001 §3.5.1 理由 | 事实核对 | 反验结论 |
|---|---|---|
| 「满足 ARC-013 背压设置位置（限流计数器 / 在线状态 / 排行榜热数据）」 | ARC-013 全文检索: 限定为「背压 / 死锁防止」; 限流 / 排行榜属 ARC-012「缓存 / 临时状态适用边界」; 不是 ARC-013 | **理由张冠李戴** — ARC-013 不背书限流计数器, ARC-013 背的是连接级 / Mailbox 限流, 在 Rust 进程内即 |
| 「成熟稳定; License 切换为 RSAL/SSPL 后仍可自托管」 | Redis 7.2+ 起的双重许可确实允许自托管（非 SaaS 阻断）; 但 REQ-005 同一性质 Redis 已判定为「Redis 2024年许可变更后的分支」, 选 Valkey 即为规避这一变更 | **理由选择性忽略上游变更** — 自托管 ≠ 无变更负担; RSAL/SSPL 变更影响的是 Redis 8.x+, 而 7.2 仍可保留旧 BSD, 这是钻版本空子, 不是合规论证 |
| 「Lua 脚本支持复杂原子操作」 | Valkey 7.x 完整继承 Redis 7.x Lua 语义 + `FUNCTION` + `EVALSHA` | **理由不构成选型差异** — Valkey 也支持, 等价 |

**3 个理由中 2 个无效, 1 个选择性忽略上游合规诉求。** 没有满足 RGS-ADR-0008 §2 闸门条件 ①「既有组件 (Valkey) 无法承担该职责」——理由是 Valkey 是 Redis 7.2 的直接 fork, 字节级兼容, 不可能「无法承担」。

### 3.4 偏离责任归属

按 RGS-IMPL-001 工程边界 + RGS-ADR-0008 §4「驳回须写明未满足哪一条」的反向解释:

- **TS-001 v0.4 (2026-08-21) 的作者**应承担主要责任: 改选了已登记组件, 既没复审 REQ-005, 也没立 ADR-xxxx 「Redis vs Valkey」单点决定, 也没在 TS-001 v0.5~v0.7 三次升版中回头补救
- **RGS-ADR-0008 的闸门**未被触发: 因为按 ARC-014 口径「未证明需要不引入」针对「新引入」, 而 TS-001 把 Redis 包装成「已决选型」规避了 ADR 流程
- **下游文档作者**（BAS-001 / DTL-006 / ADR-0052 / CAP-001 / OPS-001 / DDD-2026-09-04）依次引用, 缺乏逐级回检机制

**没有 ADR-xxxx 「Redis 取代 Valkey 的决定」**。这就是结构性问题: 选型报告绕过 ADR 流程改了上游决议。

### 3.5 实际代码层证据

| 路径 | 内容 |
|---|---|
| `Cargo.toml` workspace deps | 仅 `tokio / serde / sqlx / rustls / tonic / opentelemetry / async-nats / lettre / wasmtime / wat / ctor`; **无 redis / valkey / fred / deadpool-redis / bb8-redis** |
| `crates/*/Cargo.toml` (32 个 crate 全检) | 仅 `crates/i18n-service/Cargo.toml` L9 描述含「Redis 缓存」字样, 无实际依赖 |
| `crates/i18n-service/src/service.rs` L53-77 | `TtlCache` = `Arc<Mutex<BTreeMap>>`, 注释 `W36+ 替换为 redis-rs 真实 client + SETEX 5 分钟` — **占位, 未实装** |
| `crates/leaderboard-service/src/service.rs` (1 处 Redis 提及) | 仅出现在注释 / 文档, 无真实依赖 |
| `crates/match-service/migrations/0040_game_sessions.sql` | 1 处提及, 无 Redis 函数 |
| `crates/cluster-ops/src/realm_lifecycle/feature_adapter.rs` (2 处) | 注释提及, 无依赖 |

**结论: 截至调查日, 缓存层偏离纯粹是「纸面偏离」, 未产生代码侧影响。** 但 RGS-OPS-001 §2.3 的 `docker-compose.dev.yml` 已固化 `image: redis:7-alpine`, 一旦开发者按此启动, Redis 客户端依赖会被引入到 lockfile, 届时从 Valkey 替换回 Redis 的成本会显著上升。

---

## 4. 横向偏离扫描 (与上游需求 + 与参考设计 v2)

把视野放大到「与上游需求 + 与本次任务的参考设计 v2」两个方向, 共发现 6 处结构性偏离。

### 4.1 偏离对照表

| # | 维度 | 上游 / 参考设计 | RGS 实际 | 偏离类型 | 严重度 |
|---|---|---|---|---|---|
| 1 | **缓存组件** | 上游 REQ-005 选 Valkey; 参考设计 §2 选 Valkey | TS-001 §3.5.1 选 Redis 7.2+ | **直接违反上游决议** | 🔴 P0 |
| 2 | **事件总线** | 参考设计 §2 + §11: Apache Kafka | TS-001 §3.6.1: NATS JetStream 2.10+ (经 Q-M-10 升「已决策」) | 替代实现 + 已显式决策 | 🟡 P1 (有 ADR 决策路径, 但与参考设计偏差大) |
| 3 | **CDC / Outbox** | 参考设计 §16-18: PostgreSQL Outbox + Debezium CDC → Kafka | REQ-100 + ADR-0015: 自研 Outbox 模式 + 5 状态机 (saga_runtime + outbox_worker), **未引入 Debezium** | 替代实现 + 自研, 未引入 Debezium | 🟡 P1 (自研路径有文档论证, 但偏离参考设计) |
| 4 | **Workflow / Saga** | 参考设计 §10 + §21: Temporal (跨服务长事务 / Saga) | BAS-100 + REQ-100: 自研 Saga Runtime (K3s Deployment, PostgreSQL `saga_fence_token_seq`, 拒绝 Temporal / Cadence 商业版) | **替换核心组件 + 自研** | 🟠 P1-P2 (与 BR-111「纯开源」一致, 但 Temporal 也是 MIT 开源, 是选择问题不是合规问题) |
| 5 | **API 网关 / 边缘** | 参考设计 §2 + §9: Envoy (HTTP/gRPC 流量治理) | TS-001 §3.10.1 + DTL-006 §7: OpenResty (边缘反代 + WAF Coraza); Envoy **未引入** | 替代实现 + 未引入 Envoy | 🟢 P2 (与参考设计路径不同, 但符合「未证明需要不引入」ARC-014) |
| 6 | **智能层 (L4)** | 参考设计全文未提智能层 / LLM / Agent 概念 | TS-001 §3.8 + ADR-0026 + ADR-0029 + ADR-0053 + ADR-0054: Python 3.11 + LangGraph + LiteLLM + 自研 Agent 平台底座 + 多 Agent 体系 | **额外引入未在参考设计内的复杂子系统** | 🔴 P0 (5 份 ADR + 1 个域 + 6+ 文档, 是 RGS 最大的范围扩展, 违背参考设计「先完成垂直闭环」原则) |

### 4.2 偏离 1 (Redis vs Valkey) 的细节

详见 §3。

### 4.3 偏离 2 (NATS JetStream vs Kafka)

| 项 | Kafka | NATS JetStream |
|---|---|---|
| 持久化 | Log 段文件, 长期保留 | Stream + Consumer 持久化, 可配置保留 |
| 顺序保证 | Partition 内强顺序 | Stream 内消息有序 |
| 吞吐量 | 百万级 QPS | 十万级 QPS (单节点) |
| 运维负担 | JVM + ZooKeeper/KRaft, 4-8GB 内存 | Go 单二进制, 50-200MB 内存 |
| DEC-005/006 是否授权 | 否 | 是 (Q-M-10 答复 + ACTIONS-v0.3 B-09) |
| 与参考设计契合度 | ✅ 100% 契合 | ⚠ 替代实现 |

**评估**: RGS 主动选 NATS 是「运维简化 > 吞吐上限」的合理 trade-off, 与 ARC-014 / OLU 约束一致, 且有显式 DEC 决策链。但偏离参考设计的事实需在交付时向上声明。

### 4.4 偏离 3 (自研 Outbox vs Debezium CDC)

参考设计 §18 明确写 `PostgreSQL WAL → Debezium → Kafka`。RGS 现状:

- **已实装**: `crates/shared-platform/src/outbox.rs` (基于 sqlx 的 5 状态机 outbox_worker, 经多文档验证)
- **未引入**: Debezium (无 K8s manifest, 无 ADR, 无 Cargo 依赖)
- **决策链**: ADR-0015 Saga 边界 + RGS-REQ-100 §7 (BR-111) 隐含「不引入 Debezium」

**评估**: 自研 Outbox 轮询在 5 域架构下 QPS 充足 (REQ-001 §9.2 NFR-PE-011 可达标), 但**失去了 Debezium 的「捕获所有 DB 变更」能力**——Outbox 只能捕获「显式写入 outbox 表」的事件, 不在事务内的 DB 变更不会被传播。这是参考设计强调 Debezium 的核心原因。RGS 把 outbox 写入强制在事务内 (`BEGIN; 业务修改; INSERT Outbox; COMMIT;`) 是规避方案, 但需要 ADR-0059 级别的「Debezium 不引入决议」正式化。

### 4.5 偏离 4 (自研 Saga Runtime vs Temporal)

参考设计 §10 + §21 明确 `Temporal → 跨服务长事务 / Saga`。RGS 现状:

- **已实装**: 自研 saga_runtime (BAS-100 §3 Saga Runtime 内部模块: Engine / State Machine / Scheduler / Retry / Timeout / Compensation / Recovery / Event Router), K3s Deployment 3+ replicas
- **决策链**: RGS-REQ-100 §7 方案 C「saga-runtime 完全外包 (e.g. Temporal / Apache Airflow) 拒绝」+ 理由「不绑闭源 Saga 协调器」+ 「纯开源约束」

**评估**: 
- Temporal 当前是 **MIT 开源** (temporal.io 仍维护开源版, 仅 Temporal Cloud 是 SaaS), 不属「闭源事务协调器」, RGS 拒绝 Temporal 的理由失实
- BR-111「纯开源约束」成立, 但 Temporal 满足
- RGS 选择自研的代价: ~5 份 ADR + 2 份 REQ/BAS + 9 张 saga 表 + 一整套状态机 + 恢复机制, OLU 极高
- 这是 **未充分证明「自研优于 Temporal」** 的典型反 ARC-014 案例 (与 §3.4 同结构)

### 4.6 偏离 5 (OpenResty vs Envoy)

参考设计 §2 写 `Envoy → HTTP/gRPC 流量治理`。RGS 现状: 边缘反代 = OpenResty 1.21+, 未引入 Envoy; ADR-0044 + ADR-0052 已显式否决 Envoy 作为边缘。

**评估**: 与参考设计路径不同但合理——Envoy 在 RGS 单语言架构下确实过度, ARC-014 闸门成立。🟢 接受。

### 4.7 偏离 6 (智能层 L4 引入) — 最大的范围扩展

参考设计全文未提及智能层 / LLM / Agent / Python / LangGraph / 向量存储。RGS 现状:

- **已实装范围**: TS-001 §3.8 + ADR-0026 + ADR-0029 + ADR-0053 + ADR-0054 + REQ-033 + BAS-033 + BAS-035 (推断) + LangGraph + LiteLLM + Python 3.11 子系统 + 多个 Agent 域
- **新增组件数**: 5+ 份 ADR, 1+ 份 REQ/BAS, 1 个 Python 子系统, 多个 Agent 平台底座 crate
- **决策链**: DEC 多次拍板, 有 ADR-0026/0029 论证「L4 只读感知 + 闸门」

**评估**: 这是 RGS 与参考设计之间 **最大的范围偏差**, 性质属「未在原 prompt 中授权的子系统扩张」。CR-011 (per ADR-0026 关联) 状态未明。违反参考设计 §39「过度设计防护: AI Agent 禁止主动引入未证明需要的抽象层」。

**注意**: 智能层本身合规 (Apache-2.0/MIT/BSD), 问题不在许可, 在范围。

---

## 5. 建议 (按优先级)

### 5.1 立即处理 (P0, 阻断 Phase 1+ 推进)

1. **撤销 TS-001 §3.5.1 的 Redis 决议, 恢复 Valkey (per REQ-005 附件D §4 上游登记)**
   - 动作: TS-001 v0.8 §3.5.1 改 `【已决】Valkey 7.2+ (per REQ-005 §4 上游登记, Redis 2024 许可变更后的合规 fork)`, 备注链向 REQ-005 + REQ-100 §7 BR-111
   - 同步: RGS-OPS-001 §1.3 / §2.3 / §2.5 改 `image: valkey:7-alpine` (Valkey 官方 Docker Hub 镜像兼容 `redis-cli`), `REDIS_URL` → `VALKEY_URL` (注意: 协议兼容, 客户端无需改)
   - 同步: RGS-BAS-001 §9.1.1 + RGS-ADR-0052 §5.2.2 + RGS-DTL-006 §4 改 `infra.kind=valkey` 或抽象为 `cache.kind` (推荐后者, 留可替换空间 per NFR-MI-005)
   - 同步: RGS-CAP-001 §3.3 改 Redis → Valkey QPS 数据
   - 立 ADR-0059 「缓存组件决议统一 (Valkey)」正式撤销 TS-001 §3.5.1 的偏离
   - 备注: 由于代码层零 Redis 依赖, 替换零成本

2. **撤销智能层 L4 范围扩展, 重新论证 CR-011**
   - 动作: DEC 决议把智能层从「基础架构必含」降级为「PH-12+ 可选」
   - 同步: ADR-0026 / 0029 / 0053 / 0054 改 status 为「Deferred / Hold per 参考设计 §39 过度设计防护」
   - 理由: 参考设计 §4「先做可运行垂直闭环」+ §39「AI Agent 禁止主动引入未证明需要的抽象层」均未授权智能层
   - 风险: 已投入 ~5 份 ADR + 多个 crate 的 OLU 沉没成本

### 5.2 中期处理 (P1, 不阻断但需正式化)

3. **正式化 NATS JetStream vs Kafka 偏离**
   - 动作: 立 ADR-0060 「事件总线偏离参考设计: NATS JetStream 取代 Kafka」, 显式记录 trade-off (运维简化 vs 吞吐上限)
   - 备注: 与 DEC-005/006 链一致, 仅作正式归档

4. **正式化「Debezium 不引入」决议**
   - 动作: 立 ADR-0061 「自研 Outbox 取代 Debezium CDC」, 记录自研路径的「事务内强制 outbox 写入」约束 + 已捕获事件族的范围 (目前 5 域 + cluster_ops + shared_platform)

5. **重新评估 Temporal vs 自研 Saga Runtime**
   - 动作: 立 ADR-0062 「Saga Runtime 偏离参考设计: 自研路径 vs Temporal (MIT 开源)」
   - 关键论证: Temporal 当前为 MIT 开源 (temporal.io), 满足 BR-111「纯开源约束」; RGS 拒绝理由「不绑闭源事务协调器」失实
   - 决策建议: 若自研路径已投入 > 50% OLU, 维持自研 + 立 ADR; 若尚未投入, 重新评估 Temporal 的开发成本节约
   - 备注: 与 §3.4 / §4.5 同结构问题——未充分论证即绕过 ARC-014

### 5.3 长期处理 (P2, 接受偏离)

6. **OpenResty 取代 Envoy** 不需处理, 与参考设计偏差合理, ADR-0044 已显式记录

---

## 6. 附录

### 6.1 全文检索命中清单 (按命中数排序, 70 文件 147 处)

| 命中数 | 路径 | 类别 |
|---|---|---|
| 28 | docs/09-部署运维/RGS-OPS-001_保姆级部署说明.md | 部署层 (Redis 镜像 + URL + 端口) |
| 11 | docs/02-运维安全与网络/RGS-DTL-006_详细设计书.md | 详细设计 (Redis Lua 限流) |
| 10 | docs/10-技术选型/RGS-TS-001_主要技术选型报告.md | 选型层 (源头) |
| 8 | docs/15-IPA-完全对齐438cmds/RGS-DDD-2026-09-04_v0.2.md | 业务层 (6 域 Redis 用法) |
| 7 | docs/08-架构决策记录/RGS-ADR-0052_Active-Active_ClusterOpsService… | ADR (Redis Redlock) |
| 6 | crates/i18n-service/src/service.rs | 代码占位 (Arc<Mutex<BTreeMap>>) |
| 5 | docs/15-IPA-完全对齐438cmds/RGS-DDD-v0.2-addendum-业务逻辑逆推.md | 业务层 (leaderboard / 逆推映射) |
| 3 | docs/15-IPA-完全对齐438cmds/RGS-BDD-v0.2-addendum-frontend适配层.md | 适配层 (内部矛盾 §1.2 + §7.2) |
| 3 | docs/11-实施QA/RGS-QA-001_实施前QA表_v0.13.md | QA |
| 3 | docs/01-核心架构与设计模式/RGS-INC-001_增量式架构升级_Function与WASM演进方案_v0.2.md | 演进方案 |
| 3 | docs/02-运维安全与网络/RGS-GOBS-001_现有游戏服务器可观测性增强现状调查书.md | 可观测性现状 |
| 2 | docs/10-技术选型/RGS-CAP-001_ClusterOps容量基线_v0.1.md | 容量基线 |
| 2 | docs/14-项目治理/RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md | 项目治理 |
| 2 | docs/12-工作流/RGS-WT-001_GitWorktree隔离开发方案.md | 工作流 |
| 2 | crates/i18n-service/src/lib.rs | 代码 |
| 2 | crates/cluster-ops/src/realm_lifecycle/feature_adapter.rs | 代码 |
| 其余 (命中数 1) | 49 个文件 | 散布 |

### 6.2 RGS-REQ-005 附件D §4 OSS 许可盘点 L452 原文

```
| 缓存基础设施 | Valkey | BSD 3-Clause | ○ | **合规** | Redis 2024年许可变更后的分支 |
```

**这是上游登记, 具法律意义, TS-001 §3.5.1 改选 Redis 时未复审此行。**

### 6.3 参考追溯 — 所有「Redis」命名的 4 个语义层

| 语义层 | 出现处 | 是否可改 Valkey | 备注 |
|---|---|---|---|
| 选型登记 (合规决议) | REQ-005 L452 | ✅ 直接 | 已合规 |
| 选型决议 (技术选型) | TS-001 §3.5.1 | ✅ 直接 | 需撤销 + 立 ADR-0059 |
| 算法设计 (INCR/EXPIRE/Lua) | DTL-006 §4 | ✅ 直接 | API 字节级兼容, 仅改命名 |
| 分布式锁 (Redlock) | ADR-0052 §5.2.2 | ✅ 直接 + 备选 NATS KV | Valkey Redlock 已有实现 |
| 容量基线 | CAP-001 §3.3 | ✅ 直接 | 同 QPS 量级 |
| 部署镜像 | OPS-001 §2.3 | ✅ 直接 | `image: valkey:7-alpine` |
| 代码占位 | i18n-service/src/service.rs | ✅ 直接 | 占位未实装, 改注释即可 |
| 业务域反推 | DDD-2026-09-04 §3/§5 + BDD-v0.2 §1.2/§7.2 | ⚠ 内部矛盾需消解 | BDD §1.2「不引入」与 §7.2「v0.3+ 评估」冲突 |

**全栈可改 Valkey, 无技术阻塞, 主要成本在文档更新。**

### 6.4 调查者备注

- 本报告不替代 ADR; 任何决议变更须先立 ADR, 再更新本文档对应行
- 调查方法 (全文检索 + 文档交叉引用 + 代码 grep) 可重复, 后续类似任务可按本模板扩展
- 限于单次会话上下文, §4.3-4.7 的偏离细节仅做摘要, 详细论证需后续工单立项

---

## 7. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-09-15 JST | worker (ULYS-54 agent) | 初版制定。覆盖 Redis vs Valkey 偏离根因追溯 + 横向 6 处偏离扫描 + 建议 6 项 + 附录 4 节 |

> **下次评审**: 随 ULYS-54 处置决议同步更新 (取消 / 修订 / 关闭)



