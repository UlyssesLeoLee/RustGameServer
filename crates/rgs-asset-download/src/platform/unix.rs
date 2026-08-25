//! Unix / macOS sparse file 预分配（fallocate / posix_fallocate）

use super::{Result, SparseFileAllocator};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct UnixSparseFile;

impl SparseFileAllocator for UnixSparseFile {
    fn pre_allocate(&self, path: &Path, length: u64) -> Result<()> {
        // IT 测试不会真实调（#[ignore]），实现仅占位
        let _ = (path, length);
        Err(crate::error::DownloadError(
            crate::error::DownloadErrorKind::Internal(
                "UnixSparseFile::pre_allocate 是 IT 测试 stub".into(),
            ),
        ))
    }
}
