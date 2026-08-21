# 03-Rust 1.98 + Cargo.lock + CI 基线

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEPLOY-00-03 |
| 版本 | 0.1（占位 + 文档化）|
| 依据 | RGS-TS-001 v0.6 §3.1 Rust 工具链 + §3.14 CI + RGS-IMPL-001 §2 + RGS-PLAN-001 v0.7 §5.1 |
| 状态 | **🟠 NO-GO 状态** |
| 保密级别 | 内部限定（Internal Use Only）|

---

## §1 Rust 1.98 stable 基线

| 项 | 目标 | 当前 | 责任方 |
|---|---|---|---|
| Rust | 1.98 stable（GA 2026-08-20 已发）| 待实测 | Platform Engineer |
| Cargo | 1.98 | 待实测 | Platform Engineer |
| Edition | 2024 | 待实测 | Platform Engineer |
| resolver | 3 | 待实测 | Platform Engineer |
| MSRV | 1.98 | 待实测 | Platform Engineer |

## §2 workspace 骨架占位（per RGS-IMPL-001 §2）

> **不实际创建**——NO-GO 状态下不创建业务 Rust 代码 / workspace。

```text
Cargo.toml                         # virtual workspace / resolver = "3"
Cargo.lock                         # 唯一根锁文件，必须入仓
proto/rgs/{domain}/v1/*.proto
crates/rgs-{player,economy,match,social,admin}/
crates/rgs-cluster-ops/
crates/rgs-contracts-{domain}/
crates/rgs-testkit/
services/rgs-cluster-ops-service/
services/player-service/
services/economy-service/
services/match-service/
services/social-service/
services/admin-service/
deploy/cluster-manifest/
```

## §3 Cargo.lock 锁定策略

| 项 | 目标 | 责任方 |
|---|---|---|
| `Cargo.lock` 入仓 | 唯一根锁文件 | Platform Engineer |
| `cargo --locked build` | CI 必须用 --locked | Platform Engineer |
| 依赖更新策略 | 每季度评估 + ADR 修订 | Platform Engineer |

## §4 CI 基线（per RGS-TS-001 v0.6 §3.14）

### §4.1 CI 工具

| 工具 | 用途 |
|---|---|
| GitHub Actions | 主 CI/CD 平台 |
| cargo fmt --check | 代码风格门禁 |
| cargo clippy -D warnings | 静态分析门禁 |
| cargo deny check | 许可证 + 漏洞 + 源门禁 |
| cargo audit | CVE 扫描 |
| cargo llvm-cov | 覆盖率报告 |
| sqlx prepare --check | sqlx 编译期一致性 |

### §4.2 占位 workflow（per 04-ci-cd/）

- `rgs-ci.yaml`：主 CI（push / PR 触发；fmt/clippy/test/deny/audit/coverage）
- `rgs-release.yaml`：发布 pipeline（tag 触发；镜像构建 + 部署）
- `rgs-nightly.yaml`：夜间 pipeline（依赖审计 + 漏洞扫描）

## §5 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。Rust 1.98 + Cargo.lock + CI 占位（不实际创建）。 |
