# 基本设计书（基本設計書 / Basic Design Document）

**集群运营中心（Cluster Operations Center, COC）与每功能原子升级（Per-Feature Atomic Upgrade）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-031（addendum） |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-031 需求定义书（ARC-051） |
| 配套设计 | RGS-BAS-002（功能挂载）、RGS-BAS-003（GM后台）、RGS-BAS-005（插件热插拔）、RGS-BAS-021（无限画布）、RGS-BAS-024（集群部署） |
| 制定日 | 2026-08-19 |
| 最终更新日 | 2026-08-19 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | — | 初版制定。将RGS-REQ-031 ARC-051展开为ClusterOpsService的组件图、`admin_db`新增Schema、Feature元数据与PFAU状态机、CEM探针订阅器设计、API契约字段级定义、RBAC角色矩阵扩展、整合既有ARC-018／021／042／019／039的强制联动点 | 全部 |
| 0.2 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | — | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，all_36 + compile_plus_runtime）：§3.1-3.6（feature_registry/feature_version_history/pfa_run_state/event_schema_registry+event_producer_registry/event_dlq_view/coc_audit_view 6 张表 lifecycle）+ §4.1（Feature 元数据生命周期状态机）+ §5.5（CEM 探针全链路综合）+ §6.4（ClusterOpsService API 契约全链路综合）+ §10.5（性能/可用性/隔离/审计综合）共 10 个"本功能日志设计"小节全部新增；每节均含 5 列详尽版（字段名/触发条件/频率估算/采样策略/脱敏与成本），字段名前缀 `coc.*`（区别于 BAS-002 `mnt.*` / BAS-003 `gm.*` / BAS-004 `log.*` / BAS-005 `plugin.*` / BAS-009 `gov.*` / BAS-019 `push.*`），命名严格 snake_case 与 BAS-004 v0.3 §4.3.1/§4.3.2 保持拼写一致（FR-LOG-013）；显式区分 `info!`/`warn!`/`error!`（release 必出，编译期常驻，per BAS-004 v0.3 §6.2 强制全采样白名单）与 `debug!`/`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only，release build 完全剔除零运行时开销）两类事件；覆盖 ARC-051 集群运营中心 + 每功能原子升级全链路——Feature 元数据 CRUD / 不可变审计 / PFAU 实例状态机 / 灰度批次推进 / CEM 探针订阅 / 事件总线可达性 / DLQ 视图 / API 调用 receive/complete/失败 / 性能 SLO 降级 / 隔离边界违反 | 全部 |
| 0.3 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实「処理フロー」段四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板, RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1): 新增 §6.5 処理フロー（处理流程 / Processing Flow）段, 含主流程图 (mermaid sequenceDiagram, 7 actor: GM Operator / AdminService / ClusterOpsService / FeatureRegistry / PFAURunner / CEMProbeAggregator / admin_db) + 異常分支表 (8 行: RBAC 拒绝 / 节点 canary 失败 / 状态机非法迁移 / 节点心跳超时 / 集群脑裂 / Saga 步骤失败 / DLQ 累积 / Feature 删除合规缺失) + 决策点矩阵 (5 行: PFAU 灰度批次推进 / 回滚 vs 继续 / DLQ 重放 vs 丢弃 / 弃用 vs 删除 / CEM 探针 vs 直连) + 验证点清单 (7 行: RBAC / 状态机迁移 / 批次确认节点数 / mTLS 握手 / 事务提交 / Saga 补偿 / trace_id 串联), 覆盖 Feature 元数据生命周期 (per §4.1) + PFAU 编排 (per §4.2/§4.3) 两个主路径; trace_id 贯穿全链路 (per BAS-004 v0.3 §4.4); 事务边界与 Saga 跨域编排标注 (per BAS-100 v0.1); 与既有 §4.1 生命周期 / §4.2 PFAU 状态机 / §4.3 灰度批次推进规则 互为详细化引用; 与 BAS-019 §1.1 范式一致 (commit `d52eaad`) | §6.5 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构） | | | ClusterOpsService的限界上下文归属（确认归AD扩展，不新建限界上下文）；与既有ARC-018／021／042的强制联动点是否完备 |
| 评审（运维／SRE） | | | PFAU状态机的暂停/恢复/回滚边界是否覆盖SRE日常SOP |
| 评审（DBA） | | | 新增`admin_db`表（`feature_registry`/`feature_version_history`/`event_schema_registry`/`event_producer_registry`/`pfa_run_state`/`coc_audit_view`）的索引/分区/外键策略 |
| 评审（安全） | | | ClusterOpsService的凭证范围（仅AD控制面凭证，不持有K8s/DB直连凭证）；CEM探针订阅器的安全沙箱 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [ClusterOpsService 组件图与限界上下文归属](#2-clusteropsservice-组件图与限界上下文归属)
3. [`admin_db` 新增 Schema](#3-admindb-新增-schema)
4. [Feature 元数据与 PFAU 状态机](#4-feature-元数据与-pfau-状态机)
5. [CEM 探针订阅器设计](#5-cem-探针订阅器设计)
6. [API 契约字段级定义](#6-api-契约字段级定义)
   6.5 [処理フロー（处理流程 / Processing Flow）](#65-処理フロー处理流程--processing-flow)
7. [COC UI 页面构成与复用 VIZ 渲染能力](#7-coc-ui-页面构成与复用-viz-渲染能力)
8. [RBAC 角色矩阵扩展](#8-rbac-角色矩阵扩展)
9. [与既有 ARC-018／021／042／019／039 的强制联动点](#9-与既有-arc-018021042019039-的强制联动点)
10. [非功能设计落地](#10-非功能设计落地)
11. [风险与未决事项](#11-风险与未决事项)

---

# 1. 前言

本文档展开RGS-REQ-031 ARC-051，给出ClusterOpsService的组件图、`admin_db`新增Schema、Feature元数据与PFAU状态机、CEM探针订阅器、API契约字段级定义、COC UI页面构成、RBAC角色矩阵扩展，以及与既有ARC-018／021／042／019／039的强制联动点。核心原则（继承ARC-051）：

- **不新建独立限界上下文**——ClusterOpsService归AD限界上下文扩展（与既有AdminService同上下文）
- **不绕过AdminService统一入口**——COC UI的全部写操作经AdminService转发
- **不强制各App改造Publisher SDK**——CEM通过事件总线探针聚合
- **不绕过RGS-ADR-0020**——PFAU的"补丁型Feature"仅走特性开关或沙箱脚本
- **不绕过ARC-008独立DB原则**——所有元数据存于既有`admin_db`（同ARC-019既有`AdminService`/`operation_audit`表同库）

# 2. ClusterOpsService 组件图与限界上下文归属

## 2.1 限界上下文归属

按RGS-REQ-031 §1.4判定原则，ClusterOpsService**不**新建独立限界上下文，**归AD限界上下文扩展**（与既有`AdminService`同上下文）。理由：①AD限界上下文已有`admin_db`（ARC-008独立的DB）、已有RBAC角色矩阵（ARC-019）、已有审计通路（`operation_audit`表）、已有统一入口（AdminService）——COC UI的新增需求均落在AD既有能力范围内；②新建独立限界上下文会增加挂载成本（ARC-018五要素），且COC UI与既有GM后台共用凭证体系，物理上同处一个Deployment即可。

## 2.2 组件图

```
┌──────────────────────────────────────────────────────────────────────┐
│                       GM后台 UI (既有, 扩展新增 COC UI 页面)            │
│  ┌──────────────┬──────────────┬──────────────┬──────────────┐        │
│  │ 账号管控      │ 服务器管控    │ 告警         │ COC UI (新增)  │        │
│  │ (既有)        │ (既有)        │ (既有)        │              │        │
│  └──────────────┴──────────────┴──────────────┴──────────────┘        │
│         │              │              │              │                │
│         └──────────────┴──────────────┴──────────┘ │                │
│                                                    │                │
│                            AdminService 统一入口 (RBAC+审计+限流)  ◄──┘
│                                                    │ gRPC
└────────────────────────────────────────────────────┼─────────────────┘
                                                     │
                                                     ▼
┌──────────────────────────────────────────────────────────────────────┐
│              AD 限界上下文 (既有 admin_db + 既有 Deployment)            │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  AdminService (既有)                                              │ │
│  │    ├─ KickSession / MuteChat / 数值表热更新 (既有)               │ │
│  │    └─ 转发到 ClusterOpsService (新增)                              │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  ClusterOpsService (新增)                                          │ │
│  │    ├─ FeatureRegistry     (FR-PFAU-002 元数据管理)                │ │
│  │    ├─ PFAURunner          (FR-PFAU-010/011 状态机)               │ │
│  │    ├─ CEMProbeAggregator  (FR-CEM-020 订阅关系图)                │ │
│  │    ├─ DLQOperator         (FR-CEM-041 DLQ管理)                   │ │
│  │    └─ ReplayOperator      (FR-CEM-051 重放管理)                   │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│         │                                                          │
│         │ 写 admin_db                                              │
│         ▼                                                          │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  admin_db (既有, 本设计新增若干表)                              │  │
│  │    ├─ operation_audit           (既有, FR-COC-040 复用)        │  │
│  │    ├─ feature_registry          (新增, FR-PFAU-002)            │  │
│  │    ├─ feature_version_history   (新增, FR-PFAU-003)            │  │
│  │    ├─ pfa_run_state             (新增, FR-PFAU-010 状态机)     │  │
│  │    ├─ event_schema_registry     (新增, FR-CEM-001)             │  │
│  │    ├─ event_producer_registry   (新增, FR-CEM-001)             │  │
│  │    ├─ event_dlq_view            (新增视图, FR-CEM-040)         │  │
│  │    └─ coc_audit_view            (新增视图, FR-COC-040)         │  │
│  └─────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
         │
         │ 读（探针订阅器 + 探针指标）
         ▼
