# 05-deploy-sop.md — 部署标准操作程序

> **文档 ID**：`RGS-DEPLOY-SOP-001`
> **版本**：v0.2（NO-GO 状态，9/1 加 §6 k3s 单节点部署派生约束）
> **生效日期**：2026-08-21（v0.1）/ 2026-09-01 22:30 JST（v0.2）
> **状态**：🔴 NO-GO 占位
> **关联**：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` + `RGS-EXEC-001 v0.3` + `RGS-OPEN-QA-2026-08-31-test-summary_v0.3.md §7.5`

---

## 0. 重要前提

> ⚠️ **本 SOP 在 53 開発環境構築 启动条件全部满足前**（7 G-CODE Closed + 12 类签字栏齐全 + 12 类环境核验全通过）**禁止执行任何步骤。**
>
> 执行部署前必须先通过 `../07-no-go-checklist_v0.2.md` 全部 ✅。

---

## 1. 部署前置 checklist

### 1.1 NO-GO 状态

- [ ] `../07-no-go-checklist_v0.2.md` 全部 ✅
- [ ] `../00-prerequisites/00-no-go-checklist_v0.2.md` 全部 ✅
- [ ] `RGS-ENV-001 v0.3 §6` 12 类签字栏全部具名签字（**当前 2/12 实际签 + 10/12 所有者背书占位**）
- [ ] `RGS-REV-003 §7.3` 12 类签字栏全部具名签字（**当前 8/12 实际签 + 10+ 所有者背书占位**）
- [ ] `RGS-EXEC-001 v0.3` 7 G-CODE 全部 Closed（**当前 7/7 Open**）

### 1.2 责任人到位

- [ ] 架构师具名（Ulysses 实际签 ✅）
- [ ] PM 具名（Ulysses 实际签 ✅）
- [ ] 评审主持人具名（Ulysses 实际签 ✅）
- [ ] DBA 具名（**待具名**）
- [ ] SRE 具名（**待具名**）
- [ ] Platform 架构师具名（**待具名**）
- [ ] QA Lead 具名（**待具名**）
- [ ] 5 域 Lead 独立具名（per DEC-005，**全部待具名**）
- [ ] 业务方代表具名（**待具名**）
- [ ] Economy 域 Lead Q-003 二次签字（**待具名**）

### 1.3 制品就位

- [ ] 5 域 + cluster-ops + shared-platform 镜像已 push 到 registry
- [ ] Cargo.lock 已 commit
- [ ] Helm chart 已 `helm dependency update` 通过
- [ ] DB migrations 已通过 staging 验证

### 1.4 环境核验

- [ ] RGS-ENV-001 v0.3 §1-§5 12 类核验全部通过
- [ ] Rust 1.98 + CI 全绿（G-CODE-06 满足）
- [ ] PG 18.6 5 独立 DB 已就绪（DBA 签字）
- [ ] QUIC edge 证书已注入 Secret（加密仓管理）

---

## 2. 部署架构概览

```
┌────────────────────────────────────────────────────┐
│                CI/CD Pipeline (04-ci-cd)           │
│  push → 镜像构建 → SBOM → push to registry         │
└────────────────────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────┐
│            Staging 环境（先部署）                    │
│  - 5 域 + cluster-ops + shared-platform             │
│  - 5 独立 DB（player/economy/match/social/admin）    │
│  - cluster_ops_db（PFAU 历史）                      │
└────────────────────────────────────────────────────┘
                       ↓
       集成测试 + 性能压测（QA Lead 签字）
                       ↓
┌────────────────────────────────────────────────────┐
│            Production 环境（按域分批灰度）           │
│  - 灰度顺序：admin (COC) → cluster-ops → player →   │
│               social → economy → match             │
│  - 灰度比例：10% → 50% → 100%                      │
└────────────────────────────────────────────────────┘
```

---

## 3. 部署步骤（按域独立）

### 步骤 1：admin 域（COC 控制面，最先部署）

> **理由**：COC 控制面需最先就位，以便监控后续 5 域部署

```bash
# 1.1 DB 迁移
psql -h PLACEHOLDER_ADMIN_DB_HOST -U admin_user -d admin_db \
  -f deploy/03-db-migrations/admin_db/0001_initial.sql
