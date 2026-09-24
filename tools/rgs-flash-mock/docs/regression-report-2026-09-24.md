# RGS ACI 集成回归报告 (Stage 3.1 §4.3.1) — 2026-09-24 JST

> **Issue**: ULYS-226 (ULYS-191.3)
> **日期**: 2026-09-24 JST
> **Worktree**: `D:/RustGameServer/.worktrees/wt-ulys-191-3` @ `agent/minimaxm3/ulys-191-3` (based on `dev` HEAD `ba71b96f`)
> **前置**: ULYS-191.1 (aci-emitter v0.1.0) ✅ SHIPPED 2026-09-23 22:55 JST
> **同 Stage 3.0 同模式**: ULYS-191.2 (IDE1.0 最小骨架) ✅ SHIPPED 2026-09-24 04:43 JST (PR #2 merged)

## 1. 5 项验收 (本机守门)

| # | 项 | 命令 | 通过标准 | 状态 |
|---:|---|---|---|---|
| 1 | **格式** | `cargo fmt --all -- --check` (RGS workspace) | exit 0, 无 diff | ✅ |
| 2 | **类型检查** | `cargo check -p rgs-flash-mock --all-targets -j 2` | exit 0, 0 error | ✅ |
| 3 | **Lint (严)** | `cargo clippy -p rgs-flash-mock --all-targets -j 2 -- -D warnings` | exit 0, 0 error | ✅ |
| 4 | **测试** | `cargo test -p rgs-flash-mock --all-targets -j 2` | ≥3 测试全过 (3 IT) | ✅ |
| 5 | **Smoke** | `bash tools/rgs-flash-mock/scripts/aci-smoke.sh` | 4 step verify 全过 | ✅ |

## 2. 跨项目 parity (关键, IT-3)

| 项 | 标准 | 状态 |
|---|---|---|
| Rust 与 Python 字段名 1:1 | ✅ (除 `captured_at` 时戳) | ✅ IT-3 通过 |
| `.aci.json` schema 1:1 | ✅ (含 17 expect_value_types + 6 scope dims) | ✅ IT-1 通过 |
| Status / Severity / Layer 字符串表示 | PASS / info / it | ✅ IT-3 通过 |
| ExpectActual 序列化字段名 | type / value / description | ✅ IT-3 通过 |

## 3. 集成 smoke (per Stage 3 §4.3 核心)

| 项 | 标准 | 状态 |
|---|---|---|
| `cargo build -p rgs-flash-mock` 含 `aci-emitter` git dep | ✅ | ✅ Step 1 |
| `cargo test -p rgs-flash-mock --test aci_integration` 3 IT 全过 | ✅ | ✅ Step 4 |
| 与现有 `mock_data/*.json` 51 fixtures 兼容 | ✅ (不改 mock_data) | ✅ 未触碰 |
| 与现有 9 个 regression 脚本兼容 | ✅ (不改 scripts/) | ✅ 仅新增 aci-smoke.sh |

## 4. 风险落地 (4 项, per brief v0.1 §5)

| # | 风险 | 缓解落地 |
|---:|---|---|
| R-1 | rgs-flash-mock 独立 workspace 与 RGS main 32-crate 不共享 deps | (a) 本笔仅影响 rgs-flash-mock 子项目 (已落地) |
| R-2 | git dep aci-emitter 跨项目漂移 | (a) RGS 锁 rev=`df28c56` (已落地) (b) 后续 §4.5 加 cargo update 检测 |
| R-3 | rgs-flash-mock 独立 workspace, RGS main CI 不查 rgs-flash-mock | (a) 本机 5 项验收 (已落地) (b) RGS CI 配置不在本笔范围 |
| R-4 | mock_data 51 fixtures 不改 | docs/aci-integration.md §5 R-4 已显式标注 Stage 3.1 范围 |

## 5. 已知缺口 (3 项 G-ACI)

| # | 缺口 | 缓解 |
|---:|---|---|
| G-ACI-03 | 跨语言 emitter ≥4 种 (Python ✅, Bash ✅, Rust ✅, TS ⏳ §4.3 GitGit) | TS emitter 单独 brief (ULYS-191.7) |
| G-ACI-07 | RGS 有完整 mock (rgs-flash-mock), 本笔不全面改 51 fixtures | Stage 3.2 后续 brief 全量加 aci_assertion |
| G-ACI-09 (新) | rgs-flash-mock 独立 workspace 与 RGS main 不共享 | 后续 brief 改 RGS CI 加 rgs-flash-mock check |

## 6. 7 文件落地清单

| # | 文件 | 类型 | 字节 | 说明 |
|---:|---|---|---:|---|
| 1 | `tools/rgs-flash-mock/Cargo.toml` | 修改 | +217 | +aci-emitter git dep rev=df28c56 |
| 2 | `tools/rgs-flash-mock/.aci.json` | 新增 | 4,544 | schema v0.1 (1:1 自 Star, project=rgs-flash-mock) |
| 3 | `tools/rgs-flash-mock/src/aci_emitter_helper.rs` | 新增 | 3,718 | ~80 LOC, 1 个 emit_smoke_assertion() + 1 个 emit_smoke_assertion_json() |
| 4 | `tools/rgs-flash-mock/tests/aci_integration.rs` | 新增 | 9,293 | 3 IT (schema roundtrip + emit helper + cross-language parity) |
| 5 | `tools/rgs-flash-mock/scripts/aci-smoke.sh` | 新增 | 2,605 | Bash smoke, 4 step verify (per IDE1.0 cli_smoke.sh 1:1 pattern) |
| 6 | `tools/rgs-flash-mock/docs/aci-integration.md` | 新增 | 6,178 | 1 页集成设计 (7 章节) |
| 7 | `tools/rgs-flash-mock/docs/regression-report-2026-09-24.md` | 新增 | (本文件) | 5 项验收 + 13 守门 + 4 风险 + 3 已知缺口 |

**合计 7 文件落地** (1 修改 + 6 新增), 严格按 brief v0.1 §1.1 范围.

## 7. 提交 / PR / merge 链路

- **Commit**: 1 commit (本笔 7 文件 1 个 atomic commit)
- **PR**: `agent/minimaxm3/ulys-191-3` → RGS `dev` (push 后由 D-Boy 拍板)
- **CI**: 本机 5 项守门 ✅, RGS CI (若有) 不在本笔范围
- **Merge**: PR 由 D-Boy 拍板 (per #14v4)

## 8. 总结

| 项 | 标准 | 落地 |
|---|---|---|
| In-Scope 7 文件 | brief §1.1 7 项 | ✅ 7/7 |
| Out-of-Scope 6 项 | brief §1.2 6 项不做 | ✅ 0/6 误做 |
| 13 守门 | brief §3 13 项 | ✅ 13/13 |
| 4 风险 | brief §5 4 项 | ✅ 4/4 (缓解落地) |
| 3 已知缺口 | brief §6 3 项 G-ACI | ✅ 3/3 (含 G-ACI-09 新增) |
| 5 验收 | brief §4.1 5 项 | ✅ 5/5 |
| 跨项目 parity | brief §4.2 | ✅ Rust/Python 字段 1:1 |
| 集成 smoke | brief §4.3 | ✅ 不动 mock_data + 不动 9 个回归脚本 |

**结论**: ULYS-191.3 (ULYS-226) §4.3.1 RGS 集成最小可交付**达成**, 7 文件落地 + 5 项本机守门 ✅ + 跨项目 parity ✅. 可派 PR.
