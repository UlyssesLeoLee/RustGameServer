# RGS-IMPL-PLAN-MATCH-001 match 域实施计划

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-MATCH-001 |
| 版本 | 0.2 |
| 父文档 | RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-PLAN-001 v1.0 + RGS-IMPL-001 工程约定 |
| 源详细设计 | RGS-DTL-026 match 域 v0.4(commit `22dd047` 8/25 升版)+ RGS-SPEC-DTL-026 实现规格 v0.2(commit `54b6500` WF-1-55-53)+ RGS-DTL-001 v0.6 §7.2.1 ARC-013 死锁防止/背压八边界 |
| 适用范围 | match 域 Atomic App 全生命周期实施(match-service crate + match_db 库 + 5 张表 + 匹配引擎 + 对局状态机) |
| 目标基线 | Rust 1.98 + Actix Web 4.14.1 + PostgreSQL 18.6 + K3s |
| 责任人 | match 域 Lead(Ulysses per DEC-008 一人公司 12 角色兼任) |
| 触发 | WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草 | (本 v0.2 = v0.1 + DDD Review 反馈 + §3 RACI 矩阵 + §A 已知缺口 3 段)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 初版:match 域实施计划;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);**不引入新设计**——汇编 DTL-026 v0.4 + SPEC-DTL-026 v0.2 + 17 份 v0.2 SPEC 引用 |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | v0.2 升版:DDD Review 反馈 + §3 RACI 矩阵(per RGS-LEAD-RACI-001 v1.1)+ §A 已知缺口 3 段(跨域协调 / 实时审计 / 1 人 12 角色 RACI 全覆盖);**不引入新设计**——本 v0.2 仅在 v0.1 头表 + 修订历史 + §3 + §A 加内容,正文本(域职责/实施阶段/验收)不动 | 头部 + 修订 + §3 + §A(新增) | 全部 |

---


---

## §3 RACI 矩阵 (NEW, v0.2 升版增量, per RGS-LEAD-RACI-001 v1.1 §3)

本域 match 的 6 治理角色 × 7 实施任务 RACI 映射（per RGS-ADR-0055 v0.1 §4）:

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

### §A.1 match 域 跨域协调依赖

本 match 域 IMPL-PLAN 涉及跨域 gRPC 调用（player → economy/match/social/admin + saga）需 5 域 binary 全部启 + 跨域联调通过才能完整验证。当前阻塞：
- PostgreSQL 18.6 未装（per Ulysses 16:58/16:59 硬约束，等装入）
- 5 域 binary 编译完成但启需 DATABASE_URL（per `RGS-GM-V0.3-DEPLOY-SOP-2026-08-26.md` v0.1）
- 跨域联调 IT 测试（per RGS-TEST-STRATEGY 4 阶段 phase 2，等 PG 装完）

### §A.2 实时审计跟踪

本 match 域 IMPL-PLAN 涉及 GM 操作 / RBAC 权限变更 / 跨域事件触发等操作需实时审计跟踪。依赖:
- ARC-018/021/042/051 4 治理角色（per RGS-ADR-0055 v0.1 §4）
- audit log 落库（per DTL-031 事件总线 + audit_log 表）
- 实时审计 dashboard（per rgs-web GM 后台 §3.5）

当前状态：审计跟踪设计在 DTL-031 §4，但 dashboard UI 未实装。

### §A.3 一人公司 12 角色 RACI 全覆盖

本 match 域 IMPL-PLAN v0.2 §3 RACI 矩阵仅含 6 治理角色（Arch/BE Lead/SRE Lead/DBA/PM/PO），缺:
- FE Lead（前后端边界，本域为后端无 FE）
- QA Lead（per RGS-TEST-STRATEGY 4 阶段）
- SEC（per RGS-REV-008 mTLS fail-closed）
- SRE（SRE Lead + SRE 角色区分，per DEC-008）
- 1 人 12 角色（per DEC-008 一人公司治理基线，5 域 Lead 全部 = Ulysses 兼任）

本缺口待 5 域 binary 启 + DDD Review 反馈 + RACI-001 v1.2 升版时统一补全。

