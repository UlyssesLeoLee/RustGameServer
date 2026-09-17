## 0. 本文定位

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PM-ULYS-1-GAP-AUDIT-V2 |
| 版本 | 0.2 |
| 父文档 | RGS-PLAN-003 REQ→BAS 同步对账（方法依据） |
| 制定日 | 2026-09-12 |
| 制定者 | ULYS-1 承接代理（Opus1m） |
| 状态 | 🟡 草案（待 Ulysses 拍板） |

ULYS-1 议题原问：**「查找是否有没有实施的设计，以及是否存在没有推进到详细设计的需求或者基本设计」**。

2026-09-11 14:16 已产出一版差距审计报告；同日 21:22 任务审核（terra）指出该报告的**配对方法无效**并列出 4 条质疑。本文是「按照审核结果优化」的交付物：用 `RGS-PLAN-003 §0` 规定的正确方法重新推导结论，并逐条复核那 4 条质疑。

**本文只修正结论与方法，不新增设计文档。**

---

## 1. 方法更正

### 1.1 前次报告的方法缺陷

前次报告按 `RGS-REQ-NNN ↔ RGS-BAS-NNN ↔ RGS-DTL-NNN` 的**文件名编号机械配对**判定链路贯通，据此得出「所有领域链路 4 段都已贯通，唯一缺口是 REQ-038」。

`docs/12-工作流/RGS-PLAN-003_REQ-BAS同步对账_2026-08-25_v0.1.md:19` 已明确否定该方法：

> 之前用文件名编号配对是错的：RGS-REQ-NNN 与 RGS-BAS-NNN 的 NNN 不一一对应（BAS 编号连续，REQ 编号含附件 addendum 等）。正确配对方法：读 BAS 头部「父文档」行，反向索引到 REQ。

编号配对在本仓库会同时产生**假贯通**（编号恰好撞上，实际父子关系不同）与**假缺口**（父文档行存在但编号不同构）。`RGS-PLAN-003 §2.6` 记录的 BAS-100「ORPHAN」误判即后者的实例。

### 1.2 落地：`scripts/reqbas_audit.py`

`RGS-PLAN-003 §0` 以伪代码形式给出了正确算法，`§4.2` 提议落地到 `scripts/`。本轮已实现并提交：

- 扫描 `docs/**`（排除 `_archive`）下 REQ / BAS / DTL / SPEC 四层文档；
- 解析头部表格的「父文档」行（兼容 父文档 / 父文書 / 親文档 / 上位文档 等写法），正则抽取上位文档编号，建立**反向索引**；
- **把「解析失败」与「真实缺口」分成独立分桶**（`parsed` / `parsed-fallback-body` / `no-parent-row` / `unrecognized-parent-format` / `declared-root`）——这是避免重犯 BAS-100 类误判的关键；
- 额外输出：重复文档编号、文件名编号与头部编号不一致、悬空父引用。

按 `RGS-PLAN-003 §4.2`，**未**接入 CI（该动作待 Ulysses 拍板）。按 `RGS-IMPL-001 §1.3`，该脚本是文档治理工具，非业务 Rust / SQL migration / Helm 制品，不触及 G-CODE 门禁。

复现命令：

```bash
python scripts/reqbas_audit.py --docs docs --md report.md --json report.json
```

---

## 2. 机械对账结果（2026-09-12）

### 2.1 文档总数

| 层级 | 份数 |
|---|---:|
| REQ | 45 |
| BAS | 37 |
| DTL | 45 |
| SPEC | 43 |

### 2.2 父文档解析分桶

| 分桶 | 份数 | 含义 |
|---|---:|---|
| `parsed` | 114 | 头部「父文档」行成功解析出上位编号 |
| `parsed-fallback-body` | 43 | SPEC 层无「父文档」行，按正文首个 `RGS-DTL-NNN` 回退绑定 |
| `no-parent-row` | 13 | **无「父文档」行 —— 解析失败，不得判为缺口** |

`no-parent-row` 的 13 份（本轮全部人工复核）：

