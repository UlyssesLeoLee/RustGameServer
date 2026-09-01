# RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1

**Token 消耗可视化子系统需求规约（rgs-web Gantt + Token 选项卡）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OLU-WEB-REQUIREMENTS-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 15:44 JST "创新一套 token 消耗图表，从需求文档设计开始"）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签） |
| 触发 | 2026-09-01 15:44 JST Ulysses "创新一套 token 消耗图表，放在甘特图那一套界面中名称为 token 的选项卡里" |
| 关联 | RGS-OLU-WEB-PLAN-2026-09-01 v0.1（设计总览）+ RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1（基本设计）+ RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1（详细设计）|
| 上游规范 | RGS-TS-001 v0.7 §6.2 OLU 双轨制 / RGS-WBS-001 v0.3 §2A 145 L4 任务 / RGS-OLU-REPORT-2026-08-27 v0.1 §3 token 估算公式 / RGS-WEB-REQUIREMENTS-2026-08-26 v0.1（rgs-web 母需求）/ RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1 / RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1 |
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 文档定位

本文档是 rgs-web 内部**新增的 Token 可视化子系统**的需求规约层，描述"为什么做"和"做什么"——不涉及"怎么做"。按 RGS 项目三层文档规范（per RGS-DTL-001 设计模式：需求规约 / 基本设计 / 详细设计），本文档回答 5W1H 中的 **What + Why**。

**三层文档对应关系**：

| 层级 | 文档 | 回答 |
|---|---|---|
| 需求规约 | **本文档** | What + Why（用户痛点 + 业务目标 + 功能需求 + 非功能需求）|
| 基本设计 | RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1 | How 概要（架构 + 技术选型 + 模块划分 + 关键流程）|
| 详细设计 | RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1 | How 细节（API 签名 + 数据模型 + 部署 + 运维 + 安全）|

**与 rgs-web 母需求文档的关系**：

- 母需求 `RGS-WEB-REQUIREMENTS-2026-08-26 v0.1` 定义 rgs-web 整体定位（10 页面 dashboard + 6 域 gRPC + cluster-ops + 6 API + k3s 代理）
- 本文档**不重复**母需求已声明的通用 NFR（NFR-1 至 NFR-21 性能 / 可用性 / 安全性 / 可维护性 / 可移植性 全部继承）
- 本文档**只新增** Token 子系统特有的 FR（功能需求 F-1 至 F-23）+ NFR（NFR-22 至 NFR-28）+ 数据需求 DR-1 至 DR-6 + 集成需求 IR-1 至 IR-4

---

## 1. 背景与痛点

### 1.1 现状（per 2026-09-01 15:44 JST git 实证）

| 维度 | 现状 | 数据来源 |
|---|---|---|
| WBS L4 任务总数 | 145（per RGS-WBS-001 v0.3 + v0.4 L4 进度表）| `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md` §3 汇总行 |
| 已 done 任务 | 22 | 同上 |
| in_progress 任务 | 0 | 同上 |
| pending 任务 | 124 | 同上 |
| rgs-web 现状 | v0.3 真实 gRPC 接入，10 页面 dashboard | `tools/rgs-web/public/index.html` nav 行 98-110 |
| rgs-web 母规范 | 需求 + 基本 + 详细 + 计划 + GM 5 份文档 | `docs/12-工作流/RGS-WEB-*.md` |
| OLU token 框架 | 1 人·天 ≈ 100K-300K tokens（基线）| RGS-TS-001 v0.7 §6.2.2.1 |
| 上一次实际 token 估算 | 2026-08-27 dev k3s 部署：725K-1.45M tokens（conservative）/ 1.95M-3.68M（aggressive）| RGS-OLU-REPORT-2026-08-27 v0.1 §3.10 |
| Mavis AI 协作 | 主会话 + explore/worker/verifier/mavis 4 类内置子代理 + 自定义子代理 | mavis runtime |

### 1.2 用户画像（per user_profile DEC-008）

