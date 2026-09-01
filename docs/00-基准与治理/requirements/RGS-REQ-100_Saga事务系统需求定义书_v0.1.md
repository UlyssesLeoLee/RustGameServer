# RGS-REQ-100 Saga 事务系统需求定义书

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-100 |
| 版本 | 0.2（v0.1 + saga-runtime 独立 Pod 评估） |
| 制定日 | 2026-08-21（v0.1）/ 2026-09-01 22:30 JST（v0.2） |
| 最终更新日 | 2026-09-01 22:30 JST |
| 制定者 | 架构师（Ulysses 兼，per DEC-008 一人公司） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 关联文档 | RGS-BAS-100（基本设计书）/ RGS-DTL-100~102（详细设计书 3 份）/ RGS-OPS-100（K3s 部署）/ RGS-GOBS-100（可观测性）/ RGS-SEC-100（安全审计）/ RGS-SPEC-CROSS-001~007（横向规范） |
| 配套标准 | IPA 共通フレーム 2013（SLCP-JCF2013）+ 150 工程日本 SI 业界标准；V 模型映射：ST ↔ REQ（本需求书） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。覆盖 L0-L5 状态分层 / OperationPolicy 决策层级 / Saga 触发条件 / 商城购买 / 角色创建 / 比赛奖励 / GM 补偿 / 跨服转移 / 浏览器关闭不影响 Saga / K3s Pod 恢复 / 多副本 OCC / 纯开源约束 / 17 个必答问题。 |
| 0.2 | 2026-09-01 22:30 JST | 架构师(Mavis 接手 agent per DEC-008, 代签 Ulysses) | v0.2 评估: saga-runtime 独立 Pod 落点 (per WBS v0.2 桶 10 Phase D D5, commit 84edf26)。决策项留给 Ulysses 拍板 (per AGENTS.md v0.4 §7 batch 域派生约束 + WBS v0.2 §4.3 拍板 3)。 |

---

## 0. 文档目的

定义 RustGameServer 平台 **Frontend Local Transaction + Distributed Saga + Event-Driven Microservices** 统一状态管理与分布式事务体系的需求基线。

**本系统不是独立项目**，而是**现有游戏服务器平台的基础设施能力**，融入 5 域（player / economy / match / social / admin）+ cluster-ops + shared-platform + 后台 10 个 App 的整体架构。

**最高架构原则**：

> 每一个操作都必须落入且只能落入最合适的一层。
> UI owns ephemeral state. Frontend owns local interaction state. Service owns domain state. Service transaction owns local ACID. Saga owns distributed business consistency.
> 目标不是"让所有东西统一经过 Saga"，而是建立一个严格、低开销、可恢复的层级体系：Local UI → Local Transaction → Local First → Single-Service ACID → Distributed Saga。

---

## 1. 范围

### 1.1 包含

- L0-L5 6 层状态分层定义与权威归属
- OperationPolicy 决策层级（UI Only / Local Only / Local First / Single-Service / Distributed Saga）
- Authority Boundary 强制约束（前端/Preference Service/各 Rust 微服务/Saga Runtime）
- Saga Runtime 自研（containerized + stateless compute + PostgreSQL persistent state + horizontal scalable）
- Reservation Model（Currency / Inventory Slot / Match Slot / Room Capacity）
- Outbox + Inbox Pattern（本地 ACID + 跨服务 at-least-once + 幂等消费）
- Saga Store 9 张表 schema（saga_definition / saga_instance / saga_step / saga_event / saga_command / saga_compensation / saga_snapshot / saga_failure / saga_audit）
- Saga Event Journal（11 种事件类型，append-only）
- K3s Pod crash 恢复（4 状态 RUNNING/WAITING/RETRYING/COMPENSATING）
- 多副本 OCC（PostgreSQL `SELECT ... FOR UPDATE SKIP LOCKED` + Fencing Token）
- K3s Service Discovery（逻辑 participant id，不写 pod IP）
- 消息总线（NATS JetStream 推荐）/ Redpanda / Kafka 评估
- 实时游戏 Tick 与 Saga 边界（Position/Combat/AI 不进 Saga，MatchFinished 后才进 Reward Saga）
- 不可逆事件处理（Corrective Event / Reconciliation / Manual Intervention）
- GM 安全（RBAC + Audit Log + Operator/Reason/RequestID + 高风险 before/after state）
- Admin Saga Console（Saga Monitor App，Timeline/DAG/Retry/Pause/Resume/Manual Compensation）
- 可观测性（OpenTelemetry + Prometheus + Grafana + Loki + Tempo，全链路 trace_id/saga_id/command_id/player_id）
- 5 域 + cluster_ops + shared-platform + Saga Runtime 的 K3s 部署资源（Deployment/Service/ConfigMap/Secret/PDB/HPA/PVC/NetworkPolicy/ServiceAccount，按需）
- 3 种部署 profile（Minimal / Standard / HA）
- 纯开源约束（Apache-2.0/MIT/BSD 优先，禁止 Redis Enterprise / 云专有 / 闭源事务协调器）

