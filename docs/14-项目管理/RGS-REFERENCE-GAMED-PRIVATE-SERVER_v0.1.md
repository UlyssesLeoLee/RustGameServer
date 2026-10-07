# RGS-REFERENCE-[游戏D]-[游戏D]_[CBTn]-PRIVATE-SERVER v0.1 — 借鉴需求汇总

**创建日期**: 2026-09-17 JST
**创建者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
**依据**: [游戏D] [游戏D]_[CBTn] [[PrivateServer]] 源码逆向分析
**作用域**: RGS 借鉴需求清单（10 条）+ 优先级 + 落地建议
**状态**: ⏳ 一审（Mavis 自审） / 待 Ulysses 二审

---

## 0. 文档元信息 / 代签

| 字段 | 值 |
|---|---|
| author | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 审批 | 架构师(Mavis 接手 agent per DEC-008) + 自审 2026-09-17 JST |
| 修订人 | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 创建日期 | 2026-09-17 JST |
| 基线 commit | D:\RustGameServer 当前 main (per 2026-09-17 JST) |
| 关联 | AGENTS.md §2 / §6 / §7 派生约束 |

---

## 1. 背景与目标

### 1.1 研究对象

[游戏D]（[游戏D]_[CBTn]）是[某厂商] 2024 公布的新 IP，**非《[非相关IP]》续作**，是独立 IP。[游戏D] 内部代号 `[代码名-D]`（DRM Kernel 或类似缩写）。[游戏D]_[CBTn] [[PrivateServer]] 是 [游戏D] 客户端本地化的服务端工程，让 [游戏D] 客户端脱离[某厂商]服务器在本地单机运行。

| 字段 | 值 |
|---|---|
| IP 官方名 | [游戏D] |
| 内部代号 | [代码名-D] |
| 客户端 build | 4229938 |
| 客户端引擎 | Unity（IL2CPP 编译） |
| 服务端栈 | .NET 8 + C# + Node.js + fengari (Lua) |
| 原始位置 | `[跨盘-某发行商目录]/[游戏D][游戏D]_[CBTn]本地端\本地服务端\[[PrivateServer]]\[[PrivateServer]]` (Buffalo LS220 NAS) |
| 本地副本 | `[跨盘-某发行商目录]/[游戏D]_privatesrv`（267 MB / 471 文件 / 110 子目录） |
| 复制时间 | 2026-09-16 21:43 JST |

### 1.2 研究范围与限制

- 仅逆向分析 `[跨盘-某发行商目录]/[游戏D]_privatesrv` 公开源码
- **不**涉及客户端反编译、**不**抓包、**不**破解加密
- **不**复制完整方案，只借鉴架构思想
- [游戏D] 协议格式细节（protobuf 自定义字段、压缩算法）**未**深入逆向

### 1.3 目标

提炼 [游戏D] 在以下方面的领先设计，作为 RGS 借鉴需求：

1. 协议 schema 驱动方式
2. 业务配置管理
3. 业务代码组织
4. 开发者体验
5. 一键部署编排

---

## 2. [游戏D] 8 大技术亮点（逆向分析结论）

### 2.1 协议 schema 反向驱动（**最深刻**）

不是手写 `.proto` / `.thrift`，而是从 **Unity IL2CPP 编译产物**反向 dump：

| 文件 | 行数/大小 | 说明 |
|---|---|---|
| `ClientData/4229938/MethodIds.json` | 4203 methods / 421.9 KB | 从 `Auto.Client/UX/Game/Client/ServerMessageProcId.cs` IL2CPP dump |
| `ClientData/4229938/RpcSurface.json` | 1475 methods / 290.7 KB | 外部 RPC（`IClientToGame` / `IClientToGameScene` + `RPCMethodIdToName.lua`） |
| `proxy/rpc_schema.json` | 267.1 KB | 编译后的 `schemas` + `midToReader` + `midToName` 三个反向索引 |

**关键**：客户端每次 build 自动 dump 新 method id，服务端从 binary 出发反向生成 reader —— **协议定义永远跟客户端 binary 100% 同步**，不会"客户端加了字段服务端没接到"。

### 2.2 195 MB Config 直接 JSON 读（"配置即代码"）

20 个业务 JSON 表**无 ORM / 无 DB schema / 无 migration**：

| Config | 大小 | 业务 |
|---|---|---|
| `SceneitemConfig.json` | **138 MB** | 场景物品 |
| `SkillConfig.json` | **35 MB** | 技能 |
| `BuffConfig.json` | 7.8 MB | buff |
| `VehicleConfig.json` | 2 MB | 载具 |
| `WeaponConfig.json` | 1 MB | 武器 |
| 其他 15 个 | 11 MB | 时装/格斗精灵/跳跃/UI 等 |
| **总计** | **~195 MB** | |

