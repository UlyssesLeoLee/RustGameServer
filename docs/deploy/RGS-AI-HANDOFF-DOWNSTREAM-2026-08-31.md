# RGS-AI-HANDOFF-DOWNSTREAM-2026-08-31.md — 下游 AI 接力 Handoff (需 k3s 集群访问)

| 字段 | 值 |
|---|---|
| 文档编号 | RGS-AI-HANDOFF-DOWNSTREAM-2026-08-31 |
| 版本 | v0.1 |
| 关联 commit | `f5c0359` (OPEN-QA v0.1) → 本次会话 v0.2 决策回复 (未 commit 前的 working tree) |
| 关联主对话 | 本次会话回复 `docs/00-基准与治理/RGS-OPEN-QA-2026-08-31-test-summary_v0.1.md` (v0.2) 的 Q1-Q11 + L1-L6 |
| 状态 | 🟡 5 项待办留待下游 AI (需 k3s 集群访问,本次会话集群不可连) |
| 收件方 | 下游 AI (需拥有可用的 k3s 集群连接 / kubectl 权限) |
| 修订人 | 上游 AI 接力 (Claude Code) |

---

## 0. 一句话当前状态

本次会话已对 `RGS-OPEN-QA-2026-08-31-test-summary_v0.1.md` 的 **11 项 P1 (Q1-Q11) + 6 项工程教训 (L1-L6)** 逐项决策 (v0.2)。其中 **7 项 (Q1-Q7 部分决策 + L1-L5) 已闭合**,**5 类待办因需要 k3s 集群访问而本次会话无法处理**——实测 `kubectl get pods -n rust-game-server` 报 `dial tcp 127.0.0.1:52551: connectex: No connection could be made because target machine actively refused it`,集群当前不可连(与 `docs/00-基准与治理/RGS-GHCR-PUSH-RESOLVED-V0.42-2026-08-31.md` §3 记录的状态一致,该问题此前也未强制启动,因已知 WSL2/k3s 存在 HPA minReplicas 强启动风暴历史问题)。

---

## 1. Q8 / Q9 / Q11 / L6 — k3s 容器诊断类 (待集群可连后处理)

### 1.1 Q8 (🔴 高): gm-backend 8081 /healthz + /readyz 不响应

**现象**: 容器在跑 (PORT 探活 PASS) 但 HTTP endpoint 不响应 (`curl` timeout, exit=28),导致 6 个 ST 场景 FAIL。

**诊断步骤**(集群可连后按序执行):
```bash
kubectl get pods -n rust-game-server -l app=gm-backend -o wide
kubectl get pods -n rust-game-server -l app=gm-backend -o jsonpath='{.items[*].status.containerStatuses[*].restartCount}'
kubectl describe pod -n rust-game-server -l app=gm-backend | grep -A20 Events
kubectl logs -n rust-game-server -l app=gm-backend --tail=100
kubectl exec -n rust-game-server deploy/gm-backend -- curl -sv http://localhost:8081/healthz
kubectl top pods -n rust-game-server -l app=gm-backend   # 排除 OOM
```

**关联历史线索**: 本项目此前诊断过一次集群级 `SandboxChanged` 风暴(根因是 HPA `minReplicas` 在 `metrics-server` 指标不可用时强制从 0 拉起 11 个 pod,导致 load average 飙到 1050、CNI 桥重建、所有 pod 报 `SandboxChanged`,表现为"容器在跑但网络/HTTP 不响应")。**建议先查 restartCount 和 events 里是否有 `SandboxChanged`**,如果有,大概率是同一类根因(HPA 强启动风暴),而非 gm-backend binary 本身的 bug。当前 HPA 应已被删除(`sudo k3s kubectl delete hpa --all -n rust-game-server`),若诊断发现 HPA 又被重建,应先确认 `metrics-server` 能正常出数(`kubectl top pods` 有数字而非报错)再决定是否恢复 HPA。

**决策项** (留给下游 AI 诊断后判断):
- [ ] 是否 binary startup 失败(看日志有无 panic / bind 失败)
- [ ] 是否容器 OOM
- [ ] 是否需要重启容器 / 重新 build image

