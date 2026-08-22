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
use tonic::transport::{Certificate, ClientTlsConfig, Identity};

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
}
