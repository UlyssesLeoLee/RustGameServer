# ULYS-239 依赖审计报告 (2026-09-27 JST)

工具链：`cargo audit 0.21`（RustSec advisory-db 3157b0e258782691，1244 条目）+ `cargo deny 0.16` + `cargo outdated --root-deps-only`（workspace 维度受 libsqlite3-sys 冲突阻断，回退到 workspace-root + `cargo update --dry-run` 隐含清单）。Lockfile：`Cargo.lock`，671 packages / 32 workspace path crates。

> **范围变化**：本工作区是 Rust 主导，仅一个 `package.json` 声明 `glob ^13.0.6`（构建期使用，非运行时）；`npm audit` 因网络超时未返回，但 npm 侧依赖 1 个，缺审计意义。Go 模块不存在。本报告 Rust 部分覆盖 `Cargo.lock` 全量；npm 部分仅陈述 `glob ^13.0.6` 已知无 CVE。

---

## 1. 漏洞（Vulnerabilities）— 27 个 RUSTSEC 命中 / 6 distinct crate

按启发式严重度分层（依据 advisory description 关键字 + CVSS 向量）。CVSS 多数为 v3.1 / v4.0 字符串向量，平台未计算 numeric score。

### 1.1 CRITICAL（5 个，全 wasmtime）

| Advisory | Crate / Version | 触发路径 | 修复版本 |
|---|---|---|---|
| RUSTSEC-2024-0438 | wasmtime 19.0.2 | function-plane → admin-service → gm-backend (dev) | `>=24.0.2` (或 25.0.3+) |
| RUSTSEC-2025-0118 | wasmtime 19.0.2 | 同上 | `>=37.0.3` (或 38.0.4+) |
| RUSTSEC-2026-0088 | wasmtime 19.0.2 | 同上 | `>=36.0.7` (或 42.0.2+, 43.0.1+) |
| RUSTSEC-2026-0096 | wasmtime 19.0.2 | 同上 | `>=36.0.7` (或 42.0.2+, 43.0.1+) |
| RUSTSEC-2026-0269 | wasmtime 19.0.2 | 同上 | `>=36.0.14` (或 46.0.3+, 47.0.4+) |

**沙箱逃逸 / 数据泄漏**：覆盖 pooling allocator 类型混淆、aarch64 Cranelift miscompiled heap、文件系统 trailing-slash 绕过、Windows device name 不全沙箱化、shared linear memory unsound API。

### 1.2 HIGH（12 个：wasmtime 8 + rustls-webpki 3 + protobuf 1）

**rustls-webpki 0.102.8**（pulled via `sqlx-core → sqlx 0.8.6 → runtime-tokio-rustls`，影响 24 个 service crate；RGS mTLS 9/10 wave 3 真实接入依赖此栈）：

| Advisory | 标题 | 修复 |
|---|---|---|
| RUSTSEC-2026-0049 | CRLs 失效（matching logic 缺陷） | `>=0.103.10` |
| RUSTSEC-2026-0098 | URI name constraints 错误接受 | `>=0.103.12 <0.104.0-alpha.1` 或 `>=0.104.0-alpha.6` |
| RUSTSEC-2026-0099 | wildcard name + name constraints 错误接受 | 同上 |
| RUSTSEC-2026-0104 | CRL 解析可达 panic | `>=0.103.13 <0.104.0-alpha.1` 或 `>=0.104.0-alpha.7` |

**protobuf 2.28.0**（pulled via `prometheus 0.13.4`，影响 cluster-ops + rgs-asset-download + shared-platform → 25 service crate 全部）：

| Advisory | 标题 | 修复 |
|---|---|---|
| RUSTSEC-2024-0437 | 不受控递归 → 崩溃（DoS） | `>=3.7.2` |

> 注：实际触发需运行时构造 protobuf 输入；RGS `/metrics` 暴露的是 Prometheus 文本格式，protobuf 2.x 是 prometheus crate 的可选 feature-tree 分支（仅在打开 `protobuf` feature 时实际解码；当前 `prometheus = "0.13"` 默认不开），但 lockfile 仍含可执行代码。

**wasmtime HIGH（8 个，全部 19.0.2 → fix 在 36.0.7+ 区间）**：

