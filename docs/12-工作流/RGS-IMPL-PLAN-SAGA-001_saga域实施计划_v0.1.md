# RGS-IMPL-PLAN-SAGA-001 saga 域实施计划

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-SAGA-001 |
| 版本 | 0.2 |
| 父文档 | RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-PLAN-001 v1.0 + RGS-IMPL-001 工程约定 |
| 源详细设计 | RGS-DTL-100 Saga 协调器 v0.2(commit `a3f0123` WF-1-55-66)+ RGS-DTL-101 Saga 事务日志 v0.1(commit `574764a` WF-1-55-67)+ RGS-DTL-102 Saga 步骤注册 v0.1(commit `97ef67c` WF-1-55-68)+ RGS-SPEC-DTL-100/101/102 实现规格 v0.2 + RGS-REQ-100 Saga 事务系统 v0.1 + RGS-DEC-Q003 跨 DB Saga + RGS-IMPL-100 Saga 实施规范 v0.1 |
| 适用范围 | saga 域 Atomic App 全生命周期实施(saga-orchestrator crate + saga_db 库 + 3 张表 + Saga 协调器 + 事务日志 + 步骤注册) |
| 目标基线 | Rust 1.98 + Actix Web 4.14.1 + PostgreSQL 18.6 + K3s |
| 责任人 | saga 域 Lead(Ulysses per DEC-008 一人公司 12 角色兼任,**独立**位 per 2026-08-21 一人公司架构师兼任拒绝证据) |
| 触发 | WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草 | (本 v0.2 = v0.1 + DDD Review 反馈 + §3 RACI 矩阵 + §A 已知缺口 3 段)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 初版:saga 域实施计划;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);**不引入新设计**——汇编 DTL-100/101/102 + SPEC-DTL-100/101/102 v0.2 + DTL-015/016/037 联动 + 17 份 v0.2 SPEC 引用 |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | v0.2 升版:DDD Review 反馈 + §3 RACI 矩阵(per RGS-LEAD-RACI-001 v1.1)+ §A 已知缺口 3 段(跨域协调 / 实时审计 / 1 人 12 角色 RACI 全覆盖);**不引入新设计**——本 v0.2 仅在 v0.1 头表 + 修订历史 + §3 + §A 加内容,正文本(域职责/实施阶段/验收)不动 | 头部 + 修订 + §3 + §A(新增) | 全部 |

---


---

## §3 RACI 矩阵 (NEW, v0.2 升版增量, per RGS-LEAD-RACI-001 v1.1 §3)

本域 saga 的 6 治理角色 × 7 实施任务 RACI 映射（per RGS-ADR-0055 v0.1 §4）:

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

### §A.1 saga 域 跨域协调依赖

本 saga 域 IMPL-PLAN 涉及跨域 gRPC 调用（player → economy/match/social/admin + saga）需 5 域 binary 全部启 + 跨域联调通过才能完整验证。当前阻塞：
- PostgreSQL 18.6 未装（per Ulysses 16:58/16:59 硬约束，等装入）
- 5 域 binary 编译完成但启需 DATABASE_URL（per `RGS-GM-V0.3-DEPLOY-SOP-2026-08-26.md` v0.1）
- 跨域联调 IT 测试（per RGS-TEST-STRATEGY 4 阶段 phase 2，等 PG 装完）

### §A.2 实时审计跟踪

本 saga 域 IMPL-PLAN 涉及 GM 操作 / RBAC 权限变更 / 跨域事件触发等操作需实时审计跟踪。依赖:
- ARC-018/021/042/051 4 治理角色（per RGS-ADR-0055 v0.1 §4）
- audit log 落库（per DTL-031 事件总线 + audit_log 表）
- 实时审计 dashboard（per rgs-web GM 后台 §3.5）

当前状态：审计跟踪设计在 DTL-031 §4，但 dashboard UI 未实装。

### §A.3 一人公司 12 角色 RACI 全覆盖

本 saga 域 IMPL-PLAN v0.2 §3 RACI 矩阵仅含 6 治理角色（Arch/BE Lead/SRE Lead/DBA/PM/PO），缺:
- FE Lead（前后端边界，本域为后端无 FE）
- QA Lead（per RGS-TEST-STRATEGY 4 阶段）
- SEC（per RGS-REV-008 mTLS fail-closed）
- SRE（SRE Lead + SRE 角色区分，per DEC-008）
- 1 人 12 角色（per DEC-008 一人公司治理基线，5 域 Lead 全部 = Ulysses 兼任）

本缺口待 5 域 binary 启 + DDD Review 反馈 + RACI-001 v1.2 升版时统一补全。

