#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! admin-service —— 5 域管理微服务业务骨架。
//!
//! 域职责：GM 命令、公告、封禁、合规审计。
//! 规范：RGS-REQ-019 / RGS-BAS-019 / RGS-DTL-019 / RGS-SPEC-DTL-019。
//! DB：独立 admin_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：admin/v1/admin.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.1 骨架：4 子模块（error / service / repository / entity）；
//! 实际业务逻辑待 WF-1-54.5-54.7。

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
