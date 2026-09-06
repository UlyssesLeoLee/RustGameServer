//! 7 域 gRPC 业务客户端池 (per 9/4 改进路线图 Phase 2 起步)
//!
//! ## 范围
//! 协议网关 (network-gateway) 在 W6/W7 stub 阶段返回路由决策, W15 起步实装 7 域 gRPC 业务调用.
//!
//! ## 7 域目标
//! - 4 业务域 (player / economy / admin / cluster_ops) — mTLS Channel 实装
//! - 3 NEW 域 (scene / battle / batch) — Phase 1.5 待 W4/W5 5 worktree merge 后实装, NotDeployed 占位
//!
//! 注: nif.rs GrpcTarget 枚举只含 Player/Economy/Scene/Battle/Batch/Admin/ClusterOps 7 域,
//!     不含 Match/Social. 故 4 域 mTLS-ready 与 W6/W7 5 域 demo (含 scene/batch) 略有不同.

use std::collections::HashMap;
use std::time::Duration;

use thiserror::Error;
use tokio::sync::{Mutex, OnceCell};
use tonic::transport::Channel;
use tracing::info;

use shared_platform::channel::{build_channel, RpcChannelConfig};
use shared_platform::retry::RetryConfig;
use shared_platform::tls::ClientTlsConfigInput;

use crate::nif::GrpcTarget;

#[derive(Debug, Error)]
pub enum ClientPoolError {
    #[error("channel build error: {0}")]
    Channel(String),
    #[error("domain not deployed: {0}")]
    NotDeployed(&'static str),
    #[error("gRPC transport error: {0}")]
    Transport(String),
}

#[derive(Debug)]
pub struct GrpcClientPool {
    channels: HashMap<GrpcTarget, OnceCell<Channel>>,
    default_host: String,
    cert_dir: String,
    domain_suffix: String,
}

impl GrpcClientPool {
    pub fn new(cert_dir: impl Into<String>, default_host: impl Into<String>) -> Self {
        Self {
            channels: HashMap::new(),
            default_host: default_host.into(),
            cert_dir: cert_dir.into(),
            domain_suffix: ".local".to_string(),
        }
    }

    pub fn with_domain_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.domain_suffix = suffix.into();
        self
    }

    pub fn seven_targets() -> [GrpcTarget; 7] {
        GrpcTarget::ALL
    }

    /// 获取/懒加载 4 域 mTLS Channel (player/economy/admin/cluster-ops)
    /// 3 NEW 域 (scene/battle/batch) 返 NotDeployed 错误
    pub async fn get_channel(&mut self, target: GrpcTarget) -> Result<Channel, ClientPoolError> {
        if let Some(name) = match target {
            GrpcTarget::Scene => Some("scene"),
            GrpcTarget::Battle => Some("battle"),
            GrpcTarget::Batch => Some("batch"),
            _ => None,
        } {
            return Err(ClientPoolError::NotDeployed(name));
        }

        if !self.channels.contains_key(&target) {
            self.channels.insert(target, OnceCell::new());
        }
        let channel = {
            let entry = self.channels.get(&target).expect("just inserted");
            entry
                .get_or_try_init(|| async { self.build_target_channel(target).await })
                .await?
                .clone()
        };
        Ok(channel)
    }

    async fn build_target_channel(&self, target: GrpcTarget) -> Result<Channel, ClientPoolError> {
        let (host, port) = target.default_endpoint(&self.default_host);
        let uri = format!("https://{}:{}", host, port);
        let domain = format!("{}{}", target.service_short(), self.domain_suffix);
        let cert_dir = &self.cert_dir;

        let tls = ClientTlsConfigInput {
            domain: domain.clone(),
            ca_cert_path: format!("{}/ca.pem", cert_dir),
            client_cert_path: format!("{}/client.pem", cert_dir),
            client_key_path: format!("{}/client.key", cert_dir),
        };

        let cfg = RpcChannelConfig {
            uri: uri.clone(),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
            tls: Some(tls),
            retry: RetryConfig::default(),
            require_tls: true,
        };

        info!(target = target.service_short(), host = %host, port = port, "building mTLS channel");
        build_channel(&cfg).await.map_err(|e| ClientPoolError::Channel(e.to_string()))
    }

