// rgs-flash-mock build.rs
// 编译 RGS 5 域 + card + common proto (gRPC client only, server=false)
//
// 路径: 跨 crates/ 引用, 因为 mock 跟 RGS 独立 workspace 但需要 RGS proto 类型
// per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §2.2 文件结构

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_paths = &[
        // RGS 6 域 + card + common
        "../../crates/shared-platform/proto/common/v1/common.proto",
        "../../crates/player-service/proto/player/v1/player.proto",
        "../../crates/economy-service/proto/economy/v1/economy.proto",
        "../../crates/match-service/proto/match/v1/match.proto",
        "../../crates/admin-service/proto/admin/v1/admin.proto",
        "../../crates/card-service/proto/card/v1/card.proto",
    ];
    let include_paths = &["../../crates"];

    tonic_build::configure()
        .build_server(false)  // mock 只用 client, 不需要 server
        .build_client(true)
        .compile(proto_paths, include_paths)?;

    Ok(())
}
