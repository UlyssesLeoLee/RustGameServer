# 卡牌游戏适配 / 需求定义书

**RustGameServer 卡牌游戏 (TCG / 休闲 / 集换) 需求定义**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-038 |
| 版本 | 0.1 (草案) |
| 制定日期 | 2026-08-29 |
| 制定人 | 架构师 (Mavis 接手 agent per DEC-008) |
| 上游依据 | RGS-REQ-001 v1.5 §4 业务需求 + §5 功能需求 + §6 性能需求 + RGS-REQ-013 v1.4 横切关注点 + Ulysses 2026-08-29 12:30 JST 决策 |
| 关联下游 | RGS-DTL-038 (待写) / RGS-BAS-038 (待写) / gm.proto v0.4+ / match.proto v2 / player.proto v2 / common.proto v2 / card.proto v1 (新域) |
| 状态 | 草案, 待 DDD Review 拍板 |

---

## 修订历史

| 版本 | 修订人 | 修订日期 | 修订内容 | 影响范围 |
|---|---|---|---|---|
| 0.1 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 初版草案, 覆盖 TCG + 休闲 + 集换 3 类卡牌游戏 | 全文 |

> 本文档为卡牌游戏适配的**业务 + 用户级需求**, 不替代 RGS-REQ-001 通用游戏服务器总需求 (本系统仍是 RGS, 卡牌游戏是其首个具体游戏类型适配).

---

## 目录

1. 背景与范围
2. 卡牌游戏类型与术语
3. 业务需求 (BR)
4. 用户级功能需求 (FR)
5. 性能 / 容量需求 (PR)
6. 安全 / 合规需求 (SR)
7. 兼容性需求 (CR)
8. 验收标准
9. 风险与未决项
10. 关联文档

---

# 1. 背景与范围

## 1.1 背景

RGS (RustGameServer) 当前已交付 8 域微服务 + gm-backend + 工具集, 累计 447+ UT/IT 跑测通过. 8 域架构是**通用游戏服务器骨架**, 不绑定具体游戏类型.

首个具体适配目标 = **卡牌游戏**, 包含 3 个子类:
- **TCG / CCG** (Trading Card Game / Collectible Card Game, 例: 炉石传说 / MTG Arena / 影之诗 / 百闻牌)
- **休闲卡牌** (例: 斗地主 / 桥牌 / UNO / 三国杀)
- **集换式卡牌** (例: PTCG / Duel Links, 强调卡牌收藏与交易)

> 文档目标: 在不破坏 RGS 通用性的前提下, 给出卡牌游戏适配的**完整需求边界**, 供下游 DTL-038 详细设计与 BAS-038 基本设计引用.

## 1.2 范围

**In-Scope (本次适配必须覆盖)**:
- 8 域 proto 兼容性 (player / match / social / economy / admin / gm / shared / cluster-ops)
- 卡牌游戏 4 大核心域: 玩家 + 卡组 + 对战 + 卡牌数据
- 通用 session / turn / move 抽象 (抽象层, 不绑定具体游戏规则)
- 卡组管理 (deck CRUD + 分享 + 收藏)
- 实时对战状态机 (turn-based, PVE + PVP)
- 匹配 (天梯 / 休闲 / AI / 房间)
- 货币与卡包 (买卡包 / 开包 / 抽卡)
- 排行榜 (天梯 / 休闲 / 集换)
- 战斗回放 (replay storage + fetch)
- 卡牌交易 / 拍卖 (集换式)
- GM 工具覆盖 (封号 / 补偿 / 维护 / 审计)

**Out-of-Scope (本次不适配)**:
- 具体游戏规则引擎 (炉石 / MTG / 影之诗 等具体卡牌效果执行) — 留给各游戏方在 game-logic 业务层实现
- 客户端 UI / 美术 / 音频
- 卡牌数值平衡 / 抽卡概率
- 运营活动 / 赛季 / 通行证
- 跨游戏账号互通 (RGS 通用层提供, 本文档不重复)

## 1.3 上游决策依据

- **2026-08-29 12:30 JST** Ulysses: "确保 API 可以适应卡牌游戏"
- **2026-08-29 12:31 JST** Ulysses: 拍板 proto 实装 + TCG+休闲+集换 全范围
- **2026-08-29 13:18 JST** Ulysses: 强化 "如果缺少需求和设计文档, 请补全之后再实施"

---

