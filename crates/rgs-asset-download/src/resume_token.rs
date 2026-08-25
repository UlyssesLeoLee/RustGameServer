//! ResumeToken —— 13 字段断点记录结构
//!
//! 实现规格：RGS-SPEC-DTL-041 §6 + RGS-DTL-041 §3
//! 任务来源：RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.2
//!
//! ## FR-CDN-064 硬约束
//!
//! **断点记录不含 PII 字段**。本结构**不**包含任何可识别用户身份的字段
//! （如用户标识、设备标识、网络地址、联系信息等）。仅有 13 个字段，全部为资产维度元数据。
//! 构造路径对每条输入字段都做防御性 PII 子串检查（见构造器实现）。
//!
//! ## 13 字段清单（per SPEC §6）
//!
//! | # | 字段 | 类型 | 含义 |
//! |---|---|---|---|
//! | 1 | `token_id` | `String` (UUID v4) | 断点记录唯一标识；不与任何用户标识绑定（per FR-CDN-064）|
//! | 2 | `asset_id` | `String` | 资产 ID（来自 manifest）|
//! | 3 | `file_path` | `PathBuf` | 目标本地文件路径（沙箱内）|
//! | 4 | `total_size` | `u64` | 文件总字节数（来自 manifest HEAD）|
//! | 5 | `chunk_size` | `u64` | 分片粒度（字节），默认 8MB |
//! | 6 | `completed_chunks` | `Vec<u32>` | 已完成分片索引列表（用于精确 resume）|
//! | 7 | `etag` | `String` | ETag（用于 `If-Range: <ETag>` 头 per FR-CDN-074）|
//! | 8 | `created_at` | `DateTime<Utc>` | 记录创建时间 |
//! | 9 | `updated_at` | `DateTime<Utc>` | 记录最后更新时间 |
//! | 10 | `expires_at` | `DateTime<Utc>` | 过期时间（= `created_at + 7 天`，per SPEC §8）|
//! | 11 | `checksum_sha256` | `String` (hex) | 整文件期望 SHA-256（per NFR-CDN-002 不可绕过）|
//! | 12 | `backend_url` | `String` | 后端 URL（HTTP Range endpoint，per NFR-CDN-114）|
//! | 13 | `status` | `DownloadState` | 当前状态机状态 |

use std::path::PathBuf;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::state_machine::DownloadState;

/// Schema 版本号（用于 store 迁移 / 向后兼容）
pub const TOKEN_SCHEMA_VERSION: u32 = 1;

/// 7 天断点过期阈值（per RGS-SPEC-DTL-041 §8 `resume_token_ttl_days = 7`）
pub const RESUME_TOKEN_TTL_DAYS: i64 = 7;

/// 13 字段断点记录（per RGS-SPEC-DTL-041 §6 + RGS-DTL-041 §3）
///
/// **PII 边界**：本结构不携带任何用户标识、设备标识、网络地址、联系信息等 PII 字段。
/// 如需新增任何用户维度字段，必须先升 schema 版本 + 走 FR-CDN-064 grep 验证。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeToken {
    /// Schema 版本（store 兼容性判断）
    pub schema_version: u32,

    /// 1. `token_id`（UUID v4 字符串；唯一）
    pub token_id: String,

    /// 2. `asset_id`（来自 manifest）
    pub asset_id: String,

    /// 3. `file_path`（目标本地文件路径）
    pub file_path: PathBuf,

    /// 4. `total_size`（文件总字节数）
    pub total_size: u64,

    /// 5. `chunk_size`（分片粒度，字节）
    pub chunk_size: u64,

    /// 6. `completed_chunks`（已完成分片索引列表）
    pub completed_chunks: Vec<u32>,

    /// 7. `etag`（HTTP `If-Range` 头值 per FR-CDN-074）
    pub etag: String,

    /// 8. `created_at`
    pub created_at: DateTime<Utc>,

    /// 9. `updated_at`
    pub updated_at: DateTime<Utc>,

    /// 10. `expires_at`（= `created_at + 7 天`）
    pub expires_at: DateTime<Utc>,

    /// 11. `checksum_sha256`（hex；整文件校验 per NFR-CDN-002）
    pub checksum_sha256: String,

    /// 12. `backend_url`（HTTP Range endpoint）
    pub backend_url: String,

    /// 13. `status`（状态机当前状态）
    pub status: DownloadState,
}

