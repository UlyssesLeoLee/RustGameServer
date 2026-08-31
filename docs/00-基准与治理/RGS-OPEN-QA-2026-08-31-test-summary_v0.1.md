# RGS-OPEN-QA-2026-08-31-test-summary — 5 域 UT+IT+ST 测试阶段问题汇总

> **文档 ID**: RGS-OPEN-QA-2026-08-31-test-summary
> **版本**: v0.1
> **生效日期**: 2026-08-31 20:10 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008,代签)
> **状态**: 🟡 OPEN (11 项 P1 backlog 待上游 AI 决策 + 6 项工程教训待 DDD Review 复盘)
> **范围**: 2026-08-31 12:09-19:48 JST,5 域 UT + IT + ST 三阶段并行测试,11 commit 落 main (`305f2cb`),+11070 行, 366+ tests, 5/5 cargo check, 10 ST 场景
> **关联**:
> - UT+IT DDD Review: `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-UT-IT_v0.1.md` (commit `bd0884f`)
> - ST DDD Review: `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-ST_v0.1.md` (commit `bd0884f`)
> - 项目级 OPEN-QA 系列: `RGS-OPEN-QA-001_*`, `RGS-OPEN-QA-2026-08-26-SPEC_v0.1.md`, `RGS-OPEN-QA-2026-08-27-k3s-deploy_v0.4.md`

---

## 0. 重要前提

- **本 OPEN-QA 不擅自给"建议方案"**——per 2026-08-26 04:30 JST 派生约束"缺标比错标安全",问题留给负责 Lead / 上游 AI 决策
- **本 OPEN-QA 中所有 commit SHA / file:line 都是 git 实证**(per 2026-08-26 04:30 JST 派生约束"引用必须 git 实证")
- **DDD Review 时 DDL Review + 5 域 Lead 联合审 + Ulysses 终审**(per 2026-08-26 08:40 JST 反转规则)
- **代签规则**: 修订人 = Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手;审批 = 架构师(Mavis 接手 agent per DEC-008) (per 2026-08-27 19:39/20:56/21:59 JST 三次强化)

---

## 1. 问题汇总总览

### 1.1 三阶段产出

| 阶段 | commit | +行 | tests | verdict | cargo check |
|---|---|---:|---:|---|---|
| UT 5 域 | 6 commit | +4057 | 307+ | — | 5/5 PASS |
| IT 5 域 | 5 commit | +5179 | 59 新 IT | — | 5/5 PASS |
| ST 5 域 | 2 commit (cd93169 + d538d9c) | +1834 | 10 场景 | 4 PASS / 6 FAIL | n/a (k3s 探活) |
| **合计** | **13 commit** | **+11070** | **366+ + 10 ST** | — | — |

### 1.2 问题分类(2 大类: 业务 P1 backlog + 工程教训)

- **业务 P1 backlog (11 项)**: 来自 5 域 worker 报告 + ST 阶段真实部署验证
- **工程教训 (6 项)**: 来自 4 阶段迭代失败模式(UT v1/v2/v3 + ST 5 worker 0 产出)

---

## 2. 业务 P1 backlog (11 项,需上游 AI / 5 域 Lead 决策)

### Q1. 🔴 P1-01: admin gm_handlers 缺 RBAC check (COCRoleRequired)

**问题描述**:
- 业务影响: 普通 GM / 玩家调用 GM 指令应被拒绝, 但 `gm_handlers` handler 入口未做应用层权限拦截
- 来源: UT worker `04a9838` 报告 §已知风险 + IT worker `67f82d6` 报告 §已知风险
- 证据: UT + IT 都在测试层用 `issue_gm_command_with_rbac` wrapper 显式模拟, 但生产代码本身应补
- 关联标准: RGS-ARC-051 §COC + 8/27 21:59 JST DDD Review 待办
- 严重性: 🔴 高 (安全漏洞)

**决策项**:
- [ ] 哪个桶 (bucket) 落地? 建议 `gm_handlers.rs` 加 COC middleware
- [ ] handler 入口还是 trait 层?
- [ ] 测试覆盖到什么粒度? (UT 单元 / IT 集成 / ST 端到端)