**Ulysses**（一人公司 12 角色 per DEC-008）：
- 角色：1 人 12 角色（架构师 / 5 域 Lead / SRE / DBA / 安全 / shared-platform / saga 召集人 / PM / ...）
- 工作流：WBS v0.3 145 L4 任务 + 5 域 IMPL-PLAN v0.2 + RACI v1.1
- 环境：Windows 11 + WSL2 Ubuntu + k3s + Rust + node 22
- AI 协作偏好（per 2026-08-21 user 反馈）：**用 token 而非人·天算 OLU**——"AI 在上下文窗口内可秒级生成数百行 Rust 代码；人类工作日含会议 / 上下文切换 / 决策等待等开销，人天单位在 AI 协作下失去精度"
- 文档代签偏好（per 2026-08-26 08:40 JST + 19:39 JST + 20:56 JST + 21:59 JST 三次强化）：Mavis 默认代签 Ulysses
- 拍板决策偏好（per 2026-09-01 14:58 JST）：必须用 ask_user 给选项

### 1.3 当前痛点

| # | 痛点 | 影响 | 频率 |
|---|---|---|---|
| 1 | **WBS 145 L4 任务没有 token 字段** | RGS-TS-001 v0.7 §6.2 OLU 框架已有"1 人·天 ≈ 100K-300K tokens"换算，但 WBS 表格只记 status/progress/owner，无 budget_tokens / actual_tokens，无法在任务维度看 token 消耗 | 每次 PM 进度盘点 |
| 2 | **AI 协作 token 消耗不可见** | Mavis 主会话 + 子代理 + worker 实际 token 流没有 ledger；RGS-OLU-REPORT-2026-08-27 §3.2 用"会话时长 × 每分钟 AI 协作 token 流"近似估算，精度低 | 每次任务执行 |
| 3 | **GitHub/GitLab issue 与 WBS 任务脱钩** | rgs-web 当前零外部集成；145 L4 任务若用 GitHub issue 跟踪，需手动去 GitHub 看 issue 状态，issue 评论里没有 token 实际消耗 | 每次 DDD Review |
| 4 | **5 域 Lead token 分摊不可比** | 5 域独立 Lead（per DEC-005 兼任拒绝），但 token 实际消耗没按域拆分，无法判断哪个域 OLU 偏高 | 每周 |
| 5 | **NFR-OP-010 双轨硬约束缺仪表盘** | RGS-TS-001 v0.7 §6.2.4：NFR-OP-010 双轨上限 = 20 人·天/周 = 20M tokens/周。当前 OLU 报告是事后 markdown 表格，不是实时仪表盘 | 每次部署后 |
| 6 | **多 AI 工具 token 无法聚合** | 5 域 binary 自身暂未调外部 LLM，但 Mavis 主代理 + 子代理 + worker 的 token 流需要统一记账 | 跨会话 |

### 1.4 业务目标

| # | 目标 | 度量 |
|---|---|---|
| O-1 | **Token 实时可见** | 145 L4 任务每条都能看到 budget_tokens / actual_tokens / percent_used 三个字段，仪表盘 30s 自动 refresh |
| O-2 | **任务卡 + Gantt + Token 三联动** | 点 Gantt 任务条 → 弹出 Token 详情卡；点 Token 卡片 → 跳转 WBS 任务源文件；点 GitHub issue → 跳转 rgs-web Token 视图 |
| O-3 | **AI 协作 token 自动入账** | Mavis 主会话 + 子代理 + worker 任务结束时，token 流自动写入 ledger；不依赖手工登记 |
| O-4 | **GitHub/GitLab 浅联动 v0.1** | rgs-web /api/git/integrations/{github,gitlab} 拉取 issue 列表 + commit 关联表，只读；issue 加 `token-budget` 标签双向桥接（rgs-web 写回 issue 评论区 bot 身份，per ask_user 推荐项）|
| O-5 | **NFR-OP-010 双轨实时仪表盘** | 仪表盘顶部固定 token 计数器（本周 / 今日 / 任务级），与 20M tokens/周硬约束做绿/黄/红三态告警 |
| O-6 | **5 域 Lead token 分摊可视化** | 仪表盘按域聚合（player / economy / match / social / admin），与 RACI v1.1 5 域 Lead 签字栏并列显示 |

---

## 2. 用户故事（User Stories）

### 2.1 US-1 Gantt + Token 选项卡总览

> **作为** Ulysses
> **我想要** 在 rgs-web 现有 10 页面 dashboard 旁加一个"📊 Gantt"页面，页面内嵌 4 个选项卡（Gantt / Tasks / Token / AI）
> **以便于** 1 个页面内看完 145 L4 任务的进度 + token + AI 协作情况，不切工具

