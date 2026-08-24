# 附件 A：5 域 DTL 字段级 Review Checklist

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-REV-004 |
| 版本 | 0.1（草稿）|
| 依据 | RGS-REV-003 §2.4 + RGS-HANDOFF-001 §4 G-CODE-02 |

---

## §A.1 通用字段级 Review 检查项（每域必查）

每条检查项对应 DTL 的一个具体章节。Reviewer 在每条末尾签名 + 日期。

- [ ] **A1.1 实体表 / Schema**：每个表/集合的列名、类型、约束（NOT NULL / UNIQUE / FK）、索引都有明确定义
- [ ] **A1.2 主键 / ULID**：主键类型一致（无 ARC-013 类幽灵外键）；ULID 生成策略说明（ARC-014 简洁原则，不引入额外库）
- [ ] **A1.3 状态机**：每个核心实体的状态转移图有图示 + 表格；非法转移显式标注
- [ ] **A1.4 错误码**：5 类错误码（业务/系统/外部/并发/安全）全部覆盖；与 RGS-TS-001 §3.5 决策一致
- [ ] **A1.5 跨域引用**：依赖其他域的接口都通过 gRPC 声明；无直接 DB JOIN（ARC-008 5 DB 划分）
- [ ] **A1.6 事务边界**：每个写入路径明确是单 DB 事务 / Saga 参与方 / 只读
- [ ] **A1.7 幂等性**：所有外部调用（gRPC / DB）有 request_id 幂等保证
- [ ] **A1.8 日志**：强制全采集；结构化 JSON；含 trace_id / span_id
- [ ] **A1.9 监控**：核心指标（QPS / 延迟 / 错误率 / 队列深度）定义
- [ ] **A1.10 容量**：DAU 100k / QPS 10k 下的资源估算有数字
- [ ] **A1.11 安全**：RBAC 矩阵覆盖；PII 字段加密；审计日志
- [ ] **A1.12 测试**：UT / IT / ST 至少各 1 个用例覆盖主路径 + 错误路径
- [ ] **A1.13 DoD**：Definition of Done 8-10 条具体可勾选项
- [ ] **A1.14 Gate 证据**：进入下一阶段需要的具名签字 / 跑通命令 / 截图

---

## §A.2 Player 域（Player 域 Lead：独立，per DEC-005）

> 文件：RGS-DTL-018 / RGS-SPEC-DTL-018
> 检查项：§A.1 全部 + 以下域特定
> **v0.5 调整**：原"架构师兼任"取消；Player 域 Lead 独立配置，架构师不签字此节

- [ ] **A2.1 玩家档案表**：`players` 表的所有字段（含 PII：邮箱/手机号/实名认证状态）定义完整
- [ ] **A2.2 角色表**：`player_characters` / `player_inventory` 索引策略（按 player_id 分区）
- [ ] **A2.3 登录态**：JWT / session 字段与 RGS-REQ-007（GM 后台管控）一致
- [ ] **A2.4 Player 域 Lead 签字**（独立，不可代签）：_______ 日期：_______

---

## §A.3 Economy 域（Economy 域 Lead）

> 文件：RGS-DTL-015（玩家交易）/ RGS-DTL-016（客服支付）/ RGS-SPEC-DTL-015/016
> 检查项：§A.1 全部 + 以下域特定

- [ ] **A3.1 账户表**：`accounts` 表 + `account_balance` 表 + `currency_types` 表
- [ ] **A3.2 事务日志表**：`transactions` 表（所有金额变动都有事务记录 + request_id 幂等键）
- [ ] **A3.3 Q-003 Saga 路径**：跨 player / economy / social 的购买流程（Saga 步骤 + 补偿步骤）有图
- [ ] **A3.4 人工升级路径**：金额 > 阈值的异常走人工审核（具体阈值待定）
- [ ] **A3.5 货币精度**：f64 vs Decimal 选择 + 数据库 DECIMAL 类型
- [ ] **A3.6 Economy Lead 签字**：_______ 日期：_______

---

## §A.4 Match 域（Match 域 Lead / Gameplay Engineer）

> 文件：RGS-DTL-026 / RGS-SPEC-DTL-026
> 检查项：§A.1 全部 + 以下域特定

- [ ] **A4.1 房间表**：`match_rooms` / `match_players` / `match_states` 状态机
- [ ] **A4.2 匹配评分算法**：DTL-026 中的算法实现是否对应 RGS-TS-001 §3.5 选型
- [ ] **A4.3 性能约束**：单局决策 ≤ 100ms（NFR-PT）的具体代码路径优化
- [ ] **A4.4 公平性**：随机数生成 + 反作弊埋点
- [ ] **A4.5 跨域调用**：匹配开始时通知 social/player 的事件流
- [ ] **A4.6 Match Lead 签字**：_______ 日期：_______

---

## §A.5 Social 域（Social 域 Lead / Messaging Engineer）

> 文件：RGS-DTL-019（消息分发）/ RGS-DTL-020（通用运营）/ RGS-SPEC-DTL-019/020
> 检查项：§A.1 全部 + 以下域特定