### 1.2 不包含

- 实时游戏模拟层（movement / rotation / combat tick / animation / physics / NPC update / heartbeat）— 走低延迟内存路径，不进 Saga
- Redis Enterprise / 云厂商专有服务 / 商业 SaaS / 闭源事务协调器
- 与游戏逻辑无关的通用 ERP / CRM / 财务系统
- 玩家客户端原生代码（iOS / Android / 主机）— 仅 Admin UI / Game Web Client 在范围内

### 1.3 适用 Apps

后台 10 个 App（每 App owns local UI state，跨 App 通过 Frontend Event Bus）：

- Player App / Account App / Character App
- Inventory App / Economy App / Mail App
- Ban App / Server App / Match App / Guild App

---

## 2. 名词术语

| 术语 | 缩写 | 含义 |
|---|---|---|
| Authority Boundary | — | 业务权威边界。跨越则必须进服务器事务或 Saga |
| Operation Policy | — | 操作策略注册表（UI_ONLY / LOCAL_ONLY / LOCAL_FIRST / SINGLE_SERVICE / DISTRIBUTED_SAGA）|
| Saga Runtime | — | 分布式 Saga 协调器（K3s Deployment，stateless + PG persistent）|
| Saga Step | — | Saga 内一个具体步骤（Reserve / Commit / Compensate）|
| Saga Command | — | Saga 发送给微服务的业务指令（带 command_id + idempotency_key）|
| Saga Event | — | Saga 内/外的事件（SagaStarted / StepSucceeded / SagaFailed 等 11 种）|
| Reservation | — | 业务预留（避免实际副作用的乐观锁定）|
| Compensation | — | 已产生副作用后的语义补偿（不可逆事件走 Corrective Event）|
| Outbox | — | 本地 DB 事务内追加事件，异步发布到 MQ |
| Inbox | — | 消费者侧去重表，保证 at-least-once + 幂等 |
| Fencing Token | — | 单调递增令牌，阻止过期 Leader 写入 |
| Optimistic Concurrency Control | OCC | 乐观并发控制（PostgreSQL row version 或 SELECT FOR UPDATE）|
| Operation Transaction | OT | Operation Transaction；本规范称"Saga"的业务侧别名 |
| Local-First | — | L2 状态：本地提交 + 异步同步 + 冲突解决策略 |
| Backward Compat | — | 向后兼容（Saga Definition 演化时旧实例不中断）|

---

## 3. 业务需求（BR）

### BR-100 状态分层强制

系统必须把状态严格分为 6 层：

| 层级 | 名称 | 权威归属 | 典型场景 |
|---|---|---|---|
| L0 | Pure UI State | 浏览器 | hover / focus / panel size / tab / dialog / scroll / animation / 临时表单 / canvas zoom |
| L1 | Frontend Local Transaction | 浏览器 | 后台布局 / 未提交表单 / 临时过滤器 / 页面导航 / 临时选择 |
| L2 | Local-First State | 浏览器 + Preference Service | GM 页面布局 / 主题 / 列宽 / Dashboard / 最近打开页面 |
| L3 | Server Projection State | 各微服务 | 玩家列表 / 在线人数 / 角色概要 / 背包展示 / 订单状态 / Saga 状态 |
| L4 | Domain State | 各 Rust 微服务 | Account / Inventory / Currency / Character / Guild / Match |
| L5 | Distributed Saga State | Saga Runtime | 商城购买 / GM 补偿 / 角色创建 / 跨服转移 / 公会创建 / 奖励发放 / 比赛结果 / 副本创建 |

