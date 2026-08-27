//! 共享测试工具（**仅** `#[cfg(test)]` 编译）
//!
//! **问题**：`std::env::set_var` 在 Rust 1.98 需 unsafe 块且**进程全局**，
//! cargo 默认并发跑 test 时多个 test 共写 env 会相互串扰。
//!
//! **解法**：所有读写 env 的 test 都通过 `lock_env()` 拿同一把进程级 Mutex，
//! 并发 → 串行。`lock_env()` 返回的 guard drop 时自动释放。
//!
//! **使用**：
//! ```ignore
//! use crate::test_utils::lock_env;
//! let _g = lock_env();
//! unsafe { std::env::set_var("FOO", "bar"); }
//! ```

use std::sync::{Mutex, MutexGuard};

/// 进程级 env 互斥锁（**不**Poison 阻断 — catch poison 后继续）
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// 拿环境变量锁；返回的 guard drop 时自动释放
pub fn lock_env() -> MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

/// 清除本 crate 涉及的全部 env（**所有** test 起始处调用）
pub fn clear_all_overflow_env() {
    for k in [
        "SUPPORT_EMAIL",
        "NATS_URI",
        "NATS_OVERFLOW_SOFT_RATIO",
        "NATS_OVERFLOW_STREAM",
        "NATS_OVERFLOW_CONSUMER_GROUP",
        "NATS_OVERFLOW_MAX_PENDING",
        "ALERT_DEDUP_WINDOW_SECS",
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USER",
        "SMTP_PASSWORD",
        "SMTP_FROM_NAME",
        "SMTP_TIMEOUT_MS",
        "PLAYER_MAX_INFLIGHT",
        "ECONOMY_MAX_INFLIGHT",
        "MATCH_MAX_INFLIGHT",
        "SOCIAL_MAX_INFLIGHT",
    ] {
        // SAFETY: serialized via ENV_LOCK
        unsafe {
            std::env::remove_var(k);
        }
    }
}

/// 一次性设置多个 env
///
/// SAFETY: caller must hold `lock_env()` before calling
pub fn set_envs(pairs: &[(&str, &str)]) {
    // SAFETY: caller must hold `lock_env()`
    unsafe {
        for (k, v) in pairs {
            std::env::set_var(k, v);
        }
    }
}
