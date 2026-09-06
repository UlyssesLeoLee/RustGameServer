//! batch-service entry point (Phase 0 scaffold)
//!
//! Per 改进路线图 §1 Phase 0 (W25 任务 brief, 2026-09-05 12:30 JST):
//! - batch 域是 6 域扩展 (per AGENTS §7 + DEC-008 + 9/1 18:00 JST 拍板)
//! - 不与 5 域 Lead 兼任 (per 8/21 JST 5 域独立 Lead 基线)
//! - DB 三分类: batch_master / batch_transaction / batch_work (per 9/1 18:30 JST + AGENTS §7.2 #2)
//! - 13 proto + 3 子域 业务实装 留 Phase 1+ 续 (per W25 任务 brief 卡住 fallback)
//! - 实际完整实装 (rgs-batch-backend + rgs-batch-console) 在 tools/ 下, 本 crate 仅 workspace 注册占位
//!
//! Phase 1+ 实装 scope (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 §3 + RGS-BATCH-PLAN-2026-09-01 v0.1):
//! - 13 proto (BATCH 域 RPC: DAG 触发 + 任务模板 + audit + DLQ + retry 等)
//! - 3 子域: batch_master (任务模板) / batch_transaction (任务实例 + 流水) / batch_work (执行 + 状态机)
//! - DB 三分类: batch_master / batch_transaction / batch_work / batch_transaction_archive
//! - mTLS 业务级: 5 域 gRPC 调用走 mTLS (per AGENTS §7.2 #4 + 5 域 ST 业务级 mTLS 实践 commit 401ac5c)
//! - audit 永久保留: audit_event T-3 永久 (per AGENTS §7.2 #10 + REQ F-10)
//! - env value 硬 ban: BATCH_DB_PASSWORD 等凭据走 env, 永不打印 (per 8/27 11:06 JST + REQ NFR-30)

use std::env;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing 初始化 (与 5 域模板对齐, per AGENTS §6.1 PT 派工简报 DoD)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,batch-service=debug")),
        )
        .init();

    let domain = env::var("BATCH_DOMAIN").unwrap_or_else(|_| "batch-master".to_string());
    let addr: std::net::SocketAddr = env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50070".to_string())
        .parse()
        .map_err(|e: std::net::AddrParseError| anyhow::anyhow!("invalid GRPC_ADDR: {e}"))?;

    tracing::info!(target: "batch-service", "batch 域 Phase 0 scaffold ready: domain={domain}, addr={addr}");

    // Phase 0 占位: 仅 ready 信号, 0 业务实装
    // Phase 1+ 实装: 13 proto + 3 子域 + DB 三分类 + mTLS + audit + DLQ (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1)
    tracing::warn!(
        target: "batch-service",
        "Phase 0 scaffold only — 业务实装留 Phase 1+ 续, 完整实装在 tools/rgs-batch-backend (per AGENTS §7.1)"
    );

    Ok(())
}
