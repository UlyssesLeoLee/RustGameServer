# 详细设计书（詳細設計書 / Detailed Design Document）

**匹配系统：队列/评分物理数据库设计・事件线格式・扩圈与跨分片撮合算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-026 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-026 匹配系统 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档为第四份详细设计文档）。细化RGS-BAS-026§3.1逻辑数据模型为MT限界上下文（`match_db`同库）内`queue_entries`／`match_ratings`／`match_quality_metrics`三表具体DDL、§4.1扩圈算法与§5.1跨分片OCC校验落实为可直接翻译为Rust实现的伪代码、§7各时序图落实为具体状态转移代码、事件落实为具体线格式。**本版本不覆盖**：评分算法本身（ELO/Glicko-2/TrueSkill）的最终选型与具体公式实现（RGS-REQ-029§11已标注为TBD，需另行ADR决定后再补充本文档）、GM/运营配置后台的UI细节。见§7 | 全部 |
| 0.2 | 2026-08-17 | 架构师 | — | 负责人指示"开子代理完成剩余的"（技术选型TBD收尾）。新增§7解决评分算法最终选型（Glicko-2，排除TrueSkill因IP历史模糊性、排除纯ELO因无不确定度建模），给出`RatingSettlement.calculate()`核心公式；`match_ratings`新增`volatility`列。原§7覆盖范围章节顺延为§8并更新内容 | §1.2、§2（`match_ratings`新增列）、§7（新增）、原§7→§8 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-026§3.1逻辑模型一致，OCC乐观锁字段是否覆盖§5.1跨分片竞态场景 |
| 评审（DBA） | | | `queue_entries`短生命周期表的清理策略是否与既有G-005清理模式脚本兼容 |
| 审批（负责人） | | | 本文档的基准化；评分算法选型ADR何时启动评审 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：MT限界上下文匹配三表](#2-物理数据库设计mt限界上下文匹配三表)
3. [事件线格式](#3-事件线格式)
4. [扩圈算法详细设计](#4-扩圈算法详细设计)
5. [跨分片OCC校验详细设计](#5-跨分片occ校验详细设计)
6. [排队/确认/回填状态转移详细设计](#6-排队确认回填状态转移详细设计)
7. [评分算法选型（Glicko-2）](#7-评分算法选型rgs-req-029§11-tbd最终决定)
8. [本文档的覆盖范围与后续计划](#8-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-026给出了`QueueEntry`/`MatchRating`的逻辑字段表、扩圈算法的文字描述与流程图、跨分片OCC校验的时序图。本文档将其落实为可执行DDL、事件消息格式，以及算法/状态机的伪代码级实现，覆盖RGS-BAS-026本身留白的边界条件（如OCC冲突后的具体重试语义、扩圈容差函数的可编程形式）。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-026已确定的任何结构性选择（三表依附MT限界上下文既有存储不新建库、跨分片撮合走OCC而非分布式锁、连败保护只影响撮合不写回真实评分）。
- 评分算法本身已于v0.2在§7给出最终选型（Glicko-2），不再是本文档遗留缺口；团队场景的精细化扩展仍留待后续（见§8）。
- 不覆盖运营配置后台（`shard_scope`/连败保护开关/回填开关）的UI细节，仅覆盖这些配置在读取侧（`MatchmakerWorker`）如何被消费。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，事件以Protobuf风格给出，算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：MT限界上下文匹配三表

对应RGS-BAS-026§3.1/§3.2。三表与`match_db`同库（同一MT限界上下文事务边界），本文档只新增表结构。

```sql
-- 队列条目表，对应FR-MM-010/013，短生命周期表
CREATE TABLE queue_entries (
    entry_id          BIGSERIAL PRIMARY KEY,
    party_ref         BIGINT NOT NULL,        -- 逻辑引用RGS-REQ-017既有队伍模型
    mode              TEXT NOT NULL,
    shard_scope       TEXT NOT NULL CHECK (shard_scope IN ('SHARD_LOCAL', 'POOL_SHARED')),
    composite_rating  DOUBLE PRECISION NOT NULL,
    status            TEXT NOT NULL DEFAULT 'WAITING'
                        CHECK (status IN ('WAITING', 'MATCHED_PENDING_CONFIRM', 'CONFIRMED', 'ABANDONED')),
    enqueued_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    match_ref         BIGINT NULL,             -- 撮合成立后指向MATCH记录(RGS-BAS-001§5.5既有表)
    version           INTEGER NOT NULL DEFAULT 0,  -- OCC乐观锁，对应§5.1跨分片竞态校验
    origin_shard_id   TEXT NOT NULL            -- 该条目发起所在分片，POOL_SHARED模式下供撮合分片选择参考
);

CREATE INDEX idx_queue_entries_scan
    ON queue_entries (mode, shard_scope, status)
    WHERE status = 'WAITING';   -- 支撑MatchmakerWorker核心扫描路径,对应§3.2既定索引

-- 匹配评分表，对应FR-MM-001，长期表
CREATE TABLE match_ratings (
    character_id        BIGINT NOT NULL,
    mode                 TEXT NOT NULL,
    rating_value          DOUBLE PRECISION NOT NULL,
    rating_deviation       DOUBLE PRECISION NULL,   -- 现已选定Glicko-2(§7),本列即为其RD不确定度值,不再恒为NULL
    volatility              DOUBLE PRECISION NOT NULL DEFAULT 0.06,  -- Glicko-2波动率σ,§7 solve_new_volatility所需的额外状态列
    consecutive_losses     INTEGER NOT NULL DEFAULT 0,
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (character_id, mode)
);

-- 匹配质量度量摘要表，对应FR-MM-030，撮合成立瞬间写入，长期保留供运营分析
CREATE TABLE match_quality_metrics (
    match_ref           BIGINT PRIMARY KEY,
    rating_gap            DOUBLE PRECISION NOT NULL,
    total_wait_seconds      INTEGER NOT NULL,       -- 取参与条目等待时长最大值,同§4.3
    used_backfill           BOOLEAN NOT NULL DEFAULT FALSE,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`queue_entries`的清理（`MATCHED`/`ABANDONED`后条目归档）复用既有G-005清理模式的调度脚本，不在本文档内重新定义清理机制本身，仅声明清理谓词：`status IN ('CONFIRMED', 'ABANDONED') AND enqueued_at < now() - retention_period`（`retention_period`取值遵循RGS-BAS-007既定短生命周期表默认保留期，无域专属例外）。

---

## 3. 事件线格式

对应RGS-BAS-026§5.1（跨分片队列同步）、§6.2（评分变更展示联动）、§7.4/§7.5（回填/组队结构信号）。

```protobuf
// QueueEntryCreated / QueueEntryStatusChanged：POOL_SHARED模式跨分片广播，对应§5.1
message QueueEntryCreated {
  int64  entry_id         = 1;
  int64  party_ref         = 2;
  string mode               = 3;
  double composite_rating    = 4;
  int64  enqueued_at_ms       = 5;
  string origin_shard_id       = 6;
  repeated int64 member_character_ids = 7;  // 对应§7.5组队结构信号,组队撮合与ANT消费均需要成员展开列表
}

message QueueEntryStatusChanged {
  int64  entry_id   = 1;
  string new_status  = 2;   // 取值同queue_entries.status CHECK约束
  int64  version_after = 3;  // 供接收方分片校验自身视图是否落后,落后则以此为准强制同步
}

// MatchRatingChanged：可选,对应§6.2单向联动GSM展示排行
message MatchRatingChanged {
  int64  character_id  = 1;
  string mode            = 2;
  double rating_value      = 3;
  int64  match_ref          = 4;
  int64  settled_at_ms        = 5;
}
```

`QueueEntryCreated`的`member_character_ids`字段同时服务于两个既有设计点：跨分片扩圈算法需要完整队伍构成计算合成评分（§4.2既定的算术平均需要成员`rating_value`），ANT域§7.5的组队结构信号消费也需要同一份成员列表——不为两个消费方重复发布两条相似事件，复用同一事件载荷即可。

---

## 4. 扩圈算法详细设计

对应RGS-BAS-026§4.1流程图，落实为容差函数与撮合尝试的伪代码。

### 4.1 容差函数（可编程形式）

```rust
// 分段线性,满足"单调不减"约束(RGS-BAS-026§4.1);具体分段参数为PH-5实测前的初始提案,非最终值
fn tolerance(waiting_seconds: u32, params: &ToleranceParams) -> f64 {
    let t = waiting_seconds as f64;
    if t <= params.grace_period_secs {
        params.initial_tolerance                              // 提案初始值: 50 (评分单位)
    } else {
        let widened = params.initial_tolerance
            + params.widen_rate_per_sec * (t - params.grace_period_secs);  // 提案速率: 2/秒
        widened.min(params.max_tolerance)                     // 提案上限: 400
    }
}
```

`ToleranceParams`三个数值（`initial_tolerance`/`widen_rate_per_sec`/`max_tolerance`）与RGS-DTL-025§5同类做法一致，标注为初始提案，随PH-5数据校准，不属于本文档最终交付值。

### 4.2 单轮撮合尝试

```rust
fn matchmaker_tick(mode: &str, shard_scope: ShardScope, now: Instant) -> Result<Vec<ProposedMatch>, MMError> {
    let candidates = scan_waiting_entries(mode, shard_scope);  // 走§2 idx_queue_entries_scan索引
    let mut proposals = vec![];
    let mut consumed: HashSet<EntryId> = HashSet::new();

    for entry in &candidates {
        if consumed.contains(&entry.entry_id) { continue; }
        let tol = tolerance(entry.waiting_seconds(now), &tolerance_params_for(mode));
        let compatible = candidates.iter()
            .filter(|c| !consumed.contains(&c.entry_id) && c.entry_id != entry.entry_id)
            .filter(|c| (c.composite_rating - entry.composite_rating).abs() <= tol)
            .collect::<Vec<_>>();

        if let Some(team_combo) = try_compose_teams(entry, &compatible, mode_roster_size(mode)) {
            // §4.2组队编制规则: 队伍规模不超上限已在入队时(QueueGateway)拒绝,此处只需补齐,不重复校验上限
            for e in &team_combo { consumed.insert(e.entry_id); }
            proposals.push(ProposedMatch { entries: team_combo, tolerance_used: tol });
        }
        // 未找到兼容组合: 该entry维持WAITING,不报错,等待下一轮tick(对应流程图D分支"否")
    }
    Ok(proposals)
}
```

`try_compose_teams`内部按§4.2既定规则（合成评分取队伍成员`rating_value`算术平均、小队伍须由单排/更小队伍补齐且补齐仍受当前容差约束）实现，具体组合搜索策略（贪心/回溯）不属于架构决策范畴，留待实现阶段按性能实测选择，本文档不强制指定。

---

## 5. 跨分片OCC校验详细设计

对应RGS-BAS-026§5.1时序图，落实为具体的乐观锁更新语句与冲突后处理路径。

```sql
-- MatchmakerWorker撮合前的OCC校验+状态更新,单条SQL保证原子性(不拆两步,避免TOCTOU)
UPDATE queue_entries
SET status = 'MATCHED_PENDING_CONFIRM', match_ref = $proposed_match_ref, version = version + 1
WHERE entry_id = $entry_id AND status = 'WAITING' AND version = $expected_version;
-- 影响行数=1: 校验通过,本次撮合对该条目生效
-- 影响行数=0: 校验失败,已被其他分片的MatchmakerWorker抢先撮合(或该条目已被玩家主动放弃)
```

```rust
fn commit_proposed_match(proposal: &ProposedMatch) -> Result<MatchCreated, MMError> {
    let mut succeeded = vec![];
    for entry in &proposal.entries {
        match occ_update_entry(entry.entry_id, entry.version, proposal.match_ref) {
            Ok(1) => succeeded.push(entry),
            Ok(0) => {
                // 对应§5.1"校验失败(已被其他分片抢先撮合)"分支: 整个候选撮合作废,不做部分撮合
                rollback_succeeded(&succeeded)?;  // 已成功更新的条目回退状态为WAITING,version不变(未被抢占方不受影响)
                return Err(MMError::ConcurrentlyMatched { losing_entry: entry.entry_id });
            }
            Err(e) => { rollback_succeeded(&succeeded)?; return Err(e.into()); }
            _ => unreachable!(),
        }
    }
    Ok(finalize_match(proposal))  // 全部条目OCC通过后才真正进入§6.1 MATCH创建路径
}
```

**关键设计要点（落实RGS-BAS-026§5.1"杜绝同一玩家被重复撮合进多个对局"）**：整个候选撮合是"全有或全无"——只要候选组合中任一条目OCC校验失败，整组候选作废并回退，不允许"部分玩家进入这场对局、被抢占的玩家留在队列"这种不一致状态，因为该状态会破坏组队编制规则（缺人的对局不应当直接开始）。

---

## 6. 排队/确认/回填状态转移详细设计

对应RGS-BAS-026§7.2〜§7.4三个时序图，统一落实为`queue_entries.status`状态机的合法转移表与对应SQL：

| 当前状态 | 触发事件 | 目标状态 | 对应RGS-BAS-026章节 |
|---|---|---|---|
| `WAITING` | 玩家主动退出 | `ABANDONED` | §7.2 |
| `WAITING` | 撮合成立（§5 OCC通过） | `MATCHED_PENDING_CONFIRM` | §5.1/§4 |
| `MATCHED_PENDING_CONFIRM` | 全员确认通过 | `CONFIRMED` | §7.3 |
| `MATCHED_PENDING_CONFIRM` | 本人放弃 | `ABANDONED` | §7.3 |
| `MATCHED_PENDING_CONFIRM` | 他人放弃/超时（本人未放弃） | `WAITING`（`enqueued_at`不重置，见下） | §7.3 |

```rust
fn on_confirmation_window_closed(match_ref: MatchRef) -> Result<(), MMError> {
    let entries = query_entries_by_match_ref(match_ref);
    if entries.iter().all(|e| e.confirmed) {
        for e in &entries { update_status(e.entry_id, e.version, Status::Confirmed)?; }
        create_match_record(match_ref, &entries)?;  // 移交RGS-BAS-001§5.5既有MATCH状态机,本文档不重复设计
    } else {
        for e in &entries {
            if e.confirmed { continue; }
            if e.self_abandoned {
                update_status(e.entry_id, e.version, Status::Abandoned)?;
            } else {
                // 对应§7.3"enqueued_at按原值保留,等待时长不清零"——UPDATE语句不触碰enqueued_at列
                reset_to_waiting_preserving_enqueued_at(e.entry_id, e.version)?;
            }
        }
    }
    Ok(())
}
```

回填（§7.4）在既有对局参与者提前退出事件触发后，`BackfillWorker`按§4扩圈算法同一套`tolerance`/撮合尝试逻辑在匹配池中寻找候选，唯一差异是撮合目标不是"创建新`MATCH`"而是"追加进已存在`MATCH_PARTICIPANT`"，故本文档不重复给出伪代码，仅声明复用关系：`BackfillWorker`调用与§4.2相同的`try_compose_teams`（`roster_size=1`，即只需补齐一个空位而非整队），成功后写入`MATCH_PARTICIPANT`而非走§6.1创建路径。

---

## 7. 评分算法选型（RGS-REQ-029§11 TBD，最终决定）

选型为**Glicko-2**（Mark Glickman发表的公开算法，含官方实现指南，无专利/许可争议，长期被开源棋类/竞技游戏服务器采用），而非ELO或TrueSkill：

- **排除TrueSkill**：其原始实现与专利历史关联微软研究院，即便published math本身可重新实现，"全部采用开源免费策略"约束下应避免任何存在IP历史模糊性的选项，优先选择零IP争议的方案——Glicko-2满足此条件，ELO同样满足但精度不足（见下）。
- **排除纯ELO**：ELO不建模"评分不确定度"，新玩家/久未匹配玩家的评分收敛速度慢，且无法自然表达"评分差不大但把握程度不同"的两个玩家；Glicko-2的`rating_deviation`字段直接解决此问题，与§2 `match_ratings`表早已预留的`rating_deviation`列（RGS-BAS-026§3.1设计时即声明"若算法需要不确定度则用此字段，若不需要则恒为空"）完全吻合，无需改动物理schema。
- **参数**：初始`rating_value=1500`，初始`rating_deviation=350`，系统常数`τ=0.5`（Glickman官方指南推荐范围0.3〜1.2，游戏竞技场景取偏保守的中间值），评分稳定期后`rating_deviation`收敛下限设为`30`（低于此值不再继续收窄，防止长期活跃玩家的不确定度归零后对评分微小波动过度敏感）。

`RatingSettlement.calculate()`伪代码（对应RGS-DTL-026§6.1既有结算路径，填入此前留空的计算逻辑）：

```rust
// Glicko-2标准更新公式的直接翻译,变量命名对应官方指南记号
fn glicko2_update(player: &MatchRating, opponent: &MatchRating, outcome: MatchOutcome, tau: f64) -> (f64, f64) {
    let (mu, phi) = to_glicko2_scale(player.rating_value, player.rating_deviation);        // 转换到Glicko-2内部标度(除以173.7178)
    let (mu_j, phi_j) = to_glicko2_scale(opponent.rating_value, opponent.rating_deviation);

    let g_phi_j = 1.0 / (1.0 + 3.0 * phi_j.powi(2) / std::f64::consts::PI.powi(2)).sqrt();
    let e = 1.0 / (1.0 + (-g_phi_j * (mu - mu_j)).exp());
    let score = match outcome { MatchOutcome::Win => 1.0, MatchOutcome::Loss => 0.0, MatchOutcome::Draw => 0.5 };

    let v = 1.0 / (g_phi_j.powi(2) * e * (1.0 - e));
    let delta = v * g_phi_j * (score - e);

    let new_sigma = solve_new_volatility(phi, player.volatility_or_default(), delta, v, tau);  // 迭代求解,官方指南附录给出的收敛算法
    let phi_star = (phi.powi(2) + new_sigma.powi(2)).sqrt();
    let new_phi = 1.0 / (1.0 / phi_star.powi(2) + 1.0 / v).sqrt();
    let new_mu = mu + new_phi.powi(2) * g_phi_j * (score - e);

    from_glicko2_scale(new_mu, new_phi.max(MIN_RATING_DEVIATION_INTERNAL_SCALE))  // 回转标度前应用§7既定收敛下限
}
```

`solve_new_volatility`（波动率迭代求解）与`to_glicko2_scale`/`from_glicko2_scale`（内部标度换算）为Glicko-2标准算法的固定组成部分，直接照官方指南实现，不属于本项目自定义逻辑，故本文档不展开其内部细节，仅确认调用契约。组队场景（多人对多人）的评分更新按"每个成员视为独立与对方队伍`composite_rating`对局一次"简化处理（Glicko-2原生为1v1设计，团队场景的精确扩展本身是另一层不确定的研究问题，简化处理是本项目的**有意选择**而非疏漏，效果留待PH-5数据评估是否需要更精细的团队评分扩展）。

license确认：Glicko-2算法本身为公开发表的数学方法，非专利，实现代码在Rust中从零编写，无第三方依赖引入，无CON-001顾虑。

---

## 8. 本文档的覆盖范围与后续计划

本文档覆盖：MT限界上下文匹配三表（`queue_entries`/`match_ratings`/`match_quality_metrics`）物理DDL、`QueueEntryCreated`/`QueueEntryStatusChanged`/`MatchRatingChanged`三个事件的具体线格式、扩圈容差函数与单轮撮合尝试伪代码、跨分片OCC"全有或全无"提交逻辑、排队/确认/回填的完整状态转移表与对应实现、**评分算法最终选型（Glicko-2）与`RatingSettlement.calculate()`核心公式**。

本版本明确不覆盖、留待后续：

- §4.1容差函数三个参数的最终校准值，以及`try_compose_teams`的具体组合搜索实现策略——均标注为PH-5实测前的初始提案/留待实现阶段选择，不属于架构层面决策。
- 连败保护偏移量计算的具体公式（RGS-BAS-026§7.1标注为TBD阈值，本文档未展开，理由与容差参数相同）。
- 运营配置后台（`shard_scope`/连败保护/回填三类开关）的写入侧UI与API细节，本文档只覆盖读取侧消费逻辑。
- Glicko-2团队场景扩展的精细化（当前为§7声明的简化处理），`solve_new_volatility`具体迭代实现代码。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-026§3.1 队列/评分逻辑数据模型 | §2 |
| RGS-BAS-026§3.2 物理落位与约束 | §2 |
| RGS-BAS-026§4.1 扩圈算法 | §4 |
| RGS-BAS-026§4.2 组队编制规则 | §4.2 |
| RGS-BAS-026§4.3 匹配质量度量 | §2（`match_quality_metrics`表） |
| RGS-BAS-026§5.1 跨分片边界判定落地 | §3、§5 |
| RGS-BAS-026§5.2 模式启用配置 | §7（明确排除写入侧UI） |
| RGS-BAS-026§6.1 结算路径 | §7（明确排除评分算法本身） |
| RGS-BAS-026§6.2 与GSM展示排行联动 | §3（`MatchRatingChanged`） |
| RGS-BAS-026§7.1〜7.5 连败保护/放弃/确认/回填/组队信号 | §6 |
| RGS-DTL-002（挂载脚手架物理落地） | 前提依赖，本文档假定MT域已按RGS-DTL-002完成挂载 |
