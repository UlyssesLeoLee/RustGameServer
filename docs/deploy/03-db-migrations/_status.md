# 03-db-migrations 状态

> **🔴 NO-GO 占位**（生成时间：2026-08-21）

## DB 状态

| DB | 域 | 状态 | 责任人 | 实际 schema 写入时间 |
|---|---|---|---|---|
| `player_db` | player | 占位 | 待 player 域 Lead + DBA 联合 | NO-GO 解除后 |
| `economy_db` | economy | 占位 | 待 economy 域 Lead + DBA 联合（Q-003 Saga 核心） | NO-GO 解除后 |
| `match_db` | match | 占位 | 待 match 域 Lead + DBA 联合 | NO-GO 解除后 |
| `social_db` | social | 占位 | 待 social 域 Lead + DBA 联合 | NO-GO 解除后 |
| `admin_db` | admin | 占位 | 待 admin 域 Lead + DBA 联合 | NO-GO 解除后 |
| `cluster_ops_db` | cluster-ops | 占位 | 待 SRE + DBA 联合（per ADR-0052 PFAU 历史） | NO-GO 解除后 |

## 迁移文件状态

| 文件 | 状态 | 实际 DDL 写入时间 |
|---|---|---|
| `player_db/0001_initial_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `economy_db/0001_initial_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `economy_db/0002_q003_saga_state_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `match_db/0001_initial_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `social_db/0001_initial_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `admin_db/0001_initial_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `admin_db/0002_coc_audit_log_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `cluster_ops_db/0001_initial_placeholder.sql` | 仅注释 | NO-GO 解除后 |
| `cluster_ops_db/0002_pfau_history_placeholder.sql` | 仅注释 | NO-GO 解除后 |

## 状态变更条件

🔴 → 🟡：7 G-CODE 全部 Closed + 12 类签字栏全部具名签字
🟡 → 🟢：DBA + 5 域 Lead 完成 schema 设计 + 迁移工具选型 + staging 验证通过

## 责任人占位

- 架构师：Ulysses（已实际签，per RGS-EXEC-001 §2.4）
- DBA：待具名（per RGS-EXEC-001 v0.2 §3.4 所有者背书）
- SRE：待具名（per RGS-EXEC-001 v0.2 §4.4 所有者背书）
- 5 域 Lead：待具名（per DEC-005 独立配置，不兼任）
