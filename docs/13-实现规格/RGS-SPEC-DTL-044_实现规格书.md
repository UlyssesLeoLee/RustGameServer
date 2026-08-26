# RGS-DTL-044 实现规格书

**RGS-SPEC-DTL-044**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-SPEC-DTL-044 |
| 版本 | 0.2 |
| 状态 | 规格草案 + 已知缺口(见 §A.3)，待 RGS-DTL-044 具名 DD Review |
| 源详细设计 | RGS-DTL-044(本 DTL 今日未升版,SPEC v0.2 为前瞻性草案,见 §A.1) |
| 实现范围 | `player-service` / `player_db`：`0004_player_characters_inventory.sql` migration（`player_characters` + `player_inventory` 新建）+ `players` 表反向 doc 对齐（不修改 0001） |
| 目标基线 | Rust 1.98 stable（当前基线；环境/CI Gate）、PostgreSQL 14+、sqlx（沿用 `player-service` 既有依赖，不新增） |
| 规格真源 | 源 DTL 的字段级 DDL、CHECK 约束、FK 级联规则、索引规划、§3 反范式禁令清单 |

---

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | Ulysses(架构师兼 / player 域 Lead兼 per DEC-008) | 2026-08-25 | 一人公司 12 角色兼任 |
| 评审（技术/架构） | Ulysses(架构师兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（平台/客户端/SRE/DBA/安全/合规/法务） | Ulysses(对应角色兼 per DEC-008) | 2026-08-25 | DEC-008 |
| 评审（运营） | Ulysses(运营兼 per DEC-008) | 2026-08-25 | 仅适用全生命周期文档 |
| **集体签字(per DEC-008)** | **Ulysses(一人公司 12 角色兼任)** | **2026-08-25** | **Ulysses 在审批栏各角色中具名签字,完整 12 角色兼任清单见 RGS-WBS-001 §17 集体签字声明。审批栏细化角色意见详见 RGS-REQ-004 §3.10。** |

---

## 修订历史(Revision History)

| 版本 | 修订人 | 修订日 | 审批者 | 修订内容 | 影响小节 |
|---|---|---|---|---|---|
| 0.2 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）) | 对齐源 DTL-044 当前版本(`0.1`) + 头表 0.2 + 新增 §A v0.2 对齐说明;**不引入新设计**;**代签已允许**(per 2026-08-26 08:40 JST 偏好反转);本 SPEC 为 17 份未升版 DTL 前瞻性 v0.2 草案之一(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2) | §A(新增) |

---

## 1. 使用规则

本规格把 RGS-DTL-044 从详细设计转成可执行的实现清单，不替代源 DTL。若本规格与 RGS-DTL-044 不一致，以 DTL 评审变更为准；不得在代码中自行调和冲突。当前工作区没有对应实现源码，本文件不代表功能已完成。

必须实现 `0004_player_characters_inventory.sql`（`player_characters` + `player_inventory` 全部字段/CHECK/FK/索引）；不得修改 `0001_init.sql` 已有 DDL（历史 migration 不可变，RGS-IMPL-001 §3.2）；不得在 `players.metadata`（未来 0005，本规格不覆盖其实施）或任何 JSONB 字段写入 DTL §3.1 反范式禁令清单中的字段。

## 2. 实现单元

| 类型 | 计划路径 | 要求 |
|---|---|---|
| DB migration | `crates/player-service/migrations/0004_player_characters_inventory.sql` | `player_characters`/`player_inventory` 全部字段、CHECK、FK、UNIQUE 与 DTL §2.2/§2.4 逐条一致 |
| 索引 | 同一 migration 内 | 9 个二级索引（DTL §4.2/§4.3）全部落地，含 2 个 GIN 索引（`stats`/`metadata`） |
| Entity（不在本任务范围） | `crates/player-service/src/entity.rs` 新增 `PlayerCharacter`/`PlayerInventory` struct | DTL §1.1 已明确排除，本规格仅登记依赖，不要求本轮实现 |
| 反向 doc 校验 | CI 脚本或人工 diff：`0001_init.sql` 现有 `players`/`player_sessions` DDL 与 DTL §5.2/§5.3 字段级对照表逐行比对 | 确认无静默漂移；发现不一致须先修订 DTL 而非静默改代码 |
| CI | fmt、clippy、test（含 migration up/down）、deny、schema、secret checks | 负例必须阻断合并 |

## 3. 实现契约

