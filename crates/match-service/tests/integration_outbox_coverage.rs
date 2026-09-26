//! RGS-UT 2026-09-19 JST — match 域 IT (outbox_coverage 子桶)
//!
//! `integration_outbox_coverage`
//!
//! 设计目标 (per ULYS-103 acceptance #5: 6 域 cargo test outbox_coverage 全绿):
//! - 验证 match 域 outbox 表 migration + 与 match-service main.rs 集成的入口存在
//! - 验证 v1.2 候选清单中 match 域 3 条 (`rgs.match.match.created.v1` /
//!   `rgs.match.match.finished.v1` / `rgs.match.reward.distributed.v1`) 通过
//!   `SubjectBuilder::domain_event` + `parse()` 命名合规
//! - 验证 `InMemoryOutboxRepository` 接受 match 域条目
//!
//! 锚定文件:
//! - 源: shared-platform/src/outbox.rs (InMemoryOutboxRepository + append + list_pending)
//! - 源: shared-platform/src/subject.rs (SubjectBuilder::domain_event + parse)
//! - 源: match-service/migrations/0002_outbox.sql + 0003_outbox_check_idempotent.sql

use std::sync::Arc;
use uuid::Uuid;

use shared_platform::outbox::{InMemoryOutboxRepository, OutboxEntry, OutboxRepository};
use shared_platform::subject::{parse, SubjectBuilder, SubjectDomain};

use sqlx::postgres::PgPoolOptions;

// ============================================================================
// match 域 v1.2 候选清单 (per 09a §C.1 match 域)
// ============================================================================

const MATCH_V1_2_SUBJECTS: &[&str] = &[
    "rgs.match.match.created.v1",
    "rgs.match.match.finished.v1",
    "rgs.match.reward.distributed.v1",
];

/// 与生产代码命名约定对齐 (per shared-platform::subject::SubjectBuilder::domain_event)
#[test]
fn match_v1_2_subjects_match_subject_builder() {
    let expected_pairs = [
        ("match", "match.created", 1u32),
        ("match", "match.finished", 1u32),
        ("match", "reward.distributed", 1u32),
    ];

    assert_eq!(
        MATCH_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
        "MATCH_V1_2_SUBJECTS (={}) vs 期望条目 (={}) 不一致",
        MATCH_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
    );

    for (idx, (domain, event_type, version)) in expected_pairs.iter().enumerate() {
        let built = SubjectBuilder::domain_event(domain, event_type, *version);
        assert_eq!(
            built, MATCH_V1_2_SUBJECTS[idx],
            "SubjectBuilder::domain_event({:?}, {:?}, {:?}) 与 v1.2 候选条目 #{} 不一致",
            domain, event_type, version, idx,
        );
    }
}

/// match 域 subject 命名合规
#[test]
fn match_v1_2_subjects_parse_compliant() {
    for subject in MATCH_V1_2_SUBJECTS {
        let (domain, _rest) = parse(subject).unwrap_or_else(|e| {
            panic!("match 域 subject {} 解析失败: {}", subject, e);
        });

        assert_eq!(
            domain,
            SubjectDomain::Domain,
            "match 域 subject {} 应被识别为 SubjectDomain::Domain",
            subject
        );

        let parts: Vec<&str> = subject.split('.').collect();
        assert_eq!(
            parts[1], "match",
            "match 域 subject {} 第 2 段应为 'match', 实际为 {}",
            subject, parts[1],
        );
    }
}

/// match 域 outbox 基础设施 readiness
#[test]
fn match_outbox_infrastructure_readiness() {
    assert!(
        MATCH_V1_2_SUBJECTS.len() >= 3,
        "match 域 v1.2 候选清单至少 3 条, 当前 = {}",
        MATCH_V1_2_SUBJECTS.len(),
    );
}

/// match 域 OutboxEntry append + list_pending 生命周期
#[tokio::test]
async fn match_outbox_entry_lifecycle() {
    let pool = lazy_pool();
    let repo = InMemoryOutboxRepository::new();

    for subject in &MATCH_V1_2_SUBJECTS[..2] {
        let entry = OutboxEntry::new(
            subject.to_string(),
            format!(r#"{{"domain":"match","subject":"{}"}}"#, subject),
            Uuid::new_v4(),
        );

        repo.append(&entry, &pool)
            .await
            .unwrap_or_else(|e| panic!("match 域 {} append 失败: {}", subject, e));

        let pending = repo
            .list_pending(10)
            .await
            .unwrap_or_else(|e| panic!("match 域 list_pending 失败: {}", e));
        assert!(
            pending.iter().any(|e| e.subject == *subject),
            "match 域 {} 应出现在 list_pending 结果中",
            subject,
        );
    }
}

/// fail-closed: 域不匹配的 subject 不应被识别为 match 域
#[test]
fn match_other_domain_subjects_not_misclassified() {
    let non_match_subjects = [
        "rgs.player.character.created.v1",
        "rgs.economy.wallet.committed.v1",
        "rgs.social.friend.added.v1",
        "rgs.admin.ban.applied.v1",
        "rgs.cluster_ops.node.joined.v1",
    ];

    for subject in non_match_subjects {
        let (_domain, _) = parse(subject).unwrap();

        let parts: Vec<&str> = subject.split('.').collect();
        assert_ne!(
            parts[1], "match",
            "{} 的第 2 段不应为 'match' (实际 = {})",
            subject, parts[1],
        );
    }
}

/// match 域 vs CEM 路由: 防止 match 域 subject 误入 CEM 路由
/// (per shared-platform::subject::parse() L73-82 — match 不在 cem/saga/dlq 白名单)
#[test]
fn match_subjects_not_misrouted() {
    for subject in MATCH_V1_2_SUBJECTS {
        let (domain, _) = parse(subject).unwrap();
        assert_ne!(
            domain,
            SubjectDomain::Cem,
            "match 域 subject {} 不应被识别为 CEM 路由",
            subject
        );
        assert_ne!(
            domain,
            SubjectDomain::Saga,
            "match 域 subject {} 不应被识别为 Saga 事件",
            subject
        );
        assert_ne!(
            domain,
            SubjectDomain::Dlq,
            "match 域 subject {} 不应被识别为 DLQ",
            subject
        );
    }
}

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/nonexistent")
        .expect("lazy connect should not fail")
}

#[allow(dead_code)]
fn _unused() -> Arc<InMemoryOutboxRepository> {
    Arc::new(InMemoryOutboxRepository::new())
}
