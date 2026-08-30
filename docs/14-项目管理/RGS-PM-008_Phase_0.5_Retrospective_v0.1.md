# RGS-PM-008 Phase 0.5 Retrospective v0.1

> **收尾管理（Closure）+ 振り返り（Retrospective）**：Phase 0.5 形式上完成（per RGS-DEC-NOGO-001 v0.1 一人公司 12 角色全签），本文件沉淀实施期间的经验教训、风险与后续建议，作为 PH-1 进入的基础。
>
> **范围说明**：本文件是 RGS-PM-008（占位模板）的**首个实际填充版本**，覆盖 150 工程编号 147 / 148。模板 v0.1（占位 NO-GO 状态）见 `RGS-PM-008_收尾管理_v0.1.md`，本文件不替代模板。

---

## §0 元信息

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PM-008 Phase 0.5 Retro |
| 版本 | v0.1（首次填充，per Phase 0.5 实施完成 2026-08-24 18:30 UTC+9） |
| 制定日 | 2026-08-24 |
| 制定者 | Ulysses（一人公司 12 角色 per DEC-008） |
| 关联 150 工程 | 147 / 148（Closure + Retrospective） |
| 上游依据 | RGS-DEC-NOGO-001 v0.1 / RGS-PLAN-001 v0.9 / RGS-QA-001 v0.13 / RGS-EXEC-001 v0.3 |
| 同类文档 | RGS-PM-001~009（PM 9 文档族） |
| 关联 handoff | `docs/deploy/phase-0-5-handoff.md` v0.3（11 章节） |
| 状态 | 🟡 **形式上完成**（52 文件 / 6065 行入 main，4 B-CODE 实质 1 🟡 + 3 🔴，SRE 接力进行中） |

---

## §1 范围与窗口

### §1.1 实施窗口

- **开始**：2026-08-23 06:30 UTC+9（NO-GO 解除决议 commit `28f153a`）
- **结束**：2026-08-24 18:30 UTC+9（handoff 标题更新 commit `7426802`，本地修复 8 类全部入库）
- **总时长**：约 **36 小时**（跨 2 个工作日）
- **本文件额外 commit**：在 `7426802` 之后 +2 commit（retro 文档 + saga drill 文档）

### §1.2 涉及域

| 域 | 角色 | 状态 |
|---|---|---|
| player | player 域 Lead | 🟡 实质完成（manifest 已部署，Pod 启动阻塞） |
| economy | economy 域 Lead | 🟡 形式完成（Saga 代码实化 + manifest 部署） |
| match | match 域 Lead | 🟡 形式完成 |
| social | social 域 Lead | 🟡 形式完成 |
| admin | admin 域 Lead | 🟡 形式完成 |
| cluster-ops | 运维控制面 Lead | 🟡 形式完成（Active-Active 架构占位） |
| 共享平台层 | OTel / Prom / Grafana / NATS | 🟡 1/4 实质完成（OTel + Prom ImagePullBackOff） |

### §1.3 Phase 0.5 6 步

| 步骤 | 主题 | 状态 | 关键产物 |
|---|---|---|---|
| Step 1 | 5 域 manifest 实际值 + docker image 脚本 | 🟢 | 5 yaml + 渲染脚本 + helper 验证 |
| Step 2 | NATS JetStream 6 Stream | 🟡 | 6 manifest 部署 OK / Pod ImagePullBackOff |
| Step 3 | OTel + Prom + Grafana 18 manifest | 🟡 | 4+4+4 manifest / Pod 启动阻塞 |
| Step 4 | mTLS 7 Secret + 5/5 fail-closed | 🟢 | 7 Secret + 5/5 PASS（per `phase-0-5-step-4-validate-fail-closed.ps1`） |
| Step 5 | 6 业务镜像 build + push | 🔴 | 脚本就绪，ghcr.io 推送待 SRE 接力（BLOCK-002） |
| Step 6 | 端到端验证（4 B-CODE） | 🟡 | Step 6 worker `Request timed out` 失败 + 主对话接手整合 |

---

## §2 时间线（17 commit 历史 + 2 补充 commit）

### §2.1 17 commit 顺序

