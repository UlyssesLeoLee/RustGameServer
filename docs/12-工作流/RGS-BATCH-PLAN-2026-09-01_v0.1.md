# RGS-BATCH-PLAN-2026-09-01 v0.1

**综合 Batch 管理平台设计总览 + 实施方案（rgs-batch-console + rgs-batch-backend）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BATCH-PLAN-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 18:00 JST "batch 平台" 决策 + 18:25 JST 范围澄清 + 18:34 JST Q2 拍板 + 19:00 JST 继续 DETAILED-DESIGN）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1（commit `fd122f6`）+ RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1（commit `e366ff8`）+ RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1（commit `62027c9`）已落地，本层补总览 + 实施计划 |
| 关联 | 上游 REQ v0.1（commit `fd122f6`）+ BASIC v0.1（commit `e366ff8`）+ DETAILED v0.1（commit `62027c9`）|
| 上游基线 | rgs-web v0.3 commit `625a3f0` + rgs-web OLU-WEB 4 文档 + gm-backend 范式 + 5 域 ST 业务级 mTLS 实践（commit `401ac5c`）+ 9/1 PT 派工 commit `ffbfb19` + saga-runtime 独立 Pod（per RGS-BAS-100 v0.1）+ OLU-WEB-PLAN v0.1 4 周落地范式 |
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 触发与背景

**触发（4 时间点）**：

| # | 时间 (JST) | 触发 | 内容 |
|---|---|---|---|
| 1 | 18:00 | Ulysses | "batch 需要一个专门的管理界面和对其支持的前后端功能，应该是一个独立的项目，但可以按照其他功能的方式融入架构，从需求文档开始设计" |
| 2 | 18:25 | Ulysses | "所有内容的批量，包括但不限于 log、数据整理"（范围澄清：综合 batch 平台）|
| 3 | 18:34 | Ulysses | Q2 拍板："独立双项目（对标 rgs-web + gm-backend，推荐）"（Node console + actix-web backend）|
| 4 | 19:00 | Ulysses | "继续 DETAILED-DESIGN（推荐）"（OLU-WEB 三层文档节奏）|

**现状（per 9/1 18:00 JST git 实证）**：

| 维度 | 现状 | 数据来源 |
|---|---|---|
| GM 批量操作 | 5 域 admin 单条接口为主，无批量入口 | `grep -i batch crates/admin-service/src` → 0 match（仅 OTel `install_batch` + `outbox.batch_size` 等内部用法）|
| 定时任务调度 | 无统一调度器，每个域自己手写 setInterval 或 nohup | `grep -r "cron\|scheduled" crates/` 仅 OTel/tracing 内部用法 |
| Log 批量处理 | 手工脚本 + PostgreSQL 直接查询 | `tools/db-seed/` + ad-hoc SQL（无平台化）|
| 数据整理 | 手工 SQL + Excel / CSV 临时导出 | ad-hoc，无版本控制无审计 |

**本文档目的**：6 周内落地 rgs-batch 综合平台，2 个独立项目（rgs-batch-console Node + rgs-batch-backend actix-web）+ 1 个 envoy 边缘代理 + 16 张 PostgreSQL 表 + 37 API endpoint + 38 WBS L4 任务，**不破坏** 5 域 / rgs-web / shared-platform / function-plane / gm-backend 现有架构（per BASIC §6.2 五不破坏）。

---

## 1. 设计目标（per REQUIREMENTS v0.1 §1.4 业务目标）

| # | 目标 | 度量 | 优先级 | 验收 |
|---|---|---|---|---|
| O-1 | **GM 批量操作可视化** | 选目标实体 × 选动作 × 预览 → 提交 → 实时进度 → 结果下载 | P0 | F-1 + F-3 + F-6 落地, 端到端 IT 通过 |
| O-2 | **定时任务统一调度** | cron / interval / 一次性 三种触发模式 + 7 天历史 | P0 | F-9 + NFR-22 落地, ST-03 通过 |
| O-3 | **Log 批量处理平台化** | 多源 log + 过滤 + 聚合 + 输出 | P0 | F-17 落地, 端到端 IT 通过 |
| O-4 | **数据整理任务化** | SQL 模板 + 迁移 / 转换 / 导入导出 + 模板复用 | P0 | F-18 + F-19 + F-24 落地, ST-06 通过 |
| O-5 | **任务监控 + 审计** | 实时进度 + 审计 log (操作人/时间/参数 hash/结果/trace_id) | P0 | F-7 + F-10 + F-26 落地, NFR-28 < 1s |
| O-6 | **失败重试 + DLQ** | 重试 3 次 + DLQ + 人工干预 | P0 | F-8 落地, NFR-26 retry 3 次, ST-04 通过 |
| O-7 | **跨 batch 编排**（v0.2 P2）| 多任务依赖 + 失败策略 | P2 | v0.2 评估 |
| O-8 | **DB 横展三分类落地** | Master / Transaction / Work 三分清晰 | P0 | §4 横展 16 张表 + GAP-12 全部归档 |
| O-9 | **5 域 gRPC 业务级集成** | rgs-batch-backend 内置 5 域 gRPC client, mTLS 业务级 | P0 | IR-1 + NFR-32 落地, ST-05 通过 |
| O-10 | **token 算 OLU（v0.2 集成）**| batch 任务自身如调外部 LLM, token 流自动入账 | P2 | v0.2 评估 (per OLU-WEB F-25) |

---

## 2. 6 周里程碑

> 6 周（4 + 2 缓冲）覆盖：基础框架 → 核心引擎 → 调度审计 → 监控迁移 → 集成测试 → 系统测试 + DDD Review。

### W1 (2026-09-02 ~ 09-08) — 基础框架

| 类别 | 验收 |
|---|---|
| (a) | rgs-batch-console Node 项目脚手架（tools/rgs-batch-console/，0 依赖 + 127.0.0.1:8789 监听）|
| (b) | rgs-batch-backend Rust 项目脚手架（tools/rgs-batch-backend/Cargo.toml + src/main.rs，actix-web 4 + tokio，0.0.0.0:8790 监听）|
| (c) | 9 个 k8s manifests（70-78：console / backend / envoy deployment + service + configmap + secret example + networkpolicy）|
| (d) | 5 域 ST 证书导出（per 8/27 ST 实践 commit 401ac5c）+ rgs-batch 自己证书生成（per crates/rgs-certgen）|
| (e) | PostgreSQL 3 schema 创建（batch_master / batch_transaction / batch_work + batch_transaction_archive）+ 19 migration 文件 |
| (f) | envoy 独立 deployment 配置（per 9/1 13:03/13:05 JST 偏好）|
| **NFR-OP-010** | 人·天轨: 10 / 1 周 = 10 ≤ 20 ✓; token 轨: 1.5M / 1 周 = 1.5M ≤ 20M ✓ |

