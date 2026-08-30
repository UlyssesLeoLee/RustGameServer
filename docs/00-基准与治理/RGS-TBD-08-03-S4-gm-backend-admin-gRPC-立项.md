# RGS-TBD-08-03 S4 立项 — gm-backend → admin-service gRPC 真实集成

> **目的**:实装 gm-backend 5 个 GM endpoint → admin-service gRPC 真实调用(替代 v0.2 AuditStore trait 抽象 + InMemory)
> **作者**:Mavis(接手 agent per DEC-008,2026-08-28 12:50 JST)
> **状态**:🟡 OPEN(立项完成,实施按 5 阶段推进,预计 1 周)
> **关联**:TBD-08-03 v0.2 抽象(per commit `ec0f11a`)+ gm-backend proto schema 立项(per `crates/gm-backend/proto/gm/v1/gm.proto`)

---

## 0. 背景

### 0.1 当前状态(v0.2, per commit `ec0f11a`)

gm-backend 5 个 GM endpoint 当前是 **stub 状态**:
- `health_view` 返回 `{service, admin_endpoint, mode}` 扁平对象,无 `services[]` 数组
- `set_maintenance` 返回 `{status, op}`,无 `propagation_status` 字段
- `query_audit` 返回 `{items, next}`,字段名 ≠ DTL-003 协议
- `ban_account` / `grant_compensation` 写 `AuditStore::append()`(InMemory 实现)
- 5 endpoint 没有调 admin-service gRPC,仅本地 stub

### 0.2 真实集成需要做的工作

1. gm-backend → admin-service gRPC client 集成(5 endpoint)
2. proto schema 完整(已完成 `gm.proto` v0.3,含 `services[]` / `propagation_status` / `entries+has_more` 字段)
3. TonicGrpcClient 替换 AuditStore trait 默认实现
4. 错误处理:admin-service 不可达 → gm-backend 返回 503 / 业务降级
5. metrics + tracing 集成

### 0.3 立项时间窗(per IT 主阶段排期)

| 阶段 | 工作量 | 累计 |
|---|---|---|
| 立项(本批) | 0.5 天 | 0.5 天 |
| Phase 1 proto schema + 编译 | 1 天 | 1.5 天 |
| Phase 2 gRPC client 集成 | 2 天 | 3.5 天 |
| Phase 3 错误处理 + 业务降级 | 1 天 | 4.5 天 |
| Phase 4 集成测试 + chaos | 1 天 | 5.5 天 |
| Phase 5 evidence 落档 + 文档同步 | 0.5 天 | 6 天 |
| **合计** | **6 天** | **6 天** |

## 1. 工作分解

### Phase 1: proto schema + 编译(1 天)

**目标**:`crates/gm-backend/proto/gm/v1/gm.proto` 编译通过,生成 Rust 代码

**任务**:
- [ ] proto 验证: `protoc gm.proto --rust_out=.` 编译通过
- [ ] 引入 `tonic-build` 0.12 + `prost` 0.13(已 dev-deps)
- [ ] `gm-backend/build.rs` 加 proto 编译入口
- [ ] `gm-backend/Cargo.toml` `tonic` feature 检查(`transport` + `tls` + `prost`)
- [ ] 编译 `cargo build -p gm-backend` 0 error

**验收**:`gm_backend::proto::gm::v1::*` 类型可用,5 个 endpoint trait 编译生成

### Phase 2: gRPC client 集成(2 天)

**目标**:5 个 GM endpoint 调 admin-service gRPC,字段对齐 DTL-003 §3

**任务**:
- [ ] `AdminServiceClient` 注入(per `gm_config.admin_grpc_endpoint`)
- [ ] 替换 `AuditStore::append()` 默认实现为 gRPC `BanAccount` / `GrantCompensation` 调用
- [ ] 替换 `query_audit` stub 为 `QueryAuditLog` gRPC 调用,返回 `entries[] + has_more`
- [ ] 替换 `set_maintenance` stub 为 `SetMaintenance` gRPC 调用,加 `propagation_status`
- [ ] 替换 `health_view` stub 为 `HealthView` gRPC 调用,加 `services[]` 5 子字段
- [ ] JWT propagation:gm_config.jwt_secret 签发 JWT 注入 gRPC metadata

