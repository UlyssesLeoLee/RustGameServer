# 基本设计书（基本設計書 / Basic Design Document）

**账号身份、第三方登录与合规 Account Identity, Third-Party Login & Compliance**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-BAS-018 |
| 版本 | 0.4 |
| 父文档 | RGS-REQ-021 需求定义书（ARC-036） |
| 制定日 | 2026-08-16 |
| 最终更新日 | 2026-09-01 |
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
| 0.4 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 落实"各BAS文档功能章节加log设计且区分debug/release级"总要求（per Ulysses 2026-09-01 15:52 JST 决策，4 拍板选项：全部 36 个BAS / 详尽版5列表 / 派worker并行 / BAS-004同步升级）：§2.1/§2.2/§2.3/§2.4/§3.1/§3.2/§4.1/§4.2/§4.3 全部 9 个 ## L2 功能段加"本功能日志设计" 5 列详尽版（字段名/触发条件/频率估算/采样策略/脱敏与成本），字段名前缀 `auth.*` 区别于既有 `mnt.*`/`gm.*`/`db.*` 命名空间；引用 BAS-001 v1.5 §4.8.3 模板（commit 32d9eb6）+ BAS-003 v0.3 样板（commit 75a001c）+ BAS-004 v0.3 §4.2 二维矩阵 + §4.3 字段 + §5.1 脱敏 + §6.2 强制全采样（commit 47e26b0/0ee6262）；覆盖账号身份第三方登录与合规域（ARC-036）"组件启动/降级 / 数据模型 CRUD / 解绑前置校验 / 绑定解绑审计 / 第三方登录冲突 / IdP 不可用降级 / 合规规则热更新 / 实名认证 Vault 访问 / 未成年人限制触发与解除"全链路；**账号身份域特殊强制**（per NFR-SE-012 + 合规要求）：登录/登出/Token 刷新/第三方登录/凭证错误/锁定/找回/实名认证/防沉迷验证 → `info!`/`warn!`/`error!` 强制全采样（release 必出，§6.2 强制全量采集范围），`auth.login.attempt.received`/`auth.login.session.established`/`auth.login.failed.no_alternative`/`auth.identity.bind.rejected.conflict`/`auth.identity.unbind.rejected.last_method`/`auth.login.idp_verification.retry.exhausted`/`auth.compliance.real_name.verification.*`/`auth.compliance.vault.access.*`/`auth.compliance.minor.restriction.*` 均为强制全采样白名单；debug-only 字段（`#[cfg(debug_assertions)]` 守护，release build 完全剔除零运行时开销）含 `auth.login.debug.request_header_dump`/`auth.login.debug.request_body_dump`/`auth.compliance.debug.vault.encrypted_payload_dump`/`auth.compliance.debug.rule_set.full_dump`；**安全/合规硬约束**（per BAS-004 v0.3 §5.1 脱敏黑名单 + §5.1 末段 IP 末段掩码）：`*token*`/`*password*`/`*credential*` 字段**禁止记录**（SDK 层面拦截），`auth.identity.client_context.ip_changed` 末段掩码（`203.0.113.0/24`），`auth.identity.client_context.geo` 仅记录粗粒度区域；§5.1 检查清单新增 6 条 log 章节上线检查项；§6 追溯性新增 AC-IDN-LOG-001（debug-only 宏 release 完全剔除）/ AC-IDN-LOG-002（每功能 BAS 文档须含本功能 log 设计章节）/ AC-IDN-LOG-003（账号身份域安全/合规字段强制全采样无遗漏），与 BAS-001 v1.5 §4.8.3.4 / BAS-003 v0.3 §13 / BAS-004 v0.3 §12 形成统一规范 | §2.1/§2.2/§2.3/§2.4/§3.1/§3.2/§4.1/§4.2/§4.3/§5.1/§6 |

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

### 2.1 本功能日志设计

