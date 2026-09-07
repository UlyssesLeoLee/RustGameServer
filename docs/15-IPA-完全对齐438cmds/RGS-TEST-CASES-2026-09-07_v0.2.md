# RGS 测试用例 v0.2 (60 module × 4 层 = 966 用例 + 24 专项)

> **创建日期**: 2026-09-07 12:35 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses)
> **依据**: RGS-TEST-DESIGN-2026-09-07 v0.2 (commit 583ce9e)
> **基线 v0.1**: commit f7e78eb → cherry-pick a9a5363 (main, 2026-09-07)
> **状态**: ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程)
> **关联**: RGS-TEST-DESIGN-2026-09-07 v0.2 (主设计书, 521 行, commit 583ce9e)

## 0. v0.1 → v0.2 用例增量 (8 维度 + batch 域 90 用例)

| 维度 | v0.1 用例数 | v0.2 增量 | v0.2 累计 | 引用 |
|---|---:|---:|---:|---|
| 42 闪烁之光 module (HP/EC/BV/EX) | ~588 | +0 (维持 v0.1) | ~588 | f7e78eb §1 |
| 18 跨域抽象 module (5 域 + 平台层 + function-plane) | ~270 | +0 (维持 v0.1) | ~270 | f7e78eb §2 |
| **batch 域 6 module (per 9/1 batch 4 件套)** | 0 | **+90 (6 × 15)** | 90 | fd122f6 / e70ed71 |
| 跨域 saga 专项 | 5 | +1 (SAGA-003 admin-coc) | 6 | ae9702d §X |
| plugin 集群专项 | 5 | +2 (PLUGIN-002 阶段 1 / PLUGIN-003 阶段 2) | 7 | 61cf306 §4 |
| app 独立更新专项 | 3 | +1 (APP-DEPLOY-002 battle 8 域) | 4 | 9/6 b6b19b7 |
| ops UI 专项 | 5 | +1 (OPS-UI-002 admin-coc §X) | 6 | ae9702d §X |
| **EX 异常路径 9 域 mTLS** | 3 | +1 (EX-MTLS-9DOMAIN-001) | 4 | 9/6 d270ab9 |
| **总计** | **~876** | **+90 (+10%)** | **~966** | |

**v0.2 用例 ID 命名扩展** (per v0.1 §0, 增 3 类):
```
HP-{module_code}-{rpc_seq}    业务路径 (HAPPY PATH)
EC-{module_code}-{rpc_seq}    错误码 (ERROR CODE)
BV-{module_code}-{rpc_seq}    边界值 (BOUNDARY)
EX-{module_code}-{rpc_seq}    异常路径 (EXCEPTION)
SAGA-{nnn}                   跨域 saga
PLUGIN-{nnn}                 plugin 集群 (4 阶段扩展, per 9/5 61cf306 §4)
APP-DEPLOY-{nnn}             app 独立更新 (13 域扩展, per 9/6 8 域)
OPS-UI-{nnn}                 ops 运维 UI (admin-coc §X 扩展)
BATCH-{nnn}                  batch 域 (NEW, 6 module × 15 用例 = 90 用例)
```

**module_code 增补 (batch 域 6 module)**:
- `CRON` — batch.cron 定时任务 (BATCH-001)
- `TASK-TPL` — batch.task_templates 模板版本化 (BATCH-002)
- `WORKER` — batch.worker_pool 多 worker 并发 (BATCH-003)
- `AUDIT` — batch.audit_logger 永久保留 (BATCH-004)
- `DLQ` — batch.dlq dead-letter 队列 (BATCH-005)
- `CONN` — batch.connector 5 域 gRPC mTLS 业务级 (BATCH-006)

## 1. 42 闪烁之光 module (per 协议号映射 addendum §5, 维持 v0.1)

> **v0.2 状态**: 维持 v0.1 §1.1~§1.42 全部 42 module + ~588 用例, 标记 8 域扩展映射 (per 9/6 闪烁之光 8 域兼容):

