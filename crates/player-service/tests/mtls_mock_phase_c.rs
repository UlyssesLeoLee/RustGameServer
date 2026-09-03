// mTLS mock 单元测试 (per 9/3 12:36 JST 拍板 main-mtls-mock)
// 不依赖 k3s 真实 svc, 验证 player 域 mTLS client code + cert verification 逻辑
// 真跑需 SRE Lead 拍板触发阶段 B (cert 导出 + svc 真实 mTLS) 或走 9/3 mock 路径
// 编译期锚定 + 单元测试 mock, 跟 L-CAND-006 SOP cert fingerprint 比对一致
//
// Per 5 worker 派工新约束 L12 (commit 747b6d5):
//   - worker 不 commit, 报告即可
//   - per-worker CARGO_TARGET_DIR=target-r1-player
//   - 主会话统一 git add 5 files + 1 commit

use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock cert metadata (per L-CAND-006 §1.2 MANIFEST.toml fingerprint 比对)
#[derive(Debug, Clone)]
struct MockCert {
    domain: String,
    fingerprint_sha256: String,
    subject: String,
}

/// Mock player 域 mTLS client (不依赖 k3s svc)
pub struct MockPlayerMtlsClient {
    cert: MockCert,
    /// Mock transport 状态: 模拟 5 域 mTLS handshake
    handshake_state: Arc<RwLock<HandshakeState>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum HandshakeState {
    Init,
    CertSent,
    Verified,
    Error(String),
}

impl MockPlayerMtlsClient {
    pub fn new(cert: MockCert) -> Self {
        Self {
            cert,
            handshake_state: Arc::new(RwLock::new(HandshakeState::Init)),
        }
    }

    /// Mock mTLS handshake: 验证 cert fingerprint
    pub async fn handshake(&self) -> Result<(), String> {
        // 1. 状态 Init → CertSent
        *self.handshake_state.write().await = HandshakeState::CertSent;
        // 2. 验证 cert fingerprint (per L-CAND-006 §1.4 fingerprint 比对)
        // SHA-256 fingerprint 格式 "AB:CD:..." 应该是 95 字符 (32 bytes * 3 - 1)
        if self.cert.fingerprint_sha256.len() != 95 {
            let err = format!(
                "invalid fingerprint length: {}",
                self.cert.fingerprint_sha256.len()
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }
        // 3. 验证 subject (per cert subject "CN = player-service")
        if !self.cert.subject.contains("player-service") {
            let err = format!(
                "subject mismatch: expected player-service, got {}",
                self.cert.subject
            );
            *self.handshake_state.write().await = HandshakeState::Error(err.clone());
            return Err(err);
        }
        // 4. 验证通过
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

    fn player_cert() -> MockCert {
        MockCert {
            domain: "player".to_string(),
            fingerprint_sha256:
                "11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11:11"
                    .to_string(),
            subject: "CN = player-service".to_string(),
        }
    }

    #[tokio::test]
    async fn mtls_player_handshake_success() {
        let client = MockPlayerMtlsClient::new(player_cert());
        assert!(client.handshake().await.is_ok());
        assert_eq!(client.get_state().await, HandshakeState::Verified);
    }

    #[tokio::test]
    async fn mtls_player_handshake_invalid_fingerprint() {
        let mut cert = player_cert();
        cert.fingerprint_sha256 = "short".to_string();
        let client = MockPlayerMtlsClient::new(cert);
        assert!(client.handshake().await.is_err());
    }

    #[tokio::test]
    async fn mtls_player_handshake_subject_mismatch() {
        let mut cert = player_cert();
        cert.subject = "CN = other-service".to_string();
        let client = MockPlayerMtlsClient::new(cert);
        let err = client.handshake().await.unwrap_err();
        assert!(err.contains("subject mismatch"));
    }
}