本节覆盖**身份联合组件启动/降级/健康**的观察点——`IdentityFederationService` / `IdPTokenVerifier` / `ComplianceRuleEngine` 三个组件本身不产生业务事件，但组件级启动、降级模式切换、依赖健康度变化是 SRE 追踪账号登录链路稳定性的关键诊断信号。账号身份域组件涉及玩家登录入口（**NFR-SE-012 高安全**），`info!`/`warn!`/`error!` 必须 release 必出，便于事后审计账号无法登录类工单。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.component.started` | `IdentityFederationService` / `IdPTokenVerifier` / `ComplianceRuleEngine` 任一组件完成启动（含依赖校验） | 启动期 1 次/组件 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2 + NFR-SE-012） | 含`component`/`started_at`/`version`；无敏感字段；约 200B/条 |
| `auth.component.degraded` | 组件进入降级模式（如 IdP 全量不可用、ComplianceRuleSet 配置未拉取） | 极低（生产事件） | release 必出（100% 强制全采样） | 含`component`/`degrade_reason`；约 250B/条 |
| `auth.component.recovered` | 组件从降级模式恢复至正常服务 | 极低 | release 必出（100% 强制全采样） | 含`component`/`recovered_at`/`downtime_seconds`；约 250B/条 |
| `auth.component.health_check.failed` | 组件健康检查连续失败（`/healthz`/`/readyz` N 次）触发 k8s 重启判定 | 极低 | release 必出（100% 强制全采样） | 含`component`/`consecutive_failures`/`last_error`；约 300B/条 |
| `auth.component.debug.startup_dependency_dump` | 启动期各依赖（IdP endpoint / Vault endpoint / 数据库连接）解析结果 | 启动期 1 次/组件 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-2KB/条（release 剔除，避免启动日志含 endpoint 明细泄漏） |

**debug-only 守护要点**（落实 BAS-004 v0.3 §4.4）：
- `auth.component.debug.startup_dependency_dump` 可能含 IdP 回调 endpoint 明细 —— release build 完全剔除，避免 RUST_LOG=debug 误开时泄漏 IdP 集成拓扑
- `auth.component.*` 系列均为 `info!`/`warn!`/`error!` 级别（release 必出，§4.8.3.2 二维矩阵常驻），便于 SRE 按 `component` 维度聚合

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

### 2.2 本功能日志设计

本节覆盖 **`AccountIdentityLink` 数据模型 CRUD 与索引命中**的观察点——`AccountIdentityLink` 写操作（bind/unbind）受 9 个 L2 章节中事件驱动；本节聚焦"数据访问层"事件（唯一索引冲突、查询失败、迁移后表结构变化），便于 SRE/DBA 诊断账号身份域的数据库层异常。`AccountIdentityLink` 不含敏感凭证字段（`idp_subject_id` 是 IdP 侧公开 ID，非凭据），日志字段无 `*token*`/`*password*` 落入风险。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.link.unique_index_conflict` | 写入 `AccountIdentityLink` 时 `(idp_type, idp_subject_id)` 复合唯一索引冲突（FR-IDN-006 数据库层兜底） | 偶发 | release 必出（100% 强制全采样，per BAS-004 v0.3 §6.2） | 含`idp_type`/`existing_account_id_hash`（account_id 哈希化）/`attempted_at`；约 300B/条 |
| `auth.link.query.failed` | 查询某账号已绑定全部第三方身份失败（DB timeout / 连接断开） | 极低 | release 必出（100% 强制全采样） | 含`account_id_hash`/`error`/`trace_id`；约 280B/条 |
| `auth.link.migration.applied` | `AccountIdentityLink` 表结构迁移执行（含复合唯一索引创建） | 1/迁移 | release 必出（100% 强制全采样） | 含`migration_id`/`table_name`/`index_added`；约 250B/条 |
| `auth.link.migration.rolled_back` | 迁移回滚（因后续迁移失败或人工介入） | 极低 | release 必出（100% 强制全采样） | 含`migration_id`/`reason`/`rolled_back_at`；约 300B/条 |
| `auth.link.debug.row_count_snapshot` | 周期性 `AccountIdentityLink` 行数 dump（用于容量规划） | 1/h | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 100B/条（release 剔除） |
| `auth.link.debug.index_usage_dump` | 索引命中率 dump（`(idp_type, idp_subject_id)` vs `(account_id)`） | 1/h | **debug-only**（`#[cfg(debug_assertions)]` 守护） | 约 500B/条（release 剔除） |

**debug-only 守护要点**：
- `auth.link.debug.row_count_snapshot` 频率 1/h 看似低，但 5 域总和可能堆量 —— release build 完全剔除以保证零运行时开销
- `auth.link.unique_index_conflict` 包含 `existing_account_id_hash`（不暴露明文 `account_id`，哈希化便于合规审计定位而不泄漏玩家标识）—— release 必出但已脱敏



### 2.3 解绑前置校验（FR-IDN-005落地）

`IdentityFederationService`处理解绑请求时，须先计算"解绑后该账号剩余可登录方式数"：`count(该account_id关联的AccountIdentityLink记录，排除本次待解绑记录) + (is_guest_capable ? 1 : 0)`。计算结果为0时拒绝本次解绑，返回明确错误（"至少保留一种登录方式"），**不得**仅在客户端做提示性拦截——校验必须在服务器侧强制执行，防止客户端被绕过后产生无法登录的账号（同ARC-005服务器权威原则）。

### 2.3 本功能日志设计

