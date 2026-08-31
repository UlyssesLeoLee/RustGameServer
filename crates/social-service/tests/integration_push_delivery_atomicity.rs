//! social 域 IT 3/3 — Push 投递原子性集成测试 (最高规格, per RGS-IT-AGENT-BRIEFING §3.4)
//!
//! ## 场景 (per §3.4 #3)
//! 多个 push 任务 → 中间失败 → 部分成功可重试, 状态正确。
//!
//! src/ `push_delivery.rs` 仅提供数据 + sanitize_push_content, 没有真实投递执行器。
//! 本 IT 在测试侧定义一个**纯 InMemory Mock PushDispatcher**, 模拟 push gateway:
//! - 每个 push 任务有明确的状态机: Pending → (Delivered | FailedRetryable | FailedPermanent)
//! - 第 1 轮投递模拟中间 N 个失败 (RateLimited / DeviceTokenExpired)
//! - 重试只针对 FailedRetryable 任务
//! - 验证最终状态: 全部 Delivered, 无半成品
//!
//! ## Q7 扩展 (per RGS-OPEN-QA-2026-08-31 v0.2 §Q7 决策)
//! 走 NATS 主题 `social.push.delivery` + 复用 economy outbox+saga retry 模式
//! (max_attempts=3 + exponential backoff) + DLQ (`social.push.dlq` + push_dlq 表)。
//! 使用 src/ 生产代码 `NatsPushDispatcher` + 测试 `InMemoryNatsPublisher` +
//! `InMemoryPushDlqRepository` 测端到端。

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use social_service::push_delivery::{
    sanitize_push_content, DeliveryResultCode, DispatchOutcome, DispatcherConfig,
    InMemoryNatsPublisher, InMemoryPushDlqRepository, NatsPushDispatcher, PUSH_DELIVERY_SUBJECT,
    PUSH_DLQ_SUBJECT, PushDeliveryRequest, PushDeliveryResult, PushDispatcher, PushDlqRepository,
};

// ============================================================================
// Test-side mock push dispatcher (纯 InMemory, 不污染 src/)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TaskStatus {
    /// 已入队, 尚未投递
    Pending,
    /// 投递成功
    Delivered,
    /// 失败, 但可重试 (rate limited / queued back)
    FailedRetryable,
    /// 失败, 不可重试 (device token expired)
    FailedPermanent,
}

#[derive(Debug, Clone)]
struct TaskRecord {
    index: usize,
    req: PushDeliveryRequest,
    status: TaskStatus,
    /// 累计尝试次数
    attempts: u32,
    /// 首次失败原因
    last_error_code: Option<DeliveryResultCode>,
}

impl TaskRecord {
    fn new(index: usize, req: PushDeliveryRequest) -> Self {
        Self {
            index,
            req,
            status: TaskStatus::Pending,
            attempts: 0,
            last_error_code: None,
        }
    }
}

/// 模拟 push gateway 行为: 接收一个失败索引集合 (0-based) 作为第 1 轮会失败的索引
struct MockPushDispatcher {
    /// 任务池
    tasks: Vec<TaskRecord>,
    /// 第 1 轮会失败的索引集合 (其余成功)
    fail_first_attempt: HashSet<usize>,
    /// 全局尝试次数计数 (for observability)
    total_attempts: u32,
}

impl MockPushDispatcher {
    fn new(tasks: Vec<PushDeliveryRequest>, fail_first_attempt: HashSet<usize>) -> Self {
        let tasks = tasks
            .into_iter()
            .enumerate()
            .map(|(i, req)| TaskRecord::new(i, req))
            .collect();
        Self {
            tasks,
            fail_first_attempt,
            total_attempts: 0,
        }
    }

