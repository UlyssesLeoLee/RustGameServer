# RGS-WEB-GM-PLAN-2026-08-26 v0.1

**RGS GM 后台总体设计 + 模块清单（参考 E:/ROPE_CS）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WEB-GM-PLAN-2026-08-26 |
| 版本 | 0.1（per Ulysses 2026-08-26 13:13 JST "参考 E:/ROPE_CS 的 GM 后台来改善"）|
| 状态 | 设计 + v0.2-gm 已落地（10 页面 + 6 API + dark theme）|
| 触发 | 2026-08-26 12:15 JST "RGS 需要一个网页端的后台" + 13:13 JST "参考 E:/ROPE_CS 改善" |
| 关联 | RGS-WEB-PLAN-2026-08-26 v0.1（总览）+ RGS-WEB-REQUIREMENTS-2026-08-26 v0.1（需求）|
| 责任人 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））|

---

## 0. 文档定位

本文档是 RGS GM Admin Web（rgs-web v0.2-gm）的**模块清单 + 横向对比 E:/ROPE_CS GM 平台 + 落地状态**。回答"做了什么 / 没做 / 未来要做什么"。

参考的 E:/ROPE_CS GM 平台是**完整 React + TypeScript + Vite + chart.js + Flask 后端**的成熟 GM 后台,19 页面 30+ 组件。RGS 当前用 zero-deps node 模拟,**保留 10 个核心页面**作为 v0.2-gm 起步,后续可换 React 重写。

---

## 1. E:/ROPE_CS GM 平台 模块清单（参考）

| # | 页面 | 功能 |
|---|---|---|
| 1 | **Dashboard** | 4 个 SummaryCard + ActiveUsersChart + RouteErrorBudgetPanel + NeonAurora 装饰 |
| 2 | **Players** | 玩家列表 / 详情 / 查 / 改 / 封禁 / 踢 |
| 3 | **Items** | 道具查询 / 修改 / 发放 / 收回 |
| 4 | **Mall** | 商城管理 / 商品上下架 / 限购 |
| 5 | **Servers** | 服务器列表 / 状态 / 启动 / 停止 / 灰度 |
| 6 | **HotUpdate** | 热更 / 灰度发布 / 版本回滚 |
| 7 | **ConfigCenter** | 配置中心 / 热更新 / 多环境 |
| 8 | **OperationsSql** | SQL 查询 / 数据修复（白名单）|
| 9 | **Reports** | 报表导出 / PDF / Excel |
| 10 | **Support** | 客服工作台 / 工单 |
| 11 | **SystemHealth** | 系统监控 / 指标 / 告警 |
| 12 | **TaskGroupBuilder** | 任务组编排 / 触发器 |
| 13 | **Accounting** | 财务 / 流水 / 对账 |
| 14 | **PaymentAnalytics** | 支付分析 / 渠道 |
| 15 | **PermissionManagement** | 权限管理 / RBAC |
| 16 | **Login** | 登录页 |
| 17 | **OaApprovals** | OA 审批流 |
| 18 | **StreamMonitor** | 实时流监控 |
| 19 | **Canvas** | 自定义仪表盘 |

ROPE_CS 技术栈：
- 前端：React 18 + Vite 5 + chart.js 4 + react-chartjs-2 + react-router 7
- 后端：Flask + protobuf + WebSocket（control plane）
- 数据库：SQLite（控制平面）+ PostgreSQL（业务平面）
- 部署：Flask + Vite dev server

---

## 2. RGS v0.2-gm 模块清单（已落地）

RGS v0.2-gm 用 **zero-deps node + 原生 HTML + CSS 变量** 实现,**10 核心页面**对应 ROPE_CS 的 1-9 + 12:

| # | 页面 | 对应 ROPE_CS | 实现方式 | 落地状态 |
|---|---|---|---|---|
| 1 | **📊 Dashboard** | Dashboard | 4 stat 卡片 + IMPL-PLAN 进度条 + 11 commit 表 | ✅ v0.2 |
| 2 | **🖥️ Servers** | Servers | 3 table(node/pod/deployment) + k8s 代理 | ✅ v0.2 |
| 3 | **👥 Players** | Players | mock 数据 + view / ban 按钮 + 详情面板 | ✅ v0.2(mock) |
| 4 | **📡 Live Console** | StreamMonitor | setInterval 模拟日志流(待 SSE/WS) | ✅ v0.2(mock) |
| 5 | **⚙️ Config Center** | ConfigCenter | 5 域 IMPL-PLAN 列表 + ConfigMap 列表 | ✅ v0.2(只读) |
| 6 | **🔄 Hot Update** | HotUpdate | PFAU 阶段表 + 7 天 git log + Helm chart 注释 | ✅ v0.2 |
| 7 | **🗃️ Operations SQL** | OperationsSql | 5 域 DB 选择 + SQL 输入 + 拦截规则 | ✅ v0.2(只读 mock) |
| 8 | **📚 Docs & Health** | SystemHealth | 3 卡片 + check-docs 输出 + 文档清单 | ✅ v0.2 |
| 9 | **🌿 Worktrees** | (对应 Servers 子集) | 45 worktree + 过滤 + 标注 | ✅ v0.2 |
| 10 | **📈 Reports** | Reports | RGS-REPORT-* 列表 + 导出占位 | ✅ v0.2 |

**10/10 已实现**(部分 mock,部分需要 k3s 代理通)

---

## 3. RGS v0.2-gm 暂未实现（v0.3+ 计划）

### 3.1 ROPE_CS 有但 RGS 暂不做的

| # | 页面 | 暂不做理由 | 后续 v0.3+ 计划 |
|---|---|---|---|
| 11 | Items | RGS 道具管理走 gRPC + saga,Web UI 不直接接 | v0.5 (待 player/economy gRPC client) |
| 12 | Mall | 同 Items | v0.5 |
| 13 | Support | 一人公司无客服工单 | 不做 |
| 14 | TaskGroupBuilder | 一人公司无运营组编排 | 不做 |
| 15 | Accounting | RGS 走 ledger 系统,Web UI 暂不直查 | v0.5 (per DTL-015) |
| 16 | PaymentAnalytics | RGS 暂未接第三方支付 | v1.0 |
| 17 | PermissionManagement | 一人公司无 RBAC(per DEC-008) | 不做 |
| 18 | Login | 一人公司无登录 | 不做 |
| 19 | OaApprovals | 一人公司无 OA 审批流 | 不做 |
| 20 | Canvas | 自定义仪表盘 | 不做 |

### 3.2 v0.3 计划新增

| # | 功能 | 描述 | 优先级 |
|---|---|---|---|
| V0.3-1 | **5 域 gRPC client** | 调 player-service / economy-service / match-service / social-service / admin-service 的 gRPC | P0 |
| V0.3-2 | **cluster-ops gRPC** | 调 cluster-ops 6444 HealthCheck + GetNode | P0 |
| V0.3-3 | **真实 player 查/改** | GetPlayer / BanPlayer / KickPlayer | P0 |
| V0.3-4 | **真实 5 域 pod 状态** | 经 k3s 代理 + WSL sudo chmod 644 | P0 |
| V0.3-5 | **WebSocket 实时日志** | 替 setInterval mock,接 cluster-ops 日志流 | P1 |
| V0.3-6 | **chart.js 趋势图** | 7 天 pod restart / 1 天 GM 操作次数 | P2 |
| V0.3-7 | **PDF 报表导出** | 5 域日报 / 周报 | P2 |

### 3.3 v0.5 计划(完成核心)

- Items / Mall / Accounting 接入 saga 域
- Payment 接第三方支付 SDK
- Permission 假 RBAC(一公司模式,DEC-008)

### 3.4 v1.0 计划(Rust 重写)

- 新增 `crates/rgs-web` workspace member
- axum 0.8 + askama 模板
- 部署到 k3s 作为 deployment + service
- WebSocket 接 cluster-ops 日志

---

## 4. 横向对比

