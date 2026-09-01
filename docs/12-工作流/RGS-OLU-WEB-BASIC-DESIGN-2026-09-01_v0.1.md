# RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1

**Token 消耗可视化子系统基本设计（rgs-web Gantt + Token 选项卡）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 15:44 JST 触发）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1 需求已落地，本层补基本设计 |
| 关联 | RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1（上游）+ RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1（下游）+ RGS-OLU-WEB-PLAN-2026-09-01 v0.1（总览）+ rgs-web 母规范 5 份 |
| 上游母规范 | RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1 §1-§5 通用架构 / §2 技术选型 / §3 模块划分 / §4 关键流程 / §5 数据模型 |
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 文档定位

本文档是 rgs-web Token 子系统**基本设计层**，回答"How 概要" — 不涉及"What + Why"（已 in REQUIREMENTS）也不涉及"How 细节"（在 DETAILED-DESIGN）。

按 RGS 项目规范（per RGS-DTL-001 设计模式）：需求 → 基本 → 详细。

**继承母规范**：rgs-web 母规范 `RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1` §1 部署架构 / §2 技术选型 / §3 模块划分 / §4 关键流程 / §5 数据模型 **全部继承**，本文档只补充 Token 子系统**新增**内容。

---

## 1. 架构总览

### 1.1 子系统在 rgs-web 中的位置

```
rgs-web (node 22, 127.0.0.1:8788, PID)
├── 母规范模块（10 页面 + 6 API + k3s 代理）
│   ├── Dashboard / Servers / Players / Live Console / Config / Hot Update / Operations SQL / Docs & Health / Worktrees / Reports
│   └── /api/health / /api/impl-plan / /api/worktrees / /api/docs-health / /api/k8s/*
│
└── ★ 本文档新增子系统（1 页面 + 4 选项卡 + 9 API + 5 数据文件）★
    ├── page-gantt（11 号页面，nav 第 11 项）
    │   ├── 选项卡 1: Gantt（Gantt 视图，按 L1 阶段分组的水平时间线）
    │   ├── 选项卡 2: Tasks（145 L4 任务表格）
    │   ├── 选项卡 3: Token（本文档主线：双柱图 + 5 域堆叠 + NFR-OP-010 计数器）★ 核心
    │   └── 选项卡 4: AI（mavis session + ai-ledger.jsonl 表格）
    │
    ├── /api/token/* (4 endpoint)
    │   ├── /api/token/summary          本周/今日/任务级 token 聚合
    │   ├── /api/token/budget-vs-actual 双柱图数据
    │   ├── /api/token/by-domain        5 域分摊
    │   └── /api/token/nfr-op-010       20M tokens/周计数器
    │
    ├── /api/ai/* (3 endpoint)
    │   ├── /api/ai/ledger              ai-ledger.jsonl 读取
    │   ├── /api/ai/sessions            mavis session list 代理
    │   └── /api/ai/agents              mavis agent list 代理
    │
    └── /api/git/integrations/* (2 endpoint)
        ├── /api/git/integrations       配置状态
        └── /api/git/issues             GitHub / GitLab issue 拉取
```

### 1.2 数据流（Token 选项卡）

```
Browser  GET /api/token/budget-vs-actual
  → rgs-web (node, port 8788)
  ├── 读 data/ai-ledger.jsonl (Mavis session finish 写入)
  ├── 读 data/git-ledger.jsonl (rgs-web 后台 30s 轮询 git log 写入)
  ├── 读 docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.X.md (145 L4 任务 + budget_tokens 推算)
  ├── 内存聚合: budget vs actual per task
  └── JSON 返 Browser SVG 渲染

Browser  GET /api/token/nfr-op-010
  → rgs-web
  ├── Σ ai-ledger + git-ledger 本周 tokens
  ├── 20M tokens/周 比例
  └── JSON 返 Browser 顶部条三态色

Mavis runtime session finish
  → mavis hook
  ├── 追加一行到 data/ai-ledger.jsonl (lockfile data/.lock)
  └── 30s 后 rgs-web 拉取可见
```

### 1.3 与外部系统的关系

