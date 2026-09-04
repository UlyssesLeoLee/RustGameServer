# RGS Plugin 集群 + App 集群架构 v0.1

> **创建日期**: 2026-09-05 06:30 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: 2026-09-05 06:24 JST 拍板 (scope_type=opt1 架构方案落档, granularity=Function 级别)
> **状态**: ⏳ 待 DDD Review 二审 (per AGENTS.md §3.x 二审流程)
> **关联**: RGS-INC-001 v0.2 §8/§9/§15 (function-plane mock 已落地, 本方案是 mock 演进方向)

## 0. 目标 (per 9/5 06:24 JST Ulysses 拍板)

> "现在的架构是由 plugin 集群组成的 app 集群吗？我需要后台有个运维界面，对每个 app 进行单独运维，允许各个功能点 app 单独更新，内部大多数内容可以 plugin 热插拔"

**核心需求**:
1. **plugin 集群 = app 集群的内部组成单元** (app = 6 域, plugin = 业务逻辑片段)
2. **后台运维 UI** (单个 app 维度操作: 上线 / 下线 / 灰度 / 回滚 / 看 metrics)
3. **每个 app 可独立更新** (单 app 升级不影响其它 app)
4. **内部 plugin 可热插拔** (WASM function register / unregister / version 切换不需要重启 app)

**粒度决策**: Function 级别 (WASM function, 被 app 内部调用), 不是 Service 级别 (整 app 替换)

## 1. 当前架构盘点 (per 9/4 git log `6c2a786` 基线)

### 1.1 已有的"准 plugin 集群"基础设施

`crates/function-plane/` (2,221 行, 11 .rs 文件) 是 **Function 级别 plugin 的雏形**:
- `contract.rs` (239 行) — FunctionMetadata / FunctionStatus / Runtime / TriggerType 类型
- `registry.rs` (216 行) — FunctionRegistry trait + InMemoryRegistry (mock)
- `wasm_host.rs` (289 行) — Wasmtime 嵌入 + fuel/epoch/memory limiter (mock)
- `gateway.rs` (114 行) — FunctionGateway facade (registry + host 组合)
- `error.rs` (61 行) — FunctionPlaneError

**状态**: mock 阶段, 没 PG 后端, 没 gRPC 端, 没后台 UI

### 1.2 当前 app 集群

- **5 域** (player / economy / match / social / admin) + **batch 域** (rgs-batch-backend)
- **平台层** (shared-platform / cluster-ops / gm-backend / function-plane)
- **工具** (rgs-testkit / rgs-arc-olu / rgs-certgen / rgs-hello / rgs-asset-download / rgs-overflow-alert)

### 1.3 当前运维入口

- `tools/gm-backend/` — GM 工具后端 (actix-web, RBAC, mTLS) — 给运营用
- `tools/gm-console/` — GM 工具前端 (envoy + dist/)
- `tools/rgs-batch-console/` — batch 域前端
- `tools/rgs-batch-backend/` — batch 域后端

**缺口**: 没有"对单个 app 独立运维"的概念, 也没有"对 function-plane plugin 运维"的概念。

## 2. 目标架构

### 2.1 三层模型

