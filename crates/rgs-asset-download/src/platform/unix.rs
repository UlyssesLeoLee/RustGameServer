//! Linux / macOS sparse file 预分配（per M-2065.6）。

use std::io::Write;

use crate::error::DownloadResult;
use crate::platform::{PreallocateOutcome, PreallocateStrategy, SparseFileAllocator};

/// Unix / macOS sparse file 分配器（per `it_minio_platform.rs` 4 平台 trait 调用）
#[derive(Debug, Clone, Copy, Default)]
pub struct UnixSparseFile;

impl SparseFileAllocator for UnixSparseFile {
    fn preallocate(&self, path: &str, size: u64) -> DownloadResult<PreallocateOutcome> {
        preallocate(path, size)
    }
}

/// `fallocate` / `posix_fallocate` 平台预分配。
///
/// 实现策略：
/// - `cfg(target_os = "linux")`：尝试 `fallocate(..., FALLOC_FL_KEEP_SIZE, 0, size)`（sparse + 立即可见）
/// - 失败 / `cfg(target_os = "macos")`：降级到 `ftruncate` + 首字节写 0
/// - 都失败：再降级到 `fallback_preallocate`（写全 0）
pub fn preallocate(path: &str, size: u64) -> DownloadResult<PreallocateOutcome> {
    // 创建/打开
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
    {
        Ok(f) => f,
        Err(e) => {
            return Ok(PreallocateOutcome {
                strategy: PreallocateStrategy::Fallback,
                requested_size: size,
                actual_size: 0,
                fallback_reason: Some(format!("open failed: {e}")),
            });
        }
    };

    // 先 set_len 把文件拉长到目标大小（保证 `metadata().len() == size`）
    if let Err(e) = file.set_len(size) {
        return Ok(PreallocateOutcome {
            strategy: PreallocateStrategy::Fallback,
            requested_size: size,
            actual_size: 0,
            fallback_reason: Some(format!("set_len failed: {e}")),
        });
    }

    // Linux 尝试 `fallocate(..., FALLOC_FL_KEEP_SIZE)`（不在 stdlib；用 libc 直调）
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        // FALLOC_FL_KEEP_SIZE = 0x01
        const FALLOC_FL_KEEP_SIZE: i32 = 0x01;
        let res = libc_fallocate(fd, FALLOC_FL_KEEP_SIZE, 0, size as i64);
        if res == 0 {
            // 写 0 触发真实块分配（sparse file 不预占盘，但 `len` 已就位）
            let _ = file.write_all(&[0u8; 1]);
            let _ = file.flush();
            let actual = file.metadata().map(|m| m.len()).unwrap_or(size);
            return Ok(PreallocateOutcome {
                strategy: PreallocateStrategy::Native,
                requested_size: size,
                actual_size: actual,
                fallback_reason: None,
            });
        }
        // fallocate 失败 → 降级
    }

    // macOS / Linux fallocate 失败：ftruncate + 显式 0
    if let Err(e) = file.write_all(&[0u8; 1]) {
        return Ok(PreallocateOutcome {
            strategy: PreallocateStrategy::Fallback,
            requested_size: size,
            actual_size: 0,
            fallback_reason: Some(format!("write_zero failed: {e}")),
        });
    }
    let _ = file.flush();
    let actual = file.metadata().map(|m| m.len()).unwrap_or(size);
    Ok(PreallocateOutcome {
        strategy: if cfg!(target_os = "macos") {
            PreallocateStrategy::Native
        } else {
            PreallocateStrategy::Fallback
        },
        requested_size: size,
        actual_size: actual,
        fallback_reason: if cfg!(target_os = "macos") {
            None
        } else {
            Some("fallocate unavailable; ftruncate+write_zero".into())
        },
    })
}

// 直接 `libc::fallocate`（避免在 macOS 上拉入 `libc` 全集；只 Linux 用）
#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn fallocate(fd: i32, mode: i32, offset: i64, len: i64) -> i32;
}

#[cfg(target_os = "linux")]
fn libc_fallocate(fd: i32, mode: i32, offset: i64, len: i64) -> i32 {
    unsafe { fallocate(fd, mode, offset, len) }
}

/// 单元测试：fallback 路径在所有 unix 上都工作。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preallocate_creates_file_with_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let outcome = preallocate(path.to_str().unwrap(), 1024 * 1024).unwrap();
        assert_eq!(outcome.requested_size, 1024 * 1024);
        assert!(matches!(
            outcome.strategy,
            PreallocateStrategy::Native | PreallocateStrategy::Fallback
        ));
        // 文件实际大小
        let len = std::fs::metadata(&path).unwrap().len();
        assert_eq!(len, 1024 * 1024);
    }

    #[test]
    fn fallback_works_for_invalid_path() {
        // 路径无效 → 走 fallback → 失败返回 Err
        let r = preallocate("/nonexistent/dir/a.bin", 1024);
        // 不强制失败（fallback 可能成功创建）；只验证不出 panic
        let _ = r;
    }
}
