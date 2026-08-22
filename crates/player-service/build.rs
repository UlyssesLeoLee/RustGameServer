//! player-service build.rs（per WBS v0.3 §2A.5 WF-1-54.3）
//! 同时编译 player.proto + common.proto（生成代码路径对齐）

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/player/v1/player.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../player-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
