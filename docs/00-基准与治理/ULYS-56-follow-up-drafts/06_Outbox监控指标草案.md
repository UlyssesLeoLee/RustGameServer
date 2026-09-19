## 0. ULYS-100 实施进展 (2026-09-19 JST)

> **状态变更**: 草案 → **代码已落地 (agent c557dae5 per DEC-008 代签)**, 等 ADR-0061 具名人类审批通过后生效。
>
> **前置依赖**: ADR-0061 待具名人类审批 (per DEC-008, 与 ULYS-89 跟踪状态一致)。
> **落地范围**: 9 个 Prometheus 指标 (per §1.1 + §1.2) + 3 个 recording rules + 7 类告警规则 + 6 域 `/metrics` HTTP endpoint。
>
> **实施位置**:
> - 代码: `crates/shared-platform/src/{metrics,outbox_relay,outbox_metrics_reporter,metrics_endpoint}.rs` + 6 域 `main.rs` (admin / cluster-ops / economy / match / player / social)
> - 文档: `docs/02-运维安全与网络/RGS-BAS-004_埋点与日志规范_基本设计书.md` v0.4 §3.4
> - Prometheus 规则: `docker/observability/prometheus-rules/rgs-outbox-alerts.yaml`
> - Grafana dashboard: `docker/observability/grafana/dashboards/rgs-outbox-overview.json`
> - 集成测试: `crates/shared-platform/tests/it_outbox_metrics.rs` (10 scenarios)
>
> **Stage 4 (PH-4 实测校准阈值)**: 待 PH-4 启动后由 SRE Lead 执行; 当前 §2 阈值为候选草案, 需 PH-4 实测校准后正式化。

---

# ULYS-56 P2-#2 草案: Outbox 监控指标 (SRE Lead 主导)

