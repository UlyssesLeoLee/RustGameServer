//! `economy-service` mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_economy_demo`
//!
//! 演示:
//! - `EconomyFixture::economy(player_id)` 创建 sample balance
//! - `NatsMock::InMemoryNatsMock` 模拟 outbox relay subject
//! - `FixtureBuilder` 链式构造带自定义 currency/gold 的 balance
//!
//! 关联: `RGS-DTL-018_经济域_详细设计书.md` §3 (OCC) + §4 (outbox)
//!       + `RGS-TST-UT-04_经济域_单元测试设计书.md` (per F3 衍生补, 待新建)

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== economy-service mock + fixture 示例 ===\n");

    // 1. EconomyFixture
    println!("[1] EconomyFixture::economy(\"alice\")");
    let bal = fixture::economy("alice");
    println!("    player_id={}, currency={}, gold={}\n", bal.player_id, bal.currency, bal.gold);

    // 2. FixtureBuilder 链式构造
    println!("[2] FixtureBuilder::new(bal).with_currency(5000).with_gold(100).build()");
    let custom = FixtureBuilder::new(bal.clone())
        .with_currency(5000)
        .with_gold(100)
        .build();
    println!("    player_id={}, currency={}, gold={}\n", custom.player_id, custom.currency, custom.gold);

    // 3. InMemoryNatsMock 模拟 outbox relay
    println!("[3] InMemoryNatsMock 模拟 outbox relay 事件");
    let nats = InMemoryNatsMock::new();
    nats.publish("economy.outbox.balance_changed", br#"{"player_id":"alice","delta":100}"#).await.unwrap();
    nats.publish("economy.outbox.balance_changed", br#"{"player_id":"alice","delta":-50}"#).await.unwrap();
    let msgs = nats.subscribe("economy.outbox.balance_changed").await.expect("subscribe ok");
    println!("    outbox received_count={} (期望 2)\n", nats.received_count("economy.outbox.balance_changed"));
    println!("    payload[0]={}", String::from_utf8_lossy(&msgs[0]));
    println!("    payload[1]={}", String::from_utf8_lossy(&msgs[1]));

    println!("\n=== 完成 ===");
}
