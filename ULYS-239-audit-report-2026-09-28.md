# ULYS-239 依赖安全审计复核报告（2026-09-28 JST）

## 范围与复现

- Rust workspace：32 个成员，当前 Cargo.lock 有 672 个依赖条目。
- cargo-audit 0.22.2：RustSec 数据库 1,273 条记录；快照 commit ef03605143a913024f864d2edf476adad5720c93，更新时间 2026-09-28 18:30 JST。
- cargo-deny 0.20.2；cargo-outdated 0.19.0；npm 10.9.3。
- 7 个 package.json 中有 3 个声明依赖；没有 go.mod / go.sum，Go 扫描不适用。

根 Cargo.lock 被 .gitignore:179 排除。当前锁文件 SHA-256 为 B1F189ECC7B5CDC2DFD1D08C25463D7D19858423850BA1518F04E27F3BFEC74C，已保存为 .audit-reports/ULYS-239-cargo-lock-2026-09-28.lock。已有 .audit-reports/ULYS-239-cargo-lock.baseline.json 的 SHA-256 为 E5F366594111DDA0798892A645B85FB01488695769AF3BE44FE6FDBD11B1C6BE，不是本轮锁文件。CI 若要复现，应先把本轮快照恢复为根 Cargo.lock，再运行 cargo audit / cargo deny check；本轮没有改 CI。

仓库没有 npm lockfile。本次在隔离临时目录对 3 个有依赖的 manifest 生成临时锁后运行 npm audit；结果代表本次 registry 解析，不代表已部署版本。JSON 结果见 .audit-reports/ULYS-239-npm-audit-2026-09-28.json。临时 npm 锁没有加入仓库。

## Rust 漏洞

cargo audit 报告 26 个漏洞 advisory、涉及 5 个 crate。RustSec 提供 CVSS 的项目为 Critical 2、High 1、Medium 10、Low 6；另外 7 项没有 CVSS，明确标为“未评分”，不按描述推测严重度。

| 包 / 版本 | 严重度与 advisory | 依赖路径及建议 |
|---|---|---|
| wasmtime 19.0.2 | Critical 2：RUSTSEC-2026-0095、0096；High 1：RUSTSEC-2026-0269；Medium 9：RUSTSEC-2026-0020、0021、0085、0087、0089、0091、0092、0093、0094；Low 6：RUSTSEC-2024-0439、RUSTSEC-2025-0046、0118、RUSTSEC-2026-0086、0088、0222；未评分 1：RUSTSEC-2024-0438 | function-plane → admin-service。共同修复范围至少 47.0.4，dry-run 候选为 49.0.1。function-plane README §9.1 第 124 行记录 Windows fuel-exhaustion 的 wasmtime_longjmp FFI abort pin；先单独验证再升级。 |
| rustls-webpki 0.102.8 | 未评分 4：RUSTSEC-2026-0049、0098、0099、0104 | 经 async-nats 0.42 到 shared-platform / rgs-overflow-alert；不是 SQLx 路径。四项共同修复需至少 0.103.13。先升级 async-nats（候选 0.50.0），确认锁到修复版本并复验 mTLS。 |
| h2 0.3.27 | 未评分 1：RUSTSEC-2026-0258 | actix-http 3.18.12 → actix-web 4.15.0 → gm-backend / rgs-grpc-bridge。修复版本 h2 0.4.16+；当前 actix-http 3.x 锁在 h2 0.3，等待兼容上游。 |
| rsa 0.9.10 | Medium，CVSS 5.9：RUSTSEC-2023-0071；无修复版本 | cargo tree --workspace --all-features --target all 未找到活动依赖路径，是 Cargo.lock 命中而非启用图依赖。确认可选 MySQL 路径未启用后再决定移除或记录例外。 |
| tokio-tar 0.3.1 | 未评分：RUSTSEC-2025-0111；无修复版本 | 经 testcontainers 0.23.3 → rgs-testkit，仅 dev/test 图。升级 testcontainers 及 modules，或移除未使用的测试依赖。 |

本轮 cargo audit 没有报告 protobuf 漏洞。另有 7 个 unmaintained：backoff、bincode、fxhash、instant、mach、paste、rustls-pemfile；建议按来源更新上游或替换，rustls-pemfile 可随 async-nats 升级改用 rustls-pki-types。另有 unsound：wasmtime-jit-debug（RUSTSEC-2024-0442），随 wasmtime 迁移处理。

cargo deny check 退出码 1：26 个漏洞与 7 个 unmaintained 导致 advisories 检查失败；licenses、bans、sources 均通过；wildcard 和多版本仍有警告。

## npm 漏洞

| Manifest | 临时解析版本 | 结果 |
|---|---|---|
| package.json | glob 13.0.6 | 0 |
| tools/h5_e2e/package.json | ws 8.22.0 | 0 |
| tools/gm-console/frontend/package.json | vite 5.4.21、esbuild 0.21.5、@vitejs/plugin-react 4.7.0 | 2 个受影响 package（Vite High、esbuild Moderate），共 4 个 advisory |

