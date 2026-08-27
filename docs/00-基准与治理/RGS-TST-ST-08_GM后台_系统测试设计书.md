# 系统测试设计书（GM 后台 / System Test Design Document）

**目录 08 GM 后台  系统测试（ST）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-TST-ST-08 |
| 版本 | 0.1 |
| 父文档 | RGS-REQ-007 运维与 GM 后台管控 / RGS-REQ-019 智能决策层 / RGS-REQ-024 GM 后台漏斗 |
| V 模型层级 | TL-6 性能 / TL-7 异常注入 / TL-8 端到端 → REQ 验收 |
| 编制标准 | IPA 共通框架 2013(SLCP-JCF2013)详细设计工程 / RGS-REQ-001 §12 |
| 编制者 | 架构师（Mavis 接手 agent per DEC-008,代签） |
| 编制日期 | 2026-08-27 23:40 JST |
| 密级 | 内部限定(Internal Use Only) |
| 许可证 | Apache-2.0(本仓库) |
| 关联源代码文档 | RGS-REQ-007, RGS-REQ-019, RGS-REQ-024, RGS-BAS-003, RGS-BAS-021, RGS-DTL-003, RGS-DTL-040 |
| 关联测试代码 | `scripts/e2e-smoke.ps1`（k3s 端到端） + gm-backend k3s pod 19/19 Running |

---

## 修订历史

| 版本 | 修订者 | 修订日期 | 修订内容 |
|---|---|---|---|
| 0.1 | 架构师（Mavis 接手 agent per DEC-008,代签） | 2026-08-27 23:40 JST | 初次编制：8 域第 8 域 GM 后台系统测试设计书（补全 7 域→8 域覆盖缺口） |

## 签字栏

| 角色 | 署名 | 签字日期 | 备注 |
|---|---|---|---|
| 编制（兼签）| 架构师 | 2026-08-27 | per DEC-008 一人公司 12 角色兼任 |
| 需求（架构师）| | | DDD Review 阶段补 |
| 设计 QA 员 | | | 待具名（per Q2 OPEN-QA） |
| 变更控制委员会 | | | DDD Review 阶段补 |

## 目录

1. 前言（Preface）
   1.1 目的（Purpose）
   1.2 适用范围（Scope）
   1.3 关联文档（Related Documents）
   1.4 术语与标记规则（Notation Rules）
   1.5 字段级映射说明
   1.6 命名约定（Naming Convention）
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

本文件为 V 模型 **TL-6 性能 / TL-7 异常注入 / TL-8 端到端**层级设计书，对应 RGS-REQ-007（运维与 GM 后台管控需求定义书）/ RGS-REQ-019（智能决策层需求定义书）/ RGS-REQ-024（GM 后台多人可观测化漏斗需求定义书）。本版本为 0.1 初次编制（per 2026-08-27 23:35 JST Ulysses 指令）。

- 验证系统**端到端**满足 RGS-REQ-007 §3-§5 全部 GM 后台业务需求
- 验证 RGS-REQ-019 §3 智能决策层无埋点可观测性增强需求
- 验证 RGS-REQ-024 §3 GM 后台漏斗可观测性增强
- 验证 NFR Lv.2/3/4：性能、可靠性、可观测性
- 验证 AC-001~019 全部验收标准的 ST 层支撑
- 验证 k3s 部署层 pod 1/1 Running + service 探活 + e2e-smoke 12/12 PASS

## 1.2 适用范围（Scope）

| 边界 | 说明 |
|---|---|
| 包含 | k3s 部署层 gm-backend pod 1/1 Running（19 Pod 中第 8 域）+ 端口探活 + 端到端调用 |
| 排除 | 单元测试（见 RGS-TST-UT-08）、集成测试 in-process（见 RGS-TST-IT-08）、admin-service gRPC v0.2 实装 |
| 当前状态 | 19/19 Pods Running（含 gm-backend-5bf87b565-6tvzt 1/1 Running @ 10.42.0.90,image ghcr.io/ulyssesleolee/rustgameserver:0.1.0-gm-backend）|

