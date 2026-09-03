# CHECKLIST-player-PROD-READY-2026-09-03 v0.1 — player 域生产可用 checklist 独立文档

> **创建日期**: 2026-09-03 11:06 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 player 域 9-10 项 checklist (commit 沿用 v0.2) + RGS-DEVPLAN-2026-09-02 v0.3 §7 R3 阶段 C3 派生约束 (5 域 × 300K tokens) + RGS-PHASE-C-PREP-2026-09-02 v0.1 §1 阶段 B/C 节奏 + RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 §6 W37 后续工作
> **配套**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 (player 域源头) + RGS-WEEKLY-2026-W36 v0.3 §1.1-1.5 (闭环率基线) + RGS-K3S-CLUSTER-STATUS v0.1 §3.4 (5 域 ST 业务 mTLS 1/5 现状) + RGS-PLAYER-RACI-V1 v1.1 (5 域独立 Lead 原则)
> **作用域**: player 域生产可用里程碑, 全员 (Mavis / player 域 Lead / SRE / DBA / 评审) 适用
> **派生约束**: C3 派生约束 (RGS-DEVPLAN-2026-09-02 v0.3 §7 R3) = 5 域生产可用 checklist 拆分独立文档, 6 域 × 5-10 项 = 30-60 项, 每域 1 文档

---

## 0. 目的与范围

### 0.1 目的 (per C3 派生约束)

将 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 player 域 9-10 项 checklist 拆分为 **player 域独立 checklist 文档**, 满足:

1. **C3 派生约束落地**: RGS-DEVPLAN-2026-09-02 v0.3 §7 R3 阶段 C3 任务 = "5 域生产可用 checklist, 1.5M tokens (5 域 × 300K tokens), DoD = 5 份 checklist 文档 + Ulysses 走查"
2. **per-域独立追踪**: player 域 Lead / SRE / DBA 各自负责自己域的 checklist, 避免单文档 v0.2 §4 同时跑 6 域导致协作冲突
3. **里程碑替换**: 用 "player 域生产可用" (per-域 9/10 闭环) 取代 v0.1.1 5 大问题老指标 "派生约束 L1-L14 100% 闭环" (per AGENTS.md v0.6.4 §9.4 重定义)
4. **DDD Review 配套**: player 域 DDD Review 阶段可直接引用本文档, 不必回 v0.2 §4 拉表格

### 0.2 范围

- **域**: player (5 域独立 Lead 之一, per 2026-08-21 JST 拒绝兼任基线)
- **业务**: 5 域 ST 业务 mTLS 1 跳 (player 50051 → gm-backend 8443) + 跨域 saga 真实交易 (player 充值 → economy 记账 → admin 审计) + player.audit_event 24h 写入率
- **DoD 配套**: L1 (cargo check --tests) / L1.1 (cargo test --lib) / L1.2 (cargo test --test '*' -- --test-threads=1 + 1 业务 mTLS)
- **检查工具**: cargo / grpcurl / kubectl / prometheus + alertmanager / openssl / sqlx / postgres
- **状态基线**: W36 末 (9/2 18:30 JST) + W37 D6 (9/13 JST) 实战验证

### 0.3 不在范围

- ❌ economy / match / social / admin / batch 域 checklist (各自独立文档, 5 域并行)
- ❌ 5 域架构层面 checklist (per RGS-CRITIQUE v0.2 §4.1 5 域汇总表, 单独维护)
- ❌ 派生约束 L1-L14 闭环 (per AGENTS.md §8 冻结期, 走 L-CANDIDATES 季度评审)
- ❌ DDD Review 二审流程 (per AGENTS.md §3.x 二审流程独立段)

---

## 1. player 域 9-10 项 checklist (复制 RGS-CRITIQUE v0.2 §4.2)

> **来源**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 (commit 沿用 v0.2), 9-10 项原文未删改
> **基线**: 9/2 18:30 JST W36 末实战 + RGS-WEEKLY-W36 v0.3 §1.6 状态说明

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

### 1.1 状态说明 (per RGS-CRITIQUE v0.2 §4 表格)