本节覆盖 **解绑前置校验（FR-IDN-005）** 的观察点——解绑校验涉及"剩余登录方式数"这一关键安全判定（防账号锁死），属账号身份域核心安全事件。拒绝路径（"剩余方式数=0"）属高优先级安全告警，必须 release 必出 + 100% 强制全采样（per NFR-SE-012 + §6.2）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.identity.unbind.allowed` | 解绑前置校验通过（剩余方式数 ≥ 1） | 偶发（玩家主动） | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`remaining_methods_count`；约 280B/条 |
| `auth.identity.unbind.rejected.last_method` | 解绑前置校验拒绝（剩余方式数 = 0，**FR-IDN-005 安全事件**） | 偶发 | release 必出（100% 强制全采样，per §6.2 + NFR-SE-012） | 含`account_id_hash`/`idp_type`/`current_methods_count`/`is_guest_capable`；约 350B/条 |
| `auth.identity.unbind.rejected.no_record` | 解绑请求时该账号未持有该 IdP 绑定（重复点击/伪造请求） | 偶发 | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`client_context`；约 300B/条 |
| `auth.identity.unbind.calculation.failed` | 剩余登录方式数计算异常（如 DB 计数失败、`is_guest_capable` 字段读取失败） | 极低 | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`error`/`trace_id`；约 350B/条 |
| `auth.identity.unbind.race_detected` | 解绑前置查询后、写入前的并发竞态（`count` 与 `write` 不在同事务）触发二次校验拒绝 | 极少 | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`concurrent_unbind_attempt_id`；约 400B/条 |
| `auth.identity.unbind.debug.full_link_state` | 解绑前 `AccountIdentityLink` 全表行 dump（用于复现"剩余方式数=0"边界场景） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-5KB/条（依赖绑定数量，release 剔除） |

**debug-only 守护要点**：
- `auth.identity.unbind.race_detected` 是 **AC-IDN-LOG-003 强制全采样白名单**（账号身份域安全/合规字段无遗漏）—— `error!` 级别，release 常驻
- `auth.identity.unbind.debug.full_link_state` 可能含玩家全部 IdP 绑定关系 —— release build 完全剔除，避免 RUST_LOG=debug 误开时通过单点日志泄漏账号拓扑



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

### 2.4 本功能日志设计

本节覆盖 **绑定/解绑审计留痕（FR-IDN-007）** 的观察点——`IdentityBindingAuditLog` 写入涉及"账号被盗申诉调查"等关键客诉场景，是客服工单系统的核心数据源。审计写入必须 100% 成功且不可丢失，因此**写入失败**属高优先级安全事件，release 必出 + 强制全采样。审计日志的访问行为本身（`IdentityBindingAuditLog` 查询）也需 release 必出，便于审计"谁查过什么账号的身份绑定历史"。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.identity.audit_log.written` | `IdentityBindingAuditLog` 写入成功（含 `bind`／`unbind`／`bind_rejected_conflict` 三类 action） | 稳态 1-10/s / 峰值 50/s | release 必出（100% 强制全采样，per §6.2 + NFR-SE-012） | 含`log_id`/`account_id_hash`/`action`/`idp_type`/`occurred_at`；约 300B/条 × 10/s = 3KB/s 稳态 |
| `auth.identity.audit_log.write_failed` | `IdentityBindingAuditLog` 写入失败（DB 故障 / 唯一键冲突） | 极低（生产事件） | release 必出（100% 强制全采样） | 含`account_id_hash`/`action`/`error`/`trace_id`；约 400B/条 |
| `auth.identity.audit_log.read` | 客服/合规角色查询某账号 `IdentityBindingAuditLog` 历史 | 偶发（客诉场景） | release 必出（100% 强制全采样） | 含`reader_id`/`account_id_hash`/`query_filter`/`result_count`；约 350B/条 |
| `auth.identity.audit_log.read_denied` | 客服/合规角色查询被独立权限域拒绝（无访问权） | 偶发（权限误用） | release 必出（100% 强制全采样） | 含`reader_id`/`attempted_account_id_hash`/`reason`；约 300B/条 |
| `auth.identity.client_context.device_fingerprint_changed` | 绑定/解绑时 `client_context` 中设备指纹与历史不一致 | 偶发 | release 必出（per BAS-004 v0.3 §5.1 末段掩码要求） | 含`account_id_hash`/`fingerprint_hash`（设备指纹哈希化）/`last_seen_at`；约 280B/条 |
| `auth.identity.client_context.ip_changed` | 绑定/解绑时 `client_context` 中 IP 与历史不一致 | 偶发 | release 必出（per BAS-004 v0.3 §5.1 IP 末段掩码） | 含`account_id_hash`/`ip_subnet`（`203.0.113.0/24` 末段掩码）/`last_seen_ip_subnet`；约 300B/条 |
| `auth.identity.audit_log.debug.full_payload` | 审计日志条目完整 dump（含 client_context 明细，**仅** debug-only 守护） | 极低 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-3KB/条（release 剔除） |

**debug-only 守护要点**：
- `auth.identity.audit_log.write_failed` 是 **AC-IDN-LOG-003 强制全采样白名单**（账号身份域安全/合规字段无遗漏）—— `error!` 级别，release 常驻，便于触发 P0 告警
- `auth.identity.audit_log.debug.full_payload` 可能含 `client_context` 中 IP 明文 —— release build 完全剔除，**严禁**出现在生产日志中
- `auth.identity.client_context.*` 系列包含 IP/设备指纹变更，**不**写入明文，按 BAS-004 v0.3 §5.1 末段掩码 / 哈希化处理



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

### 3.1 本功能日志设计

