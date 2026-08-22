# 可观测性栈（per WBS v0.3 §2A.5 WF-1-53.12）

## 组件

| 组件 | 镜像 | 端口 | 用途 |
|---|---|---|---|
| otel-collector | otel/opentelemetry-collector-contrib:0.110.0 | 4317/4318/8889/13133 | OTLP gRPC/HTTP 接收 + Prometheus 导出 |
| prometheus | prom/prometheus:v2.54.1 | 9090 | metrics scrape + TSDB 存储 |
| grafana | grafana/grafana:11.2.0 | 3000 | dashboards + alerting |

## 启动（叠加 53.8 docker-compose）

```bash
cd docker/compose
docker compose -f docker-compose.yml -f ../observability/docker-compose.observability.yml --profile dev up -d
```

## 接入 5 域（待 54.x 业务实施）

5 域服务的 gRPC 端口在 53.8 已暴露，OTel SDK 接入待：
- 54.13 OTel span 注入（每 gRPC method）
- 54.14 Prometheus metrics 暴露（每 service /metrics）
- 54.15 tracing 日志 + 结构化输出

## Grafana 默认

- URL: http://localhost:3000
- 用户: admin
- 密码: 见 docker/compose/.env GRAFANA_ADMIN_PASSWORD
- Dashboard provider 自动加载 `docker/observability/grafana/dashboards/`

## 53.12 范围

- ✅ OTel Collector 配置（OTLP + Prometheus exporter + debug）
- ✅ Prometheus 配置（5 域 scrape + OTel + 自抓）
- ✅ Grafana provisioning（datasource + dashboard provider + 1 占位 dashboard）
- ✅ docker-compose 增量文件
- ⚠ 5 域服务实际暴露 /metrics 待 54.14
- ⚠ 真实业务 dashboard 待 55.x（service 实施后）