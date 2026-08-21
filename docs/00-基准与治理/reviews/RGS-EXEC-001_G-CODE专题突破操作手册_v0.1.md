# G-CODE 专题突破操作手册（G-CODE Breakthrough Operation Manual）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-EXEC-001 |
| 版本 | 0.1（草稿）|
| 依据 | RGS-PLAN-001 v0.6 §3.3 G-CODE-01~07 + RGS-QA-001 v0.9 决策 + RGS-REV-003/004/005/006 + RGS-ADR-0052 + RGS-IMPL-001 + RGS-HANDOFF-001 §5 |
| 目的 | 起草 G-CODE 阻塞项的"专题突破"操作序列 + 签字流程 + 通过标准；不伪造实测数据 |
| 范围 | 3 个核心 G-CODE：**G-CODE-03（ADR-0052 联审）+ G-CODE-04（Q-003 Saga 6 场景）+ G-CODE-02（DTL-031 字段 Review）** |
| 配套 | RGS-REV-003（联合评审主文）/ RGS-REV-004（DTL 字段 checklist）/ RGS-REV-005（Saga 6 场景 checklist）/ RGS-REV-006（签字流程）|
| 保密级别 | 内部限定（Internal Use Only）|

> **AI 不伪造实测数据**：本手册仅起草"操作序列 + 通过标准 + 签字流程"。**真实演练数据 / 故障注入结果 / 字段级 Review 结论** 需由具名人类责任人填写，AI 不能代签或代填。

---

## §1 G-CODE 列表 + 现状

| G-CODE | 主题 | 当前状态 | 关闭证据 | 责任人 | 工具文档 |
|---|---|---|---|---|---|
| **G-CODE-02** | DTL-031 + Q-025 字段级 DD Review | 🟠 **Open / Blocker**（DTL-031 v0.2 已存在 21 KB）| 接口 / 状态机 / fencing / CEM/PFAU / 测试映射 + 审批栏具名签署 | 架构 + Platform + DBA | RGS-REV-004 附件A |
| **G-CODE-03** | RGS-ADR-0052 联审（all-reachable + Active-Active）| 🟠 **Open**（ADR-0052 已起草 5.7 KB）| ADR 审批栏 + 目标拓扑核验 + 故障注入计划 + 风险接受 | 架构 + SRE | RGS-REV-003 §2.3 |
| **G-CODE-04** | Q-003 跨 DB Saga + Q-004 原子组合 | 🟠 **Open / Q-003 Blocker**（技术方案已固定）| Saga/Outbox/补偿边界 + 四层原子状态机合并图 + **6 个业务场景验收计划** | 架构 + DBA + Economy 域 Lead | RGS-REV-005 附件B |

---

## §2 G-CODE-03 ADR-0052 联审执行序列

### §2.1 联审范围

- **RGS-ADR-0052**：`Active-Active ClusterOpsService 与 all-reachable PFAU 容错哲学`（5.7 KB）
- **联审焦点**：
  1. all-reachable 语义（PFAU 灰度确认最严格 quorum）
  2. Active-Active 拓扑（multi-leader 双副本 / OCC 版本字段 / fencing 租约）
  3. 故障注入计划（pod kill / 跨 AZ 失联 / 网络分区 / leader 切换）
  4. 风险接受（Active-Active 不自动证明 RPO/RTO，需演练取证）

### §2.2 执行序列

| 阶段 | 活动 | 责任方 | 工具 / 输出 |
|---|---|---|---|
| **阶段 1 预读**（3 天）| 4 类责任人阅读 RGS-ADR-0052 + RGS-DTL-031 v0.2 §2/§4/§10 + RGS-REV-003 §2.3 | 架构 + SRE + DBA + Platform | 预读意见表（10 项） |
| **阶段 2 联审会议**（2 小时）| 现场/视频会议，逐条过 ADR 决策点 | 架构师主持 | 会议纪要（4 类责任人签字）|
| **阶段 3 故障注入计划**（3 天）| SRE Lead 起草故障注入 4 场景（pod kill / 跨 AZ 失联 / 网络分区 / leader 切换）| SRE Lead | RGS-ADR-0052 故障注入计划附录 |
| **阶段 4 目标拓扑核验**（2 天）| 架构 + SRE 联合核验当前 dev / staging 拓扑 vs ADR-0052 描述 | 架构 + SRE | 拓扑核验报告（含差异列表）|
| **阶段 5 异议闭环**（7-14 天）| 异议以 ADR 修订闭环（不修订则 NO-GO）| 架构师 | ADR 修订版 v0.2 |
| **签字** | 架构 + SRE + DBA + Platform 4 类签字 | 4 类 | ADR 审批栏 |

