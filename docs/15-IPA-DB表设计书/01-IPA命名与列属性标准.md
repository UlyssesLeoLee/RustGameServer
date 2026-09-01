# 01-IPA 命名与列属性标准（命名規約 + データ型 + 列属性標準 / IPA Naming & Column Property Standard）

> **本文件定位**：RGS 仓库全部 42 张表的**列属性标准化定義**——按 JIS X 0123 + IPA SLCP-JCF2013 詳細設計工程 + RGS-BAS-007 既定標准三層基準，定義物理名 / 論理名 / データ型 / 桁数 / PK / FK / NOT NULL / DEFAULT / CHECK / 索引 / 説明 的**统一填写规范**。
>
> 后续 02〜12 域表设计書的所有列均**遵循本规范**填表。本规范不重複 RGS-BAS-007 §2 命名规范——本规范是 RGS-BAS-007 §2 的**操作化扩展**。

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-DB-COL-STD |
| 版本 | 0.1（IPA 標準初版） |
| 作成日 | 2026-09-01 JST |
| 適用基準 | JIS X 0123 / IPA SLCP-JCF2013 §6 詳細設計 / RGS-BAS-007 §2 |
| 適用範囲 | RGS 全部 PG 库 + SQLite 库全部 42 张表 |

---

## 1. 物理名 / 論理名 命名規約（Naming Convention）

### 1.1 物理名（Physical / 物理名 / 英字名）

| 对象 | 規約 | 例 | 違反例 | 出典 |
|---|---|---|---|---|
| 数据库名 | `<domain>_db`（snake_case 全小写） | `player_db`, `economy_db` | `PlayerDB`, `PLAYER_DB` | RGS-BAS-007 §2 |
| 表名 | snake_case 复数或领域名词 | `players`, `transaction_ledger`, `player_characters` | `Player`, `TransactionLedger`, `PLAYERS` | RGS-BAS-007 §2 |
| 列名 | snake_case，与 RGS-BAS-004 §4.3 API/日志字段同名概念拼写一致 | `player_id`, `created_at`, `retry_count` | `PlayerId`, `CreatedAt`, `RetryCount` | RGS-BAS-007 §2 |
| 主键列 | `id`（UUID 类型） | `id` | `player_id`（仅 PK 自身叫 `id`） | RGS-BAS-001 §5.8 |
| 外键列 | `<被引用表名单数>_id`（不强制全大写或复数） | `player_id`, `guild_id`, `account_id` | `PlayerId`, `playerID` | RGS-BAS-007 §2 |
| 时间戳列 | `<event>_at`（动词过去式 + _at） | `created_at`, `updated_at`, `started_at`, `expires_at`, `joined_at` | `CreatedAt`, `create_time`, `createDate` | RGS-BAS-007 §2 + JIS X 0123 |
| 布尔列 | `is_<adjective>` 或 `<verb>ed` / `enabled` | `is_public`, `is_mvp`, `enabled`, `accepted`, `disabled` | `public`, `mvp`, `IsPublic` | RFC 2119 + 慣用 |
| 状态列 | `status`（TEXT + CHECK 约束枚举值） | `status IN ('active', 'banned', ...)` | `state`, `is_active` | RGS-DTL-100 §3 状态机 |
| 枚举列（业务） | `kind`, `type`, `category`, `level`（小写 snake） + CHECK | `kind IN ('deposit', 'spend', ...)` | `Kind` | RGS-BAS-001 §5 |
| 索引名 | `idx_<表名>_<列名或用途简写>` | `idx_outbox_pending`, `idx_players_name` | `outbox_pending_idx`, `OutboxPending` | RGS-BAS-007 §2 |
| 唯一索引名 | `uq_<表名>_<列名或用途简写>` | `uq_sagas_command_id`, `uq_pi_player_slot` | `unique_sagas_command_id` | RGS-BAS-007 §2（推断） |
| 外键约束名 | `fk_<表名>_<被引用表名>`（同库内） | `fk_pc_player`, `fk_pc_weapon` | `fk_player_inventory`（双向反） | RGS-BAS-007 §2 |
| CHECK 约束名 | `chk_<表名>_<列名或语义>` | `chk_outbox_status`, `chk_merge_conflict_lock_consistency` | `outbox_status_check` | RGS-BAS-007 §2（推断） |
| 部分索引谓词 | `WHERE <条件>` 全大写关键字 | `WHERE status = 'pending'` | `where status = 'pending'` | PG 慣用 |

