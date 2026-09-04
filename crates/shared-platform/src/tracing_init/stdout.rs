//! Stdout tracing 路径（per P2-9 拆分，2026-09-05）
//!
//! 给 dev/test 用：纯 stdout + EnvFilter，不走 OTel。
//! 跟 OTel 路径互斥（全局 tracing subscriber 只能 1 个）。
//!
//! **新增**（原 tracing_init.rs 没单独 stdout 路径，53.12 拆分时一并加）：
//! - `init_stdout_tracing(service_name)` 装 EnvFilter + fmt layer

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// 初始化 stdout tracing（dev/test 路径）
///
/// EnvFilter 从 RUST_LOG 读，缺省 `info,sqlx=warn,hyper=warn,h2=warn`。
/// fmt layer 走 compact 模式（控制台友好）。
///
/// **与 OTel 路径互斥**：本函数调 `try_init()`，已 init 过的 subscriber 会失败返回 Err。
pub fn init_stdout_tracing(service_name: &str) -> Result<(), crate::tracing_init::TracingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,hyper=warn,h2=warn"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(false)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| crate::tracing_init::TracingError::SubscriberInit(e.to_string()))?;

    tracing::info!(
        target: "tracing_init::stdout",
        service = service_name,
        "stdout tracing initialized (no OTel)"
    );
    Ok(())
}