本节覆盖 **第三方身份绑定冲突（FR-IDN-006）** 的观察点——`idp_subject_id` 唯一性冲突是账号被盗/误绑定调查的关键信号，**必属**高优先级安全事件，release 必出 + 100% 强制全采样（per NFR-SE-012）。冲突事件同时触发 §2.4 `IdentityBindingAuditLog` 的 `bind_rejected_conflict` 写入（双重记录：业务日志 + 审计日志），便于不同查询场景分别检索。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.identity.bind.rejected.conflict` | 第三方身份绑定时 `(idp_type, idp_subject_id)` 已被既有账号占用（FR-IDN-006） | 偶发 | release 必出（100% 强制全采样，per §6.2 + NFR-SE-012） | 含`idp_type`/`existing_account_id_hash`/`attempting_account_id_hash`/`occurred_at`；约 350B/条 |
| `auth.identity.bind.rejected.conflict.spike` | 同一 `idp_subject_id` 在短窗口内（5 min）冲突尝试 ≥ N 次（**疑似撞库/盗号**） | 极少 | release 必出（100% 强制全采样，触发 P1 告警） | 含`idp_type`/`idp_subject_id_hash`/`attempt_count`/`window_seconds`；约 400B/条 |
| `auth.identity.bind.rejected.conflict.support_escalation` | 玩家就冲突事件发起客服工单（RGS-REQ-019） | 偶发 | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`ticket_id`；约 300B/条 |
| `auth.identity.bind.audit_log.bind_rejected_conflict.written` | §2.4 `IdentityBindingAuditLog` 因本次冲突写入 `bind_rejected_conflict` 条目成功 | 偶发 | release 必出（100% 强制全采样） | 含`log_id`/`account_id_hash`；约 250B/条 |
| `auth.identity.bind.conflict.debug.existing_link_state` | 冲突时既有 `AccountIdentityLink` 记录 dump（**仅** debug-only 守护） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除，避免 RUST_LOG=debug 误开时泄漏既有绑定拓扑） |

**debug-only 守护要点**：
- `auth.identity.bind.rejected.conflict.spike` 是 **AC-IDN-LOG-003 强制全采样白名单**（账号身份域安全/合规字段无遗漏）—— `warn!` 级别，release 常驻
- `auth.identity.bind.conflict.debug.existing_link_state` 可能反向暴露"哪些账号持有该第三方身份"——release build 完全剔除，**严禁**在生产日志中复现



## 3.2 IdP不可用时的降级（RSK-IDN-002落地）

```
IdPTokenVerifier向IdP发起验证请求超时/IdP返回5xx（区别于"令牌无效"的4xx明确拒绝）
  → 按既定重试策略重试有限次数（复用ARC-009标准消费者重试参数量级，避免无限重试拖长客户端等待）
  → 仍不可用 → 返回明确错误"IdP服务暂时不可用"（区别于"令牌无效"，避免误导玩家反复尝试无效令牌）
  → 客户端侧引导：若账号此前已具备游客登录能力（is_guest_capable=true）或已绑定其他仍可用的IdP，提示改用其他登录方式
  → 若账号仅绑定该不可用IdP且无游客能力，登录暂不可用，记录告警（复用RGS-BAS-003§6），供运维判断是否需要发布公告