| 文件 | 复核结论 |
|---|---|
| `docs/00-基准与治理/RGS-REQ-001_需求定义书.md` | L1 顶层需求，本就无父 |
| `docs/00-基准与治理/requirements/RGS-REQ-100_Saga事务系统需求定义书_v0.1.md` | L1 独立需求，无父 |
| `docs/01-核心架构与设计模式/RGS-BAS-100_Saga事务系统基本设计书_v0.1.md` | **父为 REQ-100，仅缺头部行** → 假缺口来源 |
| `docs/01-核心架构与设计模式/RGS-DTL-{100,101,102}_*.md` | **父为 BAS-100，仅缺头部行** → 假缺口来源 |
| `docs/00-基准与治理/RGS-REQ-038_卡牌游戏适配_需求定义书.md` | 缺头部行 |
| `docs/00-基准与治理/RGS-DTL-038_卡牌游戏适配_详细设计书.md` | 缺头部行 |
| `docs/02-运维安全与网络/RGS-DTL-044_player主表_v0.1.md` | 缺头部行 |
| `docs/15-IPA-完全对齐438cmds/RGS-REQ-2026-09-04_v0.2.md` | 非 NNN 编号体系，见 §3.2 |
| `docs/00-基准与治理/RGS-DTL-036-REVIEW-2026-08-26-feedback-to-agents.md` | 评审反馈件，非 DTL 正文（文件名误用 DTL 前缀） |
| `docs/14-项目治理/RGS-BAS-{FLOW-STANDARD,MERMAID-VERIFY}-2026-09-02_v0.1.md` | 流程标准件，非领域 BAS（文件名误用 BAS 前缀） |

**结论：13 份 `no-parent-row` 中，0 份是真实设计缺口；全部是头部元数据缺失或文件名前缀误用。**

### 2.3 重复文档编号（新发现，前次报告未系统登记）

| 编号 | 两份文件 | 性质 |
|---|---|---|
| **REQ-038** | `00-基准与治理/RGS-REQ-038_卡牌游戏适配_需求定义书.md`<br>`02-运维安全与网络/RGS-REQ-038_核心传输防丢包强化与周边协议选型_需求定义书.md` | **真实编号冲突**：两份互不相关的需求共用 038 |
| **DTL-038** | `00-基准与治理/RGS-DTL-038_卡牌游戏适配_详细设计书.md`<br>`07-社交运营与玩家治理/RGS-DTL-038_Match域_详细设计书.md` | **真实编号冲突** |
| REQ-007 / 025 / 028 / 030 | 各 1 份本体 + 1 份 `_addendum_` | 正常（addendum 共用主编号，`RGS-PLAN-003 §2.1` 已说明） |

REQ-038 / DTL-038 的编号冲突是**前次报告「REQ-038 缺 BAS」这一结论产生歧义的根因**：两份 REQ-038 中只有「防丢包」那份缺 BAS，「卡牌适配」那份的下游是 DTL-038（卡牌）。任何按编号索引的工具都会把两者混为一谈。

### 2.4 悬空父引用

无。所有被引用的上位文档编号在仓库中均有对应文件。

### 2.5 链路缺口候选（机械输出，需人工复核，非结论）

| 候选 | 复核后定性 |
|---|---|
| REQ-002 / 003 / 004 / 005 | ❌ 非缺口。术语表 / NFR 等级 / 可追溯性矩阵 / 问题风险管理表 4 份附件，`RGS-PLAN-003 §2.5` 已裁定**本就不需要 BAS** |
| REQ-100 | ❌ 非缺口。BAS-100 存在，仅缺头部父文档行（§2.2） |
| REQ-2026-09-04 | ❌ 非本体系缺口，见 §3.2 |
| **REQ-038（防丢包）** | ⚠️ **真实缺口**，但立项时点有争议，见 §3.1 |
| BAS-010 | ❌ 非缺口。设计模式与核心算法总纲，`docs/README.md §1.1` 明示无独立 REQ / DTL |
| BAS-100 | ❌ 非缺口。DTL-100/101/102 存在，仅缺头部父文档行 |
| BAS-FLOW-STANDARD / BAS-MERMAID-VERIFY | ❌ 非缺口。流程标准件，文件名误用 BAS 前缀 |
| DTL-036-REVIEW | ❌ 非缺口。评审反馈件，文件名误用 DTL 前缀 |

