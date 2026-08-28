//! M-2065.11: `IntegrityGate` 整文件 hash + 篡改负例 UT（per SPEC §6 + IMPL-PLAN §3.3）。
//!
//! 覆盖：
//! - 已知向量（SHA-256("abc") = ba7816bf...）
//! - 落盘文件 hash 匹配
//! - 篡改（落盘后改一个字节）→ Mismatch
//! - 期望 hash 大小写不敏感（uppercase → lowercase normalize）
//! - 空文件 hash
//! - 大文件 hash（4 MiB）

use rgs_asset_download::integrity_gate::{IntegrityGate, IntegrityStatus};
use std::io::Write;
use tempfile::tempdir;

#[tokio::test]
async fn known_vector_abc() {
    let gate = IntegrityGate::new();
    // 不验证文件路径；改用 hash_bytes
    let _ = gate.verify_for_test("ignored", "").await;
    let hex = IntegrityGate::hash_bytes(b"abc");
    assert_eq!(
        hex,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[tokio::test]
async fn match_for_known_payload() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.bin");
    let payload = b"hello, integrity";
    std::fs::File::create(&path)
        .unwrap()
        .write_all(payload)
        .unwrap();
    let expected = IntegrityGate::hash_bytes(payload);
    let gate = IntegrityGate::new();
    let report = gate.verify(path.to_str().unwrap(), &expected).await.unwrap();
    assert_eq!(report.status, IntegrityStatus::Match);
    assert_eq!(report.size_bytes, payload.len() as u64);
}

#[tokio::test]
async fn mismatch_detects_tampering() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.bin");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(b"original")
        .unwrap();
    let gate = IntegrityGate::new();
    let report = gate
        .verify(
            path.to_str().unwrap(),
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .await
        .unwrap();
    assert_eq!(report.status, IntegrityStatus::Mismatch);
    assert_eq!(
        report.actual_sha256,
        IntegrityGate::hash_bytes(b"original")
    );
}

#[tokio::test]
async fn mismatch_detects_single_byte_tamper() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.bin");
    let original = b"the quick brown fox";
    std::fs::File::create(&path)
        .unwrap()
        .write_all(original)
        .unwrap();
    let expected = IntegrityGate::hash_bytes(original);

    // 篡改一个字节
    let mut content = std::fs::read(&path).unwrap();
    content[0] ^= 0xFF;
    std::fs::write(&path, &content).unwrap();

    let gate = IntegrityGate::new();
    let report = gate
        .verify(path.to_str().unwrap(), &expected)
        .await
        .unwrap();
    assert_eq!(report.status, IntegrityStatus::Mismatch);
}

#[tokio::test]
async fn uppercase_expected_normalized() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.bin");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(b"abc")
        .unwrap();
    let gate = IntegrityGate::new();
    // 大写 expected 也应 normalize
    let report_upper = gate
        .verify(
            path.to_str().unwrap(),
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
        )
        .await
        .unwrap();
    assert_eq!(report_upper.status, IntegrityStatus::Match);
}

#[tokio::test]
async fn empty_file_hash() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty.bin");
    std::fs::File::create(&path).unwrap();
    let gate = IntegrityGate::new();
    let report = gate
        .verify(
            path.to_str().unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .await
        .unwrap();
    assert_eq!(report.status, IntegrityStatus::Match);
    assert_eq!(report.size_bytes, 0);
}

#[tokio::test]
async fn large_file_hash_4mb() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.bin");
    let mut file = std::fs::File::create(&path).unwrap();
    let chunk = vec![0xCDu8; 64 * 1024]; // 64 KiB
    for _ in 0..64 {
        file.write_all(&chunk).unwrap();
    }
    file.flush().unwrap();
    drop(file);

    // 4 MiB expected hash 算出来
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(vec![0xCDu8; 4 * 1024 * 1024]);
    let expected = hex::encode(hasher.finalize());

    let gate = IntegrityGate::new();
    let report = gate
        .verify(path.to_str().unwrap(), &expected)
        .await
        .unwrap();
    assert_eq!(report.status, IntegrityStatus::Match);
    assert_eq!(report.size_bytes, 4 * 1024 * 1024);
}

#[tokio::test]
async fn missing_file_returns_io_error() {
    let gate = IntegrityGate::new();
    let r = gate
        .verify(
            "/nonexistent/path/to/file.bin",
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .await;
    assert!(r.is_err());
}

/// 仅测试用：绕过文件 IO 直接 hash（用于 known-vector 单测）
trait IntegrityGateTestExt {
    async fn verify_for_test(&self, _path: &str, _expected: &str);
}

impl IntegrityGateTestExt for IntegrityGate {
    async fn verify_for_test(&self, _path: &str, _expected: &str) {
        // 桩：实际测 hash 用 hash_bytes；本方法只用于保持模块导出 IntegrityGate
    }
}
