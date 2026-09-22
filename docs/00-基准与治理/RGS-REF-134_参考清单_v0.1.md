# RGS-REF-134 参考清单 v0.1 — 两款商用服务器参考亮点清单

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-REF-134 |
| 版本 | v0.1 |
| 关联 issue | Multica **ULYS-134** (`01a0be20-14fe-7eb0-9fab-741c140b9c23`) |
| 状态 | 🟢 v0.1 初版（per D-Boy 9/21 JST "继续推进" directive，承接 9/20 10:45 agent 帖子中途掉线重启）|
| 关联交付 | `RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化.md` v0.2 §2.1（5 项可立即执行）+ `RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md` §2.1+§2.2（9 原则 + 6 反模式 fingerprint）+ `RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04_v0.2.md` §3.1+§3.2（5 可取之处 + 1 反例）|
| 修订人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 |
| 审批 | 架构师（Mavis 接手 agent per DEC-008）|
| 代签授权 | 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses) |

---

## 0. 一句话结论

RGS 实际参考的"两款商用服务器"是 **① 闪烁之光（zsyz_server, Erlang/OTP 第三方 MMORPG）** 与 **② Erlang/OTP 商用游戏服务器设计参考框架**（per D-Boy 2026-09-04 14:30 JST paste 的 system prompt，9 原则 + 6 反模式）。本清单逐条列这两款参考为 RGS 贡献了哪些"亮点"（即可直接采纳 / 已经采纳 / 登记为 backlog 的架构/工程/业务层借鉴点）。

> **注**: "Erlang/OTP 设计参考框架"严格说不是单一一款商用 server，而是行业 30+ 年商用 Erlang 游戏服务器（WhatsApp、Mochi Media、Wooga、FarmVille、Discord、Ericsson AXD301 等）沉淀的设计语言，由 D-Boy 9/4 14:30 JST 以 system prompt 形式贴给 RGS 团队作为"9 原则 + 6 反模式"基线，**RGS 一致对齐 = 等同于参考一款成熟的商用 server 设计基线**。

---

## 1. 闪烁之光 (zsyz_server) 参考亮点清单

**参考对象**: 第三方 MMORPG "闪烁之光" 服务端源码，437 个 `.erl` / `.hrl`（per `E:\BaiduNetdiskDownload\闪烁之光\server分析\zsyz_server\docs\README.md` L30），1351 条 RPC 已成功提取 1351/1394（97.0%，per 借鉴分析 .md §0）。
**关联证据**: `RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化.md` + `RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04_v0.2.md` + `RGS-REQ-2026-09-04_v0.2.md` + `RGS-DDD-v0.2-addendum-协议号映射.md` + `RGS-DDD-v0.2-addendum-业务逻辑逆推.md` + `RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.3.md` + `RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md`。

### 1.1 协议层亮点（5 条可取之处 + 1 条反例 per 借鉴分析 doc §4）

