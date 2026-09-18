# RGS-BASIC-ANANTA-CBT3-INSPIRED v0.1 — 基本设计

**创建日期**: 2026-09-17 JST
**创建者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**依据**: `RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md`（借鉴需求汇总）
**作用域**: ANANTA 借鉴项目的架构级基本设计
**状态**: ⏳ 一审（Mavis 自审） / 待 Ulysses 二审
**下游**: 后续 RGS-DETAILED-*（每个 REQ 一份详细设计）

---

## 0. 文档元信息 / 代签

| 字段 | 值 |
|---|---|
| author | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 审批 | 架构师(Mavis 接手 agent per DEC-008) + 自审 2026-09-17 JST |
| 修订人 | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 创建日期 | 2026-09-17 JST |
| 基线 commit | D:\RustGameServer 当前 main (per 2026-09-17 JST) |
| 上游 REQ | `docs/14-项目管理/RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md` |
| 下游 DETAILED | 待拆分（每个 REQ 一份 v0.1） |
| 关联 | AGENTS.md §2 / §7 + 守门#9 DB 三分类横展原则 |

---

## 1. 设计目标 + 范围

### 1.1 设计目标

实现 REQ 文档中提炼的 **10 条借鉴需求**，分 3 个 Sprint 渐进落地，**Sprint N 优先**解决 RGS 长期最缺的"协议兼容性"基础设施。

### 1.2 范围

| Sprint | REQ 编号 | 详细度 |
|---|---|---|
| Sprint N（9 人日 / 2 周） | REQ-ANANTA-001 + REQ-ANANTA-004 | **详细设计** |
| Sprint N+1（14 人日 / 3 周） | REQ-ANANTA-002 + REQ-ANANTA-005 | 概要设计 |
| Sprint N+2（12 人日 / 2.5 周） | REQ-ANANTA-003 + REQ-ANANTA-006 + REQ-ANANTA-007 + REQ-ANANTA-008 | 概要设计 |
| 待评估 | REQ-ANANTA-009 + REQ-ANANTA-010 | 备注 |

### 1.3 不做什么（per REQ 文档 §5.4）

| 不借鉴 | 原因 |
|---|---|
| self-signed 证书 + hosts 劫持 | RGS 是生产级 mTLS，不需要 CBT 本地化 |
| 客户端 IL2CPP dump 工具 | Unity 专有，RGS 客户端栈可能不同 |
| ANANTA 的 Configs/ 195 MB 全部 JSON | RGS Master 表不需要全部 JSON，需评估 |
| fengari JS 版 Lua 解释器 | RGS 客户端不是 Lua 主导 |

---

## 2. 架构总览

### 2.1 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                  RustGameServer (5 域 + batch)              │
│                                                              │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐ │
│  │ player-svc │ │ economy-svc│ │ match-svc  │ │ social-  │ │
│  │            │ │            │ │            │ │ svc      │ │
│  └────────────┘ └────────────┘ └────────────┘ └──────────┘ │
│  ┌────────────┐ ┌────────────┐                               │
│  │ admin-svc  │ │ batch-svc  │  ← 6 域 (现成)                │
│  └────────────┘ └────────────┘                               │
│         │              │                                       │
│         └──────┬───────┘                                       │
│                │                                               │
│       ┌────────▼─────────┐                                     │
│       │ shared-platform  │  ← 平台层 (现成)                   │
│       │ + cluster-ops    │                                     │
│       └────────┬─────────┘                                     │
│                │                                               │
└────────────────┼────────────────────────────────────────────┘
                 │
   ┌─────────────┼─────────────┐  ← 本项目新增（借鉴 ANANTA）
   │             │             │
   ▼             ▼             ▼
┌─────────┐ ┌─────────┐ ┌─────────────────┐
│rgs-proto│ │ rgs-    │ │ rgs-debug       │
│  -dump  │ │ protocol│ │                 │
│(REQ-001)│ │/builds/ │ │ (REQ-005)       │
│         │ │(REQ-004)│ │                 │
│二进制反 │ │协议版本 │ │ HTTP debug      │
│向 schema│ │目录隔离 │ │ server +        │
│同步工具 │ │         │ │ DebugPanel      │
└─────────┘ └─────────┘ └─────────────────┘
                 │
        ┌────────▼─────────┐  ← 本项目新增
        │ rgs-config-loader│
        │   (REQ-002)      │
        │  业务配置 JSON    │
        │  直接进内存       │
        └──────────────────┘
