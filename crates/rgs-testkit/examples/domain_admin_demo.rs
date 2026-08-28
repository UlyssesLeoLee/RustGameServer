//! `admin-service` mock + fixture 使用示例
//!
//! Run: `cargo run -p rgs-testkit --example domain_admin_demo`
//!
//! 演示:
//! - `AdminFixture::admin_action(admin, action, target)` 创建 sample admin 操作
//! - `FixtureBuilder` 链式构造带 action/target 的 admin fixture
//! - `InMemoryNatsMock` 模拟 admin.audit subject
//! - `TonicGrpcMock` 模拟 admin-service 5 个 GM endpoint
//!
//! 关联: `RGS-DTL-031_Admin域_详细设计书.md` §3 (RPC) + §4 (审计)
//!       + `RGS-BAS-003_运维与GM后台管控_基本设计书.md` §3 (字段级API)
//!       + `RGS-TST-UT-08_GM后台_单元测试设计书.md` (per f13acc6, 19/19 PASS)

use rgs_testkit::fixture::{self, FixtureBuilder};
use rgs_testkit::mock::{GrpcMock, InMemoryNatsMock, NatsMock};

#[tokio::main]
async fn main() {
    println!("=== admin-service mock + fixture 示例 ===\n");

    // 1. AdminFixture
    println!("[1] AdminFixture::admin_action(\"admin01\", \"ban\", \"player123\")");
    let a = fixture::admin_action("admin01", "ban", "player123");
    println!(
        "    admin_id={}, action={}, target_id={}\n",
        a.admin_id, a.action, a.target_id
    );

    // 2. FixtureBuilder 链式构造
    println!("[2] FixtureBuilder::new(a).with_action(\"mute\").with_target(\"player456\").build()");
    let custom = FixtureBuilder::new(a.clone())
        .with_action("mute")
        .with_target("player456")
        .build();
    println!(
        "    admin_id={}, action={}, target_id={}\n",
        custom.admin_id, custom.action, custom.target_id
    );

    // 3. InMemoryNatsMock 模拟 admin.audit
    println!("[3] InMemoryNatsMock 模拟 admin.audit subject");
    let nats = InMemoryNatsMock::new();
    nats.publish(
        "admin.audit",
        br#"{"admin":"admin01","action":"ban","target":"player123"}"#,
    )
    .await
    .unwrap();
    nats.publish(
        "admin.audit",
        br#"{"admin":"admin01","action":"mute","target":"player456"}"#,
    )
    .await
    .unwrap();
    let count = nats.received_count("admin.audit");
    println!("    admin.audit received_count={} (期望 2)\n", count);

    // 4. TonicGrpcMock 模拟 admin-service 5 个 GM endpoint (per BAS-003 §3.1-§3.4)
    println!("[4] TonicGrpcMock 模拟 5 个 GM endpoint 字段级 stub (per BAS-003 §3.1-§3.4)");
    let mut grpc = rgs_testkit::mock::TonicGrpcMock::new().await;
    grpc.expect(
        "POST",
        "/player.v1.PlayerService/Login",
        200,
        br#"{"ok":true,"token":"mock-jwt"}"#,
    );
    grpc.expect(
        "POST",
        "/admin.v1.AdminService/KickSession",
        200,
        br#"{"status":"queued","op":"kick"}"#,
    );
    grpc.expect(
        "POST",
        "/admin.v1.AdminService/SetMaintenanceMode",
        200,
        br#"{"status":"queued","op":"maintenance"}"#,
    );
    grpc.expect(
        "POST",
        "/admin.v1.AdminService/QueryHealthView",
        200,
        br#"{"service":"admin","admin_endpoint":"http://localhost","mode":"stub-ok"}"#,
    );
    grpc.expect(
        "POST",
        "/admin.v1.AdminService/QueryAuditLog",
        200,
        br#"{"items":[],"next":"stub"}"#,
    );
    println!("    TonicGrpcMock url={}\n", grpc.url());
    println!("    5 expectations registered\n");

    println!("=== 完成 ===");
}
