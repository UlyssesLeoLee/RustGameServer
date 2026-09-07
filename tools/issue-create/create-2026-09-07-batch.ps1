$ErrorActionPreference = 'Stop'
$repoRoot = 'D:\RustGameServer'
Set-Location $repoRoot

# 代签格式 per 8/27 JST 三次强化 (19:39 / 20:56 / 21:59) — Mavis 默认代签 Ulysses
$signature = @'

---
**代签** (per 8/27 JST 三次强化 + DEC-008):
- author = Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
- 审批 = 架构师 (Mavis 接手 agent per DEC-008) + 自审 2026-09-07
- 修订人 = Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
'@

# helper: write body to tmp file without BOM, then gh issue create
function New-RgsIssue {
    param(
        [string]$Title,
        [string]$Body,
        [string]$Labels,
        [string]$Assignee = 'Ulysses'
    )
    $tmp = [System.IO.Path]::GetTempFileName()
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($tmp, $Body, $utf8NoBom)
    Write-Host ">>> $Title"
    Write-Host "    labels = $Labels"
    $out = & gh issue create --title $Title --body-file $tmp --label $Labels --assignee $Assignee 2>&1
    Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    if ($LASTEXITCODE -ne 0) {
        Write-Host "    ERROR: $out" -ForegroundColor Red
        return $null
    }
    Write-Host "    $out" -ForegroundColor Green
    return $out
}

$results = @()

# ============ B-1: 5 域 E2E PoolTimedOut ============
$body = @'
## 背景
2026-09-07 跑 L1.2 E2E (cargo test --test '*' 跨域集成),5 域 integration test target 全部失败,根因为 sqlx::test fixture 启动后无法连接真实 PG。

## 证据
- `cargo-test-e2e-fix3-2026-09-07.log:232-260` — 5 target 全部 0/3 失败
- 失败 target:
  - `-p admin-service --test integration_admin_basic`
  - `-p match-service --test integration_match_basic`
  - `-p player-service --test integration_player_basic`
  - `-p rgs-overflow-alert --test it_cross_domain`
  - `-p social-service --test integration_social_basic`
- 全部 panic: `failed to connect to setup test database: PoolTimedOut` (sqlx-core 0.8.6 testing/mod.rs:226)
- 同一天 `fix*` 跑了 3 轮 (cargo-test-e2e-2026-09-07 / -fix / -fix2 / -fix3) 都未解决

## 影响
- L1.2 DoD 标记通过,但实际 E2E 业务级 0 通过
- 治理指标 (commit + 报告) ≠ 业务跑通
- 跨域 saga / 5 域主链路 commit 都依赖此通过

## 根因候选
1. sqlx::test fixture 没起来 (没有测试 PG 实例 / docker-compose 没启)
2. DATABASE_URL env 没在测试 shell 注入
3. .env.test / TestContainers 缺失

## 关联
- AGENTS.md §2.1 L1.2 必跑 (跨域 saga / 5 域主链路)
- 已有 issue #8 [M1] PH-1 启动 Gate 收尾,本 issue 是 L1.2 阻塞
'@ + $signature
$r = New-RgsIssue -Title '[BUG] 5 域 integration test 9/7 E2E 全 0/3 失败 (sqlx PoolTimedOut)' -Body $body -Labels 'bug,help wanted'
$results += @{ Id = 'B-1'; Result = $r }

# ============ B-2: proptest 回归 seeds 未处理 ============
$body = @'
## 背景
两处 proptest-regressions 文件留有失败 seeds,但之前 commit 是用 `#[ignore]` / 容忍方式绕过,不是真修。

## 证据
- `crates/shared-platform/proptest-regressions/subject.txt`: 2 个 seeds (含 InvalidDomain 边界)
- `crates/function-plane/tests/ut_proptest.proptest-regressions`: 2 个 seeds (`cmp_v` 简化版局限)
- commit `c36c3f9 fix(function-plane): proptest_semver_suffixes 跳过 build suffix 测试` — 跳过而非修复
- commit `8b962fe fix(shared-platform): subject proptest 容忍 UnknownDomain 边界` — 容忍而非修复

## 影响
- proptest 回归用例积累,后续 contributor 跑会一直 fail
- 简化版 (cmp_v) 局限是已知技术债,迟早要真实现
- AGENTS.md §2.1 L1 派生约束禁"只写不验"绕过

