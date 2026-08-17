# 基本设计书（基本設計書 / Basic Design Document）

**账号身份、第三方登录与合规 Account Identity, Third-Party Login & Compliance**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-018 |
| 版本 | 0.3 |
| 父文档 | RGS-REQ-021 需求定义书（ARC-036） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-08-16 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响需求ID |
|---|---|---|---|---|---|
| 0.1 | 2026-08-16 | 架构师 | — | 初版制定。将RGS-REQ-021§8 ARC-036展开为身份联合组件设计、第三方登录时序、合规规则引擎设计 | 全部 |
| 0.2 | 2026-08-16 | 架构师 | — | 补强字段级细节：①补充解绑"至少保留一种登录方式"的字段级校验逻辑（FR-IDN-005）②补充绑定/解绑审计日志字段与实名认证信息独立访问权限设计（FR-IDN-007、FR-IDN-013）③补充第三方IdP不可用时的降级时序（RSK-IDN-002） | FR-IDN-005、FR-IDN-007、FR-IDN-013、RSK-IDN-002 |
| 0.3 | 2026-08-17 | 架构师 | — | 审计发现FR-IDN-014（未成年人保护限制的触发与解除留痕）此前仅在§4.1提及`restriction_flags`写入，无独立审计留痕设计，与FR-IDN-007/§2.4已有的绑定/解绑审计留痕待遇不一致。新增§4.3 `MinorRestrictionAuditLog`设计，复用既有独立权限域治理思想 | FR-IDN-014 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-16 | — |
| 评审（技术） | | | 第三方IdP验证失败/超时的降级路径是否完备 |
| 审批（负责人） | | | 本文档的基准化 |

---

## 目录