┌──────────────────────────────────────────────────────────────────────┐
│                  事件总线 + 可观测性基础设施 (既有)                     │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐    │
│  │ 事件总线探针       │  │ OTel Collector   │  │ Prometheus       │    │
│  │ (CEM 新增探针)    │  │ (既有)            │  │ (既有)            │    │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘    │
└──────────────────────────────────────────────────────────────────────┘
```

## 2.3 与既有ARC-018挂载脚手架的复用

ClusterOpsService作为AD限界上下文的扩展，其部署形态沿用RGS-BAS-002 §4的脚手架——独立的Deployment、独立的事件发布订阅、独立的可观测性埋点、独立的Helm chart基座。**不**为COC新增独立挂载流程，而是按ARC-018将COC归为AD限界上下文的扩展功能走"既有上下文内的扩展"简化路径（FR-MNT-010），即：①DB迁移脚本走既有CI迁移流水线；②新增gRPC方法遵循ARC-015后向兼容方针；③追加/更新对应服务的可观测性埋点。

# 3. `admin_db` 新增 Schema

所有新表**必须**遵循RGS-REQ-011（数据库标准化）的既有规范（命名、约束、索引、分区、迁移）。**不**新建独立DB。

## 3.1 `feature_registry`（Feature 元数据，FR-PFAU-002）

```sql
CREATE TABLE feature_registry (
    feature_id          TEXT        NOT NULL PRIMARY KEY,  -- 稳定不变, 如 'rgs.evt.spring_festival_2026'
    feature_type        TEXT        NOT NULL,             -- bounded_context | plugin | patch | config
    display_name        TEXT        NOT NULL,
    description         TEXT        NOT NULL DEFAULT '',
    current_version     TEXT        NOT NULL,             -- 当前 active 版本
    target_version      TEXT        NULL,                 -- PFAU 升级中的目标版本
    depends_on          TEXT[]      NOT NULL DEFAULT '{}',-- Feature 级依赖 (Feature ID 列表)
    status              TEXT        NOT NULL,             -- pending | in_progress | active | rolling_back | rolled_back | deprecated
    owner_team          TEXT        NOT NULL,             -- 业务方负责团队
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_feature_type CHECK (feature_type IN ('bounded_context','plugin','patch','config')),
    CONSTRAINT chk_status CHECK (status IN ('pending','in_progress','active','rolling_back','rolled_back','deprecated'))
);

CREATE INDEX idx_feature_registry_type_status ON feature_registry (feature_type, status);
CREATE INDEX idx_feature_registry_owner ON feature_registry (owner_team);
```

### 3.1 本功能日志设计

`feature_registry` 是 COC 元数据根基表，CRUD 事件是 §4.1 Feature 元数据生命周期的**持久化回声**。所有写操作 release 必出（per BAS-004 v0.3 §6.2 合规审计白名单），读操作 debug-only（按 trace_id 排查时按需开启）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.feature_registry.row_registered` | `AdminService.RegisterFeature` 成功写入新行 | 极低（每月 0-3 个新 Feature） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `feature_id` / `feature_type` / `display_name` / `owner_team` / `operator_id`；约 250B/条 |
| `coc.feature_registry.row_updated` | `current_version` / `target_version` / `status` / `depends_on` 等字段修订 | 极低（每次 PFAU 启动 / 完成各 1 条） | release 必出（100% 强制全采样） | 含 `feature_id` / `field_changed` / `old_value` / `new_value` / `operator_id`；约 300B/条 |
| `coc.feature_registry.status_transitioned` | `status` 字段从 `pending` → `in_progress` → `active` / `rolling_back` / `rolled_back` / `deprecated` 迁移（与 §4.1 状态机一一对应） | 每次 PFAU 启动 / 完成 / 回滚 1 条 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 状态机迁移强制全采） | 含 `feature_id` / `from_status` / `to_status` / `pfa_run_id`（如有）；约 250B/条 |
| `coc.feature_registry.duplicate_feature_id` | CI 校验或人工录入时主键冲突 | 配置错（应极少） | release 必出（100% 强制全采样） | 含 `feature_id` / `existing_owner_team` / `new_owner_team`；约 250B/条 |
| `coc.feature_registry.invalid_depends_on` | `depends_on` 数组引用不存在的 `feature_id`（图引用断链） | 偶发（首版登记） | release 必出（100% 强制全采样） | 含 `feature_id` / `missing_dependency`；约 200B/条 |
| `coc.feature_registry.row_deprecated` | `AdminService.DeprecateFeature` 写入 `deprecated` 状态 | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `feature_id` / `operator_id` / `deprecation_reason`；约 280B/条 |
| `coc.feature_registry.debug.read_query_plan` | 读查询（典型 `SELECT * WHERE feature_id=?`）的执行计划 dump | 高频（GM 后台查询） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-2KB/条（release 剔除） |
| `coc.feature_registry.debug.full_row_dump` | 完整行 dump（含 `depends_on` 数组 / `notes` 等大字段） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.3）：
- `coc.feature_registry.debug.read_query_plan` 是**高频**事件（GM 后台查询 feature 元数据），**必须** `#[cfg(debug_assertions)]` 守护
- `coc.feature_registry.status_transitioned` 与 §4.1 Feature 元数据生命周期状态机一一对应——是 PFAU 编排可观测性的"持久化层回声"，与 §4.2 PFAU 状态机的 `pfa_run_state.*` 事件按 `feature_id` + `pfa_run_id` 串联

## 3.2 `feature_version_history`（版本历史，FR-PFAU-003 不可变）

```sql
CREATE TABLE feature_version_history (
    history_id          BIGSERIAL   NOT NULL PRIMARY KEY,
    feature_id          TEXT        NOT NULL REFERENCES feature_registry(feature_id),
    version             TEXT        NOT NULL,            -- 语义化版本
    state               TEXT        NOT NULL,            -- declared | active | rolled_back | deprecated
    declared_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    declared_by         TEXT        NOT NULL,            -- 操作者 (AdminService RBAC 角色)
    pfa_run_id          UUID        NULL,                -- 关联的 PFAU 实例
    rollout_batch_count INT         NULL,                -- 灰度批次总数
    confirmed_nodes     TEXT[]      NULL,                -- 全集群确认的节点 ID 列表
    notes               TEXT        NOT NULL DEFAULT '',
    -- 不可变: 不允许 UPDATE/DELETE, 仅 INSERT
    CONSTRAINT chk_state CHECK (state IN ('declared','active','rolled_back','deprecated'))
);

CREATE INDEX idx_feature_version_history_feature_id ON feature_version_history (feature_id, declared_at DESC);
```

DB trigger（FR-DB-001 第③类协同职责）：

```sql
CREATE OR REPLACE FUNCTION prevent_feature_version_history_modify()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'feature_version_history is append-only';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_feature_version_history_update
BEFORE UPDATE OR DELETE ON feature_version_history
FOR EACH ROW EXECUTE FUNCTION prevent_feature_version_history_modify();
```

### 3.2 本功能日志设计

`feature_version_history` 是**不可变**表（DB trigger 拦截 UPDATE/DELETE），仅 INSERT 事件 release 必出——这是 §3.1 `feature_registry.status_transitioned` 的"版本侧回声"，按 `pfa_run_id` 关联。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.feature_version_history.row_inserted` | PFAU 状态迁移触发 INSERT（典型 `declared` / `active` / `rolled_back` / `deprecated` 四个 state 转换） | 每次 PFAU 4 条（start/canary_confirmed/completed/rolled_back 或 completed 等） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 不可变审计） | 含 `history_id` / `feature_id` / `version` / `state` / `declared_by` / `pfa_run_id`；约 300B/条 |
| `coc.feature_version_history.update_attempted_blocked` | 收到 UPDATE/DELETE 请求被 trigger 拒绝（违反不可变性） | 极少（应为代码 bug） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 安全告警） | 含 `feature_id` / `attempted_op`（update/delete）/ `actor_session` / `trace_id`；约 280B/条 |
| `coc.feature_version_history.delete_attempted_blocked` | 同上，DELETE 路径 | 极少 | release 必出（100% 强制全采样） | 同上字段集 |
| `coc.feature_version_history.debug.full_version_chain` | 某 feature_id 的全部历史行 dump（运维追溯用） | 偶发（按需） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-10KB/条（取决于版本数，release 剔除） |

**debug-only 守护要点**：`coc.feature_version_history.debug.full_version_chain` 涉及整个 feature 全部版本历史（典型 5-50 个版本），release 完全剔除避免 RUST_LOG=debug 误开时撑爆日志通道。

## 3.3 `pfa_run_state`（PFAU 实例状态机，FR-PFAU-010/011）

```sql
CREATE TABLE pfa_run_state (
    run_id              UUID        NOT NULL PRIMARY KEY,
    feature_id          TEXT        NOT NULL REFERENCES feature_registry(feature_id),
    from_version        TEXT        NOT NULL,
    to_version          TEXT        NOT NULL,
    direction           TEXT        NOT NULL,            -- upgrade | rollback
    state               TEXT        NOT NULL,            -- declared | canary_in_progress | canary_confirmed | completed | paused | rolled_back | failed
    current_batch       INT         NOT NULL DEFAULT 0,  -- 当前灰度批次
    total_batches       INT         NOT NULL,            -- 灰度批次总数
    batch_size_pct      INT[]       NOT NULL,            -- 每批次节点百分比, 如 [20,20,20,20,20]
    target_node_ids     TEXT[]      NOT NULL,            -- 目标节点 ID 列表
    confirmed_node_ids  TEXT[]      NOT NULL DEFAULT '{}',
    failed_node_ids     TEXT[]      NOT NULL DEFAULT '{}',
    declared_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    declared_by         TEXT        NOT NULL,
    completed_at        TIMESTAMPTZ NULL,
    pause_reason        TEXT        NULL,                -- 暂停原因 (人工重试/跳过/回滚前)
    last_heartbeat_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_state CHECK (state IN ('declared','canary_in_progress','canary_confirmed','completed','paused','rolled_back','failed')),
    CONSTRAINT chk_direction CHECK (direction IN ('upgrade','rollback'))
);

