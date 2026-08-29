//! card-service build.rs（per WBS v0.5 §2 桶 7 proto 实装）
//!
//! 编译 card.proto + common.proto 到 OUT_DIR。
//! 与 match-service / player-service build.rs 对齐: 共享 common.proto 源,
//! include 路径包含 ../shared-platform/proto 供 `import "common/v1/common.proto"`。

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/card/v1/card.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../card-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
