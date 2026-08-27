# 基本设计书（基本設計書 / Basic Design Document）

**超限排队与客服邮箱告警 — 弹性容量规划（BAS-022）补强**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-022-ADD1 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-022 弹性容量规划与超大规模并发架构 基本设计书（v0.2，2026-08-16） |
| 增补类别 | 新增软/硬双阈值 + NATS JS 队列 + SMTP 告警降级 |
| 制定日 | 2026-08-27 |
| 制定者 | 架构师(Mavis 接手 agent per DEC-008) |
| 审批者 | 架构师(Mavis 接手 agent per DEC-008) |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

> 审批者代签依据：per 2026-08-26 08:40 JST Ulysses 规则反转，子代理 / Mavis 可在审批栏直接填写真实责任署名。

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|---|---|---|---|---|
| 0.1 | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版。补强 RGS-BAS-022 容量规划基本设计：明确"超限行为" = 双阈值限流 + NATS JS 排队 + SMTP 告警降级。引入 `crates/rgs-overflow-alert` 6 模块结构（domain/config/limiter/queue/alert/guard）。 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师(Mavis 接手 agent per DEC-008) | 2026-08-27 | — |
| 评审（技术） |  |  | 限流是 RGS-BAS-023 处理链的一个 Layer，不与既有 Layer 冲突 |
| 评审（业务/运营） |  |  | SMTP 缺密码降级路径与客服告警 SLA 关系 |
| 审批（负责人） |  |  | 与 BAS-022 容量规划正交；分片路由与超限行为解耦 |

---

## 目录

