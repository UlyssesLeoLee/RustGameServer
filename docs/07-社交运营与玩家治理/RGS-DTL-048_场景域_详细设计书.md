# 详细设计书（詳細設計書 / Detailed Document）

**场景域（Scene Domain）详细设计 — SceneActor 物理结构、scene_db 物理 DDL、触发器算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-048 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-041 v0.1 场景域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/scene-service/` (scene_db, 3,823 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-049 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-041 §3〜§7 为：SceneActor 物理结构 + 场景状态机 5 阶段伪代码、scene_db 物理 DDL（`scene_instances`/`scene_entities`/`scene_triggers`/`scene_configs`/`scene_persistent_state`/`entity_ai_state`）、EnterScene 事务边界 + 触发器服务器权威校验算法（per NFR-SCN-005）+ AOI 差分快照发布伪代码（复用 ARC-002 既有） | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 触发器回调算法是否严格服务器权威；AOI 差分快照是否复用 ARC-002 既有格式 |
| 评审（DBA） | | | scene_db 索引是否覆盖高频路径（玩家当前场景查询 / AOI 可见集合计算） |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：scene_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

## 1.1 定位

RGS-BAS-041 给出了场景域 1 service 的组件划分、接口契约、核心时序、ARC-049 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-041 已确定的任何结构性选择。
- 不覆盖客户端地图渲染 / LOD。
- 不覆盖场景编辑器工具（属独立工具链）。
- 不覆盖 AOI 算法本身（属 ARC-002 既有同步层职责）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：scene_db

scene_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 6 张表。

## 2.1 scene_instances（场景实例表）

```sql
CREATE TABLE scene_instances (
    scene_instance_id  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scene_id           VARCHAR(64) NOT NULL,  -- SceneConfig 主键 (per ARC-049)
    scene_type         SMALLINT NOT NULL,  -- WORLD_CITY / DUNGEON / PVP_ROOM / BATTLEFIELD / ACTIVITY
    status             SMALLINT NOT NULL DEFAULT 0,  -- 0=Init 1=Loading 2=Running 3=Persisting 4=Unloading 5=End
    player_count       INTEGER NOT NULL DEFAULT 0,
    max_players        INTEGER NOT NULL,
    persistent_state   JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 任务进度 / NPC 死亡状态
    session_epoch      BIGINT NOT NULL,
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC 乐观锁
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at           TIMESTAMPTZ,
    CONSTRAINT chk_scene_instances_status CHECK (status BETWEEN 0 AND 5)
);
CREATE INDEX idx_scene_instances_scene_status
    ON scene_instances (scene_id, status) WHERE status IN (1, 2);
    -- 支撑"已加载场景实例查询"（玩家进入场景时定位 instance）
```

## 2.2 scene_entities（场景实体表）

```sql
CREATE TABLE scene_entities (
    entity_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scene_instance_id  UUID NOT NULL REFERENCES scene_instances(scene_instance_id) ON DELETE CASCADE,
    entity_type        SMALLINT NOT NULL,  -- 0=NPC 1=MONSTER 2=ITEM 3=DECORATION
    entity_def_id      VARCHAR(64) NOT NULL,  -- 实体定义 ID (per EntityConfig)
    position_x         REAL NOT NULL,
    position_y         REAL NOT NULL,
    position_z         REAL NOT NULL DEFAULT 0,
    attrs              JSONB NOT NULL DEFAULT '{}'::jsonb,  -- hp / mp / status / ...
    ai_state           JSONB NOT NULL DEFAULT '{}'::jsonb,  -- AI 状态 (per ARC-049 Config 驱动)
    alive              BOOLEAN NOT NULL DEFAULT TRUE,
    spawned_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    removed_at         TIMESTAMPTZ
);
CREATE INDEX idx_scene_entities_instance_type
    ON scene_entities (scene_instance_id, entity_type);
    -- 支撑"场景内实体列表查询"（AOI 差分基线）
