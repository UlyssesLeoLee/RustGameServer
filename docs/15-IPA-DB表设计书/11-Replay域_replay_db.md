# 11-Replay 域（replay_db）

> **本文件定位**：Replay 域 1 张表的詳細表設計書。覆盖回放元数据表（replays）。**PostgreSQL 仅存元数据，回放数据存对象存储**（per DEC-038-03 推荐 A）。

| 项目 | 内容 |
|---|---|
| 物理库 | `replay_db` |
| 担当 crate | `replay-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 1 |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/replay-service/migrations/0001_init.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 11.1 | `replays` | リプレイ（メタデータ） / Replays (Metadata Only) | 中期表 | 千万级（按 mode 7-90 天 TTL 清理）| 4 |

---

## 11.1 `replays` リプレイ（メタデータ）

### 概要

回放元数据表（per RGS-DTL-038 §3 DEC-038-03 + §7.1 #7）。**PostgreSQL 仅存元数据**（replay_id / match_id / players / mode / object_key / expires_at），**回放数据存对象存储**（cluster-ops S3-兼容 / LocalFs）。生命周期：RANKED 90 天 / CASUAL 7 天 / ROOM 30 天。

| 项目 | 内容 |
|---|---|
| 物理表名 | `replays` |
| 論理名 | リプレイ（メタデータ） / Replays (Metadata Only) |
| 出典 | `crates/replay-service/migrations/0001_init.sql:18-29` |
| 父文档 | RGS-DTL-038 §3 DEC-038-03 / §7.1 #7 |
| 関連表 | `players` (跨域弱引用×2), `game_sessions` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `replay_id` | リプレイ ID / Replay Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `match_id` | マッチ ID / Match Identifier | UUID | 128-bit | — | — (跨域弱引用 → game_sessions) | — | ✅ | — | — | 所属对局 |
| 3 | `player_a` | プレイヤー A / Player A | TEXT | UUID 文字列 | — | — (跨域弱引用 → players) | — | ✅ | — | — | 主玩家 A |
| 4 | `player_b` | プレイヤー B / Player B | TEXT | UUID 文字列 | — | — (跨域弱引用 → players) | — | ❌ | NULL | — | 玩家 B（多玩家模式必填，1v1 单玩家场景可空？）|
| 5 | `mode` | ゲームモード / Game Mode | SMALLINT | 1-4 | — | — | — | ✅ | — | — | 1=ranked 2=casual 3=room 4=pve_ai |
| 6 | `object_key` | オブジェクトキー（オブジェクトストレージ） / Object Key (Object Storage) | TEXT | 1-512 字符 | — | — | — | ✅ | — | — | 对象存储键（如 `replays/2026/09/01/UUID.bin`）|
| 7 | `object_size` | オブジェクトサイズ（バイト） / Object Size (Bytes) | BIGINT | >= 0 | — | — | — | ❌ | NULL | — | 对象大小（NULL = 待上传完成）|
| 8 | `duration_secs` | 試合時間（秒） / Match Duration (Seconds) | INTEGER | >= 0 | — | — | — | ❌ | NULL | — | 比赛时长 |
| 9 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 入库时间 |
| 10 | `expires_at` | 有効期限 / Expiration Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | — | — | 过期时间（per mode 7-90 天）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `replays_pkey` | B-tree (PK) | `(replay_id)` | 主键（自动）|
| 2 | `idx_replays_player_a` | B-tree | `(player_a)` | 查玩家 A 的所有回放（ListReplays）|
| 3 | `idx_replays_player_b` | B-tree | `(player_b)` | 查玩家 B 的所有回放（同上）|
| 4 | `idx_replays_match_id` | B-tree | `(match_id)` | 查某 match 的所有回放（一个 match 可能多次保存）|
| 5 | `idx_replays_expires` | B-tree | `(expires_at)` | 过期清理 cron job |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `replays_pkey` | `(replay_id)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (跨域) | `players` (player_db) × 2 | `player_a` / `player_b` | app-layer | ❌ 弱引用 |
| N:1 (跨域) | `game_sessions` (match_db) | `replays.match_id = game_sessions.match_id` | app-layer | ❌ 弱引用 |

### 既知偏差

- **缺 `mode` 与 `expires_at` 一致性约束**：mode 不同 → expires_at 偏移不同（7/30/90 天）——建议 PH-2 加应用层在 `created_at` 时计算 expires_at + DB 层 `CHECK (expires_at > created_at)`
- **缺 `player_b` 必填约束**：2v2/5v5/battle_royale 模式应至少有 2 个玩家，但当前 `player_b` 允许 NULL——见 [17-P1-14](17-不合理设计识别与优化建议.md)
- `object_size` / `duration_secs` 缺 CHECK (`>= 0`)
- 缺 `created_at` / `expires_at` 简单一致性 CHECK

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/replay-service/migrations/0001_init.sql` |
| DTL-038 | `docs/01-核心架构与设计模式/RGS-DTL-038_*.md` §3 DEC-038-03 / §7.1 #7 |
| DEC-038-03 | `docs/12-未决事项/RGS-DEC-038-03_*.md` |

> 任何实际 schema 与本文档不一致之处，以 `crates/replay-service/migrations/*.sql` 实际 SQL 为准。
