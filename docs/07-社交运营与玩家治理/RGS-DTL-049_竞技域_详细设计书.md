# 详细设计书（詳細設計書 / Detailed Document）

**竞技域（PvP-Full Domain）详细设计 — 赛季 / 段位 / 战绩物理 DDL + 段位变更算法 + 跨服务并发一致性**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-049 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-BAS-042 v0.1 竞技域 基本设计书（本文档为其物理/实现级细化） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联 crate | `crates/pvp-full-service/` (pvp_full_db, 1,402 LOC) |
| 状态 | 草案, 待评审 |
| ARC | ARC-050 |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计）。细化 RGS-BAS-042 §3〜§7 为：pvp_full_db 物理 DDL（`seasons`/`player_ranks`/`pvp_records`/`season_configs`/`rank_protection_state`/`spectate_sessions`）、UpdateRank 事务边界（与战斗结算在同一 Saga 子流程，RSK-PVP-002 关键路径）、段位继承算法（per RGS-BAS-014 §5）、赛季归档伪代码 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | UpdateRank 事务边界是否真正与战斗结算一致；段位变更的并发场景是否防双发改段位 |
| 评审（DBA） | | | pvp_full_db 索引是否覆盖高频路径（玩家当前段位查询 / 战绩查询） |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 前言
2. 物理数据库设计：pvp_full_db
3. 关键算法详细设计
4. 对接点
5. 数据库 DDL 权威边界
6. 性能预算
7. 本文档的覆盖范围与后续计划

---

# 1. 前言

## 1.1 定位

RGS-BAS-042 给出了竞技域 1 service 的组件划分、接口契约、核心时序、ARC-050 数据驱动 Schema 的逻辑层设计——均为逻辑层面。本文档将其中涉及持久化数据的部分落实为物理 DDL，涉及判定逻辑的部分落实为可直接翻译为 Rust 实现的伪代码。

## 1.2 本文档不做什么

- 不重新决定 RGS-BAS-042 已确定的任何结构性选择。
- 不覆盖战斗状态机（属 battle PvPService 既有）。
- 不覆盖排行榜派生视图（属 GSM 既有）。
- 不覆盖段位保护算法（属 RGS-BAS-026 §4 既有）。

## 1.3 记述规则

DDL 以 PostgreSQL 为准，事件/消息以 Protobuf 风格给出，算法伪代码可直接对应 Rust `Result` 实现。沿用 RGS-DTL-001 §3.2 OCC + 幂等模式。

---

# 2. 物理数据库设计：pvp_full_db

pvp_full_db 为独立库（per ARC-008 5 独立 DB → 7 域扩展原则）。本文档新增 6 张表。

## 2.1 seasons（赛季实例表）

```sql
CREATE TABLE seasons (
    season_id          VARCHAR(64) PRIMARY KEY,  -- SeasonConfig 主键
    name               VARCHAR(128) NOT NULL,
    status             SMALLINT NOT NULL DEFAULT 0,  -- 0=Init 1=Running 2=Settling 3=Archiving 4=End
    start_at           TIMESTAMPTZ NOT NULL,
    end_at             TIMESTAMPTZ NOT NULL,
    reward_table       VARCHAR(64) NOT NULL,
    archive_after_days INTEGER NOT NULL DEFAULT 90,
    max_players        INTEGER NOT NULL,
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    archived_at        TIMESTAMPTZ,
    CONSTRAINT chk_seasons_status CHECK (status BETWEEN 0 AND 4)
);
CREATE INDEX idx_seasons_status_window
    ON seasons (status, start_at, end_at) WHERE status IN (0, 1);
```

## 2.2 player_ranks（玩家段位表）

```sql
CREATE TABLE player_ranks (
    player_id          UUID NOT NULL,
    season_id          VARCHAR(64) NOT NULL,
    current_rank       SMALLINT NOT NULL DEFAULT 0,  -- 0=青铜 1=白银 ... 7=王者
    current_score      INTEGER NOT NULL DEFAULT 0,
    rank_protected     BOOLEAN NOT NULL DEFAULT FALSE,
    version            INTEGER NOT NULL DEFAULT 0,  -- OCC
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (player_id, season_id),
    CONSTRAINT chk_player_ranks_rank CHECK (current_rank BETWEEN 0 AND 7)
);
CREATE INDEX idx_player_ranks_season_score
    ON player_ranks (season_id, current_score DESC);
    -- 支撑"赛季积分榜"高频查询（GSM 派生视图基线）
```

## 2.3 pvp_records（战绩表）

