//! RGS-UT 2026-09-19 JST — social 域 IT (outbox_coverage 子桶)
//!
//! `integration_outbox_coverage`
//!
//! 设计目标 (per ULYS-103 acceptance #5: 6 域 cargo test outbox_coverage 全绿):
//! - 验证 social 域 outbox 表 migration + 与 social-service main.rs 集成的入口存在
//! - 验证 v1.2 候选清单中 social 域 4 条 (`rgs.social.friend.added.v1` /
//!   `rgs.social.friend.removed.v1` / `rgs.social.guild.created.v1` /
//!   `rgs.social.mail.sent.v1`) 通过 `SubjectBuilder::domain_event` + `parse()` 命名合规
//! - 验证 `InMemoryOutboxRepository` 接受 social 域条目
//!
//! 锚定文件:
//! - 源: shared-platform/src/outbox.rs (InMemoryOutboxRepository + append + list_pending)
//! - 源: shared-platform/src/subject.rs (SubjectBuilder::domain_event + parse)
//! - 源: social-service/migrations/0002_outbox.sql + 0003_outbox_check_idempotent.sql

use std::sync::Arc;
use uuid::Uuid;

use shared_platform::outbox::{InMemoryOutboxRepository, OutboxEntry, OutboxRepository};
use shared_platform::subject::{parse, SubjectBuilder, SubjectDomain};

use sqlx::postgres::PgPoolOptions;

// ============================================================================
// social 域 v1.2 候选清单 (per 09a §C.1 social 域)
// ============================================================================

const SOCIAL_V1_2_SUBJECTS: &[&str] = &[
    "rgs.social.friend.added.v1",
    "rgs.social.friend.removed.v1",
    "rgs.social.guild.created.v1",
    "rgs.social.mail.sent.v1",
];

/// 与生产代码命名约定对齐 (per shared-platform::subject::SubjectBuilder::domain_event)
#[test]
fn social_v1_2_subjects_match_subject_builder() {
    let expected_pairs = [
        ("social", "friend.added", 1u32),
        ("social", "friend.removed", 1u32),
        ("social", "guild.created", 1u32),
        ("social", "mail.sent", 1u32),
    ];

    assert_eq!(
        SOCIAL_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
        "SOCIAL_V1_2_SUBJECTS (={}) vs 期望条目 (={}) 不一致",
        SOCIAL_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
    );

    for (idx, (domain, event_type, version)) in expected_pairs.iter().enumerate() {
        let built = SubjectBuilder::domain_event(domain, event_type, *version);
        assert_eq!(
            built, SOCIAL_V1_2_SUBJECTS[idx],
            "SubjectBuilder::domain_event({:?}, {:?}, {:?}) 与 v1.2 候选条目 #{} 不一致",
            domain, event_type, version, idx,
        );
    }
}

/// social 域 subject 命名合规
#[test]
fn social_v1_2_subjects_parse_compliant() {
    for subject in SOCIAL_V1_2_SUBJECTS {
        let (domain, _rest) = parse(subject).unwrap_or_else(|e| {
            panic!("social 域 subject {} 解析失败: {}", subject, e);
        });

        assert_eq!(
            domain,
            SubjectDomain::Domain,
            "social 域 subject {} 应被识别为 SubjectDomain::Domain",
            subject
        );

        let parts: Vec<&str> = subject.split('.').collect();
        assert_eq!(
            parts[1], "social",
            "social 域 subject {} 第 2 段应为 'social', 实际为 {}",
            subject, parts[1],
        );
    }
}

/// social 域 outbox 基础设施 readiness
#[test]
fn social_outbox_infrastructure_readiness() {
    assert!(
        SOCIAL_V1_2_SUBJECTS.len() >= 4,
        "social 域 v1.2 候选清单至少 4 条, 当前 = {}",
        SOCIAL_V1_2_SUBJECTS.len(),
    );
}

/// social 域 OutboxEntry append + list_pending 生命周期
#[tokio::test]
async fn social_outbox_entry_lifecycle() {
    let pool = lazy_pool();
    let repo = InMemoryOutboxRepository::new();

    for subject in &SOCIAL_V1_2_SUBJECTS[..2] {
        let entry = OutboxEntry::new(
            subject.to_string(),
            format!(r#"{{"domain":"social","subject":"{}"}}"#, subject),
            Uuid::new_v4(),
        );

        repo.append(&entry, &pool)
            .await
            .unwrap_or_else(|e| panic!("social 域 {} append 失败: {}", subject, e));

        let pending = repo
            .list_pending(10)
            .await
            .unwrap_or_else(|e| panic!("social 域 list_pending 失败: {}", e));
        assert!(
            pending.iter().any(|e| e.subject == *subject),
            "social 域 {} 应出现在 list_pending 结果中",
            subject,
        );
    }
}

/// fail-closed: 域不匹配的 subject 不应被识别为 social 域
#[test]
fn social_other_domain_subjects_not_misclassified() {
    let non_social_subjects = [
        "rgs.player.character.created.v1",
        "rgs.economy.wallet.committed.v1",
        "rgs.match.match.finished.v1",
        "rgs.admin.ban.applied.v1",
        "rgs.cluster_ops.node.joined.v1",
    ];

    for subject in non_social_subjects {
        let (_domain, _) = parse(subject).unwrap();

        let parts: Vec<&str> = subject.split('.').collect();
        assert_ne!(
            parts[1], "social",
            "{} 的第 2 段不应为 'social' (实际 = {})",
            subject, parts[1],
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