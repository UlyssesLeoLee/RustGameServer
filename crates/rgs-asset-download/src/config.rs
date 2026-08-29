//! 运行时配置（per SPEC §2.3 + IMPL-PLAN §5.2 实测参数回填位）。
//!
//! v0.1 阶段：参数给到安全默认值，**不**做运行时自适应（PH-3 实测后回填到 `Default::default()`）。
//! 默认值来源：IMPL-PLAN §5.2（chunk 8MB / LRU 100MB / TTL 7 天 / 桌面 ≤ 16 路 / 移动 ≤ 4 路）。
//!
//! 关键约束：
//! - 不在配置里塞 PII 字段（FR-CDN-064）；如有需要由调用方在 `DownloadRequest` 层注入。
//! - `PlatformProfile` 决定并发上限；与 `rgs-version` 协商出来的 `is_mobile` 标志配合使用。

use serde::{Deserialize, Serialize};

/// 平台画像（per SPEC §3 / IMPL-PLAN §6 R1 移动 ≤ 4 / 桌面 ≤ 16）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlatformProfile {
    /// 桌面（macOS 14 / Windows 11 / Linux 桌面）
    #[default]
    Desktop,
    /// 移动（iOS 17 / Android 14）
    Mobile,
    /// 服务器 / CI（无并发限制外暴露；用于测压）
    Server,
}

impl PlatformProfile {
    /// 当前平台画像允许的最大并发 Range 请求数。
    /// 桌面 ≤ 16 / 移动 ≤ 4 / 服务器 = 32（per IMPL-PLAN §3.3 + SPEC §3）。
    pub const fn max_concurrent_ranges(self) -> usize {
        match self {
            Self::Desktop => 16,
            Self::Mobile => 4,
            Self::Server => 32,
        }
    }
}

/// 下载运行时配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// 平台画像（决定并发上限 + 预分配策略）
    pub platform_profile: PlatformProfile,
    /// 分片粒度（4~16 MB；默认 8 MB；per IMPL-PLAN §5.2）
    pub chunk_size_bytes: u64,
    /// 单 chunk 最大重试次数（默认 3；per SPEC §3）
    pub max_retries_per_chunk: u32,
    /// 指数退避初始延迟（毫秒；默认 100ms；per SPEC §3）
    pub initial_backoff_ms: u64,
    /// 单次 Range 请求超时（秒；默认 30s；过大文件按 chunk 数线性放大）
    pub range_request_timeout_secs: u64,
    /// 断点记录 TTL（天；默认 7；per IMPL-PLAN §5.2）
    pub resume_token_ttl_days: u32,
    /// LRU store 上限（字节；默认 100 MB；per IMPL-PLAN §5.2）
    pub lru_max_bytes: u64,
    /// 单次下载允许的最大文件大小（字节；默认 16 GiB；防止误用拖垮磁盘）
    pub max_file_size_bytes: u64,
    /// User-Agent 标识（不含 PII；不带 device_id / player_id / ip）
    pub user_agent: String,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            platform_profile: PlatformProfile::Desktop,
            chunk_size_bytes: 8 * 1024 * 1024,
            max_retries_per_chunk: 3,
            initial_backoff_ms: 100,
            range_request_timeout_secs: 30,
            resume_token_ttl_days: 7,
            lru_max_bytes: 100 * 1024 * 1024,
            max_file_size_bytes: 16 * 1024 * 1024 * 1024,
            user_agent: format!("rgs-asset-download/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl DownloadConfig {
    /// 按平台画像收紧并发数（返回 `min(configured, platform_max)`）。
    pub fn effective_max_concurrent(&self) -> usize {
        let platform_cap = self.platform_profile.max_concurrent_ranges();
        // `max_retries_per_chunk` 留作字段，但默认不参与并发计算
        platform_cap
    }

    /// 按分片粒度计算分片数量（向上取整）。
    pub fn chunk_count_for(&self, total_size: u64) -> u64 {
        if total_size == 0 {
            return 0;
        }
        let cs = self.chunk_size_bytes.max(1);
        total_size.div_ceil(cs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_safe() {
        let c = DownloadConfig::default();
        assert_eq!(c.chunk_size_bytes, 8 * 1024 * 1024);
        assert_eq!(c.lru_max_bytes, 100 * 1024 * 1024);
        assert_eq!(c.resume_token_ttl_days, 7);
        assert_eq!(c.platform_profile, PlatformProfile::Desktop);
        // 移动 ≤ 4 / 桌面 ≤ 16 / 服务器 = 32
        assert_eq!(PlatformProfile::Mobile.max_concurrent_ranges(), 4);
        assert_eq!(PlatformProfile::Desktop.max_concurrent_ranges(), 16);
        assert_eq!(PlatformProfile::Server.max_concurrent_ranges(), 32);
    }

    #[test]
    fn chunk_count_for_uses_ceil() {
        let c = DownloadConfig::default(); // 8 MB
        assert_eq!(c.chunk_count_for(0), 0);
        assert_eq!(c.chunk_count_for(1), 1);
        assert_eq!(c.chunk_count_for(8 * 1024 * 1024), 1);
        assert_eq!(c.chunk_count_for(8 * 1024 * 1024 + 1), 2);
        assert_eq!(c.chunk_count_for(16 * 1024 * 1024), 2);
    }
}
