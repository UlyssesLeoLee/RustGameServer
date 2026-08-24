# social-service

> RustGameServer 5 域之一: 社交域
> 规范: DTL-019 + DTL-020 + RGS-SPEC-000 §2.4

## 职责

- 好友关系 (add / remove / block)
- 消息 (私聊 / 群聊)
- 公会 (guild) / 聊天频道

## 测试 (rgs-testkit)

本域测试统一用 `rgs-testkit` 提供 fixture + mock.

### 真 PG 集成测试 (56.x 起必须)

```rust
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn guild_join_rollback_on_duplicate(pool: PgPool) {
    // 真 PG 事务: 加入公会 + 写入成员表, 重复加入触发 UNIQUE constraint
    // 整事务回滚, 验 0 row 副作用
    let guild_svc = social_service::PgGuildService::new(pool.clone());
    // ... 准备 guild
    // ... 1 个 join 成功
    // ... 另一个 join(同 player) 触发 UNIQUE 冲突, 整事务回滚
    // ... 断言: guild.members 表无重复行
}
```

### 53.3 fixture (已实现 + 54.x 路线)

```rust
use rgs_testkit::fixture;

#[test]
fn my_social_test() {
    // 53.3 仅 player / economy / saga fixture
    // social 域 fixture 待 54.x 接入: fixture::social_message("alice", "bob")
    let p = fixture::player();
    assert_eq!(p.id, "player-test-001");
}
```

### 跑测试

```bash
# 单元测试 (CI 默认, 不需 PG)
cargo test -p social-service

# 集成测试 (需 PG + DATABASE_URL)
cargo test -p social-service --features pg-integration

# 集成测试 + 跑 ignored (smoke)
cargo test -p social-service --features pg-integration -- --include-ignored
```

### 54.x fixture (新增)

```rust
use rgs_testkit::fixture::{self, FixtureBuilder};

#[test]
fn my_social_test() {
    let s = fixture::social_message("alice", "bob");
    assert_eq!(s.player_id, "alice");
    assert_eq!(s.friend_id, "bob");
    assert_eq!(s.message, "Hello from test");
}

#[test]
fn my_social_test_with_builder() {
    let s = FixtureBuilder::new(fixture::social_message("alice", "bob"))
        .with_message("Hello from test fixture!")
        .build();
    assert_eq!(s.message, "Hello from test fixture!");
}
```

### 强约束 (per RGS-REV-009 V3 H-1)

- **新加的** social 域 DB / 关系 / 事务 / outbox 测试**必须**用 `#[rgs_testkit::pg_test]`
- 禁止 `#[tokio::test]` + InMemoryRepository
- 禁止 `mock::NoopMock` / `mock_url()`

详见 `crates/rgs-testkit/README.md` §7 + RGS-SPEC-000 §2.4.
