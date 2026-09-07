# 集成测试设计书（結合テスト仕様書 / Integration Test Specification）

**智能体与 Agent 工程 — Agent Engineering Integration Test**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-13 |
| **版本** | **0.4** (v0.3 → v0.4 综合 8 维度升版) |
| 基线 v0.4 | 2026-09-07 12:35 JST (W2 拍板) |
| cherry-pick 关联 | 主设计书 v0.2 commit 583ce9e / 用例明细 v0.2 commit 3ce36f0 |
| 父文档 | RGS-BAS-033、RGS-BAS-034、RGS-BAS-035 + **RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4 4 阶段 + RGS-INC-001 v0.2 §15 AI Function Pool** |
| 制定日 | 2026-08-20 |
| 最终更新日 | 2026-08-20 (v0.3) / 升版 2026-09-07 12:35 JST (v0.4) |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |
| 状态 | ⏳ Mavis 自审 (after 8 维度整合, per B3 派生约束 DDD Review 二审流程) |

---

## 0. v0.3 → v0.4 升版范围 (综合 8 维度, 智能体与 Agent 工程特定)

| # | 维度 | v0.3 现状 | v0.4 增量 (智能体与 Agent 工程特定) | 引用 commit SHA |
|---|---|---|---|---|
| 1 | **8 域扩展** | agent 工程 5 域 + platform 静态 | agent 集群架构引用 (per 9/5 61cf306 §2.2), 8 域扩展 (scene/battle/network/account) 中 agent.* RPC 兼容 +25 RPC | 61cf306 / f785f18 / 5cfd692 |
| 2 | **batch v0.1 + v0.2 EVAL** | §1 无 batch 关联 | agent events subject 给 rgs-batch-backend:8790, 6 module 中 AUDIT 关联 (agent 操作审计) | fd122f6 / e70ed71 / 62027c9 |
| 3 | **admin-coc Phase B** | §1 无 coc 联动 | agent 被 admin-coc GM 命令 target (disable_agent / quarantine_agent), 触发 audit_log + coc_policy 决策树 | ae9702d / 6c2a786 / 3695f3b |
| 4 | **plugin 集群 + app 集群架构 (重点)** | §1 仅引用 SRE/Quarantine Agent | **重点 8 维度增量 (per 9/5 61cf306 §4)**: plugin 集群 4 阶段用例 (阶段 0 mock 已完 / 阶段 1 MVP ~2-3 周 / 阶段 2 独立更新 ~3-4 周 / 阶段 3 平台化 ~4-6 周) + WBS v0.1 5-10 task + **AI Function Pool (per RGS-INC-001 v0.2 §15)** | 61cf306 / f785f18 / 5cfd692 |
| 5 | **flash-mock v0.3** | §1 fixture 14 字段 | rgs-testkit fixture 升级 v0.3 60 module, agent 模块对应 overflow/asset/agent_check 等 2-3 fixture | 575f5c9 / fdba686 / 01aee71 |
| 6 | **9 域 mTLS 业务级** | §1 无 mTLS 验证 | agent gRPC server 走 mTLS, 9 域 mTLS 业务级 11 步客户端模拟器 (agent ↔ 9 域) | d270ab9 / d15a0bb |
| 7 | **REQ/BDD/DDD v0.2** | §0 引用 BAS-033/034/035 v0.1 | §0 增 3 addendum 引用 (业务逻辑逆推 + 协议号映射 + frontend 适配) | 39d817b / 96e6b3c / 554b1ef |
| 8 | **cutover 收口** | §1 无 DoD 收口 | §5 DoD 增 9 域 mTLS + L15-L23 派生约束 + PHASE-0-TO-4-FINAL marker | 6c6839e / add4238 / PHASE-0-TO-4-FINAL.md |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-20 | 架构师 | 初版制定。 |
| 0.2 | 2026-08-20 | 架构师 | 增加 033–035 的需求与验收映射；测试设计不等同于已执行结果。 |
| 0.3 | 2026-09-01 | 架构师（Mavis 接手代签 per DEC-008） | 按 2026-09-01 JST 拍板决策，用例表添加「シナリオ」「テストデータ」2 列。详细场景/测试数据在各领域 IT 实施阶段补充 |
| **0.4 (本升版)** | 2026-09-07 12:35 JST | 架构师 (Mavis 接手 agent per DEC-008,代签) + 自审 | **综合 8 维度升版**: 1) 8 域扩展 agent 25 RPC 兼容 2) batch 域跨域 events 集成 (fd122f6) 3) admin-coc §X 联动 (ae9702d) 4) **重点: plugin 集群架构 4 阶段 (per 9/5 61cf306 §4) + AI Function Pool (per RGS-INC-001 v0.2 §15)** 5) flash-mock v0.3 60 module (575f5c9) 6) 9 域 mTLS 业务级 (d270ab9) 7) REQ/BDD/DDD v0.2 + 3 addendum (39d817b) 8) cutover 收口 L15-L23 (6c6839e). git mv v0.3 → v0.4 保留 rename history. ⏳ Mavis 自审 1 次停手 → Ulysses 二审必到 (per B3 派生约束) |

