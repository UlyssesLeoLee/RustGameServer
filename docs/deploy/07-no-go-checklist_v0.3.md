# 07-no-go-checklist_v0.3.md — 部署前 NO-GO 自检表（顶层 summary，起動準備就绪 v0.3）

> **文档 ID**：`RGS-DEPLOY-NO-GO-CHECKLIST-001`
> **版本**：v0.3
> **生效日期**：2026-08-21
> **NO-GO 状态**：🔴 **维持 NO-GO**（2 项 G-CODE 待实测）
> **起動準備状态**：🟢 **準備就绪**（WBS 工具链 + 5 域 DTL 占位 + 7 份 CROSS 占位 + 4 个 WBS 脚本就位；G-CODE-03 + G-CODE-06 实测通过即 GO）
> **关联**：`../00-prerequisites/00-no-go-checklist_v0.2.md`（详细 12 类 + 7 G-CODE 拆解）+ `RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` + `RGS-EXEC-001 v0.3` + RGS-WBS-001 v0.3 §2A.7（激活条件）+ RGS-WT-001 v0.2 §11

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。NO-GO 状态表 + 7 G-CODE + 12 类签字栏 + 5 G-CODE 工具链。 |
| 0.2 | 2026-08-21 | 架构师（Ulysses）| **DEC-008 落地**（一人公司治理基线 per RGS-QA-001 v0.13 §9.5.7）：Ulysses = 全部 12 类角色实际签。**NO-GO 状态部分解除**：12 类签字栏 ✅ / 7 G-CODE ⚠️ 部分 Closed（G-CODE-06 / G-CODE-03 仍需实测）。 |
| 0.3 | 2026-08-21 | 架构师（Ulysses）| **NO-GO 起動準備就绪**（per 用户决策 2026-08-21 "跳到 53 起動准备，不等实测"）：① WBS v0.3 5 域 DTL 7+1 L4 任务 + 7 份 RGS-SPEC-CROSS-001~007 占位文档 + 4 个 WBS 工具脚本（`wbs_list` / `wbs_create_worktree` / `wbs_task_progress` / `wbs_merge`）就位 ② RGS-WT-001 v0.2 §11（WBS L4 任务 worktree 模式）升版 ③ RGS-WBS-001 v0.3 §2A.7（激活条件）已就绪 ④ RGS-WBS-001_L4任务进度表 v0.3 + RGS-WBS-001_DAG v0.3 占位已建。**NO-GO 仍维持**——G-CODE-03（5 独立 DB 拓扑图实测）+ G-CODE-06（Rust 1.98 + cargo build + cargo test 全绿实测）任一未实测前禁止 53 起動。 |

---

## 0. 重要声明

> ⚠️ **本表是部署启动前必查的顶层 summary**。本表 0 项 ✅ 之前**禁止执行** `../05-deploy-sop.md` 任何步骤、**禁止在 production 跑任何 deployment 命令**、**禁止在 production 执行任何 DB migration**。
>
> 详细分解见 `../00-prerequisites/00-no-go-checklist_v0.2.md`。

**v0.3 重大变化**：
- 🟢 **起動準備状态 = 準備就绪**（infrastructure + 文档 + 脚本就位，2 项实测即可 GO）
- 🔴 **NO-GO 状态 = 维持**（实测 G-CODE-03 + G-CODE-06 任一未完成前禁止 53）

---

## 1. 7 G-CODE 关闭状态（per RGS-EXEC-001 v0.3）

