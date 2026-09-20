# 需求定义书（要件定義書 / Requirements Definition Document）

**WebSocket 网络网关 WebSocket Network Gateway**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-058 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-006 网络安全 基本设计书 §3.3（ARC-022 协议边界 + WebSocket 路径传输层安全基线）；RGS-IMPL-001 §1.3 实施门禁（G-CODE-01〜07） |
| 上游依据 | `crates/network-gateway/src/ws.rs`（364 行）；`crates/network-gateway/src/codec.rs`（398 行；Frame + FrameError + FrameRouter trait line 179）；`crates/network-gateway/src/tcp.rs`（267 行；同包对比路径）；`crates/network-gateway/src/router.rs`（315 行；RouteTable）；`crates/network-gateway/tests/ws_smoke.rs`（5 集成用例）；`crates/network-gateway/tests/integration_phase15_demo.rs`（5 集成用例） |
| 关联文档 | RGS-DTL-027 WebSocket 网络网关 详细设计书 v0.1（已落档，§11 追溯性 + §10 收口 9 条 TBD-WSG-*）；RGS-BAS-006 网络安全 基本设计书；RGS-BAS-058 WebSocket 网络网关 基本设计书（待制定，本批 ULYS-85 派生）；RGS-IMPL-001 §1.3 实施门禁；RGS-SPEC-CROSS-002（gRPC/Proto 风格指南） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』需求工程 + RGS-IMPL-001 工程边界 |
| 制定日 | 2026-09-20 |
| 制定者 | 架构师 (Hermes Agent c557dae5 per ULYS-84) |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 状态 | v0.1 草案；§10 收口 9 条 TBD-WSG-* 与下游 DTL-027 §10 同步；§11 追溯性已列基线 Arc/Req 编号 |
| 编号冲突说明 | **本文档采用 RGS-REQ-058 编号**；RGS-REQ-027 已被 App 集群自动化部署脚本占用，RGS-REQ-028 已被 反作弊/风控规则 DSL 占用，与本文不同主题。三者关系在 `docs/document-registry.toml` 注释中登记 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-20 | Hermes Agent (c557dae5) per ULYS-84 | — | 初版制定。落实 `crates/network-gateway/src/ws.rs` 现状（`accept_hdr_async` + `PathCheck` Callback 路径校验、`BytesMut` + `Frame::decode` 粘包/半包、`Arc<dyn FrameRouter>` 异步分发、`Message::Close/Ping/Pong/Text` 分支处理）到需求层文档；与下游 RGS-DTL-027 v0.1 详细设计书（已落档）形成 REQ↔DTL 双层对齐；§10 显式收口 9 条 TBD（mTLS、Origin、gRPC 演进、性能基准、idle timeout、graceful shutdown 等） | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Hermes Agent (c557dae5) | 2026-09-20 | per ULYS-84 |
| 评审（网络/平台） |  |  | `ws.rs:107-113` accept loop 与 `ws.rs:222-311` frame loop 是否真正满足 §5.5 NFR-WSG-001/002/003；§5.1 路径校验 fallback 是否覆盖 §5.4 中"非 /websocket 路径"边界 |
| 评审（安全） |  |  | §7 安全考虑是否覆盖 RGS-BAS-006 §3.3 协议边界；§10 TBD-WSG-001 mTLS / TBD-WSG-002 Origin 校验在 Phase 1.5 升版前是否构成真实风险 |
| 评审（性能） |  |  | §5.5 性能约束验证（NFR-WSG-001/002/003）是否与 `ws_smoke.rs` 现有 5 用例（含 heartbeat_roundtrip）覆盖范围一致；§8 TCP vs WS 对比表中 §8.3 行"WS 单帧多 frame 粘包"吞吐量结论需 §10 TBD-WSG-005 性能基准验证 |
| 评审（负责人） |  |  | 本文档的基准化；TBD-WSG-001/002/003/004 阶段安排；ULYS-87 DTL-027 编号冲突归档处置 |

---

## 目录

