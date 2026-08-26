//! cluster-ops proto 模块（per WGS v0.3 §2A.5 WF-1-54.3）
//!
//! WF-1-2073 扩展：增列 3 业务 service proto（player / economy / social）以支持
//! LCM 跨域 Saga 7 步 gRPC client 调用（per RGS-IMPL-PLAN-LCM-001 §3.6 + §6 R1
//! 缓解："不直连业务 service DB；只用 gRPC client"）。
//!
//! 协议编译在 build.rs（`compile_protos`），本文件通过 `tonic::include_proto!`
//! 暴露给 lib.rs 引用。注意同名命名空间隔离：每个子模块在自己的 v1 下，
//! 避免和 cluster_ops.v1 冲突。

#![allow(clippy::all)]

/// cluster_ops.v1 生成的 gRPC 类型
pub mod v1 {
    tonic::include_proto!("cluster_ops.v1");
}

/// player.v1 生成的 gRPC client 类型（per WF-1-2073 M-2073.1 跨域联动）
pub mod player_v1 {
    tonic::include_proto!("player.v1");
}

/// economy.v1 生成的 gRPC client 类型（per WF-1-2073 M-2073.2 跨域联动）
pub mod economy_v1 {
    tonic::include_proto!("economy.v1");
}

/// social.v1 生成的 gRPC client 类型（per WF-1-2073 M-2073.3 跨域联动）
pub mod social_v1 {
    tonic::include_proto!("social.v1");
}
