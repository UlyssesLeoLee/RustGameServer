# 05-Social 域（social_db）

> **本文件定位**：Social 域 3 张表的詳細表設計書。覆盖 2 业务表（guilds / guild_members）+ 1 公共 outbox。

| 项目 | 内容 |
|---|---|
| 物理库 | `social_db` |
| 担当 crate | `social-service` |
| DBMS | PostgreSQL 18 |
| 表数 | 3（含 outbox） |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/social-service/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 5.1 | `guilds` | ギルド（公会） / Guilds | 永久事实表 | 十万级 | 2 |
| 5.2 | `guild_members` | ギルドメンバー / Guild Members | 永久事实表 | 百万级 | 2 |
| 5.3 | `outbox` | アウトボックス（公共） / Outbox | 时序短期表 | 千万级/日 | 3 |

---

## 5.1 `guilds` ギルド（公会）

### 概要

公会主表（per RGS-DTL-026 §3）。`name` UNIQUE 全局唯一。`leader_id` 跨域弱引用（player_db.players.id）。`member_count` 冗余字段（应用层维护一致性）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `guilds` |
| 論理名 | ギルド（公会） / Guilds |
| 出典 | `crates/social-service/migrations/0001_init.sql:5-15` |
| 父文档 | RGS-DTL-026 §3 |
| 関連表 | `guild_members` (1:N CASCADE), `players` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `name` | ギルド名 / Guild Name | TEXT | 1-64 字符 | — | — | ✅ | ✅ | — | — | 全局唯一公会名 |
| 3 | `description` | ギルド説明 / Guild Description | TEXT | — | — | — | — | ✅ | `''` | — | 公会描述 |
| 4 | `leader_id` | リーダー ID / Leader Identifier | UUID | 128-bit | — | — (跨域弱引用) | — | ✅ | — | — | 公会会长 player_id |
| 5 | `level` | ギルドレベル / Guild Level | INTEGER | 1-999（应用层校验） | — | — | — | ✅ | 1 | — | 公会等级 |
| 6 | `member_count` | メンバー数 / Member Count | INTEGER | >= 0 | — | — | — | ✅ | 1 | — | 冗余字段（**应拆表** — 见 [17-P1-08](17-不合理设计识别与优化建议.md)）|
| 7 | `experience` | ギルド経験値 / Guild Experience | BIGINT | >= 0 | — | — | — | ✅ | 0 | — | 公会经验值 |
| 8 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 9 | `updated_at` | 更新日時 / Update Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 修改时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `guilds_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `guilds_name_key` | B-tree (UNIQUE) | `(name)` | 公会名唯一 |
| 3 | `idx_guilds_leader_id` | B-tree | `(leader_id)` | 按会长筛选 |
| 4 | `idx_guilds_level` | B-tree | `(level)` | 按等级筛选/排行 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `guilds_pkey` | `(id)` |
| UNIQUE | `guilds_name_key` | `(name)` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N | `guild_members` | `guild_members.guild_id → guilds.id` | CASCADE | ✅ |
| N:1 (跨域) | `players` (player_db) | `guilds.leader_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- `member_count` 冗余字段（应有 `guild_member_count` 物化视图 或 通过 `SELECT COUNT(*)` 实时计算）——见 [17-P1-08](17-不合理设计识别与优化建议.md)
- 缺 `member_count <= 50` 上限 CHECK（per Q5 决策：公会容量 50 上限 vs 代码 64 现状的 social Lead 待确认）——见 [17-P1-09](17-不合理设计识别与优化建议.md)
- 缺 `description` 长度上限

---

## 5.2 `guild_members` ギルドメンバー

### 概要

公会成员表（per RGS-DTL-026 §3.2）。`UNIQUE (guild_id, player_id)` 防重入。3 角色（leader / officer / member）。`player_id` 跨域弱引用。

| 项目 | 内容 |
|---|---|
| 物理表名 | `guild_members` |
| 論理名 | ギルドメンバー / Guild Members |
| 出典 | `crates/social-service/migrations/0001_init.sql:21-29` |
| 父文档 | RGS-DTL-026 §3.2 |
| 関連表 | `guilds` (N:1 CASCADE), `players` (跨域弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `guild_id` | ギルド ID / Guild Identifier | UUID | 128-bit | — | `guilds(id) ON DELETE CASCADE` | `(guild_id, player_id)` | ✅ | — | — | 所属公会 |
| 3 | `player_id` | プレイヤー ID / Player Identifier | UUID | 128-bit | — | — (跨域弱引用) | `(guild_id, player_id)` | ✅ | — | — | 成员 player_id |
| 4 | `role` | ギルド内役割 / Guild Role | TEXT | — | — | — | — | ✅ | `'member'` | `role IN ('leader', 'officer', 'member')` | 3 选 1 角色 |
| 5 | `contribution` | 貢献度 / Contribution | BIGINT | >= 0 | — | — | — | ✅ | 0 | — | 贡献值 |
| 6 | `joined_at` | 参加日時 / Joined At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 加入时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `guild_members_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `guild_members_guild_id_player_id_key` | B-tree (UNIQUE) | `(guild_id, player_id)` | 一玩家一公会唯一 |
| 3 | `idx_members_guild_id` | B-tree | `(guild_id)` | FK 索引（CASCADE 性能）|
| 4 | `idx_members_player_id` | B-tree | `(player_id)` | 查玩家所属公会 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `guild_members_pkey` | `(id)` |
| UNIQUE | `guild_members_guild_id_player_id_key` | `(guild_id, player_id)` |
| FOREIGN KEY | (隐式) | `(guild_id) REFERENCES guilds(id) ON DELETE CASCADE` |
| CHECK | (隐式) `role_check` | `role IN ('leader', 'officer', 'member')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 | `guilds` | `guild_members.guild_id → guilds.id` | CASCADE | ✅ |
| N:1 (跨域) | `players` (player_db) | `guild_members.player_id = players.id` | app-layer | ❌ 弱引用 |

### 既知偏差

- 缺 `role='leader'` 唯一性约束（一个公会只能有一个 leader）——建议 PH-2 加 `CREATE UNIQUE INDEX idx_members_leader ON guild_members (guild_id) WHERE role = 'leader'`
- 缺 `player_id` 跨域弱引用的应用层校验 SOP（与 DTL-018 §3.1 配合）

---

## 5.3 `outbox` アウトボックス（公共 / Social 域）

> 完整模板見 [13-Outbox 跨域模板](13-Outbox跨域模板.md)。

- **位置**：`social_db.outbox`
- **结构**：与模板 1:1 一致
- **特有应用层**：
  - `social.guild.created` / `disbanded`
  - `social.guild.member.joined` / `left` / `promoted` / `demoted`
  - `social.guild.experience.donated`
- **Known Drift**：0002 写入 outbox 但 CHECK 约束在已部署环境失效；0003 修复

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 全部 SQL | `crates/social-service/migrations/0001_init.sql` + `0002_outbox.sql` + `0003_outbox_check_idempotent.sql` |
| DTL-026 | `docs/01-核心架构与设计模式/RGS-DTL-026_详细设计书.md` §3 / §3.2 |
| DTL-039 | `docs/01-核心架构与设计模式/RGS-DTL-039_Social域_详细设计书.md` |
| DTL-018 | `docs/01-核心架构与设计模式/RGS-DTL-018_详细设计书.md` §3.1 |

> 任何实际 schema 与本文档不一致之处，以 `crates/social-service/migrations/*.sql` 实际 SQL 为准。
