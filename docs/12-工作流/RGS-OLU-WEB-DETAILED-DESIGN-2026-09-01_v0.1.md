# RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1

**Token 消耗可视化子系统详细设计（rgs-web Gantt + Token 选项卡）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 15:44 JST 触发）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1 已落地，本层补详细设计 |
| 关联 | RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1（上游）+ RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1（上游）+ RGS-OLU-WEB-PLAN-2026-09-01 v0.1（总览）+ rgs-web 母规范 5 份 |
| 上游母规范 | RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1 §1-§5 API 签名 / §6 数据模型 / §7 部署 / §8 运维 / §9 安全 |
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 文档定位

本文档是 rgs-web Token 子系统**详细设计层**，回答"How 细节" — 不涉及"What + Why"（已 in REQUIREMENTS）也不涉及"How 概要"（已 in BASIC-DESIGN）。

按 RGS 项目规范（per RGS-DTL-001 设计模式）：需求 → 基本 → 详细。

**继承母规范**：rgs-web 母规范 `RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1` §1 API 签名 / §6 数据模型 / §7 部署 / §8 运维 / §9 安全 **全部继承**，本文档只补充 Token 子系统**新增**内容。

---

## 1. 9 API 详细签名

> 母规范 §1 通用 API 模式（端口 8788 + JSON 响应 + 30s 轮询）**全部继承**。本节列 9 新增 API 详细签名。

### 1.1 /api/token/summary

**方法**：GET

**Query 参数**：
- `week_start` (optional): ISO 8601 日期，默认本周一（Asia/Tokyo 时区）

**响应 200**：
```json
{
  "week_start": "2026-08-31T00:00:00+09:00",
  "total_budget": 30400000,
  "total_actual": 2150000,
  "percent_used": 7.07,
  "nfr_op_010_limit": 20000000,
  "nfr_op_010_percent": 10.75,
  "status": "green",
  "today": 350000,
  "domains": [
    { "domain": "player", "total_actual": 600000, "total_budget": 5200000, "percent_used": 11.54, "task_count": 28, "done_count": 4, "color": "var(--blue)" },
    { "domain": "economy", "total_actual": 800000, "total_budget": 6800000, "percent_used": 11.76, "task_count": 31, "done_count": 5, "color": "var(--green)" },
    { "domain": "match", "total_actual": 200000, "total_budget": 4400000, "percent_used": 4.55, "task_count": 27, "done_count": 3, "color": "var(--orange)" },
    { "domain": "social", "total_actual": 150000, "total_budget": 4000000, "percent_used": 3.75, "task_count": 25, "done_count": 4, "color": "var(--purple)" },
    { "domain": "admin", "total_actual": 100000, "total_budget": 3600000, "percent_used": 2.78, "task_count": 20, "done_count": 2, "color": "var(--red)" },
    { "domain": "shared-platform", "total_actual": 200000, "total_budget": 2400000, "percent_used": 8.33, "task_count": 8, "done_count": 2, "color": "var(--cyan)" },
    { "domain": "cluster-ops", "total_actual": 100000, "total_budget": 2000000, "percent_used": 5.0, "task_count": 4, "done_count": 1, "color": "var(--yellow)" },
    { "domain": "saga", "total_actual": 0, "total_budget": 2000000, "percent_used": 0.0, "task_count": 2, "done_count": 0, "color": "var(--pink)" }
  ],
  "tasks": [
    { "task_id": "WF-1-55.27", "summary": "ReserveHandler OCC cleanup + reservation release 失败路径真修", "status": "done", "progress": 100, "budget_tokens": 400000, "actual_tokens": 245000, "percent_used": 61.25, "sessions": 1, "commits": 4, "github_issues": [] }
  ],
  "generated_at": "2026-09-01T15:30:00+09:00"
}
```

**错误码**：
- 500: ai-ledger.jsonl 解析失败（`{ error, hint: "检查 data/ai-ledger.jsonl 是否被外部修改" }`）
- 500: WBS L4 进度表读取失败（`{ error, hint: "per RGS-WT-001 §8.3.4 跨 worktree 锁冲突" }`）