## 1. 域职责

per RGS-DTL-100 v0.2 §1「Saga 协调器」+ RGS-DTL-101 v0.1 §1「Saga 事务日志」+ RGS-DTL-102 v0.1 §1「Saga 步骤注册」+ RGS-SPEC-DTL-100/101/102 v0.2 §1:

saga 域是 RGS 5 域之上的**跨域 Saga 协调**核心域,职责覆盖:

- **3 张 saga 表** CRUD(per RGS-DTL-100 v0.2 §3):saga_instance / saga_step_log / saga_compensation_log
- **Saga 协调器**——`SagaOrchestrator` 接收跨域事务请求,按 6 场景步骤编号 1.0~6.0 调度(per RGS-DTL-015 v0.2 §3.4 + RGS-REV-005 附件 B)
- **Saga 事务日志**——所有 Saga 执行步骤写入 `saga_step_log`,失败补偿写入 `saga_compensation_log`(per DTL-101 v0.1)
- **Saga 步骤注册**——各域通过 `SagaStepRegistry` 注册自己的步骤实现(per DTL-102 v0.1,只读依赖)
- **Q-003 跨 DB Saga 决策**——per RGS-DEC-Q003,跨 DB Saga 经 Saga 域统一协调,不允许业务域直接跨 DB 事务
- **Saga reference 实现接收**——economy 域 `saga_orchestrator` reference 实现被 saga 域 reference,不允许反向覆盖(per DEC-008)
- **6 场景演练**——per RGS-REV-005 附件 B:1.0 单一事务 / 2.0 顺序 Saga / 3.0 并行 Saga / 4.0 嵌套 Saga / 5.0 跨域补偿 / 6.0 失败重试

**域边界(per DTL-100 v0.2 §1.2 + DTL-101 v0.1 + DTL-102 v0.1)**:
- ❌ **不**持有具体业务逻辑(归各业务域)
- ❌ **不**实现 Saga reference 实现本身(归 economy 域 DTL-015 v0.2 §3.4,saga 域仅 reference)
- ❌ **不**直接连业务 service DB(经 gRPC client + 业务域步骤注册)
- ❌ **不**实现 RBAC(归 shared-platform::rbac)
- ✅ **仅**做 Saga 协调 + 事务日志 + 步骤注册 + 跨域补偿

**关键硬约束(per SPEC-DTL-100/101/102 v0.2 §3 + DTL-100 v0.2 + DTL-101 v0.1 + DTL-102 v0.1 + RGS-IMPL-100)**:

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-SAG-001 | Saga 步骤编号 1.0~6.0 与 RGS-REV-005 附件 B 6 场景一致 | 既有 |
| NFR-SAG-002 | Saga 事务日志完整性(per DTL-101 v0.1 §3) | 硬约束 |
| NFR-SAG-003 | Saga 协调器单调解者原则(per RGS-ADR-0015 Saga 适用边界) | 硬约束 |
| NFR-SAG-004 | 跨 DB Saga 经 DEC-Q003 治理(per RGS-DEC-Q003) | 硬约束 |
| NFR-SAG-005 | 失败补偿可重入(idempotency per request_id) | 硬约束 |
| NFR-SAG-006 | Saga 步骤注册不允许反向覆盖(per DEC-008 域独立基线) | 硬约束 |
| NFR-SAG-007 | 经 shared-platform::outbox 落地 Saga 事件(per DTL-015 v0.2 + DTL-016 v0.3) | 硬约束 |
| AC-SAG-001~012 | 12 项验收门槛 | 实测 |
| TBD-SAG-101~107 | 7 项 TBD | PH-3 实测填 |
| Saga 步骤 1.0~6.0 | per DTL-015 v0.2 §3.4 + RGS-REV-005 附件 B | 引用基线 |

> **重要**:per 2026-08-21 一人公司 5 域 Lead 兼任方案决议,**saga 域 Lead 不得由其他域 Lead 兼任**(per RGS-ADR-0055 §4 + DEC-008);即使一人公司 12 角色兼任,saga 域 Lead 责任矩阵 + RACI 简表中 A 角色**必须** Ulysses 显式签字。本 v0.1 阶段"责任人"字段标注"独立位"。

---

## 2. 实施阶段(8 任务簇 × 4 L4 任务 = 32 L4)

