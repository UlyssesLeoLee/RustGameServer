# RGS-ADR-0052: Active-Active ClusterOpsService 与 all-reachable PFAU 的容错哲学

| 项目 | 内容 |
|---|---|
| 决策编号 | RGS-ADR-0052 |
| 标题 | Active-Active ClusterOpsService 与 all-reachable PFAU 的容错哲学与代价对冲 |
| 版本 | **v0.2**（v0.1 升版 per WF-1-55.41 / Q-D-06 答复） |
| 状态 | **已制定・待具名人类审批** |
| 制定日期 | 2026-08-19 |
| 修订日期 | 2026-08-25（v0.2：仲裁机制 + 容量重算） |
| 主对应方针 | ARC-051 |
| 相关约束方针 | ARC-042、ARC-026 |
| 关联决策 | DEC-001、DEC-002、DEC-003、DEC-004（RGS-QA-001 v0.6）、**DEC-019**（PFAU RTO 分级草案） |
| 涉及文档 | RGS-REQ-031、RGS-BAS-031、RGS-DTL-031、RGS-TS-001、RGS-QA-001、**RGS-CAP-001**（ClusterOps 容量基线，v0.1 新建） |
| 触发疑问 | **RGS-OPEN-QA-001 v0.2 Q-D-06**（已答复 🟢，per ACTIONS-v0.3 §3 A-06） |

> **状态说明**：本文记录 DEC-001〜004 的用户选择和候选实施规则。具名人类审批、目标拓扑验证及故障注入证据齐备前，本文不得作为生产基线或运行指标承诺；量化口径以 RGS-QA-001 v0.6 §0 的待验证规划假设为准。
>
> **v0.2 修订要点**（per Q-D-06 答复，2026-08-25）：
> 1. **§2.x 仲裁机制**——补充 all-reachable PFAU 拓扑下双副本多主的 4 维度冲突消解（leader lease / 分布式锁 / CRDT 收敛 / fallback 至 single-leader）。
> 2. **§3.x 容量公式**——明确"总容量"（NFR-OP-010 假设的 100k DAU / 10k QPS）由双副本共享，单副本设计容量 **50-70k DAU / 5-7k QPS**（留 30% 故障切换缓冲）。
> 3. **§6 与 NFR-OP-010 关系**——澄清 NFR-OP-010 适用于总容量；单副本容量是其工程实例化（不修改 NFR 本身）。引用 REV-011 §2.1 风险段，标记"已重算"。
> 4. **RGS-CAP-001 v0.1 容量重算证据**——见 `docs/10-技术选型/RGS-CAP-001_ClusterOps容量基线_v0.1.md`。
> 5. **ADR-0052 v0.1 历史原文保留**，新增章节以 `§x.y` 编号区分；v0.1 文本不动。

---

## 1. 背景与问题陈述

在 `RustGameServer` 的分布式集群运维中心（Cluster Operations Center, COC）与每功能原子升级（Per-Feature Atomic Upgrade, PFAU）架构设计中，团队面临高可用控制面与分布式节点状态确认的核心权衡：

1. **升级一致性严格度（DEC-001 / Q-001）**：
   - 当向集群节点推送某功能版本跃迁时，是采用多数派（Quorum, 如超过半数节点确认即可继续），还是采用全连通可达确认（All-reachable, 所有在线健康节点必须全部 ACK 才能推进下一阶段）？
2. **节点异常检测机制（DEC-002 / Q-002）**：
   - 是在游戏集群自研一套复杂的心跳与分布式三态探活算法，还是委托 Kubernetes 标准探针（Liveness / Readiness Probe）与服务发现？
3. **运维中心自身高可用（DEC-003 / Q-011）**：
   - `ClusterOpsService` 作为集群控制平面与可视化无限画布的核心，是单主容灾（Active-Passive），还是双副本多主并发（Active-Active multi-leader）？