## 根因候选
1. cmp_v 简化版是设计取舍,需要补 build suffix 完整版
2. subject 边界 (UnknownDomain) 是 domain 枚举不全,需补枚举值

## 关联
- 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 通用缺口
'@ + $signature
$r = New-RgsIssue -Title '[BUG] proptest-regressions 2 处 seeds 未真修 (commit c36c3f9/8b962fe 仅 ignore 绕过)' -Body $body -Labels 'bug,help wanted'
$results += @{ Id = 'B-2'; Result = $r }

# ============ I-1: cluster-ops 3 处 unimplemented!() ============
$body = @'
## 背景
`cluster-ops/src/realm_lifecycle/operations/archive.rs` §8 集成测试桩 3 处 `unimplemented!()`,标 `#[ignore]` + `pending SRE handover`,但 SRE 接力未启动,无追踪 issue。

## 证据
- `crates/cluster-ops/src/realm_lifecycle/operations/archive.rs:1493` — `it_archive_policy_persists_to_admin_db`
  - `unimplemented!("real admin_db integration test pending SRE handover")`
  - `#[ignore = "requires real admin_db + LCM migration 0020_lcm_tables.sql applied"]`
- `crates/cluster-ops/src/realm_lifecycle/operations/archive.rs:1500` — `it_n_plus_2_replicas_in_s3`
  - `unimplemented!("real S3 N+2 verification pending SRE handover")`
  - `#[ignore = "requires real S3/MinIO with lifecycle policy"]`
- `crates/cluster-ops/src/realm_lifecycle/operations/archive.rs:1507` — `it_gdpr_delete_full_path`
  - `unimplemented!("real GDPR delete IT pending SRE handover")`
  - `#[ignore = "requires real admin_db + player_db + economy_db + social_db"]`

## 影响
- 真实 LCM 集成测试桩空转,SRE 接力 PH-6 实测无追踪
- GDPR 路径未实测 (合规风险)

## 关联
- DTL-031 §11.1 第 5 项 (ADR-0052 具名审批完成) 需此 3 项通过
- AGENTS.md §0 6 域独立 Lead (admin 域 Lead 待命)
'@ + $signature
$r = New-RgsIssue -Title '[BUG] cluster-ops archive.rs 3 处 unimplemented!() SRE 接力未启动 (LCM + S3 N+2 + GDPR)' -Body $body -Labels 'bug,help wanted'
$results += @{ Id = 'I-1'; Result = $r }

# ============ D-1.1: A1.9 监控指标 6/7 DTL 缺 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 关键缺口 1,5 域 DTL 通用 §A.1.9 监控指标:仅 DTL-031 §9 显式 7 项指标,其余 6/7 DTL 缺。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:36 跨域总结 1/7 DTL 有指标
- 5-DOMAIN-DTL-REVIEW-REPORT.md:168 关键缺口 1 (`A1.9 监控 / A1.10 容量`)
- 5-DOMAIN-DTL-REVIEW-REPORT.md:97 §A.7.5 (`0/7 DTL 显式指标定义`)

## 缺口清单
- DTL-018 (Player 域子模块): 无显式指标
- DTL-015 (Economy 交易): 无显式指标
- DTL-016 (Economy 对账): 无显式指标
- DTL-026 (Match): §4.3 部分覆盖,§9 未列
- DTL-019 (Social 推送+兑换码): 无显式指标
- DTL-020 (Social 内购/选服): 无显式指标
- DTL-031 (Admin ClusterOps): §9 7 项 (独有)

## 关联
- A7.5 5 域指标命名一致基线缺失 (本 issue 是其前置)
- AGENTS.md §0 5 域独立 Lead 需每域补 1 份
'@ + $signature
$r = New-RgsIssue -Title '[DOC] A1.9 监控指标 6/7 DTL 缺 (仅 DTL-031 §9 7 项显式)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-1.1'; Result = $r }

# ============ D-1.2: A1.10 容量估算 0/7 DTL 数字 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 关键缺口 1,5 域 DTL 通用 §A.1.10 容量估算:0/7 DTL 有 DAU 100k / QPS 10k 数字。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:36 跨域总结 `0/7 DTL 有 DAU 100k/QPS 10k 估算;1/7 提到待验证参数`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:168 关键缺口 1
- 仅 DTL-031 §4.3 `300 秒观察窗口和 120 秒超时均为待验证规划参数,不是已承诺的 p99/SLA` (line 191)

## 影响
- 性能基线测试 (PH-4) 无法做:容量数字是性能基线输入
- 容量规划/资源申请无依据

