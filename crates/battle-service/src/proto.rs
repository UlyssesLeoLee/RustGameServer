//! battle-service proto 模块 (per 桶 7 proto 实装)
//!
//! build.rs 编译 proto/battle/v1/battle.proto 到 OUT_DIR;
//! 此模块通过 tonic::include_proto! 暴露生成的 Rust struct 给 lib.rs 引用。

#![allow(clippy::all)]

/// battle.v1 生成的 gRPC 类型（service trait + request/response message）
pub mod v1 {
    tonic::include_proto!("battle.v1");
}
