//! 业务层最常用的高层 API：`OverflowGuard`
//!
//! 一个 guard = `OverflowLimiter`（Arc 共享）+ `QueueBackend`（dyn 注入）+ `AlertDeduplicator`
//!
//! 业务调用：
//! ```ignore
//! let decision = guard.check(domain, &payload).await;
//! match decision.status {
//!     OverflowStatus::Pass => { /* 正常处理 */ }
//!     OverflowStatus::Queued => { /* 业务可选择：等 permit / 直接返回 ResourceExhausted */ }
//!     OverflowStatus::Rejected => { /* 拒绝 + 告警已触发 */ }
//! }
//! ```
//!
//! **不**在此处决定"Queued 时是否同步等 permit" — 留给 4 域业务 service 自适配 RPC 风格
//! （unary 直接 ResourceExhausted；streaming 可 await permit）。

use crate::alert::{AlertDeduplicator, AlertEvent, AlertKind};
use crate::config::{OverflowConfig, SmtpConfig};
use crate::domain::Domain;
use crate::limiter::{AcquireOutcome, OverflowLimiter};
use crate::queue::{AckToken, OverflowPayload, QueueBackend};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// guard 内部错误（**不**给业务 — 业务只关心 OverflowStatus）
#[derive(Debug, Error)]
pub enum GuardError {
    #[error("queue backend error: {0}")]
    Queue(String),
    #[error("queue full")]
    QueueFull,
}

/// 业务层最终决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowStatus {
    /// 在软阈值内放行
    Pass,
    /// 软阈值已超：入队成功（业务可选择同步等 permit / 直接 ResourceExhausted）
    Queued,
    /// 硬上限已满：拒绝 + 告警
    Rejected,
}

impl OverflowStatus {
    pub fn is_pass(self) -> bool {
        matches!(self, OverflowStatus::Pass)
    }
    pub fn is_queued(self) -> bool {
        matches!(self, OverflowStatus::Queued)
    }
    pub fn is_rejected(self) -> bool {
        matches!(self, OverflowStatus::Rejected)
    }
}

/// check 结果
#[derive(Debug)]
pub struct OverflowDecision {
    pub status: OverflowStatus,
    /// 仅 `Queued` 时有值
    pub ack_token: Option<AckToken>,
    /// `Pass` / `Queued` 时有值：业务方持有至请求处理完成 drop
    ///
    /// - `Pass`：业务处理完请求时 drop（释放 in_flight）
    /// - `Queued`：入队后立即 drop 或由消费者在完成时 drop（释放 in_flight 槽位给后续请求）
    /// - `Rejected`：无 guard
    pub guard: Option<crate::limiter::InFlightGuard>,
}

