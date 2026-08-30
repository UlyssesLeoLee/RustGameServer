# G3 fixture + G4 覆盖率 跑测总结

**Batch**: g3-g4-20260828-062457  
**跑测时间**: 2026-08-28 15:24-15:26 JST  
**Runner**: WSL native cargo (/home/leo19/.cargo/bin/cargo v1.98)  
**Target dir**: /tmp/cargo-target-wsl-g3  
**namespace**: rust-game-server

---

## 跑测统计

| Status | Targets | Passed | Failed | Ignored |
|---|---|---|---|---|
| ok | 73 | 617 | 0 | 28 |
| FAILED | 8 | 31 | 17 | 9 |
| **合计** | **81** | **648** | **17** | **37** |

**总通过率(按 test case)**: 648 / (648+17) = **97.4%**

---

## 8 个 FAIL Target 归因

### A. sqlx fixture auth 失败(5 域,13 测试 fail)

| 域 | target | 失败测试数 | 根因 |
|---|---|---|---|
| player-service | integration_player_basic | 4 | sqlx 0.8.6 testing/mod.rs:226 用 OS user `leo19` 试图连 player_db |
| economy-service | integration_outbox | 3 | 同上,试图用 leo19 创 outbox 测试 DB |
| match-service | integration_match_basic | 2 | 同上 |
| social-service | integration_social_basic | 3 | 同上 |
| admin-service | integration_admin_basic | 1 | 同上 |

**统一根因**:
- `g3-g4-runner.sh:69-71` 解析 `player-db-credentials` secret 得 `USER=player_user DB=player_db PASSWORD=***`
- 第 93 行 `export DATABASE_URL="postgres://player_user:***@localhost:5432/player_db"` 正确
- 但 `cargo test --workspace` 跑 `integration_*_basic` 时,这些 target 的 sqlx `#[pg_test]` fixture 在 mod.rs:226 试图连 DB 时,实际 url 可能没传透或 fallback 到 OS user `leo19`
- 报错:`password authentication failed for user "leo19"` (sqlx-core 0.8.6 testing/mod.rs:226:14)

**注意**:port-forward 实际**已经成功**(5432 端口已被 dnsmasq-user 进程占用,因为 k3s 把 postgres 5432 hostPort 直接暴露到 WSL host,不需要 port-forward)。新跑时 port-forward 报 "address already in use" 不影响 cargo test。

### B. rgs-asset-download 3 个 test bug(非 fixture)

| target | 测试 | 根因 |
|---|---|---|
| it_cloudflare_canary | try_build_scheduler_returns_some_with_env | `try_build_scheduler` 还需要更多 env 变量(测试只 set 4 个) |
| it_cloudflare_edge | try_build_client_returns_none_without_env | `try_build_client` 实际行为:缺 env 时**不**返回 None,可能 fallback 到默认 client |
| ut_resume_token_store | json_file_store_returns_specific_error_on_io_failure | 测试用 `Z:\definitely-not-existing\store`(Windows 风格路径),WSL 视为合法相对路径,实际能创建,无 IO 错 |

### C. 5 域 fixture 共享 player_db 问题

`g3-g4-runner.sh:52` 注释: `#[pg_test] 会自动 create per-test DB, 只用 player_db 即可`

但 rgs-testkit `pg_test_db.rs` 是否**真的**能跨 5 域共享 player_db?如果 fixture 内部硬编码了 user 跟 db name,5 域 testsuite 会相互撞车。

需查 `crates/rgs-testkit/src/pg_test_db.rs` 实际行为。

---

## 3 个非 fixture 失败处置建议(3 选 1)

1. **修测试**(短期):`it_cloudflare_canary` 补 set 完整 env;`it_cloudflare_edge` 改测试期望;`ut_resume_token_store` 改用 `///` POSIX 风格错误路径
2. **修实现**(若 bug):让 `try_build_scheduler` / `try_build_client` 行为匹配测试期望
3. **标注 `#[ignore]`**(临时):WIP,后续 issue 跟踪

---

## 9 fail 域的决策草案

- **方案 X (推荐)**:修 rgs-testkit 让 `pg_test` 接受 `DATABASE_URL` 显式 user/db,不强求 OS user;5 域 fixture 用各自 user/db 但共享一个 postgres pod
- **方案 Y**:只跑 5 域中的 player 域(因 rgs-testkit 强约束),其他 4 域标记为 `#[ignore] = "require multi-tenant fixture (TBD-XX)"`
- **方案 Z**:5 域 fixture 都用 superuser (`postgres` user) 创建测试 DB,牺牲多租户隔离

需 Ulysses 拍板。
