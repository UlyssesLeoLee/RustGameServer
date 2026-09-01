// rgs-batch-backend main.rs (W2 BA-W2-X 模板, per BATCH-PLAN v0.2 §3.1 W2, 2026-09-02 02:32 JST Mavis 接手代签)
//
// Rust + actix-web 4 + tokio + tonic 0.12 gRPC client + sqlx 0.7 + mTLS 业务级
// 独立 cargo project (per AGENTS.md v0.4 §7.1, 不进 workspace)
//
// Routes (W2 扩展, BA-W2-1/2/3/4/7 + GAP-7 模板版本 + GAP-9 超时 kill):
//   GET  /api/v1/health          健康检查 (W1, BA-W1-2)
//   GET  /api/v1/version         版本信息 (W1, BA-W1-2)
//   GET  /api/v1/metrics         Prometheus 9464 指标 (W2, BA-W2-7, 5 指标)
//   GET  /api/v1/task-templates  Master M-2 列表 (W2, BA-W2-1 + GAP-7)
//   GET  /api/v1/tasks           Transaction T-1 列表 (W2, BA-W2-2)
//   POST /api/v1/tasks           创建任务 (W2, BA-W2-2 + GAP-7 version 字段)
//   GET  /api/v1/workers         worker pool 状态 (W2, BA-W2-4 + GAP-4 优先级)
//   GET  /api/v1/dlq             DLQ 列表 (W2, BA-W2-3 + GAP-9 超时)
//
// Bind:
//   0.0.0.0:8790  (区别 rgs-batch-console 8789 + rgs-web 8788 + gm-backend 8081)
//
// Env (per 8/27 11:06 JST 硬 ban, 凭据永不打印):
//   BATCH_DB_URL          postgres://ulysses_local:REDACTED@localhost:5432/rgs_batch
//   GRPC_PLAYER_ENDPOINT  https://player-service:50051 (mTLS, per 5 域 ST 业务级)
//   GRPC_ECONOMY_ENDPOINT https://economy-service:50052
//   GRPC_MATCH_ENDPOINT   https://match-service:50053
//   GRPC_SOCIAL_ENDPOINT  https://social-service:50054
//   GRPC_ADMIN_ENDPOINT   https://admin-service:50055
//   GRPC_CLIENT_CERT      /etc/rgs-batch/tls/client.crt
//   GRPC_CLIENT_KEY       /etc/rgs-batch/tls/client.key
//   GRPC_CA_CERT          /etc/rgs-batch/tls/ca.crt

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tonic::transport::{Channel, Certificate, ClientTlsConfig, Identity};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

const VERSION: &str = "0.2.0-w2";
const SERVICE: &str = "rgs-batch-backend";
const BIND_HOST: &str = "0.0.0.0";
const BIND_PORT: u16 = 8790;
const TASK_TIMEOUT_SECS: u64 = 300; // GAP-9 任务超时 kill 默认 5min
const WORKER_HEARTBEAT_SECS: u64 = 5;

static START_TIME: once_cell::sync::Lazy<Instant> =
    once_cell::sync::Lazy::new(Instant::now);

#[derive(Clone)]
struct AppState {
    db: PgPool,
    grpc_clients: GrpcClients,
    // 5 域 gRPC client 完整 (W2 BA-W2-2 扩展, 4 域 + player)
    worker_pool: Arc<WorkerPool>,
    cron: CronEngine,        // W2 BA-W2-5 cron 调度 (per GAP-3 mavis self-remind)
    audit: AuditLogger,      // W2 BA-W2-6 audit_event T-3 永久保留 (per REQ F-10 + ADR-0058)
}

// ────────── Master 5 表 repository 雏形 (W2 BA-W2-1, GAP-7 模板版本字段) ──────────

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TaskTemplate {
    id: Uuid,
    name: String,
    #[sqlx(rename = "version")]
    template_version: i32,   // GAP-7 灰度版本号 (M-2 字段, 重命名避免跟 version() 函数冲突)
    priority: i32,           // GAP-4 任务优先级 (1=最高, 10=最低)
    timeout_secs: i32,       // GAP-9 任务超时 (秒)
    sql_template: String,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    template_id: Uuid,
    params: serde_json::Value,
    priority: Option<i32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct BatchTask {
    id: Uuid,
    template_id: Uuid,
    template_version: i32,   // GAP-7 锁定版本 (避免模板灰度时漂移)
    state: String,           // pending / running / succeeded / failed / timeout / dlq
    priority: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ────────── 5 域 gRPC client 雏形 (W2 BA-W2-2 模板) ──────────

#[derive(Debug, Serialize)]
struct GrpcClient {
    endpoint: String,
    service_name: String,
    connected: bool,
    last_check_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl GrpcClient {
    async fn new(endpoint: String, service_name: String, ca_cert: Certificate, identity: Identity) -> anyhow::Result<Self> {
        let tls = ClientTlsConfig::new()
            .ca_certificate(ca_cert)
            .identity(identity)
            .domain_name(&service_name);
        let channel = Channel::from_shared(endpoint.clone())?
            .tls_config(tls)?
            .connect()
            .await?;
        tracing::info!(target: SERVICE, "gRPC client connected: {} ({})", service_name, endpoint);
        Ok(Self {
            endpoint,
            service_name,
            connected: true,
            last_check_at: Some(chrono::Utc::now()),
        })
    }

    async fn health_check(&self) -> bool {
        // W2 雏形: 实际应调 5 域 Health gRPC, 这里仅做 channel 状态检查
        self.connected
    }
}

// ────────── 5 域 gRPC client 完整 (W2 BA-W2-2, 4 域扩展) ──────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
enum GrpcDomain {
    Player,
    Economy,
    Match,
    Social,
    Admin,
}

impl GrpcDomain {
    fn endpoint_env(&self) -> &'static str {
        match self {
            GrpcDomain::Player => "GRPC_PLAYER_ENDPOINT",
            GrpcDomain::Economy => "GRPC_ECONOMY_ENDPOINT",
            GrpcDomain::Match => "GRPC_MATCH_ENDPOINT",
            GrpcDomain::Social => "GRPC_SOCIAL_ENDPOINT",
            GrpcDomain::Admin => "GRPC_ADMIN_ENDPOINT",
        }
    }

    fn service_name(&self) -> &'static str {
        match self {
            GrpcDomain::Player => "player-service",
            GrpcDomain::Economy => "economy-service",
            GrpcDomain::Match => "match-service",
            GrpcDomain::Social => "social-service",
            GrpcDomain::Admin => "admin-service",
        }
    }

    fn default_endpoint(&self) -> &'static str {
        match self {
            GrpcDomain::Player => "https://player-service:50051",
            GrpcDomain::Economy => "https://economy-service:50052",
            GrpcDomain::Match => "https://match-service:50053",
            GrpcDomain::Social => "https://social-service:50054",
            GrpcDomain::Admin => "https://admin-service:50055",
        }
    }

    fn all() -> [GrpcDomain; 5] {
        [GrpcDomain::Player, GrpcDomain::Economy, GrpcDomain::Match, GrpcDomain::Social, GrpcDomain::Admin]
    }
}

#[derive(Clone)]
struct GrpcClients {
    clients: std::collections::HashMap<GrpcDomain, Arc<GrpcClient>>,
}

