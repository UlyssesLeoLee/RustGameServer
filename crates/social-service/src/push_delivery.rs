//! Social 域 PushDeliveryRequest 协议线(per RGS-DTL-019 §3)
//!
//! ## 协议字段(per DTL-019 §3 protobuf 镜像)
//! - `account_id` 收件人账号
//! - `category` 对应 push_consents.category(投递前已通过同意校验)
//! - `title` 已过 PushContentSanitizer 校验
//! - `body` 消息正文
//! - `dedup_window_id` 频率限制窗口标识(PushGatewayAdapter 侧幂等用)
//!
//! ## DeliveryResultCode(per DTL-019 §3)
//! - DELIVERED = 0
//! - DEVICE_TOKEN_EXPIRED = 1
//! - RATE_LIMITED_DROPPED = 2
//! - RATE_LIMITED_QUEUED = 3
//!
//! ## 54.x + 55.21+22 增量
//! - Q7 (per RGS-OPEN-QA-2026-08-31-test-summary v0.2 §Q7): 加 `PushDispatcher`
//!   trait + `NatsPushDispatcher` 实现(走 NATS 主题 `social.push.delivery`),
//!   retry 复用 economy outbox+saga 模式(max_attempts + backoff), 失败超 max
//!   进 DLQ 主题 `social.push.dlq` + 写 `push_dlq` 表。
//! - retry 策略: max_attempts=3, backoff=exponential(50ms, 100ms, 200ms)。
//! - 生产路径走 `async_nats::Client`（per shared-platform producer.rs 模式）,
//!   测试走 `InMemoryNatsPublisher`（本模块自带,不引 rgs-testkit 强约束）。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDeliveryRequest {
    pub account_id: String,
    pub category: String,
    pub title: String,
    pub body: String,
    pub dedup_window_id: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(i32)]
pub enum DeliveryResultCode {
    Delivered = 0,
    DeviceTokenExpired = 1,
    RateLimitedDropped = 2,
    RateLimitedQueued = 3,
}

impl DeliveryResultCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Delivered),
            1 => Some(Self::DeviceTokenExpired),
            2 => Some(Self::RateLimitedDropped),
            3 => Some(Self::RateLimitedQueued),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDeliveryResult {
    pub result_code: DeliveryResultCode,
}

/// PushContentSanitizer:校验 title/body 不含禁止模式
/// per RGS-BAS-019 §2.2 简化的占位实装
pub fn sanitize_push_content(title: &str, body: &str) -> Result<(), String> {
    const BANNED_PATTERNS: &[&str] = &["<script>", "javascript:", "data:"];
    for p in BANNED_PATTERNS {
        if title.contains(p) || body.contains(p) {
            return Err(format!("banned pattern: {}", p));
        }
    }
    Ok(())
}

// ============================================================================
// Q7 (per RGS-OPEN-QA-2026-08-31-test-summary v0.2 §Q7): PushDispatcher + NATS impl
// ============================================================================

/// NATS 主题常量（per Q7 决策）
pub const PUSH_DELIVERY_SUBJECT: &str = "social.push.delivery";
pub const PUSH_DLQ_SUBJECT: &str = "social.push.dlq";

/// Push 投递错误
#[derive(Debug, Error)]
pub enum PushDispatcherError {
    /// NATS publish 失败（网络/连接/序列化）
    #[error("publish failed: {0}")]
    Publish(String),

    /// sanitizer 拒绝（业务层校验失败, 不可重试）
    #[error("sanitizer reject: {0}")]
    Sanitizer(String),
}

pub type PushDispatcherResult<T> = std::result::Result<T, PushDispatcherError>;

/// 投递策略（per Q7 决策: 复用 economy outbox+saga retry 模式）
#[derive(Debug, Clone)]
pub struct DispatcherConfig {
    /// 最大尝试次数（含首次）, 超过则进 DLQ
    pub max_attempts: u32,
    /// 退避基数（exponential: base * 2^(attempt-1)）
    pub backoff_base: Duration,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_base: Duration::from_millis(50),
        }
    }
}

/// Push 投递状态（per Q7 决策: 终态 / retry 决策用）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispatchOutcome {
    /// 成功投递（attempts 达到 max_attempts 前至少一次成功）
    Delivered { attempts: u32 },
    /// 失败超 max_attempts, 已进 DLQ
    DeadLettered { attempts: u32, last_error: String },
    /// Sanitizer 拒绝, 直接进 DLQ, 不重试
    RejectedBySanitizer { reason: String },
}

