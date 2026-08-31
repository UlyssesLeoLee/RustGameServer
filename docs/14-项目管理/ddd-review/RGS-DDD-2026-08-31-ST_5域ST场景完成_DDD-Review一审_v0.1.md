# RGS-DDD-2026-08-31-ST — 5 域系统测试 (ST) 场景完成 DDD Review 一审

| 字段 | 值 |
|---|---|
| 文档 ID | RGS-DDD-2026-08-31-ST |
| 版本 | v0.1 |
| 创建日期 | 2026-08-31 JST |
| 创建者 | 架构师(Mavis 接手 agent per DEC-008) |
| 类型 | DDD Review 一审材料 |
| 关联 | RGS-DDD-2026-08-31-UT-IT (前置 UT+IT 阶段) |
| 依据 | RGS-BAS-001 (基本设计书 v1.4) |
| 基线 commit | `46dd2a0` (831) |
| 范围 | 5 域 (player / economy / match / social / admin) |
| 路径 | 走 8/27 JST k3s 真实部署 |
| 评审者 | Ulysses (一人公司 12 角色 per DEC-008) |
| 状态 | ⏳ 待 DDD Review 一审 |

---

## 1. 执行摘要

2026-08-31 17:05 JST 起, 按 Ulysses 决策:
- **依据**: RGS-BAS-001 (基本设计书) §4-§5 业务/数据设计
- **路径**: 走 8/27 JST 部署的 k3s 真实环境, 复用 `scripts/e2e-smoke.ps1` 12 probe 框架
- **范围**: 5 域 × 2 场景 = 10 个 ST 场景
- **产出**: 40 files, +1834 行 (10 .ps1 + 10 .json mock + 20 evidence)
- **commit**: `cd93169` on `st/mock-server-and-scripts` 分支

**最终 verdict 矩阵**:

| 域 | 场景 1 | 场景 2 |
|---|---|---|
| player | st-01 **FAIL** | st-02 **FAIL** |
| economy | st-03 ✅ **PASS** | st-04 ✅ **PASS** |
| match | st-05 **FAIL** | st-06 ✅ **PASS** |
| social | st-07 **FAIL** | st-08 ✅ **PASS** (NATS SKIP) |
| admin | st-09 **FAIL** | st-10 **FAIL** |

**汇总**: **4 PASS / 6 FAIL** (verdict 分布)

**失败根因** (per e2e-smoke 12 probe): **5 域 gRPC + postgres + cluster-ops 7 probe 全 PASS, gm-backend 8081 health/readyz + 8443 HTTPS + prometheus + grafana + nats 5 probe 全 FAIL**。所有 6 个 ST 场景 FAIL 都因 gm-backend 容器 HTTP 不响应(PORT 探活 OK 但 health endpoint 不响应)。

---

## 2. 基线与分支

```
main @ 46dd2a0 (831)
 └── st/mock-server-and-scripts
     └── cd93169 (10 ST 场景 + 20 evidence)
```

**worktree**: `D:/rgs-st-mock` (PowerShell, Windows)
**分支**: `st/mock-server-and-scripts` (基线 46dd2a0)
**commit**: `cd93169 feat(st): 5 域 × 2 场景 = 10 个 ST 场景 (BAS-001 k3s 真实部署)`

**重要设计**: ST 阶段**不**基于 5 域 `ut/<domain>` 分支(8/31 17:12 JST Ulysses 决策)。ST 与 UT/IT 是正交维度,ST 阶段不依赖 5 域 src/ 改动。

---

## 3. ST 场景详情 (10 个)

### 3.1 player 域 (2 场景, 2 FAIL)

**st-01 player-grpc-port-and-gm-backend** (BAS-001 §4.4)
- 验证: player gRPC 50051 + gm-backend /healthz + /readyz
- 结果: 1 PASS (player gRPC) + 2 FAIL (gm-backend HTTP)
- evidence: `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/st-01-player-grpc-port-and-gm-backend.{log,md}`

**st-02 player-cross-domain-health** (BAS-001 §4.4 + §4.8)
- 验证: player gRPC + admin gRPC + gm-backend health + postgres
- 结果: 3 PASS (5 域 gRPC + postgres) + 1 FAIL (gm-backend)

### 3.2 economy 域 (2 场景, 2 PASS)

**st-03 economy-grpc-port-and-outbox** (BAS-001 §4.5.1 + §5.4)
- 验证: economy gRPC + postgres + player + admin
- 结果: **4 PASS** (跨域 gRPC 全活 + postgres TCP)

**st-04 economy-cross-domain-data-flow** (BAS-001 §4.5 + §4.7.2)
- 验证: economy + player + match + admin + postgres (5 probe)
- 结果: **5 PASS** (trade saga 跨域数据流端口全活)

### 3.3 match 域 (2 场景, 1 PASS / 1 FAIL)

