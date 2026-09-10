// rgs-flash-mock v0.1 lib 入口
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3
//
// 解决: cargo 自动 main.rs + sub-module 模式触发 lib 编译, AppState 在 main.rs 不可见
// 修法: AppState 跟 sub-modules 放 lib.rs, main.rs 仅启 actix-web

use std::sync::Arc;
use tokio::sync::Mutex;

pub mod gap_matrix;
pub mod handlers;

pub use gap_matrix::{CoverageReport, GapMatrix, RpcCategory, RpcStatus};

/// 全局 app state (per main.rs 共享)
pub struct AppState {
    pub matrix: Arc<Mutex<GapMatrix>>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}
