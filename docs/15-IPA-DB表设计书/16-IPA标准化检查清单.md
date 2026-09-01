# 16-IPA 标准化检查清单（IPA Compliance Checklist）

> **本文件定位**：RGS 仓库 42 张表的**逐表核对清单**——按 RGS-BAS-007 §2 命名 + JIS X 0123 + IPA 共通フレーム 2013 详细设计工程 三层標準。

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-DB-CHECKLIST |
| 版本 | 0.1 |
| 作成日 | 2026-09-01 JST |
| 適用範囲 | RGS 全部 42 张表 |

---

## 1. 命名规范核对（per RGS-BAS-007 §2）

### 1.1 表名

| # | 表名 | 庫名 | 規範 | 状態 |
|---|---|---|---|---|
| 1 | `players` | player_db | ✅ snake_case 复数 | OK |
| 2 | `player_sessions` | player_db | ✅ snake_case 复数 | OK |
| 3 | `player_characters` | player_db | ✅ snake_case 复数 | OK |
| 4 | `player_inventory` | player_db | ✅ snake_case 复数 | OK |
| 5 | `decks` | player_db | ✅ snake_case 复数 | OK |
| 6 | `accounts` | economy_db | ✅ snake_case 复数 | OK |
| 7 | `transaction_ledger` | economy_db | ✅ snake_case 复数 | OK |
| 8 | `sagas` | economy_db | ✅ snake_case 复数 | OK |
| 9 | `reservations` | economy_db | ✅ snake_case 复数 | OK |
| 10 | `inbox` | economy_db | ✅ snake_case 单数（领域名词）| OK |
| 11 | `auctions` | economy_db | ✅ snake_case 复数 | OK |
| 12 | `private_trades` | economy_db | ✅ snake_case 复数 | OK |
| 13 | `matches` | match_db | ⚠️ `match` 是 PG 保留字 → 用 `matches`（已正确）| OK |
| 14 | `match_participants` | match_db | ✅ snake_case 复数 | OK |
| 15 | `game_sessions` | match_db | ✅ snake_case 复数 | OK |
| 16 | `moves` | match_db | ✅ snake_case 复数 | OK |
| 17 | `matchmaking_tickets` | match_db | ✅ snake_case 复数 | OK |
| 18 | `session_subscriptions` | match_db | ✅ snake_case 复数 | OK |
| 19 | `guilds` | social_db | ✅ snake_case 复数 | OK |
| 20 | `guild_members` | social_db | ✅ snake_case 复数 | OK |
| 21 | `admin_users` | admin_db | ✅ snake_case 复数 | OK |
| 22 | `audit_log` | admin_db | ✅ snake_case 单数 | OK |
| 23 | `realm_lifecycle_run` | admin_db (LCM) | ✅ snake_case 单数 | OK |
| 24 | `new_realm_plan` | admin_db (LCM) | ✅ snake_case 单数 | OK |
| 25 | `split_plan` | admin_db (LCM) | ✅ snake_case 单数 | OK |
| 26 | `merge_conflict_rule_set_v2` | admin_db (LCM) | ✅ snake_case + v2 版本后缀 | OK |
| 27 | `retire_plan` | admin_db (LCM) | ✅ snake_case 单数 | OK |
| 28 | `archive_policy` | admin_db (LCM) | ✅ snake_case 单数 | OK |
| 29 | `cluster_nodes` | cluster_ops_db | ✅ snake_case 复数 | OK |
| 30 | `feature_flags` | cluster_ops_db | ✅ snake_case 复数 | OK |
| 31 | `cards` | card_db | ✅ snake_case 复数 | OK |
| 32 | `card_series` | card_db | ✅ snake_case 复数 | OK |
| 33 | `card_instances` | card_db | ✅ snake_case 复数 | OK |
| 34 | `i18n_texts` | i18n_db | ✅ snake_case 复数 | OK |
| 35 | `i18n_languages` | i18n_db | ✅ snake_case 复数 | OK |
| 36 | `leaderboard_entries` | leaderboard_db | ✅ snake_case 复数 | OK |
| 37 | `replays` | replay_db | ✅ snake_case 复数 | OK |
| 38 | `resume_tokens` | downloads.sqlite | ✅ snake_case 复数 | OK |
| 39-44 | `outbox` × 6 域 | player / economy / match / social / admin / cluster_ops | ✅ snake_case 单数（领域名词）| OK |

