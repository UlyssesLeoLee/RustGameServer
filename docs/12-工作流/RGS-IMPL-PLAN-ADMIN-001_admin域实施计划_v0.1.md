# RGS-IMPL-PLAN-ADMIN-001 admin 域实施计划

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-ADMIN-001 |
| 版本 | 0.2 |
| 父文档 | RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-PLAN-001 v1.0 + RGS-IMPL-001 工程约定 |
| 源详细设计 | RGS-DTL-040 Admin 域 v0.1(commit `e043f81` WF-1-55-61)+ RGS-SPEC-DTL-040 实现规格 v0.2 + RGS-DTL-042 平台 LCM v0.2(commit `735ae4f` WF-1-55-63)+ RGS-DTL-021 v0.2(跨域 GM 审计) |
| 适用范围 | admin 域 Atomic App 全生命周期实施(admin-service crate + admin_db 库 + 6 张表 + GM 后台 + 审计 + 合规删除) |
| 目标基线 | Rust 1.98 + Actix Web 4.14.1 + PostgreSQL 18.6 + K3s |
| 责任人 | admin 域 Lead(Ulysses per DEC-008 一人公司 12 角色兼任,**独立**位 per 2026-08-21 一人公司架构师兼任拒绝证据) |
| 触发 | WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草 | (本 v0.2 = v0.1 + DDD Review 反馈 + §3 RACI 矩阵 + §A 已知缺口 3 段)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 初版:admin 域实施计划;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);**不引入新设计**——汇编 DTL-040 + SPEC-DTL-040 v0.2 + DTL-042 联动 + 17 份 v0.2 SPEC 引用 |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | v0.2 升版:DDD Review 反馈 + §3 RACI 矩阵(per RGS-LEAD-RACI-001 v1.1)+ §A 已知缺口 3 段(跨域协调 / 实时审计 / 1 人 12 角色 RACI 全覆盖);**不引入新设计**——本 v0.2 仅在 v0.1 头表 + 修订历史 + §3 + §A 加内容,正文本(域职责/实施阶段/验收)不动 | 头部 + 修订 + §3 + §A(新增) | 全部 |

---


---

## §3 RACI 矩阵 (NEW, v0.2 升版增量, per RGS-LEAD-RACI-001 v1.1 §3)

本域 admin 的 6 治理角色 × 7 实施任务 RACI 映射（per RGS-ADR-0055 v0.1 §4）:

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

### §A.1 admin 域 跨域协调依赖

本 admin 域 IMPL-PLAN 涉及跨域 gRPC 调用（player → economy/match/social/admin + saga）需 5 域 binary 全部启 + 跨域联调通过才能完整验证。当前阻塞：
- PostgreSQL 18.6 未装（per Ulysses 16:58/16:59 硬约束，等装入）
- 5 域 binary 编译完成但启需 DATABASE_URL（per `RGS-GM-V0.3-DEPLOY-SOP-2026-08-26.md` v0.1）
- 跨域联调 IT 测试（per RGS-TEST-STRATEGY 4 阶段 phase 2，等 PG 装完）

### §A.2 实时审计跟踪

本 admin 域 IMPL-PLAN 涉及 GM 操作 / RBAC 权限变更 / 跨域事件触发等操作需实时审计跟踪。依赖:
- ARC-018/021/042/051 4 治理角色（per RGS-ADR-0055 v0.1 §4）
- audit log 落库（per DTL-031 事件总线 + audit_log 表）
- 实时审计 dashboard（per rgs-web GM 后台 §3.5）

当前状态：审计跟踪设计在 DTL-031 §4，但 dashboard UI 未实装。

### §A.3 一人公司 12 角色 RACI 全覆盖

本 admin 域 IMPL-PLAN v0.2 §3 RACI 矩阵仅含 6 治理角色（Arch/BE Lead/SRE Lead/DBA/PM/PO），缺:
- FE Lead（前后端边界，本域为后端无 FE）
- QA Lead（per RGS-TEST-STRATEGY 4 阶段）
- SEC（per RGS-REV-008 mTLS fail-closed）
- SRE（SRE Lead + SRE 角色区分，per DEC-008）
- 1 人 12 角色（per DEC-008 一人公司治理基线，5 域 Lead 全部 = Ulysses 兼任）

本缺口待 5 域 binary 启 + DDD Review 反馈 + RACI-001 v1.2 升版时统一补全。

