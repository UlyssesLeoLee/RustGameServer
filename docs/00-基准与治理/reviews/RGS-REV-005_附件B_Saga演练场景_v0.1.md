# RGS-REV-005 附件B Saga 演练场景 v0.1

> **G-CODE-04 Saga 端到端 6 场景详细演练**：本文件是 `RGS-REV-005_附件B_Saga演练场景Checklist.md`（占位 Checklist）的**首个详细填充版本**，覆盖 RGS-REV-003 §2.4 + RGS-IMPL-001 §3 + RGS-QA-001 v0.13 Q-003 要求的 6 场景。每场景包含输入条件 / 状态机迁移 / DB 状态 / 验证命令 / 预期结果 / 边界异常。
>
> **不替代 Checklist**：Checklist 用于运行时勾选，本文件用于设计与验证命令推导。

---

## §0 元信息

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REV-005 附件B |
| 版本 | v0.1（首次详细填充，per Phase 0.5 形式完成 2026-08-24） |
| 制定日 | 2026-08-24 |
| 制定者 | Ulysses（一人公司 12 角色 per DEC-008） |
| 关联 G-CODE | G-CODE-04（跨 DB Saga 一致性，per RGS-EXEC-001 v0.3 §3.4） |
| 关联 Q | Q-003（5 域一致性，per RGS-QA-001 v0.13） |
| 关联 ADR | RGS-ADR-0052（Active-Active + all-reachable PFAU） |
| 关联代码 | `crates/economy-service/src/saga.rs` + `saga_orchestrator.rs` + `inbox.rs` + `reservation.rs` |
| 关联 SQL | `crates/economy-service/migrations/0002_saga_init.sql`（sagas + reservations + inbox） |
| 状态机 | `Pending → Running → Compensating → Completed / Failed / Aborted`（per `saga.rs` `SagaStatus`） |
| 关联 Checklist | `RGS-REV-005_附件B_Saga演练场景Checklist.md`（B.1~B.8 简版） |

---

## §B.1 演练前置环境

### §B.1.1 数据准备

- 5 独立 PostgreSQL 18.6 DB（per ARC-008）：
  - `player_db`（player 域）
  - `economy_db`（economy 域）
  - `match_db`（match 域）
  - `social_db`（social 域）
  - `admin_db`（admin 域 + 跨域 Saga 协调者元数据）
- 每个 DB 需应用 migration `0001_init.sql` + `0002_saga_init.sql` + `0003_outbox.sql` + `0004_outbox_check.sql`（仅 economy_db 应用全部 4 个，其他 4 个 DB 应用前 2 个 + 各自域初始化）

### §B.1.2 工具

- `psql`（PostgreSQL 客户端）
- `cargo test --workspace`（运行所有 Rust 测试）
- `grpcurl`（gRPC 客户端）
- `kubectl -n rust-game-server`（K3s 验证）
- OTel Collector / Tempo（trace 可观测）

### §B.1.3 关键 SQL 表

```sql
-- sagas 表（per 0002_saga_init.sql）
CREATE TABLE sagas (
    id UUID PRIMARY KEY,
    saga_type TEXT NOT NULL CHECK (saga_type IN ('transfer', 'daily_reward', 'purchase')),
    command_id UUID NOT NULL,
    idempotency_key TEXT NOT NULL,
    current_step INTEGER NOT NULL DEFAULT 0,
    steps JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'compensating', 'completed', 'failed', 'aborted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX uq_sagas_command_id ON sagas (command_id);

-- reservations 表
CREATE TABLE reservations (
    id UUID PRIMARY KEY,
    saga_id UUID NOT NULL REFERENCES sagas(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id),
    amount BIGINT NOT NULL CHECK (amount > 0),
    currency TEXT NOT NULL CHECK (currency IN ('gold', 'diamond', 'token')),
    status TEXT NOT NULL DEFAULT 'reserved'
        CHECK (status IN ('reserved', 'confirmed', 'compensated', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL  -- 默认 5 分钟
);

-- inbox 表（幂等）
CREATE TABLE inbox (
    id UUID PRIMARY KEY,
    command_id UUID NOT NULL,
    handler TEXT NOT NULL,
    result TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'processed' CHECK (status IN ('processed', 'failed')),
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (command_id, handler)
);
```

---

## §B.2 场景 1：正常 Saga 路径（玩家购买道具）

> **业务路径**：player → economy（扣款）→ match（发放道具）→ social（通知）→ player（更新余额）
> **SagaType**：`Purchase`
> **跨 DB 数量**：4 域（player + economy + match + social）

### §B.2.1 输入条件

```bash
# 前置：player 账户余额 1000 gold，道具价格 100 gold
# 通过 gRPC 发起购买请求
grpcurl -insecure -d '{
  "player_id": "<uuid>",
  "item_id": "sword_001",
  "price": {"currency": "gold", "amount": 100},
  "idempotency_key": "purchase-2026-08-24-001"
}' player-service.rust-game-server.svc.cluster.local:50051 \
  player.v1.PlayerService/Purchase
```

