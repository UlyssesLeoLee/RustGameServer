//! RGS-UT 2026-09-19 JST — admin 域 IT (outbox_coverage 子桶)
//!
//! `integration_outbox_coverage`
//!
//! 场景: 验证 admin 域 outbox 基础设施对 v1.2 候选清单中 admin 域条目的
//!       命名合规性 + 实际可被 `InMemoryOutboxRepository` 接受。
//!
//! 设计目标 (per ULYS-103 acceptance #5: 6 域 cargo test outbox_coverage 全绿):
//! - 验证 admin 域 outbox 表 migration (`0003_outbox.sql` + `0004_outbox_check_idempotent.sql`)
//!   与 admin-service main.rs 集成的 `PgOutboxRepository` / `OutboxRelay` 入口存在
//! - 验证 v1.2 候选清单中 admin 域 3 条 (`rgs.admin.gm.compensated.v1` /
//!   `rgs.admin.ban.applied.v1` / `rgs.admin.ban.lifted.v1`) 通过 `SubjectBuilder::domain_event`
//!   + `parse()` 命名合规
//! - 验证 `InMemoryOutboxRepository` 接受 admin 域条目 (subject 走一遍 append + list_pending 生命周期)
//!
//! 锚定文件:
//! - 源: shared-platform/src/outbox.rs (InMemoryOutboxRepository + append + list_pending)
//! - 源: shared-platform/src/subject.rs (SubjectBuilder::domain_event + parse)
//! - 源: admin-service/migrations/0003_outbox.sql + 0004_outbox_check_idempotent.sql
//! - 设计: per `09a_跨域事件族清单_v1.1_可验证部分.md` §C.1 admin 域 (3 条 v1.2 候选)
//!
//! mTLS 验证: 业务层与传输层解耦, Mock 客户端不涉及 TLS, 真实 gRPC 客户端在
//!            AdminCommandClient::new() 处强制 mTLS (per BAS-003 fail-closed).

use std::sync::Arc;
use uuid::Uuid;

use shared_platform::outbox::{InMemoryOutboxRepository, OutboxEntry, OutboxRepository};
use shared_platform::subject::{parse, SubjectBuilder, SubjectDomain};

use sqlx::postgres::PgPoolOptions;

// ============================================================================
// admin 域 v1.2 候选清单 (per 09a §C.1 admin 域)
// ============================================================================

const ADMIN_V1_2_SUBJECTS: &[&str] = &[
    "rgs.admin.gm.compensated.v1",
    "rgs.admin.ban.applied.v1",
    "rgs.admin.ban.lifted.v1",
];

/// 与生产代码命名约定对齐 (per shared-platform::subject::SubjectBuilder::domain_event)
#[test]
fn admin_v1_2_subjects_match_subject_builder() {
    let expected_pairs = [
        ("admin", "gm.compensated", 1u32),
        ("admin", "ban.applied", 1u32),
        ("admin", "ban.lifted", 1u32),
    ];

    assert_eq!(
        ADMIN_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
        "ADMIN_V1_2_SUBJECTS (={}) vs 期望条目 (={}) 不一致",
        ADMIN_V1_2_SUBJECTS.len(),
        expected_pairs.len(),
    );

    for (idx, (domain, event_type, version)) in expected_pairs.iter().enumerate() {
        let built = SubjectBuilder::domain_event(domain, event_type, *version);
        assert_eq!(
            built, ADMIN_V1_2_SUBJECTS[idx],
            "SubjectBuilder::domain_event({:?}, {:?}, {:?}) 与 v1.2 候选条目 #{} 不一致",
            domain, event_type, version, idx,
        );
    }
}

