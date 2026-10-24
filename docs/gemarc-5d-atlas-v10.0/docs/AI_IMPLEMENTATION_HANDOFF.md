# AI 実装交接契約 (AI Implementation Handoff Contract)

> **対象**: RustGameServer (RGS)
> **ソース Revision**: `b6a998174582` (branch `dev`)
> **モデル採取**: 2026-10-22T15:11:52.672Z
> **生成**: `scripts/generate-handoff.mjs` (モデルから自動派生)
> **注意**: 各タスクの sourceEvidence はすべて `file:startLine-endLine` + revision を持つ。
> ここに無い事実は本リポジトリから**確認できなかった**という 뜻であり、捏造して補完しないこと。

## 0. 読み方

各タスクは 10 フィールド固定:
`objective` / `requirementIds` / `sourceEvidence` / `targetModules` / `inputOutput` /
`rules` / `failureModes` / `acceptanceTests` / `dependency` / `rollback-ops`


---
## TASK-001 — activity-service

### objective
activity-service 提供 1 个 gRPC 服务 / 30 个方法, 持有 0 张表

### requirementIds
REQ-001

関連 Prompt: `PRM-001` (vv1.0.0)

### sourceEvidence
- `crates/activity-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=activity-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=64
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/activity-service/` (crate `rgs::activity-service` v0.1.0, .rs 5 件, テスト関数 64 件)
  - `activity.v1.ActivityService` — `crates/activity-service/proto/activity/v1/activity.proto:19-71` (30 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: activity-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 64
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/activity-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-002 — admin-service

### objective
admin-service 提供 1 个 gRPC 服务 / 6 个方法, 持有 6 张表

### requirementIds
REQ-002

関連 Prompt: `PRM-002` (vv1.0.0)

### sourceEvidence
- `crates/admin-service/Cargo.toml:1-63` (rev `b6a998174582`, SourceCode) — crate 清单 (name=admin-service, version=0.1.0); src .rs=12, 测试 .rs=0, #[test] 计数=121
- `docs/deploy/01-k8s-manifests/05-admin-service.yaml:27-56` (rev `b6a998174582`, SourceCode) — k8s Deployment admin-service (ns=rust-game-server) replicas=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/admin-service/` (crate `rgs::admin-service` v0.1.0, .rs 12 件, テスト関数 121 件)
  - `admin.v1.AdminService` — `crates/admin-service/proto/admin/v1/admin.proto:10-21` (6 メソッド)
  - テーブル `admin_users` — `crates/admin-service/migrations/0001_init.sql:5-14`
  - テーブル `audit_log` — `crates/admin-service/migrations/0001_init.sql:20-29`
  - テーブル `outbox` — `crates/admin-service/migrations/0003_outbox.sql:16-29`
  - テーブル `lcm_step_execution` — `crates/admin-service/migrations/0005_lcm_step_execution.sql:45-61`
  - テーブル `audit_log_partitioned` — `crates/admin-service/migrations/0006_audit_log_partitioned.sql:41-51`
  - テーブル `second_review` — `crates/admin-service/migrations/0007_second_review.sql:42-61`
- 配備: `docs/deploy/01-k8s-manifests/05-admin-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: admin-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **FailurePropagation** (CASE-006, DesignedSimulation): FAILURE — 熔断生效, 防止 cascade failure — gm-backend::AdminGrpcClient 的 CircuitBreaker 状态机: Closed → (连续 5 次失败) → Open (30000ms) → HalfOpen (仅 1 个 probe)。Open 期间 
- **Recovery** (CASE-007, DesignedSimulation): SUCCESS — 熔断器复位 — Open 超时后进入 HalfOpen, probe 成功则 record_success → Closed, consecutive_failures 归零。

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 6 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 121
- [ ] k8s Deployment 已定义
- [ ] Case `CASE-006` (FailurePropagation) を再現可能にすること
- [ ] Case `CASE-007` (Recovery) を再現可能にすること
- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/admin-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/05-admin-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-003 — batch-service

### objective
batch-service 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-003

関連 Prompt: `PRM-003` (vv1.0.0)

### sourceEvidence
- `crates/batch-service/Cargo.toml:1-36` (rev `b6a998174582`, SourceCode) — crate 清单 (name=batch-service, version=0.1.0); src .rs=1, 测试 .rs=0, #[test] 计数=0
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/batch-service/` (crate `rgs::batch-service` v0.1.0, .rs 1 件, テスト関数 0 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: batch-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 0
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/batch-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-004 — battle-service

### objective
battle-service 提供 12 个 gRPC 服务 / 250 个方法, 持有 3 张表

### requirementIds
REQ-004

関連 Prompt: `PRM-004` (vv1.0.0)

### sourceEvidence
- `crates/battle-service/Cargo.toml:1-46` (rev `b6a998174582`, SourceCode) — crate 清单 (name=battle-service, version=0.1.0); src .rs=7, 测试 .rs=0, #[test] 计数=33
- `docs/deploy/01-k8s-manifests/51-battle-service.yaml:2-31` (rev `b6a998174582`, SourceCode) — k8s Deployment battle-service (ns=rust-game-server) replicas=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/battle-service/` (crate `rgs::battle-service` v0.1.0, .rs 7 件, テスト関数 33 件)
  - `battle.v1.BattleEngineService` — `crates/battle-service/proto/battle/v1/battle.proto:176-217` (32 メソッド)
  - `battle.v1.PvPService` — `crates/battle-service/proto/battle/v1/battle.proto:223-262` (30 メソッド)
  - `battle.v1.BossService` — `crates/battle-service/proto/battle/v1/battle.proto:268-288` (15 メソッド)
  - `battle.v1.RoomService` — `crates/battle-service/proto/battle/v1/battle.proto:294-347` (46 メソッド)
  - `battle.v1.InstanceService` — `crates/battle-service/proto/battle/v1/battle.proto:353-361` (6 メソッド)
  - `battle.v1.EndlessTowerService` — `crates/battle-service/proto/battle/v1/battle.proto:367-382` (13 メソッド)
  - `battle.v1.EscortService` — `crates/battle-service/proto/battle/v1/battle.proto:388-408` (18 メソッド)
  - `battle.v1.HolyEquipService` — `crates/battle-service/proto/battle/v1/battle.proto:414-445` (24 メソッド)
  - `battle.v1.GuildWarService` — `crates/battle-service/proto/battle/v1/battle.proto:451-470` (17 メソッド)
  - `battle.v1.CrossServerService` — `crates/battle-service/proto/battle/v1/battle.proto:476-497` (19 メソッド)
  - `battle.v1.ExpeditionService` — `crates/battle-service/proto/battle/v1/battle.proto:503-520` (15 メソッド)
  - `battle.v1.HolidayActivityService` — `crates/battle-service/proto/battle/v1/battle.proto:526-551` (15 メソッド)
  - テーブル `battle_master` — `crates/battle-service/migrations/0001_init_battle.sql:5-10`
  - テーブル `battle_transaction` — `crates/battle-service/migrations/0001_init_battle.sql:13-20`
  - テーブル `battle_work` — `crates/battle-service/migrations/0001_init_battle.sql:23-28`
- 配備: `docs/deploy/01-k8s-manifests/51-battle-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: battle-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 12 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 3 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 33
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/battle-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/51-battle-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-005 — card-service

### objective
card-service 提供 1 个 gRPC 服务 / 9 个方法, 持有 3 张表

### requirementIds
REQ-005

関連 Prompt: `PRM-005` (vv1.0.0)

### sourceEvidence
- `crates/card-service/Cargo.toml:1-52` (rev `b6a998174582`, SourceCode) — crate 清单 (name=card-service, version=0.1.0); src .rs=8, 测试 .rs=0, #[test] 计数=44
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/card-service/` (crate `rgs::card-service` v0.1.0, .rs 8 件, テスト関数 44 件)
  - `card.v1.CardService` — `crates/card-service/proto/card/v1/card.proto:14-31` (9 メソッド)
  - テーブル `cards` — `crates/card-service/migrations/0001_init.sql:7-20`
  - テーブル `card_series` — `crates/card-service/migrations/0001_init.sql:26-36`
  - テーブル `card_instances` — `crates/card-service/migrations/0001_init.sql:40-50`
- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: card-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **Timeout** (CASE-008, HypotheticalExperiment): FAILURE — 重试耗尽 (设计意图, 非实测) — ⚠️ 该调用对在源码中**仅见于注释** (trade_saga_clients.rs), 无真实调用点, 因此本 Case 为 Hypothetical, 不得当作已实现行为。

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 3 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 44
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown
- [ ] Case `CASE-008` (Timeout) を再現可能にすること
- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/card-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-006 — cluster-ops

### objective
cluster-ops 提供 1 个 gRPC 服务 / 3 个方法, 持有 11 张表

### requirementIds
REQ-006

関連 Prompt: `PRM-006` (vv1.0.0)

### sourceEvidence
- `crates/cluster-ops/Cargo.toml:1-48` (rev `b6a998174582`, SourceCode) — crate 清单 (name=cluster-ops, version=0.1.0); src .rs=36, 测试 .rs=3, #[test] 计数=203
- `docs/deploy/01-k8s-manifests/06-cluster-ops-service.yaml:29-58` (rev `b6a998174582`, SourceCode) — k8s Deployment cluster-ops (ns=rust-game-server) replicas=3
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/cluster-ops/` (crate `rgs::cluster-ops` v0.1.0, .rs 36 件, テスト関数 203 件)
  - `cluster_ops.v1.ClusterOpsService` — `crates/cluster-ops/proto/cluster_ops/v1/cluster_ops.proto:6-13` (3 メソッド)
  - テーブル `cluster_nodes` — `crates/cluster-ops/migrations/0001_init.sql:5-16`
  - テーブル `feature_flags` — `crates/cluster-ops/migrations/0001_init.sql:23-32`
  - テーブル `outbox` — `crates/cluster-ops/migrations/0002_outbox.sql:16-29`
  - テーブル `realm_lifecycle_run` — `crates/cluster-ops/migrations/0020_lcm_tables.sql:31-47`
  - テーブル `new_realm_plan` — `crates/cluster-ops/migrations/0020_lcm_tables.sql:83-93`
  - テーブル `split_plan` — `crates/cluster-ops/migrations/0020_lcm_tables.sql:103-114`
  - テーブル `merge_conflict_rule_set_v2` — `crates/cluster-ops/migrations/0020_lcm_tables.sql:124-137`
  - テーブル `retire_plan` — `crates/cluster-ops/migrations/0020_lcm_tables.sql:151-162`
  - テーブル `archive_policy` — `crates/cluster-ops/migrations/0020_lcm_tables.sql:174-185`
  - テーブル `lcm_step_execution` — `crates/cluster-ops/migrations/0021_lcm_step_execution.sql:39-56`
  - テーブル `function_registry` — `crates/cluster-ops/migrations/0022_function_registry.sql:41-51`
- 配備: `docs/deploy/01-k8s-manifests/06-cluster-ops-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: cluster-ops 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 11 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 203
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/cluster-ops/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/06-cluster-ops-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-007 — economy-service

### objective
economy-service 提供 1 个 gRPC 服务 / 92 个方法, 持有 10 张表

### requirementIds
REQ-007

関連 Prompt: `PRM-007` (vv1.0.0)

### sourceEvidence
- `crates/economy-service/Cargo.toml:1-63` (rev `b6a998174582`, SourceCode) — crate 清单 (name=economy-service, version=0.1.0); src .rs=20, 测试 .rs=0, #[test] 计数=156
- `docs/deploy/01-k8s-manifests/02-economy-service.yaml:27-56` (rev `b6a998174582`, SourceCode) — k8s Deployment economy-service (ns=rust-game-server) replicas=2
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/economy-service/` (crate `rgs::economy-service` v0.1.0, .rs 20 件, テスト関数 156 件)
  - `economy.v1.EconomyService` — `crates/economy-service/proto/economy/v1/economy.proto:24-150` (92 メソッド)
  - テーブル `accounts` — `crates/economy-service/migrations/0001_init.sql:5-15`
  - テーブル `transaction_ledger` — `crates/economy-service/migrations/0001_init.sql:21-33`
  - テーブル `sagas` — `crates/economy-service/migrations/0002_saga_init.sql:4-16`
  - テーブル `reservations` — `crates/economy-service/migrations/0002_saga_init.sql:24-34`
  - テーブル `inbox` — `crates/economy-service/migrations/0002_saga_init.sql:42-50`
  - テーブル `outbox` — `crates/economy-service/migrations/0003_outbox.sql:17-30`
  - テーブル `auctions` — `crates/economy-service/migrations/0005_auctions.sql:32-49`
  - テーブル `private_trades` — `crates/economy-service/migrations/0005_auctions.sql:60-75`
  - テーブル `transaction_ledger_partitioned` — `crates/economy-service/migrations/0006_transaction_ledger_partitioned.sql:41-52`
  - テーブル `sagas_partitioned` — `crates/economy-service/migrations/0007_sagas_partitioned.sql:42-60`
- 配備: `docs/deploy/01-k8s-manifests/02-economy-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: economy-service 的实现变更 / 对应 UT/IT 用例

### rules
- `RULE-SAGA-001` Saga 失败触发补偿: `when state == Failed` → `then SagaOrchestrator::compensate 逆序执行各 handler 的 compensate`

### failureModes
- **Normal** (CASE-001, DesignedSimulation): SUCCESS — saga 提交, 无补偿 — 沿 economy-service 真实 saga 步骤链执行, 每步成功则推进, 最终 confirm。无异常。
- **Compensation** (CASE-002, DesignedSimulation): COMPENSATED — 资源回滚, 与 chaos_trade_saga_compensation.rs 的测试意图一致 — 第 2 步注入失败, 触发 SagaOrchestrator::compensate 逆序补偿。补偿使用真实 compensate 处理器, 不是简单倒放动画。
- **Timeout** (CASE-008, HypotheticalExperiment): FAILURE — 重试耗尽 (设计意图, 非实测) — ⚠️ 该调用对在源码中**仅见于注释** (trade_saga_clients.rs), 无真实调用点, 因此本 Case 为 Hypothetical, 不得当作已实现行为。

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 10 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 156
- [ ] k8s Deployment 已定义
- [ ] Case `CASE-001` (Normal) を再現可能にすること
- [ ] Case `CASE-002` (Compensation) を再現可能にすること
- [ ] Case `CASE-008` (Timeout) を再現可能にすること
- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/economy-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/02-economy-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-008 — function-plane

### objective
function-plane 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-008

関連 Prompt: `PRM-008` (vv1.0.0)

### sourceEvidence
- `crates/function-plane/Cargo.toml:1-31` (rev `b6a998174582`, SourceCode) — crate 清单 (name=function-plane, version=0.1.0); src .rs=6, 测试 .rs=0, #[test] 计数=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/function-plane/` (crate `rgs::function-plane` v0.1.0, .rs 6 件, テスト関数 1 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: function-plane 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 1
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/function-plane/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-009 — gm-backend

### objective
gm-backend 提供 1 个 gRPC 服务 / 5 个方法, 持有 0 张表

### requirementIds
REQ-009

関連 Prompt: `PRM-009` (vv1.0.0)

### sourceEvidence
- `crates/gm-backend/Cargo.toml:1-66` (rev `b6a998174582`, SourceCode) — crate 清单 (name=gm-backend, version=0.1.0); src .rs=15, 测试 .rs=0, #[test] 计数=5
- `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml:19-48` (rev `b6a998174582`, SourceCode) — k8s Deployment gm-backend (ns=rust-game-server) replicas=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/gm-backend/` (crate `rgs::gm-backend` v0.1.0, .rs 15 件, テスト関数 5 件)
  - `gm.v1.GmBackend` — `crates/gm-backend/proto/gm/v1/gm.proto:25-40` (5 メソッド)

- 配備: `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: gm-backend 的实现变更 / 对应 UT/IT 用例

### rules
- `RULE-BREAKER-001` 熔断打开: `when breaker_state == Closed && consecutive_failures >= 5` → `then breaker_state = Open; opened_at = now` (timeout 30000ms)
- `RULE-BREAKER-002` 熔断半开与复位: `when breaker_state == Open && elapsed >= open_duration` → `then breaker_state = HalfOpen; 允许 1 个 probe` (timeout 30000ms)

### failureModes
- **FailurePropagation** (CASE-006, DesignedSimulation): FAILURE — 熔断生效, 防止 cascade failure — gm-backend::AdminGrpcClient 的 CircuitBreaker 状态机: Closed → (连续 5 次失败) → Open (30000ms) → HalfOpen (仅 1 个 probe)。Open 期间 
- **Recovery** (CASE-007, DesignedSimulation): SUCCESS — 熔断器复位 — Open 超时后进入 HalfOpen, probe 成功则 record_success → Closed, consecutive_failures 归零。

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 5
- [ ] k8s Deployment 已定义
- [ ] Case `CASE-006` (FailurePropagation) を再現可能にすること
- [ ] Case `CASE-007` (Recovery) を再現可能にすること
- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/gm-backend/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-010 — gm-extra-service

### objective
gm-extra-service 提供 1 个 gRPC 服务 / 10 个方法, 持有 0 张表

### requirementIds
REQ-010

関連 Prompt: `PRM-010` (vv1.0.0)

### sourceEvidence
- `crates/gm-extra-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=gm-extra-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=18
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/gm-extra-service/` (crate `rgs::gm-extra-service` v0.1.0, .rs 5 件, テスト関数 18 件)
  - `gm_extra.v1.GmExtraService` — `crates/gm-extra-service/proto/gm_extra/v1/gm_extra.proto:11-32` (10 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: gm-extra-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 18
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/gm-extra-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-011 — guild-service

### objective
guild-service 提供 1 个 gRPC 服务 / 30 个方法, 持有 0 张表

### requirementIds
REQ-011

関連 Prompt: `PRM-011` (vv1.0.0)

### sourceEvidence
- `crates/guild-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=guild-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=53
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/guild-service/` (crate `rgs::guild-service` v0.1.0, .rs 5 件, テスト関数 53 件)
  - `guild.v1.GuildService` — `crates/guild-service/proto/guild/v1/guild.proto:12-63` (30 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: guild-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 53
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/guild-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-012 — i18n-service

### objective
i18n-service 提供 1 个 gRPC 服务 / 4 个方法, 持有 2 张表

### requirementIds
REQ-012

関連 Prompt: `PRM-012` (vv1.0.0)

### sourceEvidence
- `crates/i18n-service/Cargo.toml:1-43` (rev `b6a998174582`, SourceCode) — crate 清单 (name=i18n-service, version=0.1.0); src .rs=6, 测试 .rs=0, #[test] 计数=20
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/i18n-service/` (crate `rgs::i18n-service` v0.1.0, .rs 6 件, テスト関数 20 件)
  - `i18n.v1.I18nService` — `crates/i18n-service/proto/i18n/v1/i18n.proto:13-18` (4 メソッド)
  - テーブル `i18n_texts` — `crates/i18n-service/migrations/0001_init.sql:10-17`
  - テーブル `i18n_languages` — `crates/i18n-service/migrations/0001_init.sql:22-29`
- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: i18n-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 2 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 20
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/i18n-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-013 — leaderboard-extra-service

### objective
leaderboard-extra-service 提供 1 个 gRPC 服务 / 7 个方法, 持有 0 张表

### requirementIds
REQ-013

関連 Prompt: `PRM-013` (vv1.0.0)

### sourceEvidence
- `crates/leaderboard-extra-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=leaderboard-extra-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=17
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/leaderboard-extra-service/` (crate `rgs::leaderboard-extra-service` v0.1.0, .rs 5 件, テスト関数 17 件)
  - `leaderboard_extra.v1.LeaderboardExtraService` — `crates/leaderboard-extra-service/proto/leaderboard_extra/v1/leaderboard_extra.proto:10-22` (7 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: leaderboard-extra-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 17
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/leaderboard-extra-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-014 — leaderboard-service

### objective
leaderboard-service 提供 1 个 gRPC 服务 / 6 个方法, 持有 1 张表

### requirementIds
REQ-014

関連 Prompt: `PRM-014` (vv1.0.0)

### sourceEvidence
- `crates/leaderboard-service/Cargo.toml:1-51` (rev `b6a998174582`, SourceCode) — crate 清单 (name=leaderboard-service, version=0.1.0); src .rs=8, 测试 .rs=0, #[test] 计数=15
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/leaderboard-service/` (crate `rgs::leaderboard-service` v0.1.0, .rs 8 件, テスト関数 15 件)
  - `leaderboard.v1.LeaderboardService` — `crates/leaderboard-service/proto/leaderboard/v1/leaderboard.proto:14-32` (6 メソッド)
  - テーブル `leaderboard_entries` — `crates/leaderboard-service/migrations/0001_init.sql:7-24`
- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: leaderboard-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 1 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 15
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/leaderboard-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-015 — match-service

### objective
match-service 提供 1 个 gRPC 服务 / 10 个方法, 持有 8 张表

### requirementIds
REQ-015

関連 Prompt: `PRM-015` (vv1.0.0)

### sourceEvidence
- `crates/match-service/Cargo.toml:1-71` (rev `b6a998174582`, SourceCode) — crate 清单 (name=match-service, version=0.1.0); src .rs=13, 测试 .rs=0, #[test] 计数=120
- `docs/deploy/01-k8s-manifests/03-match-service.yaml:22-51` (rev `b6a998174582`, SourceCode) — k8s Deployment match-service (ns=rust-game-server) replicas=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/match-service/` (crate `rgs::match-service` v0.1.0, .rs 13 件, テスト関数 120 件)
  - `match.v1.MatchService` — `crates/match-service/proto/match/v1/match.proto:6-23` (10 メソッド)
  - テーブル `matches` — `crates/match-service/migrations/0001_init.sql:5-17`
  - テーブル `match_participants` — `crates/match-service/migrations/0001_init.sql:23-35`
  - テーブル `outbox` — `crates/match-service/migrations/0002_outbox.sql:16-29`
  - テーブル `game_sessions` — `crates/match-service/migrations/0040_game_sessions.sql:9-33`
  - テーブル `moves` — `crates/match-service/migrations/0040_game_sessions.sql:42-54`
  - テーブル `matchmaking_tickets` — `crates/match-service/migrations/0040_game_sessions.sql:63-77`
  - テーブル `session_subscriptions` — `crates/match-service/migrations/0040_game_sessions.sql:86-94`
  - テーブル `moves_partitioned` — `crates/match-service/migrations/0041_moves_partitioned.sql:40-52`
- 配備: `docs/deploy/01-k8s-manifests/03-match-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: match-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 8 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 120
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/match-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/03-match-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-016 — network-gateway

### objective
network-gateway 提供 1 个 gRPC 服务 / 4 个方法, 持有 0 张表

### requirementIds
REQ-016

関連 Prompt: `PRM-016` (vv1.0.0)

### sourceEvidence
- `crates/network-gateway/Cargo.toml:1-51` (rev `b6a998174582`, SourceCode) — crate 清单 (name=network-gateway, version=0.1.0); src .rs=18, 测试 .rs=0, #[test] 计数=123
- `docs/deploy/01-k8s-manifests/52-network-gateway.yaml:2-31` (rev `b6a998174582`, SourceCode) — k8s Deployment network-gateway (ns=rust-game-server) replicas=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/network-gateway/` (crate `rgs::network-gateway` v0.1.0, .rs 18 件, テスト関数 123 件)
  - `gateway.v1.GatewayAdmin` — `crates/network-gateway/proto/gateway/v1/gateway.proto:13-26` (4 メソッド)

- 配備: `docs/deploy/01-k8s-manifests/52-network-gateway.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: network-gateway 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 123
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/network-gateway/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/52-network-gateway.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-017 — operate-service

### objective
operate-service 提供 1 个 gRPC 服务 / 7 个方法, 持有 0 张表

### requirementIds
REQ-017

関連 Prompt: `PRM-017` (vv1.0.0)

### sourceEvidence
- `crates/operate-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=operate-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=18
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/operate-service/` (crate `rgs::operate-service` v0.1.0, .rs 5 件, テスト関数 18 件)
  - `operate.v1.OperateService` — `crates/operate-service/proto/operate/v1/operate.proto:10-24` (7 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: operate-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 18
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/operate-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-018 — player-service

### objective
player-service 提供 1 个 gRPC 服务 / 26 个方法, 持有 7 张表

### requirementIds
REQ-018

関連 Prompt: `PRM-018` (vv1.0.0)

### sourceEvidence
- `crates/player-service/Cargo.toml:1-55` (rev `b6a998174582`, SourceCode) — crate 清单 (name=player-service, version=0.1.0); src .rs=8, 测试 .rs=0, #[test] 计数=113
- `docs/deploy/01-k8s-manifests/01-player-service.yaml:28-57` (rev `b6a998174582`, SourceCode) — k8s Deployment player-service (ns=rust-game-server) replicas=2
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/player-service/` (crate `rgs::player-service` v0.1.0, .rs 8 件, テスト関数 113 件)
  - `player.v1.PlayerService` — `crates/player-service/proto/player/v1/player.proto:6-87` (26 メソッド)
  - テーブル `players` — `crates/player-service/migrations/0001_init.sql:5-15`
  - テーブル `player_sessions` — `crates/player-service/migrations/0001_init.sql:22-30`
  - テーブル `outbox` — `crates/player-service/migrations/0002_outbox.sql:16-29`
  - テーブル `player_characters` — `crates/player-service/migrations/0004_player_characters_inventory.sql:51-97`
  - テーブル `player_inventory` — `crates/player-service/migrations/0004_player_characters_inventory.sql:117-151`
  - テーブル `decks` — `crates/player-service/migrations/0005_decks.sql:33-69`
  - テーブル `characters` — `crates/player-service/migrations/0006_characters.sql:42-63`
- 配備: `docs/deploy/01-k8s-manifests/01-player-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: player-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 7 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 113
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/player-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/01-player-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-019 — pvp-full-service

### objective
pvp-full-service 提供 1 个 gRPC 服务 / 30 个方法, 持有 0 张表

### requirementIds
REQ-019

関連 Prompt: `PRM-019` (vv1.0.0)

### sourceEvidence
- `crates/pvp-full-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=pvp-full-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=65
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/pvp-full-service/` (crate `rgs::pvp-full-service` v0.1.0, .rs 5 件, テスト関数 65 件)
  - `pvp_full.v1.PvpFullService` — `crates/pvp-full-service/proto/pvp_full/v1/pvp_full.proto:12-51` (30 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: pvp-full-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 65
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/pvp-full-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-020 — replay-extra-service

### objective
replay-extra-service 提供 1 个 gRPC 服务 / 10 个方法, 持有 0 张表

### requirementIds
REQ-020

関連 Prompt: `PRM-020` (vv1.0.0)

### sourceEvidence
- `crates/replay-extra-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=replay-extra-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=15
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/replay-extra-service/` (crate `rgs::replay-extra-service` v0.1.0, .rs 5 件, テスト関数 15 件)
  - `replay_extra.v1.ReplayExtraService` — `crates/replay-extra-service/proto/replay_extra/v1/replay_extra.proto:11-30` (10 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: replay-extra-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 15
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/replay-extra-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-021 — replay-service

### objective
replay-service 提供 1 个 gRPC 服务 / 9 个方法, 持有 1 张表

### requirementIds
REQ-021

関連 Prompt: `PRM-021` (vv1.0.0)

### sourceEvidence
- `crates/replay-service/Cargo.toml:1-53` (rev `b6a998174582`, SourceCode) — crate 清单 (name=replay-service, version=0.1.0); src .rs=9, 测试 .rs=0, #[test] 计数=36
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/replay-service/` (crate `rgs::replay-service` v0.1.0, .rs 9 件, テスト関数 36 件)
  - `replay.v1.ReplayService` — `crates/replay-service/proto/replay/v1/replay.proto:23-57` (9 メソッド)
  - テーブル `replays` — `crates/replay-service/migrations/0001_init.sql:18-29`
- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: replay-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 1 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 36
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/replay-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-022 — rgs-arc-olu

### objective
rgs-arc-olu 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-022

関連 Prompt: `PRM-022` (vv1.0.0)

### sourceEvidence
- `crates/rgs-arc-olu/Cargo.toml:1-21` (rev `b6a998174582`, SourceCode) — crate 清单 (name=rgs-arc-olu, version=0.1.0); src .rs=1, 测试 .rs=0, #[test] 计数=10
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/rgs-arc-olu/` (crate `rgs::rgs-arc-olu` v0.1.0, .rs 1 件, テスト関数 10 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-arc-olu 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 10
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/rgs-arc-olu/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-023 — rgs-asset-download

### objective
rgs-asset-download 提供 0 个 gRPC 服务 / 0 个方法, 持有 1 张表

### requirementIds
REQ-023

関連 Prompt: `PRM-023` (vv1.0.0)

### sourceEvidence
- `crates/rgs-asset-download/Cargo.toml:1-43` (rev `b6a998174582`, SourceCode) — crate 清单 (name=rgs-asset-download, version=0.1.0); src .rs=16, 测试 .rs=0, #[test] 计数=59
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/rgs-asset-download/` (crate `rgs::rgs-asset-download` v0.1.0, .rs 16 件, テスト関数 59 件)

  - テーブル `resume_tokens` — `crates/rgs-asset-download/migrations/0001_resume_token_index.sql:20-29`
- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-asset-download 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 1 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 59
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/rgs-asset-download/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-024 — rgs-certgen

### objective
rgs-certgen 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-024

関連 Prompt: `PRM-024` (vv1.0.0)

### sourceEvidence
- `crates/rgs-certgen/Cargo.toml:1-32` (rev `b6a998174582`, SourceCode) — crate 清单 (name=rgs-certgen, version=0.1.0); src .rs=1, 测试 .rs=0, #[test] 计数=0
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/rgs-certgen/` (crate `rgs::rgs-certgen` v0.1.0, .rs 1 件, テスト関数 0 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-certgen 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 0
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/rgs-certgen/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-025 — rgs-hello

### objective
rgs-hello 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-025

関連 Prompt: `PRM-025` (vv1.0.0)

### sourceEvidence
- `crates/rgs-hello/Cargo.toml:1-16` (rev `b6a998174582`, SourceCode) — crate 清单 (name=rgs-hello, version=0.1.0); src .rs=1, 测试 .rs=0, #[test] 计数=0
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/rgs-hello/` (crate `rgs::rgs-hello` v0.1.0, .rs 1 件, テスト関数 0 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-hello 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 0
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/rgs-hello/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-026 — rgs-overflow-alert

### objective
rgs-overflow-alert 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-026

関連 Prompt: `PRM-026` (vv1.0.0)

### sourceEvidence
- `crates/rgs-overflow-alert/Cargo.toml:1-40` (rev `b6a998174582`, SourceCode) — crate 清单 (name=rgs-overflow-alert, version=0.1.0); src .rs=8, 测试 .rs=0, #[test] 计数=30
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/rgs-overflow-alert/` (crate `rgs::rgs-overflow-alert` v0.1.0, .rs 8 件, テスト関数 30 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-overflow-alert 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 30
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/rgs-overflow-alert/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-027 — rgs-testkit

### objective
rgs-testkit 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-027

関連 Prompt: `PRM-027` (vv1.0.0)

### sourceEvidence
- `crates/rgs-testkit/Cargo.toml:1-85` (rev `b6a998174582`, SourceCode) — crate 清单 (name=rgs-testkit, version=0.1.0); src .rs=17, 测试 .rs=0, #[test] 计数=84
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-testkit 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 84
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-028 — scene-service

### objective
scene-service 提供 1 个 gRPC 服务 / 145 个方法, 持有 0 张表

### requirementIds
REQ-028

関連 Prompt: `PRM-028` (vv1.0.0)

### sourceEvidence
- `crates/scene-service/Cargo.toml:1-49` (rev `b6a998174582`, SourceCode) — crate 清单 (name=scene-service, version=0.1.0); src .rs=9, 测试 .rs=0, #[test] 计数=68
- `docs/deploy/01-k8s-manifests/50-scene-service.yaml:2-31` (rev `b6a998174582`, SourceCode) — k8s Deployment scene-service (ns=rust-game-server) replicas=1
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/scene-service/` (crate `rgs::scene-service` v0.1.0, .rs 9 件, テスト関数 68 件)
  - `scene.v1.SceneService` — `crates/scene-service/proto/scene/v1/scene.proto:13-185` (145 メソッド)

- 配備: `docs/deploy/01-k8s-manifests/50-scene-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: scene-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 68
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/scene-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/50-scene-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-029 — shared-platform

### objective
shared-platform 提供 0 个 gRPC 服务 / 0 个方法, 持有 0 张表

### requirementIds
REQ-029

関連 Prompt: `PRM-029` (vv1.0.0)

### sourceEvidence
- `crates/shared-platform/Cargo.toml:1-52` (rev `b6a998174582`, SourceCode) — crate 清单 (name=shared-platform, version=0.1.0); src .rs=23, 测试 .rs=0, #[test] 计数=166
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/shared-platform/` (crate `rgs::shared-platform` v0.1.0, .rs 23 件, テスト関数 166 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: shared-platform 的实现变更 / 对应 UT/IT 用例

### rules
- `RULE-RETRY-002` 退避封顶: `when base_ms > max_interval` → `then backoff = max_interval (+jitter)` (timeout 30000ms) (retry max=3, backoff=100ms)

### failureModes
- **Normal** (CASE-001, DesignedSimulation): SUCCESS — saga 提交, 无补偿 — 沿 economy-service 真实 saga 步骤链执行, 每步成功则推进, 最终 confirm。无异常。
- **Retry** (CASE-003, DesignedSimulation): FAILURE — 达到 max_retries=3 后放弃 (退避上限 30000ms) — 按 shared-platform::retry::RetryConfig::default() 实参退避: max_retries=3, initial=100ms, multiplier=2, max=30000ms, jitter=±
- **Boundary** (CASE-004, DesignedSimulation): FAILURE — 退避封顶, 不无限增长 — attempts 足够大时退避被截断到 30000ms。源码 backoff_duration 对 capped_ms 再叠加 ±20% jitter, 故上限为 30000 * 1.2 = 36000ms。

### acceptanceTests
- [ ] 0 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 166
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown
- [ ] Case `CASE-001` (Normal) を再現可能にすること
- [ ] Case `CASE-003` (Retry) を再現可能にすること
- [ ] Case `CASE-004` (Boundary) を再現可能にすること
- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/shared-platform/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-030 — social-extra-service

### objective
social-extra-service 提供 1 个 gRPC 服务 / 15 个方法, 持有 0 张表

### requirementIds
REQ-030

関連 Prompt: `PRM-030` (vv1.0.0)

### sourceEvidence
- `crates/social-extra-service/Cargo.toml:1-35` (rev `b6a998174582`, SourceCode) — crate 清单 (name=social-extra-service, version=0.1.0); src .rs=5, 测试 .rs=0, #[test] 计数=21
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/social-extra-service/` (crate `rgs::social-extra-service` v0.1.0, .rs 5 件, テスト関数 21 件)
  - `social_extra.v1.SocialExtraService` — `crates/social-extra-service/proto/social_extra/v1/social_extra.proto:11-35` (15 メソッド)

- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: social-extra-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 0 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 21
- [ ] ⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/social-extra-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-031 — social-service

### objective
social-service 提供 1 个 gRPC 服务 / 2 个方法, 持有 8 张表

### requirementIds
REQ-031

関連 Prompt: `PRM-031` (vv1.0.0)

### sourceEvidence
- `crates/social-service/Cargo.toml:1-55` (rev `b6a998174582`, SourceCode) — crate 清单 (name=social-service, version=0.1.0); src .rs=9, 测试 .rs=0, #[test] 计数=73
- `docs/deploy/01-k8s-manifests/04-social-service.yaml:22-51` (rev `b6a998174582`, SourceCode) — k8s Deployment social-service (ns=rust-game-server) replicas=2
- `AGENTS.md:14-14` (rev `b6a998174582`, Document) — 业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处

### targetModules
- `crates/social-service/` (crate `rgs::social-service` v0.1.0, .rs 9 件, テスト関数 73 件)
  - `social.v1.SocialService` — `crates/social-service/proto/social/v1/social.proto:6-9` (2 メソッド)
  - テーブル `guilds` — `crates/social-service/migrations/0001_init.sql:5-15`
  - テーブル `guild_members` — `crates/social-service/migrations/0001_init.sql:21-29`
  - テーブル `outbox` — `crates/social-service/migrations/0002_outbox.sql:16-29`
  - テーブル `guild_invitations` — `crates/social-service/migrations/0004_social_work_tables.sql:35-46`
  - テーブル `guild_join_requests` — `crates/social-service/migrations/0004_social_work_tables.sql:58-69`
  - テーブル `guild_applications` — `crates/social-service/migrations/0004_social_work_tables.sql:83-95`
  - テーブル `friend_requests` — `crates/social-service/migrations/0004_social_work_tables.sql:108-118`
  - テーブル `private_messages` — `crates/social-service/migrations/0004_social_work_tables.sql:132-143`
- 配備: `docs/deploy/01-k8s-manifests/04-social-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: social-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 1 个 proto service 定义可解析 (证据含 fileLine)
- [ ] 8 张 CREATE TABLE 具备 PK/FK 解析结果
- [ ] crate 内 #[test] 计数 = 73
- [ ] k8s Deployment 已定义

- 非目標: 不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)
- 非目標: 不断言运行时行为已通过验证 (未执行真实流量)

### dependency
- 上流: `crates/social-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: なし / **TBD**

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/04-social-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-032 — gm-backend → admin-service 呼び出し

### objective
gm-backend → admin-service 跨服务 gRPC 调用 (AdminServiceClient, type-annotation)

### requirementIds
REQ-032

関連 Prompt: `PRM-032` (vv1.0.0)

### sourceEvidence
- `crates/gm-backend/src/lib.rs:687-687` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 AdminServiceClient (caller=gm-backend → callee=admin-service; 形态=type-annotation)

### targetModules
- `crates/gm-backend/` (crate `rgs::gm-backend` v0.1.0, .rs 15 件, テスト関数 5 件)
  - `gm.v1.GmBackend` — `crates/gm-backend/proto/gm/v1/gm.proto:25-40` (5 メソッド)

- 配備: `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: gm-backend 的实现变更 / 对应 UT/IT 用例

### rules
- `RULE-BREAKER-001` 熔断打开: `when breaker_state == Closed && consecutive_failures >= 5` → `then breaker_state = Open; opened_at = now` (timeout 30000ms)
- `RULE-BREAKER-002` 熔断半开与复位: `when breaker_state == Open && elapsed >= open_duration` → `then breaker_state = HalfOpen; 允许 1 个 probe` (timeout 30000ms)

### failureModes
- **FailurePropagation** (CASE-006, DesignedSimulation): FAILURE — 熔断生效, 防止 cascade failure — gm-backend::AdminGrpcClient 的 CircuitBreaker 状态机: Closed → (连续 5 次失败) → Open (30000ms) → HalfOpen (仅 1 个 probe)。Open 期间 
- **Recovery** (CASE-007, DesignedSimulation): SUCCESS — 熔断器复位 — Open 超时后进入 HalfOpen, probe 成功则 record_success → Closed, consecutive_failures 归零。

### acceptanceTests
- [ ] 源码存在真实调用点 crates/gm-backend/src/lib.rs:687
- [ ] Case `CASE-006` (FailurePropagation) を再現可能にすること
- [ ] Case `CASE-007` (Recovery) を再現可能にすること
- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/gm-backend/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/admin-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-033 — gm-backend → admin-service 呼び出し

### objective
gm-backend → admin-service 跨服务 gRPC 调用 (AdminServiceClient, ctor)

### requirementIds
REQ-033

関連 Prompt: `PRM-033` (vv1.0.0)

### sourceEvidence
- `crates/gm-backend/src/lib.rs:698-698` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 AdminServiceClient (caller=gm-backend → callee=admin-service; 形态=ctor)

### targetModules
- `crates/gm-backend/` (crate `rgs::gm-backend` v0.1.0, .rs 15 件, テスト関数 5 件)
  - `gm.v1.GmBackend` — `crates/gm-backend/proto/gm/v1/gm.proto:25-40` (5 メソッド)

- 配備: `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: gm-backend 的实现变更 / 对应 UT/IT 用例

### rules
- `RULE-BREAKER-001` 熔断打开: `when breaker_state == Closed && consecutive_failures >= 5` → `then breaker_state = Open; opened_at = now` (timeout 30000ms)
- `RULE-BREAKER-002` 熔断半开与复位: `when breaker_state == Open && elapsed >= open_duration` → `then breaker_state = HalfOpen; 允许 1 个 probe` (timeout 30000ms)

### failureModes
- **FailurePropagation** (CASE-006, DesignedSimulation): FAILURE — 熔断生效, 防止 cascade failure — gm-backend::AdminGrpcClient 的 CircuitBreaker 状态机: Closed → (连续 5 次失败) → Open (30000ms) → HalfOpen (仅 1 个 probe)。Open 期间 
- **Recovery** (CASE-007, DesignedSimulation): SUCCESS — 熔断器复位 — Open 超时后进入 HalfOpen, probe 成功则 record_success → Closed, consecutive_failures 归零。

### acceptanceTests
- [ ] 源码存在真实调用点 crates/gm-backend/src/lib.rs:698
- [ ] Case `CASE-006` (FailurePropagation) を再現可能にすること
- [ ] Case `CASE-007` (Recovery) を再現可能にすること
- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/gm-backend/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/admin-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/50-gm-backend-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-034 — match-service → replay-service 呼び出し

### objective
match-service → replay-service 跨服务 gRPC 调用 (ReplayServiceClient, type-annotation)

### requirementIds
REQ-034

関連 Prompt: `PRM-034` (vv1.0.0)

### sourceEvidence
- `crates/match-service/src/replay_client.rs:295-295` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 ReplayServiceClient (caller=match-service → callee=replay-service; 形态=type-annotation)

### targetModules
- `crates/match-service/` (crate `rgs::match-service` v0.1.0, .rs 13 件, テスト関数 120 件)
  - `match.v1.MatchService` — `crates/match-service/proto/match/v1/match.proto:6-23` (10 メソッド)
  - テーブル `matches` — `crates/match-service/migrations/0001_init.sql:5-17`
  - テーブル `match_participants` — `crates/match-service/migrations/0001_init.sql:23-35`
  - テーブル `outbox` — `crates/match-service/migrations/0002_outbox.sql:16-29`
  - テーブル `game_sessions` — `crates/match-service/migrations/0040_game_sessions.sql:9-33`
  - テーブル `moves` — `crates/match-service/migrations/0040_game_sessions.sql:42-54`
  - テーブル `matchmaking_tickets` — `crates/match-service/migrations/0040_game_sessions.sql:63-77`
  - テーブル `session_subscriptions` — `crates/match-service/migrations/0040_game_sessions.sql:86-94`
  - テーブル `moves_partitioned` — `crates/match-service/migrations/0041_moves_partitioned.sql:40-52`
- 配備: `docs/deploy/01-k8s-manifests/03-match-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: match-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/match-service/src/replay_client.rs:295

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/match-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/replay-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/03-match-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-035 — match-service → replay-service 呼び出し

### objective
match-service → replay-service 跨服务 gRPC 调用 (ReplayServiceClient, ctor)

### requirementIds
REQ-035

関連 Prompt: `PRM-035` (vv1.0.0)

### sourceEvidence
- `crates/match-service/src/replay_client.rs:338-338` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 ReplayServiceClient (caller=match-service → callee=replay-service; 形态=ctor)

### targetModules
- `crates/match-service/` (crate `rgs::match-service` v0.1.0, .rs 13 件, テスト関数 120 件)
  - `match.v1.MatchService` — `crates/match-service/proto/match/v1/match.proto:6-23` (10 メソッド)
  - テーブル `matches` — `crates/match-service/migrations/0001_init.sql:5-17`
  - テーブル `match_participants` — `crates/match-service/migrations/0001_init.sql:23-35`
  - テーブル `outbox` — `crates/match-service/migrations/0002_outbox.sql:16-29`
  - テーブル `game_sessions` — `crates/match-service/migrations/0040_game_sessions.sql:9-33`
  - テーブル `moves` — `crates/match-service/migrations/0040_game_sessions.sql:42-54`
  - テーブル `matchmaking_tickets` — `crates/match-service/migrations/0040_game_sessions.sql:63-77`
  - テーブル `session_subscriptions` — `crates/match-service/migrations/0040_game_sessions.sql:86-94`
  - テーブル `moves_partitioned` — `crates/match-service/migrations/0041_moves_partitioned.sql:40-52`
- 配備: `docs/deploy/01-k8s-manifests/03-match-service.yaml` (Deployment)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: match-service 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/match-service/src/replay_client.rs:338

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/match-service/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/replay-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: k8s `docs/deploy/01-k8s-manifests/03-match-service.yaml` の Deployment をロールバック
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-036 — rgs-testkit → economy-service 呼び出し

### objective
rgs-testkit → economy-service 跨服务 gRPC 调用 (EconomyServiceClient, type-annotation)

### requirementIds
REQ-036

関連 Prompt: `PRM-036` (vv1.0.0)

### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/economy.rs:211-211` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 EconomyServiceClient (caller=rgs-testkit → callee=economy-service; 形态=type-annotation)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-testkit 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/economy.rs:211

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/economy-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-037 — rgs-testkit → economy-service 呼び出し

### objective
rgs-testkit → economy-service 跨服务 gRPC 调用 (EconomyServiceClient, type-annotation)

### requirementIds
REQ-037

関連 Prompt: `PRM-037` (vv1.0.0)

### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/economy.rs:297-297` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 EconomyServiceClient (caller=rgs-testkit → callee=economy-service; 形态=type-annotation)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-testkit 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/economy.rs:297

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/economy-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-038 — rgs-testkit → economy-service 呼び出し

### objective
rgs-testkit → economy-service 跨服务 gRPC 调用 (EconomyServiceClient, ctor)

### requirementIds
REQ-038

関連 Prompt: `PRM-038` (vv1.0.0)

### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/economy.rs:299-299` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 EconomyServiceClient (caller=rgs-testkit → callee=economy-service; 形态=ctor)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-testkit 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/economy.rs:299

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/economy-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-039 — rgs-testkit → economy-service 呼び出し

### objective
rgs-testkit → economy-service 跨服务 gRPC 调用 (EconomyServiceClient, type-annotation)

### requirementIds
REQ-039

関連 Prompt: `PRM-039` (vv1.0.0)

### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/economy.rs:308-308` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 EconomyServiceClient (caller=rgs-testkit → callee=economy-service; 形态=type-annotation)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-testkit 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/economy.rs:308

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/economy-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-040 — rgs-testkit → match-service 呼び出し

### objective
rgs-testkit → match-service 跨服务 gRPC 调用 (MatchServiceClient, type-annotation)

### requirementIds
REQ-040

関連 Prompt: `PRM-040` (vv1.0.0)

### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/match.rs:472-472` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 MatchServiceClient (caller=rgs-testkit → callee=match-service; 形态=type-annotation)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: 目标服务的 proto 定义 (evidence: fileLine) / 目标服务 migrations (evidence: fileLine)
- **出力**: rgs-testkit 的实现变更 / 对应 UT/IT 用例

### rules
- 仅使用仓库中已存在证据的路径; 不得引入未登记的新实体
- 凭据一律走环境变量, 永不打印 (per 守门 #5)
- 新表必须含 PK; 外键必须显式声明 REFERENCES

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/match.rs:472

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/match-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-041 — rgs-testkit → match-service 呼び出し

### objective
rgs-testkit → match-service 跨服务 gRPC 调用 (MatchServiceClient, ctor)

### requirementIds
REQ-041


### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/match.rs:473-473` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 MatchServiceClient (caller=rgs-testkit → callee=match-service; 形态=ctor)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: **TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)
- **出力**: **TBD** — 出力契約が未定義

### rules
- **TBD**: 業務ルール未定義

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/match.rs:473

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/match-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-042 — rgs-testkit → player-service 呼び出し

### objective
rgs-testkit → player-service 跨服务 gRPC 调用 (PlayerServiceClient, ctor)

### requirementIds
REQ-042


### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/player.rs:447-447` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 PlayerServiceClient (caller=rgs-testkit → callee=player-service; 形态=ctor)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: **TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)
- **出力**: **TBD** — 出力契約が未定義

### rules
- **TBD**: 業務ルール未定義

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/player.rs:447

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/player-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-043 — rgs-testkit → player-service 呼び出し

### objective
rgs-testkit → player-service 跨服务 gRPC 调用 (PlayerServiceClient, ctor)

### requirementIds
REQ-043


### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/player.rs:484-484` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 PlayerServiceClient (caller=rgs-testkit → callee=player-service; 形态=ctor)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: **TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)
- **出力**: **TBD** — 出力契約が未定義

### rules
- **TBD**: 業務ルール未定義

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/player.rs:484

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/player-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-044 — rgs-testkit → social-service 呼び出し

### objective
rgs-testkit → social-service 跨服务 gRPC 调用 (SocialServiceClient, type-annotation)

### requirementIds
REQ-044


### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/social.rs:459-459` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 SocialServiceClient (caller=rgs-testkit → callee=social-service; 形态=type-annotation)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: **TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)
- **出力**: **TBD** — 出力契約が未定義

### rules
- **TBD**: 業務ルール未定義

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/social.rs:459

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/social-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-045 — rgs-testkit → social-service 呼び出し

### objective
rgs-testkit → social-service 跨服务 gRPC 调用 (SocialServiceClient, ctor)

### requirementIds
REQ-045


### sourceEvidence
- `crates/rgs-testkit/src/bot/ai/social.rs:462-462` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 SocialServiceClient (caller=rgs-testkit → callee=social-service; 形态=ctor)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: **TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)
- **出力**: **TBD** — 出力契約が未定義

### rules
- **TBD**: 業務ルール未定義

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/ai/social.rs:462

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/social-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## TASK-046 — rgs-testkit → admin-service 呼び出し

### objective
rgs-testkit → admin-service 跨服务 gRPC 调用 (AdminServiceClient, ctor)

### requirementIds
REQ-046


### sourceEvidence
- `crates/rgs-testkit/src/bot/gm.rs:537-537` (rev `b6a998174582`, SourceCode) — 跨服务 client 真实使用点 AdminServiceClient (caller=rgs-testkit → callee=admin-service; 形态=ctor)

### targetModules
- `crates/rgs-testkit/` (crate `rgs::rgs-testkit` v0.1.0, .rs 17 件, テスト関数 84 件)


- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)

### inputOutput
- **入力**: **TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)
- **出力**: **TBD** — 出力契約が未定義

### rules
- **TBD**: 業務ルール未定義

### failureModes
- **TBD**: 障害モード未定義

### acceptanceTests
- [ ] 源码存在真实调用点 crates/rgs-testkit/src/bot/gm.rs:537

- 非目標: 不承诺该调用的端到端 SLA (无压测数据)

### dependency
- 上流: `crates/rgs-testkit/Cargo.toml` の依存 (src 実測)
- 呼び出し先: `crates/admin-service/` (実使用点あり — 依存が実在する証明)

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: 配備定義が **Unknown** のため運用手順は **TBD**
- 監視: 該当 crate の `shared-platform/outbox_metrics_reporter.rs` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**


---
## 付録: 全体統計

| 項目 | 件数 |
|---|---|
| 実装タスク | 46 |
| 要件 | 46 |
| プロンプト | 40 |
| Case | 9 |
| 因果ルール | 5 |
| 証拠レコード | 885 |
| サービス (crate) | 31 |
| gRPC サービス | 33 |
| テーブル | 61 |

## 付��: 実装時に守るべき共通規則

1. **証拠なしの事実を作らない。** 対象 crate に存在しない API / テーブル / 設定名を書かない。
2. **資格情報を出力しない。** 環境変数参照の形でのみ扱う。
3. **PK を持つこと。** 既存テーブルへの追加 стол は FK を明示し、`REFERENCES` を書く。
4. **可逆性。** 破壊的変更 (DROP / 削除) は、可観測性翘い Anterior を残してから行う。
5. **追跡可能性。** 実装は `REQ-xxx` / `CASE-xxx` / `file:line` を引数として持ち、
   成果物の PR 記述と本表を相互参照できるようにする。
