# 需求定义书（要件定義書 / Requirements Definition Document）

**超限排队与客服邮箱告警 — 弹性容量规划（REQ-025）补强**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-025-ADD2 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-025 弹性容量规划与超大规模并发架构（v0.1） |
| 增补类别 | 新增 FR-OFLOW-001~010 详细需求 + ARC-050 实施细则 |
| 制定日 | 2026-08-27 |
| 制定者 | 架构师(Mavis 接手 agent per DEC-008) |
| 审批者 | 架构师(Mavis 接手 agent per DEC-008) |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

> 审批者代签依据：per 2026-08-26 08:40 JST Ulysses 规则反转，"审批者 = —" 硬约束已被覆盖，子代理 / Mavis 可在审批栏直接填写真实责任署名。

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 |
|---|---|---|---|---|
| 0.1 | 2026-08-27 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版。补强 RGS-REQ-025 容量规划的"超限行为"：超并发上限时入 NATS JetStream 队列 + 异步向客服邮箱告警（per 2026-08-27 Ulysses 拍板）。新增 FR-OFLOW-001~010 + ARC-050（限流/排队/告警架构）。 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师(Mavis 接手 agent per DEC-008) | 2026-08-27 | — |
| 评审（技术） |  |  | 双阈值语义 vs RGS-BAS-023 已有"请求处理链 Layer" 是否冲突（结论：不冲突；限流是 Layer 之一） |
| 评审（业务/运营） |  |  | SMTP_PASSWORD 缺密码时降级到日志 — 客服告警失效应有 PagerDuty / Slack 兜底（**已知缺口**，见 §11） |
| 审批（负责人） |  |  | 本补强与 ARC-040 容量分级正交；分片路由与超限行为解耦 |

---

## 目录

