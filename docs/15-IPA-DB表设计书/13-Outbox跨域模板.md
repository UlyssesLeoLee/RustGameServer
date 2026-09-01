# 13-Outbox 跨域模板（Outbox Cross-Domain Template）

> **本文件定位**：6 域 outbox 表（player / economy / match / social / admin / cluster-ops）的**共享模板**。完整定义 12 列 + 3 索引 + 1 CHECK，避免在 6 个域表设计書中重复展开。

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-DB-OUTBOX-TPL |
| 版本 | 0.1 |
| 作成日 | 2026-09-01 JST |
| 適用範囲 | 6 个 PostgreSQL 库中各持一份的 `outbox` 表 |

---

## 1. 6 域分布

| # | 物理库 | 表名 | 担当 crate | 关联 saga 表 |
|---|---|---|---|---|
| 13.1 | `player_db.outbox` | 玩家域事件 | `player-service` | (无) |
| 13.2 | `economy_db.outbox` | 经济域事件 | `economy-service` | `sagas` (同库弱引用) |
| 13.3 | `match_db.outbox` | 匹配域事件 | `match-service` | (无) |
| 13.4 | `social_db.outbox` | 社交域事件 | `social-service` | (无) |
| 13.5 | `admin_db.outbox` | 管理域事件（含 LCM 转发）| `admin-service` | (无) |
| 13.6 | `cluster_ops_db.outbox` | 集群事件 | `cluster-ops` | (无) |

> **重複不冗余**：每域各持一份以保证**库自治 + 故障隔离**（per RGS-DTL-100 §5.3）。Worker 跨域消费通过 NATS subject 路由。

---

## 2. 模板：完整表定义

### 2.1 カラム一覧（12 列，6 域共用）

| # | 物理名 | 論理名 | データ型 | 桁数/範囲 | PK | FK | UK | NOT NULL | DEFAULT | CHECK | 説明 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `id` | 識別子 / Identifier | UUID | 128-bit | ✅ | — | — | ✅ | — | — | 主键 |
| 2 | `subject` | NATS 件名 / NATS Subject | VARCHAR | 256 | — | — | — | ✅ | — | — | NATS subject（限 256 字符，per NATS spec）|
| 3 | `payload` | ペイロード / Payload | JSONB | — | — | — | — | ✅ | — | — | 事件 payload（JSONB 序列化）|
| 4 | `command_id` | コマンド ID / Command Identifier | UUID | 128-bit | — | — | — | ✅ | — | — | 幂等键（对应 command）|
| 5 | `saga_id` | サーガ ID / Saga Identifier | UUID | 128-bit | — | — (跨域弱引用 → economy_db.sagas) | — | ❌ | NULL | — | 关联 saga（仅 economy 域命令关联，其他域常 NULL）|
| 6 | `status` | 状態 / Status | VARCHAR | 16 | — | — | — | ✅ | `'pending'` | `status IN ('pending', 'in_flight', 'sent', 'failed')` | 4 状态机 |
| 7 | `retry_count` | リトライ回数 / Retry Count | INTEGER | >= 0 | — | — | — | ✅ | 0 | — | 重试次数 |
| 8 | `last_error` | 最新エラーメッセージ / Latest Error | TEXT | — | — | — | — | ❌ | NULL | — | 最近错误信息（失败时填）|
| 9 | `lease_until` | リース期限 / Lease Expiration | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | worker lease 截止（in_flight 时填）|
| 10 | `created_at` | 作成日時 / Creation Timestamp | TIMESTAMPTZ | — | — | — | — | ✅ | `NOW()` | — | 创建时间 |
| 11 | `sent_at` | 送信日時 / Send Timestamp | TIMESTAMPTZ | — | — | — | — | ❌ | NULL | — | 发送成功时间 |
| (隐式) | `CONSTRAINT chk_outbox_status` | CHECK 制約 | — | — | — | — | — | — | — | `status IN ('pending', 'in_flight', 'sent', 'failed')` | 由 CHECK 自动命名 |

### 2.2 インデックス（3 个，6 域共用）

| # | 索引名 | 種別 | 列 | 目的 |
|---|---|---|---|---|
| 1 | `idx_outbox_pending` | partial B-tree | `(created_at) WHERE status = 'pending'` | 调度器拉取（按时间 FIFO）|
| 2 | `idx_outbox_in_flight` | partial B-tree | `(lease_until) WHERE status = 'in_flight'` | lease 过期扫描（worker crash recovery）|
| 3 | `idx_outbox_command_id` | B-tree | `(command_id)` | 幂等查重（at-least-once 投递去重）|

### 2.3 制約一覧

| 種別 | 名前 | 表达式 |
|---|---|---|
| PRIMARY KEY | (PostgreSQL 自动 `outbox_pkey`) | `(id)` |
| CHECK | (PG 自动命名) | `status IN ('pending', 'in_flight', 'sent', 'failed')` |

### 2.4 典型 DDL（模板原样复制到 6 域 migration）

```sql
-- 模板（per crates/economy-service/migrations/0003_outbox.sql 等）
CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    subject VARCHAR(256) NOT NULL,
    payload JSONB NOT NULL,
    command_id UUID NOT NULL,
    saga_id UUID,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    lease_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox (created_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_outbox_in_flight ON outbox (lease_until) WHERE status = 'in_flight';
CREATE INDEX IF NOT EXISTS idx_outbox_command_id ON outbox (command_id);
```

---