```sql
CREATE TABLE pvp_records (
    record_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id          UUID NOT NULL,
    opponent_id        UUID NOT NULL,
    season_id          VARCHAR(64) NOT NULL,
    outcome            SMALLINT NOT NULL,  -- 0=Win 1=Lose 2=Draw
    rank_before        SMALLINT NOT NULL,
    rank_after         SMALLINT NOT NULL,
    score_delta        INTEGER NOT NULL,
    finished_at_ms     BIGINT NOT NULL,
    CONSTRAINT chk_pvp_records_outcome CHECK (outcome BETWEEN 0 AND 2)
);
CREATE INDEX idx_pvp_records_player_time
    ON pvp_records (player_id, finished_at_ms DESC);
    -- 支撑"玩家 30 天内战绩查询"（per FR-PVP-021）
```

## 2.4 season_configs（赛季配置表）

```sql
CREATE TABLE season_configs (
    season_id          VARCHAR(64) PRIMARY KEY,
    name               VARCHAR(128) NOT NULL,
    start_at           TIMESTAMPTZ NOT NULL,
    end_at             TIMESTAMPTZ NOT NULL,
    rank_rules         JSONB NOT NULL,  -- promotion_threshold / demotion_threshold / protection_streak
    reward_table       VARCHAR(64) NOT NULL,
    archive_after_days INTEGER NOT NULL DEFAULT 90,
    max_players        INTEGER NOT NULL,
    enabled            BOOLEAN NOT NULL DEFAULT FALSE,
    version            INTEGER NOT NULL DEFAULT 0,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## 2.5 rank_protection_state（段位保护状态表）

```sql
CREATE TABLE rank_protection_state (
    player_id          UUID NOT NULL,
    season_id          VARCHAR(64) NOT NULL,
    loss_streak        INTEGER NOT NULL DEFAULT 0,  -- 当前连败数
    last_loss_at_ms    BIGINT,                          -- 最后一次失败时间
    protection_active  BOOLEAN NOT NULL DEFAULT FALSE, -- 段位保护激活中
    PRIMARY KEY (player_id, season_id)
);
```

## 2.6 spectate_sessions（观战会话表）

```sql
CREATE TABLE spectate_sessions (
    session_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    battle_id          UUID NOT NULL,
    spectator_id       UUID NOT NULL,
    spectate_type      SMALLINT NOT NULL,  -- 0=Live 1=Replay
    started_at_ms      BIGINT NOT NULL,
    ended_at_ms        BIGINT
);
CREATE INDEX idx_spectate_sessions_battle
    ON spectate_sessions (battle_id);
