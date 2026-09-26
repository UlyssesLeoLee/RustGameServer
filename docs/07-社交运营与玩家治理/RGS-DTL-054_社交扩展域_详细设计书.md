# 详细设计书（詳細設計書 / Detailed Document）

**社交扩展域（Social-Extra Domain）详细设计 — 邮件 / 好友扩展 / 家园 / 聊天扩展物理 DDL + 邮件附件 EC 事务边界**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-054 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-047 v0.1 社交扩展域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/social-extra-service/` (social_extra_db, 540 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-055 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-047 §3〜§7 为：social_extra_db 物理 DDL（`mails`/`mail_attachments`/`friend_interactions`/`friend_remarks`/`homes`/`home_visits`/`chat_emoji_usage`）、邮件附件领取 EC 事务边界（per FR-GOV-001，含幂等防双领，RSK-SOC-001）、家园访问次数伪代码、表情包 ChatAbuseGuard 委托路径 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 邮件附件领取幂等防双领；表情包 ChatAbuseGuard 委托路径 |
| 评审（DBA） | | | social_extra_db 索引是否覆盖高频路径 |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：social_extra_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

RGS-BAS-047 给出了社交扩展域 1 service 的组件划分、接口契约、核心时序、ARC-055 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-047 已确定的任何结构性选择。
- 不覆盖好友关系核心（属 social-service 既有）。
- 不覆盖频道聊天核心（属 RGS-DTL-013 §3 既有）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：social_extra_db

social_extra_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 7 张表。

## 2.1 mails（站内邮件表）

```sql
CREATE TABLE mails (
    mail_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_id            VARCHAR(64) NOT NULL,  -- 系统 / 玩家 / GM
    to_id              UUID NOT NULL,
    mail_type          SMALLINT NOT NULL,  -- 0=SYSTEM 1=PLAYER 2=GM
    title              VARCHAR(256) NOT NULL,
    body               TEXT,
    read_at_ms         BIGINT,                -- 首次读取时间 (NULL=未读)
    deleted_at         TIMESTAMPTZ,           -- 软删除
    expires_at_ms      BIGINT NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_mails_type CHECK (mail_type BETWEEN 0 AND 2)
);
CREATE INDEX idx_mails_to_unread
    ON mails (to_id, created_at DESC) WHERE deleted_at IS NULL AND read_at_ms IS NULL;
    -- 支撑"玩家未读邮件"高频查询
```

## 2.2 mail_attachments（邮件附件表）

```sql
CREATE TABLE mail_attachments (
    attachment_id      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mail_id            UUID NOT NULL REFERENCES mails(mail_id) ON DELETE CASCADE,
    item_id            VARCHAR(64) NOT NULL,
    count              INTEGER NOT NULL,
    claimed            BOOLEAN NOT NULL DEFAULT FALSE,
    claimed_at_ms      BIGINT,
    UNIQUE (mail_id, item_id)  -- 同一邮件同一 item 至多 1 行
);
CREATE INDEX idx_mail_attachments_mail_unclaimed
    ON mail_attachments (mail_id) WHERE claimed = FALSE;