## 1.3 关联文档（Related Documents）

| 文档编号 | 文档名 | 与本文件关系 |
|---|---|---|
| RGS-REQ-007 运维与 GM 后台管控 需求定义书 | 需求 | 来源 |
| RGS-REQ-019 智能决策层（无埋点可观测性增强）需求定义书 | 需求 | 观测字段 |
| RGS-REQ-024 GM 后台多人可观测化漏斗 需求定义书 | 需求 | 漏斗 |
| RGS-BAS-003 运维与 GM 后台管控 基本设计书 | 设计 | 父文档 |
| RGS-BAS-021 GM 后台多人可观测化漏斗 基本设计书 | 设计 | 父文档 |
| RGS-DTL-003 运维与 GM 后台管控 详细设计书 | 详细设计 | 父文档 |
| RGS-DTL-040 Admin 域 详细设计书 | 详细设计 | 父文档 |
| RGS-TST-ST-01 核心架构与设计模式 系统测试设计书 | 参考 | V 模型对应 |
| RGS-TST-ST-00 基准与治理 系统测试设计书 | 参考 | V 模型对应 |
| RGS-OPEN-QA-2026-08-27-k3s-deploy v0.1 | OPEN-QA | 决策项 |
| RGS-OLU-REPORT-2026-08-27_dev-k3s-deploy v0.1 | OLU 报告 | 资源估算 |
| `scripts/e2e-smoke.ps1` + `e2e-smoke.sh` | k3s 端到端探活 | 实际工具 |
| `crates/gm-backend/Dockerfile` (distroless cc-debian12) | 镜像 | 运行时 |

## 1.4 术语与标记规则（Notation Rules）

### 1.4.1 强约束标记（RFC 2119 / IPA 共通框架 2013）

| 中文 | 英文 | 强约束度 |
|---|---|---|
| **必须** / 必 | MUST | 强制 |
| **应当** / 应 | SHOULD | 强推荐 |
| **不得** / 禁 | MUST NOT | 强制 |
| **可** / 许 | MAY | 可选 |

### 1.4.2 优先级

| 标记 | 含义 | 处理 |
|---|---|---|
| P0 | 紧急 | 当前阶段必须实现 |
| P1 | 强推荐 | 当前阶段应实现 |
| P2 | 推荐 | 中后期补 |
| P3 | 范围外 | 留待下期 |

### 1.4.3 标识符体系

- `RGS-TST-{UT|IT|ST}-XX-NNN`：测试设计书
- `RGS-TST-{UT|IT|ST}-XX-NNN-AAA`：测试用例
- `RGS-{REQ|BAS|DTL}-NNN`：核心文档
- `RGS-ADR-NNNN`：架构决策记录
- `NFR-<类>-NNN`：非功能需求
- `AC-NNN` / `VF-NNN` / `FT-NNN`：验收 / 验证 / 容错用例

## 1.5 字段级映射说明

本版本为 0.1 初次编制，**强调字段级映射**：每个 ST 测试用例"对应需求"列精确到"REQ-XXX §X.Y + 验收标准 AC-NNN"。

**V 模型强对应**：本文件对应"GM 后台 APIGW 系统层 + 端到端 + 验收"。

## 1.6 命名约定（Naming Convention）

- 测试 ID：`TST-{UT|IT|ST}-08-NNN`
- V 模型层级标注：ST 标 [TL-6/7/8/E2E]
- 用例类型：N=正常 / A=异常 / B=边界 / P=性能 / S=状态机
- 测试运行时：`scripts/e2e-smoke.ps1`（k3s 端到端）

---

## 2. 测试策略

## 2.1 V 模型对应关系

```
需求   RGS-REQ-007/019/024  → ST  (RGS-TST-ST-08,本文件)
设计   RGS-BAS-003/021       → IT  (RGS-TST-IT-08)
详细   RGS-DTL-003/040       → UT  (RGS-TST-UT-08)
实现   Rust 源码 + k3s 部署  ←
```