psql -h PLACEHOLDER_ADMIN_DB_HOST -U admin_user -d admin_db \
  -f deploy/03-db-migrations/admin_db/0002_coc_audit_log.sql

# 1.2 Namespace + RBAC
kubectl apply -f deploy/01-k8s-manifests/00-namespace.yaml
kubectl apply -f deploy/01-k8s-manifests/10-rbac-template.yaml   # admin SA

# 1.3 Secret 注入（加密仓）
kubectl apply -f deploy/secrets/admin-db-secret.yaml
kubectl apply -f deploy/secrets/coc-ops-secret.yaml

# 1.4 ConfigMap
kubectl apply -f deploy/01-k8s-manifests/08-configmap-template.yaml

# 1.5 admin 域 Deployment
kubectl apply -f deploy/01-k8s-manifests/05-admin-service.yaml

# 1.6 验收
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=admin -n PLACEHOLDER_NAMESPACE --timeout=300s
kubectl port-forward svc/PLACEHOLDER_ADMIN_SVC_NAME PLACEHOLDER_ADMIN_COC_WEB_PORT:PLACEHOLDER_ADMIN_COC_WEB_PORT -n PLACEHOLDER_NAMESPACE &
# 浏览器访问 COC Web UI 确认
```

**责任人**：admin 域 Lead + SRE
**签字栏**：`RGS-ENV-001 v0.3 §6` admin 域 / SRE 类别

### 步骤 2：cluster-ops 域（Active-Active，固定 3 副本）

> **理由**：cluster-ops 域为 PFAU 控制面，需在业务域前就位

```bash
# 2.1 DB 迁移
psql -h PLACEHOLDER_CLUSTER_OPS_DB_HOST -U cluster_ops_user -d cluster_ops_db \
  -f deploy/03-db-migrations/cluster_ops_db/0001_initial.sql
psql -h PLACEHOLDER_CLUSTER_OPS_DB_HOST -U cluster_ops_user -d cluster_ops_db \
  -f deploy/03-db-migrations/cluster_ops_db/0002_pfau_history.sql

# 2.2 cluster-ops 域 Deployment（**禁 HPA**）
kubectl apply -f deploy/01-k8s-manifests/06-cluster-ops-service.yaml

# 2.3 验收
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=cluster-ops -n PLACEHOLDER_NAMESPACE --timeout=300s
# 验证 all-reachable：每个副本都能响应 PFAU 操作
kubectl exec -it PLACEHOLDER_CLUSTER_OPS_POD_NAME -n PLACEHOLDER_NAMESPACE -- \
  curl http://localhost:PLACEHOLDER_CLUSTER_OPS_PFAU_PORT/health
```

**责任人**：SRE + 架构师
**签字栏**：`RGS-ENV-001 v0.3 §6` SRE / 架构师 类别
**特别约束**：per ADR-0052，**禁 HPA**，固定 3 副本 + topologySpreadConstraints 跨节点

### 步骤 3：player 域 / social 域（可并行部署，无 Saga 依赖）

```bash
# player
kubectl apply -f deploy/01-k8s-manifests/01-player-service.yaml
# social
kubectl apply -f deploy/01-k8s-manifests/04-social-service.yaml
```

**责任人**：player 域 Lead / social 域 Lead
**签字栏**：`RGS-ENV-001 v0.3 §6` 对应域 Lead 类别

### 步骤 4：economy 域（Q-003 Saga 核心，**最敏感**）

> **理由**：Q-003 Saga 跨域核心，事务最密集，部署失败影响最大

```bash
# 4.1 DB 迁移（含 Q-003 Saga 状态机）
psql -h PLACEHOLDER_ECONOMY_DB_HOST -U economy_user -d economy_db \
  -f deploy/03-db-migrations/economy_db/0001_initial.sql