# 2. 卡牌游戏类型与术语

## 2.1 三类卡牌游戏对比

| 维度 | TCG / CCG | 休闲卡牌 | 集换式卡牌 |
|---|---|---|---|
| 核心循环 | 组牌 → 对战 → 调整 | 快速对战 → 升级 / 奖励 | 抽卡 → 收藏 → 交易 → 组牌 |
| 卡组大小 | 30-60 张 | 固定 (54 张 / 17 张) | 40-60 张 |
| 比赛模式 | 排位 / 休闲 / 锦标赛 | 休闲 / 房间 / 比赛 | 排位 / 休闲 / 交易 |
| 状态机复杂度 | 高 (效果链 / 触发) | 中 (固定规则) | 高 (类似 TCG) |
| 实时性 | turn-based, 秒级响应 | turn-based / 实时 (UNO) | turn-based |
| 卡牌获取 | 卡包 / 合成 / 活动 | 固定卡组 | 卡包 / 抽卡 / 交易 |
| 经济系统 | 强 (卡包 / 钻石 / 金币) | 弱 (体力 / 奖励) | 极强 (稀有卡交易) |
| 回放 | 必须 (排位回放) | 可选 | 必须 (争议申诉) |

## 2.2 关键术语

| 术语 | 英文 | 含义 |
|---|---|---|
| 卡组 | Deck | 玩家在单场对战使用的卡牌集合 |
| 收藏 | Collection | 玩家拥有的全部卡牌 (含未编入卡组的) |
| 卡包 | Pack | 抽卡包, 含 N 张随机卡 |
| 对战 | Match | 一次完整对局, 由多个 turn 组成 |
| 回合 | Turn | 对战中一个玩家的一次操作周期 |
| 操作 | Move | 回合内的单个动作 (出牌 / 攻击 / 结束回合) |
| 战牌 | Board | 对战双方的场上状态 (手牌 / 战场 / 墓地) |
| 战报 | Replay | 对战的完整操作日志, 可重放 |
| 卡牌数据 | Card | 卡牌的静态定义 (id / name / cost / type / effect) |
| 卡牌实例 | CardInstance | 玩家收藏中的一张具体卡牌 (含稀有度 / 等级) |
| 天梯 | Ranked | 排位赛, 影响段位积分 |
| 休闲 | Casual | 非排位, 仅影响个人战绩 |
| 房间 | Room | 玩家自建 / 邀请制对战, 可配置规则 |
| 拍卖 | Auction | 集换式卡牌的公开 / 私密交易 |

---

# 3. 业务需求 (BR)

## BR-001 多模式对战

RGS 应支持至少 4 种对战模式:
- **天梯 (Ranked)**: 段位匹配, 影响积分, 强制回放记录
- **休闲 (Casual)**: 快速匹配, 不影响积分
- **房间 (Room)**: 自建 / 加入, 可配置规则
- **AI 试玩 (PvE)**: 单人对战机器人, 用于新手引导

**验收**: 4 种模式均能匹配成功并完成完整对局.

## BR-002 卡组管理

玩家应能:
- 创建 / 删除 / 重命名卡组 (上限 N 个, 初始 N=10, 未来可调)
- 编辑卡组内容 (拖拽 / 替换 / 排序)
- 校验卡组合法性 (per 规则引擎, 留给业务层)
- 分享卡组 (好友 / 公开)
- 收藏 (单卡 + 整套) 查看

**验收**: 玩家能 CRUD 至少 5 个卡组, 分享 / 收藏可见.

## BR-003 抽卡 / 卡包

- 购买卡包 (消耗货币: 软通 / 硬通)
- 开包动画 (数据层, UI 由客户端)
- 抽卡概率公开 (per 法律要求, 部分地区)
- 卡牌入库收藏 (transactional)

**验收**: 购买 → 开包 → 入库 全链路事务一致, 失败可回滚 (saga).

## BR-004 排行榜

- 天梯段位 + 积分
- 休闲胜率榜
- 集换价值榜 (按收藏价值)
- 周榜 / 月榜 / 季榜 / 历史榜

**验收**: 排行榜可分页 / 过滤 / 实时刷新.

## BR-005 战斗回放

- 完整记录: 操作日志 + 战牌状态快照
- 存储周期: 天梯 90 天, 休闲 7 天, 房间自定义
- 复盘支持: 玩家可播放历史对局
- 申诉支持: GM 可调取任意回放

