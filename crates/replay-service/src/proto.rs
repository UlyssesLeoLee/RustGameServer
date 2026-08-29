//! replay-service proto 模块 (per RGS-DTL-038 §3 DEC-038-03 + 桶 13)
//!
//! build.rs 编译 proto/replay/v1/replay.proto 到 OUT_DIR;
//! 此模块通过 tonic::include_proto! 暴露生成的 Rust struct 给 lib.rs 引用。

#![allow(clippy::all)]

/// replay.v1 生成的 gRPC 类型 (4 RPC + 5 message)
pub mod v1 {
    tonic::include_proto!("replay.v1");
}
