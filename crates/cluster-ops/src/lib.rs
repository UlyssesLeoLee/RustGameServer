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
//!
//! WF-1-2073 扩展：新增 `realm_lifecycle` 子模块（per RGS-IMPL-PLAN-LCM-001 §2.2
//! + SPEC-DTL-042 §2 + FR-LCM-004 硬约束）。

pub mod entity;
pub mod error;
pub mod realm_lifecycle;
pub mod repository;
pub mod service;

/// WF-1-2073: 服务器全生命周期管理子模块（per RGS-IMPL-PLAN-LCM-001 v0.1）
///
/// 含 6 阶段操作器 trait + 跨域 Saga 7 步 + 6 张 plan 表（含 retire RBAC）。
/// 本子模块**不**对外暴露独立 gRPC（per FR-LCM-004 硬约束）。
pub mod realm_lifecycle;

pub use error::{Error, Result};
pub use repository::{
    ClusterNodeRepository, FeatureFlagRepository, InMemoryClusterNodeRepository,
    InMemoryFeatureFlagRepository, PgClusterNodeRepository, PgFeatureFlagRepository,
};

pub mod proto;

pub mod db;

// ===== WBS L4 #2074（per RGS-IMPL-PLAN-LCM-001 v0.1 §3.7 PH-6）=====
// 归档冷热分层 + N+2 冗余 + GDPR "被遗忘权" 删除通路
// 硬约束：FR-LCM-081 归档不删数据 / NFR-SE-010 双层审计 / RSK-LCM-005 N+2
pub mod realm_lifecycle;

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
