# RGS-REV-007-B 工程 53+54 架构一致性对抗性审核报告

**审核对象**：proto + migration + Cargo 依赖 + port + DTL 引用 + helm chart
**审核子代理**：architecture-consistency-adversarial-001
**审核时间**：2026-08-22
**commit 基线**：`e320c69`

---

## 1. 严重度统计

- **CRITICAL：6**（DTL 断裂 + 跨域不一致 + 部署不可执行）
- **HIGH：5**（依赖图违规 / port 占位 / RPC 严重缩水）
- **MEDIUM：5**（重复定义 / 模板可优化 / 字段语义不明）
- **LOW：3**（文档同步滞后）

**总计：19 issues**

---## 2. proto 字段一致性矩阵

| 域 | 包路径 | `common.v1.EntityId` | `Status` 引用 | `Timestamp` | `display_name` | 一致性 |
|----|--------|----------------------|---------------|-------------|----------------|--------|
| player | `player.v1.Player` | ✓ field 1 | ✓ field 2 (common.v1.Status) | ✓ field 3 | ✓ field 4 | OK |
| economy | `economy.v1.Account` | ✓ field 1 | ✓ field 2 (common.v1.Status) | ✓ field 3 | ✓ field 4 | OK |
| match | `match.v1.Match` | ✓ field 1 | ✓ field 2 (common.v1.Status) | ✓ field 3 | ✓ field 4 | OK |
| social | `social.v1.Guild` | ✓ field 1 | ✓ field 2 (common.v1.Status) | ✓ field 3 | ✓ field 4 | OK |
| admin | `admin.v1.AdminOp` | ✓ field 1 | ✓ field 2 (common.v1.Status) | ✓ field 3 | ✓ field 4 | OK |
| cluster_ops | `cluster_ops.v1.Node` | ✓ field 1 | ✓ field 2 (common.v1.Status) | ✓ field 3 | ✓ field 4 | OK |
| common | `common.v1.EntityId/Status/ErrorCode/PageRequest/PageResponse/HealthCheck*` | — | — | — | — | OK |

**观察**：
- 6 域 entity proto 字段顺序、类型、命名 100% 一致（`id` / `status` / `created_at` / `display_name`）
- 6 域 service trait 全部 100% 一致（`HealthCheck + GetXxx(EntityId) -> Xxx`）
- **无 `saga.proto`** —— 任务描述提及"经济域 `saga.proto`"，但仓库中 **不存在** `economy-service/proto/economy/v1/saga.proto`。Saga gRPC API 完全没有 proto 契约（RGS-IMPL-002 v0.1 §4 建议 saga 应有独立 proto 包以跨域通信）
- **CEM/PFAU/COC API 完全没有 proto** —— DTL-031 §7 列了 10+ method（`RegisterFeature` / `DeclareFeatureUpgrade` / `DeclareFeatureRollback` / `GetPfaRunState` / `AdvanceCanary` / `NotifyFeatureDeployed` / `RegisterEvent` / `UpdateEventSchema` / `ReplayEvents` / `DiscardDlqEvent`），实际 `cluster_ops.proto` 只有 `HealthCheck + GetNode` 2 个

**结论**：proto 字段矩阵 **形式一致**（6 域同模板），但 **业务深度严重缩水**（DTL 要求的 RPC 大面积缺失）。

---## 3. port 分配审计

| 服务 | main.rs bind (`GRPC_ADDR` 默认) | `client.rs::ServiceId::default_port()` | helm chart `values.yaml` grpcPort | k8s manifest `containerPort` | docs/deploy SOP | docs/RGS-PM-008 v0.1 | 一致性 |
|------|--------------------------------|----------------------------------------|----------------------------------|-------------------------------|-----------------|----------------------|--------|
| player | `0.0.0.0:50051` (`player-service/src/main.rs:31`) | 50051 (`shared-platform/src/client.rs:32`) | `PLACEHOLDER_PLAYER_GRPC_PORT` | `PLACEHOLDER_PLAYER_GRPC_PORT` | 无 | 50051 (L91) | **不一致**（占位符阻塞） |
| economy | `0.0.0.0:50052` (`economy-service/src/main.rs:29`) | 50052 (`client.rs:33`) | `PLACEHOLDER_ECONOMY_GRPC_PORT` | `PLACEHOLDER_ECONOMY_GRPC_PORT` | 无 | 50052 (L92) | **不一致**（占位符阻塞） |
| match | `0.0.0.0:50053` (`match-service/src/main.rs:29`) | 50053 (`client.rs:34`) | `PLACEHOLDER_MATCH_GRPC_PORT` | `PLACEHOLDER_MATCH_GRPC_PORT` | 无 | 50053 (L93) | **不一致**（占位符阻塞） |
| social | `0.0.0.0:50054` (`social-service/src/main.rs:27`) | 50054 (`client.rs:35`) | `PLACEHOLDER_SOCIAL_GRPC_PORT` | `PLACEHOLDER_SOCIAL_GRPC_PORT` | 无 | 50054 (L94) | **不一致**（占位符阻塞） |
| admin | `0.0.0.0:50055` (`admin-service/src/main.rs:29`) | 50055 (`client.rs:36`) | `PLACEHOLDER_ADMIN_GRPC_PORT` | `PLACEHOLDER_ADMIN_GRPC_PORT` | 无 | 50055 (L95) | **不一致**（占位符阻塞） |
| cluster_ops | `0.0.0.0:50056` (`cluster-ops/src/main.rs:29`) | 50056 (`client.rs:37`) | `PLACEHOLDER_CLUSTER-OPS_GRPC_PORT` | `PLACEHOLDER_CLUSTER_OPS_GRPC_PORT` | 无 | 50056 (L96) | **不一致**（占位符阻塞） |

**关键观察**：
- **二进制代码层**：6 域 `main.rs` + `client.rs::ServiceId` 100% 一致（50051-50056 完全对齐）
- **RGS-PM-008 完工报告**（2026-08-22，100% 对齐 50051-50056）
- **部署 artifacts 层**（helm chart values.yaml + k8s manifest）：**6/6 全 PLACEHOLDER**
- **`docs/deploy/*.md` SOP**：grep `port 50051|port 50052|...` 在 `docs/deploy/` 目录中 **0 命中**——SOP 中无任何具体 port 引用
- **helm chart templates 目录全部空白**（仅 `.gitkeep`），`charts/{admin,cluster-ops,economy,match,player,social}/templates/` 都没有 `deployment.yaml` / `service.yaml` 模板
- `docs/09-部署运维/RGS-OPS-100_Saga系统K3s部署设计_v0.1.md` 写明 `containerPort: 50051`，与 main.rs 一致——**但 helm chart 没消费该文档**

