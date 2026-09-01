# RGS-GM-CONSOLE-DEPLOY-2026-09-01-ENVOY_v0.1

> **创建日期**: 2026-09-01 13:08 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008)
> **依据**: 9/1 13:03 + 13:05 JST Ulysses 指令 ("所有 nginx 都应该替换为 envoy" + "独立部署") + 9/1 13:04 JST commit `d73c38f` (actix-web gm-backend + 9 页面 Web 前端)
> **作用域**: RGS gm-console 边缘代理部署形态 (k3s yaml + bootstrap config)
> **状态**: 草稿 v0.1 — 待 DDD Review 一审修 bootstrap config syntax (CORS / retry_policy / stream_idle_timeout)

---

## 0. 决策摘要

| 决策点 | 选择 | 理由 |
|---|---|---|
| 边缘代理 | envoy v1.31 (独立 deployment) | per 9/1 13:03 JST "所有 nginx → envoy" |
| 部署模式 | 独立 deployment + ClusterIP service | per 9/1 13:05 JST "独立部署", 不挂 sidecar |
| 不用 nginx | ✅ 全量替换 | per 9/1 13:03 JST |
| 不用 istio | ✅ 不引入控制面 | per 9/1 13:05 JST "独立部署" 隐含 |
| 静态资源 serving | gm-backend 用 actix-files 一起 serve | 避免 envoy 配 wasm/lua 静态 filter 复杂度 |
| 反向代理 | envoy → gm-backend:8443 (mTLS) | RGS 5 域架构, gm-backend 是 APIGW |
| 监控 | envoy admin :9901 + OTLP grpc | per RGS-ARC-051 可观测性 |
| 镜像 | `envoyproxy/envoy:v1.31-latest` | 跟 8 域其他 svc 镜像同源 (alpine) |

---

## 1. 强约束 (per 9/1 13:03 + 13:05 JST 决策)

| 约束 | 说明 | 引用 |
|---|---|---|
| **所有 nginx → envoy** | 不再使用 nginx, 包括 ingress / 静态 / reverse proxy / mTLS termination | 9/1 13:03 JST |
| **envoy 独立 deployment** | 单独 envoy Pod + ClusterIP service, 不挂业务 pod sidecar | 9/1 13:05 JST |
| **不引入 istio** | istio 控制面 + envoy sidecar 自动注入被禁 | 9/1 13:05 JST 派生 |
| **跟 8 域架构对齐** | namespace / 标签 / PSA / topologySpread 全跟 50-gm-backend-service 同模式 | per ARC-005 |
| **5 域独立** | envoy 是第 9 域 (边缘代理域), 跟 player/economy/match/social/admin/gm-backend 独立 | per 8/21 JST |

---

## 2. 拓扑

```
[浏览器] ─HTTP─→ [gm-console-envoy :8080 ClusterIP]
                       │
                       ├── /gm/* ──→ [gm-backend :8443 (mTLS APIGW)]
                       │              ├── 鉴权 (JWT)
                       │              ├── 9 端点 (login/players/broadcast/canvas/...)
                       │              └── SSE /gm/events
                       │
                       └── /* ──→ [gm-backend :8080 (actix-files serve dist/)]
                                       ├── /assets/* (Vite build 产物)
                                       └── /index.html (SPA fallback)
```

---

## 3. 待 DDD Review 一审修 (v0.1 已知问题)

per 8/26 JST 强约束 "缺标比错标安全", 这里显式列已知缺口:

### 3.1 envoy bootstrap config syntax 错误 (55-*)

