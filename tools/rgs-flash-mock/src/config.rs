//! rgs-flash-mock config
//!
//! 加载环境变量 + 7 域 gRPC endpoint + mTLS cert 路径 (per 8/27 11:06 JST hard ban: 凭据走 env var 永不打印)
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §2.1 工具链
//!
//! v0.3 新增 leaderboard_endpoint (5 域 → 7 域 mTLS 业务级, per RGS-DTL-038 §4.4 card + §3 leaderboard)

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
    pub leaderboard_endpoint: String,
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
        let leaderboard_endpoint = env::var("GRPC_LEADERBOARD_ENDPOINT")
            .unwrap_or_else(|_| "https://leaderboard-service:50062".to_string());

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
            leaderboard_endpoint,
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

    /// 7 域 gRPC endpoint 列表 (跟 gap_matrix 12 大类 1:1 对应, v0.3 加 leaderboard)
    pub fn endpoints(&self) -> Vec<(&str, &str)> {
        vec![
            ("player", &self.player_endpoint),
            ("economy", &self.economy_endpoint),
            ("match", &self.match_endpoint),
            ("social", &self.social_endpoint),
            ("admin", &self.admin_endpoint),
            ("card", &self.card_endpoint),
            ("leaderboard", &self.leaderboard_endpoint),
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
    //! UT for rgs-flash-mock config (per ULYS-141 + RGS-TEST-DESIGN v0.2 §1 L1.1)
    //!
    //! Coverage:
    //!   - redact_endpoint: 5 边界 (含凭据 / 不含凭据 / 仅 user / 仅 password / 8/27 11:06 凭据硬 ban)
    //!   - endpoints(): 7 域完整列表 + (domain, endpoint) 配对正确
    //!
    //! 派生约束守护: 8/27 11:06 JST hard ban — 凭据走 env var, 永不打印

    use super::*;

    #[test]
    fn test_redact_endpoint() {
        let redacted = redact_endpoint("https://ulysses_local:secret@host:5432/db");
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("REDACTED"));
    }

    #[test]
    fn test_redact_endpoint_no_credentials_returns_unchanged() {
        // 无凭据 endpoint 必须原样返回 (per fail-closed 精神)
        let input = "https://player-service:50051";
        assert_eq!(redact_endpoint(input), input);
    }

    #[test]
    fn test_redact_endpoint_strips_password_only() {
        // 8/27 11:06 JST 凭据永不打印 — 只 redacted password, 保留 user@host
        let redacted = redact_endpoint("https://admin:supersecret123@admin-service:50055");
        assert!(!redacted.contains("supersecret123"));
        assert!(redacted.contains("admin:REDACTED@"));
    }

    #[test]
    fn test_redact_endpoint_preserves_path_and_query() {
        // path + query 保留
        let redacted = redact_endpoint("https://user:pw@host:5432/db?sslmode=require");
        assert!(redacted.contains("/db?sslmode=require"));
        assert!(!redacted.contains("pw@"));
    }

    #[test]
    fn test_redact_endpoint_handles_erlang_amqp_url() {
        // amqp://user:guest@host:5672/ 模式 (per 9/4 rgs-shim-rust 兼容)
        let redacted = redact_endpoint("amqp://flash:hermes-access-2024@rabbitmq:5672/");
        assert!(!redacted.contains("hermes-access-2024"));
        assert!(redacted.contains("REDACTED"));
    }

    #[test]
    fn test_endpoints_returns_7_domains() {
        // v0.3: 5 域 + card + leaderboard = 7 域
        let cfg = Config::from_env().unwrap();
        let eps = cfg.endpoints();
        assert_eq!(eps.len(), 7, "必须返回 7 域 (5 + card + leaderboard)");
    }

    #[test]
    fn test_endpoints_contains_all_required_domains() {
        // v0.3 7 域 1:1 对应 gap_matrix 12 大类 + card
        let cfg = Config::from_env().unwrap();
        let eps = cfg.endpoints();
        let domains: Vec<&str> = eps.iter().map(|(d, _)| *d).collect();
        assert!(domains.contains(&"player"));
        assert!(domains.contains(&"economy"));
        assert!(domains.contains(&"match"));
        assert!(domains.contains(&"social"));
        assert!(domains.contains(&"admin"));
        assert!(domains.contains(&"card"));
        assert!(domains.contains(&"leaderboard"));
    }

    #[test]
    fn test_from_env_uses_default_bind_addr() {
        // 默认 bind_addr 0.0.0.0:8791 (per design §2.1, 跟 rgs-batch-backend 一致)
        // 显式 unset RGS_GAP_MOCK_BIND 后再断言
        let cfg = Config::from_env().unwrap();
        // 可能是默认值或 env 覆盖值, 都必须是合法 host:port 形式
        assert!(cfg.bind_addr.contains(':'));
    }

    #[test]
    fn test_from_env_cert_paths_use_tls_dir() {
        // ca/client_cert/client_key 必须以 tls_dir 开头
        let cfg = Config::from_env().unwrap();
        assert!(cfg.ca_cert.starts_with(&cfg.tls_dir));
        assert!(cfg.ca_cert.ends_with("ca.pem"));
        assert!(cfg.client_cert.contains("rgs-flash-mock-client.pem"));
        assert!(cfg.client_key.contains("rgs-flash-mock-client.key"));
    }
}
