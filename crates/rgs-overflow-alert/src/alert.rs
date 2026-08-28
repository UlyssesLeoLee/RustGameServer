//! 告警层 — SMTP 发送 + 日志 fallback + 窗口去重
//!
//! 关键点（per 任务 §1）：
//! - **trait abstraction**：`AlertSink` — 业务可注入 mock 测试
//! - **两个实现**：`SmtpAlertSink`（lettre，密码非空）+ `LogOnlySink`（密码空 / SMTP 失败降级，**不抛错**）
//! - **去重**：`AlertDeduplicator` 用 `(domain, kind)` key + `ALERT_DEDUP_WINDOW_SECS` 窗口
//!   内同 key 只发 1 次
//! - 邮件主题：`[RGS-ALERT] <domain> overflow @ <RFC3339>`
//! - 正文：domain / in-flight / 硬上限 / 软上限 / queue pending / Pod / service / 5min reject / 首次-末次时间

use crate::config::SmtpConfig;
use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error as SmtpError;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;

/// 告警错误（**仅**给上层观测；`SmtpAlertSink::send` 出错时由 `AlertDeduplicator`
/// 内部 fallback 到 `LogOnlySink`，**不**上抛 — per 任务 §1 "缺密码时告警只落
/// `tracing::warn!`，不抛错，不阻断入队"）
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("SMTP send error: {0}")]
    Smtp(String),
    #[error("invalid email: {0}")]
    InvalidEmail(String),
    #[error("message build error: {0}")]
    Build(String),
}

/// 告警类型（用于去重 key 区分 + 邮件正文）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertKind {
    /// 硬上限已满：拒绝 + 告警
    HardCapReached,
    /// 软阈值已超：入队（仅在大量涌入时告警，避免噪音）
    SoftCapSurge,
    /// 队列满
    QueueFull,
    /// 邮件 / SMTP 自身故障
    SinkFailure,
}

/// 告警事件
#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub kind: AlertKind,
    pub domain: String,
    pub in_flight: u32,
    pub hard_cap: u32,
    pub soft_cap: u32,
    pub queue_pending: u64,
    pub pod: String,
    pub service: String,
    pub reject_count_5min: u64,
    pub first_at: String,
    pub last_at: String,
}

impl AlertEvent {
    /// 邮件主题
    pub fn subject(&self) -> String {
        format!(
            "[RGS-ALERT] {} overflow @ {}",
            self.domain,
            chrono::Utc::now().to_rfc3339()
        )
    }

    /// 邮件正文（纯文本 / RFC822）
    pub fn body(&self) -> String {
        format!(
            "RGS 超限告警\n\n\
             domain:           {domain}\n\
             kind:             {kind:?}\n\
             in_flight:        {in_flight} / {hard_cap} (soft={soft_cap})\n\
             queue_pending:    {queue_pending}\n\
             pod:              {pod}\n\
             service:          {service}\n\
             reject_count_5m:  {reject_count_5min}\n\
             first_at:         {first_at}\n\
             last_at:          {last_at}\n",
            domain = self.domain,
            kind = self.kind,
            in_flight = self.in_flight,
            hard_cap = self.hard_cap,
            soft_cap = self.soft_cap,
            queue_pending = self.queue_pending,
            pod = self.pod,
            service = self.service,
            reject_count_5min = self.reject_count_5min,
            first_at = self.first_at,
            last_at = self.last_at,
        )
    }
}

/// 告警 sink trait
#[async_trait]
pub trait AlertSink: Send + Sync {
    /// 发送告警（实现内部应当不抛错；失败仅 logging）
    async fn send(&self, to: &str, event: &AlertEvent) -> Result<(), AlertError>;
}

/// SMTP 邮件 sink
pub struct SmtpAlertSink {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_addr: String,
    from_name: String,
    timeout: Duration,
}

impl SmtpAlertSink {
    /// 构造 SMTP sink（**不**做密码空检查 — 由调用方 `OverflowGuard::new` 决定用哪个 sink）
    pub fn new(cfg: &SmtpConfig) -> Result<Self, AlertError> {
        if cfg.host.trim().is_empty() {
            return Err(AlertError::InvalidEmail("SMTP_HOST empty".to_string()));
        }
        if cfg.user.trim().is_empty() {
            return Err(AlertError::InvalidEmail("SMTP_USER empty".to_string()));
        }
        let creds = Credentials::new(cfg.user.clone(), cfg.password.clone());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)
            .map_err(|e: SmtpError| AlertError::Smtp(e.to_string()))?
            .port(cfg.port)
            .credentials(creds)
            .timeout(Some(cfg.timeout))
            .build();
        let from_addr = cfg.user.clone();
        let from_name = cfg.from_name.clone();
        Ok(Self {
            transport,
            from_addr,
            from_name,
            timeout: cfg.timeout,
        })
    }

    /// 构造 from 头部（`"Name" <addr>`）
    fn build_from_header(&self) -> String {
        format!("\"{}\" <{}>", self.from_name, self.from_addr)
    }
}

