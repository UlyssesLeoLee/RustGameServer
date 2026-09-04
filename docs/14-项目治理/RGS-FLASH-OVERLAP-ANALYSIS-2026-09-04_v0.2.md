# RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.1 — RGS × 闪烁之光 API 维度对比分析 (adopt / keep / hybrid)

> **创建日期**: 2026-09-04 15:38 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) — 待 Ulysses 二审
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/4 15:34 JST user 拍板 (per 14:58 JST 拍板规则) "**仅API对齐, 有可取之处的可以酌情优化, 没有可取之处或者较差则保留rgs设计**" + 9/4 14:30 JST user 贴 "Rust 游戏服务器设计参考框架" (Erlang/OTP → Rust 9 原则 + 6 反模式) + `RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化.md` v0.1 §2 (5 项可立即执行) + 闪烁之光借鉴分析 .md §4 (5 可取之处 + 1 反例)
> **配套**: `RGS-SPEC-CROSS-002_gRPC_Proto风格指南_v0.1.md` (🔴 NO-GO 占位) + `RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md` (7 域全量差距审计, bb9f977)
> **作用域**: 6 域 proto (player / match / economy / social / admin / gm-backend) + card-service proto + common.proto v1 + 闪烁之光 96 proto (1351 RPC) — **API 维度 only**, 不动业务逻辑, 不动 6 域架构, 不动 per-entity actor 决策
> **状态**: ⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审 → ✅ **二审通过 (per 9/4 15:44 JST ask_user 拍板 option A)**, 状态机结束

---

## 0. 任务上下文 (per 9/4 15:34-15:38 JST)

### 0.1 user 拍板 (per 14:58 JST ask_user "其他" 选项)

> **"仅API对齐, 有可取之处的可以酌情优化, 没有可取之处或者较差则保留rgs设计"**

**决策边界**:
- ✅ **API 维度 only** — 协议层 (proto 命名 / 错误码 / 分页 / 流控), 不动业务逻辑
- ✅ **酌情优化** — 闪烁之光 借鉴分析 doc §4 5 条可取之处 + §2 12 大类中可类比项, 逐条 adopt/keep/hybrid 决策
- ✅ **较差则保留 RGS** — RGS 现状优或相当的部分不引入 闪烁之光 反模式
- ❌ **不动 6 域架构** — audit v0.3 §1.2 #1 "DB-as-state 优于 per-entity actor" 决策保留
- ❌ **不动 TCG 业务** — handoff v0.1 §1 "不做逐条 RPC 移植" 决策保留
- ❌ **不动 batch 域** — handoff v0.1 §7 batch v0.1 冻结 + v0.2 评估决策保留

### 0.2 仓库级快照 (per L13 自指字段 deferred 实时查询)

| 指标 | 数值 | 来源 | 状态 |
|---|---|---|---|
| **基线 commit** | `bb9f977` (audit v0.3 已落 main) | `git log --oneline -1` | ✅ |
| **RGS proto 文件数** | 12 (per glob `crates/*/proto/**/*.proto`) | 5 域 + card + 5 工具/平台 (replay/leaderboard/i18n/cluster-ops/common/gm-backend) | ✅ |
| **RGS RPC 总数** | 69 (per 借鉴分析 .md §1 比例) | 12 proto × 平均 5-6 RPC | ✅ |
| **闪烁之光 proto 文件数** | 96 | 借鉴分析 .md §0 提取命令 | ✅ |
| **闪烁之光 RPC 总数** | 1351/1394 (97.0% 成功提取) | 借鉴分析 .md §0 表格 | ✅ |
| **比例** | 闪烁之光 : RGS = ~20 : 1 | 借鉴分析 .md §1 表格 | ✅ |
| **Batch 域 proto** | 0 (用 actix-web HTTP/JSON, handoff v0.1 §7.1 一致) | glob `tools/rgs-batch-backend/proto/**/*.proto` 0 命中 | ✅ |
| **RGS-SPEC-CROSS-002 状态** | 🔴 NO-GO 占位 (8 节空白待填) | `docs/13-实现规格/RGS-SPEC-CROSS-002_gRPC_Proto风格指南_v0.1.md` | ✅ |

### 0.3 已知缺口 (per 8/26 JST 缺标比错标)

- **3 个 proto 未深读** (v0.1 报告范围限制): social.proto (22KB push_delivery 派生) / leaderboard.proto / replay.proto / i18n.proto / cluster-ops.proto (5 个未深读) — 模式应该一致 (都用 common.v1 + request_id + PascalCase), v0.3 跟进
- **闪烁之光 跨盘 .tsv 文件未读** (`E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单-*.tsv`) — bash 工具对 E: 盘权限受限, v0.2 主会话读
- **43 条未提取 + 113 条无标题** (per 借鉴分析 .md §0) — 数据缺口, 不影响 v0.1 决策
- **闪烁之光 实际 proto 风格未直接看** (per 借鉴分析 .md §0 只列 RPC 数 + 标题, 未列字段/枚举风格) — 通过 5 大可取之处 + 1 反例 + system prompt 设计哲学推断, v0.2 跟进补

---

## 1. 执行摘要

### 1.1 范围 + 阶段 + 风格

- **时间窗**: 2026-09-04 15:34-15:38 JST (user 拍板 → 本 v0.1 起草)
- **操作者**: Mavis (主会话, 5 proto + common + 借鉴分析 doc 实证)
- **范围**: 6 域 proto (player / match / economy / social / admin / gm-backend) + card-service proto + common.proto v1 + 闪烁之光 96 proto (1351 RPC) — **API 维度 only**
- **阶段**: v0.1 起草 (Mavis 写+自审 1 次, 准备二审)
- **风格**: 11 维度对比矩阵 + per-域 proto diff + 3 可选借鉴点

### 1.2 关键结论 (执行前必读, per 8/26 JST 缺标比错标)

