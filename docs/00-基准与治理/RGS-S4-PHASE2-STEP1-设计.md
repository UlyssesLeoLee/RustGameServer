# S4 Phase 2 Step 1 设计 — gm-backend admin-service gRPC client 注入

> **目标**:实装 gm-backend tonic Channel for admin-service + HealthView 走 gRPC 调 admin-service HealthCheck, 其他 4 endpoint 保留 stub
> **作者**:Mavis (接手 agent per DEC-008,2026-08-28 16:32 JST)
> **状态**:⏳ OPEN (worker bg_5e6b37c7 实施中)
> **关联**:RGS-TBD-08-03 S4 立项 v0.2 (commit `404e3ea` 之后) + gm.proto v0.3 (commit `c5c9f5f`)

---

## 1. 范围

### 1.1 Step 1 范围 (本 commit)
- ✅ build.rs 纳入 `admin.proto` (gm-backend 作 client-only)
- ✅ lib.rs 加 `AdminGrpcClient` + `AppState.admin_grpc: Option<Arc<...>>`
- ✅ HealthView 调 admin-service gRPC `HealthCheck`, 500ms timeout, 失败降级
- ✅ 4 个 stub endpoint (ban/grant/maintenance/query_audit) 加 TODO 注释
- ✅ 新增 IT: `it_admin_grpc_client.rs` (3+ 测试)
- ✅ 36/36 UT 不变

### 1.2 Step 2+ 范围 (后续)
- ⏳ admin-service 加 5 GM RPC (BanAccount / GrantCompensation / SetMaintenance / QueryAuditLog)
- ⏳ gm-backend BanAccount / GrantCompensation 调 admin-service gRPC, 失败降级 InMemory AuditStore
- ⏳ SetMaintenance 调 admin-service, propagation_status 从 admin 返回
- ⏳ QueryAuditLog 调 admin-service, 替换 InMemory list_entries
- ⏳ JWT propagation: gRPC metadata `authorization: Bearer <jwt>`
- ⏳ Circuit breaker: 5 次失败 → 断开 30s
- ⏳ IT chaos: admin-service 503 → gm-backend 503 + 业务降级
- ⏳ 跨域 IT (W2): cluster-ops ↔ 5 域 ↔ admin ↔ gm-backend 链路

---

## 2. 关键设计决策

### 2.1 fail-open for connection init
- `AppState::new()` 调 `tonic::transport::Channel::connect(admin_grpc_endpoint).await`
- **失败不 panic**, 设 `admin_grpc = None` + `tracing::warn!`
- 原因: gm-backend 启动时 admin-service 可能未就绪 (k8s rollout 顺序)
- 运行时 admin-service 不可达 → HealthView 失败降级 (ready=false), 不让 8081 探针误判 gm-backend 自己挂

### 2.2 client-only tonic-build
- `tonic_build::configure().build_server(false).build_client(true).compile_protos(...)`
- 原因: gm-backend 不是 gRPC server, 只做 client; build_server(false) 防 Rpc/Service trait 不生成, 减少编译时间

### 2.3 common.proto 共享
- admin.proto `import "common/v1/common.proto"`
- shared-platform crate 已有 `proto/common/v1/common.proto`
- build.rs 用 `.include_path("../shared-platform/proto")` 让 tonic-build 找得到
- 这是 v0.4 统一 proto 仓库的过渡方案 (per gm.proto v0.3 注释)

### 2.4 失败降级: HealthView 行为
```
admin_grpc.is_some() AND gRPC 500ms 内返回 Ok:
  → services[0].ready = response.healthy
  → db_pool_usage_ratio = 0.0 (admin-service 未暴露)
  → checked_at_ms = now
admin_grpc.is_some() AND gRPC 失败/超时:
  → services[0].ready = false
  → db_pool_usage_ratio = 0.0
  → checked_at_ms = now
  → tracing::warn! 记录
admin_grpc.is_none() (测试 / 连接初始化失败):
  → services[0].ready = true (保持 v0.2 stub 行为, 兼容现有 UT)
```

