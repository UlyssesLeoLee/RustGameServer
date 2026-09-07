# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 02 运维安全与网络 — 服务器全生命周期管理（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-02-ADD3 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-037 v0.1 + RGS-DTL-042 v0.1 |
| 升版基线 | v0.1 (2026-08-21) → v0.2 (2026-09-07, 8 维度增量) |
| V模型层级 | TL-6 负载 / TL-7 故障注入 |
| 制定日 | 2026-08-21 |
| 升版日 | 2026-09-07 |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / Admin 域 Lead兼 per DEC-008) | 2026-08-21 | 一人公司 12 角色兼任 |
| 升版（v0.2 自审） | 架构师（Mavis 接手代签 per DEC-008） | 2026-09-07 | 8 维度增量；B3 二审待 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-21 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-21 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-21** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师 | 初版制定 |
| **0.2** | 2026-09-07 | 架构师（Mavis 接手代签 per DEC-008） | 8 维度增量升版：① 9/6 8 域扩展 server lifecycle（per 闪烁之光兼容）② 9 域 mTLS cert 轮换（per L-CAND-006）③ 派生约束守护 L15-L23（per 9/6 6c6839e cutover）④ batch 域 6 module 跨域（per 9/1 batch v0.1） |

---

## 0. v0.1 → v0.2 升版范围（8 维度增量，主题特定子集）

| # | 维度 | v0.1 现状 (8/21) | v0.2 增量 (9/7) | 引用 |
|---|---|---|---|---|
| 1 | **9/6 8 域扩展 server lifecycle** | §2 L001~L015 仅覆盖 5 域 + admin 域 lifecycle | §2 增 L016~L020：5 NEW 域（scene/battle/network/account/sub8）+ batch 域 server lifecycle 端到端 | a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673 |
| 2 | **9 域 mTLS cert 轮换 (per L-CAND-006)** | §2 缺 mTLS cert 轮换 | §2 增 L021~L023：9 域 mTLS cert 轮换 + server lifecycle 0 中断 | L-CAND-006 / 9/6 d270ab9 / 9/6 d15a0bb |
| 3 | **派生约束守护 L15-L23** | §5 通过判定未含派生约束 | §5 增 §派生约束守护段，列出 L15-L23 落地状态 | 6c6839e / add4238 |
| 4 | **batch 域 6 module 跨域 (per 9/1 batch v0.1)** | §2 缺 batch 域 lifecycle | §2 增 L024~L026：batch 域 6 module 跨域 server lifecycle | fd122f6/e70ed71/e366ff8/62027c9/eb1e15d |

**已知缺口（per 缺标比错标安全, per 8/26 JST）**:
- 5 NEW 域（scene/battle/network/account/sub8）+ batch 域 server lifecycle 真实跨域集成需 ST Phase C 验证
- 9 域 mTLS cert 轮换 + server lifecycle 0 中断实测待 k3s 集群可达
- batch 域 6 module 跨域 server lifecycle 待 NATS 部署就绪

---

## 1. 目的

端到端验证 RGS-REQ-037 AC-LCM-001~010 + NFR-LCM-001~008 在真实集群（演练环境 + 生产环境）下的端到端表现，覆盖 10 个 AC。v0.2 新增 8 域 server lifecycle + 9 域 mTLS cert 轮换 + batch 域 6 module 跨域。

## 2. 测试用例

