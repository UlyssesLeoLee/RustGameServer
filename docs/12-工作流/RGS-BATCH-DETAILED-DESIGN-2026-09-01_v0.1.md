# RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1

**综合 Batch 管理平台详细设计（rgs-batch-console + rgs-batch-backend）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BATCH-DETAILED-DESIGN-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 18:00 JST "batch 平台" 决策 + 18:34 JST Q2 "独立双项目" 拍板）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1（commit `fd122f6`）+ RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1（commit `e366ff8`）已落地，本层补详细设计 |
| 关联 | 上游 RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1（commit `fd122f6`）+ RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1（commit `e366ff8`）+ 下游 RGS-BATCH-PLAN-2026-09-01 v0.1（待起草）|
| 上游基线 | rgs-web v0.3 commit `625a3f0`（merge 5 域 gRPC + 6 API + http2 + mTLS + port-forward, per 8/26 22:47 JST）+ rgs-web OLU-WEB 4 文档 + gm-backend 范式 + 5 域 ST 业务级 mTLS 实践（commit `401ac5c`）+ 9/1 PT 派工 commit `ffbfb19` + saga-runtime 独立 Pod（per RGS-BAS-100 v0.1）|
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 文档定位

本文档是 rgs-batch 综合平台**详细设计层**，回答"How 细节"——不涉及"What + Why"（已 in REQUIREMENTS v0.1）也不涉及"How 概要"（已 in BASIC-DESIGN v0.1）。

按 RGS 项目规范（per RGS-DTL-001 设计模式）：需求 → 基本 → 详细。

**继承上游全部内容**：REQUIREMENTS v0.1 全部（12 痛点 + 7 US + 44 FR + 16 表 + 12 IR + 14 NFR + 12 GAP + 8 R）+ BASIC-DESIGN v0.1 全部（架构总览 + 技术选型 + 模块划分 + 关键流程 + 数据模型 + 演进）。本文档只补"如何做细节"（API 签名 + 数据模型实现 + 部署 manifest + 运维 SOP + 安全策略 + 性能 + 测试）。

**对 rgs-web 母规范的继承**：本文档 rgs-batch-console 部分继承 `RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1` 的 0 依赖 + 127.0.0.1 only 模式；rgs-batch-backend 部分继承 `RGS-IMPL-PLAN-ADMIN-001` §gm-backend 范式（actix-web + mTLS + 8/27 ST 实践）。

---

## 1. API 详细签名

> 母规范 API 模式（端口 + JSON 响应 + 30s 轮询）**全部继承**。本节列 rgs-batch **新增** API 详细签名（console 11 + backend 26 = 37 endpoint）。

### 1.1 rgs-batch-console API (10 endpoint)

> 监听 `127.0.0.1:8789`，envoy 独立 deployment 代理（per 9/1 13:03/13:05 JST 偏好）。console 是 thin proxy + 静态资源，转发到 rgs-batch-backend ClusterIP `8790`。

#### 1.1.1 GET /api/health

**方法**：GET
**Query 参数**：无
**响应 200**：
```json
{
  "rgs_batch_console_version": "0.1.0",
  "backend_url": "http://rgs-batch-backend:8790",
  "uptime_seconds": 3600,
  "generated_at": "2026-09-01T18:00:00+09:00"
}
```
**实现**：`tools/rgs-batch-console/server.js` `routeHealth(req, res)`
**错误码**：500（backend 不可达 + hint: "per rgs-web §健康检查降级模式"）

#### 1.1.2 GET /api/version

**方法**：GET
**响应 200**：
```json
{
  "console": "0.1.0",
  "backend_target": "0.1.0",
  "envoy_version": "1.30.x",
  "generated_at": "..."
}
```

#### 1.1.3 POST /api/batch/gm-grant

**方法**：POST
**Body**：
```json
{
  "player_ids": ["p001", "p002", "..."],
  "rewards": { "gold": 100, "items": ["sword_001"] },
  "reason": "Q3 运营活动发奖"  // 必填, ≥ 20 字符 (per F-19 / audit)
}
```
**响应 202**：
```json
{
  "task_id": "uuid",
  "exec_id": "uuid",
  "status": "pending",
  "estimated_subtasks": 100,
  "created_at": "..."
}
```
**实现**：`routeGmGrant(req, res)` → 转发 `POST http://rgs-batch-backend:8790/api/v1/tasks`
**错误码**：400（参数错）/ 500（backend 错）

#### 1.1.4 POST /api/batch/schedule

**方法**：POST
**Body**：
```json
{
  "name": "夜间结算",
  "task_type": "settlement",  // 'cron' / 'interval' / 'oneshot' / 'gm_grant' / 'log_process' / 'data_migration'
  "trigger": { "type": "cron", "expr": "0 3 * * *" },
  "params": { "...": "..." },
  "enabled": true
}
```
**响应 201**：
```json
{ "schedule_id": "uuid", "task_id": "uuid", "next_run_at": "..." }
```
**实现**：`routeScheduleCreate(req, res)` → 转发 `POST http://rgs-batch-backend:8790/api/v1/schedules`

#### 1.1.5 GET /api/batch/tasks

**方法**：GET
**Query 参数**：
- `status` (optional, enum: pending/running/completed/failed/partial/all, default all)
- `task_type` (optional)
- `limit` (default 50, max 200)
- `offset` (default 0)
- `since` (optional ISO 8601)
**响应 200**：
```json
{
  "tasks": [
    { "task_id": "uuid", "task_type": "gm_grant", "status": "completed", "created_at": "...", "exec_id": "uuid", "result_summary": { "completed": 98, "failed": 2, "dlq": 2 } }
  ],
  "total": 100,
  "generated_at": "..."
}
```
**实现**：转发 `GET http://rgs-batch-backend:8790/api/v1/tasks`

#### 1.1.6 GET /api/batch/tasks/{id}/progress

**方法**：GET
**响应 200**：
```json
{
  "exec_id": "uuid",
  "completed": 50,
  "failed": 2,
  "dlq": 2,
  "total": 100,
  "eta_seconds": 60,
  "status": "running",
  "updated_at": "..."
}
```
**实现**：转发 `GET http://rgs-batch-backend:8790/api/v1/tasks/{id}/progress`
**错误码**：404（task 不存在）/ 500

#### 1.1.7 POST /api/batch/log-process

**方法**：POST
**Body**：
```json
{
  "source": "player/50051",  // 'player/50051' / 'file:/var/log/...' / 'kubectl:player-service'
  "filter": { "level": "info", "pattern": "error", "time_range": { "from": "...", "to": "..." } },
  "aggregate": { "type": "count", "group_by": "player_id" },
  "output": "postgres"  // 'postgres' / 'csv' / 'rgs-web-embed'
}
```
**响应 202**：
```json
{ "task_id": "uuid", "log_task_id": "uuid", "status": "pending" }
```
**实现**：转发 `POST http://rgs-batch-backend:8790/api/v1/log-tasks`

#### 1.1.8 POST /api/batch/data-migration

**方法**：POST
**Body**：
```json
{
  "source": "postgres://5 域 player_characters",
  "target": "postgres://archive.player_characters_2026q3",
  "type": "migration",  // 'migration' / 'aggregation' / 'transformation' / 'import' / 'export' / 'archive'
  "dry_run": true,
  "params": { "...": "..." },
  "reason": "Q3 玩家数据归档"  // 必填, ≥ 20 字符
}
```
**响应 202**：
```json
{ "task_id": "uuid", "migration_id": "uuid", "rollback_sql": "...", "status": "pending" }
```
**实现**：转发 `POST http://rgs-batch-backend:8790/api/v1/migration-tasks`