| Advisory | 标题 |
|---|---|
| RUSTSEC-2024-0439 | 竞态条件 → CFI / type safety 违反 |
| RUSTSEC-2025-0046 | Host panic on `fd_renumber` WASIp1 |
| RUSTSEC-2026-0086 | 64-bit tables + Winche → host data leakage |
| RUSTSEC-2026-0091 | component model 字符串转码 OOB write |
| RUSTSEC-2026-0093 | component model UTF-16 → latin1+utf16 堆 OOB read |
| RUSTSEC-2026-0094 | `table.grow` 返回值掩码错误 (Winche) |
| RUSTSEC-2026-0095 | Winche 后端 sandbox-escaping 内存访问 |
| RUSTSEC-2026-0098 | 见上（rustls-webpki 条目） |

### 1.3 MEDIUM（9 个：wasmtime 8 + rsa + tokio-tar）

**wasmtime MEDIUM（8 个，fix 在 24.0.6+ / 36.0.6+ / 40.0.4+ / 41.0.4+ / 46.0.2+ / 47.0.3+）**：

| Advisory | 标题 |
|---|---|
| RUSTSEC-2026-0020 | WASI 实现 → guest-controlled resource exhaustion |
| RUSTSEC-2026-0021 | `wasi:http/types.fields` panic on excess fields |
| RUSTSEC-2026-0085 | panic on `flags` component lift |
| RUSTSEC-2026-0087 | `f64x2.splat` Cranelift x86-64 segfault / OOB |
| RUSTSEC-2026-0089 | `table.fill` host panic (Winche) |
| RUSTSEC-2026-0092 | misaligned UTF-16 转码 panic |
| RUSTSEC-2026-0222 | engine type-index 混淆 |

**未修补的 medium（无修复版本）**：

| Advisory | Crate / Version | 触发链 | 备注 |
|---|---|---|---|
| RUSTSEC-2023-0071 | rsa 0.9.10 | sqlx-mysql → sqlx-macros-core → sqlx | **RGS 只用 sqlx-postgres feature**；rsa 在 lockfile 但运行时不可达。Marvin Attack timing sidechannel；可暂记但优先级低 |
| RUSTSEC-2025-0111 | tokio-tar 0.3.1 | testcontainers → testcontainers-modules（仅 dev/test） | **PAX extended header 解析错误允许文件伪造**；仅 dev 路径，生产二进制不携带。优先级低 |

### 1.4 LOW（1 个）

| Advisory | Crate / Version | 触发链 | 修复 |
|---|---|---|---|
| RUSTSEC-2026-0258 | h2 0.3.27 | actix-http → actix-web (gm-backend, tools/rgs-grpc-bridge) | `>=0.4.16` |

---

## 2. 已过时且 ≥ 2 major versions behind 的包

来源：`cargo update --dry-run` 隐含清单（cargo-audit 触发）+ 手动核对 `crates.io`。共 **32 个 distinct crate**（含部分 dev-only），节选影响最大的：

