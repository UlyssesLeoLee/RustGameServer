# V2 正确性审查报告 (WF-1-55.26 5 commit)

## 元数据
- 审查范围: `1b30878..cc888b5` (5 commit)
- 审查维度: Correctness (资金一致性 / Saga 崩溃恢复 / 事务边界 / 状态机正确性)
- 审查者: V2 (verifier sub-agent, branch session)
- 日期: 2026-08-23
- Worktree: `D:/adversarial-55-26-V2` (独立 worktree, `CARGO_TARGET_DIR=D:\target-adversarial-V2`)
- 复审: 独立 git 验证 5 commit 的实际代码改动;实际编译并跑完 209 个单元测试 + clippy 6 域

---

## CRITICAL (2)

### [CC-4-DEAD-001] `apply_atomic_with_reservation` 是死代码 — CC-4 修复未触及生产路径
- **文件**: `crates/economy-service/src/service.rs:86-160` (修复目标) vs `crates/economy-service/src/saga_orchestrator.rs:248-289` (生产路径)
- **证据**:
  - 修复后的 helper `apply_atomic_with_reservation` 在 service.rs:86 起,签名 `pub async fn apply_atomic_with_reservation(&self, account: &Account, ...)`。
  - `grep -r "apply_atomic_with_reservation" crates/` 显示 8 个匹配,**全部是测试调用** (service.rs:487, 536, 580, 660 + 4 个定义/doc 行) — **0 个生产代码调用**。
  - 生产路径 `ReserveHandler::execute` (saga_orchestrator.rs:248-289) **未使用** 该 helper,直接 inline 实现:
    - L252-253: 构造并 `self.reservations.save(&r).await?` — reservation 落库
    - L257-265: `try_debit` 失败路径使用 `let _ = self.reservations.delete_by_id(r.id).await;` (L259) — **静默吞错,无 tracing::warn**
    - L277: `self.accounts.apply_atomic(&account, &entry).await?` — OCC 失败时,reservation 已落库且**无清理**
  - service.rs:7-10 doc 注释自称"给 SagaOrchestrator 的 ReserveHandler/ConfirmHandler 用",**与实际不符**。
- **影响**:
  - CC-4.1 资金幻影 (verify-C §4.4 + 55.12 bug) 仍然存在:ReserveHandler.execute → apply_atomic OCC 失败 → step 标 Failed → 触发 compensate → `ReserveHandler.compensate` (L291-342) 找到 dangling reservation → `account.credit(refund_amount)` (L316) → `apply_atomic` 写入 +amount (L327) → 凭空造钱。
  - 修复点的 helper 通过 2 个新 test (OCC + InsufficientFunds),**但生产代码路径完全未变**,helper 是 dead code。
  - 测试在 InMemoryAccountRepository 跑,**OCC 失败路径永不触发** (verify-C 报告原话),所以即使是 helper 自身也只走了 happy path 测试,**OCC 失败真在生产 PG 上才会发生**。
- **建议修复**:
  - 删除 `apply_atomic_with_reservation` 这个未使用的 helper;或
  - **真正修复 `ReserveHandler::execute` (saga_orchestrator.rs:248-289)**:
    1. L259 `let _ = ...` 改为 `if let Err(e) = ... { tracing::warn!(...) }` (与 helper 一致)
    2. L277 改为 `match self.accounts.apply_atomic(&account, &entry).await { Ok(_) => Ok(()), Err(e) => { /* cleanup reservation */; Err(e) } }`
  - 同步修复 `ConfirmHandler::execute` (L369-394) — 该函数也有类似问题 (虽然 confirm 不直接 OCC,但若未来加 try_debit 会同样踩坑)。
  - **写真实 PG integration test** 用 `PgAccountRepository` 注入 OCC 失败,验证生产路径。

### [CC-3-MIGRATION-001] CHECK 约束在已部署环境上**永不生效** — `CREATE TABLE IF NOT EXISTS` 静默跳过
- **文件**: 6 域 outbox migration (admin/cluster-ops/economy/match/player/social 各自 migrations/0002|0003_outbox.sql)
- **证据**:
  - 完整 diff 显示 CC-3 把 CHECK 约束**写在同一个** `CREATE TABLE IF NOT EXISTS outbox (...)` 块内 (例 economy-service/migrations/0003_outbox.sql:19)。
  - `git log -- crates/economy-service/migrations/0003_outbox.sql` 显示该文件由 55.17 commit `55af339` 首次创建(无 CHECK),1b30878 修改同文件追加 CHECK。
  - 在**已部署环境**中(55.17 已跑过、`outbox` 表已存在),`CREATE TABLE IF NOT EXISTS` 是 no-op,**CHECK 约束永不生效**;只有新部署(表不存在)才会随 CREATE 一起加 CHECK。
  - 没有任何 ALTER TABLE ADD CONSTRAINT 兜底。