**实现位置**：`tools/rgs-web/server.js` `routeTokenSummary(req, res)`

**实现要点**：
- 读 `data/ai-ledger.jsonl`（filter 本周）+ 读 `data/git-ledger.jsonl`（filter 本周）
- 读 `docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md` 解析 §3 汇总 + §4 详细行
- 内存聚合：本周 / 今日 / 域级 / 任务级
- budget_tokens 推算：若 WBS 表格无 `budget_tokens` 字段，按 `人·天 × 200000` 推算（per BASIC-DESIGN §5.2.1 + RGS-TS-001 v0.7 §6.2.2.1）
- percent_used 颜色阈值：≤ 70% green / 70-90% yellow / > 90% red

### 1.2 /api/token/budget-vs-actual

**方法**：GET

**Query 参数**：
- `domain` (optional): player/economy/match/social/admin/shared-platform/cluster-ops/saga/all
- `status` (optional): pending/in_progress/done/blocked/all

**响应 200**：
```json
{
  "tasks": [
    { "task_id": "WF-1-55.27", "budget": 400000, "actual": 245000, "status": "done", "percent": 61.25, "summary": "..." }
  ],
  "p95_percent": 95.0,
  "over_budget": ["WF-1-55.69", "WF-1-55.77"],
  "generated_at": "..."
}
```

**实现要点**：同 1.1 + 计算 P95 分位 + 异常任务列表（percent > 95%）

### 1.3 /api/token/by-domain

**方法**：GET

**响应 200**：
```json
{
  "domains": [
    { "domain": "player", "actual": 600000, "budget": 5200000, "color": "var(--blue)", "task_count": 28, "done_count": 4 }
  ],
  "generated_at": "..."
}
```

### 1.4 /api/token/nfr-op-010

**方法**：GET

**响应 200**：
```json
{
  "week_start": "2026-08-31T00:00:00+09:00",
  "total_actual": 2150000,
  "limit": 20000000,
  "percent_used": 10.75,
  "status": "green",
  "week_breakdown": [
    { "day": "2026-08-31", "tokens": 200000 },
    { "day": "2026-09-01", "tokens": 350000 }
  ],
  "man_day_track": 1.5,
  "man_day_limit": 20
}
```

**实现要点**：本周 7 天按 ISO 周一算；man_day_track 简化（per RGS-TS-001 v0.7 §6.2 双轨）

### 1.5 /api/ai/ledger

**方法**：GET

**Query 参数**：
- `limit` (default 50, max 500)
- `task_id` (optional)
- `session_id` (optional)
- `since` (optional ISO 8601)

**响应 200**：
```json
{
  "entries": [
    { "ts": "2026-09-01T15:00:00+09:00", "session_id": "mvs_xxx", "agent_name": "mavis", "task_id": "WF-1-55.27", "tokens_in": 0, "tokens_out": 0, "estimated": true, "message_count": 42, "estimated_tokens": 210000, "source": "mavis-hook-v0.1" }
  ],
  "total": 1,
  "generated_at": "..."
}
```

### 1.6 /api/ai/sessions

**方法**：GET

**实现要点**：
- `execSync('mavis session list --json')` 30s 缓存到内存
- 拉取 `mavis session list` 真实 session 历史（per BASIC-DESIGN §3.2）
- v0.1 替代 hook 未集成场景

**响应 200**：
```json
{
  "sessions": [
    { "id": "mvs_xxx", "agent": "mavis", "title": "5 域 gRPC 探活", "created_at": "...", "updated_at": "...", "message_count": 42, "estimated_tokens": 210000 }
  ],
  "generated_at": "..."
}
```

### 1.7 /api/ai/agents

**方法**：GET

**实现要点**：`execSync('mavis agent list --json')` 30s 缓存

**响应 200**：
```json
{
  "agents": [
    { "name": "mavis", "display_name": "Mavis", "role": "orchestrator", "is_primary": true },
    { "name": "explore", "display_name": "Explore", "role": "explore" },
    { "name": "worker", "display_name": "Worker", "role": "worker" },
    { "name": "verifier", "display_name": "Verifier", "role": "verifier" }
  ],
  "generated_at": "..."
}
```