| 用例 ID | 试验级别 | 对应 AC | 测试目的 |
|---|---|---|---|
| TST-ST-02-L001 | [E2E] | AC-LCM-001 | 完整开新服流程（资源评估 → 挂载 → 灰度开放 → 全量）每步可执行可审计可演练 |
| TST-ST-02-L002 | [E2E] | AC-LCM-002 | 合服前后两服玩家资产总量 100% 一致（FR-LCM-001 端到端验证）|
| TST-ST-02-L003 | [E2E] | AC-LCM-003 | 分服后两服独立可玩，玩家分流与 `SplitPlan` 策略一致 |
| TST-ST-02-L004 | [E2E] | AC-LCM-004 | 退场后客服系统能按 RBAC 查询玩家历史数据，玩家侧**不**可登录 |
| TST-ST-02-L005 | [E2E] | AC-LCM-005 | 归档后 GDPR "被遗忘权"删除通路命中，从冷归档定位并删除 |
| TST-ST-02-L006 | [E2E] | AC-LCM-006 | 任意阶段变更 `admin_db.operation_audit` 留痕完整 |
| TST-ST-02-L007 | [E2E] | AC-LCM-007 | 阶段变更 OLU 预算纳入 ARC-026 核算，**未**核算视为未批准 |
| TST-ST-02-L008 | [E2E] | AC-LCM-008 | 退场后 30 天内二次激活成功（数据完整、运行时节点可恢复）|
| TST-ST-02-L009 | [E2E] | AC-LCM-009 | 合服回退窗口期（7~30 天）内可按 Saga 反向步骤回退 |
| TST-ST-02-L010 | [E2E] | AC-LCM-010 | 跨服合并回溯：两个已归档退场服曾被合服，合服前资产归属记录可还原 |
| TST-ST-02-L011 | [TL-6] | NFR-LCM-001 | 阶段变更资产不丢不重 100% 一致 |
| TST-ST-02-L012 | [E2E] | NFR-LCM-004 | 玩家通知 ≥ 7 天预告（开新服/合服/退场前）|
| TST-ST-02-L013 | [E2E] | NFR-LCM-006 | 归档后客服查询 p99 < 5 秒（含冷归档按需还原）|
| TST-ST-02-L014 | [TL-7] | RSK-LCM-001 | 阶段变更中途崩溃：Saga 补偿回退至变更前状态 |
| TST-ST-02-L015 | [E2E] | RSK-LCM-005 | 归档 N+2 冗余存储：单副本失效查询仍可用 |
| **TST-ST-02-L016** | **[E2E]** | **8 域扩展 server lifecycle (per 9/6 闪烁之光兼容)** | **scene 域 server lifecycle 端到端 (开新服/合服/分服/退场/归档 5 阶段)** | scene 域 server lifecycle |
| **TST-ST-02-L017** | **[E2E]** | **8 域扩展 server lifecycle (per 9/6)** | **battle 域 server lifecycle 端到端** | battle 域 server lifecycle |
| **TST-ST-02-L018** | **[E2E]** | **8 域扩展 server lifecycle (per 9/6)** | **network + account + sub8 域 server lifecycle 端到端** | 3 域 server lifecycle |
| **TST-ST-02-L019** | **[E2E]** | **8 域扩展 server lifecycle (per 9/6)** | **5 NEW 域 server lifecycle 跨域 saga** | 5 NEW 域跨域 saga |
| **TST-ST-02-L020** | **[E2E]** | **8 域扩展 server lifecycle (per 9/6)** | **8 域 server lifecycle 0 中断 (per L19 mTLS=saga 触达)** | 8 域 0 中断 |
| **TST-ST-02-L021** | **[E2E]** | **9 域 mTLS cert 轮换 (per L-CAND-006)** | **9 域 mTLS cert 轮换 + server lifecycle 0 中断** | 9 域 cert 轮换 |
| **TST-ST-02-L022** | **[E2E]** | **9 域 mTLS cert 轮换 (per L-CAND-006) + 3 NEW 域 k8s yaml** | **3 NEW 域 k8s yaml + mTLS cert 落档 (per 9/6 d15a0bb)** | 3 NEW 域 k8s yaml |
| **TST-ST-02-L023** | **[E2E]** | **9 域 mTLS cert 轮换 (per L-CAND-006) + ca.crt 0 字节 (per L20)** | **9 域 ca.crt 0 字节验证** | 9 域 ca.crt 0 字节 |
| **TST-ST-02-L024** | **[E2E]** | **batch 域 6 module 跨域 server lifecycle (per 9/1 batch v0.1)** | **batch 域 CRON 跨域 server lifecycle** | batch cron |
| **TST-ST-02-L025** | **[E2E]** | **batch 域 6 module 跨域 server lifecycle (per 9/1 batch v0.1)** | **batch 域 TASK-TPL + WORKER 跨域 server lifecycle** | batch tpl/worker |
| **TST-ST-02-L026** | **[E2E]** | **batch 域 6 module 跨域 server lifecycle (per 9/1 batch v0.1)** | **batch 域 AUDIT/DLQ/CONN 跨域 server lifecycle (per F-10/F-9/NFR-32)** | batch audit/dlq/conn |

## 3. 最小可复现实验

### 3.1 固定基线与取证规则