## 1. 域职责

per RGS-DTL-040 v0.1 §1「Admin 域」+ RGS-SPEC-DTL-040 v0.2 §1「实施范围」:

admin 域是 RGS 5 域中**GM 后台与审计/合规**核心域,职责覆盖:

- **6 张 admin 表** CRUD(per RGS-DTL-040 v0.1 §3):gm_user / gm_role / gm_operation_log / compliance_request / compliance_audit / admin_feature_config
- **GM 后台入口**——经 `AdminService` 转发(per FR-LCM-004 既有,RealmLifecycleService 不对外暴露独立接口)
- **GM 审计**——所有 GM 操作经 `admin_db.operation_audit` 双层审计(per NFR-SE-010 合规例外)
- **合规删除**——GDPR / 玩家主动注销 / 数据导出通路
- **客服工单**——per RGS-REQ-019 客服工单 + RGS-REQ-016 客服工单与支付对账
- **平台 LCM 转发**——admin 域作为 cluster-ops `RealmLifecycleService` Feature 子类注册入口(per DTL-042 v0.2 + RGS-IMPL-PLAN-LCM-001)
- **跨域 GM 操作**——对 player / economy / match / social 域的 GM 干预(补偿 / 扣款 / 封禁)

**域边界(per DTL-040 v0.1 §1.2 不做"具体业务逻辑"边界)**:
- ❌ **不**做具体业务逻辑(扣款逻辑归 economy / 封禁逻辑归 anti-cheat)
- ❌ **不**实现跨域 Saga 协调(归 saga 域 DTL-100/101/102)
- ❌ **不**直接连业务 service DB(经 gRPC client)
- ✅ **仅**做 GM 入口 + 审计 + 合规删除 + 平台 LCM 转发

**关键硬约束(per SPEC-DTL-040 v0.2 §3 + DTL-040 v0.1 + DTL-042 v0.2)**:

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-ADM-001 | GM 操作必须经 admin_db.operation_audit 双层审计 | 硬约束 |
| NFR-ADM-002 | 合规删除通路不可绕过审计(per NFR-SE-010 既有) | 硬约束 |
| NFR-ADM-003 | 平台 LCM 不对外暴露独立接口(per FR-LCM-004) | 硬约束 |
| NFR-ADM-004 | 跨域事件 schema 经 SPEC-CROSS-003 v0.2 校验 | 硬约束 |
| AC-ADM-001~010 | 10 项验收门槛 | 实测 |
| TBD-ADM-101~105 | 5 项 TBD | PH-3 实测填 |

> **重要**:per 2026-08-21 一人公司 5 域 Lead 兼任方案决议,**admin 域 Lead 不得由其他域 Lead 兼任**(per RGS-ADR-0055 §4 + DEC-008);即使一人公司 12 角色兼任,admin 域 Lead 责任矩阵 + RACI 简表中 A 角色**必须** Ulysses 显式签字。本 v0.1 阶段"责任人"字段标注"独立位"。

---

## 2. 实施阶段(8 任务簇 × 4 L4 任务 = 32 L4)

