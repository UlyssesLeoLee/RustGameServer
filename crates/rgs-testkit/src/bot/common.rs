//! common proto 模块 (per DDD Review v0.3.2 §7.3 L1.2 wave 4 真实 RPC 接入)
//!
//! 用途: admin proto 的 generated code 引用 `super::super::common::v1::*`
//! (EntityId / Timestamp / HealthCheckRequest / HealthCheckResponse / Status
//! 等), 需要在 `bot` 模块下提供 `common::v1` 兄弟模块供其解析.
//!
//! # 强约束
//!
//! - **不**对 common proto 二次封装, 仅做 `tonic::include_proto!("common.v1")`
//! - 单一来源: 复用 `../shared-platform/proto/common/v1/common.proto` (per
//!   admin-service/build.rs 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
//! - L-CAND-016 防御: common 是 admin proto 强依赖, 不算"5 worker 公共 proto
//!   RPC 调用", 仅为 admin proto 服务的间接依赖
//!
//! # 调用方
//!
//! `bot::gm::admin_proto` (admin proto client) 通过 `super::super::common::v1`
//! 路径访问. 注意此模块**不应**被 bot 业务代码直接使用 — 业务代码应走
//! `GmClient` / `BotAi` 等高层 API, 不直接操作 proto 类型.

/// common.v1 生成的 gRPC 类型 (EntityId / Timestamp / HealthCheckRequest / 等)
pub mod v1 {
    tonic::include_proto!("common.v1");
}