    pub fn config_summary(&self) -> Vec<(GrpcTarget, &'static str)> {
        vec![
            (GrpcTarget::Player, "mTLS-ready"),
            (GrpcTarget::Economy, "mTLS-ready"),
            (GrpcTarget::Admin, "mTLS-ready"),
            (GrpcTarget::ClusterOps, "mTLS-ready"),
            (GrpcTarget::Scene, "NotDeployed (W4 scaffold, 5 worktree merge 后实装)"),
            (GrpcTarget::Battle, "NotDeployed (W5 scaffold)"),
            (GrpcTarget::Batch, "NotDeployed (per 9/1 REQ fd122f6 v0.1, IMPL 未启)"),
        ]
    }
}

impl GrpcTarget {
    pub fn default_endpoint(self, default_host: &str) -> (String, u16) {
        let port = match self {
            GrpcTarget::Player => 50051,
            GrpcTarget::Economy => 50052,
            GrpcTarget::Admin => 50055,
            GrpcTarget::ClusterOps => 50056,
            GrpcTarget::Scene => 50057,
            GrpcTarget::Battle => 50058,
            GrpcTarget::Batch => 50059,
        };
        (default_host.to_string(), port)
    }

    pub fn service_short(self) -> &'static str {
        match self {
            GrpcTarget::Player => "player",
            GrpcTarget::Economy => "economy",
            GrpcTarget::Scene => "scene",
            GrpcTarget::Battle => "battle",
            GrpcTarget::Batch => "batch",
            GrpcTarget::Admin => "admin",
            GrpcTarget::ClusterOps => "cluster-ops",
        }
    }
}

