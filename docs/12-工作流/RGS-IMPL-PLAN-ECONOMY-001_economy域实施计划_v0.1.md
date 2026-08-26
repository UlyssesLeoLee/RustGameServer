# RGS-IMPL-PLAN-ECONOMY-001 economy 域实施计划

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-ECONOMY-001 |
| 版本 | 0.2 |
| 父文档 | RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-PLAN-001 v1.0 + RGS-IMPL-001 工程约定 |
| 源详细设计 | RGS-DTL-037 Economy 域 v0.2(commit `7e961ee` WF-1-55-59)+ RGS-DTL-015 v0.2(commit `756bcd3` Saga 步骤编号映射)+ RGS-DTL-016 v0.3(commit `756bcd3` Saga 步骤编号映射)+ RGS-SPEC-DTL-015/016/037 实现规格 v0.2 + RGS-REQ-100 Saga 事务系统 v0.1 |
| 适用范围 | economy 域 Atomic App 全生命周期实施(economy-service crate + economy_db 库 + 6 张经济表 + Saga 编排 reference 实现) |
| 目标基线 | Rust 1.98 + Actix Web 4.14.1 + PostgreSQL 18.6 + K3s |
| 责任人 | economy 域 Lead(Ulysses per DEC-008 一人公司 12 角色兼任) |
| 触发 | WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草 | (本 v0.2 = v0.1 + DDD Review 反馈 + §3 RACI 矩阵 + §A 已知缺口 3 段)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 初版:economy 域实施计划;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);**不引入新设计**——汇编 DTL-015/016/037 + SPEC-DTL-015/016/037 v0.2 + DTL-100 联动 |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | v0.2 升版:DDD Review 反馈 + §3 RACI 矩阵(per RGS-LEAD-RACI-001 v1.1)+ §A 已知缺口 3 段(跨域协调 / 实时审计 / 1 人 12 角色 RACI 全覆盖);**不引入新设计**——本 v0.2 仅在 v0.1 头表 + 修订历史 + §3 + §A 加内容,正文本(域职责/实施阶段/验收)不动 | 头部 + 修订 + §3 + §A(新增) | 全部 |

---


---

## §3 RACI 矩阵 (NEW, v0.2 升版增量, per RGS-LEAD-RACI-001 v1.1 §3)

本域 economy 的 6 治理角色 × 7 实施任务 RACI 映射（per RGS-ADR-0055 v0.1 §4）:

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

### §A.1 economy 域 跨域协调依赖

本 economy 域 IMPL-PLAN 涉及跨域 gRPC 调用（player → economy/match/social/admin + saga）需 5 域 binary 全部启 + 跨域联调通过才能完整验证。当前阻塞：
- PostgreSQL 18.6 未装（per Ulysses 16:58/16:59 硬约束，等装入）
- 5 域 binary 编译完成但启需 DATABASE_URL（per `RGS-GM-V0.3-DEPLOY-SOP-2026-08-26.md` v0.1）
- 跨域联调 IT 测试（per RGS-TEST-STRATEGY 4 阶段 phase 2，等 PG 装完）

### §A.2 实时审计跟踪

本 economy 域 IMPL-PLAN 涉及 GM 操作 / RBAC 权限变更 / 跨域事件触发等操作需实时审计跟踪。依赖:
- ARC-018/021/042/051 4 治理角色（per RGS-ADR-0055 v0.1 §4）
- audit log 落库（per DTL-031 事件总线 + audit_log 表）
- 实时审计 dashboard（per rgs-web GM 后台 §3.5）

当前状态：审计跟踪设计在 DTL-031 §4，但 dashboard UI 未实装。

### §A.3 一人公司 12 角色 RACI 全覆盖

