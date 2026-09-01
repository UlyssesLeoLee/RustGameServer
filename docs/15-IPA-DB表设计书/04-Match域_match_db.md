# 04-Match 域（match_db）

> **本文件定位**：Match 域 7 张表的詳細表設計書。覆盖 2 基础表（matches / match_participants）+ 4 卡牌游戏扩展表（game_sessions / moves / matchmaking_tickets / session_subscriptions）+ 1 公共 outbox。

| 项目 | 内容 |
|---|---|
| 物理库 | `match_db` |
| 担当 crate | `match-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 7（含 outbox） |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/match-service/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` + `0040_game_sessions.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 4.1 | `matches` | マッチ（対局） / Matches (Generic) | 永久追加表 | 千万级 | 2 |
| 4.2 | `match_participants` | マッチ参加者 / Match Participants | 永久追加表 | 千万级 | 2 |
| 4.3 | `game_sessions` | ゲームセッション（カノニカル状態） / Game Sessions (Canonical State) | 永久追加表 | 千万级 | 4 + 1 JSONB-GIN |
| 4.4 | `moves` | ムーブ（操作ログ） / Moves (Operation Log) | 永久追加表 | 亿级 | 4 |
| 4.5 | `matchmaking_tickets` | マッチメイキングチケット / Matchmaking Tickets | 短期表 | 千万级（短期清理） | 4 |
| 4.6 | `session_subscriptions` | セッション購読 / Session Subscriptions | 短期表 | 千万级 | 1 |
| 4.7 | `outbox` | アウトボックス（公共） / Outbox | 时序短期表 | 千万级/日 | 3 |

---

## 4.1 `matches` マッチ（対局）

### 概要

基础对局表（per RGS-DTL-016 §3）。4 种模式（1v1 / 2v2 / 5v5 / battle_royale），4 状态机（waiting / in_progress / finished / cancelled）。`room_id` UNIQUE 用于房间号唯一。`winner_team` 允许 NULL（未结束时）但有 CHECK 约束限定枚举值（blue / red / none）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `matches` |
| 論理名 | マッチ（対局） / Matches (Generic) |
| 出典 | `crates/match-service/migrations/0001_init.sql:5-17` |
| 父文档 | RGS-DTL-016 §3 / RGS-DTL-038 §7.1 #5（卡牌 8 桶时未重命名） |
| 関連表 | `match_participants` (1:N CASCADE), `game_sessions` (1:1 跨文件) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `room_id` | ルーム ID / Room Identifier | TEXT | 1-64 字符 | — | — | ✅ | ✅ | — | — | 房间号（全局唯一）|
| 3 | `mode` | ゲームモード / Game Mode | TEXT | — | — | — | — | ✅ | — | `mode IN ('1v1', '2v2', '5v5', 'battle_royale')` | 4 选 1 模式 |
| 4 | `status` | マッチ状態 / Match Status | TEXT | — | — | — | — | ✅ | `'waiting'` | `status IN ('waiting', 'in_progress', 'finished', 'cancelled')` | 4 状态机 |
| 5 | `winner_team` | 勝利チーム / Winning Team | TEXT | — | — | — | — | ❌ | NULL | `winner_team IS NULL OR winner_team IN ('blue', 'red', 'none')` | 胜方（未结束时 NULL）|
| 6 | `scheduled_at` | 開始予定日時 / Scheduled At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 计划开始时间 |
| 7 | `started_at` | 開始日時 / Started At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 实际开始时间 |
| 8 | `ended_at` | 終了日時 / Ended At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 结束时间 |
| 9 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 10 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `matches_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `matches_room_id_key` | B-tree (UNIQUE) | `(room_id)` | 房间号唯一 |
| 3 | `idx_matches_status` | B-tree | `(status)` | 按状态筛选（活跃对局）|
| 4 | `idx_matches_scheduled_at` | B-tree | `(scheduled_at)` | 按计划时间排序/筛选 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `matches_pkey` | `(id)` |
| UNIQUE | `matches_room_id_key` | `(room_id)` |
| CHECK | (隐式) `mode_check` | `mode IN ('1v1', '2v2', '5v5', 'battle_royale')` |
| CHECK | (隐式) `status_check` | `status IN ('waiting', 'in_progress', 'finished', 'cancelled')` |
| CHECK | (隐式) `winner_team_check` | `winner_team IS NULL OR winner_team IN ('blue', 'red', 'none')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N | `match_participants` | `match_participants.match_id → matches.id` | CASCADE | ✅ |
| 1:0..1 (跨 migration) | `game_sessions` (同库) | `game_sessions.match_id = matches.id` | app-layer | ❌ 弱引用（**应物化** — 见 [17-P1-05](17-不合理设计识别与优化建议.md)）|

### 既知偏差

- `winner_team` 在 status='finished' 时 DB 层无强保证非 NULL——建议 PH-2 加 `CHECK ((status = 'finished') = (winner_team IS NOT NULL))`
- `started_at` / `ended_at` 在 status 转换时的一致性无 CHECK 约束（应用层负责）

---

## 4.2 `match_participants` マッチ参加者

### 概要

对局参与者表（per RGS-DTL-016 §3.2）。`UNIQUE (match_id, player_id)` 防重入。统计字段（score / kills / deaths / assists）+ MVP 标记。

| 项目 | 内容 |
|---|---|
| 物理表名 | `match_participants` |
| 論理名 | マッチ参加者 / Match Participants |
| 出典 | `crates/match-service/migrations/0001_init.sql:23-35` |
| 父文档 | RGS-DTL-016 §3.2 |
| 関連表 | `matches` (N:1 CASCADE), `players` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `match_id` | マッチ ID / Match Identifier | UUID | 128-bit | — | `matches(id) ON DELETE CASCADE` | `(match_id, player_id)` | ✅ | — | — | 所属对局 |
| 3 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | — (跨域弱引用) | `(match_id, player_id)` | ✅ | — | — | 玩家（跨 player_db 弱引用）|
| 4 | `team` | チーム / Team | TEXT | — | — | — | — | ✅ | — | `team IN ('blue', 'red', 'none')` | 阵营 |
| 5 | `score` | スコア / Score | INTEGER | — | — | — | — | ✅ | 0 | — | 比赛得分 |
| 6 | `kills` | キル数 / Kills | INTEGER | — | — | — | — | ✅ | 0 | — | 击杀数 |
| 7 | `deaths` | デス数 / Deaths | INTEGER | — | — | — | — | ✅ | 0 | — | 死亡数 |
| 8 | `assists` | アシスト数 / Assists | INTEGER | — | — | — | — | ✅ | 0 | — | 助攻数 |
| 9 | `is_mvp` | MVP フラグ / MVP Flag | BOOLEAN | — | — | — | — | ✅ | FALSE | — | 是否本场 MVP |
| 10 | `joined_at` | 参加日時 / Joined At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 加入时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `match_participants_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `match_participants_match_id_player_id_key` | B-tree (UNIQUE) | `(match_id, player_id)` | 由 UNIQUE 约束自动创建 |
| 3 | `idx_participants_match_id` | B-tree | `(match_id)` | FK 索引（CASCADE 性能）|
| 4 | `idx_participants_player_id` | B-tree | `(player_id)` | 查玩家所有对局 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `match_participants_pkey` | `(id)` |
| UNIQUE | `match_participants_match_id_player_id_key` | `(match_id, player_id)` |
| FOREIGN KEY | (隐式) | `(match_id) REFERENCES matches(id) ON DELETE CASCADE` |
| CHECK | (隐式) `team_check` | `team IN ('blue', 'red', 'none')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `matches` | `match_participants.match_id → matches.id` | CASCADE | ✅ |
| N:1 (跨域) | `players` (player_db) | `match_participants.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- 缺 `is_mvp` 索引（MVP 查询路径）——建议 PH-2 评审

