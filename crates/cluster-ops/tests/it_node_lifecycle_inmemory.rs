//! cluster-ops 域 IT - 跨模块场景（per PT-WORKER-BRIEFING.md §2 Step 4）
//!
//! ## 目的
//! 验证 cluster-ops 域 repository + service + entity 三层在内存 mock 下的
//! 跨模块协作 (node 注册 → 心跳 → 状态转换 → 节点清理; feature flag
//! 跨 scope 隔离 + version 单调性; stale 节点批量标记)。
//!
//! ## 范围 (4 IT 覆盖 per 任务书要求)
//! 1. test_register_heartbeat_list_full_flow
//! 2. test_duplicate_hostname_rejection_across_service
//! 3. test_feature_flag_cross_scope_isolation
//! 4. test_stale_node_sweep_then_recovery
//!
//! ## 设计
//! - 走 InMemoryClusterNodeRepository + InMemoryFeatureFlagRepository, 不用真 PG
//! - 不依赖 DATABASE_URL / k3s / WSL
//! - 复用 service.rs 中已有的 ClusterOpsServiceImpl
//!
//! ## 跳过机制
//! - 无需 WSL / 真 DB; 任何平台都能跑

use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use cluster_ops::entity::{FlagScope, NodeRole, NodeStatus};
use cluster_ops::repository::{
    ClusterNodeRepository, InMemoryClusterNodeRepository, InMemoryFeatureFlagRepository,
};
use cluster_ops::service::{ClusterOpsService, ClusterOpsServiceImpl};

fn make_service() -> (ClusterOpsServiceImpl, Arc<InMemoryClusterNodeRepository>) {
    let node_repo = Arc::new(InMemoryClusterNodeRepository::new());
    let flag_repo = Arc::new(InMemoryFeatureFlagRepository::new());
    let svc = ClusterOpsServiceImpl::new(node_repo.clone(), flag_repo);
    (svc, node_repo)
}

/// 场景 1: register → heartbeat → list 完整链路
#[tokio::test]
async fn test_register_heartbeat_list_full_flow() {
    let (svc, node_repo) = make_service();

    // 注册 3 个节点 (Primary / Replica / Candidate)
    let n1 = svc
        .register_node(
            "primary-1".to_string(),
            "10.0.0.1".to_string(),
            NodeRole::Primary,
            "0.1.0".to_string(),
        )
        .await
        .unwrap();
    let n2 = svc
        .register_node(
            "replica-1".to_string(),
            "10.0.0.2".to_string(),
            NodeRole::Replica,
            "0.1.0".to_string(),
        )
        .await
        .unwrap();
    let n3 = svc
        .register_node(
            "candidate-1".to_string(),
            "10.0.0.3".to_string(),
            NodeRole::Candidate,
            "0.2.0-rc1".to_string(),
        )
        .await
        .unwrap();

    // 3 个节点都应 Healthy
    let initial = svc.list_active_nodes().await.unwrap();
    assert_eq!(initial.len(), 3);

    // 心跳 1 次
    let h = svc.heartbeat(n1.id).await.unwrap();
    assert!(h.last_heartbeat_at >= n1.last_heartbeat_at);
    assert_eq!(h.status, NodeStatus::Healthy);

    // 通过 repo 直接验证 last_heartbeat_at 已写入
    let loaded = node_repo.find_by_id(n1.id).await.unwrap().unwrap();
    assert_eq!(loaded.last_heartbeat_at, h.last_heartbeat_at);

    // 验证 3 节点 role 各不相同 (NodeRole 未 derive Hash, 用两两 != 断言替代 HashSet)
    assert_ne!(n1.role, n2.role);
    assert_ne!(n1.role, n3.role);
    assert_ne!(n2.role, n3.role);
}

/// 场景 2: 重复 hostname 在 service 层被拒 (Conflict error)
#[tokio::test]
async fn test_duplicate_hostname_rejection_across_service() {
    let (svc, _) = make_service();

    svc.register_node(
        "shared-host".to_string(),
        "10.0.0.1".to_string(),
        NodeRole::Primary,
        "0.1.0".to_string(),
    )
    .await
    .unwrap();

    // 同 hostname 不同 IP/role 也应被拒
    let err = svc
        .register_node(
            "shared-host".to_string(),
            "10.0.0.99".to_string(),
            NodeRole::Replica,
            "0.1.0".to_string(),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, cluster_ops::Error::Conflict(_)));

    // 同 IP 但不同 hostname 应允许 (per 现状: 仅 hostname 唯一)
    let ok = svc
        .register_node(
            "different-host".to_string(),
            "10.0.0.1".to_string(),
            NodeRole::Replica,
            "0.1.0".to_string(),
        )
        .await
        .expect("不同 hostname 应允许");
    assert_eq!(ok.hostname, "different-host");
}