### 1.8 /api/git/integrations

**方法**：GET

**实现要点**：
- 读 `process.env.GITHUB_TOKEN` / `GITLAB_TOKEN` 存在性检查（**不读值**）
- 读 `process.env.GITHUB_REPO` / `GITLAB_PROJECT_ID`（仓库/项目标识，**非 secret**）
- 调用 `https://api.github.com/rate_limit` 探测 GitHub 凭据 + 限流状态（可选）
- 缓存 5 min

**响应 200**：
```json
{
  "github": {
    "enabled": true,
    "repo": "ulyssesleolee/RustGameServer",
    "auth_configured": true,
    "rate_limit": { "remaining": 4999, "reset_at": "2026-09-01T16:00:00+09:00" }
  },
  "gitlab": {
    "enabled": false,
    "auth_configured": false
  }
}
```

> **强制约束**：响应中 `auth_configured` 字段**不**包含 token 值；`token` 字段永不出现在响应（per 2026-08-27 11:06 JST 硬 ban + NFR-28）

### 1.9 /api/git/issues

**方法**：GET

**Query 参数**：
- `provider` (default github, enum: github/gitlab)
- `repo` (optional, override env GITHUB_REPO)
- `labels` (default token-budget, 多标签逗号分隔)
- `state` (default open, enum: open/closed/all)
- `limit` (default 30, max 100)

**方法 POST**：写回 issue 评论

**POST body**：
```json
{
  "issue_number": 123,
  "task_id": "WF-1-55.27",
  "include_deep_link": true
}
```

**POST 响应 201**：
```json
{
  "comment_id": 456789,
  "html_url": "https://github.com/.../issues/123#issuecomment-456789",
  "posted_at": "..."
}
```

**实现要点（POST）**：
- 读 `process.env.GITHUB_TOKEN`（不打印）
- 构造 POST `https://api.github.com/repos/<owner>/<repo>/issues/<n>/comments`
- body 模板：
  ```
  🤖 rgs-oludash-bot

  Task: WF-1-55.27
  Actual tokens: 245K / 400K (61%)
  Status: done 100%
  Sessions: 1
  Commits: 4

  Detail: http://127.0.0.1:8788/?page=gantt&task_id=WF-1-55.27&tab=token
  ```
- headers: `Authorization: token ${GITHUB_TOKEN}` + `User-Agent: rgs-oludash/0.1`
- 失败处理：401/403/404/429 分别返 4xx/5xx，**不**包含 token 值

---

## 2. 数据模型

### 2.1 5 数据文件

> BASIC-DESIGN §5.1 已声明 5 数据文件 Schema，本节补充**实现细节**。

#### 2.1.1 tools/rgs-web/data/ai-ledger.jsonl

**路径**：`tools/rgs-web/data/ai-ledger.jsonl`

**格式**：每行一个 JSON 对象，UTF-8，LF 换行（避免 Windows CRLF 污染）

**权限**：0600（仅当前用户可读写，per user_profile 127.0.0.1 only + 一人公司本机工具）

**滚动策略**：单文件 > 50MB（per NFR-24）自动滚动归档
- 触发：rgs-web 启动时 + 30s 轮询时
- 归档命名：`data/ai-ledger-YYYY-MM.jsonl`（按写入月归档）
- 当前活跃文件：`data/ai-ledger.jsonl`（追加）

**写并发**：`data/.lock` 原子锁（per BASIC-DESIGN §5.1.5）

**v0.1 写入方**：
1. mavis runtime hook（per mavis skill §hook management）
2. rgs-web 后台降级：`mavis session list --json` 30s 轮询补历史

**v0.2 写入方**：增加 provider counter 真实值

#### 2.1.2 tools/rgs-web/data/git-ledger.jsonl

**路径**：`tools/rgs-web/data/git-ledger.jsonl`

**格式**：同 2.1.1

