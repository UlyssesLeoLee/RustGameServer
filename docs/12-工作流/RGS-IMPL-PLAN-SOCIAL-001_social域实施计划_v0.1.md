# RGS-IMPL-PLAN-SOCIAL-001 social 域实施计划

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-IMPL-PLAN-SOCIAL-001 |
| 版本 | 0.2 |
| 父文档 | RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-PLAN-001 v1.0 + RGS-IMPL-001 工程约定 |
| 源详细设计 | RGS-DTL-039 social 域 v0.1(commit `833e7f7` WF-1-55-60)+ RGS-DTL-043 v0.1(commit `246f0c2` WF-1-55-64)+ RGS-SPEC-DTL-039/043 实现规格 v0.2 + RGS-DTL-013 v0.3(ChatAbuseGuard/Signal 落实现状) |
| 适用范围 | social 域 Atomic App 全生命周期实施(social-service crate + social_db 库 + 4 张表 + 好友/聊天/公会/礼物) |
| 目标基线 | Rust 1.98 + Actix Web 4.14.1 + PostgreSQL 18.6 + K3s |
| 责任人 | social 域 Lead(Ulysses per DEC-008 一人公司 12 角色兼任) |
| 触发 | WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草 | (本 v0.2 = v0.1 + DDD Review 反馈 + §3 RACI 矩阵 + §A 已知缺口 3 段)

---

## 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 初版:social 域实施计划;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);**不引入新设计**——汇编 DTL-039/043 + SPEC-DTL-039/043 v0.2 + 17 份 v0.2 SPEC 引用 |
| 0.2 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | v0.2 升版:DDD Review 反馈 + §3 RACI 矩阵(per RGS-LEAD-RACI-001 v1.1)+ §A 已知缺口 3 段(跨域协调 / 实时审计 / 1 人 12 角色 RACI 全覆盖);**不引入新设计**——本 v0.2 仅在 v0.1 头表 + 修订历史 + §3 + §A 加内容,正文本(域职责/实施阶段/验收)不动 | 头部 + 修订 + §3 + §A(新增) | 全部 |

---


---

## §3 RACI 矩阵 (NEW, v0.2 升版增量, per RGS-LEAD-RACI-001 v1.1 §3)

本域 social 的 6 治理角色 × 7 实施任务 RACI 映射（per RGS-ADR-0055 v0.1 §4）:

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

### §A.1 social 域 跨域协调依赖

本 social 域 IMPL-PLAN 涉及跨域 gRPC 调用（player → economy/match/social/admin + saga）需 5 域 binary 全部启 + 跨域联调通过才能完整验证。当前阻塞：
- PostgreSQL 18.6 未装（per Ulysses 16:58/16:59 硬约束，等装入）
- 5 域 binary 编译完成但启需 DATABASE_URL（per `RGS-GM-V0.3-DEPLOY-SOP-2026-08-26.md` v0.1）
- 跨域联调 IT 测试（per RGS-TEST-STRATEGY 4 阶段 phase 2，等 PG 装完）

### §A.2 实时审计跟踪

本 social 域 IMPL-PLAN 涉及 GM 操作 / RBAC 权限变更 / 跨域事件触发等操作需实时审计跟踪。依赖:
- ARC-018/021/042/051 4 治理角色（per RGS-ADR-0055 v0.1 §4）
- audit log 落库（per DTL-031 事件总线 + audit_log 表）
- 实时审计 dashboard（per rgs-web GM 后台 §3.5）

当前状态：审计跟踪设计在 DTL-031 §4，但 dashboard UI 未实装。

### §A.3 一人公司 12 角色 RACI 全覆盖

本 social 域 IMPL-PLAN v0.2 §3 RACI 矩阵仅含 6 治理角色（Arch/BE Lead/SRE Lead/DBA/PM/PO），缺:
- FE Lead（前后端边界，本域为后端无 FE）
- QA Lead（per RGS-TEST-STRATEGY 4 阶段）
- SEC（per RGS-REV-008 mTLS fail-closed）
- SRE（SRE Lead + SRE 角色区分，per DEC-008）
- 1 人 12 角色（per DEC-008 一人公司治理基线，5 域 Lead 全部 = Ulysses 兼任）

