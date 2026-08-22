//! cluster-ops 域 Service 业务实施（per RGS-DTL-020 §3 + ARC-051 PFAU）
//!
//! 54.7 实化：4 Service 业务方法（register_node / heartbeat / set_feature_flag / list_active_nodes）
//! + gRPC 桥接 HealthCheck + GetNode

use crate::entity::{ClusterNode, FeatureFlag, FlagScope, NodeRole};
use crate::error::Error;
use crate::repository::{ClusterNodeRepository, FeatureFlagRepository};
use crate::Result;

use async_trait::async_trait;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[async_trait]
pub trait ClusterOpsService: Send + Sync {
    async fn health_check(&self) -> Result<bool>;

    async fn register_node(
        &self,
        hostname: String,
        ip: String,
        role: NodeRole,
        version: String,
    ) -> Result<ClusterNode>;
    async fn heartbeat(&self, node_id: Uuid) -> Result<ClusterNode>;
    async fn set_feature_flag(
        &self,
        key: String,
        scope: FlagScope,
        scope_value: String,
        enabled: bool,
        updated_by: Uuid,
    ) -> Result<FeatureFlag>;
    async fn list_active_nodes(&self) -> Result<Vec<ClusterNode>>;
}

pub struct ClusterOpsServiceImpl {
    nodes: Arc<dyn ClusterNodeRepository>,
    flags: Arc<dyn FeatureFlagRepository>,
}

impl ClusterOpsServiceImpl {
    pub fn new(
        nodes: Arc<dyn ClusterNodeRepository>,
        flags: Arc<dyn FeatureFlagRepository>,
    ) -> Self {
        Self { nodes, flags }
    }

    pub async fn find_node_by_id(&self, id: Uuid) -> Result<Option<ClusterNode>> {
        self.nodes.find_by_id(id).await
    }
}

