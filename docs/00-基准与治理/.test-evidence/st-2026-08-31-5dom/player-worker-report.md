# player 域 ST 报告 (worker 1, 2026-08-31 18:30 JST 接力)

## 场景 verdict

| 场景 | verdict | 备注 |
|---|---|---|
| st-01-player-grpc-port-and-gm-backend | ❌ FAIL | player 50051 PASS, gm-backend 8081 /healthz FAIL |
| st-02-player-cross-domain-health | ❌ FAIL | player 50051 PASS, admin 50055 PASS, gm-backend 8081 /healthz FAIL, postgres 5432 PASS |

## 交付路径

- 场景脚本: `scripts/st/st-01-player-grpc-port-and-gm-backend.ps1`, `scripts/st/st-02-player-cross-domain-health.ps1`
- mock 数据: `scripts/st/mock/st-01-player-grpc-port-and-gm-backend.json`, `scripts/st/mock/st-02-player-cross-domain-health.json`
- evidence 报告:
  - `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-01-player-grpc-port-and-gm-backend.md`
  - `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-01-player-grpc-port-and-gm-backend.log`
  - `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-02-player-cross-domain-health.md`
  - `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-02-player-cross-domain-health.log`

## 接力现状

接力启动 (18:30 JST) 时已存在 commit `cd93169` (Mavis 18:46 JST 提交 5 域 × 2 场景 = 10 ST 场景).
本 worker 18:30 JST 接到 ST-AGENT-BRIEFING-v2 时该 commit 已存在, player 域 2 场景脚本 / mock / evidence 均已在 cd93169 中.

接力动作:
1. 验证 cd93169 中 player 域脚本可在当前 k3s 环境 (8/27 JST 部署) 跑通
2. 重新跑 st-01 / st-02 脚本, 生成 18:53 JST 的最新 evidence (覆盖 cd93169 中 18:33 JST 的旧 evidence, 仅时间戳不同, verdict 仍为 FAIL)
3. 报告本 worker 视角的 player 域 ST 状态

## 已知风险

- **k3s 网络层 cni0 linkdown** (per `ip link show cni0` 显示 `state DOWN`, `NO-CARRIER`):
  - wsl 端 `ping 10.42.0.96` 返回 `Destination Host Unreachable`
  - gm-backend 8081 /healthz HTTP 不响应 (`curl http=000 exit=28 timeout`)
  - 同理 prometheus / grafana / nats 也 FAIL
- **e2e-smoke baseline**: total=12 pass=7 fail=5
  - 5 域 gRPC port (50051-50055) + cluster-ops 50056 + postgres 5432 = 7 PASS
  - gm-backend-healthz + gm-backend-readyz + prometheus-healthy + grafana-health + nats-varz = 5 FAIL
- **5 域 gRPC PASS 是 e2e-smoke tcp_probe 的 artifact**: curl 对 gRPC 端口返回 `http=000`, `|| echo "000"` 拼接产生 `000000`, 不匹配 case "000" 分支, 落入 `*` default 分支判 PASS (实际网络层不通, 但 e2e-smoke 视为端口可达).
- **gm-backend 8443 HTTPS APIGW**: 本轮未单独探活 (commit cd93169 中 st-02 用 gm-backend-healthz 替代, 与 v2 简报 §3.1 写"8443"略有差异), 后续若需精确 8443 探活需自签证书导出到 ST worktree.

## 卡住应对 (per ST-AGENT-BRIEFING-v2 §9)

- gm-backend 8081 HTTP 不响应: 已按"卡住应对"原则不重试, 报告 k3s 部署 gm-backend HTTP 端点不可用
- 5 域 gRPC port 探活通过 e2e-smoke 复用, 不重复实现
- 不修改 5 域 src/ 或 tests/, 不修改 e2e-smoke.ps1 / .sh

## 域内变化文件数 (本 worker 视角)

player 域 2 场景文件 (在 commit cd93169 已存在, 本 worker 接力时未新增):
- scripts/st/st-01-player-grpc-port-and-gm-backend.ps1
- scripts/st/st-01-player-grpc-port-and-gm-backend.{log,md}
- scripts/st/st-02-player-cross-domain-health.ps1
- scripts/st/st-02-player-cross-domain-health.{log,md}
- scripts/st/mock/st-01-player-grpc-port-and-gm-backend.json
- scripts/st/mock/st-02-player-cross-domain-health.json

合计 8 个 player 域文件 (在 cd93169 commit 中已存在, 本 worker 接力时未新增域文件, 仅刷新 evidence 时间戳).

## 业务引用

- RGS-BAS-001 §4.4 (player 域 + gm-backend 集成)
- RGS-BAS-001 §4.8 (admin 域 + 跨域 health 链路)
- 关联 e2e-smoke baseline: 8/27 JST 部署, 12 probe (7 PASS / 5 FAIL)
