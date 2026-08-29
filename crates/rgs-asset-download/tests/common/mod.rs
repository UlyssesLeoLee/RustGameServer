//! IT 测试公共模块（per RGS-IMPL-PLAN-CDN-001 v0.1 §3.4）
//!
//! 提供：
//! - MinIO container fixture（feature-gated，参照 rgs-testkit testcontainers 模式）
//! - wiremock Range server helper
//! - 资源生成（test asset factory）
//! - Latency histogram（per NFR-CDN-110 p99）

#![allow(dead_code)]

use std::path::PathBuf;
use std::time::Duration;

/// MinIO 端点（PH-4 实测时由 docker compose 启动）
pub const MINIO_ENDPOINT: &str = "http://127.0.0.1:9000";
pub const MINIO_ACCESS_KEY: &str = "rustgameserver";
pub const MINIO_SECRET_KEY: &str = "rustgameserver";
pub const MINIO_BUCKET: &str = "asset-bundle";

/// wiremock 默认端口
pub const WIREMOCK_DEFAULT_PORT: u16 = 9090;

/// 资源生成大小（per IT 实测规格）
pub mod size {
    /// 100 MB 测试 asset
    pub const SMALL: u64 = 100 * 1024 * 1024;
    /// 1 GB 测试 asset
    pub const MEDIUM: u64 = 1024 * 1024 * 1024;
    /// 5 GB 测试 asset（GB 级 Load）
    pub const LARGE: u64 = 5 * 1024 * 1024 * 1024;
}

/// 测试资源数（per AC-CDN-110 实测规格 1000 资源）
pub const N_RESOURCES_SMOKE: usize = 10;
pub const N_RESOURCES_FULL: usize = 1000;

/// Chunk 大小（per RGS-IMPL-PLAN-CDN-001 §5.2）
pub const CHUNK_SIZE_8MB: u32 = 8 * 1024 * 1024;
pub const CHUNK_SIZE_4MB: u32 = 4 * 1024 * 1024;
pub const CHUNK_SIZE_16MB: u32 = 16 * 1024 * 1024;

/// 4 平台列表（per SPEC-DTL-041 §2 + AC-CDN-113）
pub const PLATFORMS: &[&str] = &["iOS-17", "Android-14", "Windows-11", "macOS-14"];

/// 生成测试 asset（伪随机内容，便于 ETag 校验）
pub fn make_test_asset(size: u64, seed: u32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(size as usize);
    let mut state = seed.wrapping_add(0x9E37_79B9);
    while buf.len() < size as usize {
        // xorshift 32-bit
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let bytes = state.to_le_bytes();
        buf.extend_from_slice(&bytes);
    }
    buf.truncate(size as usize);
    buf
}

/// 计算 SHA-256 hex（per IntegrityGate 校验）
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// 检查 MinIO 是否可达（IT setup 阶段用）
pub fn minio_reachable() -> bool {
    std::net::TcpStream::connect_timeout(
        &"127.0.0.1:9000".parse().unwrap(),
        Duration::from_millis(500),
    )
    .is_ok()
}

/// 临时目录
pub fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rgs-asset-download-it-{name}"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// 简易 latency 统计（per NFR-CDN-110 p99）
pub struct LatencyHistogram {
    samples_ms: Vec<u64>,
}

impl LatencyHistogram {
    pub fn new() -> Self {
        Self {
            samples_ms: Vec::new(),
        }
    }
    pub fn record(&mut self, d: Duration) {
        self.samples_ms.push(d.as_millis() as u64);
    }
    pub fn p50(&self) -> u64 {
        self.percentile(50.0)
    }
    pub fn p99(&self) -> u64 {
        self.percentile(99.0)
    }
    pub fn len(&self) -> usize {
        self.samples_ms.len()
    }
    pub fn is_empty(&self) -> bool {
        self.samples_ms.is_empty()
    }
    fn percentile(&self, p: f64) -> u64 {
        if self.samples_ms.is_empty() {
            return 0;
        }
        let mut sorted = self.samples_ms.clone();
        sorted.sort_unstable();
        let rank = (p / 100.0 * (sorted.len() as f64 - 1.0)).round() as usize;
        sorted[rank.min(sorted.len() - 1)]
    }
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}
