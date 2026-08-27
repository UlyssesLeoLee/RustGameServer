# 单元测试设计书（工具集 / Unit Test Design Document）

**目录 09 工具集  单元测试（UT）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-UT-09 |
| 版本 | 0.1 |
| 父文档 | RGS-IMPL-001 实施约定 / RGS-SPEC-000 详细设计规格总表 |
| 适用范围 | 验证 rgs-certgen（QUIC/TLS 证书生成工具）的纯函数式逻辑：CA 证书生成、域名服务证书生成、CLI 参数解析 |
| V 模型层级 | TL-1 单元测试 → DTL 详细设计 |
| 编制标准 | IPA 共通框架 2013(SLCP-JCF2013)详细设计工程 / RGS-REQ-001 §12.1 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签） |
| 编制日期 | 2026-08-28 06:50 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-IMPL-001 §4, RGS-SPEC-000 §2.1, RGS-REQ-007 §2.1, RGS-BAS-003 §2.1 |
| 关联源代码 | `crates/rgs-certgen/src/main.rs`（4495 字节, 3 个 fn + 1 个 struct,**均非 pub**——`main.rs:28 struct Cli` / `:49 fn main` / `:74 fn generate_ca` / `:99 fn generate_server_cert`）|
| 关联测试代码 | **暂无**（rgs-certgen 暂未实现任何测试，本次设计先行） |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-28 06:50 JST | 初次编制：09 工具集测试设计书（rgs-certgen 工具类，rgs-arc-olu 占位 crate 不在范围） |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制（兼签）| 架构师 | 2026-08-28 | per DEC-008 一人公司 12 角色兼任 |
| 需求（架构师）| | | DDD Review 阶段补 |
| 设计 QA 员 | | | 待具名（per Q2 OPEN-QA） |
| 变更控制委员会 | | | DDD Review 阶段补 |

## 目录

1. 前言（Preface）
2. 测试策略（Test Strategy）
3. 测试用例（Test Cases）
4. 追溯矩阵（Traceability Matrix）
5. 测试执行计划（Test Execution Plan）
6. 通过判定标准（Pass Criteria）
7. 风险与未决事项（Risks and TBDs）

注：本文件实际以下章节内容为准。

---

## 1. 前言

## 1.1 目的（Purpose）

本文件为 V 模型 **TL-1 单元测试**层级设计书，对应 `rgs-certgen` QUIC/TLS 证书生成工具的源代码（`crates/rgs-certgen/src/main.rs`）。本版本为 0.1 初次编制（per 2026-08-28 06:45 JST Ulysses 指令"整个项目 UT/IT/ST 测试设计书齐全了吗"）。

- 验证 `Cli` 结构体的 clap 派生参数解析
- 验证 `generate_ca()` / `generate_server_cert()` / 整体 `main()` 流程
- 验证输出文件的命名约定（`ca.crt.pem` / `<domain>.crt.pem`）
- 验证有效性周期（validity_days）的边界处理
- 为后续 09 工具集编号域开路（rgs-certgen 是首个工具类 crate）

## 1.2 适用范围（Scope）

| 边界 | 说明 |
|---|---|
| 包含 | rgs-certgen crate **所有 3 个 fn + 1 个 struct(均非 pub,无 lib 入口面)** + Cli 结构体 + 黑盒端到端(assert_cmd 启 binary + 输出文件断言) |
| 排除 | 集成测试（见 RGS-TST-IT-09）、系统测试（见 RGS-TST-ST-09）、rgs-arc-olu 占位 crate（不写设计书）、rcgen 库自身测试 |
| 当前状态 | rgs-certgen 暂未实现任何测试代码（**rgs-testkit-style mock / fixture 0 个**） |

## 1.3 关联文档（Related Documents）

| 文档编号 | 文档名 | 与本文件关系 |
|---|---|---|
| RGS-IMPL-001 实施约定与工程边界 §4 | 实施 | 工具集约束 |
| RGS-SPEC-000 详细设计规格总表 §2.1 | 详细设计 | 父文档 |
| RGS-REQ-007 运维与 GM 后台管控 §2.1 | 需求 | TLS 证书需求来源 |
| RGS-BAS-003 运维与 GM 后台管控 §2.1 | 设计 | mTLS 证书基线 |
| RGS-IMPL-005 BUILD 镜像规范 v0.1 | 实施 | 工具链集成 |
| RGS-TST-UT-00 基准与治理 单元测试设计书 | 参考 | V 模型对应 |