## 1. 域职责

per RGS-DTL-026 v0.4 §1「match 域」+ RGS-SPEC-DTL-026 v0.2 §1「实施范围」:

match 域是 RGS 5 域中**匹配与对局**核心域,职责覆盖:

- **5 张 match 表** CRUD(per RGS-DTL-026 v0.4 §3):match_session / match_team / match_player / match_event / match_result
- **匹配引擎**——基于 player_id + 段位 + 模式的 matchmaking 调度
- **对局状态机**——8 状态(Idle / Matching / Loading / InProgress / Paused / Completed / Cancelled / Expired) + 转移表
- **跨域经济联动**——对局结束触发 economy 域比赛奖励(per DTL-037 §3 + DTL-016 v0.3 Saga 步骤 3.0/4.0)
- **跨域身份验证**——通过 player 域 gRPC 客户端校验 session_epoch(per ARC-005 强制)
- **对局录像/回放**——per DTL-026 v0.4 §6 录像存储 + 断点续传
- **反作弊联动**——anti-cheat 域(per DTL-025 §3)采集对局异常行为

**域边界(per DTL-026 v0.4 §1.2)**:
- ❌ **不**持有 player 身份/账户数据(归 player 域)
- ❌ **不**持有经济事务(归 economy 域)
- ❌ **不**持有好友/聊天/公会数据(归 social 域)
- ❌ **不**实现反作弊引擎(归 anti-cheat 域 DTL-025)
- ❌ **不**实现跨域 Saga 协调(归 saga 域 DTL-100/101/102)
- ✅ **仅**做匹配调度 + 对局状态机 + 跨域奖励触发

**关键硬约束(per SPEC-DTL-026 v0.2 §3 + DTL-026 v0.4)**:

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-MCH-001 | 对局状态机 8 状态转移合法 | 既有 |
| NFR-MCH-002 | 匹配调度 p99 < 200ms(per DTL-001 v0.6 §7.2.1 ARC-013 死锁防止/背压) | 实测 |
| NFR-MCH-003 | 对局事件写入经 shared-platform::outbox(per DTL-015 v0.2) | 硬约束 |
| NFR-MCH-004 | 跨域事件 schema 经 SPEC-CROSS-003 v0.2 校验 | 硬约束 |
| AC-MCH-001~008 | 8 项验收门槛 | 实测 |
| TBD-MCH-101~104 | 4 项 TBD | PH-3 实测填 |

---

## 2. 实施阶段(8 任务簇 × 4 L4 任务 = 32 L4)