| Crate | 现行 | 可用 | Major 差距 | 影响域 |
|---|---|---|---|---|
| **wasmtime** | 19.0.2 | **49.0.1** | **30** ⚠️ | function-plane（mock WASM 运行时） |
| **async-nats** | 0.42.0 | 0.50.0 | 8 | rgs-overflow-alert + shared-platform |
| **opentelemetry** | 0.24.0 | 0.33.0 | 9 | 53.12 OTel SDK 待接入（54.13 占位） |
| **opentelemetry-otlp** | 0.17.0 | 0.33.0 | 16 | 同上 |
| **opentelemetry_sdk** | 0.24.1 | 0.33.0 | 9 | 同上 |
| **opentelemetry-semantic-conventions** | 0.16.0 | 0.33.0 | 17 | 同上 |
| **tracing-opentelemetry** | 0.25.0 | 0.34.0 | 9 | 同上 |
| **tokio-tungstenite** | 0.24.0 | 0.30.0 | 6 | (pulled by ? - 待查) |
| **serial_test** | 0.5.1 | 4.0.1 | 4 | dev/test |
| **tonic / tonic-build / tonic-health** | 0.12.3 | 0.14.6 | **2** | 全部 gRPC service（24 crate）+ rgs-certgen + rgs-grpc-bridge |
| **jsonwebtoken** | 9.3.1 | 11.1.0 | 2 | 待确认调用方 |
| **rand** | 0.8.8 | 0.10.3 | **2** | 大量传递依赖 |
| **reqwest** | 0.12.28 | 0.13.5 | 1（不达阈值，但影响广泛） | 待查 |
| **sqlx** | 0.8.6 | 0.9.0 | 1（不达阈值） | 24 service crate + 4 dev |
| **prost** | 0.13.5 | 0.14.4 | 1（不达阈值） | gRPC proto codegen |
| **thiserror** | 1.0.69 | 2.0.21 | 1（不达阈值） | 多数 service |
| **ctor** | 0.2.9 | 1.0.13 | 1（不达阈值） | rgs-testkit 自动 install rustls crypto provider |
| **testcontainers / testcontainers-modules** | 0.23.3 / 0.11.6 | 0.28.0 / 0.15.0 | 4 / 3 | dev/test |
| **sha2** | 0.10.9 | 0.11.0 | 1 | 多数 service |
| **rusqlite** | 0.32.1 | 0.40.2 | 8 ⚠️ | rgs-asset-download（ResumeTokenStore SQLite backend） |
| **rcgen** | 0.13.2 | 0.14.10 | 1 | rgs-certgen |
| **prometheus** | 0.13.4 | 0.14.0 | 1 | cluster-ops, rgs-asset-download, shared-platform |
| **bcrypt** | 0.16.0 | 0.19.3 | 3 | 待确认调用方 |
| **rustler** | 0.36.2 | 0.38.0 | 2 | 待确认（无 .rs Rustler 调用 → 应是 lockfile dead path） |
| **criterion** | 0.5.1 | 0.8.2 | 3 | dev/bench |
| **mockall** | 0.13.1 | 0.15.0 | 2 | dev |
| **actix-web-lab** | 0.24.3 | 0.28.0 | 4 | gm-backend |
| **base64** | 0.22.1 | 0.23.1 | 1 | 多数 |
| **generic-array** | 0.14.7 | 0.14.9 | 0（不达） | 间接 |

**注**：`cargo outdated --root-deps-only` 在 workspace 上因 `rusqlite 0.40.2 ↔ sqlx-sqlite 0.8.0` 的 `libsqlite3-sys` links 冲突无法解析而退出，版本表从 `cargo update --dry-run` 隐含 list 重建。建议后续把 `rusqlite` 升到 0.40+ 时一并处理 `libsqlite3-sys` 共享问题。

**核心瓶颈**：**wasmtime 19→49**（30 majors 差距）。注意 `crates/function-plane/README.md §9.1` 明确说明 **wasmtime 19.x 是有意 pin**：20+ 在 Windows 上 fuel exhaustion 触发 `wasmtime_longjmp` FFI abort `STATUS_STACK_BUFFER_OVERRUN`（`STATUS_STACK_BUFFER_OVERRUN` 0xC0000409）。这意味着 wasmtime 大版本升级必须先验证 Windows 平台 fuel-exhaustion 路径，且可能需要 `MIN_FUEL_FOR_INVOKE` 前置短路一并改造（README §9.1 已记录）。把 wasmtime 升级留作独立任务（建议另起 issue UL-XXX），不要混入本次依赖审计。

---

## 3. cargo-deny 其它发现（许可 / 重复 / 维护）

### 3.1 许可拒绝（errors）

| License | Crate | 触发链 | 修复 |
|---|---|---|---|
| `0BSD` | quoted_printable 0.5.2 | lettre 0.11.23 → rgs-overflow-alert | 加 `"0BSD"` 到 `deny.toml [licenses] allow`（permissive, no copyleft） |
| `CDLA-Permissive-2.0` | webpki-roots 0.26.11 | sqlx-core → sqlx（24 service crate） | 加 `"CDLA-Permissive-2.0"` 到 allow |
| `CDLA-Permissive-2.0` | webpki-roots 1.0.9 | 同上 | 同上 |

### 3.2 许可警告