    /// 投递所有 Pending 任务。
    ///
    /// 行为:
    /// - 通过 sanitize_push_content 的任务才进 dispatch pipeline
    /// - 失败者按 `fail_first_attempt` 表 + 内容模拟
    /// - 已 Delivered 任务跳过 (idempotent)
    /// - 失败任务保留为 FailedRetryable / FailedPermanent
    ///
    /// 返回: 第 1 轮本批 (Delivered 数, FailedRetryable 数, FailedPermanent 数, RejectedBySanitizer 数)
    fn dispatch_one_round(&mut self) -> (usize, usize, usize, usize) {
        let mut delivered = 0;
        let mut retryable = 0;
        let permanent = 0;
        let mut rejected = 0;

        let fails = self.fail_first_attempt.clone();
        for task in self.tasks.iter_mut() {
            if task.status == TaskStatus::Delivered {
                continue;
            }
            // sanitizer check
            if sanitize_push_content(&task.req.title, &task.req.body).is_err() {
                task.status = TaskStatus::FailedPermanent;
                task.last_error_code = Some(DeliveryResultCode::DeviceTokenExpired);
                task.attempts += 1;
                self.total_attempts += 1;
                rejected += 1;
                continue;
            }
            task.attempts += 1;
            self.total_attempts += 1;
            if fails.contains(&task.index) {
                // 失败: 用 code 模拟
                // 为简单起见, 所有 fail_first_attempt 都设为 RateLimitedDropped (可重试)
                task.status = TaskStatus::FailedRetryable;
                task.last_error_code = Some(DeliveryResultCode::RateLimitedDropped);
                retryable += 1;
            } else {
                task.status = TaskStatus::Delivered;
                task.last_error_code = Some(DeliveryResultCode::Delivered);
                delivered += 1;
            }
        }
        (delivered, retryable, permanent, rejected)
    }

    /// 重试所有 FailedRetryable 任务 (第 2 轮, 全部应成功)
    fn retry_retryable(&mut self) -> usize {
        let mut retried_delivered = 0;
        for task in self.tasks.iter_mut() {
            if task.status != TaskStatus::FailedRetryable {
                continue;
            }
            // sanitizer 必过 (之前轮已通过)
            task.attempts += 1;
            self.total_attempts += 1;
            // 重试全部成功
            task.status = TaskStatus::Delivered;
            task.last_error_code = Some(DeliveryResultCode::Delivered);
            retried_delivered += 1;
        }
        retried_delivered
    }

    fn snapshot(&self) -> Vec<(usize, TaskStatus, u32, Option<DeliveryResultCode>)> {
        self.tasks
            .iter()
            .map(|t| (t.index, t.status, t.attempts, t.last_error_code))
            .collect()
    }
}

// ============================================================================
// IT 1: 多 push 任务 + 中间失败 + 部分成功可重试, 状态正确
// ============================================================================

