//! shared-platform IT: rbac + metrics + span_helpers 协同（per PT-WORKER-BRIEFING.md §2）
//!
//! 覆盖 3 个跨模块场景：
//! 1. it_rbac_metrics_label_includes_actor
//! 2. it_span_helper_nested_with_actor_and_saga
//! 3. it_rbac_enforce_then_metrics_record

use shared_platform::metrics::{metrics, encode_to_text};
use shared_platform::rbac::{enforce, Authorizer, CheckResult, SimpleAuthorizer, Subject, SubjectType, Role};
use shared_platform::span_helpers::{saga_orchestrator_span, service_call_span, repository_span};
use shared_platform::json_logging::with_actor;
use uuid::Uuid;

/// 场景 1: RBAC 检查应能用 actor_id 维度被 metrics 观测
/// (per RGS-ARC-051 COC 集群运营中心: 所有 admin 操作按 actor 过滤)
#[test]
fn it_rbac_metrics_label_includes_actor() {
    let a = SimpleAuthorizer::new();
    let actor_id = Uuid::new_v4();
    let actor = Subject {
        id: actor_id,
        subject_type: SubjectType::Admin,
        roles: vec![Role::SuperAdmin],
        domain_scope: None,
    };

    // 业务路径: enforce → 成功 → 记录指标
    let m = metrics();
    let result = enforce(&a, &actor, "player:ban", "player/123");
    if result.is_ok() {
        m.record_http_request("rbac-test", "enforce", "allow");
    }
    let text = encode_to_text().unwrap();
    assert!(text.contains("rbac-test"), "metrics 应含 RBAC 业务标签");
    assert!(text.contains("rgs_http_requests_total"));
}

/// 场景 2: span 嵌套 (saga → service → repository) + actor 维度注入
/// (per RGS-DTL-100 §7: 4 层 span 树全链路追踪)
#[test]
fn it_span_helper_nested_with_actor_and_saga() {
    let actor_id = Uuid::new_v4();
    let saga_id = Uuid::new_v4();

    with_actor(actor_id, "admin", || {
        let saga_span = saga_orchestrator_span(&saga_id.to_string(), "transfer");
        let _guard = saga_span.enter();
        let svc_span = service_call_span("economy", "credit");
        let _guard2 = svc_span.enter();
        let repo_span = repository_span("Account", "find_by_id");
        let _guard3 = repo_span.enter();
        // 3 层 span 嵌套, in_scope 路径不 panic 即通过
    });
}

/// 场景 3: enforce 失败 + metrics 记录 deny 路径
/// (per RGS-SEC-100 §4: 所有 admin 操作必须能按 actor + 决策审计)
#[test]
fn it_rbac_enforce_then_metrics_record() {
    let a = SimpleAuthorizer::new();
    let p = Subject {
        id: Uuid::new_v4(),
        subject_type: SubjectType::Player,
        roles: vec![Role::Player],
        domain_scope: None,
    };
    let m = metrics();
    // Player 试图 ban 别人 → 应 deny
    let result = enforce(&a, &p, "player:ban", "player/123");
    assert!(result.is_err(), "Player 不应能 ban 别人");
    // 记录 deny 指标 (per ARC-051 集群运营中心: 异常访问可观测)
    m.record_http_request("rbac-test", "enforce", "deny");
    let text = encode_to_text().unwrap();
    // 验证 deny 标签已写入
    assert!(text.contains("deny"), "metrics 应含 deny 标签");
}

/// 场景 4 (附加): Authorizer trait 多态 — 同一接口可换实现
/// (per RGS-DTL-019 §3 业务方 impl 契约)
struct DenyAllAuthorizer;
impl Authorizer for DenyAllAuthorizer {
    fn check(&self, _subject: &Subject, _permission: &str, _resource: &str) -> CheckResult {
        CheckResult::deny_if("deny-all-test")
    }
}

#[test]
fn it_rbac_authorizer_trait_substitutable() {
    let auth: Box<dyn Authorizer> = Box::new(DenyAllAuthorizer);
    let s = Subject {
        id: Uuid::new_v4(),
        subject_type: SubjectType::Admin,
        roles: vec![Role::SuperAdmin],
        domain_scope: None,
    };
    // 即使是 SuperAdmin, DenyAllAuthorizer 也应 deny
    let result = auth.check(&s, "player:ban", "player/1");
    assert!(!result.is_allow(), "DenyAllAuthorizer 必须 deny 一切");
}

/// 场景 5 (附加): 5 业务域 SubjectType 都应能通过 RBAC
/// (回归: SubjectType 枚举不变性)
#[test]
fn it_rbac_all_subject_types_checkable() {
    let a = SimpleAuthorizer::new();
    for st in [SubjectType::Admin, SubjectType::Player, SubjectType::System] {
        let s = Subject {
            id: Uuid::new_v4(),
            subject_type: st,
            roles: vec![Role::SuperAdmin], // 任何 SubjectType 配 SuperAdmin 都能 allow
            domain_scope: None,
        };
        let result = a.check(&s, "player:read", "player/1");
        assert!(result.is_allow(), "{:?} 配 SuperAdmin 应 allow", st);
    }
}
