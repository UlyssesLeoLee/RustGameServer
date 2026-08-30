# 孤儿 SPEC 自查清单

**RGS-ANTIPATTERN-001**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-ANTIPATTERN-001 |
| 版本 | 0.1（初版，per 主对话 2026-08-25 11:52~12:01 SPEC 实现盘点延伸）|
| 状态 | 草案；待 5 域 Lead 复审 + SRE Lead 接入 PM 签字流程后升 v0.2 |
| 制定日 | 2026-08-25 |
| 制定者 | Ulysses（一人公司 12 角色兼任 per DEC-008）|
| 适用范围 | 全部 42 份 RGS-SPEC-DTL-XXX（特别是 v0.2 升版时的断点续传 / 服务器全生命周期一类）|
| 关联 | RGS-WBS-001 v0.3 §8.3 anti-pattern + §17.3 双轨签字 + RGS-SPEC-000 §4 总表 + RGS-WF-001 v0.6 G-CODE-06 |
| 真源 | 文档与代码状态必须可被 git + 文件系统独立核验 |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-25 | Ulysses | 首版：定义"孤儿 SPEC" anti-pattern + 8 步自查程序 + 当前 2 个孤儿（SPEC-DTL-041/042）+ 4 路处置决策树 + 与 WBS §8.3 anti-pattern 的边界 |

---

## 1. 目的与适用场景

本清单识别并治理一类**结构性反模式**：

> **"SPEC 文档已签字 + WBS L4 任务已标 done，但代码层无对应实现产物（crate/模块/文件）"**

这与 WBS v0.7 §8.3 第 3 条 anti-pattern（「已合并进 main」与「任务实质完成」混为一谈）**同源**，但发生在**文档治理签字 → 代码实现**的边界，而不是 git 合并 → 任务完成的边界。

### 1.1 适用时机

强制触发本清单的 4 种场景：

| # | 触发 | 入口 | 输出 |
|---|---|---|---|
| T1 | SPEC 升 v0.X（含 v0.1→v0.2 的"集体签字(per DEC-008)"追加）| `git log` 含 `RGS-SPEC-* v*` | 自查表 + 受影响 L4 任务 |
| T2 | WBS §16.1 类批量签字落盘 | WBS 修改 + `git log` 含 v0.4 增量 | 受影响 SPEC + 代码层差异 |
| T3 | SRE 接力前 / G-CODE-06 实测前 | `docs/deploy/phase-*-handoff.md` | 受影响 SPEC 列表 |
| T4 | PM 复审 / 一人公司 12 角色周会 | `RGS-PM-007` 评审节奏 | 孤儿清单 + 处置决议 |

### 1.2 不适用场景

- WBS L4 任务明确标 `pending`（无签字的 SPEC）→ 走常规 WBS 进度跟踪
- 纯文档任务（REQ/BAS/DTL 而非 SPEC）→ 本清单不覆盖（文档任务本身即交付物）
- ADR 决策记录类（RGS-ADR-*）→ 走 RGS-DEC-NOGO 流程

---

## 2. 孤儿 SPEC 的精确定义

### 2.1 双轨签字

RGS-WBS-001 v0.3 §17.3 已明确双轨签字原则（**L985**）：

> "**生产基线仍需 G-CODE-06 实测通过**……本次'具名审批完成'仅代表**文档治理闭环 v0.1→v0.2**，**不**代表代码/部署已完成。代码/部署的 G-CODE-06 实测是**独立**的下一道门禁。"

任何 SPEC 必须**同时**满足两轨才算"实质完成"：

| 轨 | 满足条件 | 证据位置 |
|---|---|---|
| **轨 1：文档治理签字** | WBS §16.1/§17.2 标 `done` + SPEC 含「集体签字(per DEC-008)」行 | WBS L883/L890（断点续传/全生命周期）+ SPEC frontmatter 审批栏 |
| **轨 2：代码实现产物** | SPEC §2 实现单元表里的所有「计划路径」都有 fs 可访问的实际文件 | `crates/<name>/` 或 `src/<name>/` 存在，且至少 1 个 .rs 文件非空 |