```

### 3.2 本功能日志设计

本节覆盖 **第三方登录 IdP 验证与降级（RSK-IDN-002）** 的观察点——第三方登录是玩家入口路径（**NFR-SE-012 高安全**），登录请求/会话建立/失败路径全部 release 必出 + 100% 强制全采样，便于账号登录类工单的事后追溯。**严禁**记录 `*token*`/`*password*`/`*credential*`（per BAS-004 v0.3 §5.1 黑名单），登录请求 header/body 完整 dump 仅限 debug-only 守护。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.login.attempt.received` | 网关接收第三方登录请求（含 IdP 类型、客户端声明） | 稳态 100/s / 峰值 1000/s | release 必出（100% 强制全采样，per §6.2 + NFR-SE-012） | 含`account_id_hash`（未登录时为 null）/`idp_type`/`client_ip_subnet`（末段掩码）/`received_at`；约 250B/条 × 100/s = 25KB/s 稳态 |
| `auth.login.idp_verification.started` | `IdPTokenVerifier` 向 IdP 发起令牌验证请求 | 稳态 100/s / 峰值 1000/s | release 必出（100% 强制全采样） | 含`idp_type`/`verify_attempt_id`；约 200B/条 |
| `auth.login.idp_verification.success` | IdP 验证成功（200 OK + 有效签名） | 稳态 95/s / 峰值 950/s | release 必出（100% 强制全采样） | 含`idp_type`/`verify_latency_ms`/`idp_subject_id_hash`；约 280B/条 |
| `auth.login.idp_verification.failed.invalid` | IdP 验证返回 4xx（**令牌无效/伪造**） | 偶发 | release 必出（100% 强制全采样，per §6.2） | 含`idp_type`/`failure_reason`（4xx code 去标识化）/`attempt_count`；约 300B/条 |
| `auth.login.idp_verification.failed.unavailable` | IdP 验证超时/5xx（**IdP 服务不可用**，区别于令牌无效） | 偶发（IdP 故障期） | release 必出（100% 强制全采样，per §6.2） | 含`idp_type`/`failure_reason`/`verify_latency_ms`；约 300B/条 |
| `auth.login.idp_verification.retry.exhausted` | 有限重试后仍不可用（ARC-009 重试耗尽） | 偶发（IdP 故障期） | release 必出（100% 强制全采样，触发 P2 告警） | 含`idp_type`/`total_attempts`/`total_elapsed_ms`；约 350B/条 |
| `auth.login.guest_promote.success` | 游客转正成功（FR-IDN-003，不丢失既有进度） | 偶发 | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`preserved_progress_marker`；约 300B/条 |
| `auth.login.new_account.created` | 新账号创建（无既有绑定） | 偶发 | release 必出（100% 强制全采样） | 含`new_account_id_hash`/`idp_type`/`created_at`；约 250B/条 |
| `auth.login.session.established` | 会话建立成功（含 `session_epoch` 递增，ARC-005） | 稳态 95/s / 峰值 950/s | release 必出（100% 强制全采样，per §6.2 + NFR-SE-012） | 含`account_id_hash`/`session_epoch`/`scene_id`；约 280B/条 |
| `auth.login.fallback.guest_capable` | 客户端被引导改用游客登录（账号具备 `is_guest_capable=true`） | 偶发（IdP 故障期） | release 必出（100% 强制全采样） | 含`account_id_hash`/`idp_type`/`is_guest_capable`；约 280B/条 |
| `auth.login.fallback.alternative_idp` | 客户端被引导改用其他 IdP 登录（账号绑定其他可用 IdP） | 偶发（IdP 故障期） | release 必出（100% 强制全采样） | 含`account_id_hash`/`failed_idp_type`/`alternative_idp_type`；约 300B/条 |
| `auth.login.failed.no_alternative` | 登录暂不可用，账号仅绑定该不可用 IdP 且无游客能力（**登录入口 P0 事件**） | 极少 | release 必出（100% 强制全采样，触发 P1 告警，per §6.2） | 含`account_id_hash`/`idp_type`/`last_login_at`；约 350B/条 |
| `auth.login.session_refresh.completed` | 短期会话刷新成功（不重新走 IdP 验证） | 稳态 50/s | release 必出（100% 强制全采样） | 含`account_id_hash`/`old_session_age_seconds`/`new_session_age_seconds`；约 280B/条 |
| `auth.login.session_refresh.failed` | 短期会话刷新失败（强制重新登录或触发会话切断） | 偶发 | release 必出（100% 强制全采样） | 含`account_id_hash`/`failure_reason`；约 300B/条 |
| `auth.login.logout.completed` | 玩家主动登出（含客户端触发与服务端超时切断） | 稳态 50/s | release 必出（100% 强制全采样，per §6.2 + NFR-SE-012） | 含`account_id_hash`/`session_duration_seconds`/`logout_trigger`；约 300B/条 |
| `auth.login.debug.request_header_dump` | 登录请求 HTTP/QUIC header 完整 dump（**仅** debug-only 守护） | 0.1/s（仅 debug build） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除，避免 Authorization header 泄漏） |
| `auth.login.debug.request_body_dump` | 登录请求 body 完整 dump（**仅** debug-only 守护） | 0.1/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-3KB/条（release 剔除，**严禁** 包含 `idp_token`/`*password*`） |
| `auth.login.debug.idp_response_dump` | IdP 验证响应 body 完整 dump（**仅** debug-only 守护） | 0.1/s | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-2KB/条（release 剔除） |

**debug-only 守护要点**：
- `auth.login.failed.no_alternative` 是 **AC-IDN-LOG-003 强制全采样白名单**（账号身份域安全/合规字段无遗漏）—— `error!` 级别，release 常驻，便于 SRE 公告判定
- `auth.login.debug.request_body_dump` 务必配合 SDK 内置黑名单（`idp_token`/`authorization` 等）确认 release build 不会输出——release build 完全剔除是最后兜底
- `auth.login.*` 系列均不写明文 IP（`client_ip_subnet` 已末段掩码）/ 不写明文 `account_id`（全部 `_hash` 化），但 `idp_subject_id_hash` 与 `session_epoch` 仍属必要关联字段，**不**进入 BAS-004 v0.3 §5.1 黑名单



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

### 4.1 本功能日志设计

