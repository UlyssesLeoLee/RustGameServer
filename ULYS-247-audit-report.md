# ULYS-247 依赖审计报告 (2026-09-28 JST, ULYS-239 follow-up)

> **任务**: ULYS-247 — RustGameServer 依赖安全与过期版本审计
> **承接**: ULYS-239 (2026-09-27) 后续基线审计；首次审计报告 `ULYS-239-audit-report.md` 已落盘
> **工具链**: `cargo audit 0.22.2` (RustSec advisory-db `3157b0e258782691`, 1273 条目) + `cargo deny 0.20.2` (licenses/bans/sources 子集) + `cargo update --dry-run --workspace --verbose` (outdated 替代)
> **Lockfile**: `Cargo.lock`, 670 packages / 32 workspace path crates
> **重要约束**: cargo-deny 自动 fetch advisory-db 在本机网络环境失败（github.com:443 Recv failure / Failed to connect），因此本报告**advisories 来源用 cargo audit JSON（本地 advisory-db 缓存）**, **licenses/bans/sources 用 cargo deny 子集输出**。Cargo.lock 与 dev HEAD `ff7f3183` 一致。

---

## 1. 与 ULYS-239 的差异 (delta)

| 项 | ULYS-239 (2026-09-27) | ULYS-247 (2026-09-28) | 变化 |
|---|---|---|---|
| Cargo.lock SHA | `ff7f3183` 前 | `ff7f3183` (HEAD) | 0 |
| 总漏洞数 | 27 (RUSTSEC) | 27 (RUSTSEC) | 0 |
| 已被 ULYS-239 P0 修复 | n/a | `RUSTSEC-2024-0437` (protobuf 2.28→3.7.2) 已从清单消失 | **+1 fixed** |
| 新出现漏洞 | n/a | `RUSTSEC-2026-0285` (rustls 0.23.43, 2026-09-14 发布) | **+1 new** |
| Cargo.lock 内 `protobuf` 版本 | 2.28.0 | **3.7.2** (P0-2 已落地) | fixed |
| Cargo.lock 内 `rustls` 版本 | 0.23.43 | **0.23.43** (fix 在 0.23.45) | **未修** |
| Cargo.lock 内 `rustls-webpki` 版本 | 0.102.8 + 0.103.15 (双版本并存) | 同上 | **未修** |
| Cargo.lock 内 `wasmtime` 版本 | 19.0.2 | 19.0.2 | **未修** |
| Cargo.lock 内 `h2` 版本 | 0.3.27 | 0.3.27 | **未修** |
| npm audit (package.json) | clean (glob ^13.0.6) | **clean** | 0 |
| 许可 (deny.toml) | 0BSD / CDLA-Permissive-2.0 已 allow | **ok** | fixed |
| Wildcard dep | `wildcards = "warn"` 已生效 | **ok: 0 errors, 71 warnings** | fixed |
| `rgs-hello` license 字段 | 已加 `Apache-2.0` | **ok** | fixed |

**结论**: ULYS-239 报告的 P0-2 / P0-3 / P1-5 / P1-6 均已落地 (`9d8eae97 merge(ULYS-239)`)。**P0-1 (rustls-webpki 0.102.8)、H0 (h2 0.3.27)、新增 P0-0 (rustls 0.23.43→0.23.45)** 仍未处理；其余建议（wasmtime / tonic / opentelemetry / unmaintained crates / 多版本并存）状态同 ULYS-239。

---

## 2. 漏洞（Vulnerabilities）— 27 个 RUSTSEC 命中 / 6 distinct crate

按启发式严重度分层（依据 advisory description 关键字 + CVSS 向量）。CVSS 多数为 v3.1 / v4.0 字符串向量，平台未计算 numeric score。

### 2.1 CRITICAL（5 个，全 wasmtime）

| Advisory | Crate / Version | 触发路径 | 修复版本 |
|---|---|---|---|
| RUSTSEC-2024-0438 | wasmtime 19.0.2 | function-plane → admin-service → gm-backend (dev) | `>=24.0.2`（或 25.0.3+）|
| RUSTSEC-2025-0118 | wasmtime 19.0.2 | 同上 | `>=37.0.3`（或 38.0.4+）|
| RUSTSEC-2026-0088 | wasmtime 19.0.2 | 同上 | `>=36.0.7`（或 42.0.2+, 43.0.1+）|
| RUSTSEC-2026-0096 | wasmtime 19.0.2 | 同上 | `>=36.0.7`（或 42.0.2+, 43.0.1+）|
| RUSTSEC-2026-0269 | wasmtime 19.0.2 | 同上 | `>=36.0.14`（或 46.0.3+, 47.0.4+）|