| 任务簇 | 任务编号 | 任务名 | owner | 工期 | 依赖 |
|---|---|---|---|---|---|
| **API Spec** | 1.1 | saga 域 gRPC Proto(SagaOrchestrator / SagaStepLog / SagaCompensationLog) | saga 域 Lead | 0.5 人·天 | BAS-001 v1.4 + SPEC-CROSS-002 v0.2 |
| API Spec | 1.2 | Saga 步骤编号 1.0~6.0 Proto 字段(per DTL-015 v0.2 §3.4 + RGS-REV-005 附件 B) | saga 域 Lead | 0.5 人·天 | 1.1 + DTL-015 v0.2 + RGS-REV-005 |
| API Spec | 1.3 | 错误码映射(per SPEC-CROSS-001 v0.2 + DTL-001 §3.4 ADR-0057) | saga 域 Lead | 0.5 人·天 | 1.1 + 1.2 |
| API Spec | 1.4 | OpenAPI 3.1 + 跨域事件 schema(per SPEC-CROSS-003 v0.2) | saga 域 Lead | 0.5 人·天 | 1.1-1.3 |
| **业务逻辑** | 2.1 | SagaOrchestrator 核心调度器(6 场景,per FR-SAG-001) | saga 域 Lead | 1.5 人·天 | 1.1-1.4 + DTL-100 v0.2 §3 + RGS-IMPL-100 |
| 业务逻辑 | 2.2 | Saga 事务日志(per NFR-SAG-002 + DTL-101 v0.1) | saga 域 Lead | 1 人·天 | 2.1 + DTL-101 v0.1 §3 |
| 业务逻辑 | 2.3 | Saga 步骤注册中心(per DTL-102 v0.1 + NFR-SAG-006 域独立) | saga 域 Lead | 1 人·天 | 2.1 + 5 域 gRPC client + NFR-SAG-006 |
| 业务逻辑 | 2.4 | 跨域补偿 + Q-003 跨 DB Saga(per NFR-SAG-004 + RGS-DEC-Q003) | saga 域 Lead | 1.5 人·天 | 2.1 + 2.2 + 2.3 + DEC-Q003 |
| **DB migration** | 3.1 | saga_db 库 + 5 独立 PG 18.6 元数据(per WBS §2A.1) | saga 域 Lead | 0.5 人·天 | DB Pool 治理基线 |
| DB migration | 3.2 | 3 张 saga 表 DDL(saga_instance / saga_step_log / saga_compensation_log) | saga 域 Lead | 0.5 人·天 | 3.1 + BAS-007 §3.2 + DTL-100 v0.2 §3 |
| DB migration | 3.3 | saga_step_log 按 created_at 月分区(per DTL-016 v0.3) | saga 域 Lead | 0.5 人·天 | 3.2 + sqlx |
| DB migration | 3.4 | migration 工具链 + Outbox 表(per NFR-SAG-007) | saga 域 Lead | 0.5 人·天 | 3.1-3.3 + IMPL-001 §3 + shared-platform::outbox |
| **UT** | 4.1 | SagaOrchestrator UT(6 场景全覆盖,per FR-SAG-001) | saga 域 Lead | 1.5 人·天 | 2.1 + rgs-testkit + RGS-REV-005 附件 B |
| UT | 4.2 | Saga 事务日志 UT(per NFR-SAG-002 + DTL-101 v0.1) | saga 域 Lead | 0.5 人·天 | 2.2 + rgs-testkit |
| UT | 4.3 | Saga 步骤注册中心 UT(per NFR-SAG-006 域独立) | saga 域 Lead | 0.5 人·天 | 2.3 + rgs-testkit |
| UT | 4.4 | 跨域补偿 + 重入 idempotency UT(per NFR-SAG-004 + NFR-SAG-005) | saga 域 Lead | 1 人·天 | 2.4 + rgs-testkit |
| **IT** | 5.1 | saga_db 集成测试(sqltest + 5 独立 PG 池) | saga 域 Lead | 0.5 人·天 | 3.1-3.4 + rgs-testkit |
| IT | 5.2 | saga → 5 域步骤注册集成(per NFR-SAG-006) | saga 域 Lead + 5 域 Lead | 1 人·天 | 2.3 + 5 域 test container |
| IT | 5.3 | Q-003 跨 DB Saga 集成(player_db + economy_db,per RGS-DEC-Q003) | saga 域 Lead | 1 人·天 | 2.4 + DEC-Q003 + economy test container |
| IT | 5.4 | saga → admin 集成(GM 补偿走 Saga,per DTL-040 v0.1) | saga 域 Lead | 0.5 人·天 | 2.4 + admin test container |
| **ST** | 6.1 | 端到端:6 场景演练(ST harness,per RGS-REV-005 附件 B) | saga 域 Lead | 1.5 人·天 | 5.1-5.4 + K3s namespace |
| ST | 6.2 | NFR-SAG-002/005 事务日志完整性 + 重入 idempotency 实测 | saga 域 Lead | 0.5 人·天 | 6.1 + prometheus |
| ST | 6.3 | AC-SAG-001~012 全部 12 项达标 | saga 域 Lead | 0.5 人·天 | 6.1 + check-docs-consistency.sh |
| ST | 6.4 | 故障注入:跨 DB Saga 失败补偿 / Outbox 重复 / 重入幂等冲突 | saga 域 Lead | 0.5 人·天 | 6.1 + chaos-mesh |
| **Helm chart** | 7.1 | saga-orchestrator Helm chart 骨架(per ARC-051) | cluster-ops 域 Lead | 0.5 人·天 | 2.1-2.4 + Helm |
| Helm chart | 7.2 | saga_db Helm chart 依赖 + StatefulSet + PVC | cluster-ops 域 Lead | 0.5 人·天 | 3.1-3.4 + Helm 依赖 |
| Helm chart | 7.3 | 5 独立 PG ConfigMap + K3s namespace 隔离 | cluster-ops 域 Lead | 0.5 人·天 | 7.1 + 7.2 |
| Helm chart | 7.4 | values.yaml 多环境 + sealed-secrets 治理 | cluster-ops 域 Lead | 0.5 人·天 | 7.1-7.3 + sealed-secrets |
| **observability** | 8.1 | OTel trace_id 传播(per SPEC-CROSS-006 v0.2) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + shared-platform::tracing |
| observability | 8.2 | 15 项 rgs_saga_* 指标(场景 / 步骤 / 补偿 / Outbox lag) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + prometheus |
| observability | 8.3 | 8 项 rgs_saga_* 日志(saga_id / step / 跨域引用 / request_id) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + tracing-subscriber |
| observability | 8.4 | Grafana dashboard JSON(场景 / 步骤 / 补偿 / Outbox) | SRE 域 Lead | 0.5 人·天 | 8.1-8.3 + Grafana |

