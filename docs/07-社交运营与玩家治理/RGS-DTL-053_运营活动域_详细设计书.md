# 详细设计书（詳細設計書 / Detailed Document）

**运营活动域（Operate Domain）详细设计 — 活动日历 / 订阅物理 DDL + 跨域协调算法**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-053 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-046 v0.1 运营活动域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/operate-service/` (operate_db, 384 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-054 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-046 §3〜§7 为：operate_db 物理 DDL（`calendar_entries`/`activity_subscriptions`/`cross_domain_activities`/`calendar_configs`）、活动日历聚合查询算法（跨域并行）、订阅推送委托路径（per RGS-REQ-008）、跨域活动元数据一致性维护（RSK-OPR-001） | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 跨域活动元数据一致性维护是否完整；委托路径是否严格复用既有 |
| 评审（DBA） | | | operate_db 索引是否覆盖高频路径（日历查询 / 订阅查询） |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：operate_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

RGS-BAS-046 给出了运营活动域 1 service 的组件划分、接口契约、核心时序、ARC-054 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-046 已确定的任何结构性选择。
- 不覆盖具体活动战斗逻辑（属 battle 既有）。
- 不覆盖推送通道实现（属 RGS-REQ-008 既有）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：operate_db

operate_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 4 张表。

## 2.1 calendar_entries（活动日历表）

```sql
CREATE TABLE calendar_entries (
    activity_id        VARCHAR(64) PRIMARY KEY,
    name               VARCHAR(128) NOT NULL,
    activity_type      SMALLINT NOT NULL,  -- 0=HOLIDAY 1=OPENING 2=LIMITED 3=CROSS_DOMAIN
    start_at_ms        BIGINT NOT NULL,
    end_at_ms          BIGINT NOT NULL,
    dispatcher_target  VARCHAR(64) NOT NULL,  -- battle_service / activity_service / multi_domain
    cross_domain       BOOLEAN NOT NULL DEFAULT FALSE,
    notify_on_start    BOOLEAN NOT NULL DEFAULT TRUE,
    notify_on_end      BOOLEAN NOT NULL DEFAULT FALSE,
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_calendar_type CHECK (activity_type BETWEEN 0 AND 3)
);
CREATE INDEX idx_calendar_entries_time_range
    ON calendar_entries (start_at_ms, end_at_ms) WHERE enabled = TRUE;
    -- 支撑"时间范围过滤"高频查询 (per FR-OPR-001)
```

## 2.2 activity_subscriptions（活动订阅表）

```sql
CREATE TABLE activity_subscriptions (
    subscription_id    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id          UUID NOT NULL,
    activity_id        VARCHAR(64) NOT NULL,
    notify_enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    subscribed_at_ms   BIGINT NOT NULL,
    last_notified_ms   BIGINT,
    UNIQUE (player_id, activity_id)
);
CREATE INDEX idx_activity_subs_player
    ON activity_subscriptions (player_id, notify_enabled) WHERE notify_enabled = TRUE;