### W2 (2026-09-09 ~ 09-15) — 核心引擎 (Master 5 表 + GM 批量 + 5 worker 池)

| 类别 | 验收 |
|---|---|
| (a) | Master 5 表（task_def / task_template / data_source / worker_pool / schedule）+ 索引（18 个, per DETAILED §2.2）|
| (b) | Transaction 前 3 表（task_execution / sub_task / audit_event）+ Work 前 3 表（task_progress / task_buffer / audit_session）|
| (c) | 5 域 gRPC client（player / economy / match / social / admin 5 个 .rs，tonic 0.12 + mTLS 业务级，per 5 域 ST 实践 commit 401ac5c）|
| (d) | worker_pool 5 worker + tokio mpsc channel + 限流（per worker_pool M-4）|
| (e) | 失败重试 3 次（指数退避 100/200/400ms，复用 shared-platform/retry.rs，per NFR-26）+ DLQ 处理器 |
| (f) | /api/v1/tasks（POST / GET / {id} / sub-tasks / progress / cancel 共 6 endpoint）|
| (g) | /api/v1/health + /metrics（Prometheus scrape 9464）|
| **DoD** | GM 批量 10 player 端到端 IT 通过（per IT-06）+ retry 3 次 + DLQ 1 场景通过 |
| **NFR-OP-010** | 人·天轨: 22 / 2 周 = 11 ≤ 20 ✓; token 轨: 3.6M / 2 周 = 1.8M ≤ 20M ✓ |

### W3 (2026-09-16 ~ 09-22) — 调度 + 审计 (Transaction 后 3 表 + Work 后 2 表 + 凭据 redact)

| 类别 | 验收 |
|---|---|
| (a) | Transaction 后 3 表（dlq_event / log_event / data_migration）+ Work 后 2 表（log_buffer / migration_buffer）|
| (b) | cron 调度器（tokio-cron-scheduler 0.3）+ interval 调度器（tokio::time::interval）+ 一次性调度器（tokio::time::sleep_until）|
| (c) | /api/v1/schedules CRUD（4 endpoint：POST / GET / {id} / PUT / DELETE）|
| (d) | /api/v1/audit + 强校验 redact filter（serde_json::Value 递归过滤 banned keys: password/key/token/secret/credential，per 8/27 11:06 JST 硬 ban + NFR-30）|
| (e) | 11 UT（executor / worker_pool / sharder / retry / dlq / progress / cron / migration_rollback / audit_filter / lockfile / token_estimate）|
| **DoD** | 定时任务 cron 表达式端到端 + 凭据泄露测试 0 命中（per ST-10）+ 11 UT 全过 |
| **NFR-OP-010** | 人·天轨: 32 / 3 周 = 10.7 ≤ 20 ✓; token 轨: 5.6M / 3 周 = 1.87M ≤ 20M ✓ |

### W4 (2026-09-23 ~ 09-29) — 监控 + 迁移 + UI (log/migration/templates/dlq + 7 页面)

| 类别 | 验收 |
|---|---|
| (a) | /api/v1/log-tasks + log/source + log/filter + log/aggregate（5 域 gRPC interceptor + 文件 glob + kubectl logs）|
| (b) | /api/v1/migration-tasks + migration/executor + migration/rollback（before snapshot + rollback SQL 生成, per F-24）|
| (c) | /api/v1/templates CRUD（4 endpoint：POST / GET / {id} / DELETE，per F-19 模板复用）|
| (d) | /api/v1/dlq CRUD（3 endpoint：GET / retry / resolve）|
| (e) | /api/v1/data-sources CRUD（3 endpoint：POST / GET / DELETE，per 8/27 11:06 JST 凭据只引用 env var 名）|
| (f) | /api/v1/worker-pools GET（per 5 域 RPM 配置可读）|
| (g) | rgs-batch-console 7 页面（gm-grant / schedule / log-process / data-migration / tasks / audit / settings, vanilla JS + 原生 SVG）|
| **DoD** | 7 页面渲染 + 8 IT（除 GM 批量 + DLQ 外）全过 + 数据迁移 + rollback SQL 端到端 |
| **NFR-OP-010** | 人·天轨: 44 / 4 周 = 11 ≤ 20 ✓; token 轨: 7.7M / 4 周 = 1.93M ≤ 20M ✓ |

### W5 (2026-09-30 ~ 10-06) — 集成测试 + 端到端 + 凭据 + OLU 报告

| 类别 | 验收 |
|---|---|
| (a) | 11 UT 全过（per W3 (e)，覆盖率 ≥ 80%）|
| (b) | 8 IT 全过（grpc_client / api_task_def / api_schedule / api_log_task / api_migration / e2e_gm_grant / dlq_recovery / audit_persistence）|
| (c) | GM 批量端到端 IT（100 player 真实 5 域发奖, 100% 成功）+ 1000 player 性能基准（per PF-5 ≤ 30s）|
| (d) | 凭据泄露测试 0 命中（响应 / 日志 / 错误信息搜索 token / password / key 全 0 命中，per ST-10）|
| (e) | OLU 报告（batch 任务自身如调外部 LLM 的 token 入账, v0.1 估算 + v0.2 真实值, per OLU-WEB F-25 + R-6）|
| **DoD** | 8 IT 全过 + 1000 player 性能达标 + 凭据泄露 0 命中 + OLU 报告落地 |

### W6 (2026-10-07 ~ 10-13) — 系统测试 + 监控 + 故障恢复 + DDD Review

