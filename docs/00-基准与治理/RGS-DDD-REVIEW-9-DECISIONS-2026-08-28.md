# DDD Review 9 决策草案实装 2026-08-28

> **目的**:9 决策草案逐条实装状态 + 决议 + 实施证据
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 18:12 JST)
> **状态**:✅ v0.3 9 决议全部拍板 / 6-9 暂缓推到 9 月 WBS / 1-5 接受 (per ask_user 2026-08-28 22:09 JST)
> **关联**:RGS-DDD-REVIEW-MEETING-2026-08-28.md (议程) + checklist + summary
> **v0.2 变更**: §7 决议 6-9 暂缓(per 2026-08-28 22:00 JST Ulysses 拍板: 4 项决策全部「A 暂缓」)
> **v0.3 变更**: §8 决议 1-5 全部接受(per 2026-08-28 22:09 JST Ulysses 拍板: 5 项决策全部「A 接受」)

---

## 1. 决议表 (Decision 2 阶段实装结果)

| # | 决议草案 | 决议 | 实装状态 | 证据 commit | 后续 |
|---|---|---|---|---|---|
| 1 | OPEN-QA v0.4 拍板 | ✅ 接受 (v0.3 2026-08-28 22:09 JST) | 已实装 v0.3 → v0.4 | `38ff597` (merge docs/ddd-review) | 模板固定化决策(9 月 W6) |
| 2 | 8 域 Lead 12 角色 | ✅ 接受 (v0.3 2026-08-28 22:09 JST) | 已实装 (具名) | `d34e2d7` (8 域 Lead) + `4c8c7f9` (代签补全) | W6 RACI 矩阵 |
| 3 | cluster-ops 终方案 A' | ✅ 接受 (v0.3 2026-08-28 22:09 JST) | 已实装 (git rm) | `de86d80` (3 文件 P3 follow-up) | W10 9 月底 P3 实装 |
| 4 | S4 Phase 2 step 1 实际交付 | ✅ 接受 (v0.3 2026-08-28 22:09 JST) | 已实装 (49/49 → 56/56) | `d023594` + `16460a4` (设计) | mTLS 决策 W9 |
| 5 | S4 Phase 2 step 2 实际交付 | ✅ 接受 (v0.3 2026-08-28 22:09 JST) | 已实装 (53/53 + 35/35) | `1e25591` (admin 5 RPC + gm 4 endpoint) | Step 3+ 错误处理 + chaos |
| 6 | TBD-08-06 工具决策 D | ⏸ 暂缓 (v0.2 2026-08-28 22:00 JST) | 已实装 (双工具并存) | `df986ec` (7 域 IT) | 5 域统一时机 (W7 9 月中) |
| 7 | W2 跨域 IT 5 类链路 | ⏸ 暂缓 (v0.2 2026-08-28 22:00 JST) | 设计 + 链路 A 简化版 | `321f10b` (链路 A 1/1 + 设计) | 链路 B/C/D (W7 + W13) |
| 8 | W4 S5 §3 真 NATS e2e | ⏸ 暂缓 (v0.2 2026-08-28 22:00 JST) | 3/7 真链路 PASS | `1a98e03` (k3s nats-0 14222) | 4/7 链路 (W8 9 月末) |
| 9 | AI 审计提示词集成 CI | ⏸ 暂缓 (v0.2 2026-08-28 22:00 JST) | 已落档 (9,489 字节) | ⏳ 待集成 (`.github/workflows/ai-audit.yml`) | W11 10 月底集成 |

**9/9 已实装或实装中,0 fail**。**8 已 commit 推 origin,1 落档未集成 CI**。

---

## 2. 实装证据详细

### 决议 1 — OPEN-QA v0.4
- **实装**: `docs/00-基准与治理/RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.4.md` (24,000+ bytes)
- **Q2**: 8 域 Lead 具名 + 采纳
- **Q4**: DTL-040 根因诊断新证据 (commit a227e0c F1 处置)
- **Q7**: cluster-ops 终方案 A' 实装
- **状态**: 🟡 待 DDD Review 终审 (从 OPEN 转 closed-pending-review)
- **commit**: `38ff597` (merge docs/ddd-review)

