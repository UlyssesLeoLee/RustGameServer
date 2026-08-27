//! 4 平台 sparse file 预分配（M-2065.6 + M-2065.7）。
//!
//! ## 目标平台
//!
//! - `unix.rs`     Linux / macOS：`fallocate` / `posix_fallocate`
//! - `windows.rs`  Windows：sparse file + `SetFileValidData` 权限评估
//! - `android.rs`  Android：复用 unix 路径（应用 sandbox 目录）
//! - `ios.rs`      iOS：复用 unix 路径（应用 sandbox 目录）
//!
//! ## 降级策略（per IMPL-PLAN §6 R2 + R3）
//!
//! - **Windows 普通用户**：无 `SeManageVolumePrivilege` → `SetFileValidData` 失败 → 降级到
//!   `SetEndOfFile` + 显式写 0（首字节 / 末字节），保证 `Content-Length` 对得上 `IntegrityGate`
//! - **iOS / Android 沙箱**：无 `fallocate` → 走 `ftruncate` + 首字节写 0
//! - **不支持的 target_os**：`PlatformPreallocateUnsupported` 错误（一般不会触发——已覆盖 4 平台）
//!
//! ## 硬约束
//!
//! - **FR-CDN-064**：本文件**禁止**引用 PII 字段；`path` 仅作为字符串传递，不进 metric label
//! - **NFR-CDN-002**：本文件不涉及整文件 hash（由 `integrity_gate.rs` 负责）
//! - **不持有服务端凭证**（FR-CDN-001 既有）

use crate::error::{DownloadError, DownloadResult};

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

/// 预分配策略（用于 metrics 标签 + 调试日志）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreallocateStrategy {
    /// 平台原生 syscall（fallocate / SetFileValidData）
    Native,
    /// 降级路径：SetEndOfFile / ftruncate + 显式写 0
    Fallback,
    /// 目标平台不支持（一般不触发；4 平台已全覆盖）
    Unsupported,
}

/// 预分配结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreallocateOutcome {
    /// 实际采用的策略
    pub strategy: PreallocateStrategy,
    /// 请求预分配的大小（字节）
    pub requested_size: u64,
    /// 实际落盘大小（与 requested_size 一致；如有差异会记录在此）
    pub actual_size: u64,
    /// 降级原因（仅 `strategy = Fallback` 时有值；不含 PII）
    pub fallback_reason: Option<String>,
}

/// Sparse file 预分配器 trait（per `it_minio_platform.rs` 4 平台统一调用入口）
///
/// 平台实现：
/// - `unix.rs`：`UnixSparseFile`（Linux / macOS / Android / iOS 共用）
/// - `windows.rs`：`WindowsSparseFile`
pub trait SparseFileAllocator {
    /// 平台预分配方法
    fn preallocate(&self, path: &str, size: u64) -> DownloadResult<PreallocateOutcome>;
}

/// 顶层入口：按当前 target_os 路由到平台实现。
///
/// 行为：
/// 1. 创建/打开文件
/// 2. 平台预分配（4 平台各自实现）
/// 3. 失败 → 降级到 [`fallback_preallocate`]
/// 4. 仍失败 → `DownloadError::Io` / `PlatformPreallocateUnsupported`
pub fn preallocate_sparse_file(
    path: &str,
    size: u64,
) -> DownloadResult<PreallocateOutcome> {
    if size == 0 {
        return Err(DownloadError::ConfigInvalid {
            field: "size".into(),
            reason: "must be > 0".into(),
        });
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        crate::platform::unix::preallocate(path, size)
    }
    #[cfg(target_os = "windows")]
    {
        crate::platform::windows::preallocate(path, size)
    }
    #[cfg(target_os = "android")]
    {
        crate::platform::android::preallocate(path, size)
    }
    #[cfg(target_os = "ios")]
    {
        crate::platform::ios::preallocate(path, size)
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )))]
    {
        Err(DownloadError::PlatformPreallocateUnsupported {
            target_os: std::env::consts::OS.to_string(),
        })
    }
}

/// 降级预分配（所有平台共用）：创建/打开 + `set_len` + 首字节写 0。
///
/// 用于：
/// - 平台 syscall 失败（Windows 普通用户、iOS/Android 沙箱）
/// - 单元测试跨平台路径
pub fn fallback_preallocate(path: &str, size: u64) -> DownloadResult<PreallocateOutcome> {
    use std::io::Write;
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| DownloadError::Io {
                path: path.to_string(),
                kind: format!("open: {e}"),
            })?;
        file.set_len(size).map_err(|e| DownloadError::Io {
            path: path.to_string(),
            kind: format!("set_len: {e}"),
        })?;
        // 显式写 0 到首字节（确保实际占盘 + Content-Length 对齐）
        file.write_all(&[0u8; 1]).map_err(|e| DownloadError::Io {
            path: path.to_string(),
            kind: format!("write_zero: {e}"),
        })?;
        file.flush().map_err(|e| DownloadError::Io {
            path: path.to_string(),
            kind: format!("flush: {e}"),
        })?;
    }
    let actual = std::fs::metadata(path)
        .map_err(|e| DownloadError::Io {
            path: path.to_string(),
            kind: format!("metadata: {e}"),
        })?
        .len();
    Ok(PreallocateOutcome {
        strategy: PreallocateStrategy::Fallback,
        requested_size: size,
        actual_size: actual,
        fallback_reason: Some("set_len+write_zero".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preallocate_zero_size_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let r = preallocate_sparse_file(path.to_str().unwrap(), 0);
        assert!(r.is_err());
    }

    #[test]
    fn fallback_preallocate_creates_file_with_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let outcome = fallback_preallocate(path.to_str().unwrap(), 64 * 1024).unwrap();
        assert_eq!(outcome.requested_size, 64 * 1024);
        assert_eq!(outcome.strategy, PreallocateStrategy::Fallback);
        assert!(path.exists());
    }
}
