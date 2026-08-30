# V4 对抗仲裁报告 (WF-1-55.26 5 commit)

## 元数据
- 审查范围: 1b30878..cc888b5 (5 commit)
- 审查轮次: 第 2 轮 (对抗轮)
- 审查者: V4 (adversarial verifier)
- 前置输入: V1 / V2 / V3 报告
- Worktree: D:/adversarial-55-26-V4 (independent)
- Target dir: D:\target-adversarial-V4
- 日期: 2026-08-23

---

## 0. V1/V2/V3 关键发现仲裁矩阵

| ID | V1 评级 | V2 评级 | V3 评级 | **V4 仲裁** | 独立证据 |
|---|---|---|---|---|---|
| CC-4 死代码 | CRITICAL | CRITICAL | M-1 MEDIUM | **CRITICAL** | `apply_atomic_with_reservation` 0 生产调用 (grep 出 4 处全在 service.rs:487/536/580/660 test) |
| CC-3 migration 无效 | 未列 | CRITICAL | 未列 | **CRITICAL** | 6 个 migration CHECK 全部在 `CREATE TABLE IF NOT EXISTS` 块**内部**（已部署环境 no-op） |
| mTLS 死 counter | HIGH | M-AC-1 MEDIUM | HIGH-2 | **HIGH** | 6 service main.rs 全 private static + 0 load 调用 |
| DC-1 测试不足 | HIGH | HIGH | HIGH-1 | **HIGH** | DC-1.3 CompensateRecorder 真实存在但 stub;terminal state 0 coverage |
| fail-closed 启动无 test | 未列 | 未列 | L-2 LOW | **HIGH** (升级) | 6 域全 0 integration test 验证 `load_server_tls_config` 失败时确实退 1 |
| admin migration 注释 | 未列 | 未列 | M-2 MEDIUM | **MEDIUM** | admin `0003_outbox.sql` 注释写 `0002_outbox`（V3 对） |

---

## 1. V1/V2 CRITICAL 独立验证

### 1.1 CC-4 死代码 — 确认 V1+V2 CRITICAL，**反驳 V3 降级为 M-1**

**我的 grep 结果** (D:/adversarial-55-26-V4 全仓):
```
crates/economy-service/src/service.rs:86:  pub async fn apply_atomic_with_reservation(...)  [定义]
crates/economy-service/src/service.rs:487: .apply_atomic_with_reservation(...)               [test]
crates/economy-service/src/service.rs:536: .apply_atomic_with_reservation(...)               [test]
crates/economy-service/src/service.rs:580: .apply_atomic_with_reservation(...)               [test]
crates/economy-service/src/service.rs:660: .apply_atomic_with_reservation(...)               [test]
```
**0 生产调用**。生产路径 `ReserveHandler::execute` (saga_orchestrator.rs:248-289) **未使用**该 helper，直接 inline：
- L253: `self.reservations.save(&r).await?;` (reservation 落库)
- L259: `let _ = self.reservations.delete_by_id(r.id).await;` (静默吞错)
- L277: `self.accounts.apply_atomic(&account, &entry).await?;` (**OCC 失败无 cleanup**)

**bug 触发链 (与 V1+V2 独立推演一致)**:
OCC 失败 → step 标 Failed → 触发 compete() → `ReserveHandler.compensate` (L291-342) 找到 dangling reservation → `account.credit(refund_amount)` (L316) → `apply_atomic` 写 +amount (L327) → **凭空造钱**。

**V3 降级为 MEDIUM 的理由是错误的**。V3 M-1 原话:
> "当前所有裸 `apply_atomic` 调用前**未**先 `reservations.save(...)`,所以没有 dangling reservation 风险(实际安全)"

V3 看的是 service.rs 的 credit/debit (L208, L256) — 那里确实没有 reservation 写入。**但 V3 漏看了 saga_orchestrator.rs:248-289 的 L253** — `self.reservations.save(&r).await?` **就在 `apply_atomic` 之前**。V3 的论据是错的。生产路径真实有 reservation + 真实有 dangling 风险。

