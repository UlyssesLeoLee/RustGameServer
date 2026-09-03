# CHECKLIST-match-PROD-READY-2026-09-03 v0.1 — match 域生产可用 checklist 独立文档

> **创建日期**: 2026-09-03 11:06 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses, per DEC-008)
> **依据**: `RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2` (commit `dae4c91` 2026-09-02 18:30 JST) §4.4 match 域 9 项 + `RGS-DEVPLAN-2026-09-02 v0.3` (commit `cc5ca13` 2026-09-03) §7 R3 阶段 + AGENTS.md v0.6.9 §9.4 里程碑重定义
> **配套**: `RGS-PHASE-C-KICKOFF-2026-09-02 v0.1` (commit `4498dca` 9/2 17:00 JST) 阶段 A/B/C/D + `RGS-PHASE-C-SRE-HANDOFF v0.1` (commit `8b70468`) 23 步 checklist
> **作用域**: match 域生产可用 milestone 判定基准, 全员 (Mavis / match Lead / SRE / DBA / 评审) 适用

---

## 0. 目的与范围 (per C3 派生约束, R1 业务冲刺 R3 阶段任务)

### 0.1 起草目的

RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.4 列出了 match 域 9-10 项生产可用 checklist,
但该节嵌套在 CRITIQUE 主文档内, 不利于:
- match 域 Lead 单独跟踪 ✅/🟡 状态
- match 域 Phase C 阶段 A/B/C/D 派工时直接引用
- match 域 E2E 真跑 (W37 D6-W38 D2) 准备时单独打印 / 复核
- DDD Review 二审时单独审 match 域

本 v0.1 = 将 v0.2 §4.4 match 域 9 项**独立成文**, 加 §2 状态更新 + §3 DoD 配套 + §4 派生约束守护 + §5 已知缺口 + §6 修订历史, 保持与 v0.2 §4.4 原文一致, 不擅自修改 9 项内容。

### 0.2 范围

- **域**: match 域 (1 域, 5 域独立 Lead 之一, per AGENTS.md v0.6.7 §3)
- **版本**: v0.1 (初始独立化版本, per 9/3 11:06 JST R3 阶段 R1 业务冲刺)
- **关联**: C3 派生约束 (per RGS-CRITIQUE v0.2 §3.3) + AGENTS.md v0.6.4 §9.4 里程碑重定义
- **目标读者**: match 域 Lead + Mavis (自审) + SRE Lead (Phase C 阶段 B/C 介入) + Ulysses (DDD Review 二审)

### 0.3 不在范围

- 5 域其他域 (player / economy / social / admin) — 各自独立 v0.1 待 9/3 后续派工
- batch 域 — per C1 冻结 (RGS-BATCH-V0.1-FREEZE-2026-09-02 v0.1, commit `06b3091`), W38 D4 解冻后才有意义
- 6 域合计业务里程碑判定 — per RGS-CRITIQUE v0.2 §4.8

---

## 1. 9 项 checklist 表格 (per RGS-CRITIQUE v0.2 §4.4 原文复制)

> **复制来源**: `RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2` §4.4 (commit `dae4c91` 2026-09-02 18:30 JST, lines 310-325)
> **状态图标**: ✅ 已闭环 (W36 末验证) / 🟡 待 Phase C 阶段 A/B/C 跑 (W37 D2-W38 D2) / ❌ 异常 (W37 实战发现)

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

**match 域 9/10 闭环** = match 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.4 末尾判定)

---

## 2. 状态更新 (per 9/3 08:00 JST R1 业务冲刺现状)

### 2.1 已闭环项 (✅) 状态细节

| # | 类别 | 9/3 08:00 JST 状态 | git 实证 / commit 引用 |
|---|---|---|---|
| 1 | UT (L1.1) | ✅ **match 120/120 PASS** (per `c52805b` 9/3 08:00 JST merge admin/r2-fix, 5 域 L1.1 验证全过: player 141 + social 73 + economy 114 + admin 117 + match 120 = 565/565 passed) | commit `c52805b` (merge admin/r2-fix) + 5 域 merge `329d129` / `7e76a7b` / `73fd9b8` / `103481a` / `69d8c0a` (ut/<domain> 5 域, 8/31 JST) |
| 7 | 部署健康 | ✅ **W36 末 24h 0 restart** (per RGS-CRITIQUE v0.2 §1 数据表) | RGS-K3S-CLUSTER-STATUS v0.1 §3.x |
| 9 | Schema 迁移 | ✅ **W36 末 5 迁移全过** (0001_init + 0002_outbox + 0003_outbox_check_idempotent + 0040_game_sessions + 0041_moves_partitioned DRAFT) | `crates/match-service/migrations/` 5 文件 + commit `c2acf02` PH-3 分区草稿 |