| 任务簇 | 任务编号 | 任务名 | owner | 工期 | 依赖 |
|---|---|---|---|---|---|
| **API Spec** | 1.1 | admin 域 gRPC Proto(gm_user / gm_role / operation_log / compliance) | admin 域 Lead | 0.5 人·天 | BAS-001 v1.4 + SPEC-CROSS-002 v0.2 |
| API Spec | 1.2 | 平台 LCM 转发 Proto(per DTL-042 v0.2 + RGS-IMPL-PLAN-LCM-001) | admin 域 Lead + cluster-ops 域 Lead | 0.5 人·天 | 1.1 + DTL-042 v0.2 |
| API Spec | 1.3 | 错误码映射(per SPEC-CROSS-001 v0.2 + DTL-001 §3.4 ADR-0057) | admin 域 Lead | 0.5 人·天 | 1.1 + 1.2 |
| API Spec | 1.4 | OpenAPI 3.1 + 跨域事件 schema(per SPEC-CROSS-003 v0.2) | admin 域 Lead | 0.5 人·天 | 1.1-1.3 |
| **业务逻辑** | 2.1 | 6 张 admin 表 Service 层(CRUD) | admin 域 Lead | 1 人·天 | 1.1-1.4 + DTL-040 v0.1 §3 |
| 业务逻辑 | 2.2 | GM 双层审计(per FR-ADM-001 + NFR-ADM-002) | admin 域 Lead | 1 人·天 | 2.1 + DTL-040 v0.1 §4 |
| 业务逻辑 | 2.3 | 合规删除通路 + 客服工单(per RGS-REQ-019) | admin 域 Lead | 1 人·天 | 2.1 + 2.2 + DTL-040 v0.1 §5 |
| 业务逻辑 | 2.4 | 平台 LCM 转发 + 跨域 GM 操作(per DTL-042 v0.2) | admin 域 Lead + cluster-ops 域 Lead | 1 人·天 | 2.1 + 1.2 + DTL-042 v0.2 |
| **DB migration** | 3.1 | admin_db 库 + 5 独立 PG 18.6 元数据(per WBS §2A.1) | admin 域 Lead | 0.5 人·天 | DB Pool 治理基线 |
| DB migration | 3.2 | 6 张 admin 表 DDL(gm_user / gm_role / gm_operation_log / compliance_request / compliance_audit / admin_feature_config) | admin 域 Lead | 1 人·天 | 3.1 + BAS-007 §3.2 + DTL-040 v0.1 §3 |
| DB migration | 3.3 | gm_operation_log 按 created_at 月分区 + 合规审计归档(per NFR-SE-010) | admin 域 Lead | 0.5 人·天 | 3.2 + sqlx + NFR-SE-010 |
| DB migration | 3.4 | migration 工具链 | admin 域 Lead | 0.5 人·天 | 3.1-3.3 + IMPL-001 §3 |
| **UT** | 4.1 | 6 张表 Service UT | admin 域 Lead | 1 人·天 | 2.1 + rgs-testkit |
| UT | 4.2 | GM 双层审计 UT(per FR-ADM-001 + NFR-ADM-002) | admin 域 Lead | 0.5 人·天 | 2.2 + rgs-testkit |
| UT | 4.3 | 合规删除 + 客服工单 UT(per RGS-REQ-019 + NFR-ADM-002) | admin 域 Lead | 0.5 人·天 | 2.3 + rgs-testkit |
| UT | 4.4 | 平台 LCM 转发 UT(per DTL-042 v0.2) | admin 域 Lead | 0.5 人·天 | 2.4 + cluster-ops test container |
| **IT** | 5.1 | admin_db 集成测试(sqltest + 5 独立 PG 池) | admin 域 Lead | 0.5 人·天 | 3.1-3.4 + rgs-testkit |
| IT | 5.2 | admin → player/economy/match/social 跨域 GM 集成 | admin 域 Lead | 0.5 人·天 | 2.4 + 5 域 test container |
| IT | 5.3 | admin → cluster-ops LCM 集成(per DTL-042 v0.2) | admin 域 Lead + cluster-ops 域 Lead | 0.5 人·天 | 2.4 + cluster-ops test container |
| IT | 5.4 | admin → saga 集成(GM 补偿走 Saga,per DTL-100) | admin 域 Lead | 0.5 人·天 | 2.4 + saga test container |
| **ST** | 6.1 | 端到端:GM 登录→操作玩家→审计留痕(ST harness) | admin 域 Lead | 1 人·天 | 5.1-5.4 + K3s namespace |
| ST | 6.2 | NFR-ADM-002 合规删除审计完整性实测 | admin 域 Lead | 0.5 人·天 | 6.1 + prometheus |
| ST | 6.3 | AC-ADM-001~010 全部 10 项达标 | admin 域 Lead | 0.5 人·天 | 6.1 + check-docs-consistency.sh |
| ST | 6.4 | 故障注入:审计失败 / 跨域断连 / LCM 转发失败 | admin 域 Lead | 0.5 人·天 | 6.1 + chaos-mesh |
| **Helm chart** | 7.1 | admin-service Helm chart 骨架(per ARC-051) | cluster-ops 域 Lead | 0.5 人·天 | 2.1-2.4 + Helm |
| Helm chart | 7.2 | admin_db Helm chart 依赖 + StatefulSet + PVC | cluster-ops 域 Lead | 0.5 人·天 | 3.1-3.4 + Helm 依赖 |
| Helm chart | 7.3 | 5 独立 PG ConfigMap + K3s namespace 隔离 | cluster-ops 域 Lead | 0.5 人·天 | 7.1 + 7.2 |
| Helm chart | 7.4 | values.yaml 多环境 + sealed-secrets 治理(GM 凭证高敏) | cluster-ops 域 Lead | 0.5 人·天 | 7.1-7.3 + sealed-secrets |
| **observability** | 8.1 | OTel trace_id 传播(per SPEC-CROSS-006 v0.2) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + shared-platform::tracing |
| observability | 8.2 | 12 项 rgs_admin_* 指标(GM 操作 / 审计 / 合规删除 / LCM 转发) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + prometheus |
| observability | 8.3 | 6 项 rgs_admin_* 日志(GM 操作 / 审计 / 跨域引用) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + tracing-subscriber |
| observability | 8.4 | Grafana dashboard JSON(GM / 审计 / 合规 / LCM) | SRE 域 Lead | 0.5 人·天 | 8.1-8.3 + Grafana |