### 1.2 論理名（Logical / 論理名 / 和文名 + 英文名）

> IPA 標準要求**每列有 論理名**（业务视角）。本规范使用**双语論理名**：`<和文名> / <English Logical Name>`。

| 物理名 | 論理名（和文 / English） | 例 |
|---|---|---|
| `id` | 識別子 / Identifier | — |
| `player_id` | 玩家 ID / Player Identifier | 玩家 ID / Player Identifier |
| `created_at` | 作成日時 / Creation Timestamp | — |
| `updated_at` | 更新日時 / Update Timestamp | — |
| `status` | 状態 / Status | — |
| `kind` | 種別 / Category | — |
| `version` | バージョン（楽観ロック用）/ Version (OCC) | — |
| `prev_hash` | 前回ハッシュ値 / Previous Hash Value | — |
| `hash` | ハッシュ値 / Hash Value | — |
| `payload` | ペイロード / Payload | — |
| `subject` | NATS 件名 / NATS Subject | — |
| `command_id` | コマンド ID / Command Identifier | — |
| `saga_id` | サーガ ID / Saga Identifier | — |
| `retry_count` | リトライ回数 / Retry Count | — |
| `lease_until` | リース期限 / Lease Expiration | — |
| `last_error` | 最新エラーメッセージ / Latest Error | — |
| `sent_at` | 送信日時 / Send Timestamp | — |
| `idempotency_key` | 冪等キー / Idempotency Key | — |
| `current_step` | 現在ステップ番号 / Current Step Number | — |
| `steps` | ステップ定義（JSONB）/ Step Definitions | — |
| `compensation_step` | 補償対象ステップ / Compensation Target Step | — |
| `amount` | 金額 / Amount | — |
| `currency` | 通貨種別 / Currency Type | — |
| `balance` | 残高 / Balance | — |
| `expires_at` | 有効期限 / Expiration Timestamp | — |
| `room_id` | ルーム ID / Room Identifier | — |
| `room_code` | ルームコード（公開用） / Room Code (Public) | — |
| `room_password_hash` | ルームパスワードハッシュ（要 bcrypt/argon2） / Room Password Hash | — |
| `winner_team` | 勝利チーム / Winning Team | — |
| `mode` | ゲームモード / Game Mode | — |
| `score`, `kills`, `deaths`, `assists` | スコア・キル数・デス数・アシスト数 / Score, Kills, Deaths, Assists | — |
| `is_mvp` | MVP フラグ / MVP Flag | — |
| `turn_index` | ターン番号 / Turn Index | — |
| `move_type` | 操作種別 / Move Type | — |
| `board_snapshot` | 盤面スナップショット（JSONB） / Board Snapshot | — |
| `board_snapshot_ref` | 盤面スナップショット参照（オブジェクトストレージ） / Board Snapshot Reference (Object Storage) | — |
| `name` | 名称 / Name | — |
| `description` | 説明 / Description | — |
| `level` | レベル / Level | — |
| `experience` | 経験値 / Experience | — |
| `contribution` | 貢献度 / Contribution | — |
| `role` | 役割（RBAC ロール） / Role (RBAC Role) | — |
| `domain_scope` | ドメインスコープ（RBAC 適用範囲） / Domain Scope (RBAC) | — |
| `disabled_at` | 無効化日時 / Disabled Timestamp | — |
| `last_login_at` | 最終ログイン日時 / Last Login Timestamp | — |
| `actor_id` | 操作者 ID / Actor Identifier | — |
| `action` | 操作 / Action | — |
| `target` | 操作対象 / Target | — |
| `feature_subtype` | 機能サブタイプ / Feature Subtype | — |
| `realm_id` | レルム ID / Realm Identifier | — |
| `operator_id` | 実行者 ID / Operator Identifier | — |
| `request_id` | リクエスト ID / Request Identifier | — |
| `approval_ref` | 承認参照 / Approval Reference | — |
| `trace_id` | トレース ID / Trace Identifier | — |
| `target_region` | 対象リージョン / Target Region | — |
| `target_player_count` | 対象プレイヤー数 / Target Player Count | — |
| `target_tps` | 対象 TPS / Target TPS | — |
| `source_realm_id` | 元レルム ID / Source Realm Identifier | — |
| `target_realm_count` | 対象レルム数 / Target Realm Count | — |
| `split_strategy` | 分裂戦略 / Split Strategy | — |
| `rule_set_version` | ルールセットバージョン / Rule Set Version | — |
| `rules` | ルール（JSONB） / Rules | — |
| `locked_at` | ロック日時 / Lock Timestamp | — |
| `locked_by` | ロック実行者 / Locked By | — |
| `target_realm_id` | 対象レルム ID（退場対象）/ Target Realm ID (Retire Target) | — |
| `archive_threshold_days` | アーカイブ閾値日数 / Archive Threshold Days | — |
| `query_channel_rbac` | クエリチャネル RBAC 設定（JSONB） / Query Channel RBAC Config | — |
| `hot_storage_tier` | ホットストレージ階層 / Hot Storage Tier | — |
| `cold_storage_tier` | コールドストレージ階層 / Cold Storage Tier | — |
| `hot_retention_years` | ホット保持年数 / Hot Retention Years | — |
| `cold_retention_years` | コールド保持年数 / Cold Retention Years | — |
| `n_plus_2_redundancy` | N+2 冗長フラグ / N+2 Redundancy Flag | — |
| `hostname`, `ip` | ホスト名 / IP アドレス / Hostname, IP Address | — |
| `enabled` | 有効フラグ / Enabled Flag | — |
| `scope` | スコープ（global / domain / node） / Scope | — |
| `scope_value` | スコープ値 / Scope Value | — |
| `version` (feature_flags) | フラグバージョン（OCC） / Flag Version (OCC) | — |
| `last_heartbeat_at` | 最終ハートビート / Last Heartbeat | — |
| `registered_at` | 登録日時 / Registered At | — |
| `card_id` | カード ID（catalog 参照） / Card ID (Catalog Reference) | — |
| `card_instance_id` | カードインスタンス ID / Card Instance ID | — |
| `series_id` | シリーズ ID / Series ID | — |
| `name_default` | デフォルト名称 / Default Name | — |
| `name_i18n` | 多言語名称（JSONB） / Multilingual Names | — |
| `description_i18n` | 多言語説明（JSONB） / Multilingual Descriptions | — |
| `rarity` | レアリティ / Rarity | — |
| `type` (cards) | カード種別 / Card Type | — |
| `base_cost` | 基本コスト / Base Cost | — |
| `effect_ref` | 効果参照 / Effect Reference | — |
| `stats` | ステータス（JSONB） / Stats | — |
| `pack_size` | パックサイズ / Pack Size | — |
| `drop_table` | 排出テーブル（JSONB） / Drop Table | — |
| `price_type` | 価格種別 / Price Type | — |
| `price_amount` | 価格 / Price Amount | — |
| `released_at` | リリース日時 / Release Timestamp | — |
| `source` (card_instances) | 取得元 / Acquisition Source | — |
| `attrs` | 個別属性（JSONB） / Instance Attributes | — |
| `tradable` | 取引可否フラグ / Tradable Flag | — |
| `locked` (card_instances) | ロックフラグ / Locked Flag | — |
| `key` (i18n_texts) | テキストキー / Text Key | — |
| `locale` | ロケール / Locale | — |
| `text` (i18n_texts) | ローカライズ済テキスト / Localized Text | — |
| `display_name` | 表示名 / Display Name | — |
| `is_default` | デフォルトルール / Default Rule | — |
| `leaderboard_type` | ランキング種別 / Leaderboard Type | — |
| `period` | 集計期間 / Period | — |
| `season_id` | シーズン ID / Season Identifier | — |
| `rank` | 順位（1-based; 0=未入）/ Rank (1-based; 0=Unranked) | — |
| `wins`, `losses` | 勝利数・敗北数 / Wins, Losses | — |
| `object_key` | オブジェクトキー（オブジェクトストレージ） / Object Key (Object Storage) | — |
| `object_size` | オブジェクトサイズ（バイト） / Object Size (Bytes) | — |
| `duration_secs` | 試合時間（秒） / Match Duration (Seconds) | — |
| `char_class` | 職業クラス / Character Class | — |
| `hp`, `atk`, `def` | HP / 攻撃力 / 防御力 / HP, Attack, Defense | — |
| `crit_rate` | クリティカル率 / Critical Rate | — |
| `primary_weapon_id` | 主力武器 ID / Primary Weapon Identifier | — |
| `quantity` | 数量 / Quantity | — |
| `slot` | スロット番号（背包位置） / Slot Number (Inventory Position) | — |
| `acquired_at` | 取得日時 / Acquisition Timestamp | — |
| `metadata` (inventory) | 個別メタデータ（JSONB） / Item Instance Metadata | — |
| `mode` (decks) | デッキモード / Deck Mode | — |
| `slots` (decks) | カードスロット（JSONB） / Card Slots | — |
| `is_public` | 公開フラグ / Public Flag | — |
| `share_code` | 共有コード / Share Code | — |
| `like_count` | いいね数 / Like Count | — |
| `host_id` | ホスト ID / Host Identifier | — |
| `max_players`, `min_players` | 最大 / 最小プレイヤー数 / Max/Min Players | — |
| `current_player_id` | 現在手番プレイヤー ID / Current Turn Player ID | — |
| `next_turn_deadline_ms` | 次ターン締切（エポック ms） / Next Turn Deadline (Epoch ms) | — |
| `winner_id` (game_sessions) | 勝者 ID / Winner Identifier | — |
| `end_reason` | 終了理由 / End Reason | — |
| `ai_difficulty` | AI 難易度 / AI Difficulty | — |
| `timeout_count` | 累計タイムアウト回数 / Total Timeout Count | — |
| `pending_moves` | 保留中操作（JSONB） / Pending Moves | — |
| `started_at`, `ended_at` | 開始日時 / 終了日時 / Start Timestamp, End Timestamp | — |
| `match_id` (moves / session_subs) | マッチ ID / Match Identifier | — |
| `player_id` (moves / tickets) | プレイヤー ID（TEXT 形式） / Player Identifier (TEXT Form) | — |
| `turn_index` (moves) | ターン番号 / Turn Index | — |
| `move_type` | 操作種別 / Move Type | — |
| `payload_json` | 操作ペイロード（JSONB） / Move Payload | — |
| `result_json` | 操作結果（JSONB） / Move Result | — |
| `accepted` | 受理フラグ / Accepted Flag | — |
| `reject_reason` | 拒否理由 / Rejection Reason | — |
| `occurred_at` | 発生日時 / Occurred At | — |
| `ticket_id` | チケット ID / Ticket Identifier | — |
| `rank_score_min`, `rank_score_max` | ランクスコア範囲 / Rank Score Range | — |
| `deck_ref_card_id` | デッキ参照（カード ID） / Deck Reference (Card ID) | — |
| `deck_ref_inst_id` | デッキ参照（インスタンス ID） / Deck Reference (Instance ID) | — |
| `matched_at`, `cancelled_at` | マッチ / キャンセル日時 / Matched/Cancelled Timestamp | — |
| `expires_at` (tickets) | チケット有効期限 / Ticket Expiration | — |
| `sub_id` | 購読 ID / Subscription Identifier | — |
| `full_first` | フル状態先行配信フラグ / Full State First Flag | — |
| `closed_at` (subs) | 購読終了日時 / Subscription Closed At | — |
| `device_id` | デバイス ID / Device Identifier | — |
| `ip` (player_sessions) | IP アドレス / IP Address | — |
| `login_at` | ログイン日時 / Login Timestamp | — |
| `last_heartbeat_at` (sessions) | 最終ハートビート / Last Heartbeat | — |
| `expires_at` (sessions) | セッション有効期限 / Session Expiration | — |
| `auction_id` | オークション ID / Auction Identifier | — |
| `seller_id` | 出品者 ID / Seller Identifier | — |
| `min_price` | 最低落札価格 / Minimum Price | — |
| `currency_type` | 通貨種別（オークション用） / Currency Type (Auction) | — |
| `highest_bid` | 最高入札額 / Highest Bid | — |
| `highest_bidder` | 最高入札者 / Highest Bidder | — |
| `ends_at` (auctions) | 終了日時 / End Timestamp | — |
| `closed_at` (auctions) | 閉鎖日時 / Closed At | — |
| `final_price` | 最終価格 / Final Price | — |
| `trade_id` | 取引 ID / Trade Identifier | — |
| `proposer_id`, `counterparty_id` | 提案者 / 相手方 ID / Proposer, Counterparty | — |
| `proposer_currency_amount`, `counterparty_currency_amount` | 提案者 / 相手方 通貨額 / Proposer/Counterparty Currency Amount | — |
| `proposer_currency_type`, `counterparty_currency_type` | 提案者 / 相手方 通貨種別 / Proposer/Counterparty Currency Type | — |
| `proposer_card_instance_id`, `counterparty_card_instance_id` | 提案者 / 相手方 カードインスタンス ID / Proposer/Counterparty Card Instance ID | — |
| `token_id`, `asset_id`, `payload` (resume_tokens) | トークン ID / アセット ID / ペイロード (BLOB) / Token ID, Asset ID, Payload (BLOB) | — |
| `payload_size` (resume_tokens) | ペイロードサイズ / Payload Size | — |
| `created_at`, `updated_at`, `expires_at` (resume_tokens) | 作成 / 更新 / 有効期限（ISO8601 TEXT）/ Created/Updated/Expires (ISO8601 TEXT) | — |

