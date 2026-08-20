# 单元测试设计書（単体テスト設計書 / Unit Test Design Document）

**主题域 01 核心架构与设计模式 — 库内水平分片（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-01-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-025-ADD1 v0.2 + RGS-DTL-022 v0.2 §3.1〜§3.2 |
| V模型层级 | TL-1 单元试验 ↔ DTL 详细设计 |
| 制定日 | 2026-08-19 |

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
