# 08-Card 域（card_db）

> **本文件定位**：Card 域 3 张表的詳細表設計書。覆盖卡牌 catalog（cards / card_series）+ 玩家收藏（card_instances）。

| 项目 | 内容 |
|---|---|
| 物理库 | `card_db` |
| 担当 crate | `card-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 3 |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/card-service/migrations/0001_init.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 8.1 | `cards` | カード（カタログ） / Cards (Catalog) | 永久静态表 | 数千级 | 3 |
| 8.2 | `card_series` | カードシリーズ / Card Series (Packs) | 永久静态表 | 数百级 | 1 |
| 8.3 | `card_instances` | カードインスタンス（玩家收藏） / Card Instances (Player Collection) | 永久追加表 | 千万级 | 2 + 1 复合 |

---

## 8.1 `cards` カード（カタログ）

### 概要

卡牌 catalog 主表（per RGS-DTL-038 §7.1 #1）。`card_id` TEXT 业务 ID（catalog 编号）。`name_i18n` / `description_i18n` JSONB 多语言。`type` / `rarity` SMALLINT 枚举。`stats` JSONB 扩展属性。

| 项目 | 内容 |
|---|---|
| 物理表名 | `cards` |
| 論理名 | カード（カタログ） / Cards (Catalog) |
| 出典 | `crates/card-service/migrations/0001_init.sql:7-20` |
| 父文档 | RGS-DTL-038 §7.1 #1 |
| 関連表 | `card_series` (N:1 弱引用 series_id), `card_instances` (1:N 弱引用 card_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `card_id` | カード ID（カタログ番号） / Card ID (Catalog Number) | TEXT | 1-32 字符 | ✅ | — | — | ✅ | — | — | 主键（业务 ID，如 `CARD_001`）|
| 2 | `series_id` | シリーズ ID / Series Identifier | TEXT | 1-32 字符 | — | — (同库弱引用 → card_series) | — | ✅ | — | — | 所属卡包 |
| 3 | `name_default` | デフォルト名 / Default Name | TEXT | 1-128 字符 | — | — | — | ✅ | — | — | 默认名（fallback locale）|
| 4 | `name_i18n` | 多言語名（JSONB） / Multilingual Names | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | `{"zh_cn": "...", "en_us": "...", ...}` |
| 5 | `type` | カード種別 / Card Type | SMALLINT | 1-9（应用层校验） | — | — | — | ✅ | — | — | 1=creature 2=spell 3=trap ... |
| 6 | `rarity` | レアリティ / Rarity | SMALLINT | 1-5 | — | — | — | ✅ | — | — | 1=common 2=uncommon 3=rare 4=epic 5=legendary |
| 7 | `base_cost` | 基本コスト / Base Cost | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 出牌基本费用 |
| 8 | `description_i18n` | 多言語説明（JSONB） / Multilingual Descriptions | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 效果说明多语言 |
| 9 | `effect_ref` | 効果参照 / Effect Reference | TEXT | 1-256 字符 | — | — | — | ✅ | `''` | — | 效果函数/规则引用（如 `builtin.summon.dragon`）|
| 10 | `stats` | ステータス（JSONB） / Stats | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 攻击/生命/特殊属性（per DTL-038 §7.1 #1）|
| 11 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 入 catalog 时间 |
| 12 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `cards_pkey` | B-tree (PK) | `(card_id)` | 主键（自动）|
| 2 | `idx_cards_series` | B-tree | `(series_id)` | 按卡包筛选 |
| 3 | `idx_cards_rarity` | B-tree | `(rarity)` | 按稀有度筛选 |
| 4 | `idx_cards_type` | B-tree | `(type)` | 按类型筛选 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `cards_pkey` | `(card_id)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (同库) | `card_series` | `cards.series_id = card_series.series_id` | app-layer | ❌ 弱引用（**应物化**）|
| 1:N (同库) | `card_instances` | `card_instances.card_id = cards.card_id` | app-layer | ❌ 弱引用（**应物化**）|

### 既知偏差

- `series_id` 同库弱引用（应物化）——见 [17-P1-12](17-不合理设计识别与优化建议.md)
- 缺 `type` / `rarity` CHECK（应用层校验）
- `name_i18n` / `description_i18n` JSONB 无 locale 枚举约束（应用层校验 locale 必须在 `i18n_languages.locale` 内）

---

## 8.2 `card_series` カードシリーズ

### 概要

卡包 / 系列表（per RGS-DTL-038 §7.1 #2）。`series_id` TEXT 业务 ID。`pack_size` > 0（每包张数）。`drop_table` JSONB 掉落表（`{"version": 1, "snapshot_at": "...", "entries": [...]}`）。`price_type` / `price_amount` 卡包定价。

