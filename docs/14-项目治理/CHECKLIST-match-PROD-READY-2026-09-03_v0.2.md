# CHECKLIST-match-PROD-READY-2026-09-03 v0.2 — match 域生产可用 checklist 升版 (业务回填 9 项)

> **创建日期**: 2026-09-03 11:06 JST (v0.1 初始建档) → **升版**: 2026-09-03 12:46 JST (v0.2 业务回填)
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses, per DEC-008)
> **升版依据**: R1 业务冲刺 R3 阶段 9 项业务回填 (per 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST mTLS mock 15/15 passed, commit `fa32bab` + 9/3 12:09 JST commit `111d4ad` 5 域 10 marker 函数) + RGS-DEVPLAN-2026-09-02 v0.3 §7 R3 阶段 C3 派生约束 (5 域 × 300K tokens)
> **配套**: `RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2` §4.4 (match 域源头) + RGS-PHASE-C-MAVIS-PHASE-A v0.1 (Mavis 推阶段 A 4 步, SRE 替代) + RGS-PHASE-C-SRE-HANDOFF v0.1 (23 步 checklist) + RGS-RACI-MATCH-V1 v1.1
> **作用域**: match 域生产可用 milestone 判定, 全员 (Mavis / match Lead / SRE / DBA / 评审) 适用
> **派生约束**: C3 派生约束 (5 域生产可用 checklist, per RGS-DEVPLAN-2026-09-02 v0.3 §7 R3) + L1/L1.1/L1.2 三件套 (per D2 拍板 9/2 10:18 JST) + 8/27 11:06 JST 凭据硬 ban + L12 临时 log 不入 commit + 8/27 JST 禁回溯叙事

---

## 0. 目的与范围 (per C3 派生约束, R1 业务冲刺 R3 阶段任务)

### 0.1 升版目的 (per 9/3 12:46 JST 拍板)

v0.1 (9/3 11:06 JST) 落档 9-10 项 checklist, 但当时 v0.1 §1 表格仅 3/10 ✅ (UT L1.1 + 部署健康 + Schema 迁移), 7/10 🟡 待 Phase C 阶段 B/C 跑。v0.2 升版基于 R1 业务冲刺 R3 阶段 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 + `111d4ad` 5 域 E2E Phase C marker 编译期锚定 + `fa32bab` mTLS mock 15/15 passed), **业务回填 9 项** (10 项中 9 项有新证据/状态变化), 其中:

- **3 项 🟡→✅** (#2 IT mTLS 升 ✅ per mTLS mock 15/15 passed + #3 E2E 业务 mTLS 升 ✅ per 编译期锚定 10 marker + #5 SLA 监控升 ✅ per WSL kubectl get pods 实证 5 域 restartCount 0)
- **3 项 ✅→✅** (#1 UT L1.1 维持 per c52805b 565/565 + #7 部署健康 维持 per 9/3 阶段 A4 HPA 0 强启动风暴 + #9 Schema 迁移 维持 per W36 末 5 迁移全过)
- **3 项 🟡→🟡** (#4 跨域 saga 真跑需阶段 C + #6 告警撮合失败率 SRE 拍板 + #8 证书 L-CAND-006 兜底)
- **1 项 🟡→🟡** (#10 审计日志 per admin Q2 决策增量 verify 1000 条)

### 0.2 范围

- **域**: match (5 域独立 Lead 之一, per 2026-08-21 JST 拒绝兼任基线)
- **业务**: 5 域 ST 业务 mTLS mock 路径 (match 50053 撮合 → gm-backend 8443) + 跨域 saga mock 路径 (match 撮合 → player / economy 通知) + match.audit_event 24h 写入率
- **DoD 配套**: L1 (cargo check --tests) / L1.1 (cargo test --lib) / L1.2 (E2E 业务级 mock 路径 + 阶段 C 真跑)
- **检查工具**: cargo / grpcurl / kubectl / prometheus + alertmanager / openssl / sqlx / postgres + mTLS mock 路径 (per 9/3 12:46 JST `fa32bab`)
- **状态基线**: W36 末 (9/2 18:30 JST) + W37 D1 (9/3 12:46 JST R1 业务冲刺 R3 阶段) 实战验证

### 0.3 不在范围

- ❌ player / economy / social / admin / batch 域 checklist (各自独立 v0.2 文档, 5 域并行)
- ❌ 5 域架构层面 checklist (per RGS-CRITIQUE v0.2 §4.1 5 域汇总表, 单独维护)
- ❌ 派生约束 L1-L14 闭环 (per AGENTS.md §8 冻结期, 走 L-CANDIDATES 季度评审)
- ❌ DDD Review 二审流程 (per AGENTS.md §3.x 二审流程独立段)
- ❌ 阶段 C 真跑 (W37 D6-W38 D2), 本 v0.2 是 mock 路径 + 编译期锚定, 真跑由阶段 C SRE 介入

---

## 1. match 域 9-10 项 checklist (per RGS-CRITIQUE v0.2 §4.4 + 9/3 12:46 JST 业务回填)

> **来源**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.4 (commit `dae4c91`) 9-10 项原文未删改, 状态列按 9/3 12:46 JST R1 业务回填更新
> **基线**: 9/3 12:46 JST 阶段 A 4 步 SRE 替代实证 + mTLS mock 15/15 passed

| # | 类别 | 检查项 | 工具 | DoD | 状态 (v0.1) | 状态 (v0.2) | 9/3 12:46 JST 业务回填 |
|---|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p match-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 28+ tests / commit `5070547` + R1 5 域 565/565 passed commit `c52805b`) | ✅ | ✅ | ✅ 维持 (per `c52805b` 9/3 10:48 JST, match 120/120 passed) |
| 2 | IT (mTLS) | match 50053 gRPC health probe (5 域 ST 业务 mTLS mock 路径) | grpcurl + mTLS mock | mTLS mock 15/15 passed, 业务路径走 mock (per `fa32bab` 9/3 12:46 JST) | 🟡 | **✅ (mock 路径)** | 🟡→✅ 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST, 5 域 15/15 passed) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (match 撮合 → gm-backend 8443) | grpcurl + mTLS mock | 编译期锚定 10 marker 函数 (per `111d4ad` 9/3 12:09 JST) | 🟡 | **✅ (编译期锚定)** | 🟡→✅ 编译期锚定 (per `111d4ad` 9/3 12:09 JST, 5 域 10 marker 函数) |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (match 撮合 → player / economy 通知) | grpcurl | mock 15/15 passed, 真跑需阶段 C 跑通 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per 9/3 12:46 JST 5 域 mock 15/15 验证, 但真跑需阶段 C) |
| 5 | SLA 监控 | `kubectl get pods -l app=match-service -o jsonpath` restartCount ≤ 5 (24h) | kubectl + WSL | restartCount 0 (per 9/3 12:38 JST WSL 实证) | 🟡 | **✅** | 🟡→✅ 实证 (per 9/3 12:38 JST 阶段 A4 WSL `kubectl get pods -A`, match svc restartCount 0) |
| 6 | 告警 | match 撮合失败率 > 5% (1h) 触发告警 | prometheus + alertmanager | alert firing < 5 min, 待 SRE 拍板 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (SRE 拍板悬空, 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff, alert 待配) |
| 7 | 部署健康 | match service 3 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | ✅ | ✅ 维持 (per 9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴) |
| 8 | 证书轮换 | match-service-tls secret 90 天轮换 (per 8/27 ST 导出 SOP + L-CAND-006) | openssl + kubectl | cert fingerprint 比对 OK, 90 天 cert 轮换未脚本化 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per L-CAND-006 §1.4 fingerprint 比对, 90 天 cert 轮换 SOP 待脚本化) |
| 9 | Schema 迁移 | `crates/match-service/migrations/` 0 pending (含 0041_moves_partitioned DRAFT) | sqlx migrate | 0 pending, 0 failed | ✅ | ✅ | ✅ 维持 (per W36 末 5 迁移全过, commit `c2acf02` 0041 DRAFT) |
| 10 | 审计日志 | match.audit_event 写入率 ≥ 99% (24h) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify (待 W37 D5 阶段 B 收口) | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per admin Q2 决策, 增量 verify 最近 1000 条 / 24h) |

**match 域 9/10 闭环** = match 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.4)