### BR-101 决策层级（最低成本优先）

每个操作必须落入且只能落入最合适的一层：

```
UI Only (L0)
   ↓
Frontend Local (L1)
   ↓
Local First (L2)
   ↓
Single-Service ACID (L3/L4)
   ↓
Distributed Saga (L5)
```

**禁止反向**：能用 Single-Service 完成的不得升级为 Saga。

### BR-102 Saga 触发条件（白名单）

只有满足以下**任意**条件才能进入 Distributed Saga：

1. 涉及两个或更多业务微服务
2. 涉及多个数据库事务边界
3. 有跨服务业务补偿
4. 涉及异步业务链
5. 涉及外部不可逆副作用（邮件 / webhook / 公告）
6. 涉及跨服资源
7. 涉及需要持久化恢复的长事务

**反例（应继续 Single-Service Transaction）**：

- 修改角色昵称（仅 Character Service + Character DB）
- 单次扣货币（仅 Economy Service + Economy DB）
- 玩家登录（仅 Account Service + session）

### BR-103 实时游戏事件禁入 Saga

明确禁止以下事件进入 Saga Runtime：

- movement / rotation / position update
- combat tick / damage event
- animation / physics update
- NPC update / AI state
- heartbeat / position broadcast

**理由**：实时战斗路径必须保持低延迟 + 内存权威 + 游戏循环，Saga 是 business workflow engine，不是 gameplay event engine。

**边界**：MatchFinished / RewardGranted / InventoryUpdate / RankUpdate / MailSent 可在 Match Service 之后进入 Saga。

### BR-104 Reservation 优先于 Compensation

设计必须明确：

- **优先 Reservation**：Currency Reserve / Inventory Slot Reserve / Match Slot Reserve / Room Capacity Reserve
- 失败 → Release
- 只有**必须产生业务状态改变**时才使用 Compensation

理由：避免"扣钱 → 退款"+"发道具 → 删道具"等高风险操作。

### BR-105 经济系统幂等性

所有 Economy Command 必须：

- **idempotent**（重复执行不重复扣钱）
- 携带 `command_id` + `saga_id` + `step_id` + `idempotency_key`
- Economy Service 保存处理结果（Inbox 表）

### BR-106 Inventory 语义补偿

Inventory Service 必须支持：

- 状态机：RESERVED → COMMITTED
- 补偿：RevokeItem（处理已使用 / 交易 / 出售 / 强化 / 拆解场景）
- **不能简单 DELETE**——必须设计 Item Lineage + Item Transaction ID

### BR-107 不可逆游戏事件

比赛已结束 / 公告已发送 / 玩家已看到奖励通知 / 第三方 webhook 已发送 → **不能简单回滚**。必须：

- 发出 **Corrective Event**
- **Reconciliation** 流程
- 必要时 **Manual Intervention**

### BR-108 浏览器不参与 Saga

Admin UI / Game Client 只是：

- **Command Initiator**（发起命令）
- **Projection Consumer**（订阅投影）
- **Local UI Coordinator**（本地状态协调）

不能成为 **Business Transaction Coordinator**。**浏览器关闭不影响 Saga**——Saga 继续。

### BR-109 GM 高风险操作审计

GM Command 必须：

- RBAC（角色 + 资源 + 操作）
- Audit Log（per 操作）
- Operator ID + Reason + Request ID
- 高风险操作（封禁 / 删号 / 扣货币 / 资产转移 / 服务器迁移）额外要求：
  - before state + after state
  - 二次权限校验（per Admin Saga Console）
  - Saga 级别隔离（per Saga Store）

### BR-110 K3s 部署标准

为每个组件按需设计 K3s 资源：

- Deployment（无状态）+ StatefulSet（有状态，如 PostgreSQL）
- Service（ClusterIP / Headless / NodePort / LoadBalancer，按需）
- ConfigMap + Secret（sealed-secrets / external-secrets）
- PodDisruptionBudget（PDB）
- HorizontalPodAutoscaler（HPA）
- PersistentVolumeClaim（PVC）
- NetworkPolicy（东西向隔离）
- ServiceAccount + RBAC

