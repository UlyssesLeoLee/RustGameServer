//! Windows sparse file + SetFileValidData 权限评估
//!
//! per R1 R2（per RGS-IMPL-PLAN-CDN-001 v0.1 §6）：
//! - `SetFileValidData` 需要 `SeManageVolumePrivilege`（普通用户无）
//! - 降级路径：`SetEndOfFile` + 显式填充 0

use super::{Result, SparseFileAllocator};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct WindowsSparseFile;

impl SparseFileAllocator for WindowsSparseFile {
    fn pre_allocate(&self, path: &Path, length: u64) -> Result<()> {
        let _ = (path, length);
        Err(crate::error::DownloadError(
            crate::error::DownloadErrorKind::Internal(
                "WindowsSparseFile::pre_allocate 是 IT 测试 stub".into(),
            ),
        ))
    }
}