| 项目 | 内容 |
|---|---|
| 物理表名 | `card_series` |
| 論理名 | カードシリーズ / Card Series (Packs) |
| 出典 | `crates/card-service/migrations/0001_init.sql:26-36` |
| 父文档 | RGS-DTL-038 §7.1 #2 |
| 関連表 | `cards` (1:N 弱引用 series_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `series_id` | シリーズ ID / Series Identifier | TEXT | 1-32 字符 | ✅ | — | — | ✅ | — | — | 主键（业务 ID，如 `SERIES_GENESIS`）|
| 2 | `name_default` | デフォルト名 / Default Name | TEXT | 1-128 字符 | — | — | — | ✅ | — | — | 默认名 |
| 3 | `name_i18n` | 多言語名（JSONB） / Multilingual Names | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 多语言名 |
| 4 | `pack_size` | パックサイズ / Pack Size | INTEGER | > 0 | — | — | — | ✅ | — | `pack_size > 0` | 每包卡数 |
| 5 | `drop_table` | 排出テーブル（JSONB） / Drop Table | JSONB | — | — | — | — | ✅ | `'{"version":1,"snapshot_at":"","entries":[]}'::jsonb` | — | 掉落规则（含 version 快照）|
| 6 | `price_type` | 価格種別 / Price Type | SMALLINT | 1-3 | — | — | — | ✅ | 1 | — | 1=soft 2=hard 3=card_value |
| 7 | `price_amount` | 価格 / Price Amount | BIGINT | >= 0 | — | — | — | ✅ | 0 | — | 定价 |
| 8 | `released_at` | リリース日時 / Release Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 上线时间 |
| 9 | `status` | シリーズ状態 / Series Status | SMALLINT | 1-4 | — | — | — | ✅ | 1 | — | 1=活跃 2=待发布 3=失败 4=取消 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `card_series_pkey` | B-tree (PK) | `(series_id)` | 主键（自动）|
| 2 | `idx_card_series_status` | B-tree | `(status)` | 按状态筛选（活跃卡包列表）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `card_series_pkey` | `(series_id)` |
| CHECK | (隐式) `pack_size_check` | `pack_size > 0` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N (同库) | `cards` | `cards.series_id = card_series.series_id` | app-layer | ❌ 弱引用（**应物化**）|

### 既知偏差

- 缺 `status` CHECK（1-4 枚举应用层校验）
- 缺 `price_type` CHECK
- `drop_table` JSONB 无 schema 校验函数（建议 PH-2 加 `CHECK (jsonb_typeof(drop_table) = 'object' AND drop_table ? 'version' AND drop_table ? 'entries')`）

---

## 8.3 `card_instances` カードインスタンス（玩家收藏）

### 概要

玩家卡牌收藏表（per RGS-DTL-038 §7.1 #3）。`instance_id` UUID PK。`card_id` / `owner_id` 跨域弱引用。`attrs` JSONB 实例属性。`tradable` / `locked` 状态标志。

| 项目 | 内容 |
|---|---|
| 物理表名 | `card_instances` |
| 論理名 | カードインスタンス（玩家收藏） / Card Instances (Player Collection) |
| 出典 | `crates/card-service/migrations/0001_init.sql:40-50` |
| 父文档 | RGS-DTL-038 §7.1 #3 |
| 関連表 | `cards` (N:1 弱引用), `players` (跨域弱引用), `auctions` (1:N 弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `instance_id` | インスタンス ID / Instance Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `card_id` | カード ID / Card Identifier | TEXT | catalog 文字列 | — | — (同库弱引用 → cards) | — | ✅ | — | — | catalog 引用 |
| 3 | `owner_id` | 所有者 ID / Owner Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用 → players) | — | ✅ | — | — | 拥有者 player_id |
| 4 | `acquired_at` | 取得日時 / Acquisition Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 获得时间 |
| 5 | `source` | 取得元 / Acquisition Source | SMALLINT | 1-9（应用层校验） | — | — | — | ✅ | — | — | 1=gacha 2=trade 3=reward 4=event ... |
| 6 | `level` | インスタンスレベル / Instance Level | INTEGER | 1-999（应用层校验） | — | — | — | ✅ | 1 | — | 卡牌强化等级 |
| 7 | `attrs` | 個別属性（JSONB） / Instance Attributes | JSONB | — | — | — | — | ✅ | `'{}'::jsonb` | — | 附魔/词条/绑定角色等 |
| 8 | `tradable` | 取引可否フラグ / Tradable Flag | BOOLEAN | — | — | — | — | ✅ | TRUE | — | 是否可交易 |
| 9 | `locked` | ロックフラグ / Locked Flag | BOOLEAN | — | — | — | — | ✅ | FALSE | — | 是否锁定（锁定期间禁止任何操作）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `card_instances_pkey` | B-tree (PK) | `(instance_id)` | 主键（自动）|
| 2 | `idx_card_instances_owner` | B-tree | `(owner_id)` | 查玩家所有卡（背包视图）|
| 3 | `idx_card_instances_card` | B-tree | `(card_id)` | 查某 card 的所有 instance（运营/补偿）|
| 4 | `idx_card_instances_owner_acquired` | B-tree | `(owner_id, acquired_at DESC)` | 玩家最近获得卡（按时间倒序）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `card_instances_pkey` | `(instance_id)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (同库) | `cards` | `card_instances.card_id = cards.card_id` | app-layer | ❌ 弱引用（**应物化**）|
| N:1 (跨域) | `players` (player_db) | `card_instances.owner_id = players.id` | app-layer | ❌ 弱引用 |
| 1:N (跨域) | `auctions` (economy_db) | `auctions.card_instance_id = card_instances.instance_id` | app-layer | ❌ 弱引用 |
| 1:N (跨域) | `private_trades` (economy_db) | `private_trades.proposer_card_instance_id / counterparty_card_instance_id` | app-layer | ❌ 弱引用 |

### 既知偏差

- 缺 `level` CHECK（1-999 应用层校验）
- 缺 `source` CHECK（1-9 应用层校验）
- `card_id` / `owner_id` 同库/跨域弱引用（应物化）——见 [17-P1-12](17-不合理设计识别与优化建议.md)
- 缺 `tradable=FALSE` 与 `locked=TRUE` 一致性约束（锁定期间不可交易）——建议 PH-2 加 `CHECK (NOT (tradable = FALSE AND locked = FALSE))`（语义："不可交易" = "锁定"）

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/card-service/migrations/0001_init.sql` |
| DTL-038 | `docs/01-核心架构与设计模式/RGS-DTL-038_*.md` §7.1 #1-3 |

> 任何实际 schema 与本文档不一致之处，以 `crates/card-service/migrations/*.sql` 实际 SQL 为准。
