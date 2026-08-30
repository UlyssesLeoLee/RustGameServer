# V1 安全审查报告 (WF-1-55.26 5 commit)

## 元数据
- 审查范围: 1b30878..cc888b5 (5 commit)
- 审查维度: Security
- 审查者: V1 (verifier)
- 日期: 2026-08-23
- Worktree: D:/adversarial-55-26-V1
- Target dir: D:/target-adversarial-V1

## CRITICAL (1 个)

### [CC-4-FIX-IN-WRONG-FN] CC-4 资金幻影修复未落在生产代码路径 — 修补了死代码
- 文件: crates/economy-service/src/saga_orchestrator.rs:248-289 (ReserveHandler.execute, 真实生产路径)
- 关联文件: crates/economy-service/src/service.rs:86-160 (apply_atomic_with_reservation, 仅被 4 个测试调用)
- 证据:
  - a950b46 commit 仅修改 service.rs (+190/-6)，**未触碰 saga_orchestrator.rs**。
  - `apply_atomic_with_reservation` (service.rs:86) 的 doc comment 自称"给 SagaOrchestrator 的 ReserveHandler/ConfirmHandler 用"，但实际全仓 grep 显示该函数**只在 service.rs 的 4 个 test (lines 487/536/580/660) 中被调用**，无任何 production 调用点。
  - production 路径 main.rs:99-105 直接 `ReserveHandler::new(...)` + `ConfirmHandler::new(...)` 注入，**ReserveHandler.execute (saga_orchestrator.rs:248-289) 仍内联 self.reservations.save / self.accounts.apply_atomic**：
    - L253: `self.reservations.save(&r).await?;` — reservation 持久化
    - L277: `self.accounts.apply_atomic(&account, &entry).await?;` — OCC 失败时 `?` 直接传播，**无 delete_by_id 清理**
  - 触发链：apply_atomic OCC 失败 → step 标 Failed → compensate(saga) → ReserveHandler.compensate (L291-342) 找到 dangling reservation → `account.credit(refund_amount)` + apply_atomic 写库 → 账户从未被扣却凭空 +amount。
  - 测试 `apply_atomic_with_reservation_occ_conflict_cleans_reservation` (service.rs:624) 验证了**死代码**的正确性，**未触及生产路径**。即使把它删掉，CC-4 资金幻影 bug 仍 100% 触发。
- 影响: CRITICAL — RGS-REV-008 verify-C 标 CC-4 为 CRITICAL 资金安全问题（"凭空造钱"）。声称修复但实际未修；测试通过完全是自欺欺人。任何 OCC 冲突（高并发转账 / 跨副本竞争）会稳定触发幻影金额。
- 建议修复:
  1. 把 `apply_atomic_with_reservation` 的逻辑下沉到 `ReserveHandler.execute`，或
  2. 直接在 L277 的 `?` 之前加 reservation 清理（参考 service.rs:142-157 模式），或
  3. 把 `reservations.save` + `accounts.apply_atomic` 合并到单事务（方案 A，per verify-C §3 CC-4 修复建议）

## HIGH (2 个)

### [AC-1-METRIC-DEAD] server 端 MTLS_BYPASSED_TOTAL 是无 getter 的死 counter
- 文件: 6 域 main.rs:38/39/46/38/38/38 (`static MTLS_BYPASSED_TOTAL: AtomicU64 = AtomicU64::new(0);`)
- 证据:
  - 6 域每个 main.rs 独立定义同名 static AtomicU64，但**没有 pub fn getter 暴露给 metrics 层**。
  - shared-platform 已有 client 端同名 static + `mtls_bypassed_total() -> u64` getter (channel.rs:80-87)，server 端无法聚合。
  - PR doc 显式承认"监控集成（Prometheus exporter / scrape handler → `mTLS_bypassed_total`）由后续任务处理"——意味着当下没有 alert / metric 抓取这条数据。
  - 6 域 atomic 是 per-process，6 副本部署时只能各自 increment 1，看不到全局聚合；即使将来加 getter 也只是 6 份独立数据。
