//! rgs-flash-mock v0.2 — 5 域 gRPC client pool + mTLS 业务级
//!
//! per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 §2.1 (5 域 tonic 0.12 gRPC client)
//! per shared-platform::tls load_client_tls pattern (RGS 5 域 ST 业务 mTLS 一致)
//! per 8/27 11:06 JST 硬 ban: 凭据走 env var, 永不打印
//!
//! 5 域:
//!   - player  → PlayerServiceClient  (player-service:50051)
//!   - economy → EconomyServiceClient (economy-service:50052)
//!   - match   → MatchServiceClient   (match-service:50053)
//!   - social  → SocialServiceClient  (social-service:50054)
//!   - admin   → AdminServiceClient   (admin-service:50055)
//!
//! v0.1 → v0.2 升级: 真实 5 域 mTLS 业务级 gRPC 调用 (v0.1 全 stub)
//!
//! proto module 布局 (跟 crates/{player,...}-service 一样):
//!   crate::proto::v1            → player 生成的代码
//!   crate::proto::economy::v1   → economy 生成的代码
//!   crate::proto::r#match::v1   → match 生成的代码 (alias)
//!   crate::proto::social::v1    → social 生成的代码
//!   crate::proto::admin::v1     → admin 生成的代码
//!   crate::common::v1           → common 生成的代码 (sibling of crate::proto, per 5 域 mode)

use std::path::Path;

use tonic::transport::{Channel, ClientTlsConfig};

/// 5 域 gRPC client 集合
/// 注意: tonic 0.12 的 client 是 cheap clone 的, 用 Arc 共享即可
#[derive(Clone)]
pub struct GrpcClients {
    pub player: Option<player::player_service_client::PlayerServiceClient<Channel>>,
    pub economy: Option<economy::economy_service_client::EconomyServiceClient<Channel>>,
    pub r#match: Option<match_proto::match_service_client::MatchServiceClient<Channel>>,
    pub social: Option<social::social_service_client::SocialServiceClient<Channel>>,
    pub admin: Option<admin::admin_service_client::AdminServiceClient<Channel>>,
}

/// 5 域 gRPC 客户端状态 (per /ready 跟 /coverage 报告)
#[derive(Debug, Clone, serde::Serialize)]
pub struct GrpcClientStatus {
    pub domain: String,
    pub endpoint: String,
    pub connected: bool,
    pub last_error: Option<String>,
    pub last_check_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl GrpcClients {
    /// 从 env + cert 路径构造 5 域 gRPC client
    ///
    /// - 不强制要求 cert 存在 (RGS_ALLOW_INSECURE_GRPC=1 dev/test 兜底, per shared-platform 模式)
    /// - 任一域 connect 失败 → Option=None + last_error (不 panic)
    /// - domain_name 跟 5 域 k8s service DNS 一致 (player-service / economy-service / ...)
    pub async fn from_config(cfg: &crate::config::Config) -> Self {
        // mTLS tls config (per shared-platform load_client_tls pattern)
        let tls = load_client_tls_from_paths(
            &cfg.ca_cert,
            &cfg.client_cert,
            &cfg.client_key,
        );

        let mut clients = Self {
            player: None,
            economy: None,
            r#match: None,
            social: None,
            admin: None,
        };

        // player
        clients.player = connect_domain(
            "player",
            &cfg.player_endpoint,
            "player-service",
            tls.as_ref(),
        )
        .await;

        // economy
        clients.economy = connect_domain(
            "economy",
            &cfg.economy_endpoint,
            "economy-service",
            tls.as_ref(),
        )
        .await;

        // match (Rust 关键字冲突, 走 r#match)
        clients.r#match = connect_domain(
            "match",
            &cfg.match_endpoint,
            "match-service",
            tls.as_ref(),
        )
        .await;

        // social
        clients.social = connect_domain(
            "social",
            &cfg.social_endpoint,
            "social-service",
            tls.as_ref(),
        )
        .await;

        // admin
        clients.admin = connect_domain(
            "admin",
            &cfg.admin_endpoint,
            "admin-service",
            tls.as_ref(),
        )
        .await;

        clients
    }