---

## 2. データ型 規約（Data Type Convention）

### 2.1 PostgreSQL 18 データ型

| 用途 | PG 18 型 | 桁数 / 範囲 | 採用根拠 | 違反例 / 推奨置換 |
|---|---|---|---|---|
| 主键（UUID） | `UUID` | 128-bit | 跨域安全、不可枚举 | ❌ `SERIAL` / `INT`（不採用） |
| 外键 UUID | `UUID` | 128-bit | 跨域引用一致 | ❌ `TEXT`（仅当被引用方为 TEXT 时，如 `card_id`） |
| 货币余额 | `BIGINT` | -9.22e18 〜 9.22e18 | 不浮点、避免精度损失 | ❌ `NUMERIC(20,2)`（重）+ ❌ `FLOAT`（精度） |
| 数量/计数 | `BIGINT` 或 `INTEGER` | 视业务量级 | `transaction_ledger.amount` 用 `BIGINT`；`level` 用 `INTEGER` | ❌ `SMALLINT` for amount（可能溢出） |
| 小数概率 | `REAL` | 6 位精度 | `crit_rate` 0.0〜1.0 | ❌ `DOUBLE PRECISION`（过精）+ ❌ `NUMERIC`（重） |
| 枚举字符串 | `TEXT` + `CHECK` | 无长度限制 | PG 慣用，添加枚举值不需 DDL | ❌ `VARCHAR(16)` for status（限 16 字符可能不够） |
| 枚举 SMALLINT | `SMALLINT` | -32768 〜 32767 | 与 proto 枚举值对应（`common.v1.GameMode` 等） | ❌ `INT`（浪费）+ 跨服务 proto 同步 |
| 短字符串 ID | `TEXT` | 无长度限制 | `card_id`, `share_code`, `room_code` 等业务 ID | ❌ `VARCHAR(36)`（限 UUID 长不够） |
| 中等字符串 | `VARCHAR(N)` | 限 N 字符 | `subject` VARCHAR(256)（NATS subject 限长） | ❌ `TEXT`（不限长会滥用） |
| 长字符串 | `TEXT` | 无长度限制 | `description`, `payload`, `memo`, `name_i18n`（JSONB 序列化） | ❌ `VARCHAR(65535)`（PG 中 VARCHAR(>10485760) 报錯） |
| 布尔 | `BOOLEAN` | TRUE/FALSE | 标准 PG 类型 | ❌ `SMALLINT 0/1` |
| 时间戳 | `TIMESTAMPTZ` | 64-bit | 带时区，UTC 存储 | ❌ `TIMESTAMP`（无时区）+ ❌ `INTEGER` epoch（时区不可读） |
| 文本时间戳 | `TEXT`（ISO8601） | SQLite 専用 | `resume_tokens.created_at` 等 | 仅 SQLite |
| 二进制 | `BYTEA` | 视配置 | 不採用 | 需用时再决定 |
| JSON 文档 | `JSONB` | 视数据 | 二进制存储 + GIN 索引支持 | ❌ `JSON`（文本存储，无 GIN） |
| 二进制 blob | `BLOB` | SQLite 専用 | `resume_tokens.payload` | 仅 SQLite |
| 整数主键备选 | `BIGSERIAL` | 1 〜 9.22e18 | 不採用（统一 UUID） | 全部用 UUID |

