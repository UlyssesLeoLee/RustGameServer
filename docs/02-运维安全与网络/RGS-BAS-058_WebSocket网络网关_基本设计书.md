# 基本设计书（基本設計書 / Basic Design Document）

**WebSocket 网络网关 WebSocket Network Gateway**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-058 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-058 WebSocket 网络网关 需求定义书 v0.1（ULYS-84 派生，本批 ULYS-114 已落档）；RGS-BAS-006 网络安全 基本设计书 §3.3（ARC-022 协议边界）；RGS-IMPL-001 §1.3 实施门禁（G-CODE-01〜07） |
| 上游依据 | `crates/network-gateway/src/ws.rs`（364 行；accept_hdr_async + PathCheck + 帧循环 + Message 分支）；`crates/network-gateway/src/codec.rs`（398 行；Frame/FrameError + FrameRouter trait line 179 + CountingRouter default impl）；`crates/network-gateway/src/tcp.rs`（267 行；同包对比路径）；`crates/network-gateway/src/router.rs`（315 行；RouteTable + 1351 路由 codegen）；`crates/network-gateway/src/lib.rs`（86 行；模块导出） |
| 关联文档 | RGS-REQ-058 v0.1 需求定义书（ULYS-114）；RGS-DTL-027 WebSocket 详细设计书 v0.1（已落档，§11 追溯性 + §10 收口 9 条 TBD-WSG-*）；RGS-BAS-006 网络安全 基本设计书 §3.3；RGS-BAS-001 §3.3/§10.4 ARC-003（QUIC 双路径）；RGS-BAS-010 设计模式与核心算法总纲（trait 抽象模式）；RGS-BAS-013 背压与限流（per-frame buffer 阈值） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』基本设计工程 + RGS-IMPL-001 工程边界 |
| 制定日 | 2026-09-20 |
| 制定者 | 架构师 (Hermes Agent c557dae5 per ULYS-85) |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 状态 | v0.1 草案；§6 协议选型矩阵对齐 RGS-BAS-056 §6.1；§10 收口与 RGS-REQ-058 §10 + RGS-DTL-027 §10 9 条 TBD-WSG-* 同步 |
| 编号冲突说明 | **本文档采用 RGS-BAS-058 编号**；RGS-BAS-027 已被客户端资源分发与热更新占用，与本文不同主题。三者关系在 `docs/document-registry.toml` 注释中登记 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-20 | Hermes Agent (c557dae5) per ULYS-85 | — | 初版制定。落实 `crates/network-gateway/src/ws.rs` + `codec.rs` + `tcp.rs` + `router.rs` + `lib.rs` 现状到系统级基本设计：TCP/WS 双路径 + FrameRouter trait 抽象 + 共享 RouteTable + 模块划分 + 接口契约 + 核心时序 + 与既有架构整合。§10 收口 9 条 TBD-WSG-*（与 REQ-058 + DTL-027 同步）| 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Hermes Agent (c557dae5) | 2026-09-20 | per ULYS-85 |
| 评审（网络/平台） |  |  | §4 双路径架构（TCP `tcp::serve` + WS `ws::serve`）边界划分；§5 模块依赖（ws/codec/tcp/router/lib）是否符合 BAS-010 trait 抽象模式；§6 协议选型矩阵与 RGS-BAS-056 §6.1 不冲突 |
| 评审（安全） |  |  | §7 安全考虑是否覆盖 RGS-BAS-006 §3.3 协议边界；§10 TBD-WSG-001 mTLS / TBD-WSG-002 Origin 校验在 Phase 1.5 升版前是否构成真实风险；FrameRouter 抽象是否避免绕过 ARC-022 mTLS / NetworkPolicy |
| 评审（性能） |  |  | §8 TCP vs WS 对比表中 §8.3 行"WS 单帧多 frame 粘包"吞吐量结论是否需 §10 TBD-WSG-005 性能基准验证；NFR-WSG-001/002/003 性能约束与 RGS-BAS-013 背压阈值对齐 |
| 评审（负责人） |  |  | 本文档的基准化；TBD-WSG-001/002/003/004 阶段安排；ULYS-87 DTL-027 编号冲突归档处置 |

---

## 目录

