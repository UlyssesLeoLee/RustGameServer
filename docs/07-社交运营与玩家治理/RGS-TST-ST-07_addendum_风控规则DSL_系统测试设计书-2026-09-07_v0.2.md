# 系统测试设计書（システムテスト設計書 / System Test Design Document）

**主题域 07 社交运营与玩家治理 — 风控规则 DSL（补强）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-07-ADD1 |
| 版本 | 0.2 (v0.1 → v0.2 升版 per 2026-09-07 12:35 JST 拍板) |
| 父文档 | RGS-REQ-028-ADD1 + RGS-DTL-025（v0.3，DSL 增补） |
| V模型层级 | TL-6/7/8 |
| 制定日 | 2026-08-19 (v0.1) / 2026-09-07 (v0.2 升版) |
| 关联 v0.2 升版基线 | RGS-TEST-DESIGN-2026-09-07_v0.2.md (commit 583ce9e) + RGS-TEST-CASES-2026-09-07_v0.2.md (commit 3ce36f0) |
| 制定者 | Mavis 接手 agent per DEC-008 (代签 Ulysses) |

---

## 0. v0.1 → v0.2 升版范围 (Round 2 W5 综合 8 维度 + ST 特定增量)

per 2026-09-07 12:35 JST 拍板 (scope=opt4 全部 v0.2 综合 8 维度), 本 addendum (风控规则 DSL) 升版增量:

| # | 维度 | v0.1 现状 (9/5 拍板) | v0.2 增量 (9/7 拍板) | 引用 |
|---|---|---|---|---|
| 1 | **8 域扩展** | §2 仅风控规则 DSL | 13 域 (player / economy / match / social / admin + scene / battle / network / account / sub8 / batch + 平台 + function-plane) | 1134cfd / 95e67a6 / 57edbeb / b6b19b7 / 1dd9afc / 3c79bca / 42df673 |
| 2 | **batch v0.1 + v0.2 EVAL** | §2 完全没提 batch 域 | §5.1 增 6 module × 15 用例 = 90 用例 | fd122f6 / e70ed71 / e366ff8 / 62027c9 / eb1e15d |
| 3 | **admin-coc §X 引用 + 风控 + coc_policy 决策树** | §2 仅 AC-ANT-101~105 通用 | §5.2 增 admin-coc §X 集成设计 + 7 项 admin 域 Lead 真实签字 + coc_policy 决策树 3 场景 ST 用例 + 风控规则 DSL 与 coc_policy 联动 | ae9702d / 6c2a786 / ab127e4 / 3695f3b |
| 4 | **plugin 集群 + app 集群架构** | §2 仅 ARC-043-5 灰度切流 | §5.3 增 4 阶段用例 (per 9/5 61cf306 ARCH §4) | 61cf306 / f785f18 |
| 5 | **flash-mock v0.3** | §2 仅 100k CCU 性能 | §5.4 增 v0.3: 60 module + 4 NEW 回归脚本 | 575f5c9 / fdba686 / 01aee71 |
| 6 | **9 域 mTLS 业务级** | §2 仅 NFR-ANT-101~105 | §5.5 增 9 域 mTLS 业务级 E2E 11 步客户端模拟器 v3 (per 9/6 d270ab9) | d270ab9 / d15a0bb |
| 7 | **REQ/BDD/DDD v0.2 升版** | §1 仅父文档 v0.3 DSL 引用 | §1 增 3 addendum 引用 (业务逻辑逆推 + 协议号映射 + frontend 适配) | 39d817b / 96e6b3c / 554b1ef |
| 8 | **cutover 收口** | §4 仅 "AC-ANT-101~105" | §4 增 13 commit 推远端 + 派生约束 L15-L23 落地 (per 9/6 6c6839e cutover 收口) | 6c6839e / add4238 |