### 2.2 SQLite 3 データ型（rgs-asset-download）

| 用途 | SQLite 型 | 採用根拠 |
|---|---|---|
| 主键 | `TEXT` | `token_id` 是业务 UUID 字符串 |
| 短字符串 | `TEXT` | 全部列皆文本 |
| 时间戳 | `TEXT`（ISO8601 字符串） | 异构存储，应用层序列化/反序列化 |
| 二进制 | `BLOB` | `payload` 字段 |
| 整数 | `INTEGER` | `payload_size` |

### 2.3 桁数 / 範囲 規約（Length / Range Convention）

> 全部 INTEGER / BIGINT 默认范围已由 PG 保证；以下仅列出**业务上限约束**。

| 列 | 業務上限 | 约束方式 | 出典 |
|---|---|---|---|
| `version` (OCC) | INT64 | `BIGINT` | — |
| `balance` (accounts) | 单次操作 1e15 之内 | `BIGINT CHECK (balance >= 0)` | `crates/economy-service/migrations/0001_init.sql:9` |
| `amount` (ledger) | 同上 | `BIGINT` | 同上 `:28` |
| `quantity` (inventory) | 1 〜 999 | `INTEGER CHECK (quantity > 0)`（上限由业务补强） | `crates/player-service/migrations/0004_*.sql:132-133` |
| `slot` (inventory) | 0 〜 199（背包 200 格） | `INTEGER CHECK (slot >= 0 AND slot < 200)` | 同上 `:136-137` |
| `level` (characters) | 1 〜 999 | `INTEGER CHECK (level >= 1 AND level <= 999)` | 同上 `:64-65` |
| `hp`, `atk`, `def` | >= 0 | `INTEGER CHECK (... >= 0)` | 同上 `:68-73` |
| `crit_rate` | 0.0 〜 1.0 | `REAL CHECK (crit_rate >= 0.0 AND crit_rate <= 1.0)` | 同上 `:74-75` |
| `archive_threshold_days` | 30 〜 90 | `INT CHECK (archive_threshold_days BETWEEN 30 AND 90)` | `crates/cluster-ops/migrations/0020_*.sql:156-157` |
| `hot_retention_years` | >= 3 | `INT CHECK (hot_retention_years >= 3)` | 同上 `:181` |
| `cold_retention_years` | >= 10 | `INT CHECK (cold_retention_years >= 10)` | 同上 `:182` |
| `target_realm_count` | >= 2 | `INT CHECK (target_realm_count >= 2)` | 同上 `:107` |
| `target_player_count`, `target_tps` | > 0 | `INT CHECK (... > 0)` | 同上 `:88-89` |
| `pack_size` (card_series) | > 0 | `INT CHECK (pack_size > 0)` | `crates/card-service/migrations/0001_init.sql:30` |
| `min_price`, `highest_bid` (auctions) | >= 0 | `BIGINT CHECK (... >= 0)` | `crates/economy-service/migrations/0005_auctions.sql:37-39` |
| `share_code` | UUIDv4 字符串 (36 字符) | `TEXT UNIQUE` | `crates/player-service/migrations/0005_decks.sql:60` |
| `subject` (outbox) | 256 字符 | `VARCHAR(256) NOT NULL` | `crates/*/migrations/0002_outbox.sql:39` |
| `status` (outbox) | 16 字符 | `VARCHAR(16) NOT NULL DEFAULT 'pending'` | 同上 `:43` |
| `password_hash` | bcrypt(60 字符) / argon2(96+ 字符) | `TEXT NOT NULL` | `crates/admin-service/migrations/0001_init.sql:11` |
| `room_password_hash` | 同上 | `TEXT`（可空） | `crates/match-service/migrations/0040_game_sessions.sql:16` |
| `hash` (audit_log) | SHA-256 hex = 64 字符 | `TEXT NOT NULL UNIQUE` | `crates/admin-service/migrations/0001_init.sql:30` |
| `next_turn_deadline_ms` | 64-bit epoch ms | `BIGINT` | `crates/match-service/migrations/0040_game_sessions.sql:21` |
| `max_players`, `min_players` | INT 默认 2 | `INT NOT NULL DEFAULT 2` | 同上 `:17-18` |
| `turn_index` | INT | `INT NOT NULL DEFAULT 0` | 同上 `:19` |

