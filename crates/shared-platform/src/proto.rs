//! shared-platform proto 模块（per WBS v0.3 §2A.5 WF-1-54.3）
//!
//! 编译 common.proto 后，OUT_DIR 包含 `common.v1.rs` 模块。
//! 6 域 crate 通过 tonic::include_proto!("common.v1") 引用本模块。

#![allow(clippy::all)]

/// common.v1 公共类型（Status / ErrorCode / EntityId / Timestamp / PageRequest / PageResponse / HealthCheck）
pub mod v1 {
    tonic::include_proto!("common.v1");
}