**证据 commit**:
- UT 报告: `8650a57` (admin 域 feat(test)) + `04a9838` (fix(test) 18 errors → 0)
- IT 报告: `67f82d6` (admin 域 feat(test) IT 3 文件)
- 5 域 merge: `103481a` (admin 域 merge)

---

### Q2. 🔴 P1-02: admin audit_log 缺 startup verify (tamper detection)

**问题描述**:
- 业务影响: 启动 reload 时未逐条 recompute hash, 篡改的审计记录无法被检测
- 来源: UT worker `04a9838` 报告 + IT worker `67f82d6` 报告
- 证据: IT 中 `tampered_audit_entry_fails_hash_recomputation` 测试用 snapshot 篡改 + payload 保留方式**间接证明** hash 链能检测, 但业务侧 startup verify 流程未实化
- 关联标准: RGS-SEC-100 §7 startup check 待办
- 严重性: 🔴 高 (审计完整性)

**决策项**:
- [ ] 启动时全表 recompute 性能? (50 条 / 5K 条 / 50K 条)
- [ ] 启动失败处理? (panic / fail-closed / 警告继续)
- [ ] 增量 verify 还是全量?

**证据 commit**:
- IT 报告: `67f82d6` 含 `integration_audit_log_chain_under_restart.rs` 3 tests

---

### Q3. 🟡 P1-03: player update_player_profile 占位未强制 wins ≤ total_matches

**问题描述**:
- 业务影响: 业务层**未**强制 `total_wins ≤ total_matches` 约束, 玩家可被刷为"全胜但 0 场"
- 来源: IT worker `bd83fb3` 报告 §已知风险
- 证据: `service.update_player_profile` 当前是占位实现 (per DTL-038 §7.2 TODO), IT 已在 `assert_wins_leq_total` helper 显式声明不变量, 但实际约束强制化是 P1 backlog
- 严重性: 🟡 中 (业务逻辑, 非安全)

**决策项**:
- [ ] 桶 (DTL-038 §7.2) 哪个阶段实现? (PH-1 / PH-2 / PH-3)
- [ ] 数据库层 CHECK 约束 vs 业务层 invariant?
- [ ] 已有 5 域 worker IT 测试是否保留? (bd83fb3 已落档)

**证据 commit**:
- IT 报告: `bd83fb3` 含 `integration_player_profile_update_chain.rs` 4 tests

---

### Q4. 🟡 P1-04: economy integration_outbox L143 缺 graceful skip

**问题描述**:
- 业务影响: `outbox_check_constraint_is_idempotent` test 缺 graceful skip (L143 `.expect("DATABASE_URL must be set")` 会 panic), 与同文件第一个 test 的 skip 风格不一致
- 来源: IT worker `afd3d65` 报告 §已知风险
- 证据: pre-existing (per commit 0623066 + 2396941), 不在本次 IT 范围
- 严重性: 🟡 中 (测试代码, 非生产)

**决策项**:
- [ ] 修这一个 expect? 还是统一 outbox 测试 skip 风格?
- [ ] 是否纳入 P1 backlog (per 8/26 JST "缺标比错标安全" — 显式列出是 lower priority)?

**证据 commit**:
- IT 报告: `afd3d65` 含 `integration_outbox_atomicity.rs` 4 tests
- pre-existing: `0623066` + `2396941`

---

### Q5. 🟢 P1-05: social guild capacity 硬上限 50 (简报假设 64)

**问题描述**:
- 业务影响: 实际硬上限是 `if guild.member_count >= 50`(注释"简单限制:50 人"), 简报假设 64
- 来源: IT worker `3f41626` 报告 §已知风险
- 证据: `src/service.rs` 硬代码 50, IT 按 50 验证
- 严重性: 🟢 低 (业务确认即可, 非安全)

**决策项**:
- [ ] 50 vs 64 哪个对? 业务确认
- [ ] 50 是临时限制还是最终?
- [ ] 如果改 64, 现有 50 边界测试要调 (per `integration_guild_capacity_boundary.rs`)