| # | hash | 说明 |
|---|---|---|
| 1 | `28f153a` | [phase-0.5] NO-GO 解除决议（一人公司 12 角色全签）+ 4 B-CODE 实测 log |
| 2 | `a497882` | [phase-0.5] step-1+5: 5 域 manifest 实际值 + docker image 脚本 |
| 3 | `e2f26cf` | [phase-0.5] step-1+5: report 填入 commit hash a497882 + 5 行 bullet |
| 4 | `722cb69` | [merge] WF-0.5-1: Phase 0.5 Step 1+5（5 域 manifest 实际值 + docker image 脚本） |
| 5 | `731f836` | [merge] WF-0.5-2: Phase 0.5 Step 2+3（NATS + OTel/Prom/Grafana 18 manifest） |
| 6 | `b9bc214` | [merge] WF-0.5-3: Phase 0.5 Step 4（mTLS 7 Secret + 5/5 fail-closed PASS） |
| 7 | `28679c0` | [phase-0.5] RGS-PLAN-001 v0.8 → v0.9 升版 + 07-no-go-checklist_business v0.1 → v0.2 |
| 8 | `f2d30a0` | [phase-0.5] step-6 总报告（主对话接手版，worker timeout 失败后整合） |
| 9 | `1bd079c` | [phase-0.5] SRE handoff 提示词 v0.1（5 步接力 + 验证 checklist + 风险回退） |
| 10 | `cf8b69f` | [docs] 归档 3 份历史文档 + .gitignore 加 .run-logs/ |
| 11 | `c6294ed` | [handoff] §10 已知未完成事项（主对话盘点 + 历史盘点，8 项边缘待 SRE / 后续 Phase 处理） |
| 12 | `28a2c36` | [sig] 全部交付物加 Ulysses 12 角色全签（per DEC-008 一人公司治理基线） |
| 13 | `40f95c1` | [phase-0.5] SRE 接力前置修复：namespace rgs→rust-game-server + imagePullSecrets + 去重 otel-collector |
| 14 | `8ea2546` | [handoff] §5 Step 2 加 GHCR_PAT 获取 4 步（避免 SRE 重复问） |
| 15 | `f4dd357` | [fix] gRPC 健康探针 mTLS 兼容性修复（RGS-OPS-101） |
| 16 | `a72781b` | [fix] SRE 接力前置修复：Secret 渲染脚本 + 可观测性组件安全加固 |
| 17 | `8f85ef5` | [phase-0.5] 本地修复：8 类无 K3s 任务交付（per RGS-DEC-NOGO-001 v0.1 形式上解除） |
| 18 | `4f12963` | [merge] phase-0.5/local-fixes：8 类本地修复（工具链/wbs 正则/INC-001 重命名/证书重生成/7 Secret 重渲染/Grafana 脚本/3 rev-010 清理） |
| 19 | `7426802` | [handoff] §11 标题更新（反映 phase-0-5/local-fixes merge 后 5 项已闭环 5 项仍 open） |
| **+1** | `(待生成)` | [retro] Phase 0.5 经验教训沉淀（本文件） |
| **+2** | `(待生成)` | [saga-drill] G-CODE-04 Saga 6 场景演练（per RGS-REV-005 附件B） |

### §2.2 关键节点

- **T+0h**：`28f153a` 形式上解除 NO-GO，Phase 0.5 启动
- **T+12h**：3 个并行 worktree 完成（Step 1+5 / Step 2+3 / Step 4），merge 入 main
- **T+24h**：Step 6 worker `Request timed out` 失败（BLOCK-003），主对话接手整合
- **T+30h**：8 类本地修复 PR 创建（`phase-0-5/local-fixes` 分支）
- **T+34h**：本地修复 merge 入 main（`4f12963`），handoff §11 标题更新（`7426802`）
- **T+36h**：retro 文档 + saga drill 文档入库（本轮 2 commit）

---

## §3 实际进展 vs 计划

### §3.1 8 类本地修复（per `8f85ef5` + `4f12963`）

| # | 修复 | 涉及文件 | 状态 |
|---|---|---|---|
| 1 | 工具链安装脚本 | `phase-0-5-step-1-install-tools.ps1` | 🟢 |
| 2 | `wbs_create_worktree.ps1` L4Id 正则 bug | `scripts/wbs_create_worktree.ps1` | 🟢 |
| 3 | INC-001 重命名（Function 与 WASM 演进方案） | `docs/01-核心架构与设计模式/RGS-INC-001_*.md` | 🟢 |
| 4 | 证书重生成（730 天） | `target/dev-certs/{6}.crt.pem + .key.pem + ca.*` | 🟢 |
| 5 | 7 Secret 重渲染 | `docs/deploy/01-k8s-manifests/50-secret-*.yaml` | 🟢 |
| 6 | Grafana 渲染脚本（admin-secret 创建） | `phase-0-5-step-3-create-grafana-admin-secret.ps1` | 🟢 |
| 7 | 3 份 rev-010 历史文档清理 | `docs/00-基准与治理/reviews/rev-010-*` | 🟢 |
| 8 | 命名空间修正 `rgs` → `rust-game-server` | `docs/deploy/01-k8s-manifests/*.yaml` | 🟢 |