**v0.1 写入方**：
- rgs-web 后台 setInterval(30s)：`git log --since=上次轮询 --pretty=format:%H|%ct|%s` 扫新增 commit
- 对每条 commit：`git -C <worktree> log -1 -- .wbs-task-marker` 读 task_id（worktree 路径从 `git worktree list --porcelain` 拿）

**估算公式**：`estimated_tokens = 80000 + diff_lines * 50`（基础 80K per RGS-OLU-REPORT §3.5 + diff 行数 × 50 tokens/行）

#### 2.1.3 tools/rgs-web/data/github-cache.json

**路径**：`tools/rgs-web/data/github-cache.json`

**TTL**：10 min（per NFR-26 + GitHub rate limit 缓解）

**v0.1 拉取源**：
- `GET https://api.github.com/repos/<owner>/<repo>/issues?labels=token-budget&state=open&per_page=30`
- headers: `Authorization: token ${GITHUB_TOKEN}`（env value 不打印）

**v0.2 拉取源**：GitLab API 同范式

#### 2.1.4 tools/rgs-web/data/gitlab-cache.json

**路径**：`tools/rgs-web/data/gitlab-cache.json`

**拉取源**：`GET https://gitlab.com/api/v4/projects/<project_id>/issues?labels=token-budget`

#### 2.1.5 tools/rgs-web/data/.lock

**格式**：空文件

**原子锁机制**：
- 锁获取：`fs.openSync('.lock', 'wx')`（O_EXCL 标志，原子创建）
- 锁释放：`fs.closeSync(fd) + fs.unlinkSync('.lock')`
- 锁失败：retry 3 次，指数退避 100ms / 200ms / 400ms
- 锁超时：单次临界区操作 < 1s

**死锁防护**：每次启动时检查 `.lock` 存在时间（mtime > 1h 视为僵死，删除）

### 2.2 WBS L4 进度表 budget_tokens 字段扩展

**WBS 文档路径**：`docs/12-工作流/RGS-WBS-001_L4任务进度表_v0.X.md`

**v0.1 字段扩展（不破坏现有格式）**：

在 §4 表格每行末尾追加 2 列：
- `人·天` (float, per RGS-WBS-001 v0.3 §2A)
- `budget_tokens` (int, 推算 = 人·天 × 200K)

**v0.1 推算 fallback**（per BASIC-DESIGN §5.2.1）：
- 若 WBS 表格无 `人·天` 字段，按默认 1 人·天 × 200K = 200K tokens
- 若 WBS 表格无 `budget_tokens` 字段，按 RGS-TS-001 v0.7 §6.2.2.1 中位数 200K/天推算
- 推算值在 UI 上标 "estimated"（per NFR-27）

**v0.2 真实校准**：等 RGS-ENV-CALIB-001 真实数据回填

### 2.3 rgs-web 内存模型

| 缓存 | Key | Value | TTL |
|---|---|---|---|
| mavis session list | `mvs:cache:mavis:sessions` | JSON array | 30s |
| mavis agent list | `mvs:cache:mavis:agents` | JSON array | 30s |
| WBS L4 任务解析 | `mvs:cache:wbs:tasks` | { [task_id]: TaskInfo } | 60s（写更新触发失效）|
| ai-ledger 聚合 | `mvs:cache:ledger:week` | { total, by_domain, by_task } | 30s |
| GitHub rate limit | `mvs:cache:gh:ratelimit` | { remaining, reset_at } | 5 min |

---

## 3. 部署

> 母规范 §7 部署（rgs-web node 22 + 8788 + WSL2 + k3s port-forward）**全部继承**。本节列 Token 子系统**新增**部署步骤。

### 3.1 新增文件

```
tools/rgs-web/
├── data/                           (新增目录)
│   ├── .gitignore                  "*.jsonl\n!.gitkeep" （jsonl 落本地不进 git, 避免敏感 token 流外泄）
│   ├── .gitkeep
│   ├── ai-ledger.jsonl             (启动时自动创建)
│   ├── git-ledger.jsonl            (启动时自动创建)
│   ├── github-cache.json           (10 min TTL)
│   └── gitlab-cache.json           (10 min TTL)
├── lib/                            (新增目录)
│   ├── lockfile.js                 原子锁
│   ├── token-estimate.js           估算公式
│   └── mavis-bridge.js             mavis execSync 包装
├── public/index.html               (扩展 11 页面 + 4 选项卡)
└── server.js                       (扩展 9 API)
```