**验收标准**：
- [ ] 新增 `page-gantt`，nav 增加"📊 Gantt"按钮
- [ ] 页面内 4 个选项卡：Gantt（时间线甘特图）/ Tasks（WBS 145 L4 列表）/ **Token（token 消耗图表，**本文档主线**）** / AI（AI 协作 ledger）
- [ ] 4 选项卡共用 task_id 选中态，点 Gantt 条 → 切到 Tasks 卡 → 切到 Token 卡，token 详情同步
- [ ] 单页加载 < 3s

### 2.2 US-2 Token 选项卡预算 vs 实际双柱图

> **作为** Ulysses
> **我想要** 在 Token 选项卡看每条 L4 任务的 budget_tokens（预算）和 actual_tokens（实际），按域聚合
> **以便于** 快速判断哪个任务 / 哪个域超预算

**验收标准**：
- [ ] 145 L4 任务每条都有 budget_tokens 字段（per RGS-TS-001 v0.7 §6.2.2.1 自动推算：人·天 × 100K-300K tokens）
- [ ] actual_tokens 字段从 ledger 读（per §6.1 数据源 DR-2）
- [ ] 图表：水平双柱图（bar chart），左柱 budget，右柱 actual，超预算 actual 标红
- [ ] 顶部下拉框按域筛选（player / economy / match / social / admin / shared-platform / cluster-ops / 所有）
- [ ] 顶部 NFR-OP-010 计数器：本周 tokens / 20M tokens = 百分比，绿/黄/红三态

### 2.3 US-3 任务卡 ↔ Token 双向跳转

> **作为** Ulysses
> **我想要** 点 Gantt 任务条 → 弹出 token 详情卡；点 token 详情卡 → 跳转到 WBS L4 进度表源文件对应行
> **以便于** 不离开页面就能追溯

**验收标准**：
- [ ] Gantt 任务条 onClick → 切到 Token 选项卡，自动选中该任务
- [ ] Token 选项卡 task 行 onClick → 弹窗显示：budget / actual / percent / 来源会话（mvs_xxxx）/ 子代理 / commit hash
- [ ] 弹窗"查看源文件"按钮 → 跳转到 `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.X.md` 对应行（锚点）
- [ ] 反向：WBS 进度表点 task_id → 唤起 rgs-web 对应任务 token 视图（per deep link `http://127.0.0.1:8788/?page=gantt&task_id=WF-1-55.27&tab=token`）

### 2.4 US-4 GitHub / GitLab 浅联动

> **作为** Ulysses
> **我想要** 在 Token 选项卡关联 GitHub/GitLab issue / MR（v0.1 只读 + 写回 issue 评论）
> **以便于** 跨平台看 token 实际消耗

**验收标准**：
- [ ] `GET /api/git/integrations` 返回当前配置的 GitHub / GitLab 仓库 + 鉴权状态（env var 不暴露值，per 2026-08-27 11:06 JST env value 硬 ban）
- [ ] `GET /api/git/issues?repo=xxx&labels=token-budget` 拉取 issue 列表（含编号 / 标题 / 状态 / 标签 / 创建时间）
- [ ] Token 选项卡每条 L4 任务行有"🔗 GitHub"按钮，点击后弹窗显示关联 issue 列表 + 当前 task 的 token 实际消耗（v0.1 写回 issue 评论，bot 身份："rgs-oludash-bot"）
- [ ] 反向：issue 加 `token-budget` 标签 → rgs-web 拉取时识别为 token 跟踪 issue

### 2.5 US-5 AI 协作 token 自动入账

> **作为** Ulysses
> **我想要** Mavis 主会话 + 子代理任务结束自动写 token 到 ledger，rgs-web 仪表盘 30s 内可见
> **以便于** 不手工登记 token

**验收标准**：
- [ ] Mavis runtime 集成：每次 `session finish` 事件触发 `rgs-oludash-hook`（mavis 自定义 hook），把 session_id / agent_name / task_id / token_in / token_out / started_at / finished_at 追加到 `data/ai-ledger.jsonl`
- [ ] rgs-web 30s 轮询 `/api/token/ai-ledger` → 仪表盘 AI 选项卡显示
- [ ] 估算方法（v0.1）：token = message_count × 5K tokens/条（per RGS-OLU-REPORT-2026-08-27 v0.1 §3.2 公式，标注 "estimated"）
- [ ] v0.2 升级：读 provider token counter（OpenAI / Anthropic / 自建 gateway 真实值），取消估算