**L4 合计**:32 个 L4 任务 / ~17 人·天(per RGS-TS-001 v0.6 §6.2 token-OLU,合计 ~3.4M-5.1M tokens)

---

## 3. 任务清单(32 L4 详细)

### 3.1 API Spec 簇(1.1-1.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 1.1 | admin 域 gRPC Proto(4 模块) | `crates/rgs-admin-service/proto/admin.proto` | 100K | BAS-001 v1.4 |
| 1.2 | 平台 LCM 转发 Proto | `crates/rgs-admin-service/proto/lcm_forward.proto` | 80K | 1.1 + DTL-042 v0.2 + RGS-IMPL-PLAN-LCM-001 |
| 1.3 | 错误码映射 | `crates/rgs-admin-service/src/error.rs` | 60K | 1.1 + 1.2 + SPEC-CROSS-001 |
| 1.4 | OpenAPI 3.1 + 跨域事件 schema | `crates/rgs-admin-service/openapi/admin.yaml` | 80K | 1.1-1.3 + SPEC-CROSS-003 |

### 3.2 业务逻辑簇(2.1-2.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 2.1 | 6 张 admin 表 Service | `crates/rgs-admin-service/src/service/{gm_user,gm_role,operation_log,compliance_request,compliance_audit,feature_config}.rs` | 250K | 1.1-1.4 + DTL-040 v0.1 §3 |
| 2.2 | GM 双层审计 | `crates/rgs-admin-service/src/service/audit.rs` | 250K | 2.1 + DTL-040 v0.1 §4 + NFR-SE-010 |
| 2.3 | 合规删除 + 客服工单 | `crates/rgs-admin-service/src/service/{compliance,ticket}.rs` | 200K | 2.1 + 2.2 + RGS-REQ-019 |
| 2.4 | 平台 LCM 转发 + 跨域 GM | `crates/rgs-admin-service/src/service/{lcm_forward,gm_cross_domain}.rs` | 200K | 2.1 + 1.2 + DTL-042 v0.2 |

### 3.3 DB migration 簇(3.1-3.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 3.1 | admin_db 库 + 5 独立 PG | `crates/rgs-admin-service/migrations/0001_admin_db.sql` | 80K | DB Pool 治理基线 |
| 3.2 | 6 张 admin 表 DDL | `crates/rgs-admin-service/migrations/0002_admin_tables.sql` | 200K | 3.1 + BAS-007 §3.2 + DTL-040 v0.1 §3 |
| 3.3 | gm_operation_log 月分区 + 合规审计归档 | `crates/rgs-admin-service/migrations/0003_partition_audit.sql` | 100K | 3.2 + sqlx + NFR-SE-010 |
| 3.4 | migration 工具链 | `crates/rgs-admin-service/migrations/sqltest.toml` | 60K | 3.1-3.3 + IMPL-001 §3 |

### 3.4 UT 簇(4.1-4.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 4.1 | 6 张表 Service UT | `crates/rgs-admin-service/tests/ut_admin_service.rs` | 200K | 2.1 + rgs-testkit |
| 4.2 | GM 双层审计 UT | `crates/rgs-admin-service/tests/ut_audit.rs` | 100K | 2.2 + rgs-testkit |
| 4.3 | 合规删除 + 客服工单 UT | `crates/rgs-admin-service/tests/ut_compliance_ticket.rs` | 100K | 2.3 + rgs-testkit |
| 4.4 | 平台 LCM 转发 UT | `crates/rgs-admin-service/tests/ut_lcm_forward.rs` | 100K | 2.4 + cluster-ops test container |