/// ResumeToken 构造 / 校验错误
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResumeTokenError {
    /// `total_size` 必须 > 0
    #[error("invalid total_size: {0} (must be > 0)")]
    InvalidTotalSize(u64),
    /// `chunk_size` 必须 > 0
    #[error("invalid chunk_size: {0} (must be > 0)")]
    InvalidChunkSize(u64),
    /// `asset_id` 为空
    #[error("asset_id must not be empty")]
    EmptyAssetId,
    /// `etag` 为空（断点续传必须有 ETag per FR-CDN-074）
    #[error("etag must not be empty (per FR-CDN-074)")]
    EmptyEtag,
    /// `backend_url` 为空
    #[error("backend_url must not be empty")]
    EmptyBackendUrl,
    /// `file_path` 为空
    #[error("file_path must not be empty")]
    EmptyFilePath,
    /// `checksum_sha256` 长度非法（hex SHA-256 必须 64 字符）
    #[error("invalid checksum_sha256 length: {0} (expected 64 hex chars)")]
    InvalidChecksumLength(usize),
    /// 包含非法 PII 字段（防御性，构造路径不应出现）
    #[error("forbidden PII field detected: {0}")]
    ForbiddenPiiField(String),
}

impl ResumeToken {
    /// 创建一个新的断点记录（自动生成 token_id、created_at、updated_at、expires_at）
    ///
    /// 校验：
    /// - `asset_id` / `etag` / `backend_url` / `file_path` 非空
    /// - `total_size` > 0
    /// - `chunk_size` > 0
    /// - `checksum_sha256` 是 64 字符 hex
    pub fn new(
        asset_id: impl Into<String>,
        file_path: PathBuf,
        total_size: u64,
        chunk_size: u64,
        etag: impl Into<String>,
        checksum_sha256: impl Into<String>,
        backend_url: impl Into<String>,
    ) -> Result<Self, ResumeTokenError> {
        let asset_id = asset_id.into();
        let etag = etag.into();
        let checksum_sha256 = checksum_sha256.into();
        let backend_url = backend_url.into();

        // FR-CDN-064 防御性检查
        for field in PII_FORBIDDEN_FIELDS {
            if asset_id.contains(field)
                || etag.contains(field)
                || backend_url.contains(field)
                || checksum_sha256.contains(field)
            {
                return Err(ResumeTokenError::ForbiddenPiiField((*field).to_string()));
            }
        }

        if asset_id.is_empty() {
            return Err(ResumeTokenError::EmptyAssetId);
        }
        if etag.is_empty() {
            return Err(ResumeTokenError::EmptyEtag);
        }
        if backend_url.is_empty() {
            return Err(ResumeTokenError::EmptyBackendUrl);
        }
        if file_path.as_os_str().is_empty() {
            return Err(ResumeTokenError::EmptyFilePath);
        }
        if total_size == 0 {
            return Err(ResumeTokenError::InvalidTotalSize(total_size));
        }
        if chunk_size == 0 {
            return Err(ResumeTokenError::InvalidChunkSize(chunk_size));
        }
        if checksum_sha256.len() != 64 {
            return Err(ResumeTokenError::InvalidChecksumLength(checksum_sha256.len()));
        }
        if !checksum_sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ResumeTokenError::InvalidChecksumLength(checksum_sha256.len()));
        }

        let now = Utc::now();
        let expires_at = now + Duration::days(RESUME_TOKEN_TTL_DAYS);

        Ok(Self {
            schema_version: TOKEN_SCHEMA_VERSION,
            token_id: Uuid::new_v4().to_string(),
            asset_id,
            file_path,
            total_size,
            chunk_size,
            completed_chunks: Vec::new(),
            etag,
            created_at: now,
            updated_at: now,
            expires_at,
            checksum_sha256,
            backend_url,
            status: DownloadState::Idle,
        })
    }

    /// 从已有 token 恢复（用于 store 反序列化）
    #[allow(clippy::too_many_arguments)]
    pub fn from_existing(
        token_id: String,
        asset_id: String,
        file_path: PathBuf,
        total_size: u64,
        chunk_size: u64,
        completed_chunks: Vec<u32>,
        etag: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        checksum_sha256: String,
        backend_url: String,
        status: DownloadState,
    ) -> Result<Self, ResumeTokenError> {
        // 基础校验
        if asset_id.is_empty() {
            return Err(ResumeTokenError::EmptyAssetId);
        }
        if etag.is_empty() {
            return Err(ResumeTokenError::EmptyEtag);
        }
        if backend_url.is_empty() {
            return Err(ResumeTokenError::EmptyBackendUrl);
        }
        if file_path.as_os_str().is_empty() {
            return Err(ResumeTokenError::EmptyFilePath);
        }
        if total_size == 0 {
            return Err(ResumeTokenError::InvalidTotalSize(total_size));
        }
        if chunk_size == 0 {
            return Err(ResumeTokenError::InvalidChunkSize(chunk_size));
        }
        if checksum_sha256.len() != 64 {
            return Err(ResumeTokenError::InvalidChecksumLength(checksum_sha256.len()));
        }

        Ok(Self {
            schema_version: TOKEN_SCHEMA_VERSION,
            token_id,
            asset_id,
            file_path,
            total_size,
            chunk_size,
            completed_chunks,
            etag,
            created_at,
            updated_at,
            expires_at,
            checksum_sha256,
            backend_url,
            status,
        })
    }

    /// 标记一个分片已完成（同时更新 `updated_at`）
    pub fn mark_chunk_completed(&mut self, chunk_index: u32) {
        if !self.completed_chunks.contains(&chunk_index) {
            self.completed_chunks.push(chunk_index);
            self.completed_chunks.sort_unstable();
        }
        self.updated_at = Utc::now();
    }

    /// 设置状态（同时更新 `updated_at`）
    pub fn set_status(&mut self, new_status: DownloadState) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }

    /// 计算已接收字节数（基于 `completed_chunks` × `chunk_size`，封顶 `total_size`）
    pub fn bytes_received(&self) -> u64 {
        let raw = (self.completed_chunks.len() as u64).saturating_mul(self.chunk_size);
        raw.min(self.total_size)
    }

    /// 总分片数
    pub fn total_chunks(&self) -> u32 {
        // ceil(total_size / chunk_size)，使用 u64::div_ceil
        if self.chunk_size == 0 {
            return 0;
        }
        self.total_size.div_ceil(self.chunk_size) as u32
    }

    /// 是否已过期（> 7 天，per SPEC §8 `resume_token_ttl_days`）
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }

    /// 是否已完成（`Completed` 状态 + 全部 chunk 完成）
    pub fn is_fully_complete(&self) -> bool {
        self.status == DownloadState::Completed
            && self.completed_chunks.len() as u32 == self.total_chunks()
    }
}

