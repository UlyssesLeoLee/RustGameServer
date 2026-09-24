# RGS-OB-DASH-001: Outbox Grafana Dashboard v0.1 (per ULYS-100 P2-#2)

> **状态**: v0.1 (2026-09-19 JST, agent c557dae5 per DEC-008 代签)
> **前置依赖**: ADR-0061 待具名人类审批
> **关联**: `docs/00-基准与治理/ULYS-56-follow-up-drafts/06_Outbox监控指标草案.md` §3 Grafana 仪表盘候选布局

---

## 1. 仪表盘概述

| 项 | 内容 |
|---|---|
| 仪表盘名称 | RGS Outbox Overview (ULYS-100 P2-#2) |
| 文件位置 | `docker/observability/grafana/dashboards/rgs-outbox-overview.json` |
| Grafana UID | `rgs-outbox-overview` |
| 标签 | `rgs`, `outbox`, `ULYS-100` |
| 刷新间隔 | 15s |
| 默认时间窗口 | 最近 1h |
| 数据源 | Prometheus (per `docker/observability/grafana/provisioning/datasources/datasource.yml`) |

## 2. 12 Panel 布局

### Row 1 — Outbox Health (顶层 Stat, 4 panel)

| Panel | 类型 | PromQL | 阈值 |
|---|---|---|---|
| Total Pending (6 域 sum) | stat | `sum(rgs_outbox_pending_count)` | >500 黄, >1000 红 |
| Total InFlight (6 域 sum) | stat | `sum(rgs_outbox_inflight_count)` | >50 黄, >100 红 |
| Total Failed (6 域 sum) | stat | `sum(rgs_outbox_failed_count)` | >0 红 |
| Publish Success Rate 5m | stat | `avg(rgs_outbox_publish_success_rate_5m)` | <0.95 黄, <0.99 绿 |

### Row 2 — Pending / Publish Rate (Time series, 2 panel)

| Panel | PromQL |
|---|---|
| Pending by service | `sum by (service) (rgs_outbox_pending_count)` |
| Publish Success Rate by service (5m) | `rgs_outbox_publish_success_rate_5m` |

### Row 3 — Lag & Backlog (2 panel)

| Panel | PromQL |
|---|---|
| Oldest Pending Age by service | `rgs_outbox_oldest_pending_age_seconds` |
| Event Age Distribution (Histogram heatmap) | `sum by (le) (rate(rgs_outbox_event_age_seconds_bucket[5m]))` |

### Row 4 — Relay Performance (2 panel)

| Panel | PromQL |
|---|---|
| Relay Poll Cycle p50/p95/p99 | `histogram_quantile(0.50/0.95/0.99, sum by (service, le) (rate(rgs_outbox_relay_poll_cycle_duration_seconds_bucket[5m])))` |
| Publish Throughput by service | `sum by (service, result) (rate(rgs_outbox_relay_publish_total[5m]))` |

### Row 5 — Lease / Batch (2 panel)

| Panel | PromQL |
|---|---|
| Lease Lag by service | `rgs_outbox_inflight_lease_lag_seconds` (>25 黄, >28 红) |
| Batch Size Distribution by service (p50/p95) | `histogram_quantile(0.50/0.95, sum by (service, le) (rate(rgs_outbox_batch_size_bucket[5m])))` |

## 3. 模板变量 (Templating)

- `service` — 多选下拉, 来源 `label_values(rgs_outbox_pending_count, service)` (6 域 + 默认 all)
  - `admin` / `economy` / `match` / `player` / `social` / `cluster_ops`

## 4. 联动告警

Dashboard 7 类告警规则 (per `docker/observability/prometheus-rules/rgs-outbox-alerts.yaml`):

1. `OutboxPendingBacklog` warning (>1000/5min)
2. `OutboxFailedAccumulating` critical (>0/1min)
3. `OutboxLeaseExpiringSoon` warning (>25s/1min)
4. `OutboxPublishFailureRateHigh` warning (>5%/5min)
5. `OutboxOldestPendingStale` warning (>300s/1min)
6. `OutboxInflightCountHigh` warning (>100/5min)
7. `OutboxPollCycleSlow` warning (p99>5s/5min)

## 5. 部署步骤

1. Prometheus 容器挂载 `docker/observability/prometheus-rules/` (per `docker-compose.observability.yml` volume 配置)
2. `prometheus.yml` 已加 `rule_files` 引用 `/etc/prometheus/rules/rgs-outbox-alerts.yaml`
3. Grafana 自动加载 `dashboards/` 目录 (per `grafana/provisioning/dashboards/dashboards.yml`)
4. 启动后 `docker exec -it rgs-prometheus promtool check rules /etc/prometheus/rules/rgs-outbox-alerts.yaml` 验证规则合法
5. Grafana UI → Dashboards → RGS folder → "RGS Outbox Overview"

## 6. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-19 JST | 架构师 (Mavis 接手 agent per ULYS-100 P2-#2) | 初版：12 panel + 模板变量 + 7 类告警联动; 实施位置 `docker/observability/grafana/dashboards/rgs-outbox-overview.json`; 联动工单 ULYS-101 (P2-#3 DLQ) + ULYS-102 (P2-#4 Schema Evolution) |