### §B.2.2 期望状态机迁移

```
Pending (t=0)
  ↓ start() [handler.reserve]
Running (t=10ms, step 0 = reserve)
  ↓ step 0 mark_completed, advance
Running (t=20ms, step 1 = transfer/扣款)
  ↓ step 1 mark_completed, advance
Running (t=50ms, step 2 = confirm/发放)
  ↓ step 2 mark_completed, advance
Running (t=80ms, step 3 = notify/通知)
  ↓ step 3 mark_completed, advance
Running (t=100ms, step 4 = balance_update)
  ↓ step 4 mark_completed, advance=false
Completed (t=110ms, completed_at = now)
```

### §B.2.3 期望 DB 状态

**`economy_db.accounts` 表**：
```sql
SELECT id, gold_balance FROM accounts WHERE id = '<player_uuid>';
-- 期望: gold_balance = 900 (1000 - 100)
```

**`economy_db.transaction_ledger` 表**：
```sql
SELECT kind, amount, status FROM transaction_ledger
WHERE saga_id = '<saga_uuid>';
-- 期望: 1 条 'debit' kind, amount=100, status='confirmed'
```

**`economy_db.sagas` 表**：
```sql
SELECT status, current_step, completed_at IS NOT NULL AS done
FROM sagas WHERE command_id = '<cmd_uuid>';
-- 期望: status='completed', current_step=4, done=true
```

**`economy_db.reservations` 表**：
```sql
SELECT status FROM reservations WHERE saga_id = '<saga_uuid>';
-- 期望: 1 条 status='confirmed'
```

**`economy_db.inbox` 表**：
```sql
SELECT handler, status FROM inbox WHERE command_id = '<cmd_uuid>';
-- 期望: 至少 5 条（每 step handler），status='processed'
```

**`match_db.player_inventory` 表**：
```sql
SELECT * FROM player_inventory WHERE player_id = '<player_uuid>' AND item_id = 'sword_001';
-- 期望: 1 条, quantity=1
```

**`social_db.notifications` 表**：
```sql
SELECT * FROM notifications WHERE player_id = '<player_uuid>' ORDER BY created_at DESC LIMIT 1;
-- 期望: 1 条, type='item_received', content 含 'sword_001'
```

### §B.2.4 验证命令

```bash
# 1. UT 验证（state machine 单元测试）
cargo test -p economy-service saga::tests::saga_lifecycle
# 期望: ok

# 2. IT 验证（端到端 5 步全 Completed）
cargo test -p economy-service --test saga_purchase_happy_path
# 期望: ok

# 3. DB 验证（psql）
PGPASSWORD=economy psql -h postgres.rust-game-server.svc.cluster.local -U economy -d economy_db \
  -c "SELECT status, current_step FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: status=completed, current_step=4

# 4. Trace 验证（OTel）
# 通过 Grafana/Tempo 查询 trace_id，确认 5 步 span 串联
```

### §B.2.5 预期结果

- 5 步全部 `Completed`，`status = completed`，`completed_at` 已设置
- 玩家余额从 1000 → 900
- 1 条 `debit` 记账
- 1 条 `confirmed` reservation
- 1 条 inbox 记录（5 个 handler 各 1 条）
- 1 条 inventory 记录（match_db）
- 1 条 notification 记录（social_db）
- inbox/outbox 表无残留
- 监控指标：所有步骤延迟 < 100ms

### §B.2.6 边界 + 异常

| 边界 | 行为 |
|---|---|
| **E1.1**：player 余额不足 | step 1 失败 → 补偿（释放 reservation）→ `status=compensating` → `status=failed` |
| **E1.2**：match 域 Pod 临时不可达 | step 2 重试 3 次（每次 100ms 间隔），仍失败则补偿 |
| **E1.3**：social 域发通知失败但 inventory 已发放 | step 3 失败 → 补偿 inventory（撤回）→ step 2 也撤回 → `failed` |
| **E1.4**：客户端在 step 0~4 中重复发送同 `idempotency_key` | 走场景 5 去重路径，结果与首次一致 |
| **E1.5**：Saga 运行时协调者 Pod crash | `saga_orchestrator.rs::resume(saga_id)` 从 DB 加载 + 续跑 |

---

## §B.3 场景 2：补偿路径（中途失败 → Failed）

> **业务路径**：step 2（match 发放）写入错误 → 补偿 step 0/1 → `Failed`
> **SagaType**：`Purchase`

### §B.3.1 输入条件

- 正常启动购买 saga（同 §B.2.1）
- **故障注入**：match 域 Pod 在 step 2 执行期间手动 `kubectl exec` 注入 DB 写错误：
  ```bash
  kubectl exec -n rust-game-server match-service-0 -- \
    sh -c "iptables -A OUTPUT -d postgres.match-db.svc.cluster.local -j DROP"
  ```

### §B.3.2 期望状态机迁移

