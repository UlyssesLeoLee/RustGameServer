# RGS-WEB-REQUIREMENTS-2026-08-26 v0.1

**RGS Admin Web 需求规约（Requirements Specification）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WEB-REQUIREMENTS-2026-08-26 |
| 版本 | 0.1（per Ulysses 2026-08-26 12:30 JST "从需求到基本设计，详细设计都要补充完整"） |
| 状态 | 草案（待 Ulysses DDD Review 阶段补签） |
| 触发 | 2026-08-26 12:15 JST Ulysses "RGS 需要一个网页端的后台" + 12:30 JST "从需求到基本设计，详细设计都要补充完整" |
| 关联 | RGS-WEB-PLAN-2026-08-26 v0.1（设计总览）+ RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1（基本设计）+ RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1（详细设计） |
| 责任人 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））|
| 适用许可 | Apache-2.0（本仓库） |

---

## 0. 文档定位

本文档是 RGS Admin Web（rgs-web）的**需求规约层**，描述"为什么做"和"做什么"——不涉及"怎么做"。按 RGS 项目三层文档规范（per RGS-DTL-001 设计模式：需求规约 / 基本设计 / 详细设计），本文档回答 5W1H 中的 **What + Why**。

**三层文档对应关系**：

| 层级 | 文档 | 回答 |
|---|---|---|
| 需求规约 | **本文档** | What + Why（用户痛点 + 业务目标 + 功能需求 + 非功能需求） |
| 基本设计 | RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1 | How 概要（架构 + 技术选型 + 模块划分 + 关键流程） |
| 详细设计 | RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1 | How 细节（API 签名 + 数据模型 + 部署 + 运维 + 安全） |

---

## 1. 背景与痛点

### 1.1 用户画像

**Ulysses**（per 用户档案 / DEC-008 一人公司）：
- 角色：1 人 12 角色（架构师 / 5 域 Lead / SRE / DBA / 安全 / shared-platform / saga 召集人 / PM / ...）
- 工作流：WBS v0.7→v0.8（commit `87a6472`）已含 145 L4 任务，DDD Review 阶段需逐项审 11 P0/P1/P2 commit
- 环境：Windows 11 + WSL2 Ubuntu + k3s + Rust + node 22
- 痛点：DDD Review 时需切换多个工具（git bash / kubectl / IDE / 文档阅读器）才能看完 1 份 SPEC

### 1.2 当前痛点（per RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分）

| # | 痛点 | 影响 | 频率 |
|---|---|---|---|
| 1 | DDD Review 无集中入口 | 5 域 Lead 签字时需手动翻 11+ commit + 3 文档表 | 每天 |
| 2 | k3s 5 域 pod 状态需手动 kubectl | 部署后验证靠 `wsl -d Ubuntu -- kubectl get pods`（WSL 命令慢 8-15s）| 每次部署 |
| 3 | check-docs-consistency.sh 需手跑 + 解析 | 1 FAIL + 1 WARN 基线看不到历史趋势 | 每次 commit 前 |
| 4 | 5 域 IMPL-PLAN 进度散落 6 份 markdown | 无法看"5 域 Lead 实际进度" | 每周 |
| 5 | 17 v0.2 SPEC merge 状态不直观 | 需逐个 `git log` 17 个 worktree 分支 | 每次 merge |

### 1.3 业务目标

**O-1**: DDD Review 阶段 5 域 Lead 能在 1 个网页内集中看 P0/P1/P2 commit 状态 + 签字栏状态（per RACI v1.0）
**O-2**: 部署验证时 1 个网页内看 k3s 集群 + 5 域 pod 状态 + cluster-ops 状态
**O-3**: 文档健康趋势可视化（1 FAIL + 1 WARN 基线 + 历次变化）
**O-4**: WBS v0.8 L4 任务 145 项总览 + 5 域 IMPL-PLAN 联动
**O-5**: 一人公司模式无 RBAC 复杂度（per DEC-008）

---

## 2. 用户故事（User Stories）

### 2.1 US-1 DDD Review 集中入口

> **作为** Ulysses（一架构师）
> **我想要** 在 1 个网页内看到 11 P0/P1/P2 commit + 5 域 RACI v1.0 签字栏
> **以便** 不切工具就能批 1 份 SPEC

**验收标准**：
- [ ] 11 commit 表格显示（branch / head hash / message / DDD Review 签字状态）
- [ ] 5 份 per-domain RACI v1.0 签字栏显示（per RGS-RACI-{域}-V1 §4）
- [ ] 单页加载 < 2s

