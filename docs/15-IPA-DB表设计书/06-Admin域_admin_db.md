# 06-Admin 域（admin_db） + LCM 子模块

> **本文件定位**：Admin 域 9 张表的詳細表設計書。覆盖 2 admin 业务表（admin_users / audit_log）+ 1 公共 outbox + 6 LCM 表（realm_lifecycle_run + 5 子表，per FR-LCM-001 全在 admin_db）。

| 项目 | 内容 |
|---|---|
| 物理库 | `admin_db` |
| 担当 crate | `admin-service` + `cluster-ops` (LCM 子模块) |
| DBMS | PostgreSQL 18 |
| 表数 | 9（含 outbox） |
| 引用規格 | [01-IPA 命名与列属性标准](01-IPA命名与列属性标准.md) |
| 引用源 | `crates/admin-service/migrations/0001_init.sql` + `0002_audit.sql` + `0003_outbox.sql` + `0004_outbox_check_idempotent.sql` + `crates/cluster-ops/migrations/0020_lcm_tables.sql` |

---

## 表一覧

| # | 物理表名 | 論理名 | 種別 | 估算規模 | 关键索引数 |
|---|---|---|---|---|---|
| 6.1 | `admin_users` | 管理者ユーザー / Admin Users | 永久事实表 | 百级 | 2 |
| 6.2 | `audit_log` | 監査ログ（ハッシュチェーン） / Audit Log (Hash-Chained) | 高频追加表 | 亿级（应分区 — 见 P0-02）| 3 + 1 UK + 2 触发器 |
| 6.3 | `outbox` | アウトボックス（公共） / Outbox | 时序短期表 | 千万级/日 | 3 |
| 6.4 | `realm_lifecycle_run` | レルムライフサイクル実行（LCM 主表） / Realm Lifecycle Run (LCM Master) | 永久事实表（已按月分区）| 十万级 | 4（partial×1）|
| 6.5 | `new_realm_plan` | 新規レルム計画 / New Realm Plan | 永久事实表 | 百级 | 3 |
| 6.6 | `split_plan` | 分裂計画 / Split Plan | 永久事实表 | 十级 | 3 |
| 6.7 | `merge_conflict_rule_set_v2` | マージ競合ルールセット V2 / Merge Conflict Rule Set V2 | 永久事实表 | 十级 | 3 + 1 JSONB-GIN + 1 UK |
| 6.8 | `retire_plan` | 退場計画 / Retire Plan | 永久事实表 | 十级 | 3 |
| 6.9 | `archive_policy` | アーカイブポリシー / Archive Policy | 永久事实表 | 百级 | 1 + 1 partial |

---

## 6.1 `admin_users` 管理者ユーザー

### 概要

管理员账号表（per RGS-DTL-019 §3 + ARC-051 COC/CEM + SEC-100 §7）。4 角色（super_admin / domain_admin / auditor / support）+ `domain_scope` 限定管理范围。`password_hash` 应使用 bcrypt/argon2（应用层保证）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `admin_users` |
| 論理名 | 管理者ユーザー / Admin Users |
| 出典 | `crates/admin-service/migrations/0001_init.sql:8-17` |
| 父文档 | RGS-DTL-019 §3 / RGS-ARC-051 COC/CEM / RGS-SEC-100 §7 |
| 関連表 | `audit_log` (1:N 弱引用) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `username` | ユーザー名 / Username | TEXT | 1-64 字符 | — | — | ✅ | ✅ | — | — | 全局唯一管理员名 |
| 3 | `password_hash` | パスワードハッシュ / Password Hash | TEXT | 60-256 字符 (bcrypt/argon2) | — | — | — | ✅ | — | — | 密码 hash（敏感字段）|
| 4 | `role` | 役割（RBAC） / Role (RBAC) | TEXT | — | — | — | — | ✅ | — | `role IN ('super_admin', 'domain_admin', 'auditor', 'support')` | 4 选 1 RBAC 角色 |
| 5 | `domain_scope` | ドメインスコープ（RBAC 適用範囲） / Domain Scope (RBAC) | TEXT | 0-128 字符（应用层校验 JSON） | — | — | — | ❌ | NULL | — | 限定管理域（多域用 JSON 数组）|
| 6 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |
| 7 | `last_login_at` | 最終ログイン日時 / Last Login Timestamp | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 最后登录时间 |
| 8 | `disabled_at` | 無効化日時 / Disabled Timestamp | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 禁用时间（NULL = 启用）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `admin_users_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `admin_users_username_key` | B-tree (UNIQUE) | `(username)` | 用户名唯一 |
| 3 | `idx_admin_users_role` | B-tree | `(role)` | 按角色筛选（超管列表/审计员列表）|
| 4 | `idx_admin_users_disabled_at` | partial B-tree | `(disabled_at) WHERE disabled_at IS NOT NULL` | 查已禁用账号 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `admin_users_pkey` | `(id)` |
| UNIQUE | `admin_users_username_key` | `(username)` |
| CHECK | (隐式) `role_check` | `role IN ('super_admin', 'domain_admin', 'auditor', 'support')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N (同库) | `audit_log` | `audit_log.actor_id = admin_users.id` | app-layer | ❌ 弱引用（**应物化**）|

