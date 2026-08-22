//! gRPC Channel 工厂（per RGS-SPEC-CROSS-002 mTLS + 跨域 RPC 规范）
//!
//! 54.9 实化：RpcChannel 工厂 + mTLS + 超时 + retry 拦截器
//!
//! 设计：
//! - tonic::transport::Channel 是多路复用（HTTP/2），1 个 Channel 可并发处理 N 个 RPC
//! - mTLS 用 rustls 加载（per 53.11 rgs-certgen）
//! - timeout 用 tonic Request<...>::set_timeout（per-call）
//! - retry 通过包装 invoke（per RGS-SPEC-CROSS-006 草案）

use std::time::Duration;

use thiserror::Error;
use tonic::transport::{Channel, Endpoint};

use crate::retry::{is_retryable, RetryConfig};
use crate::tls::{load_client_tls, ClientTlsConfigInput};

/// Channel 错误
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("invalid endpoint URI: {0}")]
    InvalidUri(String),

    #[error("TLS error: {0}")]
    Tls(#[from] crate::tls::TlsError),

    #[error("connection error: {0}")]
    Connect(String),
}

/// RpcChannel 配置
#[derive(Debug, Clone)]
pub struct RpcChannelConfig {
    /// 目标 URI（http://host:port 或 https://host:port）
    pub uri: String,
    /// 连接超时
    pub connect_timeout: Duration,
    /// 单次 RPC 超时
    pub request_timeout: Duration,
    /// TLS 配置（None = 明文）
    pub tls: Option<ClientTlsConfigInput>,
    /// 重试配置
    pub retry: RetryConfig,
}

impl Default for RpcChannelConfig {
    fn default() -> Self {
        Self {
            uri: String::new(),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            tls: None,
            retry: RetryConfig::default(),
        }
    }
}

/// 构造 RpcChannel（带 mTLS + 超时）
pub async fn build_channel(cfg: &RpcChannelConfig) -> Result<Channel, ChannelError> {
    let mut endpoint = Endpoint::from_shared(cfg.uri.clone())
        .map_err(|e| ChannelError::InvalidUri(e.to_string()))?
        .connect_timeout(cfg.connect_timeout)
        .timeout(cfg.request_timeout);

    if let Some(tls_input) = &cfg.tls {
        let tls_config = load_client_tls(tls_input)?;
        endpoint = endpoint
            .tls_config(tls_config)
            .map_err(|e| ChannelError::Connect(format!("tls config: {}", e)))?;
    }

    let channel = endpoint
        .connect()
        .await
        .map_err(|e| ChannelError::Connect(e.to_string()))?;
    Ok(channel)
}

/// 判定并返回重试退避时长（per RGS-SPEC-CROSS-006）
pub fn retry_backoff(status: &tonic::Status, attempt: u32, cfg: &RetryConfig) -> Option<Duration> {
    if attempt >= cfg.max_retries {
        return None;
    }
    if !is_retryable(status.code()) {
        return None;
    }
    Some(crate::retry::backoff_duration(attempt, cfg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[tokio::test]
    async fn build_channel_invalid_uri() {
        let cfg = RpcChannelConfig {
            uri: "not a uri".to_string(),
            ..Default::default()
        };
        let result = build_channel(&cfg).await;
        assert!(matches!(result, Err(ChannelError::InvalidUri(_))));
    }

    #[tokio::test]
    async fn retry_backoff_returns_none_for_non_retryable() {
        let cfg = RetryConfig::default();
        let status = tonic::Status::new(Code::NotFound, "x");
        let result = retry_backoff(&status, 0, &cfg);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn retry_backoff_returns_none_when_exhausted() {
        let cfg = RetryConfig::default();
        let status = tonic::Status::new(Code::Unavailable, "x");
        let result = retry_backoff(&status, 100, &cfg);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn retry_backoff_returns_some_for_retryable() {
        let cfg = RetryConfig::default();
        let status = tonic::Status::new(Code::Unavailable, "x");
        let result = retry_backoff(&status, 0, &cfg);
        assert!(result.is_some());
    }
}
