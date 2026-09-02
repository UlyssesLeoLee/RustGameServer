# RGS-PHASE-C-PREP-2026-09-02 v0.1 — Phase C SRE 介入准备包

> **创建日期**: 2026-09-02 16:10 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: RGS-PHASE-C-SRE-HANDOFF v0.1 (commit `8b70468`) + RGS-TEST-RUN-PLAN v0.1 (commit `82671df`) + 9/2 16:10 JST 集群摸底
> **配套**: RGS-K3S-CLUSTER-STATUS-2026-09-02 v0.1 (集群摸底报告) + RGS-BATCH-V0.1-FREEZE v0.1 (解冻触发条件)
> **作用域**: Phase C SRE 介入 4 阶段 23 步 checklist + 6 测试包, 供 SRE Lead 拍板

---

## 0. 触发与背景

Ulysses 2026-09-02 16:10 JST 拍板"全做 4 候选" (D4 周报 v0.3 + Phase C 准备 + 5 域 ST mTLS 1 跳 + D1 E2E 抢跑), 摸底发现:
- ✅ k3s 集群可达 (ulyssespc Ready 31h v1.36.3+k3s1)
- ✅ 5 域 svc + gm-backend endpoints OK
- ✅ gm-backend 8081/healthz HTTP 探活通过
- ❌ prometheus-84c47f7669-qnf4q CrashLoopBackOff 27h (SRE 范围)
- ❌ 5 域 gRPC mTLS 1 跳无法跑 (container minimal image 无 curl/wget, 需 SRE 装 grpcurl + certs)

**结论**: 5 域 ST 业务 mTLS 1 跳 + D1 5 域 E2E 抢跑 = **依赖 SRE 介入**, 产 Phase C 准备包供 SRE Lead 拍板.

---

## 1. 4 阶段 23 步 checklist (per RGS-PHASE-C-SRE-HANDOFF v0.1)

### 阶段 A: k3s 节点健康 (4 步, W37 D2 启动)

| 步骤 | 任务 | 工具 | 期望输出 |
|---|---|---|---|
| A1 | `kubectl get nodes` 节点状态 | kubectl | ulyssespc Ready (持续) |
| A2 | `kubectl get pods -A` 全 namespace 状态 | kubectl | 0 CrashLoopBackOff |
| A3 | **prometheus ReplicaSet 缩容** (本次发现) | `kubectl scale deploy prometheus --replicas=0 && kubectl delete pod prometheus-84c47f7669-qnf4q && kubectl scale deploy prometheus --replicas=1` | prometheus 1/1 Running, 0 CrashLoop |
| A4 | HPA / minReplicas 检查 (per §2.5 L6 教训) | `kubectl get hpa -A` | minReplicas ≤ desiredReplicas, 防止强启动风暴 |

### 阶段 B: 5 域 mTLS 业务级 ST (8 步, W37 D3-5)

| 步骤 | 任务 | 工具 | 期望输出 |
|---|---|---|---|
| B1 | k8s secret 导出 (5 域 + CA) | `kubectl get secret -n rust-game-server -o yaml > certs/<domain>-tls.yaml` | 6 个 cert yaml 文件 |
| B2 | mTLS 证书本地化 (per 8/27 ST 导出 SOP) | openssl x509 -text | cert 链验证 |
| B3 | grpcurl 安装到 admin pod (container minimal image 无) | `kubectl exec -n rust-game-server deploy/admin-service -- sh -c "apk add curl && curl ..."` 或 sidecar | admin pod 有 grpcurl + certs |
| B4 | player 50051 gRPC health probe | `grpcurl -cacert=ca.pem -cert=client.pem -key=client-key.pem player-service:50051 list` | 列出 player proto 服务 |
| B5 | economy 50052 gRPC health probe | 同上 | 列出 economy proto 服务 |
| B6 | match 50053 gRPC health probe | 同上 | 列出 match proto 服务 |
| B7 | social 50054 gRPC health probe | 同上 | 列出 social proto 服务 |
| B8 | admin 50055 gRPC health probe | 同上 | 列出 admin proto 服务 |

### 阶段 C: 22 测试函数真跑 (8 步, W37 D6-7 + W38)

| 步骤 | 任务 | 工具 | 期望输出 |
|---|---|---|---|
| C1 | 11 UT 函数 (per RGS-TEST-RUN-PLAN v0.1) | `cargo test --lib -p rgs-batch-backend` | 11/11 PASS |
| C2 | 11 E2E 函数 (DAG topology + rgs-web bridge + system health + OLU + credentials audit + Prometheus 12 + GAP-1 + GAP-6 + T-3 audit + message_outbox + sub_task lifecycle) | per BATCH-PLAN v0.2 §3.1 W3 | 11/11 PASS |
| C3 | 5 域跨域 saga 业务 E2E (per BATCH-PLAN W4-W6) | `cargo test --test '*' -- --test-threads=1` | 5 域 E2E PASS |
| C4 | mTLS 业务级连接 1 跳验证 (5 域 → gm-backend 8443) | grpcurl | 业务 mTLS OK |
| C5 | mTLS 业务级连接 2 跳验证 (gm-backend → 5 域) | grpcurl | 业务 mTLS OK |
| C6 | 跨域 saga 真实交易 (player → economy → admin) | per 1 笔测试交易 | 1 笔交易跑通, ledger 写入 |
| C7 | batch 域 GAP-10 跨域 saga 触发验证 (per commit `ea4c874`) | grpcurl | batch → saga OK |
| C8 | 22 测试函数合并 verdict + commit | commit `test-pass-phase-c` | 22/22 PASS, 1 commit |

