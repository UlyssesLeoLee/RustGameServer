//! `FixtureBuilder` + 5 域 fixture 完整示例 (54.x 实质化版本)
//!
//! Run: `cargo run -p rgs-testkit --example fixture_builder_demo`
//!
//! 演示 5 域 fixture 工厂 + `FixtureBuilder` 链式 API:
//! - `fixture::player()` / `economy()` / `saga()` (53.3 已有)
//! - `fixture::match_game()` / `social_message()` / `admin_action()` (54.x 新增)
//! - `FixtureBuilder::new(fixture).with_xxx(...).build()` 链式定制
//!
//! 关联: `RGS-SPEC-000 §2.4` + 5 域 DTL-018/015-016/026/019-020/031

use rgs_testkit::fixture::{self, FixtureBuilder};

fn main() {
    println!("=== FixtureBuilder + 5 域 fixture 完整示例 (54.x) ===\n");

    // 1. 53.3 已实现: player / economy / saga
    println!("[1] 53.3 fixture (向后兼容):");
    let p = fixture::player();
    println!("    player()       → id={}, name={}, level={}", p.id, p.name, p.level);
    let e = fixture::economy("alice");
    println!("    economy(\"alice\") → player_id={}, currency={}, gold={}", e.player_id, e.currency, e.gold);
    let s = fixture::saga("transfer");
    println!("    saga(\"transfer\") → saga_id={}, state={}", s.saga_id, s.state);

    // 2. 54.x 新增: match / social / admin
    println!("\n[2] 54.x 新增 fixture:");
    let m = fixture::match_game("alice");
    println!("    match_game(\"alice\")   → match_id={}, status={}", m.match_id, m.status);
    let sm = fixture::social_message("alice", "bob");
    println!("    social_message(\"alice\", \"bob\") → message={:?}", sm.message);
    let a = fixture::admin_action("admin1", "ban", "bad_player");
    println!("    admin_action(\"admin1\", \"ban\", \"bad_player\") → action={}", a.action);

    // 3. FixtureBuilder 链式 API: 定制 player
    println!("\n[3] FixtureBuilder 链式 API (player):");
    let p_custom = FixtureBuilder::new(fixture::player())
        .with_name("Alice the Brave")
        .with_level(50)
        .build();
    println!("    .with_name(\"Alice the Brave\").with_level(50)");
    println!("    → name={}, level={}", p_custom.name, p_custom.level);

    // 4. FixtureBuilder 链式 API: 定制 economy
    println!("\n[4] FixtureBuilder 链式 API (economy):");
    let e_custom = FixtureBuilder::new(fixture::economy("alice"))
        .with_currency(9999)
        .with_gold(500)
        .build();
    println!("    .with_currency(9999).with_gold(500)");
    println!("    → currency={}, gold={}", e_custom.currency, e_custom.gold);

    // 5. FixtureBuilder 链式 API: 定制 match
    println!("\n[5] FixtureBuilder 链式 API (match):");
    let m_custom = FixtureBuilder::new(fixture::match_game("alice"))
        .with_score(100)
        .with_status("Active")
        .build();
    println!("    .with_score(100).with_status(\"Active\")");
    println!("    → score={}, status={}", m_custom.score, m_custom.status);

    // 6. FixtureBuilder 链式 API: 定制 social
    println!("\n[6] FixtureBuilder 链式 API (social):");
    let s_custom = FixtureBuilder::new(fixture::social_message("alice", "bob"))
        .with_message("Hello from test fixture!")
        .build();
    println!("    .with_message(\"Hello from test fixture!\")");
    println!("    → message={:?}", s_custom.message);

    // 7. FixtureBuilder 链式 API: 定制 admin
    println!("\n[7] FixtureBuilder 链式 API (admin):");
    let a_custom = FixtureBuilder::new(fixture::admin_action("admin1", "promote", "alice"))
        .with_action("demote")
        .with_target("bob")
        .build();
    println!("    .with_action(\"demote\").with_target(\"bob\")");
    println!("    → action={}, target_id={}", a_custom.action, a_custom.target_id);

    // 8. 实战: 5 域 fixture 组合
    println!("\n[8] 实战: 5 域 fixture 组合 (跨域测试):");
    let p = FixtureBuilder::new(fixture::player()).with_name("Hero").with_level(10).build();
    let e = fixture::economy(&p.id);
    let m = fixture::match_game(&p.id);
    let s = fixture::social_message(&p.id, "friend_1");
    let a = fixture::admin_action("admin_001", "audit", &p.id);
    println!("    player({}) + economy({}) + match({}) + social({}) + admin({})",
        p.name, e.player_id, m.match_id, s.friend_id, a.target_id);

    println!("\n=== Demo complete ===");
}