```
Pending
  ↓ start()
Running (step 0 = reserve)
  ↓ mark_completed
Running (step 1 = transfer)
  ↓ mark_completed
Running (step 2 = confirm/match 发放)
  ↓ [match 域 DB 写错误] mark_failed
  ↓ SagaOrchestrator.compensate()
  ↓ handler.compensate(step 1) [refund gold]
  ↓ handler.compensate(step 0) [release reservation]
  ↓ saga.compensate()  [status=compensating]
  ↓ saga.fail()        [status=failed]
Failed (t=1.2s, completed_at = now)
```

### §B.3.3 期望 DB 状态

**`economy_db.sagas` 表**：
```sql
SELECT status, current_step, steps FROM sagas WHERE command_id = '<cmd_uuid>';
-- 期望:
--   status = 'failed'
--   current_step = 2
--   steps[0].status = 'compensated'
--   steps[1].status = 'compensated'
--   steps[2].status = 'failed' (有 error 字段)
--   steps[3..4].status = 'pending'
```

**`economy_db.accounts` 表**：
```sql
SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';
-- 期望: gold_balance = 1000 (补偿恢复原始余额)
```

**`economy_db.transaction_ledger` 表**：
```sql
SELECT kind, amount, status FROM transaction_ledger WHERE saga_id = '<saga_uuid>';
-- 期望:
--   1 条 'debit' amount=100 status='confirmed'  (step 1)
--   1 条 'credit' amount=100 status='confirmed' (补偿退款)
```

**`economy_db.reservations` 表**：
```sql
SELECT status FROM reservations WHERE saga_id = '<saga_uuid>';
-- 期望: status='compensated'
```

**`match_db.player_inventory` 表**：
```sql
SELECT count(*) FROM player_inventory WHERE player_id = '<player_uuid>' AND item_id = 'sword_001';
-- 期望: 0 (没发放成功)
```

### §B.3.4 验证命令

```bash
# 1. 模拟故障（前置）
kubectl exec -n rust-game-server match-service-0 -- \
  iptables -A OUTPUT -d postgres.match-db.svc.cluster.local -j DROP

# 2. 发起购买
grpcurl ... player.v1.PlayerService/Purchase
# 期望: 返回 FAILED 状态 + trace_id

# 3. DB 验证
PGPASSWORD=economy psql -h postgres.rust-game-server.svc.cluster.local -U economy -d economy_db \
  -c "SELECT status FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: status=failed

# 4. 余额验证
PGPASSWORD=economy psql ... -c "SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';"
# 期望: 1000 (与原始一致)

# 5. UT 验证（状态机 + 补偿顺序）
cargo test -p economy-service saga_orchestrator::tests::compensate_lifecycle
# 期望: ok

# 6. 恢复故障
kubectl exec -n rust-game-server match-service-0 -- \
  iptables -D OUTPUT -d postgres.match-db.svc.cluster.local -j DROP
```

### §B.3.5 预期结果

- `sagas.status = failed`，`completed_at` 已设置
- 步骤 0/1 状态 = `compensated`，步骤 2 状态 = `failed`，步骤 3/4 状态 = `pending`
- 玩家余额恢复（1000 → 900 → 1000 via 补偿 credit）
- 1 条 `credit` 记账（补偿）
- 1 条 reservation `compensated`
- `match_db.player_inventory` 无新记录
- 失败日志完整（含 `trace_id` + `saga_id` + `command_id`）
- 监控指标：补偿完成时间 < 1s

### §B.3.6 边界 + 异常

| 边界 | 行为 |
|---|---|
| **E2.1**：补偿 step 1（refund）也失败 | 标记 `compensation_failed`，**保留 saga.status=compensating** 不进入 `failed`（待人工介入 per 场景 4） |
| **E2.2**：补偿 step 0（release reservation）时 reservation 已过期（5 分钟） | `Reservation.is_expired() == true` → 跳过释放步骤，记 `skipped_reason=expired` |
| **E2.3**：补偿期间 economy 域 Pod crash | `saga_orchestrator.resume(saga_id)` 重入：从 `Compensating` 状态继续，**不重复已补偿的 step**（per `saga_orchestrator.rs::compensate` 的"先收集 Completed 列表 + 再 handler.compensate"顺序修复） |
| **E2.4**：第 1 步就失败（reserve 失败） | 跳过 step 1~4 补偿（因为还没执行），直接 `failed` |
| **E2.5**：第 5 步（最后一步）失败 | 补偿 step 0/1/2/3，回退到原始状态 |

---

## §B.4 场景 3：超时路径（步进超过 deadline → Failed + 补偿）

> **业务路径**：step 2 执行超过 5s deadline（reservation 过期阈值），自动 `Failed` + 补偿
> **SagaType**：`Purchase`
> **触发**：人工延迟 step 2 handler（注入 sleep）

### §B.4.1 输入条件

```bash
# 故障注入：match 域 step 2 handler 注入 10s sleep（超过 reservation 5 分钟过期 + 步进 deadline 30s）
kubectl exec -n rust-game-server match-service-0 -- \
  sh -c "echo 'env SAGA_STEP2_DELAY_MS=10000 >> /etc/economy-service/env'"
kubectl rollout restart deployment/match-service -n rust-game-server
```