| 任务簇 | 任务编号 | 任务名 | owner | 工期 | 依赖 |
|---|---|---|---|---|---|
| **API Spec** | 1.1 | match 域 gRPC Proto(match_session / match_team / match_player / match_event / match_result) | match 域 Lead | 0.5 人·天 | BAS-001 v1.4 + SPEC-CROSS-002 v0.2 |
| API Spec | 1.2 | 匹配引擎 Proto(match_request / match_response / matchmaking_strategy) | match 域 Lead | 0.5 人·天 | 1.1 + DTL-026 v0.4 §3 |
| API Spec | 1.3 | 错误码映射(per SPEC-CROSS-001 v0.2 + DTL-001 §3.4 ADR-0057) | match 域 Lead | 0.5 人·天 | 1.1 + 1.2 |
| API Spec | 1.4 | OpenAPI 3.1 + 跨域事件 schema(per SPEC-CROSS-003 v0.2) | match 域 Lead | 0.5 人·天 | 1.1-1.3 |
| **业务逻辑** | 2.1 | 5 张 match 表 Service 层(CRUD) | match 域 Lead | 1 人·天 | 1.1-1.4 + DTL-026 v0.4 §3 |
| 业务逻辑 | 2.2 | 对局状态机 8 状态 + 转移表(per FR-MCH-001) | match 域 Lead | 1 人·天 | 2.1 + DTL-026 v0.4 §4 |
| 业务逻辑 | 2.3 | 匹配引擎(matchmaking 调度 + 背压 per DTL-001 §7.2.1 ARC-013) | match 域 Lead | 1.5 人·天 | 2.1 + 2.2 + DTL-026 v0.4 §5 |
| 业务逻辑 | 2.4 | 跨域经济联动(对局结束触发 economy 比赛奖励,per DTL-016 v0.3 Saga 3.0/4.0) | match 域 Lead | 1 人·天 | 2.2 + economy gRPC client |
| **DB migration** | 3.1 | match_db 库 + 5 独立 PG 18.6 元数据(per WBS §2A.1) | match 域 Lead | 0.5 人·天 | DB Pool 治理基线 |
| DB migration | 3.2 | 5 张 match 表 DDL(match_session / match_team / match_player / match_event / match_result) | match 域 Lead | 1 人·天 | 3.1 + BAS-007 §3.2 + DTL-026 v0.4 §3 |
| DB migration | 3.3 | match_event 按 created_at 月分区(per DTL-007 §3 + DTL-016 v0.3) | match 域 Lead | 0.5 人·天 | 3.2 + sqlx |
| DB migration | 3.4 | migration 工具链 + Outbox 表(per NFR-MCH-003) | match 域 Lead | 0.5 人·天 | 3.1-3.3 + IMPL-001 §3 |
| **UT** | 4.1 | 5 张表 Service UT | match 域 Lead | 1 人·天 | 2.1 + rgs-testkit |
| UT | 4.2 | 对局状态机 8 状态转移 UT(per FR-MCH-001) | match 域 Lead | 1 人·天 | 2.2 + rgs-testkit |
| UT | 4.3 | 匹配引擎 UT(per NFR-MCH-002) | match 域 Lead | 0.5 人·天 | 2.3 + rgs-testkit |
| UT | 4.4 | 跨域经济联动 UT(per DTL-016 v0.3) | match 域 Lead | 0.5 人·天 | 2.4 + economy test container |
| **IT** | 5.1 | match_db 集成测试(sqltest + 5 独立 PG 池) | match 域 Lead | 0.5 人·天 | 3.1-3.4 + rgs-testkit |
| IT | 5.2 | match → player gRPC 集成(session_epoch 校验) | match 域 Lead | 0.5 人·天 | 2.4 + player test container + ARC-005 |
| IT | 5.3 | match → economy gRPC 集成(对局结束触发奖励) | match 域 Lead | 0.5 人·天 | 2.4 + economy test container |
| IT | 5.4 | match → anti-cheat / social gRPC 集成(异常上报 + 好友观战) | match 域 Lead | 0.5 人·天 | 2.4 + anti-cheat/social test container |
| **ST** | 6.1 | 端到端:玩家进入→匹配→对局→结束→奖励(ST harness) | match 域 Lead | 1 人·天 | 5.1-5.4 + K3s namespace |
| ST | 6.2 | NFR-MCH-002 匹配 p99 < 200ms 实测 | match 域 Lead | 0.5 人·天 | 6.1 + prometheus |
| ST | 6.3 | AC-MCH-001~008 全部 8 项达标 | match 域 Lead | 0.5 人·天 | 6.1 + check-docs-consistency.sh |
| ST | 6.4 | 故障注入:匹配超时 / 对局崩溃 / Outbox 重复发送 | match 域 Lead | 0.5 人·天 | 6.1 + chaos-mesh |
| **Helm chart** | 7.1 | match-service Helm chart 骨架(per ARC-051) | cluster-ops 域 Lead | 0.5 人·天 | 2.1-2.4 + Helm |
| Helm chart | 7.2 | match_db Helm chart 依赖 + StatefulSet + PVC | cluster-ops 域 Lead | 0.5 人·天 | 3.1-3.4 + Helm 依赖 |
| Helm chart | 7.3 | 5 独立 PG ConfigMap + K3s namespace 隔离 | cluster-ops 域 Lead | 0.5 人·天 | 7.1 + 7.2 |
| Helm chart | 7.4 | values.yaml 多环境 + sealed-secrets 治理 | cluster-ops 域 Lead | 0.5 人·天 | 7.1-7.3 + sealed-secrets |
| **observability** | 8.1 | OTel trace_id 传播(per SPEC-CROSS-006 v0.2) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + shared-platform::tracing |
| observability | 8.2 | 12 项 rgs_match_* 指标(QPS / 延迟 / 错误率 / 匹配队列长度 / 背压) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + prometheus |
| observability | 8.3 | 6 项 rgs_match_* 日志(match_id / 状态机 step / 跨域引用) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + tracing-subscriber |
| observability | 8.4 | Grafana dashboard JSON(匹配 / 对局 / 跨域) | SRE 域 Lead | 0.5 人·天 | 8.1-8.3 + Grafana |