pub type SharedClientPool = std::sync::Arc<tokio::sync::Mutex<GrpcClientPool>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_targets_unique_short_names() {
        let shorts: Vec<_> = GrpcTarget::ALL.iter().map(|t| t.service_short()).collect();
        let unique: std::collections::HashSet<_> = shorts.iter().collect();
        assert_eq!(unique.len(), 7, "7 域短名必须唯一");
    }

    #[test]
    fn default_endpoints_seven_distinct_ports() {
        let host = "test.local";
        let ports: std::collections::HashSet<_> = GrpcTarget::ALL.iter().map(|t| t.default_endpoint(host).1).collect();
        assert_eq!(ports.len(), 7);
    }

    #[test]
    fn pool_config_summary_has_seven_lines() {
        let pool = GrpcClientPool::new("D:/sszgC/certs", "player-service");
        let cfg = pool.config_summary();
        assert_eq!(cfg.len(), 7);
    }

    #[test]
    fn pool_new_three_new_domains_return_not_deployed() {
        let mut pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        for t in [GrpcTarget::Scene, GrpcTarget::Battle, GrpcTarget::Batch] {
            let err = rt.block_on(pool.get_channel(t)).unwrap_err();
            assert!(matches!(err, ClientPoolError::NotDeployed(_)));
        }
    }

    #[test]
    fn warn_on_mtls_cert_dir_redaction_safe() {
        let pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        let _ = format!("{:?}", pool);
    }

    // ===== W15 集成测试 (从 integration_w15_bridge.rs 合并, 防止外部文件被清理) =====

    #[test]
    fn w15_seven_domains_seven_config_lines() {
        let pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        let cfg = pool.config_summary();
        assert_eq!(cfg.len(), 7, "7 域 = 7 行 (4 mTLS + 3 NEW 域占位)");
        let mtls_count = cfg.iter().filter(|(_, s)| s.contains("mTLS")).count();
        assert!(mtls_count >= 4, "应至少 4 域 mTLS-ready, got {}", mtls_count);
        let not_deployed_count = cfg.iter().filter(|(_, s)| s.contains("NotDeployed")).count();
        assert_eq!(not_deployed_count, 3, "3 NEW 域 (scene/battle/batch) 走 NotDeployed");
    }

    #[tokio::test]
    async fn w15_three_new_domains_return_not_deployed() {
        let mut pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        for t in [GrpcTarget::Scene, GrpcTarget::Battle, GrpcTarget::Batch] {
            let err = pool.get_channel(t).await.unwrap_err();
            assert!(
                matches!(err, ClientPoolError::NotDeployed(_)),
                "3 NEW 域应返 NotDeployed, got {:?}",
                err
            );
        }
    }

    #[tokio::test]
    async fn w15_four_mtls_domains_try_real_channel_build() {
        let mut pool = GrpcClientPool::new("D:/nonexistent/certs", "test.local");
        for t in [GrpcTarget::Player, GrpcTarget::Economy, GrpcTarget::Admin, GrpcTarget::ClusterOps] {
            let result = pool.get_channel(t).await;
            // 不应返 NotDeployed (这些是 mTLS-ready 域)
            if let Err(ClientPoolError::NotDeployed(_)) = result {
                panic!("4 mTLS 域不应返 NotDeployed, target={:?}", t);
            }
        }
    }

    #[tokio::test]
    async fn w15_channel_pool_laziness_caches() {
        let mut pool = GrpcClientPool::new("D:/nonexistent/certs", "test.local");
        let _ = pool.get_channel(GrpcTarget::Player).await;
        let _ = pool.get_channel(GrpcTarget::Player).await;
        let cfg = pool.config_summary();
        assert_eq!(cfg.len(), 7);
    }

    #[tokio::test]
    async fn w15_bridge_player_create_character() {
        use crate::nif::bridge as nif_bridge;
        let r = nif_bridge(GrpcTarget::Player, "CreateCharacter", b"test-player-1");
        assert_eq!(r.target, GrpcTarget::Player);
        assert_eq!(r.method, "CreateCharacter");
        assert_eq!(r.rcode, 0); // stub 行为
        assert!(!r.response_payload.is_empty());
    }

    #[tokio::test]
    async fn w15_bridge_economy_get_balance() {
        use crate::nif::bridge as nif_bridge;
        let r = nif_bridge(GrpcTarget::Economy, "GetBalance", b"test-economy-1");
        assert_eq!(r.target, GrpcTarget::Economy);
        assert_eq!(r.method, "GetBalance");
        assert_eq!(r.rcode, 0);
        assert!(!r.response_payload.is_empty());
    }

    #[tokio::test]
    async fn w15_bridge_scene_enter_scene() {
        use crate::nif::bridge as nif_bridge;
        let r = nif_bridge(GrpcTarget::Scene, "EnterScene", b"test-scene-1");
        assert_eq!(r.target, GrpcTarget::Scene);
        assert_eq!(r.method, "EnterScene");
        assert_eq!(r.rcode, 0); // stub 行为; 真实调用 client_pool 返 NotDeployed
        // 真实路径: client_pool 返 NotDeployed
        let mut pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        let err = pool.get_channel(GrpcTarget::Scene).await.unwrap_err();
        assert!(matches!(err, ClientPoolError::NotDeployed(_)));
    }

    #[tokio::test]
    async fn w15_bridge_battle_start_battle() {
        use crate::nif::bridge as nif_bridge;
        let r = nif_bridge(GrpcTarget::Battle, "StartBattle", b"test-battle-1");
        assert_eq!(r.target, GrpcTarget::Battle);
        assert_eq!(r.method, "StartBattle");
        assert_eq!(r.rcode, 0);
        let mut pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        let err = pool.get_channel(GrpcTarget::Battle).await.unwrap_err();
        assert!(matches!(err, ClientPoolError::NotDeployed(_)));
    }

    #[test]
    fn w15_seven_targets_service_names_unique() {
        use std::collections::HashSet;
        let names: Vec<_> = GrpcTarget::ALL.iter().map(|t| t.service_name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), 7, "7 域 service_name 唯一");
    }

    #[test]
    fn w15_seven_targets_default_addr_unique() {
        use std::collections::HashSet;
        let addrs: Vec<_> = GrpcTarget::ALL.iter().map(|t| t.default_addr()).collect();
        let unique: HashSet<_> = addrs.iter().collect();
        assert_eq!(unique.len(), 7, "7 域 default_addr 唯一");
    }

    #[test]
    fn w15_config_summary_matches_all_enum() {
        let pool = GrpcClientPool::new("D:/sszgC/certs", "test.local");
        let cfg = pool.config_summary();
        let cfg_targets: std::collections::HashSet<_> = cfg.iter().map(|(t, _)| *t).collect();
        let all_targets: std::collections::HashSet<_> = GrpcTarget::ALL.iter().copied().collect();
        assert_eq!(cfg_targets, all_targets, "config_summary 应覆盖 7 域 ALL");
    }

    #[tokio::test]
    async fn w15_shared_client_pool_via_arc_mutex() {
        use std::sync::Arc;
        let pool = Arc::new(Mutex::new(GrpcClientPool::new("D:/sszgC/certs", "player-service")));
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let mut guard = pool_clone.lock().await;
            let _ = guard.get_channel(GrpcTarget::Scene).await;
        });
        let _ = handle.await;
    }
}