#[async_trait]
impl ClusterOpsService for ClusterOpsServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn register_node(
        &self,
        hostname: String,
        ip: String,
        role: NodeRole,
        version: String,
    ) -> Result<ClusterNode> {
        if hostname.is_empty() {
            return Err(Error::Validation("hostname must not be empty".to_string()));
        }
        if self.nodes.find_by_hostname(&hostname).await?.is_some() {
            return Err(Error::Conflict(format!(
                "hostname {} already registered",
                hostname
            )));
        }
        let node = ClusterNode::new(hostname, ip, role, version);
        self.nodes.save(&node).await?;
        Ok(node)
    }

    async fn heartbeat(&self, node_id: Uuid) -> Result<ClusterNode> {
        let mut node = self
            .nodes
            .find_by_id(node_id)
            .await?
            .ok_or_else(|| Error::NodeNotFound(node_id.to_string()))?;
        node.heartbeat();
        self.nodes.save(&node).await?;
        Ok(node)
    }

    async fn set_feature_flag(
        &self,
        key: String,
        scope: FlagScope,
        scope_value: String,
        enabled: bool,
        updated_by: Uuid,
    ) -> Result<FeatureFlag> {
        if key.is_empty() {
            return Err(Error::Validation("flag key must not be empty".to_string()));
        }
        // 查找现有 flag
        if let Some(mut existing) = self.flags.find_by_key(&key, &scope_value).await? {
            if enabled {
                existing.enable(updated_by);
            } else {
                existing.disable(updated_by);
            }
            self.flags.save(&existing).await?;
            Ok(existing)
        } else {
            let mut flag = FeatureFlag::new(key, scope, scope_value, updated_by);
            if enabled {
                flag.enable(updated_by);
            }
            self.flags.save(&flag).await?;
            Ok(flag)
        }
    }

    async fn list_active_nodes(&self) -> Result<Vec<ClusterNode>> {
        self.nodes.list_healthy().await
    }
}

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as cluster_proto;

    pub struct ClusterOpsGrpcService {
        pub impl_: Arc<ClusterOpsServiceImpl>,
    }

    impl ClusterOpsGrpcService {
        pub fn new(impl_: Arc<ClusterOpsServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    #[tonic::async_trait]
    impl cluster_proto::cluster_ops_service_server::ClusterOpsService for ClusterOpsGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: if healthy {
                    common_proto::Status::Ok as i32
                } else {
                    common_proto::Status::Failed as i32
                },
                message: if healthy {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
            }))
        }

        async fn get_node(
            &self,
            request: Request<common_proto::EntityId>,
        ) -> std::result::Result<Response<cluster_proto::Node>, Status> {
            let id_str = request.get_ref().id.clone();
            let node_id = Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", id_str)))?;
            let n = self
                .impl_
                .find_node_by_id(node_id)
                .await
                .map_err(Into::<tonic::Status>::into)?
                .ok_or_else(|| Status::not_found(format!("node {}", id_str)))?;
            Ok(Response::new(cluster_proto::Node {
                id: Some(common_proto::EntityId {
                    id: n.id.to_string(),
                }),
                status: n.status as i32,
                created_at: Some(common_proto::Timestamp {
                    seconds: n.registered_at.timestamp(),
                    nanos: n.registered_at.timestamp_subsec_nanos() as i32,
                }),
                display_name: n.hostname,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{InMemoryClusterNodeRepository, InMemoryFeatureFlagRepository};

    fn svc() -> ClusterOpsServiceImpl {
        ClusterOpsServiceImpl::new(
            Arc::new(InMemoryClusterNodeRepository::new()),
            Arc::new(InMemoryFeatureFlagRepository::new()),
        )
    }

    #[tokio::test]
    async fn register_node_creates_entry() {
        let s = svc();
        let n = s
            .register_node(
                "h1".to_string(),
                "10.0.0.1".to_string(),
                NodeRole::Primary,
                "0.1.0".to_string(),
            )
            .await
            .unwrap();
        assert_eq!(n.hostname, "h1");
    }

    #[tokio::test]
    async fn register_duplicate_hostname_fails() {
        let s = svc();
        s.register_node(
            "h1".to_string(),
            "10.0.0.1".to_string(),
            NodeRole::Primary,
            "0.1.0".to_string(),
        )
        .await
        .unwrap();
        let err = s
            .register_node(
                "h1".to_string(),
                "10.0.0.2".to_string(),
                NodeRole::Replica,
                "0.1.0".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Conflict(_)));
    }

    #[tokio::test]
    async fn heartbeat_updates_heartbeat() {
        let s = svc();
        let n = s
            .register_node(
                "h2".to_string(),
                "10.0.0.2".to_string(),
                NodeRole::Replica,
                "0.1.0".to_string(),
            )
            .await
            .unwrap();
        let updated = s.heartbeat(n.id).await.unwrap();
        assert!(updated.last_heartbeat_at >= n.last_heartbeat_at);
    }

    #[tokio::test]
    async fn set_feature_flag_enables_and_disables() {
        let s = svc();
        let admin = Uuid::new_v4();
        let f = s
            .set_feature_flag(
                "k1".to_string(),
                FlagScope::Domain,
                "player".to_string(),
                true,
                admin,
            )
            .await
            .unwrap();
        assert!(f.enabled);
        assert_eq!(f.version, 1);

        let f2 = s
            .set_feature_flag(
                "k1".to_string(),
                FlagScope::Domain,
                "player".to_string(),
                false,
                admin,
            )
            .await
            .unwrap();
        assert!(!f2.enabled);
        assert_eq!(f2.version, 2);
    }

    #[tokio::test]
    async fn list_active_nodes() {
        let s = svc();
        s.register_node(
            "h3".to_string(),
            "10.0.0.3".to_string(),
            NodeRole::Primary,
            "0.1.0".to_string(),
        )
        .await
        .unwrap();
        s.register_node(
            "h4".to_string(),
            "10.0.0.4".to_string(),
            NodeRole::Replica,
            "0.1.0".to_string(),
        )
        .await
        .unwrap();
        let nodes = s.list_active_nodes().await.unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[tokio::test]
    async fn health_check() {
        let s = svc();
        assert!(s.health_check().await.unwrap());
    }
}