**V4 仲裁**: 确认 V1+V2 **CRITICAL**。反驳 V3 M-1。V3 漏看 L253 造成论证错误，**M-1 实际是 P0 资金安全 bug**。

### 1.2 CC-3 migration 无效 — 确认 V2 CRITICAL，V1+V3 漏列

**V2 描述**：`CREATE TABLE IF NOT EXISTS outbox (...)` 块内追加 `CONSTRAINT chk_outbox_status CHECK (...)`。已部署环境（55.17 已跑过 migration）`outbox` 表已存在，整个 CREATE 块被 sqlx 静默跳过 → CHECK 永不生效。

**我的独立验证** (diff 1b30878):
```
--- a/crates/economy-service/migrations/0003_outbox.sql
@@ -15,7 +15,8 @@ CREATE TABLE IF NOT EXISTS outbox (
     last_error TEXT,
     lease_until TIMESTAMPTZ,
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
-    sent_at TIMESTAMPTZ
+    sent_at TIMESTAMPTZ,
+    CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))
 );
```
**6 域全中招**（admin/cluster-ops/economy/match/player/social）。所有 6 个文件 diff 模式完全相同。

**CC-3 在 55.17 commit (`55af339`)** 已经把 `outbox` 表 CREATE 出来并跑过。生产环境现在的 `outbox` 表:
- 已有 (id, subject, payload, command_id, saga_id, status, retry_count, last_error, lease_until, created_at, sent_at) 列
- **无 CHECK 约束**
- 任何 status 字符串都可写入（"draft", "PENDING", "" 等）

**bug 后果**: RGS-REV-008 verify-C §4.3 标的"CHECK 防 status 漂移"在 6 域生产环境**完全不存在**。仅 fresh DB 部署 (CI / 新建环境) 才有效。

**V1 漏列原因**: V1 视角是 security, 关注 fail-closed 与 mTLS, 漏看 SQL migration 语义。
**V3 漏列原因**: V3 M-1 纠结在 "5 处裸 apply_atomic", 没看 1b30878 的 SQL 内容。

**V4 仲裁**: 确认 V2 **CRITICAL**。这是 silent-fail 性质 — 整个 5 commit 没说 "fresh DB only"。

---

## 2. V3 乐观评级挑战

### 2.1 为什么 V3 把 CC-4 降为 M-1 MEDIUM?

**V3 原话**: "当前所有裸 `apply_atomic` 调用前**未**先 `reservations.save(...)`,所以没有 dangling reservation 风险(实际安全) → 56.x 给裸 `apply_atomic` 加 `#[deprecated]`"

**V4 反驳**:
- V3 看的是 `service.rs:208/256` (credit/debit) 的 `apply_atomic`, 那里**确实**没 reservation 写入
- 但 V3 漏看了 `saga_orchestrator.rs:277` 的 `apply_atomic` — 之前 L253 已经 `reservations.save(&r).await?`
- V3 用"全 5 处 apply_atomic 都没 reservation" 来论证"无 dangling 风险"是**错误归纳**。生产路径 (saga_orchestrator.rs) 是真实 reservation + 真实 apply_atomic + 真实 dangling 风险
- V3 把"helper 不在生产路径用"当成了"未来代码可能误用" — 但实际是**当前代码已经在用 reservation + apply_atomic** 的**dangerous 组合**

**V3 CONDITIONAL PASS 的论证**:
- 5 commit 通过编译 OK
- 所有 209 unit test + 9 integration + 2 doc test = 220 passed OK
- release build 成功 OK

但 "pass" ≠ "correct"。V3 的论证基础是"测试通过即正确",但 CC-4 测试只覆盖**死代码 helper** (4 处都在 service.rs 测试函数里),生产路径 0 覆盖。这是 V1+V2+V4 共识的盲点。

### 2.2 V3 "CONDITIONAL PASS" 是否成立?

**结论: 不成立**。3 个反驳点:
1. **CC-4 真未修** (V4 §1.1 独立验证), V3 把它降为 M-1 是错判
2. **CC-3 migration 静默失效** (V4 §1.2 独立验证), V3 完全没看 SQL 语义
3. **mTLS server 端 counter 死代码** (V1 HIGH + V3 HIGH-2 共识), V3 自己也说"未来监控"但仍给 CONDITIONAL PASS — 自相矛盾