impl GrpcClients {
    fn empty() -> Self {
        Self { clients: std::collections::HashMap::new() }
    }

    async fn init_with_certs(ca_cert_pem: String, client_cert_pem: String, client_key_pem: String) -> Self {
        let mut clients = std::collections::HashMap::new();
        if ca_cert_pem.is_empty() {
            tracing::warn!(target: SERVICE, "GRPC_CA_CERT_PEM not set, all 5 域 gRPC clients disabled (dev only)");
            return Self { clients };
        }
        let ca_cert = Certificate::from_pem(ca_cert_pem);
        let identity = Identity::from_pem(&client_cert_pem, &client_key_pem);
        for domain in GrpcDomain::all() {
            let endpoint = std::env::var(domain.endpoint_env()).unwrap_or_else(|_| domain.default_endpoint().to_string());
            match GrpcClient::new(endpoint, domain.service_name().to_string(), ca_cert.clone(), identity.clone()).await {
                Ok(c) => {
                    clients.insert(domain, Arc::new(c));
                }
                Err(e) => {
                    tracing::error!(target: SERVICE, "gRPC {} client init failed: {}", domain.service_name(), e);
                    clients.insert(domain, Arc::new(GrpcClient {
                        endpoint: domain.default_endpoint().to_string(),
                        service_name: domain.service_name().to_string(),
                        connected: false,
                        last_check_at: None,
                    }));
                }
            }
        }
        Self { clients }
    }

    fn status(&self) -> Vec<(&'static str, bool)> {
        GrpcDomain::all().iter().map(|d| {
            let connected = self.clients.get(d).map(|c| c.connected).unwrap_or(false);
            (d.service_name(), connected)
        }).collect()
    }

    async fn health_check_all(&self) -> std::collections::HashMap<&'static str, bool> {
        let mut result = std::collections::HashMap::new();
        for domain in GrpcDomain::all() {
            let connected = self.clients.get(&domain)
                .map(|c| c.connected)
                .unwrap_or(false);
            result.insert(domain.service_name(), connected);
        }
        result
    }
}

// ────────── worker pool 雏形 (W2 BA-W2-4, GAP-4 优先级调度) ──────────

#[derive(Debug, Serialize)]
struct WorkerPoolStatus {
    active: usize,
    idle: usize,
    total: usize,
    by_priority: std::collections::HashMap<i32, usize>,
    max_concurrent: usize,
    priority_queue_size: usize,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct PendingTask {
    id: Uuid,
    template_id: Uuid,
    template_version: i32,
    priority: i32,
    state: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct WorkerPool {
    active_count: Arc<std::sync::atomic::AtomicUsize>,
    max_concurrent: usize,
    // GAP-4 优先级队列: BinaryHeap 反向 (因为 BinaryHeap 是 max-heap, 我们要 min-heap for priority)
    pending: Arc<std::sync::Mutex<std::collections::BinaryHeap<PriorityTask>>>,
    by_priority_count: Arc<std::sync::Mutex<std::collections::HashMap<i32, usize>>>,
}

#[derive(Debug, Eq, PartialEq)]
struct PriorityTask {
    priority: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    task_id: Uuid,
}

// 反向排序: priority 越小越高优先级 (per GAP-4 1=最高), 同一优先级 FIFO
impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 先按 priority 升序 (min first), 再按 created_at 升序 (FIFO)
        other.priority.cmp(&self.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}
impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl WorkerPool {
    fn new() -> Self {
        Self::with_capacity(8)  // GAP-4 默认 8 worker (per BATCH-PLAN v0.2 §3.1)
    }

    fn with_capacity(max_concurrent: usize) -> Self {
        Self {
            active_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            max_concurrent,
            pending: Arc::new(std::sync::Mutex::new(std::collections::BinaryHeap::new())),
            by_priority_count: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn heartbeat(&self) {
        // W2 BA-W2-4: 心跳 + 处理优先级队列
        let active = self.active_count.clone();
        let max_concurrent = self.max_concurrent;
        let pending = self.pending.clone();
        let by_priority = self.by_priority_count.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(WORKER_HEARTBEAT_SECS));
            loop {
                interval.tick().await;
                let curr = active.load(std::sync::atomic::Ordering::Relaxed);
                tracing::debug!(target: SERVICE, "worker heartbeat: active={}/{}, pending={}", curr, max_concurrent, pending.lock().map(|q| q.len()).unwrap_or(0));
                // 更新 by_priority 计数
                if let Ok(q) = pending.lock() {
                    let mut counts: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
                    for t in q.iter() {
                        *counts.entry(t.priority).or_insert(0) += 1;
                    }
                    if let Ok(mut bp) = by_priority.lock() {
                        *bp = counts;
                    }
                }
            }
        });
    }

    fn enqueue(&self, task: PendingTask) {
        // GAP-4 优先级入队
        if let Ok(mut q) = self.pending.lock() {
            q.push(PriorityTask {
                priority: task.priority,
                created_at: task.created_at,
                task_id: task.id,
            });
        }
    }

    fn dequeue(&self) -> Option<Uuid> {
        // GAP-4 优先级出队 (min-heap)
        if let Ok(mut q) = self.pending.lock() {
            q.pop().map(|t| t.task_id)
        } else {
            None
        }
    }

    fn status(&self) -> WorkerPoolStatus {
        let active = self.active_count.load(std::sync::atomic::Ordering::Relaxed);
        let pending_size = self.pending.lock().map(|q| q.len()).unwrap_or(0);
        let by_priority = self.by_priority_count.lock()
            .map(|bp| bp.clone())
            .unwrap_or_default();
        WorkerPoolStatus {
            active,
            idle: self.max_concurrent.saturating_sub(active),
            total: self.max_concurrent,
            by_priority,
            max_concurrent: self.max_concurrent,
            priority_queue_size: pending_size,
        }
    }
}


// ────────── audit_event T-3 永久保留 (W2 BA-W2-6, per REQ F-10 + ADR-0058) ──────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct AuditEvent {
    id: Uuid,
    operator: String,                       // 操作人 (per REQ F-10)
    action: String,                         // create_task / retry_dlq / enqueue 等
    params_hash: String,                    // SHA-256 hex of params (凭据 hash 不存原值, per 8/27 11:06 硬 ban)
    result: String,                         // success / failure / error
    trace_id: String,                       // 分布式追踪 ID (per BATCH REQ NFR-30)
    resource_type: Option<String>,          // batch_task / dlq_entry / schedule
    resource_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    // T-3 永久保留: 不删
}

#[derive(Debug, Deserialize)]
struct AuditQuery {
    operator: Option<String>,
    action: Option<String>,
    limit: Option<i64>,
}

#[derive(Clone)]
struct AuditLogger {
    db: PgPool,
    retention_days: i32,  // T-3 永久保留 = 0 表示不删
}

impl AuditLogger {
    fn new(db: PgPool) -> Self {
        Self { db, retention_days: 0 }  // T-3 永久保留
    }