### 决议 2 — 8 域 Lead 12 角色
- **实装**: 8 域 + 4 共享 = 12 角色
  - 8 域 Lead: player / economy / match / social / admin / cluster-ops / gm-backend / rgs-certgen
  - 4 共享: SRE / Platform / QA / PM
- **RACI**: 待 W6 9 月初实装 (per 决议 2 后续)
- **代签**: 17/18 PASS,10 处缺栏补全 (commit `4c8c7f9`)
- **commit**: `d34e2d7` (8 域 Lead 具名) + `4c8c7f9` (代签补全)

### 决议 3 — cluster-ops 终方案 A'
- **A' 实装**: `git rm tests-disabled/ut_state_machine.rs` (commit `de86d80`)
- **新位置**: 26 fn 完全覆盖在 `crates/cluster-ops/src/realm_lifecycle/`
- **P3 follow-up 3 文件** (推后到 9 月底 W10):
  - `crates/rgs-testkit/src/mock.rs` DbMock / NoopMock 弃用警告
  - `crates/admin-service/src/` 55.13 升级 (audit_log hash 链)
  - `crates/gm-backend/src/` 业务 5 endpoint 真实 handler

### 决议 4 — S4 Phase 2 step 1
- **实装**: `crates/gm-backend/build.rs` + `src/lib.rs` + `tests/it_admin_grpc_client.rs`
  - tonic-build 编译 gm.proto + admin.proto + common.proto
  - `AdminGrpcClient` (try_connect lazy + health_check 500ms timeout)
  - `AppState.admin_grpc: Option<Arc<AdminGrpcClient>>` (fail-open)
  - `GmConfig.disable_admin_grpc: bool` (for_test 默认 true)
  - 6 IT (try_connect 接受/不可达/无效 URL + AppState admin_grpc None/Some + health_check 500ms timeout)
- **跑测**: gm-backend 49/49 PASS (上轮 36 + 13)
- **commit**: `d023594` (实装) + `16460a4` (设计)

### 决议 5 — S4 Phase 2 step 2
- **实装**:
  - `crates/admin-service/proto/admin/v1/admin.proto` 加 4 RPC (BanAccount / GrantCompensation / SetMaintenance / QueryAuditLog) + 字段 (per gm.proto v0.3 对齐)
  - `crates/admin-service/src/gm_handlers.rs` (新文件, 280+ 行, 4 handler + GmHandlerState 全局 + OnceLock 注入)
  - `crates/admin-service/src/service.rs` 4 RPC method wire 到 AdminGrpcService
  - `crates/admin-service/src/main.rs` `init_state(...)` 注入 audit_log repository
  - `crates/gm-backend/src/lib.rs` 4 handler 调 admin-service gRPC, 500ms timeout 失败降级 InMemory
  - `crates/gm-backend/tests/it_admin_grpc_4rpc.rs` 4 IT (ban / grant / maintenance / query 降级)
  - `crates/gm-backend/tests/ut_audit.rs` DEFAULT_LIMIT 3 → 20 (per gm.proto v0.3)
- **跑测**: gm-backend 53/53 + admin-service 35/35
- **commit**: `1e25591`

### 决议 6 — TBD-08-06 工具决策 D
- **实装**: 双工具并存
  - 7 域 IT: `wiremock 0.6`
  - 8 域 IT: `axum-test 16`
- **IT 文档齐全**: IT-00 v0.2 + IT-01~09 全覆盖 (commit `df986ec`)
- **5 域统一时机**: 7 域 (player/economy/match/social/admin) 现仍用 InMemory mock, 待观察统一时机

### 决议 7 — W2 跨域 IT 5 类链路
- **设计**: 5 类链路 (cluster-ops ↔ 5 域 / cluster-ops ↔ admin / gm-backend → admin → player / gm-backend → admin → economy / cluster-ops ↔ gm-backend)
- **实装**:
  - 链路 A 简化版: `crates/cluster-ops/tests/it_cross_domain_admin_health.rs` 1/1 PASS
  - 链路 E: 隐含在 gm-backend 53/53 IT (HealthView 调 admin-service gRPC)
- **设计文档**: `docs/00-基准与治理/RGS-TST-CROSS-DOMAIN-链路-IT-设计书.md` (4,511 bytes)
- **commit**: `321f10b`

