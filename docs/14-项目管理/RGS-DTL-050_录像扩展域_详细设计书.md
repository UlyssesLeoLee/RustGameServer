# 详细设计书（詳細設計書 / Detailed Document）

**录像扩展域（Replay-Extra Domain）详细设计 — 元数据 / 互动 / 收藏物理 DDL + 级联失效算法**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-050 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-043 v0.1 录像扩展域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/replay-extra-service/` (replay_extra_db, 386 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-051 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-043 §3〜§7 为：replay_extra_db 物理 DDL（`replay_metadata`/`share_links`/`replay_likes`/`replay_comments`/`replay_favorites`/`visibility_configs`/`favorite_configs`）、录像删除级联失效伪代码（per NFR-RPL-002 RSK-RPL-001 关键路径）、分享链接 token 生成 + 密码哈希算法 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 录像删除级联失效算法是否完整覆盖 likes/comments/favorites/share_links 4 类 |
| 评审（DBA） | | | replay_extra_db 索引是否覆盖高频路径（元数据查询 / 互动查询） |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：replay_extra_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

RGS-BAS-043 给出了录像扩展域 1 service 的组件划分、接口契约、核心时序、ARC-051 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-043 已确定的任何结构性选择。
- 不覆盖录像文件存储底层（属 replay-service 既有职责）。
- 不覆盖录像生成（由上游 battle-service 等触发）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：replay_extra_db

replay_extra_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 7 张表。

## 2.1 replay_metadata（录像元数据表）

```sql
CREATE TABLE replay_metadata (
    replay_id          VARCHAR(64) PRIMARY KEY,
    owner_id           UUID NOT NULL,  -- 跨库引用 player_db.characters
    title              VARCHAR(256) NOT NULL,
    description        TEXT,
    tags               JSONB NOT NULL DEFAULT '[]'::jsonb,
    visibility         SMALLINT NOT NULL DEFAULT 0,  -- 0=PRIVATE 1=FRIENDS 2=PUBLIC
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC
    deleted_at         TIMESTAMPTZ,                  -- 软删除, 级联失效触发
    CONSTRAINT chk_replay_metadata_visibility CHECK (visibility BETWEEN 0 AND 2)
);
CREATE INDEX idx_replay_metadata_owner
    ON replay_metadata (owner_id, updated_at DESC) WHERE deleted_at IS NULL;
    -- 支撑"玩家录像列表"高频查询