- **影响**:
  - 报告 CC-3 (verify-C §4.3) 标的"CHECK 约束"对**生产环境实际无效**,只对 fresh DB 有效。
  - 6 域都中招:admin/cluster-ops/match/player/social 用 0002_outbox.sql,economy 用 0003_outbox.sql,模式相同。
- **建议修复**:
  - 选项 A(推荐):新增 0004 migration 文件 (或 6 域递增),内容为 `ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))`,与 1b30878 文件解耦,保证已部署环境升级时生效。
  - 选项 B:在 1b30878 文件内追加 `DO $$ BEGIN ... EXCEPTION ... ALTER TABLE outbox ADD CONSTRAINT ...; END $$;` 的幂等块。
  - 同理需要 6 域各一份(或在 shared-platform 抽 outbox schema builder)。

---

## HIGH (2)

### [CC-4-TEST-001] 2 个新 test 仅覆盖 helper 路径,未覆盖生产 `ReserveHandler::execute`
- **文件**: `crates/economy-service/src/service.rs:551-702` (2 新 test)
- **证据**:
  - `apply_atomic_with_reservation_insufficient_funds_cleans_reservation` (L551) 和 `apply_atomic_with_reservation_occ_conflict_cleans_reservation` (L623) 都**直接调** `svc.apply_atomic_with_reservation(...)`。
  - 它们**没有**经过 `SagaOrchestrator::execute → ReserveHandler::execute` 路径。
  - 因此**生产路径** (saga_orchestrator.rs:248-289) 的同种 bug 完全没被新 test 触达。
- **影响**:
  - 2 个 test 给出的 100% pass 给"假阳性安全感":开发者/审查者会以为 CC-4 修复了,但实际生产路径未动。
  - 顺带:这 2 个 test 也没在 PG 集成环境下验证 (用 InMemoryAccountRepository,OCC 不会自然失败,需要手 bump version 才触发),所以 helper 自身能否在 PG 上正确清理也未被验证。
- **建议修复**:
  - 写 end-to-end test: `saga = make_transfer_saga; env.orch.execute(&mut saga).await` 触发真实 `ReserveHandler::execute` → 在 test 内手 bump `env.accounts` 中 account.version → 触发 apply_atomic OCC 失败 → 验证 (a) saga 终态 Failed, (b) reservation 表里该 saga 无 dangling 行,(c) 账户余额未减。

### [DC-1-REGRESSION-001] `resume_compensating_saga_triggers_compensation` 用了 stub handler,**未覆盖 55.12 真实 bug**
- **文件**: `crates/economy-service/src/saga_orchestrator.rs:935-1004`
- **证据**:
  - test 自定义 `CompensateRecorder` (L938-954),其 `compensate` 仅设置 flag,没有真实退款或清理逻辑。
  - test 用 `FailingHandler` (L964-966) 替代真实 `ConfirmHandler`。
  - 55.12 真实 bug 修复的回归点 (per RGS-REV-008 §4.4 资金幻影) 需要**真实 `ReserveHandler.compensate`** 在 OCC 失败路径上被正确触发且**不加钱** — 当前 test 完全没测这点。
- **影响**:
  - DC-1 报告标的"55.12 pre-existing bug 修复的回归点"在该 test 内**未真正被覆盖**。
  - 若 CC-4-DEAD-001 提到的 `ReserveHandler.compensate` 加钱 bug 未来回归,本 test 不会捕捉。
- **建议修复**:
  - 改用真实 `ReserveHandler` + `ConfirmHandler`(配合 PENDING 余额+预存 reservation),构造 OCC 失败场景,验证补偿路径**不**触发 `account.credit(refund_amount)`。

---

## MEDIUM (3)

### [M-CC-4-SWALLOW-001] `ReserveHandler::execute` L259 `let _ = ... .delete_by_id(r.id).await` 静默吞错
- **文件**: `crates/economy-service/src/saga_orchestrator.rs:259`
- **证据**:
  ```rust
  if !account.try_debit(self.amount) {
      // 清理 dangling reservation
      let _ = self.reservations.delete_by_id(r.id).await;  // <-- 静默吞错
      return Err(Error::InsufficientFunds { ... });
  }
  ```
  - `delete_by_id` 返回 `Result<bool>` (reservation.rs:97, 208-210),吞掉 `Err` 等于吞掉 DB 故障,reservation 永远 dangling 而无任何告警。
  - 与 helper (service.rs:110-118) 的新写法 `if let Err(cleanup_err) = ... { tracing::warn!(...) }` **不一致**。