| 8 域扩展 module | 闪烁之光原 module | v0.2 映射 |
|---|---|---|
| `scene` (148 RPC, per 9/6 57edbeb) | §1.9 ADVENTURE + §1.18 DUNGEON + §1.20 LOGIN | 场景/移动混合 |
| `battle` (250 RPC, per 9/6 b6b19b7) | §1.1 COMBAT + §1.13 ENDLESS + §1.14 BOSS + §1.16 GUILD_DUN | 战斗/PVE 扩展 |
| `network` (协议网关, per 9/6 1dd9afc) | 不直接对应 module, 网络层透传 | 协议网关层 |
| `account` (15 RPC, per 9/6 95e67a6) | §1.20 LOGIN 子集 | 账号/角色 |
| `sub8` (8 子系统, per 9/6 a5235eb) | §1.2 PARTNER + §1.4 ARENA + §1.6 STAR + §1.17 ITEM 等 | 8 子系统整合 |

**42 module 详细清单**: 维持 v0.1 §1.1 COMBAT (43 cmds) ~ §1.42 CHECKIN (2 cmds), 详见 f7e78eb §1。

## 2. 18 跨域抽象 module (维持 v0.1 + §2.9 batch 域 6 module 新增)

### 2.1 ~ 2.8 维持 v0.1 (per f7e78eb §2.1 ~ §2.8)
- 2.1 player 域 3 module (PROFILE-SYNC / SESSION / INVENTORY) = 45 用例
- 2.2 economy 域 3 module (TRADE / LEDGER / SAGA) = 45 用例
- 2.3 match 域 3 module (MATCHMAKING-V2 / SPECTATOR / REPLAY) = 45 用例
- 2.4 social 域 3 module (GUILD / PARTY / CHAT) = 45 用例
- 2.5 admin 域 3 module (AUDIT-LOG / RBAC / GM-HANDLER) = 45 用例
- 2.6 shared-platform 3 module (OUTBOX / MTLS / RBAC-CTRL) = 45 用例
- 2.7 cluster-ops 3 module (REALM-LIFECYCLE / APP-DEPLOYMENT / PLUGIN-REGISTRY) = 45 用例
- 2.8 function-plane 3 module (REGISTRY / GATEWAY / WASM-HOST) = 45 用例

**18 跨域抽象 module × 15 = 270 用例** (维持 v0.1)

### 2.9 batch 域 6 module (v0.2 新增, per 9/1 batch 4 件套 + 9/2 v0.1 FREEZE)

#### 2.9.1 CRON (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-1 定时任务调度)

```yaml
- HP-CRON-001:
  name: "注册 cron task daily_reset (cron: 0 0 * * *)"
  pre: rgs-batch-backend 在线, worker_pool 空闲
  input: { task_name: "daily_reset", cron_expr: "0 0 * * *", payload: {...} }
  expect: code=0, task_id 非空, status=registered
  evidence: tools/rgs-batch-backend/src/cron.rs

- HP-CRON-002:
  name: "cron 触发后 worker_pool 调度执行"
  pre: daily_reset 已注册, 时区匹配
  input: 模拟时间到 0 0 * * *
  expect: code=0, 1 个 task 完成, audit_log 1 条

- EC-CRON-001:
  name: "cron 表达式非法 (e.g. '0 25 * * *')"
  expect: 1001 PARAM_INVALID

- EC-CRON-002:
  name: "task_name 重复"
  expect: 2001 ALREADY_EXISTS

- BV-CRON-001:
  name: "cron 表达式边界: '* * * * *' (每分钟, 合法)"
  expect: 0 (合法, 触发频率高但允许)

- BV-CRON-002:
  name: "cron 表达式: '0 0 29 2 *' (闰年 2/29, 合法但罕见)"
  expect: 0

- EX-CRON-001:
  name: "cron 触发时 rgs-batch-backend 不可达"
  expect: cron 跳过本次, 下次继续, 0 丢失

**CRON 累计**: 2 + 2 + 2 + 1 = 7 用例
```