> **deadline 策略**（per RGS-IMPL-001 §3）：单 step 30s，Saga 整体 5 分钟（reservation 过期一致）

### §B.4.2 期望状态机迁移

```
Pending
  ↓ start() (t=0)
Running (step 0 = reserve, t=10ms, Completed)
  ↓
Running (step 1 = transfer, t=20ms, Completed)
  ↓
Running (step 2 = confirm/match 发放, t=30ms Running, ..., t=30s 仍未响应)
  ↓ [协调者发现 step 2 超过 deadline 30s] 强制 mark_failed
  ↓ SagaOrchestrator.compensate()
  ↓ handler.compensate(step 1) [refund]
  ↓ handler.compensate(step 0) [release]
  ↓
Failed (t=30.5s, completed_at = now)
```

### §B.4.3 期望 DB 状态

**`economy_db.sagas` 表**：
```sql
SELECT status, current_step,
       steps->2->>'error' AS step2_error,
       EXTRACT(EPOCH FROM (completed_at - created_at)) AS duration_sec
FROM sagas WHERE command_id = '<cmd_uuid>';
-- 期望:
--   status = 'failed'
--   current_step = 2
--   step2_error 含 'deadline exceeded' 或 'timeout'
--   duration_sec ≈ 30
```

**`economy_db.reservations` 表**：
```sql
SELECT status FROM reservations WHERE saga_id = '<saga_uuid>';
-- 期望: status='compensated' (补偿释放)
```

**`economy_db.accounts` 表**：
```sql
SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';
-- 期望: 1000 (补偿恢复)
```

### §B.4.4 验证命令

```bash
# 1. 注入延迟 + 重启 match 域
kubectl exec -n rust-game-server match-service-0 -- \
  sh -c "echo 'SAGA_STEP2_DELAY_MS=10000' >> /etc/economy-service/env"
kubectl rollout restart deployment/match-service -n rust-game-server
sleep 30  # 等 match 域 Ready

# 2. 发起购买 + 记录 t0
T0=$(date +%s)
grpcurl ... player.v1.PlayerService/Purchase

# 3. 等待 35s
sleep 35

# 4. DB 验证
PGPASSWORD=economy psql ... -c "SELECT status, EXTRACT(EPOCH FROM (completed_at - created_at)) AS dur FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: status=failed, dur ≈ 30

# 5. UT 验证（deadline 检测）
cargo test -p economy-service saga_orchestrator::tests::step_deadline_exceeded
# 期望: ok

# 6. 恢复
kubectl exec -n rust-game-server match-service-0 -- \
  sh -c "sed -i '/SAGA_STEP2_DELAY_MS/d' /etc/economy-service/env"
kubectl rollout restart deployment/match-service -n rust-game-server
```

### §B.4.5 预期结果

- `sagas.status = failed`
- 步进时间 ≈ 30s（deadline 触发）
- 步骤 2 error 字段含 `"deadline exceeded"`
- 步骤 0/1 状态 = `compensated`
- 玩家余额恢复（1000 → 900 → 1000）
- reservation 状态 = `compensated`
- 监控告警：saga_timeout_total 指标 +1（per `RGS-OPS-100 §4 关键 SLO`）
- 30s 内触发人工升级（per §B.5）— 若超过 60s 仍未处理

### §B.4.6 边界 + 异常

| 边界 | 行为 |
|---|---|
| **E3.1**：reservation 在补偿前已过期 | 同 E2.2：跳过释放，记 `skipped_reason=expired` |
| **E3.2**：deadline 检测与补偿执行重叠（协调者并发） | 协调者单线程（per RGS-IMPL-001 §3.1 单一协调者 + DB `version` 字段 CAS），互斥执行 |
| **E3.3**：match 域在第 31s 时恢复并尝试完成 step 2 | 协调者已 mark_failed，match 域 handler 写入被 in_short_circuit（per `saga_orchestrator.rs::compensate` 顺序修复后，inbox 校验 `saga.status` ≠ `Running` 直接拒收） |
| **E3.4**：整体 Saga 超过 5 分钟（reservation 过期阈值） | 步进任何 step 都会先检查 reservation 过期；过期则强制 `Failed` + 补偿 |
| **E3.5**：客户端断连但协调者仍在跑 | 协调者独立完成 saga，客户端重连后通过 `query_saga(saga_id)` 获取最终状态 |

---

## §B.5 场景 4：人工升级路径（金额 > 阈值 → 待 GM 审批）

> **业务路径**：购买价格 15000 gold（> 10000 阈值）→ step 0 后暂停 → admin 域审批 → 通过/拒绝
> **SagaType**：`Purchase`
> **新状态扩展**：`Pending → Running → PendingReview → Running → Completed / Failed`（per RGS-IMPL-100 §3.4 人工审核挂起态）

### §B.5.1 输入条件