CREATE INDEX idx_pfa_run_state_feature_id ON pfa_run_state (feature_id, declared_at DESC);
CREATE INDEX idx_pfa_run_state_state ON pfa_run_state (state) WHERE state IN ('declared','canary_in_progress','canary_confirmed','paused');
```

### 3.3 本功能日志设计

`pfa_run_state` 是 PFAU 实例的"实时状态"表，与 §4.2 PFAU 状态机一一对应——任何状态字段变更（`state` / `current_batch` / `confirmed_node_ids` / `failed_node_ids` / `pause_reason`）均 release 必出，便于 SRE 按 `run_id` 排查 PFAU 编排过程。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.pfa_run_state.instance_created` | `ClusterOpsService.StartPFAU` 创建新 `run_id` | 极低（每次 PFAU 启动 1 条） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 配置变更衍生） | 含 `run_id` / `feature_id` / `from_version` / `to_version` / `direction`（upgrade/rollback）/ `declared_by` / `total_batches` / `batch_size_pct`；约 350B/条 |
| `coc.pfa_run_state.batch_advanced` | `current_batch` 递增（每批次 canary 完成时） | 每次 PFAU 推进 1 批 | release 必出（100% 强制全采样） | 含 `run_id` / `current_batch` / `total_batches` / `confirmed_count` / `failed_count`；约 280B/条 |
| `coc.pfa_run_state.state_transitioned` | `state` 字段从 `declared` → `canary_in_progress` → `canary_confirmed` → `completed` / `paused` / `rolled_back` / `failed` 迁移 | 每次 PFAU 3-5 条 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 状态机迁移强制全采） | 含 `run_id` / `from_state` / `to_state` / `pause_reason`（如有）；约 250B/条 |
| `coc.pfa_run_state.node_confirmed` | 某批次 canary 节点回执确认 | 每批次 N 条（N = 节点数 × 批次百分比） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 强制全采样白名单） | 含 `run_id` / `node_id` / `batch` / `confirmed_at`；约 250B/条 × 节点数 |
| `coc.pfa_run_state.node_failed` | 某批次 canary 节点回执失败（如版本不兼容） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `run_id` / `node_id` / `batch` / `error` / `trace_id`；约 320B/条 |
| `coc.pfa_run_state.paused_by_operator` | `AdminService.PausePFAU` 人工暂停 | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `run_id` / `operator_id` / `pause_reason`；约 280B/条 |
| `coc.pfa_run_state.rolled_back` | `AdminService.RollbackPFAU` 触发回滚 | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `run_id` / `operator_id` / `from_version` / `to_version` / `reason`；约 320B/条 |
| `coc.pfa_run_state.heartbeat_timeout` | `last_heartbeat_at` 超过阈值（per §4.3 灰度批次推进规则） | 极少（编排器故障） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `run_id` / `last_heartbeat_at` / `timeout_threshold_ms`；约 280B/条 |
| `coc.pfa_run_state.completed` | 全部批次完成，`state=completed` | 每次 PFAU 1 条 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 配置变更完成） | 含 `run_id` / `feature_id` / `duration_ms` / `total_confirmed_nodes`；约 280B/条 |
| `coc.pfa_run_state.debug.full_node_table` | 某 PFAU 实例的 `target_node_ids` + `confirmed_node_ids` + `failed_node_ids` 完整三表 dump | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-5KB/条（节点数决定，release 剔除） |
| `coc.pfa_run_state.debug.batch_progression_timeline` | 批次推进完整时间线（每批次开始/结束时刻 + 节点回执延迟） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 2-8KB/条（release 剔除） |

**debug-only 守护要点**：
- `coc.pfa_run_state.node_confirmed` 是**高频**事件（每批次 canary N 节点回执），release 必出 + 100% 全采样（per BAS-004 v0.3 §6.2 配置热更新强制全采样）——**不能**挂 `#[cfg]`，这是 PFAU 编排进度的核心证据链
- `coc.pfa_run_state.debug.batch_progression_timeline` 在大集群下可能 8KB+ —— release 完全剔除

## 3.4 `event_schema_registry` 与 `event_producer_registry`（FR-CEM-001/010）

```sql
CREATE TABLE event_schema_registry (
    event_type          TEXT        NOT NULL PRIMARY KEY,  -- 如 'rgs.evt.spring_festival_2026.player_rewarded'
    feature_id          TEXT        NOT NULL REFERENCES feature_registry(feature_id),
    schema_version      INT         NOT NULL DEFAULT 1,
    schema_lang         TEXT        NOT NULL,              -- 'json_schema' | 'protobuf'
    schema_ref          TEXT        NOT NULL,              -- 源码仓库 commit hash
    partition_key_rule  TEXT        NOT NULL,              -- 'rgs.evt.<feature>.player_id' 等
    retention_days      INT         NOT NULL DEFAULT 7,
    deprecated_at       TIMESTAMPTZ NULL,
    registered_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    registered_by       TEXT        NOT NULL,
    CONSTRAINT chk_schema_lang CHECK (schema_lang IN ('json_schema','protobuf'))
);

CREATE TABLE event_producer_registry (
    event_type          TEXT        NOT NULL REFERENCES event_schema_registry(event_type),
    app_id              TEXT        NOT NULL,              -- Producer 所在 App
    app_version         TEXT        NOT NULL,              -- App 版本
    first_seen_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (event_type, app_id, app_version)
);

CREATE INDEX idx_event_producer_app ON event_producer_registry (app_id);
```

### 3.4 本功能日志设计

`event_schema_registry` / `event_producer_registry` 双表是 CEM（Cluster Event Mesh）的元数据根基——`event_type` 注册与 Producer 自我声明。CRUD 事件 release 必出（per BAS-004 v0.3 §6.2 事件基础设施元数据），Producer 自我声明是高频（每 App 启动 1 次），但属于"基础设施心跳"性质，按 `app_id` 维度去重即可。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.event_schema_registry.row_registered` | 新 `event_type` 在 CEM 注册（首次声明） | 极低（每 Feature 1-3 个新事件类型） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 事件基础设施元数据） | 含 `event_type` / `feature_id` / `schema_version` / `schema_lang` / `partition_key_rule` / `retention_days` / `registered_by`；约 380B/条 |
| `coc.event_schema_registry.schema_version_bumped` | 同一 `event_type` 的 `schema_version` 递增（含旧版本 deprecation） | 偶发（Feature 演进） | release 必出（100% 强制全采样） | 含 `event_type` / `old_version` / `new_version` / `schema_ref`（commit hash）；约 300B/条 |
| `coc.event_schema_registry.deprecated` | `deprecated_at` 写入（旧 schema 标记弃用） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 弃用事件） | 含 `event_type` / `deprecated_at` / `deprecation_reason`；约 280B/条 |
| `coc.event_schema_registry.duplicate_event_type` | 重复注册同一 `event_type`（主键冲突） | 配置错 | release 必出（100% 强制全采样） | 含 `event_type` / `existing_feature_id` / `new_feature_id`；约 280B/条 |
| `coc.event_producer_registry.producer_first_seen` | 新 Producer 组合（`event_type` + `app_id` + `app_version`）首次声明 | 偶发（每 App 升级 1 次） | release 必出（100% 强制全采样） | 含 `event_type` / `app_id` / `app_version` / `feature_id`；约 300B/条 |
| `coc.event_producer_registry.producer_heartbeat` | 既有 Producer 在 `event_producer_registry` 更新 `last_seen_at`（每 N 分钟一次） | 每 App × 事件类型，每 5-15min | release 必出（按 `app_id` 维度抽样，5-15% 采样率）+ 自动去重（同一 `app_id` 1h 内只留 1 条） | 含 `event_type` / `app_id` / `app_version` / `last_seen_at`；约 220B/条 × 5% = 11B/条 |
| `coc.event_producer_registry.producer_stale` | `last_seen_at` 超过阈值（如 24h 未更新）触发告警 | 极少（App 下线） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 告警事件） | 含 `event_type` / `app_id` / `app_version` / `stale_hours`；约 280B/条 |
| `coc.event_schema_registry.debug.full_schema_dump` | 完整 schema dump（json_schema 文本 / protobuf 描述符） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB/条（schema 大小决定，release 剔除） |
| `coc.event_producer_registry.debug.producer_inventory` | 某 feature_id 的全部 Producer 清单（含 app_version 分布） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |

**debug-only 守护要点**：
- `coc.event_producer_registry.producer_heartbeat` 是**高频**事件（每 App × 事件类型 × 5-15min）——release 必出但**5-15% 抽样**（避免日志通道被心跳淹没），per BAS-004 v0.3 §6.1 采样率
- `coc.event_schema_registry.debug.full_schema_dump` 可能含 protobuf 描述符（10KB+）——release 完全剔除

## 3.5 `event_dlq_view`（视图，FR-CEM-040 死信队列只读视图）

DLQ本身由事件总线维护（不归COC所有），COC仅以**视图**形式聚合展示。视图定义：

```sql
CREATE VIEW event_dlq_view AS
SELECT
    esr.event_type,
    esr.feature_id,
    COUNT(*) FILTER (WHERE dlq.dead_at > now() - INTERVAL '1 hour') AS last_1h_count,
    COUNT(*) FILTER (WHERE dlq.dead_at > now() - INTERVAL '24 hour') AS last_24h_count,
    MAX(dlq.dead_at) AS latest_dead_at
FROM event_schema_registry esr
LEFT JOIN <event_bus_dlq_table> dlq ON dlq.event_type = esr.event_type
GROUP BY esr.event_type, esr.feature_id;
-- 实际 JOIN 取决于事件总线实现, 此处为示意
```

### 3.5 本功能日志设计

`event_dlq_view` 是**只读视图**（DLQ 本身由事件总线维护），不产生新业务事件。视图刷新失败/聚合查询失败产生 release 必出事件（运维告警信号），查询结果详情 debug-only。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.event_dlq_view.refresh_failed` | 视图刷新失败（事件总线 DLQ 表 JOIN 失败） | 极少（数据源事故） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `error` / `trace_id`；约 280B/条 |
| `coc.event_dlq_view.high_dlq_count_detected` | `last_1h_count` 超过阈值触发告警（如 1h 内 > 100 条死信） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 告警事件） | 含 `event_type` / `feature_id` / `last_1h_count` / `last_24h_count`；约 280B/条 |
| `coc.event_dlq_view.query_served` | `AdminService.QueryDLQView` 返回结果 | GM 后台轮询 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 GM 指令衍生） | 含 `request_id` / `operator_id` / `result_count` / `latency_ms`；约 250B/条 |
| `coc.event_dlq_view.debug.raw_dlq_samples` | 视图底层 DLQ 表的样本 dump（最近 10 条死信全文） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-10KB/条（release 剔除，DLQ 样本可能含事件 payload 隐私字段） |

**debug-only 守护要点**：`coc.event_dlq_view.debug.raw_dlq_samples` 可能含事件 payload 隐私字段——release 完全剔除避免 RUST_LOG=debug 误开时泄漏。

## 3.6 `coc_audit_view`（视图，FR-COC-040 审计查询只读视图）

```sql
CREATE VIEW coc_audit_view AS
SELECT
    oa.audit_id,
    oa.operator_id,
    oa.operator_role,
    oa.action_type,           -- 'feature.upgrade' | 'feature.rollback' | 'feature.plug' | 'feature.unplug' | 'dlq.replay' | ...
    oa.target_feature_id,
    oa.target_version,
    oa.pfa_run_id,
    oa.result,                -- 'success' | 'failed' | 'paused'
    oa.created_at
FROM operation_audit oa
WHERE oa.action_type LIKE 'coc.%' OR oa.action_type IN (
    'feature.upgrade','feature.rollback','feature.plug','feature.unplug',
    'dlq.replay','event_registry.update'
);
-- operation_audit 表结构既有, 此处仅为视图过滤 COC 相关操作
```

### 3.6 本功能日志设计