**禁止机械地所有组件都配置所有资源**——按需选择。

### BR-111 纯开源约束

最终输出组件清单（每组件标注 License / Commercial Use / Role / Alternative）。**核心组件必须 OSI-compatible 开源许可证**。禁止：

- Redis Enterprise
- 云厂商专有服务
- 商业 SaaS
- 闭源事务协调器（Temporal Cloud / Cadence 等商业版）

如 Redis 功能确实需要，评估纯开源替代方案（KeyDB / Dragonfly / 自研 PostgreSQL-based 缓存）。

### BR-112 可观测性

使用 OpenTelemetry 统一追踪：

- `trace_id`（端到端）
- `saga_id`（业务事务）
- `step_id`（Saga 步骤）
- `command_id`（业务指令）
- `event_id`（事件唯一）
- `player_id` / `match_id`
- `service` / `pod`（基础设施）

可以追踪：Admin Click → Admin Gateway → Saga → Economy → NATS → Inventory → Compensation。

---

## 4. 功能需求（FR）

### FR-100 状态分层强制

每个 Rust 微服务必须**只写自己的 DB**（Database per Service）。**禁止直接 SQL 访问其他服务数据**。

实际部署允许 `shared PostgreSQL cluster + separate database/schema/user` 以减少 K3s 资源，但服务之间仍禁止 direct SQL。

### FR-101 Authority Map

必须为每个领域建立 Authority Map：

```
Frontend Layout       → Browser
Admin Preferences     → Preference Service
Account               → Account Service
Character             → Character Service
Inventory             → Inventory Service
Currency              → Economy Service
Match                 → Match Service
Guild                 → Guild Service
Mail                  → Mail Service
Saga                  → Saga Runtime
```

### FR-102 Operation Policy Registry

每个后台操作必须注册：

| 操作 | Scope | 触发 G-CODE |
|---|---|---|
| ResizePanel | UI_ONLY | — |
| ChangeAdminTheme | LOCAL_FIRST | — |
| ViewPlayer | READ_ONLY (L3) | — |
| EditPlayerNote | SINGLE_SERVICE (Character) | — |
| GrantCurrency | SINGLE_SERVICE (Economy) | — |
| SendCompensationPack | DISTRIBUTED_SAGA | G-CODE-SAGA-01 |
| TransferPlayerServer | DISTRIBUTED_SAGA | G-CODE-SAGA-02 |
| DeleteCharacter | DISTRIBUTED_SAGA | G-CODE-SAGA-03 |
| PurchaseShopItem | DISTRIBUTED_SAGA | G-CODE-SAGA-04 |
| CreateCharacterWithStarterPack | DISTRIBUTED_SAGA | G-CODE-SAGA-05 |
| DistributeMatchReward | DISTRIBUTED_SAGA | G-CODE-SAGA-06 |

### FR-103 Saga Store Schema

`cluster_ops_db.saga_store` 9 张表：

- `saga_definition`（saga 类型 / 步骤编排 / 补偿规则 / 超时）
- `saga_instance`（saga_id / definition_id / state / 当前 step / 持有者 / fence_token / payload）
- `saga_step`（step_id / saga_id / participant / action / state / 锁定 fencing_token）
- `saga_event`（event_id / saga_id / type / payload / ts，append-only）
- `saga_command`（command_id / saga_id / step_id / idempotency_key / state / response）
- `saga_compensation`（compensation_id / saga_id / step_id / type / state）
- `saga_snapshot`（saga_id / state_json / ts，用于快速恢复）
- `saga_failure`（saga_id / step_id / reason / retry_count / next_retry_at）
- `saga_audit`（audit_id / saga_id / operator / action / before_state / after_state / ts）

### FR-104 Saga Event Journal 11 事件

append-only 记录：

- `SagaStarted` / `SagaCompleted` / `SagaFailed`
- `StepScheduled` / `StepSucceeded` / `StepFailed`
- `CommandPublished` / `RetryScheduled`
- `CompensationStarted` / `CompensationSucceeded`

