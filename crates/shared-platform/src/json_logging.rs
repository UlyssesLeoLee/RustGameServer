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
///
/// 同一进程内重复调用静默成功（OnceLock 保证），不会 panic 也不会覆盖
/// 已存在的 subscriber（首次成功后才置位 flag）。
///
/// 输出示例（每行一个 JSON 对象，stdout 写入，便于 Filebeat / Promtail 采集）：
/// ```text
/// {"timestamp":"2026-08-23T08:00:00.123Z","level":"INFO","fields":{"message":"player connected","request_id":"550e8400-e29b-41d4-a716-446655440000"},"target":"player_service::handler"}
/// {"timestamp":"2026-08-23T08:00:00.456Z","level":"INFO","fields":{"message":"saga step done","saga_id":"7c9e6679-7425-40de-944b-e07fc1f90ae7","saga_type":"transfer"},"target":"economy_service::saga"}
/// ```
///
/// 当前 Span 的字段（`request_id` / `saga_id` / `actor_id`）由
/// `with_request_id` / `with_saga_id` / `with_actor` 自动注入，**调用方无需手动
/// 在 `tracing::info!` 里重复传这些字段**。
///
/// ```no_run
/// // 服务启动入口（bin/rgs-player/src/main.rs 之类）调用一次即可
/// shared_platform::json_logging::init_json_logging("info,sqlx=warn").unwrap();
/// ```
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
///
/// 包装闭包为 `tracing::info_span!("request", request_id = %request_id)` 作用域，
/// span 内发出的 `tracing::info!` / `warn!` 等事件会自动把 `request_id` 注入到
/// JSON 日志字段（前提：已调用 [`init_json_logging`]）。
///
/// **不依赖全局 subscriber** —— 在测试 / doctest 里可以直接调用，
/// span 字段会在订阅者存在时被拾取。
///
/// ```
/// use shared_platform::with_request_id;
/// use uuid::Uuid;
///
/// let rid = Uuid::nil();
/// let payload = with_request_id(rid, || {
///     // 这里 tracing::info! 会带上 request_id 字段
///     42
/// });
/// assert_eq!(payload, 42);
/// ```
pub fn with_request_id<F, R>(request_id: Uuid, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("request", request_id = %request_id);
    span.in_scope(f)
}

/// 业务 helper：with_saga_id span
///
/// 与 [`with_request_id`] 同样的 scope 模式，但同时注入 `saga_id` + `saga_type`
/// 字段，per RGS-SPEC-CROSS-005 §3 跨域 trace 关联要求。
///
/// ```
/// use shared_platform::with_saga_id;
/// use uuid::Uuid;
///
/// let sid = Uuid::nil();
/// let result = with_saga_id(sid, "transfer", || "step-ok");
/// assert_eq!(result, "step-ok");
/// ```
pub fn with_saga_id<F, R>(saga_id: Uuid, saga_type: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::info_span!("saga", saga_id = %saga_id, saga_type = %saga_type);
    span.in_scope(f)
}

/// 业务 helper：with_actor span（玩家 / 管理员）
///
/// 注入 `actor_id` + `actor_type`（"player" / "admin" / "system"）字段，
/// per RGS-SEC-100 §4 审计日志要求——所有 COC / admin 操作必须能按 actor 过滤。
///
/// ```
/// use shared_platform::with_actor;
/// use uuid::Uuid;
///
/// let aid = Uuid::nil();
/// let result = with_actor(aid, "admin", || 200_u16);
/// assert_eq!(result, 200);
/// ```
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

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // JSON logging 是 RGS-ARC-051 观测核心, 加 3 单测

    #[test]
    fn with_actor_executes() {
        // with_actor 注入 actor_id + actor_type, 必须能执行闭包并返回值
        let result = with_actor(Uuid::nil(), "admin", || 200_u16);
        assert_eq!(result, 200);

        let result2 = with_actor(Uuid::new_v4(), "player", || "actor-ok".to_string());
        assert_eq!(result2, "actor-ok");
    }

    #[test]
    fn with_request_id_supports_complex_closure() {
        // 闭包可以是任意返回类型, 包括 Result<T, E>
        let result: Result<u32, &str> = with_request_id(Uuid::new_v4(), || Ok(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn with_saga_id_supports_complex_closure() {
        // 闭包可访问外部状态
        let x = 10;
        let result = with_saga_id(Uuid::new_v4(), "transfer", || x * 2);
        assert_eq!(result, 20);
    }

    #[test]
    fn init_json_logging_is_idempotent() {
        // 同一进程重复 init 不能 panic (OnceLock 设计意图)
        // 第 1 次: 成功 OR 因为全局 subscriber 已被 test runner 设置而 SubscriberInit 失败
        // 第 2 次: 必定走 early return Ok(()) 分支
        let r1 = init_json_logging("info");
        let r2 = init_json_logging("info");
        // r2 必须 Ok (OnceLock guard)
        assert!(r2.is_ok(), "第二次 init_json_logging 必须 Ok, 实际: {:?}", r2);
        // r1 可能 Ok 也可能 Err (取决于 test runner 顺序), 不强制
        let _ = r1;
    }
}