**证据 commit**:
- IT 报告: `3f41626` 含 `integration_guild_capacity_boundary.rs` 3 tests

---

### Q6. 🟡 P1-06: social leave_guild 业务方法缺失

**问题描述**:
- 业务影响: 玩家无法退出公会, IT 用 `InMemoryGuildMemberRepository.delete_by_id` + `InMemoryGuildRepository.save` 模拟 leave
- 来源: IT worker `3f41626` 报告 §已知风险
- 证据: src/ **无 leave_guild 业务方法**, mock_leave() 在测试里手写
- 严重性: 🟡 中 (业务 API 缺失, 玩家被锁在公会里)

**决策项**:
- [ ] 哪个桶 (PH-6 社交) 补 leave_guild API?
- [ ] leader 退出时 leadership 转移规则?
- [ ] 离开后 player.profile 字段清理?

**证据 commit**:
- IT 报告: `3f41626` 含 `integration_guild_lifecycle.rs` 3 tests

---

### Q7. 🟡 P1-07: social push_delivery 缺真实 dispatcher

**问题描述**:
- 业务影响: src/push_delivery.rs 仅提供数据 + sanitize, 无真实 dispatcher (与 NATS / APNs / FCM 集成)
- 来源: IT worker `3f41626` 报告 §已知风险
- 证据: IT 端定义 test-only MockPushDispatcher (Pending/Delivered/FailedRetryable/FailedPermanent + attempts 计数)
- 严重性: 🟡 中 (业务功能未端到端)

**决策项**:
- [ ] dispatcher 走 NATS 还是直接 HTTP (FCM/APNs webhook)?
- [ ] retry 策略? (max attempts / backoff)
- [ ] 死信队列 (DLQ)?

**证据 commit**:
- IT 报告: `3f41626` 含 `integration_push_delivery_atomicity.rs` 3 tests

---

### Q8. 🔴 P1-08: ST gm-backend 8081 /healthz + /readyz 不响应

**问题描述**:
- 业务影响: k8s exec 探针 (grpc_health_probe + mTLS) 失败, ST 6 个场景因此 FAIL
- 来源: ST worker (main session) `cd93169` 报告 + e2e-smoke.ps1 baseline 12 probe (7 PASS / 5 FAIL)
- 证据: `curl http=000000 exit=28` (timeout), 容器在跑 (PORT 探活 PASS) 但 HTTP 不响应
- 关联标准: k3s gm-backend Deployment manifest (per `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml`)
- 严重性: 🔴 高 (k8s 探针失效, 6 个 ST 场景 FAIL 根因)

**决策项**:
- [ ] 容器内 `curl localhost:8081/healthz` 诊断? (需 kubectl exec)
- [ ] gm-backend binary startup 失败? (看 container log)
- [ ] 容器 OOM? (memory limit)
- [ ] 重启容器 / 重 build image?

**证据 commit**:
- ST 报告: `cd93169` + `d538d9c` (player 域 evidence 刷新)
- ST merge: `305f2cb`
- e2e-smoke baseline: `scripts/e2e-smoke.ps1` (per 8/27 部署成果)

---

### Q9. 🟡 P1-09: ST prometheus + grafana HTTP 探活 000000

**问题描述**:
- 业务影响: prometheus + grafana HTTP 探活 000000, ST 4 个相关 probe FAIL
- 来源: e2e-smoke baseline + ST 阶段
- 证据: 容器 PORT 探活可达, 但 HTTP endpoint 不响应
- 严重性: 🟡 中 (监控可观测性受损)

**决策项**:
- [ ] 容器内 prometheus 进程在跑?
- [ ] grafana admin password 改了? (per 8/22 部署)
- [ ] prometheus config reload 失败?

**证据 commit**:
- ST 报告: `cd93169` (verdict 矩阵 st-01, st-05, st-07, st-09, st-10 FAIL 根因)
- e2e-smoke: 12 probe baseline

---

### Q10. 🟡 P1-10: ST 5 域 mTLS 业务级 ST 缺失

