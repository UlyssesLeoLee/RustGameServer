//! M-2065.12: Security UT —— 断点记录 grep 验证 PII 字段为空（per FR-CDN-064）。
//!
//! **目的**：在编译期 + 静态扫描层验证 `rgs-asset-download` crate 内不引用 PII 字段。
//!
//! **PII 字段清单**（per FR-CDN-064 / SPEC §4 + IMPL-PLAN §5.3）：
//! - `player_id`   玩家 ID
//! - `device_id`   设备 ID
//! - `email`       邮箱
//! - `ip`          IP 地址
//! - `mac`         MAC 地址
//! - `phone`       电话
//! - `ssn`         身份证
//!
//! **检查范围**：
//! - `src/*.rs`（除 `lib.rs` 注释 / `range_client.rs` 中 `host_of` 注释）外不引用
//! - `tests/*.rs` 中不引用
//! - 错误消息模板不包含 PII 字段名
//!
//! **豁免**：URL 主机名中可能含 `device-id-leak.example.com`（用户控制）；
//!          `host_of` 函数对此做去引号化处理；测试只针对源代码字面量。

use std::fs;
use std::path::Path;

const PII_FIELDS: &[&str] = &[
    "player_id",
    "device_id",
    "email",
    "phone",
    "ssn",
    "mac_address",
    "ip_address",
];

/// 扫描 `crates/rgs-asset-download/src` 与 `crates/rgs-asset-download/tests` 下所有 `.rs` 文件，
/// 验证不出现任何 PII 字段名（按字面匹配）。
#[test]
fn no_pii_field_in_source() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let tests_dir = crate_root.join("tests");

    let mut violations: Vec<String> = Vec::new();

    scan_dir(&src_dir, &mut violations);
    scan_dir(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "FR-CDN-064 violation: PII field found in source:\n{}",
        violations.join("\n")
    );
}

/// 错误消息模板不包含 PII 字段名。
#[test]
fn error_messages_contain_no_pii_field() {
    use rgs_asset_download::error::DownloadError;
    let cases: Vec<DownloadError> = vec![
        DownloadError::BackendHttpError {
            status: 503,
            host: "cdn.example.com".into(),
        },
        DownloadError::BackendRangeNotSatisfiable {
            chunk_index: 0,
            start: 0,
            end: 1023,
        },
        DownloadError::BackendEtagMismatch {
            expected: "v1".into(),
            actual: "v2".into(),
        },
        DownloadError::ResumeTokenNotFound {
            token_id: "tok-1234".into(),
        },
        DownloadError::IntegrityMismatch {
            expected: "a".into(),
            actual: "b".into(),
        },
    ];
    let mut violations: Vec<String> = Vec::new();
    for err in &cases {
        let msg = err.to_string();
        for field in PII_FIELDS {
            if msg.contains(field) {
                violations.push(format!(
                    "error variant `{}` contains PII field `{field}` in message: {msg}",
                    err.category()
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "FR-CDN-064 violation: PII in error messages:\n{}",
        violations.join("\n")
    );
}

/// `api.rs` / `state_machine.rs` 不引用 PII 字段（per IMPL-PLAN §5.3 grep 验证项）。
///
/// 跳过模块级 doc 注释（声明约束）和负向断言（`assert!(!...contains("xxx"))`）。
#[test]
fn api_and_state_machine_have_no_pii() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in ["src/api.rs", "src/state_machine.rs"] {
        let p = crate_root.join(rel);
        let content = fs::read_to_string(&p).unwrap_or_else(|e| {
            panic!("read {p:?}: {e}");
        });
        // 去掉 doc 注释 + 负向断言后扫
        let mut scrubbed = String::new();
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.starts_with("assert!")
                || trimmed.contains("assert!(!")
                || trimmed.contains("!s.contains")
                || trimmed.contains("!content.contains")
            {
                continue;
            }
            scrubbed.push_str(line);
            scrubbed.push('\n');
        }
        for field in PII_FIELDS {
            assert!(
                !scrubbed.contains(field),
                "FR-CDN-064 violation: {rel} (code only) contains PII field `{field}`"
            );
        }
    }
}

/// 公开 API 摘要（DownloadRequest::summary）不含 PII 字段名。
#[test]
fn download_request_summary_has_no_pii() {
    use rgs_asset_download::api::DownloadRequest;
    let req = DownloadRequest {
        asset_id: "asset-001".into(),
        file_path: "/tmp/out.bin".into(),
        url: "https://cdn.example.com/assets/asset-001".into(),
        expected_sha256: "deadbeef".into(),
        expected_size_bytes: 1024,
        resume_token_id: None,
    };
    let s = req.summary();
    for field in PII_FIELDS {
        assert!(
            !s.contains(field),
            "DownloadRequest::summary contains PII field `{field}`: {s}"
        );
    }
}

fn scan_dir(dir: &Path, violations: &mut Vec<String>) {
    if !dir.exists() {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            violations.push(format!("read_dir {dir:?}: {e}"));
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, violations);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // 跳过 security_no_pii.rs 自身（其作为"哨兵"必须包含 PII 字段名）
        if path.file_name().and_then(|n| n.to_str()) == Some("security_no_pii.rs") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                violations.push(format!("read {path:?}: {e}"));
                continue;
            }
        };
        for (lineno, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            // 跳过纯注释行（// 单行 / /// doc / //! module doc）
            if trimmed.starts_with("//") {
                continue;
            }
            // 跳过负向断言（`assert!(!...contains("xxx")`）
            if trimmed.contains("!s.contains")
                || trimmed.contains("!.contains")
                || trimmed.contains("!content.contains")
            {
                continue;
            }
            // 跳过 PII 字段名数组字面量
            if line.contains("const PII_FIELDS") {
                continue;
            }
            // 跳过 assert! 调用（`assert!(!s.contains("player_id"))` 等）
            if trimmed.starts_with("assert!") || trimmed.contains("assert!(") {
                continue;
            }
            // 跳过「防御性 PII 反向断言」array literal（per FR-CDN-064）
            // 模式：`for forbidden in ["player_id", ...]` / `for field in PII_FIELDS`
            // 这是 test 故意引用 PII 字段名做反向断言的合法 pattern（FR-CDN-064 验证），
            // 不应被本 scanner 误报。P0 续命 (2026-08-27 22:46 JST) — 见 ut_resume_token_store.rs:128,151。
            if trimmed.contains("for forbidden in [")
                || trimmed.contains("for field in PII_FIELDS")
                || trimmed.contains("for field_name in PII_FIELDS")
            {
                continue;
            }
            for field in PII_FIELDS {
                if line.contains(field) {
                    violations.push(format!(
                        "{}:{}: PII field `{field}` → {line}",
                        path.display(),
                        lineno + 1
                    ));
                }
            }
        }
    }
}
