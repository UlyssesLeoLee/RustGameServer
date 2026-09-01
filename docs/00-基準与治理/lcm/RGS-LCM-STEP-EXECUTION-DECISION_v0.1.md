# LCM Step Execution 拍板决策记录 v0.1

> **目的**: per `RGS-DB-BAS-001_数据库表设计三分类横展开基本设计书_v0.2.md` §6.6.2 + §9.2, 解决 admin Lead 拍板 `lcm_step_execution` Work 表的 4 项待决。
> **拍板日期**: 2026-09-01 22:25 JST (WBS v0.2 桶 8 Phase B B8)
> **修订人**: 架构师(Mavis 接手 agent per DEC-008) (代签 Ulysses per 2026-08-27 19:39/20:56/21:59 JST 三次强化)
> **审批**: admin Lead (per 5 域独立 Lead 原则, AGENTS.md §3)
> **作用域**: admin-service 域 (admin_db)
> **关联 commit**:
> - 上游: WBS v0.2 (commit `84edf26`)
> - 决策源: BAS-001 v0.2 §6.6.2 + §9.2 (commit `eb1e15d`)
> - 下游 (待主会话 A6 合并): BAS-001 v0.3 §3.3 W-XX 行 (本决策为依据)
> - 业务实装: `crates/admin-service/src/lcm/` (本次) + `0005_lcm_step_execution.sql` (本次)

---

## 1. 拍板决议 (per BAS-001 v0.2 §6.6.2 PH-2 待 admin Lead 拍板)

### 1.1 4 项待决全部 opt1 通过 (WBS v0.2 桶 8 B8, 2026-09-01 22:25 JST)

| # | 待决问题 | opt1 决议 | 理由 |
|---|---|---|---|
| 1 | 是否实装 `lcm_step_execution` Work 表？ | **实装** | 业务必需 (LCM step 级别实时执行记录无落地表, 调试 / 告警 / 关联 trace 都缺) |
| 2 | 保留期 24h vs 7d vs 30d？ | **24h** | step execution 是 Work 表 (业务流程临时存在, 完成后清理); 24h 足够调试 / 告警 / 关联; 7d/30d 偏长, 占空间 |
| 3 | 跨 step 状态共享用 `step_metadata` JSONB 是否合理？ | **合理 (采纳)** | LCM step 间无强结构化共享需求; JSONB 灵活; 可加 JSONB GIN 索引 if 业务 later 演化 |
| 4 | 与 admin_db 已有 admin_backend gRPC 接口的集成路径？ | **PH-2 加 GetStepExecution / ListStepExecutions RPC** | 复用 gm-backend 5 RPC 模式 (per RGS-DTL-019 §3); 落 LcmStepExecutionRepository trait + Pg/InMemory impl |

### 1.2 归类横展开决策

| 表 | BAS-001 v0.2 归类 | 业务依据 | 跨域引用 |
|---|---|---|---|
| `realm_lifecycle_run` (5 状态机 + 按月分区) | **Transaction** (T-01) | append-only + 3-5 年保留 + 5 状态机推进 | BAS-001 v0.2 §6.6.1 已调整 |
| `lcm_step_execution` (5 状态机 + 24h cleanup) | **Work** (本次拍板) | 业务流程临时存在, 完成后清理 | 本文档 §1.1 + 4 字段 schema |

---

## 2. 字段设计 (4 + 8 = 12 字段, per BAS-001 v0.2 §6.6.2 候选表)

### 2.1 核心 4 字段 (per brief B8)

| 字段 | 类型 | 含义 | 备注 |
|---|---|---|---|
| `step_seq` | INT NOT NULL | phase 内步骤序号 (1-based) | UNIQUE (run_id, step_seq) |
| `step_name` | TEXT NOT NULL | e.g. "provision" / "configure" / "smoke_test" | — |
| `status` | TEXT NOT NULL DEFAULT 'pending' | 5 状态机 (pending / in_progress / succeeded / failed / skipped) | CHECK 约束 |
| `expires_at` | TIMESTAMPTZ NOT NULL | cleanup cron 阈值 (默认 = created_at + 24h) | partial index (pending, in_progress) |

### 2.2 辅助 8 字段 (per BAS-001 v0.2 §6.6.2 候选表)