**问题描述**:
- 业务影响: 5 域 gRPC 50051-50055 port 探活全 PASS, 但 mTLS 业务调用没验证
- 来源: ST 阶段 v2 决策摸底 (per 8/31 17:12 JST)
- 证据: 5 域 mTLS 证书在 k8s secret (`docs/deploy/01-k8s-manifests/50-secret-{player,economy,match,social,admin}-tls.yaml`), 本地 ST worktree 拉不到
- 严重性: 🟡 中 (业务级 E2E 缺失, 仅基础设施层验证)

**决策项**:
- [ ] mTLS 证书从 k8s secret 导出到 ST worktree? (`kubectl get secret player-tls -o yaml > certs/`)
- [ ] grpcurl + mTLS 业务调用? 哪个工具链 (grpcurl / 自写 Rust client / Postman gRPC)?
- [ ] trade saga 端到端 (跨 economy+match+admin) 何时落?
- [ ] replay 端到端 (跨 match+admin) 何时落?

**证据 commit**:
- ST 报告: `cd93169` (ST 范围 = 基础设施层 + gm-backend APIGW 层)
- mTLS k8s secret: `docs/deploy/01-k8s-manifests/50-secret-*-tls.yaml`
- 证书生成 SOP: `docs/deploy/00-prerequisites/phase-0-5-step-4-gen-certs.ps1`

---

### Q11. 🟢 P1-11: ST NATS 8222 部署范围

**问题描述**:
- 业务影响: NATS 8222 探活 000000, 8/27 部署可能不含
- 来源: ST 阶段
- 证据: e2e-smoke baseline 12 probe, NATS 列在 PROBES 但可能未部署
- 严重性: 🟢 低 (per 8/27 部署范围确认)

**决策项**:
- [ ] 8/27 部署是否含 NATS? 查 `kubectl get pods -n rust-game-server -l app.kubernetes.io/name=nats`
- [ ] 含 → 修启动 (per Q9 类似)
- [ ] 不含 → 标 SKIP, 不算 P1

**证据 commit**:
- ST 报告: `cd93169` (st-08 social-cross-domain-push 含 NATS probe, 已 SKIP accept)

---

## 3. 工程教训 (6 项,需 DDD Review 复盘 → AGENTS.md / 文档化)

### L1. 🔴 UT v1 worker cargo test polling 循环失败

**问题**:
- 5 worker 跑 `cargo test -p <domain>-service` 长编译 (5-15 min) → 陷入 `Start-Sleep + Get-Process cargo` 计数循环
- 5 worker 里 1 succeeded 但 0 产出 (working tree 改动但没 commit), 4 个被我主动 stop
- 根因: worker 不知道怎么处理长编译, 默认 fallback 到 polling

**教训**:
- **worker 任务里禁止"必须 cargo test 通过"**, 改 "必须 cargo check 通过 (快, 几秒) + 最终 cargo test 在主会话统一跑"
- **polling 是反 pattern**, worker 应在启动编译时直接返回, 等任务完成信号

**证据**:
- UT v1 报告: `bg_7f4613b2` succeeded but 0 output
- 4 worker: `bg_dae4ac05` `bg_bd325ab2` `bg_617daef4` `bg_113d1c11` 全部 cancelled by me
- reflog 证据: `D:/rgs-ut-player` HEAD reflog 显示 reset → 3d31f53 → 3cfeedb (worker 后来 commit)

**决策项**:
- [ ] 文档化到 `AGENTS.md` "Worker cargo 长编译反 pattern"
- [ ] 未来 worker 简报模板禁用 "cargo test" / "cargo build", 改 "cargo check --tests"

---

### L2. 🔴 UT v2 禁 cargo 导致 38 编译错误

**问题**:
- 我把"严格禁止 cargo"推到极端, worker 写的代码没编译过就 commit
- 类型 move / proptest Result / trait 引用等基础错误都没暴露
- 4 域 38 errors 需要二次 hotfix 修复 (v3)

**教训**:
- **worker 必须跑 `cargo check` 至少 1 次**, 不能跳过验证直接 commit
- "缺标比错标安全" (per 8/26 JST 派生约束) → 应用到 worker 工作流: 必须有编译验证

