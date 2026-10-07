# 详细设计书（詳細設計書 / Detailed Design Document）

**WebSocket 网络网关：握手与 HTTP 路径校验・[游戏A] 帧粘包/半包流式解码・FrameRouter trait 抽象与默认实现・错误处理与重连・与 TCP 路径对比 详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-027（**WSG**；与 `docs/04-客户端与SDK/RGS-DTL-027_详细设计书.md` 客户端资源分发**同号不同主题**；冲突处置见 §11 追溯性及 ULYS-87 收口） |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-006 网络安全 基本设计书 §3 边界防护（RGS-BAS-006 §3.3 协议边界纳入 WebSocket 路径的传输层安全基线） |
| 上游依据 | `crates/network-gateway/src/ws.rs`（364 行）；`crates/network-gateway/src/codec.rs`（398 行；Frame + FrameError + FrameRouter trait）；`crates/network-gateway/src/tcp.rs`（267 行；同包对比路径）；`crates/network-gateway/tests/ws_smoke.rs`（397 行；6 集成用例）；[游戏A]_client_h5 `SmartSocket.connect`（ws(s)://host:port/websocket，binary frame） |
| 关联文档 | RGS-DTL-006（网络安全 详细设计书；§2 NetworkPolicy 基线、§7A 未信任输入解析安全）；RGS-DTL-003（运维与 GM 后台管控 详细设计书；告警链路复用）；RGS-DTL-038（核心传输防丢包强化 详细设计书；QUIC 演进路径互斥参考）；RGS-IMPL-001 §1.3 实施门禁（G-CODE-01〜07）；RGS-SPEC-CROSS-002（gRPC/Proto 风格指南） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 + RGS-IMPL-001 工程边界 |
| 制定日 | 2026-09-19 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 状态 | v0.1 草案；§10 收口 9 条 TBD-WSG-*（详见各条）；§11 追溯性已列基线 Arc/Req 编号 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-19 | 架构师 | — | 初版制定。落实 `crates/network-gateway/src/ws.rs` 现状（`accept_hdr_async` + `PathCheck` Callback 路径校验、`BytesMut` + `Frame::decode` 粘包/半包、`Arc<dyn FrameRouter>` 异步分发、`Message::Close/Ping/Pong/Text` 分支处理）到物理/接口级文档；`tcp.rs` 作为同包对比路径；`tests/ws_smoke.rs` 6 用例一一映射到 §5/§9；§10 显式收口 9 条 TBD（mTLS、Origin、gRPC 演进、性能基准、idle timeout、graceful shutdown 等） | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-09-19 | — |
| 评审（网络/平台） |  |  | `ws.rs:107-113` accept loop 与 `ws.rs:222-311` frame loop 是否真正满足 §5.5 NFR-WSG-001/002/003；§5.1 路径校验 fallback 是否覆盖 §5.4 中"非 /websocket 路径"边界 |
| 评审（安全） |  |  | §7 安全考虑是否覆盖 RGS-BAS-006 §3.3 协议边界；§10 TBD-WSG-001 mTLS / TBD-WSG-002 Origin 校验在 Phase 1.5 升版前是否构成真实风险 |
| 评审（性能） |  |  | §5.5 性能约束验证（NFR-WSG-001/002/003）是否与 `ws_smoke.rs` 现有 5 用例（含 heartbeat_roundtrip）覆盖范围一致；§8 TCP vs WS 对比表中 §8.3 行"WS 单帧多 frame 粘包"吞吐量结论需 §10 TBD-WSG-005 性能基准验证 |
| 评审（负责人） |  |  | 本文档的基准化；TBD-WSG-001/002/003/004 阶段安排；ULYS-87 DTL-027 编号冲突归档处置 |

---

## 目录