### 既知偏差

- `password_hash` 缺长度 CHECK（建议 PH-2 加 `CHECK (length(password_hash) BETWEEN 60 AND 256)`）
- `domain_scope` TEXT 但承载 JSON——建议 PH-2 改 `JSONB` 以支持 GIN 查询（按需）
- `actor_id` 跨表弱引用（应物化）

---

## 6.2 `audit_log` 監査ログ（ハッシュチェーン）

### 概要

审计日志表（per RGS-SEC-100 §7 hash 链防篡改）。`prev_hash` + `hash` 链式结构，**禁 UPDATE/DELETE 触发器**保证 append-only。`hash` UNIQUE 防止重放。**未按月分区**（per RGS-BAS-007 §4 应分区）——见 [17-P0-02](17-不合理设计识别与优化建议.md)。

| 项目 | 内容 |
|---|---|
| 物理表名 | `audit_log` |
| 論理名 | 監査ログ（ハッシュチェーン） / Audit Log (Hash-Chained) |
| 出典 | `crates/admin-service/migrations/0001_init.sql:23-32` |
| 父文档 | RGS-SEC-100 §7 hash 链防篡改 |
| 関連表 | `admin_users` (N:1 弱引用 actor_id) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `actor_id` | 操作者 ID / Actor Identifier | UUID | 128-bit | — | — (同库弱引用 → admin_users, 应物化) | — | ✅ | — | — | 操作者 admin_users.id |
| 3 | `action` | 操作 / Action | TEXT | 1-128 字符 | — | — | — | ✅ | — | — | 操作类型（如 `gm_command_issued`）|
| 4 | `target` | 操作対象 / Target | TEXT | 1-512 字符 | — | — | — | ✅ | — | — | 操作目标（资源 ID/路径）|
| 5 | `payload` | ペイロード / Payload | TEXT | JSON 文字列 | — | — | — | ✅ | `'{}'` | — | 操作详细参数（JSON 字符串）|
| 6 | `prev_hash` | 前回ハッシュ値 / Previous Hash | TEXT | 64 字符 (SHA-256 hex) | — | — | — | ✅ | — | — | 前一条 hash（链式）|
| 7 | `hash` | ハッシュ値 / Hash | TEXT | 64 字符 (SHA-256 hex) | — | — | ✅ | ✅ | — | — | 当前条 hash（含 prev_hash）|
| 8 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 操作时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `audit_log_pkey` | B-tree (PK) | `(id)` | 主键（自动） |
| 2 | `audit_log_hash_key` | B-tree (UNIQUE) | `(hash)` | hash 唯一（防重放）|
| 3 | `idx_audit_actor_id` | B-tree | `(actor_id)` | 查某操作者历史 |
| 4 | `idx_audit_action` | B-tree | `(action)` | 按操作类型筛选 |
| 5 | `idx_audit_created_at` | B-tree | `(created_at)` | 按时间排序/分区 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `audit_log_pkey` | `(id)` |
| UNIQUE | `audit_log_hash_key` | `(hash)` |
| TRIGGER | `audit_log_no_update` | BEFORE UPDATE → 抛异常 `audit_log is append-only` |
| TRIGGER | `audit_log_no_delete` | BEFORE DELETE → 抛异常 `audit_log is append-only` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| N:1 (同库) | `admin_users` | `audit_log.actor_id = admin_users.id` | app-layer | ❌ 弱引用（**应物化**）|

### 既知偏差