## 1.4 术语与标记规则

per RGS-TST-UT-00 §1.4(RFC 2119 + IPA 共通框架 2013)。
- 测试 ID：`TST-{UT|IT|ST}-09-NNN`
- 用例类型：N=正常 / A=异常 / B=边界

## 1.5 字段级映射

每个测试用例"对应需求"列精确到"`rgs-certgen/src/main.rs` 行号 + 函数名 + 参数"。

## 1.6 命名约定

- 测试 ID：`TST-{UT|IT|ST}-09-NNN`
- V 模型层级标注：UT 无标注
- 测试运行时：`cargo test -p rgs-certgen`

---

## 2. 测试策略

## 2.1 V 模型对应关系

```
需求   RGS-IMPL-001 §4    → ST  (RGS-TST-ST-09)
设计   RGS-SPEC-000 §2.1   → IT  (RGS-TST-IT-09)
详细   rgs-certgen main.rs  → UT  (RGS-TST-UT-09,本文件)
实现   Rust 4495 字节       ←
```

## 2.2 测试层次

| 层次 | 范围 | 工具 |
|---|---|---|
| L1 | crate 内模块 | `cargo test -p rgs-certgen` |
| L2 | 输出文件验证 | `assert_cmd` + tempfile |

## 2.3 接口契约

- CLI：`rgs-certgen --output DIR --domains a,b,c --validity-days N`
- 输出文件：`<DIR>/ca.crt.pem` + `<DIR>/<domain>.crt.pem` + 密钥
- 默认域名：6 个（5 域 + cluster-ops）

## 2.4 测试质量目标

| 维度 | 目标 |
|---|---|
| CLI 解析覆盖 | 100%（3 参数） |
| 输出文件覆盖 | 100%（CA + N 服务） |
| 业务路径覆盖率 | ≥ 70% |

---

## 3. 测试用例

## 3.1 模块 A：Cli 参数解析（`rgs-certgen/src/main.rs:22-47`）

| 测试 ID | 对应源码 | 字段/参数 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-A001 | main.rs:22 Cli | output, domains, validity_days | N | 默认参数：output=./certs, domains=6 个, validity=365 |
| TST-UT-09-A002 | main.rs:22 Cli | --output /tmp/foo | N | 自定义 output 路径生效 |
| TST-UT-09-A003 | main.rs:22 Cli | --domains a,b | N | 自定义 domains 逗号分隔生效 |
| TST-UT-09-A004 | main.rs:22 Cli | --validity-days 30 | N | validity_days 边界值 30 |
| TST-UT-09-A005 | main.rs:22 Cli | --validity-days 0 | B | validity_days 边界值 0（可能 rcgen 拒绝） |
| TST-UT-09-A006 | main.rs:22 Cli | --validity-days 99999 | B | validity_days 极大值 |

**实现位置**：`crates/rgs-certgen/tests/ut_cli.rs`（**TBD,待补**）

## 3.2 模块 B：CA 证书生成（`rgs-certgen/src/main.rs:74-?`）

| 测试 ID | 对应源码 | 字段/输出 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-B001 | main.rs generate_ca | output dir + validity_days | N | 生成 ca.crt.pem + ca.key.pem |
| TST-UT-09-B002 | main.rs generate_ca | IsCa::Ca(BasicConstraints::Unconstrained) | N | CA 标志 + KeyCertSign + CrlSign |
| TST-UT-09-B003 | main.rs:82 generate_ca | 硬编码 CN = "RustGameServer Dev CA" | N | CA CN 字段固定值（无 CLI 可配置参数，CLI 仅 `output` / `domains` / `validity_days`）|

**实现位置**：`crates/rgs-certgen/tests/ut_ca.rs`（**TBD**）

## 3.3 模块 C：服务证书生成（`rgs-certgen/src/main.rs generate_server_cert`）

| 测试 ID | 对应源码 | 字段/输出 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-C001 | generate_server_cert | domain + ca_cert + ca_key + validity | N | 生成 `<domain>.crt.pem` |
| TST-UT-09-C002 | generate_server_cert | SAN Type DNS = domain | N | 服务证书含正确 SAN |
| TST-UT-09-C003 | generate_server_cert | 6 域默认列表 | N | 6 个服务证书都生成 |
| TST-UT-09-C004 | generate_server_cert | 重复 domain | A | 拒绝/覆盖（**待 main.rs 看实际行为**） |