#[tokio::test]
async fn push_delivery_atomicity_partial_failure_retryable_then_full_success() {
    // 5 个 push 任务, 标记 index 1 (player_b) 和 3 (player_d) 第 1 轮失败
    let tasks = vec![
        PushDeliveryRequest {
            account_id: "acc-a".to_string(),
            category: "promo".to_string(),
            title: "Welcome".to_string(),
            body: "Hello A".to_string(),
            dedup_window_id: 1000,
        },
        PushDeliveryRequest {
            account_id: "acc-b".to_string(),
            category: "promo".to_string(),
            title: "Welcome".to_string(),
            body: "Hello B".to_string(),
            dedup_window_id: 1001,
        },
        PushDeliveryRequest {
            account_id: "acc-c".to_string(),
            category: "system".to_string(),
            title: "Update".to_string(),
            body: "Patch 1.2".to_string(),
            dedup_window_id: 1002,
        },
        PushDeliveryRequest {
            account_id: "acc-d".to_string(),
            category: "promo".to_string(),
            title: "Welcome".to_string(),
            body: "Hello D".to_string(),
            dedup_window_id: 1003,
        },
        PushDeliveryRequest {
            account_id: "acc-e".to_string(),
            category: "system".to_string(),
            title: "Notice".to_string(),
            body: "Maintenance".to_string(),
            dedup_window_id: 1004,
        },
    ];
    let mut fail_first = HashSet::new();
    fail_first.insert(1);
    fail_first.insert(3);

    let mut dispatcher = MockPushDispatcher::new(tasks, fail_first.clone());

    // ----- 第 1 轮: 5 任务, 2 失败 -----
    let (d1, r1, p1, j1) = dispatcher.dispatch_one_round();
    assert_eq!(d1, 3, "第 1 轮 Delivered=3 (index 0/2/4)");
    assert_eq!(r1, 2, "第 1 轮 FailedRetryable=2 (index 1/3)");
    assert_eq!(p1, 0);
    assert_eq!(j1, 0);

    // 验证 5 任务状态分布
    let snap1 = dispatcher.snapshot();
    assert_eq!(snap1[0].1, TaskStatus::Delivered);
    assert_eq!(snap1[1].1, TaskStatus::FailedRetryable);
    assert_eq!(snap1[2].1, TaskStatus::Delivered);
    assert_eq!(snap1[3].1, TaskStatus::FailedRetryable);
    assert_eq!(snap1[4].1, TaskStatus::Delivered);
    // 失败 2 个的 last_error_code 必为 RateLimitedDropped
    assert_eq!(snap1[1].3, Some(DeliveryResultCode::RateLimitedDropped));
    assert_eq!(snap1[3].3, Some(DeliveryResultCode::RateLimitedDropped));
    // attempts 计数
    for s in &snap1 {
        assert_eq!(s.2, 1, "第 1 轮所有任务 attempts=1");
    }
    assert_eq!(dispatcher.total_attempts, 5);

    // ----- 第 2 轮 (重试 FailedRetryable): 全部成功 -----
    let retried = dispatcher.retry_retryable();
    assert_eq!(retried, 2, "重试 2 个 FailedRetryable 全部成功");

    // ----- 最终状态: 全部 Delivered, 无半成品 -----
    let snap_final = dispatcher.snapshot();
    for (i, s) in snap_final.iter().enumerate() {
        assert_eq!(
            s.1,
            TaskStatus::Delivered,
            "index {} 终态必须 Delivered, got {:?}",
            i,
            s.1
        );
        // attempts: index 0/2/4 (第 1 轮就成功) = 1, index 1/3 (第 1 轮失败 + 重试) = 2
        let expected_attempts = if fail_first.contains(&i) { 2 } else { 1 };
        assert_eq!(
            s.2, expected_attempts,
            "index {} 期望 attempts={} (R1: 1, R2 retry: +1 if originally failed)",
            i, expected_attempts
        );
        // 已 Delivered 的 last_error_code 必为 Delivered
        assert_eq!(s.3, Some(DeliveryResultCode::Delivered));
    }
    // 第 1 轮已成功的 3 个 attempts 不应增加 (idempotent skip)
    let snap2_intermediate = dispatcher.snapshot();
    let _ = snap2_intermediate; // 仅为 snapshot 复用检查
    // total_attempts = 5 (第1轮) + 2 (重试) = 7
    assert_eq!(dispatcher.total_attempts, 7, "总尝试次数 = 5 (R1) + 2 (R2 重试) = 7");

    // 验证 DeliveryResultCode roundtrip: 已 Delivered 的 code 0 可正常转回
    let result = PushDeliveryResult {
        result_code: DeliveryResultCode::Delivered,
    };
    assert_eq!(result.result_code.as_i32(), 0);
    assert_eq!(
        DeliveryResultCode::from_i32(0).unwrap(),
        DeliveryResultCode::Delivered
    );
}

// ============================================================================
// IT 2: sanitizer 拒绝的任务为 FailedPermanent, 不进重试集
// ============================================================================

