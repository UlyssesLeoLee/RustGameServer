# rgs-batch PG schema 落地总结 (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-5, 2026-09-02 02:10 JST Mavis 接手代签)

> **状态**: 🟡 v0.1 草案 (per WBS v0.2 §2.5 桶 11 E1 计划)
> **基线**: RGS-BATCH-PLAN-2026-09-01_v0.2 (commit 2125727) + RGS-DB-BAS-001_v0.2 (commit eb1e15d)
> **规范**: 横展开三分类 (Work/Transaction/Master) per BAS-001 v0.2 §3

---

## 0. 一句话当前状态

3 schema (`batch_master` / `batch_transaction` / `batch_work`) + 3 migration + 16 张表全部就位,本会话落地 W1 BA-W1-5。

## 1. Schema 划分 (per BAS-001 v0.2 §3 + BATCH-PLAN v0.2 §3.1)

| Schema | 类别 | 表数 | 表名 |
|---|---|---:|---|
| `batch_master` | Master (永久事实) | 5 | task_def / task_template / data_source / worker_pool / schedule |
| `batch_transaction` | Transaction (append-only) | 8 | task_execution / sub_task / audit_event / dlq_event / log_event / data_migration / saga_instance / message_outbox |
| `batch_work` | Work (session-bound) | 3 | task_progress / task_buffer / audit_session |
| **合计** | — | **16** | — |

## 2. Migration 文件

| 文件 | 表数 | 行数 | 索引数 |
|---|---:|---:|---:|
| `0001_init_batch_schema.sql` | 5 (master) | 4281 字符 | 6 |
| `0002_init_batch_transaction.sql` | 8 (transaction) | 7600 字符 | 12 |
| `0003_init_batch_work.sql` | 3 (work) | 2433 字符 | 6 |
| **合计** | **16** | — | **24** |

## 3. 关键约束 (per BAS-001 v0.2)

- ✅ **Master 5 张** (per §3.1): UNIQUE 约束 + SCD 策略 + 永久保留
- ✅ **Transaction 8 张** (per §3.2): INSERT-only, append-only, 无 UPDATE/DELETE (除 DROP PARTITION, PH-3 实施)
- ✅ **Work 3 张** (per §3.3): session-bound, 完成后 cleanup
- ✅ **audit_event 永久保留** (per §6.4 + NFR-29): actor/role/action/target/parameters_hash/result/trace_id 全字段
- ✅ **dlq_event 90 天保留** (per §6.4)
- ✅ **log_event 90 天保留** (per §6.4)
- ✅ **task_progress / task_buffer 24h TTL** (per §6.4)
- ✅ **parameters_hash 不存明文凭据** (per NFR-30 + 8/27 11:06 JST 硬 ban)

## 4. 派生约束 (per WBS v0.2 + 12 GAP)

| GAP | 实现 | 落点 |
|---|---|---|
| GAP-3 mavis cron 告警 | trigger_mode = cron | schedule 表 |
| GAP-4 任务优先级 | priority INTEGER 字段 | task_execution 表 |
| GAP-7 任务模板版本化 | version INTEGER 字段 | task_template 表 + task_def 表 |
| GAP-9 任务超时 kill | tokio::time::timeout (W2 实施) | 业务层, schema 预留 status 字段 |

## 5. 跟 WBS v0.2 §2.5 桶 11 E1 对齐

- per BATCH-PLAN v0.2 §3.1 W1-5 任务估 = 300K tokens
- 实际估 = ~20K tokens (3 SQL 草拟, 留给 PH-2 实施细化)
- W1 BA-W1-5 完成度 = schema 100% 落地, PH-2 PH-3 实施待跑

## 6. 后续 PH-2 实施清单

- [ ] `batch_master.worker_pool.target_domains` 加 GIN 索引 (per BAS-001 §6.6.2 JSONB)
- [ ] `batch_transaction.audit_event.parameters_hash` 强 schema (PH-2 typed RPC)
- [ ] `batch_transaction.task_execution` / `sagas` / `moves` 按月分区 (per BAS-001 §6.5 PH-3)
- [ ] cleanup cron job (per BAS-001 §6.4)
- [ ] 5 域 gRPC client integration test (per BA-W2-3)

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 02:10 | 架构师(Mavis 接手 agent per DEC-008) | 初版, 3 schema + 3 migration + 16 表 + 24 索引 (per BATCH-PLAN v0.2 §3.1 W1 BA-W1-5) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