### 2.6 US-6 NFR-OP-010 双轨告警

> **作为** Ulysses
> **我想要** 仪表盘顶部固定显示本周 tokens / 20M tokens 比例，绿/黄/红三态
> **以便于** 不超 NFR-OP-010 硬约束

**验收标准**：
- [ ] 顶部固定条：本周 tokens = 实时聚合（来自 ai-ledger.jsonl + git-ledger.jsonl）
- [ ] 比例 ≤ 70% 绿 / 70-90% 黄 / > 90% 红
- [ ] 红态触发 mavis cron `nfr-op-010-watchdog` 自检 + 通知（"本周 NFR-OP-010 超 90%，请暂停 AI 协作或申请额外 SRE 编制"）
- [ ] 双轨并列显示：人·天轨 = 145 L4 任务 Σ / 5 人·周；token 轨 = Σ token / 20M tokens/周

### 2.7 US-7 5 域 Lead token 分摊

> **作为** Ulysses
> **我想要** 仪表盘按 5 域（player / economy / match / social / admin）+ 域簇（shared-platform / cluster-ops / saga）聚合 token 实际消耗
> **以便于** 评估 5 域 Lead 实际 OLU

**验收标准**：
- [ ] 仪表盘按域堆叠图（stacked bar）：横轴 = 域，纵轴 = tokens
- [ ] 每域颜色对应 rgs-web 现有颜色方案（player = blue / economy = green / match = orange / social = purple / admin = red / shared-platform = cyan / cluster-ops = yellow）
- [ ] 鼠标 hover 显示该域任务数 / done 数 / token 合计 / percent_used 95% 分位数
- [ ] 与 RACI v1.1 5 域 Lead 签字栏并列（5 域 Lead = 5 域 token 负责人，per DEC-008 一人公司 12 角色代签基线）

---

## 3. 功能需求（Functional Requirements）

### 3.1 必备 v0.1（Must Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-1 | 新增 Gantt 页面 | `page-gantt`，nav 第 11 项；4 选项卡（Gantt / Tasks / Token / AI）| P0 |
| F-2 | Gantt 视图 | 基于 RGS-WBS-001 v0.3 §2A 145 L4 任务，按 L1 阶段（WF-0 ~ WF-7）分组，水平时间线，里程碑三角号 | P0 |
| F-3 | Tasks 选项卡 | 145 L4 任务表格（task_id / 摘要 / owner / status / progress / budget_tokens / actual_tokens / 关联 issue）| P0 |
| F-4 | **Token 选项卡** | **本文档主线**：双柱图（budget vs actual）+ 域聚合堆叠图 + 95% 分位线 + 异常任务列表 | **P0** |
| F-5 | AI 选项卡 | Mavis 主会话 + 子代理 ledger 表格 + 估算公式标注 | P0 |
| F-6 | Token 预算字段 | 145 L4 任务每条加 `budget_tokens`（推算公式：人·天 × 100K-300K tokens 中位数 200K）| P0 |
| F-7 | Token 实际字段 | `actual_tokens` 字段，从 ai-ledger.jsonl + git-ledger.jsonl 聚合 | P0 |
| F-8 | ai-ledger.jsonl 写入 | Mavis runtime hook 每次 session finish 追加一行（session_id / agent_name / task_id / tokens_in / tokens_out / started_at / finished_at）| P0 |
| F-9 | git-ledger.jsonl 写入 | rgs-web 写脚本监听 git log，每 30s 把新 commit 关联到 task_id（per `.wbs-task-marker` worktree 内 JSON 文件）| P0 |
| F-10 | NFR-OP-010 实时计数器 | 顶部固定条，本周 tokens / 20M tokens 比例 + 三态色 | P0 |
| F-11 | 5 域分摊视图 | 按域聚合堆叠图，颜色 per rgs-web 现有方案 | P0 |
| F-12 | 30s 自动 refresh | 仪表盘 30s 轮询 /api/token/summary | P0 |
| F-13 | 深链支持 | URL `?page=gantt&task_id=WF-1-55.27&tab=token` 直接打开 | P0 |
| F-14 | 中文路径支持 | 读 WBS L4 进度表（含中文）正确处理 | P0 |
| F-15 | 端口可配 | `RGS_WEB_PORT` 环境变量改默认 8788 | P0 |
| F-16 | 127.0.0.1 only 监听 | 不暴露 0.0.0.0（一人公司本地工具）| P0 |