| 外部系统 | 关系 | v0.1 数据流 | 触发点 |
|---|---|---|---|
| mavis runtime (主会话 + 子代理) | 写 ai-ledger.jsonl | session finish → hook 追加 | mavis hook |
| git (RustGameServer 仓库) | 读 .wbs-task-marker + git log | rgs-web 后台 30s 轮询 | setInterval |
| GitHub API | 拉取 + 写回 issue 评论 | `/api/git/issues` GET / POST | 用户点 Token 选项卡"🔗 GitHub"按钮 |
| GitLab API | 同 GitHub | 同 | 同 |
| provider token counter | v0.2 接入 | OpenAI / Anthropic / 自建 gateway | v0.2 |
| mavis cron | v0.2 自检 + 通知 | `nfr-op-010-watchdog` 红态触发 | v0.2 |

---

## 2. 技术选型

> 母规范 §2 决策表**全部继承**。本节只列 Token 子系统**新增**选型。

| 维度 | 选择 | 备选 | 决策理由 |
|---|---|---|---|
| Gantt 渲染 | **原生 SVG（手写）+ CSS 定位** | dhtmlx-gantt / frappe-gantt | 零依赖，145 任务足够；备选需 npm install（per rgs-web 母规范 §2.2 不选）|
| Token 双柱图 | **原生 SVG bar + 数字** | chart.js / d3 | 沿用 rgs-web 母规范 §2.1 决策"原生 CSS bar + 数字" |
| 数据存储 | **JSON 文件 + jsonl append-only + lockfile** | SQLite / better-sqlite3 / markdown frontmatter | per ask_user 推荐项 + 零依赖 + 1 写者约束；锁文件 `data/.lock` 用 `fs.openSync(path, 'wx')` 原子创建 |
| Token 估算（v0.1）| **message_count × 5K tokens/条** | provider counter 真实值 | v0.1 估算公式 per RGS-OLU-REPORT-2026-08-27 v0.1 §3.2；v0.2 切真实值 |
| GitHub / GitLab 客户端 | **node:https 原生 GET/POST** | octokit / @gitbeaker/node | 零依赖；GitHub API 简单 GET 不需要 octokit；POST 评论用 https.request |
| 凭据注入 | **env var only（K3S_TOKEN / GITHUB_TOKEN / GITLAB_TOKEN）** | 配置文件 / 凭据文件 | per 2026-08-27 11:06 JST env value hard ban，rgs-web 启动时 `process.env.X` 引用，**不**打印值 |
| mavis runtime hook | **mavis skill / plugin (per mavis skill 文档)** | 子进程 | per mavis skill §"hook management"，hook 入口标准 mavis 集成 |
| 数据刷新 | **30s 轮询（沿用母规范 §2.1）** | WebSocket / SSE | 单人使用，30s 足够 |

### 2.1 不选用的方案

| 方案 | 不选理由 |
|---|---|
| **better-sqlite3** | native binding 编译慢（per ask_user 推荐项不选），JSON + jsonl 够用 |
| **markdown frontmatter 存 token** | 解析成本高，不能聚合查询（per ask_user 不选）|
| **chart.js / d3** | 145 任务 SVG 手写足够，引入 npm 依赖违反母规范 §2.2 |
| **WebSocket 实时推送** | 母规范 §2.1 决策 30s 轮询够用，v0.2 再加 |
| **Docker 部署** | 一人公司本机工具，无意义 |
| **登录 / RBAC** | DEC-008 一人公司，127.0.0.1 only 足够安全 |
| **Rust axum 重写** | 母规范 §2.2 决策 v1.0 才做 |
| **nginx 反向代理** | per 2026-09-01 13:03 JST + 13:05 JST user_profile 偏好，边缘层用 envoy 独立 deployment；rgs-web 当前直起 node http 不经反向代理 |
| **mavis runtime hook 阻塞等待** | v0.1 降级为 mavis session list 拉历史 + 估算公式（per REQUIREMENTS §9 风险 1）|

---

## 3. 模块划分

### 3.1 新增模块清单

