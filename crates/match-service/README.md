# match-service

> RustGameServer 5 域之一: 比赛域
> 规范: DTL-026 + RGS-SPEC-000 §2.4

## 职责

- 比赛创建 / 报名 / 匹配
- 比赛状态 (Pending / Active / Finished)
- 比赛结果 / 得分记录

## 测试 (rgs-testkit)

本域测试统一用 `rgs-testkit` 提供 fixture + mock.

### 真 PG 集成测试 (56.x 起必须)

```rust
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn match_assignment_concurrent_no_double_book(pool: PgPool) {
    // 真 PG OCC: 同一比赛 slot 只能分配给一个玩家
    // UPDATE match_assignments SET player_id = $1, version = version + 1
    //   WHERE match_id = $2 AND slot = $3 AND version = $4
    // 0 row → slot 已被占, 拒绝
    let repo = match_service::PgMatchAssignmentRepository::new(pool.clone());
    let match_id = "match-001";
    // ... 2 个并发 assign(同 slot) 触发竞争
    // ... 断言: 仅 1 个成功, 另一个收到 OCC conflict
}
```

### 53.3 fixture (已实现 + 54.x 路线)

```rust
use rgs_testkit::fixture;

#[test]
fn my_match_test() {
    // 53.3 仅 player / economy / saga fixture
    // match 域 fixture 待 54.x 接入: fixture::match_game("alice")
    let p = fixture::player();
    let s = fixture::saga("match_end");
    assert_eq!(s.saga_type, "match_end");
}
```

### 跑测试

```bash
# 单元测试 (CI 默认, 不需 PG)
cargo test -p match-service

# 集成测试 (需 PG + DATABASE_URL)
cargo test -p match-service --features pg-integration

# 集成测试 + 跑 ignored (smoke)
cargo test -p match-service --features pg-integration -- --include-ignored
```

### 强约束 (per RGS-REV-009 V3 H-1)

- **新加的** match 域 DB / OCC / 状态机 / outbox 测试**必须**用 `#[rgs_testkit::pg_test]`
- 禁止 `#[tokio::test]` + InMemoryRepository
- 禁止 `mock::NoopMock` / `mock_url()`

详见 `crates/rgs-testkit/README.md` §7 + RGS-SPEC-000 §2.4.
