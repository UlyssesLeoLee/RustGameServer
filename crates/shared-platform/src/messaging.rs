//! NATS 连接 + JetStream context 工厂（per RGS-DTL-100 §5 + RGS-SPEC-CROSS-005）
//!
//! 54.10 实化：NATS client + JetStream Context 统一构造
//!
//! 设计：
//! - async_nats::Client 持有连接
//! - JetStream Context 从 Client 派生
//! - 工厂模式：build_messaging_client(uri) → (Client, Producer, Consumer Handler 注册表)

use async_nats::jetstream;
use thiserror::Error;

/// Messaging 错误
#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("NATS connect error: {0}")]
    Connect(String),

    #[error("invalid URI: {0}")]
    InvalidUri(String),
}

/// Messaging 配置
#[derive(Debug, Clone)]
pub struct MessagingConfig {
    /// NATS server URI（如 nats://localhost:4222）
    pub uri: String,
    /// 客户端名称（用于 NATS server 端识别）
    pub name: String,
}

impl Default for MessagingConfig {
    fn default() -> Self {
        Self {
            uri: "nats://localhost:4222".to_string(),
            name: "rgs-shared-platform".to_string(),
        }
    }
}

/// 构造 NATS client + JetStream context
pub async fn build_messaging_client(
    config: &MessagingConfig,
) -> Result<(async_nats::Client, jetstream::Context), MessagingError> {
    let client = async_nats::ConnectOptions::new()
        .name(config.name.clone())
        .connect(&config.uri)
        .await
        .map_err(|e| MessagingError::Connect(e.to_string()))?;
    let js_ctx = jetstream::new(client.clone());
    Ok((client, js_ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messaging_config_default() {
        let cfg = MessagingConfig::default();
        assert_eq!(cfg.uri, "nats://localhost:4222");
        assert!(cfg.name.starts_with("rgs-"));
    }
}
