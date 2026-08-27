# 系统测试设计书（工具集 / System Test Design Document）

**目录 09 工具集  系统测试（ST）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-09 |
| 版本 | 0.1 |
| 父文档 | RGS-IMPL-001 实施约定 §4 / RGS-SPEC-000 详细设计规格总表 §2.1 |
| 适用范围 | 验证 rgs-certgen 在真实 CI/CD + k3s 部署链中的端到端可用性 |
| V 模型层级 | TL-6 性能 / TL-7 异常注入 / TL-8 端到端 |
| 编制标准 | IPA 共通框架 2013(SLCP-JCF2013)详细设计工程 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签） |
| 编制日期 | 2026-08-28 06:54 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-IMPL-001 §4, RGS-IMPL-005 BUILD 镜像规范, RGS-REQ-007 §2.1 |
| 关联测试代码 | `scripts/cert-openssl-verify.sh`（**TBD**） |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-28 06:54 JST | 初次编制 |

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

TL-6/7/8 层级设计书,验证 rgs-certgen 在生产链中的端到端可用性:

- CI 镜像能 build 并 spawn rgs-certgen
- 生成的证书能直接被 k3s tls secret 消费
- 5 域 service mTLS 启动时能读这些证书(已验证 0.1.0-cluster-ops)
- 性能 / 异常 / 端到端三层验证

## 1.2 适用范围

| 边界 | 说明 |
|---|---|
| 包含 | CI 镜像 build + 5 域 service 真实启动消费 + 端到端 mTLS 验证 |
| 排除 | 单元测试（UT-09）、集成测试 in-process（IT-09） |

## 1.3 关联文档

per RGS-TST-IT-09 §1.3。

---

## 2. 测试策略

## 2.1 V 模型对应

```
rgs-certgen main.rs
  → UT-09 单测
  → IT-09 集成
  → ST-09(本文件) 部署链
```

## 2.2 阶段归属

| 阶段 | 范围 | rgs-certgen 归属 |
|---|---|---|
| PH-1 | CS SDK | 不适用 |
| PH-2 | GW/RT/SY | 部分(rgs-certgen 是 GW 启动前置) |
| PH-4 | 运维/CAP | **主战场**(证书生成) |
| PH-7 | DEP/CAP T2/T3 | k8s tls secret 集成 |

## 2.3 验收标准映射

| AC | 判定 | 对应 ST 用例 |
|---|---|---|
| AC-002 失败外部支持 | rgs-certgen 在 macOS / Linux / Windows 都能跑 | TST-ST-09-A001 |
| AC-013 15min | 证书生成 ≤ 30s(子流程) | TST-ST-09-F001 |
| AC-015 OSI 100% | 100% | TST-ST-09-A001 |

---

## 3. 测试用例

## 3.1 模块 A：跨平台 CI 镜像 build

| 测试 ID | 对应需求 | 字段/平台 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-09-A001 | IMPL-001 §4 + IMPL-005 | GitHub Actions ubuntu-latest | [TL-8] | N | rgs-certgen build + run 成功 |
| TST-ST-09-A002 | IMPL-005 | macOS runner | [TL-8] | N | **TBD**（macOS runner 可选）|
| TST-ST-09-A003 | IMPL-005 | Windows runner | [TL-8] | N | **TBD**（Windows cert 路径不同）|

**实现位置**：`.github/workflows/rust-ci.yml` + `rgs-certgen` build step（**TBD**）

## 3.2 模块 B：5 域 service mTLS 消费证书

| 测试 ID | 对应需求 | 字段/k8s | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-09-B001 | REQ-007 §2.1 + k3s 部署 | rgs-certgen 生成 → k8s tls secret | [TL-8] | N | 5 域 service mTLS 启动成功 |
| TST-ST-09-B002 | REQ-007 §2.1 | `RGS_TLS_DIR=/etc/rgs/certs` | [TL-8] | N | 5 域读证书成功 |
| TST-ST-09-B003 | REQ-007 §2.1 | 证书过期时 mTLS fail-closed | [TL-8] | A | service 启动失败 + 明确日志 |

**实现位置**：`scripts/cert-deploy-e2e.sh`（**TBD**）

## 3.3 模块 C：FT 容错

| 测试 ID | 对应需求 | 字段/异常 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-09-C001 | FT-001 | output 目录不存在 | [TL-7] | A | create_dir_all 创建 |
| TST-ST-09-C002 | FT-002 | output 目录不可写 | [TL-7] | A | anyhow::Error 退出 1 |
| TST-ST-09-C003 | FT-003 | domains 为空 | [TL-7] | A | **TBD** 行为（main.rs 默认值兜底 or clap 拒绝）|

**实现位置**：`scripts/cert-ft.sh`（**TBD**）

## 3.4 模块 D：性能（v0.2 TBD）

| 测试 ID | 对应需求 | 字段/性能 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-09-D001 | NFR-PE-013 | 6 域 cert 生成 < 5s | [TL-6] | P | **TBD** hyperfine bench |

---

## 4. 追溯矩阵

| 测试 ID | RGS-IMPL | RGS-SPEC | 工具 / 脚本 |
|---|---|---|---|
| TST-ST-09-A001~A003 | §4 + IMPL-005 | §2.1 | `.github/workflows/rust-ci.yml` |
| TST-ST-09-B001~B003 | REQ-007 §2.1 | §2.1 | `scripts/cert-deploy-e2e.sh` |
| TST-ST-09-C001~C003 | FT-001~003 | §2.1 | `scripts/cert-ft.sh` |
| TST-ST-09-D001 | NFR-PE-013 | §2.1 | `hyperfine` |

**总计**：10 测试用例 ID（**全部 TBD**）

---

## 5. 测试执行计划

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L8 CI | GitHub Actions | push to main | 必跑 |
| L8 本地 | shell | `scripts/cert-deploy-e2e.sh` | 部署前手跑 |
| L7 FT | shell | `scripts/cert-ft.sh` | PH-4 / PH-7 |
| L6 性能 | hyperfine | `hyperfine 'cargo run -p rgs-certgen --release'` | v0.2 |

---

## 6. 通过判定标准

| 维度 | 通过阈值 |
|---|---|
| 跨平台 build | 100% |
| k3s mTLS 消费 | 100% |
| FT 容错 | 100%（目录不存在 / 不可写 / 空 domains）|
| 性能 | 6 域 cert 生成 < 5s |

---

## 7. 风险与未决事项

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| TBD-09-01 | 零系统测试代码 | P1 | 实施 ST-09 A/B/C/D |
| TBD-09-02 | Windows 平台 PEM 路径不同 | P2 | IMPL-005 补 Windows 路径处理 |
| TBD-09-03 | cert 过期自动轮换未实装（dev 用 365 天）| P2 | 配合 cert-manager / 定时任务 |
| TBD-09-04 | 跨 cluster 证书复用（per ARC-052 Active-Active）未实装 | P3 | 后续 31 addendum 跟进 |

**保留派生约束**（per 2026-08-26 04:30 JST）：同 UT-09 §7。

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**时间**：2026-08-28 06:54 JST