---

## 3. 列属性填表規約（Column Property Filling Convention）

每张表设计书按以下列布局填写（每列含义）：

| 列名 | 填法 | 説明 |
|---|---|---|
| # | 1-based 序号 | 物理顺序 |
| 物理名 (Physical) | snake_case 英文 | 列名（无类型） |
| 論理名 (Logical) | `<和文名> / <English>` | 双语論理名 |
| データ型 (Data Type) | PG 18 类型 / SQLite 类型 | 见 §2 |
| 桁数 / 範囲 (Length / Range) | 视类型而定 | 见 §2.3 |
| PK | ✅ / ❌ | 主键 |
| FK | `<表名>.<列> [ON DELETE/UPDATE <action>]` | 外键；跨域/跨库弱引用填 `—`（弱引用） |
| UK | ✅ / ❌ | 唯一约束（非 PK） |
| NOT NULL | ✅ / ❌ | 必填 |
| DEFAULT | 字面值或表达式 | 默认值 |
| CHECK | 约束表达式 | CHECK 约束内联 |
| 説明 (Description) | 1-3 行业务语义 | 1 行说明 |

### 3.1 索引 / 制約 填表規約

| 段 | 填法 | 例 |
|---|---|---|
| 索引一覧 | `<idx 名> ON <表>(<列>[, ...]) [USING <方法>] [WHERE <谓词>]` | `idx_outbox_pending ON outbox (created_at) WHERE status = 'pending'` |
| 制約一覧 | `<種別> <名> <表达式>` | `UNIQUE (player_id, slot)`, `FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE`, `CHECK (status IN (...))` |
| 関連表一覧 | `<表名>.<列> [方向]` + 关系説明 | `players.id ← player_characters.player_id (1:N CASCADE)` |

