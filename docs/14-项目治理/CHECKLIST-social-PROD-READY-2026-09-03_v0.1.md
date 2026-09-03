# CHECKLIST-social-PROD-READY-2026-09-03 v0.1 — social 域生产可用 checklist 独立文档

> **创建日期**: 2026-09-03 11:06 JST
> **作者**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **审批**: 架构师(Mavis 接手 agent per DEC-008)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **依据**: C3 派生约束 (per RGS-CRITIQUE-IMPROVEMENT v0.1.1 §3.3 + v0.2 §3.3) + AGENTS.md v0.6.4 §9.4 里程碑重定义 + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 阶段 + RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §4.5 social 域 9 项 checklist
> **配套**: RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 (commit `c52805b` 后续 W37 实战主文档) + RGS-PHASE-C-PREP-2026-09-02 v0.1 (阶段 A/B/C/D) + RGS-PHASE-C-KICKOFF-2026-09-02 v0.1 §6 W37 后续工作 5 天
> **作用域**: social 域独立落地 (player / economy / match / admin / batch 5 域按同模板拆 v0.1, social 域为第 1 拆落地)

---

## 0. 目的与范围 (per C3 派生约束 + R1 业务冲刺 R3 阶段任务)

**触发**: per 2026-09-02 16:10 JST 拍板 C3 (RGS-CRITIQUE-IMPROVEMENT v0.1.1 §3.3) + 9/2 18:30 JST v0.2 反馈 (RGS-CRITIQUE-IMPROVEMENT-2026-09-02 v0.2 §3.3 + §4) + RGS-DEVPLAN-2026-09-02 v0.1 §7 R3 阶段任务 (5 域生产可用 milestone 业务冲刺)。

**目的**:
1. 把 RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5 social 域 9 项 checklist **独立成档**, 便于 social Lead 单独追踪 + 状态更新 (避免在主文档长篇治理反思里搜)
2. 作为 **social 域生产可用 milestone** 客观度量 (取代 v0.1.1 老指标"派生约束 L1-L14 100% 闭环")
3. 5 域 × 独立 checklist 文档 (player / economy / match / social / admin) + batch (冻结) 6 文档系列 = §4 总览 60 项基准, 本文档是 social 域独立档
4. 9/3 08:00 JST R1 业务冲刺现状 → 9/8-14 W37 实战 → 9/15-19 W38 衔接 = 状态更新主线

**本档 vs 主文档关系**:
- 主文档 `RGS-CRITIQUE-IMPROVEMENT-2026-09-02_v0.2.md` §4.5 = 治理反思视角的 social 域 checklist
- 本档 `CHECKLIST-social-PROD-READY-2026-09-03_v0.1.md` = social Lead 业务冲刺视角的 checklist, 状态可独立更新, 9/2 拍板 + W37 实战 + W38 衔接

**R1 业务冲刺 R3 阶段任务对应** (per RGS-DEVPLAN-2026-09-02 v0.1 §7):
- R1 (UT + IT 8 套件冻结) → 已落地 (commit `c52805b` admin/r2-fix 565/565 passed)
- R2 (5 域 main 二轮修复) → 已落地 (per commit `6bc55ec` admin verify_recent_n 修复, 等 5 域全过)
- **R3 (5 域生产可用 checklist 落地) = 本档**
- R4 (Phase C SRE 介入) → 待 W37 D2 启动
- R5 (5 域 E2E 业务 mTLS 跑通) → W37 D6-W38 D2 阶段 C

---

## 1. social 域 9-10 项 checklist (per RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5 原文)

> **来源**: 复制自 `docs/14-项目治理/RGS-CRITIQUE-IMPROVEMENT-2026-09-02_v0.2.md` §4.5 lines 329-344, 不修改原表内容, 仅在 §2 状态更新段添加 W37 实战回填.

| # | 类别 | 检查项 | 工具 | DoD | 状态 (v0.2 原表) | W37 实战 (v0.2 原表) |
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

**social 域 9/10 闭环** = social 域生产可用 ✅ (per v0.2 §4.5 判定)

