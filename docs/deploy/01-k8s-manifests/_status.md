# 01-k8s-manifests 状态

> **🔴 NO-GO 占位**（生成时间：2026-08-21）
>
> 任何对本目录下文件的实际内容替换（镜像 tag、副本数、resources、env、Secret values）**必须等 `../07-no-go-checklist_v0.2.md` 全部 ✅ 后**，由 SRE + 5 域 Lead 联合发起，架构师审批。

## 当前清单

| 文件 | 状态 | 责任人 | 实际值写入时间 |
|---|---|---|---|
| `00-namespace.yaml` | 占位 | 待 SRE + DBA 具名 | NO-GO 解除后 |
| `01-player-service.yaml` | 占位 | 待 player 域 Lead 具名 | NO-GO 解除后 |
| `02-economy-service.yaml` | 占位 | 待 economy 域 Lead 具名 | NO-GO 解除后 |
| `03-match-service.yaml` | 占位 | 待 match 域 Lead 具名 | NO-GO 解除后 |
| `04-social-service.yaml` | 占位 | 待 social 域 Lead 具名 | NO-GO 解除后 |
| `05-admin-service.yaml` | 占位 | 待 admin 域 Lead 具名 | NO-GO 解除后 |
| `06-cluster-ops-service.yaml` | 占位 | 待 SRE 具名 + ADR-0052 复核 | NO-GO 解除后 |
| `07-shared-platform.yaml` | 占位 | 待 Platform 架构师具名 | NO-GO 解除后 |
| `08-configmap-template.yaml` | 占位 | 待 5 域 Lead 联合校准 | NO-GO 解除后 |
| `09-secret-template.yaml` | 占位 | 待 SRE 具名（Secret values 单独加密仓管理） | NO-GO 解除后 |
| `10-rbac-template.yaml` | 占位 | 待 SRE + Platform 联合签字 | NO-GO 解除后 |

## 状态变更条件

🔴 → 🟡：7 G-CODE 全部 Closed（per `RGS-EXEC-001 v0.3`）
🟡 → 🟢：RGS-ENV-001 v0.3 §6 12 类签字栏全部具名签字 + RGS-REV-003 §7.3 12 类签字栏全部具名签字

## 责任人占位

- 架构师：Ulysses（已实际签，per RGS-EXEC-001 §2.4 / §3.4 / §4.4）
- SRE：待具名（per RGS-EXEC-001 v0.3 §4.4 所有者背书）
- DBA：待具名（per RGS-EXEC-001 v0.3 §3.4 所有者背书）
- Platform 架构师：待具名（per RGS-EXEC-001 v0.3 §5 所有者背书）
- QA Lead：待具名（per RGS-EXEC-001 v0.3 §6 所有者背书）
- 5 域 Lead：待具名（per DEC-005 独立配置，不兼任）
- 业务方代表：待具名（per RGS-EXEC-001 v0.3 §7 所有者背书）
