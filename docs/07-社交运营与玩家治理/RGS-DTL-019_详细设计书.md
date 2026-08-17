# 详细设计书（詳細設計書 / Detailed Design Document）

**消息推送与兑换码运营工具：PL/AD限界上下文物理数据库设计・推送投递协议格式・并发防超发核销算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-019 |
| 版本 | 0.1 |
| 父文档 | RGS-BAS-019 消息推送与兑换码运营工具 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档与RGS-DTL-011/012/013/014并行产出）。细化RGS-BAS-019§3.1`RedemptionCodeBatch`/`RedemptionCode`/`RedemptionRecord`逻辑字段为PL/AD限界上下文具体DDL、§2.2推送发送时序落实为具体协议格式与Rust伪代码、§3.2核销时序（含§3.2既定"条件更新不得先读后写"强约束）落实为可直接翻译为Rust实现的具体SQL与并发防超发伪代码、TBD-OPT-002兑换码生成方式给出初始提案（同RGS-DTL-025§5既定处理方式）。**本版本不覆盖**：APNs/FCM第三方网关适配层的具体SDK调用代码、敏感信息正则模式库的具体规则集内容。见§5 | 全部 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-019§3.1逻辑字段表完全一致，`used_count`条件更新伪代码是否确实杜绝先读后写模式 |
| 评审（DBA） | | | `code`唯一索引与`(code, account_id)`幂等键索引设计是否覆盖高并发核销场景 |
| 审批（负责人） | | | 本文档的基准化；TBD-OPT-002兑换码生成方式提案是否可直接采纳 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：兑换码三表](#2-物理数据库设计兑换码三表)
3. [协议线格式：推送投递](#3-协议线格式推送投递)
4. [算法详细设计](#4-算法详细设计)
5. [本文档的覆盖范围与后续计划](#5-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-019给出了推送组件划分（`PushConsentStore`/`PushDispatcher`/`PushGatewayAdapter`/`PushContentSanitizer`）与发送时序文字流程、兑换码`RedemptionCodeBatch`/`RedemptionCode`/`RedemptionRecord`逻辑字段表与核销时序文字流程（含"条件更新而非先读后写"这一已经具体到SQL语义层面但尚未给出实际DDL/SQL语句的强约束）。本文档将其落实为可执行DDL、推送投递的具体协议格式、核销并发防超发的完整SQL与伪代码。

### 1.2 本文档不做什么

- **不重新决定**RGS-BAS-019已确定的任何结构性选择（推送发送前必须先查`PushConsentStore`、内容脱敏命中禁止模式时拒绝而非静默处理、兑换码核销幂等键为`(code, account_id)`、`used_count`必须以条件更新而非先读后写实现）。
- **不覆盖**`PushGatewayAdapter`对接APNs/FCM第三方网关的具体SDK调用代码——RGS-BAS-019§2.1已述"密钥复用既有Secrets管理"，本文档不重复展开第三方SDK集成细节，该部分随实现阶段所选SDK版本变化，非架构层面决策。
- **不覆盖**敏感信息正则模式库（邮箱/手机号/身份证号匹配规则）与违禁词库的具体规则集内容——RGS-BAS-019§2.1.1已述"复用既有日志脱敏规则集的模式库"，规则集内容本身的维护属既有脱敏基础设施职责，非本文档新增。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，协议格式以Protobuf/HTTP风格给出，算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：兑换码三表

对应RGS-BAS-019§3.1。三表落位既有PL（推送同意状态另行落位，见下）/AD（兑换码运营为GM控制平面职责，同RGS-DTL-025反作弊三表挂靠原则）限界上下文数据库，本文档只新增表结构。

```sql
-- 推送同意状态表，对应RGS-BAS-019§2.1 PushConsentStore(原文仅描述职责，未给出字段表，本文档在详细设计层面补齐最小字段集)
CREATE TABLE push_consents (
    account_id       UUID NOT NULL,
    category            TEXT NOT NULL,   -- 推送类别，如'friend_invite'/'activity_start'等，随FR-OPT-001既定类别集合扩展
    consented              BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, category)
    -- 复合主键: 一个账号对每个类别独立记录同意状态，符合RGS-BAS-019§2.1"分类别的推送同意状态"表述，
    -- 未见于原文显式字段表，但"分类别"这一形容词本身已隐含该表结构，本文档将其从文字表述提升为物理schema，
    -- 同RGS-DTL-001§2.2"角色名全局唯一"类同性质的"物理落实既有隐含要求，不算新决策"
);

-- 兑换码批次表，对应§3.1 RedemptionCodeBatch，落位AD
CREATE TABLE redemption_code_batches (
    batch_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reward_spec            JSONB NOT NULL,
    expire_at                 TIMESTAMPTZ NOT NULL,
    max_uses_per_code            INT NOT NULL DEFAULT 1 CHECK (max_uses_per_code >= 1),
    preview_confirmed_by            UUID,   -- NULL=未确认预览，批量生成前必须非空(见下方业务约束说明)
    created_at                         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 兑换码本体表，对应§3.1 RedemptionCode，落位AD
CREATE TABLE redemption_codes (
    code            VARCHAR(32) PRIMARY KEY,   -- 高熵随机生成，见§4.4 TBD-OPT-002提案
    batch_id           UUID NOT NULL REFERENCES redemption_code_batches(batch_id),
    used_count            INT NOT NULL DEFAULT 0
);
CREATE INDEX idx_redemption_codes_batch ON redemption_codes (batch_id);
    -- 支撑§3.1既定FR-OPT-015批次核销进度聚合查询
    -- (SELECT count(*), sum(used_count) ... WHERE batch_id=? GROUP BY ...)

-- 核销记录表，对应§3.1 RedemptionRecord，落位AD
CREATE TABLE redemption_records (
    code            VARCHAR(32) NOT NULL REFERENCES redemption_codes(code),
    account_id         UUID NOT NULL,
    redeemed_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (code, account_id)
    -- 复合主键即幂等键本身(§3.1既定"同一账号对同一码的核销请求")，
    -- 与§4.2核销伪代码中"INSERT INTO redemption_records ... ON CONFLICT DO NOTHING风格判定已核销"直接对应
);
```

`redemption_code_batches.preview_confirmed_by`允许在表结构上为空（不用`NOT NULL`），是因为该字段的"批量生成前必须非空"约束是**流程时序**约束（先预览确认、后生成`redemption_codes`行）而非**数据存在性**约束——若用`NOT NULL`,则`redemption_code_batches`行本身的插入时点就必须晚于确认，但实际流程是"先插入批次记录（`preview_confirmed_by`为空，此时展示预览）→ GM确认后UPDATE写入`preview_confirmed_by`→ 才允许触发`redemption_codes`批量生成"，故该约束落在应用层"批量生成前检查`preview_confirmed_by IS NOT NULL`"这一显式校验点（见§4.3），而非DB CHECK约束——这是本文档为数不多未把RGS-BAS-019的"必须"文字表述提升为DB层强制的一处，理由是该约束涉及跨越"批次记录存在"与"兑换码集合存在"两个不同的生命周期时点，不是单表内可表达的不变式。

---

## 3. 协议线格式：推送投递

对应RGS-BAS-019§2.2发送时序。

```protobuf
// PushDeliveryRequest: PushDispatcher内部构造后交由PushGatewayAdapter投递的载荷契约
// （本文档固定的是PushGatewayAdapter接口边界，而非APNs/FCM各自的私有协议格式——那属于§1.2明确排除的第三方SDK细节）
message PushDeliveryRequest {
  string account_id       = 1;
  string category           = 2;   // 对应push_consents.category，投递前已通过同意校验
  string title                 = 3;   // 已经过PushContentSanitizer校验，本消息不会携带命中禁止模式的内容
  string body                    = 4;
  int64  dedup_window_id             = 5;   // 供PushGatewayAdapter侧幂等/去重使用，值为频率限制窗口标识
}

message PushDeliveryResult {
  DeliveryResultCode result_code = 1;
}
enum DeliveryResultCode {
  DELIVERED = 0;
  DEVICE_TOKEN_EXPIRED = 1;     // 网关返回失败，不无限重试(RGS-BAS-019§2.2既定)
  RATE_LIMITED_DROPPED = 2;      // 频率限制超限且类别配置为"丢弃"而非"排队"
  RATE_LIMITED_QUEUED = 3;         // 频率限制超限，排队至下一可发送窗口
}
```

---

## 4. 算法详细设计

### 4.1 推送发送主流程（落实RGS-BAS-019§2.2/§2.1.1）

```rust
fn dispatch_push(req: &PushSendRequest, ctx: &PlContext) -> Result<PushDeliveryResult, PushError> {
    // 1. 同意校验(ARC-037①)
    let consent = ctx.query_consent(&req.account_id, &req.category)?;
    if !consent.consented {
        record_push_skipped(&req.account_id, &req.category, SkipReason::NotConsented);
        return Ok(PushDeliveryResult { result_code: DeliveryResultCode::RATE_LIMITED_DROPPED });
        // ↑ 注: 未同意场景直接丢弃不投递，本函数返回值语义为"未产生实际投递"，
        //   调用方不应将其视为错误，而是正常的策略性跳过(同RGS-DTL-001§5.1"静默丢弃，不报错"同类处理原则)
    }

    // 2. 内容脱敏校验(§2.1.1)，先于频率限制校验(拒绝发送优先于其他任何后续处理，避免敏感内容进入排队队列滞留)
    if ctx.content_sanitizer.matches_forbidden_pattern(&req.title, &req.body) {
        record_push_rejected(&req.account_id, RejectReason::SensitiveContent);  // 拒绝并记录告警(§2.1.1既定)
        return Err(PushError::SensitiveContentDetected);
        // 命中禁止模式时拒绝发送而非静默脱敏后继续发送(§2.1.1既定设计理由: 模板变量注入不该出现的字段应阻断而非静默处理)
    }

    // 3. 频率限制校验(FR-OPT-004)
    match ctx.rate_limiter.check(&req.account_id, &req.category) {
        RateLimitOutcome::Ok => {
            let result = ctx.gateway_adapter.deliver(build_delivery_request(req))?;
            if result.result_code == DeliveryResultCode::DEVICE_TOKEN_EXPIRED {
                record_push_failure(&req.account_id, FailureReason::TokenExpired);  // 不无限重试
            }
            Ok(result)
        }
        RateLimitOutcome::ExceededDrop => Ok(PushDeliveryResult { result_code: DeliveryResultCode::RATE_LIMITED_DROPPED }),
        RateLimitOutcome::ExceededQueue => {
            enqueue_for_next_window(req);  // 由类别配置决定丢弃或排队(§2.2既定)
            Ok(PushDeliveryResult { result_code: DeliveryResultCode::RATE_LIMITED_QUEUED })
        }
    }
}
```

### 4.2 兑换码核销：并发防超发（落实RGS-BAS-019§3.2核心强约束）

```sql
-- 核销主事务，单条条件更新语句杜绝先读后写竞态(RGS-BAS-019§3.2已明确规定的实现方式，本文档给出对应SQL)
BEGIN;
  -- 前置校验(不存在/已过期，返回行数为0或expire_at已过则提前拒绝，此处从略，同§4.3展开)

  -- 幂等校验: 若(code, account_id)已存在于redemption_records，直接返回既有结果
  --   （查询逻辑在应用层完成，见§4.3；本SQL块仅展开"不存在"分支的核心写入）

  UPDATE redemption_codes
  SET used_count = used_count + 1
  WHERE code = $code AND used_count < max_uses_per_code_of(batch_id)
  -- 说明: max_uses_per_code取自redemption_code_batches，本行为简化表达；
  --   实际实现须JOIN或子查询取得阈值，等价写法：
  --   UPDATE redemption_codes rc SET used_count = rc.used_count + 1
  --   FROM redemption_code_batches rb
  --   WHERE rc.code = $code AND rc.batch_id = rb.batch_id AND rc.used_count < rb.max_uses_per_code;
  RETURNING used_count;
  -- 影响行数=1: 本次递增成功，继续下方发放
  -- 影响行数=0: 已用完(或code不存在，前置校验已排除后者)，即便此前预检通过也以本次结果为准拒绝(RGS-BAS-019§3.2既定)

  INSERT INTO redemption_records (code, account_id) VALUES ($code, $account_id);
  -- 与上一步UPDATE同一事务: 若INSERT因主键冲突失败(理论上不会发生，因为进入本事务前已完成幂等校验查询，
  --   但仍以数据库约束作为纵深防御的最后一道防线，同RGS-DTL-001§3.2"唯一约束是幂等性物理强制层"同类设计精神)
  --   则事务整体回滚，上一步的used_count递增也随之回滚，不产生"计数已加但记录未写"的不一致态
COMMIT;
```

### 4.3 核销主流程（落实RGS-BAS-019§3.2完整时序，含前置校验与幂等分支）

```rust
fn redeem_code(code: &str, account_id: AccountId, ctx: &AdContext) -> Result<RedemptionOutcome, RedemptionError> {
    ctx.rate_limiter.check_multi_layer(account_id, ctx.request_ip)?;  // NFR-OPT-002多层限制(账号/IP)，复用既有NFR-SEC-008

    let record = ctx.query_code(code)?.ok_or(RedemptionError::CodeNotFound)?;
    if record.batch.expire_at < now() {
        return Err(RedemptionError::Expired);
    }

    // 幂等校验(先查询，用于快速返回已知结果；真正的并发防护在下方条件更新，本次查询不构成竞态防护本身)
    if let Some(existing) = ctx.query_redemption_record(code, account_id)? {
        return Ok(RedemptionOutcome::AlreadyRedeemed { redeemed_at: existing.redeemed_at });
    }

    // §4.2条件更新，单条SQL原子递增+边界判定，不做"先SELECT used_count再判断再UPDATE"
    let update_result = ctx.conditional_increment_used_count(code)?;  // 对应§4.2 UPDATE ... RETURNING
    match update_result {
        UpdateOutcome::Incremented => {
            ctx.insert_redemption_record(code, account_id)?;  // 同一事务，见§4.2 SQL块
            let grant = commit_transaction(build_grant_request(code, account_id, &record.batch.reward_spec))?;
            // 复用FR-EC-003确定请求路径发放(RGS-BAS-019§3.2既定，同RGS-DTL-001§3.2/RGS-DTL-013§5.4同一模式)
            Ok(RedemptionOutcome::Redeemed(grant))
        }
        UpdateOutcome::NoRowsAffected => Err(RedemptionError::AlreadyExhausted),
        // 即使此前"used_count < max_uses_per_code"的预检(若曾执行)已通过，仍以本次条件更新的实际结果为准
        // (RGS-BAS-019§3.2"并发防超发"强约束的直接体现——预检结果可能因并发已过期)
    }
}
```

### 4.4 TBD-OPT-002兑换码生成方式初始提案

RGS-BAS-019§3.1"`code`...高熵随机生成，TBD-OPT-002"——本文档提出以下初始提案，供PH阶段评审前的实现参考，非最终结论：

| 参数 | 提案默认值 | 依据 |
|---|---|---|
| 生成算法 | 128位密码学安全随机数（CSPRNG），Base32编码去除易混淆字符（`0`/`O`/`1`/`I`/`L`），固定长度16字符 | Base32较Base64更适合人工输入场景（无大小写敏感/特殊符号问题），去混淆字符表降低玩家手动输入出错率 |
| 唯一性冲突处理 | 生成后先在应用层查`redemption_codes`表是否已存在，冲突则重新生成（预期冲突概率在128位熵下可忽略不计，重试仅作纵深防御），最终落地依赖`code`主键约束的数据库层唯一性强制（同RGS-DTL-001系列"唯一约束是幂等性物理强制层"同类原则） | 避免仅依赖应用层查重导致的竞态窗口，主键约束才是真正的最后防线 |
| 批量生成规模上限（单批次） | 提案：单次批量生成不超过100万个兑换码（超出则拆分为多个`RedemptionCodeBatch`），避免单次批量INSERT事务过大影响数据库表现 | PH阶段前的保守初始值，非最终值 |

以上默认值为初始提案，非最终值，最终生成算法与批量规模上限需按实现阶段实测与安全评审确定后回写本文档新版本。

---

## 5. 本文档的覆盖范围与后续计划

本文档覆盖：`push_consents`（补齐RGS-BAS-019原文未显式给出字段表的最小字段集）/`redemption_code_batches`/`redemption_codes`/`redemption_records`四表物理DDL、`PushDeliveryRequest`/`PushDeliveryResult`推送投递具体协议格式、推送发送主流程（含同意/脱敏/频率限制三重校验顺序）与兑换码核销并发防超发的完整SQL与伪代码、TBD-OPT-002兑换码生成方式初始提案。

本版本明确不覆盖、留待后续：

- `PushGatewayAdapter`对接APNs/FCM第三方网关的具体SDK调用代码与密钥轮换细节——随实现阶段所选SDK版本变化，非架构层面决策，本文档只固定了该适配层的输入契约（`PushDeliveryRequest`）边界。
- 敏感信息正则模式库与违禁词库的具体规则集内容——复用既有日志脱敏基础设施的模式库，规则集维护属该既有基础设施职责范围，非本文档新增。
- TBD-OPT-001（RGS-BAS-019§5追溯性表已登记但正文未展开具体内容的开放问题）——本文档未见该TBD在正文中的具体描述文字，若该项确有独立技术内容待补，需先回到RGS-BAS-019正文补充其文字描述，本文档不代为猜测其内容。
- TBD-OPT-002兑换码生成方式的最终选型评审结论——本文档§4.4给出的是初始提案，非结论。
- 频率限制（`FR-OPT-004`推送频率、`NFR-OPT-002`兑换码提交速率）各层级的具体阈值数值——均为运营/安全参数，需按PH阶段实测数据与安全评审确定，本文档不给出具体数值。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-019§2.1 推送组件划分 | §2（`push_consents`）、§3 |
| RGS-BAS-019§2.1.1 推送内容脱敏校验 | §4.1 |
| RGS-BAS-019§2.2 发送时序 | §3、§4.1 |
| RGS-BAS-019§3.1 兑换码数据模型 | §2 |
| RGS-BAS-019§3.2 核销时序（含并发防超发强约束） | §4.2、§4.3 |
| TBD-OPT-002（附件/追溯性表登记，正文未展开） | §4.4 |
| RGS-DTL-001§3.2 确定请求API物理执行语义 | §4.3（复用FR-EC-003同一路径） |
| RGS-DTL-025§5（提案默认值处理方式的既定先例） | §4.4 |
