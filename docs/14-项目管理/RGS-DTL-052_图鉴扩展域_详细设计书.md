# 详细设计书（詳細設計書 / Detailed Document）

**图鉴扩展域（Leaderboard-Extra Domain）详细设计 — 成就 / 收藏 / 称号物理 DDL + 触发条件评估算法**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-052 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-045 v0.1 图鉴扩展域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/leaderboard-extra-service/` (leaderboard_extra_db, 405 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-053 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-045 §3〜§7 为：leaderboard_extra_db 物理 DDL（`player_achievements`/`player_collections`/`player_titles`/`achievement_progress`/`achievement_configs`/`collection_configs`/`title_configs`）、成就触发条件评估算法（CUMULATIVE / THRESHOLD 两种）、并发触发防护（RSK-LBX-001）、阶段性奖励触发算法 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 成就触发条件评估是否严格服务器权威；并发触发防护是否完整 |
| 评审（DBA） | | | leaderboard_extra_db 索引是否覆盖高频路径 |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：leaderboard_extra_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

RGS-BAS-045 给出了图鉴扩展域 1 service 的组件划分、接口契约、核心时序、ARC-053 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-045 已确定的任何结构性选择。
- 不覆盖排行榜核心（属 leaderboard-service 既有）。
- 不覆盖客户端 UI 渲染。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：leaderboard_extra_db

leaderboard_extra_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 7 张表。

## 2.1 player_achievements（玩家成就表）

```sql
CREATE TABLE player_achievements (
    player_id          UUID NOT NULL,
    achievement_id     VARCHAR(64) NOT NULL,
    status             SMALLINT NOT NULL DEFAULT 0,  -- 0=Locked 1=InProgress 2=Unlocked 3=Rewarded
    unlocked_at_ms     BIGINT,
    rewarded_at_ms     BIGINT,
    PRIMARY KEY (player_id, achievement_id)
);
CREATE INDEX idx_player_achievements_status
    ON player_achievements (player_id, status);
    -- 支撑"玩家已解锁成就列表"高频查询
```

## 2.2 player_collections（玩家收藏册表）

```sql
CREATE TABLE player_collections (
    collection_id      VARCHAR(64) NOT NULL,
    player_id          UUID NOT NULL,
    item_def_id        VARCHAR(64) NOT NULL,
    acquired_at_ms     BIGINT NOT NULL,
    PRIMARY KEY (collection_id, player_id, item_def_id)
);
CREATE INDEX idx_player_collections_player
    ON player_collections (player_id, collection_id);
```

## 2.3 player_titles（玩家称号表）

```sql
CREATE TABLE player_titles (
    player_id          UUID NOT NULL,
    title_id           VARCHAR(64) NOT NULL,
    awarded_at_ms      BIGINT NOT NULL,
    equipped           BOOLEAN NOT NULL DEFAULT FALSE,  -- 同一 player_id 至多 1 行 equipped=TRUE
    PRIMARY KEY (player_id, title_id)
);
CREATE UNIQUE INDEX uq_player_titles_equipped
    ON player_titles (player_id) WHERE equipped = TRUE;
    -- 部分唯一索引: 同一玩家同时至多佩戴 1 个称号
```

## 2.4 achievement_progress（成就进度表）

```sql
CREATE TABLE achievement_progress (
    player_id          UUID NOT NULL,
    achievement_id     VARCHAR(64) NOT NULL,
    current_value      BIGINT NOT NULL DEFAULT 0,  -- 当前进度值
    target_value       BIGINT NOT NULL,              -- 目标值
    last_updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (player_id, achievement_id)
);
CREATE INDEX idx_achievement_progress_value
    ON achievement_progress (player_id, current_value);
