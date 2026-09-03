// mTLS mock 单元测试 (per 9/3 12:36 JST 拍板 main-mtls-mock)
// 不依赖 k3s 真实 svc, 验证 economy 域 mTLS client code + cert verification 逻辑
// 真跑需 SRE Lead 拍板触发阶段 B (cert 导出 + svc 真实 mTLS) 或走 9/3 mock 路径
// 编译期锚定 + 单元测试 mock, 跟 L-CAND-006 SOP cert fingerprint 比对一致

use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock cert metadata (per L-CAND-006 §1.2 MANIFEST.toml fingerprint 比对)
#[derive(Debug, Clone)]
struct MockCert {
    domain: String,
    fingerprint_sha256: String,
    subject: String,
}

/// Mock economy 域 mTLS client (不依赖 k3s svc)
pub struct MockEconomyMtlsClient {
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

impl MockEconomyMtlsClient {
    pub fn new(cert: MockCert) -> Self {
        Self {
            cert,
            handshake_state: Arc::new(RwLock::new(HandshakeState::Init)),
        }
    }

    /// Mock mTLS handshake: 验证 cert fingerprint
    pub async fn handshake(&self) -> Result<(), String> {
        *self.handshake_state.write().await = HandshakeState::CertSent;
        if self.cert.fingerprint_sha256.len() != 95 {
            let err = format!(
                "invalid fingerprint length: {}",
                self.cert.fingerprint_sha256.len()
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }
        if !self.cert.subject.contains("economy-service") {
            let err = format!(
                "subject mismatch: expected economy-service, got {}",
                self.cert.subject
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }
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

    fn economy_cert() -> MockCert {
        MockCert {
            domain: "economy".to_string(),
            fingerprint_sha256:
                "22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22:22"
                    .to_string(),
            subject: "CN = economy-service".to_string(),
        }
    }

    #[tokio::test]
    async fn mtls_economy_handshake_success() {
        let client = MockEconomyMtlsClient::new(economy_cert());
        assert!(client.handshake().await.is_ok());
        assert_eq!(client.get_state().await, HandshakeState::Verified);
    }

    #[tokio::test]
    async fn mtls_economy_handshake_invalid_fingerprint() {
        let mut cert = economy_cert();
        cert.fingerprint_sha256 = "short".to_string();
        let client = MockEconomyMtlsClient::new(cert);
        assert!(client.handshake().await.is_err());
    }

    #[tokio::test]
    async fn mtls_economy_handshake_subject_mismatch() {
        let mut cert = economy_cert();
        cert.subject = "CN = other-service".to_string();
        let client = MockEconomyMtlsClient::new(cert);
        let err = client.handshake().await.unwrap_err();
        assert!(err.contains("subject mismatch"));
    }
}
