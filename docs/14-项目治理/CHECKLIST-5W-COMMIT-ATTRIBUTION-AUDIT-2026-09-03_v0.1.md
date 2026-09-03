# CHECKLIST 5 域 commit 归属异常 补正 audit trail (per 9/3 11:08 JST)

> **创建日期**: 2026-09-03 11:58 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: 9/3 11:58 JST Ulysses 拍板选项 B = 补 audit commit (per ask_user option-b-audit-commit) + 8/27 JST 禁回溯叙事 (不动历史) + 8/26 JST 缺标比错标安全
> **配套**: 5 worker 报告 (player + match + social + economy + admin) + AGENTS.md §6.3 PT 派工简报模板 L12 派生约束

---

## 0. 事件背景

**9/3 11:08 JST 5 worker 并发派工**（per 9/3 11:06 JST Mavis 派工决策），5 worker 在主仓库 (D:/RustGameServer) 共享 main HEAD `c52805b`，各自写 1 域 `CHECKLIST-<domain>-PROD-READY-2026-09-03_v0.1.md` + `git add` + `git commit`。

**预期**：5 worker → 5 个独立 commit (1 域 1 commit, scope 严格)

**实际**：5 worker → 3 个 commit (race condition 互相捕获 untracked 文件)

## 1. 实际 commit 归属表

| 域 | 实际所在 commit | 实际 commit 标题 | 跨域文件数 | 状态 |
|---|---|---|---|---|
| **player** | `bc82700` | `docs(critique): CHECKLIST-player-PROD-READY v0.1 落档 (R3 阶段 C3 派生约束拆分)` | 1 (player 17.5 KB) | 本域 ✅ |
| **admin** | `bc82700` (同 player commit) | (player commit 顺手带 admin) | 1 (admin 18.2 KB) | **跨域被收** ⚠️ |
| **economy** | `7f6a9d5` (同 social commit) | (social commit 顺手带 economy) | 1 (economy 18.9 KB) | **跨域被收** ⚠️ |
| **social** | `7f6a9d5` | `docs(critique): CHECKLIST-social-PROD-READY v0.1 落档 (R3 阶段 C3 派生约束拆分)` | 1 (social 18.1 KB) | 本域 ✅ |
| **match** | `f0fe990` | `docs(critique): CHECKLIST-match-PROD-READY v0.1 落档 (R3 阶段 C3 派生约束拆分)` | 1 (match 14.0 KB) | 本域 ✅（独占） |

**5 worker 报告全部确认**: 各自文件内容正确，D3 元信息完整，9-10 项 checklist 复制原文一致，派生约束守护段齐全。**commit 归属 race condition 副作用** 不影响文档内容正确性，仅影响 commit 标题与文件对应关系。

## 2. 根因 (per 8/31 PT 派工 8 worker 教训同症复发)

5 worker 并发 `git add` 时，**互相捕获其他 worker 的 untracked 文件**：
1. social worker 11:08:18 JST 抢先 `git add docs/14-项目治理/CHECKLIST-social-PROD-READY-2026-09-03_v0.1.md` (假设只 add social)
2. 但 `git status` 列出 5 域 untracked 文件，social worker 用 `git add <file>` 精确只 add social，但 commit 触发时如有其他 worker 已 staged 文件，**commit 把所有 staged 文件一起带走**
3. 实际路径: economy worker 11:08:18 JST `git add CHECKLIST-economy-...`，social worker 11:08:18 JST `git add CHECKLIST-social-...` + commit → 7f6a9d5 含 2 文件 (economy + social)
4. 同样 player 11:08:18 JST commit → bc82700 含 2 文件 (player + admin，已被 admin worker 暂存)
5. match 11:08:56 JST 单独 commit → f0fe990 含 1 文件 (match 独占)

**5 worker 派工 design flaw**：
- ❌ 5 worker 共享主仓库 + 各自 `git add` + 各自 `git commit` → 互相捕获 untracked 文件
- ❌ 5 worker 用 `git add .` (全 add) → 必定捕获所有 untracked
- ❌ 5 worker 用 `git add <file>` 精确 add → 但 git commit 时如其他 worker 已 staged 文件，一起被带走
- ❌ 即使 stagger 启动也解决不了 race condition（5 worker 间隔 5-10s 启动，但 git add 时刻重叠）

## 3. 教训 (L12 派生约束应补案例)

**5 worker 派工共享主仓库时, 应 per-file `git add <file>` 不 `git add .`**:
- 但即使 per-file add 仍可能被 race condition 捕获
- 真正解决: 5 worker 用独立 worktree (per 8/31 W37 5 域独立 Lead 模式 ut/player / ut/economy / ut/match / ut/social / ut/admin), 各 worktree commit 后主会话 merge
- 或 1 worker 串行 5 域, 失去"5 worker 并行"形式
- 或 task tool 派工简报加 "DoD: worker 报告 0 commit (仅写文件, 主会话统一 commit)"

**跟 8/31 PT 派工 8 worker 25 min 派工基线同症**: 8/31 8 worker 报告 "目录根污染临时文件"（per L12 派生约束），但当时是临时 log 污染，不是 untracked .md 跨域被收。本次是 L12 派生约束没防住 untracked .md 跨域。

