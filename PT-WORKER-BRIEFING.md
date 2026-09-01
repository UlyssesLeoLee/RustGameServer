# 平台层 5 + 工具 3 派工简报 (per 9/1 14:15 JST 续 DDD Review §5.2 P1)

> **基线 commit**: 9c84c48 (main, 9/1 14:00 JST st-11/st-12 merge)
> **派工时间**: 2026-09-01 14:15 JST
> **预估**: 2-3h, +5000-8000 行, +200 tests
> **关联**: DDD Review v0.1 §420-426 平台层 5 + 工具 9 拆 worker 方案

---

## 0. 派工总览

| 类别 | Worker | Worktree | 分支 | Crate 范围 | .rs 现状 | 预估产出 |
|---|---|---|---|---|---|---|
| 平台层 | w1-pt-shared-platform | D:/rgs-pt-shared-platform | pt/shared-platform | shared-platform | 22 (1 test) | +200 行, +10 tests |
| 平台层 | w2-pt-cluster-ops | D:/rgs-pt-cluster-ops | pt/cluster-ops | cluster-ops (含 realm_lifecycle) | 55 (1 test) | +300 行, +15 tests |
| 平台层 | w3-pt-function-plane | D:/rgs-pt-function-plane | pt/function-plane | function-plane | 7 (1 test) | +150 行, +5 tests |
| 平台层 | w4-pt-gm-backend | D:/rgs-pt-gm-backend | pt/gm-backend | gm-backend | 17 (1 test) | +250 行, +12 tests |
| 平台层 | w5-pt-rgs-testkit | D:/rgs-pt-rgs-testkit | pt/rgs-testkit | rgs-testkit (含 examples) | 20 (4 test) | +150 行, +8 tests |
| 工具 | w6-pt-card-replay-i18n | D:/rgs-pt-card-replay-i18n | pt/card-replay-i18n | card-service + replay-service + i18n-service | 32 (6 test) | +500 行, +20 tests |
| 工具 | w7-pt-leaderboard-overflow-asset | D:/rgs-pt-leaderboard-overflow-asset | pt/leaderboard-overflow-asset | leaderboard-service + rgs-overflow-alert + rgs-asset-download | 55 (21 test) | +500 行, +20 tests |
| 工具 | w8-pt-arc-certgen-hello | D:/rgs-pt-arc-certgen-hello | pt/arc-certgen-hello | rgs-arc-olu + rgs-certgen + rgs-hello | 5 (4 test) | +200 行, +10 tests |

---

## 1. 工作环境 (per worker, 8 worker 统一)

- worktree: D:/rgs-pt-<scope> (8 worker 各一)
- 分支: pt/<scope> (基线 9c84c48)
- 负责 crate: 见上表

## 2. 必做 (per worker)

1. **读本简报 + AGENTS.md §6 模板 + DDD Review v0.1 §420-426**
2. **探索**: Get-ChildItem crates/<your-crate> -Recurse -Filter *.rs
3. **写 UT 优先** (per DDD Review 沿用 InMemory mock 风格):
   - 单元级: 验证 5+ 业务函数 (含边界 + 异常 + proptest)
   - proptest 块: entity / domain 关键 invariant
4. **写 IT 优先** (per DDD Review 沿用 InMemory mock 风格):
   - 集成级: 验证 3+ 跨模块场景 (mock DB / mock 5 域 svc)
5. **验证**: `cd D:/rgs-pt-<scope>; cargo check -p <crate> --tests 2>&1 | tail -20`
6. **修到 0 error 后 commit** (代签格式 per 8/27 JST)
7. **(可选 2nd commit)** git push origin pt/<scope> 推 worktree 分支

## 3. DoD (per worker, 强约束)

- ✅ 必跑 `cargo check --tests` (限时 60s) - 禁止 `cargo test` 长编译
- ✅ commit 1+ 段带代签 (修订人=Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手 / 审批=架构师(Mavis 接手 agent per DEC-008))
- ✅ worktree 内只动自己 crate, 域间不交叉改
- ✅ 不改 `AGENTS.md` / `docs/00-基准与治理/RGS-OPEN-QA-*` / `docs/14-项目管理/ddd-review/RGS-DDD-*` (主会话负责)
- ✅ commit message 末尾 "代签 + 审批 + 修订人" 三行齐全
- ✅ 不需要 merge 到 main, 推到 pt/<scope> 分支即可 (主会话负责 merge)

## 4. 卡住的应对 (per 8/31 经验, 严格)

- cargo check 超 60s → 接受 warning, 先 commit 占位
- 找不到合适 mock → 复用 src/ 已有 InMemory*Repository (per rgs-testkit 强约束禁 InMemory? 用 NoOp 或 real DB 链接)
- 等编译不要用 Start-Sleep 轮询
- 单 commit 跨多个 crate → 不允许, 每个 crate 单 commit

## 5. 派工基础信息 (8 worker 共享)

- **基线 commit**: 9c84c48 (main)
- **8/31 UT 模板参考**: `D:/rgs-ut-economy` (per commit 1db3249 137+ tests 模式)
- **强约束** (per AGENTS.md v0.1 §2.1 L1 + DDD Review v0.1 §425-426):
  - 不跑 `cargo test`, 只跑 `cargo check --tests`
  - worker 必跑 `cargo check -p <crate> --tests` (限时 60s)
  - InMemory mock 风格沿用
  - 5 域独立 Lead 原则 (per 8/21 JST): 1 worker 1 crate, 不交叉
- **rgs-testkit 强约束** (per WF-1-55.31): 禁 InMemory mock, 验证真实 DB 链接
- **P1 backlog 同步** (per DDD Review §429): worker 注意到 6 项 P1 (RBAC / audit verify / wins≤total / outbox / guild capacity / leave_guild API / push dispatcher) 不动, 留给 5 业务域 worker 后续处理

## 6. 输出格式 (per worker commit)

```
<type>(<scope>): <subject>

<body>

代签: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手
审批: 架构师(Mavis 接手 agent per DEC-008)
修订人: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手
```

例: `test(shared-platform): UT 增 10 测试覆盖 channel/tls/...`

## 7. 主会话协调 (per worker 完成后)

- 主会话验证 `cargo check -p <crate> --tests` 0 error
- 主会话负责 8 worker merge --no-ff 到 main (per 8/31 4 fix merge 模式)
- 主会话负责 5 业务域 + 5 平台 + 3 工具 = 13 域 DDD Review 终审汇总

## 8. 风险控制 (per AGENTS.md v0.1 §2.4 L4)

- 8/31 5 worker 0 产出教训: 5 worker 派工时没主会话先打头阵
- 本轮 8 worker 已直接派工 (per DDD Review §425-426 明确方案)
- 主会话监控第 1 worker 完成, 模板 OK 后批量 push + merge
- 任何 worker 跑超 30min 仍 0 commit → 主会话取消 + 报告 Ulysses

## 9. 工作流 checklist (per worker)

- [ ] cd D:/rgs-pt-<scope>
- [ ] git log --oneline -3 (确认 HEAD = 9c84c48)
- [ ] Get-ChildItem crates/<crate> -Recurse -Filter *.rs (清单)
- [ ] 写新测试 (UT + IT, 沿用 5 业务域 InMemory 风格)
- [ ] cargo check -p <crate> --tests 2>&1 | tail -20
- [ ] 0 error → git add + git commit
- [ ] git push origin pt/<scope> (可选)
- [ ] 报告主会话: worktree path + commit SHA + test count

## 10. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-01 14:15 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建, 8 worker 派工简报 (5 平台 + 3 工具) |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化