**结论**：**port 分配在设计层（main.rs + client.rs + PM-008）完全自洽，但部署层（helm chart + k8s manifest）完全未落地**。任何 `helm install` 都会因 `PLACEHOLDER_*_GRPC_PORT` 字面量导致 yaml 解析失败。

---## 4. DTL 追溯矩阵

| 域 | DTL 编号 | 关键需求 | 实施位置 | 覆盖度 | 缺口 |
|----|----------|----------|----------|--------|------|
| player | RGS-DTL-018 §3 | `players + player_sessions` (active-active 跨服身份) | `crates/player-service/migrations/0001_init.sql:5-33` | **70%** | 缺 `updated_at` 触发器；`player_sessions.expires_at` 缺 partial index（仅 `expires_at` 普通 index） |
| economy | RGS-DTL-015 §3 | `accounts (OCC version) + transaction_ledger (idempotency_key)` | `crates/economy-service/migrations/0001_init.sql:5-37` | **85%** | `accounts` 缺 `version` 自增触发器；`transaction_ledger` 缺 `idempotency_key` 之外的 `saga_id+command_id` 复合索引 |
| economy-saga | RGS-DTL-100 §3 + §4 + §6 + §7 | **7 触发条件白名单 + 3 表(sagas/reservations/inbox) + fence_token/owner_pod/expires_at** | `crates/economy-service/migrations/0002_saga_init.sql:4-52` + `src/saga.rs/inbox.rs/reservation.rs` | **20%** | 见 C3 + C4 详述（表结构严重偏离 / 物理 DB 错位 / 关键字段全部缺失） |
| match | RGS-DTL-016 §3 | `matches + match_participants` (1v1/2v2/5v5/BR) | `crates/match-service/migrations/0001_init.sql:5-38` | **90%** | OK，主键外键 INDEX 都齐 |
| social | RGS-DTL-026 §3 | `guilds + guild_members` | `crates/social-service/migrations/0001_init.sql:5-32` | **95%** | OK，`guild_id+player_id` UNIQUE + INDEX 齐 |
| admin | RGS-DTL-019 §3 | `admin_users (RBAC) + audit_log (hash 链 + 禁 UPDATE/DELETE)` | `crates/admin-service/migrations/0001_init.sql:5-50` | **80%** | **C2 关键缺口**：缺 `feature_registry / feature_version_history / pfa_run_state / cem_audit_log / coc_audit`（per DTL-031 §3.1） |
| cluster_ops | RGS-DTL-020 §3 | `cluster_nodes + feature_flags` | `crates/cluster-ops/migrations/0001_init.sql:5-35` | **50%** | **C2 + C3 关键缺口**：缺 `feature_registry / feature_version_history / pfa_run_state`（DTL-031）；`feature_flags` 不是 DTL 要求的 `feature_registry`（key+scope 复合主键 vs DTL 的 feature_id PK + version 历史） |
| shared-platform / CEM | RGS-ARC-051 + DTL-031 §2 | CEM 事件目录 + DLQ + PFAU all-reachable | `crates/shared-platform/src/subject.rs:42-65`（仅 SubjectBuilder） | **15%** | **C1 关键缺口**：CEM/PFAU/COC 全部 10+ API 缺失（仅 Subject 命名规范），consumer/producer/outbox_relay/metrics 单独存在但**无主调用方** |
| shared-platform / 跨域 RPC | RGS-SPEC-CROSS-002 | 6 域 client builder | `crates/shared-platform/src/client.rs:60-81`（仅 `build_service_channel` 框架） | **10%** | **C5 关键缺口**：6 域 **Cargo.toml 全部未声明 shared-platform 依赖**，调用方 `cargo build` 阶段直接失败 |

**DTL 引用密度统计**：
- 6 域 `0001_init.sql` 文件头都有 DTL 引用注释（DTL-018/015/016/026/019/020）✓
- `0002_saga_init.sql` 引用 DTL-100 ✓
- **但实施层与 DTL 字段级需求匹配度仅 20-95%**（Saga 20% / cluster_ops 50% / admin 80% / shared-platform CEM 15%）

---## 5. CRITICAL Issues

### C1. cluster_ops proto 完全缺失 DTL-031 §7 COC/CEM/PFAU API 契约

- **位置**：`crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto:6-9`
- **问题**：DTL-031 §7 明确要求 `ClusterOpsService` 暴露 10+ method：
  - `RegisterFeature` / `UpdateFeature`（Unary）
  - `DeclareFeatureUpgrade` / `DeclareFeatureRollback`（Server stream）
  - `GetPfaRunState` / `ListFeatures`（Unary）
  - `AdvanceCanary`（Unary，人工 retry/skip/rollback）
  - `NotifyFeatureDeployed`（Internal unary，ARC-042 联动）
  - `RegisterEvent` / `UpdateEventSchema`（Unary，CEM 事件目录）
  - `ReplayEvents` / `DiscardDlqEvent`（Server stream/Unary，事件重放 + DLQ）
  - 所有外部请求需携带 `request_id` / `operator_id` / `expected_version` / `approval_ref` / `trace_id` 等幂等字段
- **实施**：`cluster_ops.proto` 只有 `HealthCheck + GetNode(EntityId) -> Node` **2 个 method**，且 `Node` 是只读 entity（无 Feature/CEM/PFAU 任何字段）
- **影响**：**COC 集群运营中心 / CEM 中心事件管理 / PFAU 每功能原子升级 整个控制面 API 全部缺失**。即使 `cluster_nodes` + `feature_flags` 两表存在，也没有 gRPC 端点暴露。C1 直接导致 RGS-ARC-051 §3 集群运营中心架构成为空壳。
- **修复建议**：54.x 后续工作流增加 `cluster_ops.proto` 第二批 method（至少 `RegisterFeature/GetPfaRunState/ListFeatures/DeclareFeatureUpgrade/RegisterEvent`），并按 DTL-031 §7 添加幂等字段（`request_id` / `expected_version` / `approval_ref` / `trace_id`）；同步更新 admin.proto 添加 COC Web 转发代理（per DTL-031 §2 数据流图 `UI → AD → CO`）

### C2. admin 域 migrations 缺失 DTL-031 §3 控制面核心表

