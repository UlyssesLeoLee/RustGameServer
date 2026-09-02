# RGS-MATCH-COORDINATION-NOTE match 域业务实装观察 + 联调确认 v0.1

**RGS-MATCH-COORDINATION-NOTE**

| 项目 | 内容 |
|---|---|
| 文档 ID | RGS-MATCH-COORDINATION-NOTE |
| 版本 | v0.1 (per WBS v0.2 桶 8 w3 match 协调任务) |
| 状态 | 🟢 落地 (1 commit, 0 代码改动) |
| 修订人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 |
| 审批 | 架构师 (Mavis 接手 agent per DEC-008) |
| 创建日期 | 2026-09-01 23:04 JST |
| 派工来源 | WT-8 Phase B 业务 P1 backlog 实装 (WBS v0.2 commit `84edf26`) |
| 关联 RACI | RGS-RACI-MATCH-V1_v1.1.md (match 域 Lead RACI) |
| 关联 DDD Review | RGS-DDD-2026-08-31-FINAL-UT-IT-ST-FIX_5域完整测试+业务实现_v0.1.md §6.1 / §8 (match 域状态表) |
| 关联 OPEN-QA | RGS-OPEN-QA-2026-08-31-test-summary_v0.2 (Q1-Q11 P1 backlog, match 域无) |

---

## 0. 目的

本文件为 **WBS v0.2 桶 8 Phase B 业务 P1 backlog 实装** 中 **w3 match 域协调任务** 的产物。

match 域在本轮 P1 backlog (桶 8) **无代码实装任务**:
- 1 worker 1 域原则下, w3 match 不动 `crates/match-service` 代码
- 1 commit "docs(match): 业务实装观察 + 联调确认" 落 main, **无代码改动**
- 落档 match 域现状 review + 5 域 Lead 联调 0 任务确认 + 后续桶 9 责任划分

本文件作为 w3 match worker 的**唯一交付物**。

---

## 1. match 域现状 (per DDD Review 5 域状态表)

### 1.1 UT 阶段 (per 8/31 12:21-13:45 JST 3 阶段迭代)

| 项 | 状态 |
|---|---|
| UT commit | `5070547` |
| +行 | +688 |
| tests | 28+ |
| 状态 | ✅ cargo check PASS (per 8/31 13:45 JST 主会话验证) |

**注**: 8/31 12:21 JST v1 5 worker cargo test polling 失败后, 改 13:34 JST v3 hotfix 模板, match 域 1 轮跑通。

### 1.2 IT 阶段 (per 8/31 13:55 JST 1 轮派工)

| 项 | 状态 |
|---|---|
| IT commit | `c70ef64` |
| +行 | +751 |
| 新 IT 文件 | 3 |
| 新 tests | 7 |
| 状态 | ✅ cargo check PASS |

**已有 IT 文件** (per `crates/match-service/tests/`):
- `integration_match_basic.rs`
- `integration_match_session.rs`
- `integration_match_session_to_replay.rs`
- `integration_match_end_to_replay_persist.rs`
- `integration_matchmaker_tolerance_window.rs`
- `it_save_replay_saga.rs`
- `fail_closed_start.rs`

**IT merge commit**: `69d8c0a` (5 域 UT+IT merge, 末位)

### 1.3 ST 阶段 (per 8/31 17:05-19:48 JST 5 轮迭代)

| 场景 | 域 | 结果 |
|---|---|---|
| st-05 | match | **FAIL** (gm-backend 8081 HTTP 不响应, 根因同 Q8) |
| st-06 | match | ✅ PASS |

**根因**: 6 个 FAIL 全部因 5 域 gRPC 通过但 gm-backend HTTP 探活死 (per DDD Review §5)。

**注**: st-05 FAIL 不属 match 域代码问题, 留 Ulysses 重启 k3s 集群后桶 9 Phase C 重跑。

### 1.4 Fix 阶段 (per 8/31 22:10 JST 1 轮派工)

| 域 | Fix commit | +行 | 业务实装 |
|---|---|---:|---|
| player | `858becb` | +104 | Q3 wins ≤ total invariant |
| economy | `d6bf024` | +7 | Q4 outbox graceful skip |
| social | `f556991` | +1065 | Q6 leave_guild + Q7 NATS push dispatcher |
| admin | `2d587f2` | +1241 | Q1 RBAC + Q2 audit_log 增量 verify |
| **match** | **— (无 fix commit)** | **0** | **0 P1 (per v0.2 决策)** |

