//! gm-backend build.rs
//!
//! 用途:把 `proto/gm/v1/gm.proto` + `admin.proto` 编译成 Rust 代码
//! 关联: docs/00-基准与治理/RGS-TBD-08-03-S4-gm-backend-admin-gRPC-立项.md
//!       docs/00-基准与治理/RGS-S4-PHASE2-STEP1-设计.md
//!
//! 5 个 GM endpoint 协议字段 (per RGS-BAS-003 §3.1-§3.4 + RGS-DTL-003 §3.3-§3.4):
//! - HealthViewRequest/Response: services[] 5 子字段
//! - BanAccount / GrantCompensation: 既有方法
//! - SetMaintenanceRequest/Response: propagation_status 枚举
//! - QueryAuditLogRequest/Response: entries[] + has_more
//!
//! Step 1 增量 (per 2026-08-28 S4 Phase 2 step 1): admin.proto + common.proto
//! gm-backend 作为 admin-service 的 gRPC client (HealthCheck endpoint)
//! - admin.proto 位于 `../admin-service/proto/admin/v1/admin.proto`
//! - common.proto 位于 `../shared-platform/proto/common/v1/common.proto`
//! - 用 `include_path` 显式指定 proto 搜索路径(参考 admin-service build.rs)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "proto";

    let gm_proto = format!("{}/gm/v1/gm.proto", proto_dir);
    let admin_proto = "../admin-service/proto/admin/v1/admin.proto".to_string();
    let common_proto = "../shared-platform/proto/common/v1/common.proto".to_string();

    println!("cargo:rerun-if-changed={}", gm_proto);
    println!("cargo:rerun-if-changed={}", admin_proto);
    println!("cargo:rerun-if-changed={}", common_proto);
    println!("cargo:rerun-if-changed=build.rs");

    let includes: &[&str] = &[
        proto_dir,
        "../shared-platform/proto",
        "../admin-service/proto",
    ];

    tonic_build::configure()
        .build_server(false)  // gm-backend 是 client, 不需要 server trait
        .build_client(true)
        .compile_protos(
            &[gm_proto.as_str(), admin_proto.as_str(), common_proto.as_str()],
            includes,
        )?;

    Ok(())
}