- `player_characters.player_id`/`player_inventory.player_id` 外键**必须**为 `ON DELETE CASCADE`（玩家删除全清角色/背包，DTL §2.2/§2.4）。
- `player_characters.primary_weapon_id` 外键**必须**为 `ON DELETE SET NULL`（武器删除角色保留），不得写成 CASCADE（DTL §2.2 复合约束理由）。
- `player_inventory` 的 `(player_id, slot)` UNIQUE 约束**必须**存在，禁止应用层用"先查空槽位再插入"绕过（防重入 bug，DTL §2.4）。
- `player_characters.hp`/`atk`/`def`/`crit_rate` 等高频属性**必须**字段化拆列，**不得**下沉到 `stats` JSONB（DTL §2.3 拆分原则），违反视为规格违反。
- `player_inventory.item_id` **不得**物化为 FK（物品 master 表归其他域，DTL §2.4"为什么不物化"）；应用层需自行做存在性校验。
- `players.metadata`（未来 0005）以及 `player_characters.stats`/`player_inventory.metadata` 的 JSONB 写入路径，**禁止**出现 DTL §3.1 反范式禁令清单中的任一字段名（`equipment_json`/`characters_json`/`inventory_json`/`sessions_json`/`idp_links_json`/`compliance_json`/`vault_json`/`audit_log_json`）；CI lint 或 code review 须 grep 校验。
- `0001_init.sql` 本身**不得**在本任务中修改；`players` 表的 `level`/`vip_level` CHECK 范围约束若要补强，须走独立 0005 migration，不在本规格范围内实施。

## 4. 可观测性规格

- 业务代码只能调用统一 observability façade；禁止直接调用裸 OTel、tracing、log。
- 指标（`player_inventory_*`/`player_character_*`）：角色创建计数、背包写入计数（按 slot 冲突拒绝次数）、`stats`/`metadata` JSONB 写入字节量（防止 JSONB 膨胀失控）。
- 指标标签：仅 `char_class`/`item_id`（物品模板 ID，非玩家可识别信息）等低基数标签；`player_id` **不**作为 metric label。
- 慢查询告警：`idx_pc_stats_gin`/`idx_pi_metadata_gin` 两个 GIN 索引路径查询延迟需纳入既有慢查询监控范围。

## 5. 安全、容错与发布

| 领域 | 必须验证 |
|---|---|
| 数据完整性 | `player_inventory.quantity > 0`、`slot` 范围 0..199、`player_characters` 数值属性非负，全部 CHECK 约束在 migration 中生效（DB 层强制，不依赖应用层） |
| 反范式合规 | §3 契约禁止清单的 grep 校验纳入 CI 或 PR checklist，任何触碰 `players.metadata`/`stats`/`player_inventory.metadata` 写入路径的 PR 必须过审 |
| 迁移安全 | `0004` migration 是纯新增（`CREATE TABLE IF NOT EXISTS`），不改动既有表；`down` 迁移（如需要）仅 `DROP TABLE` 新建的两张表 |
| 级联行为 | 玩家删除级联清角色/背包的行为需在 staging 环境验证（避免生产误删导致不可逆数据丢失） |
| 发布 | migration 上线前需在非生产环境验证 up/down 均可执行；FK 索引（`idx_pc_player_id`/`idx_pc_weapon`/`idx_pi_player_id`）缺失会导致 CASCADE 删除全表扫描，上线前必须确认索引已生效 |

## 6. 测试规格

- UT：覆盖 `player_characters`/`player_inventory` 全部 CHECK 约束（数值非负、slot 范围、quantity>0）+ `(player_id, slot)` UNIQUE 冲突路径 + `primary_weapon_id` SET NULL 行为。
- IT：覆盖玩家删除级联清理（`players` → `player_characters`/`player_inventory` 全清）+ 武器删除仅清 `primary_weapon_id`（角色不被删除）+ `stats`/`metadata` GIN 索引查询正确性。
- Migration 测试：`0004` up/down 幂等性（重复执行 `up` 不报错，`IF NOT EXISTS` 生效）；`0001_init.sql` 内容 diff 校验（确保未被本任务误改）。
- Security：grep 验证 `players.metadata`/`player_characters.stats`/`player_inventory.metadata` 三处 JSONB 写入代码路径不含 §3.1 禁止字段名。
- Performance：`idx_pc_class_level` 复合索引最左前缀查询（DTL §4.2 示例）执行计划验证走索引而非全表扫描。

测试必须回填 RGS-REQ-004 追踪矩阵和 DTL-044 §8 追踪矩阵的下游动作项；不能只证明"migration 跑通"。

## 7. Definition of Done

- RGS-DTL-044 §6 签字栏中 #1（player 域 Lead）与 #7（架构师）两项 R/A 角色已签（DTL §6 升版条件，v0.1→v1.0 的最低门槛）。
- `0004_player_characters_inventory.sql` 落地，字段/CHECK/FK/索引与 DTL §2.2/§2.4/§4 逐项对账；`0001_init.sql` 未被修改（diff 为空）。
- Cargo fmt、clippy、test（含 migration 测试）、deny、schema、secret 检查通过。
- §3 反范式禁令 grep 校验通过（无禁止字段名出现在任何 JSONB 写入路径）。
- 当前无实现文件时保持"待实现/待评审"，不得标记生产完成。

## 8. Gate 证据与实测参数

