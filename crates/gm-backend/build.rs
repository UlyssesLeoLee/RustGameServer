//! gm-backend build.rs
//!
//! 用途:把 `proto/gm/v1/gm.proto` 编译成 Rust 代码(per S4 Phase 1)
//! 关联: docs/00-基准与治理/RGS-TBD-08-03-S4-gm-backend-admin-gRPC-立项.md
//!
//! 5 个 GM endpoint 协议字段 (per RGS-BAS-003 §3.1-§3.4 + RGS-DTL-003 §3.3-§3.4):
//! - HealthViewRequest/Response: services[] 5 子字段
//! - BanAccount / GrantCompensation: 既有方法
//! - SetMaintenanceRequest/Response: propagation_status 枚举
//! - QueryAuditLogRequest/Response: entries[] + has_more

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = "proto";

    // 编译 gm.proto (不依赖 common.proto, v0.3 简化)
    let gm_proto = format!("{}/gm/v1/gm.proto", proto_dir);
    println!("cargo:rerun-if-changed={}", gm_proto);
    println!("cargo:rerun-if-changed=build.rs");

    tonic_build::configure()
        .build_server(false)  // gm-backend 是 client, 不需要 server trait
        .build_client(true)
        .compile_protos(&[gm_proto.as_str()], &[proto_dir])?;

    Ok(())
}