### 2.2 US-2 k3s 集群状态

> **作为** Ulysses
> **我想要** 在网页看 5 域 pod 状态
> **以便** 部署后立即验证

**验收标准**：
- [ ] 显示 5 域 pod（player / economy / match / social / admin）
- [ ] 显示 pod phase（Running / Pending / Failed）
- [ ] 显示 restart count
- [ ] 单页加载 < 3s（k3s API 代理）

### 2.3 US-3 文档健康基线

> **作为** Ulysses
> **我想要** 看 check-docs-consistency.sh 跑过的最近 1 次结果
> **以便** commit 前确认无新 FAIL/WARN

**验收标准**：
- [ ] 显示当前 1 FAIL + 1 WARN（per RGS-DOCS-HEALTH-2026-08-26）
- [ ] 显示 FAIL/WARN 详情
- [ ] 显示上次 check 时间

### 2.4 US-4 5 域 IMPL-PLAN 总览

> **作为** Ulysses
> **我想要** 8 份 IMPL-PLAN v0.1（6 域 + CDN + LCM）的状态 + 任务簇进度
> **以便** 评估 5 域 Lead 实际执行进度

**验收标准**：
- [ ] 8 份 IMPL-PLAN 表格（文件 / 状态 / owner / 任务簇完成度）
- [ ] 联动 WBS v0.8 §2A.2.55.续3

### 2.5 US-5 11 worktree merge 状态

> **作为** Ulysses
> **我想要** 看 11 P0/P1/P2 worktree + 17 v0.2 + 1 main 的状态
> **以便** 决定哪些先 merge

**验收标准**：
- [ ] 41 个 worktree 表格
- [ ] 标注 P0/P1/P2 / v0.2 / 待 merge
- [ ] head commit 显示

---

## 3. 功能需求（Functional Requirements）