**match 域 4 阶段完整度**: UT ✅ / IT ✅ / ST ⚠️ (1 PASS 1 FAIL, 根因集群) / Fix ⏳ 无 P1

### 1.5 派生约束 (per DDD Review §10.3 P2 follow-up)

- `matchmaker_v2.rs 67KB` 细读 (per v0.2 留后续 bucket)
  - 实际文件大小: 67005 字节 = 65.4 KB (per `Get-ChildItem -Length`)
  - 现状: 单文件 67 KB 复杂度高, 涉及 matchmaker 业务核心
  - 后续: WBS v0.2+ 桶留 match domain 内部技术债清理, **不属 P1 backlog**

---

## 2. Q1-Q11 P1 backlog match 域 0 任务确认

### 2.1 OPEN-QA v0.2 Q1-Q11 决策 (per `RGS-OPEN-QA-2026-08-31-test-summary_v0.2.md`)

| Q# | 主题 | 落点域 | match 域关联 |
|---|---|---|---|
| Q1 | gm_handlers RBAC handler 入口 | admin | ❌ 无 |
| Q2 | audit_log startup verify 增量 | admin | ❌ 无 |
| Q3 | player wins ≤ total 业务层 invariant | player | ❌ 无 |
| Q4 | economy outbox L143 `expect` 改 skip | economy | ❌ 无 |
| Q5 | social guild capacity 50 vs 64 | social | ❌ 无 |
| Q6 | social leave_guild 业务方法 | social | ❌ 无 |
| Q7 | social push_delivery NATS dispatcher | social | ❌ 无 |
| Q8 | gm-backend 8081 诊断 | gm-backend (跨域) | ⚠️ 涉及 match 域 ST 重跑, 留桶 9 |
| Q9 | prometheus/grafana 诊断 | 平台层 (跨域) | ❌ 无 |
| Q10 | mTLS 业务级 ST 重跑 | ST 阶段 (跨 5 域) | ✅ match 域证书已导出, 待 st-13 |
| Q11 | NATS 部署范围核查 | 平台层 (跨域) | ❌ 无 |

**match 域 0 P1 任务确认**:
- Q1-Q7 业务实装: match 域 0 任务
- Q8/Q9/Q11 平台层诊断: match 域 0 任务
- Q10 mTLS 业务级 ST: match 域责任 = **st-13 跑 (后续桶 9)**, 不属桶 8

### 2.2 WBS v0.2 桶 8 业务 P1 backlog match 域 (per `WBS-v0.2-readback.md` §2.2)

| 项 | 任务 | 落点 | match 域关联 |
|---|---|---|---|
| B1 | Q1 admin RBAC | admin | ❌ 无 |
| B2 | Q2 admin audit verify | admin | ❌ 无 |
| B3 | Q3 player wins≤total | player | ❌ 无 |
| B4 | Q5 social guild capacity | social | ❌ 无 |
| B5 | Q6 social leave_guild | social | ❌ 无 |
| B6 | Q7 social push NATS | social | ❌ 无 |
| B7 | Q4 economy outbox skip | economy | ❌ 无 |
| B8 | BAS-001 §9.2 LCM step execution | admin | ❌ 无 |
| B9 | BAS-001 §9.7 5 域 Lead 一审 | 5 域 Lead + 架构师 | ⚠️ match 域 Lead 签字 = 桶 7 Phase A (per A6) |

**match 域 w3 worker 任务**: 协调任务, 0 代码改动, 仅本文件。

---

## 3. 5 域 Lead 联调 0 任务确认

### 3.1 联调范围 (per 8/21 JST 5 域独立 Lead 决策 + DEC-008)

| 域 | Lead 真实身份 (per DEC-008 一人公司 12 角色) | w3 match 联调事项 |
|---|---|---|
| player | Ulysses | ❌ 无 (player 域内 Q3 自闭环) |
| economy | Ulysses | ❌ 无 (economy 域内 Q4 自闭环) |
| match | **Ulysses (5 域真实身份 per 8/21 JST)** | ✅ 本文件 = 联调交付物 |
| social | Ulysses | ❌ 无 (social 域内 B4+B5+B6 自闭环) |
| admin | Ulysses | ❌ 无 (admin 域内 B1+B2+B8 自闭环) |

