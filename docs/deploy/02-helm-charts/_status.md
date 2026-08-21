# 02-helm-charts 状态

> **🔴 NO-GO 占位**（生成时间：2026-08-21）

## Chart 状态

| Chart | version | appVersion | 状态 | 责任人 | 激活条件 |
|---|---|---|---|---|---|
| `rust-game-server` (umbrella) | 0.0.0 | PLACEHOLDER | 占位 | 待 SRE 具名 | NO-GO 解除后升 0.1.0 |
| `player` | 0.0.0 | PLACEHOLDER | 占位 | 待 player 域 Lead 具名 | NO-GO 解除后升 0.1.0 |
| `economy` | 0.0.0 | PLACEHOLDER | 占位 | 待 economy 域 Lead 具名（Q-003 独立决策权） | NO-GO 解除后升 0.1.0 |
| `match` | 0.0.0 | PLACEHOLDER | 占位 | 待 match 域 Lead 具名 | NO-GO 解除后升 0.1.0 |
| `social` | 0.0.0 | PLACEHOLDER | 占位 | 待 social 域 Lead 具名 | NO-GO 解除后升 0.1.0 |
| `admin` (COC) | 0.0.0 | PLACEHOLDER | 占位 | 待 admin 域 Lead 具名 | NO-GO 解除后升 0.1.0 |
| `cluster-ops` | 0.0.0 | PLACEHOLDER | 占位 | 待 SRE 具名 + ADR-0052 复核 | NO-GO 解除后升 0.1.0 |

## 状态变更条件

🔴 → 🟡：7 G-CODE 全部 Closed + 12 类签字栏全部具名签字
🟡 → 🟢：`helm install --dry-run` 通过 + 5 域 Lead 联合校准 + 架构师审批

## 责任人占位

- 架构师：Ulysses（已实际签，per RGS-EXEC-001 §2.4）
- SRE：待具名（per RGS-EXEC-001 v0.3 §4.4 所有者背书）
- 5 域 Lead：待具名（per DEC-005 独立配置，不兼任）
- Platform 架构师：待具名（per RGS-EXEC-001 v0.3 §5 所有者背书）
