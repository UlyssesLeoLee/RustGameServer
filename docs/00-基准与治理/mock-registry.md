# Mock 资产登记册 (Mock Registry)

> **目的**:8 域 + cluster-ops + 工具集 全部 mock 资产集中索引,新增 mock 资产时同步追加
> **维护人**:Mavis(接手 agent per DEC-008,2026-08-28 跨反馈处置)
> **关联**:`RGS-SPEC-000 §2.4` + `RGS-IMPL-001 §3` + `crates/rgs-testkit/src/lib.rs`
> **强约束**:`mock::DbMock` / `mock::NoopMock` / `mock_url()` 全部 `#[deprecated]`,禁止用于 saga / 事务 / OCC / outbox 测试 (per RGS-REV-009 V3 H-1 / WF-1-55.31)

---

## 1. mock 资产中枢 (rgs-testkit crate)

**位置**:`crates/rgs-testkit/src/{mock.rs, fixture.rs, pg_test_db.rs, helper.rs, lib.rs}`

| 模块 | 入口 | 用途 | 状态 |
|---|---|---|---|
| `mock` | `rgs_testkit::mock::*` | 5 类 mock:DbMock(deprecated) / GrpcMock / NatsMock / InMemoryNatsMock / TonicGrpcMock | ✅ 53.3 + 54.x 稳定 |
| `fixture` | `rgs_testkit::fixture::*` | 5 域 + player/economy/saga 7 类 sample data | ✅ 53.3 + 54.x 稳定 |
| `pg_test_db` | `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]` | 真 PG 测试 fixture (per RGS-REV-009 V3 H-1 强约束) | ✅ 55.31 接入 |
| `helper` | `rgs_testkit::helper::*` | config 解析 / tracing 初始化 / 断言 helper | ✅ 53.3 稳定 |
| `lib` | `rgs_testkit::pg_pool` + `rgs_testkit::pg_test` re-export | 强约束 re-export | ✅ 55.31 接入 |

## 2. mock 入口 API 一览

### 2.1 NATS mock (InMemoryNatsMock, 54.x 实际可跑版本)

```rust
use rgs_testkit::mock::{InMemoryNatsMock, NatsMock};

let nats = InMemoryNatsMock::new();
nats.publish("subject", br#"{"event":"..."}"#).await?;
let msgs = nats.subscribe("subject").await?;
let count = nats.received_count("subject");
```

**适用场景**:单元 / 集成 test 中需要 mock NATS 但不起真 NATS server
**禁止场景**:e2e / chaos test (这些场景必须用真 NATS server)

### 2.2 gRPC mock (TonicGrpcMock, 54.x 实际可跑版本)

```rust
use rgs_testkit::mock::{GrpcMock, TonicGrpcMock};

let mut grpc = TonicGrpcMock::new().await;
grpc.expect("POST", "/player.v1.PlayerService/Login", 200, br#"{"ok":true}"#);
let url = grpc.url(); // 给 tonic client::connect(url)
```

**适用场景**:5 域 + gm-backend 测试中需要 mock 上下游 admin-service / player-service
**禁止场景**:5 域 gRPC 强一致测试 (Chaos 演练需用真 server)

### 2.3 PG mock (pg_pool + #[pg_test], 55.31 强约束)

```rust
use rgs_testkit::pg_test;
use sqlx::PgPool;

#[pg_test]
async fn my_real_pg_test(pool: PgPool) {
    let row: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(row, 1);
}
```

**适用场景**:saga / 事务 / OCC / outbox 测试 **必须**用
**禁止场景**:无 (强约束,5 域 + cluster-ops + admin 全部 55+ 个 DB-touched test 都已迁)

### 2.4 fixture (53.3 + 54.x 5 域)

```rust
use rgs_testkit::fixture::{self, FixtureBuilder};

// 53.3 backward compat
let p = fixture::player();
let e = fixture::economy("alice");
let s = fixture::saga("transfer");

// 54.x 5 域
let m = fixture::match_game("alice");
let s = fixture::social_message("alice", "bob");
let a = fixture::admin_action("admin01", "ban", "player123");

// FixtureBuilder 链式
let custom = FixtureBuilder::new(fixture::player())
    .with_name("Alice")
    .with_level(42)
    .build();
```

## 3. 8 域 + cluster-ops + 工具集 mock 资产分布