| 类别 | 验收 |
|---|---|
| (a) | ST-01 ~ ST-05（k3s 部署 e2e + GM 真实 5 域 + 定时调度真实 + DLQ 真实 + mTLS 业务级, per DETAILED §7.3）|
| (b) | ST-06 ~ ST-10（数据迁移 + rollback + audit 永久保留 + envoy 边缘代理 + 127.0.0.1 only + env value 硬 ban）|
| (c) | 监控指标 10 项（per DETAILED §4.1）+ Prometheus scrape + 告警规则 |
| (d) | 故障恢复 10 场景（per DETAILED §4.2）+ 数据备份 7 项（per DETAILED §4.3）+ 8 运维脚本（per DETAILED §4.4）|
| (e) | DDD Review + 签字栏补签（架构师 + batch Lead + 5 域 Lead + 平台/集群/SRE/DBA/安全/PM 共 9 签字栏）|
| **DoD** | 10 ST 全过 + 监控/故障恢复/备份全部落地 + DDD Review 通过 + v0.1 升版 commit |

---

## 3. 实施分解（WBS 6 周 L4 任务）

> **本节是 v0.1 落地的实际 WBS L4 任务清单**，per RGS-WBS-001 v0.3 §2A.2 拆分原则（每个 L4 任务 = 1 人/agent 最小可拆分单位，≤ 2 人·天 或 ≤ 500K tokens）。

### 3.1 W1 任务（6 任务 / ~10 人·天 / ~1500K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| BA-W1-1 | rgs-batch-console Node 脚手架（server.js + public/index.html + app.js + lib/lockfile.js + lib/token-estimate.js, 0 依赖 + 127.0.0.1:8789）| 架构师 | 1.5 | 250K | 无 | `node server.js` 启动 < 1s + curl 200 OK | revert commit | `wbs/BA-W1-1` |
| BA-W1-2 | rgs-batch-backend Rust 脚手架（Cargo.toml + src/main.rs + 路由 + /api/v1/health, actix-web 4 + tokio, 0.0.0.0:8790）| 架构师 | 2.0 | 300K | 无 | `cargo run` 启动 < 3s + curl 200 OK | revert | `wbs/BA-W1-2` |
| BA-W1-3 | 9 个 k8s manifests（70-78：console / backend / envoy deployment + service + configmap + secret example + networkpolicy, per DETAILED §3.1-§3.6）| 架构师 | 2.0 | 300K | BA-W1-1, BA-W1-2 | kubectl apply 全过 + 3 pod 1/1 Running | revert | `wbs/BA-W1-3` |
| BA-W1-4 | 5 域 ST 证书导出（per 8/27 ST 实践 commit 401ac5c）+ rgs-batch 自己证书生成（per crates/rgs-certgen batch）| 架构师 | 1.5 | 250K | 无 | certs/ 5 域 + rgs-batch 8 文件齐 | revert | `wbs/BA-W1-4` |
| BA-W1-5 | PostgreSQL 3 schema 创建 + 19 migration（batch_master / batch_transaction / batch_work / batch_transaction_archive, per DETAILED §2.1）| 架构师 | 2.0 | 300K | 无 | `sqlx migrate run` 19 文件全过 + 16 张表齐 | revert | `wbs/BA-W1-5` |
| BA-W1-6 | envoy 独立 deployment 配置（per 9/1 13:03/13:05 JST 偏好, file_system HTTP filter + mTLS termination, per DETAILED §3.4）| 架构师 | 1.0 | 200K | BA-W1-3 | 2 replicas Running + curl https://localhost:8443 200 OK | revert | `wbs/BA-W1-6` |

### 3.2 W2 任务（7 任务 / ~12 人·天 / ~2100K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| BA-W2-1 | Master 5 表（task_def M-1 / task_template M-2 / data_source M-3 / worker_pool M-4 / schedule M-5, per DETAILED §2.3）+ 索引（5 索引）| 架构师 | 1.5 | 250K | BA-W1-5 | sqlx query 5 表全 OK + 5 索引生效 | revert | `wbs/BA-W2-1` |
| BA-W2-2 | Transaction 前 3 表（task_execution T-1 / sub_task T-2 / audit_event T-3, per DETAILED §2.3）+ Work 前 3 表（task_progress W-1 / task_buffer W-2 / audit_session W-3）| 架构师 | 1.5 | 250K | BA-W1-5 | sqlx query 6 表全 OK + 6 索引生效 | revert | `wbs/BA-W2-2` |
| BA-W2-3 | 5 域 gRPC client（player / economy / match / social / admin 5 个 .rs, tonic 0.12 + mTLS 业务级, per 5 域 ST 实践 commit 401ac5c）| 架构师 | 2.5 | 400K | BA-W1-4 | 5 域 gRPC 调用 + mTLS 双向认证通过（per IT-01）| revert | `wbs/BA-W2-3` |
| BA-W2-4 | worker_pool 5 worker + tokio mpsc channel + 限流（per worker_pool M-4, 5 域 RPM 可配）| 架构师 | 2.0 | 300K | BA-W2-1, BA-W2-3 | 5 worker 并发执行 1K 子任务 ≤ 30s（per PF-5）| revert | `wbs/BA-W2-4` |
| BA-W2-5 | 失败重试 3 次（指数退避 100/200/400ms, 复用 shared-platform/retry.rs, per NFR-26）+ DLQ 处理器（写 dlq_event T-4）| 架构师 | 1.5 | 250K | BA-W2-2, BA-W2-4 | 5 域 gRPC 失败 → retry 3 次 → DLQ（per IT-07）| revert | `wbs/BA-W2-5` |
| BA-W2-6 | /api/v1/tasks 6 endpoint（POST / GET / {id} / sub-tasks / progress / cancel, per DETAILED §1.2.1-§1.2.6）| 架构师 | 2.0 | 300K | BA-W2-1, BA-W2-2, BA-W2-4, BA-W2-5 | 6 endpoint 全过 + 端到端 IT-02 通过 | revert | `wbs/BA-W2-6` |
| BA-W2-7 | /api/v1/health + /metrics（Prometheus scrape 9464, per DETAILED §1.2.15-§1.2.16）| 架构师 | 1.0 | 200K | BA-W1-2 | 2 endpoint 全过 + Prometheus 5 指标可见 | revert | `wbs/BA-W2-7` |