- **未按月分区**（per RGS-BAS-007 §4 应分区，3 年 36 个分区滚动保留）——见 [17-P0-02](17-不合理设计识别与优化建议.md)
- `payload` 用 TEXT 存 JSON——建议 PH-2 改 `JSONB` 以支持 GIN 索引（按需）
- `actor_id` 同库弱引用（应物化）
- `hash` 算法 SHA-256 hex（64 字符）应用层保证——DB 层无长度 CHECK（建议 `CHECK (length(hash) = 64)`）

---

## 6.3 `outbox` アウトボックス（公共 / Admin 域）

> 完整模板見 [13-Outbox 跨域模板](13-Outbox跨域模板.md)。

- **位置**：`admin_db.outbox`
- **结构**：与模板 1:1 一致
- **特有应用层**：
  - `admin.gm_command.issued` / `completed` / `failed`
  - `admin.audit_log.queried` (审计日志被查询的事件)
  - `admin.feature_flag.updated`
  - `admin.coc.policy_changed`
  - `admin.realm_lifecycle.<subtype>.started` / `completed` / `failed`（LCM 事件转发）

---

## 6.4 `realm_lifecycle_run` レルムライフサイクル実行（LCM 主表）

### 概要

服务器全生命周期管理主表（per FR-LCM-001 + M-2068.1 + DTL-042 §3 + SPEC-DTL-042 §3 第 5 条）。**已按月分区**（per RGS-BAS-007 §4，3 年 36 个分区滚动保留）。7 种 feature_subtype 覆盖 new_realm / scale / split / merge / merge_rollback / retire / archive。

| 项目 | 内容 |
|---|---|
| 物理表名 | `realm_lifecycle_run` |
| 論理名 | レルムライフサイクル実行 / Realm Lifecycle Run |
| 出典 | `crates/cluster-ops/migrations/0020_lcm_tables.sql:31-47` |
| 父文档 | RGS-DTL-042 §3 / RGS-SPEC-DTL-042 §3 / RGS-FR-LCM-001~081 |
| 関連表 | `new_realm_plan` / `split_plan` / `retire_plan` (1:N RESTRICT) |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `feature_subtype` | 機能サブタイプ / Feature Subtype | TEXT | — | — | — | — | ✅ | — | `feature_subtype IN ('new_realm', 'scale', 'split', 'merge', 'merge_rollback', 'retire', 'archive')` | 7 种 LCM 操作 |
| 3 | `realm_id` | レルム ID / Realm Identifier | UUID | 128-bit | — | — | — | ✅ | — | — | 目标 realm |
| 4 | `operator_id` | 実行者 ID / Operator Identifier | UUID | 128-bit | — | — (跨域弱引用 → admin_users, 应物化) | — | ✅ | — | — | LCM 操作者 |
| 5 | `request_id` | リクエスト ID / Request Identifier | UUID | 128-bit | — | — | `(request_id, operator_id)` | ✅ | — | — | 幂等键（per FR-LCM-001）|
| 6 | `approval_ref` | 承認参照 / Approval Reference | TEXT | 1-512 字符 | — | — | — | ❌ | NULL | — | 审批引用（per 多人审批 SOP）|
| 7 | `status` | 実行状態 / Run Status | TEXT | — | — | — | — | ✅ | `'pending'` | `status IN ('pending', 'in_progress', 'completed', 'failed', 'rolled_back')` | 5 状态机 |
| 8 | `started_at` | 開始日時 / Started At | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 开始时间 |
| 9 | `completed_at` | 完了日時 / Completed At | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 完成时间 |
| 10 | `created_at` | 作成日時（分区键） / Creation Timestamp (Partition Key) | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | **分区键**（按月范围分区）|
| 11 | `trace_id` | トレース ID / Trace Identifier | TEXT | 1-128 字符 | — | — | — | ❌ | NULL | — | 分布式追踪 ID（cross-service）|

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `realm_lifecycle_run_pkey` | B-tree (PK) | `(id)` | 主键（自动，含分区）|
| 2 | `uq_lifecycle_run_request_operator` | B-tree (UNIQUE) | `(request_id, operator_id)` | 幂等性 |
| 3 | `idx_lifecycle_run_status_created_at` | B-tree | `(status, created_at DESC)` | 状态筛选 + 时间排序 |
| 4 | `idx_lifecycle_run_realm_id` | B-tree | `(realm_id)` | 查某 realm 的所有 LCM 记录 |
| 5 | `idx_lifecycle_run_feature_subtype` | B-tree | `(feature_subtype, created_at DESC)` | 按 LCM 类型 + 时间筛选 |
| 6 | `idx_lifecycle_run_trace_id` | partial B-tree | `(trace_id) WHERE trace_id IS NOT NULL` | 通过 trace_id 查找 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `realm_lifecycle_run_pkey` | `(id)`（含分区）|
| UNIQUE | `uq_lifecycle_run_request_operator` | `(request_id, operator_id)` |
| PARTITION | `realm_lifecycle_run_yYYYYMM` | `PARTITION BY RANGE (created_at)`，月分区 |
| CHECK | (隐式) `feature_subtype_check` | `feature_subtype IN ('new_realm', 'scale', 'split', 'merge', 'merge_rollback', 'retire', 'archive')` |
| CHECK | (隐式) `status_check` | `status IN ('pending', 'in_progress', 'completed', 'failed', 'rolled_back')` |