**关联决策引用** (per RGS-OPEN-QA-2026-08-31 v0.2 §4.2 social 域 Q5-Q7 拍板):
- **Q5 guild capacity 50 vs 64**: 代码现状 50 为准, 不擅自改 64, 转 social Lead 业务确认 → 对应 #9 Schema 迁移 "Q5 决策" 注释
- **Q6 leave_guild**: PH-6 社交域下一轮实现, leadership 转移规则 = 加入时间最早剩余成员, 离开后 `player.profile.guild_id` 置空 → 对应 #4 E2E 跨域 saga 真实交易
- **Q7 push_delivery dispatcher**: 走 NATS (不新增 FCM/APNs 直连), retry 复用 economy outbox+saga 模式, 需要 DLQ → 对应 #6 告警 "NATS DLQ" 注释

---

## 2. 状态更新 (per 9/3 08:00 JST R1 业务冲刺现状)

> **更新原则**: 本节是 v0.2 原表 9/2 18:30 JST 截止后的增量更新, 沿用 v0.2 §4.5 表格 + 状态图标 (✅/🟡/❌/🆕), W37 D2 阶段 A 跑通前状态不重置, 仅在 W37 实战节点回填实际进展.

### 2.1 9/3 08:00 JST R1 业务冲刺现状

| 维度 | 9/2 18:30 (v0.2) | 9/3 08:00 (R1 现状) | 趋势 | 引用 |
|---|---|---|---|---|
| 5 域 main HEAD | `c52805b` (admin/r2-fix merge) | `c52805b` (持平, 9/2 18:30 后无新 merge) | ➡ 持平 | `git log --oneline -1` |
| 5 域 L1.1 验证 | admin 117/117 passed (5 域 565/565) | **5 域 565/565** 持平 (per admin/r2-fix 落地) | ✅ 持平 | commit `c52805b` |
| 9/3 hotfix 数 | 0 (per 9/2 持平) | **0** (持平, B1 pre-commit hook 兜底生效) | ✅ 持平 | `git log --since 2026-09-03` |
| ahead of origin/main | 221 commit (per v0.2 §1) | **228+** (⬆ +7, 9/2 18:30~9/3 08:00 增 L-CAND-006 落地 5 commit + 临时文件清理 1 commit + AGENTS v0.6.9 1 commit) | ⬆ +7 | `git log --oneline origin/main..HEAD \| Measure-Object` |
| social 域 binary 运行态 | W36 末 24h 0 restart | **9/2 18:30~9/3 08:00 0 restart** (持平, SRE 摸底前) | ✅ 持平 | `kubectl get pods -l app=social-service -o jsonpath='{.items[*].status.containerStatuses[0].restartCount}'` |
| social 域 9 项 checklist 状态 | v0.2 §4.5 4✅ / 6🟡 | **4✅ / 6🟡** 持平 (W37 D2 阶段 A 启动前) | ➡ 持平 | 本档 §1 |
| L-CAND-006 安全例外路径 | 未立 (候选清单) | ✅ **已立** (commit `932ab3c` 9/3, 8/31 ST 5 域 cert 解除跟踪) | ✅ 立 | AGENTS.md v0.6.9 + L-CAND-006-EXCEPTION-PATH-2026-09-03_v0.1.md |

### 2.2 W37 实战回填位 (per RGS-PHASE-C-KICKOFF v0.1 §6)

> **回填节点**: W37 D2 (9/9 一) 阶段 A 跑通后, W37 D5 (9/12 四) 阶段 B 收口后, W37 D6 (9/13 五) 阶段 C 启动后, W37 D7 (9/14 六) W37 周报 v0.3 出后. 主会话在节点完成后回填本节, social Lead 不直接改.

| 节点 | 状态变化预期 | 回填字段 |
|---|---|---|
| **W37 D2 (9/9)** 阶段 A 跑通 | #5 SLA 监控 prometheus PVC 备份后, 当前 restartCount 锁定基准 | #5 状态 / 实际 restartCount |
| **W37 D3 (9/10)** 阶段 B 启动 + certs 导出 | #8 证书轮换基准锁定, 90 天后到期日 | #8 状态 / 90 天到期日 / cert SHA-256 fingerprint (per L-CAND-006 兜底, 永不入 commit) |
| **W37 D5 (9/12)** 阶段 B 收口 + social 50054 gRPC health probe | #2 IT (mTLS) SERVING, #10 审计日志最近 1000 条 verify | #2 / #10 状态 / actual probe 结果 |
| **W37 D6 (9/13)** 阶段 C 启动 + 11 UT 真跑 | #1 UT (L1.1) 跑通, R1 业务冲刺 R3 阶段 = social 域 4✅ / 6🟡 → 5✅ / 5🟡 (升 1) | #1 状态 / 11/11 PASS 时间 |
| **W37 D7 (9/14)** 11 E2E 准备 | 不触发 #3 #4 (E2E 待 W38 D1-D2), 仅准备 | 准备状态 |
| **W38 D1-D2 (9/15-16)** 阶段 C 11 E2E 真跑 + 跨域 saga | #3 E2E 1 跳 5 域 ST 业务 mTLS + #4 E2E 跨域 saga leave_guild → push → admin 审计 | #3 / #4 状态 / leave_guild 实际跑通 |
| **W38 D3 (9/17)** 阶段 D 评审 + 5 域 E2E 跑通 = 业务里程碑 | 6🟡 → 全部 ✅, social 域 10/10 闭环 = social 域生产可用 ✅ | 全部 ✅ + 实际跑通时间 |