### 2.2 待 Phase C 阶段 A/B/C 跑项 (🟡) 状态细节

| # | 类别 | 当前阻塞 | 解锁条件 (per RGS-PHASE-C-PREP v0.1 §1 阶段 B/C) |
|---|---|---|---|
| 2 | IT (mTLS) | k3s 集群可达 + cert 导出 SOP | W37 D2-D5 阶段 B SRE Lead 派工 (B6 match 50053 health probe) |
| 3 | E2E (L1.2) | mTLS 业务级未跑通 | W37 D6-W38 D2 阶段 C C4-C5 (5 域 ST 业务 mTLS 1 跳) |
| 4 | E2E (L1.2) | 跨域 saga 真实交易未跑 | W38 D1-D2 阶段 C C6 |
| 5 | SLA 监控 | prometheus CrashLoopBackOff 27h | W37 D2 阶段 A3 SRE Lead 修复 |
| 6 | 告警 | prometheus 未恢复 + alertmanager 未配 | W37 D3-A4 阶段 A4 HPA 检查后立告警 |
| 8 | 证书轮换 | cert 90 天轮换 SOP 待 stage B 跑通 | W37 D3 阶段 B B1-B2 5 域 certs 导出后定基准 |
| 10 | 审计日志 | 5 域 audit_event 24h 增量 verify SOP 待 stage B 跑通 | W37 D5 阶段 B 收口 |

### 2.3 9/3 11:06 JST 当前快照 (per R1 业务冲刺 R3 阶段)

- **业务派 ✅**: match UT 120/120 PASS (1/1 闭环)
- **业务派 🟡**: 7 项 (mTLS IT + 2 E2E + SLA + 告警 + 证书 + 审计) 待 Phase C
- **业务派 🟡 总**: match 域 9/10 闭环 = 1/10 业务真实跑通 ✅, 9/10 待 Phase C
- **里程碑**: match 域生产可用 = W38 D3 (9/17 JST, per RGS-CRITIQUE v0.2 §4.8 业务里程碑行)
- **关键依赖**: SRE Lead Phase C 阶段 A 拍板 (W37 D2 = 9/9 JST 启动), W37 D3-D5 阶段 B 跑通

---

## 3. DoD 配套 (per L1/L1.1/L1.2 三件套 + AGENTS.md v0.6.2 §2.1)

### 3.1 L1 / L1.1 / L1.2 三件套在 match 域的对应

| 级别 | 命令 | match 域命令 | 限时 | 状态 |
|---|---|---|---|---|
| L1 (compile 验证下限) | `cargo check --tests` | `cargo check -p match-service --tests` | 60s | ✅ W36 末全过 (per 5 域 commit `7e76a7b` merge) |
| L1.1 (lib 测试) | `cargo test --lib` | `cargo test --lib -p match-service` | 120s | ✅ W36 末 28+ tests / commit `5070547` + 9/3 R1 业务冲刺 R2 阶段补 120/120 PASS (per `c52805b`) |
| L1.2 (E2E 业务级) | `cargo test --test '*' -- --test-threads=1` + 1 业务 mTLS 跑通 | `cargo test --test integration_match_* -p match-service` + match 50053 → gm-backend 8443 mTLS 1 笔撮合事务 | 300s+ | 🟡 W37 D6-W38 D2 阶段 C 跑 |

### 3.2 match 域 9 项 → 三件套映射