| 项目 | 固定条件 |
|---|---|
| 拓扑/规格 | 演练环境：K3s 多节点 + 1 个 ClusterOpsService 双副本 + 5 域 Atomic App 最小部署；生产环境：PH-1 既有 10 万 CCU 集群；每类阶段变更的演练剧本模板各 1 份。 |
| 数据集与负载模型 | 演练数据快照生成器：从生产环境拉取（脱敏）至少 1 万玩家 + 100 万道具 + 50 万交易记录的样本；正式执行数据来自生产环境。 |
| 预热与持续时间 | 演练：每类阶段变更演练 1 次 + 联合演练（含开新服+合服+退场+归档串行）1 次；正式：按运营节奏执行（开新服前 N 天演练，正式执行 < 8 小时）。 |
| 故障注入 | ① 阶段变更中途节点故障；② Saga 步骤注入失败；③ 灰度回滚触发阶段变更被拒；④ admin_db 写失败；⑤ 业务 DB 跨 DB 写入失败；⑥ 归档 N+2 副本单副本失效。 |
| 采样/SLO计算 | 每阶段变更记录：阶段类型 / 演练 vs 正式 / 操作者 / 演练报告 / 资产总量前后 / 玩家通知触达 / Saga 步骤执行轨迹 / 失败回退 / OLU 消耗 / 留痕完整度。 |
| 原始证据路径 | `artifacts/test-results/TST-ST-02-ADD3/<run-id>/<case-id>/{topology.yaml,drill_report.json,asset_snapshot_before.json,asset_snapshot_after.json,audit_chain.jsonl,olu_consumption.json,notification_log.json,summary.json}`；`summary.json` 必须含 AC-LCM-nnn 对应的通过证据。 |
| 清理步骤 | 演练环境清理演练数据；正式执行保留全部审计证据；删除测试通知；保留 `summary.json` 与 `audit_chain.jsonl`。 |

### 3.2 用例执行矩阵与可判定预期

| 用例 | 拓扑/数据 | 故障注入 | 可判定预期 |
|---|---|---|---|
| C001 (AC-LCM-001) | 演练环境 + 开新服 SOP | 演练 1 次 + 正式 1 次 | 演练通过后方可切到 `executing`；正式执行 6 阶段全流程无异常；operation_audit 完整。 |
| C002 (AC-LCM-002) | 演练环境 + 2 服各 1 万玩家 | 演练 1 次 | 合服前后 `asset_snapshot` 对比：玩家/道具/交易 100% 一致。 |
| C003 (AC-LCM-003) | 1 服拆为 2 服 | 演练 1 次 | 两服各自可登录；分流与 `SplitPlan.forced_rule` 哈希结果一致；跨服好友/工会按 `cross_realm_relation` 规则保留或拆分。 |
| C004 (AC-LCM-004) | 1 服退场 | RBAC 角色测试 | 客服系统按 `cs_agent` 角色可查询退场服历史数据；玩家侧登录被拒。 |
| C005 (AC-LCM-005) | 1 服退场 + 归档 | GDPR 删除请求 | 冷归档中定位并删除玩家数据；保留审计链；其他玩家数据不受影响。 |
| C006 (AC-LCM-006) | 任意阶段变更 | 抽查审计 | `admin_db.operation_audit` 100% 覆盖，无"无法解释的状态变更"。 |
| C007 (AC-LCM-007) | OLU 预算超限测试 | 故意超限 | 阶段变更被 `rgs-arc-olu` 拒绝并告警。 |
| C008 (AC-LCM-008) | 退场后 15 天二次激活 | 反向退场 | 数据完整、节点可恢复、路由登记重新生效。 |
| C009 (AC-LCM-009) | 合服后 7 天回退 | Saga 反向步骤 | 资产 100% 还原至合服前状态。 |
| C010 (AC-LCM-010) | 2 服合服 + 归档后查询 | 模拟客服查询 | 合服前玩家归属服记录可被还原。 |
| C011 (NFR-LCM-001) | 6 阶段 + 资产对比 | 100 万玩家级数据 | 资产总量前后差异 = 0。 |
| C012 (NFR-LCM-004) | 开新服/合服/退场前 | 玩家通知 | 公告 + 邮件 + 横幅 全部 ≥ 7 天预告；触达率 ≥ 99%。 |
| C013 (NFR-LCM-006) | 归档后客服查询 | 1000 次查询 | p99 < 5s（含冷归档按需还原）。 |
| C014 (RSK-LCM-001) | 分服中途节点故障 | 步骤 3 注入失败 | Saga 补偿回退至分服前状态；operation_audit 留痕。 |
| C015 (RSK-LCM-005) | 归档 N+2 副本 | 杀 1 个副本 | 客服查询仍可用，无数据丢失。 |
| C016 (L016) | scene 域 server lifecycle | 5 阶段演练 | scene 域开新服+合服+分服+退场+归档端到端走通 |
| C017 (L017) | battle 域 server lifecycle | 5 阶段演练 | battle 域开新服+合服+分服+退场+归档端到端走通 |
| C018 (L018) | network + account + sub8 域 server lifecycle | 5 阶段演练 | 3 域 server lifecycle 端到端走通 |
| C019 (L019) | 5 NEW 域 server lifecycle 跨域 saga | 5 阶段演练 | 5 NEW 域 server lifecycle 跨域 saga 端到端走通 |
| C020 (L020) | 8 域 server lifecycle 0 中断 | 5 阶段演练 | 8 域 server lifecycle 0 中断 (per L19 mTLS=saga 触达) |
| C021 (L021) | 9 域 mTLS cert 轮换 | cert 轮换 | 9 域 cert 轮换 + server lifecycle 0 中断 |
| C022 (L022) | 3 NEW 域 k8s yaml + mTLS cert 落档 | 3 NEW 域 k8s yaml | 3 NEW 域 k8s yaml + mTLS cert 落档 |
| C023 (L023) | 9 域 ca.crt 0 字节 | ca.crt 0 字节 | 9 域 ca.crt 0 字节验证 |
| C024 (L024) | batch 域 CRON 跨域 server lifecycle | batch cron | batch cron 跨域 server lifecycle |
| C025 (L025) | batch 域 TASK-TPL + WORKER 跨域 server lifecycle | batch tpl/worker | batch tpl/worker 跨域 server lifecycle |
| C026 (L026) | batch 域 AUDIT/DLQ/CONN 跨域 server lifecycle | batch audit/dlq/conn | batch audit/dlq/conn 跨域 server lifecycle |

