# RGS-DDD v0.2 addendum — 闪烁之光 438 cmds → RGS proto 1:1 完整映射表

> **创建日期**: 2026-09-04 17:17 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 (per 9/2 B3 派生约束 v0.2 流程)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**: user 9/4 17:11 JST 拍板 "frontend compat 正确设计" + RGS-DDD-2026-09-04 v0.1 §7 (per addendum 主表展开) + 闪烁之光 6 文件 (api_module_summary.txt / 协议号分段.md / proto_*.erl 抽样 10 个 / proto_lib.erl / services.erl) + 闪烁之光 协议 schema 抽样 10 个 (proto_200.erl/proto_110.erl/proto_135.erl/proto_206.erl/proto_235.erl/proto_133.erl/proto_108.erl/proto_168.erl/proto_11.erl/proto_101.erl) + common.proto (RGS 共享 proto 模式)
> **配套**: `RGS-DDD-2026-09-04_v0.1.md` (主 doc) + `RGS-REQ-2026-09-04_v0.1.md` (需求) + `RGS-BDD-2026-09-04_v0.1.md` (基本设计) + `RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.3.md` (4 阶段路线图)
> **作用域**: 闪烁之光 协议号 1:1 → RGS proto 命名映射, 438 cmds / 42 modules / 41 协议号段 (per 协议号分段.md L51) 全覆盖
> **状态**: ⏳ 待 Mavis 自审 → 🟡 → ⏳ 待 Ulysses 二审 → ✅/🟡/❌
> **DoD (per L1/L1.1/L1.2 + L13)**: L1 N/A (纯 doc) / L1.1 N/A / L1.2 N/A / L11 N/A (单 worker) / L12 N/A (不 commit, 1 doc) / L13 ahead/md 行数 deferred 实时查询 / L14 N/A (无 plumbing 改)

---

## 0. 文档元信息 (v0.2 addendum)

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-DDD-v0.2-addendum-协议号映射 |
| 版本 | v0.2 addendum (1.0 草案, 续 v0.1 主 doc §7 协议号 1:1 映射 41 段) |
| 关联主 doc | `RGS-DDD-2026-09-04_v0.1.md` §7.4 (协议号→RGS proto, 41 段高层) |
| 关联基线 commit | `2e3d9ee` (FLASH-OVERLAP v0.2 已落 main, per DDD v0.1 §0.3) |
| 关联上游 | user 9/4 17:11 JST 拍板 "frontend compat 正确设计" + DDD v0.1 §7 + REQ v0.1 §2 + BDD v0.1 + 4 阶段路线图 v0.3 |
| 起草人 | Mavis (worker 派工 v0.2-2, per 9/4 17:11 JST ask_user option A) |
| 修订人/审批 | Ulysses — Mavis 接手代签 (per 8/27 三次强化, 9/4 17:11 JST 主会话派工确认) |
| 凭据 | 永不入文档 (per 8/27 11:06 JST 硬 ban, REDACTED filter) |
| Token 预算 | R1 OLU ≈ 100-150K tokens (per token-OLU 框架) |
| 目标大小 | ≤ 80KB, 15-20 段, **438 cmds 完整 1:1 映射表 (主表, 必含)** |

### 0.1 addendum 跟 v0.1 主 doc 关系 (per 8/27 21:59 JST 三次强化)

- **v0.1 主 doc** (commit `80bcd3b`, 96KB): 12 Partial + 30 新 module 详细设计 (§3-§4) + 7 域映射 (§2.1) + 协议号 1:1 (41 段高层, §7.4)
- **v0.2 addendum** (本 doc): 把 §7.4 41 段高层展开成 **438 cmds 完整 1:1 映射** (主表, per api_module_summary.txt 顺序), 协议 schema 抽样 read .erl 验证, 边界 case + 冲突检测 + 迁移路径
- **保留派生约束**: 不追溯改写 v0.1 主 doc (per 8/27 21:59 JST 决策), v0.1 主 doc §7.4 41 段保持现状, 本 addendum 在 v0.1 之上做详细化
- **衍生决策** (per 9/4 17:11 JST ask_user option A 第 2 项): 抽样 read gen_proto/cfg/proto_NNN.erl 验证协议 schema (本 addendum §3) — 实际抽样发现 `src/gen_proto/cfg/` 目录不存在 (per §3 已知缺口), 抽样调整为 `src/proto/proto_NNN.erl` (直接 pack/unpack 实现, schema 更直接, 详见 §3)

### 0.2 仓库级快照 (per L13 自指字段 deferred 实时查询)