- 影响: HIGH — fail-closed 防线只在 trace log 留痕（`tracing::warn!`），**生产 Prometheus / Grafana 看不到 "有人在 prod 跑了 insecure gRPC"**。一旦误注入 `RGS_ALLOW_INSECURE_GRPC=1` 到 k8s deployment，监控盲区，攻击窗口无声打开。**整个 metric 加了等于没加**——只是合规 checklist 上的勾。
- 建议修复:
  - 短期：在 shared-platform 加 `pub fn mtls_server_bypassed_total() -> u64` getter（即使 PR 约束不动 saga/main.rs，shared-platform 加 read-only getter 不算破坏约束）。
  - 中期：把 6 域 static 收回到 shared-platform（与 client 端同模块），由 metrics 层统一 scrape。
  - 长期：每域 emit `tracing::warn!` 同时追加 `tracing::warn!(counter.inc())` 到 OTel metrics，避免依赖 log 抓取。

### [DC-1-TEST-NO-DOUBLE-COMP] 缺生产 handler 的"防双补偿"回归测试
- 文件: crates/economy-service/src/saga_orchestrator.rs:935-1004 (DC-1.3 测试用自定义 CompensateRecorder)
- 证据:
  - DC-1.3 测试用 `struct CompensateRecorder` + `FailingHandler`，**没有使用真实 ReserveHandler/ConfirmHandler**。
  - `compete()` (L142-166) 的 filter 逻辑是 `s.status == SagaStepStatus::Completed`——已 Compensated 步骤会被跳过，这是设计上的幂等保证。
  - 但**没有测试**验证：当真实 ReserveHandler.compensate 已经因崩溃部分执行（账户已 +amount 退款、ledger 已写、saga.step 标 Compensated）后，resume(Compensating) 重跑失败步触发新 compensate，**不会**再次调 ReserveHandler.compensate。
  - 当前 code review 显示逻辑正确（filter 排除了已 Compensated），但**没 test 锚定这个 invariant**。下一次重构若有人改成 `filter(|s| s.status != Failed)` 之类的，立即产生 +amount 双倍退款（双倍造钱）bug。
- 影响: HIGH — 资金安全 invariant 缺乏回归测试。CC-4 fix 路径已暴露"测试通过但代码错"的反模式，DC-1 必须用真实 handler 测才可信。
- 建议修复: 加测试 `resume_after_partial_compensation_does_not_double_refund`，预置：step_a=Compensated + 账户已 +amount credit + ledger 已有 Compensated entry，调用 orch.resume，断言 step_a 不被再次补偿、账户余额不再次 +amount、ledger 不再次写 Compensated entry。

## MEDIUM (2 个)

### [CC-4-COMPENSATION-CRASH] 补偿半途崩溃 → 资金丢失路径（pre-existing，55.12 引入，55.26 未触及）
- 文件: crates/economy-service/src/saga_orchestrator.rs:141-166 (compete 函数)
- 证据:
  - compete() 流程：L146-152 收集 Completed 步骤 → L154 `saga.compensate()`（把 Completed 标 Compensated，DB 持久化 L155）→ L157-161 调 `handler.compensate(saga, resource_id)`。
  - 若系统在 L155 save 后、L161 handler.compensate 前崩溃：DB 中 step=Compensated 但实际退款 + ledger 写入未发生。
  - 之后 `resume(Compensating)`：filter `s.status == Completed` 把已 Compensated 步骤排除 → handler.compensate **不会**被再次调用。
  - 结果：账户已被 debit 但 refund side effect 永远丢失。saga 终态 Failed，账户金额净减少。
- 影响: MEDIUM — 资金丢失（不是凭空造钱，但是钱真丢）。崩溃窗口在 `saga.save(L155)` 之后、`handler.compensate(L161)` 之前（几 ms 窗口），发生概率低但非零，且**不产生任何告警**。
- 建议修复:
  - 方案 1：`handler.compensate` 调用顺序倒过来——先调 handler 后标 status。已标 Compensated 步骤强制允许重试（`compete()` 改 filter 为 `status != Compensated`），但要保证 handler 自身幂等。
  - 方案 2：增加 reconciliation cron——每日扫 `saga.status=Failed` 且 step 标 Compensated 但无对应 Compensated ledger entry 的记录，触发人工 / 自动对账。
  - 短期：至少在 handler.compensate 失败时不要 mark saga 为 Failed（让 saga 留 Compensating 状态等下次 resume）。

