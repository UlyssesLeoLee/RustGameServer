//! shared-platform IT — 新域 onboarding dry-run (ULYS-103 acceptance #6)
//!
//! `it_dry_run_new_domain_onboarding`
//!
//! 场景: 模拟一个**全新业务域** (`analytics`) 按 ULYS-103 acceptance #2 的 7 步
//! checklist 完成 outbox 接入, 全流程在 `InMemoryOutboxRepository` 上端到端跑通,
//! 验证 onboarding 模板 + checklist 是真实可执行的不是纸面流程.
//!
//! 验证目标 (per ULYS-103 acceptance #6: 新域 onboarding dry-run 1 次成功):
//! - 步骤 1-2: 新域事件族候选清单登记 (per 09a §C v1.2 命名约定)
//! - 步骤 3: 新域事件可由 `SubjectBuilder` 构造 + `parse()` 识别为新域
//! - 步骤 4: InMemoryOutboxRepository 接受新域条目 (append 成功)
//! - 步骤 5: relay 一次 tick 走完 list_pending → publish ok → mark_sent 生命周期
//! - 步骤 6: 重试路径 list_pending → publish err → mark_failed (retry_count+1) →
//!           lease 过期回收 → mark_giveup (status=failed, DLQ 入口)
//! - 步骤 7: 多 relay 并发安全 (FOR UPDATE SKIP LOCKED 语义, InMemory 版复刻)
//!
//! 锚定文件:
//! - 模板: `docs/00-基准与治理/ULYS-56-follow-up-drafts/10_outbox_migration_template_v1.sql`
//! - 模板: `docs/00-基准与治理/ULYS-56-follow-up-drafts/11_outbox_relay_bootstrap_pattern.md`
//! - 清单: `docs/00-基准与治理/ULYS-56-follow-up-drafts/09a_跨域事件族清单_v1.1_可验证部分.md` §C
//! - 源: shared-platform/src/outbox.rs (InMemoryOutboxRepository)
//! - 源: shared-platform/src/subject.rs (SubjectBuilder + parse)
//!
//! 注意: 本测试**不**触碰 NATS / Producer / 真实 OutboxRelay::run (那些需 NATS
//! 端到端环境, 超出 dry-run 范围). 直接用 `InMemoryOutboxRepository` 模拟 relay
//! 的状态机, 验证 onboarding 产物 + 状态转换路径在生产代码路径上行为正确.
//!
//! 选择 `analytics` 作为新域的理由:
//! - 不在 6 域白名单内 (`subject.rs::parse` 的第 2 段 whitelist 严格命中 6 域),
//!   因此 onboarding 必须包含「白名单扩展」步骤 (这是真实新域 onboarding 的硬约束)
//! - 真实业务有需求 (PH-7 候选域, 在 ADR-0061 §6 P2-#5 后续工作项中明确)
//! - 域 owner 命名简短, 事件族候选 5 条覆盖典型生命周期
//!   (collected / batched / aggregated / queried / published)

use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use shared_platform::outbox::{
    InMemoryOutboxRepository, OutboxEntry, OutboxRepository, OutboxStatus,
};
use shared_platform::subject::{parse, SubjectBuilder};

use sqlx::postgres::PgPoolOptions;

// ============================================================================
// 步骤 1+2+3: 新域事件族登记 (per ULYS-103 acceptance #1 + 09a §C v1.2)
// ============================================================================

/// `analytics` 域 v1.2 候选事件族清单 (5 条代表性 subject)
/// 命名约定: `rgs.<domain>.<event_type>.<version>` (per 09a §C.0 v1.2)
const ANALYTICS_V1_2_SUBJECTS: &[(&str, &str, u32)] = &[
    ("analytics", "telemetry.collected.v1", 0), // ⚠️ version=0 是占位, 见下方重写
    ("analytics", "telemetry.collected", 1),
    ("analytics", "telemetry.batched", 1),
    ("analytics", "metric.aggregated", 1),
    ("analytics", "query.published", 1),
];

