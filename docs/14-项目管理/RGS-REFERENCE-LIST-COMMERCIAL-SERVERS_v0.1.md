# RGS-REFERENCE-LIST-COMMERCIAL-SERVERS v0.1 — 两款商用服务器参考亮点清单

**创建日期**: 2026-09-20 JST
**创建者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**依据**:
- 上游 REQ-A: `docs/14-项目管理/RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md` (Mavis 2026-09-17 创建)
- 上游 REQ-B: `tools/rgs-shim-rust/docs/ERLANG_TO_RGS_MIGRATION.md` + `H5_ZSYZ_CLIENT_MIGRATION_MATRIX.md` + `SHIM_V05_DISPATCH_DESIGN.md` (Mavis 2026-09-09 创建)
**作用域**: RGS 已参考的两款商用服务器各自亮点汇总
**状态**: ⏳ 一审（Mavis 自审） / 待 Ulysses 二审
**对应工单**: ULYS-134

---

## 0. 文档元信息 / 代签

| 字段 | 值 |
|---|---|
| author | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 审批 | 架构师(Mavis 接手 agent per DEC-008) + 自审 2026-09-20 JST |
| 修订人 | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 创建日期 | 2026-09-20 JST |
| 上游 REQ-A | `docs/14-项目管理/RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md` v0.1 (commit 7b1bc71f) |
| 上游 BASIC-A | `docs/14-项目管理/RGS-BASIC-ANANTA-CBT3-INSPIRED-2026-09-17_v0.1.md` v0.1 (commit 1bece32f) |
| 上游 REQ-B | `tools/rgs-shim-rust/docs/ERLANG_TO_RGS_MIGRATION.md` (commit 5420542) |
| 上游 H5-B | `tools/rgs-shim-rust/docs/H5_ZSYZ_CLIENT_MIGRATION_MATRIX.md` (commit 54872c39) |
| 上游 Dispatch-B | `tools/rgs-shim-rust/docs/SHIM_V05_DISPATCH_DESIGN.md` (commit 076bebf3) |
| 关联 | AGENTS.md §2 / §6 / §7 + 守门 #1 (缺标比错标) |

---

## 1. 两款商用服务器总览

### 1.1 识别依据