本缺口待 5 域 binary 启 + DDD Review 反馈 + RACI-001 v1.2 升版时统一补全。

## 1. 域职责

per RGS-DTL-039 v0.1 §1「social 域」+ RGS-DTL-043 v0.1 §1「social 子模块」+ RGS-SPEC-DTL-039/043 v0.2 §1:

social 域是 RGS 5 域中**社交与互动**核心域,职责覆盖:

- **4 张 social 表** CRUD(per RGS-DTL-039 v0.1 §3):friendship / chat_message / guild / gift_record
- **好友系统**——好友申请 / 通过 / 双向好友关系 / 黑名单
- **聊天系统**——私聊 / 群聊 / ChatAbuseGuard 反滥用(per DTL-013 v0.3 复核 §3.4)
- **公会系统**——per DTL-043 v0.1 §3 公会创建 / 加入 / 退出 / 解散
- **礼物系统**——per DTL-039 v0.1 §3 礼物赠送 + 跨域经济联动(归 economy 域,通过 gRPC)
- **Signal 实时推送**——per DTL-013 v0.3 §3.4 Signal 服务集成(WebSocket / SSE)

**域边界(per DTL-039 v0.1 §1.2 + DTL-043 v0.1 §1.2)**:
- ❌ **不**持有 player 身份/账户数据(归 player 域)
- ❌ **不**持有经济事务(归 economy 域,通过 gRPC 联动)
- ❌ **不**实现反滥用引擎核心(归 DTL-013 ChatAbuseGuard,本域仅集成)
- ❌ **不**实现跨域 Saga 协调(归 saga 域 DTL-100/101/102)
- ❌ **不**持有 GM/审计数据(归 admin 域)
- ✅ **仅**做好友/聊天/公会/礼物 4 类社交功能 + Signal 推送

**关键硬约束(per SPEC-DTL-039/043 v0.2 §3 + DTL-039/043 v0.1 + DTL-013 v0.3)**:

| 编号 | 内容 | 类型 |
|---|---|---|
| FR-SOC-001 | 好友关系双向(申请方 + 被申请方都需记录) | 既有 |
| NFR-SOC-002 | 私聊消息端到端延迟 < 100ms | 实测 |
| NFR-SOC-003 | 跨域事件 schema 经 SPEC-CROSS-003 v0.2 校验 | 硬约束 |
| NFR-SOC-004 | ChatAbuseGuard 集成必经 shared-platform::chat_guard(per DTL-013 v0.3) | 硬约束 |
| AC-SOC-001~008 | 8 项验收门槛 | 实测 |
| TBD-SOC-101~105 | 5 项 TBD | PH-3 实测填 |
| DTL-039 §6 4 项待补齐 | per DTL-039 §6 v0.1 起草时既有 | 已知缺口 |

---

## 2. 实施阶段(8 任务簇 × 4 L4 任务 = 32 L4)