## 2.2 阶段归属

| 阶段 | 范围 | gm-backend 归属 |
|---|---|---|
| PH-1 | 001-020 (CS SDK) | 不适用 |
| PH-2 | 021-060 (GW/RT/SY) | 部分归属（GM APIGW 是 GW 的一部分） |
| PH-3 | 061-090 (PL/EC/战放) | 不适用 |
| PH-4 | 091-110 (运维/CAP) | **GM 后台主战场** |
| PH-5 | 111-120 (EV/WF) | 漏斗可观测性 |
| PH-6 | 121-140 (PPL) | 性能验证 |
| PH-7 | 141-150 (DEP/CAP T2/T3) | k3s 部署验证 |
| PH-8 | 151-200 (100k + FT) | FT-001~010 容错 |

## 2.3 验收标准映射

| AC | 判定 | 对应 ST 用例 |
|---|---|---|
| AC-001 一直可服务 | 7 阶段稳定运行 | TST-ST-08-101 |
| AC-002 失败外部支持 | 成功 + 失败回归 | TST-ST-08-122, 142 |
| AC-003 运营 | 概率同步 | TST-ST-08-072 |
| AC-004 全面禁止迁移 | 100% 拒绝 | TST-ST-08-130 |
| AC-005 100k CCU | NFR-PE-001~006 全过 | TST-ST-08-121 |
| AC-006 80% 覆盖 | ≥ 80% | TST-ST-08-122 |
| AC-007 3x 容错 | 杀/网络/分区 | TST-ST-08-123 |
| AC-008 FT-001~010 | 10/10 | TST-ST-08-131~140 |
| AC-009 FT-004 损失=0 | 0 | TST-ST-08-134 |
| AC-010 重复/高频 1 处 | 1 | TST-ST-08-140, 181, 182 |
| AC-011 观测 0 盲点 | 0 | TST-ST-08-109 |
| AC-012 trace 贯通 | 6 ID | TST-ST-08-106 |
| AC-013 15min | ≤ 15 | TST-ST-08-111 |
| AC-014 ≤ 2 SRE | ≤ 2.0 | TST-ST-08-113 |
| AC-015 OSI 100% | 100% | TST-ST-08-123 |
| AC-016 TBD 盲点 | 0 新增 | 见 §7 |
| AC-017 ARC-014 | 100% | TST-ST-08-178 |
| AC-018 ≥ 三家客户 | 100% | TST-ST-08-272 |
| AC-019 生态完整 | 100% | 全部 ST |

---

## 3. 测试用例

## 3.1 模块 A：gm-backend 部署验证（k3s 端到端）

| 测试 ID | 对应需求 | 字段/部署 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-A001 | REQ-007 §3.1 + 部署报告 | k3s pod gm-backend-5bf87b565-* 1/1 Running | [TL-8] | N | 19/19 Pods Running，含 gm-backend 1 副本 |
| TST-ST-08-A002 | REQ-007 §3.1 + REQ-019 §3 | image: ghcr.io/ulyssesleolee/rustgameserver:0.1.0-gm-backend | [TL-8] | N | 镜像源 = ghcr.io 0.1.0-gm-backend |
| TST-ST-08-A003 | REQ-007 §3.1 + 端口分配 | Service ClusterIP 8443/8081/9464 | [TL-8] | N | Service 暴露 3 端口 |
| TST-ST-08-A004 | REQ-007 §3.1 + 探针 | readiness/liveness: httpGet /healthz /readyz port=health (8081) | [TL-8] | N | 探针配置正确（非 exec grpc_health_probe） |
| TST-ST-08-A005 | REQ-007 §3.1 + 资源限制 | limits: 500m / 256Mi | [TL-8] | N | resource limits 设置 |
| TST-ST-08-A006 | REQ-007 §3.1 + 安全 | securityContext runAsNonRoot=65532, readOnlyRootFilesystem=true | [TL-8] | N | 容器以 nonroot 运行 + rootfs 只读 |