#### 1.1.9 GET /api/batch/audit

**方法**：GET
**Query 参数**：
- `task_id` (optional)
- `player_id` (optional, 通过 sub_task.target_id 反查)
- `action` (optional, enum)
- `since` (optional ISO 8601)
- `limit` (default 50, max 200)
**响应 200**：
```json
{
  "events": [
    { "event_id": "uuid", "exec_id": "uuid", "operator": "Ulysses", "action": "complete", "params_hash": "sha256:...", "result": { "completed": 98 }, "created_at": "...", "trace_id": "..." }
  ],
  "total": 50,
  "generated_at": "..."
}
```
**实现**：转发 `GET http://rgs-batch-backend:8790/api/v1/audit`
**重要**：`params_hash` 永不解密（per 8/27 11:06 JST 硬 ban + NFR-30）

#### 1.1.10 GET /api/batch/dlq

**方法**：GET
**Query 参数**：
- `exec_id` (optional)
- `resolved` (optional, default false)
- `limit` (default 50)
**响应 200**：
```json
{
  "events": [
    { "dlq_id": "uuid", "exec_id": "uuid", "sub_id": "uuid", "error": "5 域 gRPC timeout after 3 retries", "retry_count": 3, "first_failed_at": "...", "last_retried_at": "...", "resolved_at": null }
  ],
  "total": 10,
  "generated_at": "..."
}
```
**实现**：转发 `GET http://rgs-batch-backend:8790/api/v1/dlq`

#### 1.1.11 POST /api/batch/dlq/{id}/retry

**方法**：POST
**响应 200**：
```json
{ "dlq_id": "uuid", "status": "requeued", "new_sub_id": "uuid" }
```
**实现**：转发 `POST http://rgs-batch-backend:8790/api/v1/dlq/{id}/retry`

### 1.2 rgs-batch-backend API (26 endpoint)

> 监听 `0.0.0.0:8790`，ClusterIP service（k3s 内部访问，不直接暴露 127.0.0.1）。所有 endpoint 走 actix-web + JSON + 标准错误码。

#### 1.2.1 POST /api/v1/tasks

**方法**：POST
**Body**：
```json
{
  "task_type": "gm_grant",  // 'gm_grant' / 'log_process' / 'data_migration' / 'aggregation'
  "params": { "player_ids": [...], "rewards": {...} },
  "operator": "Ulysses",  // 默认 Ulysses
  "trace_id": "..."  // 可选, 不传则生成
}
```
**响应 202**：
```json
{
  "task_id": "uuid",
  "exec_id": "uuid",
  "status": "pending",
  "trace_id": "...",
  "created_at": "..."
}
```
**实现**：`src/api/task_def.rs` `create_task`
**关键逻辑**：
1. 验证 task_type 合法
2. 写 task_def M-1 (status=pending)
3. 写 task_execution T-1 (exec_id, params_snapshot, trace_id)
4. 写 audit_event T-3 (action=create, params_hash=sha256(params))
5. 写 audit_session W-3
6. 入队 mpsc channel → 5 worker pool
7. 返回 202
**错误码**：400（参数错）/ 500（DB 错）

#### 1.2.2 GET /api/v1/tasks

**方法**：GET
**Query 参数**：见 §1.1.5
**响应 200**：见 §1.1.5
**实现**：`src/api/task_execution.rs` `list_tasks`，sqlx 查询 batch_transaction.task_execution + batch_master.task_def JOIN

#### 1.2.3 GET /api/v1/tasks/{id}

**方法**：GET
**响应 200**：
```json
{
  "task_id": "uuid",
  "exec_id": "uuid",
  "task_type": "gm_grant",
  "status": "running",
  "params": { "player_ids": [...], "rewards": {...} },
  "params_snapshot": { "...": "..." },
  "progress": { "completed": 50, "failed": 2, "total": 100, "eta_seconds": 60 },
  "started_at": "...",
  "finished_at": null,
  "result_summary": null,
  "trace_id": "..."
}
```
**实现**：`src/api/task_execution.rs` `get_task`，sqlx 读 task_def M-1 + task_execution T-1 + task_progress W-1

#### 1.2.4 GET /api/v1/tasks/{id}/sub-tasks

**方法**：GET
**Query 参数**：
- `status` (optional)
- `limit` (default 100, max 1000)
**响应 200**：
```json
{
  "sub_tasks": [
    { "sub_id": "uuid", "exec_id": "uuid", "target_id": "p001", "status": "completed", "retry_count": 0, "started_at": "...", "finished_at": "...", "error": null, "result": { "gold": 100 } }
  ],
  "total": 100
}
```
**实现**：`src/api/sub_task.rs` `list_sub_tasks`，sqlx 读 sub_task T-2

#### 1.2.5 GET /api/v1/tasks/{id}/progress

**方法**：GET
**响应 200**：见 §1.1.6
**实现**：`src/api/task_execution.rs` `get_progress`，优先读 task_progress W-1（in-memory），fallback 聚合 sub_task T-2

#### 1.2.6 POST /api/v1/tasks/{id}/cancel

**方法**：POST
**响应 200**：
```json
{ "task_id": "uuid", "status": "cancelling" }
```
**实现**：`src/api/task_execution.rs` `cancel_task`
**关键逻辑**：
1. 写 task_def.status = 'cancelling'
2. worker 池每 30s 检查 status → 停止新 sub_task
3. 已执行的 sub_task 不撤销（per F-21 限制）
4. 写 audit_event T-3 (action=cancel)
5. 任务完成时 status = 'cancelled'

#### 1.2.7 CRUD /api/v1/schedules (4 endpoint)

**POST /api/v1/schedules**：创建定时任务（body 含 cron / interval / at）
**GET /api/v1/schedules**：列出（filter by enabled, task_type）
**GET /api/v1/schedules/{id}**：详情
**PUT /api/v1/schedules/{id}**：更新（启停 / 改 cron / 改 params）
**DELETE /api/v1/schedules/{id}**：删除（写 audit_event）

**实现**：`src/api/schedule.rs`
**关键逻辑**：
- cron: tokio-cron-scheduler 调度
- interval: tokio::time::interval
- oneshot: tokio::time::sleep_until(at)
- 触发时创建 task_execution T-1 (triggered_by=cron/interval/oneshot)
- 入队 worker pool（同 GM 流程）

#### 1.2.8 CRUD /api/v1/templates (4 endpoint)

**POST /api/v1/templates**：保存 SQL 模板（per F-19）
**GET /api/v1/templates**：列出（filter by type）
**GET /api/v1/templates/{id}**：详情 + sql_template
**DELETE /api/v1/templates/{id}**：删除

**实现**：`src/api/template.rs`，写 task_template M-2

#### 1.2.9 POST /api/v1/log-tasks

**方法**：POST
**Body**：见 §1.1.7
**响应 202**：见 §1.1.7
**实现**：`src/api/log_task.rs` `create_log_task`
**关键逻辑**：
1. 拉取 log 源（5 域 gRPC interceptor / 文件 glob / kubectl logs）
2. 写 task_def M-1 (type=log_process)
3. 写 task_execution T-1
4. 入队 → log/source → log/filter → log/aggregate → output
5. 写 log_event T-5（per 30 天保留）
6. 输出到 PostgreSQL / CSV / rgs-web embed