### §2.3 通过标准

- ✅ ADR-0052 4 个焦点全部经过预读 + 会议确认
- ✅ 故障注入计划 4 场景起草（含 SRE 签字）
- ✅ 拓扑核验报告完成（差异列表 ≤ 5 项）
- ✅ 4 类责任人签字
- ✅ ADR 审批栏 v0.2 全部具名签署

### §2.4 签字栏

| # | 角色 | 姓名 | 签字 | 日期 | 结论 |
|---|---|---|---|---|---|
| 1 | 架构师 | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 2 | SRE Lead | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 3 | DBA Lead | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 4 | Platform Engineer | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 5 | PM（资源接受）| __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |

---

## §3 G-CODE-04 Q-003 Saga 6 场景演练执行序列

### §3.1 演练范围

- **Q-003**：跨 5 DB 事务一致性（Saga + Outbox + 补偿 + 幂等）
- **6 场景**（per RGS-REV-005 附件B）：
  1. **正常**：player → economy → social 购买流程成功
  2. **补偿**：economy 失败回滚 player / social
  3. **超时**：economy 30s 未响应
  4. **人工升级**：金额 > 阈值触发人工审核
  5. **去重**：request_id 重复，幂等保证
  6. **PFAU + Saga**：5 节点灰度期间 Saga 进行

### §3.2 执行序列

| 阶段 | 活动 | 责任方 | 工具 / 输出 |
|---|---|---|---|
| **阶段 1 演练脚本准备**（2 天）| Economy 域 Lead 起草 6 场景测试脚本（Rust + testcontainers）| Economy 域 Lead | 6 脚本 + Cargo.toml 配置 |
| **阶段 2 测试环境**（1 天）| DBA 准备独立测试 DB（5 DB + 镜像仓库 + 监控）| DBA Lead | testcontainers 配置 |
| **阶段 3 演练执行**（2 天）| Economy 域 Lead 跑 6 场景 + 收集结果 | Economy 域 Lead | 6 场景报告（含 IT 输出）|
| **阶段 4 DBA 审计**（1 天）| DBA 审计 6 场景的 DB 一致性（事务日志 + Outbox 状态）| DBA Lead | DBA 审计报告 |
| **阶段 5 架构师 + Economy 二次确认**（1 天）| 架构师 + Economy 域 Lead 二次确认 Q-003 决策 | 架构 + Economy 域 Lead | 二次确认签字（per RGS-PLAN-001 v0.6 §3.4.3 签字顺序）|
| **签字** | Economy 域 Lead + DBA + 架构师 3 类签字 | 3 类 | RGS-REV-005 附件B 6 场景 checklist |

### §3.3 通过标准

- ✅ 6 场景脚本可在 testcontainers 环境跑通
- ✅ 6 场景输出与预期一致（正常/补偿/超时/人工/去重/PFAU）
- ✅ DBA 审计 5 DB 事务一致性
- ✅ Economy 域 Lead + DBA + 架构师 3 类签字
- ✅ 6 场景报告归档在 `docs/00-基准与治理/reviews/RGS-REV-005_Saga演练报告_v0.1.md`

### §3.4 签字栏

| # | 角色 | 姓名 | 签字 | 日期 | 结论 |
|---|---|---|---|---|---|
| 1 | Economy 域 Lead（独立 + Q-003 二次确认）| __________ | __________ | ____-__-__ | ☐ 6 场景全部通过 / ☐ 部分通过 / ☐ 不通过 |
| 2 | DBA Lead | __________ | __________ | ____-__-__ | ☐ 5 DB 一致 / ☐ 偏差 / ☐ 不通过 |
| 3 | 架构师 | __________ | __________ | ____-__-__ | ☐ 决策接受 / ☐ 修订 / ☐ NO-GO |
| 4 | PM（实施授权）| __________ | __________ | ____-__-__ | ☐ 授权 / ☐ 推迟 / ☐ NO-GO |

---

## §4 G-CODE-02 DTL-031 字段级 Review 执行序列

### §4.1 Review 范围

