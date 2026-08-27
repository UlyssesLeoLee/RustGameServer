//! `match-service` mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_match_demo`
//!
//! 演示:
//! - `MatchFixture::match_game(player_id)` 创建 sample match 数据
//! - `FixtureBuilder` 链式构造带 score/status 的 match
//! - `InMemoryNatsMock` 模拟 match.events subject
//!
//! 关联: `RGS-DTL-026_Match域_详细设计书.md` §3 (房间状态机) + §4 (撮合)
//!       + `RGS-TST-UT-05_对战域_单元测试设计书.md` (per F3 衍生补, 待新建)

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== match-service mock + fixture 示例 ===\n");

    // 1. MatchFixture
    println!("[1] MatchFixture::match_game(\"alice\")");
    let m = fixture::match_game("alice");
    println!("    match_id={}, player_id={}, score={}, status={}\n", m.match_id, m.player_id, m.score, m.status);

    // 2. FixtureBuilder 链式构造
    println!("[2] FixtureBuilder::new(m).with_score(100).with_status(\"Active\").build()");
    let custom = FixtureBuilder::new(m.clone())
        .with_score(100)
        .with_status("Active")
        .build();
    println!("    match_id={}, score={}, status={}\n", custom.match_id, custom.score, custom.status);

    // 3. InMemoryNatsMock 模拟 match.events
    println!("[3] InMemoryNatsMock 模拟 match.events");
    let nats = InMemoryNatsMock::new();
    nats.publish("match.events", br#"{"event":"room_created","room_id":"r1"}"#).await.unwrap();
    nats.publish("match.events", br#"{"event":"team_assigned","room_id":"r1","team":0}"#).await.unwrap();
    nats.publish("match.events", br#"{"event":"match_started","room_id":"r1"}"#).await.unwrap();
    let count = nats.received_count("match.events");
    println!("    match.events received_count={} (期望 3)\n", count);

    println!("=== 完成 ===");
}