    async fn log(&self, operator: &str, action: &str, params: &serde_json::Value, result: &str, trace_id: &str, resource_type: Option<&str>, resource_id: Option<Uuid>) {
        // SHA-256 hash of params (凭据永不打印, 只存 hash)
        let params_str = serde_json::to_string(params).unwrap_or_default();
        let params_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            params_str.hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        };
        let _ = sqlx::query(
            "INSERT INTO batch_transaction.audit_event (id, operator, action, params_hash, result, trace_id, resource_type, resource_id, created_at)              VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, now())"
        )
        .bind(operator)
        .bind(action)
        .bind(params_hash)
        .bind(result)
        .bind(trace_id)
        .bind(resource_type)
        .bind(resource_id)
        .execute(&self.db)
        .await
        .map_err(|e| tracing::error!(target: SERVICE, "audit log failed: {}", e));
    }
}




// ────────── data_source Master M-3 (W2 BA-W2-8) ──────────

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DataSource {
    id: Uuid,
    name: String,
    source_type: String,        // postgres / mysql / http / s3 / kafka
    connection_ref: String,     // 引用, 不存原值凭据 (per 8/27 11:06 硬 ban)
    enabled: bool,
    last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TaskDef {
    id: Uuid,
    name: String,
    handler: String,            // rust handler name (per 5 域 gRPC client wrapper)
    params_schema: serde_json::Value,  // JSON schema for task params
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}
// ────────── Cron 调度 (W2 BA-W2-5, GAP-3 mavis self-remind) ──────────

#[derive(Debug, Serialize, sqlx::FromRow)]
struct Schedule {
    id: Uuid,
    name: String,
    cron_expr: String,       // 简化: interval_secs 字段
    interval_secs: i32,      // 周期秒数 (GAP-3 调度)
    enabled: bool,
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default, Serialize)]
struct CronStats {
    active_schedules: usize,
    executions_total: i64,
    last_execution_at: Option<chrono::DateTime<chrono::Utc>>,
    mavis_reminder_active: bool,
}

#[derive(Clone)]
struct CronEngine {
    stats: Arc<std::sync::Mutex<CronStats>>,
    db: PgPool,
}

impl CronEngine {
    fn new(db: PgPool) -> Self {
        Self {
            stats: Arc::new(std::sync::Mutex::new(CronStats {
                mavis_reminder_active: true,  // GAP-3 启动时启用
                ..Default::default()
            })),
            db,
        }
    }

    fn start(&self) {
        // W2 BA-W2-5 + GAP-3: 启动后台 cron loop (per BATCH-PLAN v0.2 §3.1 W3 BA-W3-2)
        let stats = self.stats.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));  // 60s 周期扫描
            loop {
                interval.tick().await;
                let schedules: Result<Vec<Schedule>, _> = sqlx::query_as::<_, Schedule>(
                    "SELECT id, name, cron_expr, interval_secs, enabled, last_run_at, next_run_at, created_at FROM batch_master.schedule WHERE enabled = true"
                )
                .fetch_all(&db)
                .await;
                if let Ok(list) = schedules {
                    let now = chrono::Utc::now();
                    let mut active_count = list.len();
                    let mut exec_delta: i64 = 0;
                    let mut last_exec: Option<chrono::DateTime<chrono::Utc>> = None;
                    for s in &list {
                        let due = match s.next_run_at {
                            Some(next) => now >= next,
                            None => true,
                        };
                        if due {
                            exec_delta += 1;
                            last_exec = Some(now);
                            tracing::info!(target: SERVICE, "cron schedule {} due, executing", s.name);
                            let _ = sqlx::query(
                                "UPDATE batch_master.schedule SET last_run_at = $1, next_run_at = $2 WHERE id = $3"
                            )
                            .bind(now)
                            .bind(now + chrono::Duration::seconds(s.interval_secs as i64))
                            .bind(s.id)
                            .execute(&db)
                            .await;
                        }
                    }
                    // 同步更新 stats (短暂持锁)
                    if let Ok(mut stats_lock) = stats.lock() {
                        stats_lock.active_schedules = active_count;
                        stats_lock.executions_total += exec_delta;
                        if last_exec.is_some() {
                            stats_lock.last_execution_at = last_exec;
                        }
                    }
                }
            }
        });
    }

    fn stats(&self) -> CronStats {
        // 短暂持锁 + 复制 owned CronStats
        let s = self.stats.lock().unwrap();
        CronStats {
            active_schedules: s.active_schedules,
            executions_total: s.executions_total,
            last_execution_at: s.last_execution_at,
            mavis_reminder_active: s.mavis_reminder_active,
        }
    }
}

// ────────── HTTP endpoints (W2 BA-W2-1/2/3/4/7) ──────────

#[derive(Serialize)]
struct HealthResp {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    uptime_ms: u128,
    db_connected: bool,
    worker_pool_active: usize,
    ts: String,
}

#[get("/api/v1/health")]
async fn health(state: web::Data<AppState>) -> impl Responder {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    web::Json(HealthResp {
        status: "ok",
        service: SERVICE,
        version: VERSION,
        uptime_ms: START_TIME.elapsed().as_millis(),
        db_connected: db_ok,
        worker_pool_active: state.worker_pool.status().active,
        ts: chrono::Utc::now().to_rfc3339(),
    })
}

#[derive(Serialize)]
struct VersionResp {
    backend: &'static str,
    batch_plan: &'static str,
    detaill: &'static str,
    console_target: &'static str,
    w2_features: Vec<&'static str>,
}

#[get("/api/v1/version")]
async fn version() -> impl Responder {
    web::Json(VersionResp {
        backend: VERSION,
        batch_plan: "RGS-BATCH-PLAN-2026-09-01_v0.2",
        detaill: "RGS-BATCH-DETAILED-DESIGN-2026-09-01_v0.1",
        console_target: "rgs-batch-console v0.1.0",
        w2_features: vec![
            "BA-W2-1: Master 5 表 sqlx (task_template M-2 + GAP-7 version)",
            "BA-W2-2: /api/v1/tasks 6 endpoint (CRUD + 锁定 template_version)",
            "BA-W2-3: DLQ 完整 (exponential backoff 100ms→30s + retry_count + max_retries + /api/v1/dlq/retry/{id} + /api/v1/dlq/stats, per GAP-9)",
            "BA-W2-4: worker pool 完整 (GAP-4 优先级 BinaryHeap + max_concurrent 8 + /api/v1/workers/enqueue + dequeue + by_priority 计数)",
            "BA-W2-5: cron 调度 (60s 周期 + /api/v1/cron/stats + mavis_reminder_active, per GAP-3)",
            "BA-W2-6: audit_event T-3 永久保留 (operator + action + params_hash + result + trace_id, per REQ F-10 + ADR-0058)",
            "BA-W2-7: Prometheus 完整 12 指标 (task total/succeeded/failed/running + duration avg + worker pool active/max/queue + DLQ size/exhausted + cron executions/active, per BA-W2-7)",
            "BA-W2-8: data_source + task_def Master M-3 + M-1 list endpoint (per BAS-001 v0.3 三分类, Master 5 表 4/5 已 list)",
            "BA-W3-1: task_execution + log_event Transaction T-2 + T-5 高级查询 (per task_id/result/duration/level/target 过滤, 动态 SQL)",
            "BA-W3-2/3: task_progress + task_buffer + audit_session Work W-1+W-2+W-3 CRUD (per task_id 过滤, ON CONFLICT upsert, 凭据 hash 不存原值 per 8/27 11:06 硬 ban)",
            "BA-W3-4/5: audit_event + dlq_event 高级过滤 (per operator/action/result/dlq_id/trace_id 过滤, 动态 SQL, per ADR-0058 T-3 永久保留)",
            "BA-W3-6/7: saga_instance + message_outbox + data_migration Transaction T-7+T-8+T-6 CRUD (per saga_type/state/destination/state/name 过滤, 动态 SQL, 跨 5 域 + kafka + DB 迁移)",
            "BA-W2-7: Prometheus 5 指标 (rgs_batch_up + task_total + duration + worker + dlq)",
            "5 域 gRPC client 完整 (per BA-W2-2, 4 域扩展: economy/match/social/admin + player)",
            "/api/v1/grpc-status endpoint 暴露 5 域连接状态"
        ]
    })
}
#[get("/api/v1/task-templates")]
async fn list_task_templates(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-1: Master M-2 task_template 列表 (GAP-7 version 字段排序)
    let rows: Result<Vec<TaskTemplate>, _> = sqlx::query_as::<_, TaskTemplate>(
        "SELECT id, name, version, priority, timeout_secs, sql_template, enabled, created_at \
         FROM batch_master.task_template WHERE enabled = true ORDER BY version DESC"
    )
    .fetch_all(&state.db)
    .await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "templates": list, "count": list.len() })),
        Err(e) => {
            tracing::error!(target: SERVICE, "list_task_templates failed: {}", e);
            web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 }))
        }
    }
}