**REQ-028 / 029 / 030 未出现在缺口候选中**，与 `RGS-PLAN-003 §2.4`「REQ-028/029/030 对应 BAS-025/026/027，已存在」一致 —— 这正是编号配对会误报、父文档追溯不会误报的实证。

---

## 3. 逐条复核任务审核（terra, 2026-09-11 21:22）的 4 点质疑

### 3.1 质疑 1：不能按编号机械配对 —— ✅ 成立，已按 §1/§2 全面重做

前次报告「所有领域链路 REQ→BAS→DTL→SPEC 4 段都已贯通」这句话**基于无效方法**。按父文档反向索引重算后，**结论方向不变**（除 REQ-038 防丢包外链路确实贯通），但取得该结论的证据链已被替换，且过程中暴露了编号冲突（§2.3）与 13 份元数据缺失（§2.2）两类前次未报的治理缺陷。

### 3.2 质疑 2：`RGS-REQ-2026-09-04_v0.2` 处于「待 Ulysses 二审」，其下游链需纳入核对 —— ✅ 成立，前次报告的定性是错的

前次报告把它一句话打发为「特殊对齐文档，无 BAS/DTL 需求」。实际复核：

`docs/15-IPA-完全对齐438cmds/` 是一条**平行文档族**，采用 REQ → BDD → DDD → TEST-DESIGN → TEST-CASES 命名，而非 REQ → BAS → DTL → SPEC：

```
RGS-REQ-2026-09-04_v0.2.md              （:11 状态：待 Ulysses 二审）
├── RGS-BDD-2026-09-04_v0.2.md          + addendum-frontend适配层
├── RGS-DDD-2026-09-04_v0.2.md          + addendum-业务逻辑逆推 / addendum-协议号映射
├── RGS-TEST-DESIGN-2026-09-07_v0.2.md
└── RGS-TEST-CASES-2026-09-07_v0.2.md
```

因此：

- **它不是「没有推进到基本设计/详细设计的需求」** —— 下游 BDD / DDD 齐备；
- **但它的下游完全游离于 REQ/BAS/DTL/SPEC 可追溯性体系之外**，任何基于 RGS-* 编号的工具（包括本文的脚本）都看不见它。**这才是真正的缺口：治理/可追溯性未整合，而非设计未推进。**

### 3.3 质疑 3：batch 需求书状态表滞后，不应误报为设计缺失 —— ✅ 成立，且问题范围比 terra 指出的更大

`docs/12-工作流/RGS-BATCH-REQUIREMENTS-2026-09-01_v0.1.md:537-541` 仍把三份文档标记为「⏳ 待起草」，但文件均已存在：

| 状态表标记 | 磁盘实际 |
|---|---|
| `RGS-BATCH-BASIC-DESIGN-2026-09-01 v0.1` ⏳ 待起草 | ✅ 存在 |
| `RGS-BATCH-DETAILED-DESIGN-2026-09-01 v0.1` ⏳ 待起草 | ✅ 存在 |
| `RGS-BATCH-PLAN-2026-09-01 v0.1` ⏳ 待起草 | ✅ 存在（且已 v0.2） |

**扩展复核 —— 同类错误在 `docs/document-registry.toml` 中系统性存在**：该注册表 18 条 `[[planned_document]]` 中，**8 条标 `status = "planned"`（待起草）但文件已在磁盘上**：

`RGS-REQ-036`、`RGS-BAS-036`、`RGS-DTL-041`、`RGS-SPEC-DTL-041`、`RGS-REQ-037`、`RGS-BAS-037`、`RGS-DTL-042`、`RGS-SPEC-DTL-042`

