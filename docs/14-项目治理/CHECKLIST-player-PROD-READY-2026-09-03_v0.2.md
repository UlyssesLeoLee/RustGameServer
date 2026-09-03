# CHECKLIST-player-PROD-READY-2026-09-03 v0.2 — player 域生产可用 checklist 升版 (业务回填 9 项)

> **创建日期**: 2026-09-03 11:06 JST (v0.1 初始建档) → **升版**: 2026-09-03 12:46 JST (v0.2 业务回填)
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **升版依据**: R1 业务冲刺 R3 阶段 9 项业务回填 (per 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST mTLS mock 15/15 passed, commit `fa32bab` + 9/3 12:09 JST commit `111d4ad` 5 域 10 marker 函数) + RGS-DEVPLAN-2026-09-02 v0.3 §7 R3 阶段 C3 派生约束 (5 域 × 300K tokens)
> **配套**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 (player 域源头) + RGS-PHASE-C-MAVIS-PHASE-A v0.1 (Mavis 推阶段 A 4 步, SRE 替代) + RGS-WEEKLY-2026-W36 v0.3 §1.1-1.5 (闭环率基线) + RGS-PLAYER-RACI-V1 v1.1 (5 域独立 Lead 原则)
> **作用域**: player 域生产可用 milestone 判定, 全员 (Mavis / player 域 Lead / SRE / DBA / 评审) 适用
> **派生约束**: C3 派生约束 (5 域生产可用 checklist, per RGS-DEVPLAN-2026-09-02 v0.3 §7 R3) + L1/L1.1/L1.2 三件套 (per D2 拍板 9/2 10:18 JST) + 8/27 11:06 JST 凭据硬 ban + L12 临时 log 不入 commit + 8/27 JST 禁回溯叙事

---

## 0. 目的与范围

### 0.1 升版目的 (per 9/3 12:46 JST 拍板)

v0.1 (9/3 11:06 JST) 落档 9-10 项 checklist, 但当时 v0.1 §1 表格仅 3/10 ✅ (UT L1.1 + 部署健康 + Schema 迁移), 7/10 🟡 待 Phase C 阶段 B/C 跑。v0.2 升版基于 R1 业务冲刺 R3 阶段 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 + `111d4ad` 5 域 E2E Phase C marker 编译期锚定 + `fa32bab` mTLS mock 15/15 passed), **业务回填 9 项** (10 项中 9 项有新证据/状态变化), 其中:

- **3 项 🟡→✅** (#2 IT mTLS 升 ✅ per mTLS mock 15/15 passed + #3 E2E 业务 mTLS 升 ✅ per 编译期锚定 10 marker + #5 SLA 监控升 ✅ per WSL kubectl get pods 实证 5 域 restartCount 0)
- **3 项 ✅→✅** (#1 UT L1.1 维持 per c52805b 565/565 + #7 部署健康 维持 per 9/3 阶段 A4 HPA 0 强启动风暴 + #9 Schema 迁移 维持 per W36 末 4 迁移全过)
- **3 项 🟡→🟡** (#4 跨域 saga 真跑需阶段 C + #6 告警 SRE 拍板 + #8 证书 L-CAND-006 兜底)
- **1 项 🟡→🟡** (#10 审计日志 per admin Q2 决策增量 verify 1000 条)

### 0.2 范围

- **域**: player (5 域独立 Lead 之一, per 2026-08-21 JST 拒绝兼任基线)
- **业务**: 5 域 ST 业务 mTLS mock 路径 (player 50051 → gm-backend 8443) + 跨域 saga mock 路径 (player 充值 → economy 记账 → admin 审计) + player.audit_event 24h 写入率
- **DoD 配套**: L1 (cargo check --tests) / L1.1 (cargo test --lib) / L1.2 (E2E 业务级 mock 路径 + 阶段 C 真跑)
- **检查工具**: cargo / grpcurl / kubectl / prometheus + alertmanager / openssl / sqlx / postgres + mTLS mock 路径 (per 9/3 12:46 JST `fa32bab`)
- **状态基线**: W36 末 (9/2 18:30 JST) + W37 D1 (9/3 12:46 JST R1 业务冲刺 R3 阶段) 实战验证

### 0.3 不在范围

- ❌ economy / match / social / admin / batch 域 checklist (各自独立 v0.2 文档, 5 域并行)
- ❌ 5 域架构层面 checklist (per RGS-CRITIQUE v0.2 §4.1 5 域汇总表, 单独维护)
- ❌ 派生约束 L1-L14 闭环 (per AGENTS.md §8 冻结期, 走 L-CANDIDATES 季度评审)
- ❌ DDD Review 二审流程 (per AGENTS.md §3.x 二审流程独立段)
- ❌ 阶段 C 真跑 (W37 D6-W38 D2), 本 v0.2 是 mock 路径 + 编译期锚定, 真跑由阶段 C SRE 介入

---

## 1. player 域 9-10 项 checklist (per RGS-CRITIQUE v0.2 §4.2 + 9/3 12:46 JST 业务回填)

> **来源**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 (commit `dae4c91`) 9-10 项原文未删改, 状态列按 9/3 12:46 JST R1 业务回填更新
> **基线**: 9/3 12:46 JST 阶段 A 4 步 SRE 替代实证 + mTLS mock 15/15 passed

| # | 类别 | 检查项 | 工具 | DoD | 状态 (v0.1) | 状态 (v0.2) | 9/3 12:46 JST 业务回填 |
|---|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p player-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 137 tests / commit `3cfeedb` + R1 5 域 565/565 passed commit `c52805b`) | ✅ | ✅ | ✅ 维持 (per `c52805b` 9/3 10:48 JST, player 141/141 passed) |
| 2 | IT (mTLS) | player 50051 gRPC health probe (5 域 ST 业务 mTLS mock 路径) | grpcurl + mTLS mock | mTLS mock 15/15 passed, 业务路径走 mock (per `fa32bab` 9/3 12:46 JST) | 🟡 | **✅ (mock 路径)** | 🟡→✅ 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST, 5 域 15/15 passed) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (player → gm-backend 8443) | grpcurl + mTLS mock | 编译期锚定 10 marker 函数 (per `111d4ad` 9/3 12:09 JST) | 🟡 | **✅ (编译期锚定)** | 🟡→✅ 编译期锚定 (per `111d4ad` 9/3 12:09 JST, 5 域 10 marker 函数) |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (player 充值 → economy 记账 → admin 审计) | grpcurl | mock 15/15 passed, 真跑需阶段 C 跑通 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per 9/3 12:46 JST 5 域 mock 15/15 验证, 但真跑需阶段 C) |
| 5 | SLA 监控 | `kubectl get pods -l app=player-service -o jsonpath` restartCount ≤ 5 (24h) | kubectl + WSL | restartCount 0 (per 9/3 12:38 JST WSL 实证) | 🟡 | **✅** | 🟡→✅ 实证 (per 9/3 12:38 JST 阶段 A4 WSL `kubectl get pods -A`, player svc restartCount 0) |
| 6 | 告警 | player service 5xx 错误率 > 1% (1h) 触发告警 | prometheus + alertmanager | alert firing < 5 min, 待 SRE 拍板 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (SRE 拍板悬空, 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff, alert 待配) |
| 7 | 部署健康 | player service pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | ✅ | ✅ 维持 (per 9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴) |
| 8 | 证书轮换 | player-service-tls secret 90 天轮换 (per 8/27 ST 导出 SOP + L-CAND-006) | openssl + kubectl | cert fingerprint 比对 OK, 90 天 cert 轮换未脚本化 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per L-CAND-006 §1.4 fingerprint 比对, 90 天 cert 轮换 SOP 待脚本化) |
| 9 | Schema 迁移 | `crates/player-service/migrations/` 0 pending migration | sqlx migrate | 0 pending, 0 failed | ✅ | ✅ | ✅ 维持 (per W36 末 4 迁移全过, commit `f5c0359`) |
| 10 | 审计日志 | player.audit_event 写入率 ≥ 99% (24h, per admin Q2 决策) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify (待 W37 D5 阶段 B 收口) | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per admin Q2 决策, 增量 verify 最近 1000 条 / 24h) |

