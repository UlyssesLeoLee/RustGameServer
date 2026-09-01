# 03-Economy 域（economy_db）

> **本文件定位**：Economy 域 8 张表的詳細表設計書。覆盖 6 核心表（accounts / ledger / sagas / reservations / inbox / outbox）+ 2 trade 表（auctions / private_trades）。

| 项目 | 内容 |
|---|---|
| 物理库 | `economy_db` |
| 担当 crate | `economy-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 8（含 outbox） |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/economy-service/migrations/0001_init.sql` + `0002_saga_init.sql` + `0003_outbox.sql` + `0004_outbox_check_idempotent.sql` + `0005_auctions.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 3.1 | `accounts` | アカウント（通貨別） / Accounts (Per-Currency) | 永久事实表 | 千万级（player×currency 笛卡尔积） | 2 |
| 3.2 | `transaction_ledger` | 取引台帳 / Transaction Ledger | 永久追加表 | 亿级 | 3 |
| 3.3 | `sagas` | サーガ / Sagas | 永久追加表 | 百万级 | 3 + 1 UK |
| 3.4 | `reservations` | リザベーション（資金留保）/ Reservations (Funds Hold) | 短期表 | 千万级（短期清理） | 4 |
| 3.5 | `inbox` | インボックス（冪等処理記録） / Inbox (Idempotency Log) | 短期表 | 千万级（短期清理） | 1 |
| 3.6 | `outbox` | アウトボックス（公共） / Outbox | 时序短期表 | 千万级/日 | 3 |
| 3.7 | `auctions` | オークション / Auctions | 中期表 | 万级活跃 | 4 |
| 3.8 | `private_trades` | プライベートトレード / Private Trades | 中期表 | 千级活跃 | 3 |

---

## 3.1 `accounts` アカウント（通貨別）

### 概要

玩家货币账户表（per RGS-DTL-015 §3）。**每个玩家每种货币一条记录**（UNIQUE 约束 `(player_id, currency)`），OCC 模式用 `version` 列 + `balance >= 0` CHECK 防止透支。3 种货币（gold / diamond / token）以 `currency` 枚举。3 状态（active / frozen / closed）支持封号/冻结/销户。

| 项目 | 内容 |
|---|---|
| 物理表名 | `accounts` |
| 論理名 | アカウント（通貨別） / Accounts (Per-Currency) |
| 出典 | `crates/economy-service/migrations/0001_init.sql:5-15` |
| 父文档 | RGS-DTL-015 §3 / RGS-DTL-100 §3.1 |
| 関連表 | `transaction_ledger` (1:N CASCADE), `reservations` (1:N RESTRICT) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | — (跨域弱引用) | `(player_id, currency)` | ✅ | — | — | 所属玩家（跨 player_db 弱引用）|
| 3 | `currency` | 通貨種別 / Currency Type | TEXT | 4-7 字符 | — | — | `(player_id, currency)` | ✅ | — | `currency IN ('gold', 'diamond', 'token')` | 3 选 1 货币 |
| 4 | `balance` | 残高 / Balance | BIGINT | >= 0 | — | — | — | ✅ | 0 | `balance >= 0` | 当前余额（不允许透支）|
| 5 | `version` | バージョン（OCC） / Version (OCC) | BIGINT | >= 0 | — | — | — | ✅ | 0 | — | 乐观锁版本号（应用层 `WHERE version = ?` 条件更新）|
| 6 | `status` | アカウント状態 / Account Status | TEXT | — | — | — | — | ✅ | `'active'` | `status IN ('active', 'frozen', 'closed')` | 3 状态枚举 |
| 7 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 8 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `accounts_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `accounts_player_id_currency_key` | B-tree (UNIQUE) | `(player_id, currency)` | 一玩家一货币唯一 |
| 3 | `idx_accounts_player_id` | B-tree | `(player_id)` | 查玩家所有货币账户（钱包视图）|
| 4 | `idx_accounts_status` | B-tree | `(status)` | 按状态筛选（冻结账户列表）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `accounts_pkey` | `(id)` |
| UNIQUE | `accounts_player_id_currency_key` | `(player_id, currency)` |
| CHECK | (隐式) `currency_check` | `currency IN ('gold', 'diamond', 'token')` |
| CHECK | (隐式) `balance_check` | `balance >= 0` |
| CHECK | (隐式) `status_check` | `status IN ('active', 'frozen', 'closed')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N | `transaction_ledger` | `transaction_ledger.account_id → accounts.id` | CASCADE | ✅ |
| 1:N | `reservations` | `reservations.account_id → accounts.id` | RESTRICT | ✅ |
| N:1 (跨域) | `players` (player_db) | `accounts.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- `player_id` 跨域弱引用（per RGS-SPEC-CROSS-005 §2）；玩家删除时 application 层走 CASCADE 清理（per DTL-018 §3.1）
- `version` OCC 由应用层在 SQL `WHERE version = ?` 条件更新时使用，DB 层无触发器（per ARC-023 极窄允许边界）

