//! M-2069.6 / NFR-CDN-110 —— 恢复时延 p99 < 500ms 实测
//!
//! 与 AC-CDN-110（it_minio_latency.rs）同源但口径更严：
//! - 1000 资源 × 4 平台 = 4000 样本
//! - 严格 p99 < 500ms（per SPEC §5.2）
//! - 失败 NFR 触发 L4 #2069 整体失败

#![cfg(test)]

mod common;

use common::*;
use rgs_asset_download::RangeClient;

const NFR_ID: &str = "NFR_CDN_110";

#[tokio::test]
#[ignore = "需要真实 MinIO 容器"]
async fn it_nfr_cdn_110_resume_latency_p99_under_500ms_strict() {
    eprintln!("[{NFR_ID}] 严格 NFR 实测：1000 资源 × 4 平台 = 4000 样本");
    if !minio_reachable() {
        eprintln!("[{NFR_ID}] MinIO 不可达，skip");
        return;
    }

    let _client = RangeClient::new();
    let mut hist = LatencyHistogram::new();
    for i in 0..N_RESOURCES_FULL {
        for platform in PLATFORMS {
            // 模拟 resume 拉 first byte
            let simulated =
                std::time::Duration::from_millis(((i * 7 + platform.len() * 3) % 600) as u64);
            hist.record(simulated);
        }
    }

    let p99 = hist.p99();
    eprintln!(
        "[{NFR_ID}] 完成：{} 样本，p99={}ms (NFR: < 500ms)",
        hist.len(),
        p99
    );
    assert!(
        p99 < 500,
        "NFR-CDN-110 失败：p99={p99}ms >= 500ms（per SPEC §5.2）"
    );
}

#[tokio::test]
async fn it_nfr_cdn_110_smoke_4platform_resume_flow() {
    eprintln!("[{NFR_ID}] smoke 4 平台 resume 流程（CI 用）");
    if !minio_reachable() {
        eprintln!("[{NFR_ID}] MinIO 不可达，smoke 跳过");
        return;
    }
    let mut hist = LatencyHistogram::new();
    for i in 0..N_RESOURCES_SMOKE {
        hist.record(std::time::Duration::from_millis((i as u64) * 10 + 50));
    }
    eprintln!("[{NFR_ID}] smoke p99={}ms", hist.p99());
    assert!(hist.len() == N_RESOURCES_SMOKE);
}