本节覆盖 **合规规则引擎配置驱动结构（FR-IDN-010）** 的观察点——`ComplianceRuleSet` 是**合规配置**基线，配置热更新涉及"哪个地区在何时切换到哪版规则"，属**合规审计必查项**，release 必出 + 100% 强制全采样。判定结果写入 `restriction_flags` 是未成年人保护链路的核心节点（与 §4.3 联动），因此判定路径（命中/未命中/降级）也属 release 必出。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.compliance.rule_set.reloaded` | `ComplianceRuleSet` 热更新完成（ARC-016 tick 边界原子切换点） | 偶发（合规政策变更） | release 必出（100% 强制全采样，per §6.2 合规要求） | 含`region`/`rule_set_version`/`previous_version`/`reloaded_at`；约 300B/条 |
| `auth.compliance.rule_set.reload_failed` | 热更新过程中配置拉取失败（合规配置不可用） | 极少（生产事件） | release 必出（100% 强制全采样，触发 P1 告警） | 含`region`/`error`/`trace_id`；约 350B/条 |
| `auth.compliance.judgment.cache.hit` | `ComplianceRuleEngine` 判定结果命中本地缓存 | 稳态 1000/s | release 必出（100% 强制全采样） | 含`region`/`rule_version`/`hit`；约 200B/条 × 1000/s = 200KB/s 稳态 |
| `auth.compliance.judgment.cache.miss` | 判定缓存未命中，回源至 PostgreSQL 查询 `ComplianceRuleSet` | 偶发 | release 必出（100% 强制全采样） | 含`region`/`miss_count`/`query_latency_ms`；约 250B/条 |
| `auth.compliance.judgment.completed` | 合规判定完成（含判定结果类型/命中规则版本） | 稳态 100/s / 峰值 1000/s | release 必出（100% 强制全采样，per §6.2） | 含`account_id_hash`/`region`/`judged_rules_count`/`restriction_flags`（位标志）/`rule_set_version`；约 350B/条 × 100/s = 35KB/s 稳态 |
| `auth.compliance.judgment.fallback.using_default` | 玩家地区无对应 `ComplianceRuleSet`，降级使用默认配置 | 偶发（未配置地区） | release 必出（100% 强制全采样） | 含`account_id_hash`/`region`/`default_rule_set_version`；约 300B/条 |
| `auth.compliance.judgment.business_service_bypass_attempt` | 业务服务绕过 `ComplianceRuleEngine` 直接判定（如自行实现时长判定） | 极少（CI 拦截） | release 必出（100% 强制全采样，触发 P1 告警） | 含`service_name`/`attempted_judgment_kind`/`caller_trace_id`；约 350B/条 |
| `auth.compliance.debug.rule_set.full_dump` | `ComplianceRuleSet` 完整 dump（用于合规审计复盘） | 偶发（季度合规评审） | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 2-5KB/条（release 剔除） |
| `auth.compliance.debug.judgment_input_dump` | 判定输入参数 dump（含 `ComplianceProfile` 字段快照） | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除） |

**debug-only 守护要点**：
- `auth.compliance.judgment.business_service_bypass_attempt` 是 **AC-IDN-LOG-003 强制全采样白名单**（账号身份域安全/合规字段无遗漏）—— `error!` 级别，release 常驻，触发 P1 告警提示架构师介入
- `auth.compliance.judgment.completed` 稳态 100/s + 峰值 1000/s 频率较高，但因属合规链路核心节点，仍 release 必出（不可降采样，否则合规审计失效）



## 4.2 实名认证信息的独立访问权限（FR-IDN-013落地）

`ComplianceProfile`中涉及个人信息的字段（若`verification_method`要求存储身份证号等原始凭证，该原始凭证**不**在`ComplianceProfile`本表明文存储，而是加密存储于独立的`IdentityVerificationVault`，`ComplianceProfile`仅保留`verification_status`/`age_bracket`/`restriction_flags`等派生判定结果）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `account_id` | 玩家账号ID | 主键 |
| `encrypted_payload` | 加密二进制 | 原始认证凭证，加密复用既有NFR-SE-012个人信息保护标准（密钥管理复用RGS-REQ-010既有Secrets管理范围） |
| `access_log_ref` | 关联访问审计记录 | 每次解密访问须先落审计记录（复用RGS-BAS-003§7） |

访问权限：`IdentityVerificationVault`的解密访问权限**独立**评审分配，**不**与日常运营查询权限（如GM后台的一般客服角色）复用同一角色定义，仅限专设的合规/法务角色访问，访问前须记录申请理由（同RGS-BAS-017§3.5"分析管线独立访问权限"同类治理思想的应用）。

### 4.2 本功能日志设计

本节覆盖 **实名认证 `IdentityVerificationVault` 访问（FR-IDN-013）** 的观察点——Vault 解密访问是**最高敏感度**操作（涉及身份证号/人脸识别等原始认证凭证），全部 access attempt（成功/拒绝/异常）必须 release 必出 + 100% 强制全采样，满足合规审计的"谁在何时访问了谁的实名信息"完整留痕。Vault 加密 payload **严禁**进入 release 必出字段，**仅** debug-only 守护（且 release build 完全剔除）。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.compliance.vault.access.requested` | 合规/法务角色发起 Vault 解密访问（含访问理由） | 偶发（合规审查场景） | release 必出（100% 强制全采样，per §6.2 + 合规要求） | 含`reader_id`/`requested_account_id_hash`/`access_reason`（必填，FR-IDN-013）/`requested_at`；约 400B/条 |
| `auth.compliance.vault.access.granted` | 访问授权通过（角色权限 + 理由合规） | 偶发 | release 必出（100% 强制全采样） | 含`reader_id`/`accessed_account_id_hash`/`access_reason`/`granted_at`；约 400B/条 |
| `auth.compliance.vault.access.denied` | 访问被拒（角色不匹配 / 理由缺失 / 频率超限） | 偶发（权限误用） | release 必出（100% 强制全采样，触发 P1 告警） | 含`reader_id`/`attempted_account_id_hash`/`denial_reason`/`denied_at`；约 400B/条 |
| `auth.compliance.vault.access.audit_log.written` | §4.2 `IdentityVerificationVault.access_log_ref` 关联审计记录写入成功 | 偶发 | release 必出（100% 强制全采样） | 含`audit_log_id`/`reader_id`/`accessed_account_id_hash`；约 300B/条 |
| `auth.compliance.vault.access.frequency_exceeded` | 同一 reader_id 在短窗口内（1h）访问次数超限（**疑似滥用**） | 极少 | release 必出（100% 强制全采样，触发 P1 告警） | 含`reader_id`/`window_attempt_count`/`threshold`；约 350B/条 |
| `auth.compliance.vault.decrypt.failed` | 解密失败（密钥轮换期 / payload 损坏） | 极少 | release 必出（100% 强制全采样，触发 P1 告警） | 含`accessed_account_id_hash`/`error`/`trace_id`；约 400B/条 |
| `auth.compliance.vault.key_rotation.started` | Vault 加密密钥轮换开始 | 极低（季度合规运维） | release 必出（100% 强制全采样） | 含`old_key_version`/`new_key_version`/`started_at`；约 300B/条 |
| `auth.compliance.vault.key_rotation.completed` | 密钥轮换完成（全部 payload 已用新密钥重加密） | 极低 | release 必出（100% 强制全采样） | 含`old_key_version`/`new_key_version`/`re_encrypted_count`/`duration_seconds`；约 350B/条 |
| `auth.compliance.debug.vault.encrypted_payload_dump` | Vault 加密 payload dump（**仅** debug-only 守护，**严禁** release 出现） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 1-10KB/条（release 剔除，避免 RUST_LOG=debug 误开时原始认证凭证泄漏） |
| `auth.compliance.debug.vault.access_justification_dump` | 访问申请理由的完整 payload（含附件） | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-2KB/条（release 剔除） |

