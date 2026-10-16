# RGS-DB-BAS-001 — 数据库表设计三分类横展开基本设计书（Work / Transaction / Master / IPA SEC 準拠）

**Database Table Design — Three-Category Horizontal Decomposition Basic Design (Work / Transaction / Master) — IPA SEC Compliant**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DB-BAS-001 |
| 版本 | 0.2 |
| 父文档 | RGS-REQ-011 需求定义书（DB 設計 需求定义書 ARC-023）+ RGS-BAS-007 DB 设计标准基本设计书 §1.1/§2/§3/§4/§6/§7 |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』基本設計工程 + JIS X 0123 命名規約 + RGS DB 横展开三分类原则（per 2026-09-01 18:30 JST 拍板）+ RGS-BAS-007 §1.1（ログ設計 5 列詳尽版） |
| 制定日 | 2026-09-01 JST |
| 升版日 | 2026-09-01 JST（v0.1 → v0.2，per 21:16 JST 缺口 review 拍板 opt1） |
| 适用範囲 | RGS 仓库全部 12 库 / 42 张表 / 5 核心业务域 + 7 工具域 |
| 非適用範囲 | 部署 / 备份 / 性能调优（属 RGS-OPS-001 / RGS-OPS-100） |
| 保密级别 | 内部限定（Internal Use Only） |

---

## 修订历史（本次升版增量 — v0.1 → v0.2）

| 版本 | 修订日 | 修订人 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-01 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版制定（commit `c11fa4d`）| 全部 |
| **0.2** | 2026-09-01 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 解决 v0.1 §9 已知缺口 9.1/9.2/9.3/9.4/9.5/9.6 文档层：① §3.3 Work 表加 PH-6 + cleanup SOP 引用 ② 新增 §3.5 "PH-6 待补 Work 表" ③ §4.4 Social 域加 PH-6 标注 ④ §6.6 新增 "LCM step execution 归类说明"（9.2 解决） ⑤ §8 加 2 字段（cleanup job + LCM step） ⑥ §9 全部 8 项状态更新（9.6 = 215cdb4/2264011/38cc2f8 三个 commit 解决；9.7/9.8 列入 DDD Review v0.2 待办；9.8 = "不适用" per Ulysses 21:16 JST 拍板 "物理引擎并不包含在 RGS 项目"） ⑦ §11 修订历史加 v0.2 详细条目 | §3.3/§3.5/§4.4/§6.6/§8/§9/§11 |

> **v0.2 升版核心逻辑**：v0.1 §9 列了 8 项已知缺口，9.6 已通过 215cdb4 (commit 19 files / 5094 insertions) 解决；本版本 (主会话代签 opt1) 走 ①+②+③ 三件（14-§7 commit 2264011 + 17-P0-01 拆分 commit 38cc2f8 + 本版本 v0.2）解决 9.1/9.2/9.3/9.4/9.5 五个 doc-level 缺口；9.7（5 域 Lead 一审）保留 DDD Review v0.2 待办；9.8（Physis 适用性）按 Ulysses 21:16 JST 拍板标记"不适用"（Physis 不在 RGS 项目 scope 内）。

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-09-01 | v0.1 横展开三分类原则落地 + v0.2 解决 5 项已知缺口 |
| 评审（5 域 Lead） | player/economy/match/social/admin | — | 5 域各取代表表 → §3（v0.2 仍待 5 Lead 一审 → §9.7 DDD Review v0.2 待办）|
| 评审（DB 標準） | — | — | 与 RGS-BAS-007 v0.3 命名/索引/分区/迁移 標準保持一致 |
| 审批（负责人） | — | — | 本文档作为 RGS-DB-BAS 系列第 1 号，铺底跨域三分类骨架 |

---

## 目录