`coc_audit_view` 是审计查询只读视图（基于 `operation_audit` 表过滤 COC 相关操作）。审计写层事件已在 BAS-003 §7.1 统一设计（`audit.write.*`），本视图**不**产生新业务事件，仅视图查询/刷新失败产生 release 必出事件。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.audit_view.query_served` | `AdminService.QueryAuditLog` 经本视图过滤后返回 | GM 后台查询 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `request_id` / `operator_id` / `result_count` / `latency_ms`；约 250B/条 |
| `coc.audit_view.refresh_failed` | 视图刷新失败（基础表 `operation_audit` JOIN 失败） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `error` / `trace_id`；约 280B/条 |
| `coc.audit_view.filter_invalid_action_type` | 查询 filter 含未在白名单的 `action_type`（视图过滤失效） | 配置错 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 安全告警） | 含 `attempted_action_type` / `operator_id` / `request_id`；约 280B/条 |
| `coc.audit_view.debug.full_audit_chain` | 某 `audit_id` 的完整审计链 dump（含关联 `pfa_run_id` / `feature_id` 反查结果） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-5KB/条（release 剔除） |

**debug-only 守护要点**：`coc.audit_view.debug.full_audit_chain` 涉及跨表反查（`operation_audit` + `pfa_run_state` + `feature_registry`），数据量大——release 完全剔除。

# 4. Feature 元数据与 PFAU 状态机

## 4.1 Feature 元数据生命周期

```
            ┌──────────┐
            │ declared │  (新 Feature 在 feature_registry 创建)
            └─────┬────┘
                  │ AdminService.RegisterFeature
                  ▼
            ┌──────────┐
   ┌───────►│  active  │  (current_version 已生效, 全集群确认)
   │        └─────┬────┘
   │              │ AdminService.DeclareFeatureUpgrade
   │              ▼
   │        ┌─────────────────┐
   │        │ upgrade_pending │  (target_version 已声明, 等待 PFAU 启动)
   │        └────────┬────────┘
   │                 │ ClusterOpsService.StartPFAU
   │                 ▼
   │        ┌──────────────────────┐
   │        │ upgrade_canary_*     │  (PFAU 实例在 pfa_run_state 中)
   │        │   declared           │
   │        │   canary_in_progress │
   │        │   canary_confirmed   │
   │        │   completed ─────────┼──► 回到 active (新 current_version)
   │        │   paused            │
   │        │   failed            │
   │        └──────────────────────┘
   │                 │
   │                 │ AdminService.RollbackFeature (人工触发)
   │                 ▼
   │        ┌──────────────────┐
   │        │ rollback_canary_*│
   │        │   ...            │
   │        │   completed ─────┼──► 回到 active (current_version 回旧)
   │        │   paused         │
   │        │   failed         │
   │        └──────────────────┘
   │
   │  AdminService.DeprecateFeature
   ▼
┌──────────────┐
│ deprecated   │  (不再接受新调用, 但已部署实例仍可服务存量)
└──────┬───────┘
       │ AdminService.RemoveFeature (经数据合规评审, 同 RGS-REQ-006 FR-MNT-013)
       ▼
┌──────────────┐
│  removed     │  (从 feature_registry 物理删除, 但 feature_version_history 保留)
└──────────────┘
```

### 4.1 本功能日志设计

Feature 元数据生命周期是 §3.1 `coc.feature_registry.status_transitioned` 的"语义层回声"——本节按 §4.1 状态机迁移点给出 GM 后台可观察的"业务事件"层视角（与持久化层一一对应）。每个状态迁移 release 必出（per BAS-004 v0.3 §6.2 状态机迁移强制全采），便于运营按 `feature_id` 时间线审计"Feature 何时光荣 / 何时回滚 / 何时弃用"。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.feature.lifecycle.registered` | Feature 在 `feature_registry` 创建，状态从无 → `pending` | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 合规审计） | 含 `feature_id` / `feature_type` / `display_name` / `owner_team` / `operator_id`；约 250B/条 |
| `coc.feature.lifecycle.upgrade_declared` | `AdminService.DeclareFeatureUpgrade` 触发，状态 `active` → `upgrade_pending` | 极低 | release 必出（100% 强制全采样） | 含 `feature_id` / `from_version` / `to_version` / `operator_id` / `expected_batches`；约 300B/条 |
| `coc.feature.lifecycle.pfau_started` | `ClusterOpsService.StartPFAU` 触发，状态 `upgrade_pending` → `in_progress` | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含 `feature_id` / `pfa_run_id` / `direction`（upgrade/rollback）/ `batch_size_pct`；约 300B/条 |
| `coc.feature.lifecycle.upgrade_completed` | PFAU 完成，状态 `in_progress` → `active`（新 current_version） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 配置变更完成） | 含 `feature_id` / `pfa_run_id` / `new_current_version` / `duration_ms`；约 280B/条 |
| `coc.feature.lifecycle.rollback_initiated` | `AdminService.RollbackFeature` 触发，状态 `in_progress` → `rolling_back` | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `feature_id` / `pfa_run_id` / `operator_id` / `reason` / `target_version`；约 320B/条 |
| `coc.feature.lifecycle.rollback_completed` | 回滚 PFAU 完成，状态 `rolling_back` → `rolled_back` | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `feature_id` / `pfa_run_id` / `restored_version` / `duration_ms`；约 280B/条 |
| `coc.feature.lifecycle.deprecated` | `AdminService.DeprecateFeature`，状态 `active` → `deprecated` | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `feature_id` / `operator_id` / `deprecation_reason` / `expected_removal_date`；约 320B/条 |
| `coc.feature.lifecycle.removed` | `AdminService.RemoveFeature`，状态 `deprecated` → `removed`（物理删除） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 数据删除合规） | 含 `feature_id` / `operator_id` / `compliance_review_ref` / `feature_version_history_retained`；约 350B/条 |
| `coc.feature.lifecycle.invalid_transition_attempted` | 非法状态机迁移（如 `removed` → `active`）被 AdminService 拒绝 | 配置错 / 攻击 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 安全告警） | 含 `feature_id` / `from_status` / `attempted_to_status` / `operator_id` / `request_id`；约 320B/条 |
| `coc.feature.lifecycle.debug.full_history_timeline` | 某 feature_id 的完整生命周期时间线（所有 status 迁移 + 对应 PFAU 实例） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-8KB/条（生命周期长度决定，release 剔除） |

**debug-only 守护要点**：
- `coc.feature.lifecycle.invalid_transition_attempted` 是**安全告警**——release 必出，便于按 `operator_id` 维度识别异常操作模式
- `coc.feature.lifecycle.debug.full_history_timeline` 涉及多年 Feature 累积的完整时间线，release 完全剔除

## 4.2 PFAU 状态机（FR-PFAU-010/011）

```
                ┌────────────┐
       ┌───────►│  declared  │  (AdminService.DeclareFeatureUpgrade 调用后)
       │        └─────┬──────┘
       │              │ ClusterOpsService 自动启动
       │              ▼
       │        ┌──────────────────────┐
       │        │ canary_in_progress   │  (第 N 批灰度进行中, N 从 1 开始)
       │        └──────┬───────────────┘
       │               │
       │        ┌──────┴──────┬──────────────┐
       │        ▼             ▼              ▼
       │   全部确认        部分确认+        全部失败
       │   (确认延迟       至少 1 个       (超过重试)
       │    < 阈值)        失败)
       │        │             │              │
       │        │             │              ▼
       │        │             │       ┌──────────┐
       │        │             │       │  paused  │ ◄──── (FR-PFAU-021 超时)
       │        │             │       └─────┬────┘       (FR-PFAU-010 任何失败)
       │        │             │             │
       │        │             │             │ 人工三选一:
       │        │             │             │  retry / skip / rollback
       │        │             │             ▼
       │        │             │     ┌──────────────────┐
       │        │             │     │ 人工决策后继续    │
       │        │             │     └──────────────────┘
       │        ▼             ▼
       │   ┌──────────────────────┐
       │   │ canary_confirmed      │  (所有批次完成 + 跨节点一致确认)
       │   └─────┬────────────────┘
       │         │ ClusterOpsService 自动推进
       │         ▼
       │   ┌────────────┐
       └────┤ completed │  (feature_registry.current_version 更新, 历史追加)
           └────────────┘
                │ 任何时刻可触发:
                │  AdminService.RollbackFeature
                ▼
           ┌──────────────────────┐
           │ rollback_canary_*     │  (类似 upgrade 状态机, 目标为 from_version)
           │   declared           │
           │   canary_in_progress │
           │   canary_confirmed   │
           │   completed ─────────┼──► 回到 active
           │   paused             │
           │   failed             │
           └──────────────────────┘
```

**关键约束**：
- 任一状态迁移**必须**写一条审计记录至`operation_audit`（FR-COC-040）
- `paused` 状态**必须**含 `pause_reason` 字段
- 跨节点一致性确认（FR-PFAU-020）通过"运行时通过既有健康检查端点声明'我已加载目标版本' → 控制面收集所有节点声明"实现
- 自动回滚（FR-PFAU-022）**仅**在节点失联（K8s Pod异常退出/节点失联）场景触发

## 4.3 灰度批次推进规则

灰度批次推进（FR-PFAU-012）伪代码：

```python
async def advance_canary(run: PfaRunState):
    if run.current_batch >= run.total_batches:
        # 全部批次完成, 跨节点一致确认
        if all_confirmed(run.target_node_ids, timeout=120):
            transition(run, "completed")
            update_feature_registry(run.feature_id, current_version=run.to_version)
            insert_feature_version_history(run)
        else:
            transition(run, "paused", reason="confirmation_timeout")
        return

    batch = run.batch_size_pct[run.current_batch]
    target_nodes = select_nodes(run.target_node_ids, batch_pct=batch, strategy=...)

    for node in target_nodes:
        try:
            await invoke_upgrade(node, run.feature_id, run.to_version)
        except Exception as e:
            run.failed_node_ids.append(node.id)
            log.error(...)

    # 等待观察期
    await sleep(run.observation_window_seconds)

    # 全部成功 + 健康检查通过 才推进
    if all_healthy(target_nodes) and len(run.failed_node_ids) == 0:
        run.current_batch += 1
        transition(run, "canary_in_progress" if run.current_batch < run.total_batches else "canary_confirmed")
    else:
        transition(run, "paused", reason="batch_failed")
```

# 5. CEM 探针订阅器设计

## 5.1 探针部署形态

CEM 探针订阅器（CEMProbeAggregator）作为AD限界上下文的**附属进程**（sidecar 或 Deployment 内独立进程），**不**作为独立Deployment。理由：①与AD限界上下文同生命周期管理；②共享AD的`admin_db`连接池与可观测性埋点；③避免新建独立Helm chart。

## 5.2 探针工作流

```
┌──────────────────────┐
│ 事件总线 (Topic X)   │
└──────────┬───────────┘
           │ 既有事件流 (Producer → Topic → Consumer)
           │
           ├─ 正常消费者 (各 App 既有 Consumer)
           │
           └─ 探针订阅器 (CEM 独有, 走"只读镜像"链路)
                  │
                  │ 1. 解析 event_type, 查询 event_schema_registry
                  │    - 若未注册: 写"未注册事件"告警, 继续监听不阻塞
                  │ 2. 更新 event_producer_registry (UPSERT last_seen_at)
                  │ 3. 采样指标 (Producer 速率, Schema 命中率)
                  │ 4. 不影响正常消费者 (走独立 Consumer Group)
                  ▼
            ┌──────────────────────┐
            │ admin_db             │
            │ event_producer_      │
            │ registry (UPSERT)    │
            └──────────────────────┘
```

