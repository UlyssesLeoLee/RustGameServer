# 技术选型报告（技術選定レポート / Technology Selection Report）

**RustGameServer 分布式游戏服务器 — 主要技术选型 v0.6**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TS-001 |
| 版本 | 0.6 |
| 父文档 | RGS-REQ-001 需求定义书（贯穿 22+ 域） |
| 配套文档 | 19 份 ADR（RGS-ADR-0001～0054）、RGS-OPS-001 部署运维说明、RGS-IMPL-001 实施约定、各域 BAS/DTL、RGS-QA-001 v0.13、RGS-PLAN-001 v0.8 |
| 制定日 | 2026-08-19 |
| 最终更新日 | 2026-08-21 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-19 | 架构师 | 初版制定。覆盖 16 个分层领域、60+ 技术组件、版本号与许可证状态；每条选型给出理由 + 备选 + 关联 ADR/TBD；按 ARC-014 判定基准确认"确有取舍"项与"决策显然"项分类 |
| 0.2 | 2026-08-20 | 架构师 | 修正按状态统计口径；新增长期记忆向量存储 TBD-MEM-001（待附件 D 登记及具名人类审批），不新增或伪造 ADR。 |
| 0.3 | 2026-08-21 | 架构师 | 按最新官方发布核验并更新 Rust stable 1.97.1、PostgreSQL 18.6、Actix Web 4.14.1；明确 PostgreSQL 19 Beta 不进入生产基线。 |
| 0.4 | 2026-08-21 | 架构师 | 收敛 RGS-IMPL-001：固定 virtual workspace、领域/服务分离、proto 与 migration 所有权、CI、错误/序列化/OTel、Saga、测试、运行时、安全和部署约定；Rust 1.98 为用户指定的 stable 目标，GA 与 CI 核验前不得宣称环境可用。 |
| 0.5 | 2026-08-21 | 架构师 | **§6.2 OLU token 重算**（per user preference 2026-08-21：AI 辅助开发场景下用 token 而非人天算 OLU；人天单位在 AI 协作下失去精度）。**引入 token-OLU 框架**：①1 人·天 ≈ 100K-300K tokens（含输入 + 输出 + 决策对话 + 验证往返）②1 人·周 ≈ 500K-1.5M tokens ③1 SRE 上限 = 1 人·周 ≈ 1M tokens。**§6.2 OLU 估算表重算**：原 v0.4 19 人·天/周 ≈ 4 SRE 改为 token 估算（按 AI 协作基线）。**5 域独立 Lead × 14-18 周 token 估算**：80-120M tokens（per RGS-PLAN-001 v0.8 + RGS-QA-001 v0.13 DEC-005 + DEC-006 路径 B），由 SRE Lead + PM 校准。**NFR-OP-010 软化**：原"2 SRE ≤ 20 人·天/周"硬上限在 AI 协作场景下不再适用；token 单位的实际约束是"上下文窗口" / "单次会话成本" / "决策质量（多轮对话）"。**下游级联**：RGS-QA-001 v0.13 §9.5.3 路径 B 标记 "OLU 校准待 SRE Lead + PM" 由本节 v0.5 校准取代；RGS-PLAN-001 v0.8 §6 Q-015 / RGS-IMPL-001 资源约束栏 / RGS-ADR-0025 OLU 预算 ADR（如果存在）需同步对齐。**未变更**：16 个分层领域选型、60+ 技术组件、版本号、许可证、ADR 关系、TBD 列表。 |
| 0.6 | 2026-08-21 | 架构师 | **§6.2 双轨制 OLU**（per user decision 2026-08-21：人·天/周 + token/周 **两种算法都要**，不是 token 取代人·天）：①§6.2.1 人·天/周算法（v0.4 算法，**保留 active**，主用于纯人类开发场景 / SRE Lead + PM 工时核算 / HR 编制申请 / 工时审计）②§6.2.2 token/周算法（v0.5 新增算法，**active**，主用于 AI 协作开发场景 / AI 算力预算 / 决策质量评估）③§6.2.3 双轨对比与切换条件（按开发模式选算法；不允许混算）④§6.2.4 5 域独立 Lead × 14-18 周 **双算法估算**（人·天 ~28-35 / token ~196M-468M）⑤§6.2.5 NFR-OP-010 双轨（人·天 ≤ 20 / token ≤ 20M）⑥§6.2.6 校准路径 4 节点（PH-0.5 / PH-1 / PH-3 / PH-7）。**算法选择原则**：纯人类开发用人·天；AI 协作开发用 token；混合开发按主导模式选 + 双轨并报。**下游级联**：RGS-IMPL-001 资源约束栏加双轨说明；RGS-QA-001 v0.13 §9.5.3 路径 B 标记 OLU 双轨已就位。 |

## 审批栏

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-19 | — |
| 评审（架构） | | | 与 RGS-ADR-0008 中间件导入判定基准的一致性；与 ARC-014 / ARC-026 守门 |
| 评审（SRE／运维） | | | 与 RGS-OPS-001 部署说明的版本对齐 |
| 评审（安全） | | | 许可证合规（CON-001/002）；漏洞面与升级路径 |
| 审批（负责人） | | | 本文档的基准化 |

> **基线状态说明**：本报告中的“已决/一致”描述技术选型语义或已有 ADR 关系，不等于本报告已经完成基准化审批。Rust 1.98 stable 是用户指定的目标；截至本文制定日可核验的 stable 仍为 1.97.1，因此 1.98 在正式 GA、可安装并通过 CI 前不是可用环境基线。Actix Web 4.14.1、PostgreSQL 18.6 同样须经 CI/预发核验和回归证据后才能成为生产完成事实。

> 本文档**不替代**各 ADR 的逐项决定——ADR 是"确有取舍"的单点决定记录，本文档是"全栈选型一览"，便于评审、招聘、新成员上手。每条选型**必须**可追溯到对应 ADR 或 TBD；ARC-014 判定基准要求"未证明需要不引入"——故"暂未选型"也是合法状态，本文档**显式标注**哪些领域**未**引入中间件。

---

## 目录

1. 前言
   1.1 目的
   1.2 适用范围
   1.3 关联文档
   1.4 选型原则（基于 ARC-014 / ARC-026 / CON 系列）
   1.5 命名约定
2. 分层选型总览
3. 选型清单
   3.1 语言与工具链
   3.2 异步运行时与网络 I/O
   3.3 RPC 与序列化
   3.4 数据库（OLTP）
   3.5 缓存与会话存储
   3.6 事件总线与消息
   3.7 沙箱脚本引擎（插件热插拔）
   3.8 智能层（不确定层 L4）
      3.8.4 长期记忆向量存储（待决）
   3.9 可观测性
   3.10 边缘与网络
   3.11 部署与编排
   3.12 对象存储
   3.13 客户端与 SDK
   3.14 持续集成／持续部署
   3.15 性能与负载测试
   3.16 安全与密钥
4. 版本与生命周期政策
5. 已决 vs 未决选型（TBD 集中列表）
6. 风险与例外

---

# 1. 前言

## 1.1 目的

本系统（RustGameServer）从 2026-08-15 立项至本报告制定时点，已演进至多域需求、基本/详细设计与 18 份 ADR。技术选型散落在 ADR、各 DTL 的"组件选型"段、RGS-OPS-001 部署说明等位置，**没有一份统一的"全栈一览"文档**。

本文档的目的：

1. **单一入口**：新成员 / 评审人 / 招聘面试时**只读一份**就能理解全栈技术选型
2. **版本冻结基线**：明确每项组件的版本号与升级策略，避免各文档版本号不一致
3. **ARC-014 守门**：显式区分"已选型（含理由）"与"未选型（含何时选）"，把"未证明需要不引入"的原则落到纸面
4. **可追溯**：每条选型**必须**引用对应 ADR 或 TBD；新增选型**必须**先有 ADR 评审，方可进入本文档

## 1.2 适用范围

| 范畴 | 说明 |
|---|---|
| **覆盖** | 服务端（Rust 进程集）、智能层（Python 子系统）、客户端核心（C ABI 共享库）、部署与运维栈、可观测性栈、CI/CD 栈、测试工具栈 |
| **不覆盖** | 业务功能设计（属各域 REQ/BAS/DTL）、产品功能（属产品 PRD，与本仓库无关）、商业 SaaS 选型（属项目立项阶段） |

## 1.3 关联文档

| 文档 | 关系 |
|---|---|
| RGS-REQ-001 / 22+ 域 REQ | 需求来源——选型必须可追溯到具体 FR/NFR |
| RGS-BAS-001 主基本设计 | 部署构成、5 服务、DB 论理、事件与可观测性、API 字段级 |
| RGS-BAS-010 设计模式与核心算法总纲 | G-001～013 设计模式 / 算法选型 |
| RGS-OPS-001 部署运维说明 | 端口/容量/环境/排障的运维实操面 |
| RGS-ADR-0001～0051 全部 ADR | 单点技术决定的权威来源 |
| 附件 D（§3 决议登记 / §4 OSS 许可 / §5 OLU 台账） | 治理闭环输入 |

