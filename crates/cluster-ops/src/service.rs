//! cluster-ops Service trait 定义
//!
//! 54.1 占位：1 个 health_check + 1 个 ping 占位方法。
//! 实际 gRPC method 实现待 WF-1-54.5-54.7 业务实施。

use crate::Result;
use async_trait::async_trait;

/// cluster-ops 域 Service trait
#[async_trait]
pub trait ClusterOpsService: Send + Sync {
    /// 健康检查（per gRPC health checking standard）
    async fn health_check(&self) -> Result<bool>;

    /// Ping 占位（54.1 骨架；54.7 业务实施时移除）
    async fn ping(&self) -> Result<String> {
        Ok("cluster-ops pong".to_string())
    }
}

/// cluster-ops 域默认 Service 实现（54.1 占位）
pub struct ClusterOpsServiceImpl;

impl ClusterOpsServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClusterOpsServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClusterOpsService for ClusterOpsServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        // 54.1 占位：仅返回 true
        // 54.13 OTel span 注入后改为 health check + DB ping
        Ok(true)
    }
}