```

### 2.2 模块关系

| 模块 | 依赖 | 被依赖 |
|---|---|---|
| `rgs-proto-dump` (REQ-001) | 第三方 binary parser（自研） | CI 工具 |
| `rgs-protocol/builds/` (REQ-004) | rgs-proto-dump | 5 域 + batch |
| `rgs-config-loader` (REQ-002) | 无（仅 serde + serde_json） | 5 域 Master 表读取 |
| `rgs-debug` (REQ-005) | shared-platform + rgs-testkit | ST 阶段 + 开发者本地 |

**关键约束**：新增模块**不破坏** RGS 现有 6 域 + 平台层 + 工具链。

### 2.3 技术栈总览

| 模块 | 语言 | 关键依赖 | 跟现有栈关系 |
|---|---|---|---|
| `rgs-proto-dump` | Rust | `clap` + `nom`（binary parser） | 新增独立二进制 |
| `rgs-protocol/builds/` | Rust + JSON | `serde` + `serde_json` | 增量，路径独立 |
| `rgs-config-loader` | Rust | `serde` + `serde_json` + `dashmap`（内存缓存） | 新增 lib crate |
| `rgs-debug` | Rust | `axum`（HTTP server） + `askama`（HTML 模板） | 新增独立二进制 |

**与现有栈差异**：
- rgs-web 用 Node.js（不需要 HTTP framework）
- gm-backend 用 actix-web 4（不需要再加 axum）
- **新增 axum 引入新依赖**，需要评估是否复用 actix-web 风格（见 §11.3）

---

## 3. Sprint N 详细设计（REQ-001 + REQ-004）

### 3.1 REQ-ANANTA-001: rgs-proto-dump

#### 3.1.1 架构

```
┌─────────────────────────┐    ┌──────────────────────────┐
│ 客户端 binary (Unity     │    │ 服务端 .proto (现有)     │
│ IL2CPP dump)             │    │                          │
│ D:\clients\*.dump        │    │ crates/rgs-protocol/proto│
└──────────┬───────────────┘    └────────────┬─────────────┘
           │                                  │
           ▼                                  ▼
   ┌────────────────┐                ┌─────────────────┐
   │ MethodIds.json │                │ .proto (compile) │
   │ (源数据)          │                │ → Rust 代码       │
   └────────┬───────┘                └────────┬────────┘
            │                                 │
            └──────────────┬──────────────────┘
                           ▼
              ┌────────────────────────┐
              │   rgs-proto-dump       │
              │   (新二进制)              │
              │   1. parse MethodIds   │
              │   2. parse .proto      │
              │   3. diff + 报告        │
              │   4. 输出 schema.json  │
              └────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │  builds/v1/             │
              │  ├── method_ids.json    │
              │  ├── rpc_surface.json   │
              │  └── schema.json        │
              │  (REQ-004 消费)         │
              └────────────────────────┘
```

#### 3.1.2 数据格式定义

```rust
// crates/rgs-proto-dump/src/types.rs

/// 单个 method 标识（来源：IL2CPP dump）
pub struct MethodId {
    pub name: String,
    pub id: u32,
    pub hex: String,    // "0x03C378C2"
}

/// MethodIds.json 顶层结构
pub struct MethodIdsFile {
    pub client_build: u32,
    pub source: String, // "Auto.Client/.../ServerMessageProcId.cs (IL2CPP dump, ...)"
    pub methods: Vec<MethodId>,
}

/// RpcSurface 单个 method
pub struct RpcMethod {
    pub name: String,
    pub method_id: u32,
    pub hex: String,
    pub kind: RpcKind,        // Notify / Return
    pub return_type: String,  // "void" / "int" / ...
    pub service: String,      // "IClientToGame" / ...
}

/// Diff 报告（CLI 输出 + 写日志）
pub struct DiffReport {
    pub only_in_client: Vec<MethodId>,  // 客户端发了服务端没声明
    pub only_in_server: Vec<RpcMethod>, // 服务端声明了客户端没发
    pub matched: usize,                  // 匹配数
    pub warnings: Vec<String>,
}
```

#### 3.1.3 CLI 设计

```bash
# 导出（从客户端 binary 提取）
rgs-proto-dump export \
    --binary D:/clients/client-4229938.dump \
    --output builds/v1/method_ids.json

