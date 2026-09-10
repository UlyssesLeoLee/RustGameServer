//! rgs-flash-mock build.rs (v0.3 — mTLS 业务级 7 域 gRPC client)
//!
//! 编译 7 域 proto (player/economy/match/social/admin + card/leaderboard) + common.proto
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §2.1 (tonic 0.12)
//! per 8/31 W37 5 域独立 proto 模式 (跟 crates/{player,...}-service/build.rs 一样)
//! v0.3 新增 card-service + leaderboard-service proto (per RGS-DTL-038 §4.4 + §3 DEC-038-02)

use std::io::Result;

fn main() -> Result<()> {
    // 7 域 proto + common.proto (v0.3 起接 mTLS 业务级 7 域)
    // 5 域 (player/economy/match/social/admin) + 2 域 (card/leaderboard) per RGS-DTL-038
    let protos: &[&str] = &[
        "proto/player/v1/player.proto",
        "proto/economy/v1/economy.proto",
        "proto/match/v1/match.proto",
        "proto/social/v1/social.proto",
        "proto/admin/v1/admin.proto",
        "proto/card/v1/card.proto",
        "proto/leaderboard/v1/leaderboard.proto",
        "proto/common/v1/common.proto",
    ];
    // proto include path: 让 7 域 proto 能 import "common/v1/common.proto"
    let includes: &[&str] = &["proto"];

    tonic_build::configure()
        // 仅 build_client, mock 是 client 不需要 server
        .build_server(false)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