- 阈值常量：`REVIEW_THRESHOLD = 10000`（per RGS-IMPL-100 §3.4，配置在 economy-service env）
- 发起 15000 gold 购买：
  ```bash
  grpcurl -insecure -d '{
    "player_id": "<uuid>",
    "item_id": "legendary_sword",
    "price": {"currency": "gold", "amount": 15000},
    "idempotency_key": "purchase-2026-08-24-big-001"
  }' player-service.rust-game-server.svc.cluster.local:50051 \
    player.v1.PlayerService/Purchase
  ```

### §B.5.2 期望状态机迁移

```
Pending
  ↓ start() (t=0)
Running (step 0 = reserve, t=10ms, Completed)
  ↓ [economy 域检测 amount > REVIEW_THRESHOLD]
PendingReview (t=20ms, paused, 通知 admin 域)
  ↓ [admin 域 GM 通过审批]
Running (t=15min, step 1 = transfer, t=15min+10ms, Completed)
  ↓ step 2, step 3, step 4
Completed (t=15min+500ms)
```

> **若 GM 拒绝**：`PendingReview → Aborted`（不进入 Failed，per RGS-IMPL-100 §3.4 "拒绝 = 用户主动取消"）

### §B.5.3 期望 DB 状态

**`economy_db.sagas` 表**（审批前）：
```sql
SELECT status, current_step FROM sagas WHERE command_id = '<cmd_uuid>';
-- 期望: status='pending_review'（PendingReview 新状态，per RGS-IMPL-100 §3.4）
```

**`admin_db.review_queue` 表**：
```sql
SELECT * FROM review_queue WHERE saga_id = '<saga_uuid>';
-- 期望: 1 条, status='pending', amount=15000, currency='gold', reason='amount_exceeds_threshold'
```

**`admin_db.audit_log` 表**：
```sql
SELECT * FROM audit_log WHERE saga_id = '<saga_uuid>' ORDER BY created_at;
-- 期望:
--   1 条 'review_requested' (amount=15000)
--   1 条 'review_approved' (gm_id, reviewed_at) — 在审批通过后
```

**`economy_db.reservations` 表**（审批通过后）：
```sql
SELECT status FROM reservations WHERE saga_id = '<saga_uuid>';
-- 期望: status='confirmed'（最终转 confirm）
```

### §B.5.4 验证命令

```bash
# 1. 发起 15000 gold 购买
grpcurl ... player.v1.PlayerService/Purchase
# 期望: 返回 pending_review 状态 + review_id

# 2. DB 验证（saga 暂停）
PGPASSWORD=economy psql ... -c "SELECT status FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: status=pending_review

# 3. DB 验证（admin 队列）
PGPASSWORD=admin psql -h postgres.rust-game-server.svc.cluster.local -U admin -d admin_db \
  -c "SELECT status FROM review_queue WHERE saga_id = '<saga_uuid>';"
# 期望: status=pending

# 4. GM 通过审批（admin gRPC）
grpcurl -insecure -d '{
  "review_id": "<review_id>",
  "decision": "approve",
  "gm_id": "gm_001",
  "comment": "VIP customer, approved"
}' admin-service.rust-game-server.svc.cluster.local:50051 \
  admin.v1.AdminService/ReviewDecision
# 期望: 0 错误码

# 5. 等待 saga 完成
sleep 5

# 6. DB 验证（saga 完成）
PGPASSWORD=economy psql ... -c "SELECT status FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: status=completed

# 7. UT 验证
cargo test -p economy-service saga_orchestrator::tests::pending_review_lifecycle
# 期望: ok
```

### §B.5.5 预期结果

- 玩家发起 15000 gold 购买后立即收到 `pending_review` 状态
- `admin_db.review_queue` 出现待审任务
- GM 审批 SLA < 30min（per RGS-IMPL-100 §3.4）
- 通过后 saga 自动续跑，最终 `completed`
- 拒绝后 saga `aborted`，reservation 释放，余额不变
- 审计日志完整（含 GM ID + 审批时间 + 评论）

### §B.5.6 边界 + 异常

| 边界 | 行为 |
|---|---|
| **E5.1**：GM 拒绝 | `PendingReview → Aborted`，reservation 释放，余额不变；玩家收到通知 |
| **E5.2**：审批 SLA 超时（> 30min） | 自动升级到 admin Lead 邮箱 + Slack 告警，但 saga 继续 `pending_review`（不强制 abort） |
| **E5.3**：GM 审批通过后协调者 crash | `saga_orchestrator.resume(saga_id)` 重新加载，发现 `status=pending_review` + 已有 approve 记录 → 触发 `pending_review → running` 续跑 |
| **E5.4**：同一笔交易被多 GM 重复审批 | inbox 唯一约束 `(command_id, handler='saga.review.approve')` 去重，第二次返回原结果 |
| **E5.5**：阈值调整（10000 → 5000）后存量 saga | 存量 saga 按启动时的阈值判断，不回溯 |

---

