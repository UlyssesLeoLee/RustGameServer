//! scene-service build.rs
//!
//! 编译 scene.proto + common.proto 到 OUT_DIR
//! 7 域独立 Lead (per 9/1 18:00 JST batch 域扩展 + 8/21 JST 5 域独立 Lead 原则)

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/scene/v1/scene.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../scene-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