**L4 合计**:32 个 L4 任务 / ~19 人·天(per RGS-TS-001 v0.6 §6.2 token-OLU,合计 ~3.8M-5.7M tokens)

---

## 3. 任务清单(32 L4 详细)

### 3.1 API Spec 簇(1.1-1.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 1.1 | saga 域 gRPC Proto(3 模块) | `crates/rgs-saga-orchestrator/proto/saga.proto` | 100K | BAS-001 v1.4 |
| 1.2 | Saga 步骤编号 1.0~6.0 Proto 字段 | `crates/rgs-saga-orchestrator/proto/saga_step.proto` | 100K | 1.1 + DTL-015 v0.2 §3.4 + RGS-REV-005 附件 B |
| 1.3 | 错误码映射 | `crates/rgs-saga-orchestrator/src/error.rs` | 60K | 1.1 + 1.2 + SPEC-CROSS-001 |
| 1.4 | OpenAPI 3.1 + 跨域事件 schema | `crates/rgs-saga-orchestrator/openapi/saga.yaml` | 80K | 1.1-1.3 + SPEC-CROSS-003 |

### 3.2 业务逻辑簇(2.1-2.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 2.1 | SagaOrchestrator 核心调度器(6 场景) | `crates/rgs-saga-orchestrator/src/orchestrator/{engine,scenarios}.rs` | 500K | 1.1-1.4 + DTL-100 v0.2 §3 + RGS-IMPL-100 + RGS-REV-005 附件 B |
| 2.2 | Saga 事务日志 | `crates/rgs-saga-orchestrator/src/log/{step_log,compensation_log}.rs` | 250K | 2.1 + DTL-101 v0.1 §3 |
| 2.3 | Saga 步骤注册中心 | `crates/rgs-saga-orchestrator/src/registry/step_registry.rs` | 250K | 2.1 + 5 域 gRPC client + NFR-SAG-006 |
| 2.4 | 跨域补偿 + Q-003 跨 DB Saga | `crates/rgs-saga-orchestrator/src/orchestrator/{compensation,cross_db}.rs` | 400K | 2.1 + 2.2 + 2.3 + RGS-DEC-Q003 |

### 3.3 DB migration 簇(3.1-3.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 3.1 | saga_db 库 + 5 独立 PG | `crates/rgs-saga-orchestrator/migrations/0001_saga_db.sql` | 80K | DB Pool 治理基线 |
| 3.2 | 3 张 saga 表 DDL | `crates/rgs-saga-orchestrator/migrations/0002_saga_tables.sql` | 100K | 3.1 + BAS-007 §3.2 + DTL-100 v0.2 §3 |
| 3.3 | saga_step_log 月分区 | `crates/rgs-saga-orchestrator/migrations/0003_partition.sql` | 100K | 3.2 + sqlx + DTL-016 v0.3 |
| 3.4 | migration + Outbox 表 | `crates/rgs-saga-orchestrator/migrations/0004_outbox.sql` | 60K | 3.1-3.3 + IMPL-001 §3 + shared-platform::outbox |