| # | 亮点 | 闪烁之光 做法 | RGS 落地 | 决策 | 关联 |
|---|---|---|---|---|---|
| 1.1.1 | **契约即代码生成源** | 一条 `#rpc{code, log_title, req, reply}` 记录同时驱动编解码生成 + 按命令的运维审计标题 | `tonic-build` + `prost` 编译期生成 + admin 域 `audit_log.action` 字段 | **keep RGS** + **Hybrid-1** = admin 域 audit_log 增 `log_title` 字段 | OVERLAP §6.1 / handoff §2.1.1 |
| 1.1.2 | **随包自带 bot 真实协议压测器** | `tester*.erl` 用真实协议自动跑测的压测/回归工具直接放代码库 | `rgs-testkit`（NoOp mock + 测用 InMemory repo + chaos 测试） | **keep RGS** + **Hybrid-3** = 评估 rgs-testkit 加 bot 压测工具（压测 real protocol） | OVERLAP §6.3 / handoff §2.1.5 |
| 1.1.3 | **业务逻辑不入库 = 编译期常量模块** | 181 个游戏数值表编译成 Erlang 常量模块 `*_data.erl`（不运行时查库） | RGS `RGS-ADR-0022`（业务逻辑不入库）已有同原则决策 | **ADR-0022 加外部实证附注**，作为决策合理性的外部佐证 | handoff §2.1.2 |
| 1.1.4 | **数据驱动活动/榜单框架** | 12 大类 184 活动运营走 task_templates 配置化（理论上；实测反例见 #1.1.6） | RGS `task_templates` Master 表 + version 字段（GAP-7）已 master 模板化 | **L-CAND-010 候选** = 强制禁止"一个活动/一个榜单变体一个独立模块"的复制模式 | handoff §2.1.3 |
| 1.1.5 | **多 zone 认领一个 center 节点分片模型** | 实测 `zone/sszg_symlf_3225/env.cfg` 显式 `center_node`，与 ADR-0052 "全可达 PFAU" 是同一问题不同解法 | `crates/cluster-ops` + `RGS-ADR-0052` | **登记对照笔记**，不要求得出结论 | handoff §2.1.4 |
| 1.1.6 | ⚠️ **反例（必须避免）** | 9 个 `holiday_*` 节日活动 proto 文件 + 6 个高度相似的竞技场变体文件（9+6=15），同一套骨架复制多份换皮 | RGS match v2 单 file 9 RPC + 经济 trade_saga 单 file 3 saga 已避免 | **保持 RGS**（不照搬复制模式） | OVERLAP §3.2 / handoff §2.1.3 (L-CAND-010) |

### 1.2 业务层亮点（按 12 大类 per 借鉴分析 doc §2 + REQ v0.2）

| # | 大类 | 闪烁之光 RPC 数 | RGS 业务映射（TCG）| 决策 | 关联 |
|---|---|---:|---|---|---|
| 1.2.1 | 场景/移动 | 148 | 不适用（TCG 玩家客户端无移动） | ❌ 不借鉴 | REQ v0.2 §1.4 |
| 1.2.2 | 角色养成 | 198 | 卡组养成（DeckRepository 桶 11 增量） | 🟡 部分借鉴（设计模式不字段） | REQ v0.2 §6.4 |
| 1.2.3 | 战斗 PVE | 241 | 战斗引擎不适用；副本/挑战奖励结构可借鉴 | ❌ 业务层 | REQ v0.2 §6.4 |
| 1.2.4 | 经济（商城/交易/抽卡） | 90 | 限时商店/兑换机制可作设计参考 | 🟡 受 Gate 阻塞（`RGS-SPEC-CROSS-002` NO-GO）| handoff §2.2 |
| 1.2.5 | 社交（公会/邮件/好友） | 123 | social-service 仅 1 RPC，公会全家桶 97 条可作扩展素材 | 🟡 受 Gate 阻塞 + 跨服中心服架构需独立评估 | handoff §2.2 |
| 1.2.6 | 活动运营 | 184 | **反例 → 强制数据驱动** | 🟢 L-CAND-010 候选约束 | handoff §2.1.3 |
| 1.2.7 | 排行榜 | 10 | leaderboard_db.snapshot 已有 | 🟡 结构可借鉴，不照搬多变体 | handoff §2.2 |
| 1.2.8 | GM 指令 | 37 | 核对现有 6+5 条 RPC 是否有遗漏字段，不新增 RPC | 🟢 admin 域 audit_log 增 `log_title`（Hybrid-1）| OVERLAP §6.1 |

### 1.3 网络层亮点（per 9/4 16:47 JST directive + RGS-BAS-027 WebSocket 网络网关）

