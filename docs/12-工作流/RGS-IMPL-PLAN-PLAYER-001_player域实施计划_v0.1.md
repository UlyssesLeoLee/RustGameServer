# RGS-IMPL-PLAN-PLAYER-001 player 域实施计划

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-PLAYER-001 |
| 版本 | 0.2 |
| 父文档 | RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-PLAN-001 v1.0 + RGS-IMPL-001 工程约定 |
| 源详细设计 | RGS-DTL-044 player 主表 v0.1(commit `90d193b` WF-1-55-65)+ RGS-SPEC-DTL-044 实现规格 v0.2(commit `90d193b`)+ RGS-DTL-001 v0.6 §7.2.1 ARC-013 死锁防止/背压八边界(commit `756bcd3`)+ RGS-DTL-002 v0.3 元数据 |
| 适用范围 | player 域 Atomic App 全生命周期实施(player-service crate + player_db 库 + gRPC 接口 + RBAC 集成) |
| 目标基线 | Rust 1.98 + Actix Web 4.14.1 + PostgreSQL 18.6 + K3s |
| 责任人 | player 域 Lead(Ulysses per DEC-008 一人公司 12 角色兼任) |
| 触发 | WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草 | (本 v0.2 = v0.1 + DDD Review 反馈 + §3 RACI 矩阵 + §A 已知缺口 3 段)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版:player 域实施计划;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);**不引入新设计**——汇编 DTL-044 + SPEC-DTL-044 v0.2 + 17 份 v0.2 SPEC 引用 |
| 0.2 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | v0.2 升版:DDD Review 反馈 + §3 RACI 矩阵(per RGS-LEAD-RACI-001 v1.1)+ §A 已知缺口 3 段(跨域协调 / 实时审计 / 1 人 12 角色 RACI 全覆盖);**不引入新设计**——本 v0.2 仅在 v0.1 头表 + 修订历史 + §3 + §A 加内容,正文本(域职责/实施阶段/验收)不动 | 头部 + 修订 + §3 + §A(新增) | 全部 |

---


---

## §3 RACI 矩阵 (NEW, v0.2 升版增量, per RGS-LEAD-RACI-001 v1.1 §3)

本域 player 的 6 治理角色 × 7 实施任务 RACI 映射（per RGS-ADR-0055 v0.1 §4）:

| 治理角色 | Phases | Tasks | Dependencies | Rollback | Migration | Monitor |
|---|---|---|---|---|---|---|
| Arch | R | R | R | R | R | R |
| BE Lead | R | R | R | R | R | R |
| SRE Lead | R | R | R | R | R | R |
| DBA | R | R | R | C | R | R |
| PM | R | R | C | C | C | C |
| PO | C | C | C | I | I | I |

> **RACI 字母单值**（per RGS-ADR-0055 v0.1 §4）: R = Responsible 执行 / A = Accountable 主责（每行 1 个）/ C = Consulted 咨询 / I = Informed 知情

> **DEC-008 一人公司 12 角色** 在此表中统一 R（单人全责），无 R/A/C/I 区分。本表仅作为正式 RACI 框架，真实跨域协调待 5 域 binary 启 + DDD Review 反馈闭环后补。

## §A 已知缺口 (NEW, per RGS-DOCS-HEALTH-2026-08-26 §0~§4 治理基线)

### §A.1 player 域 跨域协调依赖

本 player 域 IMPL-PLAN 涉及跨域 gRPC 调用（player → economy/match/social/admin + saga）需 5 域 binary 全部启 + 跨域联调通过才能完整验证。当前阻塞：
- PostgreSQL 18.6 未装（per Ulysses 16:58/16:59 硬约束，等装入）
- 5 域 binary 编译完成但启需 DATABASE_URL（per `RGS-GM-V0.3-DEPLOY-SOP-2026-08-26.md` v0.1）
- 跨域联调 IT 测试（per RGS-TEST-STRATEGY 4 阶段 phase 2，等 PG 装完）

### §A.2 实时审计跟踪

本 player 域 IMPL-PLAN 涉及 GM 操作 / RBAC 权限变更 / 跨域事件触发等操作需实时审计跟踪。依赖:
- ARC-018/021/042/051 4 治理角色（per RGS-ADR-0055 v0.1 §4）
- audit log 落库（per DTL-031 事件总线 + audit_log 表）
- 实时审计 dashboard（per rgs-web GM 后台 §3.5）