**debug-only 守护要点**：
- `auth.compliance.vault.*` 系列全部 release 必出（除 debug-only 外）—— **AC-IDN-LOG-003 强制全采样白名单**核心条目（合规最高优先级）
- `auth.compliance.vault.access.denied` 与 `auth.compliance.vault.access.frequency_exceeded` 是**安全/合规关键告警**—— `error!` 级别，release 常驻 + P1 告警链路
- `auth.compliance.debug.vault.encrypted_payload_dump` 是**账号身份域**最敏感的 debug-only 字段—— release build 完全剔除是**最后兜底**，CI 静态扫描（BAS-004 v0.3 §9 第 3 项）必须检测其 release 必出版本不存在



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

### 4.3 本功能日志设计

本节覆盖 **未成年人保护限制触发/解除留痕（FR-IDN-014）** 的观察点——`MinorRestrictionAuditLog` 写入涉及合规审计与防沉迷监管，**全部**触发/解除事件 release 必出 + 100% 强制全采样。`restriction_triggered` 与 `restriction_lifted` 是合规复盘的关键事件，**不得**降采样。`restriction_flags` 变更前的合规判定（命中哪条 `ComplianceRuleSet` 规则）也属 release 必出，便于复盘"为什么这个账号被限制/解除"。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `auth.compliance.minor.restriction.triggered` | `ComplianceRuleEngine` 触发某项未成年人限制（`playtime_limit` / `payment_disabled` / 等），**FR-IDN-014 核心事件** | 稳态 10/s / 峰值 100/s | release 必出（100% 强制全采样，per §6.2 + FR-IDN-014） | 含`account_id_hash`/`restriction_type`/`trigger_reason`/`rule_set_version`/`audit_log_id`；约 350B/条 × 10/s = 3.5KB/s 稳态 |
| `auth.compliance.minor.restriction.lifted` | 限制解除（周期重置 / 实名认证更新 / 玩家达到成年） | 稳态 5/s / 峰值 50/s | release 必出（100% 强制全采样，per §6.2 + FR-IDN-014） | 含`account_id_hash`/`restriction_type`/`lift_reason`/`rule_set_version`/`audit_log_id`；约 350B/条 |
| `auth.compliance.minor.restriction.playtime_limit_hit` | 玩家当日游玩时长达到 `minor_playtime_limit_minutes` 上限 | 稳态 1/s / 峰值 10/s | release 必出（100% 强制全采样） | 含`account_id_hash`/`daily_playtime_minutes`/`limit_minutes`；约 300B/条 |
| `auth.compliance.minor.restriction.cycle_reset` | 每日/每周/每月周期重置（如 0 点重置 `playtime_limit`） | 1-2/min | release 必出（100% 强制全采样） | 含`region`/`cycle_kind`/`reset_at`/`affected_account_count`；约 280B/条 |
| `auth.compliance.minor.restriction.age_bracket_changed` | 实名认证更新导致 `age_bracket` 变化（解除/触发新的限制集合） | 偶发 | release 必出（100% 强制全采样） | 含`account_id_hash`/`old_age_bracket`/`new_age_bracket`/`recomputed_restrictions`；约 400B/条 |
| `auth.compliance.minor.audit_log.written` | §4.3 `MinorRestrictionAuditLog` 写入成功 | 与 `restriction_triggered`/`restriction_lifted` 同步 | release 必出（100% 强制全采样） | 含`audit_log_id`/`account_id_hash`/`action`/`restriction_type`；约 300B/条 |
| `auth.compliance.minor.audit_log.read` | 合规/法务角色查询 `MinorRestrictionAuditLog` | 偶发（合规审查） | release 必出（100% 强制全采样） | 含`reader_id`/`query_filter`/`result_count`；约 350B/条 |
| `auth.compliance.minor.audit_log.read_denied` | 查询被独立权限域拒绝 | 偶发（权限误用） | release 必出（100% 强制全采样，触发 P2 告警） | 含`reader_id`/`attempted_filter`/`denial_reason`；约 300B/条 |
| `auth.compliance.minor.business_service_query_skip` | 业务服务（付费/游玩统计）查询时未走 `ComplianceRuleEngine`（**疑似绕过合规判定**） | 极少（CI 拦截） | release 必出（100% 强制全采样，触发 P1 告警） | 含`service_name`/`attempted_action`/`caller_trace_id`；约 350B/条 |
| `auth.compliance.minor.debug.restriction_flags_full_diff` | `restriction_flags` 变更前后全量 diff dump | 偶发 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 100-300B/条（release 剔除） |
| `auth.compliance.minor.debug.audit_log_full_payload` | 审计日志条目完整 payload dump | 极少 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除） | 约 500B-1KB/条（release 剔除） |

