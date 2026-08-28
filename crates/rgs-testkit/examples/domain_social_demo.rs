//! `social-service` mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_social_demo`
//!
//! 演示:
//! - `SocialFixture::social_message(from, to)` 创建 sample 消息
//! - `FixtureBuilder` 链式构造带 message 的 social fixture
//! - `InMemoryNatsMock` 模拟 social.events subject
//!
//! 关联: `RGS-DTL-019_社交域_详细设计书.md` §3 (好友) + §4 (消息)
//!       + `RGS-DTL-020_聊天域_详细设计书.md`
//!       + `RGS-TST-UT-03_社交域_单元测试设计书.md` (per F3 衍生补, 待新建)

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== social-service mock + fixture 示例 ===\n");

    // 1. SocialFixture
    println!("[1] SocialFixture::social_message(\"alice\", \"bob\")");
    let m = fixture::social_message("alice", "bob");
    println!(
        "    player_id={}, friend_id={}, message={:?}\n",
        m.player_id, m.friend_id, m.message
    );

    // 2. FixtureBuilder 链式构造
    println!("[2] FixtureBuilder::new(m).with_message(\"hello world\").build()");
    let custom = FixtureBuilder::new(m.clone())
        .with_message("hello world")
        .build();
    println!(
        "    player_id={}, friend_id={}, message={:?}\n",
        custom.player_id, custom.friend_id, custom.message
    );

    // 3. InMemoryNatsMock 模拟 social.events
    println!("[3] InMemoryNatsMock 模拟 social.events");
    let nats = InMemoryNatsMock::new();
    nats.publish(
        "social.events",
        br#"{"event":"friend_request","from":"alice","to":"bob"}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "social.events",
        br#"{"event":"friend_accepted","from":"bob","to":"alice"}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "social.events",
        br#"{"event":"message_sent","from":"alice","to":"bob","text":"hi"}"#,
    )
    .await
    .unwrap();
    let count = nats.received_count("social.events");
    println!("    social.events received_count={} (期望 3)\n", count);

    println!("=== 完成 ===");
}