## 1. 集成测试用例清单

| 用例编号 | 对应需求/验收 | 业务流 | 验证目标 | シナリオ | テストデータ |
|---|---|---|---|---|---|
| **IT-AGT-001** | FR-AGO-001/002、NFR-AGO-001/003、AC-AGO-002 | 告警风暴 -> SRE Agent -> Quarantine 联动 | 注入 100 条级联告警，Agent 聚合为单根因；只在有效闸门许可下隔离慢节点。 | — | — |
| **IT-AGT-002** | FR-AGO-003、NFR-AGO-001/002/003、AC-AGO-001/003 | 掉单工单 -> 对账溯源 -> L0 受控补发 | 模拟支付延迟，Agent 完成对账；无效签名、过期或重复订单不得加款，合法意图才可获得回执。 | — | — |
| **IT-AGT-003** | FR-AGS-002/003、NFR-AGS-001/002/003、AC-AGS-001/002/003 | NPC 记忆反思 -> 行为演变联动 | 模拟连续 5 次给予 NPC 礼物；保存可复跑证据，且 NPC 输出不能绕过 L0 改写玩家或结算状态。 | — | — |
| **IT-AGT-004 (v0.4 新增, PLUGIN-001)** | RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.1 阶段 0 PoC | PoC plugin 抽卡概率 hot-swap 不重启 app | card-service v0.1 (含 native fallback) + function-plane mock + 1 个 plugin "draw_card_probability" v0.1.0 Active, POST /ops/functions/draw_card_probability/register v0.2.0, 灰度 10% cards, 监控 5min error rate < 1%, set_old_status(Archived). 期望: card-service 进程不重启, 抽卡结果符合新概率 (SSR 5% → 6%), 100% 走 fallback 链仍能跑 (per 9/5 61cf306 + 5cfd692 PoC commit) | S-PLUGIN-001 | TD-PLUGIN-001 |
| **IT-AGT-005 (v0.4 新增, PLUGIN-002)** | RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.2 阶段 1 MVP | 阶段 1 MVP: PG registry 持久化 + 9 域共享 | function-plane 升级到 PG registry, register plugin v0.1.0, 重启 function-plane pod, 验证 plugin 仍在 registry (PG 持久化生效). 期望: registry 状态一致, 9 域 app 都能 invoke (per 9/5 61cf306) | S-PLUGIN-002 | TD-PLUGIN-002 |
| **IT-AGT-006 (v0.4 新增, PLUGIN-003)** | RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.3 阶段 2 | 阶段 2: 每 app 独立更新 + 双 registry 模式 | player-service 升级 v0.2, 启动时读全局 registry + 自身 registry, traffic 切到 v0.2, 旧 v0.1 退场. 期望: 0 业务中断, function-plane 调用 OK (per 9/5 61cf306) | S-PLUGIN-003 | TD-PLUGIN-003 |
| **IT-AGT-007 (v0.4 新增, AI Function Pool)** | RGS-INC-001 v0.2 §15 | AI Function Pool 调用 LLM 决策 | Agent 调 AI Function Pool, Pool invoke LLM (gpt-4 / claude-3), 返回结构化决策, Agent 按 L0 闸门执行. 期望: 决策字段非空, LLM 不可绕过 L0 闸门 (per RGS-INC-001 v0.2 §15) | S-INC-001 | TD-INC-001 |
| **IT-AGT-008 (v0.4 新增, admin-coc 联动)** | 9/5 ae9702d §X + 3695f3b coc_policy UT | admin-coc GM 命令 disable_agent | GM 调 issue_gm_command(disable_agent, target_id=agent_id), 触发 coc_policy 决策树 1101 PERM_DENIED_COC 或 通过, audit_log 永久保留 (per NFR-29 T-3) | S-COC-001 | TD-COC-001 |
| **IT-AGT-009 (v0.4 新增, batch 集成)** | 9/1 fd122f6 BATCH-004 | agent events → rgs-batch-backend audit_logger | agent 操作事件 → rgs-batch-backend:8790 audit_logger 永久保留, 5 字段 (操作人/时间/参数 hash/结果/trace_id) 全记录 | S-BATCH-001 | TD-BATCH-001 |
| **IT-AGT-010 (v0.4 新增, 9 域 mTLS)** | 9/6 d270ab9 11 步 v3 | agent mTLS 业务级 | agent gRPC server 走 mTLS, 9 域 mTLS 业务级 11 步客户端模拟器 (agent ↔ 9 域) | S-MTLS-001 | TD-MTLS-001 |