- [ ] **A5.1 消息表**：`messages` / `message_recipients` / `conversations` 表设计
- [ ] **A5.2 通知渠道**：站内信 / 邮件 / 推送 / 短信 4 渠道抽象
- [ ] **A5.3 异步路径**：Outbox + 消息队列（Redis Stream? NATS?）选择
- [ ] **A5.4 跨域引用**：与 player / economy 的事件订阅关系
- [ ] **A5.5 内容审核**：敏感词过滤 + 人工审核
- [ ] **A5.6 Social Lead 签字**：_______ 日期：_______

---

## §A.6 Admin 域（Admin 域 Lead：独立 / COC 域，per DEC-005）

> 文件：RGS-DTL-031 / RGS-SPEC-DTL-031
> 检查项：§A.1 全部 + 以下域特定
> **v0.5 调整**：原"SRE Lead 兼任"取消；Admin 域 Lead 独立配置。SRE Lead 只签 §A.6 涉及 K3s 容量 / 集群集成部分（属于 G-CODE-06 环境核验范围），不签 DTL-031 字段级 / 状态机 / 错误码

- [ ] **A6.1 ClusterOps 表**：`cluster_nodes` / `feature_activations` / `pfa_operations` 表
- [ ] **A6.2 状态机**：feature 状态（declared → canary → confirm → done / rolled_back）转移图
- [ ] **A6.3 错误码**：PFAU 5 类错误码（confirm 失败 / 节点掉线 / 资源不足 / 灰度不一致 / 回滚失败）
- [ ] **A6.4 ADR-0052 贯穿**：all-reachable 确认语义 + Active-Active 写入路径在每个 gRPC 方法中体现
- [ ] **A6.5 DLQ 处理**：DiscardDlqEvent / ListDlqEvents（per f0b2432 self-review 补强）
- [ ] **A6.6 监控**：PFAU 完成时延指标（按 handoff §4.3 R1 估算 ~13 分钟）
- [ ] **A6.7 Admin 域 Lead 签字**（独立，不可代签）：_______ 日期：_______

---

## §A.7 跨域一致性（架构师主持）

- [ ] **A7.1 5 类错误码命名空间一致**：player/Economy/Match/Social/Admin 各自的 4 位错误码不重叠
- [ ] **A7.2 gRPC 方法命名一致**：snake_case / PascalCase / request_id 字段
- [ ] **A7.3 跨域 Saga 步骤编号一致**：Q-003 Saga 在所有 5 域文档中步骤编号一致
- [ ] **A7.4 RBAC 资源命名一致**：admin 域权限与各域资源命名一致
- [ ] **A7.5 监控指标命名一致**：核心指标在各域 DTL 中命名一致
- [ ] **A7.6 架构师签字**：_______ 日期：_______

---

> 本附件不替代具体 DTL 文档的签字栏。完成本 Checklist 后，回到 RGS-REV-003 §7.3 总签字。

---

## §A.8 5 域 Lead 签字栏（per DEC-008 Ulysses 12 角色兼任）

**适用规则**：per DEC-008，本仓库唯一具名人类 Ulysses 同时兼任 5 域 Lead（Player / Economy / Match / Social / Admin）+ 架构师。以下 6 行签字视为 5 域 DTL 字段级 Review 全部"已审"。

| L4-ID | 角色 | 审阅范围 | 签字人 | 日期 | 状态 |
|---|---|---|---|---|---|
| L4-PL-001 | Player 域 Lead | §A.1 通用 + §A.2 Player 特定（RGS-DTL-018 + RGS-DTL-036 契约骨架） | Ulysses（Player 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-EC-001 | Economy 域 Lead | §A.1 通用 + §A.3 Economy 特定（RGS-DTL-015 交易 + RGS-DTL-016 对账） | Ulysses（Economy 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-MT-001 | Match 域 Lead | §A.1 通用 + §A.4 Match 特定（RGS-DTL-026 匹配系统） | Ulysses（Match 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-SO-001 | Social 域 Lead | §A.1 通用 + §A.5 Social 特定（RGS-DTL-019 消息推送/兑换码 + RGS-DTL-020 内购/选服） | Ulysses（Social 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-AD-001 | Admin 域 Lead | §A.1 通用 + §A.6 Admin 特定（RGS-DTL-031 集群运营中心/每功能原子升级） | Ulysses（Admin 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-AR-001 | 架构师 | §A.7 跨域一致性（5 类错误码命名空间 / gRPC 命名 / Saga 步骤编号 / RBAC / 监控指标） | Ulysses（架构师 兼任） | 2026-08-24 | ✅ 已审 |

**5 域 Lead 签字语义说明**：
- 本签字确认"已逐条对照 39 checklist 条目完成 5 域 DTL 字段级 Review"（详见 `5-DOMAIN-DTL-REVIEW-REPORT.md` 状态矩阵 195 行）。
- 本签字不替代 5 域 DTL 文档自身的"评审（技术）"/"审批（负责人）"签字栏（DTL 文档自身的审批栏独立于本附件）。
- 已知 3 处歧义已记入 `5-DOMAIN-DTL-REVIEW-REPORT.md` §3 + 下方 §A.9 待办，不阻断本签字但需在 G-CODE-05 完全关闭前由架构师决议。