**证据**:
- v2 报告: 4 worker 提交, `cargo check` 暴露 38 errors
- v3 hotfix: `04a9838` (admin 18 errors) + `1db3249` (economy 14 errors) + `3e456b4` (social 6 errors)

**决策项**:
- [ ] 文档化到 `AGENTS.md` "Worker cargo check 必跑"
- [ ] 未来简报: "✅ 必跑 `cargo check --tests` (限时 60s)" 列为强约束

---

### L3. 🟡 ST 阶段 rgs-testkit 强约束 vs mock 决策冲突

**问题**:
- rgs-testkit 强约束 "禁 InMemory mock" + workspace 无 axum/hyper → "新 mock server binary" 决策不可调和
- 必须改走 k3s 真实部署 (第二轮决策)
- 浪费时间 ~20 min

**教训**:
- **跨工具链决策前要先查 workspace 依赖**, 不能再拍脑袋
- **rgs-testkit 是项目级强约束** (per 8/30 JST 落档 `pg_pool` + `pg_test`), mock server 不应绕开

**证据**:
- `crates/rgs-testkit/src/lib.rs` L17-25 (强约束 §"唯一接受的 API")
- `crates/rgs-testkit/src/lib.rs` L30-34 (拒绝的 API: `mock::DbMock` / `InMemoryAccountRepository`)
- workspace Cargo.toml: 无 axum/hyper/warp/actix-web

**决策项**:
- [ ] 文档化 rgs-testkit 强约束到 `AGENTS.md`
- [ ] 未来 mock server 决策先 grep workspace 依赖 + 看 rgs-testkit 强约束段

---

### L4. 🟡 ST 阶段 5 worker 0 产出 → 主会话自写

**问题**:
- 5 worker (player / economy / match / social / admin) 在 ST 阶段全部 0 产出
- 跟 UT v1 player 同症: 跨多工具链 (WSL + sudo + k3s + 5 域 + e2e-smoke) worker 复现成本太高
- 正确做法: 主会话先写 1 个完整 ST 脚本跑通链路, 再让 worker 复用模板

**教训**:
- **跨多工具链场景 (WSL + sudo + k3s + 5 域) 不要直接派 worker**, 应主会话先打头阵
- **worker 适合做模板化工作** (复制 + 改 probe 列表), 不适合做从 0 探索

**证据**:
- ST v2 5 worker 报告: 5 个 `bg_*` 全部 succeeded but 0 commit
- 主会话自写: `cd93169` (40 files, 10 场景) — 证明主会话可成

**决策项**:
- [ ] 文档化到 `AGENTS.md` "跨工具链场景先主会话打头阵"
- [ ] 未来简报模板: 跨 WSL / Docker / k3s / 多 binary 场景默认主会话先跑 1 个

---

### L5. 🟡 mTLS 证书在 k8s secret 拉不到

**问题**:
- 5 域 mTLS 业务调用需要证书, 证书在 `50-secret-*-tls.yaml` k8s secret
- ST worktree 拉不到 (kubeconfig 权限 per 8/27 11:06 JST 强化)
- 4h 预算内没时间导证书 + 写 5 域业务级 ST

**教训**:
- **ST 阶段路径选择前要先 grep k8s secret 位置 + 证书导出 SOP**
- **5 域 mTLS 业务级 ST 是下轮工作**, 本轮 ST 范围 = 基础设施层 + gm-backend APIGW 层

**证据**:
- mTLS k8s secret: `docs/deploy/01-k8s-manifests/50-secret-{player,economy,match,social,admin}-tls.yaml`
- 证书生成 SOP: `docs/deploy/00-prerequisites/phase-0-5-step-4-gen-certs.ps1`
- ST 报告: `cd93169` §"下轮 ST 升级路径" 写明 mTLS 缺失

**决策项**:
- [ ] 下轮 ST 前先导出 mTLS 证书到 ST worktree
- [ ] 文档化: "ST worktree 必备 5 域 mTLS 证书" 列入 ST 启动 checklist

---

### L6. 🟡 gm-backend k3s 容器 HTTP 不响应 (运维, 非测试)

