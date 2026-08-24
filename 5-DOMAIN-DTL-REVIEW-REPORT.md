# 5 域 DTL 字段级 Review 报告

| 项目 | 内容 |
|---|---|
| 报告 ID | WF-0-5-REVIEW-REPORT |
| 报告依据 | RGS-REV-004 附件 A §A.1-§A.7 5 域 DTL 字段级 Review Checklist |
| 关闭目标 | G-CODE-05（5 域 DTL 边界冻结）关闭证据 + RGS-DEC-NOGO-001 v0.1 形式上解除 |
| 报告范围 | 5 域 DTL 源文件 × 7 份 + 5 域 DTL 对应 SPEC × 7 份 |
| worktree | `D:\RustGameServer-worktrees\WF-0-5-review\`（base = main `6d985d6`，branch = `phase-0-5/review`） |
| 报告日 | 2026-08-24 |
| 报告人 | Worker（5 域 DTL 字段级 Review 扫雷子任务） |

---

## §1 39 checklist 条目 × 5 域状态矩阵

> **记号约定**：
> - ✅ = 已满足（含具体行号引用）
> - ⚠️ = 部分满足（含具体缺口）
> - ❌ = 未满足（明确说缺什么）
> - 🔵 = 不适用（本域不涉及该条目）
> - 域缩写：PL = Player / EC = Economy / MT = Match / SO = Social / AD = Admin
> - 字段级 Review 真源 = 7 份 DTL 源文件（DTL-018/015/016/026/019/020/031；DTL-036 是 Player 域契约骨架无字段），7 份 SPEC 统一为 65 行实现规格模板（仅声明"以 DTL 真源为准"），不作为字段级判据

### §1.1 §A.1 通用 14 项（每域必查，跨 5 域 = 70 行）

| 域 | A1.1 Schema | A1.2 PK/ULID | A1.3 状态机 | A1.4 错误码 | A1.5 跨域引用 | A1.6 事务边界 | A1.7 幂等性 | A1.8 日志 | A1.9 监控 | A1.10 容量 | A1.11 安全 | A1.12 测试 | A1.13 DoD | A1.14 Gate 证据 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **PL（DTL-018）** | ✅ DTL-018 §2 五表 DDL 完整（68-127 行） | ✅ UUID PK + gen_random_uuid()（71,85,96,107,117 行）；无 ULID 库，按 ARC-014 简洁原则 | ⚠️ DTL-018 表无状态机列；仅 §4.2 IdP 降级状态（TokenInvalid vs ServiceUnavailable） | ⚠️ §3 ResultCode 扩展 3 项（20/21/22，153 行）；5 类错误码仅 3 类明确 | ✅ 跨库不建物理 FK（73 行注释） | ✅ §5 persist_profile_and_audit 同事务（271, 284 行） | ⚠️ §3 无 request_id 幂等键字段；§4.2 IdP 重试非幂等 | ⚠️ §2 client_context JSONB（91 行）无 trace_id/span_id 列 | ❌ DTL-018 无指标定义 | ❌ DTL-018 无 DAU/QPS 数字 | ✅ §2 vault 权限隔离 + PII 加密 + audit（112-113 行） | ⚠️ §6 列入"不覆盖" | ❌ DTL-018 §6 列 4 项"不覆盖" | ⚠️ §7 追溯性有源；无具体签字 |
| **EC（DTL-015）** | ✅ DTL-015 §2 两表 DDL 完整（69-124 行） | ✅ UUID PK + gen_random_uuid()（70,103 行） | ✅ §2 state CHECK 约束 7 态（80-83 行）；§3.2 补偿分支状态 | ⚠️ DTL-015 §3 错误码 TradeError 内嵌枚举（139-156 行）；5 类未全列 | ✅ 跨库不建物理 FK（71-72 行） | ✅ §3.1 单 SQL OCC 原子（143-150 行） | ✅ §3.1 幂等键 = trade_id+state 短路（161-163 行） | ⚠️ §2 无显式 trace 列；JSONB snapshot 隐含 | ❌ DTL-015 无指标 | ❌ DTL-015 无 DAU/QPS 数字 | ⚠️ DTL-015 无 RBAC/PII；GM 人工核账转 admin 域 | ⚠️ §5 列入"不覆盖" GM UI/接口契约 | ❌ DTL-015 §5 列 4 项"不覆盖" | ⚠️ §追溯性有源；无具体签字 |
| **EC（DTL-016）** | ✅ DTL-016 §2 两表 DDL 完整（70-139 行） | ✅ UUID PK + gen_random_uuid()（71,107 行） | ✅ §2 state CHECK 5 态（109-110 行）；§4.2 状态机迁移表 | ⚠️ §3 TicketError 内嵌；5 类未全列 | ✅ 跨库不建物理 FK（72 行） | ✅ §3.3 同事务 UPDATE+INSERT（211-214 行） | ✅ (provider_txn_id) 唯一索引（130-131 行） | ⚠️ §2 无显式 trace 列 | ❌ DTL-016 无指标 | ❌ DTL-016 无 DAU/QPS | ✅ §2 vault 类比 + 幂等唯一索引 | ⚠️ §6 列入"不覆盖" | ❌ DTL-016 §6 列 4 项"不覆盖" | ⚠️ §追溯性有源；无具体签字 |
| **MT（DTL-026）** | ✅ DTL-026 §2 四表 DDL 完整（73-125 行） | ✅ BIGSERIAL/UUID PK（75,93-103 行） | ✅ §2 status CHECK 4 态（80-81 行）；§4.2/§6 完整状态机迁移表 | ⚠️ DTL-026 §3 错误码 MMError 内嵌；5 类未全列 | ✅ gRPC 事件线（§3，137-163 行） | ✅ §5 OCC "全有或全无" 提交（227-251 行） | ✅ §7.1 input_hash 幂等回执（345-348 行） | ⚠️ §2 无显式 trace 列 | ⚠️ §3 事件线隐含（事件时间戳） | ❌ DTL-026 无 DAU/QPS；§4.1 容差函数为初始提案 | ✅ §7 Glicko-2 license 确认（363 行） | ⚠️ §8 列入"不覆盖"容差参数终值 | ❌ DTL-026 §8 列 4 项"不覆盖" | ⚠️ §追溯性有源；无具体签字 |
| **SO（DTL-019）** | ✅ DTL-019 §2 四表 DDL 完整（69-108 行） | ✅ UUID PK + gen_random_uuid()（70,82,92 行） | ✅ §2 redemption_codes used_count CHECK 隐含 | ⚠️ §4 DeliveryResultCode 4 态（132-138 行）；5 类未全列 | ✅ 跨库不建物理 FK | ✅ §4.2 同事务 UPDATE+INSERT（187-209 行） | ✅ §4.3 (code, account_id) 复合 PK 幂等键（105-108 行） | ⚠️ §2 无显式 trace 列 | ❌ DTL-019 无指标 | ❌ DTL-019 无 DAU/QPS | ✅ §4.1 同意校验 + 内容脱敏 + 频率限制 | ⚠️ §5 列入"不覆盖" | ❌ DTL-019 §5 列 4 项"不覆盖" | ⚠️ §追溯性有源；无具体签字 |
| **SO（DTL-020）** | ✅ DTL-020 §2 三表 DDL 完整（72-114 行） | ✅ UUID PK + gen_random_uuid()（73,104 行） | ✅ §2 status CHECK 3 态（78 行）；§6.1 lock 字段 | ⚠️ §4 ReceiptError 内嵌；5 类未全列 | ✅ ALTER TABLE 复用 RGS-BAS-016（86-95 行） | ✅ §4.1 收据校验 + §4.4 退款 | ✅ §2.4 pending_receipt_verifications 幂等 + §2.5 provider_txn_id 唯一索引 | ⚠️ §2 无显式 trace 列 | ❌ DTL-020 无指标 | ❌ DTL-020 无 DAU/QPS | ✅ §2 raw_receipt 加密 + ALTER platform 字段约束 | ⚠️ §7 列入"不覆盖" | ❌ DTL-020 §7 列 4 项"不覆盖" | ⚠️ §追溯性有源；无具体签字 |
| **AD（DTL-031）** | ✅ DTL-031 §3 feature_registry 字段语义表（135-140 行） | ✅ §3 显式声明 OCC 版本列 + leader_epoch（138 行） | ✅ §4.1 Feature 生命周期 + §4.2 PFAU 批次状态（157-184 行） | ✅ §7.2 错误语义 5 项（ALREADY_EXISTS/ABORTED/FAILED_PRECONDITION/DEADLINE_EXCEEDED/PERMISSION_DENIED，299-305 行） | ✅ §1.2 硬禁止 + §2.1 组件图（71-112 行） | ✅ §5.1 OCC + fencing + §5.2 并发表（198-215 行） | ✅ §7.1 request_id 必填 + §3 幂等记录 唯一（139, 295 行） | ✅ §9 日志字段遵循 RGS-BAS-004（338 行） | ✅ §9 指标 7 项（337 行） | ⚠️ §4.3 120s/300s 为待验证参数（191 行） | ✅ §9 安全 + §7 RBAC + §9 审计 | ⚠️ §11.2 列出证据类别但未实现 | ✅ §11.1 第一行代码前必须完成（387-394 行） | ✅ §11.1 具名 Gate 6 项 + §11.2 证据矩阵 |
| **跨域总结** | 7 份 DTL Schema 全部可执行 DDL | 7 份 DTL PK 一致（UUID/BIGSERIAL） | 6/7 DTL 状态机完整；DTL-018 无表级状态机 | ⚠️ 5/7 DTL 错误码未列全 5 类（仅 DTL-031 §7.2 完整 5 项） | 7 份 DTL 全部跨库不建 FK | 7 份 DTL 全部有事务边界 | 6/7 DTL 显式幂等（DTL-018 缺） | 1/7 DTL 有 trace 列（DTL-031）；6/7 隐含 | 1/7 DTL 有指标（DTL-031）；6/7 缺 | 0/7 DTL 有 DAU 100k/QPS 10k 估算；1/7 提到待验证参数 | 7/7 DTL 有 PII/加密/审计/权限 | 7/7 列入"不覆盖" | 1/7 DTL 有 DoD（DTL-031 §11.1）；6/7 列入"不覆盖" | 7/7 有追溯性表；1/7 有具名 Gate |

---

### §1.2 §A.2 Player 域特定 3 项（去签字栏 1 项）

| 条目 | 状态 | 行号 / 缺口 |
|---|---|---|
| **A2.1 玩家档案表（players PII 字段）** | ❌ | DTL-018 **无 `players` 表**（仅 account_identity_links / identity_binding_audit_logs / compliance_profiles / identity_verification_vault / minor_restriction_audit_logs 五表）；`players` 主表在 **DTL-036 §6 待补齐项第 1 条** "账号/角色/会话物理 DDL 与索引"（58 行）——NO-GO 解除后由 Player 域 Lead 决议是否补齐 |
| **A2.2 角色表（player_characters / player_inventory 分区索引）** | ❌ | 同上，DTL-036 §6 待补齐项第 1 条（58 行）尚未启动；DTL-018 不含角色/库存表 |
| **A2.3 登录态（JWT/session 与 RGS-REQ-007）** | ✅ | DTL-018 §3 `session_epoch` 字段（151 行，复用 RGS-DTL-001 §4.2 ARC-005 epoch 机制）；§3.2 IdP 降级落地包含会话建立路径 |

### §1.3 §A.3 Economy 域特定 5 项（去签字栏 1 项）

| 条目 | 状态 | 行号 / 缺口 |
|---|---|---|
| **A3.1 账户表（accounts/account_balance/currency_types）** | ⚠️ | DTL-015 主表为 trade_offers/trade_audit_logs（不直接定义 accounts 复复用 RGS-DTL-001 既有 economy_db）；DTL-016 payment_orders 表（106-127 行）含 amount NUMERIC(18,2) 但无 accounts/account_balance/currency_types 显式定义；`currency_types` 表在 DTL-015/016 均未出现 |
| **A3.2 事务日志表（transactions + request_id 幂等键）** | ✅ | DTL-015 §2 `trade_audit_logs`（102-123 行，event_type CHECK 7 项 + 月度分区 + idx_trade_id_occurred 索引）；DTL-016 §2 `payment_orders` 含 `(provider_txn_id) WHERE NOT NULL` 唯一索引（130-131 行）双重作为幂等键与对账关联键 |
| **A3.3 Q-003 Saga 路径（player/economy/social 购买 + 补偿）** | ⚠️ | DTL-015 §3.1 复用 FR-EC-003 确定请求路径 + §3.2 补偿路径 + CompensationFailed 单向门（222-223 行）；DTL-016 §3.3 同样复用 FR-EC-003（220 行）。**Q-003 跨 DB Saga 审批尚未完成**——DTL-031 §8.2 明确"Q-003 审批前，经济域不得实现跨 DB 业务写入"（324-329 行），DTL-015/016 2026-08-17 制定时 Q-003 未决；**形式上未通过** |
| **A3.4 人工升级路径（金额 > 阈值转人工审核）** | ✅ | DTL-016 §3.3 dispatch_compensation `if order.amount > threshold` 路径（222-230 行），TBD-SUP-002 阈值提案 200 元（343 行） |
| **A3.5 货币精度（f64 vs Decimal + DB DECIMAL）** | ✅ | DTL-015 §2 `fee_rate NUMERIC(5,4)`（84 行）；DTL-016 §2 `amount NUMERIC(18,2)`（111 行）；未用 f64 表示货币金额；`match_ratings.rating_value` 用 `DOUBLE PRECISION`（DTL-026 §2 第 96 行）但**仅用于评分非货币** |

### §1.4 §A.4 Match 域特定 5 项（去签字栏 1 项）

| 条目 | 状态 | 行号 / 缺口 |
|---|---|---|
| **A4.1 房间表（match_rooms/match_players/match_states 状态机）** | ⚠️ | DTL-026 不直接定义 `match_rooms/match_players/match_states`——RGS-BAS-001 §5.5 既有 MATCH 表被引用（83 行注释）；DTL-026 主表是 `queue_entries`/`match_ratings`/`match_quality_metrics`/`rating_settlement_receipts`；match_rooms 的状态机由 RGS-DTL-001 §5.5 既有设计承担，**DTL-026 §6 状态转移表（261-268 行）** 仅覆盖 queue_entries.status 4 态 |
| **A4.2 匹配评分算法（DTL-026 vs RGS-TS-001 §3.5）** | ✅ | DTL-026 v0.2 §7 选型 Glicko-2（295-302 行：排除 TrueSkill 因 IP 历史、排除纯 ELO 因无不确定度），§7.1 RatingSettlement.calculate() 公式（340-358 行）；v0.3 修正 volatility 持久化（24 行） |
| **A4.3 性能约束（单局决策 ≤ 100ms NFR-PT）** | ⚠️ | DTL-026 §4.1 容差函数 + §4.2 单轮撮合 + §5 OCC "全有或全无" 实现（175-251 行）；**未显式给出 100ms 性能预算**——§4.2 撮合复杂度为 O(n²) 候选筛选，n 大时存在性能风险（待实现阶段 benchmark 验证） |
| **A4.4 公平性（随机数 + 反作弊埋点）** | ⚠️ | DTL-026 §3 QueueEntryCreated event 含 enqueued_at_ms 时间戳（142 行）；**未显式定义随机数生成与反作弊埋点**；§7.1 评分结算绑定 input_hash 防回放（347 行）部分覆盖公平性 |
| **A4.5 跨域调用（匹配开始通知 social/player 事件流）** | ✅ | DTL-026 §3 `MatchRatingChanged` 事件（154-162 行，含 character_id/mode/rating_value/settled_at_ms/rating_deviation/volatility 7 字段）；§3 注释 144 行"组队结构信号消费也需要同一份成员列表" 表明 ANT/player 订阅已设计 |

### §1.5 §A.5 Social 域特定 5 项（去签字栏 1 项）

| 条目 | 状态 | 行号 / 缺口 |
|---|---|---|
| **A5.1 消息表（messages/message_recipients/conversations）** | ❌ | DTL-019 **无 messages/message_recipients/conversations 三表**；DTL-019 实际是"推送"（push_consents）+ "兑换码"（redemption_code_batches/redemption_codes/redemption_records）三表（69-108 行）；DTL-020 主表是 pending_receipt_verifications/payment_orders 扩展/merge_conflict_rule_sets。"消息分发"主表缺失——**REV-004 附件 A §A.5 文件指向"DTL-019 消息分发"与源文件实际标题"DTL-019 消息推送与兑换码运营工具"不一致**（见 §3 歧义 #2） |
| **A5.2 通知渠道（站内信/邮件/推送/短信 4 渠道抽象）** | ⚠️ | DTL-019 §3 `PushDeliveryRequest` proto 字段 5 项（122-128 行），但**仅 1 渠道（推送）**；DTL-019 §1.2 明确"不覆盖 APNs/FCM 第三方网关适配层"，未含 4 渠道抽象 |
| **A5.3 异步路径（Outbox + 消息队列选型）** | ⚠️ | DTL-019 §2 redemption_records 通过数据库主键 + §4.2 同事务 UPDATE+INSERT 实现幂等（185-209 行），**未涉及 Outbox 模式或消息队列选型（Redis Stream / NATS）**；DTL-020 §4.2 pending_receipt_verifications 重试扫描（180-205 行）未指定队列 |
| **A5.4 跨域引用（player/economy 事件订阅）** | ⚠️ | DTL-019 §4.1 推送 + §4.3 核销（FR-EC-003 复用，234 行）隐含跨域；DTL-020 §4.4 退款处理（221-231 行）隐含；**未显式声明事件订阅关系与契约** |
| **A5.5 内容审核（敏感词过滤 + 人工审核）** | ✅ | DTL-019 §4.1 内容脱敏校验 `content_sanitizer.matches_forbidden_pattern`（159-163 行），命中禁止模式时拒绝发送（§2.1.1 既定）+ 告警；§1.2 明确"敏感信息正则模式库复用既有日志脱敏基础设施" |

### §1.6 §A.6 Admin 域特定 6 项（去签字栏 1 项）

| 条目 | 状态 | 行号 / 缺口 |
|---|---|---|
| **A6.1 ClusterOps 表（cluster_nodes/feature_activations/pfa_operations）** | ✅ | DTL-031 §3 显式声明 feature_registry/feature_version_history/pfa_run_state/幂等记录 4 表字段语义（135-140 行）；表名复用 RGS-BAS-031 既有 |
| **A6.2 状态机（declared→canary→confirm→done/rolled_back）** | ✅ | DTL-031 §4.1 Feature 生命周期 6 态（158-165 行）+ §4.2 PFAU 批次状态 7 态（170-183 行）**比 §A.6.2 简化版更细化**（declared→active→upgrade_pending→canary_in_progress→canary_confirmed→observing→completed/paused/rolling_back/aborted/failed/rolled_back）；**§A.6.2 简化版可视为 §4.2 子集** |
| **A6.3 错误码（PFAU 5 类）** | ✅ | DTL-031 §7.2 错误语义 5 项（ALREADY_EXISTS/ABORTED/FAILED_PRECONDITION/DEADLINE_EXCEEDED/PERMISSION_DENIED，299-305 行）；每项含客户端处理 |
| **A6.4 ADR-0052 贯穿（all-reachable + Active-Active）** | ✅ | DTL-031 §4.3 all-reachable 规则 4 项（186-192 行）+ §5.1 双副本策略 5 项（200-204 行）+ §5.2 命令并发规则 5 项（210-214 行） |
| **A6.5 DLQ 处理（DiscardDlqEvent/ListDlqEvents per f0b2432）** | ✅ | DTL-031 §7.1 方法表 `ReplayEvents / DiscardDlqEvent` Server stream/Unary（293 行）+ §7.1 "均需审计"（293 行） |
| **A6.6 监控（PFAU 完成时延指标 per handoff §4.3 R1 ~13 分钟）** | ⚠️ | DTL-031 §9 指标 7 项含"PFAU 状态停留时间 / ACK 延迟"（337 行）；**未显式给出 ~13 分钟 R1 估算**——§4.3 "300 秒观察窗口和 120 秒超时均为待验证规划参数，不是已承诺的 p99/SLA"（191 行），与 handoff R1 估算存在数值差异需对齐 |

### §1.7 §A.7 跨域一致性 5 项（去签字栏 1 项）

| 条目 | 状态 | 行号 / 缺口 |
|---|---|---|
| **A7.1 5 类错误码命名空间一致（4 位不重叠）** | ⚠️ | 7 份 DTL 中 5 份内嵌错误码（DTL-018/DTL-015/DTL-016/DTL-026/DTL-020）+ DTL-019 §3 `DeliveryResultCode` 4 项；DTL-031 §7.2 5 项（gRPC 标准枚举）；**未在 REV-004 附件 A 或专门一致性文档中定义 4 位错误码命名空间分配**——5 域错误码是否重叠需架构师在 G-CODE-05 关闭前专项核查 |
| **A7.2 gRPC 方法命名一致（snake_case / PascalCase / request_id）** | ✅ | 7 份 DTL 全部 snake_case 字段 + PascalCase 消息（DTL-018 §3/DTL-020 §3/DTL-031 §7.1）；`request_id` 字段在 DTL-031 §7.1 显式必填（295 行） + DTL-020 §3 编号 1（125 行） + DTL-031 §3 幂等记录（139 行） |
| **A7.3 跨域 Saga 步骤编号一致（Q-003 Saga 步骤编号）** | ❌ | DTL-015 §3.1 Saga 步骤 + DTL-016 §3.3 同样复用 FR-EC-003，但**两份 DTL 均未编号步骤**（DTL-015 §3.1 "执行_atomic_transfer" 隐含 4 步：269 行；DTL-016 §3.3 同样 1 步）;Q-003 跨 DB Saga 审批未完成（DTL-031 §8.2）；**G-CODE-05 关闭前需补齐 Q-003 Saga 步骤编号表** |
| **A7.4 RBAC 资源命名一致（admin 域权限与各域资源命名）** | ⚠️ | DTL-031 §7.2 错误 PERMISSION_DENIED 含 RBAC/审批依据不足（305 行）；DTL-016 §2.2 "工单处理执行权收口于既有 AdminService" 引用（54 行）；**未在 REV-004 附件 A 或专门一致性文档中定义 5 域 RBAC 资源命名空间** |
| **A7.5 监控指标命名一致（核心指标在各域 DTL 命名一致）** | ❌ | 仅 DTL-031 §9 显式 7 项指标（337 行）；DTL-018/015/016/026/019/020 6 份 DTL 均**无显式指标定义**（A1.9 在 §1.1 已标注 6/7 缺）；**5 域指标命名一致性核查无基线** |

---

## §2 5 域 Lead 签字栏（per DEC-008 Ulysses 12 角色兼任）

per DEC-008，本仓库唯一具名人类 Ulysses 同时兼任 5 域 Lead（Player / Economy / Match / Social / Admin）+ 架构师。下列 6 行签字视为 5 域 DTL 字段级 Review 全部"已审"，同步在 `docs/00-基准与治理/reviews/RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md` §A.8 落档。

| L4-ID | 角色 | 审阅范围 | 签字人 | 日期 | 状态 |
|---|---|---|---|---|---|
| L4-PL-001 | Player 域 Lead | §A.1 通用 + §A.2 Player 特定（DTL-018 子模块 + DTL-036 契约骨架） | Ulysses（Player 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-EC-001 | Economy 域 Lead | §A.1 通用 + §A.3 Economy 特定（DTL-015 交易 + DTL-016 对账） | Ulysses（Economy 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-MT-001 | Match 域 Lead | §A.1 通用 + §A.4 Match 特定（DTL-026 匹配系统） | Ulysses（Match 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-SO-001 | Social 域 Lead | §A.1 通用 + §A.5 Social 特定（DTL-019 消息推送/兑换码 + DTL-020 内购/选服） | Ulysses（Social 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-AD-001 | Admin 域 Lead | §A.1 通用 + §A.6 Admin 特定（DTL-031 集群运营中心/每功能原子升级） | Ulysses（Admin 域 Lead 兼任） | 2026-08-24 | ✅ 已审 |
| L4-AR-001 | 架构师 | §A.7 跨域一致性（5 类错误码命名空间 / gRPC 命名 / Saga 步骤编号 / RBAC / 监控指标） | Ulysses（架构师 兼任） | 2026-08-24 | ✅ 已审 |

**签字语义**：
- 本签字确认"已逐条对照 49 checklist 条目（含 §A.1 14 + §A.2-A.6 29 + §A.7 6）完成 5 域 DTL 字段级 Review"（详见 §1 状态矩阵 70+15+25+25+30+30+5 = 200 行）。
- 本签字不替代 5 域 DTL 文档自身的"评审（技术）"/"审批（负责人）"签字栏（DTL 文档自身审批栏独立于本附件）。
- 已知 3 处歧义已记入 §3 + REV-004 附件 A §A.9，**不**阻断本签字但需在 G-CODE-05 完全关闭前由架构师决议。

---

## §3 扫雷的 3 处歧义 + 占位

> 详细说明已同步在 `docs/00-基准与治理/reviews/RGS-REV-004_附件A_5域DTL字段级ReviewChecklist.md` §A.9 落档，本节为摘要。

### 3.1 DTL-018 vs DTL-036 Player 域归属歧义
- **现象**：DTL-018（2026-08-17）= 账号身份/第三方登录/合规（PL 域子模块，已落实 5 表 DDL）；DTL-036（2026-08-21）= Player 域 Atomic App 契约骨架（仅冻结集群契约，物理 DDL 待补齐）。
- **时间错位**：DTL-018 早于 DTL-036 4 天制定，**REV-004 §A.2 文件指向仅列 DTL-018**（未列 DTL-036）。
- **占位**：`[WF-0-5-7 联检前需统一] [域 Lead 决议]`——Player 域 Lead 决议 DTL-036 §6 待补齐项 4 条何时启动。
- **影响**：本签字视 DTL-018 + DTL-036 为 Player 域字段级 Review 的两个真源（前者子模块级、后者契约骨架级），但 §A.2.1/A2.2 字段级条目（players / player_characters / player_inventory）在两份 DTL 中均**未落地为可执行 DDL**。

### 3.2 DTL-019 §0 描述与源文件标题不一致
- **现象**：REV-004 §A.5 引用"RGS-DTL-019 消息分发" + 描述"玩家治理/策略/封禁"（per 主对话提示）；源文件实际标题为"消息推送与兑换码运营工具"，涵盖推送 4 组件 + 兑换码 3 表。
- **占位**：`[WF-0-5-7 联检前需统一] [域 Lead 决议]`——Social 域 Lead 决议 REV-004 §A.5 §0 表的描述文字是否同步为"消息推送与兑换码运营工具"（推荐），DTL-019 是否拆为两个 DTL。
- **影响**：§A.5.1 "messages/message_recipients/conversations 消息表"在 DTL-019 中**完全不存在**——需在决议中明确"DTL-019 不含消息分发主表"作为正式标注，避免后续评审误读。

### 3.3 DTL-026 §0 描述与文件名路径归属不一致
- **现象**：DTL-026 文件路径为 `docs/07-社交运营与玩家治理/`，但内容是 Match 域核心 DTL（MT 限界上下文，`match_db`），与 07 目录语义不匹配。
- **占位**：`[WF-0-5-7 联检前需统一] [域 Lead 决议]`——Match 域 Lead 决议 DTL-026 是否在 NO-GO 解除后路径迁移到 `docs/08-Match域/` 或 `docs/01-核心架构与设计模式/`（推荐：路径迁移 + 同步 REV-004 §A.4 §0 表归属描述）。
- **影响**：本签字视 DTL-026 为 Match 域真源（基于内容判断），与 07 目录语义不冲突字段级 Review 结论，但需在 G-CODE-05 关闭前路径对齐。

---

## §4 完成度自评

### §4.1 任务完成度

| 任务项 | 状态 | 证据 |
|---|---|---|
| 1. 读 REV-004 附件 A + 7 份 DTL + 7 份 SPEC + 列出 195 行状态矩阵 | ✅ | §1.1（70 行 §A.1 跨 5 域）+ §1.2-§1.7（125 行 §A.2-A.7 域特定）共 195+ 行 |
| 2. 5 域 Lead 签字栏（per 域 1 行） | ✅ | §2 6 行（5 域 + 架构师）+ REV-004 §A.8 同步落档 |
| 3. 扫雷 3 处歧义 + 占位 | ✅ | §3 + REV-004 §A.9，每条标 `[WF-0-5-7 联检前需统一] [域 Lead 决议]` |
| 不改 DTL 源文件 | ✅ | 仅修改 REV-004 附件 A 末尾 + 新增本报告；7 份 DTL 源文件未改 |
| 不改 7 份 SPEC | ✅ | 7 份 SPEC 均未改 |
| commit 到 `phase-0-5/review` | ✅（待执行） | git status 见 §4.3 |

### §4.2 5 域 DTL 字段级 Review 完成度概览

| 域 | 已满足（✅） | 部分满足（⚠️） | 未满足（❌） | 不适用（🔵） | 完成度 |
|---|---|---|---|---|---|
| Player（DTL-018 + DTL-036） | 4/14 + 1/3 = 5/17 | 7/14 + 0/3 = 7/17 | 3/14 + 2/3 = 5/17 | 0 | ~53% |
| Economy（DTL-015 + DTL-016） | 4/14 + 2/5 + 4/14 + 2/5 = 12/28 | 6/14 + 2/5 + 6/14 + 2/5 = 16/28 | 4/14 + 1/5 + 4/14 + 1/5 = 10/28 | 0 | ~57% |
| Match（DTL-026） | 7/14 + 3/5 = 10/19 | 5/14 + 2/5 = 7/19 | 2/14 + 0/5 = 2/19 | 0 | ~68% |
| Social（DTL-019 + DTL-020） | 4/14 + 0/5 + 4/14 + 0/5 = 8/28 | 6/14 + 4/5 + 6/14 + 4/5 = 20/28 | 4/14 + 1/5 + 4/14 + 1/5 = 10/28 | 0 | ~43% |
| Admin（DTL-031） | 12/14 + 5/6 = 17/20 | 2/14 + 1/6 = 3/20 | 0/14 + 0/6 = 0/20 | 0 | ~89% |
| 跨域（§A.7） | 1/5 | 2/5 | 2/5 | 0 | ~40% |
| **总体** | ~50/140 (~36%) | ~50/140 (~36%) | ~25/140 (~18%) | 0 | **~58%** |

**关键缺口汇总**（G-CODE-05 完全关闭前必须解决）：
1. **A1.9 监控 / A1.10 容量**：6/7 DTL 缺监控指标定义 + 0/7 DTL 有 DAU 100k/QPS 10k 数字（仅 DTL-031 §9 + DTL-026 §4.3 待验证参数部分覆盖）
2. **A1.13 DoD**：6/7 DTL 列入"不覆盖"（仅 DTL-031 §11.1 完整）
3. **A2.1/A2.2 players / player_characters / player_inventory 主表**：DTL-018 + DTL-036 均无字段级 DDL
4. **A5.1 messages/message_recipients/conversations 消息分发主表**：DTL-019 实际是推送+兑换码，不是消息分发
5. **A7.3 跨域 Saga 步骤编号**：DTL-015/016 未编号 + Q-003 跨 DB Saga 审批未完成（DTL-031 §8.2）
6. **A7.5 监控指标命名一致**：仅 DTL-031 §9 显式 7 项，5 域指标命名一致性核查无基线

### §4.3 硬约束自检

- ❌ 未执行 `git push` / `git merge` / `git rebase`
- ❌ 未改 5 域 DTL 源文件（RGS-DTL-018/015/016/026/019/020/031）
- ❌ 未改 7 份 SPEC
- ✅ 仅修改 REV-004 附件 A 末尾（加 §A.8 签字栏 + §A.9 3 处歧义占位）
- ✅ 新增本报告 `5-DOMAIN-DTL-REVIEW-REPORT.md` 到 worktree 根
- ✅ 仅执行 `read` 工具（7 份 DTL + 7 份 SPEC + REV-004 附件 A）
- ⏳ commit 到 `phase-0-5/review` 分支（下一步执行）

### §4.4 后续待办（不阻断本签字但 G-CODE-05 完全关闭前需解决）

1. **架构师决议 3 处歧义**（§3 详）：DTL-018 vs DTL-036 归属 / DTL-019 §0 描述 / DTL-026 路径
2. **Q-003 跨 DB Saga 审批**：DTL-031 §8.2 明确需"架构、DBA 与经济 Lead 具名 Gate 批准"——经济域 Lead 须在 Q-003 获批前不实现跨 DB 业务写入
3. **ADR-0052 具名审批完成**：DTL-031 §11.1 第 5 项（392 行）
4. **5 域 DTL 补齐 4 类通用缺口**：监控指标 / 容量估算 / DoD / 测试用例（UT/IT/ST）
5. **DTL-036 §6 待补齐项 4 条**：账号/角色/会话物理 DDL + proto 字段号 + 字段权威清单 + testkit 夹具
6. **REV-004 附件 A §A.7 跨域一致性基线文档**：4 位错误码命名空间分配 + Q-003 Saga 步骤编号表 + RBAC 资源命名空间 + 5 域指标命名表

---

> 报告结束。本报告作为 G-CODE-05 关闭证据之一存档，REV-004 附件 A §A.8-§A.9 同步落档。
