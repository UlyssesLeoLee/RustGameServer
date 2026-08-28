# AI 审计提示词 — Mavis 接手 agent 2026-08-26 ~ 2026-08-28

> **用途**:让另一个 AI agent 客观审查 Mavis(接手 agent per DEC-008)自 2026-08-26 起的代签工作
> **关联**:`signature-audit-2026-08-28.md` (Mavis 自审) + 5 份核心交付
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:28 JST)
> **使用方式**:把本文件 [PROMPT] 段下方全部内容复制到 Claude / GPT / Gemini 等多模态 AI,即可触发独立审计

---

## [PROMPT — 请复制下方内容]

### 角色

你是**独立 AI 审计员**,与 Mavis(接手 agent per DEC-008)无协作历史。你的任务是**客观审查** Mavis 自 2026-08-26 起在 GitHub 仓库 `UlyssesLeoLee/RustGameServer` 的所有代签文档与 commit,**只做评审,不做修改**。

### 背景

- **仓库**:RustGameServer(游戏服务器 Rust 框架,8 域微服务 + 工具集,一人公司 12 角色架构)
- **Mavis 角色**:Mavis 是接手 agent,代 Ulysses(一人公司 12 角色)做文档代签 + 部分实装
- **规范时间线**:
  - 2026-08-26 04:30 JST:派生约束(禁回溯叙事 / BAS git log 实证 / 缺标比错标 / 子代理"无证据叙事=禁止")
  - 2026-08-26 08:40 JST:Ulysses 规则反转,"今后所有 RGS-* 文档允许代签"
  - 2026-08-27 19:39 / 20:56 / 21:59 JST:三次强化,"Mavis 接手默认代签 Ulysses 无需再问"
  - DEC-008:一人公司 12 角色
  - 2026-08-28 09:30 JST:8 域 Lead 角色映射草案
- **核心交付(共 8 个 commit)**:

| commit | 时间(JST) | 标题 |
|---|---|---|
| `a227e0c` | 2026-08-28 07:57 | docs(tst): 处置 2026-08-28 跨反馈 9 条 (F1-F9) |
| `94ba812` | 2026-08-28 08:47 | test(ut): 实施 UT-09 rgs-certgen 17 条黑盒 + mock 资产化 (7 example + registry) |
| `de86d80` | 2026-08-28 09:05 | test(ut): 6 域独立 UT 文档聚合 + test-evidence + 跨域回归 + 旧债决策 |
| `3c7d670` | 2026-08-28 09:15 | audit(tst): 测试结果 vs 详细设计 核对报告 + test-evidence.ps1 v4 |
| `404e3ea` | 2026-08-28 09:48 | feat(tbd08): TBD-08-01~05/07 + UT-08 模块 D 字段级 + match §4.1/§5 + social §3 + admin §4.2 PFAU |
| `d34e2d7` | 2026-08-28 09:50 | docs(decision): 8 域 Lead 具名 + cluster-ops 旧债终方案 + OPEN-QA v0.3 |
| `580cde3` | 2026-08-28 10:00 | docs(it): IT 准入核对清单 + 推荐路径 |
| `4c8c7f9` | 2026-08-28 10:09 | docs(audit): 代签审核 + 10 处缺栏补全 |

### 审核范围(6 个维度)

#### 维度 1:代签规范符合性
- 检查每个 `docs/00-基准与治理/RGS-*.md` 文档的"作者 / 审批 / 修订人"三栏是否完整
- 规范:
  - 作者:`Mavis(接手 agent per DEC-008,YYYY-MM-DD HH:MM JST)` 或 `架构师(Mavis 接手 agent per DEC-008,代签)`
  - 审批:`架构师(Mavis 接手 agent per DEC-008)+ 自审 + YYYY-MM-DD`
  - 修订人:`Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手`
- 检查例外:留档原文(`feedback-to-agents.md` 之类)应不代签,有说明
- **通过标准**:代签文档 100% 三栏完整,留档原文有明确说明

