//! M-2065.9: `RangeClient` HEAD + Range 全状态码 UT（per SPEC §6 + IMPL-PLAN §3.3）。
//!
//! 覆盖：
//! - 200 OK（含 `Accept-Ranges: bytes` + ETag）
//! - 206 Partial Content（带 `Content-Range`）
//! - 416 Range Not Satisfiable
//! - 200 OK（ETag mismatch → BackendEtagMismatch）
//! - 429 Too Many Requests
//! - 5xx
//! - HEAD 探测：缺 `Accept-Ranges` → BackendRangeUnsupported
//! - **FR-CDN-074**：所有 Range 请求携带 `If-Range: "<etag>"`（不传 Last-Modified）

use rgs_asset_download::error::DownloadError;
use rgs_asset_download::range_client::{HttpRangeSpec, RangeBackendProbe, RangeClient, RangeClientConfig};
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> RangeClient {
    RangeClient::with_config(RangeClientConfig {
        user_agent: "rgs-asset-download-test".into(),
        timeout_secs: 10,
        verify_tls: false,
    })
    .expect("build client")
}

fn token() -> CancellationToken {
    CancellationToken::new()
}

#[tokio::test]
async fn head_probe_supported_returns_supported() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/asset.bin"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Accept-Ranges", "bytes")
                .insert_header("Content-Length", "1024")
                .insert_header("ETag", "\"v1\""),
        )
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let probe = client.probe(&url, &token()).await.unwrap();
    assert_eq!(probe, RangeBackendProbe::Supported);
}

#[tokio::test]
async fn head_probe_missing_accept_ranges_returns_unsupported_error() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/asset.bin"))
        .respond_with(
            ResponseTemplate::new(200).insert_header("Content-Length", "1024"),
        )
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client.probe(&url, &token()).await.unwrap_err();
    assert!(matches!(
        err,
        DownloadError::BackendRangeUnsupported { .. }
    ));
}

#[tokio::test]
async fn head_probe_explicit_accept_ranges_none_returns_unsupported() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/asset.bin"))
        .respond_with(ResponseTemplate::new(200).insert_header("Accept-Ranges", "none"))
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client.probe(&url, &token()).await.unwrap_err();
    assert!(matches!(
        err,
        DownloadError::BackendRangeUnsupported { .. }
    ));
}

#[tokio::test]
async fn range_request_206_returns_partial_content() {
    let server = MockServer::start().await;
    let body_bytes: Vec<u8> = (0..256u32).map(|i| (i & 0xFF) as u8).collect();

    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .and(header("Range", "bytes=0-255"))
        // FR-CDN-074：必须携带 If-Range: "<etag>"
        .and(header("If-Range", "\"v1\""))
        .respond_with(
            ResponseTemplate::new(206)
                .insert_header("Content-Range", "bytes 0-255/1024")
                .insert_header("ETag", "\"v1\"")
                .set_body_bytes(body_bytes.clone()),
        )
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let resp = client
        .fetch_range(
            &url,
            &HttpRangeSpec::new(0, 255),
            Some("v1"),
            &token(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status, 206);
    assert_eq!(resp.body, body_bytes);
    let cr = resp.content_range.unwrap();
    assert_eq!(cr.start, 0);
    assert_eq!(cr.end, 255);
    assert_eq!(cr.complete_length, 1024);
}

#[tokio::test]
async fn range_request_416_returns_range_not_satisfiable() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .respond_with(ResponseTemplate::new(416))
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client
        .fetch_range(
            &url,
            &HttpRangeSpec::new(9999, 99999),
            Some("v1"),
            &token(),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        DownloadError::BackendRangeNotSatisfiable { .. }
    ));
}

#[tokio::test]
async fn range_request_200_means_etag_mismatch() {
    // 服务器忽略 Range + 返回 200 OK = ETag 不匹配 → BackendEtagMismatch
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("ETag", "\"v2\"")
                .set_body_bytes(vec![0u8; 1024]),
        )
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client
        .fetch_range(
            &url,
            &HttpRangeSpec::new(0, 255),
            Some("v1"),
            &token(),
        )
        .await
        .unwrap_err();
    match err {
        DownloadError::BackendEtagMismatch { expected, actual } => {
            // reqwest 把 ETag 头的引号保留在 value 中（RFC 7232 §2.3）
            assert_eq!(expected, "v1");
            assert_eq!(actual, "\"v2\"");
        }
        other => panic!("expected BackendEtagMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn range_request_429_returns_too_many_requests() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client
        .fetch_range(
            &url,
            &HttpRangeSpec::new(0, 255),
            Some("v1"),
            &token(),
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        DownloadError::BackendTooManyRequests { .. }
    ));
}

#[tokio::test]
async fn range_request_5xx_returns_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client
        .fetch_range(
            &url,
            &HttpRangeSpec::new(0, 255),
            Some("v1"),
            &token(),
        )
        .await
        .unwrap_err();
    match err {
        DownloadError::BackendHttpError { status, .. } => assert_eq!(status, 503),
        other => panic!("expected BackendHttpError, got {other:?}"),
    }
}

#[tokio::test]
async fn cancelled_token_aborts_request() {
    // 服务端故意 hang：5s 延迟
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/slow.bin"))
        .respond_with(
            ResponseTemplate::new(206)
                .insert_header("Content-Range", "bytes 0-255/1024")
                .set_delay(std::time::Duration::from_secs(5))
                .set_body_bytes(vec![0u8; 256]),
        )
        .mount(&server)
        .await;

    let url = format!("{}/slow.bin", server.uri());
    let client = test_client();
    let cancel = CancellationToken::new();
    let cancel_for_task = cancel.clone();
    let task = tokio::spawn(async move {
        client
            .fetch_range(
                &url,
                &HttpRangeSpec::new(0, 255),
                Some("v1"),
                &cancel_for_task,
            )
            .await
    });
    // 立即取消
    cancel.cancel();
    let result = task.await.unwrap();
    assert!(matches!(result, Err(DownloadError::Cancelled)));
}

#[tokio::test]
async fn head_probe_5xx_returns_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/asset.bin"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let url = format!("{}/asset.bin", server.uri());
    let client = test_client();
    let err = client.probe(&url, &token()).await.unwrap_err();
    match err {
        DownloadError::BackendHttpError { status, .. } => assert_eq!(status, 500),
        other => panic!("expected BackendHttpError, got {other:?}"),
    }
}