---

## 3.2 `transaction_ledger` 取引台帳

### 概要

交易账目表（per RGS-DTL-100 §6 幂等性 + Saga）。`idempotency_key` UNIQUE 是核心——保证"同一 command 重发不会重复入账"。`saga_id` + `command_id` 双链接 saga 上下文。5 种 `kind`（deposit / spend / transfer / refund / compensation）+ 4 状态（pending / confirmed / reversed / failed）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `transaction_ledger` |
| 論理名 | 取引台帳 / Transaction Ledger |
| 出典 | `crates/economy-service/migrations/0001_init.sql:21-33` |
| 父文档 | RGS-DTL-100 §6 幂等性 |
| 関連表 | `accounts` (N:1 CASCADE), `sagas` (弱引用, saga_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `account_id` | アカウント ID / Account Identifier | UUID | 128-bit | — | `accounts(id) ON DELETE CASCADE` | — | ✅ | — | — | 所属账户 |
| 3 | `idempotency_key` | 冪等キー / Idempotency Key | TEXT | 1-256 字符 | — | — | ✅ | ✅ | — | — | 全局幂等键（同 key 重发不入账）|
| 4 | `saga_id` | サーガ ID / Saga Identifier | UUID | 128-bit | — | — (同库弱引用, 可物化待 PH-2 评审) | — | ❌ | NULL | — | 关联 saga |
| 5 | `command_id` | コマンド ID / Command Identifier | UUID | 128-bit | — | — (同库弱引用) | — | ❌ | NULL | — | 关联 command |
| 6 | `amount` | 金額 / Amount | BIGINT | 不限（受 `accounts.balance` 约束） | — | — | — | ✅ | — | — | 金额（正负表示入/出）|
| 7 | `currency` | 通貨種別 / Currency Type | TEXT | 4-7 字符 | — | — | — | ✅ | — | `currency IN ('gold', 'diamond', 'token')` | 货币种类（与 account 同步）|
| 8 | `kind` | 取引種別 / Transaction Kind | TEXT | — | — | — | — | ✅ | — | `kind IN ('deposit', 'spend', 'transfer', 'refund', 'compensation')` | 5 种交易类型 |
| 9 | `status` | 取引状態 / Transaction Status | TEXT | — | — | — | — | ✅ | `'pending'` | `status IN ('pending', 'confirmed', 'reversed', 'failed')` | 4 状态机 |
| 10 | `memo` | メモ / Memo | TEXT | — | — | — | — | ❌ | NULL | — | 备注（业务上下文）|
| 11 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 入账时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `transaction_ledger_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `transaction_ledger_idempotency_key_key` | B-tree (UNIQUE) | `(idempotency_key)` | 由 UNIQUE 自动创建 |
| 3 | `idx_ledger_saga_id` | B-tree | `(saga_id)` | 查某 saga 的所有账目 |
| 4 | `idx_ledger_account_id` | B-tree | `(account_id)` | FK 索引（CASCADE 性能）+ 查某账户历史 |
| 5 | `idx_ledger_status` | B-tree | `(status)` | 按状态筛选（待确认/失败）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `transaction_ledger_pkey` | `(id)` |
| UNIQUE | `transaction_ledger_idempotency_key_key` | `(idempotency_key)` |
| FOREIGN KEY | (隐式) | `(account_id) REFERENCES accounts(id) ON DELETE CASCADE` |
| CHECK | (隐式) `currency_check` | `currency IN ('gold', 'diamond', 'token')` |
| CHECK | (隐式) `kind_check` | `kind IN ('deposit', 'spend', 'transfer', 'refund', 'compensation')` |
| CHECK | (隐式) `status_check` | `status IN ('pending', 'confirmed', 'reversed', 'failed')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `accounts` | `transaction_ledger.account_id → accounts.id` | CASCADE | ✅ |
| N:1 (同库) | `sagas` | `transaction_ledger.saga_id = sagas.id` | app-layer | ❌ 弱引用（待 PH-2 评审）|

### 既知偏差

- `saga_id` 同库弱引用——按 RGS-SPEC-CROSS-005 跨 DB 禁用外键，但同库内物化是允许的（待 PH-2 评审是否物化）

---

## 3.3 `sagas` サーガ

### 概要

Saga 状态机主表（per RGS-DTL-100 §3 Saga 状态机）。3 种 saga 类型（transfer / daily_reward / purchase），6 状态机（pending / running / compensating / completed / failed / aborted）。`command_id` UNIQUE 防止同 command 启动多个 saga。`steps` JSONB 存步骤定义。

| 项目 | 内容 |
|---|---|
| 物理表名 | `sagas` |
| 論理名 | サーガ / Sagas |
| 出典 | `crates/economy-service/migrations/0002_saga_init.sql:4-16` |
| 父文档 | RGS-DTL-100 §3 状态机 / §6 幂等性 |
| 関連表 | `reservations` (1:N CASCADE), `transaction_ledger` (1:N 弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `saga_type` | サーガ種別 / Saga Type | TEXT | — | — | — | — | ✅ | — | `saga_type IN ('transfer', 'daily_reward', 'purchase')` | 3 种 saga 类型 |
| 3 | `command_id` | コマンド ID / Command Identifier | UUID | 128-bit | — | — | ✅ | ✅ | — | — | 同 command 唯一启动一个 saga |
| 4 | `idempotency_key` | 冪等キー / Idempotency Key | TEXT | 1-256 字符 | — | — | — | ✅ | — | — | 客户端幂等键 |
| 5 | `current_step` | 現在ステップ / Current Step | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 当前执行步骤（0-based）|
| 6 | `steps` | ステップ定義（JSONB） / Step Definitions | JSONB | — | — | — | — | ✅ | `'[]'::jsonb` | — | saga 步骤定义数组 |
| 7 | `status` | サーガ状態 / Saga Status | TEXT | — | — | — | — | ✅ | `'pending'` | `status IN ('pending', 'running', 'compensating', 'completed', 'failed', 'aborted')` | 6 状态机 |
| 8 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 9 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |
| 10 | `completed_at` | 完了日時 / Completion Timestamp | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 完成时间（completed/failed/aborted 时填）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `sagas_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `uq_sagas_command_id` | B-tree (UNIQUE) | `(command_id)` | 同 command 唯一 |
| 3 | `idx_sagas_command_id` | B-tree | `(command_id)` | 与 (2) **重複** — 见 [17-P1-04](17-不合理设计识别与优化建议.md) |
| 4 | `idx_sagas_status` | B-tree | `(status)` | 按状态筛选（运行中/失败补偿）|
| 5 | `idx_sagas_idempotency_key` | B-tree | `(idempotency_key)` | 客户端重发查重 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `sagas_pkey` | `(id)` |
| UNIQUE | `uq_sagas_command_id` | `(command_id)` |
| CHECK | (隐式) `saga_type_check` | `saga_type IN ('transfer', 'daily_reward', 'purchase')` |
| CHECK | (隐式) `status_check` | `status IN ('pending', 'running', 'compensating', 'completed', 'failed', 'aborted')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N | `reservations` | `reservations.saga_id → sagas.id` | CASCADE | ✅ |
| 1:N (同库) | `transaction_ledger` | `transaction_ledger.saga_id = sagas.id` | app-layer | ❌ 弱引用 |
| 1:N (跨域/同域) | `outbox` × 6 域 | `outbox.saga_id = sagas.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- `idx_sagas_command_id` 与 `uq_sagas_command_id` 在 `command_id` 列上重複（既有现状，PH-2 评审时合并）
- `current_step` 无 CHECK（`>= 0` 由应用层保证），建议 PH-2 加 `CHECK (current_step >= 0)`

---

## 3.4 `reservations` リザベーション

### 概要

资金预留表（per RGS-DTL-100 §3.2 Reservation）。Saga 启动时预留金额，confirmed 时转 transaction_ledger，compensated 时回滚，expired 时定时清理。`expires_at` 由应用层在 saga 启动时计算（典型 5-30 分钟）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `reservations` |
| 論理名 | リザベーション（資金留保） / Reservations (Funds Hold) |
| 出典 | `crates/economy-service/migrations/0002_saga_init.sql:24-34` |
| 父文档 | RGS-DTL-100 §3.2 |
| 関連表 | `sagas` (N:1 CASCADE), `accounts` (N:1) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `saga_id` | サーガ ID / Saga Identifier | UUID | 128-bit | — | `sagas(id) ON DELETE CASCADE` | — | ✅ | — | — | 所属 saga |
| 3 | `account_id` | アカウント ID / Account Identifier | UUID | 128-bit | — | `accounts(id)` (无 ON DELETE 子句 = NO ACTION) | — | ✅ | — | — | 预留的账户 |
| 4 | `amount` | 留保金額 / Reserved Amount | BIGINT | > 0 | — | — | — | ✅ | — | `amount > 0` | 预留金额（必须为正）|
| 5 | `currency` | 通貨種別 / Currency Type | TEXT | 4-7 字符 | — | — | — | ✅ | — | `currency IN ('gold', 'diamond', 'token')` | 货币种类 |
| 6 | `status` | 留保状態 / Reservation Status | TEXT | — | — | — | — | ✅ | `'reserved'` | `status IN ('reserved', 'confirmed', 'compensated', 'expired')` | 4 状态机 |
| 7 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 8 | `expires_at` | 有効期限 / Expiration Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | — | — | TTL 过期时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `reservations_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `idx_reservations_saga_id` | B-tree | `(saga_id)` | FK 索引（CASCADE 性能）|
| 3 | `idx_reservations_account_id` | B-tree | `(account_id)` | FK 索引 + 查账户预留总额 |
| 4 | `idx_reservations_status` | B-tree | `(status)` | 按状态筛选 |
| 5 | `idx_reservations_expires_at` | B-tree | `(expires_at)` | 过期扫描 job |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `reservations_pkey` | `(id)` |
| FOREIGN KEY | (隐式) | `(saga_id) REFERENCES sagas(id) ON DELETE CASCADE` |
| FOREIGN KEY | (隐式) | `(account_id) REFERENCES accounts(id)` (NO ACTION) |
| CHECK | (隐式) `amount_check` | `amount > 0` |
| CHECK | (隐式) `currency_check` | `currency IN ('gold', 'diamond', 'token')` |
| CHECK | (隐式) `status_check` | `status IN ('reserved', 'confirmed', 'compensated', 'expired')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `sagas` | `reservations.saga_id → sagas.id` | CASCADE | ✅ |
| N:1 | `accounts` | `reservations.account_id → accounts.id` | NO ACTION | ✅ |

### 既知偏差

- `reservations` 数据会随时间膨胀——按 RGS-BAS-007 §4 规范 `outbox` 类高频写表应分区，但 reservations 暂无分区（**建议 PH-2 评审**：按 `created_at` 月度分区 + 7 天保留期）

---

## 3.5 `inbox` インボックス

### 概要

Inbox 幂等处理记录（per RGS-DTL-100 §6）。`UNIQUE (command_id, handler)` 保证同 (command, handler) 仅处理一次，`status` 区分 processed / failed 两种终态。

| 项目 | 内容 |
|---|---|
| 物理表名 | `inbox` |
| 論理名 | インボックス（冪等処理記録） / Inbox (Idempotency Log) |
| 出典 | `crates/economy-service/migrations/0002_saga_init.sql:42-50` |
| 父文档 | RGS-DTL-100 §6 |
| 関連表 | (无 FK) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `command_id` | コマンド ID / Command Identifier | UUID | 128-bit | — | — | `(command_id, handler)` | ✅ | — | — | 处理过的 command |
| 3 | `handler` | ハンドラ名 / Handler Name | TEXT | 1-128 字符 | — | — | `(command_id, handler)` | ✅ | — | — | handler 标识（class.method）|
| 4 | `result` | 処理結果（JSON 文字列） / Processing Result | TEXT | JSON 文字列 | — | — | — | ✅ | `'{}'` | — | handler 返回结果 |
| 5 | `status` | 処理状態 / Processing Status | TEXT | — | — | — | — | ✅ | `'processed'` | `status IN ('processed', 'failed')` | 2 终态 |
| 6 | `processed_at` | 処理日時 / Processed At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 处理时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `inbox_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `inbox_command_id_handler_key` | B-tree (UNIQUE) | `(command_id, handler)` | 由 UNIQUE 约束自动创建 |
| 3 | `idx_inbox_processed_at` | B-tree | `(processed_at)` | 过期清理 + 监控 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `inbox_pkey` | `(id)` |
| UNIQUE | `inbox_command_id_handler_key` | `(command_id, handler)` |
| CHECK | (隐式) `status_check` | `status IN ('processed', 'failed')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| (无) | — | — | — | — |

### 既知偏差

- `inbox` 数据会随时间膨胀——按 RGS-BAS-007 §4 规范应按时间分区，**建议 PH-2 评审**：按 `processed_at` 月度分区 + 30 天保留期
- `result` 用 TEXT 存 JSON 字符串——建议 PH-2 改 `JSONB` 以支持 GIN 索引（按需）

---

## 3.6 `outbox` アウトボックス（公共 / Economy 域）

> 完整模板 + 6 域分布見 [13-Outbox 跨域模板](13-Outbox跨域模板.md)。

- **位置**：`economy_db.outbox`
- **结构**：与模板 1:1 一致
- **特有应用层**：
  - `economy.transaction.confirmed` — 交易确认
  - `economy.saga.started` / `step.completed` / `compensating` / `completed` / `failed`
  - `economy.account.frozen` / `unfrozen` / `closed`
- **关联表**：`sagas` (同域弱引用，saga_id)

---

## 3.7 `auctions` オークション

### 概要

公开拍卖表（per RGS-DTL-038 §7.1 #8 + DEC-038-04 trade 域归属）。`auctions` + `private_trades` 复用 economy_db（per ARC-008 + DEC-038-04 A 方案）。跨域 saga ExecuteAuction 需联动 card-service 转移卡牌实例（W36+ 接入）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `auctions` |
| 論理名 | オークション / Auctions |
| 出典 | `crates/economy-service/migrations/0005_auctions.sql:32-49` |
| 父文档 | RGS-DTL-038 §7.1 #8 / RGS-DEC-038-04 |
| 関連表 | `players` (跨域弱引用), `card_instances` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `auction_id` | オークション ID / Auction Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `seller_id` | 出品者 ID / Seller Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 出品玩家 |
| 3 | `card_id` | カード ID / Card Identifier | TEXT | catalog 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 卡牌 catalog 引用 |
| 4 | `card_instance_id` | カードインスタンス ID / Card Instance Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 卡牌实例引用 |
| 5 | `min_price` | 最低落札価格 / Minimum Price | BIGINT | >= 0 | — | — | — | ✅ | — | `min_price >= 0` | 起拍价 / 一口价 |
| 6 | `currency_type` | 通貨種別（オークション用） / Currency Type (Auction) | SMALLINT | 1-3 | — | — | — | ✅ | — | `currency_type IN (1, 2, 3)` | 1=soft 2=hard 3=card_value (per common.proto CurrencyType) |
| 7 | `highest_bid` | 最高入札額 / Highest Bid | BIGINT | >= 0 | — | — | — | ✅ | 0 | `highest_bid >= 0` | 当前最高价（0 = 无人）|
| 8 | `highest_bidder` | 最高入札者 / Highest Bidder | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | `''` | — | 当前最高出价者（'' = 无人）|
| 9 | `status` | オークション状態 / Auction Status | SMALLINT | 1-4 | — | — | — | ✅ | 1 | `status IN (1, 2, 3, 4)` | 1=active 2=sold 3=cancelled 4=expired |
| 10 | `started_at` | 開始日時 / Start Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 开始时间 |
| 11 | `ends_at` | 終了日時 / End Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | — | — | 结束时间 |
| 12 | `closed_at` | 閉鎖日時 / Closed At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 成交/撤单/过期时间（NULL = 进行中）|
| 13 | `winner_id` | 落札者 ID / Winner Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 成交时买家 ID |
| 14 | `final_price` | 最終価格 / Final Price | BIGINT | — | — | — | — | ✅ | 0 | — | 成交价（0 = 未成交）|
| 15 | `saga_id` | サーガ ID / Saga Identifier | UUID | 128-bit | — | — (跨域弱引用, 跨域) | — | ❌ | NULL | — | 关联 ExecuteAuction saga |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `auctions_pkey` | B-tree (PK) | `(auction_id)` | 主键（自动） |
| 2 | `idx_auctions_status` | B-tree | `(status)` | 列表查询（活跃拍卖）|
| 3 | `idx_auctions_seller` | B-tree | `(seller_id)` | 玩家历史（我卖出的）|
| 4 | `idx_auctions_highest_bidder` | partial B-tree | `(highest_bidder) WHERE highest_bidder <> ''` | 玩家历史（我参与的）|
| 5 | `idx_auctions_ends_at` | partial B-tree | `(ends_at) WHERE status = 1` | 过期扫描 job |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `auctions_pkey` | `(auction_id)` |
| CHECK | (隐式) `min_price_check` | `min_price >= 0` |
| CHECK | (隐式) `currency_type_check` | `currency_type IN (1, 2, 3)` |
| CHECK | (隐式) `highest_bid_check` | `highest_bid >= 0` |
| CHECK | (隐式) `status_check` | `status IN (1, 2, 3, 4)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (跨域) | `players` (player_db) | `auctions.seller_id = players.id` | app-layer | ❌ 弱引用 |
| N:1 (跨域) | `cards` (card_db) | `auctions.card_id = cards.card_id` | app-layer | ❌ 弱引用 |
| N:1 (跨域) | `card_instances` (card_db) | `auctions.card_instance_id = card_instances.instance_id` | app-layer | ❌ 弱引用 |
| N:1 (跨域) | `sagas` (本库) | `auctions.saga_id = sagas.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- `seller_id` / `winner_id` / `highest_bidder` 用 TEXT 而非 UUID——**不一致**：其他域用 UUID，trade 表用 TEXT 跨服务兼容；建议 PH-2 统一为 UUID（但跨服务边界要确认）
- `currency_type` 用 SMALLINT 枚举（per common.proto CurrencyType）—— 待 PH-2 迁 `currency_type` 到 `common.v1.CurrencyType` 共享 proto
- 缺 `seller_id` 的部分索引（仅 `highest_bidder` 有 partial index）

---

## 3.8 `private_trades` プライベートトレード

### 概要

私下交易表（per RGS-DTL-038 §7.1 #8 + DEC-038-04 A 方案）。W36+ 跨域 saga 实装后填 schema 字段，当前仅建表。`proposer_id` / `counterparty_id` 双方各出货币 + 卡牌实例。

| 项目 | 内容 |
|---|---|
| 物理表名 | `private_trades` |
| 論理名 | プライベートトレード / Private Trades |
| 出典 | `crates/economy-service/migrations/0005_auctions.sql:60-75` |
| 父文档 | RGS-DTL-038 §7.1 #8 / RGS-DEC-038-04 / §6.3 (跨域 saga) |
| 関連表 | `players` (跨域弱引用×2), `card_instances` (跨域弱引用×2) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `trade_id` | トレード ID / Trade Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `proposer_id` | 提案者 ID / Proposer Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 提案玩家 |
| 3 | `counterparty_id` | 相手方 ID / Counterparty Identifier | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ✅ | — | — | 对方玩家 |
| 4 | `status` | トレード状態 / Trade Status | SMALLINT | 1-4 | — | — | — | ✅ | 1 | `status IN (1, 2, 3, 4)` | 1=proposed 2=accepted 3=completed 4=cancelled |
| 5 | `proposer_currency_amount` | 提案者通貨額 / Proposer Currency Amount | BIGINT | — | — | — | — | ✅ | 0 | — | 提案者出多少货币 |
| 6 | `proposer_currency_type` | 提案者通貨種別 / Proposer Currency Type | SMALLINT | 1-3 | — | — | — | ❌ | NULL | — | 提案者货币种类 |
| 7 | `proposer_card_instance_id` | 提案者カードインスタンス / Proposer Card Instance | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 提案者出哪张卡 |
| 8 | `counterparty_currency_amount` | 相手方通貨額 / Counterparty Currency Amount | BIGINT | — | — | — | — | ✅ | 0 | — | 对方出多少货币 |
| 9 | `counterparty_currency_type` | 相手方通貨種別 / Counterparty Currency Type | SMALLINT | 1-3 | — | — | — | ❌ | NULL | — | 对方货币种类 |
| 10 | `counterparty_card_instance_id` | 相手方カードインスタンス / Counterparty Card Instance | TEXT | UUID 文字列 | — | — (跨域弱引用) | — | ❌ | NULL | — | 对方出哪张卡 |
| 11 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 12 | `closed_at` | 閉鎖日時 / Closed At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 接受/取消时间 |
| 13 | `saga_id` | サーガ ID / Saga Identifier | UUID | 128-bit | — | — (同库弱引用) | — | ❌ | NULL | — | 关联跨域 trade saga |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `private_trades_pkey` | B-tree (PK) | `(trade_id)` | 主键（自动） |
| 2 | `idx_private_trades_proposer` | B-tree | `(proposer_id)` | 提案者历史 |
| 3 | `idx_private_trades_counterparty` | B-tree | `(counterparty_id)` | 对方历史 |
| 4 | `idx_private_trades_status` | B-tree | `(status)` | 按状态筛选（待接受/已完成）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `private_trades_pkey` | `(trade_id)` |
| CHECK | (隐式) `status_check` | `status IN (1, 2, 3, 4)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (跨域) | `players` (player_db) × 2 | `proposer_id` / `counterparty_id` | app-layer | ❌ 弱引用 |
| N:1 (跨域) | `card_instances` (card_db) × 2 | `proposer_card_instance_id` / `counterparty_card_instance_id` | app-layer | ❌ 弱引用 |
| N:1 (同库) | `sagas` (本库) | `private_trades.saga_id = sagas.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- **未物化 FK**：5 处跨域/同库弱引用，W36+ 跨域 saga 实施时需补强
- `status` 缺 `CHECK ((status IN (3, 4)) = (closed_at IS NOT NULL))` 一致性约束
- `proposer_id <> counterparty_id` 应用层校验，DB 层无强约束（建议 PH-2 加 `CHECK (proposer_id <> counterparty_id)`）

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/economy-service/migrations/0001_init.sql` + `0002_saga_init.sql` + `0003_outbox.sql` + `0004_outbox_check_idempotent.sql` + `0005_auctions.sql` |
| DTL-015 | `docs/01-核心架构与设计模式/RGS-DTL-015_详细设计书.md` |
| DTL-037 | `docs/01-核心架构与设计模式/RGS-DTL-037_Economy域_详细设计书.md` |
| DTL-100 | `docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式定义_v0.1.md` §3 / §3.2 / §6 |
| DTL-038 | `docs/01-核心架构与设计模式/RGS-DTL-038_*.md` §6.3 / §7.1 #8 |
| DEC-038-04 | `docs/12-未决事项/RGS-DEC-038-04_*.md`（trade 域归属） |

> 任何实际 schema 与本文档不一致之处，以 `crates/economy-service/migrations/*.sql` 实际 SQL 为准。
