# 依赖审计报告 (ULYS-253)

> **执行时间**: 2026-10-10 23:15 CST | **工具链**: `npm audit` (7 manifests) + `cargo audit 0.21` (RustSec advisory-db 1296条目) + `cargo deny 0.20` + `cargo update --dry-run` | **Lockfile**: Cargo.lock 672 packages / 32 workspace path crates

---

## 1. npm 审计结果 (7 个 package.json)

| Manifest | 依赖 | `npm audit` 结果 |
|---|---|---|
| `package.json` (根) | `glob ^13.0.6` (构建期) | **0 vulnerabilities** |
| `tools/gm-console/frontend/package.json` | 7 runtime + 5 dev (react/vite/chart.js等) | **2 vulnerabilities**: `esbuild ≤0.24.2` (moderate, dev-server仅) + `vite ≤6.4.2` (high, dev工具链) |
| `tools/h5_e2e/package.json` | `ws ^8.21.3` | **0 vulnerabilities** |
| `tools/rgs-batch-console/package.json` | 无依赖声明 | **0 vulnerabilities** |
| `tools/rgs-shim/package.json` | 无依赖声明 | **0 vulnerabilities** |
| `tools/rgs-web/package.json` | 无依赖声明 | **0 vulnerabilities** |
| `tools/rgs-shanshuo-game/rgs-proxy/package.json` | 空文件 (非合法JSON) | N/A |

**npm 结论**: 仅前端 dev 依赖有 2 个漏洞 (esbuild/vite)，均为开发工具链，非生产运行时。修复需将 `vite` 升至 `^6.4.3` (连带 esbuild 0.25+) —— 涉及 breaking changes，建议单独 issue 跟进。

---

## 2. Rust 漏洞 (cargo audit) — 27 个 RUSTSEC 命中 / 6 distinct crate

### 2.1 CRITICAL (5) — 全部 wasmtime 19.0.2
| Advisory | 标题 | 修复版本 |
|---|---|---|
| RUSTSEC-2024-0438 | Windows 设备文件名沙箱逃逸 | ≥24.0.2 / ≥25.0.3 |
| RUSTSEC-2025-0118 | — | ≥37.0.3 / ≥38.0.4 |
| RUSTSEC-2026-0088 | — | ≥36.0.7 / ≥42.0.2+ |
| RUSTSEC-2026-0096 | — | ≥36.0.7 / ≥42.0.2+ |
| RUSTSEC-2026-0269 | 文件系统 trailing-slash 沙箱逃逸 | ≥36.0.14 / ≥46.0.3+ |

**影响**: function-plane → admin-service → gm-backend (dev)。wasmtime 19→49 跨度 30 个 major，**需独立 issue 验证 Windows fuel-exhaustion FFI abort 是否已修复后再升级**。

### 2.2 HIGH (12) — rustls-webpki 0.102.8 (4) + wasmtime (8)
| Advisory | Crate | 标题 | 修复 |
|---|---|---|---|
| RUSTSEC-2026-0049 | rustls-webpki 0.102.8 | CRLs 失效 | ≥0.103.10 |
| RUSTSEC-2026-0098 | rustls-webpki 0.102.8 | URI name constraints 错误接受 | ≥0.103.12 |
| RUSTSEC-2026-0099 | rustls-webpki 0.102.8 | wildcard name + name constraints 错误接受 | ≥0.103.12 |
| RUSTSEC-2026-0104 | rustls-webpki 0.102.8 | CRL 解析可达 panic | ≥0.103.13 |
| + 8 个 wasmtime HIGH | wasmtime 19.0.2 | 各类沙箱/类型安全问题 | ≥36.0.7+ |

**关键**: rustls-webpki 0.102.8 来自 `async-nats 0.42.0` (非 sqlx)，修复路径需升级 `async-nats 0.42→0.50` (8 major breaking)。