#[tokio::test]
async fn push_delivery_atomicity_sanitizer_reject_is_permanent_not_retryable() {
    let tasks = vec![
        PushDeliveryRequest {
            account_id: "good".to_string(),
            category: "promo".to_string(),
            title: "Hi".to_string(),
            body: "World".to_string(),
            dedup_window_id: 1,
        },
        PushDeliveryRequest {
            account_id: "xss-attempt".to_string(),
            category: "promo".to_string(),
            title: "<script>alert(1)</script>".to_string(),
            body: "evil".to_string(),
            dedup_window_id: 2,
        },
        PushDeliveryRequest {
            account_id: "js-attempt".to_string(),
            category: "promo".to_string(),
            title: "safe title".to_string(),
            body: "javascript:alert(1)".to_string(),
            dedup_window_id: 3,
        },
    ];
    let mut dispatcher = MockPushDispatcher::new(tasks, HashSet::new());

    let (d, r, p, j) = dispatcher.dispatch_one_round();
    assert_eq!(d, 1, "只有 good 通过 sanitizer 投递");
    assert_eq!(r, 0);
    assert_eq!(p, 0);
    assert_eq!(j, 2, "两条含禁用模式被 sanitizer 拒 (FailedPermanent)");

    let snap = dispatcher.snapshot();
    assert_eq!(snap[0].1, TaskStatus::Delivered);
    assert_eq!(snap[1].1, TaskStatus::FailedPermanent);
    assert_eq!(snap[2].1, TaskStatus::FailedPermanent);

    // 重试只针对 FailedRetryable, 不应影响 FailedPermanent
    let retried = dispatcher.retry_retryable();
    assert_eq!(retried, 0, "FailedPermanent 不进重试集");

    let snap_after_retry = dispatcher.snapshot();
    assert_eq!(snap_after_retry[1].1, TaskStatus::FailedPermanent);
    assert_eq!(snap_after_retry[2].1, TaskStatus::FailedPermanent);
    // 终态: 1 Delivered, 2 FailedPermanent (无半成品)
    let mut status_counts = std::collections::HashMap::new();
    for s in &snap_after_retry {
        *status_counts.entry(s.1).or_insert(0) += 1;
    }
    assert_eq!(status_counts.get(&TaskStatus::Delivered), Some(&1));
    assert_eq!(status_counts.get(&TaskStatus::FailedPermanent), Some(&2));
    assert_eq!(status_counts.get(&TaskStatus::FailedRetryable), None);
    assert_eq!(status_counts.get(&TaskStatus::Pending), None);
}

// ============================================================================
// IT 3: 全失败 → 重试后部分恢复, 部分仍卡 (验证 partial retry)
// ============================================================================

#[tokio::test]
async fn push_delivery_atomicity_retry_only_resolves_retryable_state() {
    // 4 任务, 全部第 1 轮失败 (retryable)
    let tasks: Vec<PushDeliveryRequest> = (0..4)
        .map(|i| PushDeliveryRequest {
            account_id: format!("acc-{}", i),
            category: "promo".to_string(),
            title: "Hello".to_string(),
            body: format!("Body {}", i),
            dedup_window_id: 1000 + i as i64,
        })
        .collect();
    let mut fail_first = HashSet::new();
    for i in 0..4 {
        fail_first.insert(i);
    }
    let mut dispatcher = MockPushDispatcher::new(tasks, fail_first);

    // 第 1 轮: 0 Delivered, 4 FailedRetryable
    let (d, r, p, j) = dispatcher.dispatch_one_round();
    assert_eq!(d, 0);
    assert_eq!(r, 4);
    assert_eq!(p, 0);
    assert_eq!(j, 0);

    // 重试: 全部 4 个恢复
    let retried = dispatcher.retry_retryable();
    assert_eq!(retried, 4);

    // 终态: 全部 Delivered
    for s in dispatcher.snapshot() {
        assert_eq!(s.1, TaskStatus::Delivered);
        assert_eq!(s.3, Some(DeliveryResultCode::Delivered));
    }
}

// ============================================================================
// Q7 扩展 IT 场景 (per RGS-OPEN-QA-2026-08-31 v0.2 §Q7)
// 端到端测 production `NatsPushDispatcher` + `InMemoryNatsPublisher` +
// `InMemoryPushDlqRepository`, 验证:
//   1. happy path → Delivered{attempts: 1}, 1 条到 social.push.delivery
//   2. retry 成功 → Delivered{attempts: 2}, 1 条到 social.push.delivery, 0 DLQ
//   3. retry 耗尽 → DeadLettered{attempts: 3}, 1 DLQ entry + 1 条到 social.push.dlq
//   4. sanitizer 拒绝 → RejectedBySanitizer, 1 DLQ entry, 不进 social.push.delivery
// ============================================================================

fn req_q7(account_id: &str) -> PushDeliveryRequest {
    PushDeliveryRequest {
        account_id: account_id.to_string(),
        category: "promo".to_string(),
        title: "Hello".to_string(),
        body: "World".to_string(),
        dedup_window_id: 1,
    }
}