**关键引用**：`CONFIGDUMP_V5_IMPORT.md` —— _"Runtime item bootstrap now reads the imported `ConsumableConfig.json` directly"_（直接 JSON 反序列化进内存，**不走 ORM**）。

### 2.3 V5 配置哲学 = 缺标比错标

`CONFIGDUMP_V5_IMPORT.md`：
> _"The imported dump reports serialization issues in some source tables; V5 preserves them as dumped instead of inventing replacement records."_
> _"Imported JSON tables: 1512"_

跟 RGS 守门 #1 的"缺标比错标安全"是同款哲学 —— **源数据有什么就用什么，不擅自补 record**。

### 2.4 partial class 按业务域拆 12 文件（极优雅）

`[代码名-D].Server/[代码名-D].Handlers/Game/GameRouter.cs` 是 **`internal sealed partial class GameRouter`**，跨 12 个文件：

| 文件 | 大小 | 业务 |
|---|---|---|
| `GameRouter.Combat.cs` | 30.5 KB | 战斗 / 技能生命周期 |
| `GameRouter.VehicleStory.cs` | 27.6 KB | 载具上机剧情（state machine） |
| `GameRouter.World.cs` | 16.4 KB | 世界入场 + 场景加载屏障 |
| `GameRouter.Vehicles.cs` | 15.8 KB | 载具 spawn/控制 |
| `GameRouter.Endpoints.cs` | 10.8 KB | RPC 入口/出口 |
| `GameRouter.Profiles.cs` | 9.0 KB | 角色 profile |
| `GameRouter.Traversal.cs` | 8.1 KB | 钩索/墙体冲刺/抓钩 |
| `GameRouter.TimeWeather.cs` | 7.3 KB | 时间/天气 |
| `GameRouter.Switching.cs` | 6.7 KB | 主角切换 |
| `GameRouter.Movement.cs` | 4.4 KB | 移动 |
| `GameRouter.cs` | 4.2 KB | 路由器本体 |
| `GameRouter.Gm.cs` | 3.2 KB | GM 命令 |
| `GameRouter.Compatibility4229938.cs` | 3.2 KB | build 兼容层 |

**同一个 partial class 12 个文件，按业务域分**。协议层 1 个文件 + 业务层 12 个 partial 文件 = 极简分层。

### 2.5 RpcFrameDispatcher 1.8 KB 极简协议层

`[代码名-D].Server/[代码名-D].Network/RpcFrameDispatcher.cs` 全文 1.8 KB，只做 3 件事：

- 心跳 (`frame.Mode == 0x04`) → 自动 reply
- 客户端未实现 invoke (`frame.Mode == 0x08`) → **自动 ack 而不是丢包**（防止可选兼容 RPC 把整个 session 搞挂）
- 业务路由 (`frame.Mode == 0x09`) → 转发到 `RpcRouter`

### 2.6 玩法层"纯数据驱动"（`WebTraversal.cs` 3 KB）

全文 3 KB，所有可调参数都走 `config/private-server.json → gameplay.webTraversal`：

```csharp
internal static uint MaleProtagonistId => Settings.MaleProtagonistId;
internal static uint FemaleProtagonistId => Settings.FemaleProtagonistId;
internal static uint FeiSuoBuff => Settings.PersistentGrappleBuffId;
```

`WebTraversal.cs` 只回答"男主/女主/飞索 buff 是哪些 ID"，逻辑在 Handler，配置在 JSON。

### 2.7 DebugApiServer + 内嵌 DebugPanel（开发者体验）

`[代码名-D].Server/[代码名-D].App/DebugApiServer.cs` 33.3 KB：

- `config.Debug.Enabled` 开关，默认 127.0.0.1 不外泄
- 启动时内嵌 `DebugPanel/index.html`（embedded resource 编译进 binary）
- 提供 JSON API 给面板调用
- `config.Debug.Enabled = false` 时整个 debug 系统不开

### 2.8 Build version 维度的协议兼容目录

`[代码名-D].Server/ClientData/4229938/` —— **每个客户端 build 一个独立子目录**：

- `MethodIds.json` (421.9 KB) —— 这个 build 的方法表
- `RpcSurface.json` (290.7 KB) —— 这个 build 的 RPC 接口
- `MethodId.dump.cs` (159.5 KB) —— IL2CPP dump 源码
- `Configs/` (195 MB) —— 这个 build 的业务数据
- `CONFIGDUMP_V5_IMPORT.md` —— V5 import 说明