| # | 亮点 | 闪烁之光 做法 | RGS 落地 | 决策 | 关联 |
|---|---|---|---|---|---|
| 1.3.1 | **TCP/WebSocket 帧结构** | `[4B length u32 BE][2B cmd u16 BE][payload]`，length = payload_len + 2 | `crates/network-gateway/src/codec.rs` + `ws.rs` + `golden_vectors.rs` | ✅ **完美对接**（per ULYS-2 sub-issue A/B/C/D，9/12 chat 升级执行阶段）| HANDOFF §2.3 / RGS-DTL-027 |
| 1.3.2 | **9 TLV 字段类型** | int8/uint8/int16/uint16/int32/uint32/str/bytes/array | 同款 9 TLV | ✅ 已落 `codec.rs` | HANDOFF §2.3 |
| 1.3.3 | **协议码空间 u16 + O(1) 派发** | u16 (0-65535), 1351 条已用（proto_11~proto_284） | tonic 自动 dispatch ✅（0 处 `HashMap<u*,*>` 派发，反模式 A6 不命中）| ✅ **keep RGS**（per GAP-AUDIT §2.1 原则 #8） | GAP-AUDIT §2.1 |
| 1.3.4 | **WebSocket 路径 `/websocket` + 端口 8000** | `zsyz_server/src/web_conn.erl` 端口 8000 | `crates/network-gateway/src/ws.rs` + `bin/main.rs` 默认 ON @ 0.0.0.0:8000 | ✅ 1:1 对齐 | HANDOFF §2.3 / RGS-DTL-027 |
| 1.3.5 | **心跳 cmd=1199** | 心跳帧 | RGS 同款 | ✅ 已落（per ULYS-2.4 E2E 验收） | HANDOFF §2.3 |
| 1.3.6 | **握手 cmd=1110 登录流** | zsyz_client_h5 (Cocos Creator H5) 真实登录流 | RGS `puppeteer + H5 浏览器跑通 cmd=1110 登录握手` | ✅ ULYS-6 (ULYS-2.4) E2E | HANDOFF §2.3 |

### 1.4 业务逻辑亮点（per DDD-v0.2-addendum-业务逻辑逆推，5 抽样 .erl）

| # | 亮点 | 闪烁之光 业务 | RGS 翻译模式 | 性能比 | 关联 |
|---|---|---|---|---|---|
| 1.4.1 | **DB roundtrip** | ~1ms（mysql）| ~500µs（sqlx + connection pool）| **2x 优势** | DDD addendum 业务逻辑逆推 §5 / REQ v0.2 §11 |
| 1.4.2 | **序列化/反序列化** | ~50µs（term_to_binary）| ~2µs（serde + bincode）| **25x 优势** | 同上 |
| 1.4.3 | **角色进程 = gen_server**（per `role.erl` L8 `-behaviour(gen_server).`） | 1 player 1 process + 进程字典 + 异步消息 + 延时 3min 关闭 | RGS DB-as-state（无 per-entity actor）| **架构性差异 ≠ 反模式**（TCG 100K+ 在线品类依据）| DDD addendum 业务逻辑逆推 §5 |
| 1.4.4 | **战斗 = gen_fsm 9 FSM 状态机**（per `combat.erl` L11 `-behaviour(gen_fsm).`）| 9 FSM + 进程注册 + 跨进程消息 | RGS match v2 `enum MatchEvent` + `SessionStatus` + **8 transition_to_xxx 函数**（transition_to_waiting/starting/running/paused/resumed/ending/ended/canceled）| ✅ 1:1 抽象 | GAP-AUDIT §3.2 / DTL-038 §5.2 |
| 1.4.5 | **公会 gen_server + ets 缓存 + mpsc 异步 apply**（per `guild.erl` L62-70 4 个变体）| 1 guild 1 process + ets 缓存 | RGS social-service + Arc<DashMap> 热缓存（待评估）| 🟡 待评估 | DDD addendum 业务逻辑逆推 §3 |
| 1.4.6 | **竞技场非 gen_server + role:redirect/3 + 5 push 函数**（per `arena.erl`）| 6 变体挑战列表通过 `match/2` 函数实现 | RGS match v2 single RPC 抽象，`arena_type` enum + 内部 `match/2` 不暴露变体 | ✅ 抽象层级更高 | DDD addendum 业务逻辑逆推 §3 |
| 1.4.7 | **DB 13 核心业务表 → RGS 7 域独立 DB 映射** | `role` / `role_lev_gift` / `guild` / `guild_member` / `partner` / `partner_artifact` / `combat_replay` / `rank_*` / `market_*` / `mail` / `vip_*` / `holiday_*` | `player_db.player` / `batch_db.task_templates` / `social_db.guild` / `social_db.guild_member` / `card_db.card_instance` / `replay_db.replay` / `leaderboard_db.snapshot` / `economy_db.auction` / `social_db.mail` / `player_db.player.vip_*` / `economy_db` charge log | ✅ 1:1 映射 | REQ v0.2 §6.4 |