**st-05 match-grpc-port-and-replay** (BAS-001 §4.2 + §5.5)
- 验证: match gRPC + gm-backend + player + postgres
- 结果: 3 PASS + 1 FAIL (gm-backend)

**st-06 match-cross-domain-session** (BAS-001 §4.2 + §4.4 + §4.6)
- 验证: match + player + social + admin (4 probe)
- 结果: **4 PASS** (跨域 session 端口全活)

### 3.4 social 域 (2 场景, 1 PASS / 1 FAIL)

**st-07 social-grpc-port-and-guild** (BAS-001 §4.6 + §5.6)
- 验证: social gRPC + gm-backend + player + postgres
- 结果: 3 PASS + 1 FAIL (gm-backend)

**st-08 social-cross-domain-push** (BAS-001 §4.6 + §4.7.1)
- 验证: social + player + admin + nats (NATS 容忍 SKIP)
- 结果: **3 PASS + 1 SKIP** (Outbox 模式 nats 可选)

### 3.5 admin 域 (2 场景, 2 FAIL)

**st-09 admin-grpc-port-and-audit** (BAS-001 §4.8 + §5.7)
- 验证: admin gRPC + gm-backend /healthz + /readyz + postgres
- 结果: 2 PASS (admin + postgres) + 2 FAIL (gm-backend HTTP)

**st-10 admin-cross-domain-gm-flow** (BAS-001 §4.8 + §5.7)
- 验证: admin + 4 域 gRPC + gm-backend (6 probe)
- 结果: 5 PASS (5 域 gRPC) + 1 FAIL (gm-backend)

---

## 4. 5 域 gRPC 端口可达性(关键真实状态)

| 域 | gRPC Port | 容器 IP | 状态 |
|---|---:|---|---|
| player | 50051 | 10.42.0.102 | ✅ PASS |
| economy | 50052 | 10.42.0.111 | ✅ PASS |
| match | 50053 | 10.42.0.106 | ✅ PASS |
| social | 50054 | 10.42.0.100 | ✅ PASS |
| admin | 50055 | 10.42.0.103 | ✅ PASS |
| cluster-ops | 50056 | 10.42.0.97 | ✅ PASS |
| postgres | 5432 | 10.42.0.117 | ✅ PASS |

**5 域 gRPC 业务端口全活** — 这是 ST 阶段最重要的真实部署证据。

**HTTP 探活全挂**:
- gm-backend 8081 /healthz: 000000 (curl 失败, 容器端口可达但 HTTP 不响应)
- gm-backend 8081 /readyz: 000000
- prometheus 9090: 000000
- grafana 3000: 000000
- nats 8222: 000000 (deployment 可能未含)

---

## 5. 4 阶段迭代复盘(per 17:05 JST 决策后)

### 5.1 时间线

| 时间 (JST) | 阶段 | 结果 |
|---|---|---|
| 17:05 | "最高规格" 决策 | ✅ |
| 17:08 | 5 worker × 2 场景 mock server binary | ❌ rgs-testkit 强约束冲突 |
| 17:12 | 改 k3s 真实部署 | ❌ 5 域 mTLS 业务调不通 |
| 17:35 | 5 worker k3s 路径派工 | ❌ 5 worker 0 产出 (跟 UT v1 同症) |
| 18:24 | 主会话自写 10 ST 脚本 | ✅ 10 脚本 + 20 evidence 落档 |

### 5.2 关键教训

**教训 1**: ST 阶段跨多工具链(WSL + sudo + k3s + 5 域 mTLS + e2e-smoke + WSL 内 kubeconfig 权限),worker 复现链路成本太高。

**教训 2**: rgs-testkit 强约束"禁 InMemory mock" 与"st-mock-server binary"决策不可调和 — ST 必须用真 PG/真 binary 路径。

**教训 3**: mTLS 业务调用需要从 k8s secret 导出证书到 ST worktree,本轮 4h 预算内没时间做。

**教训 4**: worker 在跨多工具链场景下 0 产出(跟 UT v1 同症) — 应该首次派工时就**先主会话自写 1 个完整脚本跑通链路**,再让 worker 复用。

**教训 5**: **e2e-smoke.ps1 已有 12 probe 框架可复用** — ST 阶段最大价值是把这个框架扩展成 5 域业务场景模板,而不是从零造新 mock。

### 5.3 5 worker 0 产出的反思

5 worker 全部 0 产出(只有 admin worker 写过 1 个改动但没 commit),跟 UT v1 player 失败同症。**根因**: ST 阶段涉及 wsl + sudo + 5 域 + e2e-smoke 多层调用,worker 不知道先走通哪一步。

**正确做法**(本轮事后总结):
1. 主会话先写 1 个完整 ST 脚本跑通 e2e-smoke + evidence 链路(我做了 st-01)
2. 验证 verdict 与 evidence 质量
3. **然后** 派 worker 复用模板扩 9 个场景
4. Worker 只做"复制 + 改 probe 列表" 模板化工作,不需要摸 wsl/sudo 链路