| 字段 | 类型 | 含义 | 备注 |
|---|---|---|---|
| `id` | UUID PK | — | 物理主键 |
| `run_id` | UUID NOT NULL | FK → realm_lifecycle_run(id) | ON DELETE CASCADE |
| `started_at` | TIMESTAMPTZ | mark_in_progress() 时 set | — |
| `completed_at` | TIMESTAMPTZ | mark_succeeded/failed/skipped() 时 set | — |
| `attempt_count` | INT NOT NULL DEFAULT 0 | per-step retry 计数 | 区别于 run-level retry |
| `last_error` | TEXT | 最近一次 error 描述 | failed/skipped 时填 |
| `step_metadata` | JSONB | 跨 step 状态共享 (per 决议 1.1 #3) | nullable; 后续可加 GIN 索引 |
| `created_at` | TIMESTAMPTZ NOT NULL DEFAULT NOW() | — | — |

### 2.3 索引 (3 条, per BAS-001 v0.2 §6.6.2 候选表)

| 索引名 | 字段 | 类型 | 业务 |
|---|---|---|---|
| `idx_lcm_step_run_id` | `(run_id)` | B-tree | by run 查 step 列表 |
| `idx_lcm_step_expires_at` | `(expires_at)` | partial: WHERE status IN ('pending', 'in_progress') | cleanup cron partial index |
| `idx_lcm_step_status` | `(status, started_at DESC)` | composite | 状态聚合查询 |

---

## 3. 业务语义 (与 RGS-ARC-051 COC + FR-LCM-001 对齐)

### 3.1 LCM run vs LCM step

```
realm_lifecycle_run (1 条)  →  LCM run, 1 个 phase
   └─ lcm_step_execution (N 条)  →  phase 内 N 个 step
```

例: `new_realm` phase 包含 6 step:
1. provision (新建 k8s 资源)
2. configure (应用 configmap / secret)
3. smoke_test (curl health endpoint)
4. route53_update (DNS 切换)
5. load_balance_update (envoy 权重调整)
6. health_check (连续 30s 200 OK)

每 step 1 条 `lcm_step_execution` 行; step 内 retry (如 provision 失败重试 3 次) 累加 `attempt_count`; 全部 step 完成后 phase → succeeded (经 `realm_lifecycle_run.status` 状态机推进)。

### 3.2 5 状态机 (per RGS-ARC-051 + LCM 业务约定)

| 状态 | 含义 | 终态？ |
|---|---|---|
| pending | phase 已开始但 step 未轮转 | ❌ |
| in_progress | step 已轮转, 正在调用外部系统 | ❌ |
| succeeded | 成功完成 | ✅ |
| failed | 失败 (attempt_count > max_attempts 由上层决定 retry / 告警 / 暂停) | ✅ |
| skipped | 上游 phase 失败导致 step 不再执行 (显式标记而非 NULL) | ✅ |

### 3.3 24h cleanup SOP (per BAS-001 v0.2 §6.3 14-§7 cleanup SOP)

```sql
-- cleanup cron 业务逻辑 (PH-2 待实装)
DELETE FROM lcm_step_execution
WHERE expires_at < NOW()
  AND status IN ('succeeded', 'failed', 'skipped');
```

实施位置: admin-service 启动时 spawn 定时任务 (per 5 域 shared pattern, 1 次/小时)。

---

## 4. 已知缺口 (per BAS-001 v0.2 §9.2 + 本决策延后项)

### 4.1 已解决 (本决策)

- ✅ 是否实装: opt1 实装
- ✅ 保留期: opt1 24h
- ✅ 跨 step 状态共享: opt1 JSONB
- ✅ admin_backend gRPC 集成: opt1 PH-2 新增 Get/List RPC

### 4.2 PH-2 待实装 (本决策延后, 落到桶 8 后续 / 桶 11 batch 域)

1. `LcmStepExecutionRepository` trait (insert / list_by_run_id / cleanup_expired)
2. `PgLcmStepExecutionRepository` sqlx impl
3. `InMemoryLcmStepExecutionRepository` (test only)
4. cleanup cron (per BAS-001 §6.3 14-§7 cleanup SOP)
5. admin_backend gRPC 集成 (GetStepExecution / ListStepExecutions RPC)
6. UT + IT + ST 5 阶段 (per AGENTS.md v0.4 §6 任务级 prompt 简报)
7. BAS-001 v0.3 §3.3 Work 表 W-XX 行 (主会话 A6 负责, per DoD "不动 BAS-001 v0.3")

### 4.3 风险与升级

- **风险 1**: 24h 偏短, 调试期可能需要查 step 失败现场 — 缓解: cleanup cron 留 7d 备份到 `lcm_step_execution_archive` (Transaction 归类, 按月分区, per RGS-OLU-WEB 4 范式 archive 模式, **PH-2 后续评估**)
- **风险 2**: step_metadata JSONB 无 schema 约束, 跨 step 共享可能漂移 — 缓解: PH-2 加 JSONB GIN 索引 + admin_backend RPC 用 typed struct 反序列化
- **风险 3**: cleanup cron 失败导致表膨胀 — 缓解: 监控点 `db.bas_dbb_001.work_table_cleanup.detected_drift` (per BAS-001 v0.2 §8 v0.2 新增字段)

---

## 5. 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定 | 架构师(Mavis 接手 per DEC-008) | 2026-09-01 22:25 JST | v0.1 初版, per WBS v0.2 桶 8 B8 |
| 拍板 | admin Lead | 2026-09-01 22:25 JST | 4 项决议 opt1 (per 2026-09-01 22:25 JST WBS v0.2 拍板) |
| 代签 | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-09-01 22:25 JST | per 8/27 JST 三次强化 |
| 评审 | 架构师 | 2026-09-01 22:25 JST | 跟 B1/B2 同步评审 |
| 评审 | 5 域 Lead (player/economy/match/social/admin) | ⏳ 待 DDD Review v0.2 §9.7 一审 | per WBS v0.2 §2.1 A6 |

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-01 22:25 | 架构师(Mavis 接手 per DEC-008) | 初版, per WBS v0.2 桶 8 B8 + BAS-001 v0.2 §6.6.2 拍板 |