psql -h PLACEHOLDER_ECONOMY_DB_HOST -U economy_user -d economy_db \
  -f deploy/03-db-migrations/economy_db/0002_q003_saga_state.sql

# 4.2 economy 域 Deployment
kubectl apply -f deploy/01-k8s-manifests/02-economy-service.yaml

# 4.3 Q-003 Saga 集成测试（QA Lead 必在场）
#   - 跨域事务：player → economy → social 三方
#   - 异常注入：网络分区 / DB 短暂不可用
#   - 回滚验证：Saga 状态机回退
```

**责任人**：economy 域 Lead + Economy 域 Lead Q-003 二次签字 + QA Lead
**签字栏**：`RGS-ENV-001 v0.3 §6` economy 域 Lead + Q-003 二次 + QA 类别（**3 重签字**）

### 步骤 5：match 域（实时匹配，最后部署）

```bash
kubectl apply -f deploy/01-k8s-manifests/03-match-service.yaml
```

**责任人**：match 域 Lead
**签字栏**：`RGS-ENV-001 v0.3 §6` match 域 Lead 类别

### 步骤 6：shared-platform（QUIC edge 占位）+ OTel collector

> 2026-08-24 架构复核：`07-shared-platform.yaml` 内原有的 otel-collector 定义与
> Step 3 的 `40-otel-collector-*.yaml` 重复，已从 07 中移除。OTel Collector 改用
> Step 3 脚本部署（见下），07 现仅保留 QUIC edge 占位说明。

```bash
pwsh -File docs/deploy/phase-0-5-step-3-render-observability.ps1
```

**责任人**：Platform 架构师
**签字栏**：`RGS-ENV-001 v0.3 §6` Platform 类别

### 步骤 7：灰度切流

| 阶段 | 比例 | 持续时间 | 监控指标 |
|---|---|---|---|
| 灰度 1 | 10% | 24h | error rate / p99 latency |
| 灰度 2 | 50% | 24h | 同上 + Q-003 Saga 成功率 |
| 全量 | 100% | — | 全监控 |

**责任人**：SRE + admin 域 Lead（COC 监控）
**触发回滚条件**：error rate > 0.5% / p99 > SLA / Q-003 Saga 失败率 > 0.1%

---

## 4. 部署验收

- [ ] 所有域 pod ready
- [ ] 所有域 health check 通过
- [ ] Q-003 Saga 集成测试通过（QA Lead 签字）
- [ ] 5 独立 DB schema 落地正确（DBA 签字）
- [ ] 灰度阶段无异常（SRE 签字）
- [ ] COC Web UI 正常显示（admin 域 Lead 签字）
- [ ] 7 G-CODE 全部 Closed 状态保留
- [ ] 12 类签字栏全部具名签字

---

## 5. 关联文档

- 回滚 SOP：`../06-rollback-sop.md`
- NO-GO 自检表：`../07-no-go-checklist_v0.2.md` + `../00-prerequisites/00-no-go-checklist_v0.2.md`
- 治理：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6`
- 架构：`RGS-ARC-051`（COC/CEM/PFAU）+ `RGS-ADR-0052`（Active-Active）
- 设计：5 域 DTL（015/016/018/019/020/026/031）

---

## 6. k3s 单节点部署派生约束（per OPEN-QA v0.3 §7.5）

> **来源**：`RGS-OPEN-QA-2026-08-31-test-summary_v0.3.md` §7.5（2026-09-01 07:55 JST Mavis 接手代签）
> **触发**：8/27 WSL 单节点 k3s `cluster-reset` A 路径失败 + 9/1 节点 `ulyssespc` 注册未恢复
> **状态**：🟡 占位（SRE 介入，Ulysses 真身操作，per §0.4 Mavis 处理边界）
> **关联 DDD Review §7.2 P2**: k3s PLEG 死锁 + cluster-reset 派生约束写入 RGS 部署 SOP

### 6.1 派生约束 5 条（per 9/1 07:55 JST 教训）