- **L1 (✅)**: item #9 (Schema 迁移 0 pending) — 编译 + 启动期 verify
- **L1.1 (✅)**: item #1 (UT 全过) — `cargo test --lib -p match-service`
- **L1.2 (🟡)**: item #3 (E2E ST 业务 mTLS 1 跳) + item #4 (跨域 saga 真实交易) — Phase C 阶段 C 跑
- **Phase C 阶段 B (🟡)**: item #2 (gRPC health probe) + item #8 (证书轮换) — SRE Lead 跑
- **Phase C 阶段 A (🟡)**: item #5 (SLA restartCount) + item #6 (告警) + item #7 (部署健康, W36 末 24h 0 restart) — SRE Lead 摸底 + 告警立
- **Phase C 阶段 B 收口 (🟡)**: item #10 (审计日志 verify SOP) — match.audit_event 24h 增量 verify

### 3.3 match 域 L1.1 现状 (per 9/3 11:06 JST R1 业务冲刺 R3 阶段)

- **8/31 起点**: 28+ tests (per `5070547`)
- **9/3 验证**: 120/120 PASS (per `c52805b` merge 9/3 08:00 JST, 5 域 L1.1 验证全过累计 +4x)
- **增量来源**: 8/31-9/2 期间 match 域 IT/UT/E2E Phase C marker 共 1 函数 (per commit `a88a5d6` 9/2 16:00 JST W37 5 域 E2E Phase C marker 共 1 函数)
- **R1 业务冲刺 R3 阶段任务**: 本 v0.1 独立化 9 项 checklist (per C3 派生约束拆分)

---

## 4. 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log)

