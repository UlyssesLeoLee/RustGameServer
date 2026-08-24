//! 5 域 fixture + builder 实质实现的集成测试
//!
//! per RGS-IMPL-001 §3 + RGS-SPEC-000 §2.4
//! 覆盖:
//! - 3 个 5 域 factory (match_game / social_message / admin_action)
//! - FixtureBuilder 链式 API (player / economy)
//! - init_test_db 返回 URL (默认 fallback: env-var 或占位)

use rgs_testkit::fixture::{self, FixtureBuilder};

// ============================================================================
// 5 域 factory 基本形状
// ============================================================================

#[test]
fn match_game_basic() {
    let m = fixture::match_game("p1");
    assert!(m.match_id.starts_with("match-test-"));
    assert_eq!(m.player_id, "p1");
    assert_eq!(m.score, 0);
    assert_eq!(m.status, "Pending");
    assert!(m.started_at <= chrono::Utc::now());
}

#[test]
fn match_game_unique_id() {
    let m1 = fixture::match_game("p1");
    let m2 = fixture::match_game("p1");
    assert_ne!(m1.match_id, m2.match_id, "match_id must be unique per call");
}

#[test]
fn social_message_basic() {
    let s = fixture::social_message("p1", "p2");
    assert_eq!(s.player_id, "p1");
    assert_eq!(s.friend_id, "p2");
    assert_eq!(s.message, "Hello from test");
    assert!(s.sent_at <= chrono::Utc::now());
}

#[test]
fn admin_action_basic() {
    let a = fixture::admin_action("admin1", "ban", "p1");
    assert_eq!(a.admin_id, "admin1");
    assert_eq!(a.action, "ban");
    assert_eq!(a.target_id, "p1");
    assert!(a.performed_at <= chrono::Utc::now());
}

// ============================================================================
// FixtureBuilder 链式 API
// ============================================================================

#[test]
fn player_builder_custom() {
    let p = FixtureBuilder::new(fixture::player())
        .with_name("Custom Name")
        .with_level(99)
        .build();
    assert_eq!(p.name, "Custom Name");
    assert_eq!(p.level, 99);
    // 其它字段保持 factory 默认
    assert_eq!(p.id, "player-test-001");
}

#[test]
fn economy_builder_custom() {
    let e = FixtureBuilder::new(fixture::economy("p1"))
        .with_currency(9999)
        .with_gold(500)
        .build();
    assert_eq!(e.currency, 9999);
    assert_eq!(e.gold, 500);
    assert_eq!(e.player_id, "p1");
}

#[test]
fn match_builder_custom() {
    let m = FixtureBuilder::new(fixture::match_game("p1"))
        .with_score(100)
        .with_status("Completed")
        .build();
    assert_eq!(m.score, 100);
    assert_eq!(m.status, "Completed");
    assert_eq!(m.player_id, "p1");
}

#[test]
fn social_builder_custom() {
    let s = FixtureBuilder::new(fixture::social_message("p1", "p2"))
        .with_message("Custom greeting")
        .build();
    assert_eq!(s.message, "Custom greeting");
    assert_eq!(s.player_id, "p1");
    assert_eq!(s.friend_id, "p2");
}

#[test]
fn admin_builder_custom() {
    let a = FixtureBuilder::new(fixture::admin_action("admin1", "ban", "p1"))
        .with_action("mute")
        .with_target("p2")
        .build();
    assert_eq!(a.action, "mute");
    assert_eq!(a.target_id, "p2");
    assert_eq!(a.admin_id, "admin1");
}

// ============================================================================
// init_test_db: 默认 fallback (testcontainers feature 未启用)
// ============================================================================

#[tokio::test]
async fn init_test_db_returns_url() {
    // 默认未启用 testcontainers feature, 应走 env-var fallback
    // 显式 unset TEST_DATABASE_URL 避免环境干扰
    let prev = std::env::var("TEST_DATABASE_URL").ok();
    std::env::remove_var("TEST_DATABASE_URL");
    let url = fixture::init_test_db("test_db").await.unwrap();
    if let Some(v) = prev {
        std::env::set_var("TEST_DATABASE_URL", v);
    }
    // 占位 URL 包含 db name
    assert!(url.contains("test_db") || url.contains("postgres"));
}