/// 场景 3: feature_flag 跨 scope_value 隔离 + version 单调性
#[tokio::test]
async fn test_feature_flag_cross_scope_isolation() {
    let (svc, _) = make_service();
    let admin = Uuid::new_v4();

    // 3 个不同 scope_value 的同名 flag 互不干扰
    let f_p = svc
        .set_feature_flag(
            "exp_metric".to_string(),
            FlagScope::Domain,
            "player".to_string(),
            true,
            admin,
        )
        .await
        .unwrap();
    let f_e = svc
        .set_feature_flag(
            "exp_metric".to_string(),
            FlagScope::Domain,
            "economy".to_string(),
            false,
            admin,
        )
        .await
        .unwrap();
    let f_g = svc
        .set_feature_flag(
            "exp_metric".to_string(),
            FlagScope::Global,
            "*".to_string(),
            true,
            admin,
        )
        .await
        .unwrap();

    // version 初始都应是 1 (enable 一次)
    assert_eq!(f_p.version, 1);
    assert!(f_p.enabled);
    assert_eq!(f_e.version, 0);
    assert!(!f_e.enabled); // false → 仅 create, 不 enable
    assert_eq!(f_g.version, 1);
    assert!(f_g.enabled);

    // player scope 反复 toggle 5 次: version 应从 1 → 6
    for expected in 2..=6 {
        let f = svc
            .set_feature_flag(
                "exp_metric".to_string(),
                FlagScope::Domain,
                "player".to_string(),
                expected % 2 == 0,
                admin,
            )
            .await
            .unwrap();
        assert_eq!(f.version, expected);
    }

    // economy scope 仍独立: version 仍是 0 (上次创建后未再 set)
    // (因为 economy 上次是 false, 后续 set 才会触发 enable/disable)
    let f_e2 = svc
        .set_feature_flag(
            "exp_metric".to_string(),
            FlagScope::Domain,
            "economy".to_string(),
            true,
            admin,
        )
        .await
        .unwrap();
    assert_eq!(f_e2.version, 1);
    assert!(f_e2.enabled);
}

/// 场景 4: stale 节点批量标记 + 部分恢复
#[tokio::test]
async fn test_stale_node_sweep_then_recovery() {
    let (svc, node_repo) = make_service();

    // 注册 4 节点
    let mut ids = Vec::new();
    for i in 0..4 {
        let n = svc
            .register_node(
                format!("node-{}", i),
                format!("10.0.1.{}", i + 1),
                if i == 0 {
                    NodeRole::Primary
                } else {
                    NodeRole::Replica
                },
                "0.1.0".to_string(),
            )
            .await
            .unwrap();
        ids.push(n.id);
    }

    assert_eq!(svc.list_active_nodes().await.unwrap().len(), 4);

    // 人为让 2 节点 last_heartbeat 变成 120s 前
    for id in &ids[..2] {
        let mut n = node_repo.find_by_id(*id).await.unwrap().unwrap();
        n.last_heartbeat_at = chrono::Utc::now() - chrono::Duration::seconds(120);
        node_repo.save(&n).await.unwrap();
    }

    // 60s 阈值扫 → 应标记 2 个为 Unhealthy
    let marked = node_repo
        .mark_stale_unhealthy(chrono::Utc::now() - chrono::Duration::seconds(60))
        .await
        .unwrap();
    assert_eq!(marked, 2);

    // list_active_nodes 只剩 2 (其余 2 已 Unhealthy)
    let active_after_sweep = svc.list_active_nodes().await.unwrap();
    assert_eq!(active_after_sweep.len(), 2);

    // 重复 mark_stale: 已 Unhealthy 不再计数
    let marked_again = node_repo
        .mark_stale_unhealthy(chrono::Utc::now() - chrono::Duration::seconds(60))
        .await
        .unwrap();
    assert_eq!(marked_again, 0);

    // 让一个 stale 节点重新 heartbeat: 应恢复为 Healthy
    let revived = svc.heartbeat(ids[0]).await.unwrap();
    assert_eq!(revived.status, NodeStatus::Healthy);
    let active_after_recovery = svc.list_active_nodes().await.unwrap();
    assert_eq!(active_after_recovery.len(), 3);

    // 短暂等待, 验证时长测量非负 (no-op 但保证 proptest 风格)
    let _ = Duration::from_millis(1);
}

/// 场景 5: 5 业务函数集成覆盖 (per 任务书 "3+ 跨模块场景" 加项)
///   health_check / register_node / heartbeat / set_feature_flag / list_active_nodes
#[tokio::test]
async fn test_five_business_methods_integration() {
    let (svc, node_repo) = make_service();
    let admin = Uuid::new_v4();

    // 1. health_check
    assert!(svc.health_check().await.unwrap());

    // 2. register_node (5 个)
    let mut ids = Vec::new();
    for i in 0..5 {
        let n = svc
            .register_node(
                format!("h5-{}", i),
                format!("10.5.0.{}", i + 1),
                NodeRole::Replica,
                "0.1.0".to_string(),
            )
            .await
            .unwrap();
        ids.push(n.id);
    }

    // 3. heartbeat (3 次)
    for _ in 0..3 {
        svc.heartbeat(ids[0]).await.unwrap();
    }

    // 4. set_feature_flag (混合 enable/disable)
    let f1 = svc
        .set_feature_flag(
            "feature_v2".to_string(),
            FlagScope::Global,
            "*".to_string(),
            true,
            admin,
        )
        .await
        .unwrap();
    assert_eq!(f1.version, 1);
    let f2 = svc
        .set_feature_flag(
            "feature_v2".to_string(),
            FlagScope::Global,
            "*".to_string(),
            false,
            admin,
        )
        .await
        .unwrap();
    assert_eq!(f2.version, 2);

    // 5. list_active_nodes
    let active = svc.list_active_nodes().await.unwrap();
    assert_eq!(active.len(), 5);

    // 6. 清理: 全部 delete
    for id in &ids {
        let removed = node_repo.delete_by_id(*id).await.unwrap();
        assert!(removed);
    }
    let active_after_cleanup = svc.list_active_nodes().await.unwrap();
    assert!(active_after_cleanup.is_empty());
}