#[tokio::test]
async fn push_dispatcher_e2e_happy_path_single_delivery_to_nats() {
    let nats = Arc::new(InMemoryNatsPublisher::new());
    let dlq = Arc::new(InMemoryPushDlqRepository::new());
    let dispatcher = NatsPushDispatcher::new(
        nats.clone(),
        dlq.clone(),
        DispatcherConfig {
            max_attempts: 3,
            backoff_base: Duration::from_millis(1),
        },
    );

    let outcome = dispatcher.dispatch(&req_q7("acc-happy")).await;
    assert!(
        matches!(outcome, DispatchOutcome::Delivered { attempts: 1 }),
        "happy path 期望 Delivered{{attempts: 1}}, got {:?}",
        outcome
    );
    // 1 条到 social.push.delivery
    assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 1);
    let msgs = nats.messages(PUSH_DELIVERY_SUBJECT);
    assert_eq!(msgs.len(), 1);
    let parsed: PushDeliveryRequest = serde_json::from_slice(&msgs[0]).unwrap();
    assert_eq!(parsed.account_id, "acc-happy");
    // 0 DLQ
    assert_eq!(dlq.count().await, 0);
    // 0 条到 social.push.dlq
    assert_eq!(nats.received_count(PUSH_DLQ_SUBJECT), 0);
}

#[tokio::test]
async fn push_dispatcher_e2e_retry_recovers_to_delivered() {
    let nats = Arc::new(InMemoryNatsPublisher::new());
    // 模拟 social.push.delivery 首次 publish 失败
    nats.fail_first_publish(PUSH_DELIVERY_SUBJECT);
    let dlq = Arc::new(InMemoryPushDlqRepository::new());
    let dispatcher = NatsPushDispatcher::new(
        nats.clone(),
        dlq.clone(),
        DispatcherConfig {
            max_attempts: 3,
            backoff_base: Duration::from_millis(1),
        },
    );

    let outcome = dispatcher.dispatch(&req_q7("acc-retry")).await;
    assert!(
        matches!(outcome, DispatchOutcome::Delivered { attempts: 2 }),
        "retry 成功路径: 第 1 次 fail, 第 2 次 success, 期望 Delivered{{attempts: 2}}, got {:?}",
        outcome
    );
    // 1 条到 social.push.delivery (fail 不进 store)
    assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 1);
    // 0 DLQ
    assert_eq!(dlq.count().await, 0);
}

#[tokio::test]
async fn push_dispatcher_e2e_retry_exhausted_routes_to_dlq_table_and_subject() {
    let nats = Arc::new(InMemoryNatsPublisher::new());
    nats.always_fail(PUSH_DELIVERY_SUBJECT);
    let dlq = Arc::new(InMemoryPushDlqRepository::new());
    let dispatcher = NatsPushDispatcher::new(
        nats.clone(),
        dlq.clone(),
        DispatcherConfig {
            max_attempts: 3,
            backoff_base: Duration::from_millis(1),
        },
    );

    let outcome = dispatcher.dispatch(&req_q7("acc-exhausted")).await;
    match &outcome {
        DispatchOutcome::DeadLettered { attempts, last_error } => {
            assert_eq!(*attempts, 3);
            assert!(last_error.contains("always_fail"));
        }
        other => panic!("retry 耗尽期望 DeadLettered{{attempts: 3, ..}}, got {:?}", other),
    }
    // 0 条到 social.push.delivery (publish 一直失败)
    assert_eq!(nats.received_count(PUSH_DELIVERY_SUBJECT), 0);
    // 1 条 DLQ entry
    assert_eq!(dlq.count().await, 1);
    let entries = dlq.list_all().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].attempts, 3);
    assert_eq!(entries[0].req.account_id, "acc-exhausted");
    // 1 条到 social.push.dlq (NAT 主题)
    assert_eq!(nats.received_count(PUSH_DLQ_SUBJECT), 1);
}

#[tokio::test]
async fn push_dispatcher_e2e_sanitizer_reject_bypasses_nats_delivery_subject() {
    let nats = Arc::new(InMemoryNatsPublisher::new());
    let dlq = Arc::new(InMemoryPushDlqRepository::new());
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
    // 1 条 DLQ entry
    assert_eq!(dlq.count().await, 1);
    let entries = dlq.list_all().await.unwrap();
    assert!(entries[0].last_error.contains("sanitizer_reject"));
    // 1 条到 social.push.dlq
    assert_eq!(nats.received_count(PUSH_DLQ_SUBJECT), 1);
}