**debug-only 守护要点**：
- `auth.compliance.minor.restriction.triggered` / `auth.compliance.minor.restriction.lifted` 是 **AC-IDN-LOG-003 强制全采样白名单**核心条目（合规最高优先级）—— `info!` 级别（业务事件而非异常），release 常驻
- `auth.compliance.minor.business_service_query_skip` 是 **FR-IDN-014 合规绕过告警**—— `error!` 级别，release 常驻 + P1 告警，提示架构师介入调查
- 全部 `auth.compliance.minor.*` 字段均**不**包含玩家明文标识（统一 `_hash`），与 §4.2 Vault 访问的合规审计域共同形成"防沉迷+实名"完整留痕链



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
- [ ] **每功能章节（§2.1/§2.2/§2.3/§2.4/§3.1/§3.2/§4.1/§4.2/§4.3）均含"本功能日志设计"子节**，且明确区分 `info!`/`warn!`/`error!`（release 必出）与 `debug!`/`trace!`（`#[cfg(debug_assertions)]` 守护，debug-only）两类
- [ ] release 必出事件清单（§2.1/§2.2/§2.3/§2.4/§3.1/§3.2/§4.1/§4.2/§4.3 共 9 节）逐项可在本功能代码中检索到对应调用点（grep 验证），未遗漏账号身份域安全/合规关键事件（per AC-IDN-LOG-003 强制全采样白名单：`auth.login.*` / `auth.identity.bind.rejected.conflict` / `auth.identity.unbind.rejected.last_method` / `auth.identity.audit_log.*` / `auth.compliance.real_name.*` / `auth.compliance.vault.*` / `auth.compliance.minor.*`）
- [ ] debug-only 事件均带 `#[cfg(debug_assertions)]` 守护，release build 完全剔除（per BAS-004 v0.3 §4.4 + AC-IDN-LOG-001），重点守护 `auth.login.debug.*` / `auth.compliance.debug.vault.*` / `auth.compliance.debug.rule_set.*` / `auth.identity.audit_log.debug.*` / `auth.identity.unbind.debug.*`
- [ ] release 必出宏（`info!`/`warn!`/`error!`）未被 `#[cfg]` 守护（per BAS-004 v0.3 §4.5 + AC-IDN-LOG-002）
- [ ] 字段名沿用 BAS-004 v0.3 §4.3.1/§4.3.2 snake_case + `auth.*` 命名空间，未使用 `playerId` 等变体（FR-LOG-013）
- [ ] **脱敏字段**（`*token*`/`*password*`/`*credential*`/`*authorization*`）**禁止**出现在 release 必出字段中（per BAS-004 v0.3 §5.1 黑名单）；**账号身份域特殊确认**：`idp_token` / `idp_access_token` / `idp_refresh_token` / `password` / `credential_hash` 等所有可能落点已逐一 grep 验证；`auth.identity.client_context.ip_changed` 末段掩码为 `203.0.113.0/24`（per BAS-004 v0.3 §5.1 末段掩码）

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
| **AC-IDN-LOG-001（debug-only 宏在 release build 完全剔除）** | §2.1/§2.2/§2.3/§2.4/§3.1/§3.2/§4.1/§4.2/§4.3 各节"debug-only 守护要点"项 + RGS-BAS-004 v0.3 §4.4 四铁律 + §9 CI 第 5/6 项静态检查 | §2-§4 各节本功能日志设计 |
| **AC-IDN-LOG-002（每功能 BAS 文档须含本功能 log 设计章节）** | §2.1/§2.2/§2.3/§2.4/§3.1/§3.2/§4.1/§4.2/§4.3 各"本功能日志设计"小节 + §5.1 检查项（每功能 log 章节存在性 + release 必出 grep 验证 + debug-only 四铁律合规 + release 必出宏未被 `#[cfg]` 守护 + 字段名 snake_case + `auth.*` 命名空间 + 脱敏字段不入 release） | §2-§4 各节本功能日志设计 |
| **AC-IDN-LOG-003（账号身份域安全/合规字段强制全采样无遗漏）** | §2.1-§4.3 各节 release 必出事件清单 + §3.2/§4.2/§4.3 中 `auth.login.failed.no_alternative` / `auth.compliance.vault.*` / `auth.compliance.minor.restriction.*` 等强制全采样白名单（per BAS-004 v0.3 §6.2 + NFR-SE-012 + FR-IDN-013/014） | §3.2、§4.2、§4.3 + 各节强制全采样白名单 |

---

> 本文档与RGS-REQ-021（账号身份、第三方登录与合规 需求定义书）配套使用。