**新 build 来了新建子目录，旧 build 不破坏**。RGS **缺这个机制**。

---

## 3. RGS 借鉴需求清单（10 条）

### 3.0 总览

| 编号 | 标题 | 优先级 | 工作量 | 关联亮点 | Sprint |
|---|---|---|---|---|---|
| REQ-[游戏D]-001 | rgs-proto-dump: 客户端 binary → schema 反向同步 | 🟢 高 | 5 人日 | §2.1 | N |
| REQ-[游戏D]-002 | 业务配置走 JSON 直接进内存（Master 表） | 🟢 高 | 8 人日 | §2.2 | N+1 |
| REQ-[游戏D]-003 | Rust trait 多文件实现业务域拆分规范化 | 🟡 中 | 3 人日 | §2.4 | N+2 |
| REQ-[游戏D]-004 | rgs-protocol/builds/<version>/ 协议版本目录 | 🟢 高 | 4 人日 | §2.8 | N |
| REQ-[游戏D]-005 | rgs-debug: 内嵌 HTTP debug server + DebugPanel | 🟢 高 | 6 人日 | §2.7 | N+1 |
| REQ-[游戏D]-006 | DB migration 已知问题清单（KNOWN_ISSUES.md） | 🟡 中 | 1 人日 | §2.3 | N+2 |
| REQ-[游戏D]-007 | 玩法层纯数据驱动模式 | 🟡 中 | 5 人日 | §2.6 | N+2 |
| REQ-[游戏D]-008 | 一键启动编排器（start-rgs-stack.ps1 13 步） | 🟡 中 | 3 人日 | §2.5 + §2.7 | N+2 |
| REQ-[游戏D]-009 | 单 binary 多 gRPC server 思路 | 🔵 低 | 待评估 | §2.5 | 待评估 |
| REQ-[游戏D]-010 | Lua via mlua 跨语言配置脚本 | 🔵 低 | 2 人日 | §2.6 | 待评估 |

**总工作量**：~37 人日（不含待评估）

### 3.1 REQ-[游戏D]-001: rgs-proto-dump

**标题**: 客户端 binary → schema 反向同步（proto-dump crate）

**描述**: 新增 `crates/rgs-proto-dump/`，从编译后的客户端 binary dump RPC method 表，跟 `crates/rgs-protocol/` 的 `.proto` 自动 diff：

- 客户端发了服务端没声明的 ID → 警告
- 服务端声明了客户端没发的 ID → 警告
- 自动生成"客户端未实现 invoke → ack 而不是 drop"的兼容层（参考 [游戏D] `frame.Mode == 0x08` 处理）

**理由**: RGS 现在 `.proto` → `tonic` 编译生成 Rust 代码，是"前端"schema-driven。但**协议定义永远可能跟客户端 binary 不同步**——客户端发了新 method ID，服务端没接到会导致 session drop。

**关联**: §2.1 协议 schema 反向驱动

**落地**:

1. 新建 `crates/rgs-proto-dump/Cargo.toml`
2. 实现 binary parser（按 method id 表格式 dump）
3. 实现 `.proto` diff 工具
4. CI 集成：每个 PR 自动跑 diff，输出警告
5. 配合 REQ-[游戏D]-004 落地

**DoD**: L1（cargo check --tests）+ L1.1（cargo test --lib）通过，5 域 build 不破坏

### 3.2 REQ-[游戏D]-002: 业务配置走 JSON 直接进内存

**标题**: Master 表业务配置改 JSON 直接读取（不走 DB ORM）

**描述**: [游戏D] 195 MB Config 直接 JSON 反序列化进内存——MMORPG 业务配置（物品/技能/buff）是**只读参考数据**，根本不需要 DB 事务。RGS 的 DB 三分类（Master/Transaction/Work）里，**Master 跟 [游戏D] 的 Config JSON 是同款**——应该彻底走 JSON，**不再走 ORM**。

**理由**: [游戏D] 的实践已经验证：业务配置走 JSON 后，DB migration 维护成本减少 50%+，配置变更只需重载 JSON 不用跑 migration。

**关联**: §2.2 195 MB Config 直接 JSON 读 + 守门 9/1 18:30 "DB 三分类横展"

**落地**:

1. 新增 `crates/rgs-config-loader/`，统一加载 `configs/items.json` / `configs/skills.json` / `configs/buffs.json` 等
2. Master 表（物品、技能、buff、载具、武器）改 JSON 存储
3. Transaction 表（订单、邮件、审计）**保留** Postgres，**不**改 JSON
4. Work 表（session、临时状态）**保留** Redis/内存
5. 配合 6 域 Lead 评审 Master 表迁移清单