- **影响**:
  - 任务说明里提示的"`reservations.delete_by_id().ok() 用 .ok() 吞错是否合理"问题:**其实没用到 `.ok()`**,但用了等价的 `let _ = ... .await` 模式,效果相同。
  - DB 暂时不可用时,reservation 永久 dangling,且无 observability 出口。
- **建议修复**:
  ```rust
  if let Err(cleanup_err) = self.reservations.delete_by_id(r.id).await {
      tracing::warn!(
          target: "saga",
          saga_id = %saga.id, account_id = %account_id,
          reservation_id = %r.id,
          "failed to cleanup dangling reservation after insufficient funds: {}", cleanup_err
      );
  }
  ```

### [M-AC-1-DEAD-001] `MTLS_BYPASSED_TOTAL` 6 处定义 0 处读 — 写完不暴露
- **文件**: 6 域 main.rs 各 1 个 `static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);` (例 admin-service/src/main.rs:38),配 `fetch_add(1, Ordering::Relaxed)` (admin L127)
- **证据**:
  - `grep -r "MTLS_BYPASSED_TOTAL" crates/` 显示 18 处(6 定义 + 6 fetch_add + 6 doc 引用),**0 个 `load`/`read`/Prometheus exporter scrape**。
  - 注释 (admin L33-34) 自承"由后续任务处理;本 PR 仅做 fail-closed 防线本身"。
  - 同时 shared-platform 也有同名 client 端 private static,文档说"未修 shared-platform 任务禁止",但 client 端是否真正在 read 这次 grep 没覆盖(shared-platform 内的 use 需单独验证,本审查范围未深入)。
- **影响**:
  - counter 写了不读,功能等同于"日志里加 1" — Prometheus 告警规则、SLO 看板都没法基于这个 metric。
  - "fail-closed + 监控"只完成一半,部署方可能误以为监控已就位。
- **建议修复**:
  - 与 verify-A AH-4 (per RGS-REV-008) 多副本聚合任务合并,在 shared-platform 加 `pub fn mtls_bypassed_total() -> u64` 暴露 getter;或在 6 域加 `/metrics` scrape endpoint。
  - 同步检查 shared-platform 的 client 端同名 static 是否真正被 read。

### [M-CC-3-LEGACY-001] CHECK 字符串与 `OutboxStatus::as_str` 列表漂移风险 — 当前一致,但无编译期强约束
- **文件**: 6 域 outbox migration CHECK 子句;`crates/shared-platform/src/outbox.rs:67-74` 的 `as_str()` 是 single source of truth
- **证据**:
  - 当前 enum 4 个值 (`Pending/InFlight/Sent/Failed`) → 字符串 ("pending/in_flight/sent/failed") 与 6 域 CHECK 子句**完全匹配**(已人工核对 outbox.rs:67-74 与 6 域 migration)。
  - 但二者**没有共享**常量:未来若有人在 outbox.rs 加新 variant(如 `DeadLetter`)而忘了同步 6 域 migration,会**编译期 + 测试期都无法察觉**,只在生产部署时 CHECK 触发错误。
- **影响**:
  - 当前 OK,长期技术债。
- **建议修复**:
  - 在 shared-platform 暴露 `pub const OUTBOX_STATUS_VALUES: &[&str] = &["pending", "in_flight", "sent", "failed"];`,在 build.rs 或 include_str! 注入到 migration 生成中。

---

## LOW (3)

### [L-AC-1-PARSE-001] `RGS_ALLOW_INSECURE_GRPC` 解析方式 6 域一致但解析结果有"全 0 = 强制"风险
- **文件**: 6 域 main.rs L113-115 区域 (例 admin L119-120)
  ```rust
  let allow_insecure = env::var("RGS_ALLOW_INSECURE_GRPC")
      .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
  ```
- **证据**:
  - 6 域使用相同的 `is_ok_and` 闭包解析 — **一致**(无 dead store / 无重复定义)。✓
  - 行为:不设/空/"0"/"false"/任意其他值 → `allow_insecure = false` → 强制 mTLS。✓
  - 唯一小风险:若运维误设 `RGS_ALLOW_INSECURE_GRPC=YES`(运维习惯),则 `is_ok_and` 返回 false → 强制 mTLS,反而是 fail-closed 安全侧的"巧合"安全。
