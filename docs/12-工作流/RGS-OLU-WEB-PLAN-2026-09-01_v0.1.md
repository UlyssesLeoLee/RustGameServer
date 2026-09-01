# RGS-OLU-WEB-PLAN-2026-09-01 v0.1

**Token 消耗可视化子系统设计总览 + 实施方案**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-OLU-WEB-PLAN-2026-09-01 |
| 版本 | 0.1（首版，per Ulysses 2026-09-01 15:44 JST 触发）|
| 状态 | 设计 + 实施计划 + 4 周落地 |
| 触发 | Ulysses 2026-09-01 15:44 JST "创新一套 token 消耗图表，放在甘特图那一套界面中名称为 token 的选项卡里" |
| 责任人 | 架构师（**Mavis 接手 agent per DEC-008**）|
| 关联 | RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1 + RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1 + RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1 + rgs-web 母规范 5 份 |
| 上游基线 | rgs-web v0.3 commit `23d447b`（per RGS-WBS-001 L4 进度表 v0.4 §A.3 git 实证）+ RGS-TS-001 v0.7 §6.2 OLU 双轨制 + RGS-WBS-001 v0.3 §2A 145 L4 任务 |
| 适用许可 | Apache-2.0（本仓库）|

---

## 0. 触发与背景

**触发**：Ulysses 2026-09-01 15:44 JST "创新一套 token 消耗图表，放在甘特图那一套界面中名称为 token 的选项卡里，主要体现各个任务的 token 消耗预算和实际消耗，配套的和任务卡以及 github/gitlab 之间联动，和 ai 之间的联动都要到位，辅助管理 token 消耗。从需求文档设计开始"

**现状**（per 2026-09-01 15:44 JST git 实证）：

| 维度 | 现状 | 数据来源 |
|---|---|---|
| rgs-web | v0.3 真实 6 域 gRPC 接入，10 页面 dashboard，node 22 + http2 + mTLS + 零依赖，127.0.0.1:8788 | `tools/rgs-web/public/index.html` nav 行 98-110 + RGS-WEB-REQUIREMENTS v0.1 |
| WBS | 145 L4 任务（22 done / 124 pending）| RGS-WBS-001 L4 进度表 v0.4 §3 汇总 |
| OLU token 框架 | 1 人·天 ≈ 100K-300K tokens（基线）| RGS-TS-001 v0.7 §6.2.2.1 |
| 上一次实际 token 估算 | 2026-08-27 dev k3s 部署：725K-1.45M tokens（conservative）| RGS-OLU-REPORT-2026-08-27 v0.1 §3.10 |
| mavis runtime | 主会话 + explore/worker/verifier 4 内置子代理 | mavis runtime |
| 凭据注入 | 母规范 v0.3 §9 NFR-11：env var only，不落盘 | rgs-web 母规范 v0.1 §NFR-11 |

**本文档目的**：4 周内落地 rgs-web Token 子系统，1 页面（page-gantt）+ 4 选项卡（Gantt / Tasks / **Token** / AI）+ 9 API + 5 数据文件 + 1 锁文件 + 2 lib helper，**不破坏 rgs-web v0.3 母规范**。

---

## 1. 设计目标（per REQUIREMENTS v0.1 §1.4 业务目标）

| # | 目标 | 度量 | 优先级 |
|---|---|---|---|
| O-1 | Token 实时可见 | 145 L4 任务每条 budget_tokens / actual_tokens / percent_used，30s refresh | P0 |
| O-2 | 任务卡 + Gantt + Token 三联动 | 点 Gantt 任务条 → 切到 Token 选项卡对应 task；深链支持 | P0 |
| O-3 | AI 协作 token 自动入账 | mavis hook + rgs-web 后台轮询，30s 可见 | P0 |
| O-4 | GitHub/GitLab 浅联动 v0.1 | /api/git/integrations + /api/git/issues 只读 + 写回评论 | P0（v0.1 浅联动）|
| O-5 | NFR-OP-010 双轨实时仪表盘 | 顶部固定条，本周 tokens / 20M tokens 比例三态 | P0 |
| O-6 | 5 域 Lead token 分摊 | 仪表盘按域聚合，5 域 + shared-platform + cluster-ops + saga | P0 |

