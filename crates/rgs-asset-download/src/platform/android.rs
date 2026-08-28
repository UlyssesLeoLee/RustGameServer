//! Android sparse file 预分配（per M-2065.6 + IMPL-PLAN §6 R3）。
//!
//! Android 应用沙箱目录（`/data/data/<package>/files/...`）位于 ext4 / f2fs，**不**暴露
//! `fallocate` syscall；用 `truncate` + 显式写 0 实现。降级路径与 unix 一致。

use crate::error::DownloadResult;
use crate::platform::{fallback_preallocate, PreallocateOutcome, PreallocateStrategy};

/// Android sparse file 预分配入口。
///
/// 当前实现：
/// 1. `OpenOptions::create + write + truncate`
/// 2. `set_len(size)` 把文件大小设为目标
/// 3. `write_all(&[0])` 触发首字节写 0（保证 `metadata().len() == size`）
/// 4. 失败 → 走 `fallback_preallocate`
#[cfg(target_os = "android")]
pub fn preallocate(path: &str, size: u64) -> DownloadResult<PreallocateOutcome> {
    use std::io::Write;
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
    {
        Ok(f) => f,
        Err(_) => return fallback_preallocate(path, size),
    };
    if file.set_len(size).is_err() {
        return fallback_preallocate(path, size);
    }
    if file.write_all(&[0u8; 1]).is_err() {
        return fallback_preallocate(path, size);
    }
    let _ = file.flush();
    let actual = file.metadata().map(|m| m.len()).unwrap_or(size);
    Ok(PreallocateOutcome {
        strategy: PreallocateStrategy::Native,
        requested_size: size,
        actual_size: actual,
        fallback_reason: None,
    })
}

#[cfg(not(target_os = "android"))]
pub fn preallocate(path: &str, _size: u64) -> DownloadResult<PreallocateOutcome> {
    // 非 Android target 调用 android::preallocate → fallback（错误）
    let _ = path;
    Err(
        crate::error::DownloadError::PlatformPreallocateUnsupported {
            target_os: "android (compiled on non-android)".into(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preallocate_outcome_shape() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let r = preallocate(path.to_str().unwrap(), 4096);
        #[cfg(target_os = "android")]
        assert!(r.is_ok());
        #[cfg(not(target_os = "android"))]
        assert!(r.is_err());
    }
}
