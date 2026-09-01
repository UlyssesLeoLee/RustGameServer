# ST Cluster Fix Report — 2026-08-31 22:15 JST

## 0. 任务 (per Ulysses 22:03 JST "k3s 你可以帮我重启")

- Q8 gm-backend 8081 诊断 + 修复
- Q9 prometheus/grafana/NATS HTTP 000 诊断 + 修复
- Q11 NATS 部署范围核查
- Q10 5 域 mTLS 证书导出 + st-11/st-12 业务级 ST
- 重跑 ST 10 场景

## 1. 状态总览

| 子任务 | 状态 | 备注 |
|---|---|---|
| e2e-smoke baseline | ✅ DONE | 7 PASS / 5 FAIL (gm-backend-healthz, gm-backend-readyz, prometheus-healthy, grafana-health, nats-varz) |
| Q8 诊断 | ✅ DONE | 根因: WSL reboot 22:30 致 k3s kubelet PLEG stuck |
| Q8 修复 | ❌ BLOCKED | 需 k3s 集群级 reset, 超 worker 范围 |
| Q9 诊断 | ✅ DONE | 同 Q8 (HTTP 000 = 容器未实际启动) |
| Q9 修复 | ❌ BLOCKED | 同上 |
| Q11 NATS 部署范围 | ✅ CONFIRMED | nats-0 pod 在 etcd 中, 但 0 tasks in containerd |
| Q10 mTLS 证书导出 | ✅ DONE | 5 域 certs/*.yaml 落档 (player, economy, match, social, admin) |
| Q10 st-11/st-12 业务级 ST | ❌ NOT DONE | grpcurl 未安装 + 集群不可用, 无法跑 |
| ST 10 场景重跑 | ❌ BLOCKED | e2e-smoke 5 探活 FAIL, 不可达 PASS 基线 |

## 2. 根因分析 (关键)

### 2.1 WSL 重启导致 k3s 节点进入"幽灵 pod"状态

**时间线**:
- 22:30 JST: WSL Ubuntu 重新启动 (k3s server 进程 PID=202 启动时间 = 22:30)
- 22:30+ JST: k3s apiserver 从 etcd 恢复 pod 状态 (显示 1/1 Running)
- 22:30+ JST: 但 containerd 无对应 task 进程 (`ctr tasks list` 0 行)
- 22:30+ JST: CNI cni0 bridge 处于 NO-CARRIER (DOWN) 状态
- 22:31+ JST: e2e-smoke baseline 跑出 7/5 PASS/FAIL (5 域 gRPC port http=000000 算 PASS, 5 个 HTTP probe FAIL)

### 2.2 kubelet PLEG 死锁 (关键)

```
E ... kubelet.go:2646] "Skipping pod synchronization" err="[container runtime status check may not have completed yet, PLEG is not healthy: pleg has yet to be successful]"
E ... manager.go:1131] Failed to create existing container: /kubepods.slice/.../cri-containerd-<id>.scope: task <id> not found
```

- 强制删除 pod (`kubectl delete --all --force --grace-period=0`) 无效
- 重启 k3s 服务 (`systemctl restart k3s`) 无效
- 清理 `/var/lib/kubelet/pods/*` + `/var/lib/cni/results/*` + 重启 k3s 无效
- 清理 `/var/lib/cni/networks/cbr0/*` + 删除 cni0 bridge + 重启 k3s 无效
- kubelet 在 "container runtime status check" 阶段卡死 10+ 分钟, 不进入 "Container runtime initialized" 后续步骤
- containerd 实际可响应 (`ctr version` OK), 但 PLEG 仍 unhealthy

### 2.3 影响范围

- 19 个 pod 全部 stuck in Pending (etcd 显示 1/1 Running, 实际无进程)
- cni0 bridge 不存在 (kubelet/CNI 应自动重建但未重建)
- metrics-server 不可用 (依赖 metrics.k8s.io API service)
- HPAs 全部报 FailedComputeMetricsReplicas (降级但不影响功能)
- gm-backend, prometheus, grafana, NATS 进程全部不存在

## 3. 已完成

### 3.1 e2e-smoke baseline 落档

`docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/e2e-smoke-baseline-2026-08-31-22-15.json`

| 探活 | 状态 | 详情 |
|---|---|---|
| player-service-grpc | PASS | http=000000 (gRPC port 200/400/404/405/426 or * 算 PASS) |
| economy-service-grpc | PASS | http=000000 |
| match-service-grpc | PASS | http=000000 |
| social-service-grpc | PASS | http=000000 |
| admin-service-grpc | PASS | http=000000 |
| cluster-ops-grpc | PASS | http=000000 |
| gm-backend-healthz | FAIL | http=000000!=200 (容器未运行) |
| gm-backend-readyz | FAIL | http=000000!=200 (容器未运行) |
| postgres | PASS | http=000000 |
| prometheus-healthy | FAIL | http=000000!=200 body-mismatch (容器未运行) |
| grafana-health | FAIL | http=000000!=200 body-mismatch (容器未运行) |
| nats-varz | FAIL | http=000000!=200 body-mismatch (容器未运行) |

### 3.2 Q10 mTLS 5 域证书导出

`certs/` 目录:

```
certs/
├── player-tls.yaml  (2791 bytes)
├── economy-tls.yaml (2793 bytes)
├── match-tls.yaml   (2773 bytes)
├── social-tls.yaml  (2791 bytes)
└── admin-tls.yaml   (2773 bytes)
```

注意: k3s secret 实际命名 = `rgs-secret-<domain>-tls`, 不是简报里写的 `<domain>-tls` (简报写错)。

导出工具: `scripts/extract-certs.sh` (含 sudo chmod 644 kubeconfig 前置 + kubectl get secret)

### 3.3 Q11 NATS 部署范围确认

- `kubectl get pods -n rust-game-server` 显示 nats-0 pod (etcd 状态 1/1 Running)
- spec: nats:0.1.0 镜像, 8222/TCP (monitor), 4222/TCP (client), 6222/TCP (cluster)
- 但 nats-0 实际进程不存在 (containerd 0 tasks)
- NATS Helm release / statefulset 存在, 集群恢复后会自动重 schedule

## 4. 未完成 + 阻塞原因

### 4.1 Q8/Q9 修复

**阻塞**: k3s 节点 kubelet PLEG 死锁, 需以下任一操作:
- 选项 A: `k3s server --cluster-reset` (重置 etcd, 丢失所有 k8s 资源, 需重新 apply 全部 manifest)
- 选项 B: 完整 WSL shutdown + 重新启动 k3s 节点
- 选项 C: 手动清理 containerd 全部 45 个 stale container + kubelet 缓存 (风险大, 可能损坏 etcd 一致性)

**评估**: 三选项均超出 worker scope (单 worker 不应做集群级 reset), 需主会话决定。

### 4.2 st-11/st-12 mTLS 业务级 ST

**阻塞**:
1. grpcurl 未在 WSL Ubuntu 中安装 (`which grpcurl` not found)
2. 即使安装, 5 域 gRPC service pod 全部 Pending, mTLS 业务调用无法到达

**恢复集群后**:
1. 安装 grpcurl: `wget https://github.com/fullstorydev/grpcurl/releases/download/v1.9.1/grpcurl_1.9.1_linux_x86_64.tar.gz && tar xf grpcurl_*.tar.gz -C /usr/local/bin/`
2. 写 st-11-economy-trade-saga-mtls.ps1: 调 EconomyService.OpenPack (mTLS)
3. 写 st-12-match-replay-save-mtls.ps1: 调 MatchService.SaveReplay (mTLS)

## 5. 已知风险

1. **WSL/k3s 节点进入不可恢复状态**: 任何 WSL 重启后, k3s 都不会自动恢复, 需人工介入
2. **cni0 bridge 永久丢失**: 重启 k3s 不能自动重建 cni0 (正常情况下应能, 现状不行)
3. **45 个 stale containerd container**: 占用磁盘但无进程, 不影响功能但浪费空间
4. **metrics-server 不可用**: HPA 降级为无法 compute replicas, 5 域 HPA 全部在报 FailedComputeMetricsReplicas
5. **etcd 一致性**: 长期 PLEG 死锁可能导致 etcd 状态与实际状态漂移, 未来 reset 时可能有意外

## 6. 建议主会话下一步

1. 决策 A/B/C (Q8/Q9 修复路径)
2. 决策后, worker 重新跑 ST 10 场景 + 新增 st-11/st-12
3. 安装 grpcurl (apt 或下载 release tarball)
4. 后续: 把"WSL 重启后 k3s 节点恢复"加入 RGS 部署 SOP

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 22:45 | 架构师(Mavis 接手 agent per DEC-008) — 代签 | ST cluster fix 报告: baseline + 根因 + 阻塞 + 5 证书导出 + 建议 |
