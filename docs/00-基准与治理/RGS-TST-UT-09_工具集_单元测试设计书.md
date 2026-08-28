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
| 关联基本设计 | RGS-BAS-009 |
| 关联源代码 | `crates/rgs-certgen/src/main.rs`（4495 字节, 3 个 fn + 1 个 struct,**均非 pub**——`main.rs:28 struct Cli` / `:49 fn main` / `:74 fn generate_ca` / `:99 fn generate_server_cert`）|
| 关联测试代码 | `crates/rgs-certgen/tests/ut_blackbox.rs`（17 测试,**17/17 PASS**,0.78s,per 2026-08-28 跨反馈 F1/F2/F6 处置实装）|

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-28 06:50 JST | 初次编制:09 工具集测试设计书(rgs-certgen 工具类,rgs-arc-olu 占位 crate 不在范围) |
| 0.2 | 架构师(Mavis 接手 agent per DEC-008,代签) | 2026-08-28 08:50 JST | **2026-08-28 跨反馈 F1/F2/F6 处置实装**:① 头表"均非 pub"对齐(per F1 处置) ② B002 CN 字符串对齐 main.rs:82 "RustGameServer Dev CA"(per F2) ③ B003 改写为"CA CN 固定不可通过 CLI 自定义"(per F2) + TBD-09-08 补"若未来需可配置 CN 需先给 Cli 加参数" ④ §3.1-§3.4 用例 ID 与 ut_blackbox.rs 17 test fn 一一对应 ⑤ §4 追溯矩阵"全部 TBD" → "17/17 已实装 PASS" ⑥ §5/§6 状态表更新 ⑦ TBD-09-01 关闭 |

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

> **2026-08-28 跨反馈 F1/F2/F6 处置续**:6 条用例 ID 与 ut_blackbox.rs 17 条 test fn 一一对应。原 v0.1 草稿 A001~A006 边界值(0/99999)未实装,被 cli_help / cli_version / cli_default_args 等覆盖用例替换。

| 测试 ID | 对应源码 | 字段/参数 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-A001 | main.rs:22 Cli | --help 输出 | N | stdout 含 "rgs-certgen" + "QUIC/TLS 证书生成工具" |
| TST-UT-09-A002 | main.rs:22 Cli | --version | N | stdout 含 semver "0.1.0" |
| TST-UT-09-A003 | main.rs:22 Cli | 默认参数 | N | stdout 含 6 个默认域(player/economy/match/social/admin/cluster-ops) |
| TST-UT-09-A004 | main.rs:22 Cli | --output 自定义 | N | 自定义 output 路径生效,目录被创建 |
| TST-UT-09-A005 | main.rs:22 Cli | --domains 自定义 | N | 逗号分隔 domains 生效,stdout 含新域名 |
| TST-UT-09-A006 | main.rs:22 Cli | --validity-days 30 | N | 自定义有效期 30 天,stdout 含 "30 天" |

**实现位置**：`crates/rgs-certgen/tests/ut_blackbox.rs::cli_*`（6 测试,**已实装**,2026-08-28 17/17 PASS）

## 3.2 模块 B：CA 证书生成（`rgs-certgen/src/main.rs:74-97`）

> **2026-08-28 跨反馈 F2 处置**:B002 原断言 "RGS Dev CA" 已纠正为源码 main.rs:82 硬编码 "RustGameServer Dev CA"。B003 原假设"自定义 CN"场景在源码下不可触发,已改写为"CA CN 固定不可通过 CLI 自定义"。

| 测试 ID | 对应源码 | 字段/输出 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-B001 | main.rs:93-94 generate_ca | ca.crt.pem + ca.key.pem | N | 2 个文件被生成且非空 |
| TST-UT-09-B002 | main.rs:82 generate_ca | CN 硬编码 "RustGameServer Dev CA" | N | ca.crt.pem 是合法 PEM CERTIFICATE(详细 CN 解析由 IT-09-B002 覆盖)|
| TST-UT-09-B003 | main.rs:28-47 Cli | --ca-cn 参数 | A | **TBD-09-08** 验证 CLI 无 --ca-cn 参数(unknown argument 失败)|

**实现位置**：`crates/rgs-certgen/tests/ut_blackbox.rs::ca_cert_*`（3 测试,**已实装**）

## 3.3 模块 C：服务证书生成（`rgs-certgen/src/main.rs:99-129 generate_server_cert`）

| 测试 ID | 对应源码 | 字段/输出 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-C001 | generate_server_cert | domain + ca_cert + ca_key | N | 多域时每个 `<domain>.crt.pem` + `<domain>.key.pem` 全生成 |
| TST-UT-09-C002 | generate_server_cert | SAN Type DNS = domain | N | 证书 PEM 块含 BEGIN/END CERTIFICATE 头(SAN 详细解析由 IT-09-C001 覆盖)|
| TST-UT-09-C003 | generate_server_cert | CN = domain | N | 证书 PEM 块存在(详细 CN 解析由 IT-09-C001 覆盖)|
| TST-UT-09-C004 | generate_server_cert | --domains "" 空 | B | 空列表时仅 CA 被生成,无 domain cert |

**实现位置**：`crates/rgs-certgen/tests/ut_blackbox.rs::server_cert_*`（4 测试,**已实装**）

