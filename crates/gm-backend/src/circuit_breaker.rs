//! gm-backend AdminGrpcClient circuit breaker (W18 2026-08-28)
//!
//! 5 次连续失败 → 断开 30s,期间所有 RPC 直接返 Err(避免 cascade failure)
//! 30s 后允许 1 个 probe,成功 → 关闭,失败 → 重置 30s 窗口
//!
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md
//! 关联: RGS-OPEN-QA v0.4 DDD Review 决议 (per 8/27 13:23 JST 指令)

use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 关闭: 正常 RPC
    Closed,
    /// 打开: 30s 窗口内所有 RPC 立即返 Err
    Open,
    /// 半开: 允许 1 个 probe 验证下游是否恢复
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Mutex<CircuitInner>,
    failure_threshold: u32,
    open_duration: Duration,
}

struct CircuitInner {
    state: CircuitState,
    consecutive_failures: u32,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, open_duration: Duration) -> Self {
        Self {
            state: Mutex::new(CircuitInner {
                state: CircuitState::Closed,
                consecutive_failures: 0,
                opened_at: None,
            }),
            failure_threshold,
            open_duration,
        }
    }

    /// 决定是否允许本次 RPC 调用
    /// - Closed: 允许
    /// - Open: 若 open_duration 已过, 转 HalfOpen 允许 probe; 否则拒绝
    /// - HalfOpen: 仅允许 1 个, 后续拒绝 (避免 thundering herd)
    pub fn try_acquire(&self) -> bool {
        let mut inner = self.state.lock().unwrap();
        match inner.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(opened_at) = inner.opened_at {
                    if opened_at.elapsed() >= self.open_duration {
                        // 转 HalfOpen 允许 probe
                        inner.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => false, // 仅 1 个 probe, 已通过 try_acquire 放行
        }
    }

    /// 记录 RPC 成功
    pub fn record_success(&self) {
        let mut inner = self.state.lock().unwrap();
        inner.consecutive_failures = 0;
        inner.state = CircuitState::Closed;
        inner.opened_at = None;
    }

    /// 记录 RPC 失败
    /// - Closed: 累计 failures,达阈值转 Open
    /// - HalfOpen: probe 失败 → 立即转 Open 重置窗口
    /// - Open: 失败计数无关 (已经拒绝)
    pub fn record_failure(&self) {
        let mut inner = self.state.lock().unwrap();
        match inner.state {
            CircuitState::Closed => {
                inner.consecutive_failures += 1;
                if inner.consecutive_failures >= self.failure_threshold {
                    inner.state = CircuitState::Open;
                    inner.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                // probe 失败, 立即重置
                inner.state = CircuitState::Open;
                inner.opened_at = Some(Instant::now());
                inner.consecutive_failures = self.failure_threshold;
            }
            CircuitState::Open => {
                // already open, 不变
            }
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state.lock().unwrap().state
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_to_open_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert!(cb.try_acquire());
        cb.record_failure();
        cb.record_failure();
        assert!(cb.try_acquire(), "still closed after 2 fails");
        cb.record_failure();
        assert!(!cb.try_acquire(), "open after 3 fails");
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn open_to_half_open_after_duration() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(60));
        assert!(cb.try_acquire(), "transition to half-open allows probe");
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_success_closes() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(60));
        assert!(cb.try_acquire());
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_failure_reopens() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(60));
        assert!(cb.try_acquire()); // probe
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.try_acquire());
    }

    #[test]
    fn success_resets_failure_count() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        cb.record_failure();
        cb.record_failure();
        assert!(
            cb.try_acquire(),
            "still closed (counter reset after success)"
        );
    }
}