### 阶段 D: 评审启动 (3 步, W38 D1-2)

| 步骤 | 任务 | 工具 | 期望输出 |
|---|---|---|---|
| D1 | 5 域 E2E 跑通 = 5 域生产可用里程碑达成 | 评审 | 业务里程碑 ✅ |
| D2 | batch 域 v0.1 解冻 (per RGS-BATCH-V0.1-FREEZE §3 触发解冻条件) | Ulysses 拍板 | `RGS-BATCH-V0.1-UNFREEZE-2026-XX-XX_v0.1.md` |
| D3 | RGS-CRITIQUE-IMPROVEMENT v0.1.1 5 大问题重新评估 | Mavis 自审 | v0.2 升版 或 维持 |

---

## 2. 6 测试包 (per RGS-TEST-RUN-PLAN v0.1)

### 2.1 11 UT (立即可跑, 不需 Phase C 介入)

```bash
cargo test --lib -p rgs-batch-backend -- --test-threads=1
# 期望: 11/11 PASS, 用时 < 60s
```

### 2.2 11 E2E (需 Phase C B/C 阶段完成)

```bash
cargo test --test '*' -p rgs-batch-backend -- --test-threads=1
# 11 个 E2E 函数 (per RGS-TEST-RUN-PLAN v0.1)
# 期望: 11/11 PASS, 用时 < 300s
```

### 2.3 5 域跨域 saga (需 Phase C B/C/D 阶段)

```bash
# per BATCH-PLAN v0.2 W4-W6, 38 L4 任务落地后跑
cargo test --test 'cross_domain_saga' -- --test-threads=1
# 期望: 跨域 saga PASS
```

### 2.4 mTLS 业务级 1 跳 (需 B4-B8 + certs 导出)

```bash
grpcurl -cacert=certs/ca.pem -cert=certs/client.pem -key=certs/client-key.pem \
  player-service:50051 grpc.health.v1.Health/Check
# 期望: status: SERVING
```

### 2.5 跨域 saga 真实交易 (需 C6 阶段)

```bash
# 1 笔测试交易: player 充值 → economy 记账 → admin 审计
# 期望: 三域业务 OK + ledger 写入正确
```

### 2.6 batch 域 GAP-10 跨域 saga 触发 (需 batch-backend 跑通)

```bash
# per commit `ea4c874` GAP-10 跨域 saga HashMap lookup 修复
# 期望: batch → saga OK
```

---

## 3. 集群摸底现状 (per RGS-K3S-CLUSTER-STATUS-2026-09-02 v0.1)

### 3.1 k3s 节点

| 节点 | 状态 | 版本 | AGE |
|---|---|---|---|
| ulyssespc | Ready control-plane | v1.36.3+k3s1 | 31h |

### 3.2 namespace pod 状态 (24 pod)

| namespace | pod | READY | STATUS | RESTARTS | AGE |
|---|---|---|---|---|---|
| kube-system | coredns-54996dc9b4-x79c7 | 1/1 | Running | 1 | 40h |
| kube-system | local-path-provisioner-58d557dc48-zjw82 | 1/1 | Running | 1 | 40h |
| kube-system | metrics-server-6dc596dfb8-68zxh | 1/1 | Running | 1 | 40h |
| rust-game-server | **admin-service-65c9cb7498-7t2f7** | 1/1 | Running | 0 | 24h |
| rust-game-server | cluster-ops-6d797744f9-6s2zq | 1/1 | Running | 0 | 28h |
| rust-game-server | cluster-ops-6d797744f9-8zjzc | 1/1 | Running | 0 | 28h |
| rust-game-server | cluster-ops-6d797744f9-rk4wd | 1/1 | Running | 0 | 28h |
| rust-game-server | **economy-service-d96947b76-f6zxb** | 1/1 | Running | 0 | 24h |
| rust-game-server | economy-service-d96947b76-zz894 | 1/1 | Running | 0 | 24h |
| rust-game-server | **gm-backend-5bf87b565-jrqkd** | 1/1 | Running | 0 | 31h |
| rust-game-server | grafana-79d54bf594-qklvp | 1/1 | Running | 0 | 31h |
| rust-game-server | **match-service-55668d9b9f-sxnmd** | 1/1 | Running | 0 | 24h |
| rust-game-server | match-service-55668d9b9f-w8cmw | 1/1 | Running | 0 | 24h |
| rust-game-server | match-service-55668d9b9f-xrc47 | 1/1 | Running | 0 | 24h |
| rust-game-server | nats-0 | 1/1 | Running | 0 | 28h |
| rust-game-server | otel-collector-64579b885d-67wt4 | 1/1 | Running | 0 | 31h |
| rust-game-server | **player-service-5ff45798b8-tw4jq** | 1/1 | Running | 0 | 24h |
| rust-game-server | player-service-5ff45798b8-xxjt2 | 1/1 | Running | 0 | 24h |
| rust-game-server | postgres-5945fc7ffb-67rzs | 1/1 | Running | 0 | 30h |
| rust-game-server | **prometheus-585fc54cfb-dr4dw** | 1/1 | Running | 0 | 27h |
| rust-game-server | **❌ prometheus-84c47f7669-qnf4q** | 0/1 | **CrashLoopBackOff** | **319 (96s ago)** | 27h |
| rust-game-server | **social-service-65656fbbc7-t89dr** | 1/1 | Running | 0 | 24h |
| rust-game-server | social-service-65656fbbc7-zsgjj | 1/1 | Running | 0 | 24h |

