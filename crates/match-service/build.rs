//! match-service build.rs
//!
//! W36 (2026-08-30): 增量 compile replay.proto (client only) for 跨域 SaveReplay saga
//! - match-service 是 replay-service 的 gRPC client (session 结束触发 SaveReplay)
//! - 服务端 trait 不需要: `build_server(false)` 仅 build client

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "proto/match/v1/match.proto",
        "../shared-platform/proto/common/v1/common.proto",
        // W36 (2026-08-30): replay.proto client (match-service → replay-service SaveReplay)
        "../replay-service/proto/replay/v1/replay.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../match-service/proto",
        // W36 (2026-08-30): replay proto 搜索路径
        "../replay-service/proto",
    ];
    println!("cargo:rerun-if-changed=proto/match/v1/match.proto");
    println!("cargo:rerun-if-changed=../shared-platform/proto/common/v1/common.proto");
    println!("cargo:rerun-if-changed=../replay-service/proto/replay/v1/replay.proto");
    println!("cargo:rerun-if-changed=build.rs");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
