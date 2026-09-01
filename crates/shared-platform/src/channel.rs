//! gRPC Channel 工厂（per RGS-SPEC-CROSS-002 mTLS + 跨域 RPC 规范）
//!
//! 54.9 实化：RpcChannel 工厂 + mTLS + 超时 + retry 拦截器
//! 55.18 实化：mTLS fail-closed（`require_tls: true` 默认） + bypass 计数
//!
//! 设计：
//! - tonic::transport::Channel 是多路复用（HTTP/2），1 个 Channel 可并发处理 N 个 RPC
//! - mTLS 用 rustls 加载（per 53.11 rgs-certgen）
//! - timeout 用 tonic Request<...>::set_timeout（per-call）
//! - retry 通过包装 invoke（per RGS-SPEC-CROSS-006 草案）
//! - 默认 fail-closed：未配置 TLS 时 `require_tls=true` 返 `TlsRequired` 错误
//!   （per RGS-REV-007 CH4 + DEC-015 P1 审计建议）
//! - 显式 `require_tls=false` 时打 warn 日志 + `mtls_bypassed_total++` 计数

use std::sync::atomic::{AtomicU64, Ordering};
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

    /// fail-closed 触发：cfg.tls 为 None 且 require_tls=true（per RGS-REV-007 CH4）
    #[error(
        "TLS required but cfg.tls is None (fail-closed); set require_tls=false to explicitly opt-out"
    )]
    TlsRequired,
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
    /// TLS 配置（None = 明文；需配合 `require_tls=false` 才允许）
    pub tls: Option<ClientTlsConfigInput>,
    /// 重试配置
    pub retry: RetryConfig,
    /// TLS 强制开关（per RGS-REV-007 CH4 fail-closed）
    ///
    /// - `true`（默认）：`cfg.tls = None` 时返 `TlsRequired` 错误
    /// - `false`：允许明文连接，但打 warn 日志并 +1 `mTLS_bypassed_total`
    pub require_tls: bool,
}

impl Default for RpcChannelConfig {
    fn default() -> Self {
        Self {
            uri: String::new(),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            tls: None,
            retry: RetryConfig::default(),
            // fail-closed by default（per RGS-REV-007 CH4 + DEC-015 P1）
            require_tls: true,
        }
    }
}

/// mTLS bypass 计数（per RGS-REV-007 CH4 监控项）
///
/// 每次 `build_channel` 因 `require_tls=false` 走明文路径时 +1。
/// 调用方应通过 `mtls_bypassed_total()` 读取当前值（Prometheus exporter
/// 或 scrape handler 暴露为 `mTLS_bypassed_total`）。
static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// 获取 mTLS bypass 累计计数（client 端）
pub fn mtls_bypassed_total() -> u64 {
    MTLS_BYPASSED_TOTAL.load(Ordering::Relaxed)
}

/// 服务端 mTLS bypass 计数（per RGS-REV-009 HI-1）
///
/// 与 client 端 `MTLS_BYPASSED_TOTAL` 对称，但独立 per-process：
/// 6 域 main.rs 启动时若 `RGS_ALLOW_INSECURE_GRPC=1` 则 `fetch_add(1)`。
/// 调用方应通过 `server_mtls_bypassed_total()` 读取（Prometheus exporter
/// 或 scrape handler 暴露为 `server_mTLS_bypassed_total`）。
pub static SERVER_MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// 获取服务端 mTLS bypass 累计计数
pub fn server_mtls_bypassed_total() -> u64 {
    SERVER_MTLS_BYPASSED_TOTAL.load(Ordering::Relaxed)
}

