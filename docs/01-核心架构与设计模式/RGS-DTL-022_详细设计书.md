# 详细设计书（詳細設計書 / Detailed Design Document）

**弹性容量规划与超大规模并发架构：分片路由参数具体化・弹性预留调度算法・插件分片同步协议详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-022 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-022 弹性容量规划与超大规模并发架构 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定。细化RGS-BAS-022§2.3三类组件扩容手段判定表为可执行的评审判定伪代码、§3.2跨分片能力清单为具体的能力注册与校验协议格式、§4.1弹性预留为具体调度算法、§5.1插件注册表分片维度扩展为具体DDL与生命周期状态机代码、§5.2跨节点同步机制为具体协议格式与规模验证算法。**本版本不覆盖**：T3多区域拓扑触发后的具体设计（RGS-BAS-022明确留给RGS-BAS-017§2.3另行评审）、TBD-CAP-001弹性预留系数的最终校准值。见§7 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 弹性预留调度算法是否与既有K8s `readinessGate`机制的实际语义一致，插件分片状态同步协议是否真正复用RGS-BAS-005§5既有机制而非另起一套 |
| 评审（容量规划） | | | 部署时长基准（RGS-BAS-024§9A）与本文档弹性预留触发时机是否存在互相矛盾的假设 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [三类组件扩容手段的判定协议](#2-三类组件扩容手段的判定协议)
3. [跨分片能力注册与校验协议格式](#3-跨分片能力注册与校验协议格式)
4. [弹性预留调度算法详细设计](#4-弹性预留调度算法详细设计)
5. [插件分片维度扩展物理设计](#5-插件分片维度扩展物理设计)
6. [跨节点同步协议与规模验证算法](#6-跨节点同步协议与规模验证算法)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-022给出了容量分级拓扑的组件清单、三类扩容手段的判定原则（文字表格）、弹性预留与预测性预热的设计要点（文字流程）、插件注册表分片维度扩展的字段清单。本文档将其落实为：评审/CI可直接执行的判定伪代码、组件间实际传输的协议格式、调度算法的完整伪代码（含边界条件），使实现人员与评审人员均可直接依据本文档判定合规性，无需再对RGS-BAS-022的文字表格做二次解读。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-022已确定的任何结构性选择（三类扩容手段互不替代、分片路由完全复用RGS-BAS-020、跨分片能力"实时状态不允许/元数据允许"的判定标准、插件热插拔在全容量级别可用）。若细化过程中发现基本设计本身有缺陷，修正应回写RGS-BAS-022，不在本文档内悄悄改写。
- 不覆盖T2→T3触发后的多区域拓扑具体设计——RGS-BAS-022§2.1已明确该部分"触发时另走RGS-BAS-017§2.3评审，非本文档预先设计"，本文档同样不越权。
- 不给出TBD-CAP-001（弹性预留系数）、TBD-CAP-002的最终数值——本文档给出可编程的判定框架与参数占位，具体数值提案见§4.1（标注为初始提案，非最终值，同RGS-DTL-025/026已确立的先例）。

### 1.3 记述规则

沿用RGS-DTL-001§1.3/RGS-DTL-002§1.3已确立的记述规则：协议格式以Protobuf/HTTP风格给出；算法伪代码可直接对应Rust `Result`实现；DDL以PostgreSQL为准。

---

## 2. 三类组件扩容手段的判定协议

对应RGS-BAS-022§2.3判定表与"判定优先级"文字规则。落实为代码评审CI阶段可调用的静态判定函数，防止§2.3"评审时若发现某扩容方案跨越了这一对应关系，必须驳回"这一人工评审要求缺乏可复核的客观依据。

```rust
enum ComponentKind { StatelessService, StatefulSceneRuntime, DatabaseLayer }

enum ScalingAction {
    IncreaseReplicaCount,          // 无状态服务：唯一合法手段
    AddNewSceneInstance,           // 有状态场景：唯一合法手段
    SplitSingleSceneSimulation,    // 违规：拆分单场景模拟到多机(违反ARC-001)
    AddReadReplica,                // 数据库层：第一优先手段
    AddPartition,                  // 数据库层：第二优先手段
    SplitWithinShard,              // 数据库层：仅在前两者确已不足时允许
}

// 对应§2.3"判定优先级"文字规则的可执行版本，CI代码评审检查清单(§6.3)调用
fn validate_scaling_plan(kind: ComponentKind, action: ScalingAction) -> Result<(), CapacityViolation> {
    let allowed = match kind {
        ComponentKind::StatelessService => matches!(action, ScalingAction::IncreaseReplicaCount),
        ComponentKind::StatefulSceneRuntime => matches!(action, ScalingAction::AddNewSceneInstance),
        ComponentKind::DatabaseLayer => matches!(
            action,
            ScalingAction::AddReadReplica | ScalingAction::AddPartition | ScalingAction::SplitWithinShard
        ),
    };
    if !allowed {
        return Err(CapacityViolation::CrossKindScalingAttempt { kind, action });
    }
    // 数据库层内部仍有优先级：SplitWithinShard须先证明AddReadReplica/AddPartition已用尽(§2.3"先用足既有手段")，
    // 该证明本身(负载数据/试验报告)不属于本函数校验范围,由§6.1检查清单的人工核对项承接
    Ok(())
}
```

`SplitSingleSceneSimulation`这一变体本身即是"永不允许"的具体化——枚举中显式列出它，是为了让静态分析/代码评审工具能够对"尝试拆分单场景模拟"这一常见误用模式做模式匹配检测（对应RGS-BAS-022§6.3"场景运行时扩容代码未尝试拆分单场景模拟到多机"检查项），而非仅停留在人工审查描述层面。

---

## 3. 跨分片能力注册与校验协议格式

对应RGS-BAS-022§3.2跨分片能力清单与"判定规则"。落实为能力注册的具体协议格式，供`RealmDirectoryService`（RGS-BAS-020§3既有组件，本文档不重复设计，仅新增字段）校验：

```protobuf
// CrossShardCapabilityDeclaration：新增跨分片能力时提交评审的声明,对应§3.2"须先按此规则判定"
message CrossShardCapabilityDeclaration {
  string capability_name        = 1;
  CapabilityCategory category    = 2;   // 见下方枚举，对应§3.2判定规则的两分类
  string consistency_note         = 3;  // category=METADATA时必填：说明滞后容忍度
  string review_ticket_ref          = 4; // 评审记录引用(GM后台运维工单同类机制,复用RGS-BAS-022§4.2已引用的既有工单能力)
}

enum CapabilityCategory {
  UNSPECIFIED       = 0;
  REALTIME_PLAYER_STATE = 1;   // 玩家实时游玩状态：永不允许跨分片，声明此类别的请求在校验阶段直接拒绝
  ACCOUNT_OR_GOVERNANCE_METADATA = 2;   // 账号/治理层面元数据：允许，但仍须逐项评审(review_ticket_ref必填)
}
```

```rust
// 对应§3.2"判定规则"，CI/评审工具调用，防止逐案自由裁量导致标准漂移
fn validate_capability_declaration(decl: &CrossShardCapabilityDeclaration) -> Result<(), CapabilityViolation> {
    match decl.category {
        CapabilityCategory::RealtimePlayerState => {
            // §3.2表格明确列出的4类实时状态(位置/战斗/背包等)一律拒绝,不接受任何例外声明
            Err(CapabilityViolation::RealtimeStateCannotCrossShard)
        }
        CapabilityCategory::AccountOrGovernanceMetadata => {
            if decl.review_ticket_ref.is_empty() {
                return Err(CapabilityViolation::MissingReviewReference);
            }
            Ok(())
        }
        CapabilityCategory::Unspecified => Err(CapabilityViolation::CategoryNotDeclared),
    }
}
```

全局排行榜聚合、账号身份跨分片唯一、客服工单/支付对账三项既有能力（RGS-BAS-022§3.2表格已批准）在部署时以`CrossShardCapabilityDeclaration`形式预注册（`review_ticket_ref`指向RGS-BAS-022本身的批准记录），使新增能力与既有能力走同一套声明式校验路径，不区分"历史遗留白名单"与"新增审批"两套机制。

---

## 4. 弹性预留调度算法详细设计

对应RGS-BAS-022§4.1弹性预留文字描述，落实为具体调度伪代码。

### 4.1 预留系数与readinessGate状态机

```rust
// TBD-CAP-001初始提案：预留系数20%，与RGS-BAS-022§4.1原文示例值一致，PH-4实测前非最终值(同RGS-DTL-025§5同类做法)
const RESERVE_RATIO_DEFAULT: f64 = 0.20;

enum ReservePodState {
    Standby,     // 已启动、已就绪，readinessGate=false，不进入流量池
    Absorbing,   // 冲击发生，readinessGate切换为true，已进入流量池承接流量
    Returning,   // HPA新副本已补足目标数，预留Pod正在交还为Standby
}

// 冲击检测触发时调用,对应§4.1"预留余量立即可用(无需等待Pod启动耗时)"
fn absorb_traffic_spike(reserve_pods: &mut [ReservePod], spike_magnitude: f64) -> Result<(), CapacityError> {
    let needed = (spike_magnitude * current_target_replicas() * RESERVE_RATIO_DEFAULT).ceil() as usize;
    let available: Vec<&mut ReservePod> = reserve_pods.iter_mut()
        .filter(|p| p.state == ReservePodState::Standby)
        .take(needed)
        .collect();
    for pod in available {
        set_readiness_gate(pod.id, true);   // 复用既有K8s readinessGate机制,立即计入流量池,不新增编排组件
        pod.state = ReservePodState::Absorbing;
    }
    // 同时触发HPA正常扩容路径(§4.1"HPA随后逐步补足新的目标副本数")，两者并行，不互斥
    trigger_hpa_scale_up(spike_magnitude)?;
    Ok(())
}

// HPA新副本Ready后调用,对应§4.1"预留余量的角色随后交还"
fn return_reserve_pods(reserve_pods: &mut [ReservePod], hpa_new_replicas_ready: usize) {
    let mut returned = 0;
    for pod in reserve_pods.iter_mut() {
        if pod.state == ReservePodState::Absorbing && returned < hpa_new_replicas_ready {
            pod.state = ReservePodState::Returning;
            set_readiness_gate(pod.id, false);   // 退出流量池
            pod.state = ReservePodState::Standby; // 交还完成,恢复待命
            returned += 1;
        }
    }
    // 若hpa_new_replicas_ready不足以覆盖全部Absorbing状态Pod,剩余部分继续Absorbing直到下一轮补足检查,
    // 不强制立即全部交还(避免HPA尚未完全补足时突然抽走承接能力造成二次冲击)
}
```

### 4.2 预测性预热调度（对应RGS-BAS-022§4.2文字流程）

```rust
// GM后台运维工单登记预热计划后，预热调度器按提前量触发
fn evaluate_prewarm_schedule(plans: &[PrewarmPlan], now: Instant, lead_time: Duration) -> Vec<PrewarmTrigger> {
    plans.iter()
        .filter(|p| p.event_time - now <= lead_time && p.event_time > now && !p.triggered)
        .map(|p| PrewarmTrigger {
            plan_id: p.plan_id,
            target_replica_multiplier: p.expected_load_multiplier,
            // 复用既有HPA配置的手动覆盖能力(临时提高目标副本数下限),对应§4.2"扩容目标复用既有HPA"
        })
        .collect()
}

// 事件结束后既定时间自动回落，对应§4.2"事件结束后既定时间自动回落"
fn schedule_prewarm_rollback(trigger: &PrewarmTrigger, rollback_delay: Duration) {
    send_later(rollback_delay, RollbackHpaOverride { plan_id: trigger.plan_id });
    // 全程写入审计(RGS-BAS-003§7既定审计设计)，本文档不重复设计审计存储本身，仅声明调用点
    audit_log(AuditEvent::PrewarmTriggered { plan_id: trigger.plan_id, at: Instant::now() });
}
```

---

## 5. 插件分片维度扩展物理设计

对应RGS-BAS-022§5.1新增字段。落实为对RGS-BAS-005§3既有插件注册表的物理DDL增量变更（`ALTER TABLE`，非新建表——插件注册表本身归属RGS-BAS-005/其DTL文档，本文档仅新增该表已声明的字段）：

```sql
-- 假定既有插件注册表为plugin_registrations(RGS-BAS-005§3既有表,本文档不重复其完整DDL)
ALTER TABLE plugin_registrations
    ADD COLUMN target_shards TEXT[] NOT NULL DEFAULT '{}';
    -- 空数组表示全部分片(向后兼容T0/T1单分片场景，RGS-BAS-022§5.1已声明的语义)

CREATE INDEX idx_plugin_registrations_target_shards
    ON plugin_registrations USING GIN (target_shards);
    -- GIN索引支撑"给定分片ID，查询该分片上生效的插件集合"这一T2+规模下的高频查询(数组包含查询)
```

```rust
// 插件生命周期状态机(复用RGS-BAS-005§4既有状态机)新增分片维度参数
fn enable_plugin(plugin_id: PluginId, target_shards: Vec<ShardId>) -> Result<(), PluginError> {
    // target_shards为空 => 全部分片，与既有单集群行为完全等价(§5"全容量级别可用性"declaration的向后兼容要求)
    let effective_shards = if target_shards.is_empty() {
        all_known_shards()
    } else {
        target_shards
    };
    for shard in &effective_shards {
        // 复用RGS-BAS-005§4既有单分片启用流程，本文档不重新实现，仅按分片循环调用
        invoke_existing_plugin_lifecycle_enable(plugin_id, *shard)?;
    }
    persist_target_shards(plugin_id, effective_shards)
}
```

同一插件在不同分片可处于不同生命周期状态（§5.1既定），故状态查询接口须携带`shard_id`参数：

```protobuf
message GetPluginStateRequest {
  string plugin_id = 1;
  string shard_id    = 2;   // 必填,查询特定分片上的状态(不提供跨分片汇总视图,避免与§3.2"实时状态不跨分片"判定规则混淆)
}
message GetPluginStateResponse {
  string state = 1;   // 取值同RGS-BAS-005§4既有生命周期状态机
}
```

---

## 6. 跨节点同步协议与规模验证算法

对应RGS-BAS-022§5.2跨节点同步机制的规模验证要求，落实为具体协议格式与验证算法。

### 6.1 分片内广播协议（缩小同步范围的具体形式，对应§5.2"若验证不通过"的缓解方案）

```protobuf
// PluginStateSyncMessage：T2+规模下,同步范围从全局广播收窄至target_shards声明的分片集合
message PluginStateSyncMessage {
  string plugin_id       = 1;
  string new_state         = 2;
  repeated string scope_shard_ids = 3;   // 空表示保持既有全局广播(T0/T1向后兼容)；非空表示仅在这些分片内广播
  int64 sync_sequence_no     = 4;        // 单调递增，接收方据此判定是否为陈旧同步消息并丢弃(乱序保护)
}
```

### 6.2 规模验证算法（对应§5.2"必须经专项负载试验验证"）

```rust
// PH-2/PH-4负载试验阶段调用,验证同步时延是否满足AC-CAP-004既定目标
struct SyncLatencyTrial {
    shard_count: usize,
    broadcast_scope: BroadcastScope,   // Global | ShardLocal(target_shards)
}

fn validate_sync_latency(trial: &SyncLatencyTrial, measured_p99_ms: u64, ac_cap_004_threshold_ms: u64) -> ValidationOutcome {
    if measured_p99_ms <= ac_cap_004_threshold_ms {
        ValidationOutcome::Pass
    } else if trial.broadcast_scope == BroadcastScope::Global {
        // 全局广播未达标：按§5.2缓解方案，建议切换到ShardLocal范围重新试验，而非直接判定该插件能力不可用
        ValidationOutcome::RetryWithNarrowedScope
    } else {
        // 即便已收窄到分片内广播仍未达标：§5.2"不得默认既有机制自然适用于更大规模"，
        // 该插件能力在此规模下暂不可启用，需人工介入(可能需要修订同步机制本身，超出本文档范围)
        ValidationOutcome::FailRequiresManualIntervention
    }
}
```

**判定优先级说明**：`RetryWithNarrowedScope`只在`broadcast_scope`当前仍是`Global`时才有意义——已经是`ShardLocal`范围仍未达标，说明问题不在广播范围而在机制本身，此时继续收窄范围无法解决问题，必须转入人工介入分支，这一分支逻辑直接对应§5.2"必须先修订同步机制方可在T2+规模启用"的文字要求。

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：三类组件扩容手段判定的可执行代码、跨分片能力声明与校验的具体协议格式、弹性预留调度算法（含readinessGate状态机与预留Pod交还逻辑）、预测性预热的调度伪代码、插件注册表分片维度扩展的DDL增量与生命周期状态机代码、跨节点同步的分片内广播协议格式与规模验证算法。

本版本明确不覆盖、留待后续：

- T2→T3触发后的多区域拓扑具体设计——按RGS-BAS-022§2.1既定，须先触发RGS-BAS-017§2.3多区域评估门禁，评审通过后另立独立DTL文档或本文档后续版本，本文档不预先设计。
- TBD-CAP-001弹性预留系数（本文档§4.1给出20%初始提案）与TBD-CAP-002的正式校准值——均需PH-4实测数据支撑，当前为设计阶段占位提案。
- RGS-BAS-005既有插件注册表本身的完整DDL与生命周期状态机——本文档§5仅给出`target_shards`字段的增量`ALTER TABLE`与分片维度调用逻辑，插件注册表核心表结构与状态机本身归属RGS-BAS-005/其未来DTL文档职责，本文档不重复设计。
- 跨节点同步机制本身（非分片维度收窄，而是同步协议底层实现，如具体选用的广播/Gossip库）——本文档只给出协议消息格式与规模验证判定逻辑，底层实现选型不属于本文档范围。

后续详细设计建议顺序：与RGS-DTL-001§12建议一致，弹性预留系数与规模验证算法涉及的负载试验建议尽早启动（阻塞§4.1/§6.2从"提案"转为"结论"）；RGS-DTL-023（请求处理链管道）与RGS-DTL-024（集群部署编排）可并行推进，三者均属核心架构支撑性设计，互不阻塞。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-022§2.3 三类组件扩容手段对应表 | §2 |
| RGS-BAS-022§3.2 跨分片能力清单与判定规则 | §3 |
| RGS-BAS-022§4.1 弹性预留 | §4.1 |
| RGS-BAS-022§4.2 预测性预热 | §4.2 |
| RGS-BAS-022§5.1 插件注册表分片维度扩展 | §5 |
| RGS-BAS-022§5.2 跨节点同步机制规模验证 | §6 |
| RGS-BAS-022§6 标准化检查清单 | §2（§6.3可脚本化部分） |
| TBD-CAP-001〜002 | §4.1（初始提案） |
| RGS-BAS-005（插件热插拔机制，本文档§5前提依赖） | §5 |
| RGS-BAS-020§3（分片路由，本文档§2/§3前提依赖） | §2、§3 |
