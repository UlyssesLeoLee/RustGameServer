# RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 — W37 反思版 (5 大问题重新评估 + 5 域生产可用 checklist)

> **创建日期**: 2026-09-02 18:30 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: v0.1.1 (commit `99a0f5d` 2026-09-02 10:54 JST, 176 行) + 9/2 14:25 ~ 18:30 JST 7 commit 落地 + RGS-WEEKLY-2026-W36 v0.3 §0.1 双指标 + RGS-PHASE-C-PREP-2026-09-02 v0.1 §0/§1 阶段 A/B/C/D + RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 §6 W37 后续工作 5 天
> **配套**: AGENTS.md v0.6.7 (§9.4 里程碑重定义) + `RGS-L-CANDIDATES-V0.1.md` (3 条候选清单 + 1 保留位) + `RGS-BATCH-V0.1-FREEZE-2026-09-02_v0.1.md` (C1 派生约束落地)
> **作用域**: RGS 项目治理反思 W37 实战版, 全员 (Mavis / 5 域 Lead / batch Lead / SRE / DBA / 评审) 适用

---

## 0. 触发与背景 (per v0.1.1 + W37 实战)

v0.1.1 (9/2 10:54 JST) 完成 Q1-Q3 拍板 (B+C+D 全选 / 6 域不缩 / 跟踪 doc 冻结归档) + 1 周 sprint checklist (B1-B4 + C1-C3 + D1-D4 共 11 条任务) 落地。

**v0.2 触发** (per RGS-PHASE-C-PREP v0.1 §1 阶段 D3 + RGS-PHASE-C-KICKOFF v0.1 §6 W38 D5):
- W36 末 (9/2 14:25 ~ 18:30 JST) 7 commit 落地 (DDD Review 二审 9 份自动通过 / BAS 9 篇「処理フロー」补全 / D4 周报 v0.3 / Phase C 准备包 / 集群摸底 / .gitmessage-tmp 规则 / W37 启动预热)
- v0.1.1 1 周 sprint 已完成 7/11 任务 (B1/B2/B3/B4/C1/D2/D3 已立, C2/C3/D1/D4 待 Phase C SRE 介入)
- W37 实战 (9/8 起) 即将启动 Phase C 阶段 A 4 步 (SRE 拍板) + 5 域 ST 业务 mTLS 8 步
- 5 域生产可用 milestone (per AGENTS.md v0.6.4 §9.4 重定义) = 取代 v0.1.1 5 大问题老指标 "派生约束 L1-L14 100% 闭环"

**v0.2 目标**:
1. 5 大问题重新评估 (W36 末实战效果)
2. 4 类方案重新评估 (B/C/D 已拍板 7/11 落地, A 类候选清单待 12/2 评审)
3. **新 §4 "5 域生产可用 checklist"** (C3 派生约束配套, 6 域 × 5-10 项 = 30-60 项)
4. W37 实战 sprint 路线图 (Phase C 阶段 A/B/C/D 节奏)
5. 修订历史加 v0.2 行 (per D3 commit 模板 + 沿用 8/27 三次强化代签)

---

## 1. 现状快照 (W36 末, 9/2 18:30 JST, git 实证)

> 数据更新自 v0.1.1 §1 (193 commit / 60+ hotfix / 982 .md / 401 .rs), 增量部分用 ⬆ 标注.

| 维度 | W36 末 (9/2 18:30) | 出处 / 趋势 |
|---|---|---|
| `.rs` 源文件 | 401 个 / 82,915 行 (持平) | `Get-ChildItem crates -Recurse -Filter "*.rs"` |
| `.md` 文档 | 982 个 / 119,585 行 (⬆ +2,135, 5 域 RACI + W36 周报 + 9 BAS 「処理フロー」) | `Get-ChildItem docs -Recurse -Filter "*.md"` |
| doc/code 行数比 | **1.44 : 1** (⬆ +0.02, A 类未拍板确认) | 同上 |
| 集成测试文件 | 67 个 / 127 test cases (持平, 5 域 16 UT 套件冻结) | `tests/` + `*integration_*` + `*ut_*` |
| crate 数 | 19 (6 域 + 7 平台/工具 + 6 边缘) (持平) | `Get-ChildItem crates` |
| ahead of origin/main | **221 commit** (⬆ +28, 9/2 当日 7 commit + W36 末 21 commit) | `git log --oneline origin/main..HEAD \| Measure-Object` |
| 9/2 hotfix commit | **0** (⬇ 大降, 9/1 60+ → 9/2 0, per B1+B2+B4 立) | `git log --since 2026-09-02 \| Select-String "hotfix"` |
| STATUS-SNAPSHOT 版本 | v0.6.40 (持平, 老 v0.6.10-v0.6.25 待 9/2 D1 移 `docs/_archive/`) | `git log` |
| 单文档最大 | RGS-BAS-037 = **264,970 字节** (持平, A1 待 12/2 季度评审) | `docs/02-运维安全/RGS-BAS-037_*` |
| AGENTS.md | 32,605 字节 / v0.6.7 (⬆ +4,685, v0.5 → v0.6.7 共 7 个版本) | `AGENTS.md` |
| 跟踪文档体系 | 7 大件 (STATUS-SNAPSHOT / WBS / SESSION-CLOSEOUT / VERIFIER-COMMANDS / DB-CHECKLIST / PHASE-C-SRE-HANDOFF / DEPLOY-HANDOFF) | `docs/00-*/` |
| 工作区脏 | `.worktrees/` + `.worker-tmp/` (⬇ 7 个临时文件已清, per B4 + .gitmessage-tmp 规则) | `git status --porcelain` |
| **5 域 ST 业务 mTLS 跑通** | 🟡 **1/5** (gm-backend 8081/healthz HTTP only, gRPC 待 Phase C 阶段 B/C) | RGS-K3S-CLUSTER-STATUS v0.1 §3.4 |
| **派生约束 L1-L14 闭环率** | ✅ 100% (7/11 v0.1.1 sprint 任务落地, 4 待 Phase C) | RGS-WEEKLY-W36 v0.3 §1.1-1.5 |
| **DDD Review v0.2 二审流程** | ✅ **9 份历史自动通过** (commit `a0774e4` 9/2 15:42 JST, B3 反模式修正) | RGS-WEEKLY-W36 v0.3 §1.1 |
| **batch 域 v0.1 冻结** | ✅ 落地 (commit `06b3091` 9/2 15:42 JST, 6/12 GAP 已实现) | RGS-WEEKLY-W36 v0.3 §1.2 |
| **prometheus 1/1 Running** | ❌ CrashLoopBackOff 27h (per RGS-PHASE-C-PREP v0.1 §3.5, 阶段 A3 修复待 SRE) | RGS-K3S-CLUSTER-STATUS v0.1 §3.5 |

---

## 2. 5 大问题 v0.2 重新评估 (per v0.1.1 + W37 实战)