## 1.4 选型原则（基于 ARC-014 / ARC-026 / CON 系列）

依 RGS-ADR-0008 与 RGS-ADR-0025，本项目技术选型遵循三条硬性原则：

| 原则 | 出处 | 含义 |
|---|---|---|
| **未证明需要不引入** | ARC-014 / RGS-ADR-0008 | 任何新中间件/库/工具引入前**必须**经 ADR 评审，证明其承担的职责是当前架构无法以更低成本实现的。"架构看起来漂亮"不是充分理由 |
| **OLU 预算硬约束** | ARC-026 / RGS-ADR-0025 | 每个新组件按 RGS-BAS-009 §3 OLU 公式核算运维负荷，缺额不补；2 SRE 团队是 NFR-OP-010 的硬上限 |
| **开源 + OSI 认可** | CON-001 / RGS-ADR-0044 | 优先 OSI 认可开源许可（MIT / Apache-2.0 / BSD-2/3 / MPL-2.0 / ISC）；商业 SaaS 默认不引入，需经独立 ADR 评估 |

辅助原则：

- **"确有取舍"才写 ADR**（RGS-ADR-0001～0051 写作前提）——选型无明显取舍时直接记入本文档，不单独立 ADR
- **同一类目只选一个**——避免同一职责有多套实现（例：序列化只用 Protobuf，不混 JSON+Protobuf）
- **可降级原则**——核心组件故障**必须**有降级路径（缓存降级 / 熔断 / 隔离），选型时一并评估

## 1.5 命名约定

| 标签 | 含义 |
|---|---|
| **【已决】** | 已有 ADR 支撑（ADR 编号在引用栏） |
| **【一致】** | 与既有 ADR 隐含选择一致，无需单独立 ADR（理由在备注栏） |
| **【目标基线·待审批】** | 版本已核验并作为当前工程目标，但技术选型报告/环境尚未完成具名基准化审批 |
| **【待决 TBD-NNN】** | 选型尚未完成，登记至附件 D §1.3 |
| **【否决】** | 此前曾考虑但已明确否决（理由附后，避免重提） |

---

# 2. 分层选型总览

| 层级 | 数量 | 主要选型 |
|---|---|---|
| 3.1 语言与工具链 | 6 | Rust 1.98 stable（用户目标、待 GA/核验）、virtual Cargo workspace、Cargo lock、cargo-generate、Clippy/rustfmt、工程约定 |
| 3.2 异步运行时与网络 | 4 | Tokio 1.x、Actix Web 4.14.1、quinn（QUIC）、tokio-rustls |
| 3.3 RPC 与序列化 | 4 | tonic（gRPC）、prost（Protobuf codegen）、Connect（HTTP/gRPC 桥接，可选） |
| 3.4 数据库 | 3 | PostgreSQL 18.6、sqlx、Redis 7.2+ |
| 3.5 缓存与会话 | 1 | Redis 7.2+（同 3.4） |
| 3.6 事件总线 | 1 | NATS JetStream 2.10+ |
| 3.7 沙箱脚本引擎 | 1 | Rhai 1.x |
| 3.8 智能层（不确定层） | 4 | Python 3.11+、LangGraph、LiteLLM（或自托管 vLLM/TGI）、向量存储（待决） |
| 3.9 可观测性 | 4 | OpenTelemetry SDK + Collector 0.96+、Prometheus 2.48+、Grafana 10+、tracing/tracing-subscriber |
| 3.10 边缘与网络 | 2 | OpenResty 1.21+、CloudNative-PG Operator（PH-2 起） |
| 3.11 部署与编排 | 4 | Ubuntu 22.04 LTS、Kubernetes 1.28+、Helm 3.13+、Docker 24+ |
| 3.12 对象存储 | 1 | MinIO RELEASE.2024-08+ |
| 3.13 客户端与 SDK | 3 | Rust 核心 + C ABI FFI、Unity / Unreal / Godot 三个薄适配 |
| 3.14 CI/CD | 2 | GitHub Actions、cargo-deny / cargo-audit / cargo-llvm-cov |
| 3.15 性能与负载测试 | 2 | k6 0.49+、playwright（仅 UI 自动化） |
| 3.16 安全与密钥 | 3 | HashiCorp Vault（或自托管 OpenBao，可选）、ring / RustCrypto、rustls |

本表共列出 44 个分层条目（含待决项）；已决、待决与明确否决的去重统计以 §5 为准，避免将 ADR 引用重复计数。

---

# 3. 选型清单

## 3.1 语言与工具链

### 3.1.1 主语言：Rust stable（目标 1.98）

| 项目 | 内容 |
|---|---|
| **决定** | 【目标基线·待审批】用户指定 Rust 1.98 stable；正式 GA、可安装和全量 CI 核验前，不得把它写为当前已验证版本。 |
| **理由** | 项目命名即"RustGameServer"——内存安全 + 零成本抽象 + 生态（Tokio/Actix Web/tonic/sqlx）已成熟；与 ARC-005 权威边界、ARC-021 拒绝动态库加载、ARC-022 业务逻辑不入库三道防线均依赖 Rust 的编译期保证 |
| **备选** | C++（否决：内存安全保证弱、与 ARC-005 冲突）、Go（否决：GC 暂停对 20Hz tick 路径不可接受、RGS-REQ-001 NFR-PE-002 排除）、Zig（否决：生态不成熟） |
| **版本策略** | CI/生产构建只接受 stable；1.98 GA 后以 `rust-toolchain.toml` 与根 `Cargo.lock` 固定已验证构建。任何 stable 升级必须通过全量 CI、迁移/回滚和性能回归；不得以 beta/nightly 绕过 Gate。 |
| **引用** | RGS-REQ-001 §6.1 编程语言约束；RGS-ADR-0022 业务逻辑不入库（依赖 Rust 类型系统）；RGS-ADR-0020 拒绝动态库加载（依赖 Rust ABI 稳定性）；Rust 官方 release announcements |

### 3.1.2 编译与包管理：Cargo workspace

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】根 virtual Cargo workspace，显式 members 为 `crates/*` 与 `services/*`，`resolver = "3"`；`proto/` 与 `deploy/` 不作为 Cargo member。 |
| **理由** | 官方工具链、零配置；workspace 支持唯一根 `Cargo.lock`、共享 `target/`、shared dev-deps，并保持业务库、contracts 与部署二进制的依赖方向清晰。 |
| **备选** | Bazel（否决：构建图复杂度对中小团队不划算；RGS-ADR-0025 OLU 预算下维护成本高）、Nix（否决：NixOS-only 假定，与 Ubuntu LTS 22.04 主力冲突） |
| **引用** | RGS-BAS-002 §3 脚手架目录结构；RGS-IMPL-001 §2 |

### 3.1.3 脚手架生成器：cargo-generate

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】cargo-generate（基于 Tera 模板） |
| **理由** | 与 Cargo 工具链原生集成；模板用 Jinja/Tera 语法，运营/新成员上手成本低 |
| **备选** | 内部 CLI（否决：维护成本高于采用开源）、Cookbook 文档（否决：可发现性差，无法强制五要素脚手架） |
| **引用** | RGS-BAS-002 §4 挂载脚手架检查清单 |

### 3.1.4 静态分析：Clippy + rustfmt

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】`cargo clippy --all-targets --all-features -- -D warnings`（CI 必跑）+ `cargo fmt --check`（CI 必跑）；`clippy::pedantic` 仅逐条经 review 启用。 |
| **理由** | 官方工具，与 rustc 同源；`-D warnings` 将 lint 警告视为错误，避免"lint 洪水" |
| **备选** | 自定义 lint crate（否决：维护成本不划算） |
| **引用** | RGS-BAS-009 §4 CI 校验脚本 |

### 3.1.5 依赖审计：cargo-deny + cargo-audit

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】cargo-deny（许可证 + 重复依赖 + 已知漏洞）+ cargo-audit（RustSec 公告） |
| **理由** | 满足 ARC-014 导入判定的"开源 OSI 认可"（CON-001）；CI 中日构建必跑，违规阻断合并 |
| **引用** | RGS-ADR-0008 §4 落地；RGS-REQ-001 §12.2 AC-015 许可盘点 |

### 3.1.6 实施约定与依赖边界

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】RGS-IMPL-001 是 workspace、crate、proto、migration、CI、错误、测试与部署的唯一工程约定索引。 |
| **边界** | 禁止泛化 `rgs-common`；领域库与服务 bin 分离；contracts 按域生成；共享 testkit 只用于测试；根 `Cargo.lock` 必须入仓并由 CI `--locked` 校验。 |
| **引用** | [RGS-IMPL-001](../13-实现规格/RGS-IMPL-001_实施约定与工程边界.md) §2～§6。 |

---

## 3.2 异步运行时与网络 I/O