| 任务簇 | 任务编号 | 任务名 | owner | 工期 | 依赖 |
|---|---|---|---|---|---|
| **API Spec** | 1.1 | social 域 gRPC Proto(friendship / chat / guild / gift) | social 域 Lead | 0.5 人·天 | BAS-001 v1.4 + SPEC-CROSS-002 v0.2 |
| API Spec | 1.2 | Signal 实时推送 Proto(WebSocket / SSE,per DTL-013 v0.3) | social 域 Lead | 0.5 人·天 | 1.1 + DTL-013 v0.3 §3.4 |
| API Spec | 1.3 | 错误码映射(per SPEC-CROSS-001 v0.2 + DTL-001 §3.4 ADR-0057) | social 域 Lead | 0.5 人·天 | 1.1 + 1.2 |
| API Spec | 1.4 | OpenAPI 3.1 + 跨域事件 schema(per SPEC-CROSS-003 v0.2) | social 域 Lead | 0.5 人·天 | 1.1-1.3 |
| **业务逻辑** | 2.1 | 4 张 social 表 Service 层(CRUD) | social 域 Lead | 1 人·天 | 1.1-1.4 + DTL-039 v0.1 §3 + DTL-043 v0.1 §3 |
| 业务逻辑 | 2.2 | 好友关系双向校验 + 公会成员管理(per FR-SOC-001) | social 域 Lead | 1 人·天 | 2.1 + DTL-039 v0.1 §4 + DTL-043 v0.1 §4 |
| 业务逻辑 | 2.3 | 聊天系统 + ChatAbuseGuard 集成(per NFR-SOC-004 + DTL-013 v0.3) | social 域 Lead | 1 人·天 | 2.1 + shared-platform::chat_guard |
| 业务逻辑 | 2.4 | Signal 实时推送 + 礼物系统(跨域经济联动 per DTL-016 v0.3) | social 域 Lead | 1 人·天 | 2.1 + 2.3 + economy gRPC + DTL-013 v0.3 |
| **DB migration** | 3.1 | social_db 库 + 5 独立 PG 18.6 元数据(per WBS §2A.1) | social 域 Lead | 0.5 人·天 | DB Pool 治理基线 |
| DB migration | 3.2 | 4 张 social 表 DDL(friendship / chat_message / guild / gift_record) | social 域 Lead | 0.5 人·天 | 3.1 + BAS-007 §3.2 + DTL-039 v0.1 §3 |
| DB migration | 3.3 | chat_message 按 created_at 月分区(per DTL-016 v0.3) | social 域 Lead | 0.5 人·天 | 3.2 + sqlx |
| DB migration | 3.4 | migration 工具链 | social 域 Lead | 0.5 人·天 | 3.1-3.3 + IMPL-001 §3 |
| **UT** | 4.1 | 4 张表 Service UT | social 域 Lead | 0.5 人·天 | 2.1 + rgs-testkit |
| UT | 4.2 | 好友关系双向 UT(per FR-SOC-001) | social 域 Lead | 0.5 人·天 | 2.2 + rgs-testkit |
| UT | 4.3 | ChatAbuseGuard 集成 UT(per NFR-SOC-004 + DTL-013 v0.3) | social 域 Lead | 0.5 人·天 | 2.3 + shared-platform::chat_guard |
| UT | 4.4 | Signal + 礼物系统 UT(per DTL-013 v0.3 + DTL-016 v0.3) | social 域 Lead | 0.5 人·天 | 2.4 + rgs-testkit |
| **IT** | 5.1 | social_db 集成测试(sqltest + 5 独立 PG 池) | social 域 Lead | 0.5 人·天 | 3.1-3.4 + rgs-testkit |
| IT | 5.2 | social → player gRPC 集成(身份验证) | social 域 Lead | 0.5 人·天 | 2.1 + player test container + ARC-005 |
| IT | 5.3 | social → economy gRPC 集成(礼物跨域) | social 域 Lead | 0.5 人·天 | 2.4 + economy test container |
| IT | 5.4 | social → match gRPC 集成(好友观战邀请) | social 域 Lead | 0.5 人·天 | 2.2 + match test container |
| **ST** | 6.1 | 端到端:加好友→聊天→送礼物→公会加入(ST harness) | social 域 Lead | 1 人·天 | 5.1-5.4 + K3s namespace |
| ST | 6.2 | NFR-SOC-002 私聊延迟 < 100ms 实测 | social 域 Lead | 0.5 人·天 | 6.1 + prometheus |
| ST | 6.3 | AC-SOC-001~008 全部 8 项达标 | social 域 Lead | 0.5 人·天 | 6.1 + check-docs-consistency.sh |
| ST | 6.4 | 故障注入:Signal 断连 / ChatAbuseGuard 误判 / 跨域超时 | social 域 Lead | 0.5 人·天 | 6.1 + chaos-mesh |
| **Helm chart** | 7.1 | social-service Helm chart 骨架(per ARC-051) | cluster-ops 域 Lead | 0.5 人·天 | 2.1-2.4 + Helm |
| Helm chart | 7.2 | social_db Helm chart 依赖 + StatefulSet + PVC | cluster-ops 域 Lead | 0.5 人·天 | 3.1-3.4 + Helm 依赖 |
| Helm chart | 7.3 | 5 独立 PG ConfigMap + K3s namespace 隔离 | cluster-ops 域 Lead | 0.5 人·天 | 7.1 + 7.2 |
| Helm chart | 7.4 | values.yaml 多环境 + sealed-secrets 治理 | cluster-ops 域 Lead | 0.5 人·天 | 7.1-7.3 + sealed-secrets |
| **observability** | 8.1 | OTel trace_id 传播(per SPEC-CROSS-006 v0.2) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + shared-platform::tracing |
| observability | 8.2 | 10 项 rgs_social_* 指标(QPS / 延迟 / 在线 / 礼物数) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + prometheus |
| observability | 8.3 | 5 项 rgs_social_* 日志(关系链 / 消息 / 跨域引用) | SRE 域 Lead | 0.5 人·天 | 2.1-2.4 + tracing-subscriber |
| observability | 8.4 | Grafana dashboard JSON(社交 / 聊天 / 礼物) | SRE 域 Lead | 0.5 人·天 | 8.1-8.3 + Grafana |