# Diff（.proto vs MethodIds.json）
rgs-proto-dump diff \
    --proto crates/rgs-protocol/proto/ \
    --method-ids builds/v1/method_ids.json \
    --format json

# 校验（CI 集成）
rgs-proto-dump check \
    --proto crates/rgs-protocol/proto/ \
    --method-ids builds/v1/method_ids.json \
    --strict   # fail if mismatch
```

#### 3.1.4 ANANTA 兼容层（自动生成）

参考 ANANTA `RpcFrameDispatcher.cs` 的 `frame.Mode == 0x08` 处理（客户端未实现 invoke 自动 ack）：

```rust
// crates/rgs-network/src/compat.rs (由 rgs-proto-dump 生成)
pub fn handle_unimplemented_invoke(
    session: &mut Session,
    method_id: u32,
    invoke_id: i32,
) -> Result<()> {
    // 自动 ack 不 drop 整个 session
    session.return_async(method_id, invoke_id, 0, &[])?;
    Ok(())
}
```

#### 3.1.5 DoD

- ✅ L1（cargo check --tests）+ L1.1（cargo test --lib）通过
- ✅ **不破坏** 6 域现有 build
- ✅ CLI 三种命令（export / diff / check）跑通
- ✅ CI 集成示例（GitHub Actions workflow 片段）

### 3.2 REQ-ANANTA-004: rgs-protocol/builds/<version>/

#### 3.2.1 目录结构

```
crates/rgs-protocol/
├── builds/
│   ├── v1/                          # 客户端 build v1
│   │   ├── method_ids.json          # 4203 method (RE-001 dump 输出)
│   │   ├── rpc_surface.json         # 1475 RPC method
│   │   ├── schema.json              # 编译后的 schema (midToReader + midToName)
│   │   └── configs/                 # (待评估) 业务配置 JSON
│   ├── v2/                          # 客户端 build v2 (新)
│   │   └── ...
│   └── current -> v2                # symlink 指向当前 build
├── src/
│   ├── lib.rs                       # 暴露 Build enum + 路由
│   ├── router.rs                    # 按 client_build 路由到对应 schema
│   └── compat.rs                    # REQ-001 生成的兼容层
└── Cargo.toml
```

#### 3.2.2 核心 API

```rust
// crates/rgs-protocol/src/lib.rs

/// 客户端 build 标识
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientBuild {
    V1,
    V2,
    // 未来 build 自动添加
}

impl ClientBuild {
    /// 从客户端 hello 消息识别 build 版本
    pub fn from_client_hello(hello: &[u8]) -> Result<Self> {
        // hello 包含 client_build 字段
        if hello.len() < 4 { return Err(...); }
        let build = u32::from_le_bytes([hello[0], hello[1], hello[2], hello[3]]);
        match build {
            4229938 => Ok(Self::V1),
            // 后续 build 加 match arm
            _ => Err(ProtocolError::UnknownBuild(build)),
        }
    }
}

/// 当前激活的 build（CI 维护）
pub fn current() -> ClientBuild {
    ClientBuild::V2 // 当前默认
}

/// 按 build 加载 schema
pub fn load_schema(build: ClientBuild) -> &'static Schema {
    match build {
        ClientBuild::V1 => &V1_SCHEMA,
        ClientBuild::V2 => &V2_SCHEMA,
    }
}
```

#### 3.2.3 协议路由流程

```
客户端 hello → build 识别
              ↓
        load_schema(build)
              ↓
       后续 RPC frame 按 schema 反序列化
              ↓
    客户端发了不属于该 build 的 RPC？
              ↓
    触发 REQ-001 的兼容处理（自动 ack）
```

#### 3.2.4 CI 维护流程

```yaml
# .github/workflows/protocol-build.yml
name: Update protocol build
on:
  workflow_dispatch:
    inputs:
      client_build:
        description: 'New client build number'
        required: true
jobs:
  dump:
    steps:
      - run: cargo run --bin rgs-proto-dump -- export --binary ${{ inputs.binary }} --output crates/rgs-protocol/builds/v${{ inputs.client_build }}/method_ids.json
      - run: cargo run --bin rgs-proto-dump -- diff --proto crates/rgs-protocol/proto/ --method-ids crates/rgs-protocol/builds/v${{ inputs.client_build }}/method_ids.json
      - uses: peter-evans/create-pull-request@main
        with:
          branch: protocol/v${{ inputs.client_build }}
          title: "protocol(v${{ inputs.client_build }}): update method_ids from IL2CPP dump"
