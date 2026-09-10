//! rgs-testkit build.rs (per DDD Review v0.3.2 §7.3 wave 4 真实 RPC 接入)
//!
//! 用途: 为 rgs-testkit 的 bot 框架 admin 域 wave 4 真实 RPC 调用
//! 生成 admin proto client 类型 (per 5 域 ST 业务级 mTLS 实践 commit
//! `401ac5c` 证书导出 SOP + DDD Review v0.3.2 §7.3 L1.2 wave 4 准备).
//!
//! # 强约束 (per AGENTS.md §2.3 L3 跨工具链决策守门 + L-CAND-016 防御)
//!
//! - 仅生成 admin proto (per L-CAND-016 "5 worker 公共 proto RPC 调用要同步,
//!   admin 域只加 admin proto 真实 RPC, 不改其他 4 域")
//! - `build_server(false)` 仅生成 client, 不引入 server trait
//!   (rgs-testkit 不需要做 admin gRPC server)
//! - 复用 admin-service/proto/ 源 proto (单一来源, 避免 proto 漂移)
//! - 复用 shared-platform/proto/ 公共 common.proto (EntityId / HealthCheckRequest 等)
//!
//! # wave 4 真实 RPC 链路
//!
//! ```text
//! rgs-testkit::bot::gm::GmClient::issue_real(cmd)
//!   → admin_service_client::AdminServiceClient<Channel>::new(self.channel)
//!   → tokio::time::timeout(2s, client.ban_account(BanAccountRequest { ... }))
//!   → 失败 (k3s baseline 0/12 connection refused) → Ok(GmResponse { ok: false, error })
//!   → 不 panic, 不静默吞
//! ```
//!
//! # proto 路径 (相对 build.rs)
//!
//! - admin.proto: `../admin-service/proto/admin/v1/admin.proto`
//! - common.proto: `../shared-platform/proto/common/v1/common.proto`
//! - includes: `../admin-service/proto` + `../shared-platform/proto`

use std::io::Result;

fn main() -> Result<()> {
    let protos: &[&str] = &[
        "../admin-service/proto/admin/v1/admin.proto",
        "../shared-platform/proto/common/v1/common.proto",
    ];
    let includes: &[&str] = &[
        "../admin-service/proto",
        "../shared-platform/proto",
    ];
    tonic_build::configure()
        .build_server(false) // 仅生成 client (rgs-testkit 不是 admin gRPC server)
        .build_client(true)
        .compile_protos(protos, includes)?;
    Ok(())
}