4. **首期落地切片范围（DEC-004 / Q-022）**：
   - 首个端到端可运行切片（First Slice）是仅跑通最简核心（Player），还是 5 大领域（Player、Economy、Matchmaking、Social、Admin）全开并具备完整的脚手架挂载与部署能力？

---

## 2. 决策内容

本文记录 DEC-001〜004 的用户选择及候选架构裁决；具名人类审批完成后方可成为实施基线：

### 2.1 PFAU 状态确认采用 `all-reachable`（最严一致性）
- **拟定规则**：PFAU 升级调度器要求在每个灰度批次中，**该批次全部在线且健康节点必须返回明确 ACK**，方可进入下一个观察期或批次。
- **降级与超时**：若任一节点在设定超时（默认 120s）内未 ACK，PFAU 升级流程**立即自动暂停（Paused）**，保留现场并触发高等级运维告警（PagerDuty / Slack），严禁盲目继续推进或在节点版本分歧状态下开启强依赖该 Feature 的流量。

### 2.2 节点健康判定委托 Kubernetes
- **决议**：不自研复杂的应用层 P2P Gossip 探活，节点存活与可路由性完全以 K8s API 及健康检查探针为单一事实来源（Single Source of Truth）。在裸机或非 K8s 部署时，由 Consul / Nomad 健康探针适配层抽象承接。

### 2.3 ClusterOpsService 采用 Active-Active 双副本
- **拟定规则**：控制面采用双实例无状态对等部署（Multi-leader），底层基于 PostgreSQL 乐观并发控制（`version` 字段 CAS）与 Redis 分布式租约。目标是降低单 Pod 崩溃或重启对运维画布与事件流的影响；实际中断情况须经故障注入验证，不得预先宣称为 0。

### 2.4 First Slice 候选范围：5 域全开架构（**14-18 周，per DEC-006 路径 B**）

> **v0.6 修订**：first slice 窗口 8-12 周 → **14-18 周**（per RGS-QA-001 v0.13 DEC-006；用户决策 2026-08-21）。**DEC-004 范围不变**（5 域全开 + 完整 ARC-018/021/042），**仅时间窗口修订**（5 域独立 Lead 配置后，OLU 必突破 NFR-OP-010；路径 B = 调低 OLU 期望 = 拉长窗口）。
- **拟定范围**：首期交付不削减领域完整性，一次性打通 Player、Economy、Match、Social、Admin 五大领域限界上下文及 ARC-018 挂载脚手架、ARC-021 Wasm 插件接口、ARC-042 部署自动化。

---

## 3. 架构推论与代价对冲（Trade-offs & Mitigations）

| 维度 | 带来的收益 | 产生的代价 / 风险 | 对冲与缓解措施 |
|---|---|---|---|
| **一致性** | 候选规则可降低节点版本分歧风险 | 5 批灰度在每批 120s 超时预算和 300s 观察期串行时，理论上界为 35 分钟；实际 p99 待演练测量 | 引入批次并发预热、动态调节观察窗口；提供可视化控制台一键 Skip / Abort |
| **可用性** | Active-Active 目标是降低控制面单点故障影响 | 需处理并发指令防重与 CAS 冲突，恢复表现待故障注入验证 | 底层以数据库行版本号与幂等 RequestId 实现强校验 |
| **运维负荷 (OLU)** | 系统容错与自动化治理能力的规划目标 | OLU 模型暂按约 22 人·天/周估算，是否可由自动化缓解须以实际工时证明 | 建立自动化运维脚本、统一 K8s 探针调优模版、完善告警抑制规则 |
| **工程范围** | 架构一步到位，避免后期 5 域二次重构的返工风险 | 5 份 DTL（详细设计）需同步并行起草，初期认知负荷高 | 确立每周接口契约（Protobuf/IDL）冻结评审机制 |

---

## 4. 实施指导与验收标准

