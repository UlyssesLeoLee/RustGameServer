# RGS-REV-009 WF-1-55.26 5 commit 3 轮对抗性审查总报告

## 元数据

- **审查范围**: `1b30878..cc888b5` (5 commit: CC-3 / CC-4 / AC-1 / housekeeping / DC-1)
- **审查模式**: 3 轮递进对抗（与 RGS-REV-008 平面 4 verifier 不同）
- **审查者**: V1 (安全) + V2 (正确性) + V3 (集成) + V4 (对抗仲裁) + V5 (综合收口) — 5 子代理
- **独立 worktree**: `D:/adversarial-55-26-V{1,2,3,4,5}`
- **独立 target dir**: `D:\target-adversarial-V{1,2,3,4,5}`
- **日期**: 2026-08-23
- **上一轮基线**: `22f662f` RGS-REV-008 (4 verifier 平面交叉审核 12 commit, 聚合 70 issue / 10C/20H/26M/14L)

---

## 1. 审查方法（3 轮递进对抗）

### 轮 1: 3 维独立审查（V1+V2+V3 并行）

- V1 安全视角 → `V1-security.md` (1C/2H/2M/3L = 8)
- V2 正确性视角 → `V2-correctness.md` (2C/2H/3M/3L = 10)
- V3 集成视角 → `V3-integration.md` (0C/2H/4M/3L = 9)

3 verifier 各自独立 worktree + 独立 target dir 编译，**未读其他 verifier 报告**。

### 轮 2: 交叉对抗仲裁（V4）

V4 读 V1/V2/V3 三份报告后：
- **独立验证** V1+V2 CRITICAL 共识（CC-4 死代码 + CC-3 migration 无效）
- **反驳 V3 CONDITIONAL PASS**：V3 M-1 评级（CC-4 降级）的论据"5 处裸 apply_atomic 都无 reservation" 错看了 `saga_orchestrator.rs:253/277` 的真实 reservation + apply_atomic 组合
- **升级 V3 L-2 fail-closed 启动 test** 从 LOW → HIGH：0 integration test 锚定整个工程最关键安全防线
- **独立 grep 验证** 5 个关键发现
- → `V4-adversarial.md` (V4 自己不产出新发现的 critical，但做 5 项 cross-impact 检查 + 仲裁矩阵)

### 轮 3: 综合收口（V5 本报告）

V5 任务：
1. 独立验证 V1-V4 关键发现
2. 整合共识矩阵（哪些 4/4 / 3/4 / 2/4 / 1/4 共识）
3. 仲裁 V1-V4 之间矛盾（仅 V3 vs V1+V2+V4 评级矛盾）
4. 产出 WF-1-55.27+ 任务清单（详见 `issues-55-27-catalog.md`）
5. 给 root session 可操作建议

**V5 独立验证结果**：
- `cargo test --workspace --lib` (worktree V5): **209/209 passed** (18+16+42+16+24+0+78+15) — 与 V1+V2+V4 一致
- `cargo clippy --workspace --all-targets -- -D warnings -A clippy::pedantic -A clippy::nursery -A clippy::cargo --exclude rgs-certgen`: **0 warning** — 与 V1+V3+V4 一致
- `git grep "apply_atomic_with_reservation"`: 1 定义 + 4 test + 2 doc, **0 生产调用**（service.rs:487/536/580/660）— V1+V2+V4 共识确认
- `git show 1b30878 -- 'crates/*/migrations/*outbox*.sql'`: 6 域 CHECK 全部在 `CREATE TABLE IF NOT EXISTS` 块**内部**（已部署环境 no-op）— V2+V4 共识确认
- `git show a950b46 --stat`: 1 file changed (仅 service.rs, +190/-6) — V1+V2 共识（CC-4 未触 saga_orchestrator.rs）
- `git show 1b30878` 验证: economy `0003_outbox.sql` 之前由 55.17 commit `55af339` 创建，1b30878 同文件追加 CHECK 块内 — 部署后无效
- `git grep "resume_completed\|resume_failed\|resume_aborted"`: **0 匹配**（3 终态 0 覆盖）— V2+V4 共识
- `git grep "load_server_tls_config"`: 单元测试有（tls.rs:142/156），但 0 integration test — V3 L-2 + V4 §3.1 升级共识

---

## 2. 共识矩阵

### 2.1 严重度统计

| 严重度 | V1 | V2 | V3 | V4 | **V5 终判** |
|---|---|---|---|---|---|
| CRITICAL | 1 | 2 | 0 | 2 | **2** (与 V2+V4 共识) |
| HIGH | 2 | 2 | 2 | 3 | **3** |
| MEDIUM | 2 | 3 | 4 | 1 | **4** |
| LOW | 3 | 3 | 3 | 0 | **4** |
| **TOTAL** | **8** | **10** | **9** | **6** | **13** |

**V5 终判原则**：任一 verifier 给 CRITICAL 即保留（除非有 V4 级别反驳），HIGH 需 2/4+ 共识，MEDIUM/LOW 保留 1/4+ 共识（除非被反驳）。

### 2.2 共识 issue 清单

| ID | 描述 | 严重度 | V1 | V2 | V3 | V4 | V5 | 评级依据 |
|---|---|---|---|---|---|---|---|---|
| **RGS-REV-009-CR-1** | CC-4 修复打偏靶（apply_atomic_with_reservation 是死代码，0 生产调用） | **CRITICAL** | 🔴 | 🔴 | 🟡(M-1) | 🔴 | 🔴 | V1+V2+V4 共识，V4 反驳 V3 降级 |
| **RGS-REV-009-CR-2** | CC-3 migration 静默失效（6 域 CHECK 在 `CREATE IF NOT EXISTS` 块内） | **CRITICAL** | — | 🔴 | — | 🔴 | 🔴 | V2+V4 独立发现，V1/V3 漏 |
| **RGS-REV-009-HI-1** | server 端 mTLS_BYPASSED_TOTAL 是死 counter（6 域 0 load/getter） | HIGH | 🟠 | 🟡(M-AC-1) | 🟠(H-2) | 🟠 | 🟠 | V1+V3+V4 共识，V2 降级保守 |
| **RGS-REV-009-HI-2** | DC-1 测试覆盖不足（4 个 stub handler 测 + 0 真 PG 集成 + 0 终态覆盖） | HIGH | 🟠 | 🟠(DC-1-REGRESSION) | 🟠(H-1) | 🟠 | 🟠 | 4/4 共识 |
| **RGS-REV-009-HI-3** | fail-closed 启动 0 integration test 验证 | HIGH | — | — | 🟢(L-2) | 🟠(升级) | 🟠 | V4 升级仲裁 |
| **RGS-REV-009-ME-1** | apply_atomic 裸调用未加 `#[deprecated]` 引导新代码走 helper | MEDIUM | — | — | 🟡(M-1) | — | 🟡 | V3 单独 |
| **RGS-REV-009-ME-2** | admin `0003_outbox.sql:1` 注释写 `0002_outbox` | MEDIUM | — | — | 🟡(M-2) | 🟡 | 🟡 | V3+V4 |
| **RGS-REV-009-ME-3** | clippy 验证脚本 `-A pedantic` 旧式被 1.98 弃用 | MEDIUM | — | — | 🟡(M-3) | — | 🟡 | V3 单独 |
| **RGS-REV-009-ME-4** | ReserveHandler L259 `let _ = self.reservations.delete_by_id(r.id).await` 静默吞错 | MEDIUM | — | 🟡(M-CC-4-SWALLOW) | — | — | 🟡 | V2 单独 |
| **RGS-REV-009-LO-1** | doctest 密度低（仅 2 个）、json_logging 修复后 `no_run` 不实际验证 | LOW | — | 🟢(L-HOUSEKEEPING) | 🟢(L-1) | — | 🟢 | V2+V3 |
| **RGS-REV-009-LO-2** | rgs-certgen 3 个 pre-existing clippy error（&PathBuf / let-binding unit） | LOW | — | — | 🟢(L-3) | — | 🟢 | V3 单独 |
| **RGS-REV-009-LO-3** | RGS-REV-008 verify-C HC-5/HC-7/MC-3 跨 commit pre-existing 未收尾 | LOW | 🟢(HC-5/7, MC-3) | — | — | — | 🟢 | V1 单独 cross-ref |
| **RGS-REV-009-LO-4** | V1 CC-4-COMPENSATION-CRASH 补偿半途崩溃 → 资金丢失路径 | LOW | 🟢 | — | — | — | 🟢 | V1 单独（55.12 pre-existing，56.x 排期） |

### 2.3 V1/V2/V3/V4 单独发现但未升级

- **V1 单独**: CC-4-COMPENSATION-CRASH 资金丢失路径（55.12 引入，55.26 未触及，pre-existing）
- **V1 单独**: HC-5 outbox lease 30s 硬编码、HC-7 Reservation::save ON CONFLICT、MC-3 reservation 无 5min GC（55.17/55.12 跨 commit pre-existing）
- **V1 单独**: AC-1-WHITESPACE-PARSE env 解析 trim 行为（V1 自评"行为正确但 doc 缺失"）
- **V2 单独**: L-DC-1-COVERAGE-001 终态 resume test 缺失（已纳入 RGS-REV-009-HI-2 终态覆盖子项）
- **V2 单独**: L-AC-1-PARSE-001 RGS_ALLOW_INSECURE_GRPC=`YES` 巧合 fail-closed（V4 仲裁：行为正确非 bug）
- **V2 单独**: L-HOUSEKEEPING-001 json_logging `no_run`（已纳入 LO-1）
- **V2 单独**: M-CC-3-LEGACY-001 CHECK 字符串与 OutboxStatus::as_str 无共享常量（长期债，未升级）
- **V3 单独**: M-4 RGS-REV-008 verify-D DC-1 评估标准更新（V5 判定为流程问题，纳入 LO 范畴）
- **V4 单独**: 无新增（V4 角色是仲裁而非新发现源）

### 2.4 V3 评级被反驳

- **V3 M-1（CC-4 降级 MEDIUM）** — V4 反驳：V3 漏看 `saga_orchestrator.rs:253` 的 `self.reservations.save(&r).await?` + L277 `self.accounts.apply_atomic(&account, &entry).await?` 的真实 reservation + apply_atomic 组合。V3 看的 service.rs:208/256 credit/debit 路径确实没 reservation，但**生产路径 saga_orchestrator.rs 是真实的 dangling 风险**。V4 仲裁：V1+V2 CRITICAL 正确，V3 M-1 错。

---

## 3. 关键 CRITICAL 详解

### 3.1 CR-1: CC-4 资金幻影未真修复（修复打偏靶）

**共识**: V1 + V2 + V4 (V3 错降为 M-1)

**证据链**（V5 独立 grep 确认）:
```
crates/economy-service/src/service.rs:86  [定义]   pub async fn apply_atomic_with_reservation(...)
crates/economy-service/src/service.rs:487 [test]   .apply_atomic_with_reservation(...)
crates/economy-service/src/service.rs:536 [test]   .apply_atomic_with_reservation(...)
crates/economy-service/src/service.rs:580 [test]   .apply_atomic_with_reservation(...)
crates/economy-service/src/service.rs:660 [test]   .apply_atomic_with_reservation(...)
0 production call.
```

CC-4 fix commit (a950b46) `--stat`: **1 file changed, +190/-6**（仅 `crates/economy-service/src/service.rs`），**未触及** `saga_orchestrator.rs`（生产路径）。

**生产路径**（`saga_orchestrator.rs:248-289` ReserveHandler::execute）:
- L253: `self.reservations.save(&r).await?;` ← reservation 落库
- L277: `self.accounts.apply_atomic(&account, &entry).await?;` ← OCC 失败时 `?` 直接传播，**reservation 留 dangling**

**触发链**（V1+V2+V4 独立推演一致）:
OCC 失败 → step 标 Failed → 触发 compete() → `ReserveHandler.compensate` (L291-342) → 找到 dangling reservation → `account.credit(refund_amount)` (L316) → `apply_atomic` 写 +amount (L327) → **凭空造钱**。

**影响**: 资金安全 P0。任何 OCC 冲突（高并发转账 / 跨副本竞争）稳定触发。

**修复方向**:
- (A 推荐) L277 改为 `match self.accounts.apply_atomic(...).await { Ok => Ok, Err(e) => { self.reservations.delete_by_id(r.id).await.ok(); tracing::warn!(...); Err(e) } }`
- (B) 把 helper 内化到 ReserveHandler，删除死代码
- 同步修 `ConfirmHandler::execute` (L369-394) 同样 OCC 模式
- 必加 `#[sqlx::test]` 真 PG 集成测试，模拟 PG OCC 失败

### 3.2 CR-2: CC-3 migration 静默失效

**共识**: V2 + V4（V1/V3 漏）

**证据链**（V5 独立 diff 确认）:
```
diff --git a/crates/economy-service/migrations/0003_outbox.sql
@@ -15,7 +15,8 @@ CREATE TABLE IF NOT EXISTS outbox (
     last_error TEXT,
     lease_until TIMESTAMPTZ,
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
-    sent_at TIMESTAMPTZ
+    sent_at TIMESTAMPTZ,
+    CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))
 );
```
6 域全中招（admin/cluster-ops/economy/match/player/social 模式相同）。

**关键 git log 证据**:
```
55af339  [wbs] WF-1-55.17: outbox SKIP LOCKED + ... (per RGS-REV-007)
1b30878  [wbs] WF-1-55.26: 6 域 outbox migration CHECK 约束 (per RGS-REV-008 CC-3)
```

55.17 commit `55af339` 已创建 outbox 表并部署。1b30878 在同文件追加 CHECK，**写在 `CREATE TABLE IF NOT EXISTS` 块内部**。PG 语义：表存在时 CREATE 块**silent skip**，CHECK 永不生效。

**影响**: 数据完整性 silent-fail。RGS-REV-008 verify-C §4.3 标的"CHECK 防 status 漂移"在 6 域生产环境**完全不存在**。仅 fresh DB 部署（CI / 新建环境）有效。

**修复方向**:
- 选项 A（推荐）: 6 域各加 `0004_outbox_check_constraint.sql`（或对应递增序号），内容 `ALTER TABLE outbox ADD CONSTRAINT chk_outbox_status CHECK (status IN ('pending', 'in_flight', 'sent', 'failed'))`
- 选项 B: 在 1b30878 文件内追加 `DO $$ BEGIN ALTER TABLE outbox ADD CONSTRAINT ...; EXCEPTION WHEN duplicate_object THEN NULL; END $$;` 幂等块
- PG 不支持 `ADD CONSTRAINT IF NOT EXISTS`（7.1+ 仍不支持），需 DO 块

---

## 4. "209 test pass ≠ correct" 现象分析

**为什么 5 commit 净增 6 test (CC-4 2 + DC-1 4) + 1 doc fix + 220 test 全过，但有 2 个 CRITICAL？**

V4 提出 5 项解释，V5 整合并补充：

1. **CC-4 测试覆盖错对象** — 2 个新 test 直接调 `svc.apply_atomic_with_reservation(...)` helper，但生产路径 `SagaOrchestrator::execute → ReserveHandler::execute` 调的是 `self.accounts.apply_atomic(...)` 直接。test 永远在测死代码，**不会 fail**。

2. **CC-4 测试用 InMemoryAccountRepository** — V3 H-1 提到，OCC 失败靠手动 `acc_repo.inner.lock().unwrap().get(&id).version = original + 99` 模拟。**生产 PG OCC** 是 `UPDATE ... WHERE version = ?` 0 row，行为不完全等价。

3. **CC-3 migration 是 SQL，无 Rust test** — 1b30878 只改 6 个 .sql 文件，没加任何 Rust test 验证 CHECK 实际生效。`cargo test` 100% pass 与 migration 是否真生效完全无关。

4. **DC-1.3 stub handler** — `CompensateRecorder.compensate` 只 set `bool flag`，不调真实 `ReserveHandler.compensate`。`account.credit(refund_amount)` 凭空造钱路径在 stub handler 内**根本无法触发**，所以 stub test 不会暴露 V1 HIGH (双倍退款)。

5. **0 真 DB 集成** — 全 InMemory 跑通 209 test，但 sqlx 行为 / 事务边界 / OCC 0 row 路径 0 验证。

**根因**（V5 补充）: 测试体系存在"**测试自身完备 ≠ 代码生产正确**" 的反模式：
- helper 单元 test 100% 覆盖 helper 自身，但 helper 是死代码 → 测试通过给的是"零价值安全感"
- 集成 test 用 InMemory repo，无法模拟 PG 真实行为 → 集成层 test 形同虚设
- 端到端 test 0 个 → 无"业务路径 0 → 1 → N 步"全链路覆盖

**教训**（V5 整合）:
- **测试覆盖对象必须是生产调用点**，不是修复目标
- **真 DB 集成测试**是 P0 资金安全 invariant 的唯一可信验证手段
- **silent-fail migration** 是 PG 类工程的常见盲区，`CREATE TABLE IF NOT EXISTS` 内追加 CHECK 在已部署环境无效

---

## 5. 修复优先级

### 5.1 Merge-blocker（P0, 必先修）

1. **CR-1**: 真修 `ReserveHandler::execute` (saga_orchestrator.rs:248-289) OCC 失败 cleanup
2. **CR-2**: 6 域新增 0004 outbox CHECK 幂等 migration
3. **HI-2-stub**: DC-1.3 stub handler 改真 `ReserveHandler.compensate`，验证 55.12 资金幻影回归点

### 5.2 Merge-with-follow-up（HIGH 推 WF-1-55.30+ 中期）

4. **HI-1**: shared-platform 加 `pub fn server_mtls_bypassed_total() -> u64` getter（与 client 端对称）
5. **HI-2-pg**: rgs-testkit 加 `PgTestDatabase` fixture（防止 "209 test pass ≠ correct" 假象复发）
6. **HI-3**: 6 域 fail-closed 启动 integration test（assert_cmd）
7. **HI-D**: DC-1 补 3 个终态 test（Completed/Failed/Aborted）

### 5.3 Defer to WF-1-55.34+（非阻断）

- **ME-1**: `apply_atomic` 裸调用加 `#[deprecated]`
- **ME-2/3**: admin migration 注释修正 / clippy 验证脚本升级
- **ME-4**: ReserveHandler L259 静默吞错改 `tracing::warn!`
- **LO-1/2/3**: doctest 密度 / rgs-certgen pre-existing / 55.12 pre-existing (HC-5/HC-7/MC-3)
- **LO-4**: V1 CC-4-COMPENSATION-CRASH 补偿半途崩溃 → 资金丢失（55.12 引入，WF-1-55.37 排期）

---

## 6. 最终决策

**当前 5 commit (cc888b5) 状态: ❌ NO MERGE**

- 理由: 2 个独立 CRITICAL (CR-1 资金安全 P0 + CR-2 数据完整性 silent-fail)
- 任何 1 个 CRITICAL 即阻断 merge，2 个并存强化 NO MERGE
- 与 V1+V2+V4 共识，**反驳 V3 CONDITIONAL PASS**

**V3 评级错判根因**（V5 仲裁）:
- V3 "集成视角" 重视 cargo test 数量（220 全过），但**漏看 209 test 测的对象是死代码 + stub handler + InMemory repo**
- V3 M-1 论据"5 处裸 apply_atomic 都无 reservation" 错看生产路径，实际 `saga_orchestrator.rs:253/277` 是真实的 reservation + apply_atomic 组合
- V3 漏看 1b30878 SQL diff，CC-3 migration 静默失效是 silent-fail 性质

**给 root session 的可操作建议**:

1. **立即否决 5 commit 当前合并**（V1+V2+V4+V5 共识）
2. **在 WF-1-55.27/28/29 (P0) 上开 ticket** 给 CR-1 + CR-2 + HI-2 (stub handler)
3. **引入 rgs-testkit 真 PG 集成测试基建**（PgTestDatabase fixture + `#[sqlx::test]`）
4. **修完后做 2 轮对抗性审查**（4+ verifier）再 merge
5. **不 push 当前 5 commit** 到 origin（main 领先 origin 106 commit，建议先在 WF-1-55.27/28 修 CR-1+CR-2 再 push）

---

## 7. 验证汇总

| 验证项 | V1 | V2 | V3 | V4 | V5 |
|---|---|---|---|---|---|
| `cargo test --workspace --lib` | 209/209 | 209/209 | 220/220 (含 9 integration + 2 doc) | 209/209 | **209/209** |
| `cargo clippy --workspace --all-targets --exclude rgs-certgen` | 0 warn | 0 warn | 0 warn | 0 warn | **0 warn** |
| 独立 grep `apply_atomic_with_reservation` | ✓ (0 prod) | ✓ (0 prod) | ⚠ (M-1 错看) | ✓ (0 prod) | **✓ (0 prod)** |
| 独立 grep `MTLS_BYPASSED_TOTAL` | ✓ (6 server 死) | ✓ (0 load) | ✓ (6 私有) | ✓ (6 server 死) | **✓ (6 server 死, 0 load)** |
| 独立 diff 1b30878 outbox CHECK | — | ✓ (CR-2) | — | ✓ (CR-2) | **✓ (6 域全中招)** |
| 独立 grep `resume_completed/failed/aborted` | — | ✓ (0 match) | — | ✓ (0 match) | **✓ (0 match)** |
| 独立看 `load_server_tls_config` integration test | — | — | ✓ (L-2 LOW) | ✓ (升级 HIGH) | **✓ (0 integration test)** |

---

## 8. 关键教训（5 commit 收尾的工程教训）

1. **测试全绿 ≠ 正确** — 测试覆盖死代码、InMemory repo 模拟不到 PG OCC。需引入真 DB 集成基建。
2. **silent-fail migration** — `CREATE TABLE IF NOT EXISTS` 内追加 CHECK 在已部署环境无效。需新 migration 文件 + 幂等 DO 块。
3. **加 metric 但不暴露** — server 端 counter 死代码，监控盲区。需对称 client 端加 pub getter。
4. **CRITICAL 修复需独立第三方验证** — V3 自己看 5 commit 没发现 V1+V2 看到的问题，V4 才看穿（V3 漏 L253/L277 + 漏 1b30878 SQL diff）。
5. **stub handler test 不可信** — DC-1.3 用 CompensateRecorder 仅 set flag，未触发真实 55.12 回归点。需真 handler 测。
6. **3 轮对抗 > 1 轮平面** — RGS-REV-008 (4 verifier 平面) 发现 70 issue 但 RGS-REV-009 (3 轮对抗) 5 verifier 通过仲裁抓出 2 个被平面审查错漏的 CRITICAL。

---

## 9. 附录

- V1 报告: `V1-security.md` (1C/2H/2M/3L = 8)
- V2 报告: `V2-correctness.md` (2C/2H/3M/3L = 10)
- V3 报告: `V3-integration.md` (0C/2H/4M/3L = 9)
- V4 报告: `V4-adversarial.md` (仲裁 + cross-impact)
- 5 commit 验证命令记录: 见各 verifier 报告 + V5 worktree (`D:/adversarial-55-26-V5`)
- RGS-REV-008 baseline: `22f662f` (4 verifier 平面交叉审核 12 commit, 70 issue)
- 修复任务清单: `issues-56x-catalog.md` (11 项, P0: 3, P1: 4, P2: 4)
- Commit 提案: `COMMIT_PROPOSAL.md`（不 commit，留给 root session 决策）

---

**End of RGS-REV-009 Total Report** (V5 verifier, 2026-08-23)
