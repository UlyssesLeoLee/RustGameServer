# RGS-GOBS-100 Saga 可观测性设计

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-GOBS-100 |
| 版本 | 0.1（初版） |
| 制定日 | 2026-08-21 |
| 最终更新日 | 2026-08-21 |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-REQ-100 / RGS-BAS-100 / RGS-DTL-100~102 / RGS-OPS-100（同侪 K3s 部署）/ RGS-SEC-100（同侪 安全审计） |
| 配套标准 | IPA 共通フレーム 2013 + 150 工程日本 SI 业界标准；V 模型映射：UT ↔ DTL |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。OpenTelemetry 全链路追踪（Mermaid）+ 5 类追踪 ID 传播 + Metrics 17 项（4 类）+ Loki 日志字段表 + Tempo trace 存储 + Admin Saga Console 监控。 |

---

## 0. 文档目的

定义 Saga 系统的**可观测性**：

1. OpenTelemetry 全链路追踪（Admin Click → Admin Gateway → Saga → Economy → NATS → Inventory → Compensation）
2. 5 类追踪 ID（trace_id / saga_id / step_id / command_id / event_id）+ 业务上下文（player_id / match_id）
3. Metrics 17 项（通用 + Saga + 业务）
4. Loki 日志字段（统一日志结构）
5. Tempo Trace 存储
6. Admin Saga Console（监控 / Manual Intervention）

---

## 1. 全链路追踪（OpenTelemetry）

```mermaid
sequenceDiagram
    autonumber
    actor GM as GM Operator
    participant Console as Admin UI<br/>(Saga Console)
    participant AG as Admin Gateway
    participant SR as Saga Runtime
    participant ES as Economy Service
    participant MB as NATS JetStream
    participant IS as Inventory Service
    participant DB as PostgreSQL
    participant OTel as OTel Collector
    participant Tempo as Tempo
    participant Loki as Loki
    participant Prom as Prometheus
    participant Graf as Grafana

    Note over GM,Graf: trace_id=ABC123 (端到端)<br/>saga_id=S-001 (业务事务)<br/>step_id=ST-005 (Saga 步骤)<br/>command_id=C-005 (业务指令)<br/>event_id=E-005 (事件唯一)

    GM->>Console: 点击 SendCompensationPack<br/>(reason=玩家反馈, 2FA 已通过)
    Console->>AG: SendBusinessCommand<br/>HTTP POST + X-Trace-Id=ABC123
    activate AG
    AG->>SR: StartSaga (PurchaseFlow v2)<br/>X-Saga-Id=S-001, X-Step-Index=0
    activate SR
    SR->>DB: INSERT saga_instance<br/>(saga_id=S-001, fence_token=42)
    SR->>DB: INSERT saga_event (SagaStarted)
    SR-->>AG: saga_id=S-001
    AG-->>Console: 200 OK (saga_id=S-001)
    Console->>Console: L0 UI 反馈 (Saga Running)

    Note over SR,ES: Step 1: grant-currency
    SR->>ES: ReserveCurrency<br/>X-Command-Id=C-001, X-Idempotency-Key=S-001:C-1
    activate ES
    ES->>DB: BEGIN; UPDATE balances
    ES->>DB: INSERT outbox (CurrencyReserved, event_id=E-001)
    ES->>DB: INSERT inbox (idempotency_key=S-001:C-1)
    ES->>DB: COMMIT
    ES-->>SR: OK
    deactivate ES
    SR->>DB: UPDATE saga_step (step=1, state=SUCCESS)

    Note over SR,IS: Step 2: grant-items
    SR->>IS: GrantItem<br/>X-Command-Id=C-002, X-Idempotency-Key=S-001:C-2
    activate IS
    IS->>DB: BEGIN; INSERT items
    IS->>DB: INSERT outbox (ItemGranted, event_id=E-002)
    IS->>DB: INSERT inbox (idempotency_key=S-001:C-2)
    IS->>DB: COMMIT
    IS-->>SR: OK
    deactivate IS
    SR->>DB: UPDATE saga_step (step=2, state=SUCCESS)

    SR->>DB: UPDATE saga_instance<br/>SET state=COMPLETED
    SR->>DB: INSERT saga_event (SagaCompleted)
    deactivate SR
    deactivate AG

    Note over OTel,Graf: 所有 span 推到 OTel Collector
    AG->>OTel: export span (HTTP /game-gateway → saga-runtime)
    SR->>OTel: export span (saga execute step 1, step 2)
    ES->>OTel: export span (reserve_currency db.begin/commit)
    IS->>OTel: export span (grant_item db.begin/commit)
    DB->>OTel: pg.trace (sql trace)

    OTel->>Tempo: store trace (by trace_id)
    OTel->>Loki: store logs (by trace_id + saga_id)
    OTel->>Prom: scrape metrics (saga_started_total++)

    GM->>Graf: Saga Console 查询 S-001
    Graf->>Tempo: get trace by saga_id=S-001
    Tempo-->>Graf: trace spans (5 services, 7 spans)
    Graf-->>GM: 完整链路 + 时序
```

