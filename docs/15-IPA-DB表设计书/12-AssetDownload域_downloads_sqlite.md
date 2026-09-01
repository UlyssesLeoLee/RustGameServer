# 12-AssetDownload 域（downloads.sqlite / SQLite 異種存儲）

> **本文件定位**：AssetDownload 域 1 张表的詳細表設計書。**异构存储**：本域使用 SQLite 3 而非 PostgreSQL（与 RGS 主库不同），**无 PII 字段**（per FR-CDN-064）。

| 项目 | 内容 |
|---|---|
| 物理库 | `downloads.sqlite` |
| 担当 crate | `rgs-asset-download` |
| DBMS | **SQLite 3**（异构） |
| 表数 | 1 |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) §2.2 SQLite データ型 |
| 引用源 | `crates/rgs-asset-download/migrations/0001_resume_token_index.sql` |

> **重要异构说明**：
> - 本域是 RGS 仓库**唯一**用 SQLite 而非 PostgreSQL 的库
> - 无 PII 字段（FR-CDN-064）：`player_id` / `device_id` / `email` / `ip` / `mac` 全部 NOT in schema
> - 应用启动时 `SqliteResumeTokenStore::new` 自动跑 `CREATE TABLE IF NOT EXISTS` 建表（不依赖 migration runner）
> - 手工部署 / 调试可使用 `sqlite3 downloads.db < 0001_resume_token_index.sql`

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 12.1 | `resume_tokens` | レジュームトークン / Resume Tokens | 短期表 | 千万级（按 expires_at 清理）| 3 |

---

## 12.1 `resume_tokens` レジュームトークン

### 概要

断点续传 token 表（per RGS-SPEC-DTL-041 §6 + RGS-IMPL-PLAN-CDN-001 §3.2 M-2064.5）。`token_id` TEXT 主键。`asset_id` 引用 asset master（无 FK）。`payload` BLOB 存续传状态。`payload_size` INTEGER 存大小。**时间戳用 TEXT (ISO8601)**——SQLite 无原生 TIMESTAMPTZ，需应用层序列化/反序列化。

| 项目 | 内容 |
|---|---|
| 物理表名 | `resume_tokens` |
| 論理名 | レジュームトークン / Resume Tokens |
| 出典 | `crates/rgs-asset-download/migrations/0001_resume_token_index.sql:20-29` |
| 父文档 | RGS-SPEC-DTL-041 §6 / RGS-IMPL-PLAN-CDN-001 §3.2 / FR-CDN-064 |
| 関連表 | (无 FK) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `token_id` | トークン ID / Token Identifier | TEXT | UUID 文字列 | ✅ | — | — | ✅ | — | — | 主键（业务 UUID 字符串）|
| 2 | `asset_id` | アセット ID / Asset Identifier | TEXT | 1-128 字符 | — | — | — | ✅ | — | — | 资源 ID（跨服务引用，应用层校验）|
| 3 | `status` | 状態 / Status | TEXT | 1-32 字符 | — | — | — | ✅ | — | — | token 状态（active / completed / abandoned，应用层枚举）|
| 4 | `payload` | ペイロード（BLOB） / Payload (BLOB) | BLOB | — | — | — | — | ✅ | — | — | 续传状态数据（BLOB 序列化）|
| 5 | `payload_size` | ペイロードサイズ / Payload Size | INTEGER | >= 0 | — | — | — | ✅ | — | — | payload 字节数 |
| 6 | `created_at` | 作成日時（ISO8601 TEXT） / Created (ISO8601 TEXT) | TEXT | 25 字符 | — | — | — | ✅ | — | — | 创建时间（**SQLite 无 TIMESTAMPTZ，用 ISO8601 TEXT**）|
| 7 | `updated_at` | 更新日時（ISO8601 TEXT） / Updated (ISO8601 TEXT) | TEXT | 25 字符 | — | — | — | ✅ | — | — | 修改时间（ISO8601 TEXT）|
| 8 | `expires_at` | 有効期限（ISO8601 TEXT） / Expires (ISO8601 TEXT) | TEXT | 25 字符 | — | — | — | ✅ | — | — | 过期时间（ISO8601 TEXT）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `resume_tokens_pkey` | B-tree (PK) | `(token_id)` | 主键（自动）|
| 2 | `idx_resume_tokens_asset_id` | B-tree | `(asset_id)` | 同 asset 多次下载会话查询 |
| 3 | `idx_resume_tokens_expires_at` | B-tree | `(expires_at)` | 过期清理（cleanup_expired）|
| 4 | `idx_resume_tokens_updated_at` | B-tree | `(updated_at)` | LRU 驱逐（按 updated_at 升序驱逐最旧）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `resume_tokens_pkey` | `(token_id)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| (无) | — | — | — | — |

### 既知偏差

- **异构存储**：SQLite 3 vs PG 18，**无统一 migration runner**——应用启动时 `CREATE TABLE IF NOT EXISTS` 自动建表，与 PG 库的 sqlx migrate 路径分离
- **无 CHECK 约束**：`status` 枚举 / `payload_size >= 0` / 时间戳格式都未在 DB 层保证——SQLite CHECK 支持有限（部分版本），建议应用层校验
- **无 PII 字段（per FR-CDN-064）**：`player_id` / `device_id` / `email` / `ip` / `mac` 全部 NOT in schema——这是 PII 隔离设计，需保持
- **时间戳 TEXT 序列化**：应用层需保证 ISO8601 格式一致（建议用 `chrono` 或 `time` crate 的 `to_rfc3339()`）

### 回滚

```sql
DROP INDEX IF EXISTS idx_resume_tokens_updated_at;
DROP INDEX IF EXISTS idx_resume_tokens_expires_at;
DROP INDEX IF EXISTS idx_resume_tokens_asset_id;
DROP TABLE IF EXISTS resume_tokens;
```

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/rgs-asset-download/migrations/0001_resume_token_index.sql` |
| SPEC-DTL-041 | `docs/13-实施规范/RGS-SPEC-DTL-041_*.md` §6 |
| IMPL-PLAN-CDN-001 | `docs/12-未决事项/RGS-IMPL-PLAN-CDN-001_*.md` §3.2 M-2064.5 |
| FR-CDN-064 | (in SPEC-DTL-041 §6) |

> 任何实际 schema 与本文档不一致之处，以 `crates/rgs-asset-download/migrations/*.sql` 实际 SQL 为准。
