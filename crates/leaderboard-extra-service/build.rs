fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos: &[&str] = &[
        "proto/leaderboard_extra/v1/leaderboard_extra.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../leaderboard-extra-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
