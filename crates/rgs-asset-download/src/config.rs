//! 配置（per SPEC-DTL-041 v0.2 §5.2 + RGS-IMPL-PLAN-CDN-001 §5.2）
//!
//! 参数目标 / 默认值（PH-3 实测回填）：
//! - `chunk_size_bytes = 8 * 1024 * 1024` (8MB，可在 4MB~16MB 调)
//! - `lru_max_bytes = 100 * 1024 * 1024` (100MB)
//! - `resume_token_ttl_days = 7`

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub chunk_size_bytes: u32,
    pub lru_max_bytes: u64,
    pub resume_token_ttl_days: u32,
    pub max_concurrent_chunks_desktop: u32,
    pub max_concurrent_chunks_mobile: u32,
    pub max_retries_per_chunk: u32,
    pub initial_backoff_ms: u32,
    /// NFR-CDN-110：恢复时延 p99 上限（实测目标 < 500ms）
    pub resume_latency_p99_ms: u32,
    /// NFR-CDN-112：恶化阈值（≤ 20%）
    pub throughput_degradation_threshold_pct: u32,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            chunk_size_bytes: 8 * 1024 * 1024,
            lru_max_bytes: 100 * 1024 * 1024,
            resume_token_ttl_days: 7,
            max_concurrent_chunks_desktop: 16,
            max_concurrent_chunks_mobile: 4,
            max_retries_per_chunk: 3,
            initial_backoff_ms: 100,
            resume_latency_p99_ms: 500,
            throughput_degradation_threshold_pct: 20,
        }
    }
}