/// 高层 guard
#[derive(Clone)]
pub struct OverflowGuard {
    domain: Domain,
    limiter: Arc<OverflowLimiter>,
    queue: Arc<dyn QueueBackend>,
    alerter: Arc<AlertDeduplicator>,
    pod: String,
    service: String,
    /// 软阈值首超时间（用于告警正文的 first_at）
    first_soft_surge_at: Arc<std::sync::Mutex<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl OverflowGuard {
    /// 构造（**业务服务 main.rs 一次**）
    ///
    /// `service` = 业务服务名（如 `player-service`）；`pod` 从 `POD_NAME` env 读，
    /// fallback 到 `hostname` crate 探测
    ///
    /// `_cfg` 保留参数位置以备后续 pod-level 覆盖（暂未使用）
    pub fn new(
        domain: Domain,
        _cfg: &OverflowConfig,
        limiter: Arc<OverflowLimiter>,
        queue: Arc<dyn QueueBackend>,
        alerter: Arc<AlertDeduplicator>,
        pod: Option<String>,
        service: String,
    ) -> Self {
        let pod = pod
            .or_else(|| std::env::var("POD_NAME").ok())
            .unwrap_or_else(|| {
                hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "unknown".to_string())
            });
        Self {
            domain,
            limiter,
            queue,
            alerter,
            pod,
            service,
            first_soft_surge_at: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn domain(&self) -> Domain {
        self.domain
    }
    pub fn pod(&self) -> &str {
        &self.pod
    }
    pub fn service(&self) -> &str {
        &self.service
    }
    pub fn limiter(&self) -> &Arc<OverflowLimiter> {
        &self.limiter
    }

    /// 业务主调用入口
    ///
    /// 流程：
    /// 1. limiter.try_acquire → Pass / Queued / Rejected
    /// 2. Pass → 业务直接处理
    /// 3. Queued → queue.enqueue，成功 → OverflowStatus::Queued，失败 → Rejected + 告警
    /// 4. Rejected → 立即告警
    #[allow(clippy::too_many_lines)]
    pub async fn check(&self, op: &str, request_id: &str, business_json: Option<&str>) -> OverflowDecision {
        // 1. 限流
        let (outcome, permit) = self.limiter.try_acquire();
        match outcome {
            AcquireOutcome::Pass => OverflowDecision {
                status: OverflowStatus::Pass,
                ack_token: None,
                guard: permit,
            },
            AcquireOutcome::Queued => {
                // Queued: permit 保留到消费者处理完才 drop（in_flight 槽位让消费者在处理期间占用，
                // 防止"无限涌入 → 永远不 Rejected"的语义漏洞）
                // 业务侧：拿到 decision 后立即返回 ResourceExhausted 给 client；
                // 消费者 task 拿到 ack_token 后持 guard，process 完 drop
                // 这里 permit 转移到 decision.guard
                // 2. 入队
                let now = chrono::Utc::now();
                let first_at = {
                    let mut g = self
                        .first_soft_surge_at
                        .lock()
                        .expect("first_soft_surge mutex");
                    *g.get_or_insert(now)
                };
                let payload = OverflowPayload {
                    op: op.to_string(),
                    request_id: request_id.to_string(),
                    domain: self.domain.as_str().to_string(),
                    in_flight: self.limiter.in_flight(),
                    hard_cap: self.limiter.hard_cap(),
                    soft_cap: self.limiter.soft_cap(),
                    pod: self.pod.clone(),
                    service: self.service.clone(),
                    business_json: business_json.map(|s| s.to_string()),
                    first_at: first_at.to_rfc3339(),
                    last_at: now.to_rfc3339(),
                    reject_count_5min: self.limiter.reject_count_5min(),
                };
                match self.queue.enqueue(self.domain, &payload).await {
                    Ok(ack) => {
                        // 软阈值首次超：触发告警（一次性）
                        self.alerter
                            .notify(&AlertEvent {
                                kind: AlertKind::SoftCapSurge,
                                domain: self.domain.as_str().to_string(),
                                in_flight: self.limiter.in_flight(),
                                hard_cap: self.limiter.hard_cap(),
                                soft_cap: self.limiter.soft_cap(),
                                queue_pending: 0, // 后续可从 queue 取
                                pod: self.pod.clone(),
                                service: self.service.clone(),
                                reject_count_5min: self.limiter.reject_count_5min(),
                                first_at: first_at.to_rfc3339(),
                                last_at: now.to_rfc3339(),
                            })
                            .await;
                        OverflowDecision {
                            status: OverflowStatus::Queued,
                            ack_token: Some(ack),
                            // permit 转交给 decision：消费者在处理完消息后 drop(decision.guard) 释放 in_flight
                            guard: permit,
                        }
                    }
                    Err(crate::queue::QueueError::QueueFull(_)) => {
                        // 队列满：等效于拒绝
                        self.fire_rejected_alert().await;
                        OverflowDecision {
                            status: OverflowStatus::Rejected,
                            ack_token: None,
                            guard: None,
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            target: "rgs-overflow-alert",
                            domain = %self.domain,
                            op = %op,
                            error = %e,
                            "queue enqueue failed, treating as Rejected"
                        );
                        self.fire_rejected_alert().await;
                        OverflowDecision {
                            status: OverflowStatus::Rejected,
                            ack_token: None,
                            guard: None,
                        }
                    }
                }
            }
            AcquireOutcome::Rejected => {
                // Rejected: permit 是 None（avail=0 时 try_acquire_owned 返回 None）
                self.fire_rejected_alert().await;
                OverflowDecision {
                    status: OverflowStatus::Rejected,
                    ack_token: None,
                    guard: permit,
                }
            }
        }
    }

    async fn fire_rejected_alert(&self) {
        let now = chrono::Utc::now();
        let event = AlertEvent {
            kind: AlertKind::HardCapReached,
            domain: self.domain.as_str().to_string(),
            in_flight: self.limiter.in_flight(),
            hard_cap: self.limiter.hard_cap(),
            soft_cap: self.limiter.soft_cap(),
            queue_pending: 0,
            pod: self.pod.clone(),
            service: self.service.clone(),
            reject_count_5min: self.limiter.reject_count_5min(),
            first_at: now.to_rfc3339(),
            last_at: now.to_rfc3339(),
        };
        self.alerter.notify(&event).await;
    }

    /// 用配置构造"标准三件套"：SMTP / LogOnly + dedup（**业务**用，省 boilerplate）
    pub fn build_standard_sink(
        cfg: &OverflowConfig,
    ) -> (Arc<dyn crate::alert::AlertSink>, Arc<dyn crate::alert::AlertSink>) {
        let primary: Arc<dyn crate::alert::AlertSink> = if cfg.smtp.password_is_empty() {
            Arc::new(crate::alert::LogOnlySink)
        } else {
            match crate::alert::SmtpAlertSink::new(&cfg.smtp) {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    tracing::warn!(
                        target: "rgs-overflow-alert",
                        error = %e,
                        "SmtpAlertSink construction failed, falling back to LogOnlySink"
                    );
                    Arc::new(crate::alert::LogOnlySink)
                }
            }
        };
        let fallback: Arc<dyn crate::alert::AlertSink> = Arc::new(crate::alert::LogOnlySink);
        (primary, fallback)
    }

    /// 构造 dedup（辅助函数）
    pub fn build_alerter(
        cfg: &OverflowConfig,
    ) -> Arc<AlertDeduplicator> {
        let (primary, fallback) = Self::build_standard_sink(cfg);
        Arc::new(AlertDeduplicator::new(
            primary,
            fallback,
            cfg.support_email.clone(),
            cfg.dedup_window,
        ))
    }

    /// 抑制 unused 警告（保持 `SmtpConfig` re-export 表面）
    #[doc(hidden)]
    pub fn _phantom_smtp(_: &SmtpConfig, _: Duration) {}
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)]
mod tests {
    use super::*;
    use crate::alert::{AlertDeduplicator, LogOnlySink};
    use crate::config::OverflowConfig;
    use crate::limiter::OverflowLimiter;
    use crate::queue::InMemoryQueueBackend;
    use crate::test_utils::{clear_all_overflow_env, lock_env, set_envs};