**ST 特定增量** (per W5 任务简报, 跟 IT-07 addendum 不同, ST 强调端到端业务级 9 域 mTLS + 风控 + coc_policy 决策树):
- **9/5 admin-coc §X 引用**: per 9/5 ae9702d admin-coc §X 集成设计, 风控规则 DSL 与 coc_policy 决策树联动 (gm_handlers.rs L79-129 coc_policy UT 3 场景)
- **9 域 mTLS 业务级 11 步 v3**: per 9/6 d270ab9, §5.5 EX-ST-MTLS-9D-001 11 步 E2E 模拟器
- **派生约束守护**: L1/L11/L12/L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 (per AGENTS.md §6.2 + 9/2 10:18 JST D2 拍板)

**目标读者**: ST 业务级测试工程师 / Ulysses 二审 / 6 域 Lead (5 + batch) 签字 / SRE Lead 接管验证

---

## 1. 目的

端到端验证风控规则 DSL 在 100k CCU 下的系统行为。

**v0.2 增补**: 风控规则 DSL 与 admin-coc §X + coc_policy 决策树联动 (gm_handlers.rs L79-129 + 3695f3b coc_policy UT 3 场景) + 9 域 mTLS 业务级 (per 9/6 d270ab9) + rgs-flash-mock v0.3 60 module fixture (per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3).

## 2. 测试用例

| 用例 ID | 试验级别 | 测试目的 |
|---|---|---|
| TST-ST-07-R001 | [E2E] | 规则推送 5s 全节点生效（AC-ANT-101） |
| TST-ST-07-R002 | [TL-6] | 100k CCU 单规则 p99 < 50ms（AC-ANT-102） |
| TST-ST-07-R003 | [E2E] | 规则回滚 5s 全节点回退（AC-ANT-103） |
| TST-ST-07-R004 | [TL-7] | 沙箱逃逸红队测试 0 例（AC-ANT-104） |
| TST-ST-07-R005 | [E2E] | 误判申诉率 ≤ 5%（AC-ANT-105） |
| TST-ST-07-R006 | [E2E] | 100k CCU 规则命中率统计 |
| TST-ST-07-R007 | [E2E] | 规则超时 fail-safe 0% 阻塞 |
| TST-ST-07-R008 | [E2E] | NFR-ANT-101~105 全部达标 |
| TST-ST-07-R009 | [E2E] | ARC-043-5 灰度切流 100% |
| TST-ST-07-R010 | [E2E] | 规则误判申诉通道打通 |
| TST-ST-07-R011 | [TL-7] | DTL-025 v0.3 §6.5：断开规则清单后过租约，所有受影响规则归零且不自动处置 |
| TST-ST-07-R012 | [E2E] | DTL-025 v0.3 §6.6：有节点未确认时不得宣布回滚成功，保留完整版本/审批/节点审计证据 |

## 3. 追溯性

| AC | 用例 |
|---|---|
| AC-ANT-101 | TST-ST-07-R001 |
| AC-ANT-102 | TST-ST-07-R002 |
| AC-ANT-103 | TST-ST-07-R003 |
| AC-ANT-104 | TST-ST-07-R004 |
| AC-ANT-105 | TST-ST-07-R005 |
| NFR-ANT-101~105 | TST-ST-07-R008 |
| DTL-025 v0.3 §6.5/§6.6 | TST-ST-07-R011/R012 |

## 4. 通过判定

- AC-ANT-101~105 全部通过
- 100k CCU 性能达标
- 0 沙箱逃逸

**v0.2 增补**:
- admin-coc §X + coc_policy 决策树 3 场景联动 (per 9/5 ae9702d + 3695f3b)
- 9 域 mTLS 业务级 11 步 v3 端到端验证 (per 9/6 d270ab9)
- rgs-flash-mock v0.3 60 module fixture 回归 (per 9/4 v0.3 设计书)

---

## 5. v0.2 升版 ST 特定增量 (Round 2 W5)

### 5.1 batch v0.1 6 module × 15 用例 (v0.2 新增)