**孤儿** = **轨 1 满足 ∧ 轨 2 不满足**

### 2.2 不属于孤儿的 3 种情况（避免误判）

| 情况 | 判定 | 例子 |
|---|---|---|
| 轨 1 不满足、轨 2 不满足 | 正常 pending | DTL-001~009、011~027、031~040、043/044、100~102 全部 40 份 |
| 轨 1 满足、轨 2 满足 | 实质 done（少见）| 无（当前 0 份）|
| 轨 1 不满足、轨 2 满足 | **反向孤儿**（代码先于文档，需补文档）| WF-1-54.1~54.15 实施产物 + WBS 仍标"未启动"（WBS L89~L103） |

> **反向孤儿**也是 anti-pattern，但治理路径与孤儿不同（补文档而非补代码）。本清单 v0.1 只覆盖孤儿，反向孤儿另出 RGS-ANTIPATTERN-002。

---

## 3. 8 步自查程序

### Step 1：列出所有"轨 1 满足"的 SPEC

```bash
# 在 docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md 中
# 搜所有含 RGS-SPEC-DTL-* + done 的行
node -e "const fs=require('fs'); const c=fs.readFileSync('docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md','utf8'); const lines=c.split('\n'); for(let i=0;i<lines.length;i++){if(lines[i].match(/SPEC-DTL-/)&&lines[i].includes('done'))console.log('L'+(i+1)+': '+lines[i].slice(0,250));}"
```

**期望输出**（v0.1 基线，2026-08-25）：

```
L883: | 2052 | PH-0.5 | 第 1-2 周 | shared-platform | CDN 资源分发扩展 | 客户端断点续传 实现规格 | 架构师（兼） | **done** | RGS-SPEC-DTL-041 |
L890: | 2059 | PH-0.5 | 第 1-2 周 | admin | LCM 全生命周期 | 服务器全生命周期 实现规格 | Admin 域 Lead（独立） | **done** | RGS-SPEC-DTL-042 |
```

### Step 2：提取每份 SPEC 的"计划路径"

读 SPEC §2「实现单元」表，提取所有"计划路径"列：

```bash
# 取 SPEC-DTL-041 §2
node -e "const c=require('fs').readFileSync('docs/13-实现规格/RGS-SPEC-DTL-041_实现规格书.md','utf8'); const sec2=c.split('## 2.')[1]?.split('## 3.')[0]||''; console.log(sec2);"
```

### Step 3：逐路径文件系统核验

对每条计划路径，**必须**独立验证：

```bash
# 例：SPEC-DTL-041 §2 提到的路径
# - crates/rgs-asset-download（独立 crate）
# - rgs-asset-download 公开 API
Test-Path -Path D:/RustGameServer/crates/rgs-asset-download
# 期望：False（v0.1 基线 2026-08-25 实测）

# SPEC-DTL-042 §2 提到的路径
# - crates/rgs-cluster-ops/src/realm_lifecycle/
Test-Path -Path D:/RustGameServer/crates/cluster-ops/src/realm_lifecycle
# 期望：False（v0.1 基线 2026-08-25 实测）
```

### Step 4：判断孤儿

| 轨 1 | 轨 2 | 判定 |
|---|---|---|
| ✅（WBS done + 集体签字）| ❌（无 fs 路径）| **🔴 孤儿** |
| ✅ | ✅ | 🟢 实质 done |
| ❌ | ❌ | ⚪ pending（正常）|
| ❌ | ✅ | 🔵 反向孤儿（不在本清单范围）|

### Step 5：孤儿证据归档

为每个孤儿生成**最小可复现证据块**（参照 phase-0-5 反馈单风格）：

