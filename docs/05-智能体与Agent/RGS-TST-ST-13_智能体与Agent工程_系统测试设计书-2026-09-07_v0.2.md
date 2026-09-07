# 系统测试设计书（システムテスト仕様書 / System Test Specification）

**智能体与 Agent 工程 — Agent Engineering System Test**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-13 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-033、RGS-REQ-034、RGS-REQ-035 |
| 升版基线 | v0.1 (2026-08-20) → v0.2 (2026-09-07, 8 维度增量) |
| 制定日 | 2026-08-20 |
| 升版日 | 2026-09-07 |
| 最终更新日 | 2026-09-07 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 状态 | ⏳ Mavis 自审 (per B3 派生约束 DDD Review 二审流程) |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-20 | 架构师 | 初版制定。 |
| 0.2 | 2026-08-20 | 架构师 | 增加 033–035 的需求与验收映射；测试设计不等同于已执行结果。 |
| **0.2.1 (升版)** | **2026-09-07** | **架构师（Mavis 接手代签 per DEC-008）** | **8 维度增量升版**：① 9/5 plugin 集群架构（per 61cf306 / f785f18 / 5cfd692）② AI Function Pool (per RGS-INC-001 v0.2 §15) ③ 8 域扩展（per 9/6 闪烁之光兼容）④ 9 域 mTLS 业务级（per 9/6 d270ab9）⑤ 派生约束守护 L15-L23（per 9/6 6c6839e cutover） |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-20 | — |
| 升版（v0.2.1 自审） | 架构师（Mavis 接手代签 per DEC-008） | 2026-09-07 | 8 维度增量；B3 二审待 |
| 评审（架构/QA） |  |  | 待 Ulysses 二审 |

---

## 0. v0.1 → v0.2.1 升版范围（8 维度增量，主题特定子集）

| # | 维度 | v0.1 现状 (8/20) | v0.2.1 增量 (9/7) | 引用 |
|---|---|---|---|---|
| 1 | **9/5 plugin 集群架构** | §1 ST-AGT-001~003 仅 3 端到端用例 | §1 增 ST-AGT-100~103：plugin 集群架构 4 阶段用例 (阶段 0 mock 已完 / 阶段 1 MVP ~2-3 周 / 阶段 2 独立更新 ~3-4 周 / 阶段 3 平台化 ~4-6 周) | 61cf306 / f785f18 / 5cfd692 |
| 2 | **AI Function Pool (per RGS-INC-001 v0.2 §15)** | §1 缺 AI Function Pool | §1 增 ST-AGT-200~205：AI Function Pool 6 用例（CRUD + 决策 + LLM 协作） | RGS-INC-001 v0.2 §15 |
| 3 | **8 域扩展** | §1 仅 5 域 agent 联动 | §1 增 ST-AGT-300~303：5 NEW 域（scene/battle/network/account/sub8）+ batch 域 agent 联动 | a5235eb/95e67a6/1134cfd/57edbeb/b6b19b7/1dd9afc/3c79bca/42df673 |
| 4 | **9 域 mTLS 业务级** | §1 缺 mTLS 业务级 | §1 增 ST-AGT-400~403：9 域 mTLS 业务级 E2E（per 9/6 d270ab9 11 步 v3） | d270ab9 / d15a0bb |
| 5 | **派生约束守护 L15-L23** | §1 通过判定未含派生约束 | §派生约束守护 段增 L15-L23 全部 | 6c6839e / add4238 |

**已知缺口（per 缺标比错标安全, per 8/26 JST）**:
- AI Function Pool 6 用例 (per RGS-INC-001 v0.2 §15) 需 LLM 协作接口联调验证
- 9/5 plugin 集群架构 4 阶段用例 WBS v0.1 5-10 task 落地待 W25 验证
- 5 NEW 域（scene/battle/network/account/sub8）+ batch 域 agent 联动 真实跨域集成需 ST Phase C 验证
- 9 域 mTLS 业务级 agent 链路端到端待 NATS 部署就绪

---

## 1. 端到端全链路验收用例

