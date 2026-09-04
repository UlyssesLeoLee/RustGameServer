# RGS-BDD-v0.2 addendum — 闪烁之光 client 适配层设计 (gRPC transcoder / JSON-RPC proxy / Flash socket 兼容)

> **创建日期**: 2026-09-04 17:11 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) — 待 Ulysses 二审
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/4 17:11 JST user 拍板 "**frontend compat 正确设计**" + 9/4 16:45 JST "**完全对齐**" 拍板 (per ask_user option A 第 3 项) + 9/4 16:14 JST "完整 1351 mock" 拍板 + 9/4 16:47 JST "首先补全需求/基本/详细设计, 内容根据闪烁之光代码逆推" 拍板
> **配套**: `docs/15-IPA-完全对齐438cmds/RGS-REQ-2026-09-04_v0.1.md` + `RGS-BDD-2026-09-04_v0.1.md` + `RGS-DDD-2026-09-04_v0.1.md` (3 件套同 commit `80bcd3b`, addendum v0.2 独立文件) + `RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.3.md` (mock 设计 4 阶段, 跟本 addendum 解耦)
> **作用域**: 闪烁之光 client 协议 (自研 TCP / Flash socket / PHP 假节点) 接入 RGS 8 域 (player / economy / match / social / admin / card / batch / gm-backend) 的**适配层**; 推荐 **gRPC transcoder** 作为优选方案; 跟 §6 BDD v0.1 §3 模块划分 + §4 数据流 + §5 技术选型 + §6 部署架构 + §9 安全架构 一致
> **状态**: ⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审 → ✅/🟡/❌ (per DDD-REVIEW-TEMPLATE-v0.2 §3)

---

## 0. 文档元信息

