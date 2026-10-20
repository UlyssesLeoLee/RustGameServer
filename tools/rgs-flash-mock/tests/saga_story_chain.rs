//! 跨域故事链 + 跨域事件链集成测试 (rgs-flash-mock)
//!
//! 覆盖 `tools/rgs-flash-mock/README.md` 已知缺口 #1 (22 RPC stub 缺 E2E 故事链)
//! 与 #2 (saga_abstract 仅 mock, 缺 cross-domain event verification)。
//!
//! 判据: 断言「错误的类别」时, 失败路径必须构造得**唯一且确定** —— 这里全部用
//! 显式注入的期望值偏差驱动, 不用随机数 / sleep / 并发, 因此每次运行结果一致。

use rgs_flash_mock::gap_matrix::{GapMatrix, RpcCategory, RpcStatus};
use rgs_flash_mock::saga_verify::{CrossDomainEvent, StoryChain, StoryStep};

#[test]
fn default_chain_is_green() {
    let report = StoryChain::cross_domain_default().verify(&GapMatrix::new());
    assert!(
        report.is_ok(),
        "默认故事链应全绿, 实际: {:?}",
        report
    );
    assert_eq!(report.passed, 5);
    assert_eq!(report.failed, 0);
    assert!(report.dangling_events.is_empty());
    assert!(report.out_of_order_events.is_empty());
}

#[test]
fn default_chain_covers_at_least_three_domains() {
    let chain = StoryChain::cross_domain_default();
    let mut domains: Vec<&str> = chain.steps.iter().map(|s| s.domain.as_str()).collect();
    domains.sort_unstable();
    domains.dedup();
    assert!(domains.len() >= 3, "故事链应覆盖 >= 3 个域");
}

#[test]
fn status_mismatch_names_the_offending_step() {
    let mut chain = StoryChain::cross_domain_default();
    let step = chain
        .steps
        .iter_mut()
        .find(|s| s.name == "match.start")
        .expect("默认链应含 match.start");
    // 实际状态是 Pass, 这里故意期望 Partial
    step.expected_status = RpcStatus::Partial;

    let report = chain.verify(&GapMatrix::new());

    assert_eq!(report.failed, 1, "只应有 1 步失败");
    assert_eq!(
        report.failed_rpc_codes(),
        vec![301],
        "失败应定位到 rpc 301 (StartCombat)"
    );
    let first_failure = report
        .first_failure
        .clone()
        .expect("失败报告必须给出首个失败步");
    assert!(
        first_failure.contains("match.start"),
        "失败定位应指向具体步骤, 实际: {}",
        first_failure
    );
    assert!(!report.is_ok());
}

#[test]
fn unknown_rpc_code_is_reported() {
    let chain = StoryChain::new(vec![StoryStep::new(
        "ghost.step",
        "player",
        99999,
        "GhostRpc",
        RpcCategory::Misc,
        RpcStatus::Pass,
    )]);
    let report = chain.verify(&GapMatrix::new());
    assert_eq!(report.failed, 1);
    let first_failure = report
        .first_failure
        .clone()
        .expect("必须给出失败原因");
    assert!(
        first_failure.contains("gap_matrix 无 rpc_code 99999"),
        "实际: {}",
        first_failure
    );
}

#[test]
fn dangling_event_is_detected() {
    let mut chain = StoryChain::cross_domain_default();
    // 末步额外产生一个无人消费的事件
    chain
        .steps
        .last_mut()
        .expect("默认链非空")
        .emits
        .push(CrossDomainEvent::new(
            "admin",
            "admin.audit.orphan",
            "player",
        ));

    let report = chain.verify(&GapMatrix::new());

    assert_eq!(
        report.dangling_events.len(),
        1,
        "应恰好检出 1 个悬空事件: {:?}",
        report.dangling_events
    );
    assert!(report.dangling_events[0].contains("admin.audit.orphan"));
    assert!(!report.is_ok());
}

#[test]
fn out_of_order_event_is_detected() {
    let mut chain = StoryChain::cross_domain_default();
    // 第 1 步去消费第 4 步才产生的事件 → 顺序错乱
    chain
        .steps
        .first_mut()
        .expect("默认链非空")
        .consumes
        .push("social.message.delivered".to_string());

    let report = chain.verify(&GapMatrix::new());

    assert_eq!(
        report.out_of_order_events.len(),
        1,
        "应恰好检出 1 个顺序错乱事件: {:?}",
        report.out_of_order_events
    );
    assert!(report.out_of_order_events[0].contains("social.message.delivered"));
    assert!(!report.is_ok());
}

#[test]
fn event_consumed_by_wrong_domain_is_detected() {
    let chain = StoryChain::new(vec![
        StoryStep::new(
            "producer",
            "player",
            201,
            "GetPlayerProfile",
            RpcCategory::Role,
            RpcStatus::Partial,
        )
        .emitting(CrossDomainEvent::new("player", "evt.cross", "economy")),
        StoryStep::new(
            "wrong-consumer",
            "match",
            301,
            "StartCombat",
            RpcCategory::Combat,
            RpcStatus::Pass,
        )
        .consuming("evt.cross"),
    ]);
    let report = chain.verify(&GapMatrix::new());
    assert!(
        !report.dangling_events.is_empty(),
        "域归属不符应被检出: {:?}",
        report
    );
}