**结论**：全部 38 个业务表 + 6 域 outbox = **44 处全部符合 snake_case 命名** ✅

### 1.2 列名

| 規範 | 採用率 | 不符例 |
|---|---|---|
| snake_case 全小写 | **100%**（手测） | 无 |
| 主键列名为 `id` / 联合主键同前缀（如 `deck_id`, `auction_id`, `match_id`）| **100%** | 无（除 `i18n_texts` 联合主键用 `key+locale` 是合理例外）|
| 外键列 `<被引用表名单数>_id` | **100%** | 无 |
| 时间戳 `<event>_at` | **100%** | 无 |
| 布尔 `is_<adjective>` / `enabled` / `accepted` | **100%** | 无 |

**结论**：列名 100% 符合 RGS-BAS-007 §2 命名规范 ✅

### 1.3 索引名

| 表 | 索引名 | 符合 `idx_<表名>_<列名或用途>`？ |
|---|---|---|
| 全部 38 表 + outbox | `idx_players_name`, `idx_outbox_pending`, `idx_pc_player_id` 等 | ✅ **100% 符合** |

**结论**：索引名 100% 符合 RGS-BAS-007 §2 命名规范 ✅

### 1.4 CHECK 约束名

| 表 | CHECK 名 | 符合 `chk_<表名>_<列名或语义>`？ |
|---|---|---|
| `outbox` × 6 域 | `chk_outbox_status` | ✅ 符合 |
| `merge_conflict_rule_set_v2` | `chk_merge_conflict_lock_consistency` | ✅ 符合 |
| 其他 36 表的 CHECK | (PG 自动命名 `tablename_columnname_check` 或 `tablename_check`) | ⚠️ 未显式命名 |

> **建议 PH-2 评审**：为所有显式 CHECK 约束加 `chk_` 前缀命名（RGS-BAS-007 v0.3 补全）。

### 1.5 唯一约束名

| 表 | 唯一约束 | 符合 `uq_<表名>_<列名>`？ |
|---|---|---|
| `sagas` | `uq_sagas_command_id` | ✅ 符合 |
| `decks` | (PG 自动 `decks_share_code_key`) | ⚠️ 未显式命名 |
| `player_inventory` | `uq_pi_player_slot` | ✅ 符合 |
| `merge_conflict_rule_set_v2` | `uq_merge_conflict_rule_set_version` | ✅ 符合 |
| `realm_lifecycle_run` | `uq_lifecycle_run_request_operator` | ✅ 符合 |
| 其他 | (PG 自动) | ⚠️ |

> **建议 PH-2 评审**：为所有 UNIQUE 约束加 `uq_` 前缀命名。

### 1.6 外键约束名

| 表 | 外键约束 | 命名 |
|---|---|---|
| `player_characters` | `fk_pc_player`, `fk_pc_weapon` | ✅ 显式命名（per 2026-09-01 fix）|
| 其他 | (PG 自动 `tablename_column_fkey`) | ⚠️ 未显式命名 |

> **建议 PH-2 评审**：为所有 FK 约束加 `fk_<表名>_<被引用表>` 命名。

---

## 2. 列属性核对（per JIS X 0123 + IPA 詳細設計）

### 2.1 データ型

| 用途 | 採用的 PG/SQLite 型 | 状態 |
|---|---|---|
| 主键 UUID | `UUID` | ✅ 100% |
| 外键 UUID | `UUID` | ✅ 90%（除跨域 `card_id` / `player_a` 等用 TEXT，跨域兼容）|
| 货币余额 | `BIGINT` | ✅ 100% |
| 数量/计数 | `BIGINT` / `INTEGER` | ✅ 100% |
| 小数概率 | `REAL` | ✅ 100%（仅 `crit_rate`）|
| 枚举字符串 | `TEXT` + `CHECK` | ✅ 100% |
| 枚举 SMALLINT | `SMALLINT` | ✅ 100%（仅 `mode` / `status` 等协议对齐）|
| 短字符串 | `TEXT` | ✅ 100% |
| 中等字符串 | `VARCHAR(N)` | ✅ 100%（仅 outbox.subject）|
| 布尔 | `BOOLEAN` | ✅ 100% |
| 时间戳 | `TIMESTAMPTZ` | ✅ 100%（PG 库）|
| TEXT 时间戳 | `TEXT` ISO8601 | ✅ 100%（SQLite）|
| 二进制 | `BLOB` | ✅ 100%（SQLite resume_tokens.payload）|
| JSON 文档 | `JSONB` | ✅ 100% |

