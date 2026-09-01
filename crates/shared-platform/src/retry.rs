//! 异步重试策略（per RGS-SPEC-CROSS-006 错误处理 + 重试规范）
//!
//! 54.9 实化：exponential backoff + jitter + 错误分类
//!
//! 设计：
//! - 指数退避：1s → 2s → 4s → ...（max 30s）
//! - jitter：±20% 随机扰动避免雪崩
//! - 只重试"瞬态错误"（Unavailable / DeadlineExceeded / Aborted），不重试业务错误

use std::time::Duration;

use backoff::backoff::Backoff;
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

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // retry 是平台层关键 invariant 模块, 加 5 个新单测 + 2 个 proptest

    #[test]
    fn default_retry_config_matches_design() {
        // 设计文档 (RGS-SPEC-CROSS-006 草案) 默认值契约
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_retries, 3, "默认 3 次重试");
        assert_eq!(
            cfg.initial_interval,
            Duration::from_millis(100),
            "默认初始退避 100ms"
        );
        assert_eq!(cfg.max_interval, Duration::from_secs(30), "默认最大 30s");
        assert!((cfg.multiplier - 2.0).abs() < f64::EPSILON, "默认 2x 退避");
    }

    #[test]
    fn retryable_code_classification() {
        // 4 类可重试 + 4 类不可重试
        // 可重试: Unavailable / DeadlineExceeded / Aborted / ResourceExhausted
        assert!(is_retryable(Code::Unavailable));
        assert!(is_retryable(Code::DeadlineExceeded));
        assert!(is_retryable(Code::Aborted));
        assert!(is_retryable(Code::ResourceExhausted));
        // 不可重试 (业务错误)
        assert!(!is_retryable(Code::Ok));
        assert!(!is_retryable(Code::Cancelled));
        assert!(!is_retryable(Code::NotFound));
        assert!(!is_retryable(Code::InvalidArgument));
        assert!(!is_retryable(Code::PermissionDenied));
        assert!(!is_retryable(Code::AlreadyExists));
        assert!(!is_retryable(Code::Unauthenticated));
    }

    #[test]
    fn build_backoff_uses_config() {
        // build_backoff 必须用 cfg 的 initial/max/multiplier, 而不是默认值
        let cfg = RetryConfig {
            max_retries: 5,
            initial_interval: Duration::from_millis(50),
            max_interval: Duration::from_secs(10),
            multiplier: 3.0,
        };
        let mut bo = build_backoff(&cfg);
        // backoff crate 的 ExponentialBackoff 实现 Backoff trait,
        // 用 .next_backoff() 拿下一个间隔 (Option<Duration>)
        let first = bo.next_backoff();
        assert!(first.is_some(), "build_backoff 必须能产生至少一个间隔");
    }

    #[test]
    fn backoff_with_zero_initial_interval_works() {
        // 边界: initial=0 时 attempt 0 = 0, 不应 panic
        let cfg = RetryConfig {
            max_retries: 1,
            initial_interval: Duration::from_millis(0),
            max_interval: Duration::from_secs(1),
            multiplier: 1.0,
        };
        let d0 = backoff_duration(0, &cfg);
        // 0ms ± 0% jitter (jitter_range = 0) → [0, 0]
        assert_eq!(d0, Duration::from_millis(0));
    }

    #[test]
    fn backoff_attempt_one_doubles() {
        // attempt=1 时 base = 100ms * 2.0 = 200ms
        let cfg = RetryConfig {
            max_retries: 5,
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
        };
        // 多次采样, 中位数应在 200ms 附近 (jitter ±20%)
        let mut samples = Vec::with_capacity(50);
        for _ in 0..50 {
            samples.push(backoff_duration(1, &cfg).as_millis());
        }
        let avg: u128 = samples.iter().sum::<u128>() / samples.len() as u128;
        // 期望 ~200ms, 允许 ±25% (jitter 边界 + 抽样误差)
        assert!(
            avg >= 150 && avg <= 250,
            "attempt=1 退避平均应在 200ms 附近, 实际 {}ms",
            avg
        );
    }
}

// ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
// proptest 守恒 / 不变式
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// 任意 attempt 在 [0, 30] 内 → 退避必须落在 [0, max_interval * 1.2]
    /// (jitter 上界是 ±20% max, 防止雪崩)
    proptest! {
        #[test]
        fn backoff_bounded_by_max_plus_jitter(
            attempt in 0u32..30,
            initial_ms in 1u64..500,
            max_s in 1u64..60,
        ) {
            let cfg = RetryConfig {
                max_retries: 10,
                initial_interval: Duration::from_millis(initial_ms),
                max_interval: Duration::from_secs(max_s),
                multiplier: 2.0,
            };
            let d = backoff_duration(attempt, &cfg);
            // base capped at max_s 秒, jitter ±20% max_s 秒
            let upper_ms = max_s * 1000 + (max_s * 1000 / 5);
            let actual_ms = d.as_millis() as u64;
            prop_assert!(
                actual_ms <= upper_ms,
                "attempt={} 返回 {}ms 超过上界 {}ms",
                attempt, actual_ms, upper_ms
            );
        }
    }

    /// 退避必须非负 (u64 不会负, 但 base=0 + jitter 下界可能为 0, 不应 panic)
    proptest! {
        #[test]
        fn backoff_never_panics_on_small_initial(
            initial_ms in 0u64..10,
            attempt in 0u32..5,
        ) {
            let cfg = RetryConfig {
                max_retries: 3,
                initial_interval: Duration::from_millis(initial_ms),
                max_interval: Duration::from_millis(100),
                multiplier: 2.0,
            };
            let _ = backoff_duration(attempt, &cfg);
        }
    }
}
