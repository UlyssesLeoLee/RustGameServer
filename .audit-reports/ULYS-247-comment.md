# ULYS-247 依赖审计总结 (ULYS-239 follow-up 复盘)

工具链：`cargo audit 0.22.2` (RustSec advisory-db `3157b0e258782691`, 1273 条目) + `cargo deny 0.20.2` (licenses/bans/sources 子集) + `cargo update --dry-run --workspace --verbose` (outdated 替代)。Lockfile `Cargo.lock` 670 packages / 32 workspace path crates。

## 与 ULYS-239 的差异

| 项 | ULYS-239 | ULYS-247 | 变化 |
|---|---|---|---|
| 总漏洞 | 27 | 27 | 0 |
| 已修 | n/a | RUSTSEC-2024-0437 (protobuf 2.28→3.7.2) | **+1 fixed** |
| 新增 | n/a | RUSTSEC-2026-0285 (rustls 0.23.43, 2026-09-14) | **+1 new** |
| wasmtime / rustls-webpki / h2 | 19.0.2 / 0.102.8 / 0.3.27 | 同 | 未修 |
| 许可 / wildcard | 0BSD+CDLA-Permissive 已 allow / `wildcards=warn` 已生效 | 0 errors, 71 warnings | fixed |

## P0 — 必修（7 日内）

1. **rustls-webpki 0.102.8 → 0.103.13+**（4 HIGH；mTLS 全栈 24 service）— `cargo update -p rustls-webpki --precise 0.103.13`。注：lockfile 已并存 `0.103.15`（sqlx 0.8 内部拉的另一路径），需 patch-path 或 sqlx 0.9 升级统一。
2. **rustls 0.23.43 → 0.23.45**（1 MEDIUM 新增 RUSTSEC-2026-0285 TLS 1.3 handshake）— `cargo update -p rustls --precise 0.23.45`。
3. **h2 0.3.27 → 0.4.16+**（1 LOW DoS；actix-web 全栈）— `cargo update -p h2 --precise 0.4.16`。需验证 actix-http 3.13.5 兼容性（h2 0.3→0.4 是 tokio 拆分）。

## P1 — 建议（独立 issue，14 日内）

- **wasmtime 19→36+**（消除 5 CRITICAL + 8 HIGH + 8 MEDIUM 共 21 条 advisory；function-plane pin 19.x 需先验证 Windows fuel-exhaustion FFI abort 已修复）
- **tonic 0.12 → 0.14**（gRPC 全栈；落后 2 major）
- **opentelemetry 0.24 → 0.33**（53.12 占位未启用，54.13 才正式接入）

## 过期版本（≥2 major 落后）

| Crate | 当前 | 最新 | 落后 major | 影响 |
|---|---|---|---|---|
| wasmtime | 19.0.2 | 49.0.1 | 30 | function-plane 唯一；fix wasmtime CRITICAL/HIGH 的唯一选项 |
| serial_test | 0.5.1 | 4.0.1 | 4 | dev-only |
| jsonwebtoken | 9.3.1 | 11.1.0 | 2 | gm-backend/admin-service JWT；breaking changes 多 |

共 86 个 crate outdated，其中 81 仅 minor/patch 落后。

## P2 — 跟踪

- rsa 0.9.10 (RUSTSEC-2023-0071) — lockfile 不可达（RGS 只用 sqlx-postgres），可加 `audit.toml ignore`
- tokio-tar 0.3.1 (RUSTSEC-2025-0111) — 仅 dev 路径
- unmaintained: backoff / bincode / fxhash / instant / mach / paste / rustls-pemfile / wasmtime-jit-debug
- 多版本并存（47+ distinct crate）— 二进制尺寸 / 编译时间

## 交付物（已落盘 `D:/RustGameServer/`）

- `ULYS-247-audit-report.md` — 人读版（含详细 CVSS 评级 / 触发链 / 不确定事项）
- `.audit-reports/ULYS-247-cargo-audit.json` + `.txt` — cargo audit 完整输出
- `.audit-reports/ULYS-247-vulns-parsed.json` — 27 advisory 结构化清单
- `.audit-reports/ULYS-247-cargo-deny-{bans,licenses,sources}.txt` — deny 子集
- `.audit-reports/ULYS-247-cargo-update-dryrun-v.txt` — 86 outdated 详单
- `.audit-reports/ULYS-247-cargo-outdated.txt` — cargo outdated 失败日志（libsqlite3-sys 冲突已知）

详见附件 + repo 内 `ULYS-247-audit-report.md` 全文。