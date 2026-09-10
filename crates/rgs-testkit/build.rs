//! rgs-testkit build.rs (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! 编译 player / match / admin 域 proto + 共享 common.proto,
//! 暴露 `tonic::include_proto!("player.v1")` / `tonic::include_proto!("match.v1")` /
//! `tonic::include_proto!("admin.v1")` / `tonic::include_proto!("common.v1")` 给
//! `bot::ai::{player,match,admin}` 模块 namespace 用, 拿到 typed client + request/response
//!
//! 5 worker 公共入口 (per L-CAND-014 模式 + L-CAND-016 防御):
//! - player / match / admin 域 worker 各自 build.rs 内容已主会话手修合并
//! - economy / social 域 worker 走 path dep (`economy-service` / `social-service`),
//!   不需要 rgs-testkit build.rs 编
//!
//! L-CAND-016 防御: 5 worker 公共 proto RPC 调用要同步, 本 build.rs 是
//! 5 worker 协同产物, 后续新增 5 域派生需更新 build.rs 路径列表
//!
//! k3s baseline 0/12 (per 9/10 16:36 JST 拍板 "接受 baseline 等 SRE 介入"):
//! - 真实 RPC 调用预期失败 (connection refused), 走 `Ok(())` 错误容忍模式
//! - 真实 cert 路径 placeholder (5 域 ST 业务级 mTLS 实践 commit 401ac5c cert 导出 SOP, SRE 介入后切换)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[
                "../../player-service/proto/player/v1/player.proto",
                "../../match-service/proto/match/v1/match.proto",
                "../../admin-service/proto/admin/v1/admin.proto",
                "../../player-service/proto/common/v1/common.proto",
            ],
            &[
                "../../player-service/proto",
                "../../match-service/proto",
                "../../admin-service/proto",
            ],
        )?;
    Ok(())
}