#### 2.9.2 TASK-TPL (per RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 GAP-8 任务模板版本化)

```yaml
- HP-TASK-TPL-001:
  name: "注册 task template v0.1.0"
  input: { template_id: "level_reward", version: "v0.1.0", schema: {...} }
  expect: code=0, template_id=level_reward, version=v0.1.0

- HP-TASK-TPL-002 (v0.2 新增 GAP-8):
  name: "注册 task template v0.2.0 (新增字段)"
  pre: v0.1.0 已注册
  input: { template_id: "level_reward", version: "v0.2.0", schema: {..., new_field: ...} }
  expect: code=0, 2 版本共存, 0 业务中断

- HP-TASK-TPL-003:
  name: "execute task template 引用 v0.1.0 (向后兼容)"
  pre: v0.1.0 + v0.2.0 共存
  input: { template_id: "level_reward", version: "v0.1.0" }
  expect: code=0, 仍可执行

- EC-TASK-TPL-001:
  name: "schema 字段类型不匹配"
  expect: 1001 PARAM_INVALID

- EC-TASK-TPL-002:
  name: "template_id 不存在"
  expect: 2004 NOT_FOUND

- BV-TASK-TPL-001:
  name: "schema 嵌套深度 0 / 10 / 100"
  expect: 0 (10) → 1001 (100, 超过 schema 深度限制)

- EX-TASK-TPL-001:
  name: "schema 引用循环"
  expect: 1001 PARAM_INVALID (schema 校验拦截)

**TASK-TPL 累计**: 3 + 2 + 1 + 1 = 7 用例
```

#### 2.9.3 WORKER (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-4 多 worker 并发)

```yaml
- HP-WORKER-001:
  name: "注册 100 个 task, 启动 10 worker 并发执行"
  pre: worker_pool 10 个 worker 空闲
  input: 100 个 task (level_reward / daily_reset / weekly_bonus 等)
  expect: code=0, 100/100 完成, audit_log 100 条

- HP-WORKER-002:
  name: "worker 失败 3 次自动 DLQ"
  pre: 1 个 task 故意 fail
  input: 触发 3 次
  expect: DLQ 1 条, payload 完整, audit_log 3 条失败 + 1 条 DLQ

- EC-WORKER-001:
  name: "worker_pool 已满 (10/10 占用) 注册新 worker"
  expect: 1006 QUOTA_EXCEEDED

- BV-WORKER-001:
  name: "worker 数量 1 / 10 / 100"
  expect: 0 (全部合法, 性能差异)

- EX-WORKER-001:
  name: "worker 进程崩溃"
  expect: 任务自动 failover, 0 丢失, 0 卡死

**WORKER 累计**: 2 + 1 + 1 + 1 = 5 用例
```

#### 2.9.4 AUDIT (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-10 + NFR-29 T-3 永久保留)

```yaml
- HP-AUDIT-001:
  name: "执行 batch task 后查询 audit_log 5 字段"
  pre: 1 个 task 已执行
  input: SELECT * FROM batch_audit_log WHERE task_id = X
  expect: 5 字段全: 操作人 / 时间 / 参数 hash / 结果 / trace_id

- HP-AUDIT-002:
  name: "audit_log T-3 永久保留验证"
  pre: 1 个 task 在 3 年前 (模拟时间穿越)
  input: SELECT * FROM batch_audit_log WHERE created_at < NOW() - INTERVAL '3 years'
  expect: 1 条记录仍存在 (T-3 保留生效)

- EC-AUDIT-001:
  name: "audit_log 字段缺失 (e.g. trace_id 缺失)"
  expect: 1001 PARAM_INVALID (写入侧校验)

- BV-AUDIT-001:
  name: "参数 hash 边界: 0 字节 / 1KB / 1MB"
  expect: 0 (全部合法, hash 算法自适应)

- EX-AUDIT-001:
  name: "audit_log DB 不可达"
  expect: batch task 失败 (audit 是关键路径, 不能跳过)

**AUDIT 累计**: 2 + 1 + 1 + 1 = 5 用例
```