```

## 2.2 share_links（分享链接表）

```sql
CREATE TABLE share_links (
    share_token        VARCHAR(64) PRIMARY KEY,  -- 256 bit random, base64 编码
    replay_id          VARCHAR(64) NOT NULL,
    password_hash      VARCHAR(255),              -- Argon2 哈希 (per NFR-RPL-003 高强度)
    expires_at_ms      BIGINT NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked            BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_share_links_replay
    ON share_links (replay_id, revoked) WHERE revoked = FALSE;
```

## 2.3 replay_likes（录像点赞表）

```sql
CREATE TABLE replay_likes (
    replay_id          VARCHAR(64) NOT NULL,
    player_id          UUID NOT NULL,
    liked_at_ms        BIGINT NOT NULL,
    PRIMARY KEY (replay_id, player_id)            -- 每录像每玩家至多 1 次点赞
);
CREATE INDEX idx_replay_likes_player
    ON replay_likes (player_id);
```

## 2.4 replay_comments（录像评论表）

```sql
CREATE TABLE replay_comments (
    comment_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    replay_id          VARCHAR(64) NOT NULL,
    player_id          UUID NOT NULL,
    content            TEXT NOT NULL,
    created_at_ms      BIGINT NOT NULL,
    deleted_at         TIMESTAMPTZ                  -- 软删除
);
CREATE INDEX idx_replay_comments_replay_time
    ON replay_comments (replay_id, created_at_ms DESC) WHERE deleted_at IS NULL;
```

## 2.5 replay_favorites（录像收藏表）

```sql
CREATE TABLE replay_favorites (
    favorite_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id          UUID NOT NULL,
    replay_id          VARCHAR(64) NOT NULL,
    category           VARCHAR(64) NOT NULL DEFAULT 'default',
    favorited_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (player_id, replay_id, category)
);
CREATE INDEX idx_replay_favorites_player
    ON replay_favorites (player_id, category);
```

## 2.6 visibility_configs（可见性配置表）

```sql
CREATE TABLE visibility_configs (
    replay_type        VARCHAR(32) PRIMARY KEY,  -- PVP_RANKED / PVE / ROOM / ...
    default_visibility SMALLINT NOT NULL DEFAULT 0,
    require_password_for_private BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 2.7 favorite_configs（收藏上限配置表）

```sql
CREATE TABLE favorite_configs (
    player_level       INTEGER PRIMARY KEY,
    max_favorite_count INTEGER NOT NULL,
    max_categories     INTEGER NOT NULL
);
```

---

# 3. 关键算法详细设计

## 3.1 分享链接 token 生成（NFR-RPL-003 高强度）

```rust
use rand::Rng;
fn generate_share_token() -> String {
    // 256 bit 随机 token, base64 url-safe 编码
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    base64::URL_SAFE_NO_PAD.encode(&bytes)
}
```

## 3.2 密码哈希 + 校验

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
fn hash_password(plain: &str) -> Result<String, ReplayError> {
    let salt = generate_salt();  // 128 bit random
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(plain.as_bytes(), &salt)
        .map_err(|e| ReplayError::PasswordHashError(e.to_string()))?;
    Ok(hash.to_string())
}

fn verify_password(plain: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default().verify_password(plain.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
    }
}
```

## 3.3 录像删除级联失效（RSK-RPL-001 关键路径）

```rust
// 落实 RGS-BAS-043 §5.3 录像删除 → 互动级联失效
// 当 replay-service 发出 ReplayDeleted 事件时, 必须级联失效:
//   1. likes (本域)
//   2. comments (本域, 软删除)
//   3. favorites (本域)
//   4. share_links (本域, 标记 revoked)
// 5 一切级联**必须**在同一事务内 (per NFR-RPL-002)
fn cascade_replay_deletion(replay_id: ReplayId) -> Result<CascadeResult, ReplayError> {
    let mut tx = pool.begin().await?;
    // 1. 软删除元数据
    sqlx::query("UPDATE replay_metadata SET deleted_at = now() WHERE replay_id = $1")
        .bind(&replay_id).execute(&mut *tx).await?;
    // 2. 删除 likes
    let likes_deleted = sqlx::query("DELETE FROM replay_likes WHERE replay_id = $1")
        .bind(&replay_id).execute(&mut *tx).await?.rows_affected();
    // 3. 软删除 comments
    let comments_deleted = sqlx::query("UPDATE replay_comments SET deleted_at = now() WHERE replay_id = $1 AND deleted_at IS NULL")
        .bind(&replay_id).execute(&mut *tx).await?.rows_affected();
    // 4. 删除 favorites
    let favorites_deleted = sqlx::query("DELETE FROM replay_favorites WHERE replay_id = $1")
        .bind(&replay_id).execute(&mut *tx).await?.rows_affected();
    // 5. 标记 share_links revoked
    let links_revoked = sqlx::query("UPDATE share_links SET revoked = TRUE WHERE replay_id = $1")
        .bind(&replay_id).execute(&mut *tx).await?.rows_affected();
    tx.commit().await?;
    Ok(CascadeResult { likes_deleted, comments_deleted, favorites_deleted, links_revoked })
}
```

## 3.4 点赞 UNIQUE 约束 + 防并发双点赞

```rust
// 落实 FR-RPL-020 每录像每玩家至多 1 次
// UNIQUE(replay_id, player_id) 在数据库层强制, 应用层仅需处理 DuplicateKey 错误
fn like_replay(replay_id: ReplayId, player_id: PlayerId) -> Result<(), ReplayError> {
    match sqlx::query("INSERT INTO replay_likes (replay_id, player_id, liked_at_ms) VALUES ($1, $2, $3)")
        .bind(&replay_id).bind(player_id).bind(now_ms())
        .execute(&pool).await {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(ReplayError::AlreadyLiked),
        Err(e) => Err(ReplayError::Database(e)),
    }
}
```

## 3.5 评论限频（FR-RPL-021 每录像每玩家至多 N 条/24h）

```rust
// 限频: 每录像每玩家 24h 内至多 N 条评论 (N 由 FavoriteConfig 决定, 默认 10)
async fn can_comment(replay_id: ReplayId, player_id: PlayerId, max_per_24h: i64) -> Result<bool, ReplayError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM replay_comments WHERE replay_id = $1 AND player_id = $2 AND created_at_ms > $3"
    )
    .bind(&replay_id).bind(player_id).bind(now_ms() - 24 * 3600 * 1000)
    .fetch_one(&pool).await?;
    Ok(count < max_per_24h)
}
```

---

# 4. 对接点

## 4.1 与 replay-service 既有的对接（CON-RPL-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 录像文件元数据查询 | `GetReplayMetadata` | replay-service 既有 | 本域仅扩展元数据 |
| 录像删除事件 | `ReplayDeleted event` | replay-service 既有 | 触发级联失效 |

## 4.2 与既有 RGS-BAS-025 反作弊体系的对接（CON-RPL-002 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 录像举报 | `ReportReplay` | RGS-BAS-025 §3 | 本域**不**自建举报 |

## 4.3 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| VisibilityConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 仅订阅 config_updated |
| FavoriteConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 同上 |

---

# 5. 数据库 DDL 权威边界

replay_extra_db 共 7 张表，以本文档为唯一权威。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| 元数据查询 | < 100ms | idx_replay_metadata_owner | PH-2 |
| 分享链接 token 生成 | < 10ms | 256 bit random + base64 | PH-2 |
| 录像删除级联失效 | < 200ms | 同事务 5 步 | PH-4 |
| 评论限频查询 | < 30ms | idx_replay_comments_replay_time | PH-2 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：

- replay_extra_db 7 张表的物理 DDL
- 录像删除级联失效伪代码（RSK-RPL-001 关键路径）
- 分享链接 token 生成 + 密码哈希（NFR-RPL-003）
- 点赞 UNIQUE 约束 + 防并发双点赞
- 评论限频算法
- 3 项对接点

本版本明确不覆盖、留待后续：

- 录像文件存储后端选型 — 属 replay-service 既有
- 录像生成 — 由上游触发，本域仅消费

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-043 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-043 §3 业务需求 | §2, §3 |
| RGS-REQ-043 §4 功能需求 | §3.4, §3.5 |
| RGS-REQ-043 §5 NFR | §6 |
| RGS-REQ-043 §6 ARC-051 | §2.6, §2.7, §3 |
| RGS-REQ-043 §7 AC-RPL-001〜004 | §3.3, §3.4, §3.5 |
| RGS-REQ-043 §8 RSK-RPL-001 | §3.3 |
| RGS-BAS-043 §3 架构总览 | §2 |
| RGS-BAS-043 §4 组件设计 | §3.1〜3.5 |
| RGS-BAS-043 §5 数据流时序 | §3.3, §3.4 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.3 |
