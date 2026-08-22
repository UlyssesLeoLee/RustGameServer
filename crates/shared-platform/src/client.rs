//! 6 域 gRPC client 工厂（per RGS-SPEC-CROSS-002 跨域 RPC）
//!
//! 54.9 实化：6 域 client builder（player/economy/match/social/admin/cluster-ops）
//! + 统一 ApiVersion + ServiceId 解析
//!
//! 55.18 实化：拆分 `build_secure_channel`（默认 mTLS）+ `build_insecure_channel`
//! （显式 opt-out，warn + `mTLS_bypassed_total++`），消除原 `use_tls: bool`
//! 误传风险（per RGS-REV-007 CH4 L4 + DEC-015 P1）
//!
//! 设计：
//! - 各域 client 由各域 crate 暴露（pub use proto::v1::*_client::*Client）
//! - shared-platform 提供 client builder 统一 mTLS / timeout / retry 配置
//! - 调用方：`build_secure_channel` / `build_insecure_channel` → 用 Channel 实例化各域 client
//!
//! 注意：此模块只声明"客户端使用规范"，不实际生成 client（client 在各域 crate）。

use crate::channel::{build_channel, RpcChannelConfig};
use crate::tls::ClientTlsConfigInput;
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

    /// 构造默认 URI（mTLS 用 https://）
    pub fn default_uri(self, host: &str) -> String {
        format!("https://{}:{}", host, self.default_port())
    }
}

/// 构造 6 域 client 共用的 mTLS Channel（per RGS-SPEC-CROSS-002 跨域 RPC 规范）
///
/// 默认路径：使用 `ServiceId` 默认域名 + 占位证书路径
/// （`/etc/rgs/certs/{domain}-ca.pem` / `client.pem` / `client.key`）。
/// 实际生产应由调用方在 5 域 `main.rs` 内通过 [`build_secure_channel_with_tls`]
/// 注入真实证书路径（per RGS-REV-007 CH4 + DEC-015 P1）。
pub async fn build_secure_channel(
    service: ServiceId,
    host: &str,
) -> Result<Channel, ChannelError> {
    let tls = default_client_tls_input(service);
    build_secure_channel_with_tls(service, host, &tls).await
}

/// 构造 6 域 client 共用的 mTLS Channel（显式证书路径）
///
/// 与 [`build_secure_channel`] 的区别：本函数接受完整 [`ClientTlsConfigInput`]，
/// 5 域 `main.rs` 应使用本函数注入 `/etc/rgs/certs/` 实际路径。
pub async fn build_secure_channel_with_tls(
    service: ServiceId,
    host: &str,
    tls: &ClientTlsConfigInput,
) -> Result<Channel, ChannelError> {
    let cfg = RpcChannelConfig {
        uri: service.default_uri(host),
        tls: Some(tls.clone()),
        require_tls: true, // 强制 mTLS（per RGS-REV-007 CH4）
        ..Default::default()
    };
    build_channel(&cfg).await
}

/// 构造 6 域 client 共用的**明文** Channel（仅 dev/test 使用）
///
/// ⚠️  警告：本函数**显式绕过 mTLS**，会触发：
/// - `tracing::warn!` 日志（含 service / host）
/// - `mTLS_bypassed_total++` 计数器（per RGS-REV-007 CH4 监控）
///
/// 生产代码应使用 [`build_secure_channel`] / [`build_secure_channel_with_tls`]。
/// 本函数仅用于本地集成测试 / dev 环境（per RGS-REV-007 CH4 显式 opt-out 原则）。
pub async fn build_insecure_channel(
    service: ServiceId,
    host: &str,
) -> Result<Channel, ChannelError> {
    tracing::warn!(
        service = ?service,
        host = %host,
        port = service.default_port(),
        "build_insecure_channel invoked: mTLS bypassed (mTLS_bypassed_total++)"
    );
    let cfg = RpcChannelConfig {
        uri: service.default_uri(host),
        tls: None,
        require_tls: false, // 显式 opt-out
        connect_timeout: std::time::Duration::from_secs(2), // 短超时便于测试
        ..Default::default()
    };
    build_channel(&cfg).await
}

/// 默认 client TLS 输入（占位证书路径，待 5 域 main.rs 替换）
fn default_client_tls_input(service: ServiceId) -> ClientTlsConfigInput {
    ClientTlsConfigInput {
        domain: service.default_domain().to_string(),
        ca_cert_path: format!("/etc/rgs/certs/{}-ca.pem", service.default_domain()),
        client_cert_path: "/etc/rgs/certs/client.pem".to_string(),
        client_key_path: "/etc/rgs/certs/client.key".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::mtls_bypassed_total;

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

    #[test]
    fn default_client_tls_input_uses_service_domain() {
        let input = default_client_tls_input(ServiceId::Economy);
        assert_eq!(input.domain, "economy.local");
        assert!(input.ca_cert_path.contains("economy.local"));
        assert!(input.client_cert_path.contains("client.pem"));
    }

    #[tokio::test]
    async fn build_secure_channel_uses_tls() {
        // 调用 build_secure_channel 时，错误类型不应该是 TlsRequired
        // （因为 tls 已配置）；可能是 Tls（证书文件不存在）或 Connect（网络失败）
        let result = build_secure_channel(ServiceId::Player, "127.0.0.1").await;
        assert!(
            !matches!(result, Err(ChannelError::TlsRequired)),
            "build_secure_channel must not return TlsRequired (TLS is configured)"
        );
        // 预期：占位证书路径不存在 → TlsError::FileRead
        assert!(
            matches!(result, Err(ChannelError::Tls(_))),
            "expected Tls error (placeholder cert paths missing), got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn build_insecure_channel_emits_warning() {
        // build_insecure_channel 必须增加 mtls_bypassed_total 计数
        let before = mtls_bypassed_total();
        // 即使连接失败，counter 也会先 +1（在 build_channel 内）
        let _ = build_insecure_channel(ServiceId::Player, "127.0.0.1").await;
        let after = mtls_bypassed_total();
        assert!(
            after > before,
            "expected mtls_bypassed_total to increment, before={} after={}",
            before,
            after
        );
    }

    #[tokio::test]
    async fn build_secure_channel_with_explicit_tls_uses_tls() {
        // 显式证书路径：与默认行为相同 — 错误类型不是 TlsRequired
        let tls = ClientTlsConfigInput {
            domain: "player.local".to_string(),
            ca_cert_path: "/nonexistent/ca.pem".to_string(),
            client_cert_path: "/nonexistent/client.pem".to_string(),
            client_key_path: "/nonexistent/client.key".to_string(),
        };
        let result = build_secure_channel_with_tls(ServiceId::Player, "127.0.0.1", &tls).await;
        assert!(
            !matches!(result, Err(ChannelError::TlsRequired)),
            "build_secure_channel_with_tls must not return TlsRequired"
        );
    }
}
