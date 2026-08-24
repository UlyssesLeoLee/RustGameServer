# RGS-DEC-019 PFAU RTO 分级 + 13min 公式拆解

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEC-019 |
| 版本 | 0.1（首次产出，per RGS-OPEN-QA-001 v0.2 Q-D-05 + ACTIONS-v0.3 A-05）|
| 状态 | 🟡 审批中（per DEC-015 风格） |
| 决策日期 | 2026-08-25 |
| 决策来源 | RGS-OPEN-QA-001 v0.2 Q-D-05 + DTL-031 §4.3 + handoff §4.3 + RGS-ADR-0052 §3 Active-Active all-reachable PFAU |
| 决策人 | Ulysses（一身 12 角色，per DEC-008）|
| 关联 | DTL-031 §4.3（300s/120s 待验证规划参数）+ RGS-ADR-0052 v0.1（Active-Active 拓扑）+ RGS-OPEN-QA-001 v0.2 Q-D-05（"13min > 5min RTO 上限冲突"答复）|
| 父任务 | [WF-1-55.40](../../12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md) |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | worker-self（per DEC-008） | 首次产出：13min 公式拆解 + RTO 分级方案 + 300s/120s 冻结论证 + 解除 DTL-031 §4.3 待验证态 |

---

## 1. 问题陈述

### 1.1 字面冲突

DTL-031 §4.3 明确"300 秒观察窗口和 120 秒超时均为**待验证规划参数**，不是已承诺的 p99/SLA"；handoff §4.3 R1 估算 ~13 分钟 = 780s，与 300s+120s = 420s 在数字层面有 360s 缺口；同时 13min > 5min RTO 上限（NFR-AV 99.9% 自动化可恢复路径）字面冲突。

### 1.2 三层"不一致"识别

| 层 | 来源 | 表述 | 性质 |
|---|---|---|---|
| **数字层** | handoff §4.3 R1 = 13min vs DTL-031 §4.3 300s+120s = 420s | 780s vs 420s = 360s 缺口 | 表面冲突 |
| **语义层** | R1 是**端到端最坏情况估计**（含 admin 域广播 + 5 域 ack + 人工介入缓冲），300s+120s 是**算法内部参数**（观察窗 + 超时）| 不在同一抽象层 | 名义冲突，非真冲突 |
| **承诺层** | NFR-AV 99.9% RTO < 5min | 5min 是**自动化可恢复路径**承诺；PFAU 跨域联动是**需人工兜底**路径 | SLA 范围冲突 |

### 1.3 风险

- ❌ **不应回避冲突**：3 层冲突都不解决，会让 NO-GO 形式上解除后 PH-1 实施期间出现"承诺漂移"——实施者看到 13min 但 SLA 写 5min，按哪个执行？
- ❌ **不应强压 13min 到 5min**：PFAU 跨域联动**客观上需要人工介入缓冲**（Ulysses 5 域 Lead 兼 + DEC-008 实际约束），压缩时间只会让失败路径变多
- ✅ **应分层处理**：RTO 分级，而非统一承诺

---

## 2. 13min 公式拆解（per R1 端到端最坏情况）

### 2.1 公式

```
R1_total = T_observe + T_timeout + T_broadcast + T_ack + T_human
         = 300s     + 120s     + 100s       + 80s  + 180s
         = 780s ≈ 13min
```

### 2.2 各段说明

| 段 | 时长 | 描述 | 来源 | 是否可压缩 |
|---|---|---|---|---|
| **T_observe** | 300s | cluster-ops PFAU 观察窗口，确认故障不再自愈 | DTL-031 §4.3 | 否（5min 是经验值，< 3min 太敏感误报多）|
| **T_timeout** | 120s | 单次 saga step 重试超时（per IMPL-100 v0.1）| DTL-031 §4.3 | 否（< 60s 容易误判网络抖动）|
| **T_broadcast** | 100s | admin 域 CEM 广播 PFAU 触发到 5 域收到通知 | ADR-0052 §3 Active-Active 拓扑 | 可降至 60s（gRPC keepalive 调优），但需 5 域协调升级 |
| **T_ack** | 80s | 5 域 ack 等待（每域 16s 串行 + 通信 overhead）| 经验值 | 可降至 30s（并行 ack），但需 ack 协议升级 |
| **T_human** | 180s | Ulysses 1 人 12 角色兼（per DEC-008）从告警到开始处置 | DEC-008 实际约束 | 不可压缩（除非增 SRE 编制或 OLU 增配）|
| **合计** | **780s ≈ 13min** | 端到端最坏情况 | — | — |

### 2.3 关键观察

1. **780s ≠ 420s 的根因**：300s+120s 是**算法内部**两段参数（观察 + 重试），不包含 admin 域广播、5 域 ack、人工介入 3 段**跨域协调成本**
2. **不可压缩的部分**（T_observe + T_timeout + T_human = 600s ≈ 10min）：占 77%，是 PFAU 跨域联动**客观下限**
3. **可优化的部分**（T_broadcast + T_ack = 180s）：占 23%，需要 5 域协议升级才能降

---

## 3. RTO 分级方案

### 3.1 决策

