# k3s 部署重启阻塞落档 (per 2026-08-30 12:55 JST)

> **目的**:记录 8/30 Ulysses 推 k3s 部署重启尝试 + 3 个阻塞
> **作者**:Mavis (接手 agent per DEC-008)
> **上游**: docs/deploy/phase-0-5-handoff.md (8/24 NO-GO 终止)
> **状态**: k3s 部署 3 阻塞,**下次会话需 Ulysses 介入**

---

## 3 个阻塞

| 阻塞 | 描述 | 需 Ulysses 行动 | 估修复时间 |
|---|---|---|---|
| **BLOCK-DEPLOY-001** | GHCR_PAT scope 不足 (push 返 `permission_denied: The token provided does not match expected scopes`) | 重新生成 PAT: https://github.com/settings/tokens → Generate new token (classic) → 勾 **write:packages** + **read:packages** → 设置 90 天 expiration → 提供新 token (以 $env:GHCR_PAT 形式 invoke, 不打印) | 5 分钟 |
| **BLOCK-DEPLOY-002** | WSL sudo 无密码 (chmod 644 /etc/rancher/k3s/k3s.yaml 卡死) | 改 /etc/sudoers 或运行 `sudo -i` 进 root 后 chmod, 或 `chmod -R a+r /etc/rancher/k3s/` (WLS 1 模式) | 5 分钟 |
| **BLOCK-DEPLOY-003** | 14 镜像 build 估 1.5+ 小时 (workspace Dockerfile build 全部 + multi-arch) | 拆分 4-6 批, 每批 ≤ 30 分钟 (1 worker 单桶), 用 buildx cache 复用 ghcr.io 已缓存层 (per handoff §5.2) | 1.5 小时 (分 4-5 worker) |

## 实际验证 (8/30 12:55 JST, 父 session 自做)

- **GHCR_PAT 存在**($env:GHCR_PAT 93 字符,符合 GitHub PAT 长度 90-100)
- **docker login ghcr.io 成功**(`echo $env:GHCR_PAT | docker login ghcr.io -u UlyssesLeoLee --password-stdin` → Login Succeeded)
- **docker push 失败**:`ghcr.io/ulyssesleolee/rustgameserver:pipeline-test-2026-08-30` 推 alpine 3.19 测试镜像 → `error from registry: permission_denied: The token provided does not match expected scopes`
- **k3s kubectl 验证**: Client v1.36.3+k3s1 可用, 但需 sudo chmod 644 k3s.yaml 才能 connect server

## 实际状态

- **k3s server 在跑**(per handoff 8/24 记录:postgres 跑 42h,NATS/OTel/Prom/Grafana ImagePullBackOff)
- **14 业务域镜像未推 ghcr.io**(per 8/29 W12 worker 失职 + GHCR_PAT 缺 write:packages)
- **B-CODE 4/4 失败**:1 部分 + 3 失败(per 8/24 handoff)
- **本次尝试(8/30 12:50)**: 3 阻塞全部失败, 1 小时窗口期 0 进展

## 下次会话推进路径

### Step 0:Ulysses 介入 (5 分钟)
1. 重新生成 GHCR_PAT (勾 write:packages + read:packages + 90 天)
2. 提供新 token: `$env:GHCR_PAT = 'ghp_...'` (PowerShell 环境变量, 不打印)
3. WSL 改 sudo 配置或手动 chmod /etc/rancher/k3s/k3s.yaml

### Step 1:验证 GHCR_PAT (5 分钟)
```bash
echo $GHCR_PAT | docker login ghcr.io -u UlyssesLeoLee --password-stdin
docker pull alpine:3.19  # 验证基础网络
docker tag alpine:3.19 ghcr.io/ulyssesleolee/rustgameserver:pipeline-test-$(date +%Y%m%d)
docker push ghcr.io/ulyssesleolee/rustgameserver:pipeline-test-$(date +%Y%m%d)
# 期望: push OK
```

### Step 2:14 镜像 build + push 拆分 (1.5-2 小时, 4-5 worker)
- **批次 1** (估 30 分钟): player-service, economy-service, match-service (3 核心域)
- **批次 2** (估 30 分钟): social-service, admin-service, cluster-ops (3 业务域)
- **批次 3** (估 30 分钟): gm-backend, card-service, leaderboard-service (3 卡牌域)
- **批次 4** (估 30 分钟): replay-service, i18n-service, deck-service, asset-service (4 卡牌/工具域)
- 每批 1 worker 单桶, 任务限 ≤ 30 分钟, 100% 成功模式

### Step 3:K8s manifest apply + verify (15 分钟)
```bash
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
kubectl apply -f docs/deploy/01-k8s-manifests/
kubectl get pods -n rust-game-server
# 期望: 14/14 Running (postgres 已有 + 5 业务域 + NATS + OTel + Prom + Grafana + 5 卡牌域)
```

### Step 4:B-CODE 4 项重测 (30 分钟)
- B-CODE-01: OTel + Prom + Grafana 3 套 K3s 部署 (验证 Pod Running 3/3)
- B-CODE-02: player gRPC HealthCheck (验证 mTLS + Health 探针)
- B-CODE-03: login → session_epoch → player_db 落库
- B-CODE-04: 跨域 trace 串联

## 关联

- docs/deploy/phase-0-5-handoff.md: SRE 接力清单 (5 步)
- docs/deploy/phase-0-5-step-6-report.md: 4 B-CODE 实际状态
- docs/deploy/RGS-PLAN-001_项目实施计划_v0.9.md: 7 G-CODE 全 Closed
- docs/00-基准与治理/RGS-DEC-NOGO-001_一人公司NO-GO解除决议_v0.1.md
- RGS-CARD-8BUCKET-W36-100PCT-V0.37-2026-08-30.md: 卡牌 8 桶 + W36 100% 闭环
- 上游 AI 通知 v1.7

## 时间线

- **8/24**: Phase 0.5 NO-GO 终止, 4 B-CODE 失败, SRE handoff 触发
- **8/25-8/28**: 智能合并 + W25-W32 阶段 (6 桶 5/8 完成)
- **8/29**: 卡牌 8 桶 100% 完成 + W36 1/3 步 (match → replay)
- **8/30**: W36 3/3 步完成 (跨域 100% 闭环) + k3s 部署重启尝试 (3 阻塞, 0 进展)
- **下次会话**: 待 Ulysses GHCR_PAT 重新生成 + WSL sudo 修 + 14 镜像分批 build/push