进入实现前必须取得：① 源 DTL RGS-DTL-044 的具名 DD Review（含 §6 签字栏 #1/#7 签字）；② `0004` migration 在非生产环境 up/down 演练通过；③ `0001_init.sql` 未变更的 diff 证据。**本规格不覆盖**：`players.metadata` 列（未来 0005 migration）的实施——DTL §2.1.1 已明确标注"待未来 0005 migration，本任务不实施"，本规格仅登记为下游依赖（DTL §8.3），不作为本轮 Gate 证据；`player_characters`/`player_inventory` 的 Rust entity 化（`entity.rs` 扩展）同样不在本规格 Gate 范围（DTL §1.1 明确排除，per WF-1-55.39 范围约束"只写 SQL + DDL 文档"）。

---

## A. v0.2 对齐说明(2026-08-26,基于源 DTL 今日状态)

> **本节定位**:本 SPEC v0.2 是 17 份"未升版 DTL"的前瞻性 v0.2 草案(per RGS-OPEN-QA-2026-08-26-SPEC-v0.2 Q2)。**不引入新设计**——仅落实/复核源 DTL 当前版本 + 父 BAS 既有内容;正文本 §1~§8 不重写,新增内容仅本节。

### A.1 源 DTL 今日升版增量(前瞻性视角)

- **源 DTL**:RGS-DTL-044
- **源 DTL 今日状态**:`0.1`(`2026-08-24`)
- **源 DTL 升版路径**:**今日未升版**(`git log --since="2026-08-26 00:00" --until="2026-08-26 23:59" -- docs/**/RGS-DTL-044_*.md` 无 commit)
- **源 DTL 升版类型**:**前瞻性草案**(非"今日升版沉淀")
- **核心要点**:源 DTL-044 v0.1(2026-08-24)为 player 域主表 DDL 首版——`players`/`player_characters`/`player_inventory` 3 张表字段级 DDL + 反向 doc `0001_init.sql` + 新建 `0004_player_characters_inventory.sql` migration;per RGS-OPEN-QA-001 v0.2 Q-D-02 + ACTIONS-v0.3 A-02 偿还;状态标注 v1.0 DTL 实体首版(A-02 偿还技术债,per DTL-018 §2 + DTL-036 §6 第 1 条)

### A.2 对本 SPEC 的影响(实现侧)

| 维度 | v0.1 | v0.2 调整 |
|---|---|---|
| 实现范围 | 与源 DTL v0.1 同步 | 与源 DTL `0.1` 同步(范围不变,仅元数据对齐) |
| 源 DTL 真源 | RGS-DTL-044 v0.1 | RGS-DTL-044 `0.1`(具体修订见 §A.1) |
| §7 DoD 状态 | 待源 DTL 具名 DD Review | 仍待源 DTL 具名 DD Review(本 SPEC v0.2 不阻塞) |
| §8 Gate 证据 | 待 ①源 DTL DD Review ② Rust 1.98 stable CI ③ PostgreSQL 18.6 迁移演练 ④ K3s 能力核验 | 同 v0.1(本前瞻性草案不新增 Gate) |

### A.3 已知缺口 / 待 DDD Review 必查项

> 缺标比错标更安全(per DTL-036 v1.4.1 hotfix 复盘 §修式)。本节列出来源 DTL 升版自身声明的待办 / 缺口,本 SPEC 不预设处置方案,待 DDD Review 阶段配套决策。

- 源 DTL 升版内容仅为 前瞻性草案(本 DTL 今日未升版)时,本节无新缺口继承。
- 若源 DTL 升版伴随 §3 已知缺口清单(如 RGS-DTL-036 v1.4.2 §3 末 5 项),则对应缺口必须由 DDD Review 阶段与父 BAS / 上位 REQ 逐条对账,本 SPEC §3 / §4 / §6 / §7 不得在缺口未处置前标 Done。
- 本 SPEC v0.2 自审 0 项发现,**等 DDD Review 阶段再行检查**。

### A.4 引用链与证据

- 源 DTL 修订历史条目:见 RGS-DTL-044 §修订历史表(本 DTL 今日未升版,引用最新一次历史升版)
- 父 BAS 升版条目:见对应父 RGS-BAS-NNN §修订历史表(本 DTL 对应父 BAS,本日是否升版需自审)
- 同期 SPEC 调整总报告:[RGS-SPEC-000 详细设计规格化总表 v0.3](../RGS-SPEC-000_详细设计规格化总表.md) + RGS-REPORT-2026-08-26-17-SPEC-Update-v0.2_v0.1.md(17 份前瞻性 SPEC v0.2 同批)
- **代签已允许**(per 2026-08-26 08:40 JST 偏好反转):本节"审批者"列 = 真实责任署名 "架构师(Ulysses（一人公司 12 角色 per DEC-008）)",**不**再受"审批者 = —"硬约束(原占位状态见 git 历史)

> **本 v0.2 调整严格遵循**:① 不引入新设计 ② 不重写正文本 §1~§8 ③ 不动父 BAS / 上位 REQ ④ 代签已允许(新规则) ⑤ 缺标比错标更安全(per DTL-036 hotfix 复盘修式)。
