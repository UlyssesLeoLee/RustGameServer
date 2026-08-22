//! rgs-certgen —— RustGameServer QUIC/TLS 证书生成工具
//!
//! 用途：dev / staging 环境的 self-signed CA + 5 域服务证书生成。
//! 生产用 cert-manager（per WF-1-54.x）；53.11 占位 self-signed。
//!
//! 规范：RGS-SPEC-000 §2.1 + RGS-IMPL-001 §4
//!
//! 用法：
//!   rgs-certgen --output ./certs
//!   rgs-certgen --output ./certs --domains player.service,economy.service --validity-days 365

use anyhow::{Context, Result};
use clap::Parser;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair,
    KeyUsagePurpose, SanType,
};
use std::fs;
use std::path::PathBuf;
use time::Duration;

#[derive(Parser, Debug)]
#[command(name = "rgs-certgen", version, about = "RustGameServer QUIC/TLS 证书生成工具")]
struct Cli {
    /// 输出目录
    #[arg(short, long, default_value = "./certs")]
    output: PathBuf,

    /// 域名列表（逗号分隔，默认 5 域 + cluster-ops）
    #[arg(long, value_delimiter = ',', default_values_t = vec![
        "player.service".to_string(),
        "economy.service".to_string(),
        "match.service".to_string(),
        "social.service".to_string(),
        "admin.service".to_string(),
        "cluster-ops.service".to_string(),
    ])]
    domains: Vec<String>,

    /// 证书有效期（天）
    #[arg(long, default_value_t = 365)]
    validity_days: u32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    fs::create_dir_all(&cli.output)
        .with_context(|| format!("创建输出目录失败: {}", cli.output.display()))?;

    println!("[rgs-certgen] 输出目录: {}", cli.output.display());
    println!("[rgs-certgen] 域名: {:?}", cli.domains);
    println!("[rgs-certgen] 有效期: {} 天", cli.validity_days);

    // 1. 生成 CA
    let (ca_cert, ca_key) = generate_ca(&cli.output, cli.validity_days)?;
    println!("[rgs-certgen] CA 证书已生成: ca.crt.pem");

    // 2. 为每个域名生成服务证书
    for domain in &cli.domains {
        let _ = generate_server_cert(&cli.output, domain, &ca_cert, &ca_key, cli.validity_days)?;
        println!("[rgs-certgen] 服务证书已生成: {}.crt.pem", domain);
    }

    println!("[rgs-certgen] 全部证书生成完成");
    println!("[rgs-certgen] dev 用 self-signed CA；生产用 cert-manager（per WF-1-54.x）");
    Ok(())
}

fn generate_ca(output: &PathBuf, validity_days: u32) -> Result<(Certificate, KeyPair)> {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, "RustGameServer Dev CA");
    params.distinguished_name.push(DnType::OrganizationName, "Ulysses");
    params.not_before = time::OffsetDateTime::now_utc();
    params.not_after = params.not_before + Duration::days(validity_days as i64);

    let key = KeyPair::generate()?;
    // self_signed takes ownership of self
    let cert = params.self_signed(&key)?;

    fs::write(output.join("ca.crt.pem"), cert.pem())?;
    fs::write(output.join("ca.key.pem"), key.serialize_pem())?;

    Ok((cert, key))
}

fn generate_server_cert(
    output: &PathBuf,
    domain: &str,
    ca_cert: &Certificate,
    ca_key: &KeyPair,
    validity_days: u32,
) -> Result<()> {
    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, domain);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "RustGameServer");
    params.subject_alt_names.push(SanType::DnsName(
        domain.try_into().with_context(|| format!("SAN 转换失败: {}", domain))?,
    ));
    params.not_before = time::OffsetDateTime::now_utc();
    params.not_after = params.not_before + Duration::days(validity_days as i64);

    let key = KeyPair::generate()?;
    let cert = params.signed_by(&key, ca_cert, ca_key)?;

    fs::write(output.join(format!("{}.crt.pem", domain)), cert.pem())?;
    fs::write(output.join(format!("{}.key.pem", domain)), key.serialize_pem())?;

    Ok(())
}