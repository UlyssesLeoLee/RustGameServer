# RGS-COVERAGE-BASELINE-2026-08-26

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-COVERAGE-BASELINE-2026-08-26 |
| 日期 | 2026-08-27T08:1x JST（实际生成时间见 commit 时间戳） |
| 工具 | cargo-llvm-cov 0.9.0（Rust 1.98-x86_64-pc-windows-msvc, Windows） |
| 触发 | RGS-TEST-STRATEGY-2026-08-26 v0.1 §3.1 P0 + commit 30a8842 编译修复后 |
| 命令 | `cargo llvm-cov --lib --workspace --summary-only`（--lib 模式，跳过 integration tests that spawn binaries） |
| 实测 commit | 30a8842135c23503abad754037964117a75d31f9（cluster-ops 编译修复） |

## 1. Workspace 汇总（lib only）

| 指标 | 数字 |
|---|---|
| Total regions | 11,059 |
| Regions covered | 7,947 |
| **Region coverage** | **71.86%** |
| Total functions | 1,267 |
| Functions covered | 752 |
| **Function coverage** | **59.35%** |
| Total lines | 7,322 |
| Lines covered | 5,080 |
| **Line coverage** | **69.38%** |
| **Branch coverage** | **未测**（需 `--branch` flag；本次因 incremental cache + 冷编译耗时未启用，列入 v0.2 P0 增量补做） |

> 与 v0.1 §1.1 估计对比：
> - Line 60-69% vs 估 40-55%：**实际比估计高**（lib-only 数据比估的 "全量含 bin" 更聚焦可测代码）
> - Function 59% vs 估 60-70%：**吻合**（略低 1-10pp）
> - Branch 未知 vs 未知：本次仍未出数字，保留 v0.1 的 "未知" 标记

## 2. Per-crate 覆盖率（line / function / region, lib only）

| Crate | Line% | Function% | Region% | 备注 |
|---|---|---|---|---|
| rgs-arc-olu | 100.00% | 100.00% | 100.00% | 22 行, 2 fn, lib 极小; 全部覆盖 |
| cluster-ops | 80.17% | 71.79% | 82.75% | 含 error.rs 100% 覆盖 (commit 9a5cc50 22:18 JST); drill/saga 已临时禁用 (commit 30a8842) |
| economy-service | 79.36% | 64.77% | 80.68% | 含 entity 100% / service 93% / saga_orchestrator 96%; repository 34% 偏弱 |
| shared-platform | 73.32% | 72.69% | 74.04% | 18 文件, 2854 region; rbac/subject/tls 100% 边界; tracing_init 仅 6.62% |
| admin-service | 73.51% | 53.77% | 75.60% | entity 99% / service 90%; repository 42% 偏弱; db 36% |
| player-service | 71.81% | 64.04% | 76.05% | 与 admin 接近; repository 偏弱 |
| social-service | 65.74% | 57.29% | 68.41% | repository 47% 偏弱; entity 100% |
| match-service | 62.85% | 54.46% | 63.70% | 最低的 5 域; db 45% 偏弱 |
| function-plane | 7.59% | 5.97% | 13.68% | **仅 registry 51%**, 其余 4 文件 0% 覆盖 |
| rgs-testkit | 6.99% | 7.14% | 7.39% | 仅 pg_test_db 47%; fixture/helper/mock 0% |
| rgs-hello | — | — | — | **无 lib**（仅 bin, --lib 不覆盖） |
| rgs-certgen | — | — | — | **无 lib**（仅 bin, --lib 不覆盖） |

## 3. 关键发现

### 3.1 0 覆盖文件清单（lib only，line+function 全 0%）

| Crate | File | Region | Function | Line |
|---|---|---|---|---|
| cluster-ops | src/db.rs | 38 | 8 | 31 |
| function-plane | src/lib.rs | 3 | 1 | 3 |
| function-plane | src/contract.rs | 67 | 12 | 78 |
| function-plane | src/gateway.rs | 86 | 10 | 61 |
| function-plane | src/wasm_host.rs | 210 | 23 | 155 |
| rgs-testkit | src/fixture.rs | 101 | 19 | 100 |
| rgs-testkit | src/helper.rs | 41 | 4 | 26 |
| rgs-testkit | src/mock.rs | 98 | 24 | 71 |

合计 644 region / 101 function / 525 line 完全无覆盖。

### 3.2 低覆盖率热点（line < 50%）

- **function-plane 全 crate 7.59%**：4/5 文件 0% 覆盖，仅 registry.rs 38.89%
- **rgs-testkit 6.99%**：3/4 文件 0% 覆盖（fixture/helper/mock 整套 test 辅助未测）
- **5 域 repository.rs 集体 30-50%**：admin 42% / economy 34% / match ~45% / player ~45% / social 47%
- **5 域 db.rs 集体 36-46%**：5 域几乎一致 45±2%，几乎全是 sqlx init / migration 路径未触达
- **shared-platform 几个低洼**：tracing_init 6.62% / messaging 38.71% / json_logging 40.91% / span_helpers 50% / outbox_relay 55.19%

### 3.3 与 v0.1 §1.1 估计对比

- v0.1 估 line 40-55%，实测 **69.38%**（lib only）— **比估高 ~14-29pp**
- v0.1 估 function 60-70%，实测 **59.35%** — **吻合下沿**
- v0.1 估 branch 未知，实测仍未出 — 保留 "未知"
- **解读**：lib-only 数据比"全量含 bin" 略偏高是预期的（bin 通常无 unit test），但 **v0.1 估计明显偏保守**，建议 v0.2 §1.1 把估计区间上调为 55-70% line

## 4. 下一步建议（P1 起点）

### 4.1 P1 最高 ROI 补测文件（5 个，per 0% / < 50% / 业务核心度综合）

1. **rgs-testkit/src/{fixture.rs, mock.rs}**（100+71=171 line 0 覆盖）— P1 阶段会被广泛使用，**不测自己会污染所有下游测试可信度**，最高优先级
2. **function-plane/src/wasm_host.rs**（155 line 0 覆盖）— WASM 业务核心，per RGS-INC-001 v0.2 §8/§9/§15；需 P1 写 wasmtime mock 集成测试
3. **5 域 repository.rs**（5 文件合计 ~800 line，30-50% 覆盖）— DB persistence 层；可写 rgs-testkit 驱动的 in-memory repository 单测补到 80%+
4. **cluster-ops/src/db.rs**（31 line 0 覆盖）— drill/saga 重写后（**待办**）才能测；不进 P1
5. **shared-platform/src/tracing_init.rs**（103 line 仅 6.62%）— tracing init 单测可补，per 53.12 OTel SDK 接入边界

### 4.2 P1 工期预估

- rgs-testkit fixture/mock 单测：0.5-1.0 人·天（@ token 算约 50K-150K tokens）
- function-plane wasm_host 集成测试：1.5-2.0 人·天（含 wasmtime mock 搭建）
- 5 域 repository 单测：2.0-3.0 人·天（每域 0.4-0.6，rgs-testkit in-memory 模板复用）
- tracing_init 单测：0.3-0.5 人·天
- **合计 4.3-6.5 人·天**，建议 P1 周期按 **5-6 人·天** 排期（@ token: 500K-1.5M tokens）

### 4.3 P1 范围外（不进入）

- **cluster-ops drill/saga 重写**（per 30a8842 commit 末尾未决项）：**待办** 单独开 worker，不在 P1 测试覆盖范围
- **rgs-hello / rgs-certgen**（无 lib）：bin-level 覆盖率需要 `--bin` flag 单独跑；可作 P1 增量任务

## 5. 实测命令 + commit hash

```bash
cargo llvm-cov --lib --workspace --summary-only
```

实测 commit: 30a8842（cluster-ops 编译修复）

## 6. 已知缺口 / Limitations

1. **本次只跑 `--lib`**，未跑 integration tests（`tests/*.rs` 5 域 + cluster-ops）。原因：
   - admin-service `fail_closed_start` 在 `--workspace` 模式下 spawn 子进程时 panic（exit 0xffffffff，普通 `cargo test -p admin-service` 单独跑 OK）
   - 完整 workspace 模式冷编译 30+ 分钟未完成，warm incremental cache 启动后改用 `--lib` 解决
   - **影响**：5 域 `tests/` 目录里的 `integration_*_basic.rs` / `chaos_*.rs` / `integration_outbox.rs` 等集成测试覆盖率未计入
2. **Branch coverage 未测**：本次未加 `--branch` flag；v0.1 §1.1 估计里 branch 也是 "未知"，本次保留此状态
3. **rgs-hello / rgs-certgen 无数据**：仅 bin 无 lib，--lib 不覆盖
4. **cluster-ops 0% 文件 (db.rs) 与 v0.1 §3.1 P0 不冲突**：drill/saga 重写待办，per 30a8842 commit 末尾未决项
5. **Cargo.lock 改动**：cargo llvm-cov 跑后 cargo 自动 update（5 insertions / 265 deletions，主要是 deps 版本收敛），随报告一并 commit

## 7. 修订历史

| 版本 | 日期 | 修订者 | 变更 |
|---|---|---|---|
| v0.1 | 2026-08-27 JST | 架构师(Mavis 接手 agent per DEC-008) | 初版：P0 baseline 落地（lib-only, 10/12 crate 覆盖, 8 文件 0% 标记, P1 起点建议） |
