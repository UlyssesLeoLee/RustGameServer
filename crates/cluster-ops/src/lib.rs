#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! cluster-ops —— 5 域集群运营微服务业务骨架。
//!
//! 域职责：跨服 Active-Active 协调（per DEC-001）、CEM 事件路由、PFAU all-reachable（per DEC-002）。
//! 规范：RGS-REQ-020 / RGS-BAS-020 / RGS-DTL-020 / RGS-SPEC-DTL-020。
//! DB：独立 cluster_ops_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：cluster-ops/v1/cluster.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
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