### [AC-1-WHITESPACE-PARSE] RGS_ALLOW_INSECURE_GRPC 解析未 trim 前后空白
- 文件: 6 域 main.rs（约 119-120 行）`env::var("RGS_ALLOW_INSECURE_GRPC").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))`
- 证据:
  - `v == "1"` 严格匹配，不 trim。若运维误操作 `RGS_ALLOW_INSECURE_GRPC=" 1"` (前置空格) 或 `RGS_ALLOW_INSECURE_GRPC="1\n"` (k8s ConfigMap 换行)，`v == "1"` 为 false → fail-closed。
  - 严格匹配**符合 fail-closed 原则**——任何不精确匹配都不会 bypass，**这是好的**。
  - 但同时 dev 同事如果复制 `RGS_ALLOW_INSECURE_GRPC="true "`（尾空格）想开 insecure，会惊讶地发现被 fail-closed 拒绝，需要排查。
  - 不算 bug，列入 MEDIUM 是因为文档/PoC 缺失。
- 影响: MEDIUM — 行为本身 fail-closed 正确（误配 → 拒绝）。但运维手册没说这一点，dev/staging 排查时间会变长。
- 建议修复: 在 doc comment 里明确"`RGS_ALLOW_INSECURE_GRPC=1` 必须严格无前后空白；任何含空白的值被视为 fail-closed"，并在 k8s ConfigMap 模板加 `strip` 注释。

## LOW (3 个)

### [HC-5-STILL-OPEN] RGS-REV-008 verify-C HC-5 outbox lease 30s 硬编码未处理
- 文件: crates/shared-platform/src/outbox.rs:301 `INTERVAL '30 seconds'`
- 证据: 与 55.17 引入的硬编码 30s 一样。55.26 不在 5 commit 范围，列出仅作 cross-reference。
- 影响: LOW — 跨区延迟 / relay 慢处理时 30s 容易过期，触发重复消费（被 `command_id` 幂等保护，但有性能损耗）。

### [HC-7-STILL-OPEN] RGS-REV-008 verify-C HC-7 Reservation::save ON CONFLICT 只更新 status
- 文件: 待 grep（55.26 不在范围）
- 影响: LOW — Reservation 除 status 外其它字段更新不生效，可能造成审计不准确。

### [MC-3-STILL-OPEN] RGS-REV-008 verify-C MC-3 Reservation 无 5 分钟过期清理
- 文件: 待 grep（55.26 不在范围）
- 影响: LOW — orphan reservation 长期堆积，storage 增长。

## 交叉对照 (vs RGS-REV-008 22f662f)

| RGS-REV-008 ID | 声称修复 | 实际验证状态 | 备注 |
|---|---|---|---|
| AC-1 | 6 域 main.rs fail-closed | ⚠️ PARTIAL | fail-closed 防线本身正确（6 域一致、env 解析保守、? 上抛导致进程退出 1），但 server 端 MTLS_BYPASSED_TOTAL 是死 counter（见 HIGH-1），无监控盲区 |
| CC-3 | outbox CHECK | ✅ PASS | 6 域 migration 一致加 `CONSTRAINT chk_outbox_status CHECK (status IN ('pending','in_flight','sent','failed'))`；应用层 OutboxStatus 枚举只有 4 个值，与 CHECK 字符完全对齐；无绕过风险 |
| CC-4 | OCC + reservation 补偿 | ❌ FAIL | 修补了 service.rs 的 `apply_atomic_with_reservation`（仅测试调用），saga_orchestrator.rs:248-289 ReserveHandler.execute 的 OCC 失败路径仍 `?` 直接传播，无 reservation 清理，原始"凭空造钱"bug 未修复（见 CRITICAL-1） |
| DC-1 | SagaOrchestrator::resume 4 test | ⚠️ PARTIAL | 4 个 test 都真实存在且通过：DC-1.1 (Pending)、DC-1.2 (Running 无 double-debit)、DC-1.3 (Compensating 触发补偿)、DC-1.4 (NotFound)。但 DC-1.3 用自定义 CompensateRecorder 而非真实 ReserveHandler/ConfirmHandler，缺"防双补偿"回归测试（见 HIGH-2） |