### 3.2 mavis runtime hook 集成

**v0.1 集成位置**：`C:\Users\leo19\.minimax\agents\mavis\hooks\oludash-write-ledger.js`（per mavis skill §hook management）

**hook 配置**（写入 mavis 配置文件）：
```json
{
  "hooks": {
    "session_finish": {
      "command": "node",
      "args": ["C:/Users/leo19/.minimax/agents/mavis/hooks/oludash-write-ledger.js"],
      "env_passthrough": ["MAVIS_SESSION_ID", "MAVIS_AGENT_NAME", "MAVIS_TASK_ID"]
    }
  }
}
```

**hook 实现要点**：
- 读 mavis env vars
- 调 mavis runtime API 拿 message count
- 调 `node C:/path/to/tools/rgs-web/lib/lockfile.js append data/ai-ledger.jsonl <json>`
- 失败重试 3 次

### 3.3 启动 SOP

```bash
# 1. (一次性) 配 mavis hook, per 3.2
# 2. (一次性) 配 env var, per 1.8:
#    $env:RGS_WEB_PORT = 8788
#    $env:GITHUB_TOKEN = '<PAT>'  # 不要 echo!
#    $env:GITHUB_REPO = 'ulyssesleolee/RustGameServer'
# 3. 启动 rgs-web
cd D:/RustGameServer
node tools/rgs-web/server.js
# 访问 http://127.0.0.1:8788/?page=gantt
```

---

## 4. 运维

> 母规范 §8 运维（启动 / 监控 / 故障恢复）**全部继承**。本节列 Token 子系统**新增**运维项。

### 4.1 监控项

| 指标 | 阈值 | 告警 |
|---|---|---|
| data/ai-ledger.jsonl 大小 | > 50MB | 自动滚动归档 |
| data/ai-ledger.jsonl 写入失败 | 连续 3 次 | 顶部条黄态 + 提示"AI ledger 写入异常" |
| mavis session list 拉取失败 | 连续 3 次 | 顶部条黄态 + 降级到无 AI 视图 |
| git log 30s 轮询失败 | 连续 3 次 | 顶部条黄态 + 降级到无 git ledger |
| NFR-OP-010 比例 | > 90% | 顶部条红态 + (v0.2) mavis cron 自检 |
| GitHub rate limit remaining | < 100 | 顶部条黄态 + 暂停写回评论 |

### 4.2 故障恢复

| 故障 | 恢复 SOP |
|---|---|
| ai-ledger.jsonl 损坏 | mv 损坏文件 `data/ai-ledger-YYYY-MM-DD-corrupt.jsonl` + 启动 rgs-web 自动重建空文件 + 顶部条黄态提示 |
| git-ledger.jsonl 损坏 | 同上 |
| WBS L4 进度表格式变化 | rgs-web 启动时检测 §4 表格列数，< 6 列 = 黄态提示"v0.1 需 §4 表格加 budget_tokens 列" |
| mavis hook 写锁死（> 1h）| 启动时检测 mtime，删除僵死 lock，顶部条黄态 |
| GITHUB_TOKEN 失效 | /api/git/integrations 返 `auth_configured: true, rate_limit: 401` + 顶部条黄态 |

### 4.3 数据备份

- 5 数据文件全部在 `tools/rgs-web/data/`，**不**纳入 git（per 3.1 `.gitignore`）
- 手动备份：`pwsh scripts/backup-rgs-web-data.ps1`（v0.2 提供，per BASIC-DESIGN §6.2 v0.1 文档三件套）
- 备份目标：`D:/rgs-archive/rgs-web-data-YYYY-MM-DD.zip`

---

## 5. 安全

> 母规范 §9 安全（NFR-10 至 NFR-14 监听地址 / TLS / 凭据 / Cookie / 文件权限）**全部继承**。本节列 Token 子系统**新增**安全约束。

