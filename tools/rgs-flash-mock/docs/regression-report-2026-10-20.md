# RGS 并行回归报告 — regression-2026-10-20-dev-gate

生成时间: 2026-10-21 01:31:13 +09:00  ·  生成器: `tools/rgs-flash-mock/scripts/aggregate-regression.ps1`

## 判定

| 指标 | 值 |
|---|---|
| lanes | 7 |
| passed | 2410 |
| failed | 16 |
| ignored | 32 |
| 真实 rustc 编译错误 | 0 |
| 覆盖 crate | 31 / 31 |
| **VERDICT** | **PASS** |

判定依据:
- 全部 7 个 lane 无真实编译错误、无 code_defect；16 条失败均可归因为环境缺失（DATABASE_URL / Docker / k3s 集群）

### 失败归因分布

| 归因类别 | 条数 | 含义 |
|---|---:|---|
| `env_database_url_missing` | 14 | sqlx `#[pg_test]` setup 阶段 panic（`DATABASE_URL` 未设置），未进入业务断言 |
| `env_docker_unavailable` | 2 | testcontainers 需要 Docker，daemon 不可用 |

## Per-Lane

| lane | crates | passed | failed | compile err | elapsed | 归因 |
|---|---|---:|---:|---:|---:|---|
| `reg-platform` | 2 | 298 | 0 | 0 | 318s | - |
| `reg-player` | 5 | 373 | 3 | 0 | 114s | env_database_url_missing=3 |
| `reg-economy` | 4 | 273 | 0 | 0 | 2391.8s | - |
| `reg-social` | 4 | 195 | 3 | 0 | 1564.5s | env_database_url_missing=3 |
| `reg-match` | 5 | 408 | 3 | 0 | 1381s | env_database_url_missing=3 |
| `reg-admin` | 8 | 475 | 3 | 0 | 3549.9s | env_database_url_missing=3 |
| `reg-tools` | 3 | 388 | 4 | 0 | 268.8s | env_database_url_missing=2, env_docker_unavailable=2 |

## 失败用例清单

