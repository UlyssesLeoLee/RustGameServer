### 本批次跑的是什么

- **mock 项目自身回归**（不是根 workspace 31 crate 的并行 lane）—— 2026-10-21 07:24-07:5x JST 实跑
- 两条轨道都跑：
  1. **CI 口径 gate**（`scripts/mock-ci-gate.sh`）—— 把 `.github/workflows/flash-mock-ci.yml`
     的 7 个 step 原样搬下来本地复现：编译门禁 / cargo test / python unittest / mock_switch
     CLI exit-code 规范 / bash -n 语法 / 缺口闭环 4 脚本 / 跨域故事链+事件链
  2. **整体回归 orchestrator**（`scripts/regression-test-all.sh`）—— UT → IT（60 module fixture
     + 9 个回归脚本）→ ST（mock server 起服 + /health /ready + 23 RPC 抽样）

### 为什么先做 CI 口径 gate

根 workspace 的并行 lane 回归（2026-10-20 dev-gate 批次）**覆盖不到** mock 项目自身：
`tools/rgs-flash-mock` 是独立 `[workspace]`，不在根 `Cargo.toml` members 内，
它自己的 `src/` + `tests/`（含 `saga_verify.rs` / `saga_story_chain.rs`）由
`.github/workflows/flash-mock-ci.yml` 单独管。上批 notes 里记的"mock 项目自身不在覆盖范围内"缺口，
本批次用本地 gate 脚本补上了。

### 机器约束（踩过的坑，写下来备查）

- **RAM**: 整机 31.8 GB，但常态 free 仅 0.3~1.8 GB（commit 80.5/85.5 GB）。
  cargo 默认 `-j 24` 会把 rustc 挤爆，表现为 `can't find crate for std` 这类**假失败**。
  本批次统一用 `CARGO_BUILD_JOBS=4` / `MOCK_GATE_JOBS=4` 限流。
- **cargo 不在 Git Bash 默认 PATH**: `C:\Windows\system32\bash.exe` 是 WSL 转发，
  本批次统一用 Git Bash（`C:\Program Files\Git\bin\bash.exe`），
  并显式 `export PATH="/e/DevCache/cargo/bin:$PATH"`。
  嵌套 `bash -c "cargo check ..."` 会继承导出的 PATH —— 这是 UT 秒挂 rc=127 的根因。
- **target dir**: 一律 E 盘 `E:/DevCache/cargo/mock-baseline`（D 盘不写，per AGENTS.md L11/L30）。
  `mock-saga` 不是完整缓存（无 mock 二进制），`mock-baseline` 才是（10-21 01:13 构建）。

### 覆盖边界（缺标比错标）

- 本批次**不含**真实 k3s 集群 / 真实域服务 / WASM runtime 层验证。
  9 个回归脚本全是 fixture 驱动的离线校验，mTLS 相关步骤校验的是 **fixture 与 k8s yaml 是否落档**，
  不是真实握手。
- 根 workspace 31 crate 的业务回归仍以 2026-10-20 dev-gate 批次为准，本批次不重复。
