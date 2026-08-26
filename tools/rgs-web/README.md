# RGS Admin Web (rgs-web) v0.1

**RGS 后台管理 Web UI — node + express + 静态 HTML dashboard**

| 项目 | 内容 |
|---|---|
| 工具目录 | `tools/rgs-web/` |
| 版本 | 0.1（2026-08-26 P2 阶段，per RGS-WEB-PLAN-2026-08-26） |
| 端口 | 8788（默认，可改 `RGS_WEB_PORT`；8787 已被其他服务占用） |
| 依赖 | node 18+ + express 4 |
| 后端 API | 代理 k3s API（6443）+ 读 5 域 IMPL-PLAN + docs 健康 + worktree 状态 |
| 设计参考 | RGS-WEB-PLAN-2026-08-26 v0.1（todo 写） |

## 启动

```bash
cd tools/rgs-web
npm install
npm start
# 访问 http://127.0.0.1:8788
```

## 启用 k3s 代理(需 WSL 内 sudo chmod)

```bash
# 在 WSL Ubuntu terminal:
sudo chmod 644 /etc/rancher/k3s/k3s.yaml

# 在 Windows terminal:
export K3S_TOKEN=$(cat /etc/rancher/k3s/k3s.yaml | grep token: | awk '{print $2}')
export K3S_CA_PATH=/etc/rancher/k3s/server/tls/server-ca.crt
npm start
```

## 路由

- `GET /` — Dashboard HTML
- `GET /api/health` — 健康检查 + k3s API URL
- `GET /api/impl-plan` — 5 域 IMPL-PLAN 状态（读 `docs/12-工作流/RGS-IMPL-PLAN-*.md`）
- `GET /api/docs-health` — 文档健康基线（1 FAIL + 1 WARN，per check-docs-consistency.sh）
- `GET /api/worktrees` — git worktree 列表（11 个 P0/P1/P2 + 17 个 v0.2 + 1 main）
- `ALL /api/k8s/*` — 代理 k3s API（需 K3S_TOKEN + K3S_CA_PATH）

## 已知缺口

- k3s 代理需 K3S_TOKEN + K3S_CA_PATH（lee19 需在 WSL 内 sudo chmod 644 k3s.yaml）
- 5 域 pod 状态展示未实现（kubectl 阻塞中）
- Web UI 静态 HTML 30s 自动 refresh（无需 WebSocket）
- 多用户/RBAC 未做（一人公司模式，per DEC-008）
