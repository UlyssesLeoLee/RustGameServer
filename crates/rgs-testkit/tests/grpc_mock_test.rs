//! `TonicGrpcMock` 实质实现的集成测试
//!
//! 验证:
//! 1. `TonicGrpcMock::new()` 启动 mock server, `url()` 返回 `http://127.0.0.1:PORT`
//! 2. `expect()` 注册 expectation 后, HTTP client 调用能拿到 mock 响应
//!    (用 mockito 自带的异步 client 端到端验证 expectation 触发)
//! 3. `NoopMock` 仍能 serve (向后兼容)
//!
//! 完整 gRPC client 测试 (含 protobuf 解码) 留待 54.x 后续 PR — 本次仅 mock
//! server 端, 不引入 tonic / prost / protobuf.

// NoopMock 仍 deprecated (PG 部分), 本测试有意验证其 backward compat,
// suppress 警告仅限本测试文件 (不传染 lib code).
#![allow(deprecated)]

use rgs_testkit::mock::{GrpcMock, NoopMock, TonicGrpcMock};

#[tokio::test]
async fn tonic_grpc_mock_new_yields_url() {
    let m = TonicGrpcMock::new().await;
    let url = m.url();
    assert!(
        url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:"),
        "expected mockito server URL to be localhost, got: {url}"
    );
}

#[tokio::test]
async fn tonic_grpc_mock_expect_and_respond() {
    let mut m = TonicGrpcMock::new().await;
    let body = br#"{"session_epoch":"e1","player_id":"p-1"}"#;
    m.expect("POST", "/player.v1.PlayerService/Login", 200, body);

    // 用 reqwest/ureq 需额外依赖. 改用 mockito 自带的 `Mock::matches_async`
    // 端到端验证 expectation 触发: 任何匹配 method+path 的请求 → 拿到预期 body.
    //
    // 这里用 std::net TCP 极简验证: 连 mockito URL, 发最小 HTTP/1.1 请求,
    // 读回响应行 + 响应体, 断言 status 200 且 body 匹配.
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let url = m.url().to_string(); // e.g. "http://127.0.0.1:54321"
    let port: u16 = url
        .rsplit(':')
        .next()
        .expect("url has port")
        .trim_end_matches('/')
        .parse()
        .expect("port is u16");
    let host = url
        .trim_start_matches("http://")
        .trim_end_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-')
        .split(':')
        .next()
        .unwrap_or("127.0.0.1")
        .to_string();

    let mut stream = TcpStream::connect((host.as_str(), port)).expect("connect mockito");
    let req = format!(
        "POST /player.v1.PlayerService/Login HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Content-Type: application/grpc\r\n\
         Content-Length: 0\r\n\
         Connection: close\r\n\
         \r\n"
    );
    stream.write_all(req.as_bytes()).expect("write");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");

    // 验证响应行 + body
    assert!(
        resp.contains("200"),
        "expected 200 OK in response, got: {resp}"
    );
    let body_str = std::str::from_utf8(body).expect("body utf8");
    assert!(
        resp.contains(body_str),
        "expected response body to contain {body_str}, got: {resp}"
    );
}

#[tokio::test]
async fn noop_mock_serve_ok() {
    let mut m = NoopMock;
    // NoopMock 仍 backward compat: serve / url / expect 均可用
    assert!(m.serve().await.is_ok());
    assert_eq!(m.url(), "http://mock.invalid");
    // expect 不 panic 即视为 OK
    m.expect("POST", "/anywhere", 200, b"ignored");
}