---

## 2. 4 周里程碑

| 周 | 里程碑 | 验收 |
|---|---|---|
| **W1 (2026-09-02 ~ 09-08)** | **架构 + 数据底层** | (a) page-gantt 容器 + 4 选项卡 DOM 落地 (b) data/ai-ledger.jsonl + git-ledger.jsonl 写入管线跑通 (c) lockfile 工具 UT 通过 |
| **W2 (2026-09-09 ~ 09-15)** | **核心 API + UI** | (a) 9 API endpoint 实现 + 集成测试 (b) Token 选项卡双柱图 SVG 渲染 (c) 5 域堆叠图 + NFR-OP-010 计数器 |
| **W3 (2026-09-16 ~ 09-22)** | **联动 + 反向** | (a) 任务卡 ↔ Token 双向跳转 (b) GitHub 浅联动（拉取 + 写回评论） (c) mavis runtime hook 集成（per mavis skill §hook management）|
| **W4 (2026-09-23 ~ 09-29)** | **运维 + 验收** | (a) 监控项 + 故障恢复 + 数据备份 (b) 凭据泄露测试 + 端到端测试 (c) 修订历史代签 + DDD Review 准备 |

**总工作量估算**：~4 周（per REQUIREMENTS §7.3 时间约束）

**token 估算**（per RGS-TS-001 v0.7 §6.2.2.1）：
- 1 人·周 ≈ 1M tokens
- 4 周 ≈ 4M tokens（per v0.5 算法） / 5.6-13.4M tokens（per v0.6 双轨制）
- 待 RGS-ENV-CALIB-001 校准

---

## 3. 技术选型（per BASIC-DESIGN §2 + DETAILED-DESIGN §1-§6）

### 3.1 决策表（10 新增决策 + 9 不选方案）

| 维度 | 选择 | 备选 | 决策理由 |
|---|---|---|---|
| Gantt 渲染 | **原生 SVG + CSS 定位** | dhtmlx-gantt / frappe-gantt | 零依赖 |
| Token 双柱图 | **原生 SVG bar + 数字** | chart.js / d3 | 沿用母规范 §2.1 |
| 数据存储 | **JSON + jsonl append-only + lockfile** | SQLite / markdown frontmatter | 零依赖 + 1 写者 |
| Token 估算 v0.1 | **message_count × 5K tokens/条** | provider counter 真实值 | 估算公式 per RGS-OLU-REPORT §3.2 |
| GitHub / GitLab 客户端 | **node:https 原生** | octokit / @gitbeaker | 零依赖 |
| 凭据注入 | **env var only** | 配置文件 | per 2026-08-27 11:06 JST 硬 ban |
| mavis hook | **mavis skill §hook management** | 子进程 | per mavis skill 文档 |
| 数据刷新 | **30s 轮询** | WebSocket / SSE | 沿用母规范 §2.1 |
| 锁文件 | **fs.openSync(path, 'wx')** | proper-lockfile | 零依赖 + 原子 |
| 写并发 | **lockfile + retry 3 次** | SQLite / 文件锁 | 1 写者约束 |

### 3.2 不选方案

| 方案 | 不选理由 |
|---|---|
| better-sqlite3 | native binding 编译慢 |
| markdown frontmatter 存 token | 解析成本高，不能聚合 |
| chart.js / d3 | 145 任务 SVG 手写足够 |
| WebSocket | 30s 轮询够用 |
| Docker 部署 | 一人公司本机工具 |
| 登录 / RBAC | DEC-008 一人公司 |
| Rust axum 重写 | 母规范 v1.0 目标 |
| nginx 反代 | per 2026-09-01 13:03 JST 偏好 envoy 独立 deployment |
| mavis hook 阻塞 | v0.1 降级为 mavis session list 拉历史 + 估算公式 |