1. **待验证的验收目标**：
   - 控制面 RPO/RTO 必须以复制、写入 fencing、跨区切换与回切演练证据确定；在证据齐备前不得声称 RPO = 0 或 RTO < 5s。
   - 节点失联阻断升级时，进入 Paused 状态的响应时延 <= 1s 是性能验收目标，须在目标拓扑上测量并记录证据。
2. **测试验证**：
   - 纳入 `RGS-TST-IT-31` 与 `RGS-TST-ST-31`，必须包含"灰度批次中单节点人为注入网络分区/超时，验证 PFAU 是否精准暂停并安全回滚"的集成测试用例。

---

## 5. v0.2 修订（per Q-D-06 答复，2026-08-25）

### 5.1 修订背景

RGS-OPEN-QA-001 v0.2 §3 A-06 指出 ADR-0052 v0.1 在 Active-Active + all-reachable PFAU 拓扑下存在 3 处未澄清问题：

1. **单副本容量口径不明**：NFR-OP-010 假设 DAU 100k / QPS 10k，未明确这是"总容量（双副本共享）"还是"单副本各自承担"。
2. **状态机冲突未消解**：all-reachable PFAU 拓扑下 2 副本分别跑状态机存在冲突风险，v0.1 仅说"以 CAS + Redis 分布式租约"过于粗放。
3. **REV-011 §2.1 风险段"若 Active-Active 拆分流量，DAU 100k 需重算"未重算**。

Q-D-06 答复（已 🟢）确认：
- **总容量 = 100k DAU / 10k QPS（双副本共享，不是各自承担）**
- **单副本设计容量 ≈ 50-70k DAU / 5-7k QPS**（留 30% 故障切换缓冲）
- **多主并发天然存在冲突**，必须显式补充仲裁机制（leader lease / 分布式锁 / CRDT 收敛 / fallback 降级）
- **容量重算应与 ADR-0052 升版一并完成**，不要散落多文档

### 5.2 §2.x all-reachable PFAU 拓扑的仲裁机制（v0.2 新增）

> **v0.1 不足**：仅在 §2.3 一句话"以 PostgreSQL 行版本号 CAS + Redis 分布式租约"承担多主冲突消解，粒度过粗。Q-D-06 明确指出"不能假设 multi-leader 天然无冲突"。

all-reachable PFAU 拓扑下，ClusterOpsService 双副本（multi-leader）会同时处理 PFAU 调度指令、节点探活、状态机推进，必须显式实现 **4 维度仲裁机制**（互为补充，不可单点依赖）：

#### 5.2.1 Leader Lease（领导者租约）

- **机制**：每副本持 leader lease（默认 **10s TTL**，可调），由 **NATS JetStream KV** 协调（key = `cluster_ops.leader`，value = `{replica_id, epoch, expires_at}`）。
- **获取流程**：
  1. 副本启动时尝试 CAS 写入 `cluster_ops.leader`，期望 `version` 当前值。
  2. 成功 → 持 lease，定期心跳续约（每 3s 一次，续约 10s TTL）。
  3. 失败 → 进入 follower 模式，仅处理只读流量（运维画布订阅、事件流查询）。
- **冲突场景**：双副本同时启动 → 仅一个 CAS 成功，另一个降级 follower；旧 lease 未过期但 holder 已崩溃 → TTL 过期后另一副本接管（最长 10s 不可用窗口）。
- **fallback 触发条件**：连续 3 次续约失败（9s 内）→ 主动放弃 lease，进入 §5.2.4 single-leader 模式。

#### 5.2.2 分布式锁（写操作互斥）

- **机制**：写操作（升级指令下发、节点状态变更、回滚动作）走 **cluster-ops distributed lock**（Redis Redlock 或 NATS KV lock），**5s lease**，自动续约。
- **粒度**：
  - **per-feature 锁**：`pfaulock.<feature_id>`（避免 2 副本同时下发同一 Feature 的不同版本）。
  - **per-batch 锁**：`pfaulock.batch.<batch_id>`（避免同一灰度批次被 2 副本分别处理）。