### 3.2.1 异步运行时：Tokio 1.x

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】Tokio 1.x（含 `tokio` 主 crate + `tokio-util` + `tokio-stream`） |
| **理由** | Rust 异步生态事实标准；与 tonic / sqlx / hyper / quinn 全栈兼容；提供多线程调度器、IO 驱动、定时器、信号处理 |
| **备选** | async-std（否决：生态显著小于 Tokio，库兼容性差）、smol（否决：同上）、自研 runtime（否决：违反 ARC-014 未证明需要不引入） |
| **版本策略** | 跟随 1.x 主线（破坏性变更罕见） |
| **引用** | RGS-BAS-001 §3 部署构成（5 服务 + 网关的并发模型） |

### 3.2.2 HTTP 服务框架：Actix Web

| 项目 | 内容 |
|---|---|
| **决定** | 【目标基线·待审批】Actix Web 4.14.1（运行于 Tokio；Cargo manifest 声明 4.14.1，锁文件固定实际解析版本） |
| **理由** | 提供稳定的 HTTP/1.x、HTTP/2、路由、中间件、WebSocket 与 streaming 能力；与 Tokio 生态兼容，适合五域 App/AdminService 的 HTTP ingress。tonic/hyper 仍只作为内部 gRPC/HTTP 底层依赖，不再作为业务 HTTP 框架。 |
| **版本策略** | 跟随 Actix Web 4.x 最新稳定版本；每次升级必须核对 MSRV、Tokio/hyper 兼容性并通过 API/契约/负载回归。 |
| **引用** | RGS-BAS-001 §3.2 API 网关；RGS-REQ-024 VIZ 无限画布 BFF；Actix Web 官方 crate 文档 |

### 3.2.3 QUIC 协议栈：quinn

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】quinn 0.10+（基于 tokio + rustls） |
| **理由** | Rust 生态唯一稳定 QUIC 实现；满足 ARC-003 传输方式决定（QUIC over UDP）；与 NAT 穿透、移动网络、连接迁移场景天然兼容 |
| **备选** | 自己基于 quiche 绑库（否决：维护成本高）、msquic C 绑定（否决：违反 ARC-020 拒绝动态库）、纯 TCP+TLS1.3（否决：丢包/重连延迟不满足 NFR-PE-*） |
| **引用** | RGS-BAS-001 §3.1 网关；RGS-REQ-001 §3 业务需求 BR-002 断线重连 |

### 3.2.4 TLS：rustls（通过 tokio-rustls / quinn）

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】rustls（纯 Rust TLS 实现） |
| **理由** | 纯 Rust 内存安全 + 无 OpenSSL 依赖；QUIC 强制要求 TLS 1.3，rustls 1.x 支持完整 |
| **备选** | OpenSSL（否决：C 依赖、内存安全不可控、违反 ARC-005）、BoringSSL（否决：同上）、NSS（否决：Mozilla-only） |
| **引用** | RGS-REQ-010 网络安全；RGS-ADR-0020 拒绝动态库加载（rustls 不引入 C FFI） |

---

## 3.3 RPC 与序列化

### 3.3.1 内部 RPC：tonic（gRPC over HTTP/2）

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】tonic 0.10+（含 prost 0.12+） |
| **理由** | Rust 生态事实标准 gRPC 实现；与 Tokio / hyper 同源；protobuf 强类型契约与 ARC-005 权威边界契合 |
| **备选** | Apache Thrift（否决：Rust 工具链成熟度低于 gRPC）、Cap'n Proto（否决：跨语言客户端覆盖不足）、JSON-RPC（否决：性能与类型安全不满足 ARC-005）、REST + JSON（否决：除 OpenAPI 文档外弱契约；仅供 COC UI 外部 API 使用） |
| **引用** | RGS-BAS-001 §6.3 5 服务 gRPC 协议；RGS-BAS-003 §3 AdminService 扩展模式 |

### 3.3.2 协议缓冲区：Protobuf 3（通过 prost）

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】protobuf 3.x，prost 0.12+ codegen |
| **理由** | 与 tonic 强绑定；契约与代码生成同步；向后兼容字段（field number）由 protoc 编译期保证 |
| **备选** | FlatBuffers（否决：零拷贝收益不抵工具链复杂度）、MessagePack（否决：跨语言互操作性弱于 protobuf） |
| **引用** | RGS-BAS-001 §3 服务接口定义；RGS-BAS-031 §6 ClusterOpsService 协议 |

### 3.3.3 客户端传输：C ABI（与 Protobuf 解耦）

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Rust 核心逻辑 → C ABI 暴露（extern "C"），由 Unity C# / Unreal C++ / Godot GDScript 三个薄适配层调用 |
| **理由** | ARC-024 决定：客户端核心逻辑单一实现，多引擎薄适配层——避免三端重复实现引入行为漂移；C ABI 跨语言可移植性最佳；FFI 边界有专门 lint（见 RGS-DTL-008） |
| **备选** | C++ 暴露（否决：要求客户端引擎全部用 C++，违反 RGS-ADR-0023）、WebAssembly 暴露（否决：移动端性能开销、调试链路断裂） |
| **引用** | RGS-ADR-0023 客户端核心逻辑单一实现；RGS-REQ-012 三引擎一致接入 |

### 3.3.4 内部 HTTP 桥（可选）：Connect

| 项目 | 内容 |
|---|---|
| **决定** | 【待决 TBD-077】tonic-grpc JSON bridge 由 tonic 自带；如需完整 Connect 协议（含 streaming over HTTP/1.1），评估 buf-build/connect-go 互操作 |
| **理由** | 仅在需要与外部 HTTP/1.1 系统对接时引入；当前未见明确需求 |
| **引用** | RGS-OPS-001 端口规划 8080/TCP（HTTP API 网关） |

---

## 3.4 数据库（OLTP）

### 3.4.1 主数据库：PostgreSQL 18.6

| 项目 | 内容 |
|---|---|
| **决定** | 【目标基线·待审批】PostgreSQL 18.6（5 个独立 DB：player_db / economy_db / match_db / social_db / admin_db） |
| **理由** | 满足 ARC-008 独立 DB 原则；提供 ACID、强类型、JSONB、部分索引、物化视图、分区表和 LISTEN/NOTIFY；与 Rust 的 sqlx 集成成熟；开源（PostgreSQL License）。PostgreSQL 19 截至本次核验仍为 Beta，不作为生产基线。 |
| **备选** | MySQL 8.x（否决：ACID 语义弱于 PG、JSON 支持弱、Outbox 实现需更多绕路；RGS-ADR-0007 已决），TiDB（否决：分布式事务对单机 5 DB 架构过剩；OLU 成本不划算；ARC-014 排除），CockroachDB（否决：同上），SQLite（否决：单进程，多副本机制不成熟） |
| **版本策略** | 开发/预发/生产基线固定 PostgreSQL 18.6；后续 18.x 补丁只能在预发灰度、备份恢复与迁移回退演练通过后升级；19.x 仅在 GA 后重新进行兼容性、OLU、迁移和回退评审 |
| **引用** | RGS-REQ-001 §5.2 DB 论理设计；RGS-ADR-0007 道具与货币统合；RGS-ADR-0008 §3.2 备选；RGS-OPS-001 §1.3 PostgreSQL；PostgreSQL 官方 release notes |

### 3.4.2 数据库访问：sqlx（异步 + 编译期校验）

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】sqlx 0.7+（异步，编译期 SQL 校验） |
| **理由** | 编译期 `query!()` 宏校验 SQL 与 DB schema 一致（捕获 schema 漂移）；无 ORM 抽象（避免"隐藏的 N+1"与隐式类型转换）；与 Tokio 深度集成 |
| **备选** | Diesel（否决：同步为主、async 适配层不够成熟），SeaORM（否决：ORM 抽象层违背"业务逻辑不入库"精神），tokio-postgres（裸驱动，需自写连接池与重试逻辑） |
| **引用** | RGS-BAS-001 §3 服务实现；RGS-ADR-0022 业务逻辑不入库（"无 ORM"是落地手段之一） |

### 3.4.3 数据库算子：CloudNative-PG Operator（PH-2 起）

| 项目 | 内容 |
|---|---|
| **决定** | 【待决 TBD-MNT-002】生产环境由人工/脚本切主备，开发/预发由 CloudNative-PG Operator 自管理；最终选型待 PH-2 前评审 |
| **理由** | CNPG 是 PostgreSQL 在 K8s 上的事实标准 Operator；自动备份、PITR、流复制均开箱；与本项目 RGS-BAS-026 备份 RTO 目标契合 |
| **备选** | Zalando Postgres Operator（否决：维护节奏慢于 CNPG、社区活跃度下降），自建 pg_basebackup + repmgr（否决：OLU 成本高） |
| **引用** | RGS-REQ-027 §10 ARC-042 编排层与 DB 协同；RGS-REQ-031 §8.2（DB侧约束）及 RGS-REQ-031 §9（FR-INT-003） |

---

## 3.5 缓存与会话存储