/// 构造 RpcChannel（带 mTLS + 超时 + fail-closed）
///
/// 行为矩阵（per RGS-REV-007 CH4 + DEC-015 P1）：
///
/// | `cfg.tls` | `require_tls` | 行为 |
/// |-----------|---------------|------|
/// | `Some(_)` | 任意          | mTLS 连接 |
/// | `None`    | `true`（默认）| 返 `TlsRequired` 错误（fail-closed） |
/// | `None`    | `false`       | 明文 + `tracing::warn!` + `mTLS_bypassed_total++` |
pub async fn build_channel(cfg: &RpcChannelConfig) -> Result<Channel, ChannelError> {
    let mut endpoint = Endpoint::from_shared(cfg.uri.clone())
        .map_err(|e| ChannelError::InvalidUri(e.to_string()))?
        .connect_timeout(cfg.connect_timeout)
        .timeout(cfg.request_timeout);

    match (&cfg.tls, cfg.require_tls) {
        (Some(tls_input), _) => {
            let tls_config = load_client_tls(tls_input)?;
            endpoint = endpoint
                .tls_config(tls_config)
                .map_err(|e| ChannelError::Connect(format!("tls config: {}", e)))?;
        }
        (None, true) => {
            // fail-closed：未配置 TLS 且未显式 opt-out → 直接拒绝
            return Err(ChannelError::TlsRequired);
        }
        (None, false) => {
            // 显式 opt-out：明文 + 告警 + 计数
            tracing::warn!(
                uri = %cfg.uri,
                "building plaintext channel (mTLS explicitly disabled via require_tls=false)"
            );
            MTLS_BYPASSED_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
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
    async fn build_channel_no_tls_returns_tls_required_error() {
        // 默认 require_tls=true + tls=None → TlsRequired（fail-closed）
        let cfg = RpcChannelConfig {
            uri: "https://127.0.0.1:50051".to_string(),
            ..Default::default()
        };
        assert!(cfg.require_tls, "default should be fail-closed");
        let result = build_channel(&cfg).await;
        assert!(
            matches!(result, Err(ChannelError::TlsRequired)),
            "expected TlsRequired, got {:?}",
            result
        );
    }

    #[tokio::test]
    async fn build_channel_with_require_tls_false_increments_bypass_counter() {
        // 显式 opt-out：counter +1（即使连接最终失败）
        let before = mtls_bypassed_total();
        let cfg = RpcChannelConfig {
            uri: "http://127.0.0.1:50051".to_string(),
            connect_timeout: Duration::from_millis(50),
            require_tls: false,
            ..Default::default()
        };
        let _ = build_channel(&cfg).await; // 连接预期失败（无服务监听）
        let after = mtls_bypassed_total();
        assert!(
            after > before,
            "expected mtls_bypassed_total to increment, before={} after={}",
            before,
            after
        );
    }

    #[tokio::test]
    async fn build_channel_with_tls_returns_tls_error_not_tls_required() {
        // 配 TLS 时不走 fail-closed 路径，错误是 Tls（证书文件不存在）而非 TlsRequired
        let cfg = RpcChannelConfig {
            uri: "https://127.0.0.1:50051".to_string(),
            connect_timeout: Duration::from_millis(50),
            require_tls: true,
            tls: Some(crate::tls::ClientTlsConfigInput {
                domain: "player.local".to_string(),
                ca_cert_path: "/nonexistent/ca.pem".to_string(),
                client_cert_path: "/nonexistent/client.pem".to_string(),
                client_key_path: "/nonexistent/client.key".to_string(),
            }),
            ..Default::default()
        };
        let result = build_channel(&cfg).await;
        assert!(
            matches!(result, Err(ChannelError::Tls(_))),
            "expected Tls error (placeholder cert paths missing), got {:?}",
            result
        );
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

    // ---- 9/1 pt/shared-platform worker 派工 (per PT-WORKER-BRIEFING.md §2) ----
    // channel + retry 协同单测, 加 5 个

    #[tokio::test]
    async fn retry_backoff_increases_with_attempt() {
        // 同一可重试 status, attempt 越大退避越长 (在未超 max 之前)
        let cfg = RetryConfig::default();
        let status = tonic::Status::new(Code::Unavailable, "x");
        let d0 = retry_backoff(&status, 0, &cfg).expect("retryable");
        let d1 = retry_backoff(&status, 1, &cfg).expect("retryable");
        // 1 的 base 是 0 的 2x, 1 必须 > 0 (含 jitter 边界, 至少 1.6x)
        assert!(
            d1 >= d0,
            "attempt 1 ({:?}) 必须 ≥ attempt 0 ({:?})",
            d1,
            d0
        );
    }

    #[tokio::test]
    async fn retry_backoff_for_all_retryable_codes() {
        // retry_backoff 应对 4 类可重试 code 都返 Some
        let cfg = RetryConfig::default();
        for code in [
            Code::Unavailable,
            Code::DeadlineExceeded,
            Code::Aborted,
            Code::ResourceExhausted,
        ] {
            let status = tonic::Status::new(code, "x");
            let result = retry_backoff(&status, 0, &cfg);
            assert!(result.is_some(), "{:?} 应该可重试", code);
        }
    }

    #[tokio::test]
    async fn retry_backoff_for_all_non_retryable_codes() {
        // retry_backoff 应对 4 类不可重试 code 都返 None
        let cfg = RetryConfig::default();
        for code in [
            Code::NotFound,
            Code::InvalidArgument,
            Code::PermissionDenied,
            Code::Unauthenticated,
        ] {
            let status = tonic::Status::new(code, "x");
            let result = retry_backoff(&status, 0, &cfg);
            assert!(result.is_none(), "{:?} 不应该可重试", code);
        }
    }

    #[tokio::test]
    async fn retry_backoff_at_max_retries_boundary() {
        // attempt = max_retries - 1 应该仍可重试
        // attempt = max_retries 应该不可重试
        let cfg = RetryConfig {
            max_retries: 2,
            initial_interval: Duration::from_millis(1),
            max_interval: Duration::from_millis(10),
            multiplier: 2.0,
        };
        let status = tonic::Status::new(Code::Unavailable, "x");
        // attempt=1 = max_retries-1 → 仍可重试
        assert!(retry_backoff(&status, 1, &cfg).is_some());
        // attempt=2 = max_retries → 不可重试 (超限)
        assert!(retry_backoff(&status, 2, &cfg).is_none());
    }

    #[tokio::test]
    async fn mtls_bypass_counter_is_atomic_and_monotonic() {
        // 多次连续 opt-out 计数应该单调递增
        let before = mtls_bypassed_total();
        for _ in 0..5 {
            let cfg = RpcChannelConfig {
                uri: "http://127.0.0.1:1".to_string(),
                connect_timeout: Duration::from_millis(1),
                require_tls: false,
                ..Default::default()
            };
            let _ = build_channel(&cfg).await; // 连接预期失败, 但 counter 已 +1
        }
        let after = mtls_bypassed_total();
        assert!(
            after >= before + 5,
            "5 次 opt-out 应当 +5, before={} after={}",
            before,
            after
        );
    }
}