当前状态：审计跟踪设计在 DTL-031 §4，但 dashboard UI 未实装。

### §A.3 一人公司 12 角色 RACI 全覆盖

本 player 域 IMPL-PLAN v0.2 §3 RACI 矩阵仅含 6 治理角色（Arch/BE Lead/SRE Lead/DBA/PM/PO），缺:
- FE Lead（前后端边界，本域为后端无 FE）
- QA Lead（per RGS-TEST-STRATEGY 4 阶段）
- SEC（per RGS-REV-008 mTLS fail-closed）
- SRE（SRE Lead + SRE 角色区分，per DEC-008）
- 1 人 12 角色（per DEC-008 一人公司治理基线，5 域 Lead 全部 = Ulysses 兼任）

本缺口待 5 域 binary 启 + DDD Review 反馈 + RACI-001 v1.2 升版时统一补全。

## 1. 域职责

per RGS-DTL-044 v0.1 §1「player 主表」+ RGS-SPEC-DTL-044 v0.2 §1「实施范围」:

player 域是 RGS 5 域中**身份与账户管理**核心域,职责覆盖:

- **player 主表** CRUD(player_id UUID v7 主键 + display_name + email + phone + 注册渠道 + 状态机 7 状态)
- **跨域身份桥**——为 economy / match / social / admin 域提供 `player_id` 外键引用 + 选服路由
- **RBAC 身份层**——`player` 角色矩阵(per RGS-SPEC-CROSS-007 §3 5 域 RBAC 角色矩阵 v0.2)
- **player session** 管理——session_epoch 必填规则(per ARC-005 强制 + DTL-036 v1.4.2 §3 双层校验)
- **匿名访问边界**——`/sdk/v1/player/whoami` 等匿名可访问端点(per FR-PLR-001 既有)
- **player 域 DB 分区**——按 `player_id` 哈希 16 分区(per BAS-007 §3.3 分片新增/下线复用)

**域边界(per DTL-044 §1.2 不做"具体业务逻辑"边界)**:
- ❌ **不**持有玩家资产/道具/订单数据(归 economy 域)
- ❌ **不**持有匹配/对局数据(归 match 域)
- ❌ **不**持有好友/聊天/公会数据(归 social 域)
- ❌ **不**持有 GM/审计/合规删除(归 admin 域)
- ❌ **不**实现跨域 Saga 协调(归 saga 域,DTL-100/101/102)
- ✅ **仅**做身份与 player_id 路由,所有跨域字段通过 gRPC 客户端查询

**关键硬约束(per SPEC-DTL-044 v0.2 §3 + DTL-001 v0.6 §7.2.1 ARC-013 死锁防止)**:

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-PLR-001 | 匿名可访问端点(whoami / register / login) | 既有 |
| NFR-PLR-002 | player 主表读 p99 < 5ms | 实测 |
| NFR-PLR-003 | player 域 RBAC 必须经 shared-platform::rbac 中间件 | 硬约束 |
| NFR-PLR-004 | session_epoch 必填(ARC-005 强制) | 硬约束(代码评审 grep) |
| AC-PLR-001~005 | 5 项验收门槛 | 实测 |
| TBD-PLR-101~103 | 3 项 TBD | PH-3 实测填 |

---

## 2. 实施阶段(8 任务簇 × 4 L4 任务 = 32 L4)

