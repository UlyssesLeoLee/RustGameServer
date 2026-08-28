# RGS-TST-CROSS-DOMAIN 跨域 IT 链路用例设计书

> **目的**:实装跨域 IT 链路用例,覆盖 cluster-ops ↔ 5 域 ↔ admin-service ↔ gm-backend 端到端
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 17:30 JST)
> **状态**:⏳ OPEN (W2 部分实装,3/5 链路)
> **关联**:RGS-OPEN-QA v0.4 + 9 决策草案 + 8 commits (c5c9f5f → 16460a4)

---

## 1. 范围 (5 类跨域链路)

### 链路 A: cluster-ops ↔ 5 域
- 用途: cluster-ops 调 5 域 (player/economy/match/social/admin) health + config
- 实装难度: 中 (需 5 域都有 tonic gRPC client)
- 实装状态: ⏳ **链路 A 简化版** (gm_backend tests/it_admin_grpc_client.rs 链路 A 简化版已实装 2026-08-28 d023594)

### 链路 B: cluster-ops ↔ admin-service
- 用途: cluster-ops 推送 config / 限流策略
- 实装难度: 中 (cluster-ops → admin-service gRPC)
- 实装状态: ⏳ pending

### 链路 C: gm-backend → admin-service → player-service
- 用途: BanAccount 链路, 5 域 handler 实际封禁玩家
- 实装难度: 高 (gm-backend → admin-service 已通, admin-service → player-service 需 player 暴露 gRPC)
- 实装状态: ⏳ pending (Step 2 已实装 gm-backend → admin-service, player-service gRPC 待 Step 3+)

### 链路 D: gm-backend → admin-service → economy-service
- 用途: GrantCompensation 链路, 5 域 handler 实际发放补偿
- 实装难度: 高 (同 C)
- 实装状态: ⏳ pending (同 C)

### 链路 E: cluster-ops ↔ gm-backend
- 用途: 健康视图聚合, cluster-ops 调 gm-backend /api/v1/gm/health/view, 验返 5 域 health
- 实装难度: 中 (gm-backend HealthView 已调 admin-service gRPC, cluster-ops → gm-backend HTTP)
- 实装状态: ⏳ **链路 E 简化版** (gm-backend 49/49 + 53/53 IT 已覆盖 HealthView 行为)

---

## 2. 实装状态 (W2 部分实装)

### 链路 A 简化版 (gm-backend 端)
- 文档: `crates/gm-backend/tests/it_admin_grpc_client.rs` (6 IT)
- 关联 commit: `d023594` (S4 Phase 2 step 1)

### 链路 E 简化版 (gm-backend 端)
- 文档: `crates/gm-backend/tests/integration_gm_basic.rs` (12 IT) + `crates/gm-backend/tests/it_admin_grpc_4rpc.rs` (4 IT)
- 关联 commit: `d023594` + `1e25591` (S4 Phase 2 step 2)

### 链路 B/C/D (跨域) — ⏳ 待 Step 3+
- 链路 B: cluster-ops → admin-service gRPC 限流/配置推送
- 链路 C: gm-backend → admin-service → player-service BanAccount 真实执行
- 链路 D: gm-backend → admin-service → economy-service Compensation 真实执行

链路 B/C/D 需 admin-service 调 5 域 gRPC (现 admin-service 只暴露 HealthCheck + GetAdminOp + 4 GM RPC), 5 域需暴露 gRPC server (现 5 域都跑 tonic server 但只服务 cluster-ops 内部)

---

## 3. 关键设计决策

### 3.1 mock vs 真 PG
- 60% 真 PG (rgs-testkit pg_test fixture)
- 40% mock (axum-test 内部, 不依赖 gRPC)
- 失败降级: 链路不可达时仍跑测试, 验证降级路径

### 3.2 IT 入口
- 在 `docs/00-基准与治理/G3-G4-it-main-stage-runbook.md` 加 §5 "W2 跨域 IT 入口"
- 跑测命令: `source scripts/db-url.sh postgres-superuser 15432 && cargo test --test it_cross_domain_* --no-fail-fast`

---

## 4. 已知缺口

- ⏳ 链路 B 需 cluster-ops 调 admin-service gRPC (新 RPC 待加)
- ⏳ 链路 C/D 需 admin-service 调 5 域 gRPC (player/economy gRPC client 待加)
- ⏳ 链路 C/D 需 5 域暴露 BanAccount / GrantCompensation gRPC (现 5 域不暴露 GM RPC)
- ⏳ 5 域 wiremock/axum-test 切换决策: per 8/27 21:00 已采纳双工具并存, 7 域 wiremock + 8 域 axum-test

---

## 5. 下一步 (W2 完整实装 + Step 3+ 业务)

### W2 完整实装
- 链路 A: cluster-ops → 5 域 health (cluster-ops 测 6 测试: 5 域 + 聚合)
- 链路 B: cluster-ops → admin-service 限流配置推送 (cluster-ops 测 1 测试)
- 链路 E: cluster-ops → gm-backend /health/view 聚合 (cluster-ops 测 1 测试)
- 链路 C/D 暂用 mock (5 域 gRPC 客户端待 Step 3+)

### Step 3+ 业务
- admin-service 加 PlayerService / EconomyService gRPC client
- player-service / economy-service 暴露 BanAccount / GrantCompensation gRPC
- gm-backend 5 endpoint 真实业务执行 (现 仍 stub 写 InMemory)

---

## 6. 参考

- **RGS-TBD-08-03** S4 立项 v0.2 + 6 天工作量分解
- **RGS-S4-PHASE2-STEP1-设计.md** + 实际 commit d023594 / 1e25591
- **G3-G4-it-main-stage-runbook.md** 主阶段入口
- **gm.proto v0.3** (commit c5c9f5f) 5 endpoint 协议
- **admin.proto v0.4** (commit 1e25591) 6 RPC (HealthCheck + GetAdminOp + 4 GM)
