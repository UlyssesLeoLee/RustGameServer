fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos: &[&str] = &[
        "proto/gm_extra/v1/gm_extra.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../gm-extra-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