/// DLQ 表 entry（per Q7 决策: 新表 `push_dlq`）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDlqEntry {
    pub id: Uuid,
    pub req: PushDeliveryRequest,
    pub attempts: u32,
    pub last_error: String,
    pub created_at: DateTime<Utc>,
}

impl PushDlqEntry {
    pub fn new(req: PushDeliveryRequest, attempts: u32, last_error: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            req,
            attempts,
            last_error,
            created_at: Utc::now(),
        }
    }
}

/// NATS client 抽象（per Q7 决策: 生产用 async-nats, 测试用 InMemory）
#[async_trait]
pub trait PushNatsPublisher: Send + Sync {
    /// 投递 payload 到 subject
    async fn publish(&self, subject: &str, payload: &[u8]) -> PushDispatcherResult<()>;
}

/// In-memory NATS publisher (per Q7 决策: 测试 fixture + 不引 async-nats 重依赖)
pub struct InMemoryNatsPublisher {
    /// subject -> 累积的 payload
    store: Mutex<HashMap<String, Vec<Vec<u8>>>>,
    /// mock 失败模式: 首次 publish 失败的 subject 集合（仅第 1 次失败, 后续成功）
    fail_first_publish: Mutex<HashMap<String, ()>>,
    /// mock 失败模式: 总是失败的 subject 集合
    always_fail: Mutex<HashMap<String, ()>>,
}

impl InMemoryNatsPublisher {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            fail_first_publish: Mutex::new(HashMap::new()),
            always_fail: Mutex::new(HashMap::new()),
        }
    }

    /// mock: 标记某 subject 第 1 次 publish 必失败（用于测 retry 后成功路径）
    pub fn fail_first_publish(&self, subject: &str) {
        self.fail_first_publish
            .lock()
            .expect("InMemoryNatsPublisher mutex poisoned")
            .insert(subject.to_string(), ());
    }

    /// mock: 标记某 subject 总是失败（用于测 retry 耗尽 → DLQ 路径）
    pub fn always_fail(&self, subject: &str) {
        self.always_fail
            .lock()
            .expect("InMemoryNatsPublisher mutex poisoned")
            .insert(subject.to_string(), ());
    }

    /// 取出该 subject 累积的所有 payload（FIFO）
    pub fn messages(&self, subject: &str) -> Vec<Vec<u8>> {
        self.store
            .lock()
            .expect("InMemoryNatsPublisher mutex poisoned")
            .get(subject)
            .cloned()
            .unwrap_or_default()
    }

    /// 计数该 subject 累积的 message 数
    pub fn received_count(&self, subject: &str) -> usize {
        self.messages(subject).len()
    }
}

impl Default for InMemoryNatsPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PushNatsPublisher for InMemoryNatsPublisher {
    async fn publish(&self, subject: &str, payload: &[u8]) -> PushDispatcherResult<()> {
        // 检查 always_fail
        if self
            .always_fail
            .lock()
            .expect("InMemoryNatsPublisher mutex poisoned")
            .contains_key(subject)
        {
            return Err(PushDispatcherError::Publish(format!(
                "mock always_fail: subject={}",
                subject
            )));
        }
        // 检查 fail_first_publish: 首次必失败, 之后删标记
        let should_fail_first = {
            let mut fails = self
                .fail_first_publish
                .lock()
                .expect("InMemoryNatsPublisher mutex poisoned");
            if fails.remove(subject).is_some() {
                true
            } else {
                false
            }
        };
        if should_fail_first {
            return Err(PushDispatcherError::Publish(format!(
                "mock fail_first_publish: subject={}",
                subject
            )));
        }
        // 成功 publish
        self.store
            .lock()
            .expect("InMemoryNatsPublisher mutex poisoned")
            .entry(subject.to_string())
            .or_default()
            .push(payload.to_vec());
        Ok(())
    }
}

/// Push DLQ 仓储（per Q7 决策: 新表 `push_dlq`）
#[async_trait]
pub trait PushDlqRepository: Send + Sync {
    async fn save(&self, entry: &PushDlqEntry) -> PushDispatcherResult<PushDlqEntry>;
    async fn list_all(&self) -> PushDispatcherResult<Vec<PushDlqEntry>>;
    async fn count(&self) -> usize;
}

/// InMemory DLQ 仓储（per Q7 决策: 测试 + 不引 sqlx 强约束）
pub struct InMemoryPushDlqRepository {
    inner: Mutex<Vec<PushDlqEntry>>,
}

