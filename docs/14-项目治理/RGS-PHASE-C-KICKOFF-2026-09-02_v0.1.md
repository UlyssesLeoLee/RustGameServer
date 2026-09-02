# RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 — Phase C 阶段 A 启动公告

> **创建日期**: 2026-09-02 17:33 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/2 17:32 JST 拍板 (进 W37 / Phase C 阶段 A 启动) + RGS-PHASE-C-PREP-2026-09-02 v0.1
> **配套**: RGS-PHASE-C-PREP-2026-09-02 v0.1 (4 阶段 23 步) + RGS-K3S-CLUSTER-STATUS-2026-09-02 v0.1 (集群摸底)
> **作用域**: Phase C 阶段 A 启动 (W37 D2 = 2026-09-09 JST) + 阶段 A 4 步派工 SRE

---

## 0. 启动条件 (per 9/2 17:32 JST 拍板)

| 条件 | 当前状态 | 启动判断 |
|---|---|---|
| Phase C 准备包 4 阶段 23 步落地 | ✅ `RGS-PHASE-C-PREP-2026-09-02_v0.1.md` | ✅ |
| 集群摸底报告落地 | ✅ `RGS-K3S-CLUSTER-STATUS-2026-09-02_v0.1.md` | ✅ |
| D1 派生约束三件套就绪 | ✅ AGENTS.md v0.6.2 §2.1 L1/L1.1/L1.2 | ✅ |
| 5 域 ST 业务 mTLS HTTP 探活 | ✅ gm-backend 8081 /healthz | ✅ (1 跳) |
| 5 域 ST 业务 mTLS gRPC 探活 | ⏳ 依赖 SRE 介入 (per RGS-PHASE-C-PREP §1 阶段 B) | 🟡 |

**结论**: 阶段 A 立即可启动 (W37 D2 = 2026-09-09 JST), 阶段 B/C 启动需 SRE 介入完成阶段 A 全部 4 步.

---

## 1. 阶段 A 4 步派工 (W37 D2 = 2026-09-09 JST)

> **SRE 派工原则** (per 8/21 JST 拒绝兼任基线 + 9/1 14:15 JST PT 派工基线):
> - SRE Lead 独立派工, Mavis 不可代签 SRE 派生决策
> - 阶段 A 4 步全在 SRE 范围 (per RGS-PHASE-C-PREP §1)
> - Mavis 责任: 写启动公告 + 监控阶段 A 完成 + 阶段 B/C 准备

### 1.1 A1 节点状态 (SRE, 5 min)

- **任务**: `kubectl get nodes -o wide` 验证 ulyssespc Ready
- **期望**: ulyssespc Ready control-plane 持续 (当前 31h)
- **风险**: 节点 NotReady → 阶段 A 全部 4 步暂停, SRE 排查 k3s 进程
- **DoD**: 1 commit 落地 + 节点截图

### 1.2 A2 namespace pod 状态 (SRE, 10 min)

- **任务**: `kubectl get pods -A -o wide` 验证 rust-game-server + kube-system 24 pod 状态
- **期望**: 22 Running / 1 CrashLoopBackOff (prometheus 已知异常, 阶段 A3 修复)
- **风险**: 新增 CrashLoopBackOff → 暂停阶段 A, SRE 排根因
- **DoD**: 1 commit 落地 + 24 pod 状态表

### 1.3 A3 prometheus ReplicaSet 缩容 (SRE, 30 min, 本次关键修复)

- **任务**: 修复 prometheus-84c47f7669-qnf4q CrashLoopBackOff (per RGS-PHASE-C-PREP §1 + RGS-K3S-CLUSTER-STATUS §3.5)
- **根因** (已定位): 2 ReplicaSet 都 desired=1 (`prometheus-585fc54cfb` 1/1/1 + `prometheus-84c47f7669` 1/1/0), 部署滚动中断
- **修复命令**:
  ```bash
  kubectl scale deploy prometheus --replicas=0 -n rust-game-server
  kubectl delete pod prometheus-84c47f7669-qnf4q -n rust-game-server
  kubectl delete pod prometheus-84c47f7669 -n rust-game-server  # 删 RS
  kubectl scale deploy prometheus --replicas=1 -n rust-game-server
  ```