| 编号 | 服务器 | 来源 | 业务背景 | RGS 关联 |
|---|---|---|---|---|
| **A** | [游戏D] [游戏D]_CBT3 PrivateServer ([游戏D]) | NAS `[跨盘-某发行商目录]/[游戏D][游戏D]_CBT3本地端\本地服务端\PrivateServer\PrivateServer`，2026-09-16 21:43 JST 复制到 `[跨盘-某发行商目录]/[游戏D]_privatesrv` (267 MB / 471 文件 / 110 子目录) | [某厂商] 2024 公布的新 IP（非《逆水寒》续作，独立 IP），内部代号 `[代码名-D]`，[游戏D] 客户端本地化的服务端工程，让 [游戏D] 客户端脱离网易服务器在本地单机运行 | **RGS-REFERENCE-[游戏D]-[游戏D]_CBT3-PRIVATE-SERVER_v0.1.md** (8 大技术亮点 + 10 条借鉴需求) + **RGS-BASIC-[游戏D]-[游戏D]_CBT3-INSPIRED-2026-09-17_v0.1.md** (基本设计) |
| **B** | [游戏A]_server (《[游戏A]》商用游戏服务端) | E 盘 `[跨盘-某发行商目录]/[游戏A]/[游戏A]\server分析\[游戏A]_server\` Erlang 源码（43 个 `proto_*.erl` + 991 pack defs + 514 unique cmd）+ `[游戏A]_client_h5\` H5 客户端源码（766 send cmd / 875 recv cmd per `proto_mate.js`） | [某厂商] MMORPG 商用服务端，Erlang/OTP + SmartSocket TCP 二进制协议，5 域业务 + Cocos2d-js H5 客户端 | **rgs-shim-rust** (SmartSocket BE → RGS gRPC 桥) + `tools/rgs-shim-rust/docs/` (3 份设计文档) |

### 1.2 对照维度（8 项）

| 维度 | [游戏D] (A) | [游戏A]_server (B) |
|---|---|---|
| 服务端栈 | .NET 8 + C# + Node.js + fengari (Lua) | Erlang/OTP + SmartSocket TCP binary |
| 客户端引擎 | Unity IL2CPP 编译 | Cocos2d-js / Cocos2d-x / Cocos Creator 2.3.2 |
| 协议层 | 自研 `RpcFrameDispatcher` (1.8 KB) + C# 反序列化 | SmartSocket BE (4B len + 2B cmd + payload) + `proto_*.erl` 991 pack defs |
| 配置 | JSON 直接进内存（195 MB Config 20 文件） | Erlang term + record + ETS |
| 业务代码组织 | `partial class` 12 文件按业务域拆分 | `mod/*` 模块 + `proto_*.erl` 协议文件 |
| 启动编排 | `Run-All.ps1` 13 步 | START.cmd + README_START.txt |
| 调试/可观测 | `DebugApiServer` (33.3 KB) + 内嵌 DebugPanel HTML | 没有专属 debug server (Erlang observer / trace) |
| 协议版本目录 | `ClientData/<build_version>/` 按 build 隔离 | 单一协议版本（无版本目录机制） |

---

## 2. 服务器 A — [游戏D] [游戏D]_CBT3 PrivateServer 的 8 大技术亮点

> **来源**: `RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md` §2（已落档 Mavis 2026-09-17 创建），本节为摘要索引。

| # | 亮点 | [游戏D] 关键事实 | RGS 借鉴状态 (per §3) |
|---|---|---|---|
| **A-1** | 协议 schema 反向驱动（最深刻） | IL2CPP dump `MethodIds.json` 4203 methods / 421.9 KB + `RpcSurface.json` 1475 methods / 290.7 KB + `rpc_schema.json` 267.1 KB；客户端 build 自动 dump 新 method id，服务端从 binary 反向生成 reader → 协议定义 100% 跟客户端 binary 同步 | **借鉴**: REQ-[游戏D]-001 `rgs-proto-dump` crate (Sprint N, 5 人日) |
| **A-2** | 195 MB Config 直接 JSON 读（"配置即代码"） | 20 个业务 JSON 表（SceneitemConfig 138 MB / SkillConfig 35 MB / BuffConfig 7.8 MB / VehicleConfig 2 MB / WeaponConfig 1 MB / 其他 15 个 11 MB）无 ORM 无 DB schema 无 migration | **借鉴**: REQ-[游戏D]-002 `rgs-config-loader` Master 表改 JSON (Sprint N+1, 8 人日) |
| **A-3** | V5 配置哲学 = 缺标比错标 | `CONFIGDUMP_V5_IMPORT.md`："V5 preserves them as dumped instead of inventing replacement records"，跟 RGS 守门 #1 同款 | **借鉴**: REQ-[游戏D]-006 DB migration 已知问题清单 (Sprint N+2, 1 人日) |
| **A-4** | `partial class` 按业务域拆 12 文件 | `[代码名-D].Server/[代码名-D].Handlers/Game/GameRouter.cs` 是 `internal sealed partial class GameRouter`，跨 12 个文件（Combat 30.5 KB / VehicleStory 27.6 KB / World 16.4 KB / Vehicles 15.8 KB / Endpoints 10.8 KB / Profiles 9.0 KB / Traversal 8.1 KB / TimeWeather 7.3 KB / Switching 6.7 KB / Movement 4.4 KB / GameRouter 4.2 KB / Gm 3.2 KB / Compatibility4229938 3.2 KB） | **借鉴**: REQ-[游戏D]-003 Rust trait 多文件实现业务域拆分规范化 (Sprint N+2, 3 人日) |
| **A-5** | `RpcFrameDispatcher` 1.8 KB 极简协议层 | `[代码名-D].Server/[代码名-D].Network/RpcFrameDispatcher.cs` 全文 1.8 KB，只做 3 件事：心跳 `frame.Mode == 0x04` 自动 reply；客户端未实现 invoke `frame.Mode == 0x08` 自动 ack 而不是丢包；业务路由 `frame.Mode == 0x09` 转发到 `RpcRouter` | **借鉴**: REQ-[游戏D]-009 单 binary 多 gRPC server 思路 (待评估) |
| **A-6** | 玩法层"纯数据驱动" (`WebTraversal.cs` 3 KB) | `[代码名-D].Server/[代码名-D].Gameplay/WebTraversal.cs` 全文 3 KB，所有可调参数走 `config/private-server.json → gameplay.webTraversal`：男主 ID / 女主 ID / 飞索 buff ID 都从 Settings 读 | **借鉴**: REQ-[游戏D]-007 玩法层纯数据驱动模式 (Sprint N+2, 5 人日) + REQ-[游戏D]-010 Lua via mlua 跨语言配置脚本 (待评估, 2 人日) |
| **A-7** | `DebugApiServer` + 内嵌 DebugPanel (开发者体验) | `[代码名-D].Server/[代码名-D].App/DebugApiServer.cs` 33.3 KB，启动时内嵌 `DebugPanel/index.html` (embedded resource 编译进 binary)，提供 JSON API 给面板调用，`config.Debug.Enabled` 开关默认 127.0.0.1 不外泄 | **借鉴**: REQ-[游戏D]-005 `rgs-debug` 内嵌 HTTP debug server + DebugPanel (Sprint N+1, 6 人日) |
| **A-8** | Build version 维度的协议兼容目录 | `[代码名-D].Server/ClientData/4229938/` 每个客户端 build 一个独立子目录（MethodIds.json / RpcSurface.json / MethodId.dump.cs / Configs/195 MB / CONFIGDUMP_V5_IMPORT.md）；新 build 来了新建子目录，旧 build 不破坏 | **借鉴**: REQ-[游戏D]-004 `rgs-protocol/builds/<version>/` 协议版本目录 (Sprint N, 4 人日) |

### 2.1 [游戏D] 借鉴落地的优先 Sprint 分布（per RGS-BASIC §4）

| Sprint | 借鉴 REQ | 工作量 | 阶段 |
|---|---|---|---|
| Sprint N（立即可做，2 周） | REQ-[游戏D]-001 (rgs-proto-dump) + REQ-[游戏D]-004 (rgs-protocol/builds/) | 9 人日 | 协议兼容性基础设施 |
| Sprint N+1（核心借鉴，3 周） | REQ-[游戏D]-002 (rgs-config-loader) + REQ-[游戏D]-005 (rgs-debug) | 14 人日 | Master 表配置 + ST 阶段可用 debug panel |
| Sprint N+2（深化，2.5 周） | REQ-[游戏D]-003 (handler partial) + REQ-[游戏D]-006 (KNOWN_ISSUES) + REQ-[游戏D]-007 (data-driven) + REQ-[游戏D]-008 (start-rgs-stack.ps1) | 12 人日 | 6 域代码规范化 + ST 启动脚本完善 |
| 待评估 | REQ-[游戏D]-009 (saga-runtime multi-server) + REQ-[游戏D]-010 (mlua) | 待评估 | 后续 sprint |

**[游戏D] 总工作量**: ~37 人日 / 3 sprint（不含待评估）

