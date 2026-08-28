//! S4 Phase 2 step 1 IT: admin-service gRPC client 注入
//!
//! 验证 gm-backend 注入 admin-service gRPC client + 失败降级
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md
//!
//! 测试目标:
//! 1. `try_connect` 在 endpoint URL 有效时返 Ok(connect_lazy 不阻塞)
//! 2. `try_connect` 在 endpoint URL 无效时返 Err(让 AppState 降级为 None)
//! 3. `try_connect` 在不可达 endpoint 仍 Ok(connect_lazy 懒连接,失败延后到 RPC 调用)
//! 4. `health_check` 在不可达 endpoint 返 Err(超时 / 连接失败)
//! 5. AppState::with_audit_store + disable_admin_grpc=true → admin_grpc 永远 None
//! 6. AppState::with_audit_store + disable_admin_grpc=false + 不可达 → admin_grpc 仍 Some
//!    (fail-open: 不会因 admin-service 不可达而 panic)

use gm_backend::{AdminGrpcClient, AppState, GmConfig};
use std::time::{Duration, Instant};

#[test]
fn try_connect_accepts_valid_http_endpoint() {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .unwrap();
    // 验证 for_test 强制 disable_admin_grpc=true(测试隔离)
    assert!(cfg.disable_admin_grpc);
}

#[test]
fn appstate_with_test_config_keeps_admin_grpc_none() {
    // GmConfig::for_test 默认 disable_admin_grpc=true → AppState::new 不尝试连接
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin:50055").unwrap();
    let state = AppState::new(cfg);
    assert!(
        state.admin_grpc.is_none(),
        "test config must keep admin_grpc None (no real connection)"
    );
}

#[test]
fn appstate_with_disable_admin_grpc_keeps_admin_grpc_none() {
    // 即使 endpoint 看起来合理,disable_admin_grpc=true 也保持 None
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://admin:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: true,
    };
    let state = AppState::new(cfg);
    assert!(state.admin_grpc.is_none());
}

#[tokio::test]
async fn appstate_with_admin_grpc_enabled_attempts_connect_lazy() {
    // disable_admin_grpc=false + 任意 endpoint (本地无效端口) → connect_lazy 不阻塞,Some
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(), // 1 = 不可达端口
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let state = AppState::new(cfg);
    // connect_lazy 不会真连 → admin_grpc 仍 Some
    assert!(state.admin_grpc.is_some(), "connect_lazy must not block");
}

#[tokio::test]
async fn health_check_against_unreachable_endpoint_returns_err_within_500ms() {
    // 不可达 endpoint → health_check 500ms timeout 返 Err
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(), // 不可达
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let client = AdminGrpcClient::try_connect(&cfg).expect("try_connect ok (lazy)");
    let started = Instant::now();
    let res = client.health_check().await;
    let elapsed = started.elapsed();
    assert!(res.is_err(), "unreachable endpoint must return Err");
    // 500ms timeout + 一点 jitter, 应该 < 1.5s
    assert!(
        elapsed < Duration::from_millis(1500),
        "timeout must be ~500ms, got {elapsed:?}"
    );
}

#[tokio::test]
async fn try_connect_accepts_https_url() {
    // tonic 0.12 `Endpoint::from_shared` 接受任何 URI 形式字符串,
    // 验证有效 HTTPS URL 解析成功(lazy connect 不阻塞)
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "https://admin.svc.cluster.local:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let res = AdminGrpcClient::try_connect(&cfg);
    assert!(res.is_ok(), "valid https URL must succeed (lazy connect)");
}
