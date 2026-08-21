# 06-rollback-sop.md — 回滚标准操作程序

> **文档 ID**：`RGS-ROLLBACK-SOP-001`
> **版本**：v0.1（NO-GO 状态）
> **生效日期**：2026-08-21
> **状态**：🔴 NO-GO 占位
> **关联**：`05-deploy-sop.md` + `RGS-PLAN-001 v0.7 §3.3` + `RGS-EXEC-001 v0.2`

---

## 0. 重要前提

> ⚠️ **本 SOP 在 53 開発環境構築 启动条件全部满足前**禁止执行任何步骤。
>
> 回滚触发后，**5 分钟内必须启动回滚决策流程**，由 admin 域 Lead（COC 控制面）+ SRE 联合决策。

---

## 1. 回滚触发条件

### 1.1 自动回滚（系统触发）

| 指标 | 阈值 | 检测源 |
|---|---|---|
| Error rate | > 0.5% | Prometheus + Alertmanager |
| p99 latency | > SLA（per 域 SPEC） | Prometheus |
| Pod restart count | > 5/min | K8s event |
| Q-003 Saga 失败率 | > 0.1% | economy 域 metrics |
| cluster-ops all-reachable 丢失 | 任一副本无响应 > 30s | cluster-ops 自检 |

### 1.2 人工回滚（人工决策）

- 业务方代表紧急叫停
- 架构师判定架构问题
- QA Lead 判定质量问题
- 安全漏洞发现
- 数据损坏迹象

---

## 2. 回滚分级

| 级别 | 范围 | 决策人 | 时间窗口 |
|---|---|---|---|
| **L1 单域回滚** | 单域 Deployment 回退到上一个镜像 tag | 该域 Lead | 5 分钟内 |
| **L2 域间回滚** | 多域同时回滚（含 Q-003 Saga） | admin 域 Lead + SRE + 架构师 | 15 分钟内 |
| **L3 全量回滚** | 全集群回退到上一个稳定版本 | PM + 架构师 + 业务方代表 | 30 分钟内 |
| **L4 灾难恢复** | 跨可用区切换 / 备份恢复 | 全员（架构师 + PM + DBA + SRE + 业务方） | 1 小时内启动 |

---

## 3. 回滚步骤

### 3.1 L1 单域回滚

> 适用：单域 Deployment 异常，Q-003 Saga 仍稳定

```bash
# 1. 确认异常域
kubectl get pods -l app.kubernetes.io/name=PLACEHOLDER_DOMAIN -n PLACEHOLDER_NAMESPACE

# 2. 查看最近 Deployment 历史
kubectl rollout history deployment/PLACEHOLDER_DOMAIN_DEPLOY_NAME -n PLACEHOLDER_NAMESPACE

# 3. 回滚到上一版本
kubectl rollout undo deployment/PLACEHOLDER_DOMAIN_DEPLOY_NAME -n PLACEHOLDER_NAMESPACE

# 4. 监控回滚进度
kubectl rollout status deployment/PLACEHOLDER_DOMAIN_DEPLOY_NAME -n PLACEHOLDER_NAMESPACE

# 5. 验收
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=PLACEHOLDER_DOMAIN -n PLACEHOLDER_NAMESPACE --timeout=300s
```

**责任人**：该域 Lead
**回滚后必做**：在 `RGS-EXEC-001 v0.2` 新增 incident entry + 通知 admin 域 Lead（COC 记录）

**特殊：cluster-ops 域 L1 回滚**

> per ADR-0052，cluster-ops 是 Active-Active + all-reachable。**单副本回滚可能破坏 all-reachable 假设。**

```bash
# cluster-ops L1 回滚必须**逐副本滚动**（不能 undo 整体）
# 1. 标记异常副本为 cordon
kubectl cordon PLACEHOLDER_NODE_NAME

# 2. 驱逐异常副本
kubectl drain PLACEHOLDER_NODE_NAME --ignore-daemonsets --force

# 3. 恢复节点
kubectl uncordon PLACEHOLDER_NODE_NAME

# 4. 验证 all-reachable
kubectl exec -it PLACEHOLDER_CLUSTER_OPS_POD_NAME -- curl http://localhost:PLACEHOLDER_CLUSTER_OPS_PFAU_PORT/health
```

**责任人**：SRE + 架构师（per ADR-0052）

### 3.2 L2 域间回滚（含 Q-003 Saga）

> 适用：Q-003 Saga 异常 / 跨域事务失败

