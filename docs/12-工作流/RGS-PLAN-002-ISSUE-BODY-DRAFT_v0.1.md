# RGS-PLAN-002 1-2 周层 GitHub Issue Body Draft v0.1

> **文档编号**：RGS-PLAN-002-ISSUE-BODY-DRAFT
> **版本**：0.1
> **日期**：2026-08-26
> **父文档**：[RGS-PLAN-002 v0.1 §5.1](RGS-PLAN-002_后续工作_2026-08-25_v0.1.md)
> **状态**：🟡 草案（待 Ulysses 终审）
> **本任务定位**：为 GitHub issue #8 ~ #19 起草 body 草稿，**不**实际调 `gh issue create`。
> **目录偏差说明**：父文档实际位于 `docs/12-工作流/`，本草稿同目录落盘以保持路径一致。

---

## §0 文档目的

本文是 12 个 GitHub issue（#8 ~ #19）的 body 草稿集合，对应 RGS-PLAN-002 v0.1 §5.1 表格。

**硬约束**（per 主对话 2026-08-26 22:17 JST 派发要求）：
1. **禁止编造未做过的 commit hash** —— 如不可证，写"待补"
2. **不沿用"per X 历史形态"回溯叙事** —— 所有 commit 引用基于**当前可实证**记录
3. **每个 issue body 含"per Ulysses 拍板" + "Mavis 起草"占位** —— 草稿状态明示

**每个 issue body 字段**：
- 标题（per §5.1 表格）
- 任务范围（per 父文档 §1.1 / §1.2 / §5.1）
- 估算 token（per 父文档 §1.1 / §5.1 表格 + RGS-TS-001 §6.2 token-OLU 框架）
- 关联 WBS / 文档
- 阻塞
- 关联 commit 引用（**已实证 / 待补** 两栏）
- "per Ulysses 拍板" + "Mavis 起草" 占位

---

## §1 4 主线 issue body（#8-#11）

### Issue #8 — [M1] PH-1 启动 Gate 收尾

- **类型**：主线 / `enhancement`
- **估算 token**：~800K（per RGS-PLAN-002 §1.1 M1）
- **任务范围**（per 父文档 §1.1）：
  - 补 WBS-001 §3 "PH-1 启动就绪"过渡到实际开工
  - 补 RGS-OPEN-QA-001 已答复的 13 个 L4 任务（WF-1-55.38~50）的 WBS-001 L4 任务进度表
  - 修订历史 v0.8 升版
  - 更新 SPEC-000 总表至 v0.3（含 LCM-001/CDN-001 IMPL-PLAN v0.1 已落地的双向引用）
- **关联 WBS / 文档**：
  - [RGS-WBS-001 L4 任务进度表](../12-工作流/RGS-WBS-001_L4任务进度表_v0.4.md)（head 实际 v0.11，**已知缺口**：本任务无 v0.8 升版的实证，引用父文档 §1.1 M1 原文）
  - SPEC-000 总表
  - RGS-OPEN-QA-001 v0.2
- **阻塞**：无（独立任务）
- **关联 commit 引用**：
  - 已实证：**待补**（本任务未跑 `git log` 实证 WBS-001 v0.8 升版 commit）
  - 父文档 §1.1 M1 描述**未**列具体 commit 链
- **per Ulysses 拍板** + **Mavis 起草**：本 issue body 草稿 = Mavis 起草（per 主对话 2026-08-26 22:17 JST 派发）；issue 启动 / 关闭 / 排期 = **等 Ulysses 拍板**。

---

### Issue #9 — [M2] 5 域 DTL 同步起草入口

- **类型**：主线 / `enhancement`
- **估算 token**：3-5M（5 域 × 600-1000K/域，**草案级**，per RGS-PLAN-002 §1.1 M2）
- **任务范围**（per 父文档 §1.1）：
  - per RGS-OPEN-QA-001 v0.2 + REV-011 §1 "5 域 DTL §1-§3 预对齐"
  - 5 域 = player / economy / match / social / admin
  - 各域 §1 适用范围 + §2 实现单元 + §3 实现契约 三段必须先冻结
  - 后续 §4-§8 引用此冻结
- **Mavis 提建议域顺序（待 Ulysses 拍板）**：player → economy（已有 DTL-037 反向文档基线）→ match（已有 DTL-026 §7 + DTL-015/016 Saga）→ social → admin（依赖前 4 域的 CEM 事件）
- **关联 WBS / 文档**：
  - RGS-WBS-001 §4.1-§4.5（5 域 PH-1 模板）
  - RGS-OPEN-QA-001 v0.2
  - REV-011 §1
  - DTL-037 v0.2 §7（economy 反向 DDL）
  - DTL-026 §7（match 撮合）+ DTL-015/016（跨 DB Saga）
  - DTL-001 §2.1（admin RBAC 矩阵）
- **阻塞**：
  - 5 域 Lead 各自签字 per DEC-005
  - ARC-014 + ARC-026 评审门禁（5 域 DTL 同步起草前必须具名）
- **关联 commit 引用**：
  - 已实证：**待补**（5 域 DTL 同步起草是未来工作）
  - 父文档 §1.1 M2 描述**未**列具体 commit 链
- **per Ulysses 拍板** + **Mavis 起草**：域顺序 / 工作量校准 / 启动日 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

### Issue #10 — [M3] WF-1-2xxx LCM/CDN 切流 + NFR-LCM 实测

- **类型**：主线 / `enhancement`
- **估算 token**：~1.5M（per RGS-PLAN-002 §1.1 M3）
- **任务范围**（per 父文档 §1.1）：
  - 已在 main（per git log 父文档引用的 commit 链），进入"切流 + SRE 接力"阶段
  - M-2063.1~5 / M-2064.1~7 / M-2065.1~12 / M-2066.1~10 / M-2067.1~6 / M-2068.1~7 / M-2069.1~10 / M-2070.1~14 / M-2071.1~7 / M-2072.1~4 / M-2073.1~6 / M-2074.1~5
  - **追加**：NFR-LCM-001/004/006 3 项 NFR 实测
  - **追加**：RSK-LCM-001/005 2 项风险缓解验证（per IMPL-PLAN-LCM-001 §8.6 验收 checklist）
- **关联 WBS / 文档**：
  - WBS-001 v0.7 L4 #2063~#2074（per 父文档 §5.1）
  - RGS-IMPL-PLAN-LCM-001 §8.6
  - RGS-OPS-001 §6/§7（实操演练报告）
- **阻塞**：
  - RGS-OPS-001 §6/§7 实操演练报告（ISS-117 待具名审批）
- **关联 commit 引用**：
  - 父文档 §1.1 M3 引用 commit 链 `71c71bb/8d55fbd/badae2a/354f768/0cae9cc/eac8f31 + 6a6c020/9ad773b/5f7da00/22ac71bb`
  - 已实证：**待补**（本任务未跑 `git log` 实证 12 个 commit 是否真实存在；**注**：Mavis 不沿用"per commit X 历史形态"叙事）
  - **建议**：Phase D 实施前由 Ulysses 拍板的执行人跑 `git log -p --follow` 实证 12 个 commit
- **per Ulysses 拍板** + **Mavis 起草**：LCM/CDN 切流顺序 / SRE 接力排期 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

### Issue #11 — [M4] 主对话退场后"接收-恢复"工具链

- **类型**：主线 / `good first issue, documentation`
- **估算 token**：~300K（per RGS-PLAN-002 §1.1 M4）
- **任务范围**（per 父文档 §1.1）：
  - main 之前存在 `codex/wt-*` 兄弟 worktree 残留（per git log "On main: REQ-001/005/007-ADD1/038 + worktrees 残留"）
  - `wf-1-2xxx` 已 merged 但 `.wbs-task-marker` 状态在 worktree 删除后丢失
  - **本任务动作**：
    - 跑 `wbs_list.ps1 -Summary` 重建 v0.8 进度表
    - `git worktree list --porcelain` 清理孤儿
    - 验证 wbs_*.ps1 仍可工作
- **关联 WBS / 文档**：
  - RGS-WBS-001 v0.7 §3（父文档 §1.1 M4 引用）
  - RGS-WT-001 v0.2 §11（WBS L4 任务 worktree 模式，per 父文档 §5.1）
  - scripts/wbs_*.ps1（wbs_list.ps1 / wbs_create_worktree.ps1 / wbs_task_progress.ps1 / wbs_merge.ps1）
- **阻塞**：无（独立任务）
- **关联 commit 引用**：
  - 已实证：**待补**（`codex/wt-*` worktree 残留的具体 commit 链**未**在父文档列出）
  - **已知缺口**：WBS-001 L4 任务进度表 head 实际 v0.11（per v0.11 修订历史），父文档 §1.1 M4 写"v0.8 进度表"—— v0.8 → v0.11 之间的 3 次升版（v0.9 / v0.9 sync / v0.10 / v0.11）由 Ulysses 17:04 JST 批次 + Ulysses 18:21 JST 批次 + Ulysses 19:05 JST 批次 + 主对话 21:01-21:02 JST 实测推动（per WBS-001 v0.9 / v0.9 sync / v0.10 / v0.11 修订历史）
- **per Ulysses 拍板** + **Mavis 起草**：进度表 v0.8 → v0.11 升版历史属于**已发生**事实，**不**回填"per v0.7 历史形态"叙事；本 issue body 草稿 = Mavis 起草。

---

## §2 4 治理 issue body（#12-#15）

### Issue #12 — [G1] DEC-NOGO-001 文档头格式修复

- **类型**：治理·阻塞 / `documentation`
- **估算 token**：起草 ~30K（per RGS-PLAN-002 §5.1）
- **任务范围**（per 父文档 §1.2 G1）：
  - 文档头表格字段为 `| 文档 ID |`（中间空格 + `ID`），与脚本 `DOCUMENT_ID_FIELD_PATTERN` 正则不兼容
  - 脚本不识别"文档 + 空格 + ID"作为字段名
  - check-docs-consistency.sh §5 第 5 项 FAIL 1
- **Mavis 起草修复方案**（待 Ulysses 拍板）：
  - 照 §3 模式：补 `| 决策编号 |` 表格行
  - 保留 `| 文档 ID |` 兼容性
- **关联 WBS / 文档**：
  - RGS-DEC-NOGO-001 v0.1
  - scripts/check-docs-consistency.sh
- **Mavis 处置边界**：**不改**——不在 `RGS-DOCS-HEALTH-2026-08-25` 反馈单 §3 范围内（per 反馈单 §3' 段标注）
- **阻塞**：
  - Ulysses 拍板"接受 Mavis 起草的修复方案"或"指定其他方案"
- **关联 commit 引用**：不适用（治理·阻塞，无 commit）
- **per Ulysses 拍板** + **Mavis 起草**：方案选择 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

### Issue #13 — [G2] 5 个 ADR 签字 + 附件D §3 状态升级

- **类型**：治理·阻塞 / `documentation`
- **估算 token**：不计（Ulysses 签字，per 父文档 §5.1）
- **任务范围**（per 父文档 §1.2 G2）：
  - 5 个 ADR：ADR-0052 / 0053 / 0054（签字栏已空）+ ADR-0058 / 0056（候选正文已起草）+ 既有 ADR-0057（已 Accepted）
  - 附件D §3 行的"未制定"/"待具名人类审批"字样保持不变（签字前）
- **Mavis 处置边界**：**不签字**（per DEC-008 一人公司治理基线 + 反馈单 §0 第 4 行"治理状态，非文档缺陷，agent 不可代签" + ADR-0058/0056 §6 "agent 不得代签事实清单"）
- **关联 WBS / 文档**：
  - RGS-ADR-0052 / 0053 / 0054 / 0058 / 0056 / 0057
  - 附件D §3
- **阻塞**：
  - Ulysses 本人在场 12 角色逐项勾选
  - 附件D §3 行的"未制定"升级为"已制定"——Mavis **不**代签，**不**改附件D §3 状态列
- **关联 commit 引用**：不适用（治理·阻塞，无 commit）
- **per Ulysses 拍板** + **Mavis 起草**：12 角色勾选 = **等 Ulysses 本人在场**；本 issue body 草稿 = Mavis 起草（**注**：Mavis 起草 ≠ Mavis 代签，issue body 不预设 ✅ / 不填签字栏 / 不预承诺截止日期，per 父文档 §5.1 末段第 2 句）。

---

### Issue #14 — [G3] ISS-126 编号漂移后续 verify

- **类型**：治理·阻塞 / `documentation`
- **估算 token**：~50K（per 父文档 §5.1）
- **任务范围**（per 父文档 §1.2 G3）：
  - ISS-126 已决议（ADR-0055 编号漂移到 ADR-0058）
  - 附件D §3 表 ADR-0055 行的"实际归档文件" vs "ARC-055 提案" 双主题分立
  - **§3 表本身**已同步更正
  - **本任务动作**：
    - 验证附件D §3 表
    - docs/08-架构决策记录 目录扫描
    - 反向引用（RGS-REQ-034 / RGS-OPEN-QA-001 v0.2）3 处全部一致
- **Mavis 处置边界**：跑一遍 verify 脚本 + 列出不一致点清单
- **关联 WBS / 文档**：
  - ISS-126
  - 附件D §3
  - RGS-ADR-0055 / 0058
  - RGS-REQ-034
  - RGS-OPEN-QA-001 v0.2
- **阻塞**：
  - Ulysses 确认清单无遗漏 / 或追加修正项
- **关联 commit 引用**：不适用（治理·阻塞，无 commit）
- **per Ulysses 拍板** + **Mavis 起草**：清单确认 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

### Issue #15 — [G4] check-docs-consistency.sh 接入 CI

- **类型**：治理·阻塞 / `enhancement`
- **估算 token**：~200K（per 父文档 §5.1）
- **任务范围**（per 父文档 §1.2 G4）：
  - 当前 4 类检查（ARC 来源 / ADR 登记 / 域内 ID / README 死链 / 跨文档引用）每 PH 阶段至少跑 1 次
  - **本任务动作**：
    - 把 `check-docs-consistency.sh` 接入 GitHub Actions（CI）
    - 确保未来 24 个 RSK/TBD 这种"正文已用但未登记附件D"的治理缺口不再回潮
- **Mavis 起草 CI workflow YAML**（基于 RGS-BAS-010 G-011 TBD-PAT-001）
- **关联 WBS / 文档**：
  - RGS-BAS-010 G-011 TBD-PAT-001
  - scripts/check-docs-consistency.sh
- **Mavis 处置边界**：起草 CI workflow YAML；**不**实际接入 CI
- **阻塞**：
  - Ulysses 拍板"加 CI 阻断门禁" 或 "软告警不阻断"
- **关联 commit 引用**：不适用（治理·阻塞，无 commit）
- **per Ulysses 拍板** + **Mavis 起草**：门禁模式（阻断 vs 软告警）= **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

## §3 1 跟踪 + 3 PH 分段 issue body（#16-#19）

### Issue #16 — [tracking] 14-18 周主线 WBS 切分（PH-1 ~ PH-8）

- **类型**：跟踪 / `enhancement`
- **估算 token**：0（tracking only，per 父文档 §5.1）
- **任务范围**（per 父文档 §5.1）：
  - 14-18 周主线 WBS 切分（PH-1 ~ PH-8）
  - 对应 WBS-001 v0.7 + PLAN-001 v1.1 §3.1
- **关联 WBS / 文档**：
  - RGS-WBS-001 v0.7（**已知缺口**：head 实际 v0.11）
  - RGS-PLAN-001 v1.1 §3.1
- **阻塞**：无（tracking only）
- **关联 commit 引用**：不适用（tracking only）
- **per Ulysses 拍板** + **Mavis 起草**：本 issue body 草稿 = Mavis 起草。

---

### Issue #17 — [PH-1 段] 5 域 PH-1 32+32+32+32+32 = 160 L4 实施

- **类型**：PH 分段 / `enhancement`
- **估算 token**：~30+M（per 父文档 §5.1）
- **任务范围**（per 父文档 §5.1）：
  - 5 域 = player / economy / match / social / admin
  - 各域 PH-1 32 L4
  - 合计 160 L4
  - per WBS-001 §4.1-§4.5 + RGS-OPEN-QA-001 v0.2
- **Mavis 提建议域顺序（待 Ulysses 拍板）**（per 父文档 §1.1 M2）：player → economy → match → social → admin
- **关联 WBS / 文档**：
  - RGS-WBS-001 §4.1-§4.5
  - RGS-OPEN-QA-001 v0.2
- **阻塞**：
  - 5 域 Lead 各自签字 per DEC-005
  - DTL-037 §7.4 `inventory_items` 是否排期（**Ulysses 拍板**）
  - DTL-037 §7.5 多角色账号经济归属账号级 vs 角色级（**Ulysses 拍板**）
  - ARC-014 + ARC-026 评审门禁
- **关联 commit 引用**：不适用（PH 分段，未来工作）
- **per Ulysses 拍板** + **Mavis 起草**：域顺序 / 各域启动日 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

### Issue #18 — [PH-2~3 段] 平台层 + 商业 CDN 选型 + 长期记忆 + 风控

- **类型**：PH 分段 / `enhancement`
- **估算 token**：待估（per 父文档 §5.1）
- **任务范围**（per 父文档 §5.1 + §2.2）：
  - 平台层：ARC-053（双 Agent 体系）/ **ARC-054**（智能体平台统一运行时）/ **ARC-055**（运营管控 + 服务 Agent 矩阵）/ **ARC-056**（游戏性生态 + 仿真 Agent 矩阵）—— **全部待具名人类审批**（per §4 反馈单处置后状态）
  - 商业 CDN 选型（ARC-045-2）：TBD-CDN-101/102 已在 §1.3 ISS-094/095
  - 长期记忆向量存储：TBD-MEM-001 pgvector vs Milvus（ISS-096 待具名审批）
  - 风控规则数量上限 + Rhai 性能：ISS-100/101 待具名审批
- **关联 WBS / 文档**：
  - ARC-053/054/055/056
  - ISS-094/095/096/097/100/101
  - RGS-OPEN-QA-001 v0.2
- **阻塞**：
  - ARC-053/054/055/056 待具名人类审批
  - ISS-094/095/096/097/100/101 待具名审批
- **关联 commit 引用**：不适用（PH 分段，未来工作）
- **per Ulysses 拍板** + **Mavis 起草**：ARC 评审 / ISS 具名审批 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

### Issue #19 — [PH-4~8 段] 性能基线 + 仿真 + COC UI + MVP/GA 门禁

- **类型**：PH 分段 / `enhancement`
- **估算 token**：待估（per 父文档 §5.1）
- **任务范围**（per 父文档 §5.1 + §2.3-§2.5）：
  - **PH-4 性能基线**：
    - 100k DAU / 10k QPS 实测（NFR-OP-010 per ADR-0052 v0.2 §3.3）
    - 单副本 50-70k DAU / 5-7k QPS 实测（per ADR-0052 v0.2 §3.3 容量公式）
    - OLU 台账季度校准（per TBD-GOV-001）
  - **PH-5 ~ PH-6 仿真**：
    - 仿真 Agent 矩阵（BR-AGS-001~003，per ADR-0056 候选）
    - 沉浸式动态 NPC（BR-AGS-002）
    - 自动补偿业务范围（per TBD-AGO-001）
  - **PH-7 ~ PH-8 收尾**：
    - COC UI（ARC-008/013/019 + TBD-COC-001~006，per 附件D §1.3 ISS-102~107）
    - 事件保留策略（COC-004/005/006）
    - MVP/GA 判定标准（TBD-WF-007）
    - 工数与成本管理（TBD-WF-008）
- **关联 WBS / 文档**：
  - NFR-OP-010
  - ADR-0052 v0.2 §3.3
  - BR-AGS-001~003
  - ARC-008/013/019
  - TBD-COC-001~006 + TBD-WF-007/008 + TBD-GOV-001 + TBD-AGO-001
- **阻塞**：
  - ISS-102~107 待具名审批
  - TBD-WF-007/008 待具名审批
  - TBD-AGO-001 待具名审批
- **关联 commit 引用**：不适用（PH 分段，未来工作）
- **per Ulysses 拍板** + **Mavis 起草**：MVP/GA 门禁 / 工数与成本管理 / 仿真矩阵范围 = **等 Ulysses 拍板**；本 issue body 草稿 = Mavis 起草。

---

## §4 跨 issue 关联矩阵

| Issue | 父文档引用 | WBS L4 范围 | 阻塞数量 | Mavis 起草状态 |
|---|---|---|---|---|
| #8 (M1) | §1.1 M1 | WF-1-55.38~50（13 个）+ WBS-001 L4 进度表 v0.8 升版 | 0 | ✅ 已起草 |
| #9 (M2) | §1.1 M2 | 5 域 PH-1 §1-§3 起草 | 1（5 域 Lead 签字 per DEC-005）| ✅ 已起草 |
| #10 (M3) | §1.1 M3 | WF-1-2063~2074（12 个 L4）| 1（ISS-117 待具名审批）| ✅ 已起草 |
| #11 (M4) | §1.1 M4 | WBS-001 L4 任务进度表 v0.8 升版（**已知缺口**：head 实际 v0.11）| 0 | ✅ 已起草 |
| #12 (G1) | §1.2 G1 | — | 1（Ulysses 拍板）| ✅ 已起草 |
| #13 (G2) | §1.2 G2 | — | 1（Ulysses 在场 12 角色勾选）| ✅ 已起草 |
| #14 (G3) | §1.2 G3 | — | 1（Ulysses 确认清单）| ✅ 已起草 |
| #15 (G4) | §1.2 G4 | — | 1（Ulysses 拍板门禁模式）| ✅ 已起草 |
| #16 (跟踪) | §5.1 | 14-18 周主线 WBS 切分 | 0 | ✅ 已起草 |
| #17 (PH-1) | §5.1 | 5 域 160 L4 | 4（5 域 Lead + DTL-037 §7.4/§7.5 + ARC 评审）| ✅ 已起草 |
| #18 (PH-2~3) | §5.1 + §2.2 | ARC-053~056 + ISS-094/095/096/097/100/101 | 2 | ✅ 已起草 |
| #19 (PH-4~8) | §5.1 + §2.3-§2.5 | NFR-OP-010 + BR-AGS-001~003 + ARC-008/013/019 + TBD-* | 3 | ✅ 已起草 |

---

## §5 已知缺口（per 缺标比错标安全原则）

> 本节显式列出本 draft 文档未填补的事实差异，避免下游误读。

1. **关联 commit 实证**：仅 #10 (M3) 父文档引用 commit 链（`71c71bb/8d55fbd/...`），本任务**未**跑 `git log` 实证。**注**：Mavis 不沿用"per commit X 历史形态"叙事，所有 commit 引用保留父文档原文，**不**在本 draft 做验证。Phase D 实施前由 Ulysses 拍板的执行人跑 `git log -p --follow` 实证。

2. **WBS-001 head 状态偏差**：父文档 §1.1 M1 / M4 引用 WBS-001 v0.8 进度表，但实际 head = v0.11（per WBS-001 v0.9 / v0.9 sync / v0.10 / v0.11 修订历史，4 次升版 = Ulysses 17:04 JST 批次 + 18:21 JST 批次 + 19:05 JST 批次 + 21:00-21:02 JST 实测）。**本 draft 不**回填"per v0.7 历史形态"叙事，引用父文档原文 = v0.8。

3. **issue 编号来源**：父文档 §5.1 末段明说"per Mavis 2026-08-25 22:03 JST 实际开 issue 时记录"——本 draft **不**独立验证 issue 编号 #8-#19 是否在 UlyssesLeoLee/RustGameServer 仓库真实存在。本 draft 是**草稿**，不实际调 `gh issue create`。

4. **G1-G4 治理项**：本 draft **只**起草 issue body，**不**实际推进签字/接 CI 起草/改文档头。

5. **G2 签字栏空**：issue body 内**未**预设 ✅、**未**填签字栏、**未**预承诺截止日期（per 父文档 §5.1 末段第 2 句）。本 draft 12 个 issue body 全部遵守此约束。

6. **目录路径偏差**：父文档实际位于 `docs/12-工作流/`，本 draft 同目录落盘以保持路径一致。

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008）— 子代理 B（任务 3）| 初版：12 个 GitHub issue（#8-#19）body 草稿集合 + 跨 issue 关联矩阵 + 6 处已知缺口显式列 | 子代理 B 起草（per 主对话 2026-08-26 22:17 JST 派发）|

---

## §7 签字栏

> **Mavis 不代签事实清单**（per RGS-PLAN-002 §6 同源约束）：
> 1. 本 ISSUE-BODY-DRAFT 状态为"🟡 草案"——本 draft 是 12 个 issue body 的本地草稿，**不**等于已开 issue。
> 2. 12 个 issue body 内**未**预设 ✅、**未**填签字栏、**未**预承诺截止日期。
> 3. 每个 issue body 末尾显式标"per Ulysses 拍板" + "Mavis 起草"，**不**自动进入任何人或 agent 的工作队列。
> 4. 本 draft **不**实际调 `gh issue create`；issue 创建动作由 Ulysses 或其授权者执行。
> 5. 估算 token 仅供参考，**不**作为正式 OLU 申报。

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 起草者 | 架构师（Mavis 接手 agent per DEC-008）— 子代理 B（任务 3）| 2026-08-26 | 🟡 草案：12 个 issue body 草稿 + 6 处已知缺口显式列；待 Ulysses 终审"接受草稿"或"追加修正项" |
| 2 | 实际发布人 | ⏳ 待 Ulysses 或其授权者 | ⏳ | ⏳ Mavis **不**代发 issue |