### 3.5 IT 簇(5.1-5.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 5.1 | admin_db 集成 | `crates/rgs-admin-service/tests/it_admin_db.rs` | 100K | 3.1-3.4 + rgs-testkit |
| 5.2 | admin → 5 域集成 | `crates/rgs-admin-service/tests/it_cross_domain.rs` | 100K | 2.4 + 5 域 test container |
| 5.3 | admin → cluster-ops LCM 集成 | `crates/rgs-admin-service/tests/it_lcm.rs` | 100K | 2.4 + cluster-ops test container |
| 5.4 | admin → saga 集成 | `crates/rgs-admin-service/tests/it_saga.rs` | 100K | 2.4 + saga test container |

### 3.6 ST 簇(6.1-6.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 6.1 | 端到端 harness | `tests/st/admin_e2e.rs` | 200K | 5.1-5.4 + K3s namespace |
| 6.2 | NFR-ADM-002 实测 | `tests/st/admin_audit.rs` | 100K | 6.1 + prometheus |
| 6.3 | AC-ADM-001~010 | `tests/st/admin_ac.rs` | 80K | 6.1 + check-docs-consistency.sh |
| 6.4 | 故障注入 | `tests/st/admin_chaos.rs` | 80K | 6.1 + chaos-mesh |

### 3.7 Helm chart 簇(7.1-7.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 7.1 | admin-service Helm chart | `deploy/helm/rgs-admin-service/Chart.yaml` | 80K | 2.1-2.4 + Helm |
| 7.2 | admin_db StatefulSet | `deploy/helm/rgs-admin-db/Chart.yaml` | 80K | 3.1-3.4 + Helm 依赖 |
| 7.3 | 5 独立 PG ConfigMap | `deploy/helm/rgs-shared-pg/configmap.yaml` | 60K | 7.1 + 7.2 |
| 7.4 | values.yaml 多环境 + GM 凭证 | `deploy/helm/rgs-admin-service/values.yaml` | 60K | 7.1-7.3 + sealed-secrets(GM 凭证) |

### 3.8 observability 簇(8.1-8.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 8.1 | OTel trace_id 传播 | `crates/rgs-admin-service/src/observability/trace.rs` | 80K | 2.1-2.4 + shared-platform::tracing |
| 8.2 | 12 项 rgs_admin_* 指标 | `crates/rgs-admin-service/src/observability/metrics.rs` | 80K | 2.1-2.4 + prometheus |
| 8.3 | 6 项 rgs_admin_* 日志 | `crates/rgs-admin-service/src/observability/log.rs` | 60K | 2.1-2.4 + tracing-subscriber |
| 8.4 | Grafana dashboard | `deploy/grafana/admin-dashboard.json` | 60K | 8.1-8.3 + Grafana |

---

## 4. RACI 责任矩阵

| 任务簇 \ 角色 | admin 域 Lead | player 域 Lead | economy 域 Lead | match 域 Lead | social 域 Lead | saga 域 Lead | foundation Lead | cluster-ops Lead | shared-platform Lead |
|---|---|---|---|---|---|---|---|---|---|
| API Spec | **R/A** | I | I | I | I | I | C | C(1.2 LCM 转发) | C(SPEC-CROSS-002) |
| 业务逻辑 | **R/A** | C(2.4 跨域 GM) | C(2.4 补偿) | C(2.4 封禁) | C(2.4 聊天封禁) | C(2.4 Saga 补偿) | I | **R/A**(2.4 LCM 转发) | C(audit) |
| DB migration | **R/A** | I | I | I | I | I | I | C(7.2 StatefulSet) | C(命名规范) |
| UT | **R/A** | I | I | I | I | I | C(rgs-testkit) | I | C |
| IT | **R/A** | C(5.2) | C(5.2) | C(5.2) | C(5.2) | C(5.4) | C(test container) | **R/A**(5.3 LCM) | C |
| ST | **R/A** | C | C | C | C | C | C | C(K3s) | C(OTel) |
| Helm chart | C(7.1/7.2 需求) | I | I | I | I | I | I | **R/A** | I |
| observability | C(8.1-8.4 需求) | I | I | I | I | I | I | C(dashboard 部署) | **R/A**(OTel/trace_id) |

