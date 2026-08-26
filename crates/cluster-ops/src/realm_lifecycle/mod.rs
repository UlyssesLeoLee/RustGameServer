//! realm_lifecycle 子模块（per RGS-SPEC-DTL-042 §2 + DTL-031 §1.1）
//!
//! 服务器全生命周期管理（AD 限界上下文扩展 Feature）。**不**独立 crate；作为
//! `cluster-ops` 子模块存在，对外接口**仅**经 `AdminService` 转发（FR-LCM-004）。
//!
//! ## 子模块清单（per SPEC §2 + IMPL-PLAN-LCM-001 §3.5）
//!
//! - [`error`]：LCM 域特化错误（含 PFAU 状态非法跳转 / Saga 失败 / OLU 上报失败）
//! - [`service`]：`RealmLifecycleService` 主入口（编排 + 状态机 + 路由）
//! - [`operators`]：6 阶段操作器 trait（new_realm / scale / split / merge / retire / archive；
//!   merge_rollback 走 merge 逆向补偿路径而非独立操作器）
//! - [`saga`]：`SagaOrchestrator` 占位 + 步骤定义
//! - [`plans`]：6 Plan 占位（new_realm_plan / scale_plan / split_plan / merge_plan /
//!   retire_plan / archive_plan）
//! - [`feature_adapter`]：FeatureType 适配 + 7 SubFeature 注册 + PFAU 5 状态编排（M-2071.1~3）
//! - [`olu_reporter`]：`rgs-arc-olu` 通道（NFR-LCM-007 硬约束；M-2071.4）
//! - [`metrics`]：10 项 `rgs_lcm_*` 指标（M-2071.5）

#![allow(clippy::result_large_err)]

pub mod error;
pub mod service;

pub mod operators;

pub mod saga;

pub mod plans;

pub mod feature_adapter;
pub mod metrics;
pub mod olu_reporter;

// 公共 re-export
pub use error::{Error, Result};
pub use feature_adapter::{
    FeatureRegistry, PfauTransition, RealmLifecycleFeatureAdapter, SubFeatureRegistration,
};
pub use metrics::LcmMetrics;
pub use olu_reporter::{OluPhase, OluReport, OluReporter};
pub use service::RealmLifecycleService;