| lane | 用例 | 归因 | 证据 |
|---|---|---|---|
| `reg-player` | `player_fixture_inserts_and_reads_back_in_real_pg` | `env_database_url_missing` | thread 'player_fixture_inserts_and_reads_back_in_real_pg' (54744) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) \| note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace |
| `reg-player` | `player_fixture_builder_customizes_name_and_level` | `env_database_url_missing` | thread 'player_fixture_builder_customizes_name_and_level' (46448) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-player` | `outbox_check_constraint_rejects_invalid_status` | `env_database_url_missing` | thread 'outbox_check_constraint_rejects_invalid_status' (56732) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) \| failures: |
| `reg-social` | `social_fixture_creates_guild_in_real_pg` | `env_database_url_missing` | thread 'social_fixture_creates_guild_in_real_pg' (44996) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-social` | `social_fixture_builder_customizes_message` | `env_database_url_missing` | thread 'social_fixture_builder_customizes_message' (20380) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-social` | `outbox_check_constraint_rejects_invalid_status` | `env_database_url_missing` | thread 'outbox_check_constraint_rejects_invalid_status' (20412) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) \| note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace |
| `reg-match` | `outbox_check_constraint_rejects_invalid_status` | `env_database_url_missing` | thread 'outbox_check_constraint_rejects_invalid_status' (47940) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-match` | `match_fixture_builder_customizes_score_and_status` | `env_database_url_missing` | thread 'match_fixture_builder_customizes_score_and_status' (50380) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-match` | `match_fixture_inserts_and_reads_back_in_real_pg` | `env_database_url_missing` | thread 'match_fixture_inserts_and_reads_back_in_real_pg' (62892) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) \| note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace |
| `reg-admin` | `admin_fixture_builder_customizes_action_and_target` | `env_database_url_missing` | thread 'admin_fixture_builder_customizes_action_and_target' (70028) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-admin` | `outbox_check_constraint_rejects_invalid_status` | `env_database_url_missing` | thread 'outbox_check_constraint_rejects_invalid_status' (75552) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-admin` | `admin_fixture_creates_audit_log_in_real_pg` | `env_database_url_missing` | thread 'admin_fixture_creates_audit_log_in_real_pg' (53856) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) \| note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace |
| `reg-tools` | `init_test_db_returns_url` | `env_docker_unavailable` | thread 'init_test_db_returns_url' (68240) panicked at crates\rgs-testkit\tests\fixture_extended_test.rs:177:54: \| called `Result::unwrap()` on an `Err` value: failed to create a container: Error in the hyper legacy client: client error (Connect) \| note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace |
| `reg-tools` | `pg_test_macro_re_export_works` | `env_database_url_missing` | thread 'pg_test_macro_re_export_works' (15060) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) \| note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace |
| `reg-tools` | `pg_test_db_smoke_connects_and_selects_one` | `env_database_url_missing` | thread 'pg_test_db_smoke_connects_and_selects_one' (73860) panicked at E:\DevCache\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\sqlx-postgres-0.8.6\src\testing\mod.rs:96:44: \| DATABASE_URL must be set: EnvVar(NotPresent) |
| `reg-tools` | `fixture_init_test_db_returns_url` | `env_docker_unavailable` | thread 'fixture_init_test_db_returns_url' (34508) panicked at crates\rgs-testkit\tests\self_test.rs:161:5: \| assertion failed: url.is_ok() \| failures: |

## 覆盖的 crate

覆盖 31 / 31 个 workspace member。

## 已知缺口（缺标比错标安全）

- **无本地 Postgres** — 14 条 sqlx #[pg_test] fixture 用例失败（去重后 11 个用例名，同名用例分布在多个 crate）在本机必然 fail（DATABASE_URL must be set），其真实通过与否无法在本机判定
- **Docker Desktop 停止** — 2 条 testcontainers 类用例必然 fail（daemon pipe //./pipe/dockerDesktopLinuxEngine 不存在，服务 com.docker.service = Stopped）
- **k3s/containerd 崩溃循环** — L3 ST `e2e-smoke.ps1` 12 probe 全挂；`/run/k3s/containerd/containerd.sock` 缺失，node 对象 `ulyssespc` 已删除，kubelet heartbeat 停在 `2026-10-18T08:52:44Z`
- **`cargo fmt --all -- --check` 仍红**（620 处 diff / 15 文件）— CI 仍卡在 fmt 步骤；`.github/workflows/rust-ci.yml` L81-82 记录 fmt 与 clippy doc-list 规则互相拉扯，未做机械重排
- **workspace clippy lane 未跑** — `cargo clippy --workspace --all-targets --all-features --locked` 需全量冷构建，与 cargo test 指纹不通用
- **`Cargo.lock` 被 `.gitignore` 忽略且未追踪** — 新 worktree 必须先播种，否则 `--locked` 直接失败
- **`tools/rgs-flash-mock` 是独立 workspace** — 根 workspace 的 31 crate 全覆盖，但 mock 项目自身的 `src/` `tests/`（含本仓新增的 `saga_verify.rs` / `saga_story_chain.rs`）**不在**本批次 7 个 lane 覆盖范围内，由其自有 CI（`.github/workflows/flash-mock-ci.yml`）覆盖，本批次未验证
- **并行链接会撞 `LNK1104`** — 同一 target dir 下 `-Jobs 4` 并行链接 test binary 时出现 `LINK : fatal error LNK1104: 无法打开文件 ...exe`（link.exe 输出句柄/内存竞争，非代码问题）；串行 `-Jobs 1` 重试即全部链接成功。链接密集的 lane 建议 `-Jobs 1`

## 被测基线

本批次 lane 全部跑在 `4f773122`（批次启动时的 dev tip）之上，判定「lane 结果 == dev 代码结果」的依据是 git 实证而非推断：

1. **lane 分支不含代码文件**
   `git diff --stat 4f773122..agent/nim3/ulys-257` 只有 `reply.md`（+16 行），
   零代码文件 → 在 lane tip 跑回归 == 在 dev 代码基线跑回归，一次运行同时充当
   「合并候选验证」+「dev 代码基线」。

2. **批次内发现并修复 3 个既有缺陷，均已在其影响的 lane 复验**

   | 缺陷 | 根因 | 复验 lane | 修复前 → 修复后 |
   |---|---|---|---|
   | network-gateway 编译失败 | `rustler::init!` 误用字节串字面量（`rustler_codegen-0.36.2/src/init.rs` L96 `concat!(#name, "\0")`） | reg-admin | lane 从编译中止 → 475 passed / 3 failed |
   | battle-service 4 个 `w41_*` 用例失败 | 丢弃 `battle_init` 返回的随机 `battle_id`，硬编码 `"x"` | reg-match | 404/7 → 408/3 |
   | rgs-testkit 掉线自愈失效 + `issue()` 契约漂移 | `stop_all()` 用 `drain(..)` 抽空 registry，`relogin_offline()` 却要从中查找 | reg-tools | 386/6 → 388/4 |

   未受影响的 lane 不需要重跑：`stop_all` / `spawn_with_stagger` / `relogin_offline`
   在全仓**仅** rgs-testkit 自身 bot 模块使用（无外部调用方），
   `bot_smoke.rs` 是 rgs-testkit 自己的集成测试文件；network-gateway 与
   battle-service 各自只被 reg-admin / reg-match 覆盖。

3. **合并后代码状态不变**
   合并 commit `5e3c46ab` 只引入 `reply.md`（非代码）。
   `git diff --stat 4f773122..5e3c46ab -- crates/` 的结果恰好是上述 3 个修复
   涉及的 4 个文件，无其他 Rust 变动 → 本报告的 lane 结果仍然描述 dev 当前
   的 Rust 代码状态。

4. **合并完整性**
   `git diff dev..agent/nim3/ulys-257` 为空 → lane 分支已 100% 含入 dev。
   rebase 保留了那条空的 baseline 提交（`--keep-empty`，680d864e），
   内容差异仍只有 `reply.md`。

## 本批次说明

### 本批次做了什么

- **7 个 lane 并行**覆盖根 workspace 全部 31 个 crate，按 crate 分片、每 lane 一个
  受管 worktree（`scripts/worktree.ps1`）+ 一个隔离 `CARGO_TARGET_DIR`
  （`E:\DevCache\cargo\rgs-reg-<lane>`）+ 一个子代理。
- **归因口径**：每条失败必须由 log 原文证据支撑；无法归因的记 `code_defect` /
  `unclassified`，**不扫进环境类**（缺标比错标安全）。本批次 `code_defect` 0 条。

### 发现的 3 个既有缺陷（均已修复并复验）

1. **`network-gateway` 编译失败**（阻断整个 31 crate 的 CI 门禁）
   `crates/network-gateway/src/nif_demo.rs` 的 `rustler::init!(b"rgs_nif\0", ...)`
   用了字节串字面量。根因从 vendored 源码确认：`rustler_codegen-0.36.2/src/init.rs`
   L96 `name: concat!(#name, "\0")`，`concat!` 不接受字节串。
   该缺陷在 dev 上已存在，被更早一步的 `cargo fmt --check` 失败掩盖 —— CI 一直
   红在 fmt 步骤，从未走到暴露它的 clippy/test 步骤。

2. **`battle-service` 4 个 `w41_*` 用例必然失败**（测试自身缺陷，非产品缺陷）
   用例丢弃 `battle_init` 返回的随机 `battle_id`（`service.rs` L1840
   `Uuid::new_v4()`）却硬编码 `"x"` 调 `battle_start`（L1887 按 id 精确查表
   → `BattleNotFound`）。同文件通过的 `battle_start_advances_turn` 写法正确，
   4 处已对齐。

3. **`rgs-testkit` 掉线自愈路径形同虚设**（真实产品缺陷）
   `stop_all()` 用 `self.bots.drain(..)` 抽空 registry，而 `relogin_offline()`
   要按 id 从同一个 registry 找回 bot 重新 `start()` —— 两者互相矛盾，
   `recovered` 恒为 0。git 实证该缺陷自 `16bfb951` 引入，从未绿过。
   同一 lane 还暴露 `issue()` 的契约漂移（`9788404f` 改成恒返 `ok:false` 后
   测试断言未同步），已改为守 fail-closed 契约 + 补一个以缺口命名的
   `#[ignore]` 用例，集群恢复后解除 ignore 即可变红提醒能力闭环。

### 过程记录（供后续复跑参考）

- **子代理传输层不稳定**：8 次子代理派工中 6 次会话中断
  （`net::ERR_CONNECTION_CLOSED` / `net::ERR_CONNECTION_RESET`），与任务内容无关。
  降为 2 并发 + 主会话兜底重跑后全部完成。
- **并行链接会撞 `LNK1104`**：同一 target dir 下 `-Jobs 4` 并行链接 test binary 时
  出现 `LINK : fatal error LNK1104: 无法打开文件 ...exe`（link.exe 输出句柄/内存
  竞争）。串行 `-Jobs 1` 重试即全部链接成功 —— 属环境资源问题，不是代码缺陷。
- **RAM 极度紧张**（整机空闲 1.8–3.8 GB）：`cargo` 默认 `-j 24` 会把 rustc 挤爆，
  表现为 `can't find crate for std` 这类**假失败**。每 lane 限流 `-Jobs 3~5`、
  最多 2–3 lane 并发、stagger 启动才跑得动。
- **`Cargo.lock` 被 `.gitignore` 忽略且未追踪**：新 worktree 必须先播种，
  否则 `--locked` 直接报 `cannot create the lock file`。
- **cargo 指纹与 `-p` 集合绑定**：`-p` 集合不同 → feature unification 不同 →
  依赖全量重编。复用暖缓存必须保持相同 `-p` 集合与相同源路径。

## 复现

```powershell
pwsh -NoProfile -NonInteractive -File tools/rgs-flash-mock/scripts/aggregate-regression.ps1 -Batch regression-2026-10-20-dev-gate
```

单 lane 复跑（warm cache，链接密集的 lane 建议 `-Jobs 1`）：

```powershell
pwsh -NoProfile -NonInteractive -File tools/rgs-flash-mock/scripts/run-regression-lane.ps1 `
     -Lane reg-tools -Crates rgs-testkit,rgs-asset-download,rgs-overflow-alert `
     -TargetDir E:\DevCache\cargo\rgs-reg-reg-economy -Jobs 1
```

