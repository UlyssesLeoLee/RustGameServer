//! admin-service build.rs

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/admin/v1/admin.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../admin-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
