//! `TonicGrpcMock` 完整使用示例 (54.x 实质化版本)
//!
//! Run: `cargo run -p rgs-testkit --example mock_grpc_demo`
//!
//! 演示 `mockito` 集成的 gRPC mock, 用于 5 域跨域测试 fixture:
//! - `TonicGrpcMock::new().await` 启动 mock server
//! - `mock.expect(method, path, status, body)` 注册 expectation
//! - `mock.url()` 给 tonic client connect 用
//!
//! 关联: `RGS-SPEC-000 §2.4` + `RGS-IMPL-001 §3` + WF-1 mock-grpc (per 53.3 骨架 → 54.x 接入)

use rgs_testkit::mock::{GrpcMock, TonicGrpcMock};

#[tokio::main]
async fn main() {
    println!("=== TonicGrpcMock 完整使用示例 (54.x 实质化) ===\n");

    // 1. 启动 mock server 拿 url
    println!("[1] TonicGrpcMock::new().await 启动 mock server");
    let mut mock = TonicGrpcMock::new().await;
    let url = mock.url().to_string(); // owned String, 避免与 expect() 的 &mut self 借用冲突
    println!("    mock server url = {}\n", url);
    assert!(url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:"));

    // 2. 注册 1 个 expectation (player 域 Login RPC)
    println!("[2] mock.expect() 注册 1 个 expectation");
    mock.expect(
        "POST",
        "/player.v1.PlayerService/Login",
        200,
        br#"{"session_epoch":"e1","player_id":"p1"}"#,
    );
    println!("    POST /player.v1.PlayerService/Login → 200\n");

    // 3. 用 std::net::TcpStream 端到端验证(不引 reqwest/ureq)
    println!("[3] 端到端验证 (raw HTTP/1.1 request)");
    use std::io::{Read, Write};
    use std::net::TcpStream;
    let host_port = url.trim_start_matches("http://");
    let mut stream = TcpStream::connect(host_port).expect("connect mock server");
    let request = format!(
        "POST /player.v1.PlayerService/Login HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: 0\r\n\
         Connection: close\r\n\
         \r\n",
        host_port
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    let status_line = response.lines().next().unwrap_or("");
    println!("    response status = {}", status_line);
    assert!(status_line.contains("200"), "expected 200 OK");
    assert!(response.contains("session_epoch"));
    println!("    ✓ status=200 + body contains session_epoch\n");

    // 4. 5 域跨域 gRPC 场景
    println!("[4] 5 域跨域 gRPC 测试场景 (per DTL-021~025):");
    println!("    player-service  → economy-service (TransferCredits, 登录后扣费)");
    println!("    player-service  → social-service  (PushNotify, 登录后推送通知)");
    println!("    match-service   → admin-service   (AuditReport, 异常比赛上报)");

    println!("\n=== Demo complete ===");
}
