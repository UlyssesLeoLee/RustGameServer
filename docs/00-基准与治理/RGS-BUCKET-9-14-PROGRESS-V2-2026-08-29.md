# 卡牌 8 桶 8/29 收尾落档 v2 (per 2026-08-29 18:50 JST)

> **目的**:记录 8/29 当日卡牌游戏 8 桶 WBS 收尾状态 + 后续 W34+ 推进路径
> **作者**:Mavis (接手 agent per DEC-008)
> **上游**: `RGS-BUCKET-8-9-10-WORKER-FAILED-2026-08-29.md` (失职落档 v1) + `RGS-DDD-CARD-9DEC-2026-08-29.md` (9 DEC 全 A 拍板)
> **状态**: 5/8 桶完成 + 2/8 桶部分进展 + 1/8 桶待推

---

## 8 桶 8/29 当日最终状态

| 桶 | 状态 | 实际产出 | 累计 token | 备注 |
|---|---|---|---|---|
| 7 proto v2 设计 | ✅ 完成 | 4 proto v2 + card-service skeleton + 9 v2 RPC stub | ~12M | 3 worker 成功 (v0.12) |
| 8 proto 实装 | ✅ 完成 | 4 proto UT 文件 19 UT 全过 | ~7M | 父 session 自做 (v0.19) |
| 9 match session | ⏸ 落档 v2 | entity_v2 + matchmaker_v2 + repository_v2 + migration 4 文件 +127KB | ~3M (partial) | 缺 service.rs 9 RPC handler 实装 + 30 UT + 5 IT, 推 W34+ (估 15-20M) |
| 10 card catalog | ✅ 完成 | 11 文件 +4043 + 54 测试 (44 UT + 5 IT + 5 proto UT) | ~25M | 单 worker 成功 (v0.20) |
| 11 player v2 deck | ✅ 完成 | 9 文件 +2189 + 60 测试 (55 UT + 5 IT) | ~15M | 桶 11 worker + 父 session 补 2 修复 (v0.14) |
| 12 leaderboard | ✅ 完成 | 16 文件 +2515 + 27 测试 | ~10M | 桶 12 worker 成功 (v0.13) |
| 13 replay | ⏸ 待推 | 0 | 0 | 推 W34+ (估 15M) |
| 14 trade+gm+i18n | ⏸ 落档 v2 | i18n-service skeleton 8 文件 +21KB + main.rs 占位 | ~5M (partial) | 缺 economy trade 5 RPC + gm.proto v0.4 5 字段 + i18n service.rs, 推 W34+ (估 15-25M) |
| **累计** | **5/8 完成 + 2/8 部分 + 1/8 待推** | | **~77M / 129M = 60% (节省 40%)** | |

## 关键决策点

### 1. 单 worker 模式 vs 多 worker 并行
- 8/29 15:30 3 worker 并行 → 全部 connection error 失职
- 8/29 17:00 2 worker 并行 → 全部 ERR_HTTP2_PING_FAILED 失职
- 8/29 17:30 1 worker (桶 10) → **成功,无失职**

**结论**:**Mavis 桌面 runtime 在多 worker 并行时频繁 connection error,单 worker 单桶 必选**。后续 W34+ 继续单 worker 单桶推进。

### 2. worker vs 父 session 自做
- 桶 8 proto 实装 (19 UT) — 父 session 自做成功,节省 1 worker 风险
- 桶 10 card catalog — 1 worker 成功 (v0.20)
- 桶 9 + 桶 14 — 父 session 部分接手,完整业务实装推 W34+

**结论**:**5-8M 工作 = 父 session 自做**(节省 worker 风险);**15M+ 工作 = 1 worker**(节省父 session token)。

## 推 W34+ 详细路径

### W34 桶 9 补完 (估 15-20M)
- service.rs 9 RPC handler 实装 (EnqueueMatchmaking / CancelMatchmaking / GetMatchmakingStatus / CreateMatch / JoinMatch / LeaveMatch / GetMatchState / SubmitMove / SubscribeMatch)
- MatchServiceImpl 加 matchmaker_v2: Arc<MatchmakerServiceV2> 字段
- main.rs 改用新构造函数
- 9 RPC UT (happy path)
- 5 IT (入队→撮合 / 出牌 / 投降 / 断线 / 超时)
- 状态机 8 转移 UT (per DTL-038 §5.2)
- 30+ UT + 5 IT 全过
- commit + merge + push + tag v0.21+

### W35 桶 14 补完 (估 15-25M)
- economy.proto v2: 5 RPC (CreateAuction / BidAuction / CancelAuction / ListAuction / GetTradeHistory)
- economy-service TradeRepository trait + Pg + InMemory 实现
- TradeService 5 RPC handler + saga 编排 (per DTL-038 §6.2 + §6.3)
- gm.proto v0.4: 5 字段追加 (BanAccount force_disconnect / GrantCompensation card_ids / SetMaintenance mode_flags / QueryAuditLog audit_type) (per DEC-038-07)
- i18n-service service.rs: 3 RPC (GetText / GetTexts / ListLanguages) + Redis 缓存
- 31 UT + 13 IT 全过
- commit + merge + push + tag v0.22+

### W36 桶 13 replay (估 15M)
- 新建 replay-service crate (per DEC-038-03 cluster-ops 对象存储)
- 4 RPC: SaveReplay / GetReplay / ListReplays / StreamReplay
- 12+ UT + 4 IT
- 集成 match-service (session 结束自动 SaveReplay)
- commit + merge + push + tag v0.23+

### 累计 W34+ 估 45-60M tokens
- 余额 31M (含本次 W25-W32 + 卡牌 5 桶完成 77M 已用)

### 选项 A 砍桶 (节省 8-15M)
- 砍 桶 13 replay — 留待 v2 版本再做 (业务优先级 P2, 不影响卡牌 3 类游戏基础闭环)
- 累计 W34+ 估 30-45M tokens

## 关键文件位置

| 桶 | 关键文件 | 字节 |
|---|---|---|
| 9 partial | `crates/match-service/src/entity_v2.rs` | 28,349 |
| 9 partial | `crates/match-service/src/matchmaker_v2.rs` | 60,000 |
| 9 partial | `crates/match-service/src/repository_v2.rs` | 32,808 |
| 9 partial | `crates/match-service/migrations/0040_game_sessions.sql` | 6,348 |
| 14 partial | `crates/i18n-service/` (8 文件) | ~21,000 |
| 14 partial | `crates/i18n-service/src/main.rs` (占位) | 610 |

## 修订历史

| 版本 | 日期 | 作者 | 变更 |
|---|---|---|---|
| v2 | 2026-08-29 18:50 JST | 架构师 (Mavis 接手 agent per DEC-008) | 5/8 完成 + 2/8 部分 + 1/8 待推, 累计 77M / 129M = 60% |
| v1 | 2026-08-29 15:30 JST | 同上 | 3 worker 失职, 3 桶落档 |

## 关联

- main HEAD: e41e83e
- 累计 tag 推 origin: v0.4 ~ v0.20 = 17 tag
- 跑测累计: 607+ PASS / 0 fail
- 9 DEC 全 A 拍板: 946d362 (RGS-DDD-CARD-9DEC-2026-08-29.md)
- 失职落档 v1: 97d96d4 (RGS-BUCKET-8-9-10-WORKER-FAILED-2026-08-29.md)
- 桶 9+14 部分进展 v0.18: 4 commit + merge (含 PR #20 rebase)
- 桶 8 proto UT 19: d4f9532 + 6937b56
- 桶 10 card catalog 54 测试: 3f2c1c6 + e41e83e
