//! `NatsMock` trait 完整使用示例 (53.3 占位形态)
//!
//! Run: `cargo run -p rgs-testkit --example mock_nats_demo`
//!
//! 54.x 接入计划 (per §3.2):
//! - `InMemoryNatsMock` (in-memory subject store) —— `publish()` / `subscribe()`
//! - 5 域 event bus 单元/集成 test (player.events / economy.tx / match.score 等)
//!
//! 当前 53.3 形态: `NatsMock` trait 仅 `publish()` 占位方法 + `NoopMock` 空实现.
//! 本 example 演示 trait 当前形态 + 54.x 演进方向.
//!
//! 注: 故意允许 deprecation 警告以演示 53.3 NoopMock 占位形态,
//! 与 `test_deprecated_warns.rs` 一致. 业务 test 禁止用 NoopMock 替代真 PG.

#![allow(deprecated)]

use rgs_testkit::mock::{NatsMock, NoopMock};

#[tokio::main]
async fn main() {
    println!("=== NatsMock 完整使用示例 (53.3 形态) ===\n");

    // 1. 53.3 当前 API: NoopMock 实现 NatsMock::publish() 占位
    let nats = NoopMock;
    println!("[1] NatsMock::publish() (53.3 占位, 返回 Ok)");
    let result = nats.publish("player.events", br#"{"event":"login"}"#).await;
    println!("    publish result = {:?}\n", result);

    // 2. 54.x 计划 API (待接入, 编译期不可用)
    println!("[2] 54.x 计划 API (尚未接入):");
    println!("    use rgs_testkit::mock::{{NatsMock, InMemoryNatsMock}};");
    println!("    let nats = InMemoryNatsMock::new();");
    println!("    nats.publish(\"player.events\", br#\"{{\"event\":\"login\"}}\"#).await.unwrap();");
    println!("    nats.publish(\"player.events\", br#\"{{\"event\":\"logout\"}}\"#).await.unwrap();");
    println!("    let msgs = nats.subscribe(\"player.events\").await.unwrap();");
    println!("    assert_eq!(msgs.len(), 2);");
    println!();

    // 3. 5 域 NATS subject 规划
    println!("[3] 5 域 NATS subject 规划 (54.x 接入后):");
    println!("    player.events    - 玩家登录/登出/升级");
    println!("    economy.tx       - 经济交易/转账");
    println!("    match.score      - 比赛得分/状态变更");
    println!("    social.notify    - 社交通知/消息");
    println!("    admin.audit      - 审计日志/权限变更");

    println!("\n=== Demo complete ===");
}
