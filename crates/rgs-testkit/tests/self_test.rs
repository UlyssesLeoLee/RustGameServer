//! rgs-testkit self-test：验证 3 个子模块基本 API 可用

use rgs_testkit::{fixture, helper, mock};
use rgs_testkit::mock::{DbMock, GrpcMock, NatsMock};

#[test]
fn fixture_player_basic() {
    let p = fixture::player();
    assert_eq!(p.id, "player-test-001");
    assert_eq!(p.level, 1);
    assert!(p.created_at <= chrono::Utc::now());
}

#[test]
fn fixture_economy_uses_player_id() {
    let e = fixture::economy("player-x");
    assert_eq!(e.player_id, "player-x");
    assert_eq!(e.currency, 1000);
    assert_eq!(e.gold, 50);
}

#[test]
fn fixture_saga_unique_id() {
    let s1 = fixture::saga("purchase");
    let s2 = fixture::saga("purchase");
    assert_ne!(s1.saga_id, s2.saga_id);
    assert_eq!(s1.saga_type, "purchase");
    assert_eq!(s1.state, "Pending");
}

#[test]
fn helper_init_tracing_idempotent() {
    helper::init_tracing();
    helper::init_tracing(); // 多次调用不应 panic
}

#[test]
fn helper_load_test_env_empty() {
    let env = helper::load_test_env();
    for (k, _) in env.iter() {
        assert!(!k.is_empty());
    }
}

#[test]
fn mock_noop_db_url_format() {
    let m = mock::NoopMock;
    let url = m.mock_url();
    assert!(url.starts_with("postgres://"));
}

#[tokio::test]
async fn mock_noop_grpc_serve_ok() {
    let m = mock::NoopMock;
    let result = m.serve().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn mock_noop_nats_publish_ok() {
    let m = mock::NoopMock;
    let result = m.publish("test.subject", b"{}").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn fixture_init_test_db_returns_url() {
    let url = fixture::init_test_db("rgs_test_self").await;
    assert!(url.is_ok());
    let url = url.unwrap();
    assert!(url.contains("rgs_test_self"));
}