#[async_trait]
impl AlertSink for SmtpAlertSink {
    async fn send(&self, to: &str, event: &AlertEvent) -> Result<(), AlertError> {
        let from = self.build_from_header();
        let subject = event.subject();
        let body = event.body();
        let email = Message::builder()
            .from(from.parse().map_err(|e: lettre::address::AddressError| {
                AlertError::Build(e.to_string())
            })?)
            .to(to.parse().map_err(|e: lettre::address::AddressError| {
                AlertError::Build(e.to_string())
            })?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| AlertError::Build(e.to_string()))?;
        let send_fut = self.transport.send(email);
        match tokio::time::timeout(self.timeout, send_fut).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(AlertError::Smtp(e.to_string())),
            Err(_elapsed) => Err(AlertError::Smtp(format!(
                "SMTP timeout after {:?}",
                self.timeout
            ))),
        }
    }
}

/// 日志 fallback sink（**永远不抛错**）
pub struct LogOnlySink;

#[async_trait]
impl AlertSink for LogOnlySink {
    async fn send(&self, to: &str, event: &AlertEvent) -> Result<(), AlertError> {
        tracing::warn!(
            target: "rgs-overflow-alert",
            to = %to,
            domain = %event.domain,
            kind = ?event.kind,
            in_flight = event.in_flight,
            hard_cap = event.hard_cap,
            soft_cap = event.soft_cap,
            queue_pending = event.queue_pending,
            pod = %event.pod,
            service = %event.service,
            reject_count_5min = event.reject_count_5min,
            first_at = %event.first_at,
            last_at = %event.last_at,
            "ALERT (LogOnlySink — SMTP password empty or transport unavailable)"
        );
        Ok(())
    }
}

/// 告警去重器
///
/// 同 `(domain, kind)` 在 `dedup_window` 内只发 1 次
/// 内部用 `HashMap<(String, AlertKind), Instant>` 记录"上次发送时间"
pub struct AlertDeduplicator {
    /// 实际 sink（注入）
    inner: Arc<dyn AlertSink>,
    /// fallback sink（**永远 LogOnlySink**；SMTP 失败时切到 fallback）
    fallback: Arc<dyn AlertSink>,
    /// 收件人
    to: String,
    /// 去重窗口
    window: Duration,
    /// key → 上次发送时间
    state: Arc<Mutex<HashMap<(String, AlertKind), Instant>>>,
}

