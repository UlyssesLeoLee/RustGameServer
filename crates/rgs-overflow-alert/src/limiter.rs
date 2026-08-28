//! 限流层 — 双阈值（软/硬）+ 排队出口
//!
//! 核心思路（per 任务 §1）：
//! - **硬上限** = k8s 单 Pod 理论承载（`<DOMAIN>_MAX_INFLIGHT`）
//! - **软阈值** = `ceil(hard × soft_ratio)`（默认 0.8）
//! - acquire 时：
//!   - `in_flight < soft` → `Pass`：原子 +1，业务持有 guard 到完成
//!   - `soft <= in_flight < hard` → `Queued`：原子 +1，业务入 NATS JS 队列后释放
//!   - `in_flight >= hard` → `Rejected`：原子 reject 计数 +1
//!
//! **0 硬上限 = 不启用**：所有 `acquire` 直接返 `Pass`（业务未上线时降级）

use crate::config::OverflowConfig;
use crate::domain::Domain;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

/// acquire 结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquireOutcome {
    /// 在软阈值内放行（含 0 硬上限场景）
    Pass,
    /// 软阈值已超：入 NATS JS 队列
    Queued,
    /// 硬上限已满：拒绝 + 触发告警
    Rejected,
}

impl AcquireOutcome {
    pub fn is_pass(self) -> bool {
        matches!(self, AcquireOutcome::Pass)
    }
    pub fn is_queued(self) -> bool {
        matches!(self, AcquireOutcome::Queued)
    }
    pub fn is_rejected(self) -> bool {
        matches!(self, AcquireOutcome::Rejected)
    }
}

/// 限流器错误（结构性，本枚举暂未使用）
#[derive(Debug, Error)]
pub enum AcquireError {
    #[error("acquire failed")]
    Failed,
}

/// RAII guard：drop 时自动释放 in_flight 计数
#[derive(Debug)]
pub struct InFlightGuard {
    counter: Arc<AtomicU32>,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        // saturating_sub 避免负数
        let prev = self
            .counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(v.saturating_sub(1))
            });
        let _ = prev; // 即使 overflow 也不 panic
    }
}

/// 限流器包装
#[derive(Clone)]
pub struct OverflowLimiter {
    domain: Domain,
    /// `None` = 不启用（hard_cap = 0）；启用时存 in_flight 计数
    counter: Option<Arc<AtomicU32>>,
    /// 软阈值
    soft: u32,
    /// 硬上限
    hard: u32,
    /// 5min reject 计数（用于告警正文 / Prometheus）
    reject_window: Arc<AtomicU64>,
    /// 上次重置时间戳（unix secs）
    reject_window_start: Arc<AtomicU64>,
}

impl OverflowLimiter {
    /// 从 `OverflowConfig` 给定域构造
    ///
    /// `hard_cap == 0` → 构造一个"空"限流器（所有 acquire 直接 `Pass`）
    pub fn new(domain: Domain, cfg: &OverflowConfig) -> Self {
        let hard = cfg.hard_cap(domain);
        let soft = cfg.soft_cap(domain);
        let counter = if hard == 0 {
            None
        } else {
            Some(Arc::new(AtomicU32::new(0)))
        };
        Self {
            domain,
            counter,
            soft,
            hard,
            reject_window: Arc::new(AtomicU64::new(0)),
            reject_window_start: Arc::new(AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            )),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.counter.is_some()
    }

    pub fn hard_cap(&self) -> u32 {
        self.hard
    }

    pub fn soft_cap(&self) -> u32 {
        self.soft
    }

