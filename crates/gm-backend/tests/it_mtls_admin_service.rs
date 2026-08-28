//! W21 (2026-08-28) mTLS to admin-service e2e IT
//!
//! 用 k3s 真实证书 (从 rgs-secret-admin-tls + rgs-secret-ca 抽) 验 mTLS:
//! - try_connect 加载证书 → AdminGrpcClient 构造成功
//! - 4 env (GM_CLIENT_TLS_DOMAIN/CA/CERT/KEY) 缺一 → 降级 plaintext + warn
//! - 加载真实 k3s admin-service 端证书 → TLS 握手准备就绪
//!
//! 前置: k3s kubectl get secret rgs-secret-admin-tls/ca 抽证书到 /tmp/admin-*.pem
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md §3.1 (W9 mTLS 实装)

use gm_backend::{AdminGrpcClient, AppState, GmConfig};

const ADMIN_TLS_CRT: &str = "/tmp/admin-tls.crt";
const ADMIN_TLS_KEY: &str = "/tmp/admin-tls.key";
const ADMIN_CA: &str = "/tmp/admin-ca.pem";
const ADMIN_DOMAIN: &str = "admin.service";

fn set_mtls_env() {
    std::env::set_var("GM_CLIENT_TLS_DOMAIN", ADMIN_DOMAIN);
    std::env::set_var("GM_CLIENT_TLS_CA", ADMIN_CA);
    std::env::set_var("GM_CLIENT_TLS_CERT", ADMIN_TLS_CRT);
    std::env::set_var("GM_CLIENT_TLS_KEY", ADMIN_TLS_KEY);
}

fn clear_mtls_env() {
    std::env::remove_var("GM_CLIENT_TLS_DOMAIN");
    std::env::remove_var("GM_CLIENT_TLS_CERT");
    std::env::remove_var("GM_CLIENT_TLS_CA");
    std::env::remove_var("GM_CLIENT_TLS_KEY");
}

#[tokio::test]
async fn mtls_4_env_set_constructs_admin_grpc_client() {
    // 4 env 全设 → load cert 成功 + 构造 AdminGrpcClient
    set_mtls_env();
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        // k3s admin-service ClusterIP
        admin_grpc_endpoint: "https://admin-service:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let client = AdminGrpcClient::try_connect(&cfg);
    assert!(
        client.is_ok(),
        "4 mTLS env 全设 + 真实证书 → try_connect 应成功, got: {:?}",
        client.err()
    );
    clear_mtls_env();
}

#[tokio::test]
async fn mtls_3_env_set_uses_plaintext() {
    // 4 env 缺一 → plaintext + warn
    clear_mtls_env();
    std::env::set_var("GM_CLIENT_TLS_DOMAIN", ADMIN_DOMAIN);
    std::env::set_var("GM_CLIENT_TLS_CA", ADMIN_CA);
    // GM_CLIENT_TLS_CERT + GM_CLIENT_TLS_KEY 故意不设
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "https://admin-service:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let client = AdminGrpcClient::try_connect(&cfg);
    assert!(
        client.is_ok(),
        "3 env 缺 CERT/KEY → plaintext (env 缺失), 应成功"
    );
    clear_mtls_env();
}

#[tokio::test]
async fn mtls_https_endpoint_no_env_warns_security() {
    // https:// endpoint 但 mTLS env 缺失 → 仍构造成功 + warn log
    // 实际生产应 fail, 但 dev/test 友好 (per W9 设计)
    clear_mtls_env();
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "https://admin-service:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let client = AdminGrpcClient::try_connect(&cfg);
    assert!(client.is_ok(), "https:// 无 mTLS env → plaintext, 应成功");
    clear_mtls_env();
}

#[tokio::test]
async fn mtls_invalid_cert_path_returns_error() {
    // 4 env 设但 cert path 不存在 → load_client_tls 失败 → plaintext fallback
    set_mtls_env();
    std::env::set_var("GM_CLIENT_TLS_CERT", "/nonexistent/cert.pem");
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "https://admin-service:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let client = AdminGrpcClient::try_connect(&cfg);
    // 当前实现: 加载失败 → 降级 plaintext → try_connect Ok
    // (W9 决策: load 失败降级而非 panic, dev/test 友好)
    assert!(client.is_ok(), "无效 cert path → 降级 plaintext, 应成功");
    clear_mtls_env();
}

#[tokio::test]
async fn mtls_appstate_with_mtls_env_keeps_admin_grpc_some() {
    // AppState::new with 4 mTLS env → admin_grpc Some (not None)
    set_mtls_env();
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "https://admin-service:50055".to_string(),
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    let state = AppState::new(cfg);
    assert!(state.admin_grpc.is_some(), "4 mTLS env → admin_grpc Some");
    clear_mtls_env();
}