### 3.1 必备（Must Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-1 | 5 域 IMPL-PLAN 列表 API | `GET /api/impl-plan` 返回 8 份 PLAN 状态 | P0 |
| F-2 | 11 commit worktree 列表 API | `GET /api/worktrees` 返回 41 个 worktree | P0 |
| F-3 | 文档健康 API | `GET /api/docs-health` 返回 1 FAIL + 1 WARN 基线 | P0 |
| F-4 | Dashboard HTML 首页 | `GET /` 返回 dark theme dashboard | P0 |
| F-5 | 30s 自动 refresh | HTML meta refresh / JS setInterval | P0 |
| F-6 | 健康检查 API | `GET /api/health` 返回 k3s API URL + 当前时间 | P0 |
| F-7 | 静态资源服务 | `GET /public/*` 返回 CSS / JS / favicon | P0 |
| F-8 | 中文路径支持 | docs/12-工作流/*.md 路径含中文，需正确处理 | P0 |
| F-9 | 端口可配 | `RGS_WEB_PORT` 环境变量改默认 8788 | P0 |
| F-10 | 127.0.0.1 only 监听 | 不暴露到 0.0.0.0（一人公司本地工具）| P0 |

### 3.2 重要（Should Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-11 | k3s API 代理 | `ALL /api/k8s/*` 透传到 6443 | P1 |
| F-12 | k3s token 鉴权 | `K3S_TOKEN` + `K3S_CA_PATH` 环境变量 | P1 |
| F-13 | 5 域 pod 状态视图 | HTML dashboard 显示 5 域 pod phase | P1 |
| F-14 | 5 域 RACI v1.0 签字栏 | HTML 嵌入 5 份 per-domain RACI §4 签字栏 | P1 |
| F-15 | P0/P1/P2 commit 表格 | HTML 显示 11 commit 状态 | P1 |
| F-16 | 错误处理友好 | 5xx 返 JSON `{error: msg}` + HTTP 状态码 | P1 |
| F-17 | 进程 ID 输出 | `npm start` log 显示 PID（用于 kill）| P1 |

### 3.3 可选（Could Have）

| # | 需求 | 描述 | 优先级 |
|---|---|---|---|
| F-18 | WebSocket 实时推送 | 替代 30s 轮询 | P2 |
| F-19 | dark/light theme 切换 | 用户偏好 | P2 |
| F-20 | 多语言（zh-CN / en-US）| 国际化 | P2 |
| F-21 | dark theme 自定义 CSS 变量 | 用户定制 | P2 |
| F-22 | mobile 端响应式布局 | 移动端 DDD Review | P2 |

### 3.4 不做（Won't Have）

| # | 需求 | 不做理由 |
|---|---|---|
| F-W1 | 用户登录 / RBAC | 一人公司模式（per DEC-008） |
| F-W2 | 5 域实时部署触发 | RGS 部署走 git commit → k3s 自动 reconcile，Web UI 不触发部署 |
| F-W3 | 跨服务事务编排 | 5 域 saga 走 gRPC，Web UI 只读 |
| F-W4 | 数据写入 k3s / git | 严格只读，所有写入走 git commit + k3s declarative manifest |

---

## 4. 非功能需求（Non-Functional Requirements）

### 4.1 性能

| # | 指标 | 目标 | 备注 |
|---|---|---|---|
| NFR-1 | 首页加载 | < 2s | 127.0.0.1 本地 |
| NFR-2 | API 响应 | < 500ms | 不含 k3s API 透传 |
| NFR-3 | 内存占用 | < 100MB | node 22 + http |
| NFR-4 | 启动时间 | < 1s | 零依赖版本 |
| NFR-5 | 30s 内最大并发 | 10 RPS | 单人公司，10 RPS 足够 |

### 4.2 可用性

| # | 指标 | 目标 |
|---|---|---|
| NFR-6 | 启动失败退出码 | 1 |
| NFR-7 | 端口占用错误 | 显示 "EADDRINUSE" + 提示用 RGS_WEB_PORT |
| NFR-8 | 文件不存在 | 404 JSON `{error: "not found"}` |
| NFR-9 | 中文路径 | 正确返回（不 mojibake）|

### 4.3 安全性

| # | 指标 | 目标 |
|---|----|---|
| NFR-10 | 监听地址 | 127.0.0.1 only（不暴露 0.0.0.0）|
| NFR-11 | k3s token | 仅经环境变量传入，不落盘 |
| NFR-12 | TLS | k3s CA 自签证书验证（k3s 默认配置）|
| NFR-13 | 无 cookie / session | 一人公司无状态 |
| NFR-14 | 文件权限 | 启动后禁止写入 RGS 仓库任何文件 |

### 4.4 可维护性

| # | 指标 | 目标 |
|---|----|---|
| NFR-15 | 零依赖（v0.1） | 不需要 npm install 即可启动 |
| NFR-16 | 文档 3 层 | 需求 / 基本 / 详细 分层 |
| NFR-17 | 修订历史 | 每份文档 修订历史 v0.1 行 + 审批者（per 2026-08-26 08:40 JST 代签新规则）|
| NFR-18 | Git 证据 | 引用 commit 必用 `git log -p --follow` 实证（per DTL-036 v1.4.2 反馈）|

### 4.5 可移植性

| # | 指标 | 目标 |
|---|----|---|
| NFR-19 | Windows / macOS / Linux | 跨平台（node 18+ 即可）|
| NFR-20 | 端口冲突 | 可改 RGS_WEB_PORT |
| NFR-21 | 编码 | UTF-8 全文 |

---

## 5. 约束（Constraints）

### 5.1 治理约束

- per DEC-008：一公司 12 角色，无 RBAC，Web UI 是 1 人工具
- per DEC-005：5 域独立 Lead 兼任禁止（Web UI 需展示 5 域 Lead 独立性）
- per 2026-08-26 08:40 JST：代签已允许新规则（修订历史"审批者"列可由 Mavis 代签"架构师(Ulysses（一人公司 12 角色 per DEC-008）)"）
- per DTL-036 v1.4.2 反馈：禁"per X 历史形态"回溯叙事，引用 BAS 必须 git 实证

### 5.2 技术约束

- Rust 工作区目前无 axum / warp / actix-web 依赖（per `Cargo.toml` workspace.dependencies）
- 加 web 框架 = 改 workspace.dependencies = 影响 5 域 build = 改动面大
- → 选择 **node + 原生 http** 隔离 Rust 工作区

### 5.3 时间约束

- v0.1 30 分钟落地（per Ulysses 12:15 JST 期望"快"）
- v0.2 后续可重写为 Rust（axum + tera 模板）

---

## 6. 验收标准（Acceptance Criteria）

### 6.1 v0.1 验收

- [ ] `node tools/rgs-web/server-no-deps.js` 启动 < 1s
- [ ] `http://127.0.0.1:8788/` 200 OK，HTML 大小 > 5KB
- [ ] `http://127.0.0.1:8788/api/health` 返回 JSON `{status: "ok", k3s: "https://127.0.0.1:6443", time: "..."}`
- [ ] `http://127.0.0.1:8788/api/impl-plan` 返回 8 份 IMPL-PLAN JSON
- [ ] `http://127.0.0.1:8788/api/worktrees` 返回 ≥ 11 个 worktree
- [ ] `http://127.0.0.1:8788/api/docs-health` 返回 1 FAIL + 1 WARN
- [ ] HTML dashboard 显示 5 域 IMPL-PLAN 表格 + 11 commit 表格
- [ ] 30s 自动 refresh 生效

### 6.2 v0.2 验收（待 WSL sudo chmod 644 k3s.yaml 后）

- [ ] `http://127.0.0.1:8788/api/k8s/api/v1/pods` 透传到 k3s
- [ ] `http://127.0.0.1:8788/` Dashboard 显示 5 域 pod phase
- [ ] `http://127.0.0.1:8788/api/k8s/api/v1/nodes` 显示 1 个 node（UlyssesPC）

### 6.3 v1.0 验收（Rust 重写目标）

- [ ] Cargo.toml 加入 axum 0.8
- [ ] 新增 `crates/rgs-web` workspace member
- [ ] 所有 5 个 API endpoint 1:1 移植
- [ ] HTML 模板用 tera 或 askama
- [ ] 部署到 k3s 作为 deployment + service

---

## 7. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| npm install 超时（实测 2min+）| 高 | 启动慢 | v0.1 零依赖版（`server-no-deps.js`） |
| WSL sudo 需密码 | 高 | kubectl 受阻 | Ulysses 手动 + WSL-KUBECONFIG-FIX-2026-08-26.md SOP |
| 8787 端口被 Cursor headroom 占用 | 高 | 启动失败 | 默认改 8788 |
| 中文路径 GBK vs UTF-8 | 中 | 文件读失败 | node 默认 UTF-8 + PowerShell `[System.IO.File]::ReadAllText(... UTF8)` |
| git worktree list 输出被 PowerShell 截断 | 低 | dashboard 缺失 | 前端用 JS fetch + 全部展示，不依赖 shell 截断 |

---

## 8. 不在范围（Out of Scope）

- 5 域真实部署（`deploy_dev_k3s.ps1` 之前 step 1 卡死，per RGS-DOCS-HEALTH-2026-08-26）
- cluster-ops gRPC 调用（Web UI 当前不调 cluster-ops gRPC server）
- 多用户 / RBAC / SSO（一人公司模式）
- WBS v0.8 §2A.2.55.续3 之外的新增 L4 任务
- 真实 SSH 到 WSL 内（仅展示 git / k3s 间接状态）

---

## 9. 验收者（per 2026-08-26 08:40 JST 代签新规则）

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））| 2026-08-26 |
| 5 域 Lead | _待 DDD Review 阶段补签_ | — |
| shared-platform Lead | _待 DDD Review 阶段补签_ | — |
| saga 召集人 | _待 DDD Review 阶段补签_ | — |

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））| 初版：5 用户故事 + 21 功能需求 + 21 非功能需求 + 3 验收阶段 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态：无需求规约
- v0.1 新增：本文档（5 用户故事 + 21 FR + 21 NFR + 6 约束 + 7 风险 + 3 验收阶段）

### A.2 对基本设计的影响

- 触发 RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1 起草
- 5 用户故事 → 5 核心模块（health / impl-plan / worktrees / docs-health / k8s proxy）
- 21 FR → 21 API endpoint / 路由设计

### A.3 已知缺口

- 5 域 Lead / shared-platform / saga 召集人 签字未到（DDD Review 阶段补）
- 6.2 v0.2 验收依赖 Ulysses WSL sudo（per WSL-KUBECONFIG-FIX-2026-08-26.md）
- 6.3 v1.0 验收依赖 Rust 重写（axum 0.8）

### A.4 引用链与证据

- rgs-web commit `5f827ee`（2026-08-26 12:22 JST，7 文件 661 行）
- RGS-WEB-PLAN-2026-08-26 v0.1（设计总览）
- WSL-KUBECONFIG-FIX-2026-08-26.md（WSL 修复 SOP）
- per WBS v0.8 commit `87a6472`（per WF-1-55.77）
- per 11 P0/P1/P2 commit（per RGS-REPORT-2026-08-26-P0P1P2_v0.2）
- per DEC-008 一人公司 12 角色
- per 2026-08-26 08:40 JST 代签新规则