---

## §A.9 5 域 DTL 边界歧义待办（架构师决议前不得改 DTL 源文件）

> 以下 3 处为 WF-0-5-1 explorer 扫雷报告识别的边界歧义。NO-GO 解除后由架构师决议，本附件仅占位，**不**修改任何 DTL 源文件。

- [ ] **[WF-0-5-7 联检前需统一] [域 Lead 决议] DTL-018 vs DTL-036 Player 域归属歧义**
  - DTL-018（`docs/02-运维安全与网络/RGS-DTL-018_详细设计书.md`，2026-08-17 制定）标题为"账号身份、第三方登录与合规"，覆盖 PL 限界上下文中 account_identity_links / identity_binding_audit_logs / compliance_profiles / identity_verification_vault / minor_restriction_audit_logs 五表，是 Player 域的"身份联合与合规"子模块 DTL。
  - DTL-036（`docs/01-核心架构与设计模式/RGS-DTL-036_Player域_详细设计书.md`，2026-08-21 制定）标题为"Player 域 Atomic App 契约骨架"，只冻结集群契约（`app_id: player-service` / `db: player_db` / gRPC + Event 骨架），明确声明"物理 DDL、字段级 IDL 和容量参数须在 Gate 通过后补齐"（§6 待补齐项）。
  - 实际现状：Player 域 Lead 字段级 Review 同时依赖 DTL-018（子模块级字段已落实）+ DTL-036（契约骨架已冻结，字段级尚未落实），两者存在"时间错位 + 粒度错位"——DTL-018 早于 DTL-036 4 天制定，**REV-004 附件 A §A.2 文件指向仅列 DTL-018**（未列 DTL-036）。
  - **决议待定**：是否将 DTL-036 字段级补齐作为 Q-025 独立任务，还是合并进 DTL-018 v0.2；DTL-036 §6 待补齐项 4 条（账号/角色/会话物理 DDL + proto 字段号 + 字段权威清单 + testkit 夹具）何时启动。

- [ ] **[WF-0-5-7 联检前需统一] [域 Lead 决议] DTL-019 §0 描述与源文件标题不一致**
  - REV-004 附件 A §A.5 引用的"RGS-DTL-019 消息分发"描述（per 主对话提示"§0 表 玩家治理/策略/封禁 vs 源文件 消息推送/兑换码 描述不符"）。
  - 源文件（`docs/07-社交运营与玩家治理/RGS-DTL-019_详细设计书.md`）实际标题为"消息推送与兑换码运营工具"，涵盖 PushConsentStore / PushDispatcher / PushGatewayAdapter / PushContentSanitizer 推送组件 + RedemptionCodeBatch / RedemptionCode / RedemptionRecord 兑换码三表。
  - 实际归属：per DTL-019 §0 标题 + 内容 + 表落位（`redemption_code_batches`/`redemption_codes`/`redemption_records` 落位 AD 限界上下文），DTL-019 是 **Social 域的"消息推送+兑换码"组合 DTL**（推送部分 PL 限界上下文 + 兑换码部分 AD 限界上下文），不是"玩家治理/策略/封禁"。
  - **决议待定**：REV-004 附件 A §A.5 §0 表的描述文字是否同步为"消息推送与兑换码运营工具"（推荐）；DTL-019 是否在 §6 后续版本中拆为两个独立 DTL（推送 vs 兑换码），还是维持组合 DTL。

- [ ] **[WF-0-5-7 联检前需统一] [域 Lead 决议] DTL-026 §0 描述与文件名路径归属不一致**
  - REV-004 附件 A §A.4 §0 表对 DTL-026 的归属描述（per 主对话提示"§0 表 社交运营与玩家治理扩展 vs 实际 match 域"）。
  - 源文件路径为 `docs/07-社交运营与玩家治理/RGS-DTL-026_详细设计书.md`（07 目录 = 社交运营与玩家治理），但 DTL-026 文档标题为"匹配系统：队列/评分物理数据库设计・事件线格式・扩圈与跨分片撮合算法详细设计"，内容覆盖 MT 限界上下文（`match_db`）的 queue_entries / match_ratings / match_quality_metrics / rating_settlement_receipts 四表。
  - 实际归属：per DTL-026 §2 DDL（`queue_entries` / `match_ratings` 落位 `match_db`）+ 内容（扩圈算法 + Glicko-2 评分），DTL-026 是 **Match 域**核心 DTL。
  - **决议待定**：DTL-026 文件是否在 NO-GO 解除后从 `docs/07-社交运营与玩家治理/` 移动到 `docs/08-Match域/`（如该目录已建）或 `docs/01-核心架构与设计模式/`，还是仅在文件名加 "(Match 域)" 后缀（推荐：路径迁移）；REV-004 附件 A §A.4 §0 表的归属描述同步。

**3 处歧义均不阻断本签字（per DEC-008 形式解除 NO-GO）**，但 G-CODE-05 完全关闭（field-level DD Review Gate）须 3 处全部决议落地。