#### 1.2.10 POST /api/v1/migration-tasks

**方法**：POST
**Body**：见 §1.1.8
**响应 202**：见 §1.1.8
**实现**：`src/api/migration.rs` `create_migration`
**关键逻辑**：
1. before snapshot（写 data_migration T-6, before_snapshot JSONB）
2. 生成 rollback SQL（基于 before snapshot, per F-24）
3. dry_run=true → 仅生成 rollback，不执行
4. dry_run=false → 执行迁移（SQL 模板 / 5 域 gRPC list + write）
5. 写 audit_event T-3 (action=migration, params_hash, rollback_sql_ref)

#### 1.2.11 GET /api/v1/audit

**方法**：GET
**Query 参数**：见 §1.1.9
**响应 200**：见 §1.1.9
**实现**：`src/api/audit.rs` `list_audit`
**重要**：`params_hash` 永不解密（per 8/27 11:06 JST 硬 ban + NFR-30）

#### 1.2.12 CRUD /api/v1/dlq (3 endpoint)

**GET /api/v1/dlq**：列表（见 §1.1.10）
**POST /api/v1/dlq/{id}/retry**：从 DLQ 重新入队
**POST /api/v1/dlq/{id}/resolve**：标记 resolved（不重试）

**实现**：`src/api/dlq.rs`
**关键逻辑**：
- retry: 写 dlq_event.resolved_at + 重新入队
- resolve: 写 dlq_event.resolved_at + audit_event T-3 (action=dlq_resolve)

#### 1.2.13 GET /api/v1/worker-pools

**方法**：GET
**响应 200**：
```json
{
  "pools": [
    { "pool_id": "uuid", "domain": "player", "max_concurrent": 5, "rpm_limit": 1000, "enabled": true, "current_concurrent": 2 }
  ]
}
```
**实现**：`src/api/dlq.rs`（或单独 src/api/worker_pool.rs）

#### 1.2.14 CRUD /api/v1/data-sources (3 endpoint)

**POST /api/v1/data-sources**：注册数据源
**GET /api/v1/data-sources**：列表
**DELETE /api/v1/data-sources/{id}**：删除

**实现**：`src/api/template.rs`（或单独 src/api/data_source.rs）
**重要**：conn_str_ref / credentials_ref 只存 env var 名，**不存值**（per 8/27 11:06 JST 硬 ban）

#### 1.2.15 GET /api/v1/health

**方法**：GET
**响应 200**：
```json
{
  "rgs_batch_backend_version": "0.1.0",
  "db_pool_size": 10,
  "worker_pool_size": 5,
  "active_tasks": 2,
  "dlq_count": 5,
  "uptime_seconds": 7200,
  "generated_at": "..."
}
```
**实现**：`src/api/health.rs`

#### 1.2.16 GET /metrics (Prometheus scrape)

**方法**：GET
**响应 200**：`text/plain` Prometheus 格式
**关键指标**：
- `rgs_batch_active_tasks`
- `rgs_batch_subtasks_total{status="completed|failed|dlq"}`
- `rgs_batch_dlq_size`
- `rgs_batch_5 域_grpc_call_duration_seconds{domain, method, status}`
- `rgs_batch_db_pool_size`
- `rgs_batch_audit_events_total`
**实现**：`src/metrics_endpoint.rs`，复用 shared-platform/metrics.rs
**端口**：9464（per gm-backend 范式，5 域统一）

---

## 2. 数据模型（实现细节）

> BASIC-DESIGN §5.3 已声明 16 张表 Schema，本节补充**实现细节**（migration 文件 + 索引 + 约束 + 性能调优）。

### 2.1 migration 文件顺序

> 19 文件对应 16 张表 + 3 schema 创建（per BASIC-DESIGN §5.2）。sqlx migrate 顺序执行。

```
migrations/
├── 0001_create_batch_master_schema.sql        (CREATE SCHEMA batch_master)
├── 0002_create_batch_transaction_schema.sql   (CREATE SCHEMA batch_transaction)
├── 0003_create_batch_work_schema.sql          (CREATE SCHEMA batch_work)
├── 0004_create_task_def.sql                   (M-1)
├── 0005_create_task_template.sql              (M-2)
├── 0006_create_data_source.sql                (M-3)
├── 0007_create_worker_pool.sql                (M-4)
├── 0008_create_schedule.sql                   (M-5)
├── 0009_create_task_execution.sql             (T-1)
├── 0010_create_sub_task.sql                   (T-2)
├── 0011_create_audit_event.sql                (T-3)
├── 0012_create_dlq_event.sql                  (T-4)
├── 0013_create_log_event.sql                  (T-5)
├── 0014_create_data_migration.sql             (T-6)
├── 0015_create_task_progress.sql              (W-1)
├── 0016_create_task_buffer.sql                (W-2)
├── 0017_create_audit_session.sql              (W-3)
├── 0018_create_log_buffer.sql                 (W-4)
└── 0019_create_migration_buffer.sql           (W-5)
```

### 2.2 关键索引与性能调优

> 索引基于查询模式设计（task_id / player_id / created_at / status 4 类高频查询）。

| # | 表 | 索引 | 索引字段 | 查询模式 | 备注 |
|---|---|---|---|---|---|
| 1 | batch_master.task_def | idx_task_def_owner | (owner) | "我的任务" 列表 | owner 默认 Ulysses |
| 2 | batch_master.task_def | idx_task_def_status | (status) | 按 status 筛选 | |
| 3 | batch_master.task_def | idx_task_def_type_created | (task_type, created_at DESC) | 类型 + 时间排序 | |
| 4 | batch_master.schedule | idx_schedule_next_run | (next_run_at) WHERE enabled = true | 调度器 tick | partial index 优化 |
| 5 | batch_transaction.task_execution | idx_task_execution_task_id | (task_id) | 按 task 查执行 | 高频 |
| 6 | batch_transaction.task_execution | idx_task_execution_started_at | (started_at DESC) | 时间排序 | |
| 7 | batch_transaction.task_execution | idx_task_execution_status_started | (status, started_at DESC) | 状态 + 时间 | |
| 8 | batch_transaction.sub_task | idx_sub_task_exec_id | (exec_id) | 按 exec 查 sub | 高频 |
| 9 | batch_transaction.sub_task | idx_sub_task_target_id | (target_id) | 按 player 查 sub | 用于审计 player 维度 |
| 10 | batch_transaction.sub_task | idx_sub_task_status | (status) | 失败 / DLQ 筛选 | |
| 11 | batch_transaction.audit_event | idx_audit_event_exec_id | (exec_id) | 按 exec 查审计 | 高频 |
| 12 | batch_transaction.audit_event | idx_audit_event_created_at | (created_at DESC) | 时间排序 | |
| 13 | batch_transaction.audit_event | idx_audit_event_target_via_sub | (created_at DESC) INCLUDE (exec_id) | 时间范围扫描 | covering index |
| 14 | batch_transaction.dlq_event | idx_dlq_event_exec_id | (exec_id) | 按 exec 查 DLQ | |
| 15 | batch_transaction.dlq_event | idx_dlq_event_resolved | (resolved_at) WHERE resolved_at IS NULL | 未解决 DLQ 列表 | partial index |
| 16 | batch_transaction.log_event | idx_log_event_source_ts | (source, ts DESC) | 按源 + 时间 | |
| 17 | batch_transaction.log_event | idx_log_event_ts | (ts DESC) | 全局时间排序 | |
| 18 | batch_transaction.data_migration | idx_data_migration_applied | (applied_at DESC) | 时间排序 | |