### 関連表

| 方向 | 关联表 | 关联列 | 関係 | 物理 FK? |
|---|---|---|---|---|
| 1:N (同库) | `new_realm_plan` | `new_realm_plan.run_id → realm_lifecycle_run.id` | RESTRICT | ✅ |
| 1:N (同库) | `split_plan` | `split_plan.run_id → realm_lifecycle_run.id` | RESTRICT | ✅ |
| 1:N (同库) | `retire_plan` | `retire_plan.run_id → realm_lifecycle_run.id` | RESTRICT | ✅ |
| N:1 (跨域) | `admin_users` | `realm_lifecycle_run.operator_id = admin_users.id` | app-layer | ❌ 弱引用（**应物化**）|

### 分区策略

- **分区方式**：`PARTITION BY RANGE (created_at)`（per 0020_lcm_tables.sql:47）
- **初始分区**：当月 + 下月（per 0020_lcm_tables.sql:51-67 DO 块）
- **保留期**：3 年（36 个分区滚动保留，超期 `DETACH` + 归档或 `DROP`）
- **滚动任务**：生产环境由 cron job 维护（**当前未实施，PH-2 评审**）

---

## 6.5 `new_realm_plan` 新規レルム計画

### 概要

新建 realm 计划表（per M-2068.2 + RGS-DTL-042 §3）。`run_id` FK RESTRICT 防止 LCM 主表被误删。`target_player_count` / `target_tps` 应用层校验。