> 注:admin 域 **不**与其他域 Lead 兼任(per 2026-08-21 一人公司架构师兼任拒绝证据);即使一人公司 12 角色兼任,admin 域 A 角色 = Ulysses 显式签字(per RGS-ADR-0055 §4 + DEC-008)。

---

## 5. Rollback 回滚路径

### 5.1 应用回滚

- admin-service **不**是必选路径——若上线后出现回归:
  1. `k8s rollout undo deployment/rgs-admin-service -n admin`
  2. 触发 PFAU 编排(per ARC-051)切换回上一 PFAU Feature 版本
  3. 监控:AC-ADM-001~010 门禁自动告警
- **GM 凭证安全**——admin-service 凭证属高敏数据(sealed-secrets + 双向加解密,per IMPL-002 §6)

### 5.2 DB migration 回滚

- 6 张 admin 表均为 idempotent + reversible(per BAS-007 §3.4)
- **gm_operation_log 合规审计表不可 DROP**——per NFR-SE-010 硬约束,即使 reverse migration 也仅可 ADD COLUMN / DROP COLUMN,**不可** DROP TABLE
- reverse migration 通过新建 `9999_rollback_*.sql`(per IMPL-001 §3.4)

### 5.3 plugin 回滚

- admin-service **无 plugin 加载点**(per BAS-005 边界)

### 5.4 配置回滚

- Helm `values.yaml` 多环境回滚经 `helm rollback rgs-admin-service <revision>`
- **GM 凭证**经 sealed-secrets 双向加解密(per IMPL-002 §6)

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
- [ ] 17 份 v0.2 SPEC 引用一致(SPEC-DTL-040 + SPEC-DTL-042 + SPEC-CROSS-001~007 + SPEC-DTL-100~102)
- [ ] RGS-DTL-040 v0.1 + DTL-042 v0.2 + DTL-021 v0.2 引用一致
- [ ] RGS-REQ-019 客服工单 / RGS-REQ-016 客服工单与支付对账 / RGS-IMPL-PLAN-LCM-001 引用一致

### 6.3 域硬约束

- [ ] FR-ADM-001(GM 双层审计)代码评审 grep + UT 覆盖
- [ ] NFR-ADM-002(合规删除审计完整性)实测 + 代码评审 grep
- [ ] NFR-ADM-003(平台 LCM 不对外暴露独立接口)代码评审 grep + 集成测试
- [ ] NFR-ADM-004(跨域事件 schema 经 SPEC-CROSS-003 校验)代码评审 grep

### 6.4 验收门槛

- [ ] AC-ADM-001~010 全部 10 项达标
- [ ] 平台 LCM 转发经 cluster-ops 域联审(per DTL-042 v0.2 + RGS-IMPL-PLAN-LCM-001)

---

## 7. Definition of Done

per RGS-SPEC-DTL-040 v0.2 §7 + RGS-SPEC-000 v0.3 §4:

- [ ] 32 个 L4 任务全部完成 + commit 落地
- [ ] 6 份 CR 全部通过
- [ ] CI 4 workflow 全过
- [ ] ST 6.1 端到端通过
- [ ] ST 6.2 NFR-ADM-002 合规删除审计完整性实测达标
- [ ] ST 6.3 AC-ADM-001~010 全部 10 项达标
- [ ] ST 6.4 故障注入恢复路径验证
- [ ] Helm chart 7.1-7.4 在 K3s 集群通过
- [ ] observability 8.1-8.4 在 staging 集群采集数据
- [ ] check-docs-consistency.sh 通过
- [ ] 17 份 v0.2 SPEC 引用一致
- [ ] RACI 责任矩阵 72 单元全部登记
- [ ] Rollback 6 路径实测演练通过
- [ ] 当前无实现文件时保持"待实现/待评审"状态(per RGS-SPEC-000 §5 第 7 条)
- [ ] **不**反向覆盖 cluster-ops 域 DTL-042 / saga 域 DTL-100~102(只读依赖,per DEC-008)
- [ ] **admin 域 Lead 不得与其他域 Lead 兼任**(per 2026-08-21 兼任拒绝证据,RACI A 角色必须 Ulysses 显式签字)

---

## 8. Gate 证据与实测参数

### 8.1 CI 证据

- CI-FMT / CI-LINT / CI-TEST / CI-DENY 全部 exit 0

### 8.2 ST 证据