**实现位置**：`crates/rgs-certgen/tests/ut_server_cert.rs`（**TBD**）

## 3.4 模块 D：main 流程（`rgs-certgen/src/main.rs:49-72`）

| 测试 ID | 对应源码 | 字段/输出 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-D001 | main.rs:49 main | fs::create_dir_all | N | 不存在的 output 目录被创建 |
| TST-UT-09-D002 | main.rs:60 + 64 | CA + 6 domains | N | 7 个 PEM 文件（1 CA + 6 server）全生成 |
| TST-UT-09-D003 | main.rs 输出 | println "[rgs-certgen] ..." | N | 输出 4 行 log 含"输出目录 / 域名 / 有效期 / 完成" |
| TST-UT-09-D004 | main.rs 错误处理 | 输出目录不可写 | A | 返回 anyhow::Error + Context |

**实现位置**：`crates/rgs-certgen/tests/integration_main.rs`（**TBD**,与 IT-09 共用）

---

## 4. 追溯矩阵

| 测试 ID | RGS-IMPL | RGS-SPEC | 源码 | 测试代码 |
|---|---|---|---|---|
| TST-UT-09-A001~A006 | §4 工具链 | §2.1 | main.rs:22-47 | **TBD** `ut_cli.rs` |
| TST-UT-09-B001~B003 | §4 工具链 | §2.1 | main.rs:74-? | **TBD** `ut_ca.rs` |
| TST-UT-09-C001~C004 | §4 工具链 | §2.1 | main.rs generate_server_cert | **TBD** `ut_server_cert.rs` |
| TST-UT-09-D001~D004 | §4 工具链 | §2.1 | main.rs:49-72 | **TBD** `integration_main.rs` |

**总计**：17 测试用例 ID（Cli 解析 6 + CA 3 + Server cert 4 + main 流程 4，**全部 TBD**）

---

## 5. 测试执行计划

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L1 本地 | cargo | `cargo test -p rgs-certgen` | 每次 commit |
| L1 CI | cargo + CI | `.github/workflows/rust-ci.yml` | push to main |
| 输出验证 | assert_cmd + tempfile | `assert_cmd::Command::cargo_bin("rgs-certgen").arg("--output").arg(tmpdir).assert().success()` | 集成测试 |

**已知 bug**：rgs-certgen **零测试**（per 2026-08-28 06:50 JST 现状）。本设计书 v0.1 是先编设计后补实现，v0.2 阶段需补 test 实现。

---

## 6. 通过判定标准

| 维度 | 通过阈值 |
|---|---|
| 测试通过率 | 100% |
| CLI 解析覆盖率 | 100%（3 参数） |
| 输出文件覆盖 | 100%（CA + 6 server cert） |
| 业务路径覆盖率 | ≥ 70% |
| 编译警告 | 0 |

---

## 7. 风险与未决事项

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| TBD-09-01 | rgs-certgen 零测试代码 | P1 | 实施 UT-09 A/B/C/D 4 个 test 文件 |
| TBD-09-02 | rgs-certgen 是 bin 不是 lib,只能通过 assert_cmd 黑盒测 | P2 | 可考虑拆 lib.rs + main.rs（per gm-backend 模式）|
| TBD-09-03 | rgs-arc-olu 占位 crate 暂无测试设计（per 2026-08-27 23:35 JST 决议,占位 crate 不写）| P3 | 等 PH-4 实施时再补 |
| TBD-09-04 | 09 编号域开路,后续工具类 crate（rgs-certgen, 未来可能 rgs-archive-tool 等）都归 09 | P3 | 评估 RGS-IMPL-001 §4 是否需调整 |

**保留派生约束**（per 2026-08-26 04:30 JST）：
- 禁"per X 历史形态"等回溯叙事
- 引用 BAS 必须 git log -p --follow 实证
- 缺标比错标安全
- 子代理授权边界要写明"无证据叙事 = 禁止"

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**时间**：2026-08-28 06:50 JST  
**后续**：DDD Review 时由 Ulysses + SRE Lead 联合审
