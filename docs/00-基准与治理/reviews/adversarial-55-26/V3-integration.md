# V3 集成+测试审查报告 (WF-1-55.26 5 commit)

## 元数据
- 审查范围: 1b30878..cc888b5
- 审查维度: Integration & Testing
- 审查者: V3
- 日期: 2026-08-23
- Worktree: D:/adversarial-55-26-V3
- Target: D:\target-adversarial-V3

---

## CRITICAL (0)

无 CRITICAL 阻断项。5 commit 通过编译、所有测试通过、release build 成功。

---

## HIGH (2)

### [H-1] DC-1 4 测试均为 InMemory unit test,缺真实 DB 集成 + 并发竞争覆盖
- 文件:
  - `crates/economy-service/src/saga_orchestrator.rs:838-1019` (DC-1.1~1.4 4 test)
  - `crates/economy-service/src/service.rs:550-700` (CC-4 2 test)
- 证据: `make_env()` helper 第 474-511 行 100% 使用 `InMemorySagaRepository` / `InMemoryReservationRepository` / `InMemoryAccountRepository`,无 `PgRepository` 实例化、无 `#[sqlx::test]` 宏、无 docker-compose 启动的 PG fixture
- 影响:
  - 55.23 economy main.rs:104-136 的 30s `list_running + resume` 轮询主路径,在 in-memory 环境下覆盖不到 sqlx 重试/连接池耗尽/事务回滚等真 DB 场景
  - OCC 冲突测试 (`apply_atomic_with_reservation_occ_conflict_cleans_reservation`) 是通过手动修改 in-memory `acc_repo.inner.lock()` 的 `stored.version = original_version + 99` 模拟,与 PG 真 OCC (UPDATE ... WHERE version = ? 0 row) 行为不完全等价
  - 无并发 resume() 同 saga_id 的竞争测试,生产中 30s 轮询 + k8s 多副本可能两个 worker 同时 resume 同一 saga
- 建议修复: 55.x 范围内加 1 个 `#[sqlx::test]` 真 DB 集成测试(resume happy path + 并发 resume 同一 saga_id),覆盖 PG OCC + 事务边界; 56.x 加 load test (k6/rusoto 模拟 100+ 并发)

### [H-2] AC-1 服务端 MTLS_BYPASSED_TOTAL 6 个 main.rs 各自 private static,缺公共读出函数
- 文件: `crates/{admin,cluster-ops,economy,match,player,social}-service/src/main.rs` (每个都有 `static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);`)
- 证据:
  - 服务端 6 个 static 都是 file-scope private,无 `pub fn` 暴露,与 shared-platform `channel.rs:82-87` 的 `static MTLS_BYPASSED_TOTAL` + `pub fn mtls_bypassed_total() -> u64` 模式**不对称**
  - shared-platform 有 2 个 test (`build_channel_with_require_tls_false_increments_bypass_counter` + `build_insecure_channel_emits_warning`) 验证 counter +1,服务端 6 main.rs 没有任何 test 验证 counter 行为
  - commit message 承诺"监控集成 (Prometheus exporter / scrape handler 暴露为 `mTLS_bypassed_total`) 由后续任务处理",意味着 55.26 范围内 6 个 static 是**死代码** (写不读)
- 影响:
  - 服务端 mTLS bypass 事件当前无法被监控抓取,违反"显式 opt-out 必须可观测"安全原则
  - verify-A AH-4 任务在 56.x 范围,如未及时跟进,bypass 状态对 SRE 不可见
- 建议修复: 在 shared-platform 加一个 `pub fn server_mtls_bypassed_total() -> u64` + 对应 `static`,6 个 main.rs 调它; 或本次 PR 至少加 6 个 unit test 验证 `MTLS_BYPASSED_TOTAL.fetch_add(1, ...)` 行为

---

## MEDIUM (4)

### [M-1] CC-4 修复仅覆盖 `apply_atomic_with_reservation` 单一调用路径,saga_orchestrator 直调 `accounts.apply_atomic` 不在保护范围
- 文件:
  - `crates/economy-service/src/service.rs:86` (新增 `apply_atomic_with_reservation`)
  - `crates/economy-service/src/saga_orchestrator.rs:277, 327, 432` (`self.accounts.apply_atomic(&account, &entry).await?;` 3 处)
  - `crates/economy-service/src/service.rs:208, 256` (credit/debit 方法内部)
- 证据: 5 处裸 `apply_atomic` 调用均不经过 `apply_atomic_with_reservation` 的 dangling reservation cleanup
- 影响:
  - 当前所有裸 `apply_atomic` 调用前**未**先 `reservations.save(...)`,所以没有 dangling reservation 风险(实际安全)
  - 但如果未来 56.x 引入新的 `apply_atomic` 调用点 + reservation 写入(顺序不当时),CC-4 修复将不会自动保护该路径
  - 没有 deprecation marker (`#[deprecated]`) 提示新代码应使用 `apply_atomic_with_reservation`
