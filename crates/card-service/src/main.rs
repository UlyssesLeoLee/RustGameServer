//! card-service 入口 (桶 7 proto 设计 阶段)
//!
//! 桶 7 阶段: 仅做编译验证 (cargo build -p card-service 通过), 不启 gRPC server
//! 桶 10 (card catalog) 起按 5 域模板实装:
//!   - tonic gRPC server 接 CardService (HealthCheck + GetCard + ... + OpenPack)
//!   - PgCardRepository / PgCardInstanceRepository (per ARC-008 5 独立 DB → card_db)
//!   - mTLS (per RGS-REV-007 CH4 / DEC-015 P1) + outbox relay (per RGS-REV-007 CH1+CH2 / DEC-015 P1)
//!   - 8 张表 migration (per RGS-DTL-038 §7.1)

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing 初始化 (与 5 域模板对齐)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,card-service=debug")),
        )
        .init();

    // 桶 10 起启用 shared-platform::tracing_init::init_otel_exporter_optional
    // let _otel_guard = shared_platform::tracing_init::init_otel_exporter_optional(
    //     "card-service",
    //     env!("CARGO_PKG_VERSION"),
    //     "dev",
    // );

    tracing::info!(
        target: "card-service",
        "card-service v0.1.0 (桶 7 proto 设计 阶段 — 不启 gRPC server, 编译验证桩)"
    );

    // 桶 10 起: 启动 tonic gRPC server + PgRepository + OutboxRelay + mTLS
    // 桶 7 阶段: 仅打印启动日志, 进程立即退出 0
    tracing::info!(target: "card-service", "card-service stub exited (桶 7 阶段, 等待桶 10 业务实装)");
    Ok(())
}