### 3.2 重要 v0.1（Should Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-17 | GitHub 浅联动 | `GET /api/git/integrations` + `GET /api/git/issues?repo=xxx&labels=token-budget` 只读拉取 | P1 |
| F-18 | GitHub 写回 issue 评论 | rgs-web 主动把 token 实际消耗以 bot 身份写回 issue 评论，**需要 GitHub PAT** 凭据（env var 注入，不落盘 per 2026-08-27 11:06 JST env value hard ban）| P1 |
| F-19 | GitLab 浅联动 | 同 F-17/F-18 范式 | P1 |
| F-20 | 任务卡 ↔ Token 双向跳转 | 弹窗显示 token 详情 + 跳转 WBS 源文件锚点 | P1 |
| F-21 | 95% 分位线 | 双柱图叠加 95% 分位线（异常任务识别）| P1 |
| F-22 | 异常任务列表 | percent_used > 95% 的任务红名单 | P1 |
| F-23 | 任务时间线 | Gantt 视图按 L1 阶段分组，里程碑标三角号 | P1 |

### 3.3 可选 v0.2（Could Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-24 | WebSocket 实时推送 | 替代 30s 轮询（v0.1 轮询够用）| P2 |
| F-25 | token 估算 vs 真实切换 | v0.1 估算（message_count × 5K），v0.2 读 provider counter 真实值 | P2 |
| F-26 | 5 域 binary 自身调外部 LLM 的 token | 5 域 Rust gRPC 暂未调外部 LLM，预留 hook | P2 |
| F-27 | token 历史趋势 | 本周 vs 上周 vs 上月 token 消耗折线图 | P2 |
| F-28 | NFR-OP-010 自检通知 | 红态触发 mavis cron `nfr-op-010-watchdog` | P2 |
| F-29 | GitLab 深联动（webhook）| v0.2 加 webhook inbound；v0.1 暂不做（per ask_user 推荐项 + per 2026-08-27 user_profile 127.0.0.1 only 硬约束冲突）| P2 |
| F-30 | dark/light theme 切换 | v0.1 dark only（per rgs-web 母规范 v0.3 现状）| P2 |

### 3.4 不做（Won't Have）

| # | 需求 | 不做理由 |
|---|---|---|
| F-W1 | 用户登录 / RBAC | 一人公司模式（per DEC-008），rgs-web 母规范已声明 |
| F-W2 | 跨服务事务编排 | 5 域 saga 走 gRPC，Web UI 只读 |
| F-W3 | token 数据 push 到 GitHub issue 列表 | v0.1 只写 issue 评论，issue 列表更新由 GitHub 自身管理 |
| F-W4 | env value 打印 | 2026-08-27 11:06 JST 硬 ban，所有 token / 凭据 env var 注入不输出 |
| F-W5 | 5 域 binary 运行时调外部 LLM | 5 域 Rust gRPC 当前不调外部 LLM（per Cargo.toml + rgs-web 母规范），预留 F-26 hook 不实现 |

---

## 4. 数据需求（Data Requirements）

| # | 需求 | 数据源 | 存储 | 写者 |
|---|---|---|---|---|
| DR-1 | WBS L4 任务 budget_tokens | 145 L4 任务 × 人·天 × 200K（中位数）| RGS-WBS-001 L4 进度表加字段 | rgs-web 启动时读 |
| DR-2 | AI 协作 ledger | Mavis session list + message count | `data/ai-ledger.jsonl`（append-only）| mavis runtime hook（写）+ rgs-web 读 |
| DR-3 | Git 协作 ledger | `git log --pretty=format:...` + `.wbs-task-marker` 关联 | `data/git-ledger.jsonl`（append-only）| rgs-web 后台轮询脚本（30s）|
| DR-4 | GitHub issue 缓存 | `GET /api/git/issues` 拉取 | `data/github-cache.json`（10 min TTL）| rgs-web API handler |
| DR-5 | GitLab issue 缓存 | 同 DR-4 | `data/gitlab-cache.json` | 同 |
| DR-6 | NFR-OP-010 计数器 | ai-ledger + git-ledger 聚合 | 内存（不落盘，每次启动重算）| rgs-web |

**存储位置**：所有 jsonl / json 落在 `tools/rgs-web/data/`（per rgs-web 母规范 v0.3 零依赖 + 1 写者约束）

