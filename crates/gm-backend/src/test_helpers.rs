//! gm-backend 测试 helper(per RGS-W26 桶 2a 业务实装)
//!
//! 提供 `build_test_state()` 给 5 个 it_business_*.rs 测试用,
//! 默认 `disable_admin_grpc=true` → admin_grpc=None, 走 InMemory 路径
//!
//! 关联: RGS-PLAN-WBS-token-bucket-v0.3 §2.2.1

use crate::{AppState, GmConfig};

/// 构造测试用 AppState(in-process 隔离, 不连 admin-service gRPC)
///
/// 默认配置:
/// - http_addr / health_addr: 127.0.0.1:0 (随机端口)
/// - admin_grpc_endpoint: http://admin-staging:50055
/// - jwt_secret: "test-secret"
/// - require_jwt: false
/// - disable_admin_grpc: true
pub async fn build_test_state() -> AppState {
    let cfg = GmConfig::for_test("127.0.0.1:0", "127.0.0.1:0", "http://admin-staging:50055")
        .expect("for_test config should be valid");
    AppState::new(cfg)
}

/// 构造不可达 admin-service 的 AppState(测降级路径用)
/// disable_admin_grpc=false + endpoint 不可达 → connect_lazy 成功, 实际 RPC 失败
pub async fn build_test_state_unreachable_admin() -> AppState {
    let cfg = GmConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        health_addr: "127.0.0.1:0".parse().unwrap(),
        admin_grpc_endpoint: "http://127.0.0.1:1".to_string(), // 不可达
        jwt_secret: "test".to_string(),
        require_jwt: false,
        disable_admin_grpc: false,
    };
    AppState::new(cfg)
}
