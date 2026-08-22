//! 异步重试策略（per RGS-SPEC-CROSS-006 错误处理 + 重试规范）
//!
//! 54.9 实化：exponential backoff + jitter + 错误分类
//!
//! 设计：
//! - 指数退避：1s → 2s → 4s → ...（max 30s）
//! - jitter：±20% 随机扰动避免雪崩
//! - 只重试"瞬态错误"（Unavailable / DeadlineExceeded / Aborted），不重试业务错误

use std::time::Duration;

use backoff::ExponentialBackoffBuilder;
use tonic::Code;

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始退避
    pub initial_interval: Duration,
    /// 最大退避
    pub max_interval: Duration,
    /// 乘数
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// 构造 exponential backoff（per `backoff` crate 0.4）
pub fn build_backoff(cfg: &RetryConfig) -> backoff::ExponentialBackoff {
    ExponentialBackoffBuilder::new()
        .with_initial_interval(cfg.initial_interval)
        .with_max_interval(cfg.max_interval)
        .with_multiplier(cfg.multiplier)
        .with_max_elapsed_time(None)
        .build()
}

/// 判断 gRPC 状态码是否可重试（per RGS-SPEC-CROSS-006 草案）
pub fn is_retryable(code: Code) -> bool {
    matches!(
        code,
        Code::Unavailable        // 服务器不可用（瞬态）
            | Code::DeadlineExceeded // 超时
            | Code::Aborted         // 中止（可能可重试）
            | Code::ResourceExhausted // 资源耗尽（可能瞬态）
    )
}

/// 计算第 N 次重试的退避时长（带 jitter）
pub fn backoff_duration(attempt: u32, cfg: &RetryConfig) -> Duration {
    let base_ms = (cfg.initial_interval.as_millis() as f64) * cfg.multiplier.powi(attempt as i32);
    let capped_ms = base_ms.min(cfg.max_interval.as_millis() as f64) as u64;
    // ±20% jitter
    let jitter_range = (capped_ms as f64 * 0.2) as u64;
    let jitter = (rand_u64() % (jitter_range * 2 + 1)).saturating_sub(jitter_range);
    Duration::from_millis(
        capped_ms
            .saturating_add(jitter)
            .saturating_sub(jitter_range),
    )
}

/// 简化随机数（per RGS-AI 不引入新 dep 原则，用 std time hash）
fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_codes() {
        assert!(is_retryable(Code::Unavailable));
        assert!(is_retryable(Code::DeadlineExceeded));
        assert!(is_retryable(Code::Aborted));
        assert!(!is_retryable(Code::NotFound));
        assert!(!is_retryable(Code::InvalidArgument));
        assert!(!is_retryable(Code::PermissionDenied));
    }

    #[test]
    fn backoff_grows_exponentially() {
        let cfg = RetryConfig::default();
        let d0 = backoff_duration(0, &cfg);
        let d2 = backoff_duration(2, &cfg);
        // d2 应当显著大于 d0
        assert!(d2 > d0);
    }

    #[test]
    fn backoff_caps_at_max() {
        let cfg = RetryConfig::default();
        let d100 = backoff_duration(100, &cfg);
        // 应当被 max_interval 限制
        assert!(d100 <= cfg.max_interval + cfg.max_interval); // 含 jitter 边界
    }
}