1. [前言](#1-前言)
2. [术语约定](#2-术语约定)
3. [设计目标与约束](#3-设计目标与约束)
4. [架构总览：TCP/WS 双路径 + FrameRouter trait](#4-架构总览tcpws-双路径--framerouter-trait)
5. [模块划分](#5-模块划分)
6. [协议选型矩阵（与 RGS-BAS-056 §6.1 对齐）](#6-协议选型矩阵与-rgs-bas-056-61-对齐)
7. [与既有架构的整合](#7-与既有架构的整合)
8. [TCP vs WebSocket 对比表](#8-tcp-vs-websocket-对比表)
9. [核心时序](#9-核心时序)
10. [后续计划与 TBD](#10-后续计划与-tbd)
11. [验收标准](#11-验收标准)
12. [追溯性](#12-追溯性)

---

# 1. 前言

## 1.1 目的

RGS 系统的网络接入层由两条等价路径（TCP + WebSocket）组成，共享 `RouteTable` 与 `FrameRouter` trait 抽象（per RGS-REQ-058 §6）。本文档作为 **WebSocket 路径的基本设计层文档**，与上游需求层（RGS-REQ-058 v0.1）以及下游详细设计层（RGS-DTL-027 v0.1 已落档）形成三层对齐：

- **REQ-058** (需求层) — 业务契约 + NFR + 9 条 TBD
- **BAS-058** (本文档，基本设计层) — 系统级模块划分 + 接口契约 + 协议选型矩阵 + 核心时序 + 既有架构整合
- **DTL-027** (详细设计层) — 物理 Rust trait / 数据流伪代码 / 接口形态

本文档**不重新设计** WS 实现；落实 `crates/network-gateway/src/ws.rs` (364 行) + `codec.rs` (398 行) + `tcp.rs` (267 行) + `router.rs` (315 行) + `lib.rs` (86 行) 的现状到系统级设计文档。

## 1.2 判定原则

- 本文档**不重写** RGS-REQ-058 已确定的结构性选择
- 本文档**不替代** RGS-DTL-027 v0.1 详细设计书（§11 追溯性指向本文档 + REQ-058）
- 本文档**不引入**新 ARC；遵循既有 ARC-003（QUIC 双路径）+ ARC-022（零信任内部网络）+ ARC-013（背压与限流）
- 本文档**不替代** RGS-BAS-006 网络安全 基本设计书（§3.3 协议边界 + WebSocket 路径传输层安全基线）
- §10 TBD-WSG-* 9 条与 RGS-REQ-058 §10 + RGS-DTL-027 §10 完全同步；不在本文档独立收口

## 1.3 与既有架构的关系

| ARC | 关联 | 本文档对应 |
|---|---|---|
| ARC-003 QUIC 双路径 | WS 作为 ARC-003 辅助降级路径 | §3 设计目标（"WS = 浏览器/H5/弱网降级通道"）|
| ARC-022 零信任内部网络 | WS 复用 mTLS + NetworkPolicy 基线 | §7 安全考虑 + NFR-WSG-006/007 |
| ARC-013 背压与限流 | WS 帧 buffer 阈值由 GatewayStats + ws.rs frame loop 处理 | §5 模块划分 + §7 整合 |
| BAS-010 设计模式总纲 | FrameRouter trait 抽象遵循 BAS-010 trait 抽象模式 | §5.2 codec.rs 模块 |
| BAS-013 背压与限流 | 反压机制（per-frame buffer 累积）| §10 TBD-WSG-008 |
| BAS-056 协议选型矩阵 | §6 协议选型矩阵对齐 BAS-056 §6.1 | §6 |

---

# 2. 术语约定

| 术语 | 定义 | 出处 |
|---|---|---|
| WS | WebSocket（RFC 6455），基于 TCP 的全双工消息协议，握手走 HTTP Upgrade | REQ-058 §2 |
| Frame | WS 协议层最小数据单元（`codec.rs::Frame`）| REQ-058 §2 |
| FrameRouter | 异步路由分发 trait（`codec.rs:179`）| REQ-058 §6 + 本文档 §5.2 |
| PathCheck | 路径校验回调（`ws.rs` accept loop）| REQ-058 §4 |
| RouteTable | 1351 路由 codegen 表（`router.rs::RouteTable::new()`）| REQ-058 §6 + 本文档 §5.4 |
| GatewayStats | 网关统计指标（`stats::GatewayStats::new()`）| 本文档 §5.5 |
| CountingRouter | FrameRouter 默认测试实现（`codec.rs:197`）| REQ-058 §6 |
| RouteTableRouter | FrameRouter 业务实现（`ws.rs:332`）| REQ-058 §6 |
| 粘包/半包 | TCP 字节流多/跨 Frame 边界 | REQ-058 §5 |
| TBD-WSG-* | WebSocket 未决事项 9 条 | REQ-058 §10 + 本文档 §10 |

---

# 3. 设计目标与约束

## 3.1 设计目标

**G-WSG-001**：WS 路径必须与 TCP 路径共享 `RouteTable`（per REQ-058 §6.3 FR-WSG-009）— 双路径路由决策等价

**G-WSG-002**：WS 路径必须通过 `FrameRouter` trait（`codec.rs:179`）解耦路由分发逻辑与传输层 — 业务侧只关心 (service, method) 元组

**G-WSG-003**：WS 路径必须支持 RFC 6455 完整 Frame 处理（含 Ping/Pong/Close/Text/Binary/Continuation/Reserved）

**G-WSG-004**：WS 路径必须正确处理 TCP 字节流的粘包/半包（per REQ-058 §5.2 FR-WSG-005）

**G-WSG-005**：WS 路径必须暴露 NFR-WSG-008 metrics（per-frame / handshake / decode error / active connections）

## 3.2 设计约束

**C-WSG-001**：不引入新 ARC（per §1.2）

**C-WSG-002**：不修改现有 `crates/network-gateway/src/ws.rs` / `codec.rs` / `tcp.rs` / `router.rs` / `lib.rs` 实现（per §1.1 + REQ-058 §1.2）

**C-WSG-003**：§10 TBD-WSG-001 mTLS / TBD-WSG-002 Origin / TBD-WSG-005 性能基准 等升版项在 Phase 1.0 阶段不实现

**C-WSG-004**：WS 路径不绕过 ARC-022 mTLS / NetworkPolicy 基线（Phase 1.0 由反向代理终结 TLS）

**C-WSG-005**：WS 路径不绕过 ARC-013 背压阈值（per-frame buffer 累积阈值由 GatewayStats 暴露，触发 Close 帧策略见 §10 TBD-WSG-008）

## 3.3 与既有架构对齐

| 既有架构元素 | WS 路径集成点 |
|---|---|
| RouteTable（1351 路由 codegen）| `Arc<RouteTable>` 注入 `RouteTableRouter`（per `ws.rs:332`）|
| FrameRouter trait（`codec.rs:179`）| 业务注入点 |
| GatewayStats | `ws.rs` accept loop 与 frame loop 注入埋点 |
| tracing | `ws.rs` 现有 span：`accept_hdr_async` / frame loop / Close 帧 |
| tokio multi-worker | `Arc<dyn FrameRouter>` Send + Sync 约束 |

---

# 4. 架构总览：TCP/WS 双路径 + FrameRouter trait

## 4.1 双路径架构图

```
                  ┌─────────────────────────────────┐
                  │       RGS Network Gateway       │
                  │   (crates/network-gateway/)     │
                  └─────────────────────────────────┘
                              ▲       ▲
                              │       │
                  TCP path    │       │  WS path
                  (tcp.rs)    │       │  (ws.rs)
                              │       │
              ┌───────────────┘       └────────────────┐
              │                                          │
   ┌──────────────────────┐               ┌──────────────────────────┐
   │   tcp::serve         │               │   ws::serve (impl)       │
   │   (tcp.rs:267)       │               │   (ws.rs:107-113 accept  │
   │                      │               │    ws.rs:222-311 frame)  │
   └──────────────────────┘               └──────────────────────────┘
              │                                          │
              │  ┌─────────────────────────────────┐     │
              └──┤   codec.rs::FrameRouter trait   ├─────┘
                 │   (codec.rs:179)                 │
                 │   async fn route_frame(...)     │
                 └─────────────────────────────────┘
                              ▲
                              │ impl FrameRouter for RouteTableRouter
                              │ (ws.rs:332) + CountingRouter (codec.rs:197)
                              │
                 ┌─────────────────────────────────┐
                 │   router.rs::RouteTable::new()  │
                 │   1351 routes codegen            │
                 └─────────────────────────────────┘
                              │
                              ▼
                 ┌─────────────────────────────────┐
                 │   Business services (gRPC)      │
                 │   crates/<domain>-service/      │
                 └─────────────────────────────────┘
```

## 4.2 双路径共享边界

| 共享元素 | TCP 路径用法 | WS 路径用法 |
|---|---|---|
| `RouteTable::new()` (1351 路由) | `tcp::serve` 注入 | `RouteTableRouter` 注入 |
| `FrameRouter` trait (`codec.rs:179`) | 通过 `Arc<dyn FrameRouter>` 间接调用 | 直接持有 `Arc<dyn FrameRouter>` |
| `GatewayStats::new()` | accept/frame 埋点 | accept/frame 埋点（per NFR-WSG-008）|
| `tokio::net::TcpListener` | TCP 三次握手 | TCP 三次握手 + HTTP Upgrade |

**§4.2 边界**：TCP 与 WS 路径**仅在传输层不同**（自定义二进制 vs RFC 6455），路由层与业务层完全共享；这是双路径"等价降级"的设计核心。

## 4.3 WS 路径内部子模块

```
ws.rs (364 lines)
├── accept_hdr_async loop (ws.rs:107-113)
│   ├── TcpListener::bind
│   ├── tokio_tungstenite::accept_hdr_async
│   ├── PathCheck Callback (filter /websocket)
│   └── spawn handle_conn task
│
└── frame loop (ws.rs:222-311)
    ├── BytesMut buffer
    ├── Frame::decode (codec.rs)
    ├── Message 分支 (Close/Ping/Pong/Text/Binary)
    └── Arc<dyn FrameRouter>::route_frame dispatch
```

---

# 5. 模块划分

## 5.1 `crates/network-gateway/src/ws.rs` (364 行)

**职责**：WS accept_hdr_async 握手 + 路径校验 + 帧循环 + Message 分支处理

**关键组件**：

| 组件 | 行号范围 | 说明 |
|---|---|---|
| `accept_hdr_async` loop | 107-113 | TcpListener bind + tokio_tungstenite handshake + PathCheck 过滤 + spawn per-conn task |
| `PathCheck` Callback | (在 accept loop 内)| 严格匹配 `/websocket`，其余返 404 HTTP |
| `handle_conn` frame loop | 222-311 | BytesMut buffer + Frame::decode 状态机 + Message 分支 + FrameRouter 路由 |
| `impl FrameRouter for RouteTableRouter` | 332 | 业务路由实现，注入 `Arc<RouteTable>` |

**模块依赖**：

```
ws.rs
├── codec.rs (Frame/FrameError/FrameRouter)
├── router.rs (RouteTable::new())
├── stats.rs (GatewayStats)
└── tokio / tokio-tungstenite / bytes / futures
```

## 5.2 `crates/network-gateway/src/codec.rs` (398 行)

**职责**：WS 协议层类型定义 + FrameRouter trait 抽象 + 默认测试实现

**关键组件**：

| 组件 | 行号 | 说明 |
|---|---|---|
| `Frame` enum | (前段)| RFC 6455 §5 Frame 字段（opcode + payload + length + masking）|
| `FrameError` enum | (中段)| ProtocolError / IoError / Capacity 三类错误 |
| `pub trait FrameRouter: Send + Sync` | 179 | 异步路由分发 trait |
| `async fn route_frame(&self, frame: Frame) -> Result<RouteDecision>` | 179-196 | trait 唯一方法 |
| `impl FrameRouter for CountingRouter` | 197 | 默认测试实现（用于 ws_smoke.rs 路由统计验证）|

**模块依赖**：

```
codec.rs
├── async-trait
├── bytes (BytesMut)
└── tokio (异步 trait)
```

## 5.3 `crates/network-gateway/src/tcp.rs` (267 行)

**职责**：TCP 路径（同包对比）

**关键组件**：

| 组件 | 行号范围 | 说明 |
|---|---|---|
| `tcp::serve` | (整段)| TcpListener bind + 自定义二进制协议解析 + FrameRouter 路由 |
| 与 WS 共享 | (依赖)| `codec.rs::FrameRouter` + `router.rs::RouteTable` + `stats.rs::GatewayStats` |

**模块依赖**：同 §5.2 codec.rs + router.rs

## 5.4 `crates/network-gateway/src/router.rs` (315 行)

**职责**：1351 路由 codegen + RouteTable 类型 + 路由匹配

**关键组件**：

| 组件 | 行号范围 | 说明 |
|---|---|---|
| `RouteTable::new()` | (构造)| 加载 1351 路由（含 113 untitled placeholders per 改进路线图 v0.2 §6）|
| `build/network-gateway-*/out/generated_routes.rs` | (build script 输出)| codegen 输出（per Cargo build.rs）|
| 路由匹配 | (中段)| (code) → (service, method) 元组查询 |

**模块依赖**：

```
router.rs
├── build script (build.rs → generated_routes.rs)
└── (无运行时依赖)
```

## 5.5 `crates/network-gateway/src/lib.rs` (86 行)

**职责**：模块导出 + 公共类型 re-export

**关键导出**：

```rust
pub mod ws;
pub mod codec;
pub mod tcp;
pub mod router;
pub mod stats;

pub use codec::{Frame, FrameError, FrameRouter, RouteDecision};
pub use router::RouteTable;
pub use stats::GatewayStats;
```

## 5.6 `crates/network-gateway/src/stats.rs`

**职责**：GatewayStats 指标收集（NFR-WSG-008）

**关键指标**（per REQ-058 §9.3）：

| 指标 | 类型 | Label |
|---|---|---|
| `ws_active_connections` | Gauge | — |
| `ws_frames_in_total` | Counter | — |
| `ws_frames_out_total` | Counter | — |
| `ws_frame_decode_errors_total` | Counter | `error_type` (ProtocolError / IoError / Capacity) |
| `ws_handshake_failures_total` | Counter | `status_code` (400 / 404) |

**集成**：埋点接入 RGS-BAS-003 运维与 GM 后台管控（per `crates/network-gateway/src/stats.rs` + RGS-DTL-003 §X）

---

# 6. 协议选型矩阵（与 RGS-BAS-056 §6.1 对齐）

**§6.1 协议选型矩阵（现状汇总确认）**

| 维度 | 协议 | 实现位置 | 状态 | 性能约束 | 备注 |
|---|---|---|---|---|---|
| 主路径（PC/移动端）| QUIC Datagram/Stream | `network-gateway` ARC-003 路径 | ✅ 已实现 | NFR-PE-004 / NFR-NET-001 | RGS-REQ-001 §10.4 |
| 浏览器/H5 降级 | **WebSocket (RFC 6455)** | `crates/network-gateway/src/ws.rs` | ✅ 已实现 (ULYS-27 PR #40 commit `6c3f440`) | NFR-WSG-001/002/003 | **本文档范围** |
| 弱网降级 | TCP（自定义二进制） | `crates/network-gateway/src/tcp.rs` | ✅ 已实现 | NFR-PE-004 / NFR-NET-001 | RGS-REQ-001 §5.2 |
| 账号/支付 | TCP | `tcp.rs` + `account-service` | ✅ 已实现 | — | RGS-REQ-001 §5.2 IF-001/002 |
| GM 后台 | **WebSocket (RFC 6455)** | `ws.rs` + GM 后台 | ✅ 已实现 | NFR-WSG-001/002/003 | 实时双向推送 |
| 资源分发 | HTTPS（已独立）| `docs/04-客户端与SDK/RGS-BAS-027_*.md` | ✅ 已实现 | — | 不同主题 |
| 实时语音 | TBD | TBD | ❌ 未实现 | — | Phase 2+ |
| 位置 | TBD | TBD | ❌ 未实现 | — | Phase 2+ |

**§6.2 关键业务场景的协议细化**：

| 业务场景 | 协议选择 | 理由 |
|---|---|---|
| H5 浏览器接入 | WebSocket | 浏览器沙盒限制，QUIC 不直接可达 |
| GM 后台实时推送 | WebSocket | 双向事件流，HTTP 长轮询开销过大 |
| 移动端弱网降级 | WebSocket | 4G / 企业代理 QUIC 协商失败 |
| 战斗高频状态同步 | QUIC Datagram | 延迟敏感 + 不可靠可接受 |
| 必达事件 | QUIC Stream | 可靠传输 + 顺序保证 |
| 账号/支付 | TCP | 自定义二进制 + 强一致 RPC |

**§6.3 不引入的协议（明确否决）**：

- **WebRTC**：peer-to-peer 与 RGS C/S 架构不匹配
- **gRPC over HTTP/2**：与现有 TCP 自定义二进制路径重复，迁移成本不抵收益
- **MQTT**：IoT 场景适用，不适用于 RGS 实时游戏

**§6.4 验收口径（per RGS-REQ-058 §11 AC-WSG-001/002）**：

- `cargo test -p network-gateway --test ws_smoke` 5 用例全部 PASS（实测 2026-09-20 13:10 JST PASS）
- `cargo test -p network-gateway --test integration_phase15_demo` 5 用例全部 PASS（实测 2026-09-20 13:10 JST PASS）
- `cargo check -p network-gateway --lib -j 4` 0 错误 0 warning（除已知 `Io` dead_code warning `ws.rs:149`）

---

# 7. 与既有架构的整合

## 7.1 与 ARC-003（QUIC 双路径）的关系

**整合点**：WS 路径是 ARC-003 的**辅助降级路径**，不替代 QUIC Datagram/Stream。

**触发场景**（per REQ-058 §1.3）：

- 浏览器/H5 客户端：WS 是唯一可达路径
- QUIC 协商失败：WS 降级
- 企业代理 / 4G 弱网：WS 降级

**§7.1.1 路由等价性**：WS 与 TCP 路径共享 `RouteTable::new()`（1351 路由），业务侧只关心 (service, method) 元组，不关心底层是 WS 还是 TCP。

**§7.1.2 协议独立性**：WS 帧格式 (RFC 6455) 与 QUIC 帧格式 (RFC 9000) 独立，路由层抽象 `FrameRouter` trait 在协议层之上。

## 7.2 与 ARC-022（零信任内部网络）的关系

**整合点**：WS 路径与 TCP 路径共享同一 mTLS + NetworkPolicy 基线。

**Phase 1.0 现状**（per §10 TBD-WSG-001）：

- WS TLS 由反向代理终结（per RGS-BAS-006 §3.3）
- 不在 `ws.rs` 内做 mTLS 完整化（避免重复 ARC-022 基线）

**Phase 1.5 升版**（per §10 TBD-WSG-001）：

- `native-tls` / `rustls` 终结 WS TLS
- 与 QUIC mTLS 共享证书管理

**NFR-WSG-007 验证**：WS 帧不绕过 mTLS / ABAC 检查（per ARC-022 边界）。

## 7.3 与 ARC-013（背压与限流）的关系

**整合点**：WS 帧 buffer 累积由 `GatewayStats` + `ws.rs` frame loop 处理。

**Phase 1.0 现状**：

- 单 WS 连接 buffer 上限：64KB（per REQ-058 §9.1 NFR-WSG-002）
- 超限返 1009 Message Too Big Close

**Phase 1.5 升版**（per §10 TBD-WSG-008）：

- server 端 buffer 累积超过阈值时主动发 1008 Policy Violation Close

## 7.4 与 BAS-010（设计模式总纲）的关系

**整合点**：FrameRouter trait 抽象遵循 BAS-010 trait 抽象模式。

**trait 约束**（per `codec.rs:179`）：

- `Send + Sync`（多 worker 并发）
- `async_trait`（非阻塞 reactor）
- 唯一方法 `route_frame(&self, frame: Frame) -> Result<RouteDecision>`

**§7.4.1 默认实现 vs 业务实现**：

- 默认 `CountingRouter`（`codec.rs:197`）：用于测试 + 统计
- 业务 `RouteTableRouter`（`ws.rs:332`）：生产环境，注入 `Arc<RouteTable>`

## 7.5 与 RGS-SPEC-CROSS-002（gRPC/Proto 风格指南）的关系

**整合点**：WS 帧的 (service, method) 路由字段遵循 CROSS-002 命名约定（如 `player.v1.PlayerService.CreateCharacter`）。

**§7.5.1 codegen 对齐**：RouteTable 的 1351 路由由 build.rs codegen 生成，service/method 字段命名直接对齐 `crates/<domain>-service/proto/v1/*.proto` 定义。

---

# 8. TCP vs WebSocket 对比表

| 维度 | TCP（`tcp.rs`）| WebSocket（`ws.rs`）| 备注 |
|---|---|---|---|
| 协议层 | 自定义二进制 | RFC 6455 | 协议独立 |
| 握手 | TCP 三次握手 | TCP 三次握手 + HTTP Upgrade | §8.1 |
| 帧边界 | 自定义长度前缀 | RFC 6455 Frame（opcode + length + masking）| §8.2 |
| 客户端兼容性 | 自研 PC / 移动端 | 浏览器 / H5 / Web 客户端原生支持 | §8.3 |
| 性能开销 | 低（无 HTTP 升级）| 中（Upgrade 握手 + masking 4 字节/帧）| §10 TBD-WSG-005 |
| 反向代理穿透 | 需透明代理 | 标准 HTTP 兼容（per §10 TBD-WSG-005）| §8.4 |
| 路由决策 | 共享 RouteTable::new() | 共享 RouteTable::new() | §4.2 边界 |
| 帧解码 | 自定义 codec | RFC 6455 Frame + BytesMut | §5.1/§5.2 |
| FrameRouter | 共享 trait | 共享 trait | §5.2 |
| TLS 终结位置 | 反向代理 / QUIC 内嵌 | 反向代理（Phase 1.0）/ native-tls（Phase 1.5）| §7.2 |
| 部署友好性 | 自定义协议 + 透明代理 | 标准 HTTP 兼容 | §8.4 |

**§8.1 握手差异**：WS 路径比 TCP 多一次 HTTP Upgrade 握手，对短连接场景有性能影响（per §10 TBD-WSG-005 性能基准验证）

**§8.2 帧边界差异**：WS 帧自带 opcode + length + masking；TCP 自定义长度前缀。两者均支持粘包/半包处理（per REQ-058 §5.2 FR-WSG-005）

**§8.3 客户端兼容性差异**：浏览器/H5 只能选 WS；自研 PC/移动端可自由选择 TCP 或 QUIC

**§8.4 反向代理穿透**：WS 标准 HTTP 兼容，企业代理 / CDN / WAF 通常放行；TCP 自定义协议需透明代理配置

**§8.5 选型决策**：WS 路径是 TCP 路径的**协议兼容层**（per REQ-058 §8.1 FR-WSG-011），不替代、不分叉。性能差异由 §10 TBD-WSG-005 性能基准给出量化结论（当前待测）。

---

# 9. 核心时序

## 9.1 WS 连接建立时序

```
Client                  ws.rs (server)           codec.rs              router.rs
  │                          │                      │                      │
  │── TCP SYN ──────────────>│                      │                      │
  │<── TCP SYN+ACK ─────────│                      │                      │
  │── TCP ACK ──────────────>│                      │                      │
  │                          │                      │                      │
  │── HTTP Upgrade ─────────>│ (accept_hdr_async)   │                      │
  │   (GET /websocket HTTP/1.1)                     │                      │
  │   Upgrade: websocket    │── PathCheck Callback ─>│                     │
  │   Sec-WebSocket-Key: ...│<── /websocket ✓ ──────│                     │
  │                          │                      │                      │
  │                          │── RouteTable::new() ──────────────────────>│
  │                          │<── Arc<RouteTable> ───────────────────────│
  │                          │                      │                      │
  │                          │── FrameRouter impl RouteTableRouter        │
  │                          │   (ws.rs:332)                               │
  │                          │                      │                      │
  │<── HTTP 101 Switching ──│                      │                      │
  │   Protocols              │                      │                      │
  │   Sec-WebSocket-Accept: ...                     │                      │
  │                          │                      │                      │
  │   === WS data frame mode ===                    │                      │
  │── WS Binary Frame ─────>│ (frame loop)          │                      │
  │   (opcode=0x2)           │── Frame::decode ───>│                      │
  │                          │<── Frame {opcode, payload, length} ────────│
  │                          │── Arc<dyn FrameRouter>::route_frame ──────>│
  │                          │                      │── RouteTable lookup  │
  │                          │                      │<── RouteDecision     │
  │<── WS Binary Frame ─────│ (业务响应)            │                      │
  │   (opcode=0x2)           │                      │                      │
```

## 9.2 心跳时序（Ping/Pong）

```
Client                  ws.rs (server)
  │                          │
  │── WS Ping Frame ────────>│ (Message::Ping branch)
  │   (opcode=0x9, payload=X) │
  │                          │── 立即回 Pong（payload 原样回传）
  │<── WS Pong Frame ────────│
  │   (opcode=0xA, payload=X) │
  │                          │
  │   === idle timeout 触发 === (per §10 TBD-WSG-006)
  │<── WS Close Frame ───────│
  │   (opcode=0x8, code=1000) │
  │── TCP FIN ──────────────>│
```

## 9.3 关闭时序（Client 主动）

```
Client                  ws.rs (server)
  │                          │
  │── WS Close Frame ───────>│ (Message::Close branch)
  │   (opcode=0x8, code=1000)│
  │                          │── 发 Close 帧回 client
  │<── WS Close Frame ───────│
  │   (opcode=0x8, code=1000)│
  │── TCP FIN ──────────────>│
  │<── TCP FIN ──────────────│
```

## 9.4 错误时序（Frame 解码失败）

```
Client                  ws.rs (server)           codec.rs
  │                          │                      │
  │── WS Binary Frame ──────>│ (frame loop)          │
  │   (opcode=0x3 Reserved)  │── Frame::decode ───>│
  │                          │<── FrameError::ProtocolError ────│
  │                          │── 发 Close 帧 (1002 Protocol Error)
  │<── WS Close Frame ───────│
  │   (opcode=0x8, code=1002)│
  │── TCP FIN ──────────────>│
  │<── TCP FIN ──────────────│
```

---

# 10. 后续计划与 TBD

**§10 TBD 收口（与 RGS-REQ-058 §10 + RGS-DTL-027 §10 9 条 TBD-WSG-* 完全同步）**

| ID | 描述 | 阶段 | 负责人 |
|---|---|---|---|
| TBD-WSG-001 | WS mTLS 完整化（per RGS-BAS-006 §3.3）：Phase 1.5 升版时引入 native-tls / rustls 终结 WS TLS 而非依赖反向代理 | Phase 1.5 | 安全 + 网络 |
| TBD-WSG-002 | Origin 校验（per FR-WSG-003）：Phase 1.5 升版时引入 allowlist 配置（per tenant 或全局）| Phase 1.5 | 安全 |
| TBD-WSG-003 | gRPC 演进路径：WS 路径未来承载 gRPC-Web（per Phase 2 §9.4 改进路线图）| Phase 2 | 架构师 |
| TBD-WSG-004 | 路径前缀严格匹配 vs 包含 `/websocket/` 前缀：决定是否接受 prefix path | Phase 1.5 | 网络 |
| TBD-WSG-005 | 性能基准：量化 TCP vs WS 性能差异，覆盖高 RPS + 多 Frame/消息场景；验证 NFR-WSG-001/003 | Phase 1.5 | 性能 |
| TBD-WSG-006 | idle timeout：服务端无 read 超过 N 秒后主动 Close（推荐 60s）| Phase 1.5 | 网络 |
| TBD-WSG-007 | graceful shutdown：服务器停止时给所有活跃 WS 连接发 1001 Going Away Close 后等待 | Phase 1.5 | 运维 |
| TBD-WSG-008 | 反压机制：WS 路径在 server 端 buffer 累积超过阈值时主动发 1008 Policy Violation Close | Phase 1.5 | 网络 |
| TBD-WSG-009 | compression permessage-deflate（per RFC 7692）：是否启用、阈值配置 | Phase 2 | 架构师 |

**§10.1 阶段安排**：Phase 1.0 升版窗口 = W43-W44 (2026-10-20 ~ 2026-11-03)，Phase 2 升版窗口 = 2027-Q1（per RGS 改进路线图 v0.2 §9.4）。

---

# 11. 验收标准

**AC-WSG-101**：`cargo test -p network-gateway --test ws_smoke` 5 用例全部 PASS（实测 2026-09-20 13:10 JST PASS；用例：`ws_handshake_and_route_roundtrip` / `ws_route_miss_returns_404` / `ws_heartbeat_roundtrip` / `ws_login_route_dispatch` / `ws_wrong_path_returns_404_http`）

**AC-WSG-102**：`cargo test -p network-gateway --test integration_phase15_demo` 5 用例全部 PASS（实测 2026-09-20 13:10 JST PASS）

**AC-WSG-103**：`cargo check -p network-gateway --lib -j 4` 0 错误 0 warning（除已知 `Io` dead_code warning `ws.rs:149`）

**AC-WSG-104**：`ws.rs:107-113` accept_hdr_async + `ws.rs:222-311` frame loop 代码与 §4/§5/§9 架构图 + 核心时序一一映射

**AC-WSG-105**：§6 协议选型矩阵与 RGS-BAS-056 §6.1 不冲突（§6.1 行 WebSocket 已纳入现状矩阵）

**AC-WSG-106**：RGS-IMPL-001 §1.3 G-CODE-01〜07 实施门禁通过（per §1.3 表）

**AC-WSG-107**：`docs/01-核心架构与设计模式/RGS-REQ-058_WebSocket网络网关_需求定义书.md` v0.1（ULYS-114 已落档）§12 追溯性引本文档

**AC-WSG-108**：`docs/02-运维安全与网络/RGS-DTL-027_WebSocket网络网关_详细设计书.md` v0.1（已落档）§11 追溯性引本文档 + REQ-058

---

# 12. 追溯性

## 12.1 上游 → 本文档

| 上游 | 本文 §X |
|---|---|
| RGS-REQ-058 v0.1 需求定义书（ULYS-114）| 全部（本文档是 REQ-058 的系统级基本设计展开）|
| `crates/network-gateway/src/ws.rs` 364 行 | §4 / §5.1 / §9.1-9.4 |
| `crates/network-gateway/src/codec.rs` 398 行 + FrameRouter trait line 179 | §5.2 / §4.1 / §7.4 |
| `crates/network-gateway/src/tcp.rs` 267 行 | §5.3 / §8（对比）|
| `crates/network-gateway/src/router.rs` 315 行 | §5.4 / §4.1 / §7.1 |
| `crates/network-gateway/src/lib.rs` 86 行 | §5.5 |
| `crates/network-gateway/tests/ws_smoke.rs` 5 用例 | §11 AC-WSG-101 |
| `crates/network-gateway/tests/integration_phase15_demo.rs` 5 用例 | §11 AC-WSG-102 |
| RGS-BAS-006 网络安全 基本设计书 §3.3 | §1.3 / §7.2 / §10 TBD-WSG-001 |
| RGS-IMPL-001 §1.3 G-CODE-01〜07 | §11 AC-WSG-106 |
| RGS-REQ-001 §10.4 ARC-003 | §1.3 ARC-003 / §7.1 |
| RGS-REQ-010 §7 ARC-022 | §1.3 ARC-022 / §7.2 |
| BAS-013 背压与限流 | §1.3 ARC-013 / §7.3 |
| BAS-010 设计模式总纲 | §1.3 BAS-010 / §7.4 |
| BAS-056 协议选型矩阵 §6.1 | §6 协议选型矩阵 |
| ULYS-27 PR #40 commit `6c3f440` | §3.2（实测当前实现覆盖范围）|

## 12.2 本文档 → 下游

| 本文 §X | 下游文档 |
|---|---|
| §3 + §4 + §5 + §7 | RGS-DTL-027 v0.1（已落档，§11 追溯性回引本文档）|
| §6 协议选型矩阵 | RGS-BAS-056 §6.1（已对齐 WebSocket 行，per ULYS-88 待补全）|
| §10 TBD-WSG-001/002/003/005/006/007/008/009 | 后续 Phase 1.5 / Phase 2 升版入口 |

## 12.3 不重写

本文档不重写以下既有文档的结构性选择：

- ARC-003（QUIC 双路径，RGS-REQ-001 §10.4）— WS 作为降级路径
- ARC-022（零信任内部网络，RGS-REQ-010 §7）— WS 复用 mTLS / NetworkPolicy 基线
- ARC-013（背压与限流，RGS-BAS-013）— WS 帧反压由 `GatewayStats` + `ws.rs` frame loop 处理
- BAS-010（设计模式与核心算法总纲）— FrameRouter trait 设计遵循 BAS-010 trait 抽象模式
- BAS-013（背压与限流）— 反压阈值由 GatewayStats 暴露
- BAS-006（网络安全 基本设计书）— WS TLS 终结由反向代理 / Phase 1.5 升版
- BAS-056 §6.1 协议选型矩阵 — 本文 §6 同步对齐
- RGS-SPEC-CROSS-002（gRPC/Proto 风格指南）— WS 帧的 (service, method) 路由字段遵循 CROSS-002 命名约定

## 12.4 编号冲突归档（per ULYS-87）

本文档采用 RGS-BAS-058 编号；以下历史编号冲突已由 ULYS-87 / ULYS-85 协同处置：

- RGS-BAS-027（客户端资源分发与热更新）— 保留不动
- RGS-BAS-058（本文档，WebSocket 网络网关）— 新启用
- RGS-DTL-027（WebSocket 网络网关 详细设计书）— 已存在，本文档 §11 引用其 §11 追溯性

`docs/document-registry.toml` 注释同步登记本文档编号来源。
