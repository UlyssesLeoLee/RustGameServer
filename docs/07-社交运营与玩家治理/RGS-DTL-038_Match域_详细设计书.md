# 详细设计书（詳細設計書 / Detailed Design Document）

**Match 域 Atomic App 契约骨架**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-038 |
| 版本 | 0.2 |
| 状态 | **契约骨架・待评审・不得作为实施授权** |
| 父文档 | RGS-REQ-029、RGS-BAS-026、RGS-DTL-026、RGS-DTL-031 |
| App/DB | `match-service` / `match_db` |
| 制定日 | 2026-08-21 |
| 制定者 | 架构师 |
| 修订历史 | 见文末"修订历史"小节 |
| 审批 | 未审批；字段级 DD Review 前不得作为实施授权 |

## 1. 领域职责与非职责

- 负责匹配队列、匹配确认、对局编排和评分结算。
- 只消费 Player 状态契约/事件，不直接读取 `player_db`。
- 评分和匹配状态只写 `match_db`；跨域结果通过事件发布。

## 2. 集群契约

```yaml
app_id: match-service
db: match_db
depends_on: [player-service, event-bus, config, observability, secrets]
scaffold_ref: services/match-service/deploy/helm
feature_host: true
health: [/healthz, /readyz]
```

## 3. API 与事件骨架

| 类型 | 契约 | 规则 |
|---|---|---|
| gRPC | `EnqueueMatch`、`ConfirmMatch`、`CancelMatch`、`GetMatchState` | request_id 幂等；状态迁移校验 expected_version |
| Event | `MatchFound`、`MatchConfirmed`、`RatingSettled` | Outbox 发布；消费者不得依赖事件到达顺序之外的隐含状态 |
| Query | 匹配质量/队列状态 | 只读路径与写路径分离 |

## 4. 插件边界

匹配规则可作为宿主 Feature，但只能通过白名单输入/输出接口运行；任何规则版本必须可追溯、可灰度、可回滚。不得通过动态库改变运行时 ABI，不得直连其他域 DB。

## 5. 迁移、回滚与测试

- 队列和评分状态使用显式状态机与 OCC；评分结算需保证重复消息不重复结算。
- 回滚只回滚规则/服务版本，不删除已确认对局和已产生的审计事实。
- 必须覆盖：队列重复请求、确认超时、评分事件重放、插件异常、跨分片边界和 PFAU 暂停。

## 6. 待补齐项

- [ ] `queue_entries`、`match_ratings` 等物理 DDL 与索引。
- [ ] Match/Player/Economy 事件字段、版本协商和兼容窗口。
- [ ] 跨分片能力逐项清单与容量参数。
- [ ] Match 端到端契约测试与 chaos 场景。

## 7. 修订历史

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | — | 建立 Match 域契约骨架（API/事件/集群契约/插件边界/迁移回滚） | 全文（初始） |
| 0.2 | 2026-08-25 | 架构师（Mavis 接手 agent per DEC-008） | — | **同步父 BAS-026 升版至 v0.2**（1 次升版，装饰性升版）: 本 DTL 是父 BAS-026 的详细化（per DTL 头部"不改变任何既有决定"），父 BAS 升版为元数据/追溯性表/装饰性修订，DTL-038 既有契约骨架内容无实质重写，本升版仅做元数据层对齐;**正文本不重写**（per `RGS-DOCS-HEALTH-2026-08-25` §0 第 4 行"治理状态, 非文档缺陷, agent 不可代签" + 反馈单 §4 要求 1 "不预填任何 ✅, 不代签"）。审批留空，待 Ulysses 在 review 时签发。 | (无) |

## 8. 审批栏（Approval）

| 角色 | 姓名 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| 制定者（架构） | 架构师 |  | 2026-08-21 |  |
| 审核者（域内/相关域） |  |  |  | 字段级 DD Review 前不得作为实施授权 |
| 批准者（架构 / Lead） |  |  |  |  |
| 批准者（GM / 业务） |  |  |  |  |
| 批准者（运维 SRE） |  |  |  |  |
| 批准者（测试 QA） |  |  |  |  |