- **影响**:
  - 解析语义符合 fail-closed 意图(默认值 = 强制 TLS)。✓
- **建议修复**: 无,可选加单元 test 覆盖 "0"/"false"/"YES"/""/缺省 5 种 case。

### [L-DC-1-COVERAGE-001] DC-1 4 个 test 未覆盖 `Completed` / `Failed` / `Aborted` 终态的 resume 行为
- **文件**: `crates/economy-service/src/saga_orchestrator.rs:856-1018`
- **证据**:
  - 4 个 test 覆盖 `Pending`/`Running`/`Compensating`/`NotFound` 共 4 个 resume 入口。
  - `execute()` (L94-99) 对 `Completed`/`Failed`/`Aborted` 返 `Error::Validation`("already in terminal state"),**无 test 验证**。
  - 50s 崩溃恢复轮询 (`economy main.rs:104-136` 的 `list_running(100)`) 实际上不会调起这些终态 saga,所以这个 gap 风险低。
- **影响**:
  - 任务说明里"是否漏掉 Completed/Failed 状态"问题 — **是**,但 55.23 list_running 不会触发。
- **建议修复**: 加 2 个 test `resume_completed_saga_returns_validation_err` / `resume_failed_saga_returns_validation_err`,作为回归。

### [L-HOUSEKEEPING-001] json_logging doctest 修复正确,但 `no_run` 意味着永远不跑该 doctest
- **文件**: `crates/shared-platform/src/json_logging.rs:11-13`
- **证据**:
  - 删 `fn main() { ... }` 包裹(从 5 行变 1 行),保留 `no_run` 标记。✓
  - `no_run` 表示 doctest **编译但不执行** (per Rust reference),所以 `init_json_logging` 是否真能用不被 doctest 验证。
  - clippy 1.98 `needless_doctest_main` 被消除 (per commit message 已确认)。
- **影响**:
  - 修复对,但**未增加一个 `ignore` 或 `should_panic` 或 `compile_fail` 的二次 doctest 覆盖**,失去了原"fn main"包裹带来的"确保能编译运行"的语义(虽 fn main 包裹本身多余)。
- **建议修复**:
  - 删 `no_run`,让 doctest 真跑一次(或不跑 main,但用 `compile_fail` / 普通方式);或者补充一个非 no_run 的 sample usage test。

---

## 状态机覆盖矩阵 (DC-1 resume 路径)

| 状态 | resume 路径覆盖? | 备注 |
|---|---|---|
| Pending | ✅ | `resume_pending_saga_starts_and_advances` (L856-875) 真实走 start() → reserve → confirm → Completed,断言 status+2 steps |
| Running | ✅ | `resume_running_saga_continues_current_step` (L877-928) 跳过 start() 续跑 step 1,断言 step 0 未重跑 + 余额单次扣款(防 double-debit) |
| Compensating | ⚠️ | `resume_compensating_saga_triggers_compensation` (L935-1004) 验证 orchestrator 调 `compensate()`,但**用 stub handler,未触发真实 55.12 bug 回归点**(见 HIGH §2) |
| Completed | ❌ | 无 test;`execute()` (L94-99) 返 `Error::Validation`("already in terminal state"),但无回归 test |
| Failed | ❌ | 同 Completed;list_running 不会触发,但仍缺 test |
| Aborted | ❌ | 同 Completed |
| NotFound | ✅ | `resume_nonexistent_saga_returns_not_found` (L1006-1018) 验证 `Error::NotFound { entity: "Saga", .. }` |

**小结**: 覆盖 3/6 + 1 stub;缺 Completed/Failed/Aborted 终态 test(LOW-3)。

---

## reservation 生命周期审计

| 路径 | 覆盖? | 证据 / 风险 |
|---|---|---|
| create | ✅ | `apply_atomic_with_reservation` L102 `reservations.save(&reservation).await?` + ReserveHandler.execute L253 `self.reservations.save(&r).await?` |
| apply 成功 (happy path) | ✅ | helper L141-157 + ReserveHandler L277 |
| apply OCC 失败补偿 | ⚠️ | helper 正确清理 (L142-156),但 **ReserveHandler.execute L277 `?` 直接传播错误,无 cleanup** — 见 CRITICAL §1 |
| apply 余额不足补偿 | ⚠️ | helper 正确清理 (L110-118) + tracing::warn,ReserveHandler L259 用 `let _ =` 静默吞错 — 见 MEDIUM §1 |
| dangling 清理 test 覆盖 | ⚠️ | 2 个 helper-level test 覆盖 (service.rs:551, 623);**生产路径 ReserveHandler 0 test** — 见 HIGH §1 |
| Compensate 路径回退余额 | ❌ | ReserveHandler.compensate L291-342 真退 `r.amount`;若 OCC 失败遗留 dangling,会**凭空 +amount 退款** — 这是 55.12 真实 bug,新 test 未覆盖 |
| 跨进程并发 (deposit 期间别的 worker 改账户) | ❌ | PG 集成测试缺失,只在 InMemoryAccountRepository 验证 |