| # | 模块 | 文件位置 | 输入 | 输出 | 复用母规范 |
|---|---|---|---|---|---|
| 1 | page-gantt（容器） | `tools/rgs-web/public/index.html` 新增 `#page-gantt` + 4 选项卡 DOM | 无 | 4 tab 容器 | 沿用 nav / 路由机制（母规范 §4.1）|
| 2 | Token 选项卡 | 同上 `#tab-token` DOM + JS `loadTokenTab()` | `/api/token/*` | 双柱图 + 5 域堆叠 + NFR 计数器 | 沿用 30s 轮询 |
| 3 | Gantt 视图 | 同上 `#tab-gantt` DOM + JS `loadGanttTab()` | `/api/wbs/tasks` | SVG 时间线 | 沿用 |
| 4 | Tasks 选项卡 | 同上 `#tab-tasks` DOM + JS `loadTasksTab()` | 同上 | 145 任务表格 | 沿用 |
| 5 | AI 选项卡 | 同上 `#tab-ai` DOM + JS `loadAITab()` | `/api/ai/*` | session + ledger 表格 | 沿用 |
| 6 | 后台数据 API | `tools/rgs-web/server.js` 新增 9 endpoint | 4 数据文件 + mavis + git | JSON | 沿用母规范 §3.2 API 模式 |
| 7 | git-ledger 轮询器 | `tools/rgs-web/server.js` 内 `setInterval` 30s | git log + .wbs-task-marker | data/git-ledger.jsonl 追加 | 沿用 setInterval |
| 8 | 锁文件工具 | `tools/rgs-web/lib/lockfile.js`（新增）| fs.openSync(path, 'wx') | lock handle | 无（新增）|
| 9 | token 估算器 | `tools/rgs-web/lib/token-estimate.js`（新增）| session message count | estimated tokens | 无（新增）|

### 3.2 与 mavis 集成的接口

| mavis 命令 | 用途 | rgs-web 调用方式 |
|---|---|---|
| `mavis agent list --json` | 拉取 agent roster | `execSync('mavis agent list --json')` 30s 缓存 |
| `mavis session list --json` | 拉取 session 历史 | `execSync('mavis session list --json')` 30s 缓存 |
| `mavis session messages <id>` | 拉取单 session 消息计数 | `execSync(...)` 单次 |

> **v0.1 替代方案**：mavis runtime hook 集成阻塞时（per REQUIREMENTS §9 风险 1），降级为 mavis session list 拉历史 + token 估算公式（message_count × 5K）。v0.2 切真实 hook。

---

## 4. 关键流程

### 4.1 Mavis session finish → ai-ledger.jsonl 写入流程

```
Mavis runtime
  ↓ session finish 事件
mavis hook (per mavis skill §hook management)
  ├── 读 env: MAVIS_SESSION_ID, MAVIS_AGENT_NAME, MAVIS_TASK_ID, MAVIS_TOKENS_IN, MAVIS_TOKENS_OUT
  ├── 读 mavis session messages <id> (message_count, 估算补 fallback)
  ├── 计算 estimated_tokens = message_count × 5000
  ├── 加锁 data/.lock
  ├── 追加一行到 data/ai-ledger.jsonl:
  │   {"ts":"2026-09-01T15:00:00+09:00","session_id":"mvs_xxx","agent_name":"mavis","task_id":"WF-1-55.27",
  │    "tokens_in":0,"tokens_out":0,"estimated":true,"message_count":42,"estimated_tokens":210000}
  └── 释放锁

rgs-web 后台 30s 轮询:
  → 读 data/ai-ledger.jsonl
  → 聚合本会话 / 本任务 / 本域 / 本周 token
  → 暴露给 /api/token/*
```

### 4.2 git commit → git-ledger.jsonl 写入流程

```
rgs-web 后台 setInterval(30s)
  ├── execSync('git log --since=上次轮询 --pretty=format:%H|%ct|%s -- .wbs-task-marker <worktree>')
  ├── 解析每条 commit:
  │   - 读 commit 内 .wbs-task-marker (worktree 内 JSON)
  │   - 关联 task_id (L4 ID 如 WF-1-55.27)
  │   - 估算 commit token = 80K（per 行业经验：单 commit 含决策 + diff 产出 + 验证，per RGS-OLU-REPORT §3.5）
  ├── 加锁 data/.lock
  ├── 追加新 commit 到 data/git-ledger.jsonl:
  │   {"ts":"...","commit_hash":"<hash>","task_id":"WF-1-55.27","worktree":"<path>","estimated_tokens":80000,"source":"git-commit"}
  └── 释放锁
```

### 4.3 Token 选项卡渲染流程