**验收**: 玩家能查看自己 30 天内所有对局回放.

## BR-006 卡牌交易 (集换式)

- 公开拍卖 (上架 / 出价 / 成交)
- 私下交易 (玩家间, 通过 inbox 协议)
- 交易税 (per 经济系统配置)
- 交易审计 (GM 工具可查)

**验收**: 玩家能上架 / 购买卡牌, 货币 / 卡牌原子转移.

## BR-007 货币体系

- 软通 (Soft Currency, 玩法产出)
- 硬通 (Hard Currency, 充值 / 活动)
- 卡牌价值 (Card Value, 集换式才有, per 拍卖市场)
- 货币流水审计 (per 经济系统 + GM 工具)

**验收**: 3 类货币 CRUD + 流水可查.

## BR-008 GM 工具覆盖

gm-backend 5 endpoint 必须支持:
- **封号**: 玩家级 / 设备级, 包含对战中踢出
- **补偿**: 货币 / 卡牌 / 卡包, 按账号 / 全服 / 活动
- **维护**: 卡牌游戏专属维护模式 (天梯冻结 / 交易冻结)
- **审计**: 抽卡 / 交易 / 对战 GM 操作可查

**验收**: 5 endpoint 均通过端到端 IT, 见 gm-backend §3.

## BR-009 多语言

卡牌游戏文本 (卡牌描述 / UI / 公告) 必须支持多语言:
- zh-CN / en-US (基础)
- ja-JP / ko-KR (扩展, 视市场)
- 文案存储独立于代码 (i18n key → value)

**验收**: 卡牌 catalog 文本字段支持语言切换.

---

# 4. 用户级功能需求 (FR)

## FR-001 玩家账号

- 注册 / 登录 (per 现有 player-service v1)
- 资料编辑 (昵称 / 头像 / 简介)
- 多设备登录 (per 现有会话管理)
- **新增**: 卡牌游戏资料 (段位 / 收藏数 / 战绩)

**RPC** (player-service v2 新增):
- `GetPlayerProfile` — 获取玩家卡牌资料
- `UpdatePlayerProfile` — 更新资料

## FR-002 卡组管理

**RPC** (player-service v2 新增, **或** card-service v1 新域):
- `CreateDeck` — 创建空卡组
- `GetDeck` — 读取卡组
- `UpdateDeck` — 编辑卡组 (增删改卡)
- `DeleteDeck` — 删除卡组
- `ListDecks` — 列出玩家所有卡组
- `ShareDeck` — 分享卡组 (生成短链 / 好友 ID)
- `GetSharedDeck` — 通过分享码 / 好友 ID 拉取卡组

> **决策点** (待 DDD Review 拍板): 卡组归属 player-service 还是新 card-service? 详见 RGS-DTL-038 §3.

## FR-003 卡牌数据

**RPC** (新 card-service v1, 静态 + 慢变数据):
- `GetCard` — 单卡数据
- `ListCards` — 卡牌列表 (分页 / 过滤 by type/rarity/series)
- `GetCardSeries` — 卡包 / 系列元数据
- `GetPlayerCollection` — 玩家收藏 (按卡包 / 稀有度)
- `AddCardToCollection` — 入库 (仅服务端内部 / 抽卡结果)

**实体**:
- `Card` (id, name_i18n, cost, type, rarity, series_id, effect_ref)
- `CardSeries` (id, name, pack_size, drop_table, price)
- `CardInstance` (instance_id, owner_id, card_id, acquired_at, source, level, attrs)

## FR-004 对战 session

**RPC** (match-service v2 新增):
- `CreateMatch` — 创建对战 (per 模式 / 规则)
- `JoinMatch` — 加入
- `LeaveMatch` — 离开 / 投降
- `GetMatchState` — 查询当前状态
- `SubmitMove` — 提交操作
- `SubscribeMatch` — 流式订阅对战事件 (per 现有 event-sourcing / outbox)

**实体**:
- `GameSession` (id, mode, players, state, turn_index, deadline_ms)
- `Move` (move_id, session_id, player_id, type, payload, result, occurred_at)
- `Board` (session_id, snapshot_json, version) — 战牌快照

## FR-005 匹配

**RPC** (match-service v2 新增, per 现有 matchmaker.rs):
- `EnqueueMatchmaking` — 入队
- `CancelMatchmaking` — 取消
- `GetMatchmakingStatus` — 查询
- `MatchFound` — 客户端订阅

