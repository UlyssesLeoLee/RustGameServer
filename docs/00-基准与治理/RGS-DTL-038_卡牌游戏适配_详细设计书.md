# 卡牌游戏适配 / 详细设计书

**RustGameServer 卡牌游戏 (TCG / 休闲 / 集换) 详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-038 |
| 版本 | 0.1 (草案) |
| 制定日期 | 2026-08-29 |
| 制定人 | 架构师 (Mavis 接手 agent per DEC-008) |
| 上游依据 | RGS-REQ-038 卡牌游戏适配需求定义书 v0.1 |
| 关联下游 | RGS-BAS-038 (待写) / gm.proto v0.4+ / match.proto v2 / player.proto v2 / common.proto v2 / card.proto v1 / leaderboard.proto v1 / replay.proto v1 / trade.proto v1 / i18n.proto v1 |
| 状态 | 草案, 待 DDD Review 拍板 (9 个 DEC) |

---

## 修订历史

| 版本 | 修订人 | 修订日期 | 修订内容 | 影响范围 |
|---|---|---|---|---|
| 0.1 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 初版草案, 含 9 DEC 候选 + 4 proto 草稿 + 1 session 状态机 + 1 saga 编排 | 全文 |

---

## 目录

1. 范围与上游
2. 架构总览
3. 域归属 9 决策 (DEC-038-01 ~ 09)
4. proto v2 草稿
5. session / turn 状态机
6. saga 编排
7. 数据库 schema 草图
8. 8 桶 WBS 排序建议
9. 风险与未决项
10. 关联文档

---

# 1. 范围与上游

## 1.1 上游文档

- **RGS-REQ-038 v0.1** 卡牌游戏适配需求定义书 (本文档上游)
- **RGS-REQ-001 v1.5** 系统总需求 §4 业务 / §5 功能 / §6 性能
- **RGS-REQ-013 v1.4** 横切关注点 (i18n / 安全 / 性能)
- **RGS-BAS-003** GM 后台基本设计 (gm.proto v0.3 上游)
- **RGS-BAS-003-mTLS-决策补充-v0.1** mTLS 范围 (gm→admin)
- **RGS-PLAN-WBS-token-bucket-v0.5** 6 桶 WBS (后续追加 8 桶)

## 1.2 本文范围

详细设计 **不重复** 需求文档, 重点是:
- **架构决策** (域归属 / 9 DEC 候选方案)
- **proto v2 message 草稿** (common / match / player / card 4 份)
- **session 状态机** (turn-based, 完整状态转移图)
- **saga 编排** (抽卡 + 交易 2 个关键 saga)
- **数据库 schema 草图** (新增 8 张表)
- **8 桶 WBS 排序建议** (per RGS-REQ-038 §8.1 8 桶)

## 1.3 不在本文范围

- 具体卡牌游戏规则 (炉石 / MTG / 影之诗 等) — 业务层 game-logic
- 客户端 SDK 内部实现 — 跨域 SDK 团队
- 运营活动 / 赛季 — 运营层

---

# 2. 架构总览

## 2.1 卡牌游戏专属域 (新增)

```
┌──────────────────────────────────────────────────────────┐
│           RGS 通用 8 域 (player / match / social /       │
│   economy / admin / gm / shared / cluster-ops)            │
└──────────────────────────────────────────────────────────┘
                          ↕ 内部 gRPC (mTLS + JWT)
┌──────────────────────────────────────────────────────────┐
│  卡牌游戏专属 6 域 (per DEC-038-01 ~ 06 决策)             │
│   - card-service       (catalog + collection, 静态)       │
│   - deck-service       (deck CRUD + share, OR 合 player) │
│   - leaderboard-service (ranked/casual/collection 4 榜)  │
│   - replay-service     (replay 存储 + 播放)              │
│   - trade-service      (拍卖 + 私下交易, OR 合 economy)  │
│   - i18n-service       (per DEC-038-05)                   │
└──────────────────────────────────────────────────────────┘
```

## 2.2 卡牌游戏跨域交互矩阵

| 起点 → 终点 | 触发场景 | 协议 | 安全 |
|---|---|---|---|
| gm-backend → match-service | 强制踢出 session | gRPC mTLS | per BAS-003 |
| gm-backend → card-service | 补偿卡牌 / 卡包 | gRPC mTLS | per BAS-003 |
| gm-backend → trade-service | 冻结交易 / 撤单 | gRPC mTLS | per BAS-003 |
| player-service → card-service | 抽卡 / 收藏 | gRPC mTLS | mTLS |
| match-service → card-service | 对战初始卡组加载 | gRPC mTLS | mTLS |
| match-service → replay-service | session 结束保存回放 | gRPC mTLS | mTLS |
| match-service → economy-service | 输赢结算 (saga) | gRPC mTLS | mTLS |
| trade-service → economy-service | 货币转移 (saga) | gRPC mTLS | mTLS |
| trade-service → card-service | 卡牌实例转移 (saga) | gRPC mTLS | mTLS |
| client → match-service | 提交 Move / 订阅 session | gRPC stream | JWT |
| client → card-service | catalog 读 / collection 读 | gRPC | JWT |
| client → leaderboard-service | 排行榜查 | gRPC | JWT |
| client → replay-service | 回放拉 / 流 | gRPC | JWT |

## 2.3 session / turn 抽象 (通用)

session 是卡牌游戏的核心抽象, 跨 3 类游戏都适用:
- **TCG/CCG**: turn-based, 复杂效果链
- **休闲卡牌**: turn-based 或 实时 (UNO), 固定规则
- **集换式**: 同 TCG

session 不绑定具体游戏规则, 只承载:
- 玩家列表 (2-N)
- 模式 (天梯 / 休闲 / 房间 / AI)
- 战牌状态 (Board snapshot)
- 回合索引 (turn_index)
- 操作日志 (Move log)
- 截止时间 (deadline_ms, per turn 超时机制)
- 状态机 (创建中 / 进行中 / 暂停 / 结束 / 异常)

具体游戏规则 = 业务层 game-logic crate, 通过 Move 提交触发, 写入 Board 快照.

---

# 3. 域归属 9 决策 (DEC-038-01 ~ 09)

> 每决策给 3 候选 + 推荐项 + 理由. 拍板由 DDD Review 决定.