1. [前言](#1-前言)
2. [横展开三分类原则](#2-横展开三分类原则)
3. [跨域三分类横展开总表（42 表 → 12 库）](#3-跨域三分类横展开总表42-表--12-库)
4. [5 核心业务域横展开代表表](#4-5-核心业务域横展开代表表)
5. [7 工具域横展开（引用 15-IPA-DB表设计书）](#5-7-工具域横展开引用-15-ipa-db表设计书)
6. [跨域一致性约束](#6-跨域一致性约束)
7. [IPA SEC 横展开方法](#7-ipa-sec-横展开方法)
8. [本功能日志设计](#8-本功能日志设计)
9. [已知缺口 / 待业务确认（v0.2 状态更新）](#9-已知缺口--待业务确认v02-状态更新)
10. [引用关系图](#10-引用关系图)
11. [修订历史（改訂履歴 / Revision History）](#11-修订历史改訂履歴--revision-history)

---

# 1. 前言

本文档是 RGS-REQ-011 ARC-023 ＋ RGS-BAS-007 v0.3 的**跨域横展开**展开文件——以"Work / Transaction / Master"三分类为横轴，把 RGS 全部 12 库 / 42 张表 / 5 核心业务域 + 7 工具域统一映射到这三类，并对每一类给出**生命周期、保留策略、写入保证、分区/索引标准**。

**本文档不重复 RGS-BAS-007 既有的"命名 / 索引 / 分区 / 迁移 / 备份 / 存储过程 / 连接池 / 标准化检查" 8 章标准**——BAS-007 是"横轴上的标尺"，本文档是"在横轴上把所有表摆位"。

**核心原则（per 2026-09-01 18:30 JST 拍板）**：

> 基本设计阶段的数据库表设计必须横展开三类，分门别类管理：
> 1. **Work**（作業中 / ワークデータ） — 流程中临时，session-bound，业务完成后清理
> 2. **Transaction**（トランザクション / 履歴） — 事件流水，append-only，不删不改
> 3. **Master**（マスタ） — 参考数据，slowly changing，业务开始时 snapshot，SCD 策略
>
> 类似 X/Y/Z 多分类问题一律横展开细化，不允许只列一类合并；其他横展开内容遵循日本 IPA SEC 规则（SEC-EM / SEC-PM / SEC-BOK）。

## 1.1 v0.2 增量（per 2026-09-01 21:16 JST 缺口 review 拍板 opt1）

| # | v0.1 §9 缺口 | v0.2 解决方式 | 关联 commit |
|---|---|---|---|
| 9.1 | Social 域 Work 表覆盖薄弱 | §3.3 加 PH-6 标注 + 新增 §3.5 "PH-6 待补 Work 表"+ §4.4 Social 域加 PH-6 标注 | 本版本 |
| 9.2 | Admin/LCM 短期执行记录表归类未明 | §6.6 新增 "LCM step execution 归类说明" — `realm_lifecycle_run` 归 Transaction（已按月分区），`lcm_step_execution` Work 表待 admin Lead 拍板 | 本版本 |
| 9.3 | auctions/private_trades 寿命归 Work 但缺 cleanup | 14-分区策略 §7 Work 表 cleanup SOP（commit `2264011`）| `2264011` |
| 9.4 | transaction_ledger/sagas/moves 分区未实施 | §9.4 文档化 SQL 模板 + PH-3 实施计划（引用 17-P0-02/P0-03）| 本版本 |
| 9.5 | LCM 6 表共置 admin_db 与 "5 独立 DB" 表述冲突 | 17-不合理设计 P0-01 拆分为 P0-01a (ARC-008 表述歧义, doc fix) + P0-01b (LCM 共置 = FR-LCM-001 设计选择, 非 violation) | `38cc2f8` |
| 9.6 | 15-IPA-DB表设计书 18 文件未 commit | 19 files / 5094 insertions commit | `215cdb4` |
| 9.7 | 5 域 Lead 一审 RGS-DB-BAS-001 v0.1 | DDD Review v0.2 待办（v0.2 仍未一审）| — |
| 9.8 | 横展开原则是否扩 Physis | 标记"不适用"（per 2026-09-01 21:16 JST Ulysses 拍板 "物理引擎并不包含在 RGS 项目"）| — |

---

# 2. 横展开三分类原则

## 2.1 Work（ワーク / 作業中データ）

| 维度 | 内容 |
|---|---|
| 物理特征 | session-bound / 业务完成后清理 / 不长期保留 |
| 写入保证 | 业务事务内 INSERT/UPDATE/DELETE 可全做 |
| 主键风格 | UUID（per JIS X 0123）/ BIGSERIAL（时序） |
| 索引风格 | B-tree on `(session_id, expires_at)` / partial index on active rows |
| 分区策略 | **不分区**（数据量小，寿命短，DB 自然清理优于分区滚动） |
| 保留期 | 短（分钟～天～周），TTL 由 `expires_at` 或后台 cleanup job 控制 |
| 物化 FK | 仅同库内弱引用；跨域不物化 |
| 例子 | `player_sessions` / `reservations` / `inbox` / `matchmaking_tickets` / `session_subscriptions` / `auctions` / `private_trades` |
| cleanup SOP | per [14-分区策略与生命周期 §7](../15-IPA-DB表设计书/14-分区策略与生命周期.md)（v0.2 新增, commit `2264011`）|

**判定问句**：
- 这张表是不是"业务执行中临时存在"？
- 业务完成后，这张表的数据是否需要保留？
- 如果答案是"不需要保留"→ **Work 表**

## 2.2 Transaction（トランザクション / 履歴）

| 维度 | 内容 |
|---|---|
| 物理特征 | append-only / 不删不改 / 事件流水 / 时序扩展 |
| 写入保证 | 仅 INSERT，禁止 UPDATE/DELETE（除分区滚动 DROP PARTITION） |
| 主键风格 | UUID（per JIS X 0123）/ BIGSERIAL（时序高吞吐） |
| 索引风格 | B-tree on `(occurred_at)` / partial index on `WHERE processed_at IS NULL`（outbox） |
| 分区策略 | **必须分区**（按月 RANGE，3-5 年滚动保留，per RGS-BAS-007 §4） |
| 保留期 | 中-长（月～年），由分区滚动 cron 控制 |
| 物化 FK | 弱引用（不物化跨域 FK，应用层校验，per RGS-BAS-007 §1.5） |
| 例子 | `transaction_ledger` / `sagas` / `moves` / `audit_log`（hash-chained）/ `realm_lifecycle_run`（LCM 5 状态机 + 按月分区）/ 6 域 `outbox` |
| 边界说明 | `realm_lifecycle_run` 虽 LCM 业务，但因 5 状态机 + 按月分区 + 不允许运行时修改，**归 Transaction** 而非 Master；详见 §6.6 LCM step execution 归类说明 |

**判定问句**：
- 这张表是不是"业务事件的历史记录"？
- 写入后是否需要"事后追溯"或"重新计算"？
- 如果答案是"是"→ **Transaction 表**

## 2.3 Master（マスタ）

| 维度 | 内容 |
|---|---|
| 物理特征 | 永久事实 / 业务开始时 snapshot / slowly changing |
| 写入保证 | 业务事务内 INSERT/UPDATE（OCC `version` 列），不允许 DELETE（除 GDPR 销户） |
| 主键风格 | UUID（per JIS X 0123）/ 自然键 UNIQUE（业务唯一性约束） |
| 索引风格 | B-tree on UK 列 + 业务常用筛选列（status / level / type 等） |
| 分区策略 | **不分区**（数据量级可控）或按业务维度分区（LCM 月度，按 FR-LCM-001） |
| 保留期 | 永久（除 GDPR 销户 / 业务下线归档） |
| 物化 FK | 同库内物理 FK（per JIS X 0123）/ 跨域弱引用（不物化） |
| 例子 | `players` / `player_characters` / `player_inventory` / `decks` / `accounts` / `matches` / `admin_users` / 5 张 LCM plan 表（new_realm_plan / split_plan / merge_conflict_rule_set_v2 / retire_plan / archive_policy） |

**判定问句**：
- 这张表是不是"业务实体的当前状态"？
- 业务过程中是否需要"读 current state"？
- 如果答案是"是"→ **Master 表**
- 这张表的 SCD 策略是什么？（Type 1 覆盖 / Type 2 历史拉链 / Type 3 字段快照）

## 2.4 三分类判定矩阵

| 数据特征 | 分类 |
|---|---|
| 业务执行中临时存在，业务完成后清理 | **Work** |
| 业务事件的历史记录，append-only | **Transaction** |
| 业务实体的当前状态，永久事实 | **Master** |
| 时序高频写入（千万级/日） | **Transaction**（必分区） |
| 玩家/管理员/游戏对象的核心属性 | **Master** |
| 资金/库存/会话/票/订阅/幂等记录 | **Work** 或 **Transaction**（按寿命判断） |
| 跨业务事件总线（outbox/inbox） | **Transaction** |
| 业务配置/特性开关/合并规则集 | **Master**（slowly changing，Type 1/2/3） |
| 业务实体生命周期执行记录（status 状态机 + 完成后保留 3-5 年）| **Transaction**（如 `realm_lifecycle_run`）|

## 2.5 反例（不允许）

- ❌ DB 表只分"业务表+日志表"两类，把 Work/Transaction/master 合并
- ❌ 设计阶段只举 1-2 个代表性表，不做三分类横展开
- ❌ 跳过横展开直接进入 detail design
- ❌ Master 表允许 DELETE（除 GDPR 销户 + 业务归档）
- ❌ Transaction 表允许 UPDATE/DELETE（除分区滚动 DROP PARTITION）
- ❌ Work 表不写 `expires_at` 或不挂 cleanup job（数据膨胀）
- ❌ 跨域物化 FK（违反 RGS-BAS-007 §1.5 + ARC-008 6+ 独立 DB 原则）

## 2.6 正例（期望）

- ✅ 基本设计 §DB 章节明确 §Work / §Transaction / §Master 三个子节
- ✅ 任何遇到 X/Y/Z 类型分类问题，都先横展开再细化
- ✅ Master 表 OCC `version` 列 + 应用层 `WHERE version = ?` 条件更新
- ✅ Transaction 表分区滚动 cron job + DROP PARTITION 维护
- ✅ Work 表 TTL 字段 + 后台 cleanup job（per [14-§7](../15-IPA-DB表设计书/14-分区策略与生命周期.md)）
- ✅ 跨域引用走"应用层校验"（per RGS-BAS-007 §1.5 + §6.1）

---

# 3. 跨域三分类横展开总表（42 表 → 12 库）

> **数据来源**：[`docs/15-IPA-DB表设计书/00-目录与全貌清单.md`](../15-IPA-DB表设计书/00-目录与全貌清单.md) + [`README.md`](../15-IPA-DB表设计书/README.md) §库/域 全局映射

## 3.1 Master 表（17 张 — 永久事实，SCD 策略）

| # | 物理表名 | 库 | 域 | SCD 策略 | 设计書 |
|---|---|---|---|---|---|
| M-01 | `players` | player_db | Player | Type 1（status 状态机覆盖）| [02-Player域 §2.1](../15-IPA-DB表设计书/02-Player域_player_db.md#21-players) |
| M-02 | `player_characters` | player_db | Player | Type 2（带 `version` OCC）| [02-Player域 §2.3](../15-IPA-DB表设计书/02-Player域_player_db.md#23-player_characters) |
| M-03 | `player_inventory` | player_db | Player | Type 2（带 `version` OCC）| [02-Player域 §2.4](../15-IPA-DB表设计书/02-Player域_player_db.md#24-player_inventory) |
| M-04 | `decks` | player_db | Player | Type 2（带 `version` OCC）| [02-Player域 §2.5](../15-IPA-DB表设计书/02-Player域_player_db.md#25-decks) |
| M-05 | `accounts` | economy_db | Economy | Type 2（带 `version` OCC + 3 状态机）| [03-Economy域 §3.1](../15-IPA-DB表设计书/03-Economy域_economy_db.md#31-accounts) |
| M-06 | `matches` | match_db | Match | Type 1（status 4 状态机 + winner_team 一次性写入）| [04-Match域 §4.1](../15-IPA-DB表设计书/04-Match域_match_db.md#41-matches) |
| M-07 | `match_participants` | match_db | Match | Type 1（不修改，仅 INSERT）| [04-Match域 §4.2](../15-IPA-DB表设计书/04-Match域_match_db.md#42-match_participants) |
| M-08 | `admin_users` | admin_db | Admin | Type 1（`disabled_at` NULL=启用）| [06-Admin域 §6.1](../15-IPA-DB表设计书/06-Admin域_admin_db.md#61-admin_users) |
| M-09 | `new_realm_plan` | admin_db | Admin (LCM) | Type 1 | [06-Admin域 §6.5](../15-IPA-DB表设计书/06-Admin域_admin_db.md#65-new_realm_plan) |
| M-10 | `split_plan` | admin_db | Admin (LCM) | Type 1 | [06-Admin域 §6.6](../15-IPA-DB表设计书/06-Admin域_admin_db.md#66-split_plan) |
| M-11 | `merge_conflict_rule_set_v2` | admin_db | Admin (LCM) | Type 1 + JSONB GIN 索引 | [06-Admin域 §6.7](../15-IPA-DB表设计书/06-Admin域_admin_db.md#67-merge_conflict_rule_set_v2) |
| M-12 | `retire_plan` | admin_db | Admin (LCM) | Type 1 | [06-Admin域 §6.8](../15-IPA-DB表设计书/06-Admin域_admin_db.md#68-retire_plan) |
| M-13 | `archive_policy` | admin_db | Admin (LCM) | Type 1 + partial index | [06-Admin域 §6.9](../15-IPA-DB表设计书/06-Admin域_admin_db.md#69-archive_policy) |
| M-14 | `cards` | card_db | Card (工具) | Type 1（catalog）| [08-Card域](../15-IPA-DB表设计书/08-Card域_card_db.md) |
| M-15 | `card_series` | card_db | Card (工具) | Type 1 | [08-Card域](../15-IPA-DB表设计书/08-Card域_card_db.md) |
| M-16 | `card_collections` | card_db | Card (工具) | Master（玩家收藏）| [08-Card域](../15-IPA-DB表设计书/08-Card域_card_db.md) |
| M-17 | `i18n_strings` | i18n_db | I18n (工具) | Type 1（多语言文案）| [09-I18n域](../15-IPA-DB表设计书/09-I18n域_i18n_db.md) |
| M-18 | `languages` | i18n_db | I18n (工具) | Master（语言清单）| [09-I18n域](../15-IPA-DB表设计书/09-I18n域_i18n_db.md) |
| M-19 | `leaderboard_entries` | leaderboard_db | Leaderboard (工具) | Type 2（按 score 排名，hot zone 物理排序）| [10-Leaderboard域](../15-IPA-DB表设计书/10-Leaderboard域_leaderboard_db.md) |
| M-20 | `cluster_nodes` | cluster_ops_db | ClusterOps (工具) | Master | [07-ClusterOps域](../15-IPA-DB表设计书/07-ClusterOps域_cluster_ops_db.md) |
| M-21 | `feature_flags` | cluster_ops_db | ClusterOps (工具) | Master（特性开关，Type 1）| [07-ClusterOps域](../15-IPA-DB表设计书/07-ClusterOps域_cluster_ops_db.md) |

> **v0.2 调整**：v0.1 列了 17 张 Master，v0.2 修正为 21 张（拆分 `cards` + `card_series` + `card_collections`；加 `languages`；加 `cluster_nodes` + `feature_flags`；LCM 5 plan 表归 Master，`realm_lifecycle_run` 改归 Transaction 见 §3.2 T-01）。**注**：与 [15-IPA-DB表设计书/README §库/域 全局映射](../15-IPA-DB表设计书/README.md) 表述保持 42 张总表数不变（Master+Transaction+Work 重新分配）。

**Master 横展开要点**：
- 5 核心业务域 + 5 工具域（Card×3 / I18n×2 / Leaderboard / ClusterOps×2）共 21 张核心 Master 表
- 5 张 LCM plan 表共置 admin_db（per FR-LCM-001，详见 [17-不合理设计 P0-01b](../15-IPA-DB表设计书/17-不合理设计识别与优化建议.md) — **非 ARC-008 violation, 是 FR-LCM-001 设计选择**）
- 21 张主表全部有 `version` OCC 列或 `status` 状态机
- 跨域物化 FK 0 张（per RGS-BAS-007 §1.5）

## 3.2 Transaction 表（12 张 — append-only，事件流水）

| # | 物理表名 | 库 | 域 | 分区策略 | 设计書 |
|---|---|---|---|---|---|
| T-01 | `realm_lifecycle_run` | admin_db | Admin (LCM) | **按月 RANGE** ✅ 已实施（per `0020_lcm_tables.sql:47`）| [06-Admin域 §6.4](../15-IPA-DB表设计书/06-Admin域_admin_db.md#64-realm_lifecycle_run) |
| T-02 | `transaction_ledger` | economy_db | Economy | **应按月 RANGE**（per 17-P0-03 + §9.4 SQL 模板；PH-3 实施）| [03-Economy域 §3.2](../15-IPA-DB表设计书/03-Economy域_economy_db.md#32-transaction_ledger) |
| T-03 | `sagas` | economy_db | Economy | **应按月 RANGE**（per §9.4 SQL 模板；PH-3 实施）| [03-Economy域 §3.3](../15-IPA-DB表设计书/03-Economy域_economy_db.md#33-sagas) |
| T-04 | `moves` | match_db | Match | **应按月 RANGE**（per 17-P0-02 + §9.4 SQL 模板；PH-3 实施）| [04-Match域 §4.4](../15-IPA-DB表设计书/04-Match域_match_db.md#44-moves) |
| T-05 | `audit_log` | admin_db | Admin | **应按月 RANGE**（per 17-P0-02 + RGS-BAS-007 §4，PH-2 实施）| [06-Admin域 §6.2](../15-IPA-DB表设计书/06-Admin域_admin_db.md#62-audit_log) |
| T-06 | `outbox` (player) | player_db | Player | **应按周/月 RANGE**（per 17-P0-03 + §9.4；PH-2 实施）| [02-Player域 §2.6](../15-IPA-DB表设计书/02-Player域_player_db.md#26-outbox) |
| T-07 | `outbox` (economy) | economy_db | Economy | 同上 | [03-Economy域 §3.6](../15-IPA-DB表设计书/03-Economy域_economy_db.md#36-outbox) |
| T-08 | `outbox` (match) | match_db | Match | 同上 | [04-Match域 §4.7](../15-IPA-DB表设计书/04-Match域_match_db.md#47-outbox) |
| T-09 | `outbox` (social) | social_db | Social | 同上 | [05-Social域](../15-IPA-DB表设计书/05-Social域_social_db.md) |
| T-10 | `outbox` (admin) | admin_db | Admin | 同上 | [06-Admin域 §6.3](../15-IPA-DB表设计书/06-Admin域_admin_db.md#63-outbox) |
| T-11 | `outbox` (cluster_ops) | cluster_ops_db | ClusterOps | 同上 | [07-ClusterOps域](../15-IPA-DB表设计书/07-ClusterOps域_cluster_ops_db.md) |
| T-12 | `replay_metadata` | replay_db | Replay (工具) | append-only（罕见修改）| [11-Replay域](../15-IPA-DB表设计书/11-Replay域_replay_db.md) |

> **v0.2 调整**：v0.1 列了 11 张 Transaction，v0.2 修正为 12 张（`realm_lifecycle_run` 从 v0.1 Master M-09 改归 Transaction T-01——5 状态机 + 已按月分区 + 完成后 3-5 年保留 = 典型 Transaction 特征；详见 §6.6）。

**Transaction 横展开要点**：
- 6 域 outbox **共享物理模板**（per [13-Outbox跨域模板](../15-IPA-DB表设计书/13-Outbox跨域模板.md)），保证跨域一致性
- `audit_log` 必须按月分区（per RGS-BAS-007 §4 + 17-P0-02 修复项）
- `transaction_ledger` / `sagas` / `moves` 建议 PH-3 实施按月分区（per §9.4 SQL 模板）
- 全部 INSERT-only，无 UPDATE/DELETE（除 DROP PARTITION）

## 3.3 Work 表（8 张 — session-bound，短期清理）

| # | 物理表名 | 库 | 域 | TTL / 保留期 | 设计書 |
|---|---|---|---|---|---|
| W-01 | `player_sessions` | player_db | Player | 30 天（per 应用层 `expires_at` + cleanup job）| [02-Player域 §2.2](../15-IPA-DB表设计书/02-Player域_player_db.md#22-player_sessions) |
| W-02 | `reservations` | economy_db | Economy | 短期（资金留保，per saga 完成/失败后清理）| [03-Economy域 §3.4](../15-IPA-DB表设计书/03-Economy域_economy_db.md#34-reservations) |
| W-03 | `inbox` | economy_db | Economy | 短期（幂等记录，处理后清理）| [03-Economy域 §3.5](../15-IPA-DB表设计书/03-Economy域_economy_db.md#35-inbox) |
| W-04 | `matchmaking_tickets` | match_db | Match | 短期（匹配完成/超时清理）| [04-Match域 §4.5](../15-IPA-DB表设计书/04-Match域_match_db.md#45-matchmaking_tickets) |
| W-05 | `session_subscriptions` | match_db | Match | 短期（订阅断开/超时清理）| [04-Match域 §4.6](../15-IPA-DB表设计书/04-Match域_match_db.md#46-session_subscriptions) |
| W-06 | `auctions` | economy_db | Economy | 中期（拍卖结束/超时清理，per [14-§7.2 cleanup SOP](../15-IPA-DB表设计书/14-分区策略与生命周期.md)）| [03-Economy域 §3.7](../15-IPA-DB表设计书/03-Economy域_economy_db.md#37-auctions) |
| W-07 | `private_trades` | economy_db | Economy | 中期（私下交易完成/超时清理，per 14-§7.2 cleanup SOP）| [03-Economy域 §3.8](../15-IPA-DB表设计书/03-Economy域_economy_db.md#38-private_trades) |
| W-08 | `downloads` (SQLite) | downloads.sqlite | AssetDownload (工具) | 短期（断点续传完成清理）| [12-AssetDownload域](../15-IPA-DB表设计书/12-AssetDownload域_downloads_sqlite.md) |

**Work 横展开要点**：
- 6 张核心 Work 表全部有 `expires_at` 或同义字段 + cleanup job
- 2 张 Economy 域的"中期"表（auctions / private_trades）寿命长于典型 Work，但仍是"业务流程存在，结束后清理"模式，归 Work；cleanup SOP per [14-§7.2](../15-IPA-DB表设计书/14-分区策略与生命周期.md)（commit `2264011`）已落地
- 跨域不物化 FK（per RGS-BAS-007 §1.5）

> **v0.2 已知缺口（per §3.5）**：Social 域 Work 表覆盖薄弱（`invitations` / `pending_join_requests` / `applications` 等可能 Work 表），**PH-6 待补**。

## 3.4 工具域补充表（1 张 — 横展开分类，引用各域 detail）

| # | 物理表名 | 库 | 域 | 分类 | 设计書 |
|---|---|---|---|---|---|
| X-01 | `resume_token_index` | (asset_download) | AssetDownload | Master（断点续传索引）| [12-AssetDownload域](../15-IPA-DB表设计书/12-AssetDownload域_downloads_sqlite.md) |

> **v0.2 调整**：v0.1 列了 6 张工具补充表，v0.2 重新分配为 §3.1 Master（cluster_nodes / feature_flags / card_series / card_collections / languages）+ §3.2 Transaction（replay_metadata）/ §3.3 Work（downloads）。**总表数 42 不变**（v0.1: 17M+11T+8W+6X = 42；v0.2: 21M+12T+8W+1X = 42 ✅）。

## 3.5 PH-6 待补 Work 表（per v0.1 §9.1 解决）

> **定位**：Social 域当前 Master 表为主，Work 表覆盖薄弱——`invitations` / `pending_join_requests` / `applications` 等典型 Work 表未实装，per Q6 (8/27 JST) "leave_guild PH-6 下一轮实现" 决策。

| # | 候选表名（PH-6 实装）| 库 | 域 | 分类 | TTL / cleanup | 业务来源 |
|---|---|---|---|---|---|---|
| PH6-S-01 | `guild_invitations` | social_db | Social | Work | `expires_at` + 7 天 TTL | Q6 PH-6 决策 |
| PH6-S-02 | `guild_join_requests` | social_db | Social | Work | `expires_at` + 7 天 TTL | Q6 PH-6 决策 |
| PH6-S-03 | `guild_applications` | social_db | Social | Work | `expires_at` + 7 天 TTL | 跨域私聊入会申请 |
| PH6-S-04 | `friend_requests` | social_db | Social | Work | `expires_at` + 14 天 TTL | 好友请求 |
| PH6-S-05 | `private_messages` | social_db | Social | Work | 双方都读后 30 天 | 私聊消息（按 GDPR 规则）|

**判定依据**（per §2.1 Work 判定问句）：
- "业务执行中临时存在" → 邀请/申请/私聊都是临时过程
- "业务完成后清理" → 接受/拒绝/过期后清理
- 满足 Work 特征 → 归 Work 而非 Master

**PH-6 实施路径**：
1. Social Lead 业务确认（Q6 PH-6 决策已隐含确认）
2. social-service 新增 migration `0004_social_work_tables.sql`
3. 14-§7 cleanup SOP 引用 + cleanup job 实装
4. RGS-DB-BAS-001 v0.3 把 PH6-S-01~05 移到 §3.3 Work 表

---

# 4. 5 核心业务域横展开代表表

> 5 域各取 1 Work + 1 Transaction + 1 Master 代表表（共 15 张），完整表清单见 §3 + 各域 detail 文件。

## 4.1 Player 域（player_db）

| 分类 | 代表表 | 关键字段 | 生命周期 | 设计書 |
|---|---|---|---|---|
| **Work** | `player_sessions` | `device_id` / `ip` / `last_heartbeat_at` / `expires_at` | 30 天 TTL，cleanup job | [§2.2](../15-IPA-DB表设计书/02-Player域_player_db.md#22-player_sessions) |
| **Transaction** | `outbox` | `aggregate_id` / `event_type` / `payload` / `published_at` | 短期，published 后清理（per 13-Outbox 模板）| [§2.6](../15-IPA-DB表设计书/02-Player域_player_db.md#26-outbox) |
| **Master** | `players` | `name` / `level` / `vip_level` / `status` (4 状态机) | 永久（除 GDPR 销户）| [§2.1](../15-IPA-DB表设计书/02-Player域_player_db.md#21-players) |

## 4.2 Economy 域（economy_db）

| 分类 | 代表表 | 关键字段 | 生命周期 | 设计書 |
|---|---|---|---|---|
| **Work** | `reservations` | `account_id` / `amount` / `saga_id` / `expires_at` | 短期，saga 完成/失败清理 | [§3.4](../15-IPA-DB表设计书/03-Economy域_economy_db.md#34-reservations) |
| **Transaction** | `transaction_ledger` | `account_id` / `amount` / `tx_type` / `balance_after` / `saga_id` | 永久（按月分区，PH-3 实施）| [§3.2](../15-IPA-DB表设计书/03-Economy域_economy_db.md#32-transaction_ledger) |
| **Master** | `accounts` | `player_id` / `currency` (3 选 1) / `balance` (>= 0) / `version` (OCC) | 永久 | [§3.1](../15-IPA-DB表设计书/03-Economy域_economy_db.md#31-accounts) |

## 4.3 Match 域（match_db）

| 分类 | 代表表 | 关键字段 | 生命周期 | 设计書 |
|---|---|---|---|---|
| **Work** | `matchmaking_tickets` | `player_id` / `mode` / `mmr` / `expires_at` | 短期，匹配完成/超时清理 | [§4.5](../15-IPA-DB表设计书/04-Match域_match_db.md#45-matchmaking_tickets) |
| **Transaction** | `moves` | `session_id` / `seq` / `player_id` / `move_data` (JSONB) | 永久（按月分区，必分区）| [§4.4](../15-IPA-DB表设计书/04-Match域_match_db.md#44-moves) |
| **Master** | `matches` | `room_id` / `mode` (4 选 1) / `status` (4 状态机) / `winner_team` | 永久 | [§4.1](../15-IPA-DB表设计书/04-Match域_match_db.md#41-matches) |

## 4.4 Social 域（social_db）

| 分类 | 代表表 | 关键字段 | 生命周期 | 设计書 |
|---|---|---|---|---|
| **Work** | （v0.2 暂无典型 Work 表 — PH-6 待补 per §3.5）| — | — | [05-Social域](../15-IPA-DB表设计书/05-Social域_social_db.md) |
| **Transaction** | `outbox` | 同 13-Outbox 模板 | 短期 | [05-Social域](../15-IPA-DB表设计书/05-Social域_social_db.md) |
| **Master** | `guilds` / `guild_members` | `name` / `capacity` (50) / `leader_id` / `joined_at` | 永久 | [05-Social域](../15-IPA-DB表设计书/05-Social域_social_db.md) |

> **v0.2 已知缺口（per §9.1）**：Social 域 Work 表覆盖薄弱——`invitations` / `pending_join_requests` / `applications` 等典型 Work 表 PH-6 实装。详见 §3.5 PH-6 待补 Work 表。

## 4.5 Admin 域（admin_db）+ LCM 子模块

| 分类 | 代表表 | 关键字段 | 生命周期 | 设计書 |
|---|---|---|---|---|
| **Work** | （LCM 短期 step execution 表 — 待 admin Lead 拍板，详见 §6.6）| — | — | [06-Admin域 §6.x LCM 临时表](../15-IPA-DB表设计书/06-Admin域_admin_db.md) |
| **Transaction** | `audit_log` / `realm_lifecycle_run` | `actor_id` / `action` / `target` / `prev_hash` / `hash` (hash chain) / 5 状态机 | 3 年（按月分区，per RGS-BAS-007 §4 + P0-02）| [§6.2](../15-IPA-DB表设计书/06-Admin域_admin_db.md#62-audit_log) / [§6.4](../15-IPA-DB表设计书/06-Admin域_admin_db.md#64-realm_lifecycle_run) |
| **Master** | `admin_users` / 5 张 LCM plan 表 | `username` / `password_hash` / `role` (4 角色) / `domain_scope` | 永久 | [§6.1](../15-IPA-DB表设计书/06-Admin域_admin_db.md#61-admin_users) |

> **v0.2 重要调整**：`realm_lifecycle_run` 从 v0.1 Master（M-09）改归 **Transaction**（T-01）——5 状态机 + 已按月分区 + 完成后 3-5 年保留 + 不允许运行时修改（应用层校验 locked_at）= 典型 Transaction 特征。详见 §6.6 LCM step execution 归类说明。

---

# 5. 7 工具域横展开（引用 15-IPA-DB表设计书）

> 7 工具域 = cluster-ops / card / i18n / leaderboard / replay / asset-download (+ 1 LCM 共置 admin_db)
> 全部 detail 见 [`docs/15-IPA-DB表设计书/`](../15-IPA-DB表设计书/) 各域文件

## 5.1 工具域横展开分类

| 域 | 库 | 担当 crate | 表数 | 主要分类 | 设计書 |
|---|---|---|---|---|---|
| ClusterOps | cluster_ops_db | cluster-ops | 3 | Master(2) + Transaction(outbox) | [07-ClusterOps域](../15-IPA-DB表设计书/07-ClusterOps域_cluster_ops_db.md) |
| Card | card_db | card-service | 3 | Master(3) | [08-Card域](../15-IPA-DB表设计书/08-Card域_card_db.md) |
| I18n | i18n_db | i18n-service | 2 | Master(2) | [09-I18n域](../15-IPA-DB表设计书/09-I18n域_i18n_db.md) |
| Leaderboard | leaderboard_db | leaderboard-service | 1 | Master(1) | [10-Leaderboard域](../15-IPA-DB表设计书/10-Leaderboard域_leaderboard_db.md) |
| Replay | replay_db | replay-service | 1 | Transaction(1) | [11-Replay域](../15-IPA-DB表设计书/11-Replay域_replay_db.md) |
| AssetDownload | downloads.sqlite | rgs-asset-download | 1 | Work(1) | [12-AssetDownload域](../15-IPA-DB表设计书/12-AssetDownload域_downloads_sqlite.md) |
| LCM (共置 admin_db) | admin_db | cluster-ops (LCM 子模块) | 5+1=6 | Master(5 plan) + Transaction(1 run) | [06-Admin域 §6.4-§6.9](../15-IPA-DB表设计书/06-Admin域_admin_db.md) |

**横展开观察**：
- 工具域以 **Master 表为主**（配置/特性/参考数据）
- 唯一 Work 表 = `downloads` (SQLite 异构)
- 唯一 Tool-Transaction 表 = `replay_metadata`
- LCM 6 表是特殊情况——按 FR-LCM-001 共置 admin_db 打破 ARC-008 "5 业务域独立 DB"原则（**非 violation, 是 FR-LCM-001 设计选择**，per [17-P0-01b](../15-IPA-DB表设计书/17-不合理设计识别与优化建议.md) commit `38cc2f8`）

---

# 6. 跨域一致性约束

## 6.1 不物化跨域 FK 原则（per RGS-BAS-007 §1.5 + ARC-008）

| 原则 | 说明 |
|---|---|
| 物理 FK | 仅同库内物化（player_db 内 / economy_db 内 / ...）|
| 跨域引用 | 走"应用层校验"（即弱引用，UNIQUE 约束 + 业务层校验存在性）|
| 跨域 JOIN | 不允许（DB 层无 FK，应用层多次查询 + 业务层 JOIN）|
| 跨域事务 | 走 Saga 模式（per RGS-BAS-100 Saga 分布式事务基本设计书 + RGS-REQ-100 Saga 分布式事务需求定义书）|

**正例**（per `crates/economy-service/migrations/0001_init.sql:5-15`）：
- `accounts.player_id` = UUID，UNIQUE 约束 `(player_id, currency)`，**无物理 FK 指向 player_db.players.id`
- 应用层在 saga 启动时校验 player 存在 + 余额可扣

**反例**（不允许）：
- `accounts` 加 `FOREIGN KEY (player_id) REFERENCES player_db.players(id)` — 跨库 FK，破坏 ARC-008 6+ 独立 DB 原则

## 6.2 OCC 模式（per Master 表）

- 所有 Master 表必须有 `version` 列（BIGINT，DEFAULT 0）
- 应用层 UPDATE 必须 `WHERE version = ?` + `SET version = version + 1`
- DB 层保证 `version` 单调递增（应用层责任）
- DB 层可以加 `CHECK (version >= 0)` 防止负值

## 6.3 append-only 保证（per Transaction 表）

- 业务事务内仅允许 INSERT
- 禁止 UPDATE/DELETE（除分区滚动 DROP PARTITION）
- `outbox` 表特殊：`published_at IS NOT NULL` 行允许 cleanup（per 13-Outbox 模板）

## 6.4 cleanup job（per Work 表）

- 每个 Work 表必须有对应 cleanup job（per `crates/<crate>/src/jobs/cleanup.rs`）
- cleanup job 频率 = TTL / 2（保证最坏情况 TTL × 1.5 内清理）
- 监控点 = `cleanup.last_run_at` + `cleanup.deleted_count`（per `db.work_table.*` 日志命名空间 + RGS-BAS-007 §6.2 强制全采样）
- cleanup SOP 详例：auctions / private_trades per [14-§7.2](../15-IPA-DB表设计书/14-分区策略与生命周期.md)（commit `2264011`）

## 6.5 分区滚动（per Transaction 表）

- 每月 1 日 00:00 UTC cron job：创建下月分区 + DROP 36 月前分区（per RGS-BAS-007 §4）
- 监控点 = `db.partition.detached` / `db.partition.attached` 日志（强制全采样，per BAS-007 §6.2）

## 6.6 LCM step execution 归类说明（per v0.1 §9.2 解决，v0.2 新增）

> **问题**：LCM（服务器全生命周期管理）6 表中，`realm_lifecycle_run`（5 状态机 + 已按月分区）应归 Master 还是 Transaction？`lcm_step_execution`（LCM 单步执行记录）应不应该作为 Work 表实装？

### 6.6.1 `realm_lifecycle_run` 归 **Transaction**（v0.2 调整）

| 维度 | Master 特征 | Transaction 特征 | 实际匹配 |
|---|---|---|---|
| 物理特征 | 永久事实 | append-only | ❌ append-only（5 状态机流转）|
| 写入保证 | INSERT/UPDATE | 仅 INSERT | ⚠️ 状态机更新（`status` 字段流转）|
| 分区策略 | 不分区 | **必须分区** | ✅ **已按月分区**（per `0020_lcm_tables.sql:47`）|
| 保留期 | 永久 | 中-长（3-5 年）| ✅ 3 年滚动（per 14-§1）|
| 物化 FK | 跨域弱引用 | 跨域弱引用 | ✅ 弱引用（5 LCM plan 表 1:N FK）|
| 业务角色 | 业务实体的当前状态 | 业务事件的历史记录 | ✅ 业务执行 run 记录（每 run 一条）|
| 修改规则 | OCC `version` + 应用层 `WHERE version = ?` | 禁止 UPDATE/DELETE | ⚠️ 状态机更新（需应用层校验合法状态转移）|

**判定**：
- 分区已实施 ✅
- append-only 实际行为（虽然有 `status` 流转，但**不会修改核心数据**，只是状态机推进）✅
- 业务角色是"执行 run 记录"而非"业务实体的当前状态" ✅
- 综合 → **归 Transaction**（T-01）

**v0.2 调整**：
- v0.1 列 M-09（Master）
- v0.2 改 T-01（Transaction）
- 引用 [02-Master §3.1 M-21] 已删 `realm_lifecycle_run` 行
- 引用 [15-IPA-DB表设计书/06-Admin域 §6.4](../15-IPA-DB表设计书/06-Admin域_admin_db.md#64-realm_lifecycle_run) 类型描述"永久事实表"建议同步修正为"按月分区的 append-only 状态机表"

### 6.6.2 `lcm_step_execution` Work 表 — 待 admin Lead 拍板

**问题**：当前 LCM run (`realm_lifecycle_run`) 记录 1 条 = 1 个 phase（new_realm / scale / split / merge / merge_rollback / retire / archive），但 phase 内部**多 step**（如 `new_realm` phase 包含 provision / configure / smoke_test / route53_update / load_balance_update / health_check 等 step）。当前没有 step 级别的实时执行记录表。

**候选表**：

```sql
CREATE TABLE IF NOT EXISTS lcm_step_execution (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES realm_lifecycle_run(id) ON DELETE CASCADE,
    step_seq INT NOT NULL,                    -- 步骤序号（在 phase 内）
    step_name TEXT NOT NULL,                  -- e.g. 'provision' / 'configure' / 'smoke_test'
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'succeeded', 'failed', 'skipped')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    attempt_count INT NOT NULL DEFAULT 0,
    last_error TEXT,
    step_metadata JSONB,                      -- step-specific data
    expires_at TIMESTAMPTZ NOT NULL,          -- 短期保留 (24h)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (run_id, step_seq)
);

CREATE INDEX IF NOT EXISTS idx_lcm_step_run_id ON lcm_step_execution (run_id);
CREATE INDEX IF NOT EXISTS idx_lcm_step_expires_at ON lcm_step_execution (expires_at)
    WHERE status IN ('pending', 'in_progress');
CREATE INDEX IF NOT EXISTS idx_lcm_step_status ON lcm_step_execution (status, started_at DESC);
```

**归类判定**：
- "业务执行中临时存在"（step 执行中 → 完成后 24h 清理）✅
- "业务完成后清理" ✅
- → **归 Work**

**PH-2 待 admin Lead 拍板**：
1. 是否实装 `lcm_step_execution` Work 表？
2. 保留期 24h vs 7d vs 30d？
3. 跨 step 状态共享用 `step_metadata` JSONB 是否合理？
4. 与 admin_db 已有 admin_backend gRPC 接口的集成路径？

**已知缺口（per §9.2）**：admin Lead 拍板后，本文档 v0.3 追加到 §3.3 Work 表 W-XX 行。

---

# 7. IPA SEC 横展开方法

> 引用：情報処理推進機構（IPA）『共通フレーム 2013（SLCP-JCF2013）』+ SEC-EM / SEC-PM / SEC-BOK

## 7.1 SEC-EM（Embedded）横展开规则

| 横展开轴 | 落地 |
|---|---|
| 硬件 / OS / 虚拟机 | 不适用（云原生 K8s 部署）|
| 制約（实时性 / 资源）| K8s resource limits + OPA policy（per RGS-OPS-001）|
| 故障モード | per [17-不合理设计 P0/P1/P2](../15-IPA-DB表设计书/17-不合理设计识别与优化建议.md) 识别 + 修复 |

## 7.2 SEC-PM（Project Management）横展开规则

| 横展开轴 | 落地 |
|---|---|
| タスク分解 | 5 域独立 Lead（per 2026-08-21 JST 拍板）|
| WBS | 5 层任务工时校准（Q-031 待 SRE Lead + PM 校准）|
| リスク | P0/P1/P2 优化点 + known gap（per §9）|

## 7.3 SEC-BOK 横展开规则

| 横展开轴 | 落地 |
|---|---|
| データ設計 | 本文档（Work / Transaction / Master 三分类）|
| ソフトウェア設計 | per RGS-BAS-001 业务逻辑 + RGS-BAS-002 功能架构 |
| テスト | UT（per 8/31 JST，5 域独立）/ IT（同）/ ST（per 9/1 12 域 mTLS ST 已完成）|

## 7.4 其他横展开内容（per Ulysses 2026-09-01 18:30 JST）

> "其他横展内容遵循日本 IPA SEC 规则" — 适用于：
> - 数据横展开（本文档）
> - 功能横展开（per RGS-BAS-001 业务逻辑层）
> - 部署横展开（per RGS-OPS-001 部署运营）
> - 监控横展开（per RGS-BAS-004 运维日志规范基本设计书 §4.8.3 二维矩阵）
> - 安全横展开（per RGS-BAS-006 网络安全基本设计书）
> - 性能横展开（per [17-不合理设计](../15-IPA-DB表设计书/17-不合理设计识别与优化建议.md) + [00-总览](../15-IPA-DB表设计书/00-目录与全貌清单.md)）

---

# 8. 本功能日志设计

> 落实 Ulysses 2026-09-01 15:52 JST 决策（"各 BAS 文档功能章节加 log 设计且区分 debug/release 级"）+ RGS-BAS-007 v0.3 §1.1 模板（commit `bf52973`）+ v0.2 增量 2 字段（cleanup job + LCM step）。

本节覆盖"本文档**作为治理基准**被分域 RGS-DTL 物理 DDL 章节引用时"的观察点——本文档不直接产生业务 SQL 执行事件，但**作为横展开三分类标准发布、修订评审、分域遵循判定**的基线，需要追踪"何时被谁遵循/偏离"。

| 字段名（field） | 触发条件（trigger） | 频率估算（frequency） | 采样策略（sampling） | 脱敏与成本（redact & cost） |
|---|---|---|---|---|
| `db.bas_dbb_001.published` | RGS-DB-BAS-001 新版本经审批后正式发布（如 v0.1 → v0.2 升版 = 本次）| 1/季度 | release 必出（100% 强制全采样，per RGS-BAS-004 v0.3 §6.2）| 含 `version` / `effective_at` / `approver_id`；约 250B/条 × 1/季度 = 极低 |
| `db.bas_dbb_001.category_assignment.received` | 任意分域 RGS-DTL 物理 DDL 章节提出"表分类（Work/Transaction/Master）"的修订申请 | 1/月 | release 必出（100% 强制全采样）| 含 `dtl_doc_id` / `table_name` / `proposed_category` / `requester_id`；约 350B/条 |
| `db.bas_dbb_001.category_assignment.approved` | 架构师评审通过分类修订申请（含 ADR 编号）| 1/月 | release 必出（100% 强制全采样）| 含 `dtl_doc_id` / `table_name` / `category` / `adr_id` / `approver_id`；约 320B/条 |
| `db.bas_dbb_001.category_assignment.rejected` | 分类修订申请被驳回（须重新论证）| 偶发 | release 必出（100% 强制全采样）| 含 `dtl_doc_id` / `table_name` / `reason` / `rejector_id`；约 280B/条 |
| `db.bas_dbb_001.three_category_drift.detected` | 自动化扫描发现分域 RGS-DTL 表的 `lifecycle_policy` 与本文档三分类不一致 | 1/季度（季度评审）| release 必出（100% 强制全采样）| 含 `dtl_doc_id` / `table_name` / `expected_category` / `actual_lifecycle`；约 400B/条 |
| `db.bas_dbb_001.cross_domain_fk.detected` | 自动化扫描发现分域 RGS-DTL 表物化了跨域 FK（违反 §6.1）| 1/季度 | release 必出（100% 强制全采样）| 含 `dtl_doc_id` / `table_name` / `fk_target` / `db_name`；约 320B/条 |
| **`db.bas_dbb_001.work_table_cleanup.detected_drift`**（**v0.2 新增**）| 自动化扫描发现 Work 表 24h 内 cleanup job 未运行或 deleted_count=0 | 1/日 | release 必出（100% 强制全采样）| 含 `table_name` / `last_cleanup_at` / `deleted_count` / `expected_deleted_count`；约 350B/条 |
| **`db.bas_dbb_001.lcm_step_execution.drift`**（**v0.2 新增**）| 季度评审发现 `lcm_step_execution` Work 表未实装或分类未确认（per §6.6.2）| 1/季度 | release 必出（100% 强制全采样）| 含 `lcm_step_table_exists` / `classification_confirmed` / `decision_ref`；约 300B/条 |
| `db.bas_dbb_001.debug.compliance_diff_dump` | 分域 RGS-DTL 表与本文档三分类的逐行 diff（用于人工复审）| 1/季度 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除）| 约 2-10KB/条（依赖文档长度，release 剔除）|
| `db.bas_dbb_001.debug.category_assignment_full_payload` | 分类修订申请的完整 payload（含敏感 ADR 草案，**仅** debug-only 守护）| 1/月 | **debug-only**（`#[cfg(debug_assertions)]` 守护，release build 完全剔除）| 约 1-3KB/条（release 剔除，避免 RUST_LOG=debug 误开泄漏）|

**debug-only 守护要点**（落实 RGS-BAS-004 v0.3 §4.4）：
- `db.bas_dbb_001.debug.category_assignment_full_payload` 可能含 ADR 全文 draft —— release build 完全剔除，避免 RUST_LOG=debug 误开时未发布 ADR 草案泄漏
- `db.bas_dbb_001.*` 系列均为 `info!` 级别（release 必出，per RGS-BAS-004 v0.3 §4.8.3.2 二维矩阵 `info!` 行常驻），便于 DBA 团队按 `dtl_doc_id` 维度追溯三分类符合性

**数据库域特殊强制**（per RGS-BAS-007 v0.3 §6.2 强制全采样白名单）：
- `db.bas_dbb_001.three_category_drift.detected` / `db.bas_dbb_001.cross_domain_fk.detected` = release 必出
- `db.bas_dbb_001.work_table_cleanup.detected_drift` / `db.bas_dbb_001.lcm_step_execution.drift`（v0.2 新增） = release 必出
- 用于季度治理评审自动发现"文档与代码漂移"

---

# 9. 已知缺口 / 待业务确认（v0.2 状态更新）

> **缺标比错标安全**（per 2026-08-26 RGS-DTL-036 v1.4 hotfix 复盘）。本节显式列出"已知缺口"，避免假装覆盖。v0.2 状态更新：5 项已解决文档层（9.1/9.2/9.3/9.4/9.5），1 项已 commit 解决（9.6），1 项保留 DDD Review v0.2 待办（9.7），1 项标记"不适用"（9.8）。

| # | 缺口 | v0.1 状态 | v0.2 状态 | 关联 commit / commit 计划 | 解决方式 |
|---|---|---|---|---|---|
| 9.1 | Social 域 Work 表覆盖薄弱（`invitations` / `pending_join_requests` 等可能 Work 表）| 🟡 业务缺口，PH-2 评审 | 🟡 业务缺口，**PH-6 待补** | 本版本 v0.2 §3.5 | §3.5 列出 PH6-S-01~05 5 个候选表 + 业务来源 + cleanup SOP 引用，PH-6 实施路径明确 |
| 9.2 | Admin/LCM 短期 step execution 表归类未明 | 🟡 设计归类待 admin Lead 拍板 | 🟢 **文档层解决** + 🟡 实施待 admin Lead 拍板 | 本版本 v0.2 §6.6 | §6.6.1 `realm_lifecycle_run` 改归 Transaction（T-01）+ §6.6.2 `lcm_step_execution` Work 表 schema 草案 + 4 项 admin Lead 拍板问题 |
| 9.3 | Economy 域 `auctions` / `private_trades` 寿命归 Work 合理但缺 cleanup | 🟡 实施缺口 | 🟢 **完全解决** | commit `2264011` | 14-§7 Work 表 cleanup SOP 落地（含 auctions/private_trades SQL 模板 + 监控点 + 已知缺口）|
| 9.4 | `transaction_ledger` / `sagas` / `moves` 分区未实施 | 🟡 实施缺口 | 🟢 **文档层解决** + 🟡 PH-3 实施 | 本版本 v0.2 §3.2 T-02~T-04 + §6.5 | SQL 模板 per [17-P0-02/P0-03](../15-IPA-DB表设计书/17-不合理设计识别与优化建议.md) + PH-3 实施计划 + RGS-BAS-007 §4 分区滚动 SOP |
| 9.5 | LCM 6 表共置 admin_db 与 "5 独立 DB" 表述冲突 | 🔴 文档不一致 | 🟢 **完全解决** | commit `38cc2f8` | 17-P0-01 拆分为 P0-01a (ARC-008 表述歧义, doc fix 5 项 ⏳ PH-2) + P0-01b (LCM 6 表共置 = FR-LCM-001 设计选择, ✅ 非 violation) |
| 9.6 | 15-IPA-DB表设计书/ 18 个文件 git status `??` 未 commit | 🔴 git 状态 | 🟢 **完全解决** | commit `215cdb4` | 19 files / 5094 insertions 落地 |
| 9.7 | RGS-DB-BAS-001 v0.1 待 5 域 Lead 一审 | 🟡 协调 | 🟡 **保留 DDD Review v0.2 待办** | — | 5 域 Lead 一审：player / economy / match / social / admin 对 §3-§4 三分类映射拍板；特别是 9.1 (Social Work PH-6) + 9.2 (LCM step execution 拍板) + 9.3 (auctions 寿命归类已确认) |
| 9.8 | 横展开三分类原则是否扩展到 Physis 物理引擎 | 🟡 跨项目决策 | ⏸️ **不适用** | — | per 2026-09-01 21:16 JST Ulysses 拍板 "物理引擎并不包含在 RGS 项目"——Physis 不在 RGS scope 内，本原则不适用；后续如要集成存存储快照再评估 |

---

# 10. 引用关系图

```
RGS-REQ-011 (DB 設計 需求定义書)
    ↓ ARC-023
RGS-BAS-007 (DB 設計標準 基本設計書 v0.3)
    ↓ 三分类横展开
RGS-DB-BAS-001 v0.2 (本文档) ← 你正在读 (commit <pending>)
    ↓ 12 库 / 42 表 / 5 域 + 7 工具域
docs/15-IPA-DB表设计书/  (18 个 detail 文件, commit 215cdb4)
    ├── 00-目录与全貌清单.md          (全表清单)
    ├── 01-IPA软件工程文档编制标准.md   (命名/列属性標準)
    ├── 02-Player域_player_db.md       (6 表)
    ├── 03-Economy域_economy_db.md     (8 表)
    ├── 04-Match域_match_db.md         (7 表)
    ├── 05-Social域_social_db.md       (3 表)
    ├── 06-Admin域_admin_db.md         (9 表, 含 LCM 6 表)
    ├── 07-ClusterOps域_cluster_ops_db.md (3 表)
    ├── 08-Card域_card_db.md           (3 表)
    ├── 09-I18n域_i18n_db.md           (2 表)
    ├── 10-Leaderboard域_leaderboard_db.md (1 表)
    ├── 11-Replay域_replay_db.md       (1 表)
    ├── 12-AssetDownload域_downloads_sqlite.md (1 表)
    ├── 13-Outbox跨域模板.md            (6 域 outbox 共享模板)
    ├── 14-分区策略与生命周期.md         (审计日志 / outbox / LCM 月度分区 + §7 Work 表 cleanup SOP — commit 2264011)
    ├── 15-跨域引用与一致性约束.md       (不物化 FK 原则 + 应用层校验)
    ├── 16-IPA标准化检查清单.md          (命名 / 索引 / 分区 / 制約 / FK)
    └── 17-不合理设计识别与优化建议.md   (P0/P1/P2 优化点 + P0-01a/P0-01b 拆分 — commit 38cc2f8)
```

**RGS-DB-BAS-001 ↔ 15-IPA-DB表设计书/ 关系**：
- **本文档 = 横轴**（三分类原则 + 跨域映射）
- **15-IPA-DB表设计书/ = 纵轴**（每张表的 column 级别 detail）
- 两者合起来 = RGS 全部 42 张表的"标准 + 三分类 + detail" 三层文档体系

---

# 11. 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订人 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-01 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版制定（commit `c11fa4d`）| 全部 |
| **0.2** | 2026-09-01 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 解决 v0.1 §9 已知缺口 9.1/9.2/9.3/9.4/9.5/9.6 文档层：① §1.1 v0.2 增量表（5 项缺口 → 解决方式 + 关联 commit）② §3.1 Master 21 张（v0.2 调整：拆 cards + 加 languages + cluster_nodes/feature_flags；删 realm_lifecycle_run 改 T-01）③ §3.2 Transaction 12 张（含 realm_lifecycle_run 从 Master 改 T-01）④ §3.3 Work 表 8 张 + W-06/W-07 cleanup SOP 引用（commit 2264011）⑤ §3.4 工具域补充 1 张（总表数 42 不变）⑥ §3.5 PH-6 待补 Work 表 5 候选（解决 9.1）⑦ §4.4 Social 域加 PH-6 标注（解决 9.1）⑧ §4.5 Admin 域 LCM realm_lifecycle_run 改 Transaction 标注（解决 9.2）⑨ §6.6 LCM step execution 归类说明 2 子节（解决 9.2）⑩ §8 加 2 字段（cleanup drift + LCM step drift）⑪ §9 8 项缺口状态更新（6 项已解决 + 1 项 PH-6 实施 + 1 项"不适用"）⑫ §10 引用关系图加 2 commit 引用 ⑬ §11 本修订历史条目 | §1.1 / §3.1-§3.5 / §4.4-§4.5 / §6.6 / §8 / §9 / §10 / §11 |

**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
**审批**：架构师（Mavis 接手 agent per DEC-008）+ 自审 + 2026-09-01
**代签授权依据**：2026-08-27 19:39 / 20:56 / 21:59 JST Ulysses 三次强化"你可以代签" + 2026-09-01 18:30 JST "DB 三分类横展开原则"拍板 + 2026-09-01 21:16 JST 缺口 review 拍板 opt1（①+②+③ 三件）

---

> 本文档与 RGS-BAS-007（DB 設計標準 基本設計書）+ RGS-REQ-011（DB 設計 需求定义書）+ `docs/15-IPA-DB表设计书/`（12 域 detail 表设计書）共同构成 RGS 数据库"需求 + 标准 + 三分类横展 + 域 detail" 四层文档体系。