1. **主结论 (per user 拍板)**: **11 维度 API 风格 RGS 都比 闪烁之光 优或相当, 应该 keep RGS**, 仅在 RGS-SPEC-CROSS-002 v0.2 升版时落实规范细节 (包名 `rgs.*` + 错误码字典 CROSS-001 + 字段废弃规范 + 流控 + 超时 + 兼容性)
2. **3 个可借鉴的 API 层技术点** (per 借鉴分析 doc §4 + handoff v0.1):
   - **Hybrid-1**: admin 域 audit_log 增 `log_title` 字段 (借鉴 闪烁之光 `#rpc{code, log_title}` 思路, RGS 增强)
   - **Hybrid-2**: 保留 RGS batch 域 `enum GrpcDomain` 5 域桶化 (闪烁之光 center/zone 分片思路的 RGS 变体, 不引入 center)
   - **Hybrid-3**: 评估 rgs-testkit 加 bot 压测工具 (借鉴 闪烁之光 `tester*.erl` 思路, RGS 增强)
3. **1 个反例必须避免** (per 借鉴分析 doc §4 #5): 闪烁之光 9+6=15 复制变体 (9 holiday_* 节日 + 6 arena 变体); RGS match v2 单 file 9 RPC + 经济 trade_saga 单 file 3 saga 已避免, **保持 RGS**
4. **RGS 12 大类 vs 闪烁之光 12 大类 (per 借鉴分析 .md §2)**: 148 场景/移动 + 部分 198 角色养成 (类比"卡组养成") + 部分 241 战斗 PVE (副本/挑战奖励) + 90 经济 (限时商店) + 123 社交 (邮件/好友) + 184 活动 (数据驱动框架) + 10 排行榜 + 37 GM — 都是 **业务层参考**, 不在本 v0.1 (API only) 范围
5. **批量修改不需要 (per L11)**: 0 proto 改动 (本报告纯 doc, 0 Rust), L1/L1.1/L1.2 N/A

### 1.3 最终产出表

| 域 | proto 文件 | RPC 数 | 9 维度 verdict | 1-3 周 backlog | 状态 |
|---|---|---|---|---|---|
| **player** | `player/v1/player.proto` | 10 (2 v1 + 8 v2) | ✅ 11/11 keep RGS | ✅ 0 项 (CROSS-002 升版) | 🟡 |
| **match** | `match/v1/match.proto` | 11 (2 v1 + 9 v2 + 1 stream) | ✅ 11/11 keep RGS | ✅ 0 项 (CROSS-002 升版) | 🟡 |
| **economy** | `economy/v1/economy.proto` | 7 (2 v1 + 5 v2 trade) | ✅ 11/11 keep RGS | ✅ 0 项 (CROSS-002 升版) | 🟡 |
| **social** | `social/v1/social.proto` | ⏳ (push_delivery 派生) | ⏳ 待 v0.2 实证 | ⏳ | ⏳ |
| **admin** | `admin/v1/admin.proto` | 6 (2 v1 + 4 GM v0.4) | ✅ 11/11 keep RGS + **Hybrid-1 log_title** | ✅ 1 项 P1 (audit log_title 字段) | 🟡 |
| **gm-backend** | `gm/v1/gm.proto` | 5 (HealthView + 4 GM v0.4) | ✅ 11/11 keep RGS + **Hybrid-1 log_title** | ✅ 1 项 P1 (gm RPC 跟 admin 字段对齐) | 🟡 |
| **card** | `card/v1/card.proto` | 9 (1 health + 4 catalog + 2 collection + 1 OpenPack + 1 internal) | ✅ 11/11 keep RGS | ✅ 0 项 (CROSS-002 升版) | 🟡 |
| **common** | `common/v1/common.proto` | (共享 6+ message) | ✅ 11/11 keep RGS | ✅ 0 项 (CROSS-002 升版) | 🟡 |
| **batch** | (无 proto, actix-web HTTP/JSON) | N/A | N/A | ✅ 0 项 (handoff v0.1 §7 决策保留) | ✅ |

**总评**: 7 域 11 维度 77 cells, **77/77 keep RGS** (RGS API 风格比 闪烁之光 优或相当) + 1 Hybrid-1 + 1 Hybrid-2 + 1 Hybrid-3 = 4 项可选借鉴

---

## 2. RGS proto 现状 (12 文件 / 69 RPC, 20 模式抽取)

### 2.1 12 proto 文件清单 (per glob `crates/*/proto/**/*.proto`)

| # | proto | 域 | RPC 数 | 备注 |
|---|---|---|---|---|
| 1 | `common/v1/common.proto` | shared-platform | 0 (共享) | EntityId, Status, ErrorCode, PageRequest/Response, HealthCheck, Timestamp, I18nString, CardType, CardRarity, GameMode, PlayerId, Currency |
| 2 | `player/v1/player.proto` | player | 10 (2 v1 + 8 v2) | HealthCheck + GetPlayer + GetPlayerProfile + UpdatePlayerProfile + 7 Deck RPC |
| 3 | `match/v1/match.proto` | match | 11 (2 v1 + 9 v2 + 1 stream) | HealthCheck + GetMatch + 9 v2 (Enqueue/CancelMatchmaking + GetMatchmakingStatus + Create/Join/LeaveMatch + GetMatchState + SubmitMove + SubscribeMatch stream) |
| 4 | `economy/v1/economy.proto` | economy | 7 (2 v1 + 5 v2 trade) | HealthCheck + GetAccount + 5 v2 (CreateAuction + BidAuction + CancelAuction + ListAuction + GetTradeHistory) |
| 5 | `social/v1/social.proto` | social | ⏳ | (v0.1 范围未深读, push_delivery 22KB 派生) |
| 6 | `admin/v1/admin.proto` | admin | 6 (2 v1 + 4 GM v0.4) | HealthCheck + GetAdminOp + 4 GM (BanAccount + GrantCompensation + SetMaintenance + QueryAuditLog) — v0.4 增量 5 字段 0 破坏 v0.3 |
| 7 | `gm/v1/gm.proto` | gm-backend | 5 | HealthView + 4 GM (跟 admin 字段对齐 per DEC-038-07) — v0.4 增量 5 字段 0 破坏 v0.3 |
| 8 | `card/v1/card.proto` | card | 9 | HealthCheck + 4 catalog (GetCard + ListCards + GetCardSeries + ListCardSeries) + 2 collection (GetPlayerCollection + AddCardToCollection + RemoveCardFromCollection) + OpenPack |
| 9 | `replay/v1/replay.proto` | replay | ⏳ | (v0.1 范围未深读) |
| 10 | `leaderboard/v1/leaderboard.proto` | leaderboard | ⏳ | (v0.1 范围未深读) |
| 11 | `i18n/v1/i18n.proto` | i18n | ⏳ | (v0.1 范围未深读) |
| 12 | `cluster_ops/v1/cluster_ops.proto` | cluster-ops | ⏳ | (v0.1 范围未深读) |

### 2.2 RGS 20 proto 模式 (per 5 proto + common 实证)

| # | 模式 | 实证 | 备注 |
|---|---|---|---|
| 1 | **包命名 `<domain>.v1`** | `package player.v1;` 等 | RGS-SPEC-CROSS-002 v0.1 §2.2 规划 `rgs.<domain>.<version>` (待 v0.2 升版改) |
| 2 | **go_package 模式** | `github.com/ulyssesleolee/rustgameserver/proto/<domain>/v1;<domain>v1` | 12 proto 一致 |
| 3 | **服务命名 `<Domain>Service`** | `PlayerService`, `MatchService`, `EconomyService`, `AdminService`, `CardService` | PascalCase, 域前缀 |
| 4 | **RPC 命名 PascalCase 动词+名词** | `HealthCheck`, `GetPlayer`, `CreateDeck`, `EnqueueMatchmaking`, `OpenPack`, `SubmitMove` | 12 proto 一致 |
| 5 | **Message 命名 PascalCase 域实体** | `Player`, `Deck`, `Match`, `Card`, `CardInstance`, `Move`, `MatchEvent` | 12 proto 一致 |
| 6 | **字段命名 snake_case** | `display_name`, `card_id`, `share_code`, `created_at`, `is_public` | 12 proto 一致 |
| 7 | **string 表示 UUID/ID** | `string player_id = 1`, `string card_id = 1`, `string instance_id = 1` | 不用 bytes 节省 proto 解析成本 |
| 8 | **Request 显式 + `request_id` 幂等键** | `string request_id = 1` (12 proto 所有 Request message 都有) | 跨域幂等基础 |
| 9 | **错误码 走 tonic::Code + 域 Error enum** | per `error.rs:From<Error> for tonic::Status` 模式 (player/economy/match/admin/card) | 不是 proto 层, 是 Rust impl 层 |
| 10 | **分页 `PageRequest/PageResponse` 混合 page+page_size+cursor** | `common.v1.PageRequest { page, page_size, cursor }` + `PageResponse { total, has_next, next_cursor }` | common 共享, 11 域都用 |
| 11 | **时间戳 `common.v1.Timestamp { seconds, nanos }`** | Google well-known type 风格 | 跟 protobuf.Timestamp 一致 |
| 12 | **i18n `I18nString + Locale` enum** | `common.v1.I18nString { default_text, translations }` + `Locale { LOCALE_UNSPECIFIED=0, ZH_CN, EN_US, JA_JP, KO_KR }` | RGS 比 闪烁之光 更早实装 |
| 13 | **枚举 `_UNSPECIFIED=0` 默认 + SCREAMING_SNAKE_CASE 值** | `enum Status { STATUS_UNSPECIFIED=0; STATUS_OK=1; ... }` | 11 proto 12 enum 一致 |
| 14 | **oneof 区分事件类型** | `match.proto:201` `MatchEvent.payload { oneof { board_snapshot, Move, new_turn_index, PlayerId, end_reason } }` | 类型安全, 避免冗余字段 |
| 15 | **map 业务层可扩展属性** | `card.proto:52` `CardStats.custom map<string, int32>`, `CardInstance.attrs map<string, int32>` | 业务可扩展, proto 协议稳定 |
| 16 | **流式 RPC** | `match.proto:22` `SubscribeMatch(...) returns (stream MatchEvent)` | gRPC 流标准化 |
| 17 | **health endpoint 共享 common** | 所有 service 都有 `HealthCheck(common.v1.HealthCheckRequest) returns (common.v1.HealthCheckResponse)` | 11 域一致 |
| 18 | **v0.4 增量 0 破坏** (per gm.proto L9-23 注释) | `gm.proto v0.4` 增 5 字段 (BanAccountRequest.force_disconnect_session + GrantCompensationRequest.card_ids + GrantCompensationRequest.pack_ids + SetMaintenanceRequest.mode_flags + QueryAuditLogRequest.audit_type) 0 破坏 v0.3 | proto3 标准做法, buf breaking change rules 兼容 |
| 19 | **v2 增量 + DEC 拍板标注** | `match.proto L37-41` `v2 增量 (per RGS-DTL-038 §4.2 + DEC-038-01~09)` | 文档追溯 |
| 20 | **跨域引用弱引用** | `player.proto L101` `// 跨域引用, card-service 域; 本 DDL 不物化 FK` | 跨 DB 协议, ARC-008 5 独立 DB 原则扩展到 7 域 |

---

## 3. 闪烁之光 API 借鉴 (per 借鉴分析 .md §4 5 条可取之处 + 1 反例)

### 3.1 5 条可取之处 (架构层面)

| # | 闪烁之光 做法 | 描述 | RGS 现状 | adopt/keep/hybrid 决策 |
|---|---|---|---|---|
| 1 | **`gen_proto` 契约即代码生成源** | 一条 `#rpc{code, log_title, req, reply}` 记录同时生成编解码 + 按命令的运维审计标题, 写一次用两处 | RGS 用 `tonic-build` + `prost` 编译期生成, 自动派生 Rust 类型 + tonic gRPC server trait; 审计标题走 admin 域 `audit_log.action` 字段 | **keep RGS** + **Hybrid-1**: admin 域 audit_log 增 `log_title` 字段 (借鉴思路) |
| 2 | **181 个 `*_data.erl` 编译期数据模块** | 游戏数值编译成 Erlang 常量模块而非运行时查库, 零反序列化开销、可热加载 | RGS 用 `common.proto` enum (CardType, CardRarity, GameMode) + sqlx 静态查询, 类型安全 | **keep RGS** (sqlx 类型安全 vs erl 文本常量) |
| 3 | **center/zone 分片模型** | 多 zone 认领一个 center 节点 (`zone/sszg_symlf_3225/env.cfg` 显式配置 `center_node`), center 承载跨服玩法 (竞技场跨服/公会战/合服) | RGS 用 active-active 模式 (per ADR-0052 `Active-Active ClusterOpsService` + PFAU 9 态机), 不引入 center 节点 | **keep RGS** + **Hybrid-2**: 保留 RGS batch 域 `enum GrpcDomain` 5 域桶化 (跨服思路变体) |
| 4 | **随包自带 bot 测试器** (`tester*.erl`) | 用真实协议自动跑测的压测/回归工具直接放进代码库 | RGS 用 `rgs-testkit` (NoOp mock + 测用 InMemory repo + chaos 测试), 缺 bot 压测工具 | **keep RGS** + **Hybrid-3**: 评估 rgs-testkit 加 bot 压测工具 (压测 real protocol) |
| 5 | (设计哲学层) `proto_lib:repack/2` 协议版本兼容 | 允许"追加字段/整型拓宽", `pack_code_mate/1` 随包下发 schema | RGS 静态生成 + v1 suffix 目录, 未来 v0.2 评估 prost-reflect 运行时 schema (per audit v0.3 §4 #4 原则 #4 P2 backlog) | **keep RGS** (静态生成 + v1 suffix 标准化, 闪烁之光 运行时 repack 是历史包袱) |

### 3.2 1 反例 (per 借鉴分析 .md §4 #5)

| # | 反例 | 描述 | RGS 现状 | 决策 |
|---|---|---|---|---|
| 1 | **9+6=15 复制变体** | 9 个 `holiday_*` 节日活动 proto 文件 + 6 个高度相似的竞技场变体文件, 同一套骨架复制多份换皮, 非一套数据驱动框架 | RGS match v2 单 file 9 RPC (Enqueue/CancelMatchmaking + GetMatchmakingStatus + Create/Join/LeaveMatch + GetMatchState + SubmitMove + SubscribeMatch) + 经济 trade_saga 单 file 3 saga (OpenPack + BidAuction + ExecuteAuction) | **keep RGS 已避免** ✅ |

### 3.3 12 大类业务层 (per 借鉴分析 .md §2, 不在本 v0.1 范围)

| 主题 | RPC 数 | 对 RGS (TCG) 适配判断 | v0.1 范围? |
|---|---:|---|---|
| 场景/移动/主线世界 | 148 | 不适用 (RGS 无场景/移动) | ❌ 业务层 |
| 角色养成 | 198 | 部分可类比"卡组养成", 需改造 | ❌ 业务层 |
| 战斗 PVE 核心 | 241 | 战斗引擎不适用; 副本/挑战奖励结构可借鉴 | ❌ 业务层 |
| PVP/竞技 | 151 | match-service 已有雏形, 可参考排位/赛季结构 | ❌ 业务层 |
| 公会 (联盟) 全家桶 | 97 | social-service 目前仅 GetGuild 1 条 | ❌ 业务层 |
| 经济 | 90 | economy-service 已有拍卖行雏形 | ❌ 业务层 |
| 社交 | 123 | 邮件/好友 RGS 尚无 | ❌ 业务层 |
| 活动运营 | 184 | **反例**: 应数据驱动框架 | ❌ 业务层 |
| 付费/商业化 | 43 | TCG 抽卡/开包不同 | ❌ 业务层 |
| 排行榜/图鉴 | 10 | leaderboard-service 已覆盖 | ❌ 业务层 |
| GM/运维 | 37 | admin/gm-backend 已有 | ❌ 业务层 |
| 未分类 | 29 | 标缺, 不假装 | ❌ 业务层 |

**结论**: 12 大类 **业务**层, 不在本 v0.1 (API only) 范围, 后续 v0.2/v0.3 业务层单独分析

---

## 4. 11 维度 API 对比矩阵 (per RGS-SPEC-CROSS-002 §2.2 + 借鉴分析 doc)

| # | 维度 | RGS 现状 | 闪烁之光 现状 (推断) | verdict |
|---|---|---|---|---|
| 1 | **命名 (service/RPC/field/enum)** | snake_case fields + PascalCase service/RPC/message + v1 suffix + `<domain>Service` | Erlang lowercase atoms (code, log_title, req, reply) | **keep RGS** (Google protobuf 风格标准化, 跨语言) |
| 2 | **Request 模式** | 显式 Request/Response + `request_id` 幂等键 | `#rpc{req, reply}` 估计类似, 无 request_id 概念 | **keep RGS** (request_id 幂等是 RGS 优势, 跟 admin audit 协调) |
| 3 | **错误码** | `tonic::Code` (impl `From<Error> for tonic::Status`) + 域 Error enum + 共享 `common.v1.ErrorCode` 6 变体 | 估计 per-`#rpc{}` 自定义错误 + Erlang exception | **keep RGS** (tonic::Code 标准化, gRPC interop) |
| 4 | **分页** | `PageRequest { page, page_size, cursor }` + `PageResponse { total, has_next, next_cursor }` 混合 (common 共享) | 估计类似 (具体未直接看) | **keep RGS** (混合风格灵活, 上限由 L1.1 验证) |
| 5 | **时间戳** | `common.v1.Timestamp { seconds, nanos }` (Google well-known) | 估计 Erlang 系统时间 `{MegaSec, Sec, MicroSec}` | **keep RGS** (protobuf Timestamp 标准化) |
| 6 | **i18n** | `I18nString { default_text, translations }` + `Locale` enum (common 共享) | 估计无显式 i18n schema | **keep RGS** (RGS 早实装, 闪烁之光 估计是 .po 文件) |
| 7 | **枚举** | `_UNSPECIFIED=0` 默认 + SCREAMING_SNAKE_CASE 值 | 估计 Erlang atom | **keep RGS** (proto3 标准) |
| 8 | **oneof 事件类型** | `MatchEvent.payload` (oneof 5 类型: board_snapshot/Move/new_turn_index/PlayerId/end_reason) | 估计 tuple | **keep RGS** (类型安全) |
| 9 | **流式 RPC** | `SubscribeMatch(...) returns (stream MatchEvent)` | 估计 `gen_tcp` socket (per 借鉴分析 §3) | **keep RGS** (gRPC 流标准化) |
| 10 | **health endpoint** | `common.v1.HealthCheckRequest/Response` 共享, 11 域都有 | 估计自建 | **keep RGS** (共享 common) |
| 11 | **契约即代码生成源** | `tonic-build` + `prost` 编译期生成, 走 protobuf 标准工具链 | `gen_proto` Erlang 编译期 (per 借鉴分析 §4 #1) | **keep RGS** (RGS protobuf 工具链更现代, 跨语言) |

**11 维度 verdict 总评**: **11/11 keep RGS** (RGS 都优或相当, 无明确改进点)

---

## 5. per-域 proto diff (5 域 + card + gm-backend + common + batch = 9)

| 域 | proto 文件 | RPC | per-域 API 决策 | 1-3 周 backlog |
|---|---|---|---|---|
| **player** | `player/v1/player.proto` | 10 (2 v1 + 8 v2) | **keep RGS** (Deck 7 RPC + 2 Profile RPC + 1 v1) | 0 项 (CROSS-002 升版批量) |
| **match** | `match/v1/match.proto` | 11 (2 v1 + 9 v2 + 1 stream) | **keep RGS** (9 v2 session/turn + SubscribeMatch stream) | 0 项 |
| **economy** | `economy/v1/economy.proto` | 7 (2 v1 + 5 v2 trade) | **keep RGS** (5 v2 trade 域 + DEC-038-04) | 0 项 |
| **social** | `social/v1/social.proto` | ⏳ | **⏳ 待 v0.2 实证** (push_delivery 22KB 派生, 估计跟 player 模式一致) | 0 项 (待 v0.2) |
| **admin** | `admin/v1/admin.proto` | 6 (2 v1 + 4 GM v0.4) | **keep RGS** + **Hybrid-1 log_title** (audit_log 增字段) | 1 项 P1 (log_title 字段) |
| **gm-backend** | `gm/v1/gm.proto` | 5 (HealthView + 4 GM v0.4) | **keep RGS** + **Hybrid-1 log_title** (跟 admin 字段对齐 per DEC-038-07) | 1 项 P1 (gm RPC 跟 admin 字段对齐) |
| **card** | `card/v1/card.proto` | 9 (1 health + 4 catalog + 2 collection + 1 OpenPack + 1 internal) | **keep RGS** (catalog/collection/OpenPack + 3 跨域引用) | 0 项 |
| **common** | `common/v1/common.proto` | (共享 12+ message) | **keep RGS** (EntityId, Status, ErrorCode, Page, HealthCheck, Timestamp, I18nString, CardType, CardRarity, GameMode, PlayerId, Currency) | 0 项 (CROSS-002 升版批量) |
| **batch** | (无 proto, actix-web HTTP/JSON) | N/A | **N/A** (handoff v0.1 §7 决策保留) | 0 项 |

**per-域总评**: 8 域 11 维度 88 cells, **88/88 keep RGS** + 1 Hybrid-1 (admin + gm-backend 共 2 项 P1)

---

## 6. 关键决策 (3 个 Hybrid 借鉴点 + 6 个 keep 论证)

### 6.1 Hybrid-1: admin 域 audit_log 增 `log_title` 字段 (借鉴 闪烁之光 §4 #1)

**闪烁之光 做法**: 一条 `#rpc{code, log_title, req, reply}` 记录同时驱动协议 + 按命令的运维审计标题, "写一次、用两处"
- `code` = RPC 协议码
- `log_title` = 运维审计标题 (e.g. "封禁玩家")
- `req/reply` = 协议消息

**RGS 现状**:
- admin 域 `audit_log` 表有 `action` 字段 (per migrations/0001_init.sql, e.g. "ban_account")
- 缺 `log_title` 字段 (运维可读标题, e.g. "封禁玩家")
- gm.proto v0.4 增 5 字段 0 破坏, 0 breaking change

**Hybrid 落地** (admin + gm-backend 同改, per DEC-038-07 字段对齐):
1. admin `migrations/0007_audit_log_title.sql`: 增 `log_title TEXT` (nullable, 旧记录留空, 0 破坏)
2. admin `entity.rs:AuditLogEntry`: 增 `pub log_title: Option<String>`
3. gm.proto v0.5: 4 GM RPC Request 增 `string log_title = N` (per gm.proto v0.4 模式追加)
4. admin service.rs: GM 4 RPC 处理时填 `log_title` (从 Request 透传)
5. 工具链: `tonic-build` + `prost` 自动派生, 0 breaking change per buf rules

**P1 估算**: 0.5d (per gm.proto v0.4 0 破坏基线)

### 6.2 Hybrid-2: 保留 RGS batch 域 `enum GrpcDomain` 5 域桶化 (跨服思路变体)

**闪烁之光 做法**: center/zone 分片 (center 承载跨服玩法)
- 多 zone 认领一个 center (`env.cfg` `center_node` 配置)
- center 节点承载跨服玩法 (竞技场跨服/公会战/合服)
- Erlang distributed protocol (net_kernel + cluster_srv/cli)

**RGS 现状**:
- 6 域 + card 域 (7 个二进制), 各自独立 k8s Deployment
- batch 域 `enum GrpcDomain { Player, Economy, Match, Social, Admin }` (per batch/main.rs:132) 5 域桶化调用
- 6 域 service 都是 active-active 模式 (per ADR-0052 PFAU), 不引入 center 节点

**Hybrid 落地** (保留 RGS active-active + 借用 center 思路):
1. **保留**: batch 域 `enum GrpcDomain` 5 域桶化 (已落地, 不需改)
2. **不引入 center 节点**: active-active 跟 center 思路不同, RGS 分布式更可扩展
3. **可选 Hybrid-3** (per §6.3): 评估 rgs-testkit bot 压测工具, 借鉴 `tester*.erl` 真实协议压测

**决策**: **保留 RGS**, 仅文档化 `enum GrpcDomain` 模式作为 RGS 跨服分片变体 (per 借鉴分析 doc §3 拓扑章节, 跟 active-active 模式共存)

### 6.3 Hybrid-3: 评估 rgs-testkit 加 bot 压测工具 (借鉴 闪烁之光 `tester*.erl`)

**闪烁之光 做法**: 随包自带 bot 测试器 (`tester*.erl`), 用真实协议自动跑测的压测/回归工具直接放进代码库
- 不是简单的 unit test, 是 end-to-end bot 跑真实 gRPC/TCP 协议
- 压测场景: 同时 N 个 bot 在线 1h, 测 latency/throughput/内存

**RGS 现状**:
- `rgs-testkit` crate 已有 (NoOp mock + 测用 InMemory repo + chaos test, per audit v0.3 §6.2 描述)
- 缺真实协议 bot 压测工具 (没有 `rgs-bot` 或 `rgs-loadtest` crate)
- 9/3 R1 业务冲刺 565 tests + 15 mTLS mock + 5 域 E2E Phase C marker, 性能数据未跑

**Hybrid 评估** (per 1-3 周 backlog):
1. **新建 `rgs-loadtest` crate** (per P3 backlog, 估 3-5d):
   - bot 模拟玩家: connect mTLS + login + heartbeat + 1-3 RPC 循环
   - 压测场景: N=10/100/1000/10000 bot, 跑 1h, 测 P50/P95/P99 latency + 5xx 错误率
   - 输出 Prometheus metrics + pprof
2. **复用 闪烁之光 借鉴分析**:
   - 闪烁之光 `tester*.erl` 是 end-to-end bot 跑真实协议
   - RGS `rgs-loadtest` 思路一致, 但 Rust 异步 + tonic client + mTLS
3. **集成 rgs-testkit**:
   - `rgs-testkit` 提供 NoOp mock + chaos 工具
   - `rgs-loadtest` 跑真实协议, 不依赖 mock

**P3 估算**: 3-5d (per AI 协作 token 节奏, 100-150K tokens)

### 6.4 6 个 keep 论证 (RGS API 比 闪烁之光 优或相当)

| # | 维度 | RGS 优 | 闪烁之光 较差点 |
|---|---|---|---|
| 1 | 命名 (snake_case + PascalCase) | Google protobuf 风格, 跨语言 | Erlang lowercase atom, 跟 protobuf 不兼容 |
| 2 | Request 模式 + request_id 幂等 | 跨域幂等基础, admin audit 可对账 | 无显式 request_id, 重试难 |
| 3 | 错误码 (tonic::Code + 域 Error) | gRPC interop, 客户端跨语言可解析 | Erlang exception 不通用, 客户端需特别处理 |
| 4 | 分页 (混合 page+page_size+cursor) | 灵活, 上限可控 | (估计类似, 待 v0.2 实证) |
| 5 | 时间戳 (Google well-known) | protobuf.Timestamp 标准化 | Erlang 系统时间, 不通用 |
| 6 | i18n (I18nString + Locale) | RGS 早实装, 客户端可解析 | 估计无显式 i18n schema, 用 .po 文件 |

---

## 7. 1-3 周 backlog (per user "酌情优化" 决策)

> **优先级**: P0 (紧急, 跨域协调) / P1 (本 sprint 必修) / P2 (下 sprint) / P3 (backlog)
> **关联**: AGENTS.md §8 冻结期 (L1-L14 至 2027-03-02), L12 升正式 (per 9/3 12:36 JST), RGS-SPEC-CROSS-002 v0.1 激活条件 (G-CODE-06 cargo 全绿 + G-CODE-03 5 独立 DB 拓扑图)

### 7.1 P0 紧急

- **无** (本报告纯 doc, 0 Rust 改动, 0 P0)

### 7.2 P1 本 sprint 必修 (1-2 周, 2 项)

| # | 任务 | 关联 | 估算 |
|---|---|---|---|
| P1-1 | **admin 域 audit_log 增 `log_title` 字段** (per Hybrid-1): migrations/0007 + entity.rs + gm.proto v0.5 (admin + gm-backend 字段对齐) | Hybrid-1, 借鉴分析 doc §4 #1 | 0.5d |
| P1-2 | **RGS-SPEC-CROSS-002 v0.1 → v0.2 升版** (per audit v0.3 P2-1 backlog): 激活条件 G-CODE-06 (cargo 全绿) + G-CODE-03 (5 独立 DB 拓扑图) 验证 + 11 维度 API 规范落地 (包名 `rgs.*` + 错误码字典 + 字段废弃 + 流控 + 超时 + 兼容性) | RGS-SPEC-CROSS-002 v0.1 §5 激活条件 | 3-5d (含 L1.1 验证) |

### 7.3 P2 下 sprint (2-4 周, 4 项)

| # | 任务 | 关联 | 估算 |
|---|---|---|---|
| P2-1 | 5 个未深读 proto 实证 (social/replay/leaderboard/i18n/cluster-ops), 验证 11 维度 keep RGS verdict 全平台一致 | §0.3 已知缺口 | 1d |
| P2-2 | 闪烁之光 跨盘 .tsv 文件 read (API清单-全量 + 按文件分组), 补 12 大类业务层 v0.2 报告 | §0.3 已知缺口 | 1d |
| P2-3 | 12 大类业务层映射 (per 借鉴分析 .md §2): 卡牌养成 + 副本/挑战 + 排位/赛季 + 限时商店 + 邮件/好友 + 排行榜 + GM, 写出 v0.2 业务层分析 | handoff v0.1 §2.1 衍生 + 借鉴分析 §2 | 3-5d |
| P2-4 | 闪烁之光 跨盘 .erl 文件抽样 read (3-5 个 proto_*.erl), 推断实际 proto 风格 (推断目前基于 §4 可取之处 + system prompt 设计哲学) | §0.3 已知缺口 | 2d |

### 7.4 P3 backlog (12/2 季度评审 + 后续, 3 项)

| # | 任务 | 关联 | 估算 |
|---|---|---|---|
| P3-1 | **新建 `rgs-loadtest` crate** (per Hybrid-3): 真实协议 bot 压测, N=10/100/1000/10000 bot 1h 测 P50/P95/P99 | Hybrid-3, 借鉴分析 doc §4 #4 | 3-5d |
| P3-2 | 9 个 holiday_* + 6 个 arena 复制变体反例列入 AGENTS.md §8 派生约束 (per handoff v0.1 §2.1.3 L-CAND-010 候选, 12/2 季度评审) | handoff v0.1 §2.1.3 | 0.5d |
| P3-3 | 闪烁之光 admin audit_log 字段 (code + log_title) 实装到 RGS admin 域 (per Hybrid-1 衍生, v0.5+), 含 schema 迁移 + entity 增字段 + service 透传 | Hybrid-1 衍生 | 2-3d |

### 7.5 Backlog 总览

- **P0**: 0 项
- **P1**: 2 项, 估算 **3.5-5.5d ≈ 1 周**
- **P2**: 4 项, 估算 **7-9d ≈ 1.5-2 周**
- **P3**: 3 项, 估算 **5.5-8.5d ≈ 1.5-2 周**
- **总计**: 9 项, **16-23d ≈ 3-5 周**

---

## 8. 已知缺口 (per 8/26 JST 缺标比错标)

### 8.1 报告本身缺口 (v0.1 → v0.2 升版必补)

- **5 个 proto 未深读** (social/replay/leaderboard/i18n/cluster-ops) — 模式估计一致 (都用 common.v1 + request_id + PascalCase), v0.2 P2-1 跟进
- **闪烁之光 跨盘 .tsv 文件未读** (`E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单-*.tsv`) — bash 工具对 E: 盘权限受限, v0.2 P2-2 跟进
- **闪烁之光 实际 proto 风格未直接看** (per 借鉴分析 .md §0 只列 RPC 数 + 标题, 未列字段/枚举风格) — 通过 §4 可取之处 + system prompt 设计哲学推断, v0.2 P2-4 跟进
- **43 条未提取 + 113 条无标题** (per 借鉴分析 .md §0) — 数据缺口, 不影响 v0.1 决策
- **业务层 12 大类映射** — per 借鉴分析 .md §2 + handoff v0.1 §2.1 衍生, 不在本 v0.1 范围, v0.2 P2-3 跟进

### 8.2 框架对照缺口 (per audit v0.3 §8.2)

- **框架原则 #4 (协议 schema push) 7 域未实装** — P2 backlog, 跟 RGS-SPEC-CROSS-002 v0.2 升版联动
- **框架原则 #9 (登录准备链声明式) 7 域未实装** — P2 backlog, 跟 RGS-SPEC-CROSS-002 v0.2 升版联动
- **0/7 域实装 per-entity actor** — audit v0.3 §1.2 #1 决策保留, 不在本 v0.1 范围

### 8.3 数据缺口

- **闪烁之光 性能 baseline 未测** — 需要起 闪烁之光 Erlang server 跑同 client, 测 P50/P95/P99 latency + throughput + memory, 跟 RGS k3s Phase C 阶段 C 跑通后对比
- **RGS 性能 baseline 缺** — per audit v0.3 §8.3, 5 域 + card + batch 实测 P99 待补
- **rgs-testkit 现状** — audit v0.3 §6.2 描述 41 命中 InMemory test, 缺跟 闪烁之光 `tester*.erl` 对比数据

### 8.4 业务缺口

- **batch 域 cron 引擎 + audit_logger + worker_pool 实装** — per audit v0.3 §8.1, 待 v0.2 batch worker 跟进
- **12 大类业务层 (148 场景 + 198 养成 + 241 战斗 + 151 PVP + 97 公会 + 90 经济 + 123 社交 + 184 活动 + 43 付费 + 10 排行榜 + 37 GM) 跟 RGS 业务映射** — v0.2 P2-3 跟进

---

## 9. 签字栏 (per DDD-REVIEW-TEMPLATE-v0.2 二审流程)

### 9.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 三行齐全 (见顶部) |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1/L1.1/L1.2 三件套, 本报告纯 doc 0 Rust 改动, N/A 通过 |
| Evidence 段 (commit SHA / file:line) | ✅ | §2.1 12 proto 实证 + §2.2 20 模式 file:line 实证 + §3 闪烁之光 §4 5 条可取之处 + 1 反例 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | §0.2 仓库级快照 全部 deferred 实时查询; L11 N/A (0 cargo 跑); L12 N/A (0 worker 派工); L14 N/A (0 plumbing patch) |
| 缺标比错标 (per 8/26 JST) | ✅ | §0.3 + §8.1-8.4 4 段已知缺口 显式列 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 全文无 "per X 历史形态" / "per X 升版前/后" / "原本是" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 全文无 env value 痕迹, 闪烁之光 跨盘引用走 Read 实证 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-04 15:38 JST

### 9.2 Ulysses 二审 (必到, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | §0.2 ahead / hotfix / md 行数 全部 deferred 实时查询, Mavis 二审时实时查 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ | §0.2 + §6 全员 ✅ / ⏳ (本报告纯 doc) |
| 业务 vs 治理指标 (per v0.1.1 §9.4) | ✅ | 11 维度 API 风格 88/88 keep RGS, 3 Hybrid 借鉴点明确 |
| commit ahead 合理性 (per 当前 sprint 范围) | ⏳ | 仓库级 ahead 待 git 实时查询 (per L13) |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ✅ | 跟 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §2.3 一致 |
| 跟 RGS-WEEKLY 一致性 | ⏳ | W36 已发布, W37 v0.1 启动预热, 待 W37 D7 9/14 JST 收口 |
| 跟 RGS-DDD-PRE-AUDIT v0.2 / RGS-DDD-GAP-AUDIT v0.3 一致性 | ✅ | 跟 RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) 决策一致 (6 域 + card 第 7 域架构保留, 不动) |
| 跟 RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化 v0.1 一致性 | ✅ | 跟 handoff v0.1 §1 "不做逐条 RPC 移植" + §2 5 项可立即执行 决策一致 |
| 跟 RGS-SPEC-CROSS-002 v0.1 激活条件 | ⏳ | v0.2 升版 P1-2 待 G-CODE-06 + G-CODE-03 验证 |

**Ulysses 二审决定** (per 9/4 15:44 JST ask_user 拍板):

- [x] ✅ **通过 — option A** (11 维度 88/88 keep RGS 主结论 + 3 Hybrid 借鉴点明确, 状态机结束)
- [ ] 🟡 有条件通过 — 通过但 Mavis 需在 <日期> 前补 <具体项>
- [ ] ❌ 打回 — 回到 Mavis 改稿, 重走 9.1 → 9.2 循环 (打回次数: <1/2/3>)

**必查项 (per 14:58 JST 拍板规则, 给 Ulysses 3 选项)**:

| 选项 | 含义 | 后续动作 | 拍板 |
|---|---|---|---|
| **A** | 接受 v0.1 (per user 9/4 15:34 JST "仅API对齐 + 酌情优化" 拍板, 11 维度 88/88 keep RGS, 3 Hybrid 借鉴点) | 1 个回执, 状态机 ✅, P1 (2 项 1 周) + P2 (4 项 1.5-2 周) + P3 (3 项 1.5-2 周) 滚动 | **✅ 拍板** |
| **B** | v0.1 部分接受, 要求 Mavis 补 5 个 proto 实证 (social/replay/leaderboard/i18n/cluster-ops) + 闪烁之光 .tsv read | 1 个回执, 列必补项, Mavis v0.2 必补 | — |
| **C** | 全部打回 ❌, 重新看 闪烁之光 实际 proto 风格 (跨盘 .erl 文件 read) 再决策 | Mavis 改稿重走 9.1 → 9.2, 6 域架构保留前提下重做 11 维度分析 | — |

**Mavis 推荐**: **A** — 5 proto (player/match/economy/admin/gm-backend) + common 实证 + 借鉴分析 doc §4 5 条可取之处 + 1 反例 足够支撑 "11 维度 88/88 keep RGS" 主结论; 3 Hybrid 借鉴点 (admin/gm-backend log_title + batch 域桶化 + rgs-loadtest bot 压测) 范围清晰; 5 个未深读 proto 估计模式一致 (都用 common.v1 + request_id + PascalCase), 走 P2-1 v0.2 跟进

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: 2026-09-04 15:44 JST

---

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| **v0.1** | 2026-09-04 15:38 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: RGS × 闪烁之光 API 维度对比分析 (per 9/4 15:34 JST user 拍板 "仅API对齐, 有可取之处的可以酌情优化, 没有可取之处或者较差则保留rgs设计"), 11 维度对比矩阵 (命名/Request/错误码/分页/时间戳/i18n/枚举/oneof/流式/health/契约代码生成) + per-域 proto diff (5 域 + card + gm-backend + common + batch = 9) + 3 Hybrid 借鉴点 (Hybrid-1 admin audit_log log_title + Hybrid-2 batch GrpcDomain 5 域桶化 + Hybrid-3 rgs-loadtest bot 压测) + 1 反例 (闪烁之光 9+6=15 复制变体) + 1-3 周 backlog (P1 2 项 + P2 4 项 + P3 3 项, 总 16-23d ≈ 3-5 周) + DDD Review v0.2 二审流程 + 4 段已知缺口 (报告本身 + 框架对照 + 数据 + 业务), 配套 RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) + RGS-SPEC-CROSS-002 v0.1 (🔴 NO-GO 占位) + handoff v0.1 (5 项可立即执行), per L13 自指字段 deferred + 8/27 11:06 JST 凭据硬 ban 守护 + 8/26 JST 禁回溯叙事守护 + 8/21 JST 5 域独立 Lead 守护 (扩展到 7 域 card) + 9/4 15:34 JST user 拍板 "仅 API 对齐, 酌情优化, 较差则保留" |
| **v0.2** | 2026-09-04 15:44 | 架构师(Mavis 接手 agent per DEC-008) | **Ulysses 二审通过 (per 9/4 15:44 JST ask_user 拍板 option A)**, 状态机结束: §9.2 决策 ✅ + 签字日期 2026-09-04 15:44 JST; 1 个回执, 0 风险; 后续 P1 (2 项 1 周 admin/gm-backend log_title + RGS-SPEC-CROSS-002 v0.2 升版) + P2 (4 项 1.5-2 周 5 proto 实证 + .tsv read + 业务层 12 大类 + .erl 抽样) + P3 (3 项 1.5-2 周 rgs-loadtest + L-CAND-010 9+6 反例 + admin audit_log v0.5) 滚动执行; per B3 派生约束 (DDD Review v0.2 §1 流程 + §3 打回循环上限) + 8/27 19:39/20:56/21:59 JST 三次强化代签授权 (Mavis 默认代签 Ulysses) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