### 3.4 UT 簇(4.1-4.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 4.1 | SagaOrchestrator UT(6 场景) | `crates/rgs-saga-orchestrator/tests/ut_orchestrator.rs` | 400K | 2.1 + rgs-testkit + RGS-REV-005 附件 B |
| 4.2 | Saga 事务日志 UT | `crates/rgs-saga-orchestrator/tests/ut_log.rs` | 100K | 2.2 + rgs-testkit |
| 4.3 | Saga 步骤注册中心 UT | `crates/rgs-saga-orchestrator/tests/ut_registry.rs` | 100K | 2.3 + rgs-testkit + NFR-SAG-006 |
| 4.4 | 跨域补偿 + 重入 idempotency UT | `crates/rgs-saga-orchestrator/tests/ut_compensation.rs` | 200K | 2.4 + rgs-testkit + NFR-SAG-004 + NFR-SAG-005 |

### 3.5 IT 簇(5.1-5.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 5.1 | saga_db 集成 | `crates/rgs-saga-orchestrator/tests/it_saga_db.rs` | 100K | 3.1-3.4 + rgs-testkit |
| 5.2 | saga → 5 域步骤注册集成 | `crates/rgs-saga-orchestrator/tests/it_5_domains.rs` | 200K | 2.3 + 5 域 test container + NFR-SAG-006 |
| 5.3 | Q-003 跨 DB Saga 集成 | `crates/rgs-saga-orchestrator/tests/it_cross_db.rs` | 200K | 2.4 + DEC-Q003 + economy test container |
| 5.4 | saga → admin 集成 | `crates/rgs-saga-orchestrator/tests/it_admin.rs` | 100K | 2.4 + admin test container + DTL-040 v0.1 |

### 3.6 ST 簇(6.1-6.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 6.1 | 端到端 6 场景演练 | `tests/st/saga_e2e.rs` | 400K | 5.1-5.4 + K3s namespace + RGS-REV-005 附件 B |
| 6.2 | NFR-SAG-002/005 实测 | `tests/st/saga_log_reentry.rs` | 100K | 6.1 + prometheus |
| 6.3 | AC-SAG-001~012 | `tests/st/saga_ac.rs` | 80K | 6.1 + check-docs-consistency.sh |
| 6.4 | 故障注入 | `tests/st/saga_chaos.rs` | 80K | 6.1 + chaos-mesh |

### 3.7 Helm chart 簇(7.1-7.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 7.1 | saga-orchestrator Helm chart | `deploy/helm/rgs-saga-orchestrator/Chart.yaml` | 80K | 2.1-2.4 + Helm |
| 7.2 | saga_db StatefulSet | `deploy/helm/rgs-saga-db/Chart.yaml` | 80K | 3.1-3.4 + Helm 依赖 |
| 7.3 | 5 独立 PG ConfigMap | `deploy/helm/rgs-shared-pg/configmap.yaml` | 60K | 7.1 + 7.2 |
| 7.4 | values.yaml 多环境 | `deploy/helm/rgs-saga-orchestrator/values.yaml` | 60K | 7.1-7.3 + sealed-secrets |

### 3.8 observability 簇(8.1-8.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 8.1 | OTel trace_id 传播 | `crates/rgs-saga-orchestrator/src/observability/trace.rs` | 80K | 2.1-2.4 + shared-platform::tracing |
| 8.2 | 15 项 rgs_saga_* 指标 | `crates/rgs-saga-orchestrator/src/observability/metrics.rs` | 80K | 2.1-2.4 + prometheus |
| 8.3 | 8 项 rgs_saga_* 日志 | `crates/rgs-saga-orchestrator/src/observability/log.rs` | 60K | 2.1-2.4 + tracing-subscriber |
| 8.4 | Grafana dashboard | `deploy/grafana/saga-dashboard.json` | 60K | 8.1-8.3 + Grafana |

---

## 4. RACI 责任矩阵