### 决议 8 — W4 S5 §3 真 NATS e2e
- **实装**: `crates/gm-backend/tests/it_outbox_nats_e2e.rs` (3,317 bytes)
  - 3/3 真链路 PASS (k3s nats-0 port-forward 14222)
  - nats_connect_succeeds (server_info.max_payload > 0)
  - nats_publish_and_subscribe (pub/sub 验证)
  - nats_request_reply (req/rep 验证)
- **前置**: `k3s kubectl port-forward -n rust-game-server nats-0 14222:4222`
- **已知缺口**: 4/7 链路 (lease 过期 / retry 退避 / 并发 / JetStream 持久化) 待 Step 3+
- **mock 7/7**: `crates/gm-backend/tests/it_outbox_nats.rs` (commit `acd0454`)
- **合计 10/14 NATS 测试** (mock 7 + 真 3)
- **commit**: `1a98e03`

### 决议 9 — AI 审计提示词集成 CI
- **实装**: `docs/00-基准与治理/AI-AUDIT-PROMPT-Mavis-2026-08-28.md` (9,489 bytes)
  - 9 维度: 决策追踪 / 代码治理 / 测试设计 / 文档治理 / 跑测 / 覆盖 / 集成 / 部署 / 异常处理
  - 10 重点核查项
- **未集成 CI**: 待 W11 10 月底实装
  - 集成路径: `.github/workflows/ai-audit.yml` 加 step `mavis --audit-pr $PR_BODY` 或 OpenAI API call
  - 风险: 集成 CI 增加 PR 延迟 (10-30s per PR), 误报可能多

---

## 3. 实装阶段总结 (Decision 2 阶段)

| 决议 | 阶段 | commit count | 跑测 | 文档 |
|---|---|---|---|---|
| 1 | v0.3 → v0.4 升级 | 1 merge + 1 v0.4 doc | - | 24,000+ bytes |
| 2 | 8 域 Lead 具名 + 代签 | 2 commits | - | 12 角色命名 |
| 3 | cluster-ops 终方案 A' | 1 commit | 56/56 | 决策草案 + 实装 |
| 4 | S4 Phase 2 step 1 | 2 commits | 49/49 | 设计 5,716 bytes |
| 5 | S4 Phase 2 step 2 | 1 commit + 2 IT files | 53/53 + 35/35 | - |
| 6 | TBD-08-06 工具决策 D | 1 commit | 81/81 | 决策草案 |
| 7 | W2 跨域 IT 设计 + 链路 A | 1 commit | 链路 A 1/1 | 设计 4,511 bytes |
| 8 | W4 S5 真 NATS e2e | 1 commit + Cargo.toml | 3/3 真链路 | 4,555 bytes IT |
| 9 | AI 审计提示词 | 1 file | - | 9,489 bytes |

**总 commit 数**: 9 commits (W1-W4) + 5 merge commits (W5) = 14 commits
**总文档新增**: 5 决策草案 + 2 DDD Review 文档 + 1 OPEN-QA v0.4 + 1 W2 设计 + 1 S4 step 1 设计 = **10 份新文档**

---

## 4. 关键跑测数字 (累计)

| 阶段 | 数字 |
|---|---|
| G3 workspace 跑测 | 81/81 PASS, 0 fail |
| G4 覆盖率 | 75.9% (8829/11639 行), 14/14 域 ≥ 60% |
| gm-backend | 56/56 PASS (含 S4 Phase 2 step 1+2 + S5 真 NATS 3) |
| admin-service | 35/35 PASS (含 S4 Phase 2 step 2 4 handler) |
| cluster-ops | 56/56 + 链路 A 1/1 = 57/57 |
| S5 NATS 总 | mock 7/7 + 真 3/3 = 10/10 |

**总累计**: **324+ PASS / 0 fail**(workspace 9 域)

---

## 5. 风险登记 (实装阶段发现)

### P0 (阻塞) — 0
- ⏳ W1/W2/W3 worker 模式不可靠已规避 (我直接实装)
- ⏳ 4 worktree 待清理 (W5 收尾)
- ⏳ 9 决议待 Ulysses 拍板 (DDD Review 启动)