**实现位置**：`scripts/e2e-smoke.ps1`（12 探活 + gm-backend 1 探活） + `kubectl get pod -l app.kubernetes.io/name=gm-backend`（**已 19/19 PASS** per 2026-08-27 19:45 JST）

## 3.2 模块 B：端口可达性 / AC-013 15min / 探活

| 测试 ID | 对应需求 | 字段/探活 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-B001 | AC-013 + e2e-smoke | 12 service 端口 nc -z | [TL-8] | N | 12/12 PASS（player 50051, economy 50052, match 50053, social 50054, admin 50055, cluster-ops 50056, gm-backend 8081, postgres 5432, prometheus 9090, grafana 3000, otel-collector 4317, nats 4222） |
| TST-ST-08-B002 | REQ-007 §3.1 | gm-backend /healthz via port-forward | [TL-8] | N | 返回 `{"status":"ok","service":"gm-backend"}` |
| TST-ST-08-B003 | REQ-007 §3.1 | gm-backend /readyz via port-forward | [TL-8] | N | 返回 `{"status":"ready","service":"gm-backend"}` |

**实现位置**：`scripts/e2e-smoke.ps1`（**已 12/12 PASS** per 2026-08-27 19:45 JST）

## 3.3 模块 C：v0.1 占位（5 endpoint 仍 stub，v0.2 实装）

| 测试 ID | 对应需求 | 字段/响应 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-C001 | BAS-003 §3.1 health_view | k8s 端到端 GET | [TL-8] | N | **TBD v0.2**:返回 admin-service 健康状态 |
| TST-ST-08-C002 | BAS-003 §3.4 ban_account | k8s 端到端 POST | [TL-8] | N | **TBD v0.2**:调 admin-service.BanAccount 实际封号 |
| TST-ST-08-C003 | BAS-003 §3.4 grant_compensation | k8s 端到端 POST | [TL-8] | N | **TBD v0.2**:调 admin-service.GrantCompensation 实际补偿 |
| TST-ST-08-C004 | BAS-003 §3.4 set_maintenance | k8s 端到端 POST | [TL-8] | N | **TBD v0.2**:调 admin-service.SetMaintenance 实际维护 |
| TST-ST-08-C005 | BAS-021 §3 audit 漏斗 | k8s 端到端 GET | [TL-8] | N | **TBD v0.2**:返回 ≥ 1 items + trace_id 贯通 |

**实现位置**：v0.2 实装（per OPEN-QA Q6 GM 后台代签边界 + TBD-08-03）

## 3.4 模块 D：可观测性 / AC-011 0 盲点 / AC-012 trace 贯通

| 测试 ID | 对应需求 | 字段/span | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-D001 | REQ-019 §3 无埋点可观测 | OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317 | [TL-8] | N | env 注入正确 |
| TST-ST-08-D002 | REQ-019 §3 OTEL_SERVICE_NAME | gm-backend span | [TL-8] | N | span service.name="gm-backend" |
| TST-ST-08-D003 | AC-012 trace 贯通 | gm-backend span → otel-collector | [TL-8] | N | trace 透传到 otel-collector |
| TST-ST-08-D004 | REQ-024 §3 漏斗 | gm-backend 5 endpoint span | [TL-8] | N | **TBD v0.2**:每个 endpoint 独立 span |

**实现位置**：OTEL 接入代码 + otel-collector 验证（commit 0.1.0-gm-backend 镜像已含 OTEL env，**实际贯通需 v0.2**）

## 3.5 模块 E：异常注入 / FT-001~010 容错