- Vite High：GHSA-fx2h-pf6j-xcff，受影响 <=6.4.2，6.4.3 已修复。
- Vite Moderate：GHSA-4w7w-66w2-5vf9，受影响 <=6.4.1，6.4.2 已修复。
- Vite 经 launch-editor 的 Moderate：GHSA-v6wh-96g9-6wx3；launch-editor <=2.14.0 受影响，2.14.1 已修复；Vite 6.4.3 已修复该路径。
- esbuild Moderate：GHSA-67mh-4wv8-2f99，受影响 <=0.24.2，0.25.0 已修复。

npm audit 自动修复建议 Vite 8.3.1。官方 advisory 列出 Vite 6.4.3 为修复版本；隔离临时副本使用 Vite ^6.4.3 重跑后得到 vite 6.4.3、esbuild 0.25.12、@vitejs/plugin-react 4.7.0，npm audit 为 0。建议把 Vite 约束升至 ^6.4.3 并生成 package-lock.json；前端构建仍待执行。

另外 4 个 package.json 没有依赖声明：tools/rgs-batch-console、tools/rgs-web、tools/rgs-shim、tools/rgs-shanshuo-game/rgs-proxy。

## 超过两级版本差距的候选

cargo outdated --workspace --format json 未能解析完整 workspace：sqlx-sqlite 0.8.0 要求 libsqlite3-sys ^0.28.0，rusqlite 0.40.2 要求 ^0.38.1，存在相同 native links 冲突。单 crate 探测也未在限时内产出。cargo update --dry-run --verbose 成功列出可用版本；以下严格按“差距 > 2”过滤，不代表完整 transitive inventory。

1.x 及以上按 semver major 计算；0.x 的 minor release series 单独列出，不称作 major。

| 包 | 当前 → 候选 | 差距 / 风险 |
|---|---|---|
| wasmtime | 19.0.2 → 49.0.1 | +30 major；有 Critical / High 漏洞，受 Windows fuel pin 阻断。 |
| actix-web-lab | 0.24.3 → 0.28.0 | +4 个 0.x series；age-only，h2 是间接 finding。 |
| async-nats | 0.42.0 → 0.50.0 | +8 个 0.x series；关联 rustls-webpki 未评分漏洞。 |
| bcrypt | 0.16.0 → 0.19.3 | +3 个 0.x series；age-only。 |
| criterion | 0.5.1 → 0.8.2 | +3 个 0.x series；dev/bench，age-only。 |
| opentelemetry | 0.24.0 → 0.33.0 | +9 个 0.x series；age-only。 |
| opentelemetry-otlp | 0.17.0 → 0.33.0 | +16 个 0.x series；age-only。 |
| opentelemetry_sdk | 0.24.1 → 0.33.0 | +9 个 0.x series；age-only。 |
| opentelemetry-semantic-conventions | 0.16.0 → 0.33.0 | +17 个 0.x series；age-only。 |
| rusqlite | 0.32.1 → 0.40.2 | +8 个 0.x series；age-only，也是 resolver 冲突一侧。 |
| serial_test | 0.5.1 → 4.0.1 | 0.x 到 4.x 的迁移，不按普通 0.x gap 解释；dev/test。 |
| testcontainers | 0.23.3 → 0.28.0 | +5 个 0.x series；dev/test，涉及 tokio-tar。 |
| testcontainers-modules | 0.11.6 → 0.15.0 | +4 个 0.x series；dev/test。 |
| tokio-tungstenite | 0.24.0 → 0.30.0 | +6 个 0.x series；age-only。 |
| tracing-opentelemetry | 0.25.0 → 0.34.0 | +9 个 0.x series；age-only。 |

严格差距为 2 或更小的候选不在表内，如 tonic 0.12→0.14、jsonwebtoken 9→11、rand 0.8→0.10、mockall 0.13→0.15、rustler 0.36→0.38。筛选清单见 .audit-reports/ULYS-239-major-upgrades-2026-09-28.txt。

## 建议顺序与已知缺口

1. 排期修复 wasmtime；先解决 function-plane Windows fuel-exhaustion 兼容性。
2. 将 GM Console Vite 更新到 6.4.3 并生成 npm lockfile；临时 audit 已通过，构建待执行。
3. 升级 async-nats 并确认 rustls-webpki >=0.103.13；复验 mTLS。
4. 跟踪 actix-http 对 h2 0.4 的兼容更新。
5. 更新 dev/test 容器依赖，清理未启用的 rsa 路径及 unmaintained / unsound 依赖。
6. 将 Rust lockfile snapshot 恢复步骤接入 CI；npm 应用提交 lockfile 并在 CI 使用 npm ci。

已知缺口：cargo outdated 完整 workspace 列表受 sqlite resolver 冲突阻断；npm 没有提交锁文件，实际部署版本无法由本次临时审计证明；RustSec 有 7 条未评分项；本轮未运行应用构建或测试。