### 3.3 W3 任务（8 任务 / ~10 人·天 / ~2000K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| BA-W3-1 | Transaction 后 3 表（dlq_event T-4 / log_event T-5 / data_migration T-6, per DETAILED §2.3）+ Work 后 2 表（log_buffer W-4 / migration_buffer W-5）| 架构师 | 1.5 | 250K | BA-W1-5 | sqlx query 5 表全 OK + 7 索引生效 | revert | `wbs/BA-W3-1` |
| BA-W3-2 | cron 调度器（tokio-cron-scheduler 0.3, per 3 种触发模式：cron / interval / oneshot）| 架构师 | 1.5 | 300K | BA-W2-1 | cron 表达式端到端 + 5 分钟 1 次自动执行（per ST-03）| revert | `wbs/BA-W3-2` |
| BA-W3-3 | interval 调度器（tokio::time::interval）+ 一次性调度器（tokio::time::sleep_until）| 架构师 | 1.0 | 200K | BA-W3-2 | 2 种调度器端到端（per IT-03）| revert | `wbs/BA-W3-3` |
| BA-W3-4 | /api/v1/schedules CRUD 4 endpoint（POST / GET / {id} / PUT / DELETE, per DETAILED §1.2.7）| 架构师 | 1.0 | 200K | BA-W3-2, BA-W3-3 | 4 endpoint 全过 + 启停生效 | revert | `wbs/BA-W3-4` |
| BA-W3-5 | /api/v1/audit + 强校验 redact filter（serde_json::Value 递归过滤 banned: password/key/token/secret/credential, per 8/27 11:06 JST 硬 ban + NFR-30）| 架构师 | 1.5 | 300K | BA-W2-2 | 凭据泄露 0 命中（per ST-10）+ 检索 100 条 < 1s（per NFR-28）| revert | `wbs/BA-W3-5` |
| BA-W3-6 | 11 UT 编写（executor / worker_pool / sharder / retry / dlq / progress / cron / migration_rollback / audit_filter / lockfile / token_estimate, per DETAILED §7.1）| 架构师 | 2.0 | 400K | BA-W3-1 ~ BA-W3-5 | `cargo test` 11 测试全过 + 覆盖率 ≥ 80% | revert | `wbs/BA-W3-6` |
| BA-W3-7 | 凭据 redact filter UT（per BA-W3-5）+ 凭据泄露测试脚本（per ST-10 + DETAILED §5.1）| 架构师 | 1.0 | 200K | BA-W3-5 | 0 命中（password / key / token / secret / credential 全过滤）| revert | `wbs/BA-W3-7` |
| BA-W3-8 | DLQ recovery IT（per IT-07：5 域 gRPC down + DLQ + 恢复, 复用 BA-W2-5 失败重试）| 架构师 | 0.5 | 150K | BA-W2-5, BA-W3-1 | DLQ 自动重试 + 5 域恢复后从 DLQ 重新入队 | revert | `wbs/BA-W3-8` |

### 3.4 W4 任务（7 任务 / ~12 人·天 / ~2100K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| BA-W4-1 | /api/v1/log-tasks + log/source + log/filter + log/aggregate（5 域 gRPC interceptor + 文件 glob + kubectl logs, per DETAILED §1.2.9）| 架构师 | 2.0 | 350K | BA-W3-1 | log 任务端到端（per IT-04）| revert | `wbs/BA-W4-1` |
| BA-W4-2 | /api/v1/migration-tasks + migration/executor + migration/rollback（before snapshot + rollback SQL 生成, per F-24 + DETAILED §1.2.10）| 架构师 | 2.5 | 400K | BA-W3-1 | 迁移 + rollback SQL 端到端（per ST-06）| revert | `wbs/BA-W4-2` |
| BA-W4-3 | /api/v1/templates CRUD 4 endpoint（POST / GET / {id} / DELETE, per F-19 模板复用 + DETAILED §1.2.8）| 架构师 | 1.0 | 200K | BA-W2-1 | 4 endpoint 全过 + 模板可保存 + 可复用 | revert | `wbs/BA-W4-3` |
| BA-W4-4 | /api/v1/dlq CRUD 3 endpoint（GET / retry / resolve, per DETAILED §1.2.12）| 架构师 | 1.0 | 200K | BA-W2-5, BA-W3-1 | 3 endpoint 全过 + DLQ retry 重新入队 + resolve 标记 | revert | `wbs/BA-W4-4` |
| BA-W4-5 | /api/v1/data-sources CRUD 3 endpoint（POST / GET / DELETE, conn_str_ref / credentials_ref 只存 env var 名, per 8/27 11:06 JST 硬 ban + DETAILED §1.2.14）| 架构师 | 1.0 | 200K | BA-W2-1 | 3 endpoint 全过 + conn_str 永不打**印值** | revert | `wbs/BA-W4-5` |
| BA-W4-6 | /api/v1/worker-pools GET（per DETAILED §1.2.13, 5 域 RPM 配置可读）| 架构师 | 0.5 | 150K | BA-W2-1 | 1 endpoint 全过 + 5 域 pool 配置可见 | revert | `wbs/BA-W4-6` |
| BA-W4-7 | rgs-batch-console 7 页面（gm-grant / schedule / log-process / data-migration / tasks / audit / settings, vanilla JS + 原生 SVG, per DETAILED §1.1）| 架构师 | 4.0 | 600K | BA-W1-1, BA-W2-6, BA-W3-4, BA-W4-1 ~ BA-W4-6 | 7 页面渲染 < 3s + 7 页面 API 集成 | revert | `wbs/BA-W4-7` |

### 3.5 W5 任务（5 任务 / ~5 人·天 / ~1000K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| BA-W5-1 | 11 UT 收尾 + 覆盖率统计（per BA-W3-6 持续, 补漏 + coverage 报告）| 架构师 | 1.0 | 200K | BA-W3-6 | `cargo test` 11 测试全过 + coverage ≥ 80% + tarpaulin 报告 | revert | `wbs/BA-W5-1` |
| BA-W5-2 | 8 IT 编写（per DETAILED §7.2, 除 GM 批量 + DLQ 外补 log + migration + audit persistence）| 架构师 | 2.0 | 350K | BA-W3-6, BA-W4-1, BA-W4-2 | 8 IT 全过 | revert | `wbs/BA-W5-2` |
| BA-W5-3 | GM 批量端到端 IT 100 player 真实 5 域发奖（per IT-06 + 性能基准 PF-5）| 架构师 | 0.5 | 150K | BA-W2-6, BA-W4-7 | 100 player 100% 成功 + 1000 player ≤ 30s | revert | `wbs/BA-W5-3` |
| BA-W5-4 | 凭据泄露测试 0 命中（响应 / 日志 / 错误信息全搜索, per ST-10 + DETAILED §5.1 强校验代码）| 架构师 | 0.5 | 150K | BA-W3-5, BA-W3-7 | 0 命中（GITHUB_TOKEN / Bearer / password / key / token / secret / credential）| revert | `wbs/BA-W5-4` |
| BA-W5-5 | OLU 报告落地（v0.1 估算 message_count × 5K + v0.2 真实值, per OLU-WEB F-25 + R-6）| 架构师 | 1.0 | 200K | BA-W4-7 | OLU 报告生成 + 仪表盘可见 | revert | `wbs/BA-W5-5` |