**沙箱逃逸 / 数据泄漏**：覆盖 pooling allocator 类型混淆、aarch64 Cranelift miscompiled heap、文件系统 trailing-slash 绕过、Windows device name 不全沙箱化、shared linear memory unsound API。

### 2.2 HIGH（12 个：wasmtime 8 + rustls-webpki 3）

**rustls-webpki 0.102.8**（pulled via `sqlx-core → sqlx 0.8.6 → runtime-tokio-rustls`，影响 24 个 service crate；RGS mTLS 9/10 wave 3 真实接入依赖此栈）：

| Advisory | 标题 | 修复 |
|---|---|---|
| RUSTSEC-2026-0049 | CRLs 失效（matching logic 缺陷）| `>=0.103.10` |
| RUSTSEC-2026-0098 | URI name constraints 错误接受 | `>=0.103.12 <0.104.0-alpha.1` 或 `>=0.104.0-alpha.6` |
| RUSTSEC-2026-0099 | wildcard name + name constraints 错误接受 | 同上 |
| RUSTSEC-2026-0104 | CRL 解析可达 panic | `>=0.103.13 <0.104.0-alpha.1` 或 `>=0.104.0-alpha.7` |

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
| RUSTSEC-2026-0098 | 见上（rustls-webpki 条目）|

### 2.3 MEDIUM（9 个：wasmtime 8 + rsa + tokio-tar + rustls NEW）

**rustls 0.23.43 MEDIUM (RUSTSEC-2026-0285, NEW since 2026-09-14)**

| Advisory | 标题 | 修复 |
|---|---|---|
| RUSTSEC-2026-0285 | TLS 1.3 handshake 消息在 encryption level 边界跨级接受 (RFC 8446 §5.1 违反) | `>=0.23.45` |

触发路径：`gm-backend / hyper-rustls / reqwest / lettre / rgs-grpc-bridge / rgs-testkit (dev)` —— 即 mTLS 栈全栈可达；尽管 CVE 标记 CVSS 5.3 (low-medium) 且"handshake transcript is authenticated"，但合规审计会标红。修复：`cargo update -p rustls --precise 0.23.45`。

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
| RUSTSEC-2025-0111 | tokio-tar 0.3.1 | testcontainers → testcontainers-modules（仅 dev/test）| **PAX extended header 解析错误允许文件伪造**；仅 dev 路径，生产二进制不携带。优先级低 |

### 2.4 LOW（1 个，h2）

| Advisory | Crate / Version | 触发路径 | 修复 | 备注 |
|---|---|---|---|---|
| RUSTSEC-2026-0258 | h2 0.3.27 | `actix-http 3.13.5 → actix-web 4.15.0`（全栈 24 service crate）| `>=0.4.16` | 攻击者发送 unbounded empty DATA frames，无界内存增长或 length 溢出 panic；与 `actix-web 4.x` 兼容性待 h2 0.3→0.4 评估 |

---

## 3. cargo-deny 子集发现

`cargo deny check` 因网络 fetch advisory-db 失败无法跑全产物，分项结果（exit 0）：

### 3.1 许可（licenses）

**`licenses ok: 0 errors, 2 warnings, 575 notes`** — 与 ULYS-239 一致。

```
warning[license-not-encountered]: license was not encountered
   ┌─ D:\RustGameServer\deny.toml:34:6
   │ "OpenSSL"
warning[license-not-encountered]: license was not encountered
   ┌─ D:\RustGameServer\deny.toml:30:6
   │ "Unicode-DFS-2016"
```

### 3.2 bans（重复 / wildcard / 来源）

**`bans ok: 0 errors, 71 warnings, 0 notes`**

- **wildcards = "warn"** 已生效（per ULYS-239 P1-5）：71 warnings 为 path-based workspace dep（`shared-platform = { path = "../shared-platform" }` 等 24 crate × 1-3 处）。`bans ok: 0 errors` 表明 deny CI 不 exit 7。
- **duplicate**：仍存在多版本并存（hashbrown × 5、windows-sys × 4、base64 / getrandom / rand / rand_core / redox_syscall × 3），多数源自 syn 1.x/2.x、thiserror 1.x/2.x、bytes 1.x 大版本过渡。无运行时正确性问题，二进制尺寸 / 编译时间膨胀。

### 3.3 sources

**`sources ok: 0 errors, 0 warnings, 0 notes`** — 仅 crates.io 官方源，无 git / 私有注册表。