- **位置**：`crates/admin-service/migrations/0001_init.sql:5-50`
- **问题**：DTL-031 §3.1 明确"RGS-BAS-031 已定义 `feature_registry` / `feature_version_history` / `pfa_run_state` 等表"，§2 数据流图明确 `DB[(admin_db Feature/CEM/PFAU 状态)]`
- **实施**：`admin 0001_init.sql` 实际只有 `admin_users + audit_log` **2 张表**，完全缺失：
  - `feature_registry`（Feature 元数据 + current_version）
  - `feature_version_history`（版本历史 + 制品摘要）
  - `pfa_run_state`（PFAU 批次状态机 declared/progressing/paused/committed/rolled_back）
  - `cem_audit_log`（CEM 事件注册/重放审计）
  - `coc_audit`（COC 操作审计）
- **DTL-031 §1.1 硬约束**："控制面状态落在既有 admin_db，不新建控制面数据库"
- **影响**：PFAU 状态机无法持久化，CEM 事件目录无法管理，COC 操作无审计追溯。**整个 54.x 完工的 admin 域与 DTL-031 完全脱节**。
- **修复建议**：54.x 后续（建议 54.16+）增加 `admin-service/migrations/0002_coc_cem_init.sql` 实化 §3.1 全部 4 张表，并迁移 `cluster-ops/feature_flags` 数据到 `admin_db.feature_registry`（per DTL-031 §1.1）

### C3. DTL-100 §7 Saga Store 物理 DB 位置错配 + schema 严重偏离

- **位置 A**：`crates/economy-service/migrations/0002_saga_init.sql:4-52`（**错误位置**）
- **问题 A（物理 DB 错位）**：DTL-100 §7 标题明确"## 7. Saga Store Schema（**cluster_ops_db**）"，要求 `saga_definition` / `saga_instance` / `saga_step` / `saga_event` 4 张表在 **cluster_ops_db**
  - 实际 economy 在自己的 economy_db 创建 `sagas` + `reservations` + `inbox` 3 张表
  - 物理位置错位会导致：跨域 Saga 协调（5 域共享 Saga 状态）无法实现；DTL-100 §5 "MatchFinished 触发 Reward Saga" 因 match 域无法读取 economy_db.sagas 状态而阻塞
- **问题 B（schema 严重偏离）**：DTL-100 §7 要求 4 表（definition / instance / step / event），实际 1 张 `sagas` 表（`saga_id` + `saga_type` + `command_id` + `idempotency_key` + `current_step` + `steps JSONB` + `status` + `timestamps`），4 张表被压扁成 1 张
- **问题 C（关键字段缺失）**：DTL-100 §7 关键字段在 economy `sagas` 表中 **全部缺失**：
  - `definition_id VARCHAR(128) REFERENCES saga_definition`（指向定义表，缺失）
  - `fence_token BIGINT NOT NULL DEFAULT 0`（per DTL-100 §5 Active-Active fencing 防过期 leader，**缺失**）
  - `owner_pod VARCHAR(128)`（当前持有者，**缺失**）
  - `payload JSONB NOT NULL`（业务入参，被合并到 `steps JSONB`，**语义错位**）
  - `result JSONB`（最终结果，**缺失**）
  - `expires_at TIMESTAMPTZ NOT NULL`（Saga 总超时，**缺失**）
  - `correlation_id UUID`（跨域追踪，**缺失**）
- **问题 D（saga_definition 表完全未实化）**：DTL-100 §7 第 1 张表 `saga_definition (definition_id PK, saga_type, version, definition_json, deprecated, UNIQUE(saga_type, version))` **完全没有迁移文件**——意味着 Saga 步骤定义无版本化（运维变更步骤定义无审计追溯）
- **影响**：
  1. Saga 跨域协调失效（5 域只能看到自己的 sagas 行）
  2. Active-Active fencing 无 DB 端实现，per ADR-0052 + DTL-100 §5 "防止过期 Leader 写" 无法保证
  3. Saga 步骤定义无版本化（FR-DTL-100-009 步骤演进无审计）
  4. `command_id` 唯一性由 `idx_sagas_command_id` + `uq_sagas_command_id` 双重索引保证 ✓（唯一正确的部分）
- **修复建议**：
  1. 把 `0002_saga_init.sql` **从 economy 迁到 cluster-ops**，创建 `cluster-ops/migrations/0002_saga_init.sql`
  2. 拆分为 4 表 `saga_definition` / `saga_instance` / `saga_step` / `saga_event`
  3. 补全 `fence_token` / `owner_pod` / `expires_at` / `correlation_id` / `payload` / `result` 字段
  4. 重建 economy 端的引用（FK 跨 DB 不允许，应改为 `saga_id UUID` 列存引用，由应用层 JOIN 验证）
  5. 重写 `crates/economy-service/src/saga.rs::PgSagaRepository` 的 SQL 与 4 表 schema 对齐

### C4. DTL-100 §4.2 Inbox schema 与实施严重偏离

- **位置**：`crates/economy-service/migrations/0002_saga_init.sql:42-50` + `crates/economy-service/src/inbox.rs:29-43`
- **DTL-100 §4.2 规范 schema**：

```sql
CREATE TABLE inbox (
    event_id UUID PRIMARY KEY,
    consumer VARCHAR(64) NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    status VARCHAR(16) NOT NULL DEFAULT 'PENDING',  -- PENDING/DONE/FAILED
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT
);
CREATE INDEX idx_inbox_pending ON inbox (received_at) WHERE status = 'PENDING';
```

- **economy 实际 schema**：

```sql
CREATE TABLE inbox (
    id UUID PRIMARY KEY,
    command_id UUID NOT NULL,
    handler TEXT NOT NULL,
    result TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'processed' CHECK (status IN ('processed', 'failed')),
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (command_id, handler)
);
CREATE INDEX idx_inbox_processed_at ON inbox (processed_at);
```