---

## 4. 实施分解（WBS 4 周 L4 任务）

> **本节是 v0.1 落地的实际 WBS L4 任务清单**，per RGS-WBS-001 v0.3 §2A.2 拆分原则（每个 L4 任务 = 1 人/agent 最小可拆分单位，≤ 2 人·天 或 ≤ 500K tokens）。

### 4.1 W1 任务（5 任务 / ~3.5 人·天 / ~700K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| OL-W1-1 | page-gantt 容器 + 4 选项卡 DOM（沿用 rgs-web v0.3 母规范 CSS 变量）| 架构师（Mavis 接手 agent per DEC-008）| 0.5 | 100K | 无 | DOM 渲染 < 1s / 4 tab 切换正常 | revert commit | `wbs/OL-W1-1` |
| OL-W1-2 | lib/lockfile.js（fs.openSync + retry 3 次）| 架构师 | 0.5 | 100K | 无 | UT 100 并发 1 成功 99 失败 | revert | `wbs/OL-W1-2` |
| OL-W1-3 | lib/token-estimate.js（message_count × 5K + 95% 分位）| 架构师 | 0.5 | 100K | 无 | UT 50 case 全过 | revert | `wbs/OL-W1-3` |
| OL-W1-4 | lib/mavis-bridge.js（execSync mavis agent/session list + 30s 缓存）| 架构师 | 1.0 | 200K | OL-W1-2 | mock 返样本断言解析 | revert | `wbs/OL-W1-4` |
| OL-W1-5 | WBS L4 进度表 budget_tokens 字段扩展（不破坏格式，仅追加列）| 架构师 | 1.0 | 200K | 无 | v0.X 进度表 §4 表格列数 +2，cargo check 通过 | revert | `wbs/OL-W1-5` |

### 4.2 W2 任务（6 任务 / ~7 人·天 / ~1400K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| OL-W2-1 | /api/token/summary（聚合 ai-ledger + git-ledger + WBS）| 架构师 | 1.0 | 200K | OL-W1-3, OL-W1-4, OL-W1-5 | 响应 < 200ms / 145 任务 + 8 域 | revert | `wbs/OL-W2-1` |
| OL-W2-2 | /api/token/budget-vs-actual（双柱图数据 + P95 + 异常）| 架构师 | 1.0 | 200K | OL-W2-1 | 响应 < 100ms | revert | `wbs/OL-W2-2` |
| OL-W2-3 | /api/token/by-domain（5 域 + shared + cluster + saga 分摊）| 架构师 | 0.5 | 100K | OL-W2-1 | 响应 < 100ms | revert | `wbs/OL-W2-3` |
| OL-W2-4 | /api/token/nfr-op-010（本周 7 天 + 双轨 + 三态）| 架构师 | 1.0 | 200K | OL-W2-1 | 响应 < 200ms / 三态颜色对 | revert | `wbs/OL-W2-4` |
| OL-W2-5 | /api/ai/ledger + /api/ai/sessions + /api/ai/agents | 架构师 | 1.0 | 200K | OL-W1-4 | 3 endpoint 全过 | revert | `wbs/OL-W2-5` |
| OL-W2-6 | Token 选项卡 UI（双柱图 SVG + 5 域堆叠 + 顶部 NFR 计数器 + 30s 轮询）| 架构师 | 2.5 | 500K | OL-W2-1, OL-W2-4 | 页面 < 3s / 渲染 145 任务 | revert | `wbs/OL-W2-6` |