```

## 2.3 scene_triggers（场景触发器配置表）

```sql
CREATE TABLE scene_triggers (
    trigger_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scene_id           VARCHAR(64) NOT NULL,
    trigger_type       SMALLINT NOT NULL,  -- 0=AREA 1=EVENT 2=TASK
    shape              SMALLINT,  -- 仅 AREA 触发器: 0=RECT 1=CIRCLE 2=POLYGON
    bounds             JSONB,     -- AREA: 矩形/圆/多边形参数
    action_id          VARCHAR(64),  -- 仅 EVENT 触发器
    callback           VARCHAR(128) NOT NULL,  -- 回调函数名 (per ARC-021 插件白名单)
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_scene_triggers_scene_enabled
    ON scene_triggers (scene_id, enabled) WHERE enabled = TRUE;
```

## 2.4 scene_configs（场景配置表，ARC-049 数据驱动）

```sql
CREATE TABLE scene_configs (
    scene_id           VARCHAR(64) PRIMARY KEY,
    name               VARCHAR(128) NOT NULL,
    scene_type         SMALLINT NOT NULL,
    map_id             VARCHAR(64) NOT NULL,
    max_players        INTEGER NOT NULL,
    entry_level        INTEGER NOT NULL DEFAULT 1,
    entry_task         VARCHAR(64),
    entities_def       JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 实体定义列表 (per FR-SCN-022)
    triggers_def       JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 触发器定义列表
    enabled            BOOLEAN NOT NULL DEFAULT FALSE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_scene_configs_enabled
    ON scene_configs (scene_id, enabled) WHERE enabled = TRUE;
```

## 2.5 scene_persistent_state（场景持久化状态表）

```sql
CREATE TABLE scene_persistent_state (
    scene_instance_id  UUID NOT NULL REFERENCES scene_instances(scene_instance_id),
    player_id          UUID NOT NULL,  -- 跨库引用 player_db.characters
    state_key          VARCHAR(64) NOT NULL,  -- quest_progress / buff_remaining / ...
    state_value        JSONB NOT NULL,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (scene_instance_id, player_id, state_key)
);
CREATE INDEX idx_scene_persistent_player
    ON scene_persistent_state (player_id);
    -- 支撑"玩家在场景内的持久化状态查询"（断线重连）
```

## 2.6 entity_ai_state（实体 AI 状态表）

```sql
CREATE TABLE entity_ai_state (
    entity_id          UUID PRIMARY KEY REFERENCES scene_entities(entity_id),
    ai_type            VARCHAR(32) NOT NULL,  -- IDLE / PATROL / CHASE / ATTACK / FLEE
    target_id          UUID,  -- AI 目标 (per AI type)
    last_decision_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    ai_history         JSONB NOT NULL DEFAULT '[]'::jsonb  -- 最近 N 个 AI 决策（用于调试）
);
```

---

# 3. 关键算法详细设计

## 3.1 场景生命周期 5 阶段推进伪代码

```rust
// 落实 RGS-BAS-041 §4.1 SceneLifecycleManager + ARC-001 Actor 模型
fn advance_scene_phase(scene_instance_id: SceneInstanceId, action: SceneAction) -> Result<SceneState, SceneError> {
    let mut scene = load_scene_instance(scene_instance_id)?;
    let current_phase = ScenePhase::from_u8(scene.status)?;
    match current_phase {
        ScenePhase::Init => {
            // Init → Loading
            let config = load_scene_config(&scene.scene_id)?;
            scene.max_players = config.max_players;
            scene.status = ScenePhase::Loading as i16;
        }
        ScenePhase::Loading => {
            // Loading → Running (实体初始化完成)
            spawn_initial_entities(&mut scene, &load_scene_config(&scene.scene_id)?)?;
            scene.status = ScenePhase::Running as i16;
        }
        ScenePhase::Running => {
            // 处理玩家进出 / 实体变更 / 触发器触发
            apply_scene_action(&scene, &action)?;
        }
        ScenePhase::Persisting => {
            // 持久化场景关键状态
            persist_scene_state(scene_instance_id)?;
            scene.status = ScenePhase::Unloading as i16;
        }
        ScenePhase::Unloading => {
            // 清理实体 + 释放资源
            cleanup_scene(&scene)?;
            scene.status = ScenePhase::End as i16;
            scene.ended_at = Some(now());
        }
        ScenePhase::End => {
            return Err(SceneError::PhaseInvalid { scene_instance_id, current: "End".into() });
        }
    }
    update_scene_instance(&scene, expected_version: scene.version)?;
    Ok(scene.into_state())
}
```

## 3.2 EnterScene 事务边界（落实 RGS-BAS-041 §5.1）

```rust
fn enter_scene(player_id: PlayerId, scene_id: SceneId, session_epoch: SessionEpoch) -> Result<SceneInstance, SceneError> {
    let config = load_scene_config(&scene_id)?;
    // 准入条件校验 (per FR-SCN-010)
    ensure_entry_gate(player_id, &config)?;
    // 查找已加载实例
    let scene_instance = sqlx::query_as::<_, SceneInstance>(
        "SELECT * FROM scene_instances WHERE scene_id = $1 AND status IN (1, 2) LIMIT 1"
    )
    .bind(&scene_id)
    .fetch_optional(&pool)
    .await?;
    let scene_instance = match scene_instance {
        Some(s) => s,
        None => create_scene_instance(&scene_id, &config, session_epoch).await?,
    };
    // 玩家数 + 1 (OCC)
    let mut scene_instance = scene_instance;
    if scene_instance.player_count >= scene_instance.max_players {
        return Err(SceneError::SceneFull { scene_id });
    }
    scene_instance.player_count += 1;
    update_scene_instance(&scene_instance, expected_version: scene_instance.version)?;
    // 注册玩家实体 (per FR-SCN-010)
    register_player_entity(&scene_instance, player_id)?;
    // 加载玩家持久化状态 (per FR-SCN-013)
    load_player_persistent_state(scene_instance.scene_instance_id, player_id)?;
    Ok(scene_instance)
}
```

## 3.3 触发器服务器权威校验（per NFR-SCN-005 落实）

```rust
// 触发器回调**严格**服务器权威执行, 客户端**不得**直接触发
// 落实 RGS-BAS-041 §4.3 TriggerSystem
fn fire_trigger(
    scene_instance_id: SceneInstanceId,
    player_id: PlayerId,
    trigger_type: TriggerType,
    payload: TriggerPayload,
) -> Result<TriggerResult, SceneError> {
    // 1. 加载场景触发器配置
    let triggers = load_scene_triggers(scene_instance_id, trigger_type)?;
    // 2. 服务器权威校验触发条件
    let matched: Vec<&SceneTrigger> = triggers.iter()
        .filter(|t| match trigger_type {
            TriggerType::Area => check_area_trigger(t, &payload.position)?,
            TriggerType::Event => check_event_trigger(t, &payload.action_id, &payload.params)?,
            TriggerType::Task => return delegate_to_gsm(t, player_id, payload),
        })
        .collect();
    // 3. 执行回调 (per ARC-021 插件白名单)
    for trigger in matched {
        // 关键: 触发器回调**不得**接受客户端直接请求
        // 必须从服务器侧上下文 (动作结果 / 位置变更) 触发
        execute_trigger_callback(trigger, scene_instance_id, player_id, &payload)?;
    }
    Ok(TriggerResult { fired: matched.len() })
}
```

## 3.4 AOI 差分快照发布（复用 ARC-002 既有）

```rust
// 落实 RGS-BAS-041 §5.2 实体变更 → AOI 广播
// 严格复用 ARC-002 既有差分快照格式, 不另建同步算法
fn publish_entity_delta(
    scene_instance_id: SceneInstanceId,
    entity_id: EntityId,
    delta: EntityDelta,
) -> Result<(), SceneError> {
    let scene = load_scene_instance(scene_instance_id)?;
    // 计算 AOI 可见玩家集合 (per ARC-002 既有 AOI 算法)
    let visible_players = compute_aoi_visible_set(scene_instance_id, &delta.position)?;
    // 构造差分快照 (ARC-002 既有格式)
    let snapshot = AoiDelta {
        entity_id,
        position: delta.position,
        attrs_delta: delta.attrs,
        timestamp_ms: now_ms(),
    };
    // 广播至可见玩家
    broadcast_to_players(&visible_players, &snapshot)?;
    Ok(())
}
```

## 3.5 场景卸载策略（无玩家 + 无持久任务）

```rust
// 落实 RGS-BAS-041 §4.1 SceneUnloadPolicy
fn should_unload(scene_instance_id: SceneInstanceId) -> Result<bool, SceneError> {
    let scene = load_scene_instance(scene_instance_id)?;
    // 条件 1: 无玩家
    if scene.player_count > 0 {
        return Ok(false);
    }
    // 条件 2: 无持久任务 (per scene_persistent_state 表)
    let persistent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scene_persistent_state WHERE scene_instance_id = $1"
    )
    .bind(scene_instance_id)
    .fetch_one(&pool)
    .await?;
    Ok(persistent_count == 0)
}
```

---

# 4. 对接点

## 4.1 与 Runtime（既有）的对接（FR-RT-008 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 玩家进入场景 | RT 触发 `EnterScene` | RGS-BAS-013 §2.2 | scene 被动接收 |
| 玩家离开场景 | RT 触发 `LeaveScene` | RGS-BAS-013 §2.2 | scene 被动接收 |

## 4.2 与 GSM 任务系统的对接（CON-SCN-006 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 任务触发 | `CheckTaskProgress` | RGS-BAS-014 §3 | scene **不**自建任务 |

## 4.3 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| SceneConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | scene 仅订阅 config_updated |
| 配置版本切换 | OCC `version` | RGS-DTL-001 §3.2 | 灰度发布安全切换 |

## 4.4 与 MatchService 的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| PVP 场景进入 | MT 匹配成功 → RT 触发 `EnterScene` | per RGS-REQ-029 | 由 RT 侧主导 |

---

# 5. 数据库 DDL 权威边界

scene_db 共 6 张表，以本文档为唯一权威。跨域表结构扩展须按 RGS-BAS-016 §3.1 回写原表权威文档。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| 场景加载 | < 500ms | per NFR-SCN-001 | PH-4 |
| EnterScene 准入校验 | < 20ms | 高频路径 | PH-2 |
| 触发器匹配 (per AOI) | < 10ms | 玩家移动高频触发 | PH-2 |
| AOI 差分快照发布 | < 50ms | per ARC-002 既有 | PH-4 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：

- scene_db 6 张表的物理 DDL
- 场景生命周期 5 阶段推进伪代码
- EnterScene 事务边界（含准入校验 + 玩家数 OCC）
- 触发器服务器权威校验算法（NFR-SCN-005）
- AOI 差分快照发布伪代码（复用 ARC-002）
- 场景卸载策略
- 4 项对接点（RT / GSM / ARC-021 / MT）

本版本明确不覆盖、留待后续：

- AOI 算法本身 — 属 ARC-002 既有同步层
- 实体 AI 决策算法 — 业务层实现（AI 类型仅由 ai_type 字段驱动）
- 客户端地图渲染 / LOD — 客户端范畴
- 场景编辑器工具 — 独立工具链
- TBD-SCN-001/002（断线重连时长 / 单场景最大玩家数）— 留待策划确认

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-041 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-041 §3 业务需求 | §2, §3 |
| RGS-REQ-041 §4 功能需求 | §3.1, §3.2, §3.3 |
| RGS-REQ-041 §5 NFR | §6 |
| RGS-REQ-041 §6 ARC-049 | §2.4, §3.1, §3.3 |
| RGS-REQ-041 §7 AC-SCN-001〜005 | §3.1〜3.5 |
| RGS-REQ-041 §8 RSK-SCN-002 | §3.3 |
| RGS-BAS-041 §3 架构总览 | §2 |
| RGS-BAS-041 §4 组件设计 | §3.1〜3.5 |
| RGS-BAS-041 §5 数据流时序 | §3.1, §3.4 |
| RGS-BAS-041 §6 接口契约 | §2.1〜2.6 |
| RGS-BAS-041 §7 ARC-049 落实 | §2.4, §3.1 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.1, §3.2 |