| 测试 ID | 对应需求 | 字段/异常 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-E001 | FT-001 k8s pod 杀重启 | kill gm-backend pod | [TL-7/8] | A | **TBD** k8s 拉新 pod 1/1 Running（其他 18 不受影响） |
| TST-ST-08-E002 | FT-004 数据=0 | 模拟 admin-service 故障 | [TL-7/8] | A | **TBD v0.2** gm-backend 返回 502 + 0 业务数据损坏 |
| TST-ST-08-E003 | AC-007 3x 容错 | 杀 + 网络 + 分区 | [TL-7/8] | A | **TBD** 三种异常下系统恢复 |

**实现位置**：`k3s` + `kubectl delete pod`（**当前未跑 FT 验证**）

## 3.6 模块 F：性能 / NFR-PE-001~006（v0.2 TBD）

| 测试 ID | 对应需求 | 字段/性能 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-F001 | NFR-PE-001 P99 < 100ms | 5 endpoint P99 | [TL-6] | P | **TBD** GM 操作 P99 < 100ms |
| TST-ST-08-F002 | NFR-PE-002 100k CCU | 100k 5 endpoint 吞吐 | [TL-6] | P | **TBD** 100k RPS |
| TST-ST-08-F003 | NFR-PE-003 health P99 < 10ms | /healthz P99 | [TL-6] | P | **TBD** |

**实现位置**：`scripts/loadtest/`（**未实现**）

## 3.7 模块 G：v0.2 TLS / mTLS（v0.2 TBD）

| 测试 ID | 对应需求 | 字段/证书 | V 层级 | 用例类型 | 测试目标 |
|---|---|---|---|---|---|
| TST-ST-08-G001 | BAS-003 §2.1 HTTPS 8443 | rustls server.pem + ca.pem | [TL-8] | N | **TBD v0.2** |
| TST-ST-08-G002 | BAS-003 §2.1 mTLS 拒绝 | 客户端无证书 | [TL-8] | A | **TBD v0.2** |

---

## 4. 追溯矩阵（Traceability Matrix）

| 测试 ID | RGS-REQ | RGS-BAS | RGS-DTL | 测试代码 / 工具 |
|---|---|---|---|---|
| TST-ST-08-A001~A006 | REQ-007 §3.1 | BAS-003 §3.1 | DTL-040 §3.2 | `e2e-smoke.ps1` + `kubectl get pod` |
| TST-ST-08-B001 | AC-013 | BAS-003 §3.1 | DTL-040 §3.2 | `e2e-smoke.ps1`（12 探活）|
| TST-ST-08-B002~B003 | REQ-007 §3.1 | BAS-003 §3.4 | DTL-040 §3.3 | `e2e-smoke.ps1`（gm-backend port-forward + curl）|
| TST-ST-08-C001~C005 | REQ-007 §3.4 / REQ-024 §3 | BAS-003 §3.4 / BAS-021 §3 | DTL-040 §3.3 | **TBD v0.2** |
| TST-ST-08-D001~D004 | REQ-019 §3 / REQ-024 §3 | BAS-019 §3 / BAS-021 §3 | DTL-040 §3.4 | k8s pod env + otel-collector |
| TST-ST-08-E001~E003 | FT-001~010 | BAS-003 §6 | DTL-040 §5 | k3s pod 操作 |
| TST-ST-08-F001~F003 | NFR-PE-001~006 | BAS-003 §5 | DTL-040 §4 | `scripts/loadtest/` |
| TST-ST-08-G001~G002 | REQ-007 §2.1 | BAS-003 §2.1 | DTL-040 §3.6 | v0.2 实装 |

**总计**：26 测试用例 ID（6 部署 + 3 端口 + 5 stub + 4 可观测 + 3 FT + 3 性能 + 2 TLS）——per §3.1-§3.7 实际 ID 区间逐条求和 = 6+3+5+4+3+3+2 = 26（与本文 §3.3-§3.7 9 PASS + 17 TBD = 26 一致）

**当前已通过**：TST-ST-08-A001~A006（k3s 部署） + TST-ST-08-B001~B003（端口 + /healthz + /readyz） = **9 测试 PASS**
**待 v0.2 实装**：5 + 4 + 3 + 3 + 2 = 17 测试

