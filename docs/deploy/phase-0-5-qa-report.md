# phase-0-5-qa-report.md

# 角色：Phase 0.5 交付物复盘型 QA 报告——核验"已签字完成"的实际质量，而非实施前假设检查
# 生成：主对话（Sonnet 5）2026-08-24，基于本 session 对集群实测 + git 历史交叉核对
# 范围：Phase 0.5 全部交付物（非仅 WF-1-55.27 / 探针修复两项）——per 用户口径「QA是目前所有内容的质量管理」
# 质量框架：参照 ISO/IEC 25010:2023（产品质量模型，9 特性版，取代 2011 版 8 特性；2011→2023 更名：易用性→交互能力、可移植性→灵活性，新增 Safety）的产品质量特性分类问题，不新造分类体系
# 与本文档配套：`phase-0-5-feedback-to-agents.md`（流程类问题的逐条整改单，本报告不重复其内容，仅引用编号）
# 使用方式：接手 agent 按「行动登记区」逐项处理，处理完成前不得将 Phase 0.5 记录为「运行时验证通过」

---

## 目录

- [§0 结论先行](#0-结论先行)
- [§1 质量特性分类评估（参照 ISO/IEC 25010:2023）](#1-质量特性分类评估参照-isoiec-250102023)
  - [§1.1 可靠性（Reliability）—— 🔴 不达标](#11-可靠性reliability--不达标)
  - [§1.2 信息安全性（Security）—— 🟡 部分达标，证据分裂](#12-信息安全性security--部分达标证据分裂)
  - [§1.3 维护性（Maintainability）—— 🔴 不达标](#13-维护性maintainability--不达标)
  - [§1.4 交互能力（原易用性）/ 灵活性（原可移植性）/ 兼容性——本报告未评估](#14-交互能力原易用性-灵活性原可移植性-兼容性本报告未评估)
- [§2 运行时实测证据](#2-运行时实测证据)
  - [§2.1 探针根因——证据链（2026-08-24 实测）](#21-探针根因证据链2026-08-24-实测)
  - [§2.2 NATS / 消息基座缺失](#22-nats-消息基座缺失)
  - [§2.3 孤儿 ReplicaSet 累积](#23-孤儿-replicaset-累积)
  - [§2.4 命名空间资源配额已耗尽——影响后续滚动更新能否成功](#24-命名空间资源配额已耗尽影响后续滚动更新能否成功)
- [§3 本期做对的地方（不要在整改中被误伤）](#3-本期做对的地方不要在整改中被误伤)
- [§4 未验证事项（明确声明，避免以偏概全）](#4-未验证事项明确声明避免以偏概全)
- [§5 行动登记区（要求接手 agent 逐项处理并回填）](#5-行动登记区要求接手-agent-逐项处理并回填)
- [§6 本 session 变更声明](#6-本-session-变更声明)

## 版本历史

| 版本 | 日期 | 改动 | 作者 |
|---|---|---|---|
| v1.0 | 2026-08-24 | 初版：§0-§6 完整复盘 + ISO 25010 评估 + 行动登记 7 条 | Sonnet 5 (主对话) |
|  |  |  |  |

## 0. 结论先行

`phase-0-5-handoff.md` 记录本期状态为「形式上完成（52 文件 / 6065 行入 main），NO-GO 解除，12 角色全签」。

本 session 对集群做运行时实测，结果如下（2026-08-24 实测，见 §2）：

| 域 | Deployment READY | Pod 状态 | 说明 |
|---|---|---|---|
| admin | 0/1 | Running（52 次重启） | 探针失败持续重启，从未 Ready |
| cluster-ops | 0/3 | Running/CrashLoopBackOff（62–63 次重启） | 同上 |
| economy | 0/2 | CrashLoopBackOff（Exit 137） | 同上，根因已定位见 §2.1 |
| match | 0/3 | CrashLoopBackOff（72–75 次重启） | 同上 |
| player | 0/2 | Running（52–66 次重启） | 同上，根因已定位见 §2.1 |
| social | 0/2 | Running（63 次重启） | 同上 |

**6 个业务域，0 个 Available。** 集群里唯一稳定 Running 的是 grafana / otel-collector / prometheus / postgres——这些不是本期新增的业务能力，是可观测性基座 + 数据库。

**核心发现（结论，非猜测，见 §2.1 证据链）**：当前部署到集群的 6 个域的 Deployment spec，探针配置停留在 `4467080`/`c6a4bef`（Phase 0.5 早期 commit）的版本——原生 `livenessProbe.grpc`（**明文**，不支持 mTLS 握手）。而 `66ff53b`「gRPC 健康探针 mTLS 兼容性修复」已经合并进 main 两个 commit 之前，修复内容（`grpc_health_probe` + `-tls` + 6 域证书）本身是对的（见 §3 证据），但**从未 `kubectl apply` 到集群**。服务端因 `RGS_ALLOW_INSECURE_GRPC=0` 强制要求 mTLS 握手，探针发明文连接必然超时，kubelet 判定失活后 SIGKILL（Exit 137），如此循环。

「12 角色全签」「NO-GO 解除」记录的是**代码仓库状态**（52 文件已 merge），不是**运行时状态**（0 服务 Available）。这两者在 handoff 文档中未被区分，是本报告要指出的最主要缺陷——不是「代码写错了」，是「验证链条在'合并到 main'这一步就停了，没有人跑到'kubectl apply 之后探针真的过了'这一步，就签字确认完成」。

---

## 1. 质量特性分类评估（参照 ISO/IEC 25010:2023）

ISO/IEC 25010:2023 产品质量模型共 9 特性：功能适合性、性能效率、兼容性、交互能力（Interaction Capability，2011 版称"易用性"）、可靠性、安全性（Security）、维护性、灵活性（Flexibility，2011 版称"可移植性"）、Safety（新增，本报告不涉及运行安全/功能安全议题，不评估）。不逐项覆盖全部 9 个特性，仅列本期实测中命中的特性；每项标注证据来源与验证程度。

### 1.0 特性映射总览

下表先列出 9 个 ISO 25010:2023 特性的覆盖状态，明示哪些被本报告实测评估、哪些未被评估（避免用「未测」冒充「达标」或「不达标」），详细分析见 §1.1-§1.4：

| 特性 | 本报告覆盖? | 覆盖位置 | 未覆盖原因 |
|---|---|---|---|
| 功能适合性 | 本报告未评估 | — | 业务功能正确性由 B-CODE log 覆盖（4 份，详见 §4 #4），本报告聚焦"代码入 main vs 运行时验证"流程缺陷，不重复功能正确性测试 |
| 性能效率 | 本报告未评估 | — | 性能基准（吞吐/延迟/资源利用率）不属于本期 QA 范围；本期问题清单聚焦"服务能否起得来"，未涉及"起得来后跑多快" |
| 兼容性 | 本报告未评估 | — | 未做跨环境（dev/staging/prod）、跨 K8s 版本、跨 gRPC 客户端版本的兼容性测试 |
| 交互能力（Interaction Capability） | 本报告未评估 | — | 本期无 UI/UX 交付物（COC 控制面前端等不在范围内），无可评估的交互场景 |
| 可靠性 | ✅ 已覆盖 | §1.1 | — |
| 安全性（Security） | ✅ 已覆盖 | §1.2 | — |
| 维护性 | ✅ 已覆盖 | §1.3 | — |
| 灵活性（Flexibility） | 本报告未评估 | — | 未做跨环境部署/可移植性测试；不评估"未测" |
| Safety | 本报告未评估 | — | 本期不涉及运行安全/功能安全议题（rail/signaling/medical 等），无可评估场景 |

### 1.1 可靠性（Reliability）—— 🔴 不达标

- **可用性子特性失败**：6/6 业务域 Available=0，非单点故障，是系统性配置错误（见 §2.1）。
- **容错性子特性失败**：探针失败 → kubelet SIGKILL → Deployment controller 重建 → 循环，无降级路径、无告警熔断（虽有 otel/prometheus 但未确认是否配置了 CrashLoopBackOff 告警规则，本报告未验证，见 §5）。
- **证据强度**：**直接实测**（`kubectl get pods/deploy`、`kubectl describe pod` 崩溃时间戳精确对应探针 failureThreshold 窗口）。

### 1.2 信息安全性（Security）—— 🟡 部分达标，证据分裂

- **证书体系本身合格**：6 个域证书 SAN/CN 逐一核对，与各 manifest 中 `-tls-server-name` 参数**完全一致**（`admin.service` / `cluster-ops.service` / `economy.service` / `match.service` / `player.service` / `social.service`），无 EKU 限制导致的 clientAuth 问题（上一轮 session 已排查并证伪该猜想）。
- **`RGS_ALLOW_INSECURE_GRPC=0` 在 6 个域全部生效**，服务端日志确认 `mTLS ENABLED`，说明"禁止明文 gRPC"这条安全基线在应用层是硬执行的——这不是弱点，是本期做对的地方。
- **矛盾点**：正是因为服务端安全基线执行得"太硬"，而部署到集群的探针配置又是明文的（*未同步*的两个变更速度不一致），两者互相打架导致 §0 描述的死循环。**这不是安全设计缺陷，是发布流程缺陷**（探针修复 commit 与实际 apply 动作脱节）。
- **证据强度**：SAN/CN 比对为**直接实测**（openssl x509 逐一 dump）；EKU 排查为上轮 session 结论，本报告未重新验证，视为**已确认**（有工具输出留痕）。

### 1.3 维护性（Maintainability）—— 🔴 不达标

- WBS 进度表 128/128 长期显示 pending，与实际至少 4+ 项已完成/已合并的状态不符（详见 `phase-0-5-feedback-to-agents.md` #5）。
- 4 个活跃 worktree 全部缺 `.wbs-task-marker`，标准合并工具链 (`wbs_merge.ps1`) 对这 4 个全部失效（详见 feedback #3）。
- `wbs/WF-1-55.27-retry`（`c96efe8`，50/50 测试通过的真实修复）未合并、未登记，handoff §11.3 仍写"仅 mock 验证"这一过时描述（详见 feedback #4）。
- **证据强度**：**直接实测**（`git worktree list` + 逐一检查 marker 文件 + `git merge-base` + `cargo test` 跑分）。

### 1.4 交互能力（原易用性）/ 灵活性（原可移植性）/ 兼容性——本报告未评估

未涉及 UI/UX（COC 控制面前端等），也未做跨环境兼容性、可移植性测试，不在本期实测范围内，不做评分（避免用「未测」冒充「达标」）。§1.1 中评估的"可用性（Availability）"是可靠性（Reliability）下的子特性，与此处"交互能力"是两个不同概念，不要混淆。

---

## 2. 运行时实测证据

### 2.1 探针根因——证据链（2026-08-24 实测）

**第一步：集群实际部署的探针配置**（`kubectl get deploy player-service -o yaml`）：

```yaml
livenessProbe:
  grpc:
    port: 50051
  initialDelaySeconds: 30
  periodSeconds: 30
  timeoutSeconds: 5
  failureThreshold: 3
```

原生 k8s `grpc:` 探针类型——**无 TLS 参数，明文 gRPC 健康检查协议**。`economy-service` 部署核实为同一模式（`grpc.port: 50052`，无 TLS）。

**第二步：与 git 历史逐 commit 比对**（`git show <sha>:docs/deploy/01-k8s-manifests/01-player-service.yaml`）：

| commit | 探针类型 | 与集群实测是否一致 |
|---|---|---|
| `4467080`（step-1+5，最早） | 原生 `grpc: port: 50051`，无 TLS | ✅ **完全一致** |
| `c6a4bef`（SRE 前置修复） | 同上（未改探针） | ✅ 一致 |
| `66ff53b`（mTLS 探针修复，当前 main） | `exec: grpc_health_probe -tls -addr=127.0.0.1:50051 -tls-server-name=player.service ...` | ❌ **与集群实测不一致** |

**结论**：集群里运行的 spec 停留在 `4467080`/`c6a4bef` 版本，`66ff53b` 修复已进 main 但**从未 apply**。

**第三步：崩溃时间戳交叉验证**（`kubectl describe pod` economy-service）：

```
Started:  2026-08-24 19:04:12
Finished: 2026-08-24 19:06:42   → 存活 150s
Exit Code: 137 (SIGKILL)
```

150s = `initialDelaySeconds(30) + periodSeconds(30) × failureThreshold(3) + 探测开销` ——与"探针在 initialDelay 后开始探测，连续 3 次失败后判定失活"的时间线完全吻合，排除"OOM/panic 等其他崩溃原因"的可能性。

**验证程度**：player / economy 两个域为**直接实测三重验证**（spec 比对 + git 历史 + 崩溃时间戳）。match / social / admin / cluster-ops 未逐一重复以上三步，仅确认了 `kubectl get deploy -o wide` 层面的 0/N 状态和相同的部署时间线（同批次 apply），**推断同因**，标记为**推断，非逐一实测**——接手 agent 若要在 §6 打钩关闭该条，需对剩余 4 个域各跑一遍 §2.1 的三步核验。

### 2.2 NATS / 消息基座缺失

- `kubectl get deploy` 全量列表中**不存在** NATS 相关 Deployment/StatefulSet。
- 应用日志（上轮 session 已采集，本报告引用不重跑）：`outbox relay DISABLED — NATS connect failed: DNS error`，代码自身告警 `outbox rows will accumulate, manual recovery required`。
- handoff 文档记录 Step 2 交付 18 个 NATS/可观测性 manifest，但 NATS 一半未落地到集群。
- **证据强度**：Deployment 列表为**直接实测**；日志内容为**引用上轮 session 记录**，本报告未重新拉取确认时效性（NATS 部署状态本身在 §2.1 集群快照中已确认仍缺失，间接佐证日志记录未过期）。

### 2.3 孤儿 ReplicaSet 累积

`kubectl get rs` 实测：

- `economy-service` 存在 **4 个** ReplicaSet（`646cd7c549` `65b79dbf8` `6bc9c57dc4` `769b8cc9f6`），仅最新两个有存活 Pod（且均 0 Ready）。
- `player-service` 存在 **5 个** ReplicaSet（`546594c968` `56cccb99db` `5d6f59c79c` `6f7557645b` `9789cf4b5`）。

这是同一 Deployment 反复被不同版本 spec `apply` → 滚动更新从未 converge（新 Pod 从未 Ready）→ controller 又建下一版 ReplicaSet 的痕迹，与 §2.1「探针配置几经变动、集群状态从未追上」的结论互相印证。**建议**：待 §2.1 根因修复并验证 Ready 后，用 `kubectl rollout status` 确认收敛。**注意**：自动 GC 依赖 `revisionHistoryLimit`（当前设为 10），`player-service` 目前只有 5 个历史 RS，未达上限，**不会**被自动清理——如需清理需手工 `kubectl delete rs`，不是"等等就会自己消失"。

### 2.4 命名空间资源配额已耗尽——影响后续滚动更新能否成功

`kubectl describe quota rust-game-server-quota`（namespace `rust-game-server`）实测：

| 资源 | Used | Hard | 占用率 |
|---|---|---|---|
| `limits.memory` | 48Gi | 48Gi | **100%** |
| `limits.cpu` | 45 | 48 | 94% |
| `requests.cpu` | 11150m | 12000m | 93% |
| `requests.memory` | 12Gi | 12Gi | **100%** |

配额已被现有的 19 个（多为 CrashLoopBackOff 循环重启中的）Pod 占满。这意味着：即便 §5 行动 #1 的 manifest 修复本身完全正确，`RollingUpdate`（`maxSurge: 1`）需要在旧 Pod 终止前先调度出新 Pod，若配额没有腾出空间，新 Pod 会卡在 `Pending`（配额不足），滚动更新无法收敛——这是**独立于探针根因的第二个会阻塞收敛的因素**，必须一并处理，不能假设"探针配置一改，Pod 自然就 Ready 了"。

---

## 3. 本期做对的地方（不要在整改中被误伤）

- 6 域 mTLS 证书生成、SAN/CN 绑定、CA 链路——**实测通过**，无需重做。
- `66ff53b` 探针修复本身的探针命令写法（exec + `grpc_health_probe` + 双向 TLS 参数）——**命令写法正确**，不要重写这部分代码。**但**该 commit 同时改了 `Dockerfile`（新增 health-probe 构建阶段，拉取 `grpc_health_probe` 二进制并校验 sha256），而当前 6 个域使用的镜像 tag（如 `0.1.0-player`）是否已重新 build/push 包含这个新构建阶段，**本报告未能验证**（尝试用临时 debug pod 核实时，命名空间 `limits.memory` 配额已 48Gi/48Gi 打满，无法调度新 pod，见 §2.4）。**不要假设"apply manifest 就够了"**——务必先按 §5 行动 #1 的子步骤确认镜像内确实存在该二进制，否则会把"探针明文握手失败"的死循环换成"探针 exec: no such file or directory"的死循环，症状相似、根因不同，容易被误判为"修复无效"。
- `wbs/WF-1-55.27-retry`（`c96efe8`）Saga 补偿修复——**50/50 测试通过**，`merge-base` 确认可直接合并无冲突，是真实可用的成果，只是没被合并/登记（详见 feedback #4）。
- 5/5 fail-closed 相关验证（上轮 session 记录）——本报告未重新验证，但无证据推翻，不纳入本期问题清单。

> ⚠️ 配套风险：本期 #2 的"66ff53b 同时改 Dockerfile"风险由 §5 行动 #1 子步骤 ① 闭环处理，本节不重复。

---

## 4. 未验证事项（明确声明，避免以偏概全）

以下内容本报告**未验证**，不要把「本报告没提」误读为「已通过」：

1. match / social / admin / cluster-ops 四个域的探针根因，仅做了部署时间线层面的推断，未逐一重复 §2.1 三步验证。
2. 未构建、未推送新镜像，也未实际执行 `kubectl apply` 应用 `66ff53b` 修复后的 manifest 到集群——即"修复后探针能过"这件事**没有被证实**，只证实了"当前失败的原因是什么"。**具体未验证点**：`66ff53b` 同时改了 Dockerfile（新增 grpc_health_probe 构建阶段），当前 6 个域使用的镜像 tag 内是否已包含该二进制**未确认**（尝试用临时 debug pod 验证时命名空间配额已耗尽，无法调度，见 §2.4）。
3. 未检查 otel/prometheus 是否配置了针对 CrashLoopBackOff 的告警规则。
4. B-CODE 验证 log（4 份，handoff 记录 1 🟡 + 3 🔴）未重新核对，鉴于 §0 结论（0 服务 Available），这些 log **不可能**在当前集群状态下变绿，只能等 §2.1 修复后重跑。
5. NATS 日志时效性（见 §2.2）。

---

## 5. 行动登记区（要求接手 agent 逐项处理并回填）

| # | 问题 | 要求 | 处理 agent | commit/依据 | 状态 |
|---|---|---|---|---|---|
| 1 | 6 域探针配置停留在 `4467080`/`c6a4bef`，`66ff53b` 修复未 apply（§2.1）；镜像内是否含 `grpc_health_probe` 二进制未验证（§3）；命名空间配额已打满，滚动更新可能卡 Pending（§2.4） | **子步骤，按序**：① 确认镜像已按 `66ff53b` 后的 Dockerfile 重新 build+push（或重新触发一次），② 先处理 #3 释放部分配额空间（或申请调高 quota），③ 逐域执行 `kubectl apply -f docs/deploy/01-k8s-manifests/0{1..6}-*.yaml`，④ `kubectl rollout status` 确认收敛，⑤ `kubectl get pods` 确认 READY=N/N，全程若探针失败症状从"超时"变为"exec 找不到文件"，立即停止批量 apply，回到步骤①排查镜像 | | | ⬜ |
| 2 | match/social/admin/cluster-ops 根因未逐一验证（§2.1 末尾） | 对这 4 个域重复 §2.1 三步（spec 比对 + git 历史 + 崩溃时间戳），确认是否为同因 | | | ⬜ |
| 3 | NATS 未部署，outbox relay 长期 DISABLED（§2.2） | 部署 NATS manifest（若已存在于 18 个可观测性/消息 manifest 中，定位并 apply；若不存在，先补齐再 apply），确认 economy-service 等日志里 outbox relay 状态转为 ENABLED | | | ⬜ |
| 4 | 孤儿 ReplicaSet 累积（§2.3），不会被自动 GC（`revisionHistoryLimit=10` 未达上限） | 待 #1 收敛后用 `kubectl rollout status` 确认，再手工 `kubectl delete rs` 清理 replicas=0 的历史 RS | | | ⬜ |
| 5 | 4 份 B-CODE log 状态过时（未反映 0 Available 的现实） | 待 #1-#3 完成、服务实际 Ready 后重新执行 B-CODE 验证，更新 log 到实测结果（不要凭"代码已 merge"推定为通过） | | | ⬜ |
| 6 | handoff §0/结论 层面未区分"代码入 main"与"运行时验证通过"，导致"12 角色全签 + NO-GO 解除"被误读为运行时已验证 | 在 handoff 文档补充运行时验证章节，或至少加一条免责声明：当前签字仅覆盖代码评审，不覆盖集群运行时状态 | | | ⬜ |
| 7 | 本报告以外，`phase-0-5-feedback-to-agents.md` #1-#5（并发覆盖 / 4 worker 0 产出 / marker 缺失 / WF-1-55.27 未合并 / WBS 长期脱节）| 按该文档逐条处理，本报告不重复登记 | | | ⬜ |

---

## 6. 本 session 变更声明

本节按事件类型拆分，三类变更各自独立、时间均为 2026-08-24。

### 6.1 手工 worktree marker

- **时间**：2026-08-24
- **事件**：本 session 在 `WF-1-55-retry` worktree 中手工补写了 `.wbs-task-marker`（`l4_id: WF-1-55.27`），**这是本 session 手工创建的，不是经 `wbs_create_worktree.ps1` 标准流程生成**——后续 agent 核对该 worktree 起源时请勿把这份 marker 当作"走过标准脚本"的证据。

### 6.2 集群只读操作

- **时间**：2026-08-24
- **事件**：本报告涉及的集群查询（`kubectl get/describe`、`openssl x509` 证书 dump）均为只读操作，未对集群做任何变更（未 apply、未 delete、未 rollout restart）。曾尝试创建一次性 debug pod（`probe-bin-check`，用于核实 §3 的镜像二进制问题）被 ResourceQuota 拒绝，**未创建成功**，集群状态未受影响。

### 6.3 分支/合并状态

- **时间**：2026-08-24
- **事件**：未触碰 `wbs/WF-1-55.27-retry` 分支合并——该操作仍需人工确认后执行 `wbs_merge.ps1`（详见 feedback #4），不在本报告授权范围内。