```

#### 3.2.5 DoD

- ✅ L1（cargo check --tests）+ L1.1（cargo test --lib）通过
- ✅ **不破坏** 6 域现有 build
- ✅ ClientBuild::from_client_hello 单元测试通过
- ✅ 客户端 build v1/v2 都能跟服务端通信（IT 验证）
- ✅ CI workflow 脚本就位

---

## 4. Sprint N+1 设计（概要）

### 4.1 REQ-ANANTA-002: rgs-config-loader（业务配置 JSON）

**架构**：
```
configs/                         # 仓库根
├── items.json                   # Master 物品表
├── skills.json                  # Master 技能表
├── buffs.json                   # Master buff 表
├── vehicles.json                # Master 载具表
└── weapons.json                 # Master 武器表
        │
        ▼
crates/rgs-config-loader/
├── src/
│   ├── lib.rs                   # ConfigRegistry（lazy load + cache）
│   ├── items.rs                 # 物品类型定义
│   ├── skills.rs                # 技能类型定义
│   └── ...
└── Cargo.toml
```

**关键设计**：
- 启动时 lazy load 所有 Master JSON 到 `DashMap<u32, Arc<ItemConfig>>`
- 提供 `ConfigRegistry::get_item(id) -> Option<Arc<ItemConfig>>`
- **不改 DB**：Transaction 表（订单、邮件、审计）保留 Postgres
- **不改 Work**：session / 临时状态保留 Redis/内存

**关联 6 域迁移清单**：
| 域 | 当前 DB Master 表 | 改 JSON 后 |
|---|---|---|
| player | `player_master_item` | `configs/items.json` |
| economy | `economy_master_currency` | 保留（业务关键） |
| match | `match_master_map` | `configs/maps.json` |
| social | `social_master_guild_level` | 保留 |
| admin | `admin_master_permission` | 保留 |
| batch | `batch_master_task` | `configs/batch_tasks.json` |

**DoD**: L1 + L1.1 + L1.2（6 域 E2E 业务跑通）

### 4.2 REQ-ANANTA-005: rgs-debug

**架构**：
```
crates/rgs-debug/
├── src/
│   ├── lib.rs                   # DebugServer (HttpListener, 127.0.0.1 only)
│   ├── api.rs                   # JSON API（玩家列表 / RPC 流量 / Session）
│   └── panel.rs                 # embedded DebugPanel HTML
└── assets/
    └── debug-panel.html         # 内嵌 web UI
```

**关键设计**：
- 默认监听 `127.0.0.1:7878`（区别 rgs-web 8788 / gm-console 8080）
- `config.debug.enabled = false` 默认关闭
- 内嵌 `debug-panel.html`（跟 ANANTA `DebugPanel/index.html` 同款）
- 提供 JSON API：
  - `GET /api/sessions` —— 当前所有 Session
  - `GET /api/rpc-stats` —— RPC 流量统计
  - `GET /api/player/{pid}` —— 玩家状态
  - `GET /api/queue-stats` —— 队列长度

**配套 rgs-testkit 集成**：
- 测试场景自动启用 `DebugServer::start_for_test()`
- 测试代码通过 JSON API 验证状态

**DoD**: L1 + L1.1 + L1.2（ST 阶段跑通 debug panel）

---

## 5. Sprint N+2 设计（概要）

### 5.1 REQ-ANANTA-003: Rust trait 多文件实现规范化

**关键规则**（文档化）：
```rust
// crates/player-service/src/handler/mod.rs
pub trait PlayerHandler {
    async fn handle(&self, ctx: &RpcContext) -> Result<...>;
}

// crates/player-service/src/handler/login.rs       (3-5 KB)
impl PlayerHandler for PlayerService { /* login */ }

// crates/player-service/src/handler/inventory.rs   (5-10 KB)
impl PlayerHandler for PlayerService { /* inventory */ }

