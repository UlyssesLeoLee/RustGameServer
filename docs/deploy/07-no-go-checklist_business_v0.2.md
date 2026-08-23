# 07-no-go-checklist_business_v0.2.md — PH-1 阶段业务能力 NO-GO 自检表（业务侧独立判定 v0.2）

## 文档元数据

| 字段 | 值 |
|---|---|
| 文档 ID | RGS-DEPLOY-NO-GO-CHECKLIST-BIZ-001 |
| 版本 | **v0.2**（Phase 0.5 实测后实测状态更新）|
| 制定日 | 2026-08-24（v0.1 → v0.2 升版） |
| 制定者 | 架构师（Ulysses，1 人公司 DEC-008） |
| 适用许可 | Apache-2.0 |
| 关联 | RGS-REQ-001 §11.2（PH-1 阶段定义）+ RGS-INC-001 v0.2 §1.4/§1.5/§2（现状基线 v0.2 勘误）+ `07-no-go-checklist_v0.4.md`（环境就绪侧，**业务侧独立**）+ `RGS-DEC-NOGO-001 v0.1`（一人公司 12 角色全签 NO-GO 形式解除）+ `RGS-PLAN-001 v0.9` |

---

## 0. 重要声明

> ⚠ 本表与 `07-no-go-checklist_v0.4.md` **正交**：v0.4 解决"环境是否就绪"（k3s / PG / Rust / Cargo.lock），本表解决"PH-1 业务能力是否就绪"（可观测性 / 鉴权 / 会话 / trace）。
>
> **两份独立签字**：环境 NO-GO 解除 ≠ 业务 NO-GO 解除。本表 0 项 ✅ 之前**禁止 Phase 10 production rollout**、**禁止执行任何依赖"5 业务域 gRPC 互通"的测试**。
>
> v0.4 "GO" 状态维持，但 v0.1（业务）默认 **🔴 NO-GO**。

---

## 1. 4 条业务判定（B-CODE）

| B-CODE | 描述 | 关联 | 当前状态 | 责任人 | 关闭条件 |
|---|---|---|---|---|---|
| **B-CODE-01** | **可观测性基础**——K3s 上 OTel Collector + Prometheus + Grafana 三个 Pod running；5 业务域 Pod 启动后 OTel 链路能跨 Service 串联 | RGS-REQ-001 §11.2 PH-1 + NFR-OP-001 Lv.4 + RGS-INC-001 §18 | 🟡 **部分 NO-GO**（14 K8s resources apply OK / 3 Deployment Scaled / 3 PVC Bound / 0/3 Pod Running 因 ImagePullBackOff） | SRE 一人公司 (Ulysses per DEC-008) | OTel Collector + Prometheus + Grafana K8s manifest 落地；5 业务域 Pod 启动后能在 Tempo / Loki / Grafana 上看到完整 trace_id |
| **B-CODE-02** | **登录鉴权可用**——player-service Pod 1+ running；通过 player.proto HealthCheck + GetPlayer gRPC 调用返回 0（非 error） | RGS-REQ-001 §11.2 PH-1 判定标准 + ARC-005 | 🔴 **NO-GO**（5 业务域镜像未推 + B-CODE-01 OTel 不 Running） | player 域 Lead 一人公司 (Ulysses per DEC-008) | player-service Pod Running + gRPC HealthCheck OK + GetPlayer 对一个测试 account 返回 200/OK |
| **B-CODE-03** | **会话创建**——能完成 login → issue session_epoch → 写 player_db 落库；gRPC 调用 trace_id 能贯穿 player → PG | RGS-REQ-001 §11.2 PH-1 判定标准 + ARC-005 + ARC-006 | 🔴 **NO-GO**（同 B-CODE-02 阻塞） | player 域 Lead 一人公司 (Ulysses per DEC-008) | 端到端脚本：发起 login → 验 PG 落库 → 验 trace_id 在 3 个 span 内（gRPC + sqlx + response） |
| **B-CODE-04** | **trace 全链路打通**——任意 gRPC 调用从 client → service → DB 端到端可由一个 trace_id 串联展示在 Grafana | RGS-REQ-001 §11.2 PH-1 判定标准 + NFR-OP-001 + NFR-OP-002 | 🔴 **NO-GO**（OTel Collector ImagePullBackOff + 5 业务域镜像未推） | Platform 架构师 一人公司 (Ulysses per DEC-008) | 在 Grafana / Tempo 中输入 trace_id 看到完整调用树；至少覆盖 player + economy 两个域 |

**汇总**：**0/4 Closed**（B-CODE-01 🟡 部分 / B-CODE-02/03/04 🔴 失败）。**🔴 NO-GO（实质）**。

---

## 2. 工具链与依赖（per RGS-IMPL-006 §4 必装）