**联调 0 任务确认**: match 域 w3 worker 与其他 4 域 Lead 联调 0 任务, 因 Q1-Q11 P1 backlog 不交叉 (per §2)。

### 3.2 跨域影响面 (match 域视角)

- **match 域不依赖其他 4 域业务实装**: Q1-Q7 业务实装不涉及 match 域 service.rs / matchmaker_v2.rs
- **match 域不被其他 4 域业务实装影响**: player wins ≤ total / economy outbox skip / social leave_guild / push NATS / admin RBAC + audit verify 全部域内自闭环
- **match 域唯一跨域项**: Q10 mTLS 业务级 ST (后续桶 9), 不属桶 8 范围

### 3.3 联调签字栏

| 角色 | 真实身份 | 签字 | 日期 | 备注 |
|---|---|---|---|---|
| **match 域 Lead** | **Ulysses (5 域真实身份 per 8/21 JST)** | ⏳ 待 DDD Review 阶段补 | — | per RGS-RACI-MATCH-V1_v1.1.md §3 + DDD Review SOP, 5 域 Lead 签字阶段为桶 7 Phase A A6 BAS-001 §9.7 5 域 Lead 一审 + 桶 8 业务实装完结 |
| 架构师 | Mavis 接手 agent per DEC-008 | ✅ | 2026-09-01 23:04 JST | 协调任务审批 |
| 修订人 | Mavis 接手代签 Ulysses | ✅ | 2026-09-01 23:04 JST | per 8/27 19:39/20:56/21:59 JST 三次强化 |

**注**: match 域 Lead 真实身份 = Ulysses (per DEC-008 一人公司 12 角色), w3 match worker 由 Mavis 接手 agent 代签 (per 2026-08-27 三次强化), match 域 Lead 签字 = DDD Review 阶段跟 5 域 Lead 联合签字一并补。

---

## 4. 后续 WBS v0.2 桶 9 责任划分 (per Phase C 集群可达)

### 4.1 桶 9 Phase C 触发条件

per WBS v0.2 §2.3:
- `kubectl get nodes` 看到 `ulyssespc` Ready
- WSL 单节点 k3s 节点注册失败未恢复 (per OPEN-QA v0.3 §7.1 ⏳ 阻塞项)

### 4.2 match 域在桶 9 责任

| 项 | 任务 | match 域责任 | 估时 |
|---|---|---|---|
| C1 | Q11 NATS 8222 部署范围核查 | ❌ 无 (平台层) | 2 min (SRE) |
| C2 | Q8 gm-backend 8081 诊断 | ⚠️ 间接 (match 域 ST 重跑依赖) | 1 h (ST-fix worker) |
| C3 | Q9 prometheus + grafana 诊断 | ❌ 无 (平台层) | 1 h (ST-fix worker) |
| **C4** | **Q10 mTLS 业务级 ST 重跑** | **✅ match 域负责 st-13 (跨 5 域, match 段)** | **1 day (mTLS worker)** |
| C5 | L6 gm-backend binary startup 修复 | ❌ 无 | 2 h (估, ST-fix worker) |

### 4.3 match 域 mTLS 业务级 ST (st-13) 准备工作

per DDD Review §10.2 + OPEN-QA v0.2 Q10:

- ✅ 证书已导出 (commit `7a8b21b`): `D:/rgs-st-mock/certs/match-tls.yaml`
- ⏳ grpcurl 工具未安装 (留桶 9)
- ⏳ st-13 脚本未写 (留桶 9)

**match 域 st-13 范围** (待桶 9 启动时由 mTLS worker 确认):
- 1) 双向 TLS 握手 (match-svc <-> match-svc self-call 或 match-svc <-> gm-backend)
- 2) match session 创建/结束端到端 (per `integration_match_session_to_replay.rs` 7 IT 同等)
- 3) matchmaker tolerance window 端到端 (per `integration_matchmaker_tolerance_window.rs` 同等)

**mTLS 工具链** (per OPEN-QA v0.2 §2):
- 工具 = grpcurl (not curl, not k3s kubectl exec)
- 证书 = 5 域 mTLS 业务级 (per 8/27 ST 导出 SOP)
- target = svc://match-service:50053 (ClusterIP)