```

## 2.5 achievement_configs（成就配置表）

```sql
CREATE TABLE achievement_configs (
    achievement_id     VARCHAR(64) PRIMARY KEY,
    category           SMALLINT NOT NULL,  -- 0=QUEST 1=COMBAT 2=SOCIAL 3=COLLECTION 4=ACTIVITY
    name               VARCHAR(128) NOT NULL,
    trigger_type       SMALLINT NOT NULL,  -- 0=CUMULATIVE 1=THRESHOLD
    trigger_target     VARCHAR(64) NOT NULL,
    trigger_threshold  BIGINT NOT NULL,
    reward_table       VARCHAR(64) NOT NULL,
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 2.6 collection_configs（收藏册配置表）

```sql
CREATE TABLE collection_configs (
    collection_id      VARCHAR(64) PRIMARY KEY,
    collection_type    SMALLINT NOT NULL,  -- 0=CARD 1=ITEM 2=NPC 3=EQUIPMENT
    name               VARCHAR(128) NOT NULL,
    total_items        INTEGER NOT NULL,
    milestone_rewards  JSONB NOT NULL,  -- {"25": "REWARD_TABLE_25", ...}
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 2.7 title_configs（称号配置表）

```sql
CREATE TABLE title_configs (
    title_id           VARCHAR(64) PRIMARY KEY,
    name               VARCHAR(128) NOT NULL,
    award_triggers     JSONB NOT NULL,  -- 多触发来源 (achievement / collection / event)
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

---

# 3. 关键算法详细设计

## 3.1 成就触发条件评估（CUMULATIVE / THRESHOLD）

```rust
// 落实 RGS-BAS-045 §5.1 成就解锁主流程
// 两种触发类型:
//   CUMULATIVE: 累计型 (如累计登录 30 天)
//   THRESHOLD: 阈值型 (如一次性完成 X 任务)
fn evaluate_achievement(
    player_id: PlayerId,
    achievement_id: AchievementId,
    delta: i64,
) -> Result<EvaluationResult, LbxError> {
    let config = load_achievement_config(&achievement_id)?;
    if !config.enabled {
        return Ok(EvaluationResult::NotEligible);
    }
    // 服务器权威校验 (per NFR-LBX-003, 客户端**不得**伪造)
    // 1. 更新进度
    update_progress(player_id, &achievement_id, delta)?;
    // 2. 评估触发条件
    let progress = load_progress(player_id, &achievement_id)?;
    let unlocked = match config.trigger_type {
        TriggerType::Cumulative => progress.current_value >= config.trigger_threshold,
        TriggerType::Threshold => delta >= config.trigger_threshold,
    };
    if !unlocked {
        return Ok(EvaluationResult::ProgressUpdated {
            current: progress.current_value,
            target: config.trigger_threshold,
        });
    }
    // 3. 解锁成就
    unlock_achievement_idempotent(player_id, &achievement_id)?;
    // 4. 发放奖励 (per FR-LBX-003, 经 EC 单点)
    let saga_id = generate_saga_id();
    let grant_result = commit_transaction(CommitTransactionRequest {
        request_id: deterministic_hash(&[player_id.as_bytes(), achievement_id.as_bytes()]),
        character_id: player_id.into(),
        operation: Operation::GrantItems(load_reward_table(&config.reward_table)?),
        session_epoch: current_session_epoch(player_id)?,
        expected_version: current_wallet_version(player_id)?,
    })?;
    mark_rewarded(player_id, &achievement_id)?;
    Ok(EvaluationResult::Unlocked { grant_result })
}
```

## 3.2 成就解锁幂等键（防并发重复触发，RSK-LBX-001）

```rust
// 同一玩家同一成就可能在同一时刻被多个触发源同时触发
// 解决: UNIQUE(player_id, achievement_id) 在数据库层强制
// 应用层仅需处理 DuplicateKey 错误
fn unlock_achievement_idempotent(player_id: PlayerId, achievement_id: &AchievementId) -> Result<(), LbxError> {
    match sqlx::query(
        "INSERT INTO player_achievements (player_id, achievement_id, status, unlocked_at_ms) VALUES ($1, $2, 2, $3) ON CONFLICT (player_id, achievement_id) DO NOTHING"
    )
    .bind(player_id).bind(achievement_id).bind(now_ms())
    .execute(&pool).await {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Ok(()),  // 已解锁, 幂等返回 OK
        Err(e) => Err(LbxError::Database(e)),
    }
}
```

## 3.3 收藏阶段性奖励触发（per FR-LBX-011）

```rust
// 落实 RGS-BAS-045 §5.2 收藏阶段性奖励
fn add_to_collection(
    player_id: PlayerId,
    collection_id: CollectionId,
    item_def_id: ItemDefId,
) -> Result<CollectionAddResult, LbxError> {
    let config = load_collection_config(&collection_id)?;
    // 1. 写入收藏 (幂等, 已拥有不重复)
    sqlx::query(
        "INSERT INTO player_collections (collection_id, player_id, item_def_id, acquired_at_ms) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"
    )
    .bind(&collection_id).bind(player_id).bind(&item_def_id).bind(now_ms())
    .execute(&pool).await?;
    // 2. 计算进度
    let owned_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM player_collections WHERE collection_id = $1 AND player_id = $2"
    )
    .bind(&collection_id).bind(player_id)
    .fetch_one(&pool).await?;
    let percent = (owned_count * 100) / config.total_items as i64;
    // 3. 检查阶段性奖励 (25% / 50% / 75% / 100%)
    let mut issued_rewards = vec![];
    for milestone in &[25, 50, 75, 100] {
        if percent >= *milestone {
            let reward_table = config.milestone_rewards.get(&milestone.to_string());
            if let Some(rt) = reward_table {
                // 检查是否已发放
                let already_issued = check_milestone_issued(player_id, &collection_id, *milestone)?;
                if !already_issued {
                    let grant = commit_transaction(CommitTransactionRequest {
                        request_id: deterministic_hash(&[player_id.as_bytes(), collection_id.as_bytes(), &milestone.to_le_bytes()]),
                        character_id: player_id.into(),
                        operation: Operation::GrantItems(load_reward_table(rt)?),
                        session_epoch: current_session_epoch(player_id)?,
                        expected_version: current_wallet_version(player_id)?,
                    })?;
                    mark_milestone_issued(player_id, &collection_id, *milestone)?;
                    issued_rewards.push((milestone, grant));
                }
            }
        }
    }
    Ok(CollectionAddResult { percent, issued_rewards })
}
```

## 3.4 称号佩戴（per FR-LBX-021 同时至多 1 个）

```rust
fn equip_title(player_id: PlayerId, title_id: TitleId) -> Result<(), LbxError> {
    let mut tx = pool.begin().await?;
    // 1. 校验 title_id 已获得
    let owned: Option<TitleRow> = sqlx::query_as(
        "SELECT * FROM player_titles WHERE player_id = $1 AND title_id = $2 FOR UPDATE"
    )
    .bind(player_id).bind(&title_id)
    .fetch_optional(&mut *tx).await?;
    if owned.is_none() {
        return Err(LbxError::TitleNotOwned { title_id });
    }
    // 2. 取消当前佩戴 (UQ 索引保证至多 1 行 equipped=TRUE)
    sqlx::query("UPDATE player_titles SET equipped = FALSE WHERE player_id = $1 AND equipped = TRUE")
        .bind(player_id).execute(&mut *tx).await?;
    // 3. 佩戴新称号
    sqlx::query("UPDATE player_titles SET equipped = TRUE WHERE player_id = $1 AND title_id = $2")
        .bind(player_id).bind(&title_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
```

---

# 4. 对接点

## 4.1 与 leaderboard-service 既有的对接（CON-LBX-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 排行榜核心 | 既有 | leaderboard-service 既有 | 本域**不**自建 |

## 4.2 与 EconomyService 的对接（CON-LBX-002 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 成就奖励发放 | `commit_transaction(GrantItems)` | RGS-DTL-001 §3.2 | 走 EC 既有路径 |
| 收藏阶段性奖励 | `commit_transaction(GrantItems)` | RGS-DTL-001 §3.2 | 同上 |

## 4.3 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| AchievementConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 仅订阅 config_updated |
| CollectionConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 同上 |
| TitleConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 同上 |

## 4.4 与触发源域的对接（战斗 / 任务 / 活动 / ...）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 触发成就评估 | 触发源发 `TryUnlock` | 本域 | 来自 battle / task / activity / ... |

---

# 5. 数据库 DDL 权威边界

leaderboard_extra_db 共 7 张表，以本文档为唯一权威。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| 成就解锁 + 奖励发放 | < 100ms | per NFR-LBX-001 | PH-4 |
| 收藏进度查询 | < 50ms | idx_player_collections_player | PH-2 |
| 阶段性奖励触发评估 | < 50ms | 4 个里程碑阈值检查 | PH-2 |
| 称号佩戴 | < 30ms | UQ 索引 + 同事务 | PH-2 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：
- leaderboard_extra_db 7 张表的物理 DDL
- 成就触发条件评估算法（CUMULATIVE / THRESHOLD）
- 成就解锁幂等（防并发重复触发，RSK-LBX-001）
- 收藏阶段性奖励触发（25% / 50% / 75% / 100%）
- 称号佩戴（同事务 + UQ 索引）
- 4 项对接点

本版本明确不覆盖、留待后续：
- 排行榜核心 — 属 leaderboard-service 既有
- 客户端 UI 渲染 — 客户端范畴
- 触发源的具体协议 — 来自 battle / task / activity 等域

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-045 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-045 §3 业务需求 | §2, §3 |
| RGS-REQ-045 §4 功能需求 | §3.1, §3.3, §3.4 |
| RGS-REQ-045 §5 NFR | §6 |
| RGS-REQ-045 §6 ARC-053 | §2.5, §2.6, §2.7, §3 |
| RGS-REQ-045 §7 AC-LBX-001〜004 | §3.1, §3.3, §3.4 |
| RGS-REQ-045 §8 RSK-LBX-001 | §3.2 |
| RGS-BAS-045 §3 架构总览 | §2 |
| RGS-BAS-045 §4 组件设计 | §3.1〜3.4 |
| RGS-BAS-045 §5 数据流时序 | §3.1, §3.3, §3.4 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.1, §3.3 |