**问题**:
- gm-backend 8081 /healthz + /readyz 返回 000000 (curl timeout)
- 容器在跑 (PORT 探活 PASS) 但 HTTP endpoint 不响应
- 6 个 ST 场景因此 FAIL
- 根因不在测试代码, 在 k3s 容器 / binary startup

**教训**:
- **ST 阶段 FAIL 不一定归测试责任**, 可能是 k3s 部署问题
- **"ST 失败先看 e2e-smoke baseline"** 是 sanity check: 12 probe baseline 7/5 是基线, ST FAIL 不一定新增

**证据**:
- ST 报告: `cd93169` (6/10 FAIL 全部因 gm-backend 8081)
- e2e-smoke baseline: 7 PASS / 5 FAIL (含 gm-backend + prometheus + grafana + nats)
- 5 域 gRPC + postgres 全 PASS (12 probe 的 7 个)

**决策项**:
- [ ] 派 1 个 worker 修 gm-backend 容器 (kubectl exec 诊断 + 重启)
- [ ] 重跑 ST 10 场景, 期望 10/10 PASS (gm-backend 修好后)

---

## 4. 关联文档与证据汇总

| 文档 | 路径 | commit | 关联 |
|---|---|---|---|
| UT+IT DDD Review | `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-UT-IT_v0.1.md` | `bd0884f` | §6 P1 决策表 (6 项) |
| ST DDD Review | `docs/14-项目管理/ddd-review/RGS-DDD-2026-08-31-ST_v0.1.md` | `bd0884f` | §7 P1 决策表 (5 项) |
| UT worker 报告 | `D:/rgs-ut-{player,economy,social,admin,match}` (5 worktree) | ut/<domain> head | L1-L2 来源 |
| IT worker 报告 | 同上 | ut/<domain> head | P1-01~P1-07 来源 |
| ST worker 报告 | `D:/rgs-st-mock` | `d538d9c` (player) + main self-write | P1-08~P1-11 来源 |
| e2e-smoke baseline | `scripts/e2e-smoke.ps1` + `scripts/e2e-smoke.sh` | 8/27 JST 部署 | 12 probe baseline 7/5 |
| rgs-testkit 强约束 | `crates/rgs-testkit/src/lib.rs` L17-25 | 落档 | L3 来源 |
| mTLS 证书 | `docs/deploy/01-k8s-manifests/50-secret-*-tls.yaml` | 8/27 JST 部署 | L5 来源 |
| 5 域独立 Lead RACI | `docs/14-项目管理/RGS-RACI-{PLAYER,ECONOMY,MATCH,SOCIAL,ADMIN}-V1_*.md` | 8/27 JST 落档 | 责任矩阵 |

---

## 5. 修订历史

| 版本 | 日期 | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-31 20:10 JST | 架构师(Mavis 接手 agent per DEC-008) | 初始创建, 11 项 P1 backlog + 6 项工程教训汇总, 待上游 AI 决策 |

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)

---

## 6. 接力说明 (给上游 AI)

本 OPEN-QA 是 Mavis 8/31 12:09-20:00 JST 的测试阶段总结。**所有 Q1-Q11 + L1-L6 都需要上游 AI / 5 域 Lead / Ulysses 决策**,Mavis 不擅自给方案(per 8/26 JST "缺标比错标安全")。

**建议接力顺序**:
1. **Q1-Q2 (admin P1, 🔴 高)**: 先看 8/21 JST 5 域独立 Lead 架构里 admin 域 Lead 的优先级
2. **Q8 (gm-backend ST 失败, 🔴 高)**: 看 k3s 部署运维, gm-backend 容器为何不响应
3. **Q3-Q7 (业务 P1, 🟡 中)**: 5 域 Lead 各自消化
4. **L1-L6 (工程教训)**: 文档化到 AGENTS.md, 影响未来 worker 工作流
5. **Q9-Q11 (运维/ST)**: 与 k3s 部署组同步
6. **Q10 (mTLS ST 缺失)**: 下轮 ST 启动 checklist 必备

**所有 commit / file:line / 8.x JST 决策时间 都是 git 实证**,可独立验证。
