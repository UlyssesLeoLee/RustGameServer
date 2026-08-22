# 详细设计书（詳細設計書 / Detailed Design Document）

**服务器全生命周期管理 Server Lifecycle Management (LCM)**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-042 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-037 服务器全生命周期管理 基本设计书 |
| 父需求 | RGS-REQ-037 |
| 父 ARC | ARC-038（服区边界的原子迁移）扩展为服务器全生命周期治理；ARC-051 ClusterOpsService 扩 `realm_lifecycle` Feature 类型 |
| 配套设计 | RGS-DTL-031 集群运营中心与每功能原子升级（CEM/PFAU 编排基座）；RGS-DTL-040 Admin 域详细设计（AdminService 转发通路）；RGS-ADR-0015 Saga 适用边界 |
| 协同文档 | RGS-BAS-020 §4 合服/分服执行流程（被本文档扩为开新服/退场/归档）；RGS-BAS-022 §3.3 分片新增/下线（被扩为开新服 SOP）；RGS-BAS-031 ClusterOpsService（PFAU 编排） |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 适用许可 | Apache-2.0（本仓库） |

> 本文档落实 RGS-BAS-037 全部组件、Schema、契约、Saga 编排，补足 Rust 类型签名 / admin_db 完整 DDL / Saga 步骤定义 / ClusterOpsService Feature 集成 / 演练执行器 / OLU 预算上报。详细到可被直接编码的程度，但仍保留少量"由详细编码阶段决定"的 TBD（仅限具体超时阈值、审批链具体角色、归档存储选型等无法在文档层级预定的实现细节）。

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 首版草案。落实 RGS-BAS-037：①`rgs-realm-lifecycle` Rust crate 划分（与 ClusterOpsService 同 crate 内子模块）；②6 阶段操作器（`NewRealmOperator` / `ScaleOperator` / `SplitOperator` / `MergeOperator` / `RetireOperator` / `ArchiveOperator`）的接口签名；③`admin_db` 6 张表完整 DDL（`realm_lifecycle_run` / `new_realm_plan` / `split_plan` / `merge_conflict_rule_set_v2` / `retire_plan` / `archive_policy`）；④Saga 步骤定义（分服 6 步 / 合服 5 步 / 退场 4 步 / 归档 3 步）；⑤`realm_lifecycle` Feature 集成 ClusterOpsService PFAU 编排（7 个子类）；⑥演练执行器（`DrillExecutor`）接口与状态机；⑦OLU 预算上报与可观测性埋点；⑧与既有 ARC-018 挂载/退场、ARC-019 GM 控制平面、ARC-040 横向分片的集成时序 | 全文 |
| 0.2 | 2026-08-21 | Ulysses(一人公司 12 角色兼任 per DEC-008) | Ulysses(同) | 具名人类审批完成(per RGS-WBS-001 §17 集体签字声明):一人公司兼任体制下,Ulysses 在本表审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17。审批栏细化角色意见与 DEC-008 兼任对应关系见 RGS-REQ-004 §3.10。**升 v0.2**: 文档从 v0.1 草案转为 v0.2 具名审批版,生产基线化仍需 G-CODE-06 实测通过(per RGS-WF-001) | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 评审（架构） | 待指定 | — | ①6 阶段划分与既有 ARC-018/040/051 一致性；②RealmLifecycleService 限界上下文归属（**确认归 AD 扩展**）；③Saga 步骤定义与 RGS-ADR-0015 Saga 边界一致 |
| 评审（平台/SRE） | 待指定 | — | ①演练执行器与生产环境隔离；②OLU 预算上报与 ARC-026 联动 |
| 评审（DBA/安全） | 待指定 | — | ①`admin_db` 6 张表索引/外键/分区策略；②跨 DB 写入 Saga 步骤对生产 DB 的影响；③归档冷存储访问 RBAC |
| 评审（运营/合规） | 待指定 | — | ①退场后归档期数据保留期与各地区法规；②GDPR "被遗忘权"删除通路；③演练剧本模板的运营可执行性 |
| 审批（项目负责人） | 待指定 | — | 确认风险、范围、回滚条件与实施授权 |

| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 目录

