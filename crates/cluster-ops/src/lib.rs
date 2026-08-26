#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

//! cluster-ops —— 5 域集群运营微服务业务骨架。
//!
//! 域职责：跨服 Active-Active 协调（per DEC-001）、CEM 事件路由、PFAU all-reachable（per DEC-002）。
//! 规范：RGS-REQ-020 / RGS-BAS-020 / RGS-DTL-020 / RGS-SPEC-DTL-020 / RGS-ARC-051。
//! DB：独立 cluster_ops_db（per ARC-008 5 独立 DB 原则）。
//! gRPC API：cluster-ops/v1/cluster.proto（per WF-1-54.2 Proto 定义 + WF-1-54.3 tonic-build）。
//!
//! 54.6 实化：entity 2 个 + Repository trait + PgRepository sqlx impl + InMemoryRepository 测用。

pub mod entity;
pub mod error;
pub mod realm_lifecycle;
pub mod repository;
pub mod service;

pub use error::{Error, Result};
pub use repository::{
    ClusterNodeRepository, FeatureFlagRepository, InMemoryClusterNodeRepository,
    InMemoryFeatureFlagRepository, PgClusterNodeRepository, PgFeatureFlagRepository,
};

pub mod proto;

pub mod db;

// WF-1-2067 启用：realm_lifecycle 子模块（per RGS-IMPL-PLAN-LCM-001 §2.1）
pub mod realm_lifecycle;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
