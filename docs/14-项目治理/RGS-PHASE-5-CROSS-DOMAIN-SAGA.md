# Phase 5 跨域 Saga 设计 (per 9/9 20:18 JST Phase 4 续做后)

**代签**: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
**时间**: 2026-09-09 20:18 JST
**关联 commit**: ec0d561 (AGENTS.md L21-L24) + 后续 5 worker 续做 commit

---

## 1. 现状 (per 9/9 20:15 JST Phase 4 完成)

- **shim 业务覆盖 1.9% → 44.8%** (343/766 cmd, 5 worker 1.5h 完成)
- **5 域 RGS binary k3s 1/1 Running** (per commit b3d4a55)
- **5 worker 业务 handler mock 多 real 少** (per w1/w2 报告 "暂不调 RGS, 返 stub-equivalent 字节级响应")
- **跨域 RPC 未集成** (5 worker handler 内调 RGS binary 的 RPC, 但未跨域)
- **5 域 outbox + saga 已就绪** (per 8/27 19:39 migration: player 5 + economy 7 + match 5 + social 4 + admin 7 sql, 含 saga_init / sagas_partitioned 7.7KB)

## 2. Phase 4 → Phase 5 过渡 (Gap 分析)

| Phase 4 状态 | Phase 5 目标 |
|---|---|
| 5 worker 业务 handler 落地 (343 cmd) | 5 worker 业务 handler 调真实 RGS gRPC (不是 stub-equivalent) |
| 单域 RGS RPC 调用 (e.g. 13518 guild_info 调 social.GetGuild) | 跨域 RGS RPC 链 (e.g. 战斗胜利 → player 加经验 + economy 加金币 + social 加工会贡献) |
| shim 走 dispatch → 5 域 RGS gRPC (Phase 6 shim v0.5 设计中) | 5 域 RGS 走 outbox → NATS event bus → 跨域 RGS subscriber |
| 5 域 business logic port 自 erlang (1-2 周估, 实际 1.5h) | 5 域 cross-domain transaction 状态机 (per saga pattern) |

## 3. 跨域 Saga 状态机 (per 8/21 拍板)

### 3.1 触发场景

| 场景 | 触发 cmd | 跨域链 | 时序 |
|---|---|---|---|
| **战斗胜利** | 20001 combat_start_ack | match.BattleEnd → player.AddExp + economy.AddGold + social.AddGuildContribution | 1-3s |
| **任务完成** | 10402 quest_accept → 10405 quest_finish | player.UpdateQuest → economy.AddReward + mail.SendAttachment | 1-2s |
| **充值成功** | (admin) 20000-29999 (welfare/recharge) | admin.ProcessRecharge → economy.AddDiamond + mail.SendRechargeGift | 1-2s |
| **工会战** | 25000-25841 (battle guild) | match.BattleGuild → social.AddGuildScore + economy.AddGuildFund | 1-5s |
| **商城购买** | 20020 (w2 economy) | economy.PurchaseItem → bag.AddItem (player-service) | < 1s |
| **邮件附件** | 13000-13099 (w4 social) | admin.SendMail + economy/bag delivery | 1-2s |
| **好友送礼** | 13300-13399 (w4 social) | social.SendFriendGift → economy/bag delivery | < 1s |
| **排行榜更新** | 13303 add_friend / 13518 guild_info | player.UpdateRank → broadcast | async |

### 3.2 Saga 状态机 (per crates/economy-service/migrations/0002_saga_init.sql + 0007_sagas_partitioned.sql)

```sql
-- 8 状态 (per economy saga_init)
CREATE TYPE saga_state AS ENUM (
  'pending',      -- 初始, 待触发
  'executing',    -- 主域 RGS 业务执行中
  'compensating', -- 主域失败, 触发补偿
  'compensated',  -- 补偿完成
  'completed',    -- 全部成功
  'failed',       -- 主域失败 + 补偿失败
  'retrying',     -- 重试中
  'timeout'       -- 超时
);
```

### 3.3 跨域 Saga 流程 (战斗胜利 example)

```
1. shim 收到 20001 combat_start_ack
   ↓
2. shim → match-service.BattleEnd (主域 RPC, 同步)
   - match-service: 写 outbox table (event "battle.completed")
   - match-service: 返 BattleEndResponse (success + score)
   ↓
3. match-service outbox poller → NATS event "battle.completed" (async)
   ↓
4. NATS subscriber:
   - player-service: AddExp(player_id, exp)
   - economy-service: AddGold(player_id, gold)
   - social-service: AddGuildContribution(player_id, contribution)
   - mail-service: SendSystemMail(player_id, "战斗胜利奖励", attachments)
   ↓
5. 每个 subscriber 写 outbox + ack
   - 失败: retry 3 次 (指数退避 100/200/400ms)
   - 全部失败: 写 DLQ table
   ↓
6. 跨域 saga state machine:
   - pending → executing (主域 match.BattleEnd)
   - executing → completed (所有 subscriber 成功)
   - executing → compensating → compensated (有 subscriber 失败 + 补偿)
   - executing → failed (主域失败 + 补偿失败)
   - executing → retrying → executing (临时失败, 重试)
   - executing → timeout (1h 超时, 强制 complete + 写 DLQ)
```