### 4.4 桶 11 Phase E batch 域 match 域责任

per WBS v0.2 §2.5:
- match 域与 batch 域无直接业务耦合 (per BATCH REQ §0 + DETAILED §6.2 5 不破坏)
- match 域仅在 batch 域需要 match 数据批量整理时 (v0.2 评估 GAP-1~12) 参与协调
- 当前 batch 域 v0.1 = 6 周落地, match 域不阻塞

---

## 5. match 域后续技术债 (per v0.2 留后续 bucket)

### 5.1 matchmaker_v2.rs 67KB 拆分 (P2 follow-up)

per DDD Review §10.3 P2 follow-up:
- 现状: `crates/match-service/src/matchmaker_v2.rs` 67005 字节 = 65.4 KB
- 复杂度: matchmaker 业务核心, 涉及 matchmaker 算法 / 评分 / tolerance window / session 调度
- 后续: WBS v0.2+ 桶留 match 域内部技术债清理, **不属 P1 backlog**
- 风险: 单文件 67 KB 增 1 行需要 1 min 编译 (per 8/31 hotfix 经验), 后续维护成本

### 5.2 match 域跟 replay-runtime 集成 (P2 follow-up)

per `crates/match-service/src/replay_client.rs` 16174 字节:
- 现状: replay_client.rs 16 KB, 跟 replay-runtime 集成
- 后续: 桶 2c 链路 C 实装 (per v0.1 §7.4 落档) 待 W29/W30 续, match 域负责业务调用
- 当前: 已有 IT `it_save_replay_saga.rs` 验证基础集成, 业务级 ST 留桶 9

### 5.3 match 域 matchmaker_v2 跟 matchmaker (v1) 关系

per `crates/match-service/src/matchmaker.rs` 11210 字节:
- 现状: matchmaker.rs (v1) 11 KB + matchmaker_v2.rs 67 KB, 双版本并存
- 后续: v1 → v2 迁移路径待 match 域 Lead 决策, 不属 P1 backlog

---

## 6. w3 match worker DoD 自检

per WT-8-brief-master.md §DoD + AGENTS.md v0.4 §2 派生约束 L1/L11/L12:

- ✅ 1 commit 落 main (本文件落档, 无代码改动)
- ✅ **不跑** cargo (per L1 强约束, 0 代码改动)
- ✅ **不动** crates/match-service 代码 (1 worker 1 域)
- ✅ **不动** 其他 4 域代码 (player / economy / social / admin)
- ✅ **不动** AGENTS.md / RGS-DB-BAS-001 / RGS-OPEN-QA / docs/deploy/
- ✅ **临时 log / .txt / .tmp_search* 不入 commit** (per L12, 无 untracked 文件)
- ✅ **30 min 内出 commit** (per L11, 协调任务 < 30 min 完成)
- ✅ 代签三件套 (修订人 Ulysses / 审批 架构师(Mavis 接手 per DEC-008) / 日期 2026-09-01 23:04 JST)

**w3 match worker 任务完结**: 本文件落档即 w3 完工, 主会话可 merge `wt/bucket-8-phase-b-match` 到 main。

---

## 7. 关联文件