**规则**:
- 天梯: ELO / TrueSkill, 段位匹配
- 休闲: 随机 / 等级匹配
- 房间: 邀请码 / 房主邀请
- AI: 立即返回机器人 session

## FR-006 货币 / 经济

**RPC** (economy-service v2 新增):
- `GetAccount` (已有) — 读取玩家账户
- `AddCurrency` (内部 / saga) — 加货币
- `DebitCurrency` (内部 / saga) — 扣货币
- `TransferCurrency` (交易税) — 转账
- `GetTransactionLog` — 流水查询 (审计)

> 当前 economy.proto 仅 GetAccount + HealthCheck, **需扩展** (per RGS-DTL-038 §3 + W29 链路 C 实施).

## FR-007 排行榜

**RPC** (新 leaderboard-service, **或** 复用 match-service 排行榜子模块):
- `GetRankedLeaderboard` — 天梯榜
- `GetCasualLeaderboard` — 休闲榜
- `GetCollectionLeaderboard` — 集换价值榜
- `GetPlayerRank` — 玩家自己在榜位置

## FR-008 战斗回放

**RPC** (新 replay-service, **或** 复用 cluster-ops 对象存储):
- `SaveReplay` — 内部调用, 对战结束时入库
- `GetReplay` — 拉取回放
- `ListReplays` — 列出玩家回放
- `StreamReplay` — 流式播放 (events over gRPC stream)

**存储**:
- 热数据: 30 天 (PostgreSQL + S3-兼容)
- 冷数据: 30 天后归档 (per cluster-ops archive_policy)

## FR-009 卡牌交易 (集换式)

**RPC** (新 trade-service, **或** economy-service v2):
- `ListAuction` — 公开拍卖列表
- `CreateAuction` — 上架
- `BidAuction` — 出价
- `CancelAuction` — 撤单
- `ExecuteTrade` — 私下交易 (per inbox 协议)
- `GetTradeHistory` — 交易历史

## FR-010 GM 工具

per gm.proto v0.3 (已有), 5 endpoint 覆盖:
- `BanAccount` — 含对战踢出 (新增强制参数 force_disconnect_session)
- `GrantCompensation` — 含卡牌 / 卡包发放
- `SetMaintenance` — 含天梯冻结 / 交易冻结 (新增 mode_flags)
- `QueryAuditLog` — 含抽卡 / 交易审计

> gm.proto v0.4+ 计划 (per RGS-DTL-038 §3.4).

## FR-011 i18n

**RPC** (新 i18n-service, **或** 复用 shared-platform config):
- `GetText` — 单 key 拉取
- `GetTexts` — 批量拉取
- `ListLanguages` — 列出支持语言

> **决策点** (待拍板): i18n 数据静态化 (build-time 嵌入) vs 动态 (DB / Redis).

---

# 5. 性能 / 容量需求 (PR)

## PR-001 单服务 QPS

| 域 | 目标 QPS | 备注 |
|---|---|---|
| player-service | 5,000 | 高频读, 写较少 |
| match-service | 10,000 (撮合) + 50,000 (session 状态查询) | session 状态查询非常频繁 |
| card-service | 8,000 (catalog 读) + 500 (collection 写) | catalog 缓存友好 |
| economy-service | 3,000 (saga 提交) + 20,000 (流水查询) | saga 是关键路径 |
| leaderboard-service | 20,000 (查询) + 200 (落榜写) | 写稀疏, 读密集 |
| replay-service | 500 (回放读) + 100 (写) | 写有, 不密集 |
| trade-service | 1,000 (撮合) + 5,000 (出价) | 撮合是热点 |

## PR-002 延迟

- **session 操作** (SubmitMove → Broadcast): < 100ms P99
- **catalog 读取**: < 20ms P99 (缓存命中 < 5ms)
- **匹配撮合**: < 3s P95 (入队 → 撮合成功)
- **抽卡 (开包)**: < 200ms P99 (saga 完成)
- **回放流式加载**: < 200ms 首帧
- **交易撮合**: < 500ms P99

## PR-003 并发对局

- 单 k3s 节点: 50,000 并发 session
- 集群 5 节点: 250,000 并发 session
- 每 session 平均 30 回合, 每回合 1.5 操作 = 45 操作 / session
- 操作吞吐: 250K session × 45 / 30min 平均对局 = 6,250 ops/s (轻载), 高峰 30K ops/s

