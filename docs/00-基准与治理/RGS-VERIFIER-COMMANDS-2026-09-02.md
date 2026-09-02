# Verifier 取数命令清单 v0.1 (per L13 守护, 2026-09-02 10:38 JST)

> **创建日期**: 2026-09-02 10:38 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008) 代签 Ulysses
> **状态**: 🟢 就绪 (L13 派生约束终极守护)
> **关联**: STATUS-SNAPSHOT v0.6.23 §0 终极守护 + v0.6.24 §1 总盘统计 + v0.6.26 §2 索引

## 0. 目的

把 STATUS-SNAPSHOT §0 §1 §2 等"自指字段"从"hotfix 改"模式切换为"verifier 命令实证"模式 — verifier 跑以下命令就能拿到最新数字,不再依赖 hotfix 改 §1 数字。

## 1. 核心取数命令 (L1 派生约束 1 worker 1 crate 守护)

### 1.1 git 状态 (5 命令)

```bash
# main HEAD (短 hash, 7 char)
git rev-parse --short main

# ahead of WBS v0.2 (84edf26) — E3 W1-W6 净增 commit 数
git rev-list --count 84edf26..main

# ahead of origin/main
git rev-list --count origin/main..main

# working tree 状态 (clean + untracked)
git status -sb

# 12 sub-bucket 状态 (per §1 7 phase + 6 E 子桶)
git log --oneline 84edf26..main | wc -l
```

### 1.2 本会话 21 个 hotfix 索引 (§2 文档 hotfix 段)

```bash
# 列出本会话全部 hotfix commit (L13 守护 §0 终极 + §1 实时更新 + §2 索引固化)
git log --oneline 84edf26..main | grep -E "snapshot.*v0\.6\.[0-9]+ hotfix|wbs.*v0\.4\.[0-9]+ hotfix" | wc -l

# 列出 4 大跟踪文档 commit (L13 + L11 + L12 守护)
git log --oneline 84edf26..main | grep -E "RGS-STATUS-SNAPSHOT|RGS-PLAN-WBS|RGS-DB-PARTITIONED|TEST-RUN-PLAN"
```

### 1.3 22 测试函数状态 (L1 派生约束 60s 限时)

```bash
# rgs-batch-backend 22 测试函数清单 (11 UT + 11 E2E)
grep -E "fn test_|fn e2e_" tools/rgs-batch-backend/tests/integration_tests.rs | wc -l

# 11 UT 函数 (per `0107d2d` BA-W3-10)
grep -E "fn test_" tools/rgs-batch-backend/tests/integration_tests.rs | wc -l

# 11 E2E 函数 (per `d3ca7be` BA-W3-11)
grep -E "fn e2e_" tools/rgs-batch-backend/tests/integration_tests.rs | wc -l

# cargo check --tests 0 error 状态 (per L1 60s 限时)
Start-Process cargo -ArgumentList @('check','--tests','-p','rgs-batch-backend') -RedirectStandardOutput 'cargo-check-tests.log' -RedirectStandardError 'cargo-check-tests.err' -PassThru
# 60s 后看 log 0 error = 状态正确
```

### 1.4 3 git stash 文件级实证 (L12 派生约束 8/25-8/26 老 stash)

```bash
# 列出全部 stash
git stash list --format='%gd: %s (branch: %cI)'

# stash@{0} Cargo.lock drop 安全
git stash show --name-status 'stash@{0}'

# stash@{1} RGS-REQ-001/005 apply 评估
git stash show --name-status 'stash@{1}'

# stash@{2} 空 stash drop 安全
git stash show --name-status 'stash@{2}'
```

### 1.5 4 DRAFT partitioned SQL 评审状态 (per c2acf02 + DB-CHECKLIST v0.1.1 + SEQUENCE v0.1)

```bash
# 4 DRAFT SQL commit c2acf02 实证
git show --stat c2acf02 | grep partitioned

# 4 DRAFT SQL 文件实证 (L1 派生约束 1 worker 1 crate 验证行数)
wc -l crates/admin-service/migrations/0006_audit_log_partitioned.sql
wc -l crates/economy-service/migrations/0006_transaction_ledger_partitioned.sql
wc -l crates/economy-service/migrations/0007_sagas_partitioned.sql
wc -l crates/match-service/migrations/0041_moves_partitioned.sql

# 评审启动材料 v0.1.1 + 时序 v0.1 实证
git log --oneline 84edf26..main | grep -E "RGS-DB-PARTITIONED-DRAFT-REVIEW-(CHECKLIST|SEQUENCE)"
```

### 1.6 working tree untracked 残留 (L11 派生约束)

```bash
# 主 worktree untracked 文件 (2 cargo build 残留, mavis-trash ban, L12 不要求清)
git status -sb | grep '^??'

# 主 worktree .worktrees 内部 5 项老临时文件 (8/29-8/30 残留, L12 不要求清)
Get-ChildItem .worktrees 2>&1

# docs/ 空目录残留 (1 项 docs/ddd-review/, 0 file 0 commit, L11 不影响 git 状态)
Get-ChildItem docs/ddd-review 2>&1
```

## 2. 派生约束守护 (L1/L11/L12/L13)

- **L1** cargo check 60s 1 次拿 status, 1 worker 1 crate (本命令清单不涉及 cargo build, 隔离 target dir)
- **L11** build dir lock 防御 (1.3 / 1.5 cargo 命令不冲突老 worktree target dir)
- **L12** 临时 log / .txt 不入 commit (本命令清单产生的 log 放 L12 临时目录, 不入 commit)
- **L13** 自指字段全 deferred 实时查询 (本命令清单是 L13 终极守护 — verifier 跑命令拿最新数字, 不依赖 hotfix 改 §1 数字)

## 3. 命令清单版本控制

- 本文档 `RGS-VERIFIER-COMMANDS-2026-09-02.md` v0.1 (per 2026-09-02 10:38 JST Mavis 接手代签)
- 后续 verifier 反馈循环时, 如果命令变化 (新增/废弃), 升版 v0.1.x hotfix
- 升版触发: 文档结构变化 / 新增命令 / 废弃命令

## 4. 关联文档

- `RGS-STATUS-SNAPSHOT-2026-09-02.md` v0.6.26 (主快照, §0 §1 §2 引用本命令清单)
- `RGS-PLAN-WBS-token-bucket-v0.4.md` v0.4.9 (WBS 跟踪表, §1.1 + §4 + §4.1 引用本命令清单)
- `RGS-DB-PARTITIONED-DRAFT-REVIEW-CHECKLIST-2026-09-02.md` v0.1.1 (评审启动材料)
- `RGS-DB-PARTITIONED-DRAFT-REVIEW-SEQUENCE-2026-09-02.md` v0.1 (评审召集时序)
- `tools/rgs-batch-backend/TEST-RUN-PLAN-2026-09-02.md` v0.1 (22 测试函数运行计划)

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

## 5. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 10:38 | 架构师(Mavis 接手 agent per DEC-008) | 初版: verifier 取数命令清单 (5 大类 6 命令组: git 状态 / hotfix 索引 / 测试函数 / git stash / DRAFT SQL 评审 / working tree 残留), L13 终极守护实现, 代签 per 8/27 19:39/20:56/21:59 JST 三次强化 |
