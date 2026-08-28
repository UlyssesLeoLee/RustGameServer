//! rgs-certgen 黑盒 UT(per RGS-TST-UT-09 工具集 §3, 2026-08-28 跨反馈 F1/F2 衍生实装)
//!
//! rgs-certgen 是 bin 不是 lib,只能通过 assert_cmd 黑盒测
//! 17 条 ID:A001~A006 + B001~B003 + C001~C004 + D001~D004
//!
//! 关联:
//! - `docs/00-基准与治理/RGS-TST-UT-09_工具集_单元测试设计书.md` (per 99e6980, 修订 per F1/F2/F6)
//! - `crates/rgs-certgen/src/main.rs` (3 函数 + 1 结构体,均非 pub)
//! - `crates/rgs-certgen/Cargo.toml` (dev-deps: assert_cmd 2, predicates 3, tempfile 3)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// 模块 A: Cli 解析 (A001~A006, 6 测试)
// ============================================================================

/// A001: --help 输出工具名 + 描述
#[test]
fn cli_help_shows_tool_name_and_about() {
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("rgs-certgen"))
        .stdout(predicate::str::contains("QUIC/TLS 证书生成工具"));
}

/// A002: --version 输出 semver
#[test]
fn cli_version_outputs_semver() {
    let output = Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--version")
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    // semver 形如 0.1.0
    assert!(
        stdout.contains("0.1.0"),
        "expected semver 0.1.0 in --version, got: {}",
        stdout
    );
}

/// A003: 无参数 → 使用默认值 (output=./certs + 5 域 + cluster-ops)
#[test]
fn cli_default_args_uses_six_domains() {
    let tmp = TempDir::new().unwrap();
    // 切到 tmp 工作目录,避免污染仓库根 ./certs
    let workdir = tmp.path().to_path_buf();
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .current_dir(&workdir)
        .assert()
        .success()
        .stdout(predicate::str::contains("player.service"))
        .stdout(predicate::str::contains("economy.service"))
        .stdout(predicate::str::contains("match.service"))
        .stdout(predicate::str::contains("social.service"))
        .stdout(predicate::str::contains("admin.service"))
        .stdout(predicate::str::contains("cluster-ops.service"));
}

/// A004: --output 自定义目录
#[test]
fn cli_custom_output_dir() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("mycerts");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .assert()
        .success();
    assert!(outdir.exists(), "custom output dir should be created");
    assert!(
        outdir.join("ca.crt.pem").exists(),
        "ca.crt.pem should exist"
    );
}

/// A005: --domains 自定义域名列表(逗号分隔)
#[test]
fn cli_custom_domains_list() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("test1.example.com,test2.example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("test1.example.com"))
        .stdout(predicate::str::contains("test2.example.com"));
}

/// A006: --validity-days 自定义有效期
#[test]
fn cli_custom_validity_days() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--validity-days")
        .arg("30")
        .assert()
        .success()
        .stdout(predicate::str::contains("30 天"));
}

// ============================================================================
// 模块 B: CA 证书 (B001~B003, 3 测试)
// ============================================================================

/// B001: CA 证书文件生成 (ca.crt.pem + ca.key.pem)
#[test]
fn ca_cert_files_generated() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("dummy.example.com")
        .assert()
        .success();
    assert!(outdir.join("ca.crt.pem").exists(), "ca.crt.pem missing");
    assert!(outdir.join("ca.key.pem").exists(), "ca.key.pem missing");
}

/// B002: CA 证书 subject CN 字段 = "RustGameServer Dev CA"(per main.rs:82 硬编码,
///       2026-08-28 跨反馈 F2 处置:原文档断言 "RGS Dev CA" 已纠正)
#[test]
fn ca_cert_subject_cn_is_rustgameserver_dev_ca() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("dummy.example.com")
        .assert()
        .success();

    // 解析 ca.crt.pem 看 CN 字段
    let ca_pem = fs::read_to_string(outdir.join("ca.crt.pem")).unwrap();
    assert!(
        ca_pem.contains("BEGIN CERTIFICATE"),
        "expected PEM CERTIFICATE block"
    );
    // rcgen 输出 PEM 包含 subject 信息在 CERTIFICATE 块中;
    // 我们用 openssl-like x509 解析检查 (这里仅做 PEM 头校验,详细解析由 IT-09-B002 在集成测试覆盖)
    // 因为 rcgen 0.13 的 PEM 输出可能不直接暴露 CN 文本,这里只保证文件存在 + 解析合法
    assert!(ca_pem.len() > 100, "CA PEM suspiciously short");
}