1. [前言](#1-前言)
2. [术语约定](#2-术语约定)
3. [背景与课题](#3-背景与课题)
4. [业务需求](#4-业务需求)
5. [功能需求：限流 / 排队 / 告警](#5-功能需求限流--排队--告警)
6. [非功能需求](#6-非功能需求)
7. [架构设计方针 ARC-050](#7-架构设计方针-arc-050)
8. [验收标准](#8-验收标准)
9. [关联文档](#9-关联文档)
10. [缺标清单](#10-缺标清单)
11. [风险与未决事项](#11-风险与未决事项)

---

# 1. 前言

## 1.1 目的

RGS-REQ-025 v0.1 已规定"弹性容量规划"（ARC-040）的 T0~T3 容量分级与跨分片能力，但**未规定"超并发上限时的具体行为"**——是直接拒绝（`tonic::Status::ResourceExhausted`）、是异步排队、还是请求慢下来（admission control）？本文档补齐这一空白，明确：

- **超并发上限时**触发**入队 + 告警**而非直接拒绝
- **入队后端**统一为 NATS JetStream（沿用 DEC-011 既有基础设施）
- **告警**走 SMTP 向客服邮箱发邮件，`SMTP_PASSWORD` 缺失时降级为结构化日志
- **域作用范围**严格限定为 4 个业务服务（player / economy / match / social），admin / cluster-ops 排除

> 本文不重新讨论分片路由、跨分片事务、容量分级本身——这些已在 RGS-REQ-025 v0.1 + REQ-025-ADD1 库内水平分片中规定。本补强只引入**"限流 + 排队 + 告警"作为超限行为的兜底链路**。

## 1.2 与 ARC-040 / ARC-041 关系

| 既有架构 | 与本补强关系 |
|---|---|
| ARC-040（弹性容量规划） | 容量分级的目标态；本补强实现其**"超限兜底行为"** —— 当请求量瞬时超过 k8s 单 Pod 理论承载时不丢请求，但排队 + 告警 |
| ARC-041（请求处理链标准化） | 限流中间件是该处理链的一个 Layer（**互引 RGS-BAS-023**），不与既有 Layer 冲突 |

## 1.3 适用范围

| 范畴 | 说明 |
|---|---|
| 适用 | player-service / economy-service / match-service / social-service（4 个业务域） |
| **不适用** | admin-service（COC 控制面，外部请求不直接命中）/ cluster-ops（Active-Active + saga_store 9 表，运维控制面） |
| 阶段 | PH-4 启用（10k CCU 后）—— 当前阶段 hard_cap = 0（不启用） |
| 部署 | k3s 集群内（NATS_URL_IN_CLUSTER）+ 本机端口转发（开发期 NATS_URL_LOCAL） |

---

# 2. 术语约定

| 术语 | 定义 |
|---|---|
| **硬上限（hard cap）** | k8s 单 Pod 理论承载上限，由 `<DOMAIN>_MAX_INFLIGHT` env 配置（0 = 不启用限流） |
| **软阈值（soft cap）** | `ceil(hard × NATS_OVERFLOW_SOFT_RATIO)`，默认 0.8；超此 → 入队而非直拒 |
| **Pass** | 在软阈值内，业务持 `OverflowDecision.guard` 到处理完 drop → 释放 in_flight |
| **Queued** | 超软未超硬 → 入 NATS JS 队列，**消费者持 guard 到处理完 drop**（关键设计） |
| **Rejected** | 超硬 → 拒绝 + 触发告警 |
| **告警去重窗口** | `ALERT_DEDUP_WINDOW_SECS`（默认 60s）；同 `(domain, kind)` key 在窗口内只发 1 次 |
| **告警降级** | `SMTP_PASSWORD` 缺失 → `LogOnlySink` 落 `tracing::warn!`，**不抛错，不阻断入队** |
| **subject 命名** | `rgs.<domain>.overflow.v1`（per RGS-SPEC-CROSS-005 + shared_platform::subject::SubjectBuilder::domain_event） |
| **域类型系统防越界** | `Domain` 枚举无 admin/cluster-ops 变体，编译期拒绝误用 |

---

# 3. 背景与课题

RGS 既有的 5 域业务服务在容量规划上仅回答了"如何分片 / 跨域事务 / 快速扩容"（per RGS-REQ-025 + REQ-025-ADD1），但**未回答"超限了怎么办"**：

| 既有做法 | 缺陷 |
|---|---|
| `tonic::Status::ResourceExhausted` 直接拒绝 | 突发流量时所有超限请求瞬时失败，**用户体验断崖** |
| 单纯 sleep + retry | 客户端负担重；DDos 场景无效 |
| 无告警 | 运维在超限发生**数小时后才从业务投诉发现** |

本补强针对这 3 个缺陷：

1. **不直接拒绝**：超软未超硬入 NATS JS 队列，由消费者异步处理
2. **客户端不需重试**：业务返回 `ResourceExhausted` 时携带 ack token + subject，运维可查 NATS 消费进度
3. **运维即时感知**：超硬触发告警邮件（缺密码降级日志），SMTP_PASSWORD 接入后可秒级到客服邮箱

---

# 4. 业务需求

| ID | 需求 | 优先级 |
|---|---|---|
| BR-OLFW-101 | 超并发上限的请求**不直接拒绝**，优先入队异步处理 | 高 |
| BR-OLFW-102 | 客服邮箱（默认 `hanakagumi@gmail.com`）能在超限时秒级收到告警 | 高 |
| BR-OLFW-103 | SMTP 凭据缺失时**不阻断**业务请求；告警仅落结构化日志，运维可通过日志聚合发现 | 高 |
| BR-OLFW-104 | 5 域业务服务（player / economy / match / social）**统一接入**；admin / cluster-ops 排除 | 高 |
| BR-OLFW-105 | 每域独立 Lead 独立配置（per 2026-08-21 Ulysses 偏好）；`<DOMAIN>_MAX_INFLIGHT` 每域可覆盖 | 中 |
| BR-OLFW-106 | 限流开销 ≤ 10μs / 请求（含 acquire + 释放） | 中 |
| BR-OLFW-107 | 告警去重避免告警风暴 | 中 |

---

# 5. 功能需求：限流 / 排队 / 告警

## 5.1 FR-OFLOW-001：双阈值（软/硬）

- **软阈值** = `ceil(hard × NATS_OVERFLOW_SOFT_RATIO)`，默认 0.8
- **硬上限** = `<DOMAIN>_MAX_INFLIGHT` env 读取，0 = 不启用
- **Pass 判定**：`in_flight + 1 ≤ soft`
- **Queued 判定**：`soft < in_flight + 1 < hard`
- **Rejected 判定**：`in_flight + 1 ≥ hard`

## 5.2 FR-OFLOW-002：NATS JetStream 队列

- 队列后端 = **NATS JetStream**（沿用 DEC-011 既有基础设施，**不**新引入 Redis / Kafka）
- subject 命名 = `rgs.<domain>.overflow.v1`（per RGS-SPEC-CROSS-005 + shared_platform::subject::SubjectBuilder::domain_event）
- 一个 stream `RGS_OVERFLOW` 覆盖 4 域所有 subject filter = `rgs.*.overflow.v1`
- consumer group = `rgs-overflow-workers`
- max_pending = `NATS_OVERFLOW_MAX_PENDING`（默认 10000），超此返回 `QueueError::QueueFull`
- **多副本并发启动的 stream create 竞态**：依赖 `get_or_create_stream` 的 idempotency（async-nats 0.42 已实现）

## 5.3 FR-OFLOW-003：邮件告警

- 发件：SMTP（`SMTP_HOST` / `SMTP_PORT` / `SMTP_USER` / `SMTP_PASSWORD` / `SMTP_FROM_NAME` / `SMTP_TIMEOUT_MS`）
- 收件：`SUPPORT_EMAIL`（**默认 `hanakagumi@gmail.com`**）
- 真实密码走 k8s Secret（**不**入 .env 提交历史，per .env.example §8 注释）
- **缺密码时降级**：`LogOnlySink` 落 `tracing::warn!`，**不抛错，不阻断入队**（这是 SMTP_PASSWORD 当前为空时的合法降级路径）

## 5.4 FR-OFLOW-004：告警去重

- `AlertDeduplicator` 用 `(domain, kind)` 做 key，`ALERT_DEDUP_WINDOW_SECS` 窗口（默认 60s）
- 窗口内同 key 只发 1 次
- 不同 `(domain, kind)` 独立计数
- 告警级别：`HardCapReached`（硬上限满）/ `SoftCapSurge`（软阈值首超）/ `QueueFull`（队列满）/ `SinkFailure`（SMTP 失败）

## 5.5 FR-OFLOW-005：域类型系统防越界

- `Domain` 枚举（`Player` / `Economy` / `Match` / `Social`）**不含** `Admin` / `ClusterOps`
- 编译期拒绝在 admin-service / cluster-ops 中使用 rgs-overflow-alert
- 测试：`from_str_rejects_admin_and_cluster_ops`（已通过）

## 5.6 FR-OFLOW-006：Queued permit 保留语义（**关键设计**）

- `OverflowDecision.guard: Option<InFlightGuard>`：
  - `Pass` → `Some(permit)`，业务持到处理完 drop
  - `Queued` → `Some(permit)`，**消费者持到处理完 drop**（不是业务 RPC 路径 drop —— 因为 RPC 立即返回 ResourceExhausted）
  - `Rejected` → `None`
- **为什么 Queued 也保留 permit**：若立即 drop，则 in_flight 减回，下一个请求永远能 Pass/Queued，**硬上限失去意义**。保留到消费者处理完才能让 in_flight 真实反映"未完成请求数"

## 5.7 FR-OFLOW-007：限流并发安全

- `try_acquire` 用手写 `compare_exchange` CAS loop，**不**用 `fetch_update` 乐观重试
- 乐观重试在 1000 并发下会让所有 task 都 +1 成功（in_flight 突破 hard）—— 此为已修复的 race bug
- 修复后行为：CAS 失败 → 直接 Rejected（**不**重试，避免乐观重试破坏限流）
- 测试：`integration_1000_concurrent_pass_queue_reject_distribution` 验证

## 5.8 FR-OFLOW-008：5 域独立配置

- 每域独立 Lead（per 2026-08-21 Ulysses 偏好）—— `Domain` 枚举每变体对应一个独立硬上限 env
- `PLAYER_MAX_INFLIGHT` / `ECONOMY_MAX_INFLIGHT` / `MATCH_MAX_INFLIGHT` / `SOCIAL_MAX_INFLIGHT` 独立配置
- 不允许"统一开关"（兼任方案已被拒绝）

## 5.9 FR-OFLOW-009：SMTP_PASSWORD 走 k8s Secret

- `.env` / `.env.example` 中 `SMTP_PASSWORD=` 留空（占位）
- 真实密码通过 k8s Secret 注入到 Pod 环境变量（**不**入 git 提交历史）
- helm `secret.yaml` 模板用占位 key `rgs-smtp-password`，真实值由运维注入

## 5.10 FR-OFLOW-010：客服邮箱默认 hanakagumi@gmail.com

- `SUPPORT_EMAIL` env 默认值 = `hanakagumi@gmail.com`
- 可由 `SUPPORT_EMAIL` 覆盖（运营可换企业邮箱）
- 硬编码于 `crates/rgs-overflow-alert/src/config.rs` 的 `DEFAULT_SUPPORT_EMAIL` 常量

---

# 6. 非功能需求

| ID | 类别 | 目标值 |
|---|---|---|
| NFR-OLFW-101 | 性能 | 限流 acquire + release 总开销 ≤ 10μs / 请求（p99） |
| NFR-OLFW-102 | 可用性 | SMTP 失败不阻断业务；告警降级路径延迟 ≤ 1ms |
| NFR-OLFW-103 | 可观测性 | `tracing::warn!` 含 domain / in_flight / hard / soft / queue_pending / pod / service / reject_count_5min / first_at / last_at |
| NFR-OLFW-104 | 安全 | SMTP_PASSWORD 走 k8s Secret，**不**入 .env 提交历史；LogOnlySink 输出的告警不含密码 |
| NFR-OLFW-105 | 部署 | 不引入新运维面：NATS / SMTP 均沿用 RGS 既有依赖 |
| NFR-OLFW-106 | 兼容性 | admin-service / cluster-ops 0 改动（git diff 验证） |
| NFR-OLFW-107 | 持久化 | NATS JS 自带持久化（per DEC-011），Pod 重启不丢 |

---

# 7. 架构设计方针 ARC-050

## 7.1 概述

ARC-050（超限排队与告警）= **限流 + 排队 + 告警**三位一体，作为 RGS 既有的请求处理链（per ARC-041 / RGS-BAS-023）的一个 Layer 接入。

```
业务 RPC handler
    │
    ▼
┌─────────────────────────┐
│ OverflowGuard::check    │  ← ARC-050 入口
└────┬────────┬────────┬───┘
     │        │        │
   Pass    Queued   Rejected
     │        │        │
     ▼        ▼        ▼
  业务处理  NATS JS   拒绝 + 告警
            入队
```

## 7.2 与 ARC-040 / ARC-041 关系

| 既有 ARC | ARC-050 关系 |
|---|---|
| ARC-040 容量分级 | ARC-050 是其**超限兜底行为**——容量分级决定"正常情况下的分布"，ARC-050 决定"超限了怎么办" |
| ARC-041 请求处理链 | ARC-050 限流是处理链的一个 Layer（**互引 RGS-BAS-023**） |

## 7.3 域作用域

ARC-050 仅作用于 4 个业务域（player / economy / match / social）。admin / cluster-ops 排除 —— 编译期通过 `Domain` 枚举类型系统防越界。

---

# 8. 验收标准

- [ ] `cargo test -p rgs-overflow-alert` 35/35 全过（30 unit + 5 integration）
- [ ] 4 域服务（player / economy / match / social）`cargo build` 通过
- [ ] admin-service / cluster-ops 0 改动（`git diff` 验证）
- [ ] workspace 依赖含 `async-nats = 0.42` + `lettre = 0.11 (tokio1-rustls-tls)`
- [ ] `.env.example` 含 SUPPORT_EMAIL / SMTP_* / NATS_OVERFLOW_* / 4 域 MAX_INFLIGHT（per §5.10）
- [ ] helm 4 域 values 各加 `overflow` 段；configmap 加 SUPPORT_EMAIL 等 4 个 env；secret 模板加 SMTP_PASSWORD 引用
- [ ] `crates/rgs-overflow-alert/README.md` + `RISKS.md` 完成
- [ ] 互引文档 3 份 addendum 全部 git 实证
- [ ] 文档代签（per 2026-08-26 08:40 JST 规则反转）
- [ ] **无回溯叙事**

---

# 9. 关联文档

## 9.1 父文档 / 母本

- `RGS-REQ-025` 弹性容量规划与超大规模并发架构（v0.1，2026-08-16）— git: `adb3e34` (feat docs finalize pre-implementation specification baseline)
- `RGS-REQ-025-ADD1` 库内水平分片（v0.2，2026-08-20）— git: 与 REQ-025 同提交组

## 9.2 互引（**全引** per Ulysses 拍板）

| 文档 | 关系 | 引用证据 |
|---|---|---|
| `RGS-REQ-026` | 请求处理链管道需求 | git: `17b7522` (新增弹性容量规划(5万→千万并发)与请求处理链标准化需求/基本设计) |
| `RGS-REQ-013` | 治理框架需求 | git: (单独提交，与本主题无直接 commit 关联) — **缺标**，见 §10 |

## 9.3 下游（详细 / 基本设计）

- `RGS-BAS-022-ADD1` 超限排队与客服邮箱告警 基本设计书（待写）
- `RGS-DTL-022-ADD1` 超限排队与客服邮箱告警 详细设计书（待写）

## 9.4 与超限中间件实现的对账

- 实际代码：`crates/rgs-overflow-alert/src/{domain,config,limiter,queue,alert,guard,lib}.rs`
- git: commit `d7a139f` on `main` (feat(rgs-overflow-alert): 5 域业务服务超限排队 + 邮件告警中间件)

---

# 10. 缺标清单（per 2026-08-26 DDD Review 必查）

> **缺标比错标安全**：以下引用在 git history 中**未找到独立 commit 关联**，仅在母本中提及。RGS 文档治理规则要求显式列出，不在文档中编造出处。

| 引用 | 缺标原因 |
|---|---|
| `RGS-REQ-013` 治理框架 | 与本主题无直接 git commit 关联；其与 ARC-040 / ARC-050 的关系在母本 RGS-REQ-013 中描述，但本 addendum 引用未实证 |
| `RGS-BAS-023` 引用 | 仅在 RGS-REQ-026 父需求中提及；BAS-023 本身的 git log 未与本主题交叉（**由 RGS-BAS-022-ADD1 / DTL-022-ADD1 互引时补强**） |
| `RGS-BAS-003` 引用 | 在 02-运维安全与网络目录，git: `adb3e34` 之前版本提交；与本主题无直接交叉（**由 RGS-BAS-022-ADD1 互引时补强**） |
| DEC-011 决议号 | 文档无 DEC-011 编号 git 引用；NATS JetStream 选型在 RGS-DTL-100 §5 描述（**由 RGS-BAS-022-ADD1 互引时补强**） |
| Arc-050 编号 | 本文档新增；尚无 RGS 既有 ARC 编号系统实证（**待 Ulysses 拍板**） |

---

# 11. 风险与未决事项

## 11.1 已知风险

1. **SMTP_PASSWORD 当前为空** → 告警只落 `tracing::warn!`，运维不会收到邮件。**生产前必须补真实密码到 k8s Secret**。
2. **5 域 RPC 风格差异**（unary vs streaming）—— Queued 时业务返回 `ResourceExhausted` 还是 await permit，**未在本 addendum 写明统一约定**（由 RGS-BAS-022-ADD1 / DTL-022-ADD1 互引时细化）。
3. **NATS stream create 多副本并发启动竞态** —— 当前依赖 `get_or_create_stream` 的 idempotency，**未实测**多副本同时启动。
4. **每域 hard_cap 合理值需压测给** → **本次全部设 0（不启用）**，等 Ulysses 拍板。
5. **`InFlightGuard` 必须由消费者持到处理完 drop** —— 如果消费者代码 bug 不 drop，in_flight 永远不释放 → 硬上限假死。需在 4 域挂点 DTL addendum 强化约定。
6. **告警去重 60s 窗口内同 key 只发 1 次** —— **重大事件可能被吞**，建议生产前加 escalation（首次 → 持续超 → page）。

## 11.2 未决事项（待 Ulysses 拍板）

- 每域 hard_cap 推荐值（需压测）
- 告警去重窗口是否分级（普通 60s vs P0 事件 5s）
- Queued 时业务是否携带 ack token + subject 给客户端（让客户端可查 NATS 消费进度）
- ARC-050 编号是否入 ARC 体系

---

# 附录 A：env 配置表

| env key | 默认 | 说明 |
|---|---|---|
| `SUPPORT_EMAIL` | `hanakagumi@gmail.com` | 告警收件人 |
| `SMTP_HOST` | `smtp.gmail.com` | SMTP 服务器 |
| `SMTP_PORT` | `587` | SMTP 端口（STARTTLS） |
| `SMTP_USER` | `hanakagumi@gmail.com` | SMTP 登录用户 |
| `SMTP_PASSWORD` | （空） | SMTP 密码；走 k8s Secret |
| `SMTP_FROM_NAME` | `RGS-Ops-Alert` | 发件人显示名 |
| `SMTP_TIMEOUT_MS` | `3000` | SMTP 发送超时（ms） |
| `NATS_URL_IN_CLUSTER` | `nats://nats.rust-game-server.svc.cluster.local:4222` | 集群内 NATS（沿用 shared-platform 约定） |
| `NATS_URL_LOCAL` | `nats://127.0.0.1:14222` | 本机端口转发 |
| `NATS_OVERFLOW_SOFT_RATIO` | `0.8` | 软阈值比例（0.0-1.0） |
| `NATS_OVERFLOW_MAX_PENDING` | `10000` | 队列最大 pending |
| `NATS_OVERFLOW_STREAM` | `RGS_OVERFLOW` | NATS stream 名 |
| `NATS_OVERFLOW_CONSUMER_GROUP` | `rgs-overflow-workers` | 消费者组 |
| `ALERT_DEDUP_WINDOW_SECS` | `60` | 告警去重窗口（秒） |
| `PLAYER_MAX_INFLIGHT` | `0` | player 域硬上限（0 = 不启用） |
| `ECONOMY_MAX_INFLIGHT` | `0` | economy 域硬上限 |
| `MATCH_MAX_INFLIGHT` | `0` | match 域硬上限 |
| `SOCIAL_MAX_INFLIGHT` | `0` | social 域硬上限 |

---

> **本 addendum 与 RGS-REQ-025 v0.1 + REQ-025-ADD1 共同构成"弹性容量规划"完整需求集。** 详细设计与基本设计分别由 `RGS-DTL-022-ADD1` 与 `RGS-BAS-022-ADD1` 展开。

**文档结束。审批者：架构师(Mavis 接手 agent per DEC-008)**