| 维度 | E:/ROPE_CS | RGS v0.2-gm |
|---|---|---|
| 前端框架 | React 18 + Vite 5 | 原生 HTML + CSS 变量 |
| 状态管理 | React Context + useState | 单一 fetch + 30s 轮询 |
| 图表库 | chart.js 4 + react-chartjs-2 | CSS bar + 数字统计 |
| 路由 | react-router 7 | 单页 + tab 切换 |
| 后端 | Flask + protobuf | node http + git exec |
| 数据库 | SQLite (config) + PG (data) | PG 18.6 (per RGS-ARC-008) |
| 实时通信 | WebSocket | setInterval mock (v0.3 → WS) |
| 部署 | Vite dev + Flask | node process 127.0.0.1:8788 |
| 依赖 | 100+ npm packages | **0**(零依赖) |
| 启动 | npm install 2 分钟+ | node 直接启动 1s |
| 总代码量 | 10000+ 行 React + 5000+ 行 Flask | 0 行(纯 HTML 32KB)+ 200 行 node |

**RGS v0.2-gm 哲学**：**先 mock 后实现**——所有 10 页面有完整 UI + mock 数据,真接口(v0.3 接入)只换 API endpoint 不改 UI。

---

## 5. 关键决策

### 5.1 零依赖 vs React 重写

**v0.2 选零依赖**:
- npm install 实测 2 分钟+(Windows 网络)
- node 22 自带 http + fetch 足够
- 30s 轮询(非 WS)满足 DDD Review 阶段需求
- 一人公司不需要复杂 state management

**v1.0 再换 React**:
- 5 域 + cluster-ops 真实接入后,需要 WebSocket + 实时数据
- React + chart.js 才能上 dashboard
- 部署到 k3s,与 5 域 pod 一起 scale

### 5.2 Mock vs 真实数据

**Dashboard / Worktrees / IMPL-PLAN / git log**:真实(直接读文件/git)
**Players / Stream / Operations SQL**:mock(v0.3 接入 player-service gRPC)
**Servers (k8s API)**:代理(需 K3S_TOKEN = WSL sudo chmod 644)

### 5.3 RBAC / Login

**不做**(一人公司 per DEC-008):
- 监听 127.0.0.1 only(不暴露 0.0.0.0)
- 无 cookie / session
- 无 CSRF
- 无 XSS 风险(零依赖 + HTML 字符串拼接)

---

## 6. 验收标准

### 6.1 v0.2-gm 验收(本版本)

- [x] 10 页面全部有 UI(HTML 32KB)
- [x] 6 API endpoint 全部 200
- [x] dark theme + responsive
- [x] 30s auto refresh
- [x] git worktree 列表(45)真实
- [x] 11 P0/P1/P2 commit 列表真实
- [x] 8 份 IMPL-PLAN 跨 worktree 真实
- [x] chart bar 简单可视化

### 6.2 v0.3 验收

- [ ] 5 域 gRPC client 接通
- [ ] 真实 player 查/改(GET player by id,POST ban)
- [ ] 真实 5 域 pod phase(经 k3s 代理)
- [ ] cluster-ops 6444 健康探针
- [ ] Operations SQL 真实查询(saga 域白名单)
- [ ] WebSocket 日志流(每 1s 推送)

### 6.3 v1.0 验收

- [ ] Rust axum 重写 + askama 模板
- [ ] `crates/rgs-web` 加入 workspace
- [ ] k3s deployment + service 部署
- [ ] 5 域 pod 启动时启动 1 个 rgs-web pod
- [ ] chart.js 趋势图
- [ ] PDF 报表导出

---

## 7. 当前 rgs-web v0.2-gm 状态