### 4.3 W3 任务（5 任务 / ~6.5 人·天 / ~1300K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| OL-W3-1 | Tasks 选项卡（145 任务表格 + 深链 task_id 高亮）| 架构师 | 1.0 | 200K | OL-W1-5 | 表格 < 1s / 深链 `?task_id=` 命中 | revert | `wbs/OL-W3-1` |
| OL-W3-2 | Gantt 选项卡（145 任务按 L1 阶段分组 SVG 时间线）| 架构师 | 1.5 | 300K | OL-W3-1 | 渲染 < 1s | revert | `wbs/OL-W3-2` |
| OL-W3-3 | 任务卡 ↔ Token 双向跳转（弹窗 + WBS 源文件锚点）| 架构师 | 1.0 | 200K | OL-W2-6, OL-W3-1 | 弹窗显示 budget/actual/sessions/commits | revert | `wbs/OL-W3-3` |
| OL-W3-4 | /api/git/integrations + /api/git/issues 拉取 + 写回评论 | 架构师 | 2.0 | 400K | OL-W2-6 | 401/403/404 错误码正确，token 不暴露 | revert | `wbs/OL-W3-4` |
| OL-W3-5 | mavis runtime hook 集成（写 ai-ledger.jsonl）| 架构师 | 1.0 | 200K | OL-W1-2, OL-W1-4 | session finish → ledger 30s 内可见 | revert | `wbs/OL-W3-5` |

### 4.4 W4 任务（5 任务 / ~5 人·天 / ~1000K tokens）

| L4 # | 任务描述 | owner | 人·天 | token/周 | 前置 | 验收项 | 回滚路径 | worktree |
|---|---|---|---:|---:|---|---|---|---|
| OL-W4-1 | 监控项（5 指标）+ 故障恢复（5 场景）+ 数据备份脚本 | 架构师 | 1.0 | 200K | OL-W2-1, OL-W3-4 | 5 监控 + 5 恢复 + 1 备份脚本 | revert | `wbs/OL-W4-1` |
| OL-W4-2 | 凭据泄露测试（响应 / 日志 / 错误信息搜索 token 0 命中）| 架构师 | 0.5 | 100K | OL-W3-4 | 0 命中（GITHUB_TOKEN / Bearer）| revert | `wbs/OL-W4-2` |
| OL-W4-3 | 端到端测试（page-gantt → 4 选项卡 → 弹窗 → 写回评论）| 架构师 | 1.0 | 200K | OL-W3-3, OL-W3-4 | 4 步骤全过 | revert | `wbs/OL-W4-3` |
| OL-W4-4 | AI 选项卡（session + ledger 表格 + 估算公式标注）| 架构师 | 1.0 | 200K | OL-W2-5, OL-W3-5 | 表格 < 1s / 估算公式 UI 可见 | revert | `wbs/OL-W4-4` |
| OL-W4-5 | 修订历史代签 + DDD Review 准备（4 份 v0.1 文档 + 签字栏）| 架构师 | 1.5 | 300K | 全部 W1-W3 任务 | 4 份文档 v0.1 签字栏就位 | revert | `wbs/OL-W4-5` |

### 4.5 总工作量

| 周 | L4 任务数 | 人·天 | token/周（v0.5 算法 200K/人·天）| token/周（v0.6 算法 100K-300K/人·天）|
|---|---:|---:|---:|---:|
| W1 | 5 | 3.5 | 700K | 350K-1050K |
| W2 | 6 | 7.0 | 1400K | 700K-2100K |
| W3 | 5 | 6.5 | 1300K | 650K-1950K |
| W4 | 5 | 5.0 | 1000K | 500K-1500K |
| **合计** | **21** | **22.0** | **4400K** | **2200K-6600K** |

> **NFR-OP-010 双轨校验**（per RGS-TS-001 v0.7 §6.2.4 + RGS-OLU-REPORT-2026-08-27 v0.1 §6）：
> - 人·天轨：22 人·天 / 4 周 = 5.5 人·天/周 ≤ 20 ✓ 绿
> - token 轨：4.4M / 4 周 = 1.1M tokens/周 ≤ 20M ✓ 绿
> - 留足余量（v0.6 算法下界 2.2M / 4 周 = 550K tokens/周 = 2.75% NFR 上限）

---

## 5. 风险与缓解

