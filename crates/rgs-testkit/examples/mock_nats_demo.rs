//! `InMemoryNatsMock` 完整使用示例 (54.x 实质化版本)
//!
//! Run: `cargo run -p rgs-testkit --example mock_nats_demo`
//!
//! 演示 in-memory subject store, 用于 5 域跨域事件测试 fixture:
//! - `InMemoryNatsMock::new()` 初始化空 store
//! - `nats.publish(subject, payload)` 存消息
//! - `nats.subscribe(subject)` 取出累积消息 (FIFO)
//! - `nats.received_count(subject)` 计数
//!
//! 关联: `RGS-SPEC-000 §2.4` + `RGS-IMPL-001 §3` + ARC-051 (ClusterOps) + DTL-021~025 (跨域事件)

use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== InMemoryNatsMock 完整使用示例 (54.x 实质化) ===\n");

    // 1. 初始化 mock
    let nats = InMemoryNatsMock::new();

    // 2. publish 3 条 player 事件
    println!("[1] nats.publish(\"player.events\", ...) × 3");
    nats.publish("player.events", br#"{"event":"login","player_id":"p1"}"#).await.unwrap();
    nats.publish("player.events", br#"{"event":"level_up","player_id":"p1","level":2}"#).await.unwrap();
    nats.publish("player.events", br#"{"event":"logout","player_id":"p1"}"#).await.unwrap();
    println!("    3 messages published\n");

    // 3. subscribe 取所有 player 事件
    println!("[2] nats.subscribe(\"player.events\") 取累积消息");
    let msgs = nats.subscribe("player.events").await.unwrap();
    println!("    received {} messages:", msgs.len());
    for (i, m) in msgs.iter().enumerate() {
        println!("    [{}] {}", i, String::from_utf8_lossy(m));
    }
    assert_eq!(msgs.len(), 3);

    // 4. 计数
    println!("\n[3] nats.received_count(\"player.events\") = {}", nats.received_count("player.events"));
    println!("    nats.received_count(\"nonexistent\")  = {}", nats.received_count("nonexistent"));
    assert_eq!(nats.received_count("player.events"), 3);
    assert_eq!(nats.received_count("nonexistent"), 0);

    // 5. 5 域 NATS subject 演示
    println!("\n[4] 5 域 NATS subject 实战:");
    println!("    player.events    ← player 域事件 (login/level_up/logout)");
    println!("    economy.tx       ← economy 域交易");
    println!("    match.score      ← match 域得分");
    println!("    social.notify    ← social 域通知");
    println!("    admin.audit      ← admin 域审计");

    // 实战: economy 域转账事件
    println!("\n[5] 实战: economy 域转账事件");
    nats.publish("economy.tx", br#"{"from":"p1","to":"p2","amount":100}"#).await.unwrap();
    nats.publish("economy.tx", br#"{"from":"p2","to":"p3","amount":50}"#).await.unwrap();
    let tx_msgs = nats.subscribe("economy.tx").await.unwrap();
    println!("    economy.tx 收到 {} 笔交易", tx_msgs.len());

    // 实战: match 域得分事件
    nats.publish("match.score", br#"{"match_id":"m1","player_id":"p1","score":100}"#).await.unwrap();
    let score_msgs = nats.subscribe("match.score").await.unwrap();
    println!("    match.score 收到 {} 次得分事件", score_msgs.len());

    println!("\n=== Demo complete ===");
}