1. [前言](#1-前言)
2. [总体设计](#2-总体设计)
3. [模块设计](#3-模块设计)
4. [数据流](#4-数据流)
5. [配置](#5-配置)
6. [部署](#6-部署)
7. [与既有架构关系](#7-与既有架构关系)
8. [测试策略](#8-测试策略)
9. [风险与缓解](#9-风险与缓解)
10. [缺标清单](#10-缺标清单)

---

# 1. 前言

## 1.1 目的

RGS-BAS-022 v0.2 已规定"弹性容量规划"的 T0~T3 容量分级与跨分片基本设计，但**未规定"超并发上限时的具体行为"**——是直接拒绝、异步排队、还是 admission control？本文档补齐这一空白，明确基本设计：

- **超并发上限时**触发**入队 + 告警**而非直接拒绝
- **入队后端**统一为 NATS JetStream（沿用 DEC-011 既有基础设施）
- **告警**走 SMTP 向客服邮箱发邮件，`SMTP_PASSWORD` 缺失时降级为结构化日志
- **域作用范围**严格限定为 4 个业务服务（player / economy / match / social），admin / cluster-ops 排除

## 1.2 与 BAS-022 母本关系

| 母本主题 | 本补强关系 |
|---|---|
| 容量分级 T0~T3 | 决定"正常情况下的分布"；本补强实现"超限兜底行为" |
| 横向分片 | 与本补强正交；分片路由与超限行为解耦 |
| 快速扩容 | 本补强不涉及；扩容是事前手段，超限是事后兜底 |

## 1.3 与下游 DTL 关系

- DTL 详细设计：`RGS-DTL-022-ADD1`（本主题的详细设计书，含代码骨架）

---

# 2. 总体设计

## 2.1 架构图

```
                  ┌──────────────────────────────────────┐
                  │  业务 RPC handler (player/economy/    │
                  │  match/social 4 域 service.rs)        │
                  └─────────────────┬────────────────────┘
                                    │ g.check(op, req_id, body)
                                    ▼
                  ┌──────────────────────────────────────┐
                  │  OverflowGuard（高层 API 入口）       │
                  │  - 编排 limiter / queue / alerter     │
                  └──────┬───────────┬──────────────┬────┘
                         │           │              │
              try_acquire│           │enqueue       │notify
                         ▼           ▼              ▼
              ┌──────────────┐ ┌──────────────┐ ┌─────────────┐
              │ OverflowLimiter│ │NatsJsQueueBck│ │AlertDedup  │
              │ (双阈值 CAS)   │ │(JS stream)   │ │(窗口去重)  │
              │ in_flight 计数 │ │ RGS_OVERFLOW │ └─────┬──────┘
              └──────────────┘ └──────┬───────┘       │
                                       │               │
                                       │               ▼
                                       │       ┌────────────────┐
                                       │       │ SmtpAlertSink   │
                                       │       │ (lettre 0.11)   │
                                       │       └────────┬───────┘
                                       │                │
                                       │         ┌──────▼──────┐
                                       │         │ LogOnlySink  │
                                       │         │(降级路径)    │
                                       │         └─────────────┘
                                       ▼
                          ┌─────────────────────┐
                          │  NATS JetStream     │
                          │  (DEC-011 既有)      │
                          │  subject filter:    │
                          │  rgs.*.overflow.v1  │
                          └─────────────────────┘
```

## 2.2 设计原则

1. **不引入新运维面**：NATS / SMTP 均沿用 RGS 既有依赖；不引入 Redis / Kafka
2. **类型系统防越界**：`Domain` 枚举无 admin / cluster-ops 变体
3. **降级而非失败**：SMTP 缺密码 / 失败 → LogOnlySink；NATS 不可达 → 不阻断业务
4. **每域独立配置**：per 2026-08-21 Ulysses 偏好，不允许"统一开关"

---

# 3. 模块设计

> 6 个模块位于 `crates/rgs-overflow-alert/src/`，每个模块 50~300 行（含单元测试）。

## 3.1 `domain` — 域抽象

**职责**：定义业务域枚举，编译期防越界。

**关键 API**：
- `enum Domain { Player, Economy, Match, Social }` — 4 个变体，**不**含 Admin/ClusterOps
- `Domain::as_str() -> &'static str` — 域小写名（用于 subject / env key）
- `Domain::env_max_inflight() -> &'static str` — 返回 env key（`PLAYER_MAX_INFLIGHT` 等）
- `Domain::ALL: [Domain; 4]` — 全部 4 域（迭代顺序 stable）

**测试**：`as_str_returns_lowercase` / `from_str_rejects_admin_and_cluster_ops` / `env_max_inflight_keys_match_dotenv`

## 3.2 `config` — 配置

**职责**：从 env 读取全部配置。

**关键 API**：
- `OverflowConfig::from_env() -> Result<Self, ConfigError>` — 一次性解析
- `OverflowConfig::hard_cap(d: Domain) -> u32` — 硬上限
- `OverflowConfig::soft_cap(d: Domain) -> u32` — 软阈值 = `ceil(hard × soft_ratio)`
- `OverflowConfig::smtp: SmtpConfig` — SMTP 配置块
- `SmtpConfig::password_is_empty() -> bool` — 密码空 → 走 LogOnlySink

**错误模型**：`ConfigError::InvalidSoftRatio` / `InvalidSmtpTimeout` / `InvalidMaxPending`（**仅**结构性错误；缺密码 / hard_cap=0 是合法降级，不是 error）

## 3.3 `limiter` — 限流

**职责**：双阈值 CAS 限流 + RAII permit 释放。

**关键 API**：
- `OverflowLimiter::new(domain, cfg) -> Self` — 构造（hard_cap=0 → 不启用）
- `OverflowLimiter::try_acquire() -> (AcquireOutcome, Option<InFlightGuard>)` — 同步获取
- `OverflowLimiter::in_flight() -> u32` — 当前 in-flight
- `OverflowLimiter::reject_count_5min() -> u64` — 5min reject 窗口
- `enum AcquireOutcome { Pass, Queued, Rejected }` — 三态
- `struct InFlightGuard` — RAII guard，drop 时自动减计数

**关键算法**（**修复过的 race bug**）：
```rust
// 不用 fetch_update 乐观重试（1000 并发下 in_flight 突破 hard）
// 用 compare_exchange + 不重试：CAS 失败直接 Rejected
let current = counter.load(Acquire);
if current >= self.hard { return Rejected; }
match counter.compare_exchange(current, current + 1, AcqRel, Acquire) {
    Ok(_) => { /* Pass or Queued */ }
    Err(_) => return Rejected,  // 不重试
}
```

## 3.4 `queue` — NATS JS 队列

**职责**：NATS JetStream 后端 + 内存后端（dev/test）。

**关键 API**：
- `trait QueueBackend: Send + Sync { async fn enqueue(...) -> Result<AckToken, QueueError> }`
- `NatsJsQueueBackend::connect(cfg) -> Result<Self, QueueError>` — 启动时 `get_or_create_stream`
- `NatsJsQueueBackend::subject_for(domain) -> String` — `rgs.<domain>.overflow.v1`
- `InMemoryQueueBackend::new(max_pending) -> Self` — dev/test 后端
- `struct AckToken { domain, sequence, enqueued_at }` — 入队后返回

**关键设计**：
- 复用 `shared_platform::messaging::build_messaging_client`（**不**自己引独立 NATS）
- 复用 `SubjectBuilder::domain_event(domain, "overflow", 1)`
- stream filter = `rgs.*.overflow.v1`（一个 stream 覆盖 4 域）
- 多副本并发启动的 stream create 竞态：依赖 `get_or_create_stream` 的 idempotency

## 3.5 `alert` — 告警

**职责**：邮件 sink + 日志 fallback + 窗口去重。

**关键 API**：
- `trait AlertSink: Send + Sync { async fn send(&self, to: &str, event: &AlertEvent) -> Result<(), AlertError> }`
- `SmtpAlertSink::new(cfg: &SmtpConfig) -> Result<Self, AlertError>` — lettre transport
- `LogOnlySink` — 永远不抛错，落 `tracing::warn!`
- `AlertDeduplicator::new(inner, fallback, to, window) -> Self` — 注入实际 sink + fallback
- `AlertDeduplicator::notify(&self, event: &AlertEvent)` — 窗口内同 key 跳过；SMTP 失败 → fallback
- `enum AlertKind { HardCapReached, SoftCapSurge, QueueFull, SinkFailure }`
- `struct AlertEvent { kind, domain, in_flight, hard_cap, soft_cap, queue_pending, pod, service, reject_count_5min, first_at, last_at }`

**邮件主题**：`[RGS-ALERT] <domain> overflow @ <RFC3339>`
**正文**：domain / in_flight / hard / soft / queue_pending / pod / service / 5min reject / first_at / last_at

## 3.6 `guard` — 业务层高层 API

**职责**：编排 limiter / queue / alerter，给业务 `OverflowGuard::check` 单点入口。

**关键 API**：
- `OverflowGuard::new(domain, cfg, limiter, queue, alerter, pod, service) -> Self`
- `OverflowGuard::check(op, request_id, business_json) -> OverflowDecision` — **业务主调用入口**
- `OverflowDecision { status: OverflowStatus, ack_token: Option<AckToken>, guard: Option<InFlightGuard> }`
- `enum OverflowStatus { Pass, Queued, Rejected }`
- `OverflowGuard::build_standard_sink(cfg) -> (primary, fallback)` — 业务 boilerplate
- `OverflowGuard::build_alerter(cfg) -> Arc<AlertDeduplicator>` — 业务 boilerplate

**关键设计**：**Queued 路径的 permit 保留到消费者处理完才 drop**（per FR-OFLOW-006）。`OverflowDecision.guard: Option<InFlightGuard>` 在 Queued 时也是 Some —— 消费者在 NATS 处理完消息后 drop。

---

# 4. 数据流

## 4.1 Pass 流

```
业务 RPC handler
  ↓ g.check(op, req_id, body)
OverflowGuard::check
  ↓ limiter.try_acquire() → (Pass, Some(permit))
  ↓ 返回 OverflowDecision { status: Pass, guard: Some(permit) }
业务：正常处理
  ↓ 处理完
业务 drop(decision.guard) → permit drop → in_flight -1
```

## 4.2 Queued 流

```
业务 RPC handler
  ↓ g.check(op, req_id, body)
OverflowGuard::check
  ↓ limiter.try_acquire() → (Queued, Some(permit))
  ↓ payload = OverflowPayload { ... }
  ↓ queue.enqueue(domain, payload) → Ok(ack)
  ↓ alerter.notify(SoftCapSurge) [首次]
  ↓ 返回 OverflowDecision { status: Queued, ack_token: Some(ack), guard: Some(permit) }
业务：立即返回 ResourceExhausted 给 client（带 ack token）
NATS 消费者（独立 task）：
  ↓ 从 stream 拉消息
  ↓ 业务处理消息
  ↓ 处理完 → drop(decision.guard) [permit] → in_flight -1
```

## 4.3 Rejected 流

```
业务 RPC handler
  ↓ g.check(op, req_id, body)
OverflowGuard::check
  ↓ limiter.try_acquire() → (Rejected, None)
  ↓ alerter.notify(HardCapReached) [首次 / 窗口内去重]
  ↓ 返回 OverflowDecision { status: Rejected, guard: None }
业务：返回 ResourceExhausted 给 client
```

## 4.4 状态转换

```
[空闲]
   │ in_flight + 1 ≤ soft
   ▼
[Pass] ──── 业务处理完 drop(guard) ──→ [空闲]
   │
   │ soft < in_flight + 1 < hard
   ▼
[Queued] ── 消费者处理完 drop(guard) ─→ [空闲]
   │
   │ in_flight + 1 ≥ hard
   ▼
[Rejected] ── 立即，无 guard ──→ [空闲]
```

---

# 5. 配置

## 5.1 env 列表

完整 env 表见 RGS-REQ-025-ADD2 附录 A。

关键设计：
- 缺密码（`SMTP_PASSWORD=""`）是合法降级，不是 error
- 硬上限 = 0（`<DOMAIN>_MAX_INFLIGHT=0`）是合法"不启用"，不是 error
- 软阈值在 (0, 1] 区间外 → ConfigError

## 5.2 配置分类

| 类别 | env key | 默认值 | 备注 |
|---|---|---|---|
| 客服邮箱 | `SUPPORT_EMAIL` | `hanakagumi@gmail.com` | 可覆盖 |
| SMTP | `SMTP_HOST/PORT/USER/PASSWORD/FROM_NAME/TIMEOUT_MS` | Gmail 默认 | 密码空 = 降级 |
| NATS | `NATS_URL_IN_CLUSTER` / `NATS_URL_LOCAL` | shared-platform 既有 | 沿用 |
| 限流 | `NATS_OVERFLOW_SOFT_RATIO` / `MAX_PENDING` / `STREAM` / `CONSUMER_GROUP` | 0.8 / 10000 / `RGS_OVERFLOW` / `rgs-overflow-workers` | |
| 告警 | `ALERT_DEDUP_WINDOW_SECS` | 60 | |
| 4 域硬上限 | `PLAYER/ECONOMY/MATCH/SOCIAL_MAX_INFLIGHT` | 0（不启用） | 0 = 不启用 |

---

# 6. 部署

## 6.1 k3s 集群

- `crates/rgs-overflow-alert` 作为 workspace member，与 5 域业务服务同 namespace `rust-game-server`
- 4 域业务服务 `Cargo.toml` 加 `rgs-overflow-alert = { path = "../rgs-overflow-alert" }`
- 业务服务 `main.rs` 启动时构造 `OverflowGuard`（Arc 共享），通过 `State` 注入 tonic handler

## 6.2 k8s Secret / ConfigMap

- `configmap.yaml` 加 4 个 env：`SUPPORT_EMAIL` / `SMTP_HOST` / `SMTP_USER` / `SMTP_FROM_NAME`（**不**挂 `SMTP_PASSWORD`）
- `secret.yaml` 模板加 `SMTP_PASSWORD` 引用（key = `rgs-smtp-password`，真实值由运维注入）

## 6.3 helm values

每个业务域 `values.yaml` 加 `overflow` 段：
```yaml
overflow:
  maxInflight: <N>      # 0 = 不启用；> 0 = 硬上限
  softRatio: 0.8
  alertSink: "log"      # dev 默认 log，staging/prod 默认 smtp
```

---

# 7. 与既有架构关系

## 7.1 互引（**全引** per Ulysses 拍板）

| 文档 | 关系 | 引用证据 |
|---|---|---|
| `RGS-REQ-025-ADD2` | 父需求 addendum | 见 `RGS-REQ-025-ADD2_超限排队与客服邮箱告警_需求定义书.md` |
| `RGS-BAS-022` v0.2 | 父基本设计 | git: `adb3e34` (feat docs finalize pre-implementation specification baseline) — 2026-08-16 |
| `RGS-BAS-023` v0.2 | **限流是处理链的一个 Layer** | git: `17b7522` (新增弹性容量规划与请求处理链标准化需求/基本设计) — 2026-08-16 |
| `RGS-BAS-003` | 告警 sink 是运维与 GM 后台管控的一部分 | git: 与 BAS-022 同期提交组（`adb3e34` 之前）— **缺标**，见 §10 |
| `RGS-BAS-009` | 体系治理与横切关注点 | git: `adb3e34` 提交组（与母本同期） |

## 7.2 与 ARC-040 / ARC-041 关系

| ARC | 关系 |
|---|---|
| ARC-040 容量分级 | 本补强实现其"超限兜底行为" |
| ARC-041 请求处理链 | 本补强的限流是处理链的一个 Layer |

## 7.3 与下游详细设计关系

- 详细设计：`RGS-DTL-022-ADD1`（含代码骨架 + 实际代码对账表）

---

# 8. 测试策略

## 8.1 单元测试（30 个，位于 `crates/rgs-overflow-alert/src/<module>.rs`）

每个模块 ≥ 1 happy + ≥ 1 降级路径：
- `domain`: 5 个测试（as_str / env_max_inflight / from_str round_trip / case_insensitive / 拒绝 admin+cluster_ops / ALL 4 域）
- `config`: 5 个测试（defaults / 拒绝 invalid_soft_ratio / soft_cap_scales / hard_cap_zero_disables / smtp_password_empty）
- `limiter`: 3 个测试（disabled_when_hard_cap_zero / pass_below_soft_then_queued_then_rejected / release_restores_permits）
- `queue`: 4 个测试（in_memory_queue_enqueues_and_caps / subject_format_matches_shared_platform / ack_token_round_trip / nats_js_backend_subject_filter_covers_4_domains / config_streams_name）
- `alert`: 5 个测试（dedup_window_suppresses / sink_failure_falls_back / different_kinds_independent / log_only_never_fails / dedup_key_isolates_domains / subject_format / body_contains_required_fields）
- `guard`: 4 个测试（pass_below_soft / queue_above_soft_below_hard / reject_above_hard / queue_full_falls_back_to_reject）

## 8.2 集成测试（5 个，位于 `crates/rgs-overflow-alert/tests/integration_overflow.rs`）

- `integration_4_domains_use_independent_subjects`
- `integration_queue_full_transitions_to_reject`
- `integration_alert_dedup_suppresses_storm`
- `integration_1000_concurrent_pass_queue_reject_distribution` — 验证 1000 并发下 limiter 不 race
- `integration_disabled_when_all_hard_caps_zero`

## 8.3 4 域挂点测试

每域 `service.rs` 内 mock limiter + mock queue + mock sink，验证：
- 软上限内 → handler 正常返回
- 超软上限 → handler 返回 enqueue ack
- 超硬上限 → handler 返回 ResourceExhausted + 调用 alert sink ≥ 1 次

## 8.4 端到端验证

`cargo test -p rgs-overflow-alert` 35/35 全过 + `cargo check --workspace` 通过。

---

# 9. 风险与缓解

详细见 `crates/rgs-overflow-alert/RISKS.md`。本节摘要：

| RISK | 缓解 |
|---|---|
| SMTP_PASSWORD 缺失 → 告警只落日志 | 文档显式说明"生产前必须补"；LogOnlySink 路径延迟 ≤ 1ms |
| 5 域 RPC 风格差异 | DTL addendum 约定"unary → ResourceExhausted；streaming → await permit" |
| NATS stream 竞态 | 依赖 `get_or_create_stream` idempotency；多副本启动压测待补 |
| Queued permit 不 drop | DTL addendum 强化消费者处理完 drop 的约定；RISKS 列出 |
| 告警去重吞重大事件 | RISKS 列出；生产前加 escalation |
| 4 域 hard_cap 推荐值 | 当前全部 0；待 Ulysses 拍板 |

---

# 10. 缺标清单（per 2026-08-26 DDD Review 必查）

> **缺标比错标安全**：以下引用在 git history 中**未找到独立 commit 关联**，仅在母本或同期提交中提及。RGS 文档治理规则要求显式列出。

| 引用 | 缺标原因 |
|---|---|
| `RGS-BAS-003` 告警 sink 引用 | BAS-003 在 02-运维安全与网络目录，git: 与 BAS-022 同期提交组（`adb3e34` 之前版本），与本主题无直接交叉 |
| `RGS-BAS-009` 体系治理引用 | BAS-009 git: `adb3e34` 提交组（与 BAS-022 同期），与本主题无独立交叉 commit |
| `RGS-SPEC-CROSS-005` subject 命名规范 | 在 `shared_platform::subject` 注释中提及；本身文档路径未实证（**由 `RGS-DTL-022-ADD1` 互引时补强**） |
| ARC-050 编号 | 本主题新增 ARC 编号；尚无 RGS 既有 ARC 编号系统实证（**待 Ulysses 拍板**） |

---

# 附录 A：与 DTL-022-ADD1 关系

本文档 = 逻辑设计；DTL-022-ADD1 = 物理/实现级设计（crate 目录树 / 6 模块 pub fn 签名 / 错误枚举 / env 读取伪代码 / 与 `crates/rgs-overflow-alert/` 实际代码对账表）。

---

> **本 addendum 与 RGS-BAS-022 v0.2 共同构成"弹性容量规划"完整基本设计集。** 详细设计由 `RGS-DTL-022-ADD1` 展开。

**文档结束。审批者：架构师(Mavis 接手 agent per DEC-008)**
