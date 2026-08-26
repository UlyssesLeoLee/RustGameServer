# RGS-TEST-STRATEGY-2026-08-26 v0.1

**RGS 单元测试覆盖率 > 90% 实施策略**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TEST-STRATEGY-2026-08-26 |
| 版本 | 0.1（per Ulysses 2026-08-26 13:28 JST "完善 mock 项目，对整个项目进行覆盖率大于 90% 的 UT 测试"）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | 2026-08-26 13:28 JST |
| 责任人 | 架构师（Mavis 接手 agent per DEC-008）|
| 适用许可 | Apache-2.0 |

---

## 0. 文档定位

本文档是 RGS 项目**单元测试覆盖率 90% 达成**的实施策略 + 4 阶段实施计划 + 验收标准。回答"怎么做到 90%"。

按 RGS-DTL-001 文档分层:本文档是**测试规约 + 实施计划**层,非具体测试代码。

---

## 1. 当前状态评估(per 2026-08-26 13:28 JST)

### 1.1 代码规模

| Crate | src 文件 | src 行数 | src-ut (#[test] in src) | int-test (in tests/) | 总测试数 |
|---|---|---|---|---|---|
| player-service | 8 | 1,434 | 13 | 1 (2 files) | 14 |
| economy-service | 12 | 4,996 | 14 | 3 (5 files) | 17 |
| match-service | 8 | 1,359 | 10 | 1 (2 files) | 11 |
| social-service | 8 | 1,247 | 9 | 1 (2 files) | 10 |
| admin-service | 8 | 1,547 | 11 | 1 (2 files) | 12 |
| **cluster-ops** | 20 | **3,952** | 52 | 12 (2 files) | 64 |
| **shared-platform** | 20 | **4,059** | **66** | 0 | 66 |
| rgs-testkit | 5 | 836 | 4 | 16 (4 files) | 20 |
| function-plane | 6 | 1,050 | 1 | 5 (1 files) | 6 |
| rgs-asset-download | 13 | 1,949 | 14 | 79 (15 files) | 93 |
| rgs-hello | 1 | 3 | 0 | 0 | 0 |
| rgs-certgen | 1 | 130 | 0 | 0 | 0 |
| **合计** | **110** | **22,562** | **194** | **119 (40 files)** | **313** |

**当前覆盖率估算**:基于 `cargo-tarpaulin` 或 `cargo-llvm-cov`(待实测),按行数比例 + 已测代码块比例粗估 **40-55%**(待 tool 实测确认)。

### 1.2 已有测试基础设施

| 资产 | 路径 | 状态 |
|---|---|---|
| `pg_test` macro | `crates/rgs-testkit/src/pg_test_db.rs` | ✅ 用 sqlx::test re-export,真 PG 集成测试 |
| `pg_pool()` helper | 同上 | ✅ 真 PG pool 强约束入口 |
| `GrpcMock` | `crates/rgs-testkit/src/mock.rs` | ✅ tonic gRPC mock server (mockito) |
| `NatsMock` + `InMemoryNatsMock` | 同上 | ✅ Arc<Mutex<HashMap>> NATS mock |
| `fixture` 工厂 | `crates/rgs-testkit/src/fixture.rs` | ✅ 6 DB init/teardown + sample data |
| `helper` | `crates/rgs-testkit/src/helper.rs` | ✅ config 加载 + tracing 初始化 |
| 强约束编译期 deprecation | `pg_test_db.rs` | ✅ NoopMock / mock_url() 编译期警告 |

**评估**:mock 工具丰富,覆盖率提升只需"补 test 函数",不需新工具。

### 1.3 缺口分布

| 缺口 | 占比 | 说明 |
|---|---|---|
| shared-platform 0 集成 | 0/20 | 4,059 行 src 缺 e2e 集成测试 |
| rgs-certgen 0 测试 | 0/130 | cert 工具需补 |
| 5 域 int-test 太少 | 7/5 | 每域 1 个 int-test 文件,缺 happy path + error path |
| 缺覆盖率数据 | 100% | 没跑过 cargo-tarpaulin/cargo-llvm-cov,无基线数据 |

---

## 2. 目标定义

### 2.1 覆盖率目标(per NFR-1/2)

| 指标 | 当前(估) | 目标 v0.1 | 目标 v1.0 |
|---|---|---|---|
| **行覆盖率(line coverage)** | 40-55% | **> 90%** | > 95% |
| **分支覆盖率(branch coverage)** | 未知 | > 85% | > 90% |
| **函数覆盖率(function coverage)** | 60-70% | > 90% | > 95% |
| **critical path 覆盖率** | 50-60% | **> 95%** | 100% |

**critical path 定义**:
- 5 域 OCC 冲突 + 事务
- cluster-ops PFAU 阶段机
- shared-platform outbox relay + producer/consumer
- rgs-testkit pg_test_db 自身
- saga 编排(RGS-IMPL-100)

### 2.2 非覆盖率目标

| 目标 | 衡量 |
|---|---|
| 编译期严格性 | `cargo build` 无 warning(deprecated 等也算) |
| 性能 | 5 域 src test < 5s,shared-platform test < 10s,workspace total < 30s |
| CI 集成 | `cargo test --workspace` 必须 < 60s 完成 |
| 可维护性 | 关键路径每函数有 doc + 测试注释 |

---

## 3. 4 阶段实施计划

### 3.1 P0:基线测量 + 工具准备(0.5 天)

| 任务 | 工时 | 输出 |
|---|---|---|
| 安装 cargo-tarpaulin | 5 min | `cargo install cargo-tarpaulin` |
| 跑基线覆盖率 | 5 min | tarpaulin report HTML |
| 安装 cargo-llvm-cov(备用) | 5 min | 备选工具 |
| 写 `coverage.sh` 脚本 | 10 min | `scripts/coverage.sh` 一键跑 + 报告 |

**P0 完成标准**:有 RGS-COVERAGE-BASELINE-2026-08-26.md 报告,记录当前覆盖率数据。

### 3.2 P1:shared-platform 100% + cluster-ops 90%(2 天)

**理由**:
- shared-platform 是 5 域 + cluster-ops 的依赖底座
- 4,059 行代码,缺 0 集成测试,src-ut 66
- 涵盖:channel/client/consumer/producer + outbox + rbac + retry + tracing

**任务清单**:
- [ ] shared-platform/src/outbox.rs 加 5-8 集成测试(用 pg_test)
- [ ] shared-platform/src/producer.rs 加 3-5 集成测试
- [ ] shared-platform/src/consumer.rs 加 3-5 集成测试
- [ ] shared-platform/src/grpc_tracing.rs 加 4-6 unit test
- [ ] shared-platform/src/retry.rs 加 3-5 unit test
- [ ] shared-platform/src/rbac.rs 加 4-6 unit test
- [ ] shared-platform/src/tls.rs 加 2-3 unit test
- [ ] shared-platform/src/json_logging.rs 加 2-3 unit test
- [ ] shared-platform/src/subject.rs 加 1-2 unit test
- [ ] shared-platform/src/messaging.rs 加 3-5 unit test
- [ ] shared-platform/src/metrics.rs 加 3-5 unit test
- [ ] shared-platform/src/span_helpers.rs 加 2-3 unit test
- [ ] shared-platform/src/dlq.rs 加 3-5 unit test
- [ ] cluster-ops/src/grpc_service.rs 加 5-8 unit test
- [ ] cluster-ops/src/repository.rs 加 5-8 unit test
- [ ] cluster-ops/src/service.rs 加 3-5 unit test

**目标**:shared-platform 100% line, cluster-ops 90% line

### 3.3 P2:5 域 + rgs-testkit 90%(3 天)

**5 域核心 path**:
- player-service:注册 / 登录 / 角色 / OCC 冲突
- economy-service:扣减 / 发放 / Saga / 退款
- match-service:房间 / 撮合 / tick 20Hz
- social-service:好友 / 聊天 / 邮件
- admin-service:RBAC / 审计 / 公告

**任务清单**(每域):
- [ ] player:10-15 unit + 5-8 integration
- [ ] economy:15-20 unit + 8-10 integration
- [ ] match:8-10 unit + 4-6 integration
- [ ] social:8-10 unit + 4-6 integration
- [ ] admin:10-12 unit + 5-8 integration
- [ ] rgs-testkit:补 helper 边界 + pg_test 边界 case

**目标**:5 域各 90% line + branch

### 3.4 P3:rgs-certgen + rgs-hello + function-plane + rgs-asset-download 100%(1 天)

- rgs-certgen: 100% line(130 行 + cert 边界 case)
- rgs-hello: 100% line(3 行 + smoke test)
- function-plane: 100% line
- rgs-asset-download: 当前 79 int-test,补 src-ut 到 100%

---

## 4. 关键实施模式

### 4.1 单元测试模式

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rgs_testkit::helper::init_test_tracing;

    #[test]
    fn test_function_happy_path() {
        init_test_tracing();
        let input = ...;
        let result = func(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_function_error_path() {
        // 给错误输入,断言错误类型
        let err = func(bad_input).unwrap_err();
        assert!(matches!(err, Error::ExpectedType { .. }));
    }

    #[test]
    fn test_function_edge_case() {
        // 边界条件:空 / 0 / max / null
    }
}
```

### 4.2 PG 集成测试模式(per rgs-testkit 强约束)

```rust
use rgs_testkit::{pg_test, pg_pool};

#[pg_test]
async fn test_saga_commit(pool: sqlx::PgPool) {
    let repo = PgAccountRepository::new(pool.clone());
    let account = repo.create_account(...).await.unwrap();
    let result = repo.apply_atomic(...).await;
    assert!(result.is_ok());
    // pool 自动 rollback 隔离
}
```

### 4.3 gRPC mock 测试模式

```rust
use rgs_testkit::mock::TonicGrpcMock;

#[tokio::test]
async fn test_client_grpc() {
    let mock = TonicGrpcMock::new().await;
    mock.expect("GetPlayer").returning(GetPlayerResponse { ... });
    let client = MyServiceClient::connect(mock.url()).await.unwrap();
    let resp = client.get_player(req).await.unwrap();
    assert_eq!(resp.player_id, expected);
}
```

### 4.4 NATS mock 测试模式

```rust
use rgs_testkit::mock::InMemoryNatsMock;

#[tokio::test]
async fn test_producer_publish() {
    let nats = InMemoryNatsMock::new();
    nats.publish("rgs.player.registered", payload).await.unwrap();
    let received = nats.subscribe("rgs.player.registered").await.unwrap();
    assert_eq!(received.len(), 1);
}
```

### 4.5 覆盖率自动化

```bash
# scripts/coverage.sh
#!/usr/bin/env bash
set -e
cargo clean -p shared-platform
cargo llvm-cov --workspace --html --output-dir coverage/
echo "Coverage: $(cat coverage/summary.txt)"
```

---

## 5. 验收标准

### 5.1 P0(基线)验收

- [ ] `scripts/coverage.sh` 可跑通
- [ ] RGS-COVERAGE-BASELINE-2026-08-26.md 存在
- [ ] 报告含 12 个 crate 的 line/branch/function 覆盖率
- [ ] baseline 数字记录在 `docs/06-测试/RGS-COVERAGE-BASELINE-2026-08-26.md`

### 5.2 P1(shared-platform + cluster-ops)验收

- [ ] shared-platform 行覆盖率 > 100%(包含 #[cfg(test)] 行,目标 100%)
- [ ] cluster-ops 行覆盖率 > 90%
- [ ] `cargo test -p shared-platform` < 10s
- [ ] `cargo test -p cluster-ops` < 10s
- [ ] 无 #[deprecated] 警告残留(除强约束外)

### 5.3 P2(5 域)验收

- [ ] 5 域各行覆盖率 > 90%
- [ ] critical path 覆盖率 100%(OCC / Saga / 5 域核心 service)
- [ ] rgs-testkit 行覆盖率 > 90%
- [ ] `cargo test --workspace` < 60s

### 5.4 P3(辅助 crate)验收

- [ ] rgs-certgen / rgs-hello / function-plane / rgs-asset-download 各 100%
- [ ] 整体 workspace 覆盖率 > 90%
- [ ] CI `cargo test --workspace` 集成 coverage 报告

### 5.5 完整验收

- [ ] workspace 总行覆盖率 > 90%
- [ ] workspace 总分支覆盖率 > 85%
- [ ] workspace 总函数覆盖率 > 90%
- [ ] critical path 100% 覆盖
- [ ] RGS-COVERAGE-FINAL-2026-08-26.md 报告

---

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| cargo-tarpaulin 安装慢(LLVM 依赖)| 高 | P0 延迟 | 用 cargo-llvm-cov 备选(更轻量) |
| shared-platform 0 集成 → 100% 难度 | 中 | P1 延迟 | 分 2 子阶段:src-ut 补 100% → 集成 50% → 80% |
| 5 域 gRPC 联调复杂 | 中 | P2 延迟 | 用 InMemoryRepository + gRPC mock,避开真 PG / gRPC |
| DB schema 变更导致测试过期 | 低 | 长期 | rgs-testkit 统一管理 migration |
| 覆盖率数字波动 | 中 | 验收 | 用 cargo-llvm-cov 多次跑取中位数 |

---

## 7. CI 集成(per RGS-DOCS-HEALTH)

### 7.1 GitHub Actions(已存在 rust-ci.yml)

```yaml
# 现有 job 基础上加 coverage job
jobs:
  test:
    steps:
      - run: cargo test --workspace
  coverage:
    needs: test
    steps:
      - run: cargo install cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v3
        with:
          file: lcov.info
          fail_ci_if_error: true
          threshold: 90%  # < 90% 阻断 merge
```

### 7.2 本地脚本(Windows)

```powershell
# scripts/coverage.ps1
$env:LLVM_PROFILE_FILE = "target/coverage/rgs-%p-%m.profraw"
cargo build --workspace
cargo test --workspace
cargo llvm-cov report --summary-only
```

---

## 8. 不在范围(Out of Scope)

- ❌ 5 域 gRPC 联调(等 v0.3 + 真实部署)
- ❌ Load test(独立 harness)
- ❌ Fuzz test(独立 v0.5 计划)
- ❌ Mutation test(独立 v1.0 计划)
- ❌ 性能 benchmark(独立 v1.0 计划)

---

## 9. 时间线

| 阶段 | 工期 | 截止 | 主要工作 |
|---|---|---|---|
| **P0** | 0.5 day | T+0.5 | 基线 + 工具 |
| **P1** | 2 days | T+2.5 | shared-platform 100% + cluster-ops 90% |
| **P2** | 3 days | T+5.5 | 5 域 + rgs-testkit 90% |
| **P3** | 1 day | T+6.5 | 辅助 crate 100% |
| **总** | **6.5 days** | T+6.5 | workspace > 90% |

---

## 10. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008)| 初版:12 crate 评估 + 4 阶段计划 + 验收 + CI 集成 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态:无覆盖率策略
- v0.1 新增:12 crate 当前状态(313 测试) + 4 阶段计划(6.5 天) + 5 模式 + 5 验收段 + 风险

### A.2 已知缺口(实施开始后)

- 当前 cargo-tarpaulin / cargo-llvm-cov 未跑过(等 P0 完成)
- shared-platform src-ut 66 但 int-test 0,实际覆盖率待测
- 5 域 critical path 边界 case(OCC 冲突 / Saga 异常路径)需重点测

### A.3 引用链与证据

- per rgs-testkit 强约束 `pg_test_db.rs`
- per RGS-REV-009 V3 H-1(WF-1-55.31 retry)共识
- per DEC-008 一人公司 12 角色
- 修订历史代签新规则 per 2026-08-26 08:40 JST
