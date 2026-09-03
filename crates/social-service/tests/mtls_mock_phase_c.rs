//! mTLS mock 单元测试 (per 9/3 12:36 JST 拍板 main-mtls-mock, social 域)
//!
//! ## 目的
//! 在 Phase C SRE 介入前, 5 域 mTLS 业务级 ST 还未跑通 (k3s 真实 svc 起来 +
//! 5 域 binary mTLS handshake 前置缺失). 本 mock 单元测试作为 5 域 mTLS client
//! code + cert verification 逻辑的本地验证锚定:
//!
//! 1. **Handshake 状态机**: `Init → CertSent → Verified | Error(String)`, 用
//!    `tokio::sync::RwLock<HandshakeState>` 跟踪.
//! 2. **Cert 校验**: 指纹长度 == 95 (`AA:BB:...` SHA-256 + 95 字符), subject 必须
//!    含 `social-service` (per 5 域 cert CN 规范).
//! 3. **失败语义**: 任何 cert 字段不匹配 → 状态机转 `Error(String)`, handshake
//!    返回 `Err`, 不静默成功 (fail-closed, per DEC-014).
//!
//! ## 风格 (per 任务简报)
//! - 不依赖 k3s 真实 svc, 不依赖 rgs-testkit (per AGENTS.md §2.3 L3 派生约束 +
//!   rgs-testkit 禁 InMemory mock).
//! - mock 仅在 tests/ 目录, 不污染 src/.
//! - 临时 log 不入 commit (per L12).
//!
//! ## 已知缺口
//! - 真实 mTLS 业务级 ST 需 k3s 真实 svc + 5 域 binary 起来 + 证书导出 (per
//!   RGS-PHASE-C-PREP-2026-09-02 v0.1 §2.4), Phase C 介入后由 SRE 主导落地.
//! - 真实 tonic mTLS client (rustls/openssl) 集成在 Phase C 5 域 mTLS ST 阶段
//!   替换本 mock, 本 mock 仅验证业务层 cert 校验逻辑.

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MockCert {
    domain: String,
    fingerprint_sha256: String,
    subject: String,
}

pub struct MockSocialMtlsClient {
    cert: MockCert,
    handshake_state: Arc<RwLock<HandshakeState>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum HandshakeState {
    Init,
    CertSent,
    Verified,
    Error(String),
}

impl MockSocialMtlsClient {
    pub fn new(cert: MockCert) -> Self {
        Self {
            cert,
            handshake_state: Arc::new(RwLock::new(HandshakeState::Init)),
        }
    }

    pub async fn handshake(&self) -> Result<(), String> {
        // 1. CertSent: 状态机推进, 表示 cert 已发出
        *self.handshake_state.write().await = HandshakeState::CertSent;

        // 2. 指纹长度校验: SHA-256 `AA:BB:..:BB` 共 32 字节, 32*3 - 1 = 95 字符
        if self.cert.fingerprint_sha256.len() != 95 {
            let err = format!(
                "invalid fingerprint length: {} (expected 95)",
                self.cert.fingerprint_sha256.len()
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }

        // 3. subject CN 校验: 必须含 `social-service` (per 5 域 cert CN 规范)
        if !self.cert.subject.contains("social-service") {
            let err = format!(
                "subject mismatch: expected social-service, got {}",
                self.cert.subject
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }

        // 4. Verified: 状态机推进, 表示握手成功
        *self.handshake_state.write().await = HandshakeState::Verified;
        Ok(())
    }

    pub async fn get_state(&self) -> HandshakeState {
        self.handshake_state.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn social_cert() -> MockCert {
        MockCert {
            domain: "social".to_string(),
            // 95 字符 SHA-256 指纹: 32 字节 hex, 每 2 字符 + `:` 分隔, 32*3-1 = 95
            fingerprint_sha256:
                "44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44:44"
                    .to_string(),
            subject: "CN = social-service".to_string(),
        }
    }

    #[tokio::test]
    async fn mtls_social_handshake_success() {
        let client = MockSocialMtlsClient::new(social_cert());
        assert!(client.handshake().await.is_ok());
        assert_eq!(client.get_state().await, HandshakeState::Verified);
    }

    #[tokio::test]
    async fn mtls_social_handshake_invalid_fingerprint() {
        let mut cert = social_cert();
        cert.fingerprint_sha256 = "short".to_string();
        let client = MockSocialMtlsClient::new(cert);
        let err = client.handshake().await.unwrap_err();
        assert!(err.contains("invalid fingerprint length"));
        assert!(matches!(
            client.get_state().await,
            HandshakeState::Error(_)
        ));
    }

    #[tokio::test]
    async fn mtls_social_handshake_subject_mismatch() {
        let mut cert = social_cert();
        cert.subject = "CN = other-service".to_string();
        let client = MockSocialMtlsClient::new(cert);
        let err = client.handshake().await.unwrap_err();
        assert!(err.contains("subject mismatch"));
        assert!(matches!(
            client.get_state().await,
            HandshakeState::Error(_)
        ));
    }
}