| 任务簇 | 任务编号 | 任务名 | owner | 工期 | 依赖 |
|---|---|---|---|---|---|
| **API Spec** | 1.1 | player 域 gRPC Proto 定义(player_id / session / register / login / whoami / update_profile) | player 域 Lead | 0.5 人·天 | 父 BAS-001 v1.4 + SPEC-CROSS-002 v0.2 gRPC 风格指南 |
| API Spec | 1.2 | REST → gRPC 适配层(/sdk/v1/player/* HTTP 端点) | player 域 Lead | 0.5 人·天 | 1.1 + DTL-044 v0.1 §4 |
| API Spec | 1.3 | 错误码映射(per SPEC-CROSS-001 v0.2 错误码字典 + DTL-001 §3.4 ADR-0057 Tier-1/Tier-2) | player 域 Lead | 0.5 人·天 | 1.1 + 1.2 |
| API Spec | 1.4 | OpenAPI 3.1 + JSON Schema 生成 + 契约测试对齐 | player 域 Lead | 0.5 人·天 | 1.1-1.3 |
| **业务逻辑** | 2.1 | player 主表 Service 层(player_id UUID v7 生成 + CRUD) | player 域 Lead | 1 人·天 | 1.1-1.4 + shared-platform::id_gen |
| 业务逻辑 | 2.2 | session_epoch 双层校验(per ARC-005 强制,代码评审 grep) | player 域 Lead | 1 人·天 | 2.1 + DTL-036 v1.4.2 §3 |
| 业务逻辑 | 2.3 | 选服路由(per BAS-020 平台内购合规 + DTL-044 §3 跨域联动) | player 域 Lead | 1 人·天 | 2.1 + economy-service / match-service gRPC client |
| 业务逻辑 | 2.4 | RBAC 中间件集成(per SPEC-CROSS-007 v0.2 §3 5 域角色矩阵) | player 域 Lead | 1 人·天 | 2.1 + shared-platform::rbac |
| **DB migration** | 3.1 | player_db 库创建 + 5 独立 PG 18.6 数据库元数据(per WBS §2A.1) | player 域 Lead | 0.5 人·天 | DB Pool 治理基线 |
| DB migration | 3.2 | player 主表 DDL(player_id UUID v7 PK + 7 状态枚举 + email 唯一索引) | player 域 Lead | 0.5 人·天 | 3.1 + BAS-007 §3.2 命名规范 |
| DB migration | 3.3 | 按 player_id 哈希 16 分区(per BAS-007 §3.3 + DTL-001 §3.4) | player 域 Lead | 0.5 人·天 | 3.2 + sqlx 集成 |
| DB migration | 3.4 | migration 工具链(sqltest + golang-migrate 替代,per IMPL-001 §3) | player 域 Lead | 0.5 人·天 | 3.1-3.3 |
| **UT** | 4.1 | player 主表 Service UT(CRUD + 7 状态机转移 + 非法转移负例) | player 域 Lead | 1 人·天 | 2.1 + rgs-testkit |
| UT | 4.2 | session_epoch 双层校验 UT(per ARC-005 强制) | player 域 Lead | 0.5 人·天 | 2.2 + DTL-036 v1.4.2 §3 |
| UT | 4.3 | 选服路由 UT(per DTL-044 §3 跨域联动) | player 域 Lead | 0.5 人·天 | 2.3 + wiremock |
| UT | 4.4 | RBAC 中间件 UT(per SPEC-CROSS-007 §3 5 域角色矩阵) | player 域 Lead | 0.5 人·天 | 2.4 + rgs-testkit |
| **IT** | 5.1 | player_db 集成测试(sqltest + 5 独立 PG 池,per WBS §2A.1) | player 域 Lead | 0.5 人·天 | 3.1-3.4 + rgs-testkit |
| IT | 5.2 | player → economy gRPC 集成(选服路由 + cross-domain event) | player 域 Lead | 0.5 人·天 | 2.3 + economy-service test container |
| IT | 5.3 | player → match gRPC 集成(选服路由 + session_epoch 透传) | player 域 Lead | 0.5 人·天 | 2.3 + match-service test container |
| IT | 5.4 | player → social / admin gRPC 集成(RBAC 鉴权 + 跨域只读) | player 域 Lead | 0.5 人·天 | 2.4 + social-service / admin-service test container |
| **ST** | 6.1 | 端到端:注册→登录→whoami→选服(ST harness per RGS-TST-ST-00) | player 域 Lead | 1 人·天 | 5.1-5.4 + K3s namespace |
| ST | 6.2 | NFR-PLR-002 读 p99 < 5ms 实测(100 万 player 数据集) | player 域 Lead | 0.5 人·天 | 6.1 + prometheus |
| ST | 6.3 | AC-PLR-001~005 全部 5 项达标 | player 域 Lead | 0.5 人·天 | 6.1 + check-docs-consistency.sh |
| ST | 6.4 | 故障注入:DB 断连 / session_epoch 篡改 / RBAC 越权 | player 域 Lead | 0.5 人·天 | 6.1 + chaos-mesh |
| **Helm chart** | 7.1 | player-service Helm chart 骨架(per ARC-051 cluster-ops Feature) | cluster-ops 域 Lead | 0.5 人·天 | 2.1-2.4 + Helm 模板 |
| Helm chart | 7.2 | player_db Helm chart 依赖 + StatefulSet + PVC | cluster-ops 域 Lead | 0.5 人·天 | 3.1-3.4 + Helm 依赖 |
| Helm chart | 7.3 | 5 独立 PG 元数据 ConfigMap + K3s namespace 隔离 | cluster-ops 域 Lead | 0.5 人·天 | 7.1 + 7.2 |
| Helm chart | 7.4 | values.yaml 多环境(dev/staging/prod)+ secrets 治理 | cluster-ops 域 Lead | 0.5 人·天 | 7.1-7.3 + sealed-secrets |
| **observability** | 8.1 | OTel trace_id 传播(per SPEC-CROSS-006 v0.2 日志 trace_id 传播规范) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + shared-platform::tracing |
| observability | 8.2 | 10 项 rgs_player_* 指标(QPS / 延迟 / 错误率 / session 数 / DB 连接池) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + prometheus crate |
| observability | 8.3 | 5 项 rgs_player_* 日志字段(player_id / session_epoch / op / result / cost_ms) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + tracing-subscriber |
| observability | 8.4 | Grafana dashboard JSON(读 / 写 / 错误率 / DB 池) | SRE 域 Lead | 0.5 人·天 | 8.1-8.3 + Grafana provisioning |

**L4 合计**:32 个 L4 任务 / ~16 人·天(per RGS-TS-001 v0.6 §6.2 token-OLU 1 人·天 ≈ 100K-300K tokens,合计 ~3.2M-4.8M tokens)

---

## 3. 任务清单(32 L4 详细)

> per WBS v0.3 §6.2 拆分原则:每个 L4 任务 = 1 agent 最小可拆分,≤ 2 人·天 / ≤ 500K tokens

### 3.1 API Spec 簇(1.1-1.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 1.1 | player 域 gRPC Proto 定义 | `crates/rgs-player-service/proto/player.proto` | 80K | BAS-001 v1.4 |
| 1.2 | REST → gRPC 适配层 | `crates/rgs-player-service/src/api/rest_grpc_bridge.rs` | 80K | 1.1 + DTL-044 §4 |
| 1.3 | 错误码映射(SPEC-CROSS-001 v0.2) | `crates/rgs-player-service/src/error.rs` | 60K | 1.1 + 1.2 |
| 1.4 | OpenAPI 3.1 + 契约测试 | `crates/rgs-player-service/openapi/player.yaml` | 80K | 1.1-1.3 |

### 3.2 业务逻辑簇(2.1-2.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 2.1 | player 主表 Service | `crates/rgs-player-service/src/service/player.rs` | 200K | 1.1-1.4 + shared-platform::id_gen |
| 2.2 | session_epoch 双层校验 | `crates/rgs-player-service/src/service/session.rs` | 150K | 2.1 + DTL-036 v1.4.2 §3 |
| 2.3 | 选服路由 | `crates/rgs-player-service/src/service/router.rs` | 150K | 2.1 + economy/match gRPC client |
| 2.4 | RBAC 中间件 | `crates/rgs-player-service/src/middleware/rbac.rs` | 150K | 2.1 + shared-platform::rbac |

### 3.3 DB migration 簇(3.1-3.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 3.1 | player_db 库 + 5 独立 PG 元数据 | `crates/rgs-player-service/migrations/0001_player_db.sql` | 80K | DB Pool 治理基线 |
| 3.2 | player 主表 DDL | `crates/rgs-player-service/migrations/0002_player_main.sql` | 80K | 3.1 + BAS-007 §3.2 |
| 3.3 | 按 player_id 哈希 16 分区 | `crates/rgs-player-service/migrations/0003_partition.sql` | 100K | 3.2 + sqlx 集成 |
| 3.4 | migration 工具链 | `crates/rgs-player-service/migrations/sqltest.toml` | 60K | 3.1-3.3 |

### 3.4 UT 簇(4.1-4.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 4.1 | player Service UT | `crates/rgs-player-service/tests/ut_player_service.rs` | 200K | 2.1 + rgs-testkit |
| 4.2 | session_epoch UT | `crates/rgs-player-service/tests/ut_session.rs` | 80K | 2.2 + DTL-036 v1.4.2 §3 |
| 4.3 | 选服路由 UT | `crates/rgs-player-service/tests/ut_router.rs` | 80K | 2.3 + wiremock |
| 4.4 | RBAC UT | `crates/rgs-player-service/tests/ut_rbac.rs` | 100K | 2.4 + rgs-testkit |

### 3.5 IT 簇(5.1-5.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 5.1 | player_db 集成 | `crates/rgs-player-service/tests/it_player_db.rs` | 100K | 3.1-3.4 + rgs-testkit |
| 5.2 | player → economy 集成 | `crates/rgs-player-service/tests/it_economy.rs` | 100K | 2.3 + economy test container |
| 5.3 | player → match 集成 | `crates/rgs-player-service/tests/it_match.rs` | 100K | 2.3 + match test container |
| 5.4 | player → social/admin 集成 | `crates/rgs-player-service/tests/it_social_admin.rs` | 100K | 2.4 + social/admin test container |

### 3.6 ST 簇(6.1-6.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 6.1 | 端到端 harness | `tests/st/player_e2e.rs` | 200K | 5.1-5.4 + K3s namespace |
| 6.2 | NFR-PLR-002 实测 | `tests/st/player_perf.rs` | 100K | 6.1 + prometheus |
| 6.3 | AC-PLR-001~005 | `tests/st/player_ac.rs` | 80K | 6.1 + check-docs-consistency.sh |
| 6.4 | 故障注入 | `tests/st/player_chaos.rs` | 80K | 6.1 + chaos-mesh |

### 3.7 Helm chart 簇(7.1-7.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 7.1 | player-service Helm chart | `deploy/helm/rgs-player-service/Chart.yaml` | 80K | 2.1-2.4 + Helm 模板 |
| 7.2 | player_db StatefulSet | `deploy/helm/rgs-player-db/Chart.yaml` | 80K | 3.1-3.4 + Helm 依赖 |
| 7.3 | 5 独立 PG ConfigMap | `deploy/helm/rgs-shared-pg/configmap.yaml` | 60K | 7.1 + 7.2 |
| 7.4 | values.yaml 多环境 | `deploy/helm/rgs-player-service/values.yaml` | 60K | 7.1-7.3 + sealed-secrets |

### 3.8 observability 簇(8.1-8.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 8.1 | OTel trace_id 传播 | `crates/rgs-player-service/src/observability/trace.rs` | 80K | 2.1-2.4 + shared-platform::tracing |
| 8.2 | 10 项 rgs_player_* 指标 | `crates/rgs-player-service/src/observability/metrics.rs` | 80K | 2.1-2.4 + prometheus crate |
| 8.3 | 5 项 rgs_player_* 日志 | `crates/rgs-player-service/src/observability/log.rs` | 60K | 2.1-2.4 + tracing-subscriber |
| 8.4 | Grafana dashboard | `deploy/grafana/player-dashboard.json` | 60K | 8.1-8.3 + Grafana provisioning |

---

## 4. RACI 责任矩阵

> per RGS-ADR-0055 §4 + DEC-008 一人公司 12 角色兼任,本域 + 5 域 + foundation + cluster-ops + shared-platform = 9 角色 × 8 任务簇 = 72 单元

| 任务簇 \ 角色 | player 域 Lead | economy 域 Lead | match 域 Lead | social 域 Lead | admin 域 Lead | saga 域 Lead | foundation Lead | cluster-ops Lead | shared-platform Lead |
|---|---|---|---|---|---|---|---|---|---|
| API Spec | **R/A** | C | C | C | C | I | C | I | C(SPEC-CROSS-002) |
| 业务逻辑 | **R/A** | C(2.3 选服) | C(2.3 选服) | I | I | I | I | I | C(RBAC) |
| DB migration | **R/A** | I | I | I | I | I | I | C(7.2 StatefulSet) | C(命名规范) |
| UT | **R/A** | I | I | I | I | I | C(rgs-testkit) | I | C |
| IT | **R/A** | C(5.2) | C(5.3) | C(5.4) | C(5.4) | I | C(test container) | C(K3s) | C |
| ST | **R/A** | C | C | C | C | C | C | C(K3s) | C(OTel) |
| Helm chart | C(7.1/7.2 需求) | I | I | I | I | I | I | **R/A** | I |
| observability | C(8.1-8.4 需求) | I | I | I | I | I | I | C(dashboard 部署) | **R/A**(OTel/trace_id) |

**RACI 简码**:R = 执行(Responsible) / A = 最终批准(Accountable) / C = 咨询(Consulted) / I = 知会(Informed)

> 注:Helm chart 与 observability 任务簇的 A 角色分别为 cluster-ops Lead 与 shared-platform Lead,因这些是平台级能力,不由 player 域独占;但 player 域 Lead 作为 C 角色提供本域业务需求(per RGS-ADR-0051/0052 平台-域协作基线)。

---

## 5. Rollback 回滚路径

> 应用 / DB migration / plugin / 配置 / Collector / dashboard 独立回滚

### 5.1 应用回滚

- player-service **不是**必选路径——若 player 域上线后出现回归:
  1. 通过 cluster-ops 域 `k8s rollout undo deployment/rgs-player-service -n player`
  2. 触发 5 域 PFAU 编排(per ARC-051)切换回上一个 PFAU Feature 版本
  3. 监控:AC-PLR-001~005 门禁自动告警

### 5.2 DB migration 回滚

- **每个 migration 都是 idempotent + reversible**(per BAS-007 §3.4 命名约定):
  - `migrations/0001_player_db.sql` → reverse = `DROP DATABASE player_db`(per FR-DB-008)
  - `migrations/0002_player_main.sql` → reverse = `DROP TABLE player_main`
  - `migrations/0003_partition.sql` → reverse = `DROP PARTITION` × 16
- **migration 版本号管理**:sqltest 单向推进,reverse 通过新建 reverse migration 文件 `9999_rollback_*.sql`(per IMPL-001 §3.4)

### 5.3 plugin 回滚

- player-service **无 plugin 加载点**(per BAS-005 插件热插拔边界,player 域不开放 plugin 接口)
- 若未来扩展,plugin 通过 shared-platform::plugin_loader 加载,回滚经 cluster-ops Feature

### 5.4 配置回滚

- `values.yaml` 多环境(dev/staging/prod)Helm 参数回滚经 `helm rollback rgs-player-service <revision>`(per RGS-WT-001 §5)
- secrets 治理经 sealed-secrets 双向加解密(per IMPL-002 §6)

### 5.5 Collector 回滚

- OTel Collector 配置由 cluster-ops 域统一管理,player 域仅在 ConfigMap 注入 trace_id 传播规则
- 回滚经 cluster-ops 域 `kubectl apply -f otel-collector-prev.yaml -n observability`

### 5.6 dashboard 回滚

- Grafana dashboard JSON 通过 Grafana provisioning 双向同步
- 回滚经 `kubectl apply -f grafana-player-dashboard-prev.json -n observability`

---

## 6. 验收项

> CI 4 workflow + check-docs-consistency.sh + 17 份 v0.2 SPEC 引用一致

### 6.1 CI 4 workflow(per RGS-IMPL-006)

- [ ] `cargo fmt --check`(per IMPL-005 §2)
- [ ] `cargo clippy --all-targets -- -D warnings`(per IMPL-005 §3)
- [ ] `cargo test --all`(per IMPL-005 §4)
- [ ] `cargo deny check`(per IMPL-005 §5)

### 6.2 文档一致性

- [ ] `check-docs-consistency.sh` 通过(per RGS-WF-001 §5)
- [ ] 17 份 v0.2 SPEC 引用一致(SPEC-DTL-044 + SPEC-CROSS-001~007 + SPEC-DTL-001/002 + SPEC-DTL-100~102)
- [ ] RGS-DTL-044 v0.1 引用一致
- [ ] RGS-BAS-001 v1.4 / RGS-BAS-007 / RGS-BAS-020 引用一致

### 6.3 域硬约束

- [ ] NFR-PLR-002(读 p99 < 5ms)实测通过
- [ ] NFR-PLR-003(RBAC 经 shared-platform::rbac 中间件)代码评审 grep 通过
- [ ] NFR-PLR-004(session_epoch 必填,ARC-005 强制)代码评审 grep 通过
- [ ] FR-PLR-001(匿名可访问端点 whoami/register/login)代码评审 grep 通过

### 6.4 验收门槛

- [ ] AC-PLR-001(注册流程端到端)
- [ ] AC-PLR-002(登录流程端到端)
- [ ] AC-PLR-003(whoami 匿名访问)
- [ ] AC-PLR-004(选服路由正确)
- [ ] AC-PLR-005(session_epoch 双层校验拒绝篡改)

---

## 7. Definition of Done

per RGS-SPEC-DTL-044 v0.2 §7 + RGS-SPEC-000 v0.3 §4:

- [ ] 32 个 L4 任务全部完成 + commit 落地
- [ ] 6 份 CR(per RGS-IMPL-004)全部通过
- [ ] CI 4 workflow 全过
- [ ] ST 6.1 端到端测试通过
- [ ] ST 6.2 NFR-PLR-002 实测达标
- [ ] ST 6.3 AC-PLR-001~005 全部 5 项达标
- [ ] ST 6.4 故障注入恢复路径验证
- [ ] Helm chart 7.1-7.4 在 K3s 集群通过
- [ ] observability 8.1-8.4 在 staging 集群采集到数据
- [ ] check-docs-consistency.sh 通过
- [ ] 17 份 v0.2 SPEC 引用一致
- [ ] RACI 责任矩阵 72 单元全部登记
- [ ] Rollback 6 路径(application / DB migration / plugin / config / Collector / dashboard)实测演练通过
- [ ] 当前无实现文件时保持"待实现/待评审"状态(per RGS-SPEC-000 §5 第 7 条)

---

## 8. Gate 证据与实测参数

> per RGS-SPEC-DTL-044 v0.2 §8 + RGS-IMPL-006 §3

### 8.1 CI 证据(per workflow)

- **CI-FMT**: `cargo fmt --check` → exit 0
- **CI-LINT**: `cargo clippy --all-targets -- -D warnings` → exit 0
- **CI-TEST**: `cargo test --all` → 全部 UT 通过
- **CI-DENY**: `cargo deny check` → 无 license / advisory 失败

### 8.2 ST 证据

- **ST-6.1 E2E**:K3s namespace `player-st` 部署成功 + 5 域集成测试通过
- **ST-6.2 NFR**:100 万 player 数据集 + 1000 QPS 持续 10 分钟 → p99 < 5ms
- **ST-6.3 AC**:AC-PLR-001~005 全部 5 项达标(自动化测试报告)
- **ST-6.4 Chaos**:5 类故障注入(断连 / 篡改 / 越权)全部恢复路径验证

### 8.3 Helm 证据

- **7.1-7.4**:`helm install rgs-player-service ./deploy/helm/rgs-player-service --namespace player-prod` 成功
- **多环境**:`helm template` 在 dev / staging / prod 三环境输出合法 YAML

### 8.4 observability 证据

- **8.1 OTel**:trace_id 从 HTTP header → gRPC metadata → DB query 完整链路
- **8.2 指标**:Prometheus 抓取 `rgs_player_*` 10 项指标,值在合理范围
- **8.3 日志**:Loki 抓取 `rgs_player_*` 5 项字段,JSON 格式 + trace_id 关联
- **8.4 dashboard**:Grafana `player-dashboard.json` 4 panel(读 / 写 / 错误率 / DB 池)显示数据

### 8.5 Rollback 证据

- 应用回滚:`kubectl rollout undo` 在 staging 验证 < 30s
- DB 回滚:9999_rollback_*.sql 在 staging DB 验证成功
- Collector 回滚:OTel Collector 上一个版本在 staging 恢复
- dashboard 回滚:Grafana provisioning 上一版本恢复

---

## 9. 关联文档

### 9.1 上行

- [RGS-DTL-044 player 主表 v0.1](../02-运维安全与网络/RGS-DTL-044_player主表_v0.1.md)
- [RGS-SPEC-DTL-044 实现规格 v0.2](../13-实现规格/RGS-SPEC-DTL-044_实现规格书.md)
- [RGS-DTL-001 v0.6 §7.2.1 ARC-013](../02-运维安全与网络/RGS-DTL-001_详细设计书.md)
- [RGS-SPEC-CROSS-001 错误码字典 v0.2](../13-实现规格/RGS-SPEC-CROSS-001_错误码字典_v0.1.md)
- [RGS-SPEC-CROSS-002 gRPC Proto 风格指南 v0.2](../13-实现规格/RGS-SPEC-CROSS-002_gRPC_Proto风格指南_v0.1.md)
- [RGS-SPEC-CROSS-006 日志 trace_id 传播规范 v0.2](../13-实现规格/RGS-SPEC-CROSS-006_日志trace_id传播规范_v0.1.md)
- [RGS-SPEC-CROSS-007 5 域 RBAC 角色矩阵 v0.2](../13-实现规格/RGS-SPEC-CROSS-007_5域RBAC角色矩阵_v0.1.md)

### 9.2 下行

- [RGS-IMPL-001 实施约定与工程边界](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
- [RGS-IMPL-002 PG 编码规范 v0.1](../13-实现规格/RGS-IMPL-002_PG_编码规范_v0.1.md)
- [RGS-IMPL-004 CR 代码审查规范 v0.1](../13-实现规格/RGS-IMPL-004_CR_代码审查规范_v0.1.md)
- [RGS-IMPL-005 BUILD 构建规范 v0.1](../13-实现规格/RGS-IMPL-005_BUILD_构建规范_v0.1.md)
- [RGS-IMPL-006 CI 持续集成规范 v0.1](../13-实现规格/RGS-IMPL-006_CI_持续集成规范_v0.1.md)

### 9.3 同级(5 域 IMPL-PLAN 联动)

- [RGS-IMPL-PLAN-ECONOMY-001 economy 域实施计划](RGS-IMPL-PLAN-ECONOMY-001_economy域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-MATCH-001 match 域实施计划](RGS-IMPL-PLAN-MATCH-001_match域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-SOCIAL-001 social 域实施计划](RGS-IMPL-PLAN-SOCIAL-001_social域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ADMIN-001 admin 域实施计划](RGS-IMPL-PLAN-ADMIN-001_admin域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-SAGA-001 saga 域实施计划](RGS-IMPL-PLAN-SAGA-001_saga域实施计划_v0.1.md)

### 9.4 模板参考

- [RGS-IMPL-PLAN-CDN-001 断点续传实施计划 v0.1](RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md)
- [RGS-IMPL-PLAN-LCM-001 服务器全生命周期实施计划 v0.1](RGS-IMPL-PLAN-LCM-001_服务器全生命周期实施计划_v0.1.md)

---

## A. v0.1 对齐说明

### A.1 触发

WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草(player 域 1 份)

### A.2 范围

- player 域各 1 份 IMPL-PLAN v0.1,32 L4 任务占位齐全
- 6 域合计 32 L4 × 6 = 192 L4 任务占位
- 实施范围:**仅汇编**各域既有 DTL + 17 份 v0.2 SPEC,**不引入新设计**

### A.3 已知缺口

- 各域实际 L4 任务待 DDD Review 阶段补完(per RGS-SPEC-DTL-044 v0.2 §A.3 "无新缺口继承" + DTL-044 §2.1 与 DTL-001 §2.1 ISS-127 悬置状态关联)
- 5 域 Lead 签字(本 v0.1 占位,等 Ulysses DDD Review 后补签)
- OLU token 估算(WF-1-55.55 task 5 联动 + RGS-TS-001 v0.6 §6.2 token-OLU 框架)
- TBD-PLR-101~103(3 项 TBD)待 PH-3 实测填入

### A.4 引用链

- 17 份 v0.2 SPEC(commit `756bcd3` ~ `97ef67c`,per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.1)
- RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-IMPL-001
- DEC-008 一人公司 12 角色治理基线
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = "架构师(Mavis 接手 agent per DEC-008)",**不**再受"审批者 = —"硬约束
