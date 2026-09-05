//! network-gateway build.rs (per W6 Phase 1 协议网关 骨架)
//!
//! 编译 gateway.proto (admin RPC schema) 到 OUT_DIR, lib.rs 通过 tonic::include_proto!
//! 引用生成的 Rust struct.

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &["proto/gateway/v1/gateway.proto"];
    let includes: &[&str] = &["proto"];
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