### 2.3 数据保留与归档策略（实现）

> BASIC-DESIGN §5.5 已声明 7 类保留策略，本节补充**实现细节**（cron 任务 + 归档 schema + 自动清理）。

| # | 数据 | 保留期 | 归档实现 | 自动清理 |
|---|---|---|---|---|
| 1 | task_def (M-1) | 永久 | 不归档 | 不清理 |
| 2 | task_execution (T-1) | 90 天 | 90 天后 `INSERT INTO batch_transaction_archive.task_execution SELECT * WHERE started_at < now() - INTERVAL '90 days'` + DELETE | nightly cron 3 AM |
| 3 | sub_task (T-2) | 90 天 | 同上 | 同上 |
| 4 | audit_event (T-3) | 永久 | 不归档（per NFR-29） | 不清理 |
| 5 | dlq_event (T-4) | 90 天 | 90 天后归档 resolved DLQ | nightly cron |
| 6 | log_event (T-5) | 30 天 | 30 天后 `COPY TO /archive/log_event_YYYY-MM.csv.gz` + DELETE | nightly cron 4 AM |
| 7 | task_progress (W-1) | 任务完成 | 任务结束 → DELETE FROM batch_work.task_progress | executor 完成后清理 |
| 8 | 其他 Work 表 (W-2/W-3/W-4/W-5) | 任务 / session 结束 | session 结束 → DELETE | executor / session 结束清理 |

**归档 cron**（per 5 域 ST 业务级 mTLS 实践 commit 401ac5c 部署模式）：

```sql
-- migration 0020_create_archive_schemas.sql
CREATE SCHEMA batch_transaction_archive;
CREATE TABLE batch_transaction_archive.task_execution (LIKE batch_transaction.task_execution INCLUDING ALL);
CREATE TABLE batch_transaction_archive.sub_task (LIKE batch_transaction.sub_task INCLUDING ALL);

-- nightly cron 3 AM JST (per 5 域 shared-platform cron 模式)
-- 实现: 单独的 sqlx migration 后台 task, 或 k8s CronJob (per 5 域 cron 实践)
```

### 2.4 数据完整性约束

```sql
-- FK 约束
ALTER TABLE batch_transaction.task_execution
  ADD CONSTRAINT fk_task_execution_task_def
  FOREIGN KEY (task_id) REFERENCES batch_master.task_def(task_id) ON DELETE CASCADE;

ALTER TABLE batch_transaction.sub_task
  ADD CONSTRAINT fk_sub_task_task_execution
  FOREIGN KEY (exec_id) REFERENCES batch_transaction.task_execution(exec_id) ON DELETE CASCADE;

ALTER TABLE batch_master.schedule
  ADD CONSTRAINT fk_schedule_task_def
  FOREIGN KEY (task_id) REFERENCES batch_master.task_def(task_id) ON DELETE CASCADE;

-- CHECK 约束
ALTER TABLE batch_master.task_def
  ADD CONSTRAINT chk_task_def_status
  CHECK (status IN ('pending', 'running', 'completed', 'failed', 'partial', 'cancelled', 'cancelling'));

ALTER TABLE batch_transaction.sub_task
  ADD CONSTRAINT chk_sub_task_retry_count
  CHECK (retry_count >= 0 AND retry_count <= 3);  -- 最多 3 次, per NFR-26

-- UNIQUE 约束
ALTER TABLE batch_master.task_template
  ADD CONSTRAINT uq_task_template_name_version
  UNIQUE (name, version);
```

### 2.5 写并发安全（1 写者约束）

> per OLU-WEB §5.1.5 + 1 写者约束：rgs-batch-backend 单进程 + tokio multi-thread runtime。

- **Master 表写并发**：rgs-batch-backend 单进程写，无外部写者，sqlx 默认 1 写者安全
- **Transaction 表写并发**：同上
- **Work 表写并发**：同上 + 任务结束后清理（无残留）
- **data/.lock 原子锁**（rgs-batch-console 端）：per OLU-WEB §5.1.5，`fs.openSync('.lock', 'wx')` 原子创建
- **不引入 SQLite / better-sqlite3**：per BASIC-DESIGN §2.3 决策

---

## 3. 部署

> 母规范 rgs-web 部署（node 22 + 8788 + WSL2 + k3s port-forward）**部分继承**。本节列 rgs-batch **新增** k8s manifests。

### 3.1 新增 k8s manifests 文件

```
docs/deploy/01-k8s-manifests/
├── 70-rgs-batch-console-deployment.yaml     (新增)
├── 71-rgs-batch-console-service.yaml        (新增, ClusterIP)
├── 72-rgs-batch-backend-deployment.yaml     (新增)
├── 73-rgs-batch-backend-service.yaml        (新增, ClusterIP)
├── 74-rgs-batch-envoy-deployment.yaml       (新增, 独立 deployment per 9/1 13:03/13:05 JST)
├── 75-rgs-batch-envoy-service.yaml          (新增, ClusterIP)
├── 76-rgs-batch-configmap.yaml              (新增, non-secret 配置)
├── 77-rgs-batch-secret.yaml.example         (新增, env var 模板, 真实 secret 不入 git per 8/27 11:06 JST)
└── 78-rgs-batch-networkpolicy.yaml          (新增, 5 域 gRPC egress only)
```

### 3.2 rgs-batch-console Deployment (示例)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rgs-batch-console
  namespace: rust-game-server
  labels:
    app.kubernetes.io/name: rgs-batch-console
    app.kubernetes.io/component: batch-ui
    app.kubernetes.io/part-of: rust-game-server
    rust-game-server.io/role: batch-console
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: rgs-batch-console
  template:
    metadata:
      labels:
        app.kubernetes.io/name: rgs-batch-console
    spec:
      containers:
      - name: console
        image: node:22-alpine
        command: ["node", "/app/server.js"]
        workingDir: /app
        ports:
        - containerPort: 8789
          name: http
        env:
        - name: RGS_BATCH_CONSOLE_PORT
          value: "8789"
        - name: RGS_BATCH_BACKEND_URL
          value: "http://rgs-batch-backend:8790"
        - name: RGS_BATCH_CONSOLE_BIND
          value: "127.0.0.1:8789"  # 实际由 envoy 代理, pod 内部仍 listen 0.0.0.0
        - name: RGS_BATCH_LOG_LEVEL
          value: "info"
        volumeMounts:
        - name: app
          mountPath: /app
        - name: data
          mountPath: /app/data
        resources:
          requests: { cpu: 100m, memory: 128Mi }
          limits: { cpu: 500m, memory: 256Mi }
        livenessProbe:
          httpGet: { path: /api/health, port: 8789 }
          initialDelaySeconds: 5
          periodSeconds: 30
        readinessProbe:
          httpGet: { path: /api/health, port: 8789 }
          initialDelaySeconds: 3
          periodSeconds: 10
      volumes:
      - name: app
        hostPath: { path: /path/to/rustgameserver/tools/rgs-batch-console }
      - name: data
        emptyDir: {}  # data 目录, 容器重启清空