其余 9 条（`RGS-TST-{UT,ST,IT}-04-ADD2`、`RGS-TST-{UT,ST,IT}-02-ADD3`、`RGS-IFS-001`、`RGS-TST-001`、`RGS-LIC-001`）确认磁盘无文件，为**真实待起草**；`RGS-DBS-001` 标 `retired`，无文件属正常。

**这是本轮最可操作的治理结论：状态表 44% 的「待起草」是陈旧标记。** 在修正之前，任何以状态表为输入的差距统计都不可信。

### 3.4 质疑 4：已有明确的未实施/受控未落地项 —— ✅ 成立，本轮逐条 grep 验证

`RGS-REQ-2026-09-04_v0.2 §9.4` 列出的条目，代码侧验证结果：

| 条目 | 文档出处 | 代码验证 | 结论 |
|---|---|---|---|
| OpenPack `economy.DebitCurrency` rollback | `:621` | `crates/economy-service/src/trade_service.rs` 多处 `TODO(W36+)`（`:220` / `:338` ExecuteAuction / `:344` FinalizeAuction / `:421` MarkCardUnlocked） | ✅ 未实施，已登记 |
| social migration `0004` DRAFT | `:625` | `crates/social-service/migrations/0004_social_work_tables.sql` 首部 `⚠️ MIGRATION_STATUS: DRAFT 待 PH-6` | ✅ 受控未落地（设计正确状态） |
| social `push_delivery` 0 wire | §9.4 | `push_delivery.rs` 仅 `InMemoryNatsPublisher` / `InMemoryPushDlqRepository`；`social-service/src/main.rs` grep `push` = **0 命中** | ✅ 业务完整但生产未接线 |
| admin `gm_handlers` InMemory fallback | §9.4 | `crates/admin-service/src/gm_handlers.rs`（非 gm-backend）`:16/:44/:217/:262` 确有 InMemory 降级路径 | ✅ 存在，定位需更正到 admin-service |
| PH-6 REQ GAP-1~12 | `:505-516` | 属 v0.2/DDD SPEC/实施计划可追溯性范畴 | ⚠️ 与 RGS-REQ-004 可追溯性矩阵未打通（同 §3.2） |

---

## 4. 修正后的结论（回答 ULYS-1 原问）

### A. 有需求但没有推进到基本设计 —— 1 项

**`RGS-REQ-038` 核心传输防丢包强化与周边协议选型（ARC-047）**，无对应 BAS。

但 `RGS-PLAN-003 §2.6 / §4.3` 已对此裁定：

> 待 REQ-038 v0.2（锚定状态）后再建对应 BAS；**v0.1 阶段不必新建**。

即：**这是一个已知的、被有意推迟的缺口，不是遗漏**。是否立项取决于 PH-1 阶段 ARC-047 的实施授权时点，需 Ulysses 拍板。

> ⚠️ **与前次交付冲突**：2026-09-11 15:17 的「并行完成」回合在分支 `agent/ulys-1/wt-a-bas38` 上**已经新建了 BAS-038 + DTL-038-防丢包**（commit `4e7c50c`，858 行）。这与 `RGS-PLAN-003 §2.6`「v0.1 阶段不必新建」直接抵触，且 REQ-038 至今仍是 v0.1。该分支**未合入 main**，建议在 Ulysses 拍板前维持未合入状态。

### B. 有基本设计但没有推进到详细设计 —— 0 项

37 份 BAS 中，4 份机械候选经复核全部证伪（§2.5）。**不存在**停在基本设计阶段的领域。

### C. 有详细设计但没有实施 —— 21 项

**前次报告的 18 与后续登记的 21 的差额已和解**：`agent/ulys-1/wt-d-trace` 分支的 RGS-REQ-005 §1.3 明确记载 `ISS-151〜171（21 项，含原审计 18 项 + ISS-127/128 已有的 DTL-022/023/024 共 3 项）`。**两个数字指同一集合，无遗漏、无重复计数。**

按可操作性分两类（这是该清单最有价值的切分）：

**C-1　设计正确状态 —— 4 项，不应视为缺陷**

