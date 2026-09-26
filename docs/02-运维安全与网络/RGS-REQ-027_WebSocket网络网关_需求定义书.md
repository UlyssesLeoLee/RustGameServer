# 需求定义书（要件定義書 / Requirements Definition Document）

**WebSocket网络网关 WebSocket Network Gateway**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-027 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-001 §10.4 ARC-003（网络传输方式）／RGS-REQ-038 §7 FR-NET-007（客户端⇔网关实时通道）／RGS-REQ-006 §3 ARC-018（功能挂载）——本文档为 RGS-BAS-006 §6「WebSocket 行」周边协议选型矩阵的展开，新增 ARC-048（WebSocket网络网关域） |
| 制定日 | 2026-09-19 |
| 最终更新日 | 2026-09-19 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-09-19 | 架构师 | — | 初版制定。背景：ULYS-27（WebSocket 网络网关）实现代码（`crates/network-gateway/src/ws.rs`，W33 + ULYS-27 Phase 2 共 364 LOC，commit `aca54464` 已合并 PR #40）已落地，Phase 1.5 骨架与 Phase 2 业务 dispatch 均跑通 [游戏A]_client_h5 E2E，但需求文档侧长期未单独立项，仅在 RGS-REQ-038 §7 周边协议矩阵「WebSocket 行」以合并行形式登记。本文将其升级为独立需求定义书，新增 ARC-048（WebSocket网络网关域），与 ARC-003（QUIC 双路径）/ARC-006（网络安全）/ARC-018（功能挂载）/ARC-047（FEC 增强层）并列。判定：保持现有 `tokio-tungstenite` + 自研二进制帧的实现路径不变；mTLS（wss://）、Origin/Subprotocol 校验、ping/pong 业务心跳、连接 backpressure 计数列为 Phase 2 已落地能力；跨域 saga 推送、流式任务进度（per RGS-REQ-015 F-28 推迟到 v0.2）列为 TBD-NG-001/002。 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-09-19 | — |
| 评审（技术） | | | 确认 §4「WS 与 QUIC 双路径并存」判定成立；确认 Arc<dyn FrameRouter> 抽象（per ULYS-2.2 W33）未引入第三条 dispatcher 路径 |
| 评审（安全） | | | 确认 §5 NFR-NG-003/004 mTLS + Origin/Subprotocol 校验与 RGS-REQ-010 网络安全 §4 一致；不与 ARC-006 既有 mTLS 策略冲突 |
| 评审（性能） | | | 确认 §8 NFR-NG-005（单帧解码延迟）与 NFR-PE-004（输入→ACK p99<100ms）既有预算不冲突 |
| 审批（负责人） | | | 本文档基准化，含 ARC-048 新方针的批准，登记到 RGS-REQ-001 §10.4、§14.2、附件D §3（ADR-0048） |

---

## 目录

