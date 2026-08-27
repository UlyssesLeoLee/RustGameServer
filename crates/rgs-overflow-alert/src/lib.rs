//! rgs-overflow-alert —— 5 域业务服务超限排队 + 邮件告警中间件
//!
//! 职责（per 2026-08-27 Ulysses 拍板 / task prompt §0）：
//! - **限流**：双阈值（软 = 硬 × `NATS_OVERFLOW_SOFT_RATIO` / 硬 = `<DOMAIN>_MAX_INFLIGHT`）
//! - **排队**：超软未超硬 → 入 NATS JetStream 队列（`rgs.<domain>.overflow.v1`，与 RGS-SPEC-CROSS-005 + RGS-DTL-100 §5 一致）
//! - **告警**：超硬 → 拒绝 + 邮件告警；`SMTP_PASSWORD` 为空时降级到 `tracing::warn!`（不抛错）
//!
//! ## 模块
//!
//! - [`config`]    `OverflowConfig::from_env()` 统一从 env 读全部配置
//! - [`domain`]    `Domain` 枚举（Player / Economy / Match / Social — 不含 admin / cluster-ops）
//! - [`limiter`]   `OverflowLimiter`（tokio::Semaphore + 双阈值判定）
//! - [`queue`]     `QueueBackend` trait + `NatsJsQueueBackend`（生产）+ 测试 mock 留 tests/
//! - [`alert`]     `AlertSink` trait + `SmtpAlertSink`（lettre）+ `LogOnlySink` + `AlertDeduplicator`
//! - [`guard`]     `OverflowGuard` — 业务层最常用的高层 API（`check()` → `Result<(), Status>`）
//!
//! ## 范围
//!
//! **作用**：player / economy / match / social 4 个业务域
//! **排除**：admin（COC 控制面）、cluster-ops（Active-Active + saga_store）
//! **域类型系统防越界**：`Domain` 枚举无 admin/cluster-ops 变体，编译期拒绝误用
//!
//! ## SMTP 密码降级约定
//!
//! `SMTP_PASSWORD` env 缺失或为空 → `OverflowConfig` 标记 `smtp_password = ""`，
//! `AlertDeduplicator` 默认用 `LogOnlySink`，**不抛错，不阻断入队**。
//! 真实密码走 k8s Secret，**不入** .env 提交历史（per .env.example §8 注释）。

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::result_large_err)]
// 测试模块允许 unsafe（env::set_var 在 Rust 1.98 需 unsafe 块）
#![cfg_attr(test, allow(unsafe_code))]

pub mod alert;
pub mod config;
pub mod domain;
pub mod guard;
pub mod limiter;
pub mod queue;

#[cfg(test)]
pub(crate) mod test_utils;

pub use alert::{
    AlertDeduplicator, AlertError, AlertEvent, AlertKind, AlertSink, LogOnlySink, SmtpAlertSink,
};
pub use config::{ConfigError, OverflowConfig};
pub use domain::Domain;
pub use guard::{OverflowGuard, OverflowStatus};
pub use limiter::{AcquireError, AcquireOutcome, OverflowLimiter};
pub use queue::{AckToken, NatsJsQueueBackend, QueueBackend, QueueError};
