# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 01 核心架构与设计模式 — 库内水平分片（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-01-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-025-ADD1 v0.2 + RGS-DTL-022 v0.2 §3.1〜§3.2 |
| V模型层级 | TL-6 负载 / TL-7 故障注入 |
| 制定日 | 2026-08-19 |
| 修订日 (v0.2) | 2026-09-07 12:35 JST |
| 修订者 (v0.2) | Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化) |
| 基线 v0.2 | RGS-TEST-DESIGN-2026-09-07 v0.2 (commit 583ce9e) + RGS-TEST-CASES-2026-09-07 v0.2 (commit 3ce36f0) |
| cherry-pick 关联 | a9a5363 (main, 2026-09-07, 2 file 684 insertions) |
| 状态 | ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程) |

## 0. 修订历史

| 版本 | 修订日 (JST) | 修订者 | 状态 | 摘要 |
|---|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | ⏳ DDD Review 二审 | 初版制定 |
| **v0.2 升版** | 2026-09-07 12:35 | Ulysses — Mavis 接手 | ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程) | 综合 8 维度升版 (8 域扩展 / batch v0.1 / admin-coc / plugin 架构 / flash-mock v0.3 / 9 域 mTLS / REQ-BDD-DDD v0.2 / cutover L15-L23). **库内分片 addendum ST 视角核心增量 = 8 域扩展后的库内分片调整端到端** (per 9/6 8 域扩展 a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673): 5 域 13 域 T2/T3 容量级 shard 端到端验证, batch 域 (per 9/1 fd122f6) T2 100 万 CCU 4 shard 跑通 + T3 1000 万 CCU 8 shard 跑通, plugin 集群 (per 9/5 61cf306) function-registry 端到端加入分片. git mv v0.1 → v0.2 保留 rename history |

## 0.1 派生约束守护段 (v0.2 增补)

- L1 (per AGENTS.md §2.1): ✅ 文档类工作 N/A
- L11 (cargo build dir lock): N/A 文档类
- L12.1 (临时 log 不入 commit): ✅ 0 临时 log
- L12.2 (5 worker 派工 3 选项): ✅ per 9/3 12:36 JST 选项 2 (主会话统一 commit)
- L13 (自指字段 deferred 实时查询): ✅ v0.2 全文 8 维度 commit SHA 实证
- L14 (plumbing 节点字符串 brace 跟踪): N/A 文档类
- B3 (DDD Review 二审流程 per 9/2 10:18 JST 拍板): ⏳ v0.2 Mavis 自审 1 次停手 → Ulysses 二审必到
- 9/1 batch 域 12 派生约束 (per AGENTS.md §7.2): ✅ §2 batch 用例对齐
- 9/1 14:58 JST 拍板选项规则: ✅ 8 维度拍板走 ask_user
- 9/4 17:47 JST 测试脚本+数据归入 mock 项目: ✅ §2 rgs-flash-mock 全过
- 9/5 04:03 JST 拍板后立即执行: ✅ 拍板后直接落地
- cutover L15-L23 (per 9/6 6c6839e): ✅ §2 全文引用
- 凭据永不打印 (per 8/27 11:06 JST + REDACTED filter): ✅ 全文 0 env value
- 缺标比错标 (per 8/26 JST): ✅ §0.2 已知缺口 2 项显式列
- 不追溯改写 (per 8/27 JST + 8/26 JST DTL-036): ✅ v0.1 → v0.2 显式升版, 不 amend 历史
- Mavis 默认代签 Ulysses (per 8/27 19:39/20:56/21:59 JST): ✅ author / 审批 / 修订人 三行齐全

## 0.2 已知缺口 (per 8/26 JST 缺标比错标, v0.2 显式列 2 项)

1. **8 域扩展后库内分片路由调整详细 WBS**: v0.2 §2 提及, 但 8 域扩展 (scene / battle / network / account / sub8) 后分片路由详细 WBS 5-10 task 待 DDD Review 阶段补全 (per 9/5 4 阶段 4-6 周估算)
2. **batch 域分片路由详细 yaml**: per 9/1 batch 4 件套 16 张表, 跨 batch DAG / WebSocket / 流式 GAP 待 DDD Review 阶段补

---

## 1. 目的

端到端验证库内分片在 T2/T3 容量级下的系统行为。

## 2. 测试用例

| 用例 ID | 試験レベル | テスト目的 |
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
| **TST-ST-01-S013 (v0.2 新增)** | [TL-6] | 8 域扩展 (scene / battle / network / account / sub8) shard 端到端 (per 9/6 8 域扩展) |
| **TST-ST-01-S014 (v0.2 新增)** | [TL-6] | batch 域 (per 9/1 fd122f6) shard 端到端: T2 100 万 CCU 4 shard + T3 1000 万 CCU 8 shard 跑通, 含 16 张表 (per 9/1 18:30 JST DB 三分类) |
| **TST-ST-01-S015 (v0.2 新增)** | [E2E] | plugin 集群 (per 9/5 61cf306) function-registry 端到端加入分片: 4 阶段 plugin-registry shard 路由 |
| **TST-ST-01-S016 (v0.2 新增)** | [TL-7] | admin-coc §X (per 9/5 ae9702d) shard 端到端: 7 项 admin 域 Lead 真实签字后 admin-coc shard 路由故障隔离 |
| **TST-ST-01-S017 (v0.2 新增)** | [E2E] | 9 域 mTLS 业务级 (per 9/6 d270ab9) shard 端到端: 9 域 mTLS shard 路由, 11 步客户端模拟器 v3 |

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
| **8 域扩展 (per 9/6 a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673)** | **TST-ST-01-S013** |
| **batch v0.1 (per 9/1 fd122f6/e70ed71/e366ff8/62027c9/eb1e15d)** | **TST-ST-01-S014** |
| **plugin 集群 (per 9/5 61cf306/f785f18/5cfd692)** | **TST-ST-01-S015** |
| **admin-coc Phase B (per 9/5 ae9702d/6c2a786/ab127e4/3695f3b)** | **TST-ST-01-S016** |
| **9 域 mTLS 业务级 (per 9/6 d270ab9/d15a0bb)** | **TST-ST-01-S017** |

## 5. 通过判定

- AC-CAP-101~105 全部通过
- 1000 万 CCU 8 shard 全 NFR 达标
- 0 高优事故

---

> 与 RGS-TST-ST-01 共存。