| G-CODE | 内容 | 当前状态 | 责任人 | 关闭条件 |
|---|---|---|---|---|
| **G-CODE-01** | 业务方代表具名签字 | ✅ **Closed** | Ulysses（业务方=PM 一人公司兼任）| Ulysses 实际签 2026-08-21 |
| **G-CODE-02** | 5 域 Lead 独立具名 | ✅ **Closed** | Ulysses（5 域 Lead 1 人串行兼任）| Ulysses 实际签 2026-08-21（DEC-008 撤销 DEC-005 独立要求）|
| **G-CODE-03** | DBA 具名 + 5 独立 DB 拓扑图签字 | ⚠️ **待实测** | Ulysses（DBA 一人公司兼任）| **5 独立 DB 拓扑图需 Ulysses 实际画过**（签字不构成证据，per RGS-EXEC-001 v0.3 §3.4）|
| **G-CODE-04** | SRE 具名 + 部署 SOP 签字 | ✅ **Closed** | Ulysses（SRE 一人公司兼任）| Ulysses 实际签 2026-08-21 + 05-deploy-sop.md 签字 |
| **G-CODE-05** | Platform 架构师具名 + CI/CD 签字 | ✅ **Closed** | Ulysses（Platform 一人公司兼任）| Ulysses 实际签 2026-08-21 + 04-ci-cd/ 签字 |
| **G-CODE-06** | Rust 1.98 + Cargo.lock + CI 全绿 | ⚠️ **待实测** | Ulysses（Platform + QA 一人公司兼任）| **Rust 1.98 + 需 Ulysses 实际跑过 cargo build + cargo test 全绿**（签字不构成证据，per RGS-EXEC-001 v0.3 §3.4）|
| **G-CODE-07** | QA Lead 具名 + 验收矩阵签字 | ✅ **Closed** | Ulysses（QA 一人公司兼任）| Ulysses 实际签 2026-08-21 + 验收矩阵签字 |

**当前汇总**（v0.3 per DEC-008）：✅ 5/7 Closed（Ulysses 实际签声明） + ⚠️ 2/7 待实测（G-CODE-03 5 独立 DB 拓扑图 + G-CODE-06 Rust 1.98 + CI 全绿）

---

## 2. 起動準備就绪清单（v0.3 新增 section）

> **本节列 53 起動的**前置准备**状态**。除 G-CODE-03/06 实测项外，其余均已就位。

### 2.1 文档就位

| 文档 | 状态 | 关联 |
|---|---|---|
| RGS-WBS-001 v0.3（瀑布式 WBS + 121 L4 任务 + §2A.6.7~§2A.6.10 横向规范）| ✅ 就位 | commit `5a717ae` + `a2ec295` + `dd960fa` |
| RGS-WBS-001_L4任务进度表 v0.3（占位）| ✅ 就位 | commit 待定（v0.3）|
| RGS-WBS-001_DAG v0.3（占位）| ✅ 就位 | commit 待定（v0.3）|
| RGS-WT-001 v0.2（WBS L4 任务 worktree 模式）| ✅ 就位 | commit 待定（v0.3）|
| 7 份 RGS-SPEC-CROSS-001~007 占位 | ✅ 就位 | commit `0f9af88`（v0.1）|
| 36 份 RGS-SPEC-DTL 占位 | ✅ 就位 | per `f198270`（v0.x）|
| RGS-EXEC-001 v0.3 + RGS-ENV-001 v0.3 + RGS-REV-003 v0.3 + RGS-PLAN-001 v0.8 + RGS-QA-001 v0.13（5 治理核心）| ✅ 就位 | per commits |

### 2.2 工具脚本就位

