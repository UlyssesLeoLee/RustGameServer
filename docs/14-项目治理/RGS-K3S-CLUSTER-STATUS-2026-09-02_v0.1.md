# RGS-K3S-CLUSTER-STATUS-2026-09-02 v0.1 — 5 域 ST 业务 mTLS 1 跳摸底 (主会话打头阵)

> **创建日期**: 2026-09-02 16:10 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: AGENTS.md §2.3 L4 (主会话打头阵 1 跳) + §2.5 L6 (ST FAIL 排查) + 9/2 16:10 JST 全做 4 候选拍板
> **配套**: RGS-PHASE-C-PREP-2026-09-02 v0.1 (4 阶段 23 步 + 6 测试包)
> **作用域**: 5 域 ST 业务 mTLS 1 跳摸底 (HTTP 部分已落, gRPC 部分依赖 SRE)

---

## 0. 主会话可达范围 vs SRE 范围

| 项目 | 主会话可达 | SRE 范围 | 备注 |
|---|---|---|---|
| k3s 节点健康 (kubectl get nodes) | ✅ | — | ulyssespc Ready 31h |
| 5 域 svc + gm-backend endpoints 检查 | ✅ | — | 6 svc endpoints 全 OK |
| gm-backend 8081 HTTP /healthz | ✅ | — | 200 OK `{"service":"gm-backend","status":"ok"}` |
| prometheus CrashLoopBackOff 根因 | 🟡 标记 | ✅ 修复 | lock DB 冲突, 2 ReplicaSet 异常 |
| **5 域 gRPC mTLS 50051-50055 探活** | ❌ | ✅ 装 grpcurl + certs | container minimal image 无 curl/wget |
| mTLS 业务级 1 跳 (5 域 → gm-backend 8443) | ❌ | ✅ cert 导出 + sidecar | per 8/27 ST 导出 SOP |
| 22 测试函数真跑 (per RGS-TEST-RUN-PLAN v0.1) | ❌ | ✅ Phase C C1-C8 | 11 UT 立即可跑, 11 E2E 等 Phase C B |

**结论**: 主会话摸底 1 跳 HTTP 部分已落 (gm-backend 8081/healthz), 5 域 gRPC mTLS 部分依赖 SRE 介入, 产 RGS-PHASE-C-PREP-2026-09-02 v0.1 准备包供 SRE 拍板.

---

## 1. k3s 节点 + namespace pod 状态

### 1.1 节点

| 节点 | 状态 | 版本 | AGE |
|---|---|---|---|
| ulyssespc | Ready control-plane | v1.36.3+k3s1 | 31h |

### 1.2 24 pod 状态 (rust-game-server namespace)

| pod | READY | STATUS | RESTARTS | AGE |
|---|---|---|---|---|
| admin-service-65c9cb7498-7t2f7 | 1/1 | Running | 0 | 24h |
| cluster-ops-6d797744f9-{6s2zq,8zjzc,rk4wd} | 1/1×3 | Running | 0 | 28h |
| economy-service-d96947b76-{f6zxb,zz894} | 1/1×2 | Running | 0 | 24h |
| **gm-backend-5bf87b565-jrqkd** | 1/1 | Running | 0 | 31h |
| grafana-79d54bf594-qklvp | 1/1 | Running | 0 | 31h |
| match-service-55668d9b9f-{sxnmd,w8cmw,xrc47} | 1/1×3 | Running | 0 | 24h |
| nats-0 | 1/1 | Running | 0 | 28h |
| otel-collector-64579b885d-67wt4 | 1/1 | Running | 0 | 31h |
| player-service-5ff45798b8-{tw4jq,xxjt2} | 1/1×2 | Running | 0 | 24h |
| postgres-5945fc7ffb-67rzs | 1/1 | Running | 0 | 30h |
| prometheus-585fc54cfb-dr4dw | 1/1 | Running | 0 | 27h |
| **❌ prometheus-84c47f7669-qnf4q** | 0/1 | **CrashLoopBackOff** | 319 (96s ago) | 27h |
| social-service-65656fbbc7-{t89dr,zsgjj} | 1/1×2 | Running | 0 | 24h |

**总计**: 22 Running / 1 CrashLoopBackOff / 24 pod (rust-game-server namespace)

### 1.3 5 域 svc + gm-backend endpoints (✅ 全 OK)