| 项目 | 内容 |
|---|---|
| 物理表名 | `new_realm_plan` |
| 論理名 | 新規レルム計画 / New Realm Plan |
| 出典 | `crates/cluster-ops/migrations/0020_lcm_tables.sql:83-93` |
| 父文档 | RGS-DTL-042 §3 / M-2068.2 |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `run_id` | 実行 ID / Run Identifier | UUID | 128-bit | — | `realm_lifecycle_run(id) ON DELETE RESTRICT` | — | ✅ | — | — | 所属 LCM run |
| 3 | `target_region` | 対象リージョン / Target Region | TEXT | 1-64 字符 | — | — | — | ✅ | — | — | 目标区域（如 `ap-northeast-1`）|
| 4 | `target_player_count` | 対象プレイヤー数 / Target Player Count | INTEGER | > 0 | — | — | — | ✅ | — | `target_player_count > 0` | 目标玩家数 |
| 5 | `target_tps` | 対象 TPS / Target TPS | INTEGER | > 0 | — | — | — | ✅ | — | `target_tps > 0` | 目标 TPS |
| 6 | `status` | 計画状態 / Plan Status | TEXT | — | — | — | — | ✅ | `'draft'` | `status IN ('draft', 'validated', 'executing', 'done', 'failed')` | 5 状态机 |
| 7 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `new_realm_plan_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `idx_new_realm_plan_run_id` | B-tree | `(run_id)` | FK 索引 + 查某 run 的所有 plan |
| 3 | `idx_new_realm_plan_status` | B-tree | `(status, created_at DESC)` | 按状态 + 时间 |
| 4 | `idx_new_realm_plan_target_region` | B-tree | `(target_region)` | 按目标区域筛选 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `new_realm_plan_pkey` | `(id)` |
| FOREIGN KEY | (隐式) | `(run_id) REFERENCES realm_lifecycle_run(id) ON DELETE RESTRICT` |
| CHECK | (隐式) `target_player_count_check` | `target_player_count > 0` |
| CHECK | (隐式) `target_tps_check` | `target_tps > 0` |
| CHECK | (隐式) `status_check` | `status IN ('draft', 'validated', 'executing', 'done', 'failed')` |

---

## 6.6 `split_plan` 分裂計画

### 概要

分服计划表（per M-2068.2 + DTL-042 §3）。`target_realm_count >= 2`（至少分裂为 2 个目标 realm）。`split_strategy` 3 选 1（hash / range / manual）。

| 项目 | 内容 |
|---|---|
| 物理表名 | `split_plan` |
| 論理名 | 分裂計画 / Split Plan |
| 出典 | `crates/cluster-ops/migrations/0020_lcm_tables.sql:103-114` |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `run_id` | 実行 ID / Run Identifier | UUID | 128-bit | — | `realm_lifecycle_run(id) ON DELETE RESTRICT` | — | ✅ | — | — | 所属 LCM run |
| 3 | `source_realm_id` | 元レルム ID / Source Realm Identifier | UUID | 128-bit | — | — | — | ✅ | — | — | 源 realm |
| 4 | `target_realm_count` | 対象レルム数 / Target Realm Count | INTEGER | >= 2 | — | — | — | ✅ | — | `target_realm_count >= 2` | 目标 realm 数（≥ 2）|
| 5 | `split_strategy` | 分裂戦略 / Split Strategy | TEXT | — | — | — | — | ✅ | — | `split_strategy IN ('hash', 'range', 'manual')` | 3 选 1 策略 |
| 6 | `status` | 計画状態 / Plan Status | TEXT | — | — | — | — | ✅ | `'draft'` | `status IN ('draft', 'validated', 'executing', 'done', 'failed')` | 5 状态机 |
| 7 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `split_plan_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `idx_split_plan_run_id` | B-tree | `(run_id)` | FK 索引 |
| 3 | `idx_split_plan_source_realm_id` | B-tree | `(source_realm_id)` | 按源 realm 筛选 |
| 4 | `idx_split_plan_status` | B-tree | `(status, created_at DESC)` | 按状态 + 时间 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `split_plan_pkey` | `(id)` |
| FOREIGN KEY | (隐式) | `(run_id) REFERENCES realm_lifecycle_run(id) ON DELETE RESTRICT` |
| CHECK | (隐式) `target_realm_count_check` | `target_realm_count >= 2` |
| CHECK | (隐式) `split_strategy_check` | `split_strategy IN ('hash', 'range', 'manual')` |
| CHECK | (隐式) `status_check` | `status IN ('draft', 'validated', 'executing', 'done', 'failed')` |

---

## 6.7 `merge_conflict_rule_set_v2` マージ競合ルールセット V2

### 概要

合服冲突规则集 v2 表（per M-2068.2 + FR-LCM-062 + RGS-SPEC-DTL-042 §5 幂等一致性）。`locked_at` 锁定后**不**允许运行时修改（应用层校验）。`rule_set_version` UNIQUE。**缺修改防护**（应用层校验 vs DB 强约束）——见 [17-P1-10](17-不合理设计识别与优化建议.md)。