## DEC-038-01 卡组归属 (RGS-REQ-038 §FR-002)

| 候选 | 优点 | 缺点 |
|---|---|---|
| **A. player-service v2 内置** ✅ 推荐 | 卡组是玩家强属性, 内置最紧; 无新域运维成本 | player-service 域变大, 拆分压力 |
| B. 新 card-service v1 内 | catalog + collection + deck 都归 card 域, 域职责清晰 | card 域变大, 业务耦合 |
| C. 独立 deck-service | 域职责最单一 | 6 域变 7 域, 运维成本 + 跨域调用 + 1 |

**推荐**: **A** — 卡组是玩家属性, 应归 player-service v2. card-service 只承担 catalog + collection.

**理由**: per RGS-DDD 8 域划分原则 (每个域 = 1 个微服务, 不轻易拆). 卡组是 player 的派生数据, 复用 player-service 既有权限 / 验证 / 缓存. 新增 deck-service 仅当卡组成为独立商品 (如交易卡组) 时才有价值.

## DEC-038-02 leaderboard 域

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. 新 leaderboard-service ✅ 推荐 | 域职责清晰, 高频读独立优化, 可独立扩缩 | 6 域变 7 域, 1 个新域运维 |
| B. match-service 子模块 | 复用 match 域基础设施, 减少域数 | match 域变重, 排行榜高频读拖累 match |
| C. 复用 shared-platform Redis 缓存 | 0 新域, 复用现有 | 不算独立域, 业务耦合在 shared |

**推荐**: **A** — 排行榜是高频读 + 低频写场景, 独立成域可专门优化 (Redis 排序集 / 写时排序). match-service 保持聚焦匹配撮合.

## DEC-038-03 replay 存储

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. cluster-ops 对象存储 ✅ 推荐 | 复用现有 k3s 对象存储, 0 新域, 成本低 | 对象存储延迟略高 (但卡牌游戏不敏感) |
| B. 新 replay-service (PostgreSQL + S3) | 域职责清晰, 可独立优化查询 | 7 域变 8 域, 存储双写复杂 |
| C. 外部 S3-兼容 (MinIO) | 业界标准, 易扩展 | 引入外部依赖, 当前不必要 |

**推荐**: **A** — cluster-ops 已就位 (per RGS-REQ-001 §7.3 cluster-ops 域), 复用对象存储. replay 元数据走 PostgreSQL, 回放数据走对象存储, 引用 ID 关联.

## DEC-038-04 trade 域归属

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. economy-service v2 内置 ✅ 推荐 | 经济是 trade 的天然组成, saga 复用 | economy 域变大 |
| B. 新 trade-service | 域职责清晰, 拍卖撮合可独立优化 | 6 域变 7 域, 跨域调用复杂 |
| C. 复用现有 inbox 协议 | 私下交易走 inbox 已有机制, 不新域 | 公开拍卖仍需独立 RPC |

**推荐**: **A** — trade 本质是经济活动, 沿用 economy-service v2, 把卡牌实例当作新型 "货币 / 资产". 公开拍卖撮合 = economy 内的子模块.