```markdown
## 孤儿 #N: SPEC-DTL-XXX（<主题>）

- **WBS 轨 1 证据**：`docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md:LINE`
- **SPEC frontmatter 状态**：`docs/13-实现规格/RGS-SPEC-DTL-XXX_实现规格书.md:STATUS_LINE`
- **§2 计划路径**：
  - `<path-1>`
  - `<path-2>`
- **fs 核验（YYYY-MM-DD）**：
  - `Test-Path <path-1>` → False
  - `Test-Path <path-2>` → False
- **冲突**：轨 1 满足 ∧ 轨 2 不满足 → 🔴 孤儿
- **建议处置**：见 §4 决策树
```

### Step 6：登记到孤儿清单（§5）

把证据块追加到本清单 §5「当前孤儿名单」。

### Step 7：触发处置决策（§4）

对每个孤儿，走 §4 4 路决策树，落到 1 个具体动作 + 1 个 owner + 1 个 deadline。

### Step 8：回填到 WBS §16.2

将每个孤儿的实施 L4 任务（如尚未在 WBS §16.2 中）补登，状态 = `pending`，owner = SPEC §3 实施契约里的域 Lead（默认即 WBS 标的所有者）。

---

## 4. 4 路处置决策树

```
孤儿 #N
  │
  ├─ Q1: 实施 L4 任务是否已在 WBS §16.2 显式登记？
  │   ├─ 是  → 检查 L4 任务是否真"pending"还是被错标"done"
  │   │         ├─ 真 pending → 维持现状（孤儿已知 + 实施有计划）
  │   │         │                决议：维持签字 + 等实施完成 + 此期间不撤销签字
  │   │         └─ 被错标 done → 立刻回退 WBS 标 done → pending
  │   │                     决议：撤销签字（轨 1 撤回）→ 等实施完成重新签字
  │   └─ 否  → Q2
  │
  ├─ Q2: SPEC §2 计划路径对应的实施工作量 ≤ 1 周？
  │   ├─ 是  → Q3
  │   └─ 否  → Q4
  │
  ├─ Q3: 现在是否有空跑该实施？（per PM-007 排期）
  │   ├─ 是  → 决议：直接实施（追加 L4 任务进 WBS §16.2 + 排期）
  │   └─ 否  → 决议：维持签字 + 显式登记"文档先行"标记（见 §4.1）+ 排到下一 PH
  │
  └─ Q4: 实施需要跨域协调？（≥ 2 个域 Lead）
      ├─ 是  → 决议：维持签字 + 触发跨域协调会 + 显式排期 + 设立阻塞风险登记
      └─ 否  → 决议：维持签字 + 显式排期到特定 PH（如 PH-3/4/5）+ 周会追踪
```

### 4.1 "文档先行"标记语法

对决议"维持签字 + 等实施"的情况，**必须**在 WBS §16.1 / §17.2 对应行**追加尾注**：

```markdown
| 2052 | PH-0.5 | … | **done** | RGS-SPEC-DTL-041 | ⚠️ 文档先行（轨 1 / 轨 2 pending；实施 L4 #2063 启动后撤销本注）|
```

- 维持现状不撤销签字 = 文档治理闭环仍有效（v0.1→v0.2 升版本身是正确动作）
- 不隐瞒轨 2 缺口 = 防止"已签字即等同于已实现"的误读
- 实施 L4 启动时撤销 = 闭环回到正常 WBS 流程

### 4.2 反向：撤销签字

仅在以下任一情况触发：

1. SPEC 计划路径本身错误（不是"未实施"而是"路径规划错"）
2. 实施工作量被严重低估，签字时已不可行
3. 一人公司 12 角色兼任签字被复审发现错误（如把"集体签字"误用在不该用的文档）

撤销动作 = 改 WBS §16.1/§17.2 该行 `**done**` → `pending`，**同时**改 SPEC frontmatter 状态行加 `[撤销 @ YYYY-MM-DD 理由：…]`。

