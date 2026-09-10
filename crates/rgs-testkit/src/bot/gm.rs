//! GM 命令注入 (per DDD Review v0.2 §5.1 M4)
//!
//! 模拟 erlang C2 (协议 10399 GM 注入), 走 RGS `admin-service::issue_gm_command`
//! mTLS 通道. **本模块为 PoC stub** — 真实 admin mTLS 接入留给后续 worker
//! (per DDD Review v0.2 §5.3 Step 2 wave 2).
//!
//! # 强约束 (per 8/27 11:06 JST hard ban + AGENTS.md §1.2)
//!
//! - 凭据 (mTLS cert path) 走 `Option<String>` 配置, **不读 env, 不打印值**
//! - 真实 mTLS 连接留待 wave 2 worker 接入, 本 stub 立即返回 `Ok(GmResponse { ok: true, .. })`
//!
//! # 后续接入路径 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`)
//!
//! ```text
//! GmClient.issue(cmd)
//!   → tonic Channel::connect_tls(endpoint, MtlsConfig)
//!   → AdminServiceClient::issue_gm_command(IssueGmCommandRequest { command: cmd })
//!   → 失败 → exponential backoff (100/200/400ms) × 3, DLQ
//! ```

use serde::{Deserialize, Serialize};

/// mTLS 配置占位 (per 5 域 ST 业务级 mTLS 实践)
///
/// **凭据**走 `Option<String>`, 调用方传入, **不读 env, 不打印值**
/// (per 8/27 11:06 JST hard ban + L3 跨工具链决策守门).
#[derive(Clone, Debug, Default)]
pub struct MtlsConfig {
    /// admin-service endpoint, e.g. `"https://admin-service:8443"`
    pub endpoint: Option<String>,
    /// client cert path (PEM), 由调用方注入, 不读 env
    pub client_cert_path: Option<String>,
    /// client key path (PEM), 由调用方注入, 不读 env
    pub client_key_path: Option<String>,
    /// CA cert path (PEM), 由调用方注入, 不读 env
    pub ca_cert_path: Option<String>,
    /// server name (SNI), e.g. `"admin-service"`
    pub server_name: Option<String>,
}

/// GM 响应 (PoC stub)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GmResponse {
    /// 是否成功 (PoC 永远 true)
    pub ok: bool,
    /// 服务端消息 (PoC 占位)
    pub message: String,
}

/// GM 客户端 (PoC stub)
///
/// 真实 admin mTLS 接入留待 wave 2 worker. 当前实现:
/// - `endpoint` / `mtls` 字段保留, 不实际连接
/// - `issue(cmd)` 立即返回 `Ok(GmResponse { ok: true, message: "stub" })`
#[derive(Clone, Debug, Default)]
pub struct GmClient {
    /// admin-service endpoint
    pub admin_endpoint: Option<String>,
    /// mTLS 配置 (凭据走 Option, 不读 env)
    pub mtls_config: MtlsConfig,
}

impl GmClient {
    /// 构造 GM 客户端 (PoC: 字段全 Option, 真实连接后续 worker 接)
    pub fn new(admin_endpoint: impl Into<String>) -> Self {
        Self {
            admin_endpoint: Some(admin_endpoint.into()),
            mtls_config: MtlsConfig::default(),
        }
    }

    /// 构造 GM 客户端 + mTLS config (凭据不读 env, 由调用方注入)
    pub fn with_mtls(admin_endpoint: impl Into<String>, mtls: MtlsConfig) -> Self {
        Self {
            admin_endpoint: Some(admin_endpoint.into()),
            mtls_config: mtls,
        }
    }

    /// 注入 GM 命令 (PoC stub)
    ///
    /// 真实实现 (wave 2): tonic Channel::connect_tls + AdminServiceClient::issue_gm_command
    /// + exponential backoff (100/200/400ms) × 3, DLQ.
    ///
    /// # 错误
    /// - 当前 PoC 不返回 Err, 留给真实实现
    pub async fn issue(&self, cmd: &str) -> anyhow::Result<GmResponse> {
        // PoC: 不打印 cmd / endpoint / 任何凭据 (per 8/27 11:06 JST hard ban)
        tracing::debug!(cmd_len = cmd.len(), "GmClient::issue stub");
        Ok(GmResponse {
            ok: true,
            message: "stub: wave 2 will wire admin mTLS".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gm_stub_returns_ok() {
        let c = GmClient::new("https://admin-service:8443");
        let r = c.issue("加经验 100").await.expect("stub ok");
        assert!(r.ok);
        assert!(r.message.contains("stub"));
    }

    #[tokio::test]
    async fn gm_stub_with_mtls_config_does_not_panic() {
        let mtls = MtlsConfig {
            endpoint: Some("https://admin-service:8443".to_string()),
            ..Default::default()
        };
        let c = GmClient::with_mtls("https://admin-service:8443", mtls);
        let r = c.issue("设等级 50").await.expect("stub ok");
        assert!(r.ok);
    }
}