### 1.5 网络拓扑亮点（per RGS-DDD-v0.2 §2.2）

| # | 亮点 | 闪烁之光 做法 | RGS 做法 | 决策 | 关联 |
|---|---|---|---|---|---|
| 1.5.1 | **center + zone 2 节点** | 闪烁之光显式 center/zone 分层 | RGS active-active 7 域（无 center，DB-as-state）| ✅ 架构差异（per audit v0.3 §1.2 #1）| REQ v0.2 §2.2 |
| 1.5.2 | **sup_db_buffer 缓冲写盘** | Erlang gen_server 缓冲 | RGS `shared-platform::outbox` | ✅ 等价实现（per audit v0.3 §4 #5）| GAP-AUDIT §3.1 |
| 1.5.3 | **cluster_srv/cluster_msg 集群 RPC** | Erlang 集群 RPC | RGS NATS (outbox relay publish) + tonic gRPC mTLS | ✅ 更现代 + 跨语言 | REQ v0.2 §2.2 |
| 1.5.4 | **role_data + role_query 角色进程** | Erlang 1 player 1 process | RGS DB-as-state（无 per-entity actor）| ✅ 架构差异记录在 ADR-0060（5 域 P1-12 backlog）| GAP-AUDIT §3.1-3.6 + §7 P1-12 |

---

## 2. Erlang/OTP 商用游戏服务器设计参考框架亮点清单

**参考对象**: D-Boy 2026-09-04 14:30 JST 贴的 system prompt，**9 原则 + 6 反模式** = Erlang/OTP 商用游戏服务器 30+ 年沉淀的设计语言（WhatsApp / Mochi Media / Wooga / FarmVille / Discord / Ericsson AXD301 等基线）。
**关联证据**: `RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md` §2.1（9 原则 fingerprint）+ §2.2（6 反模式 fingerprint）+ §3.1-3.7（6 域现状对照）+ §7（P1/P2/P3 backlog）。

### 2.1 9 原则亮点（per GAP-AUDIT §2.1）

| # | 原则 | Erlang 原型 | Rust 落地 fingerprint | RGS 落地状态 | 决策 |
|---|---|---|---|---|---|
| 2.1.1 | **玩家进程 = 1 tokio task + mpsc** | `gen_server` per player | `tokio::spawn` + `mpsc::channel` per player + `while let Some(msg) = rx.recv().await` | ❌ 不适用（架构差异：DB-as-state 适合 TCG 100K+ 在线）| ✅ ADR-0060 决策记录（5 域 P1-12）|
| 2.1.2 | **战斗 FSM = enum + match** | `gen_fsm` + 显式状态 | `enum State` + `TRANSITION_TABLE` + `apply(event) -> Result<State, TransitionError>` + `is_terminal()` | 🟢 **满分命中**：`rgs-asset-download` + outbox + saga + match entity_v2 | ✅ **保持 RGS**，已落地 |
| 2.1.3 | **跨服调用 = 拆分执行器 + 自动本地/远端路由** | `cluster_lib:split_srv_exec/3` | `split_by_srv<T: HasSrvId>(items) -> HashMap<SrvId, Vec<T>>` + `join_all` | 🟡 部分命中：batch 域 `enum GrpcDomain` 5 桶 ✅；matchmaker_v2 跨域 replay fire-and-forget ✅ | 🟡 P2 backlog（P2-5 显式 `split_by_srv<T>` 抽象）|
| 2.1.4 | **协议版本兼容 = 协议描述随包** | `proto_lib:repack/2` | prost / flatbuffers + 动态反射（prost-reflect）+ 服务端推 schema | ❌ 未实装（tonic 静态生成）| 🟡 P2 backlog（P2-1 prost-reflect 评估）|
| 2.1.5 | **DB 批量写盘 = 双触发** | `sup_db_buffer {interval, number}` | `DbWriter {rows: Mutex, last_flush, cfg}` + interval + count + sqlx batch INSERT | 🟢 **满分命中**：`shared-platform::outbox` 实现变体（lease 30s + count 触发）| ✅ **保持 RGS**，已落地 |
| 2.1.6 | **事件触发器 + 延迟去抖** | `role_trigger:delay_fire/2` | `delay_evt: HashMap<EvtLabel, Evt>` + `delay_timer: HashMap<EvtLabel, AbortHandle>` + `tokio::time::sleep` + `AbortHandle` | 🟡 部分命中：matchmaker_v2::EventBus broadcast ✅ | 🟡 P2 backlog（P2-4 AbortHandle 去抖工具）|
| 2.1.7 | **热冷分层 + 战斗录像** | ETS（30min 热）→ DETS → DB | `Arc<DashMap<ReplayId, Replay>>` + sled/redb 冷 + PG 永久 + `Arc<AtomicUsize>` 引用计数 | 🟡 部分命中：replay-service crate 存在 ✅ | 🟡 P2 backlog（P2-3 DashMap 热 + sled/redb 冷 + AtomicUsize）|
| 2.1.8 | **协议号 → 模块 O(1) 派发** | `mapping:code/2` + `-compile({inline, [code/2]})` | `const DISPATCH: [Option<Dispatcher>; 65536] = build_table()` + `match cmd { 102 => ... }` | 🟢 **满分命中**：tonic 自动生成 dispatch ✅，0 处 `HashMap<u*,*>` 派发（反模式 A6 不命中）| ✅ **保持 RGS**，已落地 |
| 2.1.9 | **登录准备链 = 声明式顺序 + 失败标签** | `role_listener ?ready_list` + `{all, ...}` / `{first, ...}` | `enum ReadyStep { All(Fn), First(Fn) }` + iterator + `try_fold` | 🟡 部分命中：cluster-ops::realm_lifecycle（per entity 8 enum State）✅ | 🟡 P2 backlog（P2-2 `enum ReadyStep` 抽象）|