### 3.6 W6 任务（5 任务 / ~5 人·天 / ~1000K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| BA-W6-1 | ST-01 ~ ST-05（k3s 部署 e2e + GM 真实 5 域 + 定时调度真实 + DLQ 真实 + mTLS 业务级, per DETAILED §7.3）| 架构师 | 1.5 | 300K | W1-W5 全过 | 5 ST 全过 + 10 pod 1/1 Running | revert | `wbs/BA-W6-1` |
| BA-W6-2 | ST-06 ~ ST-10（数据迁移 + rollback + audit 永久保留 + envoy 边缘代理 + 127.0.0.1 only + env value 硬 ban）| 架构师 | 1.0 | 200K | BA-W6-1 | 5 ST 全过 + 凭据 0 命中 | revert | `wbs/BA-W6-2` |
| BA-W6-3 | 监控指标 10 项（per DETAILED §4.1）+ Prometheus scrape + 告警规则 | 架构师 | 1.0 | 200K | BA-W2-7 | 10 指标可见 + 3 告警规则生效 | revert | `wbs/BA-W6-3` |
| BA-W6-4 | 故障恢复 10 场景（per DETAILED §4.2）+ 数据备份 7 项（per DETAILED §4.3）+ 8 运维脚本（per DETAILED §4.4）| 架构师 | 1.0 | 200K | BA-W6-3 | 10 场景 SOP + 7 备份落地 + 8 脚本可执行 | revert | `wbs/BA-W6-4` |
| BA-W6-5 | DDD Review + 签字栏补签（架构师 + batch Lead + 5 域 Lead + 平台/集群/SRE/DBA/安全/PM 共 9 签字栏）| 架构师 | 0.5 | 100K | W1-W6 全过 | 4 文档签字栏就位 + v0.1 升版 commit | revert | `wbs/BA-W6-5` |

### 3.7 总工作量

| 周 | L4 任务数 | 人·天 | token/周（v0.5 算法 200K/人·天）| token/周（v0.6 算法 100K-300K/人·天）|
|---|---:|---:|---:|---:|
| W1 | 6 | 10.0 | 1500K | 1000K-3000K |
| W2 | 7 | 12.0 | 2100K | 1200K-3600K |
| W3 | 8 | 10.0 | 2000K | 1000K-3000K |
| W4 | 7 | 12.0 | 2100K | 1200K-3600K |
| W5 | 5 | 5.0 | 1050K | 500K-1500K |
| W6 | 5 | 5.0 | 1000K | 500K-1500K |
| **合计** | **38** | **54.0** | **9650K (9.65M)** | **4400K-15600K (4.4M-15.6M)** |

> **NFR-OP-010 双轨校验**（per RGS-TS-001 v0.7 §6.2.4 + RGS-OLU-REPORT-2026-08-27 v0.1 §6）：
> - 人·天轨：54 人·天 / 6 周 = 9 人·天/周 ≤ 20 ✓ 绿
> - token 轨：9.65M / 6 周 = 1.6M tokens/周 ≤ 20M ✓ 绿
> - 留足余量（v0.6 算法下界 4.4M / 6 周 = 733K tokens/周 = 3.7% NFR 上限）

---

## 4. 技术选型（per BASIC-DESIGN v0.1 §2）

> 完整 13 console + 23 backend 决策 + 20 不选方案 详见 BASIC-DESIGN §2。本节列关键 5 决策。

| # | 维度 | 选择 | 决策依据 |
|---|---|---|---|
| 1 | 后端框架 | actix-web 4 | 跟 gm-backend 同栈 (per 8/27 ST 实践) |
| 2 | gRPC client | tonic 0.12 | 5 域统一 (per RGS-INC-002) |
| 3 | DB 客户端 | sqlx 0.7 (async + compile-time check) | 5 域统一 + async 优势 |
| 4 | 调度 | tokio-cron-scheduler 0.3 | cron / interval / 一次性 三种触发模式 |
| 5 | 反向代理 | envoy 独立 deployment | per 2026-09-01 13:03 / 13:05 JST 偏好 |

**不选 5 项**（per BASIC §2.3）：
- ❌ Express / Koa / Fastify（npm 依赖，违反 0 依赖约束）
- ❌ React / Vue / Svelte（vanilla JS 够用）
- ❌ chart.js / d3（SVG 手写足够）
- ❌ axum / warp（跟 gm-backend 不一致）
- ❌ Kafka / NATS 流式（v0.1 简单 worker 池够用）

---

## 5. 资源 + RACI

### 5.1 资源分配

| 资源 | 角色 | 投入 | 备注 |
|---|---|---|---|
| **batch 域 Lead** | Ulysses（一人公司 12 角色 per DEC-008）| 100% 6 周 | 新增独立 Lead, 5 域扩展为 6 域 (per 8/21 JST 拒绝兼任) |
| **架构师** | Mavis 接手 agent per DEC-008 | 100% 6 周 | 设计 + 实现 + 自审 + commit + 代签 |
| **5 域 Lead 协调** | Ulysses（player / economy / match / social / admin）| ~10% 6 周 | 5 域 gRPC 接口协调 + 限流 RPM 拍板 |
| **shared-platform Lead** | Ulysses | ~5% 6 周 | outbox / retry / dlq / rbac / tls 复用协调 |
| **SRE Lead** | Ulysses | ~5% 6 周 | k8s deployment + envoy + cert + 监控告警 |
| **DBA** | Ulysses | ~5% 6 周 | PostgreSQL schema + 索引 + 归档 cron |
| **安全** | Ulysses | ~5% 6 周 | 凭据管理 + 强校验 + mTLS 业务级 |
| **PM** | Ulysses | ~5% 6 周 | WBS 跟踪 + 进度盘点 + DDD Review 协调 |

**总投入**：1.4 人·周 / 周（1 主 + 0.4 协调），符合 1 人 12 角色 + AI 协作模式（per RGS-TS-001 v0.4 §6.2 + user_profile token-OLU 框架）。