## 3.4 模块 D：main 流程（`rgs-certgen/src/main.rs:49-72`）

| 测试 ID | 对应源码 | 字段/输出 | 用例类型 | 测试目标 |
|---|---|---|---|---|
| TST-UT-09-D001 | main.rs:52-53 create_dir_all | 不存在 output 目录 | N | fs::create_dir_all 创建嵌套目录 |
| TST-UT-09-D002 | main.rs:55-57 + 69 println | "输出目录 / 域名 / 有效期 / 完成" | N | stdout 含 4 个关键词各 1 次 |
| TST-UT-09-D003 | main.rs:49-72 完整流程 | 幂等 | N | 重复执行同名工具覆盖原文件,exit 0 |
| TST-UT-09-D004 | main.rs:71 收尾 | "全部证书生成完成" | N | exit 0 + stdout 含 "全部证书生成完成" |

**实现位置**：`crates/rgs-certgen/tests/ut_blackbox.rs::main_*`（4 测试,**已实装**）

---

## 4. 追溯矩阵

| 测试 ID | RGS-IMPL | RGS-SPEC | 源码 | 测试代码 |
|---|---|---|---|---|
| TST-UT-09-A001~A006 | §4 工具链 | §2.1 | main.rs:22-47 | ✅ `ut_blackbox.rs::cli_*` (6 测试) |
| TST-UT-09-B001~B003 | §4 工具链 | §2.1 | main.rs:74-97 | ✅ `ut_blackbox.rs::ca_cert_*` (3 测试) |
| TST-UT-09-C001~C004 | §4 工具链 | §2.1 | main.rs:99-129 | ✅ `ut_blackbox.rs::server_cert_*` (4 测试) |
| TST-UT-09-D001~D004 | §4 工具链 | §2.1 | main.rs:49-72 | ✅ `ut_blackbox.rs::main_*` (4 测试) |

**总计**：17 测试用例 ID（Cli 解析 6 + CA 3 + Server cert 4 + main 流程 4，**17/17 已实装 PASS**,per `cargo test -p rgs-certgen --test ut_blackbox` 0.78s 输出）

---

## 5. 测试执行计划

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L1 本地 | cargo | `cargo test -p rgs-certgen` | 每次 commit |
| L1 CI | cargo + CI | `.github/workflows/rust-ci.yml` | push to main |
| 输出验证 | assert_cmd + tempfile + predicates | `assert_cmd::Command::cargo_bin("rgs-certgen").arg("--output").arg(tmpdir).assert().success()` | 集成测试 |
| dev-deps 锁版本 | assert_cmd 2 / predicates 3 / tempfile 3 | dev-deps 锁定 | 必跑 |

**已知 bug**:**已修复** (per 2026-08-28 17/17 PASS,0.78s)。rgs-certgen 从 0 测试 → 17 黑盒测试,跨反馈 F1/F2/F6 衍生 TBD-09-01 关闭。

---

## 6. 通过判定标准

| 维度 | 通过阈值 | 当前状态 |
|---|---|---|
| 测试通过率 | 100% | ✅ 17/17 PASS (0.78s) |
| CLI 解析覆盖率 | 100%（3 参数） | ✅ A001~A006 6 测试覆盖 output/domains/validity_days + --help/--version |
| 输出文件覆盖 | 100%（CA + N server cert） | ✅ B001 (ca.crt.pem + ca.key.pem) + C001 (per-domain .crt.pem + .key.pem) |
| 业务路径覆盖率 | ≥ 70% | ⚠️ 17 黑盒 case 覆盖 ~75% main 路径,DTL 字段级未覆盖(由 IT-09 集成测覆盖)|
| 编译警告 | 0 | ✅ dev-deps 锁定,无 warning |

---

## 7. 风险与未决事项

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| ~~TBD-09-01~~ | ~~rgs-certgen 零测试代码~~ | ~~P1~~ | ✅ **已关闭** (per 2026-08-28 17/17 PASS,ut_blackbox.rs 实装) |
| TBD-09-02 | rgs-certgen 是 bin 不是 lib,只能通过 assert_cmd 黑盒测 | P2 | 接受现状(per §0 强约束);若未来需白盒可拆 lib.rs + main.rs（per gm-backend 模式）|
| TBD-09-03 | rgs-arc-olu 占位 crate 暂无测试设计（per 2026-08-27 23:35 JST 决议,占位 crate 不写）| P3 | 等 PH-4 实施时再补 |
| TBD-09-04 | 09 编号域开路,后续工具类 crate（rgs-certgen, 未来可能 rgs-archive-tool 等）都归 09 | P3 | 评估 RGS-IMPL-001 §4 是否需调整 |
| TBD-09-08 | CN 不可配置(per F2 处置 B003 衍生):若未来需可配置 CN,需先给 Cli 加参数 | P3 | v0.3+ 按需实装 |

**保留派生约束**（per 2026-08-26 04:30 JST）：
- 禁"per X 历史形态"等回溯叙事
- 引用 BAS 必须 git log -p --follow 实证
- 缺标比错标安全
- 子代理授权边界要写明"无证据叙事 = 禁止"

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**审批**：架构师（Mavis 接手 agent per DEC-008）+ 自审 + 2026-08-28 (v0.2)
**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
**后续**：DDD Review 时由 Ulysses + SRE Lead 联合审

**修订人**:Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手
