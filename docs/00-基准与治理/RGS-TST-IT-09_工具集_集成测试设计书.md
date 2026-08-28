# 集成测试设计书（工具集 / Integration Test Design Document）

**目录 09 工具集  集成测试（IT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-IT-09 |
| 版本 | 0.1 |
| 父文档 | RGS-IMPL-001 实施约定 §4 / RGS-SPEC-000 详细设计规格总表 §2.1 |
| 适用范围 | 验证 rgs-certgen 与文件系统 + 证书解析(openSSL/rcgen 互操作)的端到端流程 |
| V 模型层级 | TL-2 接口契约 / TL-3 协议一致性 / TL-4 集成（端到端）|
| 编制标准 | IPA 共通框架 2013(SLCP-JCF2013)详细设计工程 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签） |
| 编制日期 | 2026-08-28 06:52 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-IMPL-001 §4, RGS-SPEC-000 §2.1, RGS-REQ-007 §2.1, RGS-BAS-003 §2.1 |
| 关联基本设计 | RGS-BAS-009 |
| 关联源代码 | `crates/rgs-certgen/src/main.rs`（4495 字节） |
| 关联测试代码 | **暂无** |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-28 06:52 JST | 初次编制 |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制（兼签）| 架构师 | 2026-08-28 | per DEC-008 |
| 变更控制委员会 | | | DDD Review 阶段补 |

## 目录

1. 前言（Preface）
2. 测试策略
3. 测试用例
4. 追溯矩阵
5. 测试执行计划
6. 通过判定标准
7. 风险与未决事项

---

## 1. 前言

## 1.1 目的（Purpose）

TL-2/3/4 层级设计书,验证 rgs-certgen 端到端:

- 与文件系统交互(目录创建、PEM 文件写入)
- 与外部证书解析工具的互操作(openSSL 解析生成的 PEM)
- 与 dev k3s 部署链的集成(生成的证书被 5 域 service mTLS 消费)

## 1.2 适用范围（Scope）

| 边界 | 说明 |
|---|---|
| 包含 | assert_cmd 启动 rgs-certgen binary + tempfile 验证输出文件 + 证书解析互操作 |
| 排除 | 单元测试（见 RGS-TST-UT-09）、k3s 集成（见 e2e-smoke） |

## 1.3 关联文档

per RGS-TST-UT-09 §1.3 通用。

---

## 2. 测试策略

## 2.1 V 模型对应

```
rgs-certgen main.rs 4495 字节
  → L1 UT (TST-UT-09) 单测
  → L2 IT (TST-IT-09,本文件) 黑盒 + 文件 + 解析
  → L3 ST (TST-ST-09) 部署链
```

## 2.2 测试工具

| 工具 | 用途 |
|---|---|
| `assert_cmd 2` | spawn rgs-certgen binary,断言 stdout/exit code |
| `tempfile 3` | 临时目录,避免污染 |
| `rcgen 0.13` | 解析生成的 PEM(同 lib 复用) |
| `rustls-pemfile 2` | 解析 PEM 文件 |

## 2.3 测试质量目标

| 维度 | 目标 |
|---|---|
| 黑盒接口覆盖率 | 100%（3 CLI 参数 × 2 模式 = 6 用例） |
| 文件产物覆盖率 | 100%（CA + server PEM） |
| 互操作验证 | 100%（openSSL/rcgen 都能解析） |

---

## 3. 测试用例

## 3.1 模块 A：端到端 CLI 调用

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-09-A001 | IMPL-001 §4 | spawn `rgs-certgen --output /tmp/x` | [TL-2/4] | N | 退出码 0 + 1 CA + 6 server cert 共 7 个文件 |
| TST-IT-09-A002 | IMPL-001 §4 | `--domains a,b,c` | [TL-2/4] | N | 1 CA + 3 server cert |
| TST-IT-09-A003 | IMPL-001 §4 | `--validity-days 30` | [TL-2/4] | N | 证书 validity 字段 ≤ 30 天 |
| TST-IT-09-A004 | IMPL-001 §4 | 组合 `--output /tmp/y --domains p,e --validity-days 1` | [TL-2/4] | N | 端到端组合 |

**实现位置**：`crates/rgs-certgen/tests/integration_cli.rs`（**TBD**）

## 3.2 模块 B：输出文件 schema 验证