---

## 5. 当前孤儿名单（v0.1 基线，2026-08-25 实测）

### 5.1 孤儿 #1: SPEC-DTL-041（CDN 资源分发扩展 · 客户端断点续传）

| 项 | 内容 |
|---|---|
| WBS 轨 1 证据 | `docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md:883` 标 `**done**` + WBS §17.2 L966 标"新增整段 ✓" + "集体签字(per DEC-008)" |
| WBS 实施计划 | WBS §16.2 L899 L4 #2063（rgs-asset-download crate 骨架，PH-3）= **pending**；后续 #2064~2065、#2069、#2072 共 5 项 pending |
| SPEC frontmatter 状态 | `docs/13-实现规格/RGS-SPEC-DTL-041_实现规格书.md:9` 仍写"规格草案，待 RGS-DTL-041 具名 DD Review"（**表/文件状态自相矛盾**）|
| SPEC §2 计划路径 | `crates/rgs-asset-download/`（独立 crate）|
| fs 核验（2026-08-25）| `Test-Path D:/RustGameServer/crates/rgs-asset-download` → **False** |
| 全局搜索核验 | `glob crates/**/rgs-asset-*` → No files matched |
| 冲突 | 轨 1 满足 ∧ 轨 2 不满足 → **🔴 孤儿** |
| 建议处置 | §4 决策树：Q1=是 + 真 pending → **维持签字 + 显式登记"文档先行"标记**（追加于 WBS L883 尾）+ 等 L4 #2063~2065/#2069/#2072 实施完成 |

### 5.2 孤儿 #2: SPEC-DTL-042（LCM 全生命周期 · 服务器全生命周期）

| 项 | 内容 |
|---|---|
| WBS 轨 1 证据 | `docs/12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md:890` 标 `**done**` + WBS §17.2 L973 标"新增整段 ✓" + "集体签字(per DEC-008)" |
| WBS 实施计划 | WBS §16.2 L902~L910 L4 #2066~#2074 共 9 项 pending（PH-3~PH-6） |
| SPEC frontmatter 状态 | `docs/13-实现规格/RGS-SPEC-DTL-042_实现规格书.md:9` 仍写"规格草案，待 RGS-DTL-042 具名 DD Review"（**表/文件状态自相矛盾**）|
| SPEC §2 计划路径 | `crates/rgs-cluster-ops/src/realm_lifecycle/`（子模块）|
| fs 核验（2026-08-25）| `Test-Path D:/RustGameServer/crates/cluster-ops/src/realm_lifecycle` → **False** |
| 全局搜索核验 | `glob crates/**/realm_lifecycle/**` → No files matched |
| 冲突 | 轨 1 满足 ∧ 轨 2 不满足 → **🔴 孤儿** |
| 建议处置 | §4 决策树：Q1=是 + 真 pending → **维持签字 + 显式登记"文档先行"标记**（追加于 WBS L890 尾）+ 等 L4 #2066~#2074 实施完成（PH-3~PH-6 跨度 6 周）|

### 5.3 已通过本清单的孤儿（治理后状态）

> **v0.1 基线：0 份**（孤儿仍为 pending 处置）

### 5.4 跨清单基线（不入孤儿，但本清单辅助识别）

- **反向孤儿（蓝）**：WF-1-54.1~54.15 在 WBS L89~L103 仍标"未启动"，但代码层已合入 main（per git log `fb73286` `a37c0e1` `a556015` 等 8 个 merge commit）→ **不归本清单**，待 RGS-ANTIPATTERN-002 覆盖
- **WBS 进度表与 git 现实脱节（per WBS v0.7 §8.3 anti-pattern #1）**：v0.7 进度表只列 5 done，git 实际 53.x/54.x/55.1~55.37 都已合 → **已在 WBS §8.3 覆盖**，本清单不重复

---

## 6. 与现有 anti-pattern 文档的边界

| 现有 anti-pattern | 范围 | 与本清单的关系 |
|---|---|---|
| WBS v0.7 §8.3 #1 | 手工改进度表写 "done" 不跑脚本 | 不重叠：本清单管"SPEC vs 代码"，§8.3 #1 管"WBS 进度表 vs marker JSON" |
| WBS v0.7 §8.3 #2 | 攒到「后续 Phase」再补 | **强相关**：本清单的 §4.1"文档先行"标记正是 #2 的**正面替代**——显式登记"在哪一 PH 补"而非无限期"后续 Phase" |
| WBS v0.7 §8.3 #3 | "已合并进 main" 但 "任务实质未完成" | **同源不同面**：#3 管 git merge 边界；本清单管文档签字边界。**两者均根植于一人公司兼任的"快进"倾向** |
| WBS v0.7 §8.3 #4 | `status: done` 但 `progress < 100%` | 不重叠：marker 状态机问题 |
| WBS v0.7 §8.5 | B-CODE/C-CODE log 强制验证证据模板 | 互补：§8.5 管 G-CODE/B-CODE/C-CODE log 形式；本清单管"SPEC 文档 vs 代码 fs"核验 |
| phase-0-5 反馈单 §1-§5 | 主对话 → 后续 agent 的问题反馈 | 互补：反馈单是"事件流"；本清单是"结构治理" |

---

## 7. Definition of Done（本清单自身的 DoD）

- [ ] T1~T4 4 种触发时机各跑过一次 Step 1~8（v0.1 已完成 T1 触发，2026-08-25）
- [ ] §5 孤儿名单每条含 8 个最小证据字段（WBS 轨 1 / 实施计划 / SPEC frontmatter / §2 路径 / fs 核验 / 全局搜索 / 冲突判定 / 建议处置）
- [ ] 每次新 SPEC 升 v0.X 时，本清单 §5 必须重跑 Step 1~8
- [ ] 本清单不替代 WBS §8 anti-pattern（不重复 §8.3 #1/#2/#4）
- [ ] 一人公司 12 角色兼任复审后升 v0.2（PM 角色 + SRE Lead 角色 + 架构师角色各 1 签）
- [ ] 治理完成后，孤儿 #1/#2 至少有一条进 §5.3「已通过本清单的孤儿」

---

## 8. 上行 / 下行

### 8.1 上行（依赖）

- RGS-WBS-001 v0.3 §16.1/§17.2（签字落盘点）
- RGS-SPEC-000 v0.2 §4 总表（SPEC → DTL → 状态）
- RGS-WF-001 v0.6 G-CODE-06（生产基线门禁）
- DEC-008 一人公司 12 角色兼任
- 2026-08-25 主对话盘点（per session mvs_be4f492991d9424e98123cd1752e8168）

### 8.2 下行（被依赖）

- RGS-ANTIPATTERN-002（反向孤儿清单，待起草）
- RGS-PM-007 周会节奏（孤儿进度追踪）
- RGS-QA-001 v0.13 复审 checklist（建议加 1 条："是否本清单 §5 为空"）

---

## 9. 元数据

- **本清单创建于**：2026-08-25 12:01 JST
- **触发原因**：主对话盘点"哪些 SPEC 没实现"时发现 WBS 标 done 但代码层无产物的 2 份 SPEC（041/042），需治理
- **下一次 review**：2026-08-25 EOD 主对话 / 或 5 域 Lead 周会
- **owner**：Ulysses（PM 角色兼 + 架构师角色兼 per DEC-008）
- **预期升 v0.2 时机**：5 域 Lead 复审通过 + 至少 1 次 T1 触发后追加证据归档

---

> **本清单是 living document**。每跑一次 Step 1~8 必追加 1 行到 §5。每次发现新 anti-pattern 模式时，在 §6 加 1 行；不删除既有 anti-pattern。
