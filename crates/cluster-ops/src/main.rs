//! cluster-ops 入口（54.7 业务实施后 binary）
//!
//! 启动 tonic gRPC server 接 ClusterOpsService（HealthCheck + GetNode）+ tracing 初始化。
//! 55.15 wire-up：main.rs 切到 PgRepository + db::pool_from_env() + migrations。

use anyhow::Context;
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use cluster_ops::db;
use cluster_ops::repository::{
    ClusterNodeRepository, FeatureFlagRepository, PgClusterNodeRepository,
    PgFeatureFlagRepository,
};
use cluster_ops::service::grpc_service::ClusterOpsGrpcService;
use cluster_ops::service::ClusterOpsServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,cluster-ops=debug")),
        )
        .init();

    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50056".to_string())
        .parse()
        .context("invalid GRPC_ADDR")?;
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "cluster-ops", "starting service at {}, db={}", addr, database_url);

    // 55.15: 真实 DB pool + migrations（InMemory 留作测试用）
    let pool = match db::pool_from_env().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(target: "cluster-ops", "DB pool init failed: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = db::run_migrations(&pool).await {
        tracing::error!(target: "cluster-ops", "DB migrations failed: {}", e);
        std::process::exit(1);
    }

    let nodes: Arc<dyn ClusterNodeRepository> =
        Arc::new(PgClusterNodeRepository::new(pool.clone()));
    let flags: Arc<dyn FeatureFlagRepository> =
        Arc::new(PgFeatureFlagRepository::new(pool.clone()));

    tracing::info!(target: "cluster-ops", "cluster-ops started, DB pool size: {}", pool.size());

    let service_impl = Arc::new(ClusterOpsServiceImpl::new(nodes, flags));
    let grpc = ClusterOpsGrpcService::new(service_impl);

    tracing::info!(target: "cluster-ops", "binding gRPC server at {}", addr);
    let svc =
        cluster_ops::proto::v1::cluster_ops_service_server::ClusterOpsServiceServer::new(grpc);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("tonic server failed")?;
    Ok(())
}