---

## 4.3 `game_sessions` ゲームセッション（カノニカル状態）

### 概要

卡牌对战 session 表（per RGS-DTL-038 §5.1 状态机 + §7.1）。**核心 canonical state 存储**——`players` JSONB 存玩家列表，`board_snapshot` JSONB 存战牌状态。9 状态机（unspec / creating / waiting / starting / running / turn_n / paused / ending / ended / canceled）。`room_password_hash` 敏感字段应使用 bcrypt/argon2。

| 项目 | 内容 |
|---|---|
| 物理表名 | `game_sessions` |
| 論理名 | ゲームセッション（カノニカル状態） / Game Sessions (Canonical State) |
| 出典 | `crates/match-service/migrations/0040_game_sessions.sql:9-33` |
| 父文档 | RGS-DTL-038 v0.1 §5.1 状态机 / §7.1 |
| 関連表 | `moves` (1:N CASCADE), `session_subscriptions` (1:N CASCADE), `matchmaking_tickets` (1:0..1 match_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `match_id` | マッチ ID / Match Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键（与 matches.id 弱引用）|
| 2 | `mode` | ゲームモード / Game Mode | SMALLINT | 0-4 | — | — | — | ✅ | — | — | 0=unspec 1=ranked 2=casual 3=room 4=pve_ai (GameMode 枚举) |
| 3 | `status` | セッション状態 / Session Status | SMALLINT | 0-9 | — | — | — | ✅ | 1 | — | 0=unspec 1=creating 2=waiting 3=starting 4=running 5=turn_n 6=paused 7=ending 8=ended 9=canceled |
| 4 | `players` | プレイヤー一覧（JSONB） / Players | JSONB | — | — | — | — | ✅ | `'[]'::jsonb` | — | `[{player_id, display_name, rank_score, level, deck_ref, team}, ...]` |
| 5 | `host_id` | ホスト ID / Host Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 房主 player_id |
| 6 | `room_code` | ルームコード（公開用） / Room Code (Public) | TEXT | 1-32 字符 | — | — | — | ❌ | NULL | — | 房间码（ROOM 模式）|
| 7 | `room_password_hash` | ルームパスワードハッシュ / Room Password Hash | TEXT | 60-256 字符 (bcrypt/argon2) | — | — | — | ❌ | NULL | — | 房间密码 hash（敏感字段）|
| 8 | `max_players` | 最大プレイヤー数 / Max Players | INTEGER | > 0 | — | — | — | ✅ | 2 | — | 房间最大人数 |
| 9 | `min_players` | 最小プレイヤー数 / Min Players | INTEGER | > 0 | — | — | — | ✅ | 2 | — | 房间最小人数 |
| 10 | `turn_index` | ターン番号 / Turn Index | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 当前回合号 |
| 11 | `current_player_id` | 現在手番プレイヤー / Current Turn Player | TEXT | UUID 文字列 | — | — | — | ❌ | NULL | — | 当前回合玩家 |
| 12 | `next_turn_deadline_ms` | 次ターン締切（エポック ms） / Next Turn Deadline (Epoch ms) | BIGINT | — | — | — | — | ❌ | NULL | — | 当前 turn 截止（epoch 毫秒）|
| 13 | `board_snapshot` | 盤面スナップショット（JSONB） / Board Snapshot | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 战牌状态（DB 存放 + 对象存储备份）|
| 14 | `board_snapshot_ref` | 盤面参照（オブジェクトストレージ） / Board Snapshot Reference (Object Storage) | TEXT | 1-512 字符 | — | — | — | ❌ | NULL | — | 对象存储引用 |
| 15 | `winner_id` | 勝者 ID / Winner Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 胜者 player_id（status=ended 时）|
| 16 | `end_reason` | 終了理由 / End Reason | TEXT | — | — | — | — | ❌ | NULL | — | 投降 / 超时 / 胜负判定 / 取消 / 强制踢出 |
| 17 | `ai_difficulty` | AI 難易度 / AI Difficulty | SMALLINT | 0-4 | — | — | — | ✅ | 0 | — | 0=无 1=随机 2=简单 3=中等 4=困难 |
| 18 | `timeout_count` | 累計タイムアウト回数 / Total Timeout Count | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 当前玩家累计超时次数 |
| 19 | `pending_moves` | 保留中操作（JSONB） / Pending Moves | JSONB | — | — | — | — | ✅ | `'[]'::jsonb` | — | 待执行 move 队列 |
| 20 | `started_at` | 開始日時 / Started At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 开始时间 |
| 21 | `ended_at` | 終了日時 / Ended At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 结束时间 |
| 22 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 23 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `game_sessions_pkey` | B-tree (PK) | `(match_id)` | 主键（自动） |
| 2 | `idx_game_sessions_status` | B-tree | `(status)` | 按状态筛选（活跃 session）|
| 3 | `idx_game_sessions_host_id` | B-tree | `(host_id)` | 查某房主所有 session |
| 4 | `idx_game_sessions_room_code` | B-tree | `(room_code)` | 通过房间码查找 |
| 5 | `idx_game_sessions_players_gin` | GIN | `players` | JSONB 路径查询（查某 player 是否在 players 列表中）|
| 6 | `idx_game_sessions_created_at` | B-tree | `(created_at)` | 按创建时间排序/清理 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `game_sessions_pkey` | `(match_id)` |
| (无 status CHECK) | — | `status` 用 SMALLINT 枚举（应用层校验） |
| (无 mode CHECK) | — | `mode` 用 SMALLINT 枚举（应用层校验） |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N | `moves` | `moves.match_id → game_sessions.match_id` | CASCADE | ✅ |
| 1:N | `session_subscriptions` | `session_subscriptions.match_id → game_sessions.match_id` | CASCADE | ✅ |
| 1:0..1 | `matchmaking_tickets` | `matchmaking_tickets.match_id = game_sessions.match_id` | app-layer | ❌ 弱引用（**应物化** — 见 [17-P1-05](17-不合理设计识别与优化建议.md)）|
| 1:0..1 (跨 migration) | `matches` (0001) | `matches.id = game_sessions.match_id` | app-layer | ❌ 弱引用（**应物化**）|

### 既知偏差

- **`players` JSONB 嵌 player_id 等列**：违反 Q-D-02 答复 #2 反范式禁令（player_id 应放独立关联表 `game_session_players`）——见 [17-P1-06](17-不合理设计识别与优化建议.md)
- **`status` / `mode` 用 SMALLINT 枚举但无 CHECK 约束**：建议 PH-2 加 `CHECK (status BETWEEN 0 AND 9)` + `CHECK (mode BETWEEN 0 AND 4)`
- **`room_password_hash` 敏感字段**：DB 层未指明 hash 算法（应为 bcrypt 或 argon2），建议 PH-2 在 DDL 注释中明确 + 加 length CHECK（如 `CHECK (length(room_password_hash) BETWEEN 60 AND 256)`）
- **`game_sessions.match_id` 与 `matches.id` 同库内物化建议**：避免 matches.id 与 game_sessions.match_id 长期分离（4.1 既有 matches 表）—— 实质上是卡牌游戏用 game_sessions 取代 matches，需 PH-2 评审废弃路径

---

## 4.4 `moves` ムーブ（操作ログ）

### 概要

对局操作日志表（per RGS-DTL-038 §4.2 Move message + §7.1）。每一步操作（出牌 / 攻击 / 结束回合 / 投降 / 使用技能）记一条。`payload_json` 存输入，`result_json` 存业务层返回结果，`accepted` + `reject_reason` 区分受理/拒绝。

| 项目 | 内容 |
|---|---|
| 物理表名 | `moves` |
| 論理名 | ムーブ（操作ログ） / Moves (Operation Log) |
| 出典 | `crates/match-service/migrations/0040_game_sessions.sql:42-54` |
| 父文档 | RGS-DTL-038 §4.2 / §7.1 |
| 関連表 | `game_sessions` (N:1 CASCADE) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `move_id` | ムーブ ID / Move Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `match_id` | マッチ ID / Match Identifier | UUID | 128-bit | — | `game_sessions(match_id) ON DELETE CASCADE` | — | ✅ | — | — | 所属 session |
| 3 | `player_id` | プレイヤー ID / Player Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 操作玩家（跨 player_db）|
| 4 | `turn_index` | ターン番号 / Turn Index | INTEGER | >= 0 | — | — | — | ✅ | — | — | 操作所在回合 |
| 5 | `move_type` | 操作種別 / Move Type | SMALLINT | 0-5 | — | — | — | ✅ | — | — | 0=unspec 1=play_card 2=attack 3=end_turn 4=surrender 5=use_ability |
| 6 | `payload_json` | 操作ペイロード（JSONB） / Move Payload | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | move 输入（业务层解析）|
| 7 | `result_json` | 操作結果（JSONB） / Move Result | JSONB | — | — | — | — | ❌ | NULL | — | 业务层返回结果（拒绝时无结果）|
| 8 | `accepted` | 受理フラグ / Accepted Flag | BOOLEAN | — | — | — | — | ✅ | TRUE | — | 业务层是否接受 |
| 9 | `reject_reason` | 拒否理由 / Rejection Reason | TEXT | — | — | — | — | ❌ | NULL | — | accepted=false 时填 |
| 10 | `occurred_at` | 発生日時 / Occurred At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 业务发生时间 |
| 11 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 入库时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `moves_pkey` | B-tree (PK) | `(move_id)` | 主键（自动） |
| 2 | `idx_moves_match_id` | B-tree | `(match_id)` | FK 索引（CASCADE 性能）|
| 3 | `idx_moves_match_turn` | B-tree | `(match_id, turn_index)` | 按 session + 回合查询 |
| 4 | `idx_moves_player_id` | B-tree | `(player_id)` | 玩家操作历史 |
| 5 | `idx_moves_occurred_at` | B-tree | `(occurred_at)` | 按时间排序/分析 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `moves_pkey` | `(move_id)` |
| FOREIGN KEY | (隐式) | `(match_id) REFERENCES game_sessions(match_id) ON DELETE CASCADE` |
| (无 move_type CHECK) | — | `move_type` 用 SMALLINT 枚举（应用层校验）|
| (无 accepted 约束) | — | 建议 PH-2 加 `CHECK ((accepted = TRUE) = (reject_reason IS NULL))` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `game_sessions` | `moves.match_id → game_sessions.match_id` | CASCADE | ✅ |
| N:1 (跨域) | `players` (player_db) | `moves.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- `moves` 表高频写（亿级/年），按 RGS-BAS-007 §4 应分区——**未实施**——见 [17-P1-07](17-不合理设计识别与优化建议.md)
- 缺 `accepted` 与 `reject_reason` 一致性 CHECK
- 缺 `move_type` CHECK

---

## 4.5 `matchmaking_tickets` マッチメイキングチケット

### 概要

匹配队列 ticket 表（per DGS-DTL-038 §4.2 + §5.2）。`rank_score_min` / `rank_score_max` 限定分数范围。`match_id` 在状态变为 matched 时填。`expires_at` 默认 +5 分钟（应用层可覆盖）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `matchmaking_tickets` |
| 論理名 | マッチメイキングチケット / Matchmaking Tickets |
| 出典 | `crates/match-service/migrations/0040_game_sessions.sql:63-77` |
| 父文档 | RGS-DTL-038 §4.2 / §5.2 |
| 関連表 | `game_sessions` (1:0..1 弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `ticket_id` | チケット ID / Ticket Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `player_id` | プレイヤー ID / Player Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 入队玩家 |
| 3 | `mode` | ゲームモード / Game Mode | SMALLINT | 0-4 | — | — | — | ✅ | — | — | GameMode 枚举 |
| 4 | `rank_score_min` | ランクスコア下限 / Rank Score Min | INTEGER | — | — | — | — | ✅ | 0 | — | 分数范围下限 |
| 5 | `rank_score_max` | ランクスコア上限 / Rank Score Max | INTEGER | — | — | — | — | ✅ | 0 | — | 分数范围上限 |
| 6 | `deck_ref_card_id` | デッキ参照（カード ID） / Deck Reference (Card ID) | TEXT | catalog 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 使用的卡组引用 |
| 7 | `deck_ref_inst_id` | デッキ参照（インスタンス ID） / Deck Reference (Instance ID) | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 卡组实例 ID |
| 8 | `status` | チケット状態 / Ticket Status | SMALLINT | 1-4 | — | — | — | ✅ | 1 | — | 1=queued 2=matched 3=cancelled 4=expired |
| 9 | `match_id` | マッチ ID / Match Identifier | UUID | 128-bit | — | — (同库弱引用 → game_sessions, 应物化) | — | ❌ | NULL | — | 匹配成功时填 |
| 10 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 11 | `matched_at` | マッチ成立日時 / Matched At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 匹配成功时间 |
| 12 | `cancelled_at` | キャンセル日時 / Cancelled At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 取消时间 |
| 13 | `expires_at` | 有効期限 / Expiration Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now() + INTERVAL '5 minutes'` | — | 默认 5 分钟过期 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `matchmaking_tickets_pkey` | B-tree (PK) | `(ticket_id)` | 主键（自动） |
| 2 | `idx_tickets_player` | B-tree | `(player_id)` | 查玩家所有 ticket |
| 3 | `idx_tickets_status` | B-tree | `(status)` | 按状态筛选（队列/已匹配）|
| 4 | `idx_tickets_mode_score` | B-tree | `(mode, rank_score_min, rank_score_max)` | 匹配扫描（按模式 + 分数范围）|
| 5 | `idx_tickets_expires` | B-tree | `(expires_at)` | 过期扫描 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `matchmaking_tickets_pkey` | `(ticket_id)` |
| (无 status CHECK) | — | 应用层校验 |
| (无 min <= max CHECK) | — | 建议 PH-2 加 `CHECK (rank_score_min <= rank_score_max)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:0..1 | `game_sessions` | `matchmaking_tickets.match_id = game_sessions.match_id` | app-layer | ❌ 弱引用（**应物化**）|

### 既知偏差

- `match_id` 同库弱引用（应物化）——见 [17-P1-05](17-不合理设计识别与优化建议.md)
- 缺 `rank_score_min <= rank_score_max` CHECK

---

## 4.6 `session_subscriptions` セッション購読

### 概要

session 事件订阅表（per RGS-DTL-038 §4.2 SubscribeMatch 流式 RPC）。`UNIQUE (match_id, player_id)` 防重订阅。`closed_at` 标记订阅结束（玩家退出 session）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `session_subscriptions` |
| 論理名 | セッション購読 / Session Subscriptions |
| 出典 | `crates/match-service/migrations/0040_game_sessions.sql:86-94` |
| 父文档 | RGS-DTL-038 §4.2 |
| 関連表 | `game_sessions` (N:1 CASCADE) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `sub_id` | 購読 ID / Subscription Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `match_id` | マッチ ID / Match Identifier | UUID | 128-bit | — | `game_sessions(match_id) ON DELETE CASCADE` | `(match_id, player_id)` | ✅ | — | — | 订阅的 session |
| 3 | `player_id` | プレイヤー ID / Player Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | `(match_id, player_id)` | ✅ | — | — | 订阅玩家 |
| 4 | `full_first` | フル状態先行配信フラグ / Full State First Flag | BOOLEAN | — | — | — | — | ✅ | TRUE | — | 重连时是否先推完整状态 |
| 5 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 订阅时间 |
| 6 | `closed_at` | 購読終了日時 / Subscription Closed At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 结束订阅时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `session_subscriptions_pkey` | B-tree (PK) | `(sub_id)` | 主键（自动） |
| 2 | `session_subscriptions_match_id_player_id_key` | B-tree (UNIQUE) | `(match_id, player_id)` | 由 UNIQUE 约束自动创建 |
| 3 | `idx_session_subs_match` | B-tree | `(match_id)` | FK 索引 + 查 session 所有订阅 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `session_subscriptions_pkey` | `(sub_id)` |
| UNIQUE | `session_subscriptions_match_id_player_id_key` | `(match_id, player_id)` |
| FOREIGN KEY | (隐式) | `(match_id) REFERENCES game_sessions(match_id) ON DELETE CASCADE` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `game_sessions` | `session_subscriptions.match_id → game_sessions.match_id` | CASCADE | ✅ |
| N:1 (跨域) | `players` (player_db) | `session_subscriptions.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- 缺 `closed_at IS NULL OR closed_at > created_at` 一致性 CHECK

---

## 4.7 `outbox` アウトボックス（公共 / Match 域）

> 完整模板見 [13-Outbox 跨域模板](13-Outbox跨域模板.md)。

- **位置**：`match_db.outbox`
- **结构**：与模板 1:1 一致
- **特有应用层**：
  - `match.session.created` / `started` / `ended` / `cancelled`
  - `match.move.played` / `attack` / `end_turn` / `surrender`
  - `match.matchmaking.ticket_queued` / `matched` / `expired`

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/match-service/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` + `0040_game_sessions.sql` |
| DTL-016 | `docs/01-核心架构与设计模式/RGS-DTL-016_详细设计书.md` §3 / §3.2 |
| DTL-038 | `docs/01-核心架构与设计模式/RGS-DTL-038_*.md` §4.2 / §5.1 / §5.2 / §7.1 |

> 任何实际 schema 与本文档不一致之处，以 `crates/match-service/migrations/*.sql` 实际 SQL 为准。