```
Browser  GET /
  → 选 "📊 Gantt" nav
  → 切到 #page-gantt
  → 默认选 #tab-tasks
  → 用户点 #tab-token
  → loadTokenTab() 启动:
      ├── fetch('/api/token/nfr-op-010') → 顶部条
      ├── fetch('/api/token/budget-vs-actual') → 双柱图 SVG
      ├── fetch('/api/token/by-domain') → 5 域堆叠图
      └── 30s setInterval 刷新
  → 用户点 task_id 弹窗
  → loadTokenDetail(task_id) 启动:
      ├── fetch('/api/token/task-detail?task_id=...')
      ├── 弹窗显示: budget / actual / percent / sessions[] / commits[] / 关联 GitHub issue
      └── 用户点 "查看源文件" 按钮 → window.open('docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.X.md#<task_id>')
```

### 4.4 GitHub 写回 issue 评论流程

```
用户点 Token 选项卡 "🔗 GitHub" 按钮
  → 弹窗显示关联 issue 列表
  → 用户选 issue 编号
  → fetch('/api/git/post-comment', POST { issue: 123, task_id: 'WF-1-55.27' })
  → rgs-web:
      ├── 读 process.env.GITHUB_TOKEN (env value hard ban, 引用后直接用, 不打印)
      ├── 构造 POST https://api.github.com/repos/<owner>/<repo>/issues/<n>/comments
      ├── body: "🤖 rgs-oludash-bot\n\nTask: WF-1-55.27\nActual tokens: 245K / 300K (81%)\nDetail: http://127.0.0.1:8788/?page=gantt&task_id=WF-1-55.27&tab=token"
      ├── headers: { Authorization: 'token ${GITHUB_TOKEN}', User-Agent: 'rgs-oludash' }
      └── https.request 返 201/200 → 弹窗 "✅ 已写回 issue #123 评论"
  → 失败返 4xx/5xx → 弹窗错误（不暴露 token 值）
```

### 4.5 异常流程

| 异常 | 处理 | 备注 |
|---|---|---|
| mavis hook 未集成 | v0.1 降级：`mavis session list --json` 30s 拉历史 + 估算公式 | per REQUIREMENTS §9 风险 1 |
| data/.lock 占用（并发写）| retry 3 次（指数退避 100ms / 200ms / 400ms）| 1 写者约束（mavis hook + rgs-web 后台轮询）|
| ai-ledger.jsonl > 50MB | 自动滚动归档 `data/ai-ledger-YYYY-MM.jsonl` | per NFR-24 |
| WBS 145 L4 任务文件读取失败 | 返 500 + `hint: "WBS 文件可能被 worktree 锁住，per §8.3.4 RGS-WT-001"` | 沿用母规范异常处理 |
| GitHub rate limit 403 | 10 min 缓存 + 退避重试 + 顶部条黄态 | per NFR-26 |
| GITHUB_TOKEN 未注入 | 弹窗提示 "需设置 GITHUB_TOKEN env var, per 2026-08-27 11:06 JST 硬 ban, 不会被自动注入" | 不暴露 |
| env value 出现在日志 / 响应 | 强校验：所有返回前过滤 `${GITHUB_TOKEN}` `Bearer <value>` 等模式 | per 2026-08-27 11:06 JST 硬 ban |
| task_id 不在 WBS 145 范围内 | 弹窗黄态 "未知 task_id, 请查 WBS L4 进度表" | 不静默失败 |

---

## 5. 数据模型

### 5.1 5 数据文件 Schema

#### 5.1.1 data/ai-ledger.jsonl（append-only）