### 5.1 凭据管理

| 凭据 | 注入方式 | 存储 | 暴露风险 |
|---|---|---|---|
| GITHUB_TOKEN | env var | 内存（`process.env.GITHUB_TOKEN`）| 仅 https.request Authorization header，永不出现在响应 / 日志 |
| GITLAB_TOKEN | env var | 内存 | 同 |
| K3S_TOKEN (母规范) | env var | 内存 | 同 |

**强制约束（per 2026-08-27 11:06 JST 硬 ban）**：
- ❌ `Get-ChildItem env: | Format-Table`
- ❌ `echo $GITHUB_TOKEN` / `cat .env`
- ❌ 响应中包含 token 字段（即使部分脱敏）
- ❌ 日志 / 错误信息中包含 token 值

**强校验**：所有 API handler 返响应前过滤 `${GITHUB_TOKEN}` / `Bearer <value>` 等模式

### 5.2 写并发安全

- ai-ledger.jsonl 多写者（mavis hook + rgs-web 后台轮询降级）：用 lockfile
- git-ledger.jsonl 单写者（rgs-web 后台轮询）：可不加锁，但加锁防御性写
- github-cache.json / gitlab-cache.json 单写者（rgs-web API handler）：可不加锁
- 1 写者约束（per BASIC-DESIGN §5.1.5 + rgs-web 母规范 v0.1 决策"零依赖 + 单进程单线程"）

### 5.3 数据隐私

- ai-ledger.jsonl 含 session_id / task_id，**不**含用户聊天内容（mavis 不输出聊天内容到 hook）
- git-ledger.jsonl 含 commit hash + diff 行数，**不**含 diff 内容
- jsonl 文件 0600 权限
- 不进 git（per §3.1 .gitignore）
- 备份加密（v0.2 评估，per user_profile 1 人公司本机工具暂时不需要）

### 5.4 127.0.0.1 only 硬约束

- rgs-web 监听 `127.0.0.1:8788`（per rgs-web 母规范 §2.1 决策）
- 不接受 `0.0.0.0` / `localhost` 之外
- webhook inbound（v0.3）冲突 → v0.3 待 user_profile 讨论后定

---

## 6. 性能

> 母规范 §NFR-1 至 NFR-5（< 2s 首页 / < 500ms API / < 100MB 内存 / < 1s 启动 / 10 RPS）**全部继承**。本节列 Token 子系统**新增**性能约束。

| # | 指标 | 目标 | 实测方法 |
|---|---|---|---|
| PF-1 | page-gantt 加载 | < 3s | `curl -w '%{time_total}\n' http://127.0.0.1:8788/?page=gantt` |
| PF-2 | /api/token/summary 响应 | < 200ms | 内存聚合，145 任务 ~20ms |
| PF-3 | /api/token/budget-vs-actual SVG 渲染 | < 500ms | 145 SVG bar × ~3ms = ~450ms |
| PF-4 | ai-ledger.jsonl 30s 轮询可见 | ≤ 30s | setInterval 30s + 写时立即触发 |
| PF-5 | data/ai-ledger.jsonl 单文件 | < 50MB | 50MB ≈ 100K 行，145 任务活跃数月无压力 |
| PF-6 | Gantt SVG 渲染 145 任务 | < 1s | SVG 节点 ~500，单次 draw 10ms |

---

## 7. 测试策略

### 7.1 单元测试

- `lib/lockfile.js`：并发 100 个 fs.openSync('.lock', 'wx')，期望 1 成功 99 失败
- `lib/token-estimate.js`：message_count=42 → 210000 tokens
- `lib/mavis-bridge.js`：mock `execSync` 返 mavis session list 样本，断言解析

### 7.2 集成测试

- mavis hook → ai-ledger.jsonl 写入 → rgs-web /api/token/summary 读取 → 顶部条更新，期望 ≤ 30s
- git commit → .wbs-task-marker → rgs-web 后台 30s 轮询 → git-ledger.jsonl → /api/token/by-domain 更新

### 7.3 端到端测试

