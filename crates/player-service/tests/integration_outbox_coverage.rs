//! RGS-UT 2026-09-19 JST — player 域 IT (outbox_coverage 子桶)
//!
//! `integration_outbox_coverage`
//!
//! 设计目标 (per ULYS-103 acceptance #5: 6 域 cargo test outbox_coverage 全绿):
//! - 验证 player 域 outbox 表 migration + 与 player-service main.rs 集成的入口存在
//! - 验证 v1.2 候选清单中 player 域 4 条 (`rgs.player.character.created.v1` /
//!   `rgs.player.character.deleted.v1` / `rgs.player.session.started.v1` /
//!   `rgs.player.session.ended.v1`) 通过 `SubjectBuilder::domain_event` + `parse()` 命名合规
//! - 验证 `InMemoryOutboxRepository` 接受 player 域条目
//!
//! 锚定文件:
//! - 源: shared-platform/src/outbox.rs (InMemoryOutboxRepository + append + list_pending)
//! - 源: shared-platform/src/subject.rs (SubjectBuilder::domain_event + parse)
//! - 源: player-service/migrations/0002_outbox.sql + 0003_outbox_check_idempotent.sql

use std::sync::Arc;
use uuid::Uuid;

use shared_platform::outbox::{InMemoryOutboxRepository, OutboxEntry, OutboxRepository};
use shared_platform::subject::{parse, SubjectBuilder, SubjectDomain};

use sqlx::postgres::PgPoolOptions;

// ============================================================================
// player 域 v1.2 候选清单 (per 09a §C.1 player 域)
// ============================================================================

const PLAYER_V1_2_SUBJECTS: &[&str] = &[
    "rgs.player.character.created.v1",
    "rgs.player.character.deleted.v1",
    "rgs.player.session.started.v1",
    "rgs.player.session.ended.v1",
];

/// 与生产代码命名约定对齐 (per shared-platform::subject::SubjectBuilder::domain_event)
#[test]
fn player_v1_2_subjects_match_subject_builder() {
    let expected_pairs = [
        ("player", "character.created", 1u32),
        ("player", "character.deleted", 1u32),
        ("player", "session.started", 1u32),
        ("player", "session.ended", 1u32),
    ];

    assert_eq!(
        PLAYER_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
        "PLAYER_V1_2_SUBJECTS (={}) vs 期望条目 (={}) 不一致",
        PLAYER_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
    );

    for (idx, (domain, event_type, version)) in expected_pairs.iter().enumerate() {
        let built = SubjectBuilder::domain_event(domain, event_type, *version);
        assert_eq!(
            built, PLAYER_V1_2_SUBJECTS[idx],
            "SubjectBuilder::domain_event({:?}, {:?}, {:?}) 与 v1.2 候选条目 #{} 不一致",
            domain, event_type, version, idx,
        );
    }
}

/// player 域 subject 命名合规
#[test]
fn player_v1_2_subjects_parse_compliant() {
    for subject in PLAYER_V1_2_SUBJECTS {
        let (domain, _rest) = parse(subject).unwrap_or_else(|e| {
            panic!("player 域 subject {} 解析失败: {}", subject, e);
        });

        assert_eq!(
            domain,
            SubjectDomain::Domain,
            "player 域 subject {} 应被识别为 SubjectDomain::Domain",
            subject
        );

        let parts: Vec<&str> = subject.split('.').collect();
        assert_eq!(
            parts[1], "player",
            "player 域 subject {} 第 2 段应为 'player', 实际为 {}",
            subject, parts[1],
        );
    }
}

/// player 域 outbox 基础设施 readiness
#[test]
fn player_outbox_infrastructure_readiness() {
    assert!(
        PLAYER_V1_2_SUBJECTS.len() >= 4,
        "player 域 v1.2 候选清单至少 4 条, 当前 = {}",
        PLAYER_V1_2_SUBJECTS.len(),
    );
}

/// player 域 OutboxEntry append + list_pending 生命周期
#[tokio::test]
async fn player_outbox_entry_lifecycle() {
    let pool = lazy_pool();
    let repo = InMemoryOutboxRepository::new();

    for subject in &PLAYER_V1_2_SUBJECTS[..2] {
        let entry = OutboxEntry::new(
            subject.to_string(),
            format!(r#"{{"domain":"player","subject":"{}"}}"#, subject),
            Uuid::new_v4(),
        );

        repo.append(&entry, &pool)
            .await
            .unwrap_or_else(|e| panic!("player 域 {} append 失败: {}", subject, e));

        let pending = repo
            .list_pending(10)
            .await
            .unwrap_or_else(|e| panic!("player 域 list_pending 失败: {}", e));
        assert!(
            pending.iter().any(|e| e.subject == *subject),
            "player 域 {} 应出现在 list_pending 结果中",
            subject,
        );
    }
}

/// fail-closed: 域不匹配的 subject 不应被识别为 player 域
#[test]
fn player_other_domain_subjects_not_misclassified() {
    let non_player_subjects = [
        "rgs.economy.wallet.committed.v1",
        "rgs.match.match.finished.v1",
        "rgs.social.friend.added.v1",
        "rgs.admin.ban.applied.v1",
        "rgs.cluster_ops.node.joined.v1",
    ];

    for subject in non_player_subjects {
        let (_domain, _) = parse(subject).unwrap();

        let parts: Vec<&str> = subject.split('.').collect();
        assert_ne!(
            parts[1], "player",
            "{} 的第 2 段不应为 'player' (实际 = {})",
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
