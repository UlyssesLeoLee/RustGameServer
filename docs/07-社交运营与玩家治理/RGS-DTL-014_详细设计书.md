# 详细设计书（詳細設計書 / Detailed Design Document）

**排行榜、任务成就与玩家治理：派生视图与任务/邮件/举报物理数据库设计・事件线格式・触发引擎与信誉度算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-014 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-014 排行榜、任务成就与玩家治理 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-011/012/013/019并行产出）。细化RGS-BAS-014§2.2`RankingDimensionConfig`为具体DDL、§3.1`QuestDefinition`为具体DDL、§4.1`MailMessage`为具体DDL、§5.1/§5.1.1/§5.2`PlayerReport`/`ReporterReputation`/`PlayerBlocklist`为具体DDL、§2.3更新时序与§2.3.1异常分支落实为具体事件线格式与Rust伪代码、§5.1.1信誉度重算公式给出初始提案（TBD留白部分，同RGS-DTL-025§5既定处理方式）。**本版本不覆盖**：TBD-GSM-001派生视图存储选型的最终决议、TBD-GSM-002赛季继承规则的具体公式、TBD-GSM-003举报处理SLA数值。见§6 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-014§2/§3/§4/§5逻辑字段表完全一致，`RankingViewUpdater`幂等/乱序覆盖防护伪代码是否遗漏边界条件 |
| 评审（DBA） | | | `PlayerReport.dedup_key`唯一索引与`PlayerBlocklist(owner_id, blocked_id)`唯一约束是否确实落地为数据库层强制 |
| 审批（负责人） | | | 本文档的基准化；信誉度重算公式的初始提案是否可直接采纳或需策划评审后再定 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：GD/EC限界上下文新增表](#2-物理数据库设计gdec限界上下文新增表)
3. [事件线格式：排行榜更新](#3-事件线格式排行榜更新)
4. [算法详细设计](#4-算法详细设计)
5. [TBD-GSM参数与公式初始提案](#5-tbd-gsm参数与公式初始提案)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-014给出了排行榜派生视图组件划分、`RankingDimensionConfig`逻辑字段表、更新时序与异常分支文字流程、任务/成就`QuestDefinition`配置表逻辑字段、邮件`MailMessage`逻辑数据模型、举报`PlayerReport`/信誉度`ReporterReputation`/黑名单`PlayerBlocklist`逻辑字段表、赛季重置时序文字流程——均为逻辑设计层面（RGS-BAS-014§1已明确"字段级设计以逻辑字段表述，物理DDL遵循RGS-BAS-007既定标准执行，不在本文档重复定义"）。本文档承接该"不重复定义"留出的物理化职责，给出可执行DDL、事件线格式、算法伪代码。

### 1.2 本文档不做什么

- **不重新决定**RGS-BAS-014已确定的任何结构性选择（排行榜权威表回落规则、任务/成就配置驱动不写专属订阅代码、举报仅产生信号不直接触发处罚、黑名单查询边界单向不可反查）。
- **不选定**TBD-GSM-001（派生视图存储选型）/TBD-GSM-002（赛季继承规则）/TBD-GSM-003（举报处理SLA）——三项均为RGS-BAS-014已标注的独立评审事项，本文档不越权决定，仅在§5对其中可提出初始默认值的部分（信誉度权重系数）给出提案，其余两项无法在不预先选定TBD-GSM-001/002本身的情况下给出物理设计，故不展开。
- **不覆盖**GM后台查询页/邮件撰写页等前端UI细节。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，事件以Protobuf风格给出，算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：GD/EC限界上下文新增表

对应RGS-BAS-014§2.2.1、§3.1、§4.1、§5.1、§5.1.1、§5.2。全部表依附既有GD/EC/AD限界上下文数据库（依RGS-BAS-014§1"依附既有限界上下文，不新建独立数据库"），本文档只新增表结构。

```sql
-- 排行维度配置表，对应§2.2.1 RankingDimensionConfig，落位GD(复用ARC-016热更新配置存储介质)
CREATE TABLE ranking_dimension_configs (
    dimension_id      VARCHAR(64) PRIMARY KEY,
    source_context      TEXT NOT NULL CHECK (source_context IN ('EC', 'GD', 'MT')),
    source_event           TEXT NOT NULL,
    score_field_path          TEXT NOT NULL,
    season_scoped                BOOLEAN NOT NULL DEFAULT FALSE,
    enabled                        BOOLEAN NOT NULL DEFAULT FALSE   -- 默认false: 新增维度须显式启用才对外暴露查询(灰度上线)
);

-- 任务/成就定义表，对应§3.1 QuestDefinition，落位GD
CREATE TABLE quest_definitions (
    quest_id          VARCHAR(64) PRIMARY KEY,
    category             TEXT NOT NULL CHECK (category IN ('quest', 'achievement')),
    trigger_condition       TEXT NOT NULL,   -- 声明式表达式原文，解析由§4.2 QuestConditionSubscriber在应用层完成，非DB层解析
    reset_policy               TEXT NOT NULL DEFAULT 'season' CHECK (reset_policy ~ '^(never|season|period_days:[0-9]+)$'),
    -- CHECK正则覆盖三种取值形态: 'never' | 'season' | 'period_days:N'(N为正整数)，
    -- 数据库层拦截格式错误的reset_policy写入，不依赖应用层解析时才发现格式非法
    reward_spec                   JSONB NOT NULL   -- 引用既有物品/货币发放规格结构
);

-- 玩家任务进度表，对应§3.2 QuestProgressStore，落位GD
CREATE TABLE quest_progress (
    character_id     UUID NOT NULL,
    quest_id            VARCHAR(64) NOT NULL REFERENCES quest_definitions(quest_id),
    state                  SMALLINT NOT NULL DEFAULT 0,
    -- 0=可领取 1=已领取 2=进行中 3=已完成 4=已领奖，对应§3.2 QuestStateMachine五态
    progress_value            JSONB NOT NULL DEFAULT '{}',  -- 进度计数(如"已击杀数量")，结构随quest_id对应的trigger_condition变体
    updated_at                    TIMESTAMPTZ NOT NULL DEFAULT now(),
    version                         INTEGER NOT NULL DEFAULT 0,  -- OCC，异步进度更新与领奖操作的并发防护(见§4.1)
    PRIMARY KEY (character_id, quest_id),
    CONSTRAINT chk_quest_progress_state CHECK (state BETWEEN 0 AND 4)
);
CREATE INDEX idx_quest_progress_character_state
    ON quest_progress (character_id, state);  -- 支撑"我的可领取任务列表"高频查询

-- 邮件表，对应§4.1 MailMessage，落位GD
CREATE TABLE mail_messages (
    mail_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id         UUID NOT NULL,
    mail_type              TEXT NOT NULL CHECK (mail_type IN ('system', 'business')),
    subject                   VARCHAR(200) NOT NULL,
    body                        TEXT NOT NULL,
    attachments                   JSONB NOT NULL DEFAULT '[]',
    read_status                     SMALLINT NOT NULL DEFAULT 0,   -- 0=unread 1=read
    claim_status                       SMALLINT NOT NULL DEFAULT 0,   -- 0=unclaimed 1=claimed
    expire_at                             TIMESTAMPTZ NOT NULL
) PARTITION BY RANGE (expire_at);
-- 按expire_at月度分区，复用RGS-BAS-007§4既定分区归档标准(§4.3既定)

CREATE INDEX idx_mail_messages_recipient_unread
    ON mail_messages (recipient_id, read_status, expire_at);
    -- 支撑"收件箱列表按收件人过滤未读/未过期"高频查询(§4.1既定索引设计)
CREATE INDEX idx_mail_messages_expire
    ON mail_messages (expire_at);
    -- 支撑§4.3到期清理批处理扫描

-- 举报表，对应§5.1 PlayerReport，落位AD(GM治理域，同RGS-DTL-025反作弊三表挂靠原则)
CREATE TABLE player_reports (
    report_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id           UUID NOT NULL,
    target_id                UUID NOT NULL,
    report_type                 TEXT NOT NULL CHECK (report_type IN ('cheating', 'harassment', 'inappropriate_name', 'other')),
    context_ref                    TEXT,
    dedup_key                        TEXT NOT NULL,   -- reporter_id+target_id+滚动时间窗口的哈希，见§4.4构造方式
    signal_weight                       NUMERIC(4,2) NOT NULL DEFAULT 1.00,
    created_at                             TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_player_reports_target_created
    ON player_reports (target_id, created_at);   -- GM后台按被举报者聚合查询
CREATE UNIQUE INDEX uq_player_reports_dedup_key
    ON player_reports (dedup_key);
    -- 唯一索引直接在数据库层阻止同一滚动窗口内的重复计数写入(FR-GSM-033既定，不依赖应用层去重兜底)

-- 举报者信誉度表，对应§5.1.1 ReporterReputation，落位AD/GD(依附既有GD/AD上下文数据库)
CREATE TABLE reporter_reputations (
    reporter_id         UUID PRIMARY KEY,
    substantiated_count    INTEGER NOT NULL DEFAULT 0,
    unsubstantiated_count     INTEGER NOT NULL DEFAULT 0,
    weight_multiplier            NUMERIC(3,2) NOT NULL DEFAULT 1.00 CHECK (weight_multiplier BETWEEN 0.10 AND 1.50),
    -- CHECK约束直接落实§5.1.1既定的[0.1, 1.5]取值范围，数据库层强制而非仅应用层校验
    updated_at                       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 黑名单表，对应§5.2 PlayerBlocklist，落位GD
CREATE TABLE player_blocklists (
    owner_id       UUID NOT NULL,
    blocked_id       UUID NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (owner_id, blocked_id)
    -- 复合主键即天然满足§5.2"(owner_id, blocked_id)复合唯一索引"要求，不另建冗余唯一索引
);
CREATE INDEX idx_player_blocklists_owner ON player_blocklists (owner_id);
-- 刻意不在blocked_id上建立任何索引: NFR-GSM-005"不得暴露谁拉黑了谁"要求不仅是查询接口层面的约束，
-- 也不应留下可被利用的索引通道——若blocked_id有索引，即便应用层不提供对应查询接口，
-- 数据库层面仍"技术上可高效反查"，与NFR-GSM-005的精神相悖，故本文档明确不建该索引(而非仅约定不用)
```

`quest_progress.version`列的OCC乐观锁用途值得单独说明：任务进度存在两条独立写入路径——`QuestConditionSubscriber`异步事件消费更新进度、玩家主动发起的领奖操作更新`state`——二者若无并发控制，可能出现"进度刚好在领奖瞬间被追加导致状态被覆盖"的竞态，`version`列复用RGS-DTL-001§3.2既定OCC模式统一处理，不为此新增专属锁机制。

---

## 3. 事件线格式：排行榜更新

对应RGS-BAS-014§2.3更新时序中`RankingScoreChanged`事件（RGS-BAS-014已述"复用ARC-010事件基础设施"，本文档固定其具体线格式）。

```protobuf
// RankingScoreChanged: RankingSource发布，对应§2.3更新时序第一步
message RankingScoreChanged {
  string player_id       = 1;
  string dimension_id      = 2;   // 对应ranking_dimension_configs.dimension_id
  double new_score            = 3;
  int64  season_id               = 4;   // dimension season_scoped=false时固定为0(哨兵值，非NULL，proto3无原生NULL语义)
  int64  event_occurred_at_ms      = 5;   // 事件产生时间戳，供§4.1"乱序到达仅当更晚才写入"判定使用
}
```

`season_id`用固定哨兵值`0`表示"非赛季维度"而非省略字段/使用proto3默认零值语义混淆——`season_scoped=false`的维度（如`player_level`）恒定发布`season_id=0`，消费端`RankingViewUpdater`按`ranking_dimension_configs.season_scoped`判断是否需要将`season_id`纳入派生视图键的一部分，不靠"season_id是否等于0"这一隐式约定单独推断（该判断依据来自配置表而非事件本身，避免事件schema本身携带隐含业务语义耦合）。

---

## 4. 算法详细设计

### 4.1 `RankingViewUpdater`增量更新与乱序覆盖防护（落实RGS-BAS-014§2.3/§2.3.1）

```rust
fn on_ranking_score_changed(event: &RankingScoreChanged, view: &mut DerivedRankingView) -> Result<(), RankingUpdateError> {
    let key = format!("ranking:{}:{}", event.dimension_id, event.season_id);  // 对应§2.2既定键格式
    let member_key = (event.player_id.clone(), event.dimension_id.clone());

    // 幂等: 以player_id+dimension为幂等键，重复投递(至少一次投递语义下的正常现象)不产生重复排序副作用——
    // 本函数本身对同一(player_id, dimension, new_score)组合的重复调用是幂等的(直接SET而非INCR)，
    // 无需额外去重表，"幂等"体现在操作本身的性质而非额外状态记录
    let last_update_ts = view.get_last_event_ts(&member_key).unwrap_or(0);
    if event.event_occurred_at_ms <= last_update_ts {
        // §2.3.1既定: 死信重放导致的乱序到达，仅当事件时间戳晚于视图当前记录的最后更新时间才写入，
        // 防止死信重放覆盖新数据——本行是该防护的直接实现，不是可选优化
        return Ok(());
    }

    view.set_member_score(&key, &event.player_id, event.new_score)?;  // 局部操作，不全量重算(§2.3既定)
    view.set_last_event_ts(&member_key, event.event_occurred_at_ms);
    view.record_last_update_lag_ms(now_ms() - event.event_occurred_at_ms);  // 供§2.5滞后监控读取
    Ok(())
}
```

### 4.2 视图重建（落实§2.3.1"运维/告警响应后可触发的兜底恢复手段"）

```rust
fn rebuild_ranking_view(dimension_id: &str, config: &RankingDimensionConfig) -> Result<(), RankingUpdateError> {
    mark_view_rebuilding(dimension_id);  // 重建期间该维度查询降级提示"数据更新中"(§2.3.1既定)，而非报错

    // 从权威数据源(source_context指向的EC/GD/MT)全量重算，而非从事件流重放——
    // 事件流本身可能包含已被后续更晚事件覆盖的中间态，全量重算直接以当前权威表状态为准更可靠
    let authoritative_scores = query_authoritative_scores(config.source_context, dimension_id)?;
    let new_view = DerivedRankingView::from_full_snapshot(authoritative_scores);
    swap_view_atomically(dimension_id, new_view);  // 原子替换，避免"重建过程中部分查询命中新视图部分命中旧视图"的混合态

    mark_view_ready(dimension_id);
    Ok(())
}
```

### 4.3 `QuestConditionSubscriber`触发条件匹配（落实RGS-BAS-014§3.2/§3.3）

```rust
fn on_domain_event(event: &DomainEvent, quest_defs: &[QuestDefinition]) -> Result<(), QuestEngineError> {
    for def in quest_defs.iter().filter(|d| condition_references_event(&d.trigger_condition, event.event_type())) {
        // condition_references_event: 声明式表达式解析器判断该定义是否订阅本事件类型，
        // 新增触发条件类型仅需扩展表达式语法(§3.3既定)，本函数本身不因新增quest_id而修改
        if evaluate_trigger_condition(&def.trigger_condition, event)? {
            // 异步更新进度(FR-GSM-015，不阻塞事件产生方)——本函数运行于独立消费者，非事件产生方的同步调用链内
            update_quest_progress(event.character_id(), &def.quest_id, event)?;  // OCC更新quest_progress.version(§2)
        }
    }
    Ok(())
}
```

### 4.4 `PlayerReport`去重键构造与信誉度折算（落实RGS-BAS-014§5.1/§5.1.1）

```rust
fn build_report_dedup_key(reporter_id: PlayerId, target_id: PlayerId, window: Duration, now: Instant) -> String {
    let window_bucket = now.as_secs() / window.as_secs();  // 滚动窗口分桶: 同一bucket内的重复举报映射到同一dedup_key
    let raw = format!("{}:{}:{}", reporter_id, target_id, window_bucket);
    sha256_hex(raw.as_bytes())
}

fn submit_report(req: &SubmitReportRequest, reputation: &ReporterReputation) -> Result<ReportId, ReportError> {
    let dedup_key = build_report_dedup_key(req.reporter_id, req.target_id, REPORT_DEDUP_WINDOW, Instant::now());
    let signal_weight = 1.0 * reputation.weight_multiplier;  // 折算，§5.1.1既定
    let insert_result = insert_player_report(req, &dedup_key, signal_weight);
    match insert_result {
        Err(DbError::UniqueViolation) => Err(ReportError::DuplicateInWindow),
        // uq_player_reports_dedup_key唯一索引冲突: 同一滚动窗口内的重复举报，数据库层直接拒绝(FR-GSM-033)
        other => other.map(|id| id).map_err(Into::into),
    }
}
```

---

## 5. TBD-GSM参数与公式初始提案

RGS-BAS-014§5.1.1"`weight_multiplier`...按`substantiated_count`/`unsubstantiated_count`比例周期性重算（详细算法留待详细设计，本表仅定义数据结构与边界）"——本文档在详细设计阶段给出以下初始提案（同RGS-DTL-025§5同类"提案默认值，非最终值"处理方式）：

| 参数/公式 | 提案默认值 | 依据 |
|---|---|---|
| 重算触发时机 | GM在`AdminService`对举报作出处置决定时异步触发（RGS-BAS-014§5.1.1已定，非本文档新增） | — |
| `weight_multiplier`重算公式 | `clamp(0.1, 1.5, 1.0 + 0.1 * substantiated_count - 0.15 * unsubstantiated_count)` | 属实举报小幅提升权重（每次+0.1，避免单次大幅波动），不实举报惩罚力度略高于奖励力度（每次-0.15），抑制"刷举报次数拉高信号强度"这一潜在滥用路径；clamp边界与§2既有CHECK约束`[0.1, 1.5]`一致 |
| 冷启动初始值 | `weight_multiplier=1.00`（`substantiated_count=unsubstantiated_count=0`） | 新举报者默认中性权重，不预先假设善意或恶意 |
| 重算周期性批量校正 | 提案：每周额外做一次全量批量重算（而非仅事件触发式），避免长期不活跃举报者的权重因个别历史记录固化不再随近期行为调整 | 与ARC-016热更新配置无关，属独立定时作业，复用既有调度基础设施 |

以上均为初始提案，非最终值，须与运营团队评审确定后方可作为正式生效公式，评审结论回写本文档新版本。

---

## 6. 本文档的覆盖范围与后续计划

本文档覆盖：`ranking_dimension_configs`/`quest_definitions`/`quest_progress`/`mail_messages`/`player_reports`/`reporter_reputations`/`player_blocklists`七表物理DDL（含唯一索引/CHECK约束/月度分区）、`RankingScoreChanged`事件具体线格式、`RankingViewUpdater`增量更新与乱序覆盖防护、视图重建、`QuestConditionSubscriber`触发匹配、举报去重键构造与信誉度折算的完整伪代码、`weight_multiplier`重算公式的初始提案。

本版本明确不覆盖、留待后续：

- TBD-GSM-001（派生视图存储选型）的最终决议——本文档§3/§4.1沿用RGS-BAS-014§2.2既定"复用ARC-012有序集合能力"默认方案描述事件消费侧的行为，但该选型本身仍待评审，若最终选型的具体存储介质API与本文档`view.set_member_score`等抽象操作的实际映射方式不同，需在选型确定后回写本文档新版本对齐。
- TBD-GSM-002（赛季继承规则：清零/按比例保留/软重置区间）的具体公式——本文档未涉及RGS-BAS-014§6.2赛季结算时序的物理落地，因该时序的原子提交实现依赖继承规则本身先确定，继承规则未定则无法给出确定性伪代码，留待TBD-GSM-002决议后补充本文档新版本。
- TBD-GSM-003（举报处理SLA数值）——纯运营流程时限参数，与本文档物理设计无直接耦合，不展开。
- `quest_definitions.trigger_condition`声明式表达式的具体语法（操作符集合、字段引用语法）——RGS-BAS-014§3.3已述"新增操作符/字段"式扩展方式，具体初始语法集合属实现阶段的语言设计，非架构层面决策，本文档不给出完整BNF。
- 赛季边界原子切换（RGS-BAS-014§6.2/§6.3）的具体事务实现——依赖TBD-GSM-002未决，见上，一并留待后续版本。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-014§2.1〜2.2 排行榜组件划分与派生视图数据结构 | §2、§3 |
| RGS-BAS-014§2.2.1 RankingDimensionConfig | §2 |
| RGS-BAS-014§2.3 更新时序 | §3、§4.1 |
| RGS-BAS-014§2.3.1 异常分支（死信/视图重建） | §4.1、§4.2 |
| RGS-BAS-014§2.4〜2.5 一致性边界与滞后监控 | §4.1（`record_last_update_lag_ms`） |
| RGS-BAS-014§3.1〜3.3 任务/成就配置化触发引擎 | §2、§4.3 |
| RGS-BAS-014§4.1〜4.3 邮件系统数据模型 | §2 |
| RGS-BAS-014§5.1 举报字段级设计 | §2、§4.4 |
| RGS-BAS-014§5.1.1 举报者信誉度 | §2、§4.4、§5 |
| RGS-BAS-014§5.2 黑名单 | §2 |
| RGS-BAS-014§6 赛季与段位重置时序 | §6（明确排除，依赖TBD-GSM-002未决） |
| RGS-DTL-001§3.2 确定请求API物理执行语义 | §2（`quest_progress.version`同款OCC模式） |
| RGS-DTL-025§5（提案默认值处理方式的既定先例） | §5 |