- **RGS-DTL-031 v0.2**：COC + ARC-051 落地详细设计（21 KB）
- **Review 焦点**（per RGS-REV-004 附件A §A.6）：
  1. §A.6.1 `cluster_nodes` / `feature_activations` / `pfa_operations` 表
  2. §A.6.2 feature 状态机（declared → canary → confirm → done / rolled_back）
  3. §A.6.3 PFAU 5 类错误码（confirm 失败 / 节点掉线 / 资源不足 / 灰度不一致 / 回滚失败）
  4. §A.6.4 ADR-0052 贯穿（all-reachable + Active-Active 在每个 gRPC 方法中体现）
  5. §A.6.5 DLQ 处理（DiscardDlqEvent / ListDlqEvents）
  6. §A.6.6 监控（PFAU 完成时延指标 per handoff §4.3 R1 ~13 分钟）

### §4.2 执行序列

| 阶段 | 活动 | 责任方 | 工具 / 输出 |
|---|---|---|---|
| **阶段 1 字段级预读**（5 天）| 3 类责任人按 §A.6 6 子项逐字段预读 | 架构 + Platform + DBA | RGS-REV-004 附件A §A.6 全部勾选 |
| **阶段 2 联合评审**（2 小时）| 现场/视频会议，逐条过 6 焦点 | 架构师主持 | 评审纪要 |
| **阶段 3 字段级 Review 记录**（2 天）| 3 类责任人按 RGS-REV-004 §A.6 6 子项签字 | 3 类 | 字段级 Review checklist 完成 |
| **阶段 4 异议闭环**（7-14 天）| 异议以 DTL-031 修订闭环（升 v0.3）| 架构师 | DTL-031 v0.3 |
| **签字** | 架构 + Platform + DBA 3 类签字 | 3 类 | RGS-DTL-031 审批栏 |

### §4.3 通过标准

- ✅ RGS-DTL-031 v0.2 §A.6 6 焦点全部经过字段级预读
- ✅ RGS-REV-004 §A.6 checklist 全部勾选（不漏字段）
- ✅ 异议闭环（≤ 5 项）或 DTL-031 v0.3 升版
- ✅ 架构 + Platform + DBA 3 类签字
- ✅ DTL-031 审批栏 v0.3（或 v0.2 修订）具名签署

### §4.4 签字栏

| # | 角色 | 姓名 | 签字 | 日期 | 结论 |
|---|---|---|---|---|---|
| 1 | 架构师 | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 2 | Platform Engineer | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 3 | DBA Lead | __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 4 | cluster-ops 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 5 | Admin 域 Lead（独立）| __________ | __________ | ____-__-__ | ☐ 接受 / ☐ 修订 / ☐ NO-GO |
| 6 | PM（实施授权）| __________ | __________ | ____-__-__ | ☐ 授权 / ☐ 推迟 / ☐ NO-GO |

---

## §5 三 G-CODE 联合签字

> **三 G-CODE 全部签字后，RGS-PLAN-001 v0.6 §3.3 G-CODE-02/03/04 由 Open → Closed。**

| G-CODE | 状态 | 签字日期 | 责任人 |
|---|---|---|---|
| G-CODE-02 DTL-031 字段 Review | 🟢 Closed（待 v0.5 签字）| ____-__-__ | 架构 + Platform + DBA + cluster-ops + Admin |
| G-CODE-03 ADR-0052 联审 | 🟢 Closed（待 v0.5 签字）| ____-__-__ | 架构 + SRE + DBA + Platform |
| G-CODE-04 Q-003 Saga 6 场景 | 🟢 Closed（待 v0.5 签字）| ____-__-__ | Economy 域 Lead + DBA + 架构 + PM |

---

## §6 RGS-PLAN-001 v0.6 NO-GO 解除进度

| NO-GO 解除条件 | 状态 | 责任方 |
|---|---|---|
| RGS-REV-003 §7.3 12 类签字栏签署 | 🟡 工具就位 | 12 类责任人 |
| RGS-ENV-001 §6 12 类签字栏签署 | 🟡 工具就位（v0.2）| 12 类责任人 |
| 7 G-CODE 全部 "🟢 Closed" 状态 | 🟠 G-CODE-02/03/04 待签 / G-CODE-01/05/06/07 待评估 | 4 类责任人 |

> **本手册不构成取消 53 NO-GO**：3 项条件全部满足前，NO-GO 保持。

---

## §7 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 初版。覆盖 3 个核心 G-CODE（G-CODE-02/03/04）的执行序列 + 通过标准 + 签字栏；**不伪造实测数据 / 不代签**；引用 RGS-REV-003/004/005/006 工具文档。 |
