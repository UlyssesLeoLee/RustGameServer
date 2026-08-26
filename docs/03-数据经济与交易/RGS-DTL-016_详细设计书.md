# 详细设计书（詳細設計書 / Detailed Design Document）

**客服工单与支付对账：物理数据库设计・对账批处理算法・SLA状态机详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-016 |
| 标题 | 客服工单与支付对账详细设计 |
| 版本 | 0.3 |
| **状态** | **🟢 v1.0**（per RGS-OPEN-QA-001 v0.2 Q-M-01 答复"先 DTL 升版，后 RGS-DEC-Q003"——状态标记 1.0/1.5 与版本号 v0.2 是两个独立维度，不要混淆） |
| 父文档 | RGS-BAS-016 客服工单与支付对账 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据 | RGS-OPEN-QA-001 v0.2 Q-M-01（先 DTL 升版 §3.4 步骤编号映射，后 RGS-DEC-Q003 审批包）+ RGS-OPEN-QA-001-ACTIONS-v0.3 §3 B-01 + RGS-REV-005 附件 B Saga 演练 6 场景 + RGS-IMPL-001 §3 Saga 编排伪代码 + DTL-001§3.2 物理执行语义 + DTL-031 §8.2 Q-003 跨 DB Saga 边界 |
| 关联 | RGS-DEC-Q003 跨 DB Saga 审批 v0.1（DTL 升版后该 DEC 引用本节编号作为审批基础）/ RGS-REV-005 附件 B 6 场景演练 / RGS-SPEC-CROSS-003 事件 Schema v0.2（含 payment_order / support_ticket 事件）|
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17（v0.1）→ 2026-08-25（v0.2 升版） |
| 制定者 | 架构师（v0.1）→ economy 域 Lead（Ulysses per DEC-008 一人公司 12 角色兼任）（v0.2 升版）|
| 修订历史 | 0.1（2026-08-17）：初版制定 / 0.2（2026-08-25）：WF-1-55.43 L4 任务升版——per Q-M-01 答复新增 §3.4「Saga 步骤编号映射」（1.0~6.0 对应 REV-005 附件 B 6 场景，场景内子步骤 1.1/1.2/1.3 嵌套），原 v0.1 §3.4「异常分支」在 v0.2 升版时改名为 §3.5（章节序号调整，已在修订历史显式声明），为后续 RGS-DEC-Q003 跨 DB Saga 审批包提供引用基础 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 责任人 | economy 域 Lead（Ulysses per DEC-008）|

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档接续RGS-DTL-001/002/025/026/027批次，与RGS-DTL-007/015同批次产出）。细化RGS-BAS-016§2.2数据模型与§2.3状态机为AD限界上下文内`support_tickets`表具体DDL、§3.1数据模型为AD/EC共用`payment_orders`表具体DDL（含RGS-BAS-016 v0.3已确认的跨文档权威字段清单四字段）、§3.2对账批处理时序与§3.3异常分支落实为可直接翻译为Rust实现的伪代码（含RSK-SUP-002"比对条件写反"防护的具体双重校验实现）、SLA超时检测落实为具体扫描算法（含TBD-SUP-001/002两项参数默认值提案沿用RGS-BAS-016原文既定建议值）。**本版本不覆盖**：`TicketEscalationNotifier`告警推送的具体消息模板、支付服务商对账文件/API的具体解析适配代码（因服务商各异，属实现阶段各自适配范畴）。见§5 | 全部 |
| 0.2 | 2026-08-25 | economy 域 Lead（Ulysses per DEC-008 一人公司 12 角色兼任）| Ulysses（per DEC-008 12 角色全签，见§6 审批栏 v0.2 补） | **WF-1-55.43 L4 任务升版**（per RGS-OPEN-QA-001 v0.2 Q-M-01 答复"先 DTL 升版，后 RGS-DEC-Q003"+ ACTIONS-v0.3 §3 B-01）：① **新增 §3.4「Saga 步骤编号映射」**（1.0~6.0 对应 REV-005 附件 B 6 场景，场景内子步骤用 1.1/1.2/1.3 嵌套；为后续 RGS-DEC-Q003 跨 DB Saga 审批包提供引用基础；本文档侧重对账场景的子步骤切分，与 DTL-015 §3.4 整数段保持一致、小数段按本文档侧重展开）；② **v0.1 §3.4「异常分支：服务商侧数据延迟/不可用」在 v0.2 升版时改名为 §3.5「异常分支：服务商侧数据延迟/不可用」**（章节序号调整，章节标题不变，章节内容不变；本调整是为 §3.4 腾出整数段位置以承载跨场景的"编号映射"横切说明，序号调整已在修订历史显式声明，不影响既有引用——任何对 v0.1 §3.4 的引用在 v0.2 起应改为 §3.5）；③ §3.1~§3.3 正文不变（v0.1 已含 `reconciliation_job_run` 主流程 + `resolve_pending_compensation` 双重校验实现 + `dispatch_compensation` 阈值判定，结构与本文档不冲突）；④ 头表加 v0.2 升版行 + 🟢 v1.0 状态标注（per Q-D-01 答复"v0.1 + 🟢 v1.0 双维度"范式）；⑤ 引用同步 checklist（per Q-M-09 答复）：全仓 grep `DTL-016` 引用见§7 修订清单，未发现 v0.1→v0.2 必改引用（DTL-007/001 等只引用 §2 DDL 句法模板不需改；DTL-031 §8.2 阻断解除由 RGS-DEC-Q003 + DTL-031 v0.3 后续处理，本版本不直接动 DTL-031）。**本版本不覆盖**：RGS-DEC-Q003 审批包正文（另一 L4 任务 WF-1-55.43 B-02 产出）。 | §3.4（新增）+ §3.4→§3.5（重编号）+ 头表 + 修订历史 + 追溯性 |
| 0.3 | 2026-08-25 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））| — | **同步父 BAS-016 升版至 v0.3**（2 次升版，BAS-016 v0.2 + v0.3 装饰性升版）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-016 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

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
   - 3.1 [主流程](#31-主流程)
   - 3.2 [比对条件（RSK-SUP-002"写反"防护的具体实现）](#32-比对条件rsk-sup-002写反防护的具体实现)
   - 3.3 [补偿分发（含TBD-SUP-002阈值判定）](#33-补偿分发含tbd-sup-002阈值判定)
   - 3.4 [Saga 步骤编号映射（v0.2 新增）](#34-saga-步骤编号映射v02-新增per-rgs-open-qa-001-v02-q-m-01--actions-v03--3-b-01)
   - 3.5 [异常分支：服务商侧数据延迟/不可用](#35-异常分支服务商侧数据延迟不可用v02-升版时由原-34-改名为-35章节内容不变)
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

**与逻辑设计的对应关系**：`payment_orders`的`platform_type`/`platform_environment`/`refund_status`/`payment_channel`四字段是RGS-BAS-016 v0.3修订历史中明确记载的"跨文档字段清单同步"结果——本文档DDL直接采纳该同步结果作为唯一权威定义，不重新讨论字段是否应归属本表，这是对RGS-BAS-016已完成的跨文档协调的物理落实，非本文档新决策。

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

### 3.4 Saga 步骤编号映射（v0.2 新增，per RGS-OPEN-QA-001 v0.2 Q-M-01 + ACTIONS-v0.3 §3 B-01）

> **本节定位**：per Q-M-01 答复"整数段=场景，小数段=场景内步骤"，将本 DTL §3.1~§3.3 各伪代码片段中的物理步骤与 **RGS-REV-005 附件 B Saga 6 场景**（§B.2~§B.7）做**唯一稳定映射**。该映射是后续 RGS-DEC-Q003 跨 DB Saga 审批包的引用基础（DEC-Q003 §2 6 场景决议直接引用 `1.0~6.0` 编号指代 REV-005 附件 B 演练结果），不在 §3.1~§3.3 内部插入以保持正文步骤图无扰。
>
> **本 DTL 编号映射侧重**：DTL-015 §3.4 侧重交易补偿（`TradeSettlementSaga`），DTL-016 §3.4（本节）侧重**对账补偿**（`reconciliation_job_run` / `dispatch_compensation` / `SupportTicket` 联动）；整数段编号（1.0~6.0）**完全一致**以保证跨 DTL 引用稳定性，小数段子步骤按 DTL 自身侧重不同（如本节 1.1~1.5 子步骤强调"对账 + 补偿"链路，与 DTL-015 §3.4 1.1~1.5 子步骤强调"价值转移 + OCC"链路不同）。
>
> **不替代 REV-005 附件 B**：本节是**编号到文档位置的反向索引**，不是新一轮场景演练；具体输入/状态机/DB/验证/边界细节全部以 REV-005 附件 B v0.1 为权威源。

#### 3.4.1 编号总览

| 编号 | 场景名 | 对应 REV-005 附件 B 章节 | 本 DTL 中物理步骤对应位置 | 涉及 DDL/对象 | 跨域范围 |
|---|---|---|---|---|---|
| **1.0** | 单事务单 DB 路径（场景 1:正常 Saga 路径）| §B.2 | §3.1 `reconciliation_job_run` 主流程 + §3.3 `execute_atomic_grant_via_fr_ec_003` 单 DB 价值发放 | `economy_db.payment_orders`（per §2 RGS-BAS-016 v0.3 权威字段清单）| 单 DB（economy_db）|
| **2.0** | 跨域单 Saga（含 admin 域 audit_log）| §B.5 + §B.2 衍生 | §3.3 补偿分发 + `create_support_ticket` 跨域写 admin_db | `economy_db.payment_orders` + `admin_db.support_tickets`（§2）| 2 域（EC + AD）|
| **3.0** | 跨 DB Saga（5 域独立 DB 拓扑，Q-003 核心场景）| §B.2 + §B.7 | §3.1 入口 + DTL-031 §8.2 边界 + RGS-IMPL-001 §3 saga_orchestrator | 5 域全部 DB + `economy_db.sagas` 协调表 | 5 域 |
| **4.0** | Saga 失败补偿（场景 2:中途失败 → Failed）| §B.3 | §3.3 `dispatch_compensation` 阈值判定 + §3.2 `resolve_pending_compensation` 双重校验 | `economy_db.payment_orders.state='待补偿'` + `transaction_ledger` 补偿 credit 行 | 最低 1 域（EC）最高 5 域 |
| **5.0** | Saga 超时 + DLQ（场景 3:步进超 deadline）| §B.4 | §3.5（原 v0.1 §3.4）异常分支 + DLQ 落库（per Q-M-06 答复）| `economy_db.payment_orders` 长时间停留在非终态 + `admin_db.dlq` | 同 4.0 |
| **6.0** | 人工介入恢复（场景 4:GM 审批 + 场景 6:PFAU 联动）| §B.5 + §B.7 | §3.3 `create_support_ticket` 转人工 + §4 SLA 升级 + DTL-031 §10 PFAU 联动 | `admin_db.support_tickets` + `admin_db.review_queue` + `admin_db.pfau_state` | 5 域全栈 |

#### 3.4.2 场景 1.0 子步骤（单事务单 DB 路径，对账主流程）

| 子步骤 | 物理动作 | 本 DTL §3.1 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **1.1** | 定时任务触发（周期 per NFR-SUP-002）| §3.1 `reconciliation_job_run(window)` 入口 | （不变更 DDL，靠 cron）| 周期错位 → 重复拉取（依赖幂等键 §1.2）|
| **1.2** | 服务商侧记录拉取（具体解析适配不在本文档范围，见§1.2）| §3.1 `fetch_provider_records(window)` | 隐式涉及 `payment_orders.provider_txn_id` 唯一索引（§2）| 拉取失败 → 5.0 异常分支（走原 v0.1 §3.4，现 §3.5）|
| **1.3** | 关联比对（`resolve_pending_compensation`，per §3.2 双重校验）| §3.2 第 7-25 行 | `payment_orders.uq_payment_orders_provider_txn_id` 唯一索引 | 条件写反（RSK-SUP-002）→ §3.2 具名 bool 变量防护 |
| **1.4** | 单 DB 价值发放（`execute_atomic_grant_via_fr_ec_003`，未超阈值）| §3.3 第 18-22 行 | `economy_db.accounts`（per RGS-DTL-001 §3.1）+ `transaction_ledger` | 发放失败 → 4.0 补偿（回滚到"待补偿"等待人工）|
| **1.5** | 订单 state 终态迁移（`已补偿`）+ audit_log 写入 | §3.3 第 23 行 | `payment_orders.state='已补偿'` + `payment_orders.updated_at` 刷新 | 终态迁移失败 → 4.0 兜底补偿（per §3.2 同类）|

#### 3.4.3 场景 2.0 子步骤（跨域单 Saga 含 admin 域 audit_log + 工单）

| 子步骤 | 物理动作 | 本 DTL §3.3 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **2.1** | 同 1.1~1.4（对账 + 单 DB 价值发放完成）| §3.1 + §3.3 | 同 1.0 子步骤 | 同 1.0 |
| **2.2** | 跨域写 admin_db.support_tickets（per RGS-BAS-016 §2.2 跨文档权威字段清单）| §3.3 第 25-32 行 `create_support_ticket` | `admin_db.support_tickets`（§2 + 唯一索引 `dedup_key`）| 跨域写失败 → 不允许掩盖（per RGS-IMPL-001 §3.4）|
| **2.3** | 跨域 1PC 兜底（admin 域延迟降级为本地缓冲 + Outbox 重试）| §3.3 隐式 | `admin_db.support_tickets_outbox`（per 0003_outbox.sql 类精神，需补具体表）| 缓冲失败 → 走 4.0 补偿（撤销 1.4 价值发放）|

#### 3.4.4 场景 3.0 子步骤（跨 DB Saga，Q-003 核心场景，对账域侧重）

| 子步骤 | 物理动作 | 本 DTL 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **3.1** | 5 域 DB 拓扑确认 | DTL-031 §10 + ARC-008 | 5 域 DB 各自 `0001_init.sql` | 拓扑不匹配 → 阻断（per DTL-031 §8.2 Q-003 审批前阻断）|
| **3.2** | 跨域对账 Saga 入口（player → economy 支付 → admin 对账 → economy 补偿）| §3.1 + RGS-IMPL-001 §3 | `economy_db.sagas` + 各域 inbox/outbox | 入口失败 → 4.0 补偿 |
| **3.3** | 跨域 step 1：economy 域 `payment_orders` 写入（per §2 + RGS-BAS-016 v0.3 权威字段清单）| §3.3 | `economy_db.payment_orders` | 写入失败 → 4.0 补偿 |
| **3.4** | 跨域 step 2：admin 域 `support_tickets`（超阈值转工单）或 audit_log | §3.3 | `admin_db.support_tickets` / `admin_db.audit_log` | admin 域不可达 → 4.0 补偿 |
| **3.5** | 跨域 step 3：player 域回写（如 `player_account_audit` 引用，per DTL-001 §3.1）| 不在本 DTL 范围 | `player_db.account_audit` | player 域不可达 → 4.0 整体补偿 |
| **3.6** | 跨域 step 4：social 域通知（支付完成通知）| 不在本 DTL 范围（social 域 DTL-043 负责）| `social_db.notifications` | 通知失败 → 业务可接受，不触发补偿（per DTL-016 §1.2）|
| **3.7** | Saga 协调者持久化状态（`sagas.status='completed'`）| RGS-IMPL-001 §3 | `economy_db.sagas` | 协调者 crash → `saga_orchestrator.resume(saga_id)` 重入 |

#### 3.4.5 场景 4.0 子步骤（Saga 失败补偿，对账域侧重）

| 子步骤 | 物理动作 | 本 DTL §3.3 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **4.1** | 阈值判定失败（`order.amount > threshold`，per TBD-SUP-002 默认 10000）：转人工工单 | §3.3 第 24-32 行 | `admin_db.support_tickets.category='payment_issue'` | 工单创建失败 → 监控告警（`AlertSeverity::High`）|
| **4.2** | 双重校验失败（`provider_side_paid && local_side_not_fulfilled` 条件写反防护）| §3.2 第 16-30 行 | （不变更 DDL，靠具名 bool 变量）| RSK-SUP-002 防护，代码评审必查 |
| **4.3** | 单 DB 价值发放失败（`execute_atomic_grant_via_fr_ec_003` 失败）：订单 state 保持"待补偿"，不迁终态 | §3.3 第 26 行 | `economy_db.payment_orders.state='待补偿'` | 终态误迁移 → 资产被双重发放（per RSK-SUP-002 同类）|
| **4.4** | 5 域跨 DB 拓扑下的补偿顺序：按 saga `steps` 倒序 | RGS-IMPL-001 §3 + REV-005 §B.3 验证 | `economy_db.sagas.steps[i].status` | 顺序错乱 → 资产状态不一致 |
| **4.5** | 异常分支兜底（服务商侧延迟，原 v0.1 §3.4，现 §3.5）：`handle_fetch_failure` 跳过本轮 + 下一周期补齐窗口 | §3.5 | （不变更 DDL，靠游标未推进）| 游标错位 → 漏单（per §3.5 注释）|

#### 3.4.6 场景 5.0 子步骤（Saga 超时 + DLQ，对账域侧重）

| 子步骤 | 物理动作 | 本 DTL 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **5.1** | 协调者发现对账 step 超过 30s deadline（per RGS-IMPL-001 §3 deadline 策略）| §3.1 隐式 | （不新增表，靠协调者内存）| 协调者 crash → 由 5.6 续跑 |
| **5.2** | 强制 `mark_failed` 触发回滚 | §3.3 + REV-005 §B.4 | `economy_db.sagas.steps[failed].error='deadline exceeded'` | 错误信息丢失 → 排查困难 |
| **5.3** | DLQ 落库（per Q-M-06 答复）：失败对账任务写入 `admin_db.dlq`（**不**留在 `economy_db.payment_orders` 防污染业务表）| DTL-031 §8.2 引用 + 新增 DLQ 表约定 | `admin_db.dlq`（per Q-M-06 答复新增表）| DLQ 写入失败 → 重试 + 监控告警 |
| **5.4** | 30s 触发人工升级（per REV-005 §B.4.5 边界）| §3.3 `create_support_ticket` 兜底 | `admin_db.support_tickets` | 60s 仍未处理 → critical 告警 |
| **5.5** | 整体对账超 5 分钟（reservation 过期阈值）→ 强制 Failed + 全量补偿 | RGS-IMPL-001 §3 + REV-005 §B.4 边界 E3.4 | `economy_db.reservations.status='expired'` | reservation 已过期 → 跳过释放（per E2.2 同类）|
| **5.6** | 协调者 crash 后续跑（`saga_orchestrator.resume(saga_id)`）| RGS-IMPL-001 §3 | `economy_db.sagas.version` 字段 CAS | version CAS 冲突 → 释放锁重新加载 |

#### 3.4.7 场景 6.0 子步骤（人工介入恢复，对账域侧重）

| 子步骤 | 物理动作 | 本 DTL 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **6.1** | 金额 > `REVIEW_THRESHOLD=10000`（per RGS-IMPL-100 §3.4）→ `PendingReview` 暂停态 | §3.3 + REV-005 §B.5 | `economy_db.sagas.status='pending_review'` | 阈值调整后存量 saga 不回溯（per E5.5）|
| **6.2** | admin 域 `support_tickets` 入队（GM 审批，category=payment_issue）| §3.3 第 24-32 行 | `admin_db.support_tickets` + 唯一索引 `dedup_key` | 队列不可达 → 监控告警 |
| **6.3** | GM 审批通过（`admin.v1.AdminService/ReviewDecision`）→ 触发 saga 续跑 | RGS-IMPL-100 §3.4 | `admin_db.audit_log` + `economy_db.sagas` | 审批后协调者 crash → 由 5.6 续跑 |
| **6.4** | GM 拒绝（`PendingReview → Aborted`，**不**进 `Failed`）| §3.3 | `economy_db.sagas.status='aborted'` + payment_orders state 保持"待补偿" | 拒绝后玩家资产被错误释放 → 走 6.6 人工兜底 |
| **6.5** | PFAU 联动（per handoff §10）：admin 域 canary 升级期间，saga 涉及 admin 域步骤暂停 | DTL-031 §10 PFAU 联动 | `admin_db.pfau_state` + `admin_db.pfau_kubernetes_pod_state` | 升级期间节点掉线 → `paused_permanently` 触发 6.6 人工介入 |
| **6.6** | 人工兜底（Ulysses 一身 12 角色 per DEC-008）：当 saga 处于 `paused_permanently` 或 `compensation_failed` 时，由 Ulysses 决策 `retry`/`rollback`/`abort` | DTL-031 §8 + DEC-008 | `admin_db.audit_log` GM 决策记录 | GM 决策与 saga 当前状态不一致 → 双签校验 |
| **6.7** | SLA 升级（per §4.1）：`ticket_escalation_scan` 超过 80% SLA 时长 → `notify_escalation` | §4.1 | `support_tickets.state_sla` 部分索引 | 通知失败 → 重试 + 监控告警 |

#### 3.4.8 编号稳定性约束（v0.2 本节新增的硬约束，与 DTL-015 §3.4.8 完全对齐）

为确保本节编号作为 RGS-DEC-Q003 跨 DB Saga 审批包的稳定引用基础，v0.2 起以下**编号稳定性约束**生效：

1. **整数段（1.0~6.0）不重定义**：未来新增场景需用 7.0+ 整数段，不允许覆盖 1.0~6.0；如有场景归类调整，须在 DEC 审批包中显式声明"旧编号 → 新编号"映射并保留旧编号 6 个月。
2. **小数段子步骤**（1.1~6.7 等）：允许在同一整数场景内**追加**新子步骤（如 1.6），不允许**重定义**已有子步骤的物理动作或 DDL 引用；如需重定义，须升 v0.3 + DEC 审批。
3. **跨 DTL 引用一致性**：DTL-015 §3.4 Saga 步骤编号映射（v0.2 升版）使用**完全相同的整数段编号**（1.0~6.0）；DTL-015 侧重交易补偿子步骤，DTL-016（本节）侧重对账补偿子步骤，两者整数段 1.0~6.0 一致即可保证跨 DTL 引用的整数段语义稳定。
4. **DEC-Q003 引用形式**：RGS-DEC-Q003 v0.1 §2 6 场景决议直接使用 `1.0~6.0` 整数段（不展开小数段），小数段在 DEC-Q003 §3 风险接受 / §4 补偿策略中按需引用；本 DTL §3.4 1.x/2.x/3.x/4.x/5.x/6.x 子步骤可被 DEC-Q003 §4 补偿策略直接引用作为具体补偿步骤的物理落点。

> **本节非权威源**：具体场景演练的输入/状态机/DB/验证/边界细节以 **RGS-REV-005 附件 B v0.1** 为权威源；本节仅做"编号 → REV-005 章节 + 本 DTL 物理步骤"反向索引。如本节与 REV-005 附件 B v0.1 冲突，**以 REV-005 附件 B 为准**并升 DTL-016 v0.3 修正本节。
>
> **v0.1 §3.4 → v0.2 §3.5 章节序号变更声明**：v0.1 §3.4「异常分支：服务商侧数据延迟/不可用」在 v0.2 升版时改名为 §3.5，章节标题、章节内容、伪代码均无变化，仅章节序号变更。任何对 v0.1 §3.4 的引用在 v0.2 起应改为 §3.5。本调整不构成 v0.1 读者的认知障碍（v0.1 文档已固定），但后续 reviewer / 维护者请注意：v0.2 起 §3.4 不再是"异常分支"，而是"Saga 步骤编号映射"；§3.5 才是"异常分支"。

### 3.5 异常分支：服务商侧数据延迟/不可用（v0.2 升版时由原 §3.4 改名为 §3.5，章节内容不变）

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

本文档覆盖：AD/EC限界上下文工单与订单两表（`support_tickets`/`payment_orders`）物理DDL（含RGS-BAS-016 v0.3已确认的四个跨文档扩展字段、去重唯一索引、幂等唯一索引）、对账批处理主流程与两类异常分支（服务商侧延迟、RSK-SUP-002写反防护）的完整伪代码、SLA超时扫描算法与工单状态转移函数、TBD-SUP-001（沿用原文默认值）/TBD-SUP-002（本文档新提案）的具体数值。

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