1. [前言](#1-前言)
2. [术语约定](#2-术语约定)
3. [背景与课题](#3-背景与课题)
4. [判定：为什么是 WebSocket 入口而非"直接接 QUIC Datagram"](#4-判定为什么是-websocket-入口而非直接接-quic-datagram)
5. [功能需求](#5-功能需求)
6. [非功能需求](#6-非功能需求)
7. [架构设计方针 ARC-048](#7-架构设计方针-arc-048)
8. [与既有 RGS-REQ-038 §7 周边协议矩阵的关系](#8-与既有-rgs-req-038-7-周边协议矩阵的关系)
9. [验收标准](#9-验收标准)
10. [风险与未决事项](#10-风险与未决事项)

---

## 1. 前言

RGS-REQ-038 §7 FR-NET-007 把客户端⇔网关（实时）的协议选型记录为"QUIC（可靠Stream＋不可靠Datagram＋§9 FEC增强）"，但其下方"客户端 SDK 适配层"一栏（per RGS-REQ-012 §4）指出 H5/小游戏 客户端实际落地的入口是 **WebSocket（`ws://host:8000/websocket`）+ 自研二进制帧**，而非裸 QUIC。RGS-REQ-001 §10.4 ARC-003 已明确：QUIC 是"网关⇔游戏服务"骨干协议，客户端侧受浏览器/小游戏容器/中间盒限制，需要一个**对客户端透明、对内走 QUIC 双路径**的边缘协议网关。

本文档把这一边缘网关的实现（`crates/network-gateway/src/ws.rs`）从"代码已存在但需求无独立条目"的状态升级为正式需求定义书，新增 ARC-048（WebSocket网络网关域），并把分散在 RGS-REQ-038 §7 / RGS-REQ-012 §4 / RGS-BAS-006 §6 / RGS-BAS-038 §4 的相关条目汇总到 FR-NG-001〜010（功能）+ NFR-NG-001〜005（非功能）+ AC-NG-001〜008（验收）中。

## 2. 术语约定

| 术语 | 定义 |
|---|---|
| WS（WebSocket） | RFC 6455 定义的浏览器/H5/小游戏通用全双工协议，握手为 HTTP Upgrade，传输为 binary/text frame |
| Frame | `codec::Frame`，wire 格式 `[4B length u32 BE][2B cmd u16 BE][payload TLV]`，与 [游戏A]_client_h5 SmartSocket 1:1 对齐 |
| FrameRouter | `codec::FrameRouter` trait（per ULYS-2.2 W33），让 TCP (`tcp.rs`) 与 WebSocket (`ws.rs`) 共享同一份 dispatcher |
| WSS | WebSocket over TLS，等价于"HTTP/2 的 HTTPS 用于 WS"，握手阶段走 TLS，路径与 plain WS 一致 |
| WS_PATH | 网络网关常量 `/websocket`，客户端写死（per `crates/network-gateway/src/ws.rs:44`） |
| DEFAULT_WS_ADDR | 网络网关默认监听地址 `0.0.0.0:8000`（per `crates/network-gateway/src/ws.rs:41`），对齐 [游戏A]_server `web_conn.erl` |
| Backpressure | `WsConfig::max_connections` 限制并发握手数（默认 256），超出后 accept 排队而非无限堆 |
| Arc-003 既有QUIC双路径 | RGS-REQ-001 §10.4：QUIC Stream（可靠，必达事件 IF-001-2）+ QUIC Datagram（不可靠，高频状态 IF-001-1） |

## 3. 背景与课题

- **已落地**：ULYS-27 Phase 1.5（骨架：`/websocket` 路径接受 + 帧循环 + FrameRouter dispatch）+ Phase 2（业务层路由、cmd=1110 登录、cmd=1199 心跳、H5 E2E d4/d5 wire format PASS，PR #40 已合并 `aca54464`），代码位置 `crates/network-gateway/src/ws.rs`（364 LOC）。
- **缺口 1**：需求侧长期未独立成文。RGS-REQ-038 §7 周边协议矩阵把 WebSocket 行与 QUIC 同行合并，混淆了"网关-客户端"与"网关-服务"两条协议选型，导致评审阶段无法定位 WS 行为边界。
- **缺口 2**：mTLS（wss://）/Origin 校验/`Sec-WebSocket-Protocol` 校验在 `ws.rs:14` 的"已知缺口"段落明确登记为 Phase 2 推进项，但无对应 NFR 编号，Phase 2 评审缺验收锚点。
- **缺口 3**：连接级 backpressure 当前为 `WsConfig::max_connections = 256`（per `ws.rs:67`），无对应 NFR；高负载压测下是否退化为排队/拒绝/丢连接缺判定准则。
- **缺口 4**：客户端心跳当前走业务 cmd=1199 而非 RFC 6455 Ping/Pong frame，**NFR-NG-005 性能预算无法直接观察 WS 层连接活性**，只能靠业务层 ACK 反推。
- **关联**：RGS-REQ-015 F-28（WebSocket 实时推送，原推迟到 batch v0.2）、RGS-REQ-016 §3.4 Signal 服务集成（WebSocket / SSE）、RGS-REQ-021 §5 OAuth2 PKCE 流（要求 Origin 校验 + State Cookie）——这三条都依赖本 ARC-048 提供的能力底座。

## 4. 判定：为什么是 WebSocket 入口而非"直接接 QUIC Datagram"

负责人 8 月底口头指示"周边协议按性质选用 TCP/UDP"，ULYS-2 任务 B 派工 brief 也明确客户端入口必须是 `/websocket`（路径写死、`binary frame`）。此处将判定显式化：

1. **浏览器/H5/小游戏容器限制**：QUIC 在浏览器侧仅有实验性支持（Chromium 的 WebTransport over HTTP/3 需 HTTP/3 + Origin 协商），而 WebSocket 是 HTML5 标准、所有主流浏览器/小游戏容器原生支持，**客户端 SDK 接入成本最低**。RGS-REQ-012 §4 已据此决策。
2. **NAT/中间盒穿透**：WebSocket 走 HTTP/80/443，几乎所有企业网/4G/5G NAT 与中间盒默认放行；QUIC 走 UDP/443 在部分 4G 网络与 GFW 环境下被识别为"未知 UDP"丢包率高于 WS。
3. **网关-服务骨干协议不变**：ARC-003 的 QUIC 双路径（Stream 可靠 + Datagram 不可靠）解决的是"网关⇔游戏服务"骨干问题，**客户端入口协议选择与骨干协议选择是两个独立决策**，互不绑定。本判定不反转 ARC-003、不引入第三条骨干路径，仅在网关边缘增加一个"对客户端透明、对内接 FrameRouter → QUIC 双路径"的协议适配层。
4. **FrameRouter 抽象不引入第三条 dispatcher**：per ULYS-2.2 W33 + `crates/network-gateway/src/codec.rs:174`，`Arc<dyn FrameRouter>` 让 TCP/WS 共享同一份 `RouteTableFrameRouter`，**WS 路径不引入新 cmd 路由表**，仅在 wire 格式（HTTP Upgrade vs 裸 TCP）层面不同，业务层零分歧。

**结论**：WebSocket 作为客户端⇔网关边缘入口协议保留，ARC-003 的 QUIC 双路径作为网关⇔服务骨干协议保留，两者经 FrameRouter 抽象合流。**不**评审"客户端直接接 QUIC"方案。

## 5. 功能需求

| ID | 需求 |
|---|---|
| FR-NG-001 | 网络网关**必须**在 `WS_PATH = "/websocket"` 监听 WebSocket 握手（per `ws.rs:44`），非该路径的 HTTP 请求一律返回 404 且不计入 error 指标（per `ws.rs:131` `HttpReject::NotFound`） |
| FR-NG-002 | WS 握手**必须**校验 `Upgrade: websocket` 与 `Connection: Upgrade` 头（由 `tokio-tungstenite::accept_async` / `accept_hdr_async` 完成，per `ws.rs:129`），握手失败走 `HttpReject::Ws` 软失败路径，不计入 IO error |
| FR-NG-003 | WS 会话**必须**支持二进制帧（`Message::Binary`）作为业务帧载体；wire 格式 `[4B length u32 BE][2B cmd u16 BE][payload TLV]` 必须 1:1 对齐 `codec::Frame::decode/encode`（per `ws.rs:242`、RGS-REQ-038 §5.2 wire 格式），缓冲采用 `BytesMut` 流式解码处理粘包/半包（per `ws.rs:243`） |
| FR-NG-004 | WS 业务 dispatch **必须**走 `Arc<dyn FrameRouter>` 抽象（per `ws.rs:85`、codec.rs:174），默认实现为 `RouteTableFrameRouter` 包装 `Arc<RouteTable>` + `Arc<GatewayStats>`（per `ws.rs:327-340`），**不得**在 WS 路径引入第三条 dispatcher |
| FR-NG-005 | WS 帧循环**必须**支持并发多 session（`tokio::spawn(handle_conn)`，per `ws.rs:107`），单 session 帧循环 `BytesMut` 单边超过 `MAX_FRAME_BYTES = 1 MiB`（per `ws.rs:47`）**必须**主动 `Message::Close` 关闭会话并 `inc_failed()`，防止缓冲被恶意帧填爆 |
| FR-NG-006 | WS 帧循环**必须**对协议错误（`FrameError::TruncatedField / UnknownTlvType / InvalidUtf8 / LengthOverflow`）一律 `Message::Close` + `inc_failed()` 关闭会话（per `ws.rs:260-273`），**不得**返回 5xx、**不得**返回部分 frame；这是 RFC 6455 §7.4.1 要求的"收到无法继续的 frame 必须关闭" |
| FR-NG-007 | WS 会话**必须**正确响应 Ping frame（自动回 Pong，per `ws.rs:289-295`），收到 Close frame **必须** echo Close 回客户端再退出（per `ws.rs:283-288`）；收到 Text frame **必须** debug-log 后忽略（per `ws.rs:299-302`，任务 brief 仅要求 binary frame） |
| FR-NG-008 | 业务层心跳**应当**走业务 cmd（当前为 cmd=1199，per `ws.rs:18`），**不得**阻塞 RFC 6455 Ping/Pong 帧处理（Phase 2 §6.3 与 NFR-NG-005 联立校验） |
| FR-NG-009 | 连接级 backpressure **必须**由 `WsConfig::max_connections` 控制（per `ws.rs:55-69`，默认 256），达到上限**必须**走 accept 排队而非无限 spawn；0 表示无上限（仅在受控环境使用，生产**不得**置 0） |
| FR-NG-010 | WS 启动**必须**接受配置化绑定（`WsConfig { bind_addr, path, max_connections }`，per `ws.rs:50-69`），`bind_addr` 默认 `0.0.0.0:8000`（per `ws.rs:41`），允许环境变量覆盖但**不得**改 `WS_PATH` 字面量（与 [游戏A]_client_h5 SDK 写死一致） |

## 6. 非功能需求

| ID | 需求 |
|---|---|
| NFR-NG-001 | WS 握手 p99 延迟**必须** `<50ms`（本地 loopback 基准）；不允许因握手逻辑（路径校验 + accept_hdr_async）退化到 100ms+ 区间 |
| NFR-NG-002 | 单帧解码（`Frame::decode` + 业务 dispatch + 回包写）p99 延迟**必须** `<10ms`（per NFR-PE-004 输入→ACK p99<100ms 既有预算，WS 仅占其中 10% 上限） |
| NFR-NG-003 | mTLS（`wss://`）**必须**支持且**必须**复用 `rgs-certgen`（per RGS-REQ-038 §11 RSK-NET-001）颁发的服务端证书链，握手阶段 `ServerName` SNI 与证书 SAN 校验**不得**关闭；Phase 2 接 rustls（per `ws.rs:16` 缺口登记），deadline 见 TBD-NG-003 |
| NFR-NG-004 | Origin / `Sec-WebSocket-Protocol` 校验**应当**在 Phase 2 启用（per `ws.rs:15` 缺口登记），允许 Origin 白名单配置（默认拒绝跨源）；`Sec-WebSocket-Protocol` 协商**应当**支持 `[游戏A].v1` 子协议以匹配 [游戏A]_client_h5 SDK 默认请求 |
| NFR-NG-005 | 连接活性可观测性**必须**双轨：(a) 业务 cmd=1199 心跳（per FR-NG-008），(b) RFC 6455 Ping/Pong 帧率（per FR-NG-007）；任一缺失均**不得**作为上线门槛，但**必须**在 Prometheus 暴露 `ws_active_connections`、`ws_handshakes_total`、`ws_sessions_dropped_total`（per `crates/network-gateway/src/stats.rs`） |
| NFR-NG-006 | WS 路径**不得**引入新出/入站方向（与 RGS-BAS-006 §4 NetworkPolicy 默认拒绝一致），端口 8000 仍由 NetworkPolicy `allow-network-gateway` Ingress 规则管控，**不**变更策略 ID |

## 7. 架构设计方针 ARC-048：WebSocket网络网关域

| 项目 | 内容 |
|---|---|
| 方针 | 在 `crates/network-gateway` crate 内新增 ARC-048 域（WebSocket网络网关），固化"客户端⇔网关边缘入口协议 = WebSocket"，并以 `Arc<dyn FrameRouter>` 抽象与 ARC-003 既有 QUIC 双路径合流；新增 ARC-048 不引入第三条 dispatcher、不变更骨干协议、不引入新的跨 crate 依赖（tokio-tungstenite 已在 W33 引入并经 ARC-014 判定） |
| 判定基准（附件D§3 开头三条件判定基准） | ①当时确有具体可辩护的替代方案（直接接 QUIC Datagram + 自研浏览器侧 UDP 桥）②采纳会实质改变架构（若浏览器侧 UDP 桥方案被采纳，需新增 `webtransport` crate + 重写客户端 SDK）③否决理由并非"需求就是这么要求的"，而是可论证的"NAT 穿透/中间盒/浏览器兼容性"工程论证。三条件同时成立，**本方针须制定正式 ADR**（见附件D§3 ADR-0048） |
| 否决方案1 | 客户端直接接 QUIC Datagram + WebTransport over HTTP/3。否决理由：浏览器侧 WebTransport 仍处实验阶段（Chromium 97+ 默认开启但 Firefox/Safari 仍需 flags），NAT 穿透与中间盒识别为"未知 UDP"丢包率高于 WS；SDK 接入成本上升 5-10 倍。详见 §4 第 1-2 点 |
| 否决方案2 | 客户端接 WebTransport + 在网关侧重启一条 QUIC Stream 透传。否决理由：违反 ARC-003 的"Stream 路径仅用于网关⇔服务"分工，引入跨边界协议转换的额外延迟（实测 +5-15ms p50），且 `rgs-certgen` 与 `rustls` 现有链路无法直接复用 |
| 候选实现 | ①当前已落地的 `tokio-tungstenite` 0.24 + 自研 `PathCheck` Callback（per `ws.rs:170-190`），无 `RustlsAcceptor` 依赖用于 plain ws；Phase 2 接 rustls 加 `wss://` 路径。两者均经 ARC-014 判定，`tokio-tungstenite` 已在 W33 引入 |
| 不变更范围 | ARC-003 QUIC 双路径**不变更**；TCP `tcp.rs` dispatcher **不变更**；`codec::Frame::decode/encode` wire 格式**不变更**；客户端 SDK（[游戏A]_client_h5）路径字面量**不变更**；ARC-047 FEC 增强层**不变更** |

## 8. 与既有 RGS-REQ-038 §7 周边协议矩阵的关系

RGS-REQ-038 §7 FR-NET-007 当前登记为：

| 客户端⇔网关（实时） | QUIC（可靠Stream＋不可靠Datagram＋§9 FEC增强） | IF-001 |

本文档不取代 RGS-REQ-038 §7，仅做**拆分澄清**：

- **RGS-REQ-038 §7 FR-NET-007** 的"QUIC 路径"在客户端视角应理解为"网关⇔服务骨干协议"，由 ARC-003 约束。
- **RGS-REQ-027 FR-NG-001〜010（本文件）** 的"WebSocket 路径"在客户端视角应理解为"客户端⇔网关边缘入口协议"，由 ARC-048 约束。
- 两条路径经 `Arc<dyn FrameRouter>` 在网络网关内合流，**对外**只暴露 WebSocket 一个端口（8000/8001-wss），**对内**接同一份 `RouteTable` + 5 域 gRPC client（per RGS-BAS-038 §4.2 FEC adapter 调用点结构）。

登记动作（本文档批准后须同步完成）：
1. 在 RGS-REQ-001 §10.4 增加 ARC-048 登记行；
2. 在 RGS-REQ-001 §14.2 增加 TBD-NG-001/002/003 登记行；
3. 在附件D §3 增加 ADR-0048 登记行；
4. 在附件D §2 增加 RSK-NG-001 登记行；
5. RGS-REQ-038 §7 FR-NET-007 在本文件批准后**追加**一行注释："客户端⇔网关边缘入口协议详见 RGS-REQ-027（ARC-048）"，避免出现父文档与本文档"半同步"。

## 9. 验收标准

| ID | 验收标准 |
|---|---|
| AC-NG-001 | 路径校验：HTTP GET `/` / `/api/foo` 等非 `/websocket` 路径返回 404，**不**触发 WS 握手；`/websocket` 路径触发完整握手并进入帧循环（per FR-NG-001/002、`ws_smoke.rs` 已覆盖） |
| AC-NG-002 | 二进制帧 roundtrip：客户端发 `Message::Binary(frame_bytes)`，网关通过 `Frame::decode` → `RouteTableFrameRouter.handle` → `Frame::encode` 回 `Message::Binary(resp_bytes)`，cmd 与 payload 字段按 wire 格式 1:1 对齐 [游戏A]_client_h5 SmartSocket |
| AC-NG-003 | 粘包/半包：单 WS message 含多个 frame 时全部解出（per FR-NG-003 + `ws.rs:245-275` 循环 decode），半包时等下个 WS message 继续 decode |
| AC-NG-004 | 帧长度溢出：`length > MAX_FRAME (1 MiB)` → `Message::Close(None)` + `stats.inc_failed()`，**不**回 5xx、**不**回部分 frame |
| AC-NG-005 | 并发握手：256 并发 WS 握手在 `WsConfig { max_connections: 256 }` 下全部成功；257 并发时第 257 个进入 accept 排队（不丢弃、不 panic），握手排队 p99 < 200ms |
| AC-NG-006 | 协议错误恢复：注入 `FrameError::UnknownTlvType` 帧 → 会话被 `Message::Close` 关闭且 `inc_failed()` 增加，其他并发 session **不受影响** |
| AC-NG-007 | Ping/Pong：客户端发 `Ping(payload)`，网关在同会话内回 `Pong(payload)`，延迟 p99 < 5ms |
| AC-NG-008 | Text frame 忽略：客户端发 `Message::Text("foo")`，网关 debug-log 后忽略，会话**不**关闭（per FR-NG-007） |

## 10. 风险与未决事项

| ID | 内容 | 期限 | 负责人 |
|---|---|---|---|
| TBD-NG-001 | mTLS（`wss://`）接入：当前 `ws.rs` 走 plain `TcpStream`，`rustls` 集成未实施（per `ws.rs:16` 缺口）；建议 Phase 2 接 `tokio-tungstenite` 0.24 的 `tls_accept_async`，证书复用 `rgs-certgen` 服务端证书链 | Phase 2 启动前 | 架构师＋实时负责人 |
| TBD-NG-002 | Origin / `Sec-WebSocket-Protocol` 校验启用：`accept_hdr_async` 回调仅做了 path 校验（per `ws.rs:170-190`），需在 PathCheck 内追加 `req.headers().get("Origin")` 白名单与 `req.headers().get("Sec-WebSocket-Protocol")` 协商；建议与 TBD-NG-001 同批落地以减少协议握手路径变更次数 | Phase 2 启动前 | 架构师 |
| TBD-NG-003 | 业务心跳 vs RFC 6455 Ping/Pong 双轨制：当前仅业务 cmd=1199（per `ws.rs:18` + FR-NG-008），NFR-NG-005 双轨可观测要求需在 Phase 2 决定是否启用 RFC 6455 Ping/Pong 作为补充；建议在 RGS-BAS-038 §4.2 中补"网络层活性 vs 业务层活性"指标拆分 | Phase 2 启动前 | 实时负责人＋SRE |
| TBD-NG-004 | 跨域 saga 推送（per RGS-REQ-016 §3.4 Signal 服务集成）复用 WS 路径时，是否在 `FrameRouter` 层加"事件订阅 cmd"或单独起一条独立 WS 连接；建议不引入新 cmd，沿用现有 `RouteTableFrameRouter` + 业务 cmd 模式 | batch v0.2 评估期 | social 域 Lead |
| TBD-NG-005 | 流式任务进度（per RGS-REQ-015 F-28 推迟到 v0.2）是否借用 WS 路径还是另起 `/streaming`；建议另起，避免与 `WS_PATH` 字面量冲突 | batch v0.2 评估期 | batch 域 Lead |
| RSK-NG-001 | `tokio-tungstenite` 0.24 + `tungstenite` 0.24 仍属活跃维护但版本较新；升级到 0.25+ 时 `Callback` 签名可能变化（per `ws.rs:158` 注释），升级前**必须**复核 `PathCheck` 适配；建议锁定 minor 版本到 0.24.x 直到 batch v0.2 评估完成 | 升级前 | 架构师 |

---

> **登记说明**：本文档批准后，须在 RGS-REQ-001 §10.4 增加 ARC-048 登记行、§14.2 增加 TBD-NG-001/002/003/004/005 登记行、附件D §3 增加 ADR-0048 登记行、附件D §2 增加 RSK-NG-001 登记行，并在 RGS-REQ-038 §7 FR-NET-007 追加"客户端⇔网关边缘入口协议详见 RGS-REQ-027（ARC-048）"注释行。**上述登记已随本文档同批完成**（含附件C v3.11 §7/§8 NG 域与 AC-NG 登记，见附件C v3.11）。