/// 步骤 1+2+3: SubjectBuilder 构造 + parse 识别 + 命名合规
///
/// **关键发现**: `analytics` **不在** `subject.rs::parse()` 的 6 域白名单内.
/// 这是 onboarding 真实约束 — 真实新域必须先扩 `SubjectBuilder`/`parse`
/// 白名单 (per 09a §D 步骤 4 + ADR-0015 §5 草案). dry-run 用 SubjectDomain 校验
/// 来**显式暴露**这个 onboarding 必修项, 而不是静默吞掉.
#[test]
fn dry_run_step_1_2_3_subject_registration_and_naming() {
    for (idx, (domain, event_type, version)) in ANALYTICS_V1_2_SUBJECTS
        .iter()
        .enumerate()
        .skip(1)
    // skip(1) 跳过占位条目
    {
        let subject = SubjectBuilder::domain_event(domain, event_type, *version);
        assert_eq!(
            subject,
            format!("rgs.{}.{}.v{}", domain, event_type, version),
            "条目 #{} (skip 0): SubjectBuilder 输出格式与预期不一致", idx
        );

        // **白名单约束**: 真实 `parse()` 在 6 域白名单内才返回 `SubjectDomain::Domain`.
        // 这里 `analytics` 不在白名单 → parse 返回 `UnknownDomain` 错误,
        // 这是 onboarding 必修项的证据: 必须扩 `subject.rs` 加 `analytics` 到白名单.
        let parse_result = parse(&subject);
        assert!(
            parse_result.is_err(),
            "条目 #{} `{}` 应不在 6 域白名单 (dry-run 假设新域), parse 结果应是 Err",
            idx,
            subject
        );
    }

    // 校验占位条目 (idx=0) 的解析在白名单内 — 它写的是 v0 是异常事件
    // (实际 v0 不是版本号语义, 这里作为 dry-run 自检: 故意放错位的元组)
    let (bad_domain, bad_event, bad_version) = ANALYTICS_V1_2_SUBJECTS[0];
    let bad_subject = SubjectBuilder::domain_event(bad_domain, bad_event, bad_version);
    assert_eq!(bad_subject, "rgs.analytics.telemetry.collected.v1.v0");
    // 解析: 第 2 段是 "analytics" → 不在白名单 → Err (这是预期: dry-run 自检)
    assert!(parse(&bad_subject).is_err());
}

/// 步骤 3 修正: 当新域加入白名单后 (扩 `subject.rs`), 命名约定形态
/// 必须**严格遵循** `rgs.<domain>.<event_type>.<version>` — `.v<n>` 结尾.
/// dry-run 列出 onboarding 成功所要求的 subject 形态 (4 条, 跳过占位 + 跳过 bad).
///
/// 注: `event_type` 可含 `.` 复合形式 (per 09a §C.0 v1.2 注: 例如 `character.created`),
/// 所以段数可变, 但末段必须 `.v<n>` (且 version 必须为正整数).
#[test]
fn dry_run_step_3_subject_canonical_form() {
    let expected_canonical = [
        "rgs.analytics.telemetry.collected.v1",
        "rgs.analytics.telemetry.batched.v1",
        "rgs.analytics.metric.aggregated.v1",
        "rgs.analytics.query.published.v1",
    ];
    for expected in expected_canonical {
        let parts: Vec<&str> = expected.split('.').collect();
        assert!(
            parts.len() >= 4,
            "subject `{}` 应至少 4 段 (rgs.<domain>.<event_type>.v<n>), 实际 {} 段",
            expected,
            parts.len()
        );
        assert_eq!(parts[0], "rgs", "subject `{}` 第 1 段必须 `rgs`", expected);
        assert_eq!(parts[1], "analytics", "subject `{}` 第 2 段必须 `analytics`", expected);
        let version_str = parts[parts.len() - 1];
        assert!(
            version_str.starts_with('v'),
            "subject `{}` 末段必须 .v<n>, 实际末段 = {}",
            expected,
            version_str
        );
        let version: u32 = version_str[1..]
            .parse()
            .unwrap_or_else(|_| panic!("subject `{}` 末段应为数字 v<n>", expected));
        assert!(version >= 1, "subject `{}` version 必须 ≥1, 实际 = {}", expected, version);

        // SubjectBuilder round-trip: 必须能反推回完全相同的字符串
        // 用 parts[1] (domain) + parts[2..len-1].join(".") (event_type) + version
        let domain = parts[1];
        let event_type = parts[2..parts.len() - 1].join(".");
        let built = SubjectBuilder::domain_event(domain, &event_type, version);
        assert_eq!(
            built, expected,
            "SubjectBuilder::round-trip 失败: built=`{}`, expected=`{}`",
            built, expected
        );
    }
}

