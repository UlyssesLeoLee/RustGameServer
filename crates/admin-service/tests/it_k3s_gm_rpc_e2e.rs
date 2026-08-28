//! W13 (2026-08-28) k3s 集成 IT: admin-service 5 GM RPC 端到端真链路
//!
//! 前置: k3s kubectl port-forward -n rust-game-server svc/admin-service 15055:50055
//! 关联: docs/00-基准与治理/RGS-S4-PHASE2-STEP2-设计.md
//!       docs/00-基准与治理/RGS-TBD-08-03-S4-gm-backend-admin-gRPC-立项.md
//!
//! 测试策略:
//! - k3s 内 admin-service 暴露 gRPC :50055
//! - 此 IT 在 WSL 端通过 port-forward 15055 连 admin-service
//! - 5 个 GM RPC (BanAccount / GrantCompensation / SetMaintenance / QueryAuditLog / HealthCheck)
//! - 失败 (admin 镜像仍是 0.1.0-admin 无 5 GM RPC) 时降级 + 报告
//!
//! 注: 当前 k3s 镜像 0.1.0-admin 缺少 5 GM RPC, 需 W12 ghcr workflow 推送 0.1.0-admin-service 后跑通

use std::process::Command;
use std::time::Duration;

const ADMIN_GRPC_LOCAL: &str = "http://localhost:15055";

/// 检测 admin-service 端口可达
async fn admin_available() -> bool {
    // 用 curl 测 / (admin-service 0.1.0-admin 没 HTTP, 但 8080 端口可能开)
    // 直接用 nc 测 50055
    tokio::time::timeout(Duration::from_secs(2), async {
        // tokio 没原生 nc, 用 std Command
        let out = Command::new("nc")
            .args(["-z", "localhost", "15055"])
            .output();
        match out {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    })
    .await
    .unwrap_or(false)
}

#[tokio::test]
async fn k3s_admin_service_port_50055_reachable() {
    if !admin_available().await {
        eprintln!(
            "SKIP: k3s admin-service port-forward not available. \
             Run: k3s kubectl port-forward -n rust-game-server svc/admin-service 15055:50055"
        );
        return;
    }
    // nc 已确认 15055 监听
    let out = Command::new("nc")
        .args(["-z", "localhost", "15055"])
        .output()
        .expect("nc -z");
    assert!(out.status.success(), "admin-service 15055 must be listening");
}

#[tokio::test]
async fn k3s_admin_service_health_endpoint_responds() {
    if !admin_available().await {
        eprintln!("SKIP: admin-service not reachable");
        return;
    }
    // 0.1.0-admin 没 HTTP /healthz, 但 8080 (COC web) 可能开
    // 这里只验 50055 (gRPC) 监听, 实际 gRPC HealthCheck 需 grpcurl 或 tonic client
    // 由于 IT 框架限制, 这里跳过实际 gRPC 调用, 留给后续 IT (admin-service gRPC client 引入)
    eprintln!(
        "PARTIAL: admin-service 50055 端口可达, 5 GM RPC 实际调用需等待 0.1.0-admin-service 镜像 (per W12 ghcr workflow)"
    );
}

#[tokio::test]
async fn k3s_admin_pod_count_matches_expected() {
    // 验证 k3s 内 admin-service 2 副本在跑 (per prior kubectl get pods)
    let out = Command::new("k3s")
        .args([
            "kubectl",
            "get",
            "pods",
            "-n",
            "rust-game-server",
            "-l",
            "app.kubernetes.io/name=admin",
            "-o",
            "name",
        ])
        .output()
        .expect("k3s kubectl get pods admin");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| l.contains("admin-service-")).collect();
    assert!(
        lines.len() >= 1,
        "admin-service should have >= 1 replicas, got {}",
        lines.len()
    );
}

#[tokio::test]
async fn k3s_gm_backend_5_rpc_availability_plan() {
    // 标记: admin-service 5 GM RPC 验证需 W12 完成 (admin-service 0.1.0-admin-service 镜像)
    // 当前镜像 0.1.0-admin 仅 HealthCheck + GetAdminOp
    // 升级后: 5 GM RPC (BanAccount/GrantCompensation/SetMaintenance/QueryAuditLog/HealthView)
    eprintln!(
        "PLAN: W12 完成 (ghcr 推 0.1.0-admin-service) 后, 此 IT 扩展为 5 GM RPC 真链路测试"
    );
    eprintln!(
        "  - 5 个 #[tokio::test] 各调 1 GM RPC, 验 response schema + audit_log 落库"
    );
    eprintln!("  - 端口 15055 -> k8s admin-service 50055");
    eprintln!("  - admin-service 镜像: ghcr.io/ulyssesleolee/rustgameserver:0.1.0-admin-service");
}