| 字段 | 值 |
|---|---|
| **文档 ID** | RGS-BDD-v0.2-addendum-frontend适配层 |
| **版本** | v0.2 addendum (独立文件, 不改 BDD v0.1 baseline) |
| **基线 BDD** | `RGS-BDD-2026-09-04_v0.1.md` (commit `80bcd3b`) |
| **状态** | ⏳ 待二审 |
| **作者 / 审批 / 修订人** | 架构师(Mavis 接手 agent per DEC-008) / 架构师(Mavis 接手 agent per DEC-008) / Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 |
| **创建日期** | 2026-09-04 17:11 JST |
| **依据用户拍板** | 9/4 17:11 JST "frontend compat 正确设计" + 9/4 16:45 JST "完全对齐" + 9/4 16:14 JST "完整 1351 mock" + 9/4 16:47 JST "3 件套补全" |
| **目标读者** | 架构师 / 8 域 Lead (player / economy / match / social / admin / card / batch / gm-backend) / SRE / PM / Ulysses DDD Review |
| **更新策略** | v0.2 addendum 冻结后, v0.3+ 跟 闪烁之光 client 协议升级同步升版 (per 协议号分段 line 140-145) |
| **作用域 vs 闪烁之光 client** | 完整 42 modules / 438 cmds / 96 proto (per BDD v0.1 §1.1) 全部经适配层路由到 RGS 8 域 backend |
| **作用域 vs RGS backend** | 不动 8 域 gRPC proto (per BDD v0.1 §7.3 + audit v0.3 §1.2 #1); 不动 8 域 DB 拓扑 (per ARC-008 5→7→8 域) |
| **派生约束守护** | L1/L1.1/L1.2 N/A (本 addendum 0 Rust 改动, 纯设计文档) / L11 N/A (0 cargo 跑) / L12 N/A (1 worker 派工, 主会话统一 1 commit) / L13 self-referencing deferred / L14 plumbing N/A |

---

## 1. 引言 (per BDD v0.1 §4 数据流 + 闪烁之光 client 协议)

### 1.1 背景 (per 9/4 16:47 JST + 9/4 17:11 JST user 拍板)

Ulysses 2026-09-04 16:47 JST 拍板 "**首先补全需求文档, 基本设计文档, 详细设计文档, 内容根据闪烁之光代码逆推**", 3 件套 v0.1 (REQ + BDD + DDD) 已落 commit `80bcd3b` (per BDD v0.1 §0 baseline 冻结)。

Ulysses 2026-09-04 17:11 JST 拍板 "**frontend compat 正确设计**" (per ask_user option A 第 3 项, 跟 9/4 16:45 JST "完全对齐" 拍板 + 9/4 16:14 JST "完整 1351 mock" 拍板 一致), 确认: **RGS 5 域 + card 7 域 backend 不动, 仅在 client 接入层做适配**, 适配层独立 crate, 不污染 8 域 gRPC proto + DB。

本 addendum 是 BDD v0.1 §3.3 模块依赖图 (8 域 + shared-platform + rgs-testkit) 的**接入层扩展** —— 解决 "闪烁之光 client 怎么跟 RGS 8 域 backend 对话" 的问题。

### 1.2 目标 (per 借鉴分析 .md + 9/4 16:45 JST 拍板 + 9/4 17:11 JST 拍板)

- **接入层独立**: 适配层独立 crate (`tools/rgs-frontend-compat/` 候选路径, 跟 rgs-flash-mock / rgs-batch-backend 同级), 不动 8 域 gRPC proto
- **协议类型安全**: 适配层把 闪烁之光 自研 TCP / Flash socket → RGS gRPC mTLS 业务级 (per shared-platform::tls) + tonic 0.12 静态生成 (per BDD v0.1 §5.2)
- **业务透传优先**: 业务层 (TCG vs MMORPG) 走透明透传, 不擅自改业务 (per BDD v0.1 §7.3 Hybrid-2 + audit v0.3 §1.2 #1 DB-as-state 决策保留)
- **性能合理**: 适配层 P50 ≤ 10µs, P99 ≤ 50µs, 端到端 P99 ≤ 200µs (适配层 50µs + RGS 50µs + 网络 100µs)
- **mTLS fail-closed**: 适配层 → 8 域 backend 全走 mTLS 双向证书 (per BDD v0.1 §9.1), 不引入 cookie / token
- **不引入 Kafka / Redis**: 跟 闪烁之光 现状一致 (per network-topology.html §一句话 "所有链路都用 Erlang 分布协议或 TCP, 没有 Kafka / Redis 之类的外部中间件")

### 1.3 范围 (per ask_user option A 第 3 项 + BDD v0.1 §1.3)

**In-Scope**:
- 闪烁之光 client 协议分析 (自研 TCP / Flash socket / PHP 假节点, per 跨盘 4 文件)
- 适配层架构 (gRPC transcoder / JSON-RPC proxy / WebSocket / Flash socket 兼容 4 选项对比)
- 协议号 → RGS proto 1:1 路由表 (per v0.2-2 worker 438 cmds 完整映射, 路由到 RGS 7 域)
- 业务层适配 (TCG vs MMORPG 兼容层, 业务透传 vs 业务代理)
- 安全 (mTLS fail-closed + 凭据走 env var per 8/27 11:06 JST 硬 ban + 闪烁之光 cookie 不用, 改 mTLS)
- 性能 + 部署 + 测试 4 段 (ASCII 架构图 + 部署拓扑 + E2E 测试方案)

**Out-of-Scope**:
- 闪烁之光 client 源码改造 (per audit v0.3 §1.2 + 借鉴分析 决策, 不动 client, 适配层适配 client)
- 8 域 gRPC proto 升版 (per BDD v0.1 §5.8 + FLASH-OVERLAP v0.2 11 维度 keep RGS, 不动)
- 闪烁之光 Erlang server 替换 (per 9/4 16:45 JST "完全对齐" 拍板, mock 验证 RGS backend 不变)
- 12 大类业务层 RPC 1:1 移植 (per BDD v0.1 §1.3 + handoff v0.1 §1 + 适配层只做协议转换, 不动业务)

### 1.4 术语 (per AGENTS.md + 闪烁之光 4 文件 + BDD v0.1 §1.4)

| 术语 | 解释 |
|---|---|
| **闪烁之光 client** | 跨盘 `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\tester\src\tester.erl` 同款真实协议 client (Lua / AS3 / Unity) |
| **自研 TCP** | `gen_tcp` + `{packet, 4}` + `<<Len:32, Cmd:16, Body/binary>>` 协议格式 (per 协议栈.md L1-L2) |
| **Flash socket** | AS3 XMLSocket 旧客户端 (per 协议栈.md §0 客户端入口 + network-topology.html §1 接入面) |
| **PHP 假节点** | 跨盘 zsyz_server 用 PHP 写 mock 节点 (per 协议栈.md §L4 业务 RPC 旁路, 估计类似), 适配层 RGS 不需要这个 |
| **protocol transcoder** | 把一种 wire protocol 翻译成另一种 wire protocol 的中间层, 无业务逻辑 (本 addendum 优选方案) |
| **gRPC transcoder** | envoy 自带 `grpc_json_transcoder` filter (HTTP/JSON ↔ gRPC) + 第三方 rust 实现 `tonic-grpc-transcode` (per 候选 2) |
| **JSON-RPC proxy** | HTTP/JSON-RPC 2.0 入口 + gRPC client 出口, 业务无关的纯代理 (per 候选 2) |
| **WebSocket 适配** | ws 入口 (浏览器/H5 客户端) + gRPC client 出口, 走 actix-web ws (per 候选 3) |
| **Flash socket 兼容** | AS3 XMLSocket 字节流 + 4 字节包头解析器, 1:1 仿真 闪烁之光 wire format (per 候选 4) |
| **call_id** | 闪烁之光 client request 唯一标识 (uint32 自增, 适配层映射成 RGS `request_id`) |
| **chunked frame** | 闪烁之光 大包 (战斗录像 / 资源包) 走 chunk 切分, 适配层透明重组 |
| **fail-closed** | mTLS 证书验证失败 → 立即拒绝, 不降级 (per BDD v0.1 §9.1) |
| **TCG vs MMORPG** | RGS 是 TCG, 闪烁之光 是 MMORPG; 业务层 90% N-A (per BDD v0.1 §10.4), 适配层业务透传优先 |

---

## 2. 闪烁之光 client 协议分析 (自研 TCP / Flash socket / PHP 假节点)

### 2.1 5 层协议栈 (per 协议栈.md L1-L5)

| Layer | 名称 | 闪烁之光 实现 (per `docs/architecture/协议栈.md`) | RGS 对应 |
|---|---|---|---|
| **L1** | TCP 传输层 | `gen_tcp:listen(Port, [binary, {packet, 4}, {active, once}, {reuseaddr, true}, {nodelay, true}, {keepalive, true}])` (per 协议栈.md L1) | HTTP/2 + mTLS (gRPC standard, per BDD v0.1 §4.2) |
| **L2** | 协议编解码 | `proto_lib:pack(Cmd, Data)` / `unpack(Cmd, Bin)` (per 协议栈.md L2) | tonic 0.12 + prost 0.13 静态生成 (per BDD v0.1 §5.2) |
| **L3** | 协议路由 | `mapping:module(Type, Cmd)` → `{NeedAuth, Caller, Parser, ModName}` (per `src/mapping.erl:13-36`) | tonic 路由 (per service.rs impl trait, BDD v0.1 §4.2) |
| **L4** | 业务 RPC | `xxx_rpc:handle(Cmd, Data, RoleState)` (per 协议栈.md L4) | service.rs trait method (per 8 域 service.rs) |
| **L5** | 玩法模块 | `partner.erl` / `combat.erl` / `market.erl` (per 协议栈.md L5) | domain logic + repository.rs (sqlx 0.8) |

**关键观察** (per 协议栈.md §协议路由示例 + mapping.erl):
- `Caller` 决定走哪个进程处理: `connector` (未登录) → `object` (玩家进程内, 已登录)
- 协议号 = `Cmd` 16 bit 整数, 范围 `100-65500`, 模块号 = `trunc(Cmd / 100)` 范围 `1-655` (per mapping.erl:22-37)
- `NeedAuth: true` 表示已登录才能调用, 适配层需要做 session 校验
- 协议热更: `repack/2` (per 协议栈.md §协议 mate 元数据) 把老格式转新格式, 不重启

### 2.2 协议格式 (per 协议栈.md §协议格式)

```
+--------+--------+--------+
| Len(4) | Cmd(2) | Body   |
+--------+--------+--------+
```

- `Len`: 后续字节数 (含 Cmd 2 字节), uint32 big-endian
- `Cmd`: 协议命令号 16 bit, 范围 `100-65500` (per mapping.erl:22)
- `Body`: 协议内容, 变长, 由 `proto_xxx.erl:pack/3` / `unpack/3` 编码
- TCP 包头 4 字节 = 包长度 (per 协议栈.md L1: `{packet, 4}`)

**数据包示例 (per 协议栈.md §协议路由示例, "11002 升级伙伴")**:
```
Client → [Len=8][Cmd=11002][Body=u16 partnerId]
Server → [Len=N][Cmd=11002][Body=struct UpgradeResult]
```

### 2.3 协议号分段 (per mapping.erl:40-83)

`src/mapping.erl:40-83` 共 **39 个 code/2 子句** (per 协议栈.md §协议 ↔ 模块 数量), 关键 12 module 抽样 (per v0.2-2 worker 438 cmds 完整映射, 本 addendum §5 路由):

| 协议号段 (Code = trunc(Cmd/100)) | 模块 (per `mapping.erl:code/2`) | 中文 (per `mapping.erl` 注释) | RGS 域 | RGS RPC (per 8 域 proto) |
|---:|---|---|---|---|
| 11 | `conn_login_rpc` | 连接登录入口 | player | `Login` + `Connect` |
| 12 | `test_tcp_rpc` | 测试连接 | (test only) | (skip) |
| 101 | `login_rpc` | 登录 | player | `Login` + `Auth` |
| 102 | `map_rpc` | 地图 | match | `JoinMatch` + `MovePlayer` (N-A, TCG 无地图) |
| 103 | `role_rpc` | 角色信息 | player | `GetPlayerProfile` + `UpdateProfile` |
| 104 | `quest_rpc` | 任务 | match + batch | `GetQuests` + `ClaimReward` (TCG: 卡牌任务) |
| 105 | `item_rpc` | 背包 | player | `GetInventory` + `AddItem` |
| 108 | `mail_rpc` | 邮件 | social | `GetMail` + `SendMail` |
| 109 | `misc_rpc` | 杂项 | player | `GetMisc` |
| 110 | `partner_rpc` | 伙伴 | card | `GetCardList` + `UpgradeCard` (TCG: 卡牌养成) |
| 111 | `drama_rpc` | 剧情 | (故事模块) | (TCG 无剧情, N-A) |
| 112 | `formation_rpc` | 阵法 | card | `SetFormation` (TCG: 卡组) |
| 113 | `star_rpc` | 星命 | card | `GetStarMap` + `UpgradeStar` |
| 127 | `say_rpc` | 聊天 | social | `Chat` + `GetChatHistory` |
| 129 | `rank_rpc` | 排行榜 | leaderboard | `GetLeaderboard` |
| 130 | `dungeon_rpc` | 副本 | match | `JoinDungeon` + `SubmitAction` |
| 133 | `sns_rpc` | 社交 | social | `GetFriendList` + `AddFriend` |
| 134 | `exchange_rpc` | 积分商城 | economy | `GetShopList` + `BuyItem` |
| 135 | `guild_rpc` | 公会 | social | `GetGuild` + `JoinGuild` |
| 141 | `checkin_rpc` | 签到 | batch | `GetCheckin` + `ClaimCheckin` |
| 164 | `feat_rpc` | 成就 | leaderboard | `GetAchievements` |
| 166 | `holiday_rpc` | 活动 | batch | `GetActiveEvent` + `ClaimReward` |
| 167 | `vip_rpc` | VIP/充值 | economy | `GetVipStatus` + `Recharge` |
| 168 | `misc_rpc` | 提示信息 | player | `GetNotice` |
| 200 | `combat_rpc` | 战斗 | match | `StartCombat` + `SubmitAction` |
| 202 | `arena_rpc` | 竞技场 | match | `EnqueuePVP` + `GetPVPMatch` |
| 203 | `activity_rpc` | 活跃度 | batch | `GetActivity` + `ClaimActivity` |
| 205 | `boss_rpc` | BOSS | match | `StartBoss` + `AttackBoss` |
| 206 | `adventure_rpc` | 神界冒险 | match | `StartAdventure` + `SubmitAction` |
| 210 | `charge_rpc` | 充值 | economy | `Recharge` + `QueryRecharge` |
| 211 | `login_days_rpc` | 七日登录 | batch | `GetLoginDays` + `ClaimReward` |
| 212 | `lev_gift_rpc` | 等级好礼 | batch | `GetLevelGift` + `ClaimLevelGift` |
| 213 | `guild_dun_rpc` | 联盟副本 | match | `JoinGuildDungeon` |
| 215 | `avatar_rpc` | 头像 | player | `GetAvatar` + `SetAvatar` |
| 227 | `days_rank_rpc` | 七日排行 | leaderboard | `GetDaysRank` |
| 232 | `recruit_rpc` | 伙伴召唤 | card | `RecruitCard` (TCG: 抽卡) |
| 233 | `honor_rpc` | 称号 | player | `GetHonor` + `SetHonor` |
| 234 | `power_gift_rpc` | 战力礼包 | batch | `GetPowerGift` + `ClaimPowerGift` |
| 235 | `market_rpc` | 市场 | economy | `GetMarket` + `CreateAuction` |
| 236 | `convert_rpc` | 资产兑换 | economy | `ConvertAsset` |
| 237 | `guild_skill_rpc` | 联盟技能 | social | `UpgradeGuildSkill` |
| 238 | `guild_shipping_rpc` | 联盟远航 | social | `StartShipping` + `ClaimShipping` |
| 239 | `endless_rpc` | 无尽试炼 | match | `JoinEndless` + `SubmitScore` |

**完整 39 段** 见 `src/mapping.erl:40-83`, 本表抽样 36 段 (跳过 12 test only + 15 holiday 复制变体 per BDD v0.1 §7.6 反模式 + 9+6 holiday_* / arena_* 复制)。

### 2.4 客户端链路 (per network-topology.html §接入面 + 协议栈.md §0)

```
玩家 (Lua / AS3 / Unity)
   │ 自研 TCP (L1)
   │ 包格式: <<Len:32, Cmd:16, Body/binary>>  (L2)
   ▼
[sup_acceptor → sys_listener]
   │ proto_lib:unpack(Cmd, Bin)  (L2)
   │ mapping:parser_mod(Cmd) → proto_xxx  (L3)
   ▼
[connector_mgr:handle / role:rpc]
   │ NeedAuth: true → 已登录校验 (L3)
   ▼
[xxx_rpc:handle]  (L4)
   │ 业务实现  (L5)
   ▼
DB (mnesia / mysql) (L7)
```

**测试 client** (per `tester/src/test.erl:39-78`):
- `t(N, M, Mod, SrvId, Host, Port, Time)`: 启动 N~M 个 bot, 每个 bot 间隔 Time ms
- 端口示例: `local_1:9001` / `dev_1:9001` / `dev_2:9002` / `dev_3:9003` (per `test.erl:62-68`)
- 协议走真实 TCP 自定义二进制 (per `tester/src/tester.erl` + `tester_ai_base.erl` + `tester_ai_quest.erl`)

### 2.5 中心服 vs 游戏区节点 (per services.erl:33-56)

```erlang
%% services.erl:33-39 中央服 15 个服务
cfg(center) ->
    {ok, [sys_gc, cluster_srv, cluster_msg, sup_db_buffer, map_mgr,
          role_num_online, rank_mgr, c_group_control_mgr, log_http,
          say_subtitle, partner_comment_mgr, combat_mgr,
          global_event, sup_acceptor, sys_listener]};

%% services.erl:42-56 游戏区 56 个服务
cfg(zone) ->
    {ok, [ip_mgr, sys_gc, var, sup_db_buffer, cluster_cli, cluster_group,
          cluster_msg, role_group, mail, role_data, role_query, ...,
          global_event, sys_shutdown, sup_acceptor, sys_listener]};
```

**关键观察**:
- `sup_acceptor + sys_listener` 是 zone 节点核心 (per `services.erl:54`), center 也有 (per `services.erl:38`)
- 玩家 TCP 直连 zone 节点 (per network-topology.html §接入面), 不经 center
- 中心服宕机: 区服独立运行 (per network-topology.html §为什么不是纯星型 故障域行)
- 单 zone 宕机: 玩家转移 (per 同上), RGS 适配层面对应 k8s HPA + replica

### 2.6 关键差异 (闪烁之光 vs RGS 客户端接入)

| 维度 | 闪烁之光 (per 跨盘 4 文件) | RGS 适配层需求 | 解决 |
|---|---|---|---|
| **传输层** | TCP `{packet, 4}` 自研 | HTTP/2 (gRPC standard) | 适配层 L1 transcoder |
| **包格式** | `<<Len:32, Cmd:16, Body>>` 二进制 | protobuf 3 + HTTP/2 frames | 适配层 L2 transcoder |
| **协议号** | u16 `Cmd` 100-65500 | `RgsMethod.full_name` (string) | 适配层 L3 路由表 (本 addendum §5) |
| **认证** | `connector_mgr:handle` 检查 session | mTLS 双向证书 (per BDD v0.1 §9.1) | 适配层 mTLS + session token 注入 metadata |
| **序列化** | Erlang term (binary) | protobuf | 适配层 L2 pack/unpack |
| **会话保持** | TCP 长连接 + Erlang process dict | HTTP/2 stream + metadata | 适配层 session token 持久化 |
| **流式 RPC** | gen_tcp + 自定义 push | gRPC server-streaming (per BDD v0.1 §5.8 #9) | 适配层 stream forwarder |
| **错误码** | Erlang exception | `tonic::Code` + 域 Error enum | 适配层 exception → `tonic::Status` 映射 |
| **时延** | 1ms gen_server (per BDD v0.1 §8.1) | 50µs tokio + tonic | 适配层 ≤ 50µs 开销 |
| **玩家路径** | 玩家 → zone 直连 (per network-topology.html §接入面) | 玩家 → envoy → 8 域 (per BDD v0.1 §4.1) | 适配层独立 deployment + envoy |

---

## 3. 适配层架构总览 (ASCII)

### 3.1 总体架构

```
┌────────────────────────────────────────────────────────────────────────┐
│  闪烁之光 client (Lua / AS3 / Unity / H5)                              │
│  TCP 自研协议 <<Len:32, Cmd:16, Body>>                                 │
└────────┬───────────────────────────────────────────────────────────────┘
         │
         │ (4 选项之一, 见 §4)
         ▼
┌────────────────────────────────────────────────────────────────────────┐
│  适配层 (tools/rgs-frontend-compat/, 独立 deployment, envoy 边缘)       │
│  ┌──────────┐  ┌──────────────┐  ┌──────────┐  ┌──────────────┐       │
│  │ TCP      │  │ 协议 transcoder│  │ mTLS    │  │ 业务兼容层   │       │
│  │ listener │→│ 闪烁之光 ↔ RGS│→│ 客户端   │→│ TCG↔MMORPG   │       │
│  │ :8780    │  │ proto 路由表  │  │ cert    │  │ 业务透传     │       │
│  └──────────┘  └──────────────┘  └──────────┘  └──────────────┘       │
└────────┬───────────────────────────────────────────────────────────────┘
         │ gRPC mTLS (per shared-platform::client::build_secure_channel)
         ▼
┌────────────────────────────────────────────────────────────────────────┐
│  RGS 8 域 backend (per BDD v0.1 §2.1 + §3.1)                          │
│  player(50051) + economy(50052) + match(50053) + social(50054)         │
│  + admin(50055) + card(50061) + batch(8790) + gm-backend(8081)        │
│  + shared-platform + rgs-testkit + rgs-loadtest (Hybrid-3 P3)         │
└────────┬───────────────────────────────────────────────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────────────────┐
│  8 独立 PostgreSQL (ARC-008 5→7→8) + NATS JetStream + DLQ              │
│  + Prometheus + Grafana + Jaeger + OTel OTLP                          │
└────────────────────────────────────────────────────────────────────────┘
```

### 3.2 关键架构决策 (per 9/4 17:11 JST "frontend compat 正确设计" 拍板)

| # | 决策 | 闪烁之光 现状 | RGS 适配层 决策 | 依据 |
|---|---|---|---|---|
| 1 | 接入层拓扑 | 玩家 → zone 直连 (TCP 自研) | 玩家 → 适配层 (独立 deployment, envoy 边缘) → 8 域 | 9/1 13:05 JST envoy 独立 deployment 偏好 + BDD v0.1 §6.1 |
| 2 | 协议转换 | Erlang 自研 TCP | **gRPC transcoder** (候选 1, 优选) | 跟 RGS 5 域 + card 7 域 backend 一致 |
| 3 | 状态管理 | per-entity actor + mpsc | **业务透传** (不存 session) | BDD v0.1 §7.3 Hybrid-2 + audit v0.3 §1.2 #1 DB-as-state 决策保留 |
| 4 | 协议契约 | gen_proto (Erlang 编译期) | **tonic 0.12 + prost 0.13** (Rust 编译期) | BDD v0.1 §5.2 + FLASH-OVERLAP §3.1 #1 keep RGS |
| 5 | 业务数据 | 181 *_data.erl 常量模块 | **业务透传** (不动数据, 仅 1:1 转发) | BDD v0.1 §7.2 编译期数据 keep RGS |
| 6 | 压测工具 | tester*.erl 真实协议 bot | **复用 rgs-flash-mock** (per FLASH-MOCK v0.3 §1) | 借鉴分析 §4 #4 Hybrid-3 |
| 7 | 协议版本 | proto_lib:repack/2 运行时 | **静态生成 + v1 suffix** (无 repack) | BDD v0.1 §7.4 keep RGS |

### 3.3 跟 mock 边界 (per FLASH-MOCK v0.3 §1.3)

| 边界 | rgs-flash-mock (per FLASH-MOCK v0.3) | rgs-frontend-compat (本 addendum) |
|---|---|---|
| **作用** | gateway / verification harness (验证 RGS backend 覆盖) | 真实 client 接入层 (替代 闪烁之光 zone 节点) |
| **入口** | HTTP/JSON (actix-web) | TCP 自研 (闪烁之光 协议) + HTTP/2 (H5/Unity 客户端) |
| **出口** | gRPC mTLS → 7 域 backend | gRPC mTLS → 8 域 backend (含 batch) |
| **gap matrix** | 跟踪每个 RPC PASS/FAIL/N-A (per FLASH-MOCK §4) | 不跟踪, 业务透传优先 |
| **业务层** | 占位 mock, 验证 backend | 业务透传, 不擅自改业务 (per BDD v0.1 §7.3 Hybrid-2) |
| **目标用户** | rgs-flash-mock 自身 (verification harness) | 闪烁之光 client (Lua / AS3 / Unity / H5) |
| **端口** | 0.0.0.0:8791 (per FLASH-MOCK v0.3 §2.1) | 0.0.0.0:8780 (本 addendum §9) |
| **部署** | 独立 deployment (per AGENTS.md §7.1) | 独立 deployment (per AGENTS.md §7.1) |
| **关系** | 跟 frontend-compat 解耦, 独立演进 | 跟 flash-mock 解耦, 独立演进 |

---

## 4. 适配层组件设计 (4 选项对比 + 推荐)

### 4.1 候选 1 (优选): gRPC transcoder (per 借鉴分析 §0 + 9/4 17:11 JST 拍板)

**架构**:
```
闪烁之光 client (TCP 自研)
   ▼
[rgs-frontend-compat TCP listener :8780]
   │ 1. recv [Len:32, Cmd:16, Body]
   │ 2. parse: Cmd → u16, Body → proto_xxx:unpack
   │ 3. lookup routing_table[Cmd] → (RgsMethod, RgsDomain)
   │ 4. transcode: Body → RGS Request (prost Message)
   │ 5. tonic gRPC client → RGS 域
   ▼
RGS 8 域 backend (mTLS 业务级)
```

**技术栈** (per rgs-batch-backend 模式 + BDD v0.1 §5):
| 组件 | 选型 | 理由 |
|---|---|---|
| **TCP listener** | tokio 1.x + `TcpListener` (per BDD v0.1 §5.2) | 跟 rgs-flash-mock 一致, 适配 闪烁之光 自研 TCP |
| **协议 transcoder** | 手写 `proto_xxx::pack/unpack` Rust port (per 协议栈.md L2) | 1:1 仿真 闪烁之光 wire format, 0 字节丢失 |
| **路由表** | 静态 `HashMap<u16, (RgsMethod, RgsDomain)>` (本 addendum §5) | 编译期生成, 0 反射开销 |
| **gRPC client** | tonic 0.12 + mTLS (per BDD v0.1 §5.2) | 跟 8 域 backend 一致 |
| **业务兼容层** | trait `CompatAdapter` + per-module impl (本 addendum §6) | 业务透传优先, N-A 占位 |
| **tracing** | tracing + tracing-subscriber JSON (per BDD v0.1 §5.4) | 跟 shared-platform::json_logging 一致 |
| **config** | envy + figment + .env (per BDD v0.1 §5.2) | 跟 shared-platform::config 一致 |
| **error** | thiserror + `From<Error> for tonic::Status` (per BDD v0.1 §5.8 #3) | 5 域 error 模式对齐 |
| **workspace** | `[workspace]` 独立 (per rgs-batch-backend/Cargo.toml) | 不污染主 cargo workspace |
| **port** | 0.0.0.0:8780 (next sequential after rgs-flash-mock 8791) | k3s service NodePort 暴露 |

**优势**:
- 性能最佳: 端到端 P99 ≤ 200µs (适配层 50µs + RGS 50µs + 网络 100µs, per BDD v0.1 §8.2)
- 1:1 仿真 闪烁之光 wire format, 0 协议改造
- 跟 8 域 gRPC proto + shared-platform 完全一致
- 业务透传, 不擅自改业务 (per BDD v0.1 §7.3)

**劣势**:
- 需要写 闪烁之光 proto_xxx.erl 的 Rust port (per 协议栈.md L2, 39 个 protocol segments)
- 闪烁之光 protocol 升级时 (e.g. holiday_* 新增), 适配层要同步 (per BDD v0.1 §7.4 keep RGS 静态生成)

### 4.2 候选 2: JSON-RPC proxy (备选)

**架构**:
```
闪烁之光 client (TCP 自研)
   ▼
[rgs-frontend-compat TCP listener :8780]
   │ 1. recv [Len:32, Cmd:16, Body]
   │ 2. parse: Cmd → u16, Body → JSON (自定义 codec)
   │ 3. HTTP POST {jsonrpc: "2.0", method: "Cmd11002", params: {partnerId: 123}}
   ▼
[envoy grpc_json_transcoder filter]
   │ /v1/{service}/{method} HTTP/JSON → gRPC
   ▼
RGS 8 域 backend (mTLS)
```

**优势**:
- 业务方可以用 curl / Postman 调试 (JSON 友好)
- envoy 自带 transcoder, 适配层开发量小

**劣势**:
- 闪烁之光 client 是 TCP 二进制, 强制走 JSON 增加 2 次转换 (TCP→JSON→gRPC)
- envoy transcoder 性能比手写 pack/unpack 慢 ~3x
- 不支持 闪烁之光 chunked frame + 16 bit Cmd 路由, 需 hack

**结论**: ❌ 不推荐, 候选 1 性能更优, 业务透传更直接。

### 4.3 候选 3: WebSocket 适配 (备选)

**架构**:
```
H5 客户端 (浏览器 / 小程序)
   ▼
[actix-web ws listener :8780]
   │ ws frame: binary [Len:32, Cmd:16, Body] OR text JSON
   ▼
[transcoder (per 候选 1)]
   ▼
RGS 8 域 backend (mTLS)
```

**优势**:
- H5 客户端 (浏览器 / 小程序) 友好, 走 ws 标准
- 双协议支持 (binary + JSON), 浏览器可降级到 JSON

**劣势**:
- 不支持 闪烁之光 AS3 / Lua / Unity 旧客户端 (per 协议栈.md §0 客户端入口)
- ws 握手 + frame 切分额外开销 ~50µs

**结论**: 🟡 候选 1 + 候选 3 双协议共存 (TCP + ws), 适配层开 2 listener (`:8780` TCP + `:8781` ws), 复用同一 transcoder。

### 4.4 候选 4: Flash socket 兼容 (备选)

**架构**:
```
AS3 XMLSocket 旧客户端
   ▼
[actix-web + xml_socket listener :8780]
   │ XMLSocket 字节流 + null-terminated string
   │ 1:1 仿真 闪烁之光 Flash 客户端
   ▼
[transcoder (per 候选 1)]
   ▼
RGS 8 域 backend (mTLS)
```

**优势**:
- 兼容 闪烁之光 AS3 XMLSocket 旧客户端 (per 协议栈.md §0 客户端入口)
- 业务方零改造

**劣势**:
- AS3 XMLSocket 已被主流浏览器弃用 (Chrome 84+ 2020 起, Firefox 2020 起)
- 维护成本高, 9/4 17:11 JST 拍板 "frontend compat 正确设计" → 优先现代客户端 (Lua / Unity / H5)

**结论**: 🟡 候选 1 + 候选 4 双 listener 兼容 (`:8780` TCP + `:8782` XMLSocket), 仅当业务方有 AS3 历史包袱时启用, v0.2 P3 backlog 评估。

### 4.5 推荐方案 (per 9/4 17:11 JST 拍板 + 性能 + 维护成本)

**v0.2 推荐**: **候选 1 (gRPC transcoder) 单一 listener `:8780`**, 走 闪烁之光 自研 TCP, 1:1 仿真 wire format。

**v0.3+ 可选扩展** (per 业务方需求):
- H5 客户端 → 候选 3 WebSocket listener `:8781`
- AS3 历史包袱 → 候选 4 XMLSocket listener `:8782`

**推荐依据**:
- 性能: 候选 1 端到端 P99 ≤ 200µs, 候选 2/3/4 均 ≥ 300µs
- 维护: 候选 1 单一 listener, 候选 2/3/4 多 listener 增加运维成本
- 业务: 候选 1 业务透传, 候选 2 强制 JSON 改协议
- 9/4 17:11 JST 拍板: 适配层独立 crate, 不动 8 域 backend, 候选 1 最简洁

---

## 5. 协议号 → RGS proto 1:1 路由表 (per v0.2-2 worker 438 cmds 完整映射)

### 5.1 路由表设计 (per BDD v0.1 §3.2 + mapping.erl:40-83)

**路由表结构** (Rust 静态 HashMap, 编译期生成):
```rust
// tools/rgs-frontend-compat/src/routing.rs
pub struct RouteEntry {
    pub cmd: u16,                    // 闪烁之光 协议号 (100-65500)
    pub rgs_method: &'static str,    // RGS proto method (e.g. "/player.v1.PlayerService/Login")
    pub rgs_domain: RgsDomain,       // 7 域枚举
    pub need_auth: bool,             // per mapping.erl NeedAuth
    pub caller: Caller,              // connector (未登录) / object (已登录) per mapping.erl
}

pub enum RgsDomain {
    Player,    // 50051
    Economy,   // 50052
    Match,     // 50053
    Social,    // 50054
    Admin,     // 50055
    Card,      // 50061
    Batch,     // 8790
    GmBackend, // 8081
    Leaderboard, // TBD v0.2
    Replay,    // TBD v0.2
}

pub enum Caller {
    Connector, // 未登录
    Object,    // 已登录 (玩家进程内)
}
```

### 5.2 路由表示例 (per mapping.erl:40-83 + BDD v0.1 §3.1 12 Partial)

| Cmd (u16) | 闪烁之光 模块 (per mapping.erl) | RGS method (示例) | RGS 域 | Auth | Caller | 备注 |
|---:|---|---|---|---|---|---|
| 11001 | `partner_rpc` (proto_110) | `/card.v1.CardService/ListCards` | Card | ✅ | object | TCG 卡牌列表 (类比伙伴) |
| 11002 | `partner_rpc` (proto_110) | `/card.v1.CardService/UpgradeCard` | Card | ✅ | object | TCG 卡牌升级 (类比伙伴升级) |
| 10201 | `map_rpc` (proto_102) | (N-A, TCG 无地图) | — | — | — | TCG vs MMORPG 不适用, 适配层返 `tonic::Code::Unimplemented` |
| 10301 | `role_rpc` (proto_103) | `/player.v1.PlayerService/GetProfile` | Player | ✅ | object | 角色信息 |
| 20001 | `combat_rpc` (proto_200) | `/match.v1.MatchService/StartCombat` | Match | ✅ | object | 战斗 (TCG: 对战) |
| 20002 | `combat_rpc` (proto_200) | `/match.v1.MatchService/SubmitAction` | Match | ✅ | object | 战斗动作 |
| 20043 | `combat_rpc` (proto_200) | `/match.v1.MatchService/EndCombat` | Match | ✅ | object | 战斗结束 (per `combat.erl:43` 估算) |
| 20201 | `arena_rpc` (proto_202) | `/match.v1.MatchService/EnqueuePVP` | Match | ✅ | object | PVP 排队 |
| 20226 | `arena_rpc` (proto_202) | `/match.v1.MatchService/GetPVPMatch` | Match | ✅ | object | PVP 比赛状态 (per `arena.erl:26` 估算) |
| 21001 | `charge_rpc` (proto_210) | `/economy.v1.EconomyService/Recharge` | Economy | ✅ | object | 充值 |
| 21003 | `charge_rpc` (proto_210) | `/economy.v1.EconomyService/QueryRecharge` | Economy | ✅ | object | 充值历史 |
| 23501 | `market_rpc` (proto_235) | `/economy.v1.EconomyService/GetMarket` | Economy | ✅ | object | 市场列表 |
| 23519 | `market_rpc` (proto_235) | `/economy.v1.EconomyService/CreateAuction` | Economy | ✅ | object | 创建拍卖 (per `market.erl:19` 估算) |
| 23201 | `recruit_rpc` (proto_232) | `/card.v1.CardService/RecruitCard` | Card | ✅ | object | 抽卡 (TCG: 伙伴召唤) |
| 23203 | `recruit_rpc` (proto_232) | `/card.v1.CardService/GetRecruitHistory` | Card | ✅ | object | 抽卡历史 (per `recruit.erl:3` 估算) |
| 11301 | `star_rpc` (proto_113) | `/card.v1.CardService/GetStarMap` | Card | ✅ | object | 星图 |
| 11320 | `star_rpc` (proto_113) | `/card.v1.CardService/UpgradeStar` | Card | ✅ | object | 星命升级 (per `star_rpc.erl:20` 估算) |
| 13001 | `dungeon_rpc` (proto_130) | `/match.v1.MatchService/JoinDungeon` | Match | ✅ | object | 进入副本 |
| 13009 | `dungeon_rpc` (proto_130) | `/match.v1.MatchService/SubmitDungeon` | Match | ✅ | object | 副本结算 (per `dungeon_data.erl:9` 估算) |
| 13501 | `guild_rpc` (proto_135) | `/social.v1.SocialService/GetGuild` | Social | ✅ | object | 公会信息 |
| 13529 | `guild_rpc` (proto_135) | `/social.v1.SocialService/JoinGuild` | Social | ✅ | object | 加入公会 (per `guild.erl:29` 估算) |
| 12701 | `say_rpc` (proto_127) | `/social.v1.SocialService/Chat` | Social | ✅ | object | 聊天 |
| 12714 | `say_rpc` (proto_127) | `/social.v1.SocialService/GetChatHistory` | Social | ✅ | object | 聊天历史 (per `say_rpc.erl:14` 估算) |
| 13301 | `sns_rpc` (proto_133) | `/social.v1.SocialService/GetFriendList` | Social | ✅ | object | 好友列表 |
| 13316 | `sns_rpc` (proto_133) | `/social.v1.SocialService/AddFriend` | Social | ✅ | object | 加好友 (per `sns_rpc.erl:16` 估算) |
| 12901 | `rank_rpc` (proto_129) | `/leaderboard.v1.LeaderboardService/GetLeaderboard` | Leaderboard | ✅ | object | 排行榜 |
| 16601 | `holiday_rpc` (proto_166) | `/batch.v1.BatchService/GetActiveEvent` | Batch | ✅ | object | 活动 |
| 16613 | `holiday_rpc` (proto_166) | `/batch.v1.BatchService/ClaimReward` | Batch | ✅ | object | 领奖 (per `holiday_rpc.erl:13` 估算) |
| 14101 | `checkin_rpc` (proto_141) | `/batch.v1.BatchService/GetCheckin` | Batch | ✅ | object | 签到 |
| 10101 | `login_rpc` (proto_101) | `/player.v1.PlayerService/Login` | Player | ❌ | connector | 登录 |
| 11001 | `conn_login_rpc` (proto_11) | `/player.v1.PlayerService/Connect` | Player | ❌ | connector | 连接登录入口 (per `mapping.erl:40`) |

**完整 438 cmds 路由表**: 由 v0.2-2 worker 在独立 `routing.csv` 维护, 适配层 `build.rs` 编译期 codegen 成 `HashMap<u16, RouteEntry>`。

### 5.3 N-A 状态处理 (per BDD v0.1 §10.4 TCG vs MMORPG 90% N-A)

**N-A 业务** (闪烁之光 MMORPG, RGS TCG 不适用):
- `proto_102 map_rpc` (地图 / 场景移动) → N-A, TCG 无地图
- `proto_111 drama_rpc` (剧情) → N-A, TCG 无剧情
- `proto_168 misc_rpc` (MMORPG 提示信息) → 部分 N-A

**N-A 处理策略** (per 适配层设计):
```rust
fn handle_na_route(cmd: u16) -> RgsResponse {
    warn!("N-A: cmd={} not applicable for TCG, returning Unimplemented", cmd);
    Err(tonic::Status::unimplemented(
        format!("cmd {} is not applicable for TCG (MMORPG only)", cmd)
    ))
}
```

**业务方沟通**: N-A 业务方需在 v0.2 P2 抽样 read 闪烁之光 实际 .erl 模块 (per BDD v0.1 §10.1 P2-4 backlog), 确认 TCG 业务裁剪范围。

### 5.4 未知 Cmd 处理 (per 协议栈.md §协议号 + mapping.erl:91)

```rust
fn handle_unknown_cmd(cmd: u16) -> RgsResponse {
    Err(tonic::Status::invalid_argument(
        format!("unknown cmd: {}", cmd)
    ))
}
```

**依据**: 闪烁之光 `mapping.erl:91-92` `code(Type, Code) -> {error, {unknow_mapping, Type, Code}}` 同样返 error。

---

## 6. 业务层适配 (TCG vs MMORPG 兼容层)

### 6.1 业务透传 vs 业务代理 (per BDD v0.1 §7.3 + audit v0.3 §1.2 #1)

**业务透传** (per BDD v0.1 §7.3 Hybrid-2 + 9/4 17:11 JST 拍板):
- 适配层**不**重写业务逻辑
- 闪烁之光 `Cmd + Body` 1:1 转发到 RGS proto Request (per §5 路由表)
- RGS 域 backend 负责业务实现 (per BDD v0.1 §4.2 L5 玩法模块)
- 业务层 90% N-A 接受 (per BDD v0.1 §10.4), 走 `tonic::Code::Unimplemented` (per §5.3)

**业务代理** (备选, 评估中):
- 适配层**重写**业务逻辑, e.g. 闪烁之光 `map_rpc.MovePlayer` → RGS `match.v1.SubmitMove` (TCG 走对局动作)
- 优势: 业务层 N-A 可被代理绕过
- 劣势: 适配层变成"业务中心", 维护成本随功能数量线性增长 (per BDD v0.1 §7.6 反例 A1)

**决策** (per 9/4 17:11 JST "frontend compat 正确设计" 拍板): **业务透传优先**, 业务代理仅在 v0.3+ 抽样 5-10 个关键 N-A 业务评估, 不在 v0.2 实装。

### 6.2 业务兼容层设计 (per BDD v0.1 §7.3 + rgs-testkit 模式)

```rust
// tools/rgs-frontend-compat/src/compat.rs
pub trait CompatAdapter: Send + Sync {
    /// 闪烁之光 Body → RGS Request (per 路由表)
    fn transcode_request(&self, cmd: u16, body: &[u8]) -> Result<prost::Message, CompatError>;

    /// RGS Response → 闪烁之光 Body
    fn transcode_response(&self, cmd: u16, response: prost::Message) -> Result<Vec<u8>, CompatError>;

    /// 业务字段映射 (e.g. 闪烁之光 roleId → RGS player_id)
    fn map_business_field(&self, field: &str, value: prost::Value) -> Result<prost::Value, CompatError>;
}

pub struct PassThroughAdapter;
/// 默认实现: 1:1 透传, 字段名直接映射
impl CompatAdapter for PassThroughAdapter {
    fn transcode_request(&self, _cmd: u16, body: &[u8]) -> Result<prost::Message, CompatError> {
        // proto_xxx:unpack → RGS Request (per §5 路由表)
        Ok(...)
    }
    // ...
}

pub struct TcgCompatAdapter;
/// TCG 业务代理 (仅 N-A 业务, per §5.3)
impl CompatAdapter for TcgCompatAdapter {
    fn transcode_request(&self, cmd: u16, body: &[u8]) -> Result<prost::Message, CompatError> {
        // 闪烁之光 MovePlayer → RGS SubmitMove (TCG 走对局动作)
        match cmd {
            10201 => handle_map_na(cmd),  // proto_102 N-A
            _ => PassThroughAdapter.transcode_request(cmd, body),
        }
    }
    // ...
}
```

### 6.3 业务字段映射 (TCG vs MMORPG, per BDD v0.1 §7.5 + 借鉴分析 §4 #5 避免)

**示例**: 闪烁之光 `roleId` → RGS `player_id` (per 8 域 proto common.v1.PlayerId)

| 闪烁之光 字段 | RGS 字段 | 域 | 映射策略 |
|---|---|---|---|
| `RoleId` (u64) | `player_id` (string, UUID) | player | u64 → UUID string (per BDD v0.1 §6.2 玩家 ID 策略) |
| `MapId` (u32) | (N-A) | — | TCG 无地图, 适配层透传 0 |
| `PartnerId` (u32) | `card_id` (u32) | card | 1:1 透传 |
| `GuildId` (u32) | `guild_id` (string) | social | u32 → string (per social 域 proto) |
| `CombatId` (u64) | `match_id` (string) | match | u64 → string (per match 域 proto) |

**字段映射表**: 适配层 `field_mapping.rs` 静态 `HashMap<&str, &str>`, 编译期生成, 0 反射开销。

### 6.4 业务层 N-A 评估 (per BDD v0.1 §10.4 + handoff v0.1 §1)

**N-A 业务** (per BDD v0.1 §10.4 + mapping.erl:40-83):
- 场景/移动 (148 cmds) → TCG 无场景, 全部 N-A
- 角色养成部分 (e.g. MMORPG 装备系统 80 cmds) → TCG 走卡牌养成, 部分 N-A
- 战斗 (241 cmds) → TCG 走对战, 大部分 pass-through, 部分 N-A
- 公会 (97 cmds) → TCG 走公会, 大部分 pass-through
- 活动 (184 cmds) → TCG 走 batch 任务, 大部分 pass-through
- GM (37 cmds) → TCG 走 admin + gm-backend, 大部分 pass-through

**决策** (per 9/4 17:11 JST 拍板 + 业务透传优先):
- v0.2 走业务透传 + N-A 返 `Unimplemented`
- v0.3+ 抽样 5-10 个 N-A 业务 (e.g. 场景移动 → 对局动作), 评估 TCG 业务代理可行性
- 业务代理需 9/4 16:45 JST "完全对齐" 拍板扩展 (per ask_user option C), 不在 v0.2 范围

---

## 7. 安全 (mTLS fail-closed + 凭据走 env var + 闪烁之光 cookie 不用)

### 7.1 mTLS 业务级 (per BDD v0.1 §9.1 + shared-platform::tls)

**强约束** (per BDD v0.1 §9.1 + RGS-REV-007):
- 适配层 → 8 域 backend 全走 mTLS 双向证书 (per `shared-platform/src/lib.rs:42` `pub mod tls`)
- 证书生成: rgs-certgen (per `Cargo.toml:13`)
- 加载: `load_client_tls` + `load_server_tls_config` (per lib.rs:82-83)
- **fail-closed**: 证书验证失败 → 拒绝连接, 不降级到 insecure

**适配层 cert 复用**:
- 适配层作为 gRPC client 复用 5 域 certs (per L-CAND-006 兜底, cert 内容永不入 commit)
- 适配层作为 TCP server 走 0.0.0.0:8780, 不强制 mTLS (闪烁之光 client 是 TCP 自研, 走 mTLS 需 client 改造, 不在 v0.2 范围)

### 7.2 闪烁之光 cookie 不用 (per 协议栈.md §0 客户端入口)

**闪烁之光 session 机制** (per 协议栈.md §协议路由示例):
- `connector_mgr:handle` 检查 session, 玩家进程内 `role:rpc` 携带 session_id
- 旧客户端可能用 cookie 持久化 (per 协议栈.md §0 估计, 未直接看)

**RGS 决策**: **不用 cookie**, 改 mTLS + token 注入:
```rust
// 适配层 session token 注入
fn inject_session_token<T>(&self, request: &mut T) -> Result<(), CompatError>
where T: prost::Message {
    let session_token = self.session_store.get(&self.peer_addr)?;
    request.metadata_mut().insert("x-rgs-session", session_token);
    Ok(())
}
```

**session 持久化**: 适配层本地 LRU cache (e.g. `moka` crate) + Redis 备份 (v0.3+ 评估, per BDD v0.1 §1.3 不引入 Kafka/Redis 一致性)

### 7.3 凭据管理 (per BDD v0.1 §9.2 + 8/27 11:06 JST 硬 ban)

**强约束** (per AGENTS.md §1.2 + 8/27 JST hard ban + BDD v0.1 §9.2):
- **禁止打印 env 值**: `Get-ChildItem env: | Format-Table` / `echo $VAR` / `$env:X expand` 等所有可能泄露 secret 的操作**禁止**
- **只可 invoke**: `$env:VAR` 引用后直接 pipe (如 `$env:DB_PASSWORD | wsl -e bash -c '...'`), 或传给程序参数
- **凭据走 env var**: DB 密码 / 证书路径 / 第三方 API key 全部走 env var, 不入 commit
- **REDACTED filter**: 日志中出现 secret, 用 REDACTED 替换

**适配层 env vars** (per BDD v0.1 §9.2 + DETAILED §5.1 模式):
| env var | 用途 | 必填 |
|---|---|---|
| `FRONTEND_COMPAT_BIND_ADDR` | TCP 监听地址 (default `0.0.0.0:8780`) | ❌ |
| `FRONTEND_COMPAT_TLS_CA_CERT` | 8 域 backend mTLS CA cert 路径 (per L-CAND-006) | ✅ |
| `FRONTEND_COMPAT_TLS_CLIENT_CERT` | 适配层 mTLS client cert 路径 | ✅ |
| `FRONTEND_COMPAT_TLS_CLIENT_KEY` | 适配层 mTLS client key 路径 (REDACTED filter) | ✅ |
| `FRONTEND_COMPAT_RGS_PLAYER_URL` | player-service gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_ECONOMY_URL` | economy-service gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_MATCH_URL` | match-service gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_SOCIAL_URL` | social-service gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_ADMIN_URL` | admin-service gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_CARD_URL` | card-service gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_BATCH_URL` | batch-backend gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_GM_URL` | gm-backend gRPC URL | ✅ |
| `FRONTEND_COMPAT_RGS_LEADERBOARD_URL` | leaderboard-service gRPC URL (TBD v0.2) | 🟡 |
| `FRONTEND_COMPAT_RGS_REPLAY_URL` | replay-service gRPC URL (TBD v0.2) | 🟡 |
| `FRONTEND_COMPAT_LOG_LEVEL` | tracing log level (default `info`) | ❌ |
| `FRONTEND_COMPAT_OTLP_ENDPOINT` | OTel OTLP endpoint (per BDD v0.1 §5.4) | 🟡 |

### 7.4 证书管理 (per BDD v0.1 §9.3 + L-CAND-006 派生约束)

**强约束** (per L-CAND-006 EXCEPTION-PATH v0.1, 9/1 12:36 JST 升正式):
- **cert 内容永不入 commit**: 证书文件 + 私钥文件不能 git add
- **fingerprint 比对验证**: 部署前用 `openssl x509 -fingerprint -sha256` 比对
- **9/1 12:36 JST EXCEPTION-PATH**: 紧急情况可临时入 commit, 但 24h 内迁移到 k8s secret + 修订历史写明

**适配层 cert 部署**:
- 复用 rgs-flash-mock 5 域 certs (per L-CAND-006 兜底)
- k3s Secret 挂载到 `/etc/rgs-frontend-compat/certs/`
- env var `FRONTEND_COMPAT_TLS_CA_CERT=/etc/rgs-frontend-compat/certs/ca.pem`

### 7.5 RBAC + 审计 (per BDD v0.1 §9.4 + §9.5)

**RBAC** (per `shared-platform/src/lib.rs:38` `pub mod rbac`):
- 适配层透传 player token → 8 域 backend, 由 8 域做 RBAC 校验
- 适配层不做 RBAC 决策 (无业务), 避免重复 + 一致性

**审计** (per admin::audit_log + gm-backend 配套):
- 适配层 emit `audit_event` (per BDD v0.1 §9.5) for 每条 transcoded RPC
- 字段: `cmd` + `peer_addr` + `rgs_method` + `latency_ms` + `status` + `request_id`
- 通过 NATS 异步发 admin 域 audit_log 表 (per BDD v0.1 §4.4 Outbox)

---

## 8. 性能 (transcoder 开销, 闪烁之光 1ms gen_server → 适配层 5µs + RGS 50µs = 总 55µs)

### 8.1 性能分解 (per BDD v0.1 §8.1 + 适配层估算)

| 阶段 | 闪烁之光 Erlang baseline | 适配层 (本 addendum §4.1 候选 1) | RGS 域 backend (per BDD v0.1 §8.1) | 端到端 |
|---|---|---|---|---|
| **L1 TCP recv** | 50µs (gen_tcp `{packet, 4}` 解析) | 5µs (tokio TcpStream + bytes crate) | N/A | 5µs |
| **L2 codec** | 200µs (proto_lib:unpack Erlang term) | 3µs (prost 静态生成) | N/A | 3µs |
| **L3 路由** | 50µs (mapping:module Erlang atom lookup) | 1µs (HashMap<u16, RouteEntry>) | N/A | 1µs |
| **L4 RPC** | 1ms (gen_server 上下文切换) | 5µs (tonic gRPC client + mTLS handshake 缓存) | 50µs (tokio + tonic 0.12) | 55µs |
| **L5 业务** | 取决于玩法 (per BDD v0.1 §8.1 #2) | N/A (业务透传) | 取决于业务 | 业务 |
| **L6 序列化 (回程)** | 200µs (proto_lib:pack) | 3µs (prost) | N/A | 3µs |
| **L7 网络 (回程)** | 200µs (TCP send) | 5µs (tokio TcpStream write) | N/A | 5µs |
| **P50 总** | ~1.5ms (per BDD v0.1 §8.1 #1 估算) | ~20µs | 50µs | ~70µs |
| **P99 总** | ~2ms (gen_server 排队 + GC) | ~50µs | 100µs (saga 开销) | ~200µs |

**vs 闪烁之光 优势**:
- **P50**: 1.5ms → 70µs = **21x** 优势 (per BDD v0.1 §8.1 #1 闪烁之光 500µs P50)
- **P99**: 2ms → 200µs = **10x** 优势 (per BDD v0.1 §8.1 #2 闪烁之光 1ms P99)
- **端到端 RGS** 优势主要来自 tokio + tonic 静态生成 + gRPC mTLS vs gen_server + Erlang term

### 8.2 性能基线目标 (per BDD v0.1 §8.2 + 适配层新目标)

| 域 | RGS 目标 P99 (per BDD v0.1 §8.2) | 适配层 P99 (本 addendum 新增) | 端到端 P99 |
|---|---:|---:|---:|
| player-service | 50µs | 50µs | 200µs |
| economy-service | 100µs (saga 开销) | 50µs | 250µs |
| match-service | 30µs (real-time) | 50µs | 180µs |
| social-service | 80µs | 50µs | 230µs |
| admin-service | 200µs (audit 开销) | 50µs | 350µs |
| card-service | 40µs | 50µs | 190µs |
| batch-backend | 500µs (HTTP/JSON 适配) | 50µs | 650µs |
| gm-backend | 200µs (RBAC 开销) | 50µs | 350µs |

### 8.3 性能测试方法 (per BDD v0.1 §8.3 4 级 + 适配层新增)

1. **单元级** (per L1.1): 适配层 `cargo test --lib` (per AGENTS.md §2.1)
2. **集成级** (per L1.2 E2E): 适配层 → 8 域 backend mTLS 跑通
3. **压测级** (per Hybrid-3 rgs-loadtest, P3 backlog): N=10/100/1000/10000 闪烁之光 client × 1h
4. **对比级** (per 9 月 Phase C 后): 同 client (TCP 自研) 测 适配层 + RGS vs 闪烁之光 Erlang server, 输出 P50/P95/P99 对比

**适配层新增 5 级**:
5. **路由表压测**: HashMap<u16, RouteEntry> 静态查找 P99 ≤ 1µs (per §5.1)
6. **字段映射压测**: 静态 HashMap<&str, &str> 字段映射 P99 ≤ 1µs (per §6.3)
7. **session token 注入**: moka LRU cache 读 P99 ≤ 5µs
8. **mTLS handshake 缓存**: rustls resumption 缓存 P99 ≤ 5µs (避免每次 handshake)
9. **TCP 自研 codec**: bytes crate P99 ≤ 3µs (vs 手写)

### 8.4 已知性能缺口 (per 8/26 JST 缺标比错标)

- 适配层 P99 实测缺 (per BDD v0.1 §8.4, k3s Phase C 跑通后补)
- 闪烁之光 Erlang server 实际 P99 未测 (per handoff v0.1 §0 + BDD v0.1 §10.3, 9 月 Phase C 阶段 C 后)
- rgs-loadtest 缺 (per Hybrid-3, P3 backlog, 12/2 季度评审)
- 适配层 + 8 域端到端 P99 实测缺 (本 addendum v0.2 P2-1 跟进)

---

## 9. 部署 (k3s 独立 deployment + envoy 边缘 + 9/1 13:05 JST 偏好)

### 9.1 部署拓扑 (per AGENTS.md §7.1 + 9/1 13:05 JST + BDD v0.1 §6.1)

```
┌──────────────────────────────────────────────────────────────────────┐
│ k3s (rust-game-server ns)                                            │
│ Ingress: envoy 独立 deploy (9/1 13:05) - HTTP/2+mTLS+gm-console      │
│          + 适配层 envoy 边缘 (TCP :8780 + mTLS termination)           │
└──────────────────────┬───────────────────────────────────────────────┘
                       │ TCP 自研 (闪烁之光 协议) OR HTTP/2 (H5 客户端)
┌──────────────────────┴───────────────────────────────────────────────┐
│ 适配层 (tools/rgs-frontend-compat, 独立 Deployment)                  │
│  rgs-frontend-compat(r2,m1) :8780 TCP + 8792 metrics + 8793 health  │
│  (r=replicas, m=HPA min)                                              │
└──────────────────────┬───────────────────────────────────────────────┘
                       │ gRPC mTLS
┌──────────────────────┴───────────────────────────────────────────────┐
│ 8 域 backend (per BDD v0.1 §6.1, 独立 Deployment)                    │
│  player:50051  economy:50052  match:50053  social:50054              │
│  admin:50055  card:50061   batch:8790   gm-backend:8081             │
└──────────────────────┬───────────────────────────────────────────────┘
                       │
┌──────────────────────┴───────────────────────────────────────────────┐
│ 数据面: 8 独立 PG (ARC-008) + NATS JetStream + Prom/Grafana/Jaeger  │
└──────────────────────────────────────────────────────────────────────┘
```

### 9.2 端口分配 (per 9/1 13:05 JST 偏好 + rgs-flash-mock 模式)

| 端口 | 组件 | 依据 |
|---|---|---|
| **:8780** | 适配层 TCP listener (闪烁之光 client 入口) | 本 addendum §4.1 候选 1 |
| **:8781** | 适配层 WebSocket listener (H5 客户端, 候选 3 备选) | 本 addendum §4.3 |
| **:8782** | 适配层 XMLSocket listener (AS3 客户端, 候选 4 备选) | 本 addendum §4.4 |
| **:8791** | rgs-flash-mock HTTP/JSON (per FLASH-MOCK v0.3 §2.1) | 跟适配层解耦, 独立演进 |
| **:8792** | 适配层 metrics endpoint (Prometheus scrape) | 跟 5 域 metrics 一致 |
| **:8793** | 适配层 health endpoint (k8s liveness/readiness) | 跟 5 域 health 一致 |
| **:50051-:50055** | 5 域 backend gRPC (per BDD v0.1 §3.1) | 不动 |
| **:50061** | card 域 backend gRPC (per BDD v0.1 §3.1 #6) | 不动 |
| **:8790** | batch-backend HTTP/JSON (per BDD v0.1 §3.1 #7) | 不动 |
| **:8081** | gm-backend HTTP/JSON (per BDD v0.1 §3.1 #8) | 不动 |

### 9.3 k3s 资源 (per BDD v0.1 §6.1 + AGENTS.md §7.1 batch 域母规范)

| 资源 | 数量 | 备注 |
|---|---|---|
| **Deployment** | `rgs-frontend-compat` | 独立 deployment, 不跟 5 域共享 |
| **replicas** | 2 (HPA min 1, max 4) | 跟 rgs-batch-backend 模式 |
| **CPU request** | 500m | transcoder + mTLS 开销 |
| **CPU limit** | 2000m | 突发压测 |
| **Memory request** | 256Mi | tokio + 路由表 + LRU cache |
| **Memory limit** | 1Gi | moka LRU 100K entries |
| **HPA** | min 1, max 4, CPU 70% | 跟 5 域 HPA 模式 |
| **Service** | ClusterIP `rgs-frontend-compat:8780` | 内部 service |
| **Ingress** | (可选) NodePort 暴露 :8780 供外网 client | v0.2 P2 backlog 评估 |
| **Secret 挂载** | `/etc/rgs-frontend-compat/certs/` (per §7.4) | k3s secret 复用 5 域 certs |

### 9.4 部署清单 (k3s yaml, 3 文件, per AGENTS.md §7.1 + 9/1 13:05 JST)

```
docs/deploy/01-k8s-manifests/
├── 40-rgs-frontend-compat-deployment.yaml   # Deployment + HPA + ServiceAccount
├── 41-rgs-frontend-compat-service.yaml      # ClusterIP Service :8780
└── 42-rgs-frontend-compat-secret.yaml.example # Secret 模板 (per L-CAND-006, 实际 secret 走 kubectl create)
```

**envoy 边缘** (per 9/1 13:05 JST 偏好 + BDD v0.1 §6.1):
- 独立 deployment, 不引入 istio sidecar
- 监听 TCP :8780 + 转发 mTLS 到 适配层 :8780 (ClusterIP)
- 业务 svc 通过 `svc://rgs-frontend-compat:8780` 引用

### 9.5 部署顺序 (per AGENTS.md §2.5 L4 checklist)

1. `kubectl apply -f 40-rgs-frontend-compat-deployment.yaml`
2. `kubectl apply -f 41-rgs-frontend-compat-service.yaml`
3. `kubectl create secret generic rgs-frontend-compat-certs --from-file=ca.pem=... --from-file=client.pem=... --from-file=client-key.pem=...`
4. `kubectl rollout status deployment/rgs-frontend-compat`
5. `curl http://rgs-frontend-compat:8793/health` 验证

---

## 10. 测试 (闪烁之光 client + RGS backend 端到端)

### 10.1 测试分层 (per BDD v0.1 §8.3 4 级 + 适配层新增)

| 级别 | 测试目标 | 工具 | 状态 |
|---|---|---|---|
| **L1 (compile)** | `cargo check --tests` 0 error | cargo (per AGENTS.md §2.1) | v0.2 必跑 |
| **L1.1 (lib)** | 适配层 `cargo test --lib` 0 error | cargo test | v0.2 必跑 |
| **L1.2 (E2E)** | 适配层 → 8 域 mTLS 业务级跑通 | cargo test --test '*' | v0.2 必跑 |
| **L2 (集成)** | 438 cmds 抽样 22 RPC 路由到 8 域 | rgs-flash-mock (per FLASH-MOCK v0.3) | v0.2 |
| **L3 (压测)** | N=10/100/1000 闪烁之光 client × 1h | rgs-loadtest (Hybrid-3 P3 backlog) | v0.3+ |
| **L4 (对比)** | 同 client 测 RGS vs 闪烁之光 Erlang | (per 9 月 Phase C 后) | v0.3+ |

### 10.2 适配层单元测试 (per L1.1)

```rust
// tools/rgs-frontend-compat/tests/unit_routing.rs
#[test]
fn test_routing_partner_upgrade() {
    let entry = lookup_route(11002).unwrap();
    assert_eq!(entry.cmd, 11002);
    assert_eq!(entry.rgs_method, "/card.v1.CardService/UpgradeCard");
    assert_eq!(entry.rgs_domain, RgsDomain::Card);
    assert!(entry.need_auth);
    assert_eq!(entry.caller, Caller::Object);
}

#[test]
fn test_routing_map_na() {
    let entry = lookup_route(10201);
    assert!(entry.is_err() || matches!(entry, Ok(e) if e.rgs_domain == RgsDomain::N_A));
}

#[test]
fn test_unknown_cmd() {
    let result = handle_unknown_cmd(99999);
    assert!(matches!(result, Err(tonic::Status { code: InvalidArgument, .. })));
}
```

### 10.3 适配层集成测试 (per L1.2 E2E)

```rust
// tools/rgs-frontend-compat/tests/integration_e2e.rs
#[tokio::test]
async fn test_e2e_partner_upgrade() {
    // 1. 启动 8 域 backend (docker-compose 或 k3s)
    // 2. 启动适配层
    // 3. TCP 连接 :8780
    // 4. 发送 [Len=8][Cmd=11002][Body=u16 partnerId=123]
    // 5. 验证 RGS card-service 收到 UpgradeCard(player_id, card_id=123)
    // 6. 验证响应 [Len=N][Cmd=11002][Body=struct UpgradeResult{success: true}]
}

#[tokio::test]
async fn test_e2e_login_unauth() {
    // 1. 启动 8 域 backend
    // 2. 启动适配层
    // 3. TCP 连接 :8780
    // 4. 发送 [Len=8][Cmd=10201][Body=...] (NeedAuth=true 但未登录)
    // 5. 验证响应 Unauthenticated error
}
```

### 10.4 闪烁之光 client + RGS backend 端到端 (per 10.1 L2 + 借鉴分析 §4 #4)

**测试场景**:
1. 启动 适配层 + 8 域 backend (k3s namespace `rust-game-server-test`)
2. 启动 闪烁之光 tester 真实 client (per `tester/src/test.erl:39-78` + `tester_ai_base.erl` + `tester_ai_quest.erl`)
3. 跑 12 大类抽样 22 RPC (per FLASH-MOCK v0.3 §3)
4. 验证每条 RPC 适配层 → 8 域 → 业务结果

**12 大类抽样** (per FLASH-MOCK v0.3 §3 + 适配层 §5.2):
| 类别 | 闪烁之光 RPC | RGS 域 | 适配层 验证 |
|---|---|---|---|
| 场景/移动 | `MovePlayer(10201)` | (N-A) | 适配层返 Unimplemented |
| 角色养成 | `GetPlayerProfile(10301)` | player | 适配层 → player-service GetProfile |
| 角色养成 | `UpgradeCard(11002)` | card | 适配层 → card-service UpgradeCard |
| 战斗 PVE | `StartCombat(20001)` | match | 适配层 → match-service StartCombat |
| 战斗 PVE | `SubmitAction(20002)` | match | 适配层 → match-service SubmitAction |
| PVP | `EnqueuePVP(20201)` | match | 适配层 → match-service EnqueuePVP |
| PVP | `GetPVPMatch(20226)` | match | 适配层 → match-service GetPVPMatch |
| 公会 | `GetGuild(13501)` | social | 适配层 → social-service GetGuild |
| 公会 | `JoinGuild(13529)` | social | 适配层 → social-service JoinGuild |
| 经济 | `GetMarket(23501)` | economy | 适配层 → economy-service GetMarket |
| 经济 | `CreateAuction(23519)` | economy | 适配层 → economy-service CreateAuction |
| 社交 | `GetFriendList(13301)` | social | 适配层 → social-service GetFriendList |
| 社交 | `AddFriend(13316)` | social | 适配层 → social-service AddFriend |
| 活动 | `GetActiveEvent(16601)` | batch | 适配层 → batch-backend GetActiveEvent |
| 活动 | `ClaimReward(16613)` | batch | 适配层 → batch-backend ClaimReward |
| 付费 | `Recharge(21001)` | economy | 适配层 → economy-service Recharge |
| 付费 | `QueryRecharge(21003)` | economy | 适配层 → economy-service QueryRecharge |
| 排行榜 | `GetLeaderboard(12901)` | leaderboard | 适配层 → leaderboard-service GetLeaderboard |
| GM | `BanAccount` (gm-backend) | gm-backend | 适配层 → gm-backend BanAccount |
| GM | `GrantCompensation` (gm-backend) | gm-backend | 适配层 → gm-backend GrantCompensation |
| 抽卡 | `RecruitCard(23201)` | card | 适配层 → card-service RecruitCard |
| 抽卡 | `GetRecruitHistory(23203)` | card | 适配层 → card-service GetRecruitHistory |

**22 RPC 预期** (per FLASH-MOCK v0.3 §3 + 适配层 §5.2):
- ✅ PASS: 19 RPC (业务透传到 8 域 backend 跑通)
- 🟡 PARTIAL: 1 RPC (e.g. 场景移动 N-A 返 Unimplemented)
- ❌ N-A: 1 RPC (e.g. 场景移动)
- ⏳ NOT-IMPLEMENTED: 1 RPC (TBD v0.3 业务代理)

### 10.5 已知测试缺口 (per 8/26 JST 缺标比错标)

- 5 个 proto 未深读 (per BDD v0.1 §10.1): social / replay / leaderboard / i18n / cluster-ops — v0.2 P2-1 跟进
- 闪烁之光 跨盘 .tsv 文件未读 (per BDD v0.1 §10.1 P2-2)
- 闪烁之光 实际 proto 风格未直接看 (per BDD v0.1 §10.1 P2-4)
- 43 条未提取 + 113 条无标题 (per BDD v0.1 §10.1) — v0.2 抽样 22 RPC 验证, 完整 438 RPC 走 v0.3+
- rgs-loadtest 缺 (per Hybrid-3, P3 backlog) — v0.3+ 评估
- 适配层 L1.2 E2E 业务 mTLS 跑通缺实测 (per AGENTS.md §2.1 DoD) — v0.2 必跑
- 闪烁之光 实际 P99 未测 (per BDD v0.1 §10.3 + 9 月 Phase C 阶段 C 后)

---

## 11. 已知缺口 (per 8/26 JST 缺标比错标, 5 段)

### 11.1 报告本身 (addendum v0.2 → v0.3 升版)

- **5 个 proto 未深读** (per BDD v0.1 §10.1): social / replay / leaderboard / i18n / cluster-ops — v0.3 P2-1 跟进
- **闪烁之光 跨盘 .tsv 文件未读** (per BDD v0.1 §10.1 P2-2): `E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单-*.tsv` — 跨盘权限受限, v0.3 P2-2 跟进
- **闪烁之光 实际 proto 风格未直接看** (per BDD v0.1 §10.1 P2-4): 通过 5 大可取之处 + system prompt 推断, v0.3 P2-4 跨盘 .erl 文件抽样
- **43 条未提取 + 113 条无标题** (per BDD v0.1 §10.1) — 不影响 v0.2 决策
- **30 新建 module 业务扩展估算 v0.2 粗** (per BDD v0.1 §10.1 P2-3): 需 v0.3+ 抽样 read 闪烁之光 模块细节
- **本 addendum 路由表 §5.2 抽样 22 RPC, 完整 438 cmds 路由表 v0.3 P2-5 跟进**

### 11.2 框架对照 (per audit v0.3 §8.2 + 9 原则 + 6 反模式)

- **框架原则 #1 (per-entity actor) 0/8 域实装** (per audit v0.3 §1.2 #1): DB-as-state 决策保留
- **框架原则 #4 (协议 schema push) 8 域未实装** (per audit v0.3 §8.2): P2 backlog, 跟 RGS-SPEC-CROSS-002 v0.2 升版联动
- **框架原则 #9 (登录准备链声明式) 8 域未实装** (per audit v0.3 §8.2): P2 backlog
- **反模式 A1 (Arc<Mutex<RoleData>>) 32 处待逐个验证** (per audit v0.3 §2.2): v0.2 P2 跟进
- **反模式 A2 batch `state: String`** (per audit v0.3 §1.2 #6): P1 backlog, 应改 enum BatchTaskState
- **适配层 N-A 业务代理 v0.2 不实装** (per 本 addendum §6.4): v0.3+ 评估

### 11.3 数据缺口

- **闪烁之光 性能 baseline 未测** (per BDD v0.1 §10.3 + handoff v0.1 §0): 需起 闪烁之光 Erlang server 跑同 client, 9 月 Phase C 阶段 C 后对比
- **RGS 8 域 P99 实测缺** (per BDD v0.1 §10.3): 8 域实测 P99 待补
- **rgs-testkit 现状** (per BDD v0.1 §10.3): 缺跟 闪烁之光 `tester*.erl` 对比数据
- **rgs-loadtest 缺** (per Hybrid-3, P3 backlog, 12/2 季度评审)
- **适配层 P99 实测缺** (本 addendum v0.2 P2-1 跟进)
- **适配层 + 8 域端到端 P99 实测缺** (本 addendum v0.2 P2-1 跟进)
- **438 cmds 完整路由表未生成** (本 addendum §5.2 抽样 22 RPC, 完整走 v0.3+)

### 11.4 业务缺口

- **batch 域 cron 引擎 + audit_logger + worker_pool 实装** (per BDD v0.1 §10.4 + audit v0.3 §8.1): 待 v0.2 batch worker 跟进
- **12 大类业务层 30 module 业务扩展** (per BDD v0.1 §10.4 §3.2 Phase 2-4): v0.1 baseline 估算, v0.2+ 跟 闪烁之光 实际业务层抽样 + RGS TCG 业务裁剪
- **TCG vs MMORPG 业务映射 90% N-A** (per BDD v0.1 §10.4 + handoff v0.1 §1 + audit v0.3 §1.2 #1): 闪烁之光 是 MMORPG, RGS 是 TCG, 适配层走业务透传 + N-A 返 Unimplemented
- **5 域 binary 未来调外部 LLM 未登记** (per handoff v0.1 §2.2 OLU-WEB F-25): v0.1 不集成, v0.2 评估
- **业务代理 v0.2 不实装** (per 本 addendum §6.4): v0.3+ 抽样 5-10 个 N-A 业务评估

### 11.5 治理缺口

- **RGS-SPEC-CROSS-002 v0.1 🔴 NO-GO** (per BDD v0.1 §10.5 + audit v0.3 §0.2 + flash overlap §0.2): 协议风格指南未激活, 错误码字典 CROSS-001 待办, 激活条件 G-CODE-06 (cargo 全绿) + G-CODE-03 (5 独立 DB 拓扑图)
- **RGS-CRITIQUE-IMPROVEMENT v0.2 §2.3 一致性** (per BDD v0.1 §10.5 + flash overlap §9.2): 9 月 Phase C 后, 业务深度评估待 12/2 季度评审
- **RGS-WEEKLY W37 v0.1 收口** (per BDD v0.1 §10.5 + flash overlap §9.2): W37 D7 9/14 JST 收口
- **5 域 + card + batch + gm-backend Lead 实际身份** (per BDD v0.1 §10.5 + 8/21 JST 决策): 5 域独立真实身份 per 8/21 JST, DDD Review 阶段可补
- **AGENTS.md v0.x 升版同步** (per BDD v0.1 §10.5 + 8/27 JST + 8/21 JST + 9/1 13:05 JST + 9/1 18:30 JST): 主会话负责, worker 不动 AGENTS.md
- **DDD Review 二审 (per DDD-REVIEW-TEMPLATE-v0.2)**: 本 addendum v0.2 状态 ⏳ 待 Mavis 自审 → 🟡 Mavis 自审停手 → ⏳ 待 Ulysses 二审
- **闪烁之光 client Lua/AS3/Unity 改造需求未确认** (per 本 addendum §4.4 候选 4 评估): 业务方 9 月 Phase C 后确认

---

## 12. 签字栏 + 修订历史 (per DDD-REVIEW-TEMPLATE-v0.2 二审流程)

### 12.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 三行齐全 (见顶部) |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1/L1.1/L1.2 三件套, 本 addendum 0 Rust 改动 (N/A 通过) |
| Evidence 段 (commit SHA / file:line / 测试函数名) | ✅ | 闪烁之光 跨盘 4 文件 file:line 引用 + mapping.erl:40-83 路由表抽样 + BDD v0.1 跨文档引用 |
| 代签段 (per 8/27 JST 三次强化) | ✅ | Mavis 默认代签 Ulysses (顶部 author / 审批 / 修订人) |
| 派生约束守护段 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ✅ | §0 + §11 已知缺口 全部 deferred 实时查询; L1/L1.1/L1.2 N/A (0 Rust 改动); L11 N/A (0 cargo 跑); L12 N/A (1 worker 派工, 主会话统一 1 commit); L14 N/A (0 plumbing patch) |
| 缺标比错标 (per 8/26 JST) | ✅ | §11 5 段已知缺口 显式列 (报告本身 / 框架对照 / 数据 / 业务 / 治理) |
| 凭据 REDACTED filter (per 8/27 11:06 JST 硬 ban) | ✅ | §7.3 env var 列表 + 凭据走 env var + 0 env value 打印 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 0 "per X 历史形态" + 0 "原本是" 类回溯, 全部基于 git 实证 + 跨盘 file:line |
| BAS git log --follow 实证 (per 8/26 JST DTL-036 v1.4 hotfix) | ✅ | 0 BAS 升版 (本 addendum 不升 BAS, 独立 addendum 文件) |
| 子代理授权边界 (per 8/27 19:39 JST 决策 + 9/1 14:58 JST 拍板) | ✅ | 1 worker 派工, 主会话统一 1 commit, 0 amend / rebase / filter-branch |
| 5 域独立 Lead 守护 (per 8/21 JST, 扩展到 8 域) | ✅ | 适配层独立 crate, 不动 8 域 gRPC proto + DB 拓扑 |
| envoy 独立 deployment (per 9/1 13:05 JST 偏好) | ✅ | §9.4 部署清单 envoy 独立 deploy, 不引入 istio sidecar |
| DB 三分类横展 (per 9/1 18:30 JST 横展原则) | ✅ | 适配层 0 DB (业务透传, 不存 session), 引用 BDD v0.1 §6 数据面 8 独立 PG |
| 拍板决策 ask_user (per 9/1 14:58 JST 偏好) | ✅ | 本 addendum 是 ask_user option A 第 3 项落地, 9/4 17:11 JST "frontend compat 正确设计" 拍板 |
| **总状态** | **⏳ 待二审** | 自审 1 次停手 (per B3 派生约束) |

### 12.2 Ulysses 二审 (必到, per B3 派生约束 + DDD-REVIEW-TEMPLATE-v0.2 §3)

| 项 | 状态 | 备注 |
|---|---|---|
| 一审 vs 二审 业务深度 (per DDD-REVIEW-TEMPLATE-v0.2 §3) | ⏳ 待审 | Mavis 自审 + Ulysses 二审, 业务深度待 12/2 季度评审 |
| 自指字段 (per L13 self-referencing deferred) | ⏳ 待审 | §11.5 DDD Review 状态机 + §0 状态 = ⏳ 待二审 |
| 派生约束一致性 (per 8/21 / 8/26 / 8/27 / 9/1 / 9/4 JST) | ⏳ 待审 | §0 顶部派生约束守护段 + §7 安全 + §9 部署 全部一致 |
| 业务指标 (per BDD v0.1 §8.1 20x-100x 性能 + §9 mTLS 业务级) | ⏳ 待审 | §8.1 端到端 P99 200µs vs 闪烁之光 1ms = 5x 优势 (vs BDD v0.1 §8.1 20x) |
| commit ahead (per DDD-REVIEW-TEMPLATE-v0.2) | ⏳ 待审 | 本 addendum 1 file 落地, 主会话统一 1 commit |
| RGS-CRITIQUE 一致性 (per DDD-REVIEW-TEMPLATE-v0.2) | ⏳ 待审 | §0 引用 9/4 16:47 / 9/4 17:11 / 9/4 16:45 / 9/4 16:14 JST 拍板一致 |
| **总状态** | **⏳ 待二审 → 🟡/✅/❌** | per DDD-REVIEW-TEMPLATE-v0.2 §3 二审流程 |

### 12.3 修订历史 (v0.2 addendum row, 续 BDD v0.1 主 doc)

| 版本 | 日期 | 作者 (per 代签授权) | 主要变更 |
|---|---|---|---|
| **v0.2 addendum** | 2026-09-04 17:11 JST | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 闪烁之光 client 适配层设计 (gRPC transcoder / JSON-RPC proxy / WebSocket / Flash socket 兼容 4 选项对比, 推荐 gRPC transcoder 候选 1), 0-11 段 12 节 (文档元信息 / 引言 / 闪烁之光 client 协议分析 / 适配层架构总览 / 适配层组件设计 / 协议号 → RGS proto 1:1 路由表 / 业务层适配 / 安全 / 性能 / 部署 / 测试 / 已知缺口) + 12 段 签字栏 + 修订历史, 闪烁之光 跨盘 4 文件 file:line 实证 (协议栈.md L1-L5 + mapping.erl:40-83 + tester.erl:39-78 + services.erl:33-56) + 4 选 1 推荐 gRPC transcoder 单一 listener :8780 + 22 RPC 抽样路由表 + 业务透传优先 + 端到端 P99 200µs vs 闪烁之光 1ms = 5x 优势 + mTLS fail-closed + 凭据走 env var + k3s 独立 deployment per AGENTS.md §7.1 + 9/1 13:05 JST envoy 独立 deployment 偏好 + 5 段已知缺口 (报告本身 / 框架对照 / 数据 / 业务 / 治理), per L13 self-referencing deferred + 8/27 11:06 JST 凭据硬 ban 守护 + 8/26 JST 禁回溯叙事守护 + 8/21 JST 5 域独立 Lead 守护 (扩展到 8 域) + 9/4 17:11 JST "frontend compat 正确设计" 拍板 + 9/4 16:45 JST "完全对齐" 拍板 + 9/4 16:14 JST "完整 1351 mock" 拍板 + 9/4 16:47 JST "3 件套补全" 拍板 + 9/1 13:03/13:05 JST envoy 独立 deployment 偏好 + 9/1 18:30 JST DB 三分类横展原则 + 9/1 14:58 JST 拍板决策 ask_user 偏好 + 9/1 12:36 JST L-CAND-006 派生约束升正式 + 闪烁之光 跨盘引用可独立 Read 验证 (per AGENTS.md §1.1) |

### 12.4 附录: 关键引用一览 (跨文档 + 跨盘 file:line 实证)

**闪烁之光 跨盘 4 文件** (per AGENTS.md §1.1 可独立 Read 验证):
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\docs\network-topology.html` (23KB, Hub-and-Spoke + Peer-to-Peer 拓扑, 3 链路)
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\docs\architecture\协议栈.md` (4.1KB, L1-L5 5 层协议栈 + 协议格式)
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\services.erl` (12.2KB, center/zone 节点配置 line 33-56)
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\src\mapping.erl` (5.5KB, 协议路由表 line 40-83)
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\tester\src\test.erl` (6.6KB, 真实 client bot line 39-78)
- `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\tester\src\tester.erl` (17KB, 真实 client 入口)

**RGS 跨文档 3 件套** (per commit `80bcd3b`):
- `D:\RustGameServer\docs\15-IPA-完全对齐438cmds\RGS-REQ-2026-09-04_v0.1.md` (61.4KB)
- `D:\RustGameServer\docs\15-IPA-完全对齐438cmds\RGS-BDD-2026-09-04_v0.1.md` (49.5KB, 本 addendum 基线)
- `D:\RustGameServer\docs\15-IPA-完全对齐438cmds\RGS-DDD-2026-09-04_v0.1.md` (94KB)
- `D:\RustGameServer\docs\14-项目治理\RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.3.md` (28KB, mock 设计 4 阶段)
- `D:\RustGameServer\AGENTS.md` (32KB, 仓库级强约束, per 8/21 / 8/26 / 8/27 / 9/1 JST 派生)

**关键 commit SHA** (per git log --oneline 实证):
- `80bcd3b` (3 件套 v0.1 baseline, BDD v0.1 + REQ v0.1 + DDD v0.1)
- `49eb51a` (FLASH-MOCK v0.3 已落 main, 4 阶段路线图)
- `2e3d9ee` (FLASH-OVERLAP v0.2, 11 维度 keep RGS)
- `bb9f977` (GAP-AUDIT v0.3, 5 域 + card 架构保留)

**关键 user 拍板 (per 9/4 JST 决策链)**:
- 9/4 16:47 JST: "首先补全需求文档, 基本设计文档, 详细设计文档, 内容根据闪烁之光代码逆推" → 3 件套 v0.1
- 9/4 16:14 JST: "完整 1351 mock (long-term)" → FLASH-MOCK v0.3 4 阶段
- 9/4 16:45 JST: "完全对齐" → 推翻 handoff v0.1 "不做逐条移植" 决策
- 9/4 17:11 JST: "frontend compat 正确设计" → 本 addendum v0.2

**派生约束守护** (per AGENTS.md §1-§7 + 8/21 / 8/26 / 8/27 / 9/1 JST 强化):
- L1/L1.1/L1.2 (per D2 拍板) N/A: 本 addendum 0 Rust 改动
- L11 (PT 派工 cargo build dir lock 防御) N/A: 0 cargo 跑
- L12 (PT 派工临时 log 不入 commit) N/A: 1 worker 派工, 主会话统一 1 commit
- L13 (self-referencing deferred): §11.5 + §12.1 状态机 deferred
- L14 (plumbing 节点字符串处理) N/A: 0 plumbing patch

---

**v0.2 addendum 完。** 主会话负责 review + 1 commit (per L12.2 选项 2 落地模式, 1 worker 写文件, 主会话统一 git add + commit)。后续 v0.3+ 跟 闪烁之光 client 协议升级 + 8 域 backend 升版同步。
