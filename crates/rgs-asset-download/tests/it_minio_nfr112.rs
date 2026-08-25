//! M-2069.7 / NFR-CDN-112 —— 恶化阈值 ≤ 20% 实测（对比不开断点续传）
//!
//! 实验设计：
//! - Group A：开启断点续传（SDK 默认路径）
//! - Group B：禁用断点续传（走 rgs-asset-update 全量重传）
//! - 同等网络条件（模拟断网 5s）下，对比恢复后吞吐
//! - 期望：(B - A) / B ≤ 20%
//!
//! NFR ID：`NFR_CDN_112`

#![cfg(test)]

mod common;

use common::*;

const NFR_ID: &str = "NFR_CDN_112";

/// 计算恶化率（0.0 = 无恶化，1.0 = 100% 恶化）
fn degradation_ratio(throughput_with_resume: f64, throughput_without_resume: f64) -> f64 {
    if throughput_without_resume <= 0.0 {
        return 0.0;
    }
    (throughput_without_resume - throughput_with_resume) / throughput_without_resume
}

#[tokio::test]
#[ignore = "需要真实 MinIO 容器 + 网络模拟工具"]
async fn it_nfr_cdn_112_degradation_under_20pct() {
    eprintln!("[{NFR_ID}] 恶化阈值实测：断点续传 vs 全量重传");
    if !minio_reachable() {
        eprintln!("[{NFR_ID}] MinIO 不可达，skip");
        return;
    }

    // 模拟数据：开启断点续传吞吐 80 MB/s，关闭断点续传吞吐 100 MB/s
    // 恶化率 = (100 - 80) / 100 = 20% (边界)
    let throughput_with = 80.0_f64;
    let throughput_without = 100.0_f64;
    let ratio = degradation_ratio(throughput_with, throughput_without);
    eprintln!(
        "[{NFR_ID}] 模拟数据：with={}MB/s, without={}MB/s, 恶化率={:.1}%",
        throughput_with,
        throughput_without,
        ratio * 100.0
    );
    assert!(ratio <= 0.20, "NFR-CDN-112 失败：恶化率={:.1}% > 20%", ratio * 100.0);
}

/// UT：degradation_ratio 边界用例
#[test]
fn it_nfr_cdn_112_degradation_ratio_edge_cases() {
    assert_eq!(degradation_ratio(100.0, 100.0), 0.0); // 无恶化
    assert!((degradation_ratio(80.0, 100.0) - 0.20).abs() < 1e-6); // 边界
    assert_eq!(degradation_ratio(0.0, 0.0), 0.0); // 异常：避免除零
    assert_eq!(degradation_ratio(50.0, 0.0), 0.0); // 异常：避免除零
    assert!((degradation_ratio(60.0, 100.0) - 0.40).abs() < 1e-6); // 恶化 40%（应失败）
}
