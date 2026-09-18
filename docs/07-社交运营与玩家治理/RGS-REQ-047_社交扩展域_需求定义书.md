# 需求定义书（要件定義書 / Requirements Definition Document）

**社交扩展域（Social-Extra Domain）需求定义**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REQ-047 |
| 版本 | 0.1 (草案) |
| 父文档 | RGS-REQ-001 v1.5 §4 业务需求 + §5 功能需求；RGS-REQ-013 v1.4 横切关注点 |
| 制定日 | 2026-09-11 |
| 制定者 | 架构师 (worktree wt-b, ULYS-1 子任务) |
| 关联下游 | RGS-BAS-047 / RGS-DTL-054 / social_extra/v1/social_extra.proto (1 service, 15 RPC) |
| 状态 | 草案, 待评审 |
| 关联 crate | `crates/social-extra-service/` (social_extra_db, 540 LOC) |
| ARC | ARC-055（社交扩展 + 复用 social-service 既有） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响范围 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-11 | 架构师 (wt-b agent) | — | 初版制定（per ULYS-1 9/4 MD §2 审计识别出 social-extra-service 为代码先行扩展域，crate 已实装 540 LOC、1 service、15 RPC，但缺正式 REQ/BAS/DTL 三层文档）。本文档为此扩展域补全的需求层 | 全部 |

> 本文档以代码先行（code-first）方式补全需求定义：对 `crates/social-extra-service/` 既已实装的 1 个 gRPC service、15 个 RPC 进行反向需求抽象。

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 (wt-b agent) | 2026-09-11 | — |
| 评审（技术） | | | 社交扩展域与 social-service / DTL-013 既有的职责边界，避免重复建设 |
| 评审（DBA） | | | social_extra_db 物理划分 |
| 审批（负责人） | | | 本文档的基准化 |

## 目录

1. 背景与范围
2. 术语约定
3. 业务需求
4. 功能需求
5. 非功能需求
6. 架构设计方针 ARC-055
7. 验收标准
8. 风险与未决事项
9. 关联文档

---

# 1. 背景与范围

## 1.1 背景

`crates/social-extra-service/` 是 RGS 7 域扩展下独立拆分的社交扩展微服务。其设计**承接** social-service 与 RGS-DTL-013 既有的社交核心能力，但承担**社交扩展**功能：邮件 / 好友 / 家园 / 聊天扩展 / 主页扩展等。

本域覆盖 1 个 gRPC service（SocialExtraService）、15 个 RPC（含 1 HealthCheck）。其中 12 RPC 为真实业务逻辑（邮件 / 好友 / 家园 / 聊天扩展），3 RPC 为 Unimplemented stub。

## 1.2 范围边界

**In-Scope**：
- 邮件系统（站内邮件，**不**含移动推送，per RGS-REQ-022 区分）
- 好友扩展（好友互动：点赞 / 评论 / 备注）
- 家园系统（家园 CRUD + 访问）
- 聊天扩展（聊天历史 / 表情包）

**Out-of-Scope**：
- 频道聊天核心（属 RGS-DTL-013 既有）
- 移动推送（属 RGS-REQ-022 既有）
- 好友关系核心（属 social-service 既有）
- 私聊路由核心（属 RGS-DTL-013 §3 既有）

## 1.3 与既有基础设施的关系

| 既有机制 | 本域使用方式 |
|---|---|
| social-service 既有 | 好友关系**严格**复用既有 |
| RGS-DTL-013 大厅社交通信 | 频道聊天核心**严格**复用既有 |
| ARC-008 5 独立 DB → 7 域扩展 | 落位独立 `social_extra_db`（仅邮件/好友扩展/家园/聊天扩展） |
| RGS-REQ-013 FR-GOV-001 | 邮件附件经 EC 单点 |
| RGS-BAS-014 §3 ChatAbuseGuard | 聊天扩展**严格**复用既有 |

---

# 2. 术语约定

| 术语 | 英文 | 含义 |
|---|---|
| 站内邮件 | InGameMail | 游戏内邮件（**不**含移动推送） |
| 好友扩展 | FriendExtra | 好友互动（点赞 / 评论 / 备注） |
| 家园 | Home | 玩家个人空间（CRUD + 访问） |
| 聊天扩展 | ChatExtra | 聊天历史 + 表情包 |

---

# 3. 业务需求 (BR)

## BR-SOC-001 站内邮件

- 玩家发送 / 接收 / 读取 / 领取附件 / 删除邮件
- 邮件按类型：系统邮件 / 玩家邮件 / GM 邮件
- 附件领取经 EC 单点

**验收**：邮件 CRUD 完整 + 附件经 EC。

## BR-SOC-002 好友扩展

- 好友互动：点赞 / 评论 / 备注
- 好友主页扩展（per 玩家档案既有）

**验收**：好友互动可 CRUD。

## BR-SOC-003 家园系统

- 家园 CRUD（主题 / 装饰 / 访问次数）
- 家园访问（其他玩家访问 + 留言）

**验收**：家园可 CRUD + 访问可记录。

## BR-SOC-004 聊天扩展

- 聊天历史查询（per 玩家）
- 表情包（per ChatEmojiConfig）