| 项 | 状态 |
|---|---|
| HTTP server 跑 | ✅ PID 14572 on 127.0.0.1:8788 |
| /api/health | ✅ rgs_web_version=0.2.0-gm, 10 页面 |
| /api/impl-plan | ✅ 8 份 (跨 worktree 去重) |
| /api/worktrees | ✅ 45 worktree (含 locked 标注) |
| /api/docs-health | ✅ 1 FAIL + 1 WARN |
| /api/git-log | ✅ 最近 30 commit |
| /api/saga-trace | ✅ saga 关键字 20 commit |
| /api/k8s/* | ⚠️ 需 K3S_TOKEN(WSL sudo chmod 644) |
| / (Dashboard) | ✅ 31068 bytes HTML |

**当前 Dashboard 显示**:
- 4 stat 卡片(5 域 IMPL-PLAN / 11 P0/P1/P2 / 45 worktree / 1 FAIL+1 WARN)
- 5 域 IMPL-PLAN 进度条(8 份, 按 KB 大小)
- 文档健康基线
- 11 P0/P1/P2 commit 表

**Servers / Players / Live Console 切换**:
- Servers: 经 k3s 代理,需 K3S_TOKEN
- Players: mock 数据(WSL 阻塞),等 v0.3 接入
- Live Console: setInterval 模拟日志,等 v0.3 接 WebSocket

---

## 8. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 8787/8788 端口冲突 | 高 | rgs-web 启动失败 | `RGS_WEB_PORT` 环境变量改 |
| k3s 代理需 K3S_TOKEN | 高 | 5 域 pod 看不到 | WSL-KUBECONFIG-FIX-2026-08-26.md SOP |
| /api/k8s 超时 5s | 中 | k8s 资源卡顿 | client timeout 调长 + UI 显示 timeout 状态 |
| ROPE_CS 19 页面 100% 对齐 | 低 | 永远不可达(人公司无 RBAC) | v0.5 选 5 核心, v1.0 5 域 + cluster-ops |
| Windows PowerShell 编码问题 | 中 | 路径含中文 get 失败 | plumbing 路径(git hash-object + update-index) |

---

## 9. 不在范围(Out of Scope)

- 5 域真实 gRPC 接入(等 v0.3)
- cluster-ops gRPC 接入(等 v0.3)
- Rust 重写(等 v1.0)
- 部署到 k3s(等 v1.0)
- WebSocket 日志(等 v0.3)
- 第三方支付 / RBAC / 多用户

---

## 10. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）)| 初版:横向对比 ROPE_CS 19 页面 + RGS 10 页面 v0.2-gm 落地 |

## A. v0.1 升版增量

### A.1 源 rgs-web v0.1 → v0.2-gm

- v0.1: 4 endpoint + 1 页面 (DASHBOARD)
- v0.2-gm: 6 endpoint + 10 页面 (Dashboard / Servers / Players / Live Console / Config / Hot Update / Operations SQL / Docs / Worktrees / Reports)
- HTML 大小: 6117 bytes → 31068 bytes (+ 409%)
- API endpoint: 4 → 6 (+ 50%)
- 总代码量: 4 KB node + 6 KB HTML = 10 KB → 6.4 KB node + 32 KB HTML = 38 KB

### A.2 对 RGS 治理的影响

- DDD Review 阶段 5 域 Lead 可在 1 个网页内集中审 11 P0/P1/P2 commit + 8 IMPL-PLAN
- 不影响 5 域 Rust 后端(独立进程,独立端口)
- 阻塞项 k3s 代理(需 Ulysses WSL sudo chmod 644)

### A.3 已知缺口

- 5 域 gRPC 客户端(等 v0.3)
- cluster-ops gRPC 客户端(等 v0.3)
- WebSocket 日志流(等 v0.3)
- 真实 Players 数据(等 v0.3)
- Operations SQL 真实查询(等 v0.3,需 kubectl exec)
- 10 个 ROPE_CS 高级页面(Payment / RBAC / Login / OA / Canvas)永不做(一人公司无需求)

### A.4 引用链与证据

- 当前 rgs-web PID 14572, 127.0.0.1:8788
- rgs-web 源码:`tools/rgs-web/{server-no-deps.js, public/index.html}`
- E:/ROPE_CS GM 平台:19 pages (React + Vite + chart.js)
- WBS v0.8 commit `87a6472`(per WF-1-55.77 P2 3 L4)
- 11 P0/P1/P2 commit(per RGS-REPORT-2026-08-26-P0P1P2_v0.2)
- per 2026-08-26 08:40 JST 代签新规则