### 3.5.1 缓存：Redis 7.2+ Cluster

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Redis 7.2+（主从 + Sentinel 或 Cluster 模式，T2 档起 Cluster） |
| **理由** | 满足 ARC-013 背压设置位置（限流计数器 / 在线状态 / 排行榜热数据）；成熟稳定；License 切换为 RSAL/SSPL 后仍可自托管；Lua 脚本支持复杂原子操作 |
| **备选** | Memcached（否决：缺乏持久化、缺乏复杂数据结构，与 ARC-013 限流实现路径不匹配），KeyDB（否决：与 Redis 7 兼容性未保证），DragonflyDB（否决：尚不成熟；PH-7 后可重新评估），自研 in-memory store（否决：违反 ARC-014） |
| **引用** | RGS-REQ-001 §3 业务需求 BR-002 断线重连（session_epoch 存储）；RGS-OPS-001 §1.3 Redis 7.2+ |

### 3.5.2 暂未引入：Hazelcast / Apache Ignite

| 项目 | 内容 |
|---|---|
| **状态** | 【否决】分布式内存网格 |
| **理由** | Redis 已覆盖 80% 需求；Hazelcast / Ignite 引入新运维实体 + 新学习成本；ARC-014 "未证明需要不引入" |

---

## 3.6 事件总线与消息

### 3.6.1 事件总线：NATS JetStream 2.10+

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】NATS JetStream 2.10+（含 `nats` Rust client + JetStream 持久化模式） |
| **理由** | 满足 ARC-010（事件命名 + partition_key 规则）+ ARC-011（Saga 边界）；单一二进制（Go 实现），运维简单；持久化 + 重放 + 消费者组 + DLQ 一站式；比 Kafka 资源占用低一个量级 |
| **备选** | Apache Kafka（否决：JVM 资源开销、运维复杂度；RGS-ADR-0008 §3.2 备选），RabbitMQ（否决：吞吐/分区能力低于 NATS JetStream；与 ARC-010 partition_key 设计契合度低），Redis Streams（否决：可靠性 + 复制能力弱于 NATS），Apache Pulsar（否决：生态复杂度高） |
| **引用** | RGS-BAS-001 §4.7 事件与可观测性设计；RGS-ADR-0015 Saga 边界；RGS-REQ-031 CEM 中心事件管理 |

### 3.6.2 暂未引入：Schema Registry（独立服务）

| 项目 | 内容 |
|---|---|
| **状态** | 【否决】Confluent Schema Registry / Apicurio Registry |
| **理由** | 当前 Schema 存于 `admin_db.event_schema_registry` 表（与 ARC-039 集群元数据同库）；引入独立 Registry 服务增加 OLU，不符合 ARC-014 |
| **例外条件** | 当事件族超过 200 个且跨组织治理需要时重新评估 |

---

## 3.7 沙箱脚本引擎（插件热插拔）

### 3.7.1 沙箱脚本：Rhai 1.x

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Rhai 1.x（MIT 许可） |
| **理由** | RGS-ADR-0020 决定：拒绝动态链接库；Rhai 提供"天然沙箱"——无 fs/net/syscall 暴露；与 Rust 同进程 FFI 风险可控；MIT 许可符合 CON-001 |
| **备选** | Lua（mlua）（否决：需 C FFI 边界 + C 库），WASM（wasmtime）（否决：RGS-ADR-0020 显式记录"插件复杂度增长时首选升级路径"，目前未到该阶段），自研 DSL（否决：维护成本与生态不划算），JavaScript（deno_core / boa）（否决：内存占用与启动延迟不满足"分钟级上线"） |
| **引用** | RGS-ADR-0020 §2 决定；RGS-DTL-005 §7 引擎选型；RGS-REQ-009 插件热插拔 |

---

## 3.8 智能层（不确定层 L4）

### 3.8.1 智能层语言：Python 3.11+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Python 3.11+（仅用于智能层 L4 子系统；服务端主体仍为 Rust） |
| **理由** | RGS-ADR-0026 §3.2 与 RGS-ADR-0029 §2 决定：智能层为 L4（不确定层），必须有"独立子系统"承载；Python 生态（LangGraph、LiteLLM、向量库、MLOps）显著成熟于 Rust 端同类工具；与 Rust 通过 gRPC 严格隔离（ARC-030 闸门） |
| **备选** | Rust + 自研 LLM 编排（否决：实现成本不划算；ARC-014 排除），商业 SaaS（OpenAI Assistants / Anthropic Claude API）（否决：CON-002 商业 SaaS 排查；自托管为默认路径），TypeScript + LangChain.js（否决：与 Rust 端 FFI 边界处理不直接） |
| **引用** | RGS-ADR-0026 仿生分层叙事 + 智能层只读感知；RGS-ADR-0029 L0~L4 确定性分级；RGS-REQ-014 智能决策层 |

### 3.8.2 智能层编排：LangGraph

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】LangGraph（Python，LangChain 生态） |
| **理由** | 显式状态机 + 图编排，原生支持确定性闸门审查（"是否在 L4 边界内"）；可与 LangSmith 集成做追踪 |
| **备选** | LangChain Agents（否决：抽象粒度粗，闸门审查难落地），自研图执行器（否决：违反 ARC-014），AutoGen（否决：与 ARC-030 单向闸门假设不完全契合） |
| **引用** | RGS-REQ-014 智能决策层；RGS-BAS-011 §5A 分析图治理 |

### 3.8.3 LLM 推理：LiteLLM（代理） + 自托管 vLLM/TGI

| 项目 | 内容 |
|---|---|
| **决定** | 【待决 TBD-NEURO-002】LiteLLM 代理（统一 API 入口）+ 后端选 vLLM 或 TGI（自托管推理），最终选型待 PH-5 前 |
| **理由** | LiteLLM 提供 OpenAI 兼容接口，降低切换后端成本；vLLM / TGI 为开源推理引擎（OSI 认可） |
| **备选** | 直接调 OpenAI / Anthropic API（否决：CON-002；数据合规风险），llama.cpp（否决：性能不足，仅适合边缘场景） |
| **引用** | RGS-REQ-014 §3 推理后端；RGS-ADR-0026 治理闭环 |

### 3.8.4 长期记忆向量存储：待决 pgvector vs Milvus

| 项目 | 内容 |
|---|---|
| **决定** | 【待决 TBD-MEM-001】候选为 `pgvector`（随 PostgreSQL 运维）与 `Milvus`（独立向量服务）；当前未选择任一方案。 |
| **理由** | RGS-BAS-033 需要长期记忆检索能力，但两方案在容量、隔离、备份、许可和 OLU 上的代价不同，不能以架构图中的并列名称替代选型。 |
| **负责人** | 架构师、DBA、SRE Lead。 |
| **截止** | PH-3 前；在附件 D 登记、完成许可/OLU/容量评估并经具名人类审批前，不得作为生产依赖。 |
| **引用** | RGS-BAS-033 §2；RGS-REQ-033 FR-AGP-003。 |

---

## 3.9 可观测性

### 3.9.1 数据采集：OpenTelemetry SDK + Collector 0.96+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】OpenTelemetry（OTel）Rust SDK 0.24+ + OTel Collector 0.96+ |
| **理由** | ARC-017 可观测性自 PH-1 起必须具备；OTel 是 CNCF 毕业项目，跨语言、跨后端的标准；黄金指标（延迟/流量/错误/饱和度）由 ARC-017 既定；Protobuf over OTLP 协议 |
| **备选** | OpenTracing + OpenCensus 旧 API（否决：已合并到 OTel），自研 trace SDK（否决：违反 ARC-014），Datadog APM（否决：商业 SaaS + 资源外送） |
| **引用** | RGS-REQ-008 埋点与日志规范；RGS-REQ-001 §10 ARC-017 |

### 3.9.2 指标存储：Prometheus 2.48+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Prometheus 2.48+（与 Alertmanager 配套） |
| **理由** | CNCF 毕业项目；与 OTel 指标导出路径（OTLP → Collector → Prometheus exporter）天然兼容；与 K8s 服务发现原生集成 |
| **备选** | InfluxDB（否决：生态弱于 Prometheus，与 OTel 集成路径更长），VictoriaMetrics（否决：可作为 Prometheus 远端存储后端，但暂不替换主存），Thanos / Cortex（否决：PH-2 后视规模引入，目前单实例足够） |
| **引用** | RGS-REQ-001 §10 ARC-017；RGS-OPS-001 §1.3 Prometheus 2.48+ |

### 3.9.3 仪表盘：Grafana 10+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Grafana 10+ |
| **理由** | 与 Prometheus / OTel 数据源原生集成；可视化能力成熟；可托管于 K8s |
| **备选** | Kibana（否决：偏 ELK 栈，需先引入 Elasticsearch 才能用），自研 Web 仪表盘（否决：违反 ARC-014） |
| **引用** | RGS-REQ-001 §10 ARC-017；RGS-OPS-001 §1.3 Grafana 10+ |

### 3.9.4 日志：tracing + tracing-subscriber + Loki

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】Rust 端 `tracing` + `tracing-subscriber`（结构化日志、OTel exporter 桥）；后端 Loki（待决 TBD-LOG-001 评估） |
| **理由** | tracing crate 是 Rust 生态事实标准；与 OTel 集成（tracing-opentelemetry）成熟；结构化日志（JSON）+ trace 关联符合 ARC-017 |
| **备选** | log4rs（否决：API 较老、不与 tracing 互通），slog（否决：生态弱于 tracing），直接 println（否决：违反 ARC-017） |
| **引用** | RGS-REQ-008 §5 埋点 SDK；RGS-BAS-004 §3 强制采集 |

