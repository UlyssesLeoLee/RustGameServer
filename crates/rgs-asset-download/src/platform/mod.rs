//! 4 平台分支（per SPEC-DTL-041 v0.2 §2 + DTL §3.6）
//!
//! - `unix`    Linux / macOS sparse file 预分配（fallocate / posix_fallocate）
//! - `windows` Windows sparse file + SetFileValidData 权限评估
//! - `android` Android sparse file（应用沙箱目录）
//! - `ios`     iOS sparse file（应用沙箱目录）

use std::path::Path;

pub type Result<T> = std::result::Result<T, crate::error::DownloadError>;

/// 4 平台 sparse file 预分配
pub trait SparseFileAllocator: Send + Sync {
    /// 预分配 length 字节（per AC-CDN-113 权限 + 性能实测）
    /// 失败时返回降级路径（普通 write 填充 0）
    fn pre_allocate(&self, path: &Path, length: u64) -> Result<()>;
}

#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;
#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(unix)]
pub use unix::UnixSparseFile;
#[cfg(windows)]
pub use windows::WindowsSparseFile;