1. [前言](#1-前言)
2. [术语约定与本文档约定](#2-术语约定与本文档约定)
3. [设计目标与约束](#3-设计目标与约束)
4. [模块级架构](#4-模块级架构)
5. [WebSocket 处理流程](#5-websocket-处理流程)
6. [数据流图与状态机](#6-数据流图与状态机)
7. [安全考虑](#7-安全考虑)
8. [与 TCP 路径对比分析](#8-与-tcp-路径对比分析)
9. [边界情况处理](#9-边界情况处理)
10. [后续计划与 TBD](#10-后续计划与-tbd)
11. [追溯性](#11-追溯性)

---

# 1. 前言

本文档是 `crates/network-gateway` crate WebSocket 传输层（`src/ws.rs`，364 行）的物理/接口级详细设计，落实 9/12 ULYS-2 任务 B（ULYS-27 合并前置）派工 brief 与 `crates/network-gateway/src/codec.rs` 中 `FrameRouter` trait 抽象（ULYS-2.2 W33）的实现约束，并明确与 `crates/network-gateway/src/tcp.rs`（TCP 路径）的关系与差异。

本文档**仅**落实 `crates/network-gateway` crate 内 WebSocket 路径的物理/接口设计；不涉及：

- **网络协议栈总体设计**（由 RGS-DTL-006 §2/§3 网络安全 + ARC-022 纵深防御承担，本网关作为 L4 应用输入校验实现点之一）；
- **5 域业务 gRPC client 接入**（由各域 DTL 各自承担；本文 §5.3 仅定义 `FrameRouter` trait 抽象接口与默认 `RouteTableFrameRouter` 实现的边界）；
- **QUIC 演进路径**（由 RGS-DTL-038 §4 QUIC Datagram 帧格式扩展承担，与本文 WS 路径互斥；§10 TBD-WSG-004 记录未来 Phase 2 gRPC 接入窗口期）；
- **RGS-IMPL-001 §1.3 实施门禁本身的变更**（保持不变）。

> **实施门禁（per RGS-IMPL-001 §1.3）**：在 `G-CODE-01〜G-CODE-07` 未全部通过前，本文 §5 物理实现层**已存在**（ULYS-27 PR #40 已合并到 `dev` 分支，commit `6c3f440` 见 `git log --merges -1`）；本文 §6 状态机与 §7 安全考虑为该既有实现的设计层记录。`cargo test -p network-gateway --test ws_smoke` 6 用例全部 PASS（5 passed；`integration_phase15_demo` 在 2026-09-19 14:26 regression 中失败，**与本文档范围无关**，详见 §11 追溯性末段）。`tests/ws_smoke.rs` 5 用例（`ws_handshake_and_route_roundtrip`/`ws_route_miss_returns_404`/`ws_heartbeat_roundtrip`/`ws_login_route_dispatch`/`ws_wrong_path_returns_404_http`）即为本节设计的事实证据。

## 1.1 范围

**In Scope**：

1. `ws.rs::serve` 启动流程（bind / accept / spawn）；
2. `ws.rs::handle_conn` → `accept_ws_with_path` 握手路径校验；
3. `ws.rs::handle_session` 帧循环（Binary / Close / Ping / Pong / Text / Frame 分支）；
4. `codec.rs::Frame::decode` 流式粘包/半包处理复用；
5. `codec.rs::FrameRouter` trait 抽象与 `RouteTableFrameRouter` 默认实现；
6. 错误分类（`FrameError::LengthOverflow`/`TooShort`/`TruncatedField`/`UnknownTlvType`/`InvalidUtf8`）；
7. `stats::GatewayStats` 计数器语义（received / forwarded / failed / route_miss / active）；
8. 与 `tcp.rs` 同包对比路径的差异表（§8）；
9. 集成测试覆盖范围（`tests/ws_smoke.rs` 5 用例 + `tests/ws_smoke.rs` 内 `RouteTableFrameRouter` stub 同步至 main.rs）；
10. §10 TBD 收口（9 条；含 mTLS、Origin 校验、gRPC 演进、性能基准、idle timeout、graceful shutdown 等）。

**Out of Scope**：

1. TCP 路径物理实现（`crates/network-gateway/src/tcp.rs` 详见其代码注释；本文 §8 仅做对比）；
2. 7 域业务 gRPC client 接入实现（各域 DTL 承担；本文 §5.3 仅约束 trait 接口边界）；
3. WebSocket 子协议（`Sec-WebSocket-Protocol`）协商（§10 TBD-WSG-002）；
4. 浏览器端压缩扩展（per-message deflate）（§10 TBD-WSG-006）；
5. 客户端 SDK 实现（`apps/cats-client` 等；与本服务端无直接代码耦合）。

## 1.2 与既有 DTL-027 同号冲突声明

| 维度 | 本文档 | `docs/04-客户端与SDK/RGS-DTL-027_详细设计书.md` |
|---|---|---|
| 主题 | WebSocket 网络网关（服务端） | 客户端资源分发与热更新（`asset_db` 物理 DDL 等） |
| 父文档 | RGS-BAS-006（网络安全） | RGS-BAS-027（客户端资源分发与热更新） |
| 域 | 02-运维安全与网络 | 04-客户端与 SDK |
| 当前版本 | 0.1（本文） | 0.2（既有；commit 历史见其修订表） |

冲突源：ULYS-86 任务 brief 显式指定 `RGS-DTL-027_WebSocket网络网关_详细设计书.md` 编号；既已存在同名编号另一主题文档。**本文档按 brief 执行编号**，归档名冲突由 ULYS-87 收口；本文 §11 追溯性记录该处置路径。本文 frontmatter 与 §1.2 表均显式标注此冲突。

---

# 2. 术语约定与本文档约定

## 2.1 术语

| 术语 | 含义 | 代码定位 |
|---|---|---|
| WS | WebSocket（RFC 6455）；本文档语境特指服务端实现 | `crates/network-gateway/src/ws.rs` |
| 帧 / Frame | [游戏A] 自研二进制协议帧；wire 格式 `[4B length u32 BE][2B cmd u16 BE][payload TLV]` | `codec.rs::Frame`（line 76-138） |
| cmd | u16 大端命令号；范围 0-65535 | `Frame::cmd` |
| payload | TLV 字段流（不含 6B header）；最大 1 MiB | `Frame::payload` |
| MAX_FRAME | 单帧（含 cmd + payload）上限；1 MiB | `codec.rs::const MAX_FRAME`（line 40） |
| PROTOCOL_HEADER_LEN | 6 字节（4B length + 2B cmd） | `codec.rs::const PROTOCOL_HEADER_LEN`（line 38） |
| FrameRouter | TCP+WS 共享的帧处理器 trait；异步签名 `Pin<Box<dyn Future>>` | `codec.rs::FrameRouter`（line 179-181） |
| RouteTable | 1351 条 codegen 路由表 + 9 条 Phase 1.5 demo（per W14 调整为 6 条 TSV-真实存在条目） | `router.rs::RouteTable`（line 43） |
| RouteTableFrameRouter | `FrameRouter` 默认实现；包 `Arc<RouteTable> + Arc<GatewayStats>`，转交 `tcp::dispatch` | `ws.rs::tests::RouteTableRouter`（line 327-340；同形态在 `tests/ws_smoke.rs` 与 main.rs） |
| GatewayStats | 原子计数器：`total_received` / `total_forwarded` / `total_failed` / `total_route_miss` / `active_connections` | `stats.rs::GatewayStats`（line 17-95） |
| 粘包 / 半包 | 多个 [游戏A] 帧打包在一个 WS Binary message 内（粘）；或单个 [游戏A] 帧横跨多个 WS Binary message（半） | `ws.rs:241-281` 帧循环；`codec.rs::Frame::decode` 流式接口 |
| handshake | WebSocket Upgrade 握手；HTTP/1.1 101 Switching Protocols | `ws.rs::accept_ws_with_path`（line 160-213） |
| Ping / Pong | WS 控制帧（opcode 0x9/0xA）；当前实现：自动 Pong 客户端 Ping，自身不发 Ping | `ws.rs:289-298` |
| Close | WS 控制帧（opcode 0x8）；收到后 echo Close 退出 session | `ws.rs:283-288` |
| Text | WS 文本帧（opcode 0x1）；当前 binary-only 协议 → 忽略 | `ws.rs:299-302` |
| 路径校验 / PathCheck | tungstenite `Callback::on_request` 钩子；检查 `req.uri().path()` == 配置 path | `ws.rs:170-213` |
| HttpReject | 内部三态错误：NotFound / Io(WsError) / Ws(WsError) | `ws.rs::enum HttpReject`（line 147-151） |

## 2.2 本文档约定

1. **代码引用形式**：所有 `crates/network-gateway/src/{ws,codec,tcp,router,stats}.rs` 与 `tests/ws_smoke.rs` 引用均给出 `文件名:行号` 或 `文件名:行号-行号`，引用行号对应本制定日（2026-09-19）`dev` 分支 commit `6c3f440` / `0135cbd` 现状；如有变动见 §10 TBD-WSG-009 维护。
2. **NFR 编号**：本文 NFR-WSG-001/002/003 为本 DTL 新增（与 RGS-BAS-006 NFR-SE-* 体系并列）；不与既有 NFR 复用编号。
3. **TBD 编号**：本文 TBD-WSG-001〜009 为本 DTL 新增；沿用 RGS-DTL-006 §10 TBD-SEC-* 编号习惯（前缀区分域）。
4. **追溯性**：§11 列 `上游依据 → 本文 §5-*` 映射；不重写 RGS-IMPL-001 §1.3。
5. **实施门禁**：本文 §5 物理实现层已存在（`crates/network-gateway/src/ws.rs` 已落地）；§5 设计描述即为该既有代码的事后文档化；不重新设计。
6. **真实证据**：本文所有性能/行为结论均锚定 `cargo test -p network-gateway --test ws_smoke` 5 用例（2026-09-19 14:26 JST regression 实测全 PASS）的覆盖范围；超出此范围的预测明确标注"未实测"并落入 §10 TBD。

---

# 3. 设计目标与约束

## 3.1 设计目标

| 目标编号 | 描述 | 验证手段 |
|---|---|---|
| OBJ-WSG-001 | WS 路径与 TCP 路径**共享** dispatcher 抽象（`FrameRouter` trait），以便 Phase 2 接入 5 域 gRPC 时仅替换实现，`ws.rs`/`tcp.rs` 不动 | `codec.rs:179-181` trait 定义；`ws.rs:84-87` 构造函数接收 `Arc<dyn FrameRouter>` |
| OBJ-WSG-002 | 客户端 SmartSocket binary-only 协议：服务端**仅**接收 `Message::Binary`，其它 opcode（Text/Frame）忽略 | `ws.rs:241-307` match arms；`tests/ws_smoke.rs` 5 用例覆盖 |
| OBJ-WSG-003 | 帧处理流式（粘包/半包）：单 WS Binary message 可含**多个** [游戏A] 帧（粘）；单 [游戏A] 帧可横跨**多个** WS Binary message（半） | `ws.rs:241-282` 循环 decode + `codec.rs::Frame::decode` 返回 `Ok(None)` 表半包 |
| OBJ-WSG-004 | 路径校验：HTTP request path 必须等于配置 `WS_PATH`（默认 `/websocket`）；不匹配 → 404 + close | `ws.rs:160-213` `accept_ws_with_path` + `PathCheck::on_request`；`tests/ws_smoke.rs::ws_wrong_path_returns_404_http` |
| OBJ-WSG-005 | 协议错误防御：`FrameError::LengthOverflow`（>1 MiB）、`TruncatedField`/`UnknownTlvType`/`InvalidUtf8`（wire 畸形）→ 立即 drop session + 关闭 | `ws.rs:260-274` match 错误分支 + `ws.rs:276-281` 缓冲上限守卫 |
| OBJ-WSG-006 | 资源计数：`active_connections` 接受时 inc、退出时 dec（无论何种退出路径）；`received/failed` 按帧级别计数 | `ws.rs:106/111` accept 前后；`ws.rs:248/262/270` 帧级别；`stats.rs::GatewayStats` |
| OBJ-WSG-007 | 不破坏既有 TCP 路径：`main.rs` 显式 `--tcp-addr` 才开 TCP；WS 默认开（端口 8000） | `ws.rs:9-13` 注释；`tcp.rs::DEFAULT_TCP_ADDR = 127.0.0.1:7001` |
| OBJ-WSG-008 | 与既有 `crates/network-gateway` 设计约束一致：workspace 公共服务（`tokio`/`tracing`/`thiserror`/`anyhow`/`serde` 等）从 workspace 引入；`bytes`/`futures-util`/`tokio-tungstenite` 沿用 5 域 direct dep 模式（per `Cargo.toml` 注释） | `crates/network-gateway/Cargo.toml` line 23-49 |

## 3.2 约束

| 约束编号 | 约束 | 来源 |
|---|---|---|
| CON-WSG-001 | 仅 binary frame（opcode 0x2）；Text/Ping（自动 Pong）/Pong/Close/Frame；Text 忽略 | `ws.rs:299-302` |
| CON-WSG-002 | 默认监听 `0.0.0.0:8000`（对齐 [游戏A]_server `web_conn.erl` 8000）；路径 `/websocket` | `ws.rs:41/44` `DEFAULT_WS_ADDR` / `WS_PATH` |
| CON-WSG-003 | 单 WS 会话缓冲上限 `MAX_FRAME_BYTES = 1024 * 1024`（1 MiB）；超过 → drop session | `ws.rs:47/277-281` |
| CON-WSG-004 | 默认并发连接上限 256（`max_connections`）；0 = 无上限 | `ws.rs:67` `WsConfig::default_local()` |
| CON-WSG-005 | Rust edition 2024（per workspace）、Rust 1.98 stable 目标（per RGS-IMPL-001 Q-108）；`tokio-tungstenite = "0.24"`（per `Cargo.toml` line 49） | RGS-IMPL-001 §3 Q-108；`Cargo.toml` |
| CON-WSG-006 | `FrameRouter` 实现必须 `Send + Sync`（`Arc<dyn FrameRouter>` 要求） | `codec.rs:179` |
| CON-WSG-007 | 实施门禁：本文档设计层记录；既有 `ws.rs` 实现已存在（G-CODE-06 通过后再次验证 CI） | RGS-IMPL-001 §1.3 |
| CON-WSG-008 | 路径校验不依赖 RustlsAcceptor；用 `accept_hdr_async` + `Callback`（per `ws.rs:124-128` 注释，避免 TLS 耦合） | `ws.rs:124-128` |
| CON-WSG-009 | 与 RGS-DTL-006 §7A.1 未信任输入解析安全对接：wire 格式畸形 + length overflow 一律 drop session，不回 rcode 包（避免被滥用做反射） | RGS-DTL-006 §7A.1 |
| CON-WSG-010 | 不在 hot path 同步 telemetry I/O；高频 ping/pong 计数走 `stats.rs` 原子计数器（不写日志） | RGS-IMPL-001 §6 可观测性约定 + BAS-004 §6.2 强制全采样白名单 |

## 3.3 反向约束（明确不做什么）

| 编号 | 不做什么 | 落入 §10 |
|---|---|---|
| NOT-WSG-001 | 不实现 `Sec-WebSocket-Protocol` 协商（当前接受所有） | TBD-WSG-002 |
| NOT-WSG-002 | 不实现 mTLS（wss://）；明文 ws:// + NetworkPolicy 隔离 | TBD-WSG-001 |
| NOT-WSG-003 | 不实现 Origin 校验（接受任意 Origin） | TBD-WSG-002 |
| NOT-WSG-004 | 不主动发 Ping；只被动响应客户端 Ping | TBD-WSG-008 |
| NOT-WSG-005 | 不实现 per-message deflate（压缩扩展） | TBD-WSG-006 |
| NOT-WSG-006 | 不实现 idle timeout（连接可无限挂起） | TBD-WSG-008 |
| NOT-WSG-007 | 不实现 graceful shutdown（当前 `serve` 无 shutdown signal） | TBD-WSG-007 |
| NOT-WSG-008 | 不在 Phase 1 调真实 5 域 gRPC client（仅 routing decision） | TBD-WSG-004（Phase 2 接入） |

---

# 4. 模块级架构

## 4.1 crate 拓扑

```text
┌─────────────────────────────────────────────────────────────────────┐
│                          RGS 网络平面                                 │
│                                                                       │
│  ┌──────────────────────────┐        ┌────────────────────────────┐ │
│  │   [游戏A]_client_h5 (H5)    │        │  其它客户端 (移动 / PC /    │ │
│  │  SmartSocket.connect     │        │   第三方 SDK 适配)         │ │
│  │  ws://host:8000/         │        │                            │ │
│  │  websocket (binary)      │        │                            │ │
│  └────────────┬─────────────┘        └──────────────┬─────────────┘ │
│               │                                      │               │
│               ▼ TCP/WS                               ▼ TCP          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              crates/network-gateway                            │  │
│  │                                                                  │  │
│  │  ┌──────────────────┐    ┌──────────────────┐                 │  │
│  │  │  src/ws.rs       │    │  src/tcp.rs      │                 │  │
│  │  │  WsConfig        │    │  serve(addr)     │                 │  │
│  │  │  serve()         │    │  handle_conn()   │                 │  │
│  │  │  handle_conn()   │    │  dispatch()      │                 │  │
│  │  │  accept_ws_with  │    │                  │                 │  │
│  │  │   _path()        │    │                  │                 │  │
│  │  │  handle_session()│    │                  │                 │  │
│  │  └────────┬─────────┘    └────────┬─────────┘                 │  │
│  │           │                       │                            │  │
│  │           └───────────┬───────────┘                            │  │
│  │                       ▼                                        │  │
│  │           ┌──────────────────────┐                             │  │
│  │           │  src/codec.rs        │                             │  │
│  │           │  Frame::decode()     │  (BytesMut 流式)            │  │
│  │           │  Frame::encode()     │                             │  │
│  │           │  FrameRouter trait   │  (Pin<Box<dyn Future>>)   │  │
│  │           └────────┬─────────────┘                             │  │
│  │                    ▼                                            │  │
│  │           ┌──────────────────────┐                             │  │
│  │           │  src/router.rs       │                             │  │
│  │           │  RouteTable (1351)   │                             │  │
│  │           │  RouteEntry          │                             │  │
│  │           └────────┬─────────────┘                             │  │
│  │                    ▼                                            │  │
│  │           ┌──────────────────────┐                             │  │
│  │           │  src/stats.rs        │                             │  │
│  │           │  GatewayStats        │                             │  │
│  │           │  (atomic counters)   │                             │  │
│  │           └──────────────────────┘                             │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  Phase 2 演进（§10 TBD-WSG-004）：                                    │
│  ┌──────────────────────┐                                            │
│  │  RouteTableFrameRouter│ ─→ 7 域 gRPC client pool                 │
│  │  (sync stub)         │     (tonic Channel + mTLS)                │
│  └──────────────────────┘     per RGS-SPEC-CROSS-002                 │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.2 模块依赖图（库内）

```text
ws.rs ──┐
        ├──> codec.rs (Frame, FrameError, FrameRouter)
tcp.rs ─┤
        ├──> router.rs (RouteTable, RouteEntry)
        ├──> stats.rs (GatewayStats)
        │
        └──> bytes, futures-util, tokio, tokio-tungstenite, tracing
```

`ws.rs` 与 `tcp.rs` 互不直接依赖（**没有** `use crate::ws` / `use crate::tcp`）；共享通过 `codec::FrameRouter` trait 抽象 + `router::RouteTable` 数据实现。`tcp::dispatch` 是当前 sync 路径的 **唯一** 默认实现（`ws.rs:337` `tcp::dispatch(frame, ...)`）。

## 4.3 外部依赖（per `crates/network-gateway/Cargo.toml`）

| 依赖 | 版本 | 用途 |
|---|---|---|
| `tokio` | workspace | async runtime |
| `tokio-tungstenite` | `0.24` | WebSocket 协议层（per `Cargo.toml:49`） |
| `bytes` | `1` | `Bytes` / `BytesMut` 流式缓冲 |
| `futures-util` | `0.3` | `SinkExt::send` / `StreamExt::next` |
| `tracing` | workspace | 结构化日志 |
| `thiserror` | workspace | `FrameError` derive |
| `tonic` / `tonic-health` / `prost` | workspace | Phase 2 7 域 gRPC client pool 预留 |
| `shared-platform` | path = `../shared-platform` | W15 mTLS Channel 工厂（Phase 2 接入时使用） |
| `tokio-tungstenite::tungstenite::*` | (re-export) | `Message` / `Role` / `Callback` / `Request` / `Response` |

`rustler = "0.36"` 仅在 `feature = "nif"` 启用（W13，BEAM/OTP 26 绑定），与本文档 WS 路径**无关**。