| # | 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|---|
| R-1 | mavis runtime hook 集成阻塞 | 中 | AI 选项卡无可用数据 | v0.1 降级：`mavis session list --json` 30s 拉历史 + 估算公式 |
| R-2 | GitHub rate limit | 中 | 写回 issue 评论 403 | 10 min 缓存 + 退避重试 + User-Agent 标识 |
| R-3 | 145 L4 任务 budget_tokens 推算偏差 | 高 | 仪表盘读数误导 | UI 强制标 "estimated"（per NFR-27），推算公式公开 |
| R-4 | 5 域 binary 未来调外部 LLM 未登记 | 中 | token 漏算 | 预留 IR-7 hook，5 域 Cargo.toml 加 `rgs-otel` 出口点 |
| R-5 | 中文路径 mojibake | 低 | 读 WBS 文件失败 | 沿用母规范 §NFR-9，node 默认 UTF-8 |
| R-6 | 凭据泄露 | 中 | GitHub 私仓暴露 | 2026-08-27 11:06 JST 硬 ban + 强校验（per DETAILED-DESIGN §5.1）|
| R-7 | user_profile 127.0.0.1 only 与 webhook 冲突 | 高 | webhook 不可达 | v0.3 才做 webhook（per 7.3 + F-W3），v0.1/v0.2 仅 outbound |
| R-8 | Gantt SVG 渲染 145 任务性能 | 中 | 页面卡顿 | 145 任务单页 < 3s（per NFR-22），分页 / 虚拟滚动备选 |
| R-9 | worker 派工后 cargo check 超时（per AGENTS.md §2.1 L1+L2 合并）| 高 | worker 失败 | W1-W4 任务单人主会话执行（per AGENTS.md §2.4 L4 "跨多工具链场景先主会话打头阵"），不派 worker |
| R-10 | 跨工具链决策前未 grep（per AGENTS.md §2.3 L3）| 中 | 假设错误 | W1 OL-W1-4 mavis bridge 实现前先 `which mavis` + `mavis --help` 实证 |
| R-11 | 决策点 Ulysses 不在场（per 2026-09-01 14:58 JST 拍板必须用选项）| 中 | 卡进度 | 关键决策点（4 个）W1 / W2 末 / W3 末 / W4 末 各用 ask_user 一次 |

---

## 6. 跨会话恢复 SOP

> per RGS-WT-001 §11.3 跨会话恢复 + AGENTS.md §2.4 L4 主会话打头阵。

**W1 任务中断恢复**：
1. `git worktree list` 查 OL-W1-* worktree 状态
2. 读 `.wbs-task-marker` 找当前 status
3. 继续推进，调 `wbs_task_progress.ps1 -Status progress -Progress N` 同步

**W2-W4 任务中断恢复**：同 W1

**4 周主会话打头阵原则**（per AGENTS.md §2.4 L4）：
- W1-W4 全部 21 任务主会话自执行，**不**派 worker（per AGENTS.md §2.4 L4 + L5）
- 单任务执行超过 60s 仍无进展，回退到 WBS 状态 = blocked + 上报 Ulysses

---

## 7. 启动 SOP

### 7.1 v0.1 启动

```bash
# 1. (一次性) 创建 worktree 主分支
git worktree add -b wbs/olu-web-v0.1 D:/.worktrees/olu-web-v0.1 main

# 2. (一次性) 配 mavis runtime hook
#    编辑 C:\Users\leo19\.minimax\agents\mavis\hooks\oludash-write-ledger.js
#    + mavis config 加 hook (per DETAILED-DESIGN §3.2)

# 3. (一次性) 配 env var (per 2026-08-27 11:06 JST env value hard ban, 不 echo)
$env:RGS_WEB_PORT = 8788
$env:GITHUB_TOKEN = '<PAT>'  # 不要 echo!
$env:GITHUB_REPO = 'ulyssesleolee/RustGameServer'
$env:GITLAB_TOKEN = '<PAT>'  # 可选
$env:GITLAB_PROJECT_ID = '123'

# 4. 启动 rgs-web (v0.1 已含 11 页面 + 4 选项卡 + 9 API)
cd D:/RustGameServer
node tools/rgs-web/server.js
# 访问 http://127.0.0.1:8788/?page=gantt&task_id=WF-1-55.27&tab=token
```