1. [前言](#1-前言)
2. [身份联合组件设计](#2-身份联合组件设计)
3. [第三方登录时序](#3-第三方登录时序)
4. [合规规则引擎设计](#4-合规规则引擎设计)
5. [标准化检查清单](#5-标准化检查清单)
6. [追溯性](#6-追溯性)

---

# 1. 前言

本文档细化RGS-REQ-021定义的ARC-036，全部组件依附既有GW（网关）/PL（玩家）限界上下文运行，不新建独立限界上下文。

---

# 2. 身份联合组件设计

## 2.1 组件划分

| 组件 | 归属限界上下文 | 职责 |
|---|---|---|
| `IdentityFederationService` | GW/PL | 管理账号与第三方身份的绑定关系，处理绑定/解绑/冲突拒绝 |
| `IdPTokenVerifier` | GW | 服务器侧向各IdP验证身份令牌签名与有效期，每个IdP一个适配子模块 |
| `ComplianceRuleEngine` | PL | 实名认证与未成年人保护的集中判定组件（ARC-036③） |

## 2.2 数据模型（逻辑字段）

`AccountIdentityLink`：

| 字段 | 类型 | 说明 |
|---|---|---|
| `account_id` | 玩家账号ID | 游戏账号 |
| `idp_type` | enum(`apple`／`google`／`steam`／...) | 第三方身份提供方类型 |
| `idp_subject_id` | string | IdP侧的用户唯一标识，全局唯一索引（用于FR-IDN-006冲突检测） |
| `bound_at` | timestamp | 绑定时间 |
| `is_guest_capable` | bool，冗余于`account_id`维度 | 该账号是否仍保留游客登录能力（无需第三方身份即可登录），供FR-IDN-005解绑校验使用 |

索引/约束：`(idp_type, idp_subject_id)`复合唯一索引（FR-IDN-006冲突检测的数据库层兜底）；`(account_id)`索引支撑"查询某账号已绑定的全部第三方身份"（FR-IDN-005解绑前置查询、BR-IDN-003自助管理列表）。

### 2.3 解绑前置校验（FR-IDN-005落地）

`IdentityFederationService`处理解绑请求时，须先计算"解绑后该账号剩余可登录方式数"：`count(该account_id关联的AccountIdentityLink记录，排除本次待解绑记录) + (is_guest_capable ? 1 : 0)`。计算结果为0时拒绝本次解绑，返回明确错误（"至少保留一种登录方式"），**不得**仅在客户端做提示性拦截——校验必须在服务器侧强制执行，防止客户端被绕过后产生无法登录的账号（同ARC-005服务器权威原则）。

### 2.4 绑定/解绑审计留痕字段（FR-IDN-007）

`IdentityBindingAuditLog`（复用RGS-BAS-003§7审计设计存储结构）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `log_id` | uuid | 唯一标识 |
| `account_id` | 玩家账号ID | 操作对象账号 |
| `action` | enum(`bind`／`unbind`／`bind_rejected_conflict`) | 含FR-IDN-006冲突拒绝事件，供账号被盗申诉调查时区分"正常解绑"与"曾有冲突尝试" |
| `idp_type` / `idp_subject_id` | — | 涉及的第三方身份 |
| `occurred_at` | timestamp | 操作时间 |
| `client_context` | 可选，设备/IP等上下文 | 供RGS-REQ-019客服工单调查账号被盗场景使用 |

`ComplianceProfile`（若启用实名认证）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `account_id` | 玩家账号ID | — |
| `verification_status` | enum(`未认证`／`已认证`／`认证中`) | — |
| `age_bracket` | enum，可选 | 认证得到的年龄区间（非精确年龄，最小化存储） |
| `restriction_flags` | 位标志，可选 | 当前生效的限制（如禁止付费、时长限制） |

---

# 3. 第三方登录时序

```
客户端发起第三方IdP登录，获得IdP签发的身份令牌
  → 客户端将令牌转发至GW
  → IdPTokenVerifier依idp_type选择对应验证子模块，向IdP验证令牌签名与有效期
  → 验证失败 → 拒绝登录，返回明确错误（区分"令牌无效"与"IdP服务不可用"）
  → 验证成功 → 取得idp_subject_id
      → 查询AccountIdentityLink：
          已存在绑定 → 以关联的account_id建立会话（复用既有FR-GW-002令牌验证与会话建立）
          不存在绑定且客户端声明"游客转正" → 触发FR-IDN-003转正流程，写入AccountIdentityLink，不丢失游客账号既有进度
          不存在绑定且客户端声明"新账号" → 创建新账号+AccountIdentityLink
```

## 3.1 绑定冲突处理（FR-IDN-006落地）

写入`AccountIdentityLink`前对`idp_subject_id`做唯一性校验（数据库唯一索引兜底），冲突时返回明确错误，**不触发**任何账号数据合并逻辑；如玩家认为冲突判定有误（如误以为该第三方身份未绑定过），走RGS-REQ-019客服工单人工处理。

## 3.2 IdP不可用时的降级（RSK-IDN-002落地）

```
IdPTokenVerifier向IdP发起验证请求超时/IdP返回5xx（区别于"令牌无效"的4xx明确拒绝）
  → 按既定重试策略重试有限次数（复用ARC-009标准消费者重试参数量级，避免无限重试拖长客户端等待）
  → 仍不可用 → 返回明确错误"IdP服务暂时不可用"（区别于"令牌无效"，避免误导玩家反复尝试无效令牌）
  → 客户端侧引导：若账号此前已具备游客登录能力（is_guest_capable=true）或已绑定其他仍可用的IdP，提示改用其他登录方式
  → 若账号仅绑定该不可用IdP且无游客能力，登录暂不可用，记录告警（复用RGS-BAS-003§6），供运维判断是否需要发布公告
```

---

# 4. 合规规则引擎设计

## 4.1 配置驱动结构（FR-IDN-010落地）

`ComplianceRuleSet`（配置表，复用ARC-016热更新机制分发）：

| 字段 | 说明 |
|---|---|
| `region` | 适用地区 |
| `require_real_name` | 是否要求实名认证 |
| `verification_method` | 验证方式（身份证号校验/人脸识别/…） |
| `minor_playtime_limit_minutes` | 未成年人日游玩时长上限，为空表示不限制 |
| `restricted_features_for_unverified` | 未认证账号的功能限制清单 |

`ComplianceRuleEngine`在账号登录/关键操作（如付费）前查询玩家所属地区对应的`ComplianceRuleSet`，结合`ComplianceProfile`做判定，判定结果写入`restriction_flags`。**全部业务服务**（付费、游玩时长统计）**必须**通过查询`restriction_flags`而非各自重新实现判定逻辑（ARC-036③）。

## 4.2 实名认证信息的独立访问权限（FR-IDN-013落地）

`ComplianceProfile`中涉及个人信息的字段（若`verification_method`要求存储身份证号等原始凭证，该原始凭证**不**在`ComplianceProfile`本表明文存储，而是加密存储于独立的`IdentityVerificationVault`，`ComplianceProfile`仅保留`verification_status`/`age_bracket`/`restriction_flags`等派生判定结果）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `account_id` | 玩家账号ID | 主键 |
| `encrypted_payload` | 加密二进制 | 原始认证凭证，加密复用既有NFR-SE-012个人信息保护标准（密钥管理复用RGS-REQ-010既有Secrets管理范围） |
| `access_log_ref` | 关联访问审计记录 | 每次解密访问须先落审计记录（复用RGS-BAS-003§7） |

访问权限：`IdentityVerificationVault`的解密访问权限**独立**评审分配，**不**与日常运营查询权限（如GM后台的一般客服角色）复用同一角色定义，仅限专设的合规/法务角色访问，访问前须记录申请理由（同RGS-BAS-017§3.5"分析管线独立访问权限"同类治理思想的应用）。

## 4.3 未成年人保护限制触发/解除留痕（FR-IDN-014落地）

`ComplianceRuleEngine`每次变更`ComplianceProfile.restriction_flags`（无论是因认证得到年龄信息后首次判定、达到`minor_playtime_limit_minutes`时长上限触发、还是周期重置解除）**必须**同步写入`MinorRestrictionAuditLog`（复用RGS-BAS-003§7审计设计存储结构；与§2.4`IdentityBindingAuditLog`记录对象不同——一个是身份绑定操作、一个是合规限制状态变更，**不得**合并入同一张表混淆审计对象）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `log_id` | uuid | 唯一标识 |
| `account_id` | 玩家账号ID | 受限账号 |
| `action` | enum(`restriction_triggered`／`restriction_lifted`) | 触发或解除 |
| `restriction_type` | 对应`restriction_flags`位标志 | 具体限制类型（如`playtime_limit`／`payment_disabled`） |
| `trigger_reason` | string，简述 | 如"当日游玩时长达到配置上限""周期重置解除" |
| `occurred_at` | timestamp | 触发/解除时间 |

`MinorRestrictionAuditLog`的访问权限**复用**本节既定的`IdentityVerificationVault`同等独立权限域（仅限专设合规/法务角色访问），**不得**与日常运营查询权限混同，供合规审计使用（FR-IDN-014、NFR-IDN-003同类要求的延伸）。

---

# 5. 标准化检查清单

## 5.1 上线前检查清单

- [ ] 伪造第三方令牌的拒绝测试通过
- [ ] 游客转正无进度丢失验证通过
- [ ] 账号绑定冲突拒绝验证通过，且不触发数据合并
- [ ] 目标发行地区的合规规则（TBD-IDN-001）已配置并评审确定
- [ ] 未成年人时长限制服务器侧强制执行验证通过（若适用）
- [ ] 解绑后剩余登录方式数为0时的拒绝校验（§2.3）已服务器侧验证，客户端拦截不作为唯一防线
- [ ] IdP不可用降级路径（§3.2）已模拟验证，错误提示区分"令牌无效"与"服务不可用"
- [ ] `IdentityVerificationVault`访问权限已独立分配，未与日常GM运营权限复用

## 5.2 代码评审检查清单

- [ ] 未出现仅信任客户端第三方身份声明、跳过服务器侧IdP验证的路径
- [ ] 业务服务未各自重新实现合规判定逻辑，均查询`ComplianceRuleEngine`结果

---

# 6. 追溯性

| 需求ID | 本设计书章节 |
|---|---|
| ARC-036、FR-IDN-001〜007 | §2、§2.3（解绑校验）、§2.4（审计字段）、§3 |
| FR-IDN-010〜014 | §4、§4.2（独立访问权限）、§4.3（未成年人限制触发/解除留痕） |
| NFR-IDN-001〜004 | §3、§4.1 |
| AC-IDN-001〜004 | §5.1 |
| TBD-IDN-001〜002、RSK-IDN-001〜002 | §5.1、§3.2（IdP降级） |

---

> 本文档与RGS-REQ-021（账号身份、第三方登录与合规 需求定义书）配套使用。
