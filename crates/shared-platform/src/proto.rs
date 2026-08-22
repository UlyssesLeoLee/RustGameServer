//! shared-platform proto 模块（per WBS v0.3 §2A.5 WF-1-54.3）
//!
//! build.rs 编译 proto/common/v1/common.proto 到 OUT_DIR。
//! 该模块通过 tonic::include_proto! 暴露生成的 Rust struct 给 lib.rs 使用。

#![allow(clippy::all)]

/// common.v1 生成的 gRPC 类型（service trait + request/response message）
pub mod v1 {
    tonic::include_proto!("common.v1");
}
