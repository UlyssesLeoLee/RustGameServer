# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 01 核心架构与设计模式 — 库内水平分片（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-01-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-025-ADD1 v0.2 + RGS-DTL-022 v0.2 §3.1〜§3.2 |
| V模型层级 | TL-6 负载 / TL-7 故障注入 |
| 制定日 | 2026-08-19 |

---

## 1. 目的

端到端验证库内分片在 T2/T3 容量级下的系统行为。

## 2. 测试用例

| 用例 ID | 试验级别 | 测试目的 |
|---|---|---|
| TST-ST-01-S001 | [TL-6] | T2 100 万 CCU 4 shard 跑通 |
| TST-ST-01-S002 | [TL-6] | T3 1000 万 CCU 8 shard 跑通 |
| TST-ST-01-S003 | [TL-7] | 1 主 shard 故障仅由同 shard 副本接管，其他 7 shard 继续服务 |
| TST-ST-01-S004 | [TL-7] | 五阶段 rebalance 期间 0 中断（AC-CAP-103） |
| TST-ST-01-S005 | [E2E] | 同物理 DB 内端到端 4 shard 跨 shard 事务原子性 |
| TST-ST-01-S006 | [E2E] | 跨 shard 业务查询性能 |
| TST-ST-01-S007 | [TL-6] | 1→4→8 shard 扩展比 ≥ 80% |
| TST-ST-01-S008 | [E2E] | AC-CAP-101 仅 `jump_consistent_hash_v1` 路由，无取模/无版本路由（lint 阻断） |
| TST-ST-01-S009 | [E2E] | AC-CAP-102 跨 shard 查询 p99 < 50ms |
| TST-ST-01-S010 | [E2E] | AC-CAP-104 线性扩展 |
| TST-ST-01-S011 | [E2E] | AC-CAP-105 故障隔离 |
| TST-ST-01-S012 | [E2E] | NFR-CAP-101~105 全部达标 |

## 3. 最小可复现实验

### 3.1 固定基线与取证规则

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | T2：4 个逻辑 shard，每个为 1 主 1 同 shard 副本（主/副均 16 vCPU、64 GiB RAM、NVMe）；T3：8 个逻辑 shard，每个为 1 主 1 同 shard 副本（32 vCPU、128 GiB RAM、NVMe）。每个服务至少 3 个实例，路由库与 `routing_version` 固定为待测构建。 |
| 数据集与负载模型 | 每 shard 至少 1,000,000 个稳定 `player_id`、10,000,000 条代表性 `ledger/outbox` 行；请求模型为 70% 单 key 读写、20% `union_all_shards()` 查询、10% 同物理 DB 双 shard 事务。T2 以 1,000,000 虚拟会话、T3 以 10,000,000 虚拟会话分布在负载发生器上。 |
| 预热与持续时间 | 所有负载用例先预热 30 分钟，再持续 60 分钟；扩展比用例在 1／4／8 shard 各独立运行 60 分钟，环境重置后再进入下一档。 |
| 故障注入 | 主 shard 故障使用进程终止和 60 秒网络隔离两种方式；rebalance 在稳定负载中触发 PREPARE → DUAL_WRITE → VERIFY → CUTOVER → RETIRE。故障注入时间、目标 shard 与恢复时间必须写入事件日志。 |
| 采样/SLO计算 | 每请求记录开始/结束、routing_version、logical shard ID、结果码和 HDR histogram。p99 为预热后每 1 分钟窗口的**最差** p99；失败率为窗口内失败请求/启动请求。扩展比为 `throughput_n/(n × throughput_1)` 的最差稳定窗口值。 |
| 原始证据路径 | `artifacts/test-results/TST-ST-01-ADD1/<run-id>/<case-id>/{topology.yaml,dataset.json,load.hdr,requests.parquet,events.jsonl,summary.json}`；`summary.json` 必须写入镜像 digest、配置哈希和起止时间。 |
| 清理步骤 | 导出并校验上述原始证据后，关闭负载发生器，撤销故障注入，等待副本追平，恢复基线 `routing_version`，销毁临时 shard/数据集与凭据；不得删除证据目录。 |

### 3.2 用例执行矩阵与可判定预期

| 用例 | 拓扑、数据与负载 | 预热/持续与故障注入 | 可判定预期 |
|---|---|---|---|
| S001 | T2 固定基线，4 shard、1,000,000 虚拟会话 | 30m/60m，无故障 | 100% 请求的 `(player_id, routing_version)` 映射与 `jump_consistent_hash_v1` 计算一致，服务无重启。 |
| S002 | T3 固定基线，8 shard、10,000,000 虚拟会话 | 30m/60m，无故障 | 100% 请求的版本化路由一致；所有 shard 健康检查在测试结束时为健康。 |
| S003 | T3 固定基线 | 稳定负载第 30 分钟终止一个主 shard，并隔离 60s | 仅同 shard 副本接管；其余 7 shard 的失败率不高于注入前基线，5 分钟窗口全局失败率 ≤ 5%。 |
| S004 | T2 固定基线，候选集合 4 → 8 | 稳定负载中执行五阶段 rebalance | 每阶段均有事件记录；因 rebalance 导致的已接受请求失败数为 0，且仅 CUTOVER 改变 routing_version。 |
| S005 | T2 固定基线，10% 双 shard 事务 | 30m/60m，无故障 | 每个事务要么两 shard 全部提交，要么全部回滚；不出现单边提交。 |
| S006 | T2 固定基线，20% 聚合查询 | 30m/60m，无故障 | 预热后所有 1 分钟窗口的跨 shard 查询 p99 < 50ms。 |
| S007 | 1／4／8 shard 三档固定数据密度与请求模型 | 各档 30m/60m，无故障 | 4、8 shard 的线性扩展比均 ≥ 80%。 |
| S008 | 任一基线构建的路由调用点 | 构建前静态 lint | `player_id % ...` 选 shard和未声明 routing_version 的路由命中数均为 0。 |
| S009 | 同 S006 | 30m/60m，无故障 | `union_all_shards()` 业务查询的最差 1 分钟窗口 p99 < 50ms。 |
| S010 | 同 S007 | 各档 30m/60m，无故障 | 复算的吞吐扩展比 ≥ 80%，且配置变更只通过 rebalance 发生。 |
| S011 | 同 S003 | 主 shard 进程终止与网络隔离各一次 | 其他逻辑 shard 的映射不变；全局失败率 ≤ 5%，无跨逻辑 shard 改路由。 |
| S012 | 汇总 S003/S004/S006/S007 的原始证据 | 完整试验结束后离线复算 | NFR-CAP-101〜105 的相应阈值全部满足；任一缺失原始证据或超阈即失败。 |

## 4. 追溯性

| AC | 用例 |
|---|---|
| AC-CAP-101 | TST-ST-01-S008 |
| AC-CAP-102 | TST-ST-01-S009 |
| AC-CAP-103 | TST-ST-01-S004 |
| AC-CAP-104 | TST-ST-01-S007/S010 |
| AC-CAP-105 | TST-ST-01-S003/S011 |
| NFR-CAP-101~105 | TST-ST-01-S012 |

## 5. 通过判定

- AC-CAP-101~105 全部通过
- 1000 万 CCU 8 shard 全 NFR 达标
- 0 高优事故

---

> 与 RGS-TST-ST-01 共存。