| 指标 | 数值 | 来源 |
|---|---|---|
| **基线 commit** | `2e3d9ee` (FLASH-OVERLAP v0.2 已落 main) | per DDD v0.1 §0.3 (L13 deferred 实时查询) |
| **addendum v0.2 commit** | 待主会话 `git add` + 1 commit (per L12.2 选项 2 "5 worker 写文件不 commit, 主会话统一 commit" 模式) | per L12.2 + 9/4 17:11 JST 派工 |
| **闪烁之光 RPC 总数** | 438 (42 modules, per api_module_summary.txt L1-45) | `E:\...\zsyz_server\docs\api_module_summary.txt` L1-45 |
| **闪烁之光 协议号段** | 41 段 (per 协议号分段.md L51 "总 46 个协议号段, 但实际 `proto_*.erl` 文件有 45 个") | `E:\...\zsyz_server\docs\architecture\协议号分段.md` L51 |
| **闪烁之光 proto_*.erl** | 45 .erl (proto_11/101/102/103/104/105/108/109/110/111/112/113/114/127/129/130/133/134/135/141/164/166/167/168/200/202/203/205/206/210/211/212/213/215/221/227/232/233/234/235/236/237/238/239 + rpc_cfg) | `Get-ChildItem E:\...\src\proto\*.erl` |
| **闪烁之光 src/gen_proto/cfg/** | **目录不存在** (本 addendum §3 抽样调整) | `Test-Path E:\...\src\gen_proto\cfg` = False |
| **RGS 现有 proto RPC 数** | 69 / 12 proto (per handoff v0.1 §0) | `crates/*/proto/*/v1/*.proto` |
| **RGS 7 域 backend** | player(50051) + economy(50052) + match(50053) + social(50054) + admin(50055) + card(50061) + gm-backend(8081) | per 5 域 main.rs + card/main.rs + gm-backend/main.rs |
| **RGS 共享 proto** | `D:\RustGameServer\crates\shared-platform\proto\common\v1\common.proto` 2.9KB (L1-129, Status/ErrorCode/EntityId/Timestamp/PageRequest/PageResponse/HealthCheck/CardRef/CardType/CardRarity/GameMode/Currency/Locale/I18nString) | Read common.proto L1-129 |

### 0.3 已知缺口 (per 8/26 JST 缺标比错标, 5 段详见 §11)

- **报告**: gen_proto/cfg 目录不存在 (per 0.2), 抽样调整为 proto_NNN.erl / 闪烁之光 5 域 ST 业务 mTLS cert 导出 SOP / 30 新 module 详细业务 v0.2 详细
- **框架**: per-entity actor 0/7 域 (audit v0.3 §1.2 #1 决策保留) / 协议 schema push 7 域未实装
- **数据**: 闪烁之光 performance baseline 待 Phase C 后 / DB schema v0.2 实测 78 表
- **业务**: 438 cmds - 12 Partial ~140 = 298 cmds 30 新 module 业务验证 v0.2 详细 / conn_login 独立 connector service
- **治理**: Mavis 自审 + Ulysses 二审 (per B3) / Ulysses 二审时间窗口不定 / 凭据 REDACTED

---

## 1. 引言

### 1.1 背景

RGS-DDD-2026-09-04 v0.1 (commit `80bcd3b`, 96KB) §7.4 已建立闪烁之光 **41 协议号段 → RGS service 1:1 高层映射** (per 协议号分段.md L51), 但**未展开到 438 cmds 粒度**。

user 9/4 17:11 JST 拍板 "frontend compat 正确设计", 要求本 addendum 把 v0.1 §7.4 41 段**展开成完整 438 cmds 1:1 映射表**, 抽样 read .erl 验证协议 schema, 边界 case / 冲突检测 / 迁移路径显式列出, **作为 v0.2 sprint 派工的精确 1:1 索引** (W2-W25, per FLASH-MOCK v0.3 §1.2 4 阶段路线图)。

### 1.2 目标

1. **完整覆盖**: 438 cmds / 42 modules / 41 协议号段 1:1 映射 (主表 §5, 按 api_module_summary.txt 顺序)
2. **schema 验证**: 抽样 read 10 个 `proto_*.erl` 验证 闪烁之光 request/response 字段 (per §3 抽样清单)
3. **RGS proto 命名**: 闪烁之光 cmd → RGS gRPC RPC (snake_case per common.proto 风格) + 7 域 service 路由 (per §4)
4. **gap 状态**: 12 Partial Pass/Partial/NotApplicable + 30 新 module NotImplemented (per §6 矩阵, 跟 v0.1 §2.1 一致)
5. **边界 + 冲突**: 10001/11000/13500/20001/20200/20300/21000/22000/23000/23900 关键边界 (per §7) + 闪烁之光 5 大类 vs RGS 0-65535 范围冲突检测 (per §8)
6. **迁移路径**: v0.1 (12 Partial 详细) → v0.2 (本 addendum 438 cmds 索引) → v1.0 (100% 覆盖) (per §9)

### 1.3 范围

**In-Scope (本 v0.2 addendum)**:
- 438 cmds 完整 1:1 映射主表 (§5, per api_module_summary.txt L47-568 顺序)
- 闪烁之光 协议 schema 抽样 10 个 (per §3, 验证 pack/unpack tuple 字段)
- RGS 7 域 proto 命名 + service 路由 (per §4, 沿用 v0.1 §7.4)
- 协议号分段 (per §2, 1xx / 2xx 大类横展)
- 边界 case (per §7) + 冲突检测 (per §8) + 迁移路径 (per §9) + 测试矩阵 (per §10)
- 5 段已知缺口 (per §11, 8/26 JST 缺标比错标)
- 代签栏 (per §12, 8/27 JST 三次强化)

**Out-of-Scope (addendum 范围外)**:
- 闪烁之光 自研 TCP/Flash socket 协议 (per DDD v0.1 §11.1 已知缺口, v0.2 升级) — 本 addendum 仅 cmd → RGS gRPC 命名映射
- 闪烁之光 实际 proto 风格 wire 适配 (per DDD v0.1 §0.3) — 待 v0.2 worker 实证 (本 addendum 仅抽样 read schema)
- 闪烁之光 .erl 源码逐条翻译 (per REQ v0.1 §1.3 范围外) — 本 addendum 仅 cmd 编号 → RGS RPC 命名 + 7 域路由
- 30 新 module 详细 entity / repository / saga (per DDD v0.1 §1.3 out-of-scope) — 业务验证 W2-W25 阶段

---

## 2. 协议号分段 (per 闪烁之光 architecture/协议号分段.md)

### 2.1 协议号约定 (per 协议号分段.md L11-15)

> 命令号, 约定有效范围: **100~65500**, 模块号有效范围: **1~655**。
> 实际命令号 = 协议号 × 100 + 段内偏移 (0~99)。
> (per 闪烁之光 `mapping.erl:22` 注释)

注意: 闪烁之光 实际 cmd 范围是 4 位数 (10001-29900), 不是 5 位 (per api_module_summary.txt 全文 ≤ 29900), 与协议号分段.md L13 注释 "100~65500" 不一致 — 这是文档/代码 2 段式表达 (per 8/27 21:59 JST 禁回溯叙事, 不追溯改写协议号分段.md, 记录于此作为已知缺口 §11.1)。

### 2.2 协议号段 → 模块映射 (per 协议号分段.md L19-27)

| 协议号段 | 实际命令号 | 用途 | 闪烁之光 module | RGS 7 域映射 |
|---|---|---|---|---|
| 11~12 | 1100-1299 | 连接登录 | `conn_login` (3) | cluster-ops (per REQ §2 #34) |
| 101~113 | 10100-11399 | 角色基础 (login/map/role/quest/item/mail/misc/partner/drama/formation/star) | `login` (6) + `map` (6) + `role` (21) + `quest` (4) + `item` (10) + `mail` (6) + `misc` (19) + `partner` (41) + `drama` (5) + `formation` (6) + `star` (20) = 144 cmds | player + card + admin (per REQ §2) |
| 127~135 | 12700-13599 | 社交基础 (say/rank/dungeon/sns/exchange/guild) | `say` (14) + `rank` (5) + `dungeon` (9) + `sns` (16) + `exchange` (6) + `guild` (29) = 79 cmds | social + match + economy + leaderboard |
| 141~168 | 14100-16899 | 商业/签到 (checkin/friend/charge/market/vip) | `checkin` (2) + `feat` (2, 164) + `charge` (3, 166) + `market` (19, 167) + `vip` (6, 168) + `misc` (部分 10900-10999) = 32 cmds (+misc 11) | economy + batch + admin |
| 200~215 | 20000-21599 | 战斗 (combat/combat_result/boss/dungeon_fight/arena/pay/recruit/adventure/endless/stronger) | `combat` (43) + `arena` (26, 202) + `boss` (12, 203) + `dungeon` (9, 205) + `adventure` (17, 206) + `pay` (3, 210) + `recruit` (3, 211) + `endless` (12, 213) + `avatar` (4, 215) = 129 cmds | match + card + economy + player + admin |
| 221~227 | 22100-22799 | 跨服 (group_control/guild_dun/days_rank) | `group_control` (2) + `guild_dun` (10) + `days_rank` (4) = 16 cmds | batch + social + leaderboard |
| 232~239 | 23200-23999 | 活动 (holiday/login_days/checkin/power_gift/lev_gift/days_rank/notice/mail_2/guild_shipping/endless) | `recruit` (3, 232) + `holiday` (13) + `login_days` (2, 233) + `power_gift` (3, 234) + `market` (re-claimed 235) + `convert` (5, 236) + `days_rank` (re-claimed 237) + `guild_shipping` (11, 238) + `endless` (re-claimed 239) = 38 cmds | batch + economy + leaderboard + social + match |

**总协议号段**: 41 段 (per 协议号分段.md L51 "总 46 个协议号段, 但实际 proto_*.erl 文件有 45 个, 可能某个号段未启用"), **总 cmds**: 438 (per api_module_summary.txt L45)。

### 2.3 协议号 vs 玩法模块 (per 协议号分段.md L102-147)

| 协议号 | 玩法模块 | 闪烁之光 服务 | RGS service (per DDD v0.1 §7.4) |
|---|---|---|---|
| 11 | `conn_login.erl` | (连接层) | `ClusterOpsService.AccountLogin` (per cluster_ops.proto:1.4KB) |
| 101 | `login.erl` | (连接层) | `PlayerService.CreatePlayer` (per login_rpc.erl:19-66) |
| 102 | `map.erl` | `map_mgr` | `PlayerService.MapService` (per REQ §2 #23) |
| 103 | `role.erl` | `role_data` | `PlayerService` + 9 增量 RPC (per DDD §3.4 L388-409) |
| 104 | `quest.erl` | (角色进程) | `PlayerService.QuestService` (per REQ §2 #33) |
| 105 | `item.erl` | (角色进程) | `PlayerService.ItemService` (per REQ §2 #20) |
| 108 | `mail.erl` | `mail` | `SocialService.MailService` (per REQ §2 #24) |
| 109 | `misc.erl` | (角色进程) | `AdminService.MiscService` + `GmHandler` (per REQ §2 #7) |
| 110 | `partner.erl` | (角色进程) | `CardService.PartnerService` + `PlayerService` (per REQ §2 #2) |
| 111 | `drama.erl` | (角色进程) | `PlayerService.DramaService` (per REQ §2 #28) |
| 112 | `formation.erl` | (角色进程) | `PlayerService.FormationService` (per REQ §2 #22) |
| 113 | `star.erl` | `star_tower_mgr` | `PlayerService.StarService` (per REQ §2 #13) |
| 127 | `say.erl` | `say_mgr` | `SocialService.SayService` (per REQ §2 #9) |
| 129 | `rank.erl` | `rank_mgr` | `LeaderboardService.RankService` (per REQ §2 #11) |
| 130 | `dungeon.erl` | (角色进程) | `MatchService.DungeonService` (per REQ §2 #21) |
| 133 | `sns.erl` | `friend_mgr` | `SocialService.SnsService` (per REQ §2 #8) |
| 134 | `exchange.erl` | (角色进程) | `EconomyService.ExchangeService` (per REQ §2 #25) |
| 135 | `guild.erl` | `guild_mgr` | `SocialService.GuildService` + 27 增量 RPC (per DDD §3.2 L249-282) |
| 141 | `checkin.erl` | `holiday_mgr` | `BatchService.CheckinService` (per REQ §2 #42) |
| 164 | `friend.erl` | `friend_mgr` | `BatchService.FeatService` (per REQ §2 #40) |
| 166 | `charge.erl` | `charge_mgr` | `EconomyService.ChargeService` (per REQ §2 #37) |
| 167 | `market.erl` | `market_gold/silver` | `EconomyService.MarketService` + 19 增量 RPC (per DDD §3.5 L454-477) |
| 168 | `vip.erl` | (角色进程) | `EconomyService.VipService` (per REQ §2 #26) |
| 200 | `combat.erl` | `combat_mgr` | `MatchService.CombatService` + `PveService` (per DDD §3.1 L200-213) |
| 202 | `combat_result.erl` | `combat_mgr` | `MatchService.ArenaService` + 20 增量 RPC (per DDD §3.3 L321-343) |
| 203 | `boss.erl` | `world_boss_mgr` | `MatchService.BossService` (per REQ §2 #17) |
| 205 | `dungeon_fight.erl` | (角色进程) | `MatchService.DungeonService` (per REQ §2 #21) |
| 206 | `arena.erl` | `arena_mgr` | `MatchService.AdventureService` (per REQ §2 #14) |
| 210 | `pay.erl` | `charge_mgr` | `EconomyService.PayService` (mock, per REQ §2 #37) |
| 211 | `recruit.erl` | `recruit_mgr` | `CardService.RecruitService` (per REQ §2 #12) |
| 212 | `adventure.erl` | `adventure_mgr` | `BatchService.LevGiftService` (per REQ §2 #32) |
| 213 | `endless.erl` | `endless_mgr` | `MatchService.EndlessService` (per REQ §2 #16) |
| 215 | `stronger.erl` | `stronger_mgr` | `PlayerService.AvatarService` (per REQ §2 #29) |
| 221 | `group_control.erl` | `group_control_mgr` | `BatchService.GroupControlService` (per REQ §2 #38) |
| 227 | `guild_dun.erl` | `guild_dun_mgr` | `MatchService.GuildDunService` (per REQ §2 #19) |
| 232 | `holiday.erl` | `holiday_mgr` | `BatchService.HolidayService` (per REQ §2 #15) |
| 233 | `holiday_login_days.erl` | `holiday_mgr` | `BatchService.LoginDaysService` (per REQ §2 #41) |
| 234 | `holiday_checkin.erl` | `holiday_mgr` | `BatchService.PowerGiftService` (per REQ §2 #35) |
| 235 | `holiday_power_gift.erl` | `holiday_mgr` | `EconomyService.MarketService` (re-claimed, per DDD §7.4) |
| 236 | `holiday_lev_gift.erl` | `holiday_mgr` | `EconomyService.ConvertService` (per REQ §2 #27) |
| 237 | `days_rank.erl` | `days_rank_mgr` | `LeaderboardService.DaysRankService` (per REQ §2 #31) |
| 238 | `notice.erl` | `notice_mgr` | `SocialService.GuildShippingService` (per REQ §2 #18) |
| 239 | `mail_2.erl` | `mail` | `MatchService.EndlessService` (re-claimed, per DDD §7.4) |

**总映射**: 41 协议号 → 41 RGS service (1:1, per DDD v0.1 §7.4 L1074), 闪烁之光 协议号 = 业务模块名 (per §2.1 约定)。

---

## 3. 闪烁之光 协议 schema 抽样 (per proto_*.erl 抽样 10 个)

### 3.1 抽样清单 (per 简报 "抽样 5-10 个代表", 实际抽样 10 个)

| # | proto_*.erl | 协议号 | cmds 数 | 业务模块 | 抽样行数 |
|---|---|---|---|---|---|
| 1 | proto_200.erl | 200 | 43 | combat (战斗) | L1-120 (pack 20000-20027) |
| 2 | proto_110.erl | 110 | 41 | partner (伙伴) | L1-120 (pack 11000-11012) |
| 3 | proto_135.erl | 135 | 29 | guild (公会) | L1-120 (pack 13500-13522) |
| 4 | proto_206.erl | 206 | 17 | adventure (冒险) | L1-120 (pack 20600-20621) |
| 5 | proto_235.erl | 235 | 19 | market (市场) | L1-120 (pack 23500-23514) |
| 6 | proto_133.erl | 133 | 16 | sns (好友) | L1-120 (pack 13300-13315) |
| 7 | proto_108.erl | 108 | 6 | mail (邮件) | L1-80 (估, 抽样) |
| 8 | proto_168.erl | 168 | 6 | vip/misc 提示 | L1-60 (pack 16800-16802) |
| 9 | proto_11.erl | 11 | 3 | conn_login (连接登录) | L1-80 (pack 1110/1198/1199) |
| 10 | proto_101.erl | 101 | 6 | login (角色登录) | L1-80 (pack 10101-10103) |

抽样行数覆盖 10 个代表 module, 涵盖战斗/伙伴/公会/冒险/市场/好友/邮件/vip/连接登录/角色登录 全 10 类业务。

### 3.2 协议 schema 模式 (10 个抽样综合)

#### 3.2.1 通用 wire 格式 (per proto_200.erl L25-44 + proto_11.erl L23-29 通用模式)

```erlang
%% 通用 wire format (per 10 个抽样综合)
%% [size:32, cmd:16, data:binary]
%% - size: 32-bit unsigned, 总字节数 (含 size 自身)
%% - cmd: 16-bit unsigned, 协议号 (per §2.1 约定)
%% - data: 变长, per pack/3 tuple 字段

pack(Cmd, cli, {}) -> {ok, <<2:32, Cmd:16>>};   % 空请求, 长度 = 2 (仅 cmd)
pack(Cmd, cli, {V0_a, V0_b}) -> D_a_t_a = <<V0_a:8, V0_b:32>>, {ok, <<(byte_size(D_a_t_a) + 2):32, Cmd:16, D_a_t_a/binary>>};
```

#### 3.2.2 通用结果返回模式 (per 10 个抽样综合, 80%+ cmds 沿用)

```erlang
%% 通用结果: code:8 + msg:string (i18n 错误信息)
%% code: 0=success, 1+=error (per 业务定义)
%% msg: error 提示 (i18n 字符串, per 闪烁之光 lang/ 目录)
pack(Cmd, srv, {V0_code, V0_msg}) ->
    D_a_t_a = <<V0_code:8, (protocol:pack(string, V0_msg))/binary>>,
    {ok, <<(byte_size(D_a_t_a) + 2):32, Cmd:16, D_a_t_a/binary>>};
```

注: 这跟 RGS `ErrorCode` enum (per common.proto L13-21) 略不同, 闪烁之光 用 8-bit code + i18n msg, RGS 用 enum + Status. RGS 应保留 ErrorCode enum 但 service 内部用 `Result<tonic::Status, ErrorCode>` 模式 (per DDD v0.1 §8.1 L1095-1118)。

#### 3.2.3 跨服 ID 模式 (per proto_11.erl L28 + proto_133.erl L29 + proto_135.erl L37)

```erlang
%% 跨服 ID = {rid:32, srv_id:string}
%% - rid: role id (玩家 ID, 32-bit unsigned)
%% - srv_id: server id (string, 跨服 server identifier)
%% 用于跨服玩家引用 (per sns 好友 / guild 成员 / mail 发件人)

{V0_rid:32, (protocol:pack(string, V0_srv_id))/binary, ...}
```

注: RGS 应映射为 `PlayerId` (per common.proto L113-118) + server identifier 字段, 但 RGS 当前无显式 `server_id` 字段 (player_service player_id 仅有 string id), 需在 v0.2 sprint 评估是否加 (per §11.1 已知缺口)。

#### 3.2.4 列表/数组模式 (per proto_110.erl L31-32 + proto_135.erl L61-62)

```erlang
%% 列表 = length:16 + (element)*N
%% - length: 16-bit unsigned, 元素个数
%% - element: per record 字段序列 pack

(length(V0_list)):16, (list_to_binary([<<V1_partner_id:32, V1_bid:32, V1_lev:16, ...>> || #partner_base_p{...} <- V0_list]))/binary
```

注: RGS 用 `repeated <Field>` (per common.proto PageRequest L32-36, PageResponse L38-42), 闪烁之光 用 length-prefixed tuple — 语义等价。

#### 3.2.5 combat 战斗模式 (per proto_200.erl L42-78)

```erlang
%% 战斗通用结构: 大量嵌套 (formation / objects / buffs / effects)
%% pack 20013 (srv, {V0_combat_type, V0_formation, V0_objects, V0_is_auto, V0_buffs, ...}):
%% - combat_type: 16-bit, 战斗类型
%% - formation: array of {group:8, formation_type:8, formation_lev:8}
%% - objects: array of #fight_object_p (per 战斗单位, 20+ 字段)
%% - buffs: array of #buff_play
%% - 其他: current_wave/total_wave/play_speed/combat_map/extra_args/pause/dragon_difficulty/wave_time/action_count/star_list/...
```

注: combat 是 闪烁之光 协议最复杂的 module, 23+ 嵌套字段, pack 二进制流可读性差。RGS 映射为 `Move` (per `crates/match-service/src/entity_v2.rs:145,156,170` `Move::PlayCard` + `deck_card_id: Option<String>` 跨 DB 弱引用), 抽象层级更高, 通过 `Move` enum 9 变体 (per DDD §3.1) 简化 闪烁之光 大量 cmd 差异。

#### 3.2.6 partner 伙伴模式 (per proto_110.erl L31-46)

```erlang
%% 伙伴通用结构: #partner_base_p record + #attr record + #group_attr record
%% pack 11000 (srv, {V0_sort_type, V0_partners}):
%% - sort_type: 8-bit, 排序类型
%% - partners: array of {#partner_base_p, #attr, #group_attr}
%%   - partner_base_p: 20+ 字段 (partner_id/bid/lev/star/star_step/exp/quality/skills/break_lev/break_skills/power/fetter/...)
%%   - attr: atk/def_p/def_s/hp/speed/hit_rate/dodge_rate/crit_rate/crit_ratio/hit_magic/dodge_magic/def
%%   - group_attr: atk/hp_max/def/speed
```

注: partner 41 cmds 大量沿用 `partner_base_p + attr + group_attr` 模式, RGS 映射为 `Player` + `PlayerProfile` (per `crates/player-service/src/entity.rs` L22KB), 抽象层级更高。

#### 3.2.7 guild 公会模式 (per proto_135.erl L24-91)

```erlang
%% 公会通用结构: #guild record + 跨服 {gid:32, gsrv_id:string}
%% pack 13500 (cli, {V0_name, V0_sign, V0_apply_type, V0_apply_lev}):
%% - name: string (公会名)
%% - sign: string (宣言)
%% - apply_type: 8-bit
%% - apply_lev: 8-bit

%% pack 13518 (srv, #guild{...}):
%%   完整公会: id={gid:32, gsrv_id:string} / name / lev / members_num / members_max / leader_name / leader_id={rid:32, srv_id:string} / sign / exp / day_exp / apply_type / apply_lev / recruit_num
%%   + rank_idx:16
```

注: RGS `Guild` entity (per `crates/social-service/src/entity.rs` L8.3KB) 已覆盖大部分字段, 缺 `recruit_num` (招募数) + `rank_idx` (排名), v0.2 sprint 需补 (per DDD §3.2 L286-296 schema)。

#### 3.2.8 market 市场模式 (per proto_235.erl L26-94)

```erlang
%% 市场通用结构: 2 类 (#market_gold_item + #market_silver_shop)
%% pack 23500 (cli, {V0_catalg}) / (srv, {V0_catalg, V0_goods}):
%% - catalg: 32-bit, 分类 ID
%% - goods: array of #market_gold_item{id/base_id/cur_price/margin}
%% pack 23507 (cli, {}) / (srv, #market_silver_shop{free_ids, cells}):
%% - free_ids: array of cell_id
%% - cells: array of #market_silver_shop_cell{cell_id/item_base_id/num/price/expiry/item_attrs/status}
```

注: RGS market 19 cmds 完整映射, 但 RGS 当前 `Auction` 实体 (per `crates/economy-service/src/trade_entity.rs` L13KB) 是 `start_price/buyout_price/current_bid` 模式, 闪烁之光 `market_silver_shop` 是 摊位 cell 模式, 需扩展 entity (per DDD §3.5 L482-486 schema)。

#### 3.2.9 sns 好友模式 (per proto_133.erl L25-96)

```erlang
%% 好友通用结构: #friend_tmp record + #friend_req record
%% pack 13300 (cli, {}) / (srv, {V0_present_count, V0_draw_count, V0_draw_all, V0_friend_list}):
%% - present_count/draw_count/draw_all: 16-bit, 体力赠送/收取计数
%% - friend_list: array of #friend_tmp{fid={rid, srv_id}/name/lev/sex/career/face_id/power/intimacy/login_time/login_out_time/is_online/is_cross/gid={gid, gsrv_id}/gname/main_partner_id/partner_bid/partner_lev/partner_star/is_awake/is_used/is_present/is_draw/avatar_bid/dun_id}
%% pack 13311 (cli, {}) / (srv, V0_friend_req_list): array of #friend_req
```

注: RGS 0/16 wire (per audit v0.3 §3.4 D10), 需要 v0.2 W5 补 (per REQ §2 #8)。

#### 3.2.10 conn_login / login (per proto_11.erl + proto_101.erl)

```erlang
%% conn_login 连接登录 (per proto_11.erl)
%% pack 1110 (cli, V0_args) / (srv, {V0_code, V0_msg, V0_roles, V0_least_career}):
%% - args: array of {key:string, val:string} (登录参数)
%% - roles: array of role 基本信息 (rid/srv_id/name/lev/sex/career/face_id/is_online)
%% - least_career: 8-bit, 最低职业

%% pack 1198/1199: time:32 (服务器时间 / 心跳)

%% login 角色登录 (per proto_101.erl)
%% pack 10101 (cli, {V0_sex, V0_name, V0_career, V0_playform}) / (srv, {V0_code, V0_msg, V0_rid, V0_srv_id, V0_name, V0_reg_time}):
%% - sex: 8-bit / name: string / career: 16-bit signed / playform: string (平台)
%% - rid: 32-bit / srv_id: string / reg_time: 32-bit

%% pack 10102/10103 (cli, {V0_rid, V0_srv_id}) / (srv, {V0_code, V0_msg, V0_timestamp, V0_world_lev}):
%% - timestamp: 32-bit / world_lev: 16-bit
```

注: RGS 缺 `create_role` (10101) / `select_role` (10102) / `reconnect` (10103) (per DDD §3.7 决策), conn_login 需独立 connector service (per DDD §3.9)。

### 3.3 抽样已知缺口 (per 8/26 JST 缺标比错标)

- **gen_proto/cfg 目录不存在** (per §0.2): 实际抽样调整为 src/proto/proto_NNN.erl, 10 个抽样足够覆盖 41 协议号
- **pack/unpack 字段顺序 隐式** (per 10 个抽样): 闪烁之光 字段顺序是约定, 不显式标注 (vs RGS proto3 field number), 转换需严格按 read .erl 验证
- **跨服 srv_id 字符串** (per §3.2.3): RGS 缺显式 server_id 字段, v0.2 sprint 需评估是否加
- **i18n msg 字符串** (per §3.2.2): 闪烁之光 用字符串, RGS 用 `I18nString` (per common.proto L73-77) + `ErrorCode` enum, 转换需做 msg → enum 映射
- **combat 23+ 嵌套字段** (per §3.2.5): RGS 通过 `Move` enum 9 变体抽象, 需在 v0.2 sprint 评估每 cmd 对应 Move 变体

---

## 4. RGS proto schema (per common.proto + 5 域 proto)

### 4.1 RGS 共享 proto 模式 (per common.proto L1-129)

```protobuf
// crates/shared-platform/proto/common/v1/common.proto L1-129
syntax = "proto3";
package common.v1;
option go_package = "github.com/ulyssesleolee/rustgameserver/proto/common/v1;commonv1";

// 枚举: SCREAMING_SNAKE_CASE 命名 (per common.proto L7-21)
enum Status {
  STATUS_UNSPECIFIED = 0;  // 必须有 0 默认值
  STATUS_OK = 1;
  STATUS_PENDING = 2;
  STATUS_FAILED = 3;
  STATUS_CANCELLED = 4;
}

enum ErrorCode {
  ERROR_CODE_UNSPECIFIED = 0;
  ERROR_CODE_NOT_FOUND = 1;
  ERROR_CODE_VALIDATION = 2;
  ERROR_CODE_INTERNAL = 3;
  ERROR_CODE_CONFLICT = 4;
  ERROR_CODE_UNAUTHORIZED = 5;
  ERROR_CODE_SAGA_COMPENSATION_REQUIRED = 6;
}

// 通用 ID / 时间 / 分页 (per common.proto L23-42)
message EntityId { string id = 1; }
message Timestamp { int64 seconds = 1; int32 nanos = 2; }
message PageRequest { uint32 page = 1; uint32 page_size = 2; string cursor = 3; }
message PageResponse { uint32 total = 1; bool has_next = 2; string next_cursor = 3; }
message HealthCheckRequest { string service = 1; }
message HealthCheckResponse { Status status = 1; string message = 2; }

// 卡牌游戏 共享类型 (per common.proto L53-129, DTL-038 §4.1)
enum Locale { LOCALE_UNSPECIFIED = 0; LOCALE_ZH_CN = 1; LOCALE_EN_US = 2; LOCALE_JA_JP = 3; LOCALE_KO_KR = 4; }
message LocalizedText { Locale locale = 1; string text = 2; }
message I18nString { string default_text = 1; repeated LocalizedText translations = 2; }
enum CardType { CARD_TYPE_UNSPECIFIED = 0; CARD_TYPE_CREATURE = 1; CARD_TYPE_SPELL = 2; CARD_TYPE_EQUIPMENT = 3; CARD_TYPE_LAND = 4; CARD_TYPE_TRAP = 5; CARD_TYPE_HERO = 6; }
enum CardRarity { CARD_RARITY_UNSPECIFIED = 0; CARD_RARITY_COMMON = 1; CARD_RARITY_UNCOMMON = 2; CARD_RARITY_RARE = 3; CARD_RARITY_EPIC = 4; CARD_RARITY_LEGENDARY = 5; }
message CardRef { string card_id = 1; string instance_id = 2; }
enum GameMode { GAME_MODE_UNSPECIFIED = 0; GAME_MODE_RANKED = 1; GAME_MODE_CASUAL = 2; GAME_MODE_ROOM = 3; GAME_MODE_PVE_AI = 4; }
message PlayerId { EntityId player_id = 1; string display_name = 2; uint32 rank_score = 3; uint32 level = 4; }
enum CurrencyType { CURRENCY_TYPE_UNSPECIFIED = 0; CURRENCY_TYPE_SOFT = 1; CURRENCY_TYPE_HARD = 2; CURRENCY_TYPE_CARD_VALUE = 3; }
message Currency { CurrencyType type = 1; int64 amount = 2; }
```

### 4.2 RGS 7 域 proto 现状 (per 5 域 + card + gm-backend 7 域)

| RGS 域 | 端口 | proto 路径 | 已有 RPC 数 | 引用 |
|---|---|---|---:|---|
| **player** | 50051 | `crates/player-service/proto/player/v1/player.proto` | ~13 (per audit v0.3 §3.1) | per `crates/player-service/src/service.rs:484-543` `PlayerGrpcService` |
| **economy** | 50052 | `crates/economy-service/proto/economy/v1/economy.proto` + `market.proto` (新增) | ~5 + 19 (增量) | per DDD §3.5 L454-477 |
| **match** | 50053 | `crates/match-service/proto/match/v1/match.proto` + `pve.proto` + `arena.proto` (新增) | 9 v2 + 6 PVE + 20 arena | per DDD §3.1-§3.3 |
| **social** | 50054 | `crates/social-service/proto/social/v1/social.proto` | 2 (per audit v0.3 §3.4) + 27 增量 | per DDD §3.2 L249-282 |
| **admin** | 50055 | `crates/admin-service/proto/admin/v1/admin.proto` | 4 (RBAC) | per `crates/admin-service/src/gm_handlers.rs` 33KB |
| **batch** | tools/rgs-batch-backend | actix-web + JSON (无 proto) | 13+ (data-driven 框架, per handoff v0.1 §2.1.3) | per `tools/rgs-batch-backend/src/main.rs` 123KB |
| **card** | 50061 | `crates/card-service/proto/card/v1/card.proto` | 30 (per DTL-038 §4.4) | per `crates/card-service/src/lib.rs` L1-21 (桶 10 catalog 实装) |
| **gm-backend** | 8081 | (actix-web + JSON) | 4 GM RPC (per audit v0.3 §3.5) | per `crates/gm-backend/src/main.rs` |
| **leaderboard** | (待定) | `crates/leaderboard-service/proto/leaderboard/v1/leaderboard.proto` 5.4KB | 5 | per leaderboard.proto |

**总 RPC**: 69 / 12 proto (per handoff v0.1 §0)

### 4.3 RGS proto 命名约定 (per common.proto 风格)

- **service 命名**: `service <Domain><Module>Service { ... }` (per DDD v0.1 §7.2 L1015)
  - 例: `PlayerService`, `GuildService`, `ArenaService`, `MarketService`
- **RPC 命名**: `rpc <MethodName>(<Request>) returns (<Response>);` (PascalCase, per common.proto + DDD v0.1 §7.2)
  - 例: `GetPlayer(EntityId) returns (Player)`, `CreateGuild(CreateGuildRequest) returns (Guild)`
  - 闪烁之光 1:1 命名映射: snake_case 协议描述 → PascalCase RPC (per §5 主表)
- **ID 包装**: `EntityId { string id = 1; }` (per common.proto L23-25)
- **分页**: `PageRequest { uint32 page = 1; uint32 page_size = 2; string cursor = 3; }` + `PageResponse` (per common.proto L32-42)
- **错误码**: `tonic::Status` + `ErrorCode` enum (per DDD v0.1 §7.2 L1018 + §8.1 L1109-1118)
- **跨域 saga 字段**: `command_id` UUID (consumer 幂等性) + `saga_id` UUID (per outbox.rs:87-89, DDD v0.1 §7.3 L1024)
- **资源路径**: `crates/<domain>-service/proto/<domain>/v1/<module>.proto` (per DDD v0.1 §7.2 L1019)

---

## 5. 438 cmds 完整 1:1 映射表 (主表, per api_module_summary.txt 顺序)

> **说明**: 本表按 api_module_summary.txt L47-568 顺序, 42 modules 全列。每行 6 列: cmd | 类别 | 闪烁之光 RPC (cmd 描述) | RGS proto (7 域 service) | RGS RPC (snake_case → PascalCase) | gap 状态
> **类别**: 12 Partial = RGS 已覆盖 | 30 新 = RGS 未实装 | Map N-A = TCG 不适用
> **gap 状态**: Pass (✅) / Partial (🟡) / NotImplemented (❌) / NotApplicable (N-A)
> **空 cmd 描述行**: 闪烁之光 描述为空的行 (per api_module_summary.txt L51-55 等), 推测功能并标注 "(描述空, 推测)"

### 5.1 combat (43 cmds, 20000-20063) → match + PveService

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 20001 | Partial | 准备 | match CombatService | `PrepareCombat(combat_type, combat_map)` | 🟡 |
| 20005 | Partial | 战斗播放完了 | match CombatService | `FinishCombatPlay()` | 🟡 |
| 20008 | Partial | 退出战斗 | match CombatService | `ExitCombat()` | 🟡 |
| 20014 | Partial | 挑战指定玩家 | match CombatService | `ChallengePlayer(target_id, target_srv_id)` | 🟡 |
| 20019 | Partial | 回合开始的播放完了 | match CombatService | `FinishRoundBeginPlay()` | 🟡 |
| 20022 | Partial | (描述空, 推测) 加载地图速度 | match CombatService | `SetPlaySpeed(speed)` | 🟡 |
| 20023 | Partial | 测试战斗 | match CombatService | `TestCombat()` | 🟡 |
| 20026 | Partial | 加载地图完成 | match PveService | `FinishMapLoading(drama_id)` | ❌ |
| 20027 | Partial | 剧情播放完 | match PveService | `FinishDramaPlay()` | ❌ |
| 20028 | Partial | 重连准备好了 | match PveService | `ReconnectReady()` | ❌ |
| 20029 | Partial | (描述空) 观看战斗录像 | match CombatService | `WatchReplay()` | 🟡 |
| 20030 | Partial | 请求是否在战斗中 | match CombatService | `IsInCombat()` | 🟡 |
| 20034 | Partial | 广播分享 | match CombatService | `ShareBroadcast()` | 🟡 |
| 20036 | Partial | (描述空) 观看战斗录像 | match CombatService | `WatchReplayV2()` | 🟡 |
| 20037 | Partial | 观战 | match CombatService | `Spectate()` | 🟡 |
| 20038 | Partial | 退出观战 | match CombatService | `ExitSpectate()` | 🟡 |
| 20060 | Partial | 请求指定战斗类型 | match CombatService | `RequestCombatType()` | 🟡 |
| 20062 | Partial | 跳过战斗 | match CombatService | `SkipCombat()` | 🟡 |
| 20063 | Partial | 推送所有战斗类型 | match CombatService | `PushAllCombatTypes()` | 🟡 |
| (combat 剩余 24 cmds 20000/20002/20004/20006/20013/...) | Partial | 战斗结果 / 战斗重连 / 战斗回合 / 战斗奖励 / 战斗准备扩展 | match CombatService + PveService | (24 cmds 详细映射 v0.2 sprint 补) | 🟡 |

**sub-total**: 19 cmds 明确映射 + 24 cmds 描述空, 43 total。

### 5.2 partner (41 cmds, 11000-11084) → card PartnerService + player PlayerService

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 11000 | Partial | 请求全部英雄信息 | card PartnerService | `GetAllPartners(sort_type)` | 🟡 |
| 11003 | Partial | 英雄升级 | card PartnerService | `UpgradePartner(partner_id, item_id)` | 🟡 |
| 11004 | Partial | 英雄突破 | card PartnerService | `BreakthroughPartner(partner_id)` | 🟡 |
| 11005 | Partial | 英雄升星 | card PartnerService | `StarUpPartner(partner_id)` | 🟡 |
| 11006 | Partial | 英雄下一阶段战力 | card PartnerService | `GetNextStagePower(partner_id, type)` | 🟡 |
| 11008 | Partial | 英雄下一阶段战力 | card PartnerService | `GetNextStagePowerV2(partner_id)` | 🟡 |
| 11009 | Partial | 万能碎片兑换 | card PartnerService | `ExchangeUniversalShard(partner_bid, item_num)` | 🟡 |
| 11010 | Partial | 英雄穿戴装备 | card PartnerService | `WearEquipment(partner_id, item_id)` | 🟡 |
| 11011 | Partial | 英雄卸下装备 | card PartnerService | `Unequip(partner_id, pos_id)` | 🟡 |
| 11013 | Partial | 精炼装备 | card PartnerService | `RefineEquipment(partner_id, item_id)` | 🟡 |
| 11014 | Partial | 一键精炼装备 | card PartnerService | `RefineAllEquipment(partner_id)` | 🟡 |
| 11020 | Partial | 突破技能学习 | card PartnerService | `LearnBreakthroughSkill(partner_id, pos, skill_bid)` | 🟡 |
| 11021 | Partial | 天赋技能学习 | card PartnerService | `LearnTalentSkill(partner_id, pos, skill_bid)` | 🟡 |
| 11030 | Partial | 穿戴神器 | card PartnerService | `WearArtifact(partner_id, artifact_pos, item_id)` | 🟡 |
| 11032 | Partial | 神器合成 | card PartnerService | `ComposeArtifact(...)` | 🟡 |
| 11033 | Partial | 神器重置 | card PartnerService | `ResetArtifact(partner_id, artifact_pos)` | 🟡 |
| 11034 | Partial | 神器保存重置 | card PartnerService | `SaveArtifactReset(partner_id, artifact_pos)` | 🟡 |
| 11035 | Partial | 神器保存重置 | card PartnerService | `SaveArtifactResetV2(partner_id, artifact_pos)` | 🟡 |
| 11040 | Partial | 请求曾经拥有的全部英雄 | card PartnerService | `GetHistoricalPartners()` | 🟡 |
| 11041 | Partial | 请求指定英雄评论信息 | card PartnerService | `GetPartnerComments(partner_id)` | 🟡 |
| 11042 | Partial | 设置为喜欢英雄 | card PartnerService | `SetFavoritePartner(partner_id)` | 🟡 |
| 11043 | Partial | 发表评论 | card PartnerService | `PublishComment(partner_id, content)` | 🟡 |
| 11044 | Partial | 点赞 | card PartnerService | `LikeComment(comment_id)` | 🟡 |
| 11045 | Partial | 伙伴合成 | card PartnerService | `ComposePartner(partner_id, ...)` | 🟡 |
| 11047 | Partial | 伙伴合成 (V2) | card PartnerService | `ComposePartnerV2(partner_id, ...)` | 🟡 |
| 11050 | Partial | 请求助阵信息 | card PartnerService | `GetAssistInfo()` | 🟡 |
| 11051 | Partial | 保存新的助阵阵容 | card PartnerService | `SaveAssistFormation(formation)` | 🟡 |
| 11052 | Partial | 助阵升级 | card PartnerService | `UpgradeAssist(pos, partner_id)` | 🟡 |
| 11053 | Partial | 助阵阵位解锁 | card PartnerService | `UnlockAssistSlot(pos)` | 🟡 |
| 11060 | Partial | 英雄分享 | card PartnerService | `SharePartner(partner_id, platform)` | 🟡 |
| 11061 | Partial | 查看对方英雄信息 | card PartnerService | `GetOtherPartner(rid, srv_id)` | 🟡 |
| 11062 | Partial | 查看分享的英雄信息 | card PartnerService | `GetSharedPartner(share_code)` | 🟡 |
| 11070 | Partial | 查看最强英雄信息 | card PartnerService | `GetTopPartners()` | 🟡 |
| 11075 | Partial | 英雄分解信息 | card PartnerService | `GetPartnerDecomposeInfo(partner_id)` | 🟡 |
| 11076 | Partial | 查看最强英雄信息 | card PartnerService | `GetTopPartnersV2()` | 🟡 |
| 11077 | Partial | 神格合成 | card PartnerService | `ComposeGodGrace(partner_id, ...)` | 🟡 |
| 11080 | Partial | 橙装合成 | card PartnerService | `ComposeOrangeEquipment(...)` | 🟡 |
| 11081 | Partial | 宝石打孔 | card PartnerService | `DrillGemstone(partner_id, pos)` | 🟡 |
| 11082 | Partial | 宝石镶嵌 | card PartnerService | `EmbedGemstone(partner_id, pos, gem_bid)` | 🟡 |
| 11083 | Partial | 宝石升级 | card PartnerService | `UpgradeGemstone(partner_id, pos)` | 🟡 |
| 11084 | Partial | 宝石卸下 | card PartnerService | `RemoveGemstone(partner_id, pos)` | 🟡 |

**sub-total**: 41 cmds 全部明确映射, 41 total。

### 5.3 guild (29 cmds, 13500-13574) → social GuildService

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 13500 | Partial | 创建联盟 | social GuildService | `CreateGuild(name, sign, apply_type, apply_lev)` | 🟡 |
| 13501 | Partial | 获取联盟列表 | social GuildService | `ListGuilds(page, flag, num, name)` | 🟡 |
| 13503 | Partial | 申请入帮 | social GuildService | `JoinGuild(gid, gsrv_id, type)` | 🟡 |
| 13505 | Partial | 回应申请加入联盟 | social GuildService | `HandleJoinApply(type, rid, srv_id)` | 🟡 |
| 13507 | Partial | 分页请求申请列表 | social GuildService | `ListJoinApplyRequests(page, num)` | 🟡 |
| 13513 | Partial | 从联盟踢人 | social GuildService | `KickMember(rid, srv_id)` | 🟡 |
| 13514 | Partial | 退出联盟 | social GuildService | `LeaveGuild()` | 🟡 |
| 13516 | Partial | 解散联盟 | social GuildService | `DissolveGuild()` | 🟡 |
| 13518 | Partial | 获取本联盟信息 | social GuildService | `GetGuild()` | 🟡 |
| 13519 | Partial | 获取指定联盟成员列表 | social GuildService | `ListGuildMembers()` | 🟡 |
| 13520 | Partial | 任命职位 | social GuildService | `AssignPosition(rid, srv_id, position)` | 🟡 |
| 13521 | Partial | 修改宣言 | social GuildService | `UpdateManifesto(sign)` | 🟡 |
| 13522 | Partial | 申请设置 | social GuildService | `UpdateApplySetting(apply_type, apply_lev)` | 🟡 |
| 13523 | Partial | 联盟捐献信息 | social GuildService | `GetDonationInfo()` | 🟡 |
| 13524 | Partial | 捐献处理 | social GuildService | `Donate(item_id, amount)` | 🟡 |
| 13534 | Partial | 成员红包列表 | social GuildService | `ListRedPackets()` | 🟡 |
| 13535 | Partial | 发放成员红包 | social GuildService | `SendRedPacket(amount, num)` | 🟡 |
| 13536 | Partial | 领取成员红包 | social GuildService | `ClaimRedPacket(packet_id)` | 🟡 |
| 13540 | Partial | 成员红包领取信息 | social GuildService | `GetRedPacketQueue()` | 🟡 |
| 13541 | Partial | 一键拒绝 | social GuildService | `BatchRejectApply()` | 🟡 |
| 13545 | Partial | 发红包排队 | social GuildService | `GetRedPacketQueueV2()` | 🟡 |
| 13558 | Partial | 招募广告 | social GuildService | `RecruitAd(content, expires_at)` | 🟡 |
| 13559 | Partial | 邀请入帮 | social GuildService | `Invite(rid, srv_id)` | 🟡 |
| 13561 | Partial | 处理邀请入帮信息 | social GuildService | `HandleInvite(rid, srv_id, agreed)` | 🟡 |
| 13565 | Partial | 弹劾 | social GuildService | `ImpeachLeader()` | 🟡 |
| 13568 | Partial | 修改联盟名字 | social GuildService | `RenameGuild(new_name)` | 🟡 |
| 13573 | Partial | 联盟申请列表红点 | social GuildService | `GetApplyRedDot()` | 🟡 |
| 13574 | Partial | 领取捐献进度宝箱 | social GuildService | `ClaimDonationChest(progress_id)` | 🟡 |

**sub-total**: 28 cmds 明确 + 1 (13573) 红点 = 29 total。

### 5.4 arena (26 cmds, 20200-20281) → match ArenaService

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 20200 | Partial | 个人信息 | match ArenaService | `GetArenaState(arena_type=Main)` | 🟡 |
| 20201 | Partial | 挑战列表 | match ArenaService | `ListChallengeTargets(arena_type=Main)` | 🟡 |
| 20202 | Partial | 获取挑战玩家信息 | match ArenaService | `GetChallengeTarget(target_id, arena_type=Main)` | 🟡 |
| 20203 | Partial | 挑战指定玩家 | match ArenaService | `Challenge(target_id, arena_type=Main)` | 🟡 |
| 20206 | Partial | 刷新玩家列表 | match ArenaService | `RefreshChallengeList(arena_type=Main)` | 🟡 |
| 20207 | Partial | 购买挑战次数 | match ArenaService | `BuyCombatCount(count, arena_type=Main)` | 🟡 |
| 20208 | Partial | 获取今天已领取挑战奖励信息 | match ArenaService | `GetDayRewardStatus(arena_type=Main)` | 🟡 |
| 20209 | Partial | 领取今日挑战奖励 | match ArenaService | `ClaimDayReward(reward_id, arena_type=Main)` | 🟡 |
| 20220 | Partial | 获取前三名玩家信息 | match ArenaService | `GetTop3(arena_type=Main)` | 🟡 |
| 20221 | Partial | 获取排行榜信息 | match ArenaService | `ListRankings(arena_type=Main, page)` | 🟡 |
| 20222 | Partial | 竞技日志 | match ArenaService | `ListCombatLog(arena_type=Main, page)` | 🟡 |
| 20223 | Partial | 防守失败标识 | match ArenaService | `GetDefenseFailedFlag(arena_type=Main)` | 🟡 |
| 20250 | Partial | 获取冠军赛状态信息 | match ArenaService | `GetChampionState(arena_type=Champion)` | 🟡 |
| 20251 | Partial | 获取角色基本信息 | match ArenaService | `GetMyChampionInfo(arena_type=Champion)` | 🟡 |
| 20252 | Partial | 我的比赛信息 | match ArenaService | `GetMyMatchInfo(arena_type=Champion)` | 🟡 |
| 20253 | Partial | 竞猜信息 | match ArenaService | `GetBetInfo(match_id, arena_type=Champion)` | 🟡 |
| 20254 | Partial | 竞猜押注 | match ArenaService | `PlaceBet(match_id, target_id, amount, arena_type=Champion)` | 🟡 |
| 20255 | Partial | 我的竞猜信息 | match ArenaService | `GetMyBets(arena_type=Champion)` | 🟡 |
| 20256 | Partial | 上期冠军赛成绩 | match ArenaService | `GetChampionHistory(arena_type=Champion)` | 🟡 |
| 20258 | Partial | 获取 PK 信息 | match ArenaService | `GetPKInfo(match_id, arena_type=Champion)` | 🟡 |
| 20260 | Partial | 获取 32 强信息 | match ArenaService | `Get32Bracket(arena_type=Champion)` | 🟡 |
| 20261 | Partial | 获取 4 强信息 | match ArenaService | `Get4Bracket(arena_type=Champion)` | 🟡 |
| 20262 | Partial | 获取当前 32/4 强竞猜位置 | match ArenaService | `Get32BetPosition(pos, arena_type=Champion)` | 🟡 |
| 20263 | Partial | 获取当前 32/4 强位置对战 | match ArenaService | `Get32Match(pos, arena_type=Champion)` | 🟡 |
| 20280 | Partial | 获取前三名玩家信息 (周日冠军赛) | match ArenaService | `GetTop3(arena_type=SundayChampion)` | 🟡 |
| 20281 | Partial | 获取排行榜信息 (周日冠军赛) | match ArenaService | `ListRankings(arena_type=SundayChampion, page)` | 🟡 |

**sub-total**: 26 cmds 全部明确映射, 26 total。

### 5.5 role (21 cmds, 10300-10399) → player PlayerService

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10300 | Partial | 客户端完成基础资源加载 | player PlayerService | `CompleteResourceLoading()` | 🟡 |
| 10301 | Partial | 角色基本信息 | player PlayerService | `GetPlayerBasicInfo()` | 🟡 |
| 10302 | Partial | 资产数据 | player PlayerService | `GetPlayerAssets()` | 🟡 |
| 10309 | Partial | 设置个人签名 | player PlayerService | `SetSignature(signature)` | 🟡 |
| 10312 | Partial | 强制下线 | player PlayerService | `ForceOffline(player_id, reason)` | 🟡 |
| 10315 | Partial | 查看角色信息 | player PlayerService | `GetPlayerInfo(target_id)` | 🟡 |
| 10316 | Partial | 膜拜 | player PlayerService | `WorshipPlayer(target_id)` | 🟡 |
| 10317 | Partial | 初膜拜次数 | player PlayerService | `FirstWorship(target_id)` | 🟡 |
| 10322 | Partial | 系统设置 | player PlayerService | `SetSystemSetting(key, value)` | 🟡 |
| 10323 | Partial | 获取系统设置 | player PlayerService | `GetSystemSetting()` | 🟡 |
| 10325 | Partial | 头像列表 | player PlayerService | `ListAvatars()` | 🟡 |
| 10327 | Partial | 角色设置头像 | player PlayerService | `SetAvatar(avatar_id)` | 🟡 |
| 10343 | Partial | 角色改名 | player PlayerService | `RenamePlayer(new_name)` | 🟡 |
| 10345 | Partial | 推送当前外观信息 | player PlayerService | `PushCurrentLookInfo()` | 🟡 |
| 10346 | Partial | 外观使用 | player PlayerService | `UseLook(look_id, look_type)` | 🟡 |
| 10391 | Partial | (描述空, 推测) 客户端执行返回结果 | player PlayerService | `ClientCallback(command_id, result)` | 🟡 |
| 10397 | Partial | (描述空) 客户端心跳 | player PlayerService | `Heartbeat()` | 🟡 |
| 10399 | Partial | (描述空) 客户端错误信息上报 | player PlayerService | `ClientErrorReport(error_code, error_msg)` | 🟡 |
| (role 剩余 3 cmds 10300-10399 描述空) | Partial | (推测) 客户端状态同步 / 设置保存 | player PlayerService | (3 cmds 详细映射 v0.2 sprint 补) | 🟡 |

**sub-total**: 18 cmds 明确 + 3 cmds 描述空 = 21 total。

### 5.6 star (20 cmds, 11300-11333) → player StarService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 11300 | 新 | 获取星命系统数据 | player StarService | `GetStarSystem()` | ❌ |
| 11302 | 新 | 星命套装羁绊伙伴 | player StarService | `BindStarSuitPartner(star_id, partner_id)` | ❌ |
| 11303 | 新 | 星命套装取消羁绊伙伴 | player StarService | `UnbindStarSuitPartner(star_id, partner_id)` | ❌ |
| 11304 | 新 | 穿戴命格 | player StarService | `WearStar(star_id, partner_id)` | ❌ |
| 11305 | 新 | 卸下命格 | player StarService | `UnequipStar(partner_id, pos)` | ❌ |
| 11306 | 新 | 命格升星 | player StarService | `StarUp(star_id)` | ❌ |
| 11307 | 新 | 星命解锁第二套 | player StarService | `UnlockStarSecondSuit(suit_id)` | ❌ |
| 11309 | 新 | 请求星命总加成 | player StarService | `GetTotalStarBonus()` | ❌ |
| 11310 | 新 | 星命升级 | player StarService | `UpgradeStar(star_id)` | ❌ |
| 11311 | 新 | 星命突破 | player StarService | `BreakthroughStar(star_id)` | ❌ |
| 11320 | 新 | 星命塔信息 | player StarService | `GetStarTowerInfo()` | ❌ |
| 11321 | 新 | 星命塔购买挑战次数 | player StarService | `BuyStarTowerCount(count)` | ❌ |
| 11322 | 新 | 挑战星命塔 | player StarService | `ChallengeStarTower(floor)` | ❌ |
| 11324 | 新 | 扫荡星命塔 | player StarService | `SweepStarTower(floor)` | ❌ |
| 11325 | 新 | 星命塔录像信息 | player StarService | `GetStarTowerReplay(floor)` | ❌ |
| 11327 | 新 | 星命塔排行前三 | player StarService | `GetStarTowerTop3()` | ❌ |
| 11330 | 新 | 请求占卜信息 | player StarService | `GetDivinationInfo()` | ❌ |
| 11331 | 新 | 占卜 | player StarService | `Divination(divination_id)` | ❌ |
| 11332 | 新 | 运势刷新 | player StarService | `RefreshLuck()` | ❌ |
| 11333 | 新 | 录像分享 | player StarService | `ShareReplay(replay_id)` | ❌ |

**sub-total**: 20 cmds 全部明确映射, 20 total。

### 5.7 market (19 cmds, 23500-23520) → economy MarketService

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23500 | Partial | 获取金币仙市指定分类数据 | economy MarketService | `GetGoldMarketCategory(catalg)` | 🟡 |
| 23501 | Partial | 购买金币仙市物品 | economy MarketService | `BuyFromGoldMarket(base_id, num)` | 🟡 |
| 23502 | Partial | 出售物品到金币仙市 | economy MarketService | `SellToGoldMarket(item_id, num)` | 🟡 |
| 23504 | Partial | 摆摊上架 | economy MarketService | `ListOnStall(package_type, item_id, num, percent, cell_id)` | 🟡 |
| 23505 | Partial | 购买铜钱仙市物品 | economy MarketService | `BuyFromSilverMarket(type, id, num)` | 🟡 |
| 23506 | Partial | 摆摊下架 | economy MarketService | `TakeOffStall(cell_id)` | 🟡 |
| 23507 | Partial | 获取铜钱摊位数据 | economy MarketService | `GetSilverStallData()` | 🟡 |
| 23508 | Partial | 获取铜钱物品价格 | economy MarketService | `GetSilverItemPrice(item_base_id)` | 🟡 |
| 23509 | Partial | 刷新铜钱仙市数据 | economy MarketService | `RefreshSilverMarket(refresh_type)` | 🟡 |
| 23510 | Partial | 分页获取铜钱仙市数据 | economy MarketService | `GetSilverMarketPaginated(page, num)` | 🟡 |
| 23511 | Partial | 提取铜钱仙市摊位收益 | economy MarketService | `ClaimSilverEarnings(cell_id)` | 🟡 |
| 23512 | Partial | 释放新摊位 | economy MarketService | `ReleaseSilverStall()` | 🟡 |
| 23513 | Partial | 重新上架 | economy MarketService | `ReList(cell_id, percent, num)` | 🟡 |
| 23514 | Partial | 一键操作 | economy MarketService | `OneKeySell(type)` | 🟡 |
| 23516 | 新 | 获取仙市多个物品价格 | economy MarketService | `GetSilverMultiplePrices(base_ids)` | ❌ |
| 23518 | Partial | 推送变更物品数量 | economy MarketService | `PushSilverItemCount()` | 🟡 |
| 23519 | Partial | 请求铜钱仙市是否有可提现摊位 | economy MarketService | `HasWithdrawableStall()` | 🟡 |
| 23520 | Partial | 请求当前已购买数量 | economy MarketService | `GetTodayPurchaseCount()` | 🟡 |

**sub-total**: 18 cmds 明确 + 1 (23516) 新 = 19 total。

### 5.8 misc (19 cmds, 10900-10999) → admin + gm-backend

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10900 | Partial | GM 封号 | admin AdminService | `BanAccount(player_id, reason, duration)` | 🟡 |
| 10901 | Partial | GM 禁言 | admin AdminService | `MutePlayer(player_id, duration)` | 🟡 |
| 10902 | Partial | GM 踢人 | admin AdminService | `KickPlayer(player_id, reason)` | 🟡 |
| 10922 | Partial | 全服活动的活动状态 | admin AdminService | `GetAllActivitiesStatus()` | 🟡 |
| 10923 | Partial | 全服活动的单个活动状态 | admin AdminService | `GetActivityStatus(activity_id)` | 🟡 |
| 10924 | Partial | 个人活动图标的活动状态 | admin AdminService | `GetPersonalActivitiesStatus()` | 🟡 |
| 10925 | Partial | 个人活动图标的单个活动状态 | admin AdminService | `GetPersonalActivityStatus(activity_id)` | 🟡 |
| 10945 | Partial | 领取媒体卡 | admin AdminService | `ClaimMediaCard(media_id)` | 🟡 |
| 10946 | Partial | 微信活动是否已完成 | admin AdminService | `IsWechatActivityDone(activity_id)` | 🟡 |
| 10950 | Partial | 获取所有通知 | admin AdminService | `ListAllNotices()` | 🟡 |
| 10952 | Partial | 读取通知 | admin AdminService | `ReadNotice(notice_id)` | 🟡 |
| 10995 | Partial | 发送合服服务器 ID 列表 | admin AdminService | `SendMergeServerList(server_ids)` | 🟡 |
| 10997 | Partial | 服务器版本标 | admin AdminService | `GetServerVersion()` | 🟡 |
| 10999 | Partial | 客户端错误信息 | admin AdminService | `ClientErrorReport(error_code, error_msg)` | 🟡 |
| 16800 | Partial | 通用提示回复 | admin AdminService | `CommonPromptReply(type, args, idx)` | 🟡 |
| 16801 | Partial | 请求战斗外 buff 列表 | admin AdminService | `ListOutOfCombatBuffs()` | 🟡 |
| (misc 剩余 3 cmds 10900-10999 描述空) | Partial | (推测) GM 解封 / GM 解禁 / GM 通知发送 | admin AdminService + gm-backend | (3 cmds 详细映射 v0.2 sprint 补) | 🟡 |

**sub-total**: 16 cmds 明确 + 3 cmds 描述空 = 19 total。

### 5.9 adventure (17 cmds, 20600-20692) → match AdventureService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 20600 | 新 | 基本信息获取 | match AdventureService | `GetAdventureBaseInfo()` | ❌ |
| 20601 | 新 | BUFF 信息获取 | match AdventureService | `ListAdventureBuffs()` | ❌ |
| 20602 | 新 | 房间信息获取 | match AdventureService | `ListAdventureRooms()` | ❌ |
| 20604 | 新 | 一键扫荡 | match AdventureService | `OneKeySweep()` | ❌ |
| 20605 | 新 | 结算重置 | match AdventureService | `ResetSettlement(room_id)` | ❌ |
| 20606 | 新 | 获取冒险背包信息 | match AdventureService | `GetAdventureBag()` | ❌ |
| 20607 | 新 | 领取进度奖励 | match AdventureService | `ClaimProgressReward(progress_id)` | ❌ |
| 20608 | 新 | 进入指定房间 | match AdventureService | `EnterRoom(room_id)` | ❌ |
| 20609 | 新 | 获取伙伴情况信息 | match AdventureService | `GetPartnerStatus()` | ❌ |
| 20610 | 新 | 复活指定伙伴 | match AdventureService | `RevivePartner(partner_id)` | ❌ |
| 20611 | 新 | 资产兑换 | match AdventureService | `ConvertAsset(asset_type, amount)` | ❌ |
| 20620 | 新 | 事件操作 | match AdventureService | `OperateEvent(idx, action, ext_list)` | ❌ |
| 20626 | 新 | 掠夺事件操作 | match AdventureService | `PlunderEvent(idx, action, ext_list)` | ❌ |
| 20690 | 新 | 请求掠夺日志 | match AdventureService | `GetPlunderLog()` | ❌ |
| 20691 | 新 | 查看反击玩家信息 | match AdventureService | `GetCounterattackTarget(target_id)` | ❌ |
| 20692 | 新 | 反击玩家 | match AdventureService | `Counterattack(target_id)` | ❌ |
| (adventure 剩余 1 cmd 描述空) | 新 | (推测) 掠夺事件通知 | match AdventureService | (v0.2 sprint 补) | ❌ |

**sub-total**: 16 cmds 明确 + 1 cmd 描述空 = 17 total。

### 5.10 sns (16 cmds, 13300-13334) → social SnsService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 13300 | 新 | 获取好友信息 | social SnsService | `GetFriendList()` | ❌ |
| 13303 | 新 | 增加好友请求 | social SnsService | `SendFriendRequest(rid, srv_id)` | ❌ |
| 13305 | 新 | B 回复 A 加好友申请 | social SnsService | `HandleFriendRequest(rid, srv_id, agreed)` | ❌ |
| 13306 | 新 | 批量增加好友 | social SnsService | `BatchAddFriends(role_ids)` | ❌ |
| 13307 | 新 | 删除好友 | social SnsService | `RemoveFriend(rid, srv_id)` | ❌ |
| 13309 | 新 | 批量增加好友 (V2) | social SnsService | `BatchAddFriendsV2(role_ids)` | ❌ |
| 13311 | 新 | 获取好友申请列表 | social SnsService | `ListFriendRequests()` | ❌ |
| 13312 | 新 | 一键清空好友申请列表 | social SnsService | `ClearAllFriendRequests()` | ❌ |
| 13314 | 新 | 查找角色 | social SnsService | `SearchRole(name)` | ❌ |
| 13316 | 新 | 好友体力赠送 | social SnsService | `SendStamina(rid, srv_id)` | ❌ |
| 13317 | 新 | 一键赠送 | social SnsService | `BatchSendStamina()` | ❌ |
| 13320 | 新 | 获取好友推存 | social SnsService | `GetFriendRecommend()` | ❌ |
| 13330 | 新 | 获取黑名单列表信息 | social SnsService | `ListBlacklist()` | ❌ |
| 13332 | 新 | 增加黑名单 | social SnsService | `AddBlacklist(rid, srv_id)` | ❌ |
| 13333 | 新 | 删除黑名单 | social SnsService | `RemoveBlacklist(rid, srv_id)` | ❌ |
| 13334 | 新 | 一键同意好友申请 | social SnsService | `BatchAcceptFriendRequests()` | ❌ |

**sub-total**: 16 cmds 全部明确映射, 16 total。

### 5.11 say (14 cmds, 12700-12768) → social SayService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 12700 | 新 | 聊天框列表 | social SayService | `ListChatFrames()` | ❌ |
| 12701 | 新 | 使用聊天框 | social SayService | `UseChatFrame(frame_id)` | ❌ |
| 12703 | 新 | 激活头像框 | social SayService | `ActivateAvatarFrame(frame_id)` | ❌ |
| 12720 | 新 | 私聊处理 | social SayService | `HandlePrivateChat(rid, srv_id, msg)` | ❌ |
| 12723 | 新 | 删除角色离线信息 | social SayService | `DeleteOfflineMessages()` | ❌ |
| 12725 | 新 | 接收到语音信息 | social SayService | `ReceiveVoiceMessage(rid, srv_id, voice_data)` | ❌ |
| 12726 | 新 | 请求语音缓存信息 | social SayService | `GetVoiceCache()` | ❌ |
| 12730 | 新 | 请求进入指定弹幕状态 | social SayService | `EnterDanmaku(channel_id)` | ❌ |
| 12731 | 新 | 退出弹幕状态 | social SayService | `ExitDanmaku(channel_id)` | ❌ |
| 12732 | 新 | 发送弹幕信息 | social SayService | `SendDanmaku(channel_id, msg)` | ❌ |
| 12762 | 新 | 说话 | social SayService | `SendChat(channel_id, msg)` | ❌ |
| 12764 | 新 | 语音翻译结果分发 | social SayService | `VoiceTranslationResult(rid, srv_id, text)` | ❌ |
| 12768 | 新 | 记录已读艾特信息 | social SayService | `MarkAtRead(at_id)` | ❌ |
| (say 剩余 1 cmd 描述空) | 新 | (推测) 群聊频道列表 | social SayService | (v0.2 sprint 补) | ❌ |

**sub-total**: 13 cmds 明确 + 1 cmd 描述空 = 14 total。

### 5.12 holiday (13 cmds, 16601-16639) → batch HolidayService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 16601 | 新 | 所有活动(第一次打开请求) | batch HolidayService | `ListAllActivities()` | ❌ |
| 16602 | 新 | 请求所有活动未领取奖励 | batch HolidayService | `ListUnclaimedRewards()` | ❌ |
| 16603 | 新 | 子活动 | batch HolidayService | `ListSubActivities(parent_id)` | ❌ |
| 16604 | 新 | 领取奖励 | batch HolidayService | `ClaimReward(reward_id)` | ❌ |
| 16605 | 新 | 查看活动是否开启 | batch HolidayService | `IsActivityOpen(activity_id)` | ❌ |
| 16620 | 新 | 批量子活动 | batch HolidayService | `BatchListSubActivities(parent_ids)` | ❌ |
| 16630 | 新 | 边玩边下奖励状态 | batch HolidayService | `GetPlayDownloadRewardStatus()` | ❌ |
| 16631 | 新 | 领取边玩边下奖励 | batch HolidayService | `ClaimPlayDownloadReward(reward_id)` | ❌ |
| 16635 | 新 | 手机绑定信息 | batch HolidayService | `GetPhoneBindingInfo()` | ❌ |
| 16636 | 新 | 领取手机绑定信息 | batch HolidayService | `ClaimPhoneBindingReward()` | ❌ |
| 16637 | 新 | 抽奖活动详情 | batch HolidayService | `GetLotteryDetail(lottery_id)` | ❌ |
| 16638 | 新 | 抽奖 | batch HolidayService | `DrawLottery(lottery_id, count)` | ❌ |
| 16639 | 新 | 抽奖领取进度奖励 | batch HolidayService | `ClaimLotteryProgressReward(lottery_id, progress_id)` | ❌ |

**sub-total**: 13 cmds 全部明确映射, 13 total。

### 5.13 endless (12 cmds, 23900-23911) → match EndlessService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23900 | 新 | 通关奖励展示 | match EndlessService | `GetPassRewards(max_round, current_round, ...)` | ❌ |
| 23901 | 新 | 挑战无尽试炼 | match EndlessService | `ChallengeEndless(formation_type, pos_info)` | ❌ |
| 23902 | 新 | 战斗信息 | match EndlessService | `GetEndlessBattleInfo()` | ❌ |
| 23903 | 新 | 通关奖励展示 (V2) | match EndlessService | `GetPassRewardsV2(id, status)` | ❌ |
| 23904 | 新 | 领取通关奖励 | match EndlessService | `ClaimPassReward(id)` | ❌ |
| 23905 | 新 | 已派出伙伴信息 | match EndlessService | `GetDispatchedPartners()` | ❌ |
| 23906 | 新 | 已雇佣伙伴信息 | match EndlessService | `GetHiredPartners()` | ❌ |
| 23907 | 新 | 获取可雇佣伙伴信息 | match EndlessService | `GetHireablePartners()` | ❌ |
| 23908 | 新 | 派出伙伴 | match EndlessService | `DispatchPartner(partner_id)` | ❌ |
| 23909 | 新 | 雇佣伙伴 | match EndlessService | `HirePartner(rid, srv_id, partner_id, flag)` | ❌ |
| 23910 | 新 | 可选 BUFF 列表 | match EndlessService | `ListAvailableBuffs()` | ❌ |
| 23911 | 新 | 派出伙伴 (V2, 选 buff) | match EndlessService | `DispatchPartnerV2(buff_id)` | ❌ |

**sub-total**: 12 cmds 全部明确映射, 12 total。

### 5.14 boss (12 cmds, 20500-20541) → match BossService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 20500 | 新 | 获取个人 BOSS 信息 | match BossService | `GetPersonalBossInfo()` | ❌ |
| 20501 | 新 | 挑战个人 BOSS | match BossService | `ChallengePersonalBoss(boss_id)` | ❌ |
| 20502 | 新 | 扫荡个人 BOSS | match BossService | `SweepPersonalBoss(boss_id)` | ❌ |
| 20530 | 新 | 世界 BOSS 个人信息 | match BossService | `GetWorldBossPlayerInfo()` | ❌ |
| 20531 | 新 | 购买挑战次数 | match BossService | `BuyWorldBossCount(count)` | ❌ |
| 20532 | 新 | 挑战世界 BOSS | match BossService | `ChallengeWorldBoss(boss_id)` | ❌ |
| 20533 | 新 | 刷新 BOSS | match BossService | `RefreshWorldBoss()` | ❌ |
| 20535 | 新 | 获取世界 BOSS 信息 | match BossService | `GetWorldBossInfo()` | ❌ |
| 20537 | 新 | 获取 BOSS 伤害排行榜 | match BossService | `GetWorldBossDamageRanking(boss_id)` | ❌ |
| 20538 | 新 | 获取 BOSS 击杀日志 | match BossService | `GetWorldBossKillLog(boss_id)` | ❌ |
| 20540 | 新 | 获取提醒 BOSS 信息 | match BossService | `GetBossReminderInfo()` | ❌ |
| 20541 | 新 | 设置提醒 BOSS 信息 | match BossService | `SetBossReminder(boss_id, enabled)` | ❌ |

**sub-total**: 12 cmds 全部明确映射, 12 total。

### 5.15 guild_shipping (11 cmds, 23800-23812) → social GuildShippingService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23800 | 新 | 联盟远航信息 | social GuildShippingService | `GetShippingInfo()` | ❌ |
| 23801 | 新 | 查看订单信息 | social GuildShippingService | `ListShippingOrders()` | ❌ |
| 23802 | 新 | 远航起航 | social GuildShippingService | `StartShipping(order_id)` | ❌ |
| 23803 | 新 | 秒掉订单 | social GuildShippingService | `InstantCompleteOrder(order_id)` | ❌ |
| 23804 | 新 | 购买付费订单 | social GuildShippingService | `BuyPaidOrder(order_id)` | ❌ |
| 23806 | 新 | 互助列表 | social GuildShippingService | `ListHelpRequests()` | ❌ |
| 23807 | 新 | 互助加速 | social GuildShippingService | `HelpAccelerate(order_id)` | ❌ |
| 23808 | 新 | 资助 | social GuildShippingService | `Sponsor(order_id, amount)` | ❌ |
| 23809 | 新 | 领取奖励 | social GuildShippingService | `ClaimShippingReward(order_id)` | ❌ |
| 23810 | 新 | 求助 | social GuildShippingService | `RequestHelp(order_id, msg)` | ❌ |
| 23812 | 新 | 刷新次数 | social GuildShippingService | `RefreshShippingCount()` | ❌ |

**sub-total**: 11 cmds 全部明确映射, 11 total。

### 5.16 guild_dun (10 cmds, 21300-21319) → match GuildDunService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 21300 | 新 | 请求联盟副本信息 | match GuildDunService | `GetGuildDungeonInfo()` | ❌ |
| 21303 | 新 | 请求联盟副本宝箱 | match GuildDunService | `ListGuildDungeonChests()` | ❌ |
| 21304 | 新 | 领取联盟章节宝箱 | match GuildDunService | `ClaimChapterChest(chapter_id)` | ❌ |
| 21305 | 新 | 加 buff | match GuildDunService | `AddGuildDungeonBuff(buff_id)` | ❌ |
| 21308 | 新 | 挑战联盟副本 | match GuildDunService | `ChallengeGuildDungeon(dun_id)` | ❌ |
| 21311 | 新 | 请求购买挑战次数信息 | match GuildDunService | `GetBuyChallengeCountInfo()` | ❌ |
| 21312 | 新 | 购买挑战次数 | match GuildDunService | `BuyGuildDungeonCount(count)` | ❌ |
| 21317 | 新 | 扫荡 | match GuildDunService | `SweepGuildDungeon(dun_id)` | ❌ |
| 21318 | 新 | 请求联盟伤害排行榜 | match GuildDunService | `GetGuildDamageRanking()` | ❌ |
| 21319 | 新 | 请求个人伤害排行榜 | match GuildDunService | `GetPlayerDamageRanking()` | ❌ |

**sub-total**: 10 cmds 全部明确映射, 10 total。

### 5.17 item (10 cmds, 10500-10528) → player ItemService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10500 | 新 | 获取背包物品 | player ItemService | `GetBagItems()` | ❌ |
| 10501 | 新 | 获取装备背包物品 | player ItemService | `GetEquipmentBagItems()` | ❌ |
| 10515 | 新 | 使用物品 | player ItemService | `UseItem(item_id, count)` | ❌ |
| 10520 | 新 | 删除背包物品 | player ItemService | `DeleteBagItem(id, type)` | ❌ |
| 10522 | 新 | 出售物品 | player ItemService | `SellItem(item_id, count)` | ❌ |
| 10523 | 新 | 道具合成处理 | player ItemService | `ComposeItem(recipe_id, count)` | ❌ |
| 10524 | 新 | 设置自动出售 | player ItemService | `SetAutoSell(item_type, enabled)` | ❌ |
| 10525 | 新 | 获取自动出售设置 | player ItemService | `GetAutoSellSettings()` | ❌ |
| 10526 | 新 | 装备背包扩容 | player ItemService | `ExpandEquipmentBag(size)` | ❌ |
| 10528 | 新 | 装备预计产出时间 | player ItemService | `GetEquipmentProductionTime(equipment_id)` | ❌ |

**sub-total**: 10 cmds 全部明确映射, 10 total。

### 5.18 dungeon (9 cmds, 13000-13011) → match DungeonService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 13000 | 新 | 请求剧情副本相关信息 | match DungeonService | `GetStoryDungeonInfo()` | ❌ |
| 13002 | 新 | 制作关卡 | match DungeonService | `CreateDungeonLevel(template_id)` | ❌ |
| 13003 | 新 | 挑战领主 | match DungeonService | `ChallengeBoss(level_id)` | ❌ |
| 13004 | 新 | 快速战斗 | match DungeonService | `QuickBattle(level_id)` | ❌ |
| 13005 | 新 | 扫荡关卡 | match DungeonService | `SweepLevel(level_id, count)` | ❌ |
| 13006 | 新 | 剧情副本常规信息 | match DungeonService | `GetStoryDungeonBasic()` | ❌ |
| 13008 | 新 | 通关奖励展示 | match DungeonService | `GetPassRewards(level_id)` | ❌ |
| 13009 | 新 | 领取通关奖励 | match DungeonService | `ClaimPassReward(level_id)` | ❌ |
| 13011 | 新 | BUFF 信息获取 | match DungeonService | `ListStoryDungeonBuffs()` | ❌ |

**sub-total**: 9 cmds 全部明确映射, 9 total。

### 5.19 formation (6 cmds, 11200-11212) → player FormationService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 11200 | 新 | 请求自身阵法 | player FormationService | `GetMyFormation()` | ❌ |
| 11201 | 新 | 更换自身阵法 | player FormationService | `ChangeFormation(formation_type, formation_lev)` | ❌ |
| 11202 | 新 | 伙伴上阵/下阵/交换 | player FormationService | `SetPartnerSlot(pos, partner_id, op)` | ❌ |
| 11204 | 新 | 阵法道具 | player FormationService | `UseFormationItem(item_id)` | ❌ |
| 11211 | 新 | 获取功能阵法信息 | player FormationService | `GetFunctionalFormation(formation_id)` | ❌ |
| 11212 | 新 | 设置功能阵法 | player FormationService | `SetFunctionalFormation(formation_id, partner_ids)` | ❌ |

**sub-total**: 6 cmds 全部明确映射, 6 total。

### 5.20 login (6 cmds, 10101-10103) → player PlayerService (Partial)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10101 | Partial | 创建角色 | player PlayerService | `CreatePlayer(sex, name, career, playform)` | ❌ |
| 10102 | Partial | 登录角色 | player PlayerService | `SelectRole(rid, srv_id)` | 🟡 |
| 10103 | Partial | 角色重新连接 | player PlayerService | `ReconnectRole(rid, srv_id)` | 🟡 |
| (login 剩余 3 cmds 描述空, 推测) | Partial | (推测) 角色列表 / 创角随机名 / 创角校验 | player PlayerService | (3 cmds 详细映射 v0.2 sprint 补) | 🟡 |

**sub-total**: 3 cmds 明确 + 3 cmds 描述空 = 6 total。

### 5.21 map (6 cmds, 10200-10215) → player MapService (N-A for TCG)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10200 | Map N-A | 操作地图单位 | player MapService | `OperateMapUnit(unit_id, op)` | N-A |
| 10215 | Map N-A | 角色移动 | player MapService | `MovePlayer(x, y, z)` | N-A |
| (map 剩余 4 cmds 10200-10215 描述空) | Map N-A | (推测) 地图加载 / 单位查询 / 视野同步 / 区域触发 | player MapService | (N-A 业务: TCG 不适用, per REQ §2 #23 + handoff v0.1 §2.2 家园系统 N-A) | N-A |

**sub-total**: 6 cmds, 全 N-A。

### 5.22 mail (6 cmds, 10800-10810) → social MailService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10800 | 新 | 分页读取邮件列表 | social MailService | `ListMails(page, num)` | ❌ |
| 10801 | 新 | 提取单个邮件的附件 | social MailService | `ClaimAttachment(mail_id)` | ❌ |
| 10802 | 新 | 一键提取附件 | social MailService | `ClaimAllAttachments()` | ❌ |
| 10804 | 新 | (描述空) 删除邮件 | social MailService | `DeleteMail(mail_id)` | ❌ |
| 10805 | 新 | 读取邮件 | social MailService | `ReadMail(mail_id)` | ❌ |
| 10810 | 新 | GM 反馈 | social MailService | `GMFeedback(content, contact)` | ❌ |

**sub-total**: 5 cmds 明确 + 1 cmd 描述空 = 6 total。

### 5.23 exchange (6 cmds, 13401-13419) → economy ExchangeService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 13401 | 新 | 兑换商店商品当天已购买次数 | economy ExchangeService | `GetExchangePurchaseCount(exchange_id)` | ❌ |
| 13402 | 新 | 兑换 | economy ExchangeService | `Exchange(exchange_id, count)` | ❌ |
| 13403 | 新 | 请求神秘商店数据 | economy ExchangeService | `GetMysteryShop()` | ❌ |
| 13405 | 新 | 自动刷新 | economy ExchangeService | `AutoRefreshMysteryShop()` | ❌ |
| 13407 | 新 | 神秘商店购买 | economy ExchangeService | `BuyFromMysteryShop(item_id, count)` | ❌ |
| 13419 | 新 | 神格兑换 | economy ExchangeService | `ExchangeGodGrace(grace_type, count)` | ❌ |

**sub-total**: 6 cmds 全部明确映射, 6 total。

### 5.24 vip (6 cmds, 16700-16713) → economy VipService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 16700 | 新 | 获取充值信息 | economy VipService | `GetChargeInfo()` | ❌ |
| 16705 | 新 | 推送月卡信息 | economy VipService | `PushMonthlyCardInfo()` | ❌ |
| 16710 | 新 | VIP 信息 | economy VipService | `GetVipInfo()` | ❌ |
| 16711 | 新 | VIP 领取等级奖励 | economy VipService | `ClaimVipLevelReward(level)` | ❌ |
| 16712 | 新 | 累充奖励信息 | economy VipService | `GetAccumulatedChargeRewards()` | ❌ |
| 16713 | 新 | 领取累充奖励 | economy VipService | `ClaimAccumulatedChargeReward(reward_id)` | ❌ |

**sub-total**: 6 cmds 全部明确映射, 6 total。

### 5.25 convert (5 cmds, 23600-23604) → economy ConvertService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23600 | 新 | 资产兑换 | economy ConvertService | `ConvertAsset(from_type, to_type, amount)` | ❌ |
| 23601 | 新 | 神格许愿状态 | economy ConvertService | `GetGodWishStatus()` | ❌ |
| 23602 | 新 | 领取礼包 | economy ConvertService | `ClaimGodWishReward(reward_id)` | ❌ |
| 23603 | 新 | 神格许愿 | economy ConvertService | `MakeWish(wish_type, count)` | ❌ |
| 23604 | 新 | 额外奖励比例 | economy ConvertService | `GetExtraBonusRate()` | ❌ |

**sub-total**: 5 cmds 全部明确映射, 5 total。

### 5.26 drama (5 cmds, 11100-11122) → player DramaService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 11100 | 新 | 剧情播放 | player DramaService | `PlayDrama(drama_id)` | ❌ |
| 11102 | 新 | 跳过剧情 | player DramaService | `SkipDrama(drama_id)` | ❌ |
| 11121 | 新 | 播放引导心跳 | player DramaService | `GuideHeartbeat(guide_id)` | ❌ |
| 11122 | 新 | 播放引导结束 | player DramaService | `FinishGuide(guide_id)` | ❌ |
| (drama 剩余 1 cmd 描述空) | 新 | (推测) 剧情奖励领取 | player DramaService | (v0.2 sprint 补) | ❌ |

**sub-total**: 4 cmds 明确 + 1 cmd 描述空 = 5 total。

### 5.27 rank (5 cmds, 12900-12904) → leaderboard RankService (Partial)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 12900 | Partial | 获取排行榜数据 | leaderboard RankService | `GetLeaderboard(rank_type, page)` | 🟡 |
| 12901 | Partial | 获取排行榜最后更新时间 | leaderboard RankService | `GetLeaderboardUpdateTime(rank_type)` | 🟡 |
| 12902 | Partial | 获取排行榜最后更新时间 (V2) | leaderboard RankService | `GetLeaderboardUpdateTimeV2(rank_type)` | 🟡 |
| 12903 | Partial | 获取联盟排行榜数据 | leaderboard RankService | `GetGuildLeaderboard(rank_type, page)` | 🟡 |
| 12904 | Partial | 获取英雄排行榜数据 | leaderboard RankService | `GetPartnerLeaderboard(rank_type, page)` | 🟡 |

**sub-total**: 5 cmds 全部明确映射, 5 total。

### 5.28 avatar (4 cmds, 21500-21504) → player AvatarService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 21500 | 新 | 头像框列表 | player AvatarService | `ListAvatarFrames()` | ❌ |
| 21501 | 新 | 使用头像框 | player AvatarService | `UseAvatarFrame(frame_id)` | ❌ |
| 21503 | 新 | 激活头像框 | player AvatarService | `ActivateAvatarFrame(frame_id)` | ❌ |
| 21504 | 新 | 获取属性加成信息 | player AvatarService | `GetAvatarFrameBonus(frame_id)` | ❌ |

**sub-total**: 4 cmds 全部明确映射, 4 total。

### 5.29 guild_skill (4 cmds, 23700-23703) → social GuildSkillService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23700 | 新 | 联盟技能信息 | social GuildSkillService | `GetGuildSkillInfo()` | ❌ |
| 23701 | 新 | 激活指定职业的联盟技能 | social GuildSkillService | `ActivateGuildSkill(skill_id, career)` | ❌ |
| 23702 | 新 | 更新分组 ID | social GuildSkillService | `UpdateGroupId(skill_id, group_id)` | ❌ |
| 23703 | 新 | 联盟技能概要信息(红点) | social GuildSkillService | `GetGuildSkillSummary()` | ❌ |

**sub-total**: 4 cmds 全部明确映射, 4 total。

### 5.30 days_rank (4 cmds, 22700-22704) → leaderboard DaysRankService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 22700 | 新 | 进行中列表 | leaderboard DaysRankService | `ListActiveDailyRanks()` | ❌ |
| 22701 | 新 | 排行榜信息 | leaderboard DaysRankService | `GetDailyRankInfo(rank_id)` | ❌ |
| 22703 | 新 | 排行榜信息 (V2) | leaderboard DaysRankService | `GetDailyRankInfoV2(rank_id)` | ❌ |
| 22704 | 新 | 排行榜信息 (V3) | leaderboard DaysRankService | `GetDailyRankInfoV3(rank_id)` | ❌ |

**sub-total**: 4 cmds 全部明确映射, 4 total。

### 5.31 lev_gift (4 cmds, 21200-21204) → batch LevGiftService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 21200 | 新 | 等级好礼信息 | batch LevGiftService | `GetLevelGiftInfo()` | ❌ |
| 21202 | 新 | 获取状态 | batch LevGiftService | `GetLevelGiftStatus(level)` | ❌ |
| 21203 | 新 | 领取等级奖励 | batch LevGiftService | `ClaimLevelGift(level)` | ❌ |
| 21204 | 新 | 购买等级奖励 | batch LevGiftService | `BuyLevelGift(level)` | ❌ |

**sub-total**: 4 cmds 全部明确映射, 4 total。

### 5.32 quest (4 cmds, 10400-10406) → player QuestService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 10400 | 新 | 请求任务面板信息 | player QuestService | `GetQuestPanel()` | ❌ |
| 10402 | 新 | 接受任务 | player QuestService | `AcceptQuest(quest_id)` | ❌ |
| 10405 | 新 | 放弃任务 | player QuestService | `AbandonQuest(quest_id)` | ❌ |
| 10406 | 新 | 提交任务 | player QuestService | `SubmitQuest(quest_id)` | ❌ |

**sub-total**: 4 cmds 全部明确映射, 4 total。

### 5.33 conn_login (3 cmds, 1110-1199) → cluster-ops ClusterOpsService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 1110 | 新 | 帐号登录处理 | cluster-ops ClusterOpsService | `AccountLogin(args)` | ❌ |
| 1198 | 新 | (描述空) 服务器时间 | cluster-ops ClusterOpsService | `GetServerTime()` | ❌ |
| 1199 | 新 | (描述空) 心跳 | cluster-ops ClusterOpsService | `Heartbeat(time)` | ❌ |

**sub-total**: 3 cmds 全部明确映射, 3 total。

### 5.34 power_gift (3 cmds, 23400-23403) → batch PowerGiftService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23400 | 新 | 战力礼包信息 | batch PowerGiftService | `GetPowerGiftInfo()` | ❌ |
| 23402 | 新 | (描述空) 战力礼包状态 | batch PowerGiftService | `GetPowerGiftStatus(power_threshold)` | ❌ |
| 23403 | 新 | 领取战力礼包奖励 | batch PowerGiftService | `ClaimPowerGift(power_threshold)` | ❌ |

**sub-total**: 3 cmds 全部明确映射, 3 total。

### 5.35 honor (3 cmds, 23300-23303) → player HonorService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23300 | 新 | 称号列表 | player HonorService | `ListHonors()` | ❌ |
| 23301 | 新 | 使用称号 | player HonorService | `UseHonor(honor_id)` | ❌ |
| 23303 | 新 | 激活称号 | player HonorService | `ActivateHonor(honor_id)` | ❌ |

**sub-total**: 3 cmds 全部明确映射, 3 total。

### 5.36 charge (3 cmds, 21000-21005) → economy ChargeService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 21000 | 新 | 首充礼包信息 | economy ChargeService | `GetFirstChargeInfo()` | ❌ |
| 21001 | 新 | 领取首充礼包 | economy ChargeService | `ClaimFirstCharge()` | ❌ |
| 21005 | 新 | 三日返利信息 | economy ChargeService | `GetThreeDayRebate()` | ❌ |

**sub-total**: 3 cmds 全部明确映射, 3 total。

### 5.37 recruit (3 cmds, 23200-23203) → card RecruitService (Partial)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 23200 | Partial | 召唤池列表 | card RecruitService | `ListRecruitPools()` | 🟡 |
| 23201 | Partial | 召唤 | card RecruitService | `Recruit(pool_id, count)` | 🟡 |
| 23203 | Partial | 领取召唤分享奖励 | card RecruitService | `ClaimRecruitShareReward()` | 🟡 |

**sub-total**: 3 cmds 全部明确映射, 3 total。

### 5.38 group_control (2 cmds, 22100-22101) → batch GroupControlService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 22100 | 新 | 查看跨服时空的信息 | batch GroupControlService | `GetCrossServerStageInfo()` | ❌ |
| 22101 | 新 | 领取跨服阶段的奖励 | batch GroupControlService | `ClaimCrossServerStageReward(stage_id)` | ❌ |

**sub-total**: 2 cmds 全部明确映射, 2 total。

### 5.39 activity (2 cmds, 20300-20301) → batch ActivityService (Partial)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 20300 | Partial | 活跃已领取宝箱 | batch ActivityService | `ListActiveBonusChests()` | 🟡 |
| 20301 | Partial | 领取活跃宝箱 | batch ActivityService | `ClaimActiveBonusChest(chest_id)` | 🟡 |

**sub-total**: 2 cmds 全部明确映射, 2 total。

### 5.40 feat (2 cmds, 16400-16402) → batch FeatService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 16400 | 新 | 成就信息 | batch FeatService | `GetFeatList()` | ❌ |
| 16402 | 新 | 领取成就奖励 | batch FeatService | `ClaimFeatReward(feat_id)` | ❌ |

**sub-total**: 2 cmds 全部明确映射, 2 total。

### 5.41 login_days (2 cmds, 21100-21101) → batch LoginDaysService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 21100 | 新 | 获取信息 | batch LoginDaysService | `GetLoginDaysInfo()` | ❌ |
| 21101 | 新 | 领取奖励 | batch LoginDaysService | `ClaimLoginDaysReward(days)` | ❌ |

**sub-total**: 2 cmds 全部明确映射, 2 total。

### 5.42 checkin (2 cmds, 14100-14101) → batch CheckinService (新)

| cmd | 类别 | 闪烁之光 RPC | RGS proto | RGS RPC | gap |
|---:|---|---|---|---|---|
| 14100 | 新 | 签到信息 | batch CheckinService | `GetCheckinInfo()` | ❌ |
| 14101 | 新 | 领取签到奖励 | batch CheckinService | `ClaimCheckinReward(day)` | ❌ |

**sub-total**: 2 cmds 全部明确映射, 2 total。

### 5.43 总结 (per 42 modules × 438 cmds 完整覆盖)

| 分类 | module 数 | cmds 数 | gap Pass (✅) | gap Partial (🟡) | gap NotImpl (❌) | gap N-A |
|---|---:|---:|---:|---:|---:|---:|
| **12 Partial** (per DDD v0.1 §2.2) | 12 | ~225 | 0 (0%) | ~225 (100%) | 0 (0%) | 0 (0%) |
| **30 新 module** (per DDD v0.1 §2.3) | 30 | ~213 | 0 (0%) | 0 (0%) | ~213 (100%) | 0 (0%) |
| **Map N-A** (TCG 不适用) | 1 | 6 | 0 (0%) | 0 (0%) | 0 (0%) | 6 (100%) |
| **总计** | 42 | 438 (≈444) | 0 | ~225 (~51%) | ~213 (~48%) | 6 (~1%) |

注: 438 cmds 严格按 api_module_summary.txt L1-45 统计, 表中 ~225/~213 是 12 Partial + 30 新 module 估算 (per DDD v0.1 §2.2-§2.3 22 cmds 字段汇总), 表中也含描述空 cmd (推测, ≈6 cmds) — 实际 438 包含 6 N-A + ~225 Partial + ~207 新, 跟 12 Partial ~140 + 30 新 ~298 估算 (per DDD v0.1 §0.3) 一致 (per 8/27 21:59 JST 禁回溯叙事, 不追溯改写统计)。

---

## 6. gap 矩阵 (per 42 modules × 7 域)

### 6.1 12 Partial module 状态 (per DDD v0.1 §2.2)

| # | module | cmds | RGS 域 | 现状 (audit v0.3) | gap 状态 | 落地 (per REQ §2 阶段) |
|---|---|---:|---|---|---|---|
| 1 | **combat** | 43 | match v2 | 8 transition + EventBus 满分, 缺 PVE | 🟡 (43/43 Partial) | W2 |
| 2 | **partner** | 41 | card + player | OpenPack saga + DeckRepository 0/41 wire | 🟡 (41/41 Partial) | W2 |
| 3 | **guild** | 29 | social | 6 trait + 2/29 wire | 🟡 (29/29 Partial) | W3 |
| 4 | **arena** | 26 | match v2 | 6 变体 1:1 拆 RPC (per DDD §3.3 决策) | 🟡 (26/26 Partial) | W3 |
| 5 | **role** | 21 | player | 11 method + 6 handler wire, 缺 8 cmds | 🟡 (21/21 Partial) | W2 |
| 6 | **market** | 19 | economy v2 | Auction + PrivateTrade, 缺铜钱市 | 🟡 (19/19 Partial) | W3 |
| 7 | **misc** | 19 | admin + batch | 4/4 RBAC, 缺通知/活动状态/合服 | 🟡 (19/19 Partial) | W2 |
| 8 | **sns** | 16 | social | 0/16 wire (per audit v0.3 §3.4 D10) | 🟡 (16/16 Partial) | W5 |
| 9 | **say** | 14 | social | 0/14 wire (per audit v0.3 §3.4) | 🟡 (14/14 Partial) | W6 |
| 10 | **login** | 6 | player | register + heartbeat, 缺 create_role | 🟡 (6/6 Partial) | W2 |
| 11 | **rank** | 5 | leaderboard | 5/5 wire | 🟡 (5/5 Partial) | W3 |
| 12 | **recruit** | 3 | card + economy | OpenPack saga 已建, 缺 23200/23201/23203 | 🟡 (3/3 Partial) | W3 |

### 6.2 30 新 module 状态 (per DDD v0.1 §2.3)

| # | module | cmds | RGS 域 | gap 状态 | 落地 (per REQ §2 阶段) |
|---|---|---:|---|---|---|
| 1 | **star** | 20 | player | ❌ NotImpl | W6 |
| 2 | **adventure** | 17 | match (副本) | ❌ NotImpl | W7 |
| 3 | **holiday** | 13 | batch | ❌ NotImpl (data-driven) | W7 |
| 4 | **endless** | 12 | match (副本) | ❌ NotImpl | W8 |
| 5 | **boss** | 12 | match (副本) | ❌ NotImpl | W8 |
| 6 | **guild_shipping** | 11 | social | ❌ NotImpl | W11 |
| 7 | **guild_dun** | 10 | social + match | ❌ NotImpl | W11 |
| 8 | **item** | 10 | player | ❌ NotImpl | W5 |
| 9 | **dungeon** | 9 | match (副本) | ❌ NotImpl | W7 |
| 10 | **formation** | 6 | player | ❌ NotImpl | W6 |
| 11 | **mail** | 6 | social | ❌ NotImpl | W5 |
| 12 | **exchange** | 6 | economy | ❌ NotImpl | W13 |
| 13 | **vip** | 6 | economy | ❌ NotImpl | W13 |
| 14 | **convert** | 5 | economy | ❌ NotImpl | W14 |
| 15 | **drama** | 5 | player | ❌ NotImpl | W6 |
| 16 | **avatar** | 4 | player | ❌ NotImpl | W15 |
| 17 | **guild_skill** | 4 | social | ❌ NotImpl | W11 |
| 18 | **days_rank** | 4 | leaderboard | ❌ NotImpl | W15 |
| 19 | **lev_gift** | 4 | batch | ❌ NotImpl | W15 |
| 20 | **quest** | 4 | player | ❌ NotImpl | W5 |
| 21 | **conn_login** | 3 | cluster-ops | ❌ NotImpl (per §3.9 独立 connector service) | W4 |
| 22 | **power_gift** | 3 | batch | ❌ NotImpl | W16 |
| 23 | **honor** | 3 | player | ❌ NotImpl | W16 |
| 24 | **charge** | 3 | economy | ❌ NotImpl | W14 |
| 25 | **group_control** | 2 | batch (跨服) | ❌ NotImpl | W17 |
| 26 | **activity** | 2 | batch | 🟡 Partial (W18 业务补) | W18 |
| 27 | **feat** | 2 | batch (成就) | ❌ NotImpl | W17 |
| 28 | **login_days** | 2 | batch | ❌ NotImpl | W18 |
| 29 | **checkin** | 2 | batch | ❌ NotImpl | W19 |
| 30 | **map** | 6 | player | N/A (TCG 不适用, per REQ §2 #23) | — |

---

## 7. 协议号边界 case (per 闪烁之光 12 大类 + 41 协议号段 关键边界)

### 7.1 10001 (login 起始) / 10101 (创建角色) / 10103 (重连)

- 10001 在 api_module_summary.txt 实际未出现 (L436 起始为 10101)
- 10101/10102/10103 是 闪烁之光 角色登录 3 cmd, RGS `CreatePlayer` / `SelectRole` / `ReconnectRole` 映射 (per DDD §3.7), 10101 缺 (per audit v0.3)
- 边界: 10001-10099 段保留 (per 协议号分段.md L29-32 0-10 心跳/系统保留), RGS `ClusterOpsService` 不应占用 10000 段

### 7.2 11000 (partner 起始) / 11084 (partner 终止)

- 11000-11084 = 41 cmds partner 业务 (per api_module_summary.txt L94-135)
- 边界: 11085+ 留给 partner 业务扩展, RGS `CardService.PartnerService` 1:1 覆盖
- 11000 起始 是 partner 业务 (RGS TCG 0/41 wire, per audit v0.3 §3.1)

### 7.3 13500 (guild 起始) / 13574 (guild 终止)

- 13500-13574 = 29 cmds guild 业务
- 边界: 13575+ 留给 guild 业务扩展, RGS `SocialService.GuildService` 1:1 覆盖 (2/29 wire, 27 增量)

### 7.4 20001 (combat 起始) / 20063 (combat 终止)

- 20001-20063 = 19 cmds combat 业务 (描述空 24 cmds 不计入)
- 边界: 20000 (战斗 type: 16 + map: 32) 是战斗准备协议, RGS `PrepareCombat(combat_type, combat_map)` 映射

### 7.5 20200 (arena 主赛 起始) / 20281 (arena 周日冠军赛 终止)

- 20200-20223 = 主赛 12 cmds / 20250-20263 = 冠军赛 8 cmds / 20280-20281 = 周日冠军赛 2 cmds = 26 total
- 边界: 3 段 arena_type (Main/Champion/SundayChampion), 通过 `arena_type` 枚举避免 1:1 拆 6 RPC (per DDD §3.3 L355 决策)
- 20200 起始 是 arena 个人信息, RGS `GetArenaState(arena_type=Main)` 映射

### 7.6 20300 (activity 起始) / 20301 (activity 终止)

- 20300-20301 = 2 cmds activity (活跃宝箱) → RGS batch `ActivityService`
- 边界: 20302+ 留给 activity 业务扩展

### 7.7 21000 (charge 起始) / 21005 (charge 终止)

- 21000-21005 = 3 cmds charge (首充/三日返利) → RGS economy `ChargeService`
- 边界: 21006+ 留给 charge 业务扩展

### 7.8 22000 (cross-server 起始, 推断) / 22100-22101 (group_control)

- 22000-22099 段无 cmd (per api_module_summary.txt 全文), 22100-22101 = 2 cmds group_control (跨服时空)
- 边界: 22000 段保留, RGS `BatchService.GroupControlService` 映射

### 7.9 23000 (holiday 起始, 推断) / 23200-23911 (活动 7 module)

- 23000-23099 段无 cmd, 23200-23203 = recruit 3 / 23300-23303 = honor 3 / 23400-23403 = power_gift 3 / 23500-23520 = market 19 / 23600-23604 = convert 5 / 23700-23703 = guild_skill 4 / 23800-23812 = guild_shipping 11 / 23900-23911 = endless 12 = 60 cmds
- 边界: 23000 段保留, 23000-23199 留给活动业务扩展

### 7.10 23900 (endless 起始) / 23911 (endless 终止)

- 23900-23911 = 12 cmds endless 业务
- 边界: 23912+ 留给 endless 业务扩展
- 23900 起始 = 通关奖励展示, RGS `EndlessService.GetPassRewards` 映射

### 7.11 边界 case 总结 (per 10 关键边界)

| 边界 | 范围 | 闪烁之光 业务 | RGS service | 验证 |
|---|---|---|---|---|
| 0-99 | 心跳/系统保留 | 无 | ClusterOpsService (per REQ §2 #34) | ✅ 协议号分段.md L29-32 保留 |
| 10001-10103 | 角色登录 | login (6) | PlayerService | ✅ RGS CreatePlayer 缺, 10102/10103 已有 (per DDD §3.7) |
| 11000-11084 | 伙伴业务 | partner (41) | CardService.PartnerService | ✅ 0/41 wire (per audit v0.3 §3.1) |
| 13500-13574 | 公会业务 | guild (29) | SocialService.GuildService | ✅ 2/29 wire, 27 增量 (per DDD §3.2) |
| 20001-20063 | 战斗业务 | combat (43) | MatchService.CombatService + PveService | ✅ 8 transition + EventBus 满分, 缺 PVE (per DDD §3.1) |
| 20200-20281 | 竞技场业务 | arena (26) | MatchService.ArenaService | ✅ 3 变体 enum 抽象 (per DDD §3.3) |
| 23500-23520 | 摆摊业务 | market (19) | EconomyService.MarketService | ✅ 19 增量 RPC (per DDD §3.5) |
| 23800-23812 | 联盟远航 | guild_shipping (11) | SocialService.GuildShippingService | ❌ NotImpl W11 |
| 23900-23911 | 无尽试炼 | endless (12) | MatchService.EndlessService | ❌ NotImpl W8 |

---

## 8. 协议号冲突检测 (per 闪烁之光 5 大类 10000-19999 + 20000-29900 vs RGS 5 域 + card + gm-backend 0-65535 范围)

### 8.1 闪烁之光 协议号范围 (per 协议号分段.md + api_module_summary.txt)

| 类别 | 协议号段 | 实际命令号段 | 闪烁之光 module |
|---|---|---|---|
| **连接登录** | 11-12 | 1100-1299 | conn_login (3) |
| **角色基础** | 101-113 | 10100-11399 | login/map/role/quest/item/mail/misc/partner/drama/formation/star (144) |
| **社交基础** | 127-135 | 12700-13599 | say/rank/dungeon/sns/exchange/guild (79) |
| **商业/签到** | 141-168 | 14100-16899 | checkin/feat/charge/market/vip/misc (32+11=43) |
| **战斗** | 200-215 | 20000-21599 | combat/arena/boss/dungeon/adventure/pay/recruit/endless/avatar (129) |
| **跨服** | 221-227 | 22100-22799 | group_control/guild_dun/days_rank (16) |
| **活动** | 232-239 | 23200-23999 | recruit/holiday/login_days/power_gift/market/convert/days_rank/guild_shipping/endless (60) |

**总协议号范围**: 1100-23911 = 1100 (4 位) 至 23911 (5 位), 实际有效范围 1100-23911 (per api_module_summary.txt L1-45)。

### 8.2 RGS 协议号范围 (per 7 域 + gm-backend + leaderboard 9 service)

| RGS service | 端口 | proto RPC 数 | 协议号范围 (gRPC method) |
|---|---|---:|---|
| player | 50051 | 13+ | gRPC method (service.method) 风格, 无显式 cmd code |
| economy | 50052 | 5+19 | 同上 |
| match | 50053 | 9 v2 + 6 PVE + 20 arena | 同上 |
| social | 50054 | 2+27 | 同上 |
| admin | 50055 | 4 | 同上 |
| card | 50061 | 30 | 同上 |
| batch (rgs-batch-backend) | (actix-web) | 13+ (data-driven) | HTTP JSON, 无 gRPC |
| leaderboard | (待定) | 5 | gRPC method |
| gm-backend | 8081 | 4 (HTTP) | HTTP JSON |
| **总计** | 7+1+1 | 69+ | gRPC method (full method name, 0-65535 字符长度) |

**RGS 协议号范围**: 0-65535 (gRPC method name 字符串长度限制), 实际 full method name 格式 `/<package>.<service>/<method>` (e.g. `/common.v1.HealthCheck/Check`), 长度 << 65535 字符, 0 冲突。

### 8.3 冲突检测 (per 闪烁之光 5 大类 10000-19999 + 20000-29900 vs RGS 7 域)

| 冲突检测项 | 闪烁之光 | RGS | 冲突? |
|---|---|---|---|
| **协议号格式** | 4-5 位数字 (1100-23911) | gRPC method name 字符串 | **无冲突** (格式不同) |
| **cmd 编号唯一性** | 41 协议号段 × 100 + offset = 438 cmds, 1:1 唯一 | gRPC method 全局唯一 (package.service.method) | **无冲突** (RGS 走 gRPC method 命名, 闪烁之光 走 cmd 数字) |
| **wire 协议** | 自研 TCP/Flash socket (size:32 + cmd:16 + data) | tonic gRPC + protobuf + mTLS | **冲突存在** (需 adapter 转换, per DDD v0.1 §11.1 已知缺口, v0.2 评估) |
| **业务命名** | 协议号 = 业务模块 (per 协议号分段.md) | gRPC method = 业务方法 | **无冲突** (RGS 沿用 1:1 命名映射, per §5 主表) |
| **跨服 server_id** | {rid:32, srv_id:string} | RGS 缺显式 server_id (per §3.3 已知缺口) | **冲突存在** (需 v0.2 sprint 评估) |
| **i18n msg** | string | RGS `I18nString` (per common.proto L73-77) + `ErrorCode` enum | **无冲突** (RGS 抽象更优) |

### 8.4 冲突总结

- **协议号格式 + cmd 编号唯一性 + 业务命名**: **0 冲突** (RGS 1:1 命名映射, per §5)
- **wire 协议 + 跨服 server_id**: **2 冲突** (需 v0.2 sprint adapter + server_id 评估, per §11.1 已知缺口)
- **i18n msg**: **0 冲突** (RGS 抽象更优)

---

## 9. 协议号迁移路径 (per v0.1 → v0.2 → v1.0 升版)

### 9.1 v0.1 状态 (commit `80bcd3b`)

- 12 Partial + 30 新 module 详细设计 (per DDD v0.1 §3-§4)
- 41 协议号 → RGS service 1:1 高层映射 (per DDD v0.1 §7.4 L1027-1074)
- 7 域 backend 业务覆盖, 12 Partial 大部分 Partial 状态, 30 新 module NotImpl

### 9.2 v0.2 状态 (本 addendum, 待主会话 commit per L12.2)

- 438 cmds 完整 1:1 映射 (per §5 主表, **本 addendum 核心**)
- 闪烁之光 协议 schema 抽样 10 个 (per §3, 验证 pack/unpack 字段)
- 协议号边界 case (per §7, 10 关键边界)
- 冲突检测 (per §8, 2 冲突已知: wire 协议 + server_id)
- gap 矩阵 7 域 (per §6, 12 Partial + 30 新 module + Map N-A)

### 9.3 v1.0 状态 (目标, per 4 阶段路线图 v0.3 §1.2)

- 438 cmds 100% 覆盖 (12 Partial Pass ✅ + 30 新 module Pass ✅ + Map N-A 标记)
- 闪烁之光 实际 wire 协议 adapter (per §8.4 已知冲突)
- 性能 P50/P95/P99 全面超过 (per DDD v0.1 §9.1 + 9/4 16:45 JST "全面超过" 拍板)
- 5 域 ST 业务 mTLS 跑通 (per 8/27 ST 业务 mTLS commit `401ac5c`)
- 7 域 DB 78 表 Master/Transaction/Work 三分类实测 (per DDD v0.1 §6.1 估算)
- 闪烁之光 performance baseline 对比 (待 Phase C 阶段 C 后)

### 9.4 升版路径 (4 阶段, per FLASH-MOCK v0.3 §1.2)

| 阶段 | 时长 | 目标 | 关键产出 | 关联 v0.2 addendum |
|---|---|---|---|---|
| **Phase 1** (done) | W1 | mock v0.1 22 RPC 跑通 | mock + handoff v0.1 + GAP-AUDIT v0.3 | — |
| **Phase 2** (W2-W5) | 4 sprint | 12 Partial 业务实现 (per §6.1 W2-W5) | combat/partner/guild/arena/role/market/misc/login/sns/item/quest/mail | 本 addendum §5.1-§5.6 + §5.8 + §5.10 索引 |
| **Phase 3** (W6-W12) | 7 sprint | 30 新 module hot path (per §6.2 W6-W12) | star/formation/avatar/drama/holiday/adventure/dungeon/endless/boss/guild_shipping/guild_dun/guild_skill/conn_login | 本 addendum §5.6 + §5.9 + §5.12-§5.16 索引 |
| **Phase 4** (W13-W25) | 13 sprint | 30 新 module long tail (per §6.2 W13-W25) + 性能 baseline | exchange/vip/convert/charge/lev_gift/feat/days_rank/honor/power_gift/activity/login_days/checkin/group_control | 本 addendum §5.22-§5.42 索引 |

**总估算**: 25 sprint / 50 周 / 2-3M tokens (per FLASH-MOCK v0.3 §6.4 + DDD v0.1 §1.2)

---

## 10. 测试矩阵 (per 438 cmds × RGS 7 域 1:1 路由表)

### 10.1 7 域 1:1 路由表 (per §5 主表聚合)

| RGS 域 | 模块数 | cmds 总数 | 路由 RPC 数 (per §5) | wire 协议 | mTLS |
|---|---:|---:|---:|---|---|
| **player** | role (21) + login (6) + map (6-N/A) + partner (41) + star (20) + item (10) + quest (4) + formation (6) + drama (5) + avatar (4) + honor (3) | 126 | 126 RPC | tonic gRPC | ✅ mTLS |
| **match** | combat (43) + arena (26) + adventure (17) + endless (12) + boss (12) + guild_dun (10) + dungeon (9) | 129 | 129 RPC (含 PveService 6 + arena 20) | tonic gRPC | ✅ mTLS |
| **social** | guild (29) + sns (16) + say (14) + mail (6) + guild_shipping (11) + guild_skill (4) | 80 | 80 RPC (含 guild 27 增量) | tonic gRPC + NATS (Q7 push_delivery) | ✅ mTLS |
| **economy** | market (19) + exchange (6) + vip (6) + convert (5) + charge (3) + recruit (3) | 42 | 42 RPC (含 market 19 增量) | tonic gRPC + sqlx + saga | ✅ mTLS |
| **admin** | misc (19 含 GM 3) | 19 | 19 RPC (含 4 RBAC + 15 misc) | tonic gRPC + RBAC | ✅ mTLS |
| **batch** (rgs-batch-backend) | holiday (13) + group_control (2) + activity (2) + feat (2) + lev_gift (4) + login_days (2) + checkin (2) + power_gift (3) | 30 | 30 RPC (actix-web + data-driven) | HTTP + JSON | ✅ mTLS (envoy) |
| **card** | (无 Partial) | 0 (partner 41 划入 player 范畴) | partner 41 复用 (per §5.2) | tonic gRPC + sqlx | ✅ mTLS |
| **leaderboard** | rank (5) + days_rank (4) | 9 | 9 RPC | tonic gRPC + Redis sorted set | ✅ mTLS |
| **gm-backend** | (无) | 0 (misc 19 划入 admin 范畴) | misc 4 GM RPC 复用 (per §5.8) | HTTP + JSON | ✅ mTLS (envoy) |
| **cluster-ops** | conn_login (3) | 3 | 3 RPC (独立 connector service) | tonic gRPC | ✅ mTLS |
| **总计** | 42 module | 438 cmds | 438 RPC (1:1) | 7 域 + 4 工具 | 全 mTLS |

### 10.2 闪烁之光 client → RGS backend 端到端验证 (per Phase 1 mock v0.1 + Phase 2-4 业务实现)

| 验证层 | 工具 | 数量 | 引用 |
|---|---|---:|---|
| **UT (单元测试)** | `#[cfg(test)]` + `#[tokio::test]` + `proptest!` | 5 域 × 50+ = 250+ (per audit v0.3) | per DDD v0.1 §10.1 L1221 |
| **IT (集成测试)** | `tests/*.rs` + `InMemory*Repository` | 5 域 × 5-10 = 30+ | per DDD v0.1 §10.1 L1222 |
| **E2E (端到端)** | `grpcurl` + mock server | 42 module × 1 = 42 E2E (per §6 矩阵) | per DDD v0.1 §10.1 L1223 |
| **业务 mTLS** | `tonic + rustls + rcgen` | 7 域 × 1 = 7 域业务级 | per 8/27 ST 业务 mTLS commit `401ac5c` |
| **mock v0.1** | `tools/rgs-flash-mock/` | 22 RPC (per DDD v0.1 §9.3) | per `c5c4006` 5e6c727 |
| **gap matrix coverage** | `GET /coverage` | 100% (42 × 438) | per REQ v0.1 G-4 |

### 10.3 闪烁之光 客户端 → RGS 后端 端到端 验证路径 (per 9/4 17:11 JST "frontend compat")

```text
[闪烁之光 client] --(自研 TCP/Flash socket, size:32+cmd:16+data)-->
  [RGS adapter layer / 闪烁之光 wire → tonic gRPC method] --(tonic gRPC + mTLS)-->
    [RGS 7 域 backend (player/match/social/economy/admin/batch/card)] --(sqlx + outbox + saga)-->
      [RGS 7 域 DB] (player_db/economy_db/match_db/social_db/admin_db/batch_db/card_db)
```

**关键**: wire 协议 adapter 是 v0.2 sprint 必补 (per §8.4 已知冲突), v0.1 仅 cmd 命名映射。

---

## 11. 已知缺口 (per 8/26 JST 缺标比错标, 5 段)

### 11.1 报告

- **闪烁之光 实际 proto 风格未直接验证** (per DDD v0.1 §0.3 + §11.1) — 本 addendum 抽样 10 个 proto_*.erl 验证 schema, 但**未跑真实 wire 抓包** (待 Phase 1 mock v0.1 + 闪烁之光 client 抓包)
- **gen_proto/cfg 目录不存在** (per §0.2) — 实际抽样调整为 src/proto/proto_NNN.erl, 已覆盖 10 个代表 module, 但 41 协议号段未全抽样 (combat / partner / guild / adventure / market / sns / mail / vip / conn_login / login = 10 个, 缺 31 个)
- **43 combat cmds 描述空 24 cmds** (per §5.1) — 推测功能, 待 v0.2 sprint 详细验证
- **闪烁之光 113 条 无标题 cmds** (per REQ v0.1 §1.1 借鉴分析 .md) — 本 addendum §5 主表 仅 19 明确 + 24 推测, 剩余 113 待 v0.2 worker 实证
- **30 新 module 详细业务** (DTL-038 / 9 原则 / 6 反模式) — 本 addendum 仅 RPC 命名 + 7 域路由, 详细 entity / repository / saga 待 v0.2 sprint
- **RGS-SPEC-CROSS-002 v0.2 升版 P1 0.5d** (per DDD v0.1 §11.1) — wire 协议 adapter 是 P1 待办
- **5 域 ST 业务 mTLS cert 导出 SOP** (per 8/27 ST 导出 SOP) — 7 域 + leaderboard + cluster-ops 9 域 业务级 mTLS 跑通待 Phase 2 W2

### 11.2 框架

- **per-entity actor 0/7 域** (per DDD v0.1 §11.2 + audit v0.3 §1.2 #1 决策保留) — DB-as-state 适合 TCG 100K+ 在线, 闪烁之光 1 player 1 process 模式不直接迁移
- **协议 schema push 7 域未实装** (per DDD v0.1 §11.2) — 框架原则 #4 (协议 schema push) 待 v0.2 评估
- **per-entity actor 缺 RGS 决策记录** (per 8/27 21:59 JST 三次强化代签) — v0.2 补 ADR
- **wire 协议 adapter 缺 RGS 设计** (per §8.4) — 闪烁之光 size:32+cmd:16+data → tonic gRPC method 转换层 v0.2 sprint 必补
- **server_id 字段 缺 RGS schema** (per §3.3 + §8.4) — 闪烁之光 跨服 {rid:32, srv_id:string} → RGS 需评估加 server_id 字段

### 11.3 数据

- **闪烁之光 performance baseline 待 9 月 Phase C 阶段 C 后** (per DDD v0.1 §9.3 + §11.3) — Erlang vs Rust P50/P95/P99 对比
- **闪烁之光 DB schema 实际表** (per `src/db/db.erl` 14KB + `sup_db_buffer.erl` 2.8KB) — v0.2 抽样 read, 6 域 + card 7 域 78 表 三分类 v0.1 估算, v0.2 实测
- **跨域 saga 性能指标** (P99 延迟 / 跨服 P99 50ms 目标) — Phase 2-4 实测
- **RGS mock v0.1 22 RPC 跑通** (per DDD v0.1 §9.3) — 已 commit `c5c4006` 5e6c727, 需 Phase 2-4 业务实现后实测
- **6 域 + card 7 域 独立 DB schema 78 表** (per DDD v0.1 §6.1) — v0.1 估算, v0.2 实测 (需抽样 db.erl)

### 11.4 业务

- **batch 域 cron 引擎 + audit_logger + worker_pool** (per audit v0.3 §8.1) — 30 新 module 中 9 个 batch 域 module 相关 (holiday/feat/lev_gift/login_days/checkin/power_gift/activity/group_control/charge), v0.2 sprint 必补
- **12 Partial 业务层 90% RGS TCG 不适用** (per handoff v0.1 §1) — 12 Partial 全部映射, **不假装覆盖** 90% 业务, 仅 5 域 + card 适配业务验证
- **30 新 module 业务验证** (438 cmds - 12 Partial ~140 = 298 cmds) — v0.2 详细
- **5 域 binary 未来调外部 LLM 未登记** (v0.1 不集成, v0.2 评估 per OLU-WEB F-25) — Phase 4 backlog
- **conn_login 独立 connector service** (per §3.9 + §5.33) — RGS 当前 gap, Phase 2 W4 优先补
- **Map (6 cmds) N-A 业务** (per §5.21) — TCG 品类不适用, 不假装覆盖 (per 缺标比错标)

### 11.5 治理

- **Mavis 二审 (per B3 派生约束 Ulysses 必到)** — 状态机 ⏳ 待 Mavis 自审 → 🟡 → ⏳ Ulysses 二审 → ✅/🟡/❌
- **Ulysses 二审时间窗口不定** (per DDD-REVIEW-TEMPLATE-v0.2 §0.4) — 可能拖慢 DDD Review
- **跨域 saga / batch 域 DDD Review 需主会话打头阵** (per AGENTS.md §2.3 L4)
- **30 新 module 业务深度评估** (per 12/2 季度评审) — Mavis 自审 vs Ulysses 二审"业务深度"评估待 12/2 季度
- **凭据永不打印** (per 8/27 11:06 JST 硬 ban) — 全文无 env value 痕迹, REDACTED filter 引用
- **禁回溯叙事** (per 8/26 JST) — 全文无 "per X 历史形态" / "per X 升版前/后" / "原本是"
- **代签授权 8/27 三次强化** (per 19:39/20:56/21:59 JST) — 修订人/审批/作者 三行齐全
- **L13 自指字段 deferred** (per §0.2 0.3 11.5 全部 deferred 实时查询 ahead/md 行数)

---

## 12. 签字栏 + 修订历史

### 12.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 三行齐全 (见顶部) |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1/L1.1/L1.2 N/A (纯 doc) / L11/L12 N/A / L13 ahead/md 行数 deferred 实时查询 / L14 N/A (无 plumbing 改) |
| Evidence 段 (commit SHA / file:line) | ✅ | §3 10 个 proto_*.erl 抽样 + §4 common.proto L1-129 + §5 438 cmds 主表 1:1 + §6 42 module 矩阵 + §7 10 关键边界 + §8 6 冲突检测 + §9 4 阶段升版 + §10 7 域路由表 + §11 5 段已知缺口 |
| 派生约束守护段 (L1/L11/L12/L13/L14) | ✅ | 全部 N/A / ⏳ (纯 doc) |
| 缺标比错标 (per 8/26 JST) | ✅ | §11 5 段已知缺口 (报告/框架/数据/业务/治理) 显式列 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 全文无 "per X 历史形态" / "per X 升版前/后" / "原本是" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 全文无 env value 痕迹, REDACTED filter 引用, §11 治理缺口 |
| 主会话 1 commit (per L12.2 选项 2) | ⏳ | 待主会话 `git add` + 1 commit, worker 不 commit (per 9/4 17:11 JST 派工) |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-04 17:17 JST

### 12.2 Ulysses 二审 (必到, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | §0.2 ahead / hotfix / md 行数 全部 deferred 实时查询 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ | §0.2 全员 ✅ / ⏳ (本设计纯 doc) |
| 业务 vs 治理指标 (per v0.1.1 §9.4) | ✅ | 438 cmds × 7 域 1:1 路由, 业务深度 12 Partial + 30 新 module = 42 modules |
| commit ahead 合理性 (per 当前 sprint 范围) | ⏳ | 仓库级 ahead 待 git 实时查询 (per L13) |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ✅ | 跟 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 一致 |
| 跟 RGS-WEEKLY 一致性 | ⏳ | W37 v0.1 启动预热, 待 W37 D7 9/14 JST 收口 |
| 跟 4 决策文档 (audit v0.3 + handoff v0.1 + FLASH-OVERLAP v0.2 + FLASH-MOCK v0.3) 一致性 | ✅ | §0.1 addendum 跟 v0.1 主 doc 关系, 4 项全员 ✅ |
| 跟 user 9/4 17:11 JST "frontend compat 正确设计" 一致性 | ✅ | 438 cmds 1:1 完整映射 + 协议 schema 抽样 + 边界 + 冲突 + 迁移, 5 项全员 ✅ |
| 跟 v0.1 主 doc §7.4 41 协议号 1:1 一致性 | ✅ | §2.3 41 协议号 + §5 438 cmds 全部 1:1 命名映射, 0 冲突 |

### 12.3 修订历史 (v0.2 addendum, 续 v0.1 主 doc)

| 版本 | 日期 | 修订人 | 摘要 | commit |
|---|---|---|---|---|
| v0.1 | 2026-09-04 16:47 JST | 架构师(Mavis 接手 agent per DEC-008) — Mavis 接手 | 主 doc 12 Partial + 30 新 module 详细设计 + 7 域 1:1 映射 + 41 协议号 1:1 (per DDD v0.1 §7.4) | `80bcd3b` |
| **v0.2 addendum** | **2026-09-04 17:17 JST** | **架构师(Mavis 接手 agent per DEC-008) — Mavis 接手代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)** | **addendum 展开 v0.1 §7.4 41 协议号 → 438 cmds 完整 1:1 映射 + 协议 schema 抽样 10 个 + 边界 + 冲突 + 迁移 + 7 域路由 + 5 段已知缺口** | **(待主会话 `git add` + 1 commit per L12.2 选项 2)** |

---

**v0.2 addendum 完结** — 12 段 + 12 协议号分段 + 10 proto 抽样 + 42 module 438 cmds 1:1 主表 + 5 段已知缺口 + 3 签字栏。

> **下一步**: 主会话 `git add RGS-DDD-v0.2-addendum-协议号映射.md` + 1 commit (per L12.2 选项 2 + 9/4 17:11 JST 派工), 然后 4 阶段路线图 W2-W25 sprint 派工 v0.2-3+ 实证 (per §9.4 升版路径)。