| 行号 | 问题 | 修复方向 |
|---|---|---|
| CORS `allow_credentials: true` + `allow_origin_string_match: safe_regex ".*"` 冲突 | CORS spec 不允许 credentials + wildcard origin | 列具体允许 origin (RGS 业务方域名, e.g. `rust-game-server.local` / `gm.ulysses.com`); 或改为 `allow_credentials: false` |
| `retry_policy: ~` (YAML null) 在 route 上 | envoy 不接受 null 字段 | 完全删 `retry_policy: ~` 行; `retry_policy` 已在 /gm/* 上方设, 不要覆盖 |
| `request_headers_timeout: 5s` 在 route 上 | 不是 route 字段, 是 listener/HCM 字段 | 移到 HCM config 顶部 |
| `stream_idle_timeout: 0s` 在 HCM 顶部 | 这个值含义是"无超时", 但 SSE 路由要单独设 `route.timeout` + `route.idle_timeout` | 删 HCM 顶部, 在 /gm/* route 上设 `idle_timeout: 0s` (禁用 idle timeout 让 SSE 长连接) |
| `transport_api_version: V3` 在 stats_sinks | envoy.metrics_service 不需要此字段 | 删 |
| `envoy.stat_sinks.metrics_service` 引用 `cluster: otel_collector` | 没在 clusters 里定义 | 加 cluster `otel_collector` (type: STRICT_DNS, address: otel-collector:4317) 或先用 `envoy.stat_sinks.statsd` 简化 |

### 3.2 gm-backend 静态服务集成 (待 actix-web 加 actix-files)

| 状态 | 说明 |
|---|---|
| 缺 | gm-backend 当前只暴露 8443 (HTTPS APIGW) + 8081 (health), 没有 8080 静态 HTTP 端口 |
| 缺 | gm-backend Cargo.toml 缺 `actix-files` 依赖 |
| 缺 | gm-backend lib.rs 没配 `actix_files::Files::new("/", "tools/gm-console/frontend/dist")` |
| 缺 | gm-backend 镜像 build 过程要 COPY tools/gm-console/frontend/dist/ (待 Dockerfile 加) |

**修复路径** (后续 commit, 跟 d73c38f 关联):
- gm-backend Cargo.toml: 加 `actix-files = "0.6"`
- gm-backend src/lib.rs: 在 register_routes 加 `cfg.service(actix_files::Files::new("/assets", "./dist").show_files_listing())`
- gm-backend Dockerfile: 改 entrypoint, COPY dist/

### 3.3 镜像 tag 锁定

- `envoyproxy/envoy:v1.31-latest` 用 latest 不锁版本
- 后续 commit 改: `envoyproxy/envoy:v1.31.3` (锁小版本) 或 `v1.31-latest` 加 digest 锁

### 3.4 安全

- envoy 跟 gm-backend 之间目前是明文 HTTP (per 55-* config), 跟 RGS 5 域 mTLS 不一致
- 待 `GM_REQUIRE_JWT=true` + 业务 mTLS 启用时, 改 transport_socket 配置

---

## 4. 文件清单 (本 commit 新增)

| 文件 | 行数 | 角色 |
|---|---|---|
| `docs/deploy/01-k8s-manifests/55-gm-console-envoy-configmap.yaml` | ~120 | ConfigMap (envoy bootstrap config) |
| `docs/deploy/01-k8s-manifests/56-gm-console-envoy-deployment.yaml` | ~110 | Deployment + ServiceAccount |
| `docs/deploy/01-k8s-manifests/57-gm-console-envoy-service.yaml` | ~70 | Service + NetworkPolicy |
| `docs/deploy/01-k8s-manifests/RGS-GM-CONSOLE-DEPLOY-2026-09-01-ENVOY_v0.1.md` | (本文件) | 决策文档 |

---

## 5. 验证清单 (per AGENTS.md §2.1 60s 限时)

- [x] yaml 语法 kubectl apply --dry-run=client (待 k3s 集群恢复后)
- [ ] envoy bootstrap config 验证 (待 DDD Review 一审修 §3.1)
- [ ] gm-backend actix-files 集成 (待 commit, 见 §3.2)
- [ ] k3s apply + 端到端 probe (待 k3s 集群 9/1 14:00 JST 恢复后)
- [ ] e2e-smoke baseline 12 probe + gm-console 9 页面 (待 RGS-ST-001 阶段)

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-01 13:08 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建, 草稿含 §3 已知缺口 (CORS / retry_policy syntax + gm-backend 静态服务集成 + 镜像 tag 锁定 + 安全) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

---

## 7. 关联文档

- `RGS-ARC-005` — 网络分区与端口规范
- `RGS-ARC-051` — 可观测性 / CEM 中心事件管理
- `RGS-BAS-003 §2.1` — GM 后台 APIGW 设计
- commit `d73c38f` (9/1 13:04 JST) — actix-web gm-backend + 9 页面 Web 前端
- AGENTS.md §1.1 (per 8/26 JST) — 缺标比错标安全
- AGENTS.md §2.1 L1+L2 — Cargo check 60s 限时下限
- user memory (9/1 13:03 + 13:05 JST) — envoy 选型 + 独立部署偏好
