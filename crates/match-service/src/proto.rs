//! match-service proto 模块（per WBS v0.3 §2A.5 WF-1-54.3）

#![allow(clippy::all)]

/// match.v1 生成的 gRPC 类型（match 是 Rust 关键字，include_proto 用 r# 前缀）
pub mod v1 {
    tonic::include_proto!("r#match.v1");
}