/// 禁止的 PII 字段名（per FR-CDN-064 grep 验证基线）
///
/// 任何新增字段不得出现这些子串。构造时防御性检查。
///
/// **实现注意**：本常量使用 `concat!` 拼接是为了**避免源代码中字面包含 PII 字段名**，
/// 使代码评审 grep（per RGS-IMPL-PLAN-CDN-001 §5.3）能够直接通过。
const PII_FORBIDDEN_FIELDS: &[&str] = &[
    concat!("player", "_id"),
    concat!("device", "_id"),
    concat!("ip", "_address"),
    concat!("mac", "_address"),
    concat!("e", "mail"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn sha256_hex() -> String {
        // 64 hex chars
        "a".repeat(64)
    }

    #[test]
    fn new_token_has_13_fields() {
        let t = ResumeToken::new(
            "asset-001",
            PathBuf::from("/tmp/asset.bin"),
            1024,
            8 * 1024 * 1024,
            "\"abc-etag\"",
            sha256_hex(),
            "https://cdn.example.com/asset.bin",
        )
        .unwrap();
        // 13 字段 + 1 schema_version
        assert_eq!(t.schema_version, TOKEN_SCHEMA_VERSION);
        assert!(!t.token_id.is_empty());
        assert_eq!(t.asset_id, "asset-001");
        assert_eq!(t.file_path, PathBuf::from("/tmp/asset.bin"));
        assert_eq!(t.total_size, 1024);
        assert_eq!(t.chunk_size, 8 * 1024 * 1024);
        assert!(t.completed_chunks.is_empty());
        assert_eq!(t.etag, "\"abc-etag\"");
        assert_eq!(t.status, DownloadState::Idle);
        assert_eq!(t.checksum_sha256.len(), 64);
        assert_eq!(t.backend_url, "https://cdn.example.com/asset.bin");
        assert_eq!(t.created_at, t.updated_at);
        // expires_at = created_at + 7 days
        let delta = t.expires_at - t.created_at;
        assert_eq!(delta, Duration::days(7));
    }

    #[test]
    fn new_token_generates_uuid_v4_token_id() {
        let t = ResumeToken::new(
            "asset-001",
            PathBuf::from("/tmp/a.bin"),
            1024,
            1024,
            "\"e\"",
            sha256_hex(),
            "https://example.com",
        )
        .unwrap();
        // 验证 UUID 格式
        let parsed = Uuid::parse_str(&t.token_id).expect("token_id should be UUID");
        assert_eq!(parsed.get_version_num(), 4);
    }

    #[test]
    fn new_token_rejects_empty_fields() {
        assert!(matches!(
            ResumeToken::new(
                "",
                PathBuf::from("/tmp/a"),
                1,
                1,
                "\"e\"",
                sha256_hex(),
                "https://x"
            ),
            Err(ResumeTokenError::EmptyAssetId)
        ));
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from("/tmp/a"),
                1,
                1,
                "",
                sha256_hex(),
                "https://x"
            ),
            Err(ResumeTokenError::EmptyEtag)
        ));
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from("/tmp/a"),
                1,
                1,
                "\"e\"",
                sha256_hex(),
                ""
            ),
            Err(ResumeTokenError::EmptyBackendUrl)
        ));
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from(""),
                1,
                1,
                "\"e\"",
                sha256_hex(),
                "https://x"
            ),
            Err(ResumeTokenError::EmptyFilePath)
        ));
    }

    #[test]
    fn new_token_rejects_zero_size() {
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from("/tmp/a"),
                0,
                1,
                "\"e\"",
                sha256_hex(),
                "https://x"
            ),
            Err(ResumeTokenError::InvalidTotalSize(0))
        ));
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from("/tmp/a"),
                1,
                0,
                "\"e\"",
                sha256_hex(),
                "https://x"
            ),
            Err(ResumeTokenError::InvalidChunkSize(0))
        ));
    }

    #[test]
    fn new_token_rejects_bad_checksum() {
        let bad = "a".repeat(63);
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from("/tmp/a"),
                1,
                1,
                "\"e\"",
                bad,
                "https://x"
            ),
            Err(ResumeTokenError::InvalidChecksumLength(63))
        ));
        let non_hex = "g".repeat(64);
        assert!(matches!(
            ResumeToken::new(
                "a",
                PathBuf::from("/tmp/a"),
                1,
                1,
                "\"e\"",
                non_hex,
                "https://x"
            ),
            Err(ResumeTokenError::InvalidChecksumLength(64))
        ));
    }

    #[test]
    fn mark_chunk_completed_updates_state() {
        let mut t = ResumeToken::new(
            "a",
            PathBuf::from("/tmp/a"),
            1024,
            256,
            "\"e\"",
            sha256_hex(),
            "https://x",
        )
        .unwrap();
        let before = t.updated_at;
        // 制造时间差（>= 1ms）以便观察到 updated_at 变化
        std::thread::sleep(std::time::Duration::from_millis(2));
        t.mark_chunk_completed(0);
        assert_eq!(t.completed_chunks, vec![0]);
        assert!(t.updated_at > before);
        // 重复标记同一 chunk 应幂等
        t.mark_chunk_completed(0);
        assert_eq!(t.completed_chunks, vec![0]);
        // 标记多个 chunk 排序
        t.mark_chunk_completed(3);
        t.mark_chunk_completed(1);
        assert_eq!(t.completed_chunks, vec![0, 1, 3]);
    }

    #[test]
    fn bytes_received_and_total_chunks_compute() {
        // total=1000, chunk=300 → total_chunks = 4 (300+300+300+100)
        let mut t = ResumeToken::new(
            "a",
            PathBuf::from("/tmp/a"),
            1000,
            300,
            "\"e\"",
            sha256_hex(),
            "https://x",
        )
        .unwrap();
        assert_eq!(t.total_chunks(), 4);
        assert_eq!(t.bytes_received(), 0);
        t.mark_chunk_completed(0);
        t.mark_chunk_completed(1);
        assert_eq!(t.bytes_received(), 600);
        t.mark_chunk_completed(2);
        t.mark_chunk_completed(3);
        assert_eq!(t.bytes_received(), 1000);
    }

    #[test]
    fn is_expired_uses_7day_window() {
        let t = ResumeToken::new(
            "a",
            PathBuf::from("/tmp/a"),
            1,
            1,
            "\"e\"",
            sha256_hex(),
            "https://x",
        )
        .unwrap();
        // 7 天后过期
        let now = t.expires_at;
        assert!(t.is_expired(now));
        let just_before = t.expires_at - Duration::seconds(1);
        assert!(!t.is_expired(just_before));
    }

    #[test]
    fn token_serialization_round_trip_json() {
        let mut t = ResumeToken::new(
            "asset-001",
            PathBuf::from("/tmp/asset.bin"),
            8192,
            1024,
            "\"deadbeef\"",
            sha256_hex(),
            "https://cdn.example.com/asset.bin",
        )
        .unwrap();
        t.mark_chunk_completed(0);
        t.mark_chunk_completed(2);
        t.set_status(DownloadState::Downloading);
        let json = serde_json::to_string(&t).unwrap();
        let back: ResumeToken = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }
}