```

## 2.3 friend_interactions（好友互动表）

```sql
CREATE TABLE friend_interactions (
    interaction_id     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_player_id     UUID NOT NULL,
    to_player_id       UUID NOT NULL,
    interaction_type   SMALLINT NOT NULL,  -- 0=LIKE 1=COMMENT
    content            TEXT,                -- 仅 COMMENT 有内容
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_friend_interactions_to_time
    ON friend_interactions (to_player_id, created_at DESC);
```

## 2.4 friend_remarks（好友备注表）

```sql
CREATE TABLE friend_remarks (
    player_id          UUID NOT NULL,
    friend_id          UUID NOT NULL,
    remark             VARCHAR(64),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (player_id, friend_id)
);
```

## 2.5 homes（家园表）

```sql
CREATE TABLE homes (
    player_id          UUID PRIMARY KEY,
    theme_id           INTEGER NOT NULL DEFAULT 0,
    slots              JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 装饰 slot 数组
    visit_count        INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 2.6 home_visits（家园访问记录表）

```sql
CREATE TABLE home_visits (
    visit_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    host_id            UUID NOT NULL,
    visitor_id         UUID NOT NULL,
    message            TEXT,                  -- 留言
    visited_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_home_visits_host_time
    ON home_visits (host_id, visited_at DESC);
```

## 2.7 chat_emoji_usage（聊天表情包使用记录表）

```sql
CREATE TABLE chat_emoji_usage (
    usage_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id          UUID NOT NULL,
    emoji_id           VARCHAR(64) NOT NULL,
    channel            VARCHAR(32) NOT NULL,
    used_at_ms         BIGINT NOT NULL
);
CREATE INDEX idx_chat_emoji_usage_player_time
    ON chat_emoji_usage (player_id, used_at_ms DESC);
    -- 支撑"表情包使用限频"查询
```

---

# 3. 关键算法详细设计

## 3.1 邮件附件领取 EC 事务边界（per FR-GOV-001 + 幂等防双领，RSK-SOC-001）

```rust
// 落实 RGS-BAS-047 §5.1 邮件附件领取
// 关键:
//   1. 经 EC 单点发放
//   2. 幂等防双领 (claimed 字段 + UNIQUE 索引)
//   3. 邮件状态变更与附件领取在同一事务
fn claim_mail_attachment(
    player_id: PlayerId,
    mail_id: MailId,
    session_epoch: SessionEpoch,
) -> Result<GrantResult, SocialError> {
    let mut tx = pool.begin().await?;
    // 1. 校验邮件存在 + 未删除 + 未过期
    let mail: Mail = sqlx::query_as("SELECT * FROM mails WHERE mail_id = $1 AND to_id = $2 AND deleted_at IS NULL AND expires_at_ms > $3 FOR UPDATE")
        .bind(mail_id).bind(player_id).bind(now_ms())
        .fetch_optional(&mut *tx).await?
        .ok_or(SocialError::MailNotFound { mail_id })?;
    // 2. 获取未领取附件 (claim 阶段 OCC 起点)
    let attachments: Vec<MailAttachment> = sqlx::query_as(
        "SELECT * FROM mail_attachments WHERE mail_id = $1 AND claimed = FALSE FOR UPDATE"
    )
    .bind(mail_id).fetch_all(&mut *tx).await?;
    if attachments.is_empty() {
        return Err(SocialError::AlreadyClaimed { mail_id });
    }
    // 3. 经 EC 单点发放 (per FR-GOV-001)
    let grants: Vec<ItemGrant> = attachments.iter().map(|a| ItemGrant {
        item_id: a.item_id.clone(),
        count: a.count,
    }).collect();
    let grant_result = commit_transaction(CommitTransactionRequest {
        request_id: deterministic_hash(&[player_id.as_bytes(), mail_id.as_bytes()]),
        character_id: player_id.into(),
        operation: Operation::GrantItems(grants),
        session_epoch,
        expected_version: current_wallet_version(player_id)?,
    })?;
    // 4. 标记附件已领取 (OCC, 防并发双领)
    sqlx::query("UPDATE mail_attachments SET claimed = TRUE, claimed_at_ms = $1 WHERE mail_id = $2 AND claimed = FALSE")
        .bind(now_ms()).bind(mail_id).execute(&mut *tx).await?;
    // 5. 标记邮件已读
    if mail.read_at_ms.is_none() {
        sqlx::query("UPDATE mails SET read_at_ms = $1 WHERE mail_id = $2")
            .bind(now_ms()).bind(mail_id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(grant_result)
}
```

## 3.2 好友点赞（扩展走既有好友关系）

```rust
// 落实 RGS-BAS-047 §5.2 好友点赞
// 复用 social-service 既有的好友关系校验 (per CON-SOC-001)
async fn like_friend(player_id: PlayerId, friend_id: PlayerId) -> Result<(), SocialError> {
    // 1. 复用 social-service 校验好友关系
    let is_friend = call_social_check_friend_relation(player_id, friend_id).await?;
    if !is_friend {
        return Err(SocialError::NotFriend { player_id, friend_id });
    }
    // 2. 写入 friend_interactions (幂等: 同一玩家同一好友每日至多 1 次点赞)
    let today_start = today_start_ms();
    let existing = sqlx::query_scalar(
        "SELECT COUNT(*) FROM friend_interactions WHERE from_player_id = $1 AND to_player_id = $2 AND interaction_type = 0 AND created_at >= to_timestamp($3)"
    )
    .bind(player_id).bind(friend_id).bind(today_start as f64 / 1000.0)
    .fetch_one(&pool).await?;
    if existing > 0 {
        return Ok(());  // 今日已点赞, 幂等返回 OK
    }
    sqlx::query(
        "INSERT INTO friend_interactions (from_player_id, to_player_id, interaction_type, content) VALUES ($1, $2, 0, NULL)"
    )
    .bind(player_id).bind(friend_id).execute(&pool).await?;
    Ok(())
}
```

## 3.3 家园访问计数 + 留言

```rust
fn visit_home(host_id: PlayerId, visitor_id: PlayerId, message: Option<String>) -> Result<HomeData, SocialError> {
    let mut tx = pool.begin().await?;
    // 1. 校验家园存在
    let home: Home = sqlx::query_as("SELECT * FROM homes WHERE player_id = $1 FOR UPDATE")
        .bind(host_id).fetch_optional(&mut *tx).await?
        .ok_or(SocialError::HomeNotFound { player_id: host_id })?;
    // 2. 写入访问记录
    sqlx::query("INSERT INTO home_visits (host_id, visitor_id, message) VALUES ($1, $2, $3)")
        .bind(host_id).bind(visitor_id).bind(&message)
        .execute(&mut *tx).await?;
    // 3. 访问次数 + 1 (OCC)
    sqlx::query("UPDATE homes SET visit_count = visit_count + 1, updated_at = now() WHERE player_id = $1")
        .bind(host_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(home.with_visit_count(home.visit_count + 1))
}
```

## 3.4 表情包 ChatAbuseGuard 委托

```rust
// 落实 RGS-BAS-047 §5.3 表情包 ChatAbuseGuard 委托
// 严格复用 RGS-BAS-014 §3 ChatAbuseGuard 既有, 不自建
fn use_emoji(player_id: PlayerId, emoji_id: EmojiId, channel: Channel) -> Result<(), SocialError> {
    let config = load_emoji_config(&emoji_id)?;
    if !config.enabled {
        return Err(SocialError::EmojiDisabled { emoji_id });
    }
    // 1. 限频检查 (per ChatEmojiConfig.max_per_message, 但实际按窗口)
    let window_start = now_ms() - 60_000;  // 1 分钟窗口
    let recent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM chat_emoji_usage WHERE player_id = $1 AND emoji_id = $2 AND used_at_ms > $3"
    )
    .bind(player_id).bind(&emoji_id).bind(window_start)
    .fetch_one(&pool).await?;
    if recent_count >= config.max_per_message as i64 {
        return Err(SocialError::EmojiRateLimited { emoji_id });
    }
    // 2. ChatAbuseGuard 校验 (per CON-SOC-004)
    let abuse_check = call_chat_abuse_guard_check(player_id, channel, &emoji_id)?;
    if abuse_check.is_rejected {
        return Err(SocialError::EmojiRejected { emoji_id });
    }
    // 3. 写入使用记录
    sqlx::query("INSERT INTO chat_emoji_usage (player_id, emoji_id, channel, used_at_ms) VALUES ($1, $2, $3, $4)")
        .bind(player_id).bind(&emoji_id).bind(&channel).bind(now_ms())
        .execute(&pool).await?;
    // 4. 委托给 RGS-DTL-013 聊天核心 (per CON-SOC-002)
    call_dtl013_send_message_with_emoji(player_id, channel, emoji_id)?;
    Ok(())
}
```

---

# 4. 对接点

## 4.1 与 social-service 既有的对接（CON-SOC-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 好友关系校验 | `CheckFriendRelation` | social-service 既有 | 本域**不**自建 |

## 4.2 与 RGS-DTL-013 §3 既有的对接（CON-SOC-002 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 频道聊天发送 | `SendMessage` | RGS-DTL-013 §3 既有 | 本域**不**自建 |
| 聊天历史查询 | `GetChatHistory` | RGS-DTL-013 §3 既有 | 严格复用 |

## 4.3 与 EconomyService 的对接（CON-SOC-003 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 邮件附件领取 | `commit_transaction(GrantItems)` | RGS-DTL-001 §3.2 | 走 EC 既有路径 |

## 4.4 与 RGS-BAS-014 §3 ChatAbuseGuard 的对接（CON-SOC-004 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 表情包校验 | `CheckEmoji` | RGS-BAS-014 §3 | 本域**不**自建 |

## 4.5 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| MailConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 仅订阅 config_updated |
| HomeConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 同上 |
| ChatEmojiConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 同上 |

---

# 5. 数据库 DDL 权威边界

social_extra_db 共 7 张表，以本文档为唯一权威。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| 邮件查询 | < 100ms | idx_mails_to_unread | PH-2 |
| 邮件附件领取 | < 200ms | EC 单点 + 同事务 5 步 | PH-4 |
| 好友点赞 | < 50ms | 复用 social-service 校验 | PH-2 |
| 表情包使用 | < 100ms | ChatAbuseGuard 委托 | PH-2 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：

- social_extra_db 7 张表的物理 DDL
- 邮件附件领取 EC 事务边界（含幂等防双领，RSK-SOC-001）
- 好友点赞（扩展走既有 social-service）
- 家园访问计数 + 留言
- 表情包 ChatAbuseGuard 委托路径
- 5 项对接点

本版本明确不覆盖、留待后续：

- 好友关系核心 — 属 social-service 既有
- 频道聊天核心 — 属 RGS-DTL-013 §3 既有
- TBD-SOC-001（邮件保留期）— 留待 PH-2 评审

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-047 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-047 §3 业务需求 | §2, §3 |
| RGS-REQ-047 §4 功能需求 | §3.1, §3.2, §3.3, §3.4 |
| RGS-REQ-047 §5 NFR | §6 |
| RGS-REQ-047 §6 ARC-055 | §2, §3 |
| RGS-REQ-047 §7 AC-SOC-001〜004 | §3.1, §3.2, §3.4 |
| RGS-REQ-047 §8 RSK-SOC-001 | §3.1 |
| RGS-BAS-047 §3 架构总览 | §2 |
| RGS-BAS-047 §4 组件设计 | §3.1〜3.4 |
| RGS-BAS-047 §5 数据流时序 | §3.1, §3.2, §3.4 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.1 |
