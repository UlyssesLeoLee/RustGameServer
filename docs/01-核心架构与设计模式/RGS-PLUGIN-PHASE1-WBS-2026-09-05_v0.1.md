# RGS Plugin 集群 + App 集群 阶段 1 WBS v0.1

> **创建日期**: 2026-09-05 07:20 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.2 + RGS-TEST-DESIGN-2026-09-05 v0.1 §2 + RGS-TEST-CASES-2026-09-05 v0.1 §3.2
> **状态**: ⏳ 待 DDD Review 二审
> **估时**: ~2-3 周 (per ARCH v0.1 §4.2 + token-OLU 1 人·周 ≈ 1M tokens, 估 8-12M tokens)

## 0. 阶段 1 目标

实现 plugin 集群 MVP, 落地 §4.2 ARCH 4 阶段路径中的阶段 1:
- function-plane: PG registry 后端 (替代 mock InMemoryRegistry)
- function-plane: gRPC front (FunctionRegistryService + FunctionInvokeService)
- shared-platform: app_resource_pool 模块 (per-app 资源配额)
- gm-backend: /ops/functions/* endpoints
- gm-console: Function Ops tab
- **1 PoC**: card-service 抽 draw_card_probability 改 plugin 调用 + native fallback

## 1. WBS 任务分解 (10 task, 估时/优先级/Owner)

### W1.1 function-plane PG registry schema (P0, 估 2 day, Owner: Platform Lead)

**任务**: 在 cluster_ops_db 加 function_registry 表, 镜像 RGS-INC-001 v0.2 §15.2 schema

```sql
-- cluster_ops_db.function_registry
CREATE TABLE function_registry (
    function_id        TEXT        NOT NULL,
    version            TEXT        NOT NULL,
    runtime            TEXT        NOT NULL,    -- 'Wasm' | 'Container'
    trigger_type       TEXT        NOT NULL,    -- 'Grpc' | 'Http' | 'Nats' | 'Cron'
    status             TEXT        NOT NULL,    -- 'Draft' | 'Active' | 'Paused' | 'Archived'
    fuel               BIGINT      NOT NULL DEFAULT 1000000,
    memory_mib         INTEGER     NOT NULL DEFAULT 32,
    code_uri           TEXT        NOT NULL,    -- s3://rgs-functions/{id}/{ver}.wasm
    hash_sha256        TEXT        NOT NULL,    -- content hash, upgrade 校验
    owner              TEXT        NOT NULL,    -- 域 Lead
    config             JSONB       NOT NULL DEFAULT '{}'::jsonb,
    input_schema_uri   TEXT,
    output_schema_uri  TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    activated_at       TIMESTAMPTZ,
    retired_at         TIMESTAMPTZ,
    PRIMARY KEY (function_id, version)
);
CREATE INDEX idx_function_registry_owner ON function_registry(owner);
CREATE INDEX idx_function_registry_status ON function_registry(status);
```

**DoD**:
- migration 文件: `crates/cluster-ops/migrations/0005_function_registry.sql`
- L1: `cargo check -p cluster-ops --tests` 0 error
- L1.1: `cargo test -p cluster-ops --lib` 通过 (含新 PG 仓库测试)
- 跨域 saga E2E: 不破现有 IT (per 9/1 8:00 JST 模板)

**风险**: cluster-ops 单测 PG mock (用 sqlx::test), 跨节点 schema 同步 (用 sqlx migrate)

### W1.2 function-plane PgFunctionRepository impl (P0, 估 2 day, Owner: Platform Lead)

**任务**: 实现 `PgFunctionRepository: FunctionRepository` trait (替代 mock InMemoryRegistry)

- 路径: `crates/function-plane/src/pg_repository.rs` (NEW)
- 依赖: sqlx 0.8 + cluster_ops_db (per W1.1)
- 实现: register / get / list_versions / set_status (4 method, 镜像 InMemoryRegistry)

**DoD**:
- L1: `cargo check -p function-plane --tests` 0 error
- L1.1: `cargo test -p function-plane --lib` 通过 (5+ 新 UT per method)
- 集成 IT: 跨 cluster_ops_db 真实 PG, register → get → set_status 流程通

### W1.3 function-plane gRPC FunctionRegistryService (P0, 估 2 day, Owner: Platform Lead)

**任务**: function-plane 加 gRPC server, 暴露 FunctionRegistryService (port 50090)

- 路径:
  - `crates/function-plane/proto/function_plane/v1/function_registry.proto` (NEW)
  - `crates/function-plane/src/grpc_service.rs` (NEW)
  - `crates/function-plane/src/main.rs` (NEW, 仿 5 域 main.rs 模板)
- proto RPC:
  - `RegisterFunction(FunctionMetadata) returns (RegisterResponse)`
  - `GetFunction(GetRequest) returns (FunctionMetadata)`
  - `ListVersions(ListRequest) returns (ListResponse)`
  - `SetStatus(SetStatusRequest) returns (SetStatusResponse)`
- 复用 service_bootstrap (per 0ccd74a)

**DoD**:
- L1: `cargo check -p function-plane --tests` 0 error
- L1.1: `cargo test -p function-plane --lib` 12+ tests (4 method × 3 case)
- 集成: grpcurl 调用 register / get / set_status 验证

### W1.4 function-plane gRPC FunctionInvokeService (P0, 估 3 day, Owner: Platform Lead)

**任务**: function-plane 加 FunctionInvokeService, 暴露 Invoke RPC

- 路径:
  - `crates/function-plane/proto/function_plane/v1/function_invoke.proto` (NEW)
  - `crates/function-plane/src/invoke_grpc.rs` (NEW)
- proto RPC:
  - `InvokeFunction(InvokeRequest) returns (InvokeResponse)`
  - 请求: function_id + version + input_json
  - 响应: output_json + trace_id + fuel_used

**DoD**:
- L1: `cargo check -p function-plane --tests` 0 error
- L1.1: `cargo test -p function-plane --lib` 8+ tests (invoke happy + 5 异常)
- 集成: PoC draw_card_probability v1.1.0 通过 InvokeFunction 调通, 验证 0.05 / 0.30 / 0.445 / 1.00 概率 (per regression-test-plugin-poc.sh §3)

### W1.5 shared-platform app_resource_pool 模块 (P1, 估 2 day, Owner: Platform Lead)

**任务**: shared-platform 加 app_resource_pool, 防止 per-app WASM 资源耗尽

- 路径: `crates/shared-platform/src/app_resource_pool.rs` (NEW)
- 设计: AppResourcePool { app_id → { fuel_budget, memory_budget, current_usage } }
- 接口: acquire(app_id, fuel, memory) / release(app_id, fuel, memory)
- 集成: WasmHost 调 acquire, invoke 完调 release

**DoD**:
- L1: `cargo check -p shared-platform --tests` 0 error
- L1.1: `cargo test -p shared-platform --lib` 6+ tests (2 app 竞争 + budget 耗尽 fail-closed)
- 文档: 资源隔离语义 (per 8/27 55.26 fail-closed 精神)

### W1.6 gm-backend /ops/functions/* endpoints (P1, 估 2 day, Owner: gm-backend Lead)

**任务**: gm-backend 加 Function Ops REST endpoints

- 路径: `crates/gm-backend/src/ops_functions.rs` (NEW)
- endpoints:
  - `POST /ops/functions/{function_id}/register` — 上传新 WASM + 设版本
  - `GET /ops/functions/{function_id}` — 查 metadata
  - `GET /ops/functions/{function_id}/versions` — 列所有版本
  - `POST /ops/functions/{function_id}/version/{version}/status` — 状态切换
  - `POST /ops/functions/{function_id}/invoke` — 直接调 (测试用)
- RBAC: 域 Lead 只能 register 自己域 function (per 8/21 JST 5 域独立 Lead 原则)
- 复用: gm-backend rbac + service_bootstrap (per 0ccd74a)

**DoD**:
- L1: `cargo check -p gm-backend --tests` 0 error
- L1.1: `cargo test -p gm-backend --lib` 8+ tests (5 endpoint × 至少 1 happy + 1 失败)
- RBAC 测试: 域 Lead 越权 register 跨域 → 403

### W1.7 gm-console Function Ops tab (P2, 估 2 day, Owner: gm-console Lead)

**任务**: gm-console 加 "Function Ops" tab, 调 gm-backend ops endpoints

- 路径: `tools/gm-console/src/ops-functions/` (NEW)
- 页面:
  - 列表: function_id / version / status / owner / activated_at
  - 详情: metadata + config + input/output schema
  - 操作: register / set_status (Active/Paused/Archived)
- 复用: 现有 ROPE_CS UI 框架 (per 9/4 gm-console 描述)
- 0 依赖 Node 22 + 原生 http (per rgs-web 母规范)

**DoD**:
- L1: `npm run build` 0 error
- L1.1: 手动 + 自动化 (Playwright) 测试 5 页面
- 集成: 端到端 register → set Active → 调 invoke 看响应

### W1.8 card-service 抽 draw_card_probability 改 plugin (P0, 估 2 day, Owner: card-Lead)

**任务**: card-service DrawCard 改用 FunctionGateway.invoke + native fallback

- 路径: `crates/card-service/src/service.rs` (改 DrawCard impl)
- 改动:
  1. 启动时构造 FunctionGateway (调 function-plane gRPC FunctionInvokeService)
  2. DrawCard 调 `gateway.invoke("draw_card_probability", input_json)`
  3. 失败 / Paused → `native_draw_card(rarity, pity)` fallback
- PoC 验证: registry v1.1.0 Active, 业务通过
- hot-swap: 上传 v1.2.0, set_status(Active) + set_old(Archived), 业务不重启

**DoD**:
- L1: `cargo check -p card-service --tests` 0 error
- L1.1: `cargo test -p card-service --lib` 现有 test 全过 + 2 新 test (plugin + fallback)
- 集成: PoC regression-test-plugin-poc.sh 全过

### W1.9 端到端 PoC 验证 (P0, 估 1 day, Owner: Platform Lead + card-Lead)

**任务**: 端到端跑通 PLUGIN-001 ~ 005 5 用例 (per RGS-TEST-CASES v0.1 §3.2)

- 场景: 5 域全栈 (function-plane PG + gRPC + card-service plugin + gm-backend ops + gm-console UI)
- 跑 `regression-test-plugin-poc.sh` + `regression-test-60-all-modules.sh` + `regression-test-error-codes.sh` + `regression-test-boundary.sh` + `regression-test-exception.sh`
- 期望: 5 脚本全过, mock_data fixture 全覆盖, function-plane registry 真实持久化
- k3s 部署: 1 namespace (rust-game-server) 部署 function-plane + card-service + gm-backend + gm-console

**DoD**:
- L1.2 (跨域 E2E): 5 脚本 verdict=PASS
- 文档: PoC runbook (部署 + 测试 + 监控)

### W1.10 文档 + DDD Review 收口 (P1, 估 1 day, Owner: Mavis)

**任务**: 写 PoC 落地报告, 二审收口

- 文档:
  - `docs/01-核心架构与设计模式/RGS-PLUGIN-PHASE1-CLOSEOUT-2026-09-XX_v0.1.md` (NEW)
  - 5 commit 摘要 + PoC 验证截图 + 下一步 (阶段 2 独立更新)
- 二审: per AGENTS.md §3.x DDD Review 二审流程
- 派生约束: L1/L1.1/L1.2 全过, 凭据永不打印, 缺标比错标

**DoD**:
- 文档 v0.1 落档
- Ulysses 二审签字
- 阶段 2 WBS 起草

## 2. 任务依赖图

```
W1.1 (PG schema)  ── W1.2 (PgFunctionRepository)  ── W1.3 (gRPC Registry)
                                                    │
                                                    ├── W1.4 (gRPC Invoke)
                                                    │       │
                                                    │       ├── W1.8 (card-service PoC) ──┐
                                                    │       │                             │
                                                    │       └── W1.9 (E2E PoC) ◀────────┘
                                                    │
                                                    ├── W1.5 (app_resource_pool) ◀──┐
                                                    │                                 │
                                                    ├── W1.6 (gm-backend ops) ◀──┐  │
                                                    │                            │  │
                                                    ├── W1.7 (gm-console UI) ◀─┐ │  │
                                                    │                          │ │  │
                                                    └── W1.10 (closout) ◀──────┴─┴──┘
```

**关键路径**: W1.1 → W1.2 → W1.3 → W1.4 → W1.8 → W1.9 → W1.10 (估 13 day 串行)
**并行加速**: W1.5 / W1.6 / W1.7 可在 W1.3 后并行 (估 5 day 实际墙钟)

## 3. 资源估算 (per 1 人·周 ≈ 1M tokens, RGS-TS-001 v0.4 §6.2)

| 任务 | 估时 | tokens | 复杂度 |
|---|---|---|---|
| W1.1 PG schema | 2 day | 200-600K | 低 (sqlx 已有) |
| W1.2 PgFunctionRepository | 2 day | 200-600K | 中 (新 trait impl) |
| W1.3 gRPC Registry | 2 day | 200-600K | 中 (新 proto + service) |
| W1.4 gRPC Invoke | 3 day | 300-900K | 高 (Wasmtime 调通) |
| W1.5 app_resource_pool | 2 day | 200-600K | 中 (新模块) |
| W1.6 gm-backend ops | 2 day | 200-600K | 中 (5 endpoint) |
| W1.7 gm-console UI | 2 day | 200-600K | 中 (前端) |
| W1.8 card-service PoC | 2 day | 200-600K | 中 (集成) |
| W1.9 E2E 验证 | 1 day | 100-300K | 中 (跑 5 脚本) |
| W1.10 文档收口 | 1 day | 100-300K | 低 (写文档) |
| **总计** | **19 day** | **1.9-5.7M tokens** | 估 2-3 周墙钟 |

**人天预算调整**: NFR-OP-010 (2 SRE ≤ 20 人·天/周) 不适用 AI 开发 (per 8/21 JST + 1 人·周 ≈ 1M tokens 规则)
**Owner 分配**: Platform Lead 7 task (W1.1-1.5, W1.9, W1.10) + card-Lead 1 task (W1.8) + gm-backend Lead 1 task (W1.6) + gm-console Lead 1 task (W1.7)

## 4. 风险与缓解 (per 8/27 55.26 fail-closed 精神)

| 风险 | 缓解 |
|---|---|
| Wasmtime 性能低于 native | 阶段 2 benchmark + 选择性下沉 (低频业务策略走 plugin) |
| plugin 故障传播 app | native fallback + DLQ + circuit breaker (per W1.8 + shared-platform/dlq.rs) |
| PG registry 变单点 | leader election + read replica (per 阶段 3) |
| 单 app 更新影响 function-plane | 双 registry 模式 (W1.5 资源隔离 + 阶段 2) |
| ops UI 越权 | 复用 gm-backend rbac (per W1.6 RBAC 测试) |
| WASM 资源泄漏 | per-app 配额 (W1.5) + fuel/epoch deadline (W1.4) |

## 5. 验收 (per 9/2 10:18 JST D2 拍板 + AGENTS.md §2.1 L1/L1.1/L1.2)

| 级别 | 标准 | 验证方法 |
|---|---|---|
| L1 (UT) | `cargo check --tests` 0 error (限时 60s) | 每 task 完 |
| L1.1 (Lib) | `cargo test --lib` 全过 (限时 120s) | 每 task 完 |
| L1.2 (E2E) | 5 回归脚本 verdict=PASS (限时 300s) | W1.9 完 |
| 阶段验收 | 10 commit + PoC 报告 + Ulysses 二审签字 | W1.10 完 |

## 6. 后续 (阶段 2 / 阶段 3, per ARCH §4.3-4.4)

- **阶段 2** (~3-4 周): 单 app 独立更新 (双 registry 模式) + traffic 灰度
- **阶段 3** (~4-6 周): KEDA 弹性 + NATS 异步触发 + Function Marketplace + AI Function Pool

## 7. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 07:20 JST | Ulysses — Mavis 接手 | ⏳ 待 DDD Review 二审 | 起草, 阶段 1 WBS 10 task + 依赖图 + 资源估算 |
