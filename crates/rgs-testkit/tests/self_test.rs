//! rgs-testkit self-test：验证 4 个子模块基本 API 可用

use rgs_testkit::mock::{DbMock, GrpcMock, NatsMock};
use rgs_testkit::{fixture, helper, mock, pg_test_db};

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
    for k in env.keys() {
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

// --- pg_test_db 子模块 (per RGS-REV-009 V3 H-1 / WF-1-55.31) ---

#[test]
fn pg_test_db_database_url_env_name_matches_sqlx() {
    // 防 fixture env var 名与 sqlx 默认脱节
    assert_eq!(pg_test_db::DATABASE_URL_ENV, "DATABASE_URL");
}

#[tokio::test]
async fn pg_test_db_pg_available_false_without_url() {
    // 显式 unset DATABASE_URL, 验证 pg_available() 不 panic 且返回 false
    let prev = std::env::var(pg_test_db::DATABASE_URL_ENV).ok();
    std::env::remove_var(pg_test_db::DATABASE_URL_ENV);
    let ok = pg_test_db::pg_available().await;
    if let Some(v) = prev {
        std::env::set_var(pg_test_db::DATABASE_URL_ENV, v);
    }
    assert!(!ok, "pg_available() must be false when DATABASE_URL unset");
}

#[tokio::test]
async fn pg_test_db_pg_pool_err_without_url() {
    let prev = std::env::var(pg_test_db::DATABASE_URL_ENV).ok();
    std::env::remove_var(pg_test_db::DATABASE_URL_ENV);
    let result = pg_test_db::pg_pool().await;
    if let Some(v) = prev {
        std::env::set_var(pg_test_db::DATABASE_URL_ENV, v);
    }
    assert!(result.is_err(), "pg_pool() must err when DATABASE_URL unset");
}

/// PG 集成 smoke test (feature-gated, 需真 PG)
/// 启用: `cargo test -p rgs-testkit --features pg-integration -- --include-ignored`
/// 前置: `docker compose -f docker/compose/docker-compose.yml up -d postgres`
///       + `export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres`
#[cfg(feature = "pg-integration")]
#[sqlx::test]
async fn pg_test_db_smoke_connects_and_selects_one(pool: sqlx::PgPool) {
    use rgs_testkit::pg_test_db::pg_pool;
    // 验证 sqlx::test 提供的 pool 与 fixture 提供的 pool 都活
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(row.0, 1);

    // fixture pg_pool() 用同一 DATABASE_URL 起第二个池, 验 fixture API 也通
    let pool2 = pg_pool().await.expect("fixture pg_pool() must succeed when DATABASE_URL set");
    let row2: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool2).await.unwrap();
    assert_eq!(row2.0, 1);
}
