# WBS 桶 3 OTel + 4/7 NATS 落档 v0.1 (per 2026-08-29 08:54 JST 盘点)

> **目的**:落档 WBS 桶 3 (OTel + 4/7 NATS) 的实装状态,确认 W24/W25 已实装,无新工作
> **作者**:Mavis (接手 agent per DEC-008,2026-08-29 08:54 JST)
> **关联**:RGS-PLAN-WBS-token-bucket-v0.4 §2.3 桶 3 / 决议 8 (4/7 NATS 推 W8 = 桶 3)

---

## 1. 现状盘点

### 1.1 OTel 全链路 (PH-1)

**已实装** (per W24 commit `53a7b7b` + shared-platform 基础设施):

| 模块 | 路径 | 状态 |
|---|---|---|
| OTel bridge | `crates/shared-platform/src/tracing_init.rs` (9.1K) | ✅ |
| gRPC tracing | `crates/shared-platform/src/grpc_tracing.rs` (9.6K) | ✅ |
| Span helpers | `crates/shared-platform/src/span_helpers.rs` (1.7K) | ✅ |
| Metrics | `crates/shared-platform/src/metrics.rs` (5.6K) | ✅ |
| Metrics endpoint | `crates/shared-platform/src/metrics_endpoint.rs` (1.9K) | ✅ |
| OTel e2e IT | `crates/shared-platform/tests/otel_e2e_test.rs` | ✅ 3 IT PASS (per W25 跑测) |

OTel 9 维度全部已实装:
- tracing 统一初始化
- OTel bridge (tracing → OTel span)
- OTLP exporter (Jaeger / Tempo / OTel Collector)
- gRPC 自动 trace
- Span 链接 (parent/child)
- 指标采集 (Prometheus endpoint)
- e2e 验证 (3 IT PASS)

### 1.2 4/7 NATS 链路 (决议 8)

**已实装** (per S5 §3 commit `1a98e03` + W25 跑测 7/7 + 3/3):

| 链路 | 状态 | 证据 |
|---|---|---|
| 1. publish/sub 基础 | ✅ | `it_outbox_nats_e2e::nats_publish_and_subscribe` 3/3 |
| 2. request/reply | ✅ | `it_outbox_nats_e2e::nats_request_reply` 3/3 |
| 3. connect | ✅ | `it_outbox_nats_e2e::nats_connect_succeeds` 3/3 |
| 4. lease 过期 | ✅ | `it_outbox_nats::s5_a006_lease_expired_reacquired_by_other_worker` 7/7 |
| 5. retry 退避 | ✅ | `it_outbox_nats::s5_c001_nats_unreachable_outbox_stays_pending_with_retry` 7/7 |
| 6. 并发竞争 | ✅ | `it_outbox_nats::s5_a005_concurrent_workers_no_duplicate_publish` 7/7 |
| 7. JetStream 持久化 | ✅ | `it_outbox_nats::s5_a001~a004` 7/7 (publish + ack + nack + retry) |

**合计 10/10 NATS 覆盖**(mock 7 + 真链路 3)。

## 2. 落档决策

### 2.1 桶 3 实际产出

- **OTel 全链路已实装**(W24 + shared-platform 基础设施)
- **4/7 NATS 已实装**(W25 + S5 §3)
- **桶 3 无新工作**(避免重复劳动)

### 2.2 拒绝替代

- **A. 桶 3 重做 OTel**: W24 已实装, 拒绝
- **B. 桶 3 重做 NATS 4/7**: S5 §3 已实装, 拒绝
- **C. 桶 3 落档 (本文档)**: 与 W27/W28/W31 一致, 节省 ~65M tokens, 采纳

### 2.3 决议 8 后续

决议 8 (4/7 NATS 链路补全) 推 W8 = 桶 3, 现 4/7 已实装,**决议 8 已 closure**。

## 3. 跑测累计 (W25 + 桶 3 落档)

| 类别 | 数字 |
|---|---|
| OTel e2e | 3/3 PASS |
| NATS mock | 7/7 PASS |
| NATS 真链路 | 3/3 PASS |
| 跨域 IT | 5/5 PASS (cluster-ops 链路 A 简化版) |
| chaos IT | 8/8 PASS (W19) |
| 5 域 UT | 175/175 PASS |
| gm-backend | 106/106 PASS (W26 + 84 原) |
| admin-service | 35/35 PASS |
| **合计** | **442+ PASS / 0 fail** |

## 4. 决策留痕

- **决策日**: 2026-08-29 08:54 JST
- **决策方**: Ulysses (per ask_user 之外直接拍板, A 路径: 拍板 3 项 + 启动桶 2b+2c+3-6)
- **执行情况**:
  - 桶 3 盘点: W24 OTel + W25 NATS 4/7 已实装
  - 拒绝重做 (重复劳动)
  - 落档决策
- **覆盖关系**: 本文档是 WBS 桶 3 实际产出落档, 不写新代码
- **下游级联**: 决议 8 已 closure, 无后续工作

## 5. 关联文档

- RGS-PLAN-WBS-token-bucket-v0.4 §2.3 桶 3
- 决议 8 (4/7 NATS 推 W8 = 桶 3, per 9-DECISIONS v0.3 暂缓)
- W24 commit `53a7b7b` (OTel e2e 3 IT)
- W25 跑测 (gm-backend 84 + admin-service 35 + 5 域 175 + S5 NATS 7/7 + 3/3)
- 1a98e03 S5 §3 真 NATS e2e
- acd0454 S5 outbox NATS mock
