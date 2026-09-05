//! battle-service build.rs (per WBS 桶 7 proto 实装 + 7 域扩展)
//!
//! 编译 battle.proto + common.proto 到 OUT_DIR。
//! 与 5 域 build.rs 对齐: 共享 common.proto 源,
//! include 路径包含 ../shared-platform/proto 供 `import "common/v1/common.proto"`。

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/battle/v1/battle.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../battle-service/proto",
    ];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