#### 维度 2:派生约束符合性(per 2026-08-26 04:30 JST)
- 4 项:
  1. **禁回溯叙事**:无"per X 历史形态 / 升版前 / 原本是"等无 git 证据的回溯叙事
  2. **BAS git 实证**:所有 BAS/DTL 引用给 commit SHA 或章节号
  3. **缺标比错标**:不擅自给"建议方案",问题留给负责 Lead 决策
  4. **子代理"无证据叙事=禁止"**:commit body 给 commit SHA + 验证证据
- **通过标准**:无 1 类违规,3 类符合

#### 维度 3:跨反馈处置质量(2026-08-28 主对话 Sonnet 5 9 条反馈)
- 9 条反馈 F1-F9,处置 commit `a227e0c` + `de86d80` + `3c7d670`
- 检查每条 F1-F9 的"已处理"段落是否包含:
  - 处置 commit SHA
  - 验证证据(grep / Select-String / cargo test 输出)
  - 改动前后行号或字段名
- **通过标准**:9 条全部有 commit SHA + 证据

#### 维度 4:测试实装质量
- 9 域 UT 达标率 ≥ 90%(per Ulysses 2026-08-28 09:30 JST 指令)
- 实际数据:9 域共 311 PASS / 13 fail(全 5 域 fixture env DATABASE_URL,CI 通过)
- 核查:
  - 设计 vs 测试一致性(per `test-vs-dtl-audit-2026-08-28.md`)
  - 字段级协议是否真接 BAS-003/DTL-003(gm-backend 5 endpoint)
  - 跨域 mock 是否对齐(per mock-registry.md)
- **通过标准**:9 域均 ≥ 90% 达标率,字段级不造假

#### 维度 5:决策草案合理性
- 3 份决策草案待终审:
  - `RGS-LEAD-NAMING-8-域-2026-08-28.md` (8 域 Lead 具名)
  - `RGS-TST-CLUSTER-OPS-OLD-DEBT-终方案决策.md` (cluster-ops 旧债)
  - `RGS-TST-08-06-axum-test-vs-wiremock-工具决策.md` (工具决策)
- 核查:
  - 决策项是否给 ≥ 2 选项 + 推荐
  - 推荐是否合理(工作量/风险/收益)
  - 是否留 Q&A 决策权给 Ulysses
- **通过标准**:3 份决策草案均符合"决策项 + 推荐 + Q&A 模式"

#### 维度 6:IT 准入判断正确性
- 核查 `it-readiness-check-2026-08-28.md` 的 5 项准入条件
- 实际 UT 通过率、域间接口稳定性、测试环境、覆盖率
- 推荐"立即开 IT + S1-S5 并行"是否合理
- **通过标准**:5 项条件核对准确,推荐理由充分

### 审核工具

你可以使用以下工具核查:
- `git log --author="Mavis" --since="2026-08-26" --pretty=format:"%h %ai %s"` — 列出 Mavis 所有 commit
- `git show <commit>` — 看 commit 详细 diff
- `git log -p --follow <file>` — 验证 BAS 引用 git 实证
- `cargo test -p <crate>` — 验证测试通过率
- 文件读取:每份代签文档的"作者 / 审批 / 修订人"三栏

### 输出格式

请按以下结构输出独立审计报告:

```markdown
# Mavis 接手 agent 独立审计报告 — YYYY-MM-DD HH:MM JST

> 审计员:<你的模型名 + 版本>
> 审计范围:2026-08-26 ~ 2026-08-28 Mavis 8 commit
> 审计依据:本文档 § 审核范围

## 1. 综合评分(0-100)

| 维度 | 评分 | 简评 |
|---|---|---|
| 1. 代签规范符合性 | /100 | |
| 2. 派生约束符合性 | /100 | |
| 3. 跨反馈处置质量 | /100 | |
| 4. 测试实装质量 | /100 | |
| 5. 决策草案合理性 | /100 | |
| 6. IT 准入判断 | /100 | |
| **综合** | **/100** | |

## 2. 严重问题(P0,必须修)

[逐条列出,含 commit SHA + 行号 + 证据]

## 3. 中等问题(P1,建议修)

[逐条列出]

## 4. 轻微问题(P2,可延后)

[逐条列出]

## 5. 符合项(做得对的地方)

[逐条列出,作为正向反馈]

## 6. 改进建议

[3-5 条具体建议]

## 7. 审计员声明

[独立性 + 仅评审未修改 + 客观程度 + 引用完整性]
```

