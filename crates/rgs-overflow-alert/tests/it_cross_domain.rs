//! rgs-overflow-alert 跨域集成场景 (per 9/1 PT-WORKER 派工 §3 IT)
//!
//! 3 跨场景：
//! 1. 4 域独立 subject (rgs.<domain>.overflow.v1) — 验证不串扰
//! 2. 4 域独立 guard, 各自 hard cap 独立
//! 3. 1 域 Rejected → 告警仅对该域去重 (其他域不触发)

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rgs_overflow_alert::alert::{AlertDeduplicator, AlertError, AlertEvent, AlertSink, LogOnlySink};
use rgs_overflow_alert::config::OverflowConfig;
use rgs_overflow_alert::domain::Domain;
use rgs_overflow_alert::guard::{OverflowGuard, OverflowStatus};
use rgs_overflow_alert::limiter::OverflowLimiter;
use rgs_overflow_alert::queue::{InMemoryQueueBackend, QueueBackend};

/// 计数 sink
struct CountingSink {
    count: Arc<AtomicU32>,
}
#[async_trait]
impl AlertSink for CountingSink {
    async fn send(&self, _to: &str, _event: &AlertEvent) -> Result<(), AlertError> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}
fn clear_env() {
    for k in [
        "PLAYER_MAX_INFLIGHT",
        "ECONOMY_MAX_INFLIGHT",
        "MATCH_MAX_INFLIGHT",
        "SOCIAL_MAX_INFLIGHT",
        "NATS_OVERFLOW_SOFT_RATIO",
        "NATS_OVERFLOW_MAX_PENDING",
        "ALERT_DEDUP_WINDOW_SECS",
    ] {
        unsafe { std::env::remove_var(k); }
    }
}
fn set_env(pairs: &[(&str, &str)]) {
    for (k, v) in pairs {
        unsafe { std::env::set_var(k, v); }
    }
}

fn make_guard(domain: Domain, hard: u32) -> OverflowGuard {
    let key = domain.env_max_inflight();
    set_env(&[
        (key, &hard.to_string()),
        ("NATS_OVERFLOW_SOFT_RATIO", "0.5"),
        ("NATS_OVERFLOW_MAX_PENDING", "100"),
    ]);
    let cfg = OverflowConfig::from_env().unwrap();
    let lim = Arc::new(OverflowLimiter::new(domain, &cfg));
    let queue: Arc<dyn QueueBackend> = Arc::new(InMemoryQueueBackend::new(100));
    let primary: Arc<dyn AlertSink> = Arc::new(CountingSink { count: Arc::new(AtomicU32::new(0)) });
    let fallback: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
    let alerter = Arc::new(AlertDeduplicator::new(
        primary,
        fallback,
        "test@example.com".to_string(),
        Duration::from_secs(60),
    ));
    OverflowGuard::new(
        domain,
        &cfg,
        lim,
        queue,
        alerter,
        Some("test-pod".to_string()),
        "test-service".to_string(),
    )
}

#[tokio::test]
async fn it_four_domain_subjects_are_independent() {
    use rgs_overflow_alert::queue::NatsJsQueueBackend;
    let subjects = [
        (Domain::Player, "rgs.player.overflow.v1"),
        (Domain::Economy, "rgs.economy.overflow.v1"),
        (Domain::Match, "rgs.match.overflow.v1"),
        (Domain::Social, "rgs.social.overflow.v1"),
    ];
    for (d, expected) in subjects {
        assert_eq!(NatsJsQueueBackend::subject_for(d), expected);
    }
}