**AGENTS.md §6.3 PT 派工简报模板应补**：
```markdown
## 5 worker 派工约束 (per 9/3 11:08 JST race condition 教训)
- 5 worker 共享主仓库时, **不推荐** 各自 git add + git commit
- 3 选项:
  1. 5 worker 独立 worktree (per 8/31 W37 模式, 5 worktree merge)
  2. 5 worker 写文件不 commit, 主会话统一 git add 5 files + 1 commit
  3. 1 worker 串行 5 域, 失去"5 worker 并行"形式
- DoD 简报明文: "worker 不 commit, 报告即可" 避免 race condition
```

## 4. 不修历史 (per 8/27 JST 禁回溯叙事 + 8/26 JST 缺标比错标)

**bc82700 / 7f6a9d5 / f0fe990 三 commit 标题与 content 不匹配**:
- 严格意义是 scope 错位
- 但 git 历史可读（commit message 仍描述实际工作）
- 不 amend / rebase / filter-branch (跟 8/27 JST "禁回溯叙事" 派生约束冲突)
- 本补正 commit 留 audit trail, 后人 git log 可追溯
- git grep / git log --follow 仍可定位每个域文件 (per 8/27 派生约束 "引用必须 git 实证")

**5 域文件本身** (内容正确)：
- `docs/14-项目治理/CHECKLIST-player-PROD-READY-2026-09-03_v0.1.md` 17,508 bytes
- `docs/14-项目治理/CHECKLIST-economy-PROD-READY-2026-09-03_v0.1.md` 18,944 bytes
- `docs/14-项目治理/CHECKLIST-match-PROD-READY-2026-09-03_v0.1.md` 14,041 bytes
- `docs/14-项目治理/CHECKLIST-social-PROD-READY-2026-09-03_v0.1.md` 18,084 bytes
- `docs/14-项目治理/CHECKLIST-admin-PROD-READY-2026-09-03_v0.1.md` 18,250 bytes
- 合计 86,827 bytes (5 域 checklist 文档独立成档, R3 阶段 C3 派生约束落地)

## 5. 5 worker 实际 token 消耗

| worker | 实际 token | 预算 | 状态 |
|---|---|---|---|
| player | ~30K (commit bc82700 + 17.5 KB 文档) | 200K | ✅ 14% 消耗 |
| match | ~30K (commit f0fe990 + 14.0 KB 文档) | 200K | ✅ 14% 消耗 |
| social | ~30K (commit 7f6a9d5 + 18.1 KB 文档) | 200K | ✅ 14% 消耗 |
| economy | ~30K (内容被 7f6a9d5 收, 自身 commit 失败) | 200K | ✅ 15% 消耗 |
| admin | ~50K (内容被 bc82700 收, 自身 commit 失败 + 3 个 BLOCKER 上报) | 200K | ✅ 25% 消耗 |
| **合计** | **~170K (5 worker 报告)** | **1M** | **留 830K 缓冲** |

## 6. R1 token 累计 (per RGS-DEVPLAN v0.3 §7 R1 业务冲刺 5.3M 触发推送)

| 任务 | token 累计 |
|---|---|
| 治理类 (RGS-DEVPLAN v0.1-v0.3 + L-CAND-006 落地) | 0.4M |
| 5 域 L1.1 验证 (5 worker 首批 + 3 worker 重派 + admin R2 修复) | 0.35M |
| L-CAND-006 配套 (certs/ gitignore + MANIFEST + mTLS 脚本 + 清理脚本) | 0.2M |
| 5 域 checklist 文档 (本批 5 worker) | 0.17M |
| **当前累计** | **≈ 1.12M** |
| **R1 5.3M 触发推送** | **仍需 ≈ 4.18M** |

## 7. 派生约束守护

- **L1/L1.1/L1.2 N/A** (本补正 commit, 不动 Rust)
- **L11** (本 commit 触及 L12 案例库, 5 worker 派工 race condition 应升 L12 案例库)
- **L12** (本 commit 临时 message 走 .gitmessage-tmp/ gitignored 目录, 不入 commit, 9/3 07:31 JST 拍板 cleanup-tmp-files.ps1 兜底)
- **8/27 11:06 JST 凭据硬 ban** (本 commit 无 env value 痕迹, 5 worker 报告均无 secret / cert / env value 打印)
- **8/27 JST 禁回溯叙事** (不 amend / rebase / filter-branch 改写历史, 仅留 audit trail)
- **8/26 JST 缺标比错标** (本 commit 显式列 5 worker race condition 根因 + 教训 + 4 选项, 不假装覆盖)

## 8. 后续动作 (per 9/3 11:58 JST Ulysses 拍板选项 B)

1. ✅ 本 audit commit 落地
2. ⏳ AGENTS.md §6.3 PT 派工简报模板 升 v0.6.10 加 5 worker 派工约束 (worker 不 commit, 主会话统一 commit) — R3 阶段 token 累计 1M 内
3. ⏳ R3 阶段 batch 域解冻后, 5 域 + batch = 6 域独立 checklist 文档 (per GAP-9 + C1 派生约束) — 6 × 200K = 1.2M token
4. ⏳ 派生约束 L12 案例库加 5 worker race condition 教训 — 12/2 季度评审候选清单 (L-CAND-007 / L-CAND-008 等)

## 9. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:58 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: 5 worker 派工 race condition 异常捕获, 实际 commit 归属表 (3 commit 散收 5 域) + 根因分析 + 教训 (L12 案例库) + 不修历史决策 + 5 worker token 消耗 + R1 token 累计 1.12M + 派生约束守护 + 4 后续动作, per 9/3 11:58 JST Ulysses 拍板选项 B (audit commit) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