| 任务簇 \ 角色 | saga 域 Lead | player 域 Lead | economy 域 Lead | match 域 Lead | social 域 Lead | admin 域 Lead | foundation Lead | cluster-ops Lead | shared-platform Lead |
|---|---|---|---|---|---|---|---|---|---|
| API Spec | **R/A** | I | I | I | I | I | C | I | C(SPEC-CROSS-002) |
| 业务逻辑 | **R/A** | C(2.3 步骤注册) | C(2.3 步骤注册 + 2.4 reference) | C(2.3 步骤注册) | C(2.3 步骤注册) | C(2.3 步骤注册 + 2.4 GM 补偿) | I | I | C(Outbox) |
| DB migration | **R/A** | I | I | I | I | I | I | C(7.2 StatefulSet) | C(命名规范 + Outbox) |
| UT | **R/A** | I | I | I | I | I | C(rgs-testkit) | I | C |
| IT | **R/A** | C(5.2 步骤) | **R/A**(5.3 跨 DB Saga reference) | C(5.2 步骤) | C(5.2 步骤) | C(5.2 + 5.4) | C(test container) | C(K3s) | C |
| ST | **R/A** | C | C | C | C | C | C | C(K3s) | C(OTel) |
| Helm chart | C(7.1/7.2 需求) | I | I | I | I | I | I | **R/A** | I |
| observability | C(8.1-8.4 需求) | I | I | I | I | I | I | C(dashboard 部署) | **R/A**(OTel/trace_id) |

> 注 1:saga 域 **不**与其他域 Lead 兼任(per 2026-08-21 一人公司架构师兼任拒绝证据);即使一人公司 12 角色兼任,saga 域 A 角色 = Ulysses 显式签字(per RGS-ADR-0055 §4 + DEC-008)。
>
> 注 2:IT 5.3 Q-003 跨 DB Saga 集成由 **saga + economy 双 R/A**(saga 协调 + economy reference 共担);economy 域 Lead 在此场景作为 A 角色之一(per RGS-IMPL-PLAN-ECONOMY-001 §4 RACI)。
>
> 注 3:业务逻辑 2.3 步骤注册由 saga 域做注册中心,**5 域只读步骤定义**(per NFR-SAG-006 + DEC-008 域独立基线,不允许反向覆盖)。

---

## 5. Rollback 回滚路径

### 5.1 应用回滚

- saga-orchestrator **不**是必选路径——若上线后出现 Saga 协调回归:
  1. `k8s rollout undo deployment/rgs-saga-orchestrator -n saga`
  2. 触发 PFAU 编排(per ARC-051)切换回上一 PFAU Feature 版本
  3. 监控:AC-SAG-001~012 门禁自动告警
- **Saga reference 实现不破坏**——saga 域不实现 reference,saga 域实现若回归,5 域 reference 实现回退到上一 PFAU 版本(per DEC-008)

### 5.2 DB migration 回滚

- 3 张 saga 表均为 idempotent + reversible(per BAS-007 §3.4)
- **saga_step_log 事务日志表不可 DROP**——per NFR-SAG-002 硬约束,即使 reverse migration 也仅可 ADD COLUMN / DROP COLUMN,**不可** DROP TABLE
- reverse migration 通过新建 `9999_rollback_*.sql`(per IMPL-001 §3.4)

### 5.3 plugin 回滚

- saga-orchestrator **无 plugin 加载点**(per BAS-005 边界)
- Saga 步骤通过 SagaStepRegistry 运行时注册,允许热插拔步骤实现(per DTL-102 v0.1 §3)

### 5.4 配置回滚

- Helm `values.yaml` 多环境回滚经 `helm rollback rgs-saga-orchestrator <revision>`

### 5.5 Collector 回滚

- OTel Collector 配置由 cluster-ops 域统一管理

### 5.6 dashboard 回滚

- Grafana dashboard JSON 通过 Grafana provisioning 双向同步

---

## 6. 验收项

### 6.1 CI 4 workflow(per RGS-IMPL-006)

- [ ] `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test --all` / `cargo deny check` 全部 exit 0

### 6.2 文档一致性

- [ ] `check-docs-consistency.sh` 通过
- [ ] 17 份 v0.2 SPEC 引用一致(SPEC-DTL-100/101/102 + SPEC-CROSS-001~007 + SPEC-DTL-015/016/037 联动)
- [ ] RGS-DTL-100 v0.2 + DTL-101 v0.1 + DTL-102 v0.1 引用一致
- [ ] RGS-REQ-100 Saga 事务系统 v0.1 + RGS-DEC-Q003 跨 DB Saga + RGS-IMPL-100 Saga 实施规范 v0.1 + RGS-REV-005 附件 B 引用一致
- [ ] RGS-DTL-015 v0.2 §3.4 Saga 步骤编号映射 + RGS-DTL-016 v0.3 Saga 异常分支引用一致

### 6.3 域硬约束

