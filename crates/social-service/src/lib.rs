#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! social-service —— 5 域社交微服务业务骨架。
//!
//! 域职责：好友、公会、聊天、邮件。
//! 规范：RGS-REQ-026 / RGS-BAS-026 / RGS-DTL-026 / RGS-SPEC-DTL-026。
//! DB：独立 social_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：social/v1/social.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.1 骨架：4 子模块（error / service / repository / entity）；
//! 实际业务逻辑待 WF-1-54.5-54.7。

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};

pub mod proto;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