## 关联
- issue #19 [PH-4~8 段] 性能基线依赖此完成
- 5 域独立 Lead 需每域提供 1 份容量估算
'@ + $signature
$r = New-RgsIssue -Title '[DOC] A1.10 容量估算 0/7 DTL 有 DAU 100k / QPS 10k 数字' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-1.2'; Result = $r }

# ============ D-1.3: A1.13 DoD 6/7 DTL 列入"不覆盖" ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 关键缺口 2,5 域 DTL 通用 §A.1.13 DoD (Definition of Done):6/7 DTL 列入"不覆盖",仅 DTL-031 §11.1 完整。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:36 跨域总结 `6/7 DTL 列入"不覆盖"`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:170 关键缺口 2 (`A1.13 DoD`)
- DTL-031 §11.1 第一行代码前必须完成 (line 387-394) + §11.1 具名 Gate 6 项 (line 392+)

## 缺口清单
- DTL-018: 列入"不覆盖"
- DTL-015: 列入"不覆盖"
- DTL-016: 列入"不覆盖"
- DTL-026: 列入"不覆盖"
- DTL-019: 列入"不覆盖"
- DTL-020: 列入"不覆盖"

## 关联
- AGENTS.md §2.1 L1/L1.1/L1.2 三件套是 DTL DoD 实施层
- DTL DoD 缺失是 DDD Review 一审材料不齐根因
'@ + $signature
$r = New-RgsIssue -Title '[DOC] A1.13 DoD 6/7 DTL 列入"不覆盖" (仅 DTL-031 §11.1 完整)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-1.3'; Result = $r }

# ============ D-1.4: A2.1/A2.2 players/player_characters/player_inventory 主表 DDL 缺失 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 关键缺口 3,Player 域主表 (players / player_characters / player_inventory) 在 DTL-018 + DTL-036 均无字段级 DDL。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:44 §A.2.1 状态 `❌` + 行号 `DTL-018 无 players 表` + `DTL-036 §6 待补齐项第 1 条 (58 行)`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:45 §A.2.2 状态 `❌` + 同上
- 5-DOMAIN-DTL-REVIEW-REPORT.md:128 `DTL-036 §6 待补齐项 4 条何时启动` 占位 `[WF-0-5-7 联检前需统一] [域 Lead 决议]`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:171 关键缺口 3

## 待补齐项
1. 账号/角色/会话物理 DDL 与索引
2. proto 字段号
3. 字段权威清单
4. testkit 夹具

## 关联
- DTL-018 (子模块级) + DTL-036 (契约骨架级) 需合一
- Player 域 Lead 决议何时启动
'@ + $signature
$r = New-RgsIssue -Title '[DOC] A2.1/A2.2 Player 域主表 DDL 缺失 (players/player_characters/player_inventory)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-1.4'; Result = $r }

# ============ D-1.5: A5.1 messages/message_recipients/conversations 主表缺失 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 关键缺口 4,Social 域消息分发主表 (messages / message_recipients / conversations) 在 DTL-019 中完全不存在——DTL-019 实际是"消息推送与兑换码运营工具"。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:72 §A.5.1 状态 `❌` + `DTL-019 无 messages/message_recipients/conversations 三表`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:72 `DTL-019 实际是"推送" (push_consents) + "兑换码" (redemption_code_batches/redemption_codes/redemption_records) 三表 (69-108 行)`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:131 §3.2 DTL-019 §0 描述与源文件标题不一致
- 5-DOMAIN-DTL-REVIEW-REPORT.md:172 关键缺口 4

## 关联
- D-2.2 DTL-019 §0 描述与源文件标题不一致 (本 issue 跟其绑,需一起决议)
- Social 域 Lead 决议:DTL-019 是否拆为两个 DTL
'@ + $signature
$r = New-RgsIssue -Title '[DOC] A5.1 Social 域消息分发主表缺失 (DTL-019 不含 messages/.../conversations)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-1.5'; Result = $r }

# ============ D-1.6: A7.3 + A7.5 跨域 Saga 步骤 + 5 域指标命名一致基线 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §4.2 关键缺口 5+6,跨域一致性 §A.7.3 Saga 步骤编号 + §A.7.5 5 域指标命名一致基线双重缺失。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:95 §A.7.3 状态 `❌` + `DTL-015 §3.1 Saga 步骤 + DTL-016 §3.3 同样复用 FR-EC-003,但两份 DTL 均未编号步骤` + `Q-003 跨 DB Saga 审批未完成 (DTL-031 §8.2)`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:97 §A.7.5 状态 `❌` + `仅 DTL-031 §9 显式 7 项指标;5 域指标命名一致性核查无基线`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:173-174 关键缺口 5+6
- DTL-031 §8.2 `Q-003 审批前,经济域不得实现跨 DB 业务写入`

