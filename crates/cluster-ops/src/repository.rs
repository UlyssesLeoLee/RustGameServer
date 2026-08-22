//! cluster-ops 域 Repository
//!
//! 54.6 实化：trait + PgRepository sqlx impl + InMemoryRepository
//! 规范：RGS-DTL-020 §3 + ARC-051 集群运营中心 + PFAU

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::entity::{ClusterNode, FeatureFlag, FlagScope, NodeRole, NodeStatus};
use crate::Result;

/// ClusterNode Repository trait
#[async_trait]
pub trait ClusterNodeRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClusterNode>>;
    async fn find_by_hostname(&self, hostname: &str) -> Result<Option<ClusterNode>>;
    async fn save(&self, entity: &ClusterNode) -> Result<ClusterNode>;
    async fn delete_by_id(&self, id: Uuid) -> Result<bool>;
    /// 列出所有健康节点（per DEC-001 PFAU all-reachable）
    async fn list_healthy(&self) -> Result<Vec<ClusterNode>>;
    /// 标记心跳超时的节点为不健康
    async fn mark_stale_unhealthy(&self, threshold: DateTime<Utc>) -> Result<u64>;
}

/// FeatureFlag Repository trait
#[async_trait]
pub trait FeatureFlagRepository: Send + Sync {
    async fn find_by_key(&self, key: &str, scope_value: &str) -> Result<Option<FeatureFlag>>;
    /// 按作用域 + 值列出（如 domain=player）
    async fn list_by_scope(&self, scope_value: &str) -> Result<Vec<FeatureFlag>>;
    async fn save(&self, entity: &FeatureFlag) -> Result<FeatureFlag>;
}

// ============================================================================
// PgRepository
// ============================================================================

pub struct PgClusterNodeRepository {
    pool: PgPool,
}

impl PgClusterNodeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_node(row: sqlx::postgres::PgRow) -> ClusterNode {
    let role_str: String = row.get("role");
    let status_str: String = row.get("status");
    ClusterNode {
        id: row.get("id"),
        hostname: row.get("hostname"),
        ip: row.get("ip"),
        role: parse_role(&role_str),
        status: parse_status(&status_str),
        last_heartbeat_at: row.get("last_heartbeat_at"),
        version: row.get("version"),
        registered_at: row.get("registered_at"),
        enabled_at: row.get("enabled_at"),
    }
}

#[async_trait]
impl ClusterNodeRepository for PgClusterNodeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClusterNode>> {
        let row = sqlx::query(
            "SELECT id, hostname, ip, role, status, last_heartbeat_at, version, registered_at, enabled_at \
             FROM cluster_nodes WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_node))
    }