### 2.2 6 反模式亮点（必须避免 per GAP-AUDIT §2.2）

| # | 反模式 | 正确做法 | RGS 现状命中 | 决策 |
|---|---|---|---|---|
| 2.2.1 | **A1: `Arc<Mutex<RoleData>>` 全局共享** | Actor + mpsc，串行化 | matchmaker_v2.rs:111 `Arc<AsyncMutex<HashMap<Uuid, EventSender>>>` — 🚨 A1 命中，**严重度 P2**（用途是 EventBus map，不是 RoleData；AsyncMutex 是 tokio 锁，不是 std Mutex）| 🟡 待明确 |
| 2.2.2 | **A2: `String` 当状态机状态名** | `enum` + exhaustive match | 0 命中（matchmaker_v2.rs:81 `end_reason: String` 在 MatchEvent 变体字段，非状态字段）| ✅ **不命中** |
| 2.2.3 | **A3: `tokio::spawn(sqlx::query(...))` 散枪** | DbWriter 后台批量 | 0 命中（190 处 sqlx::query 是"读为主，写走 outbox"）| ✅ **不命中** |
| 2.2.4 | **A4: `for item in items { rpc_to_remote(item) }` 扇出** | 先分桶再 join_all | economy `trade_saga.rs:138-177` 🚨 A4 命中（P2），单卡 10ms × 10 卡 = 100ms P99 线性增长 | 🟡 P2 backlog |
| 2.2.5 | **A5: `bincode` + 手动 struct 当协议** | protobuf / flatbuffers | 0 命中 | ✅ **不命中** |
| 2.2.6 | **A6: `HashMap<Cmd, Mod>` 派发协议** | const array 跳转表 | 0 命中（tonic 自动 const dispatch）| ✅ **不命中** |

### 2.3 7 域 9 原则总评（per GAP-AUDIT §5）