## PR-004 存储容量 (5 节点集群, 1 年)

| 数据 | 估算 | 增长速率 |
|---|---|---|
| 玩家账号 | 10M × 5KB = 50GB | 稳态 |
| 卡牌收藏 | 10M × 200 卡 × 200B = 400GB | 增长 |
| 对战 session (热) | 250K × 50KB = 12.5GB / 实时, 30 天滚动 | 持续 |
| 战斗回放 (热) | 10M 对局 / 天 × 100KB = 1TB / 天, 30 天 = 30TB | 持续 |
| 流水 (经济) | 100M / 天 × 500B = 50GB / 天 | 持续 |
| 卡牌 catalog | 10K × 5KB = 50MB | 准静态 |
| 排行榜 | 10M × 100B = 1GB | 稳态 |
| 交易记录 | 1M / 天 × 1KB = 1GB / 天 | 持续 |

## PR-005 可用性

- 单服务 RTO: < 30s (per 现有 HPA + cluster-ops)
- 整体 RTO: < 60s
- 数据持久性: 99.999% (per existing NFR-OP-005)

---

# 6. 安全 / 合规需求 (SR)

## SR-001 抽卡概率公开

集换式卡牌游戏的部分地区 (中国 / 日本) 法律要求公开抽卡概率. RGS 应:
- 抽卡结果包含概率快照 (drop_table_snapshot)
- GM 工具可审计

## SR-002 未成年人保护

- 防沉迷 (per 国家法律, 中国 18 岁以下限时)
- 消费上限 (per 监管, 单日 / 单月)
- 退款机制 (per 法律)
- 实名认证 (per 现有 player-service 扩展)

## SR-003 反作弊

- 客户端操作签名 (per 现有 JWT 框架)
- 操作频率限制 (per session / per player)
- 异常模式检测 (per GM 工具 audit, 集成)

## SR-004 数据隐私

- 卡组 / 收藏 / 战绩 = 个人数据, 加密存储
- GM 工具访问留痕
- GDPR / 个保法合规 (per RGS-REQ-013 §3 横切关注点)

## SR-005 货币合规

- 充值实名 + 限额 (per 监管)
- 抽卡流水不可篡改 (per 区块链式 hash 链, 简化: append-only DB + 审计)

---

# 7. 兼容性需求 (CR)

## CR-001 8 域 proto 兼容

新需求不破坏现有 8 域的 proto v1 兼容:
- player.proto v2 (FR-001) — 新增 RPC, 不改老 RPC
- match.proto v2 (FR-004, FR-005) — 新增 RPC, 老 GetMatch / HealthCheck 保留
- economy.proto v2 (FR-006) — 扩展, 老 GetAccount 保留
- gm.proto v0.4+ (FR-010) — 扩展, 老 5 endpoint 保留 (per 桶 2a 实装 106 IT)
- common.proto v2 (FR-009 i18n 等) — 新增字段, 老字段保留

## CR-002 数据库 schema 兼容

- PostgreSQL: 新表 (cards / card_instances / decks / game_sessions / moves / replays / trades) 不破坏现有 35+ 张表
- 现有索引 / 约束保留
- migration 用 sqlx 现有工具链

## CR-003 跨语言客户端 SDK

- 现有 Unity / Unreal / Godot SDK (per 路径基准) 不破坏
- 新增 card / match / replay 模块的 SDK 同步更新

## CR-004 跨域 gRPC 兼容

- 现有 gm → admin mTLS 链路 (per BAS-003 决策补充) 不变
- 新增 gm → match / gm → card / gm → leaderboard 链路沿用 mTLS + JWT 模式

---

# 8. 验收标准

## 8.1 阶段验收 (per WBS token 桶 8 桶)

| 阶段 | 验收标准 | Token 估 |
|---|---|---|
| 桶 7 proto v0 设计 | 4 份 proto 草案 (common v2 / match v2 / player v2 / card v1) + 1 份本设计文档 | 8M |
| 桶 8 proto v1 实装 | 4 份 proto v1 落档 + 6 域编译通过 + 25+ UT | 18M |
| 桶 9 session/turn 抽象 | match-service v2 + session 状态机 + 30+ IT | 25M |
| 桶 10 card catalog | card-service v1 + collection CRUD + 20+ IT | 18M |
| 桶 11 deck + share | deck CRUD + 分享 + 合法性校验 + 15+ IT | 12M |
| 桶 12 leaderboard | 排行榜 4 类 + 分页 + 实时刷新 + 10+ IT | 8M |
| 桶 13 replay | replay 存储 + 播放 + 申诉接口 + 12+ IT | 15M |
| 桶 14 trade + gm 扩展 | 拍卖 + 私下交易 + gm v0.4 5 endpoint 扩展 + 18+ IT | 25M |
| **合计** | **8 桶全过, 累计 130+ 新 IT, 总估 129M tokens** | **129M** |