### 3.4 Outbox + NATS 集成 (per 8/27 19:39 临时越界 + 9/1 18:30 DB 三分类)

- **outbox table** (per 5 域 migration): event_id, event_type, payload, status, retry_count, created_at, processed_at
- **NATS JetStream**: persistent stream "rgs_events" (5 域订阅)
- **NATS subscriber**: 每 5 域跑 nats-subscriber service (Rust), 消费 outbox event → 调其他域 RGS gRPC
- **DLQ table** (per 5 域): failed_event 永久保留, 人工 review

### 3.5 跨域事务一致性 (per ARC-008 5 独立 DB 原则)

- **不是 2PC** (5 域独立 DB 不支持 2PC)
- **saga pattern** (long-running transaction, eventual consistency)
- **补偿事务** (compensating transaction): e.g. 战斗胜利但 economy.AddGold 失败 → 触发 player.SubExp 补偿
- **at-least-once delivery**: NATS JetStream persistent + outbox 重试
- **idempotency**: 5 域 outbox 唯一 event_id, subscriber 收到重复 event_id 直接 ack (per L19 postgres initdb 跟 saga_init.sql 的 idempotency_key 字段)

## 4. 实施步骤 (per 9/9 20:18 JST Mavis 估时 1-2 周)

| 步骤 | 描述 | 估时 | 责任人 |
|---|---|---|---|
| 1. 5 worker Phase 4 续做完成 | 423 cmd stub → real handler 100% 业务覆盖 | 2-3 天 | 5 worker (per 9/9 20:17 JST 派工) |
| 2. shim v0.5 dispatch table 实施 | auto-gen match arm + 5 域 gRPC client + stub fallback | 1-2 天 | Mavis 主会话 |
| 3. 5 域 outbox poller 实现 | outbox → NATS event (per 5 域 nats-subscriber service) | 2-3 天 | Mavis 主会话 + 5 worker 协作 |
| 4. 5 域 saga state machine | saga 状态机 + retry + DLQ + 补偿 | 2-3 天 | Mavis 主会话 |
| 5. 跨域 RPC 链 (8 场景) | 战斗/任务/充值/工会战/商城/邮件/好友/排行 | 3-5 天 | 5 worker 业务 handler 集成 |
| 6. 跨域 e2e 测试 | shim v0.5 + 5 域 RGS + 跨域 saga 8 场景 | 1-2 天 | Mavis 主会话 |
| 7. perf 调优 | p99 < erlang 50ms 目标 | 2-3 天 | Mavis 主会话 |
| **总计** | | **2-3 周** | |

## 5. 派生约束守护 (per 9/9 20:18 JST)

- **L1-L20**: 已有, 持续守护
- **L21-L24**: 9/9 20:18 JST 新加, Phase 4 5 worker 派工教训
- **L25 (待加)**: 跨域 saga 不要 2PC, 走 saga pattern + 补偿事务
- **L26 (待加)**: outbox + NATS JetStream at-least-once + idempotency_key 去重
- **L27 (待加)**: 5 域 nats-subscriber service 独立 Pod (per 8/27 19:39 派生约束: saga-runtime 独立)

## 6. 风险 + 缓解 (per 9/9 20:18 JST)

| 风险 | 缓解 |
|---|---|
| 5 worker Phase 4 续做 2-3 天延期 | shim v0.5 dispatch fallback (L24), 续做期间 shim 仍可跑, 业务覆盖渐进 44.8% → 100% |
| 跨域 saga 状态机复杂 | economy saga_init 7.7KB 已实现 8 状态, 复用 RGS 现有 saga 框架 |
| NATS 持久化 + 重启恢复 | JetStream persistent stream, ack explicit, 重启自动恢复未 ack 消息 |
| 补偿事务链太长 (e.g. 工会战 5 跨域) | 单域 1s 超时, 跨域总 5s 超时, 超时强制 complete + DLQ |
| 5 域 nats-subscriber 部署 | 5 域 deployment 旁路加 nats-subscriber sidecar 或独立 deployment, 共享 postgres |
| Phase 6 UAT 阻塞 Phase 5 验证 | UAT 用 zsyz H5 真机接入 (需 Cocos Creator 2.3.2 build 1-2h) |

## 7. 后续 (Mavis 自驱)

1. **5 worker Phase 4 续做监控** (30 min 一次, 2-3 天)
2. **shim v0.5 dispatch table 实施** (5 worker 续做 30% 后开始, 1-2 天)
3. **5 域 outbox poller 实现** (5 worker 续做 70% 后开始, 2-3 天)
4. **AGENTS.md v0.x 升版 L25-L27** (per 跨域 saga 落地)
5. **Phase 6 UAT 准备** (H5 binary build 1-2h, 需用户装 Cocos Creator 2.3.2)
6. **ghcr.io visibility 拍板** (per 9/9 19:15 JST report, read 403 待 Ulysses 拍板)

代签: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