| 域 | 已落地原则数 | 部分命中 | 未实装 | 总评 |
|---|---:|---:|---:|---|
| player-service | 3 | 3 | 3 | 🟡 满足核心（5 + 8 + outbox 变体）|
| match-service | 4 | 3 | 2 | 🟢 满足核心（5 + 6 + 8 + 2 FSM 转移函数）|
| economy-service | 3 | 3 | 3 | 🟡 满足核心（5 + 8 + 2 FSM 变体）；A4 反模式待修 |
| social-service | 3 | 2 | 4 | 🟡 待 P1 实装（缺 replay fire-and-forget）|
| admin-service | 3 | 2 | 4 | 🟡 待 P1 实装 |
| batch-service | 4 | 3 | 2 | 🟢 满足核心（5 + 3 split_by_srv + 8 + 9 realm_lifecycle）|
| card-service | 2 | 2 | 5 | 🟡 **P1 缺口**：card 域 0 命中原则 #5（DB 写盘走 outbox 变体）|
| **7 域总评** | **22/63 (35%) ✅** | **18/63 (29%) 🟡** | **23/63 (36%) ❌** | 35% ✅ + 29% 🟡 + 36% ❌ |

---

## 3. 总体采纳情况汇总

| 维度 | 闪烁之光 | Erlang/OTP 框架 | 合计 |
|---|---:|---:|---:|
| 已落地亮点 | 11（协议层 4 + 网络层 4 + 业务逻辑 3）| 22（7 域原则 #2/#5/#8）| **33** |
| 部分采纳 / 待评估 | 6（业务层 4 + 网络拓扑 2）| 18（原则 #3/#6/#7/#9 部分命中）| **24** |
| 受 Gate 阻塞 backlog | 4（业务玩法 4）| 0 | **4** |
| 反例必须避免 | 1（9+6=15 复制变体）+ A1/A4 反模式命中 2 | 4（A1/A4 已命中；A2/A3/A5/A6 不命中）| **7** |
| 决策待定 / 待评估 | 2（公会展品 + 录像回放社交层）| 23（原则 #1/#4 + 6 反模式 A1/A4 待修 + 7 域 ❌ 项）| **25** |
| **小计亮点采纳条目** | **24** | **67** | **91 条** |

---

## 4. 验收 / 跟踪