- **ST-6.1 E2E**:K3s namespace `admin-st` 部署成功 + 6 张表 + LCM 转发 + 跨域 GM
- **ST-6.2 NFR**:合规删除审计完整性 100% 覆盖 + 100 万 GM 操作 → 0 漏审计
- **ST-6.3 AC**:AC-ADM-001~010 全部 10 项达标
- **ST-6.4 Chaos**:5 类故障注入(审计失败 / 跨域断连 / LCM 转发失败 / 合规删除失败 / GM 凭证泄漏)全部恢复路径验证

### 8.3 Helm 证据

- 7.1-7.4 K3s 多环境部署通过
- **GM 凭证**经 sealed-secrets 治理 + IMPL-002 §6 加解密验证

### 8.4 observability 证据

- OTel trace_id 完整链路
- Prometheus `rgs_admin_*` 12 项指标 + Loki `rgs_admin_*` 6 项字段
- Grafana `admin-dashboard.json` 4 panel(GM / 审计 / 合规 / LCM)

### 8.5 Rollback 证据

- 应用 / DB / Collector / dashboard 4 路径在 staging 演练通过
- **gm_operation_log 合规审计表在 reverse migration 中保持不 DROP** 验证

---

## 9. 关联文档

### 9.1 上行

- [RGS-DTL-040 Admin 域 v0.1](../02-运维安全与网络/RGS-DTL-040_Admin域_详细设计书.md)
- [RGS-SPEC-DTL-040 实现规格 v0.2](../13-实现规格/RGS-SPEC-DTL-040_实现规格书.md)
- [RGS-DTL-042 平台 LCM v0.2](../02-运维安全与网络/RGS-DTL-042_服务器全生命周期管理_详细设计书.md)
- [RGS-IMPL-PLAN-LCM-001 服务器全生命周期实施计划 v0.1](RGS-IMPL-PLAN-LCM-001_服务器全生命周期实施计划_v0.1.md)
- [RGS-DTL-021 v0.2 跨域 GM 审计](../02-运维安全与网络/RGS-DTL-021_详细设计书.md)
- [RGS-REQ-019 客服工单与支付对账](../03-数据经济与交易/RGS-REQ-019_客服工单与支付对账_需求定义书.md)
- [RGS-SPEC-CROSS-003 跨域事件 Schema 字典 v0.2](../13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md)
- [RGS-SEC-100 GM 审计与 Saga 安全设计 v0.1](../02-运维安全与网络/RGS-SEC-100_GM审计与Saga安全设计_v0.1.md)

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
- [RGS-IMPL-PLAN-SAGA-001 saga 域实施计划](RGS-IMPL-PLAN-SAGA-001_saga域实施计划_v0.1.md)

### 9.4 模板参考

- [RGS-IMPL-PLAN-CDN-001 断点续传实施计划 v0.1](RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md)
- [RGS-IMPL-PLAN-LCM-001 服务器全生命周期实施计划 v0.1](RGS-IMPL-PLAN-LCM-001_服务器全生命周期实施计划_v0.1.md)

---

## A. v0.1 对齐说明

### A.1 触发

WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草(admin 域 1 份)

### A.2 范围

- admin 域各 1 份 IMPL-PLAN v0.1,32 L4 任务占位齐全
- 6 域合计 32 L4 × 6 = 192 L4 任务占位
- 实施范围:**仅汇编**各域既有 DTL + 17 份 v0.2 SPEC,**不引入新设计**

### A.3 已知缺口

- 各域实际 L4 任务待 DDD Review 阶段补完
- 5 域 Lead 签字(本 v0.1 占位,等 Ulysses DDD Review 后补签)
- OLU token 估算(WF-1-55.55 task 5 联动)
- TBD-ADM-101~105(5 项 TBD)待 PH-3 实测填入
- DTL-040 §1.2 不做"具体业务逻辑"边界(per DTL-040 v0.1 既有,已在 SPEC-DTL-040 v0.2 §A.3 列已知缺口)
- **不**反向覆盖 cluster-ops 域 DTL-042 / saga 域 DTL-100~102(只读依赖,per DEC-008)

### A.4 引用链

- 17 份 v0.2 SPEC(commit `756bcd3` ~ `97ef67c`,per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.1)
- RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-IMPL-001
- DEC-008 一人公司 12 角色治理基线
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束
- **admin 域独立位**(per 2026-08-21 一人公司架构师兼任拒绝证据 + DEC-008)
