# RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1

**综合 Batch 管理平台基本设计（rgs-batch-console + rgs-batch-backend）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BATCH-BASIC-DESIGN-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 18:00 JST "batch 平台" 决策 + 18:34 JST Q2 "独立双项目" 拍板）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1（commit `fd122f6`）已落地，本层补基本设计 |
| 关联 | 上游 RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1（commit `fd122f6`）+ 下游 RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1（待起草）+ RGS-BATCH-PLAN-2026-09-01 v0.1（待起草）|
| 上游基线 | rgs-web v0.3 commit `625a3f0`（merge 5 域 gRPC + 6 API + http2 + mTLS + port-forward, per 8/26 22:47 JST）+ rgs-web OLU-WEB 4 文档 + gm-backend 范式 + 5 域 ST 业务级 mTLS 实践（commit `401ac5c`）+ 9/1 PT 派工 commit `ffbfb19` + saga-runtime 独立 Pod（per `docs/01-核心架构与设计模式/RGS-BAS-100_Saga事务系统基本设计书_v0.1.md`）|
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 文档定位

本文档是 rgs-batch 综合平台**基本设计层**，回答"How 概要"——不涉及"What + Why"（已 in REQUIREMENTS v0.1）也不涉及"How 细节"（在 DETAILED-DESIGN）。

按 RGS 项目规范（per RGS-DTL-001 设计模式）：需求 → 基本 → 详细。

**继承上游 REQUIREMENTS v0.1 全部内容**：1 现状 + 12 痛点 + 10 业务目标 + 7 US + 44 FR（16 P0 + 10 P1 + 9 P2 + 9 Won't）+ 16 张表 DB 三分类横展 + 12 IR + 14 NFR + 治理/技术/时间/资源约束 + 12 GAP + 8 R。本文档不重复，只补"如何做"。

---

## 1. 架构总览

### 1.1 子系统在 RGS 整体架构中的位置

```
RGS 整体架构 (per 9/1 k3s 部署恢复后现状)
├── 5 域 gRPC 服务 (player / economy / match / social / admin) + cluster-ops
│   ├── 端口 50051-50055 (k8s targetPort, per docs/deploy/01-k8s-manifests/0{1-5}-*-service.yaml)
│   ├── mTLS 业务级 (per 5 域 ST 业务级 mTLS 实践 commit 401ac5c)
│   └── 1 人 12 角色下 5 域独立 Lead
├── shared-platform (outbox / tracing / span_helpers / retry / dlq / grpc_tracing / rbac / tls / metrics / messaging 等 20 模块)
│   └── crates/shared-platform/ (per 9/1 PT 派工 commit ffbfb19)
├── function-plane (gateway + registry + wasm_host + contract)
│   └── crates/function-plane/ (与 saga-runtime 独立, 详见 RGS-BAS-100)
├── rgs-web (Node 22, 127.0.0.1:8788, 10 页面 dashboard, 5 域 gRPC 接入)
│   └── tools/rgs-web/ (per commit 625a3f0)
├── gm-backend (actix-web APIGW, 8443 HTTPS / 8081 HTTP health / 9464 metrics)
│   └── crates/gm-backend/ + docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml
└── ★ rgs-batch (新增, per 9/1 18:34 JST Q2 拍板) ★
    ├── rgs-batch-console (Node 22, 127.0.0.1:8789, 0 依赖前端)
    ├── rgs-batch-backend (actix-web, ClusterIP:8790, 5 域 gRPC client)
    └── envoy (独立 deployment, mTLS termination, per 9/1 13:03/13:05 JST 偏好)
```

### 1.2 rgs-batch 双项目架构

```
Browser (127.0.0.1, Ulysses)
  ↓ HTTPS (envoy TLS termination)
envoy (独立 deployment, file_system HTTP filter + mTLS termination)
  ├─→ rgs-batch-console (127.0.0.1:8789, Node 22 + 原生 http, 0 依赖)
  │     ├─→ static assets (HTML/CSS/JS)
  │     └─→ reverse proxy → rgs-batch-backend API
  │              ↓ HTTP API (cluster 内部)
  └─→ rgs-batch-backend (ClusterIP:8790, actix-web + tokio)
        ├─→ 5 域 gRPC client (player 50051 / economy 50052 / match 50053 / social 50054 / admin 50055)
        │     └─→ mTLS 双向认证 (per 5 域 ST 业务级 mTLS 实践 commit 401ac5c)
        ├─→ PostgreSQL (5 域共享实例 + schema batch_master / batch_transaction / batch_work)
        │     └─→ 16 张表 (per REQUIREMENTS §4 DB 三分类横展)
        ├─→ shared-platform 复用 (outbox / tracing / span_helpers / retry / dlq / rbac / tls)
        ├─→ (v0.2 集成) saga-runtime Pod (per RGS-BAS-100 v0.1, F-27 + GAP-11 跨域 saga 触发)
        └─→ (v0.2 集成) rgs-web /api/batch/* 代理 (F-34)
```

### 1.3 数据流（4 类核心场景）

#### 1.3.1 GM 批量发奖流程