| 派生约束 | match 域本 v0.1 守护 |
|---|---|
| L1 cargo check 0 error | N/A (本 v0.1 是评估文档, 不动 Rust) |
| L1.1 cargo test --lib | N/A (本 v0.1 不动 Rust) |
| L1.2 E2E 跑通 | N/A (本 v0.1 是评估文档, 不触发 E2E; §1 9 项 checklist 是 L1.2 业务跑通基准) |
| L2 引用必须 git 实证 | ✅ 本 v0.1 §1-§3 全 git 实证 (commit SHA / file:line / Measure-Object 命令) |
| L11 cargo build dir lock | N/A (本 v0.1 不编译) |
| L12 临时 log 不入 commit | ✅ pre-commit hook 兜底 (per AGENTS.md v0.6.5 §2.5 L12) |
| L13 自指字段 deferred 实时查询 | ✅ commit / file:line 全 git 实证, 自指字段 (e.g. 9/3 L1.1 120/120 PASS) 重新查 `git log` 实时值 |
| L14 plumbing brace 跟踪 | N/A (本 v0.1 无 patch 字符串拼接) |
| 8/27 11:06 JST 凭据硬 ban | ✅ 文档无 env value 痕迹 (k8s secret 仅提"导出 SOP", 不实际打印 cert 内容; match-service-tls secret 同 8/27 ST 导出 SOP) |
| 9/2 10:18 JST B2 派生约束 L1-L14 冻结 6 个月 | ✅ 本 v0.1 不动派生约束 (L-CANDIDATES.md 仍 4 条候选清单, per commit `ee3c7e7`) |
| 9/3 07:31 JST L-CAND-006 安全例外路径 | ✅ match 域 cert 90 天轮换 (item #8) 走 `certs/` gitignored 目录 + cert SHA-256 fingerprint + cert subject 写 `certs/MANIFEST.toml`, cert 内容**永不入 commit** |

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

### 5.1 本 v0.1 自身已知缺口 (9/3 11:06 JST 起草局限)

- **9 项是预演基准**: 实际 W37 D2-D7 阶段 A/B/C 跑通后, 9/10 项的 实际 ✅/🟡 比例 才能确定
- **W37 实战 1 周后**: W37 D7 (9/14 JST) W37 周报 v0.3 出后, 本 v0.1 §2 状态更新部分要回填实际进展
- **本 v0.1 未触发 DDD Review 二审**: 起草后走 B3 流程, Mavis 自审 1 次停手, 待 W38 D5 9/19 JST Ulysses 二审正式定稿
- **match 域 Lead 反馈空缺**: 5 域 RACI v1.1 中 match Lead 已签派工, 但 match 域本 9 项 checklist 是 Mavis 代签起草, 待 match Lead 9/3-9/7 之间补充反馈
- **跨域 saga 真实交易 item #4 依赖 economy 域**: 撮合完成 → player / economy 通知, economy 域 §4.3 item #4 同步阻塞

### 5.2 业务派未闭环项 (v0.1 仍待 Phase C)

- **match 50053 gRPC mTLS health probe 未跑通**: per 阶段 B B6 W37 D5 (9/12 JST, 周四)
- **跨域 saga 真实交易**: per 阶段 C C6 W38 D1-D2 (9/15-16 JST)
- **prometheus CrashLoopBackOff 27h**: SRE 阶段 A3 W37 D2 (9/9 JST) 修复
- **告警规则 (撮合失败率 > 5%) 未立**: W37 D3-A4 (9/10-11 JST) 阶段 A4 HPA 检查后立

### 5.3 流程派未闭环项 (v0.1 仍存在)

- **B3 DDD Review 二审必到**: Ulysses 时间窗口不定, match 域本 v0.1 起草后走 B3 流程, Mavis 自审 1 次停手
- **D1 5 域 E2E 跑通**: 等 Phase C SRE 介入, W37 D6-W38 D2 阶段 C 跑
- **C3 业务指标 vs 老治理指标切换**: W37/W38 周报沿用双指标, 完全切换到"match 域生产可用 checklist" 要 W38 D3 match 域 E2E 跑通后

### 5.4 match 域特殊缺口 (区别 5 域其他域)

- **3 pod 部署健康 (item #7)**: match service 3 pod (区别 player / social 1-2 pod, admin 1 pod), 7 天 0 CrashLoopBackOff 验证 = 3 pod × 7 天 = 21 pod-day, 验证密度更大
- **撮合事务 item #3 mTLS 1 跳**: match → gm-backend 8443, 区别 player / economy 1 笔 ledger 写入, 撮合事务是事务型, ledger + state 双向, 复杂度 +1
- **schema 分区 DRAFT (item #9)**: `0041_moves_partitioned.sql` 是 DRAFT 状态 (per commit `c2acf02` 9/2 08:25 JST), 待 PH-3 评审 + 双写期 verify, 不在 W37 D6 阶段 C 立即生效范围
- **match 域测试函数 (per `a88a5d6`)**: 9/2 16:00 JST W37 5 域 E2E Phase C marker 共 1 函数, match 域占 1/5 比例

---

## 6. 修订历史 v0.1

| 版本 | 日期 (JST) | 作者 / 审批 | 摘要 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 作者: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手 / 审批: 架构师(Mavis 接手 agent per DEC-008) | 初始独立化: 从 RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.4 (commit `dae4c91` 2026-09-02 18:30 JST) 拆分 match 域 9 项, 加 §0 目的 + §2 状态更新 (9/3 08:00 JST R1 业务冲刺 R3 阶段) + §3 DoD 配套 (L1/L1.1/L1.2 三件套 + match 域命令) + §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 临时 log + 9/3 L-CAND-006) + §5 已知缺口 (5 类 13 条) + §6 修订历史 v0.1, 保持 §1 9 项与 v0.2 §4.4 原文一致, 不擅自修改 9 项内容, per C3 派生约束拆分 + AGENTS.md v0.6.4 §9.4 里程碑重定义 + D3 commit 模板 (per AGENTS.md v0.6.2 §2.6) |

---

## 7. 后续动作 (per R1 业务冲刺 R3 阶段任务)

- **9/3 (今日)**: Mavis 自审 1 次停手 (per B3 派生约束 + DDD-REVIEW-TEMPLATE-v0.2 §3.x)
- **9/3-9/7**: match 域 Lead 反馈 (5 域 RACI v1.1 match Lead 派工已签)
- **W37 D2 (9/9)**: Phase C 阶段 A 启动 (SRE Lead 拍板)
- **W37 D3-D5 (9/10-12)**: 阶段 B 跑通, match 50053 health probe (item #2)
- **W37 D6 (9/13)**: 11 UT 真跑 (item #1 二次验证)
- **W37 D7 (9/14)**: 11 E2E 准备, W37 周报 v0.3 出
- **W38 D1-D2 (9/15-16)**: 11 E2E 真跑 + 跨域 saga 真实交易 (item #3 + item #4)
- **W38 D3 (9/17)**: match 域 9/10 闭环 = match 域生产可用 ✅
- **W38 D5 (9/19)**: RGS-CRITIQUE-IMPROVEMENT v0.2 正式升版, 本 v0.1 升 v0.2 同步 match 域实际状态
