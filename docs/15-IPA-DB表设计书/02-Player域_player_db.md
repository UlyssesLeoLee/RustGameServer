# 02-Player 域（player_db）

> **本文件定位**：Player 域 6 张表的詳細表設計書（テーブル定義書）。覆盖 5 业务表 + 1 公共 outbox。

| 项目 | 内容 |
|---|---|
| 物理库 | `player_db` |
| 担当 crate | `player-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 6（含 outbox） |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/player-service/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` + `0004_player_characters_inventory.sql` + `0005_decks.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 2.1 | `players` | 玩家（アカウント） / Players (Accounts) | 永久事实表 | 百万〜千万级 | 3 |
| 2.2 | `player_sessions` | プレイヤーセッション / Player Sessions | 时序短期表 | 千万级/活跃 | 2 |
| 2.3 | `player_characters` | プレイヤーキャラクター / Player Characters | 永久事实表 | 千万级 | 4 + 1 JSONB-GIN |
| 2.4 | `player_inventory` | プレイヤーインベントリ / Player Inventory | 永久事实表 | 亿级 | 3 + 1 JSONB-GIN + 1 UK |
| 2.5 | `decks` | デッキ / Decks | 永久事实表 | 十万〜百万级 | 4 + 1 JSONB-GIN + 1 UK |
| 2.6 | `outbox` | アウトボックス（公共） / Outbox | 时序短期表 | 千万级/日 | 3（partial×2 + B-tree×1）|

---

## 2.1 `players` 玩家（アカウント）

### 概要

玩家账号主表。账号唯一由 `name` 约束（不依赖外部 SSO），支持 4 种状态（active / banned / disabled / pending），并以 `level` / `vip_level` 两个独立维度记录成长阶段。**不物化跨域 FK**（与 economy 域的 `accounts` 仅共享 `player_id` 弱引用）。`updated_at` 供 OCC 配合使用。

| 项目 | 内容 |
|---|---|
| 物理表名 | `players` |
| 論理名 | プレイヤー（アカウント） / Players (Accounts) |
| スキーマ | `public` |
| 出典 | `crates/player-service/migrations/0001_init.sql:5-15` |
| 父文档 | RGS-DTL-018 §3.1 / RGS-DTL-044 §2.1 |
| 関連表 | `player_sessions` (1:N CASCADE), `player_characters` (1:N CASCADE), `player_inventory` (1:N CASCADE), `decks` (1:N app-layer CASCADE), `accounts` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `name` | プレイヤー名 / Player Name | TEXT | 1-64 字符（应用层校验）| — | — | ✅ | ✅ | — | — | 全局唯一名 |
| 3 | `level` | アカウントレベル / Account Level | INTEGER | 1-999（应用层校验） | — | — | — | ✅ | 1 | — | 账号等级（独立于角色等级）|
| 4 | `vip_level` | VIP レベル / VIP Level | INTEGER | 0-20（应用层校验） | — | — | — | ✅ | 0 | — | VIP 等级 |
| 5 | `status` | アカウント状態 / Account Status | TEXT | — | — | — | — | ✅ | `'active'` | `status IN ('active', 'banned', 'disabled', 'pending')` | 4 状态枚举 |
| 6 | `last_login_at` | 最終ログイン日時 / Last Login Timestamp | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 跨会话追踪 |
| 7 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 8 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间（OCC 辅助） |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `players_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `players_name_key` | B-tree (UNIQUE) | `(name)` | 由 `name` UNIQUE 约束自动创建 |
| 3 | `idx_players_name` | B-tree | `(name)` | 与 (2) 重複 — 见 [17-不合理设计 P1-01](17-不合理设计识别与优化建议.md) |
| 4 | `idx_players_level` | B-tree | `(level)` | 按等级筛选（匹配池/排行榜）|
| 5 | `idx_players_status` | B-tree | `(status)` | 按状态筛选（封禁列表/激活列表）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `players_pkey` | `(id)` |
| UNIQUE | `players_name_key` | `(name)` |
| CHECK | `players_status_check` | `status IN ('active', 'banned', 'disabled', 'pending')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N | `player_sessions` | `player_sessions.player_id → players.id` | CASCADE | ✅ |
| 1:N | `player_characters` | `player_characters.player_id → players.id` | CASCADE | ✅ |
| 1:N | `player_inventory` | `player_inventory.player_id → players.id` | CASCADE | ✅ |
| 1:N | `decks` | `decks.owner_id → players.id` | app-layer CASCADE | ❌ 弱引用 |
| 1:N (跨域) | `accounts` (economy_db) | `accounts.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- `idx_players_name` 与 `players_name_key` 重複（既有现状，未清理；PH-2 评审时移除冗余索引）
- `metadata JSONB` 字段（per DTL-044 §2.1.1）尚未添加（属 0006+ future migration）