本 economy 域 IMPL-PLAN v0.2 §3 RACI 矩阵仅含 6 治理角色（Arch/BE Lead/SRE Lead/DBA/PM/PO），缺:
- FE Lead（前后端边界，本域为后端无 FE）
- QA Lead（per RGS-TEST-STRATEGY 4 阶段）
- SEC（per RGS-REV-008 mTLS fail-closed）
- SRE（SRE Lead + SRE 角色区分，per DEC-008）
- 1 人 12 角色（per DEC-008 一人公司治理基线，5 域 Lead 全部 = Ulysses 兼任）

本缺口待 5 域 binary 启 + DDD Review 反馈 + RACI-001 v1.2 升版时统一补全。

## 1. 域职责

per RGS-DTL-037 v0.2 §1「Economy 域」+ RGS-SPEC-DTL-037 v0.2 §1「实施范围」+ RGS-DTL-015 v0.2 §3.4 Saga 步骤编号映射(1.0~6.0):

economy 域是 RGS 5 域中**经济与交易**核心域,职责覆盖:

- **6 张经济表** CRUD(per RGS-DTL-037 §3 表清单):player_wallet / item_inventory / order / trade_log / currency_ledger / quota_account
- **Saga 编排 reference 实现**——`economy-service::saga_orchestrator` 是 DTL-100/101/102 Saga 事务系统的参考实现,提供 `apply_atomic_with_reservation` + `Outbox` 模式(per RGS-DTL-015 v0.2 §3.4)
- **跨域经济联动**——为 match 域(比赛奖励)、social 域(礼物/打赏)、admin 域(补偿/扣款)提供 gRPC 客户端
- **跨 DB Saga**——Q-003 跨 DB Saga 决策 per RGS-DEC-Q003,涉及 player_db + economy_db 双 DB
- **Saga 步骤编号映射**——1.0~6.0 对应 RGS-REV-005 附件 B 6 场景演练(per DTL-015 v0.2 §3.4 新增)
- **player 间交易系统**——per RGS-REQ-018 玩家间交易系统 + RGS-DTL-015 v0.2

**域边界(per DTL-037 §1.2)**:
- ❌ **不**持有 player 身份/账户数据(归 player 域)
- ❌ **不**持有匹配/对局数据(归 match 域)
- ❌ **不**实现跨域 Saga 协调规范本身(归 saga 域 DTL-100/101/102,本域仅提供 reference 实现)
- ❌ **不**实现 GM/审计/合规删除(归 admin 域)
- ✅ **仅**做经济事务原子性保证 + 6 张经济表 CRUD + 跨域经济联动

**关键硬约束(per SPEC-DTL-037 v0.2 §3 + DTL-015 v0.2 + DTL-016 v0.3)**:

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-ECO-001 | 6 张经济表双写一致性(per Q-003 Saga 模式) | 既有 |
| NFR-ECO-002 | 经济事务原子性(per `apply_atomic_with_reservation`) | 硬约束 |
| NFR-ECO-003 | Outbox 模式必经(per DTL-015 v0.2) | 硬约束 |
| NFR-ECO-004 | 跨域事件 schema 必须经 SPEC-CROSS-003 v0.2 校验 | 硬约束 |
| AC-ECO-001~010 | 10 项验收门槛 | 实测 |
| TBD-ECO-101~105 | 5 项 TBD | PH-3 实测填 |
| TBD-DTL-037-01~03 | 3 项 TBD(per DTL-037 §6 既有) | 已知缺口 |
| Saga 步骤 1.0~6.0 | per RGS-DTL-015 v0.2 §3.4 + RGS-REV-005 附件 B 6 场景 | 引用基线 |

---

## 2. 实施阶段(8 任务簇 × 4 L4 任务 = 32 L4)

