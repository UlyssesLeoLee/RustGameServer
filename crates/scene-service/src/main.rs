//! scene-service 入口
//!
//! 7 域 binary entry (per 9/1 18:00 JST batch 域扩展 + 8/21 JST 5 域独立 Lead 原则)
//!
//! 启动 tonic gRPC server 接 SceneService 148 RPC + tracing 初始化。
//! Phase 2 起步: 仅启动 server, 业务全 InMemory (per L1 DoD cargo check 0 error)

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use scene_service::repository::{
    InMemoryMapUnitRepository, InMemorySceneInstanceRepository, InMemorySpaceRepository,
    MapUnitRepository, SceneInstanceRepository, SpaceRepository,
};
use scene_service::service::{SceneGrpcService, SceneService, SceneServiceImpl};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared_platform::install_default_crypto_provider();

    // tracing 初始化
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,scene-service=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50059".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;

    tracing::info!(target: "scene-service", "starting 7-domain scene service at {}", addr);

    // 7 域独立 Lead — 不与 5 域 Lead 兼任 (per 8/21 JST + 9/1 JST)
    // Phase 2 起步用 InMemory; Phase 3 接 PgRepository (per 9/5 改进路线图)
    let instances: Arc<dyn SceneInstanceRepository> =
        Arc::new(InMemorySceneInstanceRepository::new());
    let units: Arc<dyn MapUnitRepository> = Arc::new(InMemoryMapUnitRepository::new());
    let spaces: Arc<dyn SpaceRepository> = Arc::new(InMemorySpaceRepository::new());

    let service_impl: Arc<dyn SceneService> = Arc::new(SceneServiceImpl::new(
        instances, units, spaces,
    ));
    let grpc = SceneGrpcService::new(service_impl);

    // grpc.health.v1.Health 服务 (k3s exec 探针 + mTLS, per RGS-OPS-101)
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<scene_service::proto::v1::scene_service_server::SceneServiceServer<SceneGrpcService>>()
        .await;
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    // 7 域 mTLS 模式: 暂 insecure, 复用 5 域 mTLS SOP (per 8/27 ST 阶段)
    let mut server_builder = tonic::transport::Server::builder();
    if env::var("RGS_ALLOW_INSECURE_GRPC").is_ok() {
        tracing::warn!(
            target: "scene-service",
            "RGS_ALLOW_INSECURE_GRPC set — mTLS DISABLED, running INSECURE gRPC (dev/test only)"
        );
    } else {
        let tls_dir = env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let tls_config = shared_platform::tls::load_server_tls_config(
            &std::path::PathBuf::from(format!("{}/server.pem", tls_dir)),
            &std::path::PathBuf::from(format!("{}/server.key", tls_dir)),
            &std::path::PathBuf::from(format!("{}/ca.pem", tls_dir)),
        )
        .context(
            "mTLS config load failed (set RGS_ALLOW_INSECURE_GRPC=1 to bypass for dev/test)",
        )?;
        server_builder = server_builder
            .tls_config(tls_config)
            .context("tls_config")?;
        tracing::info!(target: "scene-service", "mTLS ENABLED — gRPC client cert verification required");
    }

    tracing::info!(target: "scene-service", "binding gRPC server at {}", addr);
    let svc = scene_service::proto::v1::scene_service_server::SceneServiceServer::new(grpc);
    server_builder
        .add_service(svc)
        .add_service(health_service)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