- 建议修复: 56.x 给裸 `apply_atomic` 加 `#[deprecated(note = "use apply_atomic_with_reservation for saga path")]`,或文档明确两条 API 的边界

### [M-2] admin-service outbox migration 文件名与首行注释不一致 (0003 vs 0002)
- 文件: `crates/admin-service/migrations/0003_outbox.sql:1` vs `crates/{cluster-ops,match,player,social}-service/migrations/0002_outbox.sql:1`
- 证据:
  - admin 文件名是 `0003_outbox.sql` (因为 admin 多了 `0002_audit.sql`),首行注释却写 `-- admin-service migration 0002_outbox`
  - economy 文件名是 `0003_outbox.sql`,首行注释写 `-- economy-service migration 0003_outbox` (正确)
  - 其他 4 域文件名 `0002_outbox.sql`,注释也是 `0002_outbox` (正确)
- 影响: 文档与文件名不一致,后续维护者/审计员可能误判迁移顺序,属 LOW 但应修正
- 建议修复: admin 的注释改为 `0003_outbox`

### [M-3] clippy --workspace --all-targets 受 rgs-hello 预存 deprecation lint 阻断,55.26 验证脚本需更新
- 文件: `crates/rgs-hello/src/main.rs` (3 行 hello world) + 全局 clippy 1.98 弃用 lint 名称
- 证据:
  - `cargo clippy --workspace --all-targets -- -D warnings -A pedantic -A nursery -A cargo` 失败:`error: lint name 'pedantic' is deprecated` (在 rgs-hello, 因 1.98 改名 `clippy::pedantic`)
  - 0240d4f commit message 中"验收"声明 `cargo clippy --no-deps -p 6 域 --all-targets -D warnings -A pedantic -A nursery -A cargo 0 warning` 实际能通过(只跑 6 域 + 排除 rgs-hello),但缺 rgs-certgen 3 个 pre-existing error
  - 改用 `-A clippy::pedantic -A clippy::nursery -A clippy::cargo` 现代语法可让 6 域 + shared-platform + rgs-testkit 通过 (我已验证),但 rgs-hello + rgs-certgen 仍 fail
- 影响: 55.26 验证脚本写法 (老式 `-A pedantic`) 在 clippy 1.98 上不能直接复现 PASS,新人/CI 用同样命令会困惑
- 建议修复: 56.x 统一所有 verify 脚本为 `-A clippy::pedantic` 形式;或在 `clippy.toml` 加 `[lints]` 节集中定义

### [M-4] RGS-REV-008 报告 §verify-D DC-1 与本审查的测试质量评估有偏差
- 证据: RGS-REV-008 将 DC-1 列为 CRITICAL 修复项,本审查认为**测试覆盖了 4 入口状态正确性,但未覆盖真 DB 集成和并发场景**。在 saga 跨进程崩溃恢复 + 30s 轮询的架构下,**仅 in-memory 单线程 unit test 不足以证明生产语义** (尤其 OCC 在 PG 端的真实行为)
- 影响: 误判风险已闭合度。55.26 merge 后,DC-1 仍可能在 56.x 暴露真 DB 集成问题
- 建议修复: 56.x WF 排期内插入 `#[sqlx::test]` + 并发竞争 case 2 项

---

## LOW (3)

### [L-1] doc test 总数仅 2,`json_logging.rs` 修后仅 1 个独立 doctest,全工程 doctest 覆盖密度低
- 证据: `cargo test --doc --workspace` 总 2 passed (shared-platform 1 + rgs-certgen 1)。55.26 修复的 `json_logging.rs:11-13` 修后 doctest 编译通过但**无实际断言**(只是 `init_json_logging` 调用演示)
- 建议: 56.x 在 `apply_atomic_with_reservation` / `SagaOrchestrator::resume` 等关键 API 加 usage example doctest

### [L-2] 6 域 main.rs mTLS 改动 33 → 31 行,行数一致;但缺**集成 test** 验证 fail-closed 行为
- 证据: 6 域 main.rs 的 `load_server_tls_config` 失败路径改用 `.context()?` 上抛 → 进程退出 1,无任何 test 模拟"PEM 文件不存在"启动场景断言退出码
- 建议: 56.x 加一个 `assert_cmd` 集成测试,启动 binary 缺 cert dir,断言非 0 退出码 + stderr 含 "mTLS config load failed"

### [L-3] housekeeping f9bf84f 仅修 1 行,未顺手修 `rgs-certgen` 3 个 clippy error
- 证据: `cargo clippy -p rgs-certgen --all-targets` 报 3 个 error (`let-binding has unit value` / 2x `&PathBuf` instead of `&Path`),commit message 自承"rgs-certgen 3 个错误仍属 55.x 范围外,56.x 处理"
- 建议: 56.x 一次性清掉,避免 56.x 范围扩散