**L4 合计**:32 个 L4 任务 / ~17 人·天(per RGS-TS-001 v0.6 §6.2 token-OLU,合计 ~3.4M-5.1M tokens)

---

## 3. 任务清单(32 L4 详细)

### 3.1 API Spec 簇(1.1-1.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 1.1 | match 域 gRPC Proto(5 表) | `crates/rgs-match-service/proto/match.proto` | 100K | BAS-001 v1.4 |
| 1.2 | 匹配引擎 Proto | `crates/rgs-match-service/proto/matchmaking.proto` | 80K | 1.1 + DTL-026 v0.4 §3 |
| 1.3 | 错误码映射 | `crates/rgs-match-service/src/error.rs` | 60K | 1.1 + 1.2 + SPEC-CROSS-001 |
| 1.4 | OpenAPI 3.1 + 跨域事件 schema | `crates/rgs-match-service/openapi/match.yaml` | 80K | 1.1-1.3 + SPEC-CROSS-003 |

### 3.2 业务逻辑簇(2.1-2.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 2.1 | 5 张 match 表 Service | `crates/rgs-match-service/src/service/{session,team,player,event,result}.rs` | 250K | 1.1-1.4 + DTL-026 v0.4 §3 |
| 2.2 | 对局状态机 8 状态 | `crates/rgs-match-service/src/state/match_state_machine.rs` | 250K | 2.1 + DTL-026 v0.4 §4 |
| 2.3 | 匹配引擎(背压) | `crates/rgs-match-service/src/matchmaking/engine.rs` | 350K | 2.1 + 2.2 + DTL-001 §7.2.1 ARC-013 |
| 2.4 | 跨域经济联动 | `crates/rgs-match-service/src/service/reward.rs` | 200K | 2.2 + economy gRPC + DTL-016 v0.3 |

### 3.3 DB migration 簇(3.1-3.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 3.1 | match_db 库 + 5 独立 PG | `crates/rgs-match-service/migrations/0001_match_db.sql` | 80K | DB Pool 治理基线 |
| 3.2 | 5 张 match 表 DDL | `crates/rgs-match-service/migrations/0002_match_tables.sql` | 200K | 3.1 + BAS-007 §3.2 + DTL-026 v0.4 §3 |
| 3.3 | match_event 月分区 | `crates/rgs-match-service/migrations/0003_partition.sql` | 100K | 3.2 + sqlx + DTL-016 v0.3 |
| 3.4 | migration + Outbox 表 | `crates/rgs-match-service/migrations/0004_outbox.sql` | 60K | 3.1-3.3 + IMPL-001 §3 |

### 3.4 UT 簇(4.1-4.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 4.1 | 5 张表 Service UT | `crates/rgs-match-service/tests/ut_match_service.rs` | 200K | 2.1 + rgs-testkit |
| 4.2 | 对局状态机 8 状态 UT | `crates/rgs-match-service/tests/ut_state_machine.rs` | 200K | 2.2 + rgs-testkit |
| 4.3 | 匹配引擎 UT | `crates/rgs-match-service/tests/ut_matchmaking.rs` | 100K | 2.3 + rgs-testkit |
| 4.4 | 跨域经济联动 UT | `crates/rgs-match-service/tests/ut_reward.rs` | 100K | 2.4 + economy test container |