**锁文件**：DR-2 / DR-3 / DR-4 / DR-5 写并发控制用 `data/.lock`（per ask_user 推荐项）

---

## 5. 集成需求（Integration Requirements）

| # | 集成 | 描述 | v0.1 | v0.2 |
|---|---|---|---|---|
| IR-1 | Mavis runtime | 每次 session finish 触发 hook 写 ai-ledger.jsonl | ✅ | ✅ |
| IR-2 | mavis agent list | `mavis agent list` 拉取 agent roster，作为 AI 选项卡下拉框 | ✅ | ✅ |
| IR-3 | mavis session list | `mavis session list` 拉取 session 历史，作为 AI 选项卡表格 | ✅ | ✅ |
| IR-4 | git log + .wbs-task-marker | 30s 轮询新 commit 关联 task_id 写 git-ledger.jsonl | ✅ | ✅ |
| IR-5 | GitHub API | `GET /repos/{owner}/{repo}/issues?labels=token-budget` | ✅（只读）| ✅（写回评论）|
| IR-6 | GitLab API | `GET /api/v4/projects/{id}/issues?labels=token-budget` | ✅（只读）| ✅（写回评论）|
| IR-7 | provider token counter | OpenAI / Anthropic / 自建 gateway 真实 token | ❌ | ✅ |
| IR-8 | mavis cron | 红态自检 + 通知 | ❌ | ✅ |

---

## 6. 非功能需求（Non-Functional Requirements）

> **以下 NFR 全部继承 rgs-web 母规范 v0.1 NFR-1 至 NFR-21，不再重复**。本节只列 Token 子系统**新增**的 NFR。

| # | 指标 | 目标 | 备注 |
|---|---|---|---|
| NFR-22 | Gantt 页面加载 | < 3s | 145 L4 任务 + 4 选项卡，127.0.0.1 本地 |
| NFR-23 | Token 选项卡双柱图渲染 | < 500ms | 145 任务 SVG 渲染 |
| NFR-24 | ai-ledger.jsonl 单文件大小 | < 50MB | 超出滚动归档 `data/ai-ledger-YYYY-MM.jsonl` |
| NFR-25 | 30s 内 ledger 新增行可见 | ≤ 30s | 30s 轮询上限 |
| NFR-26 | GitHub/GitLab 缓存 TTL | 10 min | 避免触发 GitHub rate limit（per 2026-08-27 11:06 JST 凭据保护）|
| NFR-27 | token 估算公式标注 | 强制 | UI 上每个 token 数字必须标 "estimated" 或 "real"，per RGS-OLU-REPORT-2026-08-27 v0.1 §3.2 + §10 GAP-1/2/3 |
| NFR-28 | env value 不打印 | 强制 | per 2026-08-27 11:06 JST 硬 ban，rgs-web 日志 / 响应 / 错误信息都不暴露 K3S_TOKEN / GITHUB_TOKEN / GITLAB_TOKEN 等凭据值 |

---

## 7. 约束（Constraints）

### 7.1 治理约束

- per DEC-008：一公司 12 角色，无 RBAC，Web UI 是 1 人工具
- per DEC-005：5 域独立 Lead 兼任禁止（5 域 token 视图需保留域拆分）
- per 2026-08-26 08:40 JST：Mavis 默认代签 Ulysses（修订历史"审批者"列）
- per 2026-08-27 11:06 JST：env value 打印硬 ban
- per 2026-08-26 04:30 JST：禁"per X 历史形态"回溯叙事，引用 RGS-OLU-REPORT / RGS-WBS / RGS-TS 必须 git log --follow 实证
- per 2026-09-01 14:58 JST：拍板决策必须用 ask_user 给选项

### 7.2 技术约束

- 沿用 rgs-web 母规范 v0.1 零依赖选型（node + 原生 http，per RGS-WEB-PLAN v0.1 §2）
- 不引入 npm 依赖（per rgs-web 母规范 v0.1 §2 选型，实测 npm install 2 分钟+）
- 不引入 SQLite（per ask_user 推荐项，JSON 文件 + lockfile 足够）
- 反向代理 / 边缘层：envoy 独立 deployment（per 2026-09-01 13:03 / 13:05 JST），rgs-web 当前是 node http 直起，**不**用 nginx
- WSL2 + kubectl port-forward 依赖 6 域 gRPC 通（per rgs-web v0.3 现状 15051-15056）
- 145 L4 任务的 token 预算推算公式 = `人·天 × 200K tokens`（中位数 per RGS-TS-001 v0.7 §6.2.2.1）