- 打开 `http://127.0.0.1:8788/?page=gantt&task_id=WF-1-55.27&tab=token`
- 期望 4 选项卡可见
- 切到 Token 选项卡
- 期望：双柱图渲染 145 任务 / 顶部 NFR-OP-010 绿态 / 5 域堆叠图
- 点 task_id 弹窗
- 期望：budget / actual / sessions[] / commits[] / GitHub 关联按钮
- 验证 < 3s 加载 / 30s 轮询

### 7.4 凭据泄露测试

- 在响应中搜索 `GITHUB_TOKEN` 值字符串，期望 0 命中
- 在日志中搜索 `Bearer\s+\w+`，期望 0 命中
- 在错误信息中搜索 `token` 字段，期望仅出现 `auth_configured` / `enabled` 等布尔字段

---

## 8. 演进路径

| 版本 | 内容 | 触发 |
|---|---|---|
| v0.1（本文）| 11 页面 + 4 选项卡 + 9 API + 5 数据文件 + 锁文件 | 立即落地（4 周内）|
| v0.2 | + provider counter 真实值接入（取消估算）+ GitLab 写回评论 + NFR-OP-010 自检 cron | RGS-ENV-CALIB-001 校准完成后 |
| v0.3 | + GitHub/GitLab webhook inbound + token 历史趋势折线图 + 5 域 binary 自身 LLM token 出口 | user_profile 127.0.0.1 only 硬约束讨论后 |
| v1.0 | rgs-web Rust 重写（axum + tera 模板），per rgs-web 母规范 v0.1 §6.3 目标 | 1 年后 |

---

## 9. 验收者（per 2026-08-26 08:40 JST 代签新规则）

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
| v0.1 | 2026-09-01 | 架构师（**Mavis 接手 agent per DEC-008**）| 首版：9 API 详细签名 + 5 数据文件实现细节 + 2 内存模型 + 3 部署 + 4 运维 + 5 安全 + 6 性能 + 7 测试 + 8 演进路径 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态：rgs-web v0.3 无 Gantt / Token / AI 子系统
- v0.1 新增：本文档
- v0.1 子系统落地：9 API 完整签名 + 5 数据文件 + 锁文件 + 内存模型 + 部署 SOP + 监控项 + 故障恢复 + 凭据管理 + 性能指标 + 测试策略

### A.2 对 PLAN 的影响

- 触发 RGS-OLU-WEB-PLAN-2026-09-01 v0.1 起草（总览 + 实施计划）
- 9 API + 5 数据文件 + 1 锁文件 + 1 内存模型 = 16 落地项
- 估算 v0.1 工作量 ~4 周（per REQUIREMENTS §7.3）

### A.3 已知缺口

- 5 域 Lead / shared-platform / cluster-ops / SRE / PM 签字未到（DDD Review 阶段补）
- mavis runtime hook 集成未确认（per REQUIREMENTS §9 风险 1 + BASIC-DESIGN §3.2 v0.1 降级方案）
- GITHUB_TOKEN / GITLAB_TOKEN 注入路径未确认（per §1.8 + REQUIREMENTS §6.2 v0.2 验收）
- RGS-ENV-CALIB-001 校准数据未生成（per RGS-OLU-REPORT-2026-08-27 v0.1 §10 GAP-1/2/3，推算公式精度待 v0.2 真实值校准）
- WBS L4 进度表加 budget_tokens 字段工作未做（per §2.2 v0.1 推算 fallback，v0.2 真实校准）

### A.4 引用链与证据

- rgs-web 母规范 5 份文档（per `docs/12-工作流/RGS-WEB-*.md`）
- RGS-TS-001 v0.7 §6.2 OLU 双轨制
- RGS-WBS-001 v0.3 §2A 145 L4 任务 + L4 进度表 v0.4
- RGS-OLU-REPORT-2026-08-27 v0.1 §3 token 估算公式 + §10 已知缺口 GAP-1/2/3
- mavis skill §hook management（per mavis runtime 文档）
- per DEC-008 一人公司 12 角色
- per 2026-08-26 08:40 JST Mavis 默认代签 Ulysses
- per 2026-08-27 11:06 JST env value hard ban
- per 2026-09-01 13:03 / 13:05 JST envoy 独立 deployment 偏好
- per 2026-09-01 14:58 JST 拍板决策必须用选项

