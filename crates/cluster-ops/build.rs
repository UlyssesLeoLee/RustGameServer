//! cluster-ops build.rs
//!
//! WF-1-2073: 增编 3 业务 service proto（player / economy / social）以支持
//! LCM 跨域 Saga 7 步通过 gRPC client 调用业务 service（per RGS-IMPL-PLAN-LCM-001
//! §3.6 + SPEC-DTL-042 §3 第 3 条："不直连业务 service DB；只用 gRPC client"）。

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        // 域内 proto
        "proto/cluster_ops/v1/cluster_ops.proto",
        "../shared-platform/proto/common/v1/common.proto",
        // 跨域 Saga 7 步依赖的 3 业务 service proto（per WF-1-2073）
        "../player-service/proto/player/v1/player.proto",
        "../economy-service/proto/economy/v1/economy.proto",
        "../social-service/proto/social/v1/social.proto",
    ];
    let includes: &[&str] = &[
        "proto",
        "../shared-platform/proto",
        "../cluster-ops/proto",
        "../player-service/proto",
        "../economy-service/proto",
        "../social-service/proto",
    ];
    // 强制 build script rerun 在 proto 改动时
    for p in protos {
        println!("cargo:rerun-if-changed={}", p);
    }
    println!("cargo:rerun-if-changed=build.rs");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