> **注**: 阶段桶编号沿用 WBS v0.5 §7 6 桶 后续追加. 当前累计 31M 余额, 需 129M - 31M = **追加 98M tokens 预算**.

## 8.2 最终验收 (PRD 完整)

- 3 类卡牌游戏 (TCG / 休闲 / 集换) 端到端跑通
- 10M 玩家 / 250K 并发 / 6,250 ops/s 压测通过
- 8 域 proto v2 全部上线, 老客户端 SDK 兼容
- 9 维度 AI 审计 CI (per W29 桶 6) PASS
- 6 桶原 WBS + 8 桶卡牌 = 14 桶全过

---

# 9. 风险与未决项

## 9.1 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 抽卡事务一致性 | P1 | economy-service saga 已就位, 卡牌入库走 AddCardToCollection saga |
| 大规模 session 状态同步 | P1 | event-sourcing + outbox (已就位), per W25 Step 3 集成 |
| 卡牌效果执行 (规则引擎) | P2 | 留给业务层 (game-logic), RGS 只承载数据流 |
| 多语言文案 | P2 | i18n 服务可独立实装, 不阻塞核心 |
| 反作弊 | P1 | 现有 JWT + 限流 + audit, 卡牌游戏沿用 |

## 9.2 未决项 (待 DDD Review 拍板)

| ID | 决策点 | 候选 |
|---|---|---|
| DEC-038-01 | 卡组归属 | player-service v2 / 新 card-service v1 / 独立 deck-service |
| DEC-038-02 | leaderboard 域 | 新 leaderboard-service / match-service 子模块 / reuse shared-platform redis |
| DEC-038-03 | replay 存储 | cluster-ops 对象存储 / 新 replay-service / S3-兼容外部 |
| DEC-038-04 | trade 域归属 | economy-service v2 / 新 trade-service / 复用 saga |
| DEC-038-05 | i18n 静态 vs 动态 | build-time 嵌入 / Redis 缓存 / DB |
| DEC-038-06 | 抽卡概率公开 | 强制 / 可选 / 关闭 |
| DEC-038-07 | gm.proto v0.4 时机 | W14 / W15 / W18 / 8 桶后 |
| DEC-038-08 | 8 桶新 WBS 排序 | 按 token 桶 / 按 5 域 RACI / 按 业务关键路径 |

---

# 10. 关联文档

- **RGS-REQ-001** 需求定义书 v1.5 (本系统总需求, 必读 §4 业务 / §5 功能)
- **RGS-REQ-013** 体系治理与横切关注点 v1.4 (i18n / 性能 / 安全)
- **RGS-BAS-003** 运维与 GM 后台管控基本设计书 (gm.proto 上游)
- **RGS-BAS-003-mTLS-决策补充-v0.1** mTLS 范围 (gm→admin)
- **RGS-DTL-038** 卡牌游戏适配详细设计书 (待写)
- **RGS-BAS-038** 卡牌游戏适配基本设计书 (待写)
- **gm.proto v0.3** GM 后台协议 (per 桶 2a 实装)
- **RGS-PLAN-WBS-token-bucket-v0.5** 6 桶 WBS (后续追加 8 桶)

---

## 制定 / 审批

| 角色 | 姓名 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 制定 (架构) | 架构师 (Mavis 接手 agent per DEC-008) | ✓ | 2026-08-29 | 初版草案 |
| 审批 (技术) | — | ⏳ | — | 待 DDD Review |
| 审批 (业务) | — | ⏳ | — | 待 Ulysses |
| **最终决策 (产品)** | **Ulysses** | ⏳ | — | 待拍板 9.2 8 未决项 |

> 本文档**已为下一阶段 RGS-DTL-038 详细设计**准备就绪. 详细设计将基于本需求给出: 域归属 / proto v2 完整 message / session 状态机图 / saga 编排 / 数据库 schema 草图.