## §B.6 场景 5：Inbox 去重路径（同 `idempotency_key` 重试 → 同一结果）

> **业务路径**：客户端重试同 `command_id` 2 次（网络抖动），第 2 次走 inbox 幂等返回原结果
> **SagaType**：`Purchase`
> **关键组件**：`inbox.rs`（per `UNIQUE (command_id, handler)`）

### §B.6.1 输入条件

```bash
# 同一 client 在 200ms 内连续发起 2 次同 idempotency_key 购买
grpcurl -insecure -d '{
  "player_id": "<uuid>",
  "item_id": "sword_001",
  "price": {"currency": "gold", "amount": 100},
  "idempotency_key": "purchase-retry-2026-08-24-001",
  "command_id": "<fixed_uuid>"
}' player-service.rust-game-server.svc.cluster.local:50051 \
  player.v1.PlayerService/Purchase &

grpcurl -insecure -d '{
  "player_id": "<uuid>",
  "item_id": "sword_001",
  "price": {"currency": "gold", "amount": 100},
  "idempotency_key": "purchase-retry-2026-08-24-001",
  "command_id": "<fixed_uuid>"
}' player-service.rust-game-server.svc.cluster.local:50051 \
  player.v1.PlayerService/Purchase
```

### §B.6.2 期望状态机迁移

```
[第 1 次请求]
Pending
  ↓ ... 5 步
Completed (saga_id = A)

[第 2 次请求 - 同 command_id]
  ↓ inbox.find_by_command(cmd, 'saga.purchase') 返回 Some(原结果)
  ↓ 跳过新建 saga，直接返回原 saga_id=A + status=completed
（无状态机迁移，仅查表）
```

### §B.6.3 期望 DB 状态

**`economy_db.sagas` 表**：
```sql
SELECT count(*), array_agg(id) FROM sagas WHERE command_id = '<cmd_uuid>';
-- 期望: count=1 (UNIQUE 约束生效), array=[saga_id_A]
```

**`economy_db.inbox` 表**：
```sql
SELECT handler, count(*) FROM inbox WHERE command_id = '<cmd_uuid>' GROUP BY handler;
-- 期望: 至少 1 个 handler 有 1 条记录（去重生效，第二次的 `ON CONFLICT DO NOTHING`）
```

**`economy_db.accounts` 表**：
```sql
SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';
-- 期望: 900 (1000 - 100)，**不** 800（不重复扣款）
```

**`match_db.player_inventory` 表**：
```sql
SELECT count(*) FROM player_inventory WHERE player_id = '<player_uuid>' AND item_id = 'sword_001';
-- 期望: 1 (不重复发放)
```

### §B.6.4 验证命令

```bash
# 1. 第 1 次请求
RESP1=$(grpcurl ... player.v1.PlayerService/Purchase -d '{...}')
echo $RESP1
# 期望: status=completed, saga_id=A

# 2. 第 2 次请求（同 command_id）
RESP2=$(grpcurl ... player.v1.PlayerService/Purchase -d '{...}')
echo $RESP2
# 期望: status=completed, saga_id=A (与第 1 次相同)

# 3. DB 验证（saga 数量）
PGPASSWORD=economy psql ... -c "SELECT count(*) FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: 1

# 4. 余额验证（不重复扣款）
PGPASSWORD=economy psql ... -c "SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';"
# 期望: 900

# 5. UT 验证
cargo test -p economy-service inbox::tests::in_memory_inbox_idempotency
# 期望: ok

# 6. 监控指标
curl http://prometheus.rust-game-server.svc.cluster.local:9090/api/v1/query?query=saga_inbox_dedup_total
# 期望: 值 = 1
```

### §B.6.5 预期结果

- 两次返回相同 `saga_id` + `status=completed`
- DB 中 `sagas` 表只有 1 条（`UNIQUE INDEX uq_sagas_command_id` 生效）
- 玩家余额只扣 1 次（900，不 800）
- inventory 只 1 条
- 监控 `saga_inbox_dedup_total += 1`

### §B.6.6 边界 + 异常

| 边界 | 行为 |
|---|---|
| **E6.1**：不同 `command_id` 但同 `idempotency_key` | inbox 不去重（按 `(command_id, handler)` 去重，不按 `idempotency_key`）；但 `idx_sagas_idempotency_key` 索引可加速查询 |
| **E6.2**：同 `command_id` 但不同 `item_id` | inbox 命中（同一 handler），返回原 saga 结果（item_id 不变）— 客户端需自检 |
| **E6.3**：第 1 次请求 saga 还在 Running（未完成）时第 2 次请求 | inbox 查 `find_by_command` 命中 `Processed`，但 saga.status='running' → 返回 `pending` + 提示客户端等待 |
| **E6.4**：第 1 次失败 + 第 2 次重试（inbox 标 `Failed`） | `InboxStatus::Failed` 时仍按 `ON CONFLICT DO NOTHING`，第 2 次不重试；客户端需显式 `retry=true` 参数 |
| **E6.5**：第 1 次成功后客户端修改 item_id 重试 | 按 E6.2，返回原结果；客户端业务层应基于 `saga_id` 做幂等 |

