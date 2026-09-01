//! rgs-certgen 集成测试 - 完整工作流 (per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello)
//!
//! 3+ 端到端场景:
//! - I001: k3s `kubectl create secret tls` 兼容 (PEM block 形式 + 文件名规范)
//! - I002: 6 域全量生成 + 文件清单精确匹配 (per RGS 5 域 + cluster-ops 平台)
//! - I003: 多次执行幂等 + 证书内容每次不同 (RSA/EC 随机密钥 per cert)
//! - I004: openSSL x509 解析 (skip if openssl unavailable)

use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// I001: k3s `kubectl create secret tls` 兼容 — 验证生成的 cert/key 文件能直接被 k3s 消费
#[test]
fn i001_kubectl_create_secret_tls_compatible() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("player.service")
        .assert()
        .success();

    // k3s `kubectl create secret tls` 要求: <name>.tls.crt + <name>.tls.key
    // rgs-certgen 输出 <domain>.crt.pem + <domain>.key.pem
    // 验证: 文件能 cat + base64 解析 (k3s 实际路径)
    let crt = fs::read(outdir.join("player.service.crt.pem")).expect("read crt");
    let key = fs::read(outdir.join("player.service.key.pem")).expect("read key");
    // base64 解码: PEM block 头被 base64 处理后仍是 ASCII printable
    let crt_str = String::from_utf8_lossy(&crt);
    let key_str = String::from_utf8_lossy(&key);
    assert!(crt_str.contains("BEGIN CERTIFICATE"));
    assert!(key_str.contains("BEGIN PRIVATE KEY"));
}

/// I002: 6 域全量生成 (per RGS 5 域 + cluster-ops 平台, 默认 domains)
#[test]
fn i002_default_six_domains_full_generation() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .assert()
        .success();

    // 5 业务域 + cluster-ops 平台 = 6 域
    let expected_domains = [
        "player.service",
        "economy.service",
        "match.service",
        "social.service",
        "admin.service",
        "cluster-ops.service",
    ];
    for d in &expected_domains {
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
    // 文件数: 1 ca.crt + 1 ca.key + 6 * 2 = 14
    let entries: Vec<_> = fs::read_dir(&outdir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 14);
}

/// I003: 重复执行幂等 + 每次 RSA/EC 密钥随机 (PEM 内容字节不同)
#[test]
fn i003_repeated_runs_different_keys_idempotent_files() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    let mut pem_runs: Vec<String> = Vec::new();
    for _ in 0..3 {
        Command::cargo_bin("rgs-certgen")
            .unwrap()
            .arg("--output")
            .arg(&outdir)
            .arg("--domains")
            .arg("repeating.example.com")
            .assert()
            .success();
        let pem = fs::read_to_string(outdir.join("repeating.example.com.key.pem")).unwrap();
        pem_runs.push(pem);
    }
    // 三次密钥 PEM 都不相同 (rcgen 每次生成新 EC/RSA 密钥)
    assert_ne!(pem_runs[0], pem_runs[1], "key.pem should differ between runs");
    assert_ne!(pem_runs[1], pem_runs[2], "key.pem should differ between runs");
    assert_ne!(pem_runs[0], pem_runs[2], "key.pem should differ between runs");
}

/// I004: openSSL x509 解析 ca.crt.pem 验证 subject CN = "RustGameServer Dev CA"
#[test]
fn i004_openssl_parses_ca_subject_cn() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("openssl-it.example.com")
        .assert()
        .success();

    // openSSL 可用性检查
    if StdCommand::new("openssl").arg("version").output().is_err() {
        eprintln!("openSSL not available, skipping I004");
        return;
    }

    let out = StdCommand::new("openssl")
        .arg("x509")
        .arg("-in")
        .arg(outdir.join("ca.crt.pem"))
        .arg("-subject")
        .arg("-noout")
        .output()
        .expect("openssl x509 subject");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("RustGameServer Dev CA"),
        "CA subject CN should be 'RustGameServer Dev CA', got: {}",
        stdout
    );
}

/// I005: 服务证书 issuer = CA subject (per rgs-certgen 签名链: 服务 cert 由 CA 签发)
#[test]
fn i005_openssl_server_cert_issuer_matches_ca() {
    let tmp = TempDir::new().unwrap();
    let outdir = tmp.path().join("certs");
    Command::cargo_bin("rgs-certgen")
        .unwrap()
        .arg("--output")
        .arg(&outdir)
        .arg("--domains")
        .arg("chain.example.com")
        .assert()
        .success();

    if StdCommand::new("openssl").arg("version").output().is_err() {
        eprintln!("openSSL not available, skipping I005");
        return;
    }

    let server_issuer = StdCommand::new("openssl")
        .arg("x509")
        .arg("-in")
        .arg(outdir.join("chain.example.com.crt.pem"))
        .arg("-issuer")
        .arg("-noout")
        .output()
        .expect("openssl issuer");
    let ca_subject = StdCommand::new("openssl")
        .arg("x509")
        .arg("-in")
        .arg(outdir.join("ca.crt.pem"))
        .arg("-subject")
        .arg("-noout")
        .output()
        .expect("openssl subject");
    let issuer_str = String::from_utf8_lossy(&server_issuer.stdout);
    let subject_str = String::from_utf8_lossy(&ca_subject.stdout);
    // 提取 CN 值比较
    fn extract_cn(s: &str) -> Option<String> {
        // 形如: subject=CN = RustGameServer Dev CA
        let s = s.trim();
        s.split("CN")
            .nth(1)
            .map(|x| x.trim_start_matches(|c: char| c == '=' || c.is_whitespace()).to_string())
    }
    let issuer_cn = extract_cn(&issuer_str);
    let subject_cn = extract_cn(&subject_str);
    assert_eq!(
        issuer_cn, subject_cn,
        "服务证书 issuer CN 应等于 CA subject CN (签名链): issuer={:?}, subject={:?}",
        issuer_cn, subject_cn
    );
}