### 1.2 Q9 (🟡 中): prometheus + grafana HTTP 探活 000000

**现象**: 容器 PORT 探活可达,但 HTTP endpoint 不响应,ST 4 个相关 probe FAIL。

**诊断步骤**: 同 Q8 模式,针对 `app=prometheus` / `app=grafana` label 跑同样的 `describe`/`logs`/`exec curl` 序列;额外检查:
```bash
kubectl exec -n rust-game-server deploy/grafana -- wget -qO- http://localhost:3000/api/health   # grafana admin password 是否 8/22 改过
kubectl logs -n rust-game-server -l app=prometheus --tail=50 | grep -i "reload\|error"
```

### 1.3 Q11 (🟢 低): NATS 8222 部署范围核查(事实核查,非决策)

**这是一条命令即可闭合的核查题**,集群可连后直接跑:
```bash
kubectl get pods -n rust-game-server -l app.kubernetes.io/name=nats
```
- 有输出(pod 存在)→ 按 Q9 模式修 HTTP 探活,升级为 P1
- 无输出(未部署)→ 标 SKIP,不算 P1,在 OPEN-QA 文档里勾掉即可

### 1.4 L6 (🟡 中,与 Q8 同一根因,工程教训已在 OPEN-QA v0.2 里确认为规则文本)

L6 的实际修复动作就是 Q8 的诊断 + 修复,不重复列出。教训规则文本已写入 OPEN-QA v0.2 §3 L6 决策段,待 §2 AGENTS.md 创建时一并落档。

---

## 2. Q10 — mTLS 业务级 ST(证书导出 + 实际重跑,待集群可连后处理)

**工具链已决策**(不需集群,已在 OPEN-QA v0.2 里闭合): **grpcurl**。

**待集群可连后执行**:
```bash
mkdir -p certs
for domain in player economy match social admin; do
  kubectl get secret ${domain}-tls -n rust-game-server -o yaml > certs/${domain}-tls.yaml
done
# 用 grpcurl + 上述证书对 5 域 50051-50055 端口做 mTLS 业务调用验证
# trade saga 端到端 (跨 economy+match+admin)、replay 端到端 (跨 match+admin) 待此步完成后再排期
```

---

## 3. AGENTS.md 创建(不需集群,但本次会话未做——避免第三个未经请求的产物)

`AGENTS.md` 当前仓库不存在。OPEN-QA v0.2 的 L1-L5 决策(以及 L6 教训文本)已经写出最终规则文本,下游 AI 只需**原样摘录**到新建的 `AGENTS.md`,无需重新决策。规则文本位置:`docs/00-基准与治理/RGS-OPEN-QA-2026-08-31-test-summary_v0.1.md` §3 各 L 条目下的"决策 (上游 AI, 2026-08-31 JST)"段落(共 6 条:worker cargo 长编译反 pattern / cargo check 必跑 / 跨工具链决策先查依赖 / 跨工具链场景先主会话打头阵 / ST worktree mTLS 证书 checklist / ST FAIL 先查 e2e-smoke baseline)。

---

## 4. 关键引用

- 本次决策回复的原始 QA 文档: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-31-test-summary_v0.1.md` (v0.2)
- UT+IT DDD Review: `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-UT-IT_v0.1.md`
- ST DDD Review: `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-ST_v0.1.md`
- k3s HPA 强启动风暴历史排障记录(per 8/26 JST 会话)
- GHCR push 阻塞已解除记录(集群不可连状态的最近一次确认): `docs/00-基准与治理/RGS-GHCR-PUSH-RESOLVED-V0.42-2026-08-31.md` §3
- e2e-smoke baseline: `scripts/e2e-smoke.ps1` / `scripts/e2e-smoke.sh`
- mTLS k8s secret: `docs/deploy/01-k8s-manifests/50-secret-*-tls.yaml`
- 证书生成 SOP: `docs/deploy/00-prerequisites/phase-0-5-step-4-gen-certs.ps1`

---

## 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 | 上游 AI 接力 (Claude Code) | 初版: 从 OPEN-QA v0.2 决策回复中拆出 5 类需 k3s 集群访问的待办 (Q8/Q9/Q11/L6/Q10证书导出) + AGENTS.md 创建指引 |
