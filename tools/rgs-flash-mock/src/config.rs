//! rgs-flash-mock config
//!
//! 加载环境变量 + 5 域 gRPC endpoint + mTLS cert 路径 (per 8/27 11:06 JST hard ban: 凭据走 env var 永不打印)
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.1 §2.1 工具链

use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub log_level: String,
    pub tls_dir: String,
    pub service_name: String,
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
    pub player_endpoint: String,
    pub economy_endpoint: String,
    pub match_endpoint: String,
    pub social_endpoint: String,
    pub admin_endpoint: String,
    pub card_endpoint: String,
}

impl Config {
    /// 从 env 加载配置, 缺失必填项返回 Err
    pub fn from_env() -> Result<Self> {
        // 8/27 11:06 JST hard ban: 只 invoke env, 不打印 value
        let bind_addr = env::var("RGS_GAP_MOCK_BIND").unwrap_or_else(|_| "0.0.0.0:8791".to_string());
        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info,rgs_flash_mock=debug".to_string());
        let tls_dir = env::var("RGS_TLS_DIR").unwrap_or_else(|_| "/etc/rgs/certs".to_string());
        let service_name = "rgs-flash-mock".to_string();

        // 5 域 gRPC endpoint (per 8/27 凭据硬 ban, 永不打 value)
        let player_endpoint = env::var("GRPC_PLAYER_ENDPOINT")
            .unwrap_or_else(|_| "https://player-service:50051".to_string());
        let economy_endpoint = env::var("GRPC_ECONOMY_ENDPOINT")
            .unwrap_or_else(|_| "https://economy-service:50052".to_string());
        let match_endpoint = env::var("GRPC_MATCH_ENDPOINT")
            .unwrap_or_else(|_| "https://match-service:50053".to_string());
        let social_endpoint = env::var("GRPC_SOCIAL_ENDPOINT")
            .unwrap_or_else(|_| "https://social-service:50054".to_string());
        let admin_endpoint = env::var("GRPC_ADMIN_ENDPOINT")
            .unwrap_or_else(|_| "https://admin-service:50055".to_string());
        let card_endpoint = env::var("GRPC_CARD_ENDPOINT")
            .unwrap_or_else(|_| "https://card-service:50061".to_string());

        let ca_cert = format!("{}/ca.pem", tls_dir);
        let client_cert = format!("{}/rgs-flash-mock-client.pem", tls_dir);
        let client_key = format!("{}/rgs-flash-mock-client.key", tls_dir);

        Ok(Self {
            bind_addr,
            log_level,
            tls_dir,
            service_name,
            ca_cert,
            client_cert,
            client_key,
            player_endpoint,
            economy_endpoint,
            match_endpoint,
            social_endpoint,
            admin_endpoint,
            card_endpoint,
        })
    }

    /// 检查 mTLS cert 是否存在 (启动时验证, per 8/27 凭据走 env var)
    pub fn verify_certs(&self) -> Result<()> {
        for path in [&self.ca_cert, &self.client_cert, &self.client_key] {
            if !std::path::Path::new(path).exists() {
                anyhow::bail!(
                    "mTLS cert 缺失: {} (per 8/27 11:06 JST 凭据走 env var + 5 域 ST 业务 mTLS, set RGS_ALLOW_INSECURE_GRPC=1 仅 dev/test 兜底)",
                    path
                );
            }
        }
        Ok(())
    }

    /// 6 域 gRPC endpoint 列表 (跟 gap_matrix 12 大类 1:1 对应)
    pub fn endpoints(&self) -> Vec<(&str, &str)> {
        vec![
            ("player", &self.player_endpoint),
            ("economy", &self.economy_endpoint),
            ("match", &self.match_endpoint),
            ("social", &self.social_endpoint),
            ("admin", &self.admin_endpoint),
            ("card", &self.card_endpoint),
        ]
    }
}

/// 凭据 REDACTED filter (per 8/27 11:06 JST hard ban)
pub fn redact_endpoint(endpoint: &str) -> String {
    // 简单的 password redaction, 避免日志打印完整 endpoint 含凭据
    if endpoint.contains('@') {
        let parts: Vec<&str> = endpoint.splitn(2, '@').collect();
        let scheme_user = parts[0];
        if let Some(slash_idx) = scheme_user.rfind('/') {
            let scheme_part = &scheme_user[..slash_idx + 1];
            let user_part = &scheme_user[slash_idx + 1..];
            if let Some(colon_idx) = user_part.find(':') {
                return format!("{}{}:REDACTED@{}", scheme_part, &user_part[..colon_idx], parts[1]);
            }
        }
    }
    endpoint.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_endpoint() {
        let redacted = redact_endpoint("https://ulysses_local:secret@host:5432/db");
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("REDACTED"));
    }
}