---

## 6 域改动一致性矩阵

| 域 | AC-1 mTLS | CC-3 CHECK | 一致性 | 备注 |
|---|---|---|---|---|
| admin-service | ✅ | ✅ | ✅ 完全 | 文件名 0003 + 注释 0002 不一致 (M-2) |
| cluster-ops | ✅ | ✅ | ✅ 完全 | service_impl 构造参数 (nodes, flags) 不同 (预期) |
| economy-service | ✅ | ✅ | ✅ 完全 | 含 Saga 启动 doc + Duration import (55.x 既有差异) |
| match-service | ✅ | ✅ | ✅ 完全 | service_impl (matches, participants) (预期) |
| player-service | ✅ | ✅ | ✅ 完全 | tracing 初始化模式不同 + 末尾 "启动 tonic server" 而非 binding log (预存差异) |
| social-service | ✅ | ✅ | ✅ 完全 | service_impl (guilds, members) (预期) |

**判定**: mTLS 33 → 31 行 hunk 6 域完全一致(已用 Python 严格 diff 验证,仅 4 域仅 service_impl 构造参数不同,economy/player 各自有预存结构性差异)。CHECK 约束完全相同(7 个值 6 域 diff 0 差异)。

---

## 测试覆盖矩阵

| commit | unit test | integration test | doc test | stress/concurrency test |
|---|---|---|---|---|
| 1b30878 (CC-3) | ❌ 无 | ❌ 无 (SQL CHECK 无 Rust 测) | ❌ N/A | ❌ |
| a950b46 (CC-4) | ✅ 2 new (service.rs:550-700) | ❌ 无 (InMemory) | ❌ | ❌ 无并发 apply_atomic |
| 0240d4f (AC-1) | ❌ 无 (commit 自承监控集成属后续任务) | ❌ 无 fail-closed 启动测试 | ❌ | N/A |
| f9bf84f (housekeeping) | ❌ | ❌ | ✅ 1 (json_logging) 编译通过 | N/A |
| cc888b5 (DC-1) | ✅ 4 new (saga_orchestrator.rs:838-1019) | ❌ 无 (InMemory) | ❌ | ❌ 无并发 resume |

**关键观察**:
- 5 commit 净增 6 个 unit test (CC-4 2 + DC-1 4) + 修复 1 个 doc test 编译
- **0 个真实 DB 集成测试** (无 `#[sqlx::test]` / `tests/` 目录 fixture)
- **0 个并发/竞争测试**
- 1 个 doc test 仅是 compile pass,无断言

---

## 验证结果

| 命令 | 结果 | 耗时 |
|---|---|---|
| `cargo test --workspace` | **220 passed / 0 failed** (含 DC-1 4 new + CC-4 2 new) | ~30s |
| `cargo test --doc --workspace` | **2 passed / 0 failed** (json_logging 编译 + rbac 1) | ~10s |
| `cargo clippy --workspace --all-targets -- -D warnings -A pedantic -A nursery -A cargo` | **FAIL** (rgs-hello 弃用 lint 阻断,pre-existing) | - |
| `cargo clippy -p 6域 -p shared-platform -p rgs-testkit --no-deps -- -D warnings -A clippy::pedantic ...` | **0 warning** ✅ | ~14s |
| `cargo build --release` | **成功** (2m 03s) | 2m 03s |
| `cargo build -p rgs-certgen` | **FAIL 3 error** (let-binding has unit value / 2x &PathBuf, pre-existing 56.x) | - |

逐 crate 测试分布 (cargo test --workspace):
- admin-service: 18
- cluster-ops: 16
- economy-service: **42** (38 + DC-1 4)
- match-service: 16
- player-service: 24
- social-service: 15
- shared-platform: 9
- rgs-testkit: 78
- doc tests: 2
- 合计: **220 passed / 0 failed**

---

## 结论

- **是否可合并**: **需修后合并 (CONDITIONAL PASS)**
- **阻塞项**: 无 CRITICAL,5 commit 编译通过 + 所有测试通过 + release build 成功
- **必收 (56.x WF 排期优先)**: 
  - H-1 真 DB 集成测试 + 并发 resume
  - H-2 服务端 MTLS_BYPASSED_TOTAL 公共读出函数 (or test 覆盖)
- **建议收 (56.x 范围)**: 
  - M-1 `apply_atomic` 加 `#[deprecated]` 或文档
  - M-2 admin 注释文件名修正
  - M-3 clippy 验证脚本语法升级
  - M-4 RGS-REV-008 verify-D 评估标准更新
  - L-1~L-3 文档/doctest/housekeeping 收尾
- **总评**: 5 commit 的**单点正确性**扎实 (单元测试 + 编译 + 静态检查),**集成层面**有显著缺口 (无真 DB 测、无并发测、无 fail-closed 启动测),属"55.x 收尾 OK,56.x 接续补"状态