| 项目 | 内容 |
|---|---|
| 物理表名 | `merge_conflict_rule_set_v2` |
| 論理名 | マージ競合ルールセット V2 / Merge Conflict Rule Set V2 |
| 出典 | `crates/cluster-ops/migrations/0020_lcm_tables.sql:124-137` |
| 父文档 | RGS-DTL-042 §3 / M-2068.2 / FR-LCM-062 |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `rule_set_version` | ルールセットバージョン / Rule Set Version | INTEGER | > 0 | — | — | ✅ | ✅ | — | `rule_set_version > 0` | 版本号（UNIQUE）|
| 3 | `rules` | ルール（JSONB） / Rules | JSONB | — | — | — | — | ✅ | `'[]'::jsonb` | — | 合服冲突规则列表 |
| 4 | `locked_at` | ロック日時 / Lock Timestamp | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | `(locked_at IS NULL AND locked_by IS NULL) OR (locked_at IS NOT NULL AND locked_by IS NOT NULL)` | 锁定时间（NULL = 未锁定）|
| 5 | `locked_by` | ロック実行者 / Locked By | UUID | 128-bit | — | — | — | ❌ | NULL | (同上) | 锁定者 admin_users.id |
| 6 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `merge_conflict_rule_set_v2_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `uq_merge_conflict_rule_set_version` | B-tree (UNIQUE) | `(rule_set_version)` | 版本号唯一 |
| 3 | `idx_merge_conflict_rule_set_version` | B-tree | `(rule_set_version DESC)` | 查最新版本 |
| 4 | `idx_merge_conflict_rule_set_locked` | partial B-tree | `(locked_at) WHERE locked_at IS NOT NULL` | 查已锁定规则集 |
| 5 | `idx_merge_conflict_rule_set_rules_gin` | GIN | `rules` | JSONB 路径查询（按 rule_id 等）|

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `merge_conflict_rule_set_v2_pkey` | `(id)` |
| UNIQUE | `uq_merge_conflict_rule_set_version` | `(rule_set_version)` |
| CHECK | (隐式) `rule_set_version_check` | `rule_set_version > 0` |
| CHECK | `chk_merge_conflict_lock_consistency` | `(locked_at IS NULL AND locked_by IS NULL) OR (locked_at IS NOT NULL AND locked_by IS NOT NULL)` |

### 既知偏差

- **`locked_at` 锁定后 UPDATE/DELETE 防护仅在应用层**——建议 PH-2 加 `BEFORE UPDATE/DELETE` 触发器（per ARC-023 极窄允许边界，属"不产生跨表副作用的单表数据完整性触发器"）——见 [17-P1-10](17-不合理设计识别与优化建议.md)
- 缺 `updated_at`（规则集修改时间无法追踪——锁定前应有 update_at）

---

## 6.8 `retire_plan` 退場計画

### 概要

退场计划表（per M-2068.3 + FR-LCM-007）。`query_channel_rbac` JSONB 配置退场后查询通道的允许角色（默认 `["cs_agent", "sre", "legal"]`）。`archive_threshold_days` 30-90 天。

| 项目 | 内容 |
|---|---|
| 物理表名 | `retire_plan` |
| 論理名 | 退場計画 / Retire Plan |
| 出典 | `crates/cluster-ops/migrations/0020_lcm_tables.sql:151-162` |
| 父文档 | RGS-DTL-042 §3 / M-2068.3 / FR-LCM-007 |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `run_id` | 実行 ID / Run Identifier | UUID | 128-bit | — | `realm_lifecycle_run(id) ON DELETE RESTRICT` | — | ✅ | — | — | 所属 LCM run |
| 3 | `target_realm_id` | 対象レルム ID / Target Realm Identifier | UUID | 128-bit | — | — | — | ✅ | — | — | 退场目标 realm |
| 4 | `archive_threshold_days` | アーカイブ閾値日数 / Archive Threshold Days | INTEGER | 30-90 | — | — | — | ✅ | — | `archive_threshold_days BETWEEN 30 AND 90` | 归档阈值（30-90 天）|
| 5 | `query_channel_rbac` | クエリチャネル RBAC（JSONB） / Query Channel RBAC | JSONB | — | — | — | — | ✅ | `'["cs_agent", "sre", "legal"]'::jsonb` | — | 退场后查询通道允许角色 |
| 6 | `status` | 計画状態 / Plan Status | TEXT | — | — | — | — | ✅ | `'draft'` | `status IN ('draft', 'validated', 'executing', 'done', 'failed')` | 5 状态机 |
| 7 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `retire_plan_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `idx_retire_plan_run_id` | B-tree | `(run_id)` | FK 索引 |
| 3 | `idx_retire_plan_target_realm_id` | B-tree | `(target_realm_id)` | 按目标 realm 筛选 |
| 4 | `idx_retire_plan_status` | B-tree | `(status, created_at DESC)` | 按状态 + 时间 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `retire_plan_pkey` | `(id)` |
| FOREIGN KEY | (隐式) | `(run_id) REFERENCES realm_lifecycle_run(id) ON DELETE RESTRICT` |
| CHECK | (隐式) `archive_threshold_days_check` | `archive_threshold_days BETWEEN 30 AND 90` |
| CHECK | (隐式) `status_check` | `status IN ('draft', 'validated', 'executing', 'done', 'failed')` |

---

## 6.9 `archive_policy` アーカイブポリシー

### 概要

归档策略表（per M-2068.3 + FR-LCM-081 + NFR-SE-010）。**不**含 DELETE/TRUNCATE 路径（per NFR-SE-010 双层审计，删除走 admin_db.audit_log）。`n_plus_2_redundancy` 3 副本冗余。`realm_id` UNIQUE 一 realm 一策略。

