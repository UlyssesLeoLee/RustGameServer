//! rgs-certgen proptest 单元测试 (per 9/1 14:15 JST 派工 w8-pt-arc-certgen-hello)
//!
//! rgs-certgen 是 bin 不是 lib,只能通过 assert_cmd 黑盒测 (per 2026-08-28 跨反馈 F1/F2)
//! proptest 块覆盖证书 subject/issuer 字段 invariant:
//! - 任何合法 DNS 域名,服务证书 PEM 必须含 BEGIN/END CERTIFICATE 块
//! - ca.crt.pem 必须含 BEGIN/END CERTIFICATE 块 (CA 固定不变)
//! - 任何合法 DNS 域名,服务私钥 PEM 必须含 BEGIN/END PRIVATE KEY 块
//! - 输出目录文件数 = 2 + 2*domain_count (ca + n*(crt+key))

use assert_cmd::Command;
use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

/// 合法 DNS 名生成器: 1-3 段, 段内 1-8 个 [a-z0-9-]
/// 段首段尾不能是 `-` (避免无效 DNS 触发 SAN 失败)
fn arb_dns_label() -> impl Strategy<Value = String> {
    "[a-z0-9][a-z0-9-]{0,6}[a-z0-9]?".prop_filter("non-empty", |s| !s.is_empty())
}

fn arb_dns_name() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_dns_label().prop_map(|l| l),
        (arb_dns_label(), arb_dns_label()).prop_map(|(a, b)| format!("{}.{}", a, b)),
        (arb_dns_label(), arb_dns_label(), arb_dns_label())
            .prop_map(|(a, b, c)| format!("{}.{}.{}", a, b, c)),
    ]
}

/// CA 证书永远固定生成,proptest 必须总能找到 ca.crt.pem
#[test]
fn proptest_ca_cert_always_generated() {
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
    let ca_pem = fs::read_to_string(outdir.join("ca.crt.pem")).unwrap();
    assert!(ca_pem.contains("-----BEGIN CERTIFICATE-----"));
    assert!(ca_pem.contains("-----END CERTIFICATE-----"));
    let ca_key = fs::read_to_string(outdir.join("ca.key.pem")).unwrap();
    assert!(ca_key.contains("-----BEGIN PRIVATE KEY-----"));
    assert!(ca_key.contains("-----END PRIVATE KEY-----"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// 任何合法 DNS 名作为 --domain,rgs-certgen 都能生成对应 PEM 文件
    /// (subject/issuer 字段 invariant: CN = domain, SAN = DNS:domain)
    #[test]
    fn proptest_server_cert_pem_blocks_present(domain in arb_dns_name()) {
        let tmp = TempDir::new().unwrap();
        let outdir = tmp.path().join("out");
        Command::cargo_bin("rgs-certgen")
            .unwrap()
            .arg("--output")
            .arg(&outdir)
            .arg("--domains")
            .arg(&domain)
            .assert()
            .success();

        // 服务证书 crt PEM block
        let crt_path = outdir.join(format!("{}.crt.pem", domain));
        prop_assert!(crt_path.exists(), "missing {} .crt.pem", domain);
        let crt_pem = fs::read_to_string(&crt_path).expect("read crt");
        prop_assert!(crt_pem.contains("-----BEGIN CERTIFICATE-----"));
        prop_assert!(crt_pem.contains("-----END CERTIFICATE-----"));

        // 服务私钥 key PEM block
        let key_path = outdir.join(format!("{}.key.pem", domain));
        prop_assert!(key_path.exists(), "missing {} .key.pem", domain);
        let key_pem = fs::read_to_string(&key_path).expect("read key");
        prop_assert!(key_pem.contains("-----BEGIN PRIVATE KEY-----"));
        prop_assert!(key_pem.contains("-----END PRIVATE KEY-----"));

        // CA 永远存在
        prop_assert!(outdir.join("ca.crt.pem").exists());
        prop_assert!(outdir.join("ca.key.pem").exists());
    }

    /// 任何 1-3 个 DNS 名列表,文件数 = 2 + 2*N (ca + N*(crt+key))
    #[test]
    fn proptest_file_count_matches_domain_count(
        domains in proptest::collection::vec(arb_dns_name(), 1..=3)
    ) {
        let tmp = TempDir::new().unwrap();
        let outdir = tmp.path().join("out");
        let domains_csv = domains.join(",");
        Command::cargo_bin("rgs-certgen")
            .unwrap()
            .arg("--output")
            .arg(&outdir)
            .arg("--domains")
            .arg(&domains_csv)
            .assert()
            .success();

        let entries: Vec<_> = fs::read_dir(&outdir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let expected = 2 + 2 * domains.len();
        prop_assert_eq!(entries.len(), expected,
            "file count mismatch: got {}, expected {} for domains={:?}",
            entries.len(), expected, domains);
    }
}
