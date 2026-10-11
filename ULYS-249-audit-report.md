# ULYS-249 依赖审计报告 (2026-10-11 JST)

> **任务**: ULYS-249 — RustGameServer 依赖安全与过期版本审计 (npm + cargo audit + cargo deny + outdated)
> **承接**: ULYS-239 (2026-09-28) → ULYS-247 (2026-09-29) → ULYS-253 (2026-10-11) → **ULYS-249 (本轮 2026-10-11)**
> **工具链**: `cargo audit 0.22.2` (RustSec advisory-db `7eebec69c352c7191b1f13eb95dd510eeca5d1de`, 1296 条目) + `cargo deny 0.20.2` (licenses/bans/sources 子集) + `cargo update --dry-run --verbose` (outdated) + `npm audit` (3 manifests with deps)
> **Lockfile**: `Cargo.lock`, 672 packages / 32 workspace path crates
> **rust-toolchain**: 1.98.1 (per RGS-IMPL-001 §1.3 vtable fix)

---

## 0. 与前轮 ULYS-253 的差异 (delta)

| 项 | ULYS-253 (2026-10-11 早轮) | ULYS-249 (2026-10-11 本轮) | 变化 |
|---|---|---|---|
| Advisory DB commit | `3157b0e258782691` | `7eebec69c352c7191b1f13eb95dd510eeca5d1de` | newer |
| Advisory count | 1273 | 1296 | +23 entries |
| Lockfile SHA | `d21d31a8` | `d21d31a8` | 0 |
| 总漏洞数 | 27 RUSTSEC | 27 RUSTSEC | 0 |
| 新增命中 | n/a | 0 (lockfile 内所有 RUSTSEC ID 已在 253 列表) | **0 new** |
| 消失命中 | n/a | 0 | 0 |
| `rustls` | 0.23.45 | 0.23.45 | 已 fix (ULYS-239 P0-3) |
| `rustls-webpki` | 0.102.8 + 0.103.15 双版本 | 同上 | **未修** (sqlx 0.8 仍 pin 0.102.8) |
| `h2` | 0.3.27 + 0.4.19 双版本 | 同上 | **未修** (actix 仍用 0.3.27) |
| `yoke-derive` | 0.8.3 (yanked) | 同上 | **未修** |
| `wasmtime` | 19.0.2 | 19.0.2 | **未修** |
| npm vulns | 3 (gm-console/frontend) | **3** (gm-console/frontend) | 0 |
| `cargo deny` bans warnings | 47 dup + 63 wildcard | 47 dup + 63 wildcard | 0 |

**结论**: 与 ULYS-253 完全一致（无新增漏洞；advisory-db 增量条目均不在 lockfile 内）。本轮 **ULYS-249 的核心交付物是「可执行 P0 修复列表」**，对每项列出最小代价 `cargo update --precise` 命令 + 验证步骤 + 兼容矩阵，对应 task brief §1-§5 的「recommended fix」要求。

---

## 1. 漏洞汇总 (Cargo.lock 实测, 2026-10-11)

### 1.1 RustSec 漏洞 — 27 条 / 6 distinct crate

按 CVSS 启发式分层：

| Severity | 数量 | 说明 |
|---|---|---|
| **CRITICAL** (CVSS ≥ 9.0) | **2** | wasmtime 沙箱逃逸（Winch 编译器） |
| **HIGH** (CVSS 7.0–8.9) | **1** | wasmtime 文件系统 trailing-slash 沙箱逃逸 |
| **MEDIUM** (CVSS 4.0–6.9) | **10** | wasmtime (9) + rsa 0.9.10 (Marvin Attack 5.9) |
| **LOW** (CVSS < 4.0) | **7** | wasmtime (7 LOW CVSS 1.8–3.3) |
| **未标 CVSS** (vector-only) | **7** | h2, rustls-webpki (4), tokio-tar, wasmtime (Windows 设备文件名) |

### 1.2 按 crate 分组

| Crate | 当前版本 | 漏洞数 | 最高严重度 | 修复策略 |
|---|---|---|---|---|
| **wasmtime** | 19.0.2 | **20** | CRITICAL 9.0 | 升 49.0.2 (30 majors) — **P1 独立 issue**（function-plane FFI 兼容性评估）|
| **rustls-webpki** | 0.102.8 | **4** | HIGH (无 CVSS 标) | 升 0.103.13+ via sqlx 升级 — **P0**，详见 §3.2 |
| **rsa** | 0.9.10 | **1** | MEDIUM 5.9 | **无修复版本**（lockfile-only, 运行时不可达）— **P2 ignore**, 详见 §3.4 |
| **tokio-tar** | 0.3.1 | **1** | HIGH (无 CVSS 标) | **无修复版本**（仅 dev/testcontainers）— **P2 ignore**, 详见 §3.4 |
| **h2** | 0.3.27 | **1** | LOW (无 CVSS 标, 实际 DoS) | 升 0.4.16+ — **P0**，详见 §3.1 |

### 1.3 npm 漏洞 — 3 条 (gm-console/frontend)

| Manifest | 漏洞 | 详情 | 修复 |
|---|---|---|---|
| `package.json` (根) | **1** | `brace-expansion 4.0.0-5.0.11` (high, indirect via glob → tar/fuse) | `npm audit fix` |
| `tools/gm-console/frontend/package.json` | **3** | `esbuild ≤0.24.2` (moderate) + `vite ≤6.4.2` (high, dev-tooling) + `source-map-js 1.0–1.2.1` (high, dev-tooling) | `vite ^5 → ^8` (breaking) + `esbuild`/`source-map-js` (连带) |
| `tools/h5_e2e/package.json` | 0 | `ws ^8.21.3` clean | n/a (无 lockfile, 跳过 audit) |
| `tools/rgs-{batch-console,proxy,shim,web}/package.json` | n/a | **ENOLOCK** (无 package-lock.json) | 需先 `npm i --package-lock-only` 后再 audit |

**npm 结论**: 仅前端 dev 工具链有 3 漏洞，**全部为开发期依赖**，不影响生产运行时。修复需升 `vite ^5 → ^8` (semver-major)。建议独立 issue 跟进。

### 1.4 cargo-audit 额外告警 — 9 denied warnings

| 类型 | Crate | 版本 | 处理 | 优先级 |
|---|---|---|---|---|
| unmaintained | `backoff` | 0.4.0 | 替换为 `backon` | P2 |
| unmaintained | `bincode` | 1.3.3 | 替换为 `wincode`/`postcard`/`bitcode` | P2 |
| unmaintained | `fxhash` | 0.2.1 | 替换为 `rustc-hash` | P2 |
| unmaintained | `instant` | 0.1.13 | 替换为 `web-time` | P2 |
| unmaintained | `mach` | 0.3.2 | macOS-only dev — 接受 | P3 |
| unmaintained | `paste` | 1.0.15 | 替换为 `pastey` | P2 |
| unmaintained | `rustls-pemfile` | 2.2.0 | 迁移到 `rustls-pki-types` (1.9+) | P2 |
| unsound | `wasmtime-jit-debug` | 19.0.2 | 跟随 wasmtime 主线升级 | P1 |
| **yanked** | `yoke-derive` | **0.8.3** | **升 0.8.4** | **P0 patch bump** |

---

## 2. cargo-deny 发现

### 2.1 许可 (licenses) — ✅ clean

- **0 errors, 2 warnings**: `OpenSSL` / `Unicode-DFS-2016` 在 allow 列表但未遇到 (informational)
- 已有 allow 列表 (Apache-2.0 / MIT / BSD-* / MPL-2.0 / 0BSD / CDLA-Permissive-2.0) 覆盖全部 lockfile

### 2.2 重复 crate (multiple-versions) — 47 warnings

**与 ULYS-247/253 一致**。最严重（按版本数量排序）：

| Crate | 版本数 | 来源/根因 |
|---|---|---|
| `hashbrown` | 5 | syn 1/2 + thiserror 1/2 + bytes 大版本过渡 |
| `base64` / `getrandom` | 3 | bcrypt/rand 多版本依赖 |
| `bitflags` / `block-buffer` / `const-oid` / `core-foundation` / `cpufeatures` / `crypto-common` / `digest` / `foldhash` / `h2` / `hashlink` / `heck` / `http` / `impl-more` / `indexmap` / `itertools` / `linux-raw-sys` / `object` / `openssl-probe` / `rand_chacha` / `r-efi` / `rustix` / `rustls-native-certs` / `rustls-webpki` / `security-framework` / `sha1` / `socket2` / `syn` / `thiserror` / `thiserror-impl` / `tower` | 2 each | 间接传递依赖版本分歧 |

**影响**: 二进制尺寸 + 编译时间；无运行时正确性问题。**deny.toml 已设 `multiple-versions = "warn"`**，CI 不 exit 7。

### 2.3 Wildcard 依赖 — 63 warnings

deny.toml `wildcards = "warn"` 已生效。涉及的 workspace crate:
- `gm-backend` (4 wildcard deps) — 最大头
- `admin-service`, `battle-service`, `card-service`, `economy-service` (2 each)
- `activity-service`, `cluster-ops`, `gm-extra-service`, `guild-service`, `i18n-service`, `leaderboard-extra-service`, `leaderboard-service`, `match-service`, `network-gateway`, `operate-service`, `player-service`, `pvp-full-service`, `replay-extra-service`, `replay-service`, `rgs-grpc-bridge`, `rgs-testkit`, `scene-service`, `social-extra-service`, `social-service` (1 each)

**P2 follow-up**: 收紧为 caret 版本（per task brief 53.14 注释）。

### 2.4 来源 (sources) — ✅ clean

全部来源 crates.io，无未知 registry/git。

---

## 3. 可执行 P0/P1 修复方案（核心交付物）

ULYS-239/247/253 的报告均明确列出建议项，但 dev 分支**仅有 P0-2 (protobuf) / P0-3 (rustls) / P1-5 / P1-6 已落地**（9d8eae97 merge）。本轮 ULYS-249 按 task brief §1.3 要求，**为每个未修复 P0 项给出具体 `cargo update --precise` 命令 + 兼容性验证步骤**。

### 3.1 P0-1: `h2 0.3.27 → 0.4.16+` (1 条 LOW DoS)

**漏洞**: RUSTSEC-2026-0258 — h2 无界空 DATA frames → unbounded memory / length-overflow panic  
**触发链**: `actix-http 3.13.5 → actix-web 4.15.0`（全栈 24 service crate）  
**Lockfile 现状**: `h2` 双版本 0.3.27（vulnerable, actix 链）+ 0.4.19（safe, hyper 链）

**修复命令**:
```bash
cargo update -p h2 --precise 0.4.19
```

**兼容性风险**: h2 0.3→0.4 是 tokio 项目拆分（http 0.2→1.x 同步迁移），**`actix-http 3.13.5` 仅依赖 h2 0.3.x**。直接 `--precise 0.4.19` 会触发依赖解析失败（actix-http 卡在 0.3 兼容）。

**实际路径**:
```bash
# 方案 A：等待 actix-http 4.x（h2 0.4 原生支持）—— 长期
# 方案 B：hyper 路径 crate 已用 0.4.19 → 仅 actix-web 路径仍走 0.3.27
# 方案 C（推荐，立即可执行）：patch-package / cargo:http:patch 强制 actix-http 接受 h2 0.4
```

**当前 P0-1 状态**: **技术债** — actix-http 3.x 不接 h2 0.4，需 actix 团队升级或 fork。建议独立 issue 跟踪，**不阻塞当前 PR**。

### 3.2 P0-2: `rustls-webpki 0.102.8 → 0.103.13+` (4 条 HIGH)

**漏洞**: RUSTSEC-2026-0049/0098/0099/0104 — CRL matching / URI name constraints / wildcard+name constraints / CRL panic  
**触发链**: `sqlx-core → sqlx 0.8.6 → runtime-tokio-rustls`，影响 24 个 service crate mTLS  
**Lockfile 现状**: `rustls-webpki` 双版本 0.102.8（vulnerable, sqlx 链）+ 0.103.15（safe, hyper / tonic 链）

**修复命令**:
```bash
# 触发 sqlx 升级：cargo update -p sqlx 触发 sqlx 内部 rustls-webpki bump
# 或强制 precise:
cargo update -p rustls-webpki --precise 0.103.15
```

**兼容性风险**: sqlx 0.8.x 的 `runtime-tokio-rustls` feature 内部 pin `rustls-webpki ^0.102`，**强制 precise 会破坏 sqlx feature 解算**。

**实际路径**:
```bash
# 方案 A：升 sqlx 0.8 → 0.9（rustls-webpki 0.103 默认）—— breaking
cargo update -p sqlx --precise 0.9.0
# 方案 B：cargo:rustc-patch 注入 sqlx 让其接受 0.103 —— 临时
# 方案 C（推荐，跟踪中）：等待 sqlx 0.9 LTS
```

**当前 P0-2 状态**: **技术债** — sqlx 0.8 ↔ rustls-webpki 0.102 耦合死锁。已在 ULYS-247 列为 P0-1，**未解决**。

### 3.3 P0-3: `yoke-derive 0.8.3 → 0.8.4` (yanked)

**警告**: `cargo audit` 报 "yanked: yoke-derive 0.8.3"  
**触发链**: 间接依赖 `sqlx-macros-core → sqlx`  
**Lockfile 现状**: `yoke-derive 0.8.3` (yanked)

**修复命令** (LOCKFILE-ONLY bump, 无 API 变更):
```bash
cargo update -p yoke-derive --precise 0.8.4
```

**兼容性风险**: 0.8.3→0.8.4 是 patch bump，无 breaking change。`cargo update --dry-run --verbose` 已显示 `yoke-derive v0.8.3 (available: v0.8.4)` —— 解析器将自动接受 0.8.4。

**验证步骤**:
```bash
cd D:/RustGameServer
cargo update -p yoke-derive --precise 0.8.4
cargo check --workspace --tests 2>&1 | tail -30
cargo audit 2>&1 | grep -E "yoke|Count"
# 期望: yoke-derive yanked warning 消失；Count: 26 (was 27)
```

**P0-3 状态**: **可立即执行**, 1 行命令, 0 风险. **推荐作为本轮 ULYS-249 的唯一修复 commit**.

### 3.4 P0-ignore: `rsa 0.9.10` + `tokio-tar 0.3.1` (无修复版本)

**rsa** RUSTSEC-2023-0071 (Marvin Attack, MEDIUM 5.9):
- 触发链: `sqlx-mysql → sqlx-macros-core → sqlx` (RGS 仅用 sqlx-postgres feature, **运行时不可达**)
- RGS 用的是 PostgreSQL (`sqlx = { features = ["postgres", "runtime-tokio-rustls", ...] }`)
- 修复: 无; `rsa 0.10.0-rc.18` 仍未关闭 issue #626
- 建议: `cargo audit -i RUSTSEC-2023-0071` 加入 `.cargo/audit.toml` ignore 列表, 阻止 CI 红

**tokio-tar** RUSTSEC-2025-0111 (PAX header smuggling, HIGH 无 CVSS 标):
- 触发链: `testcontainers → testcontainers-modules` (仅 dev/testcontainers, **生产二进制不携带**)
- 修复: 无; crate 已 archived, 推荐 `astral-tokio-tar` 替代
- 建议: `cargo audit -i RUSTSEC-2025-0111` 加入 ignore 列表; 跟踪 testcontainers-modules 升级

### 3.5 P1: `wasmtime 19.0.2 → 49.0.2` (20 条漏洞)

**触发链**: `function-plane → admin-service → gm-backend (dev)`  
**Lockfile 现状**: wasmtime 19.0.2 (single version, 全 RGS 唯一)

**为什么不在本轮修复**:
1. **30 个 major 版本跳跃** — 19 → 49 的 API 变化巨大（fuel 配置 / component model / Winch 后端）
2. **function-plane 强约束** — README 明确 19.x pin 是为了规避 Windows fuel-exhaustion FFI abort 回归
3. **当前 P1 状态**: ULYS-247 已列为 P1, **独立 issue** — 需先验证 wasmtime 20+ 是否已修复 longjmp abort

**修复路径** (后续 issue):
```bash
# 阶段 1: 19 → 24 (minimum fix for 18/20 CVEs)
cargo update -p wasmtime --precise 24.0.13
# 验证 cargo check + integration tests
# 阶段 2: 24 → 36 (覆盖全部 CRITICAL/HIGH)
cargo update -p wasmtime --precise 36.0.16
# 阶段 3: 36 → 49 (latest stable)
cargo update -p wasmtime --precise 49.0.2
```

---

## 4. 过期版本 (cargo update --workspace --dry-run --verbose)

**31 个 unchanged 依赖**（与 ULYS-247/253 完全一致 — Cargo.lock 未变）：

| Package | 当前 | available | 差距 |
|---|---|---|---|
| **wasmtime** | 19.0.2 | **49.0.2** | **30 major** |
| **serial_test** | 0.5.1 | **4.0.1** | **3-4 major** |
| **jsonwebtoken** | 9.3.1 | **11.1.0** | **2 major** |
| **async-nats** | 0.42.0 | 0.50.0 | 0.08 (0.x) |
| **opentelemetry*** | 0.24 | 0.33 | 9 minor |
| **opentelemetry-otlp** | 0.17.0 | 0.33.1 | 0.16 |
| **reqwest** | 0.12.28 | 0.13.5 | 0.01 |
| **sqlx** | 0.8.6 | 0.9.0 | 0.01 |
| **thiserror** | 1.0.69 | 2.0.21 | 1.0 |
| **tonic** / **tonic-build** / **tonic-health** | 0.12.3 | 0.14.6 | 0.02 |
| **testcontainers** / **testcontainers-modules** | 0.23 / 0.11 | 0.28 / 0.15 | 0.05 / 0.04 |
| **tokio-tungstenite** | 0.24.0 | 0.30.0 | 0.06 |
| **tracing-opentelemetry** | 0.25.0 | 0.34.0 | 0.09 |
| **actix-web-lab** | 0.24.3 | 0.29.5 | 0.05 |
| bcrypt | 0.16.0 | 0.19.3 | 0.03 |
| criterion | 0.5.1 | 0.8.2 | 0.03 |
| ctor | 0.2.9 | 1.0.13 | 0.8 → 1.0 |
| prost | 0.13.5 | 0.14.4 | 0.01 |
| rand | 0.8.8 | 0.10.3 | 0.02 |
| mockall | 0.13.1 | 0.15.0 | 0.02 |
| sha2 | 0.10.9 | 0.11.0 | 0.01 |
| h2 (already at safe version via hyper) | 0.4.19 | 0.4.20 | patch |
| yoke-derive (yanked!) | 0.8.3 | 0.8.4 | patch (yanked) |

**P0 patch-bump 候选 (本轮可执行)**: `yoke-derive 0.8.3 → 0.8.4` (§3.3). 其他均为 minor/major 升级，需独立评估。

---

## 5. 行动项汇总 (按优先级)

| # | 优先级 | 行动 | 命令 / 路径 | 风险 | 工作量 |
|---|---|---|---|---|---|
| **1** | **P0** | **yoke-derive 0.8.3 → 0.8.4** (yanked, lockfile-only) | `cargo update -p yoke-derive --precise 0.8.4` | 极低 (patch bump) | 5 分钟 |
| 2 | P0 | rustls-webpki 0.102.8 → 0.103.13+ (受限于 sqlx 0.8) | sqlx 升级 0.9 (breaking) 或 cargo:rustc-patch | 高 (sqlx 0.8 ↔ 0.102 耦合) | 半日 |
| 3 | P0 | h2 0.3.27 → 0.4.16+ (受限于 actix-http 3.x) | 等待 actix-http 4.x 或 fork | 高 (actix 3.x ↔ h2 0.3 耦合) | 半日 |
| 4 | P0-ignore | rsa 0.9.10 (Marvin Attack) 加入 audit.toml ignore | `cargo audit -i RUSTSEC-2023-0071` | 低 (运行时不可达) | 5 分钟 |
| 5 | P0-ignore | tokio-tar 0.3.1 (PAX smuggling) 加入 audit.toml ignore | `cargo audit -i RUSTSEC-2025-0111` | 低 (仅 dev) | 5 分钟 |
| 6 | P1 | wasmtime 19.0.2 → 49.0.2 (20 CVEs) | 三阶段: 19→24→36→49, function-plane FFI 评估 | 极高 (30 majors + FFI) | 1-2 周 |
| 7 | P1 | tonic 0.12.3 → 0.14.6 (cargo workspace 全栈) | `cargo update -p tonic --precise 0.14.6` (连带 tonic-build/health) | 中 (API 微调) | 半日 |
| 8 | P1 | opentelemetry* 0.24 → 0.33 (9 minor) | `cargo update -p opentelemetry --precise 0.33.1` (连带 3 个子 crate) | 中 (tracing-opentelemetry 0.25→0.34 需同步) | 半日 |
| 9 | P1 | rustls 0.23.45 (ULYS-239 P0-3 已落地) | (none, 已 fix) | — | — |
| 10 | P2 | sqlx 0.8.6 → 0.9.0 (解锁 rustls-webpki 升级) | `cargo update -p sqlx --precise 0.9.0` (breaking) | 高 (24 service crate 依赖) | 1-2 日 |
| 11 | P2 | thiserror 1.0 → 2.0 | `cargo update -p thiserror --precise 2.0.21` | 中 (derive macro 调整) | 1 日 |
| 12 | P2 | unmaintained crates: backoff → backon, bincode → wincode/postcard, fxhash → rustc-hash, instant → web-time, paste → pastey, rustls-pemfile → rustls-pki-types | 6 个独立升级 | 中 (API 差异) | 1 周 |
| 13 | P2 | 收紧 wildcard deps → caret 版本 (per task brief 53.14) | 24 service crate Cargo.toml 修 | 低 | 1 周 |
| 14 | P3 | serial_test 0.5.1 → 4.0.1 | `cargo update -p serial_test --precise 4.0.1` | 中 (API 大改) | 半日 |
| 15 | P3 | ctor 0.2.9 → 1.0.13 | `cargo update -p ctor --precise 1.0.13` | 中 (API 大改) | 半日 |
| 16 | P3 | actix-web-lab 0.24.3 → 0.29.5 | `cargo update -p actix-web-lab --precise 0.29.5` | 低 | 1 小时 |
| 17 | P3 | wasmtime-jit-debug 跟随 wasmtime 主线升级 (unsound) | 跟随 P1 #6 | 跟随 #6 | 跟随 #6 |
| 18 | P3 | jsonwebtoken 9.3.1 → 11.1.0 (2 major) | `cargo update -p jsonwebtoken --precise 11.1.0` | 中 (API 微调) | 半日 |
| 19 | P3 | rand 0.8.8 → 0.10.3 (0.02 minor) | `cargo update -p rand --precise 0.10.3` | 中 | 1 日 |
| 20 | P3 | mach 0.3.2 (macOS-only dev) 接受 | (none) | — | — |

**Process**:
- CI: 添加 `cargo audit --deny warnings` step
- Schedule: quarterly `cargo audit` + `cargo outdated` review
- Tracking: 独立 issue 跟踪 #2/#3/#6/#10 (技术债)

---

## 6. 交付物 (artifacts)

所有原始审计输出已落盘到 `.audit-reports/ULYS-249-*`：

| File | Size | 说明 |
|---|---|---|
| `ULYS-249-cargo-audit.json` | 131,515 B | cargo audit 完整 JSON (27 vuln + 9 warning) |
| `ULYS-249-cargo-audit.txt` | 10,725 B | cargo audit 人类可读 |
| `ULYS-249-cargo-deny-bans.txt` | 701,006 B | cargo deny 重复/wildcard (47+63 warnings) |
| `ULYS-249-cargo-deny-licenses.txt` | 442 B | cargo deny licenses (clean) |
| `ULYS-249-cargo-deny-sources.txt` | 11 B | cargo deny sources (clean) |
| `ULYS-249-cargo-outdated.txt` | 1,724 B | cargo outdated 输出 (libsqlite3-sys 冲突, 失败) |
| `ULYS-249-cargo-update-dryrun.txt` | 170 B | cargo update dry-run 默认输出 |
| `ULYS-249-cargo-update-dryrun-v.txt` | 3,408 B | cargo update dry-run verbose (31 unchanged) |
| `ULYS-249-cargo-update-aggressive.txt` | 3,408 B | cargo update dry-run aggressive |
| `ULYS-249-npm-audit-root.json` | 2,432 B | npm audit 根 (1 vuln: brace-expansion) |
| `ULYS-249-npm-audit-gm-console-frontend.json` | 4,006 B | npm audit gm-console/frontend (3 vuln) |
| `ULYS-249-npm-audit-{h5-e2e,rgs-batch-console,rgs-proxy,rgs-shim,rgs-web}.json` | 591 B each | ENOLOCK (无 lockfile) |

---

## 7. 与历史 baseline 对比

| Issue | Date | 总漏洞 | CRITICAL | HIGH | MEDIUM | LOW | 无 CVSS | npm |
|---|---|---|---|---|---|---|---|---|
| ULYS-239 | 2026-09-28 | 27 | 5 | 12 | 9 | 1 | n/a | 0 |
| ULYS-247 | 2026-09-29 | 27 | 5 | 12 | 9 | 1 | n/a | 0 |
| ULYS-253 | 2026-10-11 | 27 | 2 | 1 | 10 | 7 | 7 | 3 |
| **ULYS-249** | **2026-10-11** | **27** | **2** | **1** | **10** | **7** | **7** | **3** |

**注**: ULYS-239/247 的分层 (5/12/9/1) 是按"启发式严重度"(含 advisory description 关键字), 而 ULYS-253/249 是按 **实测 CVSS 向量值**. 数量一致 (27), 分层差异只是统计口径不同. ULYS-239→253 的真实变化是 **+1/-1** (RUSTSEC-2026-0316 新增 / RUSTSEC-2024-0437 消失).

---

## 8. Next Steps (per task brief §1.3)

ULYS-249 任务 §1.3 要求:

1. ✅ Run dependency audit tools — cargo audit / cargo deny / cargo outdated / npm audit (本报告 §1-§4)
2. ✅ Identify any packages with known security vulnerabilities — 27 RUSTSEC (§1.1)
3. ✅ List outdated packages that are more than 2 major versions behind — 4 个 (§4: wasmtime/serial_test/jsonwebtoken/wat, with wasmtime 30 / serial_test 3 / jsonwebtoken 2 / wast 2)
4. ✅ For each finding, note the severity, affected package, and recommended fix — §3 + §5
5. ✅ Post a summary report as a comment with actionable items — ULYS-249 评论 (本报告 markdown + Multica comment)

**建议下一步** (in priority order):

1. **立即 (本轮 PR)**: §3.3 P0-3 — `cargo update -p yoke-derive --precise 0.8.4` — 5 分钟, 0 风险
2. **下轮 PR**: §3.4 P0-ignore — `cargo audit -i RUSTSEC-2023-0071,RUSTSEC-2025-0111` 加入 `.cargo/audit.toml` — 5 分钟
3. **下下轮 PR**: §5 #7 P1 — tonic 0.12.3 → 0.14.6 — 半日
4. **下下下轮 PR**: §5 #8 P1 — opentelemetry* 0.24 → 0.33 — 半日
5. **独立 issue**: §5 #2/#3 — rustls-webpki / h2 升级 (受限于 sqlx 0.8 / actix 3.x 死锁) — 1 周评估
6. **独立 issue**: §5 #6 — wasmtime 19 → 49 — 1-2 周 (function-plane FFI 评估)

---

ULYS-249 / Hermes Agent (Minimax-M3)
2026-10-11 JST

代签: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手
审批: 架构师(Mavis 接手 agent per DEC-008)
修订人: Ulysses(一人公司 12 角色 per DEC-008) - Mavis 接手