---

## 3.10 边缘与网络

### 3.10.1 CDN 边缘：OpenResty 1.21+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】OpenResty 1.21+（基于 Nginx + LuaJIT） |
| **理由** | Lua 边缘脚本可与 Rust gRPC 互通（通过 nginx-grpc-module 或 OpenResty 自带 lua-resty-grpc）；LuaJIT 性能足；WAF 模块（lua-resty-waf）开箱；自托管 + 开源（OSI 认可） |
| **备选** | 商业 CDN（Cloudflare / Akamai）（否决：CON-002 商业 SaaS 默认不引入；RGS-ADR-0044 客户端资源分发明示），Envoy（否决：作为边缘反代可，但 WAF/规则生态弱于 OpenResty），Caddy（否决：插件生态不成熟） |
| **引用** | RGS-OPS-001 §1.3 OpenResty 1.21+；RGS-ADR-0044 客户端资源分发 |

### 3.10.2 暂未引入：Service Mesh（Istio / Linkerd）

| 项目 | 内容 |
|---|---|
| **状态** | 【否决】Istio / Linkerd |
| **理由** | ARC-007 单写者 + NetworkPolicy + RBAC 已在应用层实现；Service Mesh 增加 OLU 显著；ARC-014 排除 |
| **例外条件** | 当多语言客户端/服务出现且 mTLS 通信矩阵无法在应用层管理时重新评估 |

---

## 3.11 部署与编排

### 3.11.1 主机 OS：Ubuntu 22.04 LTS

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Ubuntu 22.04 LTS（统一） |
| **理由** | 5 年安全更新至 2027；systemd / netplan / AppArmor 与 RGS-BAS-026 备份/恢复脚本兼容；运维脚本与监控 exporter 均针对 Ubuntu 优化 |
| **备选** | Debian 12（否决：与 Ubuntu LTS 差距小但 LTS 支持期短 2 年），RHEL 9（否决：商业订阅费 + CentOS Stream 替代方案不成熟），AlmaLinux（否决：同上） |
| **引用** | RGS-OPS-001 §1.2 OS；RGS-BAS-026 §3 备份恢复 |

### 3.11.2 容器：Docker 24+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Docker 24+（含 buildkit） |
| **理由** | 行业事实标准；与 K8s/CRI 兼容；multi-stage build 减少镜像体积；buildkit 缓存加速 CI |
| **备选** | Podman（否决：K8s 集成路径稍长），containerd-only（否决：本地开发体验不如 Docker） |
| **引用** | RGS-OPS-001 §1.3 Docker 24+ |

### 3.11.3 编排：Kubernetes 1.28+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Kubernetes 1.28+（自建用 kubeadm；云上用 EKS / GKE / AKS） |
| **理由** | CNCF 毕业项目；自托管/托管两种部署路径都成熟；NetworkPolicy / PodDisruptionBudget / HPA / PDB 满足 RGS-BAS-026 高可用与 ARC-007 部署边界 |
| **备选** | Docker Swarm（否决：功能集显著小于 K8s），Nomad（否决：生态与人才储备弱于 K8s），Amazon ECS-only（否决：单一云锁定，违反 RGS-ADR-0033 部署区域方针） |
| **引用** | RGS-REQ-001 §10 ARC-018；RGS-BAS-001 §3.2 部署构成；RGS-OPS-001 §1.3 K8s 1.28+ |

### 3.11.4 模板：Helm 3.13+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Helm 3.13+ |
| **理由** | K8s 生态事实标准；模板 + value 分离；与 GitOps 工具（ArgoCD / Flux）原生兼容；与 ARC-018 挂载脚手架的 Helm chart 路径一致 |
| **备选** | Kustomize（否决：表达能力弱于 Helm 模板；可与 Helm 互补而非替代），Jsonnet（否决：生态与人才储备弱），自研模板引擎（否决：违反 ARC-014） |
| **引用** | RGS-BAS-002 §3.2 脚手架；RGS-BAS-024 §2 集群清单 Schema；RGS-OPS-001 §1.3 Helm 3.13+ |

---

## 3.12 对象存储

### 3.12.1 自托管对象存储：MinIO RELEASE.2024-08+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】MinIO RELEASE.2024-08+（AGPL-3.0） |
| **理由** | RGS-ADR-0044 决定客户端资源分发默认自托管 + 开源；MinIO 兼容 S3 API（降低切换商业云的迁移成本）；单二进制部署；K8s operator 成熟 |
| **备选** | 直接用 AWS S3 / GCS / 阿里云 OSS（否决：CON-002；商业 SaaS 默认不引入），Ceph RGW（否决：部署复杂度显著高于 MinIO），Garage（否决：生态不成熟） |
| **License 注意** | MinIO 自 2024-01 起从 Apache-2.0 切换为 AGPL-3.0——属于 OSI 认可开源（CON-001），但若二次开发需评估 AGPL 传染性；本项目仅作为客户端调用、不修改 MinIO 源码，影响有限 |
| **引用** | RGS-ADR-0044 §2 决定；RGS-OPS-001 §1.3 MinIO |

---

## 3.13 客户端与 SDK

### 3.13.1 客户端核心：Rust 编译为 C ABI 共享库

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】Rust 核心逻辑（`crates/client_core/*`）→ `cdylib` → C ABI 暴露 |
| **理由** | RGS-ADR-0023 决定客户端核心逻辑单一实现；Rust 编译为 C ABI 跨三端可移植；FFI 边界安全 lint 已在 RGS-DTL-008 落实 |
| **备选** | C++ 核心（否决：要求三引擎统一 C++，违反三引擎异构前提），纯 C 核心（否决：内存安全不可控），WASM 核心（否决：移动端性能开销） |
| **引用** | RGS-ADR-0023 客户端核心逻辑单一实现；RGS-REQ-012 三引擎接入 |

### 3.13.2 引擎适配：Unity (C#) / Unreal (C++) / Godot (GDScript 或 C#)

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】三引擎薄适配，**不**重写核心逻辑 |
| **理由** | 适配层仅做"调用 C ABI + 引擎特定事件循环绑定 + 引擎特定资源管理"；行为一致由 VF-015 三引擎一致性重放验证 |
| **引用** | RGS-REQ-012 SDK；RGS-BAS-007 §3 适配层 |

### 3.13.3 客户端网络：QUIC via quinn（共享服务端运行时）

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】客户端 Rust 核心使用 quinn（与服务端同一 crate 版本） |
| **理由** | 同 crate 同步演进；保证协议实现一致性 |
| **备选** | 客户端单独用 msquic（否决：与服务端 C ABI 边界增加复杂度） |

---

## 3.14 持续集成／持续部署

### 3.14.1 CI：GitHub Actions

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】GitHub Actions（自托管 runner + GitHub-hosted runner 混合） |
| **理由** | 仓库已托管于 GitHub（origin = github.com/UlyssesLeoLee/RustGameServer）；与其他 CI 工具（GitLab CI / Jenkins）相比运维成本最低 |
| **备选** | Jenkins（否决：自维护 CI master 实例 OLU 成本），Drone（否决：与 K8s 集成路径复杂），CircleCI（否决：商业 SaaS） |
| **引用** | RGS-BAS-009 §4 CI 校验脚本；RGS-OPS-001 §3 CI/CD |

### 3.14.2 质量门禁工具链

| 工具 | 作用 | 强制级别 |
|---|---|---|
| `cargo test --workspace` | 全部测试 | PR 阻断 |
| `cargo clippy --all-targets -- -D warnings` | Lint | PR 阻断 |
| `cargo fmt --check` | 格式 | PR 阻断 |
| `cargo deny check` | 许可证 + 重复依赖 + 漏洞 | PR 阻断 |
| `cargo audit` | RustSec 公告 | PR 阻断 |
| `cargo tarpaulin --workspace --out Html` | 覆盖率（QA-001 ≥ 80%） | PR 阻断 |
| `proptest` | 属性测试（不变量验证） | 状态机模块 PR 阻断 |
| `criterion` | 性能回归 | NFR-PE-* 相关 PR 阻断 |
| `cargo outdated` | 依赖陈旧度 | 周报 |

---

## 3.15 性能与负载测试

### 3.15.1 负载生成：k6 0.49+

| 项目 | 内容 |
|---|---|
| **决定** | 【已决】k6 0.49+（Go 实现，CLI + JS 脚本） |
| **理由** | 满足 RGS-REQ-015 测试基础设施；HTTP/WebSocket/gRPC 全协议支持；与 Grafana 集成（k6 cloud 或自部署）；脚本可纳入版本控制 |
| **备选** | JMeter（否决：JVM 资源占用 + GUI 历史包袱），Locust（否决：Python 性能弱于 k6 Go），Gatling（否决：Scala 学习成本 + 商业版），wrk / vegeta（否决：协议支持不全） |
| **引用** | RGS-REQ-015 §3 模拟客户端；RGS-OPS-001 §1.3 k6 |

