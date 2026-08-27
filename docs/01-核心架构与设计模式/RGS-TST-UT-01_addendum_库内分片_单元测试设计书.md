# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 01 核心架构与设计模式 — 库内水平分片（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-01-ADD1 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-025-ADD1 v0.2 + RGS-DTL-022 v0.3 §3.1〜§3.2 |
| V模型层级 | TL-1 单元试验 ↔ DTL 详细设计 |
| 制定日 | 2026-08-19 |
| 升版日 | 2026-08-27 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.2 | 2026-08-20 | 架构师 | 整合入仓即此版（git 实证 2bb4a77 "feat(docs): 整合 74 份 RGS 文档入仓"，无中间内容变更）。整合前上游文本已为 0.2，文件头标注与正文一致。 |
| 0.3 | 2026-08-27 | 架构师（Mavis 接手 agent per DEC-008） | 同步父 DTL-022 v0.2→v0.3 元数据层（git 实证 b8c8598）：补 `state` 五状态字段、加载时算法名不匹配/未声明 routing_version 拒绝路径、状态机非法跳跃拒绝、阶段转换审计、RETIRE 只读窗口 5 项用例。RGS-REQ-025-ADD1 8/19-8/27 无内容变更（git 实证）；ARC-040 在 repo 无独立 ADR（ARC-040-1~5 为 ADD1 §5 决定编号系列，v0.2 S015~S018 已覆盖）。 |

---

## 1. 目的

覆盖 RGS-REQ-025-ADD1 库内水平分片（FR-CAP-004~009）的函数/类型级正确性。

## 2. 测试用例

| 用例 ID | 对应 FR/字段 | 测试目的 | 覆盖类型 |
|---|---|---|---|
| TST-UT-01-S001 | FR-CAP-004 `jump_consistent_hash_v1` | 同 player_id、routing_version、active_shard_ids → 同 shard | P |
| TST-UT-01-S002 | FR-CAP-004 | 候选集合追加 shard 时仅预期子集重映射；旧 routing_version 保持旧映射 | P |
| TST-UT-01-S003 | FR-CAP-005 shard_config | active_shard_ids 1 → 64 合法、唯一有序，routing_version 单调递增 | N |
| TST-UT-01-S004 | FR-CAP-005 | 空、重复或非升序 active_shard_ids 拒绝 | A |
| TST-UT-01-S005 | FR-CAP-005 | 直接改写路由集合拒绝；仅五阶段 rebalance 可生效 | N |
| TST-UT-01-S006 | FR-CAP-006 跨 shard 事务 | 同物理 DB 的单连接事务成功 → COMMIT 全成 | N |
| TST-UT-01-S007 | FR-CAP-006 | 任一子事务失败 → 全部 ROLLBACK | A |
| TST-UT-01-S008 | FR-CAP-006 | 子事务超时 → 全部 ROLLBACK + 告警 | A |
| TST-UT-01-S009 | FR-CAP-007 union_all_shards | 4 shard 聚合返回统一结果 | N |
| TST-UT-01-S010 | FR-CAP-007 | 跨 shard 排序正确 | N |
| TST-UT-01-S011 | FR-CAP-008 Rebalance | PREPARE → DUAL_WRITE → VERIFY → CUTOVER → RETIRE，且仅 CUTOVER 改 routing_version | N |
| TST-UT-01-S012 | FR-CAP-008 | rebalance 双写以幂等操作 ID 收敛，校验水位达标后才 CUTOVER | P |
| TST-UT-01-S013 | FR-CAP-009 故障隔离 | 单主 shard 断开 → 仅切同 shard 副本 | A |
| TST-UT-01-S014 | FR-CAP-009 | shard 断开不级联 | A |
| TST-UT-01-S015 | ARC-040-1 应用层分片 | lint 阻断任意 `player_id % ...` 路由及未声明 routing_version 的选 shard | A |
| TST-UT-01-S016 | ARC-040-2 hash 策略 | `jump_consistent_hash_v1` 与跨 DB 路由共用同一实现 | N |
| TST-UT-01-S017 | ARC-040-3 跨 shard 限制 | 跨 DB 不允许跨 shard 事务 | A |
| TST-UT-01-S018 | ARC-040-2 版本 | shard_config 含 routing_version、hash_algorithm、active_shard_ids | N |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CAP-004~009 | TST-UT-01-S001~S014 |
| ARC-040-1~5 | TST-UT-01-S015~S018 |
| AC-CAP-101 | TST-UT-01-S001/S015/S016 |
| AC-CAP-103 | TST-UT-01-S011/S012 |
| AC-CAP-104 | TST-UT-01-S002/S003/S005 |
| AC-CAP-105 | TST-UT-01-S013/S014 |