impl InMemoryPushDlqRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryPushDlqRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PushDlqRepository for InMemoryPushDlqRepository {
    async fn save(&self, entry: &PushDlqEntry) -> PushDispatcherResult<PushDlqEntry> {
        self.inner
            .lock()
            .expect("InMemoryPushDlqRepository mutex poisoned")
            .push(entry.clone());
        Ok(entry.clone())
    }
    async fn list_all(&self) -> PushDispatcherResult<Vec<PushDlqEntry>> {
        Ok(self
            .inner
            .lock()
            .expect("InMemoryPushDlqRepository mutex poisoned")
            .clone())
    }
    async fn count(&self) -> usize {
        self.inner
            .lock()
            .expect("InMemoryPushDlqRepository mutex poisoned")
            .len()
    }
}

/// PushDispatcher trait（per Q7 决策: 业务层抽象）
#[async_trait]
pub trait PushDispatcher: Send + Sync {
    /// 投递单个 push 请求, 走 NATS 主题 + retry + DLQ
    async fn dispatch(&self, req: &PushDeliveryRequest) -> DispatchOutcome;
}

/// NATS Push Dispatcher（per Q7 决策: 走 NATS, retry 复用 economy outbox+saga 模式）
pub struct NatsPushDispatcher {
    nats: std::sync::Arc<dyn PushNatsPublisher>,
    dlq: std::sync::Arc<dyn PushDlqRepository>,
    config: DispatcherConfig,
}

impl NatsPushDispatcher {
    pub fn new(
        nats: std::sync::Arc<dyn PushNatsPublisher>,
        dlq: std::sync::Arc<dyn PushDlqRepository>,
        config: DispatcherConfig,
    ) -> Self {
        Self { nats, dlq, config }
    }
}