## 5.3 探针的关键约束

- **不**消费事件内容——仅解析 `event_type`/`event_version`/`producer_id` 三字段后丢弃payload，**不**进入事件内容处理路径
- **走独立Consumer Group**——`coc.cem.probe`，与正常消费者**不**共享offset，**不**阻塞正常消费
- **批量写**——每5秒批量UPSERT `event_producer_registry`，**不**逐事件写DB（避免admin_db的写入热点）
- **指标采样**——通过OTel SDK导出 producer 速率与 schema 命中率指标，复用既有可观测性基础设施

## 5.4 死信队列与可重放历史

- **DLQ**：由事件总线维护，COC通过`event_dlq_view`只读聚合（§3.5），不直接管理DLQ存储
- **可重放历史**：事件总线本身提供`Replay` API（FR-CEM-051），COC通过`AdminService.ReplayEvents`调用并审计

### 5.5 本功能日志设计（CEM 探针全链路综合）

`CEMProbeAggregator` 与 §3.4 `event_producer_registry` 是同一逻辑链路的"运行时/持久化"两个观察点；DLQ/可重放历史在 §3.5 `event_dlq_view` 已覆盖。本节覆盖探针本身的启停/订阅/采样/失败/重试事件。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.cem.probe.started` | CEM 探针容器启动并完成事件总线连接 | 每节点启动 1 次 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 基础设施心跳） | 含 `node_id` / `event_bus_endpoint` / `consumer_group_id`；约 250B/条 |
| `coc.cem.probe.stopped` | 探针容器优雅关闭（SIGTERM） | 每节点关闭 1 次 | release 必出（100% 强制全采样） | 含 `node_id` / `inflight_event_count` / `shutdown_kind`；约 280B/条 |
| `coc.cem.probe.subscribed` | 探针对某 `event_type` 完成 Consumer Group 订阅 | 偶发（订阅关系变化） | release 必出（100% 强制全采样） | 含 `event_type` / `feature_id` / `partition_count` / `consumer_group_id`；约 280B/条 |
| `coc.cem.probe.unsubscribed` | 探针取消某 `event_type` 订阅（如 Feature 弃用） | 偶发 | release 必出（100% 强制全采样） | 含 `event_type` / `feature_id` / `reason`（deprecation/manual）；约 280B/条 |
| `coc.cem.probe.event_sampled` | 探针对某事件做采样并写入聚合视图 | 取决于业务流量（典型 10-100/s 集群） | release 必出（5-10% 采样率） | 含 `event_type` / `feature_id` / `partition_key` / `payload_size_bytes`；约 250B/条 × 5% = 12.5B/条 |
| `coc.cem.probe.consumer_lag_high` | 探针 Consumer Group lag 超过阈值（如 > 1000 条）触发告警 | 偶发（业务流量突增） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 告警事件） | 含 `event_type` / `lag_count` / `threshold` / `node_id`；约 280B/条 |
| `coc.cem.probe.event_bus_unreachable` | 探针与事件总线连接断开 | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `event_bus_endpoint` / `error` / `retry_count` / `trace_id`；约 320B/条 |
| `coc.cem.probe.subscription_failed` | 探针订阅某 `event_type` 失败（event_bus 拒接 / 权限不足） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `event_type` / `feature_id` / `error` / `trace_id`；约 300B/条 |
| `coc.cem.probe.replay_started` | `AdminService.ReplayEvents` 触发事件重放 | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 数据回灌合规） | 含 `event_type` / `time_range` / `operator_id` / `consumer_group_target`；约 320B/条 |
| `coc.cem.probe.replay_completed` | 事件重放完成 | 极低 | release 必出（100% 强制全采样） | 含 `event_type` / `replayed_count` / `duration_ms` / `operator_id`；约 280B/条 |
| `coc.cem.probe.replay_failed` | 事件重放失败（如时间窗超出 retention） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `event_type` / `error` / `operator_id` / `trace_id`；约 320B/条 |
| `coc.cem.probe.dlq_discarded` | DLQ 事件被人工丢弃（`AdminService.DiscardDLQEvent`，FR-CEM-041） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `dlq_event_id` / `event_type` / `operator_id` / `discard_reason`；约 320B/条 |
| `coc.cem.probe.debug.event_payload_sample` | 事件 payload 完整 dump（采样命中后） | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-5KB/条（payload 大小决定，**含业务隐私风险**，release 剔除） |
| `coc.cem.probe.debug.consumer_group_state` | Consumer Group 完整状态 dump（成员/分区分配/lag 分布） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 1-3KB/条（release 剔除） |

**debug-only 守护要点**：
- `coc.cem.probe.event_sampled` 是**高频**事件（10-100/s 集群），release 必出但**5-10% 抽样**避免日志通道淹没
- `coc.cem.probe.debug.event_payload_sample` **必须** `#[cfg(debug_assertions)]` 守护——事件 payload 可能含 PII（运营 ID / 玩家 ID / 业务字段），release 误开 RUST_LOG=debug 时**不能**泄漏
- `coc.cem.probe.replay_started` / `replay_completed` / `replay_failed` 是**数据回灌事件**——release 必出 + 全采样（合规审计要求"谁在何时重放了哪些事件"必须可追溯）

# 6. API 契约字段级定义

## 6.1 gRPC 方法列表

| 方法 | 方向 | 流式 | 用途 |
|---|---|---|---|
| `ClusterOpsService.RegisterFeature` | C→S | Unary | 注册新 Feature (创建 feature_registry 行) |
| `ClusterOpsService.UpdateFeature` | C→S | Unary | 更新 Feature 元数据 (display_name/owner_team/depends_on) |
| `ClusterOpsService.DeprecateFeature` | C→S | Unary | 标记 Feature 为 deprecated |
| `ClusterOpsService.DeclareFeatureUpgrade` | C→S | Server stream | 声明 Feature 升级 (返回 PFAU 启动进度) |
| `ClusterOpsService.DeclareFeatureRollback` | C→S | Server stream | 声明 Feature 回滚 |
| `ClusterOpsService.GetPfaRunState` | C→S | Unary | 查询 PFAU 实例状态 |
| `ClusterOpsService.AdvanceCanary` | C→S | Unary | 人工推进灰度批次 (retry/skip/rollback) |
| `ClusterOpsService.ListFeatures` | C→S | Unary | 功能矩阵列表 (支持 type/status 筛选) |
| `ClusterOpsService.ListEventSchemas` | C→S | Unary | 事件 Schema 目录列表 |
| `ClusterOpsService.GetEventSchema` | C→S | Unary | 单个事件 Schema 详情 |
| `ClusterOpsService.RegisterEvent` | C→S | Unary | 注册新 event_type (CEM 注册表) |
| `ClusterOpsService.UpdateEventSchema` | C→S | Unary | 更新 event Schema (FR-CEM-003) |
| `ClusterOpsService.ReplayEvents` | C→S | Server stream | 重放 Topic 事件 (FR-CEM-051) |
| `ClusterOpsService.DiscardDlqEvent` | C→S | Unary | 单条丢弃 DLQ 事件 (FR-CEM-041, **自审补强**) |
| `ClusterOpsService.ListDlqEvents` | C→S | Unary | DLQ 列表查询 (FR-CEM-041 配套) |

> 全部方法**必须**经`AdminService`转发（FR-API-005），**不**让COC UI或第三方直接调用ClusterOpsService。

## 6.2 关键消息字段定义

### 6.2.1 `RegisterFeatureRequest`

```protobuf
message RegisterFeatureRequest {
    string feature_id = 1;              // 稳定不变, 如 'rgs.evt.spring_festival_2026'
    FeatureType feature_type = 2;       // BOUNDED_CONTEXT | PLUGIN | PATCH | CONFIG
    string display_name = 3;
    string description = 4;
    string current_version = 5;         // 初始版本, 语义化
    repeated string depends_on = 6;     // Feature ID 列表
    string owner_team = 7;
    string request_id = 8;              // 幂等键, 同 ARC-009
}

enum FeatureType {
    FEATURE_TYPE_UNSPECIFIED = 0;
    BOUNDED_CONTEXT = 1;
    PLUGIN = 2;
    PATCH = 3;
    CONFIG = 4;
}
```

### 6.2.2 `DeclareFeatureUpgradeRequest` 与响应流

```protobuf
message DeclareFeatureUpgradeRequest {
    string feature_id = 1;
    string target_version = 2;          // 语义化
    CanaryStrategy strategy = 3;        // 灰度策略
    repeated int32 batch_size_pct = 4;  // 每批次节点百分比
    int32 observation_window_seconds = 5; // 批次间观察期, 默认 300
    int32 confirmation_timeout_seconds = 6; // 跨节点确认超时, 默认 120 (FR-PFAU-021)
    string request_id = 7;              // 幂等键
}

message DeclareFeatureUpgradeResponse {
    oneof event {
        PfaRunInitialized initialized = 1;        // PFAU 启动成功
        PfaRunStateUpdate state_update = 2;        // 状态机推进
        PfaRunCompleted completed = 3;             // PFAU 完成
        PfaRunPaused paused = 4;                   // PFAU 暂停 (含 pause_reason)
        PfaRunFailed failed = 5;                   // PFAU 失败
    }
}

message PfaRunStateUpdate {
    string run_id = 1;
    PfaRunState state = 2;
    int32 current_batch = 3;
    int32 total_batches = 4;
    repeated string confirmed_node_ids = 5;
    repeated string failed_node_ids = 6;
    int64 unix_timestamp_ms = 7;
}

enum PfaRunState {
    PFA_RUN_STATE_UNSPECIFIED = 0;
    DECLARED = 1;
    CANARY_IN_PROGRESS = 2;
    CANARY_CONFIRMED = 3;
    COMPLETED = 4;
    PAUSED = 5;
    ROLLED_BACK = 6;
    FAILED = 7;
}
```

### 6.2.3 `ReplayEventsRequest`

```protobuf
message ReplayEventsRequest {
    string topic = 1;
    int64 from_offset = 2;              // -1 表示从头
    int64 to_offset = 3;                // -1 表示到当前高水位
    int64 from_unix_ms = 4;             // 替代 from_offset 的时间窗选项
    int64 to_unix_ms = 5;
    repeated string target_consumer_group_whitelist = 6; // FR-CEM-052 必填
    string original_event_id = 7;       // 原始事件 ID (单条重放场景)
    string replay_request_id = 8;       // 重放操作唯一 ID, 幂等键 (FR-CEM-042)
    string request_id = 9;              // 整体请求幂等键
}
```