**小结**: helper 自我完备,生产路径多处 break;saga_orchestrator.rs:248-342 是**实际生产代码**,本次 commit 未触及。

---

## 验证结果

| 验证项 | 结果 | 备注 |
|---|---|---|
| `cargo test --workspace --lib` | **209/209 passed** (admin 18 + cluster-ops 16 + economy 42 + match 16 + player 24 + rgs-testkit 0 + shared 78 + social 15) | 0 failed, 0 ignored;耗时 ~0.5s |
| `cargo test -p economy-service saga_orchestrator --lib -- --test-threads=1` | **12/12 passed** (含 4 个 DC-1 新 test) | 0 failed |
| `cargo test -p economy-service saga_orchestrator::tests::resume_ -- --test-threads=1` | **4/4 passed** (DC-1.1-1.4 独立可过) | 0 failed |
| `cargo clippy -p {6域+shared+rgs-testkit} --all-targets --no-deps -- -D warnings` | **0 warning** | 跳过 rgs-certgen/rgs-hello(pre-existing 55.x 范围外) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **失败** (rgs-certgen/rgs-hello 3 个 pre-existing 错误,与本 5 commit 无关) | commit message 已声明 |

**真跑可过**: DC-1 4 个 test + CC-4 2 个 helper test + 55.x 历史 test 全 pass。但 "pass" 不等于 "correct" — 见 CRITICAL §1,生产路径的同种 bug 完全没被新 test 触达。

---

## 结论

- **是否可合并**: ❌ **否** (待修)
- **阻塞合并的问题** (按严重度):
  1. **CRITICAL §1** (CC-4-DEAD-001): `apply_atomic_with_reservation` 是死代码,生产路径 `ReserveHandler::execute` (saga_orchestrator.rs:248-289) **未修复**,55.12 资金幻影 bug 仍然存在。
  2. **CRITICAL §2** (CC-3-MIGRATION-001): outbox CHECK 约束在已部署环境上**永不生效** (`CREATE TABLE IF NOT EXISTS` 静默跳过),需新增 6 域 ALTER TABLE migration。
  3. **HIGH §1** (CC-4-TEST-001): 2 个新 test 覆盖的是未使用的 helper,不是生产路径,需要补 ReserveHandler.execute 的 end-to-end test。
  4. **HIGH §2** (DC-1-REGRESSION-001): 真实 55.12 回归点未被 stub handler test 覆盖,需用真实 ReserveHandler/ConfirmHandler 改写。
- **不阻塞但应修** (MEDIUM/LOW): 6 域 MTLS_BYPASSED_TOTAL 监控出口(M-AH-1)、ReserveHandler L259 静默吞错(M-1)、3 个终态 resume test 缺失(L-1)。
- **可保留的 fix** (✅): DC-1 4 个 test 本身编译 + 跑通 + 文档清晰,AC-1 6 域 mTLS fail-closed 解析模式一致,housekeeping doctest 修复正确,outbox 字符串与 enum 当前一致(M-3 仅长期债)。

---

## 建议下一步

1. **回归 CRITICAL §1**: 修改 `ReserveHandler::execute` 真正接入 helper,或 inlined 修复 + `tracing::warn!`;在 PgAccountRepository 上写 OCC 失败 integration test。
2. **回归 CRITICAL §2**: 新增 6 域 `0004_outbox_check_constraint.sql` (或对应序号),内容 `ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))`;需考虑 IF NOT EXISTS 适配(7.1+ 支持 `ADD CONSTRAINT IF NOT EXISTS`? — 不支持,需用 `DO $$ ... $$` 块)。
3. **回归 HIGH §1-2**: 重写 CC-4 + DC-1 test 覆盖生产路径,删 helper 或把 helper 改名为内联。
4. **MERGE blocker**: 完成 1+2 后再 re-run verify-C 真实 PG 环境(目前仅 InMemory)。