| 域 / 组件 | 测试 crate / 路径 | 用到 mock | 现有 mock 数 | TBD mock 数 | 关联 example |
|---|---|---|---|---|---|
| player-service | `crates/player-service/tests/integration_player_basic.rs` + `fail_closed_start.rs` | pg_test | 0(直接 PG) | 0 | `domain_player_demo.rs` |
| economy-service | `crates/economy-service/tests/{integration_reservation,integration_outbox,span_assertion,chaos_reservation}.rs` | pg_test + InMemoryNatsMock | 0 + 0 | 0 | `domain_economy_demo.rs` |
| match-service | `crates/match-service/tests/integration_match_basic.rs` + `fail_closed_start.rs` | pg_test + TonicGrpcMock | 0 + 0 | 0 | `domain_match_demo.rs` |
| social-service | `crates/social-service/tests/integration_social_basic.rs` + `fail_closed_start.rs` | pg_test + InMemoryNatsMock | 0 + 0 | 0 | `domain_social_demo.rs` |
| admin-service | `crates/admin-service/tests/integration_admin_basic.rs` + `fail_closed_start.rs` | pg_test + TonicGrpcMock | 0 + 0 | 0 | `domain_admin_demo.rs` |
| cluster-ops | `crates/cluster-ops/tests-disabled/*.rs` (4 ut_*, 旧债待清理) + `src/realm_lifecycle/tests/*.rs` (2 ut_*) + `tests/{drill_*,fail_closed_start,it_cross_domain,load_snapshot}.rs` | InMemoryNatsMock + TonicGrpcMock | 0 | 0(等旧债清理) | `domain_cluster_ops_demo.rs` |
| gm-backend | `crates/gm-backend/tests/{ut_config,integration_gm_basic,fail_closed_start}.rs` | axum-test 16 + assert_cmd 2 + serial_test 0.5 | 0 | 0 | `domain_gm_backend_demo.rs` |
| rgs-certgen(工具集) | **新增** `crates/rgs-certgen/tests/ut_blackbox.rs` (per 2026-08-28 跨反馈 F1/F2 衍生,本轮实装) | assert_cmd 2 | 0 | **17**(本轮实装) | (本轮新增) |

## 4. mock 资产例 (8 域 + cluster-ops + 工具集 9 个 example)

**位置**:`crates/rgs-testkit/examples/`

| example | 行数 | 跑通命令 | 关联 8 域 |
|---|---|---|---|
| `domain_player_demo.rs` | 41 | `cargo run --example domain_player_demo -p rgs-testkit` | player |
| `domain_economy_demo.rs` | 47 | `cargo run --example domain_economy_demo -p rgs-testkit` | economy |
| `domain_match_demo.rs` | 38 | `cargo run --example domain_match_demo -p rgs-testkit` | match |
| `domain_social_demo.rs` | 41 | `cargo run --example domain_social_demo -p rgs-testkit` | social |
| `domain_admin_demo.rs` | 53 | `cargo run --example domain_admin_demo -p rgs-testkit` | admin |
| `domain_cluster_ops_demo.rs` | 47 | `cargo run --example domain_cluster_ops_demo -p rgs-testkit` | cluster-ops |
| `domain_gm_backend_demo.rs` | 52 | `cargo run --example domain_gm_backend_demo -p rgs-testkit` | gm-backend |
| (per 53.3) `mock_grpc_demo.rs` | (已有) | `cargo run --example mock_grpc_demo -p rgs-testkit` | 通用 |
| (per 53.3) `mock_nats_demo.rs` | (已有) | `cargo run --example mock_nats_demo -p rgs-testkit` | 通用 |
| (per 53.3) `fixture_builder_demo.rs` | (已有) | `cargo run --example fixture_builder_demo -p rgs-testkit` | 通用 |
| (per 53.3) `test_deprecated_warns.rs` | (已有) | `cargo run --example test_deprecated_warns -p rgs-testkit` | 通用 |

**总 example 数**:11(7 域新 + 4 通用)

## 5. mock 资产回归测试 (Regression Smoke)

**入口**:`scripts/regression-smoke.sh` (per 本轮新增,见 scripts/ 目录)

**用途**:本机或 CI 触发后,按顺序跑:
1. 7 域 example (`cargo run --example domain_*_demo -p rgs-testkit`)
2. 5 域 + cluster-ops + gm-backend `cargo test`
3. 工具集 rgs-certgen 黑盒 test (本轮新增)
4. e2e smoke 12 端口 (per scripts/e2e-smoke.sh)

**evidence 落点**:`docs/00-基准与治理/.test-evidence/regression/{date}/`

## 6. mock 资产新增流程

1. 在 `crates/rgs-testkit/src/{mock,fixture}.rs` 加新 API
2. 在 `crates/rgs-testkit/examples/` 加 example
3. 在本登记册 §2 / §3 / §4 同步追加入口
4. `cargo run --example new_demo -p rgs-testkit` 跑通
5. commit 前跑 `scripts/regression-smoke.sh` 确认无回归

## 7. 强约束 (per RGS-REV-009 V3 H-1 / WF-1-55.31)

- **`mock::DbMock` / `mock::NoopMock` / `mock_url()` 全部 `#[deprecated]`** — 禁止用于 saga / 事务 / OCC / outbox 测试
- **DB / saga / OCC / outbox 测试** 必须用 `rgs_testkit::pg_pool()` + `#[rgs_testkit::pg_test]`
- **gRPC mock** 53.3 trait 占位 + 54.x 实际 `TonicGrpcMock` (mockito HTTP server) 两者并存,实际用后者
- **NATS mock** trait 占位 + 实际 `InMemoryNatsMock` (Arc<Mutex<HashMap>>) 两者并存,实际用后者

**违反处置**:CI 上 `cargo build --tests --all-features` 会有 `#[deprecated]` warning,定位为 P1 违规

---

**作者**:Mavis(接手 agent per DEC-008)
**审批**:架构师(Mavis 接手 agent per DEC-008)+ 自审 + 2026-08-28
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