---

## 4. 跨库弱引用表記規約（Cross-Domain Weak Reference Convention）

> per RGS-SPEC-CROSS-005 §2 "跨 DB 禁用外键"——应用层校验责任。

| 表記 | 含义 | 应用层责任 |
|---|---|---|
| FK 列填 `— (弱引用)` | 跨 DB / 跨域 / 物化会引入强耦合 | 写入前 existence check + 删除后 tombstone 处理 |
| FK 列填 `→ <表名>.<列> (弱引用, app-layer)` | 同上但更明确方向 | 同上 + 在 repository 层加 `_ensure_exists` helper |
| FK 列填 `→ <表名>.<列> [CASCADE]` | 同库内物化 FK | DB 强保证 |

> **跨域弱引用清单**（共 26 处）见 [00-总览 §4](00-总览与全表清单.md#4-跨域引用矩阵cross-domain-reference-matrix)。

---

## 5. 既知偏差（Known Drift）— 本规范与 RGS-BAS-007 的差异

| 差异 | 现状 | RGS-BAS-007 §2 表述 | 本规范选择 | 修正 |
|---|---|---|---|---|
| CHECK 约束名 | 实际命名 `chk_<表名>_<列名>` (例: `chk_outbox_status`, `chk_merge_conflict_lock_consistency`) | RGS-BAS-007 §2 未规定 CHECK 命名 | 采用 `chk_<表名>_<列名或语义>` 慣用 | 建议 RGS-BAS-007 v0.3 补全 |
| 唯一索引名 | 实际命名 `uq_<表名>_<列名>` (例: `uq_sagas_command_id`, `uq_pi_player_slot`, `uq_merge_conflict_rule_set_version`, `uq_lifecycle_run_request_operator`) | RGS-BAS-007 §2 未规定 UK 命名 | 采用 `uq_<表名>_<列名>` 慣用 | 建议 RGS-BAS-007 v0.3 补全 |
| OUTBOX 表的 `version` 列 | 无（OCC 由 `sagas.version` 提供） | 未规定 | 不加，避免和 saga version 混淆 | — |
| 監査日 (`created_at` 全部 `TIMESTAMPTZ`) | 全部使用 `TIMESTAMPTZ` ✅ | RGS-BAS-007 未明示 | 沿用 TIMESTAMPTZ | — |
| 跨域引用 | 弱引用 + app-layer 校验 | RGS-SPEC-CROSS-005 §2 規定 | 一致 | — |
| SQLite 異種存儲 | 仅 `rgs-asset-download` 用 SQLite | 未规定 | 保留异构（asset-download 单服务轻量），详见 [12-AssetDownload](12-AssetDownload域_downloads_sqlite.md) | — |

---

## 6. 修订追溯

| 引用 | 路径 |
|---|---|
| RGS 命名規範 | `docs/03-数据经济与交易/RGS-BAS-007_数据库设计标准与存储过程规范_基本设计书.md` §2 |
| 跨域 FK 禁則 | `docs/13-实施规范/RGS-SPEC-CROSS-005_数据库设计约束_v0.1.md` §2 |
| OCC version 列 | `docs/01-核心架构与设计模式/RGS-BAS-001_基本设计书.md` §5.8 |
| 5 域 RBAC 角色 | `docs/13-实施规范/RGS-SPEC-CROSS-007_5域RBAC角色定义_v0.1.md` |
| IPA 共通フレーム 2013 | `https://www.ipa.go.jp/sec/publish/tn12-005.html`（外部参考） |
| JIS X 0123 | 日本産業標準調査会（外部参考） |

> 本文档不重複 RGS-BAS-007 §2 命名规范——本规范是 RGS-BAS-007 §2 的**操作化扩展**。后续 02〜12 域表设计書均遵循本规范填表。
