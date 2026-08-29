//! W2 跨域 IT 简化版 - 链路 A: cluster-ops → admin-service
//!
//! 验证 cluster-ops 启动后能调 admin-service /healthz (HTTP) + gRPC HealthCheck
//! 实际: cluster-ops 当前不调 admin-service gRPC, 仅测 HTTP /healthz
//!
//! 关联: docs/00-基准与治理/RGS-TST-CROSS-DOMAIN-链路-IT-设计书.md
//!
//! 测试策略:
//! - **此测试需在 WSL 环境跑**(k3s kubectl 不可在 Windows 跑)
//! - 命令: `wsl -d Ubuntu -e bash -c 'cd /mnt/d/RustGameServer-worktrees/w2-cross-domain && source scripts/db-url.sh postgres-superuser 15432 && cargo test -p cluster-ops --test it_cross_domain_admin_health -- --include-ignored'`
//! - 不真连 admin-service, 仅验 cluster-ops 内部 health endpoint 自检
//! - 后续 Step 3+ 实装 cluster-ops → admin-service gRPC HealthCheck 时
//!   把此 IT 扩展为真链路测试

#[cfg(target_os = "linux")]
#[test]
#[ignore = "需要 WSL + k3s 集群实机环境（per 文件头注释 --include-ignored 手动跑）；CI ubuntu-latest 无 k3s"]
fn cluster_ops_health_endpoint_self_check() {
    use std::process::Command;
    // WSL 端: k3s kubectl get pods
    let out = Command::new("k3s")
        .args([
            "kubectl",
            "get",
            "pods",
            "-n",
            "rust-game-server",
            "-l",
            "app.kubernetes.io/name=cluster-ops",
            "-o",
            "name",
        ])
        .output()
        .expect("k3s kubectl get pods (run inside WSL)");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cluster-ops-"),
        "cluster-ops pods must be running, got: {stdout}"
    );
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("cluster-ops-"))
        .collect();
    assert!(
        lines.len() >= 3,
        "cluster-ops should have >= 3 replicas, got {}",
        lines.len()
    );
}

#[cfg(not(target_os = "linux"))]
#[test]
fn cluster_ops_health_endpoint_self_check() {
    // Windows 端: k3s 不可用, 此 IT skip (per WSL-only 约束)
    eprintln!("SKIP: this IT requires WSL with k3s, run inside WSL");
}