    fn make_guard(max_inflight: u32, max_pending: u64, soft_ratio: f64) -> OverflowGuard {
        // 注意：make_guard **不** clear_all_overflow_env，caller 负责整个 env 设置
        // 这样 caller 可以预设 soft_ratio 等其他 env 后再调 make_guard
        set_envs(&[
            ("PLAYER_MAX_INFLIGHT", &max_inflight.to_string()),
            ("NATS_OVERFLOW_MAX_PENDING", &max_pending.to_string()),
            ("NATS_OVERFLOW_SOFT_RATIO", &soft_ratio.to_string()),
        ]);
        let cfg = OverflowConfig::from_env().unwrap();
        let lim = Arc::new(OverflowLimiter::new(Domain::Player, &cfg));
        let queue: Arc<dyn QueueBackend> = Arc::new(InMemoryQueueBackend::new(max_pending));
        let primary: Arc<dyn crate::alert::AlertSink> = Arc::new(LogOnlySink);
        let fallback: Arc<dyn crate::alert::AlertSink> = Arc::new(LogOnlySink);
        let alerter = Arc::new(AlertDeduplicator::new(
            primary,
            fallback,
            "test@example.com".to_string(),
            Duration::from_secs(60),
        ));
        OverflowGuard::new(
            Domain::Player,
            &cfg,
            lim,
            queue,
            alerter,
            Some("test-pod".to_string()),
            "player-service".to_string(),
        )
    }