### 2.3 MEDIUM (10) — wasmtime (8) + rsa + tokio-tar + rustls NEW
| Advisory | Crate | 标题 | 修复/备注 |
|---|---|---|---|
| RUSTSEC-2026-0285 | **rustls 0.23.43** (NEW) | TLS 1.3 handshake 跨级接受 | **≥0.23.45** (patch bump) |
| RUSTSEC-2023-0071 | rsa 0.9.10 | Marvin Attack timing sidechannel | **无修复版本**；RGS 仅用 sqlx-postgres，运行时不可达 |
| RUSTSEC-2025-0111 | tokio-tar 0.3.1 | PAX header 解析错误允许文件伪造 | **无修复版本**；仅 dev/test 路径 (testcontainers) |
| + 8 个 wasmtime MEDIUM | wasmtime 19.0.2 | panic/OOB/resource exhaustion | ≥24.0.6+ / ≥36.0.6+ |

### 2.4 LOW (1)
| Advisory | Crate | 标题 | 修复 |
|---|---|---|---|
| RUSTSEC-2026-0258 | h2 0.3.27 | unbounded empty DATA frames | ≥0.4.16 |

---

## 3. cargo-deny 发现

### 3.1 许可
- **0 errors, 2 warnings**: `OpenSSL` / `Unicode-DFS-2016` 在 allow 列表但未遇到
- **已修复** (per ULYS-239): 0BSD (quoted_printable via lettre) + CDLA-Permissive-2.0 (webpki-roots via sqlx) 已加入 allow

### 3.2 Wildcard 依赖
- **71 warnings** (path-based workspace dep)，deny.toml `wildcards = "warn"` 已生效，CI 不 exit 7
- 20 个 workspace crate 使用了 wildcard，建议逐步收紧为 caret

### 3.3 多版本并存 (47+ distinct crate)
- 最严重: `hashbrown` ×5、`windows-sys` ×4、`base64/rand/rand_core` ×3
- 来源: syn 1/2、thiserror 1/2、bytes 1 大版本过渡
- 仅影响二进制尺寸/编译时间，无运行时正确性问题

### 3.4 Unmaintained (7 crate, informational)
| Crate | 备注 |
|---|---|
| backoff | 替代: `backon` (活跃) |
| bincode | 团队停止维护；替代: wincode/postcard/bitcode/rkyv |
| fxhash | 替代: `rustc-hash` |
| instant | 替代: `web-time` |
| mach | macOS-only dev path |
| paste | 归档；替代: `pastey` / `with_builtin_macros` |
| rustls-pemfile | 归档；迁移到 `rustls-pki-types` (1.9+) |

---

## 4. 过期版本 (cargo update --dry-run) — 31 个 unchanged

### ≥2 major 落后 (任务要求: >2 major versions behind)
| Crate | 当前 | 最新 | 落后 major | 影响 |
|---|---|---|---|---|
| **wasmtime** | 19.0.2 | 49.0.2 | **30** ⚠️ | function-plane 唯一；修复所有 wasmtime 漏洞唯一路径 |
| **serial_test** | 0.5.1 | 4.0.1 | **4** | dev-only (gm-backend 测试串行化) |
| **jsonwebtoken** | 9.3.1 | 11.1.0 | **2** | gm-backend/admin-service JWT；v10 breaking (algorithm enum 化) |

### 1 major 落后 (记录)
| Crate | 当前 | 最新 |
|---|---|---|
| opentelemetry / -otlp / -sdk / -semantic-conventions | 0.24/0.17/0.24/0.16 | 0.33.1 |
| tracing-opentelemetry | 0.25.0 | 0.34.0 |
| tonic / tonic-build / tonic-health | 0.12.3 | 0.14.6 |

### 其余 81 个仅 minor/patch 落后 (常规维护窗)

---

## 5. 行动项汇总 (Action Items)

