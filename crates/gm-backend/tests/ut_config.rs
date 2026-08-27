//! gm-backend UT — GmConfig 单元测试
//!
//! 覆盖:
//! - 默认值(env 缺失时)
//! - 显式 env 覆盖
//! - invalid SocketAddr 报错
//! - for_test builder(UT 友好)
//!
//! env 测试需要串行执行:env 是进程级全局,`cargo test` 默认多线程并行
//! 会导致 test 之间互相污染。这里用 serial_test 串行化。
//! 注:`serial_test` 是常用轻量 dev-dep(0.5),加到 Cargo.toml。

use gm_backend::GmConfig;
use serial_test::serial;

/// 串行 env 隔离测试(清 env + 读默认值)
#[test]
#[serial]
fn gm_config_defaults_when_env_missing() {
    for k in ["GM_HTTP_ADDR", "GM_HEALTH_ADDR", "ADMIN_GRPC_ENDPOINT", "GM_JWT_SECRET"] {
        std::env::remove_var(k);
    }
    let cfg = GmConfig::from_env().expect("from_env should succeed with defaults");
    assert_eq!(cfg.http_addr.to_string(), "0.0.0.0:8443");
    assert_eq!(cfg.health_addr.to_string(), "0.0.0.0:8081");
    assert_eq!(cfg.admin_grpc_endpoint, "https://admin-service:50055");
    assert_eq!(cfg.jwt_secret, "dev-only-do-not-use-in-prod");
}

/// 串行 env 隔离测试(显式 set_var + 读)
#[test]
#[serial]
fn gm_config_respects_env_overrides() {
    std::env::set_var("GM_HTTP_ADDR", "127.0.0.1:9001");
    std::env::set_var("GM_HEALTH_ADDR", "127.0.0.1:9002");
    std::env::set_var("ADMIN_GRPC_ENDPOINT", "http://admin-staging:50055");
    std::env::set_var("GM_JWT_SECRET", "test-jwt-secret-32-bytes-min!!");

    let cfg = GmConfig::from_env().expect("from_env should pick up env vars");
    assert_eq!(cfg.http_addr.to_string(), "127.0.0.1:9001");
    assert_eq!(cfg.health_addr.to_string(), "127.0.0.1:9002");
    assert_eq!(cfg.admin_grpc_endpoint, "http://admin-staging:50055");
    assert_eq!(cfg.jwt_secret, "test-jwt-secret-32-bytes-min!!");

    for k in ["GM_HTTP_ADDR", "GM_HEALTH_ADDR", "ADMIN_GRPC_ENDPOINT", "GM_JWT_SECRET"] {
        std::env::remove_var(k);
    }
}

#[test]
#[serial]
fn gm_config_rejects_invalid_socket_addr() {
    std::env::set_var("GM_HTTP_ADDR", "not-a-socket-addr");
    let err = GmConfig::from_env().expect_err("invalid GM_HTTP_ADDR should error");
    let msg = err.to_string();
    assert!(
        msg.contains("invalid GM_HTTP_ADDR"),
        "error should mention GM_HTTP_ADDR, got: {}",
        msg
    );
    std::env::remove_var("GM_HTTP_ADDR");
}

#[test]
#[serial]
fn gm_config_rejects_invalid_health_addr() {
    std::env::set_var("GM_HEALTH_ADDR", "99999");
    let err = GmConfig::from_env().expect_err("invalid GM_HEALTH_ADDR should error");
    assert!(err.to_string().contains("invalid GM_HEALTH_ADDR"));
    std::env::remove_var("GM_HEALTH_ADDR");
}

#[test]
fn gm_config_for_test_builder() {
    // 不动 env,纯 builder,无串行需求
    let cfg = GmConfig::for_test("127.0.0.1:18000", "127.0.0.1:18001", "http://admin-test:50055")
        .expect("for_test should succeed");
    assert_eq!(cfg.http_addr.to_string(), "127.0.0.1:18000");
    assert_eq!(cfg.health_addr.to_string(), "127.0.0.1:18001");
    assert_eq!(cfg.admin_grpc_endpoint, "http://admin-test:50055");
    assert_eq!(cfg.jwt_secret, "test-secret");
}

#[test]
fn gm_config_clone_equality() {
    let cfg1 = GmConfig::for_test("0.0.0.0:8443", "0.0.0.0:8081", "http://x:1").unwrap();
    let cfg2 = cfg1.clone();
    assert_eq!(cfg1, cfg2);
}