**追踪 ID 传播矩阵**：

| ID | 生命周期 | 传播方式 | 存储位置 |
|---|---|---|---|
| `trace_id` | 端到端（GM click → DB commit）| W3C Trace Context (HTTP / gRPC metadata) | OTel → Tempo |
| `saga_id` | Saga 生命周期（start → completed/failed）| gRPC metadata + DB column + log field | saga_instance / saga_event |
| `step_id` | Saga 单步（start → success/fail）| DB column + log field | saga_step / saga_event |
| `command_id` | 单个 Command（send → ACK）| gRPC metadata + idempotency_key | saga_command / inbox |
| `event_id` | 单个 Event（emit → publish）| Outbox event_id + log field | outbox / inbox |
| `player_id` | 业务操作生命周期 | payload + log field | 业务表 + log |
| `match_id` | 比赛相关操作 | payload + log field | match_db + log |
| `operator_id` | GM 操作 | audit log + log field | saga_audit |
| `correlation_id` | 跨服务请求关联 | HTTP / gRPC header | saga_instance |

---

## 2. 5 类追踪 ID 规范

### 2.1 trace_id（W3C Trace Context）

```rust
// 32 hex chars (16 bytes), 128-bit
// 例: "4bf92f3577b34da6a3ce929d0e0e4736"
pub struct TraceId(pub [u8; 16]);

// 通过 OpenTelemetry propagation 传播
// HTTP: traceparent header
// gRPC: grpc-trace-bin header
```

### 2.2 saga_id（业务事务 ID）

```rust
// UUID v7 (时间排序)
pub type SagaId = uuid::Uuid;
// 例: "0190a3b4-7c8d-7891-bcd0-1234567890ab"

// 传播方式:
// - HTTP header: X-Saga-Id
// - gRPC metadata: x-saga-id
// - DB column: saga_instance.saga_id
// - Log field: saga_id=0190a3b4-...
```

### 2.3 step_id（Saga 步骤 ID）

```rust
// UUID v4
pub type StepId = uuid::Uuid;
// 例: "f47ac10b-58cc-4372-a567-0e02b2c3d479"
```

### 2.4 command_id（业务指令 ID）

```rust
// UUID v7
pub type CommandId = uuid::Uuid;
// 例: "0190a3b4-7d00-7891-bcd0-9876543210cd"
```

### 2.5 event_id（事件唯一 ID）

```rust
// UUID v7
pub type EventId = uuid::Uuid;
// 例: "0190a3b4-7d20-7891-bcd0-aabbccddeeff"
```

---

## 3. 业务上下文 ID 规范

| 字段 | 类型 | 长度 | 格式 | 例 |
|---|---|---|---|---|
| `player_id` | UUID v7 | 36 | 8-4-4-4-12 | `0190a3b4-...` |
| `account_id` | UUID v7 | 36 | 同上 | `0190a3b4-...` |
| `character_id` | i64 | 19 | snowflake | `1771234567890123456` |
| `match_id` | UUID v7 | 36 | 同上 | `0190a3b4-...` |
| `guild_id` | i64 | 19 | snowflake | `1771234567890123457` |
| `item_id` | i64 | 19 | snowflake | `1771234567890123458` |
| `currency_type` | VARCHAR | 16 | enum | `GOLD / DIAMOND / TOKEN` |
| `operator_id` | VARCHAR | 64 | user@domain | `ulysses@local` |
| `correlation_id` | UUID v7 | 36 | 同上 | `0190a3b4-...` |

