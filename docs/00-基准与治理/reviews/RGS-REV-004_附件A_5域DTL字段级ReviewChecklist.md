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

## §A.2 Player 域（player 域 Lead：架构师兼任）

> 文件：RGS-DTL-018 / RGS-SPEC-DTL-018
> 检查项：§A.1 全部 + 以下域特定

- [ ] **A2.1 玩家档案表**：`players` 表的所有字段（含 PII：邮箱/手机号/实名认证状态）定义完整
- [ ] **A2.2 角色表**：`player_characters` / `player_inventory` 索引策略（按 player_id 分区）
- [ ] **A2.3 登录态**：JWT / session 字段与 RGS-REQ-007（GM 后台管控）一致
- [ ] **A2.4 Player Lead 签字**：_______ 日期：_______

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

## §A.6 Admin 域（SRE Lead 兼任 / COC 域）

> 文件：RGS-DTL-031 / RGS-SPEC-DTL-031
> 检查项：§A.1 全部 + 以下域特定

- [ ] **A6.1 ClusterOps 表**：`cluster_nodes` / `feature_activations` / `pfa_operations` 表
- [ ] **A6.2 状态机**：feature 状态（declared → canary → confirm → done / rolled_back）转移图
- [ ] **A6.3 错误码**：PFAU 5 类错误码（confirm 失败 / 节点掉线 / 资源不足 / 灰度不一致 / 回滚失败）
- [ ] **A6.4 ADR-0052 贯穿**：all-reachable 确认语义 + Active-Active 写入路径在每个 gRPC 方法中体现
- [ ] **A6.5 DLQ 处理**：DiscardDlqEvent / ListDlqEvents（per f0b2432 self-review 补强）
- [ ] **A6.6 监控**：PFAU 完成时延指标（按 handoff §4.3 R1 估算 ~13 分钟）
- [ ] **A6.7 SRE Lead 签字**：_______ 日期：_______

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
