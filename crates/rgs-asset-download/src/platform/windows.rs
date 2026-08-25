//! Windows 平台 sparse file + `SetFileValidData` 权限评估。
//!
//! per **RGS-IMPL-PLAN-CDN-001 v0.1 §2.2**（windows.rs 文件）+ **§6 R2 风险**
//! （`SetFileValidData` 权限评估降级路径）。
//!
//! # 当前状态
//!
//! **M-2063.2 骨架 stub**：直接返回 `Ok(())`；M-2065.6 + M-2065.7 实测补全：
//! 1. 创建文件 → `FSCTL_SET_SPARSE` 标记 sparse
//! 2. `SetFileValidData` 设置有效数据长度（需 `SeManageVolumePrivilege`）
//! 3. 权限不足时降级：`SetEndOfFile` + 显式填充 0
//!
//! # 硬约束
//!
//! - **NFR-CDN-002（整文件校验不可绕过）**：`preallocate` 仅做磁盘预分配，
//!   **不**写入文件内容；整文件 hash 校验由 `IntegrityGate`（M-2065.5）执行。
//! - **R2 风险缓解**（per 实施计划 §6 R2）：普通用户无
//!   `SeManageVolumePrivilege`，必须走降级路径。

use std::path::Path;

use crate::error::DownloadError;

/// Windows 平台 sparse file 预分配桩（M-2065.6 + M-2065.7 实测补全）。
#[allow(dead_code)]
pub async fn preallocate(path: &Path, total_size: u64) -> Result<(), DownloadError> {
    let _ = (path, total_size);
    // M-2065.6 + M-2065.7 实测补全：FSCTL_SET_SPARSE + SetFileValidData + 降级路径
    Ok(())
}