impl AlertDeduplicator {
    /// 构造：`inner` = 实际 sink（失败时 fallback）；`fallback` 通常 = `LogOnlySink`
    pub fn new(
        inner: Arc<dyn AlertSink>,
        fallback: Arc<dyn AlertSink>,
        to: String,
        window: Duration,
    ) -> Self {
        Self {
            inner,
            fallback,
            to,
            window,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 发送告警（窗口内同 key 跳过；SMTP 失败 → fallback；**永不抛错**）
    pub async fn notify(&self, event: &AlertEvent) {
        let key = (event.domain.clone(), event.kind);
        let now = Instant::now();
        let mut g = self.state.lock().await;
        if let Some(&last) = g.get(&key) {
            if now.duration_since(last) < self.window {
                // 窗口内：跳过（**不**记 last，避免拖长窗口）
                return;
            }
        }
        g.insert(key, now);
        // 释放锁后再 await（避免 sink 慢时锁住 state）
        drop(g);
        match self.inner.send(&self.to, event).await {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!(
                    target: "rgs-overflow-alert",
                    sink = "SmtpAlertSink",
                    error = %e,
                    "primary sink failed, falling back to LogOnlySink"
                );
                let _ = self.fallback.send(&self.to, event).await;
            }
        }
    }

    /// 手动 flush（测试用 — 清空去重窗口）
    pub async fn reset(&self) {
        self.state.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 计数 mock sink（断言是否发了几次）
    struct CountingSink {
        count: Arc<AtomicU32>,
        fail: bool,
    }
    #[async_trait]
    impl AlertSink for CountingSink {
        async fn send(&self, _to: &str, _event: &AlertEvent) -> Result<(), AlertError> {
            self.count.fetch_add(1, Ordering::Relaxed);
            if self.fail {
                Err(AlertError::Smtp("mock failure".to_string()))
            } else {
                Ok(())
            }
        }
    }

    fn sample_event(kind: AlertKind) -> AlertEvent {
        AlertEvent {
            kind,
            domain: "player".to_string(),
            in_flight: 10,
            hard_cap: 10,
            soft_cap: 8,
            queue_pending: 5,
            pod: "pod-1".to_string(),
            service: "player-service".to_string(),
            reject_count_5min: 1,
            first_at: "2026-08-27T09:00:00Z".to_string(),
            last_at: "2026-08-27T09:00:01Z".to_string(),
        }
    }

    #[tokio::test]
    async fn dedup_window_suppresses_repeats() {
        let count = Arc::new(AtomicU32::new(0));
        let sink: Arc<dyn AlertSink> = Arc::new(CountingSink {
            count: count.clone(),
            fail: false,
        });
        let fb: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
        let d = AlertDeduplicator::new(
            sink,
            fb,
            "test@example.com".to_string(),
            Duration::from_secs(60),
        );
        for _ in 0..10 {
            d.notify(&sample_event(AlertKind::HardCapReached)).await;
        }
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn sink_failure_falls_back() {
        let primary_count = Arc::new(AtomicU32::new(0));
        let primary: Arc<dyn AlertSink> = Arc::new(CountingSink {
            count: primary_count.clone(),
            fail: true,
        });
        let fb_count = Arc::new(AtomicU32::new(0));
        let fb: Arc<dyn AlertSink> = Arc::new(CountingSink {
            count: fb_count.clone(),
            fail: false,
        });
        let d = AlertDeduplicator::new(
            primary,
            fb,
            "test@example.com".to_string(),
            Duration::from_secs(60),
        );
        d.notify(&sample_event(AlertKind::QueueFull)).await;
        assert_eq!(primary_count.load(Ordering::Relaxed), 1);
        assert_eq!(fb_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn different_kinds_send_independently() {
        let count = Arc::new(AtomicU32::new(0));
        let sink: Arc<dyn AlertSink> = Arc::new(CountingSink {
            count: count.clone(),
            fail: false,
        });
        let fb: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
        let d = AlertDeduplicator::new(
            sink,
            fb,
            "test@example.com".to_string(),
            Duration::from_secs(60),
        );
        d.notify(&sample_event(AlertKind::HardCapReached)).await;
        d.notify(&sample_event(AlertKind::QueueFull)).await;
        d.notify(&sample_event(AlertKind::SoftCapSurge)).await;
        assert_eq!(count.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn log_only_sink_never_fails() {
        let sink = LogOnlySink;
        let r = sink.send("x@example.com", &sample_event(AlertKind::HardCapReached)).await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn dedup_key_isolates_domains() {
        let count = Arc::new(AtomicU32::new(0));
        let sink: Arc<dyn AlertSink> = Arc::new(CountingSink {
            count: count.clone(),
            fail: false,
        });
        let fb: Arc<dyn AlertSink> = Arc::new(LogOnlySink);
        let d = AlertDeduplicator::new(
            sink,
            fb,
            "test@example.com".to_string(),
            Duration::from_secs(60),
        );
        let mut e1 = sample_event(AlertKind::HardCapReached);
        e1.domain = "player".to_string();
        let mut e2 = sample_event(AlertKind::HardCapReached);
        e2.domain = "economy".to_string();
        d.notify(&e1).await;
        d.notify(&e2).await;
        assert_eq!(count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn subject_format() {
        let e = sample_event(AlertKind::HardCapReached);
        let s = e.subject();
        assert!(s.starts_with("[RGS-ALERT] player overflow @ "));
    }

    #[test]
    fn body_contains_required_fields() {
        let e = sample_event(AlertKind::HardCapReached);
        let b = e.body();
        for f in &[
            "domain:",
            "kind:",
            "in_flight:",
            "queue_pending:",
            "pod:",
            "service:",
            "reject_count_5m:",
            "first_at:",
            "last_at:",
        ] {
            assert!(b.contains(f), "body missing field: {}", f);
        }
    }
}