**L4 合计**:32 个 L4 任务 / ~16 人·天(per RGS-TS-001 v0.6 §6.2 token-OLU,合计 ~3.2M-4.8M tokens)

---

## 3. 任务清单(32 L4 详细)

### 3.1 API Spec 簇(1.1-1.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 1.1 | social 域 gRPC Proto(4 模块) | `crates/rgs-social-service/proto/social.proto` | 100K | BAS-001 v1.4 |
| 1.2 | Signal 推送 Proto | `crates/rgs-social-service/proto/signal.proto` | 80K | 1.1 + DTL-013 v0.3 §3.4 |
| 1.3 | 错误码映射 | `crates/rgs-social-service/src/error.rs` | 60K | 1.1 + 1.2 + SPEC-CROSS-001 |
| 1.4 | OpenAPI 3.1 + 跨域事件 schema | `crates/rgs-social-service/openapi/social.yaml` | 80K | 1.1-1.3 + SPEC-CROSS-003 |

### 3.2 业务逻辑簇(2.1-2.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 2.1 | 4 张 social 表 Service | `crates/rgs-social-service/src/service/{friendship,chat,guild,gift}.rs` | 250K | 1.1-1.4 + DTL-039 v0.1 §3 |
| 2.2 | 好友关系双向 + 公会 | `crates/rgs-social-service/src/service/{friendship,guild}_manager.rs` | 250K | 2.1 + DTL-039 v0.1 §4 + DTL-043 v0.1 §4 |
| 2.3 | 聊天系统 + ChatAbuseGuard | `crates/rgs-social-service/src/service/chat.rs` | 250K | 2.1 + shared-platform::chat_guard + DTL-013 v0.3 |
| 2.4 | Signal + 礼物系统 | `crates/rgs-social-service/src/service/{signal,gift}.rs` | 200K | 2.1 + 2.3 + economy gRPC |

### 3.3 DB migration 簇(3.1-3.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 3.1 | social_db 库 + 5 独立 PG | `crates/rgs-social-service/migrations/0001_social_db.sql` | 80K | DB Pool 治理基线 |
| 3.2 | 4 张 social 表 DDL | `crates/rgs-social-service/migrations/0002_social_tables.sql` | 100K | 3.1 + BAS-007 §3.2 + DTL-039 v0.1 §3 |
| 3.3 | chat_message 月分区 | `crates/rgs-social-service/migrations/0003_partition.sql` | 100K | 3.2 + sqlx + DTL-016 v0.3 |
| 3.4 | migration 工具链 | `crates/rgs-social-service/migrations/sqltest.toml` | 60K | 3.1-3.3 + IMPL-001 §3 |

### 3.4 UT 簇(4.1-4.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 4.1 | 4 张表 Service UT | `crates/rgs-social-service/tests/ut_social_service.rs` | 100K | 2.1 + rgs-testkit |
| 4.2 | 好友关系双向 UT | `crates/rgs-social-service/tests/ut_friendship.rs` | 100K | 2.2 + rgs-testkit |
| 4.3 | ChatAbuseGuard UT | `crates/rgs-social-service/tests/ut_chat_guard.rs` | 100K | 2.3 + shared-platform::chat_guard |
| 4.4 | Signal + 礼物系统 UT | `crates/rgs-social-service/tests/ut_signal_gift.rs` | 100K | 2.4 + rgs-testkit |