| 测试 ID | 对应需求 | 字段/schema | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-09-B001 | IMPL-001 §4 | `ca.crt.pem` PEM 格式 | [TL-3] | N | PEM 头"-----BEGIN CERTIFICATE-----" |
| TST-IT-09-B002 | IMPL-001 §4 | CA 证书 subject = "RustGameServer Dev CA" | [TL-3] | N | subject CN 字段（硬编码 `main.rs:82`,非 CLI 可配置） |
| TST-IT-09-B003 | IMPL-001 §4 | CA is_ca=true, key_usages=[KeyCertSign, CrlSign] | [TL-3] | N | CA 标志正确 |
| TST-IT-09-B004 | IMPL-001 §4 | `<domain>.crt.pem` PEM | [TL-3] | N | 服务证书 PEM 格式 |
| TST-IT-09-B005 | IMPL-001 §4 | 服务证书 SAN = domain | [TL-3] | N | SAN DNS 字段 |
| TST-IT-09-B006 | IMPL-001 §4 | 服务证书 is_ca=false | [TL-3] | N | 不是 CA 标志 |

**实现位置**：`crates/rgs-certgen/tests/integration_pem.rs`（**TBD**）

## 3.3 模块 C：互操作（openSSL 解析）

| 测试 ID | 对应需求 | 字段/工具 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-09-C001 | IMPL-001 §4 + IMPL-005 | openssl x509 -in ca.crt.pem -text | [TL-4] | N | openSSL 能解析 |
| TST-IT-09-C002 | IMPL-001 §4 + IMPL-005 | openssl verify -CAfile ca.crt.pem server.crt.pem | [TL-4] | N | 服务证书由 CA 签发成功 |
| TST-IT-09-C003 | IMPL-001 §4 + IMPL-005 | openssl s_server test | [TL-4] | N | 启动 TLS server 成功(可选) |

**实现位置**：`crates/rgs-certgen/tests/integration_openssl.rs`（**TBD,需 openssl CLI**）

## 3.4 模块 D：CI/CD 链集成

| 测试 ID | 对应需求 | 字段/工具 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-IT-09-D001 | RGS-IMPL-001 §4 + IMPL-005 | `make certs` 或 `cargo run -p rgs-certgen` | [TL-4] | N | CI 脚本能调用 |
| TST-IT-09-D002 | RGS-IMPL-001 §4 | k3s tls secret | [TL-4] | N | 生成 cert 后能 create secret |

**实现位置**：`scripts/ci-integration.sh`（**TBD**）

---

## 4. 追溯矩阵

| 测试 ID | RGS-IMPL | RGS-SPEC | 源码 | 测试代码 |
|---|---|---|---|---|
| TST-IT-09-A001~A004 | §4 | §2.1 | main.rs:49-72 | **TBD** `integration_cli.rs` |
| TST-IT-09-B001~B006 | §4 | §2.1 | main.rs:74-? | **TBD** `integration_pem.rs` |
| TST-IT-09-C001~C003 | §4 + IMPL-005 | §2.1 | main.rs | **TBD** `integration_openssl.rs` |
| TST-IT-09-D001~D002 | §4 | §2.1 | scripts/ | **TBD** `ci-integration.sh` |

**总计**：15 测试用例 ID（**全部 TBD**）

---

## 5. 测试执行计划

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L4 本地 | cargo | `cargo test -p rgs-certgen --test integration_*` | 每次 commit |
| L4 CI | cargo + CI | `.github/workflows/rust-ci.yml` | push to main |
| L4 互操作 | shell + openssl | `scripts/cert-openssl-verify.sh` | v0.2 实施时 |

---

## 6. 通过判定标准

| 维度 | 通过阈值 |
|---|---|
| 端到端 spawn | 100% |
| PEM schema 验证 | 100% |
| openSSL 互操作 | 100%（开箱即用,无 manual fix） |
| 编译警告 | 0 |

---

## 7. 风险与未决事项

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| TBD-09-01 | 零集成测试代码 | P1 | 实施 IT-09 A/B/C/D 4 个 test 文件 |
| TBD-09-02 | openSSL 互操作需 openssl CLI 在 PATH | P2 | CI 镜像预装 / or skip on Windows |
| TBD-09-03 | k3s tls secret 集成(per k8s TLS Secret spec)需 kubectl + cluster | P3 | 集成 e2e-smoke 后续 |

**保留派生约束**（per 2026-08-26 04:30 JST）：同 UT-09 §7。

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**时间**：2026-08-28 06:52 JST