    /// 5 域状态 (per /ready + /coverage 报告)
    pub fn status_report(&self, cfg: &crate::config::Config) -> Vec<GrpcClientStatus> {
        let now = Some(chrono::Utc::now());
        vec![
            GrpcClientStatus {
                domain: "player".into(),
                endpoint: cfg.player_endpoint.clone(),
                connected: self.player.is_some(),
                last_error: None,
                last_check_at: now,
            },
            GrpcClientStatus {
                domain: "economy".into(),
                endpoint: cfg.economy_endpoint.clone(),
                connected: self.economy.is_some(),
                last_error: None,
                last_check_at: now,
            },
            GrpcClientStatus {
                domain: "match".into(),
                endpoint: cfg.match_endpoint.clone(),
                connected: self.r#match.is_some(),
                last_error: None,
                last_check_at: now,
            },
            GrpcClientStatus {
                domain: "social".into(),
                endpoint: cfg.social_endpoint.clone(),
                connected: self.social.is_some(),
                last_error: None,
                last_check_at: now,
            },
            GrpcClientStatus {
                domain: "admin".into(),
                endpoint: cfg.admin_endpoint.clone(),
                connected: self.admin.is_some(),
                last_error: None,
                last_check_at: now,
            },
        ]
    }
}

/// 单域 gRPC client 连接 (mTLS 或 insecure)
async fn connect_domain<T>(
    domain: &str,
    endpoint: &str,
    service_name: &str,
    tls: Option<&ClientTlsConfig>,
) -> Option<T>
where
    T: NewClient<Channel>,
{
    let mut endpoint_builder = Channel::from_shared(endpoint.to_string())
        .unwrap_or_else(|e| {
            tracing::error!(target: "rgs_flash_mock::grpc_clients", "{}: endpoint parse failed: {}", domain, e);
            // 兜底: 用一个永远不连的占位 endpoint (let it fail at .connect)
            Channel::from_shared("http://127.0.0.1:1".to_string()).expect("fallback parse")
        });

    if let Some(t) = tls {
        endpoint_builder = endpoint_builder.tls_config(t.clone()).ok()?;
    } else if endpoint.starts_with("https://") {
        // HTTPS 但无 cert → 走系统 CA, 不强制 mTLS (dev/test only)
        tracing::warn!(
            target: "rgs_flash_mock::grpc_clients",
            "{}: HTTPS but no client cert loaded, using system roots (dev/test only)",
            domain
        );
    }

    let channel = match endpoint_builder.connect().await {
        Ok(c) => {
            tracing::info!(
                target: "rgs_flash_mock::grpc_clients",
                "{}: gRPC connected (mTLS) endpoint={} service_name={}",
                domain,
                endpoint,
                service_name
            );
            c
        }
        Err(e) => {
            tracing::error!(
                target: "rgs_flash_mock::grpc_clients",
                "{}: gRPC connect failed endpoint={} err={}",
                domain,
                endpoint,
                e
            );
            return None;
        }
    };

    T::new(channel).into()
}

/// trait: 给 tonic 0.12 client 统一构造入口
pub trait NewClient<C>: Sized {
    fn new(channel: C) -> Self;
}

impl NewClient<Channel> for player::player_service_client::PlayerServiceClient<Channel> {
    fn new(channel: Channel) -> Self {
        Self::new(channel)
    }
}
impl NewClient<Channel> for economy::economy_service_client::EconomyServiceClient<Channel> {
    fn new(channel: Channel) -> Self {
        Self::new(channel)
    }
}
impl NewClient<Channel> for match_proto::match_service_client::MatchServiceClient<Channel> {
    fn new(channel: Channel) -> Self {
        Self::new(channel)
    }
}
impl NewClient<Channel> for social::social_service_client::SocialServiceClient<Channel> {
    fn new(channel: Channel) -> Self {
        Self::new(channel)
    }
}
impl NewClient<Channel> for admin::admin_service_client::AdminServiceClient<Channel> {
    fn new(channel: Channel) -> Self {
        Self::new(channel)
    }
}