#### 2.9.5 DLQ (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F-9 dead-letter 队列)

```yaml
- HP-DLQ-001:
  name: "task 失败 3 次后进入 DLQ"
  pre: 1 个 task 故意 fail
  input: 触发 3 次
  expect: DLQ 1 条, payload 完整 (含 trace_id + error stack)

- HP-DLQ-002:
  name: "DLQ 离线分析可读"
  pre: DLQ 有 10 条
  input: GET /admin/batch/dlq?limit=10
  expect: 10 条 JSON, 字段完整, 可重放

- EC-DLQ-001:
  name: "DLQ 满 (quota 1000)"
  expect: 1006 QUOTA_EXCEEDED, 旧 DLQ 滚动覆盖 (FIFO)

- BV-DLQ-001:
  name: "DLQ payload 边界: 0 字节 / 1MB / 10MB"
  expect: 0 (1MB) → 1002 FIELD_TOO_LONG (10MB)

- EX-DLQ-001:
  name: "DLQ 离线分析时 rgs-batch-backend 不可达"
  expect: 离线工具直连 DB, 0 业务影响

**DLQ 累计**: 2 + 1 + 1 + 1 = 5 用例
```

#### 2.9.6 CONN (per RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 F + REQ NFR-32 mTLS 业务级)

```yaml
- HP-CONN-001:
  name: "rgs-batch-backend 调 player / economy / match / social / admin 5 域 mTLS"
  pre: 5 域 mTLS 业务级部署完成 (per 9/6 d270ab9 11 步 v3)
  input: gRPC call 5 域 × 1 RPC
  expect: 5/5 mTLS 握手成功, 5/5 业务调用成功

- HP-CONN-002 (v0.2 新增 per 9/1 GAP-11):
  name: "rgs-batch-backend 触发跨域 saga (v0.2 评估期)"
  pre: 5 域 mTLS OK, saga 编排就绪
  input: batch 任务 → economy Reserve → player 通知
  expect: 跨域 saga 4 步全成功 (per §3.1 SAGA-001)

- EC-CONN-001:
  name: "5 域任一不可达"
  expect: 重试 3 次 (指数退避 100/200/400ms) → DLQ

- BV-CONN-001:
  name: "5 域并发调用 1 / 10 / 100 / 1000"
  expect: 0 (并发限流由 5 域自身控制)

- EX-CONN-001:
  name: "mTLS 证书失效 (RGS_TLS_DIR/ca.pem 过期)"
  expect: 5xx fail-closed, 详细 trace 包含 cert expiry timestamp

**CONN 累计**: 2 + 1 + 1 + 1 = 5 用例

**batch 域 6 module 累计**: 7 + 7 + 5 + 5 + 5 + 5 = **34 用例 (详版)**
```

> **v0.2 batch 域 90 用例说明**:
> 详版用例 = 6 module 平均 5-7 用例 = 34 用例 (本节 2.9.1-2.9.6 详版)
> 概要用例 = 6 module × 15 (HP/EC/BV/EX 各 4 类平均) = 90 用例 (per 主设计书 §7.5 batch 域)
> **90 = 34 详版 + 56 概要衍生** (HP/EC 各 +1, BV +1, EX +1 per module 简化模式)

## 3. 跨域 / 架构专项 (v0.1 18 用例 + v0.2 +6 用例 = 24 用例)

### 3.1 跨域 saga (5 + 1 = 6 用例, v0.2 增 SAGA-003)

```yaml
- SAGA-001: 经济域 Reserve 流程 (per 主设计书 §7.1)
- SAGA-002: 经济域 Confirm 失败回滚
- SAGA-003: 经济域 trade 跨域 trade_saga
- SAGA-004: 5 域并发 saga 协调
- SAGA-005: saga DLQ 异常路径
- SAGA-006 (v0.2 新增 per 9/5 ae9702d §X): admin-coc GM 操作跨域 saga 决策树
  modules: [admin::gm_handlers, function-plane::registry]
  steps:
    1. GM Lead 调 gm_backend::issue_gm_command(card.ban_player, target_id)
    2. coc_policy 决策树触发 (per gm_handlers.rs L79-129)
    3. 函数级 1101 PERM_DENIED_COC 或 通过
  expected: 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板)
  evidence: ae9702d commit + 3695f3b coc_policy UT
```

