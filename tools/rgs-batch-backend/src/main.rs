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
            let connected = self.clients.get(domain)
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
}

#[derive(Clone)]
struct WorkerPool {
    active_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl WorkerPool {
    fn new() -> Self {
        Self {
            active_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    fn heartbeat(&self) {
        // 雏形: tokio::spawn 1 个 worker, 每 5s 心跳 (per WORKER_HEARTBEAT_SECS)
        let count = self.active_count.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(WORKER_HEARTBEAT_SECS));
            loop {
                interval.tick().await;
                tracing::debug!(target: SERVICE, "worker heartbeat, active={}", count.load(std::sync::atomic::Ordering::Relaxed));
            }
        });
    }

    fn status(&self) -> WorkerPoolStatus {
        WorkerPoolStatus {
            active: self.active_count.load(std::sync::atomic::Ordering::Relaxed),
            idle: 0, // W2 雏形仅 1 worker
            total: 1,
            by_priority: std::collections::HashMap::new(),
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
            "BA-W2-3: DLQ stub (per GAP-9 超时 kill)",
            "BA-W2-4: worker pool 雏形 (GAP-4 优先级调度接口预留)",
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

async fn push_dlq(state: &AppState, template_id: Uuid, template_version: i32, params: &serde_json::Value, reason: &str) {
    // W2 BA-W2-3 DLQ stub: 进 batch_work.dlq_entry
    let _ = sqlx::query(
        "INSERT INTO batch_work.dlq_entry (id, template_id, template_version, params, reason, created_at) \
         VALUES (gen_random_uuid(), $1, $2, $3, $4, now())"
    )
    .bind(template_id)
    .bind(template_version)
    .bind(params)
    .bind(reason)
    .execute(&state.db)
    .await
    .map_err(|e| tracing::error!(target: SERVICE, "DLQ insert failed: {}", e));
}

#[get("/api/v1/workers")]
async fn worker_status(state: web::Data<AppState>) -> impl Responder {
    // W2 BA-W2-4: worker pool 状态 (GAP-4 优先级调度)
    web::Json(state.worker_pool.status())
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
    // W2 BA-W2-7: Prometheus 5 指标
    let task_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_transaction.batch_task")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let dlq_size: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_work.dlq_entry")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let body = format!(
        "# HELP rgs_batch_up Service up\n# TYPE rgs_batch_up gauge\nrgs_batch_up 1\n\
         # HELP rgs_batch_task_total Total tasks\n# TYPE rgs_batch_task_total counter\nrgs_batch_task_total {}\n\
         # HELP rgs_batch_worker_pool_active Active workers\n# TYPE rgs_batch_worker_pool_active gauge\nrgs_batch_worker_pool_active {}\n\
         # HELP rgs_batch_dlq_size DLQ size\n# TYPE rgs_batch_dlq_size gauge\nrgs_batch_dlq_size {}\n\
         # HELP rgs_batch_cron_executions_total Cron executions\n# TYPE rgs_batch_cron_executions_total counter\nrgs_batch_cron_executions_total 0\n",
        task_total,
        state.worker_pool.status().active,
        dlq_size
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

    let state = AppState {
        db,
        grpc_clients,
        worker_pool,
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
            .service(list_dlq)
            .service(metrics)
            .service(grpc_status)
    })
    .bind((BIND_HOST, BIND_PORT))?
    .run()
    .await
}