#[async_trait]
impl PushDispatcher for NatsPushDispatcher {
    async fn dispatch(&self, req: &PushDeliveryRequest) -> DispatchOutcome {
        // 1. sanitizer 校验: 失败直接 DLQ, 不重试
        if let Err(e) = sanitize_push_content(&req.title, &req.body) {
            // DLQ 记录 + 推到 social.push.dlq (尽力发, 失败也仍 DLQ)
            let entry = PushDlqEntry::new(req.clone(), 0, format!("sanitizer_reject: {}", e));
            let _ = self.dlq.save(&entry).await;
            let dlq_payload = serde_json::to_vec(&entry).unwrap_or_default();
            let _ = self.nats.publish(PUSH_DLQ_SUBJECT, &dlq_payload).await;
            return DispatchOutcome::RejectedBySanitizer { reason: e };
        }

        // 2. serialize + 投到 social.push.delivery, 失败 retry (exponential backoff)
        let payload = serde_json::to_vec(req).unwrap_or_default();
        let mut last_error = String::new();
        for attempt in 1..=self.config.max_attempts {
            match self.nats.publish(PUSH_DELIVERY_SUBJECT, &payload).await {
                Ok(()) => {
                    return DispatchOutcome::Delivered { attempts: attempt };
                }
                Err(e) => {
                    last_error = e.to_string();
                    if attempt < self.config.max_attempts {
                        // exponential backoff: base * 2^(attempt-1)
                        let backoff = self.config.backoff_base * 2u32.pow(attempt - 1);
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        // 3. retry 耗尽 → DLQ
        let entry = PushDlqEntry::new(req.clone(), self.config.max_attempts, last_error.clone());
        let _ = self.dlq.save(&entry).await;
        let dlq_payload = serde_json::to_vec(&entry).unwrap_or_default();
        let _ = self.nats.publish(PUSH_DLQ_SUBJECT, &dlq_payload).await;
        DispatchOutcome::DeadLettered {
            attempts: self.config.max_attempts,
            last_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_delivery_request_serializes_all_fields() {
        let req = PushDeliveryRequest {
            account_id: "acc-1".to_string(),
            category: "promo".to_string(),
            title: "Welcome".to_string(),
            body: "Hello world".to_string(),
            dedup_window_id: 1700000000,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("account_id"));
        assert!(json.contains("category"));
        assert!(json.contains("title"));
        assert!(json.contains("body"));
        assert!(json.contains("dedup_window_id"));
    }

    #[test]
    fn delivery_result_code_roundtrip() {
        for v in 0..=3 {
            let code = DeliveryResultCode::from_i32(v).unwrap();
            assert_eq!(code.as_i32(), v);
        }
        assert!(DeliveryResultCode::from_i32(99).is_none());
    }

    #[test]
    fn sanitize_rejects_banned_patterns() {
        assert!(sanitize_push_content("Hello", "World").is_ok());
        assert!(sanitize_push_content("<script>alert(1)</script>", "x").is_err());
        assert!(sanitize_push_content("x", "javascript:alert(1)").is_err());
        assert!(sanitize_push_content("x", "data:text/html").is_err());
    }

    #[test]
    fn push_delivery_result_contains_code() {
        let r = PushDeliveryResult { result_code: DeliveryResultCode::Delivered };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("result_code"));
        let back: PushDeliveryResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.result_code, DeliveryResultCode::Delivered);
    }

    #[test]
    fn sanitize_accepts_long_safe_text() {
        let long = "a".repeat(10_000);
        assert!(sanitize_push_content(&long, &long).is_ok());
    }

    // ========================================================================
    // Q7 UT: PushDispatcher (per RGS-OPEN-QA-2026-08-31 v0.2 §Q7)
    // ========================================================================

    fn req_ok() -> PushDeliveryRequest {
        PushDeliveryRequest {
            account_id: "acc-ut".to_string(),
            category: "promo".to_string(),
            title: "Hello".to_string(),
            body: "World".to_string(),
            dedup_window_id: 1000,
        }
    }

    #[tokio::test]
    async fn push_dispatcher_happy_path_delivered_first_attempt() {
        let nats = std::sync::Arc::new(InMemoryNatsPublisher::new());
        let dlq = std::sync::Arc::new(InMemoryPushDlqRepository::new());
        let dispatcher = NatsPushDispatcher::new(
            nats.clone(),
            dlq.clone(),
            DispatcherConfig {
                max_attempts: 3,
                backoff_base: Duration::from_millis(1),
            },
        );

        let outcome = dispatcher.dispatch(&req_ok()).await;
        assert!(matches!(outcome, DispatchOutcome::Delivered { attempts: 1 }));
        // 仅 1 条 message 到 social.push.delivery
        assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 1);
        // 0 条 DLQ
        assert_eq!(dlq.count().await, 0);
        // 0 条到 DLQ 主题
        assert_eq!(nats.received_count(PUSH_DLQ_SUBJECT), 0);
    }

    #[tokio::test]
    async fn push_dispatcher_retry_succeeds_on_second_attempt() {
        let nats = std::sync::Arc::new(InMemoryNatsPublisher::new());
        nats.fail_first_publish(PUSH_DELIVERY_SUBJECT);
        let dlq = std::sync::Arc::new(InMemoryPushDlqRepository::new());
        let dispatcher = NatsPushDispatcher::new(
            nats.clone(),
            dlq.clone(),
            DispatcherConfig {
                max_attempts: 3,
                backoff_base: Duration::from_millis(1),
            },
        );

        let outcome = dispatcher.dispatch(&req_ok()).await;
        assert!(
            matches!(outcome, DispatchOutcome::Delivered { attempts: 2 }),
            "第 1 次失败, 第 2 次成功, 期望 attempts=2, got {:?}",
            outcome
        );
        // 2 条 message 到 social.push.delivery (1 fail + 1 success, 但失败不进 store)
        assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 1);
        // 0 条 DLQ
        assert_eq!(dlq.count().await, 0);
    }

    #[tokio::test]
    async fn push_dispatcher_retry_exhausted_routes_to_dlq() {
        let nats = std::sync::Arc::new(InMemoryNatsPublisher::new());
        nats.always_fail(PUSH_DELIVERY_SUBJECT);
        let dlq = std::sync::Arc::new(InMemoryPushDlqRepository::new());
        let dispatcher = NatsPushDispatcher::new(
            nats.clone(),
            dlq.clone(),
            DispatcherConfig {
                max_attempts: 3,
                backoff_base: Duration::from_millis(1),
            },
        );

        let outcome = dispatcher.dispatch(&req_ok()).await;
        assert!(
            matches!(outcome, DispatchOutcome::DeadLettered { attempts: 3, .. }),
            "3 次都失败, 期望 DeadLettered{{attempts: 3, ..}}, got {:?}",
            outcome
        );
        // 1 条 DLQ entry
        assert_eq!(dlq.count().await, 1);
        let dlq_entries = dlq.list_all().await.unwrap();
        assert_eq!(dlq_entries.len(), 1);
        assert_eq!(dlq_entries[0].attempts, 3);
        assert!(dlq_entries[0].last_error.contains("always_fail"));
        // 0 条到 social.push.delivery
        assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 0);
        // 1 条到 social.push.dlq
        assert_eq!(nats.received_count(PUSH_DLQ_SUBJECT), 1);
    }

    #[tokio::test]
    async fn push_dispatcher_sanitizer_reject_goes_directly_to_dlq_no_retry() {
        let nats = std::sync::Arc::new(InMemoryNatsPublisher::new());
        let dlq = std::sync::Arc::new(InMemoryPushDlqRepository::new());
        let dispatcher = NatsPushDispatcher::new(
            nats.clone(),
            dlq.clone(),
            DispatcherConfig {
                max_attempts: 3,
                backoff_base: Duration::from_millis(1),
            },
        );

        let bad_req = PushDeliveryRequest {
            account_id: "acc-xss".to_string(),
            category: "promo".to_string(),
            title: "<script>alert(1)</script>".to_string(),
            body: "evil".to_string(),
            dedup_window_id: 1,
        };
        let outcome = dispatcher.dispatch(&bad_req).await;
        assert!(
            matches!(outcome, DispatchOutcome::RejectedBySanitizer { .. }),
            "sanitizer 拒绝应直接进 DLQ, got {:?}",
            outcome
        );
        // 0 条到 social.push.delivery (sanitizer 拒, 不进 NATS)
        assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 0);
        // 1 条 DLQ entry + 1 条到 social.push.dlq
        assert_eq!(dlq.count().await, 1);
        assert_eq!(nats.received_count(PUSH_DLQ_SUBJECT), 1);
    }