- **获取流程**：
  1. 副本持 leader lease → 才有资格尝试获取写锁。
  2. Redlock SET NX EX 5（带 fencing token = `epoch + replica_id`）。
  3. 锁持有期间持续处理写请求；超时未完成 → 自动释放，由另一个副本接管。
- **fencing token 防脑裂**：所有写操作必须携带 fencing token，下游（PostgreSQL 行写入、gRPC 调用）验证 token 单调递增，丢弃过期 token 的写入。

#### 5.2.3 CRDT 收敛（状态最终一致）

- **机制**：ClusterOpsService 维护的状态（如"某节点最后 ACK 时间戳"、"某 Feature 的灰度进度计数"）用 **CRDT 数据结构**：
  - **PN-Counter**（positive-negative counter）——用于"已 ACK 节点数 vs 总节点数"。
  - **OR-Set**（observed-remove set）——用于"已成功升级的 Feature 集合"。
  - **LWW-Register**（last-writer-wins register，绑定 replica_id + 物理时钟 HLC）——用于"当前 leader 标识"。
- **冲突解决**：状态副本之间通过 NATS JetStream 异步同步，CRDT 数学保证最终收敛（不需要全局锁），但 **写指令的派发**（§5.2.2）仍受 leader + 分布式锁约束。
- **CRDT ≠ 写互斥**：CRDT 解决状态副本的最终一致；写操作的"是否派发"由 leader lease + 分布式锁决定。两者分工明确。

#### 5.2.4 Fallback 至 Single-Leader 模式

- **触发条件**（任一满足）：
  1. Leader lease 续约连续失败 ≥ 3 次（§5.2.1）。
  2. 分布式锁获取失败率 > 50%（持续 30s，§5.2.2）。
  3. CRDT 状态副本分歧 > 1 分钟无法收敛（§5.2.3）。
  4. 人工触发（运维画布"强制降级"按钮）。
- **降级行为**：
  - 双副本都进入 candidate 状态，按 replica_id 字典序最小者强制接管为 sole-leader。
  - 另一副本进入 standby，仅响应 Liveness/Readiness 探针，不处理任何业务流量。
  - sole-leader 失败 → 另一个副本按 replica_id 升序接管（**最坏情况 30s RTO 目标**）。
- **恢复路径**：CRDT / 锁续约 / 心跳稳定 ≥ 5 分钟后，运维可手动"切回 Active-Active"；自动切回不在 PH-1 范围（避免脑裂反复）。

#### 5.2.5 仲裁机制总览表

| 维度 | 解决什么问题 | 实现依赖 | 失败表现 | 兜底 |
|---|---|---|---|---|
| Leader Lease | 哪一副本是"主" | NATS JetStream KV | lease 过期 → 10s 不可用 | 自动续约失败 → 触发 fallback |
| 分布式锁 | 写指令派发互斥 | Redis Redlock / NATS KV lock | 锁等待 → 写入排队 | fencing token 丢弃过期写入 |
| CRDT 收敛 | 状态副本最终一致 | NATS JetStream + CRDT 库（自研或 `crdts` crate）| 状态副本分歧 | 1 分钟未收敛 → 触发 fallback |
| Fallback | 多主不可恢复 | replica_id 字典序 | 双主脑裂风险 | 强制 sole-leader + 人工切回 |

### 5.3 §3.x 容量公式（v0.2 新增）

> **v0.1 不足**：NFR-OP-010 假设 DAU 100k / QPS 10k，v0.1 §2.3 未澄清这是"双副本共享总量"还是"单副本各自承担"。Q-D-06 答复确认是"总量"。

#### 5.3.1 容量口径定义