## 4. 追溯性

| AC/NFR/RSK | 用例 |
|---|---|
| AC-LCM-001 | TST-ST-02-L001 |
| AC-LCM-002 | TST-ST-02-L002 |
| AC-LCM-003 | TST-ST-02-L003 |
| AC-LCM-004 | TST-ST-02-L004 |
| AC-LCM-005 | TST-ST-02-L005 |
| AC-LCM-006 | TST-ST-02-L006 |
| AC-LCM-007 | TST-ST-02-L007 |
| AC-LCM-008 | TST-ST-02-L008 |
| AC-LCM-009 | TST-ST-02-L009 |
| AC-LCM-010 | TST-ST-02-L010 |
| NFR-LCM-001 | TST-ST-02-L011 |
| NFR-LCM-004 | TST-ST-02-L012 |
| NFR-LCM-006 | TST-ST-02-L013 |
| RSK-LCM-001 | TST-ST-02-L014 |
| RSK-LCM-005 | TST-ST-02-L015 |
| 9/6 8 域扩展 server lifecycle | TST-ST-02-L016~L020 |
| L-CAND-006 9 域 mTLS cert 轮换 | TST-ST-02-L021~L023 |
| 9/1 batch v0.1 6 module 跨域 server lifecycle | TST-ST-02-L024~L026 |

## 5. 通过判定

- AC-LCM-001~010 全部 10 项通过
- NFR-LCM-001/004/006 全部 3 项达标
- RSK-LCM-001/005 缓解 100% 生效
- 0 高优事故
- 资产不丢不重 100% 验证
- 演练通过后方可正式执行（FR-LCM-003 硬约束）
- 5 NEW 域（scene/battle/network/account/sub8）+ batch 域 server lifecycle 端到端走通
- 9 域 mTLS cert 轮换 + server lifecycle 0 中断（per L-CAND-006）
- batch 域 6 module 跨域 server lifecycle（per 9/1 batch v0.1）

## 6. 派生约束守护

| 派生约束 | 状态 | 引用 |
|---|---|---|
| L1 (cargo check 60s) | ✅ N/A (doc) | AGENTS.md §2.1 |
| L11 (cargo build dir lock) | ✅ N/A (doc) | AGENTS.md §2.2 |
| L12 (临时 log 不入 commit) | ✅ (git status 0 untracked) | AGENTS.md §2.2 |
| L13 (8 维度 git show --stat 实证) | ✅ (本节 §0 引用全部 commit 实证) | AGENTS.md §2.2 |
| L15 (native binary 跨工具链) | ⏳ (待 ST Phase C) | 6c6839e |
| L19 (mTLS 业务级 = saga 触达) | ✅ (TST-ST-02-L020/L021 落档) | 6c6839e |
| L20 (ca.crt 0 字节) | ✅ (TST-ST-02-L023 落档) | 6c6839e / 9/6 d15a0bb |
| L21 (跨工具链 gRPC Code 解析) | ✅ (server lifecycle 跨工具链) | 6c6839e |
| L22 (协议码映射表) | ✅ (per L-CAND-007) | 6c6839e |
| L23 (4 层自动探针) | ✅ (server lifecycle 4 层探针) | 6c6839e |
| B3 (DDD Review 二审) | ⏳ (Mavis 自审 1 次停手, Ulysses 二审待) | AGENTS.md §3.x |

---

> 与 RGS-TST-ST-02 + RGS-TST-ST-02-ADD1/ADD2 共存（v0.2 升版）。
