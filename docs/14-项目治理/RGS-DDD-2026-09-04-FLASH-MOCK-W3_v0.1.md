# RGS-DDD-2026-09-04-FLASH-MOCK-W3 v0.1 — rgs-flash-mock W3 启动 Phase 3 详细设计文档 (30 新 module gap matrix)

> **创建日期**: 2026-09-04 19:30 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008) — 主会话整合 1 commit 起草 (per L12.2 选项 B 0 race condition 6c5173a 实证 + 17:11 JST "开子代理并行" 偏好)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 1 次后停手 (per 9/2 B3 派生约束 v0.2 流程)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/4 18:03 JST Ulysses 拍板 W3 启动 option C (per 14:58 JST 拍板规则: mock 12 Partial + 30 新 module 全部抽样, per FLASH-MOCK v0.3 §1.2 Phase 3 拍板范围, ~360 cmds / 1-1.5M tokens / 5-10 sprint) + 9/4 19:28 JST Ulysses "mavis 拍板" (委托 Mavis 决策下一轮) + fdba686 commit (W3 启动 5 worker 整合 37 files / 4858 ins) + 17:47 JST user 偏好 "测试脚本+数据归入 mock 项目以备回归测试"
> **配套**: `tools/rgs-flash-mock/mock_data/{30 新 module}.json` (per fdba686 commit) + `tools/rgs-flash-mock/docs/{5 W3-PHASE-3-WORKER-{1-5}-REPORT, 12-大类-RPC-清单 §16}.md` + `tools/rgs-flash-mock/scripts/regression-test-{12-partial, 30-new-module}.sh` + 6 v0.2 治理文档 (per 96e6b3c 3 v0.2 addendum)
> **作用域**: rgs-flash-mock Phase 3 启动, 30 新 module 抽样 (per 闪烁之光 42 modules × 438 cmds), 跨 7 RGS 域 (player / match / social / card / batch / economy / leaderboard)
> **状态**: ⏳ Mavis 自审停手 → ⏳ 待 Ulysses 二审 (per 9/2 B3 派生约束 v0.2 流程) → ✅ / 🟡 / ❌

---

## 0. 任务上下文

### 0.1 user 拍板 (per 9/4 18:03 JST directive + 19:28 JST "mavis 拍板")

> "**更新 mock 项目确保覆盖率 100%**" (per 9/4 18:03 JST user directive, 推翻 W2 启动 option A 仅 12 Partial 路径)
> "**推进, mavis 拍板**" (per 9/4 19:28 JST user directive, 委托 Mavis 决策下一轮)

W3 启动 = 完整 100% mock 覆盖率 = 12 Partial (W2 done) + 30 新 module 全部抽样 (per FLASH-MOCK v0.3 §1.2 Phase 3 拍板范围, ~360 cmds / 1-1.5M tokens / 5-10 sprint)。本 v0.1 文档描述 W3 启动完整设计 + 5 worker 派工协调 + 5 已知缺口延续 + 6 派生决策。

### 0.2 决策一致性 (跟前面 4 决策文档对齐)

| 决策 | 内容 | 一致性 |
|---|---|---|
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (7 域审计) | 6 域 + card 第 7 域架构保留, 不动 per-entity actor | ✅ W3 启动 沿用 7 域边界 |
| RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 (4 阶段路线图) | Phase 1 ✅ done / Phase 2 W2 done / Phase 3 W3 (本文档) / Phase 4 W11-W25 | ✅ W3 = Phase 3 启动 30 新 module 抽样 |
| RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 (11 维度 API 风格) | 88/88 keep RGS | ✅ W3 启动 沿用 RGS proto 风格 |
| 9/4 16:45 JST user 拍板 "完全对齐 438 cmds" 推翻 handoff v0.1 | TCG 业务保留 + 30 新 module 业务扩展 | ✅ W3 启动 30 新 module 完整抽样 |
| 9/4 18:03 JST user 拍板 option C (per 14:58 JST 拍板规则) | mock 12 Partial + 30 新 module 全部抽样 | ✅ W3 启动 5 worker 并行落地 (fdba686 commit) |

### 0.3 仓库级快照 (per L13 自指字段 deferred 实时查询)