### 3.5 IT 簇(5.1-5.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 5.1 | social_db 集成 | `crates/rgs-social-service/tests/it_social_db.rs` | 100K | 3.1-3.4 + rgs-testkit |
| 5.2 | social → player 集成 | `crates/rgs-social-service/tests/it_player.rs` | 100K | 2.1 + player test container + ARC-005 |
| 5.3 | social → economy 集成 | `crates/rgs-social-service/tests/it_economy.rs` | 100K | 2.4 + economy test container |
| 5.4 | social → match 集成 | `crates/rgs-social-service/tests/it_match.rs` | 100K | 2.2 + match test container |

### 3.6 ST 簇(6.1-6.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 6.1 | 端到端 harness | `tests/st/social_e2e.rs` | 200K | 5.1-5.4 + K3s namespace |
| 6.2 | NFR-SOC-002 私聊延迟 | `tests/st/social_perf.rs` | 100K | 6.1 + prometheus |
| 6.3 | AC-SOC-001~008 | `tests/st/social_ac.rs` | 80K | 6.1 + check-docs-consistency.sh |
| 6.4 | 故障注入 | `tests/st/social_chaos.rs` | 80K | 6.1 + chaos-mesh |

### 3.7 Helm chart 簇(7.1-7.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 7.1 | social-service Helm chart | `deploy/helm/rgs-social-service/Chart.yaml` | 80K | 2.1-2.4 + Helm |
| 7.2 | social_db StatefulSet | `deploy/helm/rgs-social-db/Chart.yaml` | 80K | 3.1-3.4 + Helm 依赖 |
| 7.3 | 5 独立 PG ConfigMap | `deploy/helm/rgs-shared-pg/configmap.yaml` | 60K | 7.1 + 7.2 |
| 7.4 | values.yaml 多环境 | `deploy/helm/rgs-social-service/values.yaml` | 60K | 7.1-7.3 + sealed-secrets |

### 3.8 observability 簇(8.1-8.4)

| L4 | 任务 | 文件 | token-OLU | 前置 |
|---|---|---|---|---|
| 8.1 | OTel trace_id 传播 | `crates/rgs-social-service/src/observability/trace.rs` | 80K | 2.1-2.4 + shared-platform::tracing |
| 8.2 | 10 项 rgs_social_* 指标 | `crates/rgs-social-service/src/observability/metrics.rs` | 80K | 2.1-2.4 + prometheus |
| 8.3 | 5 项 rgs_social_* 日志 | `crates/rgs-social-service/src/observability/log.rs` | 60K | 2.1-2.4 + tracing-subscriber |
| 8.4 | Grafana dashboard | `deploy/grafana/social-dashboard.json` | 60K | 8.1-8.3 + Grafana |

---

## 4. RACI 责任矩阵

| 任务簇 \ 角色 | social 域 Lead | player 域 Lead | economy 域 Lead | match 域 Lead | admin 域 Lead | saga 域 Lead | foundation Lead | cluster-ops Lead | shared-platform Lead |
|---|---|---|---|---|---|---|---|---|---|
| API Spec | **R/A** | I | I | I | I | I | C | I | C(SPEC-CROSS-002) |
| 业务逻辑 | **R/A** | C(2.4 身份验证) | C(2.4 礼物) | C(2.4 观战) | I | C(2.4 Saga) | I | I | C(ChatAbuseGuard) |
| DB migration | **R/A** | I | I | I | I | I | I | C(7.2 StatefulSet) | C(命名规范) |
| UT | **R/A** | I | I | I | I | I | C(rgs-testkit) | I | C(ChatAbuseGuard) |
| IT | **R/A** | C(5.2) | C(5.3) | C(5.4) | I | C(5.3 Saga) | C(test container) | C(K3s) | C |
| ST | **R/A** | C | C | C | C | C | C | C(K3s) | C(OTel) |
| Helm chart | C(7.1/7.2 需求) | I | I | I | I | I | I | **R/A** | I |
| observability | C(8.1-8.4 需求) | I | I | I | I | I | I | C(dashboard 部署) | **R/A**(OTel/trace_id) |