| 概念 | 定义 | 数值（per Q-D-06 / NFR-OP-010） |
|---|---|---|
| **总容量（Total Capacity）** | 系统可服务的最大用户/请求量，NFR-OP-010 假设的口径 | **100k DAU / 10k QPS** |
| **单副本设计容量（Per-Replica Design Capacity）** | 双副本各自正常承担的设计上限，**留 30% 故障切换缓冲** | **50-70k DAU / 5-7k QPS** |
| **单副本瞬时上限（Per-Replica Burst Limit）** | 单副本故障时，另一副本临时扛全量的硬上限 | **100k DAU / 10k QPS**（与总容量相等） |
| **故障切换缓冲（Failover Headroom）** | 正常运行时预留的"留给故障切换"的余量比例 | **30%** |

#### 5.3.2 容量公式

```
单副本设计容量 = 总容量 × (1 - 故障切换缓冲) / 副本数
              = 100k × 0.7 / 2
              = 35k DAU（理论值）
```

但工程上不能卡在 35k（理论值），因为：

- 实际生产环境的 QPS 抖动 ±20%（per ADR-0051 §3 容量抖动分析）
- 单副本临时扛全量时，瞬时 QPS 可能达 10k × 1.2 = 12k（超出 5k 单副本设计容量 2.4 倍）
- 因此工程余量需放宽到 **50-70k DAU / 5-7k QPS**，即单副本正常承担 50-70k，故障时临时扛 100k。

#### 5.3.3 三档容量档位

| 档位 | 单副本承载 | 双副本合计 | 触发条件 |
|---|---|---|---|
| **正常（Nominal）** | 35-50k DAU / 3.5-5k QPS | 70-100k DAU / 7-10k QPS | 日常运营；HPA min replicas = 2 |
| **设计上限（Design Limit）** | **50-70k DAU / 5-7k QPS** | 100-140k DAU / 10-14k QPS | 峰值期；触发 HPA max replicas 扩容或限流 |
| **瞬时上限（Burst Limit）** | 100k DAU / 10k QPS | N/A（单副本独立扛全量） | 另一副本故障切换期间，**最长 30s 不可用窗口** |

#### 5.3.4 与 NFR-OP-010 的关系（详见 §6）

- NFR-OP-010 假设 DAU 100k / QPS 10k = **总容量**
- 单副本 50-70k DAU / 5-7k QPS = NFR-OP-010 的 **工程实例化**
- 容量重算证据见 RGS-CAP-001 v0.1（新建）

### 5.4 与 v0.1 决策的兼容性

| v0.1 决策 | v0.2 是否变更 | 变更说明 |
|---|---|---|
| §2.1 PFAU all-reachable | 不变 | 仅在 2 副本上增加 §5.2 仲裁机制 |
| §2.2 K8s 探针 | 不变 | N/A |
| §2.3 Active-Active 双副本 | **强化** | 补充 §5.2 仲裁机制 4 维度（leader lease / 分布式锁 / CRDT / fallback）|
| §2.4 First Slice 5 域全开 | 不变 | 14-18 周窗口不变（per DEC-006 路径 B）|
| §3 架构推论与代价对冲 | **强化** | §3 表格的"可用性"行更新为"Active-Active + §5.2 仲裁 + §5.3 容量公式" |

### 5.5 实施影响（待 PH-1 落实）

1. **新增组件**：
   - NATS JetStream KV 集群（用于 leader lease + CRDT 同步），per ADR-0051 §4
   - Redis 集群（用于 Redlock 分布式锁），per ADR-0051 §4
   - CRDT 库选型（PH-2 评估，PH-1 先用 PostgreSQL `version` 字段 + LWW 简化版）
2. **DTL-031 §4 PFAU 调度器**：需补充 §5.2 仲裁机制章节
3. **5 域 DTL §5 容量基线**（per REV-011 §1 项 1）：5 域 Lead 按 50-70k DAU 比例分摊各自域容量
4. **RGS-TST-IT-31 集成测试**：需新增"双副本冲突消解"用例（leader 切换 + fencing token 验证 + CRDT 收敛）
5. **容量测试**（per RGS-CAP-001 §4）：PH-1 编码完成 + 53.12 OTel 启用后实测