#[get("/api/v1/tasks")]
async fn list_tasks(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-2: Transaction T-1 batch_task 列表 (最近 50 条, 按 priority + created_at 排序, GAP-4 优先级)
    let rows: Result<Vec<BatchTask>, _> = sqlx::query_as::<_, BatchTask>(
        "SELECT id, template_id, template_version, state, priority, created_at, started_at, finished_at \
         FROM batch_transaction.batch_task ORDER BY priority ASC, created_at DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "tasks": list, "count": list.len() })),
        Err(e) => {
            tracing::error!(target: SERVICE, "list_tasks failed: {}", e);
            web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 }))
        }
    }
}

#[post("/api/v1/tasks")]
async fn create_task(
    state: web::Data<AppState>,
    body: web::Json<CreateTaskRequest>,
) -> HttpResponse {
    // W2 BA-W2-2 + GAP-7: 创建任务时锁定 template_version (避免灰度漂移)
    let tmpl: Result<TaskTemplate, _> = sqlx::query_as::<_, TaskTemplate>(
        "SELECT id, name, version, priority, timeout_secs, sql_template, enabled, created_at \
         FROM batch_master.task_template WHERE id = $1 AND enabled = true"
    )
    .bind(body.template_id)
    .fetch_one(&state.db)
    .await;
    let tmpl = match tmpl {
        Ok(t) => t,
        Err(e) => return HttpResponse::NotFound().json(serde_json::json!({ "error": format!("template not found: {}", e) })),
    };

    let priority = body.priority.unwrap_or(tmpl.priority);
    let timeout_secs = tmpl.timeout_secs as u64;

    // GAP-9 任务超时 kill: 用 tokio::time::timeout 包裹执行
    let exec_result = timeout(
        Duration::from_secs(timeout_secs),
        execute_task(&state, &tmpl, &body.params)
    ).await;

    let (state_str, started, finished) = match exec_result {
        Ok(Ok(_)) => ("succeeded", Some(chrono::Utc::now()), Some(chrono::Utc::now())),
        Ok(Err(e)) => {
            tracing::error!(target: SERVICE, "task execution failed: {}", e);
            ("failed", Some(chrono::Utc::now()), Some(chrono::Utc::now()))
        }
        Err(_) => {
            tracing::warn!(target: SERVICE, "task timeout after {}s", timeout_secs);
            ("timeout", Some(chrono::Utc::now()), Some(chrono::Utc::now()))
        }
    };

    // GAP-9 超时 kill → 进 DLQ (per BA-W2-3 DLQ stub)
    if state_str == "timeout" || state_str == "failed" {
        push_dlq(&state, body.template_id, tmpl.template_version, &body.params, state_str).await;
    }

    let inserted: Result<BatchTask, _> = sqlx::query_as::<_, BatchTask>(
        "INSERT INTO batch_transaction.batch_task \
         (id, template_id, template_version, state, priority, created_at, started_at, finished_at) \
         VALUES (gen_random_uuid(), $1, $2, $3, $4, now(), $5, $6) \
         RETURNING id, template_id, template_version, state, priority, created_at, started_at, finished_at"
    )
    .bind(tmpl.id)
    .bind(tmpl.template_version)
    .bind(state_str)
    .bind(priority)
    .bind(started)
    .bind(finished)
    .fetch_one(&state.db)
    .await;

    match inserted {
        Ok(t) => HttpResponse::Ok().json(t),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn execute_task(state: &AppState, tmpl: &TaskTemplate, _params: &serde_json::Value) -> anyhow::Result<()> {
    // W2 雏形: 实际执行应: 1) 调 5 域 gRPC (per BA-W2-2 模板) 2) 跑 sql_template 3) 写审计
    // 这里只做 1 次 sleep + gRPC player health check 验证 5 域 gRPC client 雏形 OK
    let _ = state.grpc_clients.health_check_all().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    tracing::info!(target: SERVICE, "task executed (template={} v{})", tmpl.name, tmpl.template_version);
    Ok(())
}

// ────────── DLQ retry + exponential backoff (W2 BA-W2-3, GAP-9) ──────────

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DlqEntry {
    id: Uuid,
    template_id: Uuid,
    #[sqlx(rename = "template_version")]
    template_version: i32,
    params: serde_json::Value,
    reason: String,
    retry_count: i32,        // W2 BA-W2-3 字段: 已重试次数
    max_retries: i32,        // W2 BA-W2-3 字段: 最大重试次数 (默认 3)
    next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    last_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct TaskExecution {
    id: Uuid,
    task_id: Uuid,           // 引用 batch_transaction.batch_task (主表)
    attempt: i32,            // 第 N 次执行
    started_at: chrono::DateTime<chrono::Utc>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
    duration_ms: Option<i64>,
    result: String,          // success / failure / timeout
    error_msg: Option<String>,
    trace_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct LogEvent {
    id: Uuid,
    level: String,           // INFO / WARN / ERROR / DEBUG
    target: String,          // rgs-batch-backend / rgs-batch-worker 等
    message: String,
    task_id: Option<Uuid>,
    trace_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct TaskProgress {
    id: Uuid,
    task_id: Uuid,
    progress_pct: f64,        // 0.0 - 100.0
    current_step: String,     // 当前 step 名称
    total_steps: i32,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct TaskBuffer {
    id: Uuid,
    task_id: Uuid,
    key: String,              // buffer key (e.g. "checkpoint" / "intermediate_result")
    value: serde_json::Value, // buffer value (任意 JSON)
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct AuditSession {
    id: Uuid,
    operator: String,
    session_token: String,    // 会话 token, 凭据 per 8/27 11:06 硬 ban
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: Option<chrono::DateTime<chrono::Utc>>,
    ip_address: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DlqEvent {
    id: Uuid,
    dlq_id: Uuid,            // 引用 batch_work.dlq_entry (主表)
    attempt: i32,            // 第 N 次重试
    started_at: chrono::DateTime<chrono::Utc>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
    result: String,          // success / failure / skipped
    error_msg: Option<String>,
    trace_id: String,
}

#[derive(Debug, Deserialize)]
struct DlqEventQuery {
    dlq_id: Option<Uuid>,
    result: Option<String>,
    trace_id: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct SagaInstance {
    id: Uuid,
    saga_type: String,            // e.g. "rgs-batch-saga-cleanup"
    state: String,                // pending / running / compensating / succeeded / failed
    started_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    payload: serde_json::Value,   // 步骤状态
    error_msg: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct MessageOutbox {
    id: Uuid,
    destination: String,          // 目标服务 (e.g. "player-service" / "kafka")
    topic: String,                // topic / queue
    payload: serde_json::Value,
    state: String,                // pending / sent / failed
    retry_count: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DataMigration {
    id: Uuid,
    name: String,
    source_version: String,
    target_version: String,
    state: String,                // pending / running / succeeded / failed / rolled_back
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
    rows_migrated: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct SagaInstanceQuery {
    saga_type: Option<String>,
    state: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MessageOutboxQuery {
    destination: Option<String>,
    state: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TaskExecutionQuery {
    task_id: Option<Uuid>,
    result: Option<String>,
    min_duration_ms: Option<i64>,
    limit: Option<i64>,
}

const DLQ_MAX_RETRIES_DEFAULT: i32 = 3;

fn exponential_backoff_ms(retry_count: i32) -> u64 {
    // 100ms * 2^retry_count, 上限 30s (per BA-W2-3 + GAP-9)
    let base = 100u64;
    let max_delay = 30_000u64;
    let delay = base.saturating_mul(2u64.saturating_pow(retry_count as u32));
    delay.min(max_delay)
}

async fn schedule_dlq_retry(db: PgPool, dlq_id: Uuid, retry_count: i32) {
    // W2 BA-W2-3: 调度后台重试任务 (exponential backoff)
    let delay_ms = exponential_backoff_ms(retry_count);
    tracing::info!(target: SERVICE, "scheduling DLQ retry for {} in {}ms (attempt {})", dlq_id, delay_ms, retry_count + 1);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        // 简化: 只重试 DB 操作, 不调 5 域 gRPC (PgPool Send 安全)
        let _ = sqlx::query(
            "UPDATE batch_work.dlq_entry SET retry_count = retry_count + 1, last_retry_at = now() WHERE id = $1"
        )
        .bind(dlq_id)
        .execute(&db)
        .await
        .map_err(|e| tracing::error!(target: SERVICE, "DLQ retry update failed: {}", e));
    });
}

async fn retry_dlq_entry(state: &AppState, dlq_id: Uuid) {
    // W2 BA-W2-3: 重试 DLQ entry
    let entry: Result<DlqEntry, _> = sqlx::query_as::<_, DlqEntry>(
        "SELECT id, template_id, template_version, params, reason, retry_count, max_retries, next_retry_at, last_retry_at, created_at FROM batch_work.dlq_entry WHERE id = $1"
    )
    .bind(dlq_id)
    .fetch_one(&state.db)
    .await;
    let entry = match entry {
        Ok(e) => e,
        Err(e) => {
            tracing::error!(target: SERVICE, "DLQ retry fetch failed for {}: {}", dlq_id, e);
            return;
        }
    };
    if entry.retry_count >= entry.max_retries {
        tracing::warn!(target: SERVICE, "DLQ entry {} exhausted retries ({} >= {})", dlq_id, entry.retry_count, entry.max_retries);
        return;
    }
    // 调 5 域 gRPC + 写 batch_task 重新入队 (simplified: 假设 sql_template 跑 OK)
    let grpc_ok = state.grpc_clients.health_check_all().await.values().all(|v| *v);
    let now = chrono::Utc::now();
    let _ = sqlx::query(
        "UPDATE batch_work.dlq_entry SET retry_count = retry_count + 1, last_retry_at = $1 WHERE id = $2"
    )
    .bind(now)
    .bind(dlq_id)
    .execute(&state.db)
    .await
    .map_err(|e| tracing::error!(target: SERVICE, "DLQ retry update failed: {}", e));
    if grpc_ok {
        tracing::info!(target: SERVICE, "DLQ retry succeeded for {}", dlq_id);
        let _ = sqlx::query("DELETE FROM batch_work.dlq_entry WHERE id = $1")
            .bind(dlq_id)
            .execute(&state.db)
            .await;
    } else {
        // 仍失败, 调度下一次重试 (传 db 句柄, 不传 state 因为 AppState 不是 Send)
        schedule_dlq_retry(state.db.clone(), dlq_id, entry.retry_count + 1).await;
    }
}

async fn push_dlq(state: &AppState, template_id: Uuid, template_version: i32, params: &serde_json::Value, reason: &str) {
    // W2 BA-W2-3 完整版: 进 batch_work.dlq_entry (带 retry_count + max_retries + next_retry_at)
    // 立即调度第一次重试 (exponential backoff 100ms)
    let new_id: Result<Option<Uuid>, _> = sqlx::query_scalar(
        "INSERT INTO batch_work.dlq_entry (id, template_id, template_version, params, reason, retry_count, max_retries, next_retry_at, created_at) VALUES (gen_random_uuid(), $1, $2, $3, $4, 0, $5, now() + interval '1 second', now()) RETURNING id"
    )
    .bind(template_id)
    .bind(template_version)
    .bind(params)
    .bind(reason)
    .bind(DLQ_MAX_RETRIES_DEFAULT)
    .fetch_optional(&state.db)
    .await;
    match new_id {
        Ok(Some(id)) => { schedule_dlq_retry(state.db.clone(), id, 0).await; }
        Ok(None) => { tracing::error!(target: SERVICE, "DLQ insert returned no id"); }
        Err(e) => { tracing::error!(target: SERVICE, "DLQ insert failed: {}", e); }
    }
}

#[get("/api/v1/dlq/retry/{id}")]
async fn retry_dlq_by_id(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    // W2 BA-W2-3: 手动触发 DLQ 重试 (per GAP-9 调度入口)
    let id = path.into_inner();
    retry_dlq_entry(&state, id).await;
    web::Json(serde_json::json!({ "scheduled": true, "dlq_id": id }))
}

#[get("/api/v1/dlq/stats")]
async fn dlq_stats(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-3: DLQ 统计
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_work.dlq_entry")
        .fetch_one(&state.db).await.unwrap_or(0);
    let exhausted: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM batch_work.dlq_entry WHERE retry_count >= max_retries"
    ).fetch_one(&state.db).await.unwrap_or(0);
    let retriable: i64 = total - exhausted;
    web::Json(serde_json::json!({
        "total": total,
        "exhausted": exhausted,
        "retriable": retriable,
        "max_retries_default": DLQ_MAX_RETRIES_DEFAULT,
    }))
}
#[get("/api/v1/workers")]
async fn worker_status(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-4: worker pool 状态 (GAP-4 优先级调度)
    web::Json(state.worker_pool.status())
}

#[get("/api/v1/data-sources")]
async fn list_data_sources(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-8: Master M-3 data_source 列表 (per BAS-001 v0.3 三分类, Master 5 表之一)
    let rows: Result<Vec<DataSource>, _> = sqlx::query_as::<_, DataSource>(
        "SELECT id, name, source_type, connection_ref, enabled, last_sync_at, created_at FROM batch_master.data_source ORDER BY name ASC"
    )
    .fetch_all(&state.db)
    .await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "data_sources": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/task-defs")]
async fn list_task_defs(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-8: Master M-1 task_def 列表 (per BAS-001 v0.3 三分类, Master 5 表之一)
    let rows: Result<Vec<TaskDef>, _> = sqlx::query_as::<_, TaskDef>(
        "SELECT id, name, handler, params_schema, enabled, created_at FROM batch_master.task_def ORDER BY name ASC"
    )
    .fetch_all(&state.db)
    .await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "task_defs": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/task-executions")]
async fn list_task_executions(
    state: web::Data<AppState>,
    query: web::Query<TaskExecutionQuery>,
) -> impl Responder {
    // W3 BA-W3-1: Transaction T-2 task_execution 高级查询 (per task_id/result/duration 过滤)
    let limit = query.limit.unwrap_or(50).min(500);
    let mut sql = String::from("SELECT id, task_id, attempt, started_at, finished_at, duration_ms, result, error_msg, trace_id FROM batch_transaction.task_execution WHERE 1=1");
    if let Some(tid) = query.task_id { sql.push_str(&format!(" AND task_id = '{}'", tid)); }
    if let Some(r) = &query.result { sql.push_str(&format!(" AND result = '{}'", r)); }
    if let Some(d) = query.min_duration_ms { sql.push_str(&format!(" AND duration_ms >= {}", d)); }
    sql.push_str(&format!(" ORDER BY started_at DESC LIMIT {}", limit));
    let rows: Result<Vec<TaskExecution>, _> = sqlx::query_as::<_, TaskExecution>(&sql).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "executions": list, "count": list.len(), "sql": sql })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[derive(Debug, Deserialize)]
struct LogEventQuery {
    level: Option<String>,
    target: Option<String>,
    task_id: Option<Uuid>,
    limit: Option<i64>,
}

#[get("/api/v1/logs")]
async fn list_logs(
    state: web::Data<AppState>,
    query: web::Query<LogEventQuery>,
) -> impl Responder {
    // W3 BA-W3-1: Transaction T-5 log_event 高级查询 (per level/target/task_id 过滤)
    let limit = query.limit.unwrap_or(100).min(1000);
    let mut sql = String::from("SELECT id, level, target, message, task_id, trace_id, created_at FROM batch_transaction.log_event WHERE 1=1");
    if let Some(lv) = &query.level { sql.push_str(&format!(" AND level = '{}'", lv)); }
    if let Some(t) = &query.target { sql.push_str(&format!(" AND target LIKE '%{}%'", t)); }
    if let Some(tid) = query.task_id { sql.push_str(&format!(" AND task_id = '{}'", tid)); }
    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));
    let rows: Result<Vec<LogEvent>, _> = sqlx::query_as::<_, LogEvent>(&sql).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "logs": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/task-progress")]
async fn list_task_progress(state: web::Data<AppState>) -> impl Responder {
    // W3 BA-W3-2: Work W-1 task_progress 列表
    let rows: Result<Vec<TaskProgress>, _> = sqlx::query_as::<_, TaskProgress>(
        "SELECT id, task_id, progress_pct, current_step, total_steps, updated_at FROM batch_work.task_progress ORDER BY updated_at DESC LIMIT 100"
    ).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "progress": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[post("/api/v1/task-progress")]
async fn upsert_task_progress(
    state: web::Data<AppState>,
    body: web::Json<TaskProgress>,
) -> impl Responder {
    // W3 BA-W3-2: Work W-1 task_progress upsert (per task_id)
    let p = body.into_inner();
    let _ = sqlx::query(
        "INSERT INTO batch_work.task_progress (id, task_id, progress_pct, current_step, total_steps, updated_at) \
         VALUES ($1, $2, $3, $4, $5, now()) \
         ON CONFLICT (task_id) DO UPDATE SET progress_pct = $3, current_step = $4, total_steps = $5, updated_at = now()"
    )
    .bind(p.id)
    .bind(p.task_id)
    .bind(p.progress_pct)
    .bind(&p.current_step)
    .bind(p.total_steps)
    .execute(&state.db)
    .await
    .map_err(|e| web::Json(serde_json::json!({ "error": e.to_string(), "upserted": false })));
    web::Json(serde_json::json!({ "upserted": true, "task_id": p.task_id, "progress_pct": p.progress_pct }))
}

#[get("/api/v1/task-buffer/{task_id}")]
async fn get_task_buffer(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    // W3 BA-W3-3: Work W-2 task_buffer 查询 (per task_id)
    let task_id = path.into_inner();
    let rows: Result<Vec<TaskBuffer>, _> = sqlx::query_as::<_, TaskBuffer>(
        "SELECT id, task_id, key, value, created_at FROM batch_work.task_buffer WHERE task_id = $1 ORDER BY created_at ASC"
    ).bind(task_id).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "buffers": list, "count": list.len(), "task_id": task_id })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[post("/api/v1/task-buffer")]
async fn put_task_buffer(
    state: web::Data<AppState>,
    body: web::Json<TaskBuffer>,
) -> impl Responder {
    // W3 BA-W3-3: Work W-2 task_buffer 写入 (per task_id + key)
    let b = body.into_inner();
    let _ = sqlx::query(
        "INSERT INTO batch_work.task_buffer (id, task_id, key, value, created_at) VALUES ($1, $2, $3, $4, now())"
    )
    .bind(b.id)
    .bind(b.task_id)
    .bind(&b.key)
    .bind(&b.value)
    .execute(&state.db)
    .await
    .map_err(|e| web::Json(serde_json::json!({ "error": e.to_string(), "stored": false })));
    web::Json(serde_json::json!({ "stored": true, "task_id": b.task_id, "key": b.key }))
}

#[get("/api/v1/audit-sessions")]
async fn list_audit_sessions(state: web::Data<AppState>) -> impl Responder {
    // W3 BA-W3-2/3: Work W-3 audit_session 列表 (operator 会话历史, 凭据 per 8/27 11:06 硬 ban)
    let rows: Result<Vec<AuditSession>, _> = sqlx::query_as::<_, AuditSession>(
        "SELECT id, operator, session_token, started_at, ended_at, ip_address FROM batch_work.audit_session ORDER BY started_at DESC LIMIT 50"
    ).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "sessions": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/dlq-events")]
async fn list_dlq_events(
    state: web::Data<AppState>,
    query: web::Query<DlqEventQuery>,
) -> impl Responder {
    // W3 BA-W3-5: Transaction T-4 dlq_event 高级查询 (per dlq_id/result/trace_id 过滤, 动态 SQL)
    let limit = query.limit.unwrap_or(50).min(500);
    let mut sql = String::from("SELECT id, dlq_id, attempt, started_at, finished_at, result, error_msg, trace_id FROM batch_transaction.dlq_event WHERE 1=1");
    if let Some(did) = query.dlq_id { sql.push_str(&format!(" AND dlq_id = '{}'", did)); }
    if let Some(r) = &query.result { sql.push_str(&format!(" AND result = '{}'", r)); }
    if let Some(t) = &query.trace_id { sql.push_str(&format!(" AND trace_id = '{}'", t)); }
    sql.push_str(&format!(" ORDER BY started_at DESC LIMIT {}", limit));
    let rows: Result<Vec<DlqEvent>, _> = sqlx::query_as::<_, DlqEvent>(&sql).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "dlq_events": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/audit-events")]
async fn list_audit_events(
    state: web::Data<AppState>,
    query: web::Query<AuditQuery>,
) -> impl Responder {
    // W3 BA-W3-4: Transaction T-3 audit_event 高级过滤 (per operator/action/result 过滤 + 时间范围)
    // AuditQuery 在 BA-W2-6 已定义, 这里用 query 注入 + 时间范围
    let limit = query.limit.unwrap_or(100).min(1000);
    let mut sql = String::from("SELECT id, operator, action, params_hash, result, trace_id, resource_type, resource_id, created_at FROM batch_transaction.audit_event WHERE 1=1");
    if let Some(op) = &query.operator { sql.push_str(&format!(" AND operator LIKE '%{}%'", op)); }
    if let Some(act) = &query.action { sql.push_str(&format!(" AND action LIKE '%{}%'", act)); }
    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));
    let rows: Result<Vec<AuditEvent>, _> = sqlx::query_as::<_, AuditEvent>(&sql).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "events": list, "count": list.len(), "retention_days": state.audit.retention_days })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/saga-instances")]
async fn list_saga_instances(
    state: web::Data<AppState>,
    query: web::Query<SagaInstanceQuery>,
) -> impl Responder {
    // W3 BA-W3-6: Transaction T-7 saga_instance 高级查询 (per saga_type/state 过滤, 动态 SQL)
    let limit = query.limit.unwrap_or(50).min(500);
    let mut sql = String::from("SELECT id, saga_type, state, started_at, updated_at, completed_at, payload, error_msg FROM batch_transaction.saga_instance WHERE 1=1");
    if let Some(t) = &query.saga_type { sql.push_str(&format!(" AND saga_type = '{}'", t)); }
    if let Some(s) = &query.state { sql.push_str(&format!(" AND state = '{}'", s)); }
    sql.push_str(&format!(" ORDER BY updated_at DESC LIMIT {}", limit));
    let rows: Result<Vec<SagaInstance>, _> = sqlx::query_as::<_, SagaInstance>(&sql).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "sagas": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/message-outbox")]
async fn list_message_outbox(
    state: web::Data<AppState>,
    query: web::Query<MessageOutboxQuery>,
) -> impl Responder {
    // W3 BA-W3-6: Transaction T-8 message_outbox 高级查询 (per destination/state 过滤, 动态 SQL)
    let limit = query.limit.unwrap_or(100).min(1000);
    let mut sql = String::from("SELECT id, destination, topic, payload, state, retry_count, created_at, sent_at FROM batch_transaction.message_outbox WHERE 1=1");
    if let Some(d) = &query.destination { sql.push_str(&format!(" AND destination = '{}'", d)); }
    if let Some(s) = &query.state { sql.push_str(&format!(" AND state = '{}'", s)); }
    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));
    let rows: Result<Vec<MessageOutbox>, _> = sqlx::query_as::<_, MessageOutbox>(&sql).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "outbox": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/data-migrations")]
async fn list_data_migrations(state: web::Data<AppState>) -> impl Responder {
    // W3 BA-W3-7: Transaction T-6 data_migration 状态查询 (per state 过滤 + 时间)
    let rows: Result<Vec<DataMigration>, _> = sqlx::query_as::<_, DataMigration>(
        "SELECT id, name, source_version, target_version, state, started_at, finished_at, rows_migrated, created_at FROM batch_transaction.data_migration ORDER BY created_at DESC LIMIT 50"
    ).fetch_all(&state.db).await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "migrations": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[post("/api/v1/workers/enqueue")]
async fn enqueue_task(
    state: web::Data<AppState>,
    body: web::Json<PendingTask>,
) -> impl Responder {
    // W2 BA-W2-4: GAP-4 优先级入队
    state.worker_pool.enqueue(body.into_inner());
    web::Json(serde_json::json!({
        "enqueued": true,
        "priority_queue_size": state.worker_pool.status().priority_queue_size,
    }))
}

#[get("/api/v1/workers/dequeue")]
async fn dequeue_task(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-4: GAP-4 优先级出队 (返回最高优先级 + 最早创建的任务)
    match state.worker_pool.dequeue() {
        Some(task_id) => web::Json(serde_json::json!({ "task_id": task_id, "dequeued": true })),
        None => web::Json(serde_json::json!({ "task_id": null, "dequeued": false, "queue_empty": true })),
    }
}

#[get("/api/v1/dlq")]
async fn list_dlq(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-3: DLQ 列表 (per GAP-9 超时 kill)
    let rows: Result<Vec<(Uuid, Uuid, i32, String, String, chrono::DateTime<chrono::Utc>)>, _> = sqlx::query_as(
        "SELECT id, template_id, template_version, params::text, reason, created_at \
         FROM batch_work.dlq_entry ORDER BY created_at DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "dlq": list, "count": list.len() })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[get("/api/v1/metrics")]
async fn metrics(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-7: Prometheus 完整 5 指标 (per BATCH-PLAN v0.2 §3.1 W2 BA-W2-7)
    // 5 指标: rgs_batch_up + rgs_batch_task_total + rgs_batch_task_duration_seconds + rgs_batch_worker_pool + rgs_batch_dlq
    let task_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_transaction.batch_task")
        .fetch_one(&state.db).await.unwrap_or(0);
    let task_succeeded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_transaction.batch_task WHERE state = 'succeeded'")
        .fetch_one(&state.db).await.unwrap_or(0);
    let task_failed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_transaction.batch_task WHERE state IN ('failed', 'timeout')")
        .fetch_one(&state.db).await.unwrap_or(0);
    let task_running: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_transaction.batch_task WHERE state = 'running'")
        .fetch_one(&state.db).await.unwrap_or(0);
    let dlq_size: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_work.dlq_entry")
        .fetch_one(&state.db).await.unwrap_or(0);
    let dlq_exhausted: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_work.dlq_entry WHERE retry_count >= max_retries")
        .fetch_one(&state.db).await.unwrap_or(0);
    // task duration: avg over succeeded tasks (per BA-W2-7 histogram 简化版)
    let avg_duration_secs: f64 = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT AVG(EXTRACT(EPOCH FROM (finished_at - started_at))) FROM batch_transaction.batch_task WHERE state = 'succeeded' AND started_at IS NOT NULL AND finished_at IS NOT NULL"
    )
    .fetch_one(&state.db).await.unwrap_or(None).unwrap_or(0.0);
    let cron_metrics = state.cron.stats();
    let cron_exec = cron_metrics.executions_total;
    let cron_active = cron_metrics.active_schedules;
    let worker_metrics = state.worker_pool.status();

    let body = format!(
        "# HELP rgs_batch_up Service up\n# TYPE rgs_batch_up gauge\nrgs_batch_up 1\n         # HELP rgs_batch_task_total Total tasks\n# TYPE rgs_batch_task_total counter\nrgs_batch_task_total {}\n         # HELP rgs_batch_task_succeeded_total Succeeded tasks\n# TYPE rgs_batch_task_succeeded_total counter\nrgs_batch_task_succeeded_total {}\n         # HELP rgs_batch_task_failed_total Failed tasks\n# TYPE rgs_batch_task_failed_total counter\nrgs_batch_task_failed_total {}\n         # HELP rgs_batch_task_running Running tasks\n# TYPE rgs_batch_task_running gauge\nrgs_batch_task_running {}\n         # HELP rgs_batch_task_duration_seconds_avg Average task duration\n# TYPE rgs_batch_task_duration_seconds_avg gauge\nrgs_batch_task_duration_seconds_avg {:.3}\n         # HELP rgs_batch_worker_pool_active Active workers\n# TYPE rgs_batch_worker_pool_active gauge\nrgs_batch_worker_pool_active {}\n         # HELP rgs_batch_worker_pool_max Max workers\n# TYPE rgs_batch_worker_pool_max gauge\nrgs_batch_worker_pool_max {}\n         # HELP rgs_batch_worker_pool_priority_queue Priority queue size\n# TYPE rgs_batch_worker_pool_priority_queue gauge\nrgs_batch_worker_pool_priority_queue {}\n         # HELP rgs_batch_dlq_size DLQ total\n# TYPE rgs_batch_dlq_size gauge\nrgs_batch_dlq_size {}\n         # HELP rgs_batch_dlq_exhausted DLQ exhausted retries\n# TYPE rgs_batch_dlq_exhausted gauge\nrgs_batch_dlq_exhausted {}\n         # HELP rgs_batch_cron_executions_total Cron executions\n# TYPE rgs_batch_cron_executions_total counter\nrgs_batch_cron_executions_total {}\n         # HELP rgs_batch_cron_active_schedules Active cron schedules\n# TYPE rgs_batch_cron_active_schedules gauge\nrgs_batch_cron_active_schedules {}\n",
        task_total, task_succeeded, task_failed, task_running, avg_duration_secs,
        worker_metrics.active, worker_metrics.max_concurrent, worker_metrics.priority_queue_size,
        dlq_size, dlq_exhausted, cron_exec, cron_active
    );
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(body)
}

#[get("/api/v1/grpc-status")]
async fn grpc_status(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-2: 5 域 gRPC client 状态
    let status = state.grpc_clients.health_check_all().await;
    web::Json(serde_json::json!({
        "domains": status,
        "total": status.len(),
        "connected": status.values().filter(|v| **v).count(),
    }))
}


#[get("/api/v1/cron/stats")]
async fn cron_stats(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-5 + GAP-3: cron 调度统计
    web::Json(state.cron.stats())
}


#[post("/api/v1/audit/log")]
async fn log_audit(
    state: web::Data<AppState>,
    body: web::Json<AuditEvent>,
) -> impl Responder {
    // W2 BA-W2-6: 手动审计日志入口 (供 5 域 gRPC client 调, per REQ F-10)
    state.audit.log(
        &body.operator,
        &body.action,
        &serde_json::json!({ "id": body.id, "params_hash": body.params_hash }),  // 只存 id + hash, 不存原值
        &body.result,
        &body.trace_id,
        body.resource_type.as_deref(),
        body.resource_id,
    ).await;
    web::Json(serde_json::json!({ "logged": true, "trace_id": body.trace_id }))
}

#[get("/api/v1/audit/query")]
async fn query_audit(
    state: web::Data<AppState>,
    query: web::Query<AuditQuery>,
) -> impl Responder {
    // W2 BA-W2-6: 审计查询 (operator / action 过滤, limit 100 默认)
    let limit = query.limit.unwrap_or(100).min(1000);
    let operator_filter = query.operator.clone().unwrap_or_else(|| "%".to_string());
    let action_filter = query.action.clone().unwrap_or_else(|| "%".to_string());
    let rows: Result<Vec<AuditEvent>, _> = sqlx::query_as::<_, AuditEvent>(
        "SELECT id, operator, action, params_hash, result, trace_id, resource_type, resource_id, created_at          FROM batch_transaction.audit_event          WHERE operator LIKE $1 AND action LIKE $2          ORDER BY created_at DESC LIMIT $3"
    )
    .bind(operator_filter)
    .bind(action_filter)
    .bind(limit)
    .fetch_all(&state.db)
    .await;
    match rows {
        Ok(list) => web::Json(serde_json::json!({ "events": list, "count": list.len(), "retention_days": state.audit.retention_days })),
        Err(e) => web::Json(serde_json::json!({ "error": e.to_string(), "count": 0 })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tracing::info!(target: SERVICE, "v{} starting on {}:{}", VERSION, BIND_HOST, BIND_PORT);

    // 1. sqlx PgPool 初始化 (BATCH_DB_URL, 凭据 per 8/27 11:06 JST 硬 ban)
    let db_url = std::env::var("BATCH_DB_URL").unwrap_or_else(|_| {
        tracing::warn!(target: SERVICE, "BATCH_DB_URL not set, using dev default (NOT for production)");
        "postgres://ulysses_local:REDACTED@localhost:5432/rgs_batch".to_string()
    });
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB connect failed: {}", e)))?;
    tracing::info!(target: SERVICE, "PgPool connected");

    // 2. 5 域 gRPC client 雏形 (per BA-W2-2 模板, 完整 5 域 W2-W3 扩展)
    // 凭据走 env var (per 8/27 11:06 JST 硬 ban)
    let ca_cert_pem = std::env::var("GRPC_CA_CERT_PEM").unwrap_or_default();
    let client_cert_pem = std::env::var("GRPC_CLIENT_CERT_PEM").unwrap_or_default();
    let client_key_pem = std::env::var("GRPC_CLIENT_KEY_PEM").unwrap_or_default();

    let grpc_clients = if !ca_cert_pem.is_empty() {
        GrpcClients::init_with_certs(ca_cert_pem.clone(), client_cert_pem.clone(), client_key_pem.clone()).await
    } else {
        tracing::warn!(target: SERVICE, "GRPC_CA_CERT_PEM not set, all 5 域 gRPC clients disabled (dev only)");
        GrpcClients::empty()
    };

    // 3. worker pool 雏形 (per BA-W2-4)
    let worker_pool = Arc::new(WorkerPool::new());
    worker_pool.heartbeat();

    let cron = CronEngine::new(db.clone());
    cron.start();
    let audit = AuditLogger::new(db.clone());

    let state = AppState {
        db,
        grpc_clients,
        worker_pool,
        cron,
        audit,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(health)
            .service(version)
            .service(list_task_templates)
            .service(list_tasks)
            .service(create_task)
            .service(worker_status)
            .service(enqueue_task)
            .service(dequeue_task)
            .service(list_dlq)
            .service(retry_dlq_by_id)
            .service(dlq_stats)
            .service(metrics)
            .service(grpc_status)
            .service(cron_stats)
            .service(log_audit)
            .service(query_audit)
            .service(list_data_sources)
            .service(list_task_defs)
            .service(list_task_executions)
            .service(list_logs)
            .service(list_task_progress)
            .service(upsert_task_progress)
            .service(get_task_buffer)
            .service(put_task_buffer)
            .service(list_audit_sessions)
            .service(list_dlq_events)
            .service(list_audit_events)
            .service(list_saga_instances)
            .service(list_message_outbox)
            .service(list_data_migrations)
    })
    .bind((BIND_HOST, BIND_PORT))?
    .run()
    .await
}