    async fn find_by_hostname(&self, hostname: &str) -> Result<Option<ClusterNode>> {
        let row = sqlx::query(
            "SELECT id, hostname, ip, role, status, last_heartbeat_at, version, registered_at, enabled_at \
             FROM cluster_nodes WHERE hostname = $1",
        )
        .bind(hostname)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_node))
    }

    async fn save(&self, entity: &ClusterNode) -> Result<ClusterNode> {
        sqlx::query(
            "INSERT INTO cluster_nodes \
             (id, hostname, ip, role, status, last_heartbeat_at, version, registered_at, enabled_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (id) DO UPDATE SET \
                ip = EXCLUDED.ip, role = EXCLUDED.role, status = EXCLUDED.status, \
                last_heartbeat_at = EXCLUDED.last_heartbeat_at, version = EXCLUDED.version, \
                enabled_at = EXCLUDED.enabled_at",
        )
        .bind(entity.id)
        .bind(&entity.hostname)
        .bind(&entity.ip)
        .bind(role_to_str(entity.role))
        .bind(status_to_str(entity.status))
        .bind(entity.last_heartbeat_at)
        .bind(&entity.version)
        .bind(entity.registered_at)
        .bind(entity.enabled_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM cluster_nodes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_healthy(&self) -> Result<Vec<ClusterNode>> {
        let rows = sqlx::query(
            "SELECT id, hostname, ip, role, status, last_heartbeat_at, version, registered_at, enabled_at \
             FROM cluster_nodes WHERE status = 'healthy' ORDER BY hostname",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_node).collect())
    }

    async fn mark_stale_unhealthy(&self, threshold: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE cluster_nodes SET status = 'unhealthy' \
             WHERE last_heartbeat_at < $1 AND status != 'unhealthy'",
        )
        .bind(threshold)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

pub struct PgFeatureFlagRepository {
    pool: PgPool,
}

impl PgFeatureFlagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_flag(row: sqlx::postgres::PgRow) -> FeatureFlag {
    let scope_str: String = row.get("scope");
    FeatureFlag {
        key: row.get("key"),
        scope: parse_scope(&scope_str),
        scope_value: row.get("scope_value"),
        enabled: row.get("enabled"),
        version: row.get("version"),
        updated_by: row.get("updated_by"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl FeatureFlagRepository for PgFeatureFlagRepository {
    async fn find_by_key(&self, key: &str, scope_value: &str) -> Result<Option<FeatureFlag>> {
        let row = sqlx::query(
            "SELECT key, scope, scope_value, enabled, version, updated_by, updated_at \
             FROM feature_flags WHERE key = $1 AND scope_value = $2",
        )
        .bind(key)
        .bind(scope_value)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_flag))
    }

    async fn list_by_scope(&self, scope_value: &str) -> Result<Vec<FeatureFlag>> {
        let rows = sqlx::query(
            "SELECT key, scope, scope_value, enabled, version, updated_by, updated_at \
             FROM feature_flags WHERE scope_value = $1 OR scope = 'global' \
             ORDER BY key",
        )
        .bind(scope_value)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(row_to_flag).collect())
    }

    async fn save(&self, entity: &FeatureFlag) -> Result<FeatureFlag> {
        sqlx::query(
            "INSERT INTO feature_flags (key, scope, scope_value, enabled, version, updated_by, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (key, scope_value) DO UPDATE SET \
                scope = EXCLUDED.scope, enabled = EXCLUDED.enabled, \
                version = EXCLUDED.version, updated_by = EXCLUDED.updated_by, \
                updated_at = EXCLUDED.updated_at",
        )
        .bind(&entity.key)
        .bind(scope_to_str(entity.scope))
        .bind(&entity.scope_value)
        .bind(entity.enabled)
        .bind(entity.version)
        .bind(entity.updated_by)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(entity.clone())
    }
}

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryClusterNodeRepository {
    inner: Mutex<HashMap<Uuid, ClusterNode>>,
}

impl InMemoryClusterNodeRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryClusterNodeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClusterNodeRepository for InMemoryClusterNodeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ClusterNode>> {
        Ok(self.inner.lock().unwrap().get(&id).cloned())
    }
    async fn find_by_hostname(&self, hostname: &str) -> Result<Option<ClusterNode>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|n| n.hostname == hostname)
            .cloned())
    }
    async fn save(&self, entity: &ClusterNode) -> Result<ClusterNode> {
        self.inner.lock().unwrap().insert(entity.id, entity.clone());
        Ok(entity.clone())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<bool> {
        Ok(self.inner.lock().unwrap().remove(&id).is_some())
    }
    async fn list_healthy(&self) -> Result<Vec<ClusterNode>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|n| n.status == NodeStatus::Healthy)
            .cloned()
            .collect())
    }
    async fn mark_stale_unhealthy(&self, threshold: DateTime<Utc>) -> Result<u64> {
        let mut guard = self.inner.lock().unwrap();
        let mut count = 0;
        for n in guard.values_mut() {
            if n.last_heartbeat_at < threshold && n.status != NodeStatus::Unhealthy {
                n.status = NodeStatus::Unhealthy;
                count += 1;
            }
        }
        Ok(count)
    }
}

pub struct InMemoryFeatureFlagRepository {
    inner: Mutex<HashMap<String, FeatureFlag>>,
}

impl InMemoryFeatureFlagRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryFeatureFlagRepository {
    fn default() -> Self {
        Self::new()
    }
}

fn flag_key(key: &str, scope_value: &str) -> String {
    format!("{}|{}", key, scope_value)
}