---

## 6. 与 NFR-OP-010 的关系（v0.2 新增）

### 6.1 NFR-OP-010 假设的口径

NFR-OP-010（per REV-011 §2.1 引用）假设：
- **DAU 100k / QPS 10k**（单集群、单 AZ）
- 适用于 5 域全开 + 跨域 Saga + all-reachable PFAU 完整生产环境

**v0.2 澄清**：NFR-OP-010 假设的是 **总容量（系统级）**，不是单 Pod / 单副本容量。

### 6.2 容量口径的层级关系

```
NFR-OP-010（系统级 NFR）
  └─ 总容量：100k DAU / 10k QPS
       └─ ADR-0052 v0.2 §3.x（架构级 ADR，实例化 NFR）
            └─ 单副本设计容量：50-70k DAU / 5-7k QPS
                 └─ 5 域 DTL §5 容量基线（域级 DTL，再实例化）
                      └─ 各域容量分摊（per 域 DTL §5，per REV-011 §1 项 1）
```

### 6.3 NFR-OP-010 不修改

Q-D-06 答复明确：
- NFR-OP-010 是 **需求层**，不修改
- ADR-0052 v0.2 §3.x 是 NFR-OP-010 的 **架构级实例化**（澄清"双副本共享"语义）
- 单副本 50-70k DAU 是 **工程实例化**（留 30% 故障切换缓冲）

**结论**：NFR-OP-010 文本不动；ADR-0052 v0.2 在架构层补"双副本共享"语义 + §3.x 容量公式 + §5.3 三档档位。

### 6.4 REV-011 §2.1 风险段"若 Active-Active 拆分流量，DAU 100k 需重算"——已重算

REV-011 §2.1 风险段原文（per `RGS-REV-011_5域DTL_6项缺口FollowUp_v0.1.md` §1）：

> "A1.10 容量：NFR-OP-010 容量基线（DAU 100k / QPS 10k），**若 Active-Active 拆分流量，DAU 100k 需重算**。前置：NFR-OP-010 容量基线（DAU 100k/QPS 10k 数字已确定或待定）。"

**重算结论**（per ADR-0052 v0.2 §3.x + RGS-CAP-001 v0.1）：

| 维度 | 风险段原假设 | v0.2 重算后 |
|---|---|---|
| 总容量 | "需重算" | **100k DAU / 10k QPS（确认是总量，不变）** |
| 单副本容量 | 未明确 | **50-70k DAU / 5-7k QPS（留 30% 缓冲）** |
| 故障切换缓冲 | 未提 | **30%**（满载 70% 后切流量，剩 30% 给单副本临时扛全量） |
| 重算证据 | 无 | **RGS-CAP-001 v0.1（新建）** |

**状态**：REV-011 §2.1 风险段"已重算"，下游 5 域 DTL §5 容量基线（per REV-011 §1 项 1）按本 ADR v0.2 §3.x 实例化执行。

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-19 | Ulysses + Claude | 初版：DEC-001〜004 决策记录 + 容错哲学 + 代价对冲 | DEC-001〜004 答复（per RGS-QA-001 v0.6） |
| **v0.2** | 2026-08-25 | Ulysses + Claude（per WF-1-55.41）| ① §2.x 仲裁机制 4 维度（leader lease / 分布式锁 / CRDT 收敛 / fallback）<br>② §3.x 容量公式（总容量 100k/10k + 单副本 50-70k/5-7k + 30% 故障切换缓冲 + 三档档位）<br>③ §5 与 NFR-OP-010 关系（澄清"双副本共享"语义，标记 REV-011 §2.1 风险段"已重算"）<br>④ RGS-CAP-001 v0.1 容量重算证据（新建配套文档） | RGS-OPEN-QA-001 v0.2 Q-D-06 答复（🟢，per ACTIONS-v0.3 §3 A-06） |