| 项 | 受控原因 |
|---|---|
| DTL-011 智能决策层 | 设计上默认关闭 / 未授权实施 |
| DTL-031 ClusterOps PFAU | §1.1 标注「草案・待具名人类审批・不得作为实施授权」 |
| DTL-032〜035 Agent 运行时 | v0.2 待评审 |
| DTL-009 治理闭环 CI | RGS-REQ-013 §6 CR-005 待审批 |

**C-2　需后续 PH 推进 —— 17 项**

DTL-005 插件宿主 / DTL-008 Unity・UE FFI / DTL-013 大厅+频道聊天 / DTL-014 GSM 举报・黑名单・赛季 / DTL-017 AnalyticsStore / DTL-018 第三方 IdP（Apple・Google・Steam OAuth・OIDC）/ DTL-019 APNs・FCM SDK + 兑换码 / DTL-020 平台收据校验 SDK / DTL-021 GM 拓扑画布 / DTL-022 弹性容量 T3 多区域 / DTL-023 请求处理链中间件串联 / DTL-024 集群部署 DAG / DTL-025 反作弊 DSL / DTL-026 Glicko-2 核心公式 / DTL-027 CDN 自托管后端 / DTL-042 服务器生命周期 6 阶段 / SPEC-DTL-038-防丢包

本轮抽样复验（`grep -ril` over `crates/`，全部 **0 命中**，支持上述定性）：

```
glicko2_update | Glicko2 | glicko        -> 0
trait Plugin | PluginManager | PluginRegistry -> 0
oidc | OAuth | apple_sign | steam_auth    -> 0
anti_cheat | anticheat | cheat_signal     -> 0
lobby                                      -> 0
```

### D. 前次报告未覆盖、本轮新增的缺口 —— 4 类

| # | 缺口 | 性质 | 证据 |
|---|---|---|---|
| **D-1** | `docs/15-IPA-完全对齐438cmds/` 整族（REQ/BDD/DDD/TEST-*）游离于 RGS 可追溯性体系外 | 治理 | §3.2 |
| **D-2** | `document-registry.toml` 18 条中 8 条「待起草」标记陈旧 | 治理 | §3.3 |
| **D-3** | REQ-038 与 DTL-038 各存在两份不相关文档共用同一编号 | ID 治理 | §2.3 |
| **D-4** | 13 份文档缺头部「父文档」行，致自动对账产生假缺口 | 元数据 | §2.2 |

---

## 5. 建议处置（请 Ulysses 拍板）

| # | 建议 | 优先级 | 说明 |
|---|---|---|---|
| 1 | 修正 `document-registry.toml` 8 条陈旧 `planned` 状态 + batch 需求书 `:537-541` 状态表 | **P0** | 纯状态修正，零设计变更；不修正则后续所有差距统计不可信（D-2 / §3.3） |
| 2 | 为 §2.2 的 13 份文档补头部「父文档」行 | **P0** | 元数据补全；之后 `reqbas_audit.py` 可无假阳性运行（D-4） |
| 3 | REQ-038 / DTL-038 编号冲突改号 | P1 | 建议「防丢包」族改用未占用编号；改号是拍板事项，本轮未擅自执行（D-3） |
| 4 | 裁定 `docs/15-IPA-完全对齐438cmds/` 族与 RGS-REQ-004 可追溯性矩阵的挂接方式 | P1 | 两选一：纳入矩阵并映射 BDD→BAS / DDD→DTL，或明确声明为独立体系（D-1） |
| 5 | 拍板 REQ-038 防丢包 BAS 的立项时点 | P1 | 与 `agent/ulys-1/wt-a-bas38` 分支的处置绑定（§4.A） |
| 6 | 裁定 5 个 `agent/ulys-1/wt-*` 分支（~11,900 行文档）的去留 | P1 | 见 §6 |
| 7 | `reqbas_audit.py` 是否接入 `check-docs-consistency.sh` CI | P2 | `RGS-PLAN-003 §4.2` 原提议，本轮按其要求未擅自接入 |

---

## 6. 关于 5 个未合入分支的说明

2026-09-11 15:17 回合产出 5 个分支，**截至本文制定时全部未合入 `main`，也未合入当前工作分支**：