```

### 3.3 rgs-batch-backend Deployment (示例)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rgs-batch-backend
  namespace: rust-game-server
  labels:
    app.kubernetes.io/name: rgs-batch-backend
    app.kubernetes.io/component: batch-engine
    app.kubernetes.io/part-of: rust-game-server
    rust-game-server.io/role: batch-backend
spec:
  replicas: 1  # 1 写者约束, per OLU-WEB §5.1.5
  selector:
    matchLabels:
      app.kubernetes.io/name: rgs-batch-backend
  template:
    metadata:
      labels:
        app.kubernetes.io/name: rgs-batch-backend
    spec:
      containers:
      - name: backend
        image: rust-batch-backend:0.1.0  # cargo build --release 后构建
        command: ["./rgs-batch-backend"]
        ports:
        - containerPort: 8790
          name: http
        - containerPort: 9464
          name: metrics
        env:
        - name: RUST_LOG
          value: "info"
        - name: BATCH_DB_HOST
          value: "postgres"
        - name: BATCH_DB_PORT
          value: "5432"
        - name: BATCH_DB_USER
          valueFrom: { secretKeyRef: { name: rgs-batch-db-secret, key: username } }
        - name: BATCH_DB_PASSWORD
          valueFrom: { secretKeyRef: { name: rgs-batch-db-secret, key: password } }
        - name: BATCH_DB_NAME
          value: "rgs_batch"
        - name: BATCH_WORKER_POOL_SIZE
          value: "5"
        - name: BATCH_BACKEND_BIND
          value: "0.0.0.0:8790"
        - name: GRPC_CA_CERT_PATH
          value: "/etc/certs/ca.crt"
        - name: GRPC_CLIENT_CERT_PATH
          value: "/etc/certs/client.crt"
        - name: GRPC_CLIENT_KEY_PATH
          value: "/etc/certs/client.key"
        - name: GRPC_CERT_PATH_PLAYER
          value: "/etc/certs/player.crt"
        - name: GRPC_CERT_PATH_ECONOMY
          value: "/etc/certs/economy.crt"
        - name: GRPC_CERT_PATH_MATCH
          value: "/etc/certs/match.crt"
        - name: GRPC_CERT_PATH_SOCIAL
          value: "/etc/certs/social.crt"
        - name: GRPC_CERT_PATH_ADMIN
          value: "/etc/certs/admin.crt"
        volumeMounts:
        - name: certs
          mountPath: /etc/certs
          readOnly: true
        resources:
          requests: { cpu: 200m, memory: 256Mi }
          limits: { cpu: 1000m, memory: 1Gi }  # NFR-33
        livenessProbe:
          httpGet: { path: /api/v1/health, port: 8790 }
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet: { path: /api/v1/health, port: 8790 }
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: certs
        secret:
          secretName: rgs-batch-grpc-certs
          defaultMode: 0400
```

### 3.4 envoy 独立 Deployment (示例)

> per 9/1 13:03/13:05 JST 偏好: envoy 独立 deployment, 不选 istio sidecar

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rgs-batch-envoy
  namespace: rust-game-server
  labels:
    app.kubernetes.io/name: rgs-batch-envoy
    app.kubernetes.io/component: edge-proxy
    app.kubernetes.io/part-of: rust-game-server
    rust-game-server.io/role: batch-edge
spec:
  replicas: 2
  selector:
    matchLabels:
      app.kubernetes.io/name: rgs-batch-envoy
  template:
    metadata:
      labels:
        app.kubernetes.io/name: rgs-batch-envoy
    spec:
      containers:
      - name: envoy
        image: envoyproxy/envoy:v1.30-latest
        ports:
        - containerPort: 8443
          name: https
        - containerPort: 9901
          name: admin
        volumeMounts:
        - name: config
          mountPath: /etc/envoy
        - name: certs
          mountPath: /etc/certs
          readOnly: true
        resources:
          requests: { cpu: 100m, memory: 128Mi }
          limits: { cpu: 500m, memory: 256Mi }
      volumes:
      - name: config
        configMap:
          name: rgs-batch-envoy-config
      - name: certs
        secret:
          secretName: rgs-batch-tls-certs
```

### 3.5 mTLS 证书挂载

> 证书复用 5 域 ST 业务级 mTLS 实践（commit 401ac5c）+ 8/27 ST 导出 SOP

```bash
# 一次性: 5 域 ST 证书导出 (per 8/27 ST 实践)
kubectl get secret player-tls -n rust-game-server -o yaml > certs/player-tls.yaml
kubectl get secret economy-tls -n rust-game-server -o yaml > certs/economy-tls.yaml
kubectl get secret match-tls -n rust-game-server -o yaml > certs/match-tls.yaml
kubectl get secret social-tls -n rust-game-server -o yaml > certs/social-tls.yaml
kubectl get secret admin-tls -n rust-game-server -o yaml > certs/admin-tls.yaml

# 一次性: rgs-batch 自己证书生成 (per 8/27 ST 实践 + 5 域 certgen 工具)
# 使用 crates/rgs-certgen 工具生成 rgs-batch 自己的 client cert + CA
cargo run -p rgs-certgen -- batch  # 输出 certs/rgs-batch-tls.yaml

# 部署: kubectl apply
kubectl apply -f certs/ -n rust-game-server
```

### 3.6 NetworkPolicy (5 域 gRPC egress only)

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: rgs-batch-backend
  namespace: rust-game-server
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: rgs-batch-backend
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector: { matchLabels: { app.kubernetes.io/name: rgs-batch-console } }
    - podSelector: { matchLabels: { app.kubernetes.io/name: rgs-batch-envoy } }
    ports:
    - port: 8790
  egress:
  # 5 域 gRPC
  - to:
    - podSelector: { matchLabels: { app.kubernetes.io/name: player-service } }
    ports: [{ port: 50051 }]
  - to:
    - podSelector: { matchLabels: { app.kubernetes.io/name: economy-service } }
    ports: [{ port: 50052 }]
  - to:
    - podSelector: { matchLabels: { app.kubernetes.io/name: match-service } }
    ports: [{ port: 50053 }]
  - to:
    - podSelector: { matchLabels: { app.kubernetes.io/name: social-service } }
    ports: [{ port: 50054 }]
  - to:
    - podSelector: { matchLabels: { app.kubernetes.io/name: admin-service } }
    ports: [{ port: 50055 }]
  # PostgreSQL
  - to:
    - podSelector: { matchLabels: { app.kubernetes.io/name: postgres } }
    ports: [{ port: 5432 }]
  # DNS
  - to: [ { namespaceSelector: {} } ]
    ports: [{ port: 53, protocol: UDP }]
```

### 3.7 启动 SOP

