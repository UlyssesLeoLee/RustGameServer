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
3. [`admin_db` 新增 Schema](#3-admin_db-新增-schema)
4. [Feature 元数据与 PFAU 状态机](#4-feature-元数据与-pfau-状态机)
5. [CEM 探针订阅器设计](#5-cem-探针订阅器设计)
6. [API 契约字段级定义](#6-api-契约字段级定义)
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
| ARC-019 GM 后台 | COC UI 写操作全部经 AdminService 转发 (FR-COC-020) | RGS-BAS-003 §6.3.4 AdminService 扩展新增转发方法 | 渗透测试: COC UI 凭证不持有 K8s/DB 直连凭证 |
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

> 本文档配套 RGS-REQ-031 需求定义书与 RGS-ADR-0051 架构决定, 完成 ARC-051 全部功能需求到设计的落地。后续将产出 RGS-DTL-031 详细设计 (ClusterOpsService 内部模块接口、探针订阅器实现细节、gRPC 客户端 SDK) 与三份测试设计书 RGS-TST-UT-31/RGS-TST-IT-31/RGS-TST-ST-31。