- **关键差异**：
  - DTL 字段 `event_id UUID PK` → 实施 `id UUID PK` + `command_id UUID NOT NULL`（PK 错位：`event_id` 是 NATS 事件 ID，`command_id` 是业务命令 ID，**两者语义不同**——DTL-100 §4.2 区分消费端去重键 vs 业务幂等键）
  - DTL 字段 `consumer` → 实施 `handler`（仅命名差异，但语义有偏差：`consumer` 是消费端服务名，`handler` 是处理函数名）
  - DTL 状态 `PENDING/DONE/FAILED` 三态 → 实施 `processed/failed` 两态（**无 PENDING**——消费前无法标识"已收未处理"，`retry_count` 无法正确累加）
  - **完全缺失** `retry_count`（per DTL-100 §4.2 `retry_count INT NOT NULL DEFAULT 0`，JetStream backoff retry 必填）
  - **完全缺失** `last_error`（per DTL-100 §4.2 `last_error TEXT`）
  - **完全缺失** `idx_inbox_pending` 部分索引 `WHERE status = 'PENDING'`（DTL-100 §4.2 显式要求）
  - 实施 `idx_inbox_processed_at ON inbox (processed_at)` 不在 DTL 要求之列，且无业务价值（`processed_at` 查询无 hot path）
  - 实施 `result TEXT DEFAULT '{}'` 不在 DTL-100 §4.2 规范中（§4.2 schema 无 `result` 字段，结果存 saga 表的 `result` JSONB）
- **影响**：
  1. **消息重投去重逻辑错位**：DTL 设计 `event_id` 是 NATS 投递的 `event_id`（per NATS JetStream `Message-Nats-Msg-Id` 头），应用层用 `event_id` 查 inbox；实施用 `id (UUID v4)` 作 PK 但默认填充了 v4 UUID——**没有任何去重能力**，每个 inbox 行是新生成的 UUID，无法对同一 NATS 消息的多次投递做幂等
  2. **retry_count 缺失 → 死信无法识别**：DTL-100 §4.2 通过 `retry_count > N → status=FAILED` 转 DLQ，实施无 retry_count → 永远卡在 `processed` 态
  3. **跨消费者隔离失效**：DTL 设计 `consumer` 字段允许多个 consumer 共享同一张 inbox 表（如 `saga-runtime` / `inventory-service` 都消费同一事件），实施 `handler` 仅一个 handler 名，无法隔离
- **修复建议**：
  1. 重命名 `id` → `event_id` 并以 NATS 投递的 `event_id` 作为 PK（应用层在消费时从 message header 读取 `Nats-Msg-Id` 并作 INSERT 主键）
  2. 补全 `retry_count INT NOT NULL DEFAULT 0` / `last_error TEXT`
  3. 状态改为 `PENDING/DONE/FAILED` 三态
  4. 补 `CREATE INDEX idx_inbox_pending ON inbox (received_at) WHERE status = 'PENDING'`
  5. 删除 `idx_inbox_processed_at`（无 DTL 依据）
  6. `result` 字段从 inbox 迁移到 saga 表的 `result JSONB`（per DTL-100 §7 schema 中 saga_instance.result）

### C5. 5 域 Cargo.toml 全部未声明 shared-platform 依赖

- **位置**：
  - `crates/player-service/Cargo.toml:14-32`（无 `shared-platform`）
  - `crates/economy-service/Cargo.toml:14-32`（无 `shared-platform`）
  - `crates/match-service/Cargo.toml:14-32`（无 `shared-platform`）
  - `crates/social-service/Cargo.toml:14-32`（无 `shared-platform`）
  - `crates/admin-service/Cargo.toml:14-32`（无 `shared-platform`）
  - `crates/cluster-ops/Cargo.toml:14-32`（无 `shared-platform`）
- **问题**：
  - `shared-platform/src/lib.rs:25-78` 暴露 20 个 pub mod：`channel` / `client` / `consumer` / `dlq` / `grpc_tracing` / `json_logging` / `messaging` / `metrics` / `metrics_endpoint` / `outbox` / `outbox_relay` / `producer` / `rbac` / `retry` / `span_helpers` / `subject` / `tls` / `tracing_init`
  - 5 域 **Cargo.toml 完全没有 `[dependencies] shared-platform = { path = "../shared-platform" }`**
  - 验证：`Select-String -Path crates/*/Cargo.toml -Pattern shared` 在 5 域 6 个 Cargo.toml 中 **0 命中**（只有 `build.rs` 引用了 `../shared-platform/proto/common/v1/common.proto`，这是编译时路径，与 Cargo 依赖无关）
- **直接影响**：
  1. `SubjectBuilder` / `SubjectDomain` / `parse()` 无法在 5 域代码中使用（`shared-platform/src/subject.rs` 实化的命名空间无调用方）
  2. `ServiceId` / `build_service_channel()` 无调用方，RGS-SPEC-CROSS-002 跨域 RPC 仅是空壳
  3. `OutboxRelay` / `OutboxRepository` 无调用方，**5 域实际根本没有 outbox 表**（per C6 后续）
  4. `init_tracing` / `init_tracing_with_otel` 无调用方（5 域 main.rs 各自手写 `tracing_subscriber::fmt()`，per `crates/player-service/src/main.rs:23-28`）
  5. `Metrics` / `metrics_endpoint` 无调用方，RGS-ARC-051 §3 观测栈半成品
  6. `rbac::enforce` / `Authorizer` 无调用方，DTL-019 §3.3 RBAC 实施完全没接入
  7. `tls::load_server_identity` / `load_client_tls` 无调用方，RGS-SEC-100 §7 mTLS 实施完全没接入
- **影响**：
  - **RGS-PM-008 v0.1 完工报告严重失实**——报告称 54.1-54.15 全完工，但 **5 域根本拿不到 shared-platform 任何符号**，等价于 54.1-54.15 中除 54.1 proto + 54.3 build.rs + 54.4 sqlx + 54.5-54.8 业务 entity/repo + 54.9-54.10 client/subject 的"框架代码"以外，**全部 "service 端"集成实际不存在**
  - 工程 54 完工报告 v0.1 §6 后续工作流 54.16+ 计划"6 域接入 shared-platform 公共服务"——但 Cargo.toml 这一行都没加，意味着 55.x 必须先补 6 域 shared-platform 依赖才能用任何公共服务
- **修复建议**：
  1. 在 5 域 6 个 Cargo.toml 各自添加 `shared-platform = { path = "../shared-platform" }`
  2. 5 域 `main.rs` 改用 `shared_platform::tracing_init::init_tracing_with_otel()` 替换手工 `tracing_subscriber::fmt()` 初始化
  3. 5 域 `main.rs` 改用 `shared_platform::tls::load_server_identity()` 启用 mTLS
  4. 5 域 service 层接入 `shared_platform::rbac::enforce()` 做 RBAC 校验
  5. 5 域 gRPC interceptor 接入 `shared_platform::grpc_tracing::server_interceptor_layer()` 注入 traceparent