| 工具 | 当前 | 关闭条件 |
|---|---|---|
| cargo-deny | ❌ NOT_INSTALLED | `cargo deny check` PASS |
| cargo-audit | ❌ NOT_INSTALLED | `cargo audit` 无 RUSTSEC 公告 |
| cargo-llvm-cov | ❌ NOT_INSTALLED | `cargo llvm-cov --workspace` 报告 |
| helm v3.10+ | ❌ WSL_ERROR | `helm version` 报 v3.10+ |
| kubectl client ≥ v1.30 | ❌ WSL_ERROR | `kubectl version --client` 报 v1.30+ |
| protoc 33+ | ✅ v33.4 | 已满足 |
| sqlx-cli 0.8+ | ✅ v0.8.6 | 已满足 |

> 数据来源：`docs/deploy/07-env-verification.log`（v0.4 环境实测同期 log）；cargo-deny / helm / kubectl 4 项 ❌ 状态直接照搬。

---

## 3. 业务 NO-GO 解除条件

- 🔴 → 🟡：**B-CODE-01~04 全部 Closed** + 工具链 4 项 ❌ 全部 ✅
- 🟡 → 🟢：在 staging 跑通 7×24h 无 P1 + Phase 1 Benchmark 准入门槛

### v0.2 Phase 0.5 实测进展（2026-08-24 实际跑通的部分）

| 阻塞 | 进展 | 剩余 |
|---|---|---|
| 11 K8s manifest 全部 PLACEHOLDER | ✅ 全部替换为实际值（per Phase 0.5 Step 1 worker commit `4467080`）| 既有 manifest (00-10) 仍含部分 PLACEHOLDER(非部署阻塞)|
| NATS/OTel/Prom/Grafana 0 Pod | ✅ 14 K8s resources apply OK + 3 Deployment Scaled + 3 PVC Bound | 3 Pod 全部 ImagePullBackOff（gcr.io + docker.io 防火墙拦截）|
| 5 业务域 0 Pod | ✅ 5 业务域 manifest 实际值就位 + rgs-certgen 6 域证书生成 PASS + 7 Secret 模板就位 | 5 业务域镜像未推（ghcr.io 需真实 PAT）|
| mTLS fail-closed 验证 | ✅ 5/5 业务域 release binary 实测 fail-closed PASS（exit=1 不静默降级） | K3s 内 opt-out 场景待镜像就位后实跑 |
| 4 B-CODE 实测 log | ✅ 4 份 log 全部生成（`b1-otel-pod-up.log` / `b2-player-grpc-healthcheck.log` / `b3-session-pg-trace.log` / `b4-cross-domain-trace.log`）| log 内容反映实际状态，不假装通过 |

**v0.2 关键事实**：NO-GO 形式上解除（per `RGS-DEC-NOGO-001 v0.1` 一人公司 12 角色全签），但 4 B-CODE 实质阻塞**未解除**——需 SRE 接力：
1. **ghcr.io 推 6 业务域镜像**（需 GITHUB_TOKEN / GHCR_PAT）
2. **K3s imagePullSecret 配通**（让 5 业务域 Pod 能拉镜像）
3. **apply 5 业务域 Deployment + 7 Secret**（镜像就位后 Pod 立即 Running）
4. **apply 18 套可观测性 manifest**（已 apply，ImagePullBackOff 解决后 Pod 立即 Running）
5. **5 业务域 Pod Running → 重跑 4 份 B-CODE log 验证 → 4 B-CODE 🟢 Closed**

---

## 4. 责任人占位(per DEC-008 一人公司治理基线)

> **本 NO-GO 解除决议 + 4 B-CODE 实测 log + Phase 0.5 全部交付物** 由 Ulysses 兼任 12 类角色实际签署。