### P0 — 必修 (漏洞 + 全栈影响)
| # | 行动 | 命令/说明 | 工作量 |
|---|---|---|---|
| **1** | **rustls 0.23.43 → 0.23.45** (RUSTSEC-2026-0285 MEDIUM) | `cargo update -p rustls --precise 0.23.45` — 单次 patch bump | 1 小时 |
| **2** | **rustls-webpki 0.102.8 → 0.103.13+** (4 HIGH) | 需升级 `async-nats 0.42→0.50` (8 major breaking，Jetstream API 变更) — 独立 issue | 半日+ |
| **3** | **h2 0.3.27 → 0.4.16+** (LOW DoS) | `cargo update -p h2 --precise 0.4.16` + 验证 actix-http 3.x 兼容性 | 半日 |

### P1 — 建议 (独立 follow-up)
| # | 行动 | 说明 |
|---|---|---|
| **4** | **wasmtime 19 → 36+** | 消除 5 CRITICAL + 8 HIGH + 8 MEDIUM；需先验证 Windows fuel-exhaustion FFI abort 修复 |
| **5** | **tonic 0.12 → 0.14** | gRPC 全栈 24 service；RGS-IMPL-003 工具链层任务 |
| **6** | **opentelemetry 0.24 → 0.33** | 53.12 OTel SDK 占位，54.13 正式接入 |

### P2 — 跟踪
| # | 行动 | 说明 |
|---|---|---|
| **7** | rsa/tokio-tar ignore 决策 | lockfile-only / dev-only；可在 `audit.toml` 加 ignore + reason |
| **8** | serial_test 0.5.1 → 4.0.1 | dev-only |
| **9** | jsonwebtoken 9→11 | JWT breaking changes review |
| **10** | unmaintained crates 替换 | 批量收紧 workspace.dependencies 时处理 |
| **11** | 多版本并存优化 | 二进制尺寸/编译时间优化 |

### P3 — 流程
| # | 行动 | 说明 |
|---|---|---|
| **12** | CI 加 `cargo deny check` | deny.toml 已就位；rust-ci.yml 缺 deny step (55.5 任务) |
| **13** | Baseline 落盘 | 当前 Cargo.lock + 审计产物写入下次对比基线 |

---

## 6. 关键交付物 (本次审计生成)

| 文件 | 内容 |
|---|---|
| `.audit-reports/ULYS-253-cargo-audit.json` | cargo audit 完整 JSON (1296 advisories db) |
| `.audit-reports/ULYS-253-cargo-deny-bans.txt` | cargo deny bans 完整输出 (71 wildcard warnings + 多版本详单) |
| `.audit-reports/ULYS-253-cargo-deny-licenses.txt` | cargo deny licenses (0 errors, 2 warnings) |
| `.audit-reports/ULYS-253-cargo-update-dryrun.txt` | 31 unchanged crates 完整清单 |
| `ULYS-253-audit-report.md` | 本报告 (人读版) |

---

## 7. 待决策事项

1. **wasmtime 升级策略**: function-plane README 明确 19.x pin 为规避 Windows fuel-exhaustion FFI abort (`STATUS_STACK_BUFFER_OVERRUN`)；需先确认 wasmtime 20+ 是否已修复 (查 CHANGELOG/Issues)
2. **rustls-webpki 双版本并存**: lockfile 同时存在 0.102.8 (async-nats 路径) 和 0.103.15 (sqlx/rustls 路径)；需 async-nats 升级或 patch-path 强制
3. **h2 0.3→0.4 与 actix-http 3.x 兼容性**: actix-http 3.13.5 当前依赖 h2 0.3；h2 0.4 是 tokio 拆分产物，actix-http 4.x 才支持
4. **rsa/tokio-tar ignore**: 需 D-Boy 确认是否加入 `audit.toml [advisories] ignore`
5. **许可白名单扩充**: 0BSD/CDLA-Permissive-2.0 均为 permissive，零合规风险，待确认

---

*ULYS-253 / 执行: Hermes Agent / 工作区: d83598db-3bfe-4386-b86d-ca266aa77e9f*