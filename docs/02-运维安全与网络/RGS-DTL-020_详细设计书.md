# 详细设计书（詳細設計書 / Detailed Design Document）

**平台内购合规与服务器选服：内购与选服物理数据库设计・收据校验协议格式・重试与合服算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-020 |
| 版本 | 0.3 |
| 父文档 | RGS-BAS-020 平台内购合规与服务器选服 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定，本文档是RGS-DTL-001/002/025/026/027之后本批次继续推进详细设计阶段的一部分。细化RGS-BAS-020§2.4/§2.5待重试队列与`PaymentOrder`扩展字段为具体DDL、§2.2/§2.3收据校验与退款时序落实为可直接翻译为Rust实现的伪代码、§3.3 `realm_id`归属键原则落实为示例表字段追加DDL、§4.1冲突解决规则配置表落实为具体DDL与合并算法伪代码（含TBD重试阈值的初始默认值提案）。**本版本不覆盖**：各平台（App Store/Google Play）收据验证适配子模块的具体SDK调用、`realm_id`字段在全部既有业务表的逐一列举（多服架构启用后另行确定）。见§7 | 全部 |
| 0.3 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | **同步父 BAS-020 升版至 v0.3**（2 次升版，BAS-020 v0.2 + v0.3 装饰性升版）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-020 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | 待重试队列DDL与幂等键设计是否与RGS-DTL-001§3幂等去重表标准一致 |
| 评审（DBA） | | | 合服冲突配置表索引是否覆盖`merge_job_id`维度查询，`realm_id`纳入主键的示例是否可直接指导后续各域详细设计 |
| 审批（负责人） | | | 本文档的基准化；重试阈值默认值提案是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：内购与选服相关表](#2-物理数据库设计内购与选服相关表)
3. [收据校验协议格式](#3-收据校验协议格式)
4. [待重试队列与退款处理算法详细设计](#4-待重试队列与退款处理算法详细设计)
5. [realm_id归属键落地示例](#5-realmid归属键落地示例)
6. [合服冲突处理算法详细设计](#6-合服冲突处理算法详细设计)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)
8. [追溯性](#8-追溯性)

---

# 1. 前言

## 1.1 定位

RGS-BAS-020给出了`PendingReceiptVerification`/`PaymentOrder`扩展字段/`MergeConflictRuleSet`的逻辑字段表、收据校验与退款处理的文字时序、`realm_id`归属键原则的文字描述。本文档将其落实为可执行DDL、`ReceiptVerifier`对外协议格式、待重试队列扫描与合服冲突解决的算法级伪代码。

## 1.2 本文档不做什么

- 不重新决定RGS-BAS-020已确定的任何结构性选择（`PaymentOrder`共享同一套数据模型不新建独立表、沙盒/生产环境不匹配须拒绝、`realm_id`不新建独立数据库/Schema而是纳入既有表主键、合服演练与正式执行读取同一份锁定配置）。
- 不覆盖各平台（App Store/Google Play）收据验证适配子模块内部与官方SDK交互的具体调用代码。
- 不逐一列举`realm_id`字段需追加到的全部既有业务表——RGS-BAS-020§3.3已明确"具体到哪些既有表结构需要补充`realm_id`字段...须在多服架构确定启用后，由各自领域的BAS文档在详细设计阶段补齐字段清单"，本文档仅给出归属键落地的**示例模式**（§5），供各业务域详细设计时套用，不越权替各域决定其表结构。

## 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，协议格式以Protobuf风格给出，算法伪代码可直接对应Rust `Result`实现。

---

# 2. 物理数据库设计：内购与选服相关表

对应RGS-BAS-020§2.4/§2.5/§4.1。依附既有PL/EC/AD限界上下文数据库，不新建独立库。

```sql
-- 平台校验待重试队列，对应§2.4，复用RGS-DTL-001§3.2幂等去重表设计标准
CREATE TABLE pending_receipt_verifications (
    pending_id      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    raw_receipt     BYTEA NOT NULL,          -- 加密存储，密钥管理复用RGS-REQ-010既有Secrets管理范围
    platform_type   TEXT NOT NULL CHECK (platform_type IN ('app_store', 'google_play')),
    retry_count     INT NOT NULL DEFAULT 0,
    next_retry_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'resolved', 'abandoned')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_pending_receipt_scan ON pending_receipt_verifications (next_retry_at)
    WHERE status = 'pending';
    -- 支撑§2.4定时任务扫描路径"status=pending AND next_retry_at<=now()"

-- PaymentOrder平台内购扩展字段，对应§2.5，ALTER既有RGS-BAS-016§3.1表(不新建独立表，遵循FR-PLT-005)
ALTER TABLE payment_orders
    ADD COLUMN payment_channel      TEXT NOT NULL DEFAULT 'direct_gateway'
        CHECK (payment_channel IN ('platform_iap', 'direct_gateway')),
    ADD COLUMN platform_type         TEXT
        CHECK (platform_type IS NULL OR platform_type IN ('app_store', 'google_play')),
    ADD COLUMN platform_environment    TEXT
        CHECK (platform_environment IS NULL OR platform_environment IN ('sandbox', 'production')),
    ADD COLUMN refund_status             TEXT NOT NULL DEFAULT 'none'
        CHECK (refund_status IN ('none', 'refunded', 'clawback_pending', 'clawback_done'));

CREATE UNIQUE INDEX uq_payment_orders_platform_txn
    ON payment_orders (platform_type, provider_txn_id)
    WHERE platform_type IS NOT NULL;
    -- 部分唯一索引：仅platform_iap订单参与本约束，direct_gateway订单的provider_txn_id语义不同不纳入本约束范围
    -- 对应§2.5"确保跨平台交易标识不产生误关联"

-- 合服冲突解决规则配置表，对应§4.1
CREATE TABLE merge_conflict_rule_sets (
    merge_job_id                   UUID PRIMARY KEY,
    character_name_conflict_rule    TEXT NOT NULL
        CHECK (character_name_conflict_rule IN ('auto_rename_with_suffix', 'require_manual_rename_on_login')),
    unique_item_conflict_rule        TEXT NOT NULL
        CHECK (unique_item_conflict_rule IN ('stack_additively', 'keep_both_as_separate', 'keep_earliest_and_compensate')),
    currency_conflict_rule            TEXT NOT NULL DEFAULT 'sum' CHECK (currency_conflict_rule = 'sum'),
    approved_by                        TEXT NOT NULL,
    approved_at                         TIMESTAMPTZ NOT NULL DEFAULT now(),
    locked                                BOOLEAN NOT NULL DEFAULT FALSE
    -- locked=true后§6.1流程禁止UPDATE本行(应用层强制，见§6.1"不得临时调整规则")
);
```

---

# 3. 收据校验协议格式

对应RGS-BAS-020§2.1/§2.2。`ReceiptVerifier`对客户端的收据提交接口复用既有gRPC路径，字段编号延续RGS-DTL-001既定纪律：

```protobuf
message SubmitReceiptRequest {
  string request_id       = 1;   // 幂等键，编号1最高频访问字段(同RGS-DTL-001§4.3 CommitTransactionRequest惯例)
  string raw_receipt        = 2;
  string platform_type       = 3;  // "app_store"/"google_play"
  string platform_environment = 4;  // 客户端上报环境，服务器侧仍以平台官方验证返回的环境为准做比对(不信任本字段单独决策)
}
message SubmitReceiptResponse {
  ResultCode result_code = 1;
  string ledger_id         = 2;   // 成功时对应EconomyService流水(复用RGS-DTL-001§3.2确定请求API)
  string failure_category    = 3;  // invalid_signature/already_used/sandbox_prod_mismatch/pending_retry，对应§2.2失败分类
}
```

`failure_category = "pending_retry"`时，客户端应理解为"已受理，正在后台重试"而非"已拒绝"——对应§2.4"验证接口不可用→不判定为欺诈，投递至待重试队列"，客户端不应在此结果码下提示玩家收据无效。

---

# 4. 待重试队列与退款处理算法详细设计

对应RGS-BAS-020§2.2/§2.3/§2.4。

## 4.1 收据提交主流程

```rust
fn submit_receipt(req: &SubmitReceiptRequest) -> Result<SubmitReceiptResponse, ReceiptError> {
    match verify_with_platform(&req.raw_receipt, &req.platform_type) {
        Ok(verified) => {
            if verified.environment != expected_environment_for_deployment() {
                // §2.5环境不匹配须拒绝，不进入发放路径
                return Ok(reject_response("sandbox_prod_mismatch"));
            }
            let existing = find_payment_order_by_provider_txn(&verified.platform_type, &verified.txn_id);
            match existing {
                Some(order) => Ok(existing_order_response(order)),  // 幂等：直接返回既有结果，不重复处理
                None => {
                    let order = create_payment_order(req, &verified)?;
                    grant_entitlement_via_commit_transaction(&order)?;  // 复用RGS-DTL-001§3.2 CommitTransaction确定请求路径
                    Ok(success_response(order))
                }
            }
        }
        Err(PlatformVerifyError::Rejected(category)) => {
            append_audit_log(req, &category)?;  // §2.2"记录审计日志，含失败原因分类"
            Ok(reject_response(&category))
        }
        Err(PlatformVerifyError::Unavailable) => {
            enqueue_pending_verification(req)?;  // §2.4：不判定欺诈，投递待重试队列
            Ok(pending_retry_response())
        }
    }
}
```

## 4.2 待重试队列扫描与终态转移

```rust
fn scan_and_retry_pending_receipts(now: Instant, max_retry: u32) -> Result<(), RetryScanError> {
    let due = query_pending_receipts_due(now);  // 走idx_pending_receipt_scan索引
    for pending in due {
        match verify_with_platform(&pending.raw_receipt, &pending.platform_type) {
            Ok(verified) => {
                complete_receipt_via_normal_path(&pending, &verified)?;  // 进入§4.1正常发放路径
                mark_pending_resolved(pending.pending_id)?;
            }
            Err(PlatformVerifyError::Rejected(category)) => {
                // 重试期间平台明确拒绝(而非仍不可用): 转拒绝路径，不再继续重试同一条
                mark_pending_resolved(pending.pending_id)?;  // 已有明确结论，退出待重试状态
                append_audit_log_for_pending(&pending, &category)?;
            }
            Err(PlatformVerifyError::Unavailable) => {
                let next_count = pending.retry_count + 1;
                if next_count > max_retry {
                    mark_pending_abandoned(pending.pending_id)?;
                    open_support_ticket(pending.pending_id, "payment_issue")?;  // 转人工，§2.4既定
                } else {
                    reschedule_pending(pending.pending_id, next_count, exponential_backoff(next_count))?;
                }
            }
        }
    }
    Ok(())
}
```

## 4.3 重试阈值参数提案（TBD，非最终值）

RGS-BAS-020§2.4"超过最大重试次数（详细设计确定阈值）"在本文档给出初始提案：

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| 最大重试次数`max_retry` | 10 | 覆盖典型平台接口短时故障（数小时级）的恢复窗口，配合指数退避避免过早放弃合法收据 |
| 指数退避基数 | 初始1分钟，倍增至上限1小时 | 与RGS-DTL-025§5同类做法一致，量级参考ARC-009标准消费者重试参数 |

以上数值为初始提案，非最终值，应在上线后按实际平台接口故障时长分布校准。

## 4.4 退款处理

```rust
fn handle_refund_notification(notification: &PlatformRefundNotification) -> Result<(), RefundError> {
    verify_notification_signature(notification)?;  // §2.3"校验通知来源真实性"
    let order = find_payment_order_by_provider_txn(&notification.platform_type, &notification.txn_id)
        .ok_or(RefundError::OrderNotFound)?;
    let clawback_method = resolve_clawback_method(&order);  // TBD-PLT-001：扣除等价物/标记负债/不追回，本文档不代为选定
    execute_clawback(&order, clawback_method)?;
    update_refund_status(order.id, RefundStatus::ClawbackDone)?;
    append_audit_log_clawback(&order, clawback_method)?;  // 复用RGS-BAS-003§7审计设计
    Ok(())
}
```

`resolve_clawback_method`的具体判定逻辑留空——TBD-PLT-001尚未决定追回方式本身，本文档不越权代为选定，函数签名固定输出类型供实现阶段接入判定结果。

---

# 5. realm_id归属键落地示例

对应RGS-BAS-020§3.3。本文档**仅**给出示例模式，不逐一列举各业务表：

```sql
-- 示例：假设RGS-DTL-001§2 characters表所属域启用多服架构后的追加变更形态(实际是否变更、
-- 何时变更由该域自身DTL文档在TBD-PLT-002评审通过后决定，此处仅作为落地示例，不代表已生效)
-- ALTER TABLE characters ADD COLUMN realm_id UUID NOT NULL;
-- ALTER TABLE characters DROP CONSTRAINT uq_characters_name;
-- ALTER TABLE characters ADD CONSTRAINT uq_characters_name_per_realm UNIQUE (realm_id, name);
--   -- 角色名唯一性范围从全局收窄为"同一realm_id内唯一"，因不同逻辑服允许同名角色(§3.3"相互独立")
```

```rust
// realm_id校验的服务器侧强制点(§3.3"不得由客户端自行声明realm_id")
fn validate_realm_scope(session: &SessionContext, requested_realm_id: RealmId) -> Result<(), RealmScopeError> {
    if session.realm_id != requested_realm_id {
        // 会话建立时(RealmRouter路由完成后)已将realm_id写入会话上下文，
        // 后续业务请求若携带的realm_id与会话不一致，一律拒绝，不信任请求体中的realm_id单独判定
        return Err(RealmScopeError::CrossRealmAccessDenied);
    }
    Ok(())
}
```

---

# 6. 合服冲突处理算法详细设计

对应RGS-BAS-020§4.1/§4.2。

```rust
fn execute_merge_job(job_id: MergeJobId, mode: MergeMode) -> Result<MergeReport, MergeError> {
    let rule_set = load_merge_conflict_rule_set(job_id)?;
    if !rule_set.locked {
        // §4.1"须完成评审并锁定后，方可进入演练"——未锁定的规则不得用于任何执行(演练或正式)
        return Err(MergeError::RuleSetNotLocked);
    }

    let conflicts = detect_conflicts(job_id);  // 扫描待合并服的角色名/唯一道具/货币
    let mut report = MergeReport::default();

    for conflict in conflicts {
        match conflict {
            Conflict::CharacterName { .. } => apply_name_rule(&conflict, &rule_set.character_name_conflict_rule, &mut report)?,
            Conflict::UniqueItem { .. } => apply_item_rule(&conflict, &rule_set.unique_item_conflict_rule, &mut report)?,
            Conflict::Currency { .. } => apply_currency_sum(&conflict, &mut report)?,  // 固定累加，无需查规则(currency_conflict_rule恒为sum)
        }
    }

    if mode == MergeMode::Trial {
        verify_asset_total_consistency(&report)?;  // §4.2步骤2"核对资产总量前后一致"
        // Trial模式不提交实际数据变更，仅在演练环境执行并产出报告，避免与正式执行混淆代码路径
    }

    Ok(report)
}
```

**"演练与正式执行代码路径不得分叉"的边界条件**：`execute_merge_job`对`Trial`与`Formal`两种`mode`共用**同一段**冲突检测与规则应用逻辑，仅在最后是否提交持久化变更上分叉——这是RGS-BAS-020§5.2代码评审检查项"合服执行代码未跳过步骤2演练直接进入步骤4正式执行"得以被静态验证的实现前提：若演练与正式执行是两套独立实现，该检查项将无法可靠核实两者行为一致。

---

# 7. 本文档的覆盖范围与后续计划

本文档覆盖：`pending_receipt_verifications`/`payment_orders`扩展字段/`merge_conflict_rule_sets`三表（含1表ALTER）的物理DDL、收据提交gRPC协议格式、收据校验主流程与待重试队列扫描/终态转移的完整伪代码（含TBD重试阈值初始提案）、退款处理流程、`realm_id`归属键落地的示例模式与服务器侧强制校验、合服冲突解决算法（演练/正式共用同一代码路径的设计）。

本版本明确不覆盖、留待后续：

- 各平台（App Store/Google Play）收据验证适配子模块内部与官方SDK交互的具体调用代码。
- `realm_id`字段需追加到的全部既有业务表清单——按RGS-BAS-020§3.3既定，须待TBD-PLT-002多服架构启用评审通过后，由各业务域自身DTL文档补齐，本文档§5仅提供示例模式。
- TBD-PLT-001退款追回方式（扣除等价物/标记负债/不追回）的最终选定——本文档`resolve_clawback_method`函数签名留空占位。
- §4.3重试阈值参数的正式校准——当前为初始提案，需上线后实测数据支撑。

后续详细设计建议顺序：与RGS-DTL-017/018/021同批次并行推进。

> **v0.3 升版补登（per ULYS-1 后续测试设计补全任务）**：原 v0.1 修订历史第 0.1 行明确"本版本不覆盖：各平台（App Store/Google Play）收据验证适配子模块的具体SDK调用、`realm_id`字段在全部既有业务表的逐一列举（多服架构启用后另行确定）"——这两项在 v0.3 仍属本文档范围外（SDK 调用属实现阶段；`realm_id` 字段清单待 TBD-PLT-002 评审后补齐），但**收据校验（含沙盒/生产不匹配拒绝）、待重试队列扫描与终态转移、退款通知校验、`realm_id` 服务器侧强制、合服冲突解决（演练/正式共用代码路径）** 的测试设计已在 v0.3 由本文档§8「后续测试设计补全」落实，补齐 v0.1 审计报告 RGS-REV-001/REV-002 标记的"§X 测试设计不覆盖"缺口。本节末声明该事实，不修改原不覆盖列表项本身。

---

# 8. 后续测试设计补全（v0.3 test-design follow-up，per ULYS-1）

## 8.1 范围与既有占位

原 v0.1 仅给出"各平台 SDK 调用、`realm_id` 全部业务表清单"两类不覆盖项；v0.1/v0.3 均**未**为本文档 §3 收据协议、§4 待重试/退款、§5 `realm_id`、§6 合服配套测试设计。本次补全聚焦 v0.1 已落实的逻辑（DTL 既有章节正文）的可测试设计点，覆盖范围：

| 序号 | 测试主题 | 对应本文档章节 | 测试层 | 关联需求 |
|---|---|---|---|---|
| TC-DTL020-RCPT-001~005 | 收据校验主流程 + 沙盒/生产不匹配 + 幂等 | §3 / §4.1 | UT/IT | FR-PLT-001~005 |
| TC-DTL020-RETRY-001~004 | 待重试队列扫描 + 终态转移 + 阈值升级 | §4.2 / §4.3 | UT/IT | NFR-PLT-002 |
| TC-DTL020-REFUND-001~002 | 退款通知 + 追回方式分发 | §4.4 | UT/IT | FR-PLT-010/011 |
| TC-DTL020-REALM-001~002 | `realm_id` 服务器侧强制 + 跨服拒绝 | §5 | UT | §3.3、FR-RLM-001 |
| TC-DTL020-MERGE-001~003 | 合服冲突解决 + 演练/正式代码路径 | §6 | UT/IT | FR-PLT-020~022 |

每条测试用例均遵循 RFC 2119 强度用语（per `RGS-TST-UT-06 §1.4.1`），引用 `crates/rgs-testkit` 既有 fixture（per `RGS-IMPL-001 §3 Q-206`）。

## 8.2 收据校验测试用例（TC-DTL020-RCPT-NNN）

### TC-DTL020-RCPT-001 — 正常收据提交：发放 + ledger_id 返回

- **层**：IT（含 mock App Store/Google Play SDK）；**对应 DTL §**：§3 `SubmitReceiptRequest` + §4.1 `submit_receipt`
- **覆盖需求**：FR-PLT-001（内购合规）、FR-EC-003（确定请求路径）
- **前置条件**：mock 平台 SDK 返回 `verified` 且 `environment=production`；`PlayerFixture::player("A")`、`EconomyFixture::economy("A")`
- **Steps**：
  - **Given**：`request_id='req-001'`，`raw_receipt` 合法，`platform_type='app_store'`，部署环境为 production
  - **When**：`submit_receipt(req)`
  - **Then**：
    - `find_payment_order_by_provider_txn` 返回 `None`（首次提交）
    - `create_payment_order(req, &verified)` 成功
    - `grant_entitlement_via_commit_transaction(&order)` 调用（per §4.1 第 11 行）
    - 返回 `Ok(success_response(order))`，`ledger_id` 非空
- **断言**：`payment_orders` 1 行新增；玩家权益已发放；`failure_category` 为空。

### TC-DTL020-RCPT-002 — 沙盒/生产环境不匹配拒绝

- **层**：IT；**对应 DTL §**：§4.1 第 6-9 行；§2.5 环境不匹配须拒绝
- **覆盖需求**：§2.5（沙盒/生产不匹配须拒绝）、FR-PLT-002
- **前置条件**：部署环境为 production；mock SDK 返回 `environment=sandbox`
- **Steps**：
  - **Given**：`verified.environment != expected_environment_for_deployment()`，即 sandbox vs production
  - **When**：`submit_receipt(req)`
  - **Then**：
    - 返回 `Ok(reject_response("sandbox_prod_mismatch"))`（per §4.1 第 8 行）
    - `failure_category="sandbox_prod_mismatch"`
    - **不**进入 `create_payment_order` 与发放路径
    - **不**新增 `payment_orders` 行
- **断言**：§2.5 强约束"环境不匹配须拒绝"生效；客户端应理解为"拒绝"而非"待重试"。

### TC-DTL020-RCPT-003 — 幂等键 `(request_id)` 重复提交返回既有结果

- **层**：IT；**对应 DTL §**：§3 `request_id` 字段 + §4.1 `find_payment_order_by_provider_txn` 第 9-12 行
- **覆盖需求**：NFR-PLT-001（幂等键）、§3 字段 1 编号最高频访问
- **前置条件**：已存在 `payment_orders` 由同一 `(platform_type, provider_txn_id)` 创建
- **Steps**：
  - **Given**：`request_id='req-001'`，`provider_txn_id='TXN-A'` 已存在
  - **When**：再次以相同 `request_id` 提交同一收据
  - **Then**：
    - `existing_order_response(order)` 返回（per §4.1 第 11 行）
    - **不**重复发放权益
    - **不**新增 `payment_orders` 行
- **断言**：`payment_orders` 行数与 Given 前一致；幂等键避免双重发放。

### TC-DTL020-RCPT-004 — 平台明确拒绝：记录审计 + 返回失败分类

- **层**：UT；**对应 DTL §**：§4.1 第 17-21 行；§2.2 失败原因分类
- **覆盖需求**：§2.2（记录审计日志含失败原因分类）
- **前置条件**：mock 平台 SDK 返回 `PlatformVerifyError::Rejected("invalid_signature")`
- **Steps**：
  - **Given**：收据签名验证失败
  - **When**：`submit_receipt(req)`
  - **Then**：
    - `append_audit_log(req, "invalid_signature")` 被调用
    - 返回 `Ok(reject_response("invalid_signature"))`
    - **不**进入待重试队列（明确拒绝路径）
- **断言**：审计留痕；与"不可用 → 待重试"路径区分（per §4.1 第 23-26 行）。

### TC-DTL020-RCPT-005 — 平台不可用：投递待重试队列（不判定欺诈）

- **层**：IT；**对应 DTL §**：§4.1 第 23-26 行
- **覆盖需求**：§2.4（验证接口不可用 → 不判定为欺诈）
- **前置条件**：mock 平台 SDK 返回 `PlatformVerifyError::Unavailable`
- **Steps**：
  - **Given**：平台 SDK 超时或 5xx
  - **When**：`submit_receipt(req)`
  - **Then**：
    - `enqueue_pending_verification(req)` 被调用
    - `pending_receipt_verifications` 1 行新增（`status='pending'`, `retry_count=0`）
    - 返回 `Ok(pending_retry_response())`
    - **不**调用 `append_audit_log`（明确非拒绝，per §3 注释）
- **断言**：客户端应理解为"已受理，正在后台重试"而非"已拒绝"（per §3 第 7 行注释）。

## 8.3 待重试队列测试用例（TC-DTL020-RETRY-NNN）

### TC-DTL020-RETRY-001 — 扫描路径走 `idx_pending_receipt_scan` 部分索引

- **层**：IT；**对应 DTL §**：§2 DDL `idx_pending_receipt_scan ON pending_receipt_verifications (next_retry_at) WHERE status = 'pending'`
- **覆盖需求**：§2.4 定时任务扫描路径
- **前置条件**：PG 18.6 实例，构造 N=1000 行，其中 500 行 `status='resolved'`，500 行 `status='pending'` 且部分 `next_retry_at <= now()`
- **Steps**：
  - **When**：执行扫描 query（`SELECT * FROM pending_receipt_verifications WHERE status='pending' AND next_retry_at <= now() ORDER BY next_retry_at`）
  - **Then**：查询走 `idx_pending_receipt_scan` 部分索引（验证 `EXPLAIN`）；已 resolved 行**不**参与扫描
- **断言**：部分索引过滤掉 500 行 resolved 记录；扫描成本降低。

### TC-DTL020-RETRY-002 — 重试期间平台明确拒绝：转终态不再重试

- **层**：UT；**对应 DTL §**：§4.2 `scan_and_retry_pending_receipts` 第 11-15 行
- **覆盖需求**：§4.2 明确结论转拒绝路径
- **前置条件**：mock 重试过程中平台返回 `PlatformVerifyError::Rejected(category)`
- **Steps**：
  - **Given**：pending receipt 进入扫描周期，平台明确拒绝
  - **When**：`scan_and_retry_pending_receipts(now, max_retry=10)`
  - **Then**：
    - `mark_pending_resolved(pending_id)` 被调用（**不**继续重试）
    - `append_audit_log_for_pending(&pending, &category)` 被调用
    - `status` 从 `'pending'` 迁移到 `'resolved'`
- **断言**：明确结论退出待重试状态（per §4.2 第 12 行注释）。

### TC-DTL020-RETRY-003 — 重试次数超 `max_retry=10`：转人工工单

- **层**：IT；**对应 DTL §**：§4.2 第 16-21 行 + §4.3 `max_retry=10`
- **覆盖需求**：§2.4（超最大重试次数转人工）
- **前置条件**：mock 让某 pending receipt 连续 `Unavailable`，retry_count=10
- **Steps**：
  - **Given**：`retry_count=10`，`max_retry=10`；本次重试仍返回 `Unavailable`
  - **When**：扫描触发
  - **Then**：
    - `next_count = 10+1 = 11`，`11 > 10`，进入超限分支
    - `mark_pending_abandoned(pending_id)` 被调用
    - `open_support_ticket(pending_id, "payment_issue")` 被调用
    - `support_tickets.category='payment_issue'` 写入
- **断言**：超限转人工（per §4.3 `max_retry=10` 提案值）；不再重试。

### TC-DTL020-RETRY-004 — 指数退避：1 分钟 → 1 小时上限

- **层**：UT；**对应 DTL §**：§4.2 第 23 行 `exponential_backoff(next_count)` + §4.3 提案
- **覆盖需求**：§4.3 退避基数
- **前置条件**：`exponential_backoff` 函数，初始 1 分钟，倍增至上限 1 小时
- **Steps**：
  - **Given**：`next_count ∈ {1, 2, 3, 7, 8, 9, 10}`
  - **When**：依次调用
  - **Then**：`backoff(1)=1m`, `backoff(2)=2m`, `backoff(3)=4m`, ...，`backoff(7)=64m ≈ 1h`（触上限），`backoff(8..10)=1h`（饱和）
- **断言**：符合 §4.3 提案"初始 1 分钟，倍增至上限 1 小时"。

## 8.4 退款处理测试用例（TC-DTL020-REFUND-NNN）

### TC-DTL020-REFUND-001 — 退款通知：签名校验 + 追回执行

- **层**：IT；**对应 DTL §**：§4.4 `handle_refund_notification` 第 1-6 行
- **覆盖需求**：FR-PLT-010（退款处理）、§2.3 校验通知来源真实性
- **前置条件**：mock 平台退款通知（已签名）；`PlayerFixture::player("A")`、`EconomyFixture::economy("A")`
- **Steps**：
  - **Given**：`notification.platform_type='app_store'`，`txn_id='TXN-A'`，`payment_orders` 存在
  - **When**：`handle_refund_notification(notification)`
  - **Then**：
    - `verify_notification_signature(notification)` 通过
    - `find_payment_order_by_provider_txn` 返回订单
    - `resolve_clawback_method(&order)` 触发（占位，per §4.4 第 6 行注释）
    - `execute_clawback(&order, clawback_method)` 被调用
    - `update_refund_status(order.id, RefundStatus::ClawbackDone)` 成功
    - `append_audit_log_clawback(&order, clawback_method)` 写入审计
- **断言**：退款链路完整；审计留痕；`refund_status` 终态迁移。

### TC-DTL020-REFUND-002 — `resolve_clawback_method` 占位：TBD-PLT-001 未决时函数签名固定

- **层**：UT；**对应 DTL §**：§4.4 第 6-8 行 + §4.4 末尾注释
- **覆盖需求**：TBD-PLT-001（追回方式占位）
- **前置条件**：`resolve_clawback_method` 函数已实现但 TBD-PLT-001 未决
- **Steps**：
  - **Given**：TBD-PLT-001 待财务/法务评审
  - **When**：调用 `resolve_clawback_method(&order)`
  - **Then**：返回占位值（per §4.4 末尾注释"函数签名固定输出类型供实现阶段接入判定结果"）；后续 `execute_clawback` 接收占位值
- **断言**：占位不影响调用链路；函数签名稳定供实现阶段接入。

## 8.5 `realm_id` 服务器侧强制测试用例（TC-DTL020-REALM-NNN）

### TC-DTL020-REALM-001 — `realm_id` 与会话一致：通过

- **层**：UT；**对应 DTL §**：§5 `validate_realm_scope` 第 6-12 行
- **覆盖需求**：§3.3（不信任客户端单独声明）
- **前置条件**：`session.realm_id='realm-001'`，`requested_realm_id='realm-001'`
- **Steps**：
  - **Given**：会话与请求的 `realm_id` 一致
  - **When**：`validate_realm_scope(&session, requested_realm_id)`
  - **Then**：返回 `Ok(())`
- **断言**：合法同服请求通过。

### TC-DTL020-REALM-002 — `realm_id` 与会话不一致：跨服拒绝

- **层**：UT；**对应 DTL §**：§5 第 8-11 行
- **覆盖需求**：§3.3（不得由客户端自行声明 realm_id）
- **前置条件**：`session.realm_id='realm-001'`，`requested_realm_id='realm-002'`
- **Steps**：
  - **Given**：客户端请求携带其他 realm_id
  - **When**：`validate_realm_scope(&session, requested_realm_id)`
  - **Then**：返回 `Err(RealmScopeError::CrossRealmAccessDenied)`（per §5 第 11 行）
- **断言**：服务器侧强制，**不**信任请求体中的 `realm_id` 单独判定（per §5 第 7-10 行注释）。

## 8.6 合服冲突解决测试用例（TC-DTL020-MERGE-NNN）

### TC-DTL020-MERGE-001 — 未锁定规则集拒绝执行（演练与正式均拒绝）

- **层**：UT；**对应 DTL §**：§6 `execute_merge_job` 第 4-9 行
- **覆盖需求**：§4.1（须完成评审并锁定后方可执行）
- **前置条件**：`merge_conflict_rule_sets.locked=false`
- **Steps**：
  - **Given**：规则集未锁定（`locked=false`）
  - **When A**：`execute_merge_job(job_id, MergeMode::Trial)` 演练模式
    - **Then**：返回 `Err(MergeError::RuleSetNotLocked)`
  - **When B**：`execute_merge_job(job_id, MergeMode::Formal)` 正式模式
    - **Then**：同样返回 `Err(MergeError::RuleSetNotLocked)`
- **断言**：演练与正式均拒绝未锁定规则（per §6 第 7 行注释）。

### TC-DTL020-MERGE-002 — 角色名冲突按 `auto_rename_with_suffix` 自动重命名

- **层**：UT + SQL fixtures；**对应 DTL §**：§6 `apply_name_rule` + §2 `merge_conflict_rule_sets.character_name_conflict_rule`
- **覆盖需求**：§4.1（角色名冲突解决）
- **前置条件**：`rule_set.character_name_conflict_rule='auto_rename_with_suffix'`；冲突角色 = "PlayerA"
- **Steps**：
  - **Given**：两服各有一个 "PlayerA"
  - **When**：`apply_name_rule(conflict, "auto_rename_with_suffix", &mut report)`
  - **Then**：后迁移服角色被重命名为 "PlayerA#001"（或 `#002`）；`report` 含 `name_renamed: 1`
- **断言**：规则应用正确；`MergeReport` 记录变更。

### TC-DTL020-MERGE-003 — 演练模式不提交持久化变更（与正式模式代码路径一致）

- **层**：IT；**对应 DTL §**：§6 第 21-25 行 + 末尾"演练与正式执行代码路径不得分叉"边界条件
- **覆盖需求**：§5.2（演练与正式共用同一代码路径）、§4.2 步骤 2（核对资产总量）
- **前置条件**：Testcontainers 启动目标 DB；`rule_set.locked=true`
- **Steps**：
  - **Given**：`execute_merge_job` 进入 `mode=Trial`
  - **When**：执行完成
  - **Then**：
    - `verify_asset_total_consistency(&report)` 被调用
    - **不**写持久化变更（演练不提交）
    - `MergeReport` 仍产出供运营审阅
- **断言**：演练与正式仅在最后是否提交上分叉（per §6 末尾批注）；代码路径共用是 §5.2 代码评审检查项的实现前提。

## 8.7 测试层分布与 rgs-testkit 引用一览

| 测试层 | 用例数 | 占比 | 主要 rgs-testkit 引用 |
|---|---|---|---|
| UT（fake/mockall） | 9 | 56% | `PlayerFixture::player()`、`EconomyFixture::economy()`、`mockall` mocks（平台 SDK、verify_notification_signature） |
| IT（Testcontainers PG 18.6） | 7 | 44% | `pg_test_db`、`PlayerFixture::player()`、`EconomyFixture::economy()` |

## 8.8 追溯性补充

| 测试用例 | 覆盖需求 | 父文档 | DTL 章节 |
|---|---|---|---|
| TC-DTL020-RCPT-001 | FR-PLT-001、FR-EC-003 | RGS-BAS-020 | §3、§4.1 |
| TC-DTL020-RCPT-002 | §2.5、FR-PLT-002 | RGS-BAS-020 | §4.1 |
| TC-DTL020-RCPT-003 | NFR-PLT-001 | RGS-BAS-020 | §3、§4.1 |
| TC-DTL020-RCPT-004 | §2.2 | RGS-BAS-020 | §4.1 |
| TC-DTL020-RCPT-005 | §2.4 | RGS-BAS-020 | §4.1 |
| TC-DTL020-RETRY-001 | §2.4 扫描 | RGS-BAS-020 | §2、§4.2 |
| TC-DTL020-RETRY-002 | §4.2 明确结论转拒绝 | RGS-BAS-020 | §4.2 |
| TC-DTL020-RETRY-003 | §2.4 超限转人工 | RGS-BAS-020 | §4.2、§4.3 |
| TC-DTL020-RETRY-004 | §4.3 指数退避 | RGS-BAS-020 | §4.2、§4.3 |
| TC-DTL020-REFUND-001~002 | FR-PLT-010、TBD-PLT-001 | RGS-BAS-020 | §4.4 |
| TC-DTL020-REALM-001~002 | §3.3、FR-RLM-001 | RGS-BAS-020 | §5 |
| TC-DTL020-MERGE-001~003 | FR-PLT-020~022、§5.2 | RGS-BAS-020 | §6 |

---

# 8. 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-020§2.1 组件划分 | §3 |
| RGS-BAS-020§2.2 收据校验时序 | §4.1 |
| RGS-BAS-020§2.3 退款处理时序 | §4.4 |
| RGS-BAS-020§2.4 待重试队列 | §2、§4.2、§4.3 |
| RGS-BAS-020§2.5 PaymentOrder扩展字段 | §2 |
| RGS-BAS-020§3 选服路由设计 | §7（未展开，选服路由无需物理落地的新算法，复用既有会话机制） |
| RGS-BAS-020§3.3 realm_id归属键 | §5 |
| RGS-BAS-020§4.1 冲突解决规则配置 | §2、§6 |
| RGS-BAS-020§4.2 合服执行流程 | §6 |
| RGS-DTL-001（幂等去重表标准/CommitTransaction复用） | §2、§4.1 |