## 4. 通过判定

- 全部 PASS
- proptest 1000 次无失败
- ARC-040-1 lint 0 命中
- shard_config 字段 100% 一致

---

> 与 RGS-TST-UT-01 主题 01 单元测试设计书共存，本补强文档仅覆盖库内分片。

---

## 5. v0.3 增量（per RGS-DTL-022 v0.3 元数据层升版）

**覆盖源**：DTL-022 v0.3 §3.1 表（`state` 字段约束 + 配置加载时拒绝条件）+ §3.2 状态机表 + §3.2 末段（阶段转换审计）。RGS-REQ-025-ADD1 8/19-8/27 窗口无内容变更（git 实证），故 v0.3 不引入新 FR-CAP 条目，仅在 DTL-022 v0.3 落实的可测约束上扩展。

| 用例 ID | 对应 DTL-022 v0.3 条款 | 测试目的 | 覆盖类型 | 优先级 |
|---|---|---|---|---|
| TST-UT-01-S019 | §3.1 表 `state` 字段 + §3.2 状态机 | `shard_config.state` 取值仅限 `PREPARE`／`DUAL_WRITE`／`VERIFY`／`CUTOVER`／`RETIRE` 五者之一；构造器/反序列化器均校验 | N | P1 |
| TST-UT-01-S020 | §3.1 表 `state` 字段约束 | `state` 取非法值（`UNKNOWN`／空串／大小写变体）→ 拒绝并返回配置加载错误 | A | P1 |
| TST-UT-01-S021 | §3.1 末段"算法名不匹配必须拒绝并告警" | 配置加载时 `hash_algorithm` ≠ `jump_consistent_hash_v1`（如泛称 `consistent_hash`）→ 加载器拒绝、告警、不进入热路径 | A | P1 |
| TST-UT-01-S022 | §3.1 末段"请求未携带已声明的 `routing_version` 必须拒绝并告警" | 路由调用方未声明 `routing_version`（或版本不在加载器已知集合）→ 路由器拒绝并告警 | A | P1 |
| TST-UT-01-S023 | §3.2 状态机表（阶段顺序） | 状态机仅接受 `PREPARE→DUAL_WRITE→VERIFY→CUTOVER→RETIRE` 单向顺序；非法跳跃（如 `PREPARE→CUTOVER`）必须拒绝并审计 | A | P1 |
| TST-UT-01-S024 | §3.2 末段"每次阶段转换必须记录 `routing_version`／迁移水位／校验结果／操作者" | 任意成功状态转换均产出审计记录，字段齐备且不可缺；缺字段的转换视为未发生 | N | P2 |
| TST-UT-01-S025 | §3.2 状态机表 `RETIRE` 行"保留窗口结束且回退门禁关闭后才清理旧数据" | `RETIRE` 状态下旧副本仅可读、写入返回 `ShardReadOnly`；窗口外清理被允许并记录审计 | A | P2 |

**已知缺口（DDD Review 必查）**：

- **未引入新 FR-CAP 条目**：RGS-REQ-025-ADD1 8/19-8/27 窗口无内容变更（git 实证），v0.3 仅在 DTL-022 v0.3 落实的可测约束上扩展；如未来 ADD1 升版引入新 FR，本 addendum 应升 v0.4 对齐。
- **未覆盖 §6.1 `sync_sequence_no` 与 BAS-022 v0.2 升版条目（FR-CAP-012 / FR-CAP-030）**：DTL-022 v0.3 §6.1 属"插件分片同步协议"范畴归口 RGS-BAS-005 / 插件域 UT；BAS-022 §3.3 在 DTL-022 v0.3 §0 已声明"属不越权设计原则覆盖范围，本 DTL 不展开具体协议，仅在追溯性表新增一行明确归属"——本 addendum 同样不展开。
- **ARC-040 在本 repo 内无独立 ADR 文件**：ARC-040-1~5 是 RGS-REQ-025-ADD1 §5 决定编号系列，已在 v0.2 S015~S018 覆盖；如未来 ARC-040 升为独立 ADR 且引入新约束，本 addendum 应升版对齐。

> §1-§4 维持 v0.2 原文以保追溯连续性（per 2026-08-26 治理："缺标比错标安全"+"历史文档保留不追溯改写"）。