## 验证结果

### cargo test --workspace --lib
- **总测试数**: 209 (18+16+42+16+24+0+78+15)
- **通过**: 209 / **失败**: 0 / **忽略**: 0
- **耗时**: 含编译 ~120s, 测试本体 <2s
- 注意: PR doc 声称 203，实际 209（多 6 个：DC-1 加 4 个 + 55.26 service.rs 加 3 个新 test，但其中一个 existed before，net +6）
- 测试 log: D:/RustGameServer/docs/00-基本与基准/reviews/adversarial-55-26/cargo-test.log

### cargo clippy --workspace --all-targets -- -D warnings
- **结果**: build **失败**（3 errors, 全部在 rgs-certgen）
- 失败位置: crates/rgs-certgen/src/main.rs:74/100（`&PathBuf` 应为 `&Path`）+ 1 个 let-binding unit value
- **pre-existing**: 55.24 housekeeping worker 漏修，per 55.26 PR doc 明示"rgs-certgen 3 个错误仍在 55.x 范围外, 56.x 处理"
- 排除 rgs-certgen 后: **0 warning**（6 域 service + shared-platform + rgs-testkit 全部通过）
- clippy log: D:/RustGameServer/docs/00-基本与基准/reviews/adversarial-55-26/cargo-clippy.log
- clippy 排除 log: D:/RustGameServer/docs/00-基本与基准/reviews/adversarial-55-26/cargo-clippy-excl-certgen.log

### 实际跑过的命令（按顺序）
1. `git -C D:/RustGameServer worktree add D:/adversarial-55-26-V1 HEAD`
2. `cargo test --workspace --lib --manifest-path D:/adversarial-55-26-V1/Cargo.toml` (CARGO_TARGET_DIR=D:/target-adversarial-V1)
3. `cargo clippy --workspace --all-targets --manifest-path D:/adversarial-55-26-V1/Cargo.toml -- -D warnings` (失败：rgs-certgen)
4. `cargo clippy --workspace --all-targets --manifest-path D:/adversarial-55-26-V1/Cargo.toml --exclude rgs-certgen -- -D warnings` (0 warning)

## 结论
- **是否可合并**: **否** — CRITICAL-1 (CC-4 资金幻影未真修复) 必须先修
- **最大 3 个风险**:
  1. **CC-4 修复在错误函数** — 修补死代码、生产路径未触动；声称通过 209 个 test 但资金安全 invariant 实际未验证；下次 OCC 冲突就是资金凭空生成事件
  2. **mTLS bypass 无监控** — fail-closed 防线只在 trace log 留痕，6 域 static 是死 counter；RGS_ALLOW_INSECURE_GRPC=1 误注入生产 k8s 不会被任何 Prometheus 告警发现
  3. **DC-1 缺防双补偿测试** — 真实 ReserveHandler.compensate 在崩溃恢复场景下"被调用 N 次"的可能性未被测试锚定；下一次重构可能引入双倍退款

## 推荐修复顺序（最小阻断 PR）
1. **必须**：把 service.rs:86-160 的 `apply_atomic_with_reservation` 整合到 saga_orchestrator.rs:248 ReserveHandler.execute，让 OCC 失败也走 delete_by_id 清理路径；或加 `match self.accounts.apply_atomic(...).await { Ok ... Err(e) => { reservations.delete_by_id(r.id).await; return Err(e); } }`
2. **必须**：用真实 ReserveHandler/ConfirmHandler 重写 DC-1.3 测试，覆盖"已 Compensated 步骤不被再次补偿"分支
3. **建议**：在 shared-platform 加 server 端 `mtls_bypassed_total()` getter；或短期内把 6 域 static 改成 `tracing::warn!(counter.inc())` 走 OTel metrics
4. **可选**：MEDIUM-1（补偿半途崩溃对账）开 56.x ticket