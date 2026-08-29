//! leaderboard-service build.rs
//!
//! 编译 leaderboard.proto v1 (4 类榜单 RPC) + 引入 common.proto

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/leaderboard/v1/leaderboard.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../leaderboard-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