    #[tokio::test]
    async fn push_dispatcher_happy_path_produces_3_deliveries_when_2_succeed_after_1_fail() {
        // 3 个 req, 第 1 个 fail-first, 第 2/3 个直接成功
        let nats = std::sync::Arc::new(InMemoryNatsPublisher::new());
        nats.fail_first_publish(PUSH_DELIVERY_SUBJECT);
        let dlq = std::sync::Arc::new(InMemoryPushDlqRepository::new());
        let dispatcher = NatsPushDispatcher::new(
            nats.clone(),
            dlq.clone(),
            DispatcherConfig {
                max_attempts: 3,
                backoff_base: Duration::from_millis(1),
            },
        );

        let mut r1 = req_ok();
        r1.account_id = "a".to_string();
        let mut r2 = req_ok();
        r2.account_id = "b".to_string();
        let mut r3 = req_ok();
        r3.account_id = "c".to_string();

        let o1 = dispatcher.dispatch(&r1).await;
        // 第 1 次 publish 失败 (因为 fail_first), 第 2 次成功
        assert!(matches!(o1, DispatchOutcome::Delivered { attempts: 2 }));
        // 但 fail_first 是 once-per-subject: 已被消费, 后续 publish 不会 fail
        let o2 = dispatcher.dispatch(&r2).await;
        assert!(matches!(o2, DispatchOutcome::Delivered { attempts: 1 }));
        let o3 = dispatcher.dispatch(&r3).await;
        assert!(matches!(o3, DispatchOutcome::Delivered { attempts: 1 }));

        // 3 条都成功 publish 到 social.push.delivery
        assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 3);
        // 0 DLQ
        assert_eq!(dlq.count().await, 0);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// DeliveryResultCode 在合法 4 值内 roundtrip 不变
        #[test]
        fn delivery_result_code_roundtrip_all_valid(v in 0i32..=3) {
            let code = DeliveryResultCode::from_i32(v).unwrap();
            prop_assert_eq!(code.as_i32(), v);
        }

        /// 任意不含禁用模式的 title/body 必 sanitize 通过
        #[test]
        fn sanitize_passes_for_safe_text(
            title in "[A-Za-z0-9 .,_!?-]{0,128}",
            body in "[A-Za-z0-9 .,_!?-]{0,256}",
        ) {
            // 我们的安全字符集不包含 <script> / javascript: / data: 等禁用模式
            prop_assert!(sanitize_push_content(&title, &body).is_ok());
        }

        /// 包含 <script> 任意 title 必被拒
        #[test]
        fn sanitize_rejects_script_in_title(prefix in ".*", suffix in ".*") {
            let title = format!("{}<script>{}</script>", prefix, suffix);
            prop_assert!(sanitize_push_content(&title, "ok").is_err());
        }
    }
}