```
Browser POST /api/batch/gm-grant { player_ids, rewards }
  ↓
rgs-batch-console (8789) handler:
  - 验证 params (player_ids 非空 + rewards 合法)
  - 写 audit_session W-3 (operator=Ulysses, ip=127.0.0.1)
  - 转发 POST /api/v1/tasks { type: 'gm_grant', params }
  ↓
rgs-batch-backend (8790) /api/v1/tasks:
  - 写 task_def M-1 (status=pending)
  - 写 task_execution T-1 (exec_id, params_snapshot, trace_id)
  - 入队 → worker pool (5 worker 默认)
  - 返回 { task_id, exec_id, status: 'pending' }
  ↓
worker pool 1 worker pick:
  - 读 task_def + task_execution
  - sharder 分片 (默认每片 100 player, 可配)
  - 写 task_buffer W-2 (chunks)
  - 写 task_progress W-1 (total = chunks × 100, completed = 0)
  ↓
5 worker 并发执行 per chunk:
  - 5 域 gRPC client (限流 per worker_pool M-4)
  - 每个 player → 1 sub_task T-2
  - 成功 → 写 sub_task.completed_at
  - 失败 → 写 sub_task.error → retry 3 次 (指数退避 100/200/400ms, 复用 shared-platform/retry)
  - 重试 3 次仍失败 → 写 dlq_event T-4
  ↓
任务完成判断:
  - 所有 sub_task 完成 (success + DLQ) → status = completed
  - 写 task_execution.finished_at + result_summary
  - 清理 task_buffer W-2 + task_progress W-1
  - 触发 mavis cron 告警 (v0.2, per F-23)
  ↓
Browser 30s 轮询 GET /api/batch/tasks/{id}/progress:
  - 读 task_progress (W-1, in-memory) + task_execution (T-1) + sub_task count (T-2)
  - 返 { completed, failed, dlq, total, eta_seconds, status }
  - 任务完成 → 浏览器跳转任务详情页
  ↓
审计 (always, 永久保留 per NFR-29):
  - 任务创建 → audit_event T-3 (action=create)
  - 任务完成 → audit_event T-3 (action=complete, result)
  - 子任务失败 → audit_event T-3 (action=sub_task_fail)
  - DLQ → audit_event T-3 (action=dlq)
```

#### 1.3.2 定时任务调度流程

```
rgs-batch-backend 启动:
  - 读 schedule M-5 → 启动 tokio-cron-scheduler
  ↓
cron tick (per schedule, next_run_at <= now):
  - 选 schedule_id
  - 读 task_def M-1
  - 创建 task_execution T-1 (triggered_by=cron)
  - 入队 worker pool (同 GM 流程)
  - 更新 schedule.next_run_at = next cron tick
  ↓
(interval 同上, 用 tokio::time::interval 替代 cron)
(一次性 同上, 用 tokio::time::sleep_until(at) 替代 cron)
```

#### 1.3.3 Log 批量处理流程

```
rgs-batch-console /api/batch/log-process POST { source, filter, aggregate, output }
  ↓
rgs-batch-backend /api/v1/log-tasks:
  - 拉取 log 源 (5 域 gRPC interceptor + 文件 glob + kubectl logs)
  - 过滤 (level / pattern / time range / 自定义 SQL where)
  - 聚合 (count / sum / avg / p95 / p99 / group by)
  - 输出 (PostgreSQL batch_transaction / CSV 文件 / rgs-web embed)
  - 任务进度轮询 (W-1)
  ↓
审计 T-3 (action=log_process, params_hash)
```

#### 1.3.4 数据整理流程

```
rgs-batch-console /api/batch/data-migration POST { source, target, type }
  ↓
rgs-batch-backend /api/v1/migration-tasks:
  - before snapshot (写 data_migration T-6, 含源表数据)
  - 执行迁移 (SQL 模板 / 5 域 gRPC list + write)
  - 生成 rollback SQL (基于 before snapshot, per F-24)
  - dry-run 模式 (可选, per F-19)
  - 任务进度轮询 (W-1)
  ↓
审计 T-3 (action=migration, params_hash, rollback_sql_ref)
```

### 1.4 与外部系统的关系

| 外部系统 | 关系 | v0.1 数据流 | 触发点 |
|---|---|---|---|
| 5 域 gRPC | 调用方 (rgs-batch-backend → 5 域) | gRPC + mTLS | 任务执行时 |
| PostgreSQL | 持久化 (Master / Transaction / Work) | 直接 SQL (sqlx) | 任务定义 / 进度 / 审计 |
| shared-platform | 复用 (outbox / tracing / retry / dlq / rbac / tls) | Rust crate import | 编译时 |
| function-plane | v0.1 不集成 (与 saga-runtime 独立) | Rust crate import (v0.2 评估) | 任务跨域编排 (v0.2 per F-27) |
| rgs-web | 联动 (v0.2 深联动 per F-34) | HTTP API 代理 | rgs-web 加 page-batch (v0.2) |
| mavis cron | 告警 (v0.2 per F-23 + IR-4) | mavis cron self-reminder | 任务失败 / 阈值告警 |
| GitHub | 审计回写 (v0.2 per IR-10) | https POST issue comment | 任务完成 |
| envoy | 边缘代理 | 独立 deployment | 浏览器访问 |
| OTel collector | 链路追踪 (per shared-platform/tracing_init) | OTLP 4317 | 任务执行时 |
| Prometheus | 指标 scrape (per OLU-WEB 实践) | metrics 9464 | 任务执行时 |

---

## 2. 技术选型

> 母规范 rgs-web + OLU-WEB + gm-backend **全部继承**。本节列 rgs-batch **新增**选型。

### 2.1 rgs-batch-console (前端)

