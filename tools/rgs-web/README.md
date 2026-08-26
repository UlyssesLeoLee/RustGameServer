# RGS Admin Web (rgs-web) v0.3

**RGS 后台管理 Web UI — Node + http2 + mTLS + 静态 HTML dashboard,真实接 5 域 gRPC + cluster-ops**

| 项目 | 内容 |
|---|---|
| 工具目录 | `tools/rgs-web/` |
| 版本 | 0.3（2026-08-26，真实 gRPC 接入，per WF-1-rgs-web-v0.3-pf） |
| 端口 | 8788（默认，可改 `RGS_WEB_PORT`） |
| 依赖 | **零 npm 依赖** — Node 内置 `http` / `http2` / `child_process` / `fs` / `path` + 手写 protobuf 编码 |
| 后端 API | 真实 5 域 + cluster-ops 共 6 域 gRPC (mTLS) + 读 IMPL-PLAN + worktree + git log |
| gRPC 传输 | Node http2 + mTLS (rgs-ca.pem + rgs-client.crt/key) → kubectl port-forward (127.0.0.1:1505x) → k3s pod:5005x |
| 设计参考 | RGS-WEB-PLAN-2026-08-26 v0.1 + v0.3 真实接入 |

## 启动

### 1) 起 kubectl port-forward (在 WSL2 内)

```bash
# /mnt/d/RustGameServer/.pf-start.sh 已在本仓库
# setsid + nohup 守护,survive wsl session exit
wsl -e bash -c 'bash /mnt/d/RustGameServer/.pf-start.sh'
# 验证(Windows PowerShell):
Test-NetConnection 127.0.0.1 -Port 15051 # player-service gRPC
Test-NetConnection 127.0.0.1 -Port 15056 # cluster-ops gRPC
# 6 域 gRPC: 15051-15056  / 6 域 metrics: 19464-19469
```

### 2) 启动 rgs-web

```bash
cd tools/rgs-web
node server.js
# 访问 http://127.0.0.1:8788
# PID 写入 server.log
```

### 3) (可选) WSL 内 psql 用于 `/api/sql/query`

WSL 内 psql 未安装时 `/api/sql/query` 自动降级 mock;装 psql:
```bash
wsl -e bash -c 'sudo apt-get install -y postgresql-client'
```

## 路由 (v0.3)

### gRPC 真实调用 (经 mTLS + port-forward)

| Endpoint | 调用的 gRPC | 备注 |
|---|---|---|
| `GET /api/player/:id` | `player.v1.PlayerService/GetPlayer` (port 15051) | DB 实际为空,字段全 null — 真实 gRPC 调用成功 |
| `GET /api/guild/:id` | `social.v1.SocialService/GetGuild` (port 15054) | |
| `GET /api/match/:id` | `match.v1.MatchService/GetMatch` (port 15053) | |
| `GET /api/account/:id` | `economy.v1.EconomyService/GetAccount` (port 15052) | |
| `GET /api/adminop/:id` | `admin.v1.AdminService/GetAdminOp` (port 15055) | |
| `GET /api/node/:id` | `cluster_ops.v1.ClusterOpsService/GetNode` (port 15056) | |
| `GET /api/health/all` | 6 域并发 `*Service/HealthCheck` | 返回各域 status/latency/message |
| `GET /api/services/status` | (静态) | 6 域 endpoints + methods 列表 |
| `GET /api/pfau/phase` | `cluster-ops/HealthCheck + GetNode` | 降级 — proto 无 PFAU method |
| `GET /api/metrics/:service` | HTTP GET pod:9464 (经 port-forward 1946x) | 已知缺口:19464 ECONNREFUSED (per 主对话 21:01 JST 探活) |

### DB / 文档辅助

| Endpoint | 数据源 | 备注 |
|---|---|---|
| `GET /api/sql/query?db=&sql=` | `wsl -e bash -c "psql ..."` | psql 不可用时自动降级 mock + 注释 |
| `GET /api/impl-plan` | 读 `docs/12-工作流/RGS-IMPL-PLAN-*.md` | |
| `GET /api/worktrees` | `git worktree list --porcelain` | |
| `GET /api/git-log` | `git log --pretty=format:... -n 30` | |
| `GET /api/docs-health` | (静态) | 1 FAIL + 1 WARN 基线 |
| `GET /api/health` | (静态) | rgs-web PID + version + transport info |
| `ALL /api/k8s/*` | https 代理 k3s API (6443) | 需 K3S_TOKEN + K3S_CA_PATH |

## 关键文件

- `server.js` — 6 API 端点 + http2/mTLS gRPC client + 手写 protobuf 编码/解码
- `public/index.html` — 10 页面 dashboard,JS fetch 全部走真实 endpoint
- `rgs-ca.pem` — RustGameServer Dev CA (k3s secret `rgs-secret-ca`)
- `rgs-client.crt.pem` / `rgs-client.key.pem` — rgs-web mTLS client cert (由 Dev CA 签发)
- `package.json` — 仍含 `express` 字段但 v0.3 实际不用 (历史兼容,见 migration note)

## 已知缺口 (不在 v0.3 范围)

1. **9464 metrics 端口 connection refused** — k3s pod 9464 未起服务 (per 主对话 21:01 JST admin-service 探活);
   `/api/metrics/:service` 端点已实现,等 k3s 9464 服务起来即通
2. **PFAU phase gRPC method 不存在** — cluster-ops proto 只声明 `HealthCheck` + `GetNode`,
   `PFAU 阶段表` 降级为 HealthCheck + GetNode 报告
3. **5 域 DB 空** — `GetPlayer`/`GetAccount`/`GetGuild`/`GetMatch`/`GetAdminOp` 返回 HTTP 200 + 空 Player,
   gRPC 管线真实打通,DB 真实数据需 Phase D.4 seed
4. **port-forward 重连** — 6 域 pod 重启后 pod IP 变化,kubectl port-forward 不会自动重连;
   本次 worktree session 内手动 `bash /mnt/d/RustGameServer/.pf-start.sh` 重启即可
5. **PSQL 未装** — WSL 内 `psql` 不存在,`/api/sql/query` 降级 mock;可降级方案:kubectl exec 跑 psql 或 port-forward 5432 + Node pg client
6. **CA 私钥原本丢失** — 5 域 binary 启动时 CA 私钥已不在 `E:\DevCache\cargo\target\dev-certs\`,
   本次 worktree 用 rgs-certgen 重新生成 CA + 6 域 server cert + rgs-web client cert,已 k3s `kubectl apply` 7 secret + 6 deployment rollout restart

## 版本历史

| 版本 | 日期 | 内容 |
|---|---|---|
| 0.1 | 2026-08-26 | 初版:node + express + 静态 HTML + 代理 k3s API |
| 0.2-gm | 2026-08-26 | GM 后台 10 页面 + 5 域 IMPL-PLAN + worktrees + git log (v0.2-gm 保留作为降级) |
| **0.3** | 2026-08-26 | **真实 6 域 gRPC 接入** (mTLS + kubectl port-forward + http2 + 手写 protobuf);10 页面 JS 全部 fetch 真实 endpoint |