### 5.2 RACI（per 1 人 12 角色 + AI 协作）

| 任务 | R (Responsible) | A (Accountable) | C (Consulted) | I (Informed) |
|---|---|---|---|---|
| REQ / BASIC / DETAILED / PLAN 起草 | 架构师 (Mavis) | 架构师 (Mavis) | 5 域 Lead, SRE, DBA, 安全 | PM |
| 代码实现 + 自审 + commit | 架构师 (Mavis) | 架构师 (Mavis) | batch Lead, 5 域 Lead | SRE, PM |
| k3s 部署 + envoy + cert | SRE | SRE Lead | 架构师, 5 域 ST 实践 commit 401ac5c | DBA, 安全 |
| PostgreSQL schema + 索引 + 归档 | DBA | DBA | 架构师, SRE | PM |
| 凭据管理 + 强校验 + mTLS | 安全 | 安全 | 架构师, SRE | 5 域 Lead |
| mavis hook 集成 (v0.2) | 架构师 | batch Lead | Mavis skill | PM |
| DDD Review + 签字栏 | Ulysses | 架构师 | 5 域 Lead + 平台/集群/SRE/DBA/安全/PM | PM |

**5 域独立 Lead 原则**（per 8/21 JST 拒绝兼任 + DEC-005）：batch 域**不与** 5 域 Lead 兼任，扩展为 6 域。RACI 表格中 5 域 Lead 仅作 **C**（咨询），不兼任 **R / A**。

---

## 6. token 估算（per RGS-TS-001 v0.7 §6.2.2.1）

### 6.1 v0.5 算法（200K tokens/人·天 中位数）

| 周 | 人·天 | token（v0.5）| token/周 |
|---|---:|---:|---:|
| W1 | 10.0 | 2000K | 2000K |
| W2 | 12.0 | 2400K | 2400K |
| W3 | 10.0 | 2000K | 2000K |
| W4 | 12.0 | 2400K | 2400K |
| W5 | 5.0 | 1000K | 1000K |
| W6 | 5.0 | 1000K | 1000K |
| **合计** | **54.0** | **10800K (10.8M)** | **1800K/周** |

### 6.2 v0.6 双轨算法（100K-300K tokens/人·天）

| 周 | 人·天 | token（v0.6 下界 100K/人·天）| token（v0.6 上界 300K/人·天）|
|---|---:|---:|---:|
| W1 | 10.0 | 1000K | 3000K |
| W2 | 12.0 | 1200K | 3600K |
| W3 | 10.0 | 1000K | 3000K |
| W4 | 12.0 | 1200K | 3600K |
| W5 | 5.0 | 500K | 1500K |
| W6 | 5.0 | 500K | 1500K |
| **合计** | **54.0** | **5400K (5.4M)** | **16200K (16.2M)** |

### 6.3 NFR-OP-010 双轨校验

| 轨 | v0.5 | v0.6 下界 | v0.6 上界 | NFR-OP-010 上限 | 状态 |
|---|---:|---:|---:|---:|---|
| 人·天/周 | 9.0 | 9.0 | 9.0 | 20 | ✓ 绿 |
| tokens/周 | 1800K | 900K | 2700K | 20M (20000K) | ✓ 绿 (留足余量) |

**NFR-OP-010 双轨绿**：v0.6 上界 2.7M tokens/周 = 13.5% NFR-OP-010 上限（20M tokens/周），留足 86.5% 余量给 DDD Review + 集成测试 + 应急。

### 6.4 vs RGS-OLU-REPORT-2026-08-27 实战对比

- 9/1 实战 PT 派工 8 worker 25 min 全交付（per AGENTS.md v0.3 §6.3 + commit `ffbfb19`）= ~100K tokens / 8 worker ≈ 12.5K tokens / worker
- 1 L4 任务平均 200-400K tokens（per OLU-WEB-PLAN v0.1 实测）
- batch v0.1 总 9.65M tokens ≈ 48 个 OLU-WEB L4 任务 等价

---

## 7. 风险与缓解

| # | 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|---|
| R-1 | 5 域 gRPC 业务级 mTLS 证书复用失败 | 中 | 后端启动失败, 任务无法执行 | 8/27 ST 导出 SOP 复用, 一次性 kubectl get secret × 5 + cargo run -p rgs-certgen -- batch (per W1-4) |
| R-2 | batch 域 Lead 兼任 5 域 Lead 风险 | 中 | 责任矩阵模糊 | per 8/21 JST 拒绝兼任, 扩展为 6 域 + 申请额外 Lead 编制 (per REQ R-5) |
| R-3 | 5 域 gRPC 接口不支持 batch 分片 | 高 | 限流 + 错误聚合难 | worker_pool M-4 RPM 配置 + sharder 分片 (per W2-4) |
| R-4 | 长跑任务资源消耗 (夜间结算 1h+) | 中 | worker 池满 | k3s resource limit (NFR-33 1GB 内存 / 4 CPU 核) + v0.2 HPA 自动扩缩 |
| R-5 | 任务撤销原子性 | 中 | 部分子任务已执行难撤销 | 仅未执行 + 未生效可撤销 (per F-21) |
| R-6 | DB 横展落地成本 (16 张表 + 19 migration) | 中 | 设计 + 实施成本高 | per 9/1 18:30 JST 原则强制 (不妥协) + W1-5 / W2-1-2 / W3-1 分 3 阶段落地 |
| R-7 | AI 协作 token 估算不准 (v0.1 估算 message_count × 5K) | 中 | OLU 仪表盘读数误导 | UI 标 "estimated" (per NFR-27), v0.2 切真实值 (per OLU-WEB F-25) |
| R-8 | mavis runtime hook 集成阻塞 (v0.2) | 中 | OLU 自动入账失效 | v0.1 不集成 (per W5-5 降级估算), v0.2 评估 |
| R-9 | 凭据泄露 (per 8/27 11:06 JST 硬 ban) | 高 | secret 暴露 | 强校验代码 (per DETAILED §5.1) + 0 命中测试 (per W5-4) + ST-10 |
| R-10 | user_profile 127.0.0.1 only 硬约束 | 低 | 浏览器无法直连 rgs-batch-backend | envoy 独立 deployment + ClusterIP service (per 9/1 13:03/13:05 JST) |
| R-11 | 决策点 Ulysses 不在场 (per 9/1 14:58 JST 拍板必须用选项) | 中 | 卡进度 | 关键决策点 (W1 / W2 末 / W3 末 / W4 末 / W5 末 / W6 末) 各用 ask_user 一次 |
| R-12 | worker 派工后 cargo check 超时 (per AGENTS.md §2.1 L1+L2 合并) | 中 | worker 失败 | W1-W6 任务全部主会话自执行 (per AGENTS.md §2.4 L4), 不派 worker |
| R-13 | k3s 资源争用 (5 域 + batch + rgs-web + gm-backend 共享) | 中 | pod 资源不足 | HPA + resource limit (NFR-33) + namespace 隔离 |

