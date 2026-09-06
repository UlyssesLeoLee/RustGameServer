fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos: &[&str] = &[
        "proto/social_extra/v1/social_extra.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../social-extra-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
