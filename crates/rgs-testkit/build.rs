//! rgs-testkit build.rs (per DDD Review v0.3.2 §7.3 L1.2 wave 4)
//!
//! wave 4 (per 2026-09-10 19:00 JST Ulysses 选 wave 4 真实 RPC 接入):
//! 生成 match 域 proto 客户端代码, 让 rgs-testkit 可直接 `tonic::include_proto!("r#match.v1")`
//! 拿到 `MatchServiceClient<Channel>` 走真实 RPC 调用.
//!
//! # L-CAND-016 防御
//!
//! 5 worker 公共 proto RPC 调用要同步, **match 域 worker 只加 match proto, 不改其他 4 域**.
//! `build_client=true` 拿客户端 (server 在 match-service 已经编),
//! `build_server=false` 避免重复生成 server 代码
//!
//! # proto 路径
//!
//! 从 rgs-testkit 角度看:
//! - `../match-service/proto/match/v1/match.proto` (match 域 proto)
//! - `../shared-platform/proto/common/v1/common.proto` (5 域共享 common 依赖)
//!
//! include 路径:
//! - `../match-service/proto` (match.proto 解析时找 `import "common/v1/common.proto"`)
//! - `../shared-platform/proto` (同上)

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "../match-service/proto/match/v1/match.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "../match-service/proto",
        "../shared-platform/proto",
    ];

    println!("cargo:rerun-if-changed=../match-service/proto/match/v1/match.proto");
    println!("cargo:rerun-if-changed=../shared-platform/proto/common/v1/common.proto");
    println!("cargo:rerun-if-changed=build.rs");

    tonic_build::configure()
        .build_server(false)  // rgs-testkit 仅做 client, server 在 match-service 编
        .build_client(true)   // 拿 MatchServiceClient<Channel>
        .compile_protos(protos, includes)?;

    Ok(())
}