- **期望**: prometheus 1/1 Running, 0 CrashLoopBackOff, lock DB directory 错误消失
- **风险**: PVC 锁竞争 / 数据丢失 (建议先 backup PVC `kubectl get pvc -n rust-game-server prometheus-data -o yaml > backup.yaml`)
- **DoD**: 1 commit 落地 + prometheus `/-/ready` 200 OK

### 1.4 A4 HPA / minReplicas 检查 (SRE, 15 min)

- **任务**: `kubectl get hpa -A` 验证 HPA minReplicas ≤ desiredReplicas
- **依据** (per §2.5 L6 ST FAIL 教训): HPA minReplicas 强启动风暴 = 容器在跑但 HTTP 不响应, 表现跟 prometheus CrashLoop 一致
- **期望**: 0 HPA 资源 或 HPA minReplicas=1 (5 域 svc 已正常, 无强启动风险)
- **风险**: HPA minReplicas > desiredReplicas → SRE 拍板缩容
- **DoD**: 1 commit 落地 + HPA 状态表 (空/正常/异常)

---

## 2. 阶段 A 完成解锁阶段 B (W37 D3-5)

| 阶段 A 4 步完成 | 阶段 B 启动判断 |
|---|---|
| A1 节点 Ready | ✅ SRE 拍板 |
| A2 24 pod 状态 (除 prometheus 已知) | ✅ SRE 拍板 |
| A3 prometheus 1/1 Running | ✅ SRE 拍板 |
| A4 HPA 0 异常 | ✅ SRE 拍板 |

**全部 ✅** → 阶段 B (5 域 mTLS 业务级 ST, 8 步) 启动 (W37 D3-5, SRE 工作量 1.5 天).

---

## 3. 阶段 A SRE 拍板依赖 (per 14:58 JST 拍板规则)

### 3.1 SRE Lead 拍板项 (4 选 1+)

| 选项 | 含义 | 后续 |
|---|---|---|
| A. 立即启动阶段 A 全 4 步 (W37 D2) | SRE 接受 1 步到位 | Mavis 写 RGS-PHASE-C-A-COMPLETE-* |
| B. 分步启动 (A1 → A2 → A3 关键 → A4) | SRE 想逐拍 | Mavis 跟每步 commit |
| C. 推迟到 W38 (待 5 域 E2E 准备就绪) | SRE 时间不够 | Mavis 写 RGS-PHASE-C-DEFER-* |
| D. SRE Lead 拒绝 (改派架构师) | SRE 不可达 | Mavis 写 RGS-PHASE-C-REASSIGN-* |

### 3.2 拍板依据 (per 14:58 拍板规则)

- **A 推荐**: 阶段 A 全 4 步都 5-30 min, 集中 W37 D2 1.5h 完工, 阶段 B/C 立即可启动
- **B 次推**: 9/1 PT 派工 8 worker 25 min 完工基线, 但 4 步串行 1.5h = A1+A2+A3+A4 节奏正常
- **C 兜底**: 5 域 E2E 准备需要 RGS-TEST-RUN-PLAN v0.1 补齐, 跟 SRE 时间不冲突
- **D 极端**: SRE 不可达才用, 8/21 JST 拒绝兼任基线不允许

---

## 4. 派生约束守护 (per AGENTS.md v0.6.5 §8 + v0.7 即时增段)