    /// 当前 in-flight 占用
    pub fn in_flight(&self) -> u32 {
        self.counter
            .as_ref()
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// 当前可用 = hard - in_flight
    pub fn available_permits(&self) -> u32 {
        self.hard.saturating_sub(self.in_flight())
    }

    /// 5min reject 窗口计数
    pub fn reject_count_5min(&self) -> u64 {
        const WINDOW_SECS: u64 = 300;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let start = self.reject_window_start.load(Ordering::Relaxed);
        if now.saturating_sub(start) > WINDOW_SECS {
            self.reject_window_start.store(now, Ordering::Relaxed);
            self.reject_window.store(0, Ordering::Relaxed);
        }
        self.reject_window.load(Ordering::Relaxed)
    }

    /// 同步获取（CAS +1）：
    /// - 不启用 → 直接 Pass
    /// - in_flight+1 后 ≤ soft → Pass（持有 guard）
    /// - in_flight+1 后 < hard → Queued（持有 guard）
    /// - 否则 → Rejected（不 +1，revert）
    ///
    /// 实现：手写 loop + compare_exchange，确保并发安全
    /// （之前用 fetch_update 在 1000 并发下出现 race — 乐观重试让 in_flight 突破 hard）
    ///
    /// 返回 `AcquireOutcome` + 持有 guard（Pass/Queued 时 `Some`）
    pub fn try_acquire(&self) -> (AcquireOutcome, Option<InFlightGuard>) {
        let Some(counter) = self.counter.as_ref() else {
            return (AcquireOutcome::Pass, None);
        };
        // 限流语义：每个 task 只有 1 次"抢"机会
        // - 看一眼 current，current < hard → CAS +1
        // - CAS 成功 → Pass/Queued（拿到 permit）
        // - CAS 失败 → Rejected（已被其他 task 抢先 +1；不重试，避免乐观重试让所有 task 都 +1）
        // - current >= hard → Rejected
        let current = counter.load(Ordering::Acquire);
        if current >= self.hard {
            self.reject_window.fetch_add(1, Ordering::Relaxed);
            return (AcquireOutcome::Rejected, None);
        }
        let next = current + 1;
        match counter.compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {
                // CAS 成功：next = in_flight_after
                if next <= self.soft {
                    (
                        AcquireOutcome::Pass,
                        Some(InFlightGuard {
                            counter: counter.clone(),
                        }),
                    )
                } else {
                    (
                        AcquireOutcome::Queued,
                        Some(InFlightGuard {
                            counter: counter.clone(),
                        }),
                    )
                }
            }
            Err(actual) => {
                // CAS 失败：被其他 task 抢先 +1
                // 关键：直接 Rejected，**不重试**（否则乐观重试会让所有并发 task 都 +1 成功，破坏限流）
                self.reject_window.fetch_add(1, Ordering::Relaxed);
                let _ = actual; // 实际值用于诊断
                (AcquireOutcome::Rejected, None)
            }
        }
    }

    /// 域
    pub fn domain(&self) -> Domain {
        self.domain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OverflowConfig;
    use crate::test_utils::{clear_all_overflow_env, lock_env, set_envs};

    #[test]
    fn disabled_when_hard_cap_zero() {
        let _g = lock_env();
        clear_all_overflow_env();
        let cfg = OverflowConfig::from_env().unwrap();
        let lim = OverflowLimiter::new(Domain::Player, &cfg);
        assert!(!lim.is_enabled());
        // 不启用 → 全部 Pass
        for _ in 0..1000 {
            assert_eq!(lim.try_acquire().0, AcquireOutcome::Pass);
        }
    }

    #[tokio::test]
    async fn pass_below_soft_then_queued_then_rejected() {
        let _g = lock_env();
        clear_all_overflow_env();
        set_envs(&[
            ("PLAYER_MAX_INFLIGHT", "10"),
            ("NATS_OVERFLOW_SOFT_RATIO", "0.5"),
        ]);
        let cfg = OverflowConfig::from_env().unwrap();
        let lim = OverflowLimiter::new(Domain::Player, &cfg);
        assert!(lim.is_enabled());
        assert_eq!(lim.hard_cap(), 10);
        assert_eq!(lim.soft_cap(), 5);

        // 占 5 个 permit（软阈值内）— 全部 Pass
        let mut permits = Vec::new();
        for _ in 0..5 {
            let (out, p) = lim.try_acquire();
            assert_eq!(out, AcquireOutcome::Pass);
            permits.push(p.expect("permit"));
        }
        assert_eq!(lim.in_flight(), 5);
        assert_eq!(lim.available_permits(), 5);

        // 第 6 个：仍在硬上限内，但 ≤ soft → Queued
        let (out, p) = lim.try_acquire();
        assert_eq!(out, AcquireOutcome::Queued);
        permits.push(p.expect("permit"));
        assert_eq!(lim.in_flight(), 6);

        // 占满 10
        for _ in 0..4 {
            let (out, p) = lim.try_acquire();
            assert_eq!(out, AcquireOutcome::Queued);
            permits.push(p.expect("permit"));
        }
        assert_eq!(lim.in_flight(), 10);
        assert_eq!(lim.available_permits(), 0);

        // 第 11 个：硬上限已满 → Rejected
        let (out, p) = lim.try_acquire();
        assert_eq!(out, AcquireOutcome::Rejected);
        assert!(p.is_none());
        assert_eq!(lim.reject_count_5min(), 1);
    }

    #[tokio::test]
    async fn release_restores_permits() {
        let _g = lock_env();
        clear_all_overflow_env();
        // hard=10, soft_ratio=0.5 → soft=5
        set_envs(&[
            ("ECONOMY_MAX_INFLIGHT", "10"),
            ("NATS_OVERFLOW_SOFT_RATIO", "0.5"),
        ]);
        let cfg = OverflowConfig::from_env().unwrap();
        let lim = OverflowLimiter::new(Domain::Economy, &cfg);
        // 占 2 个（< soft），全部 Pass
        let _p1 = lim.try_acquire().1.expect("permit");
        let _p2 = lim.try_acquire().1.expect("permit");
        assert_eq!(lim.in_flight(), 2);
        // 释放 _p1
        drop(_p1);
        // permit 释放后又有可用
        let (out, p) = lim.try_acquire();
        assert!(out.is_pass());
        assert!(p.is_some());
    }
}