#[async_trait]
impl FeatureFlagRepository for InMemoryFeatureFlagRepository {
    async fn find_by_key(&self, key: &str, scope_value: &str) -> Result<Option<FeatureFlag>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .get(&flag_key(key, scope_value))
            .cloned())
    }
    async fn list_by_scope(&self, scope_value: &str) -> Result<Vec<FeatureFlag>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .filter(|f| f.scope_value == scope_value || f.scope == FlagScope::Global)
            .cloned()
            .collect())
    }
    async fn save(&self, entity: &FeatureFlag) -> Result<FeatureFlag> {
        self.inner
            .lock()
            .unwrap()
            .insert(flag_key(&entity.key, &entity.scope_value), entity.clone());
        Ok(entity.clone())
    }
}

// ============================================================================
// helpers
// ============================================================================

fn role_to_str(r: NodeRole) -> &'static str {
    match r {
        NodeRole::Primary => "primary",
        NodeRole::Replica => "replica",
        NodeRole::Candidate => "candidate",
    }
}

fn parse_role(s: &str) -> NodeRole {
    match s {
        "primary" => NodeRole::Primary,
        "replica" => NodeRole::Replica,
        _ => NodeRole::Candidate,
    }
}

fn status_to_str(s: NodeStatus) -> &'static str {
    match s {
        NodeStatus::Healthy => "healthy",
        NodeStatus::Degraded => "degraded",
        NodeStatus::Unhealthy => "unhealthy",
        NodeStatus::Maintenance => "maintenance",
    }
}

fn parse_status(s: &str) -> NodeStatus {
    match s {
        "healthy" => NodeStatus::Healthy,
        "degraded" => NodeStatus::Degraded,
        "unhealthy" => NodeStatus::Unhealthy,
        _ => NodeStatus::Maintenance,
    }
}

fn scope_to_str(s: FlagScope) -> &'static str {
    match s {
        FlagScope::Global => "global",
        FlagScope::Domain => "domain",
        FlagScope::Node => "node",
    }
}

fn parse_scope(s: &str) -> FlagScope {
    match s {
        "global" => FlagScope::Global,
        "domain" => FlagScope::Domain,
        _ => FlagScope::Node,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_node_heartbeat() {
        let repo = InMemoryClusterNodeRepository::new();
        let n = ClusterNode::new(
            "h1".to_string(),
            "1.1.1.1".to_string(),
            NodeRole::Primary,
            "0.1.0".to_string(),
        );
        let id = n.id;
        repo.save(&n).await.unwrap();
        let loaded = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(loaded.status, NodeStatus::Healthy);
    }

    #[tokio::test]
    async fn in_memory_node_mark_stale() {
        let repo = InMemoryClusterNodeRepository::new();
        let mut n = ClusterNode::new(
            "h1".to_string(),
            "1.1.1.1".to_string(),
            NodeRole::Replica,
            "0.1.0".to_string(),
        );
        n.last_heartbeat_at = Utc::now() - chrono::Duration::seconds(120);
        repo.save(&n).await.unwrap();
        let marked = repo
            .mark_stale_unhealthy(Utc::now() - chrono::Duration::seconds(60))
            .await
            .unwrap();
        assert_eq!(marked, 1);
    }

    #[tokio::test]
    async fn in_memory_flag_list_by_scope() {
        let repo = InMemoryFeatureFlagRepository::new();
        let admin = Uuid::new_v4();
        let f1 = FeatureFlag::new(
            "k1".to_string(),
            FlagScope::Domain,
            "player".to_string(),
            admin,
        );
        let f2 = FeatureFlag::new("k2".to_string(), FlagScope::Global, "*".to_string(), admin);
        let f3 = FeatureFlag::new(
            "k3".to_string(),
            FlagScope::Domain,
            "economy".to_string(),
            admin,
        );
        repo.save(&f1).await.unwrap();
        repo.save(&f2).await.unwrap();
        repo.save(&f3).await.unwrap();
        let list = repo.list_by_scope("player").await.unwrap();
        assert_eq!(list.len(), 2); // f1 + f2 (global)
    }
}