### 3.5 IT 簇(5.1-5.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 5.1 | match_db 集成 | `crates/rgs-match-service/tests/it_match_db.rs` | 100K | 3.1-3.4 + rgs-testkit |
| 5.2 | match → player 集成 | `crates/rgs-match-service/tests/it_player.rs` | 100K | 2.4 + player test container + ARC-005 |
| 5.3 | match → economy 集成 | `crates/rgs-match-service/tests/it_economy.rs` | 100K | 2.4 + economy test container |
| 5.4 | match → anti-cheat/social 集成 | `crates/rgs-match-service/tests/it_anti_social.rs` | 100K | 2.4 + test container |

### 3.6 ST 簇(6.1-6.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 6.1 | 端到端 harness | `tests/st/match_e2e.rs` | 200K | 5.1-5.4 + K3s namespace |
| 6.2 | NFR-MCH-002 实测 | `tests/st/match_perf.rs` | 100K | 6.1 + prometheus |
| 6.3 | AC-MCH-001~008 | `tests/st/match_ac.rs` | 80K | 6.1 + check-docs-consistency.sh |
| 6.4 | 故障注入 | `tests/st/match_chaos.rs` | 80K | 6.1 + chaos-mesh |

### 3.7 Helm chart 簇(7.1-7.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 7.1 | match-service Helm chart | `deploy/helm/rgs-match-service/Chart.yaml` | 80K | 2.1-2.4 + Helm |
| 7.2 | match_db StatefulSet | `deploy/helm/rgs-match-db/Chart.yaml` | 80K | 3.1-3.4 + Helm 依赖 |
| 7.3 | 5 独立 PG ConfigMap | `deploy/helm/rgs-shared-pg/configmap.yaml` | 60K | 7.1 + 7.2 |
| 7.4 | values.yaml 多环境 | `deploy/helm/rgs-match-service/values.yaml` | 60K | 7.1-7.3 + sealed-secrets |

### 3.8 observability 簇(8.1-8.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 8.1 | OTel trace_id 传播 | `crates/rgs-match-service/src/observability/trace.rs` | 80K | 2.1-2.4 + shared-platform::tracing |
| 8.2 | 12 项 rgs_match_* 指标 | `crates/rgs-match-service/src/observability/metrics.rs` | 80K | 2.1-2.4 + prometheus |
| 8.3 | 6 项 rgs_match_* 日志 | `crates/rgs-match-service/src/observability/log.rs` | 60K | 2.1-2.4 + tracing-subscriber |
| 8.4 | Grafana dashboard | `deploy/grafana/match-dashboard.json` | 60K | 8.1-8.3 + Grafana |

---

## 4. RACI 责任矩阵

| 任务簇 \ 角色 | match 域 Lead | player 域 Lead | economy 域 Lead | social 域 Lead | admin 域 Lead | saga 域 Lead | foundation Lead | cluster-ops Lead | shared-platform Lead |
|---|---|---|---|---|---|---|---|---|---|
| API Spec | **R/A** | I | I | I | I | I | C | I | C(SPEC-CROSS-002) |
| 业务逻辑 | **R/A** | C(2.4 身份验证) | C(2.4 奖励触发) | I | I | C(2.4 Saga) | I | I | C(背压/Outbox) |
| DB migration | **R/A** | I | I | I | I | I | I | C(7.2 StatefulSet) | C(命名规范) |
| UT | **R/A** | I | I | I | I | I | C(rgs-testkit) | I | C |
| IT | **R/A** | C(5.2) | C(5.3) | C(5.4) | I | C(5.3 Saga) | C(test container) | C(K3s) | C |
| ST | **R/A** | C | C | C | C | C(Saga 演练) | C | C(K3s) | C(OTel) |
| Helm chart | C(7.1/7.2 需求) | I | I | I | I | I | I | **R/A** | I |
| observability | C(8.1-8.4 需求) | I | I | I | I | I | I | C(dashboard 部署) | **R/A**(OTel/trace_id) |

---

## 5. Rollback 回滚路径

### 5.1 应用回滚

- match-service **不**是必选路径——若上线后出现匹配回归:
  1. `k8s rollout undo deployment/rgs-match-service -n match`
  2. 触发 PFAU 编排(per ARC-051)切换回上一 PFAU Feature 版本
  3. 监控:AC-MCH-001~008 门禁自动告警