| 任务簇 | 任务编号 | 任务名 | owner | 工期 | 依赖 |
|---|---|---|---|---|---|
| **API Spec** | 1.1 | economy 域 gRPC Proto 定义(wallet / inventory / order / trade / currency / quota 6 模块) | economy 域 Lead | 0.5 人·天 | 父 BAS-001 v1.4 + SPEC-CROSS-002 v0.2 |
| API Spec | 1.2 | Saga 步骤编号映射 1.0~6.0 Proto 字段(per DTL-015 v0.2 §3.4) | economy 域 Lead | 0.5 人·天 | 1.1 + REV-005 附件 B |
| API Spec | 1.3 | 错误码映射(per SPEC-CROSS-001 v0.2 + DTL-001 §3.4 ADR-0057) | economy 域 Lead | 0.5 人·天 | 1.1 + 1.2 |
| API Spec | 1.4 | OpenAPI 3.1 + 跨域事件 schema(per SPEC-CROSS-003 v0.2) | economy 域 Lead | 0.5 人·天 | 1.1-1.3 |
| **业务逻辑** | 2.1 | 6 张经济表 Service 层(Saga reference 实现) | economy 域 Lead | 1.5 人·天 | 1.1-1.4 + DTL-037 §3 |
| 业务逻辑 | 2.2 | `apply_atomic_with_reservation` + `Outbox` 实现(per DTL-015 v0.2) | economy 域 Lead | 1.5 人·天 | 2.1 + shared-platform::outbox |
| 业务逻辑 | 2.3 | 跨域经济联动(match / social / admin gRPC client) | economy 域 Lead | 1 人·天 | 2.1 + match/social/admin test container |
| 业务逻辑 | 2.4 | 玩家间交易系统(per RGS-REQ-018 + DTL-015 v0.2 §3.4) | economy 域 Lead | 1 人·天 | 2.1 + 2.2 + RBAC |
| **DB migration** | 3.1 | economy_db 库 + 5 独立 PG 18.6 元数据(per WBS §2A.1) | economy 域 Lead | 0.5 人·天 | DB Pool 治理基线 |
| DB migration | 3.2 | 6 张经济表 DDL(wallet / inventory / order / trade_log / currency_ledger / quota_account) | economy 域 Lead | 1 人·天 | 3.1 + BAS-007 §3.2 + DTL-037 §3 |
| DB migration | 3.3 | currency_ledger 按 created_at 月度范围分区(per DTL-007 §3 + DTL-016 v0.3) | economy 域 Lead | 0.5 人·天 | 3.2 + sqlx 集成 |
| DB migration | 3.4 | migration 工具链 + Outbox 表 | economy 域 Lead | 0.5 人·天 | 3.1-3.3 + IMPL-001 §3 |
| **UT** | 4.1 | 6 张表 Service UT(CRUD + Saga reference 模式) | economy 域 Lead | 1 人·天 | 2.1 + rgs-testkit |
| UT | 4.2 | `apply_atomic_with_reservation` UT(per NFR-ECO-002 硬约束) | economy 域 Lead | 1 人·天 | 2.2 + DTL-015 v0.2 |
| UT | 4.3 | Outbox 模式 UT(per NFR-ECO-003 硬约束) | economy 域 Lead | 0.5 人·天 | 2.2 + shared-platform::outbox |
| UT | 4.4 | 玩家间交易系统 UT(per RGS-REQ-018) | economy 域 Lead | 0.5 人·天 | 2.4 + rgs-testkit |
| **IT** | 5.1 | economy_db 集成测试(sqltest + 5 独立 PG 池) | economy 域 Lead | 0.5 人·天 | 3.1-3.4 + rgs-testkit |
| IT | 5.2 | Q-003 跨 DB Saga 集成(player_db + economy_db,per RGS-DEC-Q003) | economy 域 Lead | 1 人·天 | 2.2 + saga-orchestrator test container |
| IT | 5.3 | economy → match / social 集成(比赛奖励 / 礼物 gRPC) | economy 域 Lead | 0.5 人·天 | 2.3 + match/social test container |
| IT | 5.4 | economy → admin 集成(补偿 / 扣款 + Saga 审计,per NFR-SE-010) | economy 域 Lead | 0.5 人·天 | 2.3 + admin test container |
| **ST** | 6.1 | 端到端:注册→钱包充值→购买道具→跨域奖励(ST harness) | economy 域 Lead | 1 人·天 | 5.1-5.4 + K3s namespace |
| ST | 6.2 | NFR-ECO-002 原子性实测(100 万级并发事务) | economy 域 Lead | 0.5 人·天 | 6.1 + prometheus |
| ST | 6.3 | AC-ECO-001~010 全部 10 项达标 | economy 域 Lead | 0.5 人·天 | 6.1 + check-docs-consistency.sh |
| ST | 6.4 | 故障注入:跨 DB Saga 失败补偿 / Outbox 重复发送 | economy 域 Lead | 0.5 人·天 | 6.1 + chaos-mesh |
| **Helm chart** | 7.1 | economy-service Helm chart 骨架(per ARC-051) | cluster-ops 域 Lead | 0.5 人·天 | 2.1-2.4 + Helm 模板 |
| Helm chart | 7.2 | economy_db Helm chart 依赖 + StatefulSet + PVC | cluster-ops 域 Lead | 0.5 人·天 | 3.1-3.4 + Helm 依赖 |
| Helm chart | 7.3 | 5 独立 PG 元数据 ConfigMap(per WBS §2A.1) | cluster-ops 域 Lead | 0.5 人·天 | 7.1 + 7.2 |
| Helm chart | 7.4 | values.yaml 多环境 + sealed-secrets 治理 | cluster-ops 域 Lead | 0.5 人·天 | 7.1-7.3 + sealed-secrets |
| **observability** | 8.1 | OTel trace_id 传播(per SPEC-CROSS-006 v0.2) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + shared-platform::tracing |
| observability | 8.2 | 12 项 rgs_economy_* 指标(QPS / 延迟 / 错误率 / 事务成功率 / Outbox lag) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + prometheus crate |
| observability | 8.3 | 6 项 rgs_economy_* 日志(transaction_id / saga_step / 跨域引用) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + tracing-subscriber |
| observability | 8.4 | Grafana dashboard JSON(事务 / 跨域事件 / Outbox) | SRE 域 Lead | 0.5 人·天 | 8.1-8.3 + Grafana provisioning |