| 用例编号 | 对应需求/验收 | 场景 | 门禁通过标准 |
|---|---|---|---|
| **ST-AGT-001** | FR-AGO-002、NFR-AGO-001/003、AC-AGO-002 | 生产级灰度升级值守 | 灰度中故意切断 1 节点网络，系统平稳转入 Quarantine；任何无效意图均不得绕过闸门。 |
| **ST-AGT-002** | FR-AGO-003/004、NFR-AGO-002/003、AC-AGO-003/004 | 7x24h 高并发工单压力演练 | 注入 10,000 张并发工单，记录候选初解率与回复耗时；依赖故障时转人工且不自动补偿。具体 SLO 仍待具名人类批准。 |
| **ST-AGT-003** | FR-AGS-001/004、NFR-AGS-001/002/003、AC-AGS-001/002/003 | 宏观经济 30 天虚拟沙盘推演 | 演练 100,000 玩家交易数据，输出带快照、版本和随机种子的候选调控参数；结果未经审批不得改变线上经济。 |
| **ST-AGT-100** | **FR-PLG-001/002 (per 61cf306 plugin 集群架构)** | **plugin 集群架构 阶段 0 mock** | **阶段 0 mock 已完：1 PoC WASM 跑通** |
| **ST-AGT-101** | **FR-PLG-010/011 (per 61cf306)** | **plugin 集群架构 阶段 1 MVP** | **阶段 1 MVP ~2-3 周：核心 plugin 框架可装载 + 调度** |
| **ST-AGT-102** | **FR-PLG-020/021 (per 61cf306)** | **plugin 集群架构 阶段 2 独立更新** | **阶段 2 独立更新 ~3-4 周：plugin 独立版本化 + 灰度** |
| **ST-AGT-103** | **FR-PLG-030/031 (per 61cf306)** | **plugin 集群架构 阶段 3 平台化** | **阶段 3 平台化 ~4-6 周：WBS v0.1 5-10 task 落地 + 平台化** |
| **ST-AGT-200** | **AI Function Pool (per RGS-INC-001 v0.2 §15)** | **AI Function Pool CRUD** | **Function Pool 6 用例：CRUD + 决策 + LLM 协作** |
| **ST-AGT-201** | **AI Function Pool (per RGS-INC-001 v0.2 §15)** | **AI Function Pool 决策** | **决策树 + 闸门联动** |
| **ST-AGT-202** | **AI Function Pool (per RGS-INC-001 v0.2 §15)** | **AI Function Pool LLM 协作** | **LLM 协作接口联调** |
| **ST-AGT-300** | **9/6 8 域扩展** | **scene 域 agent 联动** | **scene 域 agent 端到端联动** |
| **ST-AGT-301** | **9/6 8 域扩展** | **battle 域 agent 联动** | **battle 域 agent 端到端联动** |
| **ST-AGT-302** | **9/6 8 域扩展** | **network + account + sub8 域 agent 联动** | **network + account + sub8 域 agent 端到端联动** |
| **ST-AGT-303** | **9/1 batch 域 agent 联动** | **batch 域 agent 联动** | **batch 域 agent 端到端联动 + coc_policy** |
| **ST-AGT-400** | **9/6 d270ab9 9 域 mTLS 业务级** | **9 域 mTLS 业务级 agent 链路 E2E** | **11 步客户端模拟器 v3 走完** |
| **ST-AGT-401** | **9/6 d15a0bb 3 NEW 域 k8s yaml** | **3 NEW 域 k8s yaml + mTLS cert 集成** | **scene / battle / account k8s yaml + mTLS cert 验证** |
| **ST-AGT-402** | **L-CAND-006 9 域 cert 轮换** | **9 域 cert 轮换 0 中断** | **9 域 cert 轮换, agent 链路保持** |
| **ST-AGT-403** | **L19 mTLS = saga 触达** | **mTLS 业务级 = saga 触达** | **mTLS + saga trace_id 贯通** |

## 2. 派生约束守护

| 派生约束 | 状态 | 引用 |
|---|---|---|
| L1 (cargo check 60s) | ✅ N/A (doc) | AGENTS.md §2.1 |
| L11 (cargo build dir lock) | ✅ N/A (doc) | AGENTS.md §2.2 |
| L12 (临时 log 不入 commit) | ✅ (git status 0 untracked) | AGENTS.md §2.2 |
| L13 (8 维度 git show --stat 实证) | ✅ (本节 §0 引用全部 commit 实证) | AGENTS.md §2.2 |
| L15 (native binary 跨工具链) | ⏳ (待 ST Phase C) | 6c6839e |
| L19 (mTLS 业务级 = saga 触达) | ✅ (ST-AGT-403 落档) | 6c6839e / 9/6 d270ab9 |
| L21 (跨工具链 gRPC Code 解析) | ✅ (per agent 链路) | 6c6839e |
| L23 (4 层自动探针) | ✅ (agent 链路 4 层探针) | 6c6839e |
| B3 (DDD Review 二审) | ⏳ (Mavis 自审 1 次停手, Ulysses 二审待) | AGENTS.md §3.x |
