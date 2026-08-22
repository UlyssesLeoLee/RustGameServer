//! social-service proto 模块（per WBS v0.3 §2A.5 WF-1-54.3）

#![allow(clippy::all)]

/// social.v1 生成的 gRPC 类型
pub mod v1 {
    tonic::include_proto!("social.v1");
}