```

---

# 3. 关键算法详细设计

## 3.1 UpdateRank 事务边界（RSK-PVP-002 关键路径）

```rust
// 落实 RGS-BAS-042 §5.1 战斗结算 → 段位变更
// UpdateRank **必须**与战斗结算在同一 Saga 子流程, 防双发改段位
fn update_rank(
    player_id: PlayerId,
    season_id: SeasonId,
    outcome: BattleOutcome,
    session_epoch: SessionEpoch,
) -> Result<RankUpdateResult, PvpError> {
    // OCC 起点: SELECT ... FOR UPDATE (per RGS-DTL-001 §3.2)
    let mut player_rank = load_player_rank_for_update(player_id, season_id)?;
    // 计算段位变化 (per RGS-BAS-026 §4 段位保护)
    let rank_delta = RankCalculator::calculate(
        &player_rank,
        outcome,
        &load_season_config(season_id)?.rank_rules,
    )?;
    // 应用段位保护 (per RGS-BAS-026 §4 既有)
    let protected_delta = RankProtection::apply(&mut player_rank, rank_delta)?;
    // 落位 (OCC)
    player_rank.current_rank = clamp_rank(player_rank.current_rank + protected_delta.rank)?;
    player_rank.current_score += protected_delta.score;
    update_player_rank(&player_rank, expected_version: player_rank.version)?;
    // 战绩落位 (per FR-PVP-020)
    write_pvp_record(player_id, opponent, season_id, outcome, &player_rank, protected_delta)?;
    // 发布事件 (per NFR-PVP-005 Outbox)
    publish_outbox_event(RankChangedEvent {
        player_id, season_id, new_rank: player_rank.current_rank, new_score: player_rank.current_score,
    })?;
    Ok(RankUpdateResult { rank_delta: protected_delta, new_rank: player_rank.current_rank })
}
```

## 3.2 段位变更并发防护（防双发改段位）

```rust
// 同一玩家同一赛季短时间内可能有多场 PVP 结算并发触发 UpdateRank
// 解决: request_id 唯一约束 + OCC version 双锁
// request_id = hash(player_id + season_id + battle_id + outcome + timestamp_bucket)
// timestamp_bucket = floor(timestamp, 1s), 同一秒内的同场战斗映射同一 request_id
fn build_update_rank_request_id(player_id: PlayerId, season_id: SeasonId, battle_id: BattleId, timestamp_ms: i64) -> String {
    let bucket = timestamp_ms / 1000;  // 1s 桶
    sha256(&format!("{}/{}/{}/{}", player_id, season_id, battle_id, bucket))
}
```

## 3.3 段位继承算法（per RGS-BAS-014 §5）

```rust
// 赛季结算时, 段位继承:
fn inherit_rank(prev_season_id: SeasonId, new_season_id: SeasonId, player_id: PlayerId) -> Result<PlayerRank, PvpError> {
    let prev_rank = load_player_rank(player_id, prev_season_id)?;
    // 继承规则 (per SeasonConfig.inherit_rules):
    //   - 王者 → 大师 1
    //   - 半步王者 → 钻石 1
    //   - 大师 → 铂金 1
    //   - 钻石 → 黄金 1
    //   - 黄金及以下 → 维持原段位
    let inherited_rank = match prev_rank.current_rank {
        7 => 5,  // 王者 → 大师 1
        6 => 4,  // 半步王者 → 钻石 1
        5 => 3,  // 大师 → 铂金 1
        4 => 2,  // 钻石 → 黄金 1
        _ => prev_rank.current_rank,  // 黄金及以下维持
    };
    let inherited_score = (prev_rank.current_score as f32 * 0.7) as i32;  // 70% 积分继承
    // 新赛季初始段位
    let new_rank = PlayerRank {
        player_id,
        season_id: new_season_id,
        current_rank: inherited_rank,
        current_score: inherited_score,
        rank_protected: false,
        version: 0,
        updated_at: now(),
    };
    insert_player_rank(&new_rank)?;
    Ok(new_rank)
}
```

## 3.4 赛季结算伪代码

```rust
fn settle_season(season_id: SeasonId) -> Result<SettlementResult, PvpError> {
    let mut season = load_season_for_update(season_id)?;
    if season.status != SeasonStatus::Running as i16 {
        return Err(PvpError::SeasonAlreadySettled { season_id });
    }
    season.status = SeasonStatus::Settling as i16;
    update_season(&season, expected_version: season.version)?;
    // 1. 获取最终排名 (per GSM 派生视图既有)
    let final_rankings = call_gsm_get_final_rankings(season_id)?;
    // 2. 赛季奖励发放 (经 EC 单点, per FR-GOV-001)
    for ranking in final_rankings.iter().take(1000) {  // 前 1000 名
        commit_transaction(CommitTransactionRequest {
            request_id: deterministic_hash(&[season_id.as_bytes(), ranking.player_id.as_bytes()]),
            character_id: ranking.player_id.into(),
            operation: Operation::GrantItems(load_season_reward_table(season_id, ranking.rank)?),
            session_epoch: current_session_epoch(ranking.player_id)?,
            expected_version: current_wallet_version(ranking.player_id)?,
        })?;
    }
    // 3. 创建新赛季 + 段位继承
    let new_season_id = generate_next_season_id(season_id)?;
    let new_season_config = load_season_config(new_season_id)?;
    create_season(&new_season_config)?;
    let active_players = get_active_players(season_id)?;
    for player_id in active_players {
        inherit_rank(season_id, new_season_id, player_id)?;
    }
    // 4. 归档当前赛季
    season.status = SeasonStatus::Archiving as i16;
    update_season(&season, expected_version: season.version)?;
    archive_season(season_id)?;
    season.status = SeasonStatus::End as i16;
    season.archived_at = Some(now());
    update_season(&season, expected_version: season.version)?;
    Ok(SettlementResult { settled_players: active_players.len() })
}
```

## 3.5 实时观战快照订阅

```rust
// 落实 RGS-BAS-042 §5.3 观战实时订阅
fn start_live_spectate(spectator_id: PlayerId, battle_id: BattleId) -> Result<SpectateSession, PvpError> {
    // 校验 PVP 房间是否公开 (per FR-PVP-040)
    let pvp_room = call_battle_get_pvp_room_public(battle_id)?;
    if !pvp_room.is_public && !pvp_room.invite_list.contains(&spectator_id) {
        return Err(PvpError::SpectateForbidden { battle_id, spectator_id });
    }
    // 创建观战会话
    let session = SpectateSession {
        session_id: gen_random_uuid(),
        battle_id,
        spectator_id,
        spectate_type: SpectateType::Live,
        started_at_ms: now_ms(),
        ended_at_ms: None,
    };
    insert_spectate_session(&session)?;
    // 订阅战斗快照流 (per ARC-002 既有)
    subscribe_battle_snapshots(battle_id, spectator_id)?;
    Ok(session)
}
```

---

# 4. 对接点

## 4.1 与 battle-service PvPService 的对接（CON-PVP-001 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 战斗结算完成 | battle 触发 `BattleFinished` event | per RGS-DTL-047 | 触发 UpdateRank |
| 观战实时订阅 | battle 提供快照流 | per RGS-DTL-047 | 转播至观众 |

## 4.2 与 EconomyService 的对接（CON-PVP-003 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 赛季奖励发放 | `commit_transaction(GrantItems)` | RGS-DTL-001 §3.2 | 走 EC 既有路径 |

## 4.3 与 GSM 派生视图的对接（CON-PVP-002 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 段位变更 → 排行榜 | `PublishRank` | RGS-BAS-014 §4 | 派生视图更新 |
| 赛季结算 → 最终排名 | `GetFinalRankings` | RGS-BAS-014 §4 | 派生视图查询 |

## 4.4 与 MatchService 的对接（CON-PVP-004 落实）

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 段位保护 | 既有 RGS-BAS-026 §4 | per RGS-REQ-029 | 本域**不**自建匹配池保护 |
| 竞技匹配池边界 | 既有 MT | per RGS-REQ-029 | 由 MT 侧主导 |

## 4.5 与 ARC-021 插件通道的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| SeasonConfig 热加载 | `LoadConfig` | RGS-BAS-005 §4 | 仅订阅 config_updated |

## 4.6 与 replay-extra-service 的对接

| 触发点 | 调用 | 接口 | 备注 |
|---|---|---|---|
| 录像回放观战 | `StartReplaySpectate` | replay-extra 既有 | 本域**不**自建录像存储 |

---

# 5. 数据库 DDL 权威边界

pvp_full_db 共 6 张表，以本文档为唯一权威。

---

# 6. 性能预算

| 路径 | 目标 P99 | 实测方法 | 留待阶段 |
|---|---|---|---|
| UpdateRank 落位 | < 100ms | per NFR-PVP-001 | PH-4 |
| 赛季积分榜查询 | < 50ms | idx_player_ranks_season_score | PH-2 |
| 战绩查询 (30 天) | < 100ms | idx_pvp_records_player_time | PH-2 |
| 观战实时延迟 | < 2s | per FR-PVP-042 | PH-6 |

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：
- pvp_full_db 6 张表的物理 DDL
- UpdateRank 事务边界（含并发防护，RSK-PVP-002）
- 段位继承算法（per RGS-BAS-014 §5）
- 赛季结算伪代码（4 步：排名查询 → 奖励发放 → 继承 → 归档）
- 实时观战快照订阅
- 6 项对接点

本版本明确不覆盖、留待后续：
- 战斗状态机 — 属 battle PvPService 既有
- 段位保护算法 — 属 RGS-BAS-026 §4 既有
- 排行榜派生视图 — 属 GSM 既有
- 录像存储后端 — 属 replay-extra 既有
- TBD-PVP-001（单赛季最大参赛人数）— 留待策划确认

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-REQ-042 §1.3 与既有基础设施关系 | §4 |
| RGS-REQ-042 §3 业务需求 | §2, §3 |
| RGS-REQ-042 §4 功能需求 | §3.1, §3.5 |
| RGS-REQ-042 §5 NFR | §6 |
| RGS-REQ-042 §6 ARC-050 | §2.4, §3.1, §3.3 |
| RGS-REQ-042 §7 AC-PVP-001〜005 | §3.1, §3.3, §3.4 |
| RGS-REQ-042 §8 RSK-PVP-002 | §3.1, §3.2 |
| RGS-BAS-042 §3 架构总览 | §2 |
| RGS-BAS-042 §4 组件设计 | §3.1〜3.5 |
| RGS-BAS-042 §5 数据流时序 | §3.1, §3.4 |
| RGS-BAS-042 §6 接口契约 | §2.1〜2.6 |
| RGS-DTL-001 §3.2 OCC + 幂等 | §3.1, §3.2, §3.4 |
| RGS-BAS-014 §5 段位继承 | §3.3 |
