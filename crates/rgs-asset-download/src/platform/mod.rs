//! 平台特定模块：4 平台 sparse file 预分配。
//!
//! per **RGS-IMPL-PLAN-CDN-001 v0.1 §2.2**（文件结构）+ **§6 R2 风险**
//! （Windows `SetFileValidData` 权限评估降级路径）。
//!
//! # 目标平台
//!
//! | 平台 | 模块 | 备注 |
//! |---|---|---|
//! | Linux / macOS | [`unix`] | `fallocate` / `posix_fallocate` |
//! | Windows 11 | [`windows`] | sparse file + `SetFileValidData` 权限评估 |
//! | Android 14 | [`android`] | 应用沙箱目录 sparse file |
//! | iOS 17 | [`ios`] | 应用沙箱目录 sparse file |
//!
//! # 当前状态
//!
//! **M-2063.2 骨架版本**：`preallocate` 桩函数返回 `Ok(())`；M-2065.6 在 4
//! 平台实测补全各平台实现（per 实施计划 §3.3 M-2065.6 + M-2065.7）。
//!
//! # 硬约束
//!
//! - **NFR-CDN-002（整文件校验不可绕过）**：`preallocate` 仅做磁盘预分配，
//!   **不**修改文件内容；整文件 hash 校验由 `IntegrityGate`（M-2065.5）执行。
//! - **R2 风险缓解**：Windows 平台若 `SetFileValidData` 权限不足（普通用户无
//!   `SeManageVolumePrivilege`），降级到 `SetEndOfFile` + 显式填充 0
//!   （per 实施计划 §6 R2 风险 / M-2065.7）。

use std::path::Path;

use crate::error::DownloadError;

// — 平台模块声明（编译期按 target_os 选择；非目标平台的 .rs 不参与编译）—

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "ios")]
mod ios;

// — 对外 API：sparse file 预分配入口 —

/// 平台特定 sparse file 预分配入口。
///
/// 编译期按 `target_os` 路由到对应平台实现；调用方无需关心底层。
///
/// # 当前状态
///
/// **M-2063.2 骨架 stub**：直接返回 `Ok(())`；M-2065.6 实测补全各平台实现。
#[allow(dead_code)]
pub async fn preallocate(path: &Path, total_size: u64) -> Result<(), DownloadError> {
    let _ = (path, total_size);

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // M-2065.6 实测补全：fallocate / posix_fallocate
        return unix::preallocate(path, total_size).await;
    }

    #[cfg(target_os = "windows")]
    {
        // M-2065.6 + M-2065.7 实测补全：sparse file + SetFileValidData 权限评估降级
        return windows::preallocate(path, total_size).await;
    }

    #[cfg(target_os = "android")]
    {
        // M-2065.6 实测补全：应用沙箱目录 sparse file
        return android::preallocate(path, total_size).await;
    }

    #[cfg(target_os = "ios")]
    {
        // M-2065.6 实测补全：应用沙箱目录 sparse file
        return ios::preallocate(path, total_size).await;
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )))]
    {
        // 不支持的平台：编译期显式报错
        compile_error!(
            "rgs-asset-download 不支持此 target_os；仅支持 linux / macos / windows / android / ios（per RGS-IMPL-PLAN-CDN-001 v0.1 §1.1 4 平台）"
        );
    }
}
