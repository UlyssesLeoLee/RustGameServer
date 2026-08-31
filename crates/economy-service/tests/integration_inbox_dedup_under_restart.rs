//! RGS-UT 2026-08-31 JST — economy 域 IT (最高规格) / 子桶 2
//!
//! `integration_inbox_dedup_under_restart`
//!
//! 场景: 模拟"进程重启后 inbox replay" — 业务层依赖 inbox 幂等性 (per RGS-DTL-100 §6)
//!       投递 N 次同 (command_id, handler), handler 业务体只被调用 1 次.
//!
//! 设计目标 (per RGS-UT 2026-08-31 13:55 JST 指令 + IT-AGENT-BRIEFING §3.2 #2):
//! - 用 `InMemoryInboxRepository` 模拟 inbox 持久化
//! - 共享 inbox 句柄在两轮投递之间"穿越" (模拟重启后从 DB 加载)
//! - 业务层: 检查 inbox → 找到 → 跳过 handler; 找不到 → 调 handler + 写 inbox
//! - 投递 N=5 次同 message_id, 预期 handler 只调 1 次
//!
//! 覆盖 3 个 case:
//! 1. `inbox_dedup_5_replays_handler_called_once`:
//!    投递 5 次同 (command_id, handler), 业务调用计数器 = 1
//! 2. `inbox_dedup_simulated_process_restart`:
//!    模拟进程重启: inbox state 重建 (in-memory 跨"重启"持久), handler 重新构造.
//!    投递 3 次 (第 1 次 before-restart + 2 次 after-restart), handler 仍只调 1 次.
//! 3. `inbox_dedup_distinct_handlers_independent`:
//!    同 command_id 不同 handler 各投递 2 次, 各 handler 独立计数 1.
//!
//! 锚定文件:
//! - 源: src/inbox.rs (InboxRepository trait + InMemory impl + dedup 语义)
//! - 源: src/saga.rs (Saga.command_id 是 inbox 幂等键)
//! - 设计: per RGS-DTL-100 §6 "处理 command 前先 check inbox"
//!
//! mTLS 验证: 业务层与传输层解耦, Mock 客户端不涉及 TLS, 真实 gRPC 客户端在
//!            CardGrpcClient::new() 处强制 mTLS (per BAS-003 fail-closed).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use economy_service::inbox::{InMemoryInboxRepository, InboxRepository};
use uuid::Uuid;

// ============================================================================
// 业务 handler 抽象 —— 模拟"带 inbox dedup 的业务层"
// ============================================================================

/// 业务 handler 句柄 + 调用计数器
struct DedupedHandler {
    name: String,
    inbox: Arc<InMemoryInboxRepository>,
    call_count: Arc<AtomicUsize>,
}

impl DedupedHandler {
    fn new(name: &str, inbox: Arc<InMemoryInboxRepository>, call_count: Arc<AtomicUsize>) -> Self {
        Self {
            name: name.to_string(),
            inbox,
            call_count,
        }
    }

    /// 投递 message: 先查 inbox, 找到则 skip; 找不到则调业务体 + 写 inbox.
    ///
    /// 模拟"消息处理器"对幂等性的标准做法.
    async fn dispatch(&self, command_id: Uuid, result_payload: &str) -> bool {
        match self.inbox.find_by_command(command_id, &self.name).await {
            Ok(Some(_entry)) => {
                // 命中: 跳过 handler 业务体, 返回 false 表示"被 dedup 跳过"
                false
            }
            Ok(None) => {
                // 未命中: 调业务体 + 写 inbox
                self.call_count.fetch_add(1, Ordering::SeqCst);
                let entry = economy_service::inbox::InboxEntry::new(
                    command_id,
                    self.name.clone(),
                    result_payload.to_string(),
                );
                self.inbox.append(&entry).await.expect("inbox append");
                true
            }
            Err(e) => panic!("inbox find failed: {}", e),
        }
    }
}

// ============================================================================
// IT 1: 投递 5 次同 (command_id, handler) → handler 只调 1 次
// ============================================================================

