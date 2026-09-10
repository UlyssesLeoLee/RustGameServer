// rgs-flash-mock v0.3 lib 入口
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3
//
// v0.1 → v0.2 升级:
//   - 加 grpc_clients 模块 (5 域 mTLS 业务级 tonic client)
//   - AppState 扩展: matrix + clients + cfg
//   - main.rs 启动时连接 5 域, 任一域失败仍启动 (Option<Client>)
// v0.2 → v0.3 升级:
//   - 5 域 → 7 域 (加 card + leaderboard, per RGS-DTL-038 §4.4 + §3)
//   - card proto + leaderboard proto compile (build.rs)
//   - GrpcClients 扩 2 字段: card + leaderboard
//   - status_report 扩 2 行: 7 域 status

use std::sync::Arc;
use tokio::sync::Mutex;

pub mod config;
pub mod gap_matrix;
pub mod grpc_clients;
pub mod handlers;

pub use gap_matrix::{CoverageReport, GapMatrix, RpcCategory, RpcStatus};
pub use grpc_clients::{GrpcClientStatus, GrpcClients};

/// 安装 ring 作为 rustls 默认 crypto provider (per shared-platform::tls 同模式)
/// 调用方应在 main 入口调用一次, 后续 TLS 操作不会 panic
pub fn install_default_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// 全局 app state (per main.rs 共享)
pub struct AppState {
    pub matrix: Arc<Mutex<GapMatrix>>,
    pub clients: GrpcClients,
    pub cfg: Arc<config::Config>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}
