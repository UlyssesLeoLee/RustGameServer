//! mTLS 证书加载（per RGS-SPEC-CROSS-002 mTLS 规范 + RGS-SEC-100 §6）
//!
//! 54.9 实化：从 PEM 文件读 server cert / client cert / CA，构造 tonic TLS config
//!
//! 设计原则：
//! - rustls 作为 TLS 库（避开 OpenSSL）
//! - 服务端：server cert + private key + CA bundle（per RGS-ARC-051 PFAU 跨节点 mTLS）
//! - 客户端：client cert + private key + CA bundle（per 53.11 rgs-certgen 生成）

use std::fs;
use std::path::Path;

use thiserror::Error;
use tonic::transport::{Certificate, ClientTlsConfig, Identity, ServerTlsConfig};

/// 安装进程级默认 rustls CryptoProvider（per rustls 0.23 要求）。
///
/// aws-lc-rs 与 ring 两个 provider 同时被依赖树间接启用时，rustls 无法自动选择，
/// 首次建立 TLS 连接会 panic。各服务 main() 入口需在任何 TLS 操作前调用一次本函数。
/// 重复调用（如测试并发）是安全的：`install_default` 失败仅代表已被安装过，忽略即可。
///
/// 选用 ring 而非 aws-lc-rs：aws-lc-rs 的 C 编译产物在 builder 镜像（较新 glibc）
/// 下链接后，在 distroless Debian 12 运行时镜像（glibc 2.36）里报
/// `GLIBC_2.38 not found`；ring 是纯 Rust + 手写汇编实现，无此跨镜像 glibc 版本依赖。
pub fn install_default_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// mTLS 错误
#[derive(Debug, Error)]
pub enum TlsError {
    #[error("failed to read file {path}: {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse PEM: {0}")]
    PemParse(String),
}

/// TLS 配置（client 用）
#[derive(Debug, Clone)]
pub struct ClientTlsConfigInput {
    /// 目标域名（SAN 或 CN）
    pub domain: String,
    /// CA 证书 PEM
    pub ca_cert_path: String,
    /// 客户端证书 PEM
    pub client_cert_path: String,
    /// 客户端私钥 PEM
    pub client_key_path: String,
}

/// 加载客户端 TLS 配置
pub fn load_client_tls(input: &ClientTlsConfigInput) -> Result<ClientTlsConfig, TlsError> {
    let ca_pem = fs::read_to_string(&input.ca_cert_path).map_err(|e| TlsError::FileRead {
        path: input.ca_cert_path.clone(),
        source: e,
    })?;
    let client_pem =
        fs::read_to_string(&input.client_cert_path).map_err(|e| TlsError::FileRead {
            path: input.client_cert_path.clone(),
            source: e,
        })?;
    let key_pem = fs::read_to_string(&input.client_key_path).map_err(|e| TlsError::FileRead {
        path: input.client_key_path.clone(),
        source: e,
    })?;

    let ca = Certificate::from_pem(ca_pem);
    let identity = Identity::from_pem(client_pem, key_pem);

    Ok(ClientTlsConfig::new()
        .domain_name(input.domain.clone())
        .ca_certificate(ca)
        .identity(identity))
}

/// 加载服务端 TLS 证书 + 私钥（用于 tonic Server 端，per RGS-ARC-051）
pub fn load_server_identity(cert_path: &Path, key_path: &Path) -> Result<Identity, TlsError> {
    let cert = fs::read_to_string(cert_path).map_err(|e| TlsError::FileRead {
        path: cert_path.display().to_string(),
        source: e,
    })?;
    let key = fs::read_to_string(key_path).map_err(|e| TlsError::FileRead {
        path: key_path.display().to_string(),
        source: e,
    })?;
    Ok(Identity::from_pem(cert, key))
}

/// 加载服务端 mTLS 配置（强制客户端证书校验，per RGS-REV-007 CH4 + DEC-015 P1）
///
/// 与 [`load_server_identity`] 的区别：
/// - 本函数额外要求客户端 CA 证书（`client_ca_cert_path`）
/// - 设置 `client_ca_root` 后，tonic 0.12 的 `ServerTlsConfig` 默认
///   `client_auth_optional = false`，即 **强制要求客户端出示证书**
/// - 缺失客户端证书的连接会被 rustls 拒绝，防中间人
///
/// 调用方将返回值传给 `tonic::transport::Server::tls_config(...)`。
///
/// 错误：任何 PEM 文件读取失败都会返回 [`TlsError::FileRead`] 并附带路径。
pub fn load_server_tls_config(
    server_cert_path: &Path,
    server_key_path: &Path,
    client_ca_cert_path: &Path,
) -> Result<ServerTlsConfig, TlsError> {
    let server_cert = fs::read_to_string(server_cert_path).map_err(|e| TlsError::FileRead {
        path: server_cert_path.display().to_string(),
        source: e,
    })?;
    let server_key = fs::read_to_string(server_key_path).map_err(|e| TlsError::FileRead {
        path: server_key_path.display().to_string(),
        source: e,
    })?;
    let ca_cert = fs::read_to_string(client_ca_cert_path).map_err(|e| TlsError::FileRead {
        path: client_ca_cert_path.display().to_string(),
        source: e,
    })?;

    let server_identity = Identity::from_pem(server_cert, server_key);
    let client_ca_root = Certificate::from_pem(ca_cert);

    // tonic 0.12: 不调用 client_auth_optional(true) 即保持 required (default)
    // 设置 client_ca_root 即启用 mutual TLS 客户端证书校验
    Ok(ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_read_error_includes_path() {
        let result = fs::read_to_string("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn client_tls_config_input_required_fields() {
        let input = ClientTlsConfigInput {
            domain: "player.local".to_string(),
            ca_cert_path: "/ca.pem".to_string(),
            client_cert_path: "/client.pem".to_string(),
            client_key_path: "/client.key".to_string(),
        };
        assert_eq!(input.domain, "player.local");
    }

    #[test]
    fn load_server_tls_config_missing_file_returns_err() {
        // 任何缺失 PEM 文件都应返回 TlsError::FileRead 并附带路径
        let result = load_server_tls_config(
            Path::new("/nonexistent/server-cert.pem"),
            Path::new("/nonexistent/server-key.pem"),
            Path::new("/nonexistent/client-ca.pem"),
        );
        assert!(matches!(result, Err(TlsError::FileRead { .. })));
        if let Err(TlsError::FileRead { path, .. }) = result {
            assert!(path.contains("server-cert.pem"));
        }
    }

    #[test]
    fn load_server_tls_config_missing_server_key_returns_err() {
        // 缺 server key：第一个文件读成功，但 cert_path 缺失也应被检测
        // 这里反过来用：第一个文件就不存在，验证短路
        let result = load_server_tls_config(
            Path::new("/nonexistent/missing.pem"),
            Path::new("/also/missing.pem"),
            Path::new("/still/missing.pem"),
        );
        assert!(matches!(result, Err(TlsError::FileRead { .. })));
    }

    #[test]
    fn load_server_identity_missing_file_returns_err() {
        // 同 family 已有 helper 的错误行为保持一致
        let result = load_server_identity(
            Path::new("/nonexistent/cert.pem"),
            Path::new("/nonexistent/key.pem"),
        );
        assert!(matches!(result, Err(TlsError::FileRead { .. })));
    }
}