### FR-105 Outbox + Inbox

每个 Rust 服务：

- 本地 DB 事务包含 `domain_update` + `outbox_event`，**一次 COMMIT**
- Outbox Worker 异步发布到 MQ
- Consumer 必须 Inbox 表保存 `event_id`，at-least-once + 幂等

### FR-106 Saga Runtime HA

至少 3 replicas（标准 profile）。多副本避免重复驱动：

- `SELECT ... FOR UPDATE SKIP LOCKED` 抢占 saga_instance
- Fencing Token 单调递增，过期 Leader 写入被拒
- 不依赖 distributed Redis lock

### FR-107 Pod Crash 恢复

支持的 4 状态：`RUNNING` / `WAITING` / `RETRYING` / `COMPENSATING`。
新 Pod 启动时：

1. 加载 `saga_instance.state IN ('RUNNING', 'WAITING', 'RETRYING', 'COMPENSATING')`
2. 恢复 journal（从最近 snapshot + append events）
3. 继续执行 / 重试 / 补偿

### FR-108 微服务升级兼容

- Saga Definition 版本化
- 升级前 old + new 双版本并行
- 在飞 Saga 用旧 Definition 跑完，新 Saga 走新 Definition
- 向后兼容 API（actor message schema 演化）

### FR-109 K3s Service Discovery

Saga Definition **不写 pod IP**，只引用 logical participant id：

```yaml
participants:
  - inventory-service        # k8s Service DNS
  - economy-service          # k8s Service DNS
  - mail-service             # k8s Service DNS
```

### FR-110 GM Saga Console

后台增加 `Saga Monitor App`：

- 列表：Running / Failed / Compensating / Completed / Manual Intervention
- 单 saga：Timeline / DAG / Retry / Pause / Resume / Manual Compensation
- 高风险操作：二次权限校验
- 查询 API：`GET /api/saga/{saga_id}/status`（admin RBAC 隔离）

---

## 5. 非功能需求（NFR）

### NFR-PT 性能

- 同步 Saga（无外部副作用）：**p95 < 200ms**（包含网络 + DB + MQ）
- 异步 Saga（含外部副作用）：**p95 < 2s**
- 实时游戏事件（不进 Saga）：**p99 < 50ms**

### NFR-AV 可用性

- Saga Runtime HA profile：**99.95%**（年停机 < 4.4 小时）
- 微服务：**99.9%**（年停机 < 8.8 小时）
- PostgreSQL HA：**99.95%**

### NFR-SC 扩展性

- Saga Runtime 水平扩展到 **≥ 10 replicas**（OCC 抢占保证无锁竞争）
- 单 Saga 步骤数：**1-20**（典型 3-7）
- 单 Saga 总执行时间：**≤ 30 分钟**（超时后强制 Compensation）

### NFR-OP 运营

- 单 SRE 维护（Saga Runtime + 5 域 + cluster_ops + shared-platform）
- 监控：Prometheus + Grafana 全指标
- 日志：Loki 聚合
- Trace：Tempo 存储
- 告警：saga_failed_total / outbox_backlog / inbox_duplicate_total / compensation_failed_total

### NFR-SE 安全

- GM 操作 100% 审计
- 跨服务调用 mTLS
- Secret 加密存储（sealed-secrets / external-secrets）
- NetworkPolicy 默认 deny + 按需 allow
- 操作 RBAC + 资源 ownership 校验

### NFR-OB 可观测性

- Metrics：saga_started_total / saga_completed_total / saga_failed_total / saga_compensation_total / saga_duration / saga_step_duration / saga_retry_total / outbox_backlog / inbox_duplicate_total / purchase_saga_failures / reward_pending / inventory_compensation_failures / currency_refund_failures
- Logs：所有日志携带 `saga_id` / `correlation_id` / `player_id` / `service` / `pod`
- Traces：OTel 端到端 + span links

### NFR-CO 兼容

- 向后兼容：Saga Definition v1 → v2 升级时 v1 实例可继续跑完
- 跨服：标准 Saga 可跨 k3s cluster（多 cluster federation）
- 协议：tonic gRPC / NATS JetStream 消息

### NFR-LIC 许可证