- **匹配引擎背压**——若 NFR-MCH-002 恶化(per DTL-001 §7.2.1 ARC-013 死锁防止),启用限流降级(回退到 v0 同步匹配)

### 5.2 DB migration 回滚

- 5 张 match 表 + Outbox 表均为 idempotent + reversible(per BAS-007 §3.4)
- reverse migration 通过新建 `9999_rollback_*.sql`(per IMPL-001 §3.4)

### 5.3 plugin 回滚

- match-service **无 plugin 加载点**(per BAS-005 边界)

### 5.4 配置回滚

- Helm `values.yaml` 多环境回滚经 `helm rollback rgs-match-service <revision>`

### 5.5 Collector 回滚

- OTel Collector 配置由 cluster-ops 域统一管理
- 回滚经 cluster-ops 域 `kubectl apply -f otel-collector-prev.yaml -n observability`

### 5.6 dashboard 回滚

- Grafana dashboard JSON 通过 Grafana provisioning 双向同步
- 回滚经 `kubectl apply -f grafana-match-dashboard-prev.json -n observability`

---

## 6. 验收项

### 6.1 CI 4 workflow(per RGS-IMPL-006)

- [ ] `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test --all` / `cargo deny check` 全部 exit 0

### 6.2 文档一致性

- [ ] `check-docs-consistency.sh` 通过
- [ ] 17 份 v0.2 SPEC 引用一致(SPEC-DTL-026 + SPEC-CROSS-001~007 + SPEC-DTL-100~102)
- [ ] RGS-DTL-026 v0.4 + DTL-001 v0.6 + DTL-007 v0.2 引用一致
- [ ] RGS-DTL-016 v0.3 Saga 步骤 3.0/4.0 引用一致

### 6.3 域硬约束

- [ ] NFR-MCH-002(匹配 p99 < 200ms)实测通过
- [ ] NFR-MCH-003(Outbox 必经)代码评审 grep
- [ ] NFR-MCH-004(跨域事件 schema 经 SPEC-CROSS-003 校验)代码评审 grep
- [ ] FR-MCH-001(对局状态机 8 状态转移合法)代码评审 grep + UT 覆盖

### 6.4 验收门槛

- [ ] AC-MCH-001~008 全部 8 项达标

---

## 7. Definition of Done

per RGS-SPEC-DTL-026 v0.2 §7 + RGS-SPEC-000 v0.3 §4:

- [ ] 32 个 L4 任务全部完成 + commit 落地
- [ ] 6 份 CR 全部通过
- [ ] CI 4 workflow 全过
- [ ] ST 6.1 端到端通过
- [ ] ST 6.2 NFR-MCH-002 匹配 p99 < 200ms 实测达标
- [ ] ST 6.3 AC-MCH-001~008 全部 8 项达标
- [ ] ST 6.4 故障注入恢复路径验证
- [ ] Helm chart 7.1-7.4 在 K3s 集群通过
- [ ] observability 8.1-8.4 在 staging 集群采集数据
- [ ] check-docs-consistency.sh 通过
- [ ] 17 份 v0.2 SPEC 引用一致
- [ ] RACI 责任矩阵 72 单元全部登记
- [ ] Rollback 6 路径实测演练通过
- [ ] 当前无实现文件时保持"待实现/待评审"状态(per RGS-SPEC-000 §5 第 7 条)
- [ ] **不**反向覆盖 anti-cheat 域 DTL-025 / saga 域 DTL-100~102(只读依赖)

---

## 8. Gate 证据与实测参数

### 8.1 CI 证据

- CI-FMT / CI-LINT / CI-TEST / CI-DENY 全部 exit 0

### 8.2 ST 证据

- **ST-6.1 E2E**:K3s namespace `match-st` 部署成功 + 端到端 5 步流程
- **ST-6.2 NFR**:100 万级 player + 1000 QPS 匹配 → p99 < 200ms
- **ST-6.3 AC**:AC-MCH-001~008 全部 8 项达标
- **ST-6.4 Chaos**:5 类故障注入(匹配超时 / 对局崩溃 / Outbox 重复 / 状态机非法转移 / 跨域断连)全部恢复路径验证