### 6.2.4 DLQ 操作请求（自审补强，FR-CEM-041）

```protobuf
// 单条丢弃 DLQ 事件 (FR-CEM-041)
// 注意: "丢弃"指从 DLQ 物理删除, 事件不再可重放
// 适用于: 已知是误投递 / 重复事件 / 测试残留等场景
message DiscardDlqEventRequest {
    string dlq_event_id = 1;            // DLQ 事件唯一 ID (事件总线在 dead_letter 时分配)
    string discard_reason = 2;          // 操作者填写的丢弃原因 (必填, 写入审计)
    string request_id = 3;              // 幂等键
}

message DiscardDlqEventResponse {
    string dlq_event_id = 1;
    int64 discarded_at_unix_ms = 2;
}

// DLQ 列表查询
message ListDlqEventsRequest {
    string topic = 1;                   // 按 Topic 过滤 (空 = 全部)
    int64 from_unix_ms = 2;             // 时间窗起点
    int64 to_unix_ms = 3;               // 时间窗终点
    int32 page_size = 4;                // 默认 50, 最大 500
    string page_token = 5;              // 分页游标
}

message ListDlqEventsResponse {
    repeated DlqEvent events = 1;
    string next_page_token = 2;
}

message DlqEvent {
    string dlq_event_id = 1;
    string original_event_id = 2;
    string topic = 3;
    int64 dead_at_unix_ms = 4;
    int32 retry_count = 5;
    string last_error = 6;
}
```

## 6.3 错误码

| 错误码 | 含义 | HTTP/gRPC status |
|---|---|---|
| `FEATURE_NOT_FOUND` | feature_id 不存在 | NOT_FOUND |
| `FEATURE_TYPE_MISMATCH` | 操作与 Feature 类型不匹配 (如对配置型 Feature 调用 plug) | INVALID_ARGUMENT |
| `PFAU_ALREADY_RUNNING` | 该 Feature 已有 PFAU 实例在进行中 | FAILED_PRECONDITION |
| `PFAU_INVALID_STATE` | PFAU 状态机不允许该操作 (如对 completed 状态调用 rollback 但无历史版本) | FAILED_PRECONDITION |
| `EVENT_NOT_REGISTERED` | event_type 未在 event_schema_registry (FR-CEM-002) | FAILED_PRECONDITION |
| `REPLAY_DENIED` | 重放操作被拒绝 (无 Consumer Group 白名单 / 时间窗超出 retention) | PERMISSION_DENIED |
| `DISCARD_DENIED` | 丢弃 DLQ 事件被拒绝 (无理由 / 事件已被丢弃) | FAILED_PRECONDITION |
| `DLQ_EVENT_NOT_FOUND` | dlq_event_id 在 DLQ 中不存在 (FR-CEM-041 自审补强) | NOT_FOUND |
| `RBAC_DENIED` | 操作者角色不足 (FR-COC-041) | PERMISSION_DENIED |
| `IDEMPOTENT_REPLAY` | request_id 重复提交, 已返回首次结果 | OK (with previous_result) |

### 6.4 本功能日志设计（API 契约全链路综合）

`ClusterOpsService` 全部 gRPC 方法（§6.1）+ 字段级消息（§6.2）的运行时事件。所有方法调用 release 必出 + 100% 强制全采样（per BAS-004 v0.3 §6.2 GM 指令衍生强制全采白名单），便于 SRE 按 `request_id` / `operator_id` 维度审计控制平面调用。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.api.register_feature.received` | `ClusterOpsService.RegisterFeature` 调用进入处理 | 极低 | release 必出（100% 强制全采样） | 含 `request_id` / `feature_id` / `feature_type` / `operator_id`；约 280B/条 |
| `coc.api.register_feature.completed` | Feature 已写入 `feature_registry`（含事务提交） | 极低 | release 必出（100% 强制全采样） | 含 `request_id` / `feature_id` / `db_tx_id`；约 250B/条 |
| `coc.api.declare_feature_upgrade.received` | `DeclareFeatureUpgrade` 调用（server stream 入口） | 极低 | release 必出（100% 强制全采样） | 含 `request_id` / `feature_id` / `from_version` / `to_version` / `expected_batches` / `operator_id`；约 350B/条 |
| `coc.api.declare_feature_upgrade.pfa_started` | 升级声明触发 `ClusterOpsService.StartPFAU` | 极低 | release 必出（100% 强制全采样） | 含 `request_id` / `feature_id` / `pfa_run_id` / `direction`；约 280B/条 |
| `coc.api.declare_feature_upgrade.stream_progress` | server stream 推送的进度事件（每批 canary 推进） | 每次 PFAU 推进 1 条 | release 必出（100% 强制全采样） | 含 `request_id` / `pfa_run_id` / `current_batch` / `total_batches` / `confirmed_count` / `failed_count`；约 320B/条 |
| `coc.api.declare_feature_upgrade.stream_completed` | server stream 正常结束 | 极低 | release 必出（100% 强制全采样） | 含 `request_id` / `pfa_run_id` / `final_state` / `total_duration_ms`；约 280B/条 |
| `coc.api.start_pfau.received` | `StartPFAU` 调用 | 极低 | release 必出（100% 强制全采样） | 含 `request_id` / `feature_id` / `pfa_run_id` / `batch_size_pct` / `operator_id`；约 320B/条 |
| `coc.api.pause_pfau.received` | `PausePFAU` 调用（人工暂停） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `request_id` / `pfa_run_id` / `operator_id` / `pause_reason`；约 300B/条 |
| `coc.api.rollback_pfau.received` | `RollbackPFAU` 调用（人工回滚） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `request_id` / `pfa_run_id` / `operator_id` / `reason` / `target_version`；约 350B/条 |
| `coc.api.replay_events.received` | `ReplayEvents` 调用 | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 数据回灌合规） | 含 `request_id` / `event_type` / `time_range` / `operator_id`；约 350B/条 |
| `coc.api.discard_dlq_event.received` | `DiscardDLQEvent` 调用（FR-CEM-041 自审补强） | 极低 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 高危操作） | 含 `request_id` / `dlq_event_id` / `operator_id` / `discard_reason`；约 320B/条 |
| `coc.api.discard_dlq_event.rbac_denied` | 操作者角色不足（无 DLQ 操作权限） | 偶发（配置错） | release 必出（100% 强制全采样） | 含 `request_id` / `operator_id` / `operator_role` / `required_role`；约 300B/条 |
| `coc.api.discard_dlq_event.not_found` | `dlq_event_id` 在 DLQ 中不存在（§6.3 `DLQ_EVENT_NOT_FOUND`） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `request_id` / `dlq_event_id` / `event_type` / `trace_id`；约 280B/条 |
| `coc.api.<method>.failed.unexpected` | 上述任一方法未预期内部异常（DB 错误/事件总线不可达/PFAU 调度器崩溃） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `request_id` / `method` / `error` / `trace_id`；约 320B/条 |
| `coc.api.debug.request_envelope` | gRPC 请求 envelope 完整 dump（含 metadata + 完整 request body） | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 0.5-3KB/条（release 剔除） |
| `coc.api.debug.stream_chunk_trace` | server stream 每条推送 chunk 的延迟与大小 | 业务触发同频 | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 200B/条（release 剔除） |

**debug-only 守护要点**：
- `coc.api.declare_feature_upgrade.stream_progress` 是**高频**事件（每 PFAU 推进 1 批），release 必出 + 全采样——**不能**挂 `#[cfg]`，是 PFAU 编排进度可观测性的核心数据流
- `coc.api.discard_dlq_event.*` 是**高危操作**白名单（per BAS-004 v0.3 §6.2），所有派生事件 release 必出便于合规审计
- `coc.api.debug.request_envelope` 完整 dump 含元数据（auth token 等敏感字段）——**严格** `#[cfg(debug_assertions)]` 守护，release 完全剔除

## 6.5 処理フロー（处理流程 / Processing Flow）

> 落实 RGS-BAS-FLOW-STANDARD-2026-09-02 v0.1 四要素标准 (per 2026-09-02 13:59 JST Ulysses 拍板)
> 详细时序见 §4.1 Feature 元数据生命周期 / §4.2 PFAU 状态机 / §4.3 灰度批次推进规则, 本段为全景流程 + 异常分支 + 决策点 + 验证点汇总
> 与 BAS-019 §1.1 范式一致 (commit `d52eaad`)

### 6.5.1 主流程图 (mermaid sequenceDiagram)

```mermaid
sequenceDiagram
    autonumber
    actor GM as GM Operator
    participant AS as AdminService
    participant COS as ClusterOpsService
    participant FR as FeatureRegistry
    participant PR as PFAURunner
    participant CEM as CEMProbeAggregator
    participant DB as admin_db

    Note over GM,DB: trace_id 贯穿全链路, per BAS-004 v0.3 §4.4
    Note over GM,DB: 事务边界: 单次 PFAU 状态机迁移是单事务; 跨批次推进是 Saga 编排, per BAS-100 v0.1

    rect rgb(240, 248, 255)
        Note over GM,DB: 主路径 1: Feature 元数据生命周期 (per §4.1)
        GM->>AS: RegisterFeature (RBAC=cluster_admin)
        AS->>AS: RBAC + 限流校验 (FR-COC-041)
        AS->>COS: 转发 (FR-COC-020, 不绕过统一入口)
        COS->>DB: BEGIN; INSERT feature_registry
        DB-->>COS: feature_id
        COS->>DB: COMMIT
        COS-->>AS: ok
        AS-->>GM: 201 Created

        GM->>AS: DeclareFeatureUpgrade (FR-PFAU-010)
        AS->>COS: 转发
        COS->>DB: BEGIN; UPDATE current_version=target_version, status=upgrade_pending
        DB-->>COS: ok
        COS->>DB: COMMIT; INSERT feature_version_history
        DB-->>COS: history_id
        COS-->>AS: ok
        AS-->>GM: 200 OK
    end

    rect rgb(255, 250, 240)
        Note over GM,DB: 主路径 2: PFAU 编排 (per §4.2/§4.3)
        GM->>AS: StartPFAU (FR-PFAU-011)
        AS->>COS: 转发
        COS->>PR: 启动 PFAU 实例
        PR->>DB: INSERT pfa_run_state (state=declared)
        PR->>PR: 选 canary 节点 (按 batch_size_pct)

        loop 每个 canary 批次 (Saga 跨域编排)
            PR->>CEM: 订阅 canary 节点的事件探针
            CEM-->>PR: 节点回执 (confirmed / failed)
            PR->>DB: UPDATE pfa_run_state.current_batch, confirmed_node_ids
            PR->>DB: 写 coc.pfa_run_state.batch_advanced 事件
        end

        alt 全部批次确认
            PR->>DB: UPDATE pfa_run_state.state=completed; UPDATE feature_registry.current_version=to_version
            PR->>DB: INSERT feature_version_history (state=active)
            PR-->>COS: 升级完成
            COS-->>AS: 200 OK
            AS-->>GM: 完成
        else 任一批次失败 (FR-PFAU-012 失败处理)
            PR->>DB: UPDATE pfa_run_state.state=failed
            PR-->>COS: 升级失败
            GM->>AS: RollbackPFAU (高危, RBAC=cluster_admin)
            AS->>COS: 转发
            COS->>PR: 启动反向 PFAU 实例 (Saga 补偿)
            PR->>DB: 沿 PFAU 状态机反向推进 (per §4.2 rollback 方向)
        end
    end

    Note over GM,DB: 异常通路: 节点失联 / 集群脑裂 -> COS 健康检查告警 (per §10.3); DLQ 累积 -> DLQOperator 重放或丢弃 (per §5.4)
```