**验收**:5 endpoint 真实调到 admin-service,字段对齐 DTL-003 §3.1-§3.4

### Phase 3: 错误处理 + 业务降级(1 天)

**目标**:admin-service 不可达时,gm-backend 优雅降级,返回明确错误码

**任务**:
- [ ] tonic Status → HTTP status 映射(UNAVAILABLE → 503,DEADLINE_EXCEEDED → 504)
- [ ] retry 策略:首次失败 100ms,再次 200ms,3 次后 fail
- [ ] circuit breaker:连续 5 次失败 → 半开(30s 后重试)
- [ ] audit_log 失败本地 fallback:写 InMemory AuditStore,后台异步刷 admin-service
- [ ] health endpoint 反映 admin-service 状态(纳入 `services[]`)

**验收**:`it_drill_admin_service_unreachable` chaos 测试通过(5 域对称)

### Phase 4: 集成测试 + chaos(1 天)

**目标**:5 endpoint 集成测试覆盖 + chaos 注入

**任务**:
- [ ] `crates/gm-backend/tests/it_gm_admin_grpc.rs` 5 endpoint 真接测试
- [ ] mock admin-service 用 `rgs_testkit::TonicGrpcMock`(已有,54.x 实装)
- [ ] chaos:admin-service 不可达 + 慢响应 + 5xx
- [ ] audit_log 重放测试
- [ ] 跨域 fixture(CI service container 自动注入)

**验收**:`cargo test -p gm-backend` 仍全 PASS(原 36 + 新增 ~15 IT = ~50)

### Phase 5: evidence 落档 + 文档同步(0.5 天)

**目标**:S4 完结,所有文档同步

**任务**:
- [ ] `evidence 2026-08-28-S4` batch 落档
- [ ] `RGS-TST-IT-08` 文档 v0.2 升级(per `crates/gm-backend/proto/gm/v1/gm.proto` 字段级协议)
- [ ] `RGS-TST-UT-08` 文档 v0.3 升级(模块 D 字段级实装完成)
- [ ] `RGS-OPEN-QA` 关闭 TBD-08-03
- [ ] `docs/00-基准与治理/.test-evidence/` 留档

**验收**:TBD-08-03 关闭,gm-backend 90% → 100%

## 2. 风险与缓解

| 风险 | 概率 | 缓解 |
|---|---|---|
| proto schema 与 admin-service 现有 admin.proto 不兼容 | 中 | v0.3 阶段 gm.proto 自包含,不调 admin.proto,等 v0.4 再统一 |
| admin-service 不可达时性能 | 高 | Phase 3 circuit breaker + audit_log 本地 fallback |
| JWT metadata 传递兼容 | 中 | Phase 2 用 `tonic::Request::metadata_mut()` 注入 |
| tonic 0.12 vs 0.13 API 不兼容 | 低 | 跟现有 tonic workspace 锁版本 |
| 测试覆盖缺口 | 中 | Phase 4 chaos 注入覆盖 |

## 3. 决策项

- [ ] S4 时间窗 1 周是否可接受
- [ ] Phase 1 proto schema 先行 commit 立即合 v0.3
- [ ] Phase 3 circuit breaker 阈值(100ms / 200ms / 30s)是否合理
- [ ] audit_log 失败本地 fallback 是否引入(影响一致性)

## 4. 待 Ulysses 一审

- [ ] 1 周时间窗是否可接受
- [ ] S4 优先级 vs S2 / S5(per IT 主阶段排期)
- [ ] 是否拆 v0.3 立项 commit(本批)+ v0.4 实施 commit(分批)
- [ ] gm-backend 5 endpoint 字段级协议(per `gm.proto`)是否需 DDD Review 先行审

---

**关联 commit**:
- `ec0f11a` TBD-08-01~05/07 + UT-08 模块 D 字段级 v0.2(per F8)
- 本批:proto schema 立项(`crates/gm-backend/proto/gm/v1/gm.proto`)
- v0.3 实施 commit:预计 2026-09-04 完成

**作者**:Mavis(接手 agent per DEC-008,2026-08-28 12:50 JST)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