| 分支 | commit | 产出 |
|---|---|---|
| `agent/ulys-1/wt-a-bas38` | `4e7c50c` | BAS-038 + DTL-038-防丢包（与 PLAN-003 §2.6 抵触，见 §4.A） |
| `agent/ulys-1/wt-b-extreq` | `c541055` | 8 个扩展域 REQ/BAS/DTL 共 24 份新文档 |
| `agent/ulys-1/wt-c-testdocs` | `728e967` | 7 份 DTL 测试设计 238 TC |
| `agent/ulys-1/wt-d-trace` | `de1e4ec` | RGS-REQ-004/005 v3.11→v3.12 |
| `agent/ulys-1/wt-e-gapspec` | `04c21d3` | 差距扫描报告 v0.1 |

这批产出**基于本文 §1.1 已否定的配对方法**得出的差距清单。其中 wt-d 的 ISS-151〜171 登记经本轮核对内容成立（§4.C），wt-a 的立项时点存疑（§4.A），其余三份本轮未逐行复核。**本轮不合并、不推送任何分支**——合并与否是拍板事项。

---

## 7. 追补（2026-09-13）：执行 §5 建议 #1/#2（P0），并更正 #1 的一处误判

本轮只执行 §5 中标 **P0** 的两项（纯元数据/状态修正，零设计变更），**P1（#3〜#6）与 P2（#7）未动**，含 5 个 `wt-*` 分支——仍保持未合入。

### 7.1 建议 #2（补 13 份缺「父文档」行）—— 已执行 7 份，5 份保留待拍板

`scripts/reqbas_audit.py` 存在两处遗留 bug（`collect()` 已改为返回 `(docs, skipped)` 二元组、`build_report()` 已改为三参数，但 `main()` 仍按旧签名调用）导致脚本无法运行；已修复。修复后 `no-parent-row` 由 13 降为 12（`RGS-DTL-044` 因头部改用「主文档」而非「父文档」键被误判缺失——已将「主文档」加入 `PARENT_KEYS` 识别集，非改动该文件正文）。

对剩余 12 份，逐份核查文档正文能否**无歧义**确定单一父文档，能确定的才补行：

| 文件 | 处置 | 依据 |
|---|---|---|
| `RGS-REQ-001` | 补「父文档：无（L1 顶层需求）」 | L1 需求本无父，无歧义 |
| `RGS-REQ-100` | 补「父文档：无（L1 顶层需求）」 | 同上 |
| `RGS-BAS-100` | 补「父文档：RGS-REQ-100」 | 「关联文档」行原生标注「RGS-REQ-100（需求定义书）」，非猜测 |
| `RGS-DTL-100/101/102` | 各补「父文档：RGS-BAS-100」 | 三份「关联文档」行均原生标注「RGS-BAS-100（基本）」，且其余关联项均明确标注「同侪」而非父子 |
| `RGS-DTL-038`（卡牌） | 补「父文档：RGS-REQ-038」 | 头部「上游依据」+ 正文 §1.1「本文档上游」均单一、明确指向 RGS-REQ-038（卡牌那份，非防丢包那份） |
| `RGS-REQ-038`（卡牌） | **不补**，保留待拍板 | 头部「上游依据」同时列 RGS-REQ-001 与 RGS-REQ-013 两份上游，无法无歧义确定单一父文档，且该文件正卷入 §2.3/D-3 编号冲突（P1 #3），不应在改号拍板前抢先写入结构化父引用 |
| `RGS-DTL-036-REVIEW` | **不补** | 评审反馈件，文件名误用 DTL 前缀，问题是分类/命名而非缺元数据 |
| `RGS-BAS-FLOW-STANDARD` / `RGS-BAS-MERMAID-VERIFY` | **不补** | 同上，流程标准件误用 BAS 前缀 |
| `RGS-REQ-2026-09-04` | **不补** | 属 `docs/15-IPA-完全对齐438cmds/` 平行文档族（D-1），未挂接 RGS-* 可追溯性体系是 P1 #4 拍板事项，不应在挂接方式确定前擅自写入父引用 |

