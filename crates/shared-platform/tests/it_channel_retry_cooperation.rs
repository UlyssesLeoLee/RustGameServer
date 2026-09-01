//! shared-platform IT: channel + retry + tls 协同（per PT-WORKER-BRIEFING.md §2）
//!
//! 覆盖 3 个跨模块场景：
//! 1. it_channel_retryable_status_uses_backoff
//! 2. it_channel_non_retryable_status_skips_backoff
//! 3. it_channel_tls_missing_ca_returns_proper_error_chain

use shared_platform::channel::{mtls_bypassed_total, retry_backoff, RpcChannelConfig};
use shared_platform::retry::{is_retryable, RetryConfig};
use shared_platform::tls::TlsError;
use std::time::Duration;
use tonic::Code;

/// 场景 1: 可重试 status 应触发 backoff 计算（不实际连网）
#[tokio::test]
async fn it_channel_retryable_status_uses_backoff() {
    let cfg = RetryConfig::default();
    // 4 类可重试 code 都应产出 Some(backoff duration)
    for code in [
        Code::Unavailable,
        Code::DeadlineExceeded,
        Code::Aborted,
        Code::ResourceExhausted,
    ] {
        let status = tonic::Status::new(code, "transient");
        // is_retryable + retry_backoff 双保险
        assert!(is_retryable(code), "{:?} 应可重试", code);
        let d = retry_backoff(&status, 0, &cfg).expect("应返 Some");
        // 首次退避应 > 0
        assert!(d > Duration::from_millis(0), "{:?} 退避应 > 0", code);
    }
}

/// 场景 2: 不可重试 status 跳过 backoff (per RGS-SPEC-CROSS-006 业务错误不重试)
#[tokio::test]
async fn it_channel_non_retryable_status_skips_backoff() {
    let cfg = RetryConfig::default();
    for code in [
        Code::NotFound,
        Code::InvalidArgument,
        Code::PermissionDenied,
        Code::AlreadyExists,
        Code::Unauthenticated,
    ] {
        let status = tonic::Status::new(code, "biz error");
        assert!(!is_retryable(code), "{:?} 应不可重试", code);
        let d = retry_backoff(&status, 0, &cfg);
        assert!(d.is_none(), "{:?} 业务错误不应 backoff", code);
    }
}

/// 场景 3: TLS 文件缺失时, build_channel 错误链应保留 TlsError::FileRead path
/// (验证 channel::Tls 变体对 tls::TlsError 的透明传递)
#[tokio::test]
async fn it_channel_tls_missing_ca_returns_proper_error_chain() {
    use shared_platform::channel::{build_channel, ChannelError};
    let cfg = RpcChannelConfig {
        uri: "https://127.0.0.1:50051".to_string(),
        connect_timeout: Duration::from_millis(50),
        request_timeout: Duration::from_millis(50),
        // 关键: tls=Some 但文件不存在, 应该走 Tls 错误路径
        tls: Some(shared_platform::tls::ClientTlsConfigInput {
            domain: "player.local".to_string(),
            ca_cert_path: "/nonexistent/it-ca.pem".to_string(),
            client_cert_path: "/nonexistent/it-client.pem".to_string(),
            client_key_path: "/nonexistent/it-client.key".to_string(),
        }),
        require_tls: true,
        retry: RetryConfig::default(),
    };
    let result = build_channel(&cfg).await;
    // 错误应是 ChannelError::Tls(TlsError::FileRead { path: "...ca.pem..." })
    match result {
        Err(ChannelError::Tls(TlsError::FileRead { path, .. })) => {
            assert!(
                path.contains("it-ca.pem"),
                "错误 path 应指向 ca 文件, 实际: {}",
                path
            );
        }
        Err(ChannelError::Tls(other)) => {
            panic!("期望 TlsError::FileRead, 实际: {:?}", other);
        }
        other => panic!("期望 ChannelError::Tls, 实际: {:?}", other),
    }
}

/// 场景 4 (附加): fail-closed 默认值契约 — 在 IT 层也守住
/// (per RGS-REV-007 CH4 + DEC-015 P1: tls=None + require_tls=true → TlsRequired)
#[tokio::test]
async fn it_channel_default_require_tls_is_true() {
    let cfg = RpcChannelConfig::default();
    assert!(
        cfg.require_tls,
        "RpcChannelConfig::default() 必须 require_tls=true (fail-closed)"
    );
    // tls 必须 None by default
    assert!(cfg.tls.is_none(), "默认 tls 应为 None");
}

/// 场景 5 (附加): mTLS bypass 计数器在 IT 视角下仍可观察
/// (per RGS-REV-007 CH4 监控项: build_channel opt-out 时 mtls_bypassed_total++)
#[tokio::test]
async fn it_channel_bypass_counter_observable_across_builds() {
    use shared_platform::channel::build_channel;
    let before = mtls_bypassed_total();
    let cfg = RpcChannelConfig {
        uri: "http://127.0.0.1:1".to_string(), // 无服务监听
        connect_timeout: Duration::from_millis(1),
        require_tls: false, // 显式 opt-out
        ..Default::default()
    };
    let _ = build_channel(&cfg).await; // 失败也可, counter 已 +1
    let after = mtls_bypassed_total();
    assert!(
        after > before,
        "bypass 计数器必须可观察增长, before={} after={}",
        before,
        after
    );
}