## DEC-038-05 i18n 静态 vs 动态

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. Redis 缓存 + DB 持久化 ✅ 推荐 | 支持运行时热更新, 跨实例共享, 1 个独立微服务 | 需维护多语言表 |
| B. build-time 嵌入 | 0 运行时依赖, 性能最优 | 改文案需重新发版 |
| C. 静态文件 (i18n/*.json) | 简单, 可托管到 CDN | 不支持热更新 |

**推荐**: **A** — 卡牌游戏运营频繁 (赛季 / 活动), 文案热更新必需. 独立 i18n-service 域, 缓存 5 分钟, 持久化 DB, 预热 Redis.

## DEC-038-06 抽卡概率公开

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. 强制公开 + drop_table_snapshot ✅ 推荐 | 合规 (中国 / 日本法律), GM 可审计 | 0 风险, 0 运营灵活度 |
| B. 可选 (per 监管) | 灵活 | 需 per 地区配置, 复杂 |
| C. 关闭 | 0 运营压力 | 合规风险 (中国 / 日本) |

**推荐**: **A** — 中国 / 日本法律要求, 强制公开. 抽卡结果含 drop_table_snapshot 字段, GM 工具可审计.

## DEC-038-07 gm.proto v0.4 时机

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. 卡牌 8 桶 完成后 (per RGS-REQ-038 §8.1 桶 14) ✅ 推荐 | 一次性覆盖 5 域扩展, gm.proto 稳态 | gm.proto 改动被卡牌需求阻塞 |
| B. 立即 (本周) | 解锁 gm 工具的卡牌能力, 不阻塞 | gm.proto 改动频繁, 不稳态 |
| C. 8 桶中段 (桶 10 card catalog 后) | 中间点, 平衡 | 仍可能二次改动 |

**推荐**: **A** — gm.proto v0.4 等卡牌 8 桶后期再升版, 避免反复改 gm.proto. 当前 gm.proto v0.3 106 IT 稳态, 不动.

## DEC-038-08 8 桶 WBS 排序

| 候选 | 优点 | 缺点 |
|---|---|---|
| A. 按 token 桶 (per RGS-REQ-038 §8.1 顺序) ✅ 推荐 | 与现有 6 桶排序一致, 可视化好 | 依赖卡牌 8 域核心, 启动慢 |
| B. 按 5 域 RACI | 责任清晰, 8 域 Lead 各自领桶 | 跨域协调多, 决策慢 |
| C. 按业务关键路径 (catalog → deck → match → trade) | 业务价值递进, 可演示 | 估时不均, 风险集中 |

**推荐**: **A** — 沿用现有 6 桶 token 排序方法, 8 桶依次为: proto 设计 / proto 实装 / session 抽象 / card catalog / deck / leaderboard / replay / trade+gm.

## DEC-038-09 总 token 预算追加

| 候选 | 备注 |
|---|---|
| **A. 追加 98M tokens** ✅ 推荐 | 当前余额 31M, 8 桶总估 129M, 追加 98M |
| B. 拆 2 阶段 | Phase 1 (4 桶, 60M) / Phase 2 (4 桶, 69M), 阶段性 |
| C. 砍 scope (7 桶) | 砍 1 桶 (选 1 非关键, e.g. leaderboard), 总估 121M |

**推荐**: **A** — 8 桶全过, 追加 98M tokens 预算. 沿用 WBS 6 桶节省 88% 经验, 8 桶估 129M 实际可能 100-110M, 余量充足.

---

# 4. proto v2 草稿

## 4.1 common.proto v2 新增 message

```proto
syntax = "proto3";
package common.v1;

// 既有 (v1 保留): Status / ErrorCode / EntityId / Timestamp / PageRequest / PageResponse / HealthCheckRequest / HealthCheckResponse

// v2 新增: 卡牌游戏通用抽象
enum Locale {
  LOCALE_UNSPECIFIED = 0;
  LOCALE_ZH_CN = 1;
  LOCALE_EN_US = 2;
  LOCALE_JA_JP = 3;
  LOCALE_KO_KR = 4;
}

message LocalizedText {
  Locale locale = 1;
  string text = 2;
}

message I18nString {
  // 默认 zh-CN, fallback en-US
  string default_text = 1;
  repeated LocalizedText translations = 2;
}

// 卡牌类型 (per RGS-REQ-038 §2.2)
enum CardType {
  CARD_TYPE_UNSPECIFIED = 0;
  CARD_TYPE_CREATURE = 1;   // 生物
  CARD_TYPE_SPELL = 2;      // 法术
  CARD_TYPE_EQUIPMENT = 3;  // 装备
  CARD_TYPE_LAND = 4;       // 地 (MTG)
  CARD_TYPE_TRAP = 5;       // 陷阱 (YGO)
  CARD_TYPE_HERO = 6;       // 英雄
}

enum CardRarity {
  CARD_RARITY_UNSPECIFIED = 0;
  CARD_RARITY_COMMON = 1;    // N
  CARD_RARITY_UNCOMMON = 2;  // R
  CARD_RARITY_RARE = 3;      // SR
  CARD_RARITY_EPIC = 4;      // SSR
  CARD_RARITY_LEGENDARY = 5; // UR
}

message CardRef {
  string card_id = 1;            // 静态 card.id
  string instance_id = 2;        // 玩家收藏 instance_id (若有)
}

// 卡牌游戏模式 (per RGS-REQ-038 §BR-001)
enum GameMode {
  GAME_MODE_UNSPECIFIED = 0;
  GAME_MODE_RANKED = 1;
  GAME_MODE_CASUAL = 2;
  GAME_MODE_ROOM = 3;
  GAME_MODE_PVE_AI = 4;
}

message PlayerId {
  common.v1.EntityId player_id = 1;
  string display_name = 2;
  uint32 rank_score = 3; // 天梯积分
  uint32 level = 4;
}

// 货币 (per RGS-REQ-038 §BR-007)
enum CurrencyType {
  CURRENCY_TYPE_UNSPECIFIED = 0;
  CURRENCY_TYPE_SOFT = 1;  // 软通
  CURRENCY_TYPE_HARD = 2;  // 硬通
  CURRENCY_TYPE_CARD_VALUE = 3; // 卡牌价值 (集换)
}

message Currency {
  CurrencyType type = 1;
  int64 amount = 2;
}
```

## 4.2 match.proto v2 新增 RPC + message

```proto
syntax = "proto3";
package match.v1;
import "common/v1/common.proto";

service MatchService {
  // 既有 (v1 保留)
  rpc HealthCheck(common.v1.HealthCheckRequest) returns (common.v1.HealthCheckResponse);
  rpc GetMatch(common.v1.EntityId) returns (Match);
  
  // v2 新增: 匹配 (per RGS-REQ-038 §FR-005)
  rpc EnqueueMatchmaking(EnqueueMatchmakingRequest) returns (EnqueueMatchmakingResponse);
  rpc CancelMatchmaking(CancelMatchmakingRequest) returns (CancelMatchmakingResponse);
  rpc GetMatchmakingStatus(GetMatchmakingStatusRequest) returns (GetMatchmakingStatusResponse);
  
  // v2 新增: session (per RGS-REQ-038 §FR-004)
  rpc CreateMatch(CreateMatchRequest) returns (CreateMatchResponse);
  rpc JoinMatch(JoinMatchRequest) returns (JoinMatchResponse);
  rpc LeaveMatch(LeaveMatchRequest) returns (LeaveMatchResponse);
  rpc GetMatchState(GetMatchStateRequest) returns (GetMatchStateResponse);
  rpc SubmitMove(SubmitMoveRequest) returns (SubmitMoveResponse);
  rpc SubscribeMatch(SubscribeMatchRequest) returns (stream MatchEvent);
}

message Match {
  common.v1.EntityId id = 1;
  common.v1.Status status = 2;
  common.v1.Timestamp created_at = 3;
  string display_name = 4;
  // v2 扩展
  common.v1.GameMode mode = 5;
  repeated common.v1.PlayerId players = 6;
  string board_snapshot_ref = 7; // 对象存储引用
  uint32 turn_index = 8;
}

message EnqueueMatchmakingRequest {
  string request_id = 1;
  common.v1.PlayerId player = 2;
  common.v1.GameMode mode = 3;
  // 天梯: rank_score 范围; 休闲: 任意
  uint32 rank_score_min = 4;
  uint32 rank_score_max = 5;
  common.v1.CardRef deck_ref = 6; // 卡组引用
}

message EnqueueMatchmakingResponse {
  string ticket_id = 1; // 匹配 ticket
  int64 estimated_wait_ms = 2;
}

message GetMatchmakingStatusRequest {
  string ticket_id = 1;
}

message GetMatchmakingStatusResponse {
  enum Status {
    TICKET_STATUS_UNSPECIFIED = 0;
    QUEUED = 1;
    MATCHED = 2;
    CANCELLED = 3;
    EXPIRED = 4;
  }
  Status status = 1;
  string match_id = 2; // MATCHED 时填
}

message CreateMatchRequest {
  string request_id = 1;
  common.v1.GameMode mode = 2;
  common.v1.PlayerId host = 3;
  common.v1.CardRef deck_ref = 4;
  // 房间模式额外字段
  string room_code = 5;  // 房间码 (ROOM 模式)
  string room_password = 6;
  uint32 max_players = 7;
  // AI 模式
  uint32 ai_difficulty = 8; // 0=随机 1=简单 2=中等 3=困难
}

message CreateMatchResponse {
  string match_id = 1;
  common.v1.GameMode mode = 2;
  string room_code = 3; // ROOM 模式
}

message JoinMatchRequest {
  string request_id = 1;
  string match_id = 2;
  common.v1.PlayerId player = 3;
  common.v1.CardRef deck_ref = 4;
  string room_code = 5;
  string room_password = 6;
}

message JoinMatchResponse {
  bool joined = 1;
  uint32 turn_index = 2; // 加入时的回合
}

message LeaveMatchRequest {
  string request_id = 1;
  string match_id = 2;
  common.v1.PlayerId player = 3;
  bool surrender = 4; // 投降
}

message LeaveMatchResponse {
  bool left = 1;
  string match_result = 2; // "surrender" / "disconnect"
}

message GetMatchStateRequest {
  string request_id = 1;
  string match_id = 2;
  common.v1.PlayerId player = 3;
}

message GetMatchStateResponse {
  Match match = 1;
  string board_snapshot = 2; // JSON 序列化
  repeated Move pending_moves = 3; // 待执行 move
  int64 next_turn_deadline_ms = 4;
}

message SubmitMoveRequest {
  string request_id = 1;
  string match_id = 2;
  common.v1.PlayerId player = 3;
  uint32 turn_index = 4; // 防并发
  Move move = 5;
}

message SubmitMoveResponse {
  bool accepted = 1;
  uint32 new_turn_index = 2;
  string new_board_snapshot_ref = 3;
  // 拒绝原因
  string reject_reason = 4;
}

message Move {
  string move_id = 1;        // UUID
  common.v1.PlayerId player = 2;
  enum MoveType {
    MOVE_TYPE_UNSPECIFIED = 0;
    PLAY_CARD = 1;     // 出牌
    ATTACK = 2;        // 攻击
    END_TURN = 3;      // 结束回合
    SURRENDER = 4;     // 投降
    USE_ABILITY = 5;   // 触发技能
  }
  MoveType type = 6;
  // move payload (JSON, 业务层解析)
  string payload_json = 7;
  int64 occurred_at_ms = 8;
  // 业务层返回的结果
  string result_json = 9;
  bool accepted = 10;
}

message SubscribeMatchRequest {
  string request_id = 1;
  string match_id = 2;
  common.v1.PlayerId player = 3;
  // 订阅类型: 全量 / 增量
  bool full_snapshot_first = 4;
}

message MatchEvent {
  enum EventType {
    EVENT_TYPE_UNSPECIFIED = 0;
    SNAPSHOT = 1;        // 战牌快照更新
    MOVE_APPLIED = 2;    // move 已应用
    TURN_CHANGED = 3;    // 回合切换
    PLAYER_JOINED = 4;
    PLAYER_LEFT = 5;
    MATCH_ENDED = 6;
    TIMEOUT_WARNING = 7; // turn 超时警告
  }
  EventType type = 1;
  int64 occurred_at_ms = 2;
  oneof payload {
    string board_snapshot = 10;
    Move move = 11;
    uint32 new_turn_index = 12;
    common.v1.PlayerId player = 13;
    string end_reason = 14;
  }
}
```

## 4.3 player.proto v2 新增 RPC

```proto
syntax = "proto3";
package player.v1;
import "common/v1/common.proto";

service PlayerService {
  // 既有 (v1 保留)
  rpc HealthCheck(common.v1.HealthCheckRequest) returns (common.v1.HealthCheckResponse);
  rpc GetPlayer(common.v1.EntityId) returns (Player);
  
  // v2 新增: 卡牌游戏资料 (per RGS-REQ-038 §FR-001)
  rpc GetPlayerProfile(GetPlayerProfileRequest) returns (PlayerProfile);
  rpc UpdatePlayerProfile(UpdatePlayerProfileRequest) returns (UpdatePlayerProfileResponse);
  
  // v2 新增: 卡组 (per RGS-REQ-038 §FR-002, per DEC-038-01)
  rpc CreateDeck(CreateDeckRequest) returns (Deck);
  rpc GetDeck(GetDeckRequest) returns (Deck);
  rpc UpdateDeck(UpdateDeckRequest) returns (UpdateDeckResponse);
  rpc DeleteDeck(DeleteDeckRequest) returns (DeleteDeckResponse);
  rpc ListDecks(ListDecksRequest) returns (ListDecksResponse);
  rpc ShareDeck(ShareDeckRequest) returns (ShareDeckResponse);
  rpc GetSharedDeck(GetSharedDeckRequest) returns (Deck);
}

message Player {
  common.v1.EntityId id = 1;
  common.v1.Status status = 2;
  common.v1.Timestamp created_at = 3;
  string display_name = 4;
}

message PlayerProfile {
  common.v1.PlayerId player = 1;
  // 卡牌游戏专属
  uint32 ranked_score = 2;        // 天梯积分
  string ranked_tier = 3;         // 段位 (青铜 / 白银 / ...)
  uint32 total_matches = 4;       // 总对战数
  uint32 total_wins = 5;          // 总胜
  uint32 collection_count = 6;    // 收藏数
  repeated common.v1.Currency currencies = 7;
  common.v1.Locale preferred_locale = 8;
}

message Deck {
  string deck_id = 1;
  common.v1.PlayerId owner = 2;
  string name = 3;
  common.v1.GameMode mode = 4; // 卡组适配模式
  repeated DeckSlot slots = 5; // 30-60 张
  common.v1.Status status = 6;
  common.v1.Timestamp created_at = 7;
  common.v1.Timestamp updated_at = 8;
  bool is_public = 9;
  string share_code = 10; // 公开分享码
  uint32 like_count = 11;
}

message DeckSlot {
  common.v1.CardRef card = 1;
  uint32 count = 2; // 同卡数量 (1-3)
}

message CreateDeckRequest {
  string request_id = 1;
  common.v1.PlayerId owner = 2;
  string name = 3;
  common.v1.GameMode mode = 4;
}

message GetDeckRequest {
  string request_id = 1;
  string deck_id = 2;
}

message UpdateDeckRequest {
  string request_id = 1;
  string deck_id = 2;
  common.v1.PlayerId owner = 3;
  // 全量替换 slots
  repeated DeckSlot slots = 4;
  string name = 5;
}

message UpdateDeckResponse {
  bool updated = 1;
  // 校验失败详情
  repeated string validation_errors = 2;
}

message DeleteDeckRequest {
  string request_id = 1;
  string deck_id = 2;
  common.v1.PlayerId owner = 3;
}

message DeleteDeckResponse {
  bool deleted = 1;
}

message ListDecksRequest {
  string request_id = 1;
  common.v1.PlayerId owner = 2;
  common.v1.PageRequest page = 3;
}

message ListDecksResponse {
  repeated Deck decks = 1;
  common.v1.PageResponse page = 2;
}

message ShareDeckRequest {
  string request_id = 1;
  string deck_id = 2;
  common.v1.PlayerId owner = 3;
  bool make_public = 4;
}

message ShareDeckResponse {
  string share_code = 1;
  string share_url = 2;
}

message GetSharedDeckRequest {
  string request_id = 1;
  string share_code = 2;
  // 兼容好友 ID 拉取
  common.v1.PlayerId friend_id = 3;
  string friend_deck_id = 4;
}
```

## 4.4 card.proto v1 (新域) 完整

```proto
syntax = "proto3";
package card.v1;
import "common/v1/common.proto";

service CardService {
  rpc HealthCheck(common.v1.HealthCheckRequest) returns (common.v1.HealthCheckResponse);
  
  // 卡牌 catalog (静态 / 慢变, 缓存友好)
  rpc GetCard(GetCardRequest) returns (Card);
  rpc ListCards(ListCardsRequest) returns (ListCardsResponse);
  rpc GetCardSeries(GetCardSeriesRequest) returns (CardSeries);
  rpc ListCardSeries(ListCardSeriesRequest) returns (ListCardSeriesResponse);
  
  // 玩家收藏 (动态)
  rpc GetPlayerCollection(GetPlayerCollectionRequest) returns (GetPlayerCollectionResponse);
  // 内部 / 抽卡结果, 不暴露客户端
  rpc AddCardToCollection(AddCardToCollectionRequest) returns (AddCardToCollectionResponse);
  rpc RemoveCardFromCollection(RemoveCardFromCollectionRequest) returns (RemoveCardFromCollectionResponse);
  
  // 抽卡 (per RGS-REQ-038 §BR-003)
  rpc OpenPack(OpenPackRequest) returns (OpenPackResponse);
}

message Card {
  string card_id = 1;
  common.v1.I18nString name = 2;
  common.v1.CardType type = 3;
  common.v1.CardRarity rarity = 4;
  string series_id = 5;     // 所属卡包 / 系列
  uint32 base_cost = 6;     // 基础费用
  common.v1.I18nString description = 7; // 卡牌描述
  // 效果引用 (业务层 game-logic 解析)
  string effect_ref = 8;
  // 卡牌属性 (攻击 / 生命 / ...)
  CardStats stats = 9;
}

message CardStats {
  uint32 attack = 1;
  uint32 health = 2;
  uint32 mana = 3;
  // 扩展
  map<string, int32> custom = 10;
}

message CardSeries {
  string series_id = 1;
  common.v1.I18nString name = 2;
  uint32 pack_size = 3;     // 一包几张
  // 抽卡概率表 (per DEC-038-06 强制公开)
  DropTable drop_table = 4;
  common.v1.Currency price = 5;
  common.v1.Timestamp released_at = 6;
  common.v1.Status status = 7; // 活跃 / 绝版
}

message DropTable {
  // 概率快照 (per SR-001)
  uint32 version = 1;          // 每次调整递增
  common.v1.Timestamp snapshot_at = 2;
  repeated DropEntry entries = 3;
}

message DropEntry {
  common.v1.CardRarity rarity = 1;
  uint32 count = 2;            // 出几张
  double probability = 3;      // 0.0-1.0
  string card_id = 4;          // 单卡 (可选, 用于保底)
}

message CardInstance {
  string instance_id = 1;       // UUID
  string card_id = 2;           // 静态 card.id
  common.v1.PlayerId owner = 3;
  common.v1.Timestamp acquired_at = 4;
  enum Source {
    SOURCE_UNSPECIFIED = 0;
    SOURCE_PACK = 1;       // 开包
    SOURCE_REWARD = 2;     // 任务奖励
    SOURCE_TRADE = 3;      // 交易
    SOURCE_GM_GRANT = 4;   // GM 补偿
    SOURCE_EVENT = 5;      // 活动
  }
  Source source = 5;
  uint32 level = 6;            // 等级 (1-N)
  map<string, int32> attrs = 7; // 个性化属性 (强化 / 精炼)
  // 交易状态
  bool tradable = 8;
  bool locked = 9;             // 锁定中
}

message GetCardRequest {
  string request_id = 1;
  string card_id = 2;
  common.v1.Locale locale = 3;
}

message ListCardsRequest {
  string request_id = 1;
  common.v1.Locale locale = 2;
  common.v1.PageRequest page = 3;
  // 过滤
  common.v1.CardType type_filter = 4;
  common.v1.CardRarity rarity_filter = 5;
  string series_id_filter = 6;
}

message ListCardsResponse {
  repeated Card cards = 1;
  common.v1.PageResponse page = 2;
}

message GetPlayerCollectionRequest {
  string request_id = 1;
  common.v1.PlayerId player = 2;
  common.v1.PageRequest page = 3;
  // 过滤
  common.v1.CardRarity rarity_filter = 4;
  string series_id_filter = 5;
}

message GetPlayerCollectionResponse {
  repeated CardInstance instances = 1;
  common.v1.PageResponse page = 2;
  // 收藏统计
  uint32 total_count = 3;
  map<string, uint32> by_rarity = 4; // rarity -> count
}

message AddCardToCollectionRequest {
  string request_id = 1;
  common.v1.PlayerId player = 2;
  string card_id = 3;
  CardInstance.Source source = 4;
  // 用于 saga 关联
  string saga_id = 5;
}

message AddCardToCollectionResponse {
  string instance_id = 1;
  CardInstance instance = 2;
}

message RemoveCardFromCollectionRequest {
  string request_id = 1;
  string instance_id = 2;
  common.v1.PlayerId player = 3;
  string reason = 4;
  string saga_id = 5;
}

message RemoveCardFromCollectionResponse {
  bool removed = 1;
}

message OpenPackRequest {
  string request_id = 1;
  common.v1.PlayerId player = 2;
  string series_id = 3;
  uint32 pack_count = 4; // 一次开 N 包
  // 关联 saga (扣货币)
  string saga_id = 5;
}

message OpenPackResponse {
  // 抽到的卡牌 (按稀有度倒序)
  repeated CardInstance instances = 1;
  // 概率快照 (per SR-001)
  DropTable drop_table = 2;
  // 交易流水 ID
  string transaction_id = 3;
}
```

---

# 5. session / turn 状态机

## 5.1 状态机图

```
                      ┌─────────────┐
                      │  CREATING   │
                      └──────┬──────┘
                             │ host CreateMatch
                             ▼
                      ┌─────────────┐
        ┌────────────│   WAITING   │  (ROOM 模式: 等玩家加入)
        │             └──────┬──────┘
        │ 取消 / 过期       │ 玩家到齐 (≥2)
        ▼                    ▼
   ┌──────────┐         ┌─────────────┐
   │ CANCELED │         │  STARTING   │
   └──────────┘         └──────┬──────┘
                              │ 加载卡组 / 初始 Board
                              ▼
                       ┌─────────────┐
                       │  RUNNING    │◄──┐
                       └──────┬──────┘   │ 下一回合
                              │          │
                              ▼          │
            ┌──────────┐  ┌─────────────┐│
            │ PAUSED   │◄─┤   TURN_N    │┘
            │(断线/GM) │  └──────┬──────┘
            └────┬─────┘         │
                 │ resume        │ 胜负判定
                 └────────────┐  ▼
                              ┌─────────────┐
                              │ ENDING      │ ── 结算 / 保存回放
                              └──────┬──────┘
                                     │
                                     ▼
                              ┌─────────────┐
                              │  ENDED      │
                              └─────────────┘
```

## 5.2 状态转移表

| From | To | 触发 | 守卫 | 动作 |
|---|---|---|---|---|
| CREATING | WAITING | CreateMatch | mode=ROOM, players<max | 创建 session, 分配 match_id |
| CREATING | STARTING | CreateMatch | mode=RANKED/CASUAL/AI, players≥2 | 创建 session, 直接 STARTING |
| CREATING | CANCELED | 取消 / 超时 | host 取消 | 释放资源 |
| WAITING | STARTING | 玩家到齐 | players≥min_players | 加载卡组 |
| WAITING | CANCELED | 超时 / host 离开 | timeout=300s | 释放资源 |
| STARTING | RUNNING | 初始 Board 完成 | deck 验证通过 | turn=0, 通知 host 先手 |
| RUNNING | TURN_N | SubmitMove (END_TURN) | current_turn 完成 | 切换 player, 增 turn_index |
| TURN_N | RUNNING | 通知到位 | 通知 ACK | 进入下一回合 |
| RUNNING | PAUSED | 玩家断线 / GM 暂停 | GM 命令 / 30s 超时 | 冻结 turn 计时 |
| PAUSED | RUNNING | 玩家重连 / GM 恢复 | 30s 内重连 / GM 命令 | 续 turn 计时 |
| RUNNING | ENDING | 胜负判定 / 投降 | game_logic 返回 winner | 标记 winner, 准备回放 |
| ENDING | ENDED | 回放保存 | replay_save 成功 | 释放 session, 推排行榜 |
| ENDED | (终态) | - | - | 不可变 |

## 5.3 turn 超时机制

- 默认 60s / turn, 可配
- 超时前 10s 推送 TIMEOUT_WARNING 事件
- 超时后 5s 自动 END_TURN (强制)
- 累计超时 3 次 = 判负

## 5.4 断线重连

- 玩家断线 → 5s 内重连: 续 turn
- 5-30s: 状态 PAUSED, 等待重连
- 30-60s: AI 接管 (per 模式)
- 60s+: 判负

## 5.5 强制踢出 (gm.proto v0.4 新增)

per RGS-REQ-038 §FR-010:
- GM 调用 `BanAccount(force_disconnect_session=true)`
- gm-backend → match-service `ForceDisconnectSession(match_id, player_id, reason)`
- match-service 立即将该玩家置为 LEAVE, 判负
- 写审计

---

# 6. saga 编排

## 6.1 抽卡 (OpenPack) saga

```
Start: 玩家 OpenPack (扣货币)
   ↓
1. economy-service.DebitCurrency(player, price, saga_id)
   - 成功 → 继续
   - 失败 → Abort (余额不足)
   ↓
2. card-service.GenerateDropResult(series, count, drop_table)
   - 成功 → 继续
   - 失败 → 触发 compensation (退货币)
   ↓
3. card-service.AddCardToCollection(player, [card_ids], saga_id)
   - 成功 → 继续
   - 失败 → 触发 compensation (退货币 + 删卡)
   ↓
4. economy-service.AddTransactionLog (审计)
   - 成功 → Commit
   - 失败 → 触发 compensation
   ↓
End: 返回 OpenPackResponse
```

补偿链 (从后向前):
- 步骤 4 失败 → 重试 3 次 → 仍失败 → 标 saga 失败 + 告警
- 步骤 3 失败 → 步骤 2 不需补偿 (无副作用) → 步骤 1 退货币
- 步骤 2 失败 → 步骤 1 退货币

## 6.2 交易 (Trade) saga

```
Start: 玩家 BidAuction (出价)
   ↓
1. trade-service.LockAuction(auction_id, bidder, amount, saga_id)
   - 成功 → 继续
   - 失败 → Abort (已被竞拍)
   ↓
2. economy-service.DebitCurrency(bidder, amount, saga_id)
   - 成功 → 继续
   - 失败 → 步骤 1 释放
   ↓
3. trade-service.UpdateHighestBid(auction_id, bidder, amount)
   - 成功 → 继续
   - 失败 → 步骤 2 退货币 + 步骤 1 释放
   ↓
4. trade-service.CheckAuctionEnded(auction_id)
   - 未结束 → Commit (saga 完成, 但拍卖进行中)
   - 已结束 → 进入 ExecuteAuction saga (见 6.3)
   ↓
End: 返回 BidResponse
```

## 6.3 成交 (ExecuteAuction) saga

```
Start: 拍卖结束
   ↓
1. trade-service.FinalizeAuction(auction_id, winner)
   ↓
2. economy-service.TransferCurrency(winner → seller, amount - tax, saga_id)
   ↓
3. card-service.RemoveCardFromCollection(seller, card_instance_id, saga_id)
   ↓
4. card-service.AddCardToCollection(winner, card_id, source=TRADE, saga_id)
   ↓
5. economy-service.AddTransactionLog (审计, 含卡牌价值)
   ↓
End: 返回 TradeResult
```

补偿链 (任意失败):
- 步骤 5 失败 → 重试 + 告警
- 步骤 4 失败 → 步骤 3 还原 (saga_id 关联)
- 步骤 3 失败 → 步骤 2 退货币 (赢家 → 卖家)
- 步骤 2 失败 → 步骤 1 撤拍卖结果, 重新开放

---

# 7. 数据库 schema 草图

## 7.1 新增 8 张表

```sql
-- 1. cards (catalog, 静态)
CREATE TABLE cards (
    card_id          TEXT PRIMARY KEY,
    series_id        TEXT NOT NULL,
    name_default     TEXT NOT NULL,
    name_i18n        JSONB NOT NULL,
    type             SMALLINT NOT NULL,
    rarity           SMALLINT NOT NULL,
    base_cost        INT NOT NULL DEFAULT 0,
    description_i18n JSONB NOT NULL,
    effect_ref       TEXT NOT NULL,
    stats            JSONB NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_cards_series ON cards(series_id);
CREATE INDEX idx_cards_rarity ON cards(rarity);

-- 2. card_series (卡包 / 系列)
CREATE TABLE card_series (
    series_id    TEXT PRIMARY KEY,
    name_default TEXT NOT NULL,
    name_i18n    JSONB NOT NULL,
    pack_size    INT NOT NULL,
    drop_table   JSONB NOT NULL,
    price_type   SMALLINT NOT NULL,
    price_amount BIGINT NOT NULL,
    released_at  TIMESTAMPTZ NOT NULL,
    status       SMALLINT NOT NULL DEFAULT 1 -- 1=活跃 2=绝版
);

-- 3. card_instances (玩家收藏, 动态)
CREATE TABLE card_instances (
    instance_id  UUID PRIMARY KEY,
    card_id      TEXT NOT NULL REFERENCES cards(card_id),
    owner_id     TEXT NOT NULL,
    acquired_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    source       SMALLINT NOT NULL,
    level        INT NOT NULL DEFAULT 1,
    attrs        JSONB NOT NULL DEFAULT '{}',
    tradable     BOOLEAN NOT NULL DEFAULT TRUE,
    locked       BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_card_instances_owner ON card_instances(owner_id);
CREATE INDEX idx_card_instances_card ON card_instances(card_id);

-- 4. decks (卡组, 玩家强属性, 在 player DB)
CREATE TABLE decks (
    deck_id     UUID PRIMARY KEY,
    owner_id    TEXT NOT NULL,
    name        TEXT NOT NULL,
    mode        SMALLINT NOT NULL,
    slots       JSONB NOT NULL DEFAULT '[]', -- [{card_id, count}]
    status      SMALLINT NOT NULL DEFAULT 1,
    is_public   BOOLEAN NOT NULL DEFAULT FALSE,
    share_code  TEXT UNIQUE,
    like_count  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_decks_owner ON decks(owner_id);
CREATE INDEX idx_decks_share_code ON decks(share_code);

-- 5. game_sessions (对战 session)
CREATE TABLE game_sessions (
    match_id         UUID PRIMARY KEY,
    mode             SMALLINT NOT NULL,
    status           SMALLINT NOT NULL, -- 状态机状态
    players          JSONB NOT NULL,    -- [player_id, ...]
    board_snapshot_ref TEXT,            -- 对象存储引用
    turn_index       INT NOT NULL DEFAULT 0,
    next_turn_deadline_ms BIGINT,
    started_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at         TIMESTAMPTZ,
    end_reason       TEXT,
    winner_id        TEXT
);
CREATE INDEX idx_sessions_status ON game_sessions(status);
CREATE INDEX idx_sessions_players ON game_sessions USING GIN(players);

-- 6. moves (操作日志)
CREATE TABLE moves (
    move_id      UUID PRIMARY KEY,
    match_id     UUID NOT NULL REFERENCES game_sessions(match_id),
    player_id    TEXT NOT NULL,
    turn_index   INT NOT NULL,
    move_type    SMALLINT NOT NULL,
    payload      JSONB NOT NULL,
    result       JSONB,
    accepted     BOOLEAN NOT NULL,
    occurred_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_moves_match ON moves(match_id);
CREATE INDEX idx_moves_turn ON moves(match_id, turn_index);

-- 7. replays (回放元数据, 数据在对象存储)
CREATE TABLE replays (
    replay_id       UUID PRIMARY KEY,
    match_id        UUID NOT NULL REFERENCES game_sessions(match_id),
    player_a        TEXT NOT NULL,
    player_b        TEXT,
    mode            SMALLINT NOT NULL,
    object_key      TEXT NOT NULL,    -- 对象存储 key
    object_size     BIGINT,
    duration_secs   INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL -- 热数据周期
);
CREATE INDEX idx_replays_player_a ON replays(player_a);
CREATE INDEX idx_replays_expires ON replays(expires_at);

-- 8. trades (拍卖 / 私下交易)
CREATE TABLE auctions (
    auction_id     UUID PRIMARY KEY,
    seller_id      TEXT NOT NULL,
    card_instance_id UUID NOT NULL REFERENCES card_instances(instance_id),
    min_price      BIGINT NOT NULL,
    currency_type  SMALLINT NOT NULL,
    highest_bid    BIGINT,
    highest_bidder TEXT,
    status         SMALLINT NOT NULL, -- 1=进行中 2=成交 3=撤单 4=过期
    started_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    ends_at        TIMESTAMPTZ NOT NULL,
    closed_at      TIMESTAMPTZ
);
CREATE INDEX idx_auctions_status ON auctions(status);
CREATE INDEX idx_auctions_seller ON auctions(seller_id);
```

## 7.2 既有表扩展 (0 破坏)

- `players`: 增加 `ranked_score` / `ranked_tier` / `preferred_locale` 3 列
- `accounts` (经济): 增加 `currency_card_value` 1 列
- 现有 35+ 张表保留

## 7.3 迁移工具

- 沿用 sqlx migrate (现有)
- 8 张新表一个 migration (v038_card_game.sql)
- 既有表 3 列扩展一个 migration (v038_player_extend.sql)

---

# 8. 8 桶 WBS 排序建议

per RGS-REQ-038 §8.1, 沿用 WBS v0.5 token 桶方法:

| 桶 | 内容 | Token 估 | 关键依赖 | 实装 |
|---|---|---|---|---|
| **7** | proto v0 设计 (4 份草稿 + 评审) | 8M | RGS-REQ-038 + RGS-DTL-038 (本档) | 设计阶段, 不实装代码 |
| **8** | proto v1 实装 (落档 + 6 域编译 + 25+ UT) | 18M | 桶 7 | 4 proto + 6 域 stub + 25 UT |
| **9** | session/turn 抽象 (match-service v2 + 状态机) | 25M | 桶 8 | match.proto v2 + 状态机 + 30 IT |
| **10** | card catalog (card-service v1 + collection CRUD) | 18M | 桶 8 | card.proto v1 + 8 表 + 20 IT |
| **11** | deck + share (player-service v2) | 12M | 桶 8 | player.proto v2 deck 部分 + 15 IT |
| **12** | leaderboard (新域 + 4 榜) | 8M | 桶 8 | leaderboard.proto v1 + 10 IT |
| **13** | replay (存储 + 播放) | 15M | 桶 9 + 10 | replay.proto v1 + 12 IT |
| **14** | trade + gm 扩展 (auction + gm.proto v0.4) | 25M | 桶 10 + 11 | trade.proto v1 + gm.proto v0.4 + 18 IT |
| **合计** | 8 桶 | **129M** | - | 130+ 新 IT |

## 8.1 8 桶与 6 桶关系

- 桶 1-6 (现有, 5/8 完成 + 3/8 落档)
- 桶 7-14 (新增, 卡牌游戏 8 桶)
- 总 14 桶, 累计 129M 估 (vs 当前 31M 余额 = **追加 98M**)

## 8.2 风险控制

- 桶 7 试运行 1 桶 (proto 设计) 验证估时
- 桶 8 后做一次阶段审计 (per W25 Step 3 模式)
- 桶 9-10 关键路径, 实装失败 → 落档 + 推后续

---

# 9. 风险与未决项

## 9.1 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 抽卡事务一致性 | P1 | saga 已就位 (per economy-service) |
| session 状态机复杂度 | P1 | 状态机 + turn 超时 + 断线重连, 充分 UT/IT 覆盖 |
| 8 域 → 14 域运维成本 | P2 | 沿用 k3s + cluster-ops, 运维基线已有 |
| 卡牌效果执行 (规则引擎) | P2 | 留给业务层 game-logic, RGS 只承载数据流 |
| 多语言文案运营 | P2 | i18n-service 独立实装, 不阻塞核心 |
| 反作弊 | P1 | 现有 JWT + 限流 + audit, 卡牌游戏沿用 |
| 拍卖安全性 | P1 | saga 编排 + 货币原子转移, 失败可补偿 |
| 14 桶估时不准 | P2 | 桶 7 试运行 + 阶段审计 |

## 9.2 未决项 (9 个 DEC 拍板)

| DEC | 候选 | 推荐 |
|---|---|---|
| 01 卡组归属 | A player / B card / C 独立 deck | **A** (player-service v2) |
| 02 leaderboard 域 | A 新域 / B match 子 / C shared | **A** (新域) |
| 03 replay 存储 | A cluster-ops / B 新域 / C 外部 S3 | **A** (cluster-ops) |
| 04 trade 域归属 | A economy / B 新域 / C inbox | **A** (economy v2) |
| 05 i18n 模式 | A Redis+DB / B build-time / C 静态文件 | **A** (Redis+DB) |
| 06 抽卡概率 | A 强制 / B 可选 / C 关闭 | **A** (强制公开) |
| 07 gm.proto v0.4 | A 桶 14 后 / B 立即 / C 桶 10 后 | **A** (桶 14 后) |
| 08 8 桶 WBS 排序 | A token 桶 / B RACI / C 业务关键 | **A** (token 桶) |
| 09 总 token 追加 | A 98M / B 拆 2 阶段 / C 砍 1 桶 | **A** (追加 98M) |

---

# 10. 关联文档

- **RGS-REQ-038** 卡牌游戏适配需求定义书 v0.1 (本文档上游)
- **RGS-BAS-038** 卡牌游戏适配基本设计书 (待写, 本文档后续)
- **gm.proto v0.3** GM 后台协议 (v0.4 计划)
- **common.proto v1** (v2 计划)
- **match.proto v1** (v2 计划)
- **player.proto v1** (v2 计划)
- **card.proto v0** (v1 计划, 新域)
- **leaderboard.proto v0** (v1 计划, 新域)
- **replay.proto v0** (v1 计划, 新域)
- **trade.proto v0** (v1 计划, 在 economy v2)
- **i18n.proto v0** (v1 计划, 新域)
- **RGS-PLAN-WBS-token-bucket-v0.5** 6 桶 (后续追加 8 桶)
- **RGS-BAS-003-mTLS-决策补充-v0.1** mTLS 范围

---

## 制定 / 审批

| 角色 | 姓名 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 制定 (架构) | 架构师 (Mavis 接手 agent per DEC-008) | ✓ | 2026-08-29 | 初版草案, 9 DEC 候选 |
| 审批 (技术) | — | ⏳ | — | 待 DDD Review |
| 审批 (业务) | — | ⏳ | — | 待 Ulysses |
| **最终决策 (产品)** | **Ulysses** | ⏳ | — | 待拍板 9.2 9 未决项 |

> 本文档**已为下一阶段 RGS-BAS-038 基本设计**准备就绪. 基本设计将基于本详细设计给出: 各域具体函数签名 / 跨域接口 / 部署 yaml / 监控指标 / IT 详细用例.
