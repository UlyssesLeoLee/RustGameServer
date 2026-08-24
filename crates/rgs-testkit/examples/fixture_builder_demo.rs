//! Fixture 工厂完整示例 (53.3 已实现)
//!
//! Run: `cargo run -p rgs-testkit --example fixture_builder_demo`
//!
//! 53.3 已实现 3 个 fixture factory (player / economy / saga).
//! 54.x 路线图 (per §5.4): 补 match / social / admin + FixtureBuilder 链式 API.

use rgs_testkit::fixture;

fn main() {
    println!("=== Fixture 完整示例 (53.3 3 域) ===\n");

    // 1. player factory
    println!("[1] fixture::player()");
    let p = fixture::player();
    println!("    id        = {}", p.id);
    println!("    name      = {}", p.name);
    println!("    level     = {}", p.level);
    println!("    created_at= {}\n", p.created_at);

    // 2. economy factory (per player)
    println!("[2] fixture::economy(\"alice\")");
    let e = fixture::economy("alice");
    println!("    player_id = {}", e.player_id);
    println!("    currency  = {}", e.currency);
    println!("    gold      = {}\n", e.gold);

    // 3. saga factory (unique saga_id)
    println!("[3] fixture::saga(\"transfer\") (uuid v4 unique)");
    let s1 = fixture::saga("transfer");
    let s2 = fixture::saga("transfer");
    println!("    saga_id[0] = {}", s1.saga_id);
    println!("    saga_id[1] = {}", s2.saga_id);
    println!("    unique     = {}", s1.saga_id != s2.saga_id);
    println!("    saga_type  = {}", s1.saga_type);
    println!("    step       = {}", s1.step);
    println!("    state      = {}\n", s1.state);

    // 4. 54.x 路线图: 5 域 fixture + FixtureBuilder
    println!("[4] 54.x 路线图 (待接入):");
    println!("    let m = fixture::match_game(\"alice\");");
    println!("    let s = fixture::social_message(\"alice\", \"bob\");");
    println!("    let a = fixture::admin_action(\"admin1\", \"ban\", \"bad_player\");");
    println!();
    println!("    let p = FixtureBuilder::new(fixture::player())");
    println!("        .with_name(\"Alice\")");
    println!("        .with_level(50)");
    println!("        .build();");

    println!("\n=== Demo complete ===");
}