**验收**：聊天历史可查询 + 表情包可使用。

---

# 4. 功能需求 (FR)

## 4.1 站内邮件

| ID | 需求 |
|---|---|
| FR-SOC-001 | `SendMail` **必须**支持 3 类（系统 / 玩家 / GM） |
| FR-SOC-002 | `ClaimMailAttachment` **必须**经 EC 单点（per FR-GOV-001） |
| FR-SOC-003 | 邮件保留期 **应当**有上限（per MailConfig.retention_days，默认 30 天） |

## 4.2 好友扩展

| ID | 需求 |
|---|---|
| FR-SOC-010 | `LikeFriend` **必须**走好友关系既有（per social-service） |
| FR-SOC-011 | `CommentFriend` **应当**限频（per CommentConfig.max_per_day） |
| FR-SOC-012 | 好友主页**应当**复用 player-service 既有档案 |

## 4.3 家园系统

| ID | 需求 |
|---|---|
| FR-SOC-020 | `GetHome` **必须**返回家园状态（主题 / 装饰 / 访问次数） |
| FR-SOC-021 | `DecorateHome` **必须**支持主题 + 装饰 slot |
| FR-SOC-022 | `VisitHome` **必须**记录访问次数 + 留言 |

## 4.4 聊天扩展

| ID | 需求 |
|---|---|
| FR-SOC-030 | `GetChatHistory` **必须**走 RGS-DTL-013 §3 既有聊天核心 |
| FR-SOC-031 | 表情包**应当**作为 ARC-021 插件承载 |

---

# 5. 非功能需求 (NFR)

| ID | 类别 | 内容 | 目标值 / 判定基准 |
|---|---|---|---|
| NFR-SOC-001 | 性能 | 邮件查询 P99 延迟 | < 100ms（per NFR-OP-005 既有） |
| NFR-SOC-002 | 一致性 | 邮件附件领取**必须**与邮件状态变更在同一事务 | per RGS-DTL-001 §3.2 |
| NFR-SOC-003 | 安全 | 聊天表情包**必须**经 ChatAbuseGuard 校验 | per RGS-BAS-014 §3 既有 |
| NFR-SOC-004 | 隐私 | 好友主页访问**必须**遵守隐私过滤 | per NFR-SE-005 |

---

# 6. 架构设计方针 ARC-055

## 6.1 ARC-055：社交扩展 + 复用 social-service / DTL-013 既有原则

| 项目 | 内容 |
|---|---|
| **决定** | social-extra-service 仅承担"邮件 / 好友扩展 / 家园 / 聊天扩展"4 类扩展功能；好友关系 / 频道聊天 / 频道路由**严格**复用 social-service + RGS-DTL-013 既有 |
| **理由** | 避免与既有 social-service / DTL-013 重复建设，聚焦"扩展 / 衍生"功能 |
| **承载机制** | 邮件规则 / 好友扩展规则 / 家园主题 / 表情包作为 ARC-021 既有插件通道承载 |
| **否决方案** | "在 social-extra 内自建好友关系核心 / 自建频道聊天核心"——直接违反既有职责，已否决 |
| **适用范围** | 本域所有社交扩展功能 |

---

# 7. 验收标准

| ID | 验收标准 |
|---|---|
| AC-SOC-001 | 邮件 CRUD 完整 + 附件经 EC 单点 |
| AC-SOC-002 | 好友扩展走既有 social-service，**不**自建好友关系 |
| AC-SOC-003 | 家园可 CRUD + 访问可记录 |
| AC-SOC-004 | 聊天扩展走既有 RGS-DTL-013，**不**自建聊天核心 |

---

# 8. 风险与未决事项

| ID | 内容 | 处理阶段 |
|---|---|---|
| TBD-SOC-001 | 邮件保留期（MailConfig.retention_days，需 PH-2 评审） | PH-2 |
| RSK-SOC-001 | 邮件附件领取与邮件状态变更的并发场景须 DTL-054 详细设计阶段重点验证 | DTL-054 §3 |

---

# 9. 关联文档

| 文档编号 | 文档名 | 与本文档的关系 |
|---|---|---|
| RGS-REQ-001 | 需求定义书 | 本文档展开其 §4 / §5 |
| RGS-REQ-013 | 体系治理与横切关注点 | 邮件附件经 EC（FR-GOV-001） |
| RGS-REQ-016 | 大厅、社交通信与运营活动 | 频道路由核心既有 |
| RGS-REQ-022 | 消息推送与兑换码运营工具 | 移动推送区分 |
| RGS-DTL-013 | 大厅社交通信 详细设计 | 频道路由核心既有 |
| social-service 既有 | 好友关系核心 | 本域严格复用 |
| RGS-BAS-014 | 排行榜任务成就 | ChatAbuseGuard 既有 |
| RGS-BAS-047 | 社交扩展域 基本设计书（本文档的下游） | 本文档需求展开 |
| RGS-DTL-054 | 社交扩展域 详细设计书（本文档的下游） | 本文档物理 / 实现级设计 |

> **文档编号说明**：本文档为 RGS-REQ-047，配套基本设计书 RGS-BAS-047，详细设计书 RGS-DTL-054。
