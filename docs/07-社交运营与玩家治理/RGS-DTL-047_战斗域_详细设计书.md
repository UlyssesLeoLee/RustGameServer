# 详细设计书（詳細設計書 / Detailed Document）

**战斗域（Battle Domain）详细设计 — BattleActor 物理结构、PvPConfig/ActivityConfig 数据库 DDL、战斗结算确定请求算法、Saga 补偿路径**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-047 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-040 v0.1 战斗域 基本设计书（本文档为其物理/实现级细化，不改变任何既有决定，仅将逻辑设计落实为物理 DDL / 算法伪代码） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/battle-service/` (battle_db, 2,875 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-046 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 issue thread 9/4 MD §2 审计识别）。细化 RGS-BAS-040 §3〜§7 为：BattleActor 物理结构 + 战斗状态机 5 阶段伪代码、battle_db 物理 DDL（`battle_actors`/`battle_actions`/`pvp_configs`/`activity_configs`/`battle_results`/`battle_replays`/`guild_war_participants`）、SubmitAction 事务边界 + 断线重连幂等键构造 + 战斗结算确定请求事务边界（复用 RGS-DTL-001 §3.2 OCC + 幂等）、战斗退出/中止幂等性验证（RSK-BAT-002 关键路径） | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 战斗状态机伪代码是否与 ARC-001 Actor 模型对齐；SubmitAction 幂等键构造是否真正防重线重连双发 |
| 评审（DBA） | | | battle_db 表结构是否覆盖 ARC-046 数据驱动配置；pvp_configs / activity_configs 索引是否支撑高频变体选路 |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：battle_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

## 1.1 定位

RGS-BAS-040 给出了战斗域 12 service 的组件划分、接口契约、核心时序、ARC-046 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-040 已确定的任何结构性选择（12 service 划分、ARC-046 反例原则、EC 单点结算、battle_db 独立）。
- 不覆盖客户端 UI 渲染 / 战斗客户端预测算法（属 ARC-002 既有同步层）。
- 不覆盖战斗录像回放分享（属 replay-extra-service 范畴）。
- 不覆盖战斗客户端 SDK 协议层（属 RGS-BAS-008 既有范围）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式（OCC 乐观锁 + request_id 唯一约束）作为本域所有写入路径的物理执行语义。

---

# 2. 物理数据库设计：battle_db

RGS-BAS-040 §3 架构图已明确 battle_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 7 张表。

## 2.1 battle_actors（战斗 Actor 实例表）

```sql
CREATE TABLE battle_actors (
    battle_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    battle_mode        SMALLINT NOT NULL,  -- BattleMode 枚举, 与 RGS-REQ-040 §2 一致
    battle_phase       SMALLINT NOT NULL DEFAULT 0,  -- 0=Init 1=Preparation 2=Combat 3=Settlement 4=End
    battle_config_id   VARCHAR(64) NOT NULL,  -- PvPConfig / ActivityConfig / InstanceConfig 主键 (per ARC-046)
    -- 跨域引用: 创建者 player_id 引用 player_db.characters, 跨库不建物理 FK (per RGS-DTL-001 §2 既定跨库约束)
    creator_player_id  UUID NOT NULL,
    participants_json  JSONB NOT NULL,        -- 参战玩家列表 (按 BattleMode 不同, 1〜N 人)
    snapshot_json      JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 当前战斗状态快照 (ARC-002 差分基线)
    session_epoch      BIGINT NOT NULL,       -- 会话 epoch (per BAS-001 §3 既有, 断线重连校验)
    status             SMALLINT NOT NULL DEFAULT 0,  -- 0=活跃 1=已结算 2=已中止 3=超时
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC 乐观锁 (per RGS-DTL-001 §3.2)
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at           TIMESTAMPTZ,           -- Settlement / End 阶段写入
    CONSTRAINT chk_battle_actors_phase CHECK (battle_phase BETWEEN 0 AND 4),
    CONSTRAINT chk_battle_actors_status CHECK (status BETWEEN 0 AND 3)
);
CREATE INDEX idx_battle_actors_creator_status
    ON battle_actors (creator_player_id, status) WHERE status = 0;
    -- 支撑"玩家当前活跃战斗查询"高频路径 (大厅回连/断线重连)