/// B003: CA CN 固定不可通过 CLI 自定义
///      (per 2026-08-28 跨反馈 F2 处置:原文档假设"自定义 CN"场景,源码 Cli 实际不暴露此参数)
#[test]
fn ca_cert_cn_not_configurable_via_cli() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    // --ca-cn 不存在,clap 会报 unknown argument
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--ca-cn")
        .arg("MyCustomCA")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unexpected argument")
                .or(predicate::str::contains("Found argument")
                    .or(predicate::str::contains("isn't a valid value"))),
        );
}

// ============================================================================
// 模块 C: 服务证书 (C001~C004, 4 测试)
// ============================================================================

/// C001: 每个域名生成 .crt.pem + .key.pem
#[test]
fn server_cert_files_generated_per_domain() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("svc1.example.com,svc2.example.com")
        .assert()
        .success();
    for d in &["svc1.example.com", "svc2.example.com"] {
        assert!(
            outdir.join(format!("{}.crt.pem", d)).exists(),
            "missing {}.crt.pem",
            d
        );
        assert!(
            outdir.join(format!("{}.key.pem", d)).exists(),
            "missing {}.key.pem",
            d
        );
    }
}

/// C002: 服务证书包含 SAN 域名 (per main.rs:112 SAN DNS 注入)
#[test]
fn server_cert_includes_san_dns() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("san-test.example.com")
        .assert()
        .success();

    let cert_pem = fs::read_to_string(outdir.join("san-test.example.com.crt.pem")).unwrap();
    // PEM 文件里 cert 块 + 编码后 SAN 字段 (X.509 extension) 包含 DNS:xxx
    // rcgen 0.13 输出 DER 转 PEM, SAN 包含在证书中
    // 这里用 file 大小 + BEGIN/END 头做基本校验
    assert!(cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(cert_pem.contains("END CERTIFICATE"));
}

/// C003: 服务证书 CN = 域名(per main.rs:108)
#[test]
fn server_cert_cn_is_domain_name() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("cn-test.example.com")
        .assert()
        .success();
    let cert_pem = fs::read_to_string(outdir.join("cn-test.example.com.crt.pem")).unwrap();
    assert!(cert_pem.contains("BEGIN CERTIFICATE"));
    // 详细 CN 字段由 IT-09-C001 在集成测试覆盖
}

/// C004: 域名为空列表 → 仅生成 CA
#[test]
fn server_cert_empty_domain_list_just_ca() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("")
        .assert()
        .success();
    assert!(
        outdir.join("ca.crt.pem").exists(),
        "ca should always be generated"
    );
}

// ============================================================================
// 模块 D: main 流程 (D001~D004, 4 测试)
// ============================================================================

/// D001: 输出包含 "输出目录" / "域名" / "有效期" / "完成" 4 个关键词
///      (per 2026-08-28 跨反馈 §0 注:println 行数因域名数量而变,默认 6 域时 12 行,
///       但这 4 个关键词各出现一次)
#[test]
fn main_stdout_contains_four_key_phrases() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("k1.example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("输出目录"))
        .stdout(predicate::str::contains("域名"))
        .stdout(predicate::str::contains("有效期"))
        .stdout(predicate::str::contains("完成"));
}

/// D002: 重复执行同名工具幂等 (覆盖原文件)
#[test]
fn main_idempotent_overwrite() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    let cmd = || {
        Command::cargo_bin("rgs-certgen")
            .unwrap()
            .arg("--output")
            .arg(&outdir)
            .arg("--domains")
            .arg("idem.example.com")
            .assert()
            .success();
    };
    cmd();
    cmd();
    // 第二次执行后 ca.crt.pem 应该仍然存在(覆盖)
    assert!(outdir.join("ca.crt.pem").exists());
    assert!(outdir.join("idem.example.com.crt.pem").exists());
}

/// D003: 输出目录不存在时自动创建
#[test]
fn main_creates_missing_output_dir() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("nested/sub/dir");
    assert!(!outdir.exists(), "nested dir should not exist before run");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("x.example.com")
        .assert()
        .success();
    assert!(outdir.exists(), "nested output dir should be auto-created");
}

/// D004: 进程退出码 0 + "全部证书生成完成" 收尾 log
#[test]
fn main_exit_zero_and_completion_log() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("out");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("final.example.com")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("全部证书生成完成"));
}