## 关联
- Q-003 跨 DB Saga 审批是经济域 Lead 阻塞项
- D-1.1 监控指标补齐是本 issue 前置
- REV-004 附件 A §A.7 跨域一致性基线文档待起草
'@ + $signature
$r = New-RgsIssue -Title '[DOC] A7.3 跨域 Saga 步骤编号 + A7.5 5 域指标命名一致基线双重缺失' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-1.6'; Result = $r }

# ============ D-2.1: DTL-018 vs DTL-036 Player 域归属歧义 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §3.1,DTL-018 (2026-08-17 子模块级) 与 DTL-036 (2026-08-21 契约骨架级) 制定时间错位 4 天,REV-004 §A.2 文件指向仅列 DTL-018 (未列 DTL-036)。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:125-129 §3.1 详细占位
- 5-DOMAIN-DTL-REVIEW-REPORT.md:128 `[WF-0-5-7 联检前需统一] [域 Lead 决议]`
- REV-004 §A.2 字段级条目 (A2.1/A2.2) 在两份 DTL 中均未落地为可执行 DDL

## 占位
`[WF-0-5-7 联检前需统一] [Player 域 Lead 决议]`:
- DTL-018 + DTL-036 视为 Player 域两个真源(子模块级 + 契约骨架级)
- DTL-036 §6 待补齐项 4 条何时启动

## 关联
- D-1.4 Player 域主表 DDL 缺失 (本 issue 是其前置决议)
'@ + $signature
$r = New-RgsIssue -Title '[DOC] DTL-018 vs DTL-036 Player 域归属歧义 (时间错位 4 天)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-2.1'; Result = $r }

# ============ D-2.2: DTL-019 §0 描述与源文件标题不一致 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §3.2,REV-004 §A.5 引用 `RGS-DTL-019 消息分发` + 描述"玩家治理/策略/封禁";源文件实际标题为"消息推送与兑换码运营工具"。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:131-134 §3.2 详细占位
- 5-DOMAIN-DTL-REVIEW-REPORT.md:132 `REV-004 §A.5 引用"RGS-DTL-019 消息分发" + 描述"玩家治理/策略/封禁" (per 主对话提示)`
- 5-DOMAIN-DTL-REVIEW-REPORT.md:133 `[WF-0-5-7 联检前需统一] [域 Lead 决议]`——Social 域 Lead 决议 §0 表描述是否同步为"消息推送与兑换码运营工具" (推荐),DTL-019 是否拆为两个 DTL
- 5-DOMAIN-DTL-REVIEW-REPORT.md:134 `§A.5.1 "messages/message_recipients/conversations 消息表"在 DTL-019 中完全不存在`

## 关联
- D-1.5 Social 域消息分发主表缺失 (本 issue 是其前置决议)
- Social 域 Lead 决议
'@ + $signature
$r = New-RgsIssue -Title '[DOC] DTL-019 §0 描述与源文件标题不一致 (消息分发 vs 推送/兑换码)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-2.2'; Result = $r }

# ============ D-2.3: DTL-026 路径在 07-社交目录但内容 Match 域 ============
$body = @'
## 背景
per 5-DOMAIN-DTL-REVIEW-REPORT.md §3.3,DTL-026 文件路径在 `docs/07-社交运营与玩家治理/`,但内容是 Match 域核心 DTL (MT 限界上下文, `match_db`),与 07 目录语义不匹配。

## 证据
- 5-DOMAIN-DTL-REVIEW-REPORT.md:136-139 §3.3 详细占位
- 5-DOMAIN-DTL-REVIEW-REPORT.md:138 `[WF-0-5-7 联检前需统一] [域 Lead 决议]`——Match 域 Lead 决议 DTL-026 是否在 NO-GO 解除后路径迁移到 `docs/08-Match域/` 或 `docs/01-核心架构与设计模式/` (推荐:路径迁移 + 同步 REV-004 §A.4 §0 表归属描述)
- 5-DOMAIN-DTL-REVIEW-REPORT.md:139 `本签字视 DTL-026 为 Match 域真源 (基于内容判断),与 07 目录语义不冲突字段级 Review 结论`