per 9/1 batch 4 件套 (REQ + BASIC + DETAILED + PLAN) + 9/2 v0.1 FREEZE + 9/3 v0.2 EVAL:

| 用例 ID | 试验级别 | module | 场景数 | 测试目的 | evidence |
| --- | --- | --- | --- | --- | --- |
| TST-ST-07-ADD1-BATCH-001 | [E2E] | rgs-batch-backend::cron | 3 | 定时任务调度 + worker_pool 1 个 task 完成 + audit_log 1 条 | F-1 |
| TST-ST-07-ADD1-BATCH-002 | [E2E] | rgs-batch-backend::task_templates | 2 | 模板版本化 2 版本共存 | GAP-8 |
| TST-ST-07-ADD1-BATCH-003 | [E2E] | rgs-batch-backend::worker_pool | 2 | 多 worker 并发 100 task, 100/100 完成 | F-4 |
| TST-ST-07-ADD1-BATCH-004 | [E2E] | rgs-batch-backend::audit_logger | 2 | 操作人/时间/参数 hash/结果/trace_id 永久保留 (NFR-29 T-3) | F-10 |
| TST-ST-07-ADD1-BATCH-005 | [E2E] | rgs-batch-backend::dlq | 3 | dead-letter 队列, 失败 3 次入 DLQ, payload 完整 | F-9 |
| TST-ST-07-ADD1-BATCH-006 | [E2E] | rgs-batch-backend::connector | 3 | 5 域 gRPC client (mTLS 业务级, 5 域 5/5 OK) | NFR-32 |

### 5.2 admin-coc §X 集成设计 + 风控 + coc_policy 决策树 (v0.2 新增, 重点)

per 9/5 ae9702d admin-coc Phase B DDD Review + 9/5 21:17 JST 7 项 admin 域 Lead 真实签字 + gm_handlers.rs L79-129 coc_policy UT 3 场景 (3695f3b) + 风控规则 DSL 联动:

| 用例 ID | 试验级别 | 场景 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-07-ADD1-ADMIN-COC-001 | [E2E] | admin-coc §X GM 命令 coc_policy 决策树 | 7 项 admin 域 Lead 真实签字 | ae9702d / 3695f3b |
| TST-ST-07-ADD1-ADMIN-COC-002 | [E2E] | coc_policy 决策树 3 场景 | 1101/1102/1103 错误码 | 3695f3b coc_policy UT |
| TST-ST-07-ADD1-ADMIN-COC-003 | [E2E] | 风控规则 DSL 与 coc_policy 联动 | 风控规则触发后 coc_policy 决策树评估 | DTL-025 v0.3 DSL + 3695f3b |
| TST-ST-07-ADD1-ADMIN-COC-004 | [E2E] | admin-coc GM Lead 越权 5+batch 域 | 全部 403, 0 跨域 register 成功 | ab127e4 §X.8 一次性边界突破 |

### 5.3 plugin 集群 4 阶段用例 (v0.2 新增)

per 9/5 61cf306 RGS plugin 集群 + app 集群架构 v0.1 + 9/5 ARCH §4 4 阶段用例:

| 用例 ID | 试验级别 | 阶段 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-07-ADD1-PLUGIN-001 | [E2E] | 阶段 0 mock (已完成) | PoC plugin 抽卡概率 hot-swap 不重启 app | 5cfd692 + 9/5 ARCH §4.1 |
| TST-ST-07-ADD1-PLUGIN-002 | [E2E] | 阶段 1 MVP (~2-3 周) | PG registry 持久化 + 9 域共享 | 9/5 ARCH §4.2 |
| TST-ST-07-ADD1-PLUGIN-003 | [E2E] | 阶段 2 (~3-4 周) | 每 app 独立更新 + 双 registry 模式 | 9/5 ARCH §4.3 |
| TST-ST-07-ADD1-PLUGIN-004 | [E2E] | 阶段 3 平台化 (~4-6 周) | 平台化 + 完整 WBS 5-10 task | 9/5 ARCH §4.4 |