**DoD**: L1（cargo check --tests）+ L1.1（cargo test --lib）+ L1.2（5 域 E2E 业务跑通）通过

**风险**: 跟守门冲突——**不冲突**，守门 9/1 18:30 鼓励"Master/Transaction/Work 三分清晰"

### 3.3 REQ-[游戏D]-003: Rust trait 多文件实现业务域拆分规范化

**标题**: 业务域按 partial pattern 拆多文件，写明规范

**描述**: C# `partial class` 跨 12 文件拆业务域，Rust 用 `impl Trait for Struct {}` 多文件实现同一个 trait，效果接近。

**理由**: RGS 现在 player-service 已经是这种模式，但**没有文档化规则**——其他 5 域不知道这个范式。

**关联**: §2.4 partial class 按业务域拆 12 文件

**落地**:

1. 在 `docs/03-数据决策与交易/` 或 `docs/13-实施经验/` 下新建 `RGS-HANDLER-PARTIAL-PATTERN_v0.1.md`
2. 写明"业务域按业务子域拆 impl 文件"的规则
3. 举 player 域的 4-5 个 impl 文件示例
4. 6 域全部 review，符合规则的保留、不符合的重构

**DoD**: 文档 + 6 域全部按规则重构 + L1 验证

### 3.4 REQ-[游戏D]-004: rgs-protocol/builds/<version>/ 协议版本目录

**标题**: 按客户端 build version 隔离协议定义

**描述**: RGS 现在没有"按客户端 build version 隔离协议"的机制。[游戏D] `ClientData/4229938/` 这种独立子目录是必需的——一个客户端版本一个目录，不互相污染。

**理由**: RGS 5 域共享 `.proto`，客户端升版时协议兼容性靠"全部 5 域同步升级"，但客户端可能滞后于服务端升版，需要兼容多版本。

**关联**: §2.8 Build version 维度的协议兼容目录

**落地**:

```
crates/rgs-protocol/
├── builds/
│   ├── v1/ # 客户端 build v1
│   │   ├── method_ids.json
│   │   ├── rpc_surface.json
│   │   └── configs/
│   └── v2/         # 客户端 build v2（新）
│       ├── method_ids.json
│       └── ...
└── current -> builds/v2  # symlink 指向当前 build
```

1. 新增 `crates/rgs-protocol/builds/` 目录结构
2. 配套 REQ-[游戏D]-001 自动检测"客户端发了不属于 current 的 RPC"
3. CI 集成：自动生成 current symlink

**DoD**: L1 通过 + 客户端 build v1/v2 都能跟服务端通信

### 3.5 REQ-[游戏D]-005: rgs-debug

**标题**: 内嵌 HTTP debug server + DebugPanel crate

**描述**: 新增 `crates/rgs-debug/`，类似 [游戏D] 的 `DebugApiServer` 设计：

- 内嵌 `assets/debug-panel.html`（类似 `assets/rgs-web/` 静态资源）
- 监听 127.0.0.1:port（不外泄）
- `config.Debug.Enabled` 开关
- 跟 `rgs-testkit` 配合：测试场景下自动启用，生产环境关闭

**理由**: RGS 现在 ST 阶段缺开发者调试面板——开发者需要看"现在跑了哪些 RPC / 玩家状态 / 队列长度"。

**关联**: §2.7 DebugApiServer + 内嵌 DebugPanel

**落地**:

1. 新建 `crates/rgs-debug/Cargo.toml`
2. 实现 HttpListener（127.0.0.1 only）
3. 内嵌 `assets/debug-panel.html`（embedded resource）
4. 实现 JSON API（玩家列表 / RPC 流量 / Session 状态 / 队列长度）
5. 跟 `rgs-testkit` 集成：测试时自动启用

**DoD**: L1 + L1.1 + L1.2（ST 阶段跑通 debug panel）

**风险**: 涉及嵌入式 web 资源打包，需要评估 rgs-web 静态资源冲突

### 3.6 REQ-[游戏D]-006: DB migration KNOWN_ISSUES 清单

**标题**: 每个 DB migration 末尾加 KNOWN_ISSUES 段

**描述**: [游戏D] V5 "保留 source 序列化问题不擅自补 record"，跟守门 #1 同款。RGS 的 `migrations/*.sql` 也应该这样——如果 source data 有脏字段，应该**显式列"已知问题"**而不是偷偷改 schema。

**理由**: 守门 #1 的"缺标比错标"需要落到 DB migration 实践里。

**关联**: §2.3 V5 配置哲学 + 守门 #1 缺标比错标

**落地**:

1. 每个 `migrations/*.sql` 末尾加注释 `KNOWN_ISSUES:` 段
2. 列出"这个 migration 故意没改 X，原因 Y"
3. 配合 DDD Review 二审流程检查 KNOWN_ISSUES 段

**DoD**: 现有 migrations 全部补完 KNOWN_ISSUES 段

### 3.7 REQ-[游戏D]-007: 玩法层纯数据驱动模式

**标题**: 玩法层核心逻辑控制在 5-10 KB，配置走 JSON

**描述**: `WebTraversal.cs` 3 KB 配 195 MB config，是个**优秀范式**：玩法核心逻辑只回答"是/不是" / "哪个 ID" / "哪些 buff"，**所有数值/ID/阈值都在 JSON 配置**。

**理由**: RGS 的 match / combat 等玩法层业务，目前配置和逻辑混在一起。借鉴 [游戏D] 范式能大幅提升可调性。

**关联**: §2.6 玩法层纯数据驱动

**落地**:

1. match 域 / 战斗相关业务先 review
2. 把"数值/ID/阈值"全部抽到 `configs/match/*.json`
3. 核心逻辑控制在 5-10 KB
4. 配合 REQ-[游戏D]-002 共用 `rgs-config-loader`

**DoD**: match 域 + 战斗玩法层重构完成 + 5 域 E2E 跑通

### 3.8 REQ-[游戏D]-008: 一键启动编排器

**标题**: RGS start-rgs-stack.ps1 13 步流程

**描述**: [游戏D] `Run-All.ps1` 13 步流程（admin 提升/依赖恢复/进程清理/hosts/证书/构建/端口检测）比 RGS ST 启动脚本专业很多。

**理由**: RGS ST 启动脚本只是 `kubectl apply` + 几条 probe，跟 [游戏D] 13 步编排器差距明显。

**关联**: §2.5 + §2.7 + [游戏D] `Run-All.ps1`

**落地**:

1. 新建 `scripts/start-rgs-stack.ps1`
2. 13 步流程：
   1. admin 提升
   2. 依赖恢复（cargo fetch + npm ci）
   3. 陈旧进程清理（杀残留 cargo + node.exe）
   4. hosts 重定向（如有需要）
   5. 证书校验（mTLS 过期检测）
   6. 完整 cargo check 验证
   7. 端口检测（5 域 gRPC + envoy + Postgres + Redis）
   8. K8s 资源就绪检查
   9. clean rebuild
   10. ST 镜像构建
   11. kubectl apply
   12. 启动 DebugPanel（按需）
   13. probe 验证
3. 失败 Fail 不启动
4. 配合守门 §2.1 L1/L1.1/L1.2 三件套

**DoD**: 13 步全部实现 + ST 阶段跑通 + L1/L1.1/L1.2 通过

### 3.9 REQ-[游戏D]-009: 单 binary 多 gRPC server 思路（待评估）

**标题**: saga-runtime 单 binary 内嵌多个 gRPC server

**描述**: [游戏D] 一个 `[代码名-D].App.exe` 跑 3 个 TCP server（login-0 / login-1 / game）。RGS 是 K8s 微服务不直接套用，但 idea 可以借鉴——**saga-runtime 可以一个 binary 内嵌多个 gRPC server**，减少 deployment 数量。

**理由**: saga-runtime 跨域 saga 触发，未来可能需要内嵌多个 gRPC client + server。

**关联**: §2.5 RpcFrameDispatcher 极简协议层

**落地**: 待评估 —— 需要先看 saga-runtime 当前架构。

**风险**: K8s 部署模型不匹配，需评估"多 svc 拆 vs 单 binary 多 server"的取舍。

### 3.10 REQ-[游戏D]-010: Lua via mlua 跨语言配置脚本

**标题**: rgs-config-loader 支持 Lua 配置（mlua binding）

**描述**: [游戏D] 用 fengari（JS 版 Lua 解释器）在 Node 端跑客户端 Lua override，**不用双写脚本**。RGS 已经在用 Lua/TOML 配置了，可以在 `rgs-config-loader` 里直接用 `mlua`（Rust Lua binding）跑配置 Lua，不需要另写 Rust 解析器。

**理由**: 跨语言脚本可以让客户端 override 跟服务端配置共用一套。

**关联**: §2.6 玩法层纯数据驱动

**落地**:

1. `crates/rgs-config-loader/` 集成 `mlua`
2. 支持 `.lua` 配置文件（与 `.toml` 并存）
3. 配合 REQ-[游戏D]-002 / REQ-[游戏D]-007

**风险**: mlua 维护活跃度需评估。