### 2.3 social 域 9 项 checklist 状态时间线预测

| # | v0.2 9/2 18:30 | W37 D2 (9/9) | W37 D5 (9/12) | W37 D6 (9/13) | W38 D3 (9/17) |
|---|---|---|---|---|---|
| 1 UT (L1.1) | ✅ | ✅ | ✅ | ✅ 重跑 | ✅ |
| 2 IT (mTLS) | 🟡 | 🟡 | ✅ health probe SERVING | ✅ | ✅ |
| 3 E2E (L1.2) | 🟡 | 🟡 | 🟡 | 🟡 | ✅ 阶段 C C4-C5 |
| 4 E2E (L1.2) | 🟡 | 🟡 | 🟡 | 🟡 | ✅ 阶段 C C6 |
| 5 SLA 监控 | 🟡 | ✅ 基准锁定 | ✅ | ✅ | ✅ |
| 6 告警 | 🟡 | 🟡 | 🟡 | 🟡 | ✅ W37 D3-A4 |
| 7 部署健康 | ✅ | ✅ | ✅ | ✅ | ✅ 7 天 0 restart |
| 8 证书轮换 | 🟡 | 🟡 基准 | ✅ 90 天到期日 | ✅ | ✅ |
| 9 Schema 迁移 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 10 审计日志 | 🟡 | 🟡 | ✅ 增量 verify | ✅ | ✅ |
| **统计** | **4✅ / 6🟡** | **4✅ / 6🟡** | **6✅ / 4🟡** | **7✅ / 3🟡** | **10✅ / 0🟡** |

**social 域业务里程碑判定** (per AGENTS.md v0.6.4 §9.4 + RGS-CRITIQUE-IMPROVEMENT v0.2 §4.8): W38 D3 (9/17 JST) 10/10 闭环 = social 域生产可用 ✅.

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
- **关联**: social 域 5 域 UT 47 tests (commit `3e456b4`) = #1 UT (L1.1) ✅, W37 D6 11 UT 真跑时重跑 (per §2.3 时间线)

### 3.3 L1.2 (E2E 业务级) — N/A 本档, 但业务跑通 = social 域 milestone

- **命令**: `cargo test --test '*' -- --test-threads=1` + 1 业务 mTLS 跑通 (限时 300s+)
- **本档状态**: N/A
- **关联**: social 域 2 项 E2E = #3 5 域 ST 业务 mTLS 1 跳 + #4 跨域 saga leave_guild → push → admin 审计, 跑通 = 业务里程碑 (W38 D1-D2 阶段 C C4-C6)

### 3.4 业务里程碑判定公式 (per RGS-CRITIQUE-IMPROVEMENT v0.2 §4.8 + AGENTS.md §9.4)

**social 域生产可用 milestone 公式**:
```
10 项 checklist 全 ✅ = social 域生产可用 ✅
= #1 L1.1 UT + #2 L1.2 mTLS IT + #3-#4 L1.2 E2E + #5-#10 治理运维
= W38 D3 (9/17 JST) 阶段 D 评审通过
```

**R1 业务冲刺 R3 阶段 (本档对应) 公式**:
```
6 域 × 独立 checklist 文档 (player / economy / match / social / admin + batch 冻结)
+ §1 9-10 项表格 + §2 状态更新 + §3 DoD 配套 + §4 派生约束守护 + §5 已知缺口 + §6 修订历史
= R3 阶段任务完成
= W37 D1 (9/8 日) 6 域 × 独立文档 commit 落档 (本档 = social 域首拆, 5 域按本模板复制)
```