修复后脚本另新增「语料核对」自检（按文件名前缀总数 = 已解析 + 已跳过，四层均已核验平衡），并把 9 份 `SPEC-CROSS-*`／`SPEC-000`／`SPEC-26Batch-REVIEW` 及 1 份 `RGS-BAS-003-mTLS-决策补充` 计入独立 `skipped` 桶（文件名声明层级但编号正则解析失败）——这些是无单一父 DTL 的横向规范/决策补充件，**未**扩大 `FILENAME_RE` 把它们强行并入 REQ→BAS→DTL→SPEC 链路（那样会人为制造约 7 个假缺口）。

### 7.2 建议 #1（8 条陈旧 `planned` 状态）—— 更正：`document-registry.toml` 部分不属实，只改了 batch 需求书状态表

复查发现 §3.3 对 `document-registry.toml` 的定性有误，**未按原计划修改该文件**：

`document-registry.toml:10-16` 自身元数据注释明确记载：这 14 条 `[[planned_document]]`（含本文档提到的 8 条 + 另 6 条 TST addendum）**本就已落盘 v0.1 草案**，`status = "planned"` 是**设计上的常态值**——该注册表模式里不存在"已批准"状态，`planned` 表示"允许受控引用 + 待具名审批"，文件是否已在磁盘上不是这个字段要表达的信息；生产基线的转正由 `RGS-QA-001` / `RGS-IMPL-001` 的 G-CODE-06 独立把关。`RGS-REQ-004` 附件 C §3.9/3.10 亦印证：这批文档已完成具名审批签字（version 0.1→0.2），但 `status` 依规仍保持 `planned` 不变。**前次报告把这个字段误读成"文件缺失标记"，据此建议的"修正 8 条陈旧状态"不成立，本轮未执行。**

（附带核查：另外 6 条被前次报告认定"确认磁盘无文件、真实待起草"的 TST addendum 中，`RGS-TST-UT-04-ADD2` / `RGS-TST-UT-02-ADD3` 2 条文件其实已存在——只是磁盘文件名比 `path` 字段多了 `-2026-09-07_v0.2` 版本后缀；`RGS-TST-ST-04-ADD2` / `RGS-TST-IT-04-ADD2` / `RGS-TST-ST-02-ADD3` / `RGS-TST-IT-02-ADD3` 4 条同理。按 7.2 上段结论，`status` 字段本就不因此需要改，**故本轮同样未改 `path`**——只在此记录以免后续误判为"未核实"。真正确认磁盘无文件的只有 `RGS-IFS-001` / `RGS-TST-001` / `RGS-LIC-001` 3 条。）

**唯一属实、已修正的是 `docs/12-工作流/RGS-BATCH-REQUIREMENTS-2026-09-01_v0.1.md` §0 关联行 + §0 三层对应表 + §10.1 状态表**：三份下游文档（BASIC-DESIGN / DETAILED-DESIGN / PLAN）确已落盘，PLAN 磁盘实际是 v0.2 非 v0.1，均已更正为已落盘状态并修正版本号。这份是普通 Markdown 表格，不像 `document-registry.toml` 有独立的治理语义，纯属陈旧记录。

### 7.3 本轮改动清单

- `scripts/reqbas_audit.py`：修复 `main()` 签名不匹配（脚本此前不可运行）；`PARENT_KEYS` 加入「主文档」；`render_md` 新增语料核对/跳过桶章节
- 7 份文档补「父文档」头部行（§7.1 表格前 7 行）
- `docs/12-工作流/RGS-BATCH-REQUIREMENTS-2026-09-01_v0.1.md`：3 处「待起草」更正为已落盘
- 清理 `_miss.txt`（上一回合调试脚本时的散落中间产物，非交付物）
- `document-registry.toml`：**未改动**（§7.2 已说明原因）
- P1（编号改号 / IPA 族挂接 / REQ-038 立项时点 / 5 个 `wt-*` 分支去留）与 P2（CI 接入）：**未动**，仍待 Ulysses 拍板
