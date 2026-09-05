//! network-gateway —— Phase 1 协议网关 骨架 (per 9/4 改进路线图.md Phase 1)
//!
//! ## 范围 (W6 0.5 SRE·d 骨架)
//! 1. TCP 二进制网关 (4 字节长度 + payload 解码) — `tcp.rs`
//! 2. 协议号 → gRPC method 路由表骨架 — `router.rs`
//! 3. EPMD 协议 stub (端口 4369, 简单 NodeInfo 响应) — `epmd.rs`
//! 4. Erlang 分布式协议 stub (net_kernel 模拟) — `dist.rs`
//! 5. 闪烁之光 自研二进制编解码 stub — `codec.rs`
//! 6. Admin gRPC 服务 (HealthCheck/ListRoutes/RegisterRoute/GetStats) — `server.rs`
//!
//! ## 真实演示 (per task brief)
//! 收 4 字节协议码 (10101 = 创建角色, per 9/4 API 清单-全量提取-2026-09-04.tsv)
//! → 解析 → 路由到 player-service.CreateCharacter
//! → TCP socket 监听 127.0.0.1:7001, 接 [4B code][4B length][payload]
//! → 调 player-service gRPC client, 返回响应字节流.
//!
//! ## W7 扩展 (Phase 1.5 stub, 30 min 不可能完整 8 SRE·d)
//! - Cookie 鉴权 stub (`cookie.rs`)
//! - NIF 桥接 stub (`nif.rs`, 7 域 GrpcTarget + bridge function)
//! - NIF 桥接 demo (`nif_demo.rs`, W13 PoC rustler 0.36 选型验证)
//! - web_conn stub (`web_conn.rs`, port 8000 HTTP 入口)
//! - Zone 启动 stub (`zone.rs`, center/zone 拓扑)
//! - 8 域 demo 路由 (per 9/4 改进路线图 Phase 2)
//!
//! ## 不动
//! - 5 域 / batch / 平台 / 工具 crate 全部不动 (per task brief)
//! - 7 域独立 Lead 原则 (per 8/21 JST 5 域拒绝兼任 → 扩 7 域)
//! - Erlang cookie / 任何 secret 不打印 (per 8/27 11:06 JST hard ban)
//!
//! ## W14 1351 路由 codegen (Phase 1 关键路径)
//! - build.rs 从 `data/api_routes_2026-09-04.tsv` codegen `GENERATED_ROUTES` (1351 条)
//! - RouteTable::new() 加载全部 1351 条 (per 9/4 API 清单-全量提取-2026-09-04.tsv)
//! - 9 demo 路由覆写: 10101/10201/10202/20001/20002/20101/30101/40101/50101
//!   (其余 1342 条用 Method_<code> 默认占位, Phase 2 接 7 域真实 .proto)
//!
//! ## W15 7 域 gRPC 客户端池 (Phase 2 起步)
//! - `client_pool.rs`: 6 域 mTLS Channel (player/economy/match/social/admin/cluster_ops) +
//!   3 NEW 域占位 (scene/battle/batch NotDeployed)
//! - 复用 shared-platform::build_secure_channel_with_tls (per W10 5 域 mTLS 验证)
//! - 7 域分配: player 200 / economy 90 / scene 148 / battle 250 / batch 30 / admin 100 /
//!   cluster_ops 33 + placeholder 500 (per 9/4 改进路线图 §1 Phase 2)
//!
//! ## 已知缺口 (per 9/4 改进路线图 Phase 1 完整 8 SRE·d, 本骨架仅 0.5)
//! - 完整 EPMD 4 字节 header + name+port+node_type 实现 (本骨架仅 NodeInfo 响应)
//! - dist_proto (net_kernel 兼容) — Phase 1.5 推进
//! - 1342 条默认 Method_<code> 占位 → Phase 2 接 7 域真实 .proto 后替换
//! - PHP↔Erlang cookie 鉴权兼容 — Phase 1.5 stub 推进
//! - TCP 粘包 / 压缩 / 加密 (per R3 风险) — Phase 2
//! - rustler 0.36 + BEAM 内嵌 (per ADR-006 Option A + ADR-007) — Phase 1.5 推进
//! - 3 NEW 域 (scene/battle/batch) proto 待 W4/W5 5 worktree merge 后实装

#![allow(clippy::result_large_err)]
#![allow(clippy::doc_overindented_list_items, clippy::doc_lazy_continuation)]

pub mod client_pool;
pub mod codec;
pub mod cookie;
pub mod dist;
pub mod epmd;
pub mod nif;
pub mod nif_demo;
pub mod proto;
pub mod router;
pub mod server;
pub mod stats;
pub mod tcp;
pub mod web_conn;
pub mod zone;

pub use client_pool::{ClientPoolError, GrpcClientPool, SharedClientPool};
pub use codec::{Frame, FrameError, PROTOCOL_HEADER_LEN};
pub use cookie::{verify_cookie, validate_cookie, CookieError, MAX_COOKIE_LEN};
pub use nif::{bridge as nif_bridge, BridgeResult, GrpcTarget};
pub use nif_demo::{add as nif_add, bridge_route as nif_demo_route, echo as nif_echo, version as nif_version};
pub use router::{RouteEntry, RouteTable, GENERATED_ROUTES};
pub use stats::GatewayStats;
pub use web_conn::{parse_http_path, WebConnConfig, WEB_CONN_PORT};
pub use zone::{ZoneConfig, ZoneRole};