```json
{"ts":"2026-09-01T15:00:00+09:00","session_id":"mvs_xxx","agent_name":"mavis","task_id":"WF-1-55.27",
 "tokens_in":0,"tokens_out":0,"estimated":true,"message_count":42,"estimated_tokens":210000,
 "source":"mavis-hook-v0.1"}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| ts | ISO 8601 | ✅ | 写入时间 |
| session_id | string | ✅ | mavis session id |
| agent_name | enum | ✅ | mavis / explore / worker / verifier / <custom> |
| task_id | string | ✅ | WBS L4 ID（per `.wbs-task-marker` 关联，无则填 "unmapped"）|
| tokens_in | int | ✅ | 真实输入 tokens（v0.2 provider counter，v0.1 = 0）|
| tokens_out | int | ✅ | 真实输出 tokens（v0.2 provider counter，v0.1 = 0）|
| estimated | bool | ✅ | true = 估算值，false = 真实值 |
| message_count | int | ✅ | session message 数（v0.1 估算公式输入）|
| estimated_tokens | int | ✅ | 估算值 = message_count × 5000 |
| source | enum | ✅ | mavis-hook-v0.1 / mavis-session-list-fallback / manual |

#### 5.1.2 data/git-ledger.jsonl（append-only）

```json
{"ts":"2026-09-01T15:00:00+09:00","commit_hash":"abc1234","task_id":"WF-1-55.27",
 "worktree":"D:/RustGameServer/.worktrees/wf-1-55.27","author":"Ulysses",
 "estimated_tokens":80000,"source":"git-commit","diff_lines":156}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| ts | ISO 8601 | ✅ | rgs-web 检测到时间 |
| commit_hash | string | ✅ | git commit hash |
| task_id | string | ✅ | 关联 L4 ID |
| worktree | path | ✅ | worktree 绝对路径 |
| author | string | ✅ | git author |
| estimated_tokens | int | ✅ | 单 commit 估算（80K 行业经验，per RGS-OLU-REPORT §3.5）|
| source | enum | ✅ | git-commit / git-merge / git-tag |
| diff_lines | int | ✅ | diff 行数（备用估算输入）|

#### 5.1.3 data/github-cache.json（10 min TTL）

```json
{"fetched_at":"2026-09-01T15:00:00+09:00","ttl":600,
 "issues":[
   {"number":123,"title":"[WF-1-55.27] ...","state":"open","labels":["token-budget","WBS-L4"],
    "created_at":"...","updated_at":"...","html_url":"https://github.com/..."}
 ]}
```

#### 5.1.4 data/gitlab-cache.json（同 5.1.3，GitLab API 路径不同）

#### 5.1.5 data/.lock（atomic lockfile）

```
格式：空文件，存在 = 已锁，不存在 = 未锁
机制：fs.openSync(path, 'wx') 原子创建（O_EXCL）
```

### 5.2 API Response Schema

#### 5.2.1 /api/token/summary

```typescript
interface TokenSummaryResp {
  week_start: string;      // ISO 8601, 本周一
  total_budget: number;    // 本周所有任务 budget 之和
  total_actual: number;    // 本周 ledger 实际值
  percent_used: number;    // 0-100
  nfr_op_010_limit: number; // 20_000_000 (20M tokens/周)
  status: 'green' | 'yellow' | 'red';  // <70% / 70-90% / >90%
  domains: DomainToken[];  // 5 域分摊
  today: number;           // 今日 token
  tasks: TaskToken[];      // 145 任务
}

interface DomainToken {
  domain: 'player' | 'economy' | 'match' | 'social' | 'admin' | 'shared-platform' | 'cluster-ops' | 'saga';
  total_actual: number;
  total_budget: number;
  percent_used: number;
  task_count: number;
  done_count: number;
}

interface TaskToken {
  task_id: string;          // WF-1-55.27
  summary: string;          // 任务摘要
  status: 'pending' | 'in_progress' | 'done' | 'blocked';
  progress: number;         // 0-100
  budget_tokens: number;    // 推算 = 人·天 × 200K
  actual_tokens: number;    // 实际
  percent_used: number;     // 0-∞
  sessions: number;         // 关联 mavis session 数
  commits: number;          // 关联 commit 数
  github_issues: number[];  // 关联 GitHub issue 编号列表
}
```

#### 5.2.2 /api/token/budget-vs-actual

```typescript
interface BudgetVsActualResp {
  tasks: { task_id: string; budget: number; actual: number; status: string; percent: number }[];
  p95_percent: number;  // 95% 分位线
  over_budget: string[];  // percent_used > 95% 的 task_id 列表
  generated_at: string;
}
```

#### 5.2.3 /api/token/by-domain

```typescript
interface ByDomainResp {
  domains: { domain: string; actual: number; budget: number; color: string }[];
  // color per rgs-web 母规范 var: player=blue / economy=green / match=orange / social=purple / admin=red
  // shared-platform=cyan / cluster-ops=yellow / saga=pink
  generated_at: string;
}
```

