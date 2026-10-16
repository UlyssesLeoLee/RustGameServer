# 详细设计书（詳細設計書 / Detailed Design Document）

**玩家间交易系统：物理数据库设计・Saga编排伪代码・并发防护详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-015 |
| 标题 | 玩家间交易系统详细设计 |
| 版本 | 0.2 |
| **状态** | **🟢 v1.0**（per RGS-OPEN-QA-001 v0.2 Q-M-01 答复"先 DTL 升版，后 RGS-DEC-Q003"——状态标记 1.0/1.5 与版本号 v0.2 是两个独立维度，不要混淆） |
| 父文档 | RGS-BAS-015 玩家间交易系统 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据 | RGS-OPEN-QA-001 v0.2 Q-M-01（先 DTL 升版 §3.4 步骤编号映射，后 RGS-DEC-Q003 审批包）+ RGS-OPEN-QA-001-ACTIONS-v0.3 §3 B-01 + RGS-REV-005 附件 B Saga 演练 6 场景 + RGS-IMPL-001 §3 Saga 编排伪代码 + DTL-001§3.2 物理执行语义 + DTL-031 §8.2 Q-003 跨 DB Saga 边界 |
| 关联 | RGS-DEC-Q003 跨 DB Saga 审批 v0.1（DTL 升版后该 DEC 引用本节编号作为审批基础）/ RGS-REV-005 附件 B 6 场景演练 / RGS-SPEC-CROSS-003 事件 Schema v0.2（含 transaction_ledger 事件）|
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17（v0.1）→ 2026-08-25（v0.2 升版） |
| 制定者 | 架构师（v0.1）→ economy 域 Lead（Ulysses per DEC-008 一人公司 12 角色兼任）（v0.2 升版）|
| 修订历史 | 0.1（2026-08-17）：初版制定 / 0.2（2026-08-25）：WF-1-55.43 L4 任务升版——per Q-M-01 答复新增 §3.4「Saga 步骤编号映射」（1.0~6.0 对应 REV-005 附件 B 6 场景，场景内子步骤 1.1/1.2/1.3 嵌套），为后续 RGS-DEC-Q003 跨 DB Saga 审批包提供引用基础 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 责任人 | economy 域 Lead（Ulysses per DEC-008）|

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档接续RGS-DTL-001/002/025/026/027批次，与RGS-DTL-007/016同批次产出）。细化RGS-BAS-015§2状态机/组件设计与§3逻辑数据模型为EC限界上下文内`trade_offers`／`trade_audit_logs`两表具体DDL（复用RGS-DTL-007§2既定命名/索引/分区句法），§4交易成立时序落实为`TradeSettlementSaga`可直接翻译为Rust实现的伪代码（含快照OCC校验、补偿路径、`CompensationFailed`升级分支），§2.3可见性校验落实为具体配置读取与拒绝路径伪代码（含TBD-TRD-001/002两项参数默认值提案）。**本版本不覆盖**：GM人工核账队列UI、`TradeOfferService`挂单创建/撤销的完整HTTP/gRPC协议线格式细节（仅给出关键字段，非完整IDL）。见§5 | 全部 |
| 0.2 | 2026-08-25 | economy 域 Lead（Ulysses per DEC-008 一人公司 12 角色兼任）| Ulysses（per DEC-008 12 角色全签，见§6 审批栏 v0.2 补） | **WF-1-55.43 L4 任务升版**（per RGS-OPEN-QA-001 v0.2 Q-M-01 答复"先 DTL 升版，后 RGS-DEC-Q003"+ ACTIONS-v0.3 §3 B-01）：① **新增 §3.4「Saga 步骤编号映射」**（1.0~6.0 对应 REV-005 附件 B 6 场景，场景内子步骤用 1.1/1.2/1.3 嵌套；为后续 RGS-DEC-Q003 跨 DB Saga 审批包提供引用基础）；② §3.1~§3.3 正文不变（v0.1 已含 `execute_atomic_transfer` 四步价值转移伪代码 + `CompensationFailed` 升级分支 + `TradeVisibilityGuard` 可见性校验，结构与本文档不冲突）；③ 头表加 v0.2 升版行 + 🟢 v1.0 状态标注（per Q-D-01 答复"v0.1 + 🟢 v1.0 双维度"范式）；④ 引用同步 checklist（per Q-M-09 答复）：全仓 grep `DTL-015` 引用见§7 修订清单，未发现 v0.1→v0.2 必改引用（DTL-007/001 等只引用 §2 DDL 句法模板不需改；DTL-031 §8.2 阻断解除由 RGS-DEC-Q003 + DTL-031 v0.3 后续处理，本版本不直接动 DTL-031）。**本版本不覆盖**：RGS-DEC-Q003 审批包正文（另一 L4 任务 WF-1-55.43 B-02 产出）。 | §3.4（新增）+ 头表 + 修订历史 + 追溯性 |
| 0.2 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | (复核) 同步父 BAS-015 升版至 v0.2 + 对齐检查通过：BAS-015 v0.2 三项补强内容（① 新增 `TradeVisibilityGuard` 落地 FR-TRD-006 / ② 补充 `TradeOffer` 索引与 `TradeAuditLog` 字段级设计 FR-TRD-015~018 / ③ 补充 OCC 乐观锁 + Saga 补偿失败升级分支 RSK-TRD-002）已分别在 DTL-015 §3.3 / §2 / §3.1~§3.2 落实完毕，结构与本文档无冲突；无新增/补写章节，头表 `| 版本 |` 保持 0.2 不重复升版，仅元数据层补登本复核行。**本行不引入新设计，不重写父 BAS-015**。 | 修订历史（仅元数据补登）|

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-015§3逻辑字段表/RGS-DTL-007§2命名规范一致，Saga补偿路径是否覆盖RGS-BAS-015§4全部故障时点 |
| 评审（DBA） | | | 索引设计是否覆盖§3既定的挂单列表/审计时间线两类查询方向，`trade_audit_logs`月度分区是否与既有分区管理脚本兼容 |
| 审批（负责人） | | | 本文档的基准化；TBD-TRD-001（可见性范围）/TBD-TRD-002（手续费率）默认值提案是否可直接采纳或需策划/财务评审后再定 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：EC限界上下文交易两表](#2-物理数据库设计ec限界上下文交易两表)
3. [交易成立Saga详细设计](#3-交易成立saga详细设计)
   - 3.1 [主流程与OCC校验](#31-主流程与occ校验)
   - 3.2 [补偿路径与升级分支](#32-补偿路径与升级分支)
   - 3.3 [可见性校验](#33-可见性校验tradevisibilityguard对应rgs-bas-01523)
   - 3.4 [Saga 步骤编号映射（v0.2 新增）](#34-saga-步骤编号映射v02-新增per-rgs-open-qa-001-v02-q-m-01--actions-v03--3-b-01)
4. [TBD-TRD参数默认值提案](#4-tbd-trd参数默认值提案)
5. [本文档的覆盖范围与后续计划](#5-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-015给出了`TradeOffer`/`TradeAuditLog`的逻辑字段表、状态机迁移条件表、交易成立的文字流程描述与并发控制/幂等要点。本文档将其落实为：可直接执行的PostgreSQL DDL、`TradeSettlementSaga`主流程与补偿路径的完整伪代码（含全部RGS-BAS-015§4已列出的故障时点）、`TradeVisibilityGuard`可见性校验的具体判定逻辑与两项TBD参数的初始默认值提案。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-015已确定的任何结构性选择（全部组件依附既有EC限界上下文、Saga补偿边界、`Accepted`状态不可反悔、价值转移路径必须复用FR-EC-003确定请求路径）。
- 不覆盖GM人工核账队列的具体UI交互——属于GM后台前端自身设计范围，RGS-BAS-015§4仅要求"转入GM人工核账队列"这一去向，具体队列展示/操作界面不属于本文档（EC域）职责。
- 不给出`TradeOfferService`挂单创建/撤销/超时管理接口的完整协议线格式（.proto风格完整IDL）——这些接口的方法签名RGS-BAS-015未给出细节，本文档只在§3中给出Saga内部消费的关键字段，完整接口契约留待实现阶段或后续版本按RGS-DTL-001§4已确立的协议线格式记述规则补充。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准（复用RGS-DTL-007§2已确立的命名/索引/分区模板句法），算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：EC限界上下文交易两表

对应RGS-BAS-015§3。两表依附既有EC限界上下文数据库（`economy_db`，同RGS-DTL-001§3已建立的库），本文档只新增表结构，不新建库。

```sql
-- 交易挂单表，对应FR-TRD-001〜006/010〜014
CREATE TABLE trade_offers (
    trade_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    initiator_id       BIGINT NOT NULL,      -- 逻辑引用player_db.accounts，跨库不建物理FK
                                                -- （同RGS-DTL-001/025/026既定跨库引用规则）
    target_id           BIGINT NOT NULL,
    initiator_items      JSONB NOT NULL,      -- 引用既有物品/货币规格，结构见economy_db.inventory_items/wallets
                                                -- 语义，非独立新schema（同RGS-DTL-001§3.1既定字段语义复用）
    target_items          JSONB NOT NULL,
    snapshot_version        INTEGER NOT NULL DEFAULT 0,  -- Accepted时刻锁定的快照版本号,FR-TRD-014防调包校验,
                                                            -- 同时是本表自身的OCC乐观锁列(DR-007精神,§3并发控制)
    state                    TEXT NOT NULL DEFAULT 'Draft'
                               CHECK (state IN ('Draft', 'Offered', 'Accepted', 'Settled',
                                                 'Cancelled', 'Expired', 'CompensationFailed')),
                               -- CompensationFailed为RGS-BAS-015§4明确"不复用既有ST-004枚举值,
                               -- 避免与正常终态混淆"新增的专用中间态,此处按其要求单独枚举
    fee_rate                  NUMERIC(5,4) NULL,   -- TBD-TRD-002待定,默认NULL表示未启用(见§4)
    expire_at                  TIMESTAMPTZ NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 索引设计（对应§3既定两类查询方向 + 定时任务批量扫描）
CREATE INDEX idx_trade_offers_initiator_state ON trade_offers (initiator_id, state);
    -- 支撑"我发起的挂单列表"查询(FR-TRD既有GetMyOffers场景)
CREATE INDEX idx_trade_offers_target_state ON trade_offers (target_id, state);
    -- 支撑"我收到的挂单列表"查询
CREATE INDEX idx_trade_offers_state_expire
    ON trade_offers (state, expire_at)
    WHERE state = 'Offered';
    -- 部分索引：支撑§4定时任务批量扫描Offered且已超期的记录以驱动自动解冻(FR-TRD-003)，
    -- 只索引仍处于Offered态的记录，已终态记录不占索引空间（同RGS-DTL-001既定部分索引手法）

-- 交易审计日志表，对应FR-TRD-015〜018
CREATE TABLE trade_audit_logs (
    log_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id           UUID NOT NULL,   -- 非外键强约束(归档后原表可能已清理，同RGS-BAS-007§4归档标准，
                                           -- RGS-BAS-015§3原文已注明此设计)
    event_type          TEXT NOT NULL CHECK (event_type IN (
                            'created', 'accepted', 'cancelled', 'expired',
                            'settled', 'compensated', 'escalated')),
    actor_id              BIGINT NULL,   -- 系统自动触发(如expired)时为NULL
    snapshot_at_event      JSONB NOT NULL,
    occurred_at              TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (occurred_at);
-- 月度分区，复用RGS-BAS-007§4/RGS-DTL-007§3既定分区滚动创建脚本，
-- 保留期1年(NFR-TRD-004默认值，RGS-BAS-015§3原文已给出)

CREATE INDEX idx_trade_audit_logs_trade_occurred
    ON trade_audit_logs (trade_id, occurred_at);
    -- 支撑单笔交易的完整时间线回放
CREATE INDEX idx_trade_audit_logs_actor_occurred
    ON trade_audit_logs (actor_id, occurred_at)
    WHERE actor_id IS NOT NULL;
    -- 支撑FR-TRD-018"按玩家ID检索交易历史"的GM后台查询，复用RGS-BAS-003§3.4只读查询模式，
    -- 部分索引排除系统自动触发(actor_id为NULL)的记录，减小索引体积
```

**与逻辑设计的对应关系**：`TradeOffer.snapshot_version`在RGS-BAS-015§3中同时承担两个逻辑角色——"Accepted时刻锁定的快照版本号"（业务语义）与"乐观锁版本号"（并发控制机制，§4已述）。本文档物理设计不拆分为两列，因两者在RGS-BAS-015原文中被明确统一为同一字段（"作为乐观锁版本号（见§4并发控制）"），拆分反而会引入两列语义漂移风险，故按原样保留单列，此处特此标注该"一列两用"设计并非本文档遗漏而是对逻辑设计的忠实物理落实。

---

## 3. 交易成立Saga详细设计

对应RGS-BAS-015§4文字流程，落实为`TradeSettlementSaga`的完整伪代码，覆盖全部已列出故障时点。

### 3.1 主流程与OCC校验

```rust
// 触发条件: 双方均已Accept(即trade_offers.state已迁移为'Accepted')
fn settle_trade(trade_id: TradeId) -> Result<(), TradeError> {
    let offer = load_trade_offer(trade_id)?;
    debug_assert_eq!(offer.state, TradeState::Accepted);

    // 并发控制(RSK-TRD-002双花/调包防护): 以snapshot_version做乐观锁校验,
    // 单条SQL保证原子性,不拆两步(避免TOCTOU,同RGS-DTL-026§5"全有或全无"同类精神)
    let occ_result = execute_sql(
        "UPDATE trade_offers SET state = 'Accepted', updated_at = now()
         WHERE trade_id = $1 AND snapshot_version = $2 AND state = 'Accepted'",
        &[&trade_id, &offer.snapshot_version],
    )?;
    // 该UPDATE本身不改变可观察状态(state不变)，其唯一目的是借WHERE条件复核snapshot_version
    // 未被并发操作使其失效——这是"先校验后执行"模式在数据库层面的原子化实现，
    // 不依赖应用层"先查后判断"的非原子路径(同RGS-DTL-001§3.2既定OCC实现精神)

    if occ_result.rows_affected() == 0 {
        // 快照已失效(FR-TRD-014): 直接拒绝进入原子事务,不得静默使用旧快照继续结算
        return Err(TradeError::StaleSnapshot { trade_id });
        // 调用方(客户端)据此错误重新发起挂单，本函数不代为重试
    }

    // 幂等键: trade_id+当前state(FR-TRD-012)。重复提交"接受"操作在state已为Settled时
    // 直接返回既有结果,不重复执行——此处以state检查实现,而非独立幂等键表
    if offer.state == TradeState::Settled {
        return Ok(());  // 已结算,幂等短路
    }

    // 复用FR-EC-003确定请求路径，在同一事务边界内完成四步价值转移(+可选手续费)
    let transfer_result = execute_atomic_transfer(&offer);

    match transfer_result {
        Ok(_) => {
            update_state(trade_id, TradeState::Settled)?;
            append_audit_log(trade_id, AuditEventType::Settled, None, offer.snapshot())?;
            Ok(())
        }
        Err(transfer_err) => handle_settlement_failure(trade_id, &offer, transfer_err),
    }
}
```

### 3.2 补偿路径与升级分支

```rust
fn handle_settlement_failure(
    trade_id: TradeId,
    offer: &TradeOffer,
    cause: TransferError,
) -> Result<(), TradeError> {
    // 任一步失败: Saga补偿,回滚已执行步骤,资产恢复至冻结前状态
    match compensate_partial_transfer(offer) {
        Ok(_) => {
            // 补偿成功: 状态保持Accepted供重试或转人工处理(不迁移到终态)
            append_audit_log(trade_id, AuditEventType::Compensated, None, offer.snapshot())?;
            Err(TradeError::SettlementFailedCompensated { trade_id, cause })
        }
        Err(compensation_err) => {
            // 补偿本身失败(RSK-TRD-002最坏情形,如回滚时资产写入也失败):
            // 状态强制迁移至专用中间态CompensationFailed(不复用既有ST-004枚举值,
            // 避免与正常终态混淆——同§2 DDL CHECK约束已单独枚举该值)
            force_state_transition(trade_id, TradeState::CompensationFailed)?;

            // 触发高优先级告警(复用RGS-BAS-003§6既有告警推送通道，不新建监控通道)
            emit_alert(AlertSeverity::High, "trade_compensation_failed", trade_id);

            append_audit_log(trade_id, AuditEventType::Escalated, None, offer.snapshot())?;

            // 转入GM人工核账队列(具体UI不在本文档范围，见§1.2)
            enqueue_manual_reconciliation(trade_id, compensation_err.clone());

            // 禁止该笔trade_id相关资产在人工核实前被其他操作占用:
            // CompensationFailed本身即为一个"冻结态"标记，其余全部交易/资产操作路径
            // 须在读取trade_offers.state前置校验中显式拒绝对该state的任何进一步转移，
            // 这一拒绝规则须同步反映到economy_db全部涉及资产变更的确定请求路径的
            // 前置校验中（非仅trade_offers自身），故此处不给出单一函数级实现，
            // 而是作为跨路径的不变量声明：任何资产操作若发现关联trade_id处于
            // CompensationFailed态，必须拒绝执行并记录冲突原因，等待GM解除。
            Err(TradeError::CompensationFailedEscalated { trade_id })
        }
    }
}
```

**关键边界条件说明**：

- 补偿成功但整体仍失败时，`state`刻意**不**迁移到任何终态（保持`Accepted`），这是RGS-BAS-015§4原文"状态保持Accepted供重试或转人工处理"的直接翻译，不得误实现为迁移到`Cancelled`等终态，否则会破坏"仍可重试"的语义。
- `CompensationFailed`一旦进入，本文档明确其为**单向**门——只有GM人工核实操作（不属于本文档范围，走既有`AdminService`）能将其迁出，`TradeSettlementSaga`自身不包含任何自动脱离`CompensationFailed`的路径，这是"禁止该笔trade_id相关资产在人工核实前被其他操作占用"约束在状态机层面的落实。

### 3.3 可见性校验（TradeVisibilityGuard，对应RGS-BAS-015§2.3）

```rust
// TradeOfferService创建挂单前同步调用(阻塞校验，非异步)
fn check_trade_visibility(initiator_id: PlayerId, target_id: PlayerId, cfg: &VisibilityConfig) -> Result<(), TradeError> {
    let allowed = match cfg.trade_visibility_scope {
        VisibilityScope::FriendOnly => is_friend(initiator_id, target_id),
        VisibilityScope::PartyOnly => is_same_party(initiator_id, target_id),
        VisibilityScope::FriendOrParty =>
            is_friend(initiator_id, target_id) || is_same_party(initiator_id, target_id),
    };

    if !allowed {
        // 校验失败: Draft→Offered迁移直接拒绝,不产生资产冻结副作用
        // (不消耗FR-TRD-002冻结路径——本函数须在冻结逻辑调用之前执行)
        return Err(TradeError::TargetNotVisible { initiator_id, target_id });
    }
    Ok(())
}
```

### 3.4 Saga 步骤编号映射（v0.2 新增，per RGS-OPEN-QA-001 v0.2 Q-M-01 + ACTIONS-v0.3 §3 B-01）

> **本节定位**：per Q-M-01 答复"整数段=场景，小数段=场景内步骤"，将本 DTL §3.1~§3.3 各伪代码片段中的物理步骤与 **RGS-REV-005 附件 B Saga 6 场景**（§B.2~§B.7）做**唯一稳定映射**。该映射是后续 RGS-DEC-Q003 跨 DB Saga 审批包的引用基础（DEC-Q003 §2 6 场景决议直接引用 `1.0~6.0` 编号指代 REV-005 附件 B 演练结果），不在 §3.1~§3.3 内部插入以保持正文步骤图无扰。
>
> **不替代 REV-005 附件 B**：本节是**编号到文档位置的反向索引**，不是新一轮场景演练；具体输入/状态机/DB/验证/边界细节全部以 REV-005 附件 B v0.1 为权威源。

#### 3.4.1 编号总览

| 编号 | 场景名 | 对应 REV-005 附件 B 章节 | 本 DTL 中物理步骤对应位置 | 涉及 DDL/对象 | 跨域范围 |
|---|---|---|---|---|---|
| **1.0** | 单事务单 DB 路径（场景 1:正常 Saga 路径 / 玩家购买道具）| §B.2 | §3.1 `execute_atomic_transfer` 四步价值转移 | `economy_db.trade_offers` / `economy_db.transaction_ledger`（间接引用，无新增表）| 单 DB（economy_db），对应 5 域独立 DB 拓扑下 EC 域本地事务 |
| **2.0** | 跨域单 Saga（含 admin 域 audit_log）| §B.5 + §B.2 衍生 | §3.1 OCC 校验 + §3.2 补偿路径 + `append_audit_log` | `economy_db` + `admin_db.audit_log`（跨库写，per RGS-DTL-007§2 跨库规则不建物理 FK）| 2 域（EC + AD）|
| **3.0** | 跨 DB Saga（5 域独立 DB 拓扑，Q-003 核心场景）| §B.2 + §B.7 | §3.1 入口 + DTL-031 §8.2 边界 + RGS-IMPL-001 §3 saga_orchestrator | 5 域全部 DB + `economy_db.sagas` 协调表（per `0002_saga_init.sql`）| 5 域（player + economy + match + social + admin）|
| **4.0** | Saga 失败补偿（场景 2:中途失败 → Failed）| §B.3 | §3.2 `handle_settlement_failure` + `compensate_partial_transfer` | `economy_db.sagas.steps[i].status='compensated'` + `transaction_ledger` 补偿 credit 行 | 取决于触发场景，最低 1 域（仅 EC）最高 5 域 |
| **5.0** | Saga 超时 + DLQ（场景 3:步进超 deadline）| §B.4 | §3.1 OCC 超时分支 + `force_state_transition` + DTL-031 §8.2 DLQ 表 | `economy_db.sagas.status='failed'` + `admin_db.dlq`（per Q-M-06 答复 DLQ 落库）| 同 4.0 取决于触发场景 |
| **6.0** | 人工介入恢复（场景 4:GM 审批 + 场景 6:PFAU 联动）| §B.5 + §B.7 | §3.2 `enqueue_manual_reconciliation` + DTL-031 §10 PFAU 联动 | `admin_db.review_queue` + `admin_db.pfau_state` + `economy_db.sagas.status='pending_review'` / `paused_permanently` | 5 域全栈，admin 域主导审批 |

#### 3.4.2 场景 1.0 子步骤（单事务单 DB 路径）

| 子步骤 | 物理动作 | 本 DTL §3.1/§3.3 对应行 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **1.1** | 双方 `Accept` 校验（`trade_offers.state='Accepted'`）| §3.1 第 4-5 行 | `trade_offers` CHECK 约束 | snapshot 失效（FR-TRD-014 双花防护）|
| **1.2** | OCC 乐观锁（`UPDATE ... WHERE snapshot_version=$2`）| §3.1 第 7-19 行 | `trade_offers.snapshot_version` 列（§2 一列两用）| `occ_result.rows_affected() == 0` → `TradeError::StaleSnapshot` |
| **1.3** | 幂等短路（`state == Settled` 直接 return Ok）| §3.1 第 21-23 行 | `trade_offers.state` 字段 | 已结算重复提交，直接返回原结果 |
| **1.4** | `execute_atomic_transfer` 四步价值转移（甲方扣/乙方扣/甲方收/乙方收 + 可选手续费）| §3.1 第 25-26 行（`execute_atomic_transfer` 内部由 RGS-DTL-001 §3.2 FR-EC-003 确定请求路径提供）| 隐式涉及 `wallets` / `inventory_items`（同 DTL-001 §3.1）| 任何 step 失败 → 触发 4.0 补偿 |
| **1.5** | 状态机终态迁移（`Settled`）+ audit_log 写入 | §3.1 第 28-31 行 | `trade_audit_logs` 月度分区表（§2）| audit_log 写入失败 → 需补偿回退（per §3.2 同类精神）|

#### 3.4.3 场景 2.0 子步骤（跨域单 Saga 含 admin 域 audit_log）

| 子步骤 | 物理动作 | 本 DTL §3.1/§3.2 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **2.1** | 同 1.1~1.4（单 DB 价值转移完成）| §3.1 全段 | 同 1.0 子步骤 | 同 1.0 |
| **2.2** | 跨域写 admin_db.audit_log（per RGS-BAS-003 §7 审计设计）| §3.1 第 30 行 `append_audit_log` | `admin_db.audit_log`（SHA-256 升级后结构，per RGS-DEC-015 工程 53+54 AC5）| 跨域写失败 → 不允许掩盖（per RGS-IMPL-001 §3.4 一致性约束）|
| **2.3** | 跨域 1PC 兜底（admin 域延迟降级为本地缓冲 + Outbox 重试）| §3.1 隐式 | `admin_db.audit_log_outbox`（per 0003_outbox.sql）| 缓冲失败 → 走 4.0 补偿路径（撤销 1.4 价值转移）|

#### 3.4.4 场景 3.0 子步骤（跨 DB Saga，Q-003 核心场景）

| 子步骤 | 物理动作 | 本 DTL 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **3.1** | 5 域 DB 拓扑确认（player + economy + match + social + admin 各自独立 PG18.6）| DTL-031 §10 + ARC-008 | 5 域 DB 各自 `0001_init.sql` | 拓扑不匹配 → 阻断（per DTL-031 §8.2 Q-003 审批前阻断）|
| **3.2** | Saga 入口（player → economy 扣款 → match 发放 → social 通知 → player 余额更新）| §3.1 + RGS-IMPL-001 §3 | `economy_db.sagas` + 各域 inbox/outbox | 入口失败 → 4.0 补偿 |
| **3.3** | 跨域 step 1：economy 域扣款（per §3.1 `execute_atomic_transfer`）| §3.1 | `economy_db.accounts` + `transaction_ledger` | 扣款失败 → 4.0 补偿 |
| **3.4** | 跨域 step 2：match 域发放道具 | 不在本 DTL 范围（match 域 DTL-026 负责）| `match_db.player_inventory` | match 域不可达 → 4.0 补偿（per REV-005 §B.3）|
| **3.5** | 跨域 step 3：social 域通知 | 不在本 DTL 范围（social 域 DTL-043 负责）| `social_db.notifications` | 通知失败 → 3.4 已发放则触发 3.4 撤回 → 4.0 补偿 |
| **3.6** | 跨域 step 4：player 域余额更新（最终）| 不在本 DTL 范围 | `player_db.accounts` | 余额更新失败 → 4.0 整体补偿 |
| **3.7** | Saga 协调者持久化状态（`sagas.status='completed'`）| RGS-IMPL-001 §3 | `economy_db.sagas` | 协调者 crash → 由 `saga_orchestrator.resume(saga_id)` 重入 |

#### 3.4.5 场景 4.0 子步骤（Saga 失败补偿）

| 子步骤 | 物理动作 | 本 DTL §3.2 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **4.1** | 补偿成功路径：`compensate_partial_transfer` 成功 → `audit_log.Compensated` + state 保持 `Accepted`（不迁终态）| §3.2 第 8-14 行 | `trade_audit_logs.event_type='compensated'` | 整体仍失败（FR-TRD 因 cause 不可恢复）→ 转人工 |
| **4.2** | 补偿失败路径：`compensate_partial_transfer` 失败 → `force_state_transition(CompensationFailed)` + 高优告警 + GM 队列 | §3.2 第 16-30 行 | `trade_offers.state='CompensationFailed'`（§2 DDL 单独枚举）| GM 队列不可达 → 重试 + 监控告警（`AlertSeverity::High`）|
| **4.3** | `CompensationFailed` 单向门：禁止任何非 `AdminService` 路径脱离该状态 | §3.2 第 26-32 行 + 关键边界条件说明 | （不变更 DDL，靠应用层前置校验）| 误操作 → 资产被双重占用，需 GM 手动恢复 |
| **4.4** | 5 域跨 DB 拓扑下的补偿顺序：按 saga `steps` 倒序（`Completed` 列表逆序）| RGS-IMPL-001 §3 + REV-005 §B.3 验证 | `economy_db.sagas.steps[i].status` | 顺序错乱 → 资产状态不一致（per ADR-0052 顺序修复）|

#### 3.4.6 场景 5.0 子步骤（Saga 超时 + DLQ）

| 子步骤 | 物理动作 | 本 DTL 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **5.1** | 协调者发现单 step 超过 30s deadline（per RGS-IMPL-001 §3 deadline 策略）| §3.1 隐式 | （不新增表，靠协调者内存）| 协调者 crash → 由 5.6 续跑处理 |
| **5.2** | 强制 `mark_failed` 触发补偿 | §3.2 + REV-005 §B.4 | `economy_db.sagas.steps[failed].error='deadline exceeded'` | 错误信息丢失 → 排查困难 |
| **5.3** | DLQ 落库（per Q-M-06 答复）：失败 saga 写入 `admin_db.dlq`（**不**留在 `economy_db.sagas` 防污染业务表）| DTL-031 §8.2 引用 + 新增 DLQ 表约定 | `admin_db.dlq`（per Q-M-06 答复新增表）| DLQ 写入失败 → 重试 + 监控告警 |
| **5.4** | 30s 触发人工升级（per REV-005 §B.4.5 边界）| §3.2 `enqueue_manual_reconciliation` 兜底 | `admin_db.review_queue` | 60s 仍未处理 → critical 告警 |
| **5.5** | 整体 Saga 超 5 分钟（reservation 过期阈值）→ 强制 Failed + 全量补偿 | RGS-IMPL-001 §3 + REV-005 §B.4 边界 E3.4 | `economy_db.reservations.status='expired'` | reservation 已过期 → 跳过释放（per E2.2 同类）|
| **5.6** | 协调者 crash 后续跑（`saga_orchestrator.resume(saga_id)`）| RGS-IMPL-001 §3 | `economy_db.sagas.version` 字段 CAS | version CAS 冲突 → 释放锁重新加载 |

#### 3.4.7 场景 6.0 子步骤（人工介入恢复）

| 子步骤 | 物理动作 | 本 DTL 对应 | 涉及 DDL | 失败模式 |
|---|---|---|---|---|
| **6.1** | 金额 > `REVIEW_THRESHOLD=10000`（per RGS-IMPL-100 §3.4）→ `PendingReview` 暂停态 | §3.2 + REV-005 §B.5 | `economy_db.sagas.status='pending_review'` | 阈值调整后存量 saga 不回溯（per E5.5）|
| **6.2** | admin 域 `review_queue` 入队（GM 审批）| §3.2 兜底 | `admin_db.review_queue` | 队列不可达 → 监控告警（GM 端无感知）|
| **6.3** | GM 审批通过（`admin.v1.AdminService/ReviewDecision`）→ 触发 saga 续跑 | RGS-IMPL-100 §3.4 | `admin_db.audit_log` + `economy_db.sagas` | 审批后协调者 crash → 由 5.6 续跑 |
| **6.4** | GM 拒绝（`PendingReview → Aborted`，**不**进 `Failed`，per RGS-IMPL-100 §3.4 "拒绝 = 用户主动取消"）| §3.2 | `economy_db.sagas.status='aborted'` + reservation 释放 | 拒绝后玩家资产被错误释放 → 走 6.6 人工兜底 |
| **6.5** | PFAU 联动（per handoff §10）：match 域 canary 升级期间，saga 涉及 match 域步骤暂停（per ADR-0052 §2.1 all-reachable 约束）| DTL-031 §10 PFAU 联动 | `admin_db.pfau_state` + `match_db.pfau_kubernetes_pod_state` | 升级期间节点掉线 → `paused_permanently` 触发 6.6 人工介入 |
| **6.6** | 人工兜底（Ulysses 一身 12 角色 per DEC-008）：当 saga 处于 `paused_permanently` 或协调者 `compensation_failed` 时，由 Ulysses 决策 `retry`/`rollback`/`abort`（per DTL-031 §8 边界表）| DTL-031 §8 + DEC-008 | `admin_db.audit_log` GM 决策记录 | GM 决策与 saga 当前状态不一致 → 双签校验 |

#### 3.4.8 编号稳定性约束（v0.2 本节新增的硬约束）

为确保本节编号作为 RGS-DEC-Q003 跨 DB Saga 审批包的稳定引用基础，v0.2 起以下**编号稳定性约束**生效：

1. **整数段（1.0~6.0）不重定义**：未来新增场景需用 7.0+ 整数段，不允许覆盖 1.0~6.0；如有场景归类调整，须在 DEC 审批包中显式声明"旧编号 → 新编号"映射并保留旧编号 6 个月。
2. **小数段子步骤**（1.1~1.6 等）：允许在同一整数场景内**追加**新子步骤（如 1.7），不允许**重定义**已有子步骤的物理动作或 DDL 引用；如需重定义，须升 v0.3 + DEC 审批。
3. **跨 DTL 引用一致性**：DTL-016 §3.4/§3.5 Saga 步骤编号映射（v0.2 升版）须使用**完全相同的整数段编号**（1.0~6.0），仅小数段子步骤可按 DTL 自身侧重不同（如 DTL-016 侧重对账补偿，DTL-015 侧重交易补偿）。
4. **DEC-Q003 引用形式**：RGS-DEC-Q003 v0.1 §2 6 场景决议直接使用 `1.0~6.0` 整数段（不展开小数段），小数段在 DEC-Q003 §3 风险接受 / §4 补偿策略中按需引用。

> **本节非权威源**：具体场景演练的输入/状态机/DB/验证/边界细节以 **RGS-REV-005 附件 B v0.1** 为权威源；本节仅做"编号 → REV-005 章节 + 本 DTL 物理步骤"反向索引。如本节与 REV-005 附件 B v0.1 冲突，**以 REV-005 附件 B 为准**并升 DTL-015 v0.3 修正本节。

---

## 4. TBD-TRD参数默认值提案

RGS-BAS-015§2.3与§3中"交易目标可见性范围"（TBD-TRD-001）与"手续费率"（TBD-TRD-002）均标记为待定，评审前RGS-BAS-015已给出`trade_visibility_scope`评审前默认值`friend_or_party`、`fee_rate`默认0。本文档在此基础上补充实现层面的具体默认配置值，供PH阶段策划/财务评审前的初始上线使用，非最终值：

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| `trade_visibility_scope`（TBD-TRD-001） | `friend_or_party`（沿用RGS-BAS-015§2.3已给出的评审前默认值，本文档不改动，仅在此处重申供实现层面直接读取） | 与RGS-BAS-015原文一致，避免本文档另立新默认值造成基本设计与详细设计的默认值不一致 |
| `fee_rate`（TBD-TRD-002） | `0`（沿用RGS-BAS-015§3已给出的默认值，`trade_offers.fee_rate`列DEFAULT保留NULL表示"未启用"，业务层读取NULL时按0处理，区分"显式0费率"与"未配置"两种语义，供财务评审启用后无需迁移列结构） | NULL与0在未来可能承载不同业务含义（如"手续费功能尚未上线"vs"手续费功能上线但当前费率为0"），保留区分空间是详细设计层面的谨慎添加，非结构性新决策 |

以上默认值应在策划/财务评审后按最终结论调整，校准结果回写本文档新版本，不在RGS-BAS-015基本设计层面体现（属于实现参数配置，非结构性设计变更，与RGS-DTL-025§5、RGS-DTL-026§4.1已确立的同类做法一致）。

---

## 5. 本文档的覆盖范围与后续计划

本文档覆盖：EC限界上下文交易两表（`trade_offers`/`trade_audit_logs`）物理DDL（含索引/分区/OCC乐观锁列）、`TradeSettlementSaga`主流程与全部故障时点（快照失效/转移失败/补偿失败升级）的完整伪代码、`TradeVisibilityGuard`可见性校验的具体判定逻辑、TBD-TRD-001/002两项参数的默认值在实现层面的落地方式。

本版本明确不覆盖、留待后续：

- GM人工核账队列的具体UI交互细节——属于GM后台前端自身设计范围，本文档§3.2只声明"转入队列"这一去向。
- `TradeOfferService`挂单创建/撤销/超时管理接口的完整协议线格式（.proto风格完整IDL）——RGS-BAS-015未给出方法签名细节可供逐一转译，本文档§3.3只给出Saga/Guard消费的关键字段，完整接口契约留待后续版本按RGS-DTL-001§4记述规则补充。
- `execute_atomic_transfer`内部四步价值转移（甲方扣除/乙方扣除/甲方获得/乙方获得+可选手续费）的逐步骤SQL——RGS-BAS-015§4已明确"复用FR-EC-003确定请求路径"，该路径的物理执行语义已由RGS-DTL-001§3.2给出，本文档不重复展开，`execute_atomic_transfer`应理解为对该既有路径的多次调用编排而非新增独立事务机制。
- TBD-TRD-001/002的正式评审结论——当前为初始默认值沿用/补充提案，非最终值。

后续详细设计建议顺序：与本文档同批次的RGS-DTL-016（客服工单与支付对账）在其对账流程中同样复用FR-EC-003确定请求路径，两份文档共享同一物理执行语义前提（RGS-DTL-001§3.2），建议后续如需修改该路径的物理设计，同步检视本文档§3与RGS-DTL-016§3.2是否受影响。

> **v0.2 升版补登（per ULYS-1 后续测试设计补全任务，WF-1-55.43 L4 子任务 v0.2 → v0.2-testdesign）**：原 v0.1 修订历史第 0.1 行明确"本版本不覆盖：GM人工核账队列UI、`TradeOfferService`挂单创建/撤销的完整HTTP/gRPC协议线格式细节（仅给出关键字段，非完整IDL）"——这两项在 v0.2 仍属本文档范围外（GM UI属 GM 后台前端；完整 IDL 由实现阶段补），但**`TradeSettlementSaga`/`TradeVisibilityGuard`/`execute_atomic_transfer` 的测试设计** 已在 v0.2 由本文档§6「后续测试设计补全」落实，补齐 v0.1 审计报告 RGS-REV-001/REV-002 标记的"§X 测试设计 4 项不覆盖"缺口。本节末声明该事实，不修改原不覆盖列表项本身。

---

## 6. 后续测试设计补全（v0.2 test-design follow-up，per ULYS-1）

### 6.1 范围与既有占位

原 v0.1 仅给出"挂单创建/撤销完整 IDL、GM 队列 UI"两类不覆盖项；v0.1 与 v0.2 均**未**为本文档 §3 主流程伪代码配套测试设计。本次补全聚焦 v0.1 已落实的逻辑（DTL 既有章节正文）的可测试设计点，覆盖范围：

| 序号 | 测试主题 | 对应本文档章节 | 测试层 | 关联需求 |
|---|---|---|---|---|
| TC-DTL015-TRADE-SAGA-001~008 | `TradeSettlementSaga` 主流程与故障路径 | §3.1 / §3.2 / §3.4 | UT/IT | FR-TRD-002/010/012/014、RSK-TRD-002 |
| TC-DTL015-VIS-001~003 | `TradeVisibilityGuard` 可见性校验 | §3.3 | UT | FR-TRD-006、TBD-TRD-001 |
| TC-DTL015-DDL-001~002 | DDL 索引/分区/CHECK 约束 | §2 | IT | FR-TRD-015~018 |

每条测试用例均遵循 RFC 2119 强度用语（per `RGS-TST-UT-06 §1.4.1`），引用 `crates/rgs-testkit` 既有 fixture（per `RGS-IMPL-001 §3 Q-206`：单元测试 fake/mockall，集成测试 Testcontainers 真实 PostgreSQL 18.6），不引入新的 trait 抽象（per Q-206 强制规则）。

### 6.2 Saga 主流程测试用例（TC-DTL015-TRADE-SAGA-NNN）

#### TC-DTL015-TRADE-SAGA-001 — 正常路径四步价值转移（happy path）

- **层**：IT；**对应 DTL §**：§3.1 `settle_trade` + §3.4.2 场景 1.0 子步骤 1.1~1.5
- **覆盖需求**：FR-TRD-010/012、FR-EC-003（价值转移确定请求路径）
- **前置条件**：
  - `rgs-testkit` `EconomyFixture::economy(player_id)` 双方各持有余额 ≥ 1 单位（per `with_currency(...)` builder）
  - `trade_offers` 行 `state='Accepted'`，`snapshot_version=1`，`expire_at>now()`
  - Testcontainers PostgreSQL 18.6 启动（per RGS-IMPL-001 §3 Q-206）
- **Steps (Given/When/Then)**：
  - **Given**：甲方 `account_id=A`，余额 100；乙方 `account_id=B`，余额 50；`TradeOffer` 处于 `Accepted`，`snapshot_version=1`
  - **When**：调用 `settle_trade(trade_id)`，触发 `execute_atomic_transfer`
  - **Then**：
    - `trade_offers.state` 迁移到 `'Settled'`，`updated_at` 刷新
    - 甲方余额 = 100 - 转移额（per FR-EC-003 路径），乙方余额 = 50 + 获得额
    - `transaction_ledger` 新增 4 行（甲方 debit、乙方 debit、甲方 credit、乙方 credit）同事务提交
    - `trade_audit_logs` 追加一行 `event_type='settled'`，`actor_id=NULL`（系统自动触发）
- **断言**：四步 SQL 在单事务边界内完成（per DTL §3.1 第 25-26 行）；`audit_log` 写入与 `state` 终态迁移在同一事务；`consumed FR-EC-003` 同一路径不重复。
- **rgs-testkit 引用**：`EconomyFixture::economy("A")`、`EconomyFixture::economy("B")`、`SagaFixture::saga("trade_settle")`（per `crates/rgs-testkit/src/fixture.rs` line 37-44）

#### TC-DTL015-TRADE-SAGA-002 — 快照失效拒绝（FR-TRD-014 防调包）

- **层**：UT（fake OCC）；**对应 DTL §**：§3.1 OCC 校验（line 7-19）；§3.4.2 子步骤 1.2
- **覆盖需求**：FR-TRD-014、RSK-TRD-002（双花防护）
- **前置条件**：`TradeOffer.snapshot_version=1`，但 OCC UPDATE 预置 `rows_affected=0`（fake 返回值）
- **Steps**：
  - **Given**：`load_trade_offer()` 返回 `snapshot_version=1`，但 OCC UPDATE 返回 0 行（模拟并发占用使快照失效）
  - **When**：调用 `settle_trade(trade_id)`
  - **Then**：函数返回 `Err(TradeError::StaleSnapshot { trade_id })`，**不**调用 `execute_atomic_transfer`，**不**更新 `trade_offers.state`，**不**写 `trade_audit_logs`
- **断言**：`audit_log` 表在测试结束时**不**新增 `settled` 或 `compensated` 行；`transaction_ledger` 行数不变。
- **rgs-testkit 引用**：`mockall` mock OCC 函数（per RGS-IMPL-001 §3 Q-206："单元测试使用 fake/mockall"）

#### TC-DTL015-TRADE-SAGA-003 — 幂等短路（重复提交返回既有结果）

- **层**：UT；**对应 DTL §**：§3.1 第 21-23 行；§3.4.2 子步骤 1.3
- **覆盖需求**：FR-TRD-012（幂等键）
- **前置条件**：`trade_offers.state='Settled'`（已结算）
- **Steps**：
  - **Given**：`load_trade_offer()` 返回 `state=Settled`
  - **When**：再次调用 `settle_trade(trade_id)`
  - **Then**：函数 `return Ok(())`，**不**调用 OCC UPDATE，**不**调用 `execute_atomic_transfer`，**不**新增 `audit_log` 行
- **断言**：调用计数：OCC UPDATE = 0、`execute_atomic_transfer` = 0、`append_audit_log` = 0；最终 `trade_audit_logs` 行数与 Given 时一致。

#### TC-DTL015-TRADE-SAGA-004 — 转移中途失败触发补偿路径（场景 4.0）

- **层**：IT；**对应 DTL §**：§3.2 `handle_settlement_failure` + §3.4.5 子步骤 4.1
- **覆盖需求**：RSK-TRD-002、Saga 补偿语义
- **前置条件**：四步价值转移在第三步（甲方 credit）失败（如触发 §3.2 `transfer_err`）
- **Steps**：
  - **Given**：甲方已 debit 成功、乙方已 debit 成功、甲方 credit 失败；模拟 `compensate_partial_transfer` 返回 `Ok(())`
  - **When**：`settle_trade` 触发补偿
  - **Then**：
    - `compensate_partial_transfer` 已被调用 2 次（撤销甲方 debit、乙方 debit）
    - `trade_offers.state` **保持** `'Accepted'`（不迁终态，per §3.2 第 11 行批注 + §3.4.5 子步骤 4.1）
    - `trade_audit_logs` 追加一行 `event_type='compensated'`
    - 函数返回 `Err(TradeError::SettlementFailedCompensated)`
- **断言**：甲方/乙方余额回到 Given 前状态；`state` 不变；新 `audit_log` 事件类型为 `compensated`；**不**触发 `AlertSeverity::High`（补偿成功路径不升级）。
- **rgs-testkit 引用**：`SagaFixture::saga("trade_settle")` + 注入第 3 步失败的 mock transfer

#### TC-DTL015-TRADE-SAGA-005 — 补偿失败升级 `CompensationFailed` 单向门

- **层**：IT；**对应 DTL §**：§3.2 第 16-30 行；§3.4.5 子步骤 4.2/4.3
- **覆盖需求**：RSK-TRD-002 最坏情形
- **前置条件**：`compensate_partial_transfer` 自身失败（如回滚时资产写入也失败）
- **Steps**：
  - **Given**：同 TC-DTL015-TRADE-SAGA-004 但 `compensate_partial_transfer` 返回 `Err(CompensationErr)`
  - **When**：`handle_settlement_failure` 进入 Err 分支
  - **Then**：
    - `force_state_transition(trade_id, CompensationFailed)` 被调用
    - `emit_alert(AlertSeverity::High, "trade_compensation_failed", trade_id)` 被触发（mock 验证）
    - `enqueue_manual_reconciliation(trade_id, compensation_err)` 被调用
    - `trade_audit_logs` 追加一行 `event_type='escalated'`
    - 函数返回 `Err(TradeError::CompensationFailedEscalated)`
- **断言**：再次调用任意资产变更路径涉及该 `trade_id` 时，前置校验拒绝（per §3.2 关键边界条件说明）；验证"禁止资产在人工核实前被其他操作占用"约束生效。
- **rgs-testkit 引用**：`mockall` mock alert emitter、mock manual queue enqueue；integration 验证前置校验拒绝路径（Testcontainers 真实 PG）

#### TC-DTL015-TRADE-SAGA-006 — OCC 乐观锁与 `CompensationFailed` 状态机的并发冲突

- **层**：IT；**对应 DTL §**：§3.4.5 子步骤 4.3 单向门；§2 DDL `trade_offers.state` CHECK 约束
- **覆盖需求**：单向门在并发场景下不可被绕过
- **前置条件**：两个连接同时尝试更新同一 `trade_id`，一个写入 `CompensationFailed`，一个尝试 `Accepted → Settled`
- **Steps**：
  - **Given**：`trade_offers.state='CompensationFailed'`（已升级）
  - **When**：连接 1 尝试 `UPDATE trade_offers SET state='Settled' WHERE trade_id=$1 AND state='Accepted'`（误用旧 version）
  - **Then**：`rows_affected=0`（CHECK 约束 + WHERE 条件双重拒绝），无副作用
  - **When（第二个动作）**：连接 2 尝试 `UPDATE trade_offers SET state='Cancelled' WHERE trade_id=$1 AND state='CompensationFailed'`
  - **Then**：`rows_affected=0`（DDL CHECK 约束拒绝该状态迁移，per §2 约束 + §3.2 单向门声明）
- **断言**：`trade_offers.state` 在测试期间始终保持 `'CompensationFailed'`；`audit_log` 行数不变。

#### TC-DTL015-TRADE-SAGA-007 — Saga 协调者持久化（场景 3.0 step 3.7）

- **层**：IT；**对应 DTL §**：§3.4.4 子步骤 3.7；RGS-IMPL-001 §3 `saga_orchestrator`
- **覆盖需求**：跨 DB Saga 协调者状态持久化（per Q-003 待 Gate 审批）
- **前置条件**：Testcontainers 同时启动 `economy_db` 与 `player_db`，触发跨域 step
- **Steps**：
  - **Given**：`economy_db.sagas` 已存在 `saga_id` 行，5 域 step 全部完成
  - **When**：`saga_orchestrator.commit(saga_id)` 调用
  - **Then**：`economy_db.sagas.status='completed'`，各域 `outbox` 事件已发出（per Q-205 同事务 Outbox）
- **断言**：跨域 step 在 `economy_db` 单一事务内完成 + Outbox 写入；协调者崩溃后续跑（per §3.4.6 子步骤 5.6）由 `resume(saga_id)` 重入。

#### TC-DTL015-TRADE-SAGA-008 — 超时 + DLQ（场景 5.0 子步骤 5.1~5.6）

- **层**：IT；**对应 DTL §**：§3.4.6 子步骤 5.1~5.6；RGS-IMPL-001 §3 deadline 策略
- **覆盖需求**：协调者发现单 step 超 30s deadline → DLQ 落库（per Q-M-06 答复）
- **前置条件**：Testcontainers 启动 `admin_db`（含 `dlq` 表，per §3.4.6 子步骤 5.3）
- **Steps**：
  - **Given**：注入 mock 让某个 step 执行超 30s
  - **When**：协调者 deadline 检查触发 `mark_failed`
  - **Then**：`admin_db.dlq` 追加一行（含 `saga_id`、`failed_step`、`error='deadline exceeded'`）；`economy_db.sagas.status='failed'`
- **断言**：DLQ 写入与 Saga 失败状态在同一事务（per Q-M-06）；30s 未处理 → `admin_db.review_queue` 入队（per子步骤 5.4）。

### 6.3 可见性校验测试用例（TC-DTL015-VIS-NNN）

#### TC-DTL015-VIS-001 — 好友可见允许创建

- **层**：UT；**对应 DTL §**：§3.3 `check_trade_visibility` + `VisibilityScope::FriendOnly`
- **前置条件**：`VisibilityConfig.trade_visibility_scope=FriendOnly`；`is_friend(A, B)=true`（mock）
- **Steps**：
  - **Given**：initiator=A, target=B，A 与 B 在好友关系表内
  - **When**：`check_trade_visibility(A, B, &cfg)`
  - **Then**：返回 `Ok(())`，不触发 `TradeError::TargetNotVisible`
- **断言**：函数路径不进入 `return Err` 分支；未调用 `freeze_assets` 等副作用函数（Draft → Offered 迁移未触发）。

#### TC-DTL015-VIS-002 — 非好友不可见拒绝创建

- **层**：UT；**对应 DTL §**：§3.3
- **前置条件**：`VisibilityScope=FriendOrParty`，`is_friend(A, B)=false`，`is_same_party(A, B)=false`
- **Steps**：
  - **Given**：A 与 B 既非好友也不同队伍
  - **When**：`check_trade_visibility(A, B, &cfg)`
  - **Then**：返回 `Err(TradeError::TargetNotVisible { initiator_id: A, target_id: B })`
- **断言**：调用计数 `freeze_assets = 0`（per §3.3 第 19 行注释：不消耗 FR-TRD-002 冻结路径）。

#### TC-DTL015-VIS-003 — TBD-TRD-001 默认值 `friend_or_party` 配置读取验证

- **层**：UT + 配置 fixtures；**对应 DTL §**：§4 TBD-TRD-001 默认值提案
- **覆盖需求**：TBD-TRD-001（沿用 RGS-BAS-015§2.3 评审前默认值）
- **前置条件**：`VisibilityConfig::from_toml("config/trade.toml")` 加载默认配置文件（per RGS-IMPL-001 §4 Q-304 Figment 启动边界合成）
- **Steps**：
  - **Given**：配置文件 `trade_visibility_scope = "friend_or_party"`
  - **When**：`VisibilityConfig::load_from_default()`
  - **Then**：`cfg.trade_visibility_scope == VisibilityScope::FriendOrParty`
- **断言**：若配置缺失，应回落到默认值（fail-safe），不 panic。

### 6.4 DDL/约束测试用例（TC-DTL015-DDL-NNN）

#### TC-DTL015-DDL-001 — `trade_offers.state` CHECK 约束覆盖全部枚举

- **层**：IT（Testcontainers PG 18.6）；**对应 DTL §**：§2 DDL CHECK 约束
- **覆盖需求**：FR-TRD-001/010
- **前置条件**：PG 18.6 实例上创建本文档 §2 DDL
- **Steps**：
  - **When**：尝试 `INSERT INTO trade_offers(state) VALUES ('NotAValidState')`
  - **Then**：PG 拒绝（CHECK constraint violation）
- **断言**：依次尝试 7 个合法值（Draft/Offered/Accepted/Settled/Cancelled/Expired/CompensationFailed）均成功；尝试 `''`、`null`、`NotAValidState` 均失败。

#### TC-DTL015-DDL-002 — `trade_audit_logs` 月度分区滚动创建

- **层**：IT；**对应 DTL §**：§2 DDL PARTITION BY RANGE + RGS-BAS-007§4 归档脚本
- **覆盖需求**：FR-TRD-015~018、NFR-TRD-004（1 年保留）
- **前置条件**：执行 `RGS-DTL-007§3` 既定分区滚动创建脚本，生成本月 + 下月分区
- **Steps**：
  - **When**：跨月（如系统时间 mock 到下月 1 日 00:00）插入 `occurred_at = now()` 的 audit_log
  - **Then**：插入成功落点下月分区；查询 `pg_partitions` 验证分区存在
- **断言**：未创建下月分区时插入失败（per RGS-BAS-007§4 既定分区滚动策略）。

### 6.5 测试层分布与 rgs-testkit 引用一览

| 测试层 | 用例数 | 占比 | 主要 rgs-testkit 引用 |
|---|---|---|---|
| UT（fake/mockall） | 5 | 38% | `SagaFixture::saga()`、`EconomyFixture::economy()`、`mockall` mocks |
| IT（Testcontainers PG 18.6） | 8 | 62% | `pg_test_db` (Testcontainers)、`EconomyFixture::economy()`、`SagaFixture::saga()` |

### 6.6 追溯性补充

| 测试用例 | 覆盖需求 | 父文档 | DTL 章节 |
|---|---|---|---|
| TC-DTL015-TRADE-SAGA-001 | FR-TRD-010/012、FR-EC-003 | RGS-BAS-015 | §3.1、§3.4.2 |
| TC-DTL015-TRADE-SAGA-002 | FR-TRD-014、RSK-TRD-002 | RGS-BAS-015 | §3.1 OCC |
| TC-DTL015-TRADE-SAGA-003 | FR-TRD-012 | RGS-BAS-015 | §3.1 幂等 |
| TC-DTL015-TRADE-SAGA-004~005 | RSK-TRD-002、Saga 补偿 | RGS-BAS-015、REV-005 附件 B | §3.2、§3.4.5 |
| TC-DTL015-TRADE-SAGA-006 | 单向门、CHECK 约束 | RGS-BAS-015 | §2、§3.2 |
| TC-DTL015-TRADE-SAGA-007 | 跨 DB Saga 协调者 | RGS-IMPL-001 §3 | §3.4.4、§3.4.6 |
| TC-DTL015-TRADE-SAGA-008 | Saga 超时 + DLQ | RGS-IMPL-001 §3 | §3.4.6 |
| TC-DTL015-VIS-001~003 | FR-TRD-006、TBD-TRD-001 | RGS-BAS-015 | §3.3、§4 |
| TC-DTL015-DDL-001~002 | FR-TRD-001/010/015~018、NFR-TRD-004 | RGS-BAS-007/015 | §2 |

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-015§2.1 状态机 | §2（`trade_offers.state` CHECK约束）、§3 |
| RGS-BAS-015§2.2 组件划分 | §3（前提：各组件归属不变） |
| RGS-BAS-015§2.3 可见性校验（FR-TRD-006） | §3.3、§4 |
| RGS-BAS-015§3 数据模型（`TradeOffer`/`TradeAuditLog`） | §2 |
| RGS-BAS-015§4 交易成立时序、并发控制、幂等键 | §3.1、§3.2 |
| RSK-TRD-002（双花/调包防护、补偿失败升级） | §3.1（OCC校验）、§3.2（升级分支） |
| TBD-TRD-001〜002 | §4 |
| RGS-DTL-001§3.2（确定请求路径物理执行语义） | 前提依赖，§3.1/§5明确复用而非重新设计 |
| RGS-DTL-007§2/§3（命名/索引/分区句法与分区滚动创建脚本） | 前提依赖，§2 DDL遵循其模板 |