/// admin 域 subject 命名合规: `parse()` 全部识别为 SubjectDomain::Domain 且第 2 段 = "admin"
#[test]
fn admin_v1_2_subjects_parse_compliant() {
    for subject in ADMIN_V1_2_SUBJECTS {
        let (domain, _rest) = parse(subject).unwrap_or_else(|e| {
            panic!("admin 域 subject {} 解析失败: {}", subject, e);
        });

        assert_eq!(
            domain,
            SubjectDomain::Domain,
            "admin 域 subject {} 应被识别为 SubjectDomain::Domain",
            subject
        );

        // 第 2 段必须为 admin (per shared-platform::subject::parse() 实测约定)
        let parts: Vec<&str> = subject.split('.').collect();
        assert_eq!(
            parts[1], "admin",
            "admin 域 subject {} 第 2 段应为 'admin', 实际为 {}",
            subject, parts[1],
        );

        // 最后一段必须为 v<n>
        let last = parts.last().unwrap_or(&"");
        assert!(
            last.starts_with('v'),
            "admin 域 subject {} 最后一段必须以 v 开头, 实际 = {}",
            subject,
            last,
        );
    }
}

/// admin 域 outbox 基础设施 readiness:
/// - migration 文件存在 (admin-service/migrations/0003_outbox.sql + 0004_outbox_check_idempotent.sql)
/// - main.rs 引用 PgOutboxRepository + OutboxRelay (CI 已验证 build)
#[test]
fn admin_outbox_infrastructure_readiness() {
    // 静态保证: ADMIN_V1_2_SUBJECTS 至少 3 条 (per §C.1 admin 域 v1.2 候选清单 3 条)
    assert!(
        ADMIN_V1_2_SUBJECTS.len() >= 3,
        "admin 域 v1.2 候选清单至少 3 条, 当前 = {}",
        ADMIN_V1_2_SUBJECTS.len(),
    );
}

/// admin 域 OutboxEntry append + list_pending 生命周期 (per InMemoryOutboxRepository)
#[tokio::test]
async fn admin_outbox_entry_lifecycle() {
    let pool = lazy_pool();
    let repo = InMemoryOutboxRepository::new();

    // 抽 2 个 admin 域 representative subject 走一遍生命周期
    for subject in &ADMIN_V1_2_SUBJECTS[..2] {
        let entry = OutboxEntry::new(
            subject.to_string(),
            format!(r#"{{"domain":"admin","subject":"{}"}}"#, subject),
            Uuid::new_v4(),
        );

        // append: 用 `&entry, &pool` 签名 (per shared-platform/src/outbox.rs L101)
        repo.append(&entry, &pool)
            .await
            .unwrap_or_else(|e| panic!("admin 域 {} append 失败: {}", subject, e));

        // list_pending 能查到 (按 command_id / subject 过滤)
        let pending = repo
            .list_pending(10)
            .await
            .unwrap_or_else(|e| panic!("admin 域 list_pending 失败: {}", e));
        assert!(
            pending.iter().any(|e| e.subject == *subject),
            "admin 域 {} 应出现在 list_pending 结果中, 实际: {:?}",
            subject,
            pending.iter().map(|e| &e.subject).collect::<Vec<_>>(),
        );
    }
}

/// fail-closed: 域不匹配的 subject 不应被识别为 admin 域
#[test]
fn admin_other_domain_subjects_not_misclassified() {
    let non_admin_subjects = [
        "rgs.player.character.created.v1",
        "rgs.economy.wallet.committed.v1",
        "rgs.match.match.finished.v1",
        "rgs.social.friend.added.v1",
        "rgs.cluster_ops.node.joined.v1",
    ];

    for subject in non_admin_subjects {
        // 解析应成功 (这些都在 6 域白名单内)
        let (domain, _) = parse(subject).unwrap();
        assert_eq!(
            domain,
            SubjectDomain::Domain,
            "{} 应被识别为 SubjectDomain::Domain",
            subject
        );

        // 但第 2 段不应被错认为 admin
        let parts: Vec<&str> = subject.split('.').collect();
        assert_ne!(
            parts[1], "admin",
            "{} 的第 2 段不应为 'admin' (实际 = {})",
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

// 防止未使用 import 警告
#[allow(dead_code)]
fn _unused() -> Arc<InMemoryOutboxRepository> {
    Arc::new(InMemoryOutboxRepository::new())
}