---

## 4. Metrics 17 项（4 类）

### 4.1 Saga 通用 Metrics

```promql
# 1. saga_started_total
saga_started_total{saga_type="PurchaseFlow",version="1"} 1234

# 2. saga_completed_total
saga_completed_total{saga_type="PurchaseFlow",version="1"} 1200

# 3. saga_failed_total
saga_failed_total{saga_type="PurchaseFlow",version="1",reason="insufficient_funds"} 30
saga_failed_total{saga_type="PurchaseFlow",version="1",reason="inventory_full"} 4

# 4. saga_compensation_total
saga_compensation_total{saga_type="PurchaseFlow",version="1"} 8

# 5. saga_duration_seconds
saga_duration_seconds{saga_type="PurchaseFlow",version="1",quantile="0.5"} 0.150
saga_duration_seconds{saga_type="PurchaseFlow",version="1",quantile="0.95"} 0.380
saga_duration_seconds{saga_type="PurchaseFlow",version="1",quantile="0.99"} 0.980

# 6. saga_step_duration_seconds
saga_step_duration_seconds{step="reserve-currency",quantile="0.95"} 0.045
saga_step_duration_seconds{step="grant-item",quantile="0.95"} 0.082

# 7. saga_retry_total
saga_retry_total{saga_type="PurchaseFlow",step="reserve-currency"} 12

# 8. saga_in_flight
saga_in_flight{saga_type="PurchaseFlow"} 5

# 9. saga_manual_intervention_total
saga_manual_intervention_total{saga_type="RewardFlow",reason="inventory_compensation_failed"} 2
```

### 4.2 基础设施 Metrics

```promql
# 10. outbox_backlog
outbox_backlog{aggregate_type="inventory",status="PENDING"} 12

# 11. outbox_publish_lag_seconds
outbox_publish_lag_seconds{aggregate_type="inventory",quantile="0.95"} 0.5

# 12. inbox_duplicate_total
inbox_duplicate_total{consumer="saga-runtime",event_type="ItemGranted"} 23

# 13. nats_published_total
nats_published_total{subject="SAGA.*"} 5678

# 14. nats_consumed_total
nats_consumed_total{subject="SAGA.*",consumer="saga-runtime"} 5670
```

### 4.3 业务 Metrics

```promql
# 15. purchase_saga_failures
purchase_saga_failures{step="grant-item",reason="inventory_full"} 4

# 16. reward_pending
reward_pending{match_id="..."} 3

# 17. inventory_compensation_failures
inventory_compensation_failures{item_state="COMMITTED",reason="already_traded"} 1

# 18. currency_refund_failures
currency_refund_failures{account_state="FROZEN"} 2
```

### 4.4 资源 Metrics（K8s 自动）

```promql
# saga_runtime_pod_cpu_usage
container_cpu_usage_seconds_total{pod="saga-runtime-xxx",container="saga-runtime"} 0.25

# saga_runtime_pod_memory_usage
container_memory_usage_bytes{pod="saga-runtime-xxx",container="saga-runtime"} 524288000
```

---

## 5. 日志规范（Loki）

### 5.1 强制日志字段（每条日志必须带）

```json
{
  "ts": "2026-08-21T12:34:56.789Z",
  "level": "INFO",
  "service": "saga-runtime",
  "pod": "saga-runtime-7c4f9b-x2k8d",
  "namespace": "rust-game-server",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "saga_id": "0190a3b4-7c8d-7891-bcd0-1234567890ab",
  "step_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "command_id": "0190a3b4-7d00-7891-bcd0-9876543210cd",
  "event_id": "0190a3b4-7d20-7891-bcd0-aabbccddeeff",
  "player_id": "0190a3b4-...-...-...-...",
  "operator_id": "ulysses@local",
  "msg": "Step 5 grant-item succeeded",
  "duration_ms": 82,
  "result": "OK"
}
```

### 5.2 日志级别

