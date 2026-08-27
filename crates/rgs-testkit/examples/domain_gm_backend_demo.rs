//! `gm-backend` (第 8 域 GM 后台) mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_gm_backend_demo`
//!
//! 演示:
//! - `axum-test 16` in-process 测试 (per gm-backend/tests/integration_gm_basic.rs)
//! - `assert_cmd 2` 启动 GM APIGW 进程做黑盒测试 (per gm-backend/tests/fail_closed_start.rs)
//! - `serial_test 0.5` 隔离 env-mutating 测试 (per gm-backend/tests/ut_config.rs)
//! - `TonicGrpcMock` 模拟 admin-service 5 endpoint (gm-backend 的下游)
//!
//! 关联: `RGS-BAS-003_运维与GM后台管控_基本设计书.md` §2 (axum 0.7) + §3 (5 endpoint)
//!       + `RGS-TST-UT-08_GM后台_单元测试设计书.md` (per f13acc6, 19/19 PASS)

use rgs_testkit::mock::{GrpcMock, TonicGrpcMock};

#[tokio::main]
async fn main() {
    println!("=== gm-backend mock + fixture 示例 ===\n");

    // 1. gm-backend 测试结构总览
    println!("[1] gm-backend 测试文件 (3 份):");
    println!("    tests/ut_config.rs               6 测试  GmConfig 配载 (serial_test env 隔离)");
    println!("    tests/integration_gm_basic.rs   12 测试  7 路由 + 5 handler + 4 路由边界 (axum-test 16)");
    println!("    tests/fail_closed_start.rs       1 测试  启动 fail-closed (assert_cmd 2)");
    println!("    total: 19/19 PASS (per f0c6ea2)\n");

    // 2. TonicGrpcMock 模拟 admin-service 5 endpoint (gm-backend 的下游 stub)
    println!("[2] TonicGrpcMock 模拟 admin-service 5 endpoint (per BAS-003 §3.1-§3.4)");
    let mut grpc = TonicGrpcMock::new().await;
    // gm-backend 当前是 stub 状态 (per TBD-08-03 v0.2 实装 admin-service gRPC client)
    // 这里只 mock 字段级 stub 返回
    grpc.expect("POST", "/admin.v1.AdminService/QueryHealthView", 200, br#"{"service":"admin","admin_endpoint":"http://admin:50055","mode":"stub-ok"}"#);
    grpc.expect("POST", "/admin.v1.AdminService/BanAccount", 200, br#"{"status":"queued","op":"ban"}"#);
    grpc.expect("POST", "/admin.v1.AdminService/GrantCompensation", 200, br#"{"status":"queued","op":"compensation"}"#);
    grpc.expect("POST", "/admin.v1.AdminService/SetMaintenanceMode", 200, br#"{"status":"queued","op":"maintenance"}"#);
    grpc.expect("POST", "/admin.v1.AdminService/QueryAuditLog", 200, br#"{"items":[],"next":"stub"}"#);
    println!("    TonicGrpcMock url={}\n", grpc.url());
    println!("    5 admin-service expectations registered\n");

    // 3. env 隔离模式 (per ut_config.rs)
    println!("[3] env 隔离 (per ut_config.rs)");
    println!("    env 是进程级全局,`cargo test` 默认多线程并行");
    println!("    会导致 test 之间相互污染,需用 serial_test 串行化");
    println!("    模式: #[test] #[serial] + 测试首行 std::env::remove_var()\n");

    // 4. v0.2 字段级协议字段 (per F8 处置段)
    println!("[4] v0.2 字段级协议字段 (per 2026-08-28 跨反馈 F8 处置)");
    println!("    当前 stub 字段 ≠ BAS-003/DTL-003 协议字段,UT-08-D001/D004/D005 未测到");
    println!("    v0.2 实装需新增:");
    println!("    - SetMaintenanceModeResponse.propagation_status (PROPAGATING/CONVERGED)");
    println!("    - QueryHealthViewResponse.services[] (5 子字段)");
    println!("    - QueryAuditLogResponse.entries[] + has_more");

    println!("\n=== 完成 ===");
}