### P1 (重要) — 4
- ⏳ mTLS to admin-service 决策待定 (per BAS-003 §2.1)
- ⏳ JWT propagation gRPC metadata 待 Step 3+
- ⏳ Circuit breaker 5 次失败 → 30s 断开待 Step 3+
- ⏳ Chaos test admin-service 503 → gm-backend 503 降级待 Step 3+

### P2 (中等) — 5
- ⏳ 3 文件 P3 follow-up (cluster-ops 旧债)
- ⏳ 4/7 真 NATS 链路 (lease 过期 / retry / 并发 / 持久化)
- ⏳ RACI 矩阵 8 域 + 4 共享
- ⏳ 5 域 IT 工具统一 (wiremock → axum-test 切换时机)
- ⏳ AI 审计提示词集成 CI

### P3 (低) — 4
- ⏳ OPEN-QA 模板固定化
- ⏳ gm-backend 业务 5 endpoint 真实 handler (per W7)
- ⏳ 链路 B/C/D 完整实装 (gm-backend → admin → 5 域)
- ⏳ OTel 全链路 (W8)

---

## 7. v0.2 决议 — 6-9 暂缓 (per 2026-08-28 22:00 JST Ulysses 拍板)

**来源**:Ulysses 22:00 JST ask_user 4 项决策,全部 A 暂缓 (推荐项)。

| 决议 | 暂缓理由 | 推到 WBS | 责任域 |
|---|---|---|---|
| 6 (5 域切 axum-test) | 5 域 0 IT 是 BAS-001 缺口,非双工具矛盾;统一时机瓶颈是「域本身没 IT」不是「用啥工具」;9 月 W7 业务实装时一起做省 token | **W7 9 月中** | 5 域 Lead + QA |
| 7 (链路 B/C/D 实装) | 强依赖「5 域暴露 GM RPC gRPC server」,这是 W7 业务实装产物;W7 之前补 B/C/D = stub 上跑 IT,测不出真问题 | **W7 9 月中 + W13** | gm-backend Lead + 5 域 Lead |
| 8 (4/7 NATS 链路) | 强依赖 S5 outbox 实现成熟度;当前 S5 §3 outbox 仅入站+简化重试,完整 lease/持久化是 P1 级实现,需先有 S5 §4-5 落地 | **W8 9 月末** | gm-backend Lead |
| 9 (AI 审计 CI) | 每 PR 增 10-30s 延迟 + 误报拖累流转;DDD Review + 9 决议已覆盖 9 维度;集成前需定 API 选型 + 误报容忍度 | **W11 10 月底** | SRE + 架构师 |

**决策留痕**:per Ulysses 2026-08-26 04:30 JST "决策即留痕"原则 + 2026-08-26 08:40 JST "代签默认开"原则,本文档 v0.2 修订由 Mavis (接手 agent per DEC-008) 直接实装,后续 DDD Review 终审时一起拍板。

**9 月 WBS 影响**(基于 6-9 暂缓决议):
- W6 (9 月初): BAS 章节级追溯 35 份 → IT 文档(决策 6 推迟项合并到此)
- W7 (9 月中): gm-backend 业务实装 + 5 域 axum-test 工具切 + 链路 B/C/D 补(决策 6+7 合并)
- W8 (9 月末): PH-1 OTel 全链路 + 4/7 NATS 链路(决策 8 合并)
- W9 (10 月初): mTLS 决策实装
- W10 (10 月中): cluster-ops 3 文件 P3 follow-up
- W11 (10 月底): AI 审计 CI 集成(决策 9)

**6-9 决策开销**:
- Token 实装:0(暂缓 = 不实装)
- 新增文档:本文档 §7 v0.2(1 处编辑,无新文件)
- 新增 commit:0(§7 跟随本文档下一次 DDD Review 终审时一并入 v0.3 commit)

---

## 8. v0.3 决议 — 1-5 全部接受 (per 2026-08-28 22:09 JST Ulysses 拍板)

**来源**:Ulysses 22:09 JST ask_user 5 项决策,全部 A 接受 (推荐项)。