| 维度 | 选择 | 备选 | 决策理由 |
|---|---|---|---|
| 运行时 | Node 22 LTS | Node 20 LTS | rgs-web 母规范 v0.1 已选 22 (per commit 625a3f0) |
| HTTP | Node 原生 `http` + `http2` | Express / Koa / Fastify | 0 依赖 (per rgs-web 母规范 §2.1) |
| TLS 客户端 | Node 原生 `https` | node-fetch | 0 依赖 |
| 前端框架 | 原生 HTML + CSS + JS (vanilla) | React / Vue / Svelte | 0 依赖 (per rgs-web v0.3 现状) |
| 图表 | 原生 SVG + CSS bar | chart.js / d3 | 0 依赖 (per rgs-web OLU-WEB §2.1) |
| 轮询 | `setInterval` 30s | WebSocket / SSE | v0.1 简化, v0.2 WebSocket (per F-28) |
| 任务编辑器 | 表单 + JSON 预览 | Monaco Editor | 0 依赖 + 1 人工具 |
| 锁文件 | `fs.openSync(path, 'wx')` 原子创建 | proper-lockfile | 0 依赖 + 原子 (per OLU-WEB §5.1.5) |
| 数据存储 | JSON 文件 + jsonl append-only | SQLite / markdown frontmatter | 0 依赖 (per OLU-WEB 决策) |
| 部署 | envoy 独立 deployment (file_system HTTP filter) | nginx | per 9/1 13:03/13:05 JST 偏好 |
| 监听地址 | 127.0.0.1:8789 | 0.0.0.0 | per rgs-web 母规范 §2.1 NFR-10 (127.0.0.1 only) |
| 凭据 | env var only (K3S_TOKEN / GITHUB_TOKEN) | 配置文件 | per 8/27 11:06 JST env value 硬 ban |
| 进程模型 | 单进程单线程 | cluster / worker_threads | 1 写者约束 (per OLU-WEB §5.1.5) |

### 2.2 rgs-batch-backend (后端)

| 维度 | 选择 | 备选 | 决策理由 |
|---|---|---|---|
| 运行时 | Rust stable (per `rust-toolchain.toml`) | Go / Java | 跟 5 域 / shared-platform 同栈 |
| Web 框架 | actix-web 4.x | axum / warp | 跟 gm-backend 同栈 (per 8/27 ST 实践) |
| async runtime | tokio (actix-web 默认) | async-std | 5 域统一 |
| gRPC client | tonic 0.12.x | grpc-rs | 5 域都用 tonic (per RGS-INC-002) |
| TLS | rustls + tokio-rustls | native-tls | 5 域 ST 业务级 mTLS 一致 |
| DB 客户端 | sqlx 0.7.x (async + compile-time check) | diesel (非 async + 编译慢) | 5 域统一 + async 优势 |
| DB migration | sqlx migrate | refinery | 5 域统一 (5 域都用 sqlx) |
| JSON | serde_json | simd-json | 5 域统一 |
| Tracing | tracing + tracing-subscriber + tracing-opentelemetry | log | 5 域统一 + 复用 shared-platform |
| OTel | opentelemetry + opentelemetry-otlp | - | 复用 shared-platform/tracing_init |
| Worker 池 | tokio::spawn + mpsc channel (5 worker 默认) | actix actor / rayon | 简单 + 5 worker 池够用 |
| 调度 | tokio-cron-scheduler 0.3.x | 自写 cron parser | cron 表达式 + interval + 一次性 三种 |
| outbox | 复用 shared-platform/outbox.rs | 自写 | 跟 5 域 outbox 模式一致 |
| retry | 复用 shared-platform/retry.rs | backoff crate | 5 域统一 (指数退避 100/200/400ms) |
| DLQ | 复用 shared-platform/dlq.rs | 自写 | 5 域统一 |
| RBAC | 复用 shared-platform/rbac.rs | 自写 | 跟 5 域 rbac 一致 (虽然一人公司, 但保持一致) |
| TLS 配置 | 复用 shared-platform/tls.rs | rustls raw | 5 域统一 |
| 锁文件 | `fs2` crate 或 `fs::OpenOptions::new().create_new(true)` | nix | 0 依赖 + 原子 |
| 时间处理 | chrono | time | 5 域统一 |
| 部署 | envoy 独立 deployment + ClusterIP service | nginx / istio | per 9/1 13:03/13:05 JST 偏好 |
| 监听地址 | 0.0.0.0:8790 (ClusterIP 内部) | 127.0.0.1:8790 (跟 console 冲突) | k3s ClusterIP service, 不暴露 127.0.0.1 |
| 凭据 | env var only | 配置文件 / 凭据文件 | per 8/27 11:06 JST 硬 ban, 启动时 `std::env::var()` 引用, **不**打印值 |
| 进程模型 | 单进程 + tokio multi-thread runtime | cluster | 5 worker 池 + tokio 调度 |

### 2.3 不选用的方案

| 方案 | 不选理由 |
|---|---|
| **Express / Koa / Fastify** | npm 依赖, 违反 rgs-web 母规范 §2.1 "0 依赖" |
| **React / Vue / Svelte** | 同上, vanilla JS 够用 (per rgs-web v0.3 现状) |
| **dhtmlx-gantt / frappe-gantt / chart.js / d3** | npm 依赖 + SVG 手写足够 (per OLU-WEB §2.1) |
| **better-sqlite3** | native binding 编译慢 (per OLU-WEB 决策) |
| **markdown frontmatter 存配置** | 解析成本高, 不能聚合 (per OLU-WEB 决策) |
| **WebSocket / SSE** | 30s 轮询够用, v0.2 升级 (per F-28) |
| **Docker 镜像强制** | 一人公司本机工具, envoy 部署到 k3s 但不强制 Docker (per OLU-WEB §F-W2) |
| **nginx 反代** | per 9/1 13:03/13:05 JST 偏好 envoy 独立 deployment |
| **istio sidecar** | per 9/1 13:05 JST 偏好 envoy 独立 deployment, **不**选 istio 控制面 |
| **axum / warp** | 跟 gm-backend 不一致, gm-backend 范式优先 (per 8/27 ST 实践) |
| **diesel ORM** | 编译慢 + 不 async, sqlx 优势明显 |
| **Kafka / NATS 流式** | 过度工程, v0.1 简单 worker 池 + tokio mpsc channel 够用 (per OLU-WEB §2.1) |
| **Spark / Flink** | 商业流式引擎, 1 人 12 角色场景下自研够用 (per REQUIREMENTS F-W6) |
| **商业 ETL 工具** | 同上 (per REQUIREMENTS F-W7) |
| **mavis hook 阻塞等待** | v0.1 降级为 mavis session list 拉历史 + 估算 (per OLU-WEB §1.2 风险 1) |
| **登录 / RBAC** | DEC-008 一人公司, 127.0.0.1 only 足够 (per OLU-WEB §F-W1) |
| **agent 主进程跑业务** | rgs-batch-backend 独立进程, 不嵌入 mavis 主代理 (per OLU-WEB §1.3) |
| **Helm chart 模板化** | v0.1 单 namespace 3 deployment 够用, v0.2 评估 |
| **Terraform / Pulumi** | v0.1 k3s kubectl apply 够用, v0.2 评估 |
| **ArgoCD / Flux** | 同上, 1 人 12 角色下 git-ops 太重 |

