//! Android 平台 sparse file 预分配（应用沙箱目录）。
//!
//! per **RGS-IMPL-PLAN-CDN-001 v0.1 §2.2**（android.rs 文件）+ **§6 R3 风险**
//! （iOS / Android 沙箱目录限制）。
//!
//! # 当前状态
//!
//! **M-2063.2 骨架 stub**：直接返回 `Ok(())`；M-2065.6 实测补全：
//! - 路径：`context.getFilesDir()/downloads/`（应用沙箱目录）
//! - 预分配：通过 `ParcelFileDescriptor` + `ftruncate` 设置文件大小
//!
//! # 硬约束
//!
//! - **NFR-CDN-002（整文件校验不可绕过）**：`preallocate` 仅做磁盘预分配，
//!   **不**写入文件内容；整文件 hash 校验由 `IntegrityGate`（M-2065.5）执行。
//! - **R3 风险缓解**（per 实施计划 §6 R3）：应用被 kill 后沙箱目录仍在，
//!   但 `token_id` 需重新生成；启动时 `ResumeTokenStore::cleanup_expired`
//!   清理 7 天前断点（per DTL §4.3）。
//! - **FR-CDN-064（断点记录不含 PII）**：`token_id` 与 app session 绑定，
//!   **不**与 `player_id` 绑定。

use std::path::Path;

use crate::error::DownloadError;

/// Android 平台 sparse file 预分配桩（M-2065.6 实测补全）。
#[allow(dead_code)]
pub async fn preallocate(path: &Path, total_size: u64) -> Result<(), DownloadError> {
    let _ = (path, total_size);
    // M-2065.6 实测补全：应用沙箱目录 + ftruncate 预分配
    Ok(())
}