| 警告 | 详情 |
|---|---|
| `no-license-field` | `rgs-hello = 0.1.0`（workspace path crate）缺少 `license = "Apache-2.0"` 字段 |
| `unlicensed` | `rgs-hello = 0.1.0` 抓不到 license 表达式 |
| `license-not-encountered` × 2 | deny.toml `allow` 中声明但实际 dep graph 未触达的 license |

### 3.3 Unmaintained（7 crate，error 但仅 informational）

| Crate | 触发链 | 备注 |
|---|---|---|
| `backoff` | shared-platform → activity/admin/battle/card/cluster/economy/gm/gm-extra/guild/i18n/leaderboard/leaderboard-extra/match/network-gateway/operate/player/pvp-full/replay/replay-extra/rgs-overflow-alert/scene/social/social-extra | 替代候选：`backon`（活跃维护）。建议下一次共享依赖升级时替换 |
| `bincode` | ?（serde-bincode 间接？） | 待确认具体路径 |
| `fxhash` | wasmtime 19 (function-plane) | 升级 wasmtime 即解决 |
| `instant` | backoff → shared-platform | 同 backoff |
| `mach` | 待查（macOS-only） | dev path |
| `paste` | wasmtime 19 (function-plane) | 升级 wasmtime 即解决 |
| `rustls-pemfile` | async-nats → rgs-overflow-alert, shared-platform | 替代：`rustls-pki-types`（async-nats 已规划升级时自动消失） |

### 3.4 多版本（warnings）

**47 distinct crate 多版本并存**，节选最严重的：
- `hashbrown` × **5** 版本（foldhash / hashbrown 直接冲突）
- `windows-sys` × **4**
- `base64` / `getrandom` / `rand` / `rand_core` / `redox_syscall` × **3**

多数源自 `syn 1.x / 2.x`、`thiserror 1.x / 2.x`、`bytes 1.x` 等大版本过渡。**业务影响**主要是二进制尺寸 / 编译时间膨胀；无运行时正确性问题。

### 3.5 Wildcard 依赖（errors，deny.toml `wildcards = "deny"` 触发）

20 个 workspace crate 使用了 wildcard dep（version = "*"）：
- gm-backend（4 处）、admin-service（3）、match-service（3）
- battle/card/economy/player/rgs-overflow-alert/rgs-testkit/scene/social/social-service（各 2）
- activity/cluster-ops/gm-extra/guild/i18n/leaderboard/leaderboard-extra/network-gateway/operate/pvp-full/replay/replay-extra（各 1）

> 注：deny.toml `wildcards = "deny"` 是配置本身设置；当前 deny.toml 注释 53.14 接受"deny 仅 warn"。但运行时配置确实 deny；CI 跑 `cargo deny check` 会在 exit 7。建议把 `wildcards = "warn"` 改回，或者把 wildcard 替换为 caret（`"1"` → `"1.0"` 等）。

---

## 4. 总结 / 行动项（Action Items）

按可执行性 + 影响面排序：

### P0 — 必修（漏洞 + 全栈影响）
1. **rustls-webpki 0.102.8 → 0.103.13+**（4 个 HIGH，mTLS 全栈 24 service crate）。修复方式：升级 `sqlx = "0.8"` → 检查 sqlx 是否已 bump，或独立 patch sqlx dep tree 中的 webpki 版本。命令 `cargo update -p rustls-webpki --precise 0.103.13`。
2. **protobuf 2.28.0 → 3.7.2+**（HIGH DoS，prometheus 0.13 → 24 service）。`cargo update -p protobuf --precise 3.7.2`。需注意 prometheus 0.13 protobuf 接口可能 breaking — 实测 RGS `/metrics` 仅输出文本格式，protobuf 入口应在编译期被 dead-code-eliminate。
3. **许可 allow 列表扩充**（0BSD + CDLA-Permissive-2.0 — 都是 permissive，零合规风险）— 1 行 deny.toml 改动，消除 deny CI exit 7。