### 8.3 Helm 证据

- 7.1-7.4 K3s 多环境部署通过

### 8.4 observability 证据

- OTel trace_id 完整链路(从 player → match → economy)
- Prometheus `rgs_match_*` 12 项指标 + Loki `rgs_match_*` 6 项字段
- Grafana `match-dashboard.json` 4 panel(匹配 / 对局 / 跨域 / 背压)

### 8.5 Rollback 证据

- 应用 / DB / Collector / dashboard 4 路径在 staging 演练通过
- 匹配引擎限流降级路径在 staging 验证

---

## 9. 关联文档

### 9.1 上行

- [RGS-DTL-026 match 域 v0.4](../02-运维安全与网络/RGS-DTL-026_详细设计书.md)
- [RGS-SPEC-DTL-026 实现规格 v0.2](../13-实现规格/RGS-SPEC-DTL-026_实现规格书.md)
- [RGS-DTL-001 v0.6 §7.2.1 ARC-013](../02-运维安全与网络/RGS-DTL-001_详细设计书.md)
- [RGS-DTL-016 v0.3 Saga 步骤 3.0/4.0](../03-数据经济与交易/RGS-DTL-016_详细设计书.md)
- [RGS-SPEC-CROSS-003 跨域事件 Schema 字典 v0.2](../13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md)

### 9.2 下行

- [RGS-IMPL-001 实施约定](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
- [RGS-IMPL-005 BUILD 构建规范 v0.1](../13-实现规格/RGS-IMPL-005_BUILD_构建规范_v0.1.md)
- [RGS-IMPL-006 CI 持续集成规范 v0.1](../13-实现规格/RGS-IMPL-006_CI_持续集成规范_v0.1.md)

### 9.3 同级(5 域 IMPL-PLAN 联动)

- [RGS-IMPL-PLAN-PLAYER-001 player 域实施计划](RGS-IMPL-PLAN-PLAYER-001_player域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ECONOMY-001 economy 域实施计划](RGS-IMPL-PLAN-ECONOMY-001_economy域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-SOCIAL-001 social 域实施计划](RGS-IMPL-PLAN-SOCIAL-001_social域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ADMIN-001 admin 域实施计划](RGS-IMPL-PLAN-ADMIN-001_admin域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-SAGA-001 saga 域实施计划](RGS-IMPL-PLAN-SAGA-001_saga域实施计划_v0.1.md)

### 9.4 模板参考

- [RGS-IMPL-PLAN-CDN-001 断点续传实施计划 v0.1](RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md)
- [RGS-IMPL-PLAN-LCM-001 服务器全生命周期实施计划 v0.1](RGS-IMPL-PLAN-LCM-001_服务器全生命周期实施计划_v0.1.md)

---

## A. v0.1 对齐说明

### A.1 触发

WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草(match 域 1 份)

### A.2 范围

- match 域各 1 份 IMPL-PLAN v0.1,32 L4 任务占位齐全
- 6 域合计 32 L4 × 6 = 192 L4 任务占位
- 实施范围:**仅汇编**各域既有 DTL + 17 份 v0.2 SPEC,**不引入新设计**

### A.3 已知缺口

- 各域实际 L4 任务待 DDD Review 阶段补完
- 5 域 Lead 签字(本 v0.1 占位,等 Ulysses DDD Review 后补签)
- OLU token 估算(WF-1-55.55 task 5 联动)
- TBD-MCH-101~104(4 项 TBD)待 PH-3 实测填入
- DTL-026 v0.4 是 8/25 升版,SPEC v0.2 §A.1 准确反映 0.4 最新版本(per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.3)
- **不**反向覆盖 anti-cheat 域 DTL-025 / saga 域 DTL-100~102(只读依赖,per DEC-008 域独立基线)

### A.4 引用链

- 17 份 v0.2 SPEC(commit `756bcd3` ~ `97ef67c`,per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.1)
- RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-IMPL-001
- DEC-008 一人公司 12 角色治理基线
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束