---

## §B.7 场景 6：PFAU + Saga 路径（跨节点 Saga + Active-Active 协调 per ADR-0052）

> **业务路径**：Saga 执行期间，admin 域对 match 域发起 PFAU canary 升级 → 协调者 Active-Active 双 leader 协调 → 升级期间某节点掉线 → 升级挂起 → Saga 完成数据一致
> **关联 ADR**：RGS-ADR-0052（Active-Active + all-reachable PFAU）
> **关键约束**：PFAU 升级期间 match 域所有节点必须 all-reachable（per ADR-0052 §2.1，120s ACK deadline）

### §B.7.1 输入条件

```bash
# 1. 启动一个跨 4 域的 saga（涉及 match 域）
grpcurl ... player.v1.PlayerService/Purchase -d '{...}'
# saga 启动，运行到 step 2 (match 发放)

# 2. 模拟 admin 域发起 PFAU 升级（match 域 canary）
kubectl exec -n rust-game-server cluster-ops-service-0 -- \
  /app/cluster-ops pfau start --target=match-service --strategy=canary --version=0.2.0
# PFAU 启动，需要 match 域 3 节点在 120s 内全部 ACK

# 3. 故障注入：在 PFAU 期间 kill match 域 1 个节点
kubectl delete pod match-service-1 -n rust-game-server --force --grace-period=0
```

### §B.7.2 期望状态机迁移（协调者 Active-Active 双 leader 视角）

```
[协调者 leader-1 持有 saga 锁]
Pending
  ↓ start()
Running (step 0 = reserve, t=10ms, Completed)
  ↓
Running (step 1 = transfer, t=20ms, Completed)
  ↓
Running (step 2 = confirm/match 发放, t=30ms)
  ↓ [PFAU 启动, 协调者双 leader 协商]
  ↓ [协调者 leader-2 通过 `version` 字段 CAS 接管, leader-1 释放锁]
  ↓ [match-service-1 killed, PFAU 等待 120s ACK 超时 → 永久挂起 per ADR-0052 §2.1]
  ↓ [协调者 leader-2 检测 PFAU 挂起, 不重试 step 2, 走补偿路径]
Compensating (handler.compensate(step 1) [refund], handler.compensate(step 0) [release])
  ↓
Failed (t=125s, completed_at = now)
```

> **关键点**：协调者 Active-Active 双 leader 通过 `version` CAS 避免脑裂；PFAU 挂起时协调者不能继续推进 saga，必须走补偿。

### §B.7.3 期望 DB 状态

**`admin_db.pfau_state` 表**：
```sql
SELECT * FROM pfau_state WHERE target = 'match-service' ORDER BY created_at DESC LIMIT 1;
-- 期望:
--   status = 'paused_permanently'
--   reason = 'all-reachable timeout: match-service-1 not ACK within 120s'
```

**`economy_db.sagas` 表**：
```sql
SELECT status, current_step,
       steps->2->>'error' AS step2_error
FROM sagas WHERE command_id = '<cmd_uuid>';
-- 期望:
--   status = 'failed'
--   current_step = 2
--   step2_error 含 'pfau_paused' 或 'cluster_unreachable'
```

**`economy_db.accounts` 表**：
```sql
SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';
-- 期望: 1000 (补偿恢复)
```

**`match_db.pfau_kubernetes_pod_state`（admin 域观测）**：
```sql
SELECT pod_name, status, last_ack_at FROM pod_state
WHERE target = 'match-service' AND pod_name = 'match-service-1';
-- 期望: status='not_ready', last_ack_at=120s+ 之前
```

### §B.7.4 验证命令

```bash
# 1. 启动 saga
grpcurl ... player.v1.PlayerService/Purchase -d '{...}' &
SAGA_PID=$!

# 2. 启动 PFAU
kubectl exec -n rust-game-server cluster-ops-service-0 -- \
  /app/cluster-ops pfau start --target=match-service --strategy=canary --version=0.2.0
PFAU_PID=$!

# 3. 在 PFAU 启动 30s 后 kill match 节点
sleep 30
kubectl delete pod match-service-1 -n rust-game-server --force --grace-period=0

# 4. 等待 PFAU 超时（120s）+ 协调者补偿
sleep 130

# 5. DB 验证（PFAU 状态）
PGPASSWORD=admin psql ... -c "SELECT status, reason FROM pfau_state WHERE target = 'match-service' ORDER BY created_at DESC LIMIT 1;"
# 期望: status=paused_permanently, reason 含 'all-reachable timeout'

# 6. DB 验证（saga 状态）
PGPASSWORD=economy psql ... -c "SELECT status FROM sagas WHERE command_id = '<cmd_uuid>';"
# 期望: status=failed

# 7. 余额验证
PGPASSWORD=economy psql ... -c "SELECT gold_balance FROM accounts WHERE id = '<player_uuid>';"
# 期望: 1000

# 8. 协调者日志（验证 Active-Active 切换）
kubectl logs -n rust-game-server cluster-ops-service-0 | grep "leader_switch"
# 期望: 1 条 leader 切换记录

# 9. UT 验证（ADR-0052 all-reachable 模拟）
cargo test -p cluster-ops-service pfau::tests::all_reachable_timeout
# 期望: ok

# 10. 恢复
kubectl rollout restart deployment/match-service -n rust-game-server
kubectl exec -n rust-game-server cluster-ops-service-0 -- \
  /app/cluster-ops pfau resume --target=match-service
```

