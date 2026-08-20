# 需求定义书（要件定義書 / Requirements Definition Document）

**库内水平分片 — 弹性容量规划（REQ-025）补强**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-025-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-025 弹性容量规划与超大规模并发架构 |
| 增补类别 | 新增 FR-CAP-004~009 详细需求 + ARC-040 实施细则 |
| 制定日 | 2026-08-19 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版。补强 RGS-REQ-025 库内水平分片章节 |
| 0.2 | 2026-08-20 | 架构师 | 统一为版本化 `jump_consistent_hash_v1` 路由契约；明确同库事务、五阶段 rebalance 和同 shard 副本故障切换；追溯至 RGS-DTL-022 v0.2。 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师 | 2026-08-19 | — |
| 审批 |  |  | 增补 FR-CAP-004~009 |

---

## 1. 前言（はじめに）

## 1.1 目的

RGS-REQ-025 v0.1 已规定"弹性容量规划"（ARC-040）的 T0~T3 容量分级与跨分片能力，但**未细化单库内水平分片（sharding）** 方案。本补强文档针对以下课题：

- 单库 PostgreSQL 在 T2 级别（100 万 CCU）单实例 CPU/IO 饱和
- 单表（如 `outbox`、`ledger`）超过 1 亿行后索引效率下降
- T3 级别（1000 万 CCU）单库物理扩展空间有限
- 跨库分片路由一致性已在 DTL-022 实现，但**单库内**水平分片策略缺失

## 1.2 与 ARC-040 关系

ARC-040（弹性容量规划）的"横向分片"原意是**跨库分片**（已实现）。本补强将其升级为**两层分片**：

```
Layer 1（库级 / 跨 DB）:    5 个限界上下文 = 5 个独立 DB     （已实现, RGS-DTL-001 §12）
Layer 2（库内 / 单 DB）:    单 DB 内 N 个 shard              （本补强新增）
Layer 3（行级 / 单 shard）: partition by date/hash          （已有部分，DTL-007 §5）
```

## 1.3 适用范围

| 范畴 | 说明 |
|---|---|
| 适用 | player_db / economy_db / match_db（3 个高写入库）的库内水平分片 |
| 不适用 | social_db（读多写少）、admin_db（数据量小）— 沿用单实例 |
| 评估期 | PH-5 启动（10k CCU 后压力评估） |

---

## 2. 业务需求

| ID | 需求 | 优先级 |
|---|---|---|
| BR-CAP-101 | 库内分片对应用层透明 | 高 |
| BR-CAP-102 | 单库内可扩展至 64 个 shard | 高 |
| BR-CAP-103 | 分片 rebalance 不停服 | 中 |
| BR-CAP-104 | 跨 shard 查询可表达 | 中 |
| BR-CAP-105 | shard 失败不影响其他 shard | 高 |

## 3. 功能需求

| ID | 需求 |
|---|---|
| FR-CAP-004 | 库内分片路由：以 `stable_hash_v1(player_id)` 输入 `jump_consistent_hash_v1`，在有序 `active_shard_ids` 中选择 `shard_id`；路由由 `routing_version` 固定，**禁止**以 `player_id % num_shards` 选 shard。 |
| FR-CAP-005 | 分片元数据管理：每个 DB 维护 `shard_config(routing_version, hash_algorithm, active_shard_ids, state)`；配置仅可经版本化 rebalance 生命周期变更，不得直接改写 `num_shards` 切流。 |
| FR-CAP-006 | 跨 shard 原子操作仅限**同一物理 DB**：在单个 PostgreSQL 事务/连接中统一 COMMIT 或 ROLLBACK；跨 DB、跨限界上下文以及 2PC/XA 均禁止。 |
| FR-CAP-007 | 跨 shard 查询：上层通过 `union_all_shards()` 聚合查询，下推到各 shard 并行 |
| FR-CAP-008 | Rebalance：新增或移除 shard 时必须按 PREPARE → DUAL_WRITE → VERIFY → CUTOVER → RETIRE 迁移；切换前旧 shard 仍可读，切换由 `routing_version` 原子生效。 |
| FR-CAP-009 | 故障隔离：单主 shard 故障只允许切换至**同一 shard 的副本**；不得把该路由改写到无同一数据归属的其他 shard，且不得级联影响其他 shard。 |