### A.5 v0.2 升版增量（per Ulysses 4 + 2 ask_user 决策，2026-09-01 16:41 JST）

> **v0.1 主体不追溯改写**。v0.2 增量 = 5 大块，落地到 DETAILED-DESIGN 各章节：

**1. GitHub/GitLab 浅联动 → 深联动 webhook inbound**（per ask_user 16:30 JST）
- §1.10 `/api/webhook/github` 新增：HMAC-SHA256 验签 + UNIQUE(provider, delivery_id) 重放保护 + 事务
- §1.11 `/api/webhook/gitlab` 新增：X-Gitlab-Token 等值比较（恒定时间防 timing attack）
- 错误码：401 验签失败 / 400 缺必填头 / 500 SQLite 写失败
- 性能 < 200ms（per NFR-33）

**2. better-sqlite3 存储 + 备份清理 batch**（per ask_user 16:30/16:41 JST）
- §2.1 SQLite 单文件 `data/olu.db` + 6 表 schema + PRAGMA (WAL / busy_timeout=5000 / synchronous=NORMAL / foreign_keys=ON)
- §2.4 备份策略：cron / Windows 任务计划 / rgs-web 启动检查 3 选 1，默认 cron
- §3.1 lib/sqlite.js + lib/backup-batch.js + data/olu.db + data/backups/olu-YYYY-MM-DD.db
- §3.2 mavis hook 改 SQLite INSERT（不静默降级 jsonl，per §7.1 派生约束 fail-fast）
- §3.3 启动 SOP 加 `npm install better-sqlite3` + `cloudflared --version` + cron 配置

**3. cloudflared tunnel 解 webhook + 127.0.0.1 only 冲突**（per ask_user 16:41 JST）
- §3.1 lib/cloudflared.js
- §3.3 启动 SOP 加 cloudflared 装 + 启动
- §5.4 127.0.0.1 only 硬约束：cloudflared 是 outbound tunnel，不破硬约束

**4. webhook 验签 + 重放保护**（per F-32/F-33）
- §1.10/§1.11 详细签名
- §3.1 lib/webhook-verifier.js
- §5.1 凭据管理增 GITHUB_WEBHOOK_SECRET / GITLAB_WEBHOOK_TOKEN
- §5.2 写并发：webhook 端点每条 webhook 1 事务（原子性）

**5. 备份 batch**（per ask_user "详细的记录备份清理 batch"）
- §2.4 备份策略（VACUUM INTO + sha256 + 90 天清理 + 写 nfr_op_010_snapshots 表）
- §3.1 lib/backup-batch.js
- §3.3 cron 启动 SOP

**已知缺口**（v0.2 新增）：
- cloudflared 二进制需 Ulysses 手动装
- GITHUB_WEBHOOK_SECRET + GITLAB_WEBHOOK_TOKEN 注入路径
- better-sqlite3 Windows 编译风险
- 备份 batch 触发方式
- mavis runtime hook 写 ai_ledger 表 schema 兼容性
- 4 周落地工作量是否够
- v0.2 6 NFR 实测基线未建立

**派生决策引用**：per 2026-09-01 14:58 JST + per "Never auto-install software" 硬约束

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 15:44 JST | 架构师（**Mavis 接手 agent per DEC-008**）| 首版（commit `a896ca9`）|
| **v0.2** | **2026-09-01 16:41 JST** | **架构师（**Mavis 接手 agent per DEC-008**）** | **v0.2 升版**：① §1.10/§1.11 webhook 端点 ② §2.1 SQLite 6 表 + PRAGMA ③ §2.4 备份策略 ④ §3.1 lib/sqlite.js / cloudflared.js / backup-batch.js ⑤ §3.3 启动 SOP + cloudflared + cron ⑥ §5.1 webhook secret ⑦ §5.2 webhook 事务 ⑧ §5.4 cloudflared 进程隔离 |