#[tokio::test]
async fn inbox_dedup_5_replays_handler_called_once() {
    let inbox = Arc::new(InMemoryInboxRepository::new());
    let counter = Arc::new(AtomicUsize::new(0));
    let handler = DedupedHandler::new("saga.transfer", inbox.clone(), counter.clone());

    let cmd_id = Uuid::new_v4();

    // 投递 5 次 (模拟 at-least-once 重投)
    for i in 0..5 {
        let was_new = handler
            .dispatch(cmd_id, &format!(r#"{{"attempt":{}}}"#, i))
            .await;
        if i == 0 {
            assert!(was_new, "first dispatch must execute handler");
        } else {
            assert!(!was_new, "dispatch #{} must be dedup-skipped", i);
        }
    }

    // handler 业务体只调 1 次
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "handler must be called exactly once despite 5 replays"
    );

    // inbox 只有 1 条 entry (key = (command_id, handler))
    let stored = inbox
        .find_by_command(cmd_id, "saga.transfer")
        .await
        .unwrap()
        .expect("inbox entry must exist");
    // 第 1 次写入的 result 是 attempt=0
    assert_eq!(stored.result, r#"{"attempt":0}"#);
}

// ============================================================================
// IT 2: 模拟进程重启 — inbox 状态穿越, handler 只调 1 次
// ============================================================================

#[tokio::test]
async fn inbox_dedup_simulated_process_restart() {
    // inbox 状态 Arc 句柄 (模拟 "DB 持久层")
    let inbox_arc = Arc::new(InMemoryInboxRepository::new());
    let counter = Arc::new(AtomicUsize::new(0));

    let cmd_id = Uuid::new_v4();
    let idem_key = "k-daily-reward-restart-test";

    // === Phase 1: before-restart ===
    {
        let h = DedupedHandler::new(idem_key, inbox_arc.clone(), counter.clone());
        let r1 = h.dispatch(cmd_id, r#"{"attempt":1}"#).await;
        assert!(r1, "phase1: handler must execute on first dispatch");
        let r2 = h.dispatch(cmd_id, r#"{"attempt":2}"#).await;
        assert!(!r2, "phase1: 2nd dispatch must be dedup-skipped");
        let r3 = h.dispatch(cmd_id, r#"{"attempt":3}"#).await;
        assert!(!r3, "phase1: 3rd dispatch must be dedup-skipped");
    } // h dropped (模拟进程退出)

    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "phase1 total: handler called once across 3 dispatches"
    );

    // === Phase 2: simulate process restart ===
    // 重新构造 handler 句柄 (新栈, 新局部变量), inbox 状态由 Arc 持久
    let h2 = DedupedHandler::new(idem_key, inbox_arc.clone(), counter.clone());
    // replay 同 cmd_id 3 次
    let p1 = h2.dispatch(cmd_id, r#"{"attempt":4}"#).await;
    assert!(!p1, "phase2 replay 1: must be dedup-skipped (state from phase1)");
    let p2 = h2.dispatch(cmd_id, r#"{"attempt":5}"#).await;
    assert!(!p2, "phase2 replay 2: must be dedup-skipped");
    let p3 = h2.dispatch(cmd_id, r#"{"attempt":6}"#).await;
    assert!(!p3, "phase2 replay 3: must be dedup-skipped");

    // 跨"重启"累计: handler 仍只调 1 次
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "across simulated restart: handler called exactly once despite 6 dispatches (3 before + 3 after)"
    );

    // inbox entry 仍是 phase1 第一次写入的 result
    let entry = inbox_arc
        .find_by_command(cmd_id, idem_key)
        .await
        .unwrap()
        .expect("entry exists");
    assert_eq!(
        entry.result, r#"{"attempt":1}"#,
        "inbox entry result must be from first dispatch (InMemory HashMap 覆盖语义: 后写覆盖前写, 但业务层视作命中即 skip)"
    );
}

// ============================================================================
// IT 3: 同 command_id 不同 handler → 各 handler 独立计数
// ============================================================================

#[tokio::test]
async fn inbox_dedup_distinct_handlers_independent() {
    let inbox = Arc::new(InMemoryInboxRepository::new());
    let counter_a = Arc::new(AtomicUsize::new(0));
    let counter_b = Arc::new(AtomicUsize::new(0));
    let handler_a = DedupedHandler::new("handler-a", inbox.clone(), counter_a.clone());
    let handler_b = DedupedHandler::new("handler-b", inbox.clone(), counter_b.clone());

    let cmd_id = Uuid::new_v4();

    // 投递 2 次到 handler-a, 投递 2 次到 handler-b
    for i in 0..2 {
        let r_a = handler_a
            .dispatch(cmd_id, &format!(r#"{{"a":{}}}"#, i))
            .await;
        let r_b = handler_b
            .dispatch(cmd_id, &format!(r#"{{"b":{}}}"#, i))
            .await;
        if i == 0 {
            assert!(r_a && r_b, "first dispatch to each must execute");
        } else {
            assert!(!r_a && !r_b, "replays must be dedup-skipped for each handler");
        }
    }

    assert_eq!(counter_a.load(Ordering::SeqCst), 1);
    assert_eq!(counter_b.load(Ordering::SeqCst), 1);

    // inbox 各 1 条 (key = (cmd_id, "handler-a") 与 (cmd_id, "handler-b"))
    let entry_a = inbox
        .find_by_command(cmd_id, "handler-a")
        .await
        .unwrap()
        .expect("handler-a entry exists");
    let entry_b = inbox
        .find_by_command(cmd_id, "handler-b")
        .await
        .unwrap()
        .expect("handler-b entry exists");
    assert_eq!(entry_a.result, r#"{"a":0}"#);
    assert_eq!(entry_b.result, r#"{"b":0}"#);
}