---

## 8. 跨会话恢复 SOP

> per RGS-WT-001 §11.3 跨会话恢复 + AGENTS.md §2.4 L4 主会话打头阵。

**W1-W6 任务中断恢复**：
1. `git worktree list` 查 BA-W*-* worktree 状态
2. 读 `.wbs-task-marker` 找当前 status
3. 继续推进，调 `wbs_task_progress.ps1 -Status progress -Progress N` 同步

**6 周主会话打头阵原则**（per AGENTS.md §2.4 L4）：
- W1-W6 全部 38 任务主会话自执行，**不**派 worker（per AGENTS.md §2.4 L4 + R-12）
- 单任务执行超过 60s 仍无进展，回退到 WBS 状态 = blocked + 上报 Ulysses
- 关键决策点 6 次 ask_user（per 9/1 14:58 JST + R-11）

---

## 9. 启动 SOP

### 9.1 v0.1 启动

```bash
# 1. (一次性) 创建 worktree 主分支
git worktree add -b wbs/rgs-batch-v0.1 D:/.worktrees/rgs-batch-v0.1 main

# 2. (一次性) 配 5 域 ST 证书导出 (per 8/27 ST 实践 commit 401ac5c + W1-4)
kubectl get secret player-tls -n rust-game-server -o yaml > certs/player-tls.yaml
kubectl get secret economy-tls -n rust-game-server -o yaml > certs/economy-tls.yaml
kubectl get secret match-tls -n rust-game-server -o yaml > certs/match-tls.yaml
kubectl get secret social-tls -n rust-game-server -o yaml > certs/social-tls.yaml
kubectl get secret admin-tls -n rust-game-server -o yaml > certs/admin-tls.yaml

# 3. (一次性) rgs-batch 自己证书 (per crates/rgs-certgen)
cargo run -p rgs-certgen -- batch
# 输出 certs/rgs-batch-tls.yaml (CA + client cert + 5 域 cert + tls cert)

# 4. (一次性) PostgreSQL schema 创建 (per W1-5)
psql -h postgres -U batch_user -d rgs_batch -c "CREATE SCHEMA IF NOT EXISTS batch_master; CREATE SCHEMA IF NOT EXISTS batch_transaction; CREATE SCHEMA IF NOT EXISTS batch_work; CREATE SCHEMA IF NOT EXISTS batch_transaction_archive;"

# 5. (一次性) rgs-batch-backend DB migration (per W1-5)
cd tools/rgs-batch-backend
sqlx migrate run  # 19 文件按顺序执行 (0001-0019)

# 6. (一次性) 配 env var (per 2026-08-27 11:06 JST env value hard ban, 不 echo)
# 注入到 deployment yaml 的 secretKeyRef
kubectl create secret generic rgs-batch-db-secret \
  --from-literal=username=batch_user \
  --from-literal=password=<from-1password> \
  -n rust-game-server

kubectl create secret generic rgs-batch-grpc-certs \
  --from-file=ca.crt=./certs/ca.crt \
  --from-file=client.crt=./certs/client.crt \
  --from-file=client.key=./certs/client.key \
  --from-file=player.crt=./certs/player.crt \
  --from-file=economy.crt=./certs/economy.crt \
  --from-file=match.crt=./certs/match.crt \
  --from-file=social.crt=./certs/social.crt \
  --from-file=admin.crt=./certs/admin.crt \
  -n rust-game-server

# 7. (一次性) 应用 9 个 manifests
kubectl apply -f docs/deploy/01-k8s-manifests/70-rgs-batch-console-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/71-rgs-batch-console-service.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/72-rgs-batch-backend-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/73-rgs-batch-backend-service.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/74-rgs-batch-envoy-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/75-rgs-batch-envoy-service.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/76-rgs-batch-configmap.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/77-rgs-batch-secret.yaml.example
kubectl apply -f docs/deploy/01-k8s-manifests/78-rgs-batch-networkpolicy.yaml

# 8. 验证
kubectl get pods -l 'app.kubernetes.io/part-of=rust-game-server,app.kubernetes.io/component in (batch-ui, batch-engine, batch-edge)' -n rust-game-server
# 期望: rgs-batch-console 1/1 + rgs-batch-backend 1/1 + rgs-batch-envoy 2/2

# 9. (本地 dev) port-forward 8789
kubectl port-forward svc/rgs-batch-envoy 8789:8443 -n rust-game-server

# 10. 浏览器访问
# http://127.0.0.1:8789
```

### 9.2 验证清单（W6 DDD Review 时逐项过）

- [ ] W1：6 pod 1/1 Running + 16 张表齐 + 5 域证书齐
- [ ] W2：6 endpoint 全过 + 1K 子任务 ≤ 30s + DLQ 1 场景
- [ ] W3：cron / interval / 一次性 3 调度器全过 + 11 UT 全过 + 凭据泄露 0 命中
- [ ] W4：7 页面渲染 + log/migration/templates/dlq/data-sources/worker-pools 16 endpoint 全过
- [ ] W5：8 IT 全过 + 1000 player 性能达标 + 凭据泄露 0 命中 + OLU 报告落地
- [ ] W6：10 ST 全过 + 监控 10 指标 + 故障恢复 10 场景 + 数据备份 7 项 + 8 运维脚本 + DDD Review 签字栏

---

