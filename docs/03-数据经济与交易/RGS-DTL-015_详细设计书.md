# 详细设计书（詳細設計書 / Detailed Design Document）

**玩家间交易系统：物理数据库设计・Saga编排伪代码・并发防护详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-015 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-015 玩家间交易系统 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档接续RGS-DTL-001/002/025/026/027批次，与RGS-DTL-007/016同批次产出）。细化RGS-BAS-015§2状态机/组件设计与§3逻辑数据模型为EC限界上下文内`trade_offers`／`trade_audit_logs`两表具体DDL（复用RGS-DTL-007§2既定命名/索引/分区句法），§4交易成立时序落实为`TradeSettlementSaga`可直接翻译为Rust实现的伪代码（含快照OCC校验、补偿路径、`CompensationFailed`升级分支），§2.3可见性校验落实为具体配置读取与拒绝路径伪代码（含TBD-TRD-001/002两项参数默认值提案）。**本版本不覆盖**：GM人工核账队列UI、`TradeOfferService`挂单创建/撤销的完整HTTP/gRPC协议线格式细节（仅给出关键字段，非完整IDL）。见§5 | 全部 |

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