1. [前言](#1-前言)
2. [术语约定](#2-术语约定)
3. [业务需求与背景](#3-业务需求与背景)
4. [功能需求：握手与路径校验](#4-功能需求握手与路径校验)
5. [功能需求：帧编解码与粘包/半包](#5-功能需求帧编解码与粘包半包)
6. [功能需求：FrameRouter trait 与异步分发](#6-功能需求framerouter-trait-与异步分发)
7. [功能需求：消息处理（Close/Ping/Pong/Text）](#7-功能需求消息处理closepingpongtext)
8. [TCP vs WebSocket 双路径对比](#8-tcp-vs-websocket-双路径对比)
9. [非功能需求](#9-非功能需求)
10. [未决事项 TBD-WSG-*](#10-未决事项-tbd-wsg)
11. [验收标准](#11-验收标准)
12. [追溯性](#12-追溯性)

---

# 1. 前言

## 1.1 目的

RGS 系统当前网络接入层由两条等价路径组成：**TCP 路径**（`crates/network-gateway/src/tcp.rs` 267 行，`RouteTable::new()` 加载 1351 路由）与 **WebSocket 路径**（`crates/network-gateway/src/ws.rs` 364 行）。两条路径共享 `RouteTable` 与 `FrameRouter` trait 抽象（`codec.rs:179`），最终归并到同一路由决策与统计管线（`router.rs` + `stats::GatewayStats`）。

WebSocket 路径的实现代码（ULYS-27 PR #40，commit `6c3f440`）已在 `dev` 分支 `abe2eda` 上落档，但**一直没有正式的需求层文档**：上游业务方（Web/H5 客户端、跨域管理后台、移动端降级通道）对接 WS 协议时缺少契约锚点；性能基准（`ws_smoke.rs` 5 用例 + `integration_phase15_demo.rs` 5 用例）的覆盖范围未在需求层显式声明；安全边界（mTLS、Origin 校验）作为 Phase 1.5 TBD 留在代码注释里但未在需求层闭环。

本文档的**目的**是把 WebSocket 路径的现有实现固化到需求层，与下游 RGS-DTL-027 v0.1 详细设计书（已落档）形成 REQ↔DTL 双层对齐；为上游业务方提供一个可签收的需求契约；为下游实施门禁（RGS-IMPL-001 §1.3 G-CODE-01〜07）提供 WS 子系统的验收口径。

## 1.2 判定原则（先声明，避免误解）

- 本文档**不重新发明** WebSocket 协议（RFC 6455）；仅声明 RGS 系统对该协议的使用边界与约束
- 本文档**不修改**现有 `crates/network-gateway/src/ws.rs` / `codec.rs` / `tcp.rs` / `router.rs` 的实现；§10 收口 9 条 TBD-WSG-* 是后续 Phase 1.5 / Phase 2 升版的入口，本文档不直接驱动代码变更
- 本文档**不引入**新的 ARC；遵循既有 ARC-003（QUIC 双路径，RGS-REQ-001 §10.4）与 ARC-022（零信任内部网络，RGS-REQ-010 §7）边界
- 本文档**不替代** RGS-DTL-027 详细设计书（已落档 v0.1，§11 追溯性指向本文档）
- 本文档**不替代** RGS-BAS-006 网络安全 基本设计书（§3.3 协议边界 + WebSocket 路径传输层安全基线）

## 1.3 与既有架构的关系

- **ARC-003（QUIC 双路径）**：WebSocket 是 ARC-003 的 **辅助降级路径**，用于浏览器/H5 客户端在企业代理、4G 弱网、QUIC 不可用场景下的协议回退。WS 不替代 QUIC Datagram/Stream，仅在"QUIC 协商失败"或"客户端只支持 WS"的边缘场景下启用。
- **ARC-022（零信任内部网络）**：WS 路径与 TCP 路径共享同一 mTLS + NetworkPolicy 基线，但 §10 TBD-WSG-001 mTLS 完整化与 TBD-WSG-002 Origin 校验是 Phase 1.5 升版的硬约束。
- **ARC-013（背压与限流）**：WS 路径复用 `GatewayStats::new()` 指标，但 §10 TBD-WSG-005 性能基准（per-frame overhead / 心跳频率 / idle timeout）需补全。

---

# 2. 术语约定

| 术语 | 定义 |
|---|---|
| **WS** | WebSocket（RFC 6455），基于 TCP 的全双工消息协议，握手走 HTTP Upgrade |
| **Frame** | WS 协议层最小数据单元（`codec.rs::Frame`），含 opcode/payload/length |
| **FrameRouter** | 异步路由分发 trait（`codec.rs:179`），由 `RouteTableRouter`（`ws.rs:332`）实现 |
| **PathCheck** | 路径校验回调（`ws.rs` accept loop），仅放行 `/websocket` 路径，其余返 404 |
| **粘包/半包** | TCP 流式字节流的常见现象：粘包 = 多个 Frame 在一个 read buffer；半包 = 单个 Frame 跨多个 read buffer。`BytesMut` + `Frame::decode` 状态机处理 |
| **Accept HDR Async** | tokio-tungstenite 提供的异步握手 API（`ws.rs:107-113`） |
| **TBD-WSG-*** | 本文档收口的 WebSocket 未决事项 9 条（详见 §10） |

---

# 3. 业务需求与背景

## 3.1 业务场景

RGS 系统接入客户端分三类，每类的网络环境约束：

| 客户端类型 | 主路径 | 降级路径 |
|---|---|---|
| PC 客户端（Windows / macOS 原生） | QUIC（ARC-003） | TCP（fallback）|
| H5 / Web 浏览器 | **WebSocket**（受浏览器沙盒限制，QUIC 不直接可达）| — |
| 移动端（iOS / Android） | QUIC（ARC-003） | WebSocket（4G 弱网/企业代理）|
| GM 后台 / 跨域管理控制台 | **WebSocket**（双向推送实时事件）| TCP |

WS 路径在 RGS 系统的核心价值是 **H5 客户端接入 + 实时双向事件推送**。

## 3.2 当前实现覆盖范围

`crates/network-gateway/src/` 现状（2026-09-19 实测，commit `6c3f440` 合并后 `dev` HEAD `abe2eda`）：

| 文件 | 行数 | 职责 |
|---|---|---|
| `ws.rs` | 364 | WS accept_hdr_async 握手 + 路径校验 + 帧循环 + Message 分支 |
| `codec.rs` | 398 | Frame/FrameError 类型 + FrameRouter trait（line 179） + 默认实现（CountingRouter）|
| `tcp.rs` | 267 | TCP 路径（同包对比：tcp::serve vs ws::serve）|
| `router.rs` | 315 | RouteTable + 1351 路由 codegen（`build/network-gateway-*/out/generated_routes.rs`）|
| `lib.rs` | 86 | 模块导出 + 公共类型 re-export |
| `tests/ws_smoke.rs` | 397 | 5 集成用例：`ws_handshake_and_route_roundtrip` / `ws_route_miss_returns_404` / `ws_heartbeat_roundtrip` / `ws_login_route_dispatch` / `ws_wrong_path_returns_404_http` |
| `tests/integration_phase15_demo.rs` | 195 | 5 集成用例（Phase 1.5 demo 路由 roundtrip）|

**实测**（2026-09-20 13:10 JST）：

```
cargo test -p network-gateway --test ws_smoke                      → 5 passed
cargo test -p network-gateway --test integration_phase15_demo       → 5 passed
```

## 3.3 本文档未覆盖范围

- **QUIC 路径需求**：见 RGS-REQ-001 §10.4（ARC-003）；本文档不重复
- **TCP 路径需求**：见 RGS-REQ-001 §5.2 IF-001〜IF-008；本文档不重复
- **协议层消息语义**：业务层 gRPC service / method 路由决策由 `RouteTable` 处理；WS 仅作为传输层
- **客户端 SDK**：浏览器端 SmartSocket (`zsyz_client_h5`) 不在本文档范围；本文档约束 server-side 实现

---

# 4. 功能需求：握手与路径校验

## 4.1 WS Upgrade 握手

**FR-WSG-001**（per `ws.rs:107-113` accept_hdr_async 实现）：服务器必须响应 RFC 6455 §1.3 规定的 HTTP Upgrade 握手请求，关键头部：
- `Upgrade: websocket`
- `Connection: Upgrade`
- `Sec-WebSocket-Key` / `Sec-WebSocket-Accept`（RFC 6455 §1.3 算法）
- `Sec-WebSocket-Version: 13`

握手成功后状态切换到 WS 数据帧模式。失败（key/version 不匹配）返 400 Bad Request。

## 4.2 路径校验

**FR-WSG-002**（per `ws.rs` PathCheck Callback）：仅 `/websocket` 路径放行；其他路径返 404 HTTP（非 WS Close）。

边界：
- 路径严格匹配 `/websocket`，前缀匹配（如 `/websocket/`）由 §10 TBD-WSG-004 决定
- 大小写敏感（per RFC 6455 §3 对 URI 的处理）
- Query string 在 Phase 1.0 阶段忽略

**FR-WSG-003**（实测）：`tests/ws_smoke.rs::ws_wrong_path_returns_404_http` 验证非 `/websocket` 路径返 HTTP 404；`ws_handshake_and_route_roundtrip` 验证 `/websocket` 路径握手成功并进入路由决策。

## 4.3 Origin 校验（Phase 1.5 升版项）

**TBD-WSG-002**：Phase 1.5 升版时引入 Origin 头部校验（per CORS + 防止跨站 WS 劫持），配置项形式注入允许列表（per tenant 或全局白名单）。

当前 Phase 1.0 阶段不实现 Origin 校验；风险由部署层（反向代理 / WAF）兜底（per RGS-BAS-006 §3.3）。

---

# 5. 功能需求：帧编解码与粘包/半包

## 5.1 Frame 编解码

**FR-WSG-004**（per `codec.rs::Frame::decode`）：必须支持 RFC 6455 §5 规定的所有 opcode：
- `0x0` Continuation
- `0x1` Text
- `0x2` Binary
- `0x8` Close
- `0x9` Ping
- `0xA` Pong
- `0x3-0x7` Reserved（拒绝并 Close）
- `0xB-0xF` Reserved（拒绝并 Close）

Payload 长度字段必须支持 7-bit / 16-bit / 64-bit 三种格式（per RFC 6455 §5.2）；masking key 强制 4 字节 client-to-server（per RFC 6455 §5.3）。

## 5.2 粘包/半包流式解码

**FR-WSG-005**（per `ws.rs` BytesMut + Frame::decode 状态机）：必须正确处理 TCP 字节流的粘包/半包：

- **粘包**：单个 read 返回多个 Frame → decode 循环处理到 BytesMut 为空
- **半包**：单个 Frame 跨多次 read → 状态保留 + 等待后续 read 拼接
- **边界错误**：opcode 非法 / length 字段不一致 / mask 缺失 → Close 帧 + 连接终止（不静默丢弃）

实现依据：`tokio_util::codec::Framed` + `BytesMut` 状态机（`ws.rs` frame loop，`ws.rs:222-311`）。

## 5.3 错误处理

**FR-WSG-006**（per `codec.rs::FrameError`）：Frame 解码错误必须区分：
- `ProtocolError`：opcode 非法 / length 字段不一致（per RFC 6455 violation）
- `IoError`：底层 socket read/write 失败
- `Capacity`：单帧 payload 超限（per §9 NFR-WSG-002）

ProtocolError 必须发 Close 帧（status code 1002 Protocol Error）后终止连接；IoError 直接终止连接；Capacity 走限流（per RGS-BAS-013 背压）。

---

# 6. 功能需求：FrameRouter trait 与异步分发

## 6.1 FrameRouter trait 抽象

**FR-WSG-007**（per `codec.rs:179` `pub trait FrameRouter: Send + Sync`）：FrameRouter 是 WS 与 TCP 路径共享的路由分发抽象，方法签名（实测 `codec.rs:179-196`）：

```rust
#[async_trait]
pub trait FrameRouter: Send + Sync {
    async fn route_frame(&self, frame: Frame) -> Result<RouteDecision>;
    // ...
}
```

**关键约束**：
- trait 必须 Send + Sync（per WS 路径多 worker 并发）
- 必须 async（不阻塞 tokio reactor）
- 路由决策与 TCP 路径等价（共享 `RouteTable::new()` 1351 路由）

## 6.2 默认实现与业务注入

**FR-WSG-008**（per `codec.rs:197` `impl FrameRouter for CountingRouter`）：FrameRouter 必须提供默认测试实现 `CountingRouter`（用于 `ws_smoke.rs` 路由统计验证）；业务注入由 `ws.rs:332 impl FrameRouter for RouteTableRouter` 提供（用 `RouteTable::new()` 加载 1351 路由）。

`Arc<dyn FrameRouter>` 是 WS accept loop 的依赖注入点（per `ws.rs` frame loop）。

## 6.3 与 TCP 路径共享 RouteTable

**FR-WSG-009**：WS 路径与 TCP 路径**共用** `RouteTable::new()`（`crates/network-gateway/src/router.rs`），不维护独立的 WS 路由表。

设计依据：业务侧只关心 (service, method) 元组，不关心底层是 WS 还是 TCP；同一路由表避免双源真理。

---

# 7. 功能需求：消息处理（Close/Ping/Pong/Text）

**FR-WSG-010**（per `ws.rs` Message 分支）：服务器必须正确处理所有 RFC 6455 §5.5 规定的控制帧与数据帧：

| Message | 处理 |
|---|---|
| `Message::Close(code, reason)` | 发 Close 帧回 client，关闭 TCP 连接 |
| `Message::Ping(payload)` | 立即回 Pong（payload 原样回传）|
| `Message::Pong(payload)` | 忽略（用于心跳响应；不应触发业务）|
| `Message::Text(text)` | 走 FrameRouter.route_frame（业务侧按需解析）|
| `Message::Binary(data)` | 走 FrameRouter.route_frame |

**心跳**：Ping/Pong 频率由客户端决定；§10 TBD-WSG-006 决定服务端 idle timeout（推荐 60s）。

---

# 8. TCP vs WebSocket 双路径对比

| 维度 | TCP（`tcp.rs`）| WebSocket（`ws.rs`）|
|---|---|---|
| 协议层 | 自定义二进制协议 | RFC 6455 |
| 握手 | TCP 三次握手 | TCP 三次握手 + HTTP Upgrade |
| 帧边界 | 自定义长度前缀 | RFC 6455 Frame（opcode + length + masking）|
| 客户端兼容性 | 自研 PC / 移动端 | 浏览器 / H5 / Web 客户端原生支持 |
| 性能开销 | 低（无 HTTP 升级）| 中（Upgrade 握手 + masking 4 字节/帧）|
| 反向代理穿透 | 需透明代理 | 标准 HTTP 兼容（per §10 TBD-WSG-005 性能基准）|
| 路由决策 | 共享 RouteTable::new() | 共享 RouteTable::new() |
| 帧解码 | 自定义 codec | RFC 6455 Frame + BytesMut |
| FrameRouter | 共享 trait | 共享 trait |

**§8.1 行 — FR-WSG-011**：TCP 与 WS 路径在路由层完全等价（共享 `RouteTable`）；性能差异仅在传输层。

**§8.2 行 — NFR-WSG-005 决策**：WS 路径是 TCP 路径的**协议兼容层**，不替代、不分叉。性能差异由 §10 TBD-WSG-005 性能基准给出量化结论（当前待测）。

**§8.3 行 — WS 单帧多 frame 粘包**：WS 路径在业务层将多个 Frame 视为独立消息（vs TCP 路径将单次 read 视为单消息），可能导致业务层多帧拼接逻辑差异；§10 TBD-WSG-005 性能基准需覆盖"高 RPS + 多 Frame/消息"场景。

---

# 9. 非功能需求

## 9.1 性能

**NFR-WSG-001**：单 WS 连接平均帧处理延迟 p99 ≤ 5ms（per Tick 周期 50ms 预算的 10%）

**NFR-WSG-002**：单 WS 连接最大 payload ≤ 64KB（per `codec.rs::Frame` length 字段上限；超限返 1009 Message Too Big Close）

**NFR-WSG-003**：单 server 实例 WS 并发连接数 ≥ 10,000（per `tokio` 多 worker 模型；实测待 §10 TBD-WSG-005 性能基准验证）

**NFR-WSG-004**（协议层）：心跳 Ping/Pong 间隔由客户端决定（推荐 30s）；服务端 idle timeout 见 §10 TBD-WSG-006

## 9.2 安全

**NFR-WSG-006**（per RGS-BAS-006 §3.3）：WS 路径必须支持传输层 TLS（wss://）；Phase 1.0 由反向代理终结 TLS（per §10 TBD-WSG-001 mTLS 完整化）

**NFR-WSG-007**（per ARC-022）：WS 路径与 TCP 路径共享同一 NetworkPolicy 基线；WS 帧不绕过 mTLS / ABAC 检查

## 9.3 可观测性

**NFR-WSG-008**（per `stats::GatewayStats`）：必须暴露 metrics：
- `ws_active_connections`（Gauge）
- `ws_frames_in_total` / `ws_frames_out_total`（Counter）
- `ws_frame_decode_errors_total`（Counter，按 error type label）
- `ws_handshake_failures_total`（Counter，按 status code label）

埋点接入 RGS-BAS-003 运维与 GM 后台管控（per `crates/network-gateway/src/stats.rs` + RGS-DTL-003 §X）。

---

# 10. 未决事项 TBD-WSG-*

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

**本文档与 RGS-DTL-027 §10 9 条 TBD 同步**（per DTL-027 修订历史 v0.1 末段"§10 收口 9 条 TBD-WSG-* 详见各条"）。

---

# 11. 验收标准

**AC-WSG-001**：`cargo test -p network-gateway --test ws_smoke` 5 用例全部 PASS（含 `ws_handshake_and_route_roundtrip` / `ws_route_miss_returns_404` / `ws_heartbeat_roundtrip` / `ws_login_route_dispatch` / `ws_wrong_path_returns_404_http`）

**AC-WSG-002**：`cargo test -p network-gateway --test integration_phase15_demo` 5 用例全部 PASS（per Phase 1.5 demo 路由 roundtrip；2026-09-20 13:10 JST 实测 PASS）

**AC-WSG-003**：`cargo check -p network-gateway --lib -j 4` 0 错误 0 warning（除已知 `Io` dead_code warning `ws.rs:149`）

**AC-WSG-004**：`ws.rs:107-113` accept_hdr_async + `ws.rs:222-311` frame loop 代码与本文档 §4/§5/§7 FR-WSG-001/004/010 一一映射

**AC-WSG-005**：RGS-IMPL-001 §1.3 G-CODE-01〜07 实施门禁通过（per §1.3 表）

**AC-WSG-006**：`docs/02-运维安全与网络/RGS-BAS-058_WebSocket网络网关_基本设计书.md`（ULYS-85 派生）已落档 + §11 追溯性引本文档

**AC-WSG-007**：`docs/02-运维安全与网络/RGS-DTL-027_WebSocket网络网关_详细设计书.md` v0.1（已落档）§11 追溯性引本文档

---

# 12. 追溯性

## 12.1 上游 → 本文档

| 上游 | 本文 §X |
|---|---|
| `crates/network-gateway/src/ws.rs` 364 行 | §4 / §5 / §7 |
| `crates/network-gateway/src/codec.rs` 398 行 + `FrameRouter` trait line 179 | §6 |
| `crates/network-gateway/src/tcp.rs` 267 行 | §8（对比）|
| `crates/network-gateway/tests/ws_smoke.rs` 5 用例 | §11 AC-WSG-001 |
| `crates/network-gateway/tests/integration_phase15_demo.rs` 5 用例 | §11 AC-WSG-002 |
| RGS-BAS-006 网络安全 基本设计书 §3.3 | §1.3 / §9.2 NFR-WSG-006/007 |
| RGS-IMPL-001 §1.3 G-CODE-01〜07 | §11 AC-WSG-005 |
| RGS-REQ-001 §10.4 ARC-003（QUIC 双路径）| §1.3 ARC-003 |
| RGS-REQ-010 §7 ARC-022（零信任内部网络）| §1.3 ARC-022 / §9.2 NFR-WSG-007 |
| ULYS-27 PR #40 commit `6c3f440` | §3.2（实测当前实现覆盖范围）|

## 12.2 本文档 → 下游

| 本文 §X | 下游文档 |
|---|---|
| §1.3 + §3 + §4 + §5 + §6 + §7 | RGS-DTL-027 v0.1（已落档，§11 追溯性回引本文档）|
| §1.3 + §3 + §4 + §5 + §6 + §7 + §9 | RGS-BAS-058 WebSocket网络网关基本设计书（待制定，ULYS-85 派生）|
| §9 NFR-WSG-001/002/003 + §10 TBD-WSG-005 | 性能基准实施（Phase 1.5 升版入口）|

## 12.3 不重写

本文档不重写以下既有文档的结构性选择：
- ARC-003（QUIC 双路径，RGS-REQ-001 §10.4）— WS 作为降级路径
- ARC-022（零信任内部网络，RGS-REQ-010 §7）— WS 复用 mTLS / NetworkPolicy 基线
- ARC-013（背压与限流，RGS-BAS-013）— WS 帧反压由 `GatewayStats` + `ws.rs` frame loop 处理
- BAS-010（设计模式与核心算法总纲）— FrameRouter trait 设计遵循 BAS-010 trait 抽象模式
- RGS-SPEC-CROSS-002（gRPC/Proto 风格指南）— WS 帧的 (service, method) 路由字段遵循 CROSS-002 命名约定

## 12.4 编号冲突归档（per ULYS-87）

本文档采用 RGS-REQ-058 编号；以下历史编号冲突已由 ULYS-87 / ULYS-84 协同处置：
- RGS-REQ-027（App 集群自动化部署脚本）— 保留不动
- RGS-REQ-028（反作弊 / 风控规则 DSL）— 保留不动
- RGS-REQ-058（本文档，WebSocket 网络网关）— 新启用

`docs/document-registry.toml` 注释同步登记本文档编号来源。
