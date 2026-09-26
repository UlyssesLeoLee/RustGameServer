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
//
// ## module_switch 接入 (per ULYS-190 §4.4 stage4, 2026-09-26 11:55 JST)
//
// RGS 的 mock_switch L1+L2 已 ship via PR 配套 commit `28a3c025` on dev
// (per ULYS-190 §4.3 brief). 本 v0.3 升级补 L3 module_switch:
//
// 5 plugin × 12 module (per handlers.rs 12 pub mod 一一映射 + 5 域 SRS):
//   - player    : role / scene / friend
//   - economy   : econ / pay / event
//   - match     : combat / pvp
//   - social    : guild / rank
//   - admin     : gm / card
//
// 跨项目範式对齐 (per G-MS-04):
//   - IM1.0 PR #24: 5 plugin × 28 module
//   - CATs PR #18: 4 plugin × 13 module
//   - Star PR #151: 7 plugin × 7 module
//   - RGS stage4 (本 commit): 5 plugin × 12 module
//
// 跨语言 dispatch 用法 (per G-MS-BRIEF-S44-01 RGS 推广):
//
// ```python
// import subprocess, json
// subprocess.run([
//     "python", "tools/rgs-flash-mock/scripts/_lib_mock_switch_rgs.py",
//     "--aci-config", "tools/rgs-flash-mock/.aci.json",
//     "read-plugins",
// ], check=True)
// ```
//
// 12 module 默认全 enabled, mode=offline. 跨项目累计当前:
//
// | 项目     | plugin | module | merged | commit |
// |----------|--------|--------|--------|--------|
// | IM1.0    | 5      | 28     | YES    | 96a2e28 on dev (PR #24) |
// | CATs     | 4      | 13     | YES    | 9b97d6b on main (PR #18) |
// | Star     | 7      | 7      | YES    | 55cf3794 on dev (PR #151) |
// | RGS      | 5      | 12     | (本 commit 跟踪) | (squash merge commit) |
//
// Rust native `_lib_mock_switch.rs` (替换 Python subprocess) 跨 session 续做
// (per G-MS-BRIEF-S44-01 推广).

use std::sync::Arc;
use tokio::sync::Mutex;

pub mod aci_emitter_helper;
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
