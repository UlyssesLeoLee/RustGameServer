# RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.1 — 项目自我批评与改善方案

> **创建日期**: 2026-09-02 10:18 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **作用域**: RGS 项目治理反思, 全员 (Mavis / 5 域 Lead / batch Lead / SRE / DBA / 评审) 适用
> **配套**: AGENTS.md v0.6 (§9 新增) + STATUS-SNAPSHOT v0.6.40 引用

---

## 0. 触发与背景

Ulysses 2026-09-02 10:18 JST 明确指令"批评一下这个项目, 看看怎么改善"。

Mavis 接手后基于 git 实证数据(commit ahead 193 / 60+ hotfix / 982 .md / 401 .rs / 19 crate / 117,450 md 行 vs 82,915 rs 行) 输出批评, 经 ask_user Q&A 拍板 (A+B+C+D 全选 / 6 域不缩 / 跟踪文档冻结归档), 形成本 v0.1 文档。

**依据**:
- `git status` ahead of origin/main 193 commit
- `git log --oneline | Select-String "hotfix"` 60+ 条 (9/1 当日)
- `Get-ChildItem crates -Recurse -Filter "*.rs"` 401 个 / 82,915 行
- `Get-ChildItem docs -Recurse -Filter "*.md"` 982 个 / 117,450 行
- 单文档最大 RGS-BAS-037 (运维安全生命周期) 264,970 字节
- AGENTS.md v0.5 27,920 字节 / 派生约束 L1-L14
- STATUS-SNAPSHOT v0.6.10 → v0.6.39 (30 次升版)

---

## 1. 现状快照 (git 实证)

| 维度 | 数字 | 出处 |
|---|---|---|
| `.rs` 源文件 | 401 个 / 82,915 行 | `Get-ChildItem crates -Recurse -Filter "*.rs"` |
| `.md` 文档 | 982 个 / 117,450 行 | `Get-ChildItem docs -Recurse -Filter "*.md"` |
| doc/code 行数比 | **1.42 : 1** | 同上 |
| 集成测试文件 | 67 个 (127 test cases) | `tests/` + `*integration_*` + `*ut_*` |
| crate 数 | 19 (6 域 + 7 平台/工具 + 6 边缘) | `Get-ChildItem crates` |
| ahead of origin/main | **193 commit** | `git status` |
| 9/1 当日 hotfix commit | 60+ 条 | `git log --since 2026-09-01 \| Select-String "hotfix"` |
| STATUS-SNAPSHOT 版本 | v0.6.10 → v0.6.39 (30 次升版) | `git log` |
| 单文档最大 | RGS-BAS-037 = **264,970 字节** (~265 KB) | `docs/02-运维安全/RGS-BAS-037_*` |
| AGENTS.md | 27,920 字节 / v0.5 | `AGENTS.md` |
| 跟踪文档体系 | 7 大件 (STATUS-SNAPSHOT / WBS / SESSION-CLOSEOUT / VERIFIER-COMMANDS / DB-CHECKLIST / PHASE-C-SRE-HANDOFF / DEPLOY-HANDOFF) | `docs/00-*/` |
| 工作区脏 | `.worktrees/` + `.worker-tmp/` + `target-bucket-8-*` 未跟踪 + `.test-evidence/2026-08-28-*-v1/v2/v3` 多版本 log | `git status` + `Get-ChildItem` |

---

## 2. 五大问题 (按严重度排序)

### 2.1 治理派压倒实现派 — 文档密度爆炸

**症状**:
- 行数比 1.42:1 (md 行数 117,450 > rs 行数 82,915)
- 单文档失控: RGS-BAS-037 (运维安全生命周期) 265 KB / RGS-BAS-036 (客户端断点续传) 218 KB / RGS-BAS-010 (分布式算法) 141 KB / RGS-BAS-011 (智能体架构) 146 KB — 任何一篇都是小型书的体量
- 9/1 batch 域一天 4 件套 165 KB 落地 (REQ 39 KB + BASIC 37 KB + DETAILED 49 KB + PLAN 43 KB), 速度比实现快 10 倍
- 截至 9/2 10:18 JST, rgs-batch-backend 还在 W2-W6 串行 commit (`faf40a8` L14 → `ea4c874` GAP-10 fix), 文档和实现进度严重错位

**后果**: 新加入者无从下手 / DDD Review 时间被文档篇幅稀释 / 可读性可维护性反向下降

### 2.2 hotfix 文化失控 — 60+ hotfix 形成"自我审计死循环"

**症状**:
- 9/1 一天 60+ hotfix commit, STATUS-SNAPSHOT 从 v0.6.10 升到 v0.6.39 (30 次)
- hotfix 内容大量是"老数字改实时 git log 表达式" (`e517cfa` v0.6.32 / `bce33cb` v0.6.29 / `49944d1` v0.6.30 / `39eded5` v0.6.23)
- L13 终极守护 = "自指字段 deferred 实时查询" — 这条规则本身就是承认"文档自我指涉维护"已变成项目正式工作流
- AGENTS.md v0.1→v0.5 共 5 个版本, 每次 hotfix 都在 L1-L14 之外加新约束: L11 cargo build dir lock / L12 临时 log / L14 plumbing brace 跟踪

**后果**: git log 实际功能 commit 被 hotfix 噪音稀释 / 新约束 L1→L14 继续膨胀就成 L15/L16…, 没有收敛信号

### 2.3 AI 自指悖论 — AI 写、AI 审、AI 修

**症状**:
- 代签完全反转: 所有 RGS-* 文档作者栏 = Ulysses / 审批 = 架构师(Mavis 接手 agent per DEC-008) / 修订人 = Ulysses—Mavis 接手 (per 8/27 19:39/20:56/21:59 三次强化 + 8/26 08:40 反转)
- 派生约束 L1-L14 全是 Mavis 自己立的"不要重复犯 X 错"清单 — L11/L12/L14 都是 8-9 月 Mavis 自己做错的教训
- DDD Review 一审 = Mavis (架构师): 既是写者又是审者, 结构性偏差
- "缺标比错标安全" + "禁回溯叙事" — 这两条本身正确, 但和"派生约束自指"加在一起形成"我加新约束, 我遵守, 我又加新约束"的闭环

**后果**: L1-L14 越多越暴露"流程本身没收敛"的事实 / 同质 hotfix 反复 (STATUS-SNAPSHOT 数字 hotfix 30 次说明规则没真正防住)

### 2.4 工作区卫生 — 派生约束 L12 防不住自己

**症状**:
- `git status` 显示 untracked: `target-bucket-8-phase-b/` / `target-bucket-8-w1-player/` (桶 8 worktree 残留)
- `.worktrees/` + `.worker-tmp/` 仍在主目录 (per L12 派生约束明确"临时 log 不入 commit", 但目录本身没清理)
- `.test-evidence/2026-08-28-*-v1/v2/v3`: 同一个测试 3 个版本 log 全在仓里 (cargo-test-admin-service-v1/v2/v3 各一份), 是"审计过度 + 不清理"的双向问题
- AGENTS.md L12 是 9/1 才加的派生约束, 但同一天 W2 派工时 8 worker 仍写 7 个临时文件到 worktree 根

**后果**: 派生约束写了, 但下次派工仍然犯 — L12 没有 hard hook 兜底

### 2.5 完成定义 (DoD) 偏轻 — 治理指标 ≠ 业务完成

**症状**:
- 当前 DoD = `cargo check --tests 0 error` (per AGENTS.md §2.1) — 编译过 ≠ 业务跑通
- 5 域 ST 业务级 mTLS (commit `401ac5c`) 刚完成; E2E 22 测试函数刚写完 (`82671df` TEST-RUN-PLAN v0.1) — 实际跑通要等 Phase C SRE 介入
- batch 域 commit `82671df` 自己承认 E2E 要 Phase C 介入才能跑: L1 cargo test 限时 60s, E2E 都没真跑过
- RGS-STATUS-SNAPSHOT / RGS-VERIFIER-COMMANDS / RGS-WBS 三套并行跟踪体系 = 治理指标好, 业务"5 域生产可用"未达成

**后果**: 跟踪文档 L1-L14 + 7 大件能 100% 闭环, 但"5 域跨域业务跑通"这个真正的里程碑被延迟

---

## 3. 改善方案 (4 类, 16 条, 拍板结果)

> **拍板结果总览** (per Q&A, 2026-09-02 10:18 JST):
> - **B 类流程自审 (B1-B4)**: 全部 4 条进入 1 周 sprint
> - **C 类业务重排 (C1-C4)**: 3 条采纳 (C1/C2/C3), C4 不采纳 (6 域不缩)
> - **D 类 DoD 升级 (D1-D4)**: 全部 4 条进入 sprint
> - **A 类文档减肥 (A1-A4)**: Q1 未选, 4 条全部进候选清单 (per B2 季度评审机制), 仅 A2 跨类例外 = 实质跟 Q3 跟踪 doc 冻结归档同条

### 3.1 A 类: 文档减肥 (省 30-50% md 行数)

**Q1 拍板**: A 类 4 条全部**未拍板**, 进候选清单 (per B2 季度评审机制)

| ID | 措施 | 收益 | 成本 | 风险 | 状态 |
|---|---|---|---|---|---|
| A1 | RGS-BAS-037 (265KB) 拆 4 份 ≤70KB | 读得动 | 中 (重排) | 引用断链, 需 grep 全文 | **候选清单** |
| A2 | 跟踪文档版本冻结: STATUS-SNAPSHOT v0.6.10-v0.6.25 移 `docs/_archive/`, 只维护 v0.6.26+ | 噪音降 | 低 | 老 commit 引用需 redirect | **跨类拍板 (per Q3, 见 §4.3)** |
| A3 | AGENTS.md 6 个月一归档: 当前 v0.5 → `AGENTS_v0.5_archive.md`, 主 AGENTS.md 只留派生约束 L1-L14 + 拍板规则 | 治理更聚焦 | 低 | 历史回溯需 git log | **候选清单** |
| A4 | `document-registry.toml` 强制登记新 doc 路径 + 大小上限 80KB | 防巨型 doc 再出现 | 低 (改 file) | 流程摩擦 | **候选清单** |

**预期 (A 类若全启)**: md 行数从 117,450 降到 ~70,000 (-40%), 单文档 ≤ 80KB

**注**: A2 实质 = Q3 跟踪 doc 冻结归档, 因此作为 Q3 的子条目落地, 不算 A 类本身拍板. sprint 内执行.

### 3.2 B 类: 流程 / 工具自我审视

**Q1 拍板**: B 类 4 条全部拍板, 进入 1 周 sprint

| ID | 措施 | 收益 | 成本 | 风险 | 状态 |
|---|---|---|---|---|---|
| B1 | 加 `pre-commit` hook 自动 `git worktree list` + `git status --porcelain` 检查 untracked target-* | 防 L12 重犯 | 低 | CI 时间 +5s | **已拍板** |
| B2 | 派生约束 L1-L14 **冻结 6 个月**, 不再加 L15. 新约束进"候选清单"季度评审 | 打破 L 永远增长 | 极低 | 真有重大教训需追溯加 | **已拍板** |
| B3 | **DDD Review 二审 = Ulysses 必审**, Mavis 改稿不写稿; Ulysses 一审前 Mavis 自审 1 次停手 | 打破 AI 自指 | 中 (改流程) | 拖慢 DDD Review | **已拍板** |
| B4 | `.test-evidence/2026-08-28-*` 移到 `docs/_archive/test-evidence-2026-08-28/` 后加 .gitignore 不再跟踪 | 仓库瘦身 | 低 | 审计痕迹丢失, 需先建 _archive/ | **已拍板** |

**预期**: hotfix 频率从 9/1 一天 60+ 降到 <10/天 (剩 hotfix 才有信息量)

### 3.3 C 类: 业务 / 实现优先级重排

**Q1+Q2 拍板**: C1/C2/C3 采纳, C4 不采纳 (6 域不缩 per Q2)

| ID | 措施 | 收益 | 成本 | 风险 | 状态 |
|---|---|---|---|---|---|
| C1 | **冻结 batch 域文档 v0.1**, 等 Phase C 跑通 5 域业务 mTLS 再回到 v0.2 评估 | 集中火力 | 极低 (暂停 commit) | 文档过期 | **已拍板** |
| C2 | **DoD 升级**: 跨域 saga / 5 域主链路 commit 必须 E2E 跑通 (不是 cargo check) | 业务可用 | 高 (补 5 域 E2E) | 跑通前不合并 | **已拍板** |
| C3 | **"派生约束 L1-L14 闭环" ≠ 项目完成**; 新指标 = "5 域 + batch 域生产可用 checklist" | 回归业务 | 中 (改 STATUS-SNAPSHOT 模板) | 治理派有意见 | **已拍板** |
| C4 | **6 域 → 3 域收敛**做 PoC: player / economy / admin 跑通 E2E → 再扩 match / social / batch | 减少并行 | 中 (砍 scope) | 推倒 6 域决策 | **不采纳** (per Q2: 6 域继续不缩) |

**预期**: 里程碑 = "5 域生产可用" 而不是 "7 大跟踪文档 v0.6.39"

### 3.4 D 类: 完成定义 (DoD) 升级

**Q1 拍板**: D 类 4 条全部拍板, 进入 1 周 sprint

| ID | 措施 | 收益 | 成本 | 风险 | 状态 |
|---|---|---|---|---|---|
| D1 | `cargo check --tests` + `cargo test --lib` + 至少 1 E2E 跑通 (三件套) | 业务真验证 | 中 (E2E 补齐) | 时间 | **已拍板** |
| D2 | AGENTS.md §2.1 L1 升级为 L1: cargo check, L1.1: cargo test --lib, L1.2: E2E (Phase C 后必跑) | 清晰分层 | 低 | 派生约束命名规范要调 | **已拍板** |
| D3 | **commit 模板**: `type(scope): summary` + DoD 段 (1-3 行) + Evidence (commit / file:line) | 自描述 | 低 | 老 commit 兼容 | **已拍板** |
| D4 | **每周 status report** 必含 "本周末未达成业务里程碑清单" + "本周末 hotfix 数" 双指标 | 治理/业务并列 | 低 | 报告格式 | **已拍板** |

**预期**: commit 频率 = 业务节奏, 不是 hotfix 节奏

---

## 4. 拍板记录 (per 14:58 JST ask_user 规则)

### 4.1 Q1 优先级 (多选)

| 选项 | 选择 | 备注 |
|---|---|---|
| A 文档减肥 (A1-A4) | ❌ 未选 | 4 条进候选清单 (per B2 季度评审机制), 仅 A2 跨类例外落地 |
| B 流程自审 (B1-B4) | ✅ | 全部 4 条进入 1 周 sprint |
| C 业务重排 (C1-C4) | ✅ | C1/C2/C3 采纳, C4 不采纳 (per Q2 6 域不缩) |
| D DoD 升级 (D1-D4) | ✅ | 全部 4 条进入 1 周 sprint |

**结论**: 3 类全选 (B+C+D) + 6 域不缩 + 跟踪 doc 冻结归档. A 类 4 条进候选清单, 季度评审 (3/2 / 6/2 / 9/2 / 12/2 JST) 走起. 1 周 sprint 内 B+C+D 共 11 条任务 (B1-B4 + C1-C3 + D1-D4) 落地.

### 4.2 Q2 范围 (单选)

**选择**: 6 域继续, 不缩 (推荐)

**理由**: 6 域决策已基于 9/1 Ulysses 拍板 (5 域 + batch), 推倒成本高. 走 C1 (batch v0.1 冻结等 Phase C) + D (DoD 升级), 6 周后业务跑通再扩.

### 4.3 Q3 文档归档 (单选)

**选择**: v0.6.10-v0.6.25 全冻结归档 (推荐)

**理由**: 30 次升版噪音降, 老 commit 引用通过 redirect 处理. 7 大跟踪 doc 体系保留, 但维护窗口 = 最近 14 版本

---

## 5. 落地 checklist (1 周 sprint)

> **范围说明**: A 类 (A1/A3/A4) 未拍板, 不进 1 周 sprint. 仅 A2 (跨类, 实质 = Q3 跟踪 doc 冻结归档) 落地. B+C+D 共 11 条任务在 sprint 内.

### 5.1 Week 1 (2026-09-02 ~ 09-08 JST)

| Day | 任务 | 负责 | DoD | 关联 |
|---|---|---|---|---|
| D1 (9/2) | 跟踪 doc 冻结: STATUS-SNAPSHOT v0.6.10-v0.6.25 移 `docs/_archive/`, 7 大件同步 | Mavis | 7 跟踪 doc 索引更新, 老引用 0 断 | A2 / Q3 |
| D1 (9/2) | AGENTS.md v0.6 升版: 加 §8 (L 冻结) + §9 (本批评与改善), 修订历史加 v0.6 | Mavis | 派生约束 L1-L14 冻结段入档, §9.6 sprint 表对齐拍板 | 本 commit + hotfix |
| D2 (9/3) | B2: 派生约束 L1-L14 冻结 6 个月, 候选清单建档 | Mavis | AGENTS.md L15 入口删, 候选清单 `docs/14-项目治理/L-CANDIDATES.md` | B2 |
| D2 (9/3) | B3: DDD Review 流程改: Ulysses 二审必到, Mavis 一审停手 | Mavis + Ulysses | DDD Review 模板 v0.2 出, 含二审签字栏 | B3 |
| D3 (9/4) | B1: pre-commit hook 加 worktree 残留检查 | Mavis | `.git/hooks/pre-commit` 启用, 阻 untracked target-* | B1 |
| D3 (9/4) | B4: `.test-evidence/2026-08-28-*` 移 `docs/_archive/`, 加 .gitignore | Mavis | 主分支 0 个 v1/v2/v3 老 log | B4 |
| D4 (9/5) | D2: AGENTS.md §2.1 L1 升级为 L1/L1.1/L1.2 三件套 | Mavis | §2.1 重写, commit 模板联动 | D2 |
| D5 (9/6) | C1: batch 域 v0.1 冻结公告, GitHub issue 标 "Phase C 后回归" | Mavis + batch Lead | commit `freeze/batch-v0.1` + issue 入 backlog | C1 |
| D5 (9/6) | D3: commit 模板 `type(scope): summary` + DoD + Evidence | Mavis | `.gitmessage` 入仓, 文档化在 AGENTS.md | D3 |
| D6 (9/7) | A 类 4 条 (A1/A3/A4) 进候选清单, L-CANDIDATES.md 入档 | Mavis | 候选清单首条 = A1 BAS-037 拆分 | A 类 (未拍板) |
| D7 (9/8) | D4 周 status report: 业务里程碑 vs hotfix 数 双指标 | Mavis | `docs/14-项目治理/RGS-WEEKLY-2026-W36.md` | D4 |

### 5.2 Week 2-6 (Phase C 跑通后)

| 周 | 任务 | 备注 |
|---|---|---|
| W2 (9/9-15) | D1: 5 域 E2E 补齐 (per BATCH-PLAN + PHASE-C-SRE-HANDOFF) | 等 Phase C SRE 介入 |
| W3 (9/16-22) | C2: 跨域 saga E2E 跑通, DoD 升级应用 | 22 测试函数真跑 |
| W4 (9/23-29) | C3: 新指标 "生产可用 checklist" 替换 "派生约束闭环" | STATUS-SNAPSHOT v0.7 模板 |
| W5 (9/30-10/6) | batch v0.1 回归, C4 不采纳, 沿用 6 域 | per 拍板 |
| W6 (10/7-13) | 总结: 5 域 + batch v0.2 评估 | 真正业务里程碑 |

---

## 6. 已知缺口 (per 8/26 JST 缺标比错标)

- **A 类 4 条未拍板**: A1/A3/A4 进候选清单 (per B2), 季度评审 (3/2 / 6/2 / 9/2 / 12/2 JST) 走起, 不阻塞 1 周 sprint
- **C4 不采纳**: 6 域 → 3 域 PoC 推倒, 已拍板为不缩 (per Q2)
- **A2 跨类落地**: 实质 = Q3 跟踪 doc 冻结归档, 在 D1 同批处理
- **A1 BAS-037 拆分风险 (候选)**: 4 份后跨引用维护成本 +20%, 1 周 sprint 内未必全完成, 进候选清单
- **B3 DDD Review 二审必到**: Ulysses 时间窗口不定, 拖慢 DDD Review 风险高
- **D1 E2E 跑通**: 等 Phase C SRE 介入, W2 才能真启动
- **A 类若全启, 跨引用维护成本**: 1-2 天额外工时 (per 6/2 季度评审)

---

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 10:18 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 5 大问题 + 4 类方案 (16 条) + 拍板 (A+B+C+D 全选默认错 / 6 域不缩 / 冻结归档) + 1 周 sprint checklist, 7 跟踪 doc 体系 1 周内冻结归档, 派生约束 L1-L14 冻结 6 个月 |
| v0.1.1 | 2026-09-02 10:54 | 架构师(Mavis 接手 agent per DEC-008) | hotfix: Q1 实际拍板 = B+C+D (A 不选). 修正 §3.1 A 类状态 (4 条全部进候选清单, 仅 A2 跨类落地), §3.2-3.4 B/C/D 状态全部标 "已拍板", §4.1 Q1 改 "3 类 (B+C+D)", §5.1 sprint 删 A1/A3/A4 任务 + 加 A 类进候选清单 (D6) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