### 3.15.2 UI 自动化（仅 COC UI 端到端）：Playwright

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】Playwright（仅用于 COC UI 端到端冒烟；游戏客户端 UI 不在范围） |
| **理由** | 跨浏览器（Chromium / Firefox / WebKit）；现代 web 自动化事实标准；与 GitHub Actions 集成良好 |
| **引用** | RGS-REQ-015 §3 |

### 3.15.3 性能 profiling：cargo-flamegraph + perf

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】`cargo flamegraph`（基于 Linux `perf`）；服务端 Linux-only 假设 |
| **理由** | 火焰图直观定位热点；与 Linux perf 集成；CI 中可生成 SVG 附件 |
| **备选** | Instruments（macOS-only，跨平台不友好），DTrace（BSD/Solaris 历史） |

---

## 3.16 安全与密钥

### 3.16.1 密钥管理：HashiCorp Vault（或自托管 OpenBao）

| 项目 | 内容 |
|---|---|
| **决定** | 【待决 TBD-SEC-003】生产环境 HashiCorp Vault 自托管 / 云上 KMS 抽象层；开源替代为 OpenBao（Vault 1.13+ 的 AGPL 分支） |
| **理由** | 满足 RGS-REQ-010 §5.3 密钥轮换不停机（FT-014）；与 K8s Service Account 集成 |
| **备选** | 直接用云 KMS（AWS KMS / GCP KMS）（否决：单一云锁定，与 ARC-033 区域方针冲突），自研密钥管理（否决：违反 ARC-014） |
| **引用** | RGS-REQ-010 §5.3；RGS-ADR-0008 |

### 3.16.2 密码学原语：RustCrypto（*-rs 系列）+ ring

| 项目 | 内容 |
|---|---|
| **决定** | 【一致】RustCrypto crates（`aes-gcm` / `chacha20poly1305` / `ed25519-dalek` / `x25519-dalek` / `sha2` 等）+ `ring`（TLS / 通用加密场景） |
| **理由** | RustCrypto 是 Rust 生态密码学事实标准；ring 是 BoringSSL 的 Rust 绑定（由 Mozilla/Rust 团队维护） |
| **备选** | OpenSSL 绑定（`openssl` crate）（否决：C 依赖、内存安全不可控），商用密码学库（否决：违反 CON-001） |
| **引用** | RGS-REQ-010 §5；RGS-ADR-0020 拒绝动态库加载 |

### 3.16.3 静态代码安全：cargo-audit + cargo-geiger + Clippy 安全 lint

| 工具 | 作用 |
|---|---|
| `cargo audit` | RustSec 已知漏洞库比对 |
| `cargo geiger` | 检测 `unsafe` 使用情况（RustCrypto / ring 等需保留 `unsafe` 边界，比例应有上限） |
| Clippy 安全 lint | `clippy::unwrap_used`、`clippy::expect_used`、`clippy::indexing_slicing` 等 |
| `semgrep`（可选） | 跨语言 SAST，扫 FFI 边界 |

---

# 4. 版本与生命周期政策

## 4.1 版本冻结原则

| 类别 | 策略 | 例外条件 |
|---|---|---|
| Rust 主版本（1.x） | 跟随 stable，1 月与 7 月各升一次 | 紧急安全补丁立即升 |
| 数据库主版本 | 跨 PH 不升；新 PH 开始前评估 | 重大 CVE 立即升 |
| K8s | 跟随上游 4 个 minor 版本（n-3） | 上游支持期结束前必须升 |
| 中间件 | 跟随上游 LTS | 上游 EOL 前 6 个月必须升 |
| 客户端引擎（Unity/UE/Godot） | 跟随各引擎 LTS | 项目实际打包能力 |

## 4.2 升级流程

任何版本升级**必须**：
1. 在 staging 环境完整回归（PR 周期外单独验证）
2. 经 ADR 评审（仅当主版本升级或破坏性变更时）
3. 在 24h 内逐步灰度（开发 → 预发 → 生产）
4. 保留回滚路径（Helm release history / 镜像 tag）

## 4.3 升级 TBD

| ID | 主题 | 状态 |
|---|---|---|
| TBD-VERSION-001 | Rust stable 基线升级窗口（当前核验 1.97.1） | 每次 stable 发布后由 CI 验证 |
| TBD-VERSION-002 | PostgreSQL 18.6 基线的后续 18.x 补丁跟随与 19.x GA 升级窗口（19 当前为 Beta） | PH-2 前建立升级/回退演练 |

---

# 5. 已决 vs 未决选型（TBD 集中列表）

## 5.1 已决选型（分类计数合计 42 项；ADR 引用不重复计数）

| 类别 | 数量 | ADR 引用 |
|---|---|---|
| 主语言/工具链 | 5 | RGS-ADR-0008 / RGS-ADR-0022 / RGS-ADR-0020 |
| 异步/网络 | 4 | RGS-ADR-0008 |
| RPC/序列化 | 3 | RGS-ADR-0008 |
| 数据库 | 2 | RGS-ADR-0007 / RGS-ADR-0008 |
| 缓存 | 1 | RGS-ADR-0008 |
| 事件总线 | 1 | RGS-ADR-0015 |
| 沙箱脚本 | 1 | RGS-ADR-0020 |
| 智能层 | 2 | RGS-ADR-0026 / RGS-ADR-0029 |
| 可观测性 | 4 | RGS-REQ-001 §10 ARC-017 |
| 边缘 | 1 | RGS-ADR-0044 |
| 部署 | 4 | RGS-ADR-0033 / RGS-REQ-027 |
| 对象存储 | 1 | RGS-ADR-0044 |
| 客户端 | 3 | RGS-ADR-0023 |
| CI/CD | 2 | RGS-REQ-001 §12 |
| 性能测试 | 2 | RGS-REQ-015 |
| 安全 | 2 | RGS-REQ-010 |
| 其他（一致性引用） | 4 | RGS-ADR-0025 / RGS-ADR-0029 / RGS-ADR-0051 |

## 5.2 未决选型（共 8 条：6 项组件/工具决策与 2 项升级计划）

| TBD ID | 主题 | 待决时间 | 关联 |
|---|---|---|---|
| TBD-077 | Connect 协议桥（HTTP/1.1 互操作） | 出现外部对接需求时 | §3.3.4 |
| TBD-MNT-002 | CloudNative-PG Operator vs 自建 | PH-2 前 | §3.4.3 |
| TBD-NEURO-002 | LLM 后端选型 vLLM vs TGI | PH-5 前 | §3.8.3 |
| TBD-MEM-001 | 长期记忆向量存储 pgvector vs Milvus（负责人：架构师、DBA、SRE Lead；待附件 D 登记） | PH-3 前 | §3.8.4 / RGS-BAS-033 |
| TBD-LOG-001 | 日志后端 Loki vs 商业 SaaS | PH-3 前 | §3.9.4 |
| TBD-SEC-003 | HashiCorp Vault vs OpenBao vs 云 KMS | PH-4 前 | §3.16.1 |
| TBD-VERSION-001 | Rust 主版本升级窗口 | PH 节点评审 | §4.3 |
| TBD-VERSION-002 | PostgreSQL 主版本升级 | PH-2 前 | §4.3 |

## 5.3 明确否决（表内 12 项）

| 否决项 | 否决理由 | 关联 |
|---|---|---|
| C++ 主语言 | 内存安全弱，违反 ARC-005 | §3.1.1 |
| Go 主语言 | GC 暂停对 20Hz tick 路径不可接受 | §3.1.1 |
| Apache Kafka | JVM 资源开销 + 运维复杂度过高 | §3.6.1 |
| RabbitMQ | 吞吐/分区能力低于 NATS JetStream | §3.6.1 |
| Service Mesh（Istio/Linkerd） | OLU 成本不划算，ARC-014 排除 | §3.10.2 |
| 商业 CDN（Cloudflare/Akamai） | CON-002 商业 SaaS 默认不引入 | §3.10.1 |
| 商业 APM（Datadog/NewRelic） | CON-002 + 资源外送 | §3.9.1 |
| Schema Registry 独立服务 | 存于 admin_db.event_schema_registry 已覆盖 | §3.6.2 |
| TiDB / CockroachDB | 分布式事务对 5 DB 单机架构过剩 | §3.4.1 |
| `dlopen` / cdylib 插件 | RGS-ADR-0020 显式否决 | §3.7.1 |
| WASM 沙箱（wasmtime） | 插件复杂度未到该阶段 | §3.7.1 |
| 自研中间件/库/工具（任何） | 违反 ARC-014 未证明需要不引入 | 全局原则 |

---

# 6. 风险与例外

## 6.1 风险登记