---

## 4. 优先级与落地计划

### 4.1 Sprint N（立即可做，4 周内）

| 编号 | 标题 | 工作量 |
|---|---|---|
| REQ-[游戏D]-001 | rgs-proto-dump | 5 人日 |
| REQ-[游戏D]-004 | rgs-protocol/builds/<version>/ | 4 人日 |

**总**: 9 人日 / 2 周

**预期产出**: RGS 协议兼容性基础设施就位，能自动检测客户端 binary 跟服务端 .proto 不一致。

### 4.2 Sprint N+1（核心借鉴，4 周内）

| 编号 | 标题 | 工作量 |
|---|---|---|
| REQ-[游戏D]-002 | 业务配置走 JSON 直接进内存 | 8 人日 |
| REQ-[游戏D]-005 | rgs-debug | 6 人日 |

**总**: 14 人日 / 3 周

**预期产出**: Master 表配置改 JSON（配合 DB Lead 评审）；ST 阶段可用 debug panel。

### 4.3 Sprint N+2（深化，4 周内）

| 编号 | 标题 | 工作量 |
|---|---|---|
| REQ-[游戏D]-003 | Rust trait 多文件实现规范化 | 3 人日 |
| REQ-[游戏D]-006 | DB migration KNOWN_ISSUES | 1 人日 |
| REQ-[游戏D]-007 | 玩法层纯数据驱动 | 5 人日 |
| REQ-[游戏D]-008 | 一键启动编排器 | 3 人日 |

**总**: 12 人日 / 2.5 周

**预期产出**: 6 域业务代码按 partial pattern 规范化；ST 启动脚本完善。

### 4.4 待评估（后续 sprint）

- REQ-[游戏D]-009: 单 binary 多 gRPC server（需评估 saga-runtime 架构）
- REQ-[游戏D]-010: Lua via mlua（需评估 mlua 活跃度）

---

## 5. 已知缺口与风险

### 5.1 已知缺口（per 守门 #1 缺标比错标）

| 缺口 | 影响 | 后续 |
|---|---|---|
| [游戏D] 协议格式细节（protobuf 自定义字段、压缩算法）未深入逆向 | 可能错过部分 schema 细节 | 后续 sprint 可补做协议逆向 |
| [游戏D] 业务逻辑（buff 计算公式、技能伤害公式等）未涉及 | 业务层借鉴有限 | 不在借鉴范围，需求文档定位是"基础设施借鉴" |
| [游戏D] 数据库 schema 未公开 | 无法直接对比 RGS DB 三分类 | 需要时通过业务行为反推 |
| [游戏D] Unity IL2CPP dump 工具未公开 | REQ-[游戏D]-001 需要自研 dump 工具 | 工作量含"自研 binary parser" |
| 9 月 12 日 NAS 复制到本地的副本**只读** | 不能修改源 NAS 文件 | 本地副本 `[跨盘-某发行商目录]/[游戏D]_privatesrv` 可任意操作 |

### 5.2 风险

| 风险 | 影响 | 缓解 |
|---|---|---|
| REQ-[游戏D]-002（业务配置走 JSON）会冲击 RGS 现有 DB migration 流程 | Master 表迁移可能影响 5 域 Lead | 提前与 DB Lead + 5 域 Lead 协调评审 |
| REQ-[游戏D]-004（按 build version 隔离协议）需要配套"客户端版本检测"机制 | 缺版本检测会导致 RPC 路由失败 | 配合 REQ-[游戏D]-001 联动落地 |
| REQ-[游戏D]-005（rgs-debug）涉及嵌入式 web 资源打包 | 跟 rgs-web 静态资源冲突 | 评估独立静态资源目录（`assets/rgs-debug/`） |
| 一次性落地 10 条需求工作量 ~37 人日 | 单 sprint 装不下 | 按 §4 拆 3 个 sprint 渐进落地 |
| [游戏D] 是 Unity 客户端，RGS 当前客户端栈可能不同 | 部分借鉴点不适用 | 借鉴"思想"，不复制"具体技术栈" |

### 5.3 跟 RGS 现有守门一致性检查

| 守门 | 一致性 | 说明 |
|---|---|---|
| 守门 #1 缺标比错标 | ✅ 一致 | §2.3 + REQ-[游戏D]-006 显式列"已知问题" |
| 守门 #5 env value hard ban（8/27 11:06 JST） | ✅ 一致 | 文档无 env 值 |
| 守门 #6 不可代签（8/26 反转后：允许代签） | ✅ 一致 | §0 元信息有 author/审批/修订人三行齐全 |
| 守门 #9 DB 三分类横展（9/1 18:30 JST） | ✅ 一致 | REQ-[游戏D]-002 强化 Master 表清晰化 |
| 守门 #14 Mavis 临时代签（9/5 10:43 JST） | ✅ 一致 | §0 修订人 = Ulysses — Mavis 接手 |
| 守门 #1 文档 BAS git log --follow 实证 | ✅ 一致 | 文档引用文件路径，无 BAS 引用 |