**L4 合计**:32 个 L4 任务 / ~18 人·天(per RGS-TS-001 v0.6 §6.2 token-OLU,合计 ~3.6M-5.4M tokens)

---

## 3. 任务清单(32 L4 详细)

> per WBS v0.3 §6.2 拆分原则:每个 L4 任务 = 1 agent 最小可拆分,≤ 2 人·天 / ≤ 500K tokens

### 3.1 API Spec 簇(1.1-1.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 1.1 | economy 域 gRPC Proto(6 模块) | `crates/rgs-economy-service/proto/economy.proto` | 100K | BAS-001 v1.4 |
| 1.2 | Saga 步骤编号 1.0~6.0 Proto 字段 | `crates/rgs-economy-service/proto/saga_step.proto` | 80K | 1.1 + REV-005 附件 B |
| 1.3 | 错误码映射 | `crates/rgs-economy-service/src/error.rs` | 60K | 1.1 + 1.2 + SPEC-CROSS-001 |
| 1.4 | OpenAPI 3.1 + 跨域事件 schema | `crates/rgs-economy-service/openapi/economy.yaml` | 80K | 1.1-1.3 + SPEC-CROSS-003 |

### 3.2 业务逻辑簇(2.1-2.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 2.1 | 6 张经济表 Service | `crates/rgs-economy-service/src/service/{wallet,inventory,order,trade,currency,quota}.rs` | 400K | 1.1-1.4 + DTL-037 §3 |
| 2.2 | `apply_atomic_with_reservation` + Outbox | `crates/rgs-economy-service/src/saga/orchestrator.rs` | 400K | 2.1 + shared-platform::outbox + DTL-015 v0.2 |
| 2.3 | 跨域经济联动 gRPC client | `crates/rgs-economy-service/src/client/{match,social,admin}.rs` | 200K | 2.1 + test container |
| 2.4 | 玩家间交易系统 | `crates/rgs-economy-service/src/service/trade.rs` | 200K | 2.1 + 2.2 + RBAC + RGS-REQ-018 |

