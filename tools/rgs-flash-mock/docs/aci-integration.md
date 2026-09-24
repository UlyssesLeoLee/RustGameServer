# RGS ACI 集成设计 v0.1

> **Issue**: ULYS-191 §4.3.1 (ULYS-226, ULYS-191.3)
> **日期**: 2026-09-24 JST
> **前置**: ULYS-191.1 (aci-emitter v0.1.0) ✅ SHIPPED 2026-09-23 22:55 JST
> **同 Stage 3.0 同模式**: ULYS-191.2 (IDE1.0 最小骨架) ✅ SHIPPED 2026-09-24 04:43 JST (PR #2 merged)

## 1. 背景

**目的**: 为 **RustGameServer (RGS)** 项目落地 `aci-emitter v0.1.0` 集成 — 复用现有 `tools/rgs-flash-mock/` standalone Rust crate, 加 `.aci.json` schema + emit ACI assertion 的最小可交付件.

**关键验证点** (§4.3 5 项目跨项目集成之第 1 个):
- 跨 monorepo (RGS 有 32-crate workspace) 接 git dep `aci-emitter`
- 加 `.aci.json` schema 1:1 拷贝 + emit 1 条 sample assertion
- 跑通现有 regression 脚本 + 与 `aci-summary` CLI 兼容

**与 ULYS-191.2 IDE1.0 差异**:
- IDE1.0: docs-only 项目, ide-cli (Rust CLI), 走 `cargo run --aci-emit` 模式
- **RGS: server-side Rust crate** (rgs-flash-mock HTTP server), smoke 走 `cargo test --nocapture` 间接 emit

## 2. 集成路径

### 2.1 git dep 锁定

```toml
# tools/rgs-flash-mock/Cargo.toml
# ACI 跨项目 emitter (per ULYS-191 §4.3.1 brief v0.1)
# 守门 #11 缺标比错标: git dep 锁 rev=df28c56 (= ULYS-191.1 / ULYS-224 main HEAD)
aci-emitter = { git = "https://github.com/UlyssesLeoLee/aci-emitter", rev = "df28c56" }
```

### 2.2 模块接线

- `src/aci_emitter_helper.rs` (~80 LOC): 1 个公开函数 `emit_smoke_assertion()` + 1 个 `emit_smoke_assertion_json()`
- `lib.rs` 加 `pub mod aci_emitter_helper;`
- `tests/aci_integration.rs` 3 个 IT (schema roundtrip + emit helper + cross-language parity)
- `scripts/aci-smoke.sh` 4 step verify
- `docs/aci-integration.md` (本文件) + `docs/regression-report-2026-09-24.md`

### 2.3 `.aci.json` schema

1:1 拷贝自 Star `tools/star-flash-mock/.aci.json` v0.1 (4,544 B):
- 17 个 `expect_value_types`
- 6 个 `scope_dimensions` (project / module / domain / subdomain / operation / http_method)
- 10 个 `schema_required_fields` (assertion_id / aci_version / layer / scope / expect / actual / status / severity / reasoning / captured_at)
- `project` 字段重命名为 `rgs-flash-mock`

## 3. CI 跨项目验证

### 3.1 本笔 (本机守门)

5 项验收 (per brief v0.1 §4):
1. `cargo fmt --all -- --check` (RGS workspace) — exit 0
2. `cargo check -p rgs-flash-mock --all-targets -j 2` — exit 0
3. `cargo clippy -p rgs-flash-mock --all-targets -j 2 -- -D warnings` — exit 0
4. `cargo test -p rgs-flash-mock --all-targets -j 2` — ≥3 测试全过
5. `bash tools/rgs-flash-mock/scripts/aci-smoke.sh` — 4 step verify 全过

### 3.2 跨项目 CI (后续 §4.5)

- RGS CI 在 integration job 加 `cargo update -p aci-emitter` 检测上游漂移
- 上游 `aci-emitter` release tag 通知
- §4.5 跨项目 CI 落地后, 改 `crates.io` publish 或 path 依赖

## 4. 未来扩展

| Stage | 范围 | 备注 |
|---|---|---|
| **§4.3.1 (本笔)** | RGS 集成最小可交付 (7 文件) | ✅ ULYS-226 (本笔) |
| §4.3.2 | CATs 集成 (接 cats-mock) | ULYS-227 / 后续 brief |
| §4.3.3 | IM1.0 集成 (接 im-testkit) | ULYS-228 / 后续 brief |
| §4.3.4 | Ada 集成 (接 ada-mock) | ULYS-229 / 后续 brief |
| §4.3.5 | GitGit 集成 (TS emitter) | ULYS-230 / 后续 brief |

## 5. 风险 (4 项)

| # | 风险 | 缓解 |
|---:|---|---|
| R-1 | **rgs-flash-mock 独立 workspace 与 RGS main 32-crate 不共享 deps**: `aci-emitter` 加到 rgs-flash-mock 不会传播到 RGS main 32 crates. | (a) 本笔仅影响 rgs-flash-mock 子项目 (b) 后续 brief 给 RGS main 也加 (c) §4.5 跨项目 CI 统一 |
| R-2 | **git dep aci-emitter 跨项目漂移**: RGS 锁 rev=`df28c56`, 但上游 master 推进时 RGS dev/feature 分支可能用旧版本. | (a) RGS CI 在 integration job 加 `cargo update -p aci-emitter` (b) 上游 release tag 通知 (c) §4.5 跨项目 CI 落地 |
| R-3 | **rgs-flash-mock 是独立 workspace, RGS main cargo check 不会触发 rgs-flash-mock 检查**: RGS CI 可能仅 main workspace, 不查 rgs-flash-mock. | (a) 本笔先本机 5 项验收 (b) RGS CI 配置不在本笔范围, 由 D-Boy 后续 brief 改 |
| R-4 | **mock_data 51 fixtures 不改**: 用户可能误以为 RGS 已"完整 ACI 集成", 但实际只 smoke 1 条. | README + 本文档显式标注 "Stage 3.1 最小可交付, 51 fixtures 全量加 aci_assertion 待 Stage 3.2+" |

## 6. 已知缺口 (3 项 G-ACI)

| # | 缺口 | 缓解 |
|---:|---|---|
| G-ACI-03 | 跨语言 emitter ≥4 种 (Python ✅, Bash ✅, Rust ✅, TS ⏳ §4.3 GitGit). 本笔 RGS 用 Rust, 不引入新语言. | ⏳ TS emitter 单独 brief (ULYS-191.7) |
| G-ACI-07 | 部分项目可能无 mock. RGS **有完整 mock** (rgs-flash-mock), 本笔不全面改 51 fixtures. | ⏳ Stage 3.2 后续 brief 全量加 aci_assertion |
| G-ACI-09 (新) | **rgs-flash-mock 独立 workspace 与 RGS main 不共享**: RGS CI 可能仅 main 32-crate, 不查 rgs-flash-mock. | (a) 本笔 R-3 缓解 (b) 后续 brief 改 RGS CI 加 rgs-flash-mock check |

## 7. 守门对齐 (13 项)

| 守门 | 本笔落地 |
|---|---|
| #5 no secret leak | RGS 0 secret (.env 不动, design docs 已存在) |
| #6 中文默认 | docs 全中文 |
| #7 unsafe_code="forbid" | rgs-flash-mock 已有 (per Cargo.toml L78), 本笔不动 |
| #9 subprocess | aci-smoke.sh 调 `cargo build/test`, 不调任意用户脚本 |
| #10 author=Ulysses | `git -c user.name=Ulysses -c user.email=ulysses@mavis.local commit ...` |
| #11 缺标比错标 | 1 git dep `aci-emitter` 锁 rev=`df28c56`, `serde`/`serde_json` 已在 rgs-flash-mock 现有 deps |
| #12 docs 同步 | 1 aci-integration.md + 1 regression-report.md 随代码 ship |
| #13 W/T/M | 单元 (aci_emitter_helper 单测) + 集成 (3 IT) + 系统 (aci-smoke.sh 间接通过 cargo test) |
| #14v4 PR merge | 1 commit → RGS dev → CI → D-Boy 拍板 |
| #15 scope creep | 1 sub-agent 1 切点 (本笔 = RGS 集成最小, 不含 CATs/IM1.0/Ada/GitGit) |
| #17 commit 完整 | 1 commit 含 7 文件 |
| #19v19 Python 化 | IT-3 `test_aci_emitter_v0_1_compatibility` 跨语言 parity 测 |
| #24 vendor 中立 | 1 git dep `aci-emitter` (自家) + 复用现有 17+ deps (actix/tonic/rustls 等) |