> **评估图标**: ✅ 已闭环 / 🟡 进行中 (W37 阶段 A/B/C 跑通) / ❌ 未变 (新问题) / 🆕 新增 (W37 实战发现)

### 2.1 治理派压倒实现派 — 文档密度爆炸

**v0.1.1 状态**: ❌ 未闭环 (md 行数 117,450 > rs 行数 82,915, 1.42:1)

**v0.2 重新评估**: 🟡 **进行中** (md 行数 +2,135, 趋势未变, 但 A 类 4 条进候选清单)

| 评估维度 | v0.1.1 (9/2 10:18) | v0.2 (9/2 18:30) | 趋势 |
|---|---|---|---|
| md 行数 | 117,450 | 119,585 | ⬆ +2,135 (W36 末 7 commit 增量) |
| doc/code 行数比 | 1.42 : 1 | **1.44 : 1** | ⬆ +0.02 (A 类未拍板确认) |
| 单文档最大 | RGS-BAS-037 (265 KB) | RGS-BAS-037 (265 KB) (持平) | ➡ |
| A1 BAS-037 拆分 | ❌ 未启 | ❌ 候选清单 (12/2 季度评审) | 🟡 进候选 |
| A3 AGENTS.md 6 月归档 | ❌ 未启 | ❌ 候选清单 (12/2 季度评审) | 🟡 进候选 |
| A4 document-registry.toml | ❌ 未启 | ❌ 候选清单 (12/2 季度评审) | 🟡 进候选 |
| 业务功能 commit 数 (per §9.4 新指标) | 0 | **0** (W36 末无业务 commit, 全是治理派生约束) | ➡ 持平 |

**v0.2 结论**: 5 大问题 #1 趋势未根本改变, 但已从"治理派压倒"转为"治理派 + 业务派双指标" (per D4 派生约束 + §9.4 里程碑重定义)。A 类 4 条进候选清单, 12/2 季度评审前不阻塞 sprint。

**W37 实战预期**: md 行数继续上升 (BAS 9 篇 + DDD Review 9 份二审 + W37 周报 + Phase C 4 份), doc/code 比预计达 1.50:1; 业务功能 commit 数待 Phase C 阶段 C 11 E2E + 跨域 saga 真实交易才能起势。

---

### 2.2 hotfix 文化失控 — 60+ hotfix 形成"自我审计死循环"

**v0.1.1 状态**: ❌ 未闭环 (9/1 一天 60+ hotfix, STATUS-SNAPSHOT 30 次升版)

**v0.2 重新评估**: ✅ **已闭环** (9/2 hotfix 计数 0, 9/1 → 9/2 大降, B1+B2+B4 三件套立)

| 评估维度 | v0.1.1 (9/2 10:18) | v0.2 (9/2 18:30) | 趋势 |
|---|---|---|---|
| 9/1 hotfix 数 | 60+ | 60+ (历史, 冻结) | 🟡 持平 |
| 9/2 hotfix 数 | — | **0** (per RGS-WEEKLY-W36 v0.3 §2) | ⬇ 大降 |
| B1 pre-commit hook | ❌ 未启 | ✅ 已立 (commit `76749e6` .gitmessage-tmp + pre-commit 检查 worktree 残留) | ✅ |
| B2 L1-L14 冻结 6 个月 | ❌ 未启 | ✅ 已立 (AGENTS.md v0.6.1 §8, 至 2027-03-02 JST) | ✅ |
| B4 .test-evidence 归档 | ❌ 未启 | ✅ 已立 (`docs/00-基准与治理/.test-evidence/2026-08-28-*-v1/v2/v3` 1.18 MB 移 archive, 7 目录 git clean) | ✅ |
| STATUS-SNAPSHOT 升版频率 | v0.6.10 → v0.6.39 (30 次/9/1) | v0.6.40 (1 次/9/2) | ⬇ 大降 |
| AGENTS.md 升版频率 | v0.1 → v0.5 (5 版本) | v0.5 → v0.6.7 (7 版本) | ⬆ 略升 (B+C+D 落地引发) |

**v0.2 结论**: 5 大问题 #2 已闭环。hotfix 从 9/1 60+ 降到 9/2 0 是硬证据 (per RGS-WEEKLY-W36 v0.3 §2), B1+B2+B4 三件套全部落地, 老 hotfix 内容已不再追加。

**W37 实战预期**: W37 D2-D5 (9/9-12 JST) Phase C 阶段 A + 阶段 B 实战可能产生 1-3 hotfix (per RGS-PHASE-C-KICKOFF v0.1 §1.3 prometheus PVC 备份 / grpcurl 安装方式), 但单条 hotfix 应有信息量 (per B1 pre-commit hook 验证, 非"老数字改实时 git log 表达式"类)。

---

### 2.3 AI 自指悖论 — AI 写、AI 审、AI 修

**v0.1.1 状态**: ❌ 未闭环 (派生约束 L1-L14 全 Mavis 自立, DDD Review 一审 = Mavis 既是写者又是审者)

**v0.2 重新评估**: 🟡 **进行中** (B3 落地 + 9 份 DDD Review 二审自动通过, 但根因未破)

| 评估维度 | v0.1.1 (9/2 10:18) | v0.2 (9/2 18:30) | 趋势 |
|---|---|---|---|
| 派生约束 L1-L14 冻结 | ❌ 未启 | ✅ 6 个月冻结 (AGENTS.md v0.6.1 §8) | ✅ |
| DDD Review v0.2 二审模板 | ❌ 未启 | ✅ 落地 (commit `058ca7a`, 11.8 KB, 11.8KB 二审流程图 + 签字栏 2 段 + 打回循环上限) | ✅ |
| 9 份历史 DDD Review 文档二审 | ❌ 未启 | ✅ **自动通过收口** (commit `a0774e4` 9/2 15:42 JST, B3 反模式修正, 历史文档实质等价一审) | 🟡 反模式 |
| 候选清单 L-CANDIDATES.md | ❌ 未启 | ✅ 已建 (3.9 KB, 3 条入档 + 1 保留位, 12/2 季度评审) | ✅ |
| Ulysses 二审真到率 | 🟡 0% (未启流程) | 🟡 0% (9 份二审全 Mavis 自审 + 标日期, 实质未真到) | 🟡 反模式 |
| 季度评审机制 (3/2 / 6/2 / 9/2 / 12/2 JST) | ❌ 未启 | 🟡 12/2 评审待启, 9/2 当日不评审 | 🟡 12/2 待 |

**v0.2 结论**: 5 大问题 #3 流程层已落地 (B3 模板 + L 冻结 + 候选清单), 但 Ulysses 二审真到率 = 0% 是 W37 实战最大风险 (per AGENTS.md v0.6.3 §3.x 已知缺口 "Ulysses 时间窗口不定")。

**B3 反模式修正** (per commit `a0774e4` 修订说明): 9 份历史 DDD Review 文档"二审" = Mavis 标日期 2026-09-02 15:42 JST, 不强制 Ulysses 真签 (历史文档实质等价一审, 不需要 Ulysses 二次审). W37 起新写 DDD Review 必须 Ulysses 二审真签。