| # | 角色 | 姓名 + 职能 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | **Ulysses(架构师)** | 2026-08-24 | ✅ 实际签:确认 4 B-CODE 状态 + Phase 0.5 形式完成 |
| 2 | SRE Lead | **Ulysses(SRE)** | 2026-08-24 | ✅ 实际签:K3s 部署 + 4 BLOCK 解决方案 |
| 3 | DBA Lead | **Ulysses(DBA)** | 2026-08-24 | ✅ 实际签:PG 18.6 6 库 + migration 0 失败 |
| 4 | QA Lead | **Ulysses(QA)** | 2026-08-24 | ✅ 实际签:4 B-CODE 实测 log(1 🟡 + 3 🔴) |
| 5 | Platform Engineer | **Ulysses(Platform)** | 2026-08-24 | ✅ 实际签:18 套可观测性 manifest 部署 |
| 6 | Player 域 Lead | **Ulysses(player 域 Lead)** | 2026-08-24 | ✅ 实际签:player DTL-018 + B-CODE-02/03 阻塞 |
| 7 | Economy 域 Lead | **Ulysses(economy 域 Lead)** | 2026-08-24 | ✅ 实际签:Q-003 Saga + DTL-015/016 + B-CODE-04 |
| 8 | Match 域 Lead | **Ulysses(match 域 Lead)** | 2026-08-24 | ✅ 实际签:DTL-026 |
| 9 | Social 域 Lead | **Ulysses(social 域 Lead)** | 2026-08-24 | ✅ 实际签:DTL-019/020 |
| 10 | Admin 域 Lead | **Ulysses(admin 域 Lead)** | 2026-08-24 | ✅ 实际签:DTL-031 v0.2 已审 |
| 11 | 评审主持人 | **Ulysses(评审主持人)** | 2026-08-24 | ✅ 实际签:REV-003 §7.3 12 类签字闭环 |
| 12 | PM | **Ulysses(PM)** | 2026-08-24 | ✅ 实际签:风险接受 + 资源 + Phase 1 授权 |

**依据**:`docs/00-基准与治理/RGS-DEC-NOGO-001_v0.1.md` §2(per DEC-008 一人公司 = 1 人 12 职责 = 真实人真实职责,不是"兼任压缩")。
**关联**:`RGS-PLAN-001 v0.9` §3.3 + `docs/deploy/phase-0-5-handoff.md` §10 12 角色全签。
**与 v0.4 环境自检的关系**:环境自检 7 G-CODE 已 Closed(per 2026-08-22 实测);本表 4 B-CODE 1 🟡 + 3 🔴,待 SRE 接力(per handoff §5 5 步)。

---

## 5. 与 v0.4 环境自检的关系

| 维度 | v0.4 环境自检 | v0.1 业务自检（本表） |
|---|---|---|
| 关注点 | K3s / PG / Rust / Cargo.lock / CI 工具链 | 5 业务域 Pod / OTel 链路 / 登录鉴权 / trace |
| 解除依据 | 7 G-CODE 全 Closed（per 实测 2026-08-22） | 4 B-CODE 全 Closed |
| 当前状态 | 🟢 GO（2026-08-22） | 🔴 NO-GO（2026-08-23 制定） |
| 谁先解除 | 已解除 | **必须本表解除后**，Phase 10 production rollout 才能进行 |
| 关联 log | `07-env-verification.log` + `08-measure-env-setup.log` | （待生成：`b1-otel-pod-up.log` / `b2-player-grpc-healthcheck.log` / `b3-session-pg-trace.log` / `b4-cross-domain-trace.log`） |

---

## 6. 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-23 | 架构师（Ulysses） | 初版。4 条 B-CODE 业务判定 + 工具链 + 与 v0.4 关系。 |
| **0.2** | **2026-08-24** | **架构师（Ulysses）** | **Phase 0.5 6 步部分完成后升版**：① B-CODE-01 状态从 🔴 0/4 变 🟡 部分（14 K8s resources apply OK / 3 PVC Bound / 0/3 Pod Running）② B-CODE-02/03/04 维持 🔴 但附实测 log 引用（`docs/deploy/b1..b4-*.log`）③ 工具链 4 项 ❌ 维持 ❌（gcr.io + docker.io 防火墙拦截 / ghcr.io 需真实 PAT）④ 4 份实测 log 全部生成（不假装通过）⑤ 责任人均改为 Ulysses(per DEC-008 一人公司 12 角色兼任)⑥ 解除阻塞 5 步：ghcr.io 推镜像 / K3s imagePullSecret / apply 5 业务域 / apply 18 manifest / 重跑 4 份 log。**未变更**：环境侧 v0.4 GO 状态；ARC-005/006/008 5 域边界；RGS-PLAN-001 v0.9 NO-GO 形式解除 + 4 B-CODE 实质未解除 状态。 |

---

## 附录 A. 关联文档

- 详细 NO-GO checklist（环境侧）：`docs/deploy/00-prerequisites/00-no-go-checklist_v0.2.md`
- 顶层 summary（环境侧）：`docs/deploy/07-no-go-checklist_v0.4.md`
- 现状基线 v0.2 勘误：`RGS-INC-001 v0.2 §1.4 / §1.5 / §2`
- PH-1 阶段定义：`RGS-REQ-001 §11.2`
- K3s 部署 log：`docs/deploy/09-deploy-dev-k3s.log`
- 环境实测 log：`docs/deploy/08-measure-env-setup.log` / `docs/deploy/07-env-verification.log`
- 治理：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` + `RGS-EXEC-001 v0.3`
- 决策：`DEC-008`（一人公司治理基线）+ `DEC-009` + `DEC-010`