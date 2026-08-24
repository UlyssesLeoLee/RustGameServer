//! `GrpcMock` trait 完整使用示例 (53.3 占位形态)
//!
//! Run: `cargo run -p rgs-testkit --example mock_grpc_demo`
//!
//! 54.x 接入计划 (per §3.2):
//! - `TonicGrpcMock` (mockito 集成) —— `expect("POST", "/path", 200, body)`
//! - 5 域跨域 gRPC 单元/集成 test
//!
//! 当前 53.3 形态: `GrpcMock` trait 仅 `serve()` 占位方法 + `NoopMock` 空实现.
//! 本 example 演示 trait 当前形态 + 54.x 演进方向.
//!
//! 注: 故意允许 deprecation 警告以演示 53.3 NoopMock 占位形态,
//! 与 `test_deprecated_warns.rs` 一致. 业务 test 禁止用 NoopMock 替代真 PG.

#![allow(deprecated)]

use rgs_testkit::mock::{GrpcMock, NoopMock};

#[tokio::main]
async fn main() {
    println!("=== GrpcMock 完整使用示例 (53.3 形态) ===\n");

    // 1. 53.3 当前 API: NoopMock 实现 GrpcMock::serve() 占位
    let m = NoopMock;
    println!("[1] GrpcMock::serve() (53.3 占位, 返回 Ok)");
    let result = m.serve().await;
    println!("    result = {:?}\n", result);

    // 2. 54.x 计划 API (待接入, 编译期不可用)
    println!("[2] 54.x 计划 API (尚未接入):");
    println!("    use rgs_testkit::mock::{{GrpcMock, TonicGrpcMock}};");
    println!("    let mut mock = TonicGrpcMock::new().await;");
    println!("    mock.expect(\"POST\", \"/player.v1.PlayerService/Login\", 200, br#\"{{\"session_epoch\":\"e1\"}}\"#);");
    println!("    let url = mock.url();");
    println!("    // 给 tonic client connect 用");
    println!("    assert!(url.starts_with(\"http://\"));");
    println!();

    // 3. 跨域场景: 5 域 DTL §6 测试规格引用
    println!("[3] 5 域跨域 gRPC 测试 (54.x 接入后可用):");
    println!("    player-service → economy-service (登录后扣费)");
    println!("    player-service → social-service (登录后推送通知)");
    println!("    match-service → admin-service (异常比赛上报)");

    println!("\n=== Demo complete ===");
}
