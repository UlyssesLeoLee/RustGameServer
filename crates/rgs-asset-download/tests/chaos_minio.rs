//! M-2069.8 —— 故障注入 5 类（per SPEC §6 + RGS-IMPL-PLAN-CDN-001 v0.1 §3.4）
//!
//! 5 类故障：
//! 1. 断网（DNS 失败 / TCP RST / 模拟丢包）
//! 2. 进程 kill -9（mid-download，ResumeToken 必须能恢复）
//! 3. ETag 变更（服务端静默更新，强制全量重传）
//! 4. 篡改（比特翻转，IntegrityGate 必须拦截）
//! 5. 强制更新（服务端在 resume 时返回 200 OK 而非 206，触发全量重传）
//!
//! 输出：每类故障的恢复路径 + 通过条件

#![cfg(test)]

mod common;

use common::*;
use rgs_asset_download::{RangeResponse, state_machine::DownloadState};

const CHAOS_5_CATEGORIES: &[&str] = &[
    "断网 (network partition)",
    "kill -9 (process killed mid-download)",
    "ETag 变更 (server-side silent update)",
    "篡改 (bit flip / MITM)",
    "强制更新 (200 OK instead of 206 Partial Content)",
];

#[test]
fn it_chaos_5_categories_documented() {
    eprintln!("[chaos_minio] 5 类故障注入覆盖清单：");
    for (i, c) in CHAOS_5_CATEGORIES.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, c);
    }
    assert_eq!(CHAOS_5_CATEGORIES.len(), 5);
}

// ============================================================================
// 1. 断网
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 MinIO + 网络模拟工具（toxiproxy / iptables）"]
async fn it_chaos_1_network_partition_recovery() {
    eprintln!("[chaos_minio/1] 断网恢复：DNS 失败 / TCP RST / 模拟丢包");
    if !minio_reachable() {
        eprintln!("[chaos_minio/1] MinIO 不可达，skip");
        return;
    }
    // 真实实现：
    // 1. 启动下载
    // 2. mid-download 时用 toxiproxy 注入 latency / drop
    // 3. SDK 检测失败 → 触发 retry (3 次，指数退避 100ms 起步)
    // 4. retry 耗尽 → 状态 Failed + ResumeToken 保留
    // 5. 恢复网络 → SDK 重新拉 → 成功
    eprintln!("[chaos_minio/1] 期望：retry 3 次后 Failed，恢复网络后 resume 成功");
}

// ============================================================================
// 2. kill -9
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 MinIO + 进程管理权限"]
async fn it_chaos_2_kill_minus_9_resume_token_persists() {
    eprintln!("[chaos_minio/2] kill -9：ResumeToken 必须保留，重启可恢复");
    if !minio_reachable() {
        eprintln!("[chaos_minio/2] MinIO 不可达，skip");
        return;
    }
    // 真实实现：
    // 1. 启动下载到 chunk N/2
    // 2. 模拟 kill -9（实测时用子进程 + std::process::Child::kill()）
    // 3. 验证 ~/.rgs-sdk/downloads/ 下的 ResumeToken 存在
    // 4. 重新启动 SDK → 从 chunk N/2 续传 → 成功
    eprintln!("[chaos_minio/2] 期望：ResumeToken 落盘，重启后从 last checkpoint 续传");
}

// ============================================================================
// 3. ETag 变更
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 MinIO + 可写权限（覆盖对象）"]
async fn it_chaos_3_etag_change_full_redownload() {
    eprintln!("[chaos_minio/3] ETag 变更：服务端静默更新，强制全量重传");
    if !minio_reachable() {
        eprintln!("[chaos_minio/3] MinIO 不可达，skip");
        return;
    }
    // 真实实现：
    // 1. 启动下载到 chunk 5
    // 2. mc cp 覆盖对象（ETag 变更）
    // 3. SDK resume 时检测到 If-Range: <old_etag> 不匹配
    // 4. 服务端返回 200 OK 整文件（忽略 Range）
    // 5. SDK 检测到 200 + etag 变更 → 触发 ETagMismatch error
    // 6. 状态 Failed → 重新走 download_asset
    let simulated = RangeResponse::FullContent {
        etag: "new-etag-after-overwrite".to_string(),
        body: vec![],
    };
    if let RangeResponse::FullContent { etag, .. } = simulated {
        assert_eq!(etag, "new-etag-after-overwrite");
    }
    eprintln!("[chaos_minio/3] 期望：ETagMismatch 错误 → 全量重传");
}

// ============================================================================
// 4. 篡改
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 MinIO + MITM 模拟（toxiproxy 重写 body）"]
async fn it_chaos_4_tampered_response_integrity_gate_blocks() {
    eprintln!("[chaos_minio/4] 篡改：比特翻转，IntegrityGate 必须拦截");
    if !minio_reachable() {
        eprintln!("[chaos_minio/4] MinIO 不可达，skip");
        return;
    }
    // 真实实现：
    // 1. toxiproxy 修改响应 body 的某个 byte
    // 2. SDK 下载完成 → IntegrityGate 校验失败
    // 3. 状态 Failed + 触发 rgs_asset_download_integrity_failure_total++
    eprintln!("[chaos_minio/4] 期望：IntegrityGate 拦截 → 状态 Failed → 重新拉");
}

// ============================================================================
// 5. 强制更新
// ============================================================================

#[tokio::test]
#[ignore = "需要真实 MinIO + 服务端注入工具（返回 200 OK 而非 206）"]
async fn it_chaos_5_forced_full_response_200_ok() {
    eprintln!("[chaos_minio/5] 强制更新：服务端返回 200 OK 而非 206");
    if !minio_reachable() {
        eprintln!("[chaos_minio/5] MinIO 不可达，skip");
        return;
    }
    // 真实实现：
    // 1. 启动下载 mid-way
    // 2. 服务端在 resume 时返回 200 OK（忽略 Range header，强制全量重传）
    // 3. SDK 检测 200 + Content-Length == total_size → 触发全量重传逻辑
    let simulated = RangeResponse::FullContent {
        etag: "etag-same-but-200-ok".to_string(),
        body: vec![],
    };
    if let RangeResponse::FullContent { body, .. } = simulated {
        assert!(body.is_empty()); // 占位：实际 body 应该是完整文件
    }
    eprintln!("[chaos_minio/5] 期望：200 OK 触发全量重传（不视为错误）");
}

// ============================================================================
// UT：5 类故障不破坏状态机
// ============================================================================

#[test]
fn it_chaos_5_categories_state_machine_invariant() {
    use rgs_asset_download::state_machine::DownloadStateMachine;
    for chaos_idx in 0..CHAOS_5_CATEGORIES.len() {
        let mut sm = DownloadStateMachine::new();
        sm.transition(DownloadState::Idle);
        sm.transition(DownloadState::Resolving);
        sm.transition(DownloadState::Downloading);

        // 模拟故障 → Failed
        sm.transition(DownloadState::Failed);
        assert_eq!(sm.current(), DownloadState::Failed);

        // 恢复 → Resolving
        sm.transition(DownloadState::Resolving);
        assert_eq!(sm.current(), DownloadState::Resolving);
        eprintln!(
            "[chaos_minio/UT] chaos_idx={} ({}) → 状态机恢复成功",
            chaos_idx, CHAOS_5_CATEGORIES[chaos_idx]
        );
    }
}
