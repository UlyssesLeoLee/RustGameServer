//! M-2065.10: `ChunkOrchestrator` 并发 + 暂停 / 取消 UT（per SPEC §6 + IMPL-PLAN §3.3）。
//!
//! 覆盖：
//! - 桌面 16 路并发上限
//! - 移动 4 路并发上限
//! - 完整调度 → 落盘字节正确
//! - 暂停 → in_flight 被取消（per FR-CDN-083 `cancel_request` 标记 + 取消 token 触发）
//! - 取消 → 丢弃所有 in_flight（per FR-CDN-083 `abort_request` 标记）
//! - ETag mismatch → 触发 BackendEtagMismatch（**不**在 chunk 层面重试）

use rgs_asset_download::chunk_orchestrator::{ChunkOrchestrator, ChunkSpec};
use rgs_asset_download::config::{DownloadConfig, PlatformProfile};
use rgs_asset_download::error::DownloadError;
use rgs_asset_download::range_client::{RangeClient, RangeClientConfig};
use tempfile::tempdir;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_client() -> RangeClient {
    RangeClient::with_config(RangeClientConfig {
        user_agent: "test".into(),
        timeout_secs: 30,
        verify_tls: false,
    })
    .unwrap()
}

#[test]
fn desktop_concurrency_caps_at_16() {
    let cfg = DownloadConfig::default();
    let orch = ChunkOrchestrator::new(cfg);
    assert_eq!(orch.concurrency_cap(), 16);
}

#[test]
fn mobile_concurrency_caps_at_4() {
    let cfg = DownloadConfig {
        platform_profile: PlatformProfile::Mobile,
        ..DownloadConfig::default()
    };
    let orch = ChunkOrchestrator::new(cfg);
    assert_eq!(orch.concurrency_cap(), 4);
}

#[test]
fn plan_chunks_aligns_to_chunk_size() {
    let cfg = DownloadConfig::default(); // 8 MB
    let orch = ChunkOrchestrator::new(cfg);
    let chunks = orch.plan_chunks(20 * 1024 * 1024, None);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].end, 8 * 1024 * 1024 - 1);
    assert_eq!(chunks[1].end, 16 * 1024 * 1024 - 1);
    assert_eq!(chunks[2].end, 20 * 1024 * 1024 - 1);
}

#[tokio::test]
async fn full_run_writes_correct_bytes() {
    let server = MockServer::start().await;
    let chunk_size: u64 = 1024;
    let total: u64 = 4 * chunk_size; // 4 KiB

    // 单 mock 匹配所有 Range 请求
    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .respond_with(
            ResponseTemplate::new(206)
                .insert_header("Content-Range", "bytes 0-1023/4096")
                .insert_header("ETag", "\"v1\"")
                .set_body_bytes(vec![0xAA; chunk_size as usize]),
        )
        .mount(&server)
        .await;

    // 4 平台默认 chunk_size = 8MB；此处显式改为 1KB 以便切出 4 个 chunk
    let cfg = DownloadConfig {
        chunk_size_bytes: chunk_size,
        ..DownloadConfig::default()
    };
    let orch = ChunkOrchestrator::new(cfg);
    let client = make_client();
    let chunks = orch.plan_chunks(total, Some("v1".into()));
    assert_eq!(chunks.len(), 4);

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("out.bin").to_string_lossy().to_string();
    let outcome = orch
        .run(
            &format!("{}/asset.bin", server.uri()),
            &file_path,
            chunks,
            &client,
        )
        .await
        .unwrap();
    assert_eq!(outcome.bytes_received, total);
    assert_eq!(outcome.chunks_completed, 4);

    let on_disk = std::fs::read(&file_path).unwrap();
    assert_eq!(on_disk.len() as u64, total);
}

#[tokio::test]
async fn pause_marks_cancel_request_on_in_flight() {
    let cfg = DownloadConfig::default();
    let orch = ChunkOrchestrator::new(cfg);
    orch.register_in_flight_for_test(ChunkSpec {
        index: 0,
        start: 0,
        end: 1023,
        etag: None,
    })
    .await;
    orch.pause().await.unwrap();
    assert!(orch.is_paused());
    assert_eq!(orch.cancel_request_count(), 1);
    let snapshot = orch.in_flight_snapshot().await;
    assert!(snapshot[0].cancel_request, "in_flight.cancel_request must be true after pause");
    assert!(!snapshot[0].abort_request, "pause should not set abort_request");
}

#[tokio::test]
async fn cancel_marks_abort_request_on_in_flight() {
    let cfg = DownloadConfig::default();
    let orch = ChunkOrchestrator::new(cfg);
    orch.register_in_flight_for_test(ChunkSpec {
        index: 1,
        start: 1024,
        end: 2047,
        etag: None,
    })
    .await;
    orch.cancel().await.unwrap();
    assert!(orch.is_aborted());
    let snapshot = orch.in_flight_snapshot().await;
    assert!(snapshot[0].cancel_request, "in_flight.cancel_request must be true after cancel");
    assert!(snapshot[0].abort_request, "in_flight.abort_request must be true after cancel");
}

#[tokio::test]
async fn etag_mismatch_short_circuits_to_full_restart() {
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

    let cfg = DownloadConfig::default();
    let orch = ChunkOrchestrator::new(cfg);
    let client = make_client();
    let chunks = orch.plan_chunks(1024, Some("v1".into()));
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("out.bin").to_string_lossy().to_string();
    let err = orch
        .run(
            &format!("{}/asset.bin", server.uri()),
            &file_path,
            chunks,
            &client,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, DownloadError::BackendEtagMismatch { .. }));
}

#[tokio::test]
async fn retry_exhausted_after_3_attempts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/asset.bin"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let cfg = DownloadConfig {
        initial_backoff_ms: 1,
        max_retries_per_chunk: 3,
        ..DownloadConfig::default()
    };
    let orch = ChunkOrchestrator::new(cfg);
    let client = make_client();
    let chunks = orch.plan_chunks(1024, Some("v1".into()));
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("out.bin").to_string_lossy().to_string();
    let err = orch
        .run(
            &format!("{}/asset.bin", server.uri()),
            &file_path,
            chunks,
            &client,
        )
        .await
        .unwrap_err();
    match err {
        DownloadError::RetryExhausted { attempts, .. } => assert_eq!(attempts, 3),
        other => panic!("expected RetryExhausted, got {other:?}"),
    }
}
