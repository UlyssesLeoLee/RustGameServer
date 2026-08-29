//! `player-service` mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_player_demo`
//!
//! 演示:
//! - `PlayerFixture::player()` 创建 sample player
//! - `FixtureBuilder` 链式构造带 name/level 的 player
//! - `InMemoryNatsMock` 模拟 player.events subject
//!
//! 关联: `RGS-DTL-015_玩家域_详细设计书.md` §3 (账号/角色) + §4 (会话)
//!       + `RGS-TST-UT-02_玩家域_单元测试设计书.md` (per F3 衍生补, 待新建)

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== player-service mock + fixture 示例 ===\n");

    // 1. PlayerFixture
    println!("[1] PlayerFixture::player()");
    let p = fixture::player();
    println!("    id={}, name={}, level={}\n", p.id, p.name, p.level);

    // 2. FixtureBuilder 链式构造
    println!("[2] FixtureBuilder::new(p).with_name(\"Alice\").with_level(42).build()");
    let custom = FixtureBuilder::new(p.clone())
        .with_name("Alice")
        .with_level(42)
        .build();
    println!(
        "    id={}, name={}, level={}\n",
        custom.id, custom.name, custom.level
    );

    // 3. InMemoryNatsMock 模拟 player.events
    println!("[3] InMemoryNatsMock 模拟 player.events");
    let nats = InMemoryNatsMock::new();
    nats.publish("player.events", br#"{"event":"login","player_id":"p1"}"#)
        .await
        .unwrap();
    nats.publish(
        "player.events",
        br#"{"event":"level_up","player_id":"p1","level":2}"#,
    )
    .await
    .unwrap();
    nats.publish("player.events", br#"{"event":"logout","player_id":"p1"}"#)
        .await
        .unwrap();
    let count = nats.received_count("player.events");
    println!("    player.events received_count={} (期望 3)\n", count);

    println!("=== 完成 ===");
}
