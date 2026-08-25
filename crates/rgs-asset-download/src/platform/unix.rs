//! Unix 平台 sparse file 预分配（Linux / macOS）。
//!
//! per **RGS-IMPL-PLAN-CDN-001 v0.1 §2.2**（unix.rs 文件）。
//!
//! # 当前状态
//!
//! **M-2063.2 骨架 stub**：直接返回 `Ok(())`；M-2065.6 实测补全：
//! - Linux：`fallocate`（`FALLOC_FL_KEEP_SIZE` / `FALLOC_FL_PUNCH_HOLE`）或
//!   `posix_fallocate`（POSIX 兼容回退）
//! - macOS：`fcntl(F_PREALLOCATE)` + `F_ALLOCATECONTIG` / `F_ALLOCATEALL`
//!
//! # 硬约束
//!
//! - **NFR-CDN-002（整文件校验不可绕过）**：`preallocate` 仅做磁盘预分配，
//!   **不**写入文件内容；整文件 hash 校验由 `IntegrityGate`（M-2065.5）执行。
//! - **R4 风险缓解**（per 实施计划 §6 R4）：单 chunk 默认 8 MiB（远小于
//!   MinIO 默认 5 GiB 单 chunk 上限）。

use std::path::Path;

use crate::error::DownloadError;

/// Unix 平台 sparse file 预分配桩（M-2065.6 实测补全）。
#[allow(dead_code)]
pub async fn preallocate(path: &Path, total_size: u64) -> Result<(), DownloadError> {
    let _ = (path, total_size);
    // M-2065.6 实测补全：fallocate / posix_fallocate / fcntl(F_PREALLOCATE)
    Ok(())
}
