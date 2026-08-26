//! `IntegrityGate` —— 整文件 SHA-256 校验闸门（M-2065.5）。
//!
//! ## 职责
//!
//! - 下载完成后对落盘文件做**整文件** SHA-256 校验
//! - 与 `manifest` 声明的 `expected_sha256`（hex）比对
//! - 通过 → 切到 `asset-update` 灰度；不通过 → 触发 `IntegrityMismatch` 错误 + metrics `rgs_asset_download_integrity_failure_total`
//!
//! ## 硬约束（NFR-CDN-002）
//!
//! - **整文件校验不可绕过**；本文件不得引入 `skip_integrity` / `bypass_integrity` 标记
//! - **分块到达不做单独校验**（性能 / 局部篡改风险由全文件 hash 兜底）
//!
//! ## 性能
//!
//! - GB 级文件 hash 用 `tokio::task::spawn_blocking` 跑同步 `Sha256::digest` 块读取
//! - 默认块大小 1 MiB（per SPEC §3「整文件 SHA-256；分块到达不做单独校验」）
//!
//! ## 安全（FR-CDN-064）
//!
//! - 本文件**禁止**引用 PII 字段

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

use crate::error::{DownloadError, DownloadResult};

/// 校验结果状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrityStatus {
    /// 期望 == 实际
    Match,
    /// 不匹配
    Mismatch,
}

/// 整文件校验报告。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    /// 结果
    pub status: IntegrityStatus,
    /// 期望 SHA-256（hex lowercase）
    pub expected_sha256: String,
    /// 实际 SHA-256（hex lowercase）
    pub actual_sha256: String,
    /// 文件大小（字节）
    pub size_bytes: u64,
    /// 校验耗时（毫秒；用于 `rgs_asset_download_integrity_duration_seconds`）
    pub duration_ms: u64,
}

/// 整文件 SHA-256 校验闸门。
#[derive(Debug, Clone, Default)]
pub struct IntegrityGate {
    /// 读块大小（默认 1 MiB；适合 GB 级文件）
    pub block_size: usize,
}

impl IntegrityGate {
    /// 新建（默认 1 MiB 块）。
    pub fn new() -> Self {
        Self {
            block_size: 1024 * 1024,
        }
    }

    /// 自定义块大小（调试 / 内存压力测试用）。
    pub fn with_block_size(block_size: usize) -> Self {
        Self {
            block_size: block_size.max(4096),
        }
    }

    /// 异步执行整文件 hash（`spawn_blocking` 跑同步 IO + 同步 hash）。
    ///
    /// - `file_path`：落盘文件路径
    /// - `expected_sha256`：manifest 声明的 hex（小写）
    /// - 返回 [`IntegrityReport`]（含 status / actual / duration_ms）
    pub async fn verify(
        &self,
        file_path: &str,
        expected_sha256: &str,
    ) -> DownloadResult<IntegrityReport> {
        let path = file_path.to_string();
        let expected = expected_sha256.to_ascii_lowercase();
        let block_size = self.block_size;

        let started = std::time::Instant::now();
        let (actual_hex, size) = tokio::task::spawn_blocking(move || {
            hash_file_blocking(path, block_size)
        })
        .await
        .map_err(|e| DownloadError::HttpClient(format!("integrity task join: {e}")))??;

        let duration_ms = started.elapsed().as_millis() as u64;
        let status = if actual_hex == expected {
            IntegrityStatus::Match
        } else {
            IntegrityStatus::Mismatch
        };

        Ok(IntegrityReport {
            status,
            expected_sha256: expected,
            actual_sha256: actual_hex,
            size_bytes: size,
            duration_ms,
        })
    }

    /// 同步版本（仅给 `&mut Sha256` 复用 / 单元测试用）。
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

fn hash_file_blocking(path: String, block_size: usize) -> DownloadResult<(String, u64)> {
    use std::io::Read;
    let mut file = std::fs::File::open(&path).map_err(|e| DownloadError::Io {
        path: path.clone(),
        kind: format!("open: {e}"),
    })?;
    let size = file.metadata().map_err(|e| DownloadError::Io {
        path: path.clone(),
        kind: format!("metadata: {e}"),
    })?.len();
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; block_size];
    loop {
        let n = file.read(&mut buf).map_err(|e| DownloadError::Io {
            path: path.clone(),
            kind: format!("read: {e}"),
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok((hex::encode(hasher.finalize()), size))
}

/// 异步文件 hash（备选；当前实装用 `spawn_blocking` + 同步 IO，性能等价且无 `AsyncReadExt` 依赖）。
#[allow(dead_code)]
pub async fn hash_file_async(path: &Path) -> DownloadResult<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| DownloadError::Io {
            path: path.to_string_lossy().to_string(),
            kind: format!("open: {e}"),
        })?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| DownloadError::Io {
                path: path.to_string_lossy().to_string(),
                kind: format!("read: {e}"),
            })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_bytes_known_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let empty = IntegrityGate::hash_bytes(b"");
        assert_eq!(
            empty,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // SHA-256("abc")
        let abc = IntegrityGate::hash_bytes(b"abc");
        assert_eq!(
            abc,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[tokio::test]
    async fn verify_match_for_known_payload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let payload = b"hello, integrity";
        tokio::fs::write(&path, payload).await.unwrap();
        let expected = IntegrityGate::hash_bytes(payload);
        let gate = IntegrityGate::new();
        let report = gate
            .verify(path.to_str().unwrap(), &expected)
            .await
            .unwrap();
        assert_eq!(report.status, IntegrityStatus::Match);
        assert_eq!(report.size_bytes, payload.len() as u64);
    }

    #[tokio::test]
    async fn verify_mismatch_detects_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        tokio::fs::write(&path, b"original").await.unwrap();
        let gate = IntegrityGate::new();
        // 声称是别的 hash → mismatch
        let report = gate
            .verify(path.to_str().unwrap(), "0000000000000000000000000000000000000000000000000000000000000000")
            .await
            .unwrap();
        assert_eq!(report.status, IntegrityStatus::Mismatch);
    }
}