```
┌─────────────────────────────────────────────────────────────────┐
│ App 集群 (App Cluster)                                            │
│   = RGS 当前架构的 6 域 (player / economy / match / social /     │
│     admin / batch) + 平台层 (cluster-ops / shared-platform)     │
│   = Rust 编译产物, 一个 app 一个 binary, 一个 Pod                 │
│                                                                  │
│   职责: 业务主流程 + 数据持久 + 跨域 saga + gRPC 服务              │
│   升级方式: 整 Pod 重启 (k8s rolling update)                    │
└─────────────────────────────────────────────────────────────────┘
         ↑ 内部由 ↓
┌─────────────────────────────────────────────────────────────────┐
│ Plugin 集群 (Plugin Cluster)                                     │
│   = WASM function 集, register 到 function-plane registry        │
│   = 高频变动的业务逻辑片段                                         │
│                                                                  │
│   职责: 业务策略 / 公式 / 模板 / 配置算法                          │
│   升级方式: registry update, hot-swap, 不重启 app                 │
│                                                                  │
│   例子:                                                            │
│   - card-service 调 draw_card_probability 算抽卡                  │
│   - economy-service 调 price_curve 算拍卖底价                     │
│   - match-service 调 matchmaking_score 算匹配分数                  │
│   - social-service 调 guild_level_formula 算工会等级               │
└─────────────────────────────────────────────────────────────────┘
         ↑ 被 ↓
┌─────────────────────────────────────────────────────────────────┐
│ 后台运维界面 (Ops Console)                                       │
│   = apps 维度: list / deploy / rollback / metrics / logs          │
│   = functions 维度: list / register / version / pause / unreg    │
│   = apps + functions 联动: 看哪些 app 调了哪些 function            │
│                                                                  │
│   复用: tools/gm-backend + tools/gm-console 扩展, 不另起新项目     │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 plugin (Function 级别) 抽象

**plugin = 单个 WASM function + metadata + version**

```rust
// per function-plane/src/contract.rs 现状
pub struct FunctionMetadata {
    pub function_id: String,      // e.g. "card.draw_card_probability"
    pub version: String,          // semver-ish "v0.2.0"
    pub runtime: Runtime,          // Wasm | Container
    pub trigger_type: TriggerType, // Grpc | Http | Nats | Cron
    pub status: FunctionStatus,   // Draft | Active | Paused | Archived
    pub fuel: u64,                // per-call fuel cap
    pub memory_mib: u32,          // per-instance memory ceiling
    pub code_uri: String,         // 现阶段: s3://rgs-functions/{id}/{ver}.wasm
                                  // Phase 2: 容器镜像 uri
    pub hash_sha256: String,      // 内容 hash, 升级时校验
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub retired_at: Option<DateTime<Utc>>,
    pub owner: String,            // 域 Lead, e.g. "economy-Lead"
    pub config: HashMap<String, Value>, // 函数内可读配置 (per-function, 不需重编)
    pub input_schema_uri: Option<String>, // JSON Schema for InvocationRequest
    pub output_schema_uri: Option<String>,
}
```

**plugin 升级流程** (per 9/5 06:24 JST Ulysses "可单独更新"):

```
1. 域 Lead 在 ops console 选 function_id + 上传新 wasm + 设新 version
2. ops console 调 registry.register(NewMetadata) → 状态 Draft
3. ops console 调灰度策略 (per app label selector, e.g. "player-service-canary")
4. 域 Lead 验灰度 metrics ok → 调 set_status(Active) + set_old_status(Paused/Archived)
5. 全量切换 → 完成. 整个过程 app Pod 不重启.
```

**plugin 调度** (per function-plane/src/gateway.rs 现状):

```rust
// app 内部调用
let result = function_gateway.invoke("draw_card_probability", json!({"rarity": "SSR"}))
    .await?;