---

## 3. 模块划分

### 3.1 rgs-batch-console 模块清单 (11 模块)

| # | 模块 | 文件位置 | 输入 | 输出 | 复用母规范 |
|---|---|---|---|---|---|
| 1 | 启动 + 路由 | `tools/rgs-batch-console/server.js` | HTTP 请求 | HTTP 响应 + 静态资源 | 沿用 rgs-web 母规范 §3 |
| 2 | 主页 (Dashboard) | `tools/rgs-batch-console/public/index.html` + `app.js` | /api/batch/* | UI 渲染 | 沿用 nav / 路由 |
| 3 | GM 批量发奖页 | `app.js` 内 #page-gm-grant | /api/batch/gm-grant | 表单 + 预览 + 进度 | 沿用 |
| 4 | 定时任务调度页 | `app.js` 内 #page-schedule | /api/batch/schedule | cron 编辑器 + 列表 | 沿用 |
| 5 | Log 批量处理页 | `app.js` 内 #page-log-process | /api/batch/log-process | 源选择 + 过滤 + 聚合 | 沿用 |
| 6 | 数据整理页 | `app.js` 内 #page-data-migration | /api/batch/data-migration | 源 + 目标 + 操作 | 沿用 |
| 7 | 任务监控页 | `app.js` 内 #page-tasks | /api/batch/tasks | 任务列表 + 详情 + KPI | 沿用 |
| 8 | 审计检索页 | `app.js` 内 #page-audit | /api/batch/audit | 按 task_id / player_id 检索 | 沿用 |
| 9 | 后台 API handler | `tools/rgs-batch-console/server.js` 内 router | HTTP / fetch | JSON | 沿用 rgs-web 模式 |
| 10 | 锁文件工具 | `tools/rgs-batch-console/lib/lockfile.js` | fs.openSync | lock handle | 沿用 OLU-WEB 实践 |
| 11 | token 估算器 | `tools/rgs-batch-console/lib/token-estimate.js` | message count | tokens | 沿用 OLU-WEB 公式 |

### 3.2 rgs-batch-backend 模块清单 (31 模块)

| # | 模块 | 文件位置 | 输入 | 输出 | 复用 |
|---|---|---|---|---|---|
| 1 | 启动 + 路由 | `tools/rgs-batch-backend/src/main.rs` | HTTP 请求 | HTTP 响应 | actix-web |
| 2 | 任务定义 API | `src/api/task_def.rs` | POST /api/v1/tasks | task_id | actix-web |
| 3 | 任务执行查询 API | `src/api/task_execution.rs` | GET /api/v1/tasks/{id} | 任务详情 | actix-web |
| 4 | 子任务查询 API | `src/api/sub_task.rs` | GET /api/v1/tasks/{id}/sub-tasks | 子任务列表 | actix-web |
| 5 | 调度 API | `src/api/schedule.rs` | CRUD /api/v1/schedules | schedule_id | actix-web |
| 6 | Log 任务 API | `src/api/log_task.rs` | POST /api/v1/log-tasks | log_task_id | actix-web |
| 7 | 数据迁移 API | `src/api/migration.rs` | POST /api/v1/migration-tasks | migration_id | actix-web |
| 8 | 模板 API | `src/api/template.rs` | CRUD /api/v1/templates | template_id | actix-web |
| 9 | 审计 API | `src/api/audit.rs` | GET /api/v1/audit + filter | audit_event 列表 | actix-web |
| 10 | DLQ API | `src/api/dlq.rs` | GET /api/v1/dlq + 操作 | DLQ 列表 + 操作结果 | actix-web |
| 11 | 任务执行引擎 | `src/engine/executor.rs` | task_id | 进度 / 结果 | tokio + 5 worker |
| 12 | worker 池 | `src/engine/worker_pool.rs` | task 队列 | 并发执行 | tokio::spawn + mpsc |
| 13 | 分片器 | `src/engine/sharder.rs` | 任务 + 配置 | chunk 列表 | 自写 |
| 14 | 限流器 | `src/engine/rate_limiter.rs` | 5 域 RPM | 限流信号 | 复用 shared-platform/retry |
| 15 | 5 域 gRPC client (5 个) | `src/grpc_clients/{player,economy,match,social,admin}.rs` | RPC 调用 | 响应 / 错误 | tonic |
| 16 | TLS 配置 | `src/grpc_clients/tls.rs` | cert 路径 | rustls config | 复用 shared-platform/tls |
| 17 | cron 调度器 | `src/scheduler/cron.rs` | cron 表达式 | tick 事件 | tokio-cron-scheduler |
| 18 | interval 调度器 | `src/scheduler/interval.rs` | interval | tick 事件 | tokio |
| 19 | 一次性调度器 | `src/scheduler/oneshot.rs` | at 时间 | tick 事件 | tokio |
| 20 | 失败重试 | `src/engine/retry.rs` | 失败 + 策略 | 重试次数 | 复用 shared-platform/retry |
| 21 | DLQ 处理器 | `src/engine/dlq.rs` | 失败超限 | DLQ event | 复用 shared-platform/dlq |
| 22 | 进度推送 | `src/engine/progress.rs` | 实时进度 | W-1 写 | actix-web (websocket v0.2) |
| 23 | 审计写入 | `src/audit/event.rs` | 事件 | T-3 写 | 复用 shared-platform/rbac |
| 24 | 数据迁移执行 | `src/migration/executor.rs` | 源 / 目标 | before snapshot + 迁移 | sqlx |
| 25 | Rollback 生成 | `src/migration/rollback.rs` | before snapshot | rollback SQL | sqlx |
| 26 | Log 源 | `src/log/source.rs` | 源配置 | log stream | tokio + sqlx |
| 27 | Log 过滤 | `src/log/filter.rs` | filter config | 过滤后 stream | 自写 |
| 28 | Log 聚合 | `src/log/aggregate.rs` | aggregate config | 聚合结果 | sqlx |
| 29 | 模板 repo | `src/template/repo.rs` | 模板 | 版本化 | sqlx |
| 30 | DB 连接池 | `src/db/pool.rs` | 配置 | sqlx pool | sqlx |
| 31 | env 凭据 | `src/config/env.rs` | env var | 配置 (不打印值) | 0 依赖 (per 8/27 11:06 JST 硬 ban) |

### 3.3 跨模块关键路径 (6 路径)

| 路径 | 模块序列 | 说明 |
|---|---|---|
| **GM 批量发奖** | page-gm-grant → API handler → task_def → worker_pool → sharder → 5 域 gRPC client → audit | 主要路径 |
| **定时任务** | cron tick → scheduler → task_def (复用) → 同 GM 路径 | 触发差异 |
| **Log 批量** | page-log-process → API handler → log/source → log/filter → log/aggregate → output | 独立路径 |
| **数据迁移** | page-data-migration → API handler → migration/executor → migration/rollback 生成 → audit | 含 rollback |
| **任务监控** | page-tasks → API handler → task_def (读) + task_execution (读) + sub_task (读) → 实时进度 | 读多写少 |
| **审计检索** | page-audit → API handler → audit/event (按索引读) | 读多写少 |

---

## 4. 关键流程

### 4.1 GM 批量发奖流程（详见 §1.3.1）

### 4.2 定时任务调度流程（详见 §1.3.2）

### 4.3 失败重试 + DLQ 流程

```
1. sub_task 失败:
   - 写 sub_task.error + finished_at
   - 写 audit_event T-3 (action=sub_task_fail, retry_count+1)
   ↓
2. retry 策略 (per worker_pool M-4):
   - 默认 3 次, 指数退避 100ms / 200ms / 400ms
   - 复用 shared-platform/retry 指数退避 (与 5 域统一)
   ↓
3. 重试 3 次仍失败:
   - 写 dlq_event T-4 (first_failed_at, retry_count=3, error)
   - 写 audit_event T-3 (action=dlq)
   - 任务继续 (其他 sub_task 不受影响)
   ↓
4. DLQ 人工干预:
   - rgs-batch-console /api/batch/dlq 列表 + 详情
   - 操作: 重试 (从 DLQ 重新入队) / 跳过 (标记 resolved) / 删除
   - 写 dlq_event.resolved_at + audit_event T-3
```

### 4.4 跨域调用 5 域 gRPC + mTLS 流程

```
1. rgs-batch-backend 启动:
   - 读 5 域 gRPC 证书路径 (env: GRPC_CERT_PATH_PLAYER / ECONOMY / MATCH / SOCIAL / ADMIN)
   - 加载 5 域 client cert + CA cert
   - 创建 tonic Channel per 域 (tls: rustls, ca: ..., identity: ...)
   - 5 域 gRPC client 实例
   ↓
2. 调用 (per task sub_task):
   - 5 域 gRPC client → channel → 5 域 svc
   - mTLS 双向认证 (per 5 域 ST 业务级 mTLS 实践 commit 401ac5c)
   - 失败 → 错误捕获 → retry 策略 (§4.3)
   ↓
3. 证书轮换 (v0.2 评估):
   - mavis cron 监控证书过期
   - 自动 reload (file watch)
```

### 4.5 异常流程 (10 类)

| 异常 | 处理 | 备注 |
|---|---|---|
| 5 域 gRPC 不可达 (network / cert / svc down) | retry 3 次 + DLQ | per §4.3 |
| PostgreSQL 不可达 | task 入队失败 → 返 500 + 写 audit_event | 不静默失败 |
| 任务定义 schema 错误 (cron 非法 / SQL 模板错) | API handler 返 400 + 写 audit_event | 启动时校验 |
| 任务超时 (默认 1h, per F-22) | worker kill + DLQ (v0.2 强制 kill) | v0.1 简化标记超时 (per REQ GAP-10) |
| 数据迁移源 / 目标 schema 不匹配 | dry-run 检测 (v0.2) | v0.1 简化 (per REQ GAP-9) |
| Log 源不可读 (file not found / kubectl 失败) | retry 3 次 + DLQ | per §4.3 |
| 调度器 crash | restart 后从 schedule M-5 恢复 (per next_run_at) | 不丢任务 |
| rgs-batch-backend crash | worker pool 丢失 → 任务状态 = 'unknown' → 重启后从 task_execution T-1 恢复 | 任务不丢 (per NFR-25) |
| 锁文件僵死 (mtime > 1h) | 启动时检测 + 删除 | per OLU-WEB §5.1.5 |
| env value 出现在日志 | 强校验: 写日志前 filter (per 8/27 11:06 JST 硬 ban) | 强制 |

---

## 5. 数据模型

### 5.1 rgs-batch-console 数据文件

```
tools/rgs-batch-console/
├── data/                           (新增, .gitignore 排除 jsonl 落本地)
│   ├── .gitignore                  "*.jsonl\n!.gitkeep"
│   ├── .gitkeep
│   ├── ai-ledger.jsonl             (Mavis session token 复用 OLU-WEB, per OLU-WEB §5.1.1)
│   └── .lock                       (原子锁, per OLU-WEB §5.1.5)
├── lib/
│   ├── lockfile.js                 (fs.openSync(path, 'wx') 原子创建)
│   └── token-estimate.js           (message_count × 5K, per OLU-WEB §2.1)
├── public/
│   ├── index.html                  (扩展 7 页面: gm-grant / schedule / log-process / data-migration / tasks / audit / settings)
│   └── app.js                      (UI 逻辑, vanilla JS + 原生 SVG)
└── server.js                       (启动 + 路由 + 9 API endpoint, per rgs-web 模式)
```

### 5.2 rgs-batch-backend 文件结构

```
tools/rgs-batch-backend/
├── Cargo.toml                      (依赖: actix-web 4 / tonic 0.12 / sqlx 0.7 / tokio-cron-scheduler 0.3 / serde / rustls)
├── migrations/                     (sqlx migrate, 19 文件对应 16 张表 + 3 索引)
│   ├── 0001_create_batch_master_schema.sql
│   ├── 0002_create_batch_transaction_schema.sql
│   ├── 0003_create_batch_work_schema.sql
│   ├── 0004_create_task_def.sql
│   ├── 0005_create_task_template.sql
│   ├── 0006_create_data_source.sql
│   ├── 0007_create_worker_pool.sql
│   ├── 0008_create_schedule.sql
│   ├── 0009_create_task_execution.sql
│   ├── 0010_create_sub_task.sql
│   ├── 0011_create_audit_event.sql
│   ├── 0012_create_dlq_event.sql
│   ├── 0013_create_log_event.sql
│   ├── 0014_create_data_migration.sql
│   ├── 0015_create_task_progress.sql
│   ├── 0016_create_task_buffer.sql
│   ├── 0017_create_audit_session.sql
│   ├── 0018_create_log_buffer.sql
│   └── 0019_create_migration_buffer.sql
├── src/
│   ├── main.rs                     (启动 + actix-web 路由)
│   ├── config/
│   │   ├── env.rs                  (env var 加载, 不打印值 per 8/27 11:06 JST)
│   │   └── db.rs                   (sqlx pool)
│   ├── api/                        (10 个, per §3.2)
│   ├── engine/                     (6 个: executor / worker_pool / sharder / rate_limiter / retry / dlq / progress)
│   ├── scheduler/                  (3 个: cron / interval / oneshot)
│   ├── grpc_clients/               (6 个: 5 域 + tls)
│   ├── audit/event.rs
│   ├── log/                        (3 个: source / filter / aggregate)
│   ├── migration/                  (2 个: executor / rollback)
│   ├── template/repo.rs
│   └── db/pool.rs
└── tests/                          (UT + IT, per REQUIREMENTS §8)
    ├── ut_executor.rs
    ├── ut_worker_pool.rs
    ├── ut_sharder.rs
    ├── ut_retry.rs
    ├── ut_dlq.rs
    ├── ut_progress.rs
    ├── ut_cron.rs
    ├── ut_migration_rollback.rs
    ├── it_grpc_client.rs
    ├── it_api_task_def.rs
    ├── it_api_schedule.rs
    ├── it_api_log_task.rs
    ├── it_api_migration.rs
    └── it_e2e_gm_grant.rs
```

### 5.3 PostgreSQL schema (per REQUIREMENTS §4 DB 三分类横展)

#### 5.3.1 batch_master schema (5 表)

```sql
-- M-1 task_def
CREATE TABLE batch_master.task_def (
  task_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_type VARCHAR(50) NOT NULL,
  cron_expr VARCHAR(100),
  target JSONB NOT NULL,
  params JSONB NOT NULL,
  owner VARCHAR(100) NOT NULL DEFAULT 'Ulysses',
  status VARCHAR(20) NOT NULL DEFAULT 'pending',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_task_def_owner ON batch_master.task_def (owner);
CREATE INDEX idx_task_def_status ON batch_master.task_def (status);

-- M-2 task_template
CREATE TABLE batch_master.task_template (
  template_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(200) NOT NULL,
  type VARCHAR(50) NOT NULL,
  sql_template TEXT,
  params_schema JSONB,
  version INT NOT NULL DEFAULT 1,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- M-3 data_source
CREATE TABLE batch_master.data_source (
  source_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  type VARCHAR(50) NOT NULL,  -- 'postgres' / 'grpc' / 'csv' / 'file'
  conn_str_ref VARCHAR(200) NOT NULL,  -- env var 名, 不存值 (per 8/27 11:06 JST 硬 ban)
  credentials_ref VARCHAR(200)  -- env var 名, 不存值
);

-- M-4 worker_pool
CREATE TABLE batch_master.worker_pool (
  pool_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  domain VARCHAR(50) NOT NULL,  -- 'player' / 'economy' / ...
  max_concurrent INT NOT NULL DEFAULT 5,
  rpm_limit INT NOT NULL DEFAULT 1000,
  enabled BOOLEAN NOT NULL DEFAULT true
);

-- M-5 schedule
CREATE TABLE batch_master.schedule (
  schedule_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id UUID NOT NULL REFERENCES batch_master.task_def(task_id),
  cron_expr VARCHAR(100),
  interval_seconds INT,
  at_time TIMESTAMPTZ,  -- 一次性
  next_run_at TIMESTAMPTZ,
  enabled BOOLEAN NOT NULL DEFAULT true
);
CREATE INDEX idx_schedule_next_run ON batch_master.schedule (next_run_at) WHERE enabled = true;
```

#### 5.3.2 batch_transaction schema (6 表)

```sql
-- T-1 task_execution
CREATE TABLE batch_transaction.task_execution (
  exec_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id UUID NOT NULL,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ,
  status VARCHAR(20) NOT NULL,  -- 'pending' / 'running' / 'completed' / 'failed' / 'partial'
  params_snapshot JSONB NOT NULL,
  result_summary JSONB,
  trace_id VARCHAR(64) NOT NULL
);
CREATE INDEX idx_task_execution_task_id ON batch_transaction.task_execution (task_id);
CREATE INDEX idx_task_execution_started_at ON batch_transaction.task_execution (started_at);

-- T-2 sub_task
CREATE TABLE batch_transaction.sub_task (
  sub_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  exec_id UUID NOT NULL REFERENCES batch_transaction.task_execution(exec_id),
  target_id VARCHAR(200) NOT NULL,  -- player_id / guild_id / etc.
  status VARCHAR(20) NOT NULL,
  retry_count INT NOT NULL DEFAULT 0,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ,
  error TEXT,
  result JSONB
);
CREATE INDEX idx_sub_task_exec_id ON batch_transaction.sub_task (exec_id);
CREATE INDEX idx_sub_task_target_id ON batch_transaction.sub_task (target_id);

-- T-3 audit_event
CREATE TABLE batch_transaction.audit_event (
  event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  exec_id UUID,
  operator VARCHAR(100) NOT NULL DEFAULT 'Ulysses',
  action VARCHAR(50) NOT NULL,  -- 'create' / 'complete' / 'fail' / 'dlq' / 'sub_task_fail' / 'retry'
  params_hash VARCHAR(64),  -- 脱敏后 hash (per 8/27 11:06 JST 硬 ban)
  result JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id VARCHAR(64) NOT NULL
);
CREATE INDEX idx_audit_event_exec_id ON batch_transaction.audit_event (exec_id);
CREATE INDEX idx_audit_event_created_at ON batch_transaction.audit_event (created_at);

-- T-4 dlq_event
CREATE TABLE batch_transaction.dlq_event (
  dlq_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  exec_id UUID NOT NULL,
  sub_id UUID,
  error TEXT NOT NULL,
  retry_count INT NOT NULL,
  first_failed_at TIMESTAMPTZ NOT NULL,
  last_retried_at TIMESTAMPTZ,
  resolved_at TIMESTAMPTZ
);
CREATE INDEX idx_dlq_event_exec_id ON batch_transaction.dlq_event (exec_id);

-- T-5 log_event
CREATE TABLE batch_transaction.log_event (
  event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source VARCHAR(200) NOT NULL,
  level VARCHAR(20) NOT NULL,
  message TEXT NOT NULL,
  fields JSONB,
  ts TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_log_event_source_ts ON batch_transaction.log_event (source, ts);
CREATE INDEX idx_log_event_ts ON batch_transaction.log_event (ts);

-- T-6 data_migration
CREATE TABLE batch_transaction.data_migration (
  migration_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source VARCHAR(200) NOT NULL,
  target VARCHAR(200) NOT NULL,
  before_snapshot JSONB,
  rollback_sql TEXT,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

#### 5.3.3 batch_work schema (5 表)

```sql
-- W-1 task_progress
CREATE TABLE batch_work.task_progress (
  exec_id UUID PRIMARY KEY,
  completed INT NOT NULL DEFAULT 0,
  failed INT NOT NULL DEFAULT 0,
  total INT NOT NULL,
  eta_seconds INT,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- W-2 task_buffer
CREATE TABLE batch_work.task_buffer (
  exec_id UUID NOT NULL,
  chunk_id INT NOT NULL,
  data JSONB,
  status VARCHAR(20) NOT NULL,
  PRIMARY KEY (exec_id, chunk_id)
);

-- W-3 audit_session
CREATE TABLE batch_work.audit_session (
  session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  operator VARCHAR(100) NOT NULL DEFAULT 'Ulysses',
  ip INET,
  started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  finished_at TIMESTAMPTZ
);

-- W-4 log_buffer
CREATE TABLE batch_work.log_buffer (
  session_id UUID NOT NULL,
  chunk_id INT NOT NULL,
  raw_data JSONB,
  status VARCHAR(20) NOT NULL,
  PRIMARY KEY (session_id, chunk_id)
);

-- W-5 migration_buffer
CREATE TABLE batch_work.migration_buffer (
  session_id UUID NOT NULL,
  chunk_id INT NOT NULL,
  source_data JSONB,
  target_status VARCHAR(20),
  PRIMARY KEY (session_id, chunk_id)
);
```

### 5.4 凭据 env var 命名约定 (per 8/27 11:06 JST 硬 ban, 永不打**印值**)

```
# 5 域 gRPC 证书 (mTLS 业务级, per 5 域 ST 实践 commit 401ac5c)
GRPC_CA_CERT_PATH=/path/to/ca.crt
GRPC_CLIENT_CERT_PATH=/path/to/client.crt
GRPC_CLIENT_KEY_PATH=/path/to/client.key
GRPC_CERT_PATH_PLAYER=/path/to/player-cert.crt
GRPC_CERT_PATH_ECONOMY=/path/to/economy-cert.crt
GRPC_CERT_PATH_MATCH=/path/to/match-cert.crt
GRPC_CERT_PATH_SOCIAL=/path/to/social-cert.crt
GRPC_CERT_PATH_ADMIN=/path/to/admin-cert.crt

# PostgreSQL
BATCH_DB_HOST=postgres
BATCH_DB_PORT=5432
BATCH_DB_USER=batch_user
BATCH_DB_PASSWORD=<env>      # NEVER print
BATCH_DB_NAME=rgs_batch

# Worker 池
BATCH_WORKER_POOL_SIZE=5
BATCH_LOG_LEVEL=info
BATCH_ENV=production

# 5 域 gRPC 端口 (k8s targetPort, per docs/deploy/01-k8s-manifests/)
BATCH_GRPC_PORT_PLAYER=50051
BATCH_GRPC_PORT_ECONOMY=50052
BATCH_GRPC_PORT_MATCH=50053
BATCH_GRPC_PORT_SOCIAL=50054
BATCH_GRPC_PORT_ADMIN=50055

# 监听地址
BATCH_BACKEND_BIND=0.0.0.0:8790
BATCH_CONSOLE_BIND=127.0.0.1:8789
```

### 5.5 数据保留策略 (per NFR-29)

| 类型 | 保留期 | 归档策略 |
|---|---|---|
| task_def (M-1) | 永久 | 不归档 |
| task_execution (T-1) | 90 天 | 90 天后归档到 `batch_transaction_archive.task_execution` |
| sub_task (T-2) | 90 天 | 90 天后归档 |
| audit_event (T-3) | 永久 | 不归档 (per NFR-29) |
| log_event (T-5) | 30 天 | 30 天后压缩到冷存储 (per NFR-29) |
| task_progress (W-1) | 任务完成后清理 | 任务结束即清理 |
| 其他 Work 表 | 任务结束后清理 | 任务结束即清理 |

---

## 6. 关联文档与演进

### 6.1 文档三件套

| 文档 | 状态 | 备注 |
|---|---|---|
| RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 | ✅ 已落地 (commit `fd122f6`) | 本文之上游 |
| **RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1** | **✅ 本文 (待 commit)** | 中游 |
| RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1 | ⏳ 待起草 | 下游 (API 签名 + 数据模型细节 + 部署 + 运维 + 安全) |
| RGS-BATCH-PLAN-2026-09-01 v0.1 | ⏳ 待起草 | 总览 + 实施计划 |

### 6.2 与上游规范关系 (5 不破坏)

- **不破坏 5 域架构**: rgs-batch-backend 作为 gRPC 客户端调用 5 域, **不修改** 5 域代码 (per 5 域独立 Lead 原则 + DEC-005)
- **不破坏 rgs-web**: rgs-batch-console 独立 Node 项目, **不嵌入** rgs-web (per "独立的项目" 用户要求)
- **不破坏 shared-platform**: 复用现有 crate, **不修改** shared-platform 代码
- **不破坏 function-plane**: v0.1 不集成 (saga-runtime 独立 Pod per RGS-BAS-100), **不修改** function-plane 代码
- **不破坏 gm-backend**: rgs-batch-console 跟 gm-console 形态不同 (gm-console 是 dist/ + envoy, rgs-batch-console 是 Node 全栈), 但都是 envoy 独立 deployment (per 9/1 13:03/13:05 JST)

### 6.3 演进路径 (3 阶段)

| 阶段 | 内容 | 预计时间 |
|---|---|---|
| **v0.1 (本文 + DETAILED + PLAN)** | 基础框架 + GM 批量 + 定时调度 + 失败重试 + DLQ + 任务监控 + 审计 + DB 16 表 + 9 异常处理 | 4-6 周 |
| **v0.2** | Log 批量 + 数据整理 + 跨 batch DAG + WebSocket + mavis cron 告警 + AI 协作 + rgs-web 深联动 + 证书轮换 + 任务超时强制 kill + dry-run | 6-10 周 |
| **v0.3** | 商业化 / 多租户 / 流式 / provider token counter / k3s HPA 自动扩缩 | 待 v0.2 评估 |

### 6.4 与 OLU-WEB 关系 (4 复用)

| OLU-WEB 模块 | rgs-batch 复用 |
|---|---|
| data/ 目录 + lockfile (per OLU-WEB §5.1.5) | ✅ rgs-batch-console/lib/lockfile.js 复用 |
| token-estimate.js (per OLU-WEB §2.1) | ✅ rgs-batch-console/lib/token-estimate.js 复用 |
| ai-ledger.jsonl (per OLU-WEB §5.1.1) | ✅ rgs-batch-console/data/ 复用 |
| data/.lock 原子锁 (per OLU-WEB §5.1.5) | ✅ rgs-batch-console/data/.lock 复用 |

---

## 7. 验收者

(per 2026-08-26 08:40 JST 代签新规则, 同 REQUIREMENTS §11)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | 架构师 (**Mavis 接手 agent per DEC-008**) | 2026-09-01 |
| batch 域 Lead | _待 DDD Review 阶段补签_ | — |
| 5 域 Lead (player / economy / match / social / admin) | _待 DDD Review 阶段补签_ | — |
| shared-platform Lead | _待 DDD Review 阶段补签_ | — |
| cluster-ops Lead | _待 DDD Review 阶段补签_ | — |
| SRE Lead | _待 DDD Review 阶段补签_ | — |
| DBA | _待 DDD Review 阶段补签_ | — |
| 安全 | _待 DDD Review 阶段补签_ | — |
| PM | _待 DDD Review 阶段补签_ | — |

---

## 8. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (**Mavis 接手 agent per DEC-008**) | 首版: 0 文档定位 + 1 架构总览 (4 子节: 整体位置 + 双项目架构 + 4 数据流 + 10 外部关系) + 2 技术选型 (13 console + 23 backend 决策 + 20 不选) + 3 模块划分 (11 console + 31 backend 模块 + 6 跨模块路径) + 4 关键流程 (5 流程 + 10 异常) + 5 数据模型 (3 数据文件 + 70 文件 + 16 schema 表 + 23 env var + 7 数据保留) + 6 关联演进 (4 文档 + 5 不破坏 + 3 演进 + 4 OLU-WEB 复用) + 7 验收者 + 8 修订历史 |

**修订人**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师 (**Mavis 接手 agent per DEC-008**)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化