```bash
# 1. 暂停所有入站流量（admin 域 COC 操作）
kubectl exec -it PLACEHOLDER_ADMIN_POD_NAME -- \
  curl -X POST http://localhost:PLACEHOLDER_ADMIN_GRPC_PORT/coc/v1/traffic/pause

# 2. 等 Q-003 Saga 队列清空（最多 5 分钟）
# 监控 economy 域 metrics
watch -n 5 'kubectl exec -it PLACEHOLDER_ECONOMY_POD_NAME -- curl http://localhost:PLACEHOLDER_ECONOMY_METRICS_PORT/metrics | grep saga_pending'

# 3. economy 域 L1 回滚（per 3.1）

# 4. 恢复入站流量
kubectl exec -it PLACEHOLDER_ADMIN_POD_NAME -- \
  curl -X POST http://localhost:PLACEHOLDER_ADMIN_GRPC_PORT/coc/v1/traffic/resume

# 5. Q-003 Saga 重放（自动）
# economy 域会自动重放未完成的 Saga 步骤
```

**责任人**：admin 域 Lead + SRE + 架构师 + economy 域 Lead + Economy 域 Lead Q-003 二次
**签字栏**：`RGS-ENV-001 v0.2 §6` admin/SRE/架构师/economy/Q-003 5 类联合签字
**特别约束**：per DEC-005，Q-003 二次签字不可跳过

### 3.3 L3 全量回滚

> 适用：多域同时异常 / 整体架构问题

```bash
# 1. 暂停所有入站流量
kubectl exec -it PLACEHOLDER_ADMIN_POD_NAME -- \
  curl -X POST http://localhost:PLACEHOLDER_ADMIN_GRPC_PORT/coc/v1/traffic/pause

# 2. 全集群 Deployment 统一回滚（按 5-deploy-sop.md §3 部署顺序倒序）
# 顺序：match → economy → social → player → cluster-ops → admin
# 即：最后部署的最先回滚
for domain in match economy social player cluster-ops admin; do
  kubectl rollout undo deployment/PLACEHOLDER_${domain^^}_DEPLOY_NAME -n PLACEHOLDER_NAMESPACE
  kubectl rollout status deployment/PLACEHOLDER_${domain^^}_DEPLOY_NAME -n PLACEHOLDER_NAMESPACE
done

# 3. DB 回滚（**高风险** — 仅在 schema 变更引入问题时执行）
# 警告：DB schema 回滚可能导致数据丢失
# 必须 DBA 联合 PM 决策
# 实际执行见 03-db-migrations/_status.md 中"DBA 主导"步骤

# 4. 恢复入站流量
kubectl exec -it PLACEHOLDER_ADMIN_POD_NAME -- \
  curl -X POST http://localhost:PLACEHOLDER_ADMIN_GRPC_PORT/coc/v1/traffic/resume
```

**责任人**：PM + 架构师 + 业务方代表 + DBA
**签字栏**：`RGS-ENV-001 v0.2 §6` PM/架构师/业务方/DBA 4 类联合签字
**特别约束**：DB schema 回滚需 DBA 单独二次确认（per DEC-005 延伸：DB 是 DBA 独立控制面）

### 3.4 L4 灾难恢复

> 适用：跨可用区故障 / 备份恢复

```bash
# 1. 启动灾备决策（PM 召集）
# 2. 切换流量到灾备可用区（per K8s multi-cluster 拓扑，待定）
# 3. 从备份恢复 DB（per 03-db-migrations/ 中"DBA 主导"步骤 + RGS-ENV-001 §6 DBA 类别）
# 4. cluster-ops 重新建立 Active-Active（per ADR-0052）
# 5. 业务方通知 + 客户公告
```

**责任人**：全员
**签字栏**：12 类全部联合签字
**特别约束**：L4 不可跳过任何签字栏

---

## 4. 回滚后必做

### 4.1 立即（5 分钟内）

- [ ] 在 `RGS-EXEC-001 v0.2` 新增 incident entry（含时间 / 触发指标 / 决策人 / 操作步骤）
- [ ] 通知业务方代表（admin 域 Lead 通过 COC 操作）
- [ ] 检查 Saga 状态机（economy 域）是否完全回退
- [ ] 检查 COC 审计日志（admin_db）

### 4.2 短期（24 小时内）

- [ ] 召开根因分析会议（架构师召集）
- [ ] 更新 5-deploy-sop.md（如 SOP 有缺陷）
- [ ] 更新 ADR（如架构决策有缺陷）
- [ ] 更新 RGS-ENV-001 v0.2 §6 签字栏（如责任人需要补强）

### 4.3 中期（1 周内）

- [ ] 修复根因
- [ ] 加固监控
- [ ] 回归测试
- [ ] 重新部署（按 5-deploy-sop.md）

---

## 5. NO-GO 状态保留

> 本 SOP 在 53 開発環境構築 启动条件全部满足前**不得激活为可执行 SOP**。
>
> 任何回滚演练需在 staging 环境执行，由 QA Lead 主导，**禁止在 production 执行回滚演练**。

---

## 6. 关联文档

- 部署 SOP：`../05-deploy-sop.md`
- NO-GO 自检表：`../07-no-go-checklist.md`
- ADR：`RGS-ADR-0052`（Active-Active + all-reachable）
- 决策：`DEC-005`（5 域 Lead 独立）
- 设计：`RGS-ARC-051`（COC/CEM/PFAU）+ Q-003 Saga（DTL-015/016）