### 7.3 时间约束

- v0.1 4 周内落地（per Ulysses 期望"快"，参考 rgs-web 母规范 v0.1 30 分钟落地但本文档是 v0.1 全量设计）
- v0.2 provider counter 真实值接入，6 周内
- v0.3 webhook 深联动（GitHub / GitLab），待 user_profile 127.0.0.1 only 硬约束讨论后定

---

## 8. 验收标准（Acceptance Criteria）

### 8.1 v0.1 验收

- [ ] `node tools/rgs-web/server.js` 启动 < 1s（沿用 rgs-web 母规范）
- [ ] `http://127.0.0.1:8788/` 新增"📊 Gantt"nav 项
- [ ] 4 选项卡（Gantt / Tasks / Token / AI）切换正常
- [ ] Gantt 视图 145 L4 任务按 L1 阶段分组渲染
- [ ] Tasks 表格 145 行完整，含 budget_tokens / actual_tokens
- [ ] **Token 选项卡双柱图渲染 145 L4 任务的 budget vs actual**（**本文档主线**）
- [ ] **顶部 NFR-OP-010 计数器：本周 tokens / 20M tokens，绿/黄/红三态正确**
- [ ] **5 域分摊堆叠图：player / economy / match / social / admin + shared-platform / cluster-ops / saga，颜色对应**
- [ ] AI 选项卡 ai-ledger.jsonl 表格渲染
- [ ] 30s 自动 refresh 生效
- [ ] 深链 `?page=gantt&task_id=WF-1-55.27&tab=token` 正确打开
- [ ] Mavis session finish 触发 ai-ledger.jsonl 追加（per mavis runtime hook）

### 8.2 v0.2 验收

- [ ] `GET /api/git/integrations` 返回 GitHub / GitLab 配置状态
- [ ] `GET /api/git/issues?repo=xxx&labels=token-budget` 拉取 issue 列表
- [ ] Token 选项卡每行"🔗 GitHub"按钮弹窗显示关联 issue
- [ ] rgs-web 写回 issue 评论（bot 身份："rgs-oludash-bot"）
- [ ] provider token counter 接入（OpenAI / Anthropic / 自建 gateway）
- [ ] 估算值 vs 真实值切换开关

### 8.3 v0.3 验收

- [ ] GitHub / GitLab webhook inbound
- [ ] NFR-OP-010 红态自检 + 通知
- [ ] token 历史趋势折线图

---

## 9. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| mavis runtime hook 集成阻塞 | 中 | AI 选项卡无可用数据 | v0.1 先用 `mavis session list --json` 拉历史 + 估算，hook 集成放 v0.2 |
| GitHub rate limit | 中 | 写回 issue 评论 403 | 10 min 缓存（per NFR-26）+ 退避重试 + User-Agent 标识 rgs-oludash |
| 145 L4 任务 budget_tokens 推算偏差 | 高 | 仪表盘读数误导 | UI 强制标 "estimated"（per NFR-27），推算公式公开（per RGS-TS-001 v0.7 §6.2.2.1），RGS-ENV-CALIB-001 校准后回填 |
| 5 域 binary 未来调外部 LLM 未登记 | 中 | token 漏算 | 预留 F-26 hook，5 域 Cargo.toml 加 `rgs-otel` 出口点 |
| 中文路径 mojibake | 低 | 读 WBS 文件失败 | 沿用 rgs-web 母规范 v0.1 §NFR-9，node 默认 UTF-8 |
| npm install 超时（v0.1 母规范 v0.1 §7 历史教训）| 高 | 启动慢 | v0.1 零依赖（per §7.2），不引新依赖 |
| 凭据泄露 | 中 | GitHub / GitLab 私仓暴露 | 2026-08-27 11:06 JST 硬 ban + rgs-web 响应 / 日志脱敏（NFR-28）|
| user_profile 127.0.0.1 only 与 webhook 冲突 | 高 | webhook 不可达 | v0.3 才做 webhook（per 7.3 + F-W3），v0.1/v0.2 仅 outbound（拉取 + 写回评论）|
| Gantt SVG 渲染 145 任务性能 | 中 | 页面卡顿 | 145 任务单页 < 3s（per NFR-22），分页 / 虚拟滚动备选 |