---

## 2.2 `player_sessions` プレイヤーセッション

### 概要

玩家登录会话记录。每次登录写入一条，活跃会话可由 `last_heartbeat_at` 与 `expires_at` 判定。会话期满由后台清理 job 处理（不在本文档展开）。支持玩家删除时的 CASCADE 清理。

| 项目 | 内容 |
|---|---|
| 物理表名 | `player_sessions` |
| 論理名 | プレイヤーセッション / Player Sessions |
| 出典 | `crates/player-service/migrations/0001_init.sql:22-32` |
| 父文档 | RGS-DTL-018 §3.2 / RGS-DTL-044 §3 |
| 関連表 | `players` (N:1 CASCADE) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | `players(id) ON DELETE CASCADE` | — | ✅ | — | — | 会话所属玩家 |
| 3 | `device_id` | デバイス ID / Device Identifier | TEXT | 1-256 字符 | — | — | — | ✅ | — | — | 登录设备指纹 |
| 4 | `ip` | IP アドレス / IP Address | TEXT | 1-45 字符 (IPv4/IPv6) | — | — | — | ✅ | — | — | 登录 IP（保留 30 天合规） |
| 5 | `login_at` | ログイン日時 / Login Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 登录时间 |
| 6 | `last_heartbeat_at` | 最終ハートビート / Last Heartbeat | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 用于活跃判定 |
| 7 | `expires_at` | 有効期限 / Expiration Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | — | — | TTL 过期时间（应用层计算） |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `player_sessions_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `idx_player_sessions_player_id` | B-tree | `(player_id)` | FK 索引（必备）+ 查某玩家所有会话 |
| 3 | `idx_player_sessions_expires_at` | B-tree | `(expires_at)` | TTL 过期清理 job |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `player_sessions_pkey` | `(id)` |
| FOREIGN KEY | (隐式) `player_sessions_player_id_fkey` | `(player_id) REFERENCES players(id) ON DELETE CASCADE` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `players` | `player_sessions.player_id → players.id` | CASCADE | ✅ |

### 既知偏差

- `ip` 列属 PII 范畴，但 NFR-SE-010 GDPR 双层审计通路未在 player_db 接入——per DTL-018 §3.2 应在应用层加 retention policy（PH-2 实施）

---

## 2.3 `player_characters` プレイヤーキャラクター

### 概要

玩家角色档案表（per RGS-DTL-044 §2.2）。每个玩家可有多个角色，**职业+等级**为匹配池分桶维度。HP/ATK/DEF/crit_rate 等高频战斗属性拆列（热路径 B-tree 索引），低频扩展属性存 JSONB。主武器走独立表（`player_inventory`）+ 外键引用，避免反范式。

| 项目 | 内容 |
|---|---|
| 物理表名 | `player_characters` |
| 論理名 | プレイヤーキャラクター / Player Characters |
| 出典 | `crates/player-service/migrations/0004_player_characters_inventory.sql:51-97` |
| 父文档 | RGS-DTL-044 v0.1 §2.2 / §2.3 / §4.2 |
| 関連表 | `players` (N:1 CASCADE), `player_inventory` (N:0..1 SET NULL) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | `players(id) ON DELETE CASCADE` (约束名 `fk_pc_player`) | — | ✅ | — | — | 所属玩家 |
| 3 | `char_class` | 職業クラス / Character Class | TEXT | — | — | — | — | ✅ | — | `char_class IN ('warrior', 'mage', 'archer', 'assassin', 'support')` | 5 选 1 职业 |
| 4 | `level` | キャラクターレベル / Character Level | INTEGER | 1-999 | — | — | — | ✅ | 1 | `level >= 1 AND level <= 999` | 角色等级（独立于账号等级）|
| 5 | `hp` | HP（体力） / HP (Health Points) | INTEGER | >= 0 | — | — | — | ✅ | 100 | `hp >= 0` | 生命值 |
| 6 | `atk` | 攻撃力 / Attack | INTEGER | >= 0 | — | — | — | ✅ | 10 | `atk >= 0` | 攻击力 |
| 7 | `def` | 防御力 / Defense | INTEGER | >= 0 | — | — | — | ✅ | 5 | `def >= 0` | 防御力 |
| 8 | `crit_rate` | クリティカル率 / Critical Rate | REAL | 0.0-1.0 | — | — | — | ✅ | 0.05 | `crit_rate >= 0.0 AND crit_rate <= 1.0` | 暴击率 |
| 9 | `stats` | 拡張ステータス（JSONB） / Extended Stats | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 抗性/buffs/debuffs/enchantments/talents/achievements |
| 10 | `primary_weapon_id` | 主力武器 ID / Primary Weapon Identifier | UUID | 128-bit | — | `player_inventory(id) ON DELETE SET NULL` (约束名 `fk_pc_weapon`) | — | ❌ | NULL | — | 主武器（弱引用 → 独立表）|
| 11 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 12 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `player_characters_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `idx_pc_player_id` | B-tree | `(player_id)` | FK 索引（CASCADE 性能）+ 查某玩家所有角色 |
| 3 | `idx_pc_class_level` | B-tree | `(char_class, level)` | 匹配池按职业+等级分桶（跨域给 match-service） |
| 4 | `idx_pc_weapon` | B-tree | `(primary_weapon_id)` | FK 索引（SET NULL 性能）+ 武器删除定位 |
| 5 | `idx_pc_stats_gin` | GIN | `stats` | JSONB 路径查询（`stats->'resistances'->>'fire' > 0.5` 等） |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `player_characters_pkey` | `(id)` |
| FOREIGN KEY | `fk_pc_player` | `(player_id) REFERENCES players(id) ON DELETE CASCADE` |
| FOREIGN KEY | `fk_pc_weapon` | `(primary_weapon_id) REFERENCES player_inventory(id) ON DELETE SET NULL` |
| CHECK | (隐式) `char_class_check` | `char_class IN ('warrior', 'mage', 'archer', 'assassin', 'support')` |
| CHECK | (隐式) `level_check` | `level >= 1 AND level <= 999` |
| CHECK | (隐式) `hp_check` | `hp >= 0` |
| CHECK | (隐式) `atk_check` | `atk >= 0` |
| CHECK | (隐式) `def_check` | `def >= 0` |
| CHECK | (隐式) `crit_rate_check` | `crit_rate >= 0.0 AND crit_rate <= 1.0` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `players` | `player_characters.player_id → players.id` | CASCADE | ✅ |
| N:0..1 | `player_inventory` | `player_characters.primary_weapon_id → player_inventory.id` | SET NULL | ✅ |

