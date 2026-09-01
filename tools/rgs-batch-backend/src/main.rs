// rgs-batch-backend main.rs (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-2, 2026-09-02 01:55 JST Mavis 接手代签)
//
// Rust + actix-web 4 + tokio + tonic 0.12 gRPC client + sqlx 0.7 + mTLS 业务级
// 独立 cargo project (per AGENTS.md v0.4 §7.1, 不加入 workspace)
//
// Routes:
//   GET  /api/v1/health      健康检查
//   GET  /api/v1/version     版本信息
//   GET  /api/v1/metrics     Prometheus 9464 指标 (per W2 BA-W2-7)
//
// Bind:
//   0.0.0.0:8790  (区别 rgs-batch-console 8789 + rgs-web 8788 + gm-backend 8081)

use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

const VERSION: &str = "0.1.0";
const SERVICE: &str = "rgs-batch-backend";
const BIND_HOST: &str = "0.0.0.0";
const BIND_PORT: u16 = 8790;

static START_TIME: once_cell::sync::Lazy<Instant> =
    once_cell::sync::Lazy::new(Instant::now);

#[derive(Serialize)]
struct HealthResp {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    uptime_ms: u128,
    ts: String,
}

#[get("/api/v1/health")]
async fn health() -> impl Responder {
    let ts = chrono::Utc::now().to_rfc3339();
    web::Json(HealthResp {
        status: "ok",
        service: SERVICE,
        version: VERSION,
        uptime_ms: START_TIME.elapsed().as_millis(),
        ts,
    })
}

#[derive(Serialize)]
struct VersionResp {
    backend: &'static str,
    batch_plan: &'static str,
    detaill: &'static str,
    console_target: &'static str,
}

#[get("/api/v1/version")]
async fn version() -> impl Responder {
    web::Json(VersionResp {
        backend: VERSION,
        batch_plan: "RGS-BATCH-PLAN-2026-09-01_v0.2",
        detaill: "RGS-BATCH-DETAILED-DESIGN-2026-09-01_v0.1",
        console_target: "rgs-batch-console v0.1.0",
    })
}

#[get("/api/v1/metrics")]
async fn metrics() -> impl Responder {
    // TODO (per W2 BA-W2-7): Prometheus 指标 - 5 项
    // - rgs_batch_task_total
    // - rgs_batch_task_duration_seconds
    // - rgs_batch_worker_pool_active
    // - rgs_batch_dlq_size
    // - rgs_batch_cron_executions_total
    let body = "# HELP rgs_batch_up Service up\n# TYPE rgs_batch_up gauge\nrgs_batch_up 1\n";
    actix_web::HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tracing::info!(target: SERVICE, "v{} starting on {}:{}", VERSION, BIND_HOST, BIND_PORT);

    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(version)
            .service(metrics)
    })
    .bind((BIND_HOST, BIND_PORT))?
    .run()
    .await
}