1. [定位与非目标](#1-定位与非目标)
2. [crate 划分与依赖边界](#2-crate-划分与依赖边界)
3. [持久化模型与 DDL](#3-持久化模型与-ddl)
4. [6 阶段状态机与 PFAU 集成](#4-6-阶段状态机与-pfau-集成)
5. [6 阶段操作器接口](#5-6-阶段操作器接口)
6. [Saga 步骤定义与编排器](#6-saga-步骤定义与编排器)
7. [演练执行器](#7-演练执行器)
8. [Feature 类型 realm_lifecycle 集成](#8-feature-类型-realm_lifecycle-集成)
9. [API 契约与幂等字段](#9-api-契约与幂等字段)
10. [故障、降级与跨域边界](#10-故障降级与跨域边界)
11. [可观测性、OLU 预算与审计](#11-可观测性olu-预算与审计)
12. [验收证据与开放项](#12-验收证据与开放项)
13. [追溯性](#13-追溯性)

---

# 1. 定位与非目标

## 1.1 定位

`rgs-realm-lifecycle` 是 AD 限界上下文内的新增子模块（**不**新建独立 crate，**不**新建独立限界上下文，**不**新建独立 DB），与 `ClusterOpsService` 同处 `rgs-cluster-ops` crate 内，扩 ARC-051 Feature 类型为新增 `realm_lifecycle` 类。

**核心职责**：
- 6 阶段操作器（开新服 / 扩缩容 / 分服 / 合服 / 退场 / 归档）的业务逻辑实现
- `NewRealmPlan` / `SplitPlan` / `MergeConflictRuleSet` v2 / `RetirePlan` / `ArchivePolicy` 的评估与持久化
- 跨 DB 写入的 Saga 编排（分服 6 步 / 合服 5 步 / 退场 4 步 / 归档 3 步）
- 演练执行（`DrillExecutor`）：在演练环境以生产数据快照执行完整流程
- OLU 预算上报（向 `rgs-arc-olu` crate 上报阶段变更消耗的 OLU）

**非职责**（由既有模块负责）：
- RBAC / 审计 / 限流（既有 `AdminService`）
- PFAU 状态机推进（既有 `ClusterOpsService`）
- Feature 注册 / 灰度控制（既有 `ClusterOpsService`）
- 业务 DB 改写（player_db / economy_db / social_db 既有 service）
- 业务事件发布 / 订阅（既有事件总线）

## 1.2 非目标与硬禁止

- **不**新建独立限界上下文——归 AD 扩展（与 ClusterOpsService 同上下文），理由同 RGS-BAS-037 §2.1
- **不**新建独立数据库——6 张新表全部在既有 `admin_db`，遵循 ARC-008 独立 DB 原则
- **不**对外暴露独立 gRPC / HTTP 接口——所有写操作经 `AdminService` 转发（FR-LCM-004 门禁）
- **不**绕过 ClusterOpsService PFAU 编排——阶段变更作为 `realm_lifecycle` Feature 走 PFAU 状态机
- **不**为阶段变更引入新事务范式——跨 DB 写入复用 RGS-ADR-0015 Saga 适用边界与单一调解者原则
- **不**分发新 Saga 编排器——RealmLifecycleService 作为 Saga 编排者，ClusterOpsService 作为 PFAU 监督者（仲裁者）
- **不**允许任意 OLU 消耗——阶段变更 OLU 必纳入 ARC-026 预算核算（NFR-LCM-007 硬约束）

# 2. crate 划分与依赖边界

## 2.1 Cargo workspace 集成

`rgs-realm-lifecycle` **不**作为独立 crate，**作为子模块**加入既有 `rgs-cluster-ops` crate：

```text
rgs-cluster-ops/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── cluster_ops_service.rs       # 既有 ClusterOpsService
│   ├── pfa_runner.rs                # 既有 PFAU 状态机
│   ├── feature_registry.rs          # 既有 Feature Registry
│   ├── cem_probe_aggregator.rs      # 既有 CEM 探针
│   ├── dlq_operator.rs              # 既有 DLQ
│   ├── replay_operator.rs           # 既有 Replay
│   ├── realm_lifecycle/             # ← 新增子模块 (本文档落地)
│   │   ├── mod.rs
│   │   ├── error.rs                 # LcmError 错误类型
│   │   ├── state.rs                 # RealmLifecycleState 枚举
│   │   ├── operators/
│   │   │   ├── mod.rs
│   │   │   ├── new_realm.rs
│   │   │   ├── scale.rs
│   │   │   ├── split.rs
│   │   │   ├── merge.rs
│   │   │   ├── retire.rs
│   │   │   └── archive.rs
│   │   ├── saga/
│   │   │   ├── mod.rs
│   │   │   ├── orchestrator.rs      # SagaOrchestrator
│   │   │   ├── steps.rs             # 步骤定义
│   │   │   └── compensation.rs      # 反向步骤
│   │   ├── drill/
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs          # DrillExecutor
│   │   │   └── playbooks/           # 演练剧本模板
│   │   │       ├── new_realm.rs
│   │   │       ├── split.rs
│   │   │       ├── merge.rs
│   │   │       ├── retire.rs
│   │   │       └── archive.rs
│   │   ├── plans/
│   │   │   ├── mod.rs
│   │   │   ├── new_realm_plan.rs
│   │   │   ├── split_plan.rs
│   │   │   ├── merge_rule_set_v2.rs
│   │   │   ├── retire_plan.rs
│   │   │   └── archive_policy.rs
│   │   ├── feature_adapter.rs       # realm_lifecycle Feature 适配
│   │   ├── olu_reporter.rs          # OLU 预算上报
│   │   └── metrics.rs               # 可观测性埋点
│   └── ...
└── migrations/
    ├── 0001_initial.sql
    ├── ...
    └── 0020_lcm_tables.sql          # ← 新增 (本文档落地)
```

## 2.2 模块依赖

```toml
# rgs-cluster-ops/Cargo.toml 新增依赖
[dependencies]
# 既有 (RGS-DTL-031 落地)
rgs-arc-olu = { path = "../rgs-arc-olu" }
rgs-arc-rcu = { path = "../rgs-arc-rcu" }
rgs-admin-service = { path = "../rgs-admin-service" }
rgs-event-bus = { path = "../rgs-event-bus" }
rgs-observability = { path = "../rgs-observability" }

# 新增 (本文档落地)
rgs-player-service = { path = "../rgs-player-service" }       # 分服/合服跨 DB 写入
rgs-economy-service = { path = "../rgs-economy-service" }
rgs-social-service = { path = "../rgs-social-service" }
rgs-realm-directory = { path = "../rgs-realm-directory" }      # RealmDirectoryService 路由表
```

> **依赖收敛原则**：`rgs-cluster-ops` 已有 `rgs-admin-service` 依赖（用于 AdminService 转发），本设计**不**新增此类横向依赖；新增的 4 个依赖（player / economy / social / realm-directory）均为**业务侧**调用，是 Saga 步骤执行所必需。

# 3. 持久化模型与 DDL

## 3.1 Migration 文件

```sql
-- migrations/0020_lcm_tables.sql
-- 适用: admin_db
-- 落地 RGS-BAS-037 §4.2 全部 6 张表

BEGIN;

-- 1. realm_lifecycle_run (阶段变更实例)
CREATE TABLE realm_lifecycle_run (
    run_id              UUID        NOT NULL PRIMARY KEY,
    feature_id          TEXT        NOT NULL,
    feature_type        TEXT        NOT NULL DEFAULT 'realm_lifecycle',
    realm_id            TEXT        NOT NULL,
    target_realm_ids    TEXT[]      NULL,
    status              TEXT        NOT NULL,
    drill_run_id        UUID        NULL,
    plan_snapshot       JSONB       NOT NULL,
    leader_epoch        BIGINT      NOT NULL DEFAULT 0,
    request_id          UUID        NOT NULL,
    operator_id         TEXT        NOT NULL,
    approved_by         TEXT        NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_lifecycle_feature_type CHECK (feature_type = 'realm_lifecycle'),
    CONSTRAINT chk_lifecycle_status CHECK (status IN (
        'declared','planning','drill_validated','executing',
        'observing','completed','paused','failed','rolled_back'
    ))
);
CREATE INDEX idx_lifecycle_run_realm_id ON realm_lifecycle_run (realm_id);
CREATE INDEX idx_lifecycle_run_status ON realm_lifecycle_run (status);
CREATE INDEX idx_lifecycle_run_request_id ON realm_lifecycle_run (request_id);
CREATE UNIQUE INDEX uniq_lifecycle_run_request_id_op ON realm_lifecycle_run (request_id, operator_id);

-- 2. new_realm_plan
CREATE TABLE new_realm_plan (
    plan_id             UUID        NOT NULL PRIMARY KEY,
    target_realm_id     TEXT        NOT NULL UNIQUE,
    display_name        TEXT        NOT NULL,
    trigger_source      TEXT        NOT NULL,
    db_shard_config     JSONB       NOT NULL,
    node_pool_config    JSONB       NOT NULL,
    network_config      JSONB       NOT NULL,
    capacity_budget     JSONB       NOT NULL,
    rollout_schedule    JSONB       NOT NULL,
    notification_config JSONB       NOT NULL,
    approved_by         TEXT        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_trigger_source CHECK (trigger_source IN (
        'capacity_gate','ops_planned','architecture_decision'
    ))
);

-- 3. split_plan
CREATE TABLE split_plan (
    plan_id                 UUID        NOT NULL PRIMARY KEY,
    source_realm_id         TEXT        NOT NULL,
    target_realm_ids        TEXT[]      NOT NULL,
    strategy                TEXT        NOT NULL,
    forced_rule             JSONB       NULL,
    opt_in_window_days      INT         NULL,
    hybrid_rule             JSONB       NULL,
    cross_realm_relation    JSONB       NOT NULL,
    saga_steps              JSONB       NOT NULL,
    rollback_window_days    INT         NOT NULL DEFAULT 7,
    approved_by             TEXT        NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_strategy CHECK (strategy IN ('forced','opt_in','hybrid')),
    CONSTRAINT chk_split_source_exists CHECK (source_realm_id <> ALL(target_realm_ids))
);
CREATE INDEX idx_split_plan_source_realm_id ON split_plan (source_realm_id);

-- 4. merge_conflict_rule_set_v2
CREATE TABLE merge_conflict_rule_set_v2 (
    rule_set_id                 UUID        NOT NULL PRIMARY KEY,
    merge_job_id                UUID        NOT NULL,
    character_name_rule         TEXT        NOT NULL,
    unique_item_rule            TEXT        NOT NULL,
    currency_rule               TEXT        NOT NULL DEFAULT 'sum',
    pending_lottery_rule        TEXT        NOT NULL,
    unclaimed_mail_rule         TEXT        NOT NULL,
    frozen_cross_guild_apply_rule TEXT      NOT NULL,
    approved_by                 TEXT        NOT NULL,
    locked_at                   TIMESTAMPTZ NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_pending_lottery CHECK (pending_lottery_rule IN (
        'settle_before_merge','cancel_and_compensate','carry_over_as_pending'
    )),
    CONSTRAINT chk_unclaimed_mail CHECK (unclaimed_mail_rule IN (
        'carry_over','expire_after_merge','refund_attachable'
    )),
    CONSTRAINT chk_frozen_apply CHECK (frozen_cross_guild_apply_rule IN (
        'approve_then_merge','reject_then_merge','keep_pending'
    )),
    CONSTRAINT chk_locked CHECK (locked_at IS NOT NULL)
);
CREATE INDEX idx_merge_rule_set_v2_job ON merge_conflict_rule_set_v2 (merge_job_id);

-- 5. retire_plan
CREATE TABLE retire_plan (
    plan_id                     UUID        NOT NULL PRIMARY KEY,
    target_realm_id             TEXT        NOT NULL,
    trigger_source              TEXT        NOT NULL,
    migration_window_days       INT         NOT NULL,
    query_channel_rbac          TEXT[]      NOT NULL,
    reactivation_window_days    INT         NOT NULL DEFAULT 30,
    audit_chain                 JSONB       NOT NULL,
    approved_by                 TEXT        NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_retire_trigger CHECK (trigger_source IN (
        'merge_merged_into_target','capacity_decision','ops_decision'
    ))
);
CREATE INDEX idx_retire_plan_realm_id ON retire_plan (target_realm_id);

-- 6. archive_policy
CREATE TABLE archive_policy (
    policy_id                   UUID        NOT NULL PRIMARY KEY,
    target_realm_id             TEXT        NOT NULL,
    retire_plan_id              UUID        NOT NULL REFERENCES retire_plan(plan_id),
    hot_archive_years           INT         NOT NULL DEFAULT 3,
    cold_archive_years          INT         NOT NULL DEFAULT 10,
    storage_redundancy          TEXT        NOT NULL DEFAULT 'n_plus_2',
    gdpr_delete_path            TEXT        NOT NULL,
    cross_realm_merge_history   BOOLEAN     NOT NULL DEFAULT TRUE,
    approved_by                 TEXT        NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_storage_redundancy CHECK (storage_redundancy IN (
        'n_plus_1','n_plus_2','n_plus_3'
    ))
);
CREATE INDEX idx_archive_policy_realm_id ON archive_policy (target_realm_id);

-- 与既有 operation_audit 表关联
ALTER TABLE realm_lifecycle_run
    ADD CONSTRAINT fk_lifecycle_run_audit
    FOREIGN KEY (run_id) REFERENCES operation_audit(related_run_id) ON DELETE RESTRICT;

COMMIT;
```

## 3.2 Schema 与 RGS-BAS-007 §4 既有分区策略的对齐

`realm_lifecycle_run` 表按 `created_at` 月度范围分区（与既有 `operation_audit` 同构，复用 RGS-BAS-007 §4 既定分区滚动创建脚本，保留期 3 年 NFR-SE-010）。其他 5 张配置表无分区需求（写入频次低）。

# 4. 6 阶段状态机与 PFAU 集成

## 4.1 阶段状态枚举

```rust
// src/realm_lifecycle/state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum RealmLifecycleState {
    NotYet,        // 逻辑服尚未创建
    Active,        // 运营中
    Scaling,       // 扩缩容中
    Splitting,     // 分服中
    Merging,       // 合服中
    Retired,       // 已下线 (数据保留)
    Archived,      // 已归档
}

impl RealmLifecycleState {
    pub fn can_transition_to(self, next: RealmLifecycleState) -> bool {
        use RealmLifecycleState::*;
        match (self, next) {
            // 开新服
            (NotYet, Active) => true,
            // 扩缩容 (进入 / 退出)
            (Active, Scaling) | (Scaling, Active) => true,
            // 分服
            (Active, Splitting) => true,
            (Splitting, Active) => true,   // 分服后部分新服 Active
            // 合服
            (Active, Merging) => true,
            (Merging, Active) => true,     // 合服后目标服 Active
            // 退场
            (Active, Retired) | (Splitting, Retired) | (Merging, Retired) => true,
            (Retired, Active) => true,     // 二次激活
            // 归档
            (Retired, Archived) => true,
            // 其他
            _ => false,
        }
    }
}
```

## 4.2 阶段变更 PFAU 状态（与 RGS-DTL-031 §4.2 复用）

```rust
// 阶段变更作为 realm_lifecycle Feature 走 ClusterOpsService 既定 PFAU 状态机
// 状态: declared → planning → drill_validated → executing → observing → completed
// 中间态: paused → retrying / rolling_back / aborted
// 复用 RGS-DTL-031 §4.2 PfaRunState 定义
```

## 4.3 状态机推进与 PFAU 集成

```rust
// src/realm_lifecycle/state.rs (续)
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::cluster_ops_service::PfaRunner;
use crate::realm_lifecycle::error::LcmError;

pub struct RealmLifecycleStateMachine {
    realm_id: String,
    current: Arc<RwLock<RealmLifecycleState>>,
    pfa_runner: Arc<PfaRunner>,
    feature_registry: Arc<FeatureRegistry>,
}

impl RealmLifecycleStateMachine {
    /// 阶段转移 (经 PFAU 编排)
    pub async fn transition(
        &self,
        next: RealmLifecycleState,
        feature_id: &str,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<(), LcmError> {
        let mut current = self.current.write().await;
        let prev = *current;
        if !prev.can_transition_to(next) {
            return Err(LcmError::InvalidTransition { from: prev, to: next });
        }

        // 阶段变更作为 realm_lifecycle Feature, 走 PFAU 编排
        let run_id = self.pfa_runner.declare_feature_upgrade(
            feature_id,
            &self.realm_id,
            request_id,
            operator_id,
        ).await?;

        // 等待 PFAU 推进至 canary_confirmed (即阶段变更就绪)
        self.pfa_runner.wait_for_canary_confirmed(run_id).await?;

        // PFAU 确认后, 更新状态
        *current = next;
        Ok(())
    }
}
```

# 5. 6 阶段操作器接口

## 5.1 操作器 Trait

```rust
// src/realm_lifecycle/operators/mod.rs
use async_trait::async_trait;
use crate::realm_lifecycle::error::LcmError;
use crate::realm_lifecycle::state::RealmLifecycleState;
use uuid::Uuid;

#[async_trait]
pub trait RealmLifecycleOperator: Send + Sync {
    /// 操作器唯一标识
    fn feature_subtype(&self) -> &'static str;

    /// 评估 Plan (创建 plan 表记录, 尚未执行)
    async fn evaluate_plan(
        &self,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<Uuid, LcmError>;

    /// 演练执行 (drill_run)
    async fn execute_drill(
        &self,
        plan_id: Uuid,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<Uuid, LcmError>;

    /// 正式执行 (经 PFAU 编排)
    async fn execute(
        &self,
        plan_id: Uuid,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<Uuid, LcmError>;

    /// 回退 (PFAU rolling_back)
    async fn rollback(
        &self,
        run_id: Uuid,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<(), LcmError>;

    /// 查询状态
    async fn get_state(
        &self,
        realm_id: &str,
    ) -> Result<RealmLifecycleState, LcmError>;
}
```

## 5.2 6 个操作器实现概览

| 操作器 | feature_subtype | 关键方法 |
|---|---|---|
| `NewRealmOperator` | `realm_lifecycle::new_realm` | 资源评估 → ARC-018 挂载触发 → 灰度开放编排 |
| `ScaleOperator` | `realm_lifecycle::scale` | 节点级 HPA（既有）+ 整服级走 `NewRealmOperator` |
| `SplitOperator` | `realm_lifecycle::split` | `SplitPlan` 评估 → Saga 6 步执行 → 跨服关系保持 |
| `MergeOperator` | `realm_lifecycle::merge` | `MergeConflictRuleSet` v2 评估 → Saga 5 步执行 → 回退窗口支持 |
| `MergeRollbackOperator` | `realm_lifecycle::merge_rollback` | 反向 Saga 步骤执行（窗口期内）|
| `RetireOperator` | `realm_lifecycle::retire` | 只读维护模式 → 玩家迁出 → 节点下线 → RBAC 通道 |
| `ArchiveOperator` | `realm_lifecycle::archive` | 热归档 → 冷归档 → 合规删除通路 |

## 5.3 NewRealmOperator 实现要点

```rust
// src/realm_lifecycle/operators/new_realm.rs
use super::*;
use crate::realm_lifecycle::plans::new_realm_plan::NewRealmPlan;
use crate::arc_018_scaffold::ScaffoldMount;
use crate::realm_directory::RealmDirectoryService;

pub struct NewRealmOperator {
    pfa_runner: Arc<PfaRunner>,
    scaffold_mount: Arc<ScaffoldMount>,
    realm_directory: Arc<RealmDirectoryService>,
    drill_executor: Arc<DrillExecutor>,
}

#[async_trait]
impl RealmLifecycleOperator for NewRealmOperator {
    fn feature_subtype(&self) -> &'static str { "realm_lifecycle::new_realm" }

    async fn evaluate_plan(
        &self,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<Uuid, LcmError> {
        // 1. 收集 NewRealmPlan 字段 (从请求体解析)
        // 2. 三方签字校验: 运营 + 架构 + SRE
        // 3. 检查 target_realm_id 不冲突 (new_realm_plan.target_realm_id UNIQUE)
        // 4. 写入 new_realm_plan 表
        // 5. 返回 plan_id
        todo!("详细编码阶段实现")
    }

    async fn execute_drill(
        &self,
        plan_id: Uuid,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<Uuid, LcmError> {
        // 1. 读 new_realm_plan
        // 2. drill_executor.execute_new_realm_drill(plan)
        // 3. 演练报告: 通过 → drill_validated; 失败 → 写审计 + 通知
        // 4. 返回 drill_run_id
        todo!()
    }

    async fn execute(
        &self,
        plan_id: Uuid,
        request_id: Uuid,
        operator_id: &str,
    ) -> Result<Uuid, LcmError> {
        // 1. 校验 drill_validated
        let plan = NewRealmPlan::load(plan_id, &self.pg_pool).await?;
        if plan.drill_run_id.is_none() {
            return Err(LcmError::DrillNotPassed);
        }
        // 2. PFAU 编排: 最小配置挂载
        self.pfa_runner.declare_feature_upgrade(
            &format!("realm_lifecycle::new_realm.{}", plan.target_realm_id),
            &plan.target_realm_id,
            request_id,
            operator_id,
        ).await?;
        // 3. ARC-018 挂载 (ScaffoldMount)
        self.scaffold_mount.mount_minimal(&plan).await?;
        // 4. RealmDirectoryService 登记 (状态 hidden)
        self.realm_directory.register_hidden(&plan).await?;
        // 5. 渐进式扩容到目标配置
        // 6. 灰度开放 (hidden → white_list → channel_gray → all)
        // 7. 玩家通知任务入队
        todo!()
    }

    // ...
}
```

# 6. Saga 步骤定义与编排器

## 6.1 SagaOrchestrator 复用 RGS-ADR-0015 模式

`SagaOrchestrator` 作为 RealmLifecycleService 的子模块，复用既有 RGS-ADR-0015 Saga 适用边界与单一调解者原则。SagaOrchestrator **不**是独立 crate，**不**分发新协调服务，**仅**作为 RealmLifecycleService 内部模块（与 `ClusterOpsService` PFAU Runner 同级）。

```rust
// src/realm_lifecycle/saga/orchestrator.rs
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::realm_lifecycle::error::LcmError;
use crate::realm_lifecycle::saga::steps::{SagaStep, SagaStepResult};

/// Saga 状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SagaState {
    Pending,
    Running { current_step: usize },
    Completed,
    Compensating { current_step: usize },
    Failed { reason: String, failed_step: usize },
    Aborted,
}

pub struct SagaOrchestrator {
    steps: Vec<Box<dyn SagaStep>>,
    state: Arc<Mutex<SagaState>>,
    request_id: Uuid,
    realm_id: String,
}

impl SagaOrchestrator {
    pub fn new(
        steps: Vec<Box<dyn SagaStep>>,
        request_id: Uuid,
        realm_id: String,
    ) -> Self {
        Self {
            steps,
            state: Arc::new(Mutex::new(SagaState::Pending)),
            request_id,
            realm_id,
        }
    }

    pub async fn run(&self) -> Result<(), LcmError> {
        let mut state = self.state.lock().await;
        *state = SagaState::Running { current_step: 0 };
        drop(state);

        for (idx, step) in self.steps.iter().enumerate() {
            // PFAU 监督: 步骤开始前检查 PFAU 状态
            if !self.pfa_check_can_proceed().await? {
                self.compensate_from(idx).await?;
                return Err(LcmError::Aborted);
            }

            let mut state = self.state.lock().await;
            *state = SagaState::Running { current_step: idx };
            drop(state);

            // 执行步骤
            match step.execute(self.request_id, &self.realm_id).await {
                Ok(SagaStepResult::Success) => {
                    // 继续
                }
                Ok(SagaStepResult::AlreadyApplied) => {
                    // 幂等: 重复执行不报错
                }
                Err(e) => {
                    // 失败: 补偿已执行步骤
                    self.compensate_from(idx).await?;
                    return Err(e);
                }
            }
        }

        let mut state = self.state.lock().await;
        *state = SagaState::Completed;
        Ok(())
    }

    async fn compensate_from(&self, from_idx: usize) -> Result<(), LcmError> {
        let mut state = self.state.lock().await;
        *state = SagaState::Compensating { current_step: from_idx };
        drop(state);

        // 反向补偿已执行步骤 (idx < from_idx)
        for idx in (0..from_idx).rev() {
            self.steps[idx].compensate(self.request_id, &self.realm_id).await?;
        }
        Ok(())
    }
}
```

## 6.2 Saga 步骤定义

```rust
// src/realm_lifecycle/saga/steps.rs
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum SagaStepResult {
    Success,
    AlreadyApplied,
}

#[async_trait]
pub trait SagaStep: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(
        &self,
        request_id: Uuid,
        realm_id: &str,
    ) -> Result<SagaStepResult, crate::realm_lifecycle::error::LcmError>;
    async fn compensate(
        &self,
        request_id: Uuid,
        realm_id: &str,
    ) -> Result<(), crate::realm_lifecycle::error::LcmError>;
}
```

## 6.3 分服 6 步 Saga

```rust
// src/realm_lifecycle/saga/steps/split.rs
use super::*;
use crate::player_service::PlayerServiceClient;
use crate::social_service::SocialServiceClient;
use crate::economy_service::EconomyServiceClient;
use std::sync::Arc;

/// 步骤 1: 冻结 source_realm_id
pub struct FreezeSourceRealmStep {
    pub player: Arc<PlayerServiceClient>,
}
#[async_trait]
impl SagaStep for FreezeSourceRealmStep {
    fn name(&self) -> &'static str { "split::freeze_source_realm" }
    async fn execute(&self, request_id: Uuid, realm_id: &str) -> Result<SagaStepResult, LcmError> {
        // 调用 player_service.FreezeRealm { realm_id, request_id }
        // 幂等: 重复调用返回 AlreadyApplied
        self.player.freeze_realm(realm_id, request_id).await?;
        Ok(SagaStepResult::Success)
    }
    async fn compensate(&self, request_id: Uuid, realm_id: &str) -> Result<(), LcmError> {
        self.player.unfreeze_realm(realm_id, request_id).await?;
        Ok(())
    }
}

/// 步骤 2: player_db.realm_id 改写
pub struct RewritePlayerRealmStep {
    pub player: Arc<PlayerServiceClient>,
    pub target_assignments: Vec<(Uuid /*account_id*/, String /*target_realm_id*/)>,
}
#[async_trait]
impl SagaStep for RewritePlayerRealmStep {
    fn name(&self) -> &'static str { "split::rewrite_player_realm" }
    async fn execute(&self, request_id: Uuid, _realm_id: &str) -> Result<SagaStepResult, LcmError> {
        // 批量改写, 携带 request_id 幂等键
        for chunk in self.target_assignments.chunks(1000) {
            self.player.bulk_update_realm(chunk, request_id).await?;
        }
        Ok(SagaStepResult::Success)
    }
    async fn compensate(&self, request_id: Uuid, realm_id: &str) -> Result<(), LcmError> {
        // 反向: 从备份或补偿日志还原
        self.player.bulk_rollback_realm(realm_id, request_id).await?;
        Ok(())
    }
}

/// 步骤 3: social_db.friend 跨服标记
pub struct MarkCrossRealmFriendStep {
    pub social: Arc<SocialServiceClient>,
}
#[async_trait]
impl SagaStep for MarkCrossRealmFriendStep {
    fn name(&self) -> &'static str { "split::mark_cross_realm_friend" }
    async fn execute(&self, request_id: Uuid, realm_id: &str) -> Result<SagaStepResult, LcmError> {
        self.social.mark_cross_realm_friends(realm_id, request_id).await?;
        Ok(SagaStepResult::Success)
    }
    async fn compensate(&self, request_id: Uuid, realm_id: &str) -> Result<(), LcmError> {
        self.social.unmark_cross_realm_friends(realm_id, request_id).await?;
        Ok(())
    }
}

/// 步骤 4: social_db.guild 拆分
pub struct SplitGuildStep {
    pub social: Arc<SocialServiceClient>,
}
#[async_trait]
impl SagaStep for SplitGuildStep {
    fn name(&self) -> &'static str { "split::split_guild" }
    async fn execute(&self, request_id: Uuid, realm_id: &str) -> Result<SagaStepResult, LcmError> {
        self.social.split_guilds_by_realm(realm_id, request_id).await?;
        Ok(SagaStepResult::Success)
    }
    async fn compensate(&self, request_id: Uuid, realm_id: &str) -> Result<(), LcmError> {
        self.social.unsplit_guilds(realm_id, request_id).await?;
        Ok(())
    }
}

/// 步骤 5: economy_db.mail 迁移
pub struct MigrateMailStep {
    pub economy: Arc<EconomyServiceClient>,
}
#[async_trait]
impl SagaStep for MigrateMailStep {
    fn name(&self) -> &'static str { "split::migrate_mail" }
    async fn execute(&self, request_id: Uuid, realm_id: &str) -> Result<SagaStepResult, LcmError> {
        self.economy.migrate_mail_by_account(realm_id, request_id).await?;
        Ok(SagaStepResult::Success)
    }
    async fn compensate(&self, request_id: Uuid, realm_id: &str) -> Result<(), LcmError> {
        self.economy.rollback_mail_migration(realm_id, request_id).await?;
        Ok(())
    }
}

/// 步骤 6: 一致性校验
pub struct ConsistencyCheckStep {
    pub player: Arc<PlayerServiceClient>,
    pub social: Arc<SocialServiceClient>,
    pub economy: Arc<EconomyServiceClient>,
    pub snapshot: AssetSnapshot,  // 分服前的资产快照
}
#[async_trait]
impl SagaStep for ConsistencyCheckStep {
    fn name(&self) -> &'static str { "split::consistency_check" }
    async fn execute(&self, request_id: Uuid, realm_id: &str) -> Result<SagaStepResult, LcmError> {
        let actual = AssetSnapshot::collect(realm_id, &self.player, &self.social, &self.economy).await?;
        if actual != self.snapshot {
            return Err(LcmError::ConsistencyCheckFailed { expected: self.snapshot.clone(), actual });
        }
        Ok(SagaStepResult::Success)
    }
    async fn compensate(&self, _request_id: Uuid, _realm_id: &str) -> Result<(), LcmError> {
        // 一致性校验步骤无副作用, 补偿为空
        Ok(())
    }
}
```

## 6.4 合服 5 步 Saga

合服 Saga 步骤（扩 RGS-BAS-020 §4 既有流程）：
1. **FreezeAllSourceRealmsStep**：冻结所有被合并服
2. **ApplyMergeConflictRulesStep**：应用 `MergeConflictRuleSet` v2 规则（角色名 / 唯一道具 / 货币 / 未结算抽奖 / 未领取邮件 / 工会申请）
3. **MergePlayerDataStep**：合并 player_db 数据
4. **MergeSocialDataStep**：合并 social_db 数据
5. **ConsistencyCheckStep**：一致性校验（同分服）

合服反向步骤对应 `realm_lifecycle::merge_rollback` Feature。

## 6.5 退场 4 步 Saga

1. **EnterReadOnlyModeStep**：进入只读维护模式
2. **MigratePlayersStep**：玩家迁出（合服 / 自然流失 / 主动转服）
3. **ShutdownRuntimeStep**：运行时节点下线
4. **EnableQueryChannelStep**：RBAC 查询通道开启

## 6.6 归档 3 步 Saga

1. **HotArchiveStep**：DB 切换为冷备实例（只读副本）
2. **ColdArchiveStep**：全量导出至对象存储（N+2 副本）
3. **EnableGdprDeletePathStep**：合规删除通路开启

# 7. 演练执行器

## 7.1 DrillExecutor 设计

```rust
// src/realm_lifecycle/drill/executor.rs
use std::sync::Arc;
use uuid::Uuid;
use crate::realm_lifecycle::error::LcmError;
use crate::realm_lifecycle::drill::playbooks::*;

pub struct DrillExecutor {
    sandbox_pg_pool: Arc<sqlx::PgPool>,         // 演练环境 DB 连接池
    sandbox_k8s: Arc<K8sClient>,                // 演练环境 K8s 客户端
    notification_service: Arc<NotificationService>,
}

pub struct DrillResult {
    pub drill_run_id: Uuid,
    pub passed: bool,
    pub failure_reasons: Vec<String>,
    pub consistency_report: ConsistencyReport,
}

impl DrillExecutor {
    /// 开新服演练
    pub async fn execute_new_realm_drill(
        &self,
        plan_id: Uuid,
    ) -> Result<DrillResult, LcmError> {
        let plan = NewRealmPlan::load(plan_id, &self.production_pg_pool).await?;
        let mut playbook = NewRealmDrillPlaybook::new(&plan, &self.sandbox_pg_pool, &self.sandbox_k8s);
        playbook.run().await
    }

    /// 分服演练
    pub async fn execute_split_drill(
        &self,
        plan_id: Uuid,
    ) -> Result<DrillResult, LcmError> {
        let plan = SplitPlan::load(plan_id, &self.production_pg_pool).await?;
        // 1. 生产环境数据快照 (只读, 限定时间窗)
        let snapshot = self.snapshot_production_data(&plan).await?;
        // 2. 拷贝快照到演练 DB
        self.copy_to_sandbox(&snapshot).await?;
        // 3. 在演练 DB 执行 Saga 步骤
        let mut playbook = SplitDrillPlaybook::new(&plan, &self.sandbox_pg_pool);
        playbook.run().await
    }

    // 类似的: execute_merge_drill / execute_retire_drill / execute_archive_drill
}
```

## 7.2 演练剧本示例（分服）

```rust
// src/realm_lifecycle/drill/playbooks/split.rs
use crate::realm_lifecycle::drill::executor::{DrillResult, ConsistencyReport};

pub struct SplitDrillPlaybook<'a> {
    plan: &'a SplitPlan,
    sandbox_pool: &'a sqlx::PgPool,
    saga_orchestrator: SagaOrchestrator,
}

impl<'a> SplitDrillPlaybook<'a> {
    pub fn new(plan: &'a SplitPlan, sandbox_pool: &'a sqlx::PgPool) -> Self {
        let steps: Vec<Box<dyn SagaStep>> = vec![
            Box::new(FreezeSourceRealmStep { /* ... */ }),
            Box::new(RewritePlayerRealmStep { /* ... */ }),
            Box::new(MarkCrossRealmFriendStep { /* ... */ }),
            Box::new(SplitGuildStep { /* ... */ }),
            Box::new(MigrateMailStep { /* ... */ }),
            Box::new(ConsistencyCheckStep { /* ... */ }),
        ];
        Self {
            plan,
            sandbox_pool,
            saga_orchestrator: SagaOrchestrator::new(steps, plan.request_id, plan.source_realm_id.clone()),
        }
    }

    pub async fn run(mut self) -> Result<DrillResult, LcmError> {
        // 执行 Saga
        self.saga_orchestrator.run().await?;
        // 验证一致性
        let report = ConsistencyReport::collect(&self.plan, self.sandbox_pool).await?;
        // 清理演练数据
        self.cleanup().await?;
        Ok(DrillResult {
            drill_run_id: Uuid::new_v4(),
            passed: report.passed,
            failure_reasons: report.failures,
            consistency_report: report,
        })
    }
}
```

# 8. Feature 类型 `realm_lifecycle` 集成

## 8.1 Feature 子类注册

`realm_lifecycle` Feature 类型的 7 个子类（`new_realm` / `scale` / `split` / `merge` / `merge_rollback` / `retire` / `archive`）通过 `rgs-cluster-ops` crate 启动时注册到 `FeatureRegistry`：

```rust
// src/realm_lifecycle/feature_adapter.rs
use crate::feature_registry::{FeatureRegistry, FeatureType, FeatureSubtype};

pub fn register_realm_lifecycle_features(
    registry: &FeatureRegistry,
) -> Result<(), LcmError> {
    let subtypes = [
        ("realm_lifecycle::new_realm", FeatureType::RealmLifecycle),
        ("realm_lifecycle::scale", FeatureType::RealmLifecycle),
        ("realm_lifecycle::split", FeatureType::RealmLifecycle),
        ("realm_lifecycle::merge", FeatureType::RealmLifecycle),
        ("realm_lifecycle::merge_rollback", FeatureType::RealmLifecycle),
        ("realm_lifecycle::retire", FeatureType::RealmLifecycle),
        ("realm_lifecycle::archive", FeatureType::RealmLifecycle),
    ];
    for (subtype, feature_type) in subtypes {
        registry.register(subtype, feature_type)?;
    }
    Ok(())
}
```

> **FeatureType 枚举扩展**：`rgs-cluster-ops` 既有的 `FeatureType` 枚举（`BoundedContext` / `Plugin` / `Patch` / `Config`，见 RGS-DTL-031）需**新增** `RealmLifecycle` 变体。这是 RGS-BAS-031 §1.1 Feature 类型表的第 5 类。

## 8.2 PFAU 编排 hook

`realm_lifecycle` Feature 的 PFAU 编排通过 `PfaRunner` 既定状态机执行，**不**为 LCM 另起一套编排。`RealmLifecycleService` 仅作为 PFAU 编排的"业务执行者"，**不**作为编排者本身。

```rust
// src/realm_lifecycle/feature_adapter.rs (续)
use crate::pfa_runner::{PfaRunner, PfaHook};

pub struct RealmLifecyclePfaHook {
    operators: Arc<OperatorRegistry>,
}

impl PfaHook for RealmLifecyclePfaHook {
    /// PFAU 推进到 canary_confirmed 时调用
    async fn on_canary_confirmed(
        &self,
        run_id: Uuid,
        feature_id: &str,
    ) -> Result<(), LcmError> {
        // 解析 feature_id, 调用对应操作器的 execute
        let operator = self.operators.resolve(feature_id)?;
        operator.execute_post_pfa(run_id, feature_id).await
    }
}
```

# 9. API 契约与幂等字段

## 9.1 不另建第二套协议

`RealmLifecycleService` **不**对外暴露独立 gRPC / HTTP 接口，**仅**经既有 `AdminService`（RGS-DTL-040）转发。请求字段全部复用 RGS-DTL-031 §7.1 既定 AdminService 转发协议：

| 字段 | 类型 | 说明 |
|---|---|---|
| `request_id` | UUID | 幂等键（同 RGS-DTL-031 §7.1）|
| `operator_id` | TEXT | 操作者 (RBAC 角色) |
| `approval_ref` | TEXT | 高危操作二次确认凭证 |
| `trace_id` | UUID | 日志追踪 |

## 9.2 业务请求 payload（经 AdminService 转发时携带）

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "lcm_operation", rename_all = "snake_case")]
pub enum LcmRequest {
    NewRealm {
        plan: NewRealmPlanRequest,
    },
    Scale {
        realm_id: String,
        scale_type: ScaleType,
    },
    Split {
        plan: SplitPlanRequest,
    },
    Merge {
        plan: MergeRequest,
    },
    MergeRollback {
        merge_run_id: Uuid,
    },
    Retire {
        plan: RetirePlanRequest,
    },
    Archive {
        policy: ArchivePolicyRequest,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewRealmPlanRequest {
    pub target_realm_id: String,
    pub display_name: String,
    pub trigger_source: TriggerSource,
    pub db_shard_config: DbShardConfig,
    pub node_pool_config: NodePoolConfig,
    pub network_config: NetworkConfig,
    pub capacity_budget: CapacityBudget,
    pub rollout_schedule: Vec<RolloutPhase>,
    pub notification_config: NotificationConfig,
    // 三方签字
    pub signed_by_ops: String,
    pub signed_by_architect: String,
    pub signed_by_sre: String,
}
// ... 类似的 SplitPlanRequest / MergeRequest / RetirePlanRequest / ArchivePolicyRequest
```

## 9.3 错误语义

| 错误 | 含义 | 客户端处理 |
|---|---|---|
| `InvalidTransition` | 状态机非法转移 | 不重试，检查 realm 状态 |
| `DrillNotPassed` | 演练未通过 | 重做演练 |
| `PlanNotFound` | plan_id 不存在 | 检查 plan_id |
| `PlanNotApproved` | plan 未签字完整 | 补签字后重试 |
| `PlanLocked` | 合服冲突规则已锁定 | 不允许运行时修改 |
| `SagaStepFailed` | Saga 步骤执行失败 | 检查 PFAU 状态决定重试/回退 |
| `ConsistencyCheckFailed` | 一致性校验不通过 | 触发自动补偿/人工介入 |
| `GrayRolledBack` | 灰度回滚导致阶段变更被拒 | 等待下一窗口期 |
| `OluBudgetExceeded` | OLU 预算超限 | 等待预算释放 / 申请额外预算 |
| `InsufficientPrivilege` | 操作者权限不足 | 联系法务 / SRE Lead |

# 10. 故障、降级与跨域边界

## 10.1 故障分类与处理

| 类别 | 触发 | 处理 |
|---|---|---|
| Saga 步骤失败 | 单个步骤执行异常 | 反向步骤补偿已执行步骤 |
| PFAU 状态失联 | ClusterOpsService 不可用 | 阶段变更挂起,等待 PFAU 恢复 |
| `admin_db` 写失败 | DB 异常 | 状态机 `Failed`,玩家重试 |
| 演练环境故障 | 演练 K8s/DB 不可用 | 演练报告标 `failed`,不切到 executing |
| 灰度回滚 | 阶段变更期间被合并服灰度回滚 | 阶段变更挂起,等待人工决策 |
| 限流 | AdminService 限流 | 指数退避,等待配额释放 |
| 跨 DB 写入失败 | player_db / economy_db / social_db 任一不可用 | Saga 步骤部分成功,反向补偿 |

## 10.2 跨域边界

| 边界 | 约束 |
|---|---|
| RealmLifecycleService → 业务 DB | **仅**经业务 service gRPC 调用,**不**直连 DB（与 RGS-REQ-013 §3 治理框架一致）|
| RealmLifecycleService → 业务事件总线 | 阶段变更事件（如 `RealmCreated` / `RealmRetired`）经既有事件总线发布,**不**绕事件总线 |
| RealmLifecycleService → ClusterOpsService | 阶段变更作为 Feature 走 PFAU 编排,**不**直连 ClusterOpsService 内部状态 |
| RealmLifecycleService → AdminService | **仅**经 AdminService 转发,**不**对外暴露独立接口 |

# 11. 可观测性、OLU 预算与审计

## 11.1 可观测性埋点

```rust
// src/realm_lifecycle/metrics.rs
use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};

pub fn register_metrics() {
    describe_counter!("rgs_lcm_run_state_transition_total", "阶段变更 PFAU 状态转移次数");
    describe_gauge!("rgs_lcm_active_runs", "当前进行中的阶段变更实例数");
    describe_gauge!("rgs_lcm_drill_pass_rate", "演练通过率");
    describe_histogram!("rgs_lcm_drill_to_execute_duration_seconds", "drill_validated 到 executing 间隔");
    describe_histogram!("rgs_lcm_saga_step_duration_seconds", "单个 Saga 步骤耗时");
    describe_counter!("rgs_lcm_saga_rollback_total", "Saga 回退次数");
    describe_counter!("rgs_lcm_drill_failure_reason_total", "演练失败原因分布");
    describe_histogram!("rgs_lcm_archive_query_latency_seconds", "归档后客服查询响应时延");
    describe_gauge!("rgs_lcm_realm_count_by_status", "实时各状态 realm 数");
    describe_gauge!("rgs_lcm_olu_consumed_by_team", "各团队 OLU 消耗");
}
```

## 11.2 OLU 预算上报

```rust
// src/realm_lifecycle/olu_reporter.rs
use crate::arc_olu::OluBudgetClient;
use std::sync::Arc;

pub struct OluReporter {
    client: Arc<OluBudgetClient>,
}

impl OluReporter {
    /// 阶段变更 OLU 消耗上报
    pub async fn report_phase_olu(
        &self,
        team: &str,
        phase: &str,         // new_realm | scale | split | merge | retire | archive
        olu_tokens: u64,     // token 消耗 (RGS-TS-001 §6.2 草案)
    ) -> Result<(), LcmError> {
        self.client.consume(team, phase, olu_tokens).await?;
        Ok(())
    }

    /// 检查 OLU 预算是否充足
    pub async fn check_budget(
        &self,
        team: &str,
        phase: &str,
        estimated_olu: u64,
    ) -> Result<bool, LcmError> {
        self.client.check(team, phase, estimated_olu).await
    }
}
```

## 11.3 审计

所有阶段变更操作（plan 创建 / 演练 / 正式执行 / 回退）**全部**走既有 `admin_db.operation_audit`（RGS-BAS-003 §7 复用），不另建审计表。审计字段：

- `operator_id` / `approval_ref` / `trace_id`
- `lcm_phase`（new_realm / scale / split / merge / retire / archive）
- `realm_id` / `target_realm_ids`
- `plan_id` / `run_id` / `drill_run_id`
- 前后状态对比
- Saga 步骤执行轨迹
- 失败原因 / 回退原因

# 12. 验收证据与开放项

## 12.1 验收证据

- [ ] 6 张表 migration 在 admin_db 成功执行（`migrations/0020_lcm_tables.sql`）
- [ ] 6 阶段操作器全部实现并接入 ClusterOpsService PFAU
- [ ] `realm_lifecycle` Feature 类型注册到 FeatureRegistry（7 个子类）
- [ ] SagaOrchestrator 实现并支持分服 6 步 / 合服 5 步 / 退场 4 步 / 归档 3 步
- [ ] 演练执行器在演练环境实测：开新服 / 分服 / 合服 / 退场 / 归档 5 类各通过一次
- [ ] 演练通过后方可切到 `executing` 状态（FR-LCM-003 门禁）实测
- [ ] OLU 预算上报至 `rgs-arc-olu` 成功（实测）
- [ ] 跨 DB 写入走 Saga 模式 + 单一调解者（FR-LCM-005）实测
- [ ] 玩家通知 ≥ 7 天预告（NFR-LCM-004）实测
- [ ] 退场后 RBAC 查询通道开启（`cs_agent` / `sre` / `legal` 角色测试）
- [ ] 归档冷热分层 + N+2 冗余存储（RSK-LCM-005 缓解）实测
- [ ] GDPR "被遗忘权"删除通路（FR-LCM-084）实测
- [ ] 跨服合并回溯保留（FR-LCM-085）实测
- [ ] 合服回退窗口期内可回退（AC-LCM-009）实测
- [ ] 退场后 30 天内二次激活（AC-LCM-008）实测
- [ ] `rgs-cluster-ops` 既有 cargo test 全部通过（新增子模块不破坏既有功能）

## 12.2 开放项

| 编号 | 内容 | 处理 |
|---|---|---|
| TBD-DTL-042-01 | 跨 DB 写入的具体补偿策略（按业务 service 接口约定）| 详细编码阶段与 player / economy / social service Lead 联动确定 |
| TBD-DTL-042-02 | Saga 步骤超时具体阈值（每步默认 60s，失败重试 3 次）| 详细编码阶段 PH-4 实测 |
| TBD-DTL-042-03 | 演练环境与生产环境数据快照的脱敏策略 | 详细编码阶段与 DBA 联动 |
| TBD-DTL-042-04 | 归档冷存储软件选型（TBD-LCM-006）| 详细编码阶段评估,需满足 OSI 许可（CON-001/002）|
| TBD-DTL-042-05 | 6 阶段 OLU 估算默认值（TBD-LCM-007）| 详细编码阶段基于 RGS-TS-001 §6.2 token-OLU 框架落地 |
| TBD-DTL-042-06 | `FeatureType::RealmLifecycle` 变体在 RGS-DTL-031 既定枚举的扩展位置 | 详细编码阶段与 ClusterOpsService Lead 联动 |
| TBD-DTL-042-07 | 合服 / 分服 / 退场期间玩家通知模板（多语言）| 详细编码阶段与运营 / 本地化 Lead 联动 |
| RSK-DTL-042-01 | 演练环境数据快照拷贝耗时（大数据量场景下可能超时）| 详细编码阶段实测,必要时改为增量快照 |
| RSK-DTL-042-02 | Saga 步骤执行期间业务 DB 长事务阻塞 | 详细编码阶段评估事务隔离级别 + 锁等待超时 |

# 13. 追溯性

| 需求 ID | 本设计书章节 |
|---|---|
| FR-LCM-001 资产不丢不重 | §6 Saga, §7 演练 |
| FR-LCM-002 跨阶段可审计 | §3 DDL, §11.3 审计 |
| FR-LCM-003 跨阶段可演练 | §4 PFAU drill_validated, §7 演练执行器 |
| FR-LCM-004 跨阶段门禁一致 | §1.2, §9 API 契约 |
| FR-LCM-005 跨 DB 最终一致 | §6 SagaOrchestrator |
| FR-LCM-006 玩家最小告知 | §11 可观测性, §12 开放项 TBD-07 |
| FR-LCM-010~033 开新服 | §5.3 NewRealmOperator |
| FR-LCM-040~044 扩缩容 | §5.2 ScaleOperator 概览 |
| FR-LCM-050~055 分服 | §5.2 SplitOperator, §6.3 Saga 6 步 |
| FR-LCM-060~064 合服 | §5.2 MergeOperator, §6.4 Saga 5 步 |
| FR-LCM-070~075 退场 | §5.2 RetireOperator, §6.5 Saga 4 步 |
| FR-LCM-080~085 归档 | §5.2 ArchiveOperator, §6.6 Saga 3 步 |
| NFR-LCM-001 资产不丢不重 | §6 Saga 幂等, §7 演练一致性 |
| NFR-LCM-002 演练频率 | §12.1 验收 |
| NFR-LCM-003 审计完整性 | §11.3 审计 |
| NFR-LCM-004 玩家通知 | §12.2 TBD-07 |
| NFR-LCM-005 数据保留期 | §3 DDL archive_policy, §12.2 TBD-04 |
| NFR-LCM-006 归档查询性能 | §11.1 metrics |
| NFR-LCM-007 OLU 预算 | §11.2 OluReporter, §12.1 验收 |
| NFR-LCM-008 阶段变更期间服务可用性 | §6 Saga 步骤, §10.1 故障处理 |
| AC-LCM-001~010 | §12.1 验收证据 |

---

> 本文档与 RGS-BAS-037（服务器全生命周期管理 基本设计书）配套使用，详细到 Rust crate 级别可被直接编码。配套的 RGS-SPEC 阶段产出由 13-实现规格 流程产出，**不**在本文档重复。