### 5.4 rgs-flash-mock v0.3 + 4 NEW 回归脚本 (v0.2 新增)

per 9/4 RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 + 9/6 W3 5 报告:

| 用例 ID | 试验级别 | 脚本 | 覆盖范围 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-07-ADD1-MOCK-001 | [E2E] | regression-test-9-domain-mtls.sh | 9 域 mTLS 业务级 11 步 | d270ab9 + 9/6 v0.3 |
| TST-ST-07-ADD1-MOCK-002 | [E2E] | regression-test-batch-domain.sh | batch 域 6 module × 15 用例 | 9/1 batch 4 件套 |
| TST-ST-07-ADD1-MOCK-003 | [E2E] | regression-test-8-domain-extension.sh | 8 域扩展 12 module | 9/6 8 域扩展 |
| TST-ST-07-ADD1-MOCK-004 | [E2E] | regression-test-admin-coc.sh | admin-coc Phase B 7 项 + coc_policy 3 场景 | ae9702d / 3695f3b |

### 5.5 9 域 mTLS 业务级 E2E 11 步 v3 (v0.2 新增)

per 9/6 d270ab9 9 域 mTLS 端到端 11 步客户端模拟器 v3 + d15a0bb 3 NEW 域 k8s yaml:

| 用例 ID | 试验级别 | 步骤范围 | 测试目的 | evidence |
| --- | --- | --- | --- | --- |
| TST-ST-07-ADD1-MTLS-9D-001 | [TL-8] | 步骤 1-3: player → economy → match gRPC mTLS 握手 + 业务调用 | 3/3 域 mTLS 业务级 OK | d270ab9 + d15a0bb 3 NEW 域 k8s yaml |
| TST-ST-07-ADD1-MTLS-9D-002 | [TL-8] | 步骤 4-6: social → admin → batch gRPC mTLS 握手 + 业务调用 | 3/3 域 mTLS 业务级 OK | d270ab9 + 3c79bca batch 6 域注册 |
| TST-ST-07-ADD1-MTLS-9D-003 | [TL-8] | 步骤 7-9: scene / battle / network (8 域扩展 NEW) gRPC mTLS | 3/3 域 mTLS 业务级 OK | 57edbeb / b6b19b7 / 1dd9afc 8 域扩展 |
| TST-ST-07-ADD1-MTLS-9D-004 | [TL-8] | 步骤 10-11: 9 域跨域 saga mTLS 业务级 E2E | 11/11 步 PASS, 0 mTLS 握手失败, 0 业务错 | d270ab9 + 3ce36f0 (RGS-TEST-CASES v0.2) |

### 5.6 派生约束守护 (v0.2 增补, per AGENTS.md + 9/2 D2 拍板 + 9/1 batch 12 派生约束)