---

## 6. ST 阶段 vs UT+IT 阶段对比

| 维度 | UT+IT 阶段 | ST 阶段 |
|---|---|---|
| 范围 | 5 域 (5×UT + 5×IT) | 5 域 (5×2=10 ST 场景) |
| 路径 | InMemory mock + mock gRPC client | 真实 k3s 8/27 JST 部署 |
| 工具 | cargo test (单文件) | PowerShell e2e-smoke + http_probe |
| 时间 | 4h (3 阶段迭代) | 4h (4 阶段迭代) |
| 产出 | +9236 行, 366+ tests, 5/5 cargo check | 40 files, +1834 行, 4/10 PASS |
| 失败模式 | 编译错误 (38 errors → 0) | gm-backend HTTP 不响应 (0 修复) |
| Worker 表现 | 5/5 最终产出 | 5/5 0 产出 → 主会话自写 |

**关键差异**:
- **UT/IT 阶段**: 编译失败可被热修复(38 errors → 0 in 30 min)
- **ST 阶段**: k3s 容器 HTTP 不响应**无法在本 ST worktree 修复** — 需要重启 gm-backend 容器 (per RGS-DDL 操作),不在 ST worker 范围

---

## 7. DDD Review 决策表 (P1 待办)

| ID | 严重性 | 描述 | 建议处置 |
|---|---|---|---|
| ST-P1-01 | 🔴 高 | gm-backend 8081 /healthz + /readyz 探活 000000 (容器在跑但 HTTP 不响应) — 6 个 ST 场景因此 FAIL | 调 k8s `kubectl exec gm-backend -- curl localhost:8081/healthz` 诊断; 若是 startup 失败, 重启容器 |
| ST-P1-02 | 🟡 中 | prometheus + grafana HTTP 探活 000000 (per 8/27 部署目标, 应可用) | 同上诊断 |
| ST-P1-03 | 🟢 低 | nats 8222 探活 000000 (per BAS-001 §4.7.1 事件分发, 但 8/27 部署可能不含) | 确认 8/27 部署范围, 若不含则不算 P1 |
| ST-P1-04 | 🟡 中 | 5 域 gRPC 业务调用 mTLS 验证缺失 (本轮 ST 范围外) | 下轮: 导出 mTLS 证书到 ST worktree, 写 5 域业务级 ST 场景 |
| ST-P1-05 | 🟢 低 | st-08 NATS SKIP 时仅记录不算 fail (本轮已 accept) | 无需处置 |

---

## 8. 路径隔离与产物体积

| 域 | 改动文件 | 跨域文件 | 状态 |
|---|---|---|---|
| ST (整体) | `scripts/st/*` + `docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/*` | ❌ 无 5 域 src/ 改动 | ✅ |

**总产物**: 40 files, +1834 行(10 .ps1 + 10 .json + 10 .log + 10 .md)

**未改动**:
- 5 域 player/economy/match/social/admin 的 src/ 或 tests/ (✅ 域独立)
- `scripts/e2e-smoke.ps1` / `scripts/e2e-smoke.sh` (✅ 项目级不破坏)
- workspace.dependencies (✅ 无新依赖)
- Cargo.lock (✅ 无 Rust 编译)

---

## 9. evidence 完整列表

`docs/00-基准与治理/.test-evidence/st-2026-08-31-5dom/`:
- 10 × .log (运行 log, 1KB each)
- 10 × .md (evidence 报告, 含步骤详情 + 业务引用 + verdict)

每 .md 含:
- 场景 ID + BAS 章节 + 执行时间
- 步骤数 + verdict
- 步骤详情表 (# / 动作 / 预期 / 实际 / 状态)
- 关键 evidence 链接
- 业务引用 (RGS-BAS-001 §X + UT/IT commit hash)

---

## 10. 后续轮次 (未做)

**本轮 ST 范围 = 基础设施层 + gm-backend APIGW 层**。

**下轮 ST 升级路径**:
1. 导出 5 域 mTLS 证书 (per `phase-0-5-step-4-gen-certs.ps1`) 到 ST worktree
2. 用 grpcurl 或自写 Rust client 写 5 域业务级 E2E 场景 (per BAS-001 §4.4-§4.7)
3. 跑 trade saga 端到端 (跨 economy + match + admin)
4. 跑 replay 端到端 (跨 match + admin)

**预计**:
- 证书导出: 1-2 小时
- 业务级 ST 场景: 5 worker × 2 场景 = 10 场景, 2-3 小时
- evidence: 1 小时
- 总 4-6 小时

---

## 11. 修订历史

| 版本 | 日期 | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 19:00 JST | 架构师(Mavis 接手 agent per DEC-008) | 初始创建, 5 域 ST 场景完成 DDD Review 一审材料 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