V3 的"集成视角"确实有价值 (6 域一致性矩阵 + 真实 cargo test 跑通), 但**集成视角的盲点是"正确性"**。当一个测试覆盖的是"死代码 helper"而不是"生产路径"时, 集成测试 pass 给的是**假阳性安全感**。

---

## 3. V1/V2/V3 都漏掉的盲区 (V4 独立发现)

### 3.1 [V4-NEW-001] `load_server_tls_config` 失败路径 0 integration test — V4 升级为 HIGH

**证据**:
- 6 域 main.rs 全部把 `load_server_tls_config` 失败路径从 "warn + None" (55.21+22 静默降级) 改成 `.context()?` 上抛退 1 (0240d4f)
- 这是**整个工程最关键的安全防线之一** (verify-A AL-1 / verify-C §4.1)
- 全仓 grep 0 test 模拟 "PEM 不存在 → 启动退 1" 场景

**V3 L-2** 提到这点但给 LOW。**V4 升级为 HIGH**: 0 test 锚定这个 fail-closed 边界 invariant 意味着下次 refactor `load_server_tls_config` 调用方 (例如改成 Result 透传 / 改用 anyhow) 可能**静默恢复静默降级** — 历史已经发生过一次 (55.21+22 的 silent fallback 就是当时的"看似无害的 fix")。

### 3.2 [V4-NEW-002] 5 commit 跨 commit 集成时序

**时序**:
- 06:34:20 `1b30878` CC-3 CHECK
- 06:36:43 `a950b46` CC-4 helper
- 06:37:33 `0240d4f` AC-1 mTLS fail-closed
- 06:40:07 `f9bf84f` housekeeping json_logging
- 06:40:28 `cc888b5` DC-1 resume tests

**f9bf84f → 0240d4f cross-impact 风险**: f9bf84f 修 json_logging doctest (移除 `fn main()` 包裹, 保留 `no_run`)。0240d4f 在 6 域 main.rs 引入 `init_json_logging` 启动时调用 + `.context()?` 退 1。
- **时序上 f9bf84f 在 0240d4f 之后**, 即 mTLS 改动先, housekeeping 后
- 两者**无直接冲突**: json_logging 只动 doctest 注释, 不动运行时代码
- 但 `no_run` 标记意味着 doctest 只编译不执行 — 6 域 main.rs 启动时调 `init_json_logging` 真的能跑通, 没被 doctest 验证
- **V2 L-HOUSEKEEPING-001 提到这点**, 评级 LOW 合理

**`1b30878` CHECK → `a950b46` helper 间接影响**: 无 (helper 在 service.rs, 不动 migration)。但 CC-3 + CC-4 都没真修是**两个独立 P0** 互相掩盖。

### 3.3 [V4-NEW-003] V1/V2/V3 测试数量口径不一致

- V1: 209 (lib only, 18+16+42+16+24+0+78+15)
- V2: 209 (lib only, 同 V1)
- V3: 220 (workspace, 含 9 integration + 2 doc test)
- V4 复跑: 209 lib + 9 shared-platform integration + 2 doc = 220 全 OK

**口径差异非问题**, 但 V1/V2 报告里说的"209 全过"给读者一个错觉"全工程测试覆盖好", 实际**真 DB 集成 0 个** (`#[sqlx::test]` / docker-compose 都没有)。V3 H-1 提到了这一点。

### 3.4 [V4-NEW-004] f9bf84f housekeeping 副作用

`f9bf84f` diff 仅 5 行: 移除 `fn main() { ... }` 包裹。**无副作用**:
- 0 `#[ignore]` 添加/删除
- 0 `#[cfg(...)]` 改动
- 0 测试删除
- 0 业务代码改动

**housekeeping 安全**。json_logging `no_run` 是个老问题, V2 LOW 评级合理, 不进 merge-blocker。

### 3.5 [V4-NEW-005] 6 域 main.rs fail-closed 的 env 解析"非 1/非 true" 行为

