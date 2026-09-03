// mTLS mock 单元测试 (per 9/3 12:36 JST 拍板 main-mtls-mock)
// 不依赖 k3s 真实 svc, 验证 admin 域 mTLS client code + cert verification 逻辑

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MockCert {
    domain: String,
    fingerprint_sha256: String,
    subject: String,
}

pub struct MockAdminMtlsClient {
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

impl MockAdminMtlsClient {
    pub fn new(cert: MockCert) -> Self {
        Self {
            cert,
            handshake_state: Arc::new(RwLock::new(HandshakeState::Init)),
        }
    }

    pub async fn handshake(&self) -> Result<(), String> {
        *self.handshake_state.write().await = HandshakeState::CertSent;

        // fingerprint 长度必须是 95 字符 (47 字节 hex + 46 个 ':' 分隔符)
        if self.cert.fingerprint_sha256.len() != 95 {
            let err = format!(
                "invalid fingerprint length: {}",
                self.cert.fingerprint_sha256.len()
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }

        // subject 必须包含 admin-service
        if !self.cert.subject.contains("admin-service") {
            let err = format!(
                "subject mismatch: expected admin-service, got {}",
                self.cert.subject
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }

        *self.handshake_state.write().await = HandshakeState::Verified;
        Ok(())
    }

    pub async fn get_state(&self) -> HandshakeState {
        let guard = self.handshake_state.read().await;
        (*guard).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_cert() -> MockCert {
        MockCert {
            domain: "admin".to_string(),
            // 47 字节 hex 串 + 46 个 ':' = 95 字符
            fingerprint_sha256:
                "55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55:55"
                    .to_string(),
            subject: "CN = admin-service".to_string(),
        }
    }

    #[tokio::test]
    async fn mtls_admin_handshake_success() {
        let client = MockAdminMtlsClient::new(admin_cert());
        assert!(client.handshake().await.is_ok());
        assert_eq!(client.get_state().await, HandshakeState::Verified);
    }

    #[tokio::test]
    async fn mtls_admin_handshake_invalid_fingerprint() {
        let mut cert = admin_cert();
        cert.fingerprint_sha256 = "short".to_string();
        let client = MockAdminMtlsClient::new(cert);
        assert!(client.handshake().await.is_err());
    }

    #[tokio::test]
    async fn mtls_admin_handshake_subject_mismatch() {
        let mut cert = admin_cert();
        cert.subject = "CN = other-service".to_string();
        let client = MockAdminMtlsClient::new(cert);
        let err = client.handshake().await.unwrap_err();
        assert!(err.contains("subject mismatch"));
    }
}