### 3.3 DB migration 簇(3.1-3.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 3.1 | economy_db 库 + 5 独立 PG | `crates/rgs-economy-service/migrations/0001_economy_db.sql` | 80K | DB Pool 治理基线 |
| 3.2 | 6 张经济表 DDL | `crates/rgs-economy-service/migrations/0002_eco_tables.sql` | 200K | 3.1 + BAS-007 §3.2 + DTL-037 §3 |
| 3.3 | currency_ledger 月分区 | `crates/rgs-economy-service/migrations/0003_partition.sql` | 100K | 3.2 + sqlx + DTL-016 v0.3 |
| 3.4 | migration 工具链 + Outbox 表 | `crates/rgs-economy-service/migrations/0004_outbox.sql` | 60K | 3.1-3.3 + IMPL-001 §3 |

### 3.4 UT 簇(4.1-4.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 4.1 | 6 张表 Service UT | `crates/rgs-economy-service/tests/ut_eco_service.rs` | 200K | 2.1 + rgs-testkit |
| 4.2 | `apply_atomic_with_reservation` UT | `crates/rgs-economy-service/tests/ut_atomic.rs` | 200K | 2.2 + DTL-015 v0.2 |
| 4.3 | Outbox 模式 UT | `crates/rgs-economy-service/tests/ut_outbox.rs` | 100K | 2.2 + shared-platform::outbox |
| 4.4 | 玩家间交易系统 UT | `crates/rgs-economy-service/tests/ut_trade.rs` | 100K | 2.4 + rgs-testkit + RGS-REQ-018 |

### 3.5 IT 簇(5.1-5.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 5.1 | economy_db 集成 | `crates/rgs-economy-service/tests/it_economy_db.rs` | 100K | 3.1-3.4 + rgs-testkit |
| 5.2 | Q-003 跨 DB Saga 集成 | `crates/rgs-economy-service/tests/it_saga.rs` | 200K | 2.2 + saga test container + RGS-DEC-Q003 |
| 5.3 | economy → match/social 集成 | `crates/rgs-economy-service/tests/it_match_social.rs` | 100K | 2.3 + match/social test container |
| 5.4 | economy → admin 集成 | `crates/rgs-economy-service/tests/it_admin.rs` | 100K | 2.3 + admin test container + NFR-SE-010 |

### 3.6 ST 簇(6.1-6.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 6.1 | 端到端 harness | `tests/st/economy_e2e.rs` | 200K | 5.1-5.4 + K3s namespace |
| 6.2 | NFR-ECO-002 原子性实测 | `tests/st/economy_atomic.rs` | 100K | 6.1 + prometheus |
| 6.3 | AC-ECO-001~010 | `tests/st/economy_ac.rs` | 80K | 6.1 + check-docs-consistency.sh |
| 6.4 | 故障注入(Saga 失败补偿) | `tests/st/economy_chaos.rs` | 80K | 6.1 + chaos-mesh |

### 3.7 Helm chart 簇(7.1-7.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 7.1 | economy-service Helm chart | `deploy/helm/rgs-economy-service/Chart.yaml` | 80K | 2.1-2.4 + Helm |
| 7.2 | economy_db StatefulSet | `deploy/helm/rgs-economy-db/Chart.yaml` | 80K | 3.1-3.4 + Helm 依赖 |
| 7.3 | 5 独立 PG ConfigMap | `deploy/helm/rgs-shared-pg/configmap.yaml` | 60K | 7.1 + 7.2 |
| 7.4 | values.yaml 多环境 | `deploy/helm/rgs-economy-service/values.yaml` | 60K | 7.1-7.3 + sealed-secrets |