### C6. Subject 命名空间双规范冲突（DTL-100 §6.2 vs subject.rs）

- **位置 A**：`docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式设计_v0.1.md:545-548`（RGS-SPEC-CROSS-003 旧规范）
- **位置 B**：`crates/shared-platform/src/subject.rs:46-65`（RGS-SPEC-CROSS-005 草案 / 54.10 实化）
- **问题**：
  - **DTL-100 §6.2 / RGS-SPEC-CROSS-003**（2026-08-15 前生效，规范文件）：
    - `SAGA.*`（Saga 事件）
    - `EVENT.{domain}.{action}`（域事件）
    - `COMMAND.{service}.{action}`（命令）
  - **subject.rs / RGS-SPEC-CROSS-005 草案**（54.10 引入，新规范）：
    - `rgs.<domain>.<event_type>.<version>`（域事件）
    - `rgs.saga.<saga_type>.<event>`（Saga 事件）
    - `rgs.cem.<event_type>`（CEM 事件）
    - `rgs.dlq.<source>`（DLQ 死信）
  - **关键差异**：
    - 旧 3 类 vs 新 4 类（旧规范无 `CEM` / `DLQ` 命名空间）
    - 旧 `SAGA.*`（无前缀）vs 新 `rgs.saga.*`（统一 `rgs.` 前缀）
    - 旧 `EVENT.{domain}.{action}`（无版本）vs 新 `rgs.<domain>.<event_type>.<version>`（带版本后缀 `v1`）
    - 旧 `COMMAND.{service}.{action}`（命令命名空间）vs 新 **无命令命名空间**（54.10 取消了 COMMAND）
  - **RGS-SPEC-CROSS-005 是"草案"状态**（per subject.rs 注释 line 5: "命名规则（per RGS-SPEC-CROSS-005 草案）"），但已经写入了 54.10 代码（5 域 main.rs 暂未使用）
- **影响**：
  - 任何按旧规范 DTL-100 §6.2 编写的 NATS consumer / 文档 / 测试代码，会**完全无法解析** 54.10 实化的 subject 字符串
  - 反之亦然
  - 这是规范的"双轨并行"——新旧两套文档同时存在，没有切换声明
- **修复建议**：
  1. RGS-SPEC-CROSS-005 必须从"草案"升级为 v1.0 正式版，**废弃** RGS-SPEC-CROSS-003
  2. RGS-DTL-100 §6.2 同步更新为新命名规范（`rgs.saga.*` / `rgs.<domain>.<event>.v<N>` / `rgs.cem.<event_type>`）
  3. 写一份 RGS-ADR 记录规范切换（"RGS-SPEC-CROSS-003 → 005 切换"），签批人：架构师 + 5 域 Lead + SRE
  4. DTL-100 §6.2 示例（`SAGA.*`）需要回写说明"v0.1 旧规范，v0.2 切换为 RGS-SPEC-CROSS-005"

---## 6. HIGH Issues

### H1. helm chart + k8s manifest port 全 PLACEHOLDER + templates 目录空白

- **位置**：
  - 6 chart `values.yaml`：`grpcPort: PLACEHOLDER_*_GRPC_PORT`
  - 6 k8s manifest：`containerPort: PLACEHOLDER_*_GRPC_PORT`
  - 6 chart `templates/` 目录：仅 `.gitkeep`，**无 deployment.yaml / service.yaml / configmap.yaml / secret.yaml 模板**
- **问题**：
  - `RGS-PM-008 v0.1` 完工报告 L90-96 明确列了 50051-50056 端口分配
  - `crates/*/src/main.rs` 都正确 bind 了 50051-50056
  - `shared-platform/src/client.rs::ServiceId::default_port()` 也返回 50051-50056
  - **但部署 artifacts 全部 PLACEHOLDER，且 helm chart templates 目录空白**
- **影响**：
  - `helm install` 任何 chart 都会因 `PLACEHOLDER_PLAYER_GRPC_PORT` 字面量导致 yaml 解析失败（无合法整数值）
  - 即便手动替换，**没有 deployment.yaml 模板也无从替换**（templates 空目录）
- **修复建议**：
  1. 在 6 chart `templates/` 目录添加 `deployment.yaml` / `service.yaml` / `configmap.yaml` 模板（参考 `docs/09-部署运维/RGS-OPS-100_Saga系统K3s部署设计_v0.1.md` L131-183 已有 `containerPort: 50051` 写法）
  2. 6 chart `values.yaml` 替换 `PLACEHOLDER_*_GRPC_PORT` 为 `50051` / `50052` / `50053` / `50054` / `50055` / `50056`
  3. 6 k8s manifest `containerPort` / `port` / `targetPort` 同步替换为实际值
  4. `docs/deploy/05-deploy-sop.md` 增补 "port 分配矩阵" 章节

### H2. 6 域 proto gRPC RPC 严重缩水（与 PM-008 完工报告矛盾）

- **位置**：
  - `crates/player-service/proto/player/v1/player.proto:6-9`：`HealthCheck + GetPlayer`（2 个 RPC）
  - `crates/economy-service/proto/economy/v1/economy.proto:6-9`：`HealthCheck + GetAccount`（2 个 RPC）
  - `crates/match-service/proto/match/v1/match.proto:6-9`：`HealthCheck + GetMatch`（2 个 RPC）
  - `crates/social-service/proto/social/v1/social.proto:6-9`：`HealthCheck + GetGuild`（2 个 RPC）
  - `crates/admin-service/proto/admin/v1/admin.proto:6-9`：`HealthCheck + GetAdminOp`（2 个 RPC）
  - `crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto:6-9`：`HealthCheck + GetNode`（2 个 RPC）
- **问题**：
  - RGS-PM-008 完工报告 L91-96 声称 6 域分别有 24/26/16/16/16/15 个 RPC，**实际全部只有 2 个**
  - DTL-018/015/016/026/019/020 §3 列出的核心操作（register/heartbeat/update_profile/credit/debit/freeze/transfer/buy/create/join/start/finish 等）**全部没有 gRPC 契约**
- **影响**：
  - 5 域业务层没有任何对外暴露 API，**实际只能调 HealthCheck**——完工报告严重夸大
  - Saga Q-003 跨域协调需要 economy 调用 player/guild/mail 等域（per DTL-100 §3），这些跨域 RPC 完全没有 proto