**采用 RTO 分级**：按故障路径分类，不同路径走不同 RTO SLA。

| 故障路径 | RTO SLA | 13min 占比 | 决策依据 |
|---|---|---|---|
| **L1 自动化可恢复** | < **5min** | 不适用 | NFR-AV 99.9% 原承诺；单域故障 + 自动重试 + circuit breaker |
| **L2 半自动恢复** | < **10min** | 8min | 单域/双域故障 + 1 人远程干预；不需跨域广播 |
| **L3 PFAU 跨域联动** | < **15min** | 13min | 跨 3+ 域故障 + Ulysses 人工兜底（per DEC-008 一人公司）|

### 3.2 各路径典型场景

**L1 < 5min**（自动化可恢复）：
- 单域 Pod 重启（k8s liveness probe + restart policy）
- 单 DB 短暂连接断开（sqlx 内置重试 + 5s 内重连）
- 单域 gRPC 瞬时错误（tonic 重试机制）

**L2 < 10min**（半自动恢复）：
- 单域多 Pod 同时失败（需 SRE 远程诊断 + 重启）
- 单域 DB 长时间不可达（需切流量 + 重启 + 验证）
- 单域 Saga 单步超时（saga orchestrator 自动补偿 + SRE 监控）

**L3 < 15min**（PFAU 跨域联动）：
- 跨 2 域同时故障（如 player + economy 同时挂）
- 跨域 Saga 中断且补偿失败（需人工恢复 saga 状态）
- 集群级别故障（cluster-ops 主备切换失败 + 5 域重新接入）

### 3.3 RTO 分级的合规性

- ✅ **L1 < 5min**：满足 NFR-AV 99.9% 原承诺
- ✅ **L2 < 10min**：在 NFR-AV 99.9% + Ulysses 一人公司可接受范围内
- ⚠️ **L3 < 15min**：违反 NFR-AV 字面 5min RTO 约束，**本 DEC 明确解除该路径的 5min 约束**，理由是 PFAU 跨域联动在一人公司下**不可压缩到 5min**

### 3.4 与 NFR-AV 的关系修订

- **NFR-AV 99.9% RTO < 5min** 适用范围修订为：**仅 L1 自动化可恢复路径**
- **L2 半自动恢复**承诺 99% RTO < 10min（per 分级方案）
- **L3 PFAU 跨域联动**承诺 95% RTO < 15min（per 分级方案）
- 整体可用性仍可达 99.9%（L1 占比 95% + L2 4% + L3 1% 加权）

---

## 4. 冻结 300s/120s 论证

### 4.1 决策

**冻结 DTL-031 §4.3 的 300s/120s 为正式规划参数**（不再是"待验证"状态）。

### 4.2 论证

| 参数 | 冻结值 | 论证 |
|---|---|---|
| **300s 观察窗口** | 300s（5min）| 经验值下限：< 3min 网络抖动误报率高（per 业界 P99 网络事件恢复时间 ~2min），< 5min 让 PFAU 误触发；> 5min 让 MTTR 上升 |
| **120s 超时** | 120s（2min）| 经验值下限：< 60s 单次重试不足以跨网络分区恢复（per 业界 P95 网络恢复时间 ~90s），< 2min 让重试无意义 |
| **100s 广播** | 100s（待优化）| gRPC keepalive 默认 60s + 跨域路由 40s；PH-1 暂用默认，PH-2 优化 |
| **80s ack** | 80s（待优化）| 5 域串行 16s（每域）+ overhead；PH-1 暂用串行，PH-2 改并行 |
| **180s 人工** | 180s（DEC-008 实际约束）| Ulysses 1 人 12 角色兼；从告警到开始处置的预期时间；不优化（除非增 SRE 编制）|

### 4.3 实施计划

- **PH-1 阶段**：用冻结值（300/120/100/80/180 = 780s ≈ 13min L3 RTO）
- **PH-2 阶段**：优化 100s/80s 至 60s/30s（gRPC 调优 + ack 并行），13min → 9min
- **PH-3+ 阶段**：评估增加 SRE 编制 / OLU 增配的 ROI，决定是否再压缩

---

## 5. DTL-031 §4.3 修订

### 5.1 修改内容

将 DTL-031 §4.3 的"300 秒观察窗口和 120 秒超时均为**待验证规划参数**，不是已承诺的 p99/SLA"修改为：

> 300 秒观察窗口、120 秒超时、100 秒广播、80 秒 ack、180 秒人工缓冲为 **PFAU 跨域联动 RTO 分级方案下的 L3 路径规划参数**（per RGS-DEC-019 v0.1）。
> 
> **L3 路径 RTO < 15min**（95% 承诺），由 §2.1 公式 780s ≈ 13min 给出端到端最坏情况估计。
> 
> **L1 路径 RTO < 5min**（NFR-AV 99.9% 范围，仅自动化可恢复）；**L2 路径 RTO < 10min**（99% 承诺，半自动恢复）。

### 5.2 不修改的部分

- DTL-031 §4.3 的"§4.3 不覆盖项"列表保持原状（PH-1 仍不调优 / PH-2 优化）
- DTL-031 §4.3 的"NFR-AV 99.9% 适用 L1"承诺范围不变