### 6.5.2 異常分支表

| 异常点 | 触发条件 | 处理动作 | 用户感知 | 补偿动作 |
|---|---|---|---|---|
| RBAC 拒绝 | 操作者角色不足 (FR-COC-041, 如 `cluster_operator` 调用 `RegisterFeature`) | AdminService 拒绝 (per BAS-003 §7.2) | 403 PERMISSION_DENIED + 写 `coc.api.<method>.rbac_denied` + 审计 | 无 (按设计意图) |
| 节点 canary 失败 | 任一 canary 节点回执 failed (版本不兼容 / 网络超时 / 健康检查不通过) | `pfa_run_state.state=failed` + 触发 `RollbackPFAU` | GM 收到 PFAU 失败告警 (per §6.4 `coc.api.declare_feature_upgrade.stream_progress` 含 `failed_count`) | Saga 反向 PFAU 恢复旧版本 (per §4.2 rollback 方向) |
| 状态机非法迁移 | 非法 status 迁移 (如 `removed` → `active`, per §4.1) | AdminService 拒绝 (写 `coc.feature.lifecycle.invalid_transition_attempted`) | 400 INVALID_ARGUMENT + 告警 | 无 (按设计意图) |
| 节点心跳超时 | `pfa_run_state.last_heartbeat_at` 超过阈值 (per §4.3) | 写 `coc.pfa_run_state.heartbeat_timeout` + 自动 `PausePFAU` | GM 收到 PFAU 暂停告警 | SRE 人工 Resume / Rollback |
| 集群脑裂 / 节点失联 | PFAU 编排器与 canary 节点通信失败 (per §10.3 可用性) | `PausePFAU` + 告警 | GM 收到 PFAU 暂停告警 | SRE 介入, 重建连接后 Resume |
| 跨域 Saga 步骤失败 | 反向 PFAU 沿状态机推进时某步失败 (per §4.2 rollback) | 写 DLQ + 报警 | GM 收到补偿失败告警 (高危) | SRE 介入, 手动恢复版本 (per §11 风险与未决事项) |
| DLQ 累积超阈值 | `event_dlq_view.last_1h_count` 超过阈值 (per §3.5) | 写 `coc.event_dlq_view.high_dlq_count_detected` + 告警 | GM 收到 DLQ 告警 | GM 手动 `ReplayEvents` 或 `DiscardDLQEvent` (高危, 需理由) |
| Feature 删除时合规缺失 | `RemoveFeature` 触发时无 `compliance_review_ref` (per §4.1) | AdminService 拒绝 (写 `coc.feature.lifecycle.invalid_transition_attempted`) | 400 FAILED_PRECONDITION | 无 (按设计意图) |

### 6.5.3 决策点矩阵

| 决策点 | 条件 | 主分支 | 备选分支 | 触发后果 |
|---|---|---|---|---|
| PFAU 灰度批次推进 | 当前批次 canary 节点 `confirmed_node_ids.count` 满足阈值 (per §4.3) | 推进 `current_batch+1`, 进入下一批 canary | `PausePFAU` (人工) / `RollbackPFAU` (高危) | GM 看到批次进度 (per §6.4 `coc.api.declare_feature_upgrade.stream_progress`), 正常推进 / 收到暂停或回滚告警 |
| 回滚 vs 继续 (PFAU 失败时) | 任一批次 `failed_node_ids.count` 超阈值 (per §4.3) | 自动 `pfa_run_state.state=failed`, 等待人工 `RollbackPFAU` | 自动反向 PFAU (per §4.2 rollback 方向) | GM 决定回滚时机, 减少自动回滚风险; 反向 PFAU 失败时写 DLQ |
| DLQ 重放 vs 丢弃 | DLQ 事件超 retention / 数据回灌需求 | `ReplayEvents` (高危, RBAC=cluster_admin) | `DiscardDLQEvent` (高危, 需理由) | 数据回灌 vs 永久丢失, GM 按合规要求选择 (per §3.5 `coc.event_dlq_view.high_dlq_count_detected`) |
| 弃用 vs 删除 | Feature 已 `active` 但需下线 | `DeprecateFeature` (标记 `deprecated`, 保留实例) | `RemoveFeature` (物理删除, 需合规评审 per FR-MNT-013) | 数据保留 vs 数据清除, 按合规与运营需求 (per §4.1) |
| CEM 探针 vs 直连 | Producer SDK 是否在用 | 探针订阅 (CEM 默认, 不改造各 App Publisher SDK) | Producer SDK 直连 (如已有 SDK) | 部署侵入性低 vs 直连性能高, 按 ARC-042 集群部署原则 (per §5.1 探针部署形态) |

### 6.5.4 验证点清单

| 验证时机 | 验证内容 | 通过标准 | 失败处理 |
|---|---|---|---|
| AdminService 入口 RBAC | 操作者角色匹配 (FR-COC-041) | 角色 ∈ {`cluster_operator`, `cluster_admin`} | 返回 403 PERMISSION_DENIED, 写 `coc.api.<method>.rbac_denied` + `operation_audit` 审计 (per BAS-003 §7) |
| Feature 状态机迁移 | 迁移合法性 (per §4.1) | `from_status` → `to_status` 在状态机允许范围 | 返回 400 INVALID_ARGUMENT, 写 `coc.feature.lifecycle.invalid_transition_attempted` |
| PFAU 批次确认节点数 | `confirmed_node_ids.count` ≥ 阈值 (per §4.3 灰度批次推进规则) | 全部 canary 节点 confirmed | 状态机迁移到 `failed`, 触发 `RollbackPFAU` (per §4.2) |
| mTLS 握手 (CEM 探针订阅) | 探针节点证书校验 (per §5.3 探针关键约束) | client cert 校验通过 (per RGS-BAS-003-mTLS v0.1 决策) | 探针订阅失败, 写 `coc.cem.probe.handshake_failed` + 告警 |
| 事务提交 (admin_db) | `feature_registry` / `pfa_run_state` / `feature_version_history` 写入同事务 | COMMIT 成功, `db_tx_id` 已回执 | 整体回滚, 写 `coc.api.<method>.failed.unexpected` + `trace_id` 串联 (per §6.4) |
| 跨域 Saga 补偿 | 反向 PFAU 推进时旧版本已生效 (per §4.2 rollback) | `feature_registry.current_version` 回退成功 | DLQ + 告警, SRE 介入 (per §10.3 可用性) |
| trace_id 串联 | AdminService 入口到 admin_db 写入全链路 trace_id 相同 (per BAS-004 v0.3 §4.4) | 所有事件 `trace_id` 一致 | 不阻断流程, 但缺失 trace_id 时写 `coc.api.debug.missing_trace_id` (debug-only 兜底) |

# 7. COC UI 页面构成与复用 VIZ 渲染能力

## 7.1 页面构成

| 页面 | 路由 | 主要组件 | 数据源 |
|---|---|---|---|
| **功能矩阵** (首页) | `/coc/features` | FeatureList, FeatureDetailDrawer, VersionTimeline | `feature_registry` + `feature_version_history` |
| **事件流** | `/coc/events` | EventTypeList, SchemaViewer, ConsumerGroupHealthGrid, DLQTable, ReplayTimeline | `event_schema_registry` + `event_producer_registry` + OTel |
| **依赖图** | `/coc/dependencies` | InfiniteCanvas (复用 VIZ) | `feature_registry.depends_on` + `pfa_run_state` 当前状态叠加 |
| **灰度面板** | `/coc/canary` | PfaRunList, PfaRunProgress (批次进度), PauseReasonDialog | `pfa_run_state` + OTel |
| **回滚面板** | `/coc/rollback` | RollbackWizard, RollbackHistoryTable | `feature_version_history` + `pfa_run_state` (rollback 方向) |
| **审计查询** | `/coc/audit` | AuditFilter, AuditResultTable | `coc_audit_view` (视图) |

## 7.2 与 VIZ 渲染能力的复用点

- **依赖图页面** (`/coc/dependencies`) **直接复用** RGS-BAS-021 §4 的无限画布渲染器（FR-COC-003），仅在数据源上叠加 PFAU 状态图层（"正在升级"边用橙色、"已回滚"边用灰色）
- **事件流订阅关系图** **复用** VIZ 的"控制流/数据流"边样式（FR-VIZ-010），但边颜色按"消费健康"映射（绿/黄/红）
- **不**复用 VIZ 的"无限画布首页"——COC UI 的功能矩阵首页是表格形式（更适合运营人员快速浏览），**不**是画布形式

## 7.3 不在 COC UI 范围

- **不**含"账号管控/服务器管控/告警"等既有 GM 后台功能——属既有页面，**不**改造
- **不**含"GM 对单个玩家的操作"——属 RGS-REQ-007 既有功能
- **不**含"插件脚本编辑"——沙箱脚本编辑属 RGS-BAS-005 §6 既有功能，**不**在 COC UI 重复实现

# 8. RBAC 角色矩阵扩展

沿用 RGS-BAS-001 §7.3 RBAC 角色矩阵（既有的 `viewer`/`operator`/`admin` 等），新增两个角色：

| 角色 | 读权限 | 写权限 | 适用 |
|---|---|---|---|
| `cluster_operator` | 功能矩阵/事件流/依赖图/灰度面板/回滚面板/审计查询 (只读) | plug/unplug (单 Feature)、灰度批次调整 (单 Feature) | 业务方运维 / SRE 初级 |
| `cluster_admin` | 全部 COC 页面 | 全部 COC 写操作（含按 Feature 回滚、批量升级、DLQ 重放） | SRE 高级 / 架构师 |

**与既有 GM 后台 RBAC 的关系**：
- `cluster_operator` 是既有 `operator` 角色的**子集**（仅含 COC 相关权限），不继承 `operator` 的"账号管控"等权限
- `cluster_admin` 是既有 `admin` 角色的**超集**（含既有 `admin` 的全部权限 + COC 高危操作），**不**改变既有 `admin` 的语义
- 新角色**必须**经架构评审通过，登记至 RGS-REQ-001 §7 角色矩阵

