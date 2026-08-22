//! 6 域 gRPC client 工厂（per RGS-SPEC-CROSS-002 跨域 RPC）
//!
//! 54.9 实化：6 域 client builder（player/economy/match/social/admin/cluster-ops）
//! + 统一 ApiVersion + ServiceId 解析
//!
//! 设计：
//! - 各域 client 由各域 crate 暴露（pub use proto::v1::*_client::*Client）
//! - shared-platform 提供 client builder 统一 mTLS / timeout / retry 配置
//! - 调用方：build_channel → 用 Channel 实例化各域 client
//!
//! 注意：此模块只声明"客户端使用规范"，不实际生成 client（client 在各域 crate）。

use crate::channel::{build_channel, RpcChannelConfig};
use crate::ChannelError;
use tonic::transport::Channel;

/// 服务 ID（5 域 + cluster-ops）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceId {
    Player,
    Economy,
    Match,
    Social,
    Admin,
    ClusterOps,
}

impl ServiceId {
    /// 默认 gRPC 端口（与 54.7 main.rs 一致）
    pub fn default_port(self) -> u16 {
        match self {
            ServiceId::Player => 50051,
            ServiceId::Economy => 50052,
            ServiceId::Match => 50053,
            ServiceId::Social => 50054,
            ServiceId::Admin => 50055,
            ServiceId::ClusterOps => 50056,
        }
    }

    /// 默认服务域名（per mTLS 证书 CN/SAN）
    pub fn default_domain(self) -> &'static str {
        match self {
            ServiceId::Player => "player.local",
            ServiceId::Economy => "economy.local",
            ServiceId::Match => "match.local",
            ServiceId::Social => "social.local",
            ServiceId::Admin => "admin.local",
            ServiceId::ClusterOps => "cluster-ops.local",
        }
    }

    /// 构造默认 URI
    pub fn default_uri(self, host: &str) -> String {
        format!("https://{}:{}", host, self.default_port())
    }
}

/// 构造 6 域 client 共用的 Channel（per RGS-SPEC-CROSS-002 跨域 RPC 规范）
pub async fn build_service_channel(
    service: ServiceId,
    host: &str,
    use_tls: bool,
) -> Result<Channel, ChannelError> {
    let uri = service.default_uri(host);
    let mut cfg = RpcChannelConfig {
        uri,
        ..Default::default()
    };
    if use_tls {
        // 实际生产：cfg.tls = Some(ClientTlsConfigInput { ... })
        // 54.9 范围只演示结构；mTLS 证书路径待 55.x 配置
        cfg.tls = Some(crate::tls::ClientTlsConfigInput {
            domain: service.default_domain().to_string(),
            ca_cert_path: format!("/etc/rgs/certs/{}-ca.pem", service.default_domain()),
            client_cert_path: "/etc/rgs/certs/client.pem".to_string(),
            client_key_path: "/etc/rgs/certs/client.key".to_string(),
        });
    }
    build_channel(&cfg).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_default_ports() {
        assert_eq!(ServiceId::Player.default_port(), 50051);
        assert_eq!(ServiceId::Economy.default_port(), 50052);
        assert_eq!(ServiceId::ClusterOps.default_port(), 50056);
    }

    #[test]
    fn service_default_uri() {
        assert_eq!(
            ServiceId::Player.default_uri("10.0.0.1"),
            "https://10.0.0.1:50051"
        );
    }

    #[test]
    fn service_default_domains() {
        assert_eq!(ServiceId::Player.default_domain(), "player.local");
        assert_eq!(ServiceId::ClusterOps.default_domain(), "cluster-ops.local");
    }
}
