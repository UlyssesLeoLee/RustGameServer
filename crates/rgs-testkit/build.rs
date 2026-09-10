//! rgs-testkit build.rs (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! 编译 player.proto + common.proto, 暴露 `tonic::include_proto!("player.v1")` 给
//! `bot::ai::player` 模块用, 拿到 `PlayerServiceClient` / `HeartbeatRequest` /
//! `GetPlayerProfileRequest` 等 generated 类型.
//!
//! # 设计动机 (per task briefing "wave 4 worker 加 player-service 为 rgs-testkit dev-dep")
//!
//! - `player-service` 当前是 `[[bin]]` only, 无 `[lib]`, 不可作为 rgs-testkit 的 dep
//! - 加 `[lib]` 到 player-service Cargo.toml 越界 (任务简报: "不动 player-service")
//! - 退而求其次: 在 rgs-testkit 内部 `build.rs` 重新编译 player proto, 用 `include_proto!`
//!   宏拿 generated 类型. 不改 player-service 任何文件 (Cargo.toml / build.rs / service.rs / proto 都不动)
//!
//! # L-CAND-016 防御 (per 9/10 18:24 JST)
//!
//! - 本 worker 只编 player 域 proto, 不编其他 4 域 proto (economy / match / social / admin)
//! - 5 域 worker 各自编自己域 proto, 避免 race condition
//! - common.proto 是共享的, 5 域 worker 都需, 但 `tonic::include_proto!("common.v1")` 在 player
//!   域只用于本域生成的代码, 不会跟其他 4 域冲突
//!
//! # 强约束 (per 8/27 11:06 JST 硬 ban)
//!
//! - 凭据 (mTLS cert path) 走 `Option<String>`, 编译期不涉及
//! - 此 build.rs 只编 proto, 不读 cert, 不打印 secret
//!
//! # proto 路径
//!
//! - `player.proto` 在 `crates/player-service/proto/player/v1/`
//! - `common.proto` 在 `crates/shared-platform/proto/common/v1/`
//! - 都从 rgs-testkit crate root 算起, 用相对路径
//!
//! # 编译选项
//!
//! - `build_server(false)`: rgs-testkit 只用 client, 不需要 server
//! - `build_client(true)`: 暴露 `PlayerServiceClient<Channel>` 等
//!
//! # 不重复声明
//!
//! 跟 `crates/player-service/build.rs` 同步存在, 各自编各自的 crate. `tonic-build` 不会冲突.

use std::io::Result;

fn main() -> Result<()> {
    // 仅在 player 域 (本 worker scope) 编 player proto
    // 不编其他 4 域 proto (per L-CAND-016 防御)
    let protos: &[&str] = &[
        "../player-service/proto/player/v1/player.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "../player-service/proto",
        "../shared-platform/proto",
    ];

    tonic_build::configure()
        .build_server(false) // rgs-testkit 只用 client, 不需要 server
        .build_client(true)
        .compile_protos(protos, includes)?;

    Ok(())
}