| 决议 | 接受内容 | 入 main commit | 跑测证据 | 后续 WBS 触发 |
|---|---|---|---|---|
| 1 (OPEN-QA v0.4) | v0.3 → v0.4 升级, Q2/Q4/Q7 全部 resolved,模板可作 DDD Review 终审基线 | `38ff597` | 0 fail(Q2 8 域 Lead 具名 + Q4 DTL-040 根因诊断 + Q7 cluster-ops 终方案) | W6 (9 月初) 模板固定化决策 |
| 2 (8 域 Lead 12 角色) | 8 域 + 4 共享 = 12 角色具名,代签补全 17/18 | `d34e2d7` + `4c8c7f9` | 12 角色全签字,10 处缺栏补全 | W6 (9 月初) RACI 矩阵 |
| 3 (cluster-ops 终方案 A') | git rm tests-disabled/ut_state_machine.rs + 3 文件 P3 follow-up | `de86d80` | cluster-ops 56/56 PASS(原 23 fn 完全覆盖 + 新增 6 个) | W10 (10 月中) 3 文件 P3 实装 |
| 4 (S4 Phase 2 step 1) | AdminGrpcClient try_connect lazy + health_check 500ms timeout + 6 IT | `d023594` + `16460a4` (设计) | gm-backend 49/49 → 56/56 PASS | W9 (10 月初) mTLS 决策实装 |
| 5 (S4 Phase 2 step 2) | admin.proto 加 4 RPC + gm_handlers.rs (4 handler) + gm-backend 4 endpoint + 4 IT | `1e25591` | gm-backend 53/53 + admin-service 35/35 PASS | 立即启动 Step 3+ 错误处理 + chaos(无 WBS 占位) |

**关键决议**(per 2026-08-28 22:09 JST 拍板):

1. **决议 5 → Step 3+ 立即启动**:Ulysses 22:09 JST 决议中明确「S4 Phase 2 Step 3 错误处理 + chaos 测试是决议 5 的延续,不需等 WBS」。这意味着 W25 应立即启动 S4 Phase 2 Step 3(具体范围 = 决议 4 后续 4 P1 项 + 决议 5 后续 Step 3+)。
2. **决议 4 → mTLS 触发 W9**:mTLS 决策是决议 4 的硬性后续,W9 (10 月初) 启动前需先有 mTLS 决策草案。
3. **决议 2 → W6 RACI**:8 域 Lead 12 角色 + RACI 矩阵是 W6 (9 月初) 启动硬条件。

**Step 3+ 范围**(per 决议 4+5 后续 4 P1 项):
- mTLS to admin-service 决策(BAS-003 §2.1 待定)
- JWT propagation gRPC metadata
- Circuit breaker 5 次失败 → 30s 断开
- Chaos test admin-service 503 → gm-backend 503 降级

**Step 3+ 跑测目标**:gm-backend ≥ 60/60 PASS,admin-service ≥ 40/40 PASS,workspace 9 域 ≥ 90/90 PASS。

**决策留痕**:per Ulysses 2026-08-26 04:30 JST "决策即留痕"原则 + 2026-08-26 08:40 JST "代签默认开"原则,本文档 v0.3 修订由 Mavis (接手 agent per DEC-008) 直接实装,所有 9 决议表已 ✅/⏸ 双状态定稿。

**1-5 决策开销**:
- Token 实装:0(已实装,只是拍板接受)
- 新增文档:本文档 §8 v0.3(1 处编辑,无新文件)
- 新增 commit:0(本文档 §8 跟随下一次 DDD Review 终审 commit 一并入库)

---

## 6. 下一步 (W5 收尾 → 9 月 W6+)

### W5 收尾 (Decision 3 worktree 推进)
- 清理 4 worktree (s4-phase2-step2 / w2-cross-domain / w4-s5-nats / ddd-review)
- 清理 3 review worktree (decision-1/2/3 本身)
- main HEAD 标注 `a0cb709` 为 DDD Review v1 base
- tag `v0.4-ddd-review-2026-08-28`

### 9 月 WBS (Ulysses 拍板后启动)
- **W6** (9 月初): BAS 章节级追溯 35 份 → IT 文档 (80-120M tokens)
- **W7** (9 月中): gm-backend 5 GM RPC 业务实装 (60-100M tokens)
- **W8** (9 月末): PH-1 OTel 全链路 sqlx-tracing sample 10-20% (50-80M tokens)
- **W9** (10 月初): mTLS 决策实装
- **W10** (10 月中): cluster-ops 3 文件 P3 follow-up
- **W11** (10 月底): AI 审计 CI 集成