## 10. 验收者（per 2026-08-26 08:40 JST 代签新规则）

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | 架构师（**Mavis 接手 agent per DEC-008**）| 2026-09-01 |
| batch 域 Lead | _待 DDD Review 阶段补签_ | — |
| 5 域 Lead (player / economy / match / social / admin) | _待 DDD Review 阶段补签_ | — |
| shared-platform Lead | _待 DDD Review 阶段补签_ | — |
| cluster-ops Lead | _待 DDD Review 阶段补签_ | — |
| SRE Lead | _待 DDD Review 阶段补签_ | — |
| DBA | _待 DDD Review 阶段补签_ | — |
| 安全 | _待 DDD Review 阶段补签_ | — |
| PM | _待 DDD Review 阶段补签_ | — |

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师（**Mavis 接手 agent per DEC-008**）| 首版：6 周里程碑 + 38 L4 任务（W1 6 + W2 7 + W3 8 + W4 7 + W5 5 + W6 5）+ 5 技术选型 + 5 不选 + 7 RACI 任务 + 9.65M tokens 估算 + 13 风险 + 跨会话恢复 + 9 步启动 SOP + NFR-OP-010 双轨校验 (人·天 9/周 ✓ 绿 + token 1.6M/周 ✓ 绿 13.5% NFR 上限) |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态：仓库无 batch 平台
- v0.1 新增：本文档 + 3 配套文档
- v0.1 6 周落地：38 L4 任务 / 54 人·天 / 9.65M tokens

### A.2 文档四件套完成（per OLU-WEB 范式）

- ✅ RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1（commit `fd122f6`，436 行 / 39.5 KB）
- ✅ RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1（commit `e366ff8`，674 行 / 37.5 KB）
- ✅ RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1（commit `62027c9`，1053 行 / 49.7 KB）
- ✅ **RGS-BATCH-PLAN-2026-09-01 v0.1（设计总览 + 实施计划，本文）**

### A.3 已知缺口（per 2026-09-01 18:30 JST 缺标比错标原则）

- 5 域 Lead / shared-platform / cluster-ops / SRE / DBA / 安全 / PM 签字未到（DDD Review 阶段补）
- mavis runtime hook 集成未确认（v0.1 降级估算, v0.2 评估 per R-8）
- 5 域 gRPC v0.1 协议在 batch 场景下的分片 / 限流 / 错误聚合深度未确认（per R-3）
- RGS-ENV-CALIB-001 校准数据未生成（per RGS-OLU-REPORT-2026-08-27 v0.1 §10 GAP-1/2/3 + R-7）
- 6 周落地主会话自执行，不派 worker（per AGENTS.md §2.4 L4 + R-12）
- 关键决策点 6 次 ask_user（per 9/1 14:58 JST 拍板必须用选项 + R-11）
- batch 域 Lead RACI 同步（RACI v1.2 扩展 5 域 → 6 域, per REQ GAP-12）
- v0.2 待评估：Log 批量深度 + 数据迁移 + DAG + WebSocket + mavis cron 告警 + AI 协作 + rgs-web 深联动 + 证书轮换 + 任务超时强制 kill + dry-run（per REQ GAP-1 ~ GAP-10）
- k3s 资源上限 + namespace 隔离策略未确认（per REQ §10.3 待协调）
- 5 域 binary 未来调外部 LLM 未登记（v0.1 不集成, v0.2 评估 per OLU-WEB F-25 + R-7）

### A.4 引用链与证据

- rgs-web 母规范 5 份（per `docs/12-工作流/RGS-WEB-*.md`）
- rgs-web OLU-WEB 4 文档（per `docs/12-工作流/RGS-OLU-WEB-*.md`）
- rgs-web v0.3 commit `625a3f0`（merge 5 域 gRPC + 6 API + http2 + mTLS + port-forward, per 8/26 22:47 JST）
- gm-backend 范式（per `crates/gm-backend/` + RGS-IMPL-PLAN-ADMIN-001 + 8/27 ST 业务级 mTLS 实践 commit `401ac5c`）
- 5 域 gRPC 协议（per `docs/12-工作流/RGS-INC-002_5域gRPC真实跑通事件复盘_2026-08-26_v0.1.md`）
- shared-platform 20 模块（per `crates/shared-platform/`, per 9/1 PT 派工 commit `ffbfb19`）
- saga-runtime 独立 Pod（per `docs/01-核心架构与设计模式/RGS-BAS-100_Saga事务系统基本设计书_v0.1.md`）
- RGS-TS-001 v0.7 §6.2 OLU 双轨制
- RGS-WBS-001 v0.3 §2A L4 任务拆分原则
- RGS-OLU-REPORT-2026-08-27 v0.1 §3 token 估算公式 + §6 双轨校准
- per DEC-008 一人公司 12 角色
- per 2026-08-26 08:40 JST Mavis 默认代签 Ulysses
- per 2026-08-27 11:06 JST env value hard ban
- per 2026-09-01 13:03 / 13:05 JST envoy 独立 deployment 偏好
- per 2026-09-01 14:58 JST 拍板决策必须用选项
- per 2026-09-01 18:30 JST DB 横展三分类 + 缺标比错标
- per AGENTS.md §2.1 L1+L2 cargo check --tests 验证
- per AGENTS.md §2.3 L3 跨工具链决策前先 grep
- per AGENTS.md §2.4 L4 跨多工具链场景先主会话打头阵
- per AGENTS.md §2.5 L5 ST worktree 启动 checklist
- per AGENTS.md v0.3 §6.3 L11/L12 cargo build dir lock 防御 + 临时 log 不入 commit 防御

---

## 附：v0.1 文档四件套完整索引

| 文档 | 路径 | commit | 行数 | 大小 | 角色 |
|---|---|---|---|---|---|
| REQUIREMENTS v0.1 | `docs/12-工作流/RGS-BATCH-REQUIREMENTS-2026-09-01_v0.1.md` | `fd122f6` | 436 | 39.5 KB | 需求规约层 (What + Why) |
| BASIC-DESIGN v0.1 | `docs/12-工作流/RGS-BATCH-BASIC-DESIGN-2026-09-01_v0.1.md` | `e366ff8` | 674 | 37.5 KB | 基本设计层 (How 概要) |
| DETAILED-DESIGN v0.1 | `docs/12-工作流/RGS-BATCH-DETAILED-DESIGN-2026-09-01_v0.1.md` | `62027c9` | 1053 | 49.7 KB | 详细设计层 (How 细节) |
| **PLAN v0.1** | **`docs/12-工作流/RGS-BATCH-PLAN-2026-09-01_v0.1.md`** | **本文 (待 commit)** | **~600** | **~30 KB** | **总览 + 实施计划 (When + Who)** |

**总规模**：4 文档 / ~2700 行 / ~157 KB / 38 L4 任务 / 6 周落地 / 9.65M tokens 估算。
