//! network-gateway proto 模块 (per W6 Phase 1 协议网关 骨架)
//!
//! build.rs 编译 proto/gateway/v1/gateway.proto 到 OUT_DIR;
//! 此模块通过 tonic::include_proto! 暴露生成的 Rust struct 给 lib.rs 引用.

#![allow(clippy::all)]

/// gateway.v1 生成的 gRPC 类型 (service trait + request/response message)
pub mod v1 {
    tonic::include_proto!("gateway.v1");
}