| 项目 | 内容 |
|---|---|
| 物理表名 | `archive_policy` |
| 論理名 | アーカイブポリシー / Archive Policy |
| 出典 | `crates/cluster-ops/migrations/0020_lcm_tables.sql:174-185` |
| 父文档 | RGS-DTL-042 §3 / M-2068.3 / FR-LCM-081 / NFR-SE-010 |

### カラム一覧

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `realm_id` | レルム ID / Realm Identifier | UUID | 128-bit | — | — | ✅ | ✅ | — | — | 一 realm 一策略 |
| 3 | `hot_storage_tier` | ホットストレージ階層 / Hot Storage Tier | TEXT | — | — | — | — | ✅ | — | `hot_storage_tier IN ('ssd', 'nvme', 'hdd')` | 热存储介质 |
| 4 | `cold_storage_tier` | コールドストレージ階層 / Cold Storage Tier | TEXT | — | — | — | — | ✅ | — | `cold_storage_tier IN ('object_storage', 'tape', 'glacier')` | 冷存储介质 |
| 5 | `hot_retention_years` | ホット保持年数 / Hot Retention Years | INTEGER | >= 3 | — | — | — | ✅ | — | `hot_retention_years >= 3` | 热保留年数 |
| 6 | `cold_retention_years` | コールド保持年数 / Cold Retention Years | INTEGER | >= 10 | — | — | — | ✅ | — | `cold_retention_years >= 10` | 冷保留年数 |
| 7 | `n_plus_2_redundancy` | N+2 冗長フラグ / N+2 Redundancy Flag | BOOLEAN | — | — | — | — | ✅ | TRUE | — | 是否启用 N+2 冗余 |
| 8 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `now()` | — | 创建时间 |

### インデックス

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `archive_policy_pkey` | B-tree (PK) | `(id)` | 主键（自动）|
| 2 | `archive_policy_realm_id_key` | B-tree (UNIQUE) | `(realm_id)` | 一 realm 一策略 |
| 3 | `idx_archive_policy_realm_id` | B-tree | `(realm_id)` | **与 (2) 重複**（同 P1-01）|
| 4 | `idx_archive_policy_n_plus_2` | partial B-tree | `(n_plus_2_redundancy) WHERE n_plus_2_redundancy = TRUE` | 查启用 N+2 的策略 |

### 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | `archive_policy_pkey` | `(id)` |
| UNIQUE | `archive_policy_realm_id_key` | `(realm_id)` |
| CHECK | (隐式) `hot_storage_tier_check` | `hot_storage_tier IN ('ssd', 'nvme', 'hdd')` |
| CHECK | (隐式) `cold_storage_tier_check` | `cold_storage_tier IN ('object_storage', 'tape', 'glacier')` |
| CHECK | (隐式) `hot_retention_years_check` | `hot_retention_years >= 3` |
| CHECK | (隐式) `cold_retention_years_check` | `cold_retention_years >= 10` |

### 既知偏差

- **缺 `updated_at`**：策略修改时间无法追踪（GDPR 双层审计要求）——见 [17-P1-11](17-不合理设计识别与优化建议.md)
- `idx_archive_policy_realm_id` 与 `archive_policy_realm_id_key` 重複（同 P1-01）

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| admin SQL | `crates/admin-service/migrations/0001_init.sql` + `0002_audit.sql` + `0003_outbox.sql` + `0004_outbox_check_idempotent.sql` |
| LCM SQL | `crates/cluster-ops/migrations/0020_lcm_tables.sql` |
| DTL-019 | `docs/01-核心架构与设计模式/RGS-DTL-019_详细设计书.md` |
| DTL-040 | `docs/01-核心架构与设计模式/RGS-DTL-040_Admin域_详细设计书.md` |
| DTL-042 | `docs/01-核心架构与设计模式/RGS-DTL-042_服务器全生命周期管理_详细设计书.md` |
| SEC-100 | `docs/01-核心架构与设计模式/RGS-SEC-100_*.md` §7 hash 链 |
| SPEC-DTL-042 | `docs/13-实施规范/RGS-SPEC-DTL-042_*.md` §3 / §5 / §8 |
| FR-LCM-001~081 | (in RGS-DTL-042 §3 / SPEC-DTL-042 §3) |

> 任何实际 schema 与本文档不一致之处，以 `crates/admin-service/migrations/*.sql` + `crates/cluster-ops/migrations/*.sql` 实际 SQL 为准。
