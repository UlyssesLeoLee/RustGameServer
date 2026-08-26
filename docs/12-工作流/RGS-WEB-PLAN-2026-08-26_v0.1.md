# RGS-WEB-PLAN-2026-08-26 v0.1

**RGS 后台管理 Web UI 设计 + 实施方案**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WEB-PLAN-2026-08-26 |
| 版本 | 0.1（per Ulysses 2026-08-26 12:15 JST "RGS 需要一个网页端的后台"） |
| 状态 | 设计 + 最小可行实现已落地（`tools/rgs-web/` v0.1,node + express-less HTTP server） |
| 触发 | 2026-08-26 12:15 JST Ulysses "RGS 需要一个网页端的后台"（per RGS-DOCS-HEALTH-2026-08-26 §2 P2 拆分） |
| 责任人 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））|
| 关联 | RGS-REPORT-2026-08-26-P0P1P2-v0.2 + RGS-DOCS-HEALTH-2026-08-26 + DEC-008 一人公司 12 角色 |

---

## 0. 触发与背景

**触发**：Ulysses 2026-08-26 12:15 JST 指令"RGS 需要一个网页端的后台"(per WBS v0.8 WF-1-55.77 commit 87a6472 后)。

**现状**：
- RGS 5 域 + cluster-ops + shared-platform = 纯 Rust gRPC 后端(无 HTTP)
- WBS v0.4 §2A.2.55.续1 11 份 IMPL-PLAN(v0.1 已落地)无 Web UI 配套
- k3s API server 6443 端口可用(curl /healthz = 401 验证)
- DDD Review 阶段 5 域 Lead 需要一个集中入口看 P0/P1/P2 进度 + docs 健康 + k3s 状态

**本文档目的**:最小可行 Web UI 设计,30 分钟内可落地,后续可扩展。

---

## 1. 设计目标

| 目标 | 优先级 | 实现位置 |
|---|---|---|
| DDD Review 阶段查看 11 commit 状态 | P0 | `/api/worktrees` + Git 列表 |
| 看 5 域 IMPL-PLAN 落地详情 | P0 | `/api/impl-plan` + 读 docs/12-工作流 |
| 看文档健康(1 FAIL + 1 WARN 基线) | P0 | `/api/docs-health`(per check-docs-consistency.sh)|
| 代理 k3s API 看 cluster 状态 | P1 | `/api/k8s/*` 透传 |
| k3s token 鉴权 | P1 | `K3S_TOKEN` 环境变量(待 WSL 内 sudo chmod 644 k3s.yaml)|
| 多用户/RBAC | P3(不做)| 一人公司模式 per DEC-008,无需 |
| WebSocket 实时推送 | P2(不做)| 30s 轮询足够 |
| 5 域 pod 实时状态 | P1 | 通过 `/api/k8s/api/v1/pods` 代理 |

---

## 2. 技术选型

| 选项 | 选择 | 理由 |
|---|---|---|
| 后端 | **node + 原生 http(无 express)** | 避免 npm install 超时(实测 2 分钟+);node 22 + http 自带足够做 5 个 API endpoint |
| 前端 | **原生 HTML + CSS + JS** | 0 依赖,刷新友好,无 webpack/vite 复杂度 |
| 端口 | **8788**(8787 已被 Cursor headroom 占用)| 用户可改 `RGS_WEB_PORT` |
| 部署 | **Windows 端 node 进程** | 后续可改 k3s deploy,但本地最简 |

---

## 3. 已落地结构

```
D:/RustGameServer/tools/rgs-web/
├── package.json          (309 B, 暂不依赖 express)
├── server.js             (3.7 KB, 含 express 实现 - 待 npm install 完成后启用)
├── server-no-deps.js     (2.5 KB, 零依赖 HTTP server - 当前运行)
├── public/
│   └── index.html        (6.2 KB, Dashboard)
└── README.md             (1.6 KB, 启动 SOP)
```

**当前运行**: `node server-no-deps.js` on port 8788(PID 4956, 后台)

---

## 4. API 路由(5 个)

| 路径 | 方法 | 后端 | 状态 |
|---|---|---|---|
| `/api/health` | GET | 静态 | ✅ 已实现 |
| `/api/impl-plan` | GET | 读 docs/12-工作流/RGS-IMPL-PLAN-*.md | ✅ 已实现 |
| `/api/worktrees` | GET | git worktree list | ✅ 已实现 |
| `/api/docs-health` | GET | 静态(per check-docs-consistency.sh 基线)| ✅ 已实现 |
| `/api/k8s/*` | ALL | 代理 k3s 6443(需 K3S_TOKEN + K3S_CA)| ⚠️ 实现但需 token |

---

## 5. Dashboard UI 布局