1. **WSL reboot 后 k3s 不会自动恢复** — 必须完全卸载（`k3s-uninstall.sh`）+ 重新安装 + 重 apply manifest
2. **8/27 部署 manifest 用了 `PLACEHOLDER_*` 模板占位符** — 应改为 kustomize / helm template，不依赖 sed 替换（per D3 改造）
3. **`cluster-reset` 不是单节点 k3s 修复方法** — 单节点用完全卸载 + 重装（per k3s 官方建议：cluster-reset 适用于多节点 etcd quorum 重建场景）
4. **测试 agent 不应承担 SRE 工作** — k3s 部署恢复 / manifest apply / 证书生成属于 SRE 范畴，Mavis 仅负责"等 SRE 修好后跑 ST 重跑 + 写测试代码"
5. **Mavis 处理 k3s 问题的尝试边界**（per 22:03 JST Ulysses "k3s 你可以帮我重启" 授权）：
   - ✅ 可做：`wsl --shutdown`, `chmod 644 kubeconfig`, `kubectl scale 0→N`, 读 `kubectl get pods` / `kubectl describe pod` / `kubectl logs` / `kubectl exec curl localhost:port`
   - ❌ 不应做：卸载 k3s, 重 apply 18 manifest（需完整 SRE 工具链）, 修证书, 改 yaml, `k3s-uninstall.sh`

### 6.2 失败日志特征（per 9/1 07:55 JST journalctl 实证）

```
E0901 ... kubelet.go:3516] "Unable to register mirror pod because node is not registered yet"
                          err="node \"ulyssespc\" not found" node="ulyssespc"
E0901 ... kubelet_node_status.go:396] "Error getting the current node from lister"
                          err="node \"ulyssespc\" not found"
E0901 ... kubelet.go:2646] "Skipping pod synchronization"
                          err="container runtime status check may not have completed yet"
I... "Unable to set control-plane role label: nodes \"ulyssespc\" not found"
```

**根因**：WSL 单节点 k3s, hostname 变化 / etcd 漂移 / PLEG 死锁（per 8/26 JST HPA 强启动风暴历史，`RGS-OPEN-QA-2026-08-27-k3s-deploy v0.4 §0`）。cluster-reset 后 kubelet 期待手动 register, 单进程 k3s 包含 server+agent 通常自动 join, **这次未自动**。

### 6.3 SRE 介入 Checklist（per 9/1 22:03 JST Ulysses 授权范围）

> ⏳ 以下步骤需 Ulysses 真身 / SRE 操作，**Mavis 不在授权范围**

- [ ] 完全卸载 k3s：`/usr/local/bin/k3s-uninstall.sh`（server 端）
- [ ] 重新安装 k3s（不带 `--cluster-reset`）：`curl -sfL https://get.k3s.io | sh -`
- [ ] 验证节点 `ulyssespc` 自动注册：`kubectl get nodes` 应见 `ulyssespc` Ready
- [ ] 重 apply 8/27 manifest + 8/29 9:30 / 17:15 两次 secret 命名修订（per `RGS-OPEN-QA-2026-08-27-k3s-deploy v0.4`）
- [ ] 5 域 mTLS 证书重生（per `scripts/phase-0-5-step-4-gen-certs.ps1`）
- [ ] 验证 18 pod 1/1 Running + e2e-smoke baseline ≥10 PASS（per 8/27 baseline 7/5 PASS/FAIL）

### 6.4 Mavis 续跑路径（per 9/1 22:25 JST Phase D 拍板）

- SRE 修好 k3s 后 Mavis 派 ST-fix worker 续跑 st-11/st-12 mTLS 业务级 ST（per OPEN-QA v0.3 §7.6）
- Mavis 跑 `git push origin main` 推 33 commits
- DDD Review 终审决议 6 项 P1 backlog（per `RGS-DDD-2026-09-01-PT-WORKERS_v0.1.md` §6）
- 完成 Q8/Q9/Q10/Q11 收尾（per OPEN-QA v0.3 §7.4 ⏳ 阻塞项）

