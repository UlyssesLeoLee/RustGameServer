//! Windows sparse file + `SetFileValidData` 权限评估（M-2065.6 + M-2065.7）。
//!
//! ## 流程
//!
//! 1. `CreateFileW` 打开（带 `FILE_ATTRIBUTE_NORMAL`）
//! 2. `FSCTL_SET_SPARSE` 把文件标记为 sparse
//! 3. `SetFileValidData(size)` 快速预分配（不写 0）
//!    - 需要 `SeManageVolumePrivilege`（普通用户没有）
//! 4. 如果 `SetFileValidData` 失败（错误 1314 / ERROR_PRIVILEGE_NOT_HELD）：
//!    - **降级路径**：`SetEndOfFile(size)` + 显式写 0 到首字节 / 末字节
//!    - 记录 `fallback_reason = "SeManageVolumePrivilege missing"`
//!
//! ## 硬约束
//!
//! - **不**调用 `WriteFile` 全写 0（性能太差，会让 sparse file 失去意义）
//! - 降级路径**保证** `Content-Length` 对齐 `IntegrityGate` 期望
//!
//! ## 平台说明
//!
//! 本文件代码可在所有 target 编译（不引入 windows-sys 依赖；用 `cfg(windows)` 隔离）。
//! 真实 Windows syscall 落到 PH-4（#2069）实装；M-2065.6+7 提供**接口 + 降级逻辑**。

use crate::error::{DownloadError, DownloadResult};
use crate::platform::{fallback_preallocate, PreallocateOutcome, PreallocateStrategy};

/// Windows sparse file 预分配入口。
///
/// 在非 Windows target 上**直接走 fallback**（无 windows-sys 依赖）。
#[cfg(target_os = "windows")]
pub fn preallocate(path: &str, size: u64) -> DownloadResult<PreallocateOutcome> {
    // 真实 Windows syscall 留给 PH-4（#2069）；M-2065.6+7 阶段：
    // 1) OpenOptions::create + write + truncate
    // 2) SetFileValidData via FSCTL_SET_VALID_DATA (windows-sys 0.52+)
    // 3) 失败 → 走 fallback
    //
    // 当前实现：直接 fallback；M-2065.7 提供权限评估 API（见 `evaluate_set_file_valid_data`）
    evaluate_then_fallback(path, size, "M-2065.7 阶段：走 fallback，PH-4 接 windows-sys")
}

#[cfg(not(target_os = "windows"))]
pub fn preallocate(path: &str, size: u64) -> DownloadResult<PreallocateOutcome> {
    // 非 Windows target 调用 windows::preallocate → 走 fallback（错误）
    let _ = path;
    let _ = size;
    Err(DownloadError::PlatformPreallocateUnsupported {
        target_os: "windows (compiled on non-windows)".into(),
    })
}

#[cfg(target_os = "windows")]
fn evaluate_then_fallback(
    path: &str,
    size: u64,
    reason: &str,
) -> DownloadResult<PreallocateOutcome> {
    let _ = reason;
    fallback_preallocate(path, size)
}

/// 评估 `SetFileValidData` 权限（per M-2065.7）。
///
/// 真实实现需要：
/// 1. `OpenProcessToken` 获取当前进程 token
/// 2. `GetTokenInformation` 查 `TokenPrivileges`
/// 3. 找 `SeManageVolumePrivilege` (LUID = 27)
///
/// 当前 PREREQ / M-2065.7 阶段：返回 `Ok(false)` + 降级原因。
/// PH-4（#2069）用 `windows-sys 0.52+` 实现真实权限查询。
pub fn evaluate_set_file_valid_data() -> DownloadResult<SetFileValidDataPermission> {
    Ok(SetFileValidDataPermission {
        granted: false,
        reason: "M-2065.7 阶段：未实现真实 LUID 查询；PH-4 实装".into(),
    })
}

/// `SetFileValidData` 权限评估结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetFileValidDataPermission {
    /// 当前进程是否拥有 `SeManageVolumePrivilege`
    pub granted: bool,
    /// 评估原因（用于 metrics 标签 + 调试日志；**不**含 PII）
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_returns_not_granted_in_pre_impl() {
        let perm = evaluate_set_file_valid_data().unwrap();
        assert!(!perm.granted);
        assert!(!perm.reason.is_empty());
    }

    #[test]
    fn preallocate_outcome_includes_fallback_reason() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        // 在非 Windows target 上 → PlatformPreallocateUnsupported；
        // 在 Windows target 上 → 走 fallback_preallocate 并记录 reason
        #[cfg(target_os = "windows")]
        {
            let outcome = preallocate(path.to_str().unwrap(), 64 * 1024).unwrap();
            assert_eq!(outcome.strategy, PreallocateStrategy::Fallback);
            assert!(outcome.fallback_reason.is_some());
        }
        #[cfg(not(target_os = "windows"))]
        {
            let r = preallocate(path.to_str().unwrap(), 64 * 1024);
            assert!(r.is_err());
        }
    }
}