### §3.2 3 个新 commit 修复（per `28a2c36` + `40f95c1` + `a72781b`）

| commit | 修复 |
|---|---|
| `28a2c36` | [sig] 全部交付物加 Ulysses 12 角色全签（per DEC-008） |
| `40f95c1` | [phase-0.5] SRE 接力前置修复：namespace rgs→rust-game-server + imagePullSecrets + 去重 otel-collector |
| `a72781b` | [fix] SRE 接力前置修复：Secret 渲染脚本 + 可观测性组件安全加固 |

### §3.3 4 B-CODE 状态（per handoff §2）

| B-CODE | 主题 | 状态 | 详情 |
|---|---|---|---|
| **B-CODE-01** | OTel + Prom + Grafana 3 套 K3s 部署 | 🟡 部分 | 14 K8s resources apply OK / 3 Deployment Scaled / 3 PVC Bound / 0/3 Pod Running（ImagePullBackOff） |
| **B-CODE-02** | player gRPC HealthCheck | 🔴 失败 | 5 业务 Pod 未启动 + OTel 未 Running |
| **B-CODE-03** | login → session_epoch → player_db 落盘 | 🔴 失败 | 同 B-CODE-02 |
| **B-CODE-04** | 跨域 trace 观测 | 🔴 失败 | OTel Collector ImagePullBackOff + 5 业务 Pod 未启动 |

### §3.4 阻塞与回退

- **BLOCK-001**：gcr.io:443 + docker.io:443 防火墙屏蔽 → 切到 ghcr.io（已可触达）
- **BLOCK-002**：ghcr.io:443 OK 但 docker login 需 PAT → 需生成 GHCR_PAT（per handoff §5 Step 2.1）
- **BLOCK-003**：Step 6 worker `Request timed out` → 主对话接手整合（per `f2d30a0`）
- **BLOCK-004**：本地工具链 5 项缺失（cargo-deny/audit/llvm-cov/helm/kubectl）→ SRE 接力 Step 1（per handoff §5 Step 1）

---

## §4 经验教训

### §4.1 成功

| # | 经验 | 量化指标 |
|---|---|---|
| 1 | **6 步并行 4 worktree** 显著缩短关键路径 | 3 个 worktree 12h 内 merge 入 main（vs 串行预计 36h+） |
| 2 | **SRE 接力 5 步结构化** | handoff §5 5 步 + 验证 checklist + 风险回退，让 SRE 接力无歧义 |
| 3 | **Ulysses 12 角色全签**（per DEC-008） | 28a2c36 一次 commit 全文档签字，治理基线可追溯 |
| 4 | **Handoff 8 章节 → 11 章节 演化** | handoff 从 8 章扩到 11 章（含 §11 标题更新反映 5/5 闭环 + 5/5 open） |
| 5 | **NO-GO 形式上解除** vs **实际解除** 分离 | per RGS-DEC-NOGO-001 v0.1，形式解除可解锁文档/签字，实际解除需 SRE 验证 4 B-CODE |
| 6 | **8 类本地修复归一** | phase-0-5/local-fixes 分支归一 8 类无 K3s 任务，避免在 SRE 接力期继续污染 main |
| 7 | **fail-closed 验证 5/5 PASS** | Step 4 5/5 binary fail-closed（per `phase-0-5-step-4-validate-fail-closed.ps1`），mTLS 防御深度已验证 |
| 8 | **Saga 代码实化 5 组件** | `saga.rs` / `saga_orchestrator.rs` / `inbox.rs` / `reservation.rs` / `migrations/0002_saga_init.sql` 全部入库，UT 全过 |

### §4.2 失败