---

## 5. Rollback 回滚路径

### 5.1 应用回滚

- social-service **不**是必选路径——若上线后出现回归:
  1. `k8s rollout undo deployment/rgs-social-service -n social`
  2. 触发 PFAU 编排(per ARC-051)切换回上一 PFAU Feature 版本
  3. 监控:AC-SOC-001~008 门禁自动告警

### 5.2 DB migration 回滚

- 4 张 social 表均为 idempotent + reversible(per BAS-007 §3.4)
- reverse migration 通过新建 `9999_rollback_*.sql`(per IMPL-001 §3.4)

### 5.3 plugin 回滚

- social-service **无 plugin 加载点**(per BAS-005 边界)

### 5.4 配置回滚

- Helm `values.yaml` 多环境回滚经 `helm rollback rgs-social-service <revision>`

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
- [ ] 17 份 v0.2 SPEC 引用一致(SPEC-DTL-039/043 + SPEC-CROSS-001~007 + SPEC-DTL-100~102)
- [ ] RGS-DTL-039 v0.1 + DTL-043 v0.1 + DTL-013 v0.3 引用一致
- [ ] RGS-DTL-016 v0.3 Saga 步骤 5.0 引用一致

### 6.3 域硬约束

- [ ] NFR-SOC-002(私聊延迟 < 100ms)实测通过
- [ ] NFR-SOC-003(跨域事件 schema 经 SPEC-CROSS-003 校验)代码评审 grep
- [ ] NFR-SOC-004(ChatAbuseGuard 集成必经 shared-platform::chat_guard)代码评审 grep + DTL-013 v0.3 §3.4 复核
- [ ] FR-SOC-001(好友关系双向)UT 覆盖

### 6.4 验收门槛

- [ ] AC-SOC-001~008 全部 8 项达标

---

## 7. Definition of Done

per RGS-SPEC-DTL-039/043 v0.2 §7 + RGS-SPEC-000 v0.3 §4:

- [ ] 32 个 L4 任务全部完成 + commit 落地
- [ ] 6 份 CR 全部通过
- [ ] CI 4 workflow 全过
- [ ] ST 6.1 端到端通过
- [ ] ST 6.2 NFR-SOC-002 私聊延迟 < 100ms 实测达标
- [ ] ST 6.3 AC-SOC-001~008 全部 8 项达标
- [ ] ST 6.4 故障注入恢复路径验证
- [ ] Helm chart 7.1-7.4 在 K3s 集群通过
- [ ] observability 8.1-8.4 在 staging 集群采集数据
- [ ] check-docs-consistency.sh 通过
- [ ] 17 份 v0.2 SPEC 引用一致
- [ ] RACI 责任矩阵 72 单元全部登记
- [ ] Rollback 6 路径实测演练通过
- [ ] 当前无实现文件时保持"待实现/待评审"状态(per RGS-SPEC-000 §5 第 7 条)
- [ ] **不**反向覆盖 saga 域 DTL-100~102 / DTL-013 v0.3 ChatAbuseGuard(只读依赖,per DEC-008)

---

## 8. Gate 证据与实测参数

### 8.1 CI 证据

- CI-FMT / CI-LINT / CI-TEST / CI-DENY 全部 exit 0

### 8.2 ST 证据

- **ST-6.1 E2E**:K3s namespace `social-st` 部署成功 + 4 类社交功能端到端
- **ST-6.2 NFR**:100 万社交关系 + 10000 在线 + 1000 QPS 私聊 → p99 < 100ms
- **ST-6.3 AC**:AC-SOC-001~008 全部 8 项达标
- **ST-6.4 Chaos**:5 类故障注入(Signal 断连 / ChatAbuseGuard 误判 / 跨域超时 / 好友不一致 / 礼物重复)全部恢复路径验证

### 8.3 Helm 证据

- 7.1-7.4 K3s 多环境部署通过

### 8.4 observability 证据

- OTel trace_id 完整链路(从 social → match / economy)
- Prometheus `rgs_social_*` 10 项指标 + Loki `rgs_social_*` 5 项字段
- Grafana `social-dashboard.json` 4 panel(社交 / 聊天 / 礼物 / Signal)

