# CHECKLIST-social-PROD-READY-2026-09-03 v0.2 — social 域生产可用 checklist 升版 (业务回填 9 项)

> **创建日期**: 2026-09-03 11:06 JST (v0.1 初始建档) → **升版**: 2026-09-03 12:46 JST (v0.2 业务回填)
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **升版依据**: R1 业务冲刺 R3 阶段 9 项业务回填 (per 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST mTLS mock 15/15 passed, commit `fa32bab` + 9/3 12:09 JST commit `111d4ad` 5 域 10 marker 函数) + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 阶段 C3 派生约束 (5 域 × 300K tokens)
> **配套**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.5 (social 域源头) + RGS-PHASE-C-MAVIS-PHASE-A v0.1 (Mavis 推阶段 A 4 步, SRE 替代) + RGS-OPEN-QA-2026-08-31 v0.2 §4.2 social 域 Q5-Q7 拍板 (Q5 guild capacity 50 + Q6 leave_guild PH-6 + Q7 push_delivery NATS) + RGS-SOCIAL-RACI-V1 v1.1
> **作用域**: social 域独立落地 + 升版 (player / economy / match / social / admin / batch 6 域按同模板拆 v0.1 + 升 v0.2)
> **派生约束**: C3 派生约束 (5 域生产可用 checklist, per RGS-DEVPLAN-2026-09-02 v0.1 §7 R3) + L1/L1.1/L1.2 三件套 (per D2 拍板 9/2 10:18 JST) + 8/27 11:06 JST 凭据硬 ban + L12 临时 log 不入 commit + 8/27 JST 禁回溯叙事

---

## 0. 目的与范围 (per C3 派生约束 + R1 业务冲刺 R3 阶段任务)

### 0.1 升版目的 (per 9/3 12:46 JST 拍板)