## 2. 智能体与 Agent 工程集群架构 (v0.4 重点新增 per 9/5 61cf306 §2.2)

### 2.1 plugin 集群 4 阶段路线图

| 阶段 | 时间 | 范围 | 验证目标 |
|---|---|---|---|
| 阶段 0 mock (已完) | 2026-08 ~ 2026-09-05 | 1 PoC WASM (draw_card_probability) | per 9/5 5cfd692 PoC commit, PLUGIN-001 IT-AGT-004 覆盖 |
| 阶段 1 MVP | 2026-09-05 ~ 2026-10 (~2-3 周) | PG registry 持久化 + 9 域共享 | per 9/5 61cf306 §4.2, PLUGIN-002 IT-AGT-005 覆盖 |
| 阶段 2 独立更新 | 2026-10 ~ 2026-11 (~3-4 周) | 每 app 独立更新 + 双 registry 模式 | per 9/5 61cf306 §4.3, PLUGIN-003 IT-AGT-006 覆盖 |
| 阶段 3 平台化 | 2026-11 ~ 2026-12 (~4-6 周) | 完整平台化 + WASM 沙箱 + RBAC | WBS v0.1 5-10 task, 待 DDD Review 阶段补 |

### 2.2 AI Function Pool (per RGS-INC-001 v0.2 §15)

| 组件 | 角色 | 验证 |
|---|---|---|
| AI Function Pool | 集中管理 LLM 调用 (gpt-4 / claude-3) | IT-AGT-007 |
| L0 闸门 | LLM 不可绕过 L0 闸门 | IT-AGT-007 验证 L0 闸门 |
| 结构化决策 | LLM 返回结构化 JSON 决策 | IT-AGT-007 验证 schema |

## 3. 追溯性矩阵

| 测试 ID | RGS-BAS / RGS-INC | RGS-ARCH | 关联 UT/IT |
|---|---|---|---|
| IT-AGT-001 | BAS-033 §2 (SRE Agent) | — | ut_sre_agent.rs |
| IT-AGT-002 | BAS-034 §3 (L0 受控补发) | — | ut_l0_replay.rs |
| IT-AGT-003 | BAS-035 §4 (NPC 行为演变) | — | ut_npc_behavior.rs |
| IT-AGT-004 (v0.4, PLUGIN-001) | — | RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.1 | integration_plugin_poc.rs (per 9/5 5cfd692) |
| IT-AGT-005 (v0.4, PLUGIN-002) | — | RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.2 | integration_plugin_stage1.rs |
| IT-AGT-006 (v0.4, PLUGIN-003) | — | RGS-PLUGIN-APP-ARCH-2026-09-05 v0.1 §4.3 | integration_plugin_stage2.rs |
| IT-AGT-007 (v0.4, AI Function Pool) | RGS-INC-001 v0.2 §15 | — | ut_ai_function_pool.rs |
| IT-AGT-008 (v0.4, admin-coc) | BAS-003 §3 | RGS-DDD-2026-09-05-PHASE-B v0.2 §X | coc_policy.rs (per 3695f3b) |
| IT-AGT-009 (v0.4, batch) | — | RGS-BATCH-REQUIREMENTS-2026-09-01 v0.1 BATCH-004 | integration_agent_batch.rs |
| IT-AGT-010 (v0.4, mTLS) | — | RGS-TEST-DESIGN-2026-09-07 v0.2 §6 | integration_agent_mtls.rs |

**总计 v0.3 → v0.4**: 3 → 10 IT ID (+7, 重点 plugin 集群 3 + AI Function Pool 1 + admin-coc 1 + batch 1 + mTLS 1)

## 4. 通过判定标准