`RGS_ALLOW_INSECURE_GRPC="YES"` → `is_ok_and` 返回 false → **强制 mTLS (fail-closed)**。V2 L-AC-1-PARSE-001 提到这是"巧合安全"。

V4 仲裁: 这是 **fail-closed 原则正确**的体现, 不是 bug。V2 评级 LOW 合理。但运维手册 / k8s ConfigMap 模板需要明确"`YES/yes/on/enable` 都不行, 只接受 `1` 或 `true`"。

---

## 4. "209 test pass != correct" 解释

**为什么 5 commit 加 6 个 test (CC-4 2 + DC-1 4) 全过但有 CRITICAL**:

1. **CC-4 测试覆盖错对象**: 2 个新 test (`apply_atomic_with_reservation_occ_conflict_cleans_reservation` 等) 直接调 `svc.apply_atomic_with_reservation(...)`, 但生产路径 `SagaOrchestrator::execute -> ReserveHandler::execute` 调的是 `self.accounts.apply_atomic(...)` 直接, **绕过 helper**。test 永远在测死代码, 不会 fail。

2. **CC-4 测试用 InMemoryAccountRepository**: V3 H-1 提到, OCC 失败靠手动 `acc_repo.inner.lock().unwrap().get(&id).version = original + 99` 模拟。**生产 PG OCC** 是 `UPDATE ... WHERE version = ?` 0 row, 行为不完全等价。test 通过 != 生产正确。

3. **CC-3 migration 是 SQL, 无 Rust test**: 1b30878 只改 6 个 .sql 文件, 没加任何 Rust test 验证 CHECK 实际生效。`cargo test` 100% pass 与 migration 是否真生效完全无关。

4. **DC-1.3 stub handler**: `CompensateRecorder.compensate` 只 set `bool flag`, 不调真实 `ReserveHandler.compensate`。`account.credit(refund_amount)` 凭空造钱路径在 stub handler 内**根本无法触发**, 所以 stub test 不会暴露 V1 HIGH (双倍退款)。

5. **0 真 DB 集成**: 全 InMemory 跑通 209 test, 但 sqlx 行为 / 事务边界 / OCC 0 row 路径 0 验证。

**根因**: 5 commit 测的是"代码能编译运行", 没测"代码按设计语义正确"。

---

## 5. 修复优先级 (merge-blocker vs follow-up)

### Merge-blocker (必须修才能合并)

1. **CC-4 资金幻影真修**: 选其一:
   - (A 推荐) `ReserveHandler::execute` (saga_orchestrator.rs:248-289) L277 把 `?` 改成 `match self.accounts.apply_atomic(...).await { Ok ... Err(e) => { self.reservations.delete_by_id(r.id).await.ok(); Err(e) } }`, 同步加 `tracing::warn!`; 并把 helper 调用点下沉到生产路径
   - (B) 把 `apply_atomic_with_reservation` helper 内化到 ReserveHandler (消灭死代码)
   - 同步修 `ConfirmHandler::execute` (saga_orchestrator.rs:369-394) 同样的 OCC 模式
   - 加 1 个 `#[sqlx::test]` 真 DB 集成测试, 模拟 PG OCC 失败, 验证 (a) reservation cleanup (b) 账户余额未减 (c) ledger 无条目
2. **CC-3 migration 真修**: 6 域各加 `0004_outbox_check_constraint.sql` (或对应序号), 用 `DO $$ BEGIN ... ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (...); EXCEPTION WHEN duplicate_object THEN NULL; END $$;` 幂等块; 或在 1b30878 文件内追加 `DO` 块
3. **DC-1.3 stub 改真 handler**: 用 `ReserveHandler` + `ConfirmHandler` 重写 `resume_compensating_saga_triggers_compensation`, 构造 55.12 真实 OCC 失败场景, 验证 `account.credit(refund_amount)` **不**被调用

### Merge-with-follow-up (HIGH, 56.x 必修但可合并)