---

## 3. 改动文件清单 (实施后由 worker 报告补充)

| 文件 | 改动 |
|---|---|
| `crates/gm-backend/build.rs` | 新增 admin.proto 编译 |
| `crates/gm-backend/src/lib.rs` | AdminGrpcClient + AppState.admin_grpc + HealthView gRPC 调 |
| `crates/gm-backend/tests/it_admin_grpc_client.rs` | 新文件, 3+ 测试 |
| `Cargo.lock` | (可能自动更新) |

---

## 4. 跑测 (实施后由 worker 报告补充)

```bash
# WSL 装 rustc 1.98 + cargo 1.98
# 独立 target dir /tmp/cargo-target-wsl-g3
# 复用 G3 fixture: postgres superuser (port 15432)

$ source scripts/db-url.sh postgres-superuser 15432
$ /home/leo19/.cargo/bin/cargo test -p gm-backend --no-fail-fast

# 期望: 36 (旧) + 3+ (新) = 39+ PASS
```

---

## 5. 已知缺口 (缺标比错标安全)

- ⏳ admin.proto 编译依赖 common.proto, 若 shared-platform proto 路径变, 需改 `.include_path`
- ⏳ HealthView 当前 500ms timeout 硬编码, 待 Phase 3 加 config-driven
- ⏳ 其他 4 endpoint (ban/grant/maintenance/query_audit) 仍 stub, 待 Step 2+ admin-service 加 5 GM RPC
- ⏳ JWT propagation 未实装, 待 Step 2+
- ⏳ Circuit breaker 未实装, 待 Step 2+
- ⏳ mTLS to admin-service: 暂用 plaintext (因 k3s 内 ClusterIP 走 in-cluster), 待 Step 2+ 决策是否启用 mTLS (per BAS-003 §2.1)

---

## 6. 关联文档

- **RGS-BAS-003** §2.1 (gm-backend APIGW + mTLS)
- **RGS-BAS-003** §3.1-§3.4 (5 GM endpoint 字段级)
- **RGS-DTL-003** §3.3-§3.4 (propagation_status / entries+has_more)
- **RGS-TBD-08-03** S4 立项 v0.2 (6 天工作量分解)
- **RGS-TBD-08-01** JWT middleware v0.2 (已实装, 36/36 UT)
- **RGS-TBD-08-04** AuditStore trait v0.2 (InMemory 默认 + PgAuditStore 延后)
- **gm.proto v0.3** (commit `c5c9f5f`, 5 endpoint 协议 + 简化)
- **commit `1c2bf91`** G3+G4 evidence (75.9% 覆盖, 14/14 域 ≥ 60%)
- **commit `1790b18`** G3 fixture 修复 (sqlx leo19 fallback + 3 非 fixture bug)

---

## 7. 下一步 (Phase 2 Step 2+)

1. **admin-service 加 5 GM RPC** proto schema (BanAccount/GrantCompensation/SetMaintenance/QueryAuditLog)
2. **gm-backend BanAccount/GrantCompensation handler** 调 admin-service gRPC
3. **SetMaintenance** 调 admin-service, propagation_status 从 admin 返回
4. **QueryAuditLog** 调 admin-service, 替换 InMemory list_entries
5. **JWT propagation** gRPC metadata
6. **Circuit breaker** 5 次失败 → 30s 断开
7. **IT chaos** admin-service 503 → gm-backend 503 降级
8. **跨域 IT (W2)** cluster-ops ↔ 5 域 ↔ admin ↔ gm-backend
9. **mTLS to admin-service** (per BAS-003 §2.1, 决策待 Step 2+)
10. **GM endpoint → 实际业务** 5 域 handler 集成 (BanAccount → player-service / economy-service)