### 3.8 observability 簇(8.1-8.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 8.1 | OTel trace_id 传播 | `crates/rgs-economy-service/src/observability/trace.rs` | 80K | 2.1-2.4 + shared-platform::tracing |
| 8.2 | 12 项 rgs_economy_* 指标 | `crates/rgs-economy-service/src/observability/metrics.rs` | 80K | 2.1-2.4 + prometheus |
| 8.3 | 6 项 rgs_economy_* 日志 | `crates/rgs-economy-service/src/observability/log.rs` | 60K | 2.1-2.4 + tracing-subscriber |
| 8.4 | Grafana dashboard | `deploy/grafana/economy-dashboard.json` | 60K | 8.1-8.3 + Grafana |

---

## 4. RACI 责任矩阵

| 任务簇 \ 角色 | economy 域 Lead | player 域 Lead | match 域 Lead | social 域 Lead | admin 域 Lead | saga 域 Lead | foundation Lead | cluster-ops Lead | shared-platform Lead |
|---|---|---|---|---|---|---|---|---|---|
| API Spec | **R/A** | I | I | I | I | C(1.2 Saga 编号) | C | I | C(SPEC-CROSS-002) |
| 业务逻辑 | **R/A** | C(2.3 钱包联动) | C(2.3 比赛奖励) | C(2.3 礼物) | C(2.3 补偿) | C(2.2 Saga reference) | I | I | C(Outbox) |
| DB migration | **R/A** | I | I | I | I | C(Outbox 表) | I | C(7.2 StatefulSet) | C(命名规范) |
| UT | **R/A** | I | I | I | I | C(4.2 Saga UT) | C(rgs-testkit) | I | C |
| IT | **R/A** | C(5.2 跨 DB) | C(5.3) | C(5.3) | C(5.4) | **R/A**(5.2 Q-003 Saga) | C(test container) | C(K3s) | C |
| ST | **R/A** | C | C | C | C | C(Saga 演练) | C | C(K3s) | C(OTel) |
| Helm chart | C(7.1/7.2 需求) | I | I | I | I | I | I | **R/A** | I |
| observability | C(8.1-8.4 需求) | I | I | I | I | C(8.2 Saga 指标) | I | C(dashboard 部署) | **R/A**(OTel/trace_id) |

> 注:IT 簇 5.2 Q-003 跨 DB Saga 集成由 **economy + saga 双 R/A**(经济事务 + Saga 编排 reference 共担);saga 域 Lead 在此场景作为 A 角色之一。

---

## 5. Rollback 回滚路径

> 应用 / DB migration / plugin / 配置 / Collector / dashboard 独立回滚

### 5.1 应用回滚

- economy-service **不**是必选路径——若上线后出现 Saga 事务回归:
  1. `k8s rollout undo deployment/rgs-economy-service -n economy`
  2. 触发 PFAU 编排(per ARC-051)切换回上一 PFAU Feature 版本
  3. 监控:AC-ECO-001~010 门禁自动告警
- **Saga reference 实现回滚**——若 DTL-100/101/102 v0.2 升版时 Saga 模式变化,reference 实现需同步升级(本 v0.1 阶段 reference 实现仅供 saga 域参考,不允许反向覆盖 saga 域)

### 5.2 DB migration 回滚

- 6 张经济表 + Outbox 表均为 idempotent + reversible(per BAS-007 §3.4)
- `0002_eco_tables.sql` reverse = `DROP TABLE` × 6
- `0003_partition.sql` reverse = `DROP PARTITION` × N
- `0004_outbox.sql` reverse = `DROP TABLE outbox`
- reverse migration 通过新建 `9999_rollback_*.sql`(per IMPL-001 §3.4)

### 5.3 plugin 回滚

- economy-service **无 plugin 加载点**(per BAS-005 边界)

### 5.4 配置回滚

- Helm `values.yaml` 多环境回滚经 `helm rollback rgs-economy-service <revision>`
- sealed-secrets 治理经双向加解密(per IMPL-002 §6)

### 5.5 Collector 回滚

- OTel Collector 配置由 cluster-ops 域统一管理
- 回滚经 cluster-ops 域 `kubectl apply -f otel-collector-prev.yaml -n observability`