```
+------------------------------------------+
|  RGS Admin Dashboard  (12:20 JST)        |
|  k3s API: 6443 · RGS v0.2 = 9e6a392      |
+------------------------------------------+
| [5 域 IMPL-PLAN]    [文档健康]            |
|  8/8 落地           1 FAIL + 1 WARN      |
+------------------------------------------+
| [P0/P1/P2 进度]     [k3s Cluster]         |
|  11/11 commit       active (curl 401)    |
+------------------------------------------+
| [11 个 worktree 状态表]                   |
|  wbs/WF-1-55.69  6c4c1eb  P0/P1  merged   |
|  wbs/WF-1-55.77  87a6472  P2    progress  |
|  ...                                      |
+------------------------------------------+
| [5 域 IMPL-PLAN 详情表]                   |
|  RGS-IMPL-PLAN-ADMIN-001  21.4 KB        |
|  ...                                      |
+------------------------------------------+
```

---

## 6. 启动 SOP

### 6.1 零依赖版(当前)

```bash
node D:/RustGameServer/tools/rgs-web/server-no-deps.js
# 访问 http://127.0.0.1:8788
```

### 6.2 完整版(待 npm install 完成)

```bash
cd D:/RustGameServer/tools/rgs-web
npm install
node server.js
```

### 6.3 启用 k3s 代理(完整 kubectl 视图)

在 WSL Ubuntu terminal:

```bash
sudo chmod 644 /etc/rancher/k3s/k3s.yaml
```

在 Windows terminal:

```bash
$env:K3S_TOKEN = (Get-Content \\wsl$\Ubuntu\etc\rancher\k3s\k3s.yaml -Raw | Select-String -Pattern 'token: (.+)' | ForEach-Object { $_.Matches.Groups[1].Value })
node D:/RustGameServer/tools/rgs-web/server-no-deps.js
```

---

## 7. 已知缺口(per DDD Review 必查)

- [ ] npm install 超时 2 分钟+:依赖安装慢(可能 Windows 网络问题),暂用 zero-deps 版
- [ ] k3s 代理需 K3S_TOKEN:WSL sudo chmod 644 k3s.yaml 需 Ulysses 手动
- [ ] 5 域 pod 实时状态:依赖 kubectl,通过 /api/k8s 代理,需 token
- [ ] 多用户认证:无,一人公司模式
- [ ] WebSocket 实时推送:无,30s 轮询
- [ ] dark/light theme 切换:无,只有 dark
- [ ] 移动端适配:部分,需测试

---

## 8. 后续 P2 扩展

| 任务 | 优先级 | 描述 |
|---|---|---|
| 启用 k3s 代理 + 5 域 pod 视图 | P1 | Ulysses sudo chmod 644 后,/api/k8s/* 完整透传 |
| WBS v0.8 进度可视化 | P1 | 把 5 域 IMPL-PLAN 状态 vs WBS §2A.2.55.续3 关联 |
| check-docs-consistency 实时跑 | P2 | 每 5 分钟跑 bash 脚本,UI 显示最新结果 |
| 5 域 Lead DDD Review 签字状态 | P1 | 读 per-domain RACI v1.0 + 5 域 Lead 签字栏 |
| Prometheus metrics 嵌入 | P3 | 5 域 gRPC 服务暴露 /metrics,UI 嵌入 |
| Login 页面 | P3(不做)| 一人公司模式 |

---

## 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）)| 初版:最小可行 Web UI(零依赖 node + 原生 HTML)|

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态:无 Web UI
- v0.1 新增:`tools/rgs-web/`(package.json + server-no-deps.js + public/index.html + README.md)
- v0.1 API:5 个 endpoint(health/impl-plan/worktrees/docs-health/k8s 代理)
- v0.1 启动:node server-no-deps.js on port 8788

### A.2 对 RGS 治理的影响

- DDD Review 阶段 5 域 Lead 可集中查看 P0/P1/P2 进度
- 不影响现有 Rust 后端 cluster-ops / 5 域 gRPC(独立进程,独立端口)

### A.3 已知缺口

- 见 §7 4 项(待 DDD Review 阶段补签/补功能)
- npm install 超时根因待查(可能 Windows 网络/DNS/registry)

### A.4 引用链与证据

- 当前运行:`node D:/RustGameServer/tools/rgs-web/server-no-deps.js` on 127.0.0.1:8788(PID 4956)
- 验证:Invoke-WebRequest http://127.0.0.1:8788/api/health → 200 OK
- WBS v0.8 + 11 P0/P1/P2 commit 全部纳入 dashboard
- 修订历史代签新规则 per 2026-08-26 08:40 JST(C:\Users\leon19\.minimax\memory\user.md "文档代签规则反转")
