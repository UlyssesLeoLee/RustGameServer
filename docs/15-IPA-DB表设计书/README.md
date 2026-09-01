# 数据库表设计书（IPA 標準準拠 / Database Table Design Specification — JIS X 0123 / IPA SLCP-JCF2013 準拠）

> **本文件夹定位**：RustGameServer 仓库全部 PostgreSQL / SQLite 数据库表的**物理表设计书（テーブル定義書）**，按日本 IPA『共通フレーム 2013（SLCP-JCF2013）』詳細設計工程 + JIS X 0123 命名規約 + RGS-BAS-007（DB 設計標準）三層基準補齐。
>
> 全部表覆盖（**42 张表 / 12 个库 / 6 域 + 5 工具域 + 1 异构 SQLite 库**），逐表列出列属性（物理名 / 論理名 / データ型 / 桁数 / PK / FK / NOT NULL / DEFAULT / CHECK / 説明）。

| 项目 | 内容 |
|---|---|
| フォルダ ID | 15-IPA-DB表设计书 |
| 作成日 | 2026-09-01 JST |
| 作成者 | Mavis（Ulysses 一人公司 12 角色 per DEC-008 代理） |
| 適用基準 | IPA『共通フレーム 2013（SLCP-JCF2013）』詳細設計工程 + JIS X 0123 命名規約 + RGS-BAS-007（DB 設計標準） |
| 親文档 | RGS-REQ-011 DB 設計標準 / RGS-BAS-007 DB 設計標準（已存在） |
| 適用範囲 | RGS 仓库全部 34 个 SQL migration 文件，11 个 PostgreSQL DB + 1 个 SQLite DB |
| 非適用範囲 | 部署 / 备份 / 性能调优（属 RGS-OPS-001 / RGS-OPS-100） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响範囲 |
|---|---|---|---|---|---|
| 0.1 | 2026-09-01 | 架构师(Mavis 接手 agent per DEC-008) | 架构师(Mavis 接手 agent per DEC-008) | 初版制定。覆盖 34 个 SQL migration 中全部 42 张表（11 PG 库 + 1 SQLite 库），按 IPA 標準補齐列属性 / 桁数 / 制約 / 索引 / 関連表；识别 7 类不合理设计并给出 P0/P1/P2 优化建議 | 全部 |

> **代签授权依据**：2026-08-27 19:39 / 20:56 / 21:59 JST Ulysses 三次强化"你可以代签"。

---

## 文件夹结构（フォルダ構成 / Folder Layout）

```
15-IPA-DB表设计书/
├── README.md                            ← 本文件（索引 + 总览）
├── 00-总览与全表清单.md                  ← 42 张表全局清单 + 库/域映射 + ER 概览
├── 01-IPA命名与列属性标准.md              ← 物理名/論理名/データ型/桁数 標準化規範
│
├── 02-Player域_player_db.md              ← 5 表（player 主域）
├── 03-Economy域_economy_db.md            ← 7 表（含 saga / 拍卖 / 私下交易）
├── 04-Match域_match_db.md                ← 6 表（matches / game_sessions / moves / 票 / 订阅）
├── 05-Social域_social_db.md              ← 2 表（公会 / 成员）
├── 06-Admin域_admin_db.md                ← 8 表（admin_users / audit_log / LCM 6 表）
├── 07-ClusterOps域_cluster_ops_db.md     ← 2 表（节点 / 特性开关）
│
├── 08-Card域_card_db.md                  ← 3 表（卡 catalog / 系列 / 玩家收藏）
├── 09-I18n域_i18n_db.md                  ← 2 表（多语言文案 / 语言清单）
├── 10-Leaderboard域_leaderboard_db.md    ← 1 表（排行榜条目）
├── 11-Replay域_replay_db.md              ← 1 表（回放元数据）
├── 12-AssetDownload域_downloads_sqlite.md ← 1 表（SQLite 异构）
│
├── 13-Outbox跨域模板.md                  ← 6 域 outbox 共享模板（避免重複定義）
├── 14-分区策略与生命周期.md               ← 审计日志 / outbox / LCM 月度分区
├── 15-跨域引用与一致性约束.md             ← 不物化 FK 原则 + 应用层校验责任
│
├── 16-IPA标准化检查清单.md                ← 命名 / 索引 / 分区 / 制約 / FK 全维度核对表
└── 17-不合理设计识别与优化建议.md         ← P0/P1/P2 級別优化点
```

---

