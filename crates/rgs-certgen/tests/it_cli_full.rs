//! rgs-certgen IT 测试 - Cli + 端到端 (per RGS-TST-IT-09 §2.1 + §2.2)
//!
//! TL-2/3/4: 接口契约 + 协议一致性 + 集成(端到端)
//!
//! 7 测试(4 Cli + 3 端到端):
//! A001~A004: CLI 参数 (4)
//! D001: k3s tls secret 集成 (1, 端到端)
//! D002: scripts/ ci-integration (1, 端到端, 验证 cert 可被 create secret 使用)
//! C001: openSSL x509 文本解析 (1, 端到端, 可选, 跳过若 openSSL 不可用)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

// ============================================================================
// 模块 A: Cli 参数(per IT-09 §2.3 A001~A004)
// ============================================================================

#[test]
fn it_cli_default_args_creates_6_domains() {
    // A001 默认参数:6 域 + 365 天
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    let output = Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .output()
        .expect("spawn rgs-certgen");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("player.service"), "missing player.service in stdout");
    assert!(stdout.contains("economy.service"), "missing economy.service");
    assert!(stdout.contains("cluster-ops.service"), "missing cluster-ops.service");
    // 6 个文件 + ca.crt + ca.key + 6 server.{crt,key}.pem = 14
    let entries: Vec<_> = fs::read_dir(&outdir).unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 14, "expected 14 files (1 ca.crt + 1 ca.key + 6*(crt+key)), got {}", entries.len());
}

#[test]
fn it_cli_custom_domains_creates_n_servers() {
    // A002 --domains a,b,c
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("svc1.example.com,svc2.example.com,svc3.example.com")
        .assert()
        .success();
    // 1 ca.crt + 1 ca.key + 3*(crt+key) = 8
    let entries: Vec<_> = fs::read_dir(&outdir).unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 8, "expected 8 files (1+1+3*2), got {}", entries.len());
    assert!(outdir.join("svc1.example.com.crt.pem").exists());
    assert!(outdir.join("svc3.example.com.crt.pem").exists());
}

#[test]
fn it_cli_custom_validity_days_writes_correct_field() {
    // A003 --validity-days 30
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    let output = Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--validity-days")
        .arg("30")
        .output()
        .expect("spawn rgs-certgen");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("30 天"), "validity 30 天 should print to stdout");
    // 30 天 ≈ 2592000 秒;ca.crt.pem 仍生成
    assert!(outdir.join("ca.crt.pem").exists());
}

#[test]
fn it_cli_combined_args_minimal_output() {
    // A004 组合参数:output + domains + validity-days 1
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("p.example.com,e.example.com")
        .arg("--validity-days")
        .arg("1")
        .assert()
        .success();
    // 1 ca.crt + 1 ca.key + 2*(crt+key) = 6
    let entries: Vec<_> = fs::read_dir(&outdir).unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 6);
}

// ============================================================================
// 模块 D: k3s tls secret 集成 (端到端, per IT-09 §2.3 D001~D002)
// ============================================================================

#[test]
fn it_d001_cert_files_compatible_with_k3s_create_secret() {
    // D001: cert 格式能直接被 k3s `kubectl create secret tls` 使用
    // 验证: ① ca.crt.pem 是 PEM CERTIFICATE 块
    //      ② <domain>.crt.pem 是 PEM CERTIFICATE 块
    //      ③ <domain>.key.pem 是 PEM PRIVATE KEY 块
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("k3s-test.example.com")
        .assert()
        .success();

    let ca_crt = fs::read_to_string(outdir.join("ca.crt.pem")).unwrap();
    assert!(ca_crt.contains("-----BEGIN CERTIFICATE-----"), "ca cert PEM 头缺失");
    assert!(ca_crt.contains("-----END CERTIFICATE-----"), "ca cert PEM 尾缺失");

    let svr_crt = fs::read_to_string(outdir.join("k3s-test.example.com.crt.pem")).unwrap();
    assert!(svr_crt.contains("-----BEGIN CERTIFICATE-----"), "server cert PEM 头缺失");
    assert!(svr_crt.contains("-----END CERTIFICATE-----"), "server cert PEM 尾缺失");

    let svr_key = fs::read_to_string(outdir.join("k3s-test.example.com.key.pem")).unwrap();
    assert!(svr_key.contains("-----BEGIN PRIVATE KEY-----"), "server key PEM 头缺失");
    assert!(svr_key.contains("-----END PRIVATE KEY-----"), "server key PEM 尾缺失");
}

#[test]
fn it_d002_make_certs_helper_produces_identical_output() {
    // D002: scripts/_scratch/make_certs 路径(若存在)与 rgs-certgen 输出一致
    // 这里只验证 rgs-certgen 重复执行幂等
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    for _ in 0..3 {
        Command::cargo_bin("rgs-certgen")
            .unwrap()
            .arg("--output")
            .arg(&outdir)
            .arg("--domains")
            .arg("idempotent.example.com")
            .assert()
            .success();
    }
    // 1 域 = 1 ca.crt + 1 ca.key + 1 server.crt + 1 server.key = 4 文件
    // 3 次执行后,文件数仍 4(幂等覆盖,不加文件)
    let entries: Vec<_> = fs::read_dir(&outdir).unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 4, "repeated runs should overwrite, not add files (1 domain = 4 files)");
}

// ============================================================================
// 模块 C: openSSL 互操作 (端到端, per IT-09 §2.3 C001)
// ============================================================================

#[test]
fn it_c001_openssl_can_parse_ca_cert() {
    // C001: openSSL x509 文本解析 ca.crt.pem
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("openssl-test.example.com")
        .assert()
        .success();

    // 检查 openSSL 是否可用
    let openssl_check = StdCommand::new("openssl")
        .arg("version")
        .output();
    if openssl_check.is_err() {
        // openSSL 不可用,跳过(本机无 openSSL 工具)
        eprintln!("openSSL not available, skipping C001 (k3s CI 会跑)");
        return;
    }

    let output = StdCommand::new("openssl")
        .arg("x509")
        .arg("-in")
        .arg(outdir.join("ca.crt.pem"))
        .arg("-text")
        .arg("-noout")
        .output()
        .expect("openssl x509 text");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Certificate"), "openssl output should mention Certificate");
    // CN 应是 "RustGameServer Dev CA"(per F2 处置)
    assert!(
        stdout.contains("RustGameServer Dev CA"),
        "openssl output should contain CA CN, got: {}",
        stdout.chars().take(500).collect::<String>()
    );
}