```bash
# 1. (一次性) PostgreSQL schema + migration 准备
psql -h postgres -U batch_user -d rgs_batch -c "CREATE SCHEMA IF NOT EXISTS batch_master; CREATE SCHEMA IF NOT EXISTS batch_transaction; CREATE SCHEMA IF NOT EXISTS batch_work; CREATE SCHEMA IF NOT EXISTS batch_transaction_archive;"

# 2. (一次性) rgs-batch-backend DB migration
cd tools/rgs-batch-backend
sqlx migrate run  # 自动跑 migrations/ 0001-0019

# 3. (一次性) 5 域 ST 证书导出 (per 8/27 ST 实践)
kubectl get secret player-tls -n rust-game-server -o yaml > certs/player-tls.yaml
# ... (重复 5 域)

# 4. (一次性) rgs-batch 自己证书 (per crates/rgs-certgen)
cargo run -p rgs-certgen -- batch

# 5. (一次性) 配置 env var (per 8/27 11:06 JST 硬 ban, 不入 git)
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

# 6. (一次性) 应用 manifests
kubectl apply -f docs/deploy/01-k8s-manifests/70-rgs-batch-console-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/72-rgs-batch-backend-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/74-rgs-batch-envoy-deployment.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/78-rgs-batch-networkpolicy.yaml

# 7. 验证
kubectl get pods -l 'app.kubernetes.io/part-of=rust-game-server,app.kubernetes.io/component in (batch-ui, batch-engine, batch-edge)' -n rust-game-server
# 期望: rgs-batch-console 1/1 + rgs-batch-backend 1/1 + rgs-batch-envoy 2/2

# 8. (本地 dev) port-forward 8789
kubectl port-forward svc/rgs-batch-envoy 8789:8443 -n rust-game-server

# 9. 浏览器访问
# http://127.0.0.1:8789
```

---

## 4. 运维

> 母规范 rgs-web 运维（启动 / 监控 / 故障恢复）**部分继承**。本节列 rgs-batch **新增**运维项。

### 4.1 监控项

| # | 指标 | 阈值 | 告警 | 备注 |
|---|---|---|---|---|
| 1 | `rgs_batch_active_tasks` | > 10 | 黄 | 同时运行任务数 |
| 2 | `rgs_batch_dlq_size` | > 50 | 黄 | DLQ 堆积 |
| 3 | `rgs_batch_5 域_grpc_call_duration_seconds` (p95) | > 1s | 黄 | 5 域 gRPC 调用慢 |
| 4 | `rgs_batch_5 域_grpc_call_total{status="error"}` | > 10/min | 红 | 5 域 gRPC 失败率高 |
| 5 | `rgs_batch_db_pool_size` | 95% 占用 | 黄 | DB 连接池满 |
| 6 | `rgs_batch_worker_pool_concurrent` | 95% of 5 | 黄 | Worker 池满 |
| 7 | `rgs_batch_audit_events_total` rate | - | - | 仅记录, 不告警 |
| 8 | task_progress.failed / task_progress.total | > 10% | 黄 | 任务失败率高 |
| 9 | schedule.next_run_at lag | > 5 min | 红 | 调度器卡住 |
| 10 | cert 过期时间 (v0.2 评估) | < 7 天 | 红 | mTLS 证书即将过期 |

### 4.2 故障恢复

| # | 故障 | 检测 | 恢复 SOP | 备注 |
|---|---|---|---|---|
| 1 | rgs-batch-backend crash | k8s liveness 失败 | k8s 自动重启 + 从 task_execution T-1 恢复未完成任务 | per NFR-25 任务不丢 |
| 2 | rgs-batch-console crash | k8s liveness 失败 | k8s 自动重启 + data/ 目录 emptyDir 重启清空 | data 不持久, 接受 |
| 3 | envoy crash | k8s liveness 失败 | k8s 自动重启 (2 replicas 高可用) | per 9/1 13:03 JST 偏好 |
| 4 | PostgreSQL 不可达 | sqlx connect 失败 | 任务入队失败 → 返 500 + audit_event + mavis cron 告警 (v0.2) | 不静默失败 |
| 5 | 5 域 gRPC 不可达 | tonic 调用失败 | retry 3 次 (per NFR-26) + DLQ | per BASIC §4.3 |
| 6 | mTLS 证书过期 | cert 验证失败 | retry + DLQ + (v0.2) 自动 reload | per §4.1 #10 |
| 7 | data/.lock 僵死 (rgs-batch-console 端) | mtime > 1h | 启动时检测 + 删除 | per OLU-WEB §5.1.5 |
| 8 | task_progress 卡住 (W-1 > 1h 未更新) | 30s 轮询检查 | 标记为 'stale' + mavis cron 告警 (v0.2) | per F-22 + NFR-24 |
| 9 | audit_log 写入失败 | sqlx INSERT 失败 | 任务继续 + 顶部条黄态 + mavis cron 告警 (v0.2) | 不阻塞主流程 |
| 10 | data_migration 失败 (F-24) | migration 任务失败 | rollback SQL 自动执行 (per F-24) | 迁移原子性 |

### 4.3 数据备份

