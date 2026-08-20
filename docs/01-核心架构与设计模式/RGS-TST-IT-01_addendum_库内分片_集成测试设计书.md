# 集成测试设计書（統合テスト設計書 / Integration Test Design Document）

**主题域 01 核心架构与设计模式 — 库内水平分片（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-01-ADD1 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-025-ADD1 v0.2 + RGS-DTL-022 v0.2 §3.1〜§3.2 |
| V模型层级 | TL-2 集成 / TL-3 契约 / TL-4 属性 |
| 制定日 | 2026-08-19 |

---

## 1. 目的

验证库内分片在跨服务 + DB 集成层级的行为。

## 2. 测试用例

| 用例 ID | 试验级别 | 测试目的 |
|---|---|---|
| TST-IT-01-S001 | [TL-2] | 4 logical shard 端到端：5 服务以 `jump_consistent_hash_v1` + routing_version 透明路由 |
| TST-IT-01-S002 | [TL-3] | shard_config gRPC 契约含 routing_version、hash_algorithm、唯一有序 active_shard_ids |
| TST-IT-01-S003 | [TL-2] | 同物理 DB 跨 shard 操作：购买 + 货币扣减以单 PostgreSQL 事务提交/回滚 |
| TST-IT-01-S004 | [TL-2] | PREPARE → DUAL_WRITE → VERIFY → CUTOVER → RETIRE 全程 0 中断（NFR-AV-007） |
| TST-IT-01-S005 | [TL-2] | 单主 shard 故障 → 仅同 shard 副本接管，其他 7 shard 继续服务 |
| TST-IT-01-S006 | [TL-4] | 1 → 4 → 8 shard 扩展比 ≥ 80% |
| TST-IT-01-S007 | [TL-3] | `jump_consistent_hash_v1` 路由与 routing_version schema 契约 |
| TST-IT-01-S008 | [TL-2] | 跨 shard 查询性能 p99 < 50ms |
| TST-IT-01-S009 | [TL-2] | sharding 与 ARC-013 死锁防止兼容 |
| TST-IT-01-S010 | [TL-2] | sharding 与 ARC-008 限界上下文一致 |

## 3. 追溯性

| 需求 | 用例 |
|---|---|
| FR-CAP-004~009 | TST-IT-01-S001~S008 |
| ARC-008/013 | TST-IT-01-S009~S010 |
| NFR-CAP-101~105 | TST-IT-01-S003~S008 |
| AC-CAP-101 | TST-IT-01-S001/S002/S007 |
| AC-CAP-102~105 | TST-IT-01-S003~S006 |

## 4. 通过判定

- 全部 PASS
- 1→4→8 扩展比 ≥ 80%
- shard 故障 ≤ 5% 流量影响
- 0 中断（NFR-AV-007）

---

> 与 RGS-TST-IT-01 共存。