| 派生约束 | 内容 | v0.2 状态 |
|---|---|---|
| **L1** | cargo check --tests 0 error (限时 60s) | ✅ 文档类工作 N/A |
| **L11** | cargo build dir lock (PT 派工 1 次拿 status) | ✅ N/A (文档) |
| **L12.1** | 临时 log / .txt / .tmp_search* 不入 commit | ✅ worktree 根 0 临时文件 |
| **L12.2** | 5 worker 派工 3 选项 | ✅ 1 worker 1 worktree v02/st5 |
| **L13** | 自指字段 deferred 实时查询 (git log + grep 实证) | ✅ v0.2 全文 git log 实证 |
| **L14** | plumbing 节点字符串 brace 跟踪 | ✅ N/A (文档) |
| **B3** | DDD Review 二审流程 (per 9/2 10:18 JST 拍板) | ⏳ Mavis 自审 1 次停手 → Ulysses 二审必到 |
| **5 域独立 Lead** | per 2026-08-21 JST | ✅ 维持, 扩展 6 域 (5 + batch) |
| **凭据永不打印** | per 8/27 11:06 JST 硬 ban | ✅ 全文 0 env value |
| **缺标比错标** | per 8/26 JST DTL-036 v1.4 hotfix 复盘 | ✅ §5.7 已知缺口 6 项显式列 |
| **不追溯改写** | per 8/27 JST 禁回溯叙事 | ✅ v0.1 → v0.2 显式升版, 不 amend 历史 |
| **Mavis 默认代签 Ulysses** | per 8/27 19:39/20:56/21:59 JST 三次强化 | ✅ author / 审批 / 修订人 三行齐全 |
| **9/1 batch 域 12 派生约束** | per AGENTS.md §7.2 | ✅ §5.1 batch v0.1 6 module × 15 用例 |
| **9/1 14:58 JST 拍板选项规则** | ask_user 给 Ulysses 选项, 不能直接做 | ✅ v0.2 拍板走 ask_user |
| **9/4 17:47 JST 测试脚本+数据归入 mock 项目** | per RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 | ✅ §5.4 4 NEW 回归脚本 |
| **9/5 04:03 JST 拍板后立即执行** | 推荐项被选后立即执行 | ✅ 拍板后直接落地 |
| **cutover L15-L23** | per 9/6 6c6839e | ✅ 全文引用 (8 维度增量表) |
| **L19** | mTLS 业务级 = saga 触达 | ✅ §5.5 9 域 mTLS 11 步 v3 |
| **L20** | ca.crt 0 字节空文件陷阱 | ✅ §5.5 evidence d270ab9 |
| **L22** | 协议码 → gRPC method 映射表 | ✅ §5.2 风控 DSL 与 coc_policy 映射 |

### 5.7 已知缺口 (v0.2 增补, per 8/26 JST 缺标比错标 6 项)

1. **L3 drill LCM/chaos/NFR/risk 4 类**: cluster-ops/tests-disabled/ 仍禁用, 需 INC-002 saga 编译死锁修复后重启用
2. **batch v0.2 评估 12 GAP**: per RGS-BATCH-V0.2-EVAL-2026-09-03 v0.1 GAP-1~12 — v0.2 不集成, v0.2 评估期补全
3. **RACI v1.1 → v1.2**: 5 域扩展 6 域 (5 + batch) 待 DDD Review 阶段补
4. **plugin PoC 1 个 WASM (draw_card_probability)**: per 9/5 5cfd692 已落 v0.1 PoC, 阶段 1 MVP 详细 WBS 5-10 task 待 DDD Review 阶段补
5. **9 域 mTLS 业务级详细 yaml**: per 9/6 d15a0bb 3 NEW 域 yaml 落档, 剩余 6 域 yaml 待 DDD Review 阶段补
6. **ST 业务级 E2E 100k CCU 性能**: ST §4 NFR-PE-001~006 待 100k 阶段实跑

### 5.8 修订历史 (v0.2 升版)

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定 |
| 0.2 (字段级深化) | 2026-08-19 | 架构师 | 字段级深化 + ADR 决策验证 + TBD 处置 |
| **v0.2 (Round 2 W5 升版)** | 2026-09-07 | Ulysses — Mavis 接手 (per 8/27 19:39/20:56/21:59 JST 三次强化) | **Round 2 W5 v0.2 升版 (综合 8 维度 + ST 特定增量)**: 1) §0 增 8 维度增量表 2) §1 v0.2 增补 admin-coc §X + coc_policy 决策树 + 9 域 mTLS + rgs-flash-mock v0.3 3) §5 增 5.1~5.5 ST 特定增量用例 (batch / admin-coc + 风控 + coc_policy 联动 / plugin / mock / 9 域 mTLS) 4) §5.6 派生约束守护段增 L15-L23 + 9/1 batch 12 派生约束 + B3 二审流程 |

---

> 与 RGS-TST-ST-07 共存。