| # | 数据 | 备份策略 | 周期 | 保留期 |
|---|---|---|---|---|
| 1 | batch_master.* | k8s PVC snapshot | daily 3 AM JST | 30 天 |
| 2 | batch_transaction.* | k8s PVC snapshot + WAL archive | daily + continuous | 90 天 |
| 3 | batch_work.* | 不备份 (session-bound, 任务结束清理) | - | - |
| 4 | batch_transaction_archive.* | k8s PVC snapshot | weekly | 永久 |
| 5 | rgs-batch-console/data/*.jsonl | hostPath + tar.gz | weekly | 30 天 |
| 6 | certs/*.crt, *.key | 加密备份到 1Password | weekly | 永久 |
| 7 | k8s manifests (70-78) | git 跟踪 (per 5 域实践) | on commit | 永久 |

### 4.4 运维工具脚本

```
scripts/
├── rgs-batch-start.sh         (本地 dev 启动: kubectl apply + port-forward)
├── rgs-batch-stop.sh          (本地 dev 停止: kubectl delete)
├── rgs-batch-logs.sh          (k8s logs 聚合: console + backend + envoy)
├── rgs-batch-health.sh        (健康检查: pod status + 5 域 gRPC + DB)
├── rgs-batch-backup.sh        (数据备份: k8s PVC snapshot + cert backup)
├── rgs-batch-restore.sh       (数据恢复: 从 snapshot 恢复)
├── rgs-batch-cert-rotate.sh   (v0.2 证书轮换: file watch + reload)
└── rgs-batch-migration-archive.sh  (90 天 / 30 天归档清理)
```

---

## 5. 安全

> 母规范 rgs-web 安全（NFR-10 至 NFR-14 监听地址 / TLS / 凭据 / Cookie / 文件权限）**全部继承**。本节列 rgs-batch **新增**安全约束。

### 5.1 凭据管理（env value hard ban 强校验）

> per 2026-08-27 11:06 JST 硬 ban。**所有**凭据 env var 注入，**永不打印值**。

| # | 凭据 | 注入方式 | 存储 | 暴露风险 | 强校验 |
|---|---|---|---|---|---|
| 1 | BATCH_DB_PASSWORD | env var + k8s secretKeyRef | 内存 (`std::env::var`) | 仅 sqlx 连接字符串 | 启动时 `println!` 过滤 + tracing 字段白名单 |
| 2 | GRPC_CLIENT_KEY | env var + k8s secret mount | 内存 | 仅 rustls TLS config | 同上 |
| 3 | GRPC_CERT_PATH_* (5 域 cert) | env var + k8s secret mount | 文件 /etc/certs/*.crt | 永不读值, 仅文件路径 | 文件权限 0400, k8s secret 加密 |
| 4 | trace_id / exec_id | 自生成 UUID | DB | 无敏感信息 | 无 |

**强校验（per 8/27 11:06 JST 反例禁）**：

```rust
// ❌ 禁止: println!("DB password: {}", std::env::var("BATCH_DB_PASSWORD")?);
// ❌ 禁止: tracing::info!("db_password={}", password);
// ✅ 允许: tracing::info!(db_host = %host, db_user = %user, "db connect");  // 脱敏
// ✅ 允许: tracing::info!(cert_path = %path, "loading cert");  // 仅路径
```

**所有 API handler 返响应前 filter**：

```rust
async fn response_filter(response: &mut serde_json::Value) {
    let banned_keys = ["password", "key", "token", "secret", "credential"];
    filter_recursive(response, &banned_keys);
}

fn filter_recursive(v: &mut serde_json::Value, banned: &[&str]) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map.iter_mut() {
                if banned.contains(&k.to_lowercase().as_str()) {
                    *val = serde_json::Value::String("***REDACTED***".to_string());
                } else {
                    filter_recursive(val, banned);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                filter_recursive(v, banned);
            }
        }
        _ => {}
    }
}
```

### 5.2 写并发安全

- rgs-batch-backend 单进程 + tokio multi-thread runtime（per BASIC §2.2 + 1 写者约束）
- sqlx 默认 1 写者安全
- rgs-batch-console 单进程（per OLU-WEB §5.1.5）+ data/.lock 原子锁
- 不引入 SQLite / better-sqlite3（per BASIC §2.3）
- 不引入共享内存 IPC

### 5.3 数据隐私

| # | 数据 | 敏感等级 | 保护 |
|---|---|---|---|
| 1 | audit_event.params_hash | 中 | sha256 哈希, 永不解密 |
| 2 | task_def.params | 中 | API 返响应时 redact 敏感字段（password / token / key）|
| 3 | sub_task.result | 中 | 5 域 gRPC 响应, 可能含 player 数据, 永久保留 (per NFR-29) |
| 4 | log_event.message | 低 | log 内容, 不含 secret (per 5 域 gRPC interceptor 过滤) |
| 5 | data_migration.before_snapshot | 高 | 源表完整数据, k8s secret 加密 + 仅 batch Lead 可读 |
| 6 | cert 文件 | 高 | k8s secret 加密 + 文件权限 0400 |

### 5.4 127.0.0.1 only 硬约束

- rgs-batch-console 监听 `127.0.0.1:8789`（per NFR-31）
- 不接受 `0.0.0.0` / `localhost` 之外
- 生产环境：envoy 边缘代理 + ClusterIP service（k3s 内部），用户访问走 envoy 8443
- 本地 dev：envoy port-forward 8789:8443

### 5.5 mTLS 业务级

- 5 域 gRPC 调用强制 mTLS（per NFR-32 + gm-backend 8/27 ST 业务级 mTLS 实践 commit 401ac5c）
- 证书路径 env var 注入
- 证书复用 5 域 ST 导出 SOP（per 8/27 ST 实践）
- 证书轮换（v0.2 评估）：file watch + 自动 reload

### 5.6 audit 永久保留

- audit_event T-3 永久保留（per NFR-29）
- 操作人 / 时间 / 参数 hash / 结果 / trace_id 全记录
- 按 task_id / player_id / 时间范围 / 操作类型 检索
- 导出 CSV / JSON

---

## 6. 性能

> 母规范 rgs-web NFR-1 至 NFR-5（< 2s 首页 / < 500ms API / < 100MB 内存 / < 1s 启动 / 10 RPS）**部分继承**。本节列 rgs-batch **新增**性能约束。

| # | 指标 | 目标 | 实测方法 |
|---|---|---|---|
| PF-1 | rgs-batch-console 启动 | < 1s | `time node server.js` |
| PF-2 | rgs-batch-backend 启动 | < 3s | `time ./rgs-batch-backend` |
| PF-3 | POST /api/v1/tasks 响应 | < 200ms | 内存 + sqlx 写入, ~50ms |
| PF-4 | GET /api/v1/tasks/{id}/progress 响应 | < 100ms | 优先 W-1 in-memory, fallback T-2 聚合 |
| PF-5 | 5 worker 池跑 1K 子任务 | ≤ 30s | 5 域 gRPC 并发 5 |
| PF-6 | 5K 子任务 | ≤ 5min | 100 player / chunk, 5 并发 |
| PF-7 | 100K 子任务 (NFR-23) | ≤ 1h | 分批提交, 5 域 RPM 限流 |
| PF-8 | 任务提交吞吐 (NFR-22) | ≥ 100 子任务/秒 | 5 worker 池默认 |
| PF-9 | 审计检索 100 条 (NFR-28) | < 1s | idx_audit_event_* 覆盖索引 |
| PF-10 | 5 域 gRPC 调用 p95 | < 200ms | mTLS + rustls + connection pool |
| PF-11 | 内存上限 (NFR-33) | < 1GB / pod | k8s resource limit + 监控告警 |
| PF-12 | DB 连接池 | 10 连接 / pod | sqlx pool size |
| PF-13 | 长跑任务 (NFR-24) | ≤ 24h | timeout 默认 1h, 可配 (per F-22) |
| PF-14 | 30s 轮询可见 (NFR-27) | ≤ 30s | setInterval 30s + 写时立即触发 |
| PF-15 | envoy 反代 p95 | < 50ms | 2 replicas 高可用 |

---

## 7. 测试策略

### 7.1 单元测试（UT）

> 沿用 rgs-testkit 禁 InMemory 模式（per AGENTS.md L3 + crates/rgs-testkit/src/lib.rs L17-34 强约束段），用 NoOp + 真实 sqlx + 真实 5 域 gRPC client mock。

| # | 测试 | 文件 | 覆盖 |
|---|---|---|---|
| 1 | `ut_executor` | `tests/ut_executor.rs` | 任务执行主循环 + 进度推送 |
| 2 | `ut_worker_pool` | `tests/ut_worker_pool.rs` | 5 worker 池并发 + mpsc channel |
| 3 | `ut_sharder` | `tests/ut_sharder.rs` | 分片策略 + chunk 大小 |
| 4 | `ut_retry` | `tests/ut_retry.rs` | 重试 3 次 + 指数退避 (per NFR-26) |
| 5 | `ut_dlq` | `tests/ut_dlq.rs` | DLQ 写入 + 读取 + resolve |
| 6 | `ut_progress` | `tests/ut_progress.rs` | 进度聚合 + ETA 计算 |
| 7 | `ut_cron` | `tests/ut_cron.rs` | cron 表达式解析 + tick |
| 8 | `ut_migration_rollback` | `tests/ut_migration_rollback.rs` | before snapshot + rollback SQL 生成 |
| 9 | `ut_audit_filter` | `tests/ut_audit_filter.rs` | 凭据 redact filter (per 8/27 11:06 JST) |
| 10 | `ut_lockfile` | `tools/rgs-batch-console/tests/ut_lockfile.js` | fs.openSync('.lock', 'wx') 原子 |
| 11 | `ut_token_estimate` | `tools/rgs-batch-console/tests/ut_token_estimate.js` | message_count × 5K |

### 7.2 集成测试（IT）

> per AGENTS.md v0.3 §2.3 L3: 跨工具链决策前先查 workspace 依赖 + rgs-testkit 强约束段。

| # | 测试 | 文件 | 覆盖 |
|---|---|---|---|
| 1 | `it_grpc_client` | `tests/it_grpc_client.rs` | 5 域 gRPC client + mTLS + retry (用 5 域真实 svc) |
| 2 | `it_api_task_def` | `tests/it_api_task_def.rs` | POST /api/v1/tasks + 5 worker 池端到端 |
| 3 | `it_api_schedule` | `tests/it_api_schedule.rs` | cron / interval / oneshot 触发 |
| 4 | `it_api_log_task` | `tests/it_api_log_task.rs` | log 源 + 过滤 + 聚合 + 输出 |
| 5 | `it_api_migration` | `tests/it_api_migration.rs` | 迁移 + rollback SQL + dry-run |
| 6 | `it_e2e_gm_grant` | `tests/it_e2e_gm_grant.rs` | GM 批量发奖端到端 (console + backend + 5 域) |
| 7 | `it_dlq_recovery` | `tests/it_dlq_recovery.rs` | 5 域 gRPC down + DLQ + 恢复 |
| 8 | `it_audit_persistence` | `tests/it_audit_persistence.rs` | audit_event 永久保留 + 检索 |

### 7.3 系统测试（ST）

> per AGENTS.md v0.3 §2.5 L5: ST worktree 启动 checklist + 证书导出 + 5 域 svc 真实部署。

| # | 测试 | 工具 | 覆盖 |
|---|---|---|---|
| 1 | ST-01: k3s 部署 e2e | kubectl + port-forward | console + backend + envoy 3 deployment 1/1 Running |
| 2 | ST-02: GM 批量发奖真实 5 域 | grpcurl + curl | 100 player 真实发奖, 100% 成功 |
| 3 | ST-03: 定时调度真实 5 域 | kubectl logs + 等待 | cron 任务 5 分钟 1 次自动执行 |
| 4 | ST-04: DLQ 真实 5 域 | 模拟 5 域 down + kubectl | retry 3 次 + DLQ + 恢复 |
| 5 | ST-05: mTLS 业务级 | grpcurl --cert --key | 5 域 gRPC mTLS 双向认证 |
| 6 | ST-06: 数据迁移 + rollback | psql + curl | 迁移执行 + rollback SQL 验证 |
| 7 | ST-07: 审计永久保留 | psql + curl | audit_event 不被自动清理 |
| 8 | ST-08: envoy 边缘代理 | curl https://localhost:8443 | 8443 HTTPS + mTLS termination |
| 9 | ST-09: 127.0.0.1 only | curl 0.0.0.0:8789 | 0.0.0.0 拒绝（envoy 边缘代理）|
| 10 | ST-10: env value 永不出现在日志 | kubectl logs + grep | kubectl logs 无 password / key / token 字符串 |

### 7.4 测试覆盖率目标

- UT 单测覆盖率 ≥ 80%
- IT 集成测试 8 个场景全过
- ST 系统测试 10 个场景全过
- 总测试数目标 ≥ 100 (UT 50+ + IT 30+ + ST 20+)

### 7.5 性能基准测试

- 5 worker 池跑 1K 子任务 ≤ 30s (per PF-5)
- 5K 子任务 ≤ 5min (per PF-6)
- 100K 子任务 ≤ 1h (per PF-7)
- 5 域 gRPC p95 < 200ms (per PF-10)

---

## 8. 关联文档与演进

### 8.1 文档三件套

| 文档 | 状态 | 备注 |
|---|---|---|
| RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 | ✅ 已落地 (commit `fd122f6`) | 本文之上游 |
| RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1 | ✅ 已落地 (commit `e366ff8`) | 本文之上游 |
| **RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1** | **✅ 本文 (待 commit)** | 终点 |
| RGS-BATCH-PLAN-2026-09-01 v0.1 | ⏳ 待起草 | 4-6 周实施计划 |

### 8.2 与上游规范关系 (5 不破坏 + 4 复用 + 3 引用)

**(5 不破坏)**：
- 不破坏 5 域架构: rgs-batch-backend 作为 gRPC 客户端调用 5 域, 不修改 5 域代码
- 不破坏 rgs-web: rgs-batch-console 独立 Node 项目, 不嵌入 rgs-web
- 不破坏 shared-platform: 复用现有 crate, 不修改 shared-platform 代码
- 不破坏 function-plane: v0.1 不集成 (saga-runtime 独立 Pod per RGS-BAS-100), 不修改 function-plane 代码
- 不破坏 gm-backend: rgs-batch-console 跟 gm-console 形态不同, 但都是 envoy 独立 deployment

**(4 复用)**：
- rgs-web 母规范 5 份: 0 依赖 + 127.0.0.1 only + 30s 轮询 + JSON 响应
- rgs-web OLU-WEB 4 份: data/ 目录 + lockfile + token-estimate + ai-ledger.jsonl
- gm-backend 范式: actix-web + mTLS + 8443 HTTPS APIGW
- 5 域 ST 业务级 mTLS 实践 (commit 401ac5c): 证书 + 双向认证 + 8/27 ST 导出 SOP

**(3 引用)**：
- shared-platform 20 模块: outbox + tracing + span_helpers + retry + dlq + grpc_tracing + rbac + tls + ...
- 5 域 gRPC client: player / economy / match / social / admin 50051-50055
- saga-runtime 独立 Pod (per RGS-BAS-100 v0.1, v0.2 集成)

### 8.3 演进路径 (3 阶段)

| 阶段 | 内容 | 预计时间 |
|---|---|---|
| **v0.1 (本文 + REQ + BASIC + PLAN)** | 基础框架 + GM 批量 + 定时调度 + 失败重试 + DLQ + 任务监控 + 审计 + DB 16 表 + 11 类运维 + 10 类 ST | 4-6 周 |
| **v0.2** | Log 批量 + 数据整理 + 跨 batch DAG + WebSocket + mavis cron 告警 + AI 协作 + rgs-web 深联动 + 证书轮换 + 任务超时强制 kill + dry-run | 6-10 周 |
| **v0.3** | 商业化 / 多租户 / 流式 / provider token counter / k3s HPA 自动扩缩 | 待 v0.2 评估 |

---

## 9. 验收者

(per 2026-08-26 08:40 JST 代签新规则, 同 REQUIREMENTS §11 + BASIC §7)

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

## 10. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (**Mavis 接手 agent per DEC-008**) | 首版: 0 文档定位 + 1 API 详细签名 (11 console + 26 backend = 37 endpoint) + 2 数据模型 (19 migration + 18 索引 + 8 保留 + 6 FK/CHECK/UNIQUE + 5 写并发) + 3 部署 (9 manifests + console deployment + backend deployment + envoy deployment + mTLS cert + NetworkPolicy + 9 步启动 SOP) + 4 运维 (10 监控项 + 10 故障恢复 + 7 数据备份 + 8 运维脚本) + 5 安全 (4 凭据 + 强校验代码 + 4 写并发 + 6 数据隐私 + 127.0.0.1 + mTLS + audit 永久) + 6 性能 (15 PF 指标) + 7 测试策略 (11 UT + 8 IT + 10 ST + 覆盖率 + 性能基准) + 8 关联演进 (4 文档 + 5 不破坏 + 4 复用 + 3 引用 + 3 演进阶段) + 9 验收者 (10 签字栏) + 10 修订历史 |

**修订人**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师 (**Mavis 接手 agent per DEC-008**)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化