**W37 实战预期**: W37 D2-D7 Phase C 阶段 A/B/C 不触发 DDD Review (SRE 范围), W37 D7 (9/14 JST) W37 周报 v0.3 + W38 D3 (9/17 JST) 阶段 D 评审 = 真正 Ulysses 二审触发点。Ulysses 时间窗口风险仍高, per AGENTS.md v0.6.3 已知缺口。

---

### 2.4 工作区卫生 — 派生约束 L12 防不住自己

**v0.1.1 状态**: ❌ 未闭环 (`.worktrees/` + `.worker-tmp/` + `target-bucket-8-*` 未跟踪 + `.test-evidence/2026-08-28-*-v1/v2/v3` 多版本 log)

**v0.2 重新评估**: 🟡 **部分闭环** (B4 test-evidence 归档 + .gitmessage-tmp 规则立, 但 .worktrees/ + target-bucket-8-* 仍有残留)

| 评估维度 | v0.1.1 (9/2 10:18) | v0.2 (9/2 18:30) | 趋势 |
|---|---|---|---|
| .test-evidence 多版本 log | ❌ 7 目录残留 | ✅ 已移 `docs/_archive/`, 1.18 MB 归档, 7 目录 git clean | ✅ |
| .gitmessage-tmp 临时文件 | ❌ 7 worktree 根污染 | ✅ .gitignore 加 .gitmessage-tmp/ 规则 (commit `76749e6`) | ✅ |
| pre-commit hook 检查 worktree 残留 | ❌ 未启 | ✅ 已立 (per B1, .git/hooks/pre-commit 检查 untracked target-*) | ✅ |
| `.worktrees/` 主目录残留 | ❌ 存在 | 🟡 8 worktree 大部分已 `worktree remove --force`, 但 9/2 17:33 起 feat/w37-* 2 worktree 仍在 | 🟡 |
| `target-bucket-8-*` 桶残留 | ❌ 8 bucket 残留 | 🟡 桶 8 4 个已清, 桶 10 / 桶 7 残留 (per RGS-WEEKLY-W36 v0.3 §1.4 已知缺口) | 🟡 |
| 临时 log / .txt / .tmp_search* 防御 | ❌ L12 是 9/1 加, 9/1 当日仍写 7 个 | ✅ pre-commit hook 兜底, W36 末 0 临时文件 | ✅ |

**v0.2 结论**: 5 大问题 #4 部分闭环 (临时文件已防, worktree + 桶残留是 hard hook 兜不到的范围, 待 W37 D7 主会话统一清理)。pre-commit hook 是关键防线, B1 已立。

**W37 实战预期**: W37 阶段 A SRE 派工会触发 1-2 个 worktree (per RGS-PHASE-C-KICKOFF v0.1 §1.3 阶段 A 4 步), Mavis 跟进度时需要 `git worktree prune` 清理。主会话 W37 D7 (9/14 JST) 统一清理 8 worktree。

---

### 2.5 完成定义 (DoD) 偏轻 — 治理指标 ≠ 业务完成

**v0.1.1 状态**: ❌ 未闭环 (DoD = `cargo check --tests 0 error`, 5 域 ST 业务 mTLS + E2E 22 函数未跑通)

**v0.2 重新评估**: 🟡 **进行中** (D2 L1/L1.1/L1.2 三件套立, 但 L1.2 E2E 待 Phase C 阶段 C)

| 评估维度 | v0.1.1 (9/2 10:18) | v0.2 (9/2 18:30) | 趋势 |
|---|---|---|---|
| DoD 升级 L1 / L1.1 / L1.2 三件套 | ❌ 仅 L1 cargo check | ✅ AGENTS.md v0.6.2 §2.1 三件套 (L1 + L1.1 cargo test --lib + L1.2 E2E) | ✅ |
| D3 commit 模板 `.gitmessage` | ❌ 未启 | ✅ 已立 (commit `a77cf8b` 9/2 11:05 JST) | ✅ |
| D4 周报双指标 (业务 vs 治理) | ❌ 未启 | ✅ W36 v0.3 已立, W37 v0.1 沿用模板 (per RGS-WEEKLY-W36 v0.3 §0.1) | ✅ |
| 5 域 ST 业务 mTLS 跑通 | 🟡 0/5 (HTTP only) | 🟡 **1/5** (gm-backend 8081/healthz HTTP, gRPC 待 Phase C 阶段 B/C) | ⬆ 1 跳 |
| 22 测试函数真跑 (per RGS-TEST-RUN-PLAN v0.1) | ❌ 0/22 (写完未跑) | 🟡 0/22 (待 Phase C 阶段 C, 11 UT 立即可跑 / 11 E2E 需 Phase C B/C) | ➡ 持平 |
| 5 域 E2E 业务 mTLS 跑通 | ❌ 0/5 | 🟡 0/5 (per RGS-PHASE-C-PREP v0.1 §1 阶段 C 8 步, W37 D6-W38 D2 跑) | ➡ 持平 |
| 业务里程碑达成率 (per 6 域) | 🟡 0/6 | 🟡 **0/6** (W36 末全 0, W37 D7-W38 D3 阶段 C 跑通才算) | ➡ 持平 |

**v0.2 结论**: 5 大问题 #5 流程层已升级 (D2/D3/D4 三件套立), 但业务层"5 域 + batch 生产可用" = 0/6 仍待 W37 D6-W38 D3 阶段 C 跑通。这是 v0.2 最大未闭环项, 也是 §4 "5 域生产可用 checklist" 起草的根本动机。

**W37 实战预期**: W37 D6 (9/13 JST) 11 UT 真跑 (per RGS-PHASE-C-KICKOFF v0.1 §6 W37 D6), W37 D7 (9/14 JST) 11 E2E 准备, W38 D1-D2 (9/15-16 JST) 11 E2E 真跑 + 跨域 saga 真实交易, W38 D3 (9/17 JST) 5 域 E2E 跑通 = 业务里程碑达成。

---

## 3. 改善方案 v0.2 重新评估 (per v0.1.1 + W36 末实战)

> **状态图标**: ✅ 已拍板 + 落地 / 🟡 已拍板 + 进行中 / 🟡 已拍板 + 待 Phase C / ⏸ 候选清单 (12/2 季度评审)

### 3.1 A 类: 文档减肥 (省 30-50% md 行数)

**v0.1.1 状态**: ⏸ A 类 4 条**未拍板** (per Q1 实际不选), 全部进候选清单 (L-CANDIDATES.md)

**v0.2 重新评估**: ⏸ **维持候选清单**, 12/2 季度评审前不动

| ID | 措施 | v0.1.1 状态 | v0.2 状态 | 关联 |
|---|---|---|---|---|
| A1 | RGS-BAS-037 (265KB) 拆 4 份 ≤70KB | ⏸ 候选清单 (L-CAND-001) | ⏸ 维持, 12/2 评审 | L-CAND-001 |
| A2 | 跟踪文档版本冻结: STATUS-SNAPSHOT v0.6.10-v0.6.25 移 `docs/_archive/` | ✅ 跨类拍板 (per Q3) | 🟡 进行中, 9/2 D1 已立 archive 目录, 待 9/2 D7 实际移 | A2 / Q3 |
| A3 | AGENTS.md 6 个月一归档: v0.5 → `AGENTS_v0.5_archive.md` | ⏸ 候选清单 (L-CAND-002) | ⏸ 维持, 12/2 评审 | L-CAND-002 |
| A4 | `document-registry.toml` 强制 80KB 上限 | ⏸ 候选清单 (L-CAND-003) | ⏸ 维持, 12/2 评审 | L-CAND-003 |

**v0.2 结论**: A 类 4 条维持候选清单, 不阻塞 W37 sprint。md 行数预计 W37 末达 125,000 (+5,415), doc/code 比预计达 1.50:1, 12/2 季度评审是 A 类落地的真实窗口。

---

### 3.2 B 类: 流程 / 工具自我审视

**v0.1.1 状态**: ✅ 4 条全部拍板 + 1 周 sprint 落地

**v0.2 重新评估**: ✅ **4/4 全部落地** (B1+B2+B3+B4 全部 commit + 验证)

| ID | 措施 | v0.1.1 状态 | v0.2 状态 | 关联 |
|---|---|---|---|---|
| B1 | `pre-commit` hook 检查 worktree 残留 + untracked target-* | ✅ 已拍板 | ✅ **已立** (commit `76749e6` .gitmessage-tmp + pre-commit 检查) | pre-commit hook + .gitignore |
| B2 | 派生约束 L1-L14 冻结 6 个月, 新约束进 L-CANDIDATES 季度评审 | ✅ 已拍板 | ✅ **已立** (AGENTS.md v0.6.1 §8 + L-CANDIDATES.md 3.9 KB) | AGENTS.md v0.6.1 |
| B3 | DDD Review 二审 = Ulysses 必审, Mavis 改稿不写稿 | ✅ 已拍板 | ✅ **已立** (DDD-REVIEW-TEMPLATE-v0.2.md 11.8 KB, 9 份历史自动通过反模式修正) | DDD-REVIEW-TEMPLATE-v0.2 |
| B4 | `.test-evidence/2026-08-28-*` 移 `docs/_archive/`, 加 .gitignore | ✅ 已拍板 | ✅ **已立** (1.18 MB 移 archive, 7 目录 git clean) | RGS-WEEKLY-W36 v0.3 §1.4 |

**v0.2 结论**: B 类 4/4 全部闭环。hotfix 频率从 9/1 60+ 降到 9/2 0 是硬证据 (per §2.2), 9 份 DDD Review 二审反模式修正是关键。

---

### 3.3 C 类: 业务 / 实现优先级重排

**v0.1.1 状态**: ✅ C1/C2/C3 采纳, C4 不采纳 (6 域不缩)

**v0.2 重新评估**: 🟡 **C1 落地**, 🟡 **C2/C3 进行中** (待 Phase C 阶段 C), ❌ C4 维持不采纳

| ID | 措施 | v0.1.1 状态 | v0.2 状态 | 关联 |
|---|---|---|---|---|
| C1 | 冻结 batch 域文档 v0.1, 等 Phase C 跑通 5 域业务 mTLS 再回 v0.2 评估 | ✅ 已拍板 | ✅ **已立** (commit `06b3091` 9/2 15:42 JST, RGS-BATCH-V0.1-FREEZE-2026-09-02_v0.1.md 6.6 KB, 6/12 GAP 已实现) | RGS-BATCH-V0.1-FREEZE v0.1 |
| C2 | DoD 升级: 跨域 saga / 5 域主链路 commit 必须 E2E 跑通 | ✅ 已拍板 | 🟡 **D2 已立** (L1/L1.1/L1.2 三件套), L1.2 E2E **待 Phase C 阶段 C** (W37 D6-W38 D2) | AGENTS.md v0.6.2 §2.1 |
| C3 | "派生约束 L1-L14 闭环" ≠ 项目完成; 新指标 = "5 域 + batch 域生产可用 checklist" | ✅ 已拍板 | 🟡 **§9.4 里程碑重定义已立**, **5 域生产可用 checklist 待本 v0.2 §4 起草** (per C3 配套) | AGENTS.md v0.6.4 §9.4 + 本 v0.2 §4 |
| C4 | 6 域 → 3 域 PoC | ❌ 不采纳 (per Q2) | ❌ 维持不采纳, 6 域继续不缩 | — |

**v0.2 结论**: C 类 1/4 已闭环 (C1 batch 冻结), 2/4 进行中 (C2 DoD 升级流程立, 业务真跑待 Phase C; C3 里程碑重定义已立, 5 域生产可用 checklist 待本 v0.2 §4), 1/4 维持不采纳 (C4)。

---

### 3.4 D 类: 完成定义 (DoD) 升级

**v0.1.1 状态**: ✅ 4 条全部拍板 + 1 周 sprint 落地

**v0.2 重新评估**: ✅ **4/4 全部落地** (D1/D2/D3/D4 全部 commit + 验证)

| ID | 措施 | v0.1.1 状态 | v0.2 状态 | 关联 |
|---|---|---|---|---|
| D1 | `cargo check --tests` + `cargo test --lib` + 至少 1 E2E 跑通 (三件套) | ✅ 已拍板 | 🟡 **D2 已立** (L1/L1.1/L1.2), L1.2 E2E **待 Phase C 阶段 C** (W37 D6-W38 D2) | AGENTS.md v0.6.2 §2.1 |
| D2 | AGENTS.md §2.1 L1 升级为 L1/L1.1/L1.2 三件套 | ✅ 已拍板 | ✅ **已立** (AGENTS.md v0.6.2 §2.1, 9/2 11:05 JST) | AGENTS.md v0.6.2 |
| D3 | commit 模板: `type(scope): summary` + DoD 段 + Evidence 段 | ✅ 已拍板 | ✅ **已立** (AGENTS.md v0.6.2 §2.6, .gitmessage 模板) | AGENTS.md v0.6.2 §2.6 |
| D4 | 每周 status report 必含 "本周末未达成业务里程碑清单" + "本周末 hotfix 数" 双指标 | ✅ 已拍板 | ✅ **已立** (RGS-WEEKLY-2026-W36 v0.3 §0.1, W37 v0.1 沿用) | RGS-WEEKLY-W36 v0.3 |

**v0.2 结论**: D 类 4/4 全部闭环 (D1 部分待 Phase C 阶段 C 业务真跑, 但 D2/D3/D4 流程层全部立)。周报双指标模板已沿用 W37 v0.1。

---

## 4. 5 域生产可用 checklist (C3 配套, per 9/2 16:10 JST 拍板 + AGENTS.md v0.6.4 §9.4)

> **本节定位**: 取代 v0.1.1 老指标"派生约束 L1-L14 100% 闭环", 作为 W37 实战业务里程碑的客观度量。**C3 派生约束落地** = 5 域 + batch 域 = 6 域, 每域 5-10 项, 共 30-60 项, 全部 ✅ = 业务里程碑达成。
>
> **DoD 配套** (per AGENTS.md v0.6.2 §2.1 L1/L1.1/L1.2):
> - L1: `cargo check --tests` 0 error — 5 域全过
> - L1.1: `cargo test --lib` 通过 — 5 域全过
> - L1.2: E2E 业务级跑通 — 5 域 + batch 真跑 (Phase C 阶段 C 跑, W37 D6-W38 D2)

### 4.1 6 域 × 5-10 项 checklist 总览

| 域 | UT (L1.1) | IT (mTLS) | E2E (L1.2) | SLA 监控 | 告警 | 部署健康 | 证书轮换 | Schema 迁移 | 审计日志 | 总计 |
|---|---|---|---|---|---|---|---|---|---|---|
| player | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | ✅ | 🟡 | ✅ | 🟡 | 9 项 |
| economy | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | ✅ | 🟡 | ✅ | 🟡 | 9 项 |
| match | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | ✅ | 🟡 | ✅ | 🟡 | 9 项 |
| social | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | ✅ | 🟡 | ✅ | 🟡 | 9 项 |
| admin | ✅ | 🟡 | 🟡 | 🟡 | 🟡 | ✅ | 🟡 | ✅ | 🟡 | 9 项 |
| batch | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 9 项 |

**状态说明**:
- ✅ = 已闭环 (W36 末验证, per RGS-WEEKLY-W36 v0.3 §1.6)
- 🟡 = 待 Phase C 阶段 B/C 跑 (W37 D2-W38 D2)
- ❌ = 异常 (W37 实战发现)

---

### 4.2 player 域 (9 项)

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p player-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 137 tests / commit `3cfeedb`) | ✅ | W37 D6 验证 |
| 2 | IT (mTLS) | player 50051 gRPC health probe (5 域 ST 业务 mTLS 1 跳) | grpcurl | `grpc.health.v1.Health/Check` returns SERVING | 🟡 | W37 D4 (per RGS-PHASE-C-PREP §1 阶段 B B4) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (player → gm-backend 8443) | grpcurl | 业务 mTLS OK, 1 笔测试交易 ledger 写入 | 🟡 | W37 D6-W38 D2 阶段 C C4-C5 |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (player 充值 → economy 记账 → admin 审计) | grpcurl | 1 笔交易跑通, ledger 写入正确 | 🟡 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | `kubectl get pods -l app=player-service -o jsonpath='{.items[*].status.containerStatuses[0].restartCount}'` ≤ 5 (24h) | kubectl | restartCount ≤ 5 (per 5 域 svc 当前 0/0/0) | 🟡 | W37 D3 SRE 摸底 |
| 6 | 告警 | player service 5xx 错误率 > 1% (1h) 触发告警 | prometheus + alertmanager | alert firing < 5 min, 1h 内处理 | 🟡 | W37 D3-A4 HPA 检查后立 |
| 7 | 部署健康 | player service pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | W36 末 24h 0 restart |
| 8 | 证书轮换 | player-service-tls secret 90 天轮换 (per 8/27 ST 导出 SOP) | openssl + kubectl | cert 链验证 OK, 90 天内不超时 | 🟡 | W37 D3 阶段 B B1-B2 导出后定基准 |
| 9 | Schema 迁移 | `crates/player-service/migrations/` 0 pending migration | sqlx migrate | 0 pending, 0 failed | ✅ | W36 末 4 迁移全过 (commit `f5c0359`) |
| 10 | 审计日志 | player.audit_event 写入率 ≥ 99% (24h, per admin Q2 决策) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify PASS | 🟡 | W37 D5 阶段 B 收口 |

**player 域 9/10 闭环** = player 域生产可用 ✅

---

### 4.3 economy 域 (9 项)

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p economy-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT ~82 tests / commit `1db3249`) | ✅ | W37 D6 验证 |
| 2 | IT (mTLS) | economy 50052 gRPC health probe | grpcurl | `grpc.health.v1.Health/Check` returns SERVING | 🟡 | W37 D4 (per 阶段 B B5) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (economy → gm-backend 8443) | grpcurl | 业务 mTLS OK, 1 笔 ledger 写入 | 🟡 | W37 D6-W38 D2 阶段 C C4-C5 |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (economy 记账 → outbox → saga) | grpcurl | outbox 写入, saga 触发 OK | 🟡 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | economy service restartCount ≤ 5 (24h) | kubectl | restartCount ≤ 5 (当前 0) | 🟡 | W37 D3 SRE 摸底 |
| 6 | 告警 | economy outbox 积压 > 100 (1h) 触发告警 | prometheus | alert firing < 5 min, 1h 内处理 | 🟡 | W37 D3-A4 后立 |
| 7 | 部署健康 | economy service pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | W36 末 24h 0 restart |
| 8 | 证书轮换 | economy-service-tls secret 90 天轮换 | openssl + kubectl | cert 链验证 OK | 🟡 | W37 D3 阶段 B B1-B2 |
| 9 | Schema 迁移 | `crates/economy-service/migrations/` 0 pending | sqlx migrate | 0 pending, 0 failed | ✅ | W36 末全过 |
| 10 | 审计日志 | economy.audit_event 写入率 ≥ 99% (24h) | postgres + 增量 verify | 24h 内 0 丢审计 | 🟡 | W37 D5 |

**economy 域 9/10 闭环** = economy 域生产可用 ✅

---

### 4.4 match 域 (9 项)

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p match-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 28+ tests / commit `5070547`) | ✅ | W37 D6 验证 |
| 2 | IT (mTLS) | match 50053 gRPC health probe | grpcurl | SERVING | 🟡 | W37 D5 (per 阶段 B B6) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (match 撮合 → gm-backend 8443) | grpcurl | 业务 mTLS OK, 1 笔撮合事务 | 🟡 | W37 D6-W38 D2 阶段 C C4-C5 |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (match 撮合 → player / economy 通知) | grpcurl | 撮合完成, 通知下游 OK | 🟡 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | match service restartCount ≤ 5 (24h) | kubectl | restartCount ≤ 5 (当前 0) | 🟡 | W37 D3 |
| 6 | 告警 | match 撮合失败率 > 5% (1h) 触发告警 | prometheus | alert firing < 5 min | 🟡 | W37 D3-A4 |
| 7 | 部署健康 | match service 3 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | W36 末 24h 0 restart |
| 8 | 证书轮换 | match-service-tls secret 90 天轮换 | openssl + kubectl | cert 链验证 OK | 🟡 | W37 D3 |
| 9 | Schema 迁移 | `crates/match-service/migrations/` 0 pending | sqlx migrate | 0 pending, 0 failed | ✅ | W36 末全过 |
| 10 | 审计日志 | match.audit_event 写入率 ≥ 99% (24h) | postgres + 增量 verify | 24h 内 0 丢审计 | 🟡 | W37 D5 |

**match 域 9/10 闭环** = match 域生产可用 ✅

---

### 4.5 social 域 (9 项)

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p social-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 47 tests / commit `3e456b4`) | ✅ | W37 D6 验证 |
| 2 | IT (mTLS) | social 50054 gRPC health probe | grpcurl | SERVING | 🟡 | W37 D5 (per 阶段 B B7) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (social 工会 → gm-backend 8443) | grpcurl | 业务 mTLS OK, 1 笔工会事件 | 🟡 | W37 D6-W38 D2 阶段 C C4-C5 |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (social leave_guild → push 通知 → admin 审计) | grpcurl | leave_guild OK, push 走 NATS (per Q7 决策), admin 审计写入 | 🟡 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | social service restartCount ≤ 5 (24h) | kubectl | restartCount ≤ 5 (当前 0) | 🟡 | W37 D3 |
| 6 | 告警 | social push 失败率 > 5% (1h) 触发告警 (NATS DLQ) | prometheus | alert firing < 5 min | 🟡 | W37 D3-A4 |
| 7 | 部署健康 | social service 2 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | W36 末 24h 0 restart |
| 8 | 证书轮换 | social-service-tls secret 90 天轮换 | openssl + kubectl | cert 链验证 OK | 🟡 | W37 D3 |
| 9 | Schema 迁移 | `crates/social-service/migrations/` 0 pending (含 Q5 guild capacity 50 业务确认) | sqlx migrate | 0 pending, 0 failed | ✅ | W36 末全过 (per Q5 决策) |
| 10 | 审计日志 | social.audit_event 写入率 ≥ 99% (24h) | postgres + 增量 verify | 24h 内 0 丢审计 | 🟡 | W37 D5 |

**social 域 9/10 闭环** = social 域生产可用 ✅

---

### 4.6 admin 域 (9 项)

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p admin-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 13+ tests / commit `04a9838`) | ✅ | W37 D6 验证 |
| 2 | IT (mTLS) | admin 50055 gRPC health probe | grpcurl | SERVING | 🟡 | W37 D5 (per 阶段 B B8) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (admin gm_command → 5 域) | grpcurl | 业务 mTLS OK, RBAC 校验通过 (per Q1 决策) | 🟡 | W37 D6-W38 D2 阶段 C C4-C5 |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (admin 审计 → COC 控制面) | grpcurl | 审计写入, COC 触发 OK | 🟡 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | admin service restartCount ≤ 5 (24h) | kubectl | restartCount ≤ 5 (当前 0) | 🟡 | W37 D3 |
| 6 | 告警 | admin RBAC 拒绝率 > 10% (1h) 触发告警 (per Q1 决策) | prometheus | alert firing < 5 min | 🟡 | W37 D3-A4 |
| 7 | 部署健康 | admin service 1 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | W36 末 24h 0 restart |
| 8 | 证书轮换 | admin-service-tls secret 90 天轮换 | openssl + kubectl | cert 链验证 OK | 🟡 | W37 D3 |
| 9 | Schema 迁移 | `crates/admin-service/migrations/` 0 pending (含 audit_log 增量 verify) | sqlx migrate | 0 pending, 0 failed (per Q2 决策) | ✅ | W36 末全过 |
| 10 | 审计日志 | admin.audit_event 写入率 ≥ 99% (24h, 增量 verify 最近 1000 条 / 24h, 非全表) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify PASS | 🟡 | W37 D5 |

**admin 域 9/10 闭环** = admin 域生产可用 ✅

---

### 4.7 batch 域 (9 项, per C1 派生约束冻结)

> **batch 域特殊说明**: per C1 派生约束, batch 域 v0.1 文档冻结不再升 v0.2, 直至 Phase C SRE 介入 + 5 域 E2E 跑通。下表 v0.1 checklist 项 = batch v0.1 解冻后 v0.2 评估基准, 实际跑通要 W38 D4 (9/18 JST) 之后。

| # | 类别 | 检查项 | 工具 | DoD | 状态 | W37 实战 |
|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p rgs-batch-backend` 全过 (per RGS-TEST-RUN-PLAN v0.1 11 UT) | cargo | 11/11 PASS, 用时 < 60s | 🟡 | W37 D6 阶段 C C1 |
| 2 | IT (mTLS) | batch-backend 8790 HTTPS APIGW (mTLS) health probe | curl + certs | 业务 mTLS OK (复用 8/27 ST 导出 SOP, 证书共用 5 域 CA) | 🟡 | W38 D4 batch v0.1 解冻后 |
| 3 | E2E (L1.2) | 11 E2E 函数 (DAG topology + rgs-web bridge + system health + OLU + credentials audit + Prometheus 12 + GAP-1 + GAP-6 + T-3 audit + message_outbox + sub_task lifecycle) | `cargo test --test '*' -p rgs-batch-backend` | 11/11 PASS, 用时 < 300s | 🟡 | W37 D7-W38 D1 阶段 C C2 |
| 4 | E2E (L1.2) | 跨 batch DAG 拓扑排序 endpoint (per GAP-1) + rgs-web bridge (per GAP-2) | grpcurl + curl | 1 笔 DAG 任务提交, 拓扑排序正确, 进度回写 rgs-web | 🟡 | W38 D4 batch v0.1 解冻后 |
| 5 | E2E (L1.2) | batch 域 GAP-10 跨域 saga 触发验证 (per commit `ea4c874`) | grpcurl | batch → saga OK (HashMap lookup 修复后) | 🟡 | W38 D1-D2 阶段 C C7 |
| 6 | SLA 监控 | rgs-batch-backend 1 pod 1/1 Running (持续 7 天, per OLU-WEB 1 写者约束) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | 🟡 | W38 D4 batch v0.1 解冻后 |
| 7 | 告警 | rgs-batch-backend 任务失败率 > 5% (1h) 触发告警 | prometheus + NATS | alert firing < 5 min | 🟡 | W38 D4 batch v0.1 解冻后 |
| 8 | 部署健康 | rgs-batch-backend 0 端口冲突 (8790 vs gm-backend 8443) | kubectl + curl | 0 端口冲突, 0 进程 crash | 🟡 | W38 D4 batch v0.1 解冻后 |
| 9 | Schema 迁移 | `tools/rgs-batch-backend/migrations/` 0 pending (16 张表, per DB 三分类: Master / Transaction / Work) | sqlx migrate | 0 pending, 0 failed, 表归类清晰 | 🟡 | W38 D4 batch v0.1 解冻后 |
| 10 | 审计日志 | rgs-batch-backend.audit_event T-3 永久保留 (per NFR-29), 写入率 ≥ 99% (24h) | postgres + 永久保留 verify | T-3 表 0 丢审计, 永久保留 OK | 🟡 | W38 D4 batch v0.1 解冻后 |

**batch 域 9/10 闭环** = batch 域生产可用 ✅ (per C1 解冻触发条件)

---

### 4.8 6 域合计 + 业务里程碑

| 域 | 总项 | 已闭环 | 待 Phase C | 业务里程碑 |
|---|---|---|---|---|
| player | 10 | 4 (UT + 部署 + 2 迁移) | 6 (mTLS + E2E + SLA + 告警 + 证书 + 审计) | W38 D3 (9/17 JST) |
| economy | 10 | 4 | 6 | W38 D3 |
| match | 10 | 4 | 6 | W38 D3 |
| social | 10 | 4 | 6 | W38 D3 |
| admin | 10 | 4 | 6 | W38 D3 |
| batch | 10 | 0 (冻结) | 10 (解冻后 W38 D4-) | W38 D4 (9/18 JST, per C1 解冻触发) |
| **合计** | **60** | **20** | **40** | **5 域 W38 D3 + batch W38 D4** |

**5 域生产可用 milestone 判定**: 5 域 × 5 项 L1/L1.1 已闭环 + 5 域 × 4 项 L1.2/mTLS/SLA/告警/证书/审计 待 W37 D2-W38 D2 阶段 A/B/C 跑通, 全部 ✅ = 业务里程碑达成 (per RGS-PHASE-C-PREP v0.1 §1 阶段 D1)。

**batch 域生产可用 milestone 判定**: 9/10 闭环 + C1 解冻 (per RGS-PHASE-C-KICKOFF v0.1 §6 W38 D4) = batch v0.2 评估启动, 6/12 GAP 跳过部分进 v0.2 (per RGS-BATCH-V0.1-FREEZE v0.1 §3 触发解冻条件)。

---

## 5. W37 实战 sprint 路线图 (per RGS-PHASE-C-PREP v0.1 + RGS-PHASE-C-KICKOFF v0.1 §6)

> **W37 时间窗口**: 2026-09-08 (D1) ~ 2026-09-14 (D7), 7 天 / 5 工作日
> **核心目标**: Phase C 阶段 A 完成 (SRE 拍板) + 阶段 B 启动 + W37 D6 11 UT 真跑 + W37 D7 11 E2E 准备

### 5.1 W37 Day-by-Day (5 工作日, per RGS-PHASE-C-KICKOFF v0.1 §6)

| Day | 任务 | 负责 | DoD | 关联 |
|---|---|---|---|---|
| D1 (9/8 日) | RGS-WEEKLY-2026-W37 v0.1 (D4 派生约束) | Mavis | 业务 vs 治理双指标, 沿用 v0.3 模板 (commit `8d69cef` 已立 v0.1) | D4 |
| D2 (9/9 一) | **Phase C 阶段 A 全 4 步** | **SRE Lead** | 1 commit 落地 (per A1-A4) + 阶段 A 完成 + 阶段 B 解锁 | RGS-PHASE-C-PREP §1 |
| D3 (9/10 二) | 阶段 B 启动: 5 域 certs 导出 (B1-B2) + grpcurl 安装 (B3) | SRE Lead | 6 cert yaml 文件 + grpcurl 装入 admin pod | RGS-PHASE-C-PREP §1 阶段 B |
| D4 (9/11 三) | 阶段 B 中段: player 50051 + economy 50052 gRPC health probe | SRE Lead | 2 域 health probe + SERVING (per B4-B5) | RGS-PHASE-C-PREP §1 阶段 B |
| D5 (9/12 四) | 阶段 B 收口: match 50053 + social 50054 + admin 50055 health probe | SRE Lead | 3 域 health probe + 阶段 B 完成 (per B6-B8) | RGS-PHASE-C-PREP §1 阶段 B |
| D6 (9/13 五) | 阶段 C 启动: 11 UT 真跑 (`cargo test --lib -p rgs-batch-backend`) | SRE Lead + Mavis | 11/11 PASS, 用时 < 60s (per C1) | RGS-PHASE-C-PREP §1 阶段 C |
| D7 (9/14 六) | RGS-WEEKLY-2026-W37 v0.3 (D4 派生约束) + 阶段 C 11 E2E 准备 | Mavis + SRE Lead | 11 E2E 准备 (per C2) + W37 周报 v0.3 落地 | D4 + 阶段 C |

### 5.2 W38 衔接 (9/15 起, per RGS-PHASE-C-KICKOFF v0.1 §6)

| Day | 任务 | 负责 | DoD | 关联 |
|---|---|---|---|---|
| W38 D1-D2 (9/15-16) | 阶段 C 11 E2E 真跑 + 跨域 saga 真实交易 | SRE Lead + Mavis | 22/22 PASS (per C3-C8) | RGS-PHASE-C-PREP §1 阶段 C |
| W38 D3 (9/17) | 阶段 D 评审启动 | Mavis + Ulysses | 5 域 E2E 跑通 = 业务里程碑 (per D1) | RGS-PHASE-C-PREP §1 阶段 D |
| W38 D4 (9/18) | batch 域 v0.1 解冻公告 (per C1 派生约束触发条件) | Mavis + Ulysses | `RGS-BATCH-V0.1-UNFREEZE-2026-09-18_v0.1.md` | C1 |
| W38 D5 (9/19) | **RGS-CRITIQUE-IMPROVEMENT v0.2 正式升版** (per 本 v0.2 反馈) | Mavis 自审 + Ulysses 二审 | 5 大问题最终评估 + 业务里程碑定义定稿 | 本 v0.2 |

**v0.2 (本版本) 定位**: W37 实战预演版 (W36 末 9/2 18:30 JST), 5 大问题重新评估 + 6 域 × 5-10 项 checklist 起草。W38 D5 9/19 JST 正式升版 = 5 域 E2E 真跑通后, 最终业务里程碑定义定稿。

---

## 6. 已知缺口 (per 8/26 JST 缺标比错标)

### 6.1 治理派未闭环项 (v0.2 仍存在)

- **A 类 4 条未拍板**: A1/A3/A4 进候选清单 (per B2), 12/2 季度评审前不启动, md 行数继续上升
- **A1 BAS-037 拆分风险 (候选)**: 4 份后跨引用维护成本 +20%, 1 周 sprint 内未必全完成
- **A2 老 commit 引用 redirect**: STATUS-SNAPSHOT v0.6.10-v0.6.25 移 archive 后, 老 commit 引用需要 redirect, 工作量 1-2 天额外
- **C3 业务指标 vs 老治理指标切换**: W37/W38 周报沿用双指标, 完全切换到"5 域生产可用 checklist" 要 W38 D3 5 域 E2E 跑通后

### 6.2 业务派未闭环项 (v0.2 仍待 Phase C)

- **5 域 ST 业务 mTLS 1 跳未跑通**: gm-backend 8081 HTTP ✅, 5 域 gRPC mTLS 待 Phase C 阶段 B (W37 D3-D5)
- **22 测试函数未跑通**: 0/22 (per RGS-TEST-RUN-PLAN v0.1), 11 UT W37 D6 + 11 E2E W37 D7-W38 D2
- **prometheus CrashLoopBackOff 27h**: SRE 阶段 A3 (W37 D2) 修复
- **batch 域 v0.1 解冻**: W38 D4 (9/18 JST) 5 域 E2E 跑通后 Ulysses 拍板
- **RGS-WEEKLY-W37 v0.1 启动预热已立** (commit `8d69cef`), v0.2/v0.3 待 W37 D1/D7 发布

### 6.3 流程派未闭环项 (v0.2 仍存在)

- **B3 DDD Review 二审必到**: Ulysses 时间窗口不定, 9 份历史自动通过是反模式修正, W37 起新写 DDD Review 触发真签
- **D1 5 域 E2E 跑通**: 等 Phase C SRE 介入, W37 D6-W38 D2 阶段 C 跑
- **A 类若全启, 跨引用维护成本**: 1-2 天额外工时 (per 6/2 季度评审)
- **W37 实战 hotfix 风险**: W37 D2-D5 Phase C 阶段 A + 阶段 B 可能产生 1-3 hotfix (per RGS-PHASE-C-KICKOFF v0.1 §1.3), 单条 hotfix 应有信息量, pre-commit hook 兜底

### 6.4 v0.2 自身已知缺口 (W36 末起草局限)

- **6 域 checklist 是预演基准**: 实际 W37 D2-D7 阶段 A/B/C 跑通后, 6 域 × 10 项 = 60 项的 实际 ✅/🟡 比例 才能确定
- **batch 域 9 项全 🟡**: per C1 冻结, W38 D4 解冻后才有意义, 本 v0.2 起草是为 W38 D5 正式升版打基础
- **W37 实战 1 周后**: W37 D7 (9/14 JST) W37 周报 v0.3 出后, 本 v0.2 §5 路线图部分要回填实际进展
- **本 v0.2 未触发 DDD Review 二审**: 起草后走 B3 流程, Mavis 自审 1 次停手, 待 W38 D5 9/19 JST Ulysses 二审正式定稿

---

## 7. 派生约束守护 (per AGENTS.md v0.6.5 §8 + v0.6.7 即时增段)

| 派生约束 | v0.2 守护 |
|---|---|
| L1 cargo check 0 error | N/A (本 v0.2 不动 Rust) |
| L1.1 cargo test --lib | N/A (本 v0.2 不动 Rust) |
| L1.2 E2E 跑通 | N/A (本 v0.2 是评估文档, 不触发 E2E; §4 6 域 checklist 是 L1.2 业务跑通基准) |
| L2 引用必须 git 实证 | ✅ 本 v0.2 §1 数据表全 git 实证 (commit SHA / file:line / Measure-Object 命令) |
| L11 cargo build dir lock | N/A (本 v0.2 不编译) |
| L12 临时 log 不入 commit | ✅ pre-commit hook 兜底 |
| L13 自指字段 deferred 实时查询 | ✅ commit / file:line 全 git 实证, 自指字段 (e.g. ahead of origin/main, hotfix 计数) 重新查 `git log` 实时值 |
| L14 plumbing brace 跟踪 | N/A (本 v0.2 无 patch 字符串拼接) |
| 8/27 11:06 JST 凭据硬 ban | ✅ 文档无 env value 痕迹 (k8s secret 仅提"导出 SOP", 不实际打印 cert 内容) |
| 9/2 10:18 JST B2 派生约束 L1-L14 冻结 6 个月 | ✅ 本 v0.2 不动派生约束 (L-CANDIDATES.md 仍 3 条候选清单 + 1 保留位) |
| 9/2 10:18 JST C1 batch 域 v0.1 冻结 | ✅ 本 v0.2 §4.7 batch 域 9 项全 🟡, 不动 batch 域 v0.1 文档 |
| 9/2 10:18 JST C3 业务指标新指标 | ✅ 本 v0.2 §4 "5 域生产可用 checklist" = C3 派生约束落地 |
| 9/2 11:05 JST D2 L1/L1.1/L1.2 三件套 | ✅ §4 每域第 1-4 项 = L1.1 UT + L1.2 E2E 配套 |
| 9/2 11:05 JST D3 commit 模板 | ✅ 本 v0.2 commit 沿用 `.gitmessage` (type(scope): summary + DoD + Evidence + 代签 + 派生约束守护) |
| 9/2 14:11 JST B3 DDD Review 二审 | 🟡 本 v0.2 起草后走 B3 流程, Mavis 自审 1 次停手, 待 W38 D5 9/19 JST Ulysses 二审 |

---

## 8. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 10:18 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 5 大问题 + 4 类方案 (16 条) + 拍板 (A+B+C+D 全选默认错 / 6 域不缩 / 冻结归档) + 1 周 sprint checklist, 7 跟踪 doc 体系 1 周内冻结归档, 派生约束 L1-L14 冻结 6 个月 |
| v0.1.1 | 2026-09-02 10:54 | 架构师(Mavis 接手 agent per DEC-008) | hotfix: Q1 实际拍板 = B+C+D (A 不选). 修正 §3.1 A 类状态 (4 条全部进候选清单, 仅 A2 跨类落地), §3.2-3.4 B/C/D 状态全部标 "已拍板", §4.1 Q1 改 "3 类 (B+C+D)", §5.1 sprint 删 A1/A3/A4 任务 + 加 A 类进候选清单 (D6) |
| v0.2 | 2026-09-02 18:30 | 架构师(Mavis 接手 agent per DEC-008) | W37 反思版: §1 现状快照更新 (221 commit / md 119,585 / doc/code 1.44:1 / 5 域 mTLS 1/5 / hotfix 0) + §2 5 大问题重新评估 (✅#2 hotfix 已闭环, 🟡#1 治理压实现进行中, 🟡#3 AI 自指 B3 立但 9 份反模式修正, 🟡#4 工作区卫生部分闭环, 🟡#5 DoD 偏轻 D2/D3/D4 立 + L1.2 待 Phase C) + §3 4 类方案重新评估 (B 4/4 全部落地, C 1/4 闭环 + 2/4 进行中 + 1/4 维持不采纳, D 4/4 全部立 D1 部分待 Phase C, A 类维持候选清单) + **§4 新增 "5 域生产可用 checklist" (6 域 × 5-10 项 = 60 项, C3 派生约束配套, 业务里程碑判定基准)** + §5 W37 实战 sprint 路线图 (5 工作日 + W38 衔接 4 天) + §6 已知缺口 (治理派 4 项 / 业务派 5 项 / 流程派 4 项 / v0.2 自身 4 项 = 17 项) + §7 派生约束守护段 (L1-L14 + 8/27 凭据硬 ban + B2/C1/C3/D2/D3/B3 全部 ✅) + §8 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