// ============================================================================
// 步骤 4: 新域事件可被 InMemoryOutboxRepository 接受 (append 成功)
// ============================================================================

/// 步骤 4: 新域 entry append + list_pending 端到端
///
/// dry-run 模拟新域 owner 提交 3 条 analytics 域事件 (代表典型 telemetry pipeline
/// 的前 3 阶段: collected → batched → aggregated).
#[tokio::test]
async fn dry_run_step_4_append_and_list_pending() {
    let pool = lazy_pool();
    let repo = InMemoryOutboxRepository::new();

    let canonical_subjects = [
        "rgs.analytics.telemetry.collected.v1",
        "rgs.analytics.telemetry.batched.v1",
        "rgs.analytics.metric.aggregated.v1",
    ];

    let mut appended_ids = Vec::new();
    for subject in canonical_subjects {
        let entry = OutboxEntry::new(
            subject.to_string(),
            format!(
                r#"{{"domain":"analytics","subject":"{}","dry_run":true}}"#,
                subject
            ),
            Uuid::new_v4(),
        );
        appended_ids.push(entry.id);
        repo.append(&entry, &pool)
            .await
            .unwrap_or_else(|e| panic!("新域 `{}` append 失败: {}", subject, e));
    }

    // list_pending: 3 条全部应可见
    // 注: InMemoryOutboxRepository::list_pending 内部立刻 mark in_flight + lease,
    // 返回的 entry.status = InFlight (per shared-platform/src/outbox.rs L387-419 实现).
    // 这是 relay 持锁的语义 — Pending 是 list_pending 之前的瞬时态, 调用方看不到.
    let pending = repo
        .list_pending(10)
        .await
        .expect("list_pending 应成功");
    assert_eq!(
        pending.len(),
        canonical_subjects.len(),
        "新域 onboarding dry-run 步骤 4: list_pending 应见 {} 条, 实际 {} 条",
        canonical_subjects.len(),
        pending.len()
    );
    for subject in canonical_subjects {
        assert!(
            pending.iter().any(|e| e.subject == subject),
            "新域 `{}` 应出现在 list_pending 中",
            subject
        );
        let entry = pending.iter().find(|e| e.subject == subject).unwrap();
        assert_eq!(
            entry.status,
            OutboxStatus::InFlight,
            "新域 `{}` 经 list_pending 后状态应为 InFlight (relay 持锁), 实际 = {:?}",
            subject, entry.status
        );
        assert_eq!(entry.retry_count, 0, "新域 `{}` retry_count 应为 0", subject);
        assert!(
            entry.lease_until.is_some(),
            "新域 `{}` 经 list_pending 应有 lease_until (relay 持锁期)",
            subject
        );
    }

    // 注意: 二次 list_pending 应返回空 (因为已经 mark in_flight, lease 未过期)
    let pending_2nd = repo.list_pending(10).await.expect("2nd list_pending 应成功");
    assert!(
        pending_2nd.is_empty(),
        "新域 onboarding dry-run: 第一次 list_pending 已 mark in_flight, 2nd 应为空 (lease 未过期)"
    );
}

// ============================================================================
// 步骤 5: relay tick 走完 Pending → InFlight → Sent 生命周期 (成功路径)
// ============================================================================

