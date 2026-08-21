# 04-PostgreSQL 18.6 + 5 DB 划分

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DEPLOY-00-04 |
| 版本 | 0.1（占位 + 文档化）|
| 依据 | RGS-TS-001 v0.6 §3.4 PG + ARC-008 5 DB 划分 + RGS-IMPL-001 §3 + RGS-PLAN-001 v0.8 §3.3 G-CODE-06 |
| 状态 | **🟠 NO-GO 状态** |
| 保密级别 | 内部限定（Internal Use Only）|

---

## §1 PostgreSQL 18.6 基线

| 项 | 目标 | 当前 | 责任方 |
|---|---|---|---|
| PostgreSQL | 18.6 | 待实测 | DBA Lead |
| psql | 18.6 | 待实测 | DBA Lead |
| libpq | 与 psql 一致 | 待实测 | DBA Lead |

## §2 5 DB 独立划分（per ARC-008）

> **核心原则**：5 域各自独立 DB，**禁止跨 DB JOIN**（per ARC-008 5 DB 划分 + RGS-IMPL-001 §3）。

| DB | 责任域 | 责任方 | 命名空间建议 |
|---|---|---|---|
| `player_db` | Player 域 | Player 域 Lead | `rgs_player` |
| `economy_db` | Economy 域 | Economy 域 Lead | `rgs_economy` |
| `match_db` | Match 域 | Match 域 Lead | `rgs_match` |
| `social_db` | Social 域 | Social 域 Lead | `rgs_social` |
| `admin_db` | Admin / COC 域 | Admin 域 Lead | `rgs_admin` |

## §3 凭证管理

| DB | 凭证存储 | 轮换策略 |
|---|---|---|
| 5 DB | HashiCorp Vault / OpenBao（per RGS-TS-001 v0.6 §3.16）| 每 90 天 |
| sqlx DATABASE_URL | 环境变量 / Vault 注入 | CI: OIDC token；dev: env var |

## §4 sqlx 编译期校验

> **NO-GO 状态下不创建实际 migration**。仅占位目录结构。

```text
crates/rgs-{player,economy,match,social,admin}/
├── migrations/                   # sqlx migration 目录
│   ├── 20260821000001_*.sql
│   └── ...
├── .sqlx/                        # sqlx prepare 输出（编译期一致性）
└── Cargo.toml
```

## §5 5 DB schema 概览（占位 + 待 Q-003 决策）

### §5.1 player_db

- `players` 表（PII 字段加密）
- `player_characters` / `player_inventory` 索引
- 登录态 JWT / session

### §5.2 economy_db

- `accounts` / `account_balance` / `currency_types`
- `transactions`（事务日志 + request_id 幂等键）
- `outbox`（per RGS-IMPL-001 §3 Saga）
- Q-003 跨域决策点：6 场景演练（per RGS-REV-005 附件B）

### §5.3 match_db

- `match_rooms` / `match_players` / `match_states`
- 匹配评分算法（per RGS-DTL-026）
- 性能约束：NFR-PT 100ms 决策

### §5.4 social_db

- `messages` / `message_recipients` / `conversations`
- 通知渠道 4 选 1（站内信 / 邮件 / 推送 / 短信）
- 内容审核 + 人工升级

### §5.5 admin_db

- `cluster_nodes` / `feature_activations` / `pfa_operations`（per RGS-DTL-031）
- `event_schema_registry`（per RGS-TS-001 v0.6 §3.6.2）
- RBAC 矩阵 + 审计日志

## §6 修订历史

| 版本 | 修订日 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-21 | 架构师（Ulysses）| 初版。PG 18.6 + 5 DB 划分 + sqlx 编译期占位（不实际创建）。 |