### §B.7.5 预期结果

- PFAU 启动后 match 域 3 节点中 match-service-1 在 120s 内未 ACK
- PFAU 状态进入 `paused_permanently`（per ADR-0052 §2.1 "Paused 永久挂起"）
- saga 在协调者 leader 切换后继续执行，**检测到 PFAU 挂起后立即补偿**
- saga 最终 `failed`，玩家余额恢复
- 协调者双 leader 通过 `version` CAS 切换无冲突（无脑裂）
- 监控告警：PFAU 永久挂起（critical 级）+ saga 失败（warning 级）

### §B.7.6 边界 + 异常

| 边界 | 行为 |
|---|---|
| **E7.1**：PFAU 升级成功（无节点掉线） | saga 继续推进（协调者等 match 域新版本 1.5x 启动时间后恢复），最终 `completed` |
| **E7.2**：协调者双 leader 脑裂（`version` CAS 冲突） | per ADR-0052 §2.3：DB 层 `version` 字段 CAS 强校验，后写入的 leader 检测到 `version` 已被对方推进 → 释放锁 + 重新加载 saga |
| **E7.3**：PFAU 期间 match 域从 3 节点降到 2 节点（1 节点恢复 + 1 节点仍在掉线） | per ADR-0052 §2.1 "PFAU 永久挂起"，不恢复；需 SRE 手动 `pfau abort` |
| **E7.4**：PFAU 期间 saga 涉及 match 域之外的其他域（economy/social） | 其他域不受 PFAU 影响，正常推进；只 match 域步骤被 PFAU 阻塞 |
| **E7.5**：PFAU 期间协调者自己 crash | Active-Active 另一 leader 通过 K8s lease 接管，继续 saga；新 leader 加载 `sagas.status='running'` + current_step 续跑 |
| **E7.6**：saga 处于 `pending_review` 时 PFAU 升级 match 域 | PFAU 不影响 admin 域（GM 审批），saga 继续 `pending_review`；PFAU 完成后 GM 审批通过即可续跑 |

---

## §B.8 总体验证标准

### §B.8.1 G-CODE-04 通过条件

- [x] §B.2~§B.7 6 场景全部覆盖（输入/状态机/DB/验证/预期/边界）
- [ ] §B.2 场景 1（正常路径）：5/5 step Completed，DB 状态全对，UT/IT 通过
- [ ] §B.3 场景 2（补偿路径）：失败步检测 + 反向补偿 + 余额恢复，UT/IT 通过
- [ ] §B.4 场景 3（超时路径）：deadline 检测 + 强制 Failed + 补偿，UT/IT 通过
- [ ] §B.5 场景 4（人工升级）：amount 阈值检测 + admin 队列 + GM 审批 + 续跑，UT/IT 通过
- [ ] §B.6 场景 5（去重路径）：同 `command_id` 重试返回原结果，DB 1 条 saga，UT/IT 通过
- [ ] §B.7 场景 6（PFAU + Saga）：Active-Active 协调者 + all-reachable 超时 + 补偿，UT/IT 通过
- [ ] 监控指标在 SLA 范围内（每步 < 100ms，补偿 < 1s，PFAU 挂起检测 < 130s）
- [ ] 具名责任人签字（一SRE Lead + economy 域 Lead + Ulysses 12 角色 per DEC-008）

### §B.8.2 不通过后果

- G-CODE-04 仍为 Open
- Q-003 仍为 Blocker
- 5 域独立 DB 拓扑跨域一致性不成立
- PH-1 业务 Saga 实施不能启动

### §B.8.3 关联 Gate

- 上游 Gate：G-CODE-06（Rust 1.98 + cargo build/test 全绿）+ G-CODE-03（5 独立 DB 拓扑图实际画过）
- 平行 Gate：G-CODE-01/02/05（5 域 manifest + 镜像构建 + SRE 接力）
- 下游 Gate：G-CODE-07（PH-1 业务 Saga 实施）

---

## §B.9 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-08-24 | Ulysses（一人公司 12 角色 per DEC-008） | 首版详细填充：覆盖 6 场景（正常/补偿/超时/人工升级/去重/PFAU+Active-Active），每场景含输入条件/状态机迁移/DB 状态/验证命令/预期结果/边界异常；关联 `saga.rs` / `saga_orchestrator.rs` / `inbox.rs` / `reservation.rs` + 0002_saga_init.sql + ADR-0052 |
