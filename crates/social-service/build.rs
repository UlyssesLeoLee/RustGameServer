//! social-service build.rs

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/social/v1/social.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../social-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