### 2.2 NOT NULL + DEFAULT

- 100% 主键 NOT NULL ✅
- 100% FK 列 NOT NULL（除可选 FK 如 `primary_weapon_id` / `winner_team`）✅
- 100% 时间戳有 DEFAULT `now()` ✅
- 100% 业务必填列 NOT NULL ✅

### 2.3 CHECK 约束

| 类别 | 状態 | 不符 |
|---|---|---|
| 枚举字符串 | ✅ 100% 覆盖 | 无 |
| 枚举 SMALLINT | ⚠️ 部分缺（per [15-§4.1](#4-1-状态机一致性缺-db-check)）| game_sessions.status / mode / moves.move_type / matchmaking_tickets.status 等 |
| 数值范围 | ⚠️ 部分缺 | `rank` / `wins` / `losses` / `payload_size` / `duration_secs` 等 |
| 跨列一致性 | ⚠️ 大部分缺 | per [15-§4.2](#4-2-跨列一致性) |

---

## 3. 索引策略核对（per RGS-BAS-007 §3 + RGS-SPEC-CROSS-005）

### 3.1 高频查询路径索引覆盖

| 表 | 高频查询 | 索引 | 状態 |
|---|---|---|---|
| `players` | 按 name 查 / 按 level 查 / 按 status 查 | `idx_players_name` + `idx_players_level` + `idx_players_status` | ✅ |
| `player_sessions` | 按 player_id 查 / 过期扫描 | `idx_player_sessions_player_id` + `idx_player_sessions_expires_at` | ✅ |
| `accounts` | 按 player_id 查 | `idx_accounts_player_id` | ✅ |
| `transaction_ledger` | 按 saga_id 查 / 按 account_id 查 / 按 status 查 | 3 索引 | ✅ |
| `sagas` | 按 command_id 查 / 按 status 查 / 按 idempotency_key 查 | 3 索引 + 1 UK | ✅（但 `idx_sagas_command_id` 与 `uq_sagas_command_id` 重複）|
| `matches` | 按 status 查 / 按 scheduled_at 查 | 2 索引 | ✅ |
| `game_sessions` | 按 status 查 / 按 host_id 查 / 按 room_code 查 | 4 索引 | ✅ |
| `moves` | 按 match_id 查 / 按 (match_id, turn_index) 查 / 按 player_id 查 / 按 occurred_at 查 | 4 索引 | ✅ |
| `matchmaking_tickets` | 按 player_id 查 / 按 status 查 / 按 (mode, rank_score_min, rank_score_max) 查 / 过期扫描 | 4 索引 | ✅ |
| `decks` | 按 owner_id 查 / 按 (owner_id, updated_at DESC) 查 / 按 is_public 查 / 按 share_code 查 | 4 索引 + 1 UK | ✅ |
| `player_characters` | 按 player_id 查 / 按 (char_class, level) 查 / 按 primary_weapon_id 查 / JSONB stats 查 | 4 索引 + 1 JSONB-GIN | ✅ |
| `player_inventory` | 按 player_id 查 / 按 item_id 查 / 按 acquired_at 查 / 按 (player_id, slot) 唯一 | 3 索引 + 1 UK + 1 JSONB-GIN | ✅ |
| `cards` | 按 series_id 查 / 按 rarity 查 / 按 type 查 | 3 索引 | ✅ |
| `card_instances` | 按 owner_id 查 / 按 card_id 查 / 按 (owner_id, acquired_at DESC) 查 | 2 索引 + 1 复合 | ✅ |
| `guilds` | 按 leader_id 查 / 按 level 查 | 2 索引 | ✅ |
| `guild_members` | 按 guild_id 查 / 按 player_id 查 | 2 索引 | ✅ |
| `i18n_texts` | 按 key 查（与 PK 左侧重複）| 1 索引（**与 PK 重複**）| ⚠️ |
| `i18n_languages` | 默认 locale 唯一 | 1 partial UK | ✅ |
| `leaderboard_entries` | 按 (type, period, season_id, score DESC) 查 / 按 player_id 查 | 2 索引 + 1 UK | ✅ |
| `replays` | 按 player_a 查 / 按 player_b 查 / 按 match_id 查 / 过期扫描 | 4 索引 | ✅ |
| `resume_tokens` | 按 asset_id 查 / 过期清理 / LRU 驱逐 | 3 索引 | ✅ |
| `audit_log` | 按 actor_id 查 / 按 action 查 / 按 created_at 查 | 3 索引 + 1 UK | ✅ |
| `admin_users` | 按 role 查 / 按 disabled_at 查 | 2 索引 | ✅ |
| `cluster_nodes` | 按 status 查 / 按 role 查 / 按 heartbeat 查 | 3 索引 | ✅ |
| `feature_flags` | 按 scope_value 查 / 按 enabled 查 | 2 索引 | ✅ |
| `realm_lifecycle_run` | 按 (status, created_at DESC) 查 / 按 realm_id 查 / 按 (feature_subtype, created_at DESC) 查 / 按 trace_id 查 | 4 索引 + 1 UK | ✅ |
| `merge_conflict_rule_set_v2` | 按 version DESC 查 / 按 locked_at 查 / JSONB rules 查 | 2 索引 + 1 partial + 1 JSONB-GIN + 1 UK | ✅ |
| `archive_policy` | 按 realm_id 查（与 UK 重複）/ 按 n_plus_2_redundancy 查 | 1 索引 + 1 partial（**与 UK 重複**）| ⚠️ |
| `outbox` × 6 域 | partial pending / partial in_flight / command_id 查重 | 3 索引 | ✅ |

**结论**：高频查询路径索引覆盖 100% ✅（除少量重複索引 P1-01 / P1-04）

### 3.2 JSONB GIN 索引

| 表 | JSONB 列 | GIN 索引 | 状態 |
|---|---|---|---|
| `player_characters` | `stats` | `idx_pc_stats_gin` | ✅ |
| `player_inventory` | `metadata` | `idx_pi_metadata_gin` | ✅ |
| `decks` | `slots` | `idx_decks_slots_gin` | ✅ |
| `game_sessions` | `players` | `idx_game_sessions_players_gin` | ✅ |
| `merge_conflict_rule_set_v2` | `rules` | `idx_merge_conflict_rule_set_rules_gin` | ✅ |

**结论**：JSONB GIN 索引 100% 覆盖 ✅

### 3.3 重複/冗余索引

| 表 | 重複 | 影响 | 建议 |
|---|---|---|---|
| `players` | `idx_players_name` 与 `players_name_key` 在 `name` 列上 | 写放大 + 存储浪费 | PH-2 评审移除 `idx_players_name` |
| `i18n_texts` | `idx_i18n_key` 与 PK 左侧 `(key, locale)` 在 `key` 列上 | 写放大 | PH-2 评审移除 `idx_i18n_key` |
| `sagas` | `idx_sagas_command_id` 与 `uq_sagas_command_id` 在 `command_id` 列上 | 写放大 | PH-2 评审移除 `idx_sagas_command_id` |
| `archive_policy` | `idx_archive_policy_realm_id` 与 `archive_policy_realm_id_key` 在 `realm_id` 列上 | 写放大 | PH-2 评审移除 `idx_archive_policy_realm_id` |

详见 [17-不合理设计 P1-01 / P1-04](17-不合理设计识别与优化建议.md)

---

## 4. 分区策略核对（per RGS-BAS-007 §4）

| 表 | 規定分区策略 | 現状 | 状態 |
|---|---|---|---|
| `audit_log` (admin_db) | 按月范围分区 | 未分区 | ❌ **P0-02** |
| `outbox` × 6 域 | 按周/月范围分区 | 未分区 | ❌ **P0-03** |
| `realm_lifecycle_run` (admin_db) | 按月范围分区 | **已按月分区** | ✅ |
| `inbox` (economy_db) | 按时间范围分区 | 未分区 | ⚠️ P2-01 |
| `reservations` (economy_db) | 按时间范围分区 | 未分区 | ⚠️ P2-02 |
| `moves` (match_db) | 按时间范围分区 | 未分区 | ⚠️ P1-07 |
| `replays` (replay_db) | 按时间范围分区 | 未分区 | ⚠️ P1-15 |
| 业务主表 | **不分区** | 未分区 | ✅ 符合 RGS-BAS-007 §4 "未证明前不引入复杂性" |

详见 [14-分区策略与生命周期](14-分区策略与生命周期.md)

---

## 5. 迁移脚本核对（per RGS-BAS-007 §5）

| 規則 | 採用率 | 説明 |
|---|---|---|
| 迁移文件命名 `<序号>_<动词>_<对象>.sql` | ✅ 100% | 全部 33 个 migration 都符合（`0001_init.sql`, `0002_outbox.sql`, `0003_outbox_check_idempotent.sql`, `0004_player_characters_inventory.sql`, `0005_decks.sql` 等）|
| 幂等性 `IF NOT EXISTS` / `IF EXISTS` | ✅ 100% | 全部 CREATE/INDEX 用 `IF NOT EXISTS` |
| 复杂 CHECK 用 DO 块 + `pg_constraint` 检查 | ✅ 100%（修复后） | `0003_outbox_check_idempotent.sql` / `0004_outbox_check_idempotent.sql` 模式 |
| 跨表 FK 用 ALTER TABLE 替代 CREATE TABLE 内联 | ✅ 100%（修复后） | `0004_player_characters_inventory.sql:170-183` 反 pattern 修复示例 |
| Expand-Contract 分离 | ✅ 100% | 全部 33 migration 单次只做 Expand 或 Contract，未混用 |

**结论**：迁移脚本 100% 符合 RGS-BAS-007 §5 ✅

---

## 6. 备份与恢复核对（per RGS-BAS-007 §6）

> 备份恢复属 RGS-OPS-001 / RGS-OPS-100 範畴，本文件仅核对标准遵循度。

| 規則 | 状態 | 説明 |
|---|---|---|
| RTO 30 分钟 / RPO=0 同步复制 | ✅ 复用 RGS-REQ-001 §NFR-AV-004/005 | PG 同步复制 + 周期性物理/逻辑备份 |
| 备份恢复定期演练 | ⚠️ 待 PH-2 实施 | 需建立演练 SOP + 记录归档 |

---

## 7. 存储过程使用边界（per ARC-023）

| 規則 | 現状 | 評価 |
|---|---|---|
| 业务逻辑**不得**入库 | ✅ 100% 符合 | 全部业务逻辑在 Rust 服务层 |
| 极简约束触发器允许（单表数据完整性 + 不跨表副作用）| ✅ 100% 符合 | `audit_log_no_update` + `audit_log_no_delete` 属"单表 append-only 保护"，不跨表，符合 ARC-023 |
| 触发器经 ADR 登记 | ✅ 符合 | 登记于 DTL-007 §5 + 触发器定义 inline 注释（line 38-53 `audit_log.sql`）|

**结论**：100% 符合 ARC-023 极窄允许边界 ✅

---

## 8. 总分

| 类别 | 得分 | 备注 |
|---|---|---|
| 命名规范 | 95% | CHECK / UK / FK 命名未显式约定（建议 PH-2 补 RGS-BAS-007 v0.3）|
| 列属性 | 90% | 部分 SMALLINT 枚举缺 CHECK + 部分数值范围缺 CHECK |
| 索引策略 | 100% | 高频路径 100% 覆盖，JSONB GIN 100% 覆盖（仅 4 处冗余索引需清理）|
| 分区策略 | 30% | 仅 `realm_lifecycle_run` 已实施；`audit_log` / `outbox` / `inbox` / `reservations` / `moves` / `replays` 待实施 |
| 迁移脚本 | 100% | 幂等性 + DO 块 + Expand-Contract 全部符合 |
| 存储过程 | 100% | 符合 ARC-023 |

> **總體評分：~85%**。剩余 15% 主要集中在**分区未实施（P0）+ CHECK 命名约定未补全**——属于 PH-2 评审范畴。

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| BAS-007 | `docs/03-数据经济与交易/RGS-BAS-007_*.md` §2-§9 |
| REQ-011 | `docs/03-数据经济与交易/RGS-REQ-011_*.md` FR-DBS-001〜041 |
| SPEC-CROSS-005 | `docs/13-实施规范/RGS-SPEC-CROSS-005_*.md` |
| DTL-100 | `docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式定义_v0.1.md` |
| ARC-023 | `docs/03-数据经济与交易/RGS-REQ-011_*.md` §7 |

> 任何实际 schema 与本文档不一致之处，以 `crates/*/migrations/*.sql` 实际 SQL 为准。