- 核心组件必须 Apache-2.0 / MIT / BSD
- **必须明确标注**许可证存在争议的项目（如 SSPL / BUSL / Elastic License / AGPL 需评估）

---

## 6. 假设

- 5 域（player / economy / match / social / admin）+ cluster_ops + shared-platform 已就位
- PostgreSQL 已就位（per DEC-009 18.6）
- K3s native in WSL2 已就位（per DEC-010）
- 现有 5 独立 DB 拓扑（player_db / economy_db / match_db / social_db / admin_db + cluster_ops_db）已遵守 ARC-008
- Docker / NATS JetStream 部署由 K3s Helm chart 负责

---

## 7. 约束

- **DEC-001~010 全部生效**（PFAU all-reachable / K8s 节点异常委托 / Active-Active / 5 域全开 / 5 域独立 Lead 撤销 / OLU 14-18 周 / OLU 双轨制 / 一人公司 / PG 18.6 / k3s native in WSL2）
- **不绑云厂商**（无 AWS / GCP / Azure 专有服务）
- **不绑 Redis Enterprise**（如需缓存，KeyDB / Dragonfly / PostgreSQL 替代）
- **不绑闭源 Saga 协调器**（自研 Rust Saga Runtime）
- **不绑闭源消息总线**（NATS JetStream 推荐 Apache-2.0）

---

## 8. 验收标准

### 8.1 业务验收

- [ ] 商城购买 Saga 在 Inventory grant 失败时自动释放 Currency 预留
- [ ] 角色创建 Saga 在 Inventory 失败时回滚 Economy + Mail
- [ ] 比赛奖励 Saga 在失败时进入 manual intervention（比赛不可回滚）
- [ ] GM 补偿礼包跨 Economy + Inventory + Mail 一致性 100%
- [ ] 跨服转移 Saga 在网络分区下不丢步骤
- [ ] 浏览器关闭时进行中的 Saga 100% 继续
- [ ] K3s Saga Runtime Pod crash 后新 Pod 接管在飞 Saga
- [ ] 多副本 Saga Runtime 不出现重复驱动同一 Saga

### 8.2 NFR 验收

- [ ] p95 同步 Saga < 200ms
- [ ] p95 异步 Saga < 2s
- [ ] Saga Runtime HA 99.95%
- [ ] 5 域微服务 99.9%
- [ ] 所有 GM 高风险操作 100% 审计
- [ ] 核心组件 License 100% OSI-compatible

### 8.3 自审清单（per spec 59）

- [ ] 无纯 UI 操作进入服务器
- [ ] 无单服务事务被错误升级为 Saga
- [ ] 无跨服务操作没有 Saga
- [ ] 无经济操作缺乏幂等
- [ ] 无可 Reservation 的地方却直接产生副作用
- [ ] 无 Saga 进入实时 Tick
- [ ] 无浏览器成为 durable transaction participant
- [ ] 无服务直接访问其他服务 DB
- [ ] 无依赖闭源组件
- [ ] K3s Pod crash 后无丢 Saga
- [ ] 无 Compensation 自身不可恢复
- [ ] GM 高风险操作 100% 审计

---

## 9. 必答 17 问题（spec 56）