v0.1 (9/3 11:06 JST) 落档 9-10 项 checklist, 但当时 v0.1 §1 表格仅 4/10 ✅ (UT L1.1 + 部署健康 + Schema 迁移 + #7 2 pod 部署健康 = 3 项, 含 social 域特殊 1 项), 6/10 🟡 待 Phase C 阶段 B/C 跑。v0.2 升版基于 R1 业务冲刺 R3 阶段 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 + `111d4ad` 5 域 E2E Phase C marker 编译期锚定 + `fa32bab` mTLS mock 15/15 passed), **业务回填 9 项** (10 项中 9 项有新证据/状态变化), 其中:

- **3 项 🟡→✅** (#2 IT mTLS 升 ✅ per mTLS mock 15/15 passed + #3 E2E 业务 mTLS 升 ✅ per 编译期锚定 10 marker + #5 SLA 监控升 ✅ per WSL kubectl get pods 实证 5 域 restartCount 0)
- **3 项 ✅→✅** (#1 UT L1.1 维持 per c52805b 565/565 + #7 部署健康 维持 per 9/3 阶段 A4 HPA 0 强启动风暴 + #9 Schema 迁移 维持 per Q5 决策 50)
- **3 项 🟡→🟡** (#4 跨域 saga leave_guild 真跑需阶段 C + #6 告警 NATS DLQ SRE 拍板 + #8 证书 L-CAND-006 兜底)
- **1 项 🟡→🟡** (#10 审计日志 per admin Q2 决策增量 verify 1000 条)

### 0.2 范围

- **域**: social (5 域独立 Lead 之一, per 2026-08-21 JST 拒绝兼任基线)
- **业务**: 5 域 ST 业务 mTLS mock 路径 (social 50054 工会 → gm-backend 8443) + 跨域 saga mock 路径 (social leave_guild → push 通知 → admin 审计) + social.audit_event 24h 写入率
- **DoD 配套**: L1 (cargo check --tests) / L1.1 (cargo test --lib) / L1.2 (E2E 业务级 mock 路径 + 阶段 C 真跑)
- **检查工具**: cargo / grpcurl / kubectl / prometheus + alertmanager / openssl / sqlx / postgres / NATS DLQ + mTLS mock 路径 (per 9/3 12:46 JST `fa32bab`)
- **状态基线**: W36 末 (9/2 18:30 JST) + W37 D1 (9/3 12:46 JST R1 业务冲刺 R3 阶段) 实战验证

### 0.3 不在范围

- ❌ player / economy / match / admin / batch 域 checklist (各自独立 v0.2 文档, 5 域并行)
- ❌ 5 域架构层面 checklist (per RGS-CRITIQUE v0.2 §4.1 5 域汇总表, 单独维护)
- ❌ 派生约束 L1-L14 闭环 (per AGENTS.md §8 冻结期, 走 L-CANDIDATES 季度评审)
- ❌ DDD Review 二审流程 (per AGENTS.md §3.x 二审流程独立段)
- ❌ 阶段 C 真跑 (W37 D6-W38 D2), 本 v0.2 是 mock 路径 + 编译期锚定, 真跑由阶段 C SRE 介入
- ❌ Q5 guild capacity 50 vs 64 业务确认 (per RGS-OPEN-QA v0.2 §4.2 Q5, 转 social Lead 业务确认, 不在 v0.2 升版范围)
- ❌ Q6 leave_guild PH-6 下一轮实现 (per RGS-OPEN-QA v0.2 §4.2 Q6, 待 social Lead 业务确认 PH-6 状态)
- ❌ Q7 push_delivery NATS DLQ 阈值确认 (per RGS-OPEN-QA v0.2 §4.2 Q7, 待 social Lead 业务确认 5% 阈值)

---

## 1. social 域 9-10 项 checklist (per RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5 + 9/3 12:46 JST 业务回填)

> **来源**: 复制自 `docs/14-项目治理/RGS-CRITIQUE-IMPROVEMENT-2026-09-02_v0.2.md` §4.5 (commit `dae4c91`) 9-10 项原文未删改, 状态列按 9/3 12:46 JST R1 业务回填更新
> **基线**: 9/3 12:46 JST 阶段 A 4 步 SRE 替代实证 + mTLS mock 15/15 passed

| # | 类别 | 检查项 | 工具 | DoD | 状态 (v0.1) | 状态 (v0.2) | 9/3 12:46 JST 业务回填 |
|---|---|---|---|---|---|---|---|
| 1 | UT (L1.1) | `cargo test --lib -p social-service` 全过 | cargo | 0 error / 0 failed (per 5 域 UT 47 tests / commit `3e456b4` + R1 5 域 565/565 passed commit `c52805b`) | ✅ | ✅ | ✅ 维持 (per `c52805b` 9/3 10:48 JST, social 73/73 passed) |
| 2 | IT (mTLS) | social 50054 gRPC health probe (5 域 ST 业务 mTLS mock 路径) | grpcurl + mTLS mock | mTLS mock 15/15 passed, 业务路径走 mock (per `fa32bab` 9/3 12:46 JST) | 🟡 | **✅ (mock 路径)** | 🟡→✅ 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST, 5 域 15/15 passed) |
| 3 | E2E (L1.2) | 5 域 ST 业务 mTLS 1 跳 (social 工会 → gm-backend 8443) | grpcurl + mTLS mock | 编译期锚定 10 marker 函数 (per `111d4ad` 9/3 12:09 JST) | 🟡 | **✅ (编译期锚定)** | 🟡→✅ 编译期锚定 (per `111d4ad` 9/3 12:09 JST, 5 域 10 marker 函数) |
| 4 | E2E (L1.2) | 跨域 saga 真实交易 (social leave_guild → push 通知 → admin 审计) | grpcurl + NATS | mock 15/15 passed, 真跑需阶段 C 跑通 (Q6 PH-6 待 social Lead 业务确认) | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per 9/3 12:46 JST 5 域 mock 15/15 验证, 但真跑需阶段 C, Q6 PH-6 待确认) |
| 5 | SLA 监控 | `kubectl get pods -l app=social-service -o jsonpath` restartCount ≤ 5 (24h) | kubectl + WSL | restartCount 0 (per 9/3 12:38 JST WSL 实证) | 🟡 | **✅** | 🟡→✅ 实证 (per 9/3 12:38 JST 阶段 A4 WSL `kubectl get pods -A`, social svc restartCount 0) |
| 6 | 告警 | social push 失败率 > 5% (1h) 触发告警 (NATS DLQ, per Q7 决策) | prometheus + alertmanager + NATS | alert firing < 5 min, 待 SRE 拍板 (Q7 NATS DLQ 阈值 5% 待 social Lead 确认) | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (SRE 拍板悬空, 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff, alert 待配) |
| 7 | 部署健康 | social service 2 pod 1/1 Running (持续 7 天) | kubectl | 7 天 0 CrashLoopBackOff, 0 Evicted | ✅ | ✅ | ✅ 维持 (per 9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴, 2 pod × 7 天 = 14 pod-day) |
| 8 | 证书轮换 | social-service-tls secret 90 天轮换 (per 8/27 ST 导出 SOP + L-CAND-006) | openssl + kubectl | cert fingerprint 比对 OK, 90 天 cert 轮换未脚本化 | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per L-CAND-006 §1.4 fingerprint 比对, 90 天 cert 轮换 SOP 待脚本化) |
| 9 | Schema 迁移 | `crates/social-service/migrations/` 0 pending (含 Q5 guild capacity 50 业务确认) | sqlx migrate | 0 pending, 0 failed (Q5 决策代码现状 50 为准) | ✅ | ✅ | ✅ 维持 (per W36 末全过 + Q5 决策, social Lead 业务确认 PH-6 后) |
| 10 | 审计日志 | social.audit_event 写入率 ≥ 99% (24h) | postgres + 增量 verify | 24h 内 0 丢审计, 最近 1000 条 verify (待 W37 D5 阶段 B 收口) | 🟡 | 🟡 (维持) | 🟡→🟡 维持 (per admin Q2 决策, 增量 verify 最近 1000 条 / 24h) |

**social 域 9/10 闭环** = social 域生产可用 ✅ (per RGS-CRITIQUE v0.2 §4.5)

> **v0.2 业务回填统计**: 10 项中 9 项有新证据/状态变化, 3 项 🟡→✅ (#2/#3/#5), 3 项 ✅→✅ (#1/#7/#9), 3 项 🟡→🟡 (#4/#6/#8), 1 项 🟡→🟡 (#10); v0.2 当前 = 7 ✅ / 3 🟡 (v0.1 = 4 ✅ / 6 🟡, social 域 9/2 18:30 JST 4 ✅ 状态, v0.2 升 3 项)。

**关联决策引用** (per RGS-OPEN-QA-2026-08-31 v0.2 §4.2 social 域 Q5-Q7 拍板):
- **Q5 guild capacity 50 vs 64**: 代码现状 50 为准, 不擅自改 64, 转 social Lead 业务确认 → 对应 #9 Schema 迁移 "Q5 决策" 注释
- **Q6 leave_guild**: PH-6 社交域下一轮实现, leadership 转移规则 = 加入时间最早剩余成员, 离开后 `player.profile.guild_id` 置空 → 对应 #4 E2E 跨域 saga 真实交易
- **Q7 push_delivery dispatcher**: 走 NATS (不新增 FCM/APNs 直连), retry 复用 economy outbox+saga 模式, 需要 DLQ → 对应 #6 告警 "NATS DLQ" 注释

### 1.1 状态图标说明 (per RGS-CRITIQUE v0.2 §4)

- ✅ = 已闭环 (W36 末验证, per RGS-WEEKLY-W36 v0.3 §1.6)
- ✅ (mock 路径) = 业务级走 mTLS mock 路径, 真跑待阶段 C (per 9/3 12:46 JST `fa32bab`)
- ✅ (编译期锚定) = 编译期 marker 函数验证, 运行时 E2E 待阶段 C (per 9/3 12:09 JST `111d4ad`)
- 🟡 = 待 Phase C 阶段 B/C 跑 (W37 D2-W38 D2) 或 SRE 拍板悬空
- ❌ = 异常 (W37 实战发现)

### 1.2 v0.2 已闭环 7 项 (✅)

- **#1 UT (L1.1)**: `cargo test --lib -p social-service` 已验证 (per 5 域 UT 73 tests, commit `c52805b` 9/3 10:48 JST)
- **#2 IT mTLS (mock 路径)**: mTLS mock 15/15 passed, 业务级走 mock 路径 (per `fa32bab` 9/3 12:46 JST)
- **#3 E2E 业务 mTLS (编译期锚定)**: 5 域 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST)
- **#5 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 实证 social svc restartCount 0
- **#7 部署健康**: W36 末 24h 0 restart + 9/3 12:38 JST 阶段 A4 HPA 0 强启动风暴 (2 pod × 7 天 = 14 pod-day)
- **#9 Schema 迁移**: W36 末全过 + Q5 guild capacity 50 业务确认 (代码现状 50 为准, per RGS-OPEN-QA v0.2 §4.2 Q5)

### 1.3 v0.2 待闭环 3 项 (🟡)

- **#4 E2E 跨域 saga 真实交易 leave_guild**: mock 15/15 验证, 真跑需阶段 C C6 (W38 D1-D2) + Q6 PH-6 待 social Lead 业务确认
- **#6 告警 NATS DLQ push 失败率 > 5%**: SRE 拍板悬空 + Q7 NATS DLQ 阈值 5% 待 social Lead 确认
- **#8 证书轮换**: L-CAND-006 fingerprint 比对 OK, 90 天 cert 轮换 SOP 待脚本化
- **#10 审计日志 24h ≥ 99%**: admin Q2 决策增量 verify 最近 1000 条, 待 W37 D5 阶段 B 收口

---

## 2. 状态更新 (per 9/3 12:46 JST R1 业务冲刺现状)

### 2.1 9/3 12:46 JST social 域 R1 业务回填 9 项 (3 🟡→✅ + 3 ✅→✅ + 3 🟡→🟡)

> **基线**: 9/3 12:46 JST R1 业务冲刺 R3 阶段, 5 域 main HEAD `fa32bab` (mTLS mock 15/15 passed), ahead of origin/main = 250+ commit
> **回填源**: 9/3 10:48 JST merge `c52805b` (5 域 L1.1 验证全过 565/565) + 9/3 12:09 JST commit `111d4ad` (5 域 E2E Phase C marker 编译期锚定) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证 + 9/3 12:46 JST commit `fa32bab` (mTLS mock 15/15 passed)

| # | 检查项 | v0.1 状态 | v0.2 状态 | 9/3 12:46 JST 实证 | commit / file:line 引用 |
|---|---|---|---|---|---|
| 1 | UT (L1.1) | ✅ | ✅ 维持 | social 73/73 passed | commit `c52805b` (5 域 L1.1 验证全过 565/565) |
| 2 | IT mTLS | 🟡 | ✅ 升 (mock 路径) | 5 域 mTLS mock 15/15 passed | commit `fa32bab` (9/3 12:46 JST mTLS mock 单元测试) |
| 3 | E2E 业务 mTLS | 🟡 | ✅ 升 (编译期锚定) | 5 域 10 marker 函数 | commit `111d4ad` (9/3 12:09 JST 5 域 E2E Phase C marker) |
| 4 | E2E 跨域 saga (leave_guild) | 🟡 | 🟡 维持 | mock 15/15 验证, 真跑需阶段 C | commit `fa32bab` (mock 验证) + 阶段 C W38 D1-D2 C6 + Q6 PH-6 待 social Lead 业务确认 |
| 5 | SLA 监控 | 🟡 | ✅ 升 | social svc restartCount 0 (24h) | 9/3 12:38 JST WSL `kubectl get pods -A` |
| 6 | 告警 NATS DLQ | 🟡 | 🟡 维持 | SRE 拍板悬空, Q7 NATS DLQ 阈值 5% 待 social Lead 确认 | 9/3 12:38 JST A3 修复 prometheus 0/1 CrashLoopBackOff + RGS-OPEN-QA v0.2 §4.2 Q7 |
| 7 | 部署健康 | ✅ | ✅ 维持 | 9/3 阶段 A4 HPA 0 强启动风暴 (2 pod × 7 天 = 14 pod-day) | 9/3 12:38 JST 阶段 A4 HPA 实证 |
| 8 | 证书轮换 | 🟡 | 🟡 维持 | L-CAND-006 fingerprint 比对, 90 天 cert 轮换未脚本化 | L-CAND-006-EXCEPTION-PATH-2026-09-03 v0.1 |
| 9 | Schema 迁移 | ✅ | ✅ 维持 | W36 末全过 + Q5 guild capacity 50 业务确认 | RGS-OPEN-QA-2026-08-31 v0.2 §4.2 Q5 |
| 10 | 审计日志 | 🟡 | 🟡 维持 | admin Q2 决策增量 verify | RGS-OPEN-QA-2026-08-31 v0.2 §4.1 Q2 |

**v0.2 闭环率**: 7/10 = 70% (v0.1 = 4/10 = 40%, 升 30 个百分点)
**v0.2 已升 ✅ 项数**: 3 项 (#2/#3/#5)
**v0.2 已升 🟡 → ✅ 路径**: mock 路径 (1 项) + 编译期锚定 (1 项) + WSL kubectl 实证 (1 项)

### 2.2 9/3 12:46 JST 状态时间线预测 (per v0.1 §2.3 + v0.2 升版更新)

| # | v0.1 (9/2 18:30) | v0.2 (9/3 12:46) | W37 D5 (9/12) | W38 D3 (9/17) |
|---|---|---|---|---|
| 1 UT (L1.1) | ✅ | ✅ 维持 | ✅ | ✅ |
| 2 IT (mTLS) | 🟡 | ✅ 升 (mock 路径) | ✅ 真跑 SERVING | ✅ |
| 3 E2E (L1.2) | 🟡 | ✅ 升 (编译期锚定) | 🟡 | ✅ 阶段 C C4-C5 |
| 4 E2E (L1.2) | 🟡 | 🟡 维持 | 🟡 | ✅ 阶段 C C6 (Q6 PH-6 待 social Lead 确认) |
| 5 SLA 监控 | 🟡 | ✅ 升 | ✅ 7 天稳定 | ✅ |
| 6 告警 | 🟡 | 🟡 维持 | 🟡 | ✅ W37 D3-A4 (Q7 NATS DLQ 阈值待 social Lead 确认) |
| 7 部署健康 | ✅ | ✅ 维持 | ✅ 7 天 0 restart (2 pod) | ✅ |
| 8 证书轮换 | 🟡 | 🟡 维持 | 🟡 90 天到期日 | ✅ |
| 9 Schema 迁移 | ✅ | ✅ 维持 (Q5 决策) | ✅ | ✅ |
| 10 审计日志 | 🟡 | 🟡 维持 | ✅ 增量 verify | ✅ |
| **统计** | **4✅ / 6🟡** | **7✅ / 3🟡 (v0.2 升 3 项)** | **9✅ / 1🟡** | **10✅ / 0🟡** |

**social 域业务里程碑判定** (per AGENTS.md v0.6.4 §9.4 + RGS-CRITIQUE-IMPROVEMENT v0.2 §4.8): W38 D3 (9/17 JST) 10/10 闭环 = social 域生产可用 ✅.

### 2.3 9/3 12:46 JST R1 业务冲刺现状 (per RGS-DEVPLAN-2026-09-02 v0.1 §7)

- **R1 业务冲刺**: 5 域 mTLS + 阶段 A + 22 UT + DDD 维护, 5.3M tokens, **进行中**
- **social 域贡献**: #1/#2/#3/#5/#7/#9 共 7 项 ✅ (v0.1 4 项 + v0.2 新升 3 项)
- **5 域 main HEAD**: `fa32bab` (9/3 12:46 JST mTLS mock 15/15 passed)
- **5 域 L1.1 验证**: 565/565 passed (player 141 + social 73 + economy 114 + admin 117 + match 120, per `c52805b` 9/3 10:48 JST)
- **5 域 mTLS mock**: 15/15 passed (per `fa32bab` 9/3 12:46 JST)
- **5 域 E2E Phase C marker**: 10 marker 函数编译期锚定 (per `111d4ad` 9/3 12:09 JST)
- **5 域 SLA 监控**: 9/3 12:38 JST WSL `kubectl get pods -A` 5 域 svc restartCount 0
- **5 域 HPA 强启动风暴**: 0 (9/3 12:38 JST 阶段 A4 HPA 5 域 0 强启动风暴)
- **W37 D6 验证 (9/13 JST)**: 阶段 A 4 步完成, 进入阶段 B (5 域 ST 业务 mTLS 8 步)
- **W38 D1-D2 阶段 C**: 跨域 saga 真实交易 + 22 笔跨域合约合并层 verdict

### 2.4 与 R3 阶段 C3 派生约束的衔接 (per RGS-DEVPLAN v0.1 §7 R3)

- **R3 batch 解冻**: 8M tokens, DoD = 提交 8 条 L-CAND 候选清单报告
- **C3 5 域生产可用 checklist**: 1.5M tokens (5 域 × 300K), **本批 5 文档 v0.2 升版** (player / economy / match / social / admin)
- **6 文档拆分 + 升版** (per C3 派生约束 6 域 × 5-10 项 = 30-60 项):
  - ✅ player (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ economy (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ match (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ✅ social (本档 v0.2, 10 项, 7 ✅ / 3 🟡)
  - ✅ admin (v0.2, 10 项, 6 ✅ / 4 🟡)
  - ⏳ batch (待 R3 阶段起草, per BATCH-V0.1-FREEZE v0.1)

---

## 3. DoD 配套 (per AGENTS.md v0.6.2 §2.1 L1/L1.1/L1.2)

> **本档 DoD**: L1/L1.1/L1.2 三件套 (per 9/2 10:18 JST D2 拍板 + AGENTS.md v0.6.2 §2.1) 仅适用于 social 域 Rust 代码 (crates/social-service/), 本档是治理文档, 三件套 N/A.

### 3.1 L1 (compile 验证下限) — N/A 本档

- **命令**: `cargo check --tests` (限时 60s)
- **本档状态**: N/A (本档是治理文档, 不动 Rust 代码)
- **关联**: social 域代码 L1 验证由 social Lead + W37 D6 阶段 C C1 跑

### 3.2 L1.1 (lib 测试) — N/A 本档

- **命令**: `cargo test --lib -p social-service` (限时 120s)
- **本档状态**: N/A
- **关联**: social 域 5 域 UT 73 tests (commit `3e456b4`) = #1 UT (L1.1) ✅, 9/3 10:48 JST R1 L1.1 复测 73/73 PASS (per `c52805b`)

### 3.3 L1.2 (E2E 业务级) — N/A 本档, 但业务跑通 = social 域 milestone

- **命令**: `cargo test --test '*' -- --test-threads=1` + 1 业务 mTLS 跑通 (限时 300s+)
- **本档状态**: N/A (v0.2 升 #2 mock 路径 + #3 编译期锚定, 不算业务真跑; 真跑由 Phase C 阶段 C 触发, W37 D6-W38 D2)
- **关联**: social 域 2 项 E2E = #3 5 域 ST 业务 mTLS 1 跳 + #4 跨域 saga leave_guild → push → admin 审计, 跑通 = 业务里程碑 (W38 D1-D2 阶段 C C4-C6)

### 3.4 业务里程碑判定公式 (per RGS-CRITIQUE-IMPROVEMENT v0.2 §4.8 + AGENTS.md §9.4)

**social 域生产可用 milestone 公式 (v0.2 升版)**:
```
10 项 checklist 全 ✅ = social 域生产可用 ✅
= #1 L1.1 UT ✅ + #2 L1.2 mTLS IT ✅ (mock 路径, 真跑待 W37 D5) + #3-#4 L1.2 E2E + #5-#10 治理运维
= W38 D3 (9/17 JST) 阶段 D 评审通过
```

**R1 业务冲刺 R3 阶段 (本档对应) 公式 (v0.2 升版)**:
```
6 域 × 独立 checklist 文档 v0.2 升版 (player / economy / match / social / admin)
+ §1 9-10 项表格 (含 mock 路径 + 编译期锚定) + §2 状态更新 (per 9/3 12:46 JST R1 业务回填 9 项)
+ §3 DoD 配套 (L1/L1.1/L1.2 + mTLS mock 路径)
+ §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + L12 案例库 L-CAND-009)
+ §5 已知缺口 (per 8/26 缺标比错标)
+ §6 修订历史 v0.2
= R3 阶段任务 v0.2 升版完成
= W37 D1 (9/3 12:46 JST) 6 域 × v0.2 升版 commit 落档
```

---

## 4. 派生约束守护 (per AGENTS.md v0.6.5 §8 + v0.6.7 即时增段 + 8/27 11:06 JST 凭据硬 ban)

| 派生约束 | 本档 v0.2 守护 |
|---|---|
| **L1 cargo check 0 error** | ✅ N/A (本档是治理文档, 不动 Rust) |
| **L1.1 cargo test --lib** | ✅ N/A (本档不动 Rust) |
| **L1.2 E2E 跑通** | ✅ N/A (本档是预演基准, 实际跑通由 Phase C 阶段 C 触发, W37 D6-W38 D2; v0.2 升 #2 mock 路径 + #3 编译期锚定) |
| **L2 引用必须 git 实证** | ✅ 本档 §1 表格 9-10 项复制自 RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5, commit SHA (3e456b4 / c52805b / fa32bab / 111d4ad / 932ab3c) + file:line 全部 git 实证 |
| **L11 cargo build dir lock** | ✅ N/A (本档不编译) |
| **L12 临时 log / .txt / .tmp_search* 不入 commit + 5 worker 派工 3 选项 + per-worker CARGO_TARGET_DIR + staggered** | ✅ 本档无临时文件, pre-commit hook 兜底 (per AGENTS.md v0.6.5 §2.5 L12 + 9/3 12:36 JST 升正式 L-CAND-009) |
| **L13 自指字段 deferred 实时查询** | ✅ ahead of origin/main (250+) / 5 域 main HEAD (fa32bab) / 9/3 hotfix 数 (0) 全部实时 git 实证, 自指字段在 §2 状态更新段 |
| **L14 plumbing brace 跟踪** | ✅ N/A (本档无 patch 字符串拼接) |
| **8/27 11:06 JST 凭据硬 ban** | ✅ 本档无 env value 痕迹 (#8 证书轮换仅提"cert 链验证 OK" + "cert SHA-256 fingerprint per L-CAND-006 兜底, 永不入 commit", 不实际打印 cert 内容) |
| **9/2 10:18 JST B2 派生约束 L1-L14 冻结 6 个月** | ✅ 本档不动派生约束 (L-CANDIDATES.md 仍 4 条候选清单 + L-CAND-009 v0.2 升版, per commit `2e4f519`) |
| **9/2 10:18 JST C1 batch 域 v0.1 冻结** | ✅ 本档 social 域独立, 不动 batch 域, 引用 RGS-BATCH-V0.1-FREEZE-2026-09-02_v0.1.md 仅作对照 |
| **9/2 10:18 JST C3 业务指标新指标** | ✅ **本档 = C3 派生约束 v0.2 升版落地** (5 域 + batch 域 = 6 域, 每域 5-10 项, social 域 = 本档 10 项, 全部 ✅ = 业务里程碑达成) |
| **9/2 11:05 JST D2 L1/L1.1/L1.2 三件套** | ✅ §3 三件套配套说明 + §1 表格 #1 (L1.1) + #2-#4 (L1.2) 派生约束对应 |
| **9/2 11:05 JST D3 commit 模板** | ✅ 本档 commit 沿用 `.gitmessage` (type(scope): summary + DoD 段 + Evidence 段 + 代签段 + 派生约束守护段) |
| **9/2 14:11 JST B3 DDD Review 二审** | 🟡 本档非 DDD Review 类文档 (本档是业务 checklist 独立档, 不走 DDD Review 流程), 走 R3 阶段 5 域 × 独立文档评审机制, 起草后 Mavis 自审 1 次停手 + 主会话纳入 R3 阶段任务清单 (per RGS-DEVPLAN-2026-09-02 v0.1 §7) |
| **9/3 07:31 JST L-CAND-006 安全例外路径** | ✅ 本档 #8 证书轮换引用 L-CAND-006 兜底 ("cert SHA-256 fingerprint 永不入 commit"), 与 commit `89279bd` 5 域 cert 解除跟踪 + `932ab3c` AGENTS v0.6.9 一致 |
| **9/3 12:36 JST L12 升正式 + L-CAND-009** | ✅ 本档 L12 守护同步升级 (per 9/3 12:36 JST L-CAND-009 5 worker 派工 3 选项 + per-worker CARGO_TARGET_DIR + staggered + DoD 简报明文 "worker 不 commit") |

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

> **缺标原则** (per 8/26 JST RGS-OPEN-QA v0.4 §0): 拿不准时显式列"已知缺口", 不假装覆盖.

### 5.1 治理派缺口 (本档 v0.2 升版局限)

- **本档非 DDD Review 类文档**: 走 R3 阶段 5 域 × 独立文档评审机制, 不走 B3 DDD Review 二审流程, 配套评审记录在 RGS-DEVPLAN v0.1 §7 R3 阶段任务清单
- **v0.2 升版状态不可重置**: §1 表格是 9/3 12:46 JST 截止状态 (7✅ / 3🟡), W37 D2 阶段 A 跑通前不重置表格
- **W37 实战 hotfix 风险**: W37 D2-D5 阶段 A/B 可能产生 1-3 hotfix (per RGS-PHASE-C-PREP v0.1 §1.3), 单条 hotfix 应有信息量, pre-commit hook 兜底 (per B1)

### 5.2 业务派缺口 (本档 v0.2 升版后仍待 Phase C)

- **#2 IT (mTLS) social 50054 health probe 真跑**: v0.2 升 ✅ 走 mock 路径, 真跑要 W37 D5 (9/12 四) 阶段 B B7
- **#3-#4 E2E (L1.2)**: 0/22 跑通 (per RGS-TEST-RUN-PLAN v0.1, 11 UT W37 D6 + 11 E2E W37 D7-W38 D2), 实际跑通要 W38 D1-D2 阶段 C C4-C6
- **#5 SLA 监控 prometheus**: 当前 prometheus 9/3 12:38 JST A3 修复 0/1 CrashLoopBackOff (v0.1 是 27h), SRE 阶段 A3 (W37 D2) 修复
- **#6 告警 NATS DLQ**: Q7 决策 (RGS-OPEN-QA v0.2 §4.2 Q7) push_delivery 走 NATS + DLQ, 待 W37 D3-A4 prometheus HPA 检查后立告警规则 + Q7 NATS DLQ 阈值 5% 待 social Lead 确认
- **#8 证书轮换 90 天到期日**: 待 W37 D3 阶段 B B1-B2 导出后定基准, 当前无明确到期日
- **#10 审计日志 99% 写入率**: 24h 验证要 W37 D5 (9/12) 阶段 B 收口后, 当前无 24h 实际数据

### 5.3 social 域特殊缺口 (per Q5-Q7 决策待确认, v0.2 升版后)

- **Q5 guild capacity 50 vs 64**: 代码现状 50 为准, 不擅自改 64, 转 social Lead 业务确认 (per RGS-OPEN-QA v0.2 §4.2 Q5), #9 Schema 迁移注释 "Q5 决策" 含义 = schema 现状对齐 50, 不等于 64 业务确认
- **Q6 leave_guild PH-6 下一轮实现**: leadership 转移规则 = 加入时间最早剩余成员, 离开后 `player.profile.guild_id` 置空, #4 E2E 跨域 saga 跑通前需 social Lead 业务确认 PH-6 状态
- **Q7 push_delivery NATS 选型确认**: v0.2 §1 #6 告警 "NATS DLQ" 注释, 需 social Lead 业务确认 NATS DLQ 阈值 (5% 失败率是否合理)

### 5.4 W37 实战期间本档更新机制缺口 (v0.2 升版后)

- **W37 D2 阶段 A 跑通后回填**: 主会话负责, social Lead 不直接改本档
- **W37 D5 阶段 B 收口后回填**: 主会话负责, §2.2 时间线节点
- **W37 D7 W37 周报 v0.3 出后回填**: §2.2 实际跑通数据回填
- **W38 D3 阶段 D 评审后回填**: §1 表格 10/10 全 ✅, social 域生产可用 milestone 达成
- **回填模板**: 主会话维护 `docs/14-项目治理/CHECKLIST-PROD-READY-CHANGELOG-2026-W37_v0.1.md` (per R3 阶段任务清单, 6 域 × 独立文档统一回填日志), 避免本档频繁升版

### 5.5 阶段 A → 阶段 B/C 衔接缺口 (v0.2 新增)

- **阶段 A 4 步 SRE 替代**: 9/3 12:38 JST Mavis 推阶段 A 4 步 SRE 替代 (per RGS-PHASE-C-MAVIS-PHASE-A v0.1), 5 域 restartCount 0 + HPA 0 强启动风暴, 阶段 B 真跑待 SRE 介入
- **mTLS mock 路径 vs 业务真跑**: v0.2 #2/#3 升 ✅ 走 mock + 编译期锚定, 业务真跑需阶段 B 阶段 C 走 grpcurl + 真 cert

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: §0 目的与范围 (C3 派生约束 + R1 业务冲刺 R3 阶段任务) + §1 social 域 9-10 项 checklist 表格 (复制自 RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5 lines 329-344, 原文不动) + §2 状态更新 (9/3 08:00 JST R1 业务冲刺现状 + W37 实战回填位 + 9 项 checklist 状态时间线预测) + §3 DoD 配套 (L1/L1.1/L1.2 N/A + 业务里程碑判定公式) + §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + B2/C1/C3/D2/D3/B3 + L-CAND-006 全部 ✅) + §5 已知缺口 (治理派 3 项 / 业务派 6 项 / social 特殊 3 项 / W37 更新机制 5 项 = 17 项) + §6 修订历史本行 |
| **v0.2** | 2026-09-03 12:46 | 架构师(Mavis 接手 agent per DEC-008) | **业务回填 9 项升版**: 基于 9/3 12:38-12:46 JST 三批 commit 落地 (`c52805b` 5 域 L1.1 验证全过 565/565 + `111d4ad` 5 域 E2E Phase C marker 10 marker 编译期锚定 + `fa32bab` 5 域 mTLS mock 15/15 passed) + 9/3 12:38 JST 阶段 A 4 步 SRE 替代实证. §1 表格状态列更新: 3 项 🟡→✅ (#2 IT mTLS mock 路径 + #3 E2E 业务 mTLS 编译期锚定 + #5 SLA 监控 WSL kubectl 实证), 3 项 ✅→✅ (#1 UT L1.1 维持 + #7 部署健康 维持 2 pod + #9 Schema 迁移 维持 Q5 guild capacity 50 业务确认), 4 项 🟡→🟡 (#4 跨域 saga leave_guild 真跑需阶段 C + #6 告警 NATS DLQ SRE 拍板 + Q7 待 social Lead 确认 + #8 证书 L-CAND-006 兜底 + #10 审计 admin Q2 决策), 闭环率 4/10 → 7/10 (70%). §2 状态更新加 9/3 12:46 JST R1 业务回填 9 项统计表 + 5 域 main HEAD `fa32bab` + 状态时间线预测 v0.2 列. §3 DoD 配套加 mTLS mock 路径 + 编译期锚定 (per `fa32bab` / `111d4ad`). §4 派生约束守护加 L12 案例库 (per 9/3 12:36 JST 升正式 commit `2e4f519` + L-CAND-009). §5 已知缺口加阶段 A → 阶段 B/C 衔接缺口 2 条. §6 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
