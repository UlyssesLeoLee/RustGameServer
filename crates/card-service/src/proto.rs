//! card-service proto 模块（per WBS v0.5 桶 7 proto 实装）
//!
//! build.rs 编译 proto/card/v1/card.proto 到 OUT_DIR；
//! 此模块通过 tonic::include_proto! 暴露生成的 Rust struct 给 lib.rs 引用。
//!
//! 桶 7 阶段: 仅 include, 业务 service trait 由 6 域 server 实装 (桶 10 card catalog 起补)

#![allow(clippy::all)]

/// card.v1 生成的 gRPC 类型（service trait + request/response message）
pub mod v1 {
    tonic::include_proto!("card.v1");
}
