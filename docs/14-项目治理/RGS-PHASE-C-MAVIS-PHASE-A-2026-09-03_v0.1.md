# RGS-PHASE-C-MAVIS-PHASE-A-2026-09-03 v0.1 — Mavis 推阶段 A 4 步 (SRE 替代)

> **创建日期**: 2026-09-03 12:38 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/3 12:36 JST ask_user 拍板 main-pioneer-sre (Mavis 跳过 SRE 拍板, 主会话推阶段 A 4 步) + 9/3 11:58 JST L-CAND-004 候选机制 (token 累计 0.5M 内 SRE 拍板必出, 超 1M 走选项 C 推迟, 当前 1.86M 已超阈值, 替代执行)
> **配套**: RGS-PHASE-C-PREP-2026-09-02 v0.1 §1 阶段 A 4 步 + RGS-K3S-CLUSTER-STATUS-2026-09-02 v0.1 (9/2 16:10 JST 集群摸底)
> **作用域**: Phase C 阶段 A 4 步 (节点健康 + 集群状态 + prometheus 修复 + HPA 检查)

---

## 0. 触发与背景

**SRE Lead 拍板悬空** (per 9/3 11:58 JST L-CAND-004 候选机制 1M token 阈值, 当前 R1 token 累计 1.86M 已超阈值):
- 9/2 17:32 JST 启动预热走"选项 1" (Mavis-side)
- 9/3 08:00 JST 12h+ 悬空
- 9/3 12:36 JST Ulysses 拍板 main-pioneer-sre: **Mavis 跳过 SRE 拍板, 主会话推阶段 A 4 步**

**9/3 12:37 JST 现场状态变化**:
- Windows 端 `kubectl get nodes` 失败: 127.0.0.1:52551 拒绝连接
- WSL 内部 `kubectl get nodes` 可达: ulyssespc Ready 2d4h v1.36.3+k3s1
- **结论**: k3s 集群本身健康, Windows 端 → WSL 端口转发失效, 走 WSL 路径

## 1. 阶段 A 4 步执行结果 (12:38 JST 实证)

### 1.1 A1 kubectl get nodes ✅

```bash
wsl -e bash -c "kubectl get nodes"
NAME        STATUS   ROLES           AGE    VERSION
ulyssespc   Ready    control-plane   2d4h   v1.36.3+k3s1
```

### 1.2 A2 kubectl get pods -A ✅ (20 pod 状态)

| pod | READY | STATUS | RESTARTS | AGE |
|---|---|---|---|---|
| admin-service-65c9cb7498-7t2f7 | 1/1 | Running | 0 | 44h |
| cluster-ops-6d797744f9-{6s2zq,8zjzc,rk4wd} | 1/1 | Running | 0 | 2d |
| economy-service-d96947b76-{f6zxb,zz894} | 1/1 | Running | 0 | 44h |
| gm-backend-5bf87b565-jrqkd | 1/1 | Running | 0 | 2d4h |
| grafana-79d54bf594-qklvp | 1/1 | Running | 0 | 2d4h |
| match-service-55668d9b9f-{sxnmd,w8cmw,xrc47} | 1/1 | Running | 0 | 44h |
| nats-0 | 1/1 | Running | 0 | 2d |
| otel-collector-64579b885d-67wt4 | 1/1 | Running | 0 | 2d4h |
| player-service-5ff45798b8-{tw4jq,xxjt2} | 1/1 | Running | 0 | 44h |
| postgres-5945fc7ffb-67rzs | 1/1 | Running | 0 | 2d2h |
| **prometheus-585fc54cfb-dr4dw** | 1/1 | **Running** | 0 | 47h (修复前 OK) |
| prometheus-84c47f7669-qnf4q | 0/1 | CrashLoopBackOff | 557 | 47h (待修) |
| social-service-65656fbbc7-{t89dr,zsgjj} | 1/1 | Running | 0 | 44h |

### 1.3 A3 prometheus ReplicaSet 修复 ✅ (核心)

**根因 (per RGS-PHASE-C-PREP §1 阶段 A3)**: 2 个 prometheus pod 抢同一 PVC lock
- `prometheus-585fc54cfb-dr4dw` 1/1 Running 47h (旧)
- `prometheus-84c47f7669-qnf4q` 0/1 CrashLoopBackOff 557 restarts (新, lock 抢不到)

**修复步骤**:
```bash
wsl -e bash -c "kubectl scale deploy prometheus -n rust-game-server --replicas=0"  # 退出旧 pod
wsl -e bash -c "kubectl delete pod -n rust-game-server prometheus-84c47f7669-qnf4q"  # 删 CrashLoop pod
sleep 5
wsl -e bash -c "kubectl scale deploy prometheus -n rust-game-server --replicas=1"  # 拉新 pod
sleep 15
wsl -e bash -c "kubectl get pods -n rust-game-server -l app.kubernetes.io/name=prometheus"
```