### 5.6 dashboard 回滚

- Grafana dashboard JSON 通过 Grafana provisioning 双向同步
- 回滚经 `kubectl apply -f grafana-economy-dashboard-prev.json -n observability`

---

## 6. 验收项

### 6.1 CI 4 workflow(per RGS-IMPL-006)

- [ ] `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test --all` / `cargo deny check` 全部 exit 0

### 6.2 文档一致性

- [ ] `check-docs-consistency.sh` 通过(per RGS-WF-001 §5)
- [ ] 17 份 v0.2 SPEC 引用一致(SPEC-DTL-015/016/037 + SPEC-CROSS-001~007 + SPEC-DTL-100~102)
- [ ] RGS-DTL-037 v0.2 + DTL-015 v0.2 + DTL-016 v0.3 引用一致
- [ ] RGS-REQ-018 玩家间交易系统 / RGS-REQ-100 Saga 事务系统 / RGS-DEC-Q003 跨 DB Saga 引用一致

### 6.3 域硬约束

- [ ] NFR-ECO-002(原子性)实测通过 + 代码评审 grep
- [ ] NFR-ECO-003(Outbox 必经)代码评审 grep
- [ ] NFR-ECO-004(跨域事件 schema 经 SPEC-CROSS-003 校验)代码评审 grep
- [ ] FR-ECO-001(6 张表双写一致性)IT 验证

### 6.4 验收门槛

- [ ] AC-ECO-001~010 全部 10 项达标
- [ ] Saga 步骤 1.0~6.0 编号映射在 6 场景演练中全部覆盖(per DTL-015 v0.2 §3.4 + RGS-REV-005 附件 B)
- [ ] Q-003 跨 DB Saga 决策经 DEC-Q003 治理(per RGS-DEC-Q003)

---

## 7. Definition of Done

per RGS-SPEC-DTL-037 v0.2 §7 + RGS-SPEC-000 v0.3 §4:

- [ ] 32 个 L4 任务全部完成 + commit 落地
- [ ] 6 份 CR(per RGS-IMPL-004)全部通过
- [ ] CI 4 workflow 全过
- [ ] ST 6.1 端到端通过
- [ ] ST 6.2 NFR-ECO-002 原子性实测达标
- [ ] ST 6.3 AC-ECO-001~010 全部 10 项达标
- [ ] ST 6.4 Saga 失败补偿恢复路径验证
- [ ] Helm chart 7.1-7.4 在 K3s 集群通过
- [ ] observability 8.1-8.4 在 staging 集群采集数据
- [ ] check-docs-consistency.sh 通过
- [ ] 17 份 v0.2 SPEC 引用一致
- [ ] RACI 责任矩阵 72 单元全部登记
- [ ] Rollback 6 路径实测演练通过
- [ ] 当前无实现文件时保持"待实现/待评审"状态(per RGS-SPEC-000 §5 第 7 条)
- [ ] **Saga reference 实现不**反向覆盖 saga 域 DTL-100/101/102(只读依赖,per DEC-008 域独立基线)

---

## 8. Gate 证据与实测参数

### 8.1 CI 证据

- CI-FMT / CI-LINT / CI-TEST / CI-DENY 全部 exit 0

### 8.2 ST 证据

- **ST-6.1 E2E**:K3s namespace `economy-st` 部署成功 + 6 张表 CRUD + Saga reference 演练通过
- **ST-6.2 NFR**:100 万级并发事务 + 1000 QPS 持续 10 分钟 → 原子性 0 违反 + Outbox lag < 100ms
- **ST-6.3 AC**:AC-ECO-001~010 全部 10 项达标
- **ST-6.4 Chaos**:5 类故障注入(跨 DB 失败 / Outbox 重复 / 订单重复 / 库存负数 / 余额不足)全部补偿路径验证

### 8.3 Helm 证据

- 7.1-7.4 K3s 多环境部署通过
- `helm template` dev / staging / prod 三环境输出合法 YAML

### 8.4 observability 证据

