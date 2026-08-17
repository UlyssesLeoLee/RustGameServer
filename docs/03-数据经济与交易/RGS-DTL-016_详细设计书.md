# 详细设计书（詳細設計書 / Detailed Design Document）

**客服工单与支付对账：物理数据库设计・对账批处理算法・SLA状态机详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-016 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-016 客服工单与支付对账 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档接续RGS-DTL-001/002/025/026/027批次，与RGS-DTL-007/015同批次产出）。细化RGS-BAS-016§2.2数据模型与§2.3状态机为AD限界上下文内`support_tickets`表具体DDL、§3.1数据模型为AD/EC共用`payment_orders`表具体DDL（含RGS-BAS-016§0.3已确认的跨文档权威字段清单四字段）、§3.2对账批处理时序与§3.3异常分支落实为可直接翻译为Rust实现的伪代码（含RSK-SUP-002"比对条件写反"防护的具体双重校验实现）、SLA超时检测落实为具体扫描算法（含TBD-SUP-001/002两项参数默认值提案沿用RGS-BAS-016原文既定建议值）。**本版本不覆盖**：`TicketEscalationNotifier`告警推送的具体消息模板、支付服务商对账文件/API的具体解析适配代码（因服务商各异，属实现阶段各自适配范畴）。见§5 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-016§2.2/§3.1逻辑字段表一致，`payment_orders`是否完整覆盖§0.3声明的四个跨文档扩展字段 |
| 评审（DBA） | | | `(provider_txn_id)`唯一索引是否确实承担NFR-SUP-004幂等键角色，对账批处理的UPSERT写法是否规避RSK-SUP-002 |
| 审批（负责人） | | | 本文档的基准化；TBD-SUP-001（SLA分级）/TBD-SUP-002（补偿阈值）默认值提案是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：AD/EC限界上下文工单与订单两表](#2-物理数据库设计adec限界上下文工单与订单两表)
3. [对账批处理算法详细设计](#3-对账批处理算法详细设计)
4. [SLA超时检测与工单状态机详细设计](#4-sla超时检测与工单状态机详细设计)
5. [TBD-SUP参数默认值提案](#5-tbd-sup参数默认值提案)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-016给出了`SupportTicket`/`PaymentOrder`的逻辑字段表、工单状态机迁移条件表、SLA分级基准表、对账批处理的文字流程描述与异常分支处理原则。本文档将其落实为：可直接执行的PostgreSQL DDL、对账批处理主流程与异常分支（服务商侧延迟/比对条件写反防护）的完整伪代码、SLA超时扫描的具体算法，以及两项TBD参数在实现层面的默认值配置提案。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-016已确定的任何结构性选择（工单处理执行权收口于既有`AdminService`、`PaymentOrder`以`(provider_txn_id)`唯一索引作为幂等键与关联键的双重保证、超阈值补偿转人工复核）。
- 不覆盖`TicketEscalationNotifier`告警推送的具体消息文案/模板——复用RGS-BAS-003§6既有告警推送通道，该通道的消息格式属于RGS-BAS-003自身范围，本文档不重复设计。
- 不覆盖支付服务商侧对账文件/API的具体解析适配代码——不同服务商（`app_store`/`google_play`/直连网关）的对账文件格式各异，具体解析属实现阶段各自适配层职责，本文档只覆盖解析完成后、统一为内部标准结构后的比对逻辑（§3）。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准（复用RGS-DTL-007§2既定命名/索引/分区模板句法），算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：AD/EC限界上下文工单与订单两表

对应RGS-BAS-016§2.2/§3.1。`support_tickets`依附既有AD限界上下文（`admin_db`，同RGS-DTL-025§2已建立的库），`payment_orders`依附既有EC限界上下文（`economy_db`，同RGS-DTL-001§3已建立的库），本文档只新增表结构，不新建库。

```sql
-- 客服工单表，对应FR-SUP-001〜007，落位admin_db
CREATE TABLE support_tickets (
    ticket_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id            BIGINT NOT NULL,     -- 逻辑引用player_db.accounts，跨库不建物理FK
                                                 -- （同RGS-DTL-001/025/026既定跨库引用规则）
    category               TEXT NOT NULL
                              CHECK (category IN ('ban_appeal', 'item_anomaly', 'payment_issue', 'other')),
    state                    TEXT NOT NULL DEFAULT '待受理'
                              CHECK (state IN ('待受理', '处理中', '待玩家补充信息', '已解决', '已驳回')),
    sla_deadline               TIMESTAMPTZ NOT NULL,  -- 依category分级计算，见§4.1
    resolution_summary           TEXT NULL,           -- 关闭时的处理结论摘要(FR-SUP-005强制关闭时必须留痕，
                                                         -- 由§4.2状态转移函数在写入前校验非空，非DB层CHECK
                                                         -- 约束——CHECK无法区分"关闭时"与"关闭前"两个时间点)
    admin_action_ref             BIGINT NULL,          -- 逻辑引用AdminService操作记录ID，跨库不建物理FK，
                                                         -- 本表不直接存储执行结果(RGS-BAS-016§2.2原文既定)
    dedup_key                     TEXT NOT NULL,        -- player_id+category+滚动时间窗口哈希，见§4.3
    created_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_support_tickets_player_state ON support_tickets (player_id, state);
    -- 支撑FR-SUP-006"玩家查询自己的工单列表"
CREATE INDEX idx_support_tickets_state_sla
    ON support_tickets (state, sla_deadline)
    WHERE state IN ('待受理', '处理中', '待玩家补充信息');
    -- 部分索引：支撑TicketEscalationNotifier定时扫描临近/超过SLA的工单，
    -- 已终态(已解决/已驳回)记录不占索引空间（同RGS-DTL-015 idx_trade_offers_state_expire同类手法）
CREATE UNIQUE INDEX uq_support_tickets_dedup_key ON support_tickets (dedup_key);
    -- 唯一索引但命中时为提示而非拒绝(§4.3)：应用层捕获唯一约束冲突后，
    -- 查询既有记录并返回"检测到相似工单"提示，而非将约束冲突本身作为拒绝创建的理由，
    -- 这是"数据库层唯一约束"与"业务层提示而非强制拒绝"两个不同层面的组合——
    -- 约束仍在数据库层生效(用于快速命中查找)，但应用层对约束冲突的响应方式是提示，
    -- 不是让约束冲突直接以HTTP 500/数据库错误形式暴露给玩家

-- 支付订单表，对应FR-SUP-010〜015/FR-PLT-003〜005，落位economy_db
-- 本表是PaymentOrder的唯一权威字段清单(RGS-BAS-016§3.1 v0.3已确认)，
-- 含RGS-BAS-020§2.5单向追加、本次同步回本表的四个跨文档扩展字段
CREATE TABLE payment_orders (
    order_id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_txn_id        TEXT NULL,           -- 待支付阶段可能尚无服务商侧交易ID，故允许NULL
    state                    TEXT NOT NULL DEFAULT '待支付'
                              CHECK (state IN ('待支付', '已支付', '已发货', '发货失败', '待补偿', '已补偿')),
    amount                    NUMERIC(18,2) NOT NULL,
    payment_channel             TEXT NOT NULL
                                  CHECK (payment_channel IN ('platform_iap', 'direct_gateway')),
    platform_type                 TEXT NULL
                                  CHECK (platform_type IS NULL OR platform_type IN ('app_store', 'google_play')),
    platform_environment            TEXT NULL
                                  CHECK (platform_environment IS NULL OR platform_environment IN ('sandbox', 'production')),
    refund_status                    TEXT NOT NULL DEFAULT 'none'
                                  CHECK (refund_status IN ('none', 'refunded', 'clawback_pending', 'clawback_done')),
    updated_at                        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at                         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_payment_orders_platform_fields_scope
        CHECK (payment_channel = 'platform_iap' OR (platform_type IS NULL AND platform_environment IS NULL))
        -- platform_type/platform_environment仅在payment_channel='platform_iap'时非空
        -- (RGS-BAS-016§3.1原文"仅payment_channel=platform_iap时非空"的物理强制化,
        -- 逻辑设计原文以自然语言表达此约束,本文档补充为可执行CHECK,不算新决策)
);

-- (provider_txn_id)唯一索引: 幂等键(NFR-SUP-004)与对账关联键的双重保证,允许NULL
CREATE UNIQUE INDEX uq_payment_orders_provider_txn_id
    ON payment_orders (provider_txn_id) WHERE provider_txn_id IS NOT NULL;
CREATE INDEX idx_payment_orders_state_updated
    ON payment_orders (state, updated_at);
    -- 支撑异常分支扫描长时间停留在非终态的订单
CREATE UNIQUE INDEX uq_payment_orders_platform_txn
    ON payment_orders (platform_type, provider_txn_id)
    WHERE platform_type IS NOT NULL AND provider_txn_id IS NOT NULL;
    -- RGS-BAS-020§2.5扩展的复合唯一索引，确保跨平台交易标识不产生误关联
```

**与逻辑设计的对应关系**：`payment_orders`的`platform_type`/`platform_environment`/`refund_status`/`payment_channel`四字段是RGS-BAS-016§0.3修订历史中明确记载的"跨文档字段清单同步"结果——本文档DDL直接采纳该同步结果作为唯一权威定义，不重新讨论字段是否应归属本表，这是对RGS-BAS-016已完成的跨文档协调的物理落实，非本文档新决策。

---

## 3. 对账批处理算法详细设计

对应RGS-BAS-016§3.2/§3.3文字流程，落实为伪代码，覆盖RGS-BAS-016§3.3已列出的两类异常场景。

### 3.1 主流程

```rust
// 定时任务触发(周期见NFR-SUP-002,本文档不重新设定具体周期数值，沿用该NFR既定要求)
fn reconciliation_job_run(window: TimeRange) -> Result<ReconciliationSummary, ReconError> {
    // 服务商侧记录拉取(具体解析适配不在本文档范围，见§1.2)
    let provider_records = match fetch_provider_records(window) {
        Ok(records) => records,
        Err(fetch_err) => return handle_fetch_failure(window, fetch_err),  // §3.2异常分支
    };

    let mut summary = ReconciliationSummary::default();

    for record in &provider_records {
        // 关联比对键: provider_txn_id, 与内部PaymentOrder唯一索引一致(§2)
        match resolve_pending_compensation(record) {
            Some(pending_order) => {
                summary.pending_compensation.push(pending_order.order_id);
                dispatch_compensation(&pending_order)?;  // §3.3细化
            }
            None => { /* 已达已发货/已补偿等终态,或本非"服务商已支付但内部未发货"情形,跳过 */ }
        }
    }

    // 全部对账动作记录审计日志(复用RGS-BAS-003§7既有审计设计存储结构，不新建审计机制)
    append_reconciliation_audit_log(&summary)?;
    Ok(summary)
}
```

### 3.2 比对条件（RSK-SUP-002"写反"防护的具体实现）

```rust
// 判定"待补偿"前须同时满足两个显式布尔条件，各自记录比对依据快照，
// 代码评审须逐行核对条件方向未写反(RGS-BAS-016§3.3/§4.2检查项)
fn resolve_pending_compensation(record: &ProviderRecord) -> Option<PaymentOrder> {
    let local_order = find_payment_order_by_provider_txn_id(&record.provider_txn_id)?;

    // 条件① provider_txn_id在服务商侧记录中状态为"支付成功"
    let provider_side_paid: bool = record.provider_status == ProviderStatus::PaidSuccess;
    // 条件② 本地PaymentOrder.state不在(已发货、已补偿)集合内
    let local_side_not_fulfilled: bool =
        !matches!(local_order.state, OrderState::已发货 | OrderState::已补偿);

    // 两条件均需显式为true才判定"待补偿"，任一为false均不判定
    // （刻意用两个具名bool变量而非内联&&表达式，是为§4.2代码评审"逐行核对条件方向"
    //   要求服务——具名变量使"条件①是否被误写为!provider_side_paid"这类写反缺陷
    //   在评审时更易被发现，比对依据快照即为record.provider_status与local_order.state本身）
    if provider_side_paid && local_side_not_fulfilled {
        Some(local_order)
    } else {
        None
    }
}
```

### 3.3 补偿分发（含TBD-SUP-002阈值判定）

```rust
fn dispatch_compensation(order: &PaymentOrder, threshold: Money) -> Result<(), ReconError> {
    // 标记为待补偿状态: 以provider_txn_id为幂等键的UPSERT/条件更新，
    // 而非应用层先查后写，避免RSK-SUP-002同类缺陷绕过数据库层唯一性保护(RGS-BAS-016§3.1原文要求)
    execute_sql(
        "UPDATE payment_orders SET state = '待补偿', updated_at = now()
         WHERE order_id = $1 AND state NOT IN ('已发货', '已补偿')",
        &[&order.order_id],
    )?;

    if order.amount <= threshold {
        // 未超阈值: 复用FR-EC-003确定请求路径自动发放(同RGS-DTL-001§3.2既定物理执行语义、
        // RGS-DTL-015§5已声明的同一路径复用关系)
        execute_atomic_grant_via_fr_ec_003(order)?;
        update_order_state(order.order_id, OrderState::已补偿)?;
    } else {
        // 超阈值: 生成SupportTicket(category=payment_issue)转人工复核，不自动发放
        create_support_ticket(SupportTicketDraft {
            player_id: order.player_id_ref(),
            category: TicketCategory::PaymentIssue,
            related_order_id: order.order_id,
        })?;
        // 订单state保持"待补偿"，不迁移到"已补偿"，等待人工复核后由AdminService既有路径处理
    }
    Ok(())
}
```

### 3.4 异常分支：服务商侧数据延迟/不可用

```rust
fn handle_fetch_failure(window: TimeRange, err: FetchError) -> Result<ReconciliationSummary, ReconError> {
    // 本轮跳过，记录告警(复用RGS-BAS-003§6)，不将"未取到数据"误判为"服务商侧无交易"
    emit_alert(AlertSeverity::Medium, "reconciliation_fetch_failed", &err);

    // 下一周期正常拉取时自动补齐窗口: 对账窗口须与上次成功窗口重叠，
    // 避免因单次失败产生的比对空档遗漏掉单——本函数不推进"上次成功窗口"游标，
    // 下一次reconciliation_job_run仍以本次失败前的窗口起点为准（游标推进仅在
    // §3.1主流程成功完成后发生，失败时游标保持不变，天然实现"窗口重叠"效果）
    Ok(ReconciliationSummary::skipped(window))
}
```

---

## 4. SLA超时检测与工单状态机详细设计

对应RGS-BAS-016§2.3/§2.4，落实为具体扫描算法与状态转移函数。

### 4.1 SLA截止时间计算

```rust
fn compute_sla_deadline(category: TicketCategory, created_at: DateTime<Utc>, sla_config: &SlaConfig) -> DateTime<Utc> {
    let response_window = sla_config.window_for(category);  // 见§5参数提案
    created_at + response_window
}

// 定时扫描: 复用§2 idx_support_tickets_state_sla索引
fn ticket_escalation_scan(now: DateTime<Utc>, sla_config: &SlaConfig) {
    for ticket in scan_open_tickets_near_sla(now) {
        let elapsed_ratio = (now - ticket.created_at).as_secs_f64()
            / sla_config.window_for(ticket.category).as_secs_f64();
        if elapsed_ratio >= 0.8 {
            // 超过SLA的80%时长即触发提前预警(RGS-BAS-016§2.4既定升级提醒触发点)
            notify_escalation(ticket.ticket_id, elapsed_ratio);  // 具体消息模板不在本文档范围
        }
    }
}
```

### 4.2 状态转移（对应§2.3迁移条件表）

```rust
fn transition_ticket(ticket_id: TicketId, to: TicketState, ctx: &TransitionContext) -> Result<(), TicketError> {
    let ticket = load_ticket(ticket_id)?;

    match (&ticket.state, &to) {
        (TicketState::待受理, TicketState::处理中) => { /* 客服/GM认领 */ }
        (TicketState::处理中, TicketState::待玩家补充信息) => { /* 客服标记 */ }
        (TicketState::待玩家补充信息, TicketState::处理中) => {
            // 超过配置的静默期(默认7天)自动转已驳回，防止工单无限期悬挂
            // 该分支由独立定时任务驱动(非本函数被动调用路径)，此处仅声明规则:
            // 若调用方是"玩家补充回复"事件，直接放行；若调用方是"静默期超时"事件，
            // 目标态应为已驳回而非处理中——两种触发源共用同一source_state，
            // 但target_state不同，调用方须按触发源正确传入to参数，本函数不代为判断触发源
        }
        (TicketState::处理中, TicketState::已解决) | (TicketState::处理中, TicketState::已驳回) => {
            if ctx.resolution_summary.as_deref().unwrap_or("").is_empty() {
                // resolution_summary为空: 拒绝(FR-SUP-005强制关闭时必须留痕)
                return Err(TicketError::ResolutionSummaryRequired { ticket_id });
            }
        }
        _ => return Err(TicketError::IllegalTransition { from: ticket.state, to }),
        // 工单已关闭(已解决/已驳回)后任何迁移请求均落入此分支被拒绝
        // (§2.3"待受理→处理中"拒绝条件"工单已关闭"的通用化实现:
        //  凡源状态为已解决/已驳回，本match不存在对应分支，自然落入通配拒绝)
    }

    apply_state_update(ticket_id, to, ctx.resolution_summary.clone())?;
    Ok(())
}
```

### 4.3 去重键计算（FR-SUP-007，提示而非拒绝语义）

```rust
fn compute_dedup_key(player_id: PlayerId, category: TicketCategory, now: DateTime<Utc>, window: Duration) -> String {
    // 滚动时间窗口哈希: 同一player_id+category在同一窗口桶内产生相同dedup_key
    let bucket = now.timestamp() / window.as_secs() as i64;
    format!("{}:{}:{}", player_id, category, bucket)
}

fn create_ticket_with_dedup_check(draft: TicketDraft, dedup_window: Duration) -> Result<TicketCreated, TicketError> {
    let dedup_key = compute_dedup_key(draft.player_id, draft.category, now(), dedup_window);
    match insert_ticket(&draft, &dedup_key) {
        Ok(ticket) => Ok(TicketCreated::Fresh(ticket)),
        Err(DbError::UniqueViolation { .. }) => {
            // 命中提示而非拒绝: 查询既有记录返回给调用方,供玩家选择合并或继续新建
            // (区别于RGS-BAS-014举报去重的强制不计数,本设计明确为提示语义)
            let existing = find_ticket_by_dedup_key(&dedup_key)?;
            Ok(TicketCreated::SimilarExists(existing))
        }
        Err(other) => Err(other.into()),
    }
}
```

---

## 5. TBD-SUP参数默认值提案

RGS-BAS-016§2.4已给出SLA分级的"评审前默认建议值"，§3.2/§4.1上线前检查清单标注TBD-SUP-002（自动补偿金额阈值）待财务团队评审。本文档在实现层面直接采纳前者、补充后者的初始提案：

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| SLA首次响应窗口（TBD-SUP-001） | `payment_issue`=4小时／`ban_appeal`=24小时／`item_anomaly`=24小时／`other`=48小时（沿用RGS-BAS-016§2.4已给出的评审前默认建议值，本文档不改动，仅在此处落地为`SlaConfig`实现层面的具体读取值） | 与RGS-BAS-016原文一致，避免默认值在基本设计与详细设计间产生不一致 |
| 自动补偿金额阈值（TBD-SUP-002） | 单笔等值人民币200元 | RGS-BAS-016本身未给出具体数值提案（仅标注为待定），本文档参照§2.4支付类工单SLA最短（4小时，反映支付问题的时效敏感度）与一般小额充值常见客单价区间，提出该初始值供财务评审前上线使用；超过该阈值的场景本身发生频率预期较低（大额支付纠纷本就倾向人工介入），故阈值定得偏保守（宁可更多走人工复核，不放宽自动发放面），符合RSK-SUP-002"防止误判造成资金损失"的既定风险取向 |

以上默认值应在与客服/运营/财务团队评审后按最终结论调整，校准结果回写本文档新版本，不在RGS-BAS-016基本设计层面体现（属于实现参数配置，非结构性设计变更，与RGS-DTL-025§5、RGS-DTL-026§4.1、RGS-DTL-015§4已确立的同类做法一致）。

---

## 6. 本文档的覆盖范围与后续计划

本文档覆盖：AD/EC限界上下文工单与订单两表（`support_tickets`/`payment_orders`）物理DDL（含RGS-BAS-016§0.3已确认的四个跨文档扩展字段、去重唯一索引、幂等唯一索引）、对账批处理主流程与两类异常分支（服务商侧延迟、RSK-SUP-002写反防护）的完整伪代码、SLA超时扫描算法与工单状态转移函数、TBD-SUP-001（沿用原文默认值）/TBD-SUP-002（本文档新提案）的具体数值。

本版本明确不覆盖、留待后续：

- `TicketEscalationNotifier`告警推送的具体消息文案/模板——复用RGS-BAS-003§6既有通道，消息格式属该文档范围。
- 支付服务商侧对账文件/API的具体解析适配代码——`app_store`/`google_play`/直连网关格式各异，属实现阶段各自适配层职责，本文档只覆盖统一为内部标准结构后的比对逻辑。
- TBD-SUP-002自动补偿金额阈值的正式评审结论——本文档§5提案为初始值，非最终值，需财务团队评审。
- `AdminService`执行工单处置决定的既有API本身详细设计——RGS-BAS-016§2.1已明确"`TicketService`本身不直接修改账号状态"，其执行入口属既有`AdminService`职责范围，不属于本文档。

后续详细设计建议顺序：本文档§3.3`execute_atomic_grant_via_fr_ec_003`与RGS-DTL-015§3.1的价值转移路径共享同一物理执行语义前提（RGS-DTL-001§3.2），建议后续如修改该路径需同步检视两份文档；本文档与RGS-DTL-007（数据库设计标准落地示例）、RGS-DTL-015（玩家间交易系统）同批次产出，三者均属03域（数据经济与交易）。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-016§2.1 组件划分 | §3（前提：各组件归属不变） |
| RGS-BAS-016§2.2 `SupportTicket`数据模型 | §2 |
| RGS-BAS-016§2.3 状态机迁移条件 | §4.2 |
| RGS-BAS-016§2.4 SLA分级基准 | §4.1、§5 |
| RGS-BAS-016§3.1 `PaymentOrder`数据模型（含§0.3跨文档权威字段清单） | §2 |
| RGS-BAS-016§3.2 对账批处理时序 | §3.1、§3.3 |
| RGS-BAS-016§3.3 异常分支（服务商延迟、RSK-SUP-002写反防护） | §3.2、§3.4 |
| RSK-SUP-002 | §3.2（具名布尔变量防写反）、§2（幂等唯一索引） |
| TBD-SUP-001〜002 | §5 |
| RGS-DTL-001§3.2（确定请求路径物理执行语义） | 前提依赖，§3.3明确复用而非重新设计 |
| RGS-DTL-007§2/§3（命名/索引/分区句法） | 前提依赖，§2 DDL遵循其模板 |
| RGS-DTL-015§5（同一FR-EC-003路径复用声明） | 前提依赖，跨文档共享同一底层路径 |