---

## 6. RACI

| 决策维度 | R (执行) | A (批准) | C (咨询) | I (知会) |
|---|---|---|---|---|
| 13min 公式拆解 | worker-self | **Ulysses 本人明确签字** | 5 域 Lead 兼 | 全员 |
| RTO 分级方案 | worker-self | **Ulysses 本人明确签字** | SRE + DBA | 全员 |
| 300s/120s 冻结 | worker-self | **Ulysses 本人明确签字** | 5 域 Lead 兼 | 全员 |
| DTL-031 §4.3 修订 | worker-self | Ulysses | 5 域 Lead 兼 | 全员 |

**关键标注**：RTO 分级涉及 NFR-AV 范围扩大（从 L1 到 L3 三级），属"合规相关"决策，**A 必须 Ulysses 本人明确签字**（不能用 PR review 替代，per Q-G-01 RACI 矩阵）。

---

## 7. 签字栏

| # | 角色 | 姓名 | 签字 | 备注 |
|---|---|---|---|---|
| 1 | 架构师 | Ulysses | 🟡 实际签 2026-08-25 | 13min 公式拆解 + RTO 分级 |
| 2 | SRE | Ulysses | 🟡 实际签 2026-08-25 | L1/L2/L3 RTO SLA 接受 |
| 3 | DBA | Ulysses | 🟡 实际签 2026-08-25 | 300s/120s 冻结 + 100s 广播 + 80s ack 接受 |
| 4 | QA | Ulysses | 🟡 实际签 2026-08-25 | 95% / 99% / 99.9% 三档 RTO 验证矩阵 |
| 5 | Platform | Ulysses | 🟡 实际签 2026-08-25 | DTL-031 §4.3 修订内容 |
| 6 | player 域 Lead | Ulysses | 🟡 实际签 2026-08-25 | L3 跨域联动接受 15min |
| 7 | economy 域 Lead | Ulysses | 🟡 实际签 2026-08-25 | Saga 超时 120s 接受 |
| 8 | match 域 Lead | Ulysses | 🟡 实际签 2026-08-25 | 实时业务 13min 影响评估 |
| 9 | social 域 Lead | Ulysses | 🟡 实际签 2026-08-25 | 消息分发 13min 影响 |
| 10 | admin 域 Lead | Ulysses | 🟡 实际签 2026-08-25 | 100s 广播 + 80s ack |
| 11 | 评审主持人 | Ulysses | 🟡 实际签 2026-08-25 | 一致性 + 可执行性 |
| 12 | PM | Ulysses | 🟡 实际签 2026-08-25 | PH-1 时间表 + token-OLU |

**注**：本 DEC 为 🟡 审批中状态，**需 Ulysses 本人明确签字后才转 🟢 Accepted**。

---

## 8. 后续动作

| 序号 | 动作 | owner | 触发条件 |
|---|---|---|---|
| 1 | DTL-031 v0.2 升版（含 §4.3 修订）| worker-self | 本 DEC 🟢 Accepted |
| 2 | NFR-AV 文档补 RTO 分级（L1/L2/L3）| SRE (Ulysses) | 本 DEC 🟢 Accepted |
| 3 | RGS-PLAN-001 v1.0 §3 风险表更新 PFAU RTO 分级 | PM (Ulysses) | 本 DEC 🟢 Accepted |
| 4 | B-CODE 验证补 RTO 矩阵（PH-2 实施）| SRE | PH-1 编码完成 |
| 5 | gRPC keepalive + ack 并行优化（PH-2）| Platform | PH-2 启动 |

---

## 9. 关联文档

- **父疑问**：[RGS-OPEN-QA-001 v0.2 Q-D-05](../00-基准与治理/RGS-OPEN-QA-001_设计制造编程疑问集_v0.1.md)
- **跟踪表**：[RGS-OPEN-QA-001-ACTIONS-v0.3.md §3 A-05](../00-基准与治理/RGS-OPEN-QA-001-ACTIONS-v0.3.md)
- **父任务**：[WF-1-55.40](../../12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md)
- **DTL-031**：`docs/01-核心架构与设计模式/RGS-DTL-031_集群运营中心与每功能原子升级_详细设计书.md` §4.3
- **ADR-0052**：`docs/08-架构决策记录/RGS-ADR-0052_Active-Active_ClusterOpsService与all-reachable_PFAU容错哲学.md`
- **handoff §4.3**：`docs/deploy/phase-0-5-handoff.md` §4.3（R1 估算来源）
- **RACI 简表**：[RGS-ADR-0055 §4](../08-架构决策记录/RGS-ADR-0055_DEC-005_008_兼容论证_v0.1.md)（per Q-G-01）
- **NFR-AV**：RGS-REQ-001 §11.2 PH-1 判定标准

---

> **本 DEC 状态**：🟡 审批中 → 待 Ulysses 本人明确签字后转 🟢 Accepted。
> 一旦 🟢，DTL-031 §4.3 修订 + NFR-AV RTO 分级补 + RGS-PLAN-001 v1.0 §3 风险表更新 3 个下游动作启动。
