# 依赖审计报告 (ULYS-253)

> **执行时间**: 2026-10-11 (Asia/Tokyo) | **工具链**: `npm audit` (3 manifests with deps) + `cargo audit 0.22.2` (RustSec advisory-db) + `cargo deny 0.20.2` + `cargo update --dry-run --verbose` | **项目**: RustGameServer 32-crate workspace
>
> **比较基线**: ULYS-239 (2026-09-28) → ULYS-247 (2026-09-29) → **ULYS-253 (本轮 2026-10-11)**
>
> **范围**: rust-toolchain 1.98.1 / Cargo.lock 全部 700+ packages / 7 个 package.json / deny.toml + audit.toml

---

## 1. 漏洞汇总

### 1.1 RustSec 漏洞 (cargo audit 0.22.2)

**总数**: 27 个漏洞 (与 ULYS-239 baseline 数量持平，净变化 +1/-1)

| Severity | 数量 | 说明 |
|---|---|---|
| **CRITICAL** (CVSS ≥ 9.0) | 2 | wasmtime 沙箱逃逸（Winch 编译器） |
| **HIGH** (CVSS 7.0–8.9) | 1 | wasmtime 文件系统 trailing-slash 沙箱逃逸 |
| **MEDIUM** (CVSS 4.0–6.9) | 10 | wasmtime (9) + rsa 0.9.10 (Marvin Attack) |
| **LOW** (CVSS < 4.0) | 7 | wasmtime (7) |
| **未标 CVSS** | 7 | h2, rustls-webpki (4), tokio-tar, wasmtime (Windows 设备文件名) |

**RustSec 基线漂移 (vs ULYS-239)**:
- 🆕 **新增 1**: `RUSTSEC-2026-0316` — wasmtime 19.0.2 "Dynamic record lifting can allocate beyond the hostcall fuel limit" (LOW 1.0, 2026-09-24 发布)
- ✅ **消失 1**: `RUSTSEC-2024-0437` — 之前的另一条 wasmtime 漏洞已被 RustSec 数据库修订/合并

### 1.2 按 crate 分组

| Crate | 当前版本 | 漏洞数 | 最高严重度 | 修复策略 |
|---|---|---|---|---|
| **wasmtime** | 19.0.2 | **20** | CRITICAL 9.0 | 升 49.0.2 (30 majors) — **P1，独立追踪** |
| **rustls-webpki** | 0.102.8 | **4** | HIGH | 升 0.103.13+ via async-nats 0.50 升级 — **P0** |
| **rsa** | 0.9.10 | **1** | MEDIUM 5.9 | 无修复版本（lockfile-only, 运行时不可达）— **P2 ignore** |
| **tokio-tar** | 0.3.1 | **1** | HIGH | 无修复版本（仅 dev/testcontainers）— **P2 ignore** |
| **h2** | 0.3.27 | **1** | UNKNOWN | 升 0.4.16+ — **P0**，需 actix-http 3.x 兼容性验证 |

### 1.3 npm 漏洞 (3 manifests with deps)

| Manifest | 漏洞 | 详情 |
|---|---|---|
| `package.json` (根) | 0 | `glob ^13.0.6` (构建期) — clean |
| `tools/gm-console/frontend/package.json` | **3** | `esbuild ≤0.24.2` (moderate) + `vite ≤6.4.2` (high, dev-tooling) + `source-map-js 1.0–1.2.1` (high, dev-tooling) |
| `tools/h5_e2e/package.json` | 0 | `ws ^8.21.3` — clean (无 lockfile，跳过 audit) |

**npm 结论**: 仅前端 dev 工具链有 3 漏洞，**全部为开发期依赖**，不影响生产运行时。修复需升 `vite ^5 → ^8`（breaking）+ `esbuild`/`source-map-js`（连带）。建议独立 issue 跟进。

### 1.4 cargo-audit 额外告警 (9 denied warnings)

| 类型 | Crate | 版本 | 处理 |
|---|---|---|---|
| unmaintained | `backoff` | 0.4.0 | 替换为 `backon` — P2 |
| unmaintained | `bincode` | 1.3.3 | 替换为 `wincode`/`postcard`/`bitcode` — P2 |
| unmaintained | `fxhash` | 0.2.1 | 替换为 `rustc-hash` — P2 |
| unmaintained | `instant` | 0.1.13 | 替换为 `web-time` — P2 |
| unmaintained | `mach` | 0.3.2 | macOS-only dev — 接受 |
| unmaintained | `paste` | 1.0.15 | 替换为 `pastey` — P2 |
| unmaintained | `rustls-pemfile` | 2.2.0 | 迁移到 `rustls-pki-types` (1.9+) — P2 |
| unsound | `wasmtime-jit-debug` | 19.0.2 | 跟随 wasmtime 主线升级 — P1 |
| yanked | `yoke-derive` | 0.8.3 | dry-run 显示 0.8.4 可用 — P0 patch bump |

---

## 2. cargo-deny 发现

### 2.1 许可 (licenses) — ✅ clean

- **0 errors, 2 warnings**: `OpenSSL` / `Unicode-DFS-2016` 在 allow 列表但未遇到 (informational)
- 已有 allow 列表 (Apache-2.0 / MIT / BSD-* / MPL-2.0 / 0BSD / CDLA-Permissive-2.0) 覆盖全部 lockfile

### 2.2 重复 crate (multiple-versions) — 47 warnings

**最严重** (按版本数量排序):

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

**P2 follow-up**: 收紧为 caret 版本（per task brief 53.14 注释）

### 2.4 来源 (sources) — ✅ clean

全部来源 crates.io，无未知 registry/git。

---

## 3. 过期版本 (cargo update --dry-run --verbose) — 31 个 unchanged

### 3.1 ≥2 major versions behind (**任务要求**)

| Crate | 当前 | 最新 | 落后 majors | 影响 | 优先级 |
|---|---|---|---|---|---|
| **wasmtime** | 19.0.2 | 49.0.2 | **30** ⚠️ | function-plane 唯一；消除全部 wasmtime 漏洞唯一路径 | **P1** |
| **serial_test** | 0.5.1 | 4.0.1 | **4** | dev-only (gm-backend 测试串行化) | **P2** |
| **jsonwebtoken** | 9.3.1 | 11.1.0 | **2** | gm-backend/admin-service JWT；v10 breaking (algorithm enum 化) | **P2** |
| **wast** | 259.0.0 | 261.0.0 | **2** | wasm 测试工具 (function-plane dev) | **P2** |

### 3.2 1 major behind (informational)

| Crate | 当前 | 最新 |
|---|---|---|
| `ctor` | 0.2.9 | 1.0.13 |
| `thiserror` | 1.0.69 | 2.0.21 |
| `async-nats` | 0.42.0 | 0.50.0 (8 majors, breaking) |
| `opentelemetry` / `-otlp` / `-sdk` / `-semantic-conventions` | 0.24/0.17/0.24/0.16 | 0.33.1 |
| `tracing-opentelemetry` | 0.25.0 | 0.34.0 |
| `tonic` / `-build` / `-health` | 0.12.3 | 0.14.6 |
| `prost` | 0.13.5 | 0.14.4 |
| `reqwest` | 0.12.28 | 0.13.5 |
| `sqlx` | 0.8.6 | 0.9.0 |
| `bcrypt` / `rand` / `rusqlite` / `rustler` / `rcgen` / `mockall` / `testcontainers` / `testcontainers-modules` / `tokio-tungstenite` / `sha2` / `criterion` / `base64` | various | 1 major each |

### 3.3 patch / minor 落后 (常规维护窗) — 44 个 updating + ~14 minor

---

## 4. 行动项汇总 (Action Items)

### P0 — 必修 (漏洞 / 安全直接相关)

| # | 行动 | 命令 / 说明 | 工作量 |
|---|---|---|---|
| **1** | **rustls-webpki 0.102.8 → 0.103.13+** (4 HIGH) | 需升级 `async-nats 0.42 → 0.50` (8 major breaking，Jetstream API 变更) — 独立 issue | 半日+ |
| **2** | **yoke-derive 0.8.3 → 0.8.4** (yanked) | `cargo update -p yoke-derive` — 一次性 patch bump | 5 分钟 |
| **3** | **h2 0.3.27 → 0.4.16+** (LOW DoS, 2026-09 发布) | `cargo update -p h2 --precise 0.4.16`；需验证 actix-http 3.x 兼容性 | 半日 |

### P1 — 建议 (独立 follow-up issue)

| # | 行动 | 说明 |
|---|---|---|
| **4** | **wasmtime 19 → 36+** | 消除 20 漏洞 (2 CRITICAL + 1 HIGH + 9 MEDIUM + 7 LOW + 1 Windows)；需先验证 Windows fuel-exhaustion FFI abort (RGS 19 pin 为规避此问题) 是否已修复 |
| **5** | **tonic 0.12 → 0.14** | gRPC 全栈 24 service；RGS-IMPL-003 工具链层任务 |
| **6** | **opentelemetry 0.24 → 0.33** | 53.12 OTel SDK 占位，54.13 正式接入 |
| **7** | **rustls 0.23.43 → 0.23.45** (ULYS-239 已识别) | patch bump |

### P2 — 跟踪

| # | 行动 | 说明 |
|---|---|---|
| **8** | rsa/tokio-tar ignore 决策 | lockfile-only / dev-only；可在 `audit.toml [advisories] ignore` 加 reason |
| **9** | serial_test 0.5 → 4.0 (4 majors) | dev-only |
| **10** | jsonwebtoken 9 → 11 (2 majors) | JWT breaking changes review |
| **11** | wast 259 → 261 (2 majors) | wasm 测试工具 |
| **12** | unmaintained crates 替换 | backoff→backon / bincode→postcard / fxhash→rustc-hash / instant→web-time / paste→pastey / rustls-pemfile→rustls-pki-types |
| **13** | 多版本并存优化 | 47 warnings；binary size / compile time |
| **14** | wildcard dep 收紧 | 63 warnings；批量把 `*` 改为 caret |

### P3 — 流程

| # | 行动 | 说明 |
|---|---|---|
| **15** | CI 加 `cargo deny check` | deny.toml 已就位；rust-ci.yml 缺 deny step (55.5 任务) |
| **16** | npm `vite ^5 → ^8` + esbuild + source-map-js | 3 dev-tooling vulns；breaking 改动，独立 issue |
| **17** | tools/h5_e2e 加 lockfile | 当前 `npm audit` 报 ENOLOCK |

---

## 5. 关键交付物 (本轮审计生成)

| 文件 | 大小 | 内容 |
|---|---|---|
| `.audit-reports/ULYS-253-cargo-audit.json` | 132 KB | cargo audit 完整 JSON |
| `.audit-reports/ULYS-253-cargo-audit.txt` | 14 KB | cargo audit --deny warnings 文本 (含 severity + solution) |
| `.audit-reports/ULYS-253-cargo-deny-bans.txt` | ~600 KB | cargo deny bans 输出 (47 duplicate + 63 wildcard warnings) |
| `.audit-reports/ULYS-253-cargo-deny-licenses.txt` | <1 KB | licenses clean |
| `.audit-reports/ULYS-253-cargo-deny-sources.txt` | <1 KB | sources clean |
| `.audit-reports/ULYS-253-cargo-update-dryrun.txt` | 2 KB | cargo update 摘要 |
| `.audit-reports/ULYS-253-cargo-update-dryrun-v.txt` | 4 KB | --verbose 完整清单 |
| `.audit-reports/ULYS-253-vulns-parsed.json` | 10 KB | 27 漏洞 + 8 warnings 结构化 (含 severity + solution) |
| `.audit-reports/ULYS-253-outdated-summary.json` | 1 KB | major-behind 汇总 |
| `.audit-reports/ULYS-253-npm-audit-frontend.json` | 1 KB | frontend npm audit JSON |
| `ULYS-253-audit-report.md` | 本文 | 人读版 |

---

## 6. 待决策事项 (需 D-Boy 确认)

1. **wasmtime 升级策略**: function-plane README 明确 19.x pin 为规避 Windows fuel-exhaustion FFI abort (`STATUS_STACK_BUFFER_OVERRUN`)；需先确认 wasmtime 20+ 是否已修复 (查 CHANGELOG/Issues)
2. **rustls-webpki 双版本并存**: lockfile 同时存在 0.102.8 (async-nats 路径) 和 0.103.15 (sqlx/rustls 路径)；本次 audit 触发的 4 HIGH 全部来自 0.102.8
3. **h2 0.3 → 0.4 与 actix-http 3.x 兼容性**: actix-http 3.13.5 当前依赖 h2 0.3；h2 0.4 是 tokio 拆分产物，actix-http 4.x 才支持
4. **rsa/tokio-tar ignore**: 需 D-Boy 确认是否加入 `audit.toml [advisories] ignore`
5. **前端 vite 升级窗口**: 升 `vite ^5 → ^8` 是 breaking，需评估前端 9 页 + SSE 影响

---

## 7. 历史趋势

| 审计轮次 | 日期 | 漏洞总数 | 备注 |
|---|---|---|---|
| ULYS-239 baseline | 2026-09-28 | 27 | 首次审计 |
| ULYS-247 | 2026-09-29 | 27 | after-fixes 验证 |
| **ULYS-253 (本轮)** | **2026-10-11** | **27** | 净 +1/-1 (RUSTSEC-2026-0316 新增 / RUSTSEC-2024-0437 消失) |

**结论**: 漏洞数持平说明 **过去 12 天无 RGS-side 安全修复落地**。P0 三项都是相对低成本修复（合计 < 1 工作日），建议立即排期。

---

*ULYS-253 / 执行: Hermes Agent (Minimax-M3) / 工作区: d83598db-3bfe-4386-b86d-ca266aa77e9f / 工具: cargo 1.98.1 + cargo-audit 0.22.2 + cargo-deny 0.20.2 + npm 11.19.0*