---

## 4. 派生约束守护 (per AGENTS.md v0.6.5 §8 + v0.6.7 即时增段 + 8/27 11:06 JST 凭据硬 ban)

| 派生约束 | 本档守护 |
|---|---|
| **L1 cargo check 0 error** | ✅ N/A (本档是治理文档, 不动 Rust) |
| **L1.1 cargo test --lib** | ✅ N/A (本档不动 Rust) |
| **L1.2 E2E 跑通** | ✅ N/A (本档是预演基准, 实际跑通由 Phase C 阶段 C 触发, W37 D6-W38 D2) |
| **L2 引用必须 git 实证** | ✅ 本档 §1 表格 9-10 项复制自 RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5 lines 329-344, commit SHA (3e456b4 / 6bc55ec / 932ab3c) + file:line (c52805b 后续主文档) 全部 git 实证 |
| **L11 cargo build dir lock** | ✅ N/A (本档不编译) |
| **L12 临时 log / .txt / .tmp_search* 不入 commit** | ✅ 本档无临时文件, pre-commit hook 兜底 |
| **L13 自指字段 deferred 实时查询** | ✅ ahead of origin/main (228+) / 5 域 main HEAD (c52805b) / 9/3 hotfix 数 (0) 全部实时 git 实证, 自指字段在 §2.1 状态更新段 |
| **L14 plumbing brace 跟踪** | ✅ N/A (本档无 patch 字符串拼接) |
| **8/27 11:06 JST 凭据硬 ban** | ✅ 本档无 env value 痕迹 (#8 证书轮换仅提"cert 链验证 OK" + "cert SHA-256 fingerprint per L-CAND-006 兜底, 永不入 commit", 不实际打印 cert 内容) |
| **9/2 10:18 JST B2 派生约束 L1-L14 冻结 6 个月** | ✅ 本档不动派生约束 (L-CANDIDATES.md 仍 3 条候选清单 + 1 保留位) |
| **9/2 10:18 JST C1 batch 域 v0.1 冻结** | ✅ 本档 social 域独立, 不动 batch 域, 引用 RGS-BATCH-V0.1-FREEZE-2026-09-02_v0.1.md 仅作对照 |
| **9/2 10:18 JST C3 业务指标新指标** | ✅ **本档 = C3 派生约束落地** (5 域 + batch 域 = 6 域, 每域 5-10 项, social 域 = 本档 10 项, 全部 ✅ = 业务里程碑达成) |
| **9/2 11:05 JST D2 L1/L1.1/L1.2 三件套** | ✅ §3 三件套配套说明 + §1 表格 #1 (L1.1) + #2-#4 (L1.2) 派生约束对应 |
| **9/2 11:05 JST D3 commit 模板** | ✅ 本档 commit 沿用 `.gitmessage` (type(scope): summary + DoD 段 + Evidence 段 + 代签段 + 派生约束守护段) |
| **9/2 14:11 JST B3 DDD Review 二审** | 🟡 本档非 DDD Review 类文档 (本档是业务 checklist 独立档, 不走 DDD Review 流程), 走 R3 阶段 5 域 × 独立文档评审机制, 起草后 Mavis 自审 1 次停手 + 主会话纳入 R3 阶段任务清单 (per RGS-DEVPLAN-2026-09-02 v0.1 §7) |
| **9/3 07:31 JST L-CAND-006 安全例外路径** | ✅ 本档 #8 证书轮换引用 L-CAND-006 兜底 ("cert SHA-256 fingerprint 永不入 commit"), 与 commit `89279bd` 5 域 cert 解除跟踪 + `932ab3c` AGENTS v0.6.9 一致 |

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标)

> **缺标原则** (per 8/26 JST RGS-OPEN-QA v0.4 §0): 拿不准时显式列"已知缺口", 不假装覆盖.

### 5.1 治理派缺口 (本档起草局限)

- **本档非 DDD Review 类文档**: 走 R3 阶段 5 域 × 独立文档评审机制, 不走 B3 DDD Review 二审流程, 配套评审记录在 RGS-DEVPLAN v0.1 §7 R3 阶段任务清单
- **W37 实战前状态不可重置**: §1 表格是 9/2 18:30 JST 截止状态 (4✅ / 6🟡), 9/3 08:00 JST R1 现状持平, W37 D2 阶段 A 跑通前不重置表格
- **W37 实战 hotfix 风险**: W37 D2-D5 阶段 A/B 可能产生 1-3 hotfix (per RGS-PHASE-C-PREP v0.1 §1.3), 单条 hotfix 应有信息量, pre-commit hook 兜底 (per B1)

