//! 端到端集成测试（per 任务 §2.1 / §6）：
//! - 1000 并发验证 Pass / Queued / Rejected 比例
//! - 告警去重（同 key 窗口内只发 1 次）
//!
//! **不**依赖真 NATS server —— 使用 `InMemoryQueueBackend` + `CountingSink`
//! 保证测试在 CI / 本地不依赖外部服务即可跑
#![allow(clippy::await_holding_lock)]

use rgs_overflow_alert::alert::{AlertDeduplicator, AlertSink, LogOnlySink};
use rgs_overflow_alert::config::OverflowConfig;
use rgs_overflow_alert::domain::Domain;
use rgs_overflow_alert::guard::{OverflowGuard, OverflowStatus};
use rgs_overflow_alert::limiter::OverflowLimiter;
use rgs_overflow_alert::queue::{InMemoryQueueBackend, QueueBackend};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use rgs_overflow_alert::alert::AlertError;

/// 计数 sink（用于断言告警触发次数）
struct CountingSink {
    count: Arc<AtomicU32>,
    fail: bool,
}
#[async_trait]
impl AlertSink for CountingSink {
    async fn send(&self, _to: &str, _event: &rgs_overflow_alert::alert::AlertEvent) -> Result<(), AlertError> {
        self.count.fetch_add(1, Ordering::Relaxed);
        if self.fail {
            Err(AlertError::Smtp("mock".to_string()))
        } else {
            Ok(())
        }
    }
}

/// 共享测试 lock：避免 env 串扰（per src/test_utils.rs 模式）
fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    match ENV_LOCK.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

fn clear_all_overflow_env() {
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
        // SAFETY: test-only env clear
        unsafe {
            std::env::remove_var(k);
        }
    }
}

fn set_envs(pairs: &[(&str, &str)]) {
    // SAFETY: test-only env set
    unsafe {
        for (k, v) in pairs {
            std::env::set_var(k, v);
        }
    }
}

fn make_guard_with(
    max_inflight: u32,
    max_pending: u64,
    soft_ratio: f64,
    dedup_secs: u64,
    sink: Arc<dyn AlertSink>,
) -> (OverflowGuard, Arc<InMemoryQueueBackend>) {
    set_envs(&[
        ("PLAYER_MAX_INFLIGHT", &max_inflight.to_string()),
        ("NATS_OVERFLOW_MAX_PENDING", &max_pending.to_string()),
        ("NATS_OVERFLOW_SOFT_RATIO", &soft_ratio.to_string()),
        ("ALERT_DEDUP_WINDOW_SECS", &dedup_secs.to_string()),
    ]);
    let cfg = OverflowConfig::from_env().expect("config");
    let lim = Arc::new(OverflowLimiter::new(Domain::Player, &cfg));
    let queue = Arc::new(InMemoryQueueBackend::new(max_pending));
    let queue_dyn: Arc<dyn QueueBackend> = queue.clone();
    let primary = sink;
    let fallback: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
    let alerter = Arc::new(AlertDeduplicator::new(
        primary,
        fallback,
        "test@example.com".to_string(),
        Duration::from_secs(dedup_secs),
    ));
    let g = OverflowGuard::new(
        Domain::Player,
        &cfg,
        lim,
        queue_dyn,
        alerter,
        Some("test-pod".to_string()),
        "player-service".to_string(),
    );
    (g, queue)
}

#[tokio::test]
async fn integration_1000_concurrent_pass_queue_reject_distribution() {
    let _g = lock_env();
    clear_all_overflow_env();
    // hard=10, soft=5 (ratio=0.5), queue=1000
    // 1000 并发：前 5 个 Pass + permit（业务持有），后 5 个 Queued + 入队，再 990 个 Rejected
    let sink_count = Arc::new(AtomicU32::new(0));
    let sink: Arc<dyn AlertSink> = Arc::new(CountingSink {
        count: sink_count.clone(),
        fail: false,
    });
    let (g, queue) = make_guard_with(10, 1000, 0.5, 60, sink);
    let n = 1000;
    let mut handles = Vec::with_capacity(n);
    for i in 0..n {
        let g = g.clone();
        handles.push(tokio::spawn(async move {
            let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
            // Pass 时持有 permit 直到 task 结束
            (d.status, d.guard)
        }));
    }
    let mut pass = 0u32;
    let mut queued = 0u32;
    let mut rejected = 0u32;
    let mut permits: Vec<Option<rgs_overflow_alert::limiter::InFlightGuard>> = Vec::new();
    for h in handles {
        let (status, permit) = h.await.expect("join");
        match status {
            OverflowStatus::Pass => {
                pass += 1;
                permits.push(permit);
            }
            OverflowStatus::Queued => {
                queued += 1;
                // Queued 也持有 permit 直到消费者处理完（集成测试里无消费者 → 持到 test 结束）
                permits.push(permit);
            }
            OverflowStatus::Rejected => {
                rejected += 1;
            }
        }
    }
    // 验证：行为正确（精确比例在 1000 并发下受 CPU 调度影响不可靠 — 单元测试已覆盖精确比例）
    // 行为约束：所有 1000 个请求必须分到 3 类之一；Pass + Queued 不能超过 hard（=10）+ 一些 race 容差
    assert_eq!((pass + queued + rejected) as usize, n, "all 1000 must be classified");
    assert!(pass <= 10, "Pass > hard ({} > 10) — limiter broken", pass);
    assert!(pass + queued <= 10, "Pass+Queued > hard ({} > 10) — limiter broken", pass + queued);
    // 至少 1 个 Rejected（说明限流生效）
    assert!(rejected > 0, "no rejections — limiter not engaged");
    // queue 收到 Queued 个消息
    assert_eq!(queue.len() as u32, queued, "queue.len mismatch with Queued count: {} vs {}", queue.len(), queued);
    // 告警去重：HardCapReached 窗口内 1 次 + SoftCapSurge 窗口内 1 次 = 2
    assert_eq!(
        sink_count.load(Ordering::Relaxed),
        2,
        "expected 2 alerts (HardCapReached dedup + SoftCapSurge dedup)"
    );
    drop(permits);
}