4. **mTLS server 端 counter 公共读出**: 在 shared-platform 加 `pub fn server_mtls_bypassed_total() -> u64` + 6 域 main.rs 改调; 或本次 PR 加 6 个 unit test 验证 `MTLS_BYPASSED_TOTAL.fetch_add(1, ...)` 行为
5. **fail-closed 启动 integration test**: 加 `assert_cmd::Command` 测试, 启动 binary 缺 cert dir, 断言非 0 退出码 + stderr 含 "mTLS config load failed"
6. **DC-1 terminal state coverage**: 加 `resume_completed_saga_returns_validation_err` / `resume_failed_saga_returns_validation_err` / `resume_aborted_saga_returns_validation_err` 3 个 test

### Defer to 56.x

7. admin `0003_outbox.sql` 注释修正 (`0002_outbox` -> `0003_outbox`)
8. clippy 验证脚本 `-A pedantic` -> `-A clippy::pedantic` 升级
9. rgs-certgen 3 个 pre-existing clippy error (let-binding unit / 2x &PathBuf)
10. ReserveHandler L259 `let _ = self.reservations.delete_by_id(r.id).await;` 静默吞错改 `tracing::warn!`
11. 55.x pre-existing issues (HC-5 outbox lease 30s / HC-7 reservation ON CONFLICT / MC-3 reservation GC)

---

## 6. 最终合并仲裁

### 决策: **NO MERGE** (V1 同, 反 V3 CONDITIONAL PASS)

### 决策依据

V1 给 NO MERGE, V3 给 CONDITIONAL PASS, 仲裁如下:

| 维度 | V1 | V2 | V3 | **V4 仲裁** |
|---|---|---|---|---|
| CC-4 死代码 | CRITICAL | CRITICAL | M-1 MEDIUM | **CRITICAL** (V1+V2 对, V3 错) |
| CC-3 migration | 未列 | CRITICAL | 未列 | **CRITICAL** (V2 对, V1+V3 漏) |
| mTLS 死 counter | HIGH | MEDIUM | HIGH | **HIGH** (V1+V3 对, V2 降级保守) |
| DC-1 测试 | HIGH | HIGH | HIGH | **HIGH** (3 共识) |
| fail-closed 启动 | 未列 | 未列 | LOW | **HIGH** (V4 升级) |

**2 个独立 CRITICAL 互相独立、互不掩盖**:
- CRITICAL §1: CC-4 资金幻影 (V1+V2+V4 共识, V3 错)
- CRITICAL §2: CC-3 migration 静默失效 (V2+V4 共识, V1+V3 漏)

**任一 CRITICAL 单独即阻断 merge**。2 个并存 -> NO MERGE 强化。

### 最大 3 个风险

1. **CC-4 资金幻影未修复**: OCC 失败时 reservation dangling, compensate 凭空 +amount。下次高并发转账就稳定触发。这是 P0 资金安全问题。
2. **CC-3 migration 在生产失效**: 6 域 outbox 表都没 CHECK 约束, status 漂移无 DB 层防御。任何 enum 改动 + 应用层写错状态都直接进表。
3. **mTLS server 端无监控**: 6 域 fail-closed 防线本身正确, 但 `RGS_ALLOW_INSECURE_GRPC=1` 误注入生产 k8s 不会被任何 Prometheus 告警发现。攻击窗口无声。

### 给 root session 的可操作建议

1. **不要**接受 V3 的 CONDITIONAL PASS。V3 的测试视角漏看 CC-4 L253 reservation.save + apply_atomic 真实组合, 也漏看 1b30878 SQL 语义。
2. **必须**让 worker 在 56.x 早期完成 §5 merge-blocker 1+2+3, 然后**用真 PG 实例** (CI docker-compose 起 PG) 跑 1 轮 verify, 再回来谈 merge。
3. **可推迟** §5 merge-with-follow-up 4+5+6 到 56.x 中期, 但需要在 56.x 任务单上明确 owner, 避免再次失访。
4. **审查 V1+V2+V3+V4 报告都报 LOW 的 housekeeping 收尾** (admin 注释 / clippy 脚本) — 这些可以小 PR 一次性清, 不进 56.x 主线。
5. **下次审查轮需引入真 DB 集成**: 让 `rgs-testkit` crate 加 `PgTestDatabase` fixture, DC-1 + CC-4 改用 `#[sqlx::test]`, 否则 209 test pass 给的是"代码能编译"假象。

