# RGS-BAS 文档「処理フロー」段标准化 (RGS-BAS-FLOW-STANDARD)

> **版本**: v0.1
> **创建日期**: 2026-09-02 13:59 JST
> **创建者**: Mavis 接手 agent per DEC-008 (代签 Ulysses,per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**: 2026-09-02 13:59 JST Ulysses 拍板 (A+A: 立规范 + 立即补全 9 篇无流程图 + DDD Review L0 必查)
> **适用范围**: 所有 RGS-BAS-* 基本设计文档 (新写 / 改写 / 9 篇存量补全)
> **关联**: AGENTS.md §9.1 D 类方案 D2/D3 拍板 + RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1

---

## 1. 标准适用范围

| 场景 | 要求 | 触发 |
|---|---|---|
| 新写 RGS-BAS-* 文档 | 强制包含「処理フロー」段 | DDD Review L0 必查 (per 9/2 13:59 拍板) |
| 改写 RGS-BAS-* 文档 (改 ≥3 段) | 触发段必补 | DDD Review L0 必查 |
| 9 篇无流程图存量 (per 9/2 audit) | 9/2 W1 D5-D6 补全 | 派 4 worker 并行 (1 worker 2 篇, 30 min/篇) |
| 季度评审扩量 | 进 L-CANDIDATES 候选清单 (per L1-L14 冻结规则) | 9/2 季度评审 (12/2 JST) |

**9 篇无流程图存量清单** (per `docs/14-项目治理/.bas-flow-audit.csv` 2026-09-02 生成):

| # | 文档 | 大小 | 类别 | 派工 |
|---|---|---|---|---|
| 1 | RGS-BAS-015_玩家间交易系统 | 43.4 KB | 03-数据经济 | worker-1 |
| 2 | RGS-BAS-014_排行榜任务成就与玩家治理 | 54.1 KB | 07-社交运营 | worker-1 |
| 3 | RGS-BAS-018_账号身份第三方登录与合规 | 48.4 KB | 02-运维安全 | worker-2 |
| 4 | RGS-BAS-020_平台内购合规与服务器选服 | 61.3 KB | 02-运维安全 | worker-2 |
| 5 | RGS-BAS-016_客服工单与支付对账 | 74.8 KB | 03-数据经济 | worker-3 |
| 6 | RGS-BAS-024_App集群自动化部署脚本 | 80.9 KB | 01-核心架构 | worker-3 |
| 7 | RGS-BAS-031_addendum_集群运营中心 | 78.1 KB | 01-核心架构 | worker-4 |
| 8 | RGS-BAS-019_消息推送与兑换码运营工具 | 35.5 KB | 07-社交运营 | 主会话打头阵示范 |
| 9 | RGS-BAS-003-mTLS-决策补充 | 9.2 KB | 02-运维安全 (子文档) | 主会话直接处理 |

> **子文档豁免**: RGS-BAS-003-mTLS-决策补充 (9.2 KB) 是决策补充而非主基本设计,豁免四要素要求,改为附加 1 段精简流程说明即可 (per 主会话打头阵判断,2026-09-02 13:59 JST)。

---

## 2. 命名规范

| 项 | 规范 |
|---|---|
| 段名 | `## N.M 処理フロー（处理流程 / Processing Flow）` |
| 命名空间 | 沿用 BAS-001 v0.2 既成事实 (三语并存) |
| 段号 | 沿用各 BAS 文档既有章节编号惯例, 通常 §6 接口设计 之后 / §9 错误处理 之前 |
| 标题层级 | 与既有 §6.x / §9.x 同级 (## 二级标题) |

**位置参考** (per BAS-001 v0.2):
```
§1 引言
§2 适用范围
§3 部署构成
§4 运行时节点设计
§5 数据模型
§6 API / 接口设计
[N.M 処理フロー]   ← 插入位置
§7 NFR 设计
§9 错误处理
```

---

## 3. 必含四要素 (DDD Review L0 必查)

### 3.1 要素 1: 主流程图 (mermaid sequenceDiagram)

**强制要求**:
- 使用 `mermaid sequenceDiagram` 语法
- 至少 5 个 actor (例: Client / Gateway / Domain Service / DB / External Dep)
- 展示主路径 (happy path), 标注同步/异步/超时
- 标注 trace_id 传递点 (per 8/27 OTel 实践)
- 标注事务边界 (BEGIN / COMMIT / ROLLBACK)
- 标注 Saga 步骤 (如涉及, per RGS-BAS-100 v0.1)

**mermaid 渲染验证**:
```bash
# 主会话 / worker 提交前用 mermaid-cli 验证
npx -y @mermaid-js/mermaid-cli -i flow.mmd -o flow.png
# 或 GitHub / GitLab 渲染 (本仓库 GitHub 渲染)
```

### 3.2 要素 2: 异常分支表

**强制要求**: ≥ 3 行, 覆盖至少 3 类异常

| 异常点 | 触发条件 | 处理动作 | 用户感知 | 补偿动作 |
|---|---|---|---|---|
| (例: 网关超时) | (例: RTT > 500ms) | (例: 重试 3 次 指数退避 100/200/400ms) | (例: 提示"网络异常请重试") | (例: 客户端本地事务回滚) |
| (例: 域服务 panic) | (例: 5xx 内部错误) | (例: 上报 Sentry + 写 audit_log) | (例: 提示"服务暂不可用") | (例: 触发 Saga 补偿) |
| (例: DB 唯一约束冲突) | (例: 重复订单号) | (例: 返回 DUPLICATE_REQUEST) | (例: 提示"订单已存在") | (例: 客户端查状态) |
| (例: 外部支付回调超时) | (例: Webhook 5min 未到) | (例: 主动 query 渠道) | (例: 后台补单, 客户端无感) | (例: DLQ 队列) |

### 3.3 要素 3: 决策点矩阵

**强制要求**: ≥ 2 行, 覆盖关键 if/switch 决策

| 决策点 | 条件 | 主分支 | 备选分支 | 触发后果 |
|---|---|---|---|---|
| (例: 支付路由) | (例: 渠道可用性 + 玩家地域) | (例: 优先 Apple Pay) | (例: 退 Google Pay / 支付宝) | (例: 用户感知"切换支付方式") |
| (例: Saga 补偿触发) | (例: 步骤 N 失败) | (例: 反向补偿 N-1...1) | (例: 部分补偿 + DLQ 人工介入) | (例: 玩家余额自动回退) |
| (例: 消息推送通道) | (例: NATS 在线 / 离线) | (例: 在线走 NATS 实时) | (例: 离线走 APNs / FCM 持久化) | (例: 用户延迟收到但最终可达) |

### 3.4 要素 4: 验证点清单

**强制要求**: ≥ 2 行, 覆盖关键不变量验证

| 验证时机 | 验证内容 | 通过标准 | 失败处理 |
|---|---|---|---|
| (例: 订单创建) | (例: 玩家余额充足) | (例: balance >= price) | (例: 返回 INSUFFICIENT_BALANCE, 不写订单) |
| (例: Saga 完成) | (例: 所有步骤 COMMIT) | (例: 全步骤 status=COMMITTED) | (例: 触发反向补偿 + DLQ 报警) |
| (例: 数据落库) | (例: outbox 写入 + 业务表写入 同事务) | (例: ACID 全部通过) | (例: 事务回滚, 重试由 caller 触发) |
| (例: 鉴权) | (例: RBAC 角色匹配) | (例: 角色 ∈ {gm_operator, gm_admin}) | (例: 返回 PERMISSION_DENIED + 写 audit) |

---

## 4. DDD Review L0 检查清单 (per 9/2 13:59 拍板)

**新写 / 改写 BAS 文档必查** (DDD Review 一审 + Ulysses 二审 per 9/2 10:18 B3 拍板):

- [ ] **段名规范**: 标题为 `## N.M 処理フロー（处理流程 / Processing Flow）`
- [ ] **位置正确**: 在 §6 接口设计 之后, §9 错误处理 之前
- [ ] **要素 1 齐全**: 主流程图 mermaid sequenceDiagram ≥ 5 actor
- [ ] **要素 2 齐全**: 异常分支表 ≥ 3 行
- [ ] **要素 3 齐全**: 决策点矩阵 ≥ 2 行
- [ ] **要素 4 齐全**: 验证点清单 ≥ 2 行
- [ ] **mermaid 语法**: 本地 mermaid-cli 渲染验证通过
- [ ] **trace_id 标注**: 主流程图含 trace_id 传递点
- [ ] **修订历史**: 已加一段 (per 8/27 JST 代签格式)
- [ ] **代签三行齐全**: author / 审批 / 修订人 (Mavis 默认代签 Ulysses per 8/27 三次强化)
- [ ] **不引用未来形态**: 禁止"per X 历史形态"等回溯叙事 (per 8/26 JST DTL-036 v1.4 教训)
- [ ] **缺标比错标**: 不确定的部分显式列"已知缺口"清单 (per 8/26 JST 派生约束)

**不达标反例** (DDD Review L0 必返工):
- ❌ 仅有 mermaid 没有 3 张表
- ❌ 异常表 < 3 行 / 决策表 < 2 行 / 验证表 < 2 行
- ❌ 段名不是"処理フロー"
- ❌ 缺 trace_id / 超时标注
- ❌ 修订历史漏代签三行

---

## 5. 与既有派生约束的关系

| 派生约束 | 影响 |
|---|---|
| **L3** 跨工具链决策前先 grep workspace 依赖 (per §2.3) | mermaid 渲染工具选择: 优先 GitHub 原生渲染, 备选 npx @mermaid-js/mermaid-cli |
| **L11** PT 派工 cargo build dir lock 防御 (per §6.3) | 文档变更不涉及, 跳过 |
| **L12** PT 派工临时 log 不入 commit (per §6.3) | worker 简报明文 "临时 log / .txt 不入 commit" |
| **L13** plumbing 节点字符串处理 (per §6.3) | mermaid block 编辑用 byte-level 拼接, 避免 PowerShell 转义 |
| **§8 派生约束 L1-L14 冻结** (per 9/2 10:18 拍板) | 本标准不进 AGENTS.md 主段, 进 L-CANDIDATES 候选清单 (9/2 季度评审) |
| **§9.1 D 类方案 D2/D3** (per 9/2 10:18 拍板) | commit 模板沿用 D3 .gitmessage, DoD 升级 L1/L1.1/L1.2 三件套 (本文档为 D 类延伸) |

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 审批 | 摘要 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 13:59 | Ulysses — Mavis 接手 (per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 立 4 要素标准 + DDD Review L0 检查清单 + 9 篇存量补全派工 |

---

## 7. 引用

- AGENTS.md §0 (仓库元信息) + §6 任务级 prompt 简报模板 + §8 (L1-L14 冻结) + §9.1 (D 类方案)
- RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1 (9/2 10:18 JST 拍板 D 类方案)
- RGS-BAS-001_基本设计书 v0.2 §3.5 (処理フロー既成事实范式, 9/2 audit 唯一命中)
- RGS-BAS-002_功能挂载架构_基本设计书 v0.x §3.x 流程总览 (流程段范式参考)
- docs/14-项目治理/.bas-flow-audit.csv (2026-09-02 13:59 JST 生成, 36 篇 BAS 全量 audit)