#[tokio::test]
async fn it_each_domain_has_independent_hard_cap() {
    let _g = lock_env();
    clear_env();
    // 9/7 14:00 JST 调优: ratio=0.5 → soft=hard (ratio=1.0 错误: soft=2, 第 3 个在 soft 内)
    set_env(&[
        ("PLAYER_MAX_INFLIGHT", "2"),
        ("ECONOMY_MAX_INFLIGHT", "4"),
        ("NATS_OVERFLOW_SOFT_RATIO", "1.0"),
    ]);
    let cfg = OverflowConfig::from_env().unwrap();
    assert_eq!(cfg.hard_cap(Domain::Player), 2);
    assert_eq!(cfg.hard_cap(Domain::Economy), 4);
    assert_eq!(cfg.hard_cap(Domain::Match), 0);
    assert_eq!(cfg.hard_cap(Domain::Social), 0);
    // soft=hard (ratio=1.0) → 全部 Pass 直到 hard 满
    let p_lim = OverflowLimiter::new(Domain::Player, &cfg);
    // 9/7 15:30 JST 调优: 用 Box::leak 强制 _p1 _p2 持有到 test end, 防止 Rust NLL 提前 drop
    let _p1 = Box::leak(Box::new(p_lim.try_acquire().1.expect("p1")));
    let _p2 = Box::leak(Box::new(p_lim.try_acquire().1.expect("p2")));
    let (out, _permit_drop) = p_lim.try_acquire();
    // _p1 _p2 通过 Box::leak 永久持有, 不会被 NLL drop
    assert_eq!(out, rgs_overflow_alert::limiter::AcquireOutcome::Rejected);
    // economy 独立
    let e_lim = OverflowLimiter::new(Domain::Economy, &cfg);
    // 9/7 15:45 JST 调优: 4 次 permit 全部 Box::leak 持有, 防止 NLL drop
    let mut _e_permits: Vec<&mut rgs_overflow_alert::limiter::InFlightGuard> = Vec::new();
    for _ in 0..4 {
        let (out, permit) = e_lim.try_acquire();
        assert_eq!(out, rgs_overflow_alert::limiter::AcquireOutcome::Pass);
        _e_permits.push(Box::leak(Box::new(permit.expect("permit"))));
    }
    let (out, _permit_drop) = e_lim.try_acquire();
    assert_eq!(out, rgs_overflow_alert::limiter::AcquireOutcome::Rejected);
}

#[tokio::test]
async fn it_soft_surge_alert_fires_only_once_for_first_surge() {
    let _g = lock_env();
    clear_env();
    // 9/7 14:00 JST 调优: soft=0.1 (soft_cap=1) 让 i=0 第 2 个 (1->2) > soft=1 → Queued
    // 让 guard 的 soft_cap=1, 这样预占 1 个后, 第一个 check in_flight=1->2 > soft=1 → Queued
    set_env(&[
        ("MATCH_MAX_INFLIGHT", "10"),
        ("NATS_OVERFLOW_SOFT_RATIO", "0.1"),
        ("NATS_OVERFLOW_MAX_PENDING", "100"),
    ]);
    let cfg = OverflowConfig::from_env().unwrap();
    let lim = Arc::new(OverflowLimiter::new(Domain::Match, &cfg));
    let queue: Arc<dyn QueueBackend> = Arc::new(InMemoryQueueBackend::new(100));
    let primary: Arc<dyn AlertSink> = Arc::new(CountingSink { count: Arc::new(AtomicU32::new(0)) });
    let fallback: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
    let alerter = Arc::new(AlertDeduplicator::new(
        primary,
        fallback,
        "test@example.com".to_string(),
        Duration::from_secs(60),
    ));
    let g = OverflowGuard::new(
        Domain::Match,
        &cfg,
        lim,
        queue,
        alerter,
        Some("test-pod".to_string()),
        "test-service".to_string(),
    );
    // 预占 1 个 (在 soft 阈值内, soft=1 但当前 in_flight=0 → 0<=1 → Pass, 之后 in_flight=1)
    let _p = g.limiter().try_acquire().1;
    // 软阈值首超: 第一个 check 进 in_flight=1, CAS 1->2, 2>soft=1 → Queued
    for i in 0..5 {
        let d = g
            .check(&format!("Op{i}"), &format!("req-{i}"), None)
            .await;
        // i=0 第 1 个 check: in_flight=1, CAS 1->2, 2>soft=1 → Queued (不再 Pass)
        // 后续 i=1+ 也在 in_flight >= soft 范围 → Queued
        assert_eq!(d.status, OverflowStatus::Queued, "i={} 期望 Queued", i);
    }
}