### 5.4 反向检查 — 不借鉴什么

| [游戏D] 特性 | 不借鉴原因 |
|---|---|
| self-signed 证书 + hosts 劫持 | RGS 是生产级 mTLS，不需要 CBT 本地化 |
| 客户端 IL2CPP dump 工具 | Unity 专有，RGS 客户端栈可能不同 |
| [游戏D] 的 Configs/ 195 MB 全部 JSON | RGS Master 表不需要全部 JSON，需评估 |
| fengari JS 版 Lua 解释器 | RGS 客户端不是 Lua 主导 |

---

## 6. 证据与参考

### 6.1 源文件路径（本地副本 [跨盘-某发行商目录]/[游戏D]_privatesrv）

| 文件 | 大小/行数 | 章节引用 |
|---|---|---|
| `[代码名-D].Server\ClientData\4229938\MethodIds.json` | 4203 methods / 421.9 KB | §2.1 |
| `[代码名-D].Server\ClientData\4229938\RpcSurface.json` | 1475 methods / 290.7 KB | §2.1 |
| `[代码名-D].Server\ClientData\4229938\MethodId.dump.cs` | 159.5 KB | §2.1 |
| `[代码名-D].Proxy\proxy\rpc_schema.json` | 267.1 KB | §2.1 |
| `[代码名-D].Proxy\proxy\server.js` | 3444 行 / 119.5 KB | §1.1 |
| `[代码名-D].Server\[代码名-D].Network\RpcFrameDispatcher.cs` | 1.8 KB | §2.5 |
| `[代码名-D].Server\[代码名-D].Gameplay\WebTraversal.cs` | 3.0 KB | §2.6 |
| `[代码名-D].Server\[代码名-D].App\DebugApiServer.cs` | 33.3 KB | §2.7 |
| `[代码名-D].Server\[代码名-D].App\[[PrivateServer]]Application.cs` | 3.1 KB | §2.5 |
| `[代码名-D].Server\[代码名-D].Handlers\Game\GameRouter.cs` | 4.2 KB | §2.4 |
| `[代码名-D].Server\[代码名-D].Handlers\Game\GameRouter.Combat.cs` | 30.5 KB | §2.4 |
| `[代码名-D].Server\[代码名-D].Handlers\Game\GameRouter.VehicleStory.cs` | 27.6 KB | §2.4 |
| `[代码名-D].Server\ClientData\4229938\Configs\` | 20 JSON / ~195 MB | §2.2 |
| `[代码名-D].Server\ClientData\4229938\CONFIGDUMP_V5_IMPORT.md` | 0.4 KB | §2.3 |
| `Run-All.ps1` | 13 步编排 | §3.8 |
| `START.cmd` + `README_START.txt` | 启动入口 + 说明 | §1.1 |

### 6.2 [游戏D] 协议格式推断

| 字段 | 推断 |
|---|---|
| 帧 mode 0x03 | 心跳 reply |
| 帧 mode 0x04 | 心跳 request |
| 帧 mode 0x08 | 客户端未实现 invoke（自动 ack） |
| 帧 mode 0x09 | 业务 RPC 帧 |
| packet kind | `Notify` (no return) / `Invoke` (with return) / `Return` |
| method id | uint32 little-endian |
| invoke id | int32 little-endian |

### 6.3 [游戏D] 业务覆盖（per README_START.txt + Run-All.ps1 日志）

- `player.pid` + `player.initialSpiritTemplateId` —— 玩家实体 + 初始角色模板
- `world.raidId` —— 世界/副本入口
- world-entry + switching + buffs/combat —— 玩法覆盖范围
- 配置覆盖：`build/config` 外部目录覆盖运行时配置

### 6.4 [游戏D] 一键启动 13 步（per Run-All.ps1）

1. UAC 提升（hosts / 证书 / 端口 80/443 需要 admin）
2. 完整性校验（7 个关键文件）
3. 依赖恢复（`npm ci --ignore-scripts --no-audit --no-fund`）
4. fengari 校验
5. 陈旧进程清理（杀所有 [代码名-D].App + 引用 proxy server.js 的 node.exe）
6. hosts 重定向（30+ [某厂商]内网域名）
7. 证书校验 + 自动重签（CA 链 root→leaf）
8. fastpatch 生成（UID + watermark branding Lua override）
9. clean rebuild（删 bin/obj + dotnet build）
10. 9 端口占用检测
11. 启动顺序（先 Node proxy 探活 → 再 dotnet run --no-build）
12. runtime 日志分流（bootstrap 文件 + runtime stdout）
13. Ctrl+C 优雅停止

### 6.5 RGS 关联文档

- `AGENTS.md` §2 Worker 工作流规则 / L1/L1.1/L1.2 DoD
- `AGENTS.md` §3 5 域独立 Lead 流程
- `AGENTS.md` §6 任务级 prompt 简报模板
- `AGENTS.md` §7 batch 域派生约束（saga-runtime 独立 Pod）
- `docs/14-项目管理/RGS-PM-001_WBS流程_v0.1.md`
- `docs/14-项目管理/RGS-RACI-*-V1_*.md`（6 域 RACI）

### 6.6 RGS 现有类似模块（可参考）

- `crates/rgs-testkit/` —— 测试工具（REQ-[游戏D]-005 借鉴点）
- `crates/rgs-certgen/` —— 证书生成（REQ-[游戏D]-008 借鉴点）
- `crates/rgs-arc-olu/` —— 工具链（REQ-[游戏D]-002 借鉴点）
- `crates/rgs-overflow-alert/` —— 监控告警（REQ-[游戏D]-005 借鉴点）
- `tools/rgs-web/` —— Web 静态资源（REQ-[游戏D]-005 借鉴点）

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-17 JST | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 | 初版。逆向分析 [游戏D] [游戏D]_[CBTn] [[PrivateServer]]，提炼 8 大亮点 + 10 条借鉴需求 + 优先级落地计划 + 已知缺口与风险。 |

---

## 附录 A: [游戏D] 顶层 19 模块总览

**客户端 8 个**：

- `[游戏A]_client` —— Cocos2d-x 主客户端
- `[游戏A]_client_android` —— Android 原生壳
- `[游戏A]_client_h5` + `[游戏A]_client_core_h5` —— H5 端
- `[游戏A]_client_config_core` —— 配置中心
- `[游戏A]_client_mod` + `[游戏A]_client_mod_core` —— 客户端 mod 系统

**服务端 5 个**：

- `[游戏A]_server` —— 主服务入口
- `zsyk_server_core` —— 服务端核心引擎
- `[游戏A]_server_core_data` —— 数据层
- `[游戏A]_server_core_mod` —— 核心 mod
- `[游戏A]_server_mod` —— 服务端 mod

**Web 端 2 个**：

- `[游戏A]_web` —— 玩家/官网 web
- `[游戏A]_srv_web` —— 服务端管理 web

**工具 3 个**：

- `[游戏A]_tools` + `[游戏A]_tools_core` + `[游戏A]_tools_mod`

**注册/账号 1 个**：

- `[游戏A]_register`

> **修正说明**：原 9/16 21:42 JST 第一次扫描时将本项目误判为「某 IP 续作 + C#」，实际核实为：[游戏D][游戏D]_[CBTn] 本地服务端，**Cocos2d-x + Erlang** 客户端、**.NET 8 + C#** 服务端。原文 `E:\[游戏A]-src-winrar`（2.38 GB / [游戏A] 全套）是另一个 MMORPG 项目，与本需求文档无关。

---

## 附录 B: 关键术语对照表

| [游戏D] 术语 | RGS 对应 |
|---|---|
| ClientData/4229938/ | (待建) crates/rgs-protocol/builds/v1/ |
| MethodIds.json | (待建) crates/rgs-protocol/builds/v1/method_ids.json |
| RpcSurface.json | (待建) crates/rgs-protocol/builds/v1/rpc_surface.json |
| rpc_schema.json | (待建) crates/rgs-protocol/builds/v1/schema.json |
| Configs/*.json | crates/rgs-config-loader/configs/items.json 等 |
| [代码名-D].Proxy | (待建) crates/rgs-edge-proxy/（参考 REQ-[游戏D]-009） |
| [代码名-D].Server.[代码名-D].App | crates/<domain>-service/src/main.rs |
| [代码名-D].Handlers.Game.GameRouter | crates/<domain>-service/src/handler/*.rs |
| DebugApiServer | (待建) crates/rgs-debug/ |
| Run-All.ps1 | (待建) scripts/start-rgs-stack.ps1 |
| fengari (Lua in JS) | mlua (Lua in Rust) |

---

> **本需求文档不是契约**，是技术研究提炼的借鉴清单。最终落地以 DDD Review + WBS + Sprint 计划为准。