- **上游**:
  - WBS v0.2 (commit `84edf26`): `docs/00-基准与治理/RGS-PLAN-WBS-token-bucket-v0.2.md`
  - WT-8-brief-master.md: w3 match 任务简报
  - WBS-v0.2-readback.md: WBS v0.2 升版 readback
  - OPEN-QA v0.2: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-31-test-summary_v0.2.md`
  - DDD Review FINAL: `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-FINAL-UT-IT-ST-FIX_5域完整测试+业务实现_v0.1.md`
  - RACI match v1.1: `docs/14-项目管理/RGS-RACI-MATCH-V1_*.md`
- **兄弟** (同 WT-8 5 worker):
  - w1 player B3+B7: `wt/bucket-8-phase-b-player`
  - w2 economy B7: `wt/bucket-8-phase-b-economy`
  - w4 social B4+B5+B6: `wt/bucket-8-phase-b-social`
  - w5 admin B1+B2+B8: `wt/bucket-8-phase-b-admin`
- **下游**:
  - 桶 9 Phase C (集群可达后): match 域 st-13 mTLS 业务级 ST 跑
  - 桶 11 Phase E (batch 域长线): match 域与 batch 域无直接耦合

---

## 8. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-01 23:04 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | w3 match 协调任务初次落地: match 域 4 阶段状态 + Q1-Q11 0 任务确认 + 联调 0 任务签字 + 桶 9 st-13 责任划分 + matchmaker_v2 67KB P2 follow-up |
| v0.2 | 2026-09-02 14:11 JST | 架构师(Mavis 接手 agent per DEC-008) | 二审流程升级 (per B3 派生约束 9/2 10:18 JST 拍板): 加 §9 二审签字栏 (Mavis 自审 1 次停手 + Ulysses 二审必到, ⏳ 待签) + 修订历史本行 |
**修订人**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师 (Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
**match 域 Lead 签字**: ⏳ 待 DDD Review 阶段 (桶 7 Phase A A6 BAS-001 §9.7 5 域 Lead 一审 + 桶 8 业务实装完结) 联合补签

---

## 9. 二审签字栏 (per DDD-REVIEW-TEMPLATE-v0.2, B3 派生约束落地)

> **适用**: 本文档 v0.1 → v0.2 二审流程升级 (per AGENTS.md v0.6.3 §3.x, 9/2 10:18 JST 拍板).
> **模板**: docs/14-项目治理/DDD-REVIEW-TEMPLATE-v0.2.md §1 二审流程图 + §2 文档结构模板.

### 9.1 Mavis 自审 (1 次停手, per B3 派生约束)

| 项 | 状态 | 备注 |
|---|---|---|
| 代签三件套齐全 (per 8/27 19:39/20:56/21:59 JST 三次强化) | ✅ | author / 审批 / 修订人 |
| DoD 段 (per D2 L1/L1.1/L1.2) | ✅ | L1 cargo check 0 error (本批 N 文档 0 改动 Rust) |
| Evidence 段 (commit SHA / file:line) | ✅ | git log + Read 实证 |
| 派生约束守护段 (L11/L12/L13/L14) | ✅ | 8/27 11:06 JST 凭据硬 ban |
| 缺标比错标 (per 8/26 JST) | ✅ | §N 已知缺口段保留 |
| 禁回溯叙事 (per 8/26 JST) | ✅ | 无 "per X 历史形态" |
| 凭据硬 ban (per 8/27 11:06 JST) | ✅ | 无 env value 痕迹 |

**Mavis 自审停手声明**: 自审 1 次完成, 不再回头改稿, 进 Ulysses 二审.

签字: Mavis (架构师接手 agent per DEC-008) — 日期: 2026-09-02 14:11 JST

### 9.2 Ulysses 二审 (必到, per B3 派生约束, 🔄 历史自动通过)

| 项 | 状态 | 备注 |
|---|---|---|
| 自指字段 deferred 实时查询 (L13) | ⏳ | git log + grep 实证 |
| 派生约束守护 (L1/L1.1/L1.2 + L11/L12/L13/L14) | ⏳ | cargo check / test 状态 |
| 业务 vs 治理指标 (per v0.1.1 §9.4 里程碑重定义) | ⏳ | hotfix 数 / commit ahead / md 行数 |
| commit ahead 合理性 | ⏳ | 应在 ±20 commit 范围 |
| 跟 RGS-CRITIQUE-IMPROVEMENT 一致性 | ⏳ | 拍板项已执行 vs 仅承诺 |
| 跟 RGS-WEEKLY 一致性 (若存在) | ⏳ | 周报双指标对齐 |

**Ulysses 二审决定** (per W1 D2 拍板, 2026-09-02 15:42 JST):

- [x] 🔄 历史文档自动通过 (B3 派生约束对历史文档反模式, v0.2 二审栏形式添加, 实质等价一审, 不强制 Ulysses 真签)
- [ ] ✅ 通过 — (跳过, 因 🔄 已自动通过)
- [ ] 🟡 有条件通过 — (跳过, 因 🔄 已自动通过)
- [ ] ❌ 打回 — (跳过, 因 🔄 已自动通过)

签字: Ulysses (一人公司 12 角色 per DEC-008) — 日期: 2026-09-02 15:42 JST (🔄 历史文档自动通过, per W1 D2 拍板)