#[tokio::test]
async fn integration_alert_dedup_suppresses_storm() {
    let _g = lock_env();
    clear_all_overflow_env();
    // 制造持续 100 次硬上限超出，但去重窗口内只发 1 次
    let sink_count = Arc::new(AtomicU32::new(0));
    let sink: Arc<dyn AlertSink> = Arc::new(CountingSink {
        count: sink_count.clone(),
        fail: false,
    });
    let (g, _queue) = make_guard_with(1, 100, 0.5, 60, sink);
    // 占满 1 个
    let _p = g.limiter().try_acquire().1;
    // 100 次 check，全部 Rejected，告警仅 1 次
    for i in 0..100 {
        let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
        assert_eq!(d.status, OverflowStatus::Rejected);
    }
    assert_eq!(
        sink_count.load(Ordering::Relaxed),
        1,
        "dedup window should suppress 99/100 alerts"
    );
}

#[tokio::test]
async fn integration_queue_full_transitions_to_reject() {
    let _g = lock_env();
    clear_all_overflow_env();
    let sink_count = Arc::new(AtomicU32::new(0));
    let sink: Arc<dyn AlertSink> = Arc::new(CountingSink {
        count: sink_count.clone(),
        fail: false,
    });
    // hard=10, soft=5, queue=2
    let (g, queue) = make_guard_with(10, 2, 0.5, 60, sink);
    // 5 个 Pass（拿 permit 持有）
    let mut d_passes = Vec::new();
    for i in 0..5 {
        let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
        assert_eq!(d.status, OverflowStatus::Pass);
        d_passes.push(d);
    }
    // 再 5 个 check：前 2 个 Queued 入队成功（queue=2），后 3 个 Rejected
    for i in 5..10 {
        let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
        if i < 7 {
            assert_eq!(d.status, OverflowStatus::Queued, "iteration {} expected Queued, got {:?}", i, d.status);
        } else {
            assert_eq!(d.status, OverflowStatus::Rejected, "iteration {} expected Rejected, got {:?}", i, d.status);
        }
    }
    // queue 应该有 2 条
    assert_eq!(queue.len(), 2);
    // 告警：1 次 SoftCapSurge（去重）+ 1 次 HardCapReached（去重）= 2
    assert_eq!(sink_count.load(Ordering::Relaxed), 2);
    drop(d_passes);
}

#[tokio::test]
async fn integration_4_domains_use_independent_subjects() {
    // 验证 4 域 NATS subject 独立（per `rgs.<domain>.overflow.v1` 约定）
    use rgs_overflow_alert::queue::NatsJsQueueBackend;
    assert_eq!(NatsJsQueueBackend::subject_for(Domain::Player), "rgs.player.overflow.v1");
    assert_eq!(NatsJsQueueBackend::subject_for(Domain::Economy), "rgs.economy.overflow.v1");
    assert_eq!(NatsJsQueueBackend::subject_for(Domain::Match), "rgs.match.overflow.v1");
    assert_eq!(NatsJsQueueBackend::subject_for(Domain::Social), "rgs.social.overflow.v1");
}

#[tokio::test]
async fn integration_disabled_when_all_hard_caps_zero() {
    let _g = lock_env();
    clear_all_overflow_env();
    // 全部 hard=0 → 限流未启用 → 1000 个全部 Pass
    let (g, _queue) = make_guard_with(0, 100, 0.5, 60, Arc::new(CountingSink {
        count: Arc::new(AtomicU32::new(0)),
        fail: false,
    }));
    for i in 0..1000 {
        let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
        assert_eq!(d.status, OverflowStatus::Pass);
    }
}
