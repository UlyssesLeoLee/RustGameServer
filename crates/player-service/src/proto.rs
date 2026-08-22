//! player-service proto 模块（per WBS v0.3 §2A.5 WF-1-54.3）
//!
//! build.rs 编译 proto/player/v1/player.proto 到 OUT_DIR；
//! 此模块通过 tonic::include_proto! 暴露生成的 Rust struct 给 lib.rs 引用。

#![allow(clippy::all)]

/// player.v1 生成的 gRPC 类型（service trait + request/response message）
pub mod v1 {
    tonic::include_proto!("player.v1");
}