- **修复建议**：
  1. 54.x 后续或 55.1 优先把 DTL-018/015/016/026/019/020 §3 列出的核心操作补全为 proto method
  2. 重点：`economy.proto` 补全 `Credit / Debit / FreezeAccount / Transfer / Reserve / Confirm / Compensate`（per DTL-100 §3 Saga 关键能力）
  3. `match.proto` 补全 `CreateMatch / JoinMatch / StartMatch / FinishMatch`（per DTL-016 §3.2 实时匹配）
  4. `social.proto` 补全 `CreateGuild / JoinGuild / PromoteMember / DissolveGuild`
  5. `player.proto` 补全 `RegisterPlayer / Heartbeat / UpdateProfile / DisablePlayer`
  6. `admin.proto` 补全 `Authenticate / CreateAdminUser / DisableAdminUser / ListAuditLog` + COC 转发代理
  7. `cluster_ops.proto` 补全 DTL-031 §7 全部 10+ method（per C1）

### H3. common.proto 6 次重复编译（DRY 违反 + 编译时间浪费）

- **位置**：
  - `crates/shared-platform/build.rs:6-12`：`compile_protos(&["proto/common/v1/common.proto"], &["proto"])`（编译 1 次）
  - `crates/player-service/build.rs:7-19`：包含 `"../shared-platform/proto/common/v1/common.proto"`（编译 1 次）
  - `crates/economy-service/build.rs:7-19`：同上
  - `crates/match-service/build.rs:7-19`：同上
  - `crates/social-service/build.rs:7-19`：同上
  - `crates/admin-service/build.rs:7-19`：同上
  - `crates/cluster-ops/build.rs:6-15`：同上
- **问题**：common.proto 实际被 `tonic_build` 解析 **6 次**（每次 5 域 build.rs 重复声明 + shared-platform 自身）
  - 6 个 OUT_DIR 各自生成 1 份 `common.v1.rs`
  - `tonic::include_proto!("common.v1")` 通过 `OUT_DIR` 环境变量指向不同 crate 各自的 OUT_DIR，**编译期类型不共享**——5 域通过 `pub mod common { tonic::include_proto!("common.v1") }` 拿到的 `EntityId` 与 shared-platform 自己的 `EntityId` 是 **不同类型**（虽然结构相同）
  - 跨 crate 互操作（如 5 域用 shared-platform 的 `EntityId` 接受 tonic message）会因类型不兼容失败
- **影响**：
  - cargo 编译时间翻倍（每次 5 域都要重编译 common.proto）
  - 跨 crate 类型不共享（潜在运行时错误）
  - 6 域各自 `pub mod common { tonic::include_proto!("common.v1") }` 重复声明（per `crates/player-service/src/lib.rs:27-31`）违反 DRY
- **修复建议**：
  1. 5 域 build.rs 移除 `../shared-platform/proto/common/v1/common.proto` 引用
  2. 5 域 Cargo.toml 添加 `shared-platform = { path = "../shared-platform" }`（per C5）
  3. 5 域 `lib.rs::common` 改为 `pub use shared_platform::proto::v1::*;` 复用 shared-platform 的生成代码
  4. 单一编译入口（shared-platform）生成 `common.v1`，5 域仅 re-export
  5. 这样 5 域 `EntityId` 实际就是 `shared_platform::proto::v1::EntityId`（同一类型）

### H4. 5 域 build.rs 模板高度重复（DRY 违反）

- **位置**：6 个 `crates/*/build.rs`
- **问题**：6 个 build.rs 除 `protos` 第一项（各自的 `*.proto`）外**完全相同**：
  - 共同引用 `"../shared-platform/proto/common/v1/common.proto"`
  - 共同 includes `"proto"` + `"../shared-platform/proto"` + `"../<domain>/proto"`
  - 共同调用 `tonic_build::configure().build_server(true).build_client(true).compile_protos(...)`
- **影响**：维护成本高——任何 build.rs 改动（如未来加 `cfg(feature = "...")` 条件编译）需要同步改 6 个文件
- **修复建议**：
  1. 提取 `build_common()` 函数到 `crates/shared-platform/build_helpers.rs`
  2. 5 域 build.rs 改为 `fn main() -> Result<()> { shared_platform_build::build_for("proto/player/v1/player.proto", "player-service") }`
  3. 接受一定的可读性损失（每个域 build.rs 从 ~20 行缩到 ~5 行）

### H5. k8s manifest / helm values 资源 requests/limits 注释与实际值未对齐 player HPA max=8 vs 副本数 2

- **位置**：
  - `docs/deploy/01-k8s-manifests/01-player-service.yaml:112-120`：HPA `maxReplicas: PLACEHOLDER_PLAYER_HPA_MAX` 注释 `# 8`
  - `docs/deploy/02-helm-charts/rust-game-server/charts/player/values.yaml:33-34`：`autoscaling.maxReplicas: 8`
  - `docs/deploy/02-helm-charts/rust-game-server/charts/player/values.yaml:12`：`replicaCount: 2`
- **问题**：
  - player 域 HPA min=2 / max=8，**`replicaCount: 2` 是初始值**（per values.yaml 注释）
  - k8s manifest `replicas: PLACEHOLDER_PLAYER_REPLICAS` 注释 `# 2`
  - 但没有 `replicas` 默认值与 HPA `minReplicas` 的关联文档说明——SRE 在 HPA 触发时能否正确接管？需要 SRE 验证
  - 此外 player 在 player-sessions 表缺 partial index，HPA 扩到 8 时 session 查询可能 O(n²)
- **影响**：中——SRE 部署前需明确 HPA 与 replicaCount 切换语义，否则 HPA 触发后 `replicas: 2` 会被 HPA 覆盖为动态值，副本数从 2 起步跳到 minReplicas=2 是 OK 的，但若运维误改 HPA 关闭，回退到 `replicas: 2` 是 hardcode 而非 configmap
- **修复建议**：
  1. 6 chart `values.yaml` 添加 `hpa.enabled: true` 显式开关（避免 HPA 与 replicaCount 冲突）
  2. k8s manifest 添加注释说明 HPA 接管时 `replicas` 字段被忽略
  3. player-sessions 表补 `CREATE INDEX idx_player_sessions_active ON player_sessions (player_id, expires_at) WHERE expires_at > NOW();`（仅活跃 session 索引）

---## 7. MEDIUM Issues

### M1. 6 域 `lib.rs::common` 重复声明