```

## 2.2 battle_actions（战斗动作流水表）

```sql
CREATE TABLE battle_actions (
    action_id          BIGSERIAL PRIMARY KEY,
    battle_id          UUID NOT NULL REFERENCES battle_actors(battle_id),
    actor_player_id    UUID NOT NULL,
    turn_index         INTEGER NOT NULL,
    action_type        VARCHAR(32) NOT NULL,  -- use_skill / attack / defend / flee / end_turn / ...
    action_payload     JSONB NOT NULL DEFAULT '{}'::jsonb,
    request_id         VARCHAR(64) NOT NULL,  -- 幂等键 (per NFR-BAT-004 断线重连防双发)
    occurred_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (battle_id, request_id)            -- 同一战斗同一 request_id 至多一次, 防重线重连双发
);
CREATE INDEX idx_battle_actions_battle_time
    ON battle_actions (battle_id, occurred_at);
    -- 支撑战斗回放 (属 replay-extra-service 既有职责, 本表仅持久化)
```

## 2.3 pvp_configs（PVP 变体配置表）

```sql
CREATE TABLE pvp_configs (
    config_id          VARCHAR(64) PRIMARY KEY,
    pvp_mode           SMALLINT NOT NULL,  -- 0=Ranked 1=Casual 2=CrossServer 3=Arena 4=Tournament 5=Custom
    match_pool         VARCHAR(64) NOT NULL,
    rating_algorithm   VARCHAR(32) NOT NULL,  -- GLICKO2 / NONE / ...
    reward_table       VARCHAR(64) NOT NULL,
    snapshot_interval_ms  INTEGER NOT NULL DEFAULT 50,
    max_turn_count     INTEGER NOT NULL DEFAULT 100,
    tie_breaker        SMALLINT NOT NULL,  -- 0=RANDOM 1=HIGHEST_RATING 2=HIGHEST_HP 3=HIGHEST_DAMAGE
    privacy_filter     SMALLINT NOT NULL DEFAULT 0,  -- 0=NONE 1=STRICT (per NFR-BAT-006 跨服 PVP)
    enabled            BOOLEAN NOT NULL DEFAULT TRUE,  -- 灰度开关 (per ARC-021 插件)
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC, 配置热更新原子切换
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_pvp_configs_mode_enabled
    ON pvp_configs (pvp_mode, enabled) WHERE enabled = TRUE;
    -- 支撑"按变体选路"高频查询 (per ARC-046 数据驱动核心路径)
```

## 2.4 activity_configs（节日/远征配置表）

```sql
CREATE TABLE activity_configs (
    activity_id        VARCHAR(64) PRIMARY KEY,
    name               VARCHAR(128) NOT NULL,
    battle_mode        SMALLINT NOT NULL,  -- ACTIVITY_PVE / ACTIVITY_PVP / EXPEDITION
    battle_config_id   VARCHAR(64) NOT NULL,
    reward_table       VARCHAR(64) NOT NULL,
    entry_level        INTEGER NOT NULL DEFAULT 1,
    duration_days      INTEGER NOT NULL DEFAULT 7,
    max_entries_per_player INTEGER NOT NULL DEFAULT 1,
    cross_server       BOOLEAN NOT NULL DEFAULT FALSE,
    enabled            BOOLEAN NOT NULL DEFAULT FALSE,  -- 默认未启用 (策划提交后开启)
    version            INTEGER NOT NULL DEFAULT 0,
    start_at           TIMESTAMPTZ,         -- 活动开始时间 (NULL=长期)
    end_at             TIMESTAMPTZ,         -- 活动结束时间 (NULL=长期)
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_activity_configs_enabled_window
    ON activity_configs (enabled, start_at, end_at) WHERE enabled = TRUE;
    -- 支撑"当前可参与活动查询"
```

## 2.5 battle_results（战斗结算结果表）

```sql
CREATE TABLE battle_results (
    battle_id          UUID PRIMARY KEY REFERENCES battle_actors(battle_id),
    outcome            SMALLINT NOT NULL,  -- 0=Win 1=Lose 2=Draw 3=Aborted 4=Timeout
    stars              SMALLINT NOT NULL DEFAULT 0,  -- 0-3 星, PVE 副本常用
    rewards_json       JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 战斗结算奖励 (item_id, count)
    summary_json       JSONB NOT NULL DEFAULT '{}'::jsonb,  -- 战斗摘要 (总回合 / 总伤害 / MVP 等)
    saga_id            UUID,                -- 关联 Saga (per ARC-011 单调解者), 战斗结算走 EC 时生成
    settled_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_battle_results_outcome CHECK (outcome BETWEEN 0 AND 4),
    CONSTRAINT chk_battle_results_stars CHECK (stars BETWEEN 0 AND 3)
);
```

## 2.6 battle_replays（战斗录像元数据表）

```sql
CREATE TABLE battle_replays (
    replay_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    battle_id          UUID NOT NULL REFERENCES battle_actors(battle_id),
    storage_uri        TEXT NOT NULL,        -- 录像文件存储 URI (S3/MinIO)
    size_bytes         BIGINT NOT NULL,
    format             VARCHAR(16) NOT NULL DEFAULT 'rgs-v1',
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at         TIMESTAMPTZ           -- 过期时间 (天梯 90d / 休闲 7d / 房间 自定义)
);
CREATE INDEX idx_battle_replays_battle
    ON battle_replays (battle_id);
    -- 本表仅落位元数据, 录像内容由 replay-extra-service 既有职责处理
```

## 2.7 guild_war_participants（公会战参战表）

```sql
CREATE TABLE guild_war_participants (
    guild_war_id       UUID NOT NULL,
    guild_id           UUID NOT NULL,
    player_id          UUID NOT NULL,
    side               SMALLINT NOT NULL,  -- 0=攻 1=守
    damage_contributed BIGINT NOT NULL DEFAULT 0,
    final_rank         INTEGER,             -- 战内排名 (NULL=未结算)
    joined_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_war_id, player_id),
    CONSTRAINT chk_guild_war_side CHECK (side IN (0, 1))
);
CREATE INDEX idx_guild_war_participants_guild
    ON guild_war_participants (guild_id, guild_war_id);
```

---

# 3. 关键算法详细设计

## 3.1 战斗状态机 5 阶段推进伪代码

```rust
// 落实 RGS-BAS-040 §4.1 BattleStateMachine + RGS-REQ-040 §2 BattlePhase
fn advance_battle_phase(battle_id: BattleId, action: BattleAction) -> Result<BattleState, BattleError> {
    let mut battle = load_battle_actor(battle_id)?;  // SELECT ... FOR UPDATE, OCC 起点
    let current_phase = BattlePhase::from_u8(battle.battle_phase)?;
    match current_phase {
        BattlePhase::Init => {
            // Init → Preparation
            battle.snapshot_json = build_initial_snapshot(&battle)?;
            battle.battle_phase = BattlePhase::Preparation as i16;
        }
        BattlePhase::Preparation => {
            // Preparation → Combat (玩家 ready 信号全部到齐)
            ensure_all_participants_ready(&battle)?;
            battle.battle_phase = BattlePhase::Combat as i16;
        }
        BattlePhase::Combat => {
            // 验证 action (服务器权威, per NFR-BAT-003)
            ActionValidator::validate(&battle, &action)?;
            // 幂等写入 (per NFR-BAT-004 断线重连防双发)
            insert_battle_action(battle_id, &action, request_id)?;
            // 状态推进 + TurnScheduler
            battle.snapshot_json = apply_action_and_tick(&battle, &action)?;
            // 判定是否结束
            if is_battle_finished(&battle) {
                battle.battle_phase = BattlePhase::Settlement as i16;
            }
        }
        BattlePhase::Settlement => {
            // 结算 (走 EC 单点, per §3.3)
            settle_battle(battle_id)?;
            battle.battle_phase = BattlePhase::End as i16;
            battle.ended_at = Some(now());
            battle.status = BattleStatus::Settled as i16;
        }
        BattlePhase::End => {
            return Err(BattleError::PhaseInvalid { battle_id, current: "End".into() });
        }
    }
    // OCC 提交 (per RGS-DTL-001 §3.2)
    update_battle_actor(&battle, expected_version: battle.version)?;
    Ok(battle.into_state())
}
```

## 3.2 SubmitAction 幂等键构造（NFR-BAT-004 断线重连防双发）

```rust
// request_id = hash(battle_id + player_id + action_type + turn_index + payload_hash)
// 同一玩家同一战斗同一回合同一动作类型的请求恒定映射到同一 request_id
// battle_actions UNIQUE(battle_id, request_id) 在数据库层强制防双发
fn build_submit_action_request_id(
    battle_id: BattleId,
    player_id: PlayerId,
    action: &BattleAction,
) -> String {
    let payload_hash = sha256(&action.payload_json);
    sha256(&format!(
        "{}/{}/{}/{}/{}",
        battle_id, player_id, action.action_type, action.turn_index, payload_hash
    ))
}
```

## 3.3 战斗结算确定请求事务边界（落实 RGS-BAS-040 §5.1）

```rust
// 战斗结算 = 一个 Saga 子流程, 走 RGS-DTL-001 §3.2 既有 OCC + 幂等模式
// 不新建 Saga 类型, 复用 EC 既有 commit_transaction 接口
fn settle_battle(battle_id: BattleId) -> Result<GrantResult, BattleError> {
    let battle = load_battle_actor(battle_id)?;
    let outcome = determine_outcome(&battle);  // Win/Lose/Draw/Timeout
    let rewards = RewardCalculator::calculate(&battle, outcome)?;  // 按 battle_mode 不同规则
    // 战斗结果落位 (本域 DB)
    insert_battle_result(battle_id, outcome, &rewards, saga_id: None)?;
    // EC 单点 (per FR-GOV-001): 货币/道具发放必须经 EC
    let saga_id = generate_saga_id();
    let grant_result = commit_transaction(CommitTransactionRequest {
        request_id: deterministic_hash(&[battle_id.as_bytes(), b"settle"]),  // 战斗结算幂等键
        character_id: battle.creator_player_id.into(),
        operation: Operation::GrantItems(rewards.clone()),  // 道具发放
        session_epoch: battle.session_epoch,
        expected_version: current_wallet_version(battle.creator_player_id)?,
    })?;
    // Saga 关联 (便于补偿审计)
    link_saga_to_battle(battle_id, saga_id)?;
    Ok(grant_result)
}
```

## 3.4 战斗退出/中止幂等性（RSK-BAT-002 关键路径）

```rust
// 战斗退出 / 中止路径, 必须保证:
//   1. 玩家主动退出 (FR-RT-008 既有场景间转移)
//   2. 玩家断线 (NFR-BAT-004)
//   3. 战斗超时 (Settlement 阶段)
//   4. GM 中止 (per RGS-REQ-007 GM 工具)
//   四路并发触发, 只能产生一次"战斗结束"事件 + 至多一次 EC 发放
fn abort_or_exit_battle(battle_id: BattleId, reason: AbortReason, player_id: Option<PlayerId>) -> Result<(), BattleError> {
    let mut battle = load_battle_actor(battle_id)?;
    if battle.status != BattleStatus::Active as i16 {
        // 已结算/已中止, 幂等返回 OK
        return Ok(());
    }
    battle.status = BattleStatus::Aborted as i16;
    battle.battle_phase = BattlePhase::End as i16;
    battle.ended_at = Some(now());
    // 中止落位 (走 battle_results, outcome = Aborted)
    insert_battle_result_idempotent(battle_id, outcome: Aborted, rewards: &[], reason)?;
    // 结算时**不**触发 EC 发放 (per FR-GOV-001: Aborted 不发放奖励)
    // OCC 提交
    update_battle_actor(&battle, expected_version: battle.version)?;
    Ok(())
}
```

## 3.5 跨服 PVP 数据脱敏（NFR-BAT-006 落实）

```rust
// 跨服 PVP 时, 对方玩家的精确位置/个人信息**不得**泄露
// 落实 RGS-BAS-040 §4.10 CrossServerService "仅承担数据脱敏"
fn filter_cross_server_visibility(self_player: PlayerSnapshot, opponent: PlayerSnapshot, privacy: PrivacyFilter) -> PlayerSnapshot {
    match privacy {
        PrivacyFilter::None => opponent,
        PrivacyFilter::Strict => PlayerSnapshot {
            // 仅暴露: 玩家 ID (用于显示) + 等级 + 段位
            player_id: opponent.player_id,
            level: opponent.level,
            rank: opponent.rank,
            // 隐藏: 装备 / 道具 / 精确战力 / 位置
            equipment: None,
            inventory_summary: None,
            precise_power: None,
            precise_location: None,
        }
    }
}
```

## 3.6 PVP 变体选路（ARC-046 数据驱动核心路径）

```rust
// PvPService 收到 StartPvp 请求 → 根据 pvp_mode 选 PvPConfig → 复用 BattleEngineService
fn dispatch_pvp_variant(pvp_mode: PvpMode) -> Result<PvpConfig, BattleError> {
    // 高频查询, idx_pvp_configs_mode_enabled 覆盖
    let config = sqlx::query_as::<_, PvpConfig>(
        "SELECT * FROM pvp_configs WHERE pvp_mode = $1 AND enabled = TRUE LIMIT 1"
    )
    .bind(pvp_mode as i16)
    .fetch_one(&pool)
    .await?;
    if config.max_turn_count <= 0 {
        return Err(BattleError::ConfigInvalid { config_id: config.config_id });
    }
    Ok(config)
}
```

---

# 4. 对接点

## 4.1 与 EconomyService 的对接（FR-GOV-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 战斗结算 Settlement 阶段 | `commit_transaction(GrantItems)` | RGS-DTL-001 §3.2 | 走 EC 既有路径, 不在 battle 内直接调 |
| 战斗结算成功 / 失败 | Saga 事件订阅 | RGS-DTL-100 Saga 既有 | 失败 → 补偿 (战斗结果回退 + 重发) |

## 4.2 与 MatchService 的对接（CON-BAT-007 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| PVP 玩家入队 | 既有 MT 匹配 | RGS-BAS-026 既有 | battle **不**自建匹配 |
| 匹配成功 → 进入战斗 | MT 返回 `match_ticket` → battle `InitBattle` | per RGS-REQ-029 | 由 MT 侧主导, battle 被动接收 |

## 4.3 与 LeaderboardService 的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 战斗结算 → 排行榜更新 | `PublishOutcome` | RGS-BAS-014 §4 派生视图既有 | 段位 / 积分 / 胜率 |
| 战斗结算 → 任务/成就触发 | `TriggerTask` | RGS-BAS-014 §3 任务配置化引擎 | 由 GSM 侧扫描, battle 仅发事件 |

## 4.4 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| PvPConfig / ActivityConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 插件注册表既有 | battle 仅订阅 config_updated 事件 |
| 配置版本切换 | OCC `version` 字段 | RGS-DTL-001 §3.2 | 灰度发布安全切换 |

## 4.5 与 SceneService 的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 战斗场景从大厅转入 | 既有 FR-RT-008 场景间转移 | RGS-BAS-013 §2.2 | 不绕过 |
| 战斗结束 → 回到大厅 | 既有场景退出 | RGS-BAS-013 §2.2 | 不绕过 |

---

# 5. 数据库 DDL 权威边界

battle_db 共 7 张表（battle_actors / battle_actions / pvp_configs / activity_configs / battle_results / battle_replays / guild_war_participants），以本文档为唯一权威。跨域表结构扩展须按 RGS-BAS-016 §3.1 回写原表权威文档。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| SubmitAction 验证 + 状态推进 | < 50ms | per ARC-002 既有同步机制目标值 | PH-4 负载试验 |
| 战斗结算确定请求 (走 EC) | < 200ms | per NFR-OP-005 既有 | PH-4 |
| 跨服 PVP 快照发布 | < 100ms | 跨服延迟容忍较高 | PH-6 |
| PvPConfig 选路 (高频查询) | < 5ms | idx_pvp_configs_mode_enabled 索引 | PH-2 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：
- battle_db 7 张表的物理 DDL（含 OCC + 跨库约束 + 索引）
- 战斗状态机 5 阶段推进伪代码（含 OCC 提交点）
- SubmitAction 幂等键构造（NFR-BAT-004 断线重连防双发）
- 战斗结算确定请求事务边界（EC 单点）
- 战斗退出/中止幂等性（RSK-BAT-002 关键路径）
- 跨服 PVP 数据脱敏（NFR-BAT-006）
- 5 项对接点（EC / MT / GSM / ARC-021 插件 / Scene）

本版本明确不覆盖、留待后续：
- 6 个 PVP 变体的 VariantHandler 内部算法 — 业务层实现
- 9 个 holiday_* 的 ActivityHandler 内部算法 — 业务层实现
- 战斗客户端预测算法 — 属 ARC-002 既有同步层
- 战斗录像存储后端选型（TBD-RPL-001） — 留待 PH-2 评审
- 跨服消息中间件选型（TBD-BAT-004） — 留待 PH-3
- 6 个 PVP 变体灰度配置切换的具体策略 — 留待运营 + 数值联合评审

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-040 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-040 §3 业务需求 | §2, §3 |
| RGS-REQ-040 §4 战斗引擎功能需求 | §3.1, §3.2 |
| RGS-REQ-040 §5 PVP 变体群 | §3.6 |
| RGS-REQ-040 §10 节日活动 | §2.4 |
| RGS-REQ-040 §11 NFR-BAT-001 | §6 |
| RGS-REQ-040 §11 NFR-BAT-003 服务器权威 | §3.1 |
| RGS-REQ-040 §11 NFR-BAT-004 断线重连 | §3.2 |
| RGS-REQ-040 §11 NFR-BAT-005 容量 | §2.7 |
| RGS-REQ-040 §11 NFR-BAT-006 隐私 | §3.5 |
| RGS-REQ-040 §13 AC-BAT-001〜006 | §3.1, §3.3, §3.4 |
| RGS-REQ-040 §14 RSK-BAT-002 中止幂等 | §3.4 |
| RGS-BAS-040 §3 架构总览 | §2 |
| RGS-BAS-040 §4 组件设计 | §3.1〜3.6 |
| RGS-BAS-040 §5 数据流时序 | §3.1, §3.3 |
| RGS-BAS-040 §6 接口契约 | §2.1〜2.7 |
| RGS-BAS-040 §7 ARC-046 落实 | §2.3, §2.4, §3.6 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.1, §3.3 |
| RGS-REQ-013 FR-GOV-001 EC 单点 | §3.3, §4.1 |