| ID | 风险 | 缓解 |
|---|---|---|
| RSK-TS-001 | MinIO 自 2024-01 切换为 AGPL-3.0——若未来需 fork/二次开发，传染性需评估 | 当前仅客户端调用、读多写少；如出现 fork 需求则单独立 ADR |
| RSK-TS-002 | NATS JetStream 单二进制（Go）——若上游维护节奏放缓，需评估替代（Apache Pulsar / 自研 Outbox 强化） | 每季度 review 上游活跃度；附件 D §4 OSS 许可盘点同步检查 |
| RSK-TS-003 | sqlx 编译期校验要求 CI 缓存 DB schema（`sqlx prepare`）——若 schema 漂移未及时更新，编译期通过但运行时失败 | 已在 RGS-BAS-009 §4 CI 校验中要求 `sqlx prepare --check` 阻断 |
| RSK-TS-004 | tonic 0.10+ 与 hyper 1.x 之间的版本耦合较紧——升级 tonic 时可能需同步升 hyper | 升级前看 tonic release notes；版本升级必须经 ADR 评审 |
| RSK-TS-005 | LangGraph 与 Python 生态强绑定——若 Python 子系统出现性能/内存瓶颈，回退成本高 | 智能层 L4 默认只读感知（ARC-030 闸门），无法直接写 L0/L1；性能瓶颈仅影响 L4 自身输出频率 |

## 6.2 OLU 影响

> **v0.6 双轨制**（per user decision 2026-08-21）：人·天/周 + token/周 **两种算法都要**，不是 token 取代人·天。**算法选择原则**：
> - 纯人类开发 → §6.2.1 人·天/周算法
> - AI 协作开发 → §6.2.2 token/周算法
> - 混合开发 → 按主导模式选 + 双轨并报（不允许混算）
>
> ARC-014 / ARC-026 双门禁（"未证明需要不引入" + "新组件必核算"）保持，按所选算法核算 OLU。

### 6.2.1 人·天/周算法（v0.4 算法，**active**）

> **适用**：纯人类开发场景 / SRE Lead + PM 工时核算 / HR 编制申请 / 工时审计

#### §6.2.1.1 已决选型人·天/周估算（粗算，**待校准**）

按 RGS-ADR-0025 公式：

| 类别 | 组件数 | 估算 OLU（人·天/周）|
|---|---|---|
| 基础设施（K8s/Helm/MinIO/Redis/PG/NATS） | 6 | 8 |
| 可观测性（OTel/Prometheus/Grafana/Loki） | 4 | 3 |
| CI/CD（GitHub Actions + 工具链） | 1 套 | 2 |
| 安全（Vault/OpenBao + crypto） | 2 | 2 |
| 客户端适配（Unity/UE/Godot） | 3 | 3 |
| 测试工具（k6/Playwright） | 2 | 1 |
| **合计** | | **19 人·天/周 ≈ 4 SRE 等效全职** |

#### §6.2.1.2 5 域独立 Lead × 14-18 周 人·天/周估算（per DEC-005 + DEC-006，**待校准**）

> 5 域 Lead 各配独立 AI 协作流 = 5 个独立上下文窗口 / 决策路径 / 代码审查 / 测试生成（AI 协作下对应 token 算法主导；人·天是参考）。

| 域 | 周窗口 | 单域人·天/周（基线）| 单域人·天 总 | 备注 |
|---|---:|---:|---:|---|
| Player 域 | 14-18 周 | ~3-5 | ~42-90 | entry point + 简单 KV 域；体量较小 |
| Economy 域 | 14-18 周 | ~5-8 | ~70-144 | Q-003 Saga 跨域核心 + 事务补偿；体量最大 |
| Match 域 | 14-18 周 | ~4-6 | ~56-108 | NFR-PT 100ms 性能敏感 + 状态机 |
| Social 域 | 14-18 周 | ~3-5 | ~42-90 | 异步消息 + 跨域引用 |
| Admin / COC 域 | 14-18 周 | ~4-6 | ~56-108 | 控制面 + ARC-051 Feature/CEM/PFAU |
| **5 域 Lead 合计** | **14-18 周** | **~19-30** | **~266-540** | **区间估算，待校准** |
| 平均周 | — | — | **~19-30/周** | 5 域 Lead 同时工作 |

> **NFR-OP-010 人·天基线 = 2 SRE ≤ 20 人·天/周**：5 域独立 Lead 配置下人·天周均 **~19-30 = 接近或超限**。与 token 算法对比，**人·天是 NFR-OP-010 的硬约束基础**（HR 编制 / 工时审计用）；token 是 AI 协作下的算力度量（AI 算力预算 / 决策质量评估用）。

### 6.2.2 token/周算法（v0.5 新增算法，**active**）

> **适用**：AI 协作开发场景 / AI 算力预算 / 决策质量评估

#### §6.2.2.1 token-人天换算公式（基线，**待校准**）

| 人类单位 | token 等价（基线） | 区间 | 备注 |
|---|---|---|---|
| 1 人·小时 | ~15K-50K tokens | 12K-60K | 1 小时人类产出 ≈ 15K-50K tokens AI 协作产出 |
| 1 人·天（8 小时） | ~100K-300K tokens | 80K-400K | 1 工作日 ≈ 100K-300K tokens（含输入 + 输出 + 决策对话 + 验证往返）|
| 1 人·周（5 工作日） | ~500K-1.5M tokens | 400K-2M | 1 工作周 ≈ 500K-1.5M tokens |
| 1 SRE 等效全职 | ~1M tokens/周 | 800K-2M | 按每周 1 SRE 全工作量计算 |

> **假设**：单次 AI 协作会话平均 5-20 轮；每轮平均 1K-5K tokens 输入 + 0.5K-3K tokens 输出。**该假设待真实会话样本校准**。

#### §6.2.2.2 已决选型 token 估算（粗算，**待校准**）

| 类别 | 组件数 | v0.4 估算（人·天/周） | v0.6 估算（**tokens/周**）| 备注 |
|---|---:|---:|---:|---|
| 基础设施（K8s/Helm/MinIO/Redis/PG/NATS） | 6 | 8 | ~8M-12M | 配置 / 故障定位 / 升级决策 |
| 可观测性（OTel/Prometheus/Grafana/Loki） | 4 | 3 | ~3M-5M | 仪表盘 / 告警规则 / OTel 集成 |
| CI/CD（GitHub Actions + 工具链） | 1 套 | 2 | ~2M-3M | CI 配置 / cache 策略 / 质量门禁 |
| 安全（Vault/OpenBao + crypto） | 2 | 2 | ~2M-3M | 密钥轮换 / 漏洞修复 / 升级评审 |
| 客户端适配（Unity/UE/Godot） | 3 | 3 | ~3M-5M | C ABI 集成 / 引擎 API 适配 |
| 测试工具（k6/Playwright） | 2 | 1 | ~1M-2M | 负载脚本 / E2E 编写 |
| **合计** | | **19 人·天/周 ≈ 4 SRE** | **~19M-30M tokens/周** | **区间估算，待校准** |

#### §6.2.2.3 5 域独立 Lead × 14-18 周 token 估算（per DEC-005 + DEC-006）

> 5 域 Lead 各配独立 AI 协作流（5 个独立上下文窗口 / 决策路径 / 代码审查 / 测试生成）。

| 域 | 周窗口 | 单域 token/周（基线） | 单域 token 总 | 备注 |
|---|---:|---:|---:|---|
| Player 域 | 14-18 周 | ~2M-4M | ~28M-72M | entry point + 简单 KV 域；体量较小 |
| Economy 域 | 14-18 周 | ~4M-8M | ~56M-144M | Q-003 Saga 跨域核心 + 事务补偿；体量最大 |
| Match 域 | 14-18 周 | ~3M-5M | ~42M-90M | NFR-PT 100ms 性能敏感 + 状态机 |
| Social 域 | 14-18 周 | ~2M-4M | ~28M-72M | 异步消息 + 跨域引用 |
| Admin / COC 域 | 14-18 周 | ~3M-5M | ~42M-90M | 控制面 + ARC-051 Feature/CEM/PFAU |
| **5 域 Lead 合计** | **14-18 周** | **~14M-26M** | **~196M-468M** | **区间估算，待校准** |
| 平均周 | — | — | **~12M-26M/周** | 5 域 Lead 同时工作 |

### 6.2.3 双轨对比与切换条件

| 维度 | 人·天/周算法（v0.4）| token/周算法（v0.5）|
|---|---|---|
| 度量对象 | 人类工时 | AI 算力开销 |
| 主要受众 | SRE Lead / PM / HR | 架构师 / AI 协作设计 |
| 主要决策 | HR 编制 / 工时审计 / 排期 | AI 算力预算 / 上下文窗口 / 决策质量 |
| 适用开发模式 | 纯人类 | AI 协作 |
| 单位 | 人·天（每人 5 天/周）| token（1 SRE ≈ 1M tokens/周）|
| 5 域 Lead 估算（14-18 周）| ~266-540 人·天（周均 ~19-30）| ~196M-468M tokens（周均 ~12M-26M）|
| 优势 | 直观 / 易审计 / HR 可读 | AI 协作下精度高 / 与上下文窗口对齐 |
| 劣势 | AI 协作下精度差 | 不直接对应 HR 编制 / 需校准 |
| 切换条件 | 开发模式偏纯人类 | 开发模式偏 AI 协作 |
| 不允许 | 同一项目内混算 | 同一项目内混算 |