**修复后** (12:38 JST):
```
NAME                          READY   STATUS    RESTARTS   AGE
prometheus-84c47f7669-b87vq   1/1     Running   0          22s
```
✅ **新 pod 1/1 Running 0 restarts, 0 CrashLoopBackOff**

### 1.4 A4 HPA / minReplicas 检查 ✅

5 域 svc HPA 全部健康:
| HPA | MIN | MAX | REPLICAS | 资源 |
|---|---|---|---|---|
| admin-service-hpa | 1 | 2 | 1 | cpu 2%/70% |
| economy-service-hpa | 2 | 6 | 2 | cpu 0%/70%, mem 0%/80% |
| match-service-hpa | 3 | 12 | 3 | cpu 0%/65%, mem 0%/75% |
| player-service-hpa | 2 | 8 | 2 | cpu 0%/70%, mem 1%/75% |
| social-service-hpa | 2 | 6 | 2 | cpu 0%/70%, mem 0%/75% |

✅ **0 强启动风暴风险** (per RGS-PHASE-C-PREP §1 阶段 A4 + AGENTS.md §2.5 L6 教训)

## 2. 阶段 A 完成 → 阶段 B 解锁

per RGS-PHASE-C-KICKOFF v0.1 §2 + RGS-PHASE-C-PREP v0.1 §1:
- ✅ 阶段 A 全 4 步完成 = 阶段 B (5 域 certs 导出 + mTLS 业务级) 解锁
- 阶段 B 仍需 SRE Lead 拍板触发 (per RGS-PHASE-C-KICKOFF §3.1 4 选 1 拍板项) 或 Mavis 跳过拍板继续推 (per 9/3 12:36 JST 拍板 main-pioneer-sre)

**阶段 B 8 步 (per RGS-PHASE-C-PREP §1)**:
1. k8s secret 导出 (5 域 + CA)
2. mTLS 证书本地化 (per 8/27 ST 导出 SOP)
3. grpcurl 安装到 admin pod
4. player 50051 gRPC health probe
5. economy 50052 gRPC health probe
6. match 50053 gRPC health probe
7. social 50054 gRPC health probe
8. admin 50055 gRPC health probe

按 9/3 12:36 JST 拍板 main-mtls-mock (主会话写 mTLS 单元测试 + mock), 走 mock 路径, 不依赖 k3s 真实 svc + cert 导出 (避免阶段 B 8 步需 24h 等待)。

## 3. 派生约束守护

- **L1 (cargo check 0 error)**: N/A (本次 k8s 操作, 不动 Rust)
- **L11 (per-worker CARGO_TARGET_DIR)**: N/A
- **L12 (临时 log 不入 commit)**: ✅ 本次 WSL 输出在 PowerShell session, 未落盘
- **L-CAND-004 候选机制 (12/2 季度评审)**: 本次 SRE 替代为该候选的"主会话跳过 SRE 拍板"子段落地模式, 12/2 评审时验证
- **8/27 11:06 JST 凭据硬 ban**: ✅ 报告无 env value / secret / cert (per L-CAND-006 SOP 后续阶段 B 才导 cert)
- **8/27 JST 禁回溯叙事**: ✅ 不 amend / rebase / filter-branch
- **8/26 JST 缺标比错标**: ✅ 显式列 5 项已知缺口 (阶段 B 待 SRE / mTLS mock 路径 / prometheus 长期 fix / WSL 路径 / cert 轮换)

## 4. 已知缺口 (per 8/26 JST 缺标比错标)

- **阶段 B 8 步仍需 SRE 拍板触发** (per RGS-PHASE-C-KICKOFF §3.1 4 选 1 拍板项), 当前 SRE 替代只完成阶段 A
- **mTLS mock 路径** (per 9/3 12:36 JST 拍板 main-mtls-mock): 主会话写 5 域 mTLS 单元测试, 不依赖 k3s 真实 svc, 验证 client code + cert verification 逻辑
- **prometheus 长期 fix**: 9/3 修复了 CrashLoopBackOff, 但 PVC lock 抢锁根因未根治 (per L-CAND-006 §3 已知缺口)
- **WSL 路径**: 127.0.0.1:52551 → WSL 6443 端口转发失效, 走 WSL 内部 kubectl 兜底, 但 WSL 端 kubeconfig 需手动配置
- **cert 轮换**: 90 天 cert 轮换未脚本化 (per L-CAND-006 §3 已知缺口), 12/2 季度评审时补

## 5. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 12:38 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: Mavis 推阶段 A 4 步 (SRE 替代, per 9/3 12:36 JST 拍板 main-pioneer-sre), A1 nodes + A2 pods + A3 prometheus 修复 (新 pod 1/1 Running 0 restarts) + A4 HPA 5 域 0 强启动风暴风险, 阶段 A 全 4 步实证完成, 阶段 B 解锁, 5 项已知缺口 (阶段 B 待 SRE + mTLS mock 路径 + prometheus 长期 fix + WSL 路径 + cert 轮换) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
