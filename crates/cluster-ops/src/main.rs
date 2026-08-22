//! cluster-ops 入口（54.1 占位二进制）
//!
//! 启动 tonic gRPC server（占位 health check service）+ 读取 DATABASE_URL。
//! 54.7 业务实施后接入实际 gRPC method。

use anyhow::Context;
use std::env;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

use cluster_ops::service::{ClusterOpsService, ClusterOpsServiceImpl};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing 初始化
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,cluster-ops=debug")),
        )
        .init();

    let addr = env::var("GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL env required")?;

    tracing::info!(target: "cluster-ops", "starting service at {}, db={}", addr, database_url);

    let service = ClusterOpsServiceImpl::new();

    // 54.1 占位：仅 health check；54.7 业务实施后加实际 gRPC method
    let svc_health = service.health_check().await?;
    tracing::info!(target: "cluster-ops", "health check: {}", svc_health);

    // 54.1 占位：tonic server 不实际 bind（待 54.2 proto + 54.3 tonic-build）
    tracing::warn!(target: "cluster-ops", "54.1 骨架：tonic server 占位不 bind；待 54.2-54.3 启用");

    // 阻塞 1 秒后退出（占位行为）
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    tracing::info!(target: "cluster-ops", "exiting (54.1 placeholder)");

    Ok(())
}
