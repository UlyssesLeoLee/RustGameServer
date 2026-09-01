# 10-Leaderboard 域（leaderboard_db）

> **本文件定位**：Leaderboard 域 1 张表的詳細表設計書。覆盖排行榜条目表（leaderboard_entries）。

| 项目 | 内容 |
|---|---|
| 物理库 | `leaderboard_db` |
| 担当 crate | `leaderboard-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 1 |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/leaderboard-service/migrations/0001_init.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 10.1 | `leaderboard_entries` | ランキングエントリ / Leaderboard Entries | 中期表 | 千万级（季度重置）| 2 + 1 UK |

---

## 10.1 `leaderboard_entries` ランキングエントリ

### 概要

排行榜条目表（per RGS-REQ-038 §FR-007 + RGS-DTL-038 §3 DEC-038-02）。一条记录 = 一个玩家在一个榜单一个周期内的当前 rank/score/wins/losses。3 种 leaderboard_type × 4 种 period × season_id 维度。rank 通过复合索引 `(leaderboard_type, period, season_id, score DESC)` 获得。

| 项目 | 内容 |
|---|---|
| 物理表名 | `leaderboard_entries` |
| 論理名 | ランキングエントリ / Leaderboard Entries |
| 出典 | `crates/leaderboard-service/migrations/0001_init.sql:7-24` |
| 父文档 | RGS-REQ-038 §FR-007 / RGS-DTL-038 §3 / RGS-DEC-038-02 |
| 関連表 | `players` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `leaderboard_type` | ランキング種別 / Leaderboard Type | TEXT | — | — | — | `(leaderboard_type, period, season_id, player_id)` | ✅ | — | `leaderboard_type IN ('ranked', 'casual', 'collection')` | 3 选 1 类型 |
| 3 | `period` | 集計期間 / Period | TEXT | — | — | — | `(leaderboard_type, period, season_id, player_id)` | ✅ | — | `period IN ('weekly', 'monthly', 'seasonal', 'all_time')` | 4 选 1 周期 |
| 4 | `season_id` | シーズン ID / Season Identifier | TEXT | 0-64 字符 | — | — | `(leaderboard_type, period, season_id, player_id)` | ✅ | `''` | — | 赛季 ID（ranked 必填，其他可空字符串）|
| 5 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | — (跨域弱引用) | `(leaderboard_type, period, season_id, player_id)` | ✅ | — | — | 入榜玩家 |
| 6 | `display_name` | 表示名 / Display Name | TEXT | 1-64 字符 | — | — | — | ✅ | — | — | 玩家展示名（快照，跨改名场景）|
| 7 | `score` | スコア / Score | BIGINT | — | — | — | — | ✅ | 0 | — | 当前积分 |
| 8 | `wins` | 勝利数 / Wins | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 胜利数 |
| 9 | `losses` | 敗北数 / Losses | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 失败数 |
| 10 | `rank` | 順位 / Rank | INTEGER | 0-999999 | — | — | — | ✅ | 0 | — | 排名 1-based; 0 = 尚未入榜 |
| 11 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |
| 12 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `leaderboard_entries_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `leaderboard_entries_leaderboard_type_period_season_id_player_id_key` | B-tree (UNIQUE) | `(leaderboard_type, period, season_id, player_id)` | 由 UNIQUE 约束自动创建 |
| 3 | `idx_lb_type_period_season_score` | B-tree | `(leaderboard_type, period, season_id, score DESC)` | 排行榜查询（按分数倒序）|
| 4 | `idx_lb_player` | B-tree | `(player_id)` | 查玩家所有榜单位置（GetPlayerRank）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `leaderboard_entries_pkey` | `(id)` |
| UNIQUE | `leaderboard_entries_leaderboard_type_period_season_id_player_id_key` | `(leaderboard_type, period, season_id, player_id)` |
| CHECK | (隐式) `leaderboard_type_check` | `leaderboard_type IN ('ranked', 'casual', 'collection')` |
| CHECK | (隐式) `period_check` | `period IN ('weekly', 'monthly', 'seasonal', 'all_time')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (跨域) | `players` (player_db) | `leaderboard_entries.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- **UNIQUE `(leaderboard_type, period, season_id, player_id)` 在 `season_id = ''` 时的语义**：当 `period = 'all_time'` + `season_id = ''` 时，所有 all_time 榜单的玩家记录仍按"空字符串"算唯一，正常；但若同时存在 `period = 'seasonal'` + `season_id = 'ALL_TIME'` 也会按 ALL_TIME 字符串算唯一——`season_id` 命名约定应文档化（避免混淆）
- 缺 `rank` CHECK (`>= 0`)
- 缺 `wins` / `losses` CHECK (`>= 0`)
- **缺 `ranked` 与 `season_id` 一致性约束**：`leaderboard_type='ranked'` 时 `season_id` 必填（非空字符串）——建议 PH-2 加 `CHECK ((leaderboard_type = 'ranked' AND season_id <> '') OR (leaderboard_type <> 'ranked'))`
- 缺 `updated_at > created_at` 简单一致性约束

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/leaderboard-service/migrations/0001_init.sql` |
| REQ-038 | `docs/00-准备阶段/RGS-REQ-038_*.md` §FR-007 |
| DTL-038 | `docs/01-核心架构与设计模式/RGS-DTL-038_*.md` §3 |
| DEC-038-02 | `docs/12-未决事项/RGS-DEC-038-02_*.md` |

> 任何实际 schema 与本文档不一致之处，以 `crates/leaderboard-service/migrations/*.sql` 实际 SQL 为准。
