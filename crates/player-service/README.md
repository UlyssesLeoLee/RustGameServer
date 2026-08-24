# player-service

> RustGameServer 5 域之一: 玩家域
> 规范: DTL-018 + RGS-SPEC-000 §2.4

## 职责

- 玩家注册 / 登录 / session 管理
- 玩家属性 (level / exp / currency)
- 玩家状态 (online / offline / banned)

## 测试 (rgs-testkit)

本域测试统一用 `rgs-testkit` 提供 fixture + mock.

### 真 PG 集成测试 (56.x 起必须)

```rust
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn player_level_up_atomic_occ(pool: PgPool) {
    // 真 PG OCC: UPDATE players SET level = level + 1, version = version + 1
    // WHERE id = $1 AND version = $2
    // 0 row → OCC conflict, 应用层拒绝
    let repo = player_service::PgPlayerRepository::new(pool.clone());
    let p = repo.create_player("alice").await.unwrap();
    // ... 2 个并发 level_up(同 version) 触发竞争
    // ... 断言: 仅 1 个成功, 另一个收到 OCC conflict
}

#[pg_test]
async fn player_login_creates_session(pool: PgPool) {
    // 测 PG 事务隔离下, login → session 创建的原子性
    // ... 跨事务 prepare + commit
}
```

### 53.3 fixture (已实现)

```rust
use rgs_testkit::fixture;

#[test]
fn my_player_test() {
    let p = fixture::player();
    assert_eq!(p.id, "player-test-001");
    assert_eq!(p.name, "Test Player");
    assert_eq!(p.level, 1);
}
```

### 跑测试

```bash
# 单元测试 (CI 默认, 不需 PG)
cargo test -p player-service

# 集成测试 (需 PG + DATABASE_URL)
cargo test -p player-service --features pg-integration

# 集成测试 + 跑 ignored (smoke)
cargo test -p player-service --features pg-integration -- --include-ignored
```

### 强约束 (per RGS-REV-009 V3 H-1)

- **新加的** player 域 DB / OCC / session / outbox 测试**必须**用 `#[rgs_testkit::pg_test]`
- 禁止 `#[tokio::test]` + InMemoryRepository
- 禁止 `mock::NoopMock` / `mock_url()`

详见 `crates/rgs-testkit/README.md` §7 + RGS-SPEC-000 §2.4.
