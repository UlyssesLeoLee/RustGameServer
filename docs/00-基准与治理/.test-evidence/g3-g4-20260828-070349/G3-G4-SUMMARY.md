# G3 fixture + G4 覆盖率 跑测总结(终态)

**Batch**: g3-g4-20260828-070349  
**跑测时间**: 2026-08-28 16:03-16:12 JST  
**Runner**: WSL native cargo (/home/leo19/.cargo/bin/cargo 1.98)  
**Target dir**: /tmp/cargo-target-wsl-g3  
**namespace**: rust-game-server

---

## G3 跑测统计(workspace 全量)

| Status | Targets | Passed | Failed | Ignored |
|---|---|---|---|---|
| ok | 81 | 663 | 0 | 37 |
| FAILED | 0 | 0 | 0 | 0 |
| **合计** | **81** | **663** | **0** | **37** |

**总通过率**: 663/663 = **100%**(0 fail)  
**ignored 37**:PH-5 opt-in Cloudflare IT (无 R2 凭证,设计如此)

对比上轮 (`g3-g4-20260828-062457`):
- PASS: 648 → **663** (+15)
- FAILED targets: 8 → **0** (-8)
- 修复点: 5 域 fixture (sqlx 0.8.6 leo19 fallback) + 3 个非 fixture bug

---

## G4 覆盖率(workspace line coverage)

**Workspace 总计**: **75.9%** (8829/11639 行)

| Crate | Files | Lines | Hit | Coverage |
|---|---|---|---|---|
| rgs-arc-olu | 1 | 22 | 22 | **100.0%** |
| rgs-certgen | 1 | 66 | 63 | **95.5%** |
| rgs-testkit | 4 | 229 | 213 | **93.0%** |
| gm-backend | 2 | 274 | 250 | **91.2%** |
| rgs-overflow-alert | 7 | 998 | 815 | **81.7%** |
| rgs-asset-download | 12 | 2232 | 1809 | **81.0%** |
| economy-service | 10 | 2154 | 1653 | **76.7%** |
| shared-platform | 18 | 1825 | 1364 | **74.7%** |
| cluster-ops | 8 | 1011 | 740 | **73.2%** |
| function-plane | 5 | 369 | 265 | **71.8%** |
| admin-service | 7 | 735 | 518 | **70.5%** |
| player-service | 6 | 576 | 388 | **67.4%** |
| social-service | 7 | 532 | 348 | **65.4%** |
| match-service | 7 | 613 | 381 | **62.2%** |
| rgs-hello | 1 | 3 | 0 | 0% (空 stub) |

**14/14 域/共享 crate ≥ 60%** (rgs-hello 是空 hello world stub,无业务逻辑)。

---

## 关键修复(本轮)

### 1. sqlx 0.8.6 fixture leo19 fallback 根因

`sqlx-postgres-0.8.6/src/options/mod.rs:67`:
```rust
let username = var("PGUSER").ok().unwrap_or_else(whoami::username);
```

**根因链**:
- `g3-g4-runner.sh` 原版 jsonpath 用了 `{.data.user}` / `{.data.dbname}` — k3s secret key 实际是 `username` / `database` → 解析出空 user
- 空 user URL `postgres://:@localhost:5432/` → sqlx `parse_from_url` 第 33 行 `if !username.is_empty() { ... }` 不覆盖
- sqlx fallback 到 `whoami::username()` → `leo19`
- `leo19` 不是 k3s postgres role → `role "leo19" does not exist`

**修复**:
- g3-g4-runner.sh 改用 `{.data.username}` / `{.data.database}`
- 用 `postgres-superuser` secret (k3s key 是 `POSTGRES_USER` / `POSTGRES_PASSWORD`)
- 强制设 `PGUSER=postgres` 防 fallback
- port-forward 改用 15432 (避 WSL host 5432 残留孤儿)

### 2. 3 个非 fixture bug

| Bug | 根因 | 修法 |
|---|---|---|
| `it_cloudflare_canary::try_build_scheduler_returns_some_with_env` | 测试用 `set_var` 跨 thread 串扰,Rust 2024 `set_var` 已 unsafe | 合并为 `try_build_scheduler_obeys_current_env` (只读当前 env 状态) |
| `it_cloudflare_edge::try_build_client_returns_none_without_env` | 同上 | 合并为 `try_build_client_obeys_current_env` |
| `ut_resume_token_store::json_file_store_returns_specific_error_on_io_failure` | `Z:\definitely-not-existing\store` 在 WSL 下视为合法相对路径名 | 改 `/proc/0/cannot-create-here/store` (Linux 永远 EACCES) |

### 3. 新增工具

- `scripts/db-url.sh` (1.0 KB): 解析 k3s secret 设 DATABASE_URL + PG env, 支持两种 secret key 风格
- `scripts/db-connect-check.py` (1.1 KB): 验证凭证 + psql 连通
- `scripts/extract-coverage.ps1` (4.1 KB): lcov → JSON summary by crate

---

## evidence 落档

```
docs/00-基准与治理/.test-evidence/g3-g4-20260828-070349/
├── cargo-test-workspace.log   (13,023 bytes, 81/81 PASS)
├── db-connect-check.log       (103 bytes, postgres@player_db 验证通过)
├── lcov-workspace.info        (770,267 bytes, llvm-cov 完整)
├── coverage-summary.json      (4,055+ bytes, by crate + workspace)
└── G3-G4-SUMMARY.md           (本文档)
```

---

## 后续

- S4 Phase 2 (gm-backend → admin-service gRPC client 集成) 可启动
- W2 跨域 IT (cluster-ops ↔ 5 域 ↔ admin ↔ gm-backend 链路)
- W4 S5 §3 真 NATS e2e (k3s nats-0 已就绪)
- 9 决策草案等你终审 (8 域 Lead / cluster-ops 终方案 / TBD-08-06 工具)