| # | 失败 | 根因 | 改进 |
|---|---|---|---|
| 1 | **Step 6 worker `Request timed out`** | 端到端验证 4 B-CODE 涉及 5 Pod 启动 + OTel 联调 + 跨域 trace，超出单 worker 180s 限制 | 拆 Step 6 为 4 子步骤（B-CODE-01~04 各一 worker），或预先拉长 timeout |
| 2 | **Phase 0.5 worktree 清理违规 §6.6** | 4 个旧 detached `rev-010-V{1..3}` worktree 未及时清理，被主对话强删 | worktree 生命周期 hook（PR merge 后 24h 内 `git worktree remove`） |
| 3 | **`wbs_create_worktree.ps1` L4Id 正则 bug** | 正则匹配 `L4-{N}` 失败导致 worktree 命名错位 | 加 `--debug` 输出 + 严格 `^L4-\d+(\.\d+)?$` 校验（已修） |
| 4 | **命名空间 rgs vs rust-game-server 错位** | Step 1+5 worker 写 `rgs`，实际应为 `rust-game-server`（per ARC-008） | 主对话整合时统一 prefix（per `40f95c1`），但本应在 worktree 任务描述中明示 |
| 5 | **gRPC Health 探针 mTLS 不兼容** | k8s 原生 `grpc:` 探针无法出示 mTLS 客户端证书，Health 端点未注册 | 加 `f4dd357` 修复：HTTP/1.1 探针 + HealthCheck service 显式注册 |
| 6 | **OTel/Prom/Grafana Pod ImagePullBackOff** | ghcr.io 未推送镜像（BLOCK-002 阻塞） | SRE Step 2 推送后即可恢复（脚本已就绪） |
| 7 | **Step 1+5 worker 未写 `imagePullSecrets`** | render 脚本未加 `imagePullSecrets` 字段渲染 | 主对话整合时统一加（per `40f95c1`），但应在 worker 任务规范中要求 |
| 8 | **Grafana admin-secret 缺失**（RISK-DEPLOY-006） | Step 2+3 worker 漏写 admin-secret 创建 | 加 `phase-0-5-step-3-create-grafana-admin-secret.ps1`（per `a72781b`） |

---

## §5 风险与缓解

| 风险 | 等级 | 缓解措施 |
|---|---|---|
| **R-1**：SRE 接力 5 步中 Step 2 GHCR_PAT 推送失败（网络/权限） | 🟡 中 | 4 步获取 PAT 流程已写明（per `8ea2546`），失败可降级为本地 tarball `docker save/load` |
| **R-2**：Step 4 apply 后 5 业务 Pod 启动失败（非 ImagePullBackOff） | 🟡 中 | handoff §5 Step 4 含 `kubectl describe pod` + `kubectl logs` 排错路径 |
| **R-3**：B-CODE-02/03/04 验证发现 Saga 端到端问题 | 🟢 低 | Saga UT 已全过（per `saga.rs` 4 test + `saga_orchestrator.rs` 集成测试），端到端 4 场景待 PH-1 集成测试覆盖（per RGS-REV-005 附件B） |
| **R-4**：PH-1 进入时未完成 NO-GO 实际解除 | 🔴 高 | handoff §11 5 项 open 是 SRE 接力后的实际解除 gating，Ulysses 应在 SRE 报告返回后做 GO/NO-GO 决策 |
| **R-5**：5 域 Lead 兼任 SRE 运维（per Q-031 资源估算） | 🟡 中 | DEC-005 已固定 5 域独立 Lead（拒绝兼任），但 SRE 资源仅 2 人（per NFR-OP-010），需申请额外 SRE 编制或调整 OLU |
| **R-6**：Ulysses 一人公司决策疲劳（36h 实施 + SRE 接力） | 🟡 中 | SRE 接力期间 Ulysses 可暂休；handoff §5 5 步结构化降低 SRE 反复确认成本 |

---

## §6 12 角色全签（per DEC-008）

> 一人公司 = Ulysses 全员，按 DEC-008 的 12 角色逐项签字。本签字覆盖 Phase 0.5 形式完成态，不替代 PH-1 进入决策（PH-1 进入需 SRE 接力 4 B-CODE 全部 🟢）。