1. **哪些后台操作完全不通信？** — UI Only（L0）/ Local Transaction（L1）：hover / focus / 面板布局 / 未提交表单 / 临时过滤器
2. **哪些操作 Local First？** — Admin Preferences（L2）：GM 主题 / Dashboard / 列宽 / 最近打开页面
3. **哪些操作只需要一个 Rust 微服务事务？** — Single-Service ACID：修改角色昵称 / 扣货币（无补偿）/ 加好友
4. **哪些操作才进入 Saga？** — 触发 BR-102 任意 1 条件
5. **Authority Boundary 如何定义？** — per FR-101 Authority Map（数据所有权 + 业务规则拥有者）
6. **谁拥有最终 Domain Truth？** — 写它的 Rust 服务（FR-100）
7. **前端本地 rollback 和 Saga Compensation 如何区分？** — 前端只 rollback 未提交 UI 状态；Saga 补偿已产生业务副作用
8. **Saga 如何避免进入实时游戏 Tick？** — BR-103 白名单
9. **Economy 如何保证不重复扣款？** — BR-105 幂等 + Inbox + idempotency_key
10. **Inventory 如何补偿已经发出的 Item？** — BR-106 Item Lineage + RevokeItem 语义补偿（处理已用/交易/出售/强化/拆解）
11. **Reservation 与 Compensation 如何选择？** — BR-104：能 Reserve 则 Reserve，不能才 Compensate
12. **K3s Pod crash 怎么恢复 Saga？** — FR-107 4 状态恢复 + snapshot + journal replay
13. **Saga Runtime 多副本怎么避免重复驱动？** — FR-106 PostgreSQL `SELECT FOR UPDATE SKIP LOCKED` + Fencing Token
14. **MQ 重复与乱序如何处理？** — Inbox 表 + event_id 幂等 + per-participant 顺序保证
15. **Outbox / Inbox 如何实现？** — FR-105 本地 ACID + 异步发布 + 幂等消费
16. **玩家跨服操作如何处理？** — Saga Definition 引用 logical participant id（K8s Service DNS），可跨 cluster 路由
17. **GM 高风险操作如何审计？** — BR-109 完整审计（operator / reason / before / after / 二次校验）
18. **浏览器关闭为什么不能影响 Saga？** — BR-108 Admin UI 不是 Coordinator，命令已发，Saga 继续
19. **微服务升级时运行中的 Saga 如何兼容？** — FR-108 Definition 版本化 + 双版本并行 + 旧实例跑完
20. **如何确保所有核心组件纯开源可商用？** — BR-111 License 矩阵 + 替代方案评估

---

## 10. 关联文档

- **基础设计**：`RGS-BAS-100_Saga事务系统基本设计书_v0.1.md`（下游）
- **详细设计**：
  - `RGS-DTL-100_Saga业务模式设计_v0.1.md`
  - `RGS-DTL-101_OperationPolicy与AuthorityBoundary设计_v0.1.md`
  - `RGS-DTL-102_Saga故障恢复设计_v0.1.md`
- **部署**：`RGS-OPS-100_Saga系统K3s部署设计_v0.1.md`
- **可观测性**：`RGS-GOBS-100_Saga可观测性设计_v0.1.md`
- **安全审计**：`RGS-SEC-100_GM审计与Saga安全设计_v0.1.md`
- **横向规范**：`RGS-SPEC-CROSS-001` 错误码 / `CROSS-002` gRPC / `CROSS-003` 跨域事件 / `CROSS-004` DTO / `CROSS-005` DB 命名 / `CROSS-006` trace_id / `CROSS-007` RBAC
- **现有架构决策**：
  - `RGS-ARC-008` 5 独立 DB 原则
  - `RGS-ARC-018` PFAU
  - `RGS-ARC-020` 拒动态库加载（沙箱仅 Rhai）
  - `RGS-ARC-021` 节点异常检测
  - `RGS-ARC-042` 中心事件管理（CEM）
  - `RGS-ADR-0052` Active-Active + all-reachable

---

## 11. v0.2 评估：saga-runtime 独立 Pod 落点 (per 2026-09-01 22:30 JST)

> **触发**：per BATCH REQ `RGS-BATCH-REQUIREMENTS-2026-09-01_v0.1.md` GAP-11
> **任务**：per WBS v0.2 桶 10 Phase D D5 (commit 84edf26)
> **关联**：per AGENTS.md v0.4 §7 batch 域派生约束 (12 约束) + WBS v0.2 §4.3 拍板 3

### 11.1 v0.1 状态 (per 2026-08-21 JST 初版)

- v0.1 不集成 saga-runtime (per RGS-BAS-100 v0.1 假设)
- saga-runtime 由 5 业务域 + cluster-ops 各自内嵌 (非独立服务)
- batch 域 v0.1 不触发跨域 saga (per BATCH REQ GAP-11)

### 11.2 v0.2 评估：saga-runtime 独立 Pod 落点

#### 方案 A: saga-runtime 独立 Pod (本评估推荐)