> **状态**: 草案 (per ADR-0061 §6 P2-#2)。本文件是**候选监控指标 spec 草案**, 候选主导者 SRE Lead, 候选实施者 架构师 + SRE Lead, 待 ADR-0061 审批通过后正式立项。
>
> **依据**: RGS-ADR-0061 §6 P2-#2 (Outbox 监控指标补全)
>
> **ULYS-56 工作范围声明**: 本文件由 ULYS-56 worker (agent c557dae5) 2026-09-16 起草, 提供指标候选清单, **不直接修改可观测性体系文档**（RGS-GOBS / RGS-BAS-004 §3.2 指标体系 / NFR-OP-001/003 等级要求）。

# ULYS-56 P2-#2 草案: Outbox 监控指标 (SRE Lead 主导)

> **状态**: 草案 (per ADR-0061 §6 P2-#2)。本文件是**候选监控指标 spec 草案**, 候选主导者 SRE Lead, 候选实施者 架构师 + SRE Lead, 待 ADR-0061 审批通过后正式立项。
>
> **依据**: RGS-ADR-0061 §6 P2-#2 (Outbox 监控指标补全)
>
> **ULYS-56 工作范围声明**: 本文件由 ULYS-56 worker (agent c557dae5) 2026-09-16 起草, 提供指标候选清单, **不直接修改可观测性体系文档**（RGS-GOBS / RGS-BAS-004 §3.2 指标体系 / NFR-OP-001/003 等级要求）。

---

## 1. 候选指标清单 (per `crates/shared-platform/src/outbox.rs` 4 状态机)

### 1.1 Counter / Gauge

| 指标名 | 类型 | 单位 | Labels | 含义 | 阈值 / 告警 |
|---|---|---|---|---|---|
| `rgs_outbox_pending_count` | Gauge | 条 | `service`, `aggregate_type` | 当前 `status = 'Pending'` 的 outbox 行数 | > 1000 持续 5min → warning |
| `rgs_outbox_inflight_count` | Gauge | 条 | `service`, `aggregate_type` | 当前 `status = 'InFlight'` 且 `lease_until > NOW()` 的行数 | > 100 持续 5min → warning（指示 relay 拥塞） |
| `rgs_outbox_inflight_lease_lag_seconds` | Gauge | 秒 | `service` | InFlight 行 `lease_until - NOW()` 最大值 | > 30s → warning（接近 lease 过期阈值） |
| `rgs_outbox_failed_count` | Gauge | 条 | `service`, `aggregate_type` | 当前 `status = 'Failed'` 的行数 | > 0 持续 1min → critical（需人工介入 / DLQ 处置） |
| `rgs_outbox_relay_poll_cycle_duration_seconds` | Histogram | 秒 | `service` | 单次轮询周期耗时（SELECT + publish + UPDATE） | p99 > 5s → warning |
| `rgs_outbox_relay_publish_total` | Counter | 次 | `service`, `aggregate_type`, `result` | 发布次数（result = `success` / `failure` / `timeout`） | failure / timeout 比率 > 5% → warning |

### 1.2 Histogram / Summary

| 指标名 | 类型 | 单位 | Labels | 含义 |
|---|---|---|---|---|
| `rgs_outbox_event_age_seconds` | Histogram | 秒 | `service`, `aggregate_type` | outbox 行从 created_at 到 published_at 的端到端延迟分布 |
| `rgs_outbox_batch_size` | Histogram | 条 | `service` | 单次轮询批量大小（SELECT LIMIT N） |

### 1.3 派生指标 (Prometheus recording rules)

| 指标名 | 公式 | 含义 |
|---|---|---|
| `rgs_outbox_publish_success_rate_5m` | `rate(rgs_outbox_relay_publish_total{result="success"}[5m]) / rate(rgs_outbox_relay_publish_total[5m])` | 5min 发布成功率 |
| `rgs_outbox_publish_failure_rate_5m` | `1 - rgs_outbox_publish_success_rate_5m` | 5min 发布失败率（告警源） |
| `rgs_outbox_oldest_pending_age_seconds` | `time() - rgs_outbox_oldest_pending_created_at_seconds` | 最旧 Pending 行年龄（告警源） |

---

## 2. 关联 Prometheus 规则 (RGS-BAS-004 §3.2 候选追加)

```yaml
# RGS-BAS-004 v0.4+ 候选新增规则 (per ADR-0061 P2-#2)
groups:
  - name: rgs_outbox_alerts
    interval: 30s
    rules:
      - alert: OutboxPendingBacklog
        expr: rgs_outbox_pending_count > 1000
        for: 5m
        labels:
          severity: warning
          team: sre
        annotations:
          summary: "Outbox pending backlog growing on {{ $labels.service }}/{{ $labels.aggregate_type }}"
          description: "{{ $value }} pending outbox rows, check relay publish rate"
      - alert: OutboxFailedAccumulating
        expr: rgs_outbox_failed_count > 0
        for: 1m
        labels:
          severity: critical
          team: sre
        annotations:
          summary: "Outbox Failed state accumulating on {{ $labels.service }}"
          description: "{{ $value }} failed rows, requires DLQ intervention per ADR-0061 P2-#3"
      - alert: OutboxLeaseExpiringSoon
        expr: rgs_outbox_inflight_lease_lag_seconds > 25
        for: 1m
        labels:
          severity: warning
          team: sre
        annotations:
          summary: "Outbox lease about to expire on {{ $labels.service }}"
          description: "InFlight lease lag {{ $value }}s, approaching 30s reclaim threshold"
      - alert: OutboxPublishFailureRateHigh
        expr: rgs_outbox_publish_failure_rate_5m > 0.05
        for: 5m
        labels:
          severity: warning
          team: sre
        annotations:
          summary: "Outbox publish failure rate {{ $value | humanizePercentage }} on {{ $labels.service }}"
          description: "Check NATS JetStream connectivity + Debezium-sink dependency (none per ADR-0061)"
```

---

## 3. Grafana 仪表盘候选布局 (FR-OB-004 候选扩展)

| Row | Panel | 数据源 |
|---|---|---|
| Outbox Health (顶层) | Stat panel: total pending / total inflight / total failed (6 service grid) | `rgs_outbox_pending_count`, `rgs_outbox_inflight_count`, `rgs_outbox_failed_count` |
| Outbox Health (顶层) | Time series: publish success rate 5m (6 service line plot) | `rgs_outbox_publish_success_rate_5m` |
| Lag & Backlog | Time series: oldest pending age (6 service line plot) | `rgs_outbox_oldest_pending_age_seconds` |
| Lag & Backlog | Histogram: event age distribution (per service) | `rgs_outbox_event_age_seconds_bucket` |
| Relay Performance | Time series: poll cycle duration p50/p95/p99 (6 service) | `rgs_outbox_relay_poll_cycle_duration_seconds_bucket` |
| Relay Performance | Time series: relay publish throughput (rate per service) | `rate(rgs_outbox_relay_publish_total[5m])` |

---

## 4. 候选实施路径

| 阶段 | 内容 | 责任方 |
|---|---|---|
| Stage 1 | 在 `crates/shared-platform/src/outbox.rs` + `outbox_relay.rs` 添加 `metrics` crate 集成, 输出候选清单 §1 全部指标 | 架构师 |
| Stage 2 | 在 `crates/shared-platform` 提供 `OutboxMetrics` trait, 6 域 `main.rs` 启动 outbox relay 时初始化 metrics endpoint (HTTP `/metrics`) | 架构师 |
| Stage 3 | RGS-BAS-004 v0.4+ 增补 §1 候选规则（告警阈值 + 仪表盘布局） | 架构师 + SRE Lead |
| Stage 4 | PH-4 实测校准阈值 (per TBD-GOV-001 季度实测方法) | SRE Lead |

---

## 5. 候选操作者签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 起草 (ULYS-56 worker) | worker (agent c557dae5) | 2026-09-16 JST |
| 候选主导者 | SRE Lead（Ulysses 一人公司兼任 per DEC-008） | 待 ADR-0061 具名审批通过后 |
| 候选实施者 | 架构师 + SRE Lead 联合 | Stage 1-4 顺序 |

---

## 6. 关联文档

- **RGS-ADR-0061 §6 P2-#2**: 后续工作项
- **RGS-BAS-004 §3.2** (可观测性指标体系, 待扩展)
- **NFR-OP-001/003** (可观测性等级要求)
- **RGS-REV-007 CH1+CH2+AH1** (Outbox 升级记录, 含历史指标需求)
- **`crates/shared-platform/src/outbox.rs`** (代码层, 待 metrics 集成)
- **RGS-OB-DASH-001** (Grafana 仪表盘, 待扩展 outbox 视图)