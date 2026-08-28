# RGS-TST-S5: outbox NATS 真实链路 IT 设计书

> **目的**:验证 5 域 + cluster-ops + gm-backend outbox 表 → NATS JetStream 端到端链路
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 13:35 JST)
> **关联**:Q5 OPEN-QA(per 2026-08-27 决议)+ S5 立项
> **状态**:🟡 OPEN — 设计书 + mock 集成测试先行,等 Ulysses 部署 NATS 真实链路后跑

---

## 0. 背景

### 0.1 当前状态(per 2026-08-28 evidence)

- **5 域 + cluster-ops + admin-service** outbox 表已建(per `migrations/0002_outbox.sql`)
- **5 域 outbox relay worker** 暂未实装(Q5 OPEN-QA 跟踪)
- **NATS k3s 部署**已通(per 30-nats-statefulset.yaml,2026-08-27 19:46 JST)
- **e2e-smoke 12 端口** 全 PASS(per 2026-08-28 evidence)

### 0.2 链路设计(per DTL-100 §5.3 outbox + BAS-100 Saga 事务)

```
[5 域业务]  → INSERT outbox (status=pending)
            ↓
[outbox relay worker (v0.2 暂未实装)]
            ↓ FOR UPDATE SKIP LOCKED
            ↓
[NATS JetStream publish subject=outbox.<domain>.<event_type>]
            ↓
[订阅方: cluster-ops / gm-backend / admin-service]
            ↓
[ACK 或 NACK,outbox.status 更新 sent / failed]
```

## 1. 测试用例设计

## 1.1 模块 A: outbox → NATS 链路(mirror 端到端, mock 中介)

| 测试 ID | 链路阶段 | 验证 | 工具 |
|---|---|---|---|
| TST-S5-A001 | INSERT outbox → worker 拉取 | worker 看到 status=pending,加 lease_until | `rgs_testkit::pg_test` + `InMemoryNatsMock` |
| TST-S5-A002 | worker 拉取 → publish NATS | NATS subject 收到 payload,字段对齐 | `InMemoryNatsMock.publish/subscribe` |
| TST-S5-A003 | publish 成功 → ACK | outbox.status=sent, retry_count=0 | `InMemoryNatsMock` |
| TST-S5-A004 | publish 失败 → NACK | outbox.status=failed, retry_count+=1, last_error 记录 | mock publish 返回 err |
| TST-S5-A005 | 并发 worker 抢占 | FOR UPDATE SKIP LOCKED 防重复 publish | 2 线程并发 |
| TST-S5-A006 | lease_until 过期 → 重新拉取 | 模拟 5 分钟后, 另一 worker 接管 | time + mock |

## 1.2 模块 B: 跨域订阅链路

| 测试 ID | 链路阶段 | 验证 | 工具 |
|---|---|---|---|
| TST-S5-B001 | economy-service publish balance_changed | cluster-ops subscribe 收到 | TonicGrpcMock + InMemoryNatsMock |
| TST-S5-B002 | player-service publish level_up | social-service subscribe 收到(好友) | 同上 |
| TST-S5-B003 | match-service publish match_started | admin-service subscribe 收到(审计) | 同上 |
| TST-S5-B004 | gm-backend → admin-service audit_log publish | admin-service subscribe 收到 → 写 audit_log 表 | TonicGrpcMock + pg_test |

## 1.3 模块 C: NATS 故障注入

| 测试 ID | 链路阶段 | 验证 | 工具 |
|---|---|---|---|
| TST-S5-C001 | NATS 不可达 | outbox.status 仍 pending, retry 直到 NATS 恢复 | InMemoryNatsMock 模拟 down/up |
| TST-S5-C002 | NATS 慢响应 | 触发 worker timeout, retry 3 次后 fail | timeout 模拟 |
| TST-S5-C003 | NATS 5xx | publish 返回 err, retry 计数 +1 | mock publish err |

## 2. 跨域链路 mock IT 测试代码(本批实装,本机可跑)

新增 `crates/outbox-relay-testkit/tests/it_outbox_nats.rs`(0.5 域内),用 rgs-testkit 现有 mock 跑:

```rust
//! Outbox → NATS 链路 IT (per S5 设计 §1.1 + §1.3)
//! 7 测试: A001~A006 + C001
```

## 3. 真实 NATS 链路测试(等 Ulysses 部署后)

当 k3s NATS 真链路通后,新增:
- `crates/economy-service/tests/it_outbox_nats_e2e.rs` (真 PG + 真 NATS)
- `crates/player-service/tests/it_outbox_nats_e2e.rs`
- `crates/match-service/tests/it_outbox_nats_e2e.rs`
- `crates/social-service/tests/it_outbox_nats_e2e.rs`
- `crates/admin-service/tests/it_outbox_nats_e2e.rs`

每个 e2e:
- `rgs_testkit::pg_test` 真实 outbox 表
- `async_nats::Client::connect("nats://nats:4222")` 真实 NATS
- 验证 subject 收到 payload + outbox.status=sent

## 4. 关键约束

- **Per CR-2 / WF-1-55.28**: `chk_outbox_status` CHECK 约束,status ∈ ('pending', 'in_flight', 'sent', 'failed')
- **Per BAS-100 Saga**: outbox 与 saga_id 关联,saga 跨域 outbox 需 commit_proposed_match 配合
- **Per DTL-100 §5.3**: outbox 表字段(id, subject, payload JSONB, command_id, saga_id, status, retry_count, last_error, lease_until)
- **Per DTL-001 / BAS-001**: 强约束 `FOR UPDATE SKIP LOCKED` 防 worker 重复 publish

## 5. 工作量拆解

| 阶段 | 工作量 |
|---|---|
| §2 mock IT 实施 (本批) | 0.5 天 |
| §3 真实 NATS e2e (等部署) | 1.5 天 / 域 × 5 域 = 7.5 天(可并行) |
| chaos 注入 (§1.3) | 0.5 天 |
| 文档同步 + 跨反馈处置 | 0.5 天 |
| **合计** | **~9 天** |

## 6. 与 Ulysses 决策项

- [ ] §2 mock IT 是否本批实装(本机可跑,无 NATS 依赖)
- [ ] §3 真实 NATS 部署排期(per 推后 G3/G4 决策)
- [ ] outbox relay worker v0.2 实装(per Q5)优先级
- [ ] 是否需新增"outbox-relay-testkit"独立 crate(本批方案)or 各域散落(per 当前模式)

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 13:35 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