### 3.3 5 域 svc endpoints (✅ 全 OK)

| svc | ClusterIP | endpoints | port | AGE |
|---|---|---|---|---|
| player-service | 10.43.249.60 | 10.42.0.238/239:9464+7000 | 50051 | 31h |
| economy-service | 10.43.232.219 | 10.42.0.240/241:9464 | 50052 | 31h |
| match-service | 10.43.47.156 | 10.42.0.242/243/244:50053 | 50053 | 31h |
| social-service | 10.43.113.116 | 10.42.0.245/246:9464 | 50054 | 31h |
| admin-service | 10.43.67.244 | 10.42.0.247:9464+8080 | 50055 | 31h |
| gm-backend | 10.43.48.244 | 10.42.0.170:8081+9464+8443 | — | 31h |

### 3.4 gm-backend 8081 HTTP 探活 (主会话打头阵 1 跳)

```json
$ curl -s -m 5 http://127.0.0.1:18081/healthz
{"service":"gm-backend","status":"ok"}
```

✅ **HTTP 部分已通** (per §2.3 L4 主会话打头阵 1 跳).

### 3.5 ❌ prometheus CrashLoopBackOff 根因 (per §2.5 L6 排查)

- **事件**: `Back-off restarting failed container prometheus in pod prometheus-84c47f7669-qnf4q 4m51s (x1971 over 27h)`
- **日志根因**: `lock DB directory: resource temporarily unavailable` — 2 个 prometheus pod 抢同一 PVC lock
- **2 个 ReplicaSet 异常**:
  - `prometheus-574b797bc` 0/0/0 31h (旧, idle)
  - `prometheus-585fc54cfb` 1/1/1 27h (现, OK)
  - `prometheus-84c47f7669` 1/1/0 27h (新, CrashLoop)
- **修复** (per A3 步骤): `kubectl scale deploy prometheus --replicas=0` → 删 CrashLoop pod → `kubectl scale deploy prometheus --replicas=1`

---

## 4. 派生约束守护 (per AGENTS.md v0.6.4 §8)

- L1 cargo check 0 error — 本文档不动 Rust, N/A
- L11 cargo build dir lock — 本文档不编译, N/A
- L12 临时 log 不入 commit — pre-commit hook 兜底
- L13 自指字段 deferred 实时查询 — 引用 RGS-PHASE-C-SRE-HANDOFF v0.1 + RGS-TEST-RUN-PLAN v0.1 + 集群摸底数据全 git 实证
- L14 plumbing brace 跟踪 — 本文档无 patch 字符串拼接, N/A
- 8/27 11:06 JST 凭据硬 ban — 文档无 env value 痕迹 (k8s secret 仅提"导出 SOP", 不实际打印 cert 内容)

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

- **A4 HPA 检查** 可能发现更多强启动风暴源 (per §2.5 L6 教训), A4 步骤列出但 SRE 真跑才知道
- **B3 grpcurl 安装方式**: container minimal image 可能无 apk / apt, 需要 SRE 选 sidecar / init container / 本地安装
- **C2 11 E2E 函数具体内容**: per RGS-TEST-RUN-PLAN v0.1 列名, 实际测试可能发现 race condition / 端口冲突, SRE 拍板补齐
- **D2 batch 域 v0.1 解冻**: 5 域 E2E 跑通后 Ulysses 拍板, 不可预判
- **D3 RGS-CRITIQUE-IMPROVEMENT v0.2 升版**: 5 域生产可用里程碑达成后由 Mavis 写, Ulysses 二审

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 16:10 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 4 阶段 23 步 checklist + 6 测试包 + 集群摸底现状 + 派生约束守护 + 已知缺口, per 9/2 16:10 JST 全做 4 候选拍板 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