### 7.2 验证清单

- [ ] `node tools/rgs-web/server.js` 启动 < 1s
- [ ] `http://127.0.0.1:8788/?page=gantt` 渲染 4 选项卡
- [ ] Token 选项卡双柱图渲染 145 任务
- [ ] 顶部 NFR-OP-010 计数器绿/黄/红三态正确
- [ ] 5 域堆叠图 + 颜色对应
- [ ] AI 选项卡 ai-ledger.jsonl 表格
- [ ] 30s 自动 refresh
- [ ] 深链 `?page=gantt&task_id=WF-1-55.27&tab=token` 正确打开
- [ ] 凭据泄露测试 0 命中

---

## 8. 验收者（per 2026-08-26 08:40 JST 代签新规则）

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
| v0.1 | 2026-09-01 | 架构师（**Mavis 接手 agent per DEC-008**）| 首版：4 周里程碑 + 10 技术决策 + 9 不选方案 + 21 L4 任务（W1 5 + W2 6 + W3 5 + W4 5）+ 11 风险 + 跨会话恢复 SOP + 启动 SOP + NFR-OP-010 双轨校验 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态：rgs-web v0.3 无 Gantt / Token / AI 子系统
- v0.1 新增：本文档
- v0.1 4 周落地：21 L4 任务 / 22 人·天 / 4.4M tokens

### A.2 文档三件套完成

- ✅ RGS-OLU-WEB-REQUIREMENTS-2026-09-01 v0.1（需求规约层）
- ✅ RGS-OLU-WEB-BASIC-DESIGN-2026-09-01 v0.1（基本设计层）
- ✅ RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01 v0.1（详细设计层）
- ✅ **RGS-OLU-WEB-PLAN-2026-09-01 v0.1（设计总览 + 实施计划，本文）**

### A.3 已知缺口

- 5 域 Lead / shared-platform / cluster-ops / SRE / PM 签字未到（DDD Review 阶段补）
- mavis runtime hook 集成未确认（per §5 R-1 + REQUIREMENTS §9 风险 1）
- GITHUB_TOKEN / GITLAB_TOKEN 注入路径未确认（per REQUIREMENTS §6.2 v0.2 验收）
- RGS-ENV-CALIB-001 校准数据未生成（per RGS-OLU-REPORT-2026-08-27 v0.1 §10 GAP-1/2/3）
- 4 周落地主会话自执行，不派 worker（per AGENTS.md §2.4 L4 + §5 R-9）
- 关键决策点 4 次 ask_user（per §5 R-11 + 2026-09-01 14:58 JST 拍板必须用选项）

### A.4 引用链与证据

- rgs-web 母规范 5 份文档（per `docs/12-工作流/RGS-WEB-*.md`）
- RGS-TS-001 v0.7 §6.2 OLU 双轨制
- RGS-WBS-001 v0.3 §2A 145 L4 任务 + L4 进度表 v0.4
- RGS-OLU-REPORT-2026-08-27 v0.1 §3 token 估算公式 + §6 双轨校准
- mavis skill §hook management
- per DEC-008 一人公司 12 角色
- per 2026-08-26 08:40 JST Mavis 默认代签 Ulysses
- per 2026-08-27 11:06 JST env value hard ban
- per 2026-09-01 13:03 / 13:05 JST envoy 独立 deployment 偏好
- per 2026-09-01 14:58 JST 拍板决策必须用选项
- per AGENTS.md §2.1 L1+L2 cargo check --tests 验证
- per AGENTS.md §2.3 L3 跨工具链决策前先 grep
- per AGENTS.md §2.4 L4 跨多工具链场景先主会话打头阵
- per AGENTS.md §2.5 L5 ST worktree 启动 checklist（v0.1 不走 ST，参考价值低）

---

## 附：v0.1 文档三件套完整索引