---

## 5. 测试执行计划（Test Execution Plan）

| 阶段 | 工具 | 命令 | 触发 |
|---|---|---|---|
| L8 本地 dev | bash | `pwsh scripts/e2e-smoke.ps1` | 每次 commit / 部署后 |
| L8 CI | bash | `.github/workflows/rust-ci.yml`（待集成 gm-backend） | push to main |
| L6 性能 | k6 / wrk | **TBD** `scripts/loadtest/` | v0.2 / 100k CCU 阶段 |
| L7 异常 | chaos-mesh / kubectl | **TBD** | v0.2 |
| L7 mTLS | openssl | **TBD v0.2** | v0.2 |

**当前状态（2026-08-27 23:40 JST）**：
- L8 端到端 9/9 PASS（`scripts/e2e-smoke.ps1` 19 Pod + 12 端口 + gm-backend /healthz）
- L7/L6 暂未跑（v0.2 阶段）

---

## 6. 通过判定标准（Pass Criteria）

| 维度 | 通过阈值 | 当前状态 |
|---|---|---|
| 部署层 PASS | gm-backend 1/1 Running | ✅ 1/1 Running |
| 端口可达 PASS | 12/12 端口 nc -z | ✅ 12/12 PASS（19:45 JST） |
| /healthz PASS | 返回 service=gm-backend | ✅ |
| /readyz PASS | 返回 status=ready | ✅ |
| AC-013 15min | ≤ 15 | **TBD** |
| AC-006 80% 覆盖 | ≥ 80% | **TBD** |
| AC-007 3x 容错 | 杀/网络/分区 恢复 | **TBD** |
| AC-011 0 盲点 | 0 观测盲点 | **TBD** |
| AC-012 trace 贯通 | 6 ID 贯通 | **TBD** |
| AC-014 ≤ 2 SRE | ≤ 2.0 | OLU 报告 §6.5: token 轨余量充足,人·天 21 略超 20,需 SRE Lead 决策 |

---

## 7. 风险与未决事项（Risks and TBDs）

| 编号 | 描述 | 风险等级 | 解决路径 |
|---|---|---|---|
| TBD-08-01 | JWT validation 未实装（`jwt_secret` 字段保留 `#[allow(dead_code)]`） | P1 | v0.2 |
| TBD-08-02 | mTLS 启动 fail-closed 路径未实装 | P1 | v0.2 |
| TBD-08-03 | 5 GM endpoint 仍 stub（202 queued / 空 items）| P2 | v0.2 |
| TBD-08-04 | audit join admin-service.QueryAudit 未实装 | P2 | v0.2 |
| TBD-08-05 | GM 后台域 Lead 未具名（per OPEN-QA Q2）| P1 | DDD Review |
| TBD-08-06 | 性能 / NFR 测试（k6 / wrk）未跑 | P1 | v0.2 / 100k 阶段 |
| TBD-08-07 | 异常注入（chaos-mesh）未跑 | P2 | v0.2 |
| TBD-08-08 | mTLS 8443 端到端未验 | P2 | v0.2 |
| TBD-08-09 | OLU 报告 §6.5:人·天 21 略超 NFR-OP-010 20 | P1 | SRE Lead + PM Lead 联合决策 |
| TBD-08-10 | OUTBOX relay 切到 NATS（per OPEN-QA Q5,5 域 service 需 `kubectl rollout restart`）| P1 | 5 域 Lead 联合决策 |

**保留派生约束**（per 2026-08-26 04:30 JST）：
- 禁"per X 历史形态"等回溯叙事
- 引用 BAS 必须 git log -p --follow 实证
- 缺标比错标安全
- 子代理授权边界要写明"无证据叙事 = 禁止"

---

**作者**：架构师（Mavis 接手 agent per DEC-008,代签）  
**时间**：2026-08-27 23:40 JST  
**后续**：DDD Review 时由 Ulysses + SRE Lead + GM 后台域 Lead 联合审