| 脚本 | 状态 | 作用 |
|---|---|---|
| `scripts/verify_docs.py` | ✅ 就位 | 文档 ID 唯一性 + 文档头 + 标题锚点 + 链接（**271 文件全 PASS**）|
| `scripts/check-cross-references.py` | ✅ 就位 | 跨文档章节号 + 文档 ID 引用校验（EXIT=True）|
| `scripts/verify_wf_v05.py` | ✅ 就位 | 150 工程编号 + V-model 配对校验 |
| `scripts/worktree.ps1` | ✅ 就位 | 通用 worktree 管理（create/list/doctor/remove）|
| `scripts/wbs_list.ps1` | ✅ 就位 | WBS L4 任务查询（识别 121 个 L4 任务）|
| `scripts/wbs_create_worktree.ps1` | ✅ 就位 | L4 worktree 创建 + `.wbs-task-marker` 写 |
| `scripts/wbs_task_progress.ps1` | ✅ 就位 | L4 进度追踪（start/progress/done/blocked）|
| `scripts/wbs_merge.ps1` | ✅ 就位 | L4 worktree 合并（自动跑 3 脚本验证）|
| `scripts/build_wbs_v02.py` | ✅ 就位 | 5 层 WBS + 2,048 L4 占位清单生成（历史 v0.2）|
| `scripts/build_wbs_dag.py` | 🟠 v0.4 待办 | DAG 依赖图自动生成 |
| `scripts/build_wbs_wf1.py` | 🟠 v0.4 待办 | 72 L4 生成器（v0.3 §2A.5 v0.4 待办）|
| 18+ Python 工具脚本 | ✅ 就位 | `add_nfr_index` / `commit_*` / `finalize_*` / `ipa_*` / `reorg_*` 等 |

### 2.3 部署目录就位

| 目录 | 状态 | 备注 |
|---|---|---|
| `../00-prerequisites/` | ✅ 已就位（5 文件）| NO-GO checklist + 环境核验 + 域 Lead 到位 + Rust + PG |
| `../01-k8s-manifests/` | ✅ 占位就位（13 文件）| 11 yaml + README + _status，全部 PLACEHOLDER_* |
| `../02-helm-charts/` | ✅ 占位就位（22 文件）| umbrella + 6 子 chart，全部 version 0.0.0 |
| `../03-db-migrations/` | ✅ 占位就位（13 文件）| 6 DB + 9 placeholder SQL |
| `../04-ci-cd/` | ✅ 占位就位（6 文件）| 4 workflow + README + _status |
| `../05-deploy-sop.md` | ✅ 已就位 | 详细部署步骤（NO-GO 状态保留）|
| `../06-rollback-sop.md` | ✅ 已就位 | L1-L4 回滚分级（NO-GO 状态保留）|
| `../07-no-go-checklist_v0.3.md` | ✅ 当前文件 | 顶层 summary（本文件）|

---

## 3. 環境核验状态（per RGS-ENV-001 v0.3 §1-§5）

> **0/12 类环境核验全部通过**——实测需 Ulysses 实际跑过才能 ✅

| 类别 | 状态 |
|---|---|
| §1 Rust 1.98 安装 | 🟠 部分满足（GA 已发，待 CI 验证 = G-CODE-06） |
| §2 PG 18.6 5 独立 DB | 🟠 未启动（DBA 待具名） |
| §3 K8s 集群 | 🟠 未启动（SRE 待具名） |
| §4 QUIC 证书 | 🟠 未启动（SRE 待具名） |
| §5 OTel 链路 | 🟠 未启动（Platform 架构师待具名） |

---

## 4. 12 类签字栏状态（per RGS-ENV-001 v0.3 §6 + RGS-REV-003 §7.3 + RGS-PLAN-001 v0.8 §3.4.4）

| # | 签字栏 | 实际签字人 | 状态 |
|---|---|---|---|
| 1 | DBA | Ulysses（一人公司 12 角色兼任）| ✅ 实际签 2026-08-21 |
| 2 | SRE | Ulysses（一人公司 12 角色兼任）| ✅ 实际签 2026-08-21 |
| 3 | player 域 Lead | Ulysses | ✅ |
| 4 | economy 域 Lead | Ulysses | ✅ |
| 5 | match 域 Lead | Ulysses | ✅ |
| 6 | social 域 Lead | Ulysses | ✅ |
| 7 | admin 域 Lead | Ulysses | ✅ |
| 8 | 架构师 | Ulysses | ✅ |
| 9 | Platform 架构师 | Ulysses | ✅ |
| 10 | QA Lead | Ulysses | ✅ |
| 11 | 业务方代表 | Ulysses | ✅ |
| 12 | PM | Ulysses | ✅ |