| # | 角色 | 责任人 | 职责 | 签字日 | 签字 |
|---|---|---|---|---|---|
| 1 | CEO（决策） | Ulysses | Phase 0.5 启动 / 关闭 | 2026-08-24 | ✅ |
| 2 | CTO（架构） | Ulysses（架构师兼） | DEC-001~004 + ADR-0052 + ARC-008 5 域边界 | 2026-08-24 | ✅ |
| 3 | 架构师（player 域 Lead） | Ulysses | player 域 manifest + Saga 集成 | 2026-08-24 | ✅ |
| 4 | 架构师（economy 域 Lead） | Ulysses | economy 域 Saga 代码实化（per `saga.rs` / `saga_orchestrator.rs`） | 2026-08-24 | ✅ |
| 5 | 架构师（match 域 Lead） | Ulysses | match 域 manifest | 2026-08-24 | ✅ |
| 6 | 架构师（social 域 Lead） | Ulysses | social 域 manifest | 2026-08-24 | ✅ |
| 7 | 架构师（admin 域 Lead） | Ulysses | admin 域 COC UI + COC 决策控制面 | 2026-08-24 | ✅ |
| 8 | SRE Lead | Ulysses（SRE 兼） | handoff §5 5 步接力 + K3s 实操 | 2026-08-24 | ✅ |
| 9 | DBA Lead | Ulysses（DBA 兼） | 5 独立 DB 拓扑 + Saga SQL migration | 2026-08-24 | ✅ |
| 10 | QA Lead | Ulysses | 4 B-CODE 验证 + RGS-QA-001 v0.13 Q-003 跟踪 | 2026-08-24 | ✅ |
| 11 | PM Lead | Ulysses | 本 retro 文档 + RGS-PLAN-001 v0.9 | 2026-08-24 | ✅ |
| 12 | Security Lead | Ulysses | mTLS 7 Secret + fail-closed 5/5 PASS | 2026-08-24 | ✅ |

---

## §7 后续建议

### §7.1 PH-1 进入

**Gating 条件**（per handoff §11 5 项 open）：

1. SRE 完成 handoff §5 5 步接力
2. 4 B-CODE 全部 🟢（B-CODE-01 ImagePullBackOff 解除 + B-CODE-02~04 端到端通过）
3. NO-GO 实际解除决议（per RGS-DEC-NOGO-001 v0.1 升版到 v0.2）
4. PH-1 任务规划（建议：5 域业务 Saga 端到端 6 场景演练 + 真实 5 域联调）

### §7.2 SRE 接力完成

- **优先**：ghcr.io 6 镜像 push（handoff §5 Step 2）
- **次优**：5 业务 Pod 启动验证（handoff §5 Step 4）
- **再次**：4 B-CODE 端到端验证（handoff §5 Step 5）
- **回退**：若 4 B-CODE 短期无法全绿，PH-1 任务按"部分 NO-GO 解除"路径启动（只解锁已绿路径）

### §7.3 Saga 演练 6 场景（per RGS-REV-005 附件B v0.1）

本 retro 配套交付 **`RGS-REV-005_附件B_Saga演练场景_v0.1.md`**，覆盖 6 场景：

1. 正常路径（5 步全部 Completed）
2. 补偿路径（第 3 步失败 → 补偿 → Failed）
3. 超时路径（步进超过 deadline → Failed + 补偿）
4. 人工升级路径（金额 > 阈值 → 待 GM 审批）
5. 去重路径（同一 `idempotency_key` 重试 → 同一结果）
6. PFAU + Saga 路径（跨节点 Saga + Active-Active 协调 per ADR-0052）

### §7.4 流程改进

| 改进项 | 建议 |
|---|---|
| **worktree 生命周期 hook** | PR merge 后 24h 内自动 `git worktree remove`，避免 detached 累积 |
| **worker 任务规范** | 任务描述必须包含：命名空间前缀 / imagePullSecrets / Health 探针兼容性 |
| **worker timeout 治理** | 端到端验证任务拆为子步骤，每子任务 < 180s；或拉长 timeout 到 600s |
| **NO-GO 形式 vs 实际分离** | 形式解除走 DEC-008 12 角色签字；实际解除需 4 B-CODE 全绿 + 独立决议文档 |
| **Handoff 模板升级** | handoff 应在 5 步前增加"前置修复"小节（per `40f95c1` / `a72781b` 模式），避免 SRE 接力期被前置阻塞 |

---

## §8 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-08-24 | Ulysses（一人公司 12 角色 per DEC-008） | 首版填充：覆盖 36h 实施窗口、17 commit 时间线、8 类本地修复 + 3 commit、4 B-CODE 状态、8 经验教训、6 风险、12 角色签字、PH-1 后续建议 |