### 既知偏差

- cross-table FK `fk_pc_weapon` 在 2026-09-01 09:50 JST 通过 `DO $$ ... ALTER TABLE ...` 修复合入（per 0004 注释 line 170-183），避开 forward ref 问题——见 [17-P0-04](17-不合理设计识别与优化建议.md)
- 上述全部 CHECK 约束内联于 `CREATE TABLE IF NOT EXISTS` 块——按 0003 反 pattern 防御（line 31-37），在已部署 0001/0002/0003 环境下本表**必**不存在时块一定会执行

---

## 2.4 `player_inventory` プレイヤーインベントリ

### 概要

玩家背包物品表（per RGS-DTL-044 §2.4）。**槽位唯一**（`UNIQUE (player_id, slot)`）防重入 bug；物品主数据跨域弱引用（DB 层不强制 FK，应用层做存在性校验 + 缓存）。`metadata` JSONB 存附魔词条/绑定角色/到期时间等实例属性。

| 项目 | 内容 |
|---|---|
| 物理表名 | `player_inventory` |
| 論理名 | プレイヤーインベントリ / Player Inventory |
| 出典 | `crates/player-service/migrations/0004_player_characters_inventory.sql:117-151` |
| 父文档 | RGS-DTL-044 v0.1 §2.4 / §4.3 |
| 関連表 | `players` (N:1 CASCADE), `player_characters` (1:N SET NULL on weapon_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | `players(id) ON DELETE CASCADE` | — | ✅ | — | — | 所属玩家 |
| 3 | `item_id` | アイテムマスター ID / Item Master Identifier | UUID | 128-bit | — | — (跨域弱引用) | — | ✅ | — | — | 物品 master ID（跨域，应用层校验存在） |
| 4 | `quantity` | 数量 / Quantity | INTEGER | > 0 | — | — | — | ✅ | 1 | `quantity > 0` | 同槽位堆叠数 |
| 5 | `slot` | スロット番号 / Slot Number | INTEGER | 0-199 | — | — | `(player_id, slot)` UK | ✅ | — | `slot >= 0 AND slot < 200` | 背包槽位（200 格） |
| 6 | `acquired_at` | 取得日時 / Acquisition Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 获取时间 |
| 7 | `metadata` | インスタンスメタデータ（JSONB） / Instance Metadata | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 附魔/绑定/到期等 |
| 8 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 9 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `player_inventory_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `uq_pi_player_slot` | B-tree (UNIQUE) | `(player_id, slot)` | 槽位唯一（防重入 + 快速定位）|
| 3 | `idx_pi_player_id` | B-tree | `(player_id)` | FK 索引（CASCADE 性能）+ 查玩家所有物品 |
| 4 | `idx_pi_item_id` | B-tree | `(item_id)` | 查持有某物品的所有玩家（运营/补偿）|
| 5 | `idx_pi_acquired_at` | B-tree | `(acquired_at)` | 按获取时间排序/限时物品 |
| 6 | `idx_pi_metadata_gin` | GIN | `metadata` | JSONB 路径查询（查附魔=火属性的武器等）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `player_inventory_pkey` | `(id)` |
| UNIQUE | `uq_pi_player_slot` | `(player_id, slot)` |
| FOREIGN KEY | (隐式) | `(player_id) REFERENCES players(id) ON DELETE CASCADE` |
| CHECK | (隐式) `quantity_check` | `quantity > 0` |
| CHECK | (隐式) `slot_check` | `slot >= 0 AND slot < 200` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `players` | `player_inventory.player_id → players.id` | CASCADE | ✅ |
| 1:N (反向) | `player_characters` | `player_characters.primary_weapon_id → player_inventory.id` | SET NULL | ✅ |

### 既知偏差

- `item_id` 不物化 FK（per DTL-044 §2.4 跨域弱引用原则），应用层在 service 层加 existence check + 缓存；**风险**：item master 删改版本时无 DB 层强保证——见 [17-P1-02](17-不合理设计识别与优化建议.md)
- `metadata` JSONB 无 schema 校验函数（per RGS-IMPL-002 §3 PG 编码规范未硬性要求 JSONB schema 校验）；建议 PH-2 加 `CHECK (jsonb_typeof(metadata) = 'object')`

---

## 2.5 `decks` デッキ

### 概要

卡组表（per RGS-DTL-038 §7.1 #4 + FR-002 卡组 CRUD + share）。`slots` 存 JSONB 卡槽列表（业务层校验 30-60 张 + 同卡 ≤ 2 张规则）。`share_code` UNIQUE 用于公开分享。`is_public` 索引支持公开 deck 列表查询。

| 项目 | 内容 |
|---|---|
| 物理表名 | `decks` |
| 論理名 | デッキ / Decks |
| 出典 | `crates/player-service/migrations/0005_decks.sql:33-69` |
| 父文档 | RGS-DTL-038 v0.1 §4.3 / §7.1 / RGS-DEC-038-01（卡组归 player-service v2） |
| 関連表 | `players` (N:1 app-layer CASCADE), `cards` (跨域弱引用，slots[].card_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `deck_id` | デッキ ID / Deck Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `owner_id` | 所有者 ID / Owner Identifier | UUID | 128-bit | — | — (跨域弱引用, app-layer CASCADE) | — | ✅ | — | — | 所属玩家 |
| 3 | `name` | デッキ名 / Deck Name | TEXT | 1-128 字符 | — | — | — | ✅ | — | — | 卡组名 |
| 4 | `mode` | デッキモード / Deck Mode | SMALLINT | 1-4 | — | — | — | ✅ | — | `mode IN (1, 2, 3, 4)` | 1=ranked 2=casual 3=room 4=ai (待迁 common.v1.GameMode) |
| 5 | `slots` | カードスロット（JSONB） / Card Slots | JSONB | — | — | — | — | ✅ | `'[]'::jsonb` | — | `[{card_id, count}, ...]` |
| 6 | `status` | デッキ状態 / Deck Status | SMALLINT | 1-3 | — | — | — | ✅ | 1 | `status IN (1, 2, 3)` | 1=draft 2=active 3=archived |
| 7 | `is_public` | 公開フラグ / Public Flag | BOOLEAN | — | — | — | — | ✅ | FALSE | — | 是否公开分享 |
| 8 | `share_code` | 共有コード / Share Code | TEXT | 36 字符 (UUIDv4) | — | — | ✅ | ❌ | NULL | — | 公开分享码（仅 is_public=TRUE 时非空）|
| 9 | `like_count` | いいね数 / Like Count | INTEGER | >= 0 | — | — | — | ✅ | 0 | `like_count >= 0` | 点赞数（冗余字段，**需拆表** — 见 [17-P1-03](17-不合理设计识别与优化建议.md)）|
| 10 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 11 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `decks_pkey` | B-tree (PK) | `(deck_id)` | 主键（自动） |
| 2 | `decks_share_code_key` | B-tree (UNIQUE) | `(share_code)` | 由 `share_code` UNIQUE 约束自动创建 |
| 3 | `idx_decks_owner_id` | B-tree | `(owner_id)` | ListDecks 按玩家过滤 |
| 4 | `idx_decks_owner_updated` | B-tree | `(owner_id, updated_at DESC)` | ListDecks 默认排序 |
| 5 | `idx_decks_is_public` | B-tree | `(is_public)` | ListDecksPublic 公开 deck 列表 |
| 6 | `idx_decks_slots_gin` | GIN | `slots` | 按 card_id 查询某卡出现在哪些 deck |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `decks_pkey` | `(deck_id)` |
| UNIQUE | `decks_share_code_key` | `(share_code)` |
| CHECK | (隐式) `mode_check` | `mode IN (1, 2, 3, 4)` |
| CHECK | (隐式) `status_check` | `status IN (1, 2, 3)` |
| CHECK | (隐式) `like_count_check` | `like_count >= 0` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `players` (跨域) | `decks.owner_id = players.id` | app-layer CASCADE | ❌ 弱引用 |
| N:M (跨域) | `cards` (card_db) | `decks.slots[].card_id = cards.card_id` | app-layer 校验 | ❌ 弱引用 |

### 既知偏差

- `like_count` 冗余字段（`deck_likes` 应拆独立表，避免并发点赞竞态）——见 [17-P1-03](17-不合理设计识别与优化建议.md)
- `share_code` 应用层需保证"is_public=TRUE 时 share_code 非空"——DB 层无强约束；建议 PH-2 加 `CHECK ((is_public = FALSE AND share_code IS NULL) OR (is_public = TRUE AND share_code IS NOT NULL))`
- `owner_id` 不物化 FK（per DTL-038 §7.1 跨域弱引用原则），玩家删除时 application 层走 CASCADE 清理（per DTL-018 §3.1）

---

## 2.6 `outbox` アウトボックス（公共 / Player 域）

> 完整模板 + 6 域分布見 [13-Outbox 跨域模板](13-Outbox跨域模板.md)。此处仅标注 Player 域特殊点：

- **位置**：`player_db.outbox`
- **结构**：与模板 1:1 一致
- **特有应用层**：`player-service` 写入 outbox 的 subject 集合（partial list）：
  - `player.profile.updated` — 玩家档案更新
  - `player.character.created` — 角色创建
  - `player.character.deleted` — 角色删除
  - `player.inventory.item_added` — 物品获得
  - `player.inventory.item_removed` — 物品消耗
  - `player.deck.created` / `updated` / `deleted` / `shared`
- **关联表**：`sagas` (跨域弱引用，saga_id)
- **Known Drift**：0002 写入 outbox 但 CHECK 约束在已部署环境失效；0003 修复（per AGENTS.md §2.1 L1）

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/player-service/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` + `0004_player_characters_inventory.sql` + `0005_decks.sql` |
| DTL-018 | `docs/01-核心架构与设计模式/RGS-DTL-018_详细设计书.md` §3.1 / §3.2 |
| DTL-036 | `docs/01-核心架构与设计模式/RGS-DTL-036_Player域_详细设计书.md` |
| DTL-038 | `docs/01-核心架构与设计模式/RGS-DTL-038_*.md` §4.3 / §7.1 |
| DTL-044 | `docs/01-核心架构与设计模式/RGS-DTL-044_player数据_v0.1.md` §2.1 / §2.2 / §2.3 / §2.4 / §3 / §4.2 / §4.3 |
| DTL-100 | `docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式定义_v0.1.md` §5.3 |
| DEC-038-01 | `docs/12-未决事项/RGS-DEC-038-01_卡组归属_v0.1.md` |

> 任何实际 schema 与本文档不一致之处，以 `crates/player-service/migrations/*.sql` 实际 SQL 为准。