### 5.2 业务派缺口 (本档仍待 Phase C)

- **#2 IT (mTLS) social 50054 health probe**: 实际跑通要 W37 D5 (9/12 四) 阶段 B B7, 当前仅 5 域 ST 业务 mTLS = 1/5 (gm-backend 8081 HTTP only, gRPC 待阶段 B/C, per RGS-CRITIQUE-IMPROVEMENT v0.2 §1 行 51)
- **#3-#4 E2E (L1.2)**: 0/22 跑通 (per RGS-TEST-RUN-PLAN v0.1, 11 UT W37 D6 + 11 E2E W37 D7-W38 D2), 实际跑通要 W38 D1-D2 阶段 C C4-C6
- **#5 SLA 监控 prometheus**: 当前 prometheus CrashLoopBackOff 27h (per RGS-PHASE-C-PREP v0.1 §3.5), SRE 阶段 A3 (W37 D2) 修复
- **#6 告警 NATS DLQ**: Q7 决策 (RGS-OPEN-QA v0.2 §4.2 Q7) push_delivery 走 NATS + DLQ, 待 W37 D3-A4 prometheus HPA 检查后立告警规则
- **#8 证书轮换 90 天到期日**: 待 W37 D3 阶段 B B1-B2 导出后定基准, 当前无明确到期日
- **#10 审计日志 99% 写入率**: 24h 验证要 W37 D5 (9/12) 阶段 B 收口后, 当前无 24h 实际数据

### 5.3 social 域特殊缺口 (per Q5-Q7 决策待确认)

- **Q5 guild capacity 50 vs 64**: 代码现状 50 为准, 不擅自改 64, 转 social Lead 业务确认 (per RGS-OPEN-QA v0.2 §4.2 Q5), #9 Schema 迁移注释 "Q5 决策" 含义 = schema 现状对齐 50, 不等于 64 业务确认
- **Q6 leave_guild PH-6 下一轮实现**: leadership 转移规则 = 加入时间最早剩余成员, 离开后 `player.profile.guild_id` 置空, #4 E2E 跨域 saga 跑通前需 social Lead 业务确认 PH-6 状态
- **Q7 push_delivery NATS 选型确认**: v0.2 §4.5 #6 告警 "NATS DLQ" 注释, 需 social Lead 业务确认 NATS DLQ 阈值 (5% 失败率是否合理)

### 5.4 W37 实战期间本档更新机制缺口

- **W37 D2 阶段 A 跑通后回填**: 主会话负责, social Lead 不直接改本档
- **W37 D5 阶段 B 收口后回填**: 主会话负责, §2.2 时间线节点
- **W37 D7 W37 周报 v0.3 出后回填**: §2.2 实际跑通数据回填
- **W38 D3 阶段 D 评审后回填**: §1 表格 10/10 全 ✅, social 域生产可用 milestone 达成
- **回填模板**: 主会话维护 `docs/14-项目治理/CHECKLIST-PROD-READY-CHANGELOG-2026-W37_v0.1.md` (per R3 阶段任务清单, 6 域 × 独立文档统一回填日志), 避免本档频繁升版

---

## 6. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-03 11:06 | 架构师(Mavis 接手 agent per DEC-008) | 初始创建: §0 目的与范围 (C3 派生约束 + R1 业务冲刺 R3 阶段任务) + §1 social 域 9-10 项 checklist 表格 (复制自 RGS-CRITIQUE-IMPROVEMENT v0.2 §4.5 lines 329-344, 原文不动) + §2 状态更新 (9/3 08:00 JST R1 业务冲刺现状 + W37 实战回填位 + 9 项 checklist 状态时间线预测) + §3 DoD 配套 (L1/L1.1/L1.2 N/A + 业务里程碑判定公式) + §4 派生约束守护 (L1-L14 + 8/27 凭据硬 ban + B2/C1/C3/D2/D3/B3 + L-CAND-006 全部 ✅) + §5 已知缺口 (治理派 3 项 / 业务派 6 项 / social 特殊 3 项 / W37 更新机制 5 项 = 17 项) + §6 修订历史本行 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
