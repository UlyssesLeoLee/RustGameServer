//! rgs-grpc-bridge build.rs
//!
//! 复用 player-service 的 .proto (不复制, 用相对路径 include)
//! 同时生成 client (用于 forwarder 调 backend) + server (用于 tonic-web 暴露)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player_proto = "../../crates/player-service/proto/player/v1/player.proto";
    let common_proto = "../../crates/shared-platform/proto/common/v1/common.proto";

    let protos: &[&str] = &[player_proto, common_proto];
    let includes: &[&str] = &[
        "../../crates/player-service/proto",
        "../../crates/shared-platform/proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