- [ ] FR-SAG-001(6 场景全覆盖)UT 覆盖 + RGS-REV-005 附件 B 演练
- [ ] NFR-SAG-002(事务日志完整性)实测 + 代码评审 grep
- [ ] NFR-SAG-003(单调解者原则)代码评审 grep
- [ ] NFR-SAG-004(跨 DB Saga 经 DEC-Q003 治理)集成测试
- [ ] NFR-SAG-005(重入 idempotency)UT + IT 覆盖
- [ ] NFR-SAG-006(不允许反向覆盖)代码评审 grep
- [ ] NFR-SAG-007(经 shared-platform::outbox 落地)代码评审 grep

### 6.4 验收门槛

- [ ] AC-SAG-001~012 全部 12 项达标
- [ ] 6 场景全部在 ST 6.1 演练通过(per RGS-REV-005 附件 B)
- [ ] Q-003 跨 DB Saga 决策经 DEC-Q003 治理(per RGS-DEC-Q003)
- [ ] economy 域 reference 实现经 saga 域 reference 通过

---

## 7. Definition of Done

per RGS-SPEC-DTL-100/101/102 v0.2 §7 + RGS-SPEC-000 v0.3 §4 + RGS-IMPL-100 v0.1:

- [ ] 32 个 L4 任务全部完成 + commit 落地
- [ ] 6 份 CR 全部通过
- [ ] CI 4 workflow 全过
- [ ] ST 6.1 端到端 6 场景演练通过
- [ ] ST 6.2 NFR-SAG-002 事务日志完整性 + NFR-SAG-005 重入 idempotency 实测达标
- [ ] ST 6.3 AC-SAG-001~012 全部 12 项达标
- [ ] ST 6.4 故障注入恢复路径验证
- [ ] Helm chart 7.1-7.4 在 K3s 集群通过
- [ ] observability 8.1-8.4 在 staging 集群采集数据
- [ ] check-docs-consistency.sh 通过
- [ ] 17 份 v0.2 SPEC 引用一致
- [ ] RACI 责任矩阵 72 单元全部登记
- [ ] Rollback 6 路径实测演练通过
- [ ] 当前无实现文件时保持"待实现/待评审"状态(per RGS-SPEC-000 §5 第 7 条)
- [ ] **saga 域不允许反向覆盖 5 域 reference 实现**(per NFR-SAG-006 + DEC-008 域独立基线)
- [ ] **saga 域 Lead 不得与其他域 Lead 兼任**(per 2026-08-21 兼任拒绝证据,RACI A 角色必须 Ulysses 显式签字)

---

## 8. Gate 证据与实测参数

### 8.1 CI 证据

- CI-FMT / CI-LINT / CI-TEST / CI-DENY 全部 exit 0

### 8.2 ST 证据

- **ST-6.1 E2E 6 场景**:K3s namespace `saga-st` 部署成功 + 1.0 单一事务 / 2.0 顺序 Saga / 3.0 并行 Saga / 4.0 嵌套 Saga / 5.0 跨域补偿 / 6.0 失败重试 全部端到端演练通过(per RGS-REV-005 附件 B)
- **ST-6.2 NFR**:事务日志 100% 完整性 + 重入 idempotency 100% 幂等
- **ST-6.3 AC**:AC-SAG-001~012 全部 12 项达标
- **ST-6.4 Chaos**:5 类故障注入(跨 DB 失败 / Outbox 重复 / 重入幂等冲突 / 步骤注册异常 / 单调解者违反)全部恢复路径验证

### 8.3 Helm 证据

- 7.1-7.4 K3s 多环境部署通过

### 8.4 observability 证据

- OTel trace_id 完整链路
- Prometheus `rgs_saga_*` 15 项指标 + Loki `rgs_saga_*` 8 项字段
- Grafana `saga-dashboard.json` 4 panel(场景 / 步骤 / 补偿 / Outbox)

### 8.5 Rollback 证据

- 应用 / DB / Collector / dashboard 4 路径在 staging 演练通过
- **saga_step_log 事务日志表在 reverse migration 中保持不 DROP** 验证
- **saga 域不破坏 5 域 reference 实现** 验证

---

## 9. 关联文档

### 9.1 上行

