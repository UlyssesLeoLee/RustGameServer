//! 协议网关 admin gRPC 服务实现 (per W6 task 4 管理 RPC)
//!
//! ## RPC 列表 (per proto/gateway/v1/gateway.proto)
//! - HealthCheck
//! - ListRoutes
//! - RegisterRoute
//! - GetStats
//!
//! ## 实现
//! GatewayAdminService 持有 Arc<RouteTable> + Arc<GatewayStats>, 走 tonic async trait.

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::v1 as gateway_proto_v1;
use crate::router::RouteTable;
use crate::stats::GatewayStats;

pub use gateway_proto_v1::gateway_admin_server::{GatewayAdmin, GatewayAdminServer};

/// Admin gRPC service impl (4 RPC, per task brief)
pub struct GatewayAdminService {
    pub routes: Arc<RouteTable>,
    pub stats: Arc<GatewayStats>,
}

impl GatewayAdminService {
    pub fn new(routes: Arc<RouteTable>, stats: Arc<GatewayStats>) -> Self {
        Self { routes, stats }
    }

    /// 构造 tonic gRPC server (供 main.rs 挂载)
    pub fn into_server(self) -> GatewayAdminServer<Self> {
        GatewayAdminServer::new(self)
    }
}

#[tonic::async_trait]
impl GatewayAdmin for GatewayAdminService {
    async fn health_check(
        &self,
        _request: Request<gateway_proto_v1::HealthCheckRequest>,
    ) -> Result<Response<gateway_proto_v1::HealthCheckResponse>, Status> {
        // Phase 1 骨架: 始终 ok; Phase 1.5 检查 route table / stats 健康
        let resp = gateway_proto_v1::HealthCheckResponse {
            healthy: true,
            status: "ok".to_string(),
            timestamp: chrono_now_iso(),
        };
        Ok(Response::new(resp))
    }

    async fn list_routes(
        &self,
        _request: Request<gateway_proto_v1::ListRoutesRequest>,
    ) -> Result<Response<gateway_proto_v1::ListRoutesResponse>, Status> {
        let routes = self.routes.list();
        let total = routes.len() as u32;
        Ok(Response::new(gateway_proto_v1::ListRoutesResponse {
            routes,
            total,
        }))
    }

    async fn register_route(
        &self,
        request: Request<gateway_proto_v1::RegisterRouteRequest>,
    ) -> Result<Response<gateway_proto_v1::RegisterRouteResponse>, Status> {
        let req = request.into_inner();
        let entry = req.route.ok_or_else(|| {
            Status::invalid_argument("route field is required")
        })?;
        let resp = match self.routes.register(entry) {
            Ok(()) => gateway_proto_v1::RegisterRouteResponse {
                success: true,
                error: String::new(),
            },
            Err(e) => gateway_proto_v1::RegisterRouteResponse {
                success: false,
                error: e,
            },
        };
        Ok(Response::new(resp))
    }

    async fn get_stats(
        &self,
        _request: Request<gateway_proto_v1::GetStatsRequest>,
    ) -> Result<Response<gateway_proto_v1::GetStatsResponse>, Status> {
        let snap = self.stats.snapshot();
        let resp = gateway_proto_v1::GetStatsResponse {
            total_received: snap.total_received,
            total_forwarded: snap.total_forwarded,
            total_failed: snap.total_failed,
            total_route_miss: snap.total_route_miss,
            active_connections: snap.active_connections,
        };
        Ok(Response::new(resp))
    }
}

/// ISO-8601 UTC now (per HealthCheckResponse.timestamp 注释)
fn chrono_now_iso() -> String {
    // Phase 1 骨架: 不引入 chrono 直接, 用 std::time::SystemTime 转 ISO-8601
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 简单转换: 1970-01-01T00:00:00Z 起点 + secs. Phase 1.5 用 chrono 替换.
    format!("1970-01-01T00:00:{}Z", secs) // 占位格式, Phase 1.5 替换
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::v1::RegisterRouteRequest;
    use crate::router::RouteTable;
    use crate::stats::GatewayStats;

    #[tokio::test]
    async fn health_check_returns_ok() {
        let svc = GatewayAdminService::new(
            Arc::new(RouteTable::new()),
            Arc::new(GatewayStats::new()),
        );
        let resp = svc
            .health_check(Request::new(gateway_proto_v1::HealthCheckRequest {}))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert!(inner.healthy);
        assert_eq!(inner.status, "ok");
    }

    #[tokio::test]
    async fn list_routes_returns_default() {
        let svc = GatewayAdminService::new(
            Arc::new(RouteTable::new()),
            Arc::new(GatewayStats::new()),
        );
        let resp = svc
            .list_routes(Request::new(gateway_proto_v1::ListRoutesRequest {}))
            .await
            .unwrap();
        let inner = resp.into_inner();
        assert!(inner.total >= 1, "Phase 1 骨架至少 1 条");
        assert!(!inner.routes.is_empty());
    }

    #[tokio::test]
    async fn register_route_adds_new() {
        let svc = GatewayAdminService::new(
            Arc::new(RouteTable::new()),
            Arc::new(GatewayStats::new()),
        );
        let entry = gateway_proto_v1::RouteEntry {
            code: 99999,
            name: "test".into(),
            target_service: "test.v1.Test".into(),
            target_method: "Ping".into(),
            target_addr: "http://127.0.0.1:9999".into(),
        };
        let req = RegisterRouteRequest { route: Some(entry) };
        let resp = svc
            .register_route(Request::new(req))
            .await
            .unwrap()
            .into_inner();
        assert!(resp.success);
    }

    #[tokio::test]
    async fn register_route_duplicate_fails() {
        let svc = GatewayAdminService::new(
            Arc::new(RouteTable::new()),
            Arc::new(GatewayStats::new()),
        );
        let entry = gateway_proto_v1::RouteEntry {
            code: 10101, // 已在默认表
            name: "dup".into(),
            target_service: "x".into(),
            target_method: "Y".into(),
            target_addr: "z".into(),
        };
        let req = RegisterRouteRequest { route: Some(entry) };
        let resp = svc
            .register_route(Request::new(req))
            .await
            .unwrap()
            .into_inner();
        assert!(!resp.success);
        assert!(!resp.error.is_empty());
    }

    #[tokio::test]
    async fn get_stats_returns_zero_initially() {
        let svc = GatewayAdminService::new(
            Arc::new(RouteTable::new()),
            Arc::new(GatewayStats::new()),
        );
        let resp = svc
            .get_stats(Request::new(gateway_proto_v1::GetStatsRequest {}))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.total_received, 0);
        assert_eq!(resp.active_connections, 0);
    }
}