#### 5.2.4 /api/token/nfr-op-010

```typescript
interface NfrOp010Resp {
  week_start: string;
  total_actual: number;
  limit: number;  // 20_000_000
  percent_used: number;
  status: 'green' | 'yellow' | 'red';
  week_breakdown: { day: string; tokens: number }[];  // 本周 7 天
  man_day_track: number;  // 人·天轨 Σ（145 任务按 owner 累加，per 母规范 §5 推导）
}
```

#### 5.2.5 /api/git/integrations

```typescript
interface IntegrationsResp {
  github: { enabled: boolean; repo?: string; rate_limit?: { remaining: number; reset_at: string } };
  gitlab: { enabled: boolean; project_id?: string };
  // env value 永不出现在响应中
  // token = "***"（脱敏）
}
```

---

## 6. 关联文档与演进

### 6.1 文档三件套

| 文档 | 状态 | 备注 |
|---|---|---|
| RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1 | ✅ 已落地 | 本文之上游 |
| **RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1** | **✅ 本文** | 中游 |
| RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1 | ⏳ 起草中 | 下游（API 签名 + 数据模型细节 + 部署 + 运维 + 安全）|
| RGS-OLU-WEB-PLAN-2026-09-01 v0.1 | ⏳ 起草中 | 总览 + 实施计划 |

### 6.2 与母规范 v0.3 关系

- **不破坏母规范**：rgs-web v0.3 已有 10 页面 + 6 API + k3s 代理全部保留
- **新增内容**：11 号 page-gantt + 9 API endpoint + 5 数据文件
- **API 路径空间**：/api/token/* / /api/ai/* / /api/git/integrations/* 三个命名空间，不与母规范 6 个 endpoint 冲突
- **前端样式**：沿用 rgs-web 母规范 v0.3 CSS 变量（`--bg / --panel / --green / --red / --blue / --orange / --purple / --cyan / --yellow`），新增 1 个 `--pink` for saga

---

## 7. 验收者（per 2026-08-26 08:40 JST 代签新规则）

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
| v0.1 | 2026-09-01 | 架构师（**Mavis 接手 agent per DEC-008**）| 首版：1 架构总览 + 2 技术选型（10 新增决策 + 9 不选方案）+ 3 模块划分（9 新增模块）+ 4 关键流程（5 流程 + 8 异常）+ 5 数据模型（5 数据文件 + 5 API Schema）|

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态：rgs-web v0.3 无 Gantt / Token / AI 子系统
- v0.1 新增：本文档
- v0.1 子系统落地：page-gantt（4 选项卡）+ 9 API + 5 数据文件 + 1 锁文件 + 2 lib helper

### A.2 对详细设计的影响

- 触发 RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1 起草
- 5 数据文件 → 5 文件详细 Schema + 写并发
- 5 API Response Schema → 9 API 完整签名 + 错误码 + 鉴权
- 5 异常流程 → 完整错误处理 + 退避策略

### A.3 已知缺口

- 5 域 Lead / shared-platform / cluster-ops / SRE / PM 签字未到（DDD Review 阶段补）
- mavis runtime hook 集成未确认（per REQUIREMENTS §9 风险 1）
- GitHub PAT / GitLab PAT 注入路径未确认（per REQUIREMENTS §6.2 v0.2 验收）
- RGS-ENV-CALIB-001 校准数据未生成（per RGS-OLU-REPORT-2026-08-27 v0.1 §10 GAP-1/2/3，推算公式精度待 v0.2 真实值校准）

### A.4 引用链与证据

- rgs-web 母规范 5 份文档（per `docs/12-工作流/RGS-WEB-*.md`）
- RGS-TS-001 v0.7 §6.2 OLU 双轨制
- RGS-WBS-001 v0.3 §2A 145 L4 任务 + L4 进度表 v0.4
- RGS-OLU-REPORT-2026-08-27 v0.1 §3 token 估算公式
- mavis skill §hook management（per mavis runtime 文档）
- per DEC-008 一人公司 12 角色
- per 2026-08-26 08:40 JST Mavis 默认代签 Ulysses
- per 2026-08-27 11:06 JST env value hard ban
- per 2026-09-01 13:03 / 13:05 JST envoy 独立 deployment 偏好
- per 2026-09-01 14:58 JST 拍板决策必须用选项
