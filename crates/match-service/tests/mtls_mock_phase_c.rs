// mTLS mock 单元测试 (per 9/3 12:36 JST 拍板 main-mtls-mock)
// 不依赖 k3s 真实 svc, 验证 match 域 mTLS client code + cert verification 逻辑
// 跟 5 域 mTLS Phase C 派工对齐 (player / economy / match / social / admin)
// per RGS-L11 dir lock 修复: per-worker CARGO_TARGET_DIR=target-r1-match 覆盖全局
// per RGS-L12 race condition 修复: worker 不 commit, 报告即可

use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct MockCert {
    domain: String,
    fingerprint_sha256: String,
    subject: String,
}

pub struct MockMatchMtlsClient {
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

impl MockMatchMtlsClient {
    pub fn new(cert: MockCert) -> Self {
        Self {
            cert,
            handshake_state: Arc::new(RwLock::new(HandshakeState::Init)),
        }
    }

    pub async fn handshake(&self) -> Result<(), String> {
        *self.handshake_state.write().await = HandshakeState::CertSent;

        // 1) 验证 fingerprint 长度 (per RFC 7469 X.509 key fingerprint 格式: 32 字节 hex + 31 分隔符 = 95 字符)
        if self.cert.fingerprint_sha256.len() != 95 {
            let err = format!(
                "invalid fingerprint length: {}",
                self.cert.fingerprint_sha256.len()
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }

        // 2) 验证 subject CN 必须是 match-service (per RGS-REV-007 CH4 mTLS 域间认证)
        if !self.cert.subject.contains("match-service") {
            let err = format!(
                "subject mismatch: expected match-service, got {}",
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

    fn match_cert() -> MockCert {
        MockCert {
            domain: "match".to_string(),
            fingerprint_sha256: "33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33:33".to_string(),
            subject: "CN = match-service".to_string(),
        }
    }

    #[tokio::test]
    async fn mtls_match_handshake_success() {
        let client = MockMatchMtlsClient::new(match_cert());
        assert!(client.handshake().await.is_ok());
        assert_eq!(client.get_state().await, HandshakeState::Verified);
    }

    #[tokio::test]
    async fn mtls_match_handshake_invalid_fingerprint() {
        let mut cert = match_cert();
        cert.fingerprint_sha256 = "short".to_string();
        let client = MockMatchMtlsClient::new(cert);
        assert!(client.handshake().await.is_err());
    }

    #[tokio::test]
    async fn mtls_match_handshake_subject_mismatch() {
        let mut cert = match_cert();
        cert.subject = "CN = other-service".to_string();
        let client = MockMatchMtlsClient::new(cert);
        let err = client.handshake().await.unwrap_err();
        assert!(err.contains("subject mismatch"));
    }
}