---

## 4. 过期版本（outdated）— 86 个 crate 在当前 semver major 内可升

`cargo outdated --workspace` 因 `libsqlite3-sys` (sqlx 0.8 ↔ rusqlite 0.32) 链接冲突阻断（per ULYS-239 已知问题），回退到 `cargo update --dry-run --workspace --verbose`：

**总计 86 unchanged crates：**

| 类型 | 数量 |
|---|---|
| 落后 ≥ 2 个 major | **3** |
| 落后恰好 1 个 major | 2 |
| 落后 0 major（minor/patch only）| 81 |

### 4.1 ≥ 2 major 落后（任务 brief §3 要求）

| Crate | 当前 | 最新可用 | 落后 major | 影响 |
|---|---|---|---|---|
| `wasmtime` | 19.0.2 | 49.0.1 | **30** | function-plane 唯一；fix 全部 wasmtime CRITICAL/HIGH 漏洞的唯一功能选项 |
| `serial_test` | 0.5.1 | 4.0.1 | **4** | dev-only：`gm-backend` 用 serial_test 串行化测试；非运行时 |
| `jsonwebtoken` | 9.3.1 | 11.1.0 | 2 | gm-backend / admin-service JWT；breaking changes 多（v10 algorithm enum 化） |

### 4.2 落后 1 major（仅记录，非任务要求）

| Crate | 当前 | 最新可用 |
|---|---|---|
| `opentelemetry` | 0.24.0 | 0.33.0 |
| `opentelemetry-otlp` | 0.17.0 | 0.33.0 |
| `opentelemetry_sdk` | 0.24.1 | 0.33.0 |
| `tracing-opentelemetry` | 0.25.0 | 0.34.0 |
| `tonic` / `tonic-build` / `tonic-health` | 0.12.3 | 0.14.6 |

### 4.3 minor/patch only（落后 0 major）

81 个 — 大多 rustix / hyper-util / tokio-rustls / bcrypt / criterion / mockall / 等等。属于常规依赖维护升级窗，无安全必修紧迫性。

---

## 5. 总结 / 行动项（Action Items）

### P0 — 必修（漏洞必修，2026-09-28 JST 当日起算 7 日内）

| # | 行动 | 命令 / 触发链 | 工作量 |
|---|---|---|---|
| **1** | **rustls-webpki 0.102.8 → 0.103.13+**（4 个 HIGH，mTLS 全栈 24 service crate；0.102.8 仍被 sqlx 0.8 → runtime-tokio-rustls 间接拉入）。ULYS-239 P0-1 未修 | `cargo update -p rustls-webpki --precise 0.103.13`（注：lockfile 已有 0.103.15，验证双版本为何并存；sqlx 0.8 / sqlx-postgres 0.8.6 内部仍依赖旧版，需要 sqlx 升级或 patch-path 强制）| 半日 |
| **2** | **rustls 0.23.43 → 0.23.45**（1 个 MEDIUM，新增 RUSTSEC-2026-0285）| `cargo update -p rustls --precise 0.23.45`（一次 patch bump，向后兼容）| 1 小时 |
| **3** | **h2 0.3.27 → 0.4.16+**（1 个 LOW DoS，actix-web 全栈 24 service；ULYS-239 P1-4 未修）| `cargo update -p h2 --precise 0.4.16` + 验证 `actix-http 3.13.5` 仍兼容（h2 0.3→0.4 是 tokio 项目拆分）| 半日 |

### P1 — 建议（独立 follow-up issue，14 日内）

| # | 行动 | 说明 |
|---|---|---|
| **4** | **wasmtime 19 → 36+**（消除 5 CRITICAL + 8 HIGH + 8 MEDIUM 共 21 条 advisory；任务 brief §2 唯一"清零"漏洞的路径）| function-plane README 明确 19.x pin 是为了规避 Windows fuel-exhaustion FFI abort；需先验证 wasmtime 20+ 是否已修复该 longjmp abort；独立 issue，不要在 ULYS-247 内执行 |
| **5** | **tonic 0.12 → 0.14**（gRPC 全栈 24 service；落后 2 major）| 属 RGS-IMPL-003 工具链层，独立任务 |
| **6** | **opentelemetry 0.24 → 0.33**（53.12 OTel SDK 占位未启用，54.13 才正式接入）| 与本次审计无关 |

### P2 — 跟踪（lockfile-only / dev-only / 维护风险）