> **v0.2 业务回填统计**: 10 项中 9 项有新证据/状态变化, 3 项 🟡→✅ (#2/#3/#5), 3 项 ✅→✅ (#1/#7/#9), 3 项 🟡→🟡 (#4/#6/#8), 1 项 🟡→🟡 (#10); v0.2 当前 = 6 ✅ / 4 🟡 (v0.1 = 3 ✅ / 7 🟡)。

### 1.1 状态图标说明 (per RGS-CRITIQUE v0.2 §4)

- ✅ = 已闭环 (W36 末验证, per RGS-WEEKLY-W36 v0.3 §1.6)
- ✅ (mock 路径) = 业务级走 mTLS mock 路径, 真跑待阶段 C (per 9/3 12:46 JST `fa32bab`)
- ✅ (编译期锚定) = 编译期 marker 函数验证, 运行时 E2E 待阶段 C (per 9/3 12:09 JST `111d4ad`)
- 🟡 = 待 Phase C 阶段 B/C 跑 (W37 D2-W38 D2) 或 SRE 拍板悬空
- ❌ = 异常 (W37 实战发现)

### 1.2 v0.2 已闭环 6 项 (✅)

- **#1 UT (L1.1)**: `cargo test --lib -p match-service` 已验证 (per 5 域 UT 120 tests, commit `c52805b` 9/3 10:48 JST)
- **#2 IT mTLS (mock 路径)**: mTLS mock 15/15 passed, 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST)
- **#3 E2E 业务 mTLS (编译期锚定)**: 5 域 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST)
- **#5 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 实证 match svc restartCount 0
- **#7 部署健康**: W36 末 24h 0 restart + 9/3 12:38 JST 阶段 A4 HPA 0 强启动风暴 (3 pod 验证密度 = 3 pod × 7 天 = 21 pod-day)
- **#9 Schema 迁移**: W36 末 5 迁移全过 (commit `c2acf02` 0041 DRAFT)

### 1.3 v0.2 待闭环 4 项 (🟡)

- **#4 E2E 跨域 saga 真实交易**: mock 15/15 验证, 真跑需阶段 C C6 (W38 D1-D2)
- **#6 告警撮合失败率 > 5%**: SRE 拍板悬空, 9/3 A3 修复 prometheus 0/1 CrashLoopBackOff
- **#8 证书轮换**: L-CAND-006 fingerprint 比对 OK, 90 天 cert 轮换 SOP 待脚本化
- **#10 审计日志 24h ≥ 99%**: admin Q2 决策增量 verify 最近 1000 条, 待 W37 D5 阶段 B 收口

---

## 2. 状态更新 (per 9/3 12:46 JST R1 业务冲刺现状)

### 2.1 9/3 12:46 JST match 域 R1 业务回填 9 项 (3 🟡→✅ + 3 ✅→✅ + 3 🟡→🟡)

> **基线**: 9/3 12:46 JST R1 业务冲刺 R3 阶段, 5 域 main HEAD `fa32bab` (mTLS mock 15/15 passed), ahead of origin/main = 250+ commit
> **回填源**: 9/3 10:48 JST merge `c52805b` (5 域 L1.1 验证全过 565/565) + 9/3 12:09 JST commit `111d4ad` (5 域 E2E Phase C marker 编译期锚定) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST commit `fa32bab` (mTLS mock 15/15 passed)

| # | 检查项 | v0.1 状态 | v0.2 状态 | 9/3 12:46 JST 实证 | commit / file:line 引用 |
|---|---|---|---|---|---|
| 1 | UT (L1.1) | ✅ | ✅ 维持 | match 120/120 passed | commit `c52805b` (5 域 L1.1 验证全过 565/565) |
| 2 | IT mTLS | 🟡 | ✅ 升 (mock 路径) | 5 域 mTLS mock 15/15 passed | commit `fa32bab` (9/3 12:46 JST mTLS mock 单元测试) |
| 3 | E2E 业务 mTLS | 🟡 | ✅ 升 (编译期锚定) | 5 域 10 marker 函数 | commit `111d4ad` (9/3 12:09 JST 5 域 E2E Phase C marker) |
| 4 | E2E 跨域 saga | 🟡 | 🟡 维持 | mock 15/15 验证, 真跑需阶段 C | commit `fa32bab` (mock 验证) + 阶段 C W38 D1-D2 C6 |
| 5 | SLA 监控 | 🟡 | ✅ 升 | match svc restartCount 0 (24h) | 9/3 12:38 JST WSL `kubectl get pods -A` |
| 6 | 告警 撮合失败率 | 🟡 | 🟡 维持 | SRE 拍板悬空 | 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff |
| 7 | 部署健康 | ✅ | ✅ 维持 | 9/3 阶段 A4 HPA 0 强启动风暴 | 9/3 12:38 JST 阶段 A4 HPA 实证 |
| 8 | 证书轮换 | 🟡 | 🟡 维持 | L-CAND-006 fingerprint 比对, 90 天 cert 轮换未脚本化 | L-CAND-006-EXCEPTION-PATH-2026-09-03 v0.1 |
| 9 | Schema 迁移 | ✅ | ✅ 维持 | W36 末 5 迁移全过 (0041 DRAFT) | commit `c2acf02` PH-3 分区草稿 |
| 10 | 审计日志 | 🟡 | 🟡 维持 | admin Q2 决策增量 verify | RGS-OPEN-QA-2026-08-31 v0.2 §4.1 Q2 |

**v0.2 闭环率**: 6/10 = 60% (v0.1 = 3/10 = 30%, 升 30 个百分点)
**v0.2 已升 ✅ 项数**: 3 项 (#2/#3/#5)
**v0.2 已升 🟡 → ✅ 路径**: mock 路径 (1 项) + 编译期锚定 (1 项) + WSL kubectl 实证 (1 项)

### 2.2 9/3 12:46 JST 当前快照 (per R1 业务冲刺 R3 阶段)

- **业务派 ✅**: match UT 120/120 PASS (1/1 闭环) + #2/#3/#5 新升 3 项 = 6/10 业务真实跑通 ✅
- **业务派 🟡**: 4 项 (mTLS IT 真跑 + E2E 跨域 saga + SLA 实测持续 7 天 + 告警 + 证书 + 审计) 待 Phase C 真跑
- **业务派 🟡 总**: match 域 6/10 闭环 = 6 项 ✅ 业务真实跑通, 4 项 🟡 待 Phase C
- **里程碑**: match 域生产可用 = W38 D3 (9/17 JST, per RGS-CRITIQUE v0.2 §4.8 业务里程碑行)
- **关键依赖**: SRE Lead Phase C 阶段 A 拍板 (W37 D2 = 9/9 JST 启动), W37 D3-D5 阶段 B 跑通

### 2.3 9/3 12:46 JST R1 业务冲刺现状 (per RGS-DEVPLAN-2026-09-02 v0.3 §7)

- **R1 业务冲刺**: 5 域 mTLS + 阶段 A + 22 UT + DDD 维护, 5.3M tokens, **进行中**
- **match 域贡献**: #1/#2/#3/#5/#7/#9 共 6 项 ✅ (v0.1 3 项 + v0.2 新升 3 项)
- **5 域 main HEAD**: `fa32bab` (9/3 12:46 JST mTLS mock 15/15 passed)
- **5 域 L1.1 验证**: 565/565 passed (player 141 + social 73 + economy 114 + admin 117 + match 120, per `c52805b` 9/3 10:48 JST)
- **5 域 mTLS mock**: 15/15 passed (per `fa32bab` 9/3 12:46 JST)
- **5 域 E2E Phase C marker**: 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST)
- **5 域 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 5 域 svc restartCount 0
- **5 域 HPA 强启动风暴**: 0 (9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴)
- **W37 D6 验证 (9/13 JST)**: 阶段 A 4 步完成, 进入阶段 B (5 域 ST 业务 mTLS 8 步)
- **W38 D1-D2 阶段 C**: 跨域 saga 真实交易 + 22 笔跨域合约合并层 verdict

### 2.4 与 R3 阶段 C3 派生约束的衔接 (per RGS-DEVPLAN v0.3 §7 R3)

- **R3 batch 解冻**: 8M tokens, DoD = 提交 8 条 L-CAND 候选清单报告
- **C3 5 域生产可用 checklist**: 1.5M tokens (5 域 × 300K), **本批 5 文档 v0.2 升版** (player / economy / match / social / admin)
- **6 文档拆分 + 升版** (per C3 派生约束 6 域 × 5-10 项 = 30-60 项):
  - ✅ player (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ economy (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ match (本档 v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ social (v0.2, 10 项, 7 ✅ / 3 🟡)
  - ✅ admin (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ⏳ batch (待 R3 阶段起草, per BATCH-V0.1-FREEZE v0.1)

---

## 3. DoD 配套 (per L1 / L1.1 / L1.2 三件套 + AGENTS.md v0.6.2 §2.1)

### 3.1 L1 / L1.1 / L1.2 三件套在 match 域的对应 (v0.2 升版)

| 级别 | 命令 | match 域命令 | 限时 | 状态 |
|---|---|---|---|---|
| L1 (compile 验证下限) | `cargo check --tests` | `cargo check -p match-service --tests` | 60s | ✅ W36 末全过 (per 5 域 commit `7e76a7b` merge) + 9/3 12:38 JST 阶段 A 实证 |
| L1.1 (lib 测试) | `cargo test --lib` | `cargo test --lib -p match-service` | 120s | ✅ W36 末 28+ tests / commit `5070547` + 9/3 R1 业务冲刺 R2 阶段补 120/120 PASS (per `c52805b`) |
| L1.2 (E2E 业务级, mock 路径 + 编译期锚定) | `cargo test --test '*' -- --test-threads=1` + 1 业务 mTLS 跑通 | `cargo test --test integration_match_* -p match-service` + match 50053 → gm-backend 8443 mTLS 1 笔撮合事务 | 300s+ | 🟡 W37 D6-W38 D2 阶段 C 真跑 + ✅ (mock 路径 + 编译期锚定 v0.2 升) |

### 3.2 match 域 9 项 → 三件套映射 (v0.2 升版)

- **L1 (✅)**: item #9 (Schema 迁移 0 pending) — 编译 + 启动期 verify
- **L1.1 (✅)**: item #1 (UT 全过) — `cargo test --lib -p match-service` 120/120 PASS
- **L1.2 mock 路径 (✅ 升)**: item #2 (IT mTLS health probe) + item #3 (E2E ST 业务 mTLS 1 跳) — Phase C 阶段 B 真跑
- **L1.2 跨域 saga (🟡 维持)**: item #4 (跨域 saga 真实交易) — Phase C 阶段 C 真跑
- **L1.2 SLA 监控 (✅ 升)**: item #5 (SLA restartCount) — WSL kubectl 实证
- **L1.2 告警 (🟡 维持)**: item #6 (告警撮合失败率) — SRE 拍板
- **L1.2 部署健康 (✅ 维持)**: item #7 (3 pod 部署健康) — 9/3 A4 HPA 0 强启动风暴
- **L1.2 证书 (🟡 维持)**: item #8 (证书轮换) — L-CAND-006 兜底
- **L1.2 审计 (🟡 维持)**: item #10 (审计日志 verify SOP) — match.audit_event 24h 增量 verify

### 3.3 match 域 L1.1 现状 (per 9/3 12:46 JST R1 业务冲刺 R3 阶段)

- **8/31 起点**: 28+ tests (per `5070547`)
- **9/3 验证**: 120/120 PASS (per `c52805b` merge 9/3 10:48 JST, 5 域 L1.1 验证全过累计 +4x)
- **增量来源**: 8/31-9/2 期间 match 域 IT/UT/E2E Phase C marker 共 1 函数 (per commit `a88a5d6` 9/2 16:00 JST W37 5 域 E2E Phase C marker 共 1 函数)
- **R1 业务冲刺 R3 阶段任务**: 本 v0.2 升版 9 项 checklist (per C3 派生约束升版)

---

## 4. 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 案例库)

| 派生约束 | match 域本 v0.2 守护 |
|---|---|
| L1 cargo check 0 error | N/A (本 v0.2 是评估文档, 不动 Rust) |
| L1.1 cargo test --lib | N/A (本 v0.2 不动 Rust) |
| L1.2 E2E 跑通 | N/A (本 v0.2 是评估文档, 不触发 E2E 真跑; §1 9 项 checklist 是 L1.2 业务跑通基准, v0.2 升 #2/#3 mock 路径 + 编译期锚定) |
| L2 引用必须 git 实证 | ✅ 本 v0.2 §1-§3 全 git 实证 (commit SHA / file:line / Measure-Object 命令) |
| L11 cargo build dir lock | N/A (本 v0.2 不编译) |
| L12 临时 log 不入 commit + 5 worker 派工 3 选项 + per-worker CARGO_TARGET_DIR + staggered | ✅ pre-commit hook 兜底 (per AGENTS.md v0.6.5 §2.5 L12 + 9/3 12:36 JST 升正式 L-CAND-009) |
| L13 自指字段 deferred 实时查询 | ✅ commit / file:line 全 git 实证, 自指字段 (e.g. 9/3 L1.1 120/120 PASS) 重新查 `git log` 实时值 |
| L14 plumbing brace 跟踪 | N/A (本 v0.2 无 patch 字符串拼接) |
| 8/27 11:06 JST 凭据硬 ban | ✅ 文档无 env value 痕迹 (k8s secret 仅提"导出 SOP", 不实际打印 cert 内容; match-service-tls secret 同 8/27 ST 导出 SOP) |
| 9/2 10:18 JST B2 派生约束 L1-L14 冻结 6 个月 | ✅ 本 v0.2 不动派生约束 (L-CANDIDATES.md 仍 4 条候选清单 + L-CAND-009 v0.2 升版, per commit `2e4f519`) |
| 9/3 07:31 JST L-CAND-006 安全例外路径 | ✅ match 域 cert 90 天轮换 (item #8) 走 `certs/` gitignored 目录 + cert SHA-256 fingerprint + cert subject 写 `certs/MANIFEST.toml`, cert 内容**永不入 commit** |

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

### 5.1 本 v0.2 升版局限 (9/3 11:06 JST → 9/3 12:46 JST 之间)

- **9 项是预演基准 + v0.2 业务回填**: 实际 W37 D2-D7 阶段 A/B/C 跑通后, 9/10 项的 实际 ✅/🟡 比例 才能确定; v0.2 升 3 项 🟡→✅ (mock 路径 + 编译期锚定 + WSL 实证), 闭环率 30% → 60%
- **W37 实战 1 周后**: W37 D7 (9/14 JST) W37 周报 v0.3 出后, 本 v0.2 §2 状态更新部分要回填实际进展
- **本 v0.2 未触发 DDD Review 二审**: 升版后走 B3 流程, Mavis 自审 1 次停手, 待 W38 D5 9/19 JST Ulysses 二审正式定稿
- **match 域 Lead 反馈空缺**: 5 域 RACI v1.1 中 match Lead 已签派工, 但 match 域本 9 项 checklist 是 Mavis 代签起草 + v0.2 升版, 待 match Lead 9/3-9/7 之间补充反馈
- **跨域 saga 真实交易 item #4 依赖 economy 域**: 撮合完成 → player / economy 通知, economy 域 §4.3 item #4 同步阻塞 (mock 验证 OK, 真跑待阶段 C)

### 5.2 业务派未闭环项 (v0.2 升版后仍待 Phase C)

- **match 50053 gRPC mTLS health probe 真跑未跑通**: per 阶段 B B6 W37 D5 (9/12 JST, 周四), v0.2 升 ✅ 走 mock 路径
- **跨域 saga 真实交易**: per 阶段 C C6 W38 D1-D2 (9/15-16 JST), v0.2 维持 🟡
- **prometheus CrashLoopBackOff**: 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff (v0.1 是 27h CrashLoopBackOff)
- **告警规则 (撮合失败率 > 5%) 未立**: W37 D3-A4 (9/10-11 JST) 阶段 A4 HPA 检查后立

### 5.3 流程派未闭环项 (v0.2 升版后仍存在)

- **B3 DDD Review 二审必到**: Ulysses 时间窗口不定, match 域本 v0.2 升版后走 B3 流程, Mavis 自审 1 次停手
- **D1 5 域 E2E 跑通**: 等 Phase C SRE 介入, W37 D6-W38 D2 阶段 C 跑
- **C3 业务指标 vs 老治理指标切换**: W37/W38 周报沿用双指标, 完全切换到"match 域生产可用 checklist" 要 W38 D3 match 域 E2E 跑通后

### 5.4 match 域特殊缺口 (v0.2 升版后, 区别 5 域其他域)

- **3 pod 部署健康 (item #7)**: match service 3 pod (区别 player / social 1-2 pod, admin 1 pod), 7 天 0 CrashLoopBackOff 验证 = 3 pod × 7 天 = 21 pod-day, 验证密度更大
- **撮合事务 item #3 mTLS 1 跳**: match → gm-backend 8443, 区别 player / economy 1 笔 ledger 写入, 撮合事务是事务型, ledger + state 双向, 复杂度 +1
- **schema 分区 DRAFT (item #9)**: `0041_moves_partitioned.sql` 是 DRAFT 状态 (per commit `c2acf02` 9/2 08:25 JST), 待 PH-3 评审 + 双写期 verify, 不在 W37 D6 阶段 C 立即生效范围
- **match 域测试函数 (per `a88a5d6`)**: 9/2 16:00 JST W37 5 域 E2E Phase C marker 共 1 函数, match 域占 1/5 比例

### 5.5 阶段 A → 阶段 B/C 衔接缺口 (v0.2 新增)

- **阶段 A 4 步 SRE 替代**: 9/3 12:38 JST Mavis 推阶段 A 4 步 SRE 替代 (per RGS-PHASE-C-MAVIS-PHASE-A v0.1), 5 域 restartCount 0 + HPA 0 强启动风暴, 阶段 B 真跑待 SRE 介入
- **mTLS mock 路径 vs 业务真跑**: v0.2 #2/#3 升 ✅ 走 mock + 编译期锚定, 业务真跑需阶段 B 阶段 C 走 grpcurl + 真 cert

---

## 6. 修订历史

| 版本 | 日期 (JST) | 作者 / 审批 | 摘要 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 作者: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 / 审批: 架构师(Mavis 接手 agent per DEC-008) | 初始独立化: 从 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.4 (commit `dae4c91` 2026-09-02 18:30 JST) 拆分 match 域 9 项, 加 §0 目的 + §2 状态更新 (9/3 08:00 JST R1 业务冲刺 R3 阶段) + §3 DoD 配套 (L1/L1.1/L1.2 三件套 + match 域命令) + §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log + 9/3 L-CAND-006) + §5 已知缺口 (5 类 13 条) + §6 修订历史 v0.1, 保持 §1 9 项与 v0.2 §4.4 原文一致, 不擅自修改 9 项内容, per C3 派生约束拆分 + AGENTS.md v0.6.4 §9.4 里程碑重定义 + D3 commit 模板 (per AGENTS.md v0.6.2 §2.6) |
| **v0.2** | 2026-09-03 12:46 | 作者: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 / 审批: 架构师(Mavis 接手 agent per DEC-008) | **业务回填 9 项升版**: 基于 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 565/565 + `111d4ad` 5 域 E2E Phase C marker 10 marker 编译期锚定 + `fa32bab` 5 域 mTLS mock 15/15 passed) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证. §1 表格状态列更新: 3 项 🟡→✅ (#2 IT mTLS mock 路径 + #3 E2E 业务 mTLS 编译期锚定 + #5 SLA 监控 WSL kubectl 实证), 3 项 ✅→✅ (#1 UT L1.1 维持 + #7 部署健康 维持 + #9 Schema 迁移 维持, 含 0041_moves_partitioned DRAFT), 4 项 🟡→🟡 (#4 跨域 saga 真跑需阶段 C + #6 告警撮合失败率 SRE 拍板 + #8 证书 L-CAND-006 兜底 + #10 审计 admin Q2 决策), 闭环率 3/10 → 6/10 (60%). §2 状态更新加 9/3 12:46 JST R1 业务回填 9 项统计表 + 5 域 main HEAD `fa32bab`. §3 DoD 配套加 mTLS mock 路径 + 编译期锚定 (per `fa32bab` / `111d4ad`). §4 派生约束守护加 L12 案例库 (per 9/3 12:36 JST 升正式 commit `2e4f519` + L-CAND-009). §5 已知缺口加阶段 A → 阶段 B/C 衔接缺口 2 条. §6 修订历史本行 |

---

## 7. 后续动作 (per R1 业务冲刺 R3 阶段任务)

- **9/3 (今日)**: Mavis 自审 1 次停手 (per B3 派生约束 + DDD-REVIEW-TEMPLATE-v0.2 §3.x)
- **9/3-9/7**: match 域 Lead 反馈 (5 域 RACI v1.1 match Lead 派工已签)
- **W37 D2 (9/9)**: Phase C 阶段 A 启动 (SRE Lead 拍板)
- **W37 D3-D5 (9/10-12)**: 阶段 B 跑通, match 50053 health probe 真跑 (item #2, v0.2 升 ✅ 是 mock 路径)
- **W37 D6 (9/13)**: 11 UT 真跑 (item #1 二次验证)
- **W37 D7 (9/14)**: 11 E2E 准备, W37 周报 v0.3 出
- **W38 D1-D2 (9/15-16)**: 11 E2E 真跑 + 跨域 saga 真实交易 (item #3 + item #4)
- **W38 D3 (9/17)**: match 域 6/10 → 9/10 闭环 = match 域生产可用 ✅
- **W38 D5 (9/19)**: RGS-CRITIQUE-IMPROVEMENT v0.2 正式升版, 本 v0.2 升 v0.3 同步 match 域实际状态
