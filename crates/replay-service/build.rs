//! replay-service build.rs (per RGS-DTL-038 §3 DEC-038-03 + 桶 13 实装)
//!
//! 编译 replay.proto v1 (4 RPC: SaveReplay / GetReplay / ListReplays / StreamReplay)
//! + 引入 common.proto (PageRequest / PageResponse / Timestamp)。

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/replay/v1/replay.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../replay-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