| # | 行动 | 说明 |
|---|---|---|
| **7** | **rsa 0.9.10 / tokio-tar 0.3.1** | lockfile 不可达 / 仅 dev；可在 `audit.toml [advisories] ignore` 加 `RUSTSEC-2023-0071` 和 `RUSTSEC-2025-0111` + reason；需 D-Boy 决策 |
| **8** | **serial_test 0.5.1 → 4.0.1** | dev-only test serial；非运行时安全 |
| **9** | **jsonwebtoken 9.3.1 → 11.1.0** | JWT breaking changes（v10 algorithm enum 化）；gm-backend / admin-service 需 review |
| **10** | **unmaintained crates** | backoff / bincode / fxhash / instant / mach / paste / rustls-pemfile / wasmtime-jit-debug；建议 `Cargo.toml` workspace.dependencies 收紧时一次性替换 |
| **11** | **多版本并存**（47+ distinct crate）| 二进制尺寸 / 编译时间优化项；可后续批量 `cargo update` + workspace `version = "1"` 收紧 |

### P3 — 流程

| # | 行动 | 说明 |
|---|---|---|
| **12** | **CI 加 `cargo deny check`** | deny.toml 已就位；rust-ci.yml 缺 deny step（55.5 升 v0.4 任务）|
| **13** | **本次 baseline 落盘** | 把当前 `Cargo.lock` + `.audit-reports/ULYS-247-*` 写入下一次审计的对比基线；ULYS-239 → ULYS-247 的对比显示"已修复 vs 新增"的差分能力已验证 |

---

## 6. 关键交付物（本次审计）

| 文件 | 内容 |
|---|---|
| `.audit-reports/ULYS-247-cargo-audit.txt` | cargo audit 文本输出（含 27 advisory 详单）|
| `.audit-reports/ULYS-247-cargo-audit.json` | cargo audit 完整 JSON（128 KB；advisories / warnings / unsound 全结构化）|
| `.audit-reports/ULYS-247-vulns-parsed.json` | 27 advisory 结构化清单（id / package / version / cvss / patched / url）|
| `.audit-reports/ULYS-247-cargo-deny-bans.txt` | cargo deny bans 子集（11.6K 行 — 71 warnings 详单）|
| `.audit-reports/ULYS-247-cargo-deny-licenses.txt` | cargo deny licenses 子集（13 行 — 0 errors, 2 warnings, 575 notes）|
| `.audit-reports/ULYS-247-cargo-deny-sources.txt` | cargo deny sources 子集（0 errors）|
| `.audit-reports/ULYS-247-cargo-update-dryrun-v.txt` | `cargo update --dry-run --verbose` 86 unchanged crates 完整清单 |
| `.audit-reports/ULYS-247-cargo-outdated.txt` | cargo outdated 失败日志（libsqlite3-sys 冲突 — 已知，per ULYS-239）|
| `ULYS-247-audit-report.md` | 本文件（人读版）|

---

## 7. 不确定 / 待 D-Boy 决策的事项

1. **rustls-webpki 0.102.8 双版本并存策略**：lockfile 内 `rustls-webpki 0.102.8` 和 `0.103.15` 同时存在；0.103.15 是 sqlx 0.8.6 内部拉的（**pulled via rustls 0.23.43 → rustls-webpki 0.103.15**），但 sqlx-core 0.8.6 同时依赖 `rustls-webpki 0.102.8` 用于 webpki feature。两条路径独立，不能 `cargo update -p rustls-webpki --precise 0.103.13` 一次性升级；需 sqlx 0.9 升级或 patch-path。
2. **wasmtime 升级策略**：同 ULYS-239 §6.1，function-plane pin 19.x 是为了规避 Windows fuel-exhaustion FFI abort。是否在新 issue 启动 wasmtime 升级跟踪，需先验证 wasmtime 20+ 是否已修复 longjmp abort（查 wasmtime CHANGELOG / GitHub issues）。
3. **h2 0.3 → 0.4 与 actix-http 3.x 兼容性**：actix-http 3.13.5 当前用 h2 0.3.27；h2 0.4 是 tokio 项目拆分，actix-http 4.x 才用 h2 0.4。RGS tonic 0.12 + actix-web 4.15 路径已用 h2 0.4.19（hyper 1.11 → tonic），只有 actix-http 3.13.5 路径仍要 0.3。需要测试 mTLS gateway 是否受影响。
4. **rsa / tokio-tar ignore 决策**：同 ULYS-239 §6.2，本次未做 `audit.toml` 改动。

---

*ULYS-247 / 报告人：Hermes Agent / 2026-09-28 JST / 工作区：d83598db-3bfe-4386-b86d-ca266aa77e9f*