---

## 10. 不在范围（Out of Scope）

- 5 域 binary 自身 token 出口（per F-W5 + IR-7 推迟 v0.2）
- 多用户 / RBAC / SSO（一人公司模式）
- WBS v0.3 §2A 之外的新增 L4 任务
- 真实 SSH 到 WSL 内（沿用 rgs-web 母规范 v0.1 §8）
- 部署到 k3s 作为 deployment + service（rgs-web 母规范 v0.3 §6.3 推迟 v1.0）
- AI 协作 token 写到 Git commit message / GitHub issue 标题（v0.1 只写 issue 评论）
- 5 域 Lead 真实签字（per RGS-WBS-001 L4 进度表 v0.4 §A.3 + DEC-005 兼任拒绝，5 域 token 视图用 Ulysses 代签基线展示）

---

## 11. 验收者（per 2026-08-26 08:40 JST 代签新规则）

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | 架构师（**Mavis 接手 agent per DEC-008**）| 2026-09-01 |
| 5 域 Lead | _待 DDD Review 阶段补签_ | — |
| shared-platform Lead | _待 DDD Review 阶段补签_ | — |
| cluster-ops Lead | _待 DDD Review 阶段补签_ | — |
| SRE Lead | _待 DDD Review 阶段补签_ | — |
| PM | _待 DDD Review 阶段补签_ | — |

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师（**Mavis 接手 agent per DEC-008**）| 首版：6 用户故事 + 30 FR + 6 DR + 8 IR + 7 NFR（新增）+ 6 约束 + 9 风险 + 3 验收阶段 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态：rgs-web v0.3 仅有 10 页面 dashboard，无 Gantt / Token / AI 子系统
- v0.1 新增：本文档（6 用户故事 + 30 FR + 6 DR + 8 IR + 7 NFR + 6 约束 + 9 风险 + 3 验收阶段）
- v0.1 新增子系统：Gantt 页面 + 4 选项卡 + Token 预算 vs 实际双柱图 + 5 域分摊 + NFR-OP-010 实时计数器 + ai-ledger.jsonl 写入 + GitHub/GitLab 浅联动

### A.2 对基本设计的影响

- 触发 RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1 起草
- 6 用户故事 → 6 核心模块（gantt / tasks / token / ai / integrations / nfr-op-010-watchdog）
- 30 FR → 30 API endpoint / 路由设计
- 6 DR → 6 数据文件 + 1 lockfile（`tools/rgs-web/data/`）
- 8 IR → 8 集成接口（mavis / git / GitHub / GitLab / provider / cron）

### A.3 已知缺口

- 5 域 Lead / shared-platform / cluster-ops / SRE / PM 签字未到（DDD Review 阶段补）
- 6.2 v0.2 验收依赖 GitHub PAT / GitLab PAT 凭据（Ulysses 手动注入 env var）
- 6.3 v0.3 验收依赖 webhook inbound（与 user_profile 127.0.0.1 only 硬约束冲突，待 Ulysses 拍板）
- mavis runtime hook 集成未确认（per §9 风险 1，v0.1 降级为 session list 拉历史 + 估算）
- provider token counter 接入路径未确认（per §9 风险 4，v0.2 待 v0.1 校准后定）

### A.4 引用链与证据

- rgs-web v0.3 commit `23d447b`（per RGS-WBS-001 L4 进度表 v0.4 §A.3 git 实证）
- RGS-TS-001 v0.7 §6.2 OLU 双轨制（per `git log -p --follow docs/10-技术选型/RGS-TS-001_*.md` 实证 v0.5/v0.6/v0.7 演化）
- RGS-WBS-001 v0.3 §2A 145 L4 任务（per `docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md`）
- RGS-OLU-REPORT-2026-08-27 v0.1 §3 token 估算公式（per `docs/14-项目管理/RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy_v0.1.md`）
- RGS-WEB-REQUIREMENTS-2026-08-26 v0.1（per `docs/12-工作流/RGS-WEB-REQUIREMENTS-2026-08-26_v0.1.md`）
- per DEC-008 一人公司 12 角色
- per 2026-08-26 08:40 JST Mavis 默认代签 Ulysses
- per 2026-08-27 11:06 JST env value hard ban
- per 2026-09-01 13:03 / 13:05 JST envoy 独立 deployment 偏好
- per 2026-09-01 14:58 JST 拍板决策必须用选项