| 文档 | 路径 | 行数估算 |
|---|---|---:|
| 需求规约 | `docs/12-工作流/RGS-OLU-WEB-REQUIREMENTS-2026-09-01_v0.1.md` | ~400 行 |
| 基本设计 | `docs/12-工作流/RGS-OLU-WEB-BASIC-DESIGN-2026-09-01_v0.1.md` | ~300 行 |
| 详细设计 | `docs/12-工作流/RGS-OLU-WEB-DETAILED-DESIGN-2026-09-01_v0.1.md` | ~350 行 |
| 计划总览 | `docs/12-工作流/RGS-OLU-WEB-PLAN-2026-09-01_v0.1.md` | ~250 行（本文）|
| **合计** | | **~1300 行** |

### A.5 v0.2 升版增量（per Ulysses 4 + 2 ask_user 决策，2026-09-01 16:41 JST）

> **v0.1 主体不追溯改写**。v0.2 增量 = 5 大块，落地到 PLAN 各章节：

**1. GitHub/GitLab 浅联动 → 深联动 webhook inbound**（per ask_user 16:30 JST）
- §3.1 决策（webhook 端点 + 验签 + 重放保护）
- §4.2 OL-W2-7（webhook 端点）
- §4.3 OL-W3-6（E2E 实测）

**2. better-sqlite3 存储 + 备份清理 batch**（per ask_user 16:30/16:41 JST）
- §3.1 决策（数据存储 + SQLite 6 表 + 备份 batch）
- §4.1 OL-W1-2（sqlite.js）
- §4.4 OL-W4-5（backup-batch.js）
- §4.5 总工作量重新核算

**3. cloudflared tunnel 解 webhook + 127.0.0.1 only 冲突**（per ask_user 16:41 JST）
- §3.1 决策（webhook 端点 cloudflared）
- §4.3 OL-W3-4（cloudflared.js）
- §7.1 启动 SOP（装 cloudflared）

**4. webhook 验签 + 重放保护**（per F-32/F-33）
- §4.1 OL-W1-6（webhook-verifier.js）
- §4.2 OL-W2-7（webhook 端点）

**5. 备份 batch**（per ask_user "详细的记录备份清理 batch"）
- §4.4 OL-W4-5（backup-batch.js）
- §7.1 启动 SOP（cron 配置）

**v0.2 总工作量**（per §4.5 重新核算）：
- 26 L4 任务（v0.1 21 + v0.2 增 5：sqlite.js / webhook-verifier.js / webhook 端点 / cloudflared.js / E2E / backup-batch.js）
- 25.5 人·天（v0.1 22 + v0.2 增 3.5）
- 5.1M tokens（v0.1 4.4M + v0.2 增 0.7M）
- NFR-OP-010 双轨校验：人·天轨 6.4/周（绿 ≤ 20）+ token 轨 1.3M/周（绿 ≤ 20M）

**v0.2 风险**（per §5 重新核算，11 → 16）：
- R-12 better-sqlite3 native binding 编译失败
- R-13 cloudflared 二进制未装
- R-14 cloudflared tunnel 公开 URL 泄露
- R-15 SQLite 写并发死锁
- R-16 备份 batch 失败

**v0.1 风险 R-7 已缓解**：127.0.0.1 only 与 webhook 冲突 → cloudflared tunnel 解冲突

**派生决策引用**：per 2026-09-01 14:58 JST 拍板决策必须用选项 + per "Never auto-install software" 硬约束

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-01 15:44 JST | 架构师（**Mavis 接手 agent per DEC-008**）| 首版（commit `a896ca9`）|
| **v0.2** | **2026-09-01 16:41 JST** | **架构师（**Mavis 接手 agent per DEC-008**）** | **v0.2 升版**：① 11 决策 + 13 不选（v0.1 10 + 9）② 26 L4 任务（v0.1 21）③ 25.5 人·天（v0.1 22）④ 5.1M tokens（v0.1 4.4M）⑤ 16 风险（v0.1 11 + 5）⑥ NFR-OP-010 双轨仍绿 |