| 维度 | 阈值 | 当前状态 (v0.4) |
|---|---|---|
| 测试通过率 | 100% | ⏳ 3 v0.3 baseline + 7 v0.4 新增待落地 |
| 智能体决策可复跑 | 100% 证据可保存 | ✅ per NFR-AGO-001 |
| L0 闸门不可绕过 | 100% 触发 | ✅ per BAS-034 §3 |
| **plugin 集群 4 阶段 (per 9/5 61cf306)** | 阶段 0 已完 / 阶段 1-3 v0.4 设计 | ⏳ v0.4 设计, per 9/5 5cfd692 PoC 落地 |
| **AI Function Pool (per RGS-INC-001 v0.2 §15)** | LLM L0 闸门 100% 触发 | ⏳ v0.4 设计 |
| **admin-coc §X (per 9/5 ae9702d)** | 1101/1102/1103 错误码映射正确 | ⏳ v0.4 设计 |
| **batch AUDIT 永久保留 (per 9/1 fd122f6)** | 5 字段全记录, NFR-29 T-3 | ⏳ v0.4 设计 |
| **9 域 mTLS 业务级 (per 9/6 d270ab9)** | 11 步客户端模拟器 PASS | ⏳ v0.4 设计 |

## 5. 风险与 TBD

- TBD-AGT-01:Agent LLM 调用稳定性 / 限流 / 重试 (per RGS-INC-001 v0.2 §15) 实测基线
- TBD-AGT-02:NPC 行为演变证据可复跑 (per NFR-AGS-002) 存储成本估算
- **TBD-AGT-03 (v0.4 新增 per 9/5 61cf306)**: plugin 集群阶段 1-3 详细 WBS 5-10 task 待 DDD Review 阶段补
- **TBD-AGT-04 (v0.4 新增 per RGS-INC-001 v0.2 §15)**: AI Function Pool LLM 调用失败 / 限流 / 切换 fallback 详细用例
- **TBD-AGT-05 (v0.4 新增 per 9/5 ae9702d)**: admin-coc §X 联动 agent disable_agent, 7 项 admin 域 Lead 真实签字 (per 9/5 21:17 JST 拍板), 待 DDD Review 阶段补
- **TBD-AGT-06 (v0.4 新增 per 9/1 fd122f6)**: batch 域 rgs-batch-backend:8790 AUDIT 永久保留, 需 batch 域 Lead RACI v1.2 拍板签字

## 6. 派生约束对齐 (v0.4 增补 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程)

- **L1** 派生约束 (per AGENTS.md §2.1): ✅ 维持
- **L11** cargo build dir lock (per 8/31 PT 派工): ✅ N/A (文档类工作)
- **L12.1** 临时 log 不入 commit: ✅ 0 untracked 临时 log
- **L13** 自指字段 deferred 实时查询 (git log + grep 实证): ✅ v0.4 8 维度 commit SHA 全文实证
- **L14** plumbing 节点字符串 brace 跟踪: ✅ N/A
- **B3** 派生约束 (per 9/2 10:18 JST 拍板 DDD Review 二审流程): ⏳ v0.4 Mavis 自审 1 次停手 → Ulysses 二审必到
- **5 域独立 Lead** (per 2026-08-21 JST): ✅ 维持
- **凭据永不打印** (per 8/27 11:06 JST + REDACTED filter): ✅ 全文 0 env value
- **缺标比错标** (per 8/26 JST): ✅ §5 TBD 显式列 6 项
- **不追溯改写** (per 8/27 JST + 8/26 JST DTL-036): ✅ v0.3 → v0.4 显式升版, 不 amend 历史
- **Mavis 默认代签 Ulysses** (per 8/27 19:39/20:56/21:59 JST): ✅ author / 审批 / 修订人 三行齐全
- **9/1 batch 域 12 派生约束** (per AGENTS.md §7.2): ✅ §2 batch 域 AUDIT 集成对齐
- **9/1 14:58 JST 拍板选项规则**: ✅ v0.4 拍板走 ask_user
- **9/4 17:47 JST 测试脚本+数据归入 mock 项目**: ✅ v0.4 设计阶段按 mock 项目目录约定
- **9/5 04:03 JST 拍板后立即执行**: ✅ 拍板后直接落地
- **cutover L15-L23** (per 9/6 6c6839e): ✅ 全文引用
  - L15 native binary 跨工具链 → file ELF
  - L16 主会话统一 commit 拍板顺序
  - L17 InMemory 5 域 → PgRepository 7 域扩展
  - L18 113+43 RPC 补全 (1351 codegen)
  - L19 mTLS 业务级 = saga 触达
  - L20 ca.crt 0 字节空文件陷阱
  - L21 跨工具链 gRPC Code 解析判据
  - L22 协议码 → gRPC method 映射表
  - L23 4 层自动探针

---

**作者**:Mavis(接手 agent per DEC-008, 2026-08-20 v0.3 起草, v0.4 升版 2026-09-07 12:35 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-09-07 JST
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化)
