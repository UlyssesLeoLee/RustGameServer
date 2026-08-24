# economy-service

> RustGameServer 5 域之一: 经济域
> 规范: DTL-015 + DTL-016 + RGS-SPEC-000 §2.4

## 职责

- 账户余额 (currency / gold) 管理
- 转账 (transfer) / 充值 (deposit) / 扣费 (withdraw)
- Saga 编排 (transfer 跨账户)

## 测试 (rgs-testkit)

本域测试统一用 `rgs-testkit` 提供 fixture + mock.

### 真 PG 集成测试 (56.x 起必须, RGS-REV-009 V3 H-1 / DC-1)

```rust
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn resume_concurrent_race_rejected_by_occ(pool: PgPool) {
    // 真 PG OCC: UPDATE accounts SET version = version + 1 WHERE id = ? AND version = ?
    // 0 row → PG 返回 rows_affected = 0, 应用层拒绝
    // 比 InMemory `stored.version = original + 99` 模拟更真
    let repo = economy_service::PgAccountRepository::new(pool.clone());
    // ... 准备 account + reservation
    // ... 2 个并发 resume(同 saga_id) 触发竞争
    // ... 断言: 仅 1 个成功, 另一个收到 OCC conflict
}

#[pg_test]
async fn saga_crash_recovery_uses_real_transaction_isolation(pool: PgPool) {
    // 测 PG Read Committed 隔离级下, saga_orchestrator::resume 的可见性
    // ... 跨事务 prepare + commit, 验 30s 轮询能 resume
}
```

### 53.3 fixture (已实现)

```rust
use rgs_testkit::fixture;

#[test]
fn my_economy_test() {
    let e = fixture::economy("alice");
    assert_eq!(e.player_id, "alice");
    assert_eq!(e.currency, 1000);
    assert_eq!(e.gold, 50);
}

#[test]
fn my_saga_test() {
    let s = fixture::saga("transfer");
    assert_eq!(s.saga_type, "transfer");
    assert_eq!(s.state, "Pending");
    assert_eq!(s.step, 0);
}
```

### 跑测试

```bash
# 单元测试 (CI 默认, 不需 PG)
cargo test -p economy-service

# 集成测试 (需 PG + DATABASE_URL)
cargo test -p economy-service --features pg-integration

# 集成测试 + 跑 ignored (smoke)
cargo test -p economy-service --features pg-integration -- --include-ignored
```

### 强约束 (per RGS-REV-009 V3 H-1)

- **新加的** economy 域 transfer / 事务 / OCC / outbox 测试**必须**用 `#[rgs_testkit::pg_test]`
- 禁止 `#[tokio::test]` + InMemoryRepository
- 禁止 `mock::NoopMock` / `mock_url()`
- **RGS-REV-009 V3 H-1 / DC-1** 重点: transfer OCC + saga crash recovery 必须真 PG 覆盖

详见 `crates/rgs-testkit/README.md` §7 + RGS-SPEC-000 §2.4.
