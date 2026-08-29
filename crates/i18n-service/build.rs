//! i18n-service build.rs (per RGS-DTL-038 §4.1 + DEC-038-05)
//!
//! Compile i18n.proto + common.proto into Rust types via tonic-build.

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/i18n/v1/i18n.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../i18n-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
