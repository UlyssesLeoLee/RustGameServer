//! iOS sparse file 预分配（per M-2065.6 + IMPL-PLAN §6 R3）。
//!
//! iOS 应用沙箱目录位于 APFS，**不**暴露 `fallocate`；用 `ftruncate` + 显式写 0。
//! 与 Android 实现一致；分文件以便未来 iOS 特定优化（如 NSFileCoordinator 集成）。

use crate::error::DownloadResult;
use crate::platform::{fallback_preallocate, PreallocateOutcome, PreallocateStrategy};

/// iOS sparse file 预分配入口。
///
/// 当前实现与 android 一致；预留 iOS 特定 hook 入口。
#[cfg(target_os = "ios")]
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

#[cfg(not(target_os = "ios"))]
pub fn preallocate(path: &str, _size: u64) -> DownloadResult<PreallocateOutcome> {
    let _ = path;
    Err(crate::error::DownloadError::PlatformPreallocateUnsupported {
        target_os: "ios (compiled on non-ios)".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preallocate_outcome_shape() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        let r = preallocate(path.to_str().unwrap(), 4096);
        #[cfg(target_os = "ios")]
        assert!(r.is_ok());
        #[cfg(not(target_os = "ios"))]
        assert!(r.is_err());
    }
}