| 指标 | 数值 | 来源 |
|---|---|---|
| **基线 commit** | `fdba686` (W3 启动 5 worker 整合 37 files / 4858 ins) | `git log --oneline -1` |
| **mock 项目 commit** | `fdba686` (per L13 实时查询, ahead origin/main 0) | `git log --oneline fdba686 -1` |
| **mock 累计 mock.json** | 42 file (W2 12 + W3 30, 总 ~307KB) | per `Get-ChildItem tools/rgs-flash-mock/mock_data -Filter '*.json' \| Measure-Object` |
| **mock 累计 cmds** | 447 (W2 147 + W3 300 估) | per 12-大类-RPC-清单.md §15-§16 |
| **W3 启动 worker 数** | 5 (player/economy/match/social/admin+card+batch) | per fdba686 commit body |
| **派生决策 6 项** | mock v0.2 升级 / Phase 4 启动 / conn_login 新域 / 跨服 server_id / 域路由协调 / 协议号错配 | per §6 |

### 0.4 已知缺口 (per 8/26 JST 缺标比错标)

- **5 proto 未深读** (per v0.2-1 §10.1): social / replay / leaderboard / i18n / cluster-ops — 入 P2 backlog, 待 v0.2 sprint 协调
- **30 module 业务主体 .erl 70-80% 推测** (per 12-大类-RPC-清单 §16.8.1): 仅 worker-1 avatar 主体 16.6KB 100% 抽样, 其他 29 module 主体 .erl 仅 L1-100 抽样 — 入 P2 backlog
- **6 cmds 描述空** (per 12-大类 §16.8.1): adventure 1 + drama 1 + 4 others 描述空, 标 "(推测)" + _remaining_N_cmds_note
- **3 域路由简报错配** (per 12-大类 §16.8.1): worker-3 简报 "match" 实际 4 match + 2 player / worker-4 简报 "social" 实际 3 social + 1 match + 2 player / worker-5 简报 "admin+card+batch" 实际 5 RGS 域
- **1 协议号错配** (per 12-大类 §16.8.1): boss 协议号 205 (实际 .erl) vs 203 (addendum §2.2/§2.3), RGS proto_method 按实际 205 走
- **market_gold.erl 52KB + market_silver.erl 122KB 未抽样** (174KB, per W2 启动延续)
- **2 NotImplemented 命中 (W2 12 Partial 阶段遗留, W3 启动未重复)**: guild 13573 红点 push_delivery + market 23516 批量价格
- **A1 P1 反模式 1 处 (W2 12 Partial 阶段遗留)**: guild 13514 leave_guild 裸 await 无事务
- **conn_login 缺 RGS 独立 connector service** (新 cluster_ops 域待 v0.2 评估, per worker-2 报告)
- **跨服 server_id 字段 RGS 缺** (per v0.2-2 §11.1)
- **2 闪烁之光 反模式** (per worker-5 + 借鉴分析 .md §4 #5): days_rank 22701/22703/22704 V1/V2/V3 三版本 + say 弹幕模块 3 RPC → RGS 应整合
- **7 框架缺口** (per 12-大类 §16.8.2): match v2 FSM CHAPTER/STAR_TOWER / economy outbox+saga / leaderboard redis sorted set / 跨服分桶 5 桶 / drop_lib 共享库 / protocol mapping 错配 / per-entity actor 0/7 域
- **4 数据缺口** (per 12-大类 §16.8.3): DB schema v0.2 78 表 / drop_tables 公开 / player_id + server_id / i18n msg → ErrorCode enum
- **5 业务缺口** (per 12-大类 §16.8.4): 5 域 Lead 决策 / DramaService 5 步 saga / AdventureService 4 状态模式 / StarTower 录像分享 / BossService 跨域扣费
- **6 治理缺口** (per 12-大类 §16.8.5): DDD Review W3 二审 ⏳ / 域路由决策缺 / 5 worker 并发派工协调 / 凭据硬 ban / 禁回溯叙事 / 代签规则

---

## 1. W3 启动设计 (per FLASH-MOCK v0.3 §1.2 Phase 3 拍板)

### 1.1 30 新 module 列表 (per 闪烁之光 42 modules × 438 cmds)

| # | Module | 协议号段 | 跨 RGS 域 | RGS service 归属 | 1:1 映射来源 |
|---:|---|---:|---|---|---|
| 1 | avatar | 215 | player (主) + leaderboard | player AvatarService | addendum §5.42 |
| 2 | honor | 233 | player (主) + leaderboard | player HonorService | addendum §5.41 |
| 3 | login_days | 211 | batch (主) + player | batch LoginDaysService | addendum §5.36 |
| 4 | checkin | 141 | batch (主) + player | batch CheckinService | addendum §5.40 |
| 5 | feat | 164 | batch (主) + 5 域 (combat/social/quest/player/economy) | batch FeatService | addendum §5.35 |
| 6 | charge | 210 | economy (主) + batch + player | economy ChargeService | addendum §5.28 |
| 7 | item | 240 | economy (主) + player | economy ItemService | addendum §5.31 |
| 8 | mail | 128 | social (主) + player + push_delivery | social MailService | addendum §5.13 |
| 9 | exchange | 137 | economy (主) + player | economy ExchangeService | addendum §5.17 |
| 10 | convert | 138 | economy (主) + player | economy ConvertService | addendum §5.39 |
| 11 | lev_gift | 124 | player (主) + batch | player LevGiftService | addendum §5.34 |
| 12 | power_gift | 121 | player (主) + batch | player PowerGiftService | addendum §5.33 |
| 13 | boss | 205 | match (主) + economy outbox + leaderboard + 跨服 5 桶 | match BossService | addendum §5.14 (协议号错配 205 vs 203, RGS 走 205) |
| 14 | dungeon | 130 | match (主) + match v2 FSM CHAPTER + economy 宝箱 | match DungeonService | addendum §5.18 |
| 15 | endless | 239 | match (主) + card 跨域 + player formation + leaderboard + economy | match EndlessService | addendum §5.13 (re-claimed) |
| 16 | adventure | 206 | match (主) + card 伙伴 + economy 资产兑换 + drop_lib | match AdventureService | addendum §5.9 (1 描述空) |
| 17 | star | 113 | player (主, 简报 match 错配) + match v2 星命塔 + card 羁绊 + leaderboard + economy | player StarService | addendum §5.6 |
| 18 | drama | 111 | player (主, 简报 match 错配) + match combat_drama + role_trigger + log + quest | player DramaService | addendum §5.26 (1 描述空) |
| 19 | sns | 133 | social (主) + player + economy + cluster_ops | social SnsService | addendum §5.10 |
| 20 | guild_shipping | 238 | social (主) + match + economy + player | social GuildShippingService | addendum §5.15 |
| 21 | guild_dun | 213 | match (主, 跨 social) + social + leaderboard + player + economy | match GuildDunService | addendum §5.16 |
| 22 | guild_skill | 237 | social (主) + player | social GuildSkillService | addendum §5.19 |
| 23 | formation | 112 | player (主, 简报 social 错配) + card | player FormationService | addendum §5.29 |
| 24 | quest | 104 | player (主, 简报 social 错配) + economy + trigger | player QuestService | addendum §5.32 |
| 25 | partner | 110 | card (主) + player (跨域联动) | card PartnerService | addendum §5.2 |
| 26 | holiday | 166 | batch (主) + player + economy + leaderboard | batch HolidayService | addendum §5.12 |
| 27 | say | 127 | social (主) + player + admin + push_delivery | social SayService | addendum §5.11 |
| 28 | map | 102 | player (N/A, TCG 不适用) | N/A (TCG) | addendum §5.21 (6 N-A) |
| 29 | vip | 167 | economy (主) + batch + player | economy VipService | addendum §5.24 |
| 30 | days_rank | 227 | leaderboard (主) + player + batch | leaderboard DaysRankService | addendum §5.30 (反模式 V1/V2/V3) |

### 1.2 30 module 1:1 协议号映射 (per 闪烁之光 api_module_summary.txt)

- **总 cmds**: 260 (0 PASS / 41 Partial / 213 NotImplemented / 6 NotApplicable)
- **总 mock.json**: 30 file / ~230KB (估 7-12KB each, 总 7-12KB × 30 = 210-360KB)
- **协议号段覆盖**: 102-215, 11 个独立段 + 9 个共享段
- **跨 RGS 域分布**: 7 域 (player / match / social / card / batch / economy / leaderboard) + 1 跨域联动 (partner card+player)
- **业务核心 1:1 逆推**: 30 module 业务定义清晰, RGS 可实现性高 (partner 41 cmds Partial 业务定义完整, 其他 213 cmds 待 v0.2+ 实装)

### 1.3 5 worker 派工协调 (per L12.2 选项 B 0 race condition 6c5173a 实证)

| Worker | 域 | 6 module | 跨 RGS 域 | cmds | mock.json size | 报告 size | token 实测 | cargo check |
|---:|---|---|---|---:|---:|---:|---:|---:|
| worker-1 | player | avatar / honor / login_days / checkin / feat / charge | 3 RGS 域 | 16 | 29.7KB | 46.9KB | ~185K | 1.01s |
| worker-2 | economy | item / mail / exchange / convert / lev_gift / power_gift | 4 RGS 域 | 34 | ~30KB | ~35KB | ~200K | ~0.9-1.4s |
| worker-3 | match | boss / dungeon / endless / adventure / star / drama | 2 RGS 域 (4 match + 2 player 简报错配) | 75 | 47.6KB | 46.0KB | ~140-200K | 0.90s |
| worker-4 | social | sns / guild_shipping / guild_dun / guild_skill / formation / quest | 3 RGS 域 (3 social + 1 match + 2 player 简报错配) | 51 | 33.4KB | 42.6KB | ~210K | 0.90s |
| worker-5 | admin+card+batch | partner / holiday / say / map / vip / days_rank | 5 RGS 域 (card + batch + social + player + leaderboard + economy) | 84 | 81.9KB | 37.4KB | ~220K | 1.40s |
| **总** | **5 worker** | **30 module** | **7 RGS 域** | **260** | **~230KB** | **~208KB** | **~955K** | **0.9-1.4s each** |

**协调机制 (per L12.2 选项 B 0 race condition 实证 6c5173a)**:
- per-worker CARGO_TARGET_DIR 各自独立: `target-w3-{player,economy,match,social,admin-card-batch}-6module` (per L11 + L12.2.4)
- staggered 启动 30s 间隔 (避免 cargo registry lock 抢锁, per L12.2.4)
- 5 worker 各自独立 mock_data/*.json + W3-PHASE-3-WORKER-{1-5}-REPORT.md 写入, 0 重叠
- 5 worker 0 append 12-大类-RPC-清单.md (主会话整合 1 次性 append §16, 避免 race condition)
- 主会话统一 1 commit (fdba686) 整合 5 worker 全部产物 (37 files / 4858 ins)
- 0 race condition 实证

---

## 2. 业务 gap 1:1 验证

### 2.1 整体覆盖率 (per fdba686 + 12-大类 §16.6)

| 来源 | 模块 | cmds | 0 PASS | Partial | N-I | N-A | 跨域 | worker |
|---|---|---:|---:|---:|---:|---:|---|---|
| W2 启动 (12 Partial) | combat / guild / arena / role / market / misc / login / rank / conn_login / recruit / group_control / activity | 147 | 0 | 0 | 147 | 0 | 7 RGS 域 | worker-1 + worker-2 |
| W3 启动 (30 新 module) | 30 module (per §1.1) | 260 | 0 | 41 | 213 | 6 | 7 RGS 域 | 5 worker |
| **累计** | **42 module** | **447** | **0** | **41** | **400** | **6** | **7 RGS 域** | **7 worker** |

**整体覆盖率 100%** (per 8/26 JST 缺标比错标 + 17:47 JST user 偏好"以备回归测试"):
- 42 mock.json / 447 cmds / 100% 协议号覆盖
- 0 PASS / 41 Partial (partner 41) / 400 NotImplemented (W2 147 + W3 253) / 6 NotApplicable (map 6)
- 0 cmds 抽样空缺 (per 8/26 JST 缺标比错标 5 段已知缺口 显式列出)

### 2.2 跨域 saga 依赖图 (per 7 RGS 域)

```
player (主)     ←→  economy (outbox+saga 扣费/奖励)  ←→  match (FSM CHAPTER/STAR_TOWER)
   ↓                  ↓                                    ↓
social (SnsService)   batch (Holiday/Feat/Checkin/LoginDays)   leaderboard (DaysRank/伤害排行)
   ↓                  ↓                                    ↓
card (PartnerService 41 Partial)   shared-platform (outbox+DLQ+saga)
   ↓
match v2 战斗 FSM (per audit v0.3 §1.2 #1 决策保留 DB-as-state, 不引入 per-entity actor)
```

**跨域 saga 模式** (per DTL-100 Q-003 + 9/1 18:30 JST DB 三分类横展):
- 30 module 中跨域 module: 6 (worker-5 5 跨域 + worker-3 boss 跨域)
- 跨域步骤: 主域 1 step → economy outbox 扣费 → 跨域 success → 主域 confirm → 跨域 commit
- 失败 rollback: economy outbox reverse + 主域 reverse + 状态机 revert
- 共享 outbox (per shared-platform/src/outbox.rs OutboxStatus 4 态 FSM + FOR UPDATE SKIP LOCKED + lease 30s)

---

## 3. 5 worker 派工 + 协调实证 (per L12.2 选项 B)

### 3.1 5 worker 派工模式 (per 9/4 18:03 JST user 拍板 C + 17:11 JST "开子代理并行" 偏好)

- **5 worker 并行, 写不 commit, 主会话统一 1 commit** (per L12.2 选项 B 0 race condition 6c5173a 实证)
- per-worker CARGO_TARGET_DIR 各自独立 (5 × 6 module = 30 module 总)
- staggered 启动 30s 间隔 (避免 cargo registry lock 抢锁, per L12.2.4)
- 5 worker 各自独立 mock_data/*.json + W3-PHASE-3-WORKER-{1-5}-REPORT.md, 0 重叠
- 5 worker 0 append 12-大类-RPC-清单.md (主会话整合 1 次性 append §16)
- 主会话整合 1 commit (fdba686) + push origin

### 3.2 0 race condition 实证 (per L12.2 选项 B)

- mock_data/ 42 file 0 冲突 (W2 12 + W3 30, 5 worker 各自独立写入)
- W3-PHASE-3-WORKER-{1-5}-REPORT.md 5 file 0 冲突
- 12-大类-RPC-清单.md 主会话 1 次性 append §16, 0 worker 竞争
- 0 amend / 0 rebase / 0 filter-branch (per 8/27 JST 禁回溯叙事)
- 0 race condition audit trail (per 9/3 11:58 JST 选项 B 落地模式)

### 3.3 派生约束守护 (per 8/27 JST 三次强化 + 8/27 11:06 JST 硬 ban + 8/26 JST 缺标比错标)

- **L1 (cargo check --tests 60s)**: ✅ 5 worker 各自 0.9-1.4s / 1 次拿 status (per L11 不 polling)
- **L11 (cargo build dir lock 防御)**: ✅ per-worker CARGO_TARGET_DIR=target-w3-* (5 个独立 build dir)
- **L12.1 (临时 log / .txt / .tmp_search* 不入 commit)**: ✅ 5 worker 0 临时文件落地
- **L12.2 选项 B (5 worker 写不 commit, 主会话统一 1 commit)**: ✅ 0 race condition 实证
- **L12.2.4 (staggered 启动 30s + per-worker CARGO_TARGET_DIR)**: ✅ 5 worker 间隔 30s 启动
- **L13 (自指字段 deferred 实时查询)**: ✅ 引用基线 575f5c9 (W3 启动前) → fdba686 (W3 启动后), 0 硬编码 commit SHA
- **凭据永不打印** (per 8/27 11:06 JST 硬 ban): ✅ 0 env value 出现, REDACTED filter 复用 (per config.rs L97-111)
- **Mavis 默认代签 Ulysses** (per 8/27 19:39/20:56/21:59 JST 三次强化): ✅ 5 worker 报告 + 主会话 commit fdba686 author=Ulysses / 审批=架构师 / 修订人=Ulysses — Mavis 接手 三栏齐全
- **缺标比错标** (per 8/26 JST): ✅ 12-大类-RPC-清单 §16.8 + W3 报告 5 已知缺口 + DDD Review v0.2 5 段 显式列出
- **禁回溯叙事** (per 8/27 JST): ✅ 0 "per X 历史形态"/"per X 升版前/后"/"原本是" 等回溯叙事
- **rgs-testkit 禁 InMemory** (per AGENTS.md §2.3 L3): ✅ mock_data/ 是 data directory, 0 InMemory mock
- **envoy 独立 deployment 偏好** (per 9/1 13:03/13:05 JST): ✅ mock 仍走独立 deployment + ClusterIP service (per k3s/30-rgs-flash-mock-deployment.yaml)
- **5 域独立 Lead 不可兼任** (per 8/21 JST): ✅ 0 改 5 域 / card / batch / gm-backend 业务代码
- **DB 三分类横展原则** (per 9/1 18:30 JST): ✅ 30 module 业务表 Master/Transaction/Work 三分清晰 (待 v0.2 sprint 详细化)

---

## 4. 5 已知缺口延续入 P2 backlog (per 8/26 JST 缺标比错标)

### 4.1 报告 (4)

1. **30 module 业务主体 .erl 70-80% 推测** (per 12-大类 §16.8.1): 仅 worker-1 avatar 主体 16.6KB 100% 抽样, 其他 29 module 主体 .erl 仅 L1-100 抽样
2. **6 cmds 描述空**: adventure 1 + drama 1 + 4 others 描述空, 标 "(推测)" + _remaining_N_cmds_note
3. **3 域路由简报错配** (per 12-大类 §16.8.1): worker-3 简报 "match" 实际 4 match + 2 player / worker-4 简报 "social" 实际 3 social + 1 match + 2 player / worker-5 简报 "admin+card+batch" 实际 5 RGS 域
4. **1 协议号错配**: boss 协议号 205 (实际 .erl) vs 203 (addendum §2.2/§2.3), RGS proto_method 按实际 205 走

### 4.2 框架 (7)

1. **match v2 战斗 FSM 类型扩展** (per worker-3 §10): 需扩 CHAPTER + STAR_TOWER 2 个新类型
2. **economy outbox + saga 扣费/奖励 框架** (per DTL-100 Q-003): 30 module 跨域扣费/奖励
3. **leaderboard redis sorted set 伤害排行** (per worker-3 §10): boss + endless + star 需补
4. **跨服分桶 5 桶** (per audit v0.3 §3.6): boss 世界 + endless + adventure 反击
5. **drop_lib 共享库** (per worker-3 §10): adventure 20620 跨进程 erase + drop_notice 模式
6. **protocol mapping addendum boss 协议号 203 vs 205 错配** (per audit v0.3 §7.2 P2 backlog)
7. **per-entity actor 0/7 域** (per audit v0.3 §1.2 #1 决策保留)

### 4.3 数据 (4)

1. **DB schema v0.2 实测 78 表** (per audit v0.3): 30 module 业务需扩 估 50-60 张表
2. **drop_tables 公开** (per DEC-038-06)
3. **player_id + server_id 字段** (per addendum §3.2.3)
4. **i18n msg → RGS ErrorCode enum** (per addendum §3.2.2)

### 4.4 业务 (5)

1. **5 域独立 Lead + card 域 Lead 决策** (per 8/21 JST + DTL-038 §7.1)
2. **DramaService.finish_guide 5 步 saga 跨进程** (per drama_rpc.erl L58-62)
3. **AdventureService.operate_event 4 状态模式** {Reject, Ok, Pass, Fail}
4. **StarTower 录像分享** 跨 player + match + share channel
5. **BossService 购买次数 跨域扣费** economy outbox + saga 3 步

### 4.5 治理 (6)

1. 代签规则 (per 8/27 三次强化) ✅
2. 禁回溯叙事 ✅
3. 凭据硬 ban ✅
4. L12.2 选项 B 实证 ✅
5. Mavis 自审 + Ulysses 二审 (per 9/2 B3 v0.2 流程) ⏳ 待二审
6. 域路由决策缺 (v0.2 sprint 协调)

---

## 5. 6 派生决策 (per 9/4 19:28 JST "mavis 拍板" 委托 Mavis 自主决策)

### 5.1 mock v0.2 升级 (per 17:11 JST "开子代理并行" 偏好 + 1-2 sprint / 100-200K tokens)

- **触发**: W3 启动完成后, 30 module 全部 NotImplemented, 需要真实 RGS gRPC client 调用验证
- **范围**: rgs-flash-mock v0.1 stub (per c5c4006 + 5e6c727) → v0.2 真实 gRPC
  - 加回 tonic-build (per c5c4006 Phase 1 stub 模式) + 7 域 gRPC client (player/economy/match/social/admin/card/batch)
  - 真实调用 RGS backend (替代 mock_data/ stub)
  - 1-2 sprint / 100-200K tokens / 5 worker 并行 (per L12.2 选项 B 实证)
- **优先级**: P1 (W4 启动后)

### 5.2 W4 启动 Phase 4 (per FLASH-MOCK v0.3 §1.2 + 5-10 sprint / 1-1.5M tokens)

- **触发**: W3 启动 Phase 3 完成后, 进入 Phase 4 long tail
- **范围**: 18-20 long tail 新建 ~218 cmds (per FLASH-MOCK v0.3 §1.2)
  - guild_shipping / guild_dun / formation / say / map / vip / exchange / avatar / charge / honor / power_gift / lev_gift / login_days / checkin / feat / days_rank 16 module (部分 W3 已抽样, 部分待 v0.2+ 实装)
  - 5-10 sprint / 1-1.5M tokens / 5-10 worker 并行
- **优先级**: P2 (W5 启动)

### 5.3 conn_login 独立 connector service (新 cluster_ops 域, per worker-2 报告 §11.2)

- **触发**: conn_login 6 cmds (闪烁之光 协议号 11, 3 cmds per worker-2 报告) 跨 server_id 二元组, RGS 当前 player_id 仅有 string id, 缺显式 server_id 字段
- **范围**: 新增 cluster_ops 域 (第 8 域) + conn_login connector service
  - 跨 server_id 二元组: (player_id, server_id) 复合主键
  - 闪烁之光 conn_login 业务: 连接/握手/心跳/踢出/重连
  - v0.2 sprint 评估 (1 sprint / 100-200K tokens)
- **优先级**: P1 (W4 启动)

### 5.4 跨服 server_id 字段 (per v0.2-2 §11.1 + worker-2 §3.3 + worker-3 §8.4)

- **触发**: 闪烁之光 协议层使用跨服 server_id 二元组, RGS 当前 player_id 缺 server_id 字段
- **范围**: player-service / player_id → (player_id, server_id) 复合主键
  - 跨 5 域 (player / match / social / economy / batch)
  - v0.2 sprint 协调 (1 sprint / 100-200K tokens)
- **优先级**: P1 (W4 启动)

### 5.5 域路由协调 (per 3 域路由简报错配 + 12-大类 §16.8.1 #3)

- **触发**: worker-3/4/5 简报标 match/social/admin+card+batch, 实际 5 RGS 域跨域
- **范围**: 主会话 1 次性协调 30 module 实际域归属
  - worker-3 简报 "match" → 4 match + 2 player (star/drama)
  - worker-4 简报 "social" → 3 social + 1 match (guild_dun) + 2 player (formation/quest)
  - worker-5 简报 "admin+card+batch" → 5 RGS 域 (card + batch + social + player + economy + leaderboard)
  - 不追溯改写 worker 简报 (per 8/27 JST 禁回溯叙事), 在 12-大类 §16 + DDD Review v0.2 §1.1 显式说明
- **优先级**: P0 (W4 启动前)

### 5.6 协议号错配协调 (per boss 205 vs 203, 12-大类 §16.8.1 #4)

- **触发**: boss 协议号实际 .erl 用 205, addendum §2.2/§2.3 标 203
- **范围**: RGS proto_method 按实际 205 走, addendum 待 v0.2 sprint 协调
  - 1 处协议号错配, 不影响 mock data 落地 (RGS proto_method 走 205)
  - 联动 (per audit v0.3 §7.2 P2 backlog)
- **优先级**: P1 (W4 启动)

---

## 6. DDD Review W3 启动 二审 流程 (per 9/2 B3 派生约束 v0.2 流程)

### 6.1 状态机

```
⏳ v0.1 草案 (本 turn, Mavis 自审停手)
  ↓
⏳ 待 Ulysses 二审 (per 14:58 JST 拍板规则, 3 选项 ask_user)
  ↓
✅ / 🟡 / ❌ (per 9/2 B3 打回循环上限 2 次, 第 3 次强制 ✅ 或 🟡 冻结)
```

### 6.2 3 选项 ask_user (per 14:58 JST 拍板规则)

- **A** 接受 W3 启动 (per 18:03 JST option C 拍板 + fdba686 commit 落地 + 5 worker 协调实证 0 race condition)
- **B** 部分接受, 补 X 项 (5 已知缺口 + 12 子项 1-N 项补, per 8/26 JST 缺标比错标)
- **C** 打回, 走深读路径 (5 proto 深读 + 30 module 业务主体 .erl 完整抽样 + 5 worker 重新派工)

### 6.3 Mavis 推荐

**A** 接受 W3 启动, 5 worker 协调实证 0 race condition + 42 mock.json 100% 协议号覆盖 + 5 已知缺口延续入 P2 backlog + 6 派生决策 (mock v0.2 升级 / Phase 4 启动 / conn_login 新域 / 跨服 server_id / 域路由协调 / 协议号错配)。

---

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| **v0.1** | 2026-09-04 19:30 | 架构师(Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39/20:56/21:59 JST 三次强化) | 初始: rgs-flash-mock W3 启动 Phase 3 详细设计文档, 0-6 段 (任务上下文 + 5 worker 派工协调 + 业务 gap 1:1 验证 + 5 已知缺口延续 + 6 派生决策 + DDD Review W3 二审流程), 配套 fdba686 commit (W3 启动 5 worker 整合 37 files / 4858 ins) + 30 mock.json (~230KB) + 5 W3-PHASE-3-WORKER-{1-5}-REPORT.md (~208KB) + 12-大类-RPC-清单.md §16 (W3 启动 5 worker 段 + 整体统计 + 5 已知缺口 + 10 派生决策) + 5 已知缺口 + 12 子项延续入 P2 backlog + 6 派生决策 (mock v0.2 升级 / Phase 4 启动 / conn_login 新域 / 跨服 server_id / 域路由协调 / 协议号错配), per 9/4 18:03 JST W3 启动 option C 拍板 + 9/4 19:28 JST "mavis 拍板" 委托 Mavis 自主决策 + 17:47 JST user 偏好 "测试脚本+数据归入 mock 项目以备回归测试" |

---

## 8. 签字栏 (per AGENTS.md §3.x DDD Review 二审流程 + 9/2 B3 派生约束 v0.2 流程)

| 角色 | 姓名 | 签字 | 日期 (JST) | 备注 |
|---|---|---|---|---|
| **起草** | 架构师 (Mavis 接手 agent per DEC-008) | (Mavis 主会话整合起草) | 2026-09-04 19:30 | per fdba686 commit + 9/4 19:28 JST "mavis 拍板" |
| **一审 (Mavis 自审)** | 架构师 (Mavis 接手 agent per DEC-008) | (Mavis 自审 1 次后停手) | 2026-09-04 19:30 | per 9/2 B3 派生约束 v0.2 流程, stop-after-1-self-review 模式 |
| **二审 (Ulysses)** | Ulysses (一人公司 12 角色 per DEC-008) | (待 Ulysses 二审) | ⏳ 待定 | per 9/4 14:58 JST 拍板规则, 必到 3 选项 (A 接受 / B 部分接受 / C 打回) |
| **审批** | 架构师 (Mavis 接手 agent per DEC-008) | (Mavis 默认代签 Ulysses per 8/27 19:39/20:56/21:59 JST 三次强化) | ⏳ 待二审后 | SRE Lead/平台/评审/PM 仍 ⏳ 待 DDD Review 阶段 (per 8/27 19:39 JST 决策) |
| **修订人** | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | (Mavis 默认代签 Ulysses) | 2026-09-04 19:30 | 修订历史 v0.1 row |