## 3. Known Drift：CHECK 静默失效 + 0003 修复

### 3.1 反 pattern（per 0003_outbox_check_idempotent.sql 注释）

**问题**：
- 0002 写入 outbox 表时，把 `CHECK (status IN (...))` 内联在 `CREATE TABLE IF NOT EXISTS` 块内
- 0002 部署成功后，0003/0004 在已部署环境跑 `CREATE TABLE IF NOT EXISTS` → PG 静默跳过整个块 → **CHECK 约束永不生效**
- 后果：业务层写入时即使 `status='invalid'`，DB 也不报错——纯应用层校验，违反"defense in depth"

**修复模式（per 0003_outbox_check_idempotent.sql）**：
```sql
-- 0003_outbox_check_idempotent.sql
DO $$
BEGIN
    -- 幂等补强：用 ALTER TABLE ADD CONSTRAINT 替代内联 CHECK
    -- 用 IF NOT EXISTS 守卫（PG 13+ 支持 CREATE TABLE IF NOT EXISTS 但不支持 ADD CONSTRAINT IF NOT EXISTS）
    -- 实际用 pg_constraint 查询做存在性检查
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_outbox_status') THEN
        ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'));
    END IF;
END $$;
```

### 3.2 6 域修复状态

| 域 | 0002 CHECK 内联 | 0003 修复 | 修复状态 |
|---|---|---|---|
| admin_db | ✅ 存在 | `0004_outbox_check_idempotent.sql` | ✅ 修复 |
| cluster_ops_db | ✅ 存在 | `0003_outbox_check_idempotent.sql` | ✅ 修复 |
| economy_db | ✅ 存在 | `0004_outbox_check_idempotent.sql` | ✅ 修复 |
| match_db | ✅ 存在 | `0003_outbox_check_idempotent.sql` | ✅ 修复 |
| player_db | ✅ 存在 | `0003_outbox_check_idempotent.sql` | ✅ 修复 |
| social_db | ✅ 存在 | `0003_outbox_check_idempotent.sql` | ✅ 修复 |

> **6 域全部已修复** ✅。但需在每个新域的 outbox migration 中**显式采用**此模式（**不要**回到 0002 反 pattern）。

### 3.3 防御策略（PH-2 评审）

建议所有未来 outbox 类高频写表的 CHECK 约束：
- **不要**内联在 `CREATE TABLE IF NOT EXISTS` 块内
- **必须**用 `DO $$ ... ALTER TABLE ADD CONSTRAINT ... $$` 幂等模式
- 详细规则应写入 RGS-BAS-007 v0.3 + RGS-IMPL-002 v0.2

---

## 4. 6 域 subject 命名规范（per 域 outbox 特有应用层）

| 域 | subject 前缀 | 例子 |
|---|---|---|
| player | `player.` | `player.profile.updated`, `player.character.created`, `player.deck.shared` |
| economy | `economy.` | `economy.transaction.confirmed`, `economy.saga.started`, `economy.account.frozen` |
| match | `match.` | `match.session.created`, `match.move.played`, `match.matchmaking.matched` |
| social | `social.` | `social.guild.created`, `social.guild.member.joined`, `social.guild.experience.donated` |
| admin | `admin.` | `admin.gm_command.issued`, `admin.audit_log.queried`, `admin.realm_lifecycle.<subtype>.started` |
| cluster_ops | `cluster_ops.` | `cluster_ops.node.heartbeat_lost`, `cluster_ops.feature_flag.toggled` |

---

## 5. Worker 跨域消费（at-least-once + 幂等）

- 6 域 outbox 由各域独立 worker 消费
- worker 流程：拉取 `status='pending'` 行 → 标记 `status='in_flight' lease_until=now()+30s` → 投递 NATS → 成功标记 `status='sent' sent_at=now()` / 失败标记 `status='failed' last_error=...` retry_count++
- **跨域幂等**：消费方收到事件后，inbox 表（仅 economy 域有）做 (command_id, handler) UNIQUE 去重
- **lease 过期恢复**：worker crash 后 `lease_until < now()` 的 `in_flight` 行被其他 worker 重新拉取

---

## 修订追溯

| 引用 | 路径 |
|---|---|
| 6 域 outbox SQL | `crates/admin-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`<br/>`crates/cluster-ops/migrations/0002_outbox.sql` + `0003_outbox_check_idempotent.sql`<br/>`crates/economy-service/migrations/0003_outbox.sql` + `0004_outbox_check_idempotent.sql`<br/>`crates/match-service/migrations/0002_outbox.sql` + `0003_outbox_check_idempotent.sql`<br/>`crates/player-service/migrations/0002_outbox.sql` + `0003_outbox_check_idempotent.sql`<br/>`crates/social-service/migrations/0002_outbox.sql` + `0003_outbox_check_idempotent.sql` |
| DTL-100 | `docs/01-核心架构与设计模式/RGS-DTL-100_Saga业务模式定义_v0.1.md` §5.3 |
| BAS-007 | `docs/03-数据经济与交易/RGS-BAS-007_*.md` §4（outbox 分区策略） |
| RGS-REV-009 | `docs/00-准备阶段/RGS-REV-009_*.md` CR-2（0003 反 pattern 防御） |
| WF-1-55.28 | (in RGS-OPEN-QA / RGS-REV 复盘) |

> 任何实际 schema 与本文档不一致之处，以 `crates/*/migrations/000*_outbox*.sql` 实际 SQL 为准。