### 8.5 Rollback 证据

- 应用 / DB / Collector / dashboard 4 路径在 staging 演练通过

---

## 9. 关联文档

### 9.1 上行

- [RGS-DTL-039 social 域 v0.1](../02-运维安全与网络/RGS-DTL-039_详细设计书.md)
- [RGS-DTL-043 social 子模块 v0.1](../02-运维安全与网络/RGS-DTL-043_详细设计书.md)
- [RGS-SPEC-DTL-039/043 实现规格 v0.2](../13-实现规格/RGS-SPEC-DTL-039_实现规格书.md)
- [RGS-DTL-013 v0.3 ChatAbuseGuard/Signal](../02-运维安全与网络/RGS-DTL-013_详细设计书.md)
- [RGS-DTL-016 v0.3 Saga 步骤 5.0](../03-数据经济与交易/RGS-DTL-016_详细设计书.md)
- [RGS-SPEC-CROSS-003 跨域事件 Schema 字典 v0.2](../13-实现规格/RGS-SPEC-CROSS-003_跨域事件Schema字典_v0.1.md)

### 9.2 下行

- [RGS-IMPL-001 实施约定](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md)
- [RGS-IMPL-005 BUILD 构建规范 v0.1](../13-实现规格/RGS-IMPL-005_BUILD_构建规范_v0.1.md)
- [RGS-IMPL-006 CI 持续集成规范 v0.1](../13-实现规格/RGS-IMPL-006_CI_持续集成规范_v0.1.md)

### 9.3 同级(5 域 IMPL-PLAN 联动)

- [RGS-IMPL-PLAN-PLAYER-001 player 域实施计划](RGS-IMPL-PLAN-PLAYER-001_player域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ECONOMY-001 economy 域实施计划](RGS-IMPL-PLAN-ECONOMY-001_economy域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-MATCH-001 match 域实施计划](RGS-IMPL-PLAN-MATCH-001_match域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-ADMIN-001 admin 域实施计划](RGS-IMPL-PLAN-ADMIN-001_admin域实施计划_v0.1.md)
- [RGS-IMPL-PLAN-SAGA-001 saga 域实施计划](RGS-IMPL-PLAN-SAGA-001_saga域实施计划_v0.1.md)

### 9.4 模板参考

- [RGS-IMPL-PLAN-CDN-001 断点续传实施计划 v0.1](RGS-IMPL-PLAN-CDN-001_断点续传实施计划_v0.1.md)
- [RGS-IMPL-PLAN-LCM-001 服务器全生命周期实施计划 v0.1](RGS-IMPL-PLAN-LCM-001_服务器全生命周期实施计划_v0.1.md)

---

## A. v0.1 对齐说明

### A.1 触发

WBS v0.3 §2A.2.55.续2 WF-1-55.74 6 域 IMPL-PLAN 起草(social 域 1 份)

### A.2 范围

- social 域各 1 份 IMPL-PLAN v0.1,32 L4 任务占位齐全
- 6 域合计 32 L4 × 6 = 192 L4 任务占位
- 实施范围:**仅汇编**各域既有 DTL + 17 份 v0.2 SPEC,**不引入新设计**

### A.3 已知缺口

- 各域实际 L4 任务待 DDD Review 阶段补完
- 5 域 Lead 签字(本 v0.1 占位,等 Ulysses DDD Review 后补签)
- OLU token 估算(WF-1-55.55 task 5 联动)
- TBD-SOC-101~105(5 项 TBD)待 PH-3 实测填入
- DTL-039 §6 4 项待补齐项(per DTL-039 v0.1 起草时既有,已在 SPEC-DTL-039 v0.2 §A.3 列已知缺口)
- **不**反向覆盖 saga 域 DTL-100~102 / DTL-013 v0.3 ChatAbuseGuard(只读依赖,per DEC-008)

### A.4 引用链

- 17 份 v0.2 SPEC(commit `756bcd3` ~ `97ef67c`,per RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2 §2.1)
- RGS-WBS-001 v0.3 §2A.2.55.续2 + RGS-IMPL-001
- DEC-008 一人公司 12 角色治理基线
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束