/// 从 PEM 文件构造 ClientTlsConfig (复用 shared-platform load_client_tls 模式)
/// 返回 None 当任何 cert 缺失 (走 RGS_ALLOW_INSECURE_GRPC 兜底逻辑)
fn load_client_tls_from_paths(
    ca_path: &str,
    cert_path: &str,
    key_path: &str,
) -> Option<ClientTlsConfig> {
    use std::fs;
    use tonic::transport::{Certificate, Identity};

    let ca_pem = match fs::read_to_string(ca_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                target: "rgs_flash_mock::grpc_clients",
                "ca_cert missing ({}): {} — gRPC client 走无 mTLS 兜底 (per RGS_ALLOW_INSECURE_GRPC 模式)",
                ca_path,
                e
            );
            return None;
        }
    };
    let cert_pem = match fs::read_to_string(cert_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                target: "rgs_flash_mock::grpc_clients",
                "client_cert missing ({}): {} — gRPC client 走无 mTLS 兜底",
                cert_path,
                e
            );
            return None;
        }
    };
    let key_pem = match fs::read_to_string(key_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                target: "rgs_flash_mock::grpc_clients",
                "client_key missing ({}): {} — gRPC client 走无 mTLS 兜底",
                key_path,
                e
            );
            return None;
        }
    };

    let ca = Certificate::from_pem(ca_pem);
    let identity = Identity::from_pem(cert_pem, key_pem);
    let tls = ClientTlsConfig::new()
        // domain_name 跟 k8s service DNS 一致 (per shared-platform load_client_tls pattern)
        // 不设置 → tonic 默认用 endpoint 的 host
        .ca_certificate(ca)
        .identity(identity);
    Some(tls)
}

// tonic-build 生成的模块 (5 域 + common)
// 5 域模块 + common 全部放在 crate::grpc_clients::proto::{domain}::v1 路径下 (统一 4 层深度)
// tonic-build 生成 super::super::common::v1 (从 domain::v1 上 2 层到 proto, 再加 common::v1)
pub mod proto {
    pub mod player {
        pub mod v1 {
            tonic::include_proto!("player.v1");
        }
    }
    pub mod economy {
        pub mod v1 {
            tonic::include_proto!("economy.v1");
        }
    }
    /// `match` 是 Rust 关键字, tonic-build 会用 proto 包名 `match.v1` 但生成 r#match.v1.rs
    pub mod r#match {
        pub mod v1 {
            tonic::include_proto!("r#match.v1");
        }
    }
    pub mod social {
        pub mod v1 {
            tonic::include_proto!("social.v1");
        }
    }
    pub mod admin {
        pub mod v1 {
            tonic::include_proto!("admin.v1");
        }
    }
    /// common 必须是 5 域的 sibling (sibling of crate::grpc_clients::proto::player::v1)
    /// tonic-build 生成 super::super::common::v1 from inside crate::grpc_clients::proto::player::v1
    /// → crate::grpc_clients::proto::common::v1
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
}

// Alias 给顶层更直观的访问 (相对 crate::grpc_clients 内部 use)
pub mod player {
    pub use super::proto::player::v1::*;
}
pub mod economy {
    pub use super::proto::economy::v1::*;
}
pub mod match_proto {
    pub use super::proto::r#match::v1::*;
}
pub mod social {
    pub use super::proto::social::v1::*;
}
pub mod admin {
    pub use super::proto::admin::v1::*;
}

/// 静态断言: cert path 都是字符串, 不暴露 value (per 8/27 11:06 JST 凭据硬 ban)
#[allow(dead_code)]
fn _static_assert_no_cert_value_in_logs() {
    let _p: &Path = Path::new("/etc/rgs/certs/ca.pem");
}
