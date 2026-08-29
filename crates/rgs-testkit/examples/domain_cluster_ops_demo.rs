//! `cluster-ops` (跨域编排) mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_cluster_ops_demo`
//!
//! 演示:
//! - `realm_lifecycle` 6 阶段状态机 fixture (per RGS-DTL-042 §4)
//! - `pfau` 7 阶段 feature registry (per RGS-IMPL-001 §3)
//! - `InMemoryNatsMock` 模拟 cluster.events subject
//! - `TonicGrpcMock` 模拟 5 域 admin RPC
//!
//! 关联: `RGS-DTL-042_ClusterOps_详细设计书.md` + `RGS-ARC-051` (ClusterOps 域)

use rgs_testkit::mock::{GrpcMock, InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== cluster-ops mock + fixture 示例 ===\n");

    // 1. 6 阶段状态机 enum 引用 (per DTL-042 §4)
    println!("[1] realm_lifecycle 6 阶段 (NewRealm / Scale / Split / Merge / Retire / Archive)");
    println!("    per RGS-DTL-042 §4 + SPEC-DTL-042 §3 §6 步约束");
    println!(
        "    测试代码位置: crates/cluster-ops/src/realm_lifecycle/tests/ut_state_machine.rs\n"
    );

    // 2. PFAU 7 阶段 feature registry
    println!("[2] PFAU 7 阶段 feature registry (per RGS-IMPL-001 §3 + ARC-051)");
    println!("    阶段: Declared / Active / UpgradePending / Canary / Paused / Retired / Archived");
    println!("    测试代码位置: crates/cluster-ops/tests-disabled/ut_feature_adapter.rs (待迁回 tests/)\n");

    // 3. InMemoryNatsMock 模拟 cluster.events
    println!("[3] InMemoryNatsMock 模拟 cluster.events subject");
    let nats = InMemoryNatsMock::new();
    nats.publish(
        "cluster.events",
        br#"{"event":"realm_created","realm_id":"r1","stage":"NewRealm"}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "cluster.events",
        br#"{"event":"realm_scaled","realm_id":"r1","stage":"Scale","nodes":5}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "cluster.events",
        br#"{"event":"realm_split","realm_id":"r1","stage":"Split","into":["r1","r2"]}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "cluster.events",
        br#"{"event":"realm_merged","realm_id":"r1+r2","stage":"Merge"}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "cluster.events",
        br#"{"event":"realm_retired","realm_id":"r1","stage":"Retire"}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "cluster.events",
        br#"{"event":"realm_archived","realm_id":"r1","stage":"Archive"}"#,
    )
    .await
    .unwrap();
    let count = nats.received_count("cluster.events");
    println!("    cluster.events received_count={} (期望 6)\n", count);

    // 4. TonicGrpcMock 模拟 5 域 admin RPC
    println!("[4] TonicGrpcMock 模拟 5 域 admin RPC (per BAS-003 §3)");
    let mut grpc = rgs_testkit::mock::TonicGrpcMock::new().await;
    grpc.expect(
        "POST",
        "/player.v1.PlayerService/Login",
        200,
        br#"{"ok":true}"#,
    );
    grpc.expect(
        "POST",
        "/economy.v1.EconomyService/Transfer",
        200,
        br#"{"ok":true,"tx_id":"t1"}"#,
    );
    grpc.expect(
        "POST",
        "/match.v1.MatchService/CreateRoom",
        200,
        br#"{"ok":true,"room_id":"rm1"}"#,
    );
    grpc.expect(
        "POST",
        "/social.v1.SocialService/SendMessage",
        200,
        br#"{"ok":true,"msg_id":"m1"}"#,
    );
    grpc.expect(
        "POST",
        "/admin.v1.AdminService/QueryHealthView",
        200,
        br#"{"ok":true}"#,
    );
    println!("    TonicGrpcMock url={}\n", grpc.url());

    println!("=== 完成 ===");
}