```

## 2.3 cross_domain_activities（跨域活动元数据表）

```sql
CREATE TABLE cross_domain_activities (
    activity_id        VARCHAR(64) PRIMARY KEY,
    participating_domains JSONB NOT NULL,  -- ["battle", "task", "collection", ...]
    coordination_rules JSONB NOT NULL,     -- 跨域协调规则
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 2.4 calendar_configs（日历配置表）

```sql
CREATE TABLE calendar_configs (
    activity_id        VARCHAR(64) PRIMARY KEY,
    config_json        JSONB NOT NULL,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

---

# 3. 关键算法详细设计

## 3.1 活动日历聚合查询（跨域并行）

```rust
// 落实 RGS-BAS-046 §5.1 活动日历聚合
// 跨域并行查询 (per NFR-OPR-001 性能)
async fn get_calendar(
    start_ms: i64,
    end_ms: i64,
    types: &[ActivityType],
) -> Result<Vec<CalendarEntry>, OprError> {
    // 1. 从本域读 calendar_entries (时间范围 + 类型过滤)
    let local_entries = sqlx::query_as::<_, CalendarEntry>(
        "SELECT * FROM calendar_entries WHERE start_at_ms <= $1 AND end_at_ms >= $2 AND activity_type = ANY($3) AND enabled = TRUE ORDER BY start_at_ms"
    )
    .bind(end_ms).bind(start_ms).bind(types)
    .fetch_all(&pool).await?;
    // 2. 跨域并行查询 battle / activity (per ARC-054 复用既有)
    let (battle_entries, activity_entries) = tokio::try_join!(
        call_battle_get_holiday_activities(start_ms, end_ms, types),
        call_activity_get_activities(start_ms, end_ms, types),
    )?;
    // 3. 合并 + 去重 (按 activity_id)
    let mut all_entries = local_entries;
    all_entries.extend(battle_entries);
    all_entries.extend(activity_entries);
    all_entries.sort_by_key(|e| e.start_at_ms);
    all_entries.dedup_by(|a, b| a.activity_id == b.activity_id);
    Ok(all_entries)
}
```

## 3.2 活动订阅推送（委托既有推送通道）

```rust
// 落实 RGS-BAS-046 §5.3 活动订阅推送
// 推送**必须**走 RGS-REQ-008 既有推送通道, 不自建
fn push_activity_progress(
    activity_id: ActivityId,
    progress_event: ActivityProgressEvent,
) -> Result<PushResult, OprError> {
    // 1. 查找订阅玩家
    let subscriptions = sqlx::query_as::<_, ActivitySubscription>(
        "SELECT * FROM activity_subscriptions WHERE activity_id = $1 AND notify_enabled = TRUE"
    )
    .bind(&activity_id).fetch_all(&pool).await?;
    // 2. 委托给 Notification Service (per RGS-REQ-008 既有)
    let mut push_results = vec![];
    for sub in subscriptions {
        let result = call_notification_push(PushRequest {
            player_id: sub.player_id,
            channel: NotificationChannel::InGame,  // 站内推送
            title: format!("活动进度更新: {}", progress_event.activity_name),
            content: progress_event.summary.clone(),
            metadata: json!({"activity_id": activity_id, "subscription_id": sub.subscription_id}),
        })?;
        push_results.push(result);
        // 更新 last_notified_ms
        sqlx::query("UPDATE activity_subscriptions SET last_notified_ms = $1 WHERE subscription_id = $2")
            .bind(now_ms()).bind(sub.subscription_id)
            .execute(&pool).await?;
    }
    Ok(PushResult { delivered: push_results.len() })
}
```

## 3.3 跨域活动元数据一致性维护（RSK-OPR-001 关键路径）

```rust
// 跨域活动元数据必须在多个域间保持一致
// 本域仅维护元数据, 具体战斗/任务逻辑由各域承担
// 解决: 启动时 + 定期拉取各域活动配置, 校验一致性
async fn sync_cross_domain_metadata(activity_id: ActivityId) -> Result<SyncResult, OprError> {
    let local_meta = load_cross_domain_metadata(activity_id)?;
    // 1. 跨域并行拉取
    let (battle_meta, task_meta, collection_meta) = tokio::try_join!(
        call_battle_get_activity_metadata(activity_id),
        call_task_get_activity_metadata(activity_id),
        call_collection_get_activity_metadata(activity_id),
    )?;
    // 2. 校验 participating_domains 与各域实际配置一致
    let domains_from_local: Vec<String> = local_meta.participating_domains.as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let mut missing_domains = vec![];
    if !battle_meta.is_null() && !domains_from_local.contains(&"battle".to_string()) {
        missing_domains.push("battle");
    }
    if !task_meta.is_null() && !domains_from_local.contains(&"task".to_string()) {
        missing_domains.push("task");
    }
    if !collection_meta.is_null() && !domains_from_local.contains(&"collection".to_string()) {
        missing_domains.push("collection");
    }
    // 3. 不一致时告警 + 自动修复 (per NFR-OPR-003 埋点)
    if !missing_domains.is_empty() {
        log::error!("cross_domain activity {} inconsistent: missing {:?}", activity_id, missing_domains);
        // 自动修复: 添加到 participating_domains
        for domain in &missing_domains {
            append_to_participating_domains(activity_id, domain)?;
        }
    }
    Ok(SyncResult { inconsistencies_fixed: missing_domains.len() })
}
```

## 3.4 进入活动（委托路径）

```rust
// 落实 RGS-BAS-046 §5.2 玩家进入活动
// 委托给对应域, 本域**不**实现具体战斗/任务逻辑
fn enter_activity(player_id: PlayerId, activity_id: ActivityId) -> Result<EnterResult, OprError> {
    let entry = load_calendar_entry(&activity_id)?;
    // 1. 校验时间窗口
    let now = now_ms();
    if now < entry.start_at_ms || now > entry.end_at_ms {
        return Err(OprError::ActivityOutOfWindow { activity_id });
    }
    // 2. 委托给 dispatcher_target
    let result = match entry.dispatcher_target.as_str() {
        "battle_service" => call_battle_enter_holiday_activity(player_id, activity_id)?,
        "activity_service" => call_activity_enter(player_id, activity_id)?,
        "multi_domain" => call_cross_domain_enter(player_id, activity_id)?,
        _ => return Err(OprError::DispatcherNotRegistered { target: entry.dispatcher_target.clone() }),
    };
    // 3. 埋点 (per NFR-OPR-003)
    emit_metric("operate.activity_entered", json!({"activity_id": activity_id, "player_id": player_id}));
    Ok(result)
}
```

---

# 4. 对接点

## 4.1 与 battle-service HolidayActivityService 的对接（CON-OPR-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 活动日历聚合 | `GetHolidayActivities` | battle 既有 | 本域**不**自建 |
| 进入 holiday_* 活动 | `EnterHolidayActivity` | battle 既有 | per ARC-046 |

## 4.2 与 activity-service 既有的对接（CON-OPR-002 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 活动日历聚合 | `GetActivities` | activity 既有 | 本域**不**自建 |
| 进入 activity | `EnterActivity` | activity 既有 | 严格复用 |

## 4.3 与 EconomyService 的对接（CON-OPR-003 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 活动奖励发放 | `commit_transaction(GrantItems)` | RGS-DTL-001 §3.2 | 走 EC 既有路径 |

## 4.4 与 RGS-REQ-008 推送通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 活动进度推送 | `Push` | RGS-REQ-008 既有 | 本域**不**自建推送 |

## 4.5 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| CalendarConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 仅订阅 config_updated |

---

# 5. 数据库 DDL 权威边界

operate_db 共 4 张表，以本文档为唯一权威。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| 活动日历查询 | < 100ms | idx_calendar_entries_time_range + 跨域并行 | PH-4 |
| 活动订阅推送 | < 200ms | per NFR-OPR-001 | PH-4 |
| 跨域元数据一致性检查 | < 5s | 启动时 + 定期 1h 一次 | PH-6 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：
- operate_db 4 张表的物理 DDL
- 活动日历聚合查询算法（跨域并行）
- 活动订阅推送（委托既有推送通道）
- 跨域活动元数据一致性维护（RSK-OPR-001）
- 进入活动委托路径
- 5 项对接点

本版本明确不覆盖、留待后续：
- 具体活动战斗逻辑 — 属 battle 既有
- 推送通道实现 — 属 RGS-REQ-008 既有
- 活动规则引擎 — 属 activity-service 既有

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-046 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-046 §3 业务需求 | §2, §3 |
| RGS-REQ-046 §4 功能需求 | §3.1, §3.2, §3.4 |
| RGS-REQ-046 §5 NFR | §6 |
| RGS-REQ-046 §6 ARC-054 | §2.1, §3 |
| RGS-REQ-046 §7 AC-OPR-001〜004 | §3.1, §3.2, §3.4 |
| RGS-REQ-046 §8 RSK-OPR-001 | §3.3 |
| RGS-BAS-046 §3 架构总览 | §2 |
| RGS-BAS-046 §4 组件设计 | §3.1〜3.4 |
| RGS-BAS-046 §5 数据流时序 | §3.1, §3.2, §3.3 |