# 9. 与既有 ARC-018／021／042／019／039 的强制联动点

| 联动对象 | 联动点 | 实施位置 | 验证方式 |
|---|---|---|---|
| ARC-018 新挂载 | 新限界上下文挂载完成时, 自动在 `feature_registry` 创建"限界上下文型"Feature (FR-INT-001) | RGS-BAS-002 §4 脚手架检查清单追加"自动调用 ClusterOpsService.RegisterFeature"步骤 | ARC-018 Mount Record 含 COC UI 可读元数据字段 |
| ARC-021 插件注册 | 新插件注册时, 自动在 `feature_registry` 创建"插件型"Feature (FR-INT-002) | RGS-BAS-005 §3 插件注册表追加"自动调用 ClusterOpsService.RegisterFeature"步骤 | CI 校验: 插件注册表行数 == feature_registry.feature_type=PLUGIN 行数 |
| ARC-042 集群部署 | 集群级部署执行时, 为每个被部署 Feature 创建 PFAU 实例 (FR-INT-003) | RGS-BAS-024 §4 编排状态机追加"为被部署 Feature 启动 PFAU"步骤 | ARC-042 一次 run 的 App 部署完成 == 对应 Feature PFAU 进入 active |
| ARC-019 GM 后台 | COC UI 写操作全部经 AdminService 转发 (FR-COC-020) | RGS-BAS-003 §3（既有 AdminService 扩展模式）；COC 转发方法字段级定义待 RGS-DTL-031 | 渗透测试: COC UI 凭证不持有 K8s/DB 直连凭证 |
| ARC-039 VIZ 只读边界 | COC UI 是写操作控制台, **不**作为 VIZ 的子页面; 但复用 VIZ 渲染能力 (FR-COC-003) | RGS-BAS-021 §4 渲染器抽离为独立组件库, COC UI 引用 | 静态检查: VIZ 路由不出现 COC 写操作入口 |

## 9.1 与 ARC-018 挂载检查清单的强制联动

RGS-BAS-002 §4 挂载脚手架检查清单追加:

> **【新增检查项 - FR-INT-001】** 新限界上下文挂载完成前, **必须**已调用 `ClusterOpsService.RegisterFeature` 在 `feature_registry` 中创建对应 Feature 记录 (feature_type=BOUNDED_CONTEXT), 否则 CI 校验失败, 挂载**不得**视为完成。

> **【新增检查项 - FR-INT-002】** 新插件注册完成前, **必须**已调用 `ClusterOpsService.RegisterFeature` 在 `feature_registry` 中创建对应 Feature 记录 (feature_type=PLUGIN), 否则 CI 校验失败。

## 9.2 与 ARC-042 编排层的强制联动

RGS-BAS-024 §4 编排状态机追加:

> **【新增联动点 - FR-INT-003】** 编排层在 Helm Release 调用 SUCCEEDED 后, **必须**调用 `ClusterOpsService.NotifyFeatureDeployed(feature_id, version)` 触发对应 Feature 的 PFAU 实例从 `declared` → `canary_in_progress` → `canary_confirmed` → `completed`。**不**直接修改 `feature_registry.current_version`。

# 10. 非功能设计落地

## 10.1 性能设计

- **功能矩阵首页** (NFR-COC-001 p95<2s) — 服务端预聚合: 每 5 秒在 `admin_db` 物化视图 `feature_matrix_view` (按 `feature_type`/`status` 预分组), COC UI 仅查询物化视图
- **事件流视图** (NFR-COC-002 p95<3s) — 复用 ARC-017 既有可观测性聚合查询, COC UI 走"分析管线"读端点 (ARC-035 物理隔离精神)
- **CEM 探针** (FR-CEM-030) — 批量 UPSERT (5秒窗口) 避免 `admin_db` 写入热点; 探针解析事件后丢弃 payload, 不进入事件内容处理路径

## 10.2 可用性设计

- **COC UI 不可用时 PFAU 可见性** (NFR-COC-004) — 降级到 `AdminService` 的 gRPC 查询 API (`GetPfaRunState`/`ListFeatures`), 运营人员可通过 CLI 调用
- **COC UI 自身可回滚** (NFR-COC-010 30 分钟) — 沿用 NFR-AV-007 既有滚动更新机制, COC UI 自身回滚**不**影响 PFAU 进行中实例 (实例元数据持久化于 `admin_db`)

## 10.3 隔离性设计

- **COC UI 数据源查询** (NFR-COC-005) — 全部走读优化路径:
  - `feature_registry`/`feature_version_history`/`pfa_run_state`/`event_schema_registry` → 物化视图 + 索引
  - 事件流/订阅关系 → ARC-017 既有可观测性读端点
  - **不**直接查询生产 App 的业务 DB
- **CEM 探针不阻塞正常消费者** (FR-API-012) — 独立 Consumer Group, 走"只读镜像"链路

## 10.4 审计设计

- 全部 COC UI 写操作 (含 plug/unplug/升级/回滚/灰度批次调整/DLQ 重放) 经 `AdminService` 转发时, **必须**在 `operation_audit` 追加一条记录, `action_type` 字段按 §6.2 协议枚举 (FR-COC-040)
- 审计保留期沿用 NFR-OPS-005 (3 年, 不可篡改)

### 10.5 本功能日志设计（非功能设计综合）

本节覆盖 §10.1 性能 + §10.2 可用性 + §10.3 隔离性 + §10.4 审计的运行时事件观察点。审计写层事件已在 BAS-003 §7.1 统一设计（`audit.write.*`），本节仅补充 COC 域特有的"非功能降级/隔离触发"信号。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `coc.nfr.slo_budget_exceeded` | SRE 定义的 COC 操作 SLO 预算（如 PFAU 端到端 P99 < 30s）本月已耗尽（如 95%）触发告警 | 偶发（运维超载） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 告警事件） | 含 `slo_name` / `budget_remaining_pct` / `current_month_consumed_pct` / `operator_id`（如本月有触发动作）；约 320B/条 |
| `coc.nfr.circuit_breaker_opened` | COC UI → AdminService → ClusterOpsService 任一链路熔断器打开（per §10.2 可用性） | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 错误路径） | 含 `downstream` / `error_rate` / `opened_at`；约 280B/条 |
| `coc.nfr.isolation_boundary_violated` | COC 域操作意外触达其他限界上下文（per §10.3 隔离性，NetworkPolicy 拦截） | 极少（配置错） | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 安全告警） | 含 `attempted_target` / `expected_boundary` / `source_session` / `request_id`；约 320B/条 |
| `coc.nfr.perf_degradation_detected` | COC UI 操作 P99 超过阈值（如 > 5s）持续 5min | 极少 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 告警事件） | 含 `operation` / `p99_latency_ms` / `threshold_ms` / `duration_min`；约 280B/条 |
| `coc.nfr.debug.coc_metrics_dump` | COC 域全部 Prometheus 指标完整 dump（运维排障用） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-8KB/条（指标数量决定，release 剔除） |

**debug-only 守护要点**：
- `coc.nfr.isolation_boundary_violated` 是**安全告警**——release 必出，便于按 `source_session` 维度识别异常访问模式
- `coc.nfr.debug.coc_metrics_dump` 涉及全部 COC 域指标，release 完全剔除避免 RUST_LOG=debug 误开时撑爆

---

# 11. 风险与未决事项

| ID | 内容 | 处理阶段 | 关联 |
|---|---|---|---|
| TBD-COC-001 | COC UI 的"无限画布"前端选型: 复用 VIZ 既有选型还是独立选型? 需评估复用是否带来"读操作与写操作耦合"的 UX 问题 | 详细设计阶段前 | RGS-REQ-031 §13 |
| TBD-COC-002 | PFAU 的"补丁型Feature"是否需要"金丝雀测试"作为强制门禁? 金丝雀测试的判定标准 (错误率阈值、p99 延迟阈值) 与既有 NFR-PE-* 是否一致? | PH-7 前 | RGS-REQ-031 §13 |
| TBD-COC-003 | COC UI 的"批量回滚"上限 20 个 Feature 是否合理? 超出时的拆批策略与单批回滚时长约束 | 详细设计阶段 | RGS-REQ-031 §13 |
| TBD-COC-004 | CEM 的"事件注册变更"事件 (`coc.event_registry_changed`) 是否需要保留为系统级事件, 与"应用级事件"做 Schema 版本管理上的区分 | 详细设计阶段 | FR-API-010 |
| TBD-COC-005 | `feature_registry` 表分区策略: 按 `feature_type` 还是按 `owner_team`? 数据增长后 (Feature 数 > 1000) 的归档策略 | PH-7 前 | §3.1 |
| TBD-COC-006 | `pfa_run_state` 表历史归档: 状态机进入终态 (completed/rolled_back/failed) 超过 90 天的 PFAU 实例是否归档到低成本存储? | PH-7 前 | §3.3 |
| RSK-COC-001 | CI 校验脚本 `scripts/check-cem-coverage.sh` 须新增, 定期扫描各 App Publisher 调用点比对 `event_registry` | PH-7 前持续跟踪 | RGS-REQ-031 RSK-COC-001 |
| RSK-COC-002 | PFAU 超时阈值 120 秒可能在弱网/高负载下频繁触发, 须 PH-7 前根据实测调整 | PH-7 前 | RGS-REQ-031 RSK-COC-002 |
| RSK-COC-003 | COC UI UX 设计不当可能让 SRE 倾向"通过 COC UI 执行所有操作", 弱化 RGS-OPS-001 既有手顺书价值; 缓解: COC UI 写操作面板**必须**保留"查看对应 RGS-OPS-001 章节"链接 | 持续跟踪 | RGS-REQ-031 RSK-COC-003 |
| RSK-COC-004 | CEM 可重放历史 7+90 天可能引入新存储成本与合规风险 (GDPR/数据本地化); 缓解: 详细设计阶段评估存储成本, 提供可配置 retention | 详细设计阶段 | RGS-REQ-031 RSK-COC-004 |
| ISS-092 | COC UI 尚未做 OLU 运维负荷核算, 须追加登记至附件 D ISS 列表 (与 ISS-065 同类) | PH-7 前 | RGS-REQ-031 §13 |

---

> 本文档配套 RGS-REQ-031 需求定义书与 RGS-ADR-0051 架构决定，完成 ARC-051 的基本设计落地。RGS-DTL-031 v0.1 草案已形成，下一步是 ClusterOpsService 内部模块接口、探针订阅器、gRPC 客户端 SDK 的字段级映射、具名 DD Review 与实现前 Gate；三份测试设计书 RGS-TST-UT-31/RGS-TST-IT-31/RGS-TST-ST-31 仍须同步补充 DTL 映射并取得执行证据。