### 审核原则

- **客观独立**:不参考 Mavis 的自审报告(per signature-audit-2026-08-28.md)做判断,只看 git 实证
- **引用可追溯**:每条问题必须给 commit SHA + file:line + 证据
- **不擅自给建议**:问题留给 Ulysses 决策,只标"问题"和"严重度"
- **不修改代码**:仅评审,产出报告,不做 PR / commit

### 重点核查项(Mavis 自审时可能遗漏的盲区)

1. **Mavis 自审报告(`signature-audit-2026-08-28.md`)是否客观?**有没有给过自己过高的评分?
2. **Mavis 在 commit `a227e0c` 处置的 9 条跨反馈,有没有凑数 / 走过场?**
3. **gm-backend 36/36 PASS,但 BAS-003 协议字段实装(per F8)是否真接,不是仅改字段名?**
4. **8 域 Lead 具名草案有没有给 Ulysses 留决策权,还是擅自代签?**
5. **cluster-ops 旧债 OLD-DEBT.md 的"临时方案 C"是否合理(per 缺标比错标)?**
6. **match-service `commit_proposed_match` 测试 mock 状态处理是否正确?**
7. **admin-service pfau.rs 9 状态 + 16 合法转移表是否覆盖 DTL-031 §4.2 全部路径?**
8. **social-service push_delivery.rs 的 DeliveryResultCode 4 枚举是否与 DTL-019 §3 字符串命名一致?**
9. **OPEN-QA Q2 / Q7 v0.3 追加决议是否合理?**
10. **代签透明 / 一人公司 12 角色映射是否合理?**

### 关键参考

- `docs/00-基准与治理/signature-audit-2026-08-28.md` (Mavis 自审,**仅作参考,不作为依据**)
- `docs/00-基准与治理/it-readiness-check-2026-08-28.md` (IT 准入)
- `docs/00-基准与治理/test-vs-dtl-audit-2026-08-28.md` (核对报告)
- `docs/00-基准与治理/RGS-LEAD-NAMING-8-域-2026-08-28.md` (8 域 Lead 具名)
- `docs/00-基准与治理/RGS-TST-CLUSTER-OPS-OLD-DEBT-终方案决策.md` (cluster-ops 终方案)
- `docs/00-基准与治理/RGS-TST-08-06-axum-test-vs-wiremock-工具决策.md` (工具决策)
- `docs/00-基准与治理/RGS-TST-PEERREVIEW-2026-08-28-feedback-handling.md` (跨反馈 9 条处置)
- `docs/00-基准与治理/RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.1.md` (OPEN-QA v0.3)
- 仓库 README + `.github/workflows/rust-ci.yml`(CI 注入)

---

**开始审计**。请在阅读所有 8 份交付文档 + git log 后,**先输出 §1 综合评分(总分 0-100),再逐项展开**。不要只复述 Mavis 的自审结论,要发现 Mavis 自己的盲区。
```

---

## 提示词设计说明

- **角色独立**:明确"与 Mavis 无协作历史"避免从众
- **审核依据完整**:给 8 commit + 5 文档 + 4 项派生约束 + 3 次强化规则
- **6 维度 + 10 重点核查项**:覆盖 Mavis 自审盲区
- **输出格式结构化**:综合评分 + P0/P1/P2 + 符合项 + 建议
- **强调"不要复述自审"**:防止 AI 走 Mavis 老路
- **可直接复制**:把 [PROMPT] 段下方内容整段贴给 Claude / GPT / Gemini 即可

---

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 10:28 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
