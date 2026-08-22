//! cluster-ops build.rs

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/cluster_ops/v1/cluster_ops.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &["proto", "../shared-platform/proto", "../cluster-ops/proto"];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