## 4. 非功能需求

| ID | 类别 | 目标值 |
|---|---|---|
| NFR-CAP-101 | 性能 | 跨 shard 查询延迟 p99 < 50ms（4 shard 聚合） |
| NFR-CAP-102 | 可用性 | 单主 shard 故障时其他 shard 的请求成功率不劣化；切至同 shard 副本后，5 分钟窗口内全局失败请求率 ≤ 5%。 |
| NFR-CAP-103 | 一致性 | 同一物理 DB 内跨 shard 操作必须在单个 PostgreSQL 事务中提交或回滚；跨 DB／跨限界上下文事务禁止。 |
| NFR-CAP-104 | 可扩展性 | 1 → 64 shard 线性扩展 ≥ 80%（NFR-PE-017） |
| NFR-CAP-105 | 运维 | shard rebalance 全程不停服（NFR-AV-007） |

## 5. 架构决定

| 编号 | 决定 |
|---|---|
| ARC-040-1 | 库内分片采用**应用层分片**（application-level sharding），不引入 Citus/pgcat 等中间件（避免 ARC-014 引入新中间件） |
| ARC-040-2 | 分片策略唯一采用 `jump_consistent_hash_v1(stable_hash_v1(player_id), active_shard_ids)`；`active_shard_ids` 必须有序且由 `routing_version` 标识（见 RGS-DTL-022 v0.2 §3.1）。 |
| ARC-040-3 | 跨 shard 原子操作仅在**同一物理 DB**内允许并使用单 PostgreSQL 事务；ARC-008 限界上下文间不跨 shard。 |
| ARC-040-4 | Rebalance 工具采用 PREPARE → DUAL_WRITE → VERIFY → CUTOVER → RETIRE，基于应用层幂等双写与校验；可使用 `COPY` 迁移存量，且不引入额外中间件。 |
| ARC-040-5 | PH-4 以 4 个 `active_shard_ids` 为容量目标、PH-8 以 8 个为目标；任何集合变更均须经版本化 rebalance 与实测审批，不得直接修改 shard 数。 |

## 6. 验收标准

| ID | 描述 |
|---|---|
| AC-CAP-101 | 路由入口仅调用 `jump_consistent_hash_v1`；lint 阻断直接以 `player_id % ...` 选 shard 或未声明 `routing_version` 的路由。 |
| AC-CAP-102 | 4 shard 跨 shard 查询 p99 < 50ms |
| AC-CAP-103 | PREPARE、DUAL_WRITE、VERIFY、CUTOVER、RETIRE 五阶段 rebalance 全程满足 NFR-AV-007 0 中断。 |
| AC-CAP-104 | 1 → 4 → 8 shard 扩展比 ≥ 80% |
| AC-CAP-105 | 注入单主 shard 故障后仅切至同 shard 副本；其他 shard 请求不受影响，且 5 分钟窗口内全局失败请求率 ≤ 5%。 |

## 7. 风险与未决

| ID | 内容 | 处理 |
|---|---|---|
| TBD-CAP-101 | 跨 shard 事务的性能开销 | PH-5 实测 |
| TBD-CAP-102 | rebalance 双写的幂等键、校验水位与保留时长 | PH-5 前以故障注入和迁移演练校准 |
| RSK-CAP-101 | 版本化路由配置或双写失配导致读写落在错误归属 | 通过共享路由库、配置校验、阶段门禁和可回退 `routing_version` 控制 |

---

> 本补强文档与 RGS-REQ-025 §5 共存，扩展其库内分片维度。详细实现见 RGS-DTL-022 v0.2 §3.1〜§3.2；测试见 RGS-TST-UT/IT/ST-01-ADD1。
