//! DownloadConfig —— 下载子系统运行时配置。
//!
//! per **RGS-SPEC-DTL-041 v0.2 §5.2** + **RGS-IMPL-PLAN-CDN-001 v0.1 §7.3**。
//!
//! 三个核心参数的默认值采用**保守基线**，等 PH-3 / PH-4 实测（per RGS-SPEC-DTL-041
//! §8 Gate 证据）后回填：
//!
//! | 参数 | 保守默认值 | 实测位置 | 目标 |
//! |---|---|---|---|
//! | `chunk_size_bytes` | 8 MiB | M-2065.3 | 4~16 MiB |
//! | `lru_max_bytes` | 100 MiB | M-2064.5 | 100 MiB（NFR-CDN-113）|
//! | `resume_token_ttl_days` | 7 天 | M-2064.3 | 7 天（FR-CDN-063）|
//!
//! # 硬约束绑定
//!
//! - **NFR-CDN-002**：本配置不暴露任何"关闭整文件校验"的开关。
//! - **NFR-CDN-110/112**：恢复时延 p99 < 500ms / 恶化阈值 ≤ 20% 是实测门禁
//!   （per RGS-IMPL-PLAN-CDN-001 v0.1 §5.2），不通过本配置绕过。
//! - **NFR-CDN-114**：DistributionBackend 后端选型门禁；本配置不绕过。
//!
//! # 关联规范
//!
//! - RGS-SPEC-DTL-041 v0.2 §5.2（实测参数回填）
//! - RGS-IMPL-PLAN-CDN-001 v0.1 §7.3（回滚策略默认值）
//! - RGS-IMPL-PLAN-CDN-001 v0.1 §3.1 M-2063.5（本任务）

/// 客户端资源下载子系统的运行时配置。
///
/// 所有字段都有保守默认值，可在启动时通过 `DownloadConfig::with_overrides` 覆盖；
/// **不**提供关闭 `IntegrityGate` 或绕过整文件校验的开关（per NFR-CDN-002 硬约束）。
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 单 chunk 大小（bytes）。
    ///
    /// 保守默认 8 MiB；实测范围 4~16 MiB（per SPEC §5.2）。
    /// MinIO 默认单 chunk ≤ 5 GiB 上限（per 实施计划 §6 R4 风险），8 MiB 远低于。
    pub chunk_size_bytes: u64,

    /// 断点记录存储 LRU 上限（bytes）。
    ///
    /// 保守默认 100 MiB；NFR-CDN-113 门禁 = 100 MiB。
    /// 触发后按 `last_updated_at` 升序淘汰（per 实施计划 §6 R7 风险）。
    pub lru_max_bytes: u64,

    /// 断点记录过期阈值（天）。
    ///
    /// 保守默认 7 天；FR-CDN-063 = 7 天。
    /// 启动时 `ResumeTokenStore::cleanup_expired` 按此阈值清理（per DTL §4.3）。
    pub resume_token_ttl_days: i64,

    /// 桌面平台 Range 并发分片数（per SPEC §3 背压：桌面 ≤ 16）。
    ///
    /// 保守默认 8 路；实测范围 4~16 路。
    pub desktop_concurrency: u32,

    /// 移动平台 Range 并发分片数（per SPEC §3 背压：移动 ≤ 4）。
    ///
    /// 保守默认 4 路。
    pub mobile_concurrency: u32,
}

impl Default for DownloadConfig {
    /// 保守默认值；与 RGS-IMPL-PLAN-CDN-001 v0.1 §7.3 回滚策略默认值一致。
    fn default() -> Self {
        Self {
            chunk_size_bytes: 8 * 1024 * 1024, // 8 MiB
            lru_max_bytes: 100 * 1024 * 1024, // 100 MiB
            resume_token_ttl_days: 7,
            desktop_concurrency: 8,
            mobile_concurrency: 4,
        }
    }
}

impl DownloadConfig {
    /// 构造默认配置（per SPEC §5.2 / 实施计划 §7.3 保守基线）。
    pub fn new() -> Self {
        Self::default()
    }

    /// 用覆盖值构造配置；**不**允许通过此 API 关闭整文件校验。
    ///
    /// 调用方应对 `chunk_size_bytes` / `lru_max_bytes` 落入 SPEC §5.2 实测范围
    /// 做合法性校验（PH-3 实测代码补全，本骨架仅占位）。
    pub fn with_overrides(
        chunk_size_bytes: Option<u64>,
        lru_max_bytes: Option<u64>,
        resume_token_ttl_days: Option<i64>,
    ) -> Self {
        let defaults = Self::default();
        Self {
            chunk_size_bytes: chunk_size_bytes.unwrap_or(defaults.chunk_size_bytes),
            lru_max_bytes: lru_max_bytes.unwrap_or(defaults.lru_max_bytes),
            resume_token_ttl_days: resume_token_ttl_days
                .unwrap_or(defaults.resume_token_ttl_days),
            desktop_concurrency: defaults.desktop_concurrency,
            mobile_concurrency: defaults.mobile_concurrency,
        }
    }
}