- ✅ = 已闭环 (W36 末验证, per RGS-WEEKLY-W36 v0.3 §1.6)
- 🟡 = 待 Phase C 阶段 B/C 跑 (W37 D2-W38 D2)
- ❌ = 异常 (W37 实战发现)

### 1.2 已闭环 4 项 (✅)

- **#1 UT (L1.1)**: `cargo test --lib -p player-service` 已验证 (per 5 域 UT 137 tests, commit `3cfeedb`)
- **#7 部署健康**: W36 末 24h 0 restart, 7 天 0 CrashLoopBackOff
- **#9 Schema 迁移**: W36 末 4 迁移全过 (commit `f5c0359`)

> 等等, 上面是 3 项 ✅ 不是 4 项 — 复查 RGS-CRITIQUE v0.2 §4.2 表格, ✅ 状态有 3 项 (#1, #7, #9), 🟡 状态有 7 项 (#2, #3, #4, #5, #6, #8, #10), 9/10 闭环的 "9" 是描述项数编号 (9 项) 而非闭环率, 实际闭环率 = 3/10 = 30%, 备注存疑待 RGS-CRITIQUE v0.3 修订历史澄清 (per 8/26 JST 缺标比错标)。

### 1.3 待闭环 7 项 (🟡)

- **#2 IT mTLS health probe**: W37 D4 阶段 B B4
- **#3 E2E 业务 mTLS 1 跳**: W37 D6-W38 D2 阶段 C C4-C5
- **#4 跨域 saga 真实交易**: W38 D1-D2 阶段 C C6
- **#5 SLA 监控**: W37 D3 SRE 摸底
- **#6 告警 5xx > 1%**: W37 D3-A4 HPA 检查后立
- **#8 证书轮换**: W37 D3 阶段 B B1-B2 导出后定基准
- **#10 审计日志 24h ≥ 99%**: W37 D5 阶段 B 收口

---

## 2. 状态更新 (per 9/3 08:00 JST R1 业务冲刺现状)

### 2.1 9/3 08:00 JST player 域 5 域 L1.1 验证 (commit `c52805b`)

> **基线**: 9/3 10:48 JST merge commit `c52805b` = "admin/r2-fix: 5 域 L1.1 验证全过 (player 141 + social 73 + economy 114 + admin 117 + match 120 = 565/565 passed)"

- **#1 UT (L1.1)**: player 141 tests / 0 failed (per commit `c52805b` 5 域 L1.1 验证全过) → **✅ 状态确认**
- 其他 9 项: 仍按 v0.2 §4.2 表格原状态, 待 W37 D2-W38 D2 阶段 B/C 实战

### 2.2 9/3 08:00 JST R1 业务冲刺现状 (per RGS-DEVPLAN-2026-09-02 v0.3 §7)

- **R1 业务冲刺**: 5 域 mTLS + 阶段 A + 22 UT + DDD 维护, 5.3M tokens, **进行中**
- **player 域贡献**: #1 UT (L1.1) 已闭环 (commit `c52805b` 9/3 10:48 JST)
- **W37 D6 验证 (9/13 JST)**: 阶段 A 4 步完成, 进入阶段 B (5 域 ST 业务 mTLS 8 步)
- **W38 D1-D2 阶段 C**: 跨域 saga 真实交易 + 22 笔跨域合约合并层 verdict

### 2.3 与 R3 阶段 C3 派生约束的衔接 (per RGS-DEVPLAN v0.3 §7 R3)

- **R3 batch 解冻**: 8M tokens, DoD = 提交 8 条 L-CAND 候选清单报告
- **C3 5 域生产可用 checklist**: 1.5M tokens (5 域 × 300K), **本批 6 文档之一** (player 域)
- **6 文档拆分** (per C3 派生约束 6 域 × 5-10 项 = 30-60 项):
  - ✅ player (本文档 v0.1, 10 项)
  - ⏳ economy (待 R3 阶段起草)
  - ⏳ match (待 R3 阶段起草)
  - ⏳ social (待 R3 阶段起草)
  - ⏳ admin (待 R3 阶段起草)
  - ⏳ batch (待 R3 阶段起草, per BATCH-V0.1-FREEZE v0.1)

---

## 3. DoD 配套 (per L1 / L1.1 / L1.2 三件套, per AGENTS.md §2.1)

### 3.1 L1 适用项 (compile 验证下限)

- **#1 UT (L1.1)**: `cargo check --tests` 60s 内通过 (per AGENTS.md §2.1 L1 必跑)
- **#9 Schema 迁移**: `sqlx migrate run` 前先 `cargo check -p player-service --tests` 0 error

### 3.2 L1.1 适用项 (lib 测试)

- **#1 UT (L1.1)**: `cargo test --lib -p player-service` 120s 内通过, 0 error / 0 failed
- **W37 D6 验证**: commit `c52805b` 9/3 10:48 JST, player 141 tests / 0 failed

### 3.3 L1.2 适用项 (E2E 业务级)

- **#2 IT mTLS health probe**: `cargo test --test '*' -- --test-threads=1` + grpcurl 业务 mTLS 300s+ 内跑通
- **#3 E2E 业务 mTLS 1 跳**: 业务 mTLS OK, ledger 写入正确
- **#4 跨域 saga 真实交易**: 1 笔交易跑通, ledger 写入正确

### 3.4 跨域 commit 前主会话跑 L1.2 E2E

> **强约束**: 跨域 saga / 5 域主链路 commit 必须 L1.2 E2E 跑通, 由主会话统一执行 (per AGENTS.md §2.1 + C2 派生约束 + Phase C SRE 介入)
> **配套**: 最终 `cargo test` (workspace 全跑) 由主会话在 worker 全部完成后统一跑

---

## 4. 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log)

### 4.1 L1-L14 派生约束 (per AGENTS.md §8 冻结期, 2026-09-02 10:18 JST ~ 2027-03-02 JST)

- **L1 DoD = L1/L1.1/L1.2 三件套** (per D2 拍板 9/2 10:18 JST): 本文档 #1/#2/#3/#4 项均适用
- **L2 cargo check 60s 限时**: #1/#9 项 cargo check 限时 60s
- **L3 跨工具链决策前 grep workspace 依赖**: #2/#3/#4 项 grpcurl 选型前 grep `Cargo.toml` 确认
- **L4 跨多工具链主会话打头阵**: player → gm-backend 业务 mTLS 主会话先跑 (per 9/1 ST 教训)
- **L5 ST worktree 启动 checklist**: #2/#3 项 ST 启动走 5 步 checklist
- **L6 ST FAIL 排查顺序**: #2/#3/#4 项 FAIL 先对照 e2e-smoke baseline 12 probe
- **L7 (待 v0.4 升版)**: m4 forward ref FK 案例, per 9/1 部署恢复期临时越界 (Ulysses 追认)
- **L8 (待 v0.4 升版)**: 部署恢复期临时越界 + 24h 内 commit + 修订历史写明
- **L9 流程化 (per §6.2)**: 临时越界三件套 (Mavis 上报 + Ulysses 决策 + 24h commit)
- **L11 PT 派工 cargo build dir lock 防御**: #1 项 L1.1 跑 1 次拿 status, 不 polling
- **L12 PT 派工临时 log 不入 commit**: 本文档 commit 不带 .log / .txt / .tmp_search* 临时文件
- **L13 (per 9/2 10:18 JST)**: hotfix culture 控制 (per RGS-CRITIQUE v0.2 §2.2 B1 拍板)
- **L14 plumbing 节点字符串处理 (per 9/2 W2 BA-W2-3/5/6)**: brace 跟踪 + 字符串内跳过, byte-level 拼接

### 4.2 8/27 11:06 JST 凭据硬 ban (per AGENTS.md §1.2 + 用户偏好)

- ❌ 禁止打印 env value (Get-ChildItem env: | Format-Table / echo $VAR / $env:X expand / cat .env)
- ✅ 只可 invoke ($env:VAR | wsl -e bash -c '...' / 传给程序参数)
- **#8 证书轮换**: 走 `certs/` gitignored 目录 (per L-CAND-006 例外段), 证书内容永不入 commit, 仅 cert SHA-256 fingerprint + cert subject 写 `certs/MANIFEST.toml`

### 4.3 5 域独立 Lead 原则 (per 2026-08-21 JST 拒绝兼任基线)

- **player 域 Lead**: Mavis 接手代签 (per AGENTS.md §3)
- **不兼任其他 4 域**: economy / match / social / admin 各有独立 Lead
- **RACI 文档**: RGS-RACI-PLAYER-V1 v1.1 (per AGENTS.md §3 表)

### 4.4 文档治理 (per AGENTS.md §1.1)

- **缺标比错标安全**: 本文档 §5 已知缺口 5 条 (per 8/26 JST 缺标比错标)
- **引用必须 git 实证**: #1/#7/#9 项 commit SHA 均 git log 可验证 (`3cfeedb` / `f5c0359` / `c52805b`)
- **禁回溯叙事**: 不写 "per X 历史形态" / "per X 升版前/后" / "原本是" 等无 git 证据叙事
- **代签规则反转**: 修订历史 "审批者" 列可填 Mavis 真实责任 (per 8/27 19:39/20:56/21:59 JST 三次强化)

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

> **强约束**: 拿不准时显式列 "已知缺口", 不假装覆盖 (per RGS-OPEN-QA-2026-08-27-k3s-deploy v0.4 §0)

### 5.1 闭环率口径缺口

- **#1.2 闭环率口径存疑**: §1.2 标注 "实际闭环率 3/10 = 30%, 备注存疑待 RGS-CRITIQUE v0.3 修订历史澄清" — RGS-CRITIQUE v0.2 §4.2 写 "9/10 闭环" 但表格 ✅ 只有 3 项, 9/10 的 含义不明 (项数编号? 闭环率?), 待 v0.3 修订历史澄清
- **后续**: player 域 DDD Review 阶段, Ulysses 二审时确认

### 5.2 工具依赖缺口

- **#2/#3/#4 grpcurl 业务 mTLS 选型**: workspace `Cargo.toml` 是否含 grpcurl 工具? 待 9/1 ST 教训 §2.3 L3 grep 验证
- **#6 prometheus + alertmanager**: W36 末 prometheus CrashLoopBackOff 27h (per RGS-K3S-CLUSTER-STATUS v0.1 §3.5), 阶段 A3 修复待 SRE
- **#8 openssl 工具链**: k3s 节点已装 openssl, 但 cert 链验证 SOP 待 8/27 ST 导出 SOP 落档

### 5.3 业务级 mTLS 跑通缺口

- **#3 业务 mTLS OK + ledger 写入**: 1 笔测试交易的具体定义? 测试账户 / 测试金额 / 测试时间? 待 RGS-PHASE-C-PREP v0.1 §1 阶段 C 详化
- **#4 跨域 saga 真实交易**: player 充值金额 / economy 记账格式 / admin 审计日志格式, 3 域对齐待 W38 D1 阶段 C C6 跑通

### 5.4 审计日志 verify 缺口

- **#10 player.audit_event 24h ≥ 99%**: "24h 内 0 丢审计" 验证方式 = per admin Q2 决策 "增量 verify (最近 1000 条 / 24h)", 但 1000 条是否够? 24h 内 audit_event 平均条数? 待 W37 D5 阶段 B 收口实战

### 5.5 文档协同缺口

- **6 域 checklist 拆分时序**: 本文档 v0.1 是 6 域首个, economy / match / social / admin / batch 待 R3 阶段起草
- **DDD Review 配套**: 5 域 DDD Review 二审 (per AGENTS.md §3.x) 时是否需要先闭环本 checklist? 待 RGS-DDD-V0.X 模板升级
- **L-CAND-006 k8s secret 导出硬 ban**: #8 证书轮换走 `certs/` gitignored 目录 + 仅 cert fingerprint 写 `certs/MANIFEST.toml` 是 9/3 07:31 JST 拍板的例外段, 正式升 AGENTS.md §1.2 待 R4 季度评审

---

## 6. 修订历史 v0.1

| 版本 | 日期 | 审批 | 变更摘要 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 JST | 架构师(Mavis 接手 agent per DEC-008) | 初始建档: 9/3 11:06 JST 拆分 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 player 域 9-10 项 checklist 为独立文档 (C3 派生约束落地), 含 #1 UT (L1.1) / #2 IT mTLS / #3 E2E 业务 mTLS 1 跳 / #4 跨域 saga / #5 SLA 监控 / #6 告警 5xx > 1% / #7 部署健康 / #8 证书轮换 / #9 Schema 迁移 / #10 审计日志 10 项 (3 ✅ + 7 🟡), 配套 L1/L1.1/L1.2 DoD + 派生约束守护 + 已知缺口 5 条, 6 域拆分首发 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手

---

## 7. 附录

### 7.1 关联文档

- **源头**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 (player 域 9-10 项 checklist 原文)
- **R3 阶段**: RGS-DEVPLAN-2026-09-02 v0.3 §7 (token 推进 R1-R5 路线图) + §7 R3 (batch 解冻 + C2 + C3 + D1 派生约束)
- **Phase C 准备**: RGS-PHASE-C-PREP-2026-09-02 v0.1 §1 阶段 A/B/C/D
- **W37 启动**: RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 §6 W37 后续工作 5 天
- **5 域 ST 业务 mTLS**: RGS-K3S-CLUSTER-STATUS v0.1 §3.4 (1/5 现状)
- **闭环率基线**: RGS-WEEKLY-2026-W36 v0.3 §1.1-1.5
- **5 域 Lead**: RGS-RACI-PLAYER-V1 v1.1 (5 域独立 Lead 原则, per AGENTS.md §3)
- **commit 实证**:
  - `3cfeedb` (player 域 UT 137 tests, W36 D3 5 域独立 worktree 阶段)
  - `f5c0359` (W36 末 4 迁移全过 + AGENTS.md v0.6 + STATUS-SNAPSHOT v0.6.10)
  - `c52805b` (9/3 10:48 JST 5 域 L1.1 验证全过, player 141 tests / 0 failed)
- **DDD Review 模板**: AGENTS.md §3.x + DDD-REVIEW-TEMPLATE-v0.2.md (Ulysses 二审流程)

### 7.2 检查命令参考

```powershell
# #1 UT (L1.1)
cd D:/RustGameServer; cargo test --lib -p player-service 2>&1 | Select-Object -Last 20

# #2 IT mTLS health probe
grpcurl -cacert certs/ca.crt -cert certs/client.crt -key certs/client.key player-service:50051 grpc.health.v1.Health/Check

# #3 E2E 业务 mTLS 1 跳
grpcurl -cacert certs/ca.crt -cert certs/client.crt -key certs/client.key player-service:50051 player.v1.PlayerService/GetPlayer -d '{"player_id":"test-001"}'

# #4 跨域 saga 真实交易
grpcurl -cacert certs/ca.crt -cert certs/client.crt -key certs/client.key player-service:50051 player.v1.PlayerService/Recharge -d '{"player_id":"test-001","amount":1000}'

# #5 SLA 监控
kubectl get pods -l app=player-service -o jsonpath='{.items[*].status.containerStatuses[0].restartCount}'

# #6 告警
curl http://prometheus:9090/api/v1/alerts | Select-String player

# #7 部署健康
kubectl get pods -l app=player-service -o wide

# #8 证书轮换
openssl x509 -noout -fingerprint -sha256 -in certs/player-service-tls.crt

# #9 Schema 迁移
sqlx migrate run --source crates/player-service/migrations

# #10 审计日志
psql -c "SELECT COUNT(*) FROM player.audit_event WHERE created_at > now() - interval '24 hours';"
```

### 7.3 token-OLU 备注 (per 8/21 JST token-OLU 框架)

- **本文档**: 1 worker / 1 域 / 纯文档 = ~30K tokens (远低于 300K 预算, 留 270K 给 economy/match/social/admin/batch 5 文档)
- **6 文档合计**: 5 域 × 300K + batch 1 × 300K = 1.8M tokens (per RGS-DEVPLAN v0.3 §7 R3 C3 = 1.5M 预算, 略超 300K 待 W37 D2 SRE 拍板)

---

**文档结束**. player 域生产可用 9/10 项 checklist (v0.1) 落档, 等 W37 D2-W38 D2 阶段 B/C 实战 + Ulysses DDD Review 二审.
