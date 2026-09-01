# rgs-batch-backend 测试运行计划 v0.1 (per 9/2 BA-W3-10/11, 2026-09-02 10:22 JST Mavis 接手代签)

> **创建日期**: 2026-09-02 10:22 JST
> **创建者**: 架构师(Mavis 接手 agent per DEC-008) 代签 Ulysses
> **状态**: 🟡 计划就绪 (等 Phase C SRE 介入 + k3s ulyssespc 节点注册恢复后实际跑)
> **关联**: `d3ca7be` (BA-W3-11 11 E2E 集成测试, 8/22 03:18 JST commit) + `0107d2d` (BA-W3-10 11 UT 基础测试) + STATUS-SNAPSHOT v0.6.24 §5.1 (Phase C 落地后解锁 cargo test --tests 实际跑)

## 0. 测试运行目标

把 `rgs-batch-backend` 已 commit 的 22 测试函数 (11 UT + 11 E2E) 从"cargo check --tests 0 error"状态推进到"cargo test --tests 全 PASS"状态。L1 派生约束 1 worker 1 crate 守护。

## 1. 22 测试函数清单 (per `tools/rgs-batch-backend/tests/integration_tests.rs`)

### 1.1 11 UT 基础测试 (per `0107d2d` BA-W3-10 commit, 8/22 03:18 JST)

| # | 测试函数 | 性质 | 依赖 |
|---|---|---|---|
| 01 | `test_01_dlq_exponential_backoff_retry_0` | 算法单元 | std |
| 02 | `test_02_dlq_exponential_backoff_retry_1` | 算法单元 | std |
| 03 | `test_03_dlq_exponential_backoff_retry_5` | 算法单元 | std |
| 04 | `test_04_dlq_exponential_backoff_capped_30s` | 算法单元 | std |
| 05 | `test_05_dlq_exponential_backoff_negative` | 算法单元 | std |
| 06 | `test_06_endpoint_json_schema_health` | 端点 JSON schema | serde_json |
| 07 | `test_07_endpoint_json_schema_version` | 端点 JSON schema | serde_json |
| 08 | `test_08_endpoint_json_schema_token_estimate` | 端点 JSON schema | serde_json |
| 09 | `test_09_endpoint_json_schema_tasks_list` | 端点 JSON schema | serde_json |
| 10 | `test_10_endpoint_json_schema_workers_list` | 端点 JSON schema | serde_json |
| 11 | `test_11_endpoint_json_schema_cron_stats` | 端点 JSON schema | serde_json |

**UT 特点**: 11 UT 全是 std + serde_json + uuid, **不依赖 DB**, 任何时候都能跑。Phase C 介入前可先跑这 11 个验证基础逻辑。

### 1.2 11 E2E 集成测试 (per `d3ca7be` BA-W3-11 commit, 8/22 08:07 JST)

| # | 测试函数 | 性质 | 依赖 |
|---|---|---|---|
| 12 | `e2e_01_dag_topology` | DAG 拓扑排序 (GAP-1) | rgs-web + DB |
| 13 | `e2e_02_rgs_web_bridge` | rgs-web 8788 桥接 (GAP-6) | rgs-web + DB |
| 14 | `e2e_03_system_health` | 系统健康度 (BA-W6-5) | DB |
| 15 | `e2e_04_olu_stats` | OLU 统计 (BA-W5-7) | DB |
| 16 | `e2e_05_credentials_audit` | 凭据审计 (BA-W5-6) | DB |
| 17 | `e2e_06_prometheus_12` | Prometheus 12 指标 (BA-W2-7) | rgs-web |
| 18 | `e2e_07_gap1_cross_batch_dag` | GAP-1 跨 batch DAG | rgs-web + DB |
| 19 | `e2e_08_gap6_rgs_web` | GAP-6 rgs-web 联动 | rgs-web |
| 20 | `e2e_09_t3_audit_event` | T-3 审计事件 | DB |
| 21 | `e2e_10_message_outbox` | message_outbox 重试 (BA-W6-4) | DB |
| 22 | `e2e_11_sub_task_lifecycle` | sub_task 全生命周期 (BA-W3-8/9) | DB |

**E2E 特点**: 11 E2E 全部依赖 rgs-batch-backend 启动 + (DB 或 rgs-web 或两者),**需要 Phase C 5 域 mTLS 部署完成 + 5 域 binary 起来 + DB 池接通**才能跑。

## 2. 测试运行前置条件

### 2.1 11 UT 前置 (Phase C 不依赖, 可立即跑)

- [x] `tools/rgs-batch-backend/tests/integration_tests.rs` 已 commit (`d3ca7be` + `0107d2d`)
- [x] `tools/rgs-batch-backend/Cargo.toml` 已 commit (actix-web 4 + tokio + tonic 0.12 + sqlx 0.7 macros)
- [x] cargo check --tests 0 error (L1 派生约束 1 worker 1 crate)
- [ ] **CD 到 `tools/rgs-batch-backend/` 跑 `cargo test --lib exponential_backoff endpoint_json_schema` 单独跑 11 UT** (per L1 派生约束, 60s 限时)

### 2.2 11 E2E 前置 (Phase C 介入后)

- [ ] **k3s ulyssespc 节点注册恢复** (per OPEN-QA v0.3 §7.1)
- [ ] **5 域 gRPC 业务级 mTLS 部署完成** (Phase C 5/5, per STATUS-SNAPSHOT v0.6.24 §5.1)
- [ ] **rgs-batch-backend 启动** + **rgs-web 启动** (8788) + **PostgreSQL 池接通** (per WBS v0.4.7 §1.1 BA-W3-12 E2E 真实 sqlx + 5 域)
- [ ] 启动后 `cargo test --test integration_tests e2e_` 跑 11 E2E

## 3. 测试运行命令 (L1 派生约束 1 worker 1 crate, 60s 限时)

### 3.1 11 UT 单独跑 (Phase C 不依赖)

```bash
cd D:/RustGameServer/tools/rgs-batch-backend
# L1 派生约束: cargo check 60s 1 次拿 status, 1 worker 1 crate
# 1 worker 1 crate: cargo 默认 1 worker, 不需要 --jobs 1
Start-Process cargo -ArgumentList @('test','--lib','exponential_backoff','endpoint_json_schema','--no-fail-fast') -RedirectStandardOutput 'cargo-test-ut-2026-09-02.log' -RedirectStandardError 'cargo-test-ut-2026-09-02.err' -PassThru
# 等待 cargo 退出 (后台跑, 看 task_output)
# L1 派生约束: 60s 限时, 超时不等
```

**预期结果**: 11/11 PASS (per L1 派生约束 cargo check 0 error + L11 build dir lock 防御)

### 3.2 11 E2E 完整跑 (Phase C 介入后)

```bash
# Phase C 5 域 mTLS 部署 + rgs-web 启动 + PG 池接通后
cd D:/RustGameServer/tools/rgs-batch-backend
Start-Process cargo -ArgumentList @('test','--test','integration_tests','e2e_','--no-fail-fast') -RedirectStandardOutput 'cargo-test-e2e-2026-09-02.log' -RedirectStandardError 'cargo-test-e2e-2026-09-02.err' -PassThru
```

**预期结果**: 11/11 PASS (per BA-W3-11 8/22 03:18 JST cargo check --tests 0 error 验证)

### 3.3 22 测试函数全跑

```bash
cd D:/RustGameServer/tools/rgs-batch-backend
Start-Process cargo -ArgumentList @('test','--tests','--no-fail-fast') -RedirectStandardOutput 'cargo-test-all-2026-09-02.log' -RedirectStandardError 'cargo-test-all-2026-09-02.err' -PassThru
```

**预期结果**: 22/22 PASS

## 4. 测试运行后动作

| 结果 | 动作 |
|---|---|
| 11 UT 全 PASS | commit `test(batch-backend): UT 实际跑 11/11 PASS (per BA-W3-10 9/2 验证 cargo test --lib), 派生约束 L1 1 worker 1 crate` |
| 11 E2E 全 PASS | commit `test(batch-backend): E2E 实际跑 11/11 PASS (per BA-W3-11 9/2 验证 cargo test --test integration_tests), 派生约束 L1 1 worker 1 crate + Phase C 5 域 mTLS 部署完成` |
| 22 全 PASS | commit `test(batch-backend): 22 测试函数全 PASS (per WBS v0.4.7 §1.1 3 项外部依赖全解锁), 派生约束 L1 + L11 + L14` |
| 任何 FAIL | 不 commit, 把 log + err 文件发给 Ulysses 拍板 |

## 5. 派生约束守护

- **L1** cargo check 60s 1 次拿 status, 1 worker 1 crate (cargo test 同 L1 限时)
- **L11** build dir lock 防御, 隔离 target dir (cargo test 跑前确保 .worktrees/feat-auto 老目录不冲突)
- **L12** 临时 log / .txt 不入 commit (cargo test log + err 文件放 L12 临时目录, 不入 commit)
- **L14** plumbing 节点字符串 brace 跟踪 (per AGENTS.md L14 派生约束, 测试代码 if let / match 表达式 brace 配对跟踪)

## 6. 关联 commit + 文档引用

- `d3ca7be` (BA-W3-11 11 E2E 集成测试, 8/22 08:07 JST, cargo check --tests 0 error)
- `0107d2d` (BA-W3-10 11 UT 基础测试, 8/22 03:18 JST)
- STATUS-SNAPSHOT v0.6.24 §1 (22 测试函数 cargo check --tests 0 error 状态)
- STATUS-SNAPSHOT v0.6.24 §5.1 (Phase C 落地后解锁 cargo test --tests 实际跑)
- WBS v0.4.7 §1.1 (3 项外部依赖含 BA-W3-12 E2E 真实 sqlx + 5 域)
- RGS-DB-PARTITIONED-DRAFT-REVIEW-CHECKLIST v0.1.1 §5 (实施前置条件)

**修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
**审批**: 架构师(Mavis 接手 agent per DEC-008)
**代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)

## 7. 修订历史

| 版本 | 日期 (JST) | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-02 10:22 | 架构师(Mavis 接手 agent per DEC-008) | 初版: 22 测试函数运行计划 (11 UT + 11 E2E 清单, 3 步运行命令: UT 立即 / E2E Phase C 介入后 / 全跑), L1 + L11 + L12 + L14 派生约束守护, 代签 per 8/27 19:39/20:56/21:59 JST 三次强化 |
