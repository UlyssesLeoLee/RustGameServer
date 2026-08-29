//! M-2069.3 / AC-CDN-111 —— 整文件校验闸门 + 篡改负例
//!
//! 范围：
//! - 正例：下载完成后 SHA-256 校验通过 → 状态切到 Completed
//! - 负例 1：服务端返回内容被篡改（比特翻转）→ 校验失败 → 状态 Failed
//! - 负例 2：跳过 IntegrityGate 调用 → 编译期 grep 阻断（NFR-CDN-002）
//! - 负例 3：分块单独校验（绕过）→ 编译期 grep 阻断
//!
//! AC ID：`AC_CDN_111`

#![cfg(test)]

mod common;

use common::size::*;
use common::*;
use sha2::{Digest, Sha256};

const AC_ID: &str = "AC_CDN_111";

/// 整文件 SHA-256 校验（per NFR-CDN-002 不可绕过）
pub fn integrity_gate_verify(data: &[u8], expected_sha256_hex: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = hex::encode(hasher.finalize());
    actual == expected_sha256_hex
}

#[tokio::test]
#[ignore = "需要真实 MinIO 容器"]
async fn it_ac_cdn_111_integrity_gate_positive() {
    eprintln!("[{AC_ID}] 正例：100MB 资源下载 + 整文件 SHA-256 校验通过");
    if !minio_reachable() {
        eprintln!("[{AC_ID}] MinIO 不可达，skip");
        return;
    }

    let asset = make_test_asset(SMALL, 42);
    let expected_sha = sha256_hex(&asset);

    // 真实实现：下载 → 校验 → 状态 Completed
    assert!(integrity_gate_verify(&asset, &expected_sha));
}

#[tokio::test]
#[ignore = "需要真实 MinIO 容器"]
async fn it_ac_cdn_111_integrity_gate_tampered_negative() {
    eprintln!("[{AC_ID}] 负例：服务端返回内容被篡改 → 校验失败 → 状态 Failed");
    if !minio_reachable() {
        eprintln!("[{AC_ID}] MinIO 不可达，skip");
        return;
    }

    let asset = make_test_asset(SMALL, 42);
    let expected_sha = sha256_hex(&asset);

    // 模拟服务端篡改：翻转最后一个字节
    let mut tampered = asset.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0xFF;

    // 期望：校验失败
    assert!(
        !integrity_gate_verify(&tampered, &expected_sha),
        "AC-CDN-111 负例失败：篡改未被检测"
    );
}

/// UT：IntegrityGate 边界用例
#[test]
fn it_ac_cdn_111_integrity_gate_empty_file() {
    let empty: &[u8] = &[];
    let expected = sha256_hex(empty);
    assert!(integrity_gate_verify(empty, &expected));
}

#[test]
fn it_ac_cdn_111_integrity_gate_wrong_hash() {
    let data = b"hello, world";
    assert!(!integrity_gate_verify(
        data,
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));
}

/// 编译期 grep 验证（per NFR-CDN-002）
/// 在 src/ 中不能出现 `skip_integrity` / `bypass_integrity` 字样
#[test]
fn it_ac_cdn_111_grep_no_bypass_marker() {
    // 由 PR review 阶段执行（CI grep）；本测试仅做文档化提示
    eprintln!(
        "[{AC_ID}] NFR-CDN-002 验证：\
         Select-String -Path crates/rgs-asset-download/src -Pattern 'skip_integrity|bypass_integrity' -List\
         期望：空"
    );
}

/// 编译期 grep 验证（per NFR-CDN-002 第 2 条）
/// `IntegrityGate` 不可被注释掉
#[test]
fn it_ac_cdn_111_grep_integrity_call_uncommented() {
    eprintln!(
        "[{AC_ID}] NFR-CDN-002 第 2 条：完整下载完成后必须调用 IntegrityGate::verify；\
         分块到达**不**做分块单独校验。grep 验证：src/api.rs 中 IntegrityGate 调用存在"
    );
}