> **双轨并报原则**：混合开发模式（部分域纯人类 / 部分域 AI 协作）下，**按域拆分双轨并报**。每个域须明确"人·天主导"或"token 主导"。

### 6.2.4 NFR-OP-010 双轨

> **v0.6 双轨重定义**：NFR-OP-010 = "2 SRE 团队 ≤ 20" 同时具备两条独立约束：

| 约束维度 | 人·天/周（v0.4）| token/周（v0.5） |
|---|---|---|
| 2 SRE 团队上限 | 20 人·天/周 | **~20M tokens/周**（按 1M tokens/人·周换算）|
| 5 域独立 Lead 编制 | ~19-30 人·天/周（**接近或超限**）| **~12M-26M tokens/周** |
| 上下文窗口约束 | 不适用 | **单会话 200K-1M tokens** |
| 单次决策质量 | 不适用 | **多轮对话 5-20 轮 / 决策** |
| 综合判断 | **5 域独立 Lead 配置下人·天周均 ~19-30 = 接近或超 20 上限**（DEC-005 必然突破人·天硬约束）| **token 周均 ~12M-26M 是 v0.4 已决选型 token 周均（~19M-30M）的 0.4-1.4 倍**（8-12 周窗口下是 1.5-3.5 倍；DEC-006 路径 B 拉长到 14-18 周已显著降低周均 token 压力）|

### 6.2.5 校准与回算路径

> **v0.6 不做实质校准**——具体数字依赖实际工作样本（人·天） / AI 协作会话（token）。
>
> **校准路径（双轨）**：
> 1. **PH-0.5**（开发前授权评审）前：SRE Lead + PM 提供 5 域独立 Lead 真实工作样本（每域 1-2 周 **人·天 + token** 双轨记录）—— **RGS-ENV-CALIB-001 校准记录模板**
> 2. **PH-1**（工程基础）期间：每域 Lead 跑 workspace / testkit / CI 基线；采集首周双轨实际值
> 3. **PH-3**（控制面）前：SRE Lead 校准 §6.2.1.2 人·天 + §6.2.2.3 token 区间；如任一算法偏差 > 30%，升 v0.7
> 4. **PH-7**（发布 Gate）前：双轨实际值 vs 估算偏差写入 CIR 闭环；按主导算法出最终 OLU 报告
>
> **配套文档**：校准执行用 [RGS-ENV-CALIB-001](../00-基准与治理/reviews/RGS-ENV-CALIB-001_OLU校准记录模板_v0.1.md) 模板；WBS 任务分解（5 域 Lead L4 任务清单）用 [RGS-WBS-001](../12-工作流/RGS-WBS-001_瀑布式工作分解结构_v0.3.md)

### 6.2.6 AI 协作开发 vs 纯人类开发 OLU 对比（参考）

| 维度 | 纯人类开发（v0.4 基线）| AI 辅助开发（v0.5 估算）| 差异说明 |
|---|---|---|---|
| 编码速度 | ~50-200 行/天（含思考 / 调试 / 会议）| ~500-2000 行/天（含 AI 生成 + 人类 review）| AI 协作下 5-10× |
| 上下文切换 | 5-10 次/天（会议 / IM / 工单）| 1-3 次/天（专注 AI 协作会话）| AI 协作下 3-5× ↓ |
| 决策等待 | 异步（IM / 会议 / 审批）| 同步（AI 提议 + 人类决策）| AI 协作下决策时延 ↓ |
| 知识检索 | 文档 / Stack Overflow / 代码搜索 | AI 内联检索 + 文档 | AI 协作下 ↓ |
| **OLU 单位（双轨）** | 人·天/周 | **token/周** | **双轨并报** |

> **注意**：AI 协作下"开发速度"非线性增长（5-10×），但**token 单位不等同于工作量**——决策质量、审查深度、边界约束仍是人类工作。token-OLU 仅度量 AI 协作的算力开销，**不替代**人审签字、单元测试、设计评审等质性工作。

## 6.3 例外条款

> 本文档**不替代**任何 ADR 的单点决定。如本文档与某 ADR 冲突，**以 ADR 为准**；本文档的修订**不修改** ADR。如需修改选型，先修改 ADR，再更新本文档对应行。

---

## 7. 工具链 Bug 登记（per RGS-REV-009 + phase-0-5 反馈单 Issue 2，2026-08-24 加）

> **范围**：本节登记**本仓库内脚本/工具链**的 bug，**不**登记外部依赖（Rust 编译器、PostgreSQL、K3s 等）的 bug——后者在各自上游 issue tracker。
>
> **登记原则**：① 现象 + 重现步骤 ② 根因（已知/未知）+ workaround ③ 严重度 ④ 修责任人 ⑤ 关联 ADR/SOP/INC。

### 7.1 BUG-001（2026-08-24 登记，per phase-0-5 反馈单 Issue 2）

| 项目 | 内容 |
|---|---|
| **ID** | BUG-001 |
| **登记日** | 2026-08-24 |
| **登记人** | worker-self（per feedback-handler worktree `phase-0-5/feedback-handler` commit 见行末）|
| **严重度** | 🔴 P0（4/4 worker 0 产出，WF-1-55.27/28/29 P0 merge-blocker 长期挂起）|
| **状态** | **OPEN**（根因待 mavis 维护方定位；workaround 已落地）|
| **现象** | mavis 框架 worker 自动派发路径（`session send` → worktree 创建）4/4 session 在创建后立即被 mavis 标 error，错误字符串 `"Canonical history target is missing after migration"`。受影响 worktree：4/4（HIT count = 100%）。|
| **重现步骤** | 1. 主对话 `mavis` 命令发起 `session send`（含 worktree 派发指令）<br>2. worktree 创建后 session 内 `git rev-parse HEAD` 返回 main tip（与 mavis 期望的「在分支 base 上」不符）<br>3. mavis 内部 state 触发检查失败 → session 立即 error |
| **根因（已知）** | mavis worktree 派发层 internal state（per session tree migration）检查失败，**根因定位需 mavis 维护方**——本仓库 worker agent 无法看到 mavis 源码。可能的根因方向：① worktree 派发时没正确 apply base commit ② session tree migration 步骤未跑 ③ worktree 文件系统状态与 mavis 缓存不一致（per git porcelain 字段）。**不在本节下定论。** |
| **Workaround（已落地）** | 1. **手工 `git worktree add -b <branch> <path> <base>`** 替代 mavis 派发（已在 phase-0-5 4/4 case 验证可用）<br>2. **`scripts/wbs_create_worktree.ps1 -L4Id <id>`** 走项目内脚本（per RGS-WT-001 v0.2 §11.2）<br>3. **主对话直接处理**（不派发 worker，Ulysses 一人公司兼任 per DEC-008）<br>4. 任何需要 `mavis session send` + worktree 派发的任务，**先**在主对话 manual sanity check 1 次，确认 `git worktree list` 输出符合预期**再**派发 |
| **关联** | `docs/deploy/phase-0-5-feedback-to-agents.md` §2 Issue 2<br>`docs/deploy/phase-0-5-handoff.md` §11.3（已记录「4/4 session 0 产出」事件）<br>RGS-REV-009 13 issue 共识矩阵（WF-1-55.27/28/29 长期挂起） |
| **修责任人** | mavis 维护方（**不**是本项目）；workaround 维护 = Ulysses |
| **修复路径** | 待 mavis 维护方提供根因 + fix；修复后本节状态 → CLOSED。**Phase 1+ 启动前必须 close**（否则 Phase 1+ 派发仍 0 产出） |
| **未做项** | ❌ 不在本项目内尝试 mavis 代码 patch（不在 scope）<br>❌ 不在本节登记外部依赖 bug（per §7 范围） |

### 7.2 后续登记模板（per BUG-001 案例）

新增 bug 登记须按 BUG-001 表格 9 字段填齐（ID / 登记日 / 登记人 / 严重度 / 状态 / 现象 / 重现步骤 / 根因 / workaround / 关联 / 修责任人 / 修复路径），缺 1 字段视为登记不合规。

**登记触发条件**（任一即登记）：
- 4 任务同 session 派发 0 产出（per BUG-001）
- `wbs_*.ps1` 脚本 throw 后主对话手工兜底 ≥ 1 次
- handoff §11「已知未完成事项」出现「worker 失败」字样 ≥ 1 次
- `git worktree list --porcelain` 出现 `prunable` 标记

**不登记触发条件**（避免噪音）：
- 业务代码 bug（→ RGS-INC-* incident）
- 文档引用错位（→ handoff §11 修订）
- 决策变更（→ RGS-DEC-* decision）

---

> 配套文档：
> - RGS-ADR-0001～0054（18 份单点决定）
> - RGS-REQ-001（总需求）
> - RGS-OPS-001（部署运维说明）
> - 附件 D §3 决议登记 / §4 OSS 许可 / §5 OLU 台账
>
> 下次评审：本报告随主版本升级窗口同步更新（每年 1 月与 7 月）。
