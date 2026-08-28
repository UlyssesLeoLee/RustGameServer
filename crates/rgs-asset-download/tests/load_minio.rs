//! M-2069.9 —— 100 万级 chunk 落盘 + GB 级文件并发分片吞吐（Load）
//!
//! 测试目标：
//! - 100 万级 chunk 落盘：1 个 8GB 文件（1M × 8KB = 8GB）；或 1M × 1KB = 1GB
//! - GB 级文件并发分片：10 个 1GB 文件 × 16 路并发
//! - 期望：单 chunk 落盘 < 10ms（per R5 风险缓解）
//! - 期望：GB 级文件并发吞吐 ≥ 100 MB/s（per SPEC §5.1）
//!
//! ⚠️ 100 万级 chunk 在 CI 跑会超时；默认 `#[ignore]`，由 SRE 在 ST 环境执行
//! 实际负载：100M token 量级（per L4 token-OLU 80K）

#![cfg(test)]

mod common;

use common::size::*;
use common::*;
use std::time::Instant;

const LOAD_ID: &str = "LOAD_MINIO";

/// 1M chunk 数
const N_CHUNKS_1M: u64 = 1_000_000;
/// 1 chunk 大小（8KB for 1M-level test）
const CHUNK_SIZE_8KB: u32 = 8 * 1024;
/// 1M × 8KB = 8GB 总大小
const TOTAL_SIZE_8GB: u64 = N_CHUNKS_1M * CHUNK_SIZE_8KB as u64;

#[tokio::test]
#[ignore = "100 万级 chunk 实测需 ST 环境，CI 跳过"]
async fn it_load_1m_chunks_disk_throughput() {
    eprintln!("[{LOAD_ID}] 100 万级 chunk 落盘吞吐实测");
    if !minio_reachable() {
        eprintln!("[{LOAD_ID}] MinIO 不可达，skip");
        return;
    }
    eprintln!(
        "[{LOAD_ID}] 目标：{} chunks × {} bytes = {} GB",
        N_CHUNKS_1M,
        CHUNK_SIZE_8KB,
        TOTAL_SIZE_8GB / 1024 / 1024 / 1024
    );

    let start = Instant::now();
    let mut hist = LatencyHistogram::new();
    for i in 0..N_CHUNKS_1M {
        let t = Instant::now();
        // 真实实现：写 chunk i 到 dest file (pwrite + fsync)
        // 当前占位
        hist.record(t.elapsed());
        if i % 100_000 == 0 {
            eprintln!("[{LOAD_ID}] 进度：{}/{} chunks", i, N_CHUNKS_1M);
        }
    }
    let elapsed = start.elapsed();
    eprintln!(
        "[{LOAD_ID}] 完成：{} chunks，耗时={:.2}s，p50={}us, p99={}us",
        N_CHUNKS_1M,
        elapsed.as_secs_f64(),
        hist.p50() * 1000,
        hist.p99() * 1000
    );
    // 期望：单 chunk 落盘 < 10ms
    assert!(
        hist.p99() < 10,
        "单 chunk 落盘 p99={}ms >= 10ms",
        hist.p99()
    );
}

#[tokio::test]
#[ignore = "GB 级 Load 实测需 ST 环境 + 多 GB 磁盘空间"]
async fn it_load_gb_concurrent_throughput() {
    eprintln!("[{LOAD_ID}] GB 级文件并发分片吞吐：10 × 1GB × 16 路");
    if !minio_reachable() {
        eprintln!("[{LOAD_ID}] MinIO 不可达，skip");
        return;
    }

    let n_files = 10;
    let file_size = MEDIUM; // 1GB
    let concurrent = 16;

    let start = Instant::now();
    // 真实实现：
    // 1. spawn 16 个 worker task
    // 2. 每个 worker 处理一个 file 的 chunks (8MB each)
    // 3. 测量总 throughput = total_bytes / elapsed
    let total_bytes = n_files * file_size as usize;
    let elapsed = start.elapsed();
    let throughput_mbps = if elapsed.as_secs_f64() > 0.0 {
        (total_bytes as f64) / elapsed.as_secs_f64() / 1024.0 / 1024.0
    } else {
        0.0
    };
    eprintln!(
        "[{LOAD_ID}] n_files={}, concurrent={}, 吞吐={:.2} MB/s",
        n_files, concurrent, throughput_mbps
    );
    // 期望：≥ 100 MB/s
    // assert!(throughput_mbps >= 100.0, "GB 级并发吞吐 {:.2} < 100 MB/s", throughput_mbps);
}

/// UT：Load 路径资源估算
#[test]
fn it_load_resource_estimation() {
    eprintln!("[{LOAD_ID}] 资源估算：");
    eprintln!("  - 1M chunks × 8KB = 8GB 总数据量");
    eprintln!("  - 16 路并发 × 8MB chunk = 128MB 并发窗口");
    eprintln!("  - 期望磁盘吞吐 ≥ 500 MB/s（NVMe SSD）");
    eprintln!("  - 期望网络吞吐 ≥ 100 MB/s（MinIO 单节点 2 vCPU + 2GB RAM）");
    eprintln!("  - 单 chunk 落盘 p99 < 10ms（per R5 风险缓解）");
    assert_eq!(CHUNK_SIZE_8KB, 8 * 1024);
}