- OTel trace_id 完整链路
- Prometheus `rgs_economy_*` 12 项指标 + Loki `rgs_economy_*` 6 项字段
- Grafana `economy-dashboard.json` 4 panel(事务 / Saga / 跨域事件 / Outbox)

### 8.5 Rollback 证据

- 应用 / DB / Collector / dashboard 4 路径在 staging 演练通过
- Saga reference 实现不破坏 saga 域 DTL-100/101/102 v0.2 既有契约

---

## 9. 关联文档

### 9.1 上行

- [RGS-DTL-015 v0.2 Saga 步骤编号映射](../03-数据经济与交易/RGS-DTL-015_详细设计书.md)
- [RGS-DTL-016 v0.3 Saga 步骤编号映射](../03-数据经济与交易/RGS-DTL-016_详细设计书.md)
- [RGS-DTL-037 Economy 域 v0.2](../03-数据经济与交易/RGS-DTL-037_Economy域_详细设计书.md)
- [RGS-SPEC-DTL-015/016/037 实现规格 v0.2](../13-实现规格/RGS-SPEC-DTL-015_实现规格书.md)
- [RGS-REQ-018 玩家间交易系统](../03-数据经济与交易/RGS-REQ-018_玩家间交易系统_需求定义书.md)
- [RGS-REQ-100 Saga 事务系统 v0.1](../00-基准与治理/requirements/RGS-REQ-100_Saga事务系统需求定义_v0.1.md)
- [RGS-DEC-Q003 跨 DB Saga](../00-基准与治理/RGS-DEC-Q003_跨DBSaga决策_v0.1.md)
- [RGS-SPEC-CROSS-003 跨域事件 Schema 字典 v0.2](../13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md)

### 9.2 下行

- [RGS-IMPL-001 实施约定](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
- [RGS-IMPL-002 PG 编码规范 v0.1](../13-实现规格/RGS-IMPL-002_PG_编码规范_v0.1.md)
- [RGS-IMPL-004 CR 代码审查规范 v0.1](../13-实现规格/RGS-IMPL-004_CR_代码审查规范_v0.1.md)
- [RGS-IMPL-005 BUILD 构建规范 v0.1](../13-实现规格/RGS-IMPL-005_BUILD_构建规范_v0.1.md)
- [RGS-IMPL-006 CI 持续集成规范 v0.1](../13-实现规格/RGS-IMPL-006_CI_持续集成规范_v0.1.md)

### 9.3 同级(5 域 IMPL-PLAN 联动)

- [RGS-IMPL-PLAN-PLAYER-001 player 域实施计划](RGS-IMPL-PLAN-PLAYER-001_player域实施计划_v0.1.md)
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

WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草(economy 域 1 份)

### A.2 范围

- economy 域各 1 份 IMPL-PLAN v0.1,32 L4 任务占位齐全
- 6 域合计 32 L4 × 6 = 192 L4 任务占位
- 实施范围:**仅汇编**各域既有 DTL + 17 份 v0.2 SPEC,**不引入新设计**

### A.3 已知缺口

- 各域实际 L4 任务待 DDD Review 阶段补完
- 5 域 Lead 签字(本 v0.1 占位,等 Ulysses DDD Review 后补签)
- OLU token 估算(WF-1-55.55 task 5 联动)
- TBD-ECO-101~105(5 项 TBD)待 PH-3 实测填入
- TBD-DTL-037-01~03(per DTL-037 §6 既有)已在 SPEC-DTL-037 v0.2 §A.3 列已知缺口
- DTL-037 §6 反向文档与 DTL-001 §3.1 ISS-128 决策记录(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 §6.2)
- Saga reference 实现不破坏 saga 域 DTL-100/101/102 契约(per DEC-008 域独立基线)

### A.4 引用链

- 17 份 v0.2 SPEC(commit `756bcd3` ~ `97ef67c`,per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.1)
- RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-IMPL-001
- DEC-008 一人公司 12 角色治理基线
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束