- [RGS-DTL-100 Saga 协调器 v0.2](../02-运维安全与网络/RGS-DTL-100_详细设计书.md)
- [RGS-DTL-101 Saga 事务日志 v0.1](../02-运维安全与网络/RGS-DTL-101_详细设计书.md)
- [RGS-DTL-102 Saga 步骤注册 v0.1](../02-运维安全与网络/RGS-DTL-102_详细设计书.md)
- [RGS-SPEC-DTL-100/101/102 实现规格 v0.2](../13-实现规格/RGS-SPEC-DTL-100_实现规格书.md)
- [RGS-DTL-015 v0.2 §3.4 Saga 步骤编号映射](../03-数据经济与交易/RGS-DTL-015_详细设计书.md)
- [RGS-DTL-016 v0.3 Saga 异常分支](../03-数据经济与交易/RGS-DTL-016_详细设计书.md)
- [RGS-REQ-100 Saga 事务系统 v0.1](../00-基准与治理/requirements/RGS-REQ-100_Saga事务系统需求定义_v0.1.md)
- [RGS-DEC-Q003 跨 DB Saga](../00-基准与治理/RGS-DEC-Q003_跨DBSaga决策_v0.1.md)
- [RGS-IMPL-100 Saga 实施规范 v0.1](../13-实现规格/RGS-IMPL-100_Saga事务系统实施规范_v0.1.md)
- [RGS-REV-005 附件 B 6 场景演练](../00-基准与治理/reviews/RGS-REV-005_附件B_Saga场景演练_v0.1.md)
- [RGS-SPEC-CROSS-003 跨域事件 Schema 字典 v0.2](../13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md)
- [RGS-ADR-0015 Saga 适用边界 + 单一调解者原则](../00-基准与治理/RGS-DEC-Q003_跨DBSaga决策_v0.1.md)

### 9.2 下行

- [RGS-IMPL-001 实施约定](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
- [RGS-IMPL-002 PG 编码规范 v0.1](../13-实现规格/RGS-IMPL-002_PG_编码规范_v0.1.md)
- [RGS-IMPL-004 CR 代码审查规范 v0.1](../13-实现规格/RGS-IMPL-004_CR_代码审查规范_v0.1.md)
- [RGS-IMPL-005 BUILD 构建规范 v0.1](../13-实现规格/RGS-IMPL-005_BUILD_构建规范_v0.1.md)
- [RGS-IMPL-006 CI 持续集成规范 v0.1](../13-实现规格/RGS-IMPL-006_CI_持续集成规范_v0.1.md)

### 9.3 同级(5 域 IMPL-PLAN 联动)

- [RGS-IMPL-PLAN-PLAYER-001 player 域实施计划](RGS-IMPL-PLAN-PLAYER-001_player域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ECONOMY-001 economy 域实施计划](RGS-IMPL-PLAN-ECONOMY-001_economy域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-MATCH-001 match 域实施计划](RGS-IMPL-PLAN-MATCH-001_match域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-SOCIAL-001 social 域实施计划](RGS-IMPL-PLAN-SOCIAL-001_social域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ADMIN-001 admin 域实施计划](RGS-IMPL-PLAN-ADMIN-001_admin域实施计划_v0.1.md)

### 9.4 模板参考

- [RGS-IMPL-PLAN-CDN-001 断点续传实施计划 v0.1](RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md)
- [RGS-IMPL-PLAN-LCM-001 服务器全生命周期实施计划 v0.1](RGS-IMPL-PLAN-LCM-001_服务器全生命周期实施计划_v0.1.md)

---

## A. v0.1 对齐说明

### A.1 触发

WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草(saga 域 1 份)

### A.2 范围

- saga 域各 1 份 IMPL-PLAN v0.1,32 L4 任务占位齐全
- 6 域合计 32 L4 × 6 = 192 L4 任务占位
- 实施范围:**仅汇编**各域既有 DTL + 17 份 v0.2 SPEC,**不引入新设计**

### A.3 已知缺口

- 各域实际 L4 任务待 DDD Review 阶段补完
- 5 域 Lead 签字(本 v0.1 占位,等 Ulysses DDD Review 后补签)
- OLU token 估算(WF-1-55.55 task 5 联动)
- TBD-SAG-101~107(7 项 TBD)待 PH-3 实测填入
- DTL-100/101/102 §3.4/§4/§5 Saga 步骤编号映射依赖 RGS-REV-005 附件 B 6 场景演练(已在 SPEC-DTL-100/101/102 v0.2 §A.3 列已知缺口)
- **不**反向覆盖 5 域 reference 实现(per NFR-SAG-006 + DEC-008 域独立基线)
- economy 域 `saga_orchestrator` reference 实现**只读依赖** saga 域

### A.4 引用链

- 17 份 v0.2 SPEC(commit `756bcd3` ~ `97ef67c`,per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.1)
- RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-IMPL-001
- DEC-008 一人公司 12 角色治理基线
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束
- **saga 域独立位**(per 2026-08-21 一人公司架构师兼任拒绝证据 + DEC-008)