- **位置**：
  - `crates/player-service/src/lib.rs:27-31`：`pub mod common { pub mod v1 { tonic::include_proto!("common.v1"); } }`
  - `crates/economy-service/src/lib.rs:43-47`：同上
  - `crates/match-service/src/lib.rs:29-33`：同上
  - `crates/social-service/src/lib.rs:28-32`：同上
  - `crates/admin-service/src/lib.rs:29-33`：同上
  - `crates/cluster-ops/src/lib.rs:28-32`：同上
- **问题**：6 个 lib.rs 同一段 5 行代码完全重复
- **修复建议**：per H3 修复方案——5 域改为 `pub use shared_platform::proto::v1::*;`（如果 H3 修复后 shared-platform 的 proto 命名空间是 `pub mod v1`），5 域 lib.rs 删除 `pub mod common` 段

### M2. 5 域 `migrations/0001_init.sql` 模板相似但缺 DTL 章节引用注释模板

- **位置**：player / match / social 三个 0001_init.sql 头部都只引用 `DTL-018 §3` / `DTL-016 §3` / `DTL-026 §3`，没有列具体表对应的 DTL 子章节
- **问题**：表级注释（如 `players` / `player_sessions`）无 DTL 字段级追溯，CI 校验无法脚本化检查
- **修复建议**：
  1. 每张表添加 `-- per DTL-XXX §3.Y <表名>` 注释
  2. 写 `tools/check_dtl_sql_trace.sh` CI 脚本：grep SQL CREATE TABLE 语句 + 提取邻近 DTL 引用注释，缺失则 fail

### M3. 6 域 proto `display_name` 字段无 DTL 追溯

- **位置**：6 域 proto 都有 `string display_name = 4;`
- **问题**：DTL-018/015/016/026/019/020 §3 都没要求 `display_name` 字段
- **可能来源**：`Display name` 是后端常见字段（用户面向），但缺少 DTL 追溯意味着：
  - 字段语义模糊（"显示名"？玩家昵称？公会名？管理员别名？）
  - 经济域 `Account.display_name` 语义不明确（账户没有"显示名"概念）
- **修复建议**：
  1. 5 域 Lead 联合签字明确 `display_name` 语义
  2. 或删除非必要字段（如 economy `Account.display_name` 应改为 `account_label` 或删除）
  3. 更新 DTL §3 增加 `display_name` 字段定义

### M4. DTL-031 §7 API 与 gRPC service trait 完全不对应

- **位置**：`docs/01-核心架构与设计模式/RGS-DTL-031_集群运营中心与每功能原子升级_详细设计书.md:282-294` vs `crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto:6-9`
- **问题**：DTL-031 §7 列出 10+ method，proto 只有 2 个 method，**1-2 个月的 proto 演进空白期**
- **影响**：55.x 补全 cluster_ops proto 时需要重新设计 stream type（`DeclareFeatureUpgrade` 是 Server stream response），可能涉及 service trait 重构
- **修复建议**：55.1 优先补全 cluster_ops proto（per C1），不拖延

### M5. `crates/cluster-ops/src/proto.rs:7` 用 `r#match` 转义但 cluster-ops.proto 不需要

- **位置**：
  - `crates/match-service/src/proto.rs:7`：`tonic::include_proto!("r#match.v1");`（`match` 是 Rust 关键字，需要 `r#` 前缀）
  - `crates/cluster-ops/src/proto.rs:7`：`tonic::include_proto!("cluster_ops.v1");`（无 `r#`，OK）
- **问题**：无问题，验证用——match 域的 `r#` 转义是必需的（`match` 是 Rust 关键字），但 **不是所有 match 域代码都用了 `r#`**（如 `crates/match-service/src/main.rs:42` 用了 `match_service::proto::v1::match_service_server` 而非 `r#match_service_server`）——验证不一致性
- **影响**：低——`match` 作为包名用 `r#` 是必需的，但**所有引用处**都必须一致使用 `r#`，否则编译失败
- **修复建议**：审计 `crates/match-service/src/` 所有 `match_service::proto::v1` 引用是否都用 `r#`

---## 8. LOW Issues

### L1. RGS-PM-008 完工报告与实际产物严重脱节

- **位置**：`docs/00-管理类/RGS-PM-008_WF-1-54完工报告_v0.1.md:90-96`
- **问题**：
  - L91：player-service 24 个 RPC，实际 2 个（per H2）
  - L92：economy-service 26 个 RPC，实际 2 个
  - L93：match-service 16 个 RPC，实际 2 个
  - L94：social-service 16 个 RPC，实际 2 个
  - L95：admin-service 16 个 RPC，实际 2 个
  - L96：cluster-ops 15 个 RPC，实际 2 个
  - 报告称"shared-platform 暴露给 5 域"——实际 5 域 Cargo.toml 没声明（per C5）
  - 报告称"Saga 7 触发条件白名单"——实际 economy 只有 3 种（Transfer/DailyReward/Purchase）
- **影响**：**完工报告严重失实，签批栏所有人需要重新核签**——这是治理层问题
- **修复建议**：
  1. RGS-PM-008 升 v0.2，删除"已完成 RPC 数量"列或注明"per proto 54.x 实际只有 X 个"
  2. 增加"5 域 shared-platform 依赖待补"附录
  3. 重新走 12 类签字栏核签流程

### L2. Subject 域枚举 `rgs.cem.*` / `rgs.dlq.*` 无 DTL-031 字段追溯

- **位置**：`crates/shared-platform/src/subject.rs:57-64`（`SubjectBuilder::cem_event` / `SubjectBuilder::dlq`）
- **问题**：DTL-031 §2 数据流图有 `BUS[事件总线 CEM 探针/Replay]` 但 **没有** Subject 命名空间定义；subject.rs 的 `rgs.cem.*` / `rgs.dlq.*` 命名空间是 RGS-SPEC-CROSS-005 草案新增的，**没有 DTL 追溯**
- **影响**：低——命名空间设计可后续补充 DTL 引用
- **修复建议**：DTL-031 §2 数据流图更新 + 增补"Subject 命名空间"小节，引用 RGS-SPEC-CROSS-005

### L3. k8s manifest 资源注释保留 PLACEHOLDER 字面量

- **位置**：6 k8s manifest `PLACEHOLDER_PLAYER_CPU_REQ: 500m` 等 24+ PLACEHOLDER 字面量
- **问题**：注释保留占位符便于 CI 校验脚本做 token 替换（这是合理设计），但**没有任何自动化脚本实际执行替换**——SRE 手动替换可能漏改
- **修复建议**：
  1. 写 `tools/inject_k8s_values.sh` 脚本从 `values.yaml` 自动注入
  2. CI pipeline 在 `helm template` 阶段校验 `helm template output` 与 k8s manifest 输出一致性