| Level | 使用场景 | 采样 |
|---|---|---|
| TRACE | 详细调试 | 0% (dev only) |
| DEBUG | 调试信息 | 10% (dev) / 0% (prod) |
| INFO | 正常事件（SagaStarted / StepSucceeded / SagaCompleted）| 100% |
| WARN | 重试 / 降级 / 慢查询 | 100% |
| ERROR | 失败 / 异常 | 100% |
| FATAL | Saga Runtime 不可用 | 100% |

### 5.3 不得输出（per BR-108 / spec 51）

```rust
// ❌ 禁止
log::error!("something failed");
log::warn!("reservation failed");

// ✅ 强制
log::info!(
    saga_id = %saga_id,
    step_id = %step_id,
    player_id = %player_id,
    duration_ms = 82,
    "Step grant-item failed, retrying"
);
```

---

## 6. Tempo Trace 存储

### 6.1 Span 结构

每个 Saga 步骤的 span：

```
Span: saga_runtime.execute_step
  Parent: saga_runtime.execute_saga
  Attributes:
    - saga.id: 0190a3b4-...
    - saga.type: PurchaseFlow
    - saga.version: 1
    - saga.step_index: 5
    - saga.step_name: grant-item
    - saga.participant: inventory-service
    - saga.action: GrantItem
    - player.id: 0190a3b4-...
    - command.id: 0190a3b4-...
    - command.idempotency_key: S-001:C-5
  Events:
    - command.published
    - command.acked
    - step.success
```

### 6.2 Span 链接

跨服务的 span 用 `links` 关联（per OTel spec）：

```
Span: economy_service.reserve_currency
  Links:
    - trace_id: ABC123
      span_id: ST-005
      attributes:
        saga.id: S-001
        saga.step_index: 1
```

---

## 7. Admin Saga Console

后台增加 `Saga Monitor App`（per FR-110）：

### 7.1 列表视图

```
┌──────────────────────────────────────────────────────────────┐
│ Saga Monitor                                                  │
├──────────────────────────────────────────────────────────────┤
│ Filters: [All] [Running 5] [Failed 2] [Compensating 1] [...]  │
│                                                              │
│ Saga ID       Type         State        Started    Duration │
│ S-001         PurchaseFlow COMPLETED    2min ago   180ms    │
│ S-002         RewardFlow   COMPENSATING 1min ago   2.1s     │
│ S-003         CreateChar   FAILED       30s ago    -        │
│ S-004         PurchaseFlow RUNNING      10s ago    -        │
└──────────────────────────────────────────────────────────────┘
```

### 7.2 单 Saga 详情

```
┌──────────────────────────────────────────────────────────────┐
│ Saga S-002 RewardFlow (Compensating)                          │
├──────────────────────────────────────────────────────────────┤
│ Definition: RewardFlow v1                                     │
│ Initiator: match-service (match_id=0190a3b4-...)             │
│ Started: 2026-08-21 12:30:00                                  │
│ Current Step: 2/4 (update-rank) - FAILED                     │
│ Fence Token: 42                                               │
│ Owner Pod: saga-runtime-7c4f9b-x2k8d                          │
│                                                              │
│ Timeline:                                                    │
│   [✓] 12:30:00 SagaStarted                                   │
│   [✓] 12:30:00.150 Step 1 grant-currency SUCCESS (45ms)     │
│   [✗] 12:30:00.250 Step 2 update-rank FAILED (timeout 30s)  │
│         Error: rank-service connection refused              │
│   [↻] 12:30:30 RETRY step 2 (backoff 1s)                    │
│   [✗] 12:30:32 RETRY FAILED                                  │
│   [↻] 12:30:34 RETRY step 2 (backoff 2s)                    │
│   [✗] 12:30:36 RETRY FAILED                                  │
│   [⏸] 12:30:40 COMPENSATING STARTED                         │
│   [✓] 12:30:40.100 Comp 1 refund-currency SUCCESS           │
│   [⏳] Comp 2 mark-match-pending-reward IN PROGRESS         │
│                                                              │
│ Actions:                                                     │
│   [Pause] [Resume] [Retry Failed Step]                       │
│   [Manual Compensate] [Cancel] [Export Audit]                │
│                                                              │
│ Trace: view in Tempo (link)                                   │
│ Logs: filter by saga_id=S-002 (link)                        │
└──────────────────────────────────────────────────────────────┘
```

### 7.3 关键查询

| 查询 | 用途 |
|---|---|
| `GET /api/saga/{saga_id}` | 单 saga 详情 + steps + events |
| `GET /api/saga?state=RUNNING&limit=100` | 列表 |
| `POST /api/saga/{saga_id}/pause` | 暂停（GM 决策前）|
| `POST /api/saga/{saga_id}/resume` | 恢复 |
| `POST /api/saga/{saga_id}/retry-step` | 重试失败 step（需 2FA）|
| `POST /api/saga/{saga_id}/manual-compensate` | 手工补偿（需 2FA）|
| `POST /api/saga/{saga_id}/cancel` | 取消（需 2FA + audit）|
| `GET /api/saga/{saga_id}/audit` | 审计日志（不可篡改）|

### 7.4 Grafana 仪表板

| 仪表板 | 关键面板 |
|---|---|
| Saga Overview | saga_started/completed/failed/compensation_total / in_flight |
| Saga Latency | p50/p95/p99 step duration + saga duration |
| Saga Errors | by reason / by step / by saga_type |
| Outbox Health | backlog / publish_lag / failed publish |
| Inbox Health | duplicate rate / pending / processing rate |
| NATS Health | published/consumed rate / lag / subject stats |
| Business Metrics | purchase_saga_failures / reward_pending / inventory_compensation_failures |

---

## 8. AlertManager 告警规则

```yaml
# alerts.yaml
groups:
  - name: saga_critical
    rules:
      - alert: SagaFailedBurst
        expr: rate(saga_failed_total[5m]) > 0.5
        for: 5m
        annotations:
          summary: "Saga 失败率过高"
          description: "5min 内失败率 > 0.5/s"
          runbook: "https://wiki/runbooks/saga-failed-burst"

      - alert: OutboxBacklogHigh
        expr: outbox_backlog{status="PENDING"} > 1000
        for: 10m
        annotations:
          summary: "Outbox backlog 过高"
          description: "{{ $labels.aggregate_type }} backlog = {{ $value }}"

      - alert: ManualInterventionQueue
        expr: saga_in_flight{state="MANUAL_INTERVENTION"} > 5
        for: 5m
        annotations:
          summary: "Manual Intervention 队列堆积"
          description: "需要 GM 介入的 saga 数量过多"

      - alert: SagaRuntimePodDown
        expr: kube_deployment_status_replicas{deployment="saga-runtime"} == 0
        for: 1m
        annotations:
          summary: "Saga Runtime 全 down"
          description: "K3s 部署的所有 saga-runtime pod 都不可用"

      - alert: NATSDown
        expr: kube_statefulset_status_replicas{statefulset="nats"} == 0
        for: 1m
        annotations:
          summary: "NATS JetStream down"

      - alert: InventoryCompensationFailures
        expr: rate(inventory_compensation_failures[15m]) > 0.1
        for: 15m
        annotations:
          summary: "Inventory 补偿失败率过高"
```

---

## 9. 关联文档

- **基础**：`RGS-REQ-100` / `RGS-BAS-100` / `RGS-DTL-100~102`
- **同侪**：
  - `RGS-OPS-100` Saga K3s 部署
  - `RGS-SEC-100` GM 审计与 Saga 安全
- **现有可观测性**：
  - `RGS-GOBS-001` 现有游戏服务器可观测性现状调查
  - `RGS-GOBS-003` 游戏服务器可观测性基本设计
  - `RGS-GOBS-004` Observability 导入计划

---

## 10. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。OpenTelemetry 全链路追踪（Mermaid 时序图：GM click → Admin Gateway → Saga → Economy → NATS → Inventory → Compensation）+ 5 类追踪 ID 规范 + 17 项 Metrics（4 类：Saga 9 + 基础设施 5 + 业务 3）+ Loki 日志字段强制表 + Tempo Span 结构 + Admin Saga Console 视图 + AlertManager 7 项告警。 |