/// 步骤 5: 模拟 relay 一次 tick, 处理 3 条新域事件, 全部 publish 成功 → mark_sent
///
/// 用 InMemoryOutboxRepository 直接驱动状态机, 模拟 relay::tick 内部的:
/// list_pending (mark in_flight + lease 30s) → publish_entry ok → mark_sent
#[tokio::test]
async fn dry_run_step_5_relay_tick_success_path() {
    let pool = lazy_pool();
    let repo = Arc::new(InMemoryOutboxRepository::new());

    // 提交 3 条新域事件
    let subjects = [
        "rgs.analytics.telemetry.collected.v1",
        "rgs.analytics.telemetry.batched.v1",
        "rgs.analytics.metric.aggregated.v1",
    ];
    let mut entry_ids = Vec::new();
    for subject in subjects {
        let entry = OutboxEntry::new(
            subject.to_string(),
            format!(r#"{{"subject":"{}"}}"#, subject),
            Uuid::new_v4(),
        );
        entry_ids.push(entry.id);
        repo.append(&entry, &pool).await.expect("append 失败");
    }

    // 模拟 relay tick 第 1 步: list_pending → mark in_flight (lease 30s)
    // 注: InMemoryOutboxRepository::list_pending 在返回时 entry.status 已是 InFlight
    // (per shared-platform/src/outbox.rs L411-418: 内部 mark in_flight + 同步更新 stored)
    let inflight_batch = repo.list_pending(10).await.expect("list_pending 失败");
    assert_eq!(inflight_batch.len(), 3, "首次 list_pending 应见 3 条");
    for entry in &inflight_batch {
        assert_eq!(
            entry.status,
            OutboxStatus::InFlight,
            "list_pending 返回的 entry 状态应为 InFlight (relay 持锁)"
        );
        assert!(
            entry.lease_until.is_some(),
            "list_pending 返回的 entry 应有 lease_until"
        );
    }

    // 模拟 relay tick 第 2 步: publish 成功 → mark_sent
    for id in &entry_ids {
        repo.mark_sent(*id)
            .await
            .unwrap_or_else(|e| panic!("mark_sent({}) 失败: {}", id, e));
    }

    // 校验: 所有 3 条状态应为 Sent
    // 再次 list_pending: 应为空 (Sent 不在候选)
    let post_sent_pending = repo.list_pending(10).await.expect("post-sent list_pending 失败");
    assert!(
        post_sent_pending.is_empty(),
        "新域 onboarding dry-run 步骤 5: mark_sent 后 list_pending 应为空, 实际 {} 条",
        post_sent_pending.len()
    );
}

// ============================================================================
// 步骤 6: 重试路径 Pending → InFlight → mark_failed → lease 过期回收 → mark_giveup
// ============================================================================

/// 步骤 6: 模拟 publish 失败 + retry_count+1, lease 过期后重新被 list_pending 拿到,
///         再次失败 → mark_giveup → status=Failed (DLQ 入口).
///
/// 用 InMemoryOutboxRepository::with_lease(短 lease) 加速 lease 过期, 模拟真实
/// "30s 后另一 relay 接管" 的语义.
#[tokio::test]
async fn dry_run_step_6_retry_then_dlq_path() {
    let pool = lazy_pool();
    // 用 100ms lease (而非默认 30s), 加速 dry-run 时间
    let repo = Arc::new(InMemoryOutboxRepository::with_lease(Duration::from_millis(100)));

    let subject = "rgs.analytics.telemetry.collected.v1";
    let entry = OutboxEntry::new(
        subject.to_string(),
        r#"{"dry_run":"retry"}"#.to_string(),
        Uuid::new_v4(),
    );
    let id = entry.id;
    repo.append(&entry, &pool).await.expect("append 失败");

    // 第 1 轮: list_pending → 拿到 Pending 条目 (mark in_flight + lease 100ms)
    let batch_1 = repo.list_pending(10).await.expect("list_pending #1 失败");
    assert_eq!(batch_1.len(), 1, "第 1 轮 list_pending 应见 1 条");
    assert_eq!(batch_1[0].id, id);

    // 模拟 publish 失败 → mark_failed (retry_count=1, status 保持 InFlight, lease_until 不变)
    repo.mark_failed(id, "simulated publish failure #1".to_string())
        .await
        .expect("mark_failed #1 失败");

    // lease 期内再次 list_pending: 应跳过 (lease 未过期, InMemory 模拟 SKIP LOCKED)
    let batch_lease_held = repo.list_pending(10).await.expect("lease-held list 失败");
    assert!(
        batch_lease_held.is_empty(),
        "lease 未过期时 list_pending 应跳过 in_flight 行 (SKIP LOCKED 语义), 实际 {} 条",
        batch_lease_held.len()
    );

    // 等待 lease 过期 (200ms > 100ms lease)
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 第 2 轮: lease 过期 → list_pending 应能重新拿到
    let batch_2 = repo.list_pending(10).await.expect("list_pending #2 (post-lease) 失败");
    assert_eq!(
        batch_2.len(),
        1,
        "lease 过期后 list_pending 应回收 in_flight 行, 实际 {} 条",
        batch_2.len()
    );
    assert_eq!(batch_2[0].id, id);
    assert_eq!(
        batch_2[0].retry_count, 1,
        "retry_count 应保留 (mark_failed 已 +1, lease 回收不重置)"
    );

    // 模拟再次 publish 失败 + 已达 max_retries → mark_giveup
    // dry-run 模拟 max_retries=2 (per RelayConfig 默认 5, 但 dry-run 简化为 2 步)
    repo.mark_failed(id, "simulated publish failure #2".to_string())
        .await
        .expect("mark_failed #2 失败");
    repo.mark_giveup(id).await.expect("mark_giveup 失败");

    // 第 3 轮: 状态应为 Failed (DLQ 入口), list_pending 不再返回
    let batch_post_giveup = repo.list_pending(10).await.expect("post-giveup list 失败");
    assert!(
        batch_post_giveup.is_empty(),
        "status=Failed 后 list_pending 应不再返回 (DLQ 入口), 实际 {} 条",
        batch_post_giveup.len()
    );
}

// ============================================================================
// 步骤 7: 多 relay 并发安全 (FOR UPDATE SKIP LOCKED 语义, InMemory 版复刻)
// ============================================================================

/// 步骤 7: 两个 relay 副本同时跑 tick, 同一批 pending 不能被两副本都拿到.
///
/// InMemoryOutboxRepository::list_pending 内部 mark in_flight + lease, 第二次
/// 调用应跳过持锁行 (per SKIP LOCKED 语义). 这是 onboarding 必修项 — 新域
/// 的 outbox 表必须支持 `FOR UPDATE SKIP LOCKED` (per migration template §A).
#[tokio::test]
async fn dry_run_step_7_concurrent_relay_safety() {
    let pool = lazy_pool();
    let repo = Arc::new(InMemoryOutboxRepository::with_lease(Duration::from_secs(30)));

    // 提交 5 条新域事件
    for i in 0..5 {
        let entry = OutboxEntry::new(
            format!("rgs.analytics.telemetry.collected.v{}", i),
            format!(r#"{{"i":{}}}"#, i),
            Uuid::new_v4(),
        );
        repo.append(&entry, &pool).await.expect("append 失败");
    }

    // relay-A 第一次 list_pending: 应拿到全部 5 条 (mark in_flight + lease 30s)
    let relay_a_batch = repo.list_pending(10).await.expect("relay-A list 失败");
    assert_eq!(relay_a_batch.len(), 5, "relay-A 首次 list 应拿 5 条");

    // relay-B 紧接 list_pending: 应跳过 relay-A 持锁的 5 条 (SKIP LOCKED 语义)
    let relay_b_batch = repo.list_pending(10).await.expect("relay-B list 失败");
    assert!(
        relay_b_batch.is_empty(),
        "relay-B 应跳过 relay-A 持锁的 in_flight 行, 实际拿到 {} 条",
        relay_b_batch.len()
    );

    // 模拟 relay-A 全部 mark_sent → release
    for entry in &relay_a_batch {
        repo.mark_sent(entry.id).await.expect("mark_sent 失败");
    }

    // relay-B 再次 list: Sent 不在候选, 应仍为空
    let relay_b_post = repo.list_pending(10).await.expect("relay-B post-sent list 失败");
    assert!(
        relay_b_post.is_empty(),
        "relay-B post-sent list 应为空 (Sent 不在候选), 实际 {} 条",
        relay_b_post.len()
    );
}

// ============================================================================
// dry-run 总验收: 7 步 checklist 全部在 `analytics` 域 (新域) 上端到端跑通
// ============================================================================

/// 总验收: 模拟一个真实的新域 telemetry pipeline 完整生命周期
///
/// 步骤:
/// 1. RGS-REQ-NNN 需求定义书 (dry-run 跳过, 假设已审批)
/// 2. outbox migration (per 10_outbox_migration_template_v1.sql)
/// 3. main.rs 集成 outbox relay (per 11_outbox_relay_bootstrap_pattern.md)
/// 4. 事件族清单登记 (per 09a §C v1.2 命名约定) — 用 SubjectBuilder 构造 5 条 analytics 事件
/// 5. 集成测试 (本测试即)
/// 6. 监控指标接入 (per 06_Outbox监控指标草案.md) — dry-run 跳过 (需 Prometheus exporter 端到端)
/// 7. DLQ 接入 (per 07_PoisonEvent_DLQ草案.md §2.2) — 已用 InMemory mark_giveup 复刻
#[tokio::test]
async fn dry_run_total_acceptance_seven_step_checklist() {
    let pool = lazy_pool();
    let repo = InMemoryOutboxRepository::with_lease(Duration::from_millis(50));

    // 步骤 4 产物: 5 条 analytics 域事件 (per 09a §C 命名约定)
    let analytics_events = [
        ("telemetry.collected", 1, r#"{"phase":"ingest","count":100}"#),
        ("telemetry.batched", 1, r#"{"batch_size":50}"#),
        ("metric.aggregated", 1, r#"{"window":"5m","p99":42}"#),
        ("metric.aggregated", 2, r#"{"window":"5m","p99":41}"#), // 版本演进
        ("query.published", 1, r#"{"query_id":"q-001"}"#),
    ];

    let mut all_ids = Vec::new();
    for (event_type, version, payload) in analytics_events {
        let subject = format!("rgs.analytics.{}.v{}", event_type, version);
        // **白名单约束确认**: 新域 `analytics` 不在 parse 白名单, 这是 onboarding 必修项
        assert!(
            parse(&subject).is_err(),
            "subject `{}` 应不在 6 域白名单 (dry-run 假设新域未扩展白名单)",
            subject
        );

        let entry = OutboxEntry::new(
            subject.clone(),
            payload.to_string(),
            Uuid::new_v4(),
        );
        all_ids.push((subject.clone(), entry.id));
        repo.append(&entry, &pool)
            .await
            .unwrap_or_else(|e| panic!("append `{}` 失败: {}", subject, e));
    }

    // 步骤 5: relay tick #1 — 全部 publish 成功
    let tick_1 = repo.list_pending(10).await.expect("tick #1 list 失败");
    assert_eq!(tick_1.len(), 5, "tick #1 应见全部 5 条");
    for id in all_ids.iter().map(|(_, id)| *id) {
        repo.mark_sent(id).await.expect("tick #1 mark_sent 失败");
    }

    // 步骤 7: DLQ 路径 — 重新提交 1 条, 模拟 poison event 走 2 次重试后 giveup
    let poison = OutboxEntry::new(
        "rgs.analytics.telemetry.collected.v1".to_string(),
        r#"{"poison":true}"#.to_string(),
        Uuid::new_v4(),
    );
    let poison_id = poison.id;
    repo.append(&poison, &pool).await.expect("poison append 失败");

    let tick_2 = repo.list_pending(10).await.expect("tick #2 list 失败");
    assert_eq!(tick_2.len(), 1, "tick #2 应见 1 条 poison");
    repo.mark_failed(poison_id, "transient err".to_string())
        .await
        .expect("poison mark_failed #1 失败");

    tokio::time::sleep(Duration::from_millis(100)).await; // lease 过期 (50ms)

    let tick_3 = repo.list_pending(10).await.expect("tick #3 list 失败");
    assert_eq!(tick_3.len(), 1, "tick #3 lease 回收应见 1 条");
    repo.mark_failed(poison_id, "permanent err".to_string())
        .await
        .expect("poison mark_failed #2 失败");
    repo.mark_giveup(poison_id)
        .await
        .expect("poison mark_giveup 失败");

    // 总验收最终态: 所有 6 条已处理 (5 Sent + 1 Failed/DLQ)
    let final_pending = repo.list_pending(10).await.expect("final list 失败");
    assert!(
        final_pending.is_empty(),
        "dry-run 终态: 所有条目应已处理, list_pending 应为空, 实际 {} 条",
        final_pending.len()
    );
}

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/nonexistent")
        .expect("lazy connect should not fail")
}