## 库/域 全局映射（Database / Domain Mapping）

| # | 物理库名 | ドメイン | 担当 crate | 引擎 | 表数 | 设计書 |
|---|---|---|---|---|---|---|
| 1 | `player_db` | Player 域 | `player-service` | PostgreSQL 18 | 6（含 outbox） | [02](02-Player域_player_db.md) |
| 2 | `economy_db` | Economy 域 | `economy-service` | PostgreSQL 18 | 8（含 outbox） | [03](03-Economy域_economy_db.md) |
| 3 | `match_db` | Match 域 | `match-service` | PostgreSQL 18 | 7（含 outbox） | [04](04-Match域_match_db.md) |
| 4 | `social_db` | Social 域 | `social-service` | PostgreSQL 18 | 3（含 outbox） | [05](05-Social域_social_db.md) |
| 5 | `admin_db` | Admin 域 + LCM | `admin-service` / `cluster-ops` (LCM 子模块) | PostgreSQL 18 | 9（含 outbox + 6 LCM 表） | [06](06-Admin域_admin_db.md) |
| 6 | `cluster_ops_db` | ClusterOps 域（节点 / 特性） | `cluster-ops` | PostgreSQL 18 | 3（含 outbox） | [07](07-ClusterOps域_cluster_ops_db.md) |
| 7 | `card_db` | Card 域 | `card-service` | PostgreSQL 18 | 3 | [08](08-Card域_card_db.md) |
| 8 | `i18n_db` | I18n 域 | `i18n-service` | PostgreSQL 18 | 2 | [09](09-I18n域_i18n_db.md) |
| 9 | `leaderboard_db` | Leaderboard 域 | `leaderboard-service` | PostgreSQL 18 | 1 | [10](10-Leaderboard域_leaderboard_db.md) |
| 10 | `replay_db` | Replay 域 | `replay-service` | PostgreSQL 18 | 1 | [11](11-Replay域_replay_db.md) |
| 11 | `downloads.sqlite` | AssetDownload 工具 | `rgs-asset-download` | **SQLite 3** | 1 | [12](12-AssetDownload域_downloads_sqlite.md) |
| — | **合計** | — | — | — | **42 张表** | — |

> **关键说明（per FR-LCM-001）**：LCM（服务器全生命周期管理）6 表归 **admin_db**，**不**新建独立数据库——打破 ARC-008 "5 独立 DB 原则" 的既定描述（实际为 6+ 独立 DB 原则），见 [00-总览与全表清单](00-总览与全表清单.md) §LCM 归属说明 + [17-不合理设计](17-不合理设计识别与优化建议.md) §P0-01。

---

## 阅读指南（読み方）

1. **先读 [00-总览](00-总览与全表清单.md)**：掌握 42 张表的全表清单 / 库映射 / 域归属 / ER 概览
2. **再读 [01-命名与列属性标准](01-IPA命名与列属性标准.md)**：掌握物理名 / 論理名 / データ型 / 桁数 标准化定義（阅读各域表之前的共同語言）
3. **按需跳读各域表设计书**：每个域的设计書是**自包含的**（可独立阅读，不依赖其他域）
4. **最后读 [17-不合理设计](17-不合理设计识别与优化建议.md)**：7 类不合理设计 + 14 项 P0/P1/P2 优化建議

---

## 引用规范

本文档中所有表名 / 列名 / 文件:行号 引用均来自：

| 类型 | 根路径 |
|---|---|
| SQL migration | `crates/<crate>/migrations/*.sql` |
| 既有 DTL | `docs/01-核心架构与设计模式/RGS-DTL-NNN_*.md` |
| 既有 BAS | `docs/01-核心架构与设计模式/RGS-BAS-NNN_*.md` |
| DB 設計標準 | `docs/03-数据经济与交易/RGS-BAS-007_*.md` + `RGS-REQ-011_*.md` |

> **缺标比错标安全**（per 2026-08-26 RGS-DTL-036 v1.4 hotfix 复盘）：若发现实际 schema 与本文档不一致，**以 migrations 实际 SQL 为准**，并在对应表设计書的「既知偏差 / Known Drift」段记录。

---

> 本文件夹与 RGS-BAS-007（DB 設計標準）+ RGS-REQ-011（DB 設計 需求定义書）共同构成 RGS 数据库"标准 + 设计 + 详细" 三层文档体系。本文件夹不重复 BAS-007/REQ-011 的"标准"部分，只展开"各表详细设计 + 不合理设计识别"层。