// crates/player-service/src/handler/profile.rs     (5-10 KB)
impl PlayerHandler for PlayerService { /* profile */ }
```

**6 域全部按此 pattern 重构**，文件命名 `<业务子域>.rs`，每个 5-15 KB。

**配套文档**：`docs/13-实施经验/RGS-HANDLER-PARTIAL-PATTERN_v0.1.md`

### 5.2 REQ-ANANTA-006: DB migration KNOWN_ISSUES 段

**格式约定**：
```sql
-- migrations/0001_init_player.sql
-- KNOWN_ISSUES:
--   1. player.profile.guild_id 不建 FK（social 域可能在 v2 才有 guilds 表）
--   2. player.inventory.slot_count 用 SMALLINT (2 bytes)，旧客户端用 TINYINT (1 byte)，迁移时需要兼容
-- ...
CREATE TABLE player.profile (...);
```

**配套**：DDD Review 二审流程新增"KNOWN_ISSUES 段完整性"检查项。

### 5.3 REQ-ANANTA-007: 玩法层纯数据驱动模式

**核心约束**：
- 玩法核心逻辑控制在 5-10 KB（参考 ANANTA `WebTraversal.cs` 3 KB）
- 所有数值 / ID / 阈值在 `configs/match/*.json`
- 配合 REQ-002 共用 `rgs-config-loader`

**改造范围**：match 域 + 战斗玩法层优先。

### 5.4 REQ-ANANTA-008: 一键启动编排器

**13 步流程**（per ANANTA `Run-All.ps1`）：

| 步骤 | 内容 |
|---|---|
| 1 | admin 提升 |
| 2 | 依赖恢复（cargo fetch + npm ci） |
| 3 | 陈旧进程清理 |
| 4 | hosts 重定向（如有需要） |
| 5 | 证书校验（mTLS 过期检测） |
| 6 | 完整 cargo check 验证 |
| 7 | 端口检测 |
| 8 | K8s 资源就绪检查 |
| 9 | clean rebuild |
| 10 | ST 镜像构建 |
| 11 | kubectl apply |
| 12 | 启动 DebugPanel（按需） |
| 13 | probe 验证 |

**文件**：`scripts/start-rgs-stack.ps1`

---

## 6. 待评估设计

### 6.1 REQ-ANANTA-009: saga-runtime 单 binary 多 server

**待评估项**：
- saga-runtime 当前是否已经支持多 server？
- K8s 部署模型下"单 binary 多 server" vs "多 svc 拆"取舍

**评估前置**：先看 `crates/saga-runtime/` 当前架构。

### 6.2 REQ-ANANTA-010: Lua via mlua 跨语言脚本

**待评估项**：
- `mlua` crate 维护活跃度（最近 commit 时间）
- 与现有 rgs-config-loader (REQ-002) 集成方式
- 是否需要 Lua sandbox 隔离

---

## 7. 数据模型（守门#9 DB 三分类横展原则）

### 7.1 Master / Transaction / Work 三分类

按守门#9（2026-09-01 18:30 JST 确立），所有 RGS 数据表必须显式归类：

| 分类 | 含义 | 适用场景 | RGS 当前实现 | 本项目调整 |
|---|---|---|---|---|
| **Master** | 参考数据，SCD 策略 | 物品/技能/buff/地图 | DB 表 | **部分改 JSON**（REQ-002） |
| **Transaction** | 事件流水，append-only | 订单/邮件/审计 | DB 表 | **保持** |
| **Work** | 作业中临时数据 | session / 临时状态 | Redis / 内存 | **保持** |

### 7.2 Master 表迁移清单（REQ-002 涉及）

| 域 | 表 | 现状 | 改 JSON 后 |
|---|---|---|---|
| player | `item_master` | Postgres | `configs/items.json` |
| player | `character_master` | Postgres | `configs/characters.json` |
| economy | `currency_master` | Postgres | 保留（业务关键） |
| economy | `shop_master` | Postgres | `configs/shops.json` |
| match | `map_master` | Postgres | `configs/maps.json` |
| match | `mode_master` | Postgres | `configs/modes.json` |
| social | `guild_level_master` | Postgres | 保留 |
| admin | `permission_master` | Postgres | 保留（安全关键） |
| batch | `task_template_master` | Postgres | `configs/batch_tasks.json` |

**判断标准**（什么 Master 表走 JSON）：
1. ✅ 走 JSON：只读参考、变更频率低、配置驱动
2. ❌ 保留 DB：业务事务关键（currency / permission）、需要强一致

### 7.3 Transaction 表（保持 Postgres）

| 域 | 表 | 备注 |
|---|---|---|
| player | `inventory` | 玩家物品，事务 |
| economy | `wallet_log` | 钱包流水 |
| match | `match_history` | 比赛历史 |
| social | `guild_event_log` | 公会事件 |
| admin | `audit_log` | 审计日志（per RGS-OPEN-QA-2026-08-27 §4.1 Q2） |

### 7.4 Work 表（保持 Redis / 内存）

| 域 | 表 | 存储 |
|---|---|---|
| player | `session_data` | Redis |
| economy | `pending_transaction` | Redis |
| match | `matchmaking_queue` | Redis |
| batch | `task_running_state` | 内存 |

### 7.5 横展清单（per 守门#9）

按守门#9，类似 X/Y/Z 多分类一律横展细化：

| 横展维度 | RGS 数据 |
|---|---|
| 时间维度 | 短期（session-bound Work）+ 长期（永久 Transaction/Master） |
| 一致性维度 | 强一致（DB Transaction）+ 最终一致（Redis Work）+ 只读（Master JSON） |
| 容量维度 | 小（DB Transaction）+ 中（Redis Work）+ 大（Master JSON 195MB 级） |
| 变更频率维度 | 永不变（Master）+ 偶尔变（DBA 操作）+ 频繁变（Work 自动过期） |

---

## 8. 接口契约

### 8.1 REQ-001 rgs-proto-dump CLI 接口

```rust
// crates/rgs-proto-dump/src/cli.rs
#[derive(Parser)]
pub enum Command {
    /// 从客户端 binary dump 提取 method_ids
    Export {
        #[arg(long)] binary: PathBuf,
        #[arg(long)] output: PathBuf,
    },
    /// Diff .proto vs MethodIds.json
    Diff {
        #[arg(long)] proto: PathBuf,
        #[arg(long)] method_ids: PathBuf,
        #[arg(long, default_value = "human")] format: DiffFormat,
    },
    /// CI 校验（fail if mismatch）
    Check {
        #[arg(long)] proto: PathBuf,
        #[arg(long)] method_ids: PathBuf,
        #[arg(long)] strict: bool,
    },
}
```

### 8.2 REQ-004 rgs-protocol 核心 API

```rust
// crates/rgs-protocol/src/lib.rs
pub enum ClientBuild { V1, V2 }

impl ClientBuild {
    pub fn from_client_hello(hello: &[u8]) -> Result<Self>;
}

pub fn current() -> ClientBuild;
pub fn load_schema(build: ClientBuild) -> &'static Schema;
```

### 8.3 REQ-002 rgs-config-loader 核心 API

```rust
// crates/rgs-config-loader/src/lib.rs
pub struct ConfigRegistry {
    items: DashMap<u32, Arc<ItemConfig>>,
    skills: DashMap<u32, Arc<SkillConfig>>,
    // ...
}

impl ConfigRegistry {
    pub fn load_all() -> Result<Self>;
    pub fn get_item(&self, id: u32) -> Option<Arc<ItemConfig>>;
    pub fn get_skill(&self, id: u32) -> Option<Arc<SkillConfig>>;
    pub fn reload(&mut self) -> Result<()>;  // hot reload 支持
}
```

### 8.4 REQ-005 rgs-debug HTTP API

```rust
// crates/rgs-debug/src/api.rs
#[derive(Serialize)]
pub struct DebugApi;

impl DebugApi {
    #[get("/api/sessions")]
    async fn list_sessions() -> Vec<SessionInfo>;

    #[get("/api/rpc-stats")]
    async fn rpc_stats() -> RpcStats;

    #[get("/api/player/{pid}")]
    async fn player_state(pid: u64) -> PlayerState;

    #[get("/api/queue-stats")]
    async fn queue_stats() -> QueueStats;
}
```

### 8.5 内部契约（不改外部）

本项目**不修改** RGS 现有公开 API：
- 6 域 gRPC 公开接口（player.proto / economy.proto 等）不变
- gm-backend HTTP API 不变
- rgs-web 前端 API 不变

仅**新增**内部契约（见 §8.1 - §8.4）。

---

## 9. 关键流程

### 9.1 启动流程（含 Sprint N 全部 4 条 REQ）

```
1. 加载 configs/*.json            ← REQ-002 rgs-config-loader
2. 加载 protocol/builds/v1/       ← REQ-004 rgs-protocol/builds/
3. 启动 6 域 gRPC server
4. 启动 DebugServer (可选)         ← REQ-005 rgs-debug
5. CI 流程：rgs-proto-dump check   ← REQ-001 rgs-proto-dump
```

### 9.2 协议兼容性检查流程

```
客户端 hello → 识别 build
              ↓
    ClientBuild::from_client_hello()
              ↓
    load_schema(build)
              ↓
    客户端发 RPC frame
              ↓
    按 schema 反序列化
              ↓
    客户端发了不属于该 build 的 method_id？
              ↓
    REQ-001 生成的兼容层：自动 ack + warning log
              ↓
    不 drop session，继续通信
```

### 9.3 配置加载流程（REQ-002）

```
启动时 lazy load
    ↓
configs/items.json → serde_json 反序列化 → DashMap<u32, Arc<ItemConfig>>
configs/skills.json → DashMap<u32, Arc<SkillConfig>>
    ↓
ConfigRegistry::get_item(id) → O(1) 读
    ↓
业务代码调用 ConfigRegistry::get_item() 
无需 DB 查询，无需 ORM
```

---

## 10. 部署模型

### 10.1 K8s 部署（沿用现有）

新增 4 个 deployment（如果需要）：
- `rgs-proto-dump` (CronJob，跑 diff)
- `rgs-protocol-builds` (ConfigMap 挂载 builds/ 目录)
- `rgs-config-loader` (initContainer 预加载 JSON)
- `rgs-debug` (Deployment, 默认 replicas=0，按需启用)

### 10.2 镜像构建

- `rgs-proto-dump`: 独立镜像（cargo run --bin rgs-proto-dump）
- 其他三个: 集成到现有 rgs-server 镜像（不需要独立镜像）

### 10.3 配置存储

- `protocol/builds/v1/` → ConfigMap（pod 启动时挂载）
- `configs/*.json` → ConfigMap 或 PV（取决于更新频率）

---

## 11. 已知限制

### 11.1 协议兼容性限制

- ANANTA 的 Unity IL2CPP dump 工具未公开（REQ-001 需自研 binary parser）
- ANANTA 协议格式细节（protobuf 自定义字段、压缩算法）未深入逆向
- 第一版 REQ-001 只支持"method id 表"对比，不支持"参数 schema 自动生成"

### 11.2 配置迁移风险

- REQ-002 涉及 6 域 Master 表迁移，需 DB Lead + 5 域 Lead 协调评审
- 部分 Master 表（如 `currency_master`）保留 DB，跨域一致性需手工保证

### 11.3 引入新依赖

- `axum` HTTP framework（REQ-005）—— 跟现有 `actix-web` (gm-backend) / Node http (rgs-web) 不同
- 需要评估：是否复用 actix-web 风格？或者 rgs-debug 独立即可？

### 11.4 不在 Sprint N 的限制

- REQ-007 玩法层纯数据驱动（Sprint N+2）—— 需要 match 域 Lead 配合
- REQ-009 saga-runtime 单 binary 多 server —— 待评估

---

## 12. 关联文档

| 文档 | 路径 | 关系 |
|---|---|---|
| REQ（上游） | `docs/14-项目管理/RGS-REFERENCE-ANANTA-CBT3-PRIVATE-SERVER_v0.1.md` | 本文档的输入 |
| ANANTA 源码（参考） | `D:\PrivateServer\` | 逆向分析对象 |
| AGENTS.md | `D:\RustGameServer\AGENTS.md` | 仓库级守门 |
| RACI（6 域） | `docs/14-项目管理/RGS-RACI-*-V1_*.md` | 6 域 Lead 协调 |
| WBS | `docs/14-项目管理/RGS-PM-001_WBS流程_v0.1.md` | 落地排期 |

### 12.1 后续 DETAILED 文档清单

| REQ | DETAILED 文档路径 |
|---|---|
| REQ-ANANTA-001 | `docs/14-项目管理/RGS-DETAILED-ANANTA-001-rgs-proto-dump_v0.1.md` |
| REQ-ANANTA-004 | `docs/14-项目管理/RGS-DETAILED-ANANTA-004-rgs-protocol-builds_v0.1.md` |
| REQ-ANANTA-002 | `docs/14-项目管理/RGS-DETAILED-ANANTA-002-rgs-config-loader_v0.1.md` |
| REQ-ANANTA-005 | `docs/14-项目管理/RGS-DETAILED-ANANTA-005-rgs-debug_v0.1.md` |
| REQ-ANANTA-003 | `docs/14-项目管理/RGS-DETAILED-ANANTA-003-handler-partial-pattern_v0.1.md` |
| REQ-ANANTA-006 | `docs/14-项目管理/RGS-DETAILED-ANANTA-006-db-known-issues_v0.1.md` |
| REQ-ANANTA-007 | `docs/14-项目管理/RGS-DETAILED-ANANTA-007-data-driven-gameplay_v0.1.md` |
| REQ-ANANTA-008 | `docs/14-项目管理/RGS-DETAILED-ANANTA-008-start-rgs-stack_v0.1.md` |
| REQ-ANANTA-009 | `docs/14-项目管理/RGS-DETAILED-ANANTA-009-saga-runtime-multi-server_v0.1.md`（待评估） |
| REQ-ANANTA-010 | `docs/14-项目管理/RGS-DETAILED-ANANTA-010-mlua-config_v0.1.md`（待评估） |

---

## 13. 修订历史

| 版本 | 日期 | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-17 JST | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 | 初版。架构总览 + Sprint N 详细设计（REQ-001 + REQ-004）+ Sprint N+1/N+2 概要 + 数据模型（守门#9 横展）+ 接口契约 + 关键流程 + 部署模型 + 已知限制 + 关联文档 |

---

## 附录 A: 模块依赖图

```
        ┌──────────────────────┐
        │  RGS 现有 6 域         │
        │  (player / economy /  │
        │   match / social /    │
        │   admin / batch)      │
        └──────────┬───────────┘
                   │ uses
        ┌──────────▼───────────┐
        │  shared-platform     │
        │  + cluster-ops       │
        └──────────┬───────────┘
                   │ uses
        ┌──────────▼───────────┐
        │  rgs-protocol/       │ ← REQ-004
        │  builds/v1/          │
        └──────────┬───────────┘
                   │ uses (CI)
        ┌──────────▼───────────┐
        │  rgs-proto-dump      │ ← REQ-001
        │  (独立二进制)         │
        └──────────────────────┘

        ┌──────────────────────┐
        │  RGS 现有 6 域         │
        └──────────┬───────────┘
                   │ uses
        ┌──────────▼───────────┐
        │  rgs-config-loader   │ ← REQ-002
        │  (lib crate)         │
        └──────────┬───────────┘
                   │ reads
        ┌──────────▼───────────┐
        │  configs/*.json      │
        │  (Master 表 JSON)    │
        └──────────────────────┘

        ┌──────────────────────┐
        │  RGS 现有 ST 工具链    │
        └──────────┬───────────┘
                   │ uses
        ┌──────────▼───────────┐
        │  rgs-debug           │ ← REQ-005
        │  (独立二进制)         │
        └──────────┬───────────┘
                   │ reads
        ┌──────────▼───────────┐
        │  DebugPanel/         │
        │  index.html          │
        │  (embedded)          │
        └──────────────────────┘
```

---

## 附录 B: 数据流图（REQ-001 + REQ-004 联动）

```
┌─────────────────┐
│ 客户端 binary    │
│ (IL2CPP dump)   │
└────────┬────────┘
         │ rgs-proto-dump export
         ▼
┌─────────────────┐
│ builds/v1/      │
│ method_ids.json │
└────────┬────────┘
         │ rgs-proto-dump diff
         ▼
┌─────────────────┐         ┌─────────────────┐
│ Diff Report     │ ◄─────► │ .proto           │
│ (CI 输出)        │         │ (现有 5 域 + batch)│
└─────────────────┘         └─────────────────┘
         │
         ▼
┌─────────────────┐
│ builds/v1/      │
│ ├── method_ids  │
│ ├── rpc_surface │
│ └── schema.json │ ← rgs-proto-dump 输出
└────────┬────────┘
         │ load_schema(build)
         ▼
┌─────────────────┐
│ 客户端 hello     │
│ → ClientBuild   │
│ → load_schema   │
│ → RPC 处理       │
└─────────────────┘
```

---

> **本基本设计不是契约**，是架构级设计草案。最终落地以 DETAILED 设计文档 + WBS + Sprint 计划为准。
> Sprint N 优先解决 RGS 长期最缺的"协议兼容性"基础设施（REQ-001 + REQ-004）。