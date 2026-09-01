//! rgs-asset-download 跨模块集成场景 (per 9/1 PT-WORKER 派工 §3 IT)
//!
//! 3 跨场景：
//! 1. ResumeToken + IntegrityGate 串联: 创建断点 → 模拟落盘文件 → verify 校验通过
//! 2. ChunkOrchestrator::plan_chunks + 落盘验证: 写多个 chunk 到临时文件, 拼成完整文件并 hash 校验
//! 3. DownloadStateMachine + ResumeToken 状态同步: Pause → Resume → Completed 生命周期

use std::path::PathBuf;

use rgs_asset_download::chunk_orchestrator::ChunkOrchestrator;
use rgs_asset_download::config::DownloadConfig;
use rgs_asset_download::integrity_gate::{IntegrityGate, IntegrityStatus};
use rgs_asset_download::resume_token::{ResumeToken, RESUME_TOKEN_TTL_DAYS};
use rgs_asset_download::state_machine::{DownloadState, DownloadStateMachine, StateEvent};

fn sha256_hex(payload: &[u8]) -> String {
    IntegrityGate::hash_bytes(payload)
}

#[tokio::test]
async fn it_resume_token_integrity_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("asset.bin");
    let payload = b"cross-scenario-resume-integrity-payload-v1";
    tokio::fs::write(&path, payload).await.unwrap();
    let expected = sha256_hex(payload);
    // 1) 创建断点
    let mut token = ResumeToken::new(
        "asset-cross-1",
        path.clone(),
        payload.len() as u64,
        8 * 1024 * 1024,
        "\"abc-etag-001\"",
        expected.clone(),
        "https://cdn.example.com/asset-cross-1.bin",
    )
    .expect("token new");
    // 2) 标记 chunk 0 + 切到 Downloading
    token.mark_chunk_completed(0);
    token.set_status(DownloadState::Downloading);
    assert_eq!(token.completed_chunks, vec![0]);
    assert_eq!(token.status, DownloadState::Downloading);
    // 3) verify
    let gate = IntegrityGate::new();
    let report = gate
        .verify(path.to_str().unwrap(), &expected)
        .await
        .unwrap();
    assert_eq!(report.status, IntegrityStatus::Match);
    assert_eq!(report.size_bytes, payload.len() as u64);
}

#[tokio::test]
async fn it_chunk_plan_then_reassemble_matches_hash() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("reassembled.bin");
    // 假装 total_size=1000, chunk_size=300 → 4 个 chunk (300, 300, 300, 100)
    let cfg = DownloadConfig {
        chunk_size_bytes: 300,
        ..DownloadConfig::default()
    };
    let orch = ChunkOrchestrator::new(cfg);
    let chunks = orch.plan_chunks(1000, Some("\"e1\"".to_string()));
    assert_eq!(chunks.len(), 4);
    // 模拟 4 个 chunk 数据, 拼成 1000 字节
    let mut blob = Vec::new();
    for i in 0..4 {
        let len = chunks[i].len() as usize;
        blob.extend(std::iter::repeat(b'a' + (i as u8)).take(len));
    }
    assert_eq!(blob.len(), 1000);
    // 写文件
    tokio::fs::write(&out, &blob).await.unwrap();
    // verify
    let expected = sha256_hex(&blob);
    let gate = IntegrityGate::new();
    let report = gate.verify(out.to_str().unwrap(), &expected).await.unwrap();
    assert_eq!(report.status, IntegrityStatus::Match);
}

#[tokio::test]
async fn it_state_machine_pause_resume_completed_lifecycle() {
    let mut sm = DownloadStateMachine::new();
    // 1) Idle → Resolving → Downloading → Paused
    assert_eq!(sm.apply(StateEvent::ResolveStart).unwrap(), DownloadState::Resolving);
    assert_eq!(sm.apply(StateEvent::ResolveSuccess).unwrap(), DownloadState::Downloading);
    assert_eq!(sm.apply(StateEvent::Pause).unwrap(), DownloadState::Paused);
    assert!(sm.is_terminal(), "Paused 是准终态");
    assert!(sm.cancel_flag().load(std::sync::atomic::Ordering::SeqCst));
    // 2) Paused → Resume → Downloading (cancel_flag 重置)
    assert_eq!(sm.apply(StateEvent::Resume).unwrap(), DownloadState::Downloading);
    assert!(!sm.cancel_flag().load(std::sync::atomic::Ordering::SeqCst));
    // 3) Downloading → Complete
    assert_eq!(sm.apply(StateEvent::Complete).unwrap(), DownloadState::Completed);
    assert!(sm.is_terminal());
    // 4) Completed 拒绝所有事件
    for ev in [StateEvent::ResolveStart, StateEvent::Pause, StateEvent::Cancel, StateEvent::Retry] {
        assert!(sm.apply(ev).is_err(), "Completed should reject {ev:?}");
    }
    // 5) ResumeToken 7 天 TTL 不变式
    let token = ResumeToken::new(
        "asset-lifecycle",
        PathBuf::from("/tmp/asset-lifecycle.bin"),
        1024,
        256,
        "\"e2\"",
        "a".repeat(64),
        "https://example.com",
    )
    .unwrap();
    let delta = token.expires_at - token.created_at;
    assert_eq!(delta, chrono::Duration::days(RESUME_TOKEN_TTL_DAYS));
}