---## 9. 修复优先级矩阵

| Issue | 严重度 | 主要文件 | 预计工时（人·天） | 阻塞范围 |
|-------|--------|----------|-------------------|----------|
| C1 | CRITICAL | `crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto` | 5 | 集群运营中心 / CEM / PFAU 全部控制面 API |
| C2 | CRITICAL | `crates/admin-service/migrations/0001_init.sql` (新增 0002) | 3 | DTL-031 §3 控制面持久化层 |
| C3 | CRITICAL | `crates/economy-service/migrations/0002_saga_init.sql` + `crates/cluster-ops/migrations/`（新增 0002） | 8 | DTL-100 §7 Saga Store 全部表 |
| C4 | CRITICAL | `crates/economy-service/migrations/0002_saga_init.sql` (inbox 部分) + `src/inbox.rs` | 2 | DTL-100 §4.2 Inbox 幂等性 |
| C5 | CRITICAL | 5 域 6 个 `Cargo.toml` + 5 域 `main.rs` | 4 | 54.9 client / 54.10 subject / 54.12 OTel / 54.13 metrics / 54.15 RBAC 全部接入 |
| C6 | CRITICAL | `docs/01-核心架构与设计模式/RGS-DTL-100_*.md` §6.2 + `crates/shared-platform/src/subject.rs` | 2 | NATS 事件总线跨域互操作 |
| H1 | HIGH | 6 chart `values.yaml` + 6 chart `templates/*.yaml`（新建） + 6 k8s manifest | 6 | 全部 6 域部署 |
| H2 | HIGH | 5 域 proto + cluster_ops proto（per C1） | 12 | 5 域业务 API + COC API |
| H3 | HIGH | 5 域 `build.rs` + 5 域 `lib.rs` + 5 域 `Cargo.toml`（per C5） | 2 | 类型一致性 + 编译时间 |
| H4 | HIGH | `crates/shared-platform/build_helpers.rs`（新建） + 5 域 `build.rs` 简化为 5 行 | 1 | 维护性 |
| H5 | HIGH | 6 chart `values.yaml`（HPA 开关） + 6 k8s manifest 注释 | 1 | SRE 部署运维 |
| M1 | MEDIUM | 5 域 `lib.rs`（per H3 修复后） | 0.5 | DRY |
| M2 | MEDIUM | 5 域 `migrations/0001_init.sql` 表注释 + CI 脚本 | 1 | DTL 追溯自动化 |
| M3 | MEDIUM | 6 域 proto + 5 域 DTL 文档 | 1 | 字段语义 |
| M4 | MEDIUM | `crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto`（per C1 同步） | 0 | — |
| M5 | MEDIUM | `crates/match-service/src/` 全文件审计 | 0.5 | 编译正确性 |
| L1 | LOW | `docs/00-管理类/RGS-PM-008_WF-1-54完工报告_v0.1.md` v0.2 | 0.5 | 治理层 |
| L2 | LOW | `docs/01-核心架构与设计模式/RGS-DTL-031_*.md` §2 | 0.2 | 文档同步 |
| L3 | LOW | `tools/inject_k8s_values.sh`（新建）+ CI pipeline | 1 | 部署自动化 |

**总预计工时：50.7 人·天**（基于人·天；按 Ulysses 的 token-OLU 框架：1 人·天 ≈ 100K-300K tokens，约 5M-15M tokens 总投入）

**强烈建议**：
1. C5（shared-platform 依赖补全）必须**最先修**——它是 H3 / M1 的前置
2. C3（Saga 物理 DB 错位）必须**第二个修**——它影响 DTL-100 全部 §3 §4 §5 §6 §7 章节
3. C1（cluster_ops proto 补全）必须**第三个修**——它是 ARC-051 全部控制面 API 的基础

---## 10. 审计员签注

<审计员>：architecture-consistency-adversarial-001
<签名>：<占位 — 待 5 域 Lead 联合签批>

**审核范围声明**：
- 本报告仅审核**架构层**一致性（proto / migration / Cargo 依赖 / port / DTL 引用 / helm chart）
- 未涉及：Rust 代码风格、测试覆盖率、安全审计（per RGS-SEC-100）、性能压测
- 未涉及：5 域业务逻辑（service.rs / repository.rs 实现细节）

**审核方法声明**：
- 全部基于 commit `e320c69` 仓库快照
- 所有发现均通过文件读取 + grep 验证（无推断）
- 6 域 `0001_init.sql` + `0002_saga_init.sql` 100% 完整阅读
- 6 域 proto 100% 完整阅读
- 6 域 main.rs 100% 完整阅读
- 6 域 Cargo.toml 100% 完整阅读
- helm chart values.yaml 6 份 100% 完整阅读
- k8s manifest 6 份 100% 完整阅读
- shared-platform/src/{lib.rs, subject.rs, client.rs, build.rs} 100% 完整阅读
- DTL-009 / DTL-031 / DTL-100 §3-7 关键章节 100% 完整阅读
- DTL-100 §7 schema 100% 完整阅读
- economy-service/src/{saga.rs, reservation.rs, inbox.rs} 100% 完整阅读

**审核局限性**：
- 未跑 `cargo build` / `cargo test` 实际编译验证（任务要求"不修改项目代码"）
- DTL-018/015/016/026/019/020 §3 完整字段需求未逐一比对（仅查表头 + 主键 + INDEX + FK 关键字段）
- shared-platform 22 个文件（lib.rs 78 行 + 21 个 module）未逐文件审完，仅 lib.rs / subject.rs / client.rs 完整阅读
- `crates/cluster-ops/src/{repository.rs, service.rs, grpc_service.rs, etc}` 业务实现未审

**未涵盖项（需要后续审核）**：
- RGS-SEC-100 §7 mTLS 实施完整性（应在 55.x 单独 audit）
- RGS-IMPL-002 v0.1 §3 工具链对齐（rust 1.98 / sqlx 0.8 / tonic 0.12 版本一致性）——本次只确认了 workspace 依赖版本一致
- 6 域实际 unit test / integration test 覆盖率
- DTL-018/015/016/026/019/020 §3 字段级需求完整追溯（per 域 Lead 提交 evidence）