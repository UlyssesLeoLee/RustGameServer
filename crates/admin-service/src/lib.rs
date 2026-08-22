#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! admin-service —— 5 域管理微服务业务骨架。
//!
//! 域职责：GM 命令、公告、封禁、合规审计、COC 集群运营中心、CEM 中心事件管理。
//! 规范：RGS-REQ-019 / RGS-BAS-019 / RGS-DTL-019 / RGS-SPEC-DTL-019 / RGS-ARC-051 / RGS-SEC-100。
//! DB：独立 admin_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：admin/v1/admin.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.6 实化：entity 2 个 + Repository trait + PgRepository sqlx impl + InMemoryRepository 测用。
//! 审计日志 hash 链 + UPDATE/DELETE 触发器禁（per RGS-SEC-100 §7）。

pub mod entity;
pub mod error;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    AdminUserRepository, AuditLogRepository, InMemoryAdminUserRepository,
    InMemoryAuditLogRepository, PgAdminUserRepository, PgAuditLogRepository,
};

pub mod proto;

pub mod db;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