**12/12 Ulysses 实际签**（per DEC-008 一人公司 1 人 12 角色兼任 = 真实人真实职责，不构成"伪造"）。

---

## 5. NO-GO 解除条件（**最终条件**）

本表所有 🟠 转为 ✅ 后，由架构师出 v0.4 删除"NO-GO"占位：

1. **G-CODE-03 Ulysses 实际画 5 独立 DB 拓扑图**（§1 中 1 项）⚠️
2. **G-CODE-06 Ulysses 实际跑过 Rust 1.98 + cargo build + cargo test 全绿**（§1 中 1 项）⚠️
3. **RGS-ENV-001 v0.3 §1-§5 12 类环境核验 0/12 → 12/12**（实测）🟠
4. **升 v0.4** 标记 GO

满足后由架构师 + PM 联合出 v0.4 删除"NO-GO"占位 → `../05-deploy-sop.md` 激活 → 53 開発環境構築 启动。

**两项实测最小 SOP**（Ulysses 可在 ~30-60 分钟完成）：

### 5.1 G-CODE-03 实测（5 独立 DB 拓扑图）

工具：draw.io / Excalidraw / Mermaid / 手画 PNG 均可

要求：
- 5 个独立 DB 框（player_db / economy_db / match_db / social_db / admin_db）+ cluster_ops_db（共 6 个）
- 每个 DB 标注端口（默认 5432）+ Schema 命名（per RGS-SPEC-CROSS-005）
- 跨 DB 访问箭头（标"禁止 JOIN"）
- Outbox + CEM 跨域协调路径
- 提交到 `docs/deploy/05-db-topology.png`（或 .svg / .drawio）

### 5.2 G-CODE-06 实测（Rust 1.98 + CI 全绿）

```bash
# 1. 装 Rust 1.98 stable
rustup toolchain install 1.98 --profile minimal
rustup default 1.98

# 2. 验证
rustc --version  # 期望: rustc 1.98.0
cargo --version  # 期望: cargo 1.98.0

# 3. 写最小 workspace（per RGS-IMPL-001 §2）
mkdir -p crates/rgs-hello
# 写 Cargo.toml + src/main.rs 最小 "Hello world"

# 4. 编译 + 测试
cargo build --locked  # 期望: 编译成功
cargo test --locked   # 期望: 测试通过
```

提交到 `docs/deploy/06-rust-198-build.log`（命令输出）。

### 5.3 12 类环境核验（per RGS-ENV-001 v0.3 §1-§5）

按 RGS-ENV-001 v0.3 §1.1-§5.3 顺序跑完 63 个 checkbox，输出到 `docs/deploy/07-env-verification.log`。

---

## 6. 关联文档

- 详细 NO-GO checklist：`../00-prerequisites/00-no-go-checklist_v0.2.md`
- 部署 SOP：`../05-deploy-sop.md`
- 回滚 SOP：`../06-rollback-sop.md`
- 治理：`RGS-PLAN-001 v0.8 §3.3` + `RGS-ENV-001 v0.3 §6` + `RGS-EXEC-001 v0.3 §3.4/§4.4`
- 评审：`RGS-REV-003 §7.3`（12 类签字栏）
- 决策：`DEC-005`（5 域 Lead 独立） + `DEC-006`（路径 B 14-18 周） + `DEC-007`（OLU 双轨制） + `DEC-008`（一人公司）
- 架构：`RGS-ARC-051`（COC/CEM/PFAU） + `RGS-ADR-0052`（Active-Active）
- WBS：`RGS-WBS-001 v0.3` + `RGS-WBS-001_L4任务进度表 v0.3` + `RGS-WBS-001_DAG v0.3`
- Worktree：`RGS-WT-001 v0.2 §11` + `scripts/wbs_*.ps1` × 4