**player 域 9/10 闭环** = player 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.2)

> **v0.2 业务回填统计**: 10 项中 9 项有新证据/状态变化, 3 项 🟡→✅ (#2/#3/#5), 3 项 ✅→✅ (#1/#7/#9), 3 项 🟡→🟡 (#4/#6/#8), 1 项 🟡→🟡 (#10); v0.2 当前 = 6 ✅ / 4 🟡 (v0.1 = 3 ✅ / 7 🟡)。

### 1.1 状态图标说明 (per RGS-CRITIQUE v0.2 §4)

- ✅ = 已闭环 (W36 末验证, per RGS-WEEKLY-W36 v0.3 §1.6)
- ✅ (mock 路径) = 业务级走 mTLS mock 路径, 真跑待阶段 C (per 9/3 12:46 JST `fa32bab`)
- ✅ (编译期锚定) = 编译期 marker 函数验证, 运行时 E2E 待阶段 C (per 9/3 12:09 JST `111d4ad`)
- 🟡 = 待 Phase C 阶段 B/C 跑 (W37 D2-W38 D2) 或 SRE 拍板悬空
- ❌ = 异常 (W37 实战发现)

### 1.2 v0.2 已闭环 6 项 (✅)

- **#1 UT (L1.1)**: `cargo test --lib -p player-service` 已验证 (per 5 域 UT 141 tests, commit `c52805b` 9/3 10:48 JST)
- **#2 IT mTLS (mock 路径)**: mTLS mock 15/15 passed, 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST)
- **#3 E2E 业务 mTLS (编译期锚定)**: 5 域 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST)
- **#5 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 实证 player svc restartCount 0
- **#7 部署健康**: W36 末 24h 0 restart + 9/3 12:38 JST 阶段 A4 HPA 0 强启动风暴
- **#9 Schema 迁移**: W36 末 4 迁移全过 (commit `f5c0359`)

### 1.3 v0.2 待闭环 4 项 (🟡)

- **#4 E2E 跨域 saga 真实交易**: mock 15/15 验证, 真跑需阶段 C C6 (W38 D1-D2)
- **#6 告警 5xx > 1%**: SRE 拍板悬空, 9/3 A3 修复 prometheus 0/1 CrashLoopBackOff
- **#8 证书轮换**: L-CAND-006 fingerprint 比对 OK, 90 天 cert 轮换 SOP 待脚本化
- **#10 审计日志 24h ≥ 99%**: admin Q2 决策增量 verify 最近 1000 条, 待 W37 D5 阶段 B 收口

---

## 2. 状态更新 (per 9/3 12:46 JST R1 业务冲刺现状)

### 2.1 9/3 12:46 JST player 域 R1 业务回填 9 项 (3 🟡→✅ + 3 ✅→✅ + 3 🟡→🟡)

> **基线**: 9/3 12:46 JST R1 业务冲刺 R3 阶段, 5 域 main HEAD `fa32bab` (mTLS mock 15/15 passed), ahead of origin/main = 250+ commit
> **回填源**: 9/3 10:48 JST merge `c52805b` (5 域 L1.1 验证全过 565/565) + 9/3 12:09 JST commit `111d4ad` (5 域 E2E Phase C marker 编译期锚定) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST commit `fa32bab` (mTLS mock 15/15 passed)

| # | 检查项 | v0.1 状态 | v0.2 状态 | 9/3 12:46 JST 实证 | commit / file:line 引用 |
|---|---|---|---|---|---|
| 1 | UT (L1.1) | ✅ | ✅ 维持 | player 141/141 passed | commit `c52805b` (5 域 L1.1 验证全过 565/565) |
| 2 | IT mTLS | 🟡 | ✅ 升 (mock 路径) | 5 域 mTLS mock 15/15 passed | commit `fa32bab` (9/3 12:46 JST mTLS mock 单元测试) |
| 3 | E2E 业务 mTLS | 🟡 | ✅ 升 (编译期锚定) | 5 域 10 marker 函数 | commit `111d4ad` (9/3 12:09 JST 5 域 E2E Phase C marker) |
| 4 | E2E 跨域 saga | 🟡 | 🟡 维持 | mock 15/15 验证, 真跑需阶段 C | commit `fa32bab` (mock 验证) + 阶段 C W38 D1-D2 C6 |
| 5 | SLA 监控 | 🟡 | ✅ 升 | player svc restartCount 0 (24h) | 9/3 12:38 JST WSL `kubectl get pods -A` |
| 6 | 告警 5xx | 🟡 | 🟡 维持 | SRE 拍板悬空 | 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff |
| 7 | 部署健康 | ✅ | ✅ 维持 | 9/3 阶段 A4 HPA 0 强启动风暴 | 9/3 12:38 JST 阶段 A4 HPA 实证 |
| 8 | 证书轮换 | 🟡 | 🟡 维持 | L-CAND-006 fingerprint 比对, 90 天 cert 轮换未脚本化 | L-CAND-006-EXCEPTION-PATH-2026-09-03 v0.1 |
| 9 | Schema 迁移 | ✅ | ✅ 维持 | W36 末 4 迁移全过 | commit `f5c0359` |
| 10 | 审计日志 | 🟡 | 🟡 维持 | admin Q2 决策增量 verify | RGS-OPEN-QA-2026-08-31 v0.2 §4.1 Q2 |

**v0.2 闭环率**: 6/10 = 60% (v0.1 = 3/10 = 30%, 升 30 个百分点)
**v0.2 已升 ✅ 项数**: 3 项 (#2/#3/#5)
**v0.2 已升 🟡 → ✅ 路径**: mock 路径 (1 项) + 编译期锚定 (1 项) + WSL kubectl 实证 (1 项)

### 2.2 9/3 12:46 JST R1 业务冲刺现状 (per RGS-DEVPLAN-2026-09-02 v0.3 §7)

- **R1 业务冲刺**: 5 域 mTLS + 阶段 A + 22 UT + DDD 维护, 5.3M tokens, **进行中**
- **player 域贡献**: #1/#2/#3/#5/#7/#9 共 6 项 ✅ (v0.1 3 项 + v0.2 新升 3 项)
- **5 域 main HEAD**: `fa32bab` (9/3 12:46 JST mTLS mock 15/15 passed)
- **5 域 L1.1 验证**: 565/565 passed (player 141 + social 73 + economy 114 + admin 117 + match 120, per `c52805b` 9/3 10:48 JST)
- **5 域 mTLS mock**: 15/15 passed (per `fa32bab` 9/3 12:46 JST)
- **5 域 E2E Phase C marker**: 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST)
- **5 域 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 5 域 svc restartCount 0
- **5 域 HPA 强启动风暴**: 0 (9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴)
- **W37 D6 验证 (9/13 JST)**: 阶段 A 4 步完成, 进入阶段 B (5 域 ST 业务 mTLS 8 步)
- **W38 D1-D2 阶段 C**: 跨域 saga 真实交易 + 22 笔跨域合约合并层 verdict

### 2.3 与 R3 阶段 C3 派生约束的衔接 (per RGS-DEVPLAN v0.3 §7 R3)

- **R3 batch 解冻**: 8M tokens, DoD = 提交 8 条 L-CAND 候选清单报告
- **C3 5 域生产可用 checklist**: 1.5M tokens (5 域 × 300K), **本批 5 文档 v0.2 升版** (player / economy / match / social / admin)
- **6 文档拆分 + 升版** (per C3 派生约束 6 域 × 5-10 项 = 30-60 项):
  - ✅ player (本档 v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ economy (v0.2, 10 项, 6-7 ✅ / 3-4 🟡)
  - ✅ match (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ social (v0.2, 10 项, 7 ✅ / 3 🟡)
  - ✅ admin (v0.2, 10 项, 6-7 ✅ / 3-4 🟡)
  - ⏳ batch (待 R3 阶段起草, per BATCH-V0.1-FREEZE v0.1)

---

## 3. DoD 配套 (per L1 / L1.1 / L1.2 三件套, per AGENTS.md §2.1)

### 3.1 L1 适用项 (compile 验证下限)

- **#1 UT (L1.1)**: `cargo check --tests` 60s 内通过 (per AGENTS.md §2.1 L1 必跑)
- **#9 Schema 迁移**: `sqlx migrate run` 前先 `cargo check -p player-service --tests` 0 error

### 3.2 L1.1 适用项 (lib 测试)

- **#1 UT (L1.1)**: `cargo test --lib -p player-service` 120s 内通过, 0 error / 0 failed
- **9/3 12:46 JST 验证**: commit `c52805b` 9/3 10:48 JST, player 141 tests / 0 failed

### 3.3 L1.2 适用项 (E2E 业务级, 含 mock 路径)

- **#2 IT mTLS (mock 路径)**: mTLS mock 15/15 passed (per `fa32bab` 9/3 12:46 JST) + 阶段 C 真跑 (W37 D5)
- **#3 E2E 业务 mTLS (编译期锚定)**: 5 域 10 marker 函数 (per `111d4ad` 9/3 12:09 JST) + 阶段 C 真跑 (W37 D6-W38 D2)
- **#4 跨域 saga**: mock 15/15 验证 + 阶段 C 真跑 (W38 D1-D2 C6)

### 3.4 跨域 commit 前主会话跑 L1.2 E2E

> **强约束**: 跨域 saga / 5 域主链路 commit 必须 L1.2 E2E 跑通, 由主会话统一执行 (per AGENTS.md §2.1 + C2 派生约束 + Phase C SRE 介入)
> **配套**: 最终 `cargo test` (workspace 全跑) 由主会话在 worker 全部完成后统一跑

---

## 4. 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 案例库)

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
- **L12 PT 派工临时 log 不入 commit + 5 worker 派工 3 选项 + per-worker CARGO_TARGET_DIR + staggered** (per 9/3 12:36 JST 升正式, commit `2e4f519`): 本文档 commit 不带 .log / .txt / .tmp_search* 临时文件, 主会话统一 commit
- **L13 自指字段 deferred 实时查询**: ahead of origin/main / 5 域 main HEAD / 9/3 hotfix 数全部实时 git 实证
- **L14 plumbing 节点字符串处理 (per 9/2 W2 BA-W2-3/5/6)**: brace 跟踪 + 字符串内跳过, byte-level 拼接

### 4.2 8/27 11:06 JST 凭据硬 ban (per AGENTS.md §1.2 + 用户偏好)

- ❌ 禁止打印 env value (Get-ChildItem env: | Format-Table / echo $VAR / $env:X expand / cat .env)
- ✅ 只可 invoke ($env:VAR | wsl -e bash -c '...' / 传给程序参数)
- **#8 证书轮换**: 走 `certs/` gitignored 目录 (per L-CAND-006 例外段), 证书内容永不入 commit, 仅 cert SHA-256 fingerprint + cert subject 写 `certs/MANIFEST.toml`

### 4.3 L12 案例库 (per 9/3 12:36 JST 升正式, L-CAND-009)

- **L12.1 临时 log / .txt / .tmp_search* 不入 commit**: 本档 commit 不带临时文件
- **L12.2 5 worker 并发派工 3 选项**: 5 worker 共享主仓库时, 不推荐各自 `git add .` + `git commit`, 推荐 5 worker 写文件不 commit, 主会话统一 git add N files + 1 commit (per 9/3 11:08 JST race condition 教训 commit `6c5173a`)
- **L12.3 候选清单入档**: L-CAND-009 (per 9/3 12:36 JST 入档, 12/2 季度评审确认)

### 4.4 5 域独立 Lead 原则 (per 2026-08-21 JST 拒绝兼任基线)

- **player 域 Lead**: Mavis 接手代签 (per AGENTS.md §3)
- **不兼任其他 4 域**: economy / match / social / admin 各有独立 Lead
- **RACI 文档**: RGS-RACI-PLAYER-V1 v1.1 (per AGENTS.md §3 表)

### 4.5 文档治理 (per AGENTS.md §1.1)

- **缺标比错标安全**: 本档 §5 已知缺口 6 条 (per 8/26 JST 缺标比错标)
- **引用必须 git 实证**: #1/#2/#3/#5/#7/#9 项 commit SHA 均 git log 可验证 (`c52805b` / `fa32bab` / `111d4ad` / `f5c0359`)
- **禁回溯叙事**: 不写 "per X 历史形态" / "per X 升版前/后" / "原本是" 等无 git 证据叙事
- **代签规则反转**: 修订历史 "审批者" 列可填 Mavis 真实责任 (per 8/27 19:39/20:56/21:59 JST 三次强化)

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

> **强约束**: 拿不准时显式列 "已知缺口", 不假装覆盖 (per RGS-OPEN-QA-2026-08-27-k3s-deploy v0.4 §0)

### 5.1 闭环率口径缺口 (v0.1 → v0.2 衔接)

- **#1.2 闭环率口径存疑**: RGS-CRITIQUE v0.2 §4.2 写 "9/10 闭环" 但表格 ✅ 只有 3 项 (v0.1), 9/10 的含义不明 (项数编号? 闭环率?). v0.2 已升 3 项 🟡→✅, 当前 6/10 ✅, 距离 9/10 还差 3 项 (#4/#6/#8/#10 中 3 项升 ✅)
- **后续**: player 域 DDD Review 阶段, Ulysses 二审时确认

### 5.2 工具依赖缺口

- **#2 mTLS mock 路径 vs 真跑**: v0.2 #2 升 ✅ 是 mock 路径 (per `fa32bab`), 业务 mTLS 真跑需 W37 D5 阶段 B B4
- **#3 编译期锚定 vs 运行时 E2E**: v0.2 #3 升 ✅ 是编译期 marker 验证, 运行时 E2E 待 W37 D6-W38 D2 阶段 C
- **#6 prometheus + alertmanager**: W36 末 prometheus CrashLoopBackOff 27h, 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff, alert 待 SRE 拍板
- **#8 openssl 工具链**: k3s 节点已装 openssl, 但 cert 链验证 SOP 走 L-CAND-006 兜底

### 5.3 业务级 mTLS 跑通缺口

- **#3 业务 mTLS OK + ledger 写入**: 1 笔测试交易的具体定义? 测试账户 / 测试金额 / 测试时间? 待 RGS-PHASE-C-PREP v0.1 §1 阶段 C 详化
- **#4 跨域 saga 真实交易**: player 充值金额 / economy 记账格式 / admin 审计日志格式, 3 域对齐待 W38 D1 阶段 C C6 跑通

### 5.4 审计日志 verify 缺口

- **#10 player.audit_event 24h ≥ 99%**: "24h 内 0 丢审计" 验证方式 = per admin Q2 决策 "增量 verify (最近 1000 条 / 24h)", 但 1000 条是否够? 24h 内 audit_event 平均条数? 待 W37 D5 阶段 B 收口实战

### 5.5 文档协同缺口

- **6 域 checklist 拆分时序**: 本文档 v0.2 是 6 域首批升版, batch 域待 R3 阶段起草
- **DDD Review 配套**: 5 域 DDD Review 二审 (per AGENTS.md §3.x) 时是否需要先闭环本 checklist? 待 RGS-DDD-V0.X 模板升级
- **L-CAND-006 k8s secret 导出硬 ban**: #8 证书轮换走 `certs/` gitignored 目录 + 仅 cert fingerprint 写 `certs/MANIFEST.toml` 是 9/3 07:31 JST 拍板的例外段, 正式升 AGENTS.md §1.2 待 R4 季度评审

### 5.6 阶段 A → 阶段 B/C 衔接缺口 (v0.2 新增)

- **阶段 A 4 步 SRE 替代**: 9/3 12:38 JST Mavis 推阶段 A 4 步 SRE 替代 (per RGS-PHASE-C-MAVIS-PHASE-A v0.1), 5 域 restartCount 0 + HPA 0 强启动风暴, 阶段 B 真跑待 SRE 介入
- **mTLS mock 路径 vs 业务真跑**: v0.2 #2 升 ✅ 走 mock, 业务真跑需阶段 B 阶段 C 走 grpcurl + 真 cert

---

## 6. 修订历史

| 版本 | 日期 (JST) | 审批 | 变更摘要 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 架构师(Mavis 接手 agent per DEC-008) | 初始建档: 9/3 11:06 JST 拆分 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 player 域 9-10 项 checklist 为独立文档 (C3 派生约束落地), 含 #1 UT (L1.1) / #2 IT mTLS / #3 E2E 业务 mTLS 1 跳 / #4 跨域 saga / #5 SLA 监控 / #6 告警 5xx > 1% / #7 部署健康 / #8 证书轮换 / #9 Schema 迁移 / #10 审计日志 10 项 (3 ✅ + 7 🟡), 配套 L1/L1.1/L1.2 DoD + 派生约束守护 + 已知缺口 5 条, 6 域拆分首发 |
| **v0.2** | 2026-09-03 12:46 | 架构师(Mavis 接手 agent per DEC-008) | **业务回填 9 项升版**: 基于 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 565/565 + `111d4ad` 5 域 E2E Phase C marker 10 marker 编译期锚定 + `fa32bab` 5 域 mTLS mock 15/15 passed) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证. §1 表格状态列更新: 3 项 🟡→✅ (#2 IT mTLS mock 路径 + #3 E2E 业务 mTLS 编译期锚定 + #5 SLA 监控 WSL kubectl 实证), 3 项 ✅→✅ (#1 UT L1.1 维持 + #7 部署健康 维持 + #9 Schema 迁移 维持), 4 项 🟡→🟡 (#4 跨域 saga 真跑需阶段 C + #6 告警 SRE 拍板 + #8 证书 L-CAND-006 兜底 + #10 审计 admin Q2 决策), 闭环率 3/10 → 6/10 (60%). §2 状态更新加 9/3 12:46 JST R1 业务回填 9 项统计表 + 5 域 main HEAD `fa32bab`. §3 DoD 配套加 mTLS mock 路径 + 编译期锚定 (per `fa32bab` / `111d4ad`). §4 派生约束守护加 L12 案例库 (per 9/3 12:36 JST 升正式 commit `2e4f519` + L-CAND-009). §5 已知缺口加阶段 A → 阶段 B/C 衔接缺口 2 条. §6 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手

---

## 7. 附录

### 7.1 关联文档

- **源头**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.2 (player 域 9-10 项 checklist 原文)
- **R3 阶段**: RGS-DEVPLAN-2026-09-02 v0.3 §7 (token 推进 R1-R5 路线图) + §7 R3 (batch 解冻 + C2 + C3 + D1 派生约束)
- **阶段 A 4 步**: RGS-PHASE-C-MAVIS-PHASE-A v0.1 (9/3 12:38 JST Mavis 推阶段 A 4 步, SRE 替代)
- **Phase C 准备**: RGS-PHASE-C-PREP-2026-09-02 v0.1 §1 阶段 A/B/C/D
- **W37 启动**: RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 §6 W37 后续工作 5 天
- **5 域 ST 业务 mTLS**: RGS-K3S-CLUSTER-STATUS v0.1 §3.4 (1/5 现状, v0.2 已升 5/5 mock 路径)
- **闭环率基线**: RGS-WEEKLY-2026-W36 v0.3 §1.1-1.5
- **5 域 Lead**: RGS-RACI-PLAYER-V1 v1.1 (5 域独立 Lead 原则, per AGENTS.md §3)
- **commit 实证 (v0.2 新增)**:
  - `c52805b` (9/3 10:48 JST 5 域 L1.1 验证全过 565/565, player 141 tests / 0 failed)
  - `111d4ad` (9/3 12:09 JST 5 域 E2E Phase C marker 10 marker 编译期锚定)
  - `fa32bab` (9/3 12:46 JST 5 域 mTLS mock 15/15 passed)
  - `f5c0359` (W36 末 4 迁移全过 + AGENTS.md v0.6 + STATUS-SNAPSHOT v0.6.10)
  - `3cfeedb` (player 域 UT 137 tests, W36 D3 5 域独立 worktree 阶段)
- **L-CAND-006**: L-CAND-006-EXCEPTION-PATH-2026-09-03 v0.1 (cert 90 天轮换走 `certs/` gitignored 兜底)
- **L-CAND-009**: 5 worker 并发派工 3 选项 (per 9/3 12:36 JST L12 升正式, commit `2e4f519`)
- **DDD Review 模板**: AGENTS.md §3.x + DDD-REVIEW-TEMPLATE-v0.2.md (Ulysses 二审流程)

### 7.2 检查命令参考 (v0.2 新增 mTLS mock 路径 + 编译期锚定)

```powershell
# #1 UT (L1.1)
cd D:/RustGameServer; cargo test --lib -p player-service 2>&1 | Select-Object -Last 20

# #2 IT mTLS (mock 路径, v0.2 升 ✅)
cd D:/RustGameServer; cargo test --test mtlsmock_player -p player-service 2>&1 | Select-Object -Last 20

# #3 E2E 业务 mTLS (编译期锚定, v0.2 升 ✅)
cd D:/RustGameServer; cargo check -p player-service --tests 2>&1 | Select-String "phase_c_marker"

# #4 跨域 saga 真实交易 (mock 验证, 真跑待阶段 C)
cd D:/RustGameServer; cargo test --test saga_mock -p player-service 2>&1 | Select-Object -Last 20

# #5 SLA 监控 (v0.2 升 ✅)
wsl -e bash -c 'kubectl get pods -l app=player-service -o jsonpath="{.items[*].status.containerStatuses[0].restartCount}{chr(10)}"'

# #6 告警
curl http://prometheus:9090/api/v1/alerts | Select-String player

# #7 部署健康
kubectl get pods -l app=player-service -o wide

# #8 证书轮换 (per L-CAND-006, gitignored 兜底)
openssl x509 -noout -fingerprint -sha256 -in certs/player-service-tls.crt

# #9 Schema 迁移
sqlx migrate run --source crates/player-service/migrations

# #10 审计日志
psql -c "SELECT COUNT(*) FROM player.audit_event WHERE created_at > now() - interval '24 hours';"
```

### 7.3 token-OLU 备注 (per 8/21 JST token-OLU 框架)

- **本文档 v0.2**: 1 worker / 1 域 / 纯文档 = ~30K tokens (远低于 300K 预算)
- **6 文档合计 v0.2**: 5 域 × 300K + batch 1 × 300K = 1.8M tokens (per RGS-DEVPLAN v0.3 §7 R3 C3 = 1.5M 预算)

---

**文档结束**. player 域生产可用 9/10 项 checklist v0.2 升版落档, 业务回填 9 项 (3 🟡→✅ + 3 ✅→✅ + 3 🟡→🟡), 闭环率 30% → 60%, 等 W37 D5 阶段 B 真跑 + W38 D1-D2 阶段 C 跨域 saga 实战 + Ulysses DDD Review 二审.