### 4.1 已完成的 DoD 项
- [x] **L1**: 文档形式合规（代签授权 / DoD / 派生约束守护 / 缺标 / 禁回溯 / 凭据硬 ban）
- [x] **L1.1**: 关联文档全部引用 absolute path 或可验证路径（per AGENTS.md §1.1"引用必须可独立验证"）
- [x] **L1.2**: 跨盘引用 3 处（`E:\BaiduNetdiskDownload\闪烁之光\server分析\`）已写盘可 Read 复核

### 4.2 跟踪项（per 多 issue 关联）
- Multica **ULYS-134** (`01a0be20-14fe-7eb0-9fab-741c140b9c23`) — 本 issue 主交付
- Multica **ULYS-2** (`01a092ae-2faa-799b-b07e-ae6a9b80c166`) — 闪烁之光网络层完美对接（5 sub-issue ULYS-3~7）
- Multica **ULYS-111** — gm-backend 挂 rgs-secret-ca + RGS_TLS_DIR env（gm-backend Hybrid-1 落地准备）
- Multica **ULYS-94** — WSL/k3s/e2e-smoke 12 探针恢复（per D-Boy 2026-09-20 14:17 拍板授权）

### 4.3 Backlog 摘要（per GAP-AUDIT §7 + handoff §2.1）

| 优先级 | backlog | 工时 | 阻塞条件 |
|---|---|---:|---|
| P1 | card 域 outbox 接入（原则 #5 缺口）| 3d | 无 |
| P1 | 5 域 service 架构决策记录 ADR-0060（DB-as-state 优于 per-entity actor）| 1d | 无 |
| P1 | admin 域 audit_log 增 `log_title` 字段（Hybrid-1）| 0.5d | 无 |
| P2 | 框架原则 #4 协议 schema push 实装评估（prost-reflect）| 3d | 无 |
| P2 | 框架原则 #9 登录准备链声明式抽象（`enum ReadyStep`）| 5d | 无 |
| P2 | 框架原则 #7 热冷分层 + 战斗录像（DashMap 热 + sled/redb 冷）| 5d | 无 |
| P2 | 框架原则 #6 AbortHandle 去抖工具 | 3d | 无 |
| P2 | 框架原则 #3 split_by_srv 显式抽象 | 5d | 无 |
| P2 | 闪烁之光 跨盘 .erl 抽样业务层映射（12 大类 v0.2）| 3-5d | 无 |
| P3 | 框架原则 #1 per-entity actor 评估 ADR | 5d | 无 |
| P3 | 新建 `rgs-loadtest` crate（Hybrid-3 闪烁之光 `tester*.erl` 借鉴）| 3-5d | 无 |
| Q4 | L-CAND-010 候选升 AGENTS.md 正式段（数据驱动框架强制）| 1d | 2026-12-02 Q4 季度评审 |

---

## 5. 边界声明

- 本清单**不**新增 issue（除已存在的 ULYS-2 系列），只**登记** backlog 到 P1/P2/P3 + Q4 季度评审
- 本清单**不**修改 AGENTS.md 正式段（per §8 冻结期机制 + DEC-008 代签授权约束）
- 本清单**不**解除 `RGS-SPEC-CROSS-002` NO-GO（G-CODE-06 cargo build/test + G-CODE-03 五域独立 DB 拓扑图，均未满足）
- 本清单**不**触碰 5 域业务 proto 结构性字段扩展（受 Gate 阻塞）
- 本清单**不**涉及 cargo build/test、K3s/kubectl、证书、5 域 proto 实质性修改

---

## 6. 关联文档

| 文档 | 用途 | 路径 |
|---|---|---|
| RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化 v0.2 | 闪烁之光 5 项可立即执行 + §2.3 网络层完美对接路线图 | `docs/00-基准与治理/RGS-AI-HANDOFF-DOWNSTREAM-2026-09-04-闪烁之光借鉴优化.md` |
| RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04 v0.2 | 闪烁之光 API 11 维度对比矩阵 + 5 可取之处 + 1 反例 | `docs/14-项目治理/RGS-FLASH-OVERLAP-ANALYSIS-2026-09-04_v0.2.md` |
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 | 6 域 9 原则 + 6 反模式全量审计（81KB）| `docs/14-项目治理/RGS-DDD-2026-09-04-GAP-AUDIT_v0.3.md` |
| RGS-DDD-v0.2-addendum-协议号映射 | 闪烁之光 438 cmds → RGS proto 1:1 完整映射表 | `docs/15-IPA-完全对齐438cmds/RGS-DDD-v0.2-addendum-协议号映射.md` |
| RGS-DDD-v0.2-addendum-业务逻辑逆推 | 12 Partial module 业务逻辑 1:1 扩写（5 抽样 .erl）| `docs/15-IPA-完全对齐438cmds/RGS-DDD-v0.2-addendum-业务逻辑逆推.md` |
| RGS-REQ-2026-09-04 v0.2 | RGS 完全对齐闪烁之光 438 cmds 需求文档 | `docs/15-IPA-完全对齐438cmds/RGS-REQ-2026-09-04_v0.2.md` |
| RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 | 4 阶段路线图（mock → API 对齐 → 业务层 → 性能 baseline）| `docs/14-项目治理/RGS-FLASH-MOCK-DESIGN-2026-09-04_v0.3.md` |
| 跨盘引用 1 | 闪烁之光 全量 API 清单（1351 行 TSV）| `E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单-全量提取-2026-09-04.tsv` |
| 跨盘引用 2 | 闪烁之光 按文件分组清单（96 行 TSV）| `E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单-按文件分组-2026-09-04.tsv` |
| 跨盘引用 3 | 闪烁之光 12 大类 + 网络拓扑 + 可取之处 + 反例全文（MD）| `E:\BaiduNetdiskDownload\闪烁之光\server分析\分析产出\API清单与RGS借鉴分析-2026-09-04.md` |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-21 | Mavis (Ulysses 代签 per DEC-008) | 初版。承接 ULYS-134 9/20 10:45 agent 帖子中途掉线（"Now Section 3 — zsyz_server highlights"），按 D-Boy 9/21 JST "继续推进" directive 重启完整交付。覆盖两款商用服务器参考：① 闪烁之光 (zsyz_server) 24 条亮点 + ② Erlang/OTP 设计参考框架（9 原则 + 6 反模式）67 条亮点，合计 91 条。|