# QA_FINAL.md — Universal 5D Design Atlas v9.0

> **任务**: DESIGN_ATLAS_ONLY
> **治理**: `inherit_enforced_rules` (AGENTS.md §0-§3 适用)
> **修订**: Ulysses — Mavis 接手 (per AGENTS.md §0 + 守门 #14 v3+v4)
> **审批**: 架构师 — Mavis 接手

---

## 0. 输入核对 (最重要)

按守门 #1 的"**缺标比错标安全**", 必须如实核对输入。

| 字段 | 用户填入值 | 状态 |
|---|---|---|
| `project_input.name` | `auto` | ⚠️ 自动 / 未指定 |
| `project_input.summary` | `""` 空 | ⚠️ 未填写 |
| `project_input.user_requirements` | `[]` 空数组 | ⚠️ 未填写 |
| `project_input.website_url` | `""` 空 | ⚠️ 未填写 |
| `project_input.target_git_repositories` | `[]` 空数组 | ⚠️ 未填写 |
| `project_input.local_folders` | `[]` 空数组 | ⚠️ 未填写 |
| `project_input.files_and_designs` | `[]` 空数组 | ⚠️ 未填写 |
| `project_input.additional_reference_urls` | `[]` 空数组 | ⚠️ 未填写 |
| `project_input.source_roles` | `[primary, baseline, pattern, visual_reference]` | ⚠️ 无来源可分配 (全部空) |
| `project_input.source_resolution` | `auto` | ⚠️ N/A (无可解析来源) |
| `project_input.reference_look_and_feel.path` | `""` 空 | ⚠️ 未填写 |
| `project_input.reference_look_and_feel.url` | `""` 空 | ⚠️ 未填写 |
| `reference_github_repositories[0].url` | `""` 空 | ⚠️ 未填写 → borrow/do_not_borrow 全部跳过 |
| `product_to_design.*` (全部子字段) | `[]` | ⚠️ 未填写 |
| `project_input.output.version` | `"v9.0"` | ✅ 已落为目录名 + README 版本 |
| `project_input.output.directory` | `"./gemarc-5d-atlas-v9.0"` | ✅ 落为 `docs/gemarc-5d-atlas-v9.0/` (per 用户"放 docs 目录下") |
| `project_input.output.entry` | `"index.html"` | ✅ `docs/gemarc-5d-atlas-v9.0/index.html` |
| `project_input.output.publish_dir` | `"dist"` | ❌ **未创建** — DESIGN_ATLAS_ONLY 无构建步骤, 记录于 §3.6 |
| `project_input.output.report` | `"QA_FINAL.md"` | ✅ 本文件 |
| `project_input.languages.ui` | `"zh-CN"` | ✅ 入口 HTML `lang="zh-CN"` |
| `project_input.languages.supported` | `[zh-CN, ja-JP, en-US]` | ⚠️ 仅 zh-CN 产出, ja-JP / en-US 未产出, 记录于 §3.6 |
| `project_input.languages.formal_documents` | `"ja-JP"` | ❌ **未落实** — 正式文档实为 zh-CN, 记录于 §3.6 |
| `project_input.languages.report` | `"zh-CN"` | ✅ 本报告为 zh-CN |
| `task.kind` | `DESIGN_ATLAS_ONLY` | ✅ |
| `task.issue_title_prefix` | `"图集"` | ⚠️ N/A — 本次无 issue tracker 场景, 未产生 issue 标题 |
| `task.allow_app_implementation` | `false` | ✅ |
| `task.allow_production_deployment` | `false` | ✅ |
| `task.allow_remote_push` | `false` | ✅ |
| `task.allow_git_merge` | `false` | ✅ |
| `execution_environment.preferred_browser` | `"Chromium/Edge"` | ✅ 入口 HTML 为纯 HTML+CSS (0 script / 0 外链), 任意现代浏览器可开 |
| `execution_environment.require_direct_file_open` | `true` | ✅ file:// 双击可用, 已验证 23 条本地链接全通 |
| `execution_environment.npm_allowed_for_development` | `true` | ✅ 允许但未使用 (本任务无构建) |
| `execution_environment.npm_required_for_reader` | `false` | ✅ 读者端 0 依赖 |
| `execution_environment.offline_core_required` | `true` | ✅ 0 外链资源, 完全离线 |
| `git_delivery.work_branch` | `""` 空 | ✅ 沿用当前分支 `dev` |
| `git_delivery.merge_target` | `""` 空 | ✅ N/A (`allow_git_merge: false`) |
| `git_delivery.final_checkout_branch` | `""` 空 | ✅ N/A (未 checkout) |
| `git_delivery.commit_after_phase` | `false` | ✅ 单次 commit 落地 |
| `git_delivery.push` | `false` | ✅ 仅本地 commit, 未 push |
| `git_delivery.merge` | `false` | ✅ 未 merge |
| `governance.uq_guard` | `inherit_enforced_rules` | ✅ 继承 AGENTS.md §0-§3 |
| `governance.reviewer_approval_for_exemption` | `false` | ✅ 本次无任何豁免申请, 无需审批 |
| `governance.allow_hypothetical_demo_without_sources` | `true` | ✅ (已用于示例) |
| `governance.keep_unresolved_in_backlog` | `true` | ✅ (已用于本报告) |

### 0.1 关键结论

> **本次输入完全不含具体业务事实, 因此本 atlas 作为"通用方法论"交付**,
> 它**不**承诺与任何具体项目的设计交付。详见 §3 已知缺口。

---

## 1. DoD (Definition of Done)

### 1.1 L1 / L1.1 / L1.2 三件套

由于本任务 `allow_app_implementation: false` 且不涉及 `crates/` 代码变更:

| 级别 | 适用 | 状态 | 说明 |
|---|---|---|---|
| **L1** `cargo check --tests` | N/A | ✅ N/A | 纯文档任务, 无 Rust 改动 |
| **L1.1** `cargo test --lib` | N/A | ✅ N/A | 同上 |
| **L1.2** `cargo test --test '*' -- e2e` | N/A | ✅ N/A | 同上 |

### 1.2 文档 / 内容 DoD

| 检查项 | 状态 | 证据 |
|---|---|---|
| 5 维度文档齐全 (D1-D5) | ✅ | `atlas/D{1..5}-*.md` |
| 5 套可复用模板 | ✅ | `templates/D{1..5}-*.template.md` |
| 5 个出口判据清单 | ✅ | `checklists/D{1..5}-checklist.md` |
| 1 个集成校验清单 | ✅ | `checklists/cross-d-integration-check.md` |
| 1 个 worked example (假设性) | ✅ | `examples/hypothetical-product/*.md` |
| 入口 HTML (单文件, 离线) | ✅ | `index.html` |
| 总览 README | ✅ | `README.md` |
| QA 报告 (本文件) | ✅ | `QA_FINAL.md` |
| 空 input 已知缺口显式列出 | ✅ | §3 |
| 按代签格式 (per AGENTS.md §0) | ✅ | README + QA_FINAL + index.html + **atlas/D1-D5 五份维度文档各有独立「代签」段** (2026-10-21 审核补齐) |

---

## 2. 入口 URL 与可访问性

由于 `execution_environment.require_direct_file_open: true` + `offline_core_required: true`,
本 atlas 不发布到外网, 仅供双击本地打开:

- **入口**: `docs/gemarc-5d-atlas-v9.0/index.html`
- **打开方式**: file:// 双击 / `pwsh Start-Process index.html`

### 2.1 入口 HTML 验证

| 检查项 | 状态 | 备注 |
|---|---|---|
| `lang="zh-CN"` | ✅ | UI 语言设置 |
| `viewport` meta | ✅ | 移动端适配 |
| `prefers-reduced-motion` CSS query | ✅ | 减弱动效支持 |
| `focus-visible` outline | ✅ | 键盘焦点可见 |
| 颜色对比 ≥ WCAG AA | ✅ | **实测 (WCAG 相对亮度公式, 2026-10-21)**: `--text #e6edf3` vs `--bg #0f1419` = **15.67:1** (AAA); vs `--surface #1a1f26` = 14.02:1; `--text-muted #8b949e` vs bg = **6.02:1** / vs surface = 5.38:1; `--primary #4493f8` vs bg = 5.97:1; `--accent #d2a8ff` vs bg = 9.51:1。**全部 ≥ AA 4.5:1 ✅; 但 muted 次要文字 < 7:1, 不得宣称"全部 ≥ 7:1"** |
| 完全离线 (无 CDN / 无外链 JS) | ✅ | 0 外部依赖 |
| 不需要 npm | ✅ | per `npm_allowed_for_development: true` 允许但本任务不用 |
| 不需网络浏览器偏好 | ✅ | Chromium / Edge / Firefox 都能跑 |

---

## 3. 已知缺口 (Known Gaps — 缺标比错标)

按 `governance.keep_unresolved_in_backlog: true`, 显式列出:

### 3.1 来源完全为空 (P0)

| 缺口 | 缓解 |
|---|---|
| 项目名 / 项目摘要 / 用户需求 全空 | atlas 作为通用方法论交付, 不绑定项目 |
| 目标仓库 / 本地文件夹 / 参考仓库 全空 | 无法做 git 历史实证, 文档需用户填真实数据后才有"可绑定"价值 |
| UI 参考 (path + url) 全空 | D4 token / 组件规格全为通用起手模板, 待真实参考补 |
| 凭据 / 监管 / 不可逆决策 证据全无 | D1 硬约束清单与示例 ADR 显式标 "⚠️ 待真实证据替换" |

### 3.2 假设性示例的诚实标注 (P0)

`examples/hypothetical-product/` 完全为虚构:
- 项目名 ACP (Arcade Control Panel) 不对应任何真实组织
- 所有 persona / 数据 / 数字仅作格式演示
- 借用了 governance `allow_hypothetical_demo_without_sources: true` 显式允许
- **不可作为对外材料的文案来源**

### 3.3 各层已知缺口 (per atlas 各层 "Known Gaps" 汇总)

各维度文档的「已知缺口 (Known Gaps)」章节位置 (逐份核对, 非统一编号): D1 §8 / D2 §9 / D3 §9 / D4 §8 / D5 §10, 此处汇总:

**D1**:
- [ ] 用户访谈样本 < 30
- [ ] NSM 基线缺失 (上线后回填)
- [ ] 法规清单只列大类
- [ ] 反对者签字栏空白

**D2**:
- [ ] 真实流量未采样 → L0 选择无证据
- [ ] 同义词表无实证
- [ ] 分类边界歧义
- [ ] 大规模 IA 变形规则缺

**D3**:
- [ ] A11y 真人验证缺
- [ ] 错误路径覆盖率 < 100%
- [ ] 撤销窗口经验值缺
- [ ] 表单字段 > 7 项

**D4**:
- [ ] 暗色模式对比度未复测
- [ ] i18n 文案扩展未实测
- [ ] 图标 license 未审计
- [ ] 减弱动效 iOS / Android 未实测

**D5**:
- [ ] 性能预算基线缺
- [ ] 告警值班未指定
- [ ] A11y 真人未做
- [ ] 回滚演练未跑
- [ ] SBOM 未生成

### 3.4 集成缺口 (per cross-d-integration-check 清单, **未跑分**)

> ⚠️ 该清单 13 项**尚未逐条跑分**, 以下仅列出已识别的主要缺口, 非穷尽:

未进行真实项目集成, 因此:
- [ ] 跨 D1→D5 名词一致性尚未实测
- [ ] NSM ↔ 性能预算一致性尚未验证
- [ ] D4 token ↔ D5 CSS 变量尚未配

### 3.5 文档完整性缺口

- [ ] mermaid 渲染需浏览器或 CI 安装 mermaid 渲染器; 当前不带 mermaid 依赖
- [ ] 图标 / 插画资产未提供 (per D4 §"图标体系"理论位置, 实际资产不在本 atlas 范围)
- [ ] localized i18n strings 未逐字符串审 (per `languages.supported: zh-CN, ja-JP, en-US`)

### 3.6 Prompt 规格未完全落实的项 (新增于 2026-10-21 审核)

以下 3 项是 prompt **明确设定但本次未落地**的, 与 §3.1 的"输入为空"性质不同 ——
这些是**规格要求**, 缺失即偏差, 不是"本来就没输入":

| # | 规格项 | 实际状态 | 判定 | 原因 / 补足方式 |
|---|---|---|---|---|
| 1 | `output.publish_dir: "dist"` | **未创建** `dist/` 目录 | **偏差 (低)** | `task.kind: DESIGN_ATLAS_ONLY` + `allow_app_implementation: false` → 无构建产物, `dist/` 无意义。README 已说明"单文件入口即最终产物"。若后续接入构建 (e.g. 静态站点打包), 需补 `dist/` 产出约定 |
| 2 | `languages.formal_documents: "ja-JP"` | 全部 25 个文件**实为 zh-CN**, 无 ja-JP 版本 | **偏差 (中)** | 正式文档 (本 QA 报告 / README / 5D 方法论) 应为 ja-JP, 本次以 zh-CN 交付。⚠️ 入口 `index.html` 原标注 "ja-JP (formal)" 属**不实宣称**, 已于同日修正为实际交付语言。补足方式: 正式文档出 ja-JP 版 (需专业日语校对, 非机翻可替代) |
| 3 | `languages.supported: [zh-CN, ja-JP, en-US]` | 仅 zh-CN 产出 | **偏差 (中)** | 同上。补足方式: en-US 版可机翻 + 人工校对; ja-JP 建议人工 |

> ⚠️ **诚实声明**: 上述 3 项在初版 (commit `63b60104`) 中未被记录, 是本次**审核自查**发现的。
> 同批次另发现 `QA_FINAL.md` 附录 A 文件计数错误、README 版本历史 commit 栏未回填、
> commit 标题超 50 字符上限 —— 均已在同批修正 (见 §7 修订历史)。

### 3.7 治理偏差 (非文档内容)

| # | 偏差 | 判定 | 处置 |
|---|---|---|---|
| 1 | commit `63b60104` 标题 `docs(atlas): add universal 5D design atlas v9.0 method set` = **58 字符**, 超 AGENTS.md §2.6 规定的 50 字符上限 | **偏差 (中)** | **不 amend** (per AGENTS.md 禁追溯改写 / 禁 filter-branch)。已在此记录; 后续 commit 标题恢复 ≤ 50 字符 |
| 2 | `task.issue_title_prefix: "图集"` 未被使用 | **N/A** | 本次无 issue tracker 场景, 不产生 issue 标题 |


---

## 4. 验收清单 (Acceptance Criteria)

按 `task.kind: DESIGN_ATLAS_ONLY` 不允许实现, 仅文档交付:

- ✅ 文档集齐 — 实测 25 个文件 (5D + 5 模板 + 6 清单 + 1 总览 README + 1 入口 HTML + 1 QA 报告 + 1 示例 README + 5 示例文档)
- ✅ 入口 HTML 可直接打开 (单文件, 离线)
- ✅ 空 input 已知缺口显式列出 (per 守门 #1)
- ✅ 假设性示例诚实标注 (per `allow_hypothetical_demo_without_sources: true`)
- ✅ 与 RGS 治理不重叠 (本 atlas 在 `docs/gemarc-5d-atlas-v9.0/`, 与 `docs/14-项目治理/` 并列)
- ✅ 凭据 / 监管 / 版权 任何可能引发法律问题处, 都加了 ⚠️ 占位警告
- ✅ 不动 `crates/` (per `allow_app_implementation: false`)
- ✅ 不动 manifests / k8s yaml (per `allow_production_deployment: false`)
- ✅ 不 git push / 不 merge (per `allow_remote_push: false` / `allow_git_merge: false`)

---

## 5. 治理与代签链

### 5.1 本次守门遵循

| 守门 | 状态 | 备注 |
|---|---|---|
| 守门 #1 禁回溯叙事 | ✅ | 文档全部基于方法论 + 假设示例, 无"原本是"类叙事 |
| 守门 #1 缺标比错标 | ✅ | 已知缺口全部显式列出 (§3) |
| 守门 #5 env 安全 (8/27 11:06 JST hard ban) | ✅ | 文档中凭据只有"⚠️ 走 env"提示, 无任何值 |
| 守门 #14 v3+v4 Mavis 默认代签 | ✅ | 见 §5.2 |
| 守门 D3 commit 模板 (per AGENTS.md §2.6) | ⚠️ **部分偏差** | body 四段 (DoD/Evidence/代签/派生约束) 齐全 ✅; 但标题 58 字符 > 50 上限, 且 scope `atlas` 不在 §2.6 枚举 (域/平台/工具/元) → 见 §3.7, **不 amend** |
| L11/L12 cargo dir lock 防御 | ✅ N/A | 纯文档任务 |

### 5.2 代签链

| 角色 | 真名 | 临时代签 |
|---|---|---|
| 修订人 | Ulysses | Mavis (per 9/8 15:19 JST 第 6 次强化) |
| 审批人 | 架构师 (一人公司 12 角色 per DEC-008) | Mavis (per 8/27 19:39/20:56/21:59 JST 三次强化) |
| 真人到位追溯 | 待真人 (Ulysses / 架构师) 上岗后 | 修订历史 +1 行 |

### 5.3 与 RGS 主线的关系

本 atlas 不影响:
- RGS 主线 (player / economy / match / social / admin / batch 6 域)
- RACI 文档 (`docs/14-项目治理/`)
- 部署恢复 / 部署 yaml (`docs/deploy/01-k8s-manifests/`)
- 任何 `crates/` 代码

它是**独立的方法论文档**, 与 RGS 项目治理解耦。

---

## 6. 后续行动 (Backlog)

按 `keep_unresolved_in_backlog: true`:

| # | 项 | 触发条件 |
|---|---|---|
| 1 | 真用户 / 真项目绑定本 atlas | 等 Ulysses 给具体项目 |
| 2 | mermaid 渲染器集成 (CI 或 HTML) | 等 atlas 引用 mermaid 图时被读者要求 |
| 3 | **ja-JP 正式文档版** (per `languages.formal_documents: "ja-JP"`) | 需专业日语校对, 非机翻可替代 |
| 4 | **en-US 版** (per `languages.supported`) | 机翻 + 人工校对 |
| 5 | **`dist/` 产出约定** (per `output.publish_dir`) | 等 atlas 接入静态站点构建 |
| 6 | D2 IA lint CI 实装 (perl / js / rs) | 等真实项目接入 IA lint 规则 |
| 7 | D4 token → Tailwind theme 转换器 | 等前端项目接入 |
| 8 | 与 visual-page / web-design skill 联动 | 等用户主动要求 |
| 9 | 真人 (Ulysses / 架构师) 上岗后追溯签字覆盖修订历史 | 真人到位 |

---

## 7. 最终签收建议

将本 QA 报告 + atlas 一起作为一份**方法论文档集**提交。
按守门 #14 v3+v4 永久代签 policy, Mavis 代签 Ulysses + 架构师。
真人到位前不追溯改写历史文档中的 "审批者 = —" 字段 (per 8/27 19:39 JST 决策)。

---

## 附录 A: 文件清单 (机器可校验)

```text
gemarc-5d-atlas-v9.0/
├── index.html                            (15 KB, 单文件离线入口)
├── README.md                             (4.5 KB)
├── QA_FINAL.md                           (本文件)
├── atlas/
│   ├── D1-concept.md
│   ├── D2-information-architecture.md
│   ├── D3-interaction.md
│   ├── D4-visual.md
│   └── D5-engineering.md
├── templates/
│   ├── D1-concept-brief.template.md
│   ├── D2-ia-map.template.md
│   ├── D3-flow-spec.template.md
│   ├── D4-component-spec.template.md
│   └── D5-adr.template.md
├── checklists/
│   ├── D1-checklist.md
│   ├── D2-checklist.md
│   ├── D3-checklist.md
│   ├── D4-checklist.md
│   ├── D5-checklist.md
│   └── cross-d-integration-check.md
└── examples/
    └── hypothetical-product/
        ├── README.md
        ├── 01-concept.md
        ├── 02-ia-map.md
        ├── 03-flow-spec.md
        ├── 04-component-specs.md
        └── 05-engineering-summary.md
```

共 **25 个文件** (实测 `Get-ChildItem -Recurse -File` = 25, 2026-10-21 审核复核):
- 5 维度方法论 (atlas/D1-D5)
- 5 模板 (templates/D1-D5)
- 6 清单 (checklists/D1-D5 + cross-d-integration-check)
- 1 总览 README
- 1 入口 HTML
- 1 QA 报告 (本文件)
- 1 示例 README + 5 示例文档

合计 5 + 5 + 6 + 1 + 1 + 1 + 1 + 5 = **25** ✅

> ⚠️ **初版此处写的是 "22 个文件"**, 与同段分项累加 (25) 自相矛盾 ——
> 属"结论与事实脱钩"的自证实例, 已于 2026-10-21 审核批次修正。

### 实测校验记录 (2026-10-21 审核)

| 检查项 | 命令 | 结果 |
|---|---|---|
| 文件数 | `Get-ChildItem -Recurse -File \| Measure-Object` | 25 ✅ |
| index.html 本地链接 | 提取 `href` → `Test-Path` | 23 条, **0 死链** ✅ |
| index.html 外链资源 | 正则 `(src\|href)="https?://` | 0 ✅ (完全离线) |
| index.html `<script>` | 正则 `<script` | 0 ✅ (纯 HTML+CSS) |
| index.html 标签配平 | `<div>` 9 / `</div>` 9; `<section>` 7 / `</section>` 7 | ✅ |
| index.html `lang` | 正则 `<html lang="` | `zh-CN` ✅ |
| 交付物内硬编码凭据 | 正则 `password\s*[:=]\s*\S+\|secret\s*[:=]\s*\S+\|token\s*[:=]\s*[A-Za-z0-9]{8,}\|BEGIN (RSA\|OPENSSH\|EC) PRIVATE KEY` (严格: 必须带赋值符 + 值) | **0 命中** ✅ (宽口径含 "secret" 单词有 2 处, 均为元描述文本: 本表自身 + `D5-engineering.md` 的 "在仓 secrets 扫描" 示例行, 非凭据) |
| commit 含 .log/.tmp | `git show --name-only` | 0 ✅ |
| commit 改动范围 | `git show --stat` | 仅 `docs/gemarc-5d-atlas-v9.0/` ✅ |
| D1-D5 出口判据 ↔ checklist 覆盖 | 逐条人工比对 (非仅关键词) | **D1/D2/D3 全覆盖 ✅; D4 补 2 条 (Storybook) ✅; D5 补 runbook 列 + ADR 索引 (D5 正文 §3.3/§3.4) ✅** → 8/8 维度全覆盖 |
| 示例文件假设标注 | 首 6 行匹配 | 6/6 带 ⚠️ 标注 ✅ |
| 示例跑分 ↔ 出口判据 1:1 | 逐份逐条 | 5/5 份已改写为 1:1 表格 (02/03/04/05 补齐缺失项; 01 原已 7/7) ✅ |
| 各维度文档代签段 | 正则 `代签` 命中 | D1-D5 五份**全部**有独立「代签」段 ✅ (原仅 D1 有) |
| 代码围栏配平 | 逐文件数 ` ``` ` | 25 个 md 全部偶数配平 ✅ |
| 各维度已知缺口章节号 | 逐份读标题 | D1 §8 / D2 §9 / D3 §9 / D4 §8 / D5 §10 (原 QA 笼统写 "§9 / §10", 2/5 不符) |

> ⚠️ **诚实声明 (本表自身)**: 上一版此处写的是 "关键词映射 → 5/5 覆盖", 但**关键词映射查不出概念缺口** ——
> D4 的 "任意 L1 组件可独立 Storybook 展示"、D5 的 "告警有 runbook 链接" / "ADR 索引表完整"
> 三条在对应 checklist 中并无正文承载, 是本次**逐条人工比对**才发现的。
> 量具本身的局限必须写进结论, 不能让读者以为覆盖是穷尽的。



---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 审批 | 摘要 | commit |
|---|---|---|---|---|---|
| v9.0 | 2026-10-21 | Ulysses (Mavis 代签) | 架构师 (Mavis 代签) | 初版: 5D 方法论 + 5 模板 + 6 清单 + 1 假设性示例 + 入口 HTML + QA 报告 | `63b60104` |
| v9.0.1 | 2026-10-21 | Ulysses (Mavis 代签) | 架构师 (Mavis 代签) | 审核批次修正 5 处 (见下表) | (本次) |

### v9.0.1 修正明细 (2026-10-21 审核)

> 审核方式: 主会话自查 + 独立 verifier 子代理 (只读, 对 commit `63b60104` 取证)。
> 独立审核结论 **PASS_WITH_FINDINGS**, 共 16 项; 下列 16 项**全部已处置**。

| # | 严重度 | 位置 | 问题 | 修正 |
|---|---|---|---|---|
| 1 | High | `index.html:84` | 宣称 "ja-JP (formal) · en-US (UI)", 实际 25 文件全为 zh-CN (假名 0 命中) → **不实宣称** | 改为 "zh-CN (实际交付) · ja-JP / en-US 未产出", 治理段加偏差表 |
| 2 | High | `QA_FINAL.md` 附录 A | 自称"机器可校验"却写 "共 22 个文件", 实测 25, 且同段分项累加也是 25 (段内自相矛盾) | 改为 25 + 分项累加式 + 实测校验记录表 |
| 3 | Medium | `QA_FINAL.md` §0 | 输入核对表仅覆盖 9/37 非空设置项; 16 个设置标识符全文 0 命中 | 补齐至全量 (source_roles / source_resolution / output.* / languages.* / issue_title_prefix / execution_environment.* / git_delivery.* / reviewer_approval_for_exemption / use_as_target / borrow / do_not_borrow) |
| 4 | Medium | `QA_FINAL.md` §2.1 | "自检 dark theme 文本 vs 背景 ≥ 7:1" 从未实测; 实测 muted 仅 6.02 / primary 5.97 | 改为 WCAG 公式实算的 7 组比值, 并显式声明"AA 全达标, 但 muted < 7:1, 不得宣称 ≥7:1" |
| 5 | Medium | `QA_FINAL.md` §1.2 | "按代签格式 ✅ README + atlas 各层" — 实测 D2/D3/D4/D5 代签命中 **0** | **给 D1-D5 五份维度文档各补独立「代签」段** (修真实缺口, 而非缩窄口径) + 改写证据栏 |
| 6 | Medium | `QA_FINAL.md` §5.1 | "守门 D3 commit 模板 ✅" — 实测标题 58 字符 > 50, scope `atlas` 不在 §2.6 枚举 | 改为 ⚠️ 部分偏差 + 指向 §3.7; **不 amend** (禁追溯改写) |
| 7 | Medium | `QA_FINAL.md` §3 | `output.publish_dir: "dist"` 未创建且未记为缺口 | 新增 §3.6 逐项记录 (含偏差等级与补足方式) |
| 8 | Medium | `atlas/D4-visual.md` vs `checklists/D4-checklist.md` | D4 判据 "任意 L1 组件可独立 Storybook 展示" 无清单承载 | `D4-checklist.md` §C 补 2 条 |
| 9 | Low | `README.md:84` | "所有 markdown 文件之间仅靠相对链接" — 24 个 md 文件 `](` 命中 **0** | 改为准确表述: 唯一链接图在 index.html (23 个相对路径, 0 死链) |
| 10 | Low | `QA_FINAL.md` §3.3 | "各维度文档 §9 / §10 都列了本层缺口" — 实为 D1 §8 / D4 §8 | 改为逐份列出真实章节号 |
| 11 | Low | `QA_FINAL.md` §3.4 | 标题含 "跑分", 但从未跑分 (13 项仅列 3 项) | 去掉"跑分"措辞 + 声明"未跑分, 以下非穷尽" |
| 12 | Low | `README.md:99` | 版本历史 commit 栏 "(待 commit)" 占位 | 回填 `63b60104` + 加 v9.0.1 行 |
| 13 | Low | `atlas/D5-engineering.md:325-326` | 可填模板里预填 "secrets 扫描: 0 命中" 等**无来源结论** (同 block 其他行用 `___`) | 改回 `___` 占位 |
| 14 | Low | `QA_FINAL.md` / `README.md` | 拼写 "perls" | 改 "perl" |
| 15 | Low | `checklists/D5-checklist.md` vs `atlas/D5-engineering.md` | checklist 要求 "告警有 runbook 链接" / "ADR 索引表完整", D5 正文无对应字段 | **D5 正文补 §3.3 runbook 列 + 新增 §3.4 ADR 索引** (保留 checklist 严度) |
| 16 | Low | `QA_FINAL.md` 实测校验表 | "覆盖 5/5 ✅" 仍高估; 凭据行所写正则宽于实际执行的严格正则 | 覆盖声明改为逐条人工比对结论 + 列出 3 条曾未覆盖项; 凭据行写出实际严格正则并注明宽口径 2 处元描述命中 |

### 额外修正 (超出 verifier 16 项)

| # | 位置 | 问题 | 修正 |
|---|---|---|---|
| 17 | `atlas/D4-visual.md` vs `D4-checklist.md` | 组件数阈值不一致: 正文 "≥ 20 个" vs checklist "L1≥12/L2≥8/L3≥6/L4≥4" (=30) | 统一为 "≥ 30 个 (L1≥12/L2≥8/L3≥6/L4≥4)" |
| 18 | `examples/hypothetical-product/02~05` | 示例"出口判据跑分"与维度出口判据**非 1:1** (02 缺 1 / 03 缺 1 / 04 缺 3 / 05 缺 1) | 4 份全部改写为 1:1 表格, 状态诚实标 ⏳ (04 仅 2/8 达成, 故意保留以示"诚实标未做") |

### 未修正项 (含原因)

| # | 项 | 原因 |
|---|---|---|
| 1 | commit `63b60104` 标题 58 字符 > 50 上限; scope `atlas` 不在 §2.6 枚举 | **禁追溯改写** (per AGENTS.md 禁 amend / rebase / filter-branch)。已记录于 §3.7, 后续 commit 标题恢复 ≤ 50 字符且 scope 取枚举内值 |
| 2 | 无 ja-JP / en-US 正式版 | 需专业日语 / 英语校对, 非机翻可替代; 已列入 §6 backlog #3 / #4 |
| 3 | 无 `dist/` 目录 | `DESIGN_ATLAS_ONLY` 无构建步骤; 已列入 §6 backlog #5 |
| 4 | `index.html` 未在真实浏览器渲染验证 | 独立审核明示"未验证"项; 静态解析 + WCAG 实算已覆盖离线性/配平/lang/对比度, 像素级渲染待人工打开确认 |
| 5 | mermaid 渲染未验证 | 环境无 mermaid 渲染器; 交付物已自陈 |

---

**报告结束**
