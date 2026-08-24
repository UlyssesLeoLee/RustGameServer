# 详细设计书（詳細設計書 / Detailed Design Document）

**匹配系统：队列/评分物理数据库设计・事件线格式・扩圈与跨分片撮合算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-026 |
| 版本 | 0.4 |
| 父文档 | RGS-BAS-026 匹配系统 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17（v0.1）/ 2026-08-25（v0.4 升版） |
| 制定者 | 架构师（v0.4 由 Ulysses per DEC-008 派生子代理 WF-1-55.42 升版） |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档为第四份详细设计文档）。细化RGS-BAS-026§3.1逻辑数据模型为MT限界上下文（`match_db`同库）内`queue_entries`／`match_ratings`／`match_quality_metrics`三表具体DDL、§4.1扩圈算法与§5.1跨分片OCC校验落实为可直接翻译为Rust实现的伪代码、§7各时序图落实为具体状态转移代码、事件落实为具体线格式。**本版本不覆盖**：评分算法本身（ELO/Glicko-2/TrueSkill）的最终选型与具体公式实现（RGS-REQ-029§11已标注为TBD，需另行ADR决定后再补充本文档）、GM/运营配置后台的UI细节。见§7 | 全部 |
| 0.2 | 2026-08-17 | 架构师 | — | 负责人指示"开子代理完成剩余的"（技术选型TBD收尾）。新增§7解决评分算法最终选型（Glicko-2，排除TrueSkill因IP历史模糊性、排除纯ELO因无不确定度建模），给出`RatingSettlement.calculate()`核心公式；`match_ratings`新增`volatility`列。原§7覆盖范围章节顺延为§8并更新内容 | §1.2、§2（`match_ratings`新增列）、§7（新增）、原§7→§8 |
| **0.3** | 2026-08-20 | 架构师 | — | 修正 Glicko-2 结算只返回/持久化 rating、RD 而遗漏 volatility 的不一致：`RatingUpdate` 现返回三项状态；新增结算幂等回执表，并规定 rating/RD/volatility、回执和 Outbox 在同一事务提交 | §2、§3、§7、§8 |
| **0.4** | 2026-08-25 | Ulysses（per DEC-008 派生子代理 WF-1-55.42） | — | per RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10：§4.1 补三段（n≤500 临时占位 / 降级策略 / benchmark 子任务），落实 O(n²) 撮合复杂度的性能边界与降级/熔断路径；`RatingSettlement.calculate()` 公式与 `match_ratings` schema 未变（v0.3 已稳定），不重复修改 | §4.1（新增 §4.1.1/§4.1.2/§4.1.3） |

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
2. [物理数据库设计：MT限界上下文匹配三表及结算幂等表](#2-物理数据库设计mt限界上下文匹配三表及结算幂等表)
3. [事件线格式](#3-事件线格式)
4. [扩圈算法详细设计](#4-扩圈算法详细设计)
5. [跨分片OCC校验详细设计](#5-跨分片occ校验详细设计)
6. [排队/确认/回填状态转移详细设计](#6-排队确认回填状态转移详细设计)
7. [评分算法选型（Glicko-2）](#7-评分算法选型rgs-req-02911-tbd最终决定)
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

## 2. 物理数据库设计：MT限界上下文匹配三表及结算幂等表

对应RGS-BAS-026§3.1/§3.2。业务三表与结算幂等回执表均位于`match_db`（同一MT限界上下文事务边界），本文档只新增表结构。

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
    rating_deviation       DOUBLE PRECISION NOT NULL DEFAULT 350.0 CHECK (rating_deviation > 0), -- Glicko-2 RD，不允许空值
    volatility              DOUBLE PRECISION NOT NULL DEFAULT 0.06 CHECK (volatility > 0), -- Glicko-2波动率σ，见§7
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

-- 评分结算幂等回执：同一对局、角色、模式只允许写入一次；保存首次计算的三项状态供安全重试返回
CREATE TABLE rating_settlement_receipts (
    match_ref           BIGINT NOT NULL,
    character_id        BIGINT NOT NULL,
    mode                TEXT NOT NULL,
    input_hash          BYTEA NOT NULL,       -- 对局结果、对手集合和算法参数的规范化摘要
    rating_value        DOUBLE PRECISION NOT NULL,
    rating_deviation    DOUBLE PRECISION NOT NULL CHECK (rating_deviation > 0),
    volatility          DOUBLE PRECISION NOT NULL CHECK (volatility > 0),
    settled_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (match_ref, character_id, mode)
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
  double rating_deviation     = 6;
  double volatility           = 7;
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

### 4.1.1 n≤500 临时占位（per RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10）

§4.2 单轮撮合 `matchmaker_tick` 的候选筛选为 O(n²)（`candidates.iter().filter(...).collect()` 全表扫描 + 双重循环），在 n 较大时存在性能风险，但本文档不规定架构层面的硬上限。**占位上限 n≤500** 仅供 PH-1 编码完成、benchmark 跑通前作为实现侧的临时门禁使用，**不**是性能承诺，也不**是**最终交付值。推导如下：

- NFR-PT 单局撮合决策延迟 p99 < 100ms（per RGS-REQ-029 §NFR + RGS-BAS-026 §4.1 既定）。
- 占位 n=500 折算：100ms / 500² ≈ 0.4μs/pair，对纯内存 candidate scan + tolerance 过滤 + `try_compose_teams` 组合搜索（仅算术比较 + HashSet 插入）属于宽松量级；该上限在现代 x86_64 / ARM64 CPU 上有 1〜2 个数量级的 headroom。
- 但**实际** n 上限取决于：CPU 微架构、Cache 命中率（500 个 `QueueEntry` 实体大小约 8-16KB 总集，冷启动时 L2 命中率高；>2000 时开始 L3 miss）、`try_compose_teams` 内部组合搜索的复杂度（贪心 vs 回溯未指定）、Rust 编译参数（`opt-level=3` + LTO 与否）、共享平台 tokio runtime 调度抖动。
- 因此**n≤500 是占位，PH-1 实测后用 benchmark 数据替换为实测值**。Q-D-10 答复明确"PH-1 启动后跑 benchmark 才能给出可信 n 上限，不应拍脑袋定数字"——本占位严格遵循此原则，仅在 benchmark 数据未出前为实现侧提供最坏情况下限。
- 占位 n 写入 `config/match-service.toml` 字段 `matchmaking.max_candidates_per_tick: 500`，实现侧启动期加载；benchmark 报告（`docs/deploy/matchmaking-bench-report.md`）出实测值后，由实现侧按 §8 后续计划触发 `config` 调整。
- 占位 n 的**作用域**仅是"单轮撮合 tick"维度（§4.2 `matchmaker_tick` 调用范围），不涉及跨 tick 累积、跨分片全局候选池大小。跨分片 POOL_SHARED 模式下，单分片承担的 `candidates` 由 NATS 广播带宽与 origin_shard 路由策略决定，另行评估，**不**在本文档 n≤500 占位的覆盖范围。
- 修订历史将追踪占位 → 实测的切换事件：v0.5 之后若 benchmark 给出 n=1200 p99 < 100ms，则更新 v0.5 字段为 `max_candidates_per_tick: 1200` 并在修订历史注明"per benchmark 报告 YYYY-MM-DD 实测 n=1200"。

**占位 n 的数值依据（量级推导）**：

- 100ms / 500² ≈ 0.4μs/pair 是**单对候选对比**的理论上限，不是端到端 `matchmaker_tick` 函数的全部耗时。实际端到端耗时还需叠加：(a) `candidates` 排序（O(n log n)），n=500 时 ~4.5K 比较，约 5-10μs；(b) `tolerance()` 重复调用（每个 entry 调一次，共 500 次），每次 ~1-2μs；(c) `try_compose_teams` 内部组合搜索（贪心策略下约 O(n×k)，k 是兼容候选数，O(n²) 最坏），500 entry 时最坏 ~125K 操作，约 200-500μs；(d) `consumed: HashSet` 插入/查询，500 次约 50-100μs。合计：1ms 量级，距 100ms 阈值有 100× headroom。
- 但 1ms 是**冷启动 + 完美 cache 命中**的乐观估计。生产环境共享平台 tokio runtime 调度抖动 + L2/L3 cache 抖动 + DB 连接池等待（即便 `candidates` 已从 DB 拉到内存）通常额外增加 5-20ms。**实际** 单轮 `matchmaker_tick` 端到端 100ms 上限对应的纯算法时间是 80-95ms。
- 因此 0.4μs/pair 的"宽松量级"判断成立，但**仅**对纯算法路径成立；含调度 + cache + GC（无 GC，但 tokio 调度抖动类似）后 0.4μs/pair 退化为 ~160μs/pair（80ms / 500²），仍是宽松量级。**仅**当 n=2000 时 0.4μs/pair → 1.6ms/pair × 4M pair = 6.4s，**远远**超 100ms——这就是 §4.1.2 降级策略必须存在的工程原因。

**与 NFR-PT 既定值的对照**：

- NFR-PT 单局决策 ≤ 100ms 包含**两段**：①撮合计算（本文档 §4 `matchmaker_tick`）≤ 80ms（实测经验值，给下游 DB 写留 20ms）；②DB OCC 校验 + 状态机更新（§5 + §6）≤ 20ms。两者合计 100ms 上限。
- §4.1.1 占位 n≤500 针对**第①段**（撮合计算），不含 DB；DB 段不在本节讨论。
- 当 n=500 时 ①段实测若 < 50ms，则 ②段 50ms 预算足够，NFR-PT 通过；当 n=1000 时 ①段若仍 < 50ms（线性外推不成立，实际可能 80-150ms），则 ②段被挤占 0-30ms，需降级到拆分撮合轮（§4.1.2）以确保 ①段始终 ≤ 50ms。
- 因此 §4.1.1 占位 500 是"①段 50ms headroom 对应的 n 值"的**保守选择**，不是"100ms 总预算对应的 n 值"的极限值。**保守**意味着：若实测 n=700 时 ①段仍 50ms headroom，可放心扩到 700；若实测 n=300 时 ①段已超 50ms（说明 `try_compose_teams` 内部策略较重），应缩到 300 并触发 §4.1.2 拆分撮合轮路径。

**与既有 G-005 清理模式的交互**：

- §2 `queue_entries` 短生命周期表的清理策略遵循 G-005，谓词 `status IN ('CONFIRMED', 'ABANDONED') AND enqueued_at < now() - retention_period`。
- `max_candidates_per_tick=500` 占位对 G-005 无直接依赖：G-005 控制"已结束条目归档"，§4.1.1 控制"单 tick 处理 WAITING 条目数"，两者作用域正交。
- 但 `WAITING` 条目数长期 > 500 时（业务侧短时间无撮合匹配成功），§4.1.2 拆分撮合轮可降低单轮压力，配合 G-005 不能解决"长时间 WAITING"问题（WAITING 不在 G-005 清理谓词内）。**若生产监控发现 `WAITING` 队列长期 > 500**，需另行考虑：(a) 缩短 `tolerance()` 的 `grace_period_secs`（§4.1 既定 30s）让容差更快扩宽，撮合成功率上升；(b) 提升 `MatchmakerWorker` tick 频率（当前 TBD，留待实现阶段调优）。两者均不修改 §4.1.1 占位本身。

### 4.1.2 降级策略（per RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10）

n 超占位上限（500）时**优先降级**而非熔断，降级路径如下，按顺序执行：

1. **第一步：拆分撮合轮（分桶降级）**
   - 单轮 `candidates` 按 `composite_rating` 排序后，按桶大小 `n'` 切片为 `ceil(n / n')` 个子轮。
   - 每个子轮独立调用 `matchmaker_tick`，互不共享 `consumed: HashSet<EntryId>`，避免桶间跨边界误撮合。
   - 桶大小 `n'` 初值取 `500`（与 §4.1.1 占位一致），每子轮 p99 仍应 < 100ms；子轮之间无强延迟预算（总延迟 = 子轮数 × 单子轮延迟）。
   - **降级触发条件**：`candidates.len() > max_candidates_per_tick`（500 占位）。
   - **降级退出条件**：n' 自适应调小到 §4.1.1 上限的 1/4（125）仍不满足时，进入第二步熔断；n' 自适应调大到上限的 2 倍（1000）且实测满足时，回写 §4.1.1 配置。

2. **第二步：熔断（仅降级后仍超时才触发）**
   - 单个子轮执行超过 NFR-PT 100ms 阈值（10% 滑动窗口内连续 3 次超阈），判定该子轮 O(n²) 实际不可承受。
   - 熔断动作：
     - 返回 `MMError::CircuitOpen { retry_after_ms }` 给 `MatchmakerWorker` 调用方。
     - `retry_after_ms` 初值 = 子轮耗时实测 p99 × 4（背压值，避免立即重试再次熔断），上限 30s（避免客户端长时间空等）。
     - 已分配的 `consumed` 条目**不释放**（避免被其他 tick 重复撮合）；熔断恢复后该子轮从断点继续。
   - 熔断期间产生的客户端错误：上游 gateway 返回 HTTP 503 + `Retry-After: <retry_after_ms>`，客户端遵循标准退避；不直接返回 500，避免客户端立即重试放大压力。

3. **降级优先于熔断的工程理由**
   - 熔断对客户端可见，是 SLO 破坏事件（HTTP 503），需要事后 SRE 介入排查。
   - 拆分撮合轮对客户端透明（仍返回撮合结果，只是延迟从 100ms 变成 n×100ms ≈ 200-400ms），属于内部降级而非 SLO 破坏。
   - 拆分撮合轮的延迟增长（n/500 倍）远低于熔断+重试的雪崩成本，符合 RGS-NFR §可降级原则。

4. **降级与 PH-5 数据回写的耦合**
   - 拆分撮合轮的桶大小 `n'` 在 benchmark 报告出实测 n 上限后重写：若实测 n=2000，则 `n' = 2000`，单子轮仍 100ms 上限；总延迟 = ceil(n/2000) × 100ms，n=10000 时总延迟 ≤ 500ms（5 子轮）。
   - 若实测 n=200，则 `n' = 200`，单子轮仍 100ms 上限；n=1000 时需 5 子轮，总延迟 500ms；此时考虑引入 §4.1.3 benchmark 子任务的"预过滤"（按 `composite_rating` 直方图先粗筛 top-K）作为 n' 自适应外的第二道降级路径，但**当前 v0.4 不引入预过滤**，留待 v0.5+ 视 benchmark 数据决定。

5. **降级路径与本文档其他章节的接口边界**
   - 不修改 §4.2 `matchmaker_tick` 签名（业务实现侧加 `matchmaker_tick_with_bucket_size(mode, scope, n')` 重载）。
   - 不修改 §5 OCC 校验（拆分撮合轮的子轮各自独立跑完 OCC，不共享 `consumed`）。
   - 不修改 §6 状态机（拆分撮合轮对 `queue_entries.status` 的写入与单轮一致）。
   - 仅在 `MatchmakerWorker` 调度层加降级逻辑（split-by-bucket），属于实现层细节，不在本文档详细化。

### 4.1.3 benchmark 子任务（per RGS-OPEN-QA-001 Q-D-10 + ACTIONS-v0.3 A-10）

**Q-D-10 答复的硬性约束**：可信 n 上限只能由 benchmark 实测给出，不应拍脑袋定数字。本节落实该约束为 benchmark 任务的契约。

1. **测试目标**
   - 被测对象：match-service 撮合核心函数（per §4.2 `matchmaker_tick` 的 O(n²) 候选筛选路径）。
   - 测试输入：n ∈ {100, 200, 500, 1000, 2000} 共 5 档，各档生成 100 iteration 的随机 `composite_rating` 分布（高斯分布，μ=1500，σ=200，模拟真实玩家评分散布）。
   - 测试输出：每档 p50 / p95 / p99 延迟（criterion.rs 原生支持，无需手写分位数计算）。

2. **方法**
   - 工具：criterion.rs（Rust 标准 benchmark 框架，per `crates/match-service/benches/matchmaking_bench.rs`）。
   - 执行命令：`cargo bench -p match-service --bench matchmaking_bench`。
   - 报告输出：criterion 自动生成 HTML 报告到 `target/criterion/matchmaking_tick/`，Markdown 摘要由实现侧人工整理到 `docs/deploy/matchmaking-bench-report.md`。
   - 环境：单线程 + `opt-level=3` + LTO（per workspace `[profile.release]` 配置），冷启动 + 热身 3s 后采样。

3. **断言**
   - n ≤ 500 时 p99 < 100ms（per NFR-PT 单局决策 ≤ 100ms）—— **硬性断言**；失败时 CI 拒绝合入 benchmark 重写。
   - n > 500 时**不**做硬性断言，**仅**记录实测 p50/p95/p99，作为 §4.1.1 占位 → 实测切换的数据来源。
   - 测试通过标准：所有 5 档 benchmark 跑完无 panic、criterion 报告生成成功、`docs/deploy/matchmaking-bench-report.md` 包含 5 档 p99 实测值。

4. **任务边界**
   - 本任务（v0.4）**仅搭 benchmark 框架**：写 `matchmaking_bench.rs` 占位实现（自包含的 §4.2 算法 stand-in），加 criterion dev-dep，确保 `cargo check -p match-service` pass。**不**实跑 `cargo bench`——理由是 `matchmaker_tick` 实际实现要到 PH-1 编码完成后才有，PH-1 之前跑出的数据无意义（测的是 stand-in，不是真实实现）。
   - 实跑任务挂到 PH-1 之后的 L4 任务（`WF-?-??.??` 编号预留，per WBS v0.7+），不在本任务（WF-1-55.42）范围内。
   - 本任务交付的 `docs/deploy/matchmaking-bench-report.md` 标注"待 PH-1 实跑后填入实测值"为占位报告，**不**构成性能数据；引用此报告前必须确认状态从"待实跑"切换为"已实跑"。

5. **不覆盖范围**
   - 不覆盖 `try_compose_teams` 内部组合搜索策略（贪心 vs 回溯）的 benchmark；该策略在 §4.2 已声明"留待实现阶段按性能实测选择"，不在本任务边界。
   - 不覆盖跨分片 POOL_SHARED 模式下的 NATS 广播 + 多分片并行撮合 benchmark；该场景在 §5 跨分片 OCC 校验中涉及，单独留待 WF-?-??.?? 任务。
   - 不覆盖真实玩家流量回放 benchmark（需生产数据脱敏后导入，PH-2 之后才有可能）。

6. **与降级策略的耦合**
   - benchmark 给出 n=1200 p99 < 100ms → §4.1.1 占位扩到 1200，§4.1.2 桶大小 `n'` 同步扩到 1200，§4.1.3 断言阈值保持 n ≤ 1200。
   - benchmark 给出 n=300 p99 > 100ms → §4.1.1 占位缩到 300，§4.1.2 桶大小 `n'` 缩到 300，**且** §4.1.2 第一步降级（拆分撮合轮）成为默认路径而非"超限才触发"，即 `max_candidates_per_tick` 与 `n'` 同值时无降级，n > `n'` 时降级——这是占位和实测一致时的边界情况，无需额外处理。
   - benchmark 给出 n=2000 p99 > 100ms → §4.1.1 占位缩到 1000（p99 < 100ms 的最大 n），§4.1.2 桶大小 `n'` 同步，n > 2000 触发第一步降级拆分。
   - 三种情况均在 `docs/deploy/matchmaking-bench-report.md` 结论部分给出"工程推荐 n 上限"与"降级路径是否需要调整"两栏。

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
    Ok(finalize_match(proposal))  // 全部条目OCC通过后才真正进入§6 MATCH创建路径
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

回填（§7.4）在既有对局参与者提前退出事件触发后，`BackfillWorker`按§4扩圈算法同一套`tolerance`/撮合尝试逻辑在匹配池中寻找候选，唯一差异是撮合目标不是"创建新`MATCH`"而是"追加进已存在`MATCH_PARTICIPANT`"，故本文档不重复给出伪代码，仅声明复用关系：`BackfillWorker`调用与§4.2相同的`try_compose_teams`（`roster_size=1`，即只需补齐一个空位而非整队），成功后写入`MATCH_PARTICIPANT`而非走§6创建路径。

---

## 7. 评分算法选型（RGS-REQ-029§11 TBD，最终决定）

选型为**Glicko-2**（Mark Glickman发表的公开算法，含官方实现指南，无专利/许可争议，长期被开源棋类/竞技游戏服务器采用），而非ELO或TrueSkill：

- **排除TrueSkill**：其原始实现与专利历史关联微软研究院，即便published math本身可重新实现，"全部采用开源免费策略"约束下应避免任何存在IP历史模糊性的选项，优先选择零IP争议的方案——Glicko-2满足此条件，ELO同样满足但精度不足（见下）。
- **排除纯ELO**：ELO不建模"评分不确定度"，新玩家/久未匹配玩家的评分收敛速度慢，且无法自然表达"评分差不大但把握程度不同"的两个玩家；Glicko-2的`rating_deviation`字段直接解决此问题，与§2 `match_ratings`表早已预留的`rating_deviation`列（RGS-BAS-026§3.1设计时即声明"若算法需要不确定度则用此字段，若不需要则恒为空"）完全吻合，无需改动物理schema。
- **参数**：初始`rating_value=1500`，初始`rating_deviation=350`，系统常数`τ=0.5`（Glickman官方指南推荐范围0.3〜1.2，游戏竞技场景取偏保守的中间值），评分稳定期后`rating_deviation`收敛下限设为`30`（低于此值不再继续收窄，防止长期活跃玩家的不确定度归零后对评分微小波动过度敏感）。

`RatingSettlement.calculate()`伪代码（对应RGS-DTL-026§7.1的幂等结算与原子持久化路径，填入此前留空的计算逻辑）：

```rust
// Glicko-2 标准更新公式的直接翻译；调用者必须持久化全部三项状态。
struct RatingUpdate {
    rating_value: f64,
    rating_deviation: f64,
    volatility: f64,
}

fn glicko2_update(player: &MatchRating, opponent: &MatchRating, outcome: MatchOutcome, tau: f64) -> RatingUpdate {
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

    let (rating_value, rating_deviation) =
        from_glicko2_scale(new_mu, new_phi.max(MIN_RATING_DEVIATION_INTERNAL_SCALE));
    RatingUpdate { rating_value, rating_deviation, volatility: new_sigma }
}
```

`solve_new_volatility`（波动率迭代求解）与`to_glicko2_scale`/`from_glicko2_scale`（内部标度换算）为Glicko-2标准算法的固定组成部分，直接照官方指南实现，不属于本项目自定义逻辑，故本文档不展开其内部细节，仅确认调用契约。组队场景（多人对多人）的评分更新按"每个成员视为独立与对方队伍`composite_rating`对局一次"简化处理（Glicko-2原生为1v1设计，团队场景的精确扩展本身是另一层不确定的研究问题，简化处理是本项目的**有意选择**而非疏漏，效果留待PH-5数据评估是否需要更精细的团队评分扩展）。

### 7.1 幂等结算与三项状态原子持久化

`RatingSettlement` 的幂等键为 `(match_ref, character_id, mode)`，并绑定对局结果、对手集合和算法参数的 `input_hash`。计算、`match_ratings` 行锁更新、`rating_settlement_receipts` 插入、`MatchRatingChanged` Outbox 写入必须在同一个 `match_db` 事务中完成；不得先发布事件再写波动率，也不得在事务外单独保存 RD/σ。

```rust
async fn settle_rating_once(
    tx: &mut Transaction<'_>, match_ref: MatchRef, player_id: CharacterId, mode: &str, input: SettlementInput,
) -> Result<RatingUpdate, SettlementError> {
    let input_hash = input.canonical_sha256();
    if let Some(receipt) = tx.find_rating_settlement(match_ref, player_id, mode).await? {
        ensure!(receipt.input_hash == input_hash, SettlementError::ConflictingReplay);
        return Ok(RatingUpdate::from(receipt)); // 安全重试返回首次的 rating/RD/volatility
    }

    let player = tx.lock_match_rating(player_id, mode).await?; // SELECT ... FOR UPDATE
    let opponent = tx.lock_match_rating(input.opponent_id, mode).await?;
    let update = glicko2_update(&player, &opponent, input.outcome, GLICKO_TAU);
    tx.update_match_rating(player_id, mode, update.rating_value, update.rating_deviation, update.volatility).await?;
    tx.insert_rating_settlement(match_ref, player_id, mode, input_hash, &update).await?;
    tx.append_match_rating_changed_outbox(match_ref, player_id, mode, &update).await?;
    Ok(update)
} // 由调用方在此处提交；任一步失败整笔事务回滚
```

对同一幂等键但不同 `input_hash` 的请求，事务必须拒绝并记录结算冲突审计，不能重算或覆盖既有评分。新建评分行使用 §2 的 `1500/350/0.06` 初始状态；迁移存量 NULL RD 前必须回填为 `350.0` 并通过约束校验，再启用 `NOT NULL`。

license确认：Glicko-2算法本身为公开发表的数学方法，非专利，实现代码在Rust中从零编写，无第三方依赖引入，无CON-001顾虑。

---

## 8. 本文档的覆盖范围与后续计划

本文档覆盖：MT限界上下文匹配三表及`rating_settlement_receipts`幂等表物理DDL、`QueueEntryCreated`/`QueueEntryStatusChanged`/`MatchRatingChanged`三个事件的具体线格式、扩圈容差函数与单轮撮合尝试伪代码、跨分片OCC"全有或全无"提交逻辑、排队/确认/回填的完整状态转移表与对应实现、**评分算法最终选型（Glicko-2）、`RatingSettlement.calculate()`核心公式以及 rating/RD/volatility 的同事务幂等持久化**。

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
| RGS-BAS-026§6.1 结算路径 | §7（Glicko-2 计算与三项状态的幂等持久化） |
| RGS-BAS-026§6.2 与GSM展示排行联动 | §3（`MatchRatingChanged`） |
| RGS-BAS-026§7.1〜7.5 连败保护/放弃/确认/回填/组队信号 | §6 |
| RGS-DTL-002（挂载脚手架物理落地） | 前提依赖，本文档假定MT域已按RGS-DTL-002完成挂载 |