| 派生约束 | 阶段 A 守护 |
|---|---|
| L1 cargo check 0 error | N/A (本批 SRE 阶段 A 不动 Rust) |
| L11 cargo build dir lock | N/A |
| L12 临时 log 不入 commit | pre-commit hook 兜底 |
| L13 自指字段 deferred 实时查询 | 引用 RGS-PHASE-C-PREP v0.1 §1 阶段 A 全 git 实证 |
| L14 plumbing brace 跟踪 | N/A |
| 8/27 11:06 JST 凭据硬 ban | 文档无 env value 痕迹 (PVC backup.yaml 不打印内容) |
| 9/2 10:18 JST B2 派生约束 L1-L14 冻结 6 个月 | 阶段 A 不动派生约束 |
| 9/2 10:18 JST C1 batch 域 v0.1 冻结 | 阶段 A 不动 batch 域 (业务 mTLS 5 域 + gm-backend) |

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

- **SRE Lead 时间窗口**: 阶段 A 全 4 步 = 1h 估算, SRE 拍板后 Mavis 跟进度, SRE 不可达则走 C 兜底
- **A3 prometheus PVC 数据备份策略**: 当前建议先 `kubectl get pvc -o yaml > backup.yaml`, 但 prometheus data 已写 27h, 备份大小 ~几 GB, 实际要不要备份待 SRE 拍板
- **A4 HPA 资源未列**: 当前集群无 HPA (per RGS-K3S-CLUSTER-STATUS §1.2), 但 SRE 真跑可能发现 ingress / cert-manager 等 HPA
- **阶段 A 完成后阶段 B 启动节奏**: 阶段 B 8 步含 grpcurl 安装 + certs 导出, 1.5 天 (W37 D3-5), 跟 5 域 E2E 准备并行

---

## 6. 后续工作 (W37 D2 起, 5 天)

| Day | 任务 | 负责 | DoD |
|---|---|---|---|
| W37 D1 (9/8) | RGS-WEEKLY-2026-W37 v0.1 (D4 派生约束) | Mavis | 业务 vs 治理双指标, 沿用 v0.3 模板 |
| W37 D2 (9/9) | **Phase C 阶段 A 全 4 步** | **SRE Lead** | 1 commit 落地 (per A1-A4) + 阶段 A 完成 |
| W37 D3 (9/10) | 阶段 B 启动: 5 域 certs 导出 | SRE Lead | 6 cert yaml 文件 (per B1-B2) |
| W37 D4 (9/11) | 阶段 B 中段: grpcurl 安装 + player/economy health probe | SRE Lead | 2 域 health probe (per B3-B5) |
| W37 D5 (9/12) | 阶段 B 收口: match/social/admin health probe | SRE Lead | 3 域 health probe + 阶段 B 完成 (per B6-B8) |
| W37 D6 (9/13) | 阶段 C 启动: 11 UT 真跑 | SRE Lead + Mavis | 11/11 PASS (per C1) |
| W37 D7 (9/14) | RGS-WEEKLY-2026-W37 v0.3 (D4 派生约束) + 阶段 C 11 E2E 准备 | Mavis + SRE Lead | 11 E2E 准备 (per C2) |
| W38 D1-D2 (9/15-16) | 阶段 C 11 E2E 真跑 + 跨域 saga 真实交易 | SRE Lead + Mavis | 22/22 PASS (per C3-C8) |
| W38 D3 (9/17) | 阶段 D 评审启动 | Mavis + Ulysses | 5 域 E2E 跑通 = 业务里程碑 (per D1) |
| W38 D4 (9/18) | batch 域 v0.1 解冻公告 (per C1 派生约束触发条件) | Mavis + Ulysses | `RGS-BATCH-V0.1-UNFREEZE-2026-09-18_v0.1.md` |
| W38 D5 (9/19) | RGS-CRITIQUE-IMPROVEMENT v0.2 升版 | Mavis 自审 + Ulysses 二审 | 5 大问题重新评估 + 业务里程碑定义 |

---

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 17:33 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: Phase C 阶段 A 启动公告 (启动条件 + 4 步派工 SRE + 阶段 A 完成解锁阶段 B + SRE 拍板 4 选项 + 派生约束守护 + 已知缺口 + W37 后续工作 5 天), per 9/2 17:32 JST 拍板 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
