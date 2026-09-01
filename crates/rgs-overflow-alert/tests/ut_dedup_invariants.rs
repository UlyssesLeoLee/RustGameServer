//! rgs-overflow-alert proptest 块 (per 9/1 PT-WORKER 派工)
//!
//! Idempotent invariant：同 (domain, kind) 窗口内发 N 次, sink 收到 ≤ 1 次
//!
//! 测试策略：每个 proptest case 拿 1 把 ENV_LOCK，构造 AlertDeduplicator
//! (CountingSink 计数) + 同一 key notify N 次 → 计数 = 1

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use proptest::prelude::*;
use rgs_overflow_alert::alert::{
    AlertDeduplicator, AlertError, AlertEvent, AlertKind, AlertSink, LogOnlySink,
};

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

/// ENV 锁 (per src/test_utils.rs 模式)
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    }
}

fn sample_event(domain: &str, kind: AlertKind) -> AlertEvent {
    AlertEvent {
        kind,
        domain: domain.to_string(),
        in_flight: 10,
        hard_cap: 10,
        soft_cap: 8,
        queue_pending: 0,
        pod: "test-pod".to_string(),
        service: "test-svc".to_string(),
        reject_count_5min: 1,
        first_at: "2026-09-01T00:00:00Z".to_string(),
        last_at: "2026-09-01T00:00:01Z".to_string(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Invariant: 同 (domain, kind) 窗口内 N 次 notify, 计数必须 == 1
    #[test]
    fn dedup_within_window_is_idempotent(
        n in 2usize..32,
        kind in prop_oneof![
            Just(AlertKind::HardCapReached),
            Just(AlertKind::SoftCapSurge),
            Just(AlertKind::QueueFull),
        ]
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _g = lock_env();
            let count = Arc::new(AtomicU32::new(0));
            let sink: Arc<dyn AlertSink> = Arc::new(CountingSink { count: count.clone() });
            let fb: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
            let d = AlertDeduplicator::new(
                sink,
                fb,
                "test@example.com".to_string(),
                Duration::from_secs(60),
            );
            for _ in 0..n {
                d.notify(&sample_event("player", kind)).await;
            }
            let got = count.load(Ordering::Relaxed);
            prop_assert_eq!(got, 1, "同 key 窗口内 N 次 notify 必须只发 1 次 (n={}, kind={:?})", n, kind);
            Ok(())
        })?;
    }

    /// Invariant: 跨 domain 的同 kind, 各自独立计数
    #[test]
    fn dedup_isolated_per_domain(
        n in 1usize..5
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _g = lock_env();
            let count = Arc::new(AtomicU32::new(0));
            let sink: Arc<dyn AlertSink> = Arc::new(CountingSink { count: count.clone() });
            let fb: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
            let d = AlertDeduplicator::new(
                sink,
                fb,
                "test@example.com".to_string(),
                Duration::from_secs(60),
            );
            // n 个不同 domain, 每个 1 次 → 应发 n 次
            for i in 0..n {
                let domain = format!("d{i}");
                d.notify(&sample_event(&domain, AlertKind::HardCapReached)).await;
            }
            prop_assert_eq!(count.load(Ordering::Relaxed) as usize, n,
                "不同 domain 应独立计数");
            Ok(())
        })?;
    }
}