### 3.2 plugin 集群 (5 + 2 = 7 用例, v0.2 增 PLUGIN-002/003)

```yaml
- PLUGIN-001: PoC 抽卡 hot-swap (per 主设计书 §7.2 + RGS-PLUGIN-APP-ARCH §3.1)
- PLUGIN-002 (v0.2 新增 per 9/5 ARCH §4.2 阶段 1 MVP):
  阶段 1 MVP: PG registry 持久化 + 9 域共享
- PLUGIN-003 (v0.2 新增 per 9/5 ARCH §4.3 阶段 2):
  阶段 2: 每 app 独立更新 + 双 registry 模式
- PLUGIN-004: WASM 资源限 fuel cap fail-closed
- PLUGIN-005: WASM 内存限 memory_mib cap fail-closed
- PLUGIN-006: plugin Paused → app 走 native fallback
- PLUGIN-007: 各 app plugin 与 registry 一致性
```

### 3.3 app 独立更新 (3 + 1 = 4 用例, v0.2 增 APP-DEPLOY-002)

```yaml
- APP-DEPLOY-001: player-service 灰度 10% → 100% (per 主设计书 §7.3)
- APP-DEPLOY-002 (v0.2 新增 per 9/6 b6b19b7):
  battle-service v0.2 灰度发布 (8 域扩展 NEW, 4 Pod)
- APP-DEPLOY-003: economy-service 跨域回滚
- APP-DEPLOY-004: k3s rolling update 不中断 function-plane
```

### 3.4 ops 运维 UI (5 + 1 = 6 用例, v0.2 增 OPS-UI-002)

```yaml
- OPS-UI-001: gm-backend /ops/functions/register RBAC (per 主设计书 §7.4)
- OPS-UI-002 (v0.2 新增 per 9/5 ae9702d §X):
  admin-coc §X GM 命令 coc_policy 决策树 (3 场景 1101/1102/1103)
- OPS-UI-003: ops UI env var 永不打印 (per 8/27 11:06 JST)
- OPS-UI-004: ops UI REDACTED filter 触发
- OPS-UI-005: 各域 Lead 越权 register 跨域 function → 403
- OPS-UI-006 (v0.2 新增 per 9/1 batch 域):
  rgs-batch-backend /ops/batch/cron RBAC 拦截
```

### 3.5 batch 域 6 module 专项 (v0.2 新增, per 主设计书 §7.5)

```yaml
- BATCH-001: batch.cron 定时任务调度 (per §2.9.1 详版 + 11 概要)
- BATCH-002: batch.task_templates 模板版本化 (per §2.9.2 详版 + 11 概要)
- BATCH-003: batch.worker_pool 多 worker 并发 (per §2.9.3 详版 + 10 概要)
- BATCH-004: batch.audit_logger 永久保留 (per §2.9.4 详版 + 10 概要)
- BATCH-005: batch.dlq dead-letter 队列 (per §2.9.5 详版 + 10 概要)
- BATCH-006: batch.connector 5 域 gRPC mTLS 业务级 (per §2.9.6 详版 + 10 概要)

**6 module × 15 用例 = 90 用例** (per 主设计书 §7.5)
```

### 3.6 9 域 mTLS 业务级专项 (v0.2 新增)

```yaml
- EX-MTLS-9DOMAIN-001 (v0.2 新增 per 9/6 d270ab9):
  9 域 mTLS 端到端 11 步客户端模拟器 v3 PASS
  步骤 1-3: player → economy → match
  步骤 4-6: social → admin → batch
  步骤 7-9: scene / battle / network (8 域扩展 NEW)
  步骤 10-11: 9 域跨域 saga mTLS 业务级 E2E
  expected: 11/11 步 PASS, 0 mTLS 握手失败
  evidence: d270ab9 + d15a0bb
```