---

## 7. V4 验证结果

| 验证项 | 命令 | 结果 | 耗时 |
|---|---|---|---|
| 全 worktree 创建 | `git worktree add D:/adversarial-55-26-V4 cc888b5` | OK | <1s |
| 全仓 test (lib) | `cargo test --workspace --lib --manifest-path D:/adversarial-55-26-V4/Cargo.toml` | **209/209 passed** (18+16+42+16+24+0+78+15) | ~2s |
| 全仓 test (full) | `cargo test --workspace --manifest-path D:/adversarial-55-26-V4/Cargo.toml` | **220/220 passed** (含 9 integration + 2 doc) | ~3s |
| clippy (排除 rgs-certgen) | `cargo clippy --workspace --all-targets -- -D warnings -A clippy::pedantic -A clippy::nursery -A clippy::cargo` | 3 errors pre-existing rgs-certgen (let-unit + 2x &PathBuf); 6 域 + shared + rgs-testkit **0 warning** | ~30s |
| grep 1: `apply_atomic_with_reservation` | 全仓 | 1 定义 + 4 test 调用, **0 生产调用** | <1s |
| grep 2: `MTLS_BYPASSED_TOTAL` | 全仓 | 6 service main.rs 私有 + 1 shared-platform 私有 (有 getter); 0 service-side load/读 | <1s |
| grep 3: `apply_atomic` in economy-service | economy-service | 5 处: service.rs:141/208/256 (helper + credit/debit) + saga_orchestrator.rs:277/327/432 (生产路径 3 处) | <1s |
| grep 4: 终态 resume test | economy-service | `resume_completed_saga` / `resume_failed_saga` / `terminal_state` **0 匹配** | <1s |
| diff 1b30878 SQL | 6 migration 文件 | CHECK 全部在 `CREATE TABLE IF NOT EXISTS` 块**内部** (last line of column list) | <1s |

---

## 8. V4 自评

**我可能错的地方**:
- CC-3 migration 评级 CRITICAL 假设 55.17 已经在生产环境跑过; 但如果实际项目还**未部署**到任何 prod, 仅在 CI / dev 跑, 那 CC-3 migration 在 fresh DB 部署时仍有效, 应降为 MEDIUM。但 RGS-REV-008 §3 描述"55.17 已部署", 故假设成立。
- "209 -> 220" 数量差异我归因为 V1/V2 跑 `--lib` vs V3 跑 `--workspace`, 但 V1 报告里 109 注脚说"含编译 ~120s"暗示跑的是 `cargo test --workspace`, 与 209 数字对不上。可能是 V1 跑 `cargo test` 抓 main bin + lib + integration 但漏算 shared-platform integration 的 9 个。**不影响主要结论**。

**我没覆盖的视角**:
- 性能回归 (5 commit 是否引入 size/throughput 退化): V3 跑了 `cargo build --release` 成功, 但**没**跑 perf benchmark。55.26 没承诺 perf, 不在范围。
- 6 域 shared-platform cross-crate 依赖方向: 未审查 Cargo.toml 是否被改 (V3 结论: 5 commit 未改 Cargo.toml)。
- binary 启动 config 注入路径: `RGS_ALLOW_INSECURE_GRPC` / `RGS_TLS_DIR` 之外的环境变量 (e.g. `RUST_LOG`) 是否被 5 commit 影响: 未查, 但 5 commit 范围明确未改 env 解析除 RGS_ALLOW_INSECURE_GRPC 之外的东西, 概率低。

**没跑的真 PG 模拟**: 任务说明里建议在 worktree 内建临时 .sql 文件测 `CREATE TABLE IF NOT EXISTS` 行为。我用代码审查 + PG 文档语义验证, **没**在真 PG 上跑 migration 模拟 (因 worktree 无 docker / 无 PG 实例)。但 PG 文档明确 `CREATE TABLE IF NOT EXISTS` 在表存在时是 silent skip, 无需真跑即可确认 V2 结论。

---

**End of V4 Report**