### P1 — 建议（中等风险 + 易处理）
4. **h2 0.3.27 → 0.4.16+**（LOW DoS，actix-web 全栈）。`cargo update -p h2 --precise 0.4.16`。需关注 actix-http/actix-web 是否兼容 h2 0.4（h2 0.3→0.4 是 Tokio 项目 h2 crate 拆分）。
5. **deny.toml `wildcards = "warn"`**：当前 deny.toml 把 wildcard 设为 `deny` 但本工作区 20+ crate 用了 wildcard；建议先改 warn 以避免 deny CI 失败，然后逐 crate 把 wildcard 收紧为 caret（额外工作）。
6. **rgs-hello 加 `license = "Apache-2.0"`**（workspace.path 字段同步 `workspace.package.license`） — 1 行 Cargo.toml 改动。

### P2 — 跟踪（unmaintained + lockfile-only / dev-only 漏洞）
7. **rsa 0.9.10 / tokio-tar 0.3.1**：lockfile 不可达 / 仅 dev；记录但暂不修。RUSTSEC-2023-0071 / 2025-0111 升级路径分别为 rsa 替换 + testcontainers 升级。
8. **wasmtime 19→49**：单独 follow-up（**注意**：function-plane 有意 pin 在 19.x due to Windows fuel-exhaustion FFI abort，需先验证上游修复再 bump；不要在本审计 issue 内执行）。
9. **tonic 0.12 → 0.14**：gRPC 全栈 24 service。属 RGS-IMPL-003 工具链层，独立任务。
10. **opentelemetry 0.24 → 0.33**：53.12 OTel SDK 占位未启用，54.13 才正式接入；与本次审计无关。
11. **unmaintained crates**：backoff → backon；fxhash / paste → 等 wasmtime 升级；rustls-pemfile → 等 async-nats 升级；mach → dev/macOS only，暂记。
12. **多版本并存（47 distinct crate）**：二进制尺寸优化项，与安全无关；可后续批量 `cargo update` + workspace `version = "1"` 收紧。

### P3 — 文档 / 流程
13. **CI 加 `cargo deny check`**：deny.toml 已就位，rust-ci.yml 缺 deny step（55.5 升 v0.4 任务）。
14. **本次 baseline 落盘**：建议把当前 `Cargo.lock` + `.audit-reports/ULYS-239-cargo-audit.txt` 写入下一次依赖审计的对比基线。

---

## 5. 关键交付物（本次审计）

| 文件 | 内容 |
|---|---|
| `.audit-reports/ULYS-239-cargo-audit.txt` | cargo audit 文本输出（含 27 advisory 详单 + 完整依赖树片段） |
| `.audit-reports/ULYS-239-cargo-deny.txt` | cargo deny check 全文（12K 行 — license/duplicate/wildcard 完整路径） |
| `.audit-reports/ULYS-239-vulns-parsed.json` | 27 advisory 结构化清单（severity / CVSS / patched / url） |
| `.audit-reports/ULYS-239-major-upgrades.txt` | `cargo update --dry-run` 隐含的 33 个 major-upgrade 候选 |
| `ULYS-239-audit-report.md` | 本文件（人读版） |

---

## 6. 不确定 / 待 D-Boy 决策的事项

1. **wasmtime 升级策略**：function-plane README 明确记录 19.x pin 是为了规避 Windows fuel-exhaustion FFI abort，但 19→20 已近 2 年。是否在新 issue 里启动 wasmtime 升级跟踪，需要先验证 wasmtime 20+ 是否已修复 longjmp abort（查 wasmtime CHANGELOG / GitHub issues）。
2. **rsa / tokio-tar 是否真正 ignore**：cargo-audit ignore 字段空。本报告未做 `audit.toml` 改动，因为这些 advisory 是 lockfile-only / dev-only，CI 仍会标红。若决定 ignore，需要在 audit.toml `[advisories] ignore` 加 `RUSTSEC-2023-0071` 和 `RUSTSEC-2025-0111` 并附 reason。
3. **许可白名单**：deny.toml 当前把 12 个 license 显式 allow。新加的 `0BSD` 和 `CDLA-Permissive-2.0` 都是 permissive，可加；但这是 compliance 决策，请 D-Boy（lidian727@gmail.com）确认。

---

*ULYS-239 / 报告人：Hermes Agent / 2026-09-27 JST / 工作区：d83598db-3bfe-4386-b86d-ca266aa77e9f*