## 关联
- Match 域 Lead 决议
- G-CODE-05 完全关闭前路径对齐
'@ + $signature
$r = New-RgsIssue -Title '[DOC] DTL-026 路径在 07-社交运营与玩家治理/ 但内容 Match 域 (match_db)' -Body $body -Labels 'documentation,help wanted'
$results += @{ Id = 'D-2.3'; Result = $r }

# ============ H-1: 根目录散落 8 份 cargo 测试日志 ============
$body = @'
## 背景
per AGENTS.md §0 L12.1 临时 log 不入 commit 派生约束,根目录散落 10 份 9/7 调试 cargo test 日志未清理。

## 证据 (`git status` 9/7 20:36 JST)
- `cargo-check-2026-09-07-v2.log`
- `cargo-check-2026-09-07.log`
- `cargo-test-e2e-2026-09-07.log`
- `cargo-test-e2e-fix-2026-09-07.log`
- `cargo-test-e2e-fix2-2026-09-07.log`
- `cargo-test-e2e-fix3-2026-09-07.log`
- `cargo-test-lib-2026-09-07.log`
- `cargo-test-lib-fix-2026-09-07.log`
- `cargo-test-overflow-alert-verify-2026-09-07.log`
- `cargo-test-shared-platform-fix-2026-09-07.log`

## 影响
- 10 个 untracked 文件污染 worktree
- 违反 AGENTS.md L12.1 临时 log 不入 commit 派生约束
- `git status` 噪声大,真实改动不易识别

## 关联
- AGENTS.md §0 L12.1 派生约束
- `scripts/cleanup-tmp-files.ps1` (per 9/3 07:31 JST 拍板) + `scripts/pre-commit-tmp-check.ps1`
'@ + $signature
$r = New-RgsIssue -Title '[H] 根目录散落 10 份 cargo test/check 日志未清理 (违反 L12.1)' -Body $body -Labels 'good first issue,help wanted'
$results += @{ Id = 'H-1'; Result = $r }

# ============ H-2: shared-platform/src/retry.rs:12 unused import ============
$body = @'
## 背景
`shared-platform/src/retry.rs:12` 有未使用 import 警告,违反 cargo build 0 warning 目标。

## 证据
- `crates/shared-platform/src/retry.rs:12` `use backoff::backoff::Backoff;`
- 警告级别: `unused import` (part of `unused`)
- `cargo-test-overflow-alert-verify-2026-09-07.log` 编译期出现此警告

## 修复
```rust
// 删除 line 12
- use backoff::backoff::Backoff;
```

## 关联
- AGENTS.md §0 L1 派生约束 (cargo check 0 warning 目标)
'@ + $signature
$r = New-RgsIssue -Title '[H] shared-platform/src/retry.rs:12 unused import (backoff::Backoff)' -Body $body -Labels 'good first issue,help wanted'
$results += @{ Id = 'H-2'; Result = $r }

# ============ H-3: rgs-overflow-alert/tests/it_cross_domain.rs:58 dead code ============
$body = @'
## 背景
`rgs-overflow-alert/tests/it_cross_domain.rs:58` 死代码 `fn make_guard`,编译期警告 + 拖低覆盖率可信度。

## 证据
- `crates/rgs-overflow-alert/tests/it_cross_domain.rs:58` `fn make_guard(domain: Domain, hard: u32) -> OverflowGuard`
- 警告: `function `make_guard` is never used` (part of `dead_code`)
- 出现在 `cargo-test-overflow-alert-verify-2026-09-07.log` 编译期

## 选项
1. 删除 (如果无下游使用)
2. 加 `#[allow(dead_code)]` + 注释说明保留原因
3. 真接入测试使用

## 关联
- AGENTS.md §0 L1 派生约束
'@ + $signature
$r = New-RgsIssue -Title '[H] rgs-overflow-alert/tests/it_cross_domain.rs:58 dead code (fn make_guard never used)' -Body $body -Labels 'good first issue,help wanted'
$results += @{ Id = 'H-3'; Result = $r }

# ============ Summary ============
Write-Host ''
Write-Host '========== Summary ==========' -ForegroundColor Cyan
foreach ($r in $results) {
    Write-Host ("  {0,-8} {1}" -f $r.Id, $r.Result)
}
Write-Host '=============================' -ForegroundColor Cyan
