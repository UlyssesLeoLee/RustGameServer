# 详细设计书（詳細設計書 / Detailed Design Document）

**账号身份、第三方登录与合规：身份联合物理数据库设计・IdP验证协议格式・合规判定算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-018 |
| 版本 | 0.3 |
| 父文档 | RGS-BAS-018 账号身份、第三方登录与合规 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定，本文档是RGS-DTL-001/002/025/026/027之后本批次继续推进详细设计阶段的一部分。细化RGS-BAS-018§2.2逻辑数据模型为`AccountIdentityLink`／`IdentityBindingAuditLog`／`ComplianceProfile`／`IdentityVerificationVault`／`MinorRestrictionAuditLog`五表具体DDL、§3第三方登录时序与§3.2 IdP降级落实为可直接翻译为Rust实现的伪代码、§4合规规则引擎判定逻辑落实为具体算法。**本版本不覆盖**：各IdP（Apple/Google/Steam...）适配子模块各自的SDK调用细节、`ComplianceRuleSet`各地区具体规则的填值（TBD-IDN-001）。见§7 | 全部 |
| 0.3 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008）| — | **同步父 BAS-018 升版至 v0.3**（2 次升版，BAS-018 v0.2 + v0.3 装饰性升版）: 本 DTL 是父 BAS 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-018 既有章节内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。 审批留空，待 Ulysses 在 review 时签发。 | (父 BAS 升版章节) |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-018§2.2/§2.4/§4.2/§4.3逻辑字段表一一对应 |
| 评审（安全） | | | `IdentityVerificationVault`加密存储伪代码是否存在明文落盘风险，解绑校验/未成年人限制是否存在服务器侧可绕过路径 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：身份联合与合规五表](#2-物理数据库设计身份联合与合规五表)
3. [第三方登录协议格式](#3-第三方登录协议格式)
4. [解绑前置校验与IdP降级算法详细设计](#4-解绑前置校验与idp降级算法详细设计)
5. [合规规则引擎判定算法详细设计](#5-合规规则引擎判定算法详细设计)
6. [本文档的覆盖范围与后续计划](#6-本文档的覆盖范围与后续计划)
7. [追溯性](#7-追溯性)

---

# 1. 前言

## 1.1 定位

RGS-BAS-018给出了`AccountIdentityLink`/`IdentityBindingAuditLog`/`ComplianceProfile`/`IdentityVerificationVault`/`MinorRestrictionAuditLog`的逻辑字段表、第三方登录与IdP降级的文字时序、合规规则引擎的配置结构描述。本文档将其落实为可执行DDL、`IdPTokenVerifier`对外交互的具体协议格式、解绑校验/IdP降级/合规判定的算法级伪代码。

## 1.2 本文档不做什么

- 不重新决定RGS-BAS-018已确定的任何结构性选择（解绑前置校验必须服务器侧强制、绑定冲突不触发数据合并、实名认证原始凭证独立存储于`IdentityVerificationVault`、`IdentityVerificationVault`与`MinorRestrictionAuditLog`共享同一独立权限域）。
- 不覆盖各IdP（Apple/Google/Steam等）适配子模块各自与官方SDK交互的具体调用代码——RGS-BAS-018§2.1已确定"每个IdP一个适配子模块"这一结构，各子模块内部实现依赖各平台官方文档，不属于本文档统一详细设计的对象。
- 不填充`ComplianceRuleSet`各地区具体规则值（TBD-IDN-001）——本文档只固定该配置表的物理结构，具体地区规则须经法务评审后另行录入，不由本文档代为决定。

## 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，IdP交互协议以HTTP+JSON风格给出（IdP官方接口通常为HTTP，非本系统内部gRPC路径），算法伪代码可直接对应Rust `Result`实现。

---

# 2. 物理数据库设计：身份联合与合规五表

对应RGS-BAS-018§2.2/§2.4/§4.2/§4.3。五表依附既有GW/PL限界上下文数据库，不新建独立库（沿用RGS-BAS-018§1"全部组件依附既有GW/PL限界上下文运行"的既定约束）。

```sql
-- 账号-第三方身份绑定表，对应§2.2
CREATE TABLE account_identity_links (
    link_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id       UUID NOT NULL,           -- 逻辑引用player_db.accounts，跨库不建物理FK
    idp_type         TEXT NOT NULL CHECK (idp_type IN ('apple', 'google', 'steam')),  -- 枚举随新IdP接入追加取值，不删改既有取值
    idp_subject_id   TEXT NOT NULL,
    bound_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_identity_links_idp UNIQUE (idp_type, idp_subject_id)
    -- 数据库层唯一约束是FR-IDN-006冲突检测的物理兜底，对应RGS-BAS-018§2.2/§3.1既定设计
);
CREATE INDEX idx_identity_links_account ON account_identity_links (account_id);
-- 支撑§2.3解绑前置查询"该account_id关联的AccountIdentityLink记录数"、BR-IDN-003自助管理列表

-- 绑定/解绑审计留痕，对应§2.4
CREATE TABLE identity_binding_audit_logs (
    log_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL,
    action          TEXT NOT NULL CHECK (action IN ('bind', 'unbind', 'bind_rejected_conflict')),
    idp_type        TEXT NOT NULL,
    idp_subject_id  TEXT NOT NULL,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    client_context  JSONB   -- 可选，设备/IP等，供RGS-REQ-019客服工单调查使用
);
CREATE INDEX idx_identity_binding_audit_account_time ON identity_binding_audit_logs (account_id, occurred_at);

-- 合规判定结果（派生值，不含原始凭证），对应§2.4
CREATE TABLE compliance_profiles (
    account_id          UUID PRIMARY KEY,
    verification_status TEXT NOT NULL DEFAULT '未认证'
                            CHECK (verification_status IN ('未认证', '已认证', '认证中')),
    age_bracket          TEXT,             -- 年龄区间枚举，可空
    restriction_flags     BIGINT NOT NULL DEFAULT 0,  -- 位标志，业务服务只读查询本列
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 实名认证原始凭证独立存储，对应§4.2，与compliance_profiles物理隔离(不同表，可配置不同数据库角色权限)
CREATE TABLE identity_verification_vault (
    account_id       UUID PRIMARY KEY,
    encrypted_payload BYTEA NOT NULL,   -- 加密算法/密钥管理复用RGS-REQ-010既有Secrets管理范围，本表不重复设计加密算法本身
    access_log_ref    UUID,             -- 关联最近一次访问审计记录
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- 数据库角色权限：仅专设合规/法务角色对本表有SELECT权限，业务服务角色对本表无任何权限授予
-- (物理落实§4.2"独立评审分配，不与日常运营查询权限复用"，权限矩阵的GRANT语句随RGS-DTL-002§5数据库开通脚本模式在部署时执行)

-- 未成年人保护限制触发/解除留痕，对应§4.3
CREATE TABLE minor_restriction_audit_logs (
    log_id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id        UUID NOT NULL,
    action             TEXT NOT NULL CHECK (action IN ('restriction_triggered', 'restriction_lifted')),
    restriction_type    TEXT NOT NULL,
    trigger_reason        TEXT NOT NULL,
    occurred_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_minor_restriction_audit_account ON minor_restriction_audit_logs (account_id, occurred_at);
-- 访问权限与identity_verification_vault共享同一独立权限域(§4.3既定"复用同等独立权限域")，
-- 数据库角色授权与上方vault表使用同一角色，不新建第二套角色定义
```

**§2.4 vs §4.3表分离的强调**：`identity_binding_audit_logs`与`minor_restriction_audit_logs`是两张独立的表——RGS-BAS-018§4.3原文已明确"不得合并入同一张表混淆审计对象"，本DDL严格遵循，不因两表结构相似（均为`log_id`/`account_id`/`action`/`occurred_at`模式）而合并，这是对既有决定的物理落实而非本文档的新判断。

---

# 3. 第三方登录协议格式

对应RGS-BAS-018§2.1/§3。`IdPTokenVerifier`对外为客户端提供的登录接口采用既有gRPC路径（复用RGS-DTL-001§4.2既定的`PlayerService`协议契约与`ResultCode`枚举，不另立一套错误码体系）：

```protobuf
message ThirdPartyLoginRequest {
  string idp_type          = 1;   // "apple"/"google"/"steam"，同account_identity_links.idp_type CHECK约束
  string idp_token          = 2;   // 客户端从IdP取得的原始身份令牌
  string device_id           = 3;
  IntentOnUnbound intent       = 4;  // GUEST_UPGRADE(游客转正) 或 NEW_ACCOUNT(新账号)，未绑定时依此判定分支
}
enum IntentOnUnbound {
  UNSPECIFIED = 0;
  GUEST_UPGRADE = 1;
  NEW_ACCOUNT = 2;
}
message ThirdPartyLoginResponse {
  string account_id      = 1;
  int64  session_epoch    = 2;   // 复用RGS-DTL-001既定ARC-005 epoch机制，登录建立会话同一套epoch分配
  ResultCode result_code    = 3;
  // 新增结果码扩展(复用RGS-DTL-001§4.4 ResultCode枚举编号纪律，新增值置于后续可用编号，不复用/变更既有值)
  // IDP_TOKEN_INVALID = 20; IDP_SERVICE_UNAVAILABLE = 21; IDENTITY_CONFLICT = 22;
}
```

**结果码复用说明**：本文档不重新定义一套独立的错误码枚举，而是在RGS-DTL-001§4.4已固定的`ResultCode`基础上追加新值——`ResultCode`枚举字段编号一经分配不得变更，本文档追加值使用20起的未分配编号，遵循同一编号纪律。

---

# 4. 解绑前置校验与IdP降级算法详细设计

对应RGS-BAS-018§2.3/§3.2。

## 4.1 解绑前置校验（§2.3落地）

```rust
fn precheck_unbind(account_id: AccountId, unbind_link_id: LinkId) -> Result<(), UnbindError> {
    let remaining_links = count_identity_links(account_id, exclude = Some(unbind_link_id));
    let is_guest_capable = query_account_guest_capability(account_id);
    let remaining_login_methods = remaining_links + if is_guest_capable { 1 } else { 0 };

    if remaining_login_methods == 0 {
        // 服务器侧强制拒绝——不存在"客户端已提示但仍放行"的路径，本函数是解绑请求处理的**唯一**入口校验点
        return Err(UnbindError::WouldLeaveNoLoginMethod);
    }
    Ok(())
}

fn execute_unbind(account_id: AccountId, link_id: LinkId) -> Result<(), UnbindError> {
    precheck_unbind(account_id, link_id)?;  // 校验与执行在同一函数调用链内，不允许调用方跳过precheck直接执行
    delete_identity_link(link_id)?;
    append_audit_log(account_id, AuditAction::Unbind, link_id)?;
    Ok(())
}
```

## 4.2 IdP不可用降级（§3.2落地）

```rust
fn verify_idp_token(idp_type: &str, token: &str) -> Result<IdpSubjectId, IdpVerifyError> {
    let mut attempt = 0;
    loop {
        match call_idp_verify_endpoint(idp_type, token) {
            Ok(subject_id) => return Ok(subject_id),
            Err(IdpCallError::TokenInvalid) => {
                // 4xx明确拒绝: 不重试，直接返回区分明确的错误
                return Err(IdpVerifyError::TokenInvalid);
            }
            Err(IdpCallError::ServiceUnavailable) if attempt < MAX_IDP_RETRY => {
                // 超时/5xx: 按ARC-009标准消费者重试参数量级重试有限次数
                attempt += 1;
                sleep(RETRY_BACKOFF.next(attempt));
                continue;
            }
            Err(IdpCallError::ServiceUnavailable) => {
                return Err(IdpVerifyError::ServiceUnavailable);  // 重试耗尽，区别于TokenInvalid的独立错误类型
            }
        }
    }
}

fn handle_login_after_verify_failure(err: IdpVerifyError, account_hint: Option<&AccountHint>) -> LoginGuidance {
    match err {
        IdpVerifyError::TokenInvalid => LoginGuidance::RejectWithMessage("令牌无效".into()),
        IdpVerifyError::ServiceUnavailable => {
            match account_hint {
                Some(hint) if hint.is_guest_capable || hint.has_other_available_idp() => {
                    LoginGuidance::SuggestAlternative  // §3.2"提示改用其他登录方式"
                }
                _ => {
                    emit_alert("idp_unavailable_no_alternative", account_hint);  // 复用RGS-BAS-003§6告警通道
                    LoginGuidance::TemporarilyUnavailable
                }
            }
        }
    }
}
```

**关键边界条件说明**：`TokenInvalid`（4xx）与`ServiceUnavailable`（超时/5xx）两类错误在整个调用链路中**全程保持独立错误类型**，不在任何环节合并为单一"验证失败"——这是RGS-BAS-018§3"区分令牌无效与IdP服务不可用"要求在实现层的落实，合并两者会导致玩家在IdP临时故障时收到"令牌无效"这一误导性提示，反复尝试无效重登。

---

# 5. 合规规则引擎判定算法详细设计

对应RGS-BAS-018§4.1/§4.3。

```rust
fn evaluate_compliance(account_id: AccountId, region: &str, context: ComplianceCheckContext) -> Result<u64, ComplianceError> {
    let rule_set = load_compliance_rule_set(region);  // 复用ARC-016热更新分发通道取得的配置
    let profile = load_or_init_compliance_profile(account_id)?;
    let mut new_flags = profile.restriction_flags;
    let mut changes: Vec<(RestrictionType, ChangeDirection, &str)> = vec![];

    if rule_set.require_real_name && profile.verification_status != VerificationStatus::Verified {
        new_flags |= RestrictionFlag::PAYMENT_DISABLED;
        // 注: 是否记为触发事件取决于此前flag是否已置位,下方统一diff判定,不在此处重复写入审计
    }

    if let Some(limit_minutes) = rule_set.minor_playtime_limit_minutes {
        if let Some(age) = profile.age_bracket.as_minor() {
            let today_playtime = query_today_playtime_minutes(account_id);  // 复用既有游玩时长统计
            if today_playtime >= limit_minutes {
                new_flags |= RestrictionFlag::PLAYTIME_LIMIT;
            } else if context.is_daily_reset {
                new_flags &= !RestrictionFlag::PLAYTIME_LIMIT;  // 周期重置解除
            }
        }
    }

    let diff = diff_restriction_flags(profile.restriction_flags, new_flags);
    for (rtype, direction) in diff {
        let reason = describe_trigger_reason(rtype, direction, &context);
        changes.push((rtype, direction, reason));
    }

    if !changes.is_empty() {
        // §4.3: flags变更与审计留痕在同一事务内完成，不允许"flags已更新、审计未写"的中间态
        persist_profile_and_audit(account_id, new_flags, &changes)?;
    }

    Ok(new_flags)
}

// 全部业务服务(付费/游玩时长统计)必须调用本函数的只读查询版本获取当前生效flags，
// 不得各自重新实现§4.1判定逻辑(RGS-BAS-018§4.1"ARC-036③"既定要求的实现落点)
fn get_current_restriction_flags(account_id: AccountId) -> u64 {
    load_compliance_profile(account_id).restriction_flags
}
```

`persist_profile_and_audit`的事务边界设计沿用RGS-DTL-001§3.2既定的"同一事务内完成关联写入"模式（`compliance_profiles.restriction_flags`更新与`minor_restriction_audit_logs`追加写入同一数据库事务），保证§4.3"必须同步写入"这一强约束在物理层不可分割。

---

# 6. 本文档的覆盖范围与后续计划

本文档覆盖：身份联合与合规五表（`account_identity_links`/`identity_binding_audit_logs`/`compliance_profiles`/`identity_verification_vault`/`minor_restriction_audit_logs`）物理DDL、第三方登录的gRPC协议格式（复用RGS-DTL-001既定`ResultCode`编号纪律追加新值）、解绑前置校验与IdP降级的完整伪代码、合规规则引擎判定算法与flags/审计同事务写入的实现。

本版本明确不覆盖、留待后续：

- 各IdP（Apple/Google/Steam等）适配子模块内部与官方SDK交互的具体调用代码——依赖各平台官方文档，不属于本文档统一详细设计对象。
- `ComplianceRuleSet`各地区具体规则值的填充（TBD-IDN-001）——须经法务评审，本文档只固定配置表结构。
- `identity_verification_vault.encrypted_payload`所用具体加密算法与密钥轮换机制——复用RGS-REQ-010既有Secrets管理范围，其密钥管理基础设施本身的详细设计（若尚未产出）不在本文档重复。
- 数据库角色权限的具体`GRANT`/`REVOKE`语句——遵循RGS-DTL-002§5既定数据库开通脚本模式，本文档只声明权限隔离原则，具体脚本随各库挂载时生成。

后续详细设计建议顺序：与RGS-DTL-017/020/021同批次并行推进。

---

# 7. 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-018§2.1 组件划分 | §3 |
| RGS-BAS-018§2.2 数据模型 | §2 |
| RGS-BAS-018§2.3 解绑前置校验 | §4.1 |
| RGS-BAS-018§2.4 绑定/解绑审计字段 | §2 |
| RGS-BAS-018§3 第三方登录时序 | §3 |
| RGS-BAS-018§3.1 绑定冲突处理 | §2（唯一约束） |
| RGS-BAS-018§3.2 IdP降级 | §4.2 |
| RGS-BAS-018§4.1 配置驱动结构 | §5 |
| RGS-BAS-018§4.2 独立访问权限 | §2（vault表权限设计） |
| RGS-BAS-018§4.3 未成年人限制留痕 | §2、§5 |
| RGS-DTL-001（ResultCode编号纪律复用） | §3 |
