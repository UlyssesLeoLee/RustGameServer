# k3s + RGS 部署状态 (per 2026-09-09 16:20 JST)

**用户要求**: 起 k3s + 部署 RGS 5 域 (per 9/9 16:00 JST Ulysses 拍板 A)
**实际进展**: 资源全备好, 本机环境阻塞, 待手动恢复

## 1. 准备 (✅ 完成)

### 1.1 k3s-deploy/ 一键部署
```
D:\RustGameServer\k3s-deploy\
├── Dockerfile                  (mcr.microsoft.com/windows/servercore base, 复用 5 binary)
├── docker-compose.yml          (5 域 + 端口 50061-50065 + rgs-net 网络)
├── deploy.sh                   (build/start/stop/status/logs/k3s 6 子命令)
├── bin/                        (5 binary 共 45MB)
│   ├── player-service.exe      (9.19 MB)
│   ├── economy-service.exe     (11.14 MB)
│   ├── match-service.exe       (9.74 MB)
│   ├── social-service.exe      (8.13 MB)
│   └── admin-service.exe       (8.69 MB)
└── DEPLOY_STATUS.md            (本文档)
```

### 1.2 真实 k8s manifests 验证 (53/53 YAML OK)
```
D:\RustGameServer\docs\deploy\01-k8s-manifests\ (53 files)
✅ 00-namespace.yaml + 5 域 Deployment+Service (01-05)
✅ 06-cluster-ops + 07-shared-platform + 08-10 configmap+secret+rbac
✅ 20-25 postgres (Secret+PVC+StatefulSet+Service+NetworkPolicy)
✅ 30-nats-* (5 files, configmap+sa+statefulset+service+networkpolicy)
✅ 40-42 otel+prometheus+grafana (10 files)
✅ 50-52 GM/scene/battle/network (8 files)
✅ 55-57 gm-console envoy (3 files)
✅ 60-test-runner-job
✅ e4-00~02 namespace 隔离 + 资源限制 + HPA 模板
```
所有 53 YAML 文件 Python yaml.safe_load_all 验证 OK

## 2. 阻塞 (本机环境问题)

| 阻塞 | 详情 | 解决 |
|------|------|------|
| **Docker Desktop service 拒绝启动** | `com.docker.service` Stopped, `sc start` 拒绝访问 (OpenService 失败 5) | 用户手动重启 Docker Desktop |
| **k3s image 拉取超时** | rancher/k3s:latest ~600MB, 网络 5min 超时 | 网络恢复后重试 |
| **k3s Windows binary URL 错** | GitHub 404 (k3s.exe 不存在, 应为 k3s Linux binary 跑 WSL2) | 改用 k3d 或 WSL2 k3s |
| **kubectl 验证需 cluster** | `--dry-run=client` 仍需 openapi schema | 改用 Python yaml.safe_load_all 验证 (已通过) |

## 3. 恢复步骤 (Docker Desktop 修复后)

### 3.1 修 Docker
```
# 1) 任务管理器 → 服务 → com.docker.service → 右键启动
# 2) 或: 重启 Docker Desktop 应用
# 3) 验证: docker ps 应列 rgs-postgres-uat
```

### 3.2 部署 RGS 5 域 (docker compose)
```bash
cd D:\RustGameServer\k3s-deploy
bash deploy.sh build      # build rgs:0.1.0 (~10min Windows image)
bash deploy.sh start      # 5 域 docker compose up
bash deploy.sh status     # 验证
```

### 3.3 部署 RGS 5 域 (真实 k3s)
```bash
# 1) 装 k3s
curl -sfL https://get.k3s.io | sh -
# 2) 部署
cd D:\RustGameServer
kubectl apply -k docs/deploy/01-k8s-manifests/
# 3) 验证
kubectl get pods -n rust-game-server
```

### 3.4 shim 仍 Windows
- k8s pod 不支持 raw TCP long-live connection (zsyz_client 心跳需要)
- shim 继续跑 Windows (或迁移到 Node.js sidecar)
- shim → rgs-proxy → k3s pod svc DNS (player-service.rust-game-server:50061 等)

## 4. 当前可用链路 (5 binary 跑 Windows native)

```
zsyz_client (E 盘 H5, 待 Cocos Creator 2.3.2 build)
    ↓ SmartSocket TCP 9001
rgs-shim-rust v0.3.2 (Windows pid 38980, commit 0dd4828)
    ↓ HTTP/JSON 8084
rgs-proxy (Node.js pid 28532, D:\playwright-test\rgs-proxy\server.js)
    ↓ gRPC
5 RGS binary (Windows native, 50061-50065):
  - player-service pid 31432
  - economy-service pid 29584
  - match-service pid 14612
  - social-service pid 41652
  - admin-service pid 38868
    ↓ sqlx
docker postgres rgs-postgres-uat (5433, 6 域 db + user)
```

**已验证**: proto-test.js 17/17 passed, 5 域 HealthCheck 5/5 OK, 真 RGS 数据 (MavisHero / Gold-uuid / RGS Vanguard) 经 shim 返回。

## 5. 真正 E2E 缺什么

| 缺 | 需要 |
|---|------|
| H5 binary (1-2h 一次性) | Cocos Creator 2.3.2 + 导入 zsyz_client_h5/ + compile -p web |
| shim 业务覆盖 100% (1-2 周) | 4-5 worker 派工扩 758 cmd (per 9/9 15:25 JST 矩阵) |
| k3s 实际部署 (网络恢复后) | Docker Desktop 重启 + 拉 k3s image + apply manifest |
| 真实 RGS in k3s (替代 Windows native) | 5 pod 跑 k3s, shim 走 svc DNS 解析 |

代签: Mavis 默认代签 Ulysses (per 8/27 三次强化 + 9/8 15:19 第 6/7 次强化)