| 项 | 内容 |
|---|---|
| 部署形态 | 独立 Deployment + ClusterIP service (per 9/1 13:05 JST envoy 独立 deployment 偏好) |
| 端口 | 0.0.0.0:50057 (gRPC, 跟 5 域 50051-50055 + cluster-ops 50056 顺序) |
| 副本数 | 3 副本 (Active-Active per ADR-0052, **禁 HPA** 避免 all-reachable 漂移) |
| 持久化 | PostgreSQL (per RGS-ARC-008 5 独立 DB 原则 — saga 9 张表走独立 `saga_db`, **不与 5 域混**) |
| 消息总线 | NATS JetStream (per 8/31 OPEN-QA v0.2 Q7 social push_delivery 复用) |
| 触发方 | 5 域 (player / economy / match / social / admin) + cluster-ops + batch 域 |
| 监控 | OTel collector 复用 (per 桶 9 部署) |
| mTLS | 业务级 (per 5 域 ST 业务级 mTLS 实践 commit `401ac5c`) |

#### 方案 B: saga-runtime 嵌入 cluster-ops Pod (拒绝)

- 拒绝理由：cluster-ops 是 Active-Active 控制面，saga 是分布式事务协调，职责混合违反 ARC-008 单一职责
- 资源耦合：saga 9 张表 + saga runtime 状态机增加 cluster-ops 复杂度
- 扩展性差：saga 流量增长会冲击 cluster-ops Active-Active 状态

#### 方案 C: saga-runtime 完全外包 (e.g. Temporal / Apache Airflow) (拒绝)

- 拒绝理由：per v0.1 §7 约束 "不绑闭源 Saga 协调器" + "纯开源约束" (BR-111)
- 拒绝替代：自研 Rust Saga Runtime (per v0.1 §0)

### 11.3 决策项（待 Ulysses 拍板）

per AGENTS.md v0.4 §7 batch 域派生约束 (Mavis 不允许默认代签 batch 域决策):

1. **方案 A 整体拍板** — 独立 Pod / 副本数 3 / saga_db 独立 DB / NATS JetStream 复用 / 业务级 mTLS
2. **saga_db 拓扑决策** — 加进 ARC-008 5 独立 DB 原则 = 6 独立 DB (player / economy / match / social / admin / saga) + cluster_ops_db = 7 DB
3. **saga-runtime 资源上限** — CPU/Mem request+limit (per RGS-INC-002 §3 资源约束)
4. **batch 域 v0.2 触发 saga 范围** — 仅交易类 (购买/补偿) 或全场景 (per BATCH REQ §9 GAP-11)
5. **触发幂等键规范** — per FR-105 Outbox / Inbox + idempotency_key (复用 v0.1 §9 决策 15)
6. **跨域 saga vs 域内 saga 分类** — per FR-101 Authority Map (复用 v0.1 §9 决策 5)

### 11.4 已知缺口 (per v0.2 评估)

- ⏳ **saga-runtime 9 张表 schema 详细设计** — 待 DTL-100 v0.2 升版 (per WBS v0.2 桶 11 Phase E E1)
- ⏳ **saga 失败恢复 4 状态机详细设计** — 待 DTL-102 v0.2 升版
- ⏳ **saga_runtime + 5 域 gRPC 客户端代码生成** — 待 v0.2 实施
- ⏳ **saga 监控 + 告警阈值** — 待 v0.2 实施 (per RGS-GOBS-100)
- ⏳ **batch 域触发 saga 的 9 GAP 项 v0.2 评估** — per BATCH REQ §9 GAP-1~12
- ⏳ **Mavis 不能默认代签的决策** — 等 Ulysses 拍板 (per §11.3 6 项)

### 11.5 v0.2 → v0.3 推进路径

- 9/1 22:30 JST: v0.2 评估 commit 落档 (本 commit)
- 9/2-9/8: Ulysses 拍板 6 项决策项 (per §11.3)
- W37+ (per WBS v0.2 桶 11 Phase E): E1 BATCH IMPL-PLAN v0.2 + 38 L4 任务 W1-W6
- W42+ (per WBS v0.2 桶 11 Phase E E3): rgs-batch-console + rgs-batch-backend 项目初始化
- 9 月内: saga-runtime 独立 Pod 实装 (per BATCH PLAN v0.1 + WBS v0.2 桶 11)