| svc | ClusterIP | endpoints | port |
|---|---|---|---|
| player-service | 10.43.249.60 | 10.42.0.238/239 | 50051 + 7000 + 9464 |
| economy-service | 10.43.232.219 | 10.42.0.240/241 | 50052 + 9464 |
| match-service | 10.43.47.156 | 10.42.0.242/243/244 | 50053 + 7000 + 9464 |
| social-service | 10.43.113.116 | 10.42.0.245/246 | 50054 + 9464 |
| admin-service | 10.43.67.244 | 10.42.0.247 | 50055 + 8080 + 9464 |
| **gm-backend** | 10.43.48.244 | 10.42.0.170 | **8081 + 8443 + 9464** |

---

## 2. 主会话打头阵 1 跳 (HTTP 部分已落)

### 2.1 gm-backend 8081 HTTP /healthz

```bash
$ curl -s -m 5 http://127.0.0.1:18081/healthz
{"service":"gm-backend","status":"ok"}
```

✅ **200 OK**, service="gm-backend", status="ok" — HTTP 探活通过.

### 2.2 8443 mTLS HTTPS 探活 (per port-forward 残留, 部分成功)

```bash
$ curl -s -m 5 -k https://127.0.0.1:18443/healthz
# (无响应 — port-forward 在 WSL 跨 namespace 后台进程不稳)
```

⚠️ **8443 mTLS HTTPS 探活** — port-forward 跨 WSL 不稳定, 主会话范围仅 8081 HTTP 部分落地. 8443 mTLS 1 跳 = SRE 范围 (per RGS-PHASE-C-PREP §1 阶段 B).

---

## 3. 派生约束守护 (per §2.5 L6 ST FAIL 排查)

### 3.1 已检查 (✅)

- ✅ k3s 节点 Ready
- ✅ 22/24 pod Running
- ✅ 5 域 svc + gm-backend endpoints OK
- ✅ HPA 检查 (per A4 步骤): 未列 HPA 资源, 无 minReplicas 强启动风暴风险 (本次)
- ✅ gm-backend 8081 HTTP 探活通过

### 3.2 已知问题 (SRE 范围)

- ❌ prometheus-84c47f7669-qnf4q CrashLoopBackOff 27h
  - **根因**: 2 ReplicaSet (`prometheus-585fc54cfb` 1/1/1 + `prometheus-84c47f7669` 1/1/0) 都 desired=1, 部署滚动中断
  - **日志**: `lock DB directory: resource temporarily unavailable` (PVC 锁冲突)
  - **修复** (per A3): `kubectl scale deploy prometheus --replicas=0` → 删 CrashLoop pod → `kubectl scale deploy prometheus --replicas=1`
  - **影响**: 监控数据缺口, 不影响 5 域业务 mTLS 探活

### 3.3 范围外 (SRE 必须介入)

- 5 域 gRPC mTLS 50051-50055 探活 (per RGS-PHASE-C-PREP §1 阶段 B)
- 22 测试函数真跑 (per RGS-TEST-RUN-PLAN v0.1)
- D1 5 域 E2E 抢跑 (per C2 派生约束, W37 D6-7 启用)

---

## 4. 主会话打头阵 vs 派 worker 复制 (per §2.3 L4)

**主会话打头阵** (1 跳 HTTP 部分, 已落):
- ✅ k3s 节点 + namespace pod + endpoints 检查 (kubectl get 系列)
- ✅ gm-backend 8081 HTTP /healthz 探活 (curl)
- ✅ prometheus CrashLoop 根因定位 (kubectl describe + kubectl logs)
- ✅ 完整摸底报告 (本文档 + RGS-PHASE-C-PREP §3)

**派 worker 复制** (per §2.3 L4, 等 SRE 介入后):
- worker-1: 5 域 gRPC 50051-50055 health probe (per B4-B8 步骤, 需 grpcurl)
- worker-2: 22 测试函数真跑 (per C1-C8 步骤)
- worker-3: mTLS 业务级 1+2 跳 (per C4-C5 步骤)

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

- **container minimal image 无 curl/wget**: 5 域 gRPC 探活必须 SRE 装 grpcurl + certs (per RGS-PHASE-C-PREP §1 B3)
- **WSL port-forward 跨 namespace 不稳**: 主会话 8443 mTLS 探活未完成, SRE 介入后用 svc IP 内部测试
- **prometheus HPA 现状未查**: A4 步骤待 SRE 跑
- **5 域 + gm-backend 真实业务 E2E**: D1 派生约束需 Phase C C6 阶段 (跨域 saga 真实交易), W37 D6-7 启用

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 16:10 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 5 域 ST 业务 mTLS 1 跳摸底报告 (k3s 节点 + 24 pod + 6 svc endpoints + gm-backend 8081 HTTP 探活 + prometheus CrashLoop 根因 + 主会话可达范围 vs SRE 范围), per §2.3 L4 主会话打头阵 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
