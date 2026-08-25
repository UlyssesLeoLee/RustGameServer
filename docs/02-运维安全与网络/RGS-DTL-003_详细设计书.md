# 详细设计书（詳細設計書 / Detailed Design Document）

**运维与GM后台管控：admin_db物理数据库设计・AdminService协议线格式・维护模式收敛算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-003 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-003 运维与GM后台管控 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档是继RGS-DTL-001／002／025／026／027之后新一批详细设计中的一份，覆盖02-运维安全与网络域首个文档）。细化RGS-BAS-003§3新增`AdminService`方法为具体协议线格式、§7审计设计新增操作类型落实为`admin_db.ops_ticket`表DDL、§5维护模式传播收敛判定落实为可直接翻译为Rust实现的Quorum-based Ack Counting伪代码、§8.2高危操作阈值（TBD-OPS-003）给出初始默认值提案。**本版本不覆盖**：Webhook签名算法具体实现、告警规则引擎选型、RTC(RuntimeControlService)内部命令队列的具体数据结构。见§7 | 全部 |
| 0.2 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008） | — | 同步父 BAS-003 升版至 v0.2 + 补 AC-OPS-001〜005 验收标准追溯表。覆盖度复查：BAS-003 v0.2 影响章节为 §3.4（新增`QueryHealthView`）与 §13（追溯性表补齐 AC-OPS-001〜005）。DTL-003 v0.1 §3.4 已落实 `QueryHealthView` 协议线格式（无字段请求 + `ServiceHealthEntry` 重复字段），本次仅做 v0.2 升版，不重写 v0.1 已落实内容；新增"验收标准追溯"子节落实 §13 5 条 AC 映射（`OPERATION_AUDIT` 覆盖／故障隔离拓扑／唯一入口留痕／高危操作双人确认+阈值／告警p99压测边界）。**不引入新设计**：补的 5 行追溯表完全对应 BAS-003 §13 既有内容，未新增任何结构性决定；AC-OPS-005 压测方案在 BAS 原文中即注明"留待详细设计"，本文档不越权先行确定。 | §3.4（已落实,无变更）、追溯性§验收标准追溯（新增子节） |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 协议字段是否与RGS-BAS-003§3表格逐一对应，Quorum收敛算法是否真正满足NFR-OPS-006"不阻塞实时路径" |
| 评审（DBA） | | | `ops_ticket`表索引是否覆盖GM后台工单列表常见查询路径 |
| 审批（负责人） | | | 本文档的基准化；TBD-OPS-003阈值提案是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：admin_db新增表](#2-物理数据库设计admindb新增表)
3. [AdminService协议线格式](#3-adminservice协议线格式)
4. [维护模式传播收敛算法详细设计](#4-维护模式传播收敛算法详细设计)
5. [运行时受限控制通道命令线格式](#5-运行时受限控制通道命令线格式)
6. [TBD-OPS-003高危操作阈值默认值提案](#6-tbd-ops-003高危操作阈值默认值提案)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-003给出了`AdminService`新增方法的字段级表格、运行时受限控制通道(`RuntimeControlService`)的组件设计表、维护模式传播时序图与"Quorum-based Ack Counting＋超时兜底"的文字描述、RBAC矩阵与高危操作判定表。本文档将其落实为：具体协议线格式（字段编号）、`admin_db.ops_ticket`可执行DDL、维护模式收敛判定的算法级伪代码（覆盖并发/超时边界条件）、TBD-OPS-003高危操作阈值的初始默认值提案。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-003已确定的任何结构性选择（GM后台经唯一入口`AdminService`、`RuntimeControlService`与运行时Pod同进程部署、`CreateOpsTicket`不触发任何K8s操作本身）。
- 不覆盖Webhook签名算法的具体实现——RGS-BAS-003原文已明确该项"留待详细设计阶段与RGS-IFS-001确定"，本文档不越权先行决定，仅在§3固定推送事件本身的线格式字段（幂等键`event_id`），签名字段的算法细节留白。
- 不覆盖告警规则引擎选型——依ARC-014判定基准，属独立技术选型决策，不属本文档（GM后台管控域）职责范围。
- 不覆盖`RuntimeControlService`内部有界命令队列的具体数据结构与背压参数——RGS-BAS-003§4.2已给出设计原则（有界、tick边界消费、拒绝而非无界排队），具体队列容量属实现阶段的性能调优参数，非架构层面决策，本文档不预先固定数值。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，协议以Protobuf风格给出（`AdminService`内部走既有gRPC鉴权路径，非RGS-DTL-027匿名HTTP场景），算法伪代码可直接对应Rust `Result`实现。字段编号规则同RGS-DTL-001§1.3：1〜15高频，16以上低频/可选，编号不可变更/复用。

---

## 2. 物理数据库设计：admin_db新增表

对应RGS-BAS-003§10。`ops_ticket`依附既有`admin_db`（AD限界上下文），复用其既有连接池与迁移工具链。

```sql
-- 运维工单表，对应RGS-BAS-003§10设计要点表，FR-GM-032落地
CREATE TABLE ops_ticket (
    ticket_id     BIGSERIAL PRIMARY KEY,
    op_type       SMALLINT NOT NULL,   -- 0=扩容建议 1=缩容建议 2=滚动更新请求 3=Pod重启请求（RGS-BAS-003§3.3枚举定义直译）
    payload       JSONB NOT NULL,      -- 具体参数，结构随op_type变体，不在DB层强约束子结构（同RGS-DTL-025 transaction_ledger.payload先例）
    status        SMALLINT NOT NULL DEFAULT 0
                    CHECK (status BETWEEN 0 AND 4),
                    -- 0=待处理 1=已批准 2=已拒绝 3=执行中 4=已完成（RGS-BAS-003§10表格状态枚举直译）
    created_by    UUID NOT NULL,       -- 逻辑引用GM后台操作者身份，跨系统边界不建物理FK（GM后台在本系统信任边界之外，同RGS-BAS-003§1.2既定拓扑）
    approved_by   UUID NULL,           -- SRE/值班审批者，待批准前为NULL
    executed_at   TIMESTAMPTZ NULL,    -- CI/CD流水线或人工执行完成时回写，执行前为NULL
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    version       INTEGER NOT NULL DEFAULT 0  -- OCC乐观锁，覆盖"SRE批准"与"CI回写执行状态"两条更新路径的并发竞态
);

CREATE INDEX idx_ops_ticket_status_created
    ON ops_ticket (status, created_at)
    WHERE status IN (0, 1, 3);
    -- 部分索引：支撑GM后台/SRE工具"当前待处理/待执行工单列表"查询，已完成/已拒绝工单不占索引空间
CREATE INDEX idx_ops_ticket_created_by ON ops_ticket (created_by);
    -- 支撑"某操作者发起的历史工单"审计追溯查询
```

`admin_db.OPERATION_AUDIT`表结构复用RGS-BAS-001§5.7既有定义，本文档不重复其DDL，仅在§3.4声明`action_type`枚举扩展值（`KICK_SESSION`／`MUTE_CHAT`／`RELOAD_CONFIG`／`SCENE_RESTART_REQUEST`／`SCENE_RESTART_CONFIRM`／`OPS_TICKET_CREATED`），与既有`BAN_ACCOUNT`／`GRANT_COMPENSATION`／`SET_MAINTENANCE_MODE`并列——这是对既有枚举列的追加值，不改变列类型本身。

---

## 3. AdminService协议线格式

对应RGS-BAS-003§3全部方法表格。以下固定字段编号，方法签名与字段名与RGS-BAS-003§3一一对应，不引入新字段。通用`request_id`／`trace_id`／`result_code`延续RGS-BAS-001§6.1既定通用字段设计。

### 3.1 账号/会话级方法

```protobuf
message KickSessionRequest {
  string request_id    = 1;
  string character_id  = 2;
  string reason         = 3;
  string operator_id     = 4;
}
message KickSessionResponse {
  ResultCode result_code = 1;   // 复用RGS-DTL-001§4.4通用ResultCode，不重复定义
  bool session_terminated = 2;
}

message MuteChatRequest {
  string request_id    = 1;
  string character_id  = 2;
  ChatChannel channel    = 3;   // 复用既有ChatMessage.channel枚举定义（RGS-BAS-001§6.2.2），不重复定义
  int64  expires_at_ms    = 4;  // 0表示永久（proto3不区分未设置与0，消费端按channel/expires_at_ms=0判定永久，同RGS-DTL-025 raw_value先例的0语义处理方式）
  string operator_id       = 5;
}
message MuteChatResponse {
  ResultCode result_code = 1;
  string mute_id           = 2;
}
```

### 3.2 场景/运行时级方法

```protobuf
message QuerySceneMetricsRequest {
  string scene_id = 1;
}
message QuerySceneMetricsResponse {
  int32  entity_count          = 1;
  double avg_tick_duration_ms   = 2;
  int32  mailbox_depth           = 3;
  SceneHealthStatus status         = 4;  // 0=正常 1=告警 2=严重
}
enum SceneHealthStatus {
  NORMAL   = 0;
  WARNING  = 1;
  CRITICAL = 2;
}

message RequestSceneRestartRequest {
  string request_id = 1;
  string scene_id     = 2;
  string reason         = 3;
  string operator_id      = 4;
}
message RequestSceneRestartResponse {
  string ticket_id       = 1;
  ApprovalTicketStatus status = 2;  // 0=待确认
}
enum ApprovalTicketStatus {
  PENDING_CONFIRMATION = 0;
}

message ConfirmSceneRestartRequest {
  string ticket_id   = 1;
  string approver_id  = 2;   // 须持有高危操作角色且与申请者不同，§4.1 RBAC校验点
}
message ConfirmSceneRestartResponse {
  ResultCode result_code = 1;
  int64 executed_at_ms      = 2;
}
```

### 3.3 配置/发布级方法

```protobuf
message ReloadConfigTableRequest {
  string request_id     = 1;
  string table_version    = 2;
  string operator_id        = 3;
}
message ReloadConfigTableResponse {
  ReloadResultCode result_code = 1;  // 0=成功 1=一致性检查未通过（ARC-016既有）
  int64 applied_at_ms             = 2;
}
enum ReloadResultCode {
  RELOAD_OK                    = 0;
  RELOAD_CONSISTENCY_CHECK_FAILED = 1;
}

// SetMaintenanceMode既有请求字段不变（RGS-BAS-001既有），响应新增propagation_status
message SetMaintenanceModeResponse {
  ResultCode result_code            = 1;
  PropagationStatus propagation_status = 2;   // 新增字段，编号紧接既有响应字段之后（既有字段编号1〜N不变，本文档不改变既有分配）
}
enum PropagationStatus {
  PROPAGATING = 0;   // 传播中
  CONVERGED   = 1;   // 已生效（§4收敛算法判定）
}

message CreateOpsTicketRequest {
  string request_id  = 1;
  OpsTicketType op_type = 2;   // 0=扩容建议 1=缩容建议 2=滚动更新请求 3=Pod重启请求，与ops_ticket.op_type同枚举
  string payload_json   = 3;    // JSON序列化字符串，具体schema随op_type变体，DB层同样不强约束（§2）
  string operator_id      = 4;
}
message CreateOpsTicketResponse {
  string ticket_id            = 1;
  OpsTicketStatus status         = 2;  // 0=待处理，与ops_ticket.status同枚举
}
enum OpsTicketType {
  SCALE_UP_SUGGESTION   = 0;
  SCALE_DOWN_SUGGESTION = 1;
  ROLLING_UPDATE_REQUEST = 2;
  POD_RESTART_REQUEST     = 3;
}
enum OpsTicketStatus {
  PENDING  = 0;
  APPROVED = 1;
  REJECTED = 2;
  EXECUTING = 3;
  COMPLETED = 4;
}
```

### 3.4 查询/审计方法

```protobuf
message QueryOnlineStatusRequest {
  OnlineStatusFilter filter = 1;
}
message OnlineStatusFilter {
  string player_id = 1;   // 可选
  string scene_id    = 2;  // 可选
  int32  page_size      = 10;  // 分页参数置于10+区间，与业务过滤字段区分（复用RGS-DTL-001§4.3 20+区间分组编号纪律的同类思路）
  string page_token       = 11;
}
message QueryOnlineStatusResponse {
  repeated OnlinePlayerEntry players = 1;
  string next_page_token                = 2;
}
message OnlinePlayerEntry {
  string character_id  = 1;
  string scene_id        = 2;
  int64  connected_at_ms   = 3;
  int64  session_epoch       = 4;   // ARC-005既有字段
}

message QueryAuditLogRequest {
  AuditLogFilter filter = 1;
  int32  page_size          = 10;
  string page_token            = 11;
}
message AuditLogFilter {
  string operator_id = 1;    // 可选
  string action_type   = 2;  // 可选，取值同§2 action_type枚举扩展
  int64  time_from_ms     = 3;  // 可选
  int64  time_to_ms          = 4;  // 可选
}
message QueryAuditLogResponse {
  repeated AuditLogEntry entries = 1;
  bool   has_more                    = 2;
}
message AuditLogEntry {
  // 字段与既有OPERATION_AUDIT表列一一对应（RGS-BAS-001§5.7），本文档不重复该表DDL
  string audit_id     = 1;
  string action_type    = 2;
  string operator_id      = 3;
  string payload_summary    = 4;
  int64  occurred_at_ms        = 5;
}

message QueryHealthViewRequest {}  // 无字段，全局聚合
message QueryHealthViewResponse {
  repeated ServiceHealthEntry services = 1;
}
message ServiceHealthEntry {
  string service_name       = 1;
  bool   ready                 = 2;
  double queue_depth              = 3;
  double db_pool_usage_ratio         = 4;
  int64  checked_at_ms                  = 5;
}
```

---

## 4. 维护模式传播收敛算法详细设计

对应RGS-BAS-003§5"收敛判定算法（RGS-BAS-010§4 G-008补强）"文字描述，落实为伪代码。

### 4.1 数据结构

```rust
struct PropagationTracker {
    maintenance_change_id: Uuid,       // 每次SetMaintenanceMode调用产生一个跟踪实例
    total_expected_nodes: u32,          // 发起时刻集群内应确认的节点总数快照（网关+运行时+业务服务）
    acked_node_ids: HashSet<NodeId>,     // 已回执节点集合
    started_at: Instant,
    quorum_ratio: f64,                    // §6提案默认值 0.95
    max_wait: Duration,                    // 复用ARC-013既有超时要求，具体值不在本文档重复定义
}
```

### 4.2 收敛判定主循环

```rust
// 由AdminService后台任务定期（如每秒）轮询各PropagationTracker，
// 或在收到节点回执时事件驱动触发，两种触发方式均调用本函数，语义等价（幂等判定）
fn evaluate_convergence(tracker: &mut PropagationTracker, now: Instant) -> PropagationStatus {
    let ack_ratio = tracker.acked_node_ids.len() as f64 / tracker.total_expected_nodes as f64;
    let elapsed = now.duration_since(tracker.started_at);

    // 法定人数达成 或 超时兜底，两者中较早满足者判定为已生效（RGS-BAS-003§5明文要求）
    if ack_ratio >= tracker.quorum_ratio {
        return PropagationStatus::Converged;
    }
    if elapsed >= tracker.max_wait {
        // 超时兜底：未响应的少数节点必须被记录并告警，但不阻塞收敛判定本身
        let unacked = tracker.total_expected_nodes as usize - tracker.acked_node_ids.len();
        emit_metric("maintenance_propagation_timeout_unacked_nodes", unacked as f64);
        // 复用RGS-BAS-004§6.2强制全量采集：本条日志属"降级/背压拒绝路径"类比场景，视为异常信号
        log_warn("maintenance propagation timed out with unacked nodes", unacked);
        trigger_alert(AlertSeverity::Warning, "maintenance_propagation_incomplete", tracker.maintenance_change_id);
        return PropagationStatus::Converged;  // 超时同样判定为已生效，不永久卡在传播中
    }
    PropagationStatus::Propagating
}
```

**边界条件说明**：

- `total_expected_nodes`在发起时刻快照固定，而非实时查询当前集群规模——若在传播期间发生扩缩容，新增节点不计入本次`maintenance_change_id`的分母（否则分母增长会导致`ack_ratio`永远无法达到法定比例），新增节点应在自身启动时主动拉取当前维护状态（同§7.1插件热插拔跨节点同步的既定"节点启动时主动同步当前状态"惯例，非本文档新增机制）。
- `acked_node_ids`使用集合而非计数器，是为了容忍同一节点重复回执（网络重传）不重复计数——幂等性要求。
- 判定函数本身不阻塞`SetMaintenanceMode`的RPC返回路径（该RPC在写入审计记录后立即返回`propagation_status=PROPAGATING`，同RGS-BAS-003§5时序图），`evaluate_convergence`是异步后台任务，`QueryHealthView`或专门的状态查询接口可用于客户端轮询当前收敛状态（具体查询接口本文档不新增，复用现有`SetMaintenanceMode`响应字段与后续状态变更通过§3.3 `PropagationStatus`枚举表达）。

---

## 5. 运行时受限控制通道命令线格式

对应RGS-BAS-003§4.3命令表格，落实为`AdminService`→`RuntimeControlService`的内部gRPC线格式（该通道mTLS+服务身份校验，不面向GM后台，编号规则同前）。

```protobuf
message RuntimeCommand {
  string request_id = 1;    // 幂等键，§4.2既定"重复下发不产生重复副作用"
  oneof command {
    KickSessionCommand kick_session = 10;
    MuteChatCommand mute_chat         = 11;
    QuerySceneMetricsCommand query_metrics = 12;
    RestartSceneCommand restart_scene        = 13;
  }
}
message KickSessionCommand {
  string character_id = 1;
  string reason          = 2;
}
message MuteChatCommand {
  string character_id  = 1;
  string channel          = 2;
  int64  expires_at_ms       = 3;
}
message QuerySceneMetricsCommand {
  string scene_id = 1;
}
message RestartSceneCommand {
  string scene_id     = 1;
  string ticket_id       = 2;   // 关联ConfirmSceneRestart产生的审批工单，供RTC侧留痕核对
}

message RuntimeCommandResult {
  ResultCode result_code = 1;
  // 过载场景专属：命令队列达到上限时返回本码而非处理命令，同RGS-BAS-003§4.2背压设计
  // OVERLOADED = 8（追加至RGS-DTL-001§4.4通用ResultCode枚举，编号紧接既有DUPLICATE_REQUEST_ID=7之后）
}
```

**队列消费边界条件**（对应RGS-BAS-003§4.2"命令在tick边界之间被消费，不得在tick执行中途插入"）：

```rust
fn scene_actor_tick(scene: &mut SceneState, tick_no: u64) {
    // ... 既有RGS-DTL-001§5.1阶段1〜5不变 ...

    // 新增阶段0（tick最开始，早于输入应用）：消费本tick边界前到达的运行时控制命令
    // 置于阶段1之前是为了让KickSession等命令在同一tick内即生效，避免"已被踢出但仍处理其输入"的窗口
    while let Some(cmd) = scene.control_command_queue.try_pop() {
        apply_runtime_command(scene, cmd);  // 幂等：request_id已处理过则直接返回既有结果，不重复应用
    }
    // ... 原阶段1〜5不变 ...
}
```

---

## 6. TBD-OPS-003高危操作阈值默认值提案

RGS-BAS-003§8.2标注"批量`BanAccount`/`GrantCompensation`触发二次确认的阈值"为TBD-OPS-003。本文档提出以下初始默认值供上线使用，非最终值，与RGS-DTL-025§5、RGS-DTL-026§4.1同类做法一致：

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| `BanAccount`批量阈值 | 单批次≥50个账号 | 覆盖常规单次GM人工处置（通常个位数到十几个账号）之外的批量场景，误伤成本随批次线性增长，50作为"明显超出人工逐个核实合理范围"的经验起点 |
| `GrantCompensation`批量阈值（影响人数） | 单批次≥100人 | 补偿发放通常成批次针对某次事故影响的玩家群体，100人以下的常规批次由既有单人操作流程处理，超过则值得双人复核 |
| `GrantCompensation`批量阈值（道具总值） | 单批次道具总值≥运营定义的"高价值"基准线的10倍（具体基准线依赖ARC-016数值表配置，本文档不重复定义该基准线本身） | 即便影响人数不足100，若单批次总价值畸高（如误操作导致的天文数字补偿），同样应触发二次确认，故两个条件为"或"关系而非"且" |
| Quorum-based Ack Counting法定比例 | 95% | 同RGS-BAS-003§5原文示例值，本文档直接采纳为初始默认值 |

以上默认值应在上线后按PH-4阶段真实运营数据（误伤率/漏检率/GM操作习惯）校准，校准结果回写本文档新版本，不在RGS-BAS-003基本设计层面体现（属于实现参数调优，非结构性设计变更）。

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：`admin_db.ops_ticket`表物理DDL、`AdminService`全部新增方法（§3.1〜3.4）的具体协议线格式、维护模式传播收敛判定的完整伪代码（含节点集合快照/幂等回执/超时兜底边界条件）、运行时受限控制通道命令线格式与tick边界消费点的具体实现、TBD-OPS-003四项高危操作阈值参数的初始默认值提案。

本版本明确不覆盖、留待后续：

- Webhook推送（IF-008）的具体签名算法——RGS-BAS-003原文明确留待详细设计阶段与RGS-IFS-001确定，本文档不越权先行选定。
- 告警规则引擎的具体选型与规则定义语法——依ARC-014判定基准，属独立技术选型评审范畴。
- `RuntimeControlService`内部有界命令队列的具体容量参数与背压拒绝的精确阈值——实现阶段按性能实测确定的调优参数，非架构决策。
- §6高危操作阈值的正式校准结果——当前为初始提案，非最终值，校准需等待PH-4真实运营数据，且需与运营团队评审（原BAS文档已注明）。
- RGS-OPS-001运维手顺书本身（值班排班、故障响应SOP、事后复盘模板）——RGS-BAS-003§11已明确该文档与本文档（及其详细设计）分工不同，不属本文档职责范围。

后续详细设计建议顺序：与RGS-DTL-004（埋点与日志规范）存在强耦合（§4收敛算法的告警/日志埋点、§6.2强制全量采集范围均引用RGS-BAS-004既有设计），建议两份文档交叉核对后再各自基准化；核心架构`admin_db`全貌（RGS-DTL-001遗留的match_db／social_db／admin_db部分）补齐后，应回头核对本文档新增的`ops_ticket`表是否需要在RGS-DTL-001新版本中被引用，避免同一数据库物理设计分散在多份文档互不知情（同RGS-DTL-025§6已提出的同类风险提示）。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-003§2 整体控制平面架构 | 前提依赖，本文档假定拓扑不变 |
| RGS-BAS-003§3 AdminService字段级API扩展设计 | §3 |
| RGS-BAS-003§4 运行时受限控制通道设计 | §5 |
| RGS-BAS-003§5 维护模式传播设计 | §4、§3.3 |
| RGS-BAS-003§7 审计与查询设计 | §2（`action_type`枚举扩展） |
| RGS-BAS-003§8 RBAC角色矩阵扩展与高危操作二次确认 | §6（TBD-OPS-003默认值提案） |
| RGS-BAS-003§10 K8s层运维工单设计 | §2（`ops_ticket`表）、§3.3 |
| RGS-DTL-001§4.4 通用ResultCode | §3、§5（复用，追加OVERLOADED枚举值） |
| RGS-DTL-025§5 / RGS-DTL-026§4.1（TBD默认值提案先例） | §6 |

### 验收标准追溯（同步RGS-BAS-003 v0.2 §13 AC-OPS-001〜005 补强）

| 验收标准 | 决定摘要 | 本文档落实章节 |
|---|---|---|
| AC-OPS-001 | 六类GM管控操作均产生`OPERATION_AUDIT`审计记录 | §2（`action_type`枚举扩展值）、§3.1〜3.3 全部方法线格式均通过`AdminService`统一入口（与`OPERATION_AUDIT`写入路径一致） |
| AC-OPS-002 | 故障注入下控制平面不影响实时路径 | §3.2（场景级方法经`RuntimeControlService`转发，与tick循环解耦，命令队列背压拒绝时返回`OVERLOADED`而非阻塞）、§5 tick边界消费设计 |
| AC-OPS-003 | GM后台凭证无法直连业务DB/K8s API | §3.1〜3.4 全部方法仅经`AdminService`唯一入口（与RGS-BAS-003§2.1组件图拓扑一致，本文档不引入新路径） |
| AC-OPS-004 | 高危操作二次确认留痕 | §3.2（`RequestSceneRestart`/`ConfirmSceneRestart`独立审计，关联`ticket_id`）、§6（TBD-OPS-003阈值提案：批量`BanAccount`≥50、批量`GrantCompensation`≥100人或道具总值≥高价值基准线10倍触发二次确认） |
| AC-OPS-005 | 告警p99时延达标 | §4.2（收敛算法`evaluate_convergence`超时兜底发`maintenance_propagation_incomplete`告警，触发链路经既有OTel Collector→告警规则引擎→Webhook，不引入新路径）、§5 运行时命令通道（命令队列`OVERLOADED`与正常处理均通过`ResultCode`上抛，沿既有告警埋点） |

**注**：AC-OPS-005 告警p99时延的具体压测方案在BAS-003 §13 原文中即注明"留待详细设计"，本文档不越权先行确定压测方案；本表只映射"验收标准→本文档已落实的逻辑/接口章节"，不补压测方案本身。
