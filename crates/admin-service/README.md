# admin-service

> RustGameServer 5 域之一: 管理域
> 规范: DTL-031 + RGS-SPEC-000 §2.4

## 职责

- 管理员操作 (ban / promote / demote)
- 审计日志 (audit log, 不可变)
- RBAC 权限检查 (role-based access control)

## 测试 (rgs-testkit)

本域测试统一用 `rgs-testkit` 提供 fixture + mock.

### 真 PG 集成测试 (56.x 起必须)

```rust
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn audit_log_immutable_insert(pool: PgPool) {
    // 真 PG: audit_log 表无 UPDATE/DELETE 权限, 仅 INSERT
    // 测 INSERT 成功, UPDATE/DELETE 报 permission denied
    let audit_repo = admin_service::PgAuditLogRepository::new(pool.clone());
    let entry = audit_repo.log(admin_id: "admin1", action: "ban", target: "bad_player").await.unwrap();
    // ... 断言 entry.id 已分配, 不允许 update
    // ... 尝试 update 应失败 (PG permission denied)
}

#[pg_test]
async fn rbac_check_concurrent_role_change(pool: PgPool) {
    // 真 PG OCC: 角色变更并发, 防止 RBAC 状态不一致
    // UPDATE users SET role = $1, version = version + 1
    //   WHERE id = $2 AND version = $3
    // 0 row → 角色已变, 重新读
    let rbac_svc = admin_service::PgRbacService::new(pool.clone());
    // ... 2 个并发 role_change(同 user) 触发竞争
    // ... 断言: 仅 1 个成功, 另一个收到 OCC conflict
}
```

### 53.3 fixture (已实现 + 54.x 路线)

```rust
use rgs_testkit::fixture;

#[test]
fn my_admin_test() {
    // 53.3 仅 player / economy / saga fixture
    // admin 域 fixture 待 54.x 接入: fixture::admin_action("admin1", "ban", "bad_player")
    let p = fixture::player();
    let s = fixture::saga("admin_audit");
    assert_eq!(s.saga_type, "admin_audit");
}
```

### 跑测试

```bash
# 单元测试 (CI 默认, 不需 PG)
cargo test -p admin-service

# 集成测试 (需 PG + DATABASE_URL)
cargo test -p admin-service --features pg-integration

# 集成测试 + 跑 ignored (smoke)
cargo test -p admin-service --features pg-integration -- --include-ignored
```

### 54.x fixture (新增)

```rust
use rgs_testkit::fixture::{self, FixtureBuilder};

#[test]
fn my_admin_test() {
    let a = fixture::admin_action("admin1", "ban", "bad_player");
    assert_eq!(a.admin_id, "admin1");
    assert_eq!(a.action, "ban");
    assert_eq!(a.target_id, "bad_player");
}

#[test]
fn my_admin_test_with_builder() {
    let a = FixtureBuilder::new(fixture::admin_action("admin1", "promote", "alice"))
        .with_action("demote")
        .with_target("bob")
        .build();
    assert_eq!(a.action, "demote");
    assert_eq!(a.target_id, "bob");
}
```

### 强约束 (per RGS-REV-009 V3 H-1)

- **新加的** admin 域 DB / RBAC / 审计 / outbox 测试**必须**用 `#[rgs_testkit::pg_test]`
- 禁止 `#[tokio::test]` + InMemoryRepository
- 禁止 `mock::NoopMock` / `mock_url()`

详见 `crates/rgs-testkit/README.md` §7 + RGS-SPEC-000 §2.4.