```

→ gateway 查 registry.get(_, version=None) → 拿最新 Active → WasmHost.instantiate → execute → result

## 3. 关键技术决策 (per 9/2 8/27 派生约束 + 9/4 git 实证)

### 3.1 plugin 升级机制: WASM hot-swap, 不重启 app

**现状**: function-plane mock 阶段, WasmHost 每次 invoke 拿新 Module (HashMap lookup)
**演进**: 升级时 registry 改 code_uri + hash, 旧 Module handle 在 WasmHost 内部 LRU 兜底
**决策**:
- 阶段 1 (mvp): in-memory registry + 单 app 进程内 hot-swap
- 阶段 2: PG registry (function_registry 表) + 跨 app 共享
- 阶段 3: gRPC/HTTP front (app 跨节点共享同一 registry) + KEDA scale

### 3.2 资源隔离: fuel + memory_mib per call (per function-plane/src/wasm_host.rs §9.5)

**现状**: Wasmtime 20 嵌入, fuel + epoch + MemLimiter
**演进**: 加 per-app 资源配额 (经济域 function 不能耗尽 player 域的 WASM 资源)
**决策**: 复用现有 MemLimiter, 加 AppResourcePool 抽象, 按 app label 分桶

### 3.3 plugin 失败处理: 降级到 native fallback

**风险**: plugin WASM crash → app 业务挂掉
**决策** (per 8/27 JST fail-closed 防线精神):
- invoke 返回 FunctionPlaneError 时, **app 必须有 native fallback** (不是 panic)
- registry 标 Paused 时, gateway 立即停止路由, 走 fallback
- DLQ (per shared-platform/src/dlq.rs) 接 WASM 异常 + payload, 离线分析

### 3.4 单 app 独立更新 (per 9/5 06:24 JST "对每个 app 单独运维")

**现状**: 5 域 binary 各 Pod rolling update, 互不依赖
**演进**:
- 单 app 升级时, k8s label selector 隔离 (per 8/27 JST 55.26 fail-closed 实践)
- 新版本 app 启动时, function-plane 走 "双 registry" 模式: 新 app 进程读自己带的 registry + 全局 registry
- 旧 app 进程继续服务, traffic 切换到新 app 后, 旧 app 退出

### 3.5 后台运维 UI (per 9/5 06:24 JST "后台有个运维界面")

**复用 tools/gm-backend + tools/gm-console**:
- gm-backend 加 `/ops/apps/*` endpoints (list / deploy / rollback / metrics)
- gm-backend 加 `/ops/functions/*` endpoints (list / register / version / pause)
- gm-console 加 "Ops" tab, 复用 ROPE_CS 9 页 (per gm-backend 描述)
- 不另起新项目 (per 9/5 拍板)

**RBAC 复用**:
- 域 Lead 只能运维自己域的 function (e.g. economy-Lead 不能 register card.*)
- Platform Lead (架构师) 可全权
- SRE 可 deploy / rollback 但不能改 function code

## 4. 落地路径 (4 阶段)

### 4.1 阶段 0: function-plane mock 升级 (per 8/27 JST 现状, 已完成)

- ✅ WASM 嵌入 (Wasmtime 20)
- ✅ Registry trait + InMemoryRegistry
- ✅ FunctionGateway facade
- ❌ PG registry backend
- ❌ gRPC front

### 4.2 阶段 1: Plugin 集群 MVP (估 ~2-3 周, 起 WBS)

- function-plane: PG registry (cluster_ops_db.function_registry 表)
- function-plane: gRPC front (FunctionRegistryService, FunctionInvokeService)
- shared-platform: 加 `app_resource_pool` 模块 (per-app 资源配额)
- gm-backend: 加 `/ops/functions/*` endpoints
- gm-console: 加 "Function Ops" tab
- 1 个 PoC: card-service 抽 draw_card_probability 改成 plugin 调用, native fallback

### 4.3 阶段 2: App 集群 + 单 app 独立更新 (估 ~3-4 周)

- cluster-ops: 加 `app_deployment` 抽象 (per-app k8s label selector)
- shared-platform: 加 `app_runtime` 模块 (启动时双 registry 模式)
- gm-backend: 加 `/ops/apps/*` endpoints
- 1 个 PoC: player-service 升级时, function-plane 切换 function 不中断

### 4.4 阶段 3: 平台化 (估 ~4-6 周)

- KEDA 弹性 (per function 流量)
- NATS JetStream 异步触发 (cron / 事件)
- Function Marketplace (跨域共享 function)
- AI Function Pool (per RGS-INC-001 v0.2 §15)

## 5. 跟当前架构的边界 (per 8/27 禁回溯叙事 + 8/21 5 域独立 Lead 原则)

| 当前实现 | 是否变 | 备注 |
|---|---|---|
| 6 域 gRPC 服务 | 不变 | 是 app 集群, 不下沉到 plugin |
| 跨域 saga (economy orchestrator) | 不变 | 是 native 业务主流程, 不下沉 |
| shared-platform (mTLS / outbox / RBAC) | 不变 | 是平台层, app 集群用 |
| function-plane mock | 升级 | 是 plugin 集群的核心, 演进到 PG + gRPC |
| cluster-ops | 扩展 | 加 app_deployment 抽象 |
| gm-backend / gm-console | 扩展 | 加 ops endpoints + UI tab |
| tools/rgs-flash-mock | 不变 | 是 mock, 跟新架构正交 |
| batch 域 | 不变 | 是 app 集群的一员 |
| 9/4 commit 11 个 D 决策 | 不追溯 | 全部保留 |

## 6. 跟 AGENTS.md 强约束对齐

- **§1.1 缺标比错标**: §4 阶段 0 已完成, 1-3 待 WBS
- **§1.2 env value 硬 ban**: GM ops UI 不能显示 env var 值
- **§2.1 L1/L1.1/L1.2**: 阶段 1-3 每步走 L1 三件套
- **§3 5 域独立 Lead**: plugin 升级需各域 Lead 签字 (per 5 域 Lead 决策原则, ROPE_CS 8 worker 教训)
- **§6.2 临时越界 + Ulysses 追认**: ops UI 写紧急 fix 时, Mavis 临时改 yaml 后 24h 内 commit + 追认
- **§8 团队偏好**: 1 人·周 ≈ 1M tokens 估算, 4 阶段总 ~ 10-14M tokens

## 7. 风险与备选 (per 8/27 JST Ulysses 拍板规则)

| 风险 | 缓解 |
|---|---|
| WASM 性能低于 native | 阶段 1 benchmark + 阶段 2 选择性下沉 (只对低频业务策略走 plugin) |
| plugin 故障传播 app | native fallback + DLQ + circuit breaker (per shared-platform) |
| Registry 变成单点 | PG + leader election (per shared-platform/subject) |
| 单 app 更新影响 function-plane | 双 registry 模式 + traffic 切换灰度 |
| ops UI 越权 | 复用 gm-backend RBAC (per gm-backend/src/rbac.rs) |

## 8. 留为下轮工单 (per 9/2 8/27 决策)

- 阶段 1 详细 WBS 任务清单 (5-10 个 task per phase)
- function-plane → PG migration 脚本
- 1 个 PoC plugin (draw_card_probability) 写 + test
- gm-backend ops endpoints 详细 OpenAPI spec
- gm-console "Function Ops" tab UX wireframe

## 9. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 06:30 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review 二审 | 起草, Function 级别 plugin 集群 + app 集群架构 |