    #[tokio::test]
    async fn pass_below_soft() {
        let _g = lock_env();
        clear_all_overflow_env();
        let g = make_guard(10, 100, 0.8);
        let d = g.check("Test::Op", "req-1", None).await;
        assert_eq!(d.status, OverflowStatus::Pass);
    }

    #[tokio::test]
    async fn queue_above_soft_below_hard() {
        let _g = lock_env();
        clear_all_overflow_env();
        let g = make_guard(10, 100, 0.5);
        // 用 check() 取 5 个 permit（Pass 路径返回 permit，business 持有）
        let mut d_passes = Vec::new();
        for i in 0..5 {
            let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
            assert_eq!(d.status, OverflowStatus::Pass);
            assert!(d.guard.is_some());
            d_passes.push(d);
        }
        // 第 6 个：> soft 但 < hard → Queued
        let d = g.check("Test::Op", "req-q", None).await;
        assert_eq!(d.status, OverflowStatus::Queued);
        assert!(d.ack_token.is_some());
        // Queued 也持有 permit（直到消费者处理完才 drop）— 这是限流语义的关键
        assert!(d.guard.is_some(), "Queued should retain permit for in_flight accounting");
        // 持 5 个 permit + 第 6 个的 permit
        d_passes.push(d);
        drop(d_passes);
    }

    #[tokio::test]
    async fn reject_above_hard() {
        let _g = lock_env();
        clear_all_overflow_env();
        // hard=2, soft_ratio=1.0 → soft=2：前 2 个 Pass + 持有 permit，第 3 个 Rejected
        let g = make_guard(2, 100, 1.0);
        let d1 = g.check("Op::1", "req-1", None).await; // Pass + permit
        let d2 = g.check("Op::2", "req-2", None).await; // Pass + permit
        assert_eq!(d1.status, OverflowStatus::Pass);
        assert_eq!(d2.status, OverflowStatus::Pass);
        assert!(d1.guard.is_some());
        assert!(d2.guard.is_some());
        // 第 3 个：硬上限已满 → Rejected
        let d = g.check("Test::Op", "req-r", None).await;
        assert_eq!(d.status, OverflowStatus::Rejected);
        assert!(d.ack_token.is_none());
        drop((d1, d2));
    }

    #[tokio::test]
    async fn queue_full_falls_back_to_reject() {
        let _g = lock_env();
        clear_all_overflow_env();
        // hard=10, soft=5, queue=1 → 第 2 个 Queued 之后会变 Rejected
        let g = make_guard(10, 1, 0.5);
        // 用 check() 取 5 个 permit
        let mut d_passes = Vec::new();
        for i in 0..5 {
            let d = g.check(&format!("Op::{}", i), &format!("req-{}", i), None).await;
            assert_eq!(d.status, OverflowStatus::Pass);
            d_passes.push(d);
        }
        // 第 6 个：Queued → 入队成功（queue=1）
        let d1 = g.check("Test::Op", "req-q1", None).await;
        assert_eq!(d1.status, OverflowStatus::Queued);
        // 第 7 个：Queued → 入队失败（queue=1 已满）→ Rejected
        let d2 = g.check("Test::Op", "req-q2", None).await;
        assert_eq!(d2.status, OverflowStatus::Rejected);
        drop(d_passes);
    }
}