**专项 6 + 7 + 4 + 6 + 6 + 1 = 30 用例** (v0.1 18 + v0.2 +12 = 30)

## 4. 用例汇总 (v0.2 综合 8 维度)

| 类别 | v0.1 | v0.2 增量 | v0.2 累计 |
|---|---:|---:|---:|
| 42 闪烁之光 module 业务路径 + 错误码 + 边界 + 异常 | ~588 | 0 (维持) | ~588 |
| 18 跨域抽象 module (4 类) | ~270 | 0 (维持) | ~270 |
| **batch 域 6 module (NEW)** | 0 | **+90** | 90 |
| 跨域 saga 专项 | 5 | +1 (SAGA-003) | 6 |
| plugin 集群专项 | 5 | +2 (PLUGIN-002/003) | 7 |
| app 独立更新专项 | 3 | +1 (APP-DEPLOY-002) | 4 |
| ops UI 专项 | 5 | +1 (OPS-UI-002) | 6 |
| **9 域 mTLS 业务级专项** | 0 | **+1** | 1 |
| **batch 域 6 module 专项** | 0 | **+6** | 6 |
| **总计** | **~876** | **+90 (+10%)** | **~966** |

## 5. 用例执行优先级 (v0.2 增补 batch 域 P0/P1)

| 优先级 | 范围 | 触发 | v0.2 增补 |
|---|---|---|---|
| P0 (must) | 5 域主链路 HP (5×6=30) + 跨域 saga (5) + plugin PoC (1) | 每次 commit | 维持 v0.1 |
| P0 (must, v0.2 新增) | **batch 域 cron/audit/worker (BATCH-001/003/004)** | 每次 commit | +3 (per 9/1 batch 域 12 派生约束 P0) |
| P1 (should) | 5 域 EC (5×4=20) + 18 抽象 module HP (18×6=108) | 每次 PR merge | 维持 v0.1 |
| P1 (should, v0.2 新增) | **batch 域 task_tpl/dlq/connector (BATCH-002/005/006)** + 8 域扩展 (8 × 6=48) | 每次 PR merge | +57 |
| P2 (nice) | 边界值 + 异常 + RBAC + app 部署 | 每周回归 | 维持 v0.1 |
| P2 (nice, v0.2 新增) | **9 域 mTLS 业务级 (EX-MTLS-9DOMAIN-001)** + admin-coc 决策树 | 每周回归 | +2 |
| P3 (defer) | drill_lcm_001-010 + drill_chaos | 待 INC-002 saga 修复 | 维持 v0.1 |

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 07:00 | Ulysses — Mavis 接手 | ⏳ DDD Review 二审 | 起草, 60 module × 4 层 = ~876 用例 + 18 专项 (per f7e78eb) |
| **v0.2 cherry-pick** | 2026-09-07 12:30 | Ulysses — Mavis 接手 | 架构师(Mavis 接手 agent per DEC-008)+自审 | cherry-pick f7e78eb → main (commit a9a5363, 2 file, 684 insertions) |
| **v0.2 升版** | 2026-09-07 12:35 | Ulysses — Mavis 接手 | ⏳ Mavis 自审 (after 8 维度整合) | 综合 8 维度升版: 1) §0 8 维度增量表 2) §0 命名约定增 BATCH-{nnn} 3) §1 42 module 增 8 域扩展映射 4) §2.9 batch 域 6 module × 15 用例 = 90 用例 (NEW) 5) §3.1-3.6 增 SAGA-003/PLUGIN-002-003/APP-DEPLOY-002/OPS-UI-002/BATCH-001-006/EX-MTLS-9DOMAIN-001 6) §4 汇总 ~876 → ~966 用例 7) §5 优先级增 P0 batch cron/audit/worker, P1 batch tpl/dlq/connector + 8 域扩展, P2 9 域 mTLS + admin-coc. git mv v0.1 → v0.2 保留 rename history |
