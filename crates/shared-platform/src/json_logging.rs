//! JSON 结构化日志（per RGS-ARC-051 观测 + ELK / Loki 对接）
//!
//! 54.14 实化：JSON layer + 业务上下文传播
//!
//! 设计：
//! - tracing_subscriber::fmt::layer().json() 输出 JSON 行
//! - 当前 Span 字段（request_id / saga_id / actor_id）自动合并到日志
//! - 业务调用 helper：with_request_id / with_saga_id
//!
//! 用法：
//! ```no_run
//! shared_platform::json_logging::init_json_logging("info").unwrap();
//! ```
//!
//! **互斥约束**：init_json_logging 与 init_tracing（tracing_init 模块）
//! 互斥 — tracing_subscriber 全局只能一个 subscriber。
//! 二选一：JSON 日志（init_json_logging）OR OTel 桥接（init_tracing），
//! 不可同时调用。

use std::sync::OnceLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

/// JSON 日志初始化错误
#[derive(Debug, Error)]
pub enum JsonLogError {
    #[error("subscriber init error: {0}")]
    SubscriberInit(String),
}

use thiserror::Error;

/// 全局 init flag（避免重复 init）
static INIT_FLAG: OnceLock<()> = OnceLock::new();

/// 初始化 JSON 日志（per ELK / Loki）
pub fn init_json_logging(default_filter: &str) -> Result<(), JsonLogError> {
    if INIT_FLAG.get().is_some() {
        return Ok(()); // 重复 init 静默成功
    }
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let json_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .try_init()
        .map_err(|e| JsonLogError::SubscriberInit(e.to_string()))?;

    INIT_FLAG.set(()).ok();
    tracing::info!(target: "json_logging", "JSON structured logging initialized");
    Ok(())
}

/// 业务 helper：with_request_id span（request_id 跨 span 传播）
pub fn with_request_id<F, R>(request_id: Uuid, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("request", request_id = %request_id);
    span.in_scope(f)
}

/// 业务 helper：with_saga_id span
pub fn with_saga_id<F, R>(saga_id: Uuid, saga_type: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("saga", saga_id = %saga_id, saga_type = %saga_type);
    span.in_scope(f)
}

/// 业务 helper：with_actor span（玩家 / 管理员）
pub fn with_actor<F, R>(actor_id: Uuid, actor_type: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("actor", actor_id = %actor_id, actor_type = %actor_type);
    span.in_scope(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_request_id_executes() {
        let result = with_request_id(Uuid::new_v4(), || 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn with_saga_id_executes() {
        let result = with_saga_id(Uuid::new_v4(), "transfer", || "ok");
        assert_eq!(result, "